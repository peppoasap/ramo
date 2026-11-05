export type ConflictPart =
  | { kind: "text"; text: string }
  | {
      kind: "conflict";
      ours: string;
      theirs: string;
      base: string;
      raw: string;
    };
/** Preserve exact line endings and all text outside ordinary/diff3 conflict blocks. */
export function conflictParts(text: string): ConflictPart[] {
  const lines = text.match(/[^\n]*\n|[^\n]+$/g) || [];
  const parts: ConflictPart[] = [];
  let plain = "";
  for (let i = 0; i < lines.length; i++) {
    if (!lines[i].startsWith("<<<<<<<")) {
      plain += lines[i];
      continue;
    }
    const start = i;
    let split = -1,
      base = -1,
      end = -1;
    for (let j = i + 1; j < lines.length; j++) {
      if (lines[j].startsWith("<<<<<<<")) break;
      if (lines[j].startsWith("|||||||")) base = j;
      if (lines[j].startsWith("=======")) split = j;
      if (lines[j].startsWith(">>>>>>>")) {
        end = j;
        break;
      }
    }
    if (split < 0 || end < split || (base >= 0 && base > split)) {
      plain += lines[i];
      continue;
    }
    if (plain) {
      parts.push({ kind: "text", text: plain });
      plain = "";
    }
    parts.push({
      kind: "conflict",
      ours: lines.slice(start + 1, base >= 0 ? base : split).join(""),
      base: base >= 0 ? lines.slice(base + 1, split).join("") : "",
      theirs: lines.slice(split + 1, end).join(""),
      raw: lines.slice(start, end + 1).join(""),
    });
    i = end;
  }
  if (plain) parts.push({ kind: "text", text: plain });
  return parts;
}
export function chooseConflict(
  text: string,
  index: number,
  choice: "ours" | "theirs" | "both",
): string {
  let current = -1;
  return conflictParts(text)
    .map((part) => {
      if (part.kind === "text") return part.text;
      if (++current !== index) return part.raw;
      return choice === "both" ? part.ours + part.theirs : part[choice];
    })
    .join("");
}
export const hasConflictMarkers = (text: string) =>
  /^(<<<<<<<|=======|>>>>>>>|\|\|\|\|\|\|\|)/m.test(text);

export function parseResolutionProposal(text: string): {
  explanation: string;
  resolution: string | null;
} {
  const value = JSON.parse(text);
  if (
    !value ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    typeof value.explanation !== "string" ||
    !(value.resolution === null || typeof value.resolution === "string") ||
    Object.keys(value).some(
      (key) => !["explanation", "resolution"].includes(key),
    )
  )
    throw new Error(
      "AI returned an invalid resolution proposal. The editor was left unchanged.",
    );
  if (
    value.explanation.length > 8000 ||
    (typeof value.resolution === "string" &&
      (value.resolution.length > 32768 ||
        value.resolution.includes("\0") ||
        hasConflictMarkers(value.resolution)))
  )
    throw new Error(
      "AI proposal is oversized or still contains conflict markers. The editor was left unchanged.",
    );
  return value;
}

/** Collect by source line number, preserving order, duplicates and line endings. */
export function selectedSourceLines(
  text: string,
  selected: readonly number[],
): string {
  const lines = text.match(/[^\n]*\n|[^\n]+$/g) || [];
  return Array.from(new Set(selected))
    .filter((n) => Number.isInteger(n) && n > 0)
    .sort((a, b) => a - b)
    .map(
      (n) =>
        lines[n - 1] ??
        (n === lines.length + 1 && text.endsWith("\n")
          ? text.includes("\r\n")
            ? "\r\n"
            : "\n"
          : ""),
    )
    .join("");
}
export function replaceConflictText(
  text: string,
  index: number,
  replacement: string,
): string {
  const parts = conflictParts(text);
  let current = -1;
  return parts
    .map((part, position) => {
      if (part.kind === "text") return part.text;
      if (++current !== index) return part.raw;
      return (
        replacement +
        (replacement &&
        !replacement.endsWith("\n") &&
        position < parts.length - 1
          ? text.includes("\r\n")
            ? "\r\n"
            : "\n"
          : "")
      );
    })
    .join("");
}

import { conflictParts } from "./conflicts";
export interface LineRange {
  from: number;
  to: number;
}
const newlineCount = (value: string) => value.split("\n").length - 1;
/** One-based editor lines, including markers. Never infer an edited/malformed block. */
export function conflictLocations(value: string) {
  let line = 1;
  return conflictParts(value).flatMap((part) => {
    const from = line;
    const text = part.kind === "text" ? part.text : part.raw;
    line += newlineCount(text);
    return part.kind === "text"
      ? []
      : [{ part, range: { from, to: line - (text.endsWith("\n") ? 1 : 0) } }];
  });
}
/** Locate whole source lines only when there is exactly one match. Repeated code
 * is ambiguous; leave it unlinked rather than jump to an invented location. */
export function sourceConflictRange(
  source: string | null,
  fragment: string,
): LineRange | null {
  if (!source || !fragment) return null;
  const needle = fragment.replace(/\r?\n$/, "");
  if (!needle) return null;
  let offset = 0,
    found = -1;
  while (offset <= source.length) {
    const at = source.indexOf(needle, offset);
    if (at < 0) break;
    const end = at + needle.length;
    if (
      (at === 0 || source[at - 1] === "\n") &&
      (end === source.length ||
        source[end] === "\n" ||
        source.slice(end, end + 2) === "\r\n")
    ) {
      if (found >= 0) return null;
      found = at;
    }
    offset = at + 1;
  }
  if (found < 0) return null;
  const from = 1 + newlineCount(source.slice(0, found));
  return { from, to: from + newlineCount(needle) };
}
export function rangeLabel(range: LineRange | null) {
  return range
    ? range.from === range.to
      ? `Line ${range.from}`
      : `Lines ${range.from}–${range.to}`
    : "Location unavailable";
}

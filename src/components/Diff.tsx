import { useMemo } from "react";
export function diffLines(text: string) {
  const combined = /^(?:diff --cc |diff --combined |@@@)/m.test(text);
  let old = 0,
    next = 0,
    hunk = -1;
  return text.split("\n").map((line) => {
    // Combined conflicts have multiple parent columns, not two-sided offsets.
    if (combined)
      return {
        line,
        type: line.startsWith("@@@") ? "hunk" : "meta",
        old: "",
        next: "",
        hunkIndex: null,
      };
    const header = /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/.exec(line);
    if (header) {
      old = Number(header[1]);
      next = Number(header[2]);
      return { line, type: "hunk", old: "", next: "", hunkIndex: ++hunk };
    }
    if (/^(diff |index |--- |\+\+\+ )/.test(line))
      return { line, type: "meta", old: "", next: "", hunkIndex: null };
    if (line.startsWith("+"))
      return {
        line,
        type: "add",
        old: "",
        next: String(next++),
        hunkIndex: null,
      };
    if (line.startsWith("-"))
      return {
        line,
        type: "remove",
        old: String(old++),
        next: "",
        hunkIndex: null,
      };
    if (line.startsWith(" "))
      return {
        line,
        type: "",
        old: String(old++),
        next: String(next++),
        hunkIndex: null,
      };
    return { line, type: "meta", old: "", next: "", hunkIndex: null };
  });
}
export function Diff({
  text,
  loading,
  hunkAction,
}: {
  text: string;
  loading: boolean;
  hunkAction?: {
    label: string;
    disabled: boolean;
    run: (index: number) => void;
  };
}) {
  const lines = useMemo(() => diffLines(text), [text]);
  if (loading)
    return (
      <div className="empty-inline" role="status">
        Reading diff…
      </div>
    );
  if (!text)
    return (
      <div className="empty-inline">
        <p>Select a changed file to inspect its diff.</p>
      </div>
    );
  return (
    <div className="diff" tabIndex={0} aria-label="Unified diff">
      {lines.map((row, i) => (
        <div key={i} className={`diff-line ${row.type}`}>
          {row.hunkIndex !== null && hunkAction && (
            <button
              className="hunk-action"
              disabled={hunkAction.disabled}
              aria-label={`${hunkAction.label} ${row.hunkIndex + 1}`}
              title="Changes only this text block in the index. The working file is untouched."
              onClick={() => hunkAction.run(row.hunkIndex!)}
            >
              {hunkAction.label} {row.hunkIndex + 1}
            </button>
          )}
          <span className="line-number" aria-hidden="true">
            {row.old}
          </span>
          <span className="line-number" aria-hidden="true">
            {row.next}
          </span>
          <code>{row.line || " "}</code>
        </div>
      ))}
    </div>
  );
}

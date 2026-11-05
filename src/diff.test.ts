import { expect, it } from "vitest";
import { diffLines } from "./components/Diff";
it("numbers ordinary unified hunks against their real offsets", () => {
  const rows = diffLines("@@ -12,2 +23,2 @@\n unchanged\n-before\n+after");
  expect(rows[1]).toMatchObject({ old: "12", next: "23" });
  expect(rows[2]).toMatchObject({ old: "13", next: "" });
  expect(rows[3]).toMatchObject({ old: "", next: "24" });
});
it("preserves a combined conflict diff without inventing two-sided line numbers", () => {
  const text =
    "diff --cc file.txt\n@@@ -1,1 -1,1 +1,5 @@@\n++<<<<<<< HEAD\n +ours\n++=======\n+ theirs\n++>>>>>>> side";
  const rows = diffLines(text);
  expect(rows.map((row) => row.line).join("\n")).toBe(text);
  expect(rows.every((row) => row.old === "" && row.next === "")).toBe(true);
});

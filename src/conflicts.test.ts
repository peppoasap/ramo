import { describe, expect, it } from "vitest";
import { chooseConflict, conflictParts, hasConflictMarkers } from "./conflicts";
describe("Guided conflict choices", () => {
  it("preserves CRLF and nonconflicting text while choosing one block", () => {
    const text =
      "before\r\n<<<<<<< HEAD\r\nours\r\n=======\r\ntheirs\r\n>>>>>>> side\r\nafter\r\n";
    expect(chooseConflict(text, 0, "both")).toBe(
      "before\r\nours\r\ntheirs\r\nafter\r\n",
    );
  });
  it("omits diff3 ancestor text and leaves other blocks unresolved", () => {
    const block =
      "<<<<<<< HEAD\nours\n||||||| base\nancestor\n=======\ntheirs\n>>>>>>> side\n";
    expect(chooseConflict(block + "middle\n" + block, 1, "theirs")).toBe(
      block + "middle\ntheirs\n",
    );
    expect(conflictParts(block)[0]).toMatchObject({
      ours: "ours\n",
      base: "ancestor\n",
      theirs: "theirs\n",
    });
  });
  it("retains malformed markers and blocks saving until manually resolved", () => {
    const text = "<<<<<<< HEAD\nunfinished\n";
    expect(chooseConflict(text, 0, "ours")).toBe(text);
    expect(hasConflictMarkers(text)).toBe(true);
    expect(hasConflictMarkers("resolved\n")).toBe(false);
  });
});

import { canCommit, isConflicted } from "./domain";
import type { FileChange, Snapshot } from "./domain";
it("recognizes every unmerged porcelain pair without misclassifying staged adds", () => {
  const file = (pair: string): FileChange => ({
    path: "synthetic.txt",
    original: null,
    index: pair[0],
    worktree: pair[1],
  });
  for (const pair of ["DD", "AU", "UD", "UA", "DU", "AA", "UU"])
    expect(isConflicted(file(pair))).toBe(true);
  for (const pair of ["A ", " M", "??", "M "])
    expect(isConflicted(file(pair))).toBe(false);
});
it("allows unchanged-tree merge completion but refuses conflicts and sequencer commits", () => {
  const s = {
    files: [],
    operation: "Merge in progress",
  } as unknown as Snapshot;
  expect(canCommit(s)).toBe(true);
  expect(canCommit({ ...s, operation: null })).toBe(false);
  expect(canCommit({ ...s, operation: "Rebase in progress" })).toBe(false);
  expect(
    canCommit({
      ...s,
      files: [{ path: "a", original: null, index: "U", worktree: "U" }],
    }),
  ).toBe(false);
});

import { parseResolutionProposal } from "./conflicts";
it("accepts only reviewable structured AI proposals and preserves file whitespace", () => {
  expect(
    parseResolutionProposal(
      JSON.stringify({
        explanation: "Both sides retained; verify behavior.",
        resolution: "line\n\n",
      }),
    ).resolution,
  ).toBe("line\n\n");
  expect(
    parseResolutionProposal(
      '{"explanation":"Deletion suggested","resolution":null}',
    ).resolution,
  ).toBeNull();
  expect(() =>
    parseResolutionProposal(
      '{"explanation":"Bad","resolution":"<<<<<<< HEAD"}',
    ),
  ).toThrow();
  expect(() =>
    parseResolutionProposal(
      '{"explanation":"Bad","resolution":"ok","command":"rm -rf"}',
    ),
  ).toThrow();
  expect(() => parseResolutionProposal("```json\n{}\n```")).toThrow();
});

import { selectedSourceLines, replaceConflictText } from "./conflicts";
it("selects individual source lines in source order without collapsing duplicates", () => {
  expect(selectedSourceLines("first\r\nsame\r\nsame\r\nlast", [3, 2, 2])).toBe(
    "same\r\nsame\r\n",
  );
  expect(selectedSourceLines("first\nlast", [2])).toBe("last");
  expect(selectedSourceLines("first\n", [2])).toBe("\n");
});
it("replaces only the active conflict with selected rows and preserves surrounding edits", () => {
  const block = "<<<<<<< HEAD\none\ntwo\n=======\nthree\nfour\n>>>>>>> side\n";
  const value =
    "manual before\n" + block + "manual middle\n" + block + "manual after\n";
  const selected =
    selectedSourceLines("one\ntwo\n", [2]) +
    selectedSourceLines("three\nfour\n", [1]);
  expect(replaceConflictText(value, 1, selected)).toBe(
    "manual before\n" + block + "manual middle\ntwo\nthree\nmanual after\n",
  );
});
it("does not join a selected final source line to unrelated following code", () => {
  const block = "<<<<<<< HEAD\none\n=======\ntwo\n>>>>>>> side\n";
  expect(replaceConflictText(block + "after\n", 0, "last")).toBe(
    "last\nafter\n",
  );
  expect(replaceConflictText(block, 0, "last")).toBe("last");
});

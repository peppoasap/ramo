import { expect, it } from "vitest";
import { supportsHunks, reviewedPatchHash } from "./hunks";
import { diffLines } from "./components/Diff";
const file = { path: "notes.txt", original: null, index: " ", worktree: "M" };
const patch =
  "diff --git a/notes.txt b/notes.txt\nindex aaaaaaa..bbbbbbb 100644\n--- a/notes.txt\n+++ b/notes.txt\n@@ -1 +1 @@\n-before\n+after\n@@ -20 +20 @@\n-old\n+new\n";
it("indexes only real unified headers, preserving code that resembles hunk syntax", () => {
  expect(
    diffLines(patch)
      .filter((row) => row.hunkIndex !== null)
      .map((row) => row.hunkIndex),
  ).toEqual([0, 1]);
  expect(
    diffLines("+@@ -1 +1 @@\n").every((row) => row.hunkIndex === null),
  ).toBe(true);
  expect(
    diffLines("diff --cc x\n@@@ -1,1 -1,1 +1,1 @@@\n").every(
      (row) => row.hunkIndex === null,
    ),
  ).toBe(true);
});
it("offers partial staging only for modified regular text previews", () => {
  expect(supportsHunks(file, patch, false)).toBe(true);
  expect(supportsHunks({ ...file, index: "M" }, patch, true)).toBe(true);
  expect(supportsHunks(file, patch.replace("100644", "120000"), false)).toBe(
    false,
  );
  expect(supportsHunks(file, patch.replace("100644", "160000"), false)).toBe(
    false,
  );
  for (const change of [
    { ...file, index: "?" },
    { ...file, index: "U" },
    { ...file, original: "old.txt" },
  ])
    expect(supportsHunks(change, patch, false)).toBe(false);
  for (const text of [
    "Binary files a/x and b/x differ\n",
    "new mode 100755\n" + patch,
    "x".repeat(262145),
  ])
    expect(supportsHunks(file, text, false)).toBe(false);
});
it("binds a selected hunk to the exact UTF-8 patch displayed", async () => {
  expect(await reviewedPatchHash("abc")).toBe(
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
  );
  expect(await reviewedPatchHash(patch)).not.toBe(
    await reviewedPatchHash(patch.replace("after", "è 🌱")),
  );
});

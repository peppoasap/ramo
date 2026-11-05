import { expect, it } from "vitest";
import { conflictLocations, sourceConflictRange } from "./conflictNavigation";
it("tracks current conflict lines after preceding resolutions and manual edits", () => {
  const block = "<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> side\n";
  const original = "before\n" + block + "middle\n" + block;
  expect(conflictLocations(original).map((c) => c.range)).toEqual([
    { from: 2, to: 6 },
    { from: 8, to: 12 },
  ]);
  expect(conflictLocations("manual\nedit\n" + block)[0].range).toEqual({
    from: 3,
    to: 7,
  });
  expect(conflictLocations("<<<<<<< unfinished\nours\n")).toEqual([]);
});
it("locates exact full lines, preserving Unicode, CRLF and missing final newline", () => {
  expect(
    sourceConflictRange("prefix\r\nαβ\r\nvalue\r\nend", "αβ\r\nvalue\r\n"),
  ).toEqual({ from: 2, to: 3 });
  expect(sourceConflictRange("before\nlast", "last\n")).toEqual({
    from: 2,
    to: 2,
  });
  expect(sourceConflictRange("oursSuffix\nprefixours\n", "ours\n")).toBeNull();
});
it("refuses ambiguous repeated source rows, absent and empty versions", () => {
  expect(sourceConflictRange("same\nother\nsame\n", "same\n")).toBeNull();
  expect(sourceConflictRange(null, "line\n")).toBeNull();
  expect(sourceConflictRange("line\n", "")).toBeNull();
  expect(sourceConflictRange("\n\n", "\n")).toBeNull();
});
it("maps diff3 base rows and counts final marker without an EOF newline", () => {
  const [block] = conflictLocations(
    "intro\r\n<<<<<<< HEAD\r\none\r\n||||||| base\r\nbase row\r\n=======\r\ntwo\r\n>>>>>>> side",
  );
  expect(block.range).toEqual({ from: 2, to: 8 });
  expect(sourceConflictRange("intro\r\nbase row\r\n", block.part.base)).toEqual(
    { from: 2, to: 2 },
  );
});

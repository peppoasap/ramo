import { describe, it, expect } from "vitest";
import { layoutGraph } from "./graph";
import type { Commit } from "./domain";
const c = (oid: string, parents: string[]): Commit => ({
  oid,
  parents,
  author: "",
  date: "",
  subject: "",
  refs: "",
});
describe("Git topology lanes", () => {
  it("keeps linear history in one lane", () => {
    const rows = layoutGraph([c("a", ["b"]), c("b", ["c"]), c("c", [])]);
    expect(rows.map((r) => r.lane)).toEqual([0, 0, 0]);
    expect(rows[2].edges).toEqual([]);
    expect(rows.map((r) => r.incoming)).toEqual([false, true, true]);
  });
  it("connects both merge parents and rejoins their common ancestor", () => {
    const rows = layoutGraph([
      c("merge", ["left", "right"]),
      c("left", ["base"]),
      c("right", ["base"]),
      c("base", []),
    ]);
    expect(rows[0].edges.map((e) => e.to)).toEqual([0, 1]);
    expect(
      rows[2].edges.some((e) => e.from === 1 && e.to === 0 && e.fromNode),
    ).toBe(true);
    expect(rows[3].edges).toEqual([]);
  });
  it("supports octopus merges and disconnected roots", () => {
    const rows = layoutGraph([
      c("a", ["b", "c", "d"]),
      c("b", []),
      c("c", []),
      c("d", []),
      c("other", []),
    ]);
    expect(new Set(rows[0].edges.map((e) => e.to)).size).toBe(3);
    expect(rows[4].edges).toEqual([]);
  });
});

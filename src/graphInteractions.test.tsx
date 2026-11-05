import { describe, it, expect } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { compactRefs, graphDrop } from "./graphInteractions";
import { History } from "./components/History";
import type { Snapshot, Commit } from "./domain";
const branch = { name: "feature", oid: "a".repeat(40), remote: false };
const current = { ...branch, name: "main", oid: "b".repeat(40) };
const commit: Commit = {
  oid: branch.oid,
  parents: ["c".repeat(40)],
  subject: "Synthetic change",
  author: "Synthetic",
  date: "2026-11-04",
  refs: "tag: v1, tag: v2",
};
const snapshot: Snapshot = {
  repo: {
    id: "synthetic",
    name: "Example",
    path: "/synthetic",
    trusted: false,
  },
  head: current.oid,
  branch: "main",
  upstream: null,
  ahead: null,
  behind: null,
  files: [],
  branches: [branch, current],
  worktrees: [],
  stashes: [],
  fingerprint: "reviewed",
  operation: null,
};
const context = "synthetic:reviewed";
describe("Graph gestures", () => {
  it("prefers current then local references without changing inputs", () => {
    const refs = [
      { ...branch, remote: true },
      branch,
      { ...branch, name: "main" },
    ];
    expect(compactRefs(refs, "main").map((b) => [b.name, b.remote])).toEqual([
      ["main", false],
      ["feature", false],
      ["feature", true],
    ]);
    expect(refs[0].remote).toBe(true);
  });
  it("plans a reviewed merge or cherry-pick, never executes it", () => {
    expect(
      graphDrop({ kind: "branch", branch }, null, snapshot, context, false)
        .confirmation?.action,
    ).toEqual({ type: "merge", name: "feature" });
    expect(
      graphDrop({ kind: "commit", commit }, null, snapshot, context, false)
        .confirmation?.action,
    ).toEqual({ type: "cherryPick", target: commit.oid });
  });
  it("rejects changed context, busy state, moved refs and unsafe mutation states", () => {
    for (const s of [
      { ...snapshot, operation: "Merge in progress" },
      { ...snapshot, head: null },
      { ...snapshot, branch: "Detached HEAD" },
      {
        ...snapshot,
        files: [
          { path: "synthetic", original: null, index: "M", worktree: " " },
        ],
      },
    ])
      expect(
        graphDrop({ kind: "commit", commit }, null, s, context, false)
          .confirmation,
      ).toBeUndefined();
    expect(
      graphDrop(
        { kind: "branch", branch },
        null,
        snapshot,
        "other:reviewed",
        false,
      ).error,
    ).toBeTruthy();
    expect(
      graphDrop({ kind: "branch", branch }, null, snapshot, context, true)
        .error,
    ).toBeTruthy();
    expect(
      graphDrop(
        { kind: "branch", branch: { ...branch, oid: "moved" } },
        null,
        snapshot,
        context,
        false,
      ).error,
    ).toBeTruthy();
    for (const b of [current, { ...branch, remote: true }])
      expect(
        graphDrop(
          { kind: "branch", branch: b },
          null,
          { ...snapshot, branches: [b] },
          context,
          false,
        ).error,
      ).toBeTruthy();
    for (const c of [
      { ...commit, parents: ["a", "b"] },
      { ...commit, oid: snapshot.head! },
    ])
      expect(
        graphDrop({ kind: "commit", commit: c }, null, snapshot, context, false)
          .error,
      ).toBeTruthy();
  });
  it("compares immutable pairs even with dirty files; refuses self comparison", () => {
    const target = { ...commit, oid: current.oid };
    expect(
      graphDrop(
        { kind: "commit", commit },
        target,
        { ...snapshot, operation: "Merge in progress" },
        context,
        false,
      ),
    ).toEqual({ comparison: { from: commit.oid, to: target.oid } });
    expect(
      graphDrop({ kind: "commit", commit }, commit, snapshot, context, false)
        .error,
    ).toBeTruthy();
  });
  it("shows one prioritized branch with a bounded overflow control and draggable commit", () => {
    const html = renderToStaticMarkup(
      <History
        branches={[branch, { ...branch, name: "main" }]}
        currentBranch="main"
        commits={[commit]}
        selected={null}
        onSelect={() => {}}
        filter=""
        onFilter={() => {}}
        onMore={() => {}}
        hasMore={false}
        busy={false}
        onBranchContext={() => {}}
        onCommitContext={() => {}}
        onGraphDrop={() => {}}
      />,
    );
    expect((html.match(/class="graph-ref  local/g) || []).length).toBe(1);
    expect(html).toContain("Branch: main");
    expect(html).toContain("Show all references: 2 branches, 2 tags");
    expect(html).toContain("+3");
    expect((html.match(/draggable="false"/g) || []).length).toBe(2);
  });
});

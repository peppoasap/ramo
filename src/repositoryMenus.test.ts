import { expect, it, vi } from "vitest";
import { branchMenu, commitMenu } from "./repositoryMenus";
import type { Snapshot, Commit } from "./domain";
const snapshot = {
  repo: { id: "test", name: "Meadow" },
  branch: "main",
  head: "a".repeat(40),
  operation: null,
  files: [],
  branches: [],
} as unknown as Snapshot;
const commands = () => ({
  action: vi.fn(),
  form: vi.fn(),
  confirm: vi.fn(),
  inspect: vi.fn(),
  copy: vi.fn(),
});
const commit: Commit = {
  oid: "b".repeat(40),
  parents: ["a".repeat(40)],
  refs: "",
  subject: "Synthetic feature",
  date: "",
  author: "Demo",
};
it("prevents history writes on a dirty or conflicted worktree while retaining inspection", () => {
  const dirty = {
    ...snapshot,
    files: [{ path: "keep.txt", original: null, index: " ", worktree: "M" }],
  };
  const items = commitMenu(commit, dirty, false, commands());
  expect(items.find((i) => i.label.startsWith("Cherry-pick"))?.disabled).toBe(
    true,
  );
  expect(items.find((i) => i.label.startsWith("Revert"))?.disabled).toBe(true);
  expect(items.find((i) => i.label === "Inspect commit")?.disabled).toBeFalsy();
  expect(
    branchMenu(
      { name: "side", remote: false, oid: commit.oid },
      dirty,
      false,
      commands(),
    ).find((i) => i.label.startsWith("Merge"))?.disabled,
  ).toBe(true);
});
it("distinguishes branch creation at a reviewed commit from switching current work", () => {
  const c = commands();
  commitMenu(commit, snapshot, false, c)
    .find((i) => i.label === "Create branch here…")!
    .run();
  expect(c.form).toHaveBeenCalledWith(
    expect.objectContaining({ kind: "createBranchAt", target: commit.oid }),
  );
  expect(c.action).not.toHaveBeenCalled();
  expect(
    commitMenu({ ...commit, parents: ["a", "c"] }, snapshot, false, c).find(
      (i) => i.label.startsWith("Cherry-pick"),
    )?.disabled,
  ).toBe(true);
});
it("protects current branch deletion and opens a tracking flow for remote refs", () => {
  const c = commands();
  expect(
    branchMenu(
      { name: "main", remote: false, oid: commit.oid },
      snapshot,
      false,
      c,
    ).find((i) => i.label.startsWith("Delete"))?.disabled,
  ).toBe(true);
  const remote = branchMenu(
    { name: "origin/topic/path", remote: true, oid: commit.oid },
    snapshot,
    false,
    c,
  );
  remote.find((i) => i.label.startsWith("Checkout"))!.run();
  expect(c.form).toHaveBeenCalledWith(
    expect.objectContaining({
      kind: "trackBranch",
      target: "origin/topic/path",
      value: "topic/path",
    }),
  );
});

it("offers worktree creation for free local branches while blocking occupied and interrupted ones", () => {
  const c = { ...commands(), worktree: vi.fn() };
  const branch = { name: "feature/free", remote: false, oid: commit.oid };
  const available = {
    ...snapshot,
    worktrees: [],
    files: [{ path: "keep.txt", original: null, index: "M", worktree: " " }],
  };
  const item = (state: Snapshot) =>
    branchMenu(branch, state, false, c).find(
      (i) => i.label === "Create worktree for branch…",
    )!;
  expect(item(available).disabled).toBeFalsy();
  item(available).run();
  expect(c.worktree).toHaveBeenCalledWith("feature/free");
  expect(c.action).not.toHaveBeenCalled();
  expect(
    item({
      ...available,
      worktrees: [{ branch: branch.name } as Snapshot["worktrees"][number]],
    }).disabled,
  ).toBe(true);
  expect(item({ ...available, operation: "merge" }).disabled).toBe(true);
  expect(
    branchMenu({ ...branch, remote: true }, available, false, c).some((i) =>
      i.label.startsWith("Create worktree"),
    ),
  ).toBe(false);
});

it("keeps explicit-object comparison available on dirty and interrupted repositories", () => {
  const c = { ...commands(), compare: vi.fn() };
  const state = {
    ...snapshot,
    operation: "merge",
    files: [{ path: "file", index: "U", worktree: "U", original: null }],
  };
  const branch = { name: "side", remote: false, oid: commit.oid };
  branchMenu(branch, state, false, c)
    .find((item) => item.label === "Compare revisions…")!
    .run();
  commitMenu(commit, state, false, c)
    .find((item) => item.label === "Compare revisions…")!
    .run();
  expect(c.compare).toHaveBeenNthCalledWith(1, commit.oid);
  expect(c.compare).toHaveBeenNthCalledWith(2, commit.oid);
  expect(c.action).not.toHaveBeenCalled();
});

it("checks out a local branch directly without an extra confirmation", () => {
  const c = commands();
  const branch = { name: "topic", remote: false, oid: commit.oid };
  const item = branchMenu(branch, snapshot, false, c).find(
    (i) => i.label === "Checkout branch",
  )!;
  expect(item.disabled).toBeFalsy();
  item.run();
  expect(c.action).toHaveBeenCalledWith({ type: "switch", name: "topic" });
  expect(c.confirm).not.toHaveBeenCalled();
  for (const [state, busy] of [
    [snapshot, true],
    [{ ...snapshot, operation: "Merge in progress" }, false],
    [{ ...snapshot, branch: "topic" }, false],
  ] as const) {
    expect(
      branchMenu(branch, state, busy, c).find(
        (i) => i.label === "Checkout branch",
      )?.disabled,
    ).toBe(true);
  }
});

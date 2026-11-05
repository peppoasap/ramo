import type {
  Action,
  Branch,
  Commit,
  Confirmation,
  Form,
  Snapshot,
} from "./domain";
import { isConflicted } from "./domain";
import type { MenuItem } from "./components/ContextMenu";
export interface MenuCommands {
  compare?: (oid: string) => void;
  worktree?: (branch: string) => void;
  action: (action: Action) => Promise<unknown>;
  form: (form: Form) => void;
  confirm: (confirmation: Confirmation) => void;
  inspect: (oid: string) => void;
  copy: (value: string) => void;
}
export function branchMenu(
  branch: Branch,
  snapshot: Snapshot,
  busy: boolean,
  commands: MenuCommands,
): MenuItem[] {
  const blocked =
    busy || !!snapshot.operation || snapshot.files.some(isConflicted);
  const current = !branch.remote && branch.name === snapshot.branch;
  const dirty = !!snapshot.files.length;
  const items: MenuItem[] = [
    { label: "Show branch in graph", run: () => commands.inspect(branch.oid) },
    { label: "Copy branch name", run: () => commands.copy(branch.name) },
    ...(commands.compare
      ? [
          {
            label: "Compare revisions…",
            run: () => commands.compare!(branch.oid),
          },
        ]
      : []),
  ];
  if (branch.remote)
    return [
      ...items,
      {
        label: "Checkout as local tracking branch…",
        disabled: blocked || dirty,
        run: () =>
          commands.form({
            kind: "trackBranch",
            title: `Track ${branch.name}`,
            target: branch.name,
            value: branch.name.slice(branch.name.indexOf("/") + 1),
          }),
      },
    ];
  return [
    ...items,
    ...(commands.worktree
      ? [
          {
            label: "Create worktree for branch…",
            disabled:
              blocked ||
              !snapshot.head ||
              snapshot.worktrees.some((tree) => tree.branch === branch.name),
            run: () => commands.worktree!(branch.name),
          },
        ]
      : []),
    {
      label: "Checkout branch",
      disabled: blocked || current,
      run: () => commands.action({ type: "switch", name: branch.name }),
    },
    {
      label: `Merge into ${snapshot.branch}…`,
      disabled:
        blocked ||
        dirty ||
        current ||
        !snapshot.head ||
        snapshot.branch === "Detached HEAD",
      run: () =>
        commands.confirm({
          action: { type: "merge", name: branch.name },
          title: "Merge local branch",
          description: `Merge ${branch.name} into ${snapshot.branch} in ${snapshot.repo.name}? Fast-forward when possible; otherwise create a merge commit. Conflicts remain for review. Push is separate.`,
        }),
    },
    {
      label: "Rename branch…",
      disabled: blocked,
      run: () =>
        commands.form({
          kind: "renameBranch",
          title: `Rename ${branch.name}`,
          target: branch.name,
          value: branch.name,
        }),
    },
    {
      label: "Delete merged branch…",
      disabled: blocked || current,
      danger: true,
      run: () =>
        commands.confirm({
          action: { type: "deleteBranch", name: branch.name },
          title: "Delete local branch",
          description: `Delete ${branch.name} in ${snapshot.repo.name}? Git refuses unmerged work and branches checked out in another worktree.`,
        }),
    },
  ];
}
export function commitMenu(
  commit: Commit,
  snapshot: Snapshot,
  busy: boolean,
  commands: MenuCommands,
): MenuItem[] {
  const blocked =
    busy || !!snapshot.operation || snapshot.files.some(isConflicted);
  const cleanRequired = blocked || !!snapshot.files.length;
  const sequenceBlocked =
    cleanRequired ||
    commit.parents.length > 1 ||
    !snapshot.head ||
    snapshot.branch === "Detached HEAD";
  return [
    { label: "Inspect commit", run: () => commands.inspect(commit.oid) },
    ...(commands.compare
      ? [
          {
            label: "Compare revisions…",
            run: () => commands.compare!(commit.oid),
          },
        ]
      : []),
    { label: "Copy commit SHA", run: () => commands.copy(commit.oid) },
    { label: "Copy commit subject", run: () => commands.copy(commit.subject) },
    {
      label: "Create branch here…",
      disabled: blocked,
      run: () =>
        commands.form({
          kind: "createBranchAt",
          title: `Create branch at ${commit.oid.slice(0, 7)}`,
          target: commit.oid,
          value: "",
        }),
    },
    {
      label: "Create tag here…",
      disabled: blocked,
      run: () =>
        commands.form({
          kind: "createTag",
          title: `Create tag at ${commit.oid.slice(0, 7)}`,
          target: commit.oid,
          value: "",
        }),
    },
    {
      label: "Checkout commit (detached HEAD)…",
      disabled: cleanRequired,
      run: () =>
        commands.confirm({
          action: { type: "checkoutCommit", target: commit.oid },
          title: "Checkout commit",
          description: `Checkout ${commit.oid.slice(0, 12)} in ${snapshot.repo.name}? HEAD becomes detached. Create a branch before keeping new work.`,
        }),
    },
    {
      label: "Cherry-pick onto current branch…",
      disabled: sequenceBlocked,
      run: () =>
        commands.confirm({
          action: { type: "cherryPick", target: commit.oid },
          title: "Cherry-pick commit",
          description: `Apply ${commit.oid.slice(0, 12)} (${commit.subject}) onto ${snapshot.branch}? This creates a local commit. Conflicts stay available for review; resolve, continue, skip or abort from the operation bar.`,
        }),
    },
    {
      label: "Revert with a new commit…",
      disabled: sequenceBlocked,
      run: () =>
        commands.confirm({
          action: { type: "revert", target: commit.oid },
          title: "Revert commit",
          description: `Reverse ${commit.oid.slice(0, 12)} (${commit.subject}) on ${snapshot.branch} with a new local commit? Existing history is retained. Conflicts stay available for review; resolve, continue, skip or abort from the operation bar.`,
        }),
    },
  ];
}

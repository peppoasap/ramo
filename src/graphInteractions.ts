import type { Branch, Commit, Confirmation, Snapshot } from "./domain";
export type GraphSource =
  { kind: "branch"; branch: Branch } | { kind: "commit"; commit: Commit };
export function compactRefs(branches: Branch[], current: string) {
  return [...branches].sort(
    (a, b) =>
      Number(!b.remote && b.name === current) -
        Number(!a.remote && a.name === current) ||
      Number(a.remote) - Number(b.remote) ||
      a.name.localeCompare(b.name),
  );
}
export function graphDrop(
  source: GraphSource,
  target: Commit | null,
  snapshot: Snapshot,
  context: string,
  busy: boolean,
): {
  confirmation?: Confirmation;
  comparison?: { from: string; to: string };
  error?: string;
} {
  if (context !== `${snapshot.repo.id}:${snapshot.fingerprint}` || busy)
    return { error: "Repository changed during the drag. Try again." };
  const oid = source.kind === "branch" ? source.branch.oid : source.commit.oid;
  if (
    source.kind === "branch" &&
    !snapshot.branches.some(
      (b) =>
        b.name === source.branch.name &&
        b.remote === source.branch.remote &&
        b.oid === oid,
    )
  )
    return { error: "This branch changed. Refresh and try again." };
  if (target)
    return oid === target.oid
      ? { error: "Choose a different commit to compare." }
      : { comparison: { from: oid, to: target.oid } };
  if (
    snapshot.operation ||
    snapshot.files.length ||
    !snapshot.head ||
    snapshot.branch === "Detached HEAD"
  )
    return {
      error:
        "A clean working tree and an attached branch are required. Finish the current operation first.",
    };
  if (source.kind === "branch") {
    if (source.branch.remote || source.branch.name === snapshot.branch)
      return {
        error: "Drag another local branch to merge into the current branch.",
      };
    return {
      confirmation: {
        title: "Merge local branch",
        description: `Merge ${source.branch.name} into ${snapshot.branch}? Fast-forward when possible; otherwise create a merge commit. Conflicts remain available in Changes. Push is separate.`,
        action: { type: "merge", name: source.branch.name },
      },
    };
  }
  if (source.commit.parents.length > 1 || oid === snapshot.head)
    return {
      error:
        "Choose a non-merge commit other than the current HEAD to cherry-pick.",
    };
  return {
    confirmation: {
      title: "Cherry-pick commit",
      description: `Apply ${oid.slice(0, 12)} (${source.commit.subject}) onto ${snapshot.branch}? This creates a local commit. Conflicts can be resolved, continued or aborted from Changes.`,
      action: { type: "cherryPick", target: oid },
    },
  };
}

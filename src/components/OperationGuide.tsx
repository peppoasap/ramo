import { Warning } from "@phosphor-icons/react";
import type { Confirmation, Snapshot } from "../domain";
import { isConflicted, isStaged } from "../domain";
export function OperationGuide({
  snapshot,
  busy,
  review,
  confirm,
}: {
  snapshot: Snapshot;
  busy: boolean;
  review: () => void;
  confirm: (value: Confirmation) => void;
}) {
  const conflicts = snapshot.files.filter(isConflicted).length;
  if (!snapshot.operation && !conflicts) return null;
  const merge = snapshot.operation === "Merge in progress";
  const kind =
    snapshot.operation === "Cherry-pick in progress"
      ? "cherryPick"
      : snapshot.operation === "Revert in progress"
        ? "revert"
        : null;
  const label = kind === "cherryPick" ? "cherry-pick" : "revert";
  const unstaged = snapshot.files.some(
    (file) => file.index !== "?" && file.worktree !== " ",
  );
  return (
    <>
      <div className="operation">
        <Warning size={16} />
        {snapshot.operation || "Unresolved conflicts"}.{" "}
        {merge
          ? "Resolve files, review the staged result, then Complete merge."
          : kind
            ? "Resolve files, review the staged result, then Continue."
            : !snapshot.operation
              ? "Resolve files in Changes, review every staged file, then create a normal commit to keep the result. Any retained stash stays saved; inspect it separately before applying or removing it."
              : "Review files here; continue or abort this operation using Git outside Ramo."}
      </div>
      <div className="merge-guide">
        <span>
          {conflicts} unresolved {conflicts === 1 ? "file" : "files"} ·{" "}
          {snapshot.files.filter(isStaged).length} staged{" "}
          {unstaged && !conflicts ? "· tracked edits still need review" : ""}
        </span>
        <button disabled={busy} onClick={review}>
          Review conflicts and result
        </button>
        {merge && (
          <button
            disabled={busy}
            onClick={() =>
              confirm({
                action: { type: "abortMerge" },
                title: "Abort merge",
                description: `Abort the merge in ${snapshot.repo.name} and restore its pre-merge state? Resolution edits will be discarded. Git may refuse if it cannot preserve other work. Remote history is unchanged.`,
              })
            }
          >
            Abort merge…
          </button>
        )}
        {kind && (
          <>
            <button
              className="primary"
              disabled={busy || !!conflicts || unstaged}
              onClick={() =>
                confirm({
                  action: {
                    type: "sequence",
                    operation: kind,
                    step: "continue",
                  },
                  title: `Continue ${label}`,
                  description: `Continue the ${label} on ${snapshot.branch} in ${snapshot.repo.name}? Git commits the entire reviewed index with its recorded message and advances any remaining commits. More conflicts may pause it again. Push is separate.`,
                })
              }
            >
              Continue {label}…
            </button>
            <button
              disabled={busy}
              onClick={() =>
                confirm({
                  action: { type: "sequence", operation: kind, step: "skip" },
                  title: `Skip current ${label} commit`,
                  description: `Skip the current commit in this ${label}? Git discards its in-progress resolution edits and advances to the next commit, if any. Review your result first. This does not push.`,
                })
              }
            >
              Skip current commit…
            </button>
            <button
              disabled={busy}
              onClick={() =>
                confirm({
                  action: { type: "sequence", operation: kind, step: "abort" },
                  title: `Abort ${label}`,
                  description: `Abort the ${label} in ${snapshot.repo.name} and return to the state before the sequence began? Resolution edits and commits made by this sequence will be discarded locally. Git may refuse if HEAD changed independently. This does not change remote history.`,
                })
              }
            >
              Abort {label}…
            </button>
          </>
        )}
      </div>
    </>
  );
}

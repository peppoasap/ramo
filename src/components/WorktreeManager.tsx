import { useState } from "react";
import type { Action, Worktree } from "../domain";
import { Dialog } from "./Dialog";
export function WorktreeManager({
  tree,
  opened,
  error,
  run,
  close,
}: {
  tree: Worktree;
  opened: boolean;
  error: string;
  run: (action: Action) => Promise<"done" | "trust" | "failed">;
  close: () => void;
}) {
  const [review, setReview] = useState<Action | null>(null);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState("");
  return (
    <Dialog title="Manage worktree" locked={busy} onClose={close}>
      <p>
        <strong>{tree.branch}</strong>
        <br />
        <code>{tree.path}</code>
        <br />
        HEAD: <code>{tree.head.slice(0, 12)}</code>
      </p>
      {notice && <p role="status">{notice}</p>}
      {error && (
        <p role="alert" className="resolution-error">
          {error}
        </p>
      )}
      {tree.primary ? (
        <p>The primary checkout cannot be locked or removed here.</p>
      ) : review ? (
        <>
          <p>
            {review.type === "removeWorktree"
              ? "Remove this checkout folder from disk? Its branch and commits are retained. Removal refuses modified, untracked or ignored files, nested repositories, hidden index flags and active Git operations. Stop other processes using this checkout before confirming; filesystem checks and removal are not atomic. There is no in-app undo."
              : tree.locked
                ? "Unlock this checkout so Git can remove or prune it? Files remain unchanged."
                : "Lock this checkout to protect it from Git removal and pruning? Files remain unchanged."}
          </p>
          <div className="dialog-actions">
            <button
              disabled={busy}
              onClick={() => {
                setReview(null);
                setNotice("");
              }}
            >
              Back
            </button>
            <button
              className="primary"
              disabled={busy}
              onClick={async () => {
                setBusy(true);
                setNotice("");
                try {
                  const result = await run(review);
                  if (result === "done") close();
                  else
                    setNotice(
                      result === "trust"
                        ? "Enable writes for this session, then confirm again. Your review is retained."
                        : "Action did not finish. Review the repository error. Close and reopen this dialog after refreshing before retrying.",
                    );
                } finally {
                  setBusy(false);
                }
              }}
            >
              {busy
                ? "Working…"
                : review.type === "removeWorktree"
                  ? "Confirm removal"
                  : tree.locked
                    ? "Confirm unlock"
                    : "Confirm lock"}
            </button>
          </div>
        </>
      ) : (
        <>
          <p>
            {tree.locked
              ? "Locked: protected from Git removal and pruning."
              : "Unlocked"}
          </p>
          {opened && (
            <p>Close this checkout’s repository tab before removing it.</p>
          )}
          {tree.branch === "Detached HEAD" && (
            <p>
              Create a branch for this detached checkout before removing it.
            </p>
          )}
          <div className="dialog-actions">
            <button
              onClick={() =>
                setReview({
                  type: "worktreeLock",
                  path: tree.path,
                  locked: !tree.locked,
                })
              }
            >
              {tree.locked ? "Unlock…" : "Lock…"}
            </button>
            <button
              disabled={
                opened || tree.locked || tree.branch === "Detached HEAD"
              }
              onClick={() =>
                setReview({ type: "removeWorktree", path: tree.path })
              }
            >
              Remove checkout…
            </button>
          </div>
        </>
      )}
    </Dialog>
  );
}

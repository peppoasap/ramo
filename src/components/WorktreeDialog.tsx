import { useState } from "react";
import { FolderOpen } from "@phosphor-icons/react";
import { call } from "../bridge";
import type { Action, Snapshot } from "../domain";
import { Dialog } from "./Dialog";
interface ParentSelection {
  id: string;
  path: string;
}
export function WorktreeDialog({
  snapshot,
  initialBranch,
  create,
  close,
}: {
  snapshot: Snapshot;
  initialBranch?: string;
  create: (action: Action) => Promise<"created" | "trust" | "failed">;
  close: () => void;
}) {
  const [newBranch, setNewBranch] = useState(!initialBranch);
  const [branch, setBranch] = useState(initialBranch ?? "work/new-task");
  const [directory, setDirectory] = useState("new-task");
  const [parent, setParent] = useState<ParentSelection | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const available = snapshot.branches.filter(
    (branch) =>
      !branch.remote &&
      !snapshot.worktrees.some((tree) => tree.branch === branch.name),
  );
  async function chooseParent() {
    setBusy(true);
    setError("");
    try {
      const selection = await call<ParentSelection | null>(
        "choose_worktree_parent",
        { repoId: snapshot.repo.id },
      );
      if (selection) setParent(selection);
    } catch (error) {
      setError(String(error));
    } finally {
      setBusy(false);
    }
  }
  return (
    <Dialog title="Create worktree" locked={busy} onClose={close}>
      <p>
        A separate checkout for <strong>{snapshot.repo.name}</strong>. Changes
        in your current working folder stay here.
      </p>
      {error && (
        <p role="alert" className="resolution-error">
          {error}
        </p>
      )}
      {notice && (
        <p role="status" className="small">
          {notice}
        </p>
      )}
      <form
        className="worktree-form"
        onSubmit={async (event) => {
          event.preventDefault();
          if (!parent || busy) return;
          setBusy(true);
          setError("");
          setNotice("");
          try {
            const success = await create({
              type: "createWorktree",
              selectionId: parent.id,
              directory,
              branch,
              createBranch: newBranch,
            });
            if (success === "created") close();
            else if (success === "trust")
              setNotice(
                "Enable writes for this session, then choose Create worktree again. Your settings are retained.",
              );
            else
              setError(
                "Worktree creation did not finish. Review the repository error and worktree list before retrying; your form is retained.",
              );
          } finally {
            setBusy(false);
          }
        }}
      >
        <label>
          Branch mode
          <select
            aria-label="Worktree branch mode"
            disabled={busy}
            value={newBranch ? "new" : "existing"}
            onChange={(event) => {
              const next = event.target.value === "new";
              setNewBranch(next);
              setBranch(next ? "work/new-task" : (available[0]?.name ?? ""));
            }}
          >
            <option value="new">Create a new branch from current HEAD</option>
            <option value="existing" disabled={!available.length}>
              Use an existing local branch
            </option>
          </select>
        </label>
        {newBranch ? (
          <label>
            New branch name
            <input
              required
              disabled={busy}
              maxLength={1024}
              value={branch}
              onChange={(event) => setBranch(event.target.value)}
            />
          </label>
        ) : (
          <label>
            Existing branch
            <select
              aria-label="Existing worktree branch"
              disabled={busy}
              required
              value={branch}
              onChange={(event) => setBranch(event.target.value)}
            >
              {available.map((branch) => (
                <option key={branch.name} value={branch.name}>
                  {branch.name}
                </option>
              ))}
            </select>
          </label>
        )}
        <label>
          Parent folder
          <div className="worktree-parent">
            <code>
              {parent?.path ??
                "Choose a folder outside existing Git repositories"}
            </code>
            <button type="button" disabled={busy} onClick={chooseParent}>
              <FolderOpen size={16} />
              Choose folder…
            </button>
          </div>
        </label>
        <label>
          New folder name
          <input
            required
            disabled={busy}
            value={directory}
            maxLength={100}
            onChange={(event) => setDirectory(event.target.value)}
            placeholder="new-task"
          />
        </label>
        {parent && (
          <p className="small">
            Destination:{" "}
            <code>
              {parent.path.replace(/[\\/]$/, "")}/{directory}
            </code>
            <br />
            {newBranch
              ? `Starts at ${snapshot.head?.slice(0, 12)} on ${snapshot.branch}.`
              : `Checks out ${branch}.`}{" "}
            Existing destination folders are never overwritten.
          </p>
        )}
        <div className="dialog-actions">
          <button type="button" disabled={busy} onClick={close}>
            Cancel
          </button>
          <button
            className="primary"
            type="submit"
            disabled={busy || !parent || !branch.trim() || !directory.trim()}
          >
            {busy ? "Preparing worktree…" : "Create worktree"}
          </button>
        </div>
      </form>
    </Dialog>
  );
}

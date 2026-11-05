import { useEffect, useState } from "react";
import { call } from "../bridge";
import {
  isConflicted,
  type Action,
  type FileChange,
  type Snapshot,
  type Stash,
  type StashDetails,
} from "../domain";
import { Dialog } from "./Dialog";
import { Diff } from "./Diff";
type Choice = "applyStash" | "popStash" | "dropStash" | "restoreStashFile";
export function StashDialog({
  repoId,
  stash,
  snapshot,
  run,
  close,
}: {
  repoId: string;
  stash: Stash;
  snapshot: Snapshot;
  run: (action: Action) => Promise<"done" | "trust" | "failed">;
  close: () => void;
}) {
  const [details, setDetails] = useState<StashDetails | null>(null);
  const [selected, setSelected] = useState<{
    file: FileChange;
    untracked: boolean;
  } | null>(null);
  const [diff, setDiff] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState(false);
  const [choice, setChoice] = useState<Choice | null>(null);
  const [restorePath, setRestorePath] = useState("");
  const blocked = !!snapshot.operation || snapshot.files.some(isConflicted);
  useEffect(() => {
    let live = true;
    setDetails(null);
    setError("");
    setSelected(null);
    void call<StashDetails>("stash_details", {
      repoId,
      index: stash.index,
      oid: stash.oid,
    }).then(
      (value) => {
        if (live) {
          setDetails(value);
          const file = value.tracked[0] ?? value.untracked[0];
          if (file) setSelected({ file, untracked: !value.tracked.length });
        }
      },
      (error) => {
        if (live) setError(String(error));
      },
    );
    return () => {
      live = false;
    };
  }, [repoId, stash.index, stash.oid]);
  useEffect(() => {
    let live = true;
    setDiff("");
    setError("");
    if (!selected) return;
    setLoading(true);
    void call<string>("stash_diff", {
      repoId,
      index: stash.index,
      oid: stash.oid,
      path: selected.file.path,
      untracked: selected.untracked,
    })
      .then(
        (value) => {
          if (live) setDiff(value);
        },
        (error) => {
          if (live) setError(String(error));
        },
      )
      .finally(() => {
        if (live) setLoading(false);
      });
    return () => {
      live = false;
    };
  }, [repoId, stash.index, stash.oid, selected]);
  async function confirm() {
    if (!choice || busy) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const requested: Action =
        choice === "restoreStashFile"
          ? {
              type: choice,
              index: stash.index,
              oid: stash.oid,
              path: restorePath,
            }
          : { type: choice, index: stash.index, oid: stash.oid };
      const result = await run(requested);
      if (result === "done") close();
      else if (result === "trust")
        setNotice(
          "Trust this repository for the session, then confirm again. Nothing has been applied.",
        );
      else
        setError(
          "The action did not complete normally. Close this view and inspect Changes and the stash list before retrying; an interrupted operation may already have changed state.",
        );
    } catch (error) {
      setError(String(error));
    } finally {
      setBusy(false);
    }
  }
  return (
    <Dialog
      title={`Inspect ${stash.reference}`}
      wide
      locked={busy}
      onClose={close}
    >
      <p>
        {stash.message}
        <br />
        <code>{stash.oid}</code>
      </p>
      <p className="small">
        Tracked preview combines saved staged and unstaged edits. Apply/Pop
        restores working files; the saved staging layout is not restored.
      </p>
      {notice && <p role="status">{notice}</p>}
      {error && (
        <p role="alert" className="resolution-error">
          {error}
        </p>
      )}
      {!details && !error && <p role="status">Reading stash…</p>}
      {details && (
        <div className="stash-preview">
          <nav aria-label="Saved stash files">
            {([false, true] as const).map((untracked) => (
              <section key={String(untracked)}>
                <h3>
                  {untracked ? "Saved untracked files" : "Tracked changes"}
                </h3>
                {(untracked ? details.untracked : details.tracked).map(
                  (file) => (
                    <button
                      key={file.path}
                      disabled={busy || !!choice}
                      className={
                        selected?.file.path === file.path &&
                        selected.untracked === untracked
                          ? "active"
                          : ""
                      }
                      onClick={() => setSelected({ file, untracked })}
                      title={file.path}
                    >
                      {file.index} {file.path}
                    </button>
                  ),
                )}
                {!(untracked ? details.untracked : details.tracked).length && (
                  <p className="small">None</p>
                )}
              </section>
            ))}
          </nav>
          <div className="stash-diff">
            <Diff text={diff} loading={loading} />
          </div>
        </div>
      )}
      {blocked && (
        <p role="status">
          Apply/Pop/Drop require resolving conflicts and finishing any active
          operation. A missing saved untracked file can be restored separately
          when no sequence is active.
        </p>
      )}
      {choice ? (
        <section className="stash-confirm" aria-label="Review stash action">
          <strong>
            {choice === "restoreStashFile"
              ? `Restore ${restorePath}?`
              : choice === "dropStash"
                ? "Remove this saved stash?"
                : choice === "popStash"
                  ? "Apply, then remove on success?"
                  : "Apply and keep the saved stash?"}
          </strong>
          <p>
            {choice === "restoreStashFile"
              ? "Creates only this missing regular file at its original worktree path, from its saved Git contents. Checkout filters are not run. Existing files, symlinks and tracked paths are refused. The file is not staged; the stash and other conflicts are retained. Limit: 8 MiB."
              : choice === "dropStash"
                ? "This removes the selected entry from the stash list. Ramo has no undo for this removal. Working files and the index stay unchanged."
                : "Git may restore some files before reporting a conflict. On failure the stash is retained; review Changes before retrying. No commit or push."}
          </p>
          <div className="dialog-actions">
            <button disabled={busy} onClick={() => setChoice(null)}>
              Back to inspection
            </button>
            <button
              className="primary"
              disabled={
                busy ||
                (choice === "restoreStashFile" ? !!snapshot.operation : blocked)
              }
              onClick={confirm}
            >
              {choice === "restoreStashFile"
                ? "Confirm restore file"
                : choice === "dropStash"
                  ? "Confirm remove stash"
                  : choice === "popStash"
                    ? "Confirm pop stash"
                    : "Confirm apply stash"}
            </button>
          </div>
        </section>
      ) : (
        <div className="dialog-actions">
          <button disabled={busy} onClick={close}>
            Close
          </button>
          {selected?.untracked && (
            <button
              disabled={busy || !details || !!snapshot.operation}
              onClick={() => {
                setRestorePath(selected.file.path);
                setChoice("restoreStashFile");
              }}
            >
              Restore selected file…
            </button>
          )}
          <button
            disabled={!details || blocked || busy}
            onClick={() => setChoice("dropStash")}
          >
            Drop stash…
          </button>
          <button
            disabled={!details || blocked || busy}
            onClick={() => setChoice("popStash")}
          >
            Pop stash…
          </button>
          <button
            className="primary"
            disabled={!details || blocked || busy}
            onClick={() => setChoice("applyStash")}
          >
            Apply and keep…
          </button>
        </div>
      )}
    </Dialog>
  );
}

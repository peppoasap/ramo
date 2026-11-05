import { useEffect, useState } from "react";
import type { Action, Snapshot } from "../domain";
import { call } from "../bridge";
import { Dialog } from "./Dialog";
interface Remote {
  name: string;
  fetch: string[];
  push: string[];
  editable: boolean;
}
function unusedName(rows: Remote[]) {
  return (
    [
      "origin",
      "upstream",
      ...Array.from({ length: 100 }, (_, i) => `remote${i + 1}`),
    ].find((name) => !rows.some((row) => row.name === name)) ?? ""
  );
}
export function RemotesDialog({
  snapshot,
  busy,
  close,
  save,
}: {
  snapshot: Snapshot;
  busy: boolean;
  close: () => void;
  save: (
    value: Extract<Action, { type: "configureRemote" }>,
  ) => Promise<"done" | "failed" | "trust">;
}) {
  const [rows, setRows] = useState<Remote[]>([]),
    [ready, setReady] = useState(false),
    [error, setError] = useState("");
  const [editing, setEditing] = useState<Remote | null>(null),
    [name, setName] = useState("origin"),
    [url, setUrl] = useState("");
  const [review, setReview] = useState(false);
  useEffect(() => {
    let active = true;
    call<Remote[]>("repository_remotes", { repoId: snapshot.repo.id })
      .then((value) => {
        if (active) {
          setRows(value);
          setName(unusedName(value));
          setReady(true);
        }
      })
      .catch((e) => {
        if (active) setError(String(e));
      });
    return () => {
      active = false;
    };
  }, [snapshot.repo.id]);
  return (
    <Dialog
      title={`Remotes · ${snapshot.repo.name}`}
      onClose={close}
      locked={busy}
    >
      {error && (
        <p role="alert" className="resolution-error">
          {error}
        </p>
      )}
      {review ? (
        <>
          <p>
            {editing ? "Change" : "Add"} <strong>{name}</strong> in this
            repository:
          </p>
          {editing && (
            <p className="small">
              From <code className="clone-path">{editing.fetch[0]}</code>
            </p>
          )}
          <code className="clone-path">{url}</code>
          <p className="small">
            Future fetch/pull/push will use this endpoint. Saving does not
            contact the server or change your files. Existing Git credentials or
            SSH agent are used for later transport.
          </p>
          <div className="dialog-actions">
            <button disabled={busy} onClick={() => setReview(false)}>
              Back
            </button>
            <button
              className="primary"
              disabled={busy}
              onClick={async () => {
                try {
                  const result = await save({
                    type: "configureRemote",
                    name,
                    url,
                    existing: !!editing,
                  });
                  if (result === "done") close();
                  else if (result === "failed")
                    setError(
                      "Remote was not saved. Close to inspect the error and refresh before retrying.",
                    );
                } catch (e) {
                  setError(String(e));
                }
              }}
            >
              Save remote
            </button>
          </div>
        </>
      ) : (
        <>
          {!ready ? (
            <p role="status">Reading remotes…</p>
          ) : (
            <section aria-label="Configured remotes" className="remote-list">
              {!rows.length && (
                <p className="small">
                  No remote configured. Add origin to connect this local
                  repository.
                </p>
              )}
              {rows.map((row) => (
                <div key={row.name}>
                  <strong>{row.name}</strong>
                  <code className="clone-path">{row.fetch.join("\n")}</code>
                  {row.push.join() !== row.fetch.join() && (
                    <>
                      <small>Push</small>
                      <code className="clone-path">{row.push.join("\n")}</code>
                    </>
                  )}
                  <button
                    disabled={busy || !row.editable}
                    onClick={() => {
                      setEditing(row);
                      setName(row.name);
                      setUrl(row.fetch[0]);
                    }}
                  >
                    Edit URL
                  </button>
                  {!row.editable && (
                    <p className="small">
                      Custom or multiple endpoints: manage with Git. Existing
                      settings are preserved.
                    </p>
                  )}
                </div>
              ))}
            </section>
          )}
          <form
            className="worktree-form"
            onSubmit={(e) => {
              e.preventDefault();
              setReview(true);
            }}
          >
            <h3>{editing ? `Edit ${editing.name}` : "Add remote"}</h3>
            <label>
              Name
              <input
                aria-label="Remote name"
                value={name}
                required
                maxLength={100}
                disabled={busy || !!editing}
                onChange={(e) => setName(e.target.value)}
              />
            </label>
            <label>
              HTTPS or SSH URL
              <input
                aria-label="Remote URL"
                required
                maxLength={2048}
                value={url}
                disabled={busy}
                placeholder="https://host/team/repository.git"
                onChange={(e) => setUrl(e.target.value)}
              />
            </label>
            <p className="small">
              Do not put tokens or passwords in URLs. This does not sign in or
              install a credential helper.
            </p>
            <div className="dialog-actions">
              <button type="button" disabled={busy} onClick={close}>
                Close
              </button>
              {editing && (
                <button
                  type="button"
                  disabled={busy}
                  onClick={() => {
                    setEditing(null);
                    setName(unusedName(rows));
                    setUrl("");
                  }}
                >
                  Add another
                </button>
              )}
              <button
                type="submit"
                className="primary"
                disabled={busy || !ready || !name.trim() || !url.trim()}
              >
                Review endpoint
              </button>
            </div>
          </form>
        </>
      )}
    </Dialog>
  );
}

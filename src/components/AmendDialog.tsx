import { useEffect, useState } from "react";
import { call } from "../bridge";
import type { Snapshot } from "../domain";
import { isStaged } from "../domain";
import { Dialog } from "./Dialog";

export function AmendDialog({
  snapshot,
  busy,
  close,
  submit,
}: {
  snapshot: Snapshot;
  busy: boolean;
  close: () => void;
  submit: (message: string) => Promise<unknown>;
}) {
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const [ready, setReady] = useState(false);
  useEffect(() => {
    let active = true;
    call<string>("amend_message", {
      repoId: snapshot.repo.id,
      head: snapshot.head,
    })
      .then((value) => {
        if (active) {
          setMessage(value);
          setReady(true);
        }
      })
      .catch((e) => {
        if (active) setError(String(e));
      });
    return () => {
      active = false;
    };
  }, [snapshot]);
  return (
    <Dialog title="Amend last commit" onClose={close} locked={busy}>
      <p>
        Rewrite <code>{snapshot.head?.slice(0, 12)}</code> on{" "}
        <strong>{snapshot.branch}</strong> with the{" "}
        {snapshot.files.filter(isStaged).length} currently staged file(s).
        Unstaged files stay outside the commit.
      </p>
      <p className="small">
        The original commit is kept locally for recovery. Fetch before
        rewriting: shared commits may be missing from local remote refs. Push
        remains separate.
      </p>
      <label htmlFor="amend-message">Replacement commit message</label>
      <textarea
        className="amend-message"
        id="amend-message"
        rows={8}
        value={message}
        disabled={!ready || busy}
        maxLength={32768}
        onChange={(e) => setMessage(e.target.value)}
      />
      {error && <p role="alert">{error}</p>}
      <div className="dialog-actions">
        <button disabled={busy} onClick={close}>
          Cancel
        </button>
        <button
          className="primary"
          disabled={!ready || busy || !message.trim()}
          onClick={async () => {
            const result = await submit(message);
            if (result === true) close();
            else if (result !== "trust")
              setError(
                "Amend was not completed. Your text is retained here. Close this review to inspect the operation error and refreshed Changes before retrying.",
              );
          }}
        >
          Rewrite reviewed commit
        </button>
      </div>
    </Dialog>
  );
}

import { useEffect, useMemo, useState } from "react";
import { call } from "../bridge";
import type { Commit, Comparison, ComparisonKind, Snapshot } from "../domain";
import { Dialog } from "./Dialog";
import { Diff } from "./Diff";
interface Pair {
  from: string;
  to: string;
  kind: ComparisonKind;
}
export function ComparisonDialog({
  repoId,
  initialFrom,
  initialTo,
  snapshot,
  commits,
  close,
}: {
  repoId: string;
  initialFrom: string;
  initialTo: string;
  snapshot: Snapshot;
  commits: Commit[];
  close: () => void;
}) {
  const [from, setFrom] = useState(initialFrom),
    [to, setTo] = useState(initialTo);
  const [kind, setKind] = useState<ComparisonKind>("trees");
  const [request, setRequest] = useState<Pair | null>({
    from: initialFrom,
    to: initialTo,
    kind: "trees",
  });
  const [comparison, setComparison] = useState<Comparison | null>(null);
  const [file, setFile] = useState("");
  const [diff, setDiff] = useState("");
  const [loading, setLoading] = useState(false),
    [diffLoading, setDiffLoading] = useState(false);
  const [error, setError] = useState("");
  const options = useMemo(() => {
    const choices = new Map<string, string>();
    if (snapshot.head) choices.set(snapshot.head, "HEAD");
    for (const branch of snapshot.branches)
      choices.set(
        branch.oid,
        [choices.get(branch.oid), branch.name].filter(Boolean).join(" · "),
      );
    for (const commit of commits)
      if (!choices.has(commit.oid)) choices.set(commit.oid, commit.subject);
    for (const oid of [initialFrom, initialTo, from, to])
      if (!choices.has(oid)) choices.set(oid, oid);
    return Array.from(choices, ([oid, label]) => ({ oid, label }));
  }, [
    snapshot.head,
    snapshot.branch,
    snapshot.branches,
    commits,
    initialFrom,
    initialTo,
    from,
    to,
  ]);
  function clear() {
    setRequest(null);
    setComparison(null);
    setFile("");
    setDiff("");
    setError("");
    setLoading(false);
    setDiffLoading(false);
  }
  useEffect(() => {
    let live = true;
    setComparison(null);
    setFile("");
    setDiff("");
    setError("");
    if (!request) return;
    setLoading(true);
    void call<Comparison>("compare_revisions", { repoId, ...request })
      .then(
        (result) => {
          if (live) {
            setComparison(result);
            setFile(result.files[0]?.path ?? "");
          }
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
  }, [repoId, request]);
  useEffect(() => {
    let live = true;
    setDiff("");
    if (!comparison || !file) {
      setDiffLoading(false);
      return;
    }
    setDiffLoading(true);
    setError("");
    void call<string>("comparison_diff", {
      repoId,
      from: comparison.effectiveFrom,
      to: comparison.to,
      kind: "trees",
      path: file,
    })
      .then(
        (result) => {
          if (live) setDiff(result);
        },
        (error) => {
          if (live) setError(String(error));
        },
      )
      .finally(() => {
        if (live) setDiffLoading(false);
      });
    return () => {
      live = false;
    };
  }, [repoId, comparison, file]);
  const valid = [from, to].every((oid) =>
    /^(?:[a-fA-F0-9]{40}|[a-fA-F0-9]{64})$/.test(oid),
  );
  return (
    <Dialog title="Compare revisions" wide onClose={close}>
      <p className="small">
        Read-only · Compares saved commit trees. Working files and the staging
        area are excluded. Branch choices are captured as object IDs; moving a
        branch does not change the displayed pair.
      </p>
      <form
        className="comparison-controls"
        onSubmit={(event) => {
          event.preventDefault();
          if (valid) setRequest({ from, to, kind });
        }}
      >
        {(["from", "to"] as const).map((side) => (
          <label key={side}>
            {side === "from" ? "Base revision" : "Target revision"}
            <select
              aria-label={
                side === "from"
                  ? "Choose base revision"
                  : "Choose target revision"
              }
              value={side === "from" ? from : to}
              onChange={(event) => {
                clear();
                (side === "from" ? setFrom : setTo)(event.target.value);
              }}
            >
              <option value="">Choose a branch or loaded commit</option>
              {options.map((option) => (
                <option key={option.oid} value={option.oid}>
                  {option.label} · {option.oid.slice(0, 8)}
                </option>
              ))}
            </select>
            <input
              aria-label={
                side === "from" ? "Base commit SHA" : "Target commit SHA"
              }
              value={side === "from" ? from : to}
              spellCheck={false}
              placeholder="Full commit SHA"
              onChange={(event) => {
                clear();
                (side === "from" ? setFrom : setTo)(event.target.value.trim());
              }}
            />
          </label>
        ))}
        <label>
          Comparison mode
          <select
            aria-label="Comparison mode"
            value={kind}
            onChange={(event) => {
              clear();
              setKind(event.target.value as ComparisonKind);
            }}
          >
            <option value="trees">Direct: base tree → target tree</option>
            <option value="mergeBase">
              Changes in target since common ancestor
            </option>
          </select>
        </label>
        <div className="dialog-actions">
          <button
            type="button"
            onClick={() => {
              clear();
              setFrom(to);
              setTo(from);
            }}
          >
            Swap revisions
          </button>
          <button
            className="primary"
            type="submit"
            disabled={!valid || loading}
          >
            Compare
          </button>
        </div>
      </form>
      {error && (
        <p role="alert" className="resolution-error">
          {error}
        </p>
      )}
      {loading && <p role="status">Comparing revisions…</p>}
      {comparison && (
        <>
          <p className="comparison-pair">
            <strong>
              {comparison.kind === "trees"
                ? "Direct tree comparison"
                : "Target changes since common ancestor"}
            </strong>
            <br />
            <code>
              {comparison.effectiveFrom} → {comparison.to}
            </code>
            <br />
            {comparison.files.length} changed{" "}
            {comparison.files.length === 1 ? "file" : "files"}
          </p>
          {!comparison.files.length ? (
            <p role="status">No file differences between these commit trees.</p>
          ) : (
            <div className="comparison-preview">
              <nav aria-label="Comparison changed files">
                {comparison.files.map((change) => (
                  <button
                    key={change.path}
                    title={change.path}
                    className={file === change.path ? "active" : ""}
                    onClick={() => setFile(change.path)}
                  >
                    {change.index} {change.path}
                  </button>
                ))}
              </nav>
              <div className="comparison-diff">
                <Diff text={diff} loading={diffLoading} />
              </div>
            </div>
          )}
        </>
      )}
      {!request && !error && (
        <p className="small">
          Choose revisions and press Compare. Full commit SHAs outside loaded
          history are supported.
        </p>
      )}
    </Dialog>
  );
}

import { supportsHunks } from "../hunks";
import {
  GitBranch,
  File,
  Check,
  Code,
  Sparkle,
  GitCommit,
} from "@phosphor-icons/react";
import type {
  Snapshot,
  Commit,
  FileChange,
  SelectedFile,
  AiConfig,
  Action,
} from "../domain";
import { isStaged as staged, isConflicted, canCommit } from "../domain";
import type { Confirmation } from "../domain";
import { Diff } from "./Diff";
interface Props {
  amend: () => void;
  onHunk: (index: number) => Promise<void>;
  resolveConflict: (path: string) => void;
  mode: "history" | "changes";
  snapshot: Snapshot;
  selected: Commit | null;
  files: FileChange[];
  file: SelectedFile | null;
  setFile: (file: SelectedFile) => void;
  busy: string;
  aiPending: boolean;
  diff: string;
  diffLoading: boolean;
  draft: string;
  draftWarning: string;
  setDraft: (value: string) => void;
  ai: AiConfig;
  action: (action: Action) => Promise<unknown>;
  prepareAi: (kind: "commit" | "explain") => Promise<void>;
  showProviders: () => void;
  setConfirmation: (value: Confirmation) => void;
}
const unstaged = (file: FileChange) =>
  !isConflicted(file) && (file.worktree !== " " || file.index === "?");

export function Inspector({
  amend,
  onHunk,
  resolveConflict,
  mode,
  snapshot,
  selected,
  files,
  file,
  setFile,
  busy,
  aiPending,
  diff,
  diffLoading,
  draft,
  draftWarning,
  setDraft,
  ai,
  action,
  prepareAi,
  showProviders,
  setConfirmation,
}: Props) {
  function fileRows(list: FileChange[], inIndex: boolean | null) {
    return list.map((item) => (
      <div
        className={`file-row ${file?.path === item.path && (inIndex === null || file.staged === inIndex) ? "selected" : ""}`}
        key={`${inIndex}-${item.path}`}
      >
        {inIndex !== null && (
          <button
            className={`stage-toggle ${inIndex ? "checked" : ""}`}
            disabled={!!busy}
            title={inIndex ? "Unstage file" : "Stage file"}
            aria-label={`${inIndex ? "Unstage" : "Stage"} ${item.path}`}
            onClick={() =>
              action({
                type: inIndex ? "unstage" : "stage",
                paths: [item.path],
              })
            }
          >
            {inIndex && <Check size={12} weight="bold" />}
          </button>
        )}
        <button
          className="file-select"
          onClick={() => setFile({ path: item.path, staged: inIndex === true })}
          title={item.original ? `${item.original} → ${item.path}` : item.path}
        >
          <File size={16} />
          <span>{item.path}</span>
          <code className="file-status">
            {inIndex === false
              ? item.worktree === "?"
                ? "NEW"
                : item.worktree
              : item.index}
          </code>
        </button>
      </div>
    ));
  }

  return (
    <aside
      className="inspector"
      aria-label={mode === "changes" ? "Working changes" : "Commit inspector"}
    >
      <div className="inspector-heading">
        <h2>
          {mode === "changes"
            ? "Working changes"
            : selected?.subject || "Commit details"}
        </h2>
        {mode === "changes" && (
          <span className="branch-chip">
            <GitBranch size={12} />
            {snapshot.branch}
          </span>
        )}
      </div>
      {mode === "history" && selected && (
        <div className="commit-detail">
          <code>{selected.oid.slice(0, 12)}</code>
          <span>{selected.author}</span>
          <time>{new Date(selected.date).toLocaleString()}</time>
        </div>
      )}
      <div className="changed-files">
        {mode === "changes" ? (
          <>
            {files.some(isConflicted) && (
              <>
                <div className="file-group-heading">
                  Conflicts ({files.filter(isConflicted).length})
                </div>
                <p className="small">
                  Open the guided resolver to compare versions, choose each
                  block and review the result. Save and stage each file, then
                  finish the current operation.
                </p>
                {files.filter(isConflicted).map((item) => (
                  <div className="file-row" key={item.path}>
                    <button
                      disabled={!!busy}
                      onClick={() => resolveConflict(item.path)}
                    >
                      Resolve…
                    </button>
                    <button
                      className="file-select"
                      onClick={() =>
                        setFile({ path: item.path, staged: false })
                      }
                      title={item.path}
                    >
                      <File size={16} />
                      <span>{item.path}</span>
                      <code>
                        {item.index}
                        {item.worktree}
                      </code>
                    </button>
                    <button
                      disabled={!!busy}
                      onClick={() =>
                        setConfirmation({
                          action: { type: "markResolved", path: item.path },
                          title: "Mark conflict resolved",
                          description: `Stage the current contents of ${item.path} as your resolution? Review it in your editor first. This does not choose a side or edit the file.`,
                        })
                      }
                    >
                      Mark resolved
                    </button>
                  </div>
                ))}
              </>
            )}
            <div className="file-group-heading">
              Staged ({files.filter(staged).length})
              <button
                disabled={!!busy || !files.some(staged)}
                onClick={() =>
                  action({
                    type: "unstage",
                    paths: files.filter(staged).map((f) => f.path),
                  })
                }
              >
                Unstage all
              </button>
            </div>
            {fileRows(files.filter(staged), true)}
            <div className="file-group-heading">
              Unstaged ({files.filter(unstaged).length})
              <button
                disabled={!!busy || !files.some(unstaged)}
                onClick={() =>
                  action({
                    type: "stage",
                    paths: files.filter(unstaged).map((f) => f.path),
                  })
                }
              >
                Stage all
              </button>
            </div>
            {fileRows(files.filter(unstaged), false)}
            {files.length === 0 && (
              <div className="clean">
                <Check size={22} />
                <strong>
                  {snapshot.operation === "Merge in progress"
                    ? "Ready to complete merge"
                    : snapshot.operation
                      ? "No pending file changes"
                      : "Working tree clean"}
                </strong>
                <p>
                  {snapshot.operation === "Merge in progress"
                    ? "Write a message below and complete the merge."
                    : snapshot.operation
                      ? "Continue or abort the current operation using Git."
                      : "All your changes are committed."}
                </p>
              </div>
            )}
          </>
        ) : (
          <>
            <div className="file-group-heading">
              Changed files ({files.length})
            </div>
            {fileRows(files, null)}
          </>
        )}
      </div>
      <div className="diff-section">
        <div className="diff-heading">
          <span title={file?.path}>
            <Code size={16} />
            {file?.path || "Diff"}
          </span>
          <button
            disabled={!file || !!busy || aiPending}
            onClick={() => prepareAi("explain")}
          >
            <Sparkle size={14} />
            Explain
          </button>
        </div>
        <Diff
          text={diff}
          loading={diffLoading}
          hunkAction={
            mode === "changes" &&
            file &&
            !snapshot.operation &&
            !snapshot.files.some(isConflicted) &&
            supportsHunks(
              snapshot.files.find((item) => item.path === file.path),
              diff,
              file.staged,
            )
              ? {
                  label: file.staged ? "Unstage hunk" : "Stage hunk",
                  disabled: !!busy || aiPending,
                  run: (index) => {
                    void onHunk(index);
                  },
                }
              : undefined
          }
        />
      </div>
      {mode === "changes" && (
        <div className="composer">
          <div className="composer-heading">
            <label htmlFor="commit-message">Commit message</label>
            <button
              disabled={
                !!busy ||
                aiPending ||
                files.some(isConflicted) ||
                !files.some(staged)
              }
              onClick={() => prepareAi("commit")}
            >
              <Sparkle size={14} />
              Write with AI
            </button>
          </div>
          <textarea
            id="commit-message"
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            placeholder="Summarize your staged changes"
            maxLength={32768}
            onKeyDown={(e) => {
              if (
                (e.metaKey || e.ctrlKey) &&
                e.key === "Enter" &&
                draft.trim() &&
                canCommit(snapshot) &&
                !busy &&
                !aiPending
              ) {
                e.preventDefault();
                void action({ type: "commit", message: draft });
              }
            }}
          />
          {draftWarning ? (
            <p role="alert" className="warning-text">
              {draftWarning}
            </p>
          ) : draft ? (
            <p className="small">Draft saved on this device.</p>
          ) : null}
          <div className="composer-hint">
            <Sparkle size={14} />
            <span>
              {ai.provider === "ollama"
                ? "Local AI"
                : ai.provider === "codex"
                  ? "Codex CLI"
                  : "Claude API"}{" "}
              · selected changes only
            </span>
            <button onClick={showProviders}>Configure</button>
          </div>
          <button
            className="primary commit-button"
            disabled={
              !!busy || aiPending || !draft.trim() || !canCommit(snapshot)
            }
            onClick={() => action({ type: "commit", message: draft })}
          >
            <GitCommit size={18} />
            {snapshot.operation === "Merge in progress" ? (
              "Complete merge"
            ) : (
              <>
                Commit {files.filter(staged).length}{" "}
                {files.filter(staged).length === 1 ? "file" : "files"}
              </>
            )}
          </button>
          <small>Creates a local commit. Push separately.</small>
          <button
            disabled={
              !!busy ||
              aiPending ||
              !snapshot.head ||
              !!snapshot.operation ||
              files.some(isConflicted)
            }
            onClick={amend}
          >
            Amend last commit…
          </button>
        </div>
      )}
    </aside>
  );
}

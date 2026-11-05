import { useAiRequest } from "../useAiRequest";
import {
  useConflictBuffer,
  type ResolutionContext,
  type ConflictVersions,
} from "../useConflictBuffer";
export type { ResolutionContext } from "../useConflictBuffer";
import { useEffect, useMemo, useRef, useState } from "react";
import {
  X,
  Sparkle,
  CaretUp,
  CaretDown,
  FloppyDisk,
  ArrowLeft,
} from "@phosphor-icons/react";
import { call } from "../bridge";
import {
  chooseConflict,
  hasConflictMarkers,
  parseResolutionProposal,
  selectedSourceLines,
  replaceConflictText,
} from "../conflicts";
import {
  conflictLocations,
  sourceConflictRange,
  rangeLabel,
} from "../conflictNavigation";
import type { AiConfig, AiInput } from "../domain";
import { CodeEditor } from "./CodeEditor";
export function ConflictResolver({
  context,
  close,
  save,
  config,
  configure,
  reportWarning,
}: {
  context: ResolutionContext;
  close: () => void;
  save: (content: string | null) => Promise<boolean>;
  config: AiConfig;
  configure: () => void;
  reportWarning: (message: string) => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [pickedLines, setPickedLines] = useState<{
    ours: number[];
    theirs: number[];
  }>({ ours: [], theirs: [] });
  const resultCursor = useRef({ from: 0, to: 0 });
  const {
    versions,
    result,
    setResult,
    remove,
    setRemove,
    loadError,
    savedRecovery,
    recoveryStatus,
    recoveryWarning,
    restoreSaved,
    discardSaved,
    persistRecovery,
    discardDraft,
    canRecover,
    clearRecovery,
  } = useConflictBuffer(context);
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);
  const [review, setReview] = useState(false);
  const [discard, setDiscard] = useState(false);
  const [showBase, setShowBase] = useState(false);
  const [block, setBlock] = useState(0);
  const [navigation, setNavigation] = useState(0);
  const [followConflict, setFollowConflict] = useState(true);
  const [aiOpen, setAiOpen] = useState(false);
  const [aiInput, setAiInput] = useState<AiInput | null>(null);
  const [preparingAi, setPreparingAi] = useState(false);
  const aiRequest = useAiRequest();
  const aiBusy = preparingAi || aiRequest.pending;
  const [answer, setAnswer] = useState("");
  const [proposal, setProposal] = useState<{
    explanation: string;
    resolution: string | null;
  } | null>(null);
  const [requestedResult, setRequestedResult] = useState("");
  const [requestedRemove, setRequestedRemove] = useState(false);
  useEffect(() => {
    const target = dialog.current!;
    target.showModal();
    return () => target.close();
  }, []);
  const locations = useMemo(() => conflictLocations(result), [result]);
  const blocks = locations.map((location) => location.part);
  const currentBlock = Math.min(block, Math.max(0, blocks.length - 1));
  const active = locations[currentBlock];
  const sourceRanges = useMemo(
    () => ({
      ours: sourceConflictRange(
        versions?.ours ?? null,
        active?.part.ours ?? "",
      ),
      theirs: sourceConflictRange(
        versions?.theirs ?? null,
        active?.part.theirs ?? "",
      ),
      base: sourceConflictRange(
        versions?.base ?? null,
        active?.part.base ?? "",
      ),
    }),
    [versions, active?.part.ours, active?.part.theirs, active?.part.base],
  );
  useEffect(() => {
    setPickedLines({ ours: [], theirs: [] });
  }, [active?.part.raw]);
  function navigate(index: number) {
    setBlock(index);
    setPickedLines({ ours: [], theirs: [] });
  }
  const pickedCount = pickedLines.ours.length + pickedLines.theirs.length;
  function toggleLine(side: "ours" | "theirs", number: number) {
    setPickedLines((previous) => ({
      ...previous,
      [side]: previous[side].includes(number)
        ? previous[side].filter((value) => value !== number)
        : [...previous[side], number],
    }));
  }
  function usePickedLines() {
    if (!versions || !pickedCount) return;
    const ours = selectedSourceLines(versions.ours ?? "", pickedLines.ours);
    const theirs = selectedSourceLines(
      versions.theirs ?? "",
      pickedLines.theirs,
    );
    const separator =
      ours && theirs && !ours.endsWith("\n")
        ? result.includes("\r\n")
          ? "\r\n"
          : "\n"
        : "";
    const chosen = ours + separator + theirs;
    if (blocks.length)
      setResult((value) => replaceConflictText(value, currentBlock, chosen));
    else {
      const { from, to } = resultCursor.current;
      setResult((value) => value.slice(0, from) + chosen + value.slice(to));
    }
    setRemove(false);
    setPickedLines({ ours: [], theirs: [] });
  }
  const dirty =
    versions &&
    (result !== (versions.working ?? versions.ours ?? versions.theirs ?? "") ||
      remove);
  function requestClose() {
    if (!saving && !aiBusy) dirty ? setDiscard(true) : close();
  }
  function choose(choice: "ours" | "theirs" | "both") {
    setResult((value) => chooseConflict(value, currentBlock, choice));
    setRemove(false);
  }
  async function prepare(kind: "resolve" | "explainConflict") {
    aiRequest.clearNotice();
    setAiOpen(true);
    setPreparingAi(true);
    setError("");
    setAnswer("");
    setProposal(null);
    setAiInput(null);
    try {
      const input = await call<AiInput>("prepare_ai", {
        repoId: context.repoId,
        kind,
        path: context.path,
        commit: null,
        staged: false,
      });
      if (input.fingerprint !== context.fingerprint)
        throw new Error(
          "Repository changed. Copy your result before reopening this editor.",
        );
      setRequestedResult(result);
      setRequestedRemove(remove);
      setAiInput(input);
    } catch (error) {
      setError(String(error));
    } finally {
      setPreparingAi(false);
    }
  }
  async function generate() {
    if (!aiInput || aiBusy) return;
    const input = aiInput;
    setError("");
    setAiInput(null);
    try {
      const response = await aiRequest.run(input.id, config);
      if (response === null) return;
      if (input.kind === "resolve")
        setProposal(parseResolutionProposal(response));
      else setAnswer(response);
    } catch (error) {
      setError(String(error));
    }
  }
  const proposalStale =
    result !== requestedResult || remove !== requestedRemove;
  return (
    <dialog
      ref={dialog}
      className="merge-editor"
      aria-label={`Merge editor: ${context.path}`}
      onCancel={(event) => {
        event.preventDefault();
        requestClose();
      }}
      onKeyDown={(event) => {
        if ((event.metaKey || event.ctrlKey) && event.key === "s") {
          event.preventDefault();
          if (
            !saving &&
            !savedRecovery &&
            !remove &&
            !hasConflictMarkers(result)
          )
            setReview(true);
        }
      }}
    >
      <header className="merge-editor-header">
        <button
          className="icon"
          aria-label="Back to repository"
          disabled={saving || aiBusy}
          onClick={requestClose}
        >
          <ArrowLeft size={18} />
        </button>
        <div>
          <strong>Merge editor</strong>
          <span>{context.path}</span>
        </div>
        <span className={`merge-status ${blocks.length ? "unresolved" : ""}`}>
          {blocks.length
            ? `${blocks.length} unresolved`
            : hasConflictMarkers(result)
              ? "Review markers"
              : "Ready to resolve"}
        </span>
        <button
          className="icon"
          aria-label="Close merge editor"
          disabled={saving || aiBusy}
          onClick={requestClose}
        >
          <X size={18} />
        </button>
      </header>
      <div className="merge-editor-toolbar">
        <label className="merge-inline-check">
          <input
            type="checkbox"
            checked={showBase}
            onChange={(event) => setShowBase(event.target.checked)}
          />
          Show base
        </label>
        <label className="merge-inline-check">
          <input
            type="checkbox"
            checked={followConflict}
            onChange={(event) => setFollowConflict(event.target.checked)}
          />
          Follow active conflict
        </label>
        <button
          disabled={!active}
          onClick={() => setNavigation((value) => value + 1)}
        >
          Locate active conflict
        </button>
        <span className="merge-toolbar-spacer" />
        <button
          disabled={aiBusy || !versions}
          onClick={() => prepare("explainConflict")}
        >
          <Sparkle size={15} />
          Explain conflict
        </button>
        <button
          disabled={aiBusy || !versions}
          onClick={() => prepare("resolve")}
        >
          <Sparkle size={15} />
          Suggest resolution
        </button>
        <button onClick={configure}>AI settings</button>
      </div>
      {(error || loadError) && (
        <div role="alert" className="merge-editor-error">
          {error || loadError}
          <button
            className="icon"
            aria-label="Dismiss merge error"
            onClick={() => setError("")}
          >
            <X size={14} />
          </button>
        </div>
      )}
      {savedRecovery && (
        <section
          className="merge-recovery-review"
          aria-label="Saved resolution review"
        >
          <strong>Git changed since this result was saved.</strong>
          <span>
            Review the saved result before restoring it into the current editor.
            Choose whether to restore or discard this saved copy before saving
            the current result. Nothing is applied to Git.
          </span>
          <details>
            <summary>
              Preview saved result ·{" "}
              {new Date(savedRecovery.updatedAt).toLocaleString()}
            </summary>
            <pre>
              {savedRecovery.remove ? "Deletion was selected.\n" : ""}
              {savedRecovery.content}
            </pre>
          </details>
          <button disabled={saving} onClick={restoreSaved}>
            Restore saved result into editor
          </button>
          <button disabled={saving} onClick={discardSaved}>
            Discard saved result
          </button>
        </section>
      )}
      {recoveryWarning && (
        <div role="alert" className="merge-editor-error">
          {recoveryWarning}
        </div>
      )}
      <p className="merge-line-pick-hint">
        Pick individual lines: click a source line number or +, then Use
        selected lines.
      </p>
      <div className={`merge-editor-body ${aiOpen ? "with-ai" : ""}`}>
        <div className="merge-code-workspace">
          {!versions && !error && !loadError && (
            <p className="small">Reading conflict versions…</p>
          )}
          {versions && (
            <>
              <div
                className={`merge-source-grid ${showBase ? "with-base" : ""}`}
              >
                {(
                  [
                    "ours",
                    ...(showBase ? ["base"] : []),
                    "theirs",
                  ] as (keyof ConflictVersions)[]
                ).map((side) => (
                  <section
                    className={`merge-code-pane ${side}`}
                    key={side}
                    aria-label={`${side} source`}
                  >
                    <header>
                      <strong>
                        {side === "ours"
                          ? "Current · ours"
                          : side === "theirs"
                            ? "Incoming · theirs"
                            : "Common ancestor"}
                      </strong>
                      <span>
                        {versions[side] === null
                          ? "File absent"
                          : !active
                            ? "Read only"
                            : rangeLabel(
                                sourceRanges[
                                  side as "ours" | "theirs" | "base"
                                ],
                              )}
                      </span>
                      <button
                        disabled={saving}
                        title={`Replace result with entire ${side} file`}
                        onClick={() => {
                          setResult(versions[side] ?? "");
                          setRemove(versions[side] === null);
                        }}
                      >
                        Use file
                      </button>
                    </header>
                    <CodeEditor
                      value={versions[side] ?? ""}
                      filename={context.path}
                      label={`${side} file contents`}
                      activeRange={
                        sourceRanges[side as "ours" | "theirs" | "base"]
                      }
                      navigation={navigation}
                      follow={followConflict}
                      selectedLines={
                        side === "ours" || side === "theirs"
                          ? pickedLines[side]
                          : undefined
                      }
                      onPickLine={
                        !saving && (side === "ours" || side === "theirs")
                          ? (number) => toggleLine(side, number)
                          : undefined
                      }
                    />
                  </section>
                ))}
              </div>
              <section className="merge-result-pane" aria-label="Merge result">
                <header>
                  <strong>Result</strong>
                  {
                    <>
                      <button
                        disabled={saving || pickedCount === 0}
                        onClick={usePickedLines}
                        title={
                          blocks.length
                            ? "Replace the active conflict with selected lines, ours then theirs"
                            : "Insert selected lines at the result cursor"
                        }
                      >
                        {blocks.length ? "Use" : "Insert"}{" "}
                        {pickedCount > 0 ? `${pickedCount} ` : ""}selected{" "}
                        {pickedCount === 1 ? "line" : "lines"}
                      </button>
                      <button
                        disabled={saving || pickedCount === 0}
                        onClick={() => setPickedLines({ ours: [], theirs: [] })}
                      >
                        Clear selection
                      </button>
                    </>
                  }
                  <span>Editable · not saved</span>
                  <span className="merge-toolbar-spacer" />
                  <button
                    aria-label="Previous conflict"
                    disabled={!blocks.length}
                    onClick={() => {
                      const next =
                        (currentBlock - 1 + blocks.length) % blocks.length;
                      navigate(next);
                    }}
                  >
                    <CaretUp size={14} />
                  </button>
                  <span>
                    {blocks.length
                      ? `${currentBlock + 1} / ${blocks.length}`
                      : "No conflicts"}
                  </span>
                  <button
                    aria-label="Next conflict"
                    disabled={!blocks.length}
                    onClick={() => {
                      const next = (currentBlock + 1) % blocks.length;
                      navigate(next);
                    }}
                  >
                    <CaretDown size={14} />
                  </button>
                  <button
                    disabled={!blocks.length || saving}
                    onClick={() => choose("ours")}
                  >
                    Accept ours
                  </button>
                  <button
                    disabled={!blocks.length || saving}
                    onClick={() => choose("theirs")}
                  >
                    Accept theirs
                  </button>
                  <button
                    disabled={!blocks.length || saving}
                    onClick={() => choose("both")}
                  >
                    Accept both
                  </button>
                </header>
                <CodeEditor
                  value={result}
                  onChange={setResult}
                  editable={!saving && !remove}
                  filename={context.path}
                  label="Resolved file contents"
                  onSelectionChange={(range) => {
                    resultCursor.current = range;
                  }}
                  activeRange={active?.range}
                  navigation={navigation}
                  follow={true}
                />
              </section>
            </>
          )}
        </div>
        {aiOpen && (
          <aside className="merge-ai-panel" aria-label="Merge AI assistance">
            <header>
              <Sparkle size={17} />
              <strong>Merge assistance</strong>
              <button
                className="icon"
                aria-label="Hide merge AI"
                disabled={aiBusy}
                onClick={() => setAiOpen(false)}
              >
                <X size={15} />
              </button>
            </header>
            <p className="small">
              {config.provider}
              {config.model ? ` · ${config.model}` : " · configured model"}.
              Suggestions need your review.
            </p>
            {aiBusy && (
              <p role="status">
                {preparingAi ? "Preparing input…" : "Generating…"}
              </p>
            )}
            {aiRequest.notice && <p role="status">{aiRequest.notice}</p>}
            {aiRequest.pending && (
              <button
                disabled={aiRequest.stopping}
                onClick={() => void aiRequest.stop()}
              >
                {aiRequest.stopping ? "Stopping…" : "Stop generation"}
              </button>
            )}
            {aiInput && (
              <>
                <strong>Review AI input</strong>
                <p className="small">
                  Only this file’s on-disk versions are included. Unsaved editor
                  changes are not sent. Generation may consume your provider
                  quota.
                </p>
                <details>
                  <summary>
                    {new TextEncoder()
                      .encode(aiInput.text)
                      .length.toLocaleString()}{" "}
                    bytes · show exact input
                  </summary>
                  <pre>{aiInput.text}</pre>
                </details>
                <button className="primary" onClick={generate}>
                  Generate with {config.provider}
                </button>
              </>
            )}
            {answer && <div className="merge-ai-answer">{answer}</div>}
            {proposal && (
              <>
                <p className="merge-ai-answer">{proposal.explanation}</p>
                <details open>
                  <summary>
                    Proposed result{" "}
                    {proposal.resolution === null ? "· delete file" : ""}
                  </summary>
                  <pre>{proposal.resolution ?? "Delete this file"}</pre>
                </details>
                {proposalStale && (
                  <p className="small">
                    Your result changed after this request. Copy useful parts
                    manually or request a new suggestion.
                  </p>
                )}
                <button
                  disabled={proposalStale || saving}
                  onClick={() => {
                    setResult(proposal.resolution ?? "");
                    setRemove(proposal.resolution === null);
                    setProposal(null);
                  }}
                >
                  Insert proposal into result
                </button>
                <p className="small">
                  Inserting changes only this editor. Saving, staging and
                  committing are separate.
                </p>
              </>
            )}
          </aside>
        )}
      </div>
      {(review || discard) && (
        <div
          className="merge-inline-confirm"
          role="alertdialog"
          aria-label={discard ? "Discard editor changes" : "Save resolution"}
        >
          <span>
            {discard
              ? "Discard unsaved editor changes? Your repository file has not been changed by this editor."
              : `${remove ? "Delete" : "Save"} ${context.path} and stage this resolution? No commit or push.`}
          </span>
          <button
            disabled={saving}
            onClick={() => {
              setDiscard(false);
              setReview(false);
            }}
          >
            Back to editor
          </button>
          {discard && (
            <button
              disabled={
                saving || !!recoveryWarning || !canRecover || !!savedRecovery
              }
              onClick={() => {
                if (persistRecovery()) close();
              }}
            >
              Keep local draft and close
            </button>
          )}
          <button
            disabled={saving}
            className="primary"
            onClick={async () => {
              if (discard) {
                if (savedRecovery || discardDraft()) close();
                return;
              }
              setSaving(true);
              try {
                const success = await save(remove ? null : result);
                if (success) {
                  try {
                    clearRecovery();
                  } catch (error) {
                    reportWarning(
                      `Resolution saved, but its recovery copy could not be cleared: ${String(error)}`,
                    );
                  }
                  close();
                } else {
                  setReview(false);
                  setError(
                    "Save failed. Your result remains in this editor. If the repository changed, copy it before reopening.",
                  );
                }
              } finally {
                setSaving(false);
              }
            }}
          >
            {discard ? "Discard changes" : "Confirm save and stage"}
          </button>
        </div>
      )}
      <footer className="merge-editor-footer">
        <label className="merge-inline-check">
          <input
            type="checkbox"
            disabled={saving}
            checked={remove}
            onChange={(event) => setRemove(event.target.checked)}
          />
          Delete file as resolution
        </label>
        <span className="merge-toolbar-spacer" />
        <span role="status">{recoveryStatus}</span>
        <span>⌘/Ctrl Z undo · ⌘/Ctrl F find · ⌘/Ctrl S save</span>
        <button
          className="primary"
          disabled={
            !versions ||
            saving ||
            !!savedRecovery ||
            (!remove && hasConflictMarkers(result))
          }
          onClick={() => setReview(true)}
        >
          <FloppyDisk size={16} />
          {saving ? "Saving…" : "Save and mark resolved"}
        </button>
      </footer>
    </dialog>
  );
}

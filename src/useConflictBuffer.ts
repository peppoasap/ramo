import { useEffect, useRef, useState } from "react";
import { call } from "./bridge";
import {
  allowsMergeRecovery,
  mergeDraftKey,
  MergeDraftSession,
  type MergeDraft,
} from "./mergeDrafts";
export interface ResolutionContext {
  path: string;
  repoId: string;
  fingerprint: string;
}
export interface ConflictVersions {
  base: string | null;
  ours: string | null;
  theirs: string | null;
  working: string | null;
}
export function useConflictBuffer(context: ResolutionContext) {
  const [versions, setVersions] = useState<ConflictVersions | null>(null);
  const [loadError, setLoadError] = useState("");
  const [result, setResultState] = useState("");
  const resultRef = useRef("");
  const [remove, setRemoveState] = useState(false);
  const removeRef = useRef(false);
  const initialResult = useRef("");
  const draftSession = useRef<MergeDraftSession | null>(null);
  const pendingRecovery = useRef(false);
  const [savedRecovery, setSavedRecovery] = useState<MergeDraft | null>(null);
  const [recoveryStatus, setRecoveryStatus] = useState(
    "Preparing local recovery…",
  );
  const [recoveryWarning, setRecoveryWarning] = useState("");
  function persistRecovery(
    content = resultRef.current,
    deletion = removeRef.current,
  ): boolean {
    if (!draftSession.current || pendingRecovery.current) return false;
    try {
      if (content === initialResult.current && !deletion)
        draftSession.current.clear();
      else draftSession.current.save(context.fingerprint, content, deletion);
      setRecoveryStatus(
        content === initialResult.current && !deletion
          ? "Local recovery ready"
          : "Draft saved on this device",
      );
      setRecoveryWarning("");
      return true;
    } catch (error) {
      setRecoveryWarning(
        `Recovery copy not updated: ${String(error)} Your current result is still in this editor.`,
      );
      return false;
    }
  }
  function setResult(value: string | ((previous: string) => string)) {
    const next = typeof value === "function" ? value(resultRef.current) : value;
    resultRef.current = next;
    setResultState(next);
    persistRecovery(next);
  }
  function setRemove(value: boolean) {
    removeRef.current = value;
    setRemoveState(value);
    persistRecovery(resultRef.current, value);
  }
  function installBuffer(content: string, deletion: boolean) {
    resultRef.current = content;
    removeRef.current = deletion;
    setResultState(content);
    setRemoveState(deletion);
  }
  function discardDraft(): boolean {
    if (!draftSession.current) return true;
    try {
      draftSession.current.clear();
      return true;
    } catch (error) {
      setRecoveryWarning(`Could not discard the saved copy: ${String(error)}`);
      return false;
    }
  }
  useEffect(() => {
    let active = true;
    call<ConflictVersions>("conflict_file", {
      repoId: context.repoId,
      expected: context.fingerprint,
      path: context.path,
    }).then(
      async (value) => {
        const initial = value.working ?? value.ours ?? value.theirs ?? "";
        let session: MergeDraftSession | null = null;
        let draft: MergeDraft | null = null;
        let warning = "";
        if (allowsMergeRecovery(context.path)) {
          try {
            const key = await mergeDraftKey(context.repoId, context.path);
            session = new MergeDraftSession(localStorage, key);
            draft = session.load();
          } catch (error) {
            session = null;
            warning = `Local recovery unavailable: ${String(error)} Copy unsaved text before closing.`;
          }
        } else
          warning =
            "Automatic local recovery is disabled for this sensitive filename. Copy unsaved text before closing.";
        if (!active) return;
        initialResult.current = initial;
        const matches = draft?.fingerprint === context.fingerprint;
        installBuffer(
          matches ? draft!.content : initial,
          matches ? draft!.remove : false,
        );
        draftSession.current = session;
        pendingRecovery.current = !!draft && !matches;
        setSavedRecovery(draft && !matches ? draft : null);
        setRecoveryWarning(warning);
        setRecoveryStatus(
          matches
            ? "Recovered local draft · review before saving"
            : draft
              ? "Saved result from a different Git state needs review"
              : session
                ? "Local recovery ready"
                : "Local recovery unavailable",
        );
        setVersions(value);
      },
      (error) => {
        if (active) setLoadError(String(error));
      },
    );
    return () => {
      active = false;
    };
  }, [context]);
  function restoreSaved() {
    if (!savedRecovery) return;
    pendingRecovery.current = false;
    setSavedRecovery(null);
    installBuffer(savedRecovery.content, savedRecovery.remove);
    persistRecovery(savedRecovery.content, savedRecovery.remove);
  }
  function discardSaved() {
    if (!discardDraft()) return;
    pendingRecovery.current = false;
    setSavedRecovery(null);
    persistRecovery();
  }
  return {
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
    canRecover: !!draftSession.current,
    clearRecovery: () => draftSession.current?.clear(),
  };
}

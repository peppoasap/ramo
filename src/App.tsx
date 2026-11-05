import { LocalActions } from "./components/LocalActions";
import { useAiRequest } from "./useAiRequest";
import { stoppedNotice } from "./aiRequest";
import { WorktreeManager } from "./components/WorktreeManager";
import type { Worktree } from "./domain";
import { version as appVersion } from "../package.json";
import { RemotesDialog } from "./components/RemotesDialog";
import { AmendDialog } from "./components/AmendDialog";
import { CloneDialog } from "./components/CloneDialog";
import { ComparisonDialog } from "./components/ComparisonDialog";
import { StashDialog } from "./components/StashDialog";
import type { Stash } from "./domain";
import { WorktreeDialog } from "./components/WorktreeDialog";
import { reviewedPatchHash } from "./hunks";
import { OperationGuide } from "./components/OperationGuide";
import type { ResolutionContext } from "./components/ConflictResolver";
import {
  ContextMenu,
  type MenuState,
  type MenuItem,
} from "./components/ContextMenu";
import { branchMenu, commitMenu } from "./repositoryMenus";
import type { Branch } from "./domain";
import type { MouseEvent, KeyboardEvent as ReactKeyboardEvent } from "react";
import { loadDraft, saveDraft, clearCommittedDraft } from "./drafts";
import { Sidebar } from "./components/Sidebar";
import { Inspector } from "./components/Inspector";
import type { SelectedFile, Form, Confirmation } from "./domain";
import {
  useCallback,
  useEffect,
  useRef,
  useState,
  lazy,
  Suspense,
} from "react";
import {
  ArrowsClockwise,
  ArrowDown,
  ArrowUp,
  CaretDown,
  Check,
  Code,
  FolderOpen,
  DownloadSimple,
  GitBranch,
  GitFork,
  Plus,
  Sparkle,
  Warning,
  X,
  CircleNotch,
  GearSix,
} from "@phosphor-icons/react";
import { call, desktop } from "./bridge";
import type {
  Action,
  AiConfig,
  AiInput,
  Commit,
  FileChange,
  Repository,
  Snapshot,
} from "./domain";
import { graphDrop } from "./graphInteractions";
import { useGraphDrag } from "./useGraphDrag";
import { History } from "./components/History";
import { isStaged as staged, isConflicted } from "./domain";
import { Dialog } from "./components/Dialog";
import { ProviderDialog } from "./components/ProviderDialog";

const ConflictResolver = lazy(() =>
  import("./components/ConflictResolver").then((module) => ({
    default: module.ConflictResolver,
  })),
);

function errorText(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
function initialAi(): AiConfig {
  try {
    return (
      JSON.parse(localStorage.getItem("ramo.ai") || "null") || {
        provider: "codex",
        model: "",
      }
    );
  } catch {
    return { provider: "codex", model: "" };
  }
}
export function App() {
  const [resolution, setResolution] = useState<ResolutionContext | null>(null);
  const [menu, setMenu] = useState<MenuState | null>(null);
  const closeMenu = useCallback(() => setMenu(null), []);
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [tabs, setTabs] = useState<Repository[]>([]);
  const [recent, setRecent] = useState<string[]>([]);
  const [amend, setAmend] = useState<Snapshot | null>(null);
  const [commits, setCommits] = useState<Commit[]>([]);
  const [selected, setSelected] = useState<Commit | null>(null);
  const [files, setFiles] = useState<FileChange[]>([]);
  const [file, setFile] = useState<SelectedFile | null>(null);
  const [mode, setMode] = useState<"history" | "changes">("history");
  const [diff, setDiff] = useState("");
  const [diffLoading, setDiffLoading] = useState(false);
  const [filter, setFilter] = useState("");
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const draftsRef = useRef<Record<string, string>>({});
  const [draftWarnings, setDraftWarnings] = useState<Record<string, string>>(
    {},
  );
  const [busy, setBusy] = useState("");
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  useEffect(() => {
    if (!notice) return;
    const timer = window.setTimeout(() => setNotice(""), 6000);
    return () => window.clearTimeout(timer);
  }, [notice]);
  const [hasMore, setHasMore] = useState(false);
  const [remotes, setRemotes] = useState<Snapshot | null>(null);
  const [trust, setTrust] = useState(false);
  const trustPrompted = useRef(new Set<string>());
  const [cloneOpen, setCloneOpen] = useState(false);
  const [comparison, setComparison] = useState<{
    repoId: string;
    from: string;
    to: string;
  } | null>(null);
  const [stashDialog, setStashDialog] = useState<{
    repoId: string;
    fingerprint: string;
    stash: Stash;
  } | null>(null);
  const [worktreeManager, setWorktreeManager] = useState<{
    repoId: string;
    fingerprint: string;
    tree: Worktree;
  } | null>(null);
  const [worktreeDialog, setWorktreeDialog] = useState<{
    repoId: string;
    fingerprint: string;
    initialBranch?: string;
  } | null>(null);
  const [form, setFormState] = useState<Form | null>(null);
  const [confirmation, setConfirmationState] = useState<Confirmation | null>(
    null,
  );
  const [providersOpen, setProvidersOpen] = useState(false);
  const [ai, setAi] = useState<AiConfig>(initialAi);
  const [aiInput, setAiInput] = useState<AiInput | null>(null);
  const [aiResult, setAiResult] = useState("");
  const aiRequest = useAiRequest();
  const aiPending = aiRequest.pending;
  const context = useRef("");
  const revision = useRef(0);
  const snapshotRef = useRef(snapshot);
  snapshotRef.current = snapshot;
  useEffect(
    () => setMenu(null),
    [snapshot?.repo.id, snapshot?.fingerprint, busy],
  );
  const graphGesture = useGraphDrag(
    snapshot ? `${snapshot.repo.id}:${snapshot.fingerprint}` : "",
    !!snapshot && !busy && !aiPending,
    (source, target, context) => {
      const current = snapshotRef.current;
      if (!current) return;
      const commit =
        target === "current"
          ? null
          : commits.find((commit) => commit.oid === target);
      if (commit === undefined) return;
      const result = graphDrop(source, commit, current, context, !!busy);
      if (result.error) setError(result.error);
      if (result.confirmation) setConfirmation(result.confirmation);
      if (result.comparison)
        setComparison({ repoId: current.repo.id, ...result.comparison });
    },
  );
  function inspectOid(oid: string) {
    const commit = commits.find((c) => c.oid === oid);
    setMode("history");
    setFilter("");
    if (commit) {
      setSelected(commit);
      requestAnimationFrame(() =>
        document
          .getElementById(`commit-${oid}`)
          ?.scrollIntoView({ block: "center" }),
      );
    } else {
      setFilter(oid);
      setNotice(
        "This branch tip is outside loaded history. Load older commits to locate it.",
      );
    }
  }
  const menuCommands = {
    compare: (oid: string) => {
      if (snapshot)
        setComparison({
          repoId: snapshot.repo.id,
          from: oid,
          to: snapshot.head ?? oid,
        });
    },
    worktree: (branch: string) => {
      if (snapshot)
        setWorktreeDialog({
          repoId: snapshot.repo.id,
          fingerprint: snapshot.fingerprint,
          initialBranch: branch,
        });
    },
    action,
    form: setForm,
    confirm: setConfirmation,
    inspect: inspectOid,
    copy: (value: string) => {
      void navigator.clipboard.writeText(value).then(
        () => setNotice("Copied to clipboard."),
        (e) => setError(errorText(e)),
      );
    },
  };
  function openMenu(
    event: MouseEvent | ReactKeyboardEvent,
    title: string,
    items: MenuItem[],
  ) {
    event.preventDefault();
    event.stopPropagation();
    const target = event.currentTarget as HTMLElement;
    (target.matches("button")
      ? target
      : target.querySelector<HTMLButtonElement>(".commit-select, button")
    )?.focus();
    const box = event.currentTarget.getBoundingClientRect();
    const pointer = "clientX" in event && (event.clientX || event.clientY);
    setMenu({
      title,
      items,
      x: pointer ? event.clientX : box.left,
      y: pointer ? event.clientY : box.bottom,
    });
  }
  function branchContext(
    event: MouseEvent | ReactKeyboardEvent,
    branch: Branch,
  ) {
    if (snapshot)
      openMenu(
        event,
        branch.name,
        branchMenu(branch, snapshot, !!busy, menuCommands),
      );
  }
  function commitContext(
    event: MouseEvent | ReactKeyboardEvent,
    commit: Commit,
  ) {
    if (snapshot)
      openMenu(
        event,
        commit.oid.slice(0, 12),
        commitMenu(commit, snapshot, !!busy, menuCommands),
      );
  }
  function setForm(value: Form | null) {
    setFormState(
      value
        ? {
            ...value,
            repoId: value.repoId ?? snapshotRef.current?.repo.id,
            fingerprint: value.fingerprint ?? snapshotRef.current?.fingerprint,
          }
        : null,
    );
  }
  function setConfirmation(value: Confirmation | null) {
    setConfirmationState(
      value
        ? {
            ...value,
            repoId: value.repoId ?? snapshotRef.current?.repo.id,
            fingerprint: value.fingerprint ?? snapshotRef.current?.fingerprint,
          }
        : null,
    );
  }
  function stillReviewed(value: { repoId?: string; fingerprint?: string }) {
    return (
      value.repoId === snapshotRef.current?.repo.id &&
      value.fingerprint === snapshotRef.current?.fingerprint
    );
  }
  const draft = snapshot ? drafts[snapshot.repo.id] || "" : "";
  function updateDraft(repoId: string, text: string) {
    draftsRef.current = { ...draftsRef.current, [repoId]: text };
    setDrafts(draftsRef.current);
    try {
      saveDraft(localStorage, repoId, text);
      setDraftWarnings((v) => ({ ...v, [repoId]: "" }));
    } catch {
      setDraftWarnings((v) => ({
        ...v,
        [repoId]:
          "Draft could not be saved on this device. Copy it before closing Ramo.",
      }));
    }
  }
  const setDraft = (text: string) => {
    if (snapshot) updateDraft(snapshot.repo.id, text);
  };

  const refresh = useCallback(
    async (quiet = false, selectOid?: string, forceHistory = false) => {
      const current = snapshotRef.current;
      if (!current) return;
      const id = current.repo.id;
      const version = revision.current;
      if (!quiet) setBusy("Refreshing repository…");
      try {
        const next = await call<Snapshot>("repository_snapshot", {
          repoId: id,
        });
        if (context.current !== id || version !== revision.current) return;
        setSnapshot(next);
        if (
          forceHistory ||
          !quiet ||
          current.fingerprint !== next.fingerprint
        ) {
          const history = await call<Commit[]>("repository_history", {
            repoId: id,
            skip: 0,
          });
          if (context.current !== id || version !== revision.current) return;
          setCommits(history);
          setHasMore(history.length === 300);
          setSelected(
            (v) =>
              history.find((c) => c.oid === (selectOid || v?.oid)) ||
              history[0] ||
              null,
          );
        }
      } catch (e) {
        if (context.current === id) setError(errorText(e));
      } finally {
        if (!quiet) setBusy("");
      }
    },
    [],
  );

  function checkoutBranch(branch: Branch) {
    const current = snapshotRef.current;
    if (!current) return;
    const item = branchMenu(
      branch,
      current,
      !!busy || aiPending,
      menuCommands,
    ).find((item) => item.label.startsWith("Checkout"));
    if (item && !item.disabled) void item.run();
  }
  async function activate(next: Snapshot) {
    const askTrust =
      !next.repo.trusted && !trustPrompted.current.has(next.repo.id);
    trustPrompted.current.add(next.repo.id);
    setTrust(askTrust);
    const version = ++revision.current;
    context.current = next.repo.id;
    if (!Object.hasOwn(draftsRef.current, next.repo.id)) {
      try {
        draftsRef.current = {
          ...draftsRef.current,
          [next.repo.id]: loadDraft(localStorage, next.repo.id),
        };
        setDrafts(draftsRef.current);
      } catch {
        setDraftWarnings((v) => ({
          ...v,
          [next.repo.id]:
            "Saved draft could not be restored. Its stored record was left unchanged.",
        }));
      }
    }
    setSnapshot(next);
    setTabs((v) =>
      v.some((r) => r.id === next.repo.id)
        ? v.map((r) => (r.id === next.repo.id ? next.repo : r))
        : [...v, next.repo],
    );
    setMode("history");
    setSelected(null);
    setFile(null);
    setFiles([]);
    setDiff("");
    setCommits([]);
    setFilter("");
    setBusy("Reading history…");
    setError("");
    try {
      const history = await call<Commit[]>("repository_history", {
        repoId: next.repo.id,
        skip: 0,
      });
      if (version !== revision.current) return;
      setCommits(history);
      setSelected(history[0] || null);
      setHasMore(history.length === 300);
      setRecent(await call<string[]>("recent_repositories"));
    } catch (e) {
      if (version === revision.current) setError(errorText(e));
    } finally {
      if (version === revision.current) setBusy("");
    }
  }
  async function open(path?: string) {
    setBusy("Opening repository…");
    setError("");
    try {
      const next = path
        ? await call<Snapshot>("open_recent", { path })
        : await call<Snapshot | null>("open_repository");
      if (next) await activate(next);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy("");
    }
  }

  useEffect(() => {
    if (!desktop) return;
    let active = true;
    call<string[]>("recent_repositories")
      .then((v) => {
        if (active) setRecent(v);
      })
      .catch((e) => setError(errorText(e)));
    call<Snapshot | null>("startup_repository")
      .then((v) => {
        if (v && active) void activate(v);
      })
      .catch((e) => setError(errorText(e)));
    return () => {
      active = false;
    };
  }, []);
  useEffect(() => {
    const onFocus = () => {
      if (!busy && !aiPending) void refresh(true);
    };
    window.addEventListener("focus", onFocus);
    const timer = window.setInterval(() => {
      if (!document.hidden) onFocus();
    }, 30000);
    return () => {
      window.removeEventListener("focus", onFocus);
      clearInterval(timer);
    };
  }, [busy, aiPending, refresh]);
  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "o") {
        event.preventDefault();
        if (!busy) void open();
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "r") {
        event.preventDefault();
        if (!busy) void refresh();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  });

  useEffect(() => {
    let current = true;
    if (!snapshot) return;
    setFile(null);
    setDiff("");
    if (mode === "changes") {
      setFiles(snapshot.files);
      const first =
        snapshot.files.find(isConflicted) ||
        snapshot.files.find(staged) ||
        snapshot.files[0];
      if (first) setFile({ path: first.path, staged: staged(first) });
    } else if (selected) {
      call<FileChange[]>("commit_files", {
        repoId: snapshot.repo.id,
        commit: selected.oid,
      })
        .then((v) => {
          if (!current) return;
          setFiles(v);
          if (v[0]) setFile({ path: v[0].path, staged: false });
        })
        .catch((e) => {
          if (current) setError(errorText(e));
        });
    } else setFiles([]);
    return () => {
      current = false;
    };
  }, [snapshot?.repo.id, snapshot?.fingerprint, selected?.oid, mode]);

  useEffect(() => {
    let current = true;
    if (!snapshot || !file) {
      setDiff("");
      setDiffLoading(false);
      return;
    }
    setDiffLoading(true);
    setDiff("");
    call<string>("file_diff", {
      repoId: snapshot.repo.id,
      path: file.path,
      commit: mode === "history" ? selected?.oid || null : null,
      staged: file.staged,
    })
      .then((v) => {
        if (current) setDiff(v);
      })
      .catch((e) => {
        if (current) setError(errorText(e));
      })
      .finally(() => {
        if (current) setDiffLoading(false);
      });
    return () => {
      current = false;
    };
  }, [
    snapshot?.repo.id,
    snapshot?.fingerprint,
    selected?.oid,
    file?.path,
    file?.staged,
    mode,
  ]);

  async function applyHunk(index: number) {
    if (!snapshot || !file || busy || diffLoading || mode !== "changes") return;
    const reviewed = {
      repoId: snapshot.repo.id,
      fingerprint: snapshot.fingerprint,
    };
    const path = file.path,
      unstage = file.staged,
      patch = diff;
    try {
      const patchHash = await reviewedPatchHash(patch);
      if (!stillReviewed(reviewed))
        throw new Error(
          "Repository changed. Refresh and review this hunk again.",
        );
      await action({ type: "hunk", path, index, unstage, patchHash });
    } catch (error) {
      setError(errorText(error));
    }
  }
  async function action(action: Action) {
    const current = snapshotRef.current;
    if (!current || busy) return;
    if (!current.repo.trusted) {
      setTrust(true);
      return;
    }
    const labels: Record<Action["type"], string> = {
      stage: "Staging files…",
      hunk: "Updating selected hunk…",
      unstage: "Unstaging files…",
      commit: "Creating local commit…",
      amend: "Amending reviewed local commit…",
      configureRemote: "Updating remote endpoint…",
      markResolved: "Staging reviewed resolution…",
      resolveFile: "Saving reviewed resolution…",
      abortMerge: "Aborting merge…",
      sequence: "Updating Git sequence…",
      merge: "Merging branch…",
      switch: "Switching branch…",
      createBranch: "Creating branch…",
      createWorktree: "Creating worktree…",
      worktreeLock: "Updating worktree lock…",
      removeWorktree: "Removing reviewed checkout…",
      createBranchAt: "Creating branch at commit…",
      createTag: "Creating tag…",
      renameBranch: "Renaming branch…",
      trackBranch: "Checking out tracking branch…",
      checkoutCommit: "Checking out commit…",
      cherryPick: "Cherry-picking commit…",
      revert: "Reverting commit…",
      deleteBranch: "Deleting merged branch…",
      fetch: "Fetching remote history…",
      pull: "Pulling incoming commits…",
      push: "Pushing commits…",
      publish: "Publishing branch…",
      stash: "Saving stash…",
      applyStash: "Applying stash…",
      popStash: "Restoring and removing stash…",
      dropStash: "Removing stash…",
      restoreStashFile: "Restoring saved file…",
    };
    setBusy(labels[action.type]);
    setError("");
    setNotice("");
    try {
      const next = await call<Snapshot>("git_action", {
        repoId: current.repo.id,
        expected: current.fingerprint,
        action,
      });
      if (action.type === "commit") {
        try {
          clearCommittedDraft(localStorage, current.repo.id, action.message);
          if (draftsRef.current[current.repo.id] === action.message) {
            draftsRef.current = { ...draftsRef.current, [current.repo.id]: "" };
            setDrafts(draftsRef.current);
          }
        } catch {
          setDraftWarnings((v) => ({
            ...v,
            [current.repo.id]:
              "Commit succeeded, but its saved draft could not be cleared. Check history before committing it again.",
          }));
        }
      }
      if (context.current !== current.repo.id) return;
      setSnapshot(next);
      setNotice(
        action.type === "restoreStashFile"
          ? `Restored ${action.path}. Not staged; the stash is retained. Ignored files may not appear in Changes.`
          : action.type === "commit"
            ? "Local commit created. Push is a separate action."
            : action.type === "merge"
              ? "Local merge completed. Push is a separate action."
              : action.type === "createWorktree"
                ? "Worktree created. Open it from Worktrees."
                : "Operation completed.",
      );
      snapshotRef.current = next;
      if (action.type === "merge") setMode("history");
      if (["applyStash", "popStash", "restoreStashFile"].includes(action.type))
        setMode("changes");
      if (action.type === "sequence")
        setMode(next.operation ? "changes" : "history");
      const movesHead = [
        "commit",
        "amend",
        "merge",
        "switch",
        "createBranch",
        "pull",
        "checkoutCommit",
        "cherryPick",
        "revert",
        "trackBranch",
        "renameBranch",
        "sequence",
      ].includes(action.type);
      const changesHistory = ![
        "stage",
        "unstage",
        "markResolved",
        "hunk",
      ].includes(action.type);
      await refresh(
        true,
        movesHead ? next.head || undefined : undefined,
        changesHistory,
      );
      return true;
    } catch (e) {
      setError(errorText(e));
      await refresh(true);
      if (
        [
          "merge",
          "cherryPick",
          "revert",
          "sequence",
          "applyStash",
          "popStash",
        ].includes(action.type) &&
        context.current === current.repo.id
      )
        setMode("changes");
      return false;
    } finally {
      setBusy("");
    }
  }
  async function trustRepository() {
    if (!snapshot) return;
    setBusy("Enabling operations…");
    try {
      const next = await call<Snapshot>("trust_repository", {
        repoId: snapshot.repo.id,
      });
      setSnapshot(next);
      snapshotRef.current = next;
      setTabs((tabs) =>
        tabs.map((repo) => (repo.id === next.repo.id ? next.repo : repo)),
      );
      setTrust(false);
      setNotice(
        "Repository trusted for this session. You can now run Git operations.",
      );
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy("");
    }
  }
  async function openWorktree(path: string) {
    if (!snapshot) return;
    setBusy("Opening worktree…");
    try {
      await activate(
        await call<Snapshot>("open_worktree", {
          repoId: snapshot.repo.id,
          path,
        }),
      );
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy("");
    }
  }
  async function more() {
    if (!snapshot || busy) return;
    const id = snapshot.repo.id;
    setBusy("Reading older commits…");
    try {
      const page = await call<Commit[]>("repository_history", {
        repoId: id,
        skip: commits.length,
      });
      if (context.current === id) {
        setCommits((v) => [
          ...v,
          ...page.filter((c) => !v.some((p) => p.oid === c.oid)),
        ]);
        setHasMore(page.length === 300);
      }
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy("");
    }
  }
  function showProviders() {
    setProvidersOpen(true);
  }
  async function prepareAi(kind: "commit" | "explain") {
    if (!snapshot || busy || aiPending) return;
    aiRequest.clearNotice();
    setBusy("Preparing selected AI input…");
    setAiResult("");
    try {
      const input = await call<AiInput>("prepare_ai", {
        repoId: snapshot.repo.id,
        kind,
        path: file?.path || null,
        commit: mode === "history" ? selected?.oid || null : null,
        staged: file?.staged || false,
      });
      if (context.current === input.repoId) setAiInput(input);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy("");
    }
  }
  async function generateAi() {
    if (!aiInput || aiPending) return;
    setError("");
    try {
      const answer = await aiRequest.run(aiInput.id, ai);
      if (answer === null) {
        setAiInput(null);
        setAiResult("");
        setNotice(stoppedNotice);
      } else if (context.current === aiInput.repoId) setAiResult(answer);
    } catch (e) {
      setError(errorText(e));
      setAiInput(null);
    }
  }
  function saveProvider(config: AiConfig) {
    setAi(config);
    localStorage.setItem("ramo.ai", JSON.stringify(config));
    setProvidersOpen(false);
    setNotice(
      "AI provider preference saved. Authentication stays with your provider; no account has been imported.",
    );
  }
  return (
    <main className="app">
      <header className="topbar">
        <div className="brand">
          <GitBranch size={25} />
          <strong>Ramo</strong>
          <span className="version">{appVersion}</span>
        </div>
        <nav className="repo-tabs" aria-label="Open repositories">
          {tabs.map((repo) => (
            <div
              className={`repo-tab ${repo.id === snapshot?.repo.id ? "active" : ""}`}
              key={repo.id}
            >
              <button
                disabled={!!busy || aiPending}
                onClick={() => {
                  if (snapshot?.repo.id !== repo.id) void open(repo.path);
                }}
                title={repo.path}
              >
                <Code size={18} />
                {repo.name}
              </button>
              <button
                className="tab-close"
                disabled={!!busy || aiPending}
                aria-label={`Close ${repo.name}`}
                onClick={async () => {
                  setBusy("Closing repository…");
                  try {
                    await call<void>("close_repository", { repoId: repo.id });
                    trustPrompted.current.delete(repo.id);
                  } catch (error) {
                    setError(errorText(error));
                    setBusy("");
                    return;
                  }
                  const rest = tabs.filter((r) => r.id !== repo.id);
                  setTabs(rest);
                  if (snapshot?.repo.id === repo.id) {
                    if (rest.length) await open(rest[rest.length - 1].path);
                    else {
                      ++revision.current;
                      context.current = "";
                      setSnapshot(null);
                      setCommits([]);
                      setFile(null);
                      setDiff("");
                    }
                  }
                  setBusy("");
                }}
              >
                <X size={13} />
              </button>
            </div>
          ))}
        </nav>
        <div className="topbar-actions">
          <button
            className="icon"
            title="Open repository (⌘/Ctrl O)"
            aria-label="Open repository"
            disabled={!!busy || aiPending}
            onClick={() => open()}
          >
            <Plus size={22} />
          </button>
          <button
            disabled={!!busy || aiPending || !desktop}
            onClick={() => setCloneOpen(true)}
            title="Clone repository"
          >
            <DownloadSimple size={17} /> Clone
          </button>
          <button
            className="icon"
            title="AI providers"
            aria-label="AI providers"
            onClick={showProviders}
          >
            <GearSix size={20} />
          </button>
        </div>
      </header>
      {error && (
        <div className="banner error" role="alert">
          <Warning size={18} />
          <span>{error}</span>
          <button
            className="icon"
            aria-label="Dismiss error"
            onClick={() => setError("")}
          >
            <X size={16} />
          </button>
        </div>
      )}
      {notice && (
        <div className="banner notice" role="status">
          <Check size={17} />
          <span>{notice}</span>
          <button
            className="icon"
            aria-label="Dismiss notification"
            onClick={() => setNotice("")}
          >
            <X size={16} />
          </button>
        </div>
      )}
      {!snapshot ? (
        <section className="welcome">
          <GitFork size={55} weight="light" />
          <h1>A clearer view of your code.</h1>
          <p>
            Open a repository. Follow its history.
            <br />
            Make your next commit with confidence.
          </p>
          <button
            className="primary"
            disabled={!!busy || !desktop}
            onClick={() => open()}
          >
            <FolderOpen size={20} />
            Open repository
          </button>
          <button
            disabled={!!busy || !desktop}
            onClick={() => setCloneOpen(true)}
          >
            Clone repository
          </button>
          {!desktop && (
            <p className="desktop-required">
              Git access is available in the desktop application.
              <br />
              Run <code>npm run desktop</code> or open the installed app.
            </p>
          )}
          {recent.length > 0 && (
            <div className="recent">
              <h2>RECENT REPOSITORIES</h2>
              {recent.map((path) => (
                <button key={path} disabled={!!busy} onClick={() => open(path)}>
                  <FolderOpen size={18} />
                  <span>
                    <strong>{path.split(/[\\/]/).pop()}</strong>
                    <small>{path}</small>
                  </span>
                </button>
              ))}
            </div>
          )}
          <small className="welcome-footer">
            Local first · Apache 2.0 · Your tools, your workflow
          </small>
        </section>
      ) : (
        <>
          <div className="commandbar">
            <button
              className="active-branch"
              disabled={
                !!busy ||
                aiPending ||
                !snapshot.branches.some((branch) => !branch.remote)
              }
              aria-label={`Switch branch: ${snapshot.branch}`}
              title="Switch branch"
              onClick={(event) =>
                openMenu(
                  event,
                  "Switch branch",
                  snapshot.branches
                    .filter((branch) => !branch.remote)
                    .map((branch) => {
                      const checkout = branchMenu(
                        branch,
                        snapshot,
                        !!busy,
                        menuCommands,
                      ).find((item) => item.label === "Checkout branch")!;
                      return {
                        ...checkout,
                        label:
                          branch.name +
                          (branch.name === snapshot.branch ? " (current)" : ""),
                      };
                    }),
                )
              }
            >
              <GitBranch size={21} />
              <strong>{snapshot.branch}</strong>
              <CaretDown size={12} />
            </button>
            <div
              className="sync-state"
              title={snapshot.upstream || "No upstream configured"}
            >
              {snapshot.upstream
                ? `${snapshot.ahead ?? "?"} ahead · ${snapshot.behind ?? "?"} behind`
                : "No upstream"}
            </div>
            <div className="command-separator" />
            <button
              disabled={!!busy || aiPending}
              onClick={() => action({ type: "fetch" })}
            >
              <ArrowsClockwise size={18} />
              Fetch
            </button>
            <button
              disabled={
                !!busy ||
                aiPending ||
                !snapshot.upstream ||
                !!snapshot.operation ||
                snapshot.files.some(isConflicted)
              }
              onClick={() =>
                setConfirmation({
                  action: { type: "pull" },
                  title: "Pull fast-forward only",
                  description: `Update ${snapshot.branch} from ${snapshot.upstream}. Divergent history will be refused; no automatic rebase or merge.`,
                })
              }
            >
              <ArrowDown size={18} />
              Pull
            </button>
            <button
              disabled={
                !!busy ||
                aiPending ||
                (snapshot.behind ?? 0) > 0 ||
                !!snapshot.operation ||
                snapshot.files.some(isConflicted)
              }
              title={
                (snapshot.behind ?? 0) > 0
                  ? "Review and pull incoming commits before pushing"
                  : snapshot.upstream
                    ? "Push commits to the configured upstream"
                    : "Publish this branch and configure its upstream"
              }
              onClick={() =>
                snapshot.upstream
                  ? setConfirmation({
                      action: { type: "push" },
                      title: "Push local commits",
                      description: `Send ${snapshot.ahead ?? ""} outgoing commits from ${snapshot.repo.name} / ${snapshot.branch} to ${snapshot.upstream}. No force push.`,
                    })
                  : setForm({
                      kind: "publish",
                      title: "Publish branch to a remote",
                      value: "origin",
                    })
              }
            >
              <ArrowUp size={18} />
              {snapshot.upstream ? "Push" : "Publish"}
              {snapshot.ahead ? (
                <span className="count">{snapshot.ahead}</span>
              ) : null}
            </button>
            <div className="command-separator" />
            <LocalActions
              snapshot={snapshot}
              locked={!!busy || aiPending}
              branch={() =>
                setForm({
                  kind: "createBranch",
                  title: "Create and switch to branch",
                  value: "",
                })
              }
              stash={() =>
                setForm({
                  kind: "stash",
                  title: "Stash working changes",
                  value: "",
                })
              }
              pop={() => {
                const stash = snapshot.stashes[0];
                if (stash)
                  setStashDialog({
                    repoId: snapshot.repo.id,
                    fingerprint: snapshot.fingerprint,
                    stash,
                  });
              }}
              more={(event) =>
                openMenu(event, "Repository actions", [
                  { label: "Manage remotes…", run: () => setRemotes(snapshot) },
                  {
                    label: "Create worktree…",
                    disabled:
                      !!snapshot.operation ||
                      snapshot.files.some(isConflicted) ||
                      !snapshot.head,
                    run: () =>
                      setWorktreeDialog({
                        repoId: snapshot.repo.id,
                        fingerprint: snapshot.fingerprint,
                      }),
                  },
                  {
                    label: "Compare revisions…",
                    disabled: !snapshot.head,
                    run: () =>
                      setComparison({
                        repoId: snapshot.repo.id,
                        from: selected?.oid ?? snapshot.head!,
                        to: snapshot.head!,
                      }),
                  },
                ])
              }
            />
            <div className="toolbar-spacer" />
            <button
              className="icon"
              disabled={!!busy || aiPending}
              aria-label="Refresh repository"
              title="Refresh (⌘/Ctrl R)"
              onClick={() => refresh()}
            >
              <ArrowsClockwise className={busy ? "spin" : ""} size={18} />
            </button>
          </div>
          <OperationGuide
            snapshot={snapshot}
            busy={!!busy}
            review={() => setMode("changes")}
            confirm={setConfirmation}
          />
          <div
            className="workbench"
            onClickCapture={(event) => {
              if (graphGesture.suppressClick.current) {
                event.preventDefault();
                event.stopPropagation();
                graphGesture.suppressClick.current = false;
              }
            }}
          >
            <Sidebar
              key={snapshot.repo.id}
              gesture={graphGesture}
              inspectBranch={inspectOid}
              checkoutBranch={checkoutBranch}
              inspectStash={(stash) =>
                setStashDialog({
                  repoId: snapshot.repo.id,
                  fingerprint: snapshot.fingerprint,
                  stash,
                })
              }
              createWorktree={() =>
                setWorktreeDialog({
                  repoId: snapshot.repo.id,
                  fingerprint: snapshot.fingerprint,
                })
              }
              onBranchContext={branchContext}
              snapshot={snapshot}
              busy={busy}
              setForm={setForm}
              manageWorktree={(tree) => {
                setError("");
                setWorktreeManager({
                  repoId: snapshot.repo.id,
                  fingerprint: snapshot.fingerprint,
                  tree,
                });
              }}
              openWorktree={openWorktree}
            />
            <div className="history-panel">
              <div className="view-tabs">
                <button
                  aria-pressed={mode === "history"}
                  className={mode === "history" ? "active" : ""}
                  onClick={() => setMode("history")}
                >
                  History
                </button>
                <button
                  aria-pressed={mode === "changes"}
                  className={mode === "changes" ? "active" : ""}
                  onClick={() => setMode("changes")}
                >
                  Changes <span className="count">{snapshot.files.length}</span>
                </button>
                <span className="scope">All branches</span>
                <button
                  className="compare-action"
                  disabled={!!busy || !snapshot.head}
                  onClick={() =>
                    snapshot.head &&
                    setComparison({
                      repoId: snapshot.repo.id,
                      from: selected?.oid ?? snapshot.head,
                      to: snapshot.head,
                    })
                  }
                >
                  Compare revisions
                </button>
              </div>
              <History
                key={snapshot.repo.id}
                branches={snapshot.branches}
                onBranchCheckout={checkoutBranch}
                onBranchOptions={(branch, anchor) =>
                  setMenu({
                    ...anchor,
                    title: branch.name,
                    items: branchMenu(branch, snapshot, !!busy, menuCommands),
                  })
                }
                gesture={graphGesture}
                currentBranch={snapshot.branch}
                onBranchContext={branchContext}
                onCommitContext={commitContext}
                commits={commits}
                selected={mode === "history" ? selected?.oid || null : null}
                onSelect={(commit) => {
                  setSelected(commit);
                  setMode("history");
                }}
                filter={filter}
                onFilter={setFilter}
                onMore={more}
                hasMore={hasMore}
                busy={!!busy}
              />
            </div>
            <Inspector
              amend={() => setAmend(snapshot)}
              onHunk={applyHunk}
              resolveConflict={(path) =>
                setResolution({
                  path,
                  repoId: snapshot.repo.id,
                  fingerprint: snapshot.fingerprint,
                })
              }
              mode={mode}
              snapshot={snapshot}
              selected={selected}
              files={files}
              file={file}
              setFile={setFile}
              busy={busy}
              aiPending={aiPending}
              diff={diff}
              diffLoading={diffLoading}
              setConfirmation={setConfirmation}
              draft={draft}
              draftWarning={
                snapshot ? draftWarnings[snapshot.repo.id] || "" : ""
              }
              setDraft={setDraft}
              ai={ai}
              action={action}
              prepareAi={prepareAi}
              showProviders={showProviders}
            />
          </div>
        </>
      )}
      <footer className="statusbar">
        <span>
          <Code size={16} />
          {snapshot
            ? `${snapshot.repo.name} · ${snapshot.branch}`
            : "Ramo · local first"}
        </span>
        <span role="status">
          {busy || aiPending ? (
            <>
              <CircleNotch size={14} className="spin" />
              {busy || "Generating with your AI provider…"}
            </>
          ) : (
            <>
              <span className="ready-dot" />
              Ready
            </>
          )}
        </span>
      </footer>
      {trust && (
        <Dialog
          title="Trust this repository?"
          onClose={() => setTrust(false)}
          locked={!!busy}
        >
          <p>
            Git operations can run this repository’s configured hooks, filters
            and credential helpers as local programs.
          </p>
          <p className="path-block">{snapshot?.repo.path}</p>
          <p>
            Only enable writes and network actions for a repository you trust.
            Read-only inspection remains available without this.
          </p>
          <div className="dialog-actions">
            <button onClick={() => setTrust(false)}>Keep read-only</button>
            <button
              className="primary"
              disabled={!!busy}
              onClick={trustRepository}
            >
              Trust for this session
            </button>
          </div>
        </Dialog>
      )}
      {resolution && (
        <Suspense fallback={<div role="status">Opening merge editor…</div>}>
          <ConflictResolver
            context={resolution}
            config={ai}
            configure={showProviders}
            reportWarning={setNotice}
            close={() => setResolution(null)}
            save={async (content) => {
              if (!stillReviewed(resolution)) {
                setError(
                  "Repository changed. Copy your resolution and reopen the editor after refreshing.",
                );
                return false;
              }
              return Boolean(
                await action({
                  type: "resolveFile",
                  path: resolution.path,
                  content,
                }),
              );
            }}
          />
        </Suspense>
      )}
      {menu && <ContextMenu menu={menu} close={closeMenu} />}
      {cloneOpen && (
        <CloneDialog close={() => setCloneOpen(false)} opened={activate} />
      )}
      {comparison && snapshot && snapshot.repo.id === comparison.repoId && (
        <ComparisonDialog
          repoId={comparison.repoId}
          initialFrom={comparison.from}
          initialTo={comparison.to}
          snapshot={snapshot}
          commits={commits}
          close={() => setComparison(null)}
        />
      )}
      {stashDialog && snapshot && snapshot.repo.id === stashDialog.repoId && (
        <StashDialog
          repoId={stashDialog.repoId}
          stash={stashDialog.stash}
          snapshot={snapshot}
          close={() => setStashDialog(null)}
          run={async (value) => {
            if (!stillReviewed(stashDialog))
              throw new Error(
                "Repository changed. Close this stash view, refresh and review again.",
              );
            if (!snapshotRef.current?.repo.trusted) {
              setTrust(true);
              return "trust";
            }
            return (await action(value)) ? "done" : "failed";
          }}
        />
      )}
      {worktreeManager && snapshot?.repo.id === worktreeManager.repoId && (
        <WorktreeManager
          tree={worktreeManager.tree}
          error={error}
          opened={tabs.some((tab) => tab.path === worktreeManager.tree.path)}
          close={() => setWorktreeManager(null)}
          run={async (value) => {
            if (!stillReviewed(worktreeManager)) {
              setError(
                "Repository changed. Close this dialog and review the worktree again.",
              );
              return "failed";
            }
            if (!snapshotRef.current?.repo.trusted) {
              setTrust(true);
              return "trust";
            }
            return (await action(value)) ? "done" : "failed";
          }}
        />
      )}
      {worktreeDialog &&
        snapshot &&
        snapshot.repo.id === worktreeDialog.repoId && (
          <WorktreeDialog
            snapshot={snapshot}
            initialBranch={worktreeDialog.initialBranch}
            close={() => setWorktreeDialog(null)}
            create={async (value) => {
              if (!stillReviewed(worktreeDialog)) {
                setError(
                  "Repository changed while the worktree dialog was open. Close it and review again.",
                );
                return "failed";
              }
              if (!snapshotRef.current?.repo.trusted) {
                setTrust(true);
                return "trust";
              }
              return (await action(value)) ? "created" : "failed";
            }}
          />
        )}
      {form && (
        <Dialog
          title={form.title}
          className="operation-dialog"
          locked={!!busy}
          onClose={() => setForm(null)}
        >
          <form
            onSubmit={(e) => {
              e.preventDefault();
              if (!snapshot?.repo.trusted) {
                setTrust(true);
                return;
              }
              const current = form;
              setForm(null);
              if (!stillReviewed(current)) {
                setError(
                  "Repository changed while this dialog was open. Review it again.",
                );
                return;
              }
              void action(
                current.kind === "stash"
                  ? { type: "stash", message: current.value }
                  : current.kind === "publish"
                    ? { type: "publish", remote: current.value }
                    : current.kind === "createBranchAt" ||
                        current.kind === "createTag"
                      ? {
                          type: current.kind,
                          name: current.value,
                          target: current.target!,
                        }
                      : current.kind === "renameBranch"
                        ? {
                            type: "renameBranch",
                            name: current.target!,
                            newName: current.value,
                          }
                        : current.kind === "trackBranch"
                          ? {
                              type: "trackBranch",
                              name: current.value,
                              remoteBranch: current.target!,
                            }
                          : { type: "createBranch", name: current.value },
              );
            }}
          >
            <label>
              {form.kind === "publish"
                ? "Configured remote"
                : form.kind === "stash"
                  ? "Stash message"
                  : form.kind === "createTag"
                    ? "Tag name"
                    : "Branch name"}
              <input
                autoFocus
                disabled={!!busy}
                required
                maxLength={1024}
                value={form.value}
                onChange={(e) => setForm({ ...form, value: e.target.value })}
                placeholder={
                  form.kind === "createBranch" ? "feat/my-change" : ""
                }
              />
            </label>
            {form.target && (
              <p className="small">
                Target: <code>{form.target}</code>
                {form.kind === "createBranchAt"
                  ? " · Creates a branch without switching your current worktree."
                  : form.kind === "createTag"
                    ? " · Creates a local lightweight tag; push is separate."
                    : ""}
              </p>
            )}
            {form.kind === "publish" && (
              <p>
                This sends the current branch to the named remote and sets its
                upstream.
              </p>
            )}
            <div className="dialog-actions">
              <button type="button" onClick={() => setForm(null)}>
                Cancel
              </button>
              <button className="primary" type="submit">
                {form.kind === "publish"
                  ? "Publish branch"
                  : form.kind === "stash"
                    ? "Create stash"
                    : form.kind === "renameBranch"
                      ? "Rename branch"
                      : form.kind === "createTag"
                        ? "Create tag"
                        : form.kind === "trackBranch"
                          ? "Checkout tracking branch"
                          : "Create branch"}
              </button>
            </div>
          </form>
        </Dialog>
      )}
      {remotes && snapshot?.repo.id === remotes.repo.id && (
        <RemotesDialog
          snapshot={remotes}
          busy={!!busy}
          close={() => setRemotes(null)}
          save={async (value) => {
            if (
              !stillReviewed({
                repoId: remotes.repo.id,
                fingerprint: remotes.fingerprint,
              })
            ) {
              throw new Error(
                "Repository or remotes changed. Close, refresh and review again.",
              );
            }
            if (!snapshotRef.current?.repo.trusted) {
              setTrust(true);
              return "trust";
            }
            return (await action(value)) ? "done" : "failed";
          }}
        />
      )}
      {amend && snapshot?.repo.id === amend.repo.id && (
        <AmendDialog
          snapshot={amend}
          busy={!!busy}
          close={() => setAmend(null)}
          submit={async (message) => {
            if (
              !stillReviewed({
                repoId: amend.repo.id,
                fingerprint: amend.fingerprint,
              })
            ) {
              setError(
                "Repository changed during amend review. Close and review again.",
              );
              return false;
            }
            if (!snapshotRef.current?.repo.trusted) {
              setTrust(true);
              return "trust";
            }
            return action({ type: "amend", head: amend.head!, message });
          }}
        />
      )}
      {confirmation && (
        <Dialog
          title={confirmation.title}
          onClose={() => setConfirmation(null)}
        >
          <p>{confirmation.description}</p>
          <div className="dialog-actions">
            <button onClick={() => setConfirmation(null)}>Cancel</button>
            <button
              className="primary"
              onClick={() => {
                const requested = confirmation.action;
                if (!snapshotRef.current?.repo.trusted) {
                  setTrust(true);
                  return;
                }
                setConfirmation(null);
                if (!stillReviewed(confirmation)) {
                  setError(
                    "Repository changed while this dialog was open. Review it again.",
                  );
                  return;
                }
                void action(requested);
              }}
            >
              Continue
            </button>
          </div>
        </Dialog>
      )}
      {providersOpen && (
        <ProviderDialog
          config={ai}
          onSave={saveProvider}
          onClose={() => setProvidersOpen(false)}
        />
      )}
      {aiInput && (
        <Dialog
          title={
            aiInput.kind === "commit"
              ? aiResult
                ? "Review AI commit message"
                : "Review input before sending"
              : aiResult
                ? "Diff explanation"
                : "Review input before sending"
          }
          wide
          onClose={() => {
            setAiInput(null);
            setAiResult("");
          }}
          locked={aiPending}
        >
          {aiRequest.notice && <p role="status">{aiRequest.notice}</p>}
          {aiResult ? (
            <>
              <pre className="ai-result">{aiResult}</pre>
              <div className="dialog-actions">
                <button
                  onClick={() => {
                    setAiInput(null);
                    setAiResult("");
                  }}
                >
                  Close
                </button>
                {aiInput.kind === "commit" && (
                  <button
                    className="primary"
                    onClick={() => {
                      if (
                        context.current === aiInput.repoId &&
                        snapshot?.fingerprint === aiInput.fingerprint
                      ) {
                        setDraft(aiResult);
                        setAiInput(null);
                        setAiResult("");
                      } else {
                        setError(
                          "Repository changed. Prepare a new AI input before using this message.",
                        );
                      }
                    }}
                  >
                    Use message
                  </button>
                )}
              </div>
            </>
          ) : (
            <>
              <p>
                Send{" "}
                <strong>
                  {aiInput.paths.length} selected{" "}
                  {aiInput.paths.length === 1 ? "file" : "files"}
                </strong>{" "}
                to <strong>{ai.provider}</strong>
                {ai.model ? ` / ${ai.model}` : ""}. Review the exact input
                below. No commit or push will run.
              </p>
              <div className="input-paths">
                {aiInput.paths.map((path) => (
                  <code key={path}>{path}</code>
                ))}
              </div>
              {aiInput.excluded.length > 0 && (
                <p className="warning-text">
                  Excluded sensitive, binary or oversized files:{" "}
                  {aiInput.excluded.join(", ")}. The suggestion will cover only
                  the included changes.
                </p>
              )}
              <pre className="ai-preview">{aiInput.text}</pre>
              <div className="dialog-actions">
                <button
                  disabled={aiPending}
                  onClick={() => {
                    setAiInput(null);
                    setAiResult("");
                  }}
                >
                  Cancel
                </button>
                {aiPending && (
                  <button
                    disabled={aiRequest.stopping}
                    onClick={() => void aiRequest.stop()}
                  >
                    {aiRequest.stopping ? "Stopping…" : "Stop generation"}
                  </button>
                )}
                <button disabled={aiPending} onClick={showProviders}>
                  Change provider
                </button>
                <button
                  className="primary"
                  disabled={aiPending}
                  onClick={generateAi}
                >
                  {aiPending ? (
                    <>
                      <CircleNotch className="spin" size={16} />
                      Generating…
                    </>
                  ) : (
                    <>
                      <Sparkle size={16} />
                      Send and generate
                    </>
                  )}
                </button>
              </div>
            </>
          )}
        </Dialog>
      )}
    </main>
  );
}

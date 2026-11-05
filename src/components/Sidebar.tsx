import {
  useState,
  type MouseEvent,
  type KeyboardEvent,
  type ReactNode,
} from "react";
import type { Branch, Stash, Worktree, Snapshot, Form } from "../domain";
import {
  DotsThree,
  GitBranch,
  Plus,
  Check,
  HardDrives,
  Stack,
  FolderOpen,
  CaretRight,
  MagnifyingGlass,
} from "@phosphor-icons/react";
import { isConflicted } from "../domain";
import type { GraphGesture } from "../useGraphDrag";
interface Props {
  createWorktree: () => void;
  manageWorktree: (tree: Worktree) => void;
  inspectStash: (stash: Stash) => void;
  onBranchContext: (event: MouseEvent | KeyboardEvent, branch: Branch) => void;
  inspectBranch: (oid: string) => void;
  checkoutBranch: (branch: Branch) => void;
  snapshot: Snapshot;
  busy: string;
  setForm: (form: Form | null) => void;
  openWorktree: (path: string) => Promise<void>;
  gesture?: GraphGesture;
}
function Group({
  label,
  count,
  icon,
  action,
  children,
  initial = true,
}: {
  label: string;
  count: number;
  icon: ReactNode;
  action?: ReactNode;
  children: ReactNode;
  initial?: boolean;
}) {
  const [expanded, setExpanded] = useState(initial);
  return (
    <section className="sidebar-group">
      <div className="section-heading">
        <button
          className="section-toggle"
          aria-expanded={expanded}
          onClick={() => setExpanded(!expanded)}
        >
          <CaretRight size={11} className={expanded ? "expanded" : ""} />
          {icon}
          <span>{label}</span>
          <b>{count}</b>
        </button>
        {action}
      </div>
      {expanded && <div className="sidebar-group-content">{children}</div>}
    </section>
  );
}
export function Sidebar({
  inspectStash,
  createWorktree,
  manageWorktree,
  onBranchContext,
  inspectBranch,
  checkoutBranch,
  snapshot,
  busy,
  setForm,
  openWorktree,
  gesture,
}: Props) {
  const [query, setQuery] = useState("");
  const unrelatedBlocked =
    !!snapshot.operation || snapshot.files.some(isConflicted);
  const locals = snapshot.branches.filter((b) => !b.remote);
  const remotes = snapshot.branches.filter((b) => b.remote);
  const matches = (branch: Branch) =>
    branch.name.toLowerCase().includes(query.trim().toLowerCase());
  function branchRow(branch: Branch) {
    const current = !branch.remote && snapshot.branch === branch.name;
    return (
      <div
        key={`${branch.remote}-${branch.name}`}
        className={`branch-row ${current ? "active" : ""}`}
        onContextMenu={(e) => onBranchContext(e, branch)}
        onKeyDown={(e) => {
          if (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10"))
            onBranchContext(e, branch);
        }}
      >
        <button
          {...gesture?.sourceEvents({ kind: "branch", branch })}
          data-graph-target={current ? "current" : undefined}
          className={`${branch.remote ? "remote-row" : ""} ${current && gesture?.dropTarget === "current" ? "drop-active" : ""}`}
          title={`Click to inspect ${branch.name} · Double-click to ${branch.remote ? "create local tracking branch" : "checkout"} · Drag to compare${branch.remote ? "" : " or merge into the current branch"}`}
          onClick={() => inspectBranch(branch.oid)}
          onDoubleClick={() => checkoutBranch(branch)}
        >
          <GitBranch size={15} />
          <span>{branch.name}</span>
          {current && <Check size={14} />}
        </button>
        <button
          className="icon branch-options"
          aria-label={`Branch actions: ${branch.name}`}
          title="Branch actions"
          onClick={(e) => onBranchContext(e, branch)}
        >
          <DotsThree size={17} />
        </button>
      </div>
    );
  }
  return (
    <aside className="sidebar" aria-label="Repository navigation">
      <label className="sidebar-search">
        <MagnifyingGlass size={14} />
        <input
          aria-label="Filter branches"
          placeholder="Filter branches…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Escape") setQuery("");
          }}
        />
      </label>
      <Group
        label="Local branches"
        count={locals.length}
        icon={<GitBranch size={14} />}
        action={
          <button
            className="icon"
            aria-label="Create branch"
            title="Create branch"
            disabled={!!busy || unrelatedBlocked}
            onClick={() =>
              setForm({
                kind: "createBranch",
                title: "Create and switch to branch",
                value: "",
              })
            }
          >
            <Plus size={14} />
          </button>
        }
      >
        {locals.filter(matches).map(branchRow)}
        {!locals.filter(matches).length && (
          <p className="sidebar-empty">
            {query ? "No matching local branches" : "No local branches"}
          </p>
        )}
      </Group>
      <Group
        label="Remote branches"
        count={remotes.length}
        icon={<HardDrives size={14} />}
        initial={remotes.length > 0}
      >
        {remotes.filter(matches).map(branchRow)}
        {!remotes.filter(matches).length && (
          <p className="sidebar-empty">
            {query ? "No matching remote branches" : "No remote branches"}
          </p>
        )}
      </Group>
      <Group
        label="Worktrees"
        count={snapshot.worktrees.length}
        icon={<Stack size={14} />}
        action={
          <button
            className="icon"
            aria-label="Create worktree"
            title="Create worktree"
            disabled={!!busy || unrelatedBlocked || !snapshot.head}
            onClick={createWorktree}
          >
            <Plus size={14} />
          </button>
        }
      >
        {snapshot.worktrees.map((tree) => (
          <div key={tree.path} className="worktree-entry">
            <button
              className={`worktree-row ${tree.path === snapshot.repo.path ? "current" : ""}`}
              disabled={!!busy}
              onClick={() =>
                tree.path !== snapshot.repo.path && openWorktree(tree.path)
              }
              title={tree.path}
            >
              <FolderOpen size={15} />
              <span>
                {tree.branch}
                <small>
                  {tree.path.split(/[\\/]/).pop()}
                  {tree.locked ? " · locked" : ""}
                </small>
              </span>
            </button>
            <button
              className="icon"
              aria-label={`Manage worktree: ${tree.branch}`}
              title="Manage worktree"
              disabled={!!busy || unrelatedBlocked}
              onClick={() => manageWorktree(tree)}
            >
              <DotsThree size={18} />
            </button>
          </div>
        ))}
      </Group>
      <Group
        label="Stashes"
        count={snapshot.stashes.length}
        icon={<Stack size={14} />}
        initial={snapshot.stashes.length > 0}
        action={
          <button
            className="icon"
            title="Stash tracked and untracked changes"
            aria-label="Create stash"
            disabled={!!busy || unrelatedBlocked || !snapshot.files.length}
            onClick={() =>
              setForm({
                kind: "stash",
                title: "Save changes in a stash",
                value: "Ramo work in progress",
              })
            }
          >
            <Plus size={14} />
          </button>
        }
      >
        {snapshot.stashes.map((stash) => (
          <button
            className="stash-row"
            key={stash.oid}
            title={`Inspect ${stash.reference}: ${stash.message}`}
            disabled={!!busy}
            onClick={() => inspectStash(stash)}
            onContextMenu={(e) => {
              e.preventDefault();
              inspectStash(stash);
            }}
          >
            {stash.reference} {stash.message}
          </button>
        ))}
        {!snapshot.stashes.length && (
          <p className="sidebar-empty">No saved stashes</p>
        )}
      </Group>
      <div className="sidebar-bottom">
        <span className="sidebar-hint">
          Click to inspect · Right-click for actions
        </span>
        <span>
          {snapshot.repo.trusted
            ? "Repository trusted"
            : "Read access · writes need trust"}
        </span>
      </div>
    </aside>
  );
}

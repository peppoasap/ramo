import {
  useMemo,
  useCallback,
  useRef,
  useState,
  useLayoutEffect,
  type MouseEvent,
  type KeyboardEvent,
} from "react";
import {
  HISTORY_ROW_HEIGHT,
  historyIndices,
  historyScrollTarget,
  historyNavigationIndex,
} from "../historyWindow";
import { useHistoryWindow } from "../useHistoryWindow";
import { GitCommit, Clock, MagnifyingGlass } from "@phosphor-icons/react";
import { useGraphDrag, type GraphGesture } from "../useGraphDrag";
import { compactRefs, type GraphSource } from "../graphInteractions";
import { ContextMenu, type MenuState } from "./ContextMenu";
import { layoutGraph } from "../graph";
import type { Commit, Branch } from "../domain";
const colors = [
  "#7bd8ac",
  "#73b7ec",
  "#e9bd77",
  "#c29ee8",
  "#eb939e",
  "#8ac9c2",
];
export function History({
  branches,
  gesture,
  dragContext = "",
  onGraphDrop,
  onBranchOptions,
  onBranchCheckout,
  currentBranch,
  onBranchContext,
  onCommitContext,
  commits,
  selected,
  onSelect,
  filter,
  onFilter,
  onMore,
  hasMore,
  busy,
}: {
  branches: Branch[];
  gesture?: GraphGesture;
  dragContext?: string;
  onBranchOptions?: (branch: Branch, anchor: { x: number; y: number }) => void;
  onBranchCheckout?: (branch: Branch) => void;
  onGraphDrop?: (
    source: GraphSource,
    target: Commit | null,
    context: string,
  ) => void;
  currentBranch: string;
  onBranchContext: (event: MouseEvent | KeyboardEvent, branch: Branch) => void;
  onCommitContext: (event: MouseEvent | KeyboardEvent, commit: Commit) => void;
  commits: Commit[];
  selected: string | null;
  onSelect: (c: Commit) => void;
  filter: string;
  onFilter: (s: string) => void;
  onMore: () => void;
  hasMore: boolean;
  busy: boolean;
}) {
  const [refsMenu, setRefsMenu] = useState<MenuState | null>(null);
  const closeRefs = useCallback(() => setRefsMenu(null), []);
  const localGesture = useGraphDrag(
    dragContext,
    !gesture && !!onGraphDrop && !busy,
    (source, target, context) => {
      const commit =
        target === "current"
          ? null
          : commits.find((commit) => commit.oid === target);
      if (commit !== undefined) onGraphDrop?.(source, commit, context);
    },
  );
  const { drag, dropTarget, endDrag, sourceEvents, suppressClick } =
    gesture ?? localGesture;
  const rows = useMemo(() => layoutGraph(commits), [commits]);
  const graphWidth = useMemo(
    () =>
      Math.max(
        72,
        rows.reduce((width, row) => Math.max(width, row.lanes), 1) * 20 + 28,
      ),
    [rows],
  );
  // Keep all graph rows when filtering, otherwise edges would imply false direct parenthood.
  const query = filter.trim().toLowerCase();
  const { scroll, window, measure } = useHistoryWindow(commits.length);
  const heading = useRef<HTMLDivElement>(null);
  const [focusedOid, setFocusedOid] = useState<string | null>(null);
  const [pendingFocus, setPendingFocus] = useState<number | null>(null);
  const positions = useMemo(
    () => new Map(commits.map((commit, index) => [commit.oid, index])),
    [commits],
  );
  const selectedIndex = positions.get(selected ?? "") ?? -1;
  const focused = positions.get(focusedOid ?? "") ?? null;
  const branchesByCommit = useMemo(() => {
    const map = new Map<string, Branch[]>();
    for (const branch of branches)
      map.set(branch.oid, [...(map.get(branch.oid) ?? []), branch]);
    return map;
  }, [branches]);
  const matches = useMemo(
    () =>
      query
        ? commits.flatMap((commit, index) =>
            `${commit.subject} ${commit.author} ${commit.oid} ${commit.refs}`
              .toLowerCase()
              .includes(query)
              ? [index]
              : [],
          )
        : [],
    [commits, query],
  );
  const matching = useMemo(() => new Set(matches), [matches]);
  const indices = historyIndices(window, commits.length, [
    focused,
    pendingFocus,
  ]);
  const minWidth = 180 + graphWidth + 220 + 80 + 16;
  function reveal(index: number) {
    const host = scroll.current;
    if (!host || index < 0) return;
    host.scrollTop = historyScrollTarget(
      index,
      host.scrollTop,
      host.clientHeight,
    );
    measure();
  }
  useLayoutEffect(() => {
    reveal(selectedIndex);
  }, [selectedIndex, selected]);
  useLayoutEffect(() => {
    if (pendingFocus === null) return;
    const button = scroll.current?.querySelector<HTMLButtonElement>(
      `[data-history-index="${pendingFocus}"] .commit-select`,
    );
    if (button) {
      button.focus({ preventScroll: true });
      setPendingFocus(null);
    }
  }, [pendingFocus, window.start, window.end]);
  function select(index: number, focus = true) {
    if (index < 0 || index >= commits.length) return;
    if (focus) setPendingFocus(index);
    reveal(index);
    onSelect(commits[index]);
  }
  function navigate(event: KeyboardEvent<HTMLDivElement>) {
    const host = scroll.current;
    if (!host || !commits.length) return;
    const target = event.target as HTMLElement;
    if (target !== host && !target.closest(".commit-select")) return;
    const current = focused ?? Math.max(0, selectedIndex);
    const next = historyNavigationIndex(
      event.key,
      current,
      commits.length,
      host.clientHeight,
    );
    if (next === null || event.altKey || event.metaKey || event.ctrlKey) return;
    event.preventDefault();
    select(Math.max(0, Math.min(commits.length - 1, next)));
  }
  function nextMatch(backward: boolean) {
    if (!matches.length) return;
    const index = backward
      ? [...matches].reverse().find((i) => i < selectedIndex)
      : matches.find((i) => i > selectedIndex);
    select(
      index ?? (backward ? matches[matches.length - 1] : matches[0]),
      false,
    );
  }

  return (
    <section
      className="history"
      aria-label="Commit history"
      onClickCapture={(event) => {
        if (suppressClick.current) {
          event.preventDefault();
          event.stopPropagation();
          suppressClick.current = false;
        }
      }}
    >
      <div className="history-tools">
        <span>
          COMMIT HISTORY <b>{commits.length}</b>
        </span>
        <label className="search">
          <MagnifyingGlass size={16} />
          <input
            aria-label="Find a commit"
            value={filter}
            onChange={(e) => onFilter(e.target.value)}
            placeholder="Find commit, author, hash…"
          />
        </label>
      </div>
      {drag && (
        <div className="graph-drag-tools" role="status">
          <span>Drop on a commit to compare</span>
          <button
            className={dropTarget === "current" ? "drop-active" : ""}
            data-graph-target="current"
          >
            {drag.source.kind === "branch" ? "Merge into" : "Cherry-pick onto"}{" "}
            {currentBranch}…
          </button>
          <button onClick={endDrag}>Cancel drag</button>
        </div>
      )}
      {query && (
        <div className="history-search-navigation">
          <span role="status">{matches.length} matches in loaded commits</span>
          <button disabled={!matches.length} onClick={() => nextMatch(true)}>
            Previous match
          </button>
          <button disabled={!matches.length} onClick={() => nextMatch(false)}>
            Next match
          </button>
        </div>
      )}
      <div className="history-heading-viewport" ref={heading}>
        <div
          className="history-head"
          style={{
            gridTemplateColumns: `180px ${graphWidth}px minmax(220px,1fr) 80px`,
            minWidth: minWidth + 12,
          }}
        >
          <span>BRANCH / TAG</span>
          <span>GRAPH</span>
          <span>COMMIT</span>
          <span>DATE</span>
        </div>
      </div>
      <div
        className="commit-scroll"
        ref={scroll}
        tabIndex={0}
        role="region"
        aria-label="Scrollable commit history"
        onKeyDown={navigate}
        onScroll={() => {
          if (heading.current && scroll.current)
            heading.current.scrollLeft = scroll.current.scrollLeft;
          measure();
        }}
      >
        {commits.length === 0 ? (
          <div className="empty-inline">
            <GitCommit size={30} />
            <h3>No commits yet</h3>
            <p>Your first local commit will appear here.</p>
          </div>
        ) : (
          <div
            className="history-canvas"
            role="list"
            aria-label="Loaded commits"
            style={{ height: window.totalHeight, minWidth }}
          >
            {indices.map((index) => {
              const commit = commits[index];
              const row = rows[index];
              const refs = compactRefs(
                branchesByCommit.get(commit.oid) ?? [],
                currentBranch,
              );
              const tags = commit.refs
                .split(", ")
                .filter((r) => r.startsWith("tag: "));
              const match = !query || matching.has(index);
              return (
                <div
                  id={`commit-${commit.oid}`}
                  key={commit.oid}
                  role="listitem"
                  aria-posinset={index + 1}
                  aria-setsize={commits.length}
                  data-history-index={index}
                  onFocusCapture={() => setFocusedOid(commit.oid)}
                  onBlurCapture={(event) => {
                    if (
                      event.relatedTarget instanceof Element &&
                      event.relatedTarget.closest(".context-menu")
                    )
                      return;
                    if (
                      !event.currentTarget.contains(
                        event.relatedTarget instanceof Node
                          ? event.relatedTarget
                          : null,
                      )
                    )
                      setFocusedOid((previous) =>
                        previous === commit.oid ? null : previous,
                      );
                  }}
                  data-graph-target={commit.oid}
                  onContextMenu={(e) => onCommitContext(e, commit)}
                  onKeyDown={(e) => {
                    if (
                      e.key === "ContextMenu" ||
                      (e.shiftKey && e.key === "F10")
                    )
                      onCommitContext(e, commit);
                  }}
                  className={`commit-row ${dropTarget === commit.oid ? "drop-active" : ""} ${selected === commit.oid ? "selected" : ""} ${!match ? "dimmed" : ""}`}
                  style={{
                    gridTemplateColumns: `180px ${graphWidth}px minmax(220px,1fr) 80px`,
                    position: "absolute",
                    top: index * HISTORY_ROW_HEIGHT,
                  }}
                  onClick={() => select(index)}
                  title={`${commit.subject}\n${commit.oid}\n${commit.author}\nDrag to compare or cherry-pick · Right-click for actions`}
                >
                  <div className="graph-refs">
                    {refs.slice(0, 1).map((branch) => (
                      <button
                        {...sourceEvents({ kind: "branch", branch })}
                        data-graph-target={
                          !branch.remote && branch.name === currentBranch
                            ? "current"
                            : undefined
                        }
                        className={`graph-ref ${dropTarget === "current" && !branch.remote && branch.name === currentBranch ? "drop-active" : ""} ${branch.remote ? "remote" : "local"} ${!branch.remote && branch.name === currentBranch ? "current" : ""}`}
                        key={`${branch.remote}-${branch.name}`}
                        title={`${branch.remote ? "Remote" : "Local"} branch: ${branch.name} · Double-click to ${branch.remote ? "create local tracking branch" : "checkout"} · Right-click for actions`}
                        aria-label={`Branch: ${branch.name}`}
                        style={{
                          borderColor: colors[row.color % colors.length],
                        }}
                        onClick={(e) => {
                          e.stopPropagation();
                          onSelect(commit);
                        }}
                        onDoubleClick={(e) => {
                          e.stopPropagation();
                          onBranchCheckout?.(branch);
                        }}
                        onContextMenu={(e) => onBranchContext(e, branch)}
                        onKeyDown={(e) => {
                          if (
                            e.key === "ContextMenu" ||
                            (e.shiftKey && e.key === "F10")
                          )
                            onBranchContext(e, branch);
                        }}
                      >
                        {!branch.remote && branch.name === currentBranch
                          ? "✓ "
                          : branch.remote
                            ? "↗ "
                            : "⑂ "}
                        {branch.name}
                      </button>
                    ))}
                    {!refs.length && tags.length > 0 && (
                      <span className="graph-tag" title={tags.join(" · ")}>
                        {tags[0].slice(5)}
                      </span>
                    )}
                    {refs.length + tags.length > 1 && (
                      <button
                        className="graph-ref-more"
                        aria-label={`Show all references: ${refs.length} branches, ${tags.length} tags`}
                        onClick={(event) => {
                          event.stopPropagation();
                          const box =
                            event.currentTarget.getBoundingClientRect();
                          setRefsMenu({
                            x: box.left,
                            y: box.bottom,
                            title: "References at this commit",
                            items: [
                              ...refs.map((branch) => ({
                                label: `${!branch.remote && branch.name === currentBranch ? "Current · " : branch.remote ? "Remote · " : "Local · "}${branch.name}`,
                                run: () => {
                                  // Open the existing action menu at the stable references trigger.
                                  onBranchOptions?.(branch, {
                                    x: box.left,
                                    y: box.bottom,
                                  });
                                },
                              })),
                              ...tags.map((tag) => ({
                                label: tag,
                                run: () => select(index),
                              })),
                            ],
                          });
                        }}
                      >
                        +{refs.length + tags.length - 1}
                      </button>
                    )}
                  </div>
                  <svg
                    className="graph"
                    width={graphWidth}
                    height={48}
                    aria-hidden="true"
                  >
                    {row.edges.map((edge, i) => (
                      <path
                        key={i}
                        d={`M ${20 + edge.from * 20} ${edge.fromNode ? 24 : 0} C ${20 + edge.from * 20} 40, ${20 + edge.to * 20} 32, ${20 + edge.to * 20} 48`}
                        fill="none"
                        stroke={colors[edge.color % colors.length]}
                        strokeWidth={2}
                      />
                    ))}
                    {row.incoming && (
                      <path
                        d={`M ${20 + row.lane * 20} 0 V 24`}
                        stroke={colors[row.color % colors.length]}
                        strokeWidth={2}
                      />
                    )}
                    <circle
                      cx={20 + row.lane * 20}
                      cy={24}
                      r={selected === commit.oid ? 5 : 4}
                      fill={colors[row.color % colors.length]}
                    />
                  </svg>
                  <button
                    className="commit-text commit-select"
                    {...sourceEvents({ kind: "commit", commit })}
                    tabIndex={
                      selectedIndex === index ||
                      (selectedIndex < 0 && index === 0)
                        ? 0
                        : -1
                    }
                  >
                    <span className="subject">{commit.subject}</span>
                    <span className="commit-meta">
                      <code>{commit.oid.slice(0, 7)}</code>
                      <span>{commit.author}</span>
                    </span>
                  </button>
                  <time dateTime={commit.date} title={commit.date}>
                    {new Date(commit.date).toLocaleDateString(undefined, {
                      month: "short",
                      day: "numeric",
                    })}
                  </time>
                </div>
              );
            })}
          </div>
        )}
        {hasMore && (
          <button className="load-more" disabled={busy} onClick={onMore}>
            <Clock size={16} />
            Load older commits
          </button>
        )}
      </div>
      {refsMenu && <ContextMenu menu={refsMenu} close={closeRefs} />}
    </section>
  );
}

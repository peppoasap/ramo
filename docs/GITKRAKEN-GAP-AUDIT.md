# Ramo daily-client gap audit

Reviewed 2026-11-05 against the current source and recorded 0.1.0 preview evidence.
This is a status audit and proposed completion contract, not an implementation or
GitKraken parity claim. No runtime code or frozen preview archive changes here.

## Current position

Ramo is a functional private preview for macOS Apple Silicon. Existing Git actions
are real; 56 frontend and 114 backend checks passed on 2026-11-04. Native
acceptance covers synthetic local workflows. Hosted credentials and other-platform
runtime checks remain unverified. The app is ad-hoc signed; Gatekeeper assessment
rejects it, and a downloaded first launch on a second Mac has not been accepted.

The GitKraken comparison uses official Desktop documentation checked on
2026-11-05; it is not a version-pinned exhaustive inventory. The locally inspected
GitKraken build in the prior report was 12.6.1. No numerical completion percentage
is assigned: a small missing recovery action can interrupt an otherwise complete
workflow. Feature presence, UI reachability and acceptance are different gates.

Latest development update, same date: sidebar name filtering/collapsible groups,
inspection-only branch clicks, real branch switcher, secondary Compare action and
shared sidebar/graph dragging are now implemented and native-checked. The frozen
0.1.0 tester ZIP still contains the previous UI. See `UI-CONVENTIONS.md` and
`verification/ux-order.json`. This does not complete graph Hide/Solo or auto-scroll.

## Coverage

| Area | Present in Ramo | Gap to a complete daily workflow |
| --- | --- | --- |
| Repositories | Native open, recents, tabs, local/HTTPS/SSH clone | Initialize a new repo from UI; clone progress/cancel; hosted credential acceptance; additional account catalogs |
| Graph | Parent-derived topology, local/remote tips, tags, compact +N references, pagination and virtual rows | Hide/solo/filter branches; search beyond loaded pages; resize/persist layout; more extreme-topology checks |
| Graph gestures | Branch/commit to commit compares pinned SHAs; sidebar branch sources share the graph gesture session; current branch targets in graph/sidebar review merge/cherry-pick | Arbitrary branch destinations with explicit review, auto-scroll, multi-selection/ranges and rebase gestures |
| Branches | Create/switch/rename; create at commit; delete only when merged; local tracking checkout | Upstream editing, remote-branch actions/deletion, published/ahead-behind clarity for all branch entries |
| Commits | Commit the index, persistent message draft; reviewed local non-merge HEAD amend/reword with recovery ref; detached checkout; non-merge cherry-pick/revert | Amend merge/published commits, multi-commit operations, templates/identity/signing UI |
| History rewriting | No built-in rebase/reset | Guided rebase and continue/abort, interactive reorder/squash/fixup/drop, soft/mixed/hard reset with recovery |
| Recovery | Commit and merge-result drafts, sequence abort/continue/skip, stale review refusal | Reflog browser, recovery refs, operation journal, operation-specific Undo/Redo; no global Git undo currently |
| Changes | Whole-file and ordinary modified-text hunk stage/unstage; separated conflicts | Individual-line staging, partial new/deleted/renamed/mode changes, reviewed discard with recovery, ignore/unignore actions |
| Diffs/files | Unified text diff; commit changed files; direct/common-ancestor comparisons | Side-by-side diff, individual file history, blame, file restore from a commit; editor/Finder integration |
| Merge | Start, fast-forward or two-parent commit, complete/abort; integrated full-space CodeMirror editor | Merge strategy controls/squash mode; rename/delete scenarios; better repeated-range alignment and synchronized scrolling |
| Conflict editor | Ours/theirs/base/result, per-block choices, individual source-line selection, manual edits, search/undo, reviewed save+stage, local draft recovery | Binary/symlink/submodule workflows or a verified external merge-tool handoff; large-file/edge-case coverage |
| Sequences | Cherry-pick/revert conflict resolution and continue/skip/abort, including native multi-step recovery | Start from selected ranges, merge-commit mainline choices; rebase UI is absent |
| Stash | Save including untracked, inspect, apply/pop/drop with review, normal text conflict recovery, selected missing saved regular-file recovery | Restore saved index state, branch from stash, dropped-stash recovery, wider saved-file types/sizes |
| Worktrees | Create new/existing-branch checkout, open, lock/unlock, guarded inactive checkout removal | Move/prune, detached checkout removal, richer tree/head/status presentation |
| Remotes | Fetch, ff-only pull, active-branch push and publish | Remove/rename remotes, edit special multi-URL/push configurations, change upstream, choose merge/rebase pull, controlled force-with-lease, prune and better auth/error/progress UX |
| Tags | Lightweight local creation at a commit; graph decoration | Dedicated tag browser/menu, annotated tags, tag push/delete, signature controls |
| Desktop | Native picker, tabs/recents, graph keyboard navigation, context menu keyboard support, open/refresh shortcuts | Full app command surface, command palette, keyboard reference, terminal/editor integration, file watcher, layout persistence |
| AI | Reviewed bounded commit/diff/conflict assistance, provider configuration, explicit Stop, no autonomous Git writes | Plug-and-play onboarding acceptance across providers; live Claude/Ollama/merge proposal acceptance, version compatibility, shutdown/crash cancellation |
| Distribution | Frozen private macOS arm64 DMG/ZIP, sources, notices, checksums and tester report | Developer ID/notarization, second-Mac acceptance, verified signing/update/release workflow |
| Platforms | macOS Apple Silicon native evidence | Intel Mac/Windows/Linux build-install-runtime acceptance; Windows/Linux deferred until runners are available |
| Hosted collaboration | Existing Git transport via configured Git helpers/SSH | In-app PRs, reviews, issue integrations, CI status and browser/provider links |
| Git extensions | Existing configured Git may participate in ordinary commands; no dedicated UI contract | Explicit LFS/submodule workflows, Gitflow and broader enterprise/provider support |

Important boundaries: merge editor Undo/Redo edits text only. Single source-line
selection for resolving conflicts is implemented; single-line staging in Changes
is not. Revert creates a reversing commit; it does not provide general undo.
Fast-forward-only pull cannot resolve a diverged branch. A generic Git credential
helper working on one machine does not establish plug-and-play hosted login.

## Exact current context-menu inventory

Source of truth: `src/repositoryMenus.ts`, `src/components/Sidebar.tsx`,
`src/components/Inspector.tsx`, `src/components/StashDialog.tsx`,
`src/components/WorktreeManager.tsx`, `src/graphInteractions.ts`.

| Object | Reachable today | Proposed additions for the first complete base |
| --- | --- | --- |
| Local branch | Show in graph, copy name, compare, create worktree, checkout, merge into current branch, rename, delete merged branch | Rebase onto target, upstream controls, publish/push/pull context, hide/solo, clear checked-out worktree status |
| Remote branch | Show in graph, copy name, compare, checkout as local tracking branch | Set/change upstream, merge/rebase explicit target, reviewed remote deletion, remote/provider links |
| Commit | Inspect, compare, copy SHA/subject, create branch/tag, detached checkout, cherry-pick, revert | Amend/reword when eligible, reset current branch here, range selection/squash, recovery, clearer published-history restrictions |
| Changed file | Select and stage/unstage controls; conflict resolver; hunk actions in diff | Consistent file context menu: stage/unstage file/selected lines, discard/recover, ignore, history/blame, open/reveal/copy path |
| Tag | Decoration or reference-list entry; creation through commit menu | Dedicated inspect/checkout/create annotated/push/delete/copy menu |
| Stash | Dedicated inspection dialog with reviewed Apply/Pop/Drop and selected-file recovery | Matching context menu, restore index, create branch, recovery path |
| Worktree | Open button and dedicated management dialog with lock/unlock/remove | Matching context menu, reveal/editor/terminal, move/prune/status |
| Repository | Open/clone, tabs/close and refresh | Initialize, reveal/editor/terminal, remote settings, copy path and operation log |

Stash/worktree operations already exist in dialogs. Missing right-click access to
them must not be counted as absence of the underlying Git operations. A new menu
entry is complete only when backed by a real guarded action, appropriate disabled
state/reason, review when necessary, operation/error state and acceptance evidence.

## Proposed definition: first complete macOS base

This is the proposed daily-use scope; it is not yet delivered. Keep hosted team
features as a separate milestone. Windows/Linux acceptance remains a separate gate
when the user has runners. Do not require deferred platforms to declare the scoped
macOS base complete, and do not describe that as cross-platform parity.

1. Everyday local lifecycle: open/initialize/clone, inspect/filter/compare, stage
   file/hunk/line, commit/amend, branch operations, merge/rebase and conflict
   recovery, cherry-pick/revert, tags/stash/worktrees, fetch/pull/push/upstream.
2. Correct a mistake in the UI: recover discarded work, undo a supported local
   operation, inspect reflog, amend/reword, reset with explicit mode and review.
   Publish a supported-undo list; no universal Undo button with unverified promises.
3. Consistent interaction: object-specific menus in sidebar/graph/inspector, same
   commands through keyboard/palette, explicit source/target drag review, visible
   reasons when actions are unavailable. Preserve a compact default interface.
4. Practical remote use: real HTTPS/SSH acceptance on a hosted disposable fixture,
   credential helper/keychain/agent handoff, understandable authentication and
   divergence failures, progress and refresh/cancel where safe.
5. End-to-end resilience: dirty/index/untracked work preserved, error/stale state
   and interrupted sequence recovery, saved drafts retained, external changes
   reflected, restart tests and usable diagnostics with secrets redacted.
6. Install on another supported Mac from an actual downloaded artifact, verify
   signing/notarization and the full basic scenario, record evidence. Expand
   unsupported platforms only after equivalent native acceptance.

Not all uncommon Git cases need a custom editor, but every declared unsupported
case needs a useful, tested external-tool route. The ordinary text workflow must
not depend on silently dropping back to a terminal.

## Proposed implementation order

| Milestone | Scope | Completion evidence |
| --- | --- | --- |
| 1. Daily fixes and file actions | Amend/reword last eligible commit, reviewed discard/recovery, line staging; file menus/history/blame and editor/reveal handoff | Real Git fixture preserves unstaged/untracked data and drafts on success/failure; all actions reachable in native UI |
| 2. History and recovery | Reflog/recovery refs, explicit reset modes, basic guided rebase then interactive rebase/ranges, operation-specific undo | Diverged/published/dirty states, conflict/continue/abort, restart and interrupted-operation recovery verified |
| 3. Graph and menu completeness | Hide/solo/search scope, explicit arbitrary targets, auto-scroll/ranges, unified stash/tag/worktree/repository commands and palette | Actual pointer and keyboard scenarios; same semantic operation regardless of invocation; large graph remains bounded |
| 4. Remotes and Git metadata | Remote/upstream management, hosted auth acceptance, pull strategy, force-with-lease, full tags/stash index and worktree polish | Disposable hosted repository end-to-end checks; no accidental push of other refs; explicit destructive review |
| 5. Operational acceptance | File watching/progress/logs, provider onboarding acceptance, signing/notarization/second-Mac check | Reproducible installer and complete daily scenario on a second machine; no unverified platform claims |

These are functional work packages, not day estimates. Split each into small
reviewable slices; begin with reviewed amend rather than implementing many menus
with incomplete actions. AI changes must not displace unfinished core Git flows.

## Broader GitKraken-like scope after the base

PR creation/review, provider issues/CI/workspaces, advanced LFS/submodules,
Gitflow, richer signed commits/tags, expanded remote/enterprise integrations,
customization/localization and every-platform distribution. File-backed Git
signing/configuration may already influence ordinary Git commands; that is not a
completed configuration/verification UX. Similarity should mean familiar command
placement and complete workflows, not copied branding or a crowded toolbar.

## Sources and evidence

Ramo: current source paths above; `src-tauri/src/git.rs` typed Action/mutate
contract; `docs/FEATURES.md`; `docs/verification/preview-validation.json` and
`preview-archive.json`; `docs/AI.md`; graph/merge/sequence evidence referenced by
STATUS.md. The last test run is dated 2026-11-04; this audit does not rerun tests or
claim fresh runtime verification.

Official GitKraken Desktop references:

- [Interface](https://help.gitkraken.com/gitkraken-desktop/interface/)
- [Branch, merge and rebase](https://help.gitkraken.com/gitkraken-desktop/branching-and-merging/)
- [Committing and amend](https://help.gitkraken.com/gitkraken-desktop/commits/)
- [Interactive rebase](https://help.gitkraken.com/gitkraken-desktop/interactive-rebase/)
- [Undo and redo](https://help.gitkraken.com/gitkraken-desktop/undo-and-redo/)
- [File diffs, history and blame](https://help.gitkraken.com/gitkraken-desktop/diff/)
- [Staging](https://help.gitkraken.com/gitkraken-desktop/staging/)
- [Push, pull and fetch](https://help.gitkraken.com/gitkraken-desktop/pushing-and-pulling/)
- [Tags](https://help.gitkraken.com/gitkraken-desktop/tags/)
- [Command palette](https://help.gitkraken.com/gitkraken-desktop/command-palette/)

Hosting development update: provider-guided clone and explicit gh-based GitHub catalog
are implemented; live account acceptance remains unverified. Basic remote add/URL
change now exists with review and endpoint freshness. Other provider catalogs, OAuth,
removal/upstream and clone progress/cancel remain gaps. See HOSTING.md.

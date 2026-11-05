# Daily Git client coverage

This is a delivery checklist, not a claim of GitKraken feature parity. Each implemented action uses real Git and reviewed repository context. Native evidence currently covers macOS Apple Silicon only.

| Workflow | Implemented | Remaining |
| --- | --- | --- |
| History | Parent-derived lanes, local/remote branch tip column, tag decorations, pagination, bounded viewport rows, keyboard navigation, loaded-history search/reveal, first-parent file diffs; direct and common-ancestor revision comparisons | Graph branch filtering/Hide/Solo, native frame-rate/heap profiling and extreme topology validation |
| Context menus | Branch graph/sidebar menus; commit menus; pointer and keyboard invocation | Tag/stash/worktree menus and richer shortcuts |
| Branches | Checkout/create/rename/protected deletion/merge; create at selected commit; local tracking checkout | Upstream editing, remote management/deletion |
| Commits | Staged commit, persisted drafts, detached checkout, single-parent cherry-pick/revert with continue/skip/abort; multi-step cherry-pick recovery native-verified | Amend, signing controls, reset/reflog recovery, interactive rebase |
| Merge | Start/complete/abort; full-space CodeMirror source/result editors; per-block choices; individual source-line selection; active-range highlights and linked conflict navigation; base view; manual edits; reviewed save-and-stage; deletion resolution | Binary, symlink and submodule resolution; rename-specific workflows; ambiguous/repeated source range alignment and pixel-linked scrolling |
| AI merge assistance | Explicit preview of one file's bounded versions; explain or structured resolution proposal; explicit insertion into editor; shared Stop with draft preservation and late-response suppression; tools disabled | Live merge-generation verification (not authorized during these checks); richer block-specific suggestions |
| Changes | Whole-file stage/unstage; ordinary modified-text hunk stage/unstage; conflict separation; text/binary diffs | Individual line staging, partial new/deleted/renamed or mode changes, discard with recovery, LFS UX |
| Stashes | Save including untracked; inspect tracked/untracked diffs; reviewed apply/keep, pop on success and drop; ordinary conflict resolution/commit with retained stash; missing regular saved untracked-file recovery | Saved-index restoration, filtered/large/symlink saved-file recovery, branch-from-stash, recovery for dropped entries |
| Worktrees | Create with new/existing local branch via native destination picker; inspect/open registered worktrees; separate indices; reviewed lock/unlock and guarded inactive-checkout removal | Prune/move, detached checkout removal, atomic coordination with external processes |
| Remotes | Reviewed HTTPS/SSH/local-source clone with native new destination; fetch, explicit upstream ff-only pull, active-branch push/publish | Hosted clone/login UX verification, clone cancellation/progress, remote setup |
| Tags | Local lightweight tag creation at a selected commit; display decorations | Tag browsing, annotated/signing/tag push/delete |
| Desktop | Tabs, recents, native picker, macOS local installer | Terminal/editor integration, localization, platform/install verification |

## Merge contract

Source gutter controls select individual rows. Use selected lines replaces only the active conflict; rows keep their order within each source, current before incoming. With no remaining conflicts, Insert selected lines inserts at the result cursor. Normal editor selection remains available. The active range is highlighted, and Previous/Next follows result and optionally source ranges. Locate explicitly recenters available ranges; manual source scrolling remains independent. Repeated, empty or unmatched source fragments show location unavailable instead of a guessed location. The editor does not save on typing, selecting rows or AI insertion. Save and mark resolved requires confirmation, session trust and the original reviewed fingerprint. Git remains authoritative: conflicts are staged only after the result is written. A failed save retains the editor text. Merge completion is a separate commit; push remains separate. Closing an edited buffer requests discard confirmation. Edited resolution buffers have local recovery, scoped by worktree/file and reviewed fingerprint. Matching drafts restore into the editor; changed-state copies require explicit review. Known sensitive filenames disable automatic persistence. Storage failures preserve in-memory text and warn. Normal restart acceptance does not prove survival of every OS/disk failure; detected competing records are preserved, without claiming an atomic multi-window transaction.

Text resolution is limited to 1 MiB per version. The editor refuses binary, symlink and submodule conflicts and offers a clear external-resolution fallback. AI input is limited to 96 KiB, excludes sensitive paths and contains only the selected file's on-disk base/ours/theirs/working versions. It does not include unsaved editor text. AI proposals are strict JSON, never executable commands, and never authorize filesystem or Git writes. If the result changed after the AI request, automatic insertion is disabled.

## Current audit and proposed next slice

The 2026-11-05 [daily-client gap audit](GITKRAKEN-GAP-AUDIT.md) checks the current
source, actual menu inventory and preview evidence. It distinguishes missing
features from dialog-only access and unverified runtime behavior. Its completion
contract and milestones are proposals, not delivered functionality.

The current private0.1.0preview has56frontend/114backend checks recorded on
2026-11-04. Worktree lock/removal, stash recovery, AI Stop, virtual history and
reviewed graph gestures are implemented. For core Git completion, the proposed
next implementation is reviewed last-commit amend, followed by file actions/line
staging and history/recovery workflows. Second-Mac first-launch acceptance and
Developer ID/notarization remain release gates. Windows/Linux remain deferred
until runners are available. Keep the frozen preview and its manifests unchanged.

Graph usability (local/native verified, 2026-11-04): prioritized reference plus +N overflow without inner row scroll; pointer drag branch/commit → commit compares pinned SHAs; branch/commit → current branch reviews merge/cherry-pick. No immediate mutation on drop. Sidebar dragging is now implemented in the local development UI; multi-select drag, arbitrary branch destinations and auto-scroll remain gaps. Evidence: [graph interactions](verification/graph-interactions.json).

Workbench UX pass (2026-11-05, local development build): sidebar branch-name filter,
collapsible sections, one ellipsis action entry per branch, read-only branch clicks,
explicit checkout selector/review, separate secondary Compare action, accurate
Publish label and shared sidebar/graph drag session. Native synthetic acceptance
checks filtering/Escape, collapse/reopen, branch inspection, checkout/merge/cherry-pick
review and cancellation, pinned comparisons and unchanged Git state. No frozen
preview ZIP update. See [UI conventions](UI-CONVENTIONS.md) and
[UX evidence](verification/ux-order/README.md).

## 0.2.0-beta.1 source candidate

Changes → Amend last commit opens the full original message for editing and
includes only the current index. Message-only rewriting is supported. Native
validation refuses stale/untrusted/detached/merge/known-remote HEAD and active
operations/conflicts. Original commits remain under local recovery refs, excluded
from the ordinary history view. Recovery currently uses Git externally. The normal
persistent commit draft is unaffected; amend dialog text is retained on failure
only while the dialog remains open. Public beta release is not yet accepted;
see `BETA-CHECKLIST.md` and `RELEASING.md`.

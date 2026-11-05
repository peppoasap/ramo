# Roadmap

The user requested functional, installable software rather than a simulated prototype. Deliver real vertical slices, keeping claims tied to evidence.

## Rust-native direction — 2026-11-05

Owner-requested direction: Rust application/UI with a more complete daily Git
workflow. Current beta.4 remains Tauri/React. Proposed order: read-only Rust graph,
editor/draft feasibility, existing workflow parity, line staging/recovery/history,
then guided rebase and remote depth. Preserve installed Git initially; assess a
Rust Git engine separately. See [migration and product plan](RUST-NATIVE-PLAN.md),
[ADR0025](decisions/0025-rust-native-direction.md) and the
[six-screen UI concept](design/rust-native/README.md). These are proposals, not
implemented features or new release evidence.

## Existing product coverage

| Slice | Current state |
| --- | --- |
| Selected graphite/mint design and Apache-2.0 | Accepted; original mockups preserved |
| Real desktop graph, status and diffs | Implemented; macOS native smoke verified |
| Staging, commits, branches, worktrees and stash | Implemented; synthetic Git integration and native commit verified |
| Clone/fetch/pull/push/publish | Implemented; native local bare clone and disposable local remote tests pass; hosted clone/credential UX unverified |
| Contextual AI | Codex 0.166.0: zero-tools contract and one live synthetic generation verified; native CLI selection implemented; Claude/Ollama live unverified |
| Installable macOS | Local Apple Silicon app/DMG; no Developer ID or notarization |
| Windows/Linux | Beta.4 installers published after passing hosted builds/tests; GUI acceptance pending |
| Public OSS source | Published at peppoasap/ramo; beta installers are gated on CI, stable release/platform gates remain |

Detailed daily-client coverage and gaps: [FEATURES.md](FEATURES.md).

## Next vertical slice

The graph branch column, branch/commit context menus and full-space merge editor are implemented. Native acceptance verifies an actual conflict-to-commit path, a context-menu tag and selecting one source line without changing another conflict. Active-range highlighting and linked conflict navigation are implemented, with native two-block large-file acceptance. Refine ambiguous/repeated source alignment and continuous scrolling separately. Local resolution recovery is implemented, with native normal-restart and changed-Git-state acceptance. Cherry-pick/revert continue/skip/abort are implemented and verified with actual Git fixtures; native cherry-pick continuation is also verified. Direct and common-ancestor commit/branch comparisons are implemented and native-verified. Keep every remaining capability visible in FEATURES.md.

Merge AI preview and structured proposal validation are locally verified; live merge inference was not executed. Codex's earlier authorized commit-generation evidence does not establish live merge proposal quality. Preserve manual results and staged input on every failure.

Hosted Windows/Linux builds now pass; beta.4 installers are public. See [build verification](BUILD-VERIFICATION.md). Record build results separately from install/open/stage/commit acceptance. Windows npm-installed CLI wrappers need explicit validation.

## Later daily-use improvements

Rebase continuation, individual line staging, worktree move/prune, keyboard polish, centralized localization strings, file watching, large-repository benchmarks and crash/recovery scenarios. Choose by observed daily-use needs. Avoid a general AI chat or plugin marketplace.

Publishing source, creating a public remote, and releasing binaries each need explicit authorization. Local artifacts and configured CI are not public releases or verified multi-platform support.

## 2026-11-05 scope audit

See [GITKRAKEN-GAP-AUDIT.md](GITKRAKEN-GAP-AUDIT.md) for the current menu inventory,
feature/evidence gaps and proposed first-complete macOS contract. Latest recorded
preview checks are56frontend/114backend, dated2026-11-04; the51-test history entry
above is historical evidence. Source-line conflict selection and guarded worktree
lock/removal are delivered; line staging, amend/reset/reflog/rebase, full remote
management, file history/blame and global Git Undo/Redo are not. Stash/worktree
actions already work through dialogs despite incomplete context-menu access.
The new milestone order is proposed for review, not an implementation claim.

Workbench refinement delivered locally, 2026-11-05: sidebar filtering/collapse,
read-only branch inspection, contextual ellipsis, actual switcher/review, separated
Compare action and shared sidebar/graph gestures. Existing semantic commands are
reused. Frozen tester ZIP remains unchanged; graph Hide/Solo, richer targets,
auto-scroll and complete file/tag/stash menus remain follow-up work.

## 0.2.0-beta.1 candidate update

Reviewed local non-merge HEAD amend/reword is implemented with a recovery ref,
index-only contents, original-author preservation and native failure/cancel checks.
Changelog, portable source staging and release metadata/CI artifact gates are in
place. Next vertical slice: individual-line staging, followed by reviewed discard
with recovery and file context menus/history. Public beta acceptance is separate;
see BETA-CHECKLIST.md.

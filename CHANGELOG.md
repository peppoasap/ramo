# Changelog

User-visible changes are recorded here. Versions describe source milestones;
only a published GitHub Release establishes an external release.

## Unreleased — Rust-native development (0.3.0-dev.1)

- Reorganize native Changes into files, selectable diff and right commit composer;
  keep original diff-row identities while hiding raw headers and highlighting rows.
- Add guarded conflict-file navigation and refine current/incoming/result merge
  surfaces. Keep local draft save distinct from reviewed save/stage and completion.

- Refine native history toward the selected graphite/mint mocks: bundled
  regular/semibold typography, underlined tabs, branch badges, integrated toolbar
  search and clearer read-only diff rows. Preserve compact density and menus.
- Pin synthetic clone checkout line endings through committed attributes, fixing
  a Windows runner-dependent LF assertion without changing production Git config.

- Clone from URL, provider shortcut or a native-picked local repository through
  destination grants and immutable one-shot review. Opening the new checkout
  preserves repository-scoped messages and starts with fresh write trust.
- Add an explicitly loaded GitHub catalog via installed `gh`, with organization
  scope/filter/pagination. Other providers remain URL shortcuts; no login/import.
- Add native remote inspection and reviewed ordinary URL add/edit, preserving
  complex/local configuration. Saving updates local settings without fetch/push.
- Keep clone/remote dialogs compact, suggest unused remote names and disable
  underlying workbench input while a dialog is open.

- Native graph gestures compare commits/branches on drop, or open reviewed
  merge/cherry-pick onto the named current branch. Sidebar branches also drag.
  No automatic mutation, checkout or push; existing trust/freshness guards apply.
- Capture drag origin before rapid mouse events can be coalesced. Keep graph
  rows fixed during dragging and cancel stale, busy, escaped or unfocused sessions.

- Compare saved revisions in a dedicated native view: choose two commits from
  branch/history menus or revision pickers, swap direction and inspect direct
  or unique-common-ancestor diffs without touching staged or working changes.
- Bind comparison/file replies to the active selection; changing an endpoint
  clears prior results and late replies cannot replace the current comparison.

- Recover repository-scoped commit messages through local autosave, with guarded
  opening/closing, competing-writer checks and explicit storage-error recovery.
  Successful normal commits clear their saved message; amend keeps it.
- Add native commit menus for copy, branch/tag creation, amend, detached checkout,
  cherry-pick and revert; add branch rename/tracking and supported sequence controls.
- Use focused action dialogs, distinct primary buttons and a stable graphite theme.
- Route macOS Quit through native close guards so immediate Cmd-Q waits for a
  pending composer save. Force termination and OS shutdown remain separate limits.

- Native diff view stages/unstages whole hunks or selected added/deleted rows,
  with Shift range selection and original/new file line numbers. Core derives
  the patch from the reviewed Git diff and rejects stale or unsupported selections.
- Partial staging preserves working bytes and unrelated index changes. Missing
  final-newline files retain whole-hunk/file staging; complex file kinds retain
  whole-file actions.

- Native daily Git actions now reuse existing trust and freshness guards: file
  staging/unstaging, index commits, branch checkout/create/merge/delete,
  fetch/pull/push and reviewed stash operations.
- Native merge results can be reviewed, saved and staged, then completed from
  Changes; unchanged-tree merge results can also be committed.
- Double-click local checkout, branch/stash contextual actions and separated
  working/index file lists. Mutation failures refresh state without retrying.

- Add a separate egui/wgpu application using the Rust core without Tauri/WebView.
- Browse actual repository history, branches/tags, commit files and diffs through
  a worker, with compact22px virtual rows, pagination, search and keyboard navigation.
- Evaluate a full-space native conflict editor with individual source-line picks,
  manual result editing, undo/redo, result search and explicit local draft recovery.
- Keep beta.4 as the released application during native workflow migration.
  No new native installer release; contextual AI still uses the beta UI.
- Add native preservation tests, scoped draft storage, dependency/font notices and
  a separate cross-platform CI workflow. Hosted and native GUI evidence are separate.

## [0.2.0-beta.4] - 2026-11-05

- Explicitly ad-hoc sign the macOS application bundle before creating its DMG.
- Mount the final CI DMG read-only and require strict signature, version and bundled
  notice checks before uploading any Mac installer. No Developer ID/notarization.
- Supersedes the beta.3 Mac artifact, whose linker-only signature failed strict
  bundle/resource verification after public download. Frozen assets are retained.

## [0.2.0-beta.3] - 2026-11-05

- Publish project sources, README with a short roadmap, contributor/security guidance
  and a real synthetic desktop screenshot at peppoasap/ramo.
- Build macOS/Windows/Linux installers in CI; publish version-matching beta tags as
  prereleases only after all platform checks pass.
- Pass native Windows drive/UNC paths to Git without verbatim device prefixes for
  clone and worktree commands. Compare reviewed native path representations without
  bypassing symlink/common-repository/open-checkout guards.
- Make Git fixtures independent of host autocrlf defaults and use valid Windows
  filenames while retaining Unicode/literal-path and CRLF coverage.
- Include supplemental dependency license texts and README images in standalone
  source exports; normalize Windows package-lock paths for notice generation.
- Preserve all earlier local beta installers and source snapshots unchanged.

## [0.2.0-beta.2] - Local distribution candidate, 2026-11-05

- Compact top bar with fixed-height Clone and grouped Git toolbar: Branch, Stash,
  reviewed Pop and additional remotes/worktree/comparison actions.
- Cleaner AI settings and operation dialogs; form input survives repository trust.
- Includes provider-guided clone, GitHub CLI catalog and reviewed remotes management.
- Updated Apple Silicon macOS package; Windows native installer still needs a Windows
  runner. Windows source build kit is not an executable installer.
- Apache-2.0; no public GitHub release, Developer ID signing or Apple notarization.

## [0.2.0-beta.1] - Unreleased

### Added
- Provider-guided clone for GitHub/Enterprise, GitLab, Bitbucket, Azure and self-hosted
  servers, with HTTPS/SSH selection and automatic destination folder suggestions.
- Explicit paginated GitHub repository catalog using the installed official gh CLI,
  organization scope and filtering; no automatic authentication or token import.
- Remotes panel to inspect/add/review URL changes; stale endpoint reviews are refused
  and special multi-URL/push-URL configurations are preserved.
- Reviewed last-commit amend, including message-only corrections and staged contents.
  Original author and unstaged/untracked files are preserved. An original-commit
  recovery ref is retained under `refs/ramo/amend/` before rewriting.
- Last-commit amend refuses detached HEAD, merge commits, active operations,
  conflicts, stale reviews and commits reachable from locally known remote branches.
  Fetch first; local refs cannot establish that a commit was never shared elsewhere.
- Portable beta source staging with explicit file selection, credential/path checks,
  source manifest and SHA-256 checksum; existing archives are never overwritten.
- Version/license/changelog consistency gate, beta acceptance checklist and release
  runbook. CI retains packaged installers, not raw macOS app directories.

### Changed
- Compact top bar and grouped Git toolbar with real Branch, Stash and reviewed Pop
  shortcuts; worktree, remotes and revision comparison in the keyboard-accessible menu.
- AI settings use quiet provider rows, selected-provider requirements and a collapsed
  installation section; operation forms preserve their input through repository trust.
- Simplified Clone with icon-and-label provider navigation, grouped destination,
  plain-language guidance and optional repository browsing/connection details.
- Shared buttons/inputs use smaller corner radii; clone review retains source,
  destination and explicit trust without the previous oversized form spacing.
- Double-click local branch labels to checkout directly in sidebar/graph; contextual
  Checkout and the branch selector also avoid an extra confirmation.
- Trust is requested once at repository opening, retained while open and cleared
  on close/restart. Read-only inspection remains available after declining.
- Branch clicks inspect without switching; checkout has its own reviewed menu.
- Sidebar filter, collapsible groups, single branch context action and branch switcher.
- Sidebar/graph share comparison, merge and cherry-pick drag gestures; drop opens review.
- Cleaner History/Changes navigation and explicit Publish label without an upstream.

### Known limits
- Beta candidate source, not a stable or published release. Downloaded first launch on a second Mac, Developer ID/notarization, hosted credentials
  and Windows/Linux installation/runtime acceptance are required or outstanding.
- No guided rebase/reset/reflog browser, individual-line staging, recovered discard,
  remote management or full GitKraken parity. See `docs/GITKRAKEN-GAP-AUDIT.md`.
- Amend dialog text remains in memory on operation failure, but is not persisted on restart.

## [0.1.0] - Private preview

Locally frozen macOS Apple Silicon preview; no public release date or Git tag.
- Real local repositories, graph, branch/commit menus, reviewed drag actions,
  staged commits, hunk staging, comparisons, remote transport and clone.
- Integrated editable conflict resolution with per-block and per-line source choices,
  merge completion/abort and cherry-pick/revert continuation.
- Worktree creation/guarded management, stash inspection/recovery, local draft recovery.
- Reviewed contextual AI commit/diff/conflict assistance and explicit cancellation.
- Apache-2.0 license, redistribution notices, installer/checksums and scoped benchmarks.

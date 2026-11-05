# Beta acceptance — 0.2.0-beta.4 prerelease

This is a release gate, not a parity percentage. Record exact platform, candidate
checksum, command results and synthetic fixture state. Passing source tests alone
is insufficient for a stable desktop release.

## Prepared in source

- [x] Apache-2.0 license, original-code NOTICE and dependency-notice generation.
- [x] Changelog and consistent prerelease versions; automated metadata check.
- [x] Reviewed Git operations and recovery-oriented regression tests.
- [x] Portable source exporter, checksums and exclusion of local settings/build caches.
- [x] Read-only CI build/test matrix; installer artifacts separated from public release.
- [x] No automatic inference, release publication or updater.

## Candidate acceptance

- [x] Record fresh frontend/backend, production build and dependency-audit evidence.
- [x] Record native amend review, cancel, success and rejected-hook behavior; check
      staged/unstaged/untracked bytes and author, retained original ref and draft.
- [x] Build exact versioned installer; verify image, signature, notices and checksum.
- [ ] Download/install/open on another supported Mac, without development tools/state.
- [ ] Open/clone and authenticate to a hosted synthetic repository; fetch/push/pull.
- [ ] Exercise graph/context menus, hunk staging, conflict editor/merge completion,
      sequence abort/continue, stash and worktrees using synthetic examples.
- [ ] Confirm failure/restart behavior, local draft recovery and explicit AI Stop.
      Live provider checks are optional and need separate account/usage authorization.

## Before public beta

- [x] Authorize public repository peppoasap/ramo; import only reviewed portable sources.
- [ ] Set and verify a real private security reporting channel.
- [x] Run hosted CI and record target architectures/build evidence in [BUILD-VERIFICATION.md](BUILD-VERIFICATION.md).
- [ ] Record downloaded Windows/Linux GUI installation/runtime acceptance.
- [ ] Developer ID signing/notarization for normal macOS distribution, or explicitly
      scope any unsigned technical preview and document actual installation limits.
- [x] Publish owner-authorized beta.4 with verified downloadable assets/checksums.

Hosted Windows/Linux runners are now available. Build/test results and GUI
installation/runtime acceptance must be recorded separately. Intel Mac runtime
acceptance is also outstanding. Do not infer these from macOS tests. Existing private
0.1.0 preview checks are historical, not acceptance of this candidate.

## Next daily-base work

Individual-line staging, reviewed discard with recovery, file history/blame,
reflog/reset recovery and guided rebase remain important functional gaps. Then
remote/upstream management and file watching. Maintain the full inventory in
`GITKRAKEN-GAP-AUDIT.md`; beta does not mean GitKraken feature parity.

Historical beta.1 evidence (2026-11-05):56frontend/118backend checks,4amend regressions rerun after
history exclusion; TypeScript/Vite/Tauri build and fresh npm audit pass. Native
synthetic amend/cancel/hook-failure and final trust/history/layout checks pass.
Installer mount/signature/hash evidence is in `verification/beta-build.json`.
No new live AI inference. Source archive manifest/export checks are recorded
separately after freezing. See STATUS.md for the exact scope.

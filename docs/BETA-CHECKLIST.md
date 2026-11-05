# Beta acceptance — 0.2.0-beta.2 candidate

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
- [ ] Run hosted CI; record target architecture and installation/runtime evidence.
- [ ] Developer ID signing/notarization for normal macOS distribution, or explicitly
      scope any unsigned technical preview and document actual installation limits.
- [ ] Inspect draft pre-release, attach verified assets/checksums and authorize publish.

Windows/Linux and Intel Mac support remain unverified; no available runners were
provided. Do not check them off on the strength of macOS tests. Existing private
0.1.0 preview checks are historical, not acceptance of this candidate.

## Next daily-base work

Individual-line staging, reviewed discard with recovery, file history/blame,
reflog/reset recovery and guided rebase remain important functional gaps. Then
remote/upstream management and file watching. Maintain the full inventory in
`GITKRAKEN-GAP-AUDIT.md`; beta does not mean GitKraken feature parity.

2026-11-05 evidence:56frontend/118backend checks,4amend regressions rerun after
history exclusion; TypeScript/Vite/Tauri build and fresh npm audit pass. Native
synthetic amend/cancel/hook-failure and final trust/history/layout checks pass.
Installer mount/signature/hash evidence is in `verification/beta-build.json`.
No new live AI inference. Source archive manifest/export checks are recorded
separately after freezing. See STATUS.md for the exact scope.

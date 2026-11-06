# Project status

Updated: 2026-11-05 (Europe/Madrid). Published prerelease:0.2.0-beta.4.
The owner authorized publication to https://github.com/peppoasap/ramo and public
beta build downloads. Initial192-file import published as b4f30fb; a follow-up includes supplemental
upstream license texts and standalone-export completeness. Local caches/settings, real repositories and private reference screenshots
are excluded by the explicit source allowlist. Hosted macOS, Windows and Linux builds/tests passed; beta.4 installers and checksums are public.
See docs/BUILD-VERIFICATION.md for the exact source, CI run and artifact checks. No stable-release claim.
Historical entries below describe the evidence at the time they were recorded.

## Implemented

Functional Tauri/React/Rust desktop app with actual Git, no demo data. Native folder opening/recents, reviewed repository cloning, topology-based paginated history with bounded viewport rendering and keyboard/search navigation, refs/worktrees (including creation on new or available local branches, reviewed lock/unlock and guarded inactive checkout removal), commit/file diffs, explicit revision comparisons, status, staging/unstaging, commits, branch creation/switch/rename/merged deletion/local merge, tracking checkout, selected-commit branch/tag creation, detached checkout, single-parent cherry-pick/revert, stash save/inspect/apply/pop/drop and selected saved untracked-file recovery, fetch, fast-forward pull and explicit active-branch push/publish. Mutations/network require session trust and reviewed-snapshot checks. Commit drafts are saved locally per canonical repository/worktree identity and restored on reopen; failed commits preserve them and successful commits do not erase a newer draft.

Contextual AI preview, commit drafting, diff/conflict explanation and structured merge-proposal adapters. Merge proposals are inserted only by user action into the editable result; save/stage/commit remain separate. Codex CLI 0.166.0 uses its own sign-in with model tools disabled; a native picker validates and atomically saves its executable path. Provider/model preferences remain separate. Claude CLI/API-key and loopback Ollama routes exist but live inference is unverified. Explicit Stop cancels running provider processes/local HTTP work, discards late answers and retains manual drafts/results; fresh input review is required before retry. No automatic inference, CLI installation, login, fallback, commit or push.

Apache-2.0 for original source/docs. Independent identity; selected graphite/mint design retained. Mockup history and native screenshots are separate.

## Verified evidence

- Virtual history: native pagination loads all5,011 commits from a real synthetic merge/branch/tag fixture while exposing about30–31 commit buttons per viewport. Home/End/PageDown selection/focus, keyboard commit menu/Escape, offscreen search to commit02500, branch/tag labels, branch menu trigger retention through scrolling and refresh from5,011 to300 without stale-index crashes are verified. HEAD/tree/count/clean status remain unchanged. Pure geometry/component tests cover bounds through100,000 rows, retained focus, shrinking lists and topology under search. Actual static component render uses21 rows for300/5,000/50,000 linear synthetic commits; timings are separately scoped and do not measure native FPS or a baseline speed-up. Benchmark disables watchers/websockets/dependency discovery; final run has empty stderr. `docs/verification/history-virtualization.json`, `history-render-performance.json`, ADR0018.

- AI cancellation: shared one-use native preview/job registry and renderer request lifecycle, cooperative process/version-probe cancellation with descendant cleanup, and cancellable async loopback HTTP retaining proxy/redirect/size/deadline boundaries. Native synthetic Stop preserves a multiline Unicode commit draft through restart, requires fresh review, and permits a new fixed-response review without insertion. Merge result edits made while waiting survive Stop and local draft closure. Both Git fixtures retain exact HEAD/index stages/status/file bytes. Final build repeats commit Stop and verifies the next preview clears old status. Original Codex provider/model preference restored; synthetic server stopped and port checked closed. No live inference. `docs/verification/ai-cancellation.json`, ADR 0017.

- Worktree management: native reviewed lock/unlock, review retained through session trust, locked-removal refusal, open-tab protection and actual tab closure/unregistration followed by clean removal. Branch retained and source HEAD/index/dirty/untracked bytes unchanged. Ignored-data removal refused with a visible dialog error; exact bytes retained. Primary checkout actions refused. Nine additional real-Git workflows cover cleanliness/ignored/hidden index flags, nested/empty directories, symlink and canonical boundaries, open/untrusted/primary/detached/active operation gates, stale reviews and branch retention. Final index.lock guard has a targeted passing regression after the complete run. `docs/verification/worktree-management.json`, ADR 0016.

- Selected stash-file recovery: native restoration of a missing saved untracked file during a real marker-free stash conflict, with exact saved bytes, unchanged index/conflict/HEAD/stash objects, and a repeated attempt refused without overwrite. Selection/review survives session trust. Six real-Git checks cover binary/Unicode/newline paths, deleted tracked parents, CRLF/no EOF newline, source identity/stale reviews, symlink/executable/size guards and active-sequence refusal. `docs/verification/stash-file-recovery.json`, ADR 0015.

- Repository cloning: native parent/local-source pickers, immutable exact-source/destination review, approval-gated execution, local bare clone and automatic clean checkout opening without automatic write trust. Source dirty/staged sentinels stay unchanged; tag and origin/main are retained. HTTPS preview was reviewed without executing a network request. Six real-Git fixtures cover grants, exclusive new destinations/late collision, symlink-parent replacement, empty clone, failure preservation and one-use review; URL checks reject unsafe/custom transports and embedded credentials. `docs/verification/clone.json`, ADR 0014.

- Multi-step native cherry-pick acceptance: first conflict is resolved/staged, Continue commits its recorded message and pauses at a second file with updated history and disabled Continue. Reviewed Abort restores exact pre-sequence HEAD/tree/both files. In another fixture, reviewed Skip discards a saved/staged second resolution while retaining the successful first commit. Both flows remove sequence markers and preserve unrelated untracked work. The extended `create-sequence-fixture.py --multi` records pre-state and ordered targets; existing single cherry-pick/revert modes were regression-checked. Runtime/installer unchanged in this verification slice. `docs/verification/multi-sequence.json`.

- Stash-conflict completion: native unmerged files with no active operation show ordinary resolve/review/commit guidance, not external continue/abort. The same editor saves the reviewed exact result through trust, then the ordinary composer creates a single-parent commit of README only. Both saved stash objects and saved untracked contents remain intact; the tree is clean. Four rendered guide checks preserve rebase/unknown fallback and real merge/sequencer controls. A new Git workflow also preserves unrelated untracked work. `docs/verification/stash-resolution.json`.

- Revision comparison: native branch menu and history-toolbar entry, direct pair/swap with three changed files, common-ancestor mode with only two target files, visible effective object hashes, read-only use without trust and exclusion of dirty/staged work. External branch movement leaves the displayed pair pinned; invalid expressions clear old results; full hashes outside loaded history and identical-tree empty state work. Backend checks cover unrelated/multiple ancestors, literal paths, binary/deletion handling and preview bounds. `docs/verification/comparison.json`, ADR 0013.

- Stash inspection and reviewed Apply/Pop/Drop: native read-only tracked/untracked previews, review retained through trust, Pop restores exact files and removes only the chosen entry, Drop preserves exact HEAD/index/file bytes with dirty and staged work present. A conflicting native Pop retains both stash entries and all three unmerged index stages, opens Changes and blocks further stash writes. Git may leave saved untracked files only in the stash after a conflict. Object identity protects same-subject replacement and stack shifts. A local Apple Git 2.54 literal-mode cleanup issue is fixed only for path-free stash subcommands. `docs/verification/stashes.json`, ADR 0012.

- Conflict navigation: native verification on a 320-line TypeScript file with conflicts at source lines 127 and 302 covers active highlighting, linked Previous/Next, independent reading with Follow off, explicit Locate, clearing old line picks, one-line resolution of only the second block, undo/redo and reviewed save through trust. Exact expected file bytes are staged, and the completed two-parent merge is clean. Pure tests cover CRLF, Unicode, EOF, diff3 base and ambiguous matches. `docs/verification/conflict-navigation.json`, ADR 0011.

- Worktree creation: native macOS acceptance covers a new branch at reviewed HEAD and a context-menu existing branch, native parent picker, form retention through trust, refreshed list and explicit open. A commit in the linked checkout changes only its own index; source HEAD, staged tree and file bytes remain unchanged. Six real-Git checks cover destination non-overwrite/traversal, branch/tag ambiguity, repository-scoped picker grants, parent replacement and stale occupancy. `docs/verification/worktree-creation.json`, ADR 0010.

- Native hunk staging: stage only the second block, unstage it back to an empty index, then commit only the first block and leave the second unstaged. Exact working-file bytes remain unchanged throughout; history updates to the new commit. Backend checks bind a selected index to the displayed native patch hash and cover CRLF, missing final newline, literal filenames, custom diff settings, worktree isolation, unsupported modes/binary data and size limits. `docs/verification/hunk-staging.json`.

- Resolution recovery: native normal restart restores exact multiline Unicode text without modifying the Git file; an external file edit requires explicit saved-result review before Save; Keep local draft and close restores on reopening; explicit discard removes only its recovery entry. Eight storage tests cover context separation, changed fingerprints, detected competing writes, cleanup, quota/size/corruption and sensitive filenames. `docs/verification/resolution-recovery.json`.

- Sequencer recovery: typed cherry-pick/revert continue/skip/abort, conflict/unstaged/wrong-kind guards and review hashes of pending todo/head/abort-safety/options. Real Git checks cover a second conflict and full-sequence abort, plus explicit skip for empty resolutions. Native cherry-pick resolution/continue preserves the recorded message and updates history/selection to the new commit with a clean tree. `docs/verification/sequences.json`.

- Branch tips have a dedicated graph column and shared branch context menus. Commit menus expose inspect/copy/create branch/tag/detach/cherry-pick/revert; branch menus include rename and tracking checkout. Real Git fixtures cover new operations, non-overwrite/dirty-work guards and merge abort.
- Full-space CodeMirror merge editor native acceptance: sources/result, accept-both, undo/redo, unresolved-save guard, retained buffer through trust, reviewed save-and-stage and two-parent completion. A TypeScript fixture with two conflict blocks verifies selecting only source line 7 replaces just its active conflict and keeps the other unresolved. Native commit-menu tag creation updates graph labels. AI merge preview shows only the selected file’s 235-byte synthetic versions and requires a separate generation click; no live merge-generation call was made. `docs/verification/merge-editor.json`.

- TypeScript and production Vite build pass. **51 frontend tests pass:** graph topology, draft recovery, conflict eligibility/choices, AI proposal validation, context-action policy, unified/combined diffs cancellation/late-response/IPC-retry/status lifecycle cases, and virtual-history geometry/component rendering.
- Latest complete backend run: **114 tests** (22 unit/transport, 86 real Git fixtures, 6 process), four workers with unchanged ordinary Git deadlines. Cancellation checks cover one-use preview/race/cleanup, synthetic CLI probe and generation termination, running/orphaned descendants, HTTP header/body disconnect, response caps and proxy/redirect refusal. An early adapter test caught cancellation being rewritten as a sign-in error; cancellation is now preserved and the full run passes. At that backend run, frontend47 tests passed. Latest history slice: frontend51 tests and TypeScript/Vite build pass; backend unchanged and not rerun. Hosted CI remains configured but unrun; native runtime evidence remains scoped to macOS Apple Silicon.

- App-initiated merge backend checks: clean fast-forward targets the explicit local branch even when a same-named tag exists; squash/no-commit configuration cannot change the requested behavior; divergent clean branches create a two-parent commit; dirty work is refused unchanged; a conflict leaves the index available for explicit resolution. History reloads after history-changing actions and selects the resulting HEAD.
- Native app-initiated merge acceptance: first click opens the reviewed confirmation; trust precedes writes; a real conflict opens Changes, preserves the index and blocks incompatible operations; explicit resolution and Complete merge create two parents. History automatically changes from 3 to 4 commits and selects the new HEAD. `docs/verification/start-merge.json`.
- Conflict acceptance: separate unresolved-file list; Stage all/Unstage cannot silently resolve files; explicit Mark resolved after external edit; incompatible branch/stash/sync actions blocked; merge completion allowed with no staged tree difference. Actual native merge created two parents while retaining the first-parent tree. Rebase ordinary commit is rejected in real Git tests. Merge file lists explicitly compare the same first parent as their file diffs, including zero-tree-change merges on Git 2.33.1. Diff loading clears when no file remains. `docs/verification/conflicts.json`.
- Native draft recovery acceptance: a controlled synthetic pre-commit hook rejected a commit; the exact multiline message survived app quit/restart. After a successful one-file commit, the draft was empty both immediately and after restart; unstaged README/test file remained. `docs/verification/draft-recovery.json`.
- Native macOS UI initial acceptance: synthetic history/merge, diff, trust, staging, AI input preview/provider settings, manual composition and real two-file commit, leaving unstaged README intact. `docs/verification/desktop-changes.png`.
- Codex 0.166.0 installed only under ignored `releases/local/tools/` with explicit user authorization. System CLI 0.139.0 remains untouched. Inference is pinned to the version checked by the adapter; older versions fail before receiving a diff.
- **One newly authorized live generation succeeded:** gpt-6.1-sol, 284 bytes from synthetic staged `src/search.ts` only; output: “Extract normalized search query into a local variable”. `docs/verification/ai-smoke-0.166.json`. No Git write or additional inference. The earlier 0.139.0 failure is preserved in `ai-smoke.json`.
- Actual adapter arguments were sent to a synthetic loopback server using an empty, credential-free CODEX_HOME: **zero model tools**. Server rejects all requests; no inference. Reproduce with `python3 scripts/check-codex-contract.py`; evidence `docs/verification/codex-contract.json`.
- Performance fixture: 5,000 commits / 2,000 files, seven warm samples; adaptive polling improved local snapshot/history medians. `npm run bench:git`, raw data and limits in `docs/verification/PERFORMANCE.md`. This is backend timing, not a universal UI speed claim.
- Generated 20 MiB binary exceeds the old buffered patch cap; incremental fingerprint hashing supports staged/unstaged snapshots and rejects stale content. Preview/output caps and timeouts remain. `docs/verification/large-binary.json`.
- Local Apple Silicon app/DMG: ad-hoc signature and image checksum checked, mounted read-only to verify app and license resources. No Developer ID or notarization. Latest artifact/checksum: `docs/verification/local-build.json`; installer `releases/local/0.1.0/Ramo_0.1.0_macos-arm64.dmg`.
- `npm run package:mac:local` reproduces the local DMG without publication. A new HFS+ image is verified before replacing the installer; the final image was mounted read-only, checked and detached. Bundling generates 352 target dependency notices on macOS. CI matrix exists but has not run; no Git repository or remote was initialized.

## Limits

Windows/Linux runtime and installers are **deferred by the user until a machine/runner is available**. Configuration is not proof of support. Windows npm `.cmd` wrappers, older macOS, hosted remote credential UX, very large/merge-heavy repositories and managed Codex environments remain unverified. Non-UTF-8 paths are unsupported.

Cloning supports configured HTTPS/SSH and picker-selected local sources. Actual native clone acceptance is local-only; hosted clone authentication is unverified. Large clones have a 120-second deadline and no exposed cancellation. Failed clones may leave partial output requiring inspection; no automatic Ramo deletion/retry.

Native Codex selection, version validation and persistence after application restart were exercised successfully without inference. The successful AI generation exercises the actual Rust adapter, not a second UI-button inference. Token refresh/revocation, every model/account, Claude and Ollama live inference remain unverified. No additional billable test is authorized by these records. Other Codex versions require contract verification before compatibility is expanded.

Text conflict editing is integrated using CodeMirror: read-only source panes, optional base, editable result, conflict choices/navigation, individual source-line selection, active range highlighting, linked conflict navigation, syntax, search and undo/redo. Reviewed save stages the result; the merge can complete or abort in Ramo. Cherry-pick/revert can continue, skip or abort through reviewed operation controls. Binary/symlink/submodule conflicts, rebase and mixed/unknown sequencer instructions still use Git outside Ramo. Edited resolution buffers have local recovery, with explicit review when Git changed. Normal restart is verified; survival of every OS/disk failure and atomic multi-window coordination are not established. Known sensitive filenames opt out with a visible message. Whole-hunk staging supports ordinary modified tracked text patches up to 256 KiB while no Git operation/conflicts are active. Individual line staging, partial new/deleted/renamed or mode changes, worktree move/prune, AI shutdown/crash recovery and file watching remain follow-up work. Core refresh occurs on focus/timer. A timed-out write may already be applied: refresh before retrying.

The three reported Vitest development-tooling findings were remediated by pinning Vitest 5.1.0 while retaining Vite 6.4.5. At the tooling update, a clean `npm ci`, the then-current 30 tests, TypeScript/Vite production build and `npm run audit:dependencies` pass with Node 22.13.0. npm currently reports zero known vulnerabilities; this is not a general security audit. The audit command is added to the configured CI matrix, which remains unrun. `docs/verification/dependency-audit.json`.

No public remote, deployment, release, verified security contact or naming review. Public source/binary publication requires separate authorization. Test tool installations are not bundled.

- Source-line selection discoverability: primary clicks on source line numbers now toggle the same accessible + selections. A visible hint and permanent disabled-until-selected action expose the workflow before interaction. Native rebuilt-app acceptance selects/deselects ours line 7, applies only that row to the active block, retains the other conflict and unresolved-save gate, then verifies undo/redo; original disk markers remain intact. Picking is disabled while saving. 43 frontend tests and TypeScript/Vite build pass. Local DMG rebuilt, verified, mounted read-only, signature/executable/license resources checked and mount detached. Backend unchanged; 94 backend checks above are prior evidence, not rerun for this UI-only slice. `docs/verification/line-selection.json`.

## Compact graph and reviewed gestures — 2026-11-04

Implemented one prioritized reference per 48px graph row with +N overflow menus for all other branches/tags. Current local branch wins priority; labels truncate with full titles and no per-row horizontal scrollbar. Sidebar merge shortcuts appear on hover/focus. Reference menu → branch action menu retains keyboard focus. Pointer gestures use a six-pixel threshold, pointer capture and an overlay toolbar so rows do not move while dragging; native HTML text/file dragging does not own the gesture.

Drop branch/commit onto a commit for immutable read-only comparison. Drop another local branch or a non-merge commit onto the current branch label/named overlay target to open merge/cherry-pick review. Drops do not execute Git. Existing trust/fingerprint/clean-tree/operation guards remain; review now survives a session trust grant and still requires a second Continue. Context menus remain the keyboard alternative. No AI inference, new dependency, project repository initialization or publication.

Evidence: `npm test` passes 56 checks in12 files; TypeScript and Vite build pass in `npm run package:mac:local`. Native Apple Silicon acceptance verifies +N with three branches/two tags, correct nested branch menu/focus, actual pointer comparison with pinned SHAs, reviewed merge/cherry-pick, cancellation preserving the original clean HEAD, trust preserving merge review, and a completed synthetic two-parent merge with both files retained and a clean tree. The development-only `scripts/graph-preview.html` harness also verifies pointer comparison/current-branch callbacks and a simple click without accidental drag; it imports no native bridge and is not part of the production entry. `scripts/create-graph-fixture.py` creates the disposable actual Git fixture. Prior114 backend tests were not rerun because native Git action code is unchanged. See `docs/verification/graph-interactions.json` and ADR0019.

Local ad-hoc DMG rebuilt, image checksum verified, mounted read-only, bundled executable/license/notices and ad-hoc signature checked, own volume detached. No Developer ID/notarization or public release. Windows/Linux pointer and installer acceptance remain pending. Sources are visible graph labels/subjects only; no sidebar dragging, multi-commit drag, arbitrary branch targets or automatic graph scrolling yet. Ordinary scrolling remains necessary for long histories or very wide topologies; the removed scroll is inside reference rows.

## First shareable preview and performance report — 2026-11-04

User authorized preparing version0.1.0 for friends/colleagues. The local DMG is frozen into a private macOS Apple Silicon preview with installation notes, Apache-2.0/notices, feedback template, offline Italian HTML/Markdown performance report, raw JSON and a portable source snapshot. This authorizes local delivery files, not uploading/public publication. `scripts/package-preview.py` verifies the existing installer hash, exports a deliberate source allowlist (no build caches, provider settings, credentials, private repositories or design captures), writes manifests/checksums and refuses overwriting an existing frozen version. `scripts/report-preview.py` derives the report from captured evidence. Source README/agent entry point and a local-build hash excerpt are made portable; code and lockfiles are retained.

Fresh checks actually run: `npm test`56 passed; full offline backend suite114 passed (22unit/transport,86Git,6process; four workers), `npm run audit:dependencies`0 known npm vulnerabilities. Current native runtime code and verified DMG are unchanged since the graph acceptance. `hdiutil verify`, read-only mounting, exact app version, executable/LICENSE/NOTICE/third-party notices/Applications shortcut and `codesign --verify --deep --strict` pass; own mount detached. `spctl --assess --type execute` rejects this ad-hoc build (exit3). Developer ID/notarization and downloaded first launch on a second Mac remain missing; the tester guide states this and links official Apple instructions. No new AI inference.

New release-profile benchmark `src-tauri/examples/delivery_performance.rs` / `npm run bench:delivery`: actual temporary Git fixture,5000commits/2000files,100modified files, one discarded warm-up plus9observations for each of13metrics. Typed mutations include native review/result snapshots. Asserted unstage state, distinct commit messages retaining99other changed files, and divergent two-parent clean merge. Final medians: history300 53.01ms, commit diff36.15ms, clean snapshot288.66ms, dirty100 snapshot204.79ms, commit450.13ms, stage100 446.10ms, unstage100 444.00ms, same-tree branch switch447.51ms, three-way merge486.07ms. Not UI/IPC/cold launch/network/AI/FPS timing. Prior two diagnostic runs are preserved and not pooled into final metrics. `node scripts/benchmark-history-render.mjs` also rerun: bounded21initial rows at300/5000/50000loaded commits; static SSR only.

Installed GitKraken12.6.1 loads the same5011-commit native acceptance fixture. No comparable instrumented end-to-end speed measurements exist, so no speed percentage is claimed. Local arm64 bundle logical bytes: Ramo16957306, GitKraken659694283 (97.4% fewer bytes; system WebKit versus bundled Electron, different feature scope). This is disk footprint, not RAM/CPU/speed. Machine: M1Pro/16GiB/macOS26.6.2/AppleGit2.54.0. Evidence: `docs/verification/delivery-performance.json`, preliminary runs, `delivery-history-render.json`, `delivery-environment.json`, `distribution-footprint.json`, `preview-validation.json`; readable report under `docs/distribution/`.

Preview archive completed: `releases/preview/0.1.0/Ramo-0.1.0-preview-macos-arm64.zip`,7182259bytes, SHA256 `f2dd14a4ec2a909236ca46e3ffa85b1438d26eb6396ee9ed60c7a5ba0be70e7d`. Outer ZIP integrity,17payload checksums and170exported source manifest hashes pass;171source files including the manifest. Report HTML browser-read and visually inspected. Frozen source status was captured immediately before export; the final archive receipt is deliberately outside the frozen source/ZIP to avoid self-referential hashes. `docs/verification/preview-archive.json` records receipt. No upload or message was sent to third parties.

## Daily-client and GitKraken gap audit — 2026-11-05

Read current menus, graph gesture policy, sidebar/inspector/native action contract,
README/FEATURES/ROADMAP and preview evidence; checked official GitKraken Desktop
documentation. Added `docs/GITKRAKEN-GAP-AUDIT.md`: actual feature/menu inventory,
implementation vs acceptance gaps, proposed macOS daily-base completion criteria
and five work packages. No arbitrary completion percentage or feature-parity claim.
Corrected stale FEATURES next-slice text and ROADMAP worktree-removal backlog;
prior51-test entry is explicitly historical (latest recorded56frontend/114backend).
No runtime changes, test rerun, live inference, installer rebuild or frozen ZIP
modification. This is an audit/proposal, not authorization to publish or evidence
that missing features have been implemented.

## Workbench UX and shared sidebar gestures — 2026-11-05

Implemented first local UI pass preserving the graphite/mint target: sidebar
branch filter with Escape clearing, collapsible named sections, empty remote/stash
groups initially closed, a single ellipsis per branch instead of merge/delete
hover buttons, inspection-only branch clicks, actual branch switcher with reviewed
checkout and disabled current/no-local choices. History/Changes lead the mode bar;
Compare is secondary. No-upstream transport says Publish with an accurate tooltip.
Provider settings are no longer duplicated in the sidebar. History inspector no
longer labels an unrelated selected commit with the current working branch.

App owns one typed pointer gesture session shared by Sidebar/History. Sidebar
branch sources and current-branch targets now work with the same pinned comparison
and reviewed merge/cherry-pick policy; no implicit checkout. Only threshold crossing
updates session UI, not every move within a target. No new native operation,
dependency, live inference or external publication. ADR0020 and UI-CONVENTIONS.md
record the maintainable interaction contract.

Checks actually run:56frontend tests pass; TypeScript/Vite production build and
local ad-hoc DMG packaging pass. Native synthetic review verifies branch inspection
without checkout, filter/Escape, worktree collapse/reopen, ellipsis/keyboard access,
branch-selector checkout review/Cancel, sidebar→current-branch merge review/Cancel,
sidebar→graph pinned comparison and graph→sidebar cherry-pick review/Cancel.
HEAD/branch/index tree/status are unchanged from the pre-check proof. Before/after
history screenshots share2880×1898; later flows reflect window resize to3024×1898.
Saved captures were inspected. No WCAG/FPS/all-platform claim. Native Git code is
unchanged; the114backend checks remain prior2026-11-04 evidence, not rerun here.

Final local DMG image verified; mounted read-only to check executable, license,
notices and ad-hoc signature; own volume detached. Latest artifact hash/size in
local-build.json and ux-order.json. No Developer ID/notarization or second-Mac
acceptance. The7.18MB frozen tester ZIP still has its original checksum and prior
UI; it was not silently replaced. Full scoped audit/evidence:
`docs/verification/ux-order/README.md`, `docs/verification/ux-order.json`.

## Reviewed amend and beta preparation — 2026-11-05

Current source version is 0.2.0-beta.1 across npm, both lockfiles, Cargo and Tauri.
The UI reads its version from package metadata. No public Git repository, remote,
tag or GitHub Release has been created. The 0.1.0 tester archive is unchanged.

Implemented a separate full-message Amend last commit dialog and native typed
Amend action. Message-only correction and index-only content updates preserve
original author and unstaged/untracked work. Expected HEAD, fingerprint, trust,
attached non-merge HEAD, no active operations/conflicts and absence from locally
known remote reachability are checked. Local refs cannot prove no prior sharing;
fetch before review. An original-commit recovery ref is retained before invoking
Git, including on hook failure, and excluded from ordinary history. Recovery is
currently external Git; no Undo/reflog GUI is claimed. The persistent normal commit
draft is untouched; amend failure text is retained only in the open dialog.

Added CHANGELOG.md, .gitignore, release metadata gate, portable beta source exporter,
installer-only CI artifact/checksum collector, issue/PR templates, BETA-CHECKLIST.md,
RELEASING.md and ADR0021. CI uses Rust1.95 and read-only permissions; it does not
publish. Source export is selected explicitly and omits historical private/local
captures and build/settings/CLI folders. Pattern scanning is not exhaustive secret
proof. Export adapts README and AGENTS only; archive contents still require review.

Fresh checks:56 frontend tests; full offline backend118 (22unit/transport,90realGit,
6process). After native acceptance exposed recovery refs in --all history, added
an exclusion and regression; all4amend cases rerun and pass. TypeScript/Vite,
release-profile Tauri app,352dependency notices and local ad-hoc DMG build pass.
Fresh npm audit:0known vulnerabilities (registry access; no AI inference). No new
backend performance or comparative GitKraken speed claims.

Native macOS synthetic acceptance: complete original message/body; cancel leaves
HEAD/index/status unchanged; trust before execution; successful index-only amend
preserves working/untracked bytes and normal message draft; rejected commit-msg
hook leaves HEAD/index/working sentinels intact and amend text visible. Normal
draft survives app restart. Final build verifies one ordinary history commit despite
recovery refs, full-width message editor and trust return without false error.
No real repositories mutated. Synthetic rejecting hook removed after verification.
Screenshots and scoped evidence: docs/verification/beta-amend/ and beta-build.json.

Candidate DMG is under releases/local/0.2.0-beta.1/. Image verification, read-only
mount, exact app version/executable/license/notices/Applications shortcut and
codesign strict verification pass; own mount detached. It remains ad-hoc, without
Developer ID/notarization. Second-Mac downloaded launch, hosted transport acceptance,
public security channel and hosted CI/platform runtime gates remain open. This is
an installable local candidate, not an accepted public/stable release.

## Direct checkout and opening trust — 2026-11-05

User requested double-click checkout and fewer confirmations. Local branch labels
in Sidebar/History now inspect on single click and invoke the shared Checkout menu
policy on double click. Contextual Checkout and the branch selector invoke typed
Switch directly. Current/busy/active-operation/conflict gates remain; remote refs
keep the tracking-branch form. Right-click/Shift+F10 menus and reviewed merge/delete
remain. No force, discard or auto-stash was added; native Git refuses overwrites.

Trust previously lasted for the open repository but appeared on first mutation.
It now appears once at opening. A per-open renderer record prevents prompting
again on tab activation even after declining; explicit write attempts can request
trust again in read-only mode. Native trust remains in memory per canonical checkout,
retained across refresh/register/branch switching, cleared on close or app restart.
No persistent trust policy or automatic operation after trust was introduced.

Checks:57frontend tests, TypeScript/Vite and release Tauri app build pass. Two new
real-Git tests pass: trust lifetime/reopen/restart, and compatible checkout plus
refused overwrite retaining HEAD/index/working bytes. Full118backend suite from
the prior slice is not relabelled as a full120-test rerun. Native synthetic opening
shows trust immediately; after acceptance sidebar double-click→topic and graph
double-click→main perform direct checkout without another trust or confirmation.
Round trip preserves original HEAD/index/untracked sentinel. No real repo writes,
AI request, external publication or replacement of frozen candidate archives.
The running local app includes this revision; prior DMG/ZIP candidates do not.
See ADR0022 and UI-CONVENTIONS.md. Next task remains below.

## Hosting shortcuts, GitHub catalog and remote configuration — 2026-11-05

Implemented provider-based Clone navigation using the existing workbench palette:
raw URL, GitHub/Enterprise, GitLab, Bitbucket, Azure DevOps, self-hosted and local
picker. HTTPS/SSH templates suggest a destination name without replacing manual
folder edits. No provider is labelled Connected without actual account behavior.
GitHub/Enterprise support explicit paginated listing via installed official gh,
optional organization and filtering loaded repositories. No auto-load/login/install,
token import or new dependency. A typed native endpoint fixes the GET API paths,
validates host/scope/page and bounded returned metadata, runs from a temporary
non-repository folder and sanitizes failures. Other provider account catalogs are
not implemented; their clone URL paths are functional. See HOSTING.md/ADR0023.

Remotes toolbar panel lists safe fetch/push endpoints even without fetched branches.
Add/change an ordinary HTTPS/SSH URL uses trust, snapshot freshness and endpoint
review. Saving is local configuration only. Endpoint output enters fingerprints.
Explicit push URLs, multiple URLs, local/custom transports and embedded-credential
URLs are not edited; sensitive/unsupported URL values are hidden. No remote removal,
rename, upstream editor, auto-fetch or OAuth credential storage is claimed.

Checks:60frontend tests; full offline backend125passed (25unit/transport,94realGit,
6process), after fixing two new test comparisons to trim Git's trailing newline.
Then tightened catalog host/name validation; its3synthetic adapter cases rerun and
pass. TypeScript/Vite and release Tauri app builds pass;352notices, no dependency
changes. No GitHub account API, hosted clone, new AI inference or live provider auth
request was made by the agent. API docs were checked against official GitHub docs.

Native synthetic acceptance: provider navigation, GitHub URL/folder suggestions,
HTTPS→SSH changes and manually edited folder preservation. Added origin with an
invalid-example HTTPS domain then changed to SSH through the reviewed Remotes UI;
actual config, HEAD/index/untracked sentinel checked outside UI. No network action.
Before/after captures under docs/verification/hosting/. Final native build verifies
compact helper copy, wrapped provider labels, destination fields, unused default
remote name (upstream) and local rejection of an Enterprise hostname containing
a scheme before any CLI/account access. The frozen0.1.0 and beta.1
archives/installers remain unchanged; current source/native app is a later local
revision. Windows/Linux and real hosted account flows remain unverified.

## Clone UI simplicity pass — 2026-11-05

User requested less rounding, clearer spacing/icons and a less technical workflow.
Kept graphite/mint and the real Clone behavior. Shared buttons/inputs now use3px
corners and dialogs6px. Clone has quiet icon-and-label provider rows (existing
Phosphor library), a plain-language introduction, grouped destination/folder fields,
one primary review action and source/destination review in ordinary readable text.
GitHub catalog and connection settings use native details/summary, collapsed until
requested; no automatic account query. Full address remains available in options
and review. Metadata IDs/backend contracts are unchanged. Added an optional dialog
className for scoped layout rather than another modal framework/dependency.

Checks:60frontend tests pass; TypeScript/Vite and final release Tauri app build pass,
352dependency notices. Native synthetic checks at2880x1898 and1922x1280 capture sizes
(about1440x949 and961x640logical) confirm the main form without scrolling. Search
and connection disclosures expand; HTTPS→SSH updates the reviewed address; native
destination picker works; reviewed source/destination and trust checkbox remain;
Back and invalid input retain manual folder text and disable an invalid review.
No clone/network/account/AI request executed; reviewed temporary destination was
not created. Final single correction batch widens the provider rail so long labels
fit, then both widths confirmed. Captures in docs/verification/clone-simplicity/.
No formal usability/WCAG certification or Windows/Linux UI acceptance claim.

Native Git code is unchanged; prior125backend pass remains historical evidence,
not a rerun for this visual slice. The running app/source include this revision;
frozen beta ZIP/DMG/source snapshots remain unchanged. Updated DESIGN.md,
UI-CONVENTIONS.md, clone surface brief and changelog for future maintainers.

## Compact toolbar and AI settings — 2026-11-05

Implemented compact48px tab/command strips, fixed30px top actions and consistent
margins. Clone no longer stretches to the header height. New LocalActions component
exposes real create-and-switch Branch, Stash and reviewed latest-stash Pop shortcuts.
The shared keyboard/pointer overflow menu opens remotes, worktree creation and revision
comparison. Existing native trust/fingerprint checks and operation/conflict guards
remain. Stash is unavailable without changes/HEAD; Pop without saved stashes.
Operation forms use narrower440px dialogs and preserve entered values when trust
is requested; trusting does not auto-submit. AI settings use quiet provider rows,
requirements of the selected provider, collapsed Codex installation controls and
busy feedback/locking for CLI selection and local-model discovery. Provider config,
credentials and inference contracts are unchanged; no new dependencies.

Evidence:60frontend tests/13files pass; TypeScript, Vite, release metadata gate and
final Tauri release app build pass (352dependency notices). npm/npx links in the
Node22 installation were broken, so equivalent scripts ran directly via Node22:
vitest/vitest.mjs run, typescript/bin/tsc --noEmit, vite/bin/vite.js build and
scripts/check-release.mjs. Tauri build used the same direct-node beforeBuildCommand.
Native Git backend unchanged; prior125backend result was not rerun for this slice.

Native macOS synthetic acceptance created feat/toolbar, reviewed/popped a stash,
then saved it again via the toolbar. Disk checks confirm clean worktree, two saved
stashes and exact tracked/untracked contents preserved. Decline-trust → enter branch
name → submit → trust retained the name and required explicit resubmission. Keyboard
Down/Return in overflow opened the worktree form. AI selection changes requirements;
Ollama without model disables Save. No settings saved, hosted account, clone, network
Git operation or inference executed. Clone form still opens with its existing flow.

Inspected at2880x1898 and1922x1280 capture sizes (about1440x949 and961x640logical).
Initial settings overflow at minimum height was corrected once by showing only the
selected provider requirements; final default settings and branch form fit without
scroll. Advanced installation disclosure may scroll. Toolbar stays on one line at
both widths. Captures and synthetic disk receipt:docs/verification/toolbar/.
No formal usability/accessibility certification or Windows/Linux acceptance claim.
Current source and rebuilt local app include this slice; frozen beta installers,
source snapshots and preview archives remain unchanged. No publication performed.

## Beta.2 distribution refresh — 2026-11-05

Version0.2.0-beta.2 synchronized across npm/Cargo locks and Tauri. Previous beta.1
archives preserved. Built the latest toolbar/settings source as a new Apple Silicon
macOS DMG with ad-hoc signature. DMG checksum verification, read-only mounted bundle
version, signature integrity, Applications shortcut and all three license/notice
resources pass. Native launch shows beta.2 and loads synthetic history/toolbar in
read-only mode. First downloaded launch on another Mac, Intel Mac, Developer ID
and Apple notarization remain unverified/unavailable.

Windows has no available native host/runner here (only the macOS Rust target is
installed). Added a fail-fast PowerShell packaging command and prerequisites/smoke
check guide, included in portable source exports. The script runs locked dependency
installation, frontend/native tests and NSIS build, refuses existing output and
collects a version-matching installer plus checksums/notices. It has not been
executed on Windows; the Windows build kit is source, NOT an installer or verified
platform support. Existing GitHub Actions matrix is prepared but not dispatched;
no Git repository/remote created or source uploaded/published.

Checks:60frontend tests and version metadata pass; production TypeScript/Vite and
Tauri macOS bundle pass; current npm audit reports0vulnerabilities (registry access
needed a host retry after sandbox DNS failure). Full backend regression passes:125tests
(25unit/transport,94real-Git fixtures,6process). Share packages contain only
application/source, notices, checksums and tester guidance; no account settings,
provider CLIs, personal repositories or private screenshots.

Share-ready Mac ZIP is releases/share/0.2.0-beta.2/Ramo-0.2.0-beta.2-macos-arm64.zip
(7,258,628bytes; SHA256 in adjacent .sha256 and SHA256SUMS). The separately labelled
Windows source build kit is in the same folder; no setup.exe is included. Outer ZIP
integrity and181portable source manifest entries verified. Source export was frozen
before this final result annotation; its earlier STATUS refers to the bundled report.

Next distribution gate: use an explicitly authorized Windows runner/repository to
execute the build, download the setup.exe and perform synthetic install/Git/runtime
acceptance. A Windows source ZIP must not be given to friends as an installer.
For macOS, perform downloaded first-launch acceptance on a second Apple Silicon Mac.

## Public repository preparation — 2026-11-05

README rewritten for GitHub with a real synthetic desktop screenshot, feature
coverage, provider prerequisites, source/build instructions, platform limitations
and a short three-step roadmap. Contributor/security guidance now references the
real repository without inventing a private contact address. Original generated
mockups remain labelled and separate from actual application screenshots. CI keeps
read-only permissions for build/test; only the tag-gated prerelease job can write
release assets. Existing releases are never replaced. Runtime acceptance remains
separate from a hosted build result. Current frontend60tests and metadata gate pass.

## First hosted Windows findings — 2026-11-05

Hosted macOS build/tests/installers passed on c4ef795. Windows reached real-Git
fixtures and failed21/86: inherited autocrlf changed expected fixture bytes,
four fixtures used Unix-only newline filenames, Git for Windows refused native
verbatim destination paths for clone/worktree, and worktree management compared
verbatim/native and Git slash paths as unequal. No failed Windows installer was
published. Linux build is still being observed.

Beta.3 keeps beta.2 local packages immutable. native_paths.rs converts only actual
native path arguments (not URLs, messages or arbitrary Git arguments), supports
normal drive/UNC forms and refuses other device namespaces. Worktree membership,
canonical/symlink/common-directory and open-tab removal guards remain. Fixtures
set core.autocrlf=false locally rather than altering user/global Git configuration;
Windows uses legal Unicode/bracket filenames, Unix retains newline cases. Two
portable path regression tests and the full127backend suite pass on macOS. Linux
packaging exposed three missing upstream license files; exact-revision MIT texts
for dlopen2/dlopen2_derive and libappindicator-sys are included as fallbacks. Windows rerun now passes85/86real-Git cases; the remaining assertion compared
Git display paths to native canonical spelling after a successful lock, and now
checks canonical identity. Linux builds and generates442notices, but the upstream
RPM bundler rejects prerelease version syntax. CI now requests explicit DMG/NSIS/
DEB formats, matching the published beta download scope. All tag jobs must pass before
release assets are published. Windows now passes the full backend check, including
the final lock/unlock fixture. macOS and Linux installer jobs pass on ccfaebc.
A Windows-target metadata preflight exposed missing webview2 crate license files;
the exact-revision MIT text is included and the355-entry inventory passes on macOS
using Windows dependency metadata (not a Windows build claim). CI now performs
this source-license preflight before expensive compilation. No failed platform
installer is published.

## Public beta.3 verification and Mac packaging correction — 2026-11-05

Tag v0.2.0-beta.3 at5f8cfff passed all hosted jobs and published three installers
plus SHA256SUMS. Anonymous downloads and all three hashes passed. Backend counts:
macOS/Linux127 each (27unit/transport+94Git+6process); Windows110 (24unit/transport+
86Git, Unix process fixtures excluded). Each platform passed60frontend tests and
npm audit. Inventories:352Mac,355Windows,442Linux notices.

Post-download Mac verification caught a linker-only ad-hoc signature without a
sealed bundle/resource envelope: strict codesign failed. Do not promote beta.3
Mac as verified. Beta.4 explicitly configures Tauri signingIdentity="-" and adds a
read-only mounted-DMG signature/version/notices gate before CI artifact upload.
No signing account, credential or notarization is used. Old assets remain frozen;
release notes will point away from the beta.3 Mac download when beta.4 publishes.

## Public beta.4 — 2026-11-05

Published tag v0.2.0-beta.4 at f0cd98eedb69f9c7133c53c3d5422932b677022c;
CI run42084594499 passed all platform checks and published DMG/NSIS/DEB plus
SHA256SUMS. All three anonymously downloaded installers match their published
hashes. The actual mounted DMG passes strict deep codesign, version, architecture,
notices and Applications-shortcut checks. Mac signing is ad-hoc, not Developer ID
or notarization. Windows is unsigned. Windows/Linux GUI installation and a second
Mac's downloaded first launch remain unverified. Beta.3 release notes now flag its
Mac installer as superseded; frozen assets are unchanged. README includes direct
public downloads and a brief planned-feature roadmap. No live AI inference used.

## Rust-native direction and UI concept — 2026-11-05

The owner requests a Rust application/native UI inspired by the Crafting Apps,
plus deeper daily Git features and a single multi-board restyling image. Read the
new docs/RUST-NATIVE-PLAN.md and ADR0025. egui/eframe is the proposed first spike,
with Iced as an evaluated alternative if editor/accessibility gates fail. Keeping
installed Git initially is explicit; replacing the Git engine is a separate track.
No new framework dependency, application code, storage migration or native build
is implemented in this design/research task. The released beta.4 remains unchanged.

Research inspected official PhotoCraft, egui, Iced, gix, GitUI, Lazygit, Gittyup and
GitButler sources. These are documented references, not hands-on competitor tests.
GitButler's current README describes Fair Source restrictions; no code/assets are
reused. The generated six-workflow board uses the actual synthetic Ramo screenshot
and previously selected mock as references. Original selected mocks are preserved.
The output is1536x1024, not the requested4096x2816. Prompt, provenance, concept vs
implemented workflow mapping and visual inaccuracies are documented under
`docs/design/rust-native/`. Generated code/topology is not a correctness fixture.

README/roadmap link the proposed direction; the feature table now accurately
includes delivered amend and remote URL add/edit. Only documentation and a visual
asset changed. Relative documentation links, release metadata and whitespace are
checked; no functional suite is rerun or claimed for this documentation-only task.
No Codex/Claude/Ollama application inference, installer or release is produced by
this task. The requested visual uses the built-in image-generation tool.

## Native implementation — 2026-11-05

Implemented `apps/native` version0.3.0-dev.1: real egui/wgpu application consuming
our existing Rust core without Tauri. Actual refs/history/diffs, worker dispatch,
stale-response guards, virtual22px/30px history and a two-source/result editor
with single-line picks and explicit, separate local drafts. This is R0 plus an
R1 feasibility slice; released beta.4 and its installers remain unchanged.

Local macOS arm64 evidence:14 native tests,127 core tests, native clippy/fmt and
notice collection passed. Actual native GUI loaded5011 synthetic commits with32
visible compact rows; keyboard navigation, search, source line selection,
undo/redo, Unicode paste and draft save/restart/restore were exercised.
See [native verification](docs/NATIVE-VERIFICATION.md) and [design QA](design-qa.md).
Git entries, HEAD, status and working bytes were preserved. The history fixture's
binary index hash changed (stat/cache metadata); the conflicted fixture index
remained byte-identical. Do not describe both binary indexes as unchanged.

A separate ad-hoc-signed local development bundle is available through
`scripts/run-native-macos.sh`. Native CI is configured for macOS/Windows/Linux;
its hosted results are separate from local GUI evidence. No native installer,
release tag, live provider inference or GitKraken speed comparison is produced.

## Native daily write workflows — 2026-11-05

Partial R2 now exposes typed existing-core operations: file staging/unstaging,
index commit, local double-click checkout, create/merge/delete branch, fetch,
fast-forward pull, push, and reviewed stash save/apply/pop/drop. Branch and stash
context menus share the same dispatcher as toolbar/Changes. Trust is per opening;
explicit reopen resets it. No credentials are imported and no live AI is run.

Native merge editor now offers reviewed Save and stage using its original
fingerprint, then Complete merge/Abort in Changes. A real GUI test found and fixed
unchanged-tree merge completion: accepting current content can produce no staged
diff but must still create a two-parent merge commit. Native GUI verified that
exact case, a double-click checkout without another trust request, and file
stage/unstage.16 native tests and clippy passed locally. Original14-test native
CI passed on macOS/Windows/Linux; these new changes need their own hosted run.

No loss of unstaged bytes in the new commit preservation fixture; failed merge
returns current conflicts, and resolve/commit completes with untracked data
preserved. UI blocks mutation-time editor/composer input and concurrent actions.
Network adapters are wired through the existing core but live hosted credentials
and native progress/cancellation remain unverified. No replacement installer.

## Native precise staging — 2026-11-05

The native diff now exposes whole-hunk and individual added/deleted-row staging
and unstaging, plus Shift range selection. Original/new file line numbers use a
fixed22px gutter. Accessibility selection state and parent-disabled state were
checked in the actual application; the initial missing check glyph and oversized
button padding were corrected before capture. Working/Staged labels remain
explicit, and mutation refresh retains the appropriate selected file/version.

Core `Action::Lines` accepts row IDs and displayed diff hash, not a renderer patch.
Fresh repository snapshot, regular-file scope and complete diff are revalidated.
Unstage derives a reverse-oriented partial patch against the actual index so
other staged changes survive. No automatic fallback or retry changes scope.
Missing EOF uses whole-hunk actions; complex file kinds use whole-file staging.

Evidence:131 core tests (27+94+4 new real-Git selection tests+6 process) and17
native tests passed on macOS arm64. Native clippy/fmt passed; native CI now runs core preservation tests as well.
An additional broad
core all-target clippy audit found pre-existing test IO-amount/style findings in
ai.rs/local_ai.rs/process.rs, not selection failures; those remain lint debt.
Actual GUI verified single-line stage/unstage, Shift range stage, whole-hunk
stage/unstage and correct checkbox off/on state. Independent Git/index/working
byte assertions confirmed the chosen subset and preservation of the second hunk.
See NATIVE-VERIFICATION.md and docs/images/native-line-selection.png. No hosted
credential/inference use, native release or GitKraken performance claim.

## Native composer and object actions — 2026-11-06

Repository-scoped commit-message autosave/recovery now uses revision-checked atomic
local records, separately from merge/beta drafts. Delayed save acknowledgements do
not replace newer typing. Ordinary opening/closing waits for saves; errors retain
in-memory/canonical copies with explicit retry/copy/reload. A successful normal
commit clears the saved message; amend preserves the next-commit composer.

Native commit menus expose copy, branch/tag creation, eligible amend, detached
checkout, cherry-pick and revert. Branch menus add rename/tracking. Supported
cherry-pick/revert sequences expose Continue/Skip/Abort using existing core guards.
Action forms/reviews use modal backdrops and clear primary actions. The selected
workbench theme is explicitly pinned dark after a real light-system startup defect.

Local evidence:22 native tests, native clippy/fmt,266 dependency notices and the
ad-hoc macOS developer bundle passed. Actual GUI synthetic amend preserved the
unstaged hunk and retained a recovery ref. Immediate Unicode paste then Cmd-Q
initially lost unsaved text: the Rust Cocoa menu bridge now routes Quit through
the existing guard, and the same test recovered exact text after restart. A later
real GUI stage/normal commit preserved file bytes, committed the recovered Unicode
message and persisted an empty composer. Modal opening, disabled actions and Escape
were observed in the latest bundle. See [verification](docs/NATIVE-VERIFICATION.md)
and [ADR0026](docs/decisions/0026-native-composer-and-quit.md).

Prior precise-staging revision5ed20e9 passed native/core CI on all three OS in
[run42119232111](https://github.com/peppoasap/ramo/actions/runs/42119232111).
Those hosted results do not cover this new composer/menu slice until its own run.
No new native installer or release; beta.4 remains unchanged. Amend-dialog text
is session-only; main drafts are plaintext/32KiB. Force kill/OS shutdown, actual
Windows/Linux GUI, draft-error/repository-switch GUI paths and dirty merge Cmd-Q
remain acceptance limits. Tests cover storage competition and repository scoping;
these are not substitutes for all platform GUI checks. No live inference.

## Native revision comparison — 2026-11-06

Separate read-only Compare view now accepts two saved commit IDs through explicit
inputs, bounded revision choices and commit/branch menus. Direct/unique-common-
ancestor modes, Swap, effective-base display and per-file diffs reuse the existing
core. Objects are pinned when selected; moving a branch does not retarget a pair.
Endpoint/mode changes clear old results. Comparison and file requests have unique
IDs; invalidated replies/errors are ignored, including after a repository refresh.
Working/index changes are excluded and no write trust is required.

Local evidence:24 native tests and native clippy pass. New worker fixture checks
divergent direct/ancestor results, a moved branch, staged/unstaged preservation and
revision-expression refusal. A state test rejects stale results. Actual macOS GUI
selected endpoints through commit/branch menus and both pickers, inspected a diff,
swapped the direction and checked reversed additions/removals, then inspected
common-ancestor output. Picker dismissal was refined after inspection. See
[verification](docs/NATIVE-VERIFICATION.md); no drag/drop or installer yet.

## Native reviewed graph gestures — 2026-11-06

Commit subjects, displayed graph branch labels and sidebar branches now drag.
Dropping on a commit opens direct immutable-pair comparison automatically; dirty
working files remain separate. Dropping a local branch or non-merge commit onto
the named current branch in the toolbar, or its temporary overlay, opens merge/
cherry-pick review. No automatic checkout, write, reset/rebase or push. Intent
checks bind repository/fingerprint/source refs; existing core trust and fresh
snapshot checks still authorize execution. Dirty/detached/active-operation states
refuse mutation intents. Context menus and Compare retain non-drag routes.

Actual native testing exposed coalesced press/release input: default drag-start
could lose the source or select the destination row. Native gesture handling now
captures raw press origin, waits six logical pixels and resolves drops after the
visible row geometry is collected. Source click is suppressed for a drag; a
plain click remains inspection. The temporary target overlays existing layout,
without shifting virtual rows. Escape, pointer leaving, focus loss, busy and stale
states clear sessions; no external drag deserialization exists.

Local evidence:28 native tests, native clippy/fmt,266 notices, developer bundle
build/signature and release-metadata consistency passed. Tests include real-Git intent/preservation and
headless egui batched-input/cancellation cases. Actual macOS GUI verified commit
comparison, sidebar/graph-label branch comparison, correct merge/cherry-pick
reviews and cancellation without Git changes. A reviewed drag merge then created
the expected two-parent commit on main, preserved original file bytes and left a
clean index/tree. See [verification](docs/NATIVE-VERIFICATION.md) and
[ADR0027](docs/decisions/0027-native-graph-gestures.md). These are synthetic fixtures;
OS-level cancellation, full keyboard/accessibility and Windows/Linux gesture GUI
acceptance remain separate from the headless tests.

The preceding comparison revision0959be3 passed native/core CI on all three OS in
[run42189798196](https://github.com/peppoasap/ramo/actions/runs/42189798196), and
legacy desktop builds passed all three OS in
[run42189798220](https://github.com/peppoasap/ramo/actions/runs/42189798220).
Those results cover composer/comparison source, not these newer gestures.
No native installer, live inference or matched GitKraken performance claim.

## Exact next task

Keep [NATIVE-COMPLETION.md](docs/NATIVE-COMPLETION.md) as the full objective ledger.
Next implement native clone and remote management using existing typed core
operations, explicit destination selection and provider choices. Continue
worktrees/tabs, contextual AI and broader graph/file menus. Preserve existing trust/fingerprint/preservation
semantics and finish editor IME/accessibility/CRLF/linked-scroll/large-file gates,
R3/R4 depth and R5 packaging/cutover. Full native migration remains incomplete.
Observe this slice's hosted runs before claiming cross-platform source checks;
actual Windows/Linux GUI and second-Mac release acceptance still remain.

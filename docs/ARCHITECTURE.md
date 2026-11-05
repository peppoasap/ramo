# Architecture

Implemented locally: Tauri 2, React 19/TypeScript, Rust and the installed Git executable. One application package; no application server or account database. See ADR 0002 for the move from design to functional implementation.

Rust-native development is isolated under [apps/native](../apps/native/README.md).
It consumes this library with `default-features=false`, exposes only typed Git
read operations through a bounded worker queue, and renders through egui/wgpu.
It has no Tauri/WebView runtime and does not replace this released app. Editor
drafts use separate versioned native files and are not Git writes or an automatic
import of legacy WebView records. See ADR0025 and RUST-NATIVE-PLAN.md.

## Ownership

| Module | Responsibility |
| --- | --- |
| `src/App.tsx` | Navigation, async context guards, per-repository drafts and operation review |
| `src/components/` | History, sidebar, inspector, diff, provider settings and accessible dialogs |
| `src/domain.ts`, `bridge.ts` | Typed data and the narrow native invocation boundary |
| `src/graph.ts` | Pure parent topology to lanes/edges; actual Git parents, not decorative log parsing |
| `src-tauri/src/desktop.rs` | Typed Tauri endpoints, blocking worker dispatch, shared operation mutex |
| `src-tauri/src/git.rs` | Registered repository IDs, snapshots, paths, refs, diffs, writes and recents |
| `src-tauri/src/process.rs` | Argument-array process execution, Git environment isolation, limits and process-tree timeout |
| `src-tauri/src/ai.rs` | Bounded immutable input, exclusions, provider adapters and stale-input checks |

The renderer has no generic shell or filesystem plugin. It cannot submit arbitrary Git commands. A native folder picker, recent-repository registration and known worktrees establish repository scope. Preferences contain local paths and provider/model choices, never provider tokens.

## Git contract

Status uses NUL-separated porcelain v1; untracked files are enumerated individually. History uses explicit object IDs and parents, paginated in batches of 300. Worktrees use NUL porcelain where available and a guarded compatibility fallback for Git 2.33.1. Non-UTF-8 filenames and ambiguous legacy worktree paths fail explicitly.

Writes and network actions require session trust. Git hooks, filters and credential helpers remain Git's behavior for trusted repositories. Inspection disables external diff, textconv and fsmonitor. Paths use literal pathspecs and validated relative components; subprocesses receive argument arrays and a cleaned Git context environment. Executables resolve only from absolute search folders; Unix executable permissions are checked, and missing programs fail without repository-local search.

Every mutation checks the reviewed snapshot fingerprint, including branch identity and upstream even when object IDs match. The backend serializes operations across open repositories/worktrees; external processes can still race, so this is optimistic validation, not an atomic filesystem transaction. Refresh after an error before retrying. Push supplies an explicit upstream remote/refspec for the active branch, independent of `push.default`.

Buffered process streams are each limited to 16 MiB, previews and AI input have smaller caps, and timeouts kill process groups on Unix or task trees on Windows. A timed-out write may already have changed Git state; never automatically retry it. Explicit Stop for running AI requests cancels native request tokens independently of the Git mutex, terminates AI process groups/trees and drops pending local HTTP work. It does not authorize cancellation of Git writes or reverse provider usage.

## Verification and limits

Real Git fixtures cover unborn repositories, staged-only commits, renames, hostile pathspecs, topology, linked worktrees, detached HEAD, stale snapshots and local bare-remote operations. Process tests cover output limits and descendants retaining output pipes. Native macOS UI acceptance uses only synthetic repositories. Hosted Windows/Linux builds and backend tests pass for beta.4; see [build verification](BUILD-VERIFICATION.md). Windows/Linux GUI installation acceptance, older macOS versions and public publisher signing remain unverified. Large-repository performance evidence is scoped in STATUS.md and does not establish a GitKraken speed comparison.

Backend process polling is adaptive for short commands; measured local results and reproducible fixture commands are in `verification/PERFORMANCE.md`. This is not a measured UI or cross-platform speed claim.

Review fingerprints consume the complete staged/unstaged binary Git patches through an incremental SHA-256 reader (8 KiB chunks), rather than buffering them. This internal digest path has no byte-output cap, retains the process deadline and bounds stderr at 16 MiB. It does not send patch bytes to the renderer or providers. Ordinary command output and previews keep their existing size caps. Read errors fail the operation instead of accepting partial output. Git itself may still use memory and CPU to produce large binary patches; this change is not a claim of constant total process memory.

Provider CLI selection is native-only. `provider_settings.rs` atomically persists a non-secret executable path; `desktop.rs` never accepts an arbitrary program path from the renderer. The Codex adapter pins the verified CLI version and shares its argument builder with the offline zero-tools request probe. One authorized live synthetic draft succeeded; see `AI.md` for the exact scope.

Commit text recovery is owned by `src/drafts.ts`: synchronous local WebView storage writes keyed by canonical repository/worktree ID, limited to 32,768 characters. It stores neither diff input nor provider credentials. App state retains typed text on storage failure and shows a persistent composer warning. A successful commit clears only its submitted draft; newer in-memory or saved text is retained. Failed commits never clear it. Native acceptance verifies a rejected commit, normal restart recovery, successful commit and empty composer after another restart. This does not establish OS-crash durability or transactional coordination between simultaneously editing windows.

Unmerged status codes are separated from ordinary staged/unstaged files. MarkResolved is a distinct typed operation protected by trust and the reviewed fingerprint; ordinary stage/unstage rejects conflicts. MERGE_HEAD/CHERRY_PICK_HEAD/REVERT_HEAD contents and operation state participate in that fingerprint, including a no-tree-change merge. An existing merge may finish with no staged tree change; other sequencer operations require their own Git continue commands and are never treated as ordinary commits. Combined conflict diffs preserve Git text without inventing two-sided line numbers.

Commit file lists use an explicit first-parent object comparison, matching selected file diffs. Do not rely on the interaction of `diff-tree -m --first-parent` on older Git versions: it may list changes against multiple parents. An unchanged-tree merge regression verifies an empty file list and empty first-parent file diff.

Merge is a typed local-branch operation: trust, reviewed snapshot, clean worktree and named local target are checked before invocation. Explicit `refs/heads/...` avoids tag ambiguity. Git receives explicit fast-forward/commit/no-squash/no-autostash options and retains trusted repository hooks/drivers. Conflicts leave Git state intact and open Changes. History-changing actions force one history reload even after the post-operation snapshot was already installed; staging-only actions do not.

History branch labels use snapshot OID equality; they do not infer topology from decorated refs. `repositoryMenus.ts` owns the shared branch/commit menu policy; `ContextMenu.tsx` handles pointer/keyboard interaction. New typed operations create refs at full validated commits, rename without force, checkout tracking refs, detach, cherry-pick and revert. Dirty-work and sequencer gates protect operations that move or apply history; merge-parent cherry-pick/revert require a future explicit mainline flow.

`ConflictResolver.tsx` is a full-space source/result workbench using `CodeEditor.tsx` (CodeMirror 6). The UI edits a buffer until reviewed save-and-stage. Backend `conflict_file` reads bounded index stages and working contents; `ResolveFile` verifies the original snapshot and file boundary again, rejects unresolved markers, then atomically replaces the text file before explicit staging. Staging errors can leave the written resolution on disk; refresh and review before retrying. `AbortMerge` invokes Git's guarded abort. AI explain/proposal inputs contain only one selected conflict's bounded on-disk versions; proposal insertion is separate from save and rejects a changed editor buffer.

CodeMirror source gutters expose individual line picks through accessible + buttons and primary-button clicks on line numbers. A visible hint and permanently displayed disabled-until-selected action make this discoverable. Picking is disabled while saving; read-only source text remains available for ordinary text selection. Selection is separate from mutation: a reviewed source selection replaces only the active conflict, or inserts at the result cursor when no markers remain. `selectedSourceLines` and `replaceConflictText` preserve source order, duplicate rows, line endings and unrelated edits; result changes remain in editor undo history.

Cherry-pick/revert recovery uses one typed Sequence action with enumerated kind and step (continue/abort/skip). The backend verifies the live operation kind and bounded sequencer instructions before invocation. Continuing refuses unresolved conflicts and tracked unstaged edits, uses the recorded Git message with explicit `--no-edit`, and may advance several commits before another conflict. Skipping and aborting require distinct reviewed confirmations. The reviewed fingerprint includes sequencer todo/head/abort-safety/options, so changing a future sequence step invalidates an open review. Rebase remains outside this contract.

Resolution recovery is split into `mergeDrafts.ts` (validated records, stable scoped keys, storage comparisons) and `useConflictBuffer.ts` (bounded conflict loading, buffer updates and recovery decisions). Result edits are stored synchronously; source versions and AI input are not persisted. The stored review fingerprint prevents silently applying an older result to new Git state. Matching buffers restore into the editor only, and changed-state records require explicit review before Save is enabled. Successful save-and-stage and explicit discard clear only the owned record. Detected competing writes or quota/corruption failures keep existing storage and in-memory text, with a visible warning. Known credential filenames opt out of recovery. These are local WebView records, not encrypted cloud backups or atomic multi-window transactions.

Partial staging is a typed Hunk action, targeting only an ordinary modified tracked text file while no operation/conflicts are active. The renderer sends the selected native hunk index and hash of the complete displayed UTF-8 patch, not a patch body. After trust/fingerprint validation, `git.rs` regenerates the canonical preview and checks its hash; `patch.rs` selects one complete hunk. `git apply --cached` changes only the index, with `--reverse` for unstage and `--whitespace=nowarn` to preserve the submitted text. No reject/force/working-file application is used. Whole-file staging remains available for unsupported file kinds or metadata changes.

Canonical working/staged previews explicitly fix context, algorithm, prefixes and output indicators. Hunk headers therefore retain the same indices under custom Git display settings. The selector rejects patches over 256 KiB, combined/binary/rename/file-mode metadata and missing indices. History views never expose write controls. Staging-only actions refresh status and file diffs without reloading history.

Worktree creation uses a native picker grant rather than renderer-provided absolute paths. `choose_worktree_parent` registers a bounded ephemeral parent token scoped to the repository. `CreateWorktree` validates the token, portable single folder name, parent canonical identity, outside-Git boundary, absent destination and local branch availability, then invokes `git worktree add` without force. New branches start at the reviewed HEAD, ignoring dirty source files; existing branches retain their own tip. Worktree records participate in review fingerprints. Successful creation consumes the token and refreshes the list; source selection remains unchanged. This is optimistic checking against concurrent filesystem changes, not an atomic directory transaction. See ADR 0010.

`conflictNavigation.ts` parses current unresolved blocks into one-based result line ranges and locates exact unique whole-line source matches. CodeMirror adds active-range decorations independently of chosen source lines. Previous/Next follows the result and optionally the sources; Locate explicitly recenters all available ranges. Scrolling does not move selection/caret or rewrite text. Ambiguous/empty/unavailable source fragments stay unlinked and display a visible status; no proportional or fuzzy mapping is used. See ADR 0011.

`stash.rs` owns NUL-delimited object-bearing entries, immutable inspection and reviewed application/removal. Snapshot fingerprints include entry object IDs. Inspection shows the first-parent-to-stash tracked diff and third-parent saved untracked files separately; preview paths must belong to the selected group. Apply targets the reviewed OID. Pop rechecks the selector after successful apply before deleting; failures retain the stash and may leave partial working/index state. Drop does not change working/index files and has a separate UI review. Stash subcommands explicitly disable inherited literal-pathspec mode to avoid observed Apple Git 2.54 untracked cleanup behavior; they accept no renderer pathspec arguments. All actual file-path endpoints retain literal mode. The final selector deletion remains optimistic against external Git processes. See ADR 0012.

`compare.rs` owns immutable revision comparisons. The renderer submits two full validated commit object IDs and an enumerated direct/common-ancestor mode. Common-ancestor resolution refuses zero/multiple bases, and returns the effective base hash. File diffs use that resolved object pair, validate changed-file membership, disable external diff/textconv and retain literal pathspecs. Names are NUL-delimited; renames are explicit delete/add and text previews are capped at 512 KiB. `ComparisonDialog.tsx` owns request-scoped async state and clears old results when the selection changes. Comparison exposes no mutation or AI controls and excludes working/index content. See ADR 0013.

`OperationGuide.tsx` treats unmerged files with no detected operation separately from an actual rebase/unknown sequence. The former can use ordinary reviewed file resolution/staging and commit; the latter retains external continuation guidance. No stash origin is inferred from missing markers. Neither guide rendering nor resolving/committing a file reapplies or consumes a retained stash. Rendered UI checks and a native single-parent commit verify this route; saved untracked contents can remain only in the stash after partial application.

`cloning.rs` owns bounded native folder/source grants and one-use immutable clone plans independently of an opened repository. Prepare validates paths/URLs without network or destination creation. Execute revalidates canonical boundaries and exclusively creates a new directory, then uses the installed Git via the existing bounded process runner. Per-invocation transport config allows file only for local sources and HTTPS/SSH only for typed network URLs; it is not stored in repository config. No hardlinks/local optimization or submodule recursion is requested. Registration opens the result without session write trust. Failures consume the plan and leave any partial output for inspection. `CloneDialog.tsx` renders a separate source/destination approval. These checks are optimistic against external filesystem races; trusted global Git templates/helpers/filters remain Git behavior. See ADR 0014.

`stash_restore.rs` implements the typed RestoreStashFile action. A fresh fingerprint and stash index/OID bind an original saved untracked path; only one regular blob up to 8 MiB is read. HEAD/index leaf paths and tracked parent overlaps are refused. Validated missing parents can be created, without following symlinks or entering nested repositories. A sibling temporary file is persisted without clobbering existing data. Unix permissions are owner-only, preserving the saved executable bit. Restoration preserves saved Git bytes without checkout filters, never changes the index/stash or commits, and is allowed only without an active operation (marker-free conflicts can remain). The UI freezes the selected path during review. See ADR 0015 and stash-file-recovery.json.

`worktree_management.rs` implements canonical known-checkout validation and guarded removal, including a bounded index-to-disk inventory for ignored/extra data that Git's default remove can delete. Primary/open/locked/detached/active-operation checkouts and hidden index/submodule states are refused. Lock/unlock/removal use typed reviewed actions. Worktree HEAD and primary identity participate in fingerprints. `close_repository` unregisters a closed UI tab before it can become removable, retaining drafts/recents but clearing native trust and scoped picker grants. See ADR 0016 for limits and external-process race semantics.

`ai_requests.rs` owns bounded one-use previews and a single active job; `cancellation.rs` supplies its shared token. `process.rs` preserves ordinary Git calls while accepting a token for provider/version-probe calls. `local_ai.rs` uses a temporary current-thread Tokio runtime and async reqwest for cancellable bounded loopback generation. The renderer shares `aiRequest.ts`/`useAiRequest.ts` across ordinary and merge assistance, suppresses late results after Stop and retains busy state until worker completion. Fresh input review is required for another generation. See ADR 0017 for delivery-race and server-side limits.

`historyWindow.ts` owns fixed 48px row ranges, reveal offsets and keyboard targets; `useHistoryWindow.ts` observes scrolling and viewport resizing. `History.tsx` keeps full loaded graph layout and object/branch lookup maps memoized, renders a bounded absolute-position window and retains focused/pending rows. Selection uses stable object identity and repository changes remount the viewport. Search keeps topology intact, with explicit match navigation. Memory/initial graph work still scale with loaded history. See ADR 0018 and the separately scoped static-render/native evidence.

Last-commit amend lives in `amend.rs`, with a separate `amend_message` read endpoint
and typed Amend mutation. It pins HEAD as well as the ordinary reviewed fingerprint,
retains a recovery ref, excludes those refs from ordinary history, preserves author
and commits only the index. It refuses detached/merge/known-remote commits and
active operations/conflicts. `AmendDialog.tsx` keeps its own in-memory text; normal
persistent commit drafts are untouched. See ADR 0021 for remote-ref and race limits.

`repositoryProviders.ts` owns pure clone URL/folder templates. `hosting.rs` supplies
an explicit GitHub catalog through installed gh with fixed read-only API calls and
bounded validated metadata, independent of Git mutation locks. `remotes.rs` lists
sanitized endpoints and adds/changes ordinary remote URLs via a typed trusted action.
Endpoint output enters snapshot fingerprints. See ADR0023/HOSTING.md for unsupported
provider catalogs, special remote configurations and live authentication limits.

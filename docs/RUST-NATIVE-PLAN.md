# Rust-native Ramo: migration and product plan

Proposed 2026-11-05. The owner requested the Rust direction; this document specifies
the next work, not a delivered rewrite. Current release: **0.2.0-beta.4**,
Tauri/React/TypeScript/CodeMirror plus Rust and installed Git. Apache-2.0 remains.

Progress: [apps/native](../apps/native/README.md) implements the first read-only
egui workbench and native editor evaluation. It is0.3.0-dev.1, not a replacement
release. The owner selected the concept with wider layout and high history density;
the development app uses22px compact rows. R0/R1 acceptance gaps remain explicit. Partial R2 now includes guarded daily Git
writes and save/stage/complete merge. See [native completion ledger](NATIVE-COMPLETION.md).

## What to take from the Crafting Apps

[PhotoCraft](https://github.com/storytold/photocraft#the-crafting-apps) describes
independent Rust applications with a data/command engine and thin egui UI. Its
[workspace manifest](https://github.com/storytold/photocraft/blob/main/Cargo.toml)
separates engine and UI. The useful lessons for Ramo are shared command semantics,
headless testing, independent releases and explicit capability coverage. Its own
alpha disclosure distinguishes wired menus from professional workflow acceptance.
Its claims, tests and benchmarks are not evidence about Ramo.

Avoid adopting a large suite framework or many crates just to resemble it. Ramo
must remain maintainable as one of several independent everyday tools.

## Define “pure Rust” honestly

| Destination | Meaning | Proposed sequence |
| --- | --- | --- |
| Rust application | UI, state, graph/diff/merge logic and adapters in Rust; no React, TypeScript, Node build or WebView in the replacement | First migration target |
| Rust Git implementation | Git operations implemented through Rust libraries rather than the installed Git executable | Separate later capability track |
| Platform integrations | OS windowing, GPU, pickers, credentials and optional provider CLIs | External/native boundaries remain documented |

Keeping Git initially preserves established hooks, filters, signing, credentials
and worktree behavior. It does **not** meet the second definition. Evaluate
[gix/gitoxide](https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md)
per operation; its capability inventory is not a drop-in porcelain-compatibility
guarantee. Begin any library experiment with read-only object/history access and
oracle comparisons to Git. No silent engine switching after a mutation fails.

## UI framework decision

| Candidate | Why evaluate it | Required proof for Ramo |
| --- | --- | --- |
| [egui + eframe](https://github.com/emilk/egui) | Rust UI, custom drawing for graph/diff gutters, portable shell; close to the architectural reference | Editor fidelity, accessible focus/semantics, IME, menus, clipboard, DPI, bounded row layout and idle resource use |
| [Iced](https://github.com/iced-rs/iced) | Rust reactive model with explicit state/messages and asynchronous actions | Same editor/graph/desktop gates; compare actual implementation effort |

Recommendation adopted for the first trial: **egui0.33.3**, with a styled workbench rather than default
demo widgets. Iced is an alternative if the spike reveals an unacceptable editor
or accessibility cost. Neither toolkit automatically supplies a GitKraken-class
editor or native platform widgets. Pin versions and audit licenses when the spike
actually introduces dependencies; the development consumer now pins its own versions/lock.

## Reuse versus replacement

| Existing code | Migration treatment |
| --- | --- |
| `src-tauri/src/git.rs`, process/path/patch/worktree/stash/remote modules | Reuse native guards and Git behavior; extract small services only as actual consumers require |
| `ai.rs`, request registry, cancellation, provider settings | Reuse bounded input/Stop behavior; no new live inference required for migration |
| `desktop.rs`, Tauri endpoints/pickers | Replace shell dispatch with Rust UI workers; retain picker grants, registered scope and operation serialization |
| `graph.ts`, `historyWindow.ts`, menu policy and conflict helpers | Port pure algorithms with shared synthetic JSON fixture expectations; preserve parent topology and selection semantics |
| `App.tsx`, dialogs and components | Replace with typed app state/messages and task-specific views; discard stale responses by repository/request generation |
| CodeMirror editor | Replace only after proving edit transactions, line picking, undo/search, Unicode/IME and conflict navigation |
| WebView draft storage | Migrate to versioned native local records with bounded input, atomic replacement and conflict detection; retain old data until verified |
| npm/Tauri packaging | Keep for beta.4; add separate Rust-native packaging after the UI works; preserve bundle identity/settings deliberately |

The current library already compiles without its optional `desktop` feature.
Use that seam first. A later workspace needs at most a core and desktop consumer
initially; introduce a CLI only when it provides a real diagnostic/automation flow.
Do not expose generic shell execution. A future command registry should describe
availability, effect, review requirements and recovery support, not execute text.
UI, contextual menus and keyboard commands must resolve to the same typed action.

## Delivery slices and acceptance

| Slice | Deliverable | Exit evidence |
| --- | --- | --- |
| R0 — read-only foundation | Rust shell opens a synthetic repository and displays branches, parent-derived graph, commit/files/diff; no mutation controls | Existing backend suite stays green; graph fixture parity; stale load/cancel handling; bounded 5k/50k graph rendering; keyboard and DPI checks |
| R1 — editor and drafts | Native editable merge buffer, two sources, single-line selection, conflict navigation, native draft storage | CRLF/LF/missing-final-newline, duplicate/empty/Unicode rows, IME, clipboard, undo/redo, find, restart and competing-draft preservation; no disk writes from selection |
| R2 — daily workflow parity | File/hunk staging, commits/amend, branch actions, merges, sequences, stash/worktrees, clone/remotes and existing AI routes | Run existing preservation fixtures plus native scenarios through all entry points; no lost dirty/index/untracked data or broadened AI input |
| R3 — daily-base completion | Line staging, reviewed discard/recovery, file history/blame, palette and consistent object menus | Tests on real Git fixtures and pointer/keyboard acceptance; each visible action is executable or disabled with a reason |
| R4 — history and remote depth | Reflog/reset, guided then interactive rebase, upstream/pull strategies, progress/activity and watched refresh | Interrupted/restarted operations, published/dirty/diverged states, hosted synthetic credential acceptance |
| R5 — release cutover | Rust-native Mac/Windows/Linux installers and reproducible CI, migration/rollback guide | Download/install/workflow/restart acceptance on actual target machines; signing state recorded; no Node/WebView runtime requirement |

Editor failure at R1 is a framework decision checkpoint, not permission to remove
merge features. New UI cannot ship as a replacement while it regresses beta.4's
existing Git workflows. Keep the old release available throughout.

## Product priorities and new ideas

1. **Precision:** stage/unstage selected lines, file context menus, file history and
   blame. Preserve binary/mode/rename fallbacks and exact worktree bytes.
2. **Recovery:** recoverable discard with retention/storage visibility, reflog and
   reviewed reset, operation-specific recovery points. No universal Undo promise.
3. **History editing:** guided rebase before range/squash/fixup/reorder; show source,
   target and affected commits, protect published history and support abort/restart.
4. **Graph confidence:** Hide/Solo, range comparison, named drag destinations,
   auto-scroll, visible drop intent and keyboard equivalents. Simple checkout
   stays direct; merge/rebase/destructive actions get a concise impact review.
5. **Remote fluency:** upstream editor, hosted credentials, observable progress,
   safe cancellation semantics, tags and signing UX. PR/CI integration comes later.
6. **Operational clarity:** searchable activity showing outcome, affected object
   and any supported recovery; quiet watched refresh that preserves selection,
   drafts and review invalidation. Diagnose failures without leaking credentials.

Additional concepts: **“What will happen?”** source→target previews beside a drop;
**return to task** reopening the same conflict/rebase after interruption; searchable
commands with short explanations and disabled reasons; a repository readiness
summary shown only when authentication/upstream/operation state needs attention.
These are proposals, not more permanent dashboards or automatic Git behavior.

AI stays contextual: commit draft, selected-diff explanation, explicit resolution
proposal, later PR summary. Show the selected provider, exact outgoing scope and
Stop; never auto-stage, resolve, execute shell or push. Future agent control starts
with read-only inspection, then the same scoped preview/apply policy, not a local
unauthenticated arbitrary-command server. Provider sessions remain provider-owned.

## Open-source references researched

These are documented capabilities, not a hands-on benchmark or copied assets.

| Reference | Evidence inspected | Lesson proposed for Ramo |
| --- | --- | --- |
| [GitUI](https://github.com/gitui-org/gitui#features) | Rust terminal client; contextual help, asynchronous work, file/hunk/line actions | Precise staging and discoverable keyboard actions |
| [Lazygit](https://github.com/jesseduffield/lazygit#features) | Line/range staging and interactive rebase workflows | Range selection and a complete continue/abort flow |
| [Gittyup](https://github.com/Murmele/Gittyup) | Graphical history client, GitAhead continuation; Qt/libgit2 dependencies | Familiar history/file/diff structure; not a Rust implementation candidate |
| [GitButler](https://github.com/gitbutlerapp/gitbutler) | Stacked/parallel branches, drag-based commit edits and activity timeline; README describes Fair Source restrictions | Reference for workflow ideas only; do not treat current code as an unrestricted OSS component to reuse |

No third-party client code, branding or screenshots are vendored. Ramo keeps its
own ordinary Git model; stacked/virtual branch semantics are not silently added.

## Benchmark contract

Capture beta.4 and native-candidate versions on the same machine and repository
snapshot. Measure cold/warm launch to interactive graph, branch switch to settled
view, file selection to displayed diff, staging and merge-buffer typing, plus idle
CPU/RSS and frame latency. Use 5k and 50k synthetic histories, merges/many refs,
small and large text/binary changes; record hardware, cache policy and repeated
median/p95 samples. Do not mutate a shared fixture between unmatched runs.

Separate UI latency from Git subprocess/network time. Establish budgets from the
baseline; no guessed “Rust is N times faster” claim. A later GitKraken comparison
requires the same workflows, fixture, machine and measurement method.

## Next task

Continue from the selected [wide history target](design/rust-native/wide-history.png)
and the implemented development consumer. Close R0/R1 platform/editor acceptance,
then port current guarded Git write workflows before adding the daily-base gaps.
Keep beta.4 builds/storage intact. Native code exists now; complete migration and
release cutover do not. See STATUS.md for dated evidence and the exact next slice.

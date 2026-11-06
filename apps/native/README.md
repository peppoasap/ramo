# Ramo Native development workbench

Rust UI and application code, using egui/eframe0.33.3, wgpu and the existing
Rust core without its `desktop` feature. No JavaScript, WebView or Tauri runtime.
Installed Git remains required. This is **0.3.0-dev.1**, an incremental migration
slice, not a replacement release for the functional Tauri beta.4.

## Run

Install Rust1.95+ and Git, then from the repository root:

```sh
cargo run --manifest-path apps/native/Cargo.toml --locked
```

Open through the native directory picker, or pass an explicit repository:

```sh
cargo run --manifest-path apps/native/Cargo.toml --locked -- --repository /path/to/repository
```

`--state-dir /path/to/isolated-state` isolates recents, commit-message drafts and explicit merge drafts
for testing. The default native state directory is separate from the released
app's WebView records. Existing beta drafts are not migrated or removed.

On Linux, install the winit/rfd native prerequisites (the CI records its packages).
The renderer uses Metal on macOS and configured Vulkan/DX12 backends where
available. Native platform build and runtime acceptance are separate evidence.

For a separate local macOS development bundle (not an installer), run
`bash scripts/run-native-macos.sh` with the same optional arguments. It builds,
collects notices and ad-hoc signs `.tools/native-dev/Ramo Native.app`; it neither
installs into Applications nor publishes a release.

## What works in this slice

- Real repository registration, snapshots, branch/worktree navigation and guarded
  worktree creation/management. Compact session tabs, close/overflow and durable recents.
- Clone by HTTPS/SSH URL, provider shortcut or native-picked local source. A
  native-picked parent and immutable one-shot review protect destinations; failed
  clones are not retried or cleaned up automatically. Opens the new checkout
  through the ordinary draft-preserving route, with fresh write trust.
- Explicit GitHub/GitHub Enterprise catalog through installed signed-in `gh`:
  organization scope, page filter and up to20 pages of100 entries. Other providers
  are URL shortcuts, not account connections. No automatic login/install or tokens.
- Inspect sanitized remote endpoints and review ordinary HTTPS/SSH additions and
  edits. Existing trust/fingerprint guards apply; saving never fetches or pushes.
  Local/custom transports, multiple URLs and separate push URLs are protected.
- Actual parent-derived history, initial300-commit pages, explicit/near-bottom
  pagination up to the core's100k loaded-history limit; no fabricated history.
- Virtual row rendering, compact22px or comfortable30px density, adjustable side
  panels and inspector visibility. Default viewport1680×960 logical points,
  minimum1000×680; OS/display limits apply. No forced bitmap scaling.
- Loaded-history search by subject/author/OID, branch-tip inspection, keyboard
  Home/End/PageUp/PageDown/arrows, commit file lists and first-parent diffs.
- Compare two immutable saved commits through the Compare tab, revision choices,
  full commit IDs or commit/branch context menus. Swap direction or compare since
  the unique common ancestor; inspect each changed file independently. No staging
  or trust is required; working/index data is excluded. Choices scan up to
  1,000 branches and1,000 loaded commits, rendering at most100 matches; other
  loaded commits remain selectable through their graph menu
  and explicit full IDs work outside loaded history. Renames appear as delete/add;
  binary changes show Git summaries and text diffs have a512KiB limit.
- Drag a commit, graph branch label or sidebar branch onto a commit to compare
  the saved pair directly. Drop another local branch onto the toolbar's current
  branch name (or its temporary drag target) to review a merge. Drop a non-merge
  commit there to review cherry-pick. No auto-checkout, rebase/reset/push or write
  from the drop itself. Escape, leaving the window, focus loss and changed state
  cancel the gesture. Six logical pixels distinguish dragging from inspection.
  Context menus and the Compare screen retain non-drag alternatives.
- History stays separate from working changes. Staged-only files use the index
  preview; mixed staged/unstaged files currently show the working diff.
- Modified regular text files support stage/unstage of whole hunks or selected
  added/deleted rows, including Shift range selection. Original/new line gutters
  and explicit Working/Staged labels distinguish the two versions. Core-derived
  patches are bound to the displayed diff hash and current repository fingerprint.
  Partial staging is limited to256KiB diffs; missing-final-newline files use whole
  hunks, while binary/new/deleted/renamed/mode-change files use whole-file actions.
- Existing regular-text conflicts open a full-space two-source/result editor.
  Individual source-line picks replace one active conflict or insert at the result
  cursor after conflicts are removed. Source text also supports normal selection.
- Manual result editing, transaction Undo/Redo, find/next, conflict navigation,
  base inspection and explicit local draft save/restore. No file/index write from
  editor typing, source selection or saving a draft.

## Boundaries

Typed Git writes now use the existing trust/fingerprint guards: file stage/unstage,
index commits (including completion of a merge with an unchanged tree), local
branch creation/checkout/merge/deletion, fetch/fast-forward pull/push, and reviewed
stash creation/apply/pop/drop. Double-click checks out a local branch without an
extra review after trust; right-click exposes branch/stash operations. Trust lasts
until explicit tab close or app exit; activation preserves it and it is never
persisted. The opening prompt also offers read-only access. Network calls
have the existing core timeouts; native progress/Stop is still missing.

The editor has a reviewed Save and stage action; typing and local draft save do
not change Git. The original editor fingerprint is checked before resolution.
Git conflicts/failures refresh the authoritative snapshot and preserve the error;
no automatic mutation retry. Commit context menus copy IDs/subjects, create branches/tags at a commit, review
eligible local amend, detached checkout, cherry-pick and revert. Branch menus
include rename and remote tracking. Supported cherry-pick/revert operations expose
Continue, Skip and Abort; conflicts still require resolution first. Amend uses
only the staged index and retains a recovery ref; published/merge commits are
refused. Its dialog text is session-only.

The main commit composer saves repository-scoped messages locally after a350ms
debounce, recovers on reopening, and clears the saved message after a successful
normal commit. Repository switching and ordinary window/Quit closing wait for
pending saves. Competing writers/storage errors preserve copies and offer explicit
retry/copy/reload actions. macOS Cmd-Q routes through the same close guard.
Force kill, OS shutdown and filesystem crash durability are not guaranteed.
Messages are plaintext, limited to32KiB, separate from merge drafts and beta storage.

No native installer is published yet. Worktree management UI and contextual AI routes remain migration tasks.
Clone shows a pending state and120-second core deadline; native Stop/streamed
progress, shallow/sparse options, additional account catalogs and live hosted
clone/authentication acceptance remain open. Remote removal, upstream controls
and richer endpoint configuration are separate unfinished workflows.

Workers serialize the existing backend. Replies are scoped to repository
generation and selection request, and stale pagination fails until refresh.
This discards stale UI results; it does not cancel already-running Git reads.
Git mutation paths/trust rules are not reimplemented in widgets. The UI prevents
concurrent mutation submission and disables the editor/composer while applying.

Drafts are local, unencrypted, versioned and scoped to repository/file/fingerprint.
Restore is explicit; an older fingerprint is identified as an older-state copy.
Writes use a create-new lock, compare-before-write revision and atomic tempfile
replacement. Competing or malformed records are preserved, with a visible error.
An interrupted writer can leave a lock requiring inspection; no automatic stale
lock deletion is attempted. Known sensitive filenames disable storage; this is
not complete secret detection. Ordinary restart testing does not prove crash
durability or multi-window transaction semantics for every filesystem.

This editor is a feasibility slice. Full IME/accessibility/platform acceptance,
typed CRLF editing, very large files, pixel-linked scrolling, syntax-language
support and existing WebView draft import remain gates
before replacing CodeMirror. Byte-preserving transformation tests alone do not
close those native editor gates. See [migration plan](../../docs/RUST-NATIVE-PLAN.md).

## Verify

```sh
cargo test --manifest-path apps/native/Cargo.toml --locked
cargo clippy --manifest-path apps/native/Cargo.toml --locked --all-targets --no-deps -- -D warnings
cargo fmt --manifest-path apps/native/Cargo.toml --check
python3 scripts/native-notices.py
```

Tests include shared legacy topology outputs,50k linear topology, CRLF/Unicode
source selection, unrelated conflict preservation, undo history, draft competition
and real-Git read-only/conflict fixtures. Bundled font/icon licenses are included
by the notice generator; source provenance is under `licenses/native/`.

Observed local evidence and screenshots: [verification](../../docs/NATIVE-VERIFICATION.md).

Native partial controls are disabled for clipped previews (over12,000 lines or
4,096 characters per line) and during an active Git operation, with a visible
whole-file fallback reason. Complete working files are never rewritten by staging.

## Visual fidelity

The native UI follows the [selected visual contract](../../docs/design/rust-native/VISUAL-CONTRACT.md).
The [current actual capture](../../docs/design/rust-native/visual-fidelity/native-history.png)
and [scoped QA report](../../design-qa.md) separate history refinements from remaining
workflow visual gates. Font files retain OFL-1.1; the license is included in
native bundle notices.

Changes uses unstaged/staged files, a selectable center diff and a right commit
composer. The merge editor has a guarded conflict-file sidebar, current/incoming
sources and editable result. Actual [Changes](../../docs/design/rust-native/workflow-fidelity/native-changes.png)
and [merge](../../docs/design/rust-native/workflow-fidelity/native-merge.png) captures
use synthetic Git fixtures; remaining editor/AI/platform gates stay explicit.

The [refined comparison view](../../docs/design/rust-native/comparison-fidelity/native-comparison.png)
uses searchable named selectors pinned to commit objects, with an optional full-ID
input. Direct/common-ancestor comparison auto-opens the first diff and keeps
working/staged changes separate. Loaded choices and rendered matches are bounded.

Worktree navigation now uses actual linked checkouts. Use the sidebar + to create
a checkout from a new or available existing branch; right-click for lock/unlock
and guarded removal. Parent folders are selected through the native picker. Git
files remain under the shared core's checks; the renderer holds no shell authority.
Commit drafts are saved before switching. Open checkouts now remain in session
tabs, with a width-aware overflow menu.
Close a tab to end that checkout's session trust. The Open menu includes durable
recent repositories; native trust is asked once on opening, with read-only access.
Accepting trust never executes a queued mutation. In-memory view intent is
restored using fresh core reads; automatic workspace restoration after restart
remains pending.
See [verification](../../docs/NATIVE-VERIFICATION.md).

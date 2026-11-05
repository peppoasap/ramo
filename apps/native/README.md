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

`--state-dir /path/to/isolated-state` isolates recents and explicit merge drafts
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

- Real repository registration, snapshot, branches and linked-worktree listing.
- Actual parent-derived history, initial300-commit pages, explicit/near-bottom
  pagination up to the core's100k loaded-history limit; no fabricated history.
- Virtual row rendering, compact22px or comfortable30px density, adjustable side
  panels and inspector visibility. Default viewport1680×960 logical points,
  minimum1000×680; OS/display limits apply. No forced bitmap scaling.
- Loaded-history search by subject/author/OID, branch-tip inspection, keyboard
  Home/End/PageUp/PageDown/arrows, commit file lists and first-parent diffs.
- History stays separate from working changes. Staged-only files use the index
  preview; mixed staged/unstaged files currently show the working diff.
- Existing regular-text conflicts open a full-space two-source/result editor.
  Individual source-line picks replace one active conflict or insert at the result
  cursor after conflicts are removed. Source text also supports normal selection.
- Manual result editing, transaction Undo/Redo, find/next, conflict navigation,
  base inspection and explicit local draft save/restore. No file/index write from
  editor typing, source selection or saving a draft.

## Boundaries

Git writes/network/provider inference are **not exposed** by this native slice.
Fetch/Pull/Push are visibly disabled with a reason. The released beta retains
staging/commit/checkout/merge completion/stash/worktree/AI functionality while the
native migration proceeds. No native installer is published by this workflow.

Workers serialize the existing backend. Replies are scoped to repository
generation and selection request, and stale pagination fails until refresh.
This discards stale UI results; it does not cancel already-running Git reads.
Git mutation paths/trust rules are not reimplemented in widgets.

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
support, save/stage/complete merge and existing WebView draft import remain gates
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

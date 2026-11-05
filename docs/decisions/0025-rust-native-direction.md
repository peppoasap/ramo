# ADR0025 — Rust application and native UI direction

Date: 2026-11-05. Status: user-requested direction; migration design proposed.

The owner requests a pure-Rust direction inspired by the Crafting Apps, plus a
more complete, understandable Git client. The released beta.4 remains Tauri,
React/TypeScript, CodeMirror and a Rust backend invoking installed Git.

Propose a Rust desktop UI using egui/eframe, retaining the existing Rust Git/AI
contracts. Evaluate Iced if the editor/accessibility/layout spike fails. This is
not a completed framework selection or an implemented native replacement.

Define the first destination precisely: application UI, state and domain logic
written in Rust, no JavaScript runtime or WebView. Installed Git, provider CLIs
and platform system libraries remain explicit dependencies. This is not an
end-to-end pure-Rust Git implementation. A gix-based engine is a separate,
capability-tested track; git2/libgit2 would not meet that stricter goal.

Reuse the core without its optional `desktop` feature before extracting packages.
Keep command policy, trust, fresh fingerprints, process limits, cancellation and
work preservation authoritative in Rust. Move the remaining frontend algorithms
and draft persistence deliberately; do not recreate business rules in widgets.
Do not introduce a portfolio-wide framework or copy PhotoCraft's crate count.

Use a read-only native graph and a working editor spike to decide feasibility.
Native editor gates include individual-line picking, normal text selection, IME,
Unicode, CRLF/EOF, undo/redo, find, linked navigation, keyboard/accessibility and
draft recovery. A multiline text field alone does not replace CodeMirror.

Keep beta.4 tags/assets immutable and usable during migration. Cut over only after
the native app reaches existing workflow coverage and downloaded-platform checks.
No speed advantage follows from changing language; compare matched end-to-end
workflows before making a claim. See [migration plan](../RUST-NATIVE-PLAN.md).

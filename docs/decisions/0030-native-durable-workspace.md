# ADR0030 — Durable native navigation intent

Date: 2026-11-06. Status: implemented; native release/platform acceptance pending.
Extends ADR0029 with restart persistence while keeping its trust boundary.

Persist a version1 document with at most20 tab identities/canonical paths, active
tab, mode, filters, commit/file selection, immutable comparison intent and compact/
inspector preferences. Validate identity/path correspondence, relative file paths,
object IDs, uniqueness, field lengths, active references and a256KiB total bound.
Unknown fields/versions are rejected. Never serialize trust, snapshots, diffs,
operation plans, credentials or editor/composer buffers in this document.

Reuse the existing atomic local record writer under the native state's workspace/
subdirectory, with revision CAS and lock. Debounce navigation changes by500ms;
a late save acknowledgement updates only the stored revision/document, never the
current UI. Normal quit flushes changes after the existing composer/editor guards.
A save conflict keeps disk and UI copies, offers retry, and permits explicit quit
without saving workspace. Corrupt/future records are preserved, with persistence
paused; do not silently reset them. Existing drafts remain separately scoped.

At startup register saved checkouts read-only, retaining unavailable tabs with
retry/close actions and a fallback active checkout. Reject paths resolving to a
different Git root. Opened registrations protect linked checkouts from removal.
Restore active previews through fresh core reads; ask trust on first activation,
never from stored authority. A --repository path adds/activates a checkout within
the20-tab bound without dropping saved tabs. Explicit tab close persists removal.

History starts with the ordinary first page; an older selected object is fetched
for its preview even if locating its graph node needs more pages. Pixel pane widths
and window geometry are not part of this navigation schema. No automatic schema
migration, corrupt-record replacement, native AI or installer cutover is implied.

Visual QA exposed missing Japanese glyphs in synthetic drafts. Append an optional
installed CJK font to existing font families after validating its format/coverage
with ab_glyph0.2.32, already locked transitively. No crate versions or bundled font
assets changed. Standard macOS/Windows/Linux paths are probed; no font download or
system-font redistribution. macOS rendering is verified; other-platform font
availability/IME/complex-script support remain separate acceptance gates.

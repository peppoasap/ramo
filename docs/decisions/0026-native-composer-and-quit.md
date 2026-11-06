# ADR0026 — Native commit drafts and guarded macOS Quit

Status: accepted implementation within the owner-authorized native migration.
Date: 2026-11-06.

The main composer must recover local typing without writing Git or silently
replacing another writer's draft. Reuse native versioned atomic/CAS draft IO in a
separate commit-drafts directory; bind by repository, not branch. Debounce350ms,
cap32KiB, acknowledge saved revisions without replacing newer text. Flush before
ordinary opening/closing; errors preserve copies and require explicit recovery.
Clear after successful normal commit, preserve during amend. Amend form text is
session-only. No migration/removal of existing beta or merge records.

A real paste/immediate Cmd-Q test showed eframe/winit's Mac Quit route bypasses
cancellable viewport close. A tiny macOS-only Rust objc2 menu target redirects
this app's terminate selector into the existing guard. It retains its target for
the application lifetime and does not replace the application delegate or OS
settings. Default termination plus a debounce was rejected because it lost text;
a separate persistence path at termination would duplicate worker/CAS semantics.

The post-fix GUI recovered exact Unicode text after immediate Quit. Storage and
commit preservation tests remain independent evidence. Plaintext drafts, interrupted
writer locks, force termination and OS shutdown are explicit limits. Other OS GUI
acceptance remains pending; no cross-platform shutdown guarantee is inferred.

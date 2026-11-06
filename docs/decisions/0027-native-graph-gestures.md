# ADR0027 — Native graph intent and source capture

Status: accepted implementation within owner-authorized native migration.
Date: 2026-11-06. Extends ADR0019 semantics to egui; beta behavior is unchanged.

Use internal typed sources bound to repository/fingerprint and saved object ID.
A commit target means direct saved-tree comparison, never mutation. The named
current branch in the toolbar, plus a temporary non-layout-shifting overlay,
means reviewed local-branch merge or single non-merge commit cherry-pick. Existing
core trust and freshness checks still govern execution. Dirty/detached/busy or
interrupted states refuse mutations; comparisons exclude local uncommitted data.
No external drag payload parser or new command-execution surface is introduced.

Actual native trials showed press/move/release can be coalesced into one input
batch. Relying only on widget drag_started/press_origin lost the source or used
the destination row. Capture the raw primary-press position, bind the source at
that position, use a six-logical-pixel threshold and resolve release after visible
row geometry is collected. Keep a pending candidate for events across frames.
Suppress the click after a drag; retain ordinary click/double-click semantics.
Escape, pointer gone, focus loss and changed/busy state cancel the session.

Native tests reproduce coalesced events and cancellation, while actual macOS
trials prove branch/commit comparison and reviewed merge/cancel workflows. These
are not Windows/Linux input, full keyboard/accessibility or performance acceptance.
No multi-object drag, graph auto-scroll or arbitrary branch checkout/rebase/reset/
push is added. Existing menus/Compare remain non-drag alternatives.

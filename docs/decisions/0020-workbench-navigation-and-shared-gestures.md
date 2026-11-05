# 0020 — Inspection-first navigation and shared graph gestures

Accepted, 2026-11-05.

Keep the selected graphite/mint three-column workbench and all native Git action
contracts. Sidebar branch clicks inspect tips; mutation commands live behind the
row's ellipsis/right-click or the explicit current-branch switcher. A single row
menu replaces competing merge/delete hover buttons. History and Changes remain
inspection modes; Compare revisions is a secondary action rather than a third
mode. Empty remote/stash groups start collapsed, groups are collapsible and a
branch-name filter reduces navigation noise. Settings retain one toolbar entry.

The existing pointer-capture gesture session is owned by App and passed through
an explicit GraphGesture interface to History and Sidebar. The standalone History
component keeps a local fallback for independent fixtures. Sidebar and graph
sources share one fingerprint, source object, click-suppression state and target
policy. Dropping on a commit opens a pinned comparison; dropping on the current
local branch reviews merge/cherry-pick. Nothing checks out, merges or pushes just
because a row was clicked or dragged. Existing native trust/freshness guards apply.

Only threshold crossing updates the drag session UI; pointer movement within the
same target does not rebuild the entire workbench. Repository changes, busy/AI
state changes, Escape, focus loss and pointer cancellation terminate the gesture.
Do not implement a second sidebar-only command executor or accept external drop
payloads as authority. Keyboard context menus and the branch switcher provide
alternatives. Arbitrary branch targets, ranges and auto-scroll remain separate.

The frozen 0.1.0 tester ZIP is immutable. New local development builds and evidence
must not silently replace that distributed preview or claim new platform support.

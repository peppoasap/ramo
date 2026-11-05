# ADR 0022 — Direct checkout and trust on opening

Status: accepted by user request, 2026-11-05. Updates ADR0020's reviewed checkout UX.

Single click inspects a branch, double click invokes Checkout in sidebar/graph.
The shared branch menu policy also serves the selector/context menu. Local Switch
runs without another confirmation. Native trust, snapshot freshness, operation
and Git overwrite guards remain authoritative. Remote checkout needs naming/tracking
input. Merge/delete/history rewrite retain explicit reviews. No automatic force,
discard or stash is inferred from checkout intent. Keyboard context menus remain.

Ask trust at repository opening instead of interrupting its first write. Remember
that this open checkout was prompted in renderer memory; actual authorization stays
in native memory. Declining keeps read-only access and does not re-prompt on every
tab activation. Explicit writes can ask again. Closing the repository clears both
records; app restart resets them. Separate worktrees retain separate authority.
No persisted trust and no queued write is executed automatically on acceptance.

This reduces repetitive UI confirmation without weakening native data-preservation
checks or broadening provider/AI access. Frozen release artifacts remain snapshots
until another candidate is deliberately packaged.

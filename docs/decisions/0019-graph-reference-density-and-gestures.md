# 0019 — Compact references and reviewed graph gestures

Accepted, 2026-11-04.

Keep fixed 48px virtual history rows. Show one reference (current local branch first, then other local branches, then remote branches); additional branches/tags use an explicit +N menu. Never add horizontal scrolling inside a row. Existing branch actions remain available from every overflow entry. Merge shortcuts in the sidebar appear on hover/focus.

Use pointer capture with a six-pixel gesture threshold and native text dragging disabled on source controls. Keep the toolbar overlay out of row geometry. Tauri native file-drop handling is disabled. Sources are graph branch labels and commit subjects. An in-memory session captures the repository fingerprint and source object. Pointer hit testing accepts only graph targets; external payloads cannot create a session. Escape, focus loss and pointer cancellation clear the session. Suppress the click following a drag so it cannot also open a branch menu. The current branch label also accepts reviewed mutations. A commit row means immutable SHA comparison; a temporary, named current-branch target means reviewed local-branch merge or single non-merge commit cherry-pick. No auto-checkout, rebase, reset or push. Existing context menus remain the keyboard alternative.

Changing repository state cancels the session. Busy, dirty, detached and active-operation states refuse mutations. A drop only opens review; existing trust, freshness and native Git guards still govern execution. Comparisons stay read-only and work with dirty files. There is no dependency or new native execution API.

Limits: source dragging is currently from graph labels/subjects, not sidebar entries. Overflow branch entries expose ordinary menus, not additional drag handles. No multi-commit drag, graph auto-scroll or arbitrary branch targets. Windows/Linux runtime acceptance remains pending.

# ADR0029 — Native repository sessions and navigation state

Date: 2026-11-06. Status: implemented locally; native release acceptance pending.
Updates the single-active-checkout native worker; follows ADR0022's trust policy.

Keep registered repositories open while their tabs exist. Canonical core IDs
coalesce repeated opens. Activating a tab reads a fresh snapshot/history from the
shared backend; it never treats a cached Git payload as live. Explicit close
removes the registration and its folder grants. Worktree removal continues to
refuse an open checkout. Separate worktrees have separate session authority.

Native tabs ask trust once on opening. Read-only selection dismisses the prompt
for this open tab, including future activations; an explicit write can ask again.
Trust is stored only in backend memory, until tab close or app exit. Accepting
trust never runs a queued Git mutation: an explicit write requested while untrusted
returns to operation review after acceptance. No AI/provider authority is inferred.

Tab-local view intent contains mode, loaded-history search/selection, changed-file
selection and immutable comparison endpoints. On activation, refetch the selected
file/comparison against the fresh core state. Commit messages retain their existing
atomic/CAS local persistence; wait for saves before active-tab navigation. Dirty
merge results use the shared keep-editing/discard gate for switch or active close.
Closing an inactive tab does not discard the current editor or await its composer.

At most20 tabs are opened through the renderer. A width-dependent tab subset keeps
the active tab visible, with overflow in a menu. Core recent paths are bounded20
and persist through its existing preferences. Recents reopen through the same
core validation and prompt policy. Errors leave the current view intact.

View intents/open tabs remain in memory in this slice; automatic workspace and
view restoration across app restart are not implemented. Persisted recent paths
and commit/merge drafts do not imply persisted trust or a restored workspace.
Native platform GUI and full keyboard/accessibility acceptance remain separate.

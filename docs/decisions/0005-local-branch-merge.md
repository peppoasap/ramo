# ADR 0005: local branch merge workflow

Date: 2026-11-03. Status: accepted for local implementation.

Allow a reviewed local branch to be merged into the active committed local branch.
Require a clean worktree and explicit confirmation; offer the action next to the
source branch. Use its full refs/heads target to avoid same-named tag ambiguity.

Fast-forward when possible, otherwise create a merge commit. Explicit commit,
no-squash and no-autostash options keep that UI promise independent of those Git
configuration defaults. Retain trusted hooks and merge drivers. Do not force push.

A conflict is an actionable Git state, not a discarded operation: refresh and open
Changes, preserving the index and file contents for explicit Mark resolved. The
existing merge-completion flow finishes it. Abort and sequencer continuation remain
outside this slice. Reload history after a history-changing operation, including
when its returned snapshot already matches the next refresh, and select known HEAD.

This extends ADR 0004's existing-merge boundary. It does not establish hosted remote
authentication or Windows/Linux runtime support.

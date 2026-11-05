# 0010: Create worktrees through a native destination grant

Status: accepted locally, 2026-11-03.

A second checkout must not move the current branch or carry its uncommitted files into another task. Ramo uses `git worktree add` with either a new branch at the reviewed HEAD or an unoccupied existing local branch. It never forces checkout or overwrites an existing destination.

The native picker authorizes an existing parent outside Git repositories. The backend records a bounded ephemeral token scoped to the registered repository; the renderer submits only that token and a portable single folder component. Trust, reviewed fingerprint, branch availability and the canonical parent are checked again before Git runs. Successful creation consumes the token. Worktree records participate in snapshot fingerprints, including external occupancy changes with unchanged HEAD/refs.

This is optimistic validation, not a filesystem transaction against hostile concurrent processes. Trusted repository hooks and filters retain Git's behavior. A failed or timed-out command may have created a branch or worktree; the form stays available and instructs the user to inspect current state before retrying. No automatic cleanup or forced retry is performed.

The UI exposes creation from Worktrees and a local branch context menu. Source selection stays unchanged after success; opening the new checkout is explicit. Worktree removal, pruning, locking and moving are separate future flows. Native verification is scoped to macOS Apple Silicon; real-Git fixtures exercise both branch modes and source/index preservation.

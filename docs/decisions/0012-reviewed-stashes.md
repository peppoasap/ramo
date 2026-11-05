# 0012: Inspect and mutate stashes by reviewed object identity

Status: accepted locally, 2026-11-03.

Snapshot stash entries contain their stack index, reflog selector, object ID and subject. The complete object-bearing list participates in review fingerprints. Both native inspection and mutation check index/object equality. Replacement with the same subject or a shifted stack cannot authorize another stash.

Inspection compares the stash working tree to its first parent for combined tracked edits and separately lists/previews the third-parent saved untracked tree. It reads Git objects without checking out files. Diffs are bounded to 512 KiB and retain literal path arguments; binaries stay Git binary summaries. The staging layout is not a separate preview or restored by the default apply flow.

Apply uses the immutable reviewed object and retains its stash entry. Pop applies that same object, then checks the selector still names it before dropping. Any apply failure, including conflicts or existing untracked-file collisions, retains the stash and leaves Git's partially restored files/index available for inspection. No automatic retry or destructive cleanup occurs. Drop is separately reviewed and removes only the selected entry; Ramo has no undo for it. Worktree/index contents are not changed by Drop.

Writes require session trust, a reviewed fingerprint and no active operation/unmerged files. External processes can still race a final selector-based deletion: these are optimistic checks, not an atomic transaction with arbitrary Git processes. An interrupted write may already have applied; refresh and inspect before retrying.

Stash subcommands have no file pathspec arguments. They explicitly disable inherited literal-pathspec mode: a local probe on Apple Git 2.54.0 retained untracked files during `stash push -u` when literal mode was enabled, while Git 2.33.1 cleaned them in both modes. Other Git endpoints retain literal pathspec mode. This exception must not be reused for endpoints accepting renderer-supplied file paths.

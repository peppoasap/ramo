# ADR 0016: Reviewed linked-worktree management

Status: accepted and implemented locally.

Expose lock/unlock and removal from a worktree's management button. Show the exact captured path, branch and HEAD, then require a separate confirmation and normal session trust. Worktree records now include HEAD and primary-checkout identity in snapshot fingerprints. A changed lock, branch or HEAD invalidates review.

Native operations accept only an exact path from the reviewed repository worktree list and revalidate canonical directory, ordinary linked `.git` file, top-level identity and common Git directory. Primary checkouts are refused. Lock/unlock use installed Git without force.

Removal additionally refuses registered/open repository tabs, locks, detached checkouts, active Git operations, dirty/staged/untracked/ignored work, submodules and hidden index flags. A bounded disk inventory checks every entry against the index and expected directories, refusing extra empty directories, nested metadata and special files without following directory symlinks. Git's default removal alone does not protect ignored data. Only after these checks does `git worktree remove -- <path>` run, without force, prune or branch deletion. Branches/commits remain. There is no automatic retry or in-app undo.

Tab closure unregisters its native repository authority and worktree-picker grants before removing the UI tab. Its local drafts and recent entry remain. Reopening registers it untrusted again, so closed tabs no longer block removal indefinitely.

Validation is optimistic against external Git/filesystem processes, not an atomic filesystem transaction. A file created or edited during the final check/remove interval can still race Git cleanup; stop other processes using a checkout before confirming removal. Non-UTF-8 paths, more than 100,000 disk entries and unsupported file/index states fail closed. Worktree move/prune and detached-commit recovery remain separate features. Native runtime evidence is macOS only.

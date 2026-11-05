# ADR 0004: explicit conflict resolution boundary

Date: 2026-11-03. Status: accepted for local implementation.

An unmerged index must not look like ordinary staged/unstaged files. Keep a
separate conflict group and a distinct MarkResolved operation after the user
edits a file externally. Ordinary stage/unstage refuses unmerged paths. Ramo
never chooses a side or rewrites conflict contents automatically.

Allow staging/inspection/fetch while an operation is pending, but block unrelated
branch, stash and synchronization actions. Finish existing merges with an explicit
commit message, including resolutions whose tree equals HEAD. Do not substitute
ordinary commits for rebase/cherry-pick/revert continuation; those continue through
Git until a dedicated verified workflow exists.

Include operation identity/target metadata in the reviewed fingerprint. List
merge files by explicit first-parent comparison, matching file diffs across Git
versions. Preserve combined conflict diff text without invented two-sided line
numbers. Evidence is in docs/verification/conflicts.json. Merge start/abort and an
embedded conflict editor remain outside this implemented slice.

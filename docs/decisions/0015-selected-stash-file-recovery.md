# 0015: Recover a missing saved untracked file without reapplying a stash

Status: accepted locally, 2026-11-04.

A failed stash apply can leave conflicts while some saved untracked files remain only in the stash. Inspecting a stash now offers a separate reviewed Restore selected file action. The request binds the stash stack index/object ID and one saved untracked path to the current repository fingerprint. It never supplies replacement bytes from the renderer.

The backend reads one regular blob from the stash's third parent, capped at 8 MiB. It preserves those saved Git bytes, including binary data and line endings; it does not run checkout/smudge/encoding filters. Symlinks and submodules require external recovery. Files tracked in HEAD/index, overlapping tracked parent files, traversal, Git metadata, existing destinations, symlink parents and nested repositories are refused. Missing ordinary parent directories may be created. Existing tracked directories can be shared without overwriting their files.

A sibling temporary file is persisted with no-clobber semantics. Unix permissions are owner-only 0600, or 0700 for saved executable files. No overwrite, staging, stash drop, commit or automatic reapply occurs. Other conflicts and the index remain unchanged. Recovery is allowed during marker-free conflicts but blocked by any active Git operation. The review explains these boundaries and holds the selected path fixed until confirmation.

Checks are optimistic against concurrent external filesystem/Git changes, not a directory transaction; an error can leave newly created directories. Ignored restored files may not appear in Changes. Files requiring checkout filters are raw saved blobs, not expanded LFS/smudge output.

Git integration tests are limited to four concurrent workers by the normal npm/CI command. An initial unrestricted run timed out an existing 20 MiB binary fingerprint case during concurrent compilation; the application process deadline was not increased. After fixing a fixture's assumption about whether Git removes an empty saved directory, the complete four-worker suite passed. See stash-file-recovery.json for exact evidence and limits.

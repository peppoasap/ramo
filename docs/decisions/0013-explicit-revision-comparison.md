# 0013: Compare explicit saved object pairs

Status: accepted locally, 2026-11-03.

Comparison is a separate read-only view reachable from branch/commit menus and the history toolbar. Branch choices resolve to their snapshot object IDs. The renderer can also submit full commit hashes outside loaded history. Native endpoints validate commit objects and never accept arbitrary Git revision expressions or commands.

Direct mode compares the two saved trees in the displayed direction. Common-ancestor mode compares the target tree to the unique merge base of the two selected objects. Unrelated histories and multiple merge bases fail explicitly and offer direct comparison instead of guessing. The effective base and target hashes remain visible. Selecting another pair clears old results before a new request; async responses are scoped to that request. File requests use the already resolved effective pair.

Files use NUL-delimited Git name/status output and literal pathspecs. Each diff request validates membership in that pair's changed-file list. Renames are intentionally represented as deletion plus addition in both list and diff. Binary changes retain Git summaries; text previews exceeding 512 KiB fail clearly rather than displaying a silently incomplete patch.

The view excludes working/index changes, exposes no staging/AI/mutation controls and requires no write trust. Moving a branch does not retarget a displayed immutable pair. It can therefore be used while the current checkout is dirty or interrupted. This is saved-tree comparison, not a merge preview, a comparison of uncommitted files or a guarantee of mergeability.

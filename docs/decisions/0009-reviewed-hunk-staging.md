# 0009 — Reviewed whole-hunk staging

Status: accepted, local implementation (2026-11-03).

Expose Stage hunk and Unstage hunk for ordinary modified tracked text. Keep whole-file actions for new/deleted/renamed files, binary data, links/submodules, mode changes and larger patches. Individual line staging remains separate future work.

The renderer supplies a file path, hunk index, direction and SHA-256 of the complete displayed patch. It never supplies an arbitrary patch body. After trust and snapshot validation, the backend regenerates the canonical native diff, verifies its digest and selects the indexed complete hunk. Apply only to Git's index with `git apply --cached`, reversing for unstage. Do not use reject, force, whitespace correction or working-file writes.

Working/staged previews use fixed Myers/three-line boundaries, no inter-hunk context, literal prefixes/indicators and no rename detection. This keeps display and backend selection consistent despite user diff display preferences. History diffs remain read-only. A changed repository or displayed patch fails before index mutation.

The selector bounds the complete patch at 256 KiB and refuses combined/binary/mode/rename metadata. Backend checks ordinary status and regular index modes. Native Git still validates applicability; a failure refreshes the view. Tests compare exact working bytes and index content, including CRLF, missing final newline, literal names and linked worktree isolation.

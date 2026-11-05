# 0011: Navigate merge conflicts with explicit, conservative source ranges

Status: accepted locally, 2026-11-03.

The active unresolved block receives line highlighting in the editable result. Source panes highlight the corresponding whole lines only when the block text has exactly one full-line match in that source. No fuzzy or proportional alignment is used. Absent, empty, edited or repeated fragments can have no unique source location; show this explicitly and leave that pane's position unchanged. Diff3 blocks can also locate base text.

Previous/Next moves the result to the selected conflict. Follow active conflict is enabled by default for source panes and can be disabled for independent reading. Locate active conflict explicitly recenters the available ranges. Navigation scrolls without moving the caret or changing editor selection. Source-line picks are cleared on conflict changes, preventing reuse on another block.

The pure `conflictNavigation.ts` module computes one-based line ranges, preserving CRLF, Unicode and missing EOF newline. CodeMirror converts those line numbers to its own document positions. This avoids mixing raw-file CRLF offsets with normalized editor offsets.

This is linked conflict navigation, not pixel-locked scrolling or a complete semantic diff alignment engine. Manual scrolling remains independent. Neither navigation nor highlighting writes to Git, accepts a resolution or requests inference. Ambiguous source mappings and arbitrary manually edited markers remain visible limitations instead of invented correspondences.

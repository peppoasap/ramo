# 0006 — Explicit history actions and an embedded merge editor

Status: accepted, local implementation (2026-11-03).

Branch tips come from snapshot object IDs rather than parsing decorated log labels. History and sidebar expose the same branch menu; commit actions target validated full object IDs. Existing refs are never overwritten implicitly. History-changing operations refresh the graph.

Use CodeMirror 6 with source panes and an editable result, line numbers, search, undo and on-demand syntax grammars. Load the merge editor separately from the main app. Keep Git execution in the existing Rust backend. A typed ResolveFile action verifies trust, snapshot identity, unresolved membership, regular file boundaries, size and remaining markers. It writes through a sibling temporary file and explicitly stages the result. AbortMerge is a separate reviewed operation. Git filters/hooks remain trusted repository behavior.

AI assistance reuses the existing isolated provider adapters. Prepare an immutable bounded one-file input, preview it, then generate only after an explicit user action. Resolution output is validated structured data. Inserting a proposal only changes the editor buffer; save/stage/commit remain separate. No new agent tools, shell integration or credentials are exposed to the renderer.

Initial scope is text conflicts, including deletion. Binary/symlink/submodule resolution and sequencer continuation remain listed gaps. Unsaved resolution recovery after crash remains open.

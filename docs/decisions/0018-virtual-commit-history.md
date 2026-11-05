# ADR 0018: Bounded history rendering

Status: accepted and implemented locally; macOS native acceptance recorded in `docs/verification/history-virtualization.json`.

Keep all loaded commits and compute their parent-derived graph layout together. Render only viewport rows with eight rows of overscan on each side, plus at most one focused and one pending-focus row. `historyWindow.ts` owns fixed 48-pixel row geometry and keyboard reveal offsets. `useHistoryWindow.ts` measures scrolling/resizing without replacing Git pagination or introducing a virtual-list dependency.

Rows keep their original absolute index and SVG geometry. Search dims nonmatching rows instead of removing graph topology; explicit previous/next match controls reveal matches anywhere in loaded history. Commit/branch context menus retain their existing typed actions. Logical list position/count metadata describes the loaded set. Arrow/Page/Home/End navigation moves selection and focus across mounted windows. Focused commits are retained by object identity during manual scrolling. Selected-commit changes reveal the selected row; appending an older page does not reset selection or return to the top. Repository changes remount history to reset scroll state.

Fixed row height matches existing branch/tag chips and graph SVGs. Multiple labels remain horizontally scrollable; column headings follow horizontal scrolling. Browser scroll anchoring is disabled for the list so appending pages does not follow a relocated Load older button.

DOM work is bounded by viewport size; loaded commit objects and graph layout still use memory/time proportional to loaded history and topology width. This is not a claim of constant total memory, server-side search, arbitrary-history scale or a measured frame-rate improvement. Search covers loaded commits only. Platform/runtime evidence is recorded separately from pure geometry/component-render tests.

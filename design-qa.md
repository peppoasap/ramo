# Native Changes and merge visual acceptance — 6 November 2026

Scope: selected workflow boards02 (Changes) and03 (Merge), preserving the
[visual contract](docs/design/rust-native/VISUAL-CONTRACT.md). History acceptance
is preserved in [the preceding report](docs/design/rust-native/qa-history-2026-11-06.md).
Full native parity remains open in NATIVE-COMPLETION.md.

## Sources, state and normalization

Source truth: [accepted workflow board](docs/design/rust-native/workbench-board.png),
1536×1024 concept image. Its Changes panel is cropped from529,116 to1010,474
(481×358); merge from1024,116 to1524,474 (500×358). These are original concept crops,
not invented substitutes. The owner accepted a wider implementation and dense
code/history. Source records and exact fonts are generated; no Git correctness
or exact pixel-equivalence claim follows from this source.

Implementation: actual macOS Rust developer bundle. Before/after full captures are
2984×1858 physical pixels; exported content2984×1796,1492×898 logical,2× density.
Only the62px OS strip is removed. Same viewport and same respective synthetic
repository state for each before/after pair: Changes has two unstaged files, one
staged file, search.rs selected; merge has two unmerged files, search.rs selected,
current/incoming above an editable marked result. Final Changes includes a typed
message; baseline message is empty. Source Changes also shows a selected-line
popover; it is an interaction state, not persistent page furniture.

[Changes combined comparison](docs/design/rust-native/workflow-fidelity/changes-comparison.jpg)
and [merge combined comparison](docs/design/rust-native/workflow-fidelity/merge-comparison.jpg)
contain concept, before and after together, each normalized to960px width with
aspect ratio preserved. Source panels have smaller/narrower original resolution;
no inferred pixel distances are used across these unequal aspect ratios.
Readable [Changes detail](docs/design/rust-native/workflow-fidelity/changes-details.jpg)
and [merge detail](docs/design/rust-native/workflow-fidelity/merge-details.jpg)
comparisons, plus original full-resolution native images, were also inspected.

## Findings and iterations

- Fixed P1, Changes: the branch sidebar plus modified-file pane competed for space,
  and a bottom composer split the reading flow. Changes now uses the actual file
  list, center diff and a dedicated right composer with persistent commit actions.
- Fixed P2, Changes: raw diff headers consumed space and staging labels dominated
  code. Hide metadata before the first hunk, retain original raw row identities,
  add semantic row backgrounds, bounded paths and explicit file status. Actual
  partial staging proves the hidden-header offset does not stage different rows.
- Fixed P2, first revised Changes capture: wide file buttons centered names.
  Right-side status slots now anchor names left; final capture confirms alignment.
- Fixed P1, Merge: no conflict-file navigation and undifferentiated source/result
  surfaces. Add conflict sidebar, bordered current/incoming sources, separate
  editable result and primary Save and mark resolved action. Save draft remains
  secondary; completing the merge remains a separate Changes operation.
- Fixed P2, first revised merge capture: source buttons stretched across both code
  panes and left too little result space. Intrinsic-width controls and a32% source
  height cap return more space to the result; final capture was reviewed again.
- Fixed P1, data preservation: moving among conflicts must not discard memory.
  Back/Open/Close/Conflict now use one explicit destination enum and modal gate;
  pending draft saves prevent discard. Actual dirty switch and Cmd-Q showed the
  gate; Keep/Escape retained the result, explicit discard changed only the view.

## Required fidelity surfaces

Typography: existing bundled Noto Sans regular/semibold hierarchy, compact mono
code and real Phosphor icons. Source font identity is unknowable; the explicit
bundled substitute is retained rather than default toolkit typography. Spacing:
three meaningful regions, resizable files/composer, short controls, bounded
source frames and persistent footers. Graphite/mint tokens and quiet borders
match the selected identity; semantic green/red lines preserve plus/minus cues.
Assets: no replacement illustration/logo or raster screenshot used as live UI;
existing licensed fonts/icons retained. Copy: actual paths, staged status, counts,
conflict blocks and messages; no fake Connected/AI response/button behavior.

## Interaction and preservation evidence

Actual macOS UI on synthetic data:

1. Select original diff rows9/10, stage them, inspect staged version, unstage that
   hunk. Independent Git proves only the selected replacement entered the index,
   the unrelated test hunk stayed unstaged, then index/status returned to baseline.
   HEAD and all working-file bytes remained unchanged. The final recompiled
   bundle also staged only hunk2 and unstaged it back to baseline after the UI
   functions were separated for maintainability.
2. Type a Unicode commit message, observe save, reopen the app and recover exactly
   the same text in the right composer. Replace with an English synthetic message
   for the exported image. Narrow1002×683 logical content keeps staging and commit
   controls visible; long code uses horizontal scroll. Restore wide viewport.
3. Select current source line7 in the merge, edit the result manually to combine
   lowercase/hyphen handling, Undo/Redo, save a local draft and change files.
   Independent Git verifies editor actions/draft save leave HEAD/index/file bytes
   untouched. Dirty switch and Cmd-Q are cancellable; explicit discard is tested.
4. Resolve README line3, review save/stage, reopen search.rs and explicitly restore
   the saved draft from the older Git fingerprint. Review and save/stage with the
   existing session trust. Git verifies the exact staged bytes and no unmerged
   entries; the resulting synthetic Rust test compiles and passes.
5. Type the merge message and Complete merge. Git verifies both expected parents,
   exact result bytes, clean checkout and removed MERGE_HEAD. Composer clears.

Local native32 tests, native clippy with warnings denied, fmt, developer-bundle
build/signature and267 notices pass. The synthetic resolved Rust test is one
additional fixture check, not counted as a native unit test. No browser/console
or other-platform GUI claim applies. Self-review; no independent reviewer.

## Open gates and follow-up

No P0/P1/P2 remains for the inspected wide Changes/merge layout and observed
workflow. This scoped pass does not establish full six-board fidelity. Dedicated
comparison/settings/AI views, syntax highlighting, linked scrolling, IME/full
screen-reader acceptance, huge files and full narrow/DPI/platform GUI acceptance
remain. Native AI controls are not presented as implemented just to match a mock.
History remains22px compact/30px comfortable; no fake commits or topology used.

Previous source5c91b59 passed native/core and legacy source CI on all three OS
(native42199162572, legacy42199162408). This workflow revision needs fresh hosted
checks. No native installer, tag or release was created.

Implementation checklist: three-column Changes; shared style and explicit states;
source/result merge hierarchy; guarded conflict navigation; original staging IDs;
real staged/merge preservation evidence; complete goal ledger retained.

final result: passed

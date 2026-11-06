# Native visual contract

Owner correction, 6 November 2026: the Rust migration must preserve the selected
Ramo design. Functional progress does not excuse default toolkit styling.

## Visual sources, in priority order

- [Original selected workbench](../mockups/01-graph-desk.png): graphite/mint identity,
  clear typography, three regions and restrained controls.
- [Wide history revision](wide-history.png): compact real topology and a useful
  number of commits, accepted explicitly by the owner.
- [Contextual AI revision](../mockups/selected-ai.png) and
  [workflow board](workbench-board.png): future workflow surfaces, not proof of
  implemented native functionality.

## Implemented foundation

| Surface | Native rule |
|---|---|
| Typography | Bundled Noto Sans Regular/SemiBold; body/button14, caption12, headings17, brand22 logical px. Code remains monospace13 |
| Palette | Graph background #171b20, panel #1c2329, border #303b45, text #e8edf3, muted #a6b1c0, mint #7bd8ac |
| Chrome |44px repository strip,40px command strip,27px status strip. Mint underline identifies active repository/history tab |
| Density |22px compact history,30px comfortable. Real virtual rows and topology; never draw fake commits to match an image |
| Regions | Resizable sidebar215px default, inspector420px default; center gets remaining width. Author column yields before subject at narrower widths |
| Controls |3px corner radius,6px modal radius, bounded dialogs and visible footers; icons from existing Phosphor font with fallback in regular and semibold text |
| Branches | Lane-colored compact badges; current local branch has sidebar selection/check. Keep single-click inspection, double-click checkout and context menus |
| Code preview | Actual diff hunk coordinates, addition/deletion backgrounds, horizontal scrolling for long lines; metadata omitted before text hunks. No syntax-highlighting claim |
| Dialogs | Consistent heading, field rhythm, selected provider indicator, short primary action, preserved modal input blocking |

Noto's unmodified font files retain [OFL-1.1](../../../apps/native/assets/fonts/OFL.txt).
Their upstream source and SHA256 hashes are recorded in the adjacent manifest;
`scripts/native-notices.py` validates them and includes the license in developer
bundle notices. These assets are independent of the application's Apache-2.0 code.

## Acceptance discipline

Compare actual source and implementation images together, then inspect readable
crops. Record viewport/density, interaction state, fixture differences and exact
limits in the root design-qa.md. Preserve original images. Do not call another
unrelated concept a match, demote material drift to polish, or infer UI acceptance
from compilation. New UI should reuse theme.rs before adding a local style.

History/navigation and Changes/merge now have scoped actual macOS visual passes.
Changes replaces branch navigation with unstaged/staged files, center diff and
right commit composer. Merge uses conflict navigation, two framed source editors,
editable result and separate draft/save-stage actions. Comparison and future
AI/settings surfaces still need dedicated matching-state visual passes. Full
migration and release acceptance remain open in NATIVE-COMPLETION.md.

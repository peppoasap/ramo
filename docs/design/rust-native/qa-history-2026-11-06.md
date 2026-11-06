# Native workbench visual fidelity — 6 November 2026

Scope: history/navigation, shared typography/controls and read-only inspector.
This is a scoped improvement pass, not a claim that the entire native application
matches all six workflow boards. Earlier records are preserved in
[the archived report](qa-before-visual-fidelity.md).

## Targets and evidence

- Source: [accepted wide history](wide-history.png),1717×916
  concept pixels; original selected graphite/mint direction also inspected.
- Implementation before/after: [before](visual-fidelity/before-history.png)
  and [after](visual-fidelity/native-history.png),3024×1836
  pixels,1512×918 logical content,2× density. Only62 physical pixels of OS chrome
  removed from3024×1898 captures; original captures remain in ignored QA storage.
- Same actual synthetic55-commit/four-local-branch fixture, compact22px rows,
  current branch feat/search, selected Normalize search input, inspector open.
  Source uses generated records and a different wider aspect ratio; it cannot
  establish exact pixel equality or Git correctness. Before/after share viewport
  and real data. The generated source's open commit menu is not a persistent bar.
- [Combined full comparison](visual-fidelity/comparison.jpg)
  puts source, before and after in one input; all normalized to1100px width,
  preserving aspect ratio. [Focused upper-region comparison](visual-fidelity/details.jpg)
  uses matching normalized widths; original full-resolution after was inspected
  for glyphs, truncation, code rows and button alignment.

## Findings and iterations

1. P1 — default regular-only typography weakened headings and navigation. Fixed
   with real bundled regular/semibold fonts and explicit named-family headings,
   selected tabs and section labels. First capture exposed missing icons in the
   named family; fixed the font fallback chain and recaptured the final binary.
2. P2 — selected repository/history looked like ordinary buttons; branch labels
   lost their badge treatment. Added restrained active-tab underlines, repository
   tint, lane-colored badges and current-branch check. Badge ellipsis remains
   bounded to its column; complete names stay accessible through existing menus.
3. P2 — repeated commit count/search row consumed graph height. Search moved into
   commands; count remains in footer, returning one row of controls to history.
   Actual search was exercised and narrowed55 records to3 matching commits.
4. P2 — raw patch metadata dominated the inspector. Hunk preview now shares the
   existing diff-coordinate parser, shows actual line numbers and semantic row
   backgrounds. Long lines scroll; code is not reflowed or modified. Existing
   truncation warning remains. No syntax-coloring claim.
5. P2 — clone navigation lacked a selected-provider indicator with frameless
   buttons. Fixed selected frame and heading weight; inspected final clone dialog
   with visible Cancel/Review footer. Branch dialog and context menu also observed.

## Required fidelity surfaces

- Fonts/type: regular14, real semibold17 headings,12 captions, monospace code.
  Source's exact generated font is not identifiable; Noto Sans is the explicit
  reproducible substitute, closer to the original system-sans direction than
  default egui typography. Phosphor glyphs checked after fallback correction.
- Spacing/layout: three regions, compact22px table, short toolbar, underlined tabs,
 3px control radii. Inspector starts wider; author yields below820px center width.
  The1002×683 logical narrow content capture retains commands but requires table
  horizontal scrolling with the inspector open; wide desktop remains the target.
- Colors/tokens: graphite surfaces, mint active accents, quiet borders, separate
  topology colors and dark-on-colored branch badges. No decorative gradients.
- Assets: no invented illustration/logo replacement. Existing licensed Phosphor
  font; unmodified Noto fonts with checked hashes and bundled OFL text. No raster
  mock used as application UI.
- Copy/content: actual Git subjects, hashes, branches and diffs. Status now says
  commits loaded rather than rendering diagnostics. Native preview status remains.

## Interactions and independent preservation checks

Actual macOS developer bundle: branch inspection, branch context menu/Escape,
Create branch form/Cancel, Clone/Escape and modal blocking, moved search/clear,
window resize and returning to the wide layout. No Git write or account request
was issued in these UI checks. Independent Git commands confirmed unchanged
fixture HEAD, index entries and clean status.32 local native tests pass; native
clippy/fmt, bundle build/signature and267 notices pass. One test validates preview
coordinates across hunks. No browser-console claim applies to this native app.

## Follow-up and boundaries

- Dedicated visual acceptance for Changes, comparison, merge, settings/AI and
  additional dialog states remains required; this pass does not close those gates.
- Commit body/stats, syntax highlighting and linked editor scrolling remain
  functional gaps; their absence is not passed as six-board fidelity.
- Platform GUI, screen-reader/IME and installation acceptance are unverified.
- Latest previous hosted source4b45f2a passed native macOS/Linux; Windows failed
  a synthetic clone LF assertion because clone honors runner autocrlf. The fixture
  now commits eol=lf attributes; local pass does not establish the next Windows run.
- Review is self-review. Future screenshots must preserve real row density.

Implementation checklist: shared typography and state tokens corrected; history
chrome/badges and inspector preview refined; final binary visually inspected;
remaining workflow and platform gates recorded in STATUS.md.

final result: passed

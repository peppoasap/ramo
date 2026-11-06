# Native workbench design QA — 5 November 2026

Source: [accepted wide concept](docs/design/rust-native/wide-history.png),1717×916.
Implementation: [history](docs/images/native-history.png), actual native macOS
capture3024×1900 before62px top chrome crop. Native window1512×950 logical at2x,
constrained by this display; default request1680×960. No CSS/browser runtime.
[Combined comparison](docs/design/rust-native/native-comparison.jpg) preserves
aspect ratio and fits both into equal880px-wide comparison regions. Different
synthetic commits and viewport proportions are deliberate; no pixel-parity claim.
State: dark, top merge selected,5011 loaded commits,32 visible compact rows.

## Findings and comparison history

- Fixed P1: source line controls previously exceeded their20px text row height.
  Gutter controls now use exact heights and visible-row virtualization. The
  [revised merge capture](docs/images/native-merge.png) shows aligned source rows;
  selecting current line7 changes only the active conflict.
- Fixed P2: branch buttons aligned centrally and current branch lacked clear
  emphasis. Revised history uses left alignment and mint selection.
- Fixed P2: saved-draft action had insufficient foreground contrast. Revised
  merge action uses dark text on mint.
- P3: egui default fonts differ from the concept. Font-family/weight refinement
  remains; current hierarchy and monospace code remain readable.

## Required surface review

Typography: explicit subject/author/hash columns, monospace diffs, default native
fonts rather than a claimed Inter match. Spacing:22px compact rows preserve density;
resizable three regions, short toolbar controls, no card grid. Colors: graphite
surfaces/mint selection, separate colored topology lanes and readable disabled
network controls. Imagery: no hero/illustration substitution; real Phosphor icons
with notices. Content: actual synthetic Git records replace generated text;
read-only limitations visible. Focused toolbar/table/inspector regions reviewed in the
[combined detail comparison](docs/design/rust-native/native-detail-comparison.jpg),
plus the full-resolution source gutter/result editor capture.
No persistent controls overlap at the observed desktop viewport.

## Interaction evidence and remaining gates

Exercised open picker, refresh, pagination, loaded-history search, keyboard
navigation, conflict source-line pick, manual result edit, undo/redo, explicit
draft save/restore and unsaved-close guard. Native accessibility labels were
inspected; full screen-reader and IME acceptance remain untested. Minimum-width
and other platform GUI acceptance remain follow-up gates, not implied by builds.
Git writes and full beta parity are intentionally outside this read-only slice.

final result: passed

This result applies only to the observed R0 desktop layout and evaluated R1
interactions. It does not approve replacing the released application/editor.

## Subsequent daily-write UI coverage

The initial pass above covers the R0 mock comparison. New Changes sections and
review actions are an incremental R2 implementation, not a completed six-board
fidelity acceptance. [Staging capture](docs/images/native-staging.png) records the
actual synthetic file/index preview. The live merge exercise found and corrected
a disabled Complete merge action when the reviewed result equals HEAD; the
post-fix GUI completed that two-parent merge. Full native workflow fidelity,
remaining dialog layouts and alternate viewport checks stay in the completion ledger.

## Native diff detail iteration

[Actual line selection](docs/images/native-line-selection.png) records a3024×1898
macOS capture with only62px of OS chrome removed (3024×1836 content),2x density. Initial gutter buttons
exceeded22px and the selected check glyph used a missing fallback; the revised
capture uses fixed-height padding and the licensed Phosphor check icon. Original/
new numbers align with code rows. Accessibility off/on selection and Shift range
selection were observed, followed by exact index-content checks. These are scoped
interaction/detail checks; six-board fidelity/cutover acceptance remains open.


## Native action/composer refinement — 2026-11-06

Actual macOS bundle inspected, not a generated preview. Commit menus expose real
core actions with unavailable mutations disabled. Forms/reviews use centered modal
backdrops, compact fields,3px button corners and a mint primary action. Escape was
verified on the branch form. A light-system palette mismatch found at runtime was
corrected by pinning the selected dark theme. Actual synthetic screenshot:
`docs/images/native-branch-modal.png` (3024×1836, OS strip only removed).

This is scoped dialog/contrast verification; dense history/mock fidelity, narrower
window behavior, platform accessibility and all remaining dialogs are still open.


## Native comparison refinement — 2026-11-06

Actual Compare view verified on macOS with ordered endpoints, direct/ancestor
modes, Swap, file list and color-coded diff. It is separate from working Changes.
Inspection found a missing arrow glyph and unnecessarily long effective-ID labels;
these now use plain text and short IDs with full hover detail. Picker selection
was updated and verified to close its popup immediately. Actual capture:
`docs/images/native-comparison.png`. Dense-history/layout fidelity and narrow-window
comparison acceptance remain separate open gates; this is scoped workflow QA.


## Native graph gestures — 2026-11-06

Actual synthetic macOS trials exercised subject/branch drags and exact ordered
comparisons, then reviewed merge/cherry-pick. Current-branch destination is named;
a temporary bounded overlay supplies intent feedback without moving graph rows.
The raw press origin is captured so rapid coalesced events do not become a drag
of the destination. Context menus remain available. Cancel left Git unchanged;
a confirmed merge completed with independently checked parents/content.

Evidence: `docs/images/native-drag-merge-review.png`. No dense-history screenshot
fidelity upgrade or OS cancellation/keyboard/accessibility certification is claimed.
Headless cancellation tests and actual gesture observations are recorded separately.


## Native clone/remotes — 2026-11-06

Actual macOS dialogs inspected in the running Rust bundle. Clone uses flat provider
navigation, compact source/destination fields and a fixed footer. A separator
initially consumed the available viewport height and hid the footer; a finite
260px divider and bounded360px content region fixed it. Provider URL/folder hints
were observed without loading an account. Optional catalog loading stays explicit.

Remotes separates sanitized existing endpoints from a short add/edit form, protects
complex/local origin and suggests an unused name. Unchanged edits cannot proceed.
Toolbar, graph, sidebar and inspector are disabled behind dialogs. Actual local
clone/draft isolation, remote add/edit and Refresh after an external change were
exercised independently of fixtures. Screenshots: `docs/images/native-clone.png`
and `native-remotes.png`, synthetic data only. Narrow-window/platform accessibility
and hosted-account acceptance remain open; these are runtime images, not mockups.

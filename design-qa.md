# Native comparison visual acceptance — 6 November 2026

Scope: revision comparison, using the accepted graphite/mint visual contract and
existing read-only comparison behavior. [History](docs/design/rust-native/qa-history-2026-11-06.md)
and [Changes/merge](docs/design/rust-native/qa-workflows-2026-11-06.md) acceptance
remain recorded separately. This is not full native completion or release acceptance.

## Target and state

The accepted [wide history](docs/design/rust-native/wide-history.png) and workflow
board define typography, tokens and restrained workbench controls. They do not
contain a dedicated comparison mock. This pass reuses those established patterns
in the existing comparison screen; it does not claim exact matching to an invented
comparison concept or to a history screen with different content.

Actual before/after: same synthetic Meadow repository, main3edbd899 to
feat/search150ad2cb, direct comparison, src/search.ts selected, clean checkout.
Both raw captures2984×1858, exported content2984×1796 with only62px OS strip removed;
1492×898 logical content at2× density. [Before](docs/design/rust-native/comparison-fidelity/before.jpg)
and [after](docs/design/rust-native/comparison-fidelity/native-comparison.png).

[Combined comparison](docs/design/rust-native/comparison-fidelity/comparison.jpg)
shows accepted style (explicitly a different workflow), actual before and actual
after together, normalized to1100px width preserving aspect ratios. [Readable
header/details comparison](docs/design/rust-native/comparison-fidelity/details.jpg)
aligns before/after at matching normalized widths. Full-resolution implementation
was inspected for icons, truncation, labels and paths. Concept1717×916 is used for
style grounding, not pixel-distance claims across unequal source states.

## Findings and iterations

- P1 fixed: raw full-hash input rows and duplicate branch navigation dominated
  comparison. Two compact side-by-side searchable selectors now show chosen names
  and pinned short IDs. Full IDs remain in an explicit expandable input section.
  Comparison has changed-file/diff regions, with no second branch navigation pane.
- P2 fixed: selecting a pair required another file click to see anything useful.
  The current successful comparison opens its first file automatically. Changing
  endpoints/file/mode still clears obsolete content and invalidates old replies.
- P1 fixed during actual input testing: the toolkit's default combo popup closed
  when clicking the search field. Explicit CloseOnClickOutside keeps input usable;
  choosing a result or clicking outside still closes the menu. Actual search MAIN
  finds main and selects the correct commit, case-insensitively.
- P2 fixed: unvalidated pasted text was described as a pinned commit. Custom IDs
  now request validation; only loaded selections or successful comparison objects
  receive the pinned label. Invalid commit IDs produce the core error and no old
  diff. Chosen names remain bound to their original objects even if a ref moves.
- P2 fixed in narrow capture: the summary's Unicode arrow lacked a glyph. Use the
  existing Phosphor arrow; final build/capture shows the correct icon.

## Required visual surfaces

Fonts: existing bundled regular/semibold Noto Sans and monospace code, explicit
caption hierarchy and selected names. Spacing: parallel selectors, compact
mode/actions, collapsible technical input, resizable file region and full diff
width. Narrow1002×683 logical content retained both selectors and Compare; long
code uses horizontal scroll. Colors: shared graphite/mint/border tokens and
semantic diff backgrounds. Imagery: existing licensed Phosphor icons only, no
mock raster used as interactive UI or substitute invented asset. Copy: actual
names, pinned objects, singular/plural file counts, effective base/target and
clear read-only/empty/error states. No AI/provider connection is simulated.

## Actual interaction and data evidence

Actual macOS developer bundle verified search and selection, automatic first diff,
Swap with inverted addition/removal, direct divergent three-file comparison and
switching README/search previews, common-ancestor comparison, same-commit empty
state, invalid raw ID error and recovery. For feat/search to docs/update, UI showed
effective base576d7820 and only CONTRIBUTING.md; independent Git agrees.

A temporary external move of the synthetic main ref did not alter selected
3edbd899/150ad2cb or its one-file result. The ref was restored with an expected-old
update. Independent commands then verified original refs, HEAD, index entries,
status and working-file hashes. This external test writes a fixture reflog; it is
not claimed as a byte-for-byte comparison of all .git metadata. UI comparison
itself issues read-only core requests; no trust or mutation/provider/network action.

Refresh in the same repository preserves the selected immutable pair and preview,
including after a controlled external ref move. Opening a different synthetic
repository clears both endpoints; returning does not reuse the other repository’s
comparison. Both cases were verified in the actual macOS app.

34 native tests pass, including two new tests for stable selected labels/search
and initial-file behavior after stale/error replies. Existing comparison, graph,
draft and Git preservation tests remain. Native clippy/fmt, developer-bundle
build/strict signature and267 notices pass. No browser-console claim applies to
this native app. Self-review; full screen-reader/IME, all DPI and other-platform
GUI acceptance remain open.

## Boundaries and next work

No P0/P1/P2 remains in this inspected comparison refinement. Full mock-board
fidelity still needs native settings/AI and future workflow surfaces. Worktrees,
repository tabs, editor syntax/linked scroll/IME/accessibility/large-file gates,
remote depth, rebase/recovery and native packaging remain in NATIVE-COMPLETION.md.
Prior sourcec7091c2 passed native/core42203220979 and legacy42203221057 across all
three OS. This comparison revision needs its own hosted checks. No native release.

Implementation checklist: compact named selectors, usable search, pinned objects,
read-only files/diff, stable async replies, validated error/empty states, preservation
proof, source/implementation comparisons and full objective ledger retained.

final result: passed

# Native workbench visual acceptance — 6 November 2026

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

## Worktree workflow refinement

Scope: functional worktree navigation/create/manage using the selected workbench
style. The wide-history mock supplies sidebar/chrome tokens and compact rows;
there is no dedicated worktree creation mock. No exact dialog pixel parity is
claimed. The fixture has one real commit and four linked checkouts, unlike the
reference's denser history; topology was not fabricated to fill the graph.

[Source and actual UI together](docs/design/rust-native/worktree-fidelity/comparison.jpg)
normalizes both to1200px width preserving aspect ratios. Actual captures3024×1898
export3024×1836 after removing only62px OS strip;1512×918 logical at2×.
[Readable menu/dialog crops](docs/design/rust-native/worktree-fidelity/details.jpg),
[full final](docs/design/rust-native/worktree-fidelity/final.jpg),
[creation](docs/design/rust-native/worktree-fidelity/create.jpg),
[existing-branch selector](docs/design/rust-native/worktree-fidelity/existing.jpg),
[refusal](docs/design/rust-native/worktree-fidelity/refused.jpg) and
[narrow](docs/design/rust-native/worktree-fidelity/narrow.jpg) were inspected.
Narrow content1002×683 logical retains dialog fields and the review/cancel footer.

Existing Noto/Phosphor assets and graphite/mint tokens are reused,3px controls and
6px modal corners. Worktree paths align left with current/lock state right and a
muted branch caption. + creates, click opens, right-click presents relevant actions.
The creation form exposes only location, folder and new/existing branch selection;
full destination wraps, footer remains visible, unavailable actions are disabled.
No imagery, dependency or invented asset was added. No P0/P1/P2 found in this scoped
pass. Broader tabs/recents and platform accessibility remain open, not polish gates
claimed complete here.

Actual macOS verified new/existing branch creation; only free existing branches
appear. Lock/unlock updates the row and locked removal is disabled. Open/return
recovered Unicode composer text. Ignored data caused a refusal; a clean checkout
was removed with its branch retained. Git independently verifies source HEAD/index
tree and working/untracked bytes, protected ignored sentinel and final linked
checkout/branch state. Worker tests add collision/trust/grant/membership/open-
checkout refusal coverage. No new conflict fixture was exercised in this pass;
the existing merge exit guard is reused.36 native tests and lint/fmt pass. Bundle
strict signature and267 notices pass. Self-review; no native release or new AI.
Previousfa2a53a native/core42208900871 and legacy42208900858 passed all three OS;
new source CI remains pending. Full goal ledger remains open.

## Repository tabs and opening trust

Scoped extension of the accepted workbench chrome: compact tabs, active mint
underline, flat close actions, Open/recents and one opening-trust dialog. The
[combined reference and implementation](docs/design/rust-native/tab-fidelity/comparison.jpg)
normalizes each to1200px width with preserved aspect ratio. Source1717×916 is the
accepted history revision; actual3024×1898 raw capture exports3024×1836 content
by removing only62px OS strip (1512×918 logical at2×). Actual55-commit fixture and
selected CONTRIBUTING commit differ from the concept; topology/data were not
fabricated to match it. No dedicated multi-tab/trust mock supplies exact pixel
parity for these added states.

[Readable strip/overflow crops](docs/design/rust-native/tab-fidelity/details.jpg),
[full actual](docs/design/rust-native/tab-fidelity/final.jpg),
[narrow](docs/design/rust-native/tab-fidelity/narrow.jpg),
[opening trust](docs/design/rust-native/tab-fidelity/open-trust.jpg),
[recent restart](docs/design/rust-native/tab-fidelity/recent-restart.jpg),
[restored Changes](docs/design/rust-native/tab-fidelity/restored-changes.jpg),
[comparison](docs/design/rust-native/tab-fidelity/restored-comparison.jpg) and
[merge guard](docs/design/rust-native/tab-fidelity/merge-guard.jpg) were inspected.
Narrow1002×683 logical shows two tabs and +2 for four open repositories, keeping
the active tab visible. Choosing the hidden repository works. No horizontal tab
scroll. The first iteration truncated normal names too early and boxed close
buttons: widened tabs to180px labels, flat20px close targets with explicit AX names
and a conservative width budget. Existing Noto/Phosphor, palette,3px controls and
6px modal are reused. No new raster, font or dependency was introduced.

Functional evidence: fresh Changes/file/comparison reads and filters restore on
activation; external edit becomes visible and is restored afterward. Unicode
composer survives switch/restart. Active switch/Cmd-W guard unsaved merge text;
inactive close preserves it. Last close shows the empty workbench; a missing
recent folder leaves the active view intact. Read-only opening is not repeatedly
prompted on activation; trusted tabs keep session consent until close. Explicit
close/reopen prompts anew. After trust acceptance for an explicit untrusted branch
request, the operation review remains and Git confirms no branch was auto-created.
Independent source/compare/conflict preservation checks pass.38 native tests,
fmt/clippy and strict developer bundle/267 notices pass. Self-review only.
No P0/P1/P2 remains in this inspected tab/trust scope. Full keyboard/screen reader/
IME/DPI/platform acceptance remains open; accessible close names do not prove it.
Open-tab/view state is in-memory; only recents/drafts survive restart at present.
Previousa1eb21c CI passes all three OS; new source requires fresh hosted checks.
Full objective and native release gates remain open.

## Durable workspace and installed glyph fallback

Scope: restore the selected workbench/navigation intent after restart, preserve
Git/read-only authority, and make recovery states understandable without adding
a dashboard. [Reference and actual UI together](docs/design/rust-native/workspace-fidelity/comparison.jpg)
normalizes each to1200px width preserving aspect ratio. The accepted1717×916
history concept and actual55-commit fixture differ in data/topology/content; no
fake commits or invented comparison pixels were used. Actual raw3024×1898 exports
3024×1836 content after removing only62px OS strip;1512×918 logical at2×.
[Full final](docs/design/rust-native/workspace-fidelity/final.jpg) and
[narrow1002×683](docs/design/rust-native/workspace-fidelity/narrow.jpg) retain
compact chrome and functional history. There is no dedicated restore-error mock
from which to claim exact pixel parity.

[Readable recovery/fault comparison](docs/design/rust-native/workspace-fidelity/details.jpg),
[missing folder](docs/design/rust-native/workspace-fidelity/missing-folder.jpg),
[recovered folder](docs/design/rust-native/workspace-fidelity/recovered-folder.jpg),
[fresh trust](docs/design/rust-native/workspace-fidelity/startup-trust.jpg),
[concurrent writer](docs/design/rust-native/workspace-fidelity/cas-error.jpg),
[corrupt record](docs/design/rust-native/workspace-fidelity/corrupt.jpg),
[explicit merge draft](docs/design/rust-native/workspace-fidelity/restored-merge-draft.jpg)
and [Unicode composer](docs/design/rust-native/workspace-fidelity/unicode-fallback.jpg)
were inspected. Existing graphite/mint/Noto/Phosphor, flat tabs, restrained radii,
three regions and visible actions remain. Recovery copy now handles singular
folder count; the notice clears on success/close. Save errors refer to workspace
rather than the generic draft store. No new decorative raster or custom icon.

P2 found/fixed in inspection: Japanese draft bytes were preserved but displayed
replacement squares. Optional installed-font fallback now validates font format
and representative glyph coverage using the already locked parser. Actual macOS
composer and merge result show Japanese text; no system font is bundled and Latin
font priority is unchanged. Windows/Linux font availability and broader shaping/
IME/a11y acceptance remain open; this is not universal Unicode certification.

Actual restarts restored three tabs/active comparison/filter/selection/Changes file,
Unicode composer and preferences. Fresh diff showed an external edit while closed
(edit restored). Missing retry failure preserved current view; restored folder retry
cleared its notice. Merge Keep editing cancelled quit; local draft recovery remained
explicit. A closed tab stayed absent. Competing and corrupt workspace bytes stayed
identical through failures/retry/quit while in-memory work remained usable. Git
independently preserves all three synthetic fixtures; no staging/completion/AI/
network action in this pass.43 tests, fmt/clippy, strict bundle and267 notices pass.
Self-review; no P0/P1/P2 remains in this inspected macOS workspace scope. Pixel pane
geometry, automatic schema migration and other-platform GUI remain outside this
slice. Previous6b1364f CI passes all three OS; this source needs fresh checks.
Full objective and native release ledger remain open.

final result: passed

# Native completion ledger

Objective: a complete Rust-native Ramo with the detailed, wide/compact UI selected
in the mockups. This ledger preserves the full objective; a passing slice does
not close it. Installed Git remains the explicitly documented initial engine.

| Requirement | Current evidence / missing proof |
|---|---|
| Rust application/UI without WebView or JS build | egui/eframe consumer builds; initial native CI passed three OS. Core reused without desktop feature |
| Wide, dense actual topology |5011 commits loaded,32 visible22px rows; graph parity/50k tests. Resizable panels; wider/narrower/DPI acceptance still needs more coverage |
| Full normal merge editor |Line picks, edit, undo/redo, find, draft restore, reviewed save/stage and merge completion observed. IME, accessibility, typed CRLF, linked scroll, syntax and large files remain |
| Fast everyday actions |File stage/unstage, commit, direct double-click checkout, branch actions, network/stash routes implemented. Hunk/line staging delivered for regular modified text; no-final-newline line selection falls back to whole hunks. Amend and supported cherry-pick/revert sequence UI delivered; rebase remains |
| Detailed object menus and drag/drop |Commit and branch/stash menus implemented; direct/unique-ancestor comparison and ordered endpoints delivered. Reviewed single-object compare/merge/cherry-pick drag delivered. Broader file/ref menus, graph auto-scroll/multi-selection and full keyboard/accessibility acceptance remain |
| Clone and remote integration |Native URL/provider/local clone, explicit GitHub catalog and remote inspect/add/edit delivered; local clone/config GUI observed. Hosted auth, Stop/streamed progress, shallow/sparse, additional catalogs and removal/upstream/richer endpoint controls remain |
| Worktrees and tabs |Native listing only; open/create/manage, recent repository tabs and state restoration missing |
| Contextual optional AI |Existing core/beta adapters; native settings, input review, draft/explain/resolve and Stop not ported. No new live inference authorized |
| Daily-base depth |Recoverable discard, file history/blame, reflog/reset and guided/interactive rebase remain |
| Maintainer readiness |Small native modules, pinned lock, Apache2/notices, synthetic tests, updated docs. Full workflow acceptance matrix remains |
| Native installers and cutover |No native release yet; packaging/CI assets, migration/rollback, platform install/runtime acceptance and signing strategy remain |
| Performance |Real paging/virtualization evidence; matched cold/warm latency/RSS/frame benchmark and GitKraken comparison not established |

Read STATUS.md for the exact next slice and NATIVE-VERIFICATION.md for dated
checks. Completion requires evidence for every remaining row, not just green CI.

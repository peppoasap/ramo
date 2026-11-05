# Design package verification

Date: 2026-11-03. Scope: local documentation and generated design assets only.

- All local Markdown links resolve.
- Four delivered images have valid PNG signatures and readable dimensions.
- Images were copied unmodified from the built-in generation outputs.
- The Apache 2.0 license was downloaded from the official Apache URL; its title, version, end marker and appendix were checked.
- All four images were visually inspected in the conversation. They use synthetic meadow demo data and preserve the intended layout hierarchy.
- The user-supplied private screenshot was not copied into the project.
- No application, desktop or provider tests were run: there is no implemented application yet.

## Delivered rasters

| File | Dimensions | SHA-256 |
| --- | --- | --- |
| 01-graph-desk.png | 1487 × 1058 | a1fd79b5b76cf0a9e9264185c6816958efe270c28d0a45f690ca0a40878d156a |
| 02-change-desk.png | 1487 × 1058 | 64e4e7984d90715f10901f3bafae8c85639d84fcca59b956ad1d824e6ede86a4 |
| 03-workspace-desk.png | 1487 × 1058 | 3ca7ecfd32593eee46e1d16872e4fea925fa2c036e50a9bc424508e5aafdb96b |
| selected-ai.png | 1487 × 1058 | 5167f493a83d03921bc10515aed9feca0ba4b6e98ad54669320b5b5b804876f5 |

## Visual limits to correct during implementation

Generated graph topology, hash text, diff line numbers and code are illustrative and are not correctness evidence. Narrow inspector content needs measured resizing and overflow behavior. The original mockup's ahead/behind and Push state must be derived from real Git data. The AI revision illustrates a disconnected provider and cannot establish a working integration. Accessibility and platform behavior remain untested.

This report verifies a design package, not a coded-prototype design QA pass.

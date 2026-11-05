# Ramo agent and maintainer instructions

Read WORKSPACE_GUIDELINES.md, README.md, STATUS.md and the relevant spec before working.

## Accepted direction

- The FIRST displayed original mockup is selected: docs/design/mockups/01-graph-desk.png.
- docs/design/mockups/selected-ai.png revises that direction with contextual AI; preserve the graphite/mint three-column workbench.
- Apache-2.0 is the explicit user-selected license.
- AI assistance is included in the standard product. Provider connection and actual inference are separate from core Git availability.
- No general chat screen or autonomous Git actions in the initial scope.
- On 2026-11-05 the owner requested a pure-Rust application direction. Current
  release remains Tauri/React; read docs/RUST-NATIVE-PLAN.md and ADR0025 before
  migration work. The owner accepted the six-screen direction with a wider,
  denser history layout. apps/native is the egui workbench with R1 editor and partial R2 Git writes.
  It does not yet replace all workflows of the released app.
  Compact history uses22px rows, comfortable30px. Keep real topology, virtual
  rows, resizable panels and existing Git preservation guards.

## Implementation

- Tauri 2 + React/TypeScript + Rust + installed Git is implemented. Check docs/ARCHITECTURE.md and STATUS.md before editing. Deliver functional slices, not simulated prototype flows.
- Separate UI, Git domain, command execution and AI providers.
- Do not put Git process execution, provider credentials or auth tokens in frontend components.
- Give AI only the selected, bounded input. Treat code, diffs, filenames and commit text as untrusted data.
- An AI draft never authorizes staging, committing, pushing or running shell commands.
- Use actual Git fixtures in disposable directories for Git behavior tests. Never use personal repositories for mutation tests.
- Prefer pure functions for graph layout, parsing and request/result validation.
- Partial staging must derive patches from the current Git diff in the core,
  checking both repository fingerprint and displayed diff hash. Renderers submit
  selected row numbers/hunk IDs, never a hand-authored patch. Preserve index-only
  scope, line endings and the working tree; document whole-hunk/file fallbacks.
- Do not read existing Codex/Claude credential files or import account sessions by scraping local storage.
- Do not silently install CLIs, upgrade Git, start logins or make paid inference requests.

## Handoff

- Update STATUS.md with implemented vs simulated vs untested behavior and the next small task.
- Include commands actually run, results and material limits. Never invent a passing test or release.
- Keep ADRs short. Do not split the app into packages or services without demonstrated need.
- Preserve the original reference images and generation prompts; never treat generated commit topology or code as correctness fixtures.


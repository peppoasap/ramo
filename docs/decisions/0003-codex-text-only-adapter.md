# ADR 0003: versioned Codex CLI contract

Date: 2026-11-03. Status: accepted for local implementation.

Use official Codex CLI 0.166.0 with its own account boundary for contextual text
assistance. An authorized live synthetic draft succeeded after the older 0.139.0
CLI had failed. Never import auth files or silently install/upgrade a CLI.

Pin this initial adapter to the exact verified version. Share its argument builder
with a credential-free loopback capture that confirms zero model tools in the
outgoing request. Prompt instructions alone do not enforce this boundary.
Unsupported versions stop before receiving a diff. Revalidate request tooling and
account behavior before expanding compatibility; verify managed environments
separately. Read-only temporary working directories and stale-input checks remain.

Users choose an installed executable through a native picker. Only the non-secret
path is saved, atomically, after a version probe. The renderer cannot supply an
arbitrary command/path. Advanced integrations may use RAMO_CODEX_PATH. Provider
software is not bundled. This introduces a deliberate compatibility/update step
but keeps account reuse and local Git independent.

Evidence: docs/verification/codex-contract.json, ai-smoke-0.166.json and
codex-native-setup.json. One successful request is not proof of every account,
model, token refresh path or desktop platform.

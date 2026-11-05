# Contextual AI

Core Git works without AI setup or a Ramo account. AI actions are part of the normal interface. Codex commit drafting has one successful authorized live check; other provider routes remain unverified.

## Flow

Choose Write with AI (staged changes only) or Explain diff (selected diff). Review the exact included/excluded paths and input, select provider/model, then explicitly generate. A suggestion is editable text. It never stages, commits or pushes. Manual drafts survive errors; repository fingerprints are checked before and after inference.

No inference runs on repository open, refresh, staging, provider discovery or CLI selection. No automatic provider/model fallback or retry spends quota.

## Codex setup

1. Install the official **Codex CLI 0.166.0**, the version verified with this adapter. Ramo does not install or update it silently.
2. Sign in through that CLI's own `codex login`. Ramo never imports credentials or reads token files.
3. In AI providers, choose Codex. If it is outside the app's PATH, use **Choose installed Codex CLI…** and select the executable. This opens a native file picker, runs only `--version`, then saves the non-secret path immediately. Other provider/model settings are saved with Use this provider.
4. Select an entitled model or leave it blank to use the non-secret model preference in Codex config. An authenticated CLI does not guarantee model access.
5. Review the staged input and generate deliberately. Usage belongs to the user's account/plan.

Advanced integrations can supply an absolute executable path through `RAMO_CODEX_PATH`; a saved native selection takes precedence. No arbitrary frontend command/path is accepted for execution. Other CLI versions are rejected before sending a diff until their request contract is verified.

The local test installation under `releases/local/tools/codex-0.166.0/` is ignored and is not bundled into Ramo. Provider software retains its own license and account terms.

## Verified contract

Ramo runs Codex in an isolated temporary working directory, ephemeral/read-only, ignoring user config with strict config parsing and no approval escalation. Shell, apps/plugins, browser/computer use, image tools, Code Mode host, skill search/dependencies and other agent features are disabled. The repository is not passed as its working directory.

`scripts/check-codex-contract.py` obtains the **actual adapter arguments** from a Rust helper, uses an empty temporary CODEX_HOME without API credentials, and captures the request at a loopback server that always rejects it. With CLI 0.166.0 the request contains **zero model tools**. This performs no inference and consumes no quota. Run it with the selected CLI available through PATH or `RAMO_CODEX_PATH`.

Evidence: `docs/verification/codex-contract.json`. This proves the tested version and local configuration, not arbitrary future releases or every managed enterprise environment. The documented `features.view_image` setting is described in [OpenAI's configuration reference](https://developers.openai.com/codex/config-reference/); the previously installed 0.139.0 does not recognize it.

## Providers and live evidence

| Adapter | Route | Evidence |
| --- | --- | --- |
| Codex | Official installed CLI 0.166.0 and its own sign-in | One authorized live generation, gpt-6.1-sol, 284 bytes from synthetic staged src/search.ts only. Returned an accurate editable commit subject; no Git mutation. See `ai-smoke-0.166.json` |
| Claude | Installed CLI with --bare, tools disabled and ANTHROPIC_API_KEY in app environment | Not installed here; live inference unverified. API billing differs from subscription usage |
| Ollama | Loopback service at 127.0.0.1:11434 | Synthetic HTTP contract tests pass; no local model/inference verified |

Three earlier invocations with Codex 0.139.0 were rejected for model/account compatibility; `ai-smoke.json` preserves that historical failure. The new successful test does not establish token refresh/revocation, every model, or UI-button end-to-end generation. No further billable requests are authorized by those records.

## Data boundary

The backend prepares immutable bounded input with repository ID, fingerprint and selected paths. Commit drafting uses the index, not unstaged changes or whole-repository history. Known sensitive paths (including rename sources), binaries and oversized diffs are excluded. Filtering is heuristic; review before sending. Per-file AI selection currently follows staging rather than a separate checkbox list.

Treat source text as untrusted data, delimit it in the prompt, and render output as text. Settings store provider/model choices and an optional CLI path, never credentials or raw diffs. Stale results are rejected. Running requests expose Stop generation in ordinary and merge assistance. Stop consumes the preview, suppresses late output and waits for local transport/process cleanup. A fresh input review is required before retrying. Manual drafts and merge buffers remain unchanged. Provider/server computation and usage may already have occurred; Stop cannot guarantee reversal of billing or server-side work. Quitting/crashing the app is not a verified replacement for Stop. See ADR 0017.

Ollama bypasses environment proxies, refuses redirects and caps model-list/generation responses at 256 KiB. CLI output is bounded. A timeout never retries automatically. Generated content always requires review.

## Offline cancellation acceptance

`python3 scripts/serve-ai-cancellation-fixture.py --state-dir <temporary-folder>` provides an Ollama-shaped loopback fixture only when port 11434 is free. It refuses model names other than `ramo-cancel-fixture`, performs no inference and records only body length/hash and connection events. Without a `respond` marker it waits for disconnect; with that marker it returns one fixed synthetic message. Do not leave it running or select a real model for these tests. Model discovery and provider preferences can be restored after native acceptance. No fake provider or test mode is bundled into the app.

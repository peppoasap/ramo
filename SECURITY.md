# Security

Ramo is public beta source at [peppoasap/ramo](https://github.com/peppoasap/ramo), not a supported stable release. Never put credentials, private diffs or vulnerability details in public issues.

## Reporting a vulnerability

Use GitHub private vulnerability reporting from this repository's Security tab if it is available. If that option is unavailable, contact the maintainer through [their GitHub profile](https://github.com/peppoasap) to agree on a private channel before sending sensitive details. A dedicated private reporting endpoint and response SLA have not yet been established; do not assume a public issue is private. This limitation is tracked in the beta checklist.

## Implemented boundaries

- Typed native operations, argument arrays and registered repository IDs; no frontend generic shell/filesystem plugin.
- Absolute executable resolution before launch; relative/empty PATH entries are ignored and missing programs fail without current-directory lookup.
- Session trust before mutation/network operations because Git hooks, filters and credential helpers may execute code.
- Inspection disables external diff, textconv and fsmonitor; inherited Git context variables are cleared.
- Literal pathspecs, bounded output, process-tree timeouts and reviewed-snapshot checks before writes, including branch identity and upstream when HEAD object IDs match.
- Shared operation mutex; external Git processes still require optimistic concurrency handling.
- Full binary patch fingerprints are hashed incrementally; ordinary captured output and stderr remain size-limited, and process deadlines still apply. Read failures never become successful partial results.
- AI input preview, staged-only commit context, known-sensitive-path/binary/size exclusions and stale-result rejection.
- Local Ollama HTTP bypasses environment proxies, refuses redirects and bounds both discovery and generation responses.
- No credentials imported from another app, no automatic inference, telemetry or background upload.

## Known limitations / release gates

The Codex adapter is pinned to CLI 0.166.0. Its actual arguments were checked against a synthetic loopback provider with an empty credential-free home: zero model tools were exposed. Other versions fail before a diff is sent. Revalidate this contract before widening compatibility, and separately verify managed policies/environments. Native CLI selection saves only a path and probes version; frontend code cannot supply an arbitrary executable path. See `docs/AI.md`.

Filename secret filtering is heuristic. Large inputs are explicitly excluded; non-UTF-8 paths are unsupported. Git hooks and configuration execute under the user's identity after trust. A timeout may leave an operation already applied: refresh and inspect before retrying. Credential-helper UX, hosted remotes, Windows/Linux runtime, crash recovery and very large repositories remain unverified.

The macOS build is local/ad-hoc signed, without Developer ID or notarization. Dependency notices accompany the bundle; public release still requires target-specific dependency review and platform validation.

## Local commit drafts

Commit message text is stored in the app WebView local storage per canonical repository/worktree identity, without uploading it. Applied AI suggestions follow the same rule only after the user chooses Use message. Ramo does not encrypt this local draft text or use the draft store for credentials or diff content. A matching submitted draft is removed after a successful commit; failed commits and newer text are retained. Storage failure is shown in the composer while the in-memory text remains available to copy. Normal restart recovery is tested; OS crash durability and concurrent multi-window editing are not established.

## Dependency checks

Run `npm run audit:dependencies` against the committed lockfile before publication. It fails on known moderate-or-higher npm advisories and is configured in the build matrix. The local 2026-11-03 report has zero npm findings after updating the Vitest development dependency; see `docs/verification/dependency-audit.json` for scope and evidence. This does not establish that the application or native dependencies have no vulnerabilities. The hosted CI matrix and target-specific runtime checks still need execution.

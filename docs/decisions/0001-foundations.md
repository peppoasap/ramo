# ADR 0001: project foundations

Date: 2026-11-03.

## Context

The user wants maintainable everyday OSS replacements, starting with the essential GitKraken workflow. They selected the first mockup, requested contextual AI using their own provider accounts where supported, and chose Apache 2.0.

## Accepted

- Independent product and identity; Ramo is a working name.
- First original mockup as the visual direction, revised with AI actions.
- Graphical Git workflow with clear repository/worktree/branch context.
- AI actions included by default; provider setup on first use; manual Git remains available.
- Suggestions are reviewed text, never authorization for an automatic commit or push.
- Apache License 2.0 for original project code and documentation.
- Separate project status, roadmap and agent/contributor instructions.

## Original proposals (superseded by ADR 0002)

- macOS first; other desktop platforms later.
- Tauri 2 + React/TypeScript + Rust + installed Git.
- Official ChatGPT OSS sign-in as the first account connection candidate.
- Claude API-key route first; evaluate user-installed unmodified Claude Code separately.
- Folder boundaries inside one app rather than multiple packages/services.

## Consequences

The main workflow stays small, while provider-specific auth can change independently. Rust adds a second language; retain it only if the first technical spike validates the trade-off. UI and fixture work must not be reported as working desktop, Git or AI integration.

## Review trigger

Update this decision after M2 build/Git compatibility evidence or M4 real provider evidence. Add another ADR only when a consequential choice changes.


See [ADR 0002](0002-functional-desktop.md) for the implemented stack and functional delivery requirement.

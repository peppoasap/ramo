# Ramo beta.2

A focused graphical Git client with contextual AI assistance. Apache-2.0.

This beta includes a compact toolbar with Branch, Stash and reviewed Pop;
branch/commit context menus and drag gestures; text-hunk staging; an editable
merge resolver with individual source-line selection; stashes/worktrees;
provider-guided clone and reviewed remote URL management; optional AI with input
review and Stop. See the [README](https://github.com/peppoasap/ramo#readme) and
[changelog](https://github.com/peppoasap/ramo/blob/main/CHANGELOG.md).

## Downloads

Select the installer matching your operating system and architecture. SHA256SUMS
covers release assets. Git must be installed separately. No provider tools,
credentials, accounts or models are included.

All installers are unsigned test candidates. macOS CI packaging is not Developer
ID signing or Apple notarization; macOS may block a downloaded app. Windows
installers have no code-signing certificate. Do not disable system protections.
Successful platform builds/tests do not establish installation or runtime
acceptance on another machine. Local native evidence covers Apple Silicon macOS;
Windows/Linux runtime and first downloaded second-Mac launch remain outstanding.

## Try safely

Start with a synthetic repository or backed-up copy. Git operations are real.
Review staging and commits before writing. Trust lasts only while a repository is
open. AI is optional: own compatible provider setup and explicit input review
required; provider usage costs/limits apply. Codex CLI support is pinned to0.166.0.

Known gaps: line staging, reviewed discard/recovery, file history/blame, reflog/reset
and guided rebase. Binary/symlink/submodule conflicts use external tools. Hosted
authentication and large repositories need further acceptance. No auto-update.
Report non-sensitive reproducible problems via repository Issues; see SECURITY.md
for sensitive findings. No GitKraken parity or speed-up claim.

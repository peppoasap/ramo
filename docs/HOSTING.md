# Hosting and remote integration

Implemented local development workflow, 2026-11-05. No automatic account import,
login, credential-helper changes or provider CLI installation.

## Clone

Clone opens a provider sidebar: URL, GitHub, GitHub Enterprise, GitLab, Bitbucket,
Azure DevOps, enterprise/self-hosted and local picker. Hosted shortcuts construct
HTTPS/SSH URLs from a repository path and suggest a destination folder name.
Manual folder edits are retained. Use the URL entry for unusual path encodings,
custom ports or already-copied clone addresses. Azure shorthand uses
organization/project/repository. Generic self-hosted accepts a hostname and nested
repository path; service-specific/custom clone URLs can use the URL entry.

The native destination picker, immutable source/destination review, trust checkbox,
exclusive new-directory creation and existing Git clone transport remain unchanged.
URL construction does not establish that a repository exists or is accessible.
Clones use installed Git helpers or SSH agent, without initializing submodules.
Large clone progress/cancellation, shallow and sparse checkout remain unfinished.

## GitHub catalog

GitHub/GitHub Enterprise offer **Load repositories**. This explicit action runs the
installed official `gh` executable from an isolated temporary working directory,
using typed `gh api --hostname HOST --method GET ENDPOINT` arguments. No general
shell/process endpoint is exposed. Ramo never reads or displays account token files.
Sign in to the selected host with the official CLI outside Ramo if needed; missing
CLI/auth/access errors offer the URL path. Ramo does not start login or install it.

An empty organization lists repositories accessible to the active CLI account;
an organization scopes the list to that organization. Fetch 100 repositories per
page, up to20pages, most recently pushed first. Find filters only loaded rows;
Load next100 is explicit. Private/public metadata is shown. Results stay in memory.
Catalog JSON is capped at2MiB, subprocess output/deadline limits also apply, and
returned clone addresses must match the chosen host and repository name. Provider
errors are generic rather than passing potentially sensitive CLI stderr to UI.
Changing provider/closing does not cancel an already running read, which is bounded
to30seconds. No API request occurs merely by opening the dialog/provider.

Catalog access and Git clone credentials are separate: a working `gh` session does
not configure Git's credential helper automatically. Enterprise hosts with unusual
clone host/path conventions should use the URL workflow. GitHub catalog live account
acceptance has not been performed; parser/argument/synthetic CLI tests are separate
from actual hosted authentication. No credentials or real repository listings are
included in screenshots, source snapshots or tests.

## Remote endpoints

The repository toolbar's **Remotes** opens fetch/push endpoints even before any
remote-tracking branches exist. Add a named HTTPS/SSH remote or review a change to
an ordinary single URL. Saving is a trusted, fingerprint-bound typed Git action;
it edits local configuration without contacting the host or touching HEAD/index/files.
Remote endpoint output participates in review fingerprints. External changes still
use optimistic concurrency; no cross-process filesystem transaction is claimed.

Token-bearing, local/custom transport URLs are hidden instead of echoed. Existing
multi-URL or explicit push-URL configurations are not edited by this basic flow,
even when the current fetch/push values match. They remain available to Git itself.
Remove/rename remote, upstream editing and per-remote fetch/prune are later slices.
No network action follows Save automatically. Existing fetch/pull/push remains.

## Provider coverage and next adapter

| Provider | Clone address/picker | Account repository catalog |
| --- | --- | --- |
| GitHub / GitHub Enterprise | Implemented | Implemented via installed gh; live auth unverified |
| GitLab / self-managed | Implemented through shortcut or custom URL | Not implemented |
| Bitbucket / Data Center | Implemented through shortcut or custom URL | Not implemented |
| Azure DevOps | Implemented shorthand or full URL | Not implemented |
| Other Git host / local | URL or native picker | Not applicable |

Provider templates live in `repositoryProviders.ts`; the narrow native catalog in
`hosting.rs`; remote reads/configuration in `remotes.rs`. Add another account
adapter behind explicit load with a bounded typed response, isolated execution,
no credential scraping, no auto-login and synthetic tests. OAuth integration is a
future decision needing an owned public application registration and secure native
credential storage, not a button labelled Connected without an implementation.

Primary references: [GitHub CLI API](https://cli.github.com/manual/gh_api),
[GitHub repository REST API](https://docs.github.com/en/rest/repos/repos).

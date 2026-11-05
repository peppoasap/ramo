# Product brief

## Goal

Replace the essential daily workflow of a graphical Git client with a clear, maintainable open-source desktop application. A developer should understand the active repository and branch, inspect work, save a local commit and deliberately synchronize it.

Multiple projects and agent-created worktrees are first-class context. A small maintainer group must be able to understand and change the application without a bespoke platform.

## Accepted product principles

1. Show the active repository, worktree and branch before actions.
2. Prioritize a readable commit graph and readable diffs.
3. Clearly distinguish working files, staged changes, local commits and remote synchronization.
4. Include practical AI actions in the normal UI, using the user's chosen provider.
5. Keep the ordinary Git workflow available immediately without AI setup or a Ramo account.
6. Prefer explicit feedback and recoverable workflows over hidden automation.
7. Keep modules independent and documentation short enough to maintain.

## First usable release: 0.1 target

| Capability | Required behavior |
| --- | --- |
| Open local repository | Recognize normal and linked worktrees; show empty/error states; remember recent folders locally |
| Repository switching | Preserve local view selection; prevent operations/results from crossing contexts |
| History | Paginated topological commit graph, branch references, author/date/hash, commit and file selection |
| Diff | Unified text diff, added/deleted lines, binary/oversized file states, no implicit external drivers |
| Changes | Distinct staged and unstaged sections; file-level staging/unstaging; manual commit message |
| Local commit | Commit the index after review; no automatic push; surface errors without losing the message |
| Branches | Create/switch with visible target and dirty-tree preconditions; respect worktree branch occupancy |
| Remotes | Explicit Fetch, fast-forward-only Pull initially, normal Push with upstream shown |
| Worktrees | Discover and open existing worktrees; creation and removal are later slices |
| Stashes | List, create and apply with errors/conflicts visible; no destructive pop by default |
| AI commit draft | Draft from the current staged snapshot, editable before committing |
| AI diff explanation | Explain the selected diff on explicit request |
| Provider setup | One concise connection screen, remember the selected provider locally, useful disconnected/error states |

Stash mutation and branch/network workflows follow the initial read-only release slices; nothing is implemented simply because it is in this target.

## Later candidates

Hunk staging, worktree creation, selective cherry-pick/revert, simple merge assistance, tag management, split diff, local AI model adapter, Windows and Linux packaging. Add only after evidence of daily need.

## Outside the initial scope

Issue trackers, hosted pull-request review, team accounts, cloud patch services, general AI chat, autonomous agents changing a repository, interactive rebase UI, force push, hard reset, repository cleanup and an embedded general terminal.

## End-to-end acceptance scenarios

- Open a linked worktree and identify its actual HEAD, index and branch; another worktree stays unchanged.
- Follow a branch through a merge without false connections; graph and commit rows remain aligned while scrolling.
- Stage one of three files, commit only its staged content and retain the other files.
- Write a message manually without configuring AI.
- Request an AI draft, inspect the input scope, connect a provider if needed, review/edit the suggestion and commit manually.
- Change repository or staged content while AI runs; the old response cannot replace the new draft.
- Inspect outgoing context and perform a normal push; an authentication failure retains work and reports the relevant remote.
- An empty repository, detached HEAD, dirty tree, conflict or missing upstream is visibly different from a normal ready state.


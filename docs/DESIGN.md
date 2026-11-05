# Design guidelines

## Visual target

Use the first displayed concept, revised with contextual AI: [selected-ai.png](design/mockups/selected-ai.png). The other two original concepts are archived exploration, not alternative implementation targets.

The visual model is a graphite workbench: repository tabs above, branches/worktrees on the left, history in the center, a contextual inspector on the right. The inspector can show a selected commit or the local Changes workflow. Keep the graph usable when the inspector opens.

AI belongs next to the task it helps. A secondary action near the message composer drafts a message; a quiet action in the diff header explains that diff. Provider setup lives in one small settings surface.

## Provisional visual tokens

| Token | Value / intent |
| --- | --- |
| Canvas | #171B20 |
| Surface | #1D232A |
| Elevated surface | #242D36 |
| Primary text | #E8EDF3 |
| Secondary text | #A6B1C0 |
| Accent | #7BD8AC, dark text on filled action |
| Secondary graph lanes | Muted blue and amber; stable mapping |
| Border | #35414D |
| Spacing | 4, 8, 12, 16, 24, 32 px |
| Typography | System sans initially, 14 px body / 16 px inspector title |
| Code and hashes | System monospace, 13–14 px |
| Radius | 3 px shared buttons/inputs, 2 px quiet navigation rows, 6 px dialogs; no nested card layout |
| Row height | 36–40 px initially; density may be tuned after actual use |

These values are starting design tokens, not a measured or accessibility-certified design system. Check actual rendered contrast, focus, scaling and readability during implementation. Bundle fonts only if their licenses and benefit justify it.

## Layout and resizing

Design viewport: approximately 1440 × 1024. The generated images may have slightly different raster dimensions; do not stretch them to infer CSS measurements.

- Top tabs: 48–52 px. Command bar: about 48 px. Status bar: 28–32 px.
- Sidebar: about 220 px, resizable. Inspector: about 360 px, resizable.
- Main history fills remaining space with a virtualized table.
- Horizontal and vertical resizing must preserve graph/row alignment.
- At narrower desktop sizes, collapse the sidebar and show the inspector as a selectable drawer.
- At 200% text zoom, retain reachable controls and readable content; allow scrolling where necessary.
- No mobile app is part of the first release.

## Interaction hierarchy

| Location | Primary information | Actions |
| --- | --- | --- |
| Repository strip | Active repository | Switch / open repository |
| Command bar | Branch, upstream and ahead/behind | Fetch, Pull, Push |
| Sidebar | Branches, remotes, worktrees, stashes | Navigate; contextual actions on demand |
| History | Topology and commit subject | Select commit, filter scope, search |
| Commit inspector | Metadata, files and diff | Select file, explain diff |
| Changes inspector | Staged vs unstaged files, diff, message | Stage, draft with AI, edit, commit |
| AI settings | Chosen provider and connection state | Connect, change provider, disconnect |

Push is not styled as the primary action while composing a local commit. Context switching never silently applies a draft or pending action to another repository.

## Copy and states

- Prefer concrete labels: “Commit 2 files”, “Write with AI”, “Explain diff”, “Connect provider”.
- “Commit” creates local history; “Push” sends commits to the shown remote.
- A count of changed files is different from a count of outgoing commits.
- Never use “Sync” to hide a sequence of fetch, merge/rebase and push.
- Show loading, empty, disconnected, unauthorized, rate-limited, oversized-input, conflict and stale-result states.
- AI errors leave the manual editor usable. Regeneration does not erase manually edited text.
- Do not show a successful connection until the backend verifies it.

## Accessibility and restraint

Use semantic controls, visible keyboard focus, named icon buttons and predictable tab order. Keep search and primary actions keyboard reachable. Selection and graph meaning must not depend only on color; include labels, row focus and branch names. Support reduced motion. Use one consistent licensed icon set rather than a mixture of custom pictograms.

Generated graphs, line numbers, code and control states are illustrative. The implementation must follow actual Git data and interaction rules even when that requires correcting a mockup detail.


## Clone simplicity revision — 2026-11-05

Preserve graphite/mint and the system typography. Provider choices are quiet
icon-and-label rows, not separate outlined cards. Keep labels: icons support
recognition without making users guess. Main content asks for repository and
destination, with one review action. Destination folder/name belong together.
GitHub browsing and connection options use explicit disclosure; neither loads
network data automatically. Plain explanations replace redundant endpoint/code
blocks in the ordinary path. Full URLs remain available in connection details
and the source/destination review. Keep the trust boundary and failed-input text.

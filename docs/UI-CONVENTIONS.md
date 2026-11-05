# Workbench interaction conventions

Accepted first pass, 2026-11-05. Preserve the graphite/mint visual target and
three-column navigation/graph/inspection layout. Use these conventions as new
Git features are added; incomplete future flows are not implied implementations.

- Single-clicking a branch inspects its tip. Double-click checks out a local
  branch directly; the contextual Checkout command and branch switcher do the
  same without an extra confirmation. Remote refs retain the local tracking form.
  Merge and delete keep their separate reviews.
- Ask for repository trust once at opening, before the first Git write. Declining
  keeps inspection available. Accepted trust lasts while that repository remains
  open; tab/branch switches do not prompt again. Closing/reopening or restarting
  requires a new decision. Trust is not persisted to disk.
- An object's action menu has the same policy in the sidebar and graph. Do not
  create competing hover merge/delete buttons for each row.
- History and Changes are modes. Compare revisions is a secondary action. Show
  current working branch globally; do not label an unrelated selected commit as
  belonging to that working branch in the inspector.
- Keep primary transport commands visible. Call the no-upstream action Publish;
  call it Push when an upstream exists. Descriptions must state target/effect.
- Sidebar groups can collapse. Empty remote/stash groups begin collapsed. Search
  narrows branch navigation; the graph search stays separate and clearly scoped.
- Drag starts after a deliberate movement threshold. Sidebar and graph share one
  session and review policy. Commit targets compare immutable SHAs; the current
  local branch target reviews merge/cherry-pick. Drag must not also trigger a row
  click, change branch implicitly or execute Git before confirmation.
- Every pointer-only shortcut needs a keyboard/menu alternative. Preserve visible
  focus and accessible names/expanded/pressed states. Never rely on color alone.
- Prefer context actions near their object to extra global toolbars. Keep provider
  setup in the existing toolbar entry; contextual AI remains near its task.
- Preserve edited text and Git working/index state through inspection, cancellation
  and errors. Native Git guards remain authoritative; UI simplification cannot
  silently remove checks or broaden AI input.

Next UX work: file menus and clearer Changes staging hierarchy, command palette,
search/Hide/Solo and richer drag targets/auto-scroll. Implement real guarded actions
before exposing new menu labels. Validate the narrow 960px window, keyboard,
large branch counts and non-empty remote/stash cases as their slices are expanded.

Clone forms use a compact icon-and-label provider list with square, quiet rows.
Repository and destination lead; optional GitHub browsing and connection settings
are collapsed until requested. Explain “clone” as making a local copy. Use ordinary
text for folder paths, reserving monospace for code/diffs/hashes. Shared controls
have3px corners; dialogs6px. Retain visible keyboard focus, accessible labels and
source/destination review; do not replace meaningful labels with unexplained icons.


### Desktop toolbar

Keep the tab strip and action buttons independent:48px chrome,30px top actions,
small consistent margins. Clone must never stretch to the tab strip height.
Group branch context, synchronization and local work using quiet separators.
Branch/Stash/Pop open existing operations; Pop always inspects the latest stash
before its explicit apply-and-remove review. Additional remotes/worktree/compare
actions use the shared pointer/keyboard menu. Disabled states reflect active
operations, conflicts, absent changes/stashes and unborn HEAD for Stash.
AI settings show requirements for the selected provider only, without hiding them
behind a second acknowledgement. Installation controls remain optional disclosure.
A trust prompt must preserve form text and must never submit that form automatically.

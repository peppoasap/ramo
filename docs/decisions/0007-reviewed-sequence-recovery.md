# 0007 — Reviewed cherry-pick and revert recovery

Status: accepted, local implementation (2026-11-03).

Expose Continue, Skip current commit and Abort for interrupted cherry-pick/revert operations. Use enumerated kinds and steps rather than arbitrary Git arguments. Rebase and unknown/mixed instructions remain explicitly external.

Continuing requires no conflicts and no unstaged tracked edits. It commits the reviewed index using Git's recorded message with `--no-edit`; no editor is launched. A remaining sequence may pause again. Empty resolutions are not dropped automatically: Git's error remains visible and skipping requires a new confirmation.

Include bounded sequencer todo/head/abort-safety/options in review fingerprints, in addition to current operation markers. A changed future step invalidates review. Skip and abort warn about discarded resolutions/sequence commits and delegate recovery to Git without forcing HEAD or cleaning unrelated files. History refreshes even when continuation creates one commit before pausing again.

Real Git fixtures cover continue/abort/skip, empty resolution, unstaged preservation, mismatched operations, changed todo, a second conflict and abort back to the pre-sequence state. Native acceptance is recorded separately.

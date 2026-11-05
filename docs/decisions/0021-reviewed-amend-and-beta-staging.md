# ADR 0021 — Conservative amend and portable beta staging

Status: accepted for local implementation, 2026-11-05.

Amend is a separate typed action and dialog, not a silent mode on Commit or its
keyboard shortcut. Native validation binds expected HEAD and snapshot, requires
session trust, attached non-merge HEAD, no operation/conflicts, a bounded nonempty
message and no reachability from locally known remote refs. Root commits are valid.
It rewrites the index without `--all` or resetting author; message-only correction
works. A new local `refs/ramo/amend/<old-oid>-<nonce>` is created before running
Git. Keep it on hook failure. No in-app reflog/restore/delete UI is claimed.

Remote reachability uses local refs; it cannot establish that the commit was never
shared. Fetch and human review remain necessary. A trusted hook/external process
can alter state; fingerprint validation is optimistic, not a Git/filesystem lock.
Recovery refs are local ordinary Git refs: standard active-branch push excludes
them, but an external mirror push could include them. They are not cloud backups.
The normal commit draft is untouched by amend; failed dialog text stays in memory,
without restart persistence. Changing repository/closing the dialog discards it.

Beta source staging uses an explicit allowlist instead of uploading the development
folder. It emits a manifest/checksum, refuses overwrites and adjusts only the
portable README/AGENTS entry point. Historical local verification files are omitted.
Version consistency is checked independently of tests. CI uploads packaged installers
and checksums, never raw app trees or a public release. Actual hosted CI, signing,
external-device acceptance and publication remain separate gates.

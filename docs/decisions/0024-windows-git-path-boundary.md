# ADR0024 — Native Windows paths at the Git boundary

Status: accepted,2026-11-05.

First hosted Windows fixtures exposed Git for Windows rejecting verbatim device
prefixes produced by fs::canonicalize for clone/worktree destinations. Git worktree
lists use slash paths while registered native repositories may retain verbatim paths.

Retain canonical native paths for filesystem identity and picker grants. Convert
only actual path arguments passed to clone/worktree Git commands: ordinary drive
and UNC paths are supported; other device namespaces are refused. Do not rewrite
URLs, ref names, messages or arbitrary Git arguments. Compare native slash/prefix
representations only at reviewed worktree membership/canonical/open-tab guards,
and execute against the original reviewed worktree record. Keep symlink, common
repository, dirty/ignored-data and stale fingerprint checks.

Fixtures choose their own core.autocrlf=false so expected byte preservation does
not depend on runner defaults; dedicated CRLF/EOF cases remain. Windows tests use
legal Unicode/bracket filenames and Unix retains newline filename coverage.
Two path regression tests and the complete127-test macOS backend suite pass.
Windows build/test results and GUI installation acceptance are separate gates.

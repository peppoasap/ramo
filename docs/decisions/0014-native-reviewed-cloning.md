# 0014: Clone only to a native-authorized new destination after source review

Status: accepted locally, 2026-11-03.

Cloning is independent of an already opened repository. A native folder picker grants an existing parent outside Git boundaries; a separate native picker grants local repository roots/bare sources. The renderer submits grant IDs and one portable folder component. Parent/source canonical paths are checked again before execution. Filesystem Git-boundary checks also refuse parents when broken Git configuration prevents discovery.

Prepare validates the source and creates a bounded in-memory review token containing immutable source, destination and transport kind. It creates no destination and contacts no remote. The final UI approval executes that token once. Exclusive directory creation refuses existing folders/symlinks, including collisions after review. A failed attempt consumes its review and receives no automatic retry or Ramo cleanup. Inspect partial output before choosing another attempt.

Typed network sources support HTTPS, ssh:// and explicit user@host:path syntax. Embedded HTTPS credentials, SSH passwords, query/fragment tokens, custom transports/helpers, raw local paths and option-like addresses are refused. Local paths come only from the picker. The clone process uses argument arrays, per-invocation protocol policy and a 120-second deadline; local clones allow file transport only, network clones allow HTTPS/SSH only. That policy is not persisted to the clone's configuration. Native local clones use --no-local/--no-hardlinks and do not initialize submodules.

The review explains that configured Git transports, credentials, global templates/hooks and checkout filters may execute. Cloning uses installed Git and existing authentication helpers without installing software or starting a login. The resulting checkout opens automatically but remains untrusted for later write/network actions. Cancellation and hosted authentication acceptance are separate unfinished flows.

These are optimistic checks against external filesystem changes, not an atomic filesystem transaction or a sandbox for trusted Git configuration. Native acceptance uses only a disposable local bare source; HTTPS review/URL validation is not proof of hosted clone/authentication support. See clone.json for exact evidence.

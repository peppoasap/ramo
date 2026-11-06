# ADR0028 — Native connection workflows reuse core grants and endpoint operations

Status: accepted implementation within owner-authorized native migration.
Date: 2026-11-06. Reuses ADR0014 and ADR0023; beta/storage remain unchanged.

Native UI sends typed connection jobs to the existing backend worker. Pickers
alone create parent/local-source grants; widgets pass grant IDs and one portable
folder name. Prepare returns a bounded immutable plan with no remote/file creation.
Final Clone consumes the plan once. Failed/partial attempts are not retried or
deleted automatically. Result opening uses the ordinary composer-flush path and
fresh trust. Dirty merge results must be saved/left before starting Clone. While
clone executes, opening/closing and duplicate mutations are blocked.

Connection replies have independent unique request IDs and nested errors, so a
closed/replaced dialog or Git epoch change cannot overwrite a different selection
or surface stale connection errors. Remote configuration still uses the shared
snapshot/trust-guarded ConfigureRemote action; only safe ordinary URLs are editable.
Remote Refresh updates the authoritative repository view before another review.

Provider shortcuts create validated ordinary URLs. GitHub catalog access is an
explicit read through installed gh, never automatic login/token import. No UI
connection claim is inferred from a valid URL. No additional protocol adapter or
cross-product permission is introduced. Source/destination/endpoint contracts
stay core-owned for UI and any future agent interface.

Local synthetic clone/config UI and preservation fixtures validate this slice.
Hosted authentication/catalog acceptance and Windows/Linux GUI remain separate.
Clone uses the existing120-second deadline, with pending UI but no Stop/streaming;
shallow/sparse, richer remotes/upstream/removal and other catalogs remain open.

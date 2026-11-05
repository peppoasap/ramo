# ADR0023 — Provider clone shortcuts, explicit catalog and remote endpoints

Status: accepted implementation slice, 2026-11-05.

Extend existing Clone rather than add a competing repository dashboard. Provider
shortcuts generate ordinary Git URLs and suggest a folder; native grants/review
and transport stay authoritative. Local picker and raw URL remain first-class.
GitHub account browsing is an explicit read via installed official gh, from a
non-repository temporary working directory with fixed GET endpoints and host/scope
validation. Return only bounded validated repository metadata, not CLI diagnostics
or tokens. No automatic login/install or new credential storage. Other services
are clone shortcuts until actual account adapters exist. No mock Connected state.

Remote configuration is a separate typed local write with snapshot/trust checks,
not a generic Git/config editor. List endpoints without returning embedded secrets;
add/change one ordinary HTTPS/SSH URL after review. Preserve explicit push URLs,
multi-URL and custom/local transport configurations by refusing edits. Endpoint
output is hashed into snapshot fingerprints to invalidate stale destination reviews.
Saving does not perform network operations. Removal/upstream controls are separate.

Live hosted credentials are not validated by fixture tests. Existing frozen beta
archives are unchanged; the new running app/source is a development revision.
Account OAuth registration, native credential storage, additional catalogs and
clone progress/cancel are future work, with no implicit authorization to publish.

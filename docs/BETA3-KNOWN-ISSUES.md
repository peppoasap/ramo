# Ramo beta.3 — macOS package superseded

**Mac users: use [beta.4](https://github.com/peppoasap/ramo/releases/tag/v0.2.0-beta.4)
instead.** Post-download inspection found that beta.3 has only a linker ad-hoc
signature, without a valid application bundle/resource seal. It fails strict
codesign verification. Beta.4 explicitly seals the bundle and verifies the mounted
DMG before publication; it still has no Developer ID or Apple notarization.

These beta.3 assets are retained unchanged for provenance, not recommended as the
current download. Windows/Linux builds and backend tests passed, but GUI install/
runtime acceptance remains outstanding. No stable-release or signing certification.

# Release runbook

Current beta line: **0.2.0-beta.4**. Stable-release validation is outstanding.
The 0.1.0 private tester archive is immutable. Never replace its installer or hashes.

## Local candidate preparation

```sh
npm ci
npm run check:release
npm test
npm run build
npm run test:git
npm run audit:dependencies
# Choose the native platform: dmg, nsis or deb.
npm run bundle -- --bundles dmg
```

Run the Git tests only against their generated disposable fixtures. No AI inference
is needed. Audit requires registry access; native platform dependencies are required.
On Windows use `powershell -NoProfile -File scripts/package-windows-local.ps1`
from the extracted source with the prerequisites in `docs/WINDOWS-BUILD.md`.
This path is prepared but has not run on a Windows host.
On macOS use `npm run package:mac:local` for the ad-hoc DMG, then verify its checksum,
image, mounted bundle signature, notices and actual version. Ad-hoc verification
is not Developer ID signing or notarization. Never reuse 0.1.0 evidence as proof
for the new candidate. Test install/open and the checklist on the exact artifact.

`npm run prepare:source` builds an explicitly selected portable source ZIP and
SHA256SUMS under `releases/candidates/<version>/`. It excludes dependencies, target,
provider CLIs/settings, historical verification media and repositories. It checks
home paths, common private key headers and selected credential patterns; this is
not a complete secret detector. Review the manifest and extracted contents before
publication. The portable README and AGENTS entry point are intentionally adapted.
No Git repository, remote, tag or GitHub Release is created by these commands.

The working folder contains local historical evidence and generated builds. Do
not publish it wholesale or rely on .gitignore to untrack existing files. Start
any later authorized public repository from the reviewed portable source snapshot.
Run checks again in its extracted root; `.github/workflows/check.yml` assumes this
root layout. Retain an exact commit ID, manifest, build log and installer checksums.

## Public beta publication

The owner authorized public source and beta builds at peppoasap/ramo. CI packages
DMG (macOS), NSIS (Windows) and DEB (Linux). RPM is not part of this beta because
the upstream bundler rejects the prerelease version syntax. AppImage/MSI formats
are not promised by this workflow. Main-branch
pushes run checks and upload short-lived workflow artifacts. After reviewing source and version, pushing a version-matching v<version> beta tag is an explicit
publication action: CI rebuilds all targets and publishes a GitHub prerelease with
installers and combined SHA256SUMS only after every platform job passes. Existing
releases/assets are not replaced. A failing target blocks this automated release.
Do not push a beta tag unless public publication is authorized. Never mark it stable
or infer GUI runtime acceptance from a green build. There is no automatic updater.

## Further release preparation

1. Confirm owner/repository, working name, maintainer identity, Apache notices and
   a real private security reporting channel. Update SECURITY.md; no fake contact.
2. Review and import only the exported source. Configure branch protection and
   required checks. Run the existing macOS/Windows/Linux matrix; configured jobs
   are not evidence until they execute. Rust is pinned to the declared minimum.
3. CI has read-only repository permissions. It uploads packaged installers and
   SHA256SUMS as workflow artifacts, never publishes a release or uploads raw .app
   directories. CI artifacts are unsigned candidates; OS architecture and build
   log must accompany them. Upload/download checksums do not verify OS trust.
4. Complete `BETA-CHECKLIST.md` for each supported target. Only advertise targets
   with install/runtime acceptance. A macOS-only beta need not wait for other
   platforms, provided they are clearly marked unsupported/experimental.
5. Freeze versions in npm/package lock, Cargo/package lock and Tauri config using
   one prerelease version. Update the changelog date only when actually released;
   attach exact checksums, notices, platform/signing status and known limitations.
6. After explicit release authorization, create a **draft**, mark **pre-release**,
   attach reviewed installers/source/checksums, and inspect it before publishing.
   Do not mark it the stable/latest release. Publication is a separate action.

No automatic updater is configured. A subsequent beta uses a new version and
new immutable artifacts. Never silently replace a tester's downloaded build.

References: [GitHub release management](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository),
[workflow artifacts](https://docs.github.com/en/actions/tutorials/store-and-share-data).

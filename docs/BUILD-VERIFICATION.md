# Published beta.4 build verification

Verified 2026-11-05. [Public prerelease](https://github.com/peppoasap/ramo/releases/tag/v0.2.0-beta.4).
Source tag `v0.2.0-beta.4`: `f0cd98eedb69f9c7133c53c3d5422932b677022c`.
[Hosted CI run 42084594499](https://github.com/peppoasap/ramo/actions/runs/42084594499)
completed successfully on all three platforms, including publication.

| Target | Frontend tests | Backend tests | Dependency notices | Published format |
| --- | ---: | ---: | ---: | --- |
| macOS Apple Silicon | 60 | 127 | 352 | DMG |
| Windows x64 | 60 | 110 | 355 | NSIS setup.exe |
| Linux amd64 | 60 | 127 | 442 | DEB |

Each job passed production build, metadata checks, npm audit, dependency-notice
generation and native packaging. Backend counts are 27 unit/transport, 94 real-Git
and 6 process tests on macOS/Linux; Windows runs 24 unit/transport and 86 real-Git
checks. Unix-specific cases are excluded on Windows, not silently counted as passed.
An audit result describes known advisories at that time, not a security certification.

## Published artifact checks

All three installers were downloaded anonymously from the public release and
matched SHA256SUMS. The DMG was mounted read-only: the actual app passed strict
`codesign --verify --deep --strict`, version, arm64 executable, notice/resource
and Applications-shortcut checks. CI also mounts and checks its final DMG before
artifact upload. Ad-hoc signing establishes bundle integrity, not publisher identity.
The Windows installer has an ordinary PE32 NSIS bootstrap; CI builds its application
for x86_64-pc-windows-msvc. The DEB metadata identifies version beta.4 and amd64.

| Asset | Bytes | SHA-256 |
| --- | ---: | --- |
| Ramo_0.2.0-beta.4_aarch64.dmg | 5768018 | `eee13cd59492086d83e6ef2b7a013b45937427a6a6456ca830704622dbe011cc` |
| Ramo_0.2.0-beta.4_x64-setup.exe | 4078574 | `dcc45962857784bcad44454a156e679d6cac8366fba66bb0783ef06c5a361229` |
| Ramo_0.2.0-beta.4_amd64.deb | 7476210 | `a9493e0f6e6705c66b4d5c0e80832fb82f017a2a4bb94b00d2147412b0c6b985` |

## Outstanding acceptance

- macOS Developer ID/notarization and a downloaded first launch on a second Mac.
- Windows publisher signing and actual GUI installation/workflow acceptance.
- Linux GUI installation/workflow acceptance; Intel Mac build/runtime acceptance.
- Broader hosted credential UX, large-repository behavior and restart/recovery checks.

Earlier synthetic native macOS workflow checks are recorded in STATUS.md. They do
not establish acceptance of the downloaded Windows/Linux installers. No live AI
inference was run for this release, and no GitKraken speed advantage was measured.

The beta.3 Mac download failed post-download strict bundle-signature verification.
Its assets remain unchanged for provenance; its notes now point to beta.4. Use
beta.4 for current downloads. No automatic updater is configured.

#!/bin/bash
# Local ad-hoc package only. Does not publish, notarize or use signing accounts.
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "$(uname -s)" != Darwin ]]; then
  echo 'This packaging command requires macOS.' >&2
  exit 1
fi
npm run bundle -- --bundles app
version="$(node -p "JSON.parse(require('fs').readFileSync('package.json','utf8')).version")"
architecture="$(uname -m)"
output="$PWD/releases/local/$version"
bundle="$PWD/src-tauri/target/release/bundle/macos/Ramo.app"
staging="$(mktemp -d "${TMPDIR:-/tmp}/ramo-package.XXXXXX")"
trap 'rm -rf "$staging"' EXIT
mkdir -p "$output"
codesign --force --deep --sign - "$bundle"
codesign --verify --deep --strict "$bundle"
mkdir "$staging/payload"
ditto "$bundle" "$staging/payload/Ramo.app"
ln -s /Applications "$staging/payload/Applications"
filename="Ramo_${version}_macos-${architecture}.dmg"
# Verify a new image before replacing the last installer.
hdiutil create -volname Ramo -fs HFS+ -srcfolder "$staging/payload" -format UDZO "$staging/installer.dmg"
hdiutil verify "$staging/installer.dmg"
mv "$staging/installer.dmg" "$output/$filename"
(cd "$output" && shasum -a 256 "$filename" > SHA256SUMS)
echo "Local ad-hoc package: $output/$filename"
echo 'No Developer ID signature or Apple notarization was performed.'

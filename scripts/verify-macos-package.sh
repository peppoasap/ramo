#!/bin/bash
# Verify actual DMG payload, not only a successful build or the source .app.
set -euo pipefail
cd "$(dirname "$0")/.."
[[ "$(uname -s)" == Darwin ]] || { echo 'Requires macOS.' >&2; exit 1; }
version="$(node -p "require('./package.json').version")"
if [[ $# -eq 1 ]]; then
  image="$1"
else
  images=(src-tauri/target/release/bundle/dmg/Ramo_"$version"_*.dmg)
  [[ ${#images[@]} -eq 1 && -f "${images[0]}" ]] || { echo 'Expected exactly one version-matching DMG.' >&2; exit 1; }
  image="${images[0]}"
fi
hdiutil verify "$image"
mount="$(mktemp -d "${TMPDIR:-/tmp}/ramo-verify.XXXXXX")"
mounted=0
cleanup() {
  if [[ "$mounted" -eq 1 ]]; then hdiutil detach "$mount"; fi
  rmdir "$mount"
}
trap cleanup EXIT
hdiutil attach -readonly -nobrowse -mountpoint "$mount" "$image"
mounted=1
app="$mount/Ramo.app"
codesign --verify --deep --strict "$app"
actual="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$app/Contents/Info.plist")"
[[ "$actual" == "$version" ]] || { echo 'DMG version differs from source.' >&2; exit 1; }
for resource in LICENSE NOTICE THIRD_PARTY_NOTICES.txt; do
  [[ -s "$app/Contents/Resources/$resource" ]] || { echo "Missing bundled $resource" >&2; exit 1; }
done
echo "Verified mounted Ramo $version: strict bundle signature, version and notices."
echo 'Ad-hoc signing is not Developer ID authentication or Apple notarization.'

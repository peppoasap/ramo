#!/usr/bin/env bash
# Local development bundle only; no installation, release, AI login or inference.
set -euo pipefail
if [[ "$(uname -s)" != Darwin ]]; then
  echo 'Use cargo run --manifest-path apps/native/Cargo.toml on this platform.' >&2
  exit 1
fi
ramo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ramo_root"
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
cargo build --manifest-path apps/native/Cargo.toml --locked
python3 scripts/native-notices.py
python3 - <<'PY'
from pathlib import Path
import os, plistlib, shutil
bundle=Path('.tools/native-dev/Ramo Native.app/Contents')
(bundle/'MacOS').mkdir(parents=True,exist_ok=True)
(bundle/'Resources').mkdir(exist_ok=True)
pending=bundle/'MacOS/ramo-native.new'
shutil.copy2('apps/native/target/debug/ramo-native',pending)
os.replace(pending,bundle/'MacOS/ramo-native')
info={'CFBundleExecutable':'ramo-native','CFBundleIdentifier':'org.ramo.native.dev',
      'CFBundleName':'Ramo Native','CFBundleDisplayName':'Ramo Native',
      'CFBundleShortVersionString':'0.3.0-dev.1','CFBundleVersion':'1',
      'CFBundlePackageType':'APPL','NSHighResolutionCapable':True,
      'LSMinimumSystemVersion':'11.0','CFBundleIconFile':'icon.icns'}
(bundle/'Info.plist').write_bytes(plistlib.dumps(info))
for source in ['LICENSE','NOTICE','apps/native/THIRD_PARTY_NOTICES.txt','src-tauri/icons/icon.icns']:
    shutil.copy2(source,bundle/'Resources'/Path(source).name)
PY
codesign --force --sign - '.tools/native-dev/Ramo Native.app'
codesign --verify --deep --strict '.tools/native-dev/Ramo Native.app'
open -n '.tools/native-dev/Ramo Native.app' --args "$@"

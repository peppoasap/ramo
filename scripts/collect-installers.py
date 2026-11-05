#!/usr/bin/env python3
"""Collect existing packaged CI installers and checksums; no publication/signing."""
from pathlib import Path
import hashlib, shutil
root = Path(__file__).resolve().parents[1]
bundle = root/'src-tauri/target/release/bundle'
out = root/'releases/ci'
if out.exists(): raise RuntimeError('CI artifact output already exists')
files = sorted(p for p in bundle.rglob('*') if p.is_file() and p.suffix in {'.dmg','.exe','.msi','.deb','.AppImage'})
if not files: raise RuntimeError('No packaged installers found')
if len({p.name for p in files}) != len(files): raise RuntimeError('Ambiguous installer names')
out.mkdir(parents=True)
checks=[]
for p in files:
    if p.is_symlink(): raise RuntimeError('Installer symlink refused')
    shutil.copyfile(p,out/p.name)
    checks.append(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.name)
(out/'SHA256SUMS').write_text('\n'.join(checks)+'\n')
print(f'Collected {len(files)} packaged installer(s); unsigned CI candidates, not a release')

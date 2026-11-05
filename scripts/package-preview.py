#!/usr/bin/env python3
"""Freeze a verified local DMG, portable source snapshot and tester documents.
No dependency installation, signing account, publication or user settings access.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
VERSION = json.loads((ROOT / 'package.json').read_text())['version']
LABEL = f'Ramo-{VERSION}-preview-macos-arm64'
OUTPUT = ROOT / 'releases' / 'preview' / VERSION
DMG_NAME = f'Ramo_{VERSION}_macos-arm64.dmg'
DMG = ROOT / 'releases' / 'local' / VERSION / DMG_NAME

def digest(data):
    return hashlib.sha256(data).hexdigest()

def archive(source, destination):
    with zipfile.ZipFile(destination, 'w', zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        for path in sorted(source.rglob('*')):
            if path.is_symlink():
                raise RuntimeError(f'Symlink not permitted: {path.relative_to(source)}')
            if path.is_file():
                z.write(path, source.name + '/' + path.relative_to(source).as_posix())
    with zipfile.ZipFile(destination) as z:
        if z.testzip():
            raise RuntimeError('Archive integrity failed')

def manifest(folder):
    return {p.relative_to(folder).as_posix(): digest(p.read_bytes())
            for p in sorted(folder.rglob('*')) if p.is_file()}

def main():
    expected = json.loads((ROOT / 'docs/verification/local-build.json').read_text())['sha256']
    if digest(DMG.read_bytes()) != expected:
        raise RuntimeError('Installer differs from verified local build record')
    if OUTPUT.exists():
        raise RuntimeError('Preview already exists; preserve the frozen package or choose a new version')
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='ramo-preview-') as temporary:
        scratch = Path(temporary)
        bundle = scratch / LABEL
        bundle.mkdir()
        source = scratch / f'Ramo-{VERSION}-source'
        source.mkdir()
        # Deliberate allowlist: no broad archive of the workspace.
        directories = ['src', 'scripts', 'assets', '.github', 'src-tauri/src',
                       'src-tauri/tests', 'src-tauri/examples', 'src-tauri/icons',
                       'src-tauri/capabilities', 'docs/decisions', 'docs/distribution']
        files = ['package.json', 'package-lock.json', '.npmrc', 'tsconfig.json',
                 'vite.config.ts', 'index.html', 'LICENSE', 'NOTICE',
                 'THIRD_PARTY_NOTICES.txt', 'AGENTS.md', 'STATUS.md',
                 'SECURITY.md', 'CONTRIBUTING.md', 'src-tauri/Cargo.toml',
                 'src-tauri/Cargo.lock', 'src-tauri/build.rs', 'src-tauri/tauri.conf.json',
                 'docs/ARCHITECTURE.md', 'docs/FEATURES.md', 'docs/AI.md',
                 'docs/PRODUCT.md', 'docs/DESIGN.md', 'docs/ROADMAP.md',
                 'docs/verification/delivery-performance.json',
                 'docs/verification/delivery-performance-initial.json',
                 'docs/verification/delivery-performance-repeat.json',
                 'docs/verification/delivery-history-render.json',
                 'docs/verification/delivery-environment.json',
                 'docs/verification/distribution-footprint.json',
                 'docs/verification/preview-validation.json']
        extensions = {'.ts','.tsx','.css','.json','.rs','.py','.mjs','.sh','.md',
                      '.png','.icns','.ico','.svg','.yml','.yaml','.html','.txt'}
        for directory in directories:
            files += [p.relative_to(ROOT).as_posix() for p in (ROOT/directory).rglob('*')
                      if p.is_file() and p.suffix in extensions and not any(part in {'__pycache__','node_modules','target','.git','.tools'} for part in p.parts)]
        for relative in sorted(set(files)):
            path = ROOT / relative
            if path.is_symlink():
                raise RuntimeError(f'Unexpected source symlink: {relative}')
            data = path.read_bytes()
            if path.suffix not in {'.png','.icns','.ico'}:
                if re.search(rb'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|AKIA[A-Z0-9]{16}', data):
                    raise RuntimeError(f'Credential-like content in {relative}; review before export')
                private_home = str(Path.home()).encode()
                if private_home + b'/' in data:
                    raise RuntimeError(f'Personal absolute path in {relative}; review before export')
            target = source / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            shutil.copymode(path, target)
        (source/'README.md').write_bytes((ROOT/'docs/distribution/README-source.md').read_bytes())
        (source/'WORKSPACE_GUIDELINES.md').write_bytes((ROOT.parent/'AGENTS.md').read_bytes())
        agents = (source/'AGENTS.md').read_text().replace('../AGENTS.md', 'WORKSPACE_GUIDELINES.md')
        (source/'AGENTS.md').write_text(agents)
        (source/'.gitignore').write_text('node_modules/\ndist/\nsrc-tauri/target/\nsrc-tauri/gen/\nreleases/\n.DS_Store\n')
        (source/'docs/verification/local-build.json').write_text(json.dumps({'version': VERSION, 'sha256': expected, 'installerBytes': DMG.stat().st_size, 'scope': 'Portable artifact hash excerpt; historical local paths omitted'},indent=2)+'\n')
        (source/'EXPORT-METADATA.json').write_text(json.dumps({
            'version': VERSION, 'status': 'private preview source snapshot; no Git commit or public release',
            'installerSha256': expected,
            'portableEdits': ['README.md', 'AGENTS.md entry-point reference', 'docs/verification/local-build.json portable hash excerpt'],
            'excluded': ['dependencies', 'build caches', 'local settings', 'AI CLI installations',
                         'credentials', 'repositories', 'design reference images', 'old verification captures']
        }, indent=2)+'\n')
        (source/'MANIFEST-SHA256.json').write_text(json.dumps(manifest(source),indent=2)+'\n')
        source_zip = bundle / f'Ramo-{VERSION}-source.zip'
        archive(source, source_zip)
        shutil.copyfile(DMG, bundle / DMG_NAME)
        for name in ['LEGGIMI-PRIMA.txt','RELEASE-NOTES.md','FEEDBACK.md',
                     'REPORT-PRESTAZIONI.md','REPORT-PRESTAZIONI.html']:
            shutil.copyfile(ROOT/'docs/distribution'/name, bundle/name)
        for name in ['LICENSE','NOTICE','THIRD_PARTY_NOTICES.txt']:
            shutil.copyfile(ROOT/name,bundle/name)
        evidence = bundle/'benchmark-data'
        evidence.mkdir()
        for name in ['delivery-performance.json','delivery-performance-initial.json',
                     'delivery-performance-repeat.json','delivery-history-render.json',
                     'delivery-environment.json','distribution-footprint.json','preview-validation.json']:
            shutil.copyfile(ROOT/'docs/verification'/name,evidence/name)
        checksums = manifest(bundle)
        (bundle/'SHA256SUMS').write_text(''.join(f'{value}  {name}\n' for name,value in checksums.items()))
        result = scratch/f'{LABEL}.zip'
        archive(bundle,result)
        result_hash=digest(result.read_bytes())
        ready=scratch/'ready'
        ready.mkdir()
        shutil.copytree(bundle,ready/LABEL)
        shutil.move(str(result),ready/result.name)
        (ready/(result.name+'.sha256')).write_text(f'{result_hash}  {result.name}\n')
        os.replace(ready,OUTPUT)
        print(json.dumps({'archive':str(OUTPUT/(LABEL+'.zip')),'sha256':result_hash,
                          'bytes':(OUTPUT/(LABEL+'.zip')).stat().st_size,
                          'sourceFiles':len(manifest(source))},indent=2))

if __name__ == '__main__':
    main()

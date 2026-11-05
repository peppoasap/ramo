#!/usr/bin/env python3
"""Stage portable candidate source only. Never publishes, initializes Git or copies local state."""
from pathlib import Path
import hashlib, json, re, shutil, tempfile, zipfile
ROOT = Path(__file__).resolve().parents[1]
VERSION = json.loads((ROOT / 'package.json').read_text())['version']
DEST = ROOT / 'releases' / 'candidates' / VERSION
LABEL = f'Ramo-{VERSION}-source'

def main():
    if DEST.exists():
        raise RuntimeError('Candidate already exists. Preserve it or explicitly move it aside for a new check.')
    dirs = ['src','assets','licenses','scripts','.github','src-tauri/src','src-tauri/tests',
            'src-tauri/examples','src-tauri/icons','src-tauri/capabilities','docs/decisions',
            'docs/design','docs/images','apps/native/src','apps/native/fixtures']
    files = ['README.md','WORKSPACE_GUIDELINES.md','.nvmrc','.gitattributes','package.json','package-lock.json','.npmrc','.gitignore','tsconfig.json','vite.config.ts',
             'index.html','LICENSE','NOTICE','THIRD_PARTY_NOTICES.txt','AGENTS.md','STATUS.md',
             'SECURITY.md','CONTRIBUTING.md','CHANGELOG.md','src-tauri/Cargo.toml',
             'src-tauri/Cargo.lock','src-tauri/build.rs','src-tauri/tauri.conf.json',
             'apps/native/Cargo.toml','apps/native/Cargo.lock','apps/native/README.md']
    files += [p.relative_to(ROOT).as_posix() for p in (ROOT/'docs').glob('*.md')]
    allowed = {'.ts','.tsx','.css','.json','.rs','.py','.mjs','.sh','.md','.png','.jpg','.icns','.ico','.svg','.yml','.yaml','.txt','.ps1','.html'}
    for directory in dirs:
        files += [p.relative_to(ROOT).as_posix() for p in (ROOT/directory).rglob('*')
                  if p.is_file() and p.suffix in allowed and not any(x in {'node_modules','target','__pycache__','.git'} for x in p.parts)]
    with tempfile.TemporaryDirectory(prefix='ramo-beta-source-') as temp:
        stage = Path(temp)/LABEL; stage.mkdir()
        for name in sorted(set(files)):
            p = ROOT/name
            if p.is_symlink(): raise RuntimeError(f'Symlink refused: {name}')
            data = p.read_bytes()
            if p.suffix not in {'.png','.jpg','.ico','.icns'}:
                if re.search(rb'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|AKIA[A-Z0-9]{16}', data):
                    raise RuntimeError(f'Credential-like content: {name}')
                if str(Path.home()).encode()+b'/' in data:
                    raise RuntimeError(f'Personal home path: {name}')
            out = stage/name; out.parent.mkdir(parents=True,exist_ok=True)
            out.write_bytes(data); shutil.copymode(p,out)
        manifest={p.relative_to(stage).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(stage.rglob('*')) if p.is_file()}
        (stage/'MANIFEST-SHA256.json').write_text(json.dumps(manifest,indent=2)+'\n')
        archive=Path(temp)/(LABEL+'.zip')
        with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED,compresslevel=9) as z:
            for p in sorted(stage.rglob('*')):
                if p.is_file(): z.write(p,LABEL+'/'+p.relative_to(stage).as_posix())
        with zipfile.ZipFile(archive) as z:
            if z.testzip(): raise RuntimeError('ZIP integrity failure')
        DEST.mkdir(parents=True)
        shutil.copyfile(archive, DEST/archive.name)
        digest=hashlib.sha256(archive.read_bytes()).hexdigest()
        (DEST/'SHA256SUMS').write_text(f'{digest}  {archive.name}\n')
        print(json.dumps({'version':VERSION,'archive':str(DEST/archive.name),'files':len(manifest)+1,'sha256':digest},indent=2))

if __name__ == '__main__': main()

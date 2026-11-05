#!/usr/bin/env python3
"""Collect exact Rust-native dependency license texts; fail rather than omit one."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import hashlib

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--fetch', action='store_true', help='Allow Cargo metadata to fetch missing sources')
parser.add_argument('--target', help='Rust target triple; defaults to host')
parser.add_argument('--check', action='store_true', help='Check coverage without writing an inventory')
args = parser.parse_args()
root = Path(__file__).resolve().parent.parent
cargo = os.environ.get('CARGO', 'cargo')
host = subprocess.check_output(['rustc', '-vV'], text=True)
target = args.target or re.search(r'^host: (.+)$', host, re.M)[1]
command = [cargo, 'metadata', '--manifest-path', str(root/'apps/native/Cargo.toml'),
           '--format-version', '1', '--locked', '--filter-platform', target]
if not args.fetch:
    command.append('--offline')
metadata = json.loads(subprocess.check_output(command))
# Include the metadata package superset conservatively, including build inputs.
sections = []
missing = []
native_licenses = root/'licenses/native'
supplements = {(p['crate'], p['version']): p for p in json.loads((native_licenses/'manifest.json').read_text(encoding='utf-8'))}
for package in sorted(metadata['packages'], key=lambda p: (p['name'], p['version'])):
    if not package['source']:
        continue
    folder = Path(package['manifest_path']).parent
    paths = [p for p in folder.iterdir() if p.is_file() and
             re.match(r'^(licen[sc]e|copying|notice)([-_.]|$)', p.name, re.I)]
    if package.get('license_file') and (folder/package['license_file']).is_file():
        paths.append(folder/package['license_file'])
    supplemental = supplements.get((package['name'], package['version']))
    if not paths and supplemental:
        path = native_licenses/supplemental['file']
        if hashlib.sha256(path.read_bytes()).hexdigest() != supplemental['sha256']:
            raise SystemExit('Supplemental license hash mismatch: '+path.name)
        paths.append(path)
    if not paths and (package['name'].startswith('objc2') or package['name'] in ('objc-sys','block2','dispatch2')):
        paths.append(root/'licenses/objc2-MIT.txt')
    extra_text = ''
    # These exact old crate releases declare MIT but ship no license file, and
    # neither does their pinned upstream revision. Keep that provenance explicit.
    if not paths and (package['name'],package['version']) in {('block','0.1.6'),('dispatch','0.2.0'),('malloc_buf','0.0.6')}:
        assert package['license'] == 'MIT'
        assert 'Steven Sheldon' in package['authors']
        terms = (native_licenses/'MIT.txt').read_text(encoding='utf-8')
        terms = terms[terms.index('Permission is hereby granted'):]
        extra_text = ('License provenance: MIT declaration and author attribution from the exact crate manifest;\n'
                      'no upstream license file was present. Canonical SPDX MIT terms follow.\n'
                      'Copyright (c) Steven Sheldon\n\n'+terms)
    if not paths and package['name']=='hexf-parse' and package['version']=='0.2.1':
        assert package['license']=='CC0-1.0'
        paths.append(native_licenses/'CC0-1.0.txt')
        extra_text = 'License provenance: crate declares CC0-1.0; canonical SPDX terms supplied.\n'
    if not paths and not extra_text:
        missing.append(f"{package['name']}@{package['version']}")
    texts = extra_text+'\n\n'+'\n\n'.join(f"--- {p.name} ---\n{p.read_text(encoding='utf-8')}" for p in sorted(set(paths)))
    if package['name']=='epaint_default_fonts':
        texts += '\n\nBundled font licenses (not relicensed as Rust code):\n'+ '\n\n'.join(
            f"--- fonts/{p.name} ---\n{p.read_text(encoding='utf-8')}" for p in sorted((folder/'fonts').glob('*.txt')))
    sections.append(f"{package['name']} {package['version']}\nLicense: {package.get('license')}\n"
                    f"Source: https://crates.io/api/v1/crates/{package['name']}/{package['version']}/download\n{texts}")
sections.append('Phosphor icon font\nSource: https://github.com/phosphor-icons/web/blob/master/LICENSE\n'+
                (native_licenses/'Phosphor-icons-MIT.txt').read_text(encoding='utf-8'))
if missing:
    raise SystemExit('Missing native license texts: '+', '.join(missing))
output = root/'apps/native/THIRD_PARTY_NOTICES.txt'
if not args.check:
    output.write_text(f'Ramo Native dependency notices\nTarget: {target}\n\n'
                  'Original Ramo code is Apache-2.0. Dependencies retain their own licenses.\n'
                  'Conservative native/build-input inventory; no JavaScript dependencies.\n'
                  'Exact unmodified dependency source is available at the URLs below.\n\n'+
                      ('\n\n'+'='*72+'\n\n').join(sections)+'\n', encoding='utf-8')
print(f'Collected {len(sections)} native dependency notices for {target}.')

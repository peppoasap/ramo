#!/usr/bin/env python3
"""Disposable stash inspection/pop/drop fixture; never uses personal repositories."""
import argparse
from pathlib import Path
import subprocess
import tempfile
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--conflict',action='store_true')
options=parser.parse_args()
root=Path(tempfile.mkdtemp(prefix='ramo-stash-')).resolve()/('meadow-stash-conflict' if options.conflict else 'meadow-stashes')
root.mkdir()
def git(*args): return subprocess.check_output(['git','-C',str(root),*args],stderr=subprocess.STDOUT)
git('init','-b','main')
for key,value in [('user.name','Synthetic Contributor'),('user.email','synthetic@example.invalid'),('commit.gpgsign','false'),('core.fsmonitor','false'),('core.hooksPath',str(root/'.git/hooks'))]:git('config',key,value)
(root/'README.md').write_text('Synthetic committed base.\n');git('add','README.md');git('commit','-m','Initial synthetic stash base')
(root/'README.md').write_text('Older saved change.\n');git('stash','push','-m','Older synthetic stash')
(root/'README.md').write_text('Latest saved change.\n');(root/'notes.txt').write_text('Synthetic saved untracked note.\n');git('stash','push','-u','-m','Latest synthetic stash')
if options.conflict:
 (root/'README.md').write_text('Different committed current version.\n');git('add','README.md');git('commit','-m','Diverged synthetic current version')
print(root)

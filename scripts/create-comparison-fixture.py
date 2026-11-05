#!/usr/bin/env python3
"""Disposable diverged revisions plus dirty/index sentinels for read-only comparison."""
from pathlib import Path
import subprocess
import tempfile
import json
root=Path(tempfile.mkdtemp(prefix='ramo-compare-')).resolve()/'meadow-compare'
root.mkdir()
def git(*args):return subprocess.check_output(['git','-C',str(root),*args],stderr=subprocess.STDOUT).decode().strip()
git('init','-b','main')
for key,value in [('user.name','Synthetic Contributor'),('user.email','synthetic@example.invalid'),('commit.gpgsign','false'),('core.fsmonitor','false'),('core.hooksPath',str(root/'.git/hooks'))]:git('config',key,value)
(root/'README.md').write_text('Synthetic common base.\n');git('add','README.md');git('commit','-m','Synthetic common ancestor');ancestor=git('rev-parse','HEAD')
git('switch','-c','feature/compare');(root/'README.md').write_text('Synthetic committed target.\n');(root/'target-only.txt').write_text('Only the target branch adds this.\n');git('add','.');git('commit','-m','Target comparison revision');target=git('rev-parse','HEAD')
git('switch','main');(root/'base-only.txt').write_text('Only the base branch adds this.\n');git('add','.');git('commit','-m','Base comparison revision');base=git('rev-parse','HEAD')
(root/'README.md').write_text('Unstaged local sentinel; exclude from comparisons.\n');(root/'local-only.txt').write_text('Staged local sentinel; exclude from comparisons.\n');git('add','local-only.txt')
(root.parent/'fixture.json').write_text(json.dumps({'ancestor':ancestor,'base':base,'target':target,'indexTree':git('write-tree')},indent=2)+'\n')
print(root)

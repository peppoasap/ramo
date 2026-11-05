#!/usr/bin/env python3
"""Disposable bare source and outside-Git destination parent for native clone QA."""
from pathlib import Path
import subprocess
import tempfile
import json
base=Path(tempfile.mkdtemp(prefix='ramo-clone-')).resolve();source=base/'meadow-source';bare=base/'source.git';parent=base/'checkouts';source.mkdir();parent.mkdir()
def git(*args):return subprocess.check_output(['git','-C',str(source),*args],stderr=subprocess.STDOUT).decode().strip()
git('init','-b','main')
for key,value in [('user.name','Synthetic Contributor'),('user.email','synthetic@example.invalid'),('commit.gpgsign','false'),('core.fsmonitor','false'),('core.hooksPath',str(source/'.git/hooks'))]:git('config',key,value)
(source/'README.md').write_text('Synthetic clone fixture.\n');git('add','.');git('commit','-m','Initial synthetic clone');git('tag','v-fixture');git('branch','feature/example');git('clone','--bare',str(source),str(bare))
(source/'README.md').write_text('Unstaged source sentinel.\n');(source/'source-only.txt').write_text('Staged source sentinel.\n');git('add','source-only.txt')
record={'source':str(source),'bare':str(bare),'parent':str(parent),'head':git('rev-parse','HEAD'),'sourceIndexTree':git('write-tree')};(base/'fixture.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record,indent=2))

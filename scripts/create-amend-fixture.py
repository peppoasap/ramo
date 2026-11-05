#!/usr/bin/env python3
"""Create a disposable amend fixture; no network or personal repository access."""
from pathlib import Path
import json, subprocess, tempfile
root = Path(tempfile.mkdtemp(prefix='ramo-amend-'))/'meadow-amend'
root.mkdir()
def git(*args):
    return subprocess.check_output(['git','-C',str(root),*args],stderr=subprocess.DEVNULL).decode().strip()
git('init','-b','main')
git('config','user.name','Synthetic Contributor')
git('config','user.email','demo@example.invalid')
git('config','commit.gpgsign','false')
(root/'search.ts').write_text('export const search = "original";\n')
git('add','search.ts');git('commit','-m','Original synthetic commit\n\nKeep full body è.')
old=git('rev-parse','HEAD')
(root/'search.ts').write_text('export const search = "staged";\n');git('add','search.ts')
(root/'search.ts').write_text('export const search = "unstaged";\n')
(root/'notes.txt').write_text('Untracked sentinel.\n')
receipt={'path':str(root),'head':old,'indexTree':git('write-tree'),'status':git('status','--porcelain')}
(root.parent/'before.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt,indent=2))

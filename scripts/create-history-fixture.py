#!/usr/bin/env python3
"""Large disposable actual Git history for native virtual scrolling acceptance."""
from pathlib import Path
import subprocess
import tempfile
import json
root=Path(tempfile.mkdtemp(prefix='ramo-history-')).resolve()/'meadow-history'
root.mkdir()
def git(*args,input=None):
    return subprocess.check_output(['git','-C',str(root),*args],input=input,stderr=subprocess.STDOUT)
git('init','-b','main')
git('config','user.name','Synthetic Contributor')
git('config','user.email','synthetic@example.invalid')
git('config','commit.gpgsign','false')
git('config','core.fsmonitor','false')
stream=[]
def commit(ref,mark,message,parent=None,merge=None,path='README.md'):
    stream.append(f'commit refs/heads/{ref}\nmark :{mark}\ncommitter Synthetic Contributor <synthetic@example.invalid> {1700000000+mark} +0000\ndata {len(message.encode())}\n{message}\n')
    if parent:stream.append(f'from :{parent}\n')
    if merge:stream.append(f'merge :{merge}\n')
    content=f'Synthetic revision {mark}.\n'
    stream.append(f'M 100644 inline {path}\ndata {len(content.encode())}\n{content}\n\n')
for mark in range(1,5001):commit('main',mark,f'History change {mark:05d}',mark-1 if mark>1 else None)
for mark in range(5001,5011):commit('feature/long-read',mark,f'Feature change {mark-5000:05d}',4950 if mark==5001 else mark-1,path='feature.txt')
commit('main',5011,'Merge synthetic feature history',5000,5010)
git('fast-import','--quiet',input=''.join(stream).encode())
git('reset','--hard','HEAD')
anchor=git('rev-parse','main~2501').decode().strip()
git('branch','work/anchor',anchor)
git('tag','v-history-anchor',anchor)
proof={'commits':int(git('rev-list','--all','--count')),'head':git('rev-parse','HEAD').decode().strip(),'anchor':anchor,'anchorSubject':git('show','-s','--format=%s',anchor).decode().strip(),'oldest':git('rev-list','--max-parents=0','HEAD').decode().strip(),'tree':git('write-tree').decode().strip()}
assert proof['commits']==5011
assert proof['anchorSubject']=='History change 02500'
(root.parent/'fixture.json').write_text(json.dumps(proof,indent=2)+'\n')
print(root)

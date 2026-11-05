"""Disposable real Git fixture for graph references and reviewed gestures."""
import pathlib, subprocess, tempfile
root = pathlib.Path(tempfile.mkdtemp(prefix='ramo-graph-')) / 'meadow-graph'
root.mkdir()
def git(*args):
    return subprocess.check_output(['git','-C',str(root),*args], text=True).strip()
git('init','-b','main')
git('config','user.name','Synthetic Contributor')
git('config','user.email','synthetic@example.invalid')
(root/'README.md').write_text('Synthetic graph fixture\n')
git('add','README.md'); git('commit','-m','Initialize synthetic project')
git('checkout','-b','feature/search')
(root/'search.txt').write_text('Synthetic search\n')
git('add','search.txt'); git('commit','-m','Add synthetic search')
git('branch','docs/update'); git('branch','work/long-reference-for-overflow-menu')
git('tag','v-fixture'); git('tag','v-fixture-two')
git('checkout','main')
(root/'main.txt').write_text('Synthetic main work\n')
git('add','main.txt'); git('commit','-m','Prepare synthetic main branch')
print(root)

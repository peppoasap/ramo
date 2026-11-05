#!/usr/bin/env python3
"""Create a disposable source with staged/unstaged sentinels and an empty parent."""
from pathlib import Path
import json
import subprocess
import tempfile

fixture = Path(tempfile.mkdtemp(prefix="ramo-worktree-")).resolve()
source = fixture / "meadow-worktree"
parent = fixture / "checkouts"
source.mkdir()
parent.mkdir()

def git(*args):
    return subprocess.check_output(["git", "-C", str(source), *args], stderr=subprocess.STDOUT).decode().strip()

git("init", "-b", "main")
for key, value in [("user.name", "Synthetic Contributor"), ("user.email", "synthetic@example.invalid"), ("commit.gpgsign", "false"), ("core.fsmonitor", "false"), ("core.hooksPath", str(source / ".git/hooks"))]:
    git("config", key, value)
(source / "README.md").write_text("# Meadow worktree\nSynthetic committed source.\n")
git("add", "README.md")
git("commit", "-m", "Initial synthetic worktree")
git("branch", "feature/existing")
(source / "draft.txt").write_text("Source-only staged sentinel.\n")
git("add", "draft.txt")
(source / "README.md").write_text("# Meadow worktree\nSynthetic unstaged source sentinel.\n")
record = {"source": str(source), "parent": str(parent), "head": git("rev-parse", "HEAD"), "indexTree": git("write-tree")}
(fixture / "fixture.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps(record, indent=2))

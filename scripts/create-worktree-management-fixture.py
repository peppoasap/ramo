#!/usr/bin/env python3
"""Disposable clean/protected linked checkouts and source sentinels for native QA."""
from pathlib import Path
import subprocess
import tempfile
import json
import hashlib
root = Path(tempfile.mkdtemp(prefix="ramo-worktree-management-")).resolve()
source = root / "meadow-worktrees"
source.mkdir()
def git(path, *args):
    return subprocess.check_output(["git", "-C", str(path), *args], stderr=subprocess.STDOUT).decode().strip()
git(source, "init", "-b", "main")
for key, value in [("user.name", "Synthetic Contributor"), ("user.email", "synthetic@example.invalid"), ("commit.gpgsign", "false"), ("core.fsmonitor", "false")]:
    git(source, "config", key, value)
(source / "README.md").write_text("Synthetic worktree repository.\n")
(source / ".gitignore").write_text("ignored/\n")
git(source, "add", ".")
git(source, "commit", "-m", "Initial synthetic checkout")
for folder, branch in [("clean-checkout", "work/clean"), ("protected-checkout", "work/protected")]:
    git(source, "worktree", "add", "-b", branch, str(root / folder))
protected = root / "protected-checkout" / "ignored"
protected.mkdir()
(protected / "keep.txt").write_text("Synthetic ignored work; must be retained.\n")
(source / "README.md").write_text("Staged source sentinel.\n")
git(source, "add", "README.md")
(source / "README.md").write_text("Unstaged source sentinel.\n")
(source / "untracked.txt").write_text("Untracked source sentinel.\n")
proof = {"head":git(source,"rev-parse","HEAD"), "index":git(source,"write-tree"), "readmeSha256":hashlib.sha256((source/"README.md").read_bytes()).hexdigest(), "untrackedSha256":hashlib.sha256((source/"untracked.txt").read_bytes()).hexdigest(), "protectedSha256":hashlib.sha256((protected/"keep.txt").read_bytes()).hexdigest()}
(root / "before.json").write_text(json.dumps(proof, indent=2)+"\n")
print(source)

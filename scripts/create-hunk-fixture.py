#!/usr/bin/env python3
"""Disposable regular-text repository with two separated unstaged hunks."""
from pathlib import Path
import subprocess
import tempfile
root = Path(tempfile.mkdtemp(prefix="ramo-hunks-")).resolve() / "meadow-hunks"
root.mkdir()

def git(*args):
    return subprocess.check_output(["git", "-C", str(root), *args], stderr=subprocess.STDOUT)

git("init", "-b", "main")
git("config", "user.name", "Synthetic Contributor")
git("config", "user.email", "synthetic@example.invalid")
git("config", "commit.gpgsign", "false")
git("config", "core.hooksPath", str(root / ".git/hooks"))
git("config", "core.fsmonitor", "false")
path = root / "settings.ts"
base = "\n".join(f"export const SETTING_{number:02} = {number};" for number in range(1,51)) + "\n"
path.write_text(base)
git("add", "settings.ts")
git("commit", "-m", "Initial synthetic settings")
path.write_text(base.replace("SETTING_05 = 5", "SETTING_05 = 50").replace("SETTING_45 = 45", "SETTING_45 = 450"))
print(root)

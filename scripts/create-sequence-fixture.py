#!/usr/bin/env python3
"""Create a disposable, genuinely interrupted cherry-pick or revert for native QA."""
import argparse
from pathlib import Path
import subprocess
import tempfile
import json

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--operation", choices=["cherry-pick", "revert"], default="cherry-pick")
parser.add_argument("--multi", action="store_true", help="Two cherry-picked commits that conflict on separate files; retain an unrelated untracked sentinel")
options = parser.parse_args()
operation = options.operation
if options.multi and operation != "cherry-pick":
    parser.error("--multi currently exercises cherry-pick; omit it for the revert fixture")
root = Path(tempfile.mkdtemp(prefix="ramo-sequence-")).resolve() / "meadow-sequence"
root.mkdir()
if options.multi:
    renamed = root.with_name("meadow-multi-sequence")
    root.rename(renamed)
    root = renamed


def git(*args):
    return subprocess.check_output(["git", "-C", str(root), *args], stderr=subprocess.STDOUT).decode().strip()


git("init", "-b", "main")
git("config", "user.name", "Synthetic Contributor")
git("config", "user.email", "synthetic@example.invalid")
git("config", "commit.gpgsign", "false")
git("config", "core.editor", "false")
git("config", "core.hooksPath", str(root / ".git/hooks"))
path = root / "settings.ts"
path.write_text("export const PAGE_SIZE = 20;\n")
if options.multi:
    (root / "secondary.ts").write_text("export const SECONDARY_LIMIT = 20;\n")
git("add", ".")
git("commit", "-m", "Initial settings")
if operation == "cherry-pick":
    git("switch", "-c", "side")
path.write_text("export const PAGE_SIZE = 50;\n")
git("commit", "-am", "Use larger page size")
target = git("rev-parse", "HEAD")
targets = [target]
if options.multi:
    (root / "secondary.ts").write_text("export const SECONDARY_LIMIT = 50;\n")
    git("commit", "-am", "Use larger secondary limit")
    targets.append(git("rev-parse", "HEAD"))
if operation == "cherry-pick":
    git("switch", "main")
path.write_text("export const PAGE_SIZE = 30;\n")
if options.multi:
    (root / "secondary.ts").write_text("export const SECONDARY_LIMIT = 30;\n")
git("commit", "-am", "Keep current page size")
if options.multi:
    (root / "keep-local.txt").write_text("Preserve unrelated local work.\n")
(root.parent / "sequence-fixture.json").write_text(json.dumps({"head":git("rev-parse","HEAD"),"tree":git("rev-parse","HEAD^{tree}"),"targets":targets,"operation":operation,"multi":options.multi},indent=2)+"\n")
try:
    git(operation, *targets)
    raise RuntimeError("Expected a genuine Git conflict")
except subprocess.CalledProcessError as error:
    marker = "CHERRY_PICK_HEAD" if operation == "cherry-pick" else "REVERT_HEAD"
    if error.returncode != 1 or not (root / ".git" / marker).is_file():
        raise
print(root)

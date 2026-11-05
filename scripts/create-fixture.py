#!/usr/bin/env python3
"""Create synthetic, isolated repositories for desktop acceptance. Never touches a user's repo."""
import json
import subprocess
import tempfile
from pathlib import Path

base = Path(tempfile.mkdtemp(prefix="ramo-acceptance-")).resolve()
root = base / "meadow-web"
root.mkdir()

def git(*args):
    return subprocess.check_output(["git", "-C", str(root), *args], stderr=subprocess.STDOUT).decode().strip()

def commit(path, content, message):
    (root / path).parent.mkdir(parents=True, exist_ok=True)
    (root / path).write_text(content)
    git("add", "--", path)
    git("commit", "-m", message)

git("init", "-b", "main")
git("config", "user.name", "Demo Contributor")
git("config", "user.email", "demo@example.invalid")
git("config", "commit.gpgsign", "false")
commit("README.md", "# Meadow\n\nSynthetic acceptance repository.\n", "Document the Meadow project")
commit("src/search.ts", "export function search(query: string) {\n  return query.trim();\n}\n", "Add search input")
git("switch", "-c", "fix/navigation")
commit("src/navigation.ts", "export const navigation = ['Home', 'Search'];\n", "Keep sidebar navigation explicit")
git("switch", "main")
commit("package.json", '{"name":"meadow-demo","private":true}\n', "Configure the project")
git("merge", "--no-ff", "fix/navigation", "-m", "Merge navigation improvements")
git("switch", "-c", "feat/search")
commit("src/search.ts", "export function search(query: string) {\n  return query.trim().toLowerCase();\n}\n", "Normalize search queries")
git("worktree", "add", "-b", "docs/update", str(base / "meadow-docs"))
(root / "src/search.ts").write_text("export function search(query: string) {\n  const normalized = query.trim().toLowerCase();\n  return normalized;\n}\n")
git("add", "--", "src/search.ts")
(root / "src/search.test.ts").write_text("// Synthetic test file for staging acceptance.\n")
(root / "README.md").write_text("# Meadow\n\nSynthetic acceptance repository.\n\nWork in progress.\n")
(base / "fixture.json").write_text(json.dumps({"repository": str(root), "worktree": str(base / "meadow-docs")}, indent=2))
print(root)


#!/usr/bin/env python3
"""Disposable existing-merge fixture for native acceptance; no customer data."""
from pathlib import Path
import subprocess
import tempfile
import argparse

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--before-merge", action="store_true", help="Leave clean diverged branches for starting the merge in Ramo")
parser.add_argument("--code", action="store_true", help="Use a TypeScript file with two separated conflict blocks")
parser.add_argument("--long", action="store_true", help="Use TypeScript with distant conflicts for editor navigation checks")
options = parser.parse_args()
options.code = options.code or options.long
root = Path(tempfile.mkdtemp(prefix="ramo-conflict-")).resolve() / "meadow-merge"
root.mkdir()


def git(*args):
    return subprocess.check_output(["git", "-C", str(root), *args], stderr=subprocess.STDOUT)


git("init", "-b", "main")
git("config", "user.name", "Synthetic Contributor")
git("config", "user.email", "synthetic@example.invalid")
git("config", "commit.gpgsign", "false")
git("config", "core.hooksPath", str(root / ".git/hooks"))
git("config", "core.fsmonitor", "false")
path = root / ("src/search.ts" if options.code else "README.md")
path.parent.mkdir(parents=True, exist_ok=True)
base = """export interface SearchItem {
  id: string;
  title: string;
  archived: boolean;
}

export const DEFAULT_LIMIT = 20;

function isVisible(item: SearchItem): boolean {
  return !item.archived;
}

function containsQuery(item: SearchItem, query: string): boolean {
  return item.title.toLowerCase().includes(query);
}

// Keep filtering local and deterministic.
// The caller owns display and selection state.
// Empty queries return visible items in their original order.

export function normalizeQuery(value: string): string {
  return value.trim();
}

export function searchItems(
  items: SearchItem[],
  input: string,
  limit = DEFAULT_LIMIT,
): SearchItem[] {
  const query = normalizeQuery(input);
  const visible = items.filter(isVisible);

  if (!query) {
    return visible.slice(0, limit);
  }

  return visible
    .filter(item => containsQuery(item, query))
    .slice(0, limit);
}
"""
if options.long:
    base = base.replace("export const DEFAULT_LIMIT", "".join(f"// First section context {number:03}\n" for number in range(120)) + "export const DEFAULT_LIMIT")
    base = base.replace("export function normalizeQuery", "".join(f"// Second section context {number:03}\n" for number in range(160)) + "export function normalizeQuery")
path.write_text(base if options.code else "Initial synthetic text.\n")
git("add", str(path.relative_to(root)))
git("commit", "-m", "Initial synthetic file")
git("switch", "-c", "side")
path.write_text(base.replace("DEFAULT_LIMIT = 20", "DEFAULT_LIMIT = 50").replace("return value.trim();", 'return value.normalize("NFKC").trim();') if options.code else "Side branch text.\n")
git("commit", "-am", "Side change")
git("switch", "main")
path.write_text(base.replace("DEFAULT_LIMIT = 20", "DEFAULT_LIMIT = 30").replace("return value.trim();", "return value.trim().toLowerCase();") if options.code else "Main branch text.\n")
git("commit", "-am", "Main change")
if not options.before_merge:
    try:
        git("merge", "side")
        raise RuntimeError("Expected an actual merge conflict")
    except subprocess.CalledProcessError as error:
        if error.returncode != 1 or not (root / ".git/MERGE_HEAD").is_file() or not git("ls-files", "-u"):
            raise

print(root)

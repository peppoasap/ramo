//! Guard linked checkout removal independently of Git's ignored-file cleanup.
use crate::git::{git, Snapshot, Worktree};
use std::{
    collections::HashSet,
    fs,
    path::{Component, Path, PathBuf},
};

pub(crate) fn target<'a>(snapshot: &'a Snapshot, path: &str) -> Result<&'a Worktree, String> {
    let tree = snapshot
        .worktrees
        .iter()
        .find(|tree| tree.path == path)
        .ok_or("Worktree is not in the reviewed repository list.")?;
    if tree.primary {
        return Err("The primary checkout cannot be locked or removed here.".into());
    }
    let root = Path::new(path);
    if !fs::symlink_metadata(root)
        .map_err(|e| e.to_string())?
        .is_dir()
        || fs::canonicalize(root).map_err(|e| e.to_string())? != root
        || !fs::symlink_metadata(root.join(".git"))
            .map_err(|e| e.to_string())?
            .is_file()
    {
        return Err("Worktree path or Git metadata changed. No action was taken.".into());
    }
    let top = output(root, &["rev-parse", "--show-toplevel"])?;
    if fs::canonicalize(top).map_err(|e| e.to_string())? != root
        || common(root)? != common(Path::new(&snapshot.repo.path))?
    {
        return Err("Worktree no longer belongs to this repository.".into());
    }
    Ok(tree)
}
fn output(root: &Path, args: &[&str]) -> Result<String, String> {
    Ok(String::from_utf8(git(root, args)?)
        .map_err(|_| "Non-UTF-8 Git output is unsupported.")?
        .trim_end_matches(['\n', '\r'])
        .into())
}
fn common(root: &Path) -> Result<PathBuf, String> {
    let path = output(root, &["rev-parse", "--git-common-dir"])?;
    fs::canonicalize(root.join(path)).map_err(|e| e.to_string())
}

pub(crate) fn remove(source: &Path, tree: &Worktree) -> Result<(), String> {
    if tree.locked {
        return Err("Unlock this worktree before removing it.".into());
    }
    if tree.branch == "Detached HEAD" || tree.head.is_empty() {
        return Err("Create a branch for this detached checkout before removing it.".into());
    }
    let root = Path::new(&tree.path);
    for marker in [
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "sequencer",
        "rebase-merge",
        "rebase-apply",
        "BISECT_LOG",
        "index.lock",
        "HEAD.lock",
        "config.worktree.lock",
    ] {
        let location = root.join(output(root, &["rev-parse", "--git-path", marker])?);
        match fs::symlink_metadata(location) {
            Ok(_) => {
                return Err(
                    "Worktree has an active Git operation. Complete or abort it before removal."
                        .into(),
                )
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    if output(root, &["rev-parse", "--verify", "HEAD"])? != tree.head {
        return Err("Worktree HEAD changed. Refresh and review again.".into());
    }
    // Ignored files also block removal. Git's own remove command does not protect them.
    if !git(
        root,
        &[
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--ignored=matching",
        ],
    )?
    .is_empty()
    {
        return Err("Worktree contains modified, untracked or ignored data. Move or commit it before removal; nothing was removed.".into());
    }
    let flags = git(root, &["ls-files", "-v", "-z"])?;
    if flags
        .split(|b| *b == 0)
        .filter(|row| !row.is_empty())
        .any(|row| row.first() != Some(&b'H'))
    {
        return Err("Worktree has hidden index flags or unresolved entries. Review them outside Ramo before removal.".into());
    }
    let entries = git(root, &["ls-files", "--stage", "-z"])?;
    let mut tracked = HashSet::new();
    let mut directories = HashSet::new();
    for entry in entries.split(|b| *b == 0).filter(|row| !row.is_empty()) {
        let entry =
            std::str::from_utf8(entry).map_err(|_| "Non-UTF-8 paths cannot be removed safely.")?;
        let (meta, path) = entry.split_once('\t').ok_or("Invalid Git index entry.")?;
        if !(meta.starts_with("100644 ")
            || meta.starts_with("100755 ")
            || meta.starts_with("120000 "))
            || !meta.ends_with(" 0")
        {
            return Err(
                "Submodules or unresolved index entries require external worktree management."
                    .into(),
            );
        }
        let relative = Path::new(path);
        if relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err("Invalid tracked path; removal refused.".into());
        }
        tracked.insert(relative.to_path_buf());
        let mut parent = relative.parent();
        while let Some(path) = parent {
            if path.as_os_str().is_empty() {
                break;
            }
            directories.insert(path.to_path_buf());
            parent = path.parent();
        }
    }
    // Every on-disk entry must be accounted for, including empty ignored directories,
    // nested repositories and non-file objects. Never follow a directory symlink.
    let mut pending = vec![PathBuf::new()];
    let mut count = 0;
    while let Some(relative) = pending.pop() {
        for entry in fs::read_dir(root.join(&relative)).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            count += 1;
            if count > 100_000 {
                return Err("Worktree inventory exceeds 100,000 entries; remove it outside Ramo after review.".into());
            }
            let path = relative.join(entry.file_name());
            if path == Path::new(".git") {
                continue;
            }
            if entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(".git")
            {
                return Err("Nested Git metadata was found. Nothing was removed.".into());
            }
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_dir() && directories.contains(&path) {
                pending.push(path);
            } else if !(kind.is_file() || kind.is_symlink()) || !tracked.contains(&path) {
                return Err("Worktree contains extra files or directories, including ignored data. Nothing was removed.".into());
            }
        }
    }
    // No force, cleanup, prune, branch deletion or automatic retry. Git rechecks its state.
    git(source, &["worktree", "remove", "--", &tree.path])?;
    Ok(())
}

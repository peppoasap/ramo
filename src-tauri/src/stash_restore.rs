//! Restore exactly one saved untracked blob without changing the index or stash.
use crate::git::{git, valid_path};
use std::{
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
};
const MAX_BYTES: usize = 8 * 1024 * 1024;
fn safe_path(path: &str) -> Result<(), String> {
    valid_path(path)?;
    for part in Path::new(path).components() {
        let Component::Normal(name) = part else {
            return Err("A normal repository-relative path is required.".into());
        };
        let name = name.to_str().ok_or("Path must be UTF-8")?;
        let folded = name.trim_end_matches([' ', '.']).to_ascii_lowercase();
        if folded == ".git" || folded.starts_with("git~") || folded.starts_with(".git~") {
            return Err("Restoring Git metadata paths is forbidden.".into());
        }
        #[cfg(windows)]
        {
            if name.chars().any(|character| "<>:\"|?*".contains(character))
                || name.ends_with([' ', '.'])
                || name.chars().any(char::is_control)
            {
                return Err("This filename cannot be safely restored on Windows.".into());
            }
            let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
            if ["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"].contains(&stem.as_str())
                || ((stem.starts_with("COM") || stem.starts_with("LPT"))
                    && stem.len() == 4
                    && matches!(stem.as_bytes()[3], b'1'..=b'9'))
            {
                return Err("Reserved Windows filename.".into());
            }
        }
    }
    Ok(())
}
fn parent(root: &Path, path: &str) -> Result<PathBuf, String> {
    let mut current = root.to_path_buf();
    let relative = Path::new(path).parent().ok_or("Missing parent")?;
    for component in relative.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
            Ok(_) => {
                return Err(
                    "A restore parent is not a regular directory. Symlinks are not followed."
                        .into(),
                )
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&current).map_err(|e| e.to_string())?;
            }
            Err(error) => return Err(error.to_string()),
        }
        if !fs::canonicalize(&current)
            .map_err(|e| e.to_string())?
            .starts_with(root)
            || fs::symlink_metadata(current.join(".git")).is_ok()
            || (current.join("HEAD").is_file() && current.join("objects").is_dir())
        {
            return Err("Restore path enters another repository or leaves this worktree.".into());
        }
    }
    Ok(current)
}
pub fn restore(
    root: &Path,
    index: usize,
    oid: &str,
    path: &str,
    head: Option<&str>,
) -> Result<(), String> {
    safe_path(path)?;
    let details = crate::stash::details(root, index, oid)?;
    if !details.untracked.iter().any(|file| file.path == path) {
        return Err("Choose a file from this stash's saved untracked files.".into());
    }
    let mut tracked = git(root, &["ls-files", "--cached", "-z"])?;
    if let Some(head) = head {
        tracked.extend(git(
            root,
            &["ls-tree", "-r", "-z", "--name-only", "--full-tree", head],
        )?);
    }
    let ancestors: Vec<_> = Path::new(path)
        .ancestors()
        .filter_map(|path| path.to_str())
        .filter(|path| !path.is_empty())
        .collect();
    let directory_prefix = format!("{path}/");
    for tracked_path in tracked
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
    {
        let tracked_path = std::str::from_utf8(tracked_path)
            .map_err(|_| "Non-UTF-8 tracked paths are unsupported")?;
        if ancestors.contains(&tracked_path) || tracked_path.starts_with(&directory_prefix) {
            return Err("This destination overlaps a path tracked in HEAD or the index. Nothing was restored.".into());
        }
    }
    let tree = format!("{}^3", details.stash.oid);
    let raw = git(
        root,
        &["ls-tree", "-r", "-z", "--full-tree", &tree, "--", path],
    )?;
    let entries: Vec<_> = raw
        .split(|b| *b == 0)
        .filter(|entry| !entry.is_empty())
        .collect();
    if entries.len() != 1 {
        return Err("Expected exactly one saved file.".into());
    }
    let value = std::str::from_utf8(entries[0]).map_err(|_| "Non-UTF-8 path")?;
    let (metadata, name) = value
        .split_once('\t')
        .ok_or("Invalid saved file metadata")?;
    let fields: Vec<_> = metadata.split_whitespace().collect();
    if name != path
        || fields.len() != 3
        || fields[1] != "blob"
        || !matches!(fields[0], "100644" | "100755")
    {
        return Err("Only regular saved files can be restored. Symlinks and submodules require external recovery.".into());
    }
    let size = git(root, &["cat-file", "-s", fields[2]])?;
    let size: usize = std::str::from_utf8(&size)
        .map_err(|_| "Invalid file size")?
        .trim()
        .parse()
        .map_err(|_| "Invalid file size")?;
    if size > MAX_BYTES {
        return Err("Saved-file restoration is limited to 8 MiB per file.".into());
    }
    let bytes = git(root, &["cat-file", "blob", fields[2]])?;
    if bytes.len() != size {
        return Err("Saved file size changed or is incomplete. Nothing was restored.".into());
    }
    let parent = parent(root, path)?;
    let destination = parent.join(Path::new(path).file_name().ok_or("Missing filename")?);
    match fs::symlink_metadata(&destination) {
        Ok(_) => return Err("Destination already exists. Nothing was overwritten.".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }
    let mut temporary = tempfile::NamedTempFile::new_in(&parent).map_err(|e| e.to_string())?;
    temporary.write_all(&bytes).map_err(|e| e.to_string())?;
    temporary.flush().map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temporary
            .as_file()
            .set_permissions(fs::Permissions::from_mode(if fields[0] == "100755" {
                0o700
            } else {
                0o600
            }))
            .map_err(|e| e.to_string())?;
    }
    temporary.persist_noclobber(&destination).map_err(|e|format!("Could not create restored file without overwriting: {e}. Inspect the working folder before retrying."))?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_repository_metadata_and_traversal() {
        for path in [
            "../outside",
            ".git/config",
            ".GIT/hooks/post-commit",
            "folder/.git./config",
            "GIT~1/config",
            "/absolute",
        ] {
            assert!(safe_path(path).is_err(), "{path}");
        }
    }
}

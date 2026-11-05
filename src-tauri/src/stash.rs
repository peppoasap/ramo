//! Object-bound stash inspection and reviewed application/removal.
use crate::git::{git, parse_names, FileChange};
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stash {
    pub index: usize,
    pub oid: String,
    pub reference: String,
    pub message: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Details {
    pub stash: Stash,
    pub tracked: Vec<FileChange>,
    pub untracked: Vec<FileChange>,
}
pub fn list(root: &Path) -> Result<(Vec<Stash>, Vec<u8>), String> {
    let raw = git(root, &["stash", "list", "-z", "--format=%H%x00%gd%x00%gs"])?;
    let value = std::str::from_utf8(&raw).map_err(|_| "Stash metadata must be UTF-8")?;
    let fields: Vec<_> = value
        .strip_suffix('\0')
        .unwrap_or(value)
        .split('\0')
        .collect();
    if raw.is_empty() {
        return Ok((vec![], raw));
    }
    if fields.len() % 3 != 0 {
        return Err("Invalid Git stash metadata".into());
    }
    let stashes = fields
        .chunks_exact(3)
        .enumerate()
        .map(|(index, fields)| {
            if fields[1] != format!("stash@{{{index}}}") {
                return Err("Invalid stash reference".into());
            }
            Ok(Stash {
                index,
                oid: fields[0].into(),
                reference: fields[1].into(),
                message: fields[2].into(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok((stashes, raw))
}
pub fn checked(root: &Path, index: usize, expected: &str) -> Result<Stash, String> {
    let stash = list(root)?
        .0
        .into_iter()
        .nth(index)
        .ok_or("Stash no longer exists. Refresh and review again.")?;
    if stash.oid != expected {
        return Err("Stash changed. Refresh and review again.".into());
    }
    Ok(stash)
}
fn parents(root: &Path, oid: &str) -> Result<Vec<String>, String> {
    let bytes = git(root, &["rev-list", "--parents", "-n", "1", oid])?;
    let value = std::str::from_utf8(&bytes).map_err(|_| "Invalid stash parents")?;
    let parents: Vec<String> = value
        .split_whitespace()
        .skip(1)
        .map(str::to_string)
        .collect();
    if !(2..=3).contains(&parents.len()) {
        return Err("This object is not a supported stash.".into());
    }
    Ok(parents)
}
pub fn details(root: &Path, index: usize, expected: &str) -> Result<Details, String> {
    let stash = checked(root, index, expected)?;
    let parents = parents(root, &stash.oid)?;
    let tracked = parse_names(&git(
        root,
        &[
            "diff-tree",
            "--no-commit-id",
            "--name-status",
            "--no-renames",
            "-r",
            "-z",
            &parents[0],
            &stash.oid,
        ],
    )?)?;
    let untracked = if let Some(untracked) = parents.get(2) {
        let raw = git(root, &["ls-tree", "-r", "-z", "--name-only", untracked])?;
        raw.split(|byte| *byte == 0)
            .filter(|part| !part.is_empty())
            .map(|path| {
                Ok(FileChange {
                    path: std::str::from_utf8(path)
                        .map_err(|_| "Non-UTF-8 filenames are unsupported")?
                        .into(),
                    original: None,
                    index: "A".into(),
                    worktree: " ".into(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?
    } else {
        vec![]
    };
    Ok(Details {
        stash,
        tracked,
        untracked,
    })
}
pub fn file_diff(
    root: &Path,
    index: usize,
    expected: &str,
    path: &str,
    untracked: bool,
) -> Result<String, String> {
    let details = details(root, index, expected)?;
    let files = if untracked {
        &details.untracked
    } else {
        &details.tracked
    };
    if !files.iter().any(|file| file.path == path) {
        return Err("File is not part of this stash.".into());
    }
    let parents = parents(root, &details.stash.oid)?;
    let raw = if untracked {
        git(
            root,
            &[
                "show",
                "--format=",
                "--root",
                "--no-ext-diff",
                "--no-textconv",
                "--no-renames",
                &parents[2],
                "--",
                path,
            ],
        )?
    } else {
        git(
            root,
            &[
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "--no-renames",
                &parents[0],
                &details.stash.oid,
                "--",
                path,
            ],
        )?
    };
    if raw.len() > 512 * 1024 {
        return Err("Stash diff exceeds the 512 KiB preview limit. Inspect it with Git.".into());
    }
    String::from_utf8(raw).map_err(|_| "This stash diff is not UTF-8.".into())
}
pub fn apply(root: &Path, index: usize, expected: &str, pop: bool) -> Result<(), String> {
    let stash = checked(root, index, expected)?;
    parents(root, &stash.oid)?;
    // Apply the immutable reviewed object; a shifted stack must never apply another entry.
    git(root,&["stash","apply",&stash.oid]).map_err(|error|format!("Stash retained. Git may have restored some files or left conflicts; review Changes before retrying.\n{error}"))?;
    if pop {
        checked(root,index,expected).map_err(|_|"Stash applied, but the stack changed. The original stash was retained; refresh before any further action.".to_string())?;
        git(root,&["stash","drop",&stash.reference]).map_err(|error|format!("Stash applied; removal did not finish. Refresh and inspect the stack before retrying.\n{error}"))?;
    }
    Ok(())
}
pub fn drop(root: &Path, index: usize, expected: &str) -> Result<(), String> {
    let stash = checked(root, index, expected)?;
    git(root, &["stash", "drop", &stash.reference])?;
    Ok(())
}

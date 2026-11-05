//! Read-only comparisons of two explicit commit objects.
use crate::git::{git, parse_names, verify_commit, FileChange};
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Trees,
    MergeBase,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparison {
    pub from: String,
    pub to: String,
    pub effective_from: String,
    pub kind: Kind,
    pub files: Vec<FileChange>,
}
fn effective_base(root: &Path, from: &str, to: &str, kind: Kind) -> Result<String, String> {
    verify_commit(root, from)?;
    verify_commit(root, to)?;
    match kind {
        Kind::Trees => Ok(from.into()),
        Kind::MergeBase => {
            let raw=git(root,&["merge-base","--all",from,to]).map_err(|_|"Cannot find a common ancestor. Use direct tree comparison for unrelated histories.".to_string())?;
            let value = std::str::from_utf8(&raw).map_err(|_| "Invalid common ancestor")?;
            let bases: Vec<_> = value.split_whitespace().collect();
            if bases.len() != 1 {
                return Err("Multiple common ancestors; choose direct tree comparison instead of guessing a merge base.".into());
            }
            Ok(bases[0].into())
        }
    }
}
pub fn details(root: &Path, from: &str, to: &str, kind: Kind) -> Result<Comparison, String> {
    let effective_from = effective_base(root, from, to, kind)?;
    let files = parse_names(&git(
        root,
        &[
            "diff-tree",
            "--no-commit-id",
            "--name-status",
            "--no-renames",
            "-r",
            "-z",
            &effective_from,
            to,
        ],
    )?)?;
    Ok(Comparison {
        from: from.into(),
        to: to.into(),
        effective_from,
        kind,
        files,
    })
}
pub fn file_diff(
    root: &Path,
    from: &str,
    to: &str,
    kind: Kind,
    path: &str,
) -> Result<String, String> {
    let comparison = details(root, from, to, kind)?;
    if !comparison.files.iter().any(|file| file.path == path) {
        return Err("This file is not changed between the selected revisions.".into());
    }
    let raw = git(
        root,
        &[
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            "--no-relative",
            "--no-color",
            &comparison.effective_from,
            to,
            "--",
            path,
        ],
    )?;
    if raw.len() > 512 * 1024 {
        return Err(
            "Comparison diff exceeds the 512 KiB preview limit. Inspect it with Git.".into(),
        );
    }
    String::from_utf8(raw).map_err(|_| "Comparison diff is not UTF-8.".into())
}

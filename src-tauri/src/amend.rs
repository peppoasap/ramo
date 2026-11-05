//! Conservative last-commit rewriting; never touches unstaged files.
use crate::git::{git, git_input, Snapshot};
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn message(snapshot: &Snapshot, expected_head: &str) -> Result<String, String> {
    let root = Path::new(&snapshot.repo.path);
    if snapshot.head.as_deref() != Some(expected_head) || snapshot.operation.is_some() {
        return Err("HEAD or operation changed. Refresh before amending.".into());
    }
    git(root, &["symbolic-ref", "--quiet", "HEAD"])
        .map_err(|_| "Amend requires an attached local branch.".to_owned())?;
    let parents = String::from_utf8(git(
        root,
        &["rev-list", "--parents", "-n", "1", expected_head],
    )?)
    .map_err(|_| "Invalid commit metadata.".to_owned())?;
    if parents.split_whitespace().count() > 2 {
        return Err("Amending merge commits is not supported.".into());
    }
    let published = git(
        root,
        &[
            "for-each-ref",
            "--format=%(refname)",
            "--contains",
            expected_head,
            "refs/remotes/",
        ],
    )?;
    if !published.is_empty() {
        return Err("This commit is reachable from a locally known remote branch. Amend is refused; create a new commit instead.".into());
    }
    let message = String::from_utf8(git(root, &["show", "-s", "--format=%B", expected_head])?)
        .map_err(|_| "The commit message is not UTF-8.".to_owned())?;
    if message.len() > 32768 {
        return Err("The original message exceeds 32 KiB.".into());
    }
    Ok(message)
}

pub fn apply(snapshot: &Snapshot, expected_head: &str, message_text: String) -> Result<(), String> {
    if message_text.trim().is_empty() || message_text.len() > 32768 {
        return Err("Enter a commit message (maximum 32 KiB).".into());
    }
    message(snapshot, expected_head)?;
    let root = Path::new(&snapshot.repo.path);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let recovery = format!("refs/ramo/amend/{expected_head}-{nonce}");
    // Empty old value requires this recovery ref to be new. Keep it even if a hook rejects the commit.
    git(root, &["update-ref", &recovery, expected_head, ""])?;
    git_input(
        root,
        &["commit", "--amend", "-F", "-"],
        Some(message_text.into_bytes()),
    )?;
    Ok(())
}

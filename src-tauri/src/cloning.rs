//! Native-scoped destination/source grants and immutable clone reviews.
use crate::git::{git, hash, validate_worktree_directory};
use crate::process::{self, strings};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
#[derive(Clone, Serialize)]
pub struct Grant {
    pub id: String,
    pub path: String,
}
#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Source {
    Url { url: String },
    Local { id: String },
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub id: String,
    pub source: String,
    pub destination: String,
    pub network: bool,
}
struct Pending {
    view: Plan,
    parent: PathBuf,
    source: Option<PathBuf>,
}
#[derive(Default)]
pub struct Grants {
    parents: HashMap<String, PathBuf>,
    sources: HashMap<String, PathBuf>,
    plans: HashMap<String, Pending>,
}
fn token(label: &str) -> Result<String, String> {
    Ok(hash(format!(
        "{label}:{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    )))
}
fn utf8(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or("Clone paths must be UTF-8".into())
}
fn parent(path: &Path) -> Result<PathBuf, String> {
    let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
    let git_boundary = path.ancestors().any(|ancestor| {
        fs::symlink_metadata(ancestor.join(".git")).is_ok()
            || (ancestor.join("HEAD").is_file() && ancestor.join("objects").is_dir())
    });
    if !path.is_dir() || git_boundary || git(&path, &["rev-parse", "--absolute-git-dir"]).is_ok() {
        return Err("Choose a parent folder outside existing Git repositories.".into());
    }
    Ok(path)
}
fn local_source(path: &Path) -> Result<PathBuf, String> {
    let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
    let bare = git(&path, &["rev-parse", "--is-bare-repository"])?;
    if bare.starts_with(b"true") {
        return Ok(path);
    }
    let top = git(&path, &["rev-parse", "--show-toplevel"])?;
    let top = std::str::from_utf8(&top)
        .map_err(|_| "Source path must be UTF-8")?
        .trim();
    if fs::canonicalize(top).map_err(|e| e.to_string())? != path {
        return Err("Choose the local repository root or a bare repository.".into());
    }
    Ok(path)
}
/** Reject custom transports/helpers, embedded credentials and raw local paths in typed URLs. */
pub fn validate_url(value: &str) -> Result<String, String> {
    if value.is_empty()
        || value.len() > 2048
        || value.trim() != value
        || value.chars().any(|c| c.is_control() || c.is_whitespace())
    {
        return Err("Enter an HTTPS or SSH repository URL without whitespace.".into());
    }
    if value.contains("://") {
        let url = reqwest::Url::parse(value).map_err(|_| "Invalid repository URL")?;
        if !matches!(url.scheme(), "https" | "ssh")
            || url.host_str().is_none_or(|h| h.starts_with('-'))
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path().is_empty()
            || url.path() == "/"
        {
            return Err("Use HTTPS/SSH without a password, query or fragment. Choose local sources with the native picker.".into());
        }
        if url.scheme() == "https" && !url.username().is_empty() {
            return Err(
                "Do not embed credentials in HTTPS URLs. Use your existing Git credential helper."
                    .into(),
            );
        }
        if url.username().starts_with('-')
            || !url
                .username()
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
        {
            return Err("Invalid SSH user.".into());
        }
        return Ok(value.into());
    }
    let Some((authority, path)) = value.split_once(':') else {
        return Err("Use HTTPS/SSH, or choose a local source with the picker.".into());
    };
    let Some((user, host)) = authority.split_once('@') else {
        return Err("Use SSH syntax user@host:path or an HTTPS/SSH URL.".into());
    };
    let valid = |part: &str| {
        !part.is_empty()
            && !part.starts_with('-')
            && part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
    };
    if !valid(user)
        || !valid(host)
        || path.is_empty()
        || path.starts_with('-')
        || !path
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "/._~@%+-".contains(c))
    {
        return Err(
            "Invalid SSH repository address. Use ssh:// syntax for other addresses.".into(),
        );
    }
    Ok(value.into())
}
impl Grants {
    /// Native picker or disposable fixture tests only; never accepts a renderer path.
    pub fn authorize_parent(&mut self, path: &Path) -> Result<Grant, String> {
        let path = parent(path)?;
        let id = token(&utf8(&path)?)?;
        if self.parents.len() >= 8 {
            self.parents.clear();
        }
        self.parents.insert(id.clone(), path.clone());
        Ok(Grant {
            id,
            path: utf8(&path)?,
        })
    }
    pub fn authorize_source(&mut self, path: &Path) -> Result<Grant, String> {
        let path = local_source(path)?;
        let id = token(&utf8(&path)?)?;
        if self.sources.len() >= 8 {
            self.sources.clear();
        }
        self.sources.insert(id.clone(), path.clone());
        Ok(Grant {
            id,
            path: utf8(&path)?,
        })
    }
    pub fn prepare(
        &mut self,
        parent_id: &str,
        directory: &str,
        source: Source,
    ) -> Result<Plan, String> {
        validate_worktree_directory(directory)?;
        let selected = self
            .parents
            .get(parent_id)
            .ok_or("Choose a parent folder with the native picker again.")?;
        let canonical = parent(selected)?;
        if canonical != *selected {
            return Err("Parent folder changed. Choose it again.".into());
        }
        let destination = canonical.join(directory);
        match fs::symlink_metadata(&destination) {
            Ok(_) => return Err("Destination already exists; choose a new folder name.".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
        let (source, local) = match source {
            Source::Url { url } => (validate_url(&url)?, None),
            Source::Local { id } => {
                let selected = self
                    .sources
                    .get(&id)
                    .ok_or("Choose the local source again.")?;
                let canonical = local_source(selected)?;
                if canonical != *selected {
                    return Err("Source changed. Choose it again.".into());
                }
                (utf8(&canonical)?, Some(canonical))
            }
        };
        let view = Plan {
            id: token(&format!("{source}:{}", utf8(&destination)?))?,
            source,
            destination: utf8(&destination)?,
            network: local.is_none(),
        };
        if self.plans.len() >= 8 {
            self.plans.clear();
        }
        self.plans.insert(
            view.id.clone(),
            Pending {
                view: view.clone(),
                parent: canonical,
                source: local,
            },
        );
        Ok(view)
    }
    pub fn execute(&mut self, id: &str) -> Result<PathBuf, String> {
        let pending = self.plans.remove(id).ok_or(
            "Clone review expired or already used. Review the source and destination again.",
        )?;
        if parent(&pending.parent)? != pending.parent {
            return Err("Parent changed since review. Choose it again.".into());
        }
        if let Some(source) = &pending.source {
            if local_source(source)? != *source {
                return Err("Local source changed since review.".into());
            }
        }
        let destination = PathBuf::from(&pending.view.destination);
        let program = process::executable("git")?;
        // Exclusive reservation; no pre-existing directory or symlink is ever reused.
        fs::create_dir(&destination)
            .map_err(|e| format!("Cannot create new destination; nothing was overwritten: {e}"))?;
        let file_policy = if pending.source.is_some() {
            "protocol.file.allow=always"
        } else {
            "protocol.file.allow=never"
        };
        let network_policy = if pending.source.is_some() {
            "never"
        } else {
            "always"
        };
        let https_policy = format!("protocol.https.allow={network_policy}");
        let ssh_policy = format!("protocol.ssh.allow={network_policy}");
        let arguments = strings(&[
            "--no-pager",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "color.ui=false",
            "-c",
            "protocol.allow=never",
            "-c",
            &https_policy,
            "-c",
            &ssh_policy,
            "-c",
            file_policy,
            "clone",
            "--no-recurse-submodules",
            "--no-hardlinks",
            "--no-local",
            "--",
            &pending.view.source,
            &pending.view.destination,
        ]);
        process::run(&program, &pending.parent, &arguments, None, std::time::Duration::from_secs(120))
            .map_err(|error| format!("Clone did not complete normally. Inspect the destination before retrying; it may contain partial data. Ramo has not deleted it.\n{error}"))?;
        Ok(destination)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_https_and_explicit_ssh_addresses() {
        for url in [
            "https://example.invalid/team/repo.git",
            "ssh://git@example.invalid:2222/team/repo.git",
            "git@example.invalid:team/repo.git",
            "ssh://git@[::1]/team/repo.git",
        ] {
            assert_eq!(validate_url(url).unwrap(), url);
        }
    }
    #[test]
    fn rejects_credentials_unreviewed_transports_helpers_and_ambiguous_inputs() {
        for url in [
            "",
            "--upload-pack=command",
            "/tmp/repository",
            "file:///tmp/repository",
            "http://example.invalid/repo.git",
            "ext::command",
            "https://user:secret@example.invalid/repo.git",
            "https://user@example.invalid/repo.git",
            "https://example.invalid/repo.git?token=secret",
            "https://example.invalid/repo.git#secret",
            "ssh://git:secret@example.invalid/repo.git",
            "ssh://-option/repo",
            "-user@example.invalid:repo",
            "git@-host:repo",
            "git@example.invalid:-option",
            "git@example.invalid:path;command",
            "git@example.invalid:repo\ncommand",
        ] {
            assert!(validate_url(url).is_err(), "{url}");
        }
    }
}

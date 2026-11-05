//! Remote endpoint management; no credentials are returned to the renderer.
use crate::{cloning::validate_url, git::git};
use serde::Serialize;
use std::path::Path;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Remote {
    pub name: String,
    pub fetch: Vec<String>,
    pub push: Vec<String>,
    pub editable: bool,
}
pub fn list(root: &Path) -> Result<Vec<Remote>, String> {
    let names = String::from_utf8(git(root, &["remote"])?)
        .map_err(|_| "Remote names must be UTF-8".to_owned())?;
    let config = git(root, &["config", "--null", "--list"])?;
    names
        .lines()
        .map(|name| {
            let read = |push: bool| -> Result<Vec<String>, String> {
                let args = if push {
                    vec!["remote", "get-url", "--all", "--push", name]
                } else {
                    vec!["remote", "get-url", "--all", name]
                };
                let bytes = git(root, &args)?;
                Ok(String::from_utf8(bytes)
                    .map_err(|_| "Remote URLs must be UTF-8".to_owned())?
                    .lines()
                    .map(str::to_owned)
                    .collect())
            };
            let fetch = read(false)?;
            let push = read(true)?;
            let push_key = format!("remote.{name}.pushurl\n");
            let explicit_push = config
                .split(|b| *b == 0)
                .any(|entry| entry.starts_with(push_key.as_bytes()));
            let editable = fetch.len() == 1
                && push == fetch
                && !explicit_push
                && validate_url(&fetch[0]).is_ok();
            let safe = |urls: Vec<String>| {
                urls.into_iter()
                    .map(|url| {
                        if validate_url(&url).is_ok() {
                            url
                        } else {
                            "[Local, custom or credential-bearing URL hidden]".into()
                        }
                    })
                    .collect()
            };
            Ok(Remote {
                name: name.into(),
                fetch: safe(fetch),
                push: safe(push),
                editable,
            })
        })
        .collect()
}
pub fn configure(root: &Path, name: &str, url: &str, existing: bool) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 100
        || name.starts_with('-')
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
    {
        return Err("Use a remote name with letters, digits, dots, underscores or hyphens.".into());
    }
    validate_url(url)?;
    let configured = list(root)?;
    let old = configured.iter().find(|r| r.name == name);
    if old.is_some() != existing {
        return Err("Remote changed. Refresh its configuration before saving.".into());
    }
    if existing && !old.is_some_and(|r| r.editable) {
        return Err("Multiple URLs, separate push URLs or custom/local transports need Git outside Ramo. Existing settings were preserved.".into());
    }
    if existing {
        git(root, &["remote", "set-url", name, url])?;
    } else {
        git(root, &["remote", "add", name, url])?;
    }
    Ok(())
}

//! Explicit read-only GitHub catalog via the installed official CLI. Never reads tokens.
use crate::{cloning::validate_url, process};
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Duration};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    pub name: String,
    pub https: String,
    pub ssh: String,
    pub private: bool,
}
#[derive(Deserialize)]
struct Entry {
    full_name: String,
    clone_url: String,
    ssh_url: String,
    private: bool,
}
fn arguments(host: &str, organization: &str, page: u32) -> Result<Vec<String>, String> {
    if host.len() > 253
        || host.is_empty()
        || host.starts_with('-')
        || host.split('.').any(|label| {
            label.is_empty() || label.len() > 63 || label.starts_with('-') || label.ends_with('-')
        })
        || !host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err("Enter a GitHub hostname without a scheme, port or path.".into());
    }
    if organization.len() > 100
        || organization.starts_with('-')
        || !organization
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        || !(1..=20).contains(&page)
    {
        return Err("Invalid organization or page (maximum 20 pages).".into());
    }
    let endpoint = if organization.is_empty() {
        "/user/repos".to_owned()
    } else {
        format!("/orgs/{organization}/repos")
    };
    Ok(process::strings(&[
        "api",
        "--hostname",
        host,
        "--method",
        "GET",
        &format!("{endpoint}?per_page=100&sort=pushed&page={page}"),
    ]))
}
fn parse(bytes: &[u8], host: &str) -> Result<Vec<Repository>, String> {
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("Repository catalog exceeds 2 MiB. Narrow the organization scope.".into());
    }
    let entries: Vec<Entry> = serde_json::from_slice(bytes)
        .map_err(|_| "GitHub returned an unsupported repository list.".to_owned())?;
    if entries.len() > 100 {
        return Err("Unexpected catalog page size.".into());
    }
    entries
        .into_iter()
        .map(|e| {
            if e.full_name.len() > 250
                || e.full_name.split('/').count() != 2
                || e.full_name.split('/').any(|part| {
                    part.is_empty() || part == "." || part == ".." || part.starts_with('-')
                })
                || !e
                    .full_name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "/._-".contains(c))
            {
                return Err("Invalid repository name in catalog.".into());
            }
            validate_url(&e.clone_url)?;
            validate_url(&e.ssh_url)?;
            // Reconstruct the clone URLs from validated metadata; never trust an API-supplied destination host/path.
            let expected_https = format!("https://{host}/{}.git", e.full_name);
            let expected_ssh = format!("git@{host}:{}.git", e.full_name);
            if e.clone_url != expected_https || e.ssh_url != expected_ssh {
                return Err(
                    "Repository clone address does not match the selected GitHub host.".into(),
                );
            }
            Ok(Repository {
                name: e.full_name,
                https: expected_https,
                ssh: expected_ssh,
                private: e.private,
            })
        })
        .collect()
}
pub fn github_repositories(
    host: &str,
    organization: &str,
    page: u32,
) -> Result<Vec<Repository>, String> {
    let args = arguments(host, organization, page)?;
    let program = process::executable("gh").map_err(|_| {
        "GitHub CLI (gh) was not found. Install/sign in outside Ramo, or clone with a URL."
            .to_owned()
    })?;
    load(&program, host, &args)
}
fn load(program: &Path, host: &str, args: &[String]) -> Result<Vec<Repository>, String> {
    let folder = tempfile::tempdir().map_err(|e| e.to_string())?;
    let bytes=process::run(program,folder.path(),args,None,Duration::from_secs(30))
        .map_err(|_| "Could not load GitHub repositories. Check gh authentication/access for this host outside Ramo, or use a repository URL.".to_owned())?;
    parse(&bytes, host)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_is_explicit_get_bounded_and_never_accepts_arbitrary_endpoints() {
        let args = arguments("github.example.invalid", "meadow", 2).unwrap();
        assert_eq!(
            args,
            process::strings(&[
                "api",
                "--hostname",
                "github.example.invalid",
                "--method",
                "GET",
                "/orgs/meadow/repos?per_page=100&sort=pushed&page=2"
            ])
        );
        for host in [
            "",
            "--hostname",
            "https://github.com",
            "github.com/path",
            "user@github.com",
            "github.com:443",
        ] {
            assert!(arguments(host, "", 1).is_err());
        }
        assert!(arguments("github.com", "../user", 1).is_err());
        assert!(arguments("github.com", "", 0).is_err());
        assert!(arguments("github.com", "", 21).is_err());
    }
    #[test]
    fn catalog_validates_response_and_refuses_credentials_or_host_changes() {
        let data=br#"[{"full_name":"meadow/app","clone_url":"https://github.com/meadow/app.git","ssh_url":"git@github.com:meadow/app.git","private":true}]"#;
        assert_eq!(parse(data, "github.com").unwrap()[0].name, "meadow/app");
        assert!(parse(data, "other.invalid").is_err());
        assert!(parse(b"{}", "github.com").is_err());
        assert!(parse(&vec![b' '; 2 * 1024 * 1024 + 1], "github.com").is_err());
    }
    #[cfg(unix)]
    #[test]
    fn synthetic_cli_receives_only_typed_read_request_and_failures_are_sanitized() {
        use std::{fs, os::unix::fs::PermissionsExt};
        let t = tempfile::tempdir().unwrap();
        let cli = t.path().join("gh");
        fs::write(
            &cli,
            "#!/bin/sh\n[ \"$1\" = api ] && [ \"$5\" = GET ] || exit 1\nprintf '[]'\n",
        )
        .unwrap();
        fs::set_permissions(&cli, fs::Permissions::from_mode(0o700)).unwrap();
        assert!(
            load(&cli, "github.com", &arguments("github.com", "", 1).unwrap())
                .unwrap()
                .is_empty()
        );
        fs::write(
            &cli,
            "#!/bin/sh\necho 'private token sentinel' >&2\nexit 1\n",
        )
        .unwrap();
        let err = load(&cli, "github.com", &arguments("github.com", "", 1).unwrap()).unwrap_err();
        assert!(!err.contains("sentinel"));
    }
}

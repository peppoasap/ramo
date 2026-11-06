//! Ordinary clone URL shortcuts, not provider login/connection claims.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum Provider {
    #[default]
    Url,
    Github,
    Enterprise,
    Gitlab,
    Bitbucket,
    Azure,
    Other,
    Local,
}
impl Provider {
    pub const ALL: [Self; 8] = [
        Self::Url,
        Self::Github,
        Self::Enterprise,
        Self::Gitlab,
        Self::Bitbucket,
        Self::Azure,
        Self::Other,
        Self::Local,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Url => "Paste a link",
            Self::Github => "GitHub",
            Self::Enterprise => "GitHub Enterprise",
            Self::Gitlab => "GitLab",
            Self::Bitbucket => "Bitbucket",
            Self::Azure => "Azure DevOps",
            Self::Other => "Other server",
            Self::Local => "On this computer",
        }
    }
    pub fn catalog(self) -> bool {
        matches!(self, Self::Github | Self::Enterprise)
    }
    pub fn custom_host(self) -> bool {
        matches!(self, Self::Enterprise | Self::Other)
    }
    pub fn host(self, host: &str) -> &str {
        match self {
            Self::Github => "github.com",
            Self::Gitlab => "gitlab.com",
            Self::Bitbucket => "bitbucket.org",
            Self::Azure => "dev.azure.com",
            _ => host.trim(),
        }
    }
    pub fn url(self, repository: &str, host: &str, ssh: bool) -> Result<String, String> {
        if self == Self::Url {
            return ramo_core::cloning::validate_url(repository.trim());
        }
        if self == Self::Local {
            return Err("Choose the source repository with the folder picker.".into());
        }
        let host = self.host(host);
        if host.is_empty()
            || host.len() > 253
            || host.starts_with('-')
            || host.ends_with('-')
            || host.contains("..")
            || !host
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || ".-".contains(c))
        {
            return Err("Enter a hostname without a scheme, port, path or credentials.".into());
        }
        let trimmed = repository.trim();
        let path = trimmed.strip_suffix(".git").unwrap_or(trimmed);
        let parts: Vec<_> = path.split('/').collect();
        if parts.len() < 2
            || parts.iter().any(|p| {
                p.is_empty()
                    || *p == "."
                    || *p == ".."
                    || p.starts_with('-')
                    || !p
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
            })
        {
            return Err("Enter a repository path such as team/project.".into());
        }
        let url = if self == Self::Azure {
            if parts.len() != 3 {
                return Err("Use organization/project/repository for Azure DevOps.".into());
            }
            if ssh {
                format!("git@ssh.dev.azure.com:v3/{path}")
            } else {
                format!(
                    "https://dev.azure.com/{}/{}/_git/{}",
                    parts[0], parts[1], parts[2]
                )
            }
        } else if ssh {
            format!("git@{host}:{path}.git")
        } else {
            format!("https://{host}/{path}.git")
        };
        ramo_core::cloning::validate_url(&url)
    }
}
pub fn suggested(value: &str) -> String {
    let component = value
        .trim()
        .trim_end_matches('/')
        .rsplit(['/', ':', '\\'])
        .next()
        .unwrap_or("");
    let name = component.strip_suffix(".git").unwrap_or(component);
    if !name.is_empty()
        && name.len() <= 100
        && name.starts_with(|c: char| c.is_ascii_alphanumeric())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
    {
        name.into()
    } else {
        String::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn provider_shortcuts_generate_only_validated_urls_and_do_not_accept_injected_hosts() {
        assert_eq!(
            Provider::Github
                .url("meadow/search.git", "", false)
                .unwrap(),
            "https://github.com/meadow/search.git"
        );
        assert_eq!(
            Provider::Gitlab.url("group/team/search", "", true).unwrap(),
            "git@gitlab.com:group/team/search.git"
        );
        assert_eq!(
            Provider::Azure
                .url("org/project/search", "", false)
                .unwrap(),
            "https://dev.azure.com/org/project/_git/search"
        );
        assert_eq!(
            Provider::Azure.url("org/project/search", "", true).unwrap(),
            "git@ssh.dev.azure.com:v3/org/project/search"
        );
        for host in [
            "-option",
            "github.com@evil.invalid",
            "good.invalid/path",
            "https://git.invalid",
            "good..invalid",
        ] {
            assert!(
                Provider::Enterprise
                    .url("meadow/search", host, false)
                    .is_err()
            );
        }
        for path in ["-x/repo", "team/../repo", "team/repo?token=x", "one"] {
            assert!(Provider::Github.url(path, "", false).is_err());
        }
        assert!(
            Provider::Url
                .url("https://token@github.com/meadow/search.git", "", false)
                .is_err()
        );
        assert_eq!(suggested("git@github.com:meadow/search.git"), "search");
        assert_eq!(suggested("/synthetic/Meadow Docs"), "");
    }
}

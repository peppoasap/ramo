//! Connection jobs reuse core grants/validation; widgets never execute commands.
use ramo_core::{cloning, git::Backend, hosting, remotes};
pub enum Job {
    Pick {
        source: bool,
    },
    Prepare {
        parent: String,
        directory: String,
        source: cloning::Source,
    },
    Clone(String),
    Catalog {
        host: String,
        organization: String,
        page: u32,
    },
    Remotes(String),
}
pub enum Reply {
    Grant {
        source: bool,
        grant: Option<cloning::Grant>,
    },
    Plan(cloning::Plan),
    Cloned(String),
    Catalog(Vec<hosting::Repository>),
    Remotes(Vec<remotes::Remote>),
}
pub fn execute(backend: &mut Backend, job: Job) -> Result<Reply, String> {
    Ok(match job {
        Job::Pick { source } => {
            let title = if source {
                "Choose local repository root or bare source"
            } else {
                "Choose clone parent folder"
            };
            let grant = rfd::FileDialog::new()
                .set_title(title)
                .pick_folder()
                .map(|path| {
                    if source {
                        backend.authorize_clone_source(&path)
                    } else {
                        backend.authorize_clone_parent(&path)
                    }
                })
                .transpose()?;
            Reply::Grant { source, grant }
        }
        Job::Prepare {
            parent,
            directory,
            source,
        } => Reply::Plan(backend.prepare_clone(&parent, &directory, source)?),
        Job::Clone(plan) => Reply::Cloned(backend.clone_repository(&plan)?.repo.path),
        Job::Catalog {
            host,
            organization,
            page,
        } => Reply::Catalog(hosting::github_repositories(&host, &organization, page)?),
        Job::Remotes(repo) => Reply::Remotes(backend.remote_list(&repo)?),
    })
}

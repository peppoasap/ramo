//! One bounded worker owns the existing backend and typed, fingerprint-guarded operations.
use crate::drafts::{self, Loaded, Scope};
use eframe::egui;
use ramo_core::git::{Backend, Commit, ConflictFile, FileChange, Snapshot};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, SyncSender},
};

pub enum Work {
    Connection(crate::connection_work::Job),
    Open(Option<PathBuf>),
    Activate(String),
    Close(String),
    RecentList,
    Recent(String),
    OpenWorktree {
        repo: String,
        path: String,
    },
    WorktreeParent {
        repo: String,
        path: Option<PathBuf>,
    },
    Refresh(String),
    Trust(String),
    Mutate {
        repo: String,
        fingerprint: String,
        action: ramo_core::git::Action,
    },
    Page {
        repo: String,
        fingerprint: String,
        skip: usize,
    },
    Commit {
        repo: String,
        oid: String,
    },
    Diff {
        repo: String,
        oid: Option<String>,
        path: String,
        staged: bool,
    },
    Conflict {
        repo: String,
        fingerprint: String,
        path: String,
    },
    AmendMessage {
        repo: String,
        head: String,
    },
    Compare {
        repo: String,
        from: String,
        to: String,
        kind: ramo_core::compare::Kind,
    },
    ComparisonDiff {
        repo: String,
        from: String,
        to: String,
        path: String,
    },
    CommitDraftLoad(Scope),
    CommitDraftSave {
        scope: Scope,
        text: String,
        expected: Option<String>,
    },
    DraftLoad(Scope),
    DraftSave {
        scope: Scope,
        text: String,
        expected: Option<String>,
    },
}
pub struct Request {
    pub sequence: u64,
    pub epoch: u64,
    pub work: Work,
}
pub enum Payload {
    Recent(Vec<String>),
    Closed(String),
    WorktreeParent(ramo_core::git::WorktreeParent),
    Connection(Result<crate::connection_work::Reply, String>),
    Opened {
        snapshot: Box<Snapshot>,
        commits: Vec<Commit>,
    },
    Changed {
        snapshot: Box<Snapshot>,
        commits: Vec<Commit>,
        error: Option<String>,
    },
    Trusted(Box<Snapshot>),
    Cancelled,
    Page {
        skip: usize,
        commits: Vec<Commit>,
    },
    Commit {
        oid: String,
        files: Vec<FileChange>,
        diff: String,
    },
    Diff(String),
    Conflict {
        path: String,
        file: ConflictFile,
    },
    AmendMessage {
        head: String,
        message: String,
    },
    Compared(Result<ramo_core::compare::Comparison, String>),
    ComparisonDiff(Result<String, String>),
    CommitDraftLoaded(Option<Loaded>),
    CommitDraftSaved(Loaded),
    DraftLoaded(Option<Loaded>),
    DraftSaved(Loaded),
}
pub struct Reply {
    pub sequence: u64,
    pub epoch: u64,
    pub result: Result<Payload, String>,
}

pub struct Worker {
    pub sender: SyncSender<Request>,
    pub receiver: Receiver<Reply>,
}
impl Worker {
    pub fn start(state: PathBuf, ctx: egui::Context) -> Self {
        let (tx, rx) = mpsc::sync_channel::<Request>(32);
        let (out, answers) = mpsc::channel();
        std::thread::spawn(move || {
            let mut backend = Backend::new(state.join("repositories.json"));
            while let Ok(request) = rx.recv() {
                let result = execute(&mut backend, &state, request.work);
                if out
                    .send(Reply {
                        sequence: request.sequence,
                        epoch: request.epoch,
                        result,
                    })
                    .is_err()
                {
                    break;
                }
                ctx.request_repaint();
            }
        });
        Self {
            sender: tx,
            receiver: answers,
        }
    }
}

fn execute(backend: &mut Backend, state: &std::path::Path, work: Work) -> Result<Payload, String> {
    Ok(match work {
        Work::Connection(job) => Payload::Connection(crate::connection_work::execute(backend, job)),
        Work::Open(path) => {
            let Some(path) = path.or_else(|| {
                rfd::FileDialog::new()
                    .set_title("Open Git repository")
                    .pick_folder()
            }) else {
                return Ok(Payload::Cancelled);
            };
            // Already-open canonical repositories retain their session trust.
            // Explicit Close removes the registration; preferences never store trust.
            let snapshot = backend.register(&path)?;
            let commits = backend.history(&snapshot.repo.id, 0)?;
            Payload::Opened {
                snapshot: Box::new(snapshot),
                commits,
            }
        }
        Work::OpenWorktree { repo, path } => {
            let snapshot = backend.open_worktree(&repo, &path)?;
            let commits = backend.history(&snapshot.repo.id, 0)?;
            Payload::Opened {
                snapshot: Box::new(snapshot),
                commits,
            }
        }
        Work::Activate(repo) => {
            let snapshot = backend.snapshot(&repo)?;
            let commits = backend.history(&repo, 0)?;
            Payload::Opened {
                snapshot: Box::new(snapshot),
                commits,
            }
        }
        Work::Close(repo) => {
            backend.close_repository(&repo);
            Payload::Closed(repo)
        }
        Work::RecentList => Payload::Recent(
            backend
                .preferences
                .recent
                .iter()
                .take(20)
                .cloned()
                .collect(),
        ),
        Work::Recent(path) => {
            let snapshot = backend.recent(&path)?;
            let commits = backend.history(&snapshot.repo.id, 0)?;
            Payload::Opened {
                snapshot: Box::new(snapshot),
                commits,
            }
        }
        Work::WorktreeParent { repo, path } => {
            let Some(path) = path.or_else(|| {
                rfd::FileDialog::new()
                    .set_title("Choose worktree parent folder")
                    .pick_folder()
            }) else {
                return Ok(Payload::Cancelled);
            };
            Payload::WorktreeParent(backend.authorize_worktree_parent(&repo, &path)?)
        }
        Work::Trust(repo) => Payload::Trusted(Box::new(backend.trust(&repo)?)),
        Work::Mutate {
            repo,
            fingerprint,
            action,
        } => {
            // A failed merge/network command may have changed Git state. Always
            // return the authoritative snapshot, never leave the old UI active.
            let result = backend.mutate(&repo, &fingerprint, action);
            let (snapshot, error) = match result {
                Ok(snapshot) => (snapshot, None),
                Err(error) => (backend.snapshot(&repo)?, Some(error)),
            };
            let (commits, error) = match backend.history(&repo, 0) {
                Ok(commits) => (commits, error),
                Err(history_error) => (
                    Vec::new(),
                    Some(format!(
                        "{} History refresh failed: {history_error}. Refresh before another operation.",
                        error.unwrap_or_else(|| "Git operation completed.".into())
                    )),
                ),
            };
            Payload::Changed {
                snapshot: Box::new(snapshot),
                commits,
                error,
            }
        }
        Work::Refresh(repo) => {
            let snapshot = backend.snapshot(&repo)?;
            let commits = backend.history(&repo, 0)?;
            Payload::Opened {
                snapshot: Box::new(snapshot),
                commits,
            }
        }
        Work::Page {
            repo,
            fingerprint,
            skip,
        } => {
            if backend.snapshot(&repo)?.fingerprint != fingerprint {
                return Err(
                    "Repository changed while loading history. Refresh before loading more.".into(),
                );
            }
            Payload::Page {
                skip,
                commits: backend.history(&repo, skip)?,
            }
        }
        Work::Commit { repo, oid } => {
            let files = backend.commit_files(&repo, &oid)?;
            let diff = if let Some(file) = files.first() {
                backend.file_diff(&repo, &file.path, Some(&oid), false)?
            } else {
                String::new()
            };
            Payload::Commit { oid, files, diff }
        }
        Work::Diff {
            repo,
            oid,
            path,
            staged,
        } => Payload::Diff(backend.file_diff(&repo, &path, oid.as_deref(), staged)?),
        Work::Conflict {
            repo,
            fingerprint,
            path,
        } => Payload::Conflict {
            file: backend.conflict_file(&repo, &fingerprint, &path)?,
            path,
        },
        Work::AmendMessage { repo, head } => Payload::AmendMessage {
            message: backend.amend_message(&repo, &head)?,
            head,
        },
        Work::Compare {
            repo,
            from,
            to,
            kind,
        } => Payload::Compared(backend.compare_revisions(&repo, &from, &to, kind)),
        Work::ComparisonDiff {
            repo,
            from,
            to,
            path,
        } => Payload::ComparisonDiff(backend.comparison_diff(
            &repo,
            &from,
            &to,
            ramo_core::compare::Kind::Trees,
            &path,
        )),
        Work::CommitDraftLoad(scope) => {
            Payload::CommitDraftLoaded(drafts::load(&state.join("commit-drafts"), &scope)?)
        }
        Work::CommitDraftSave {
            scope,
            text,
            expected,
        } => {
            if text.len() > 32768 {
                return Err(
                    "Commit message exceeds32KiB. Shorten it or copy it before closing.".into(),
                );
            }
            Payload::CommitDraftSaved(drafts::save(
                &state.join("commit-drafts"),
                scope,
                text,
                expected,
            )?)
        }
        Work::DraftLoad(scope) => {
            Payload::DraftLoaded(drafts::load(&state.join("merge-drafts"), &scope)?)
        }
        Work::DraftSave {
            scope,
            text,
            expected,
        } => Payload::DraftSaved(drafts::save(
            &state.join("merge-drafts"),
            scope,
            text,
            expected,
        )?),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};
    fn git(root: &Path, args: &[&str]) -> Vec<u8> {
        ramo_core::git::git(root, args).unwrap()
    }
    fn fixture() -> (tempfile::TempDir, PathBuf, Backend) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("meadow");
        fs::create_dir(&root).unwrap();
        git(&root, &["init", "-b", "main"]);
        for (key, value) in [
            ("user.name", "Demo Contributor"),
            ("user.email", "demo@example.invalid"),
            ("commit.gpgsign", "false"),
            ("core.autocrlf", "false"),
            ("core.fsmonitor", "false"),
        ] {
            git(&root, &["config", key, value]);
        }
        let hooks = root.join(".git/empty-hooks");
        fs::create_dir(&hooks).unwrap();
        git(
            &root,
            &["config", "core.hooksPath", hooks.to_str().unwrap()],
        );
        fs::write(root.join("search.txt"), b"base\n").unwrap();
        git(&root, &["add", "search.txt"]);
        git(&root, &["commit", "-m", "Initial search"]);
        let backend = Backend::new(temp.path().join("app-state/repositories.json"));
        (temp, root, backend)
    }
    fn proof(root: &Path) -> Vec<Vec<u8>> {
        vec![
            git(root, &["rev-parse", "HEAD"]),
            fs::read(root.join(".git/index")).unwrap(),
            git(root, &["status", "--porcelain=v1", "-z"]),
            fs::read(root.join("search.txt")).unwrap(),
        ]
    }
    #[test]
    fn native_tabs_keep_live_trust_until_close_and_recents_reopen_without_it() {
        let (temp, root, mut backend) = fixture();
        let (_other_temp, other_root, _) = fixture();
        let Payload::Opened {
            snapshot: first, ..
        } = execute(&mut backend, temp.path(), Work::Open(Some(root.clone()))).unwrap()
        else {
            panic!()
        };
        backend.trust(&first.repo.id).unwrap();
        let Payload::Opened {
            snapshot: other, ..
        } = execute(&mut backend, temp.path(), Work::Open(Some(other_root))).unwrap()
        else {
            panic!()
        };
        assert!(!other.repo.trusted);
        fs::write(root.join("search.txt"), b"external local edit\n").unwrap();
        let before = proof(&root);
        let Payload::Opened {
            snapshot: active, ..
        } = execute(
            &mut backend,
            temp.path(),
            Work::Activate(first.repo.id.clone()),
        )
        .unwrap()
        else {
            panic!()
        };
        assert!(active.repo.trusted);
        assert!(!active.files.is_empty());
        let Payload::Opened {
            snapshot: duplicate,
            ..
        } = execute(&mut backend, temp.path(), Work::Open(Some(root.clone()))).unwrap()
        else {
            panic!()
        };
        assert_eq!(duplicate.repo.id, first.repo.id);
        assert!(duplicate.repo.trusted);
        execute(
            &mut backend,
            temp.path(),
            Work::Close(first.repo.id.clone()),
        )
        .unwrap();
        assert!(
            execute(
                &mut backend,
                temp.path(),
                Work::Activate(first.repo.id.clone())
            )
            .is_err()
        );
        assert!(backend.repository(&other.repo.id).is_ok());
        let Payload::Opened {
            snapshot: reopened, ..
        } = execute(
            &mut backend,
            temp.path(),
            Work::Recent(first.repo.path.clone()),
        )
        .unwrap()
        else {
            panic!()
        };
        assert!(!reopened.repo.trusted);
        assert_eq!(proof(&root), before);
        drop(backend);
        let mut restarted = Backend::new(temp.path().join("app-state/repositories.json"));
        let Payload::Recent(paths) =
            execute(&mut restarted, temp.path(), Work::RecentList).unwrap()
        else {
            panic!()
        };
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], first.repo.path);
        assert!(restarted.repository(&first.repo.id).is_err());
        assert!(
            execute(
                &mut restarted,
                temp.path(),
                Work::Recent(temp.path().to_string_lossy().into())
            )
            .is_err()
        );
        let Payload::Opened {
            snapshot: reopened, ..
        } = execute(&mut restarted, temp.path(), Work::Recent(first.repo.path)).unwrap()
        else {
            panic!()
        };
        assert!(!reopened.repo.trusted);
        assert_eq!(proof(&root), before);
    }
    #[test]
    fn native_worktree_jobs_reuse_grants_membership_and_preserve_source_work() {
        let (temp, root, mut backend) = fixture();
        // Commit EOL policy so a linked checkout does not inherit runner defaults.
        fs::write(root.join(".gitattributes"), b"*.txt text eol=lf\n").unwrap();
        git(&root, &["add", ".gitattributes"]);
        git(&root, &["commit", "-m", "Pin checkout EOL"]);
        fs::write(root.join("search.txt"), b"staged\n").unwrap();
        git(&root, &["add", "search.txt"]);
        fs::write(root.join("search.txt"), b"working\n").unwrap();
        let snapshot = backend.register(&root).unwrap();
        let before = proof(&root);
        assert!(
            execute(
                &mut backend,
                temp.path(),
                Work::WorktreeParent {
                    repo: snapshot.repo.id.clone(),
                    path: Some(root.clone())
                }
            )
            .is_err()
        );
        let Payload::WorktreeParent(parent) = execute(
            &mut backend,
            temp.path(),
            Work::WorktreeParent {
                repo: snapshot.repo.id.clone(),
                path: Some(temp.path().to_owned()),
            },
        )
        .unwrap() else {
            panic!("parent grant expected")
        };
        let action = |grant: String| ramo_core::git::Action::CreateWorktree {
            selection_id: grant,
            directory: "linked".into(),
            branch: "work/search".into(),
            create_branch: true,
        };
        let Payload::Changed { error: Some(_), .. } = execute(
            &mut backend,
            temp.path(),
            Work::Mutate {
                repo: snapshot.repo.id.clone(),
                fingerprint: snapshot.fingerprint.clone(),
                action: action(parent.id.clone()),
            },
        )
        .unwrap() else {
            panic!("trust required")
        };
        let trusted = backend.trust(&snapshot.repo.id).unwrap();
        let Payload::Changed { error: Some(_), .. } = execute(
            &mut backend,
            temp.path(),
            Work::Mutate {
                repo: snapshot.repo.id.clone(),
                fingerprint: trusted.fingerprint.clone(),
                action: action("forged".into()),
            },
        )
        .unwrap() else {
            panic!("grant required")
        };
        fs::create_dir(temp.path().join("linked")).unwrap();
        fs::write(temp.path().join("linked/keep.txt"), b"collision sentinel").unwrap();
        let Payload::Changed { error: Some(_), .. } = execute(
            &mut backend,
            temp.path(),
            Work::Mutate {
                repo: snapshot.repo.id.clone(),
                fingerprint: trusted.fingerprint.clone(),
                action: action(parent.id.clone()),
            },
        )
        .unwrap() else {
            panic!("collision must fail")
        };
        assert_eq!(
            fs::read(temp.path().join("linked/keep.txt")).unwrap(),
            b"collision sentinel"
        );
        fs::remove_file(temp.path().join("linked/keep.txt")).unwrap();
        fs::remove_dir(temp.path().join("linked")).unwrap();
        let Payload::Changed {
            snapshot: created,
            error: None,
            ..
        } = execute(
            &mut backend,
            temp.path(),
            Work::Mutate {
                repo: snapshot.repo.id.clone(),
                fingerprint: trusted.fingerprint,
                action: action(parent.id),
            },
        )
        .unwrap()
        else {
            panic!("creation expected")
        };
        assert_eq!(created.worktrees.len(), 2);
        assert_eq!(
            fs::read(temp.path().join("linked/search.txt")).unwrap(),
            b"base\n"
        );
        let unrelated = temp.path().join("unrelated");
        fs::create_dir(&unrelated).unwrap();
        git(&unrelated, &["init", "-b", "main"]);
        assert!(
            execute(
                &mut backend,
                temp.path(),
                Work::OpenWorktree {
                    repo: snapshot.repo.id.clone(),
                    path: unrelated.to_str().unwrap().into()
                }
            )
            .is_err()
        );
        let Payload::Opened {
            snapshot: linked, ..
        } = execute(
            &mut backend,
            temp.path(),
            Work::OpenWorktree {
                repo: snapshot.repo.id.clone(),
                path: temp.path().join("linked").to_str().unwrap().into(),
            },
        )
        .unwrap()
        else {
            panic!("open expected")
        };
        assert!(!linked.repo.trusted);
        assert_eq!(linked.branch, "work/search");
        let fresh = backend.snapshot(&snapshot.repo.id).unwrap();
        let linked_path = created
            .worktrees
            .iter()
            .find(|w| !w.primary)
            .unwrap()
            .path
            .clone();
        let Payload::Changed { error: Some(_), .. } = execute(
            &mut backend,
            temp.path(),
            Work::Mutate {
                repo: snapshot.repo.id.clone(),
                fingerprint: fresh.fingerprint,
                action: ramo_core::git::Action::RemoveWorktree { path: linked_path },
            },
        )
        .unwrap() else {
            panic!("open checkout must not be removed")
        };
        backend.close_repository(&linked.repo.id);
        let fresh = backend.snapshot(&snapshot.repo.id).unwrap();
        fs::write(temp.path().join("linked/keep.txt"), b"untracked sentinel").unwrap();
        let reviewed_path = created
            .worktrees
            .iter()
            .find(|w| !w.primary)
            .unwrap()
            .path
            .clone();
        let remove = || ramo_core::git::Action::RemoveWorktree {
            path: reviewed_path.clone(),
        };
        let Payload::Changed { error: Some(_), .. } = execute(
            &mut backend,
            temp.path(),
            Work::Mutate {
                repo: snapshot.repo.id.clone(),
                fingerprint: fresh.fingerprint.clone(),
                action: remove(),
            },
        )
        .unwrap() else {
            panic!("untracked data must block removal")
        };
        assert_eq!(
            fs::read(temp.path().join("linked/keep.txt")).unwrap(),
            b"untracked sentinel"
        );
        fs::remove_file(temp.path().join("linked/keep.txt")).unwrap();
        let fresh = backend.snapshot(&snapshot.repo.id).unwrap();
        let Payload::Changed { error, .. } = execute(
            &mut backend,
            temp.path(),
            Work::Mutate {
                repo: snapshot.repo.id,
                fingerprint: fresh.fingerprint,
                action: remove(),
            },
        )
        .unwrap() else {
            panic!("clean removal expected")
        };
        assert!(error.is_none(), "{error:?}");
        assert!(!temp.path().join("linked").exists());
        assert_eq!(proof(&root), before);
    }
    #[test]
    fn native_local_clone_review_is_read_only_one_shot_and_collision_safe() {
        use crate::connection_work::{Job, Reply};
        let (temp, root, mut backend) = fixture();
        // A clone has its own Git configuration. Pin the fixture's checkout EOL
        // in committed attributes rather than assuming the runner's autocrlf.
        fs::write(root.join(".gitattributes"), b"*.txt text eol=lf\n").unwrap();
        git(&root, &["add", ".gitattributes"]);
        git(
            &root,
            &["commit", "-m", "Pin synthetic checkout line endings"],
        );
        fs::write(root.join("search.txt"), b"uncommitted source stays here\n").unwrap();
        let original = backend.register(&root).unwrap();
        let before = proof(&root);
        // Connection jobs also work with no current/registered repository.
        let mut backend = Backend::new(temp.path().join("fresh-native-state.json"));
        let parent = temp.path().join("clones");
        fs::create_dir(&parent).unwrap();
        let grant = backend.authorize_clone_parent(&parent).unwrap();
        let source = backend.authorize_clone_source(&root).unwrap();
        let prepare = |directory: &str| {
            Work::Connection(Job::Prepare {
                parent: grant.id.clone(),
                directory: directory.into(),
                source: ramo_core::cloning::Source::Local {
                    id: source.id.clone(),
                },
            })
        };
        let Payload::Connection(Ok(Reply::Plan(plan))) =
            execute(&mut backend, temp.path(), prepare("meadow-copy")).unwrap()
        else {
            panic!("clone review expected")
        };
        assert!(!parent.join("meadow-copy").exists());
        assert_eq!(proof(&root), before);
        let Payload::Connection(Ok(Reply::Cloned(path))) = execute(
            &mut backend,
            temp.path(),
            Work::Connection(Job::Clone(plan.id.clone())),
        )
        .unwrap() else {
            panic!("local clone expected")
        };
        let cloned = PathBuf::from(path);
        assert_eq!(
            git(&cloned, &["rev-parse", "HEAD"]),
            git(&root, &["rev-parse", "HEAD"])
        );
        assert_eq!(fs::read(cloned.join("search.txt")).unwrap(), b"base\n");
        assert!(!backend.register(&cloned).unwrap().repo.trusted);
        assert!(!original.repo.trusted);
        assert_eq!(proof(&root), before);
        assert!(matches!(
            execute(
                &mut backend,
                temp.path(),
                Work::Connection(Job::Clone(plan.id))
            )
            .unwrap(),
            Payload::Connection(Err(_))
        ));
        let Payload::Connection(Ok(Reply::Plan(collision))) =
            execute(&mut backend, temp.path(), prepare("collision")).unwrap()
        else {
            panic!("review expected")
        };
        fs::create_dir(parent.join("collision")).unwrap();
        fs::write(parent.join("collision/keep.txt"), b"keep me").unwrap();
        assert!(matches!(
            execute(
                &mut backend,
                temp.path(),
                Work::Connection(Job::Clone(collision.id))
            )
            .unwrap(),
            Payload::Connection(Err(_))
        ));
        assert_eq!(
            fs::read(parent.join("collision/keep.txt")).unwrap(),
            b"keep me"
        );
        assert!(matches!(
            execute(
                &mut backend,
                temp.path(),
                Work::Connection(Job::Prepare {
                    parent: "forged-grant".into(),
                    directory: "other".into(),
                    source: ramo_core::cloning::Source::Url {
                        url: "https://github.com/meadow/search.git".into()
                    }
                })
            )
            .unwrap(),
            Payload::Connection(Err(_))
        ));
        assert_eq!(proof(&root), before);
    }
    #[test]
    fn native_remote_jobs_reuse_trust_freshness_redaction_and_preserve_local_work() {
        use crate::connection_work::{Job, Reply};
        let (temp, root, mut backend) = fixture();
        fs::write(root.join("search.txt"), b"staged local\n").unwrap();
        git(&root, &["add", "search.txt"]);
        fs::write(root.join("search.txt"), b"unstaged local\n").unwrap();
        let snapshot = backend.register(&root).unwrap();
        let before = proof(&root);
        let action = || ramo_core::git::Action::ConfigureRemote {
            name: "origin".into(),
            url: "https://github.com/meadow/search.git".into(),
            existing: false,
        };
        assert!(matches!(
            execute(
                &mut backend,
                temp.path(),
                Work::Mutate {
                    repo: snapshot.repo.id.clone(),
                    fingerprint: snapshot.fingerprint.clone(),
                    action: action()
                }
            )
            .unwrap(),
            Payload::Changed { error: Some(_), .. }
        ));
        let trusted = backend.trust(&snapshot.repo.id).unwrap();
        let Payload::Changed {
            snapshot: saved,
            error: None,
            ..
        } = execute(
            &mut backend,
            temp.path(),
            Work::Mutate {
                repo: trusted.repo.id.clone(),
                fingerprint: trusted.fingerprint.clone(),
                action: action(),
            },
        )
        .unwrap()
        else {
            panic!("remote saved")
        };
        let Payload::Connection(Ok(Reply::Remotes(rows))) = execute(
            &mut backend,
            temp.path(),
            Work::Connection(Job::Remotes(snapshot.repo.id.clone())),
        )
        .unwrap() else {
            panic!("remote list")
        };
        assert_eq!(rows.len(), 1);
        assert!(rows[0].editable);
        assert_eq!(rows[0].fetch, ["https://github.com/meadow/search.git"]);
        assert!(matches!(
            execute(
                &mut backend,
                temp.path(),
                Work::Mutate {
                    repo: snapshot.repo.id.clone(),
                    fingerprint: trusted.fingerprint,
                    action: ramo_core::git::Action::ConfigureRemote {
                        name: "origin".into(),
                        url: "https://github.com/meadow/other.git".into(),
                        existing: true
                    }
                }
            )
            .unwrap(),
            Payload::Changed { error: Some(_), .. }
        ));
        assert_ne!(saved.fingerprint, snapshot.fingerprint);
        assert_eq!(proof(&root), before);
        git(
            &root,
            &[
                "config",
                "remote.origin.pushurl",
                "https://github.com/meadow/push.git",
            ],
        );
        git(
            &root,
            &[
                "remote",
                "add",
                "synthetic-secret",
                "https://synthetic-token@github.com/meadow/private.git",
            ],
        );
        let config = fs::read(root.join(".git/config")).unwrap();
        let current = backend.snapshot(&snapshot.repo.id).unwrap();
        let Payload::Connection(Ok(Reply::Remotes(rows))) = execute(
            &mut backend,
            temp.path(),
            Work::Connection(Job::Remotes(snapshot.repo.id.clone())),
        )
        .unwrap() else {
            panic!("remote list")
        };
        assert!(!rows.iter().find(|r| r.name == "origin").unwrap().editable);
        assert!(
            rows.iter()
                .find(|r| r.name == "synthetic-secret")
                .unwrap()
                .fetch[0]
                .starts_with("[Local, custom")
        );
        assert!(matches!(
            execute(
                &mut backend,
                temp.path(),
                Work::Mutate {
                    repo: snapshot.repo.id,
                    fingerprint: current.fingerprint,
                    action: ramo_core::git::Action::ConfigureRemote {
                        name: "origin".into(),
                        url: "https://github.com/meadow/other.git".into(),
                        existing: true
                    }
                }
            )
            .unwrap(),
            Payload::Changed { error: Some(_), .. }
        ));
        assert_eq!(fs::read(root.join(".git/config")).unwrap(), config);
        assert_eq!(proof(&root), before);
    }
    #[test]
    fn native_drag_intents_bind_source_context_and_never_mutate_git() {
        use crate::gestures::{Drag, Intent, Source};
        let (temp, root, mut backend) = fixture();
        git(&root, &["branch", "side"]);
        git(&root, &["update-ref", "refs/remotes/origin/side", "HEAD"]);
        fs::write(root.join("search.txt"), b"next\n").unwrap();
        git(&root, &["add", "search.txt"]);
        git(&root, &["commit", "-m", "Next search"]);
        let snapshot = backend.register(&root).unwrap();
        let side = snapshot.branches.iter().find(|b| b.name == "side").unwrap();
        let branch = Drag::new(
            &snapshot,
            Source::Branch {
                oid: side.oid.clone(),
                name: side.name.clone(),
                remote: false,
            },
        );
        let head = snapshot.head.as_deref().unwrap();
        let before = proof(&root);
        assert!(
            matches!(branch.intent(&snapshot,Some(head),false).unwrap(),Intent::Compare {from,to} if from==side.oid&&to==head)
        );
        assert!(
            matches!(branch.intent(&snapshot,None,false).unwrap(),Intent::Review {action:ramo_core::git::Action::Merge {name},..} if name=="side")
        );
        assert!(branch.intent(&snapshot, Some(head), true).is_err());
        assert!(branch.intent(&snapshot, Some(&side.oid), false).is_err());
        let mut stale = branch.clone();
        stale.fingerprint = "old".into();
        assert!(stale.intent(&snapshot, Some(head), false).is_err());
        stale = branch.clone();
        stale.repository = "another repository".into();
        assert!(stale.intent(&snapshot, None, false).is_err());
        stale = branch.clone();
        if let Source::Branch { oid, .. } = &mut stale.source {
            *oid = head.into();
        }
        assert!(stale.intent(&snapshot, Some(head), false).is_err());
        let commit = Drag::new(
            &snapshot,
            Source::Commit {
                oid: side.oid.clone(),
                parents: 0,
                subject: "Initial".into(),
            },
        );
        assert!(
            matches!(commit.intent(&snapshot,None,false).unwrap(),Intent::Review {action:ramo_core::git::Action::CherryPick{target},..} if target==side.oid)
        );
        let merged = Drag::new(
            &snapshot,
            Source::Commit {
                oid: side.oid.clone(),
                parents: 2,
                subject: "Merge".into(),
            },
        );
        assert!(merged.intent(&snapshot, None, false).is_err());
        let current = Drag::new(
            &snapshot,
            Source::Commit {
                oid: head.into(),
                parents: 1,
                subject: "Current".into(),
            },
        );
        assert!(current.intent(&snapshot, None, false).is_err());
        let main = snapshot.branches.iter().find(|b| b.name == "main").unwrap();
        let current_branch = Drag::new(
            &snapshot,
            Source::Branch {
                oid: main.oid.clone(),
                name: main.name.clone(),
                remote: false,
            },
        );
        assert!(current_branch.intent(&snapshot, None, false).is_err());
        assert_eq!(proof(&root), before);
        // Comparison remains available with dirty files; mutation intent does not.
        fs::write(root.join("search.txt"), b"keep local work\n").unwrap();
        let dirty = backend.snapshot(&snapshot.repo.id).unwrap();
        let branch = Drag::new(&dirty, branch.source);
        assert!(branch.intent(&dirty, None, false).is_err());
        assert!(matches!(
            branch.intent(&dirty, Some(head), false).unwrap(),
            Intent::Compare { .. }
        ));
        let mut detached = dirty;
        detached.files.clear();
        detached.branch = "Detached HEAD".into();
        let commit = Drag::new(&detached, commit.source);
        assert!(commit.intent(&detached, None, false).is_err());
        detached.branch = "main".into();
        detached.operation = Some("Merge in progress".into());
        let commit = Drag::new(&detached, commit.source);
        assert!(commit.intent(&detached, None, false).is_err());
        drop(temp);
    }
    #[test]
    fn native_comparison_uses_immutable_pairs_and_preserves_dirty_index_and_work() {
        let (temp, root, mut backend) = fixture();
        let initial = String::from_utf8(git(&root, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
        git(&root, &["checkout", "-b", "left"]);
        fs::write(root.join("left.txt"), b"left only\n").unwrap();
        git(&root, &["add", "left.txt"]);
        git(&root, &["commit", "-m", "Left branch"]);
        let left = String::from_utf8(git(&root, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
        git(&root, &["checkout", "main"]);
        fs::write(root.join("search.txt"), "target 日本語\n".as_bytes()).unwrap();
        git(&root, &["add", "search.txt"]);
        git(&root, &["commit", "-m", "Target search"]);
        let target = String::from_utf8(git(&root, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
        fs::write(root.join("search.txt"), b"staged private fixture\n").unwrap();
        git(&root, &["add", "search.txt"]);
        fs::write(root.join("search.txt"), b"unstaged private fixture\n").unwrap();
        let snapshot = backend.register(&root).unwrap();
        assert!(!snapshot.repo.trusted);
        git(&root, &["update-ref", "refs/heads/left", &target]);
        let before = proof(&root);
        let Payload::Compared(direct) = execute(
            &mut backend,
            temp.path(),
            Work::Compare {
                repo: snapshot.repo.id.clone(),
                from: left.clone(),
                to: target.clone(),
                kind: ramo_core::compare::Kind::Trees,
            },
        )
        .unwrap() else {
            panic!("comparison expected")
        };
        let direct = direct.unwrap();
        assert_eq!(direct.effective_from, left);
        assert!(direct.files.iter().any(|f| f.path == "left.txt"));
        let Payload::Compared(ancestor) = execute(
            &mut backend,
            temp.path(),
            Work::Compare {
                repo: snapshot.repo.id.clone(),
                from: left,
                to: target.clone(),
                kind: ramo_core::compare::Kind::MergeBase,
            },
        )
        .unwrap() else {
            panic!("comparison expected")
        };
        let ancestor = ancestor.unwrap();
        assert_eq!(ancestor.effective_from, initial);
        assert_eq!(ancestor.files.len(), 1);
        let Payload::ComparisonDiff(diff) = execute(
            &mut backend,
            temp.path(),
            Work::ComparisonDiff {
                repo: snapshot.repo.id.clone(),
                from: ancestor.effective_from,
                to: target.clone(),
                path: "search.txt".into(),
            },
        )
        .unwrap() else {
            panic!("diff expected")
        };
        let diff = diff.unwrap();
        assert!(diff.contains("+target 日本語"));
        assert!(!diff.contains("private fixture"));
        assert!(matches!(
            execute(
                &mut backend,
                temp.path(),
                Work::Compare {
                    repo: snapshot.repo.id,
                    from: "HEAD~1".into(),
                    to: target,
                    kind: ramo_core::compare::Kind::Trees
                }
            )
            .unwrap(),
            Payload::Compared(Err(_))
        ));
        assert_eq!(proof(&root), before);
    }
    #[test]
    fn browsing_dirty_staged_and_unstaged_data_is_read_only_and_stale_pages_fail() {
        let (temp, root, mut backend) = fixture();
        fs::write(root.join("search.txt"), b"staged\r\n").unwrap();
        git(&root, &["add", "search.txt"]);
        fs::write(
            root.join("search.txt"),
            "working 日本語\r\nno final newline",
        )
        .unwrap();
        fs::write(root.join("untracked.txt"), "keep me").unwrap();
        let before = proof(&root);
        let Payload::Opened { snapshot, commits } =
            execute(&mut backend, temp.path(), Work::Open(Some(root.clone()))).unwrap()
        else {
            panic!()
        };
        assert!(!snapshot.repo.trusted);
        assert_eq!(commits.len(), 1);
        let repo = snapshot.repo.id.clone();
        let Payload::Diff(staged) = execute(
            &mut backend,
            temp.path(),
            Work::Diff {
                repo: repo.clone(),
                oid: None,
                path: "search.txt".into(),
                staged: true,
            },
        )
        .unwrap() else {
            panic!()
        };
        let Payload::Diff(working) = execute(
            &mut backend,
            temp.path(),
            Work::Diff {
                repo: repo.clone(),
                oid: None,
                path: "search.txt".into(),
                staged: false,
            },
        )
        .unwrap() else {
            panic!()
        };
        assert!(staged.contains("+staged"));
        assert!(working.contains("working 日本語"));
        assert_eq!(proof(&root), before);
        assert_eq!(fs::read(root.join("untracked.txt")).unwrap(), b"keep me");
        fs::write(root.join("search.txt"), b"external edit").unwrap();
        let changed = proof(&root);
        assert!(
            execute(
                &mut backend,
                temp.path(),
                Work::Page {
                    repo,
                    fingerprint: snapshot.fingerprint,
                    skip: 300
                }
            )
            .is_err()
        );
        assert_eq!(proof(&root), changed);
    }
    #[test]
    fn conflict_sources_and_native_draft_never_resolve_or_stage_repository_data() {
        let (temp, root, mut backend) = fixture();
        git(&root, &["switch", "-c", "side"]);
        fs::write(root.join("search.txt"), b"incoming\n").unwrap();
        git(&root, &["commit", "-am", "Incoming"]);
        git(&root, &["switch", "main"]);
        fs::write(root.join("search.txt"), b"current\n").unwrap();
        git(&root, &["commit", "-am", "Current"]);
        assert!(ramo_core::git::git(&root, &["merge", "side"]).is_err());
        let before = proof(&root);
        let Payload::Opened { snapshot, .. } =
            execute(&mut backend, temp.path(), Work::Open(Some(root.clone()))).unwrap()
        else {
            panic!()
        };
        let scope = Scope {
            repository: snapshot.repo.id.clone(),
            fingerprint: snapshot.fingerprint.clone(),
            path: "search.txt".into(),
        };
        let Payload::Conflict { file, .. } = execute(
            &mut backend,
            temp.path(),
            Work::Conflict {
                repo: snapshot.repo.id,
                fingerprint: snapshot.fingerprint,
                path: "search.txt".into(),
            },
        )
        .unwrap() else {
            panic!()
        };
        assert_eq!(file.ours.as_deref(), Some("current\n"));
        assert_eq!(file.theirs.as_deref(), Some("incoming\n"));
        execute(
            &mut backend,
            temp.path(),
            Work::DraftSave {
                scope: scope.clone(),
                text: "local resolution 日本語\r\n".into(),
                expected: None,
            },
        )
        .unwrap();
        let Payload::DraftLoaded(Some(loaded)) =
            execute(&mut backend, temp.path(), Work::DraftLoad(scope)).unwrap()
        else {
            panic!()
        };
        assert_eq!(loaded.record.text, "local resolution 日本語\r\n");
        assert_eq!(proof(&root), before);
        assert!(!git(&root, &["ls-files", "-u"]).is_empty());
    }
    fn changed(payload: Payload) -> (Box<Snapshot>, Option<String>) {
        match payload {
            Payload::Changed {
                snapshot, error, ..
            } => (snapshot, error),
            _ => panic!("expected mutation result"),
        }
    }
    fn mutate(
        backend: &mut Backend,
        state: &Path,
        snapshot: &Snapshot,
        action: ramo_core::git::Action,
    ) -> (Box<Snapshot>, Option<String>) {
        changed(
            execute(
                backend,
                state,
                Work::Mutate {
                    repo: snapshot.repo.id.clone(),
                    fingerprint: snapshot.fingerprint.clone(),
                    action,
                },
            )
            .unwrap(),
        )
    }
    #[test]
    fn native_write_dispatch_requires_trust_and_fresh_state_and_preserves_unstaged_bytes() {
        use ramo_core::git::Action;
        let (temp, root, mut backend) = fixture();
        let initial = backend.register(&root).unwrap();
        fs::write(root.join("search.txt"), b"staged change\n").unwrap();
        let current = backend.snapshot(&initial.repo.id).unwrap();
        let (_, error) = mutate(
            &mut backend,
            temp.path(),
            &current,
            Action::Stage {
                paths: vec!["search.txt".into()],
            },
        );
        assert!(error.unwrap().contains("Trust"));
        let trusted = backend.trust(&current.repo.id).unwrap();
        let (_, error) = mutate(
            &mut backend,
            temp.path(),
            &initial,
            Action::Stage {
                paths: vec!["search.txt".into()],
            },
        );
        assert!(error.unwrap().contains("changed"));
        let (staged, error) = mutate(
            &mut backend,
            temp.path(),
            &trusted,
            Action::Stage {
                paths: vec!["search.txt".into()],
            },
        );
        assert!(error.is_none());
        fs::write(root.join("search.txt"), b"keep unstaged\n").unwrap();
        let current = backend.snapshot(&staged.repo.id).unwrap();
        let (committed, error) = mutate(
            &mut backend,
            temp.path(),
            &current,
            Action::Commit {
                message: "Native commit".into(),
            },
        );
        assert!(error.is_none());
        assert_eq!(git(&root, &["show", "HEAD:search.txt"]), b"staged change\n");
        assert_eq!(
            fs::read(root.join("search.txt")).unwrap(),
            b"keep unstaged\n"
        );
        assert!(committed.repo.trusted);
        let Payload::Opened { snapshot, .. } =
            execute(&mut backend, temp.path(), Work::Open(Some(root.clone()))).unwrap()
        else {
            panic!()
        };
        assert!(snapshot.repo.trusted);
        execute(&mut backend, temp.path(), Work::Close(snapshot.repo.id)).unwrap();
        let Payload::Opened { snapshot, .. } =
            execute(&mut backend, temp.path(), Work::Open(Some(root))).unwrap()
        else {
            panic!()
        };
        assert!(!snapshot.repo.trusted);
    }
    #[test]
    fn native_merge_failure_refreshes_conflicts_then_resolution_and_commit_complete_merge() {
        use ramo_core::git::Action;
        let (temp, root, mut backend) = fixture();
        git(&root, &["switch", "-c", "side"]);
        fs::write(root.join("search.txt"), b"incoming\n").unwrap();
        git(&root, &["commit", "-am", "Incoming"]);
        git(&root, &["switch", "main"]);
        fs::write(root.join("search.txt"), b"current\n").unwrap();
        git(&root, &["commit", "-am", "Current"]);

        let s = backend.register(&root).unwrap();
        let s = backend.trust(&s.repo.id).unwrap();
        let (conflicted, error) = mutate(
            &mut backend,
            temp.path(),
            &s,
            Action::Merge {
                name: "side".into(),
            },
        );
        assert!(error.is_some());
        assert!(conflicted.operation.is_some());
        assert!(conflicted.files.iter().any(ramo_core::git::is_conflicted));
        fs::write(root.join("keep.txt"), b"untracked preservation\n").unwrap();
        let conflicted = Box::new(backend.snapshot(&conflicted.repo.id).unwrap());
        let (_, error) = mutate(
            &mut backend,
            temp.path(),
            &conflicted,
            Action::ResolveFile {
                path: "search.txt".into(),
                content: Some("<<<<<<< invalid\n".into()),
            },
        );
        assert!(error.is_some());
        let (resolved, error) = mutate(
            &mut backend,
            temp.path(),
            &conflicted,
            Action::ResolveFile {
                path: "search.txt".into(),
                content: Some("resolved 日本語\n".into()),
            },
        );
        assert!(error.is_none());
        let (done, error) = mutate(
            &mut backend,
            temp.path(),
            &resolved,
            Action::Commit {
                message: "Merge side".into(),
            },
        );
        assert!(error.is_none());
        assert!(done.operation.is_none());
        assert_eq!(
            fs::read(root.join("keep.txt")).unwrap(),
            b"untracked preservation\n"
        );
        assert_eq!(
            git(&root, &["show", "HEAD:search.txt"]),
            "resolved 日本語\n".as_bytes()
        );
        assert_eq!(
            git(&root, &["show", "--format=%P", "-s", "HEAD"])
                .split(|b| *b == b' ')
                .count(),
            2
        );
    }
    #[test]
    fn commit_message_survives_backend_restart_without_git_writes_and_conflicts_preserve_copies() {
        let (temp, root, mut backend) = fixture();
        let snapshot = backend.register(&root).unwrap();
        let before = proof(&root);
        let scope = Scope {
            repository: snapshot.repo.id,
            path: "COMMIT_MESSAGE".into(),
            fingerprint: "synthetic-head".into(),
        };
        let Payload::CommitDraftSaved(first) = execute(
            &mut backend,
            temp.path(),
            Work::CommitDraftSave {
                scope: scope.clone(),
                text: "Review search 日本語\n\nKeep unstaged edits.".into(),
                expected: None,
            },
        )
        .unwrap() else {
            panic!()
        };
        let mut restarted = Backend::new(temp.path().join("state/repositories.json"));
        let Payload::CommitDraftLoaded(Some(loaded)) = execute(
            &mut restarted,
            temp.path(),
            Work::CommitDraftLoad(scope.clone()),
        )
        .unwrap() else {
            panic!()
        };
        assert_eq!(first.record.text, loaded.record.text);
        assert!(
            execute(
                &mut restarted,
                temp.path(),
                Work::CommitDraftSave {
                    scope: scope.clone(),
                    text: "stale writer".into(),
                    expected: None
                }
            )
            .is_err()
        );
        assert!(
            execute(
                &mut restarted,
                temp.path(),
                Work::CommitDraftSave {
                    scope: scope.clone(),
                    text: "x".repeat(32769),
                    expected: Some(first.revision.clone())
                }
            )
            .is_err()
        );
        let Payload::CommitDraftLoaded(Some(still_saved)) =
            execute(&mut restarted, temp.path(), Work::CommitDraftLoad(scope)).unwrap()
        else {
            panic!()
        };
        assert_eq!(still_saved.record.text, first.record.text);
        assert_eq!(proof(&root), before);
    }
    #[test]
    fn native_amend_reads_full_message_preserves_unstaged_work_and_refuses_published_head() {
        use ramo_core::git::Action;
        let (temp, root, mut backend) = fixture();
        let s = backend.register(&root).unwrap();
        let s = backend.trust(&s.repo.id).unwrap();
        let head = s.head.clone().unwrap();
        let Payload::AmendMessage { message, .. } = execute(
            &mut backend,
            temp.path(),
            Work::AmendMessage {
                repo: s.repo.id.clone(),
                head: head.clone(),
            },
        )
        .unwrap() else {
            panic!()
        };
        assert_eq!(message.trim(), "Initial search");
        fs::write(root.join("search.txt"), b"staged replacement\n").unwrap();
        git(&root, &["add", "search.txt"]);
        fs::write(root.join("search.txt"), b"keep unstaged\n").unwrap();
        let current = backend.snapshot(&s.repo.id).unwrap();
        let (done, error) = mutate(
            &mut backend,
            temp.path(),
            &current,
            Action::Amend {
                head: head.clone(),
                message: "Reviewed amend\n\nFull body".into(),
            },
        );
        assert!(error.is_none());
        assert_ne!(done.head.as_deref(), Some(head.as_str()));
        assert_eq!(
            fs::read(root.join("search.txt")).unwrap(),
            b"keep unstaged\n"
        );
        assert_eq!(
            git(&root, &["show", "HEAD:search.txt"]),
            b"staged replacement\n"
        );
        assert!(
            !git(
                &root,
                &["for-each-ref", "--format=%(refname)", "refs/ramo/amend/"]
            )
            .is_empty()
        );
        git(
            &root,
            &[
                "update-ref",
                "refs/remotes/origin/main",
                done.head.as_deref().unwrap(),
            ],
        );
        let before = proof(&root);
        let error = match execute(
            &mut backend,
            temp.path(),
            Work::AmendMessage {
                repo: s.repo.id,
                head: done.head.unwrap(),
            },
        ) {
            Err(error) => error,
            _ => panic!("published amend must fail"),
        };
        assert!(error.contains("remote"));
        assert_eq!(proof(&root), before);
    }
    #[test]
    fn native_cherry_pick_conflict_can_resolve_then_continue_with_untracked_work_preserved() {
        use ramo_core::git::{Action, SequenceKind, SequenceStep};
        let (temp, root, mut backend) = fixture();
        git(&root, &["switch", "-c", "side"]);
        fs::write(root.join("search.txt"), b"incoming\n").unwrap();
        git(&root, &["commit", "-am", "Incoming"]);
        let target = String::from_utf8(git(&root, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
        git(&root, &["switch", "main"]);
        fs::write(root.join("search.txt"), b"current\n").unwrap();
        git(&root, &["commit", "-am", "Current"]);
        let s = backend.register(&root).unwrap();
        let s = backend.trust(&s.repo.id).unwrap();
        let (conflicted, error) =
            mutate(&mut backend, temp.path(), &s, Action::CherryPick { target });
        assert!(error.is_some());
        assert_eq!(
            conflicted.operation.as_deref(),
            Some("Cherry-pick in progress")
        );
        fs::write(root.join("keep.txt"), b"untracked\n").unwrap();
        let current = backend.snapshot(&s.repo.id).unwrap();
        let (resolved, error) = mutate(
            &mut backend,
            temp.path(),
            &current,
            Action::ResolveFile {
                path: "search.txt".into(),
                content: Some("resolved\n".into()),
            },
        );
        assert!(error.is_none());
        let (done, error) = mutate(
            &mut backend,
            temp.path(),
            &resolved,
            Action::Sequence {
                operation: SequenceKind::CherryPick,
                step: SequenceStep::Continue,
            },
        );
        assert!(error.is_none());
        assert!(done.operation.is_none());
        assert_eq!(git(&root, &["show", "HEAD:search.txt"]), b"resolved\n");
        assert_eq!(fs::read(root.join("keep.txt")).unwrap(), b"untracked\n");
    }
}

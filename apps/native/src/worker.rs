//! One bounded worker owns the existing backend. No Git mutation endpoint is exposed.
use crate::drafts::{self, Loaded, Scope};
use eframe::egui;
use ramo_core::git::{Backend, Commit, ConflictFile, FileChange, Snapshot};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, SyncSender},
};

pub enum Work {
    Open(Option<PathBuf>),
    Refresh(String),
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
    Opened {
        snapshot: Box<Snapshot>,
        commits: Vec<Commit>,
    },
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
        Work::Open(path) => {
            let Some(path) = path.or_else(|| {
                rfd::FileDialog::new()
                    .set_title("Open Git repository")
                    .pick_folder()
            }) else {
                return Ok(Payload::Cancelled);
            };
            let snapshot = backend.register(&path)?;
            let commits = backend.history(&snapshot.repo.id, 0)?;
            Payload::Opened {
                snapshot: Box::new(snapshot),
                commits,
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
}

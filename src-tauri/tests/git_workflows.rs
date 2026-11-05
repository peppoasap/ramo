use ramo_core::{
    ai,
    git::{self, Action, Backend},
};
use std::{fs, path::Path};
struct Fixture {
    _dir: tempfile::TempDir,
    pub root: std::path::PathBuf,
    pub backend: Backend,
    pub id: String,
}
fn g(root: &Path, args: &[&str]) -> String {
    String::from_utf8(git::git(root, args).unwrap()).unwrap()
}
impl Fixture {
    fn conflicted_merge(&mut self) {
        self.seed();
        g(&self.root, &["switch", "-c", "side"]);
        self.write("README.md", "Side branch text.\n");
        g(&self.root, &["add", "README.md"]);
        g(&self.root, &["commit", "-m", "Side change"]);
        g(&self.root, &["switch", "main"]);
        self.write("README.md", "Main branch text.\n");
        g(&self.root, &["add", "README.md"]);
        g(&self.root, &["commit", "-m", "Main change"]);
        assert!(git::git(&self.root, &["merge", "side"]).is_err());
        self.backend.trust(&self.id).unwrap();
    }
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("meadow");
        fs::create_dir(&root).unwrap();
        g(&root, &["init", "-b", "main"]);
        g(&root, &["config", "user.name", "Demo Contributor"]);
        g(&root, &["config", "user.email", "demo@example.invalid"]);
        g(&root, &["config", "commit.gpgsign", "false"]);
        // Fixture bytes are explicit; do not inherit the host Git checkout conversion.
        g(&root, &["config", "core.autocrlf", "false"]);
        let mut backend = Backend::new(dir.path().join("preferences.json"));
        let snapshot = backend.register(&root).unwrap();
        Self {
            _dir: dir,
            root,
            backend,
            id: snapshot.repo.id,
        }
    }
    fn write(&self, path: &str, content: &str) {
        if let Some(parent) = self.root.join(path).parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(self.root.join(path), content).unwrap();
    }
    fn seed(&mut self) {
        self.write("README.md", "A synthetic repository.\n");
        g(&self.root, &["add", "README.md"]);
        g(&self.root, &["commit", "-m", "Initial documentation"]);
    }
    fn act(&mut self, action: Action) -> Result<ramo_core::git::Snapshot, String> {
        let snapshot = self.backend.snapshot(&self.id).unwrap();
        self.backend.mutate(&self.id, &snapshot.fingerprint, action)
    }
}

#[test]
fn explicit_branch_merge_fast_forwards_the_branch_not_an_ambiguous_tag() {
    let mut f = Fixture::new();
    f.seed();
    g(&f.root, &["switch", "-c", "side"]);
    f.write("feature.txt", "Reviewed feature.\n");
    g(&f.root, &["add", "feature.txt"]);
    g(&f.root, &["commit", "-m", "Feature"]);
    let target = g(&f.root, &["rev-parse", "HEAD"]).trim().to_owned();
    g(&f.root, &["switch", "main"]);
    g(&f.root, &["tag", "side"]);
    g(&f.root, &["config", "merge.ff", "false"]);
    g(
        &f.root,
        &["config", "branch.main.mergeOptions", "--squash --no-commit"],
    );
    f.backend.trust(&f.id).unwrap();
    let next = f
        .act(Action::Merge {
            name: "side".into(),
        })
        .unwrap();
    assert_eq!(next.head.as_deref(), Some(target.as_str()));
    assert!(next.files.is_empty() && next.operation.is_none());
}

#[test]
fn branch_merge_refuses_dirty_work_without_staging_or_losing_it() {
    let mut f = Fixture::new();
    f.seed();
    g(&f.root, &["branch", "side"]);
    f.backend.trust(&f.id).unwrap();
    f.write("README.md", "Unstaged work must be preserved.\n");
    let before = f.backend.snapshot(&f.id).unwrap();
    let error = f
        .act(Action::Merge {
            name: "side".into(),
        })
        .unwrap_err();
    assert!(error.contains("Commit or stash"));
    assert_eq!(
        before.fingerprint,
        f.backend.snapshot(&f.id).unwrap().fingerprint
    );
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "Unstaged work must be preserved.\n"
    );
    assert!(g(&f.root, &["diff", "--cached"]).is_empty());
}

#[test]
fn divergent_branch_merge_creates_a_local_two_parent_commit() {
    let mut f = Fixture::new();
    f.seed();
    g(&f.root, &["switch", "-c", "side"]);
    f.write("side.txt", "Side content.\n");
    g(&f.root, &["add", "side.txt"]);
    g(&f.root, &["commit", "-m", "Side file"]);
    g(&f.root, &["switch", "main"]);
    f.write("main.txt", "Main content.\n");
    g(&f.root, &["add", "main.txt"]);
    g(&f.root, &["commit", "-m", "Main file"]);
    f.backend.trust(&f.id).unwrap();
    let before = f.backend.snapshot(&f.id).unwrap();
    let merged = f
        .act(Action::Merge {
            name: "side".into(),
        })
        .unwrap();
    assert_ne!(merged.head, before.head);
    assert!(merged.operation.is_none());
    assert_eq!(
        g(&f.root, &["rev-list", "--parents", "-n", "1", "HEAD"])
            .split_whitespace()
            .count(),
        3
    );
    assert_eq!(
        fs::read_to_string(f.root.join("main.txt")).unwrap(),
        "Main content.\n"
    );
    assert_eq!(
        fs::read_to_string(f.root.join("side.txt")).unwrap(),
        "Side content.\n"
    );
}

#[test]
fn app_started_merge_conflict_remains_available_for_explicit_resolution() {
    let mut f = Fixture::new();
    f.conflicted_merge();
    g(&f.root, &["merge", "--abort"]);
    let head = g(&f.root, &["rev-parse", "HEAD"]);
    assert!(f
        .act(Action::Merge {
            name: "side".into()
        })
        .is_err());
    let pending = f.backend.snapshot(&f.id).unwrap();
    assert_eq!(pending.operation.as_deref(), Some("Merge in progress"));
    assert!(pending.files.iter().any(git::is_conflicted));
    assert_eq!(g(&f.root, &["rev-parse", "HEAD"]), head);
    assert!(fs::read_to_string(f.root.join("README.md"))
        .unwrap()
        .contains("<<<<<<<"));
}

#[test]
fn conflicts_require_explicit_resolution_before_completing_a_merge() {
    let mut f = Fixture::new();
    f.conflicted_merge();
    let review = f.backend.snapshot(&f.id).unwrap();
    assert_eq!(review.operation.as_deref(), Some("Merge in progress"));
    assert!(review.files.iter().any(git::is_conflicted));
    let unmerged = g(&f.root, &["ls-files", "-u"]);
    for action in [
        Action::Stage {
            paths: vec!["README.md".into()],
        },
        Action::Unstage {
            paths: vec!["README.md".into()],
        },
        Action::Switch {
            name: "side".into(),
        },
        Action::CreateBranch {
            name: "unsafe".into(),
        },
        Action::Stash {
            message: "unsafe".into(),
        },
        Action::Push,
        Action::Commit {
            message: "Not resolved".into(),
        },
    ] {
        assert!(f
            .backend
            .mutate(&f.id, &review.fingerprint, action)
            .is_err());
        assert_eq!(g(&f.root, &["ls-files", "-u"]), unmerged);
    }
    f.write("unrelated.txt", "Preserve this unstaged work.\n");
    f.write("README.md", "Reviewed combined resolution.\n");
    let resolved = f
        .act(Action::MarkResolved {
            path: "README.md".into(),
        })
        .unwrap();
    assert!(!resolved.files.iter().any(git::is_conflicted));
    let done = f
        .act(Action::Commit {
            message: "Complete reviewed merge".into(),
        })
        .unwrap();
    assert!(done.operation.is_none());
    assert_eq!(
        g(&f.root, &["rev-list", "--parents", "-n", "1", "HEAD"])
            .split_whitespace()
            .count(),
        3
    );
    assert_eq!(
        fs::read_to_string(f.root.join("unrelated.txt")).unwrap(),
        "Preserve this unstaged work.\n"
    );
}

#[test]
fn merge_resolved_to_ours_can_finish_with_no_staged_tree_difference() {
    let mut f = Fixture::new();
    f.conflicted_merge();
    f.write("README.md", "Main branch text.\n");
    f.act(Action::MarkResolved {
        path: "README.md".into(),
    })
    .unwrap();
    assert!(g(&f.root, &["diff", "--cached"]).is_empty());
    f.act(Action::Commit {
        message: "Keep reviewed main content".into(),
    })
    .unwrap();
    let head = g(&f.root, &["rev-parse", "HEAD"]).trim().to_owned();
    assert!(f.backend.commit_files(&f.id, &head).unwrap().is_empty());
    assert!(f
        .backend
        .file_diff(&f.id, "README.md", Some(&head), false)
        .unwrap()
        .is_empty());
    assert_eq!(
        g(&f.root, &["rev-list", "--parents", "-n", "1", "HEAD"])
            .split_whitespace()
            .count(),
        3
    );
}

#[test]
fn starting_a_no_tree_change_merge_invalidates_the_old_review() {
    let mut f = Fixture::new();
    f.seed();
    g(&f.root, &["switch", "-c", "empty-side"]);
    g(
        &f.root,
        &["commit", "--allow-empty", "-m", "Synthetic empty side"],
    );
    g(&f.root, &["switch", "main"]);
    f.backend.trust(&f.id).unwrap();
    let before = f.backend.snapshot(&f.id).unwrap();
    g(&f.root, &["merge", "--no-ff", "--no-commit", "empty-side"]);
    let after = f.backend.snapshot(&f.id).unwrap();
    assert_eq!(before.head, after.head);
    assert!(before.files.is_empty() && after.files.is_empty());
    assert_ne!(before.fingerprint, after.fingerprint);
    assert!(f
        .backend
        .mutate(
            &f.id,
            &before.fingerprint,
            Action::Commit {
                message: "Stale review".into()
            }
        )
        .unwrap_err()
        .contains("changed since"));
    f.act(Action::Commit {
        message: "Complete empty-tree merge".into(),
    })
    .unwrap();
}

#[test]
fn rebase_resolution_cannot_be_mistaken_for_an_ordinary_commit() {
    let mut f = Fixture::new();
    f.conflicted_merge();
    g(&f.root, &["merge", "--abort"]);
    g(&f.root, &["switch", "side"]);
    assert!(git::git(&f.root, &["rebase", "main"]).is_err());
    assert_eq!(
        f.backend.snapshot(&f.id).unwrap().operation.as_deref(),
        Some("Rebase in progress")
    );
    f.write("README.md", "Rebase resolution.\n");
    f.act(Action::MarkResolved {
        path: "README.md".into(),
    })
    .unwrap();
    assert!(f
        .act(Action::Commit {
            message: "Do not bypass rebase continuation".into()
        })
        .unwrap_err()
        .contains("continue command"));
    assert!(!g(&f.root, &["diff", "--cached"]).is_empty());
}

#[test]
fn large_binary_patches_do_not_block_review_and_stale_content_still_invalidates_it() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    // Deterministic incompressible-ish data: Git's encoded binary patch exceeds
    // the ordinary process output cap. No fixture asset is checked into source.
    let mut bytes = vec![0u8; 20 * 1024 * 1024];
    let mut random = 0x12345678u32;
    for byte in bytes.iter_mut().skip(1) {
        random ^= random << 13;
        random ^= random >> 17;
        random ^= random << 5;
        *byte = random as u8;
    }
    fs::write(f.root.join("asset.bin"), &bytes).unwrap();
    g(&f.root, &["add", "asset.bin"]);
    // Prove this scenario would exceed the old buffered fingerprint path.
    assert!(git::git(&f.root, &["diff", "--cached", "--binary"])
        .unwrap_err()
        .contains("limit"));
    let reviewed = f.backend.snapshot(&f.id).unwrap();
    assert!(f
        .backend
        .file_diff(&f.id, "asset.bin", None, true)
        .unwrap()
        .contains("Binary files"));
    *bytes.last_mut().unwrap() ^= 1;
    fs::write(f.root.join("asset.bin"), &bytes).unwrap();
    g(&f.root, &["add", "asset.bin"]);
    let current = f.backend.snapshot(&f.id).unwrap();
    assert_ne!(reviewed.fingerprint, current.fingerprint);
    assert!(f
        .backend
        .mutate(
            &f.id,
            &reviewed.fingerprint,
            Action::Commit {
                message: "Stale binary".into()
            }
        )
        .unwrap_err()
        .contains("changed since"));
    // Verify the same large file works on the unstaged side as well.
    g(&f.root, &["commit", "-m", "Add synthetic binary"]);
    bytes[1] ^= 1;
    fs::write(f.root.join("asset.bin"), &bytes).unwrap();
    let unstaged = f.backend.snapshot(&f.id).unwrap();
    assert!(unstaged
        .files
        .iter()
        .any(|file| file.path == "asset.bin" && file.worktree == "M"));
}

#[test]
fn switching_equal_head_branches_invalidates_review_and_preserves_staged_work() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    g(&f.root, &["branch", "another"]);
    f.write(
        "README.md",
        "Staged work must not be committed on another branch.\n",
    );
    g(&f.root, &["add", "README.md"]);
    let reviewed = f.backend.snapshot(&f.id).unwrap();
    g(&f.root, &["switch", "another"]);
    let current = f.backend.snapshot(&f.id).unwrap();
    assert_eq!(reviewed.head, current.head);
    assert_ne!(reviewed.fingerprint, current.fingerprint);
    let error = f
        .backend
        .mutate(
            &f.id,
            &reviewed.fingerprint,
            Action::Commit {
                message: "Must not commit".into(),
            },
        )
        .unwrap_err();
    assert!(error.contains("changed since"));
    assert_eq!(
        g(&f.root, &["log", "-1", "--format=%s"]).trim(),
        "Initial documentation"
    );
    assert!(!g(&f.root, &["diff", "--cached"]).is_empty());
}

#[test]
fn changing_upstream_at_equal_object_id_invalidates_review() {
    let mut f = Fixture::new();
    f.seed();
    g(
        &f.root,
        &[
            "remote",
            "add",
            "origin",
            "https://example.invalid/synthetic.git",
        ],
    );
    g(&f.root, &["update-ref", "refs/remotes/origin/one", "HEAD"]);
    g(&f.root, &["update-ref", "refs/remotes/origin/two", "HEAD"]);
    g(&f.root, &["branch", "--set-upstream-to=origin/one"]);
    let reviewed = f.backend.snapshot(&f.id).unwrap();
    g(&f.root, &["branch", "--set-upstream-to=origin/two"]);
    let current = f.backend.snapshot(&f.id).unwrap();
    assert_eq!(reviewed.head, current.head);
    assert_ne!(reviewed.upstream, current.upstream);
    assert_ne!(reviewed.fingerprint, current.fingerprint);
}

#[test]
fn unstaging_a_rename_restores_both_index_paths_without_touching_files() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    g(&f.root, &["mv", "README.md", "RENAMED.md"]);
    let snapshot = f.backend.snapshot(&f.id).unwrap();
    assert_eq!(snapshot.files[0].original.as_deref(), Some("README.md"));
    let result = f
        .act(Action::Unstage {
            paths: vec!["RENAMED.md".into()],
        })
        .unwrap();
    assert!(result
        .files
        .iter()
        .all(|file| file.index == " " || file.index == "?"));
    assert!(f.root.join("RENAMED.md").is_file());
    assert!(!f.root.join("README.md").exists());
}
#[test]
fn push_never_sends_other_branches_even_with_matching_push_configuration() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    let remote = f.root.parent().unwrap().join("remote.git");
    fs::create_dir(&remote).unwrap();
    g(&remote, &["init", "--bare"]);
    g(
        &f.root,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    f.act(Action::Publish {
        remote: "origin".into(),
    })
    .unwrap();
    g(&f.root, &["branch", "other"]);
    g(&f.root, &["push", "origin", "other"]);
    g(&f.root, &["switch", "other"]);
    f.write("other.txt", "other branch only\n");
    g(&f.root, &["add", "other.txt"]);
    g(&f.root, &["commit", "-m", "Advance unrelated branch"]);
    g(&f.root, &["switch", "main"]);
    g(&f.root, &["config", "push.default", "matching"]);
    let old = g(&remote, &["rev-parse", "refs/heads/other"]);
    f.write("main.txt", "active branch only\n");
    f.act(Action::Stage {
        paths: vec!["main.txt".into()],
    })
    .unwrap();
    f.act(Action::Commit {
        message: "Advance active branch".into(),
    })
    .unwrap();
    f.act(Action::Push).unwrap();
    assert_eq!(g(&remote, &["rev-parse", "refs/heads/other"]), old);
    assert_eq!(
        g(&remote, &["rev-parse", "refs/heads/main"]),
        g(&f.root, &["rev-parse", "HEAD"])
    );
}

#[test]
fn empty_repository_and_unborn_staging() {
    let mut f = Fixture::new();
    assert!(f.backend.history(&f.id, 0).unwrap().is_empty());
    f.write("hello.txt", "hello\n");
    assert!(f
        .act(Action::Stage {
            paths: vec!["hello.txt".into()]
        })
        .unwrap_err()
        .contains("Trust"));
    f.backend.trust(&f.id).unwrap();
    f.act(Action::Stage {
        paths: vec!["hello.txt".into()],
    })
    .unwrap();
    let unstaged = f
        .act(Action::Unstage {
            paths: vec!["hello.txt".into()],
        })
        .unwrap();
    assert_eq!(unstaged.files[0].index, "?");
    assert!(f.root.join("hello.txt").exists());
    f.act(Action::Stage {
        paths: vec!["hello.txt".into()],
    })
    .unwrap();
    let done = f
        .act(Action::Commit {
            message: "Create greeting".into(),
        })
        .unwrap();
    assert!(done.files.is_empty());
    assert_eq!(
        f.backend.history(&f.id, 0).unwrap()[0].subject,
        "Create greeting"
    );
}
#[test]
fn commits_only_staged_content_and_preserves_other_work() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("README.md", "A synthetic repository.\nStaged line.\n");
    f.act(Action::Stage {
        paths: vec!["README.md".into()],
    })
    .unwrap();
    f.write(
        "README.md",
        "A synthetic repository.\nStaged line.\nUnstaged line.\n",
    );
    f.write("other.txt", "untouched\n");
    f.act(Action::Commit {
        message: "Extend documentation".into(),
    })
    .unwrap();
    let committed = g(&f.root, &["show", "HEAD:README.md"]);
    assert!(committed.contains("Staged line."));
    assert!(!committed.contains("Unstaged line."));
    assert!(fs::read_to_string(f.root.join("README.md"))
        .unwrap()
        .contains("Unstaged line."));
    assert!(f.root.join("other.txt").exists());
    assert_eq!(f.backend.snapshot(&f.id).unwrap().files.len(), 2);
}
#[test]
fn rejects_stale_review_and_path_traversal() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("README.md", "first review\n");
    let previous = f.backend.snapshot(&f.id).unwrap();
    f.write("README.md", "changed after review\n");
    assert!(f
        .backend
        .mutate(
            &f.id,
            &previous.fingerprint,
            Action::Stage {
                paths: vec!["README.md".into()]
            }
        )
        .unwrap_err()
        .contains("changed"));
    assert!(f
        .act(Action::Stage {
            paths: vec!["../outside".into()]
        })
        .is_err());
    assert!(f
        .act(Action::CreateBranch {
            name: "--orphan".into()
        })
        .is_err());
}
#[test]
fn handles_unicode_newlines_and_literal_pathspecs() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    let paths = if cfg!(windows) {
        vec!["café.txt", "[literal].txt"]
    } else {
        vec!["café.txt", "line\nbreak.txt", ":(glob)*.txt"]
    };
    for path in &paths {
        f.write(path, "synthetic content\n");
        let snapshot = f.backend.snapshot(&f.id).unwrap();
        assert!(snapshot.files.iter().any(|v| &v.path == path));
        f.act(Action::Stage {
            paths: vec![(*path).into()],
        })
        .unwrap();
    }
    f.act(Action::Commit {
        message: "Add unusual filenames".into(),
    })
    .unwrap();
    let snapshot = f.backend.snapshot(&f.id).unwrap();
    let files = f
        .backend
        .commit_files(&f.id, snapshot.head.as_deref().unwrap())
        .unwrap();
    assert_eq!(files.len(), paths.len());
}
#[test]
fn merge_history_preserves_parent_oids_and_first_parent_diff() {
    let mut f = Fixture::new();
    f.seed();
    g(&f.root, &["switch", "-c", "feature"]);
    f.write("feature.txt", "feature\n");
    g(&f.root, &["add", "feature.txt"]);
    g(&f.root, &["commit", "-m", "Add feature"]);
    g(&f.root, &["switch", "main"]);
    f.write("main.txt", "main\n");
    g(&f.root, &["add", "main.txt"]);
    g(&f.root, &["commit", "-m", "Advance main"]);
    g(
        &f.root,
        &["merge", "--no-ff", "feature", "-m", "Merge feature"],
    );
    let commits = f.backend.history(&f.id, 0).unwrap();
    assert_eq!(commits[0].parents.len(), 2);
    assert_eq!(commits.len(), 4);
    let files = f.backend.commit_files(&f.id, &commits[0].oid).unwrap();
    assert!(files.iter().any(|v| v.path == "feature.txt"));
    assert!(f
        .backend
        .file_diff(&f.id, "feature.txt", Some(&commits[0].oid), false)
        .unwrap()
        .contains("+feature"));
}
#[test]
fn linked_worktree_uses_its_own_index_and_checks_occupancy() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    let linked = f.root.parent().unwrap().join("linked");
    let linked_text = linked.to_str().unwrap();
    g(&f.root, &["worktree", "add", "-b", "feature", linked_text]);
    let other = f.backend.open_worktree(&f.id, linked_text).unwrap();
    assert_eq!(other.branch, "feature");
    assert_ne!(other.repo.id, f.id);
    f.backend.trust(&other.repo.id).unwrap();
    fs::write(linked.join("linked.txt"), "separate worktree\n").unwrap();
    let before = f.backend.snapshot(&other.repo.id).unwrap();
    f.backend
        .mutate(
            &other.repo.id,
            &before.fingerprint,
            Action::Stage {
                paths: vec!["linked.txt".into()],
            },
        )
        .unwrap();
    assert!(f.backend.snapshot(&f.id).unwrap().files.is_empty());
    assert!(f
        .act(Action::Switch {
            name: "feature".into()
        })
        .is_err());
}
#[test]
fn branch_stash_and_detached_head_are_real_git_states() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    let branch = f
        .act(Action::CreateBranch {
            name: "feat/search".into(),
        })
        .unwrap();
    assert_eq!(branch.branch, "feat/search");
    f.act(Action::Switch {
        name: "main".into(),
    })
    .unwrap();
    f.act(Action::DeleteBranch {
        name: "feat/search".into(),
    })
    .unwrap();
    f.write("README.md", "work in progress\n");
    f.act(Action::Stash {
        message: "Synthetic stash".into(),
    })
    .unwrap();
    let clean = f.backend.snapshot(&f.id).unwrap();
    assert!(clean.files.is_empty());
    assert_eq!(clean.stashes.len(), 1);
    f.act(Action::ApplyStash {
        index: 0,
        oid: clean.stashes[0].oid.clone(),
    })
    .unwrap();
    assert_eq!(f.backend.snapshot(&f.id).unwrap().stashes.len(), 1);
    g(&f.root, &["checkout", "--detach"]);
    assert_eq!(f.backend.snapshot(&f.id).unwrap().branch, "Detached HEAD");
}
#[test]
fn ai_uses_only_staged_eligible_diff_and_rejects_stale_snapshot() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("README.md", "reviewed staged change\n");
    f.act(Action::Stage {
        paths: vec!["README.md".into()],
    })
    .unwrap();
    f.write("README.md", "reviewed staged change\nUNSTAGED_SENTINEL\n");
    f.write(".env", "PRIVATE_SENTINEL=secret\n");
    f.act(Action::Stage {
        paths: vec![".env".into()],
    })
    .unwrap();
    let input = ai::prepare(&f.backend, &f.id, "commit", None, None, true).unwrap();
    assert!(input.text.contains("reviewed staged change"));
    assert!(!input.text.contains("UNSTAGED_SENTINEL"));
    assert!(!input.text.contains("PRIVATE_SENTINEL"));
    assert_eq!(input.excluded, vec![".env"]);
    f.write("README.md", "changed while generating\n");
    assert!(ai::ensure_current(&input, &f.backend.snapshot(&f.id).unwrap()).is_err());
}
#[test]
fn read_diff_never_invokes_external_diff_driver() {
    let mut f = Fixture::new();
    f.seed();
    f.write("README.md", "modified\n");
    g(
        &f.root,
        &["config", "diff.external", "program-that-must-never-run"],
    );
    assert!(f
        .backend
        .file_diff(&f.id, "README.md", None, false)
        .unwrap()
        .contains("+modified"));
}
#[test]
fn fetch_publish_pull_and_push_use_explicit_upstream() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    let remote = f.root.parent().unwrap().join("remote.git");
    fs::create_dir(&remote).unwrap();
    g(&remote, &["init", "--bare"]);
    g(
        &f.root,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    assert!(f.act(Action::Push).is_err());
    let published = f
        .act(Action::Publish {
            remote: "origin".into(),
        })
        .unwrap();
    assert_eq!(published.upstream.as_deref(), Some("origin/main"));
    assert_eq!(published.ahead, Some(0));
    f.write("next.txt", "next\n");
    f.act(Action::Stage {
        paths: vec!["next.txt".into()],
    })
    .unwrap();
    f.act(Action::Commit {
        message: "Add next file".into(),
    })
    .unwrap();
    assert_eq!(f.backend.snapshot(&f.id).unwrap().ahead, Some(1));
    f.act(Action::Push).unwrap();
    f.act(Action::Fetch).unwrap();
    f.act(Action::Pull).unwrap();
    assert_eq!(f.backend.snapshot(&f.id).unwrap().ahead, Some(0));
}

#[test]
fn commit_context_creates_refs_without_moving_head_and_refuses_replacement() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    let target = g(&f.root, &["rev-parse", "HEAD"]).trim().to_owned();
    let next = f
        .act(Action::CreateBranchAt {
            name: "review/here".into(),
            target: target.clone(),
        })
        .unwrap();
    assert_eq!(next.branch, "main");
    assert_eq!(
        g(&f.root, &["rev-parse", "refs/heads/review/here"]).trim(),
        target
    );
    f.act(Action::CreateTag {
        name: "v-test".into(),
        target: target.clone(),
    })
    .unwrap();
    assert!(f
        .act(Action::CreateTag {
            name: "v-test".into(),
            target: target.clone()
        })
        .is_err());
    assert!(f
        .act(Action::CreateBranchAt {
            name: "main".into(),
            target: target.clone()
        })
        .is_err());
    assert!(f
        .act(Action::CheckoutCommit {
            target: "--help".into()
        })
        .is_err());
    let next = f
        .act(Action::CheckoutCommit {
            target: target.clone(),
        })
        .unwrap();
    assert_eq!(next.branch, "Detached HEAD");
    assert_eq!(next.head.as_deref(), Some(target.as_str()));
}

#[test]
fn rename_and_remote_checkout_use_explicit_local_refs() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.act(Action::RenameBranch {
        name: "main".into(),
        new_name: "trunk".into(),
    })
    .unwrap();
    assert_eq!(f.backend.snapshot(&f.id).unwrap().branch, "trunk");
    g(
        &f.root,
        &[
            "remote",
            "add",
            "origin",
            "https://example.invalid/synthetic.git",
        ],
    );
    g(
        &f.root,
        &["update-ref", "refs/remotes/origin/topic", "HEAD"],
    );
    let next = f
        .act(Action::TrackBranch {
            name: "topic".into(),
            remote_branch: "origin/topic".into(),
        })
        .unwrap();
    assert_eq!(next.branch, "topic");
    assert_eq!(next.upstream.as_deref(), Some("origin/topic"));
    assert!(f
        .act(Action::RenameBranch {
            name: "topic".into(),
            new_name: "trunk".into()
        })
        .is_err());
    assert!(f
        .act(Action::TrackBranch {
            name: "missing".into(),
            remote_branch: "origin/absent".into()
        })
        .is_err());
}

#[test]
fn cherry_pick_and_revert_make_local_commits_without_rewriting_history() {
    let mut f = Fixture::new();
    f.seed();
    g(&f.root, &["switch", "-c", "side"]);
    f.write("feature.txt", "Synthetic feature\n");
    g(&f.root, &["add", "feature.txt"]);
    g(&f.root, &["commit", "-m", "Feature"]);
    let target = g(&f.root, &["rev-parse", "HEAD"]).trim().to_owned();
    g(&f.root, &["switch", "main"]);
    f.backend.trust(&f.id).unwrap();
    f.write("untracked.txt", "Keep this\n");
    assert!(f
        .act(Action::CherryPick {
            target: target.clone()
        })
        .is_err());
    assert_eq!(
        fs::read_to_string(f.root.join("untracked.txt")).unwrap(),
        "Keep this\n"
    );
    fs::remove_file(f.root.join("untracked.txt")).unwrap();
    let picked = f.act(Action::CherryPick { target }).unwrap();
    assert!(f.root.join("feature.txt").exists());
    let picked_head = picked.head.unwrap();
    let reverted = f
        .act(Action::Revert {
            target: picked_head.clone(),
        })
        .unwrap();
    assert!(!f.root.join("feature.txt").exists());
    assert_eq!(g(&f.root, &["rev-parse", "HEAD^"]).trim(), picked_head);
    assert!(reverted.files.is_empty());
}

#[test]
fn guided_resolution_reads_three_versions_and_saves_stages_then_completes() {
    let mut f = Fixture::new();
    f.conflicted_merge();
    let snap = f.backend.snapshot(&f.id).unwrap();
    let versions = f
        .backend
        .conflict_file(&f.id, &snap.fingerprint, "README.md")
        .unwrap();
    assert_eq!(versions.ours.as_deref(), Some("Main branch text.\n"));
    assert_eq!(versions.theirs.as_deref(), Some("Side branch text.\n"));
    assert!(versions.working.unwrap().contains("<<<<<<<"));
    assert!(f
        .act(Action::ResolveFile {
            path: "README.md".into(),
            content: Some("<<<<<<< unresolved\n".into())
        })
        .is_err());
    f.act(Action::ResolveFile {
        path: "README.md".into(),
        content: Some("Reviewed combined result\n".into()),
    })
    .unwrap();
    assert_eq!(
        g(&f.root, &["show", ":README.md"]),
        "Reviewed combined result\n"
    );
    assert!(g(&f.root, &["ls-files", "-u"]).is_empty());
    let next = f
        .act(Action::Commit {
            message: "Reviewed merge".into(),
        })
        .unwrap();
    assert!(next.operation.is_none() && next.files.is_empty());
    assert_eq!(
        g(&f.root, &["show", "-s", "--format=%P"])
            .split_whitespace()
            .count(),
        2
    );
}

#[test]
fn guided_resolution_rejects_stale_reviews_and_can_accept_deletion() {
    let mut f = Fixture::new();
    f.conflicted_merge();
    let reviewed = f.backend.snapshot(&f.id).unwrap();
    f.write("README.md", "External edit to preserve\n");
    assert!(f
        .backend
        .mutate(
            &f.id,
            &reviewed.fingerprint,
            Action::ResolveFile {
                path: "README.md".into(),
                content: Some("Old result".into())
            }
        )
        .is_err());
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "External edit to preserve\n"
    );
    f.act(Action::ResolveFile {
        path: "README.md".into(),
        content: None,
    })
    .unwrap();
    assert!(!f.root.join("README.md").exists());
    assert!(g(&f.root, &["ls-files", "-u"]).is_empty());
    assert!(f
        .act(Action::ResolveFile {
            path: "../outside".into(),
            content: Some("No".into())
        })
        .is_err());
}

#[test]
fn abort_merge_restores_head_and_files_after_resolution() {
    let mut f = Fixture::new();
    f.conflicted_merge();
    let before = g(&f.root, &["rev-parse", "HEAD"]);
    f.act(Action::ResolveFile {
        path: "README.md".into(),
        content: Some("Resolution to discard\n".into()),
    })
    .unwrap();
    let next = f.act(Action::AbortMerge).unwrap();
    assert_eq!(g(&f.root, &["rev-parse", "HEAD"]), before);
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "Main branch text.\n"
    );
    assert!(next.operation.is_none() && next.files.is_empty());
    assert!(f.act(Action::AbortMerge).is_err());
}

#[cfg(unix)]
#[test]
fn guided_resolution_rejects_symlink_replacement_and_binary_files() {
    let mut f = Fixture::new();
    f.conflicted_merge();
    let outside = f._dir.path().join("outside.txt");
    fs::write(&outside, "Keep outside").unwrap();
    fs::remove_file(f.root.join("README.md")).unwrap();
    std::os::unix::fs::symlink(&outside, f.root.join("README.md")).unwrap();
    assert!(f
        .act(Action::ResolveFile {
            path: "README.md".into(),
            content: Some("No".into())
        })
        .is_err());
    assert_eq!(fs::read_to_string(outside).unwrap(), "Keep outside");
    fs::remove_file(f.root.join("README.md")).unwrap();
    fs::write(f.root.join("README.md"), [0u8, 1, 2]).unwrap();
    assert!(f
        .act(Action::ResolveFile {
            path: "README.md".into(),
            content: Some("No".into())
        })
        .is_err());
}

#[test]
fn merge_ai_preview_contains_only_selected_versions_and_rejects_stale_context() {
    let mut f = Fixture::new();
    f.conflicted_merge();
    f.write(
        "unrelated-private.txt",
        "Do not include this unrelated data",
    );
    let input = ai::prepare(&f.backend, &f.id, "resolve", Some("README.md"), None, false).unwrap();
    let data: serde_json::Value = serde_json::from_str(&input.text).unwrap();
    assert_eq!(data["ours"], "Main branch text.\n");
    assert_eq!(data["theirs"], "Side branch text.\n");
    assert_eq!(input.paths, vec!["README.md"]);
    assert!(!input.text.contains("Do not include this unrelated data"));
    f.write("README.md", "External edit\n");
    assert!(ai::ensure_current(&input, &f.backend.snapshot(&f.id).unwrap()).is_err());
    assert!(ai::prepare(&f.backend, &f.id, "resolve", Some(".env"), None, false).is_err());
    assert!(ai::prepare(
        &f.backend,
        &f.id,
        "resolve",
        Some("unrelated-private.txt"),
        None,
        false
    )
    .is_err());
}

fn interrupted_pick(f: &mut Fixture) -> String {
    f.seed();
    g(&f.root, &["switch", "-c", "side"]);
    f.write("README.md", "Side change\n");
    g(&f.root, &["commit", "-am", "Pick this change"]);
    let target = g(&f.root, &["rev-parse", "HEAD"]).trim().to_owned();
    g(&f.root, &["switch", "main"]);
    f.write("README.md", "Main change\n");
    g(&f.root, &["commit", "-am", "Keep main work"]);
    f.backend.trust(&f.id).unwrap();
    assert!(f
        .act(Action::CherryPick {
            target: target.clone()
        })
        .is_err());
    target
}

#[test]
fn cherry_pick_can_resolve_and_continue_without_opening_an_editor() {
    use git::{SequenceKind, SequenceStep};
    let mut f = Fixture::new();
    interrupted_pick(&mut f);
    g(&f.root, &["config", "core.editor", "false"]);
    assert!(f
        .act(Action::Sequence {
            operation: SequenceKind::CherryPick,
            step: SequenceStep::Continue
        })
        .is_err());
    f.act(Action::ResolveFile {
        path: "README.md".into(),
        content: Some("Combined review\n".into()),
    })
    .unwrap();
    let next = f
        .act(Action::Sequence {
            operation: SequenceKind::CherryPick,
            step: SequenceStep::Continue,
        })
        .unwrap();
    assert!(next.operation.is_none() && next.files.is_empty());
    assert_eq!(
        g(&f.root, &["show", "-s", "--format=%s"]).trim(),
        "Pick this change"
    );
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "Combined review\n"
    );
}

#[test]
fn cherry_pick_abort_and_skip_preserve_pre_sequence_work() {
    use git::{SequenceKind, SequenceStep};
    for step in [SequenceStep::Abort, SequenceStep::Skip] {
        let mut f = Fixture::new();
        interrupted_pick(&mut f);
        let head = g(&f.root, &["rev-parse", "HEAD"]);
        f.write("extra-untracked.txt", "Unrelated work\n");
        let next = f
            .act(Action::Sequence {
                operation: SequenceKind::CherryPick,
                step,
            })
            .unwrap();
        assert!(next.operation.is_none());
        assert_eq!(g(&f.root, &["rev-parse", "HEAD"]), head);
        assert_eq!(
            fs::read_to_string(f.root.join("README.md")).unwrap(),
            "Main change\n"
        );
        assert_eq!(
            fs::read_to_string(f.root.join("extra-untracked.txt")).unwrap(),
            "Unrelated work\n"
        );
    }
}

#[test]
fn revert_conflict_can_continue_and_abort_with_typed_operation_guards() {
    use git::{SequenceKind, SequenceStep};
    for abort in [true, false] {
        let mut f = Fixture::new();
        f.seed();
        f.write("README.md", "Change to revert\n");
        g(&f.root, &["commit", "-am", "Original change"]);
        let target = g(&f.root, &["rev-parse", "HEAD"]).trim().to_owned();
        f.write("README.md", "Later work\n");
        g(&f.root, &["commit", "-am", "Later change"]);
        let before = g(&f.root, &["rev-parse", "HEAD"]);
        f.backend.trust(&f.id).unwrap();
        assert!(f.act(Action::Revert { target }).is_err());
        assert!(f
            .act(Action::Sequence {
                operation: SequenceKind::CherryPick,
                step: SequenceStep::Abort
            })
            .is_err());
        f.act(Action::ResolveFile {
            path: "README.md".into(),
            content: Some("Reviewed revert\n".into()),
        })
        .unwrap();
        g(&f.root, &["config", "core.editor", "false"]);
        let next = f
            .act(Action::Sequence {
                operation: SequenceKind::Revert,
                step: if abort {
                    SequenceStep::Abort
                } else {
                    SequenceStep::Continue
                },
            })
            .unwrap();
        assert!(next.operation.is_none() && next.files.is_empty());
        if abort {
            assert_eq!(g(&f.root, &["rev-parse", "HEAD"]), before);
        } else {
            assert_eq!(g(&f.root, &["rev-parse", "HEAD^"]), before);
        }
    }
}

#[test]
fn sequence_continue_refuses_unstaged_edits_and_changed_todo() {
    use git::{SequenceKind, SequenceStep};
    let mut f = Fixture::new();
    interrupted_pick(&mut f);
    f.act(Action::ResolveFile {
        path: "README.md".into(),
        content: Some("Staged resolution\n".into()),
    })
    .unwrap();
    f.write("README.md", "New unstaged work\n");
    assert!(f
        .act(Action::Sequence {
            operation: SequenceKind::CherryPick,
            step: SequenceStep::Continue
        })
        .is_err());
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "New unstaged work\n"
    );
    let reviewed = f.backend.snapshot(&f.id).unwrap();
    fs::create_dir_all(f.root.join(".git/sequencer")).unwrap();
    fs::write(
        f.root.join(".git/sequencer/todo"),
        format!(
            "pick {} changed future step\n",
            reviewed.head.as_ref().unwrap()
        ),
    )
    .unwrap();
    assert!(f
        .backend
        .mutate(
            &f.id,
            &reviewed.fingerprint,
            Action::Sequence {
                operation: SequenceKind::CherryPick,
                step: SequenceStep::Abort
            }
        )
        .unwrap_err()
        .contains("Repository changed"));
}

#[test]
fn multi_commit_cherry_pick_can_pause_again_then_abort_the_whole_sequence() {
    use git::{SequenceKind, SequenceStep};
    let mut f = Fixture::new();
    f.seed();
    f.write("second.txt", "Base second\n");
    g(&f.root, &["add", "second.txt"]);
    g(&f.root, &["commit", "-m", "Second base"]);
    g(&f.root, &["switch", "-c", "side"]);
    f.write("README.md", "Side first\n");
    g(&f.root, &["commit", "-am", "First picked commit"]);
    let first = g(&f.root, &["rev-parse", "HEAD"]).trim().to_owned();
    f.write("second.txt", "Side second\n");
    g(&f.root, &["commit", "-am", "Second picked commit"]);
    let second = g(&f.root, &["rev-parse", "HEAD"]).trim().to_owned();
    g(&f.root, &["switch", "main"]);
    f.write("README.md", "Main first\n");
    f.write("second.txt", "Main second\n");
    g(&f.root, &["commit", "-am", "Keep main work"]);
    let original = g(&f.root, &["rev-parse", "HEAD"]);
    assert!(git::git(&f.root, &["cherry-pick", &first, &second]).is_err());
    f.backend.trust(&f.id).unwrap();
    f.act(Action::ResolveFile {
        path: "README.md".into(),
        content: Some("Combined first\n".into()),
    })
    .unwrap();
    let reviewed = f.backend.snapshot(&f.id).unwrap();
    let todo = f.root.join(".git/sequencer/todo");
    let original_todo = fs::read_to_string(&todo).unwrap();
    fs::write(&todo, format!("{original_todo}# changed review context\n")).unwrap();
    assert!(f
        .backend
        .mutate(
            &f.id,
            &reviewed.fingerprint,
            Action::Sequence {
                operation: SequenceKind::CherryPick,
                step: SequenceStep::Continue
            }
        )
        .is_err());
    fs::write(&todo, original_todo).unwrap();
    g(&f.root, &["config", "core.editor", "false"]);
    assert!(f
        .act(Action::Sequence {
            operation: SequenceKind::CherryPick,
            step: SequenceStep::Continue
        })
        .is_err());
    let paused = f.backend.snapshot(&f.id).unwrap();
    assert_eq!(paused.operation.as_deref(), Some("Cherry-pick in progress"));
    assert_eq!(
        g(&f.root, &["show", "-s", "--format=%s"]).trim(),
        "First picked commit"
    );
    assert!(paused
        .files
        .iter()
        .any(|file| file.path == "second.txt" && git::is_conflicted(file)));
    let next = f
        .act(Action::Sequence {
            operation: SequenceKind::CherryPick,
            step: SequenceStep::Abort,
        })
        .unwrap();
    assert!(next.operation.is_none() && next.files.is_empty());
    assert_eq!(g(&f.root, &["rev-parse", "HEAD"]), original);
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "Main first\n"
    );
    assert_eq!(
        fs::read_to_string(f.root.join("second.txt")).unwrap(),
        "Main second\n"
    );
}

#[test]
fn empty_cherry_pick_resolution_requires_an_explicit_skip() {
    use git::{SequenceKind, SequenceStep};
    let mut f = Fixture::new();
    interrupted_pick(&mut f);
    let head = g(&f.root, &["rev-parse", "HEAD"]);
    f.act(Action::ResolveFile {
        path: "README.md".into(),
        content: Some("Main change\n".into()),
    })
    .unwrap();
    assert!(f
        .act(Action::Sequence {
            operation: SequenceKind::CherryPick,
            step: SequenceStep::Continue
        })
        .is_err());
    assert_eq!(
        f.backend.snapshot(&f.id).unwrap().operation.as_deref(),
        Some("Cherry-pick in progress")
    );
    let next = f
        .act(Action::Sequence {
            operation: SequenceKind::CherryPick,
            step: SequenceStep::Skip,
        })
        .unwrap();
    assert!(next.operation.is_none() && next.files.is_empty());
    assert_eq!(g(&f.root, &["rev-parse", "HEAD"]), head);
}

fn hunk_fixture(f: &mut Fixture, path: &str, crlf: bool) -> (String, String) {
    let ending = if crlf { "\r\n" } else { "\n" };
    let before = (1..=50)
        .map(|n| format!("Line {n:02}"))
        .collect::<Vec<_>>()
        .join(ending);
    f.write(path, &before);
    g(&f.root, &["add", "--", path]);
    g(&f.root, &["commit", "-m", "Synthetic text base"]);
    let after = before
        .replace("Line 05", "First reviewed change")
        .replace("Line 45", "Second reviewed change");
    f.write(path, &after);
    f.backend.trust(&f.id).unwrap();
    (before, after)
}
fn act_hunk(
    f: &mut Fixture,
    path: &str,
    index: usize,
    unstage: bool,
) -> Result<git::Snapshot, String> {
    let patch = f.backend.file_diff(&f.id, path, None, unstage)?;
    f.act(Action::Hunk {
        path: path.into(),
        index,
        unstage,
        patch_hash: git::hash(patch.as_bytes()),
    })
}
#[test]
fn selected_hunk_stages_only_its_text_and_unstaging_never_touches_working_file() {
    let mut f = Fixture::new();
    let (before, after) = hunk_fixture(&mut f, "notes.txt", false);
    act_hunk(&mut f, "notes.txt", 1, false).unwrap();
    let indexed = g(&f.root, &["show", ":notes.txt"]);
    assert_eq!(indexed, before.replace("Line 45", "Second reviewed change"));
    assert_eq!(fs::read_to_string(f.root.join("notes.txt")).unwrap(), after);
    act_hunk(&mut f, "notes.txt", 0, true).unwrap();
    assert_eq!(g(&f.root, &["show", ":notes.txt"]), before);
    assert_eq!(fs::read_to_string(f.root.join("notes.txt")).unwrap(), after);
    assert!(g(&f.root, &["diff", "--cached"]).is_empty());
}
#[test]
fn unstaging_one_of_two_hunks_keeps_the_other_in_the_commit() {
    let mut f = Fixture::new();
    let (before, after) = hunk_fixture(&mut f, "notes.txt", false);
    f.act(Action::Stage {
        paths: vec!["notes.txt".into()],
    })
    .unwrap();
    act_hunk(&mut f, "notes.txt", 0, true).unwrap();
    assert_eq!(
        g(&f.root, &["show", ":notes.txt"]),
        before.replace("Line 45", "Second reviewed change")
    );
    f.act(Action::Commit {
        message: "Commit only the reviewed second hunk".into(),
    })
    .unwrap();
    assert_eq!(
        g(&f.root, &["show", "HEAD:notes.txt"]),
        before.replace("Line 45", "Second reviewed change")
    );
    assert_eq!(fs::read_to_string(f.root.join("notes.txt")).unwrap(), after);
    assert!(g(&f.root, &["diff"]).contains("First reviewed change"));
}
#[test]
fn hunk_application_preserves_crlf_no_final_newline_and_literal_filenames() {
    let mut f = Fixture::new();
    let path = if cfg!(windows) {
        "literal[1] è.txt"
    } else {
        "literal[1]\nè.txt"
    };
    g(&f.root, &["config", "core.autocrlf", "false"]);
    let (before, after) = hunk_fixture(&mut f, path, true);
    let after = after.replace("Line 50", "Changed final line");
    f.write(path, &after);
    act_hunk(&mut f, path, 1, false).unwrap();
    assert_eq!(
        g(&f.root, &["show", &format!(":{path}")]),
        before
            .replace("Line 45", "Second reviewed change")
            .replace("Line 50", "Changed final line")
    );
    assert!(!after.ends_with('\n'));
    assert_eq!(fs::read(f.root.join(path)).unwrap(), after.as_bytes());
    act_hunk(&mut f, path, 0, true).unwrap();
    assert_eq!(g(&f.root, &["show", &format!(":{path}")]), before);
    assert_eq!(fs::read(f.root.join(path)).unwrap(), after.as_bytes());
}
#[test]
fn stale_hunk_diff_or_snapshot_never_changes_the_index() {
    let mut f = Fixture::new();
    let (before, after) = hunk_fixture(&mut f, "notes.txt", false);
    let reviewed = f.backend.snapshot(&f.id).unwrap();
    let patch = f
        .backend
        .file_diff(&f.id, "notes.txt", None, false)
        .unwrap();
    f.write(
        "notes.txt",
        &after.replace("First reviewed change", "Changed externally"),
    );
    let action = Action::Hunk {
        path: "notes.txt".into(),
        index: 0,
        unstage: false,
        patch_hash: git::hash(patch.as_bytes()),
    };
    assert!(f
        .backend
        .mutate(&f.id, &reviewed.fingerprint, action.clone())
        .is_err());
    assert!(f
        .act(action)
        .unwrap_err()
        .contains("displayed diff changed"));
    assert!(act_hunk(&mut f, "notes.txt", 99, false).is_err());
    assert_eq!(g(&f.root, &["show", ":notes.txt"]), before);
    assert!(fs::read_to_string(f.root.join("notes.txt"))
        .unwrap()
        .contains("Changed externally"));
}
#[test]
fn canonical_hunk_boundaries_ignore_custom_diff_display_settings() {
    let mut f = Fixture::new();
    let (before, _) = hunk_fixture(&mut f, "notes.txt", false);
    for (key, value) in [
        ("diff.context", "100"),
        ("diff.interHunkContext", "100"),
        ("diff.noprefix", "true"),
        ("diff.mnemonicPrefix", "true"),
        ("diff.outputIndicatorNew", ">"),
    ] {
        g(&f.root, &["config", key, value]);
    }
    let patch = f
        .backend
        .file_diff(&f.id, "notes.txt", None, false)
        .unwrap();
    assert_eq!(
        patch.lines().filter(|line| line.starts_with("@@ ")).count(),
        2
    );
    assert!(patch.contains("--- a/notes.txt") && patch.contains("+First reviewed change"));
    act_hunk(&mut f, "notes.txt", 1, false).unwrap();
    assert_eq!(
        g(&f.root, &["show", ":notes.txt"]),
        before.replace("Line 45", "Second reviewed change")
    );
}
#[cfg(unix)]
#[test]
fn hunk_staging_refuses_mode_changes_without_losing_text_or_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let mut f = Fixture::new();
    let (before, after) = hunk_fixture(&mut f, "notes.txt", false);
    g(&f.root, &["config", "core.filemode", "true"]);
    fs::set_permissions(f.root.join("notes.txt"), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(act_hunk(&mut f, "notes.txt", 0, false).is_err());
    assert_eq!(g(&f.root, &["show", ":notes.txt"]), before);
    assert_eq!(fs::read_to_string(f.root.join("notes.txt")).unwrap(), after);
    assert_eq!(
        fs::metadata(f.root.join("notes.txt"))
            .unwrap()
            .permissions()
            .mode()
            & 0o111,
        0o111
    );
}

#[test]
fn binary_hunks_are_refused_without_changing_index_or_working_bytes() {
    let mut f = Fixture::new();
    f.seed();
    let before = vec![0, 255, 1, 2, 3];
    let after = vec![0, 255, 4, 5, 6];
    fs::write(f.root.join("sample.bin"), &before).unwrap();
    g(&f.root, &["add", "sample.bin"]);
    g(&f.root, &["commit", "-m", "Binary base"]);
    fs::write(f.root.join("sample.bin"), &after).unwrap();
    f.backend.trust(&f.id).unwrap();
    assert!(act_hunk(&mut f, "sample.bin", 0, false).is_err());
    assert_eq!(git::git(&f.root, &["show", ":sample.bin"]).unwrap(), before);
    assert_eq!(fs::read(f.root.join("sample.bin")).unwrap(), after);
}
#[test]
fn partial_staging_uses_only_the_selected_linked_worktree_index() {
    let mut f = Fixture::new();
    let (before, _) = hunk_fixture(&mut f, "notes.txt", false);
    f.write("notes.txt", &before);
    let linked = f._dir.path().join("linked");
    g(
        &f.root,
        &["worktree", "add", "-b", "linked", linked.to_str().unwrap()],
    );
    let after = before
        .replace("Line 05", "Linked first")
        .replace("Line 45", "Linked second");
    fs::write(linked.join("notes.txt"), &after).unwrap();
    let registered = f.backend.register(&linked).unwrap();
    f.backend.trust(&registered.repo.id).unwrap();
    let snapshot = f.backend.snapshot(&registered.repo.id).unwrap();
    let patch = f
        .backend
        .file_diff(&registered.repo.id, "notes.txt", None, false)
        .unwrap();
    f.backend
        .mutate(
            &registered.repo.id,
            &snapshot.fingerprint,
            Action::Hunk {
                path: "notes.txt".into(),
                index: 1,
                unstage: false,
                patch_hash: git::hash(patch.as_bytes()),
            },
        )
        .unwrap();
    assert!(g(&f.root, &["diff", "--cached"]).is_empty());
    assert_eq!(
        g(&linked, &["show", ":notes.txt"]),
        before.replace("Line 45", "Linked second")
    );
    assert_eq!(fs::read_to_string(linked.join("notes.txt")).unwrap(), after);
}

#[test]
fn oversized_text_hunk_refuses_without_truncating_or_staging_partial_content() {
    let mut f = Fixture::new();
    f.seed();
    let before = (0..30000)
        .map(|n| format!("Original line {n}\n"))
        .collect::<String>();
    let after = before.replace("Original", "Reviewed replacement");
    f.write("large.txt", &before);
    g(&f.root, &["add", "large.txt"]);
    g(&f.root, &["commit", "-m", "Large text base"]);
    f.write("large.txt", &after);
    f.backend.trust(&f.id).unwrap();
    let patch = f
        .backend
        .file_diff(&f.id, "large.txt", None, false)
        .unwrap();
    assert!(patch.len() > 256 * 1024);
    assert!(act_hunk(&mut f, "large.txt", 0, false)
        .unwrap_err()
        .contains("256 KiB"));
    assert_eq!(g(&f.root, &["show", ":large.txt"]), before);
    assert_eq!(fs::read_to_string(f.root.join("large.txt")).unwrap(), after);
}

#[test]
fn new_worktree_starts_at_reviewed_head_without_touching_dirty_source_or_index() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("README.md", "Unstaged source work\n");
    f.write("draft.txt", "Staged source work\n");
    g(&f.root, &["add", "draft.txt"]);
    let before = f.backend.snapshot(&f.id).unwrap();
    let index_before = g(&f.root, &["diff", "--cached"]);
    let parent = f
        .backend
        .authorize_worktree_parent(&f.id, f._dir.path())
        .unwrap();
    let next = f
        .act(Action::CreateWorktree {
            selection_id: parent.id.clone(),
            directory: "new-task".into(),
            branch: "work/new-task".into(),
            create_branch: true,
        })
        .unwrap();
    let destination = f._dir.path().join("new-task");
    assert!(next
        .worktrees
        .iter()
        .any(|tree| tree.branch == "work/new-task"));
    assert_eq!(
        g(&destination, &["rev-parse", "HEAD"]).trim(),
        before.head.unwrap()
    );
    assert_eq!(
        g(&destination, &["symbolic-ref", "--short", "HEAD"]).trim(),
        "work/new-task"
    );
    assert_eq!(
        fs::read_to_string(destination.join("README.md")).unwrap(),
        "A synthetic repository.\n"
    );
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "Unstaged source work\n"
    );
    assert_eq!(g(&f.root, &["diff", "--cached"]), index_before);
    assert!(f
        .act(Action::CreateWorktree {
            selection_id: parent.id,
            directory: "other".into(),
            branch: "work/other".into(),
            create_branch: true
        })
        .is_err());
    let opened = f
        .backend
        .open_worktree(&f.id, destination.to_str().unwrap())
        .unwrap();
    assert!(opened.files.is_empty());
    assert_eq!(opened.branch, "work/new-task");
}
#[test]
fn existing_branch_worktree_checks_out_the_branch_even_when_a_tag_shares_its_name() {
    let mut f = Fixture::new();
    f.seed();
    g(&f.root, &["branch", "side"]);
    g(&f.root, &["tag", "side"]);
    f.backend.trust(&f.id).unwrap();
    let parent = f
        .backend
        .authorize_worktree_parent(&f.id, f._dir.path())
        .unwrap();
    f.act(Action::CreateWorktree {
        selection_id: parent.id,
        directory: "side-folder".into(),
        branch: "side".into(),
        create_branch: false,
    })
    .unwrap();
    assert_eq!(
        g(
            &f._dir.path().join("side-folder"),
            &["symbolic-ref", "HEAD"]
        )
        .trim(),
        "refs/heads/side"
    );
    let another = f
        .backend
        .authorize_worktree_parent(&f.id, f._dir.path())
        .unwrap();
    assert!(f
        .act(Action::CreateWorktree {
            selection_id: another.id,
            directory: "side-again".into(),
            branch: "side".into(),
            create_branch: false
        })
        .is_err());
    assert!(!f._dir.path().join("side-again").exists());
}
#[test]
fn worktree_destination_cannot_escape_selection_or_overwrite_existing_data() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    let parent = f
        .backend
        .authorize_worktree_parent(&f.id, f._dir.path())
        .unwrap();
    for directory in [
        "../escape",
        "nested/path",
        "nested\\path",
        ".",
        "..",
        ".git",
        "CON",
        "LPT1.txt",
        "bad:folder",
        "trailing.",
    ] {
        assert!(f
            .act(Action::CreateWorktree {
                selection_id: parent.id.clone(),
                directory: directory.into(),
                branch: "work/safe".into(),
                create_branch: true
            })
            .is_err());
    }
    let existing = f._dir.path().join("existing");
    fs::create_dir(&existing).unwrap();
    fs::write(existing.join("keep.txt"), "Preserve this file").unwrap();
    assert!(f
        .act(Action::CreateWorktree {
            selection_id: parent.id,
            directory: "existing".into(),
            branch: "work/safe".into(),
            create_branch: true
        })
        .is_err());
    assert_eq!(
        fs::read_to_string(existing.join("keep.txt")).unwrap(),
        "Preserve this file"
    );
    assert!(!g(&f.root, &["branch", "--list", "work/safe"]).contains("work/safe"));
    assert!(f
        .act(Action::CreateWorktree {
            selection_id: "forged-path-token".into(),
            directory: "safe".into(),
            branch: "work/safe".into(),
            create_branch: true
        })
        .is_err());
}
#[test]
fn worktree_parent_authority_is_repository_scoped_and_never_inside_git_metadata() {
    let mut first = Fixture::new();
    first.seed();
    let mut second = Fixture::new();
    second.seed();
    second.backend.trust(&second.id).unwrap();
    assert!(first
        .backend
        .authorize_worktree_parent(&first.id, &first.root)
        .is_err());
    assert!(first
        .backend
        .authorize_worktree_parent(&first.id, &first.root.join(".git"))
        .is_err());
    let parent = first
        .backend
        .authorize_worktree_parent(&first.id, first._dir.path())
        .unwrap();
    let other = first.backend.register(&second.root).unwrap();
    first.backend.trust(&other.repo.id).unwrap();
    let other = first.backend.snapshot(&other.repo.id).unwrap();
    let error = first
        .backend
        .mutate(
            &other.repo.id,
            &other.fingerprint,
            Action::CreateWorktree {
                selection_id: parent.id.clone(),
                directory: "wrong-owner".into(),
                branch: "work/wrong".into(),
                create_branch: true,
            },
        )
        .unwrap_err();
    assert!(error.contains("another repository"));
    assert!(!first._dir.path().join("wrong-owner").exists());
    assert!(second
        .act(Action::CreateWorktree {
            selection_id: parent.id,
            directory: "wrong-repo".into(),
            branch: "work/wrong".into(),
            create_branch: true
        })
        .is_err());
    assert!(!first._dir.path().join("wrong-repo").exists());
}
#[test]
fn worktree_occupancy_change_invalidates_a_review_even_when_refs_and_head_match() {
    let mut f = Fixture::new();
    f.seed();
    g(&f.root, &["branch", "side"]);
    f.backend.trust(&f.id).unwrap();
    let reviewed = f.backend.snapshot(&f.id).unwrap();
    let linked = f._dir.path().join("external");
    g(
        &f.root,
        &["worktree", "add", linked.to_str().unwrap(), "side"],
    );
    let current = f.backend.snapshot(&f.id).unwrap();
    assert_eq!(reviewed.head, current.head);
    assert_ne!(reviewed.fingerprint, current.fingerprint);
    assert!(f
        .backend
        .mutate(
            &f.id,
            &reviewed.fingerprint,
            Action::DeleteBranch {
                name: "side".into()
            }
        )
        .is_err());
    assert!(linked.join("README.md").exists());
}
#[cfg(unix)]
#[test]
fn replacing_an_authorized_parent_with_a_symlink_never_redirects_worktree_creation() {
    use std::os::unix::fs::symlink;
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    let selected = f._dir.path().join("selected");
    let outside = f._dir.path().join("outside");
    fs::create_dir(&selected).unwrap();
    fs::create_dir(&outside).unwrap();
    let token = f
        .backend
        .authorize_worktree_parent(&f.id, &selected)
        .unwrap();
    fs::remove_dir(&selected).unwrap();
    symlink(&outside, &selected).unwrap();
    assert!(f
        .act(Action::CreateWorktree {
            selection_id: token.id,
            directory: "must-not-exist".into(),
            branch: "work/safe".into(),
            create_branch: true
        })
        .is_err());
    assert!(!outside.join("must-not-exist").exists());
}

#[test]
fn stash_preview_includes_tracked_and_saved_untracked_literal_paths_without_writes() {
    let mut f = Fixture::new();
    f.seed();
    f.write("README.md", "Saved tracked edit\n");
    let saved_path = if cfg!(windows) {
        "new folder/α[file].txt"
    } else {
        "new folder/α\nfile.txt"
    };
    f.write(saved_path, "Untracked saved contents\n");
    g(
        &f.root,
        &["stash", "push", "-u", "-m", "Preview synthetic stash"],
    );
    let before = f.backend.snapshot(&f.id).unwrap();
    let stash = &before.stashes[0];
    let details = f.backend.stash_details(&f.id, 0, &stash.oid).unwrap();
    assert_eq!(details.tracked.len(), 1);
    assert_eq!(details.tracked[0].path, "README.md");
    assert_eq!(details.untracked.len(), 1);
    assert_eq!(details.untracked[0].path, saved_path);
    assert!(f
        .backend
        .stash_diff(&f.id, 0, &stash.oid, "README.md", false)
        .unwrap()
        .contains("+Saved tracked edit"));
    assert!(f
        .backend
        .stash_diff(&f.id, 0, &stash.oid, saved_path, true)
        .unwrap()
        .contains("+Untracked saved contents"));
    assert!(f
        .backend
        .stash_diff(&f.id, 0, &stash.oid, "README.md", true)
        .is_err());
    assert!(f
        .backend
        .stash_diff(&f.id, 0, &stash.oid, "../outside", false)
        .is_err());
    assert_eq!(
        before.fingerprint,
        f.backend.snapshot(&f.id).unwrap().fingerprint
    );
    assert!(!f.root.join("new folder/α\nfile.txt").exists());
}
#[test]
fn pop_restores_files_and_untracked_only_then_removes_the_selected_stash() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("README.md", "Saved older edit\n");
    g(&f.root, &["stash", "push", "-m", "Older"]);
    f.write("README.md", "Saved latest edit\n");
    f.write("draft.txt", "Keep untracked\n");
    g(&f.root, &["add", "README.md"]);
    g(&f.root, &["stash", "push", "-u", "-m", "Latest"]);
    let before = f.backend.snapshot(&f.id).unwrap();
    let older = before.stashes[1].oid.clone();
    f.act(Action::PopStash {
        index: 0,
        oid: before.stashes[0].oid.clone(),
    })
    .unwrap();
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "Saved latest edit\n"
    );
    assert_eq!(
        fs::read_to_string(f.root.join("draft.txt")).unwrap(),
        "Keep untracked\n"
    );
    let after = f.backend.snapshot(&f.id).unwrap();
    assert_eq!(after.head, before.head);
    assert_eq!(after.stashes.len(), 1);
    assert_eq!(after.stashes[0].oid, older);
    assert!(
        g(&f.root, &["diff", "--cached"]).is_empty(),
        "default apply does not restore the old staging layout"
    );
}
#[test]
fn pop_conflicts_keep_stash_and_original_head_with_resolvable_index() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("README.md", "Saved stash version\n");
    g(&f.root, &["stash", "push", "-m", "Conflict"]);
    f.write("README.md", "Committed different version\n");
    g(&f.root, &["commit", "-am", "Diverged main"]);
    let before = f.backend.snapshot(&f.id).unwrap();
    let oid = before.stashes[0].oid.clone();
    let error = f
        .act(Action::PopStash {
            index: 0,
            oid: oid.clone(),
        })
        .unwrap_err();
    assert!(error.contains("Stash retained"));
    let after = f.backend.snapshot(&f.id).unwrap();
    assert_eq!(after.head, before.head);
    assert_eq!(after.stashes[0].oid, oid);
    assert!(after.files.iter().any(git::is_conflicted));
    assert!(!g(&f.root, &["ls-files", "-u"]).is_empty());
    assert!(f.act(Action::DropStash { index: 0, oid }).is_err());
}
#[test]
fn drop_removes_only_reviewed_entry_without_touching_dirty_work_or_index() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    for message in ["Older", "Latest"] {
        f.write("README.md", message);
        g(&f.root, &["stash", "push", "-m", message]);
    }
    f.write("README.md", "Current dirty work\n");
    f.write("staged.txt", "Current staged work\n");
    g(&f.root, &["add", "staged.txt"]);
    let before = f.backend.snapshot(&f.id).unwrap();
    let tree = g(&f.root, &["write-tree"]);
    f.act(Action::DropStash {
        index: 1,
        oid: before.stashes[1].oid.clone(),
    })
    .unwrap();
    assert_eq!(g(&f.root, &["write-tree"]), tree);
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "Current dirty work\n"
    );
    assert_eq!(
        f.backend.snapshot(&f.id).unwrap().stashes[0].oid,
        before.stashes[0].oid
    );
}
#[test]
fn stash_identity_and_fingerprint_reject_same_message_replacement_and_stack_shifts() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("README.md", "First\n");
    g(&f.root, &["stash", "push", "-m", "Same message"]);
    let reviewed = f.backend.snapshot(&f.id).unwrap();
    let oid = reviewed.stashes[0].oid.clone();
    g(&f.root, &["stash", "drop"]);
    f.write("README.md", "Second\n");
    g(&f.root, &["stash", "push", "-m", "Same message"]);
    let current = f.backend.snapshot(&f.id).unwrap();
    assert_eq!(reviewed.stashes[0].message, current.stashes[0].message);
    assert_ne!(reviewed.fingerprint, current.fingerprint);
    assert!(f
        .backend
        .mutate(
            &f.id,
            &reviewed.fingerprint,
            Action::DropStash {
                index: 0,
                oid: oid.clone()
            }
        )
        .is_err());
    assert!(f
        .act(Action::ApplyStash {
            index: 0,
            oid: oid.clone()
        })
        .is_err());
    assert!(f.backend.stash_details(&f.id, 0, &oid).is_err());
    assert!(f.backend.snapshot(&f.id).unwrap().files.is_empty());
}
#[test]
fn failed_untracked_restore_retains_stash_and_never_overwrites_existing_file() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("draft.txt", "Saved untracked\n");
    g(
        &f.root,
        &["stash", "push", "-u", "-m", "Untracked collision"],
    );
    let oid = f.backend.snapshot(&f.id).unwrap().stashes[0].oid.clone();
    f.write("draft.txt", "Current untracked sentinel\n");
    assert!(f
        .act(Action::PopStash {
            index: 0,
            oid: oid.clone()
        })
        .is_err());
    assert_eq!(
        fs::read_to_string(f.root.join("draft.txt")).unwrap(),
        "Current untracked sentinel\n"
    );
    assert_eq!(f.backend.snapshot(&f.id).unwrap().stashes[0].oid, oid);
}

#[test]
fn saving_stash_cleans_the_saved_untracked_files_and_preserves_them_in_git_objects() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("README.md", "Tracked saved edit\n");
    f.write("draft notes.txt", "Untracked saved edit\n");
    let saved = f
        .act(Action::Stash {
            message: "Save both file kinds".into(),
        })
        .unwrap();
    assert!(saved.files.is_empty());
    assert!(!f.root.join("draft notes.txt").exists());
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "A synthetic repository.\n"
    );
    assert!(f
        .backend
        .stash_diff(&f.id, 0, &saved.stashes[0].oid, "draft notes.txt", true)
        .unwrap()
        .contains("+Untracked saved edit"));
}

#[test]
fn direct_comparison_is_directional_read_only_and_excludes_dirty_work_and_index() {
    use ramo_core::compare::Kind;
    let mut f = Fixture::new();
    f.seed();
    let base = g(&f.root, &["rev-parse", "HEAD"]).trim().to_string();
    f.write("README.md", "Committed target\n");
    g(&f.root, &["commit", "-am", "Target revision"]);
    let target = g(&f.root, &["rev-parse", "HEAD"]).trim().to_string();
    f.write("README.md", "Dirty sentinel\n");
    f.write("staged.txt", "Staged sentinel\n");
    g(&f.root, &["add", "staged.txt"]);
    let before = f.backend.snapshot(&f.id).unwrap();
    let tree = g(&f.root, &["write-tree"]);
    let comparison = f
        .backend
        .compare_revisions(&f.id, &base, &target, Kind::Trees)
        .unwrap();
    assert_eq!(comparison.files.len(), 1);
    assert_eq!(comparison.files[0].path, "README.md");
    let patch = f
        .backend
        .comparison_diff(&f.id, &base, &target, Kind::Trees, "README.md")
        .unwrap();
    assert!(patch.contains("+Committed target"));
    assert!(!patch.contains("Dirty sentinel"));
    let reverse = f
        .backend
        .comparison_diff(&f.id, &target, &base, Kind::Trees, "README.md")
        .unwrap();
    assert!(reverse.contains("-Committed target"));
    assert!(f
        .backend
        .compare_revisions(&f.id, &target, &target, Kind::Trees)
        .unwrap()
        .files
        .is_empty());
    assert_eq!(
        f.backend.snapshot(&f.id).unwrap().fingerprint,
        before.fingerprint
    );
    assert_eq!(g(&f.root, &["write-tree"]), tree);
    assert!(!before.repo.trusted);
}
#[test]
fn common_ancestor_comparison_omits_changes_unique_to_the_base_branch() {
    use ramo_core::compare::Kind;
    let mut f = Fixture::new();
    f.seed();
    let ancestor = g(&f.root, &["rev-parse", "HEAD"]).trim().to_string();
    g(&f.root, &["switch", "-c", "side"]);
    f.write("target.txt", "Target branch\n");
    g(&f.root, &["add", "target.txt"]);
    g(&f.root, &["commit", "-m", "Target only"]);
    let target = g(&f.root, &["rev-parse", "HEAD"]).trim().to_string();
    g(&f.root, &["switch", "main"]);
    f.write("base.txt", "Base branch\n");
    g(&f.root, &["add", "base.txt"]);
    g(&f.root, &["commit", "-m", "Base only"]);
    let base = g(&f.root, &["rev-parse", "HEAD"]).trim().to_string();
    let direct = f
        .backend
        .compare_revisions(&f.id, &base, &target, Kind::Trees)
        .unwrap();
    assert_eq!(direct.files.len(), 2);
    let introduced = f
        .backend
        .compare_revisions(&f.id, &base, &target, Kind::MergeBase)
        .unwrap();
    assert_eq!(introduced.effective_from, ancestor);
    assert_eq!(introduced.files.len(), 1);
    assert_eq!(introduced.files[0].path, "target.txt");
    assert!(f
        .backend
        .comparison_diff(&f.id, &base, &target, Kind::MergeBase, "base.txt")
        .is_err());
    assert!(f
        .backend
        .comparison_diff(&f.id, &base, &target, Kind::MergeBase, "target.txt")
        .unwrap()
        .contains("+Target branch"));
}
#[test]
fn comparison_handles_literal_paths_binary_deletion_and_branch_moves_by_object_pair() {
    use ramo_core::compare::Kind;
    let mut f = Fixture::new();
    f.seed();
    f.write("old.txt", "Rename source\n");
    g(&f.root, &["add", "old.txt"]);
    g(&f.root, &["commit", "-m", "Base files"]);
    let base = g(&f.root, &["rev-parse", "HEAD"]).trim().to_string();
    g(&f.root, &["mv", "old.txt", "new.txt"]);
    let literal_path = if cfg!(windows) {
        "odd/α[file].txt"
    } else {
        "odd/α\nfile.txt"
    };
    f.write(literal_path, "Literal path contents\n");
    fs::write(f.root.join("image.bin"), [0, 1, 2, 3]).unwrap();
    f.write("huge.txt", &"x".repeat(600 * 1024));
    g(&f.root, &["add", "."]);
    g(&f.root, &["commit", "-m", "Target files"]);
    let target = g(&f.root, &["rev-parse", "HEAD"]).trim().to_string();
    f.write("later.txt", "Not in comparison\n");
    g(&f.root, &["add", "later.txt"]);
    g(&f.root, &["commit", "-m", "Branch moved"]);
    let pair = f
        .backend
        .compare_revisions(&f.id, &base, &target, Kind::Trees)
        .unwrap();
    assert_eq!(pair.files.len(), 5);
    assert!(pair
        .files
        .iter()
        .any(|file| file.path == "old.txt" && file.index == "D"));
    assert!(pair
        .files
        .iter()
        .any(|file| file.path == "new.txt" && file.index == "A"));
    assert!(f
        .backend
        .comparison_diff(&f.id, &base, &target, Kind::Trees, literal_path)
        .unwrap()
        .contains("+Literal path contents"));
    assert!(f
        .backend
        .comparison_diff(&f.id, &base, &target, Kind::Trees, "image.bin")
        .unwrap()
        .contains("Binary files"));
    assert!(f
        .backend
        .comparison_diff(&f.id, &base, &target, Kind::Trees, "later.txt")
        .is_err());
    assert!(f
        .backend
        .comparison_diff(&f.id, &base, &target, Kind::Trees, "huge.txt")
        .unwrap_err()
        .contains("512 KiB"));
}

#[test]
fn comparison_rejects_invalid_objects_and_unrelated_ancestor_mode_without_writes() {
    use ramo_core::compare::Kind;
    let mut f = Fixture::new();
    f.seed();
    let first = g(&f.root, &["rev-parse", "HEAD"]).trim().to_string();
    let tree = g(&f.root, &["rev-parse", "HEAD^{tree}"]).trim().to_string();
    g(&f.root, &["checkout", "--orphan", "independent"]);
    g(&f.root, &["commit", "-m", "Independent root"]);
    let second = g(&f.root, &["rev-parse", "HEAD"]).trim().to_string();
    assert!(f
        .backend
        .compare_revisions(&f.id, &first, &second, Kind::MergeBase)
        .unwrap_err()
        .contains("common ancestor"));
    assert!(f
        .backend
        .compare_revisions(&f.id, &first, &second, Kind::Trees)
        .unwrap()
        .files
        .is_empty());
    for invalid in ["HEAD", "--output=/tmp/escape", &tree] {
        assert!(f
            .backend
            .compare_revisions(&f.id, invalid, &second, Kind::Trees)
            .is_err());
    }
    assert!(f
        .backend
        .comparison_diff(&f.id, &first, &second, Kind::Trees, "../outside")
        .is_err());
    assert!(f
        .backend
        .comparison_diff(&f.id, &first, &second, Kind::Trees, "README.md")
        .is_err());
}

#[test]
fn comparison_does_not_guess_among_multiple_common_ancestors() {
    use ramo_core::compare::Kind;
    let mut f = Fixture::new();
    f.seed();
    let root = g(&f.root, &["rev-parse", "HEAD"]).trim().to_string();
    let tree = g(&f.root, &["rev-parse", "HEAD^{tree}"]).trim().to_string();
    let left = g(
        &f.root,
        &["commit-tree", &tree, "-p", &root, "-m", "Left ancestor"],
    )
    .trim()
    .to_string();
    let right = g(
        &f.root,
        &["commit-tree", &tree, "-p", &root, "-m", "Right ancestor"],
    )
    .trim()
    .to_string();
    let first = g(
        &f.root,
        &[
            "commit-tree",
            &tree,
            "-p",
            &left,
            "-p",
            &right,
            "-m",
            "First crossing merge",
        ],
    )
    .trim()
    .to_string();
    let second = g(
        &f.root,
        &[
            "commit-tree",
            &tree,
            "-p",
            &right,
            "-p",
            &left,
            "-m",
            "Second crossing merge",
        ],
    )
    .trim()
    .to_string();
    assert!(f
        .backend
        .compare_revisions(&f.id, &first, &second, Kind::MergeBase)
        .unwrap_err()
        .contains("Multiple common ancestors"));
    assert!(f
        .backend
        .compare_revisions(&f.id, &first, &second, Kind::Trees)
        .unwrap()
        .files
        .is_empty());
}

#[test]
fn stash_conflict_resolution_can_commit_normally_without_consuming_stash_or_other_work() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("README.md", "Saved stash text\n");
    f.write("saved-note.txt", "Saved untracked text\n");
    f.act(Action::Stash {
        message: "Retain through conflict recovery".into(),
    })
    .unwrap();
    f.write("README.md", "Different committed main text\n");
    g(&f.root, &["commit", "-am", "Diverged main"]);
    let before = f.backend.snapshot(&f.id).unwrap();
    let stash = before.stashes[0].oid.clone();
    assert!(f
        .act(Action::PopStash {
            index: 0,
            oid: stash.clone()
        })
        .is_err());
    let conflicted = f.backend.snapshot(&f.id).unwrap();
    assert!(conflicted.operation.is_none());
    assert!(conflicted.files.iter().any(git::is_conflicted));
    assert!(f
        .act(Action::Commit {
            message: "Must not commit unresolved files".into()
        })
        .is_err());
    f.write("unrelated.txt", "Preserve unrelated untracked work\n");
    f.act(Action::ResolveFile {
        path: "README.md".into(),
        content: Some("Reviewed combined stash result\n".into()),
    })
    .unwrap();
    let resolved = f.backend.snapshot(&f.id).unwrap();
    assert!(resolved.operation.is_none());
    assert!(!resolved.files.iter().any(git::is_conflicted));
    assert_eq!(resolved.stashes[0].oid, stash);
    f.act(Action::Commit {
        message: "Keep reviewed stash resolution".into(),
    })
    .unwrap();
    let after = f.backend.snapshot(&f.id).unwrap();
    assert_eq!(after.stashes[0].oid, stash);
    assert!(after.operation.is_none());
    let parents = g(&f.root, &["rev-list", "--parents", "-n", "1", "HEAD"]);
    let parents: Vec<_> = parents.split_whitespace().collect();
    assert_eq!(parents.len(), 2);
    assert_eq!(parents[1], before.head.as_deref().unwrap());
    assert_eq!(
        g(&f.root, &["show", "--pretty=", "--name-only", "HEAD"]).trim(),
        "README.md"
    );
    assert_eq!(
        g(&f.root, &["show", "HEAD:README.md"]),
        "Reviewed combined stash result\n"
    );
    assert_eq!(
        fs::read_to_string(f.root.join("unrelated.txt")).unwrap(),
        "Preserve unrelated untracked work\n"
    );
    assert_eq!(
        g(&f.root, &["show", &format!("{stash}^3:saved-note.txt")]),
        "Saved untracked text\n"
    );
}

#[test]
fn clone_review_opens_a_real_independent_checkout_without_trusting_it_or_changing_source_work() {
    use ramo_core::cloning::Source;
    let mut f = Fixture::new();
    f.seed();
    g(&f.root, &["tag", "v-fixture"]);
    let bare = f._dir.path().join("source.git");
    g(
        &f.root,
        &[
            "clone",
            "--bare",
            f.root.to_str().unwrap(),
            bare.to_str().unwrap(),
        ],
    );
    f.write("README.md", "Unstaged source sentinel\n");
    f.write("source-only.txt", "Staged source sentinel\n");
    g(&f.root, &["add", "source-only.txt"]);
    let tree = g(&f.root, &["write-tree"]);
    let head = g(&f.root, &["rev-parse", "HEAD"]).trim().to_string();
    let parent = f.backend.authorize_clone_parent(f._dir.path()).unwrap();
    let source = f.backend.authorize_clone_source(&bare).unwrap();
    let plan = f
        .backend
        .prepare_clone(&parent.id, "cloned", Source::Local { id: source.id })
        .unwrap();
    assert!(!plan.network);
    assert!(!f._dir.path().join("cloned").exists());
    let cloned = f.backend.clone_repository(&plan.id).unwrap();
    let path = Path::new(&cloned.repo.path);
    assert_eq!(cloned.head.as_deref(), Some(head.as_str()));
    assert!(!cloned.repo.trusted);
    assert!(cloned.files.is_empty());
    assert_eq!(cloned.upstream.as_deref(), Some("origin/main"));
    assert_eq!(g(path, &["tag", "--list"]).trim(), "v-fixture");
    assert!(!path.join("source-only.txt").exists());
    assert!(!path.join(".git/objects/info/alternates").exists());
    assert!(!g(path, &["config", "--local", "--list"]).contains("protocol."));
    assert_eq!(g(&f.root, &["write-tree"]), tree);
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "Unstaged source sentinel\n"
    );
    assert!(f.backend.clone_repository(&plan.id).is_err());
}
#[test]
fn clone_never_reuses_existing_destinations_or_accepts_renderer_paths_as_grants() {
    use ramo_core::cloning::Source;
    let mut f = Fixture::new();
    f.seed();
    let parent = f.backend.authorize_clone_parent(f._dir.path()).unwrap();
    let source = f.backend.authorize_clone_source(&f.root).unwrap();
    let local = Source::Local { id: source.id };
    assert!(f.backend.authorize_clone_parent(&f.root).is_err());
    assert!(f
        .backend
        .prepare_clone("/tmp/forged", "clone", local.clone())
        .is_err());
    assert!(f
        .backend
        .prepare_clone(&parent.id, "../escape", local.clone())
        .is_err());
    let plan = f
        .backend
        .prepare_clone(&parent.id, "collision", local.clone())
        .unwrap();
    let destination = f._dir.path().join("collision");
    fs::create_dir(&destination).unwrap();
    fs::write(destination.join("keep.txt"), "Preserve existing data").unwrap();
    assert!(f.backend.clone_repository(&plan.id).is_err());
    assert_eq!(
        fs::read_to_string(destination.join("keep.txt")).unwrap(),
        "Preserve existing data"
    );
    assert!(f
        .backend
        .prepare_clone(&parent.id, "collision", local)
        .is_err());
    assert!(f
        .backend
        .prepare_clone(
            &parent.id,
            "unsafe",
            Source::Url {
                url: "ext::touch marker".into()
            }
        )
        .is_err());
    assert!(!f._dir.path().join("unsafe").exists());
}
#[cfg(unix)]
#[test]
fn clone_parent_replacement_cannot_redirect_the_reviewed_destination() {
    use ramo_core::cloning::Source;
    let mut f = Fixture::new();
    f.seed();
    let parent = f._dir.path().join("parent");
    let outside = f._dir.path().join("outside");
    fs::create_dir(&parent).unwrap();
    fs::create_dir(&outside).unwrap();
    let grant = f.backend.authorize_clone_parent(&parent).unwrap();
    let source = f.backend.authorize_clone_source(&f.root).unwrap();
    let plan = f
        .backend
        .prepare_clone(&grant.id, "new-clone", Source::Local { id: source.id })
        .unwrap();
    fs::remove_dir(&parent).unwrap();
    std::os::unix::fs::symlink(&outside, &parent).unwrap();
    assert!(f.backend.clone_repository(&plan.id).is_err());
    assert!(!outside.join("new-clone").exists());
}
#[test]
fn clone_failure_consumes_review_and_preserves_other_repositories_and_files() {
    use ramo_core::cloning::Source;
    let mut f = Fixture::new();
    f.seed();
    let bare = f._dir.path().join("broken.git");
    g(
        &f.root,
        &[
            "clone",
            "--bare",
            f.root.to_str().unwrap(),
            bare.to_str().unwrap(),
        ],
    );
    let head = g(&f.root, &["rev-parse", "HEAD"]).trim().to_string();
    fs::create_dir_all(bare.join("refs/heads")).unwrap();
    fs::write(
        bare.join("refs/heads/main"),
        format!("{}\n", "1".repeat(40)),
    )
    .unwrap();
    let parent = f.backend.authorize_clone_parent(f._dir.path()).unwrap();
    let source = f.backend.authorize_clone_source(&bare).unwrap();
    let plan = f
        .backend
        .prepare_clone(&parent.id, "failed-clone", Source::Local { id: source.id })
        .unwrap();
    let error = f.backend.clone_repository(&plan.id).unwrap_err();
    assert!(error.contains("Inspect the destination"));
    assert!(f.backend.clone_repository(&plan.id).is_err());
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "A synthetic repository.\n"
    );
    assert_eq!(g(&f.root, &["rev-parse", "HEAD"]).trim(), head);
}

#[test]
fn cloning_an_empty_bare_repository_opens_an_unborn_untrusted_checkout() {
    use ramo_core::cloning::Source;
    let mut f = Fixture::new();
    let bare = f._dir.path().join("empty.git");
    g(&f.root, &["init", "--bare", bare.to_str().unwrap()]);
    let parent = f.backend.authorize_clone_parent(f._dir.path()).unwrap();
    let source = f.backend.authorize_clone_source(&bare).unwrap();
    let plan = f
        .backend
        .prepare_clone(&parent.id, "empty-clone", Source::Local { id: source.id })
        .unwrap();
    let cloned = f.backend.clone_repository(&plan.id).unwrap();
    assert!(cloned.head.is_none());
    assert!(cloned.files.is_empty());
    assert!(!cloned.repo.trusted);
    assert!(f.backend.history(&cloned.repo.id, 0).unwrap().is_empty());
}
#[test]
fn clone_url_review_is_immutable_and_unrecognized_review_tokens_never_create_a_directory() {
    use ramo_core::cloning::Source;
    let mut f = Fixture::new();
    let parent = f.backend.authorize_clone_parent(f._dir.path()).unwrap();
    let plan = f
        .backend
        .prepare_clone(
            &parent.id,
            "url-clone",
            Source::Url {
                url: "https://example.invalid/team/repo.git".into(),
            },
        )
        .unwrap();
    assert!(plan.network);
    assert_eq!(plan.source, "https://example.invalid/team/repo.git");
    assert!(!f._dir.path().join("url-clone").exists());
    let mut other = Fixture::new();
    assert!(other.backend.clone_repository(&plan.id).is_err());
    assert!(!f._dir.path().join("url-clone").exists());
    assert!(f.backend.clone_repository("forged-review").is_err());
}

#[test]
fn restore_saved_untracked_file_preserves_binary_bytes_and_never_changes_index_or_stash() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    let name = if cfg!(windows) {
        "nested/α[image].bin"
    } else {
        "nested/α\nimage.bin"
    };
    fs::create_dir(f.root.join("nested")).unwrap();
    let bytes = [0, 255, 1, 13, 10, 12, 0];
    fs::write(f.root.join(name), bytes).unwrap();
    f.act(Action::Stash {
        message: "Saved binary file".into(),
    })
    .unwrap();
    if f.root.join("nested").exists() {
        fs::remove_dir(f.root.join("nested")).unwrap();
    }
    let before = f.backend.snapshot(&f.id).unwrap();
    let oid = before.stashes[0].oid.clone();
    let tree = g(&f.root, &["write-tree"]);
    f.act(Action::RestoreStashFile {
        index: 0,
        oid: oid.clone(),
        path: name.into(),
    })
    .unwrap();
    assert_eq!(fs::read(f.root.join(name)).unwrap(), bytes);
    assert_eq!(g(&f.root, &["write-tree"]), tree);
    let after = f.backend.snapshot(&f.id).unwrap();
    assert_eq!(after.head, before.head);
    assert_eq!(after.stashes[0].oid, oid);
    assert!(after
        .files
        .iter()
        .any(|file| file.path == name && file.index == "?"));
    fs::write(f.root.join(name), "Current sentinel").unwrap();
    assert!(f
        .act(Action::RestoreStashFile {
            index: 0,
            oid,
            path: name.into()
        })
        .is_err());
    assert_eq!(
        fs::read_to_string(f.root.join(name)).unwrap(),
        "Current sentinel"
    );
}
#[test]
fn restoring_a_missing_stash_file_is_allowed_during_marker_free_conflicts_but_preserves_them() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("README.md", "Saved side\n");
    f.write("notes.txt", "Saved missing note\r\nwithout final newline");
    f.act(Action::Stash {
        message: "Partial restoration".into(),
    })
    .unwrap();
    f.write("README.md", "Committed current\n");
    g(&f.root, &["commit", "-am", "Current branch"]);
    let saved = f.backend.snapshot(&f.id).unwrap();
    let oid = saved.stashes[0].oid.clone();
    assert!(f
        .act(Action::PopStash {
            index: 0,
            oid: oid.clone()
        })
        .is_err());
    // Different Git versions may already restore the untracked file; remove only that disposable fixture copy.
    if f.root.join("notes.txt").exists() {
        fs::remove_file(f.root.join("notes.txt")).unwrap();
    }
    let stages = g(&f.root, &["ls-files", "-u"]);
    let conflict = fs::read(f.root.join("README.md")).unwrap();
    let before = f.backend.snapshot(&f.id).unwrap();
    assert!(before.operation.is_none());
    f.act(Action::RestoreStashFile {
        index: 0,
        oid: oid.clone(),
        path: "notes.txt".into(),
    })
    .unwrap();
    assert_eq!(g(&f.root, &["ls-files", "-u"]), stages);
    assert_eq!(fs::read(f.root.join("README.md")).unwrap(), conflict);
    assert_eq!(
        fs::read(f.root.join("notes.txt")).unwrap(),
        b"Saved missing note\r\nwithout final newline"
    );
    let after = f.backend.snapshot(&f.id).unwrap();
    assert_eq!(after.stashes[0].oid, oid);
    assert_eq!(after.head, before.head);
}
#[test]
fn restore_stash_file_refuses_tracked_paths_stale_reviews_and_unrelated_paths() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("saved.txt", "Saved file\n");
    f.act(Action::Stash {
        message: "Save".into(),
    })
    .unwrap();
    let saved = f.backend.snapshot(&f.id).unwrap();
    let oid = saved.stashes[0].oid.clone();
    for path in ["../outside", ".git/config", "README.md", "missing.txt"] {
        assert!(f
            .act(Action::RestoreStashFile {
                index: 0,
                oid: oid.clone(),
                path: path.into()
            })
            .is_err());
    }
    f.write("saved.txt", "Current tracked version\n");
    g(&f.root, &["add", "saved.txt"]);
    g(&f.root, &["commit", "-m", "Path now tracked"]);
    fs::remove_file(f.root.join("saved.txt")).unwrap();
    assert!(f
        .act(Action::RestoreStashFile {
            index: 0,
            oid: oid.clone(),
            path: "saved.txt".into()
        })
        .is_err());
    assert!(!f.root.join("saved.txt").exists());
    g(&f.root, &["rm", "--cached", "saved.txt"]);
    assert!(f
        .act(Action::RestoreStashFile {
            index: 0,
            oid: oid.clone(),
            path: "saved.txt".into()
        })
        .is_err());
    assert!(f
        .backend
        .mutate(
            &f.id,
            &saved.fingerprint,
            Action::RestoreStashFile {
                index: 0,
                oid,
                path: "saved.txt".into()
            }
        )
        .is_err());
}
#[cfg(unix)]
#[test]
fn stash_file_restoration_refuses_symlinks_and_keeps_executable_bit_private() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("directory/run.sh", "#!/bin/sh\necho synthetic\n");
    fs::set_permissions(
        f.root.join("directory/run.sh"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    symlink("README.md", f.root.join("link")).unwrap();
    f.act(Action::Stash {
        message: "Executable and link".into(),
    })
    .unwrap();
    let oid = f.backend.snapshot(&f.id).unwrap().stashes[0].oid.clone();
    assert!(f
        .act(Action::RestoreStashFile {
            index: 0,
            oid: oid.clone(),
            path: "link".into()
        })
        .is_err());
    let outside = f._dir.path().join("outside");
    fs::create_dir(&outside).unwrap();
    if f.root.join("directory").exists() {
        fs::remove_dir(f.root.join("directory")).unwrap();
    }
    symlink(&outside, f.root.join("directory")).unwrap();
    assert!(f
        .act(Action::RestoreStashFile {
            index: 0,
            oid: oid.clone(),
            path: "directory/run.sh".into()
        })
        .is_err());
    assert!(!outside.join("run.sh").exists());
    fs::remove_file(f.root.join("directory")).unwrap();
    f.act(Action::RestoreStashFile {
        index: 0,
        oid: oid.clone(),
        path: "directory/run.sh".into(),
    })
    .unwrap();
    assert_eq!(
        fs::metadata(f.root.join("directory/run.sh"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    fs::remove_file(f.root.join("directory/run.sh")).unwrap();
    symlink(outside.join("absent"), f.root.join("directory/run.sh")).unwrap();
    assert!(f
        .act(Action::RestoreStashFile {
            index: 0,
            oid,
            path: "directory/run.sh".into()
        })
        .is_err());
    assert!(!outside.join("absent").exists());
}
#[test]
fn stash_file_restoration_checks_size_and_blocks_active_sequences_without_writes() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("huge.bin", &"x".repeat(8 * 1024 * 1024 + 1));
    f.write("small.txt", "Saved small\n");
    f.act(Action::Stash {
        message: "Bounded recovery".into(),
    })
    .unwrap();
    let oid = f.backend.snapshot(&f.id).unwrap().stashes[0].oid.clone();
    assert!(f
        .act(Action::RestoreStashFile {
            index: 0,
            oid: oid.clone(),
            path: "huge.bin".into()
        })
        .unwrap_err()
        .contains("8 MiB"));
    assert!(!f.root.join("huge.bin").exists());
    let target = g(&f.root, &["rev-parse", "HEAD"]);
    fs::write(f.root.join(".git/CHERRY_PICK_HEAD"), target).unwrap();
    assert!(f
        .act(Action::RestoreStashFile {
            index: 0,
            oid,
            path: "small.txt".into()
        })
        .is_err());
    assert!(!f.root.join("small.txt").exists());
}

#[test]
fn stash_file_restore_distinguishes_shared_directories_from_deleted_tracked_parent_files() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("folder/tracked.txt", "Tracked leaf\n");
    g(&f.root, &["add", "folder/tracked.txt"]);
    g(&f.root, &["commit", "-m", "Tracked folder leaf"]);
    f.write("folder/new.txt", "Saved alongside tracked files\n");
    f.write("obstruction/new.txt", "Saved under missing parent\n");
    f.act(Action::Stash {
        message: "Parent boundaries".into(),
    })
    .unwrap();
    let oid = f.backend.snapshot(&f.id).unwrap().stashes[0].oid.clone();
    f.act(Action::RestoreStashFile {
        index: 0,
        oid: oid.clone(),
        path: "folder/new.txt".into(),
    })
    .unwrap();
    assert_eq!(
        fs::read_to_string(f.root.join("folder/tracked.txt")).unwrap(),
        "Tracked leaf\n"
    );
    if f.root.join("obstruction").exists() {
        fs::remove_dir(f.root.join("obstruction")).unwrap();
    }
    f.write("obstruction", "Current tracked parent file\n");
    g(&f.root, &["add", "obstruction"]);
    g(&f.root, &["commit", "-m", "Parent path is a file"]);
    fs::remove_file(f.root.join("obstruction")).unwrap();
    assert!(f
        .act(Action::RestoreStashFile {
            index: 0,
            oid,
            path: "obstruction/new.txt".into()
        })
        .is_err());
    assert!(!f.root.join("obstruction").exists());
}

fn managed_checkout(f: &mut Fixture) -> std::path::PathBuf {
    f.seed();
    f.backend.trust(&f.id).unwrap();
    let linked = f._dir.path().join("linked checkout");
    g(
        &f.root,
        &[
            "worktree",
            "add",
            "-b",
            "work/side",
            linked.to_str().unwrap(),
        ],
    );
    fs::canonicalize(linked).unwrap()
}
fn remove_checkout(f: &mut Fixture, path: &Path) -> Result<git::Snapshot, String> {
    f.act(Action::RemoveWorktree {
        path: path.to_str().unwrap().into(),
    })
}
#[test]
fn reviewed_worktree_removal_retains_branch_and_dirty_source_index() {
    let mut f = Fixture::new();
    let linked = managed_checkout(&mut f);
    let head = g(&linked, &["rev-parse", "HEAD"]);
    f.write("README.md", "staged source\n");
    g(&f.root, &["add", "README.md"]);
    f.write("README.md", "unstaged source\n");
    f.write("extra.txt", "source untracked\n");
    let index = g(&f.root, &["write-tree"]);
    let next = remove_checkout(&mut f, &linked).unwrap();
    assert!(!linked.exists());
    assert_eq!(next.worktrees.len(), 1);
    assert_eq!(g(&f.root, &["rev-parse", "refs/heads/work/side"]), head);
    assert_eq!(g(&f.root, &["write-tree"]), index);
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "unstaged source\n"
    );
    assert_eq!(
        fs::read_to_string(f.root.join("extra.txt")).unwrap(),
        "source untracked\n"
    );
}
#[test]
fn worktree_lock_unlock_and_stale_review_are_enforced() {
    let mut f = Fixture::new();
    let linked = managed_checkout(&mut f);
    let before = f.backend.snapshot(&f.id).unwrap();
    let lock = Action::WorktreeLock {
        path: linked.to_str().unwrap().into(),
        locked: true,
    };
    let next = f.act(lock).unwrap();
    assert!(
        next.worktrees
            .iter()
            .find(|t| fs::canonicalize(&t.path).ok().as_ref() == Some(&linked))
            .unwrap()
            .locked
    );
    assert!(remove_checkout(&mut f, &linked)
        .unwrap_err()
        .contains("Unlock"));
    assert!(f
        .backend
        .mutate(
            &f.id,
            &before.fingerprint,
            Action::RemoveWorktree {
                path: linked.to_str().unwrap().into()
            }
        )
        .unwrap_err()
        .contains("changed"));
    f.act(Action::WorktreeLock {
        path: linked.to_str().unwrap().into(),
        locked: false,
    })
    .unwrap();
    remove_checkout(&mut f, &linked).unwrap();
}
#[test]
fn worktree_removal_refuses_ignored_untracked_staged_and_hidden_index_work() {
    let mut f = Fixture::new();
    let linked = managed_checkout(&mut f);
    for (path, contents) in [
        ("new.txt", "untracked"),
        ("ignored.txt", "ignored"),
        ("README.md", "dirty tracked"),
    ] {
        if path == "ignored.txt" {
            fs::write(f.root.join(".git/info/exclude"), "ignored.txt\n").unwrap();
        }
        fs::write(linked.join(path), contents).unwrap();
        assert!(remove_checkout(&mut f, &linked).is_err());
        assert_eq!(fs::read_to_string(linked.join(path)).unwrap(), contents);
        if path == "README.md" {
            g(&linked, &["checkout", "--", "README.md"]);
        } else {
            fs::remove_file(linked.join(path)).unwrap();
        }
    }
    fs::write(linked.join("README.md"), "staged\n").unwrap();
    g(&linked, &["add", "README.md"]);
    let index = g(&linked, &["write-tree"]);
    assert!(remove_checkout(&mut f, &linked).is_err());
    assert_eq!(g(&linked, &["write-tree"]), index);
    g(&linked, &["reset", "--hard", "HEAD"]);
    for flag in ["--assume-unchanged", "--skip-worktree"] {
        g(&linked, &["update-index", flag, "README.md"]);
        fs::write(linked.join("README.md"), "hidden local work\n").unwrap();
        assert!(remove_checkout(&mut f, &linked).is_err());
        assert_eq!(
            fs::read_to_string(linked.join("README.md")).unwrap(),
            "hidden local work\n"
        );
        g(
            &linked,
            &["update-index", "--no-assume-unchanged", "README.md"],
        );
        g(
            &linked,
            &["update-index", "--no-skip-worktree", "README.md"],
        );
        g(&linked, &["checkout", "--", "README.md"]);
    }
    remove_checkout(&mut f, &linked).unwrap();
}
#[test]
fn worktree_removal_refuses_empty_extra_directories_and_nested_repositories() {
    let mut f = Fixture::new();
    let linked = managed_checkout(&mut f);
    fs::write(f.root.join(".git/info/exclude"), "ignored/\n").unwrap();
    fs::create_dir(linked.join("ignored")).unwrap();
    assert!(remove_checkout(&mut f, &linked).is_err());
    assert!(linked.join("ignored").is_dir());
    fs::remove_dir(linked.join("ignored")).unwrap();
    fs::create_dir(linked.join("nested")).unwrap();
    g(&linked.join("nested"), &["init", "-b", "main"]);
    assert!(remove_checkout(&mut f, &linked).is_err());
    assert!(linked.join("nested/.git").is_dir());
}
#[test]
fn worktree_management_refuses_primary_arbitrary_open_and_untrusted_targets() {
    let mut f = Fixture::new();
    let linked = managed_checkout(&mut f);
    let primary = f.root.clone();
    assert!(remove_checkout(&mut f, &primary).is_err());
    assert!(f
        .act(Action::WorktreeLock {
            path: primary.to_str().unwrap().into(),
            locked: true
        })
        .is_err());
    let outside = f._dir.path().join("outside");
    assert!(remove_checkout(&mut f, &outside).is_err());
    let opened = f
        .backend
        .open_worktree(&f.id, linked.to_str().unwrap())
        .unwrap();
    assert!(remove_checkout(&mut f, &linked)
        .unwrap_err()
        .contains("Close"));
    f.backend.close_repository(&opened.repo.id);
    assert!(f.backend.snapshot(&opened.repo.id).is_err());
    f.backend.close_repository(&f.id);
    let reopened = f.backend.register(&f.root).unwrap();
    f.id = reopened.repo.id;
    assert!(remove_checkout(&mut f, &linked)
        .unwrap_err()
        .contains("Trust"));
    f.backend.trust(&f.id).unwrap();
    remove_checkout(&mut f, &linked).unwrap();
}
#[test]
fn worktree_removal_refuses_detached_heads_and_in_progress_operations() {
    let mut f = Fixture::new();
    let linked = managed_checkout(&mut f);
    g(&linked, &["checkout", "--detach"]);
    assert!(remove_checkout(&mut f, &linked)
        .unwrap_err()
        .contains("branch"));
    g(&linked, &["switch", "work/side"]);
    let marker = Path::new(g(&linked, &["rev-parse", "--git-path", "CHERRY_PICK_HEAD"]).trim())
        .to_path_buf();
    fs::write(&marker, g(&linked, &["rev-parse", "HEAD"])).unwrap();
    assert!(remove_checkout(&mut f, &linked)
        .unwrap_err()
        .contains("active Git operation"));
    assert!(marker.exists());
    assert!(linked.exists());
    fs::remove_file(marker).unwrap();
    let lock =
        Path::new(g(&linked, &["rev-parse", "--git-path", "index.lock"]).trim()).to_path_buf();
    fs::write(&lock, "synthetic Git writer lock").unwrap();
    assert!(remove_checkout(&mut f, &linked)
        .unwrap_err()
        .contains("active Git operation"));
    assert_eq!(
        fs::read_to_string(lock).unwrap(),
        "synthetic Git writer lock"
    );
}
#[cfg(unix)]
#[test]
fn worktree_removal_never_follows_replaced_roots_or_untracked_symlinks() {
    use std::os::unix::fs::symlink;
    let mut f = Fixture::new();
    let linked = managed_checkout(&mut f);
    let outside = f._dir.path().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("keep"), "private sentinel").unwrap();
    symlink(&outside, linked.join("alias")).unwrap();
    assert!(remove_checkout(&mut f, &linked).is_err());
    fs::remove_file(linked.join("alias")).unwrap();
    let renamed = f._dir.path().join("original");
    fs::rename(&linked, &renamed).unwrap();
    symlink(&outside, &linked).unwrap();
    assert!(remove_checkout(&mut f, &linked).is_err());
    assert_eq!(
        fs::read_to_string(outside.join("keep")).unwrap(),
        "private sentinel"
    );
    assert!(renamed.join("README.md").is_file());
}
#[test]
fn worktree_removal_rechecks_ignored_files_created_after_review() {
    let mut f = Fixture::new();
    let linked = managed_checkout(&mut f);
    fs::write(f.root.join(".git/info/exclude"), "secret-cache\n").unwrap();
    let reviewed = f.backend.snapshot(&f.id).unwrap();
    fs::write(linked.join("secret-cache"), "do not delete\n").unwrap();
    let result = f.backend.mutate(
        &f.id,
        &reviewed.fingerprint,
        Action::RemoveWorktree {
            path: linked.to_str().unwrap().into(),
        },
    );
    assert!(result.unwrap_err().contains("ignored"));
    assert_eq!(
        fs::read_to_string(linked.join("secret-cache")).unwrap(),
        "do not delete\n"
    );
}
#[cfg(unix)]
#[test]
fn clean_tracked_symlink_removal_preserves_its_external_destination() {
    use std::os::unix::fs::symlink;
    let mut f = Fixture::new();
    let linked = managed_checkout(&mut f);
    let outside = f._dir.path().join("outside-sentinel");
    fs::write(&outside, "keep\n").unwrap();
    symlink(&outside, linked.join("tracked-link")).unwrap();
    g(&linked, &["add", "tracked-link"]);
    g(&linked, &["commit", "-m", "Track synthetic symlink"]);
    remove_checkout(&mut f, &linked).unwrap();
    assert_eq!(fs::read_to_string(outside).unwrap(), "keep\n");
}

#[test]
fn amend_rewrites_only_index_and_keeps_author_work_and_recovery() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    let before = f.backend.snapshot(&f.id).unwrap();
    let old = before.head.unwrap();
    let author = g(&f.root, &["show", "-s", "--format=%an <%ae>", "HEAD"]);
    assert!(f
        .backend
        .amend_message(&f.id, &old)
        .unwrap()
        .contains("Initial documentation"));
    f.write("README.md", "Staged replacement.\n");
    g(&f.root, &["add", "README.md"]);
    f.write("README.md", "Unstaged replacement.\n");
    f.write("private.txt", "Untracked sentinel.\n");
    let next = f
        .act(Action::Amend {
            head: old.clone(),
            message: "Amended synthetic commit\n\nDetails è".into(),
        })
        .unwrap();
    assert_ne!(next.head.as_deref(), Some(old.as_str()));
    let history = f.backend.history(&f.id, 0).unwrap();
    assert_eq!(
        history.len(),
        1,
        "Recovery refs must not clutter ordinary history"
    );
    assert_ne!(history[0].oid, old);
    assert_eq!(
        g(&f.root, &["show", "HEAD:README.md"]),
        "Staged replacement.\n"
    );
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "Unstaged replacement.\n"
    );
    assert_eq!(
        fs::read_to_string(f.root.join("private.txt")).unwrap(),
        "Untracked sentinel.\n"
    );
    assert_eq!(
        g(&f.root, &["show", "-s", "--format=%an <%ae>", "HEAD"]),
        author
    );
    assert_eq!(
        g(
            &f.root,
            &["for-each-ref", "--format=%(objectname)", "refs/ramo/amend/"]
        )
        .trim(),
        old
    );
    assert_eq!(g(&f.root, &["rev-list", "--count", "HEAD"]).trim(), "1");
}

#[test]
fn amend_rewords_without_staging_and_refuses_known_remote_commit() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    let old = f.backend.snapshot(&f.id).unwrap().head.unwrap();
    let tree = g(&f.root, &["rev-parse", "HEAD^{tree}"]);
    f.write("README.md", "Keep outside index.\n");
    f.act(Action::Amend {
        head: old,
        message: "Reword only".into(),
    })
    .unwrap();
    assert_eq!(g(&f.root, &["rev-parse", "HEAD^{tree}"]), tree);
    let head = f.backend.snapshot(&f.id).unwrap().head.unwrap();
    g(&f.root, &["update-ref", "refs/remotes/origin/other", &head]);
    assert!(f
        .act(Action::Amend {
            head: head.clone(),
            message: "Refused".into()
        })
        .unwrap_err()
        .contains("remote branch"));
    assert_eq!(g(&f.root, &["rev-parse", "HEAD"]).trim(), head);
}

#[test]
fn amend_refuses_stale_untrusted_detached_merge_and_active_operation() {
    let mut f = Fixture::new();
    f.seed();
    let old = f.backend.snapshot(&f.id).unwrap();
    let head = old.head.clone().unwrap();
    let action = || Action::Amend {
        head: head.clone(),
        message: "Not applied".into(),
    };
    assert!(f.act(action()).unwrap_err().contains("Trust"));
    f.backend.trust(&f.id).unwrap();
    let reviewed = f.backend.snapshot(&f.id).unwrap();
    f.write("extra.txt", "Changed since review");
    assert!(f
        .backend
        .mutate(&f.id, &reviewed.fingerprint, action())
        .unwrap_err()
        .contains("changed"));
    assert!(f
        .act(Action::Amend {
            head: "0".repeat(40),
            message: "No".into()
        })
        .is_err());
    g(&f.root, &["checkout", "--detach"]);
    assert!(f.act(action()).unwrap_err().contains("attached"));
    g(&f.root, &["switch", "main"]);
    g(&f.root, &["switch", "-c", "side"]);
    f.write("side.txt", "Side");
    g(&f.root, &["add", "side.txt"]);
    g(&f.root, &["commit", "-m", "Side"]);
    g(&f.root, &["switch", "main"]);
    g(&f.root, &["merge", "--no-ff", "side", "-m", "Merge"]);
    let merge = f.backend.snapshot(&f.id).unwrap().head.unwrap();
    assert!(f
        .act(Action::Amend {
            head: merge,
            message: "No".into()
        })
        .unwrap_err()
        .contains("merge commits"));
    let mut conflict = Fixture::new();
    conflict.conflicted_merge();
    let h = conflict
        .backend
        .snapshot(&conflict.id)
        .unwrap()
        .head
        .unwrap();
    assert!(conflict
        .act(Action::Amend {
            head: h,
            message: "No".into()
        })
        .is_err());
}

#[cfg(unix)]
#[test]
fn failed_amend_hook_preserves_index_head_message_and_recovery() {
    use std::os::unix::fs::PermissionsExt;
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    f.write("staged.txt", "Keep staged");
    g(&f.root, &["add", "staged.txt"]);
    let old = f.backend.snapshot(&f.id).unwrap().head.unwrap();
    let index = g(&f.root, &["write-tree"]);
    let hook = f.root.join(".git/hooks/commit-msg");
    fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(f
        .act(Action::Amend {
            head: old.clone(),
            message: "Keep my draft".into()
        })
        .is_err());
    assert_eq!(g(&f.root, &["rev-parse", "HEAD"]).trim(), old);
    assert_eq!(g(&f.root, &["write-tree"]), index);
    assert_eq!(
        g(
            &f.root,
            &["for-each-ref", "--format=%(objectname)", "refs/ramo/amend/"]
        )
        .trim(),
        old
    );
}

#[test]
fn repository_trust_lasts_while_open_and_resets_after_close_or_restart() {
    let mut f = Fixture::new();
    f.seed();
    assert!(!f.backend.snapshot(&f.id).unwrap().repo.trusted);
    f.backend.trust(&f.id).unwrap();
    assert!(f.backend.register(&f.root).unwrap().repo.trusted);
    g(&f.root, &["branch", "topic"]);
    f.act(Action::Switch {
        name: "topic".into(),
    })
    .unwrap();
    assert!(f.backend.snapshot(&f.id).unwrap().repo.trusted);
    f.act(Action::Switch {
        name: "main".into(),
    })
    .unwrap();
    assert!(f.backend.snapshot(&f.id).unwrap().repo.trusted);
    f.backend.close_repository(&f.id);
    assert!(!f.backend.register(&f.root).unwrap().repo.trusted);
    let mut restarted = Backend::new(f._dir.path().join("preferences.json"));
    assert!(!restarted.register(&f.root).unwrap().repo.trusted);
}

#[test]
fn direct_checkout_preserves_compatible_work_and_refuses_overwrites() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    g(&f.root, &["switch", "-c", "topic"]);
    f.write("README.md", "Committed on topic.\n");
    g(&f.root, &["add", "README.md"]);
    g(&f.root, &["commit", "-m", "Topic"]);
    g(&f.root, &["switch", "main"]);
    f.write("keep.txt", "Keep untracked.\n");
    f.act(Action::Switch {
        name: "topic".into(),
    })
    .unwrap();
    assert_eq!(
        fs::read_to_string(f.root.join("keep.txt")).unwrap(),
        "Keep untracked.\n"
    );
    f.write("README.md", "Unstaged work.\n");
    let head = g(&f.root, &["rev-parse", "HEAD"]);
    let index = g(&f.root, &["write-tree"]);
    assert!(f
        .act(Action::Switch {
            name: "main".into()
        })
        .is_err());
    assert_eq!(g(&f.root, &["rev-parse", "HEAD"]), head);
    assert_eq!(g(&f.root, &["write-tree"]), index);
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "Unstaged work.\n"
    );
}

#[test]
fn reviewed_remote_configuration_preserves_work_and_invalidates_old_reviews() {
    let mut f = Fixture::new();
    f.seed();
    f.write("README.md", "Keep unstaged\n");
    f.write("notes.txt", "Keep untracked\n");
    let action = || Action::ConfigureRemote {
        name: "origin".into(),
        url: "https://github.example.invalid/meadow/app.git".into(),
        existing: false,
    };
    assert!(f.act(action()).unwrap_err().contains("Trust"));
    f.backend.trust(&f.id).unwrap();
    let before = f.backend.snapshot(&f.id).unwrap();
    let index = g(&f.root, &["write-tree"]);
    let next = f.act(action()).unwrap();
    assert_ne!(next.fingerprint, before.fingerprint);
    assert_eq!(g(&f.root, &["write-tree"]), index);
    assert_eq!(
        fs::read_to_string(f.root.join("README.md")).unwrap(),
        "Keep unstaged\n"
    );
    let rows = f.backend.remote_list(&f.id).unwrap();
    assert_eq!(rows.len(), 1);
    assert!(rows[0].editable);
    let reviewed = f.backend.snapshot(&f.id).unwrap();
    g(
        &f.root,
        &[
            "remote",
            "set-url",
            "origin",
            "git@github.example.invalid:meadow/changed.git",
        ],
    );
    assert!(f
        .backend
        .mutate(
            &f.id,
            &reviewed.fingerprint,
            Action::ConfigureRemote {
                name: "origin".into(),
                url: "https://github.example.invalid/meadow/new.git".into(),
                existing: true
            }
        )
        .unwrap_err()
        .contains("changed"));
    f.act(Action::ConfigureRemote {
        name: "origin".into(),
        url: "https://github.example.invalid/meadow/new.git".into(),
        existing: true,
    })
    .unwrap();
    assert_eq!(
        g(&f.root, &["remote", "get-url", "origin"]).trim(),
        "https://github.example.invalid/meadow/new.git"
    );
}
#[test]
fn remote_management_refuses_unsafe_urls_and_preserves_special_push_settings() {
    let mut f = Fixture::new();
    f.seed();
    f.backend.trust(&f.id).unwrap();
    for (name, url) in [
        ("--help", "https://host.invalid/a/b"),
        ("origin", "https://user:secret@host.invalid/a/b"),
        ("origin", "ext::program"),
        ("origin", "file:///tmp/repo"),
    ] {
        assert!(f
            .act(Action::ConfigureRemote {
                name: name.into(),
                url: url.into(),
                existing: false
            })
            .is_err());
    }
    g(
        &f.root,
        &[
            "remote",
            "add",
            "origin",
            "https://host.invalid/meadow/app.git",
        ],
    );
    g(
        &f.root,
        &[
            "config",
            "remote.origin.pushurl",
            "https://host.invalid/meadow/app.git",
        ],
    );
    assert!(!f.backend.remote_list(&f.id).unwrap()[0].editable);
    assert!(f
        .act(Action::ConfigureRemote {
            name: "origin".into(),
            url: "https://other.invalid/meadow/app.git".into(),
            existing: true
        })
        .is_err());
    assert_eq!(
        g(&f.root, &["remote", "get-url", "origin"]).trim(),
        "https://host.invalid/meadow/app.git"
    );
    g(
        &f.root,
        &[
            "config",
            "remote.origin.url",
            "https://user:secret@host.invalid/meadow/app.git",
        ],
    );
    let rows = f.backend.remote_list(&f.id).unwrap();
    assert!(!rows[0].fetch[0].contains("secret"));
}

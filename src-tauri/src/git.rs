//! Typed Git operations. All writes require an opened, explicitly trusted context and a fresh snapshot.
use crate::process::{self, strings};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
    time::Duration,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    pub id: String,
    pub path: String,
    pub name: String,
    pub trusted: bool,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileChange {
    pub path: String,
    pub original: Option<String>,
    pub index: String,
    pub worktree: String,
}
/// Shared partial-staging availability for native renderers; mutation revalidates it.
pub fn partial_staging_available(diff: &str) -> Result<(), String> {
    crate::patch::select_hunk(diff, 0).map(|_| ())
}
pub fn is_conflicted(file: &FileChange) -> bool {
    file.index == "U"
        || file.worktree == "U"
        || (file.index == "A" && file.worktree == "A")
        || (file.index == "D" && file.worktree == "D")
}
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Branch {
    pub name: String,
    pub oid: String,
    pub remote: bool,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Worktree {
    pub path: String,
    pub branch: String,
    pub locked: bool,
    pub primary: bool,
    pub head: String,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub oid: String,
    pub parents: Vec<String>,
    pub author: String,
    pub date: String,
    pub subject: String,
    pub refs: String,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub repo: Repository,
    pub branch: String,
    pub head: Option<String>,
    pub upstream: Option<String>,
    pub ahead: Option<u64>,
    pub behind: Option<u64>,
    pub files: Vec<FileChange>,
    pub branches: Vec<Branch>,
    pub worktrees: Vec<Worktree>,
    pub stashes: Vec<crate::stash::Stash>,
    pub fingerprint: String,
    pub operation: Option<String>,
}
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Preferences {
    pub recent: Vec<String>,
}

pub struct Backend {
    repositories: HashMap<String, Repository>,
    pub preferences: Preferences,
    preferences_path: PathBuf,
    clone_grants: crate::cloning::Grants,
    worktree_parents: HashMap<String, (String, PathBuf)>,
}
pub fn hash(bytes: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(bytes.as_ref()))
}
fn text(bytes: Vec<u8>) -> Result<String, String> {
    String::from_utf8(bytes).map_err(|_| "This repository contains non-UTF-8 paths or metadata, which this version cannot safely display.".into())
}
pub(crate) fn valid_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || Path::new(path).is_absolute()
        || Path::new(path)
            .components()
            .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err("A repository-relative file path is required.".into());
    }
    Ok(())
}
fn oid(value: &str) -> Result<(), String> {
    if !matches!(value.len(), 40 | 64) || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Invalid commit ID".into());
    }
    Ok(())
}
pub fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    git_input(root, args, None)
}
pub(crate) fn git_input(
    root: &Path,
    args: &[&str],
    input: Option<Vec<u8>>,
) -> Result<Vec<u8>, String> {
    git_output(root, args, input, false)
}
fn git_digest(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    git_output(root, args, None, true)
}
fn git_output(
    root: &Path,
    args: &[&str],
    input: Option<Vec<u8>>,
    digest: bool,
) -> Result<Vec<u8>, String> {
    let mut all = strings(&[
        "--no-pager",
        // Stash endpoints take no pathspecs. Apple Git 2.54 leaves saved untracked
        // files behind when its internal clean inherits literal pathspec mode.
        if args.first() == Some(&"stash") {
            "--no-literal-pathspecs"
        } else {
            "--literal-pathspecs"
        },
        "-c",
        "core.fsmonitor=false",
        "-c",
        "core.quotePath=false",
        "-c",
        "color.ui=false",
    ]);
    all.extend(args.iter().map(|s| s.to_string()));
    let seconds = if [
        "fetch",
        "pull",
        "push",
        "commit",
        "clone",
        "stash",
        "merge",
        "worktree",
        "cherry-pick",
        "revert",
    ]
    .iter()
    .any(|c| args.first() == Some(c))
    {
        120
    } else {
        20
    };
    if digest {
        return process::run_digest(
            &process::executable("git")?,
            root,
            &all,
            Duration::from_secs(seconds),
        );
    }
    process::run(
        &process::executable("git")?,
        root,
        &all,
        input,
        Duration::from_secs(seconds),
    )
}
pub fn diff(root: &Path, args: &[&str]) -> Result<String, String> {
    let mut all = vec!["diff", "--no-ext-diff", "--no-textconv", "--no-color"];
    all.extend(args);
    text(git(root, &all)?)
}
fn working_diff(root: &Path, path: &str, staged: bool) -> Result<String, String> {
    let mut args = vec![
        "--unified=3",
        "--inter-hunk-context=0",
        "--diff-algorithm=myers",
        "--no-renames",
        "--no-relative",
        "--src-prefix=a/",
        "--dst-prefix=b/",
        "--line-prefix=",
        "--output-indicator-new=+",
        "--output-indicator-old=-",
        "--output-indicator-context= ",
    ];
    if staged {
        args.push("--cached");
    }
    args.extend(["--", path]);
    diff(root, &args)
}
impl Backend {
    pub fn new(preferences_path: PathBuf) -> Self {
        let preferences = fs::read(&preferences_path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        Self {
            repositories: HashMap::new(),
            preferences,
            preferences_path,
            worktree_parents: HashMap::new(),
            clone_grants: crate::cloning::Grants::default(),
        }
    }
    /// Only call after the native directory picker, or from synthetic fixture tests.
    pub fn authorize_worktree_parent(
        &mut self,
        id: &str,
        parent: &Path,
    ) -> Result<WorktreeParent, String> {
        self.repository(id)?;
        let parent = fs::canonicalize(parent).map_err(|e| e.to_string())?;
        if !parent.is_dir() {
            return Err("Choose an existing parent folder.".into());
        }
        if git(&parent, &["rev-parse", "--absolute-git-dir"]).is_ok() {
            return Err("Choose a parent folder outside existing Git worktrees.".into());
        }
        let path = parent
            .to_str()
            .ok_or("Parent path must be valid UTF-8")?
            .to_owned();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let token = hash(format!("{id}:{path}:{now}"));
        if self.worktree_parents.len() >= 8 {
            self.worktree_parents.clear();
        }
        self.worktree_parents
            .insert(token.clone(), (id.into(), parent));
        Ok(WorktreeParent { id: token, path })
    }
    pub fn authorize_clone_parent(&mut self, path: &Path) -> Result<crate::cloning::Grant, String> {
        self.clone_grants.authorize_parent(path)
    }
    pub fn authorize_clone_source(&mut self, path: &Path) -> Result<crate::cloning::Grant, String> {
        self.clone_grants.authorize_source(path)
    }
    pub fn prepare_clone(
        &mut self,
        parent_id: &str,
        directory: &str,
        source: crate::cloning::Source,
    ) -> Result<crate::cloning::Plan, String> {
        self.clone_grants.prepare(parent_id, directory, source)
    }
    pub fn clone_repository(&mut self, plan_id: &str) -> Result<Snapshot, String> {
        let path = self.clone_grants.execute(plan_id)?;
        self.register(&path).map_err(|e|format!("Clone was created at {}, but could not open automatically. Open it manually after inspection.\n{e}",path.display()))
    }
    pub fn register(&mut self, path: &Path) -> Result<Snapshot, String> {
        let path = fs::canonicalize(path).map_err(|e| format!("Cannot open folder: {e}"))?;
        let root = text(git(&path, &["rev-parse", "--show-toplevel"])?)?;
        let root =
            fs::canonicalize(root.trim_end_matches(['\r', '\n'])).map_err(|e| e.to_string())?;
        let root_text = root
            .to_str()
            .ok_or("Repository path must be valid UTF-8")?
            .to_owned();
        let id = hash(root_text.as_bytes());
        let name = root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        self.repositories.entry(id.clone()).or_insert(Repository {
            id: id.clone(),
            path: root_text.clone(),
            name,
            trusted: false,
        });
        self.preferences.recent.retain(|p| p != &root_text);
        self.preferences.recent.insert(0, root_text);
        self.preferences.recent.truncate(20);
        if let Some(parent) = self.preferences_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let temporary = self.preferences_path.with_extension("tmp");
        fs::write(
            &temporary,
            serde_json::to_vec_pretty(&self.preferences).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        #[cfg(windows)]
        if self.preferences_path.exists() {
            fs::remove_file(&self.preferences_path).map_err(|e| e.to_string())?;
        }
        fs::rename(temporary, &self.preferences_path).map_err(|e| e.to_string())?;
        self.snapshot(&id)
    }
    pub fn recent(&mut self, path: &str) -> Result<Snapshot, String> {
        if !self.preferences.recent.iter().any(|p| p == path) {
            return Err("Choose this folder with Open repository first.".into());
        }
        self.register(Path::new(path))
    }
    pub fn repository(&self, id: &str) -> Result<&Repository, String> {
        self.repositories
            .get(id)
            .ok_or_else(|| "Repository is not open in this session".into())
    }
    pub fn trust(&mut self, id: &str) -> Result<Snapshot, String> {
        self.repositories
            .get_mut(id)
            .ok_or("Repository is not open")?
            .trusted = true;
        self.snapshot(id)
    }
    pub fn open_worktree(&mut self, id: &str, path: &str) -> Result<Snapshot, String> {
        let snapshot = self.snapshot(id)?;
        let target = fs::canonicalize(path).map_err(|e| e.to_string())?;
        if !snapshot
            .worktrees
            .iter()
            .any(|w| fs::canonicalize(&w.path).ok().as_ref() == Some(&target))
        {
            return Err("Worktree is not part of this repository".into());
        }
        self.register(&target)
    }
    pub fn close_repository(&mut self, id: &str) {
        self.repositories.remove(id);
        self.worktree_parents.retain(|_, (owner, _)| owner != id);
    }
    pub fn snapshot(&self, id: &str) -> Result<Snapshot, String> {
        let repo = self.repository(id)?.clone();
        let root = Path::new(&repo.path);
        let status = git(
            root,
            &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        )?;
        let files = parse_status(&status)?;
        let head = git(root, &["rev-parse", "--verify", "HEAD"])
            .ok()
            .and_then(|b| text(b).ok())
            .map(|s| s.trim().to_string());
        let branch = git(root, &["symbolic-ref", "--short", "HEAD"])
            .ok()
            .and_then(|b| text(b).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "Detached HEAD".into());
        let upstream = git(
            root,
            &[
                "rev-parse",
                "--abbrev-ref",
                "--symbolic-full-name",
                "@{upstream}",
            ],
        )
        .ok()
        .and_then(|b| text(b).ok())
        .map(|s| s.trim().to_owned());
        let counts = git(
            root,
            &["rev-list", "--left-right", "--count", "HEAD...@{upstream}"],
        )
        .ok()
        .and_then(|b| text(b).ok())
        .and_then(|s| {
            let v: Vec<_> = s
                .split_whitespace()
                .filter_map(|s| s.parse::<u64>().ok())
                .collect();
            if v.len() == 2 {
                Some((v[0], v[1]))
            } else {
                None
            }
        });
        let refs = git(
            root,
            &[
                "for-each-ref",
                "--format=%(refname)%00%(objectname)",
                "refs/heads/",
                "refs/remotes/",
            ],
        )?;
        let branches = text(refs.clone())?
            .lines()
            .filter_map(|s| {
                let (name, oid) = s.split_once('\0')?;
                let remote = name.starts_with("refs/remotes/");
                Some(Branch {
                    name: name
                        .trim_start_matches("refs/heads/")
                        .trim_start_matches("refs/remotes/")
                        .into(),
                    oid: oid.into(),
                    remote,
                })
            })
            .collect();
        let (stashes, stash_bytes) = crate::stash::list(root)?;
        let worktrees = worktrees(root)?;
        let unstaged = git_digest(
            root,
            &["diff", "--no-ext-diff", "--no-textconv", "--binary"],
        )?;
        let staged = git_digest(
            root,
            &[
                "diff",
                "--cached",
                "--no-ext-diff",
                "--no-textconv",
                "--binary",
            ],
        )?;
        let mut state = Sha256::new();
        // Endpoint changes invalidate open reviews even if refs/upstream are identical.
        let remote_config = git(root, &["remote", "-v"])?;
        state.update((remote_config.len() as u64).to_le_bytes());
        state.update(remote_config);
        let worktree_state = serde_json::to_vec(&worktrees).map_err(|e| e.to_string())?;
        state.update((worktree_state.len() as u64).to_le_bytes());
        state.update(worktree_state);
        for part in [&status, &refs, &stash_bytes, &unstaged, &staged] {
            state.update((part.len() as u64).to_le_bytes());
            state.update(part);
        }
        // HEAD's object ID alone cannot distinguish two branches at the same
        // commit, or a tracking-target change. Both affect reviewed actions.
        for part in [
            branch.as_str(),
            head.as_deref().unwrap_or("unborn"),
            upstream.as_deref().unwrap_or(""),
        ] {
            state.update((part.len() as u64).to_le_bytes());
            state.update(part.as_bytes());
        }
        // Include untracked file metadata: staging is also guarded against a changed file set.
        for file in files.iter().filter(|f| f.index == "?") {
            if let Ok(meta) = fs::symlink_metadata(root.join(&file.path)) {
                state.update(meta.len().to_le_bytes());
                state.update(format!("{:?}", meta.modified()).as_bytes());
            }
        }
        let gitdir = text(git(root, &["rev-parse", "--absolute-git-dir"])?)?;
        let dir = Path::new(gitdir.trim());
        let operation: Option<String> = if dir.join("MERGE_HEAD").exists() {
            Some("Merge in progress".into())
        } else if dir.join("rebase-merge").exists() || dir.join("rebase-apply").exists() {
            Some("Rebase in progress".into())
        } else if dir.join("CHERRY_PICK_HEAD").exists() {
            Some("Cherry-pick in progress".into())
        } else if dir.join("REVERT_HEAD").exists() {
            Some("Revert in progress".into())
        } else if dir.join("sequencer").is_dir() {
            Some(
                match sequence_kind(dir)? {
                    Some(SequenceKind::CherryPick) => "Cherry-pick in progress",
                    Some(SequenceKind::Revert) => "Revert in progress",
                    None => "Sequence in progress",
                }
                .into(),
            )
        } else {
            None
        };
        state.update(operation.as_deref().unwrap_or("idle").as_bytes());
        for name in [
            "MERGE_HEAD",
            "CHERRY_PICK_HEAD",
            "REVERT_HEAD",
            "sequencer/todo",
            "sequencer/head",
            "sequencer/abort-safety",
            "sequencer/opts",
        ] {
            match fs::File::open(dir.join(name)) {
                Ok(file) => {
                    let mut bytes = Vec::new();
                    file.take(16385)
                        .read_to_end(&mut bytes)
                        .map_err(|e| e.to_string())?;
                    if bytes.len() > 16384 {
                        return Err("Git operation metadata is too large to review safely".into());
                    }
                    state.update(name.as_bytes());
                    state.update((bytes.len() as u64).to_le_bytes());
                    state.update(bytes);
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(format!("Cannot review Git operation: {e}")),
            }
        }
        Ok(Snapshot {
            repo,
            branch,
            head,
            upstream,
            ahead: counts.map(|c| c.0),
            behind: counts.map(|c| c.1),
            files,
            branches,
            worktrees,
            stashes,
            fingerprint: format!("{:x}", state.finalize()),
            operation,
        })
    }
    pub fn history(&self, id: &str, skip: usize) -> Result<Vec<Commit>, String> {
        let root = Path::new(&self.repository(id)?.path);
        if git(root, &["rev-parse", "--verify", "HEAD"]).is_err() {
            return Ok(vec![]);
        }
        let skip = format!("--skip={}", skip.min(100000));
        let raw = text(git(
            root,
            &[
                "log",
                "--exclude=refs/ramo/*",
                "--all",
                "HEAD",
                "--topo-order",
                "--max-count=300",
                &skip,
                "--format=%H%x00%P%x00%an%x00%aI%x00%s%x00%D%x00",
            ],
        )?)?;
        let fields: Vec<_> = raw.split('\0').collect();
        Ok(fields
            .chunks(6)
            .filter_map(|c| {
                if c.len() == 6 {
                    Some(Commit {
                        oid: c[0].trim().into(),
                        parents: c[1].split_whitespace().map(str::to_string).collect(),
                        author: c[2].into(),
                        date: c[3].into(),
                        subject: c[4].into(),
                        refs: c[5].into(),
                    })
                } else {
                    None
                }
            })
            .collect())
    }
    pub fn compare_revisions(
        &self,
        id: &str,
        from: &str,
        to: &str,
        kind: crate::compare::Kind,
    ) -> Result<crate::compare::Comparison, String> {
        crate::compare::details(Path::new(&self.repository(id)?.path), from, to, kind)
    }
    pub fn comparison_diff(
        &self,
        id: &str,
        from: &str,
        to: &str,
        kind: crate::compare::Kind,
        path: &str,
    ) -> Result<String, String> {
        valid_path(path)?;
        crate::compare::file_diff(Path::new(&self.repository(id)?.path), from, to, kind, path)
    }
    pub fn stash_details(
        &self,
        id: &str,
        index: usize,
        oid: &str,
    ) -> Result<crate::stash::Details, String> {
        crate::stash::details(Path::new(&self.repository(id)?.path), index, oid)
    }
    pub fn stash_diff(
        &self,
        id: &str,
        index: usize,
        oid: &str,
        path: &str,
        untracked: bool,
    ) -> Result<String, String> {
        valid_path(path)?;
        crate::stash::file_diff(
            Path::new(&self.repository(id)?.path),
            index,
            oid,
            path,
            untracked,
        )
    }
    pub fn commit_files(&self, id: &str, commit: &str) -> Result<Vec<FileChange>, String> {
        oid(commit)?;
        let root = Path::new(&self.repository(id)?.path);
        let parents = text(git(root, &["rev-list", "--parents", "-n", "1", commit])?)?;
        let mut args = vec!["diff-tree", "--no-commit-id", "--name-status", "-r", "-z"];
        if let Some(parent) = parents.split_whitespace().nth(1) {
            // Compare the exact same first parent used by file_diff. Older Git
            // versions can list every parent when -m and --first-parent combine.
            args.extend([parent, commit]);
        } else {
            args.extend(["--root", commit]);
        }
        let raw = git(root, &args)?;
        parse_names(&raw)
    }
    pub fn file_diff(
        &self,
        id: &str,
        path: &str,
        commit: Option<&str>,
        staged: bool,
    ) -> Result<String, String> {
        valid_path(path)?;
        let root = Path::new(&self.repository(id)?.path);
        if let Some(commit) = commit {
            oid(commit)?;
            let parents = text(git(root, &["rev-list", "--parents", "-n", "1", commit])?)?;
            let parent = parents.split_whitespace().nth(1);
            return if let Some(parent) = parent {
                diff(root, &[parent, commit, "--", path])
            } else {
                text(git(
                    root,
                    &[
                        "show",
                        "--format=",
                        "--no-ext-diff",
                        "--no-textconv",
                        "--root",
                        commit,
                        "--",
                        path,
                    ],
                )?)
            };
        }
        let files = parse_status(&git(
            root,
            &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        )?)?;
        if !files.iter().any(|f| f.path == path) {
            return Err("This file is no longer changed. Refresh the repository.".into());
        }
        if !staged && files.iter().any(|f| f.path == path && f.index == "?") {
            let target = root.join(path);
            let metadata = fs::symlink_metadata(&target).map_err(|e| e.to_string())?;
            if metadata.file_type().is_symlink() {
                return Ok(format!(
                    "Symbolic link → {}",
                    fs::read_link(target).map_err(|e| e.to_string())?.display()
                ));
            }
            if !metadata.is_file() || metadata.len() > 512 * 1024 {
                return Err("Untracked preview is limited to regular files up to 512 KiB.".into());
            }
            let canonical = fs::canonicalize(&target).map_err(|e| e.to_string())?;
            if !canonical.starts_with(root) {
                return Err("File is outside this worktree".into());
            }
            let bytes = fs::read(canonical).map_err(|e| e.to_string())?;
            if bytes.contains(&0) {
                return Ok("Binary file — text preview unavailable".into());
            }
            return text(bytes).map(|s| {
                s.lines()
                    .map(|s| format!("+{s}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            });
        }
        working_diff(root, path, staged)
    }

    pub fn conflict_file(
        &self,
        id: &str,
        expected: &str,
        path: &str,
    ) -> Result<ConflictFile, String> {
        let snapshot = self.snapshot(id)?;
        if snapshot.fingerprint != expected {
            return Err("Repository changed. Refresh before resolving.".into());
        }
        valid_path(path)?;
        if !snapshot
            .files
            .iter()
            .any(|f| f.path == path && is_conflicted(f))
        {
            return Err("Choose an unresolved file.".into());
        }
        let root = Path::new(&snapshot.repo.path);
        let modes = text(git(root, &["ls-files", "--stage", "--", path])?)?;
        if modes
            .lines()
            .any(|l| l.starts_with("120000") || l.starts_with("160000"))
        {
            return Err("Symbolic link/submodule conflicts require an external editor.".into());
        }
        let stage = |number| -> Result<Option<String>, String> {
            let spec = format!(":{number}:{path}");
            if git(root, &["cat-file", "-e", &spec]).is_err() {
                return Ok(None);
            }
            let size = text(git(root, &["cat-file", "-s", &spec])?)?
                .trim()
                .parse::<u64>()
                .map_err(|e| e.to_string())?;
            if size > 1_048_576 {
                return Err("Guided editor supports text files up to 1 MiB.".into());
            }
            let bytes = git(root, &["show", &spec])?;
            if bytes.contains(&0) {
                return Err("Binary conflicts require an external editor.".into());
            }
            text(bytes).map(Some)
        };
        let file = root.join(path);
        let working = if file.symlink_metadata().is_ok() {
            regular_resolution_path(root, path)?;
            let metadata = fs::metadata(&file).map_err(|e| e.to_string())?;
            if metadata.len() > 1_048_576 {
                return Err("Guided editor supports text files up to 1 MiB.".into());
            }
            let bytes = fs::read(file).map_err(|e| e.to_string())?;
            if bytes.contains(&0) {
                return Err("Binary conflicts require an external editor.".into());
            }
            Some(text(bytes)?)
        } else {
            None
        };
        Ok(ConflictFile {
            base: stage(1)?,
            ours: stage(2)?,
            theirs: stage(3)?,
            working,
        })
    }
    pub fn remote_list(&self, id: &str) -> Result<Vec<crate::remotes::Remote>, String> {
        crate::remotes::list(Path::new(&self.repository(id)?.path))
    }
    pub fn amend_message(&self, id: &str, head: &str) -> Result<String, String> {
        crate::amend::message(&self.snapshot(id)?, head)
    }
    pub fn mutate(&mut self, id: &str, expected: &str, action: Action) -> Result<Snapshot, String> {
        let snapshot = self.snapshot(id)?;
        if !snapshot.repo.trusted {
            return Err("Trust this repository before running write or network operations. Git hooks, filters and credential helpers can execute local programs.".into());
        }
        if snapshot.fingerprint != expected {
            return Err(
                "Repository changed since your last review. Refresh and review it before retrying."
                    .into(),
            );
        }
        let conflicts = snapshot.files.iter().any(is_conflicted);
        if (snapshot.operation.is_some() || conflicts)
            && !matches!(
                &action,
                Action::Stage { .. }
                    | Action::Unstage { .. }
                    | Action::MarkResolved { .. }
                    | Action::Commit { .. }
                    | Action::ResolveFile { .. }
                    | Action::AbortMerge
                    | Action::Sequence { .. }
                    | Action::Fetch
            )
            && !(snapshot.operation.is_none() && matches!(&action, Action::RestoreStashFile { .. }))
        {
            return Err("Complete the current Git operation and resolve conflicts before switching branches, stashing or synchronizing.".into());
        }
        let root = Path::new(&snapshot.repo.path);
        let stage = matches!(&action, Action::Stage { .. });
        let line_rows = match &action {
            Action::Lines { rows, .. } => Some(rows.clone()),
            _ => None,
        };
        let hunk_index = match &action {
            Action::Hunk { index, .. } => *index,
            _ => 0,
        };
        let action_kind = match &action {
            Action::CheckoutCommit { .. } => "checkout",
            Action::CherryPick { .. } => "cherry-pick",
            Action::Revert { .. } => "revert",
            _ => "",
        }
        .to_owned();
        match action {
            Action::Hunk {
                path,
                index: _,
                unstage,
                patch_hash,
            }
            | Action::Lines {
                path,
                unstage,
                patch_hash,
                ..
            } => {
                valid_path(&path)?;
                let file = snapshot
                    .files
                    .iter()
                    .find(|f| f.path == path)
                    .ok_or("File is not in the reviewed change list")?;
                if is_conflicted(file)
                    || file.original.is_some()
                    || if unstage {
                        file.index != "M"
                    } else {
                        file.worktree != "M" || file.index == "?"
                    }
                {
                    return Err("Partial staging requires an ordinary modified tracked file. Use whole-file staging for this change.".into());
                }
                let modes = git(root, &["ls-files", "--stage", "-z", "--", &path])?;
                if !(modes.starts_with(b"100644 ") || modes.starts_with(b"100755 ")) {
                    return Err("Partial staging requires a regular tracked file.".into());
                }
                let complete = self.file_diff(id, &path, None, unstage)?;
                if hash(complete.as_bytes()) != patch_hash {
                    return Err(
                        "The displayed diff changed. Refresh and review the selection before retrying."
                            .into(),
                    );
                }
                let patch = if let Some(rows) = &line_rows {
                    crate::patch::select_lines(&complete, rows, unstage)?
                } else {
                    crate::patch::select_hunk(&complete, hunk_index)?
                };
                let mut args = vec!["apply", "--cached", "--whitespace=nowarn"];
                if unstage && line_rows.is_none() {
                    args.push("--reverse");
                }
                args.push("--");
                git_input(root, &args, Some(patch.into_bytes()))?;
            }
            Action::Stage { paths } | Action::Unstage { paths } => {
                if paths.is_empty() {
                    return Err("Select at least one file".into());
                }
                if paths.iter().any(|path| {
                    snapshot
                        .files
                        .iter()
                        .any(|file| &file.path == path && is_conflicted(file))
                }) {
                    return Err("Unmerged files need explicit resolution. Edit the file, then use Mark resolved.".into());
                }
                let mut args: Vec<&str> = if stage {
                    vec!["add", "--"]
                } else if snapshot.head.is_some() {
                    vec!["reset", "-q", "HEAD", "--"]
                } else {
                    vec!["update-index", "--force-remove", "--"]
                };
                let mut expanded = paths.clone();
                for path in &paths {
                    valid_path(path)?;
                    if !snapshot.files.iter().any(|f| &f.path == path) {
                        return Err("File is not in the reviewed change list".into());
                    }
                    if let Some(file) = snapshot.files.iter().find(|f| &f.path == path) {
                        if !stage || file.worktree == "R" {
                            if let Some(original) = &file.original {
                                valid_path(original)?;
                                if !expanded.contains(original) {
                                    expanded.push(original.clone());
                                }
                            }
                        }
                    }
                }
                args.extend(expanded.iter().map(String::as_str));
                git(root, &args)?;
            }
            Action::Amend { head, message } => {
                crate::amend::apply(&snapshot, &head, message)?;
            }
            Action::Commit { message } => {
                if message.trim().is_empty() || message.len() > 32768 {
                    return Err("Enter a commit message (maximum 32 KiB).".into());
                }
                if conflicts {
                    return Err("Resolve unmerged files before committing.".into());
                }
                let merge = snapshot.operation.as_deref() == Some("Merge in progress");
                if snapshot.operation.is_some() && !merge {
                    return Err("Use the operation bar to continue cherry-pick/revert, or Git's continue command outside Ramo for rebase. An ordinary commit cannot continue this sequence.".into());
                }
                if !merge
                    && !snapshot
                        .files
                        .iter()
                        .any(|f| f.index != " " && f.index != "?")
                {
                    return Err("Stage at least one file before committing.".into());
                }
                git_input(root, &["commit", "-F", "-"], Some(message.into_bytes()))?;
            }
            Action::Sequence { operation, step } => {
                let expected_operation = operation.label();
                if snapshot.operation.as_deref() != Some(expected_operation) {
                    return Err("This is not the reviewed cherry-pick/revert operation. Refresh before continuing.".into());
                }
                let gitdir = text(git(root, &["rev-parse", "--absolute-git-dir"])?)?;
                if let Some(todo_kind) = sequence_kind(Path::new(gitdir.trim()))? {
                    if todo_kind != operation {
                        return Err(
                            "Mixed or mismatched sequence. Continue it with Git outside Ramo."
                                .into(),
                        );
                    }
                }
                if step == SequenceStep::Continue {
                    if conflicts {
                        return Err("Resolve and stage every conflict before continuing.".into());
                    }
                    if snapshot
                        .files
                        .iter()
                        .any(|f| f.index != "?" && f.worktree != " ")
                    {
                        return Err("Review and stage or undo remaining tracked worktree edits before continuing. Files were left unchanged.".into());
                    }
                    git(root, &[operation.command(), "--continue", "--no-edit"])?;
                } else {
                    git(
                        root,
                        &[
                            operation.command(),
                            if step == SequenceStep::Abort {
                                "--abort"
                            } else {
                                "--skip"
                            },
                        ],
                    )?;
                }
            }
            Action::AbortMerge => {
                if snapshot.operation.as_deref() != Some("Merge in progress") {
                    return Err("There is no merge to abort.".into());
                }
                git(root, &["merge", "--abort"])?;
            }
            Action::ResolveFile { path, content } => {
                // Reuse the bounded/type checks and conflict membership check before writing.
                self.conflict_file(id, expected, &path)?;
                let destination = regular_resolution_path(root, &path)?;
                if let Some(content) = content {
                    if content.len() > 1_048_576 || content.contains('\0') {
                        return Err("Resolution must be UTF-8 text up to 1 MiB.".into());
                    }
                    if content.lines().any(|l| {
                        l.starts_with("<<<<<<<")
                            || l.starts_with("=======")
                            || l.starts_with(">>>>>>>")
                            || l.starts_with("|||||||")
                    }) {
                        return Err("Resolve all conflict markers before saving.".into());
                    }
                    use std::io::Write;
                    let mut temporary = tempfile::NamedTempFile::new_in(
                        destination.parent().ok_or("Missing parent")?,
                    )
                    .map_err(|e| e.to_string())?;
                    if let Ok(metadata) = fs::metadata(&destination) {
                        temporary
                            .as_file()
                            .set_permissions(metadata.permissions())
                            .map_err(|e| e.to_string())?;
                    }
                    temporary
                        .write_all(content.as_bytes())
                        .map_err(|e| e.to_string())?;
                    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
                    temporary.persist(&destination).map_err(|e| e.to_string())?;
                } else if destination.exists() {
                    fs::remove_file(destination).map_err(|e| e.to_string())?;
                }
                git(root, &["add", "--", &path])?;
            }
            Action::MarkResolved { path } => {
                valid_path(&path)?;
                if !snapshot
                    .files
                    .iter()
                    .any(|file| file.path == path && is_conflicted(file))
                {
                    return Err("This file is no longer an unresolved conflict. Refresh and review it again.".into());
                }
                git(root, &["add", "--", &path])?;
            }
            Action::ConfigureRemote {
                name,
                url,
                existing,
            } => {
                crate::remotes::configure(root, &name, &url, existing)?;
            }
            Action::Switch { name } => {
                validate_branch(root, &name)?;
                git(root, &["switch", "--", &name])?;
            }
            Action::WorktreeLock { path, locked } => {
                let target = crate::worktree_management::target(&snapshot, &path)?;
                if target.locked == locked {
                    return Err("Worktree lock state changed. Refresh and review again.".into());
                }
                git(
                    root,
                    &[
                        "worktree",
                        if locked { "lock" } else { "unlock" },
                        "--",
                        &crate::native_paths::git_path(Path::new(&target.path))?,
                    ],
                )?;
            }
            Action::RemoveWorktree { path } => {
                let target = crate::worktree_management::target(&snapshot, &path)?;
                if self.repositories.values().any(|repo| {
                    crate::native_paths::same_path(Path::new(&repo.path), Path::new(&target.path))
                }) {
                    return Err("Close this worktree's repository tab before removing it.".into());
                }
                crate::worktree_management::remove(root, target)?;
            }
            Action::CreateWorktree {
                selection_id,
                directory,
                branch,
                create_branch,
            } => {
                let (owner, parent) = self
                    .worktree_parents
                    .get(&selection_id)
                    .cloned()
                    .ok_or("Choose the parent folder with the native picker again.")?;
                if owner != id {
                    return Err("This folder selection belongs to another repository.".into());
                }
                validate_worktree_directory(&directory)?;
                validate_branch(root, &branch)?;
                let head = snapshot
                    .head
                    .as_deref()
                    .ok_or("Create a first commit before adding a worktree.")?;
                if fs::canonicalize(&parent).map_err(|e| e.to_string())? != parent
                    || git(&parent, &["rev-parse", "--absolute-git-dir"]).is_ok()
                {
                    return Err(
                        "Parent folder changed or is inside a Git worktree. Choose it again."
                            .into(),
                    );
                }
                let destination = parent.join(&directory);
                match fs::symlink_metadata(&destination) {
                    Ok(_) => return Err("Destination already exists. Choose a new folder name; nothing was overwritten.".into()),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
                    Err(error) => return Err(format!("Cannot inspect destination: {error}")),
                }
                let exists = snapshot
                    .branches
                    .iter()
                    .any(|b| !b.remote && b.name == branch);
                if create_branch && exists {
                    return Err("That local branch already exists. Choose another name or use an existing branch.".into());
                }
                if !create_branch
                    && (!exists || snapshot.worktrees.iter().any(|tree| tree.branch == branch))
                {
                    return Err("Choose an existing local branch that is not checked out in another worktree.".into());
                }
                let destination = crate::native_paths::git_path(&destination)?;
                if create_branch {
                    git(
                        root,
                        &[
                            "worktree",
                            "add",
                            "--checkout",
                            "-b",
                            &branch,
                            "--",
                            &destination,
                            head,
                        ],
                    )?;
                } else {
                    git(
                        root,
                        &["worktree", "add", "--checkout", "--", &destination, &branch],
                    )?;
                }
                self.worktree_parents.remove(&selection_id);
            }
            Action::CreateBranch { name } => {
                validate_branch(root, &name)?;
                git(root, &["switch", "-c", &name])?;
            }
            Action::CreateBranchAt { name, target } => {
                validate_branch(root, &name)?;
                verify_commit(root, &target)?;
                git(root, &["branch", "--", &name, &target])?;
            }
            Action::CreateTag { name, target } => {
                if name.starts_with('-') || name.len() > 1024 {
                    return Err("Invalid tag name".into());
                }
                git(root, &["check-ref-format", &format!("refs/tags/{name}")])?;
                verify_commit(root, &target)?;
                git(root, &["tag", "--no-sign", "--", &name, &target])?;
            }
            Action::RenameBranch { name, new_name } => {
                validate_branch(root, &name)?;
                validate_branch(root, &new_name)?;
                if !snapshot
                    .branches
                    .iter()
                    .any(|b| !b.remote && b.name == name)
                {
                    return Err("Choose an existing local branch.".into());
                }
                git(root, &["branch", "-m", &name, &new_name])?;
            }
            Action::TrackBranch {
                name,
                remote_branch,
            } => {
                validate_branch(root, &name)?;
                if !snapshot
                    .branches
                    .iter()
                    .any(|b| b.remote && b.name == remote_branch)
                {
                    return Err("Choose an existing remote branch.".into());
                }
                if !snapshot.files.is_empty() {
                    return Err("Commit or stash working changes before checkout.".into());
                }
                git(
                    root,
                    &[
                        "switch",
                        "--track",
                        "-c",
                        &name,
                        &format!("refs/remotes/{remote_branch}"),
                    ],
                )?;
            }
            Action::CheckoutCommit { target }
            | Action::CherryPick { target }
            | Action::Revert { target } => {
                verify_commit(root, &target)?;
                if !snapshot.files.is_empty() {
                    return Err("Commit or stash working changes before this operation. Your files were left unchanged.".into());
                }
                match action_kind.as_str() {
                    "checkout" => {
                        git(root, &["switch", "--detach", &target])?;
                    }
                    "cherry-pick" | "revert" => {
                        if snapshot.head.is_none() || snapshot.branch == "Detached HEAD" {
                            return Err("Switch to a committed local branch first.".into());
                        }
                        let parents = text(git(root, &["show", "-s", "--format=%P", &target])?)?;
                        if parents.split_whitespace().count() > 1 {
                            return Err("Merge commits require a mainline parent; this operation is not available yet.".into());
                        }
                        if action_kind == "revert" {
                            git(root, &["revert", "--no-edit", &target])?;
                        } else {
                            git(root, &["cherry-pick", &target])?;
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Action::Merge { name } => {
                validate_branch(root, &name)?;
                if snapshot.head.is_none() || snapshot.branch == "Detached HEAD" {
                    return Err("Switch to a committed local branch before merging.".into());
                }
                if name == snapshot.branch
                    || !snapshot
                        .branches
                        .iter()
                        .any(|branch| !branch.remote && branch.name == name)
                {
                    return Err("Choose another existing local branch to merge.".into());
                }
                if !snapshot.files.is_empty() {
                    return Err("Commit or stash working changes before merging. Your files were left unchanged.".into());
                }
                let reference = format!("refs/heads/{name}");
                git(
                    root,
                    &[
                        "merge",
                        "--ff",
                        "--commit",
                        "--no-squash",
                        "--no-edit",
                        "--no-autostash",
                        "--",
                        &reference,
                    ],
                )?;
            }
            Action::DeleteBranch { name } => {
                validate_branch(root, &name)?;
                git(root, &["branch", "-d", "--", &name])?;
            }
            Action::Fetch => {
                git(root, &["fetch", "--all"])?;
            }
            Action::Pull => {
                if snapshot.upstream.is_none() {
                    return Err("Set an upstream before pulling.".into());
                }
                let (remote, reference) = tracking(root, &snapshot.branch)?;
                git(root, &["pull", "--ff-only", "--", &remote, &reference])?;
            }
            Action::Push => {
                if snapshot.branch == "Detached HEAD" {
                    return Err("Create or switch to a branch before pushing.".into());
                }
                if snapshot.upstream.is_none() {
                    return Err("Set an upstream explicitly with Publish branch.".into());
                }
                let (remote, reference) = tracking(root, &snapshot.branch)?;
                let refspec = format!("refs/heads/{}:{}", snapshot.branch, reference);
                git(root, &["push", "--", &remote, &refspec])?;
            }
            Action::Publish { remote } => {
                let remotes = text(git(root, &["remote"])?)?;
                if !remotes.lines().any(|r| r == remote) || remote.starts_with('-') {
                    return Err("Choose a configured remote.".into());
                }
                validate_branch(root, &snapshot.branch)?;
                let refspec = format!("refs/heads/{0}:refs/heads/{0}", snapshot.branch);
                git(root, &["push", "--set-upstream", "--", &remote, &refspec])?;
            }
            Action::Stash { message } => {
                git(
                    root,
                    &["stash", "push", "--include-untracked", "-m", &message],
                )?;
            }
            Action::ApplyStash { index, oid } => crate::stash::apply(root, index, &oid, false)?,
            Action::PopStash { index, oid } => crate::stash::apply(root, index, &oid, true)?,
            Action::DropStash { index, oid } => crate::stash::drop(root, index, &oid)?,
            Action::RestoreStashFile { index, oid, path } => {
                crate::stash_restore::restore(root, index, &oid, &path, snapshot.head.as_deref())?
            }
        }
        self.snapshot(id)
    }
}
fn tracking(root: &Path, branch: &str) -> Result<(String, String), String> {
    let current = format!("refs/heads/{branch}");
    let raw = text(git(
        root,
        &[
            "for-each-ref",
            "--format=%(refname)%00%(upstream:remotename)%00%(upstream:remoteref)",
            "refs/heads/",
        ],
    )?)?;
    for line in raw.lines() {
        let fields: Vec<_> = line.split('\0').collect();
        if fields.len() == 3 && fields[0] == current {
            if fields[1].is_empty()
                || fields[1].starts_with('-')
                || !fields[2].starts_with("refs/heads/")
            {
                break;
            }
            return Ok((fields[1].into(), fields[2].into()));
        }
    }
    Err("This branch must track a configured remote branch before synchronizing.".into())
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeParent {
    pub id: String,
    pub path: String,
}
pub(crate) fn validate_worktree_directory(name: &str) -> Result<(), String> {
    let upper = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    let device = ["CON", "PRN", "AUX", "NUL"].contains(&upper.as_str())
        || (upper.len() == 4
            && (upper.starts_with("COM") || upper.starts_with("LPT"))
            && matches!(upper.as_bytes()[3], b'1'..=b'9'));
    if name.is_empty()
        || name.len() > 100
        || name.trim() != name
        || name.ends_with('.')
        || name == "."
        || name == ".."
        || name.eq_ignore_ascii_case(".git")
        || device
        || name
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
    {
        return Err("Use a portable single folder name (up to 100 UTF-8 bytes), without paths, reserved characters or device names.".into());
    }
    Ok(())
}
#[derive(Clone, Copy, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SequenceKind {
    CherryPick,
    Revert,
}
impl SequenceKind {
    fn command(self) -> &'static str {
        match self {
            Self::CherryPick => "cherry-pick",
            Self::Revert => "revert",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::CherryPick => "Cherry-pick in progress",
            Self::Revert => "Revert in progress",
        }
    }
}
#[derive(Clone, Copy, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SequenceStep {
    Continue,
    Abort,
    Skip,
}
/// Identify the full remaining sequence, never guess from a single current commit.
fn sequence_kind(dir: &Path) -> Result<Option<SequenceKind>, String> {
    let path = dir.join("sequencer/todo");
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("Cannot read sequence: {error}")),
    };
    let mut bytes = Vec::new();
    file.take(16385)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 16384 {
        return Err("Sequence is too large to review safely.".into());
    }
    let content = text(bytes)?;
    let mut kind = None;
    for line in content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
    {
        let next = match line.split_whitespace().next() {
            Some("pick") => SequenceKind::CherryPick,
            Some("revert") => SequenceKind::Revert,
            _ => {
                return Err("Unknown sequencer instruction. Continue with Git outside Ramo.".into())
            }
        };
        if kind.is_some_and(|previous| previous != next) {
            return Err("Mixed sequencer instructions require Git outside Ramo.".into());
        }
        kind = Some(next);
    }
    Ok(kind)
}
#[derive(Serialize)]
pub struct ConflictFile {
    pub base: Option<String>,
    pub ours: Option<String>,
    pub theirs: Option<String>,
    pub working: Option<String>,
}
fn regular_resolution_path(root: &Path, path: &str) -> Result<PathBuf, String> {
    valid_path(path)?;
    let destination = root.join(path);
    let parent = fs::canonicalize(destination.parent().ok_or("Missing parent")?)
        .map_err(|e| e.to_string())?;
    if !parent.starts_with(root) {
        return Err("File is outside this worktree.".into());
    }
    if let Ok(metadata) = fs::symlink_metadata(&destination) {
        if !metadata.file_type().is_file() {
            return Err("Guided editor requires a regular file.".into());
        }
    }
    Ok(parent.join(destination.file_name().ok_or("Missing filename")?))
}
pub(crate) fn verify_commit(root: &Path, target: &str) -> Result<(), String> {
    oid(target)?;
    git(root, &["cat-file", "-e", &format!("{target}^{{commit}}")]).map(|_| ())
}
fn validate_branch(root: &Path, name: &str) -> Result<(), String> {
    if name.starts_with('-') || name == "HEAD" || name.len() > 1024 {
        return Err("Invalid branch name".into());
    }
    git(root, &["check-ref-format", "--branch", name]).map(|_| ())
}
#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    WorktreeLock {
        path: String,
        locked: bool,
    },
    RemoveWorktree {
        path: String,
    },
    CreateWorktree {
        #[serde(rename = "selectionId")]
        selection_id: String,
        directory: String,
        branch: String,
        #[serde(rename = "createBranch")]
        create_branch: bool,
    },
    Lines {
        path: String,
        rows: Vec<usize>,
        unstage: bool,
        #[serde(rename = "patchHash")]
        patch_hash: String,
    },
    Hunk {
        path: String,
        index: usize,
        unstage: bool,
        #[serde(rename = "patchHash")]
        patch_hash: String,
    },
    Stage {
        paths: Vec<String>,
    },
    Unstage {
        paths: Vec<String>,
    },
    ConfigureRemote {
        name: String,
        url: String,
        existing: bool,
    },
    Commit {
        message: String,
    },
    Amend {
        head: String,
        message: String,
    },
    MarkResolved {
        path: String,
    },
    ResolveFile {
        path: String,
        content: Option<String>,
    },
    AbortMerge,
    Sequence {
        operation: SequenceKind,
        step: SequenceStep,
    },
    Merge {
        name: String,
    },
    Switch {
        name: String,
    },
    CreateBranch {
        name: String,
    },
    DeleteBranch {
        name: String,
    },
    CreateBranchAt {
        name: String,
        target: String,
    },
    CreateTag {
        name: String,
        target: String,
    },
    RenameBranch {
        name: String,
        #[serde(rename = "newName")]
        new_name: String,
    },
    TrackBranch {
        name: String,
        #[serde(rename = "remoteBranch")]
        remote_branch: String,
    },
    CheckoutCommit {
        target: String,
    },
    CherryPick {
        target: String,
    },
    Revert {
        target: String,
    },
    Fetch,
    Pull,
    Push,
    Publish {
        remote: String,
    },
    Stash {
        message: String,
    },
    ApplyStash {
        index: usize,
        oid: String,
    },
    PopStash {
        index: usize,
        oid: String,
    },
    RestoreStashFile {
        index: usize,
        oid: String,
        path: String,
    },
    DropStash {
        index: usize,
        oid: String,
    },
}

pub fn parse_status(raw: &[u8]) -> Result<Vec<FileChange>, String> {
    let mut chunks = raw.split(|b| *b == 0).filter(|b| !b.is_empty());
    let mut files = vec![];
    while let Some(row) = chunks.next() {
        if row.len() < 4 || row[2] != b' ' {
            return Err("Unexpected Git status record".into());
        }
        let original = if matches!(row[0], b'R' | b'C') || matches!(row[1], b'R' | b'C') {
            Some(text(
                chunks.next().ok_or("Missing rename source")?.to_vec(),
            )?)
        } else {
            None
        };
        files.push(FileChange {
            path: text(row[3..].to_vec())?,
            original,
            index: (row[0] as char).to_string(),
            worktree: (row[1] as char).to_string(),
        });
    }
    Ok(files)
}
pub(crate) fn parse_names(raw: &[u8]) -> Result<Vec<FileChange>, String> {
    let mut chunks = raw.split(|b| *b == 0).filter(|b| !b.is_empty());
    let mut files = vec![];
    while let Some(status) = chunks.next() {
        let first = text(chunks.next().ok_or("Missing changed path")?.to_vec())?;
        let rename = matches!(status.first(), Some(b'R' | b'C'));
        let path = if rename {
            text(chunks.next().ok_or("Missing rename target")?.to_vec())?
        } else {
            first.clone()
        };
        files.push(FileChange {
            path,
            original: if rename { Some(first) } else { None },
            index: (status[0] as char).to_string(),
            worktree: " ".into(),
        });
    }
    Ok(files)
}
fn worktrees(root: &Path) -> Result<Vec<Worktree>, String> {
    // -z is not supported by every system Git. Refuse ambiguous fallback paths.
    let (raw, nul) = match git(root, &["worktree", "list", "--porcelain", "-z"]) {
        Ok(b) => (text(b)?, true),
        Err(_) => (
            text(git(root, &["worktree", "list", "--porcelain"])?)?,
            false,
        ),
    };
    let mut trees = vec![];
    let mut current: Option<Worktree> = None;
    for line in raw.split(if nul { '\0' } else { '\n' }) {
        if let Some(path) = line.strip_prefix("worktree ") {
            if let Some(previous) = current.take() {
                trees.push(previous);
            }
            if !nul && path.starts_with('"') {
                return Err("This Git version cannot safely represent this worktree path. Use a newer Git with worktree -z support.".into());
            }
            current = Some(Worktree {
                path: path.into(),
                branch: "Detached HEAD".into(),
                locked: false,
                primary: trees.is_empty(),
                head: String::new(),
            });
        } else if let Some(tree) = &mut current {
            if let Some(head) = line.strip_prefix("HEAD ") {
                tree.head = head.into();
            }
            if let Some(branch) = line.strip_prefix("branch refs/heads/") {
                tree.branch = branch.into();
            }
            if line.starts_with("locked") {
                tree.locked = true;
            }
        }
    }
    if let Some(tree) = current {
        trees.push(tree);
    }
    Ok(trees)
}

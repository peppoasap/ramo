//! Release-candidate timings on disposable actual Git. No network or AI.
use ramo_core::{
    git::{self, Action, Backend},
    process,
};
use std::{
    collections::BTreeMap,
    fs,
    time::{Duration, Instant},
};
fn record<T>(
    out: &mut BTreeMap<String, Vec<f64>>,
    name: &str,
    action: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let start = Instant::now();
    let value = action()?;
    out.entry(name.into())
        .or_default()
        .push(start.elapsed().as_secs_f64() * 1000.0);
    Ok(value)
}
fn main() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|e| e.to_string())?;
    let root = temp.path().join("meadow-delivery");
    fs::create_dir(&root).map_err(|e| e.to_string())?;
    git::git(&root, &["init", "-b", "main"])?;
    for (key, value) in [
        ("user.name", "Synthetic Contributor"),
        ("user.email", "synthetic@example.invalid"),
        ("commit.gpgsign", "false"),
        ("core.fsmonitor", "false"),
    ] {
        git::git(&root, &["config", key, value])?;
    }
    let mut stream = String::new();
    for i in 0..5000 {
        let message = format!("Synthetic change {i}");
        stream.push_str(&format!("commit refs/heads/main\ncommitter Synthetic <synthetic@example.invalid> {} +0000\ndata {}\n{}\n",1700000000+i,message.len(),message));
        if i == 0 {
            for n in 0..2000 {
                stream.push_str(&format!(
                    "M 100644 inline file-{n}.txt\ndata 8\ninitial\n\n"
                ));
            }
        }
        let content = format!("version {i}\n");
        stream.push_str(&format!(
            "M 100644 inline file-0.txt\ndata {}\n{}\n\n",
            content.len(),
            content
        ));
    }
    process::run(
        &process::executable("git")?,
        &root,
        &process::strings(&["fast-import", "--quiet"]),
        Some(stream.into_bytes()),
        Duration::from_secs(30),
    )?;
    git::git(&root, &["reset", "--hard", "HEAD"])?;
    git::git(&root, &["branch", "other"])?;
    let original = String::from_utf8(git::git(&root, &["rev-parse", "HEAD"])?)
        .map_err(|e| e.to_string())?
        .trim()
        .to_owned();
    // Preparation and compilation are outside every recorded interval.
    let mut backend = Backend::new(temp.path().join("preferences.json"));
    let snapshot = backend.register(&root)?;
    let id = snapshot.repo.id;
    backend.trust(&id)?;
    let mut data = BTreeMap::new();
    for pass in 0..10 {
        record(&mut data, "cleanSnapshotMs", || backend.snapshot(&id))?;
        let page = record(&mut data, "history300Ms", || backend.history(&id, 0))?;
        if page.len() != 300 {
            return Err("Wrong history page length".into());
        }
        let last = record(&mut data, "historyLastPageMs", || {
            backend.history(&id, 4800)
        })?;
        if last.len() != 200 {
            return Err("Wrong last page length".into());
        }
        record(&mut data, "commitFilesMs", || {
            backend.commit_files(&id, &original)
        })?;
        record(&mut data, "commitDiffMs", || {
            backend.file_diff(&id, "file-0.txt", Some(&original), false)
        })?;
        let mut fresh = Backend::new(temp.path().join(format!("open-{pass}.json")));
        record(&mut data, "registerWarmRepositoryMs", || {
            fresh.register(&root)
        })?;
    }
    let paths: Vec<String> = (0..100).map(|i| format!("file-{i}.txt")).collect();
    for path in &paths {
        fs::write(root.join(path), "Synthetic modified content\n").map_err(|e| e.to_string())?;
    }
    for _ in 0..10 {
        let snap = record(&mut data, "dirtySnapshot100Ms", || backend.snapshot(&id))?;
        if snap.files.len() != 100 {
            return Err("Expected 100 dirty files".into());
        }
        record(&mut data, "workingDiffMs", || {
            backend.file_diff(&id, "file-0.txt", None, false)
        })?;
        let staged = record(&mut data, "stage100FilesMs", || {
            backend.mutate(
                &id,
                &snap.fingerprint,
                Action::Stage {
                    paths: paths.clone(),
                },
            )
        })?;
        if !staged.files.iter().all(|f| f.index == "M") {
            return Err("Stage failed".into());
        }
        let unstaged = record(&mut data, "unstage100FilesMs", || {
            backend.mutate(
                &id,
                &staged.fingerprint,
                Action::Unstage {
                    paths: paths.clone(),
                },
            )
        })?;
        if !unstaged
            .files
            .iter()
            .all(|f| f.index == " " && f.worktree == "M")
        {
            return Err("Unstage did not restore the expected unstaged state".into());
        }
    }
    for pass in 0..10 {
        git::git(&root, &["add", "--", "file-0.txt"])?;
        let snap = backend.snapshot(&id)?;
        let committed = record(&mut data, "commitOneFileMs", || {
            backend.mutate(
                &id,
                &snap.fingerprint,
                Action::Commit {
                    message: format!("Synthetic benchmark commit {pass}"),
                },
            )
        })?;
        if committed.files.len() != 99 {
            return Err("Commit included unexpected work".into());
        }
        // Reset only this automatically disposable fixture; retain the dirty bytes for the next sample.
        git::git(&root, &["reset", "--mixed", &original])?;
    }
    git::git(&root, &["reset", "--hard", &original])?;
    for _ in 0..10 {
        let snap = backend.snapshot(&id)?;
        let switched = record(&mut data, "switchBranchMs", || {
            backend.mutate(
                &id,
                &snap.fingerprint,
                Action::Switch {
                    name: "other".into(),
                },
            )
        })?;
        if switched.branch != "other" {
            return Err("Branch switch failed".into());
        }
        backend.mutate(
            &id,
            &switched.fingerprint,
            Action::Switch {
                name: "main".into(),
            },
        )?;
    }
    // A separate divergent, conflict-free merge phase; setup is not timed.
    git::git(&root, &["checkout", "-b", "merge-source"])?;
    fs::write(root.join("feature-only.txt"), "Synthetic feature\n").map_err(|e| e.to_string())?;
    git::git(&root, &["add", "--", "feature-only.txt"])?;
    git::git(&root, &["commit", "-m", "Synthetic merge source"])?;
    git::git(&root, &["checkout", "main"])?;
    fs::write(root.join("main-only.txt"), "Synthetic main\n").map_err(|e| e.to_string())?;
    git::git(&root, &["add", "--", "main-only.txt"])?;
    git::git(&root, &["commit", "-m", "Synthetic merge target"])?;
    let merge_base = String::from_utf8(git::git(&root, &["rev-parse", "HEAD"])?)
        .map_err(|e| e.to_string())?
        .trim()
        .to_owned();
    for pass in 0..10 {
        let name = format!("merge-sample-{pass}");
        git::git(&root, &["branch", &name, "merge-source"])?;
        let snap = backend.snapshot(&id)?;
        let merged = record(&mut data, "mergeThreeWayMs", || {
            backend.mutate(&id, &snap.fingerprint, Action::Merge { name })
        })?;
        if !merged.files.is_empty()
            || merged.operation.is_some()
            || !root.join("main-only.txt").exists()
            || !root.join("feature-only.txt").exists()
        {
            return Err("Merge did not preserve the expected clean combined files".into());
        }
        let parents = String::from_utf8(git::git(&root, &["show", "-s", "--format=%P", "HEAD"])?)
            .map_err(|e| e.to_string())?;
        if parents.split_whitespace().count() != 2 {
            return Err("Expected two-parent merge commit".into());
        }
        git::git(&root, &["reset", "--hard", &merge_base])?;
    }
    for values in data.values_mut() {
        values.remove(0);
        for v in values {
            *v = (*v * 100.0).round() / 100.0;
        }
    }
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({
      "scenario":"5000 linear commits, 2000 tracked small text files; 100 modified files in dirty phase. Merge phase adds two divergent commits/files and a new source ref per sample.",
      "profile":if cfg!(debug_assertions){"debug"}else{"release"},
      "platform":std::env::consts::OS,"architecture":std::env::consts::ARCH,
      "git":String::from_utf8_lossy(&git::git(&root,&["--version"])?).trim(),
      "method":"One discarded warm-up then nine samples per operation. Stopwatch covers actual Backend call including Git subprocesses and mutation freshness/result snapshots. Fixture preparation, user review, IPC, UI, network and AI excluded. Open is warm repository registration, not application launch. Switch changes branch identity at the same tree, not a large checkout.",
      "samples":data
    })).unwrap());
    Ok(())
}

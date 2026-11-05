//! Repeatable backend timings on disposable synthetic data. No network or AI.
use ramo_core::{
    git::{self, Backend},
    process,
};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

fn samples(mut action: impl FnMut() -> Result<(), String>) -> Result<Vec<f64>, String> {
    (0..7)
        .map(|_| {
            let start = Instant::now();
            action()?;
            Ok((start.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0)
        })
        .collect()
}

fn main() -> Result<(), String> {
    let temporary = tempfile::tempdir().map_err(|e| e.to_string())?;
    let root = temporary.path().join("meadow-performance");
    fs::create_dir(&root).map_err(|e| e.to_string())?;
    git::git(&root, &["init", "-b", "main"])?;
    let mut import = String::new();
    for i in 0..5000 {
        let message = format!("Synthetic change {i}");
        import.push_str(&format!("commit refs/heads/main\ncommitter Synthetic <test@example.invalid> {} +0000\ndata {}\n{}\n", 1700000000 + i, message.len(), message));
        if i == 0 {
            for file in 0..2000 {
                import.push_str(&format!(
                    "M 100644 inline file-{file}.txt\ndata 8\ninitial\n\n"
                ));
            }
        }
        let content = format!("version {i}\n");
        import.push_str(&format!(
            "M 100644 inline file-0.txt\ndata {}\n{}\n\n",
            content.len(),
            content
        ));
    }
    process::run(
        &process::executable("git")?,
        &root,
        &process::strings(&["fast-import", "--quiet"]),
        Some(import.into_bytes()),
        Duration::from_secs(30),
    )?;
    git::git(&root, &["reset", "--hard", "HEAD"])?;
    let mut backend = Backend::new(temporary.path().join("preferences.json"));
    let open = Instant::now();
    let snapshot = backend.register(Path::new(&root))?;
    let opening_ms = open.elapsed().as_secs_f64() * 1000.0;
    let id = snapshot.repo.id;
    let clean = samples(|| backend.snapshot(&id).map(|_| ()))?;
    let history = samples(|| {
        let commits = backend.history(&id, 0)?;
        if commits.len() != 300 {
            return Err("History page is not 300 commits".into());
        }
        Ok(())
    })?;
    for i in 0..100 {
        fs::write(
            root.join(format!("file-{i}.txt")),
            format!("modified synthetic {i}\n"),
        )
        .map_err(|e| e.to_string())?;
    }
    git::git(&root, &["add", "--", "file-0.txt"])?;
    let dirty = samples(|| {
        let snapshot = backend.snapshot(&id)?;
        if snapshot.files.len() != 100 {
            return Err("Expected 100 changed files".into());
        }
        Ok(())
    })?;
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "scenario":"Meadow synthetic: 5000 linear commits, 2000 tracked files, then 100 changed files (1 staged)",
        "platform":std::env::consts::OS,"architecture":std::env::consts::ARCH,
        "profile":if cfg!(debug_assertions) {"debug"} else {"release"},
        "scope":"Backend wall-clock; seven warm repetitions; no UI/startup/network/AI timing",
        "git":String::from_utf8_lossy(&git::git(&root,&["--version"])?).trim(),
        "firstOpenMs":opening_ms,"cleanSnapshotMs":clean,"historyPageMs":history,"dirtySnapshotMs":dirty
    })).unwrap());
    Ok(())
}

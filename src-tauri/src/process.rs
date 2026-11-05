//! Bounded subprocess execution. No shell interpolation; timeout kills the process group.
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

pub const OUTPUT_LIMIT: usize = 16 * 1024 * 1024;

pub fn executable(name: &str) -> Result<PathBuf, String> {
    if name == "codex" {
        if let Some(path) = std::env::var_os("RAMO_CODEX_PATH") {
            let path = PathBuf::from(path);
            if path.is_absolute() && is_executable_file(&path) {
                return Ok(path);
            }
            return Err(
                "RAMO_CODEX_PATH must be an absolute path to an executable Codex CLI".into(),
            );
        }
    }
    // GUI apps on macOS often inherit a smaller PATH than a login shell.
    let suffix = if cfg!(windows) { ".exe" } else { "" };
    let filename = format!("{name}{suffix}");
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        let home = PathBuf::from(home);
        dirs.extend([home.join(".local/bin"), home.join(".cargo/bin")]);
    }
    dirs.extend([
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
    ]);
    find_executable(&filename, &dirs).ok_or_else(|| {
        format!("Cannot find {name}. Install its official CLI and restart Ramo with its executable folder in PATH.")
    })
}

fn find_executable(filename: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    dirs.iter()
        // Empty/relative PATH entries would let the repository choose a program.
        .filter(|dir| dir.is_absolute())
        .map(|dir| dir.join(filename))
        .find(|path| is_executable_file(path))
}

fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        return path
            .metadata()
            .is_ok_and(|m| m.permissions().mode() & 0o111 != 0);
    }
    #[cfg(not(unix))]
    true
}

fn collect(mut input: impl Read, exceeded: Arc<AtomicBool>, hash: bool) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut digest = Sha256::new();
    let mut chunk = [0; 8192];
    loop {
        match input.read(&mut chunk) {
            Ok(0) => break,
            Err(e) => return Err(format!("Cannot read process output: {e}")),
            Ok(n) => {
                if hash {
                    digest.update(&chunk[..n]);
                } else if bytes.len() + n > OUTPUT_LIMIT {
                    exceeded.store(true, Ordering::Relaxed);
                    break;
                } else {
                    bytes.extend_from_slice(&chunk[..n]);
                }
            }
        }
    }
    Ok(if hash {
        digest.finalize().to_vec()
    } else {
        bytes
    })
}

// Most Git inspection commands finish in a few milliseconds. Avoid imposing
// a 10 ms floor on every command in a snapshot, while backing off for long
// fetch/AI processes so they do not keep waking at the short-command rate.
fn poll_delay(elapsed: Duration) -> Duration {
    Duration::from_millis(if elapsed < Duration::from_millis(50) {
        2
    } else {
        10
    })
}

pub fn run(
    program: &Path,
    cwd: &Path,
    args: &[String],
    input: Option<Vec<u8>>,
    timeout: Duration,
) -> Result<Vec<u8>, String> {
    run_captured(program, cwd, args, input, timeout, false, None)
}
pub fn run_cancellable(
    program: &Path,
    cwd: &Path,
    args: &[String],
    input: Option<Vec<u8>>,
    timeout: Duration,
    cancellation: &crate::cancellation::Cancellation,
) -> Result<Vec<u8>, String> {
    cancellation.check()?;
    run_captured(
        program,
        cwd,
        args,
        input,
        timeout,
        false,
        Some(cancellation),
    )
}

/// Hash stdout incrementally instead of retaining a full binary Git patch.
/// The process deadline still applies; stderr retains its ordinary size cap.
pub fn run_digest(
    program: &Path,
    cwd: &Path,
    args: &[String],
    timeout: Duration,
) -> Result<Vec<u8>, String> {
    run_captured(program, cwd, args, None, timeout, true, None)
}

fn run_captured(
    program: &Path,
    cwd: &Path,
    args: &[String],
    input: Option<Vec<u8>>,
    timeout: Duration,
    hash_stdout: bool,
    cancellation: Option<&crate::cancellation::Cancellation>,
) -> Result<Vec<u8>, String> {
    if !program.is_absolute() {
        return Err("Refusing to run a program without an absolute executable path".into());
    }
    let mut command = Command::new(program);
    command
        .current_dir(cwd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GCM_INTERACTIVE", "Never");
    // A repository-specific Git environment from the launching terminal must not redirect operations.
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_COMMON_DIR",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_CONFIG_COUNT",
        "GIT_CONFIG_PARAMETERS",
    ] {
        command.env_remove(key);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000 | 0x00000200);
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("Cannot start {}: {e}", program.display()))?;
    let stdin = child.stdin.take();
    let writer = thread::spawn(move || {
        if let (Some(mut pipe), Some(bytes)) = (stdin, input) {
            let _ = pipe.write_all(&bytes);
        }
    });
    let exceeded = Arc::new(AtomicBool::new(false));
    let out_flag = exceeded.clone();
    let err_flag = exceeded.clone();
    let stdout = child.stdout.take().ok_or("Missing process output")?;
    let stderr = child.stderr.take().ok_or("Missing process error output")?;
    let out = thread::spawn(move || collect(stdout, out_flag, hash_stdout));
    let err = thread::spawn(move || collect(stderr, err_flag, false));
    let started = Instant::now();
    let mut exit_status = None;
    let status = loop {
        let cancelled = cancellation.is_some_and(|token| token.is_cancelled());
        if cancelled || started.elapsed() > timeout || exceeded.load(Ordering::Relaxed) {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            #[cfg(windows)]
            {
                let _ = Command::new("taskkill")
                    .args(["/PID", &child.id().to_string(), "/T", "/F"])
                    .output();
            }
            let _ = child.kill();
            let _ = child.wait();
            let _ = writer.join();
            let _ = out.join();
            let _ = err.join();
            return Err(if cancelled { crate::cancellation::CANCELLED } else if exceeded.load(Ordering::Relaxed) { "Operation exceeded the 16 MiB output limit. Narrow the selected history or diff." } else { "Operation timed out. Refresh the repository to check its actual state before retrying a write." }.into());
        }
        if let Some(status) = exit_status {
            if out.is_finished() && err.is_finished() && writer.is_finished() {
                break status;
            }
            thread::sleep(poll_delay(started.elapsed()));
            continue;
        }
        match child.try_wait() {
            Ok(Some(status)) => exit_status = Some(status),
            Ok(None) => thread::sleep(poll_delay(started.elapsed())),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("Cannot check child process: {e}"));
            }
        }
    };
    let _ = writer.join();
    let output = out.join().map_err(|_| "Cannot read process output")??;
    let errors = err.join().map_err(|_| "Cannot read process errors")??;
    if let Some(token) = cancellation {
        token.check()?;
    }
    if exceeded.load(Ordering::Relaxed) {
        return Err("Process output exceeded its limit".into());
    }
    if status.success() {
        Ok(output)
    } else {
        if hash_stdout && errors.is_empty() {
            return Err(
                "Git could not compute the review fingerprint. Refresh before retrying.".into(),
            );
        }
        let detail = String::from_utf8_lossy(if errors.is_empty() { &output } else { &errors });
        let mut tail = detail.trim().chars().rev().take(4000).collect::<Vec<_>>();
        tail.reverse();
        Err(tail.into_iter().collect())
    }
}

pub fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_read_errors_cannot_become_successful_partial_results() {
        struct BrokenReader;
        impl Read for BrokenReader {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "synthetic read failure",
                ))
            }
        }
        for hash in [false, true] {
            let error = collect(BrokenReader, Arc::new(AtomicBool::new(false)), hash).unwrap_err();
            assert!(error.contains("synthetic read failure"));
        }
    }

    #[test]
    fn refuses_implicit_repository_program_lookup() {
        let folder = tempfile::tempdir().unwrap();
        let error = run(
            Path::new("git"),
            folder.path(),
            &[],
            None,
            Duration::from_secs(1),
        )
        .unwrap_err();
        assert!(error.contains("absolute executable path"));
        assert!(find_executable("git", &[PathBuf::new(), PathBuf::from(".")]).is_none());
        assert!(find_executable("git", &[folder.path().to_path_buf()]).is_none());
    }

    #[test]
    fn resolves_absolute_program_folders_in_order() {
        let folder = tempfile::tempdir().unwrap();
        let program = folder.path().join("synthetic-tool");
        std::fs::write(&program, b"synthetic; never executed").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert!(find_executable("synthetic-tool", &[folder.path().to_path_buf()]).is_none());
            std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        assert_eq!(
            find_executable("synthetic-tool", &[folder.path().to_path_buf()]),
            Some(program)
        );
    }
}

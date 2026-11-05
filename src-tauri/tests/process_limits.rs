//! Process fixtures do not execute any repository-provided commands.
#[cfg(unix)]
mod unix {
    use ramo_core::process::{run, run_digest, strings};
    use sha2::{Digest, Sha256};
    use std::{
        path::Path,
        time::{Duration, Instant},
    };
    #[test]
    fn fingerprints_large_output_while_normal_capture_remains_bounded() {
        let folder = tempfile::tempdir().unwrap();
        let digest = run_digest(
            Path::new("/bin/sh"),
            folder.path(),
            &strings(&["-c", "head -c 18000000 /dev/zero"]),
            Duration::from_secs(3),
        )
        .unwrap();
        assert_eq!(digest, Sha256::digest(vec![0u8; 18_000_000]).to_vec());
    }
    #[test]
    fn times_out_a_running_child() {
        let folder = tempfile::tempdir().unwrap();
        let start = Instant::now();
        let result = run(
            Path::new("/bin/sh"),
            folder.path(),
            &strings(&["-c", "sleep 10"]),
            None,
            Duration::from_millis(150),
        );
        assert!(result.unwrap_err().contains("timed out"));
        assert!(start.elapsed() < Duration::from_secs(3));
    }
    #[test]
    fn bounds_output_wait_even_when_the_parent_exits_before_its_child() {
        let folder = tempfile::tempdir().unwrap();
        let start = Instant::now();
        let result = run(
            Path::new("/bin/sh"),
            folder.path(),
            &strings(&["-c", "sleep 10 & exit 0"]),
            None,
            Duration::from_millis(150),
        );
        assert!(result.unwrap_err().contains("timed out"));
        assert!(start.elapsed() < Duration::from_secs(3));
    }
    #[test]
    fn rejects_oversized_child_output() {
        let folder = tempfile::tempdir().unwrap();
        let result = run(
            Path::new("/bin/sh"),
            folder.path(),
            &strings(&["-c", "head -c 18000000 /dev/zero"]),
            None,
            Duration::from_secs(3),
        );
        assert!(result.unwrap_err().contains("limit"));
    }
}
#[cfg(unix)]
mod cancellation {
    use ramo_core::{
        cancellation::Cancellation,
        process::{run_cancellable, strings},
    };
    use std::{
        path::Path,
        thread,
        time::{Duration, Instant},
    };
    #[test]
    fn cancelled_process_never_starts() {
        let folder = tempfile::tempdir().unwrap();
        let token = Cancellation::default();
        token.cancel();
        let result = run_cancellable(
            Path::new("/bin/sh"),
            folder.path(),
            &strings(&["-c", "echo unexpected > started"]),
            None,
            Duration::from_secs(10),
            &token,
        );
        assert!(result.unwrap_err().contains("cancelled"));
        assert!(!folder.path().join("started").exists());
    }
    #[test]
    fn cancellation_kills_running_and_orphaned_descendants_before_pipe_join() {
        for script in [
            "echo ready > ready; sleep 30",
            "sleep 30 & echo ready > ready; exit 0",
        ] {
            let folder = tempfile::tempdir().unwrap();
            let path = folder.path().to_path_buf();
            let token = Cancellation::default();
            let worker_token = token.clone();
            let worker = thread::spawn(move || {
                run_cancellable(
                    Path::new("/bin/sh"),
                    &path,
                    &strings(&["-c", script]),
                    None,
                    Duration::from_secs(30),
                    &worker_token,
                )
            });
            let deadline = Instant::now() + Duration::from_secs(3);
            while !folder.path().join("ready").exists() {
                assert!(Instant::now() < deadline, "fixture did not start");
                thread::sleep(Duration::from_millis(10));
            }
            let start = Instant::now();
            token.cancel();
            assert!(worker.join().unwrap().unwrap_err().contains("cancelled"));
            assert!(start.elapsed() < Duration::from_secs(3));
        }
    }
}

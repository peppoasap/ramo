use ramo_core::git::{self, Action, Backend};
use std::{
    fs,
    path::{Path, PathBuf},
};
fn run(root: &Path, args: &[&str]) -> Vec<u8> {
    git::git(root, args).unwrap()
}
fn fixture(base: &[u8]) -> (tempfile::TempDir, PathBuf, Backend, String) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("meadow");
    fs::create_dir(&root).unwrap();
    run(&root, &["init", "-b", "main"]);
    for (key, value) in [
        ("user.name", "Synthetic Contributor"),
        ("user.email", "demo@example.invalid"),
        ("commit.gpgsign", "false"),
        ("core.autocrlf", "false"),
        ("core.fsmonitor", "false"),
    ] {
        run(&root, &["config", key, value]);
    }
    let hooks = root.join(".git/no-hooks");
    fs::create_dir(&hooks).unwrap();
    run(
        &root,
        &["config", "core.hooksPath", hooks.to_str().unwrap()],
    );
    fs::write(root.join("file.txt"), base).unwrap();
    run(&root, &["add", "file.txt"]);
    run(&root, &["commit", "-m", "Base"]);
    let mut backend = Backend::new(temp.path().join("state/repos.json"));
    let s = backend.register(&root).unwrap();
    backend.trust(&s.repo.id).unwrap();
    (temp, root, backend, s.repo.id)
}
fn apply(b: &mut Backend, id: &str, unstage: bool, needles: &[&str]) -> Result<(), String> {
    let snapshot = b.snapshot(id)?;
    let diff = b.file_diff(id, "file.txt", None, unstage)?;
    let rows = diff
        .split_inclusive('\n')
        .enumerate()
        .filter(|(_, l)| needles.contains(l))
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), needles.len());
    b.mutate(
        id,
        &snapshot.fingerprint,
        Action::Lines {
            path: "file.txt".into(),
            rows,
            unstage,
            patch_hash: git::hash(&diff),
        },
    )
    .map(|_| ())
}
#[test]
fn every_subset_of_a_replacement_stages_and_unstages_without_touching_worktree() {
    let (_temp, root, mut b, id) = fixture(b"alpha\nold\nomega\n");
    let work = b"alpha\nnew1\nnew2\nomega\n";
    fs::write(root.join("file.txt"), work).unwrap();
    let head = run(&root, &["rev-parse", "HEAD"]);
    for mask in 1..8 {
        let all = ["-old\n", "+new1\n", "+new2\n"];
        let selected = all
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, s)| *s)
            .collect::<Vec<_>>();
        for unstage in [false, true] {
            run(&root, &["reset", "--mixed", "HEAD"]);
            if unstage {
                run(&root, &["add", "file.txt"]);
            }
            apply(&mut b, &id, unstage, &selected).unwrap();
            let mut expected = String::from("alpha\n");
            if (mask & 1 == 0) != unstage {
                expected.push_str("old\n");
            }
            if (mask & 2 != 0) != unstage {
                expected.push_str("new1\n");
            }
            if (mask & 4 != 0) != unstage {
                expected.push_str("new2\n");
            }
            expected.push_str("omega\n");
            assert_eq!(
                run(&root, &["show", ":file.txt"]),
                expected.as_bytes(),
                "mask={mask}, unstage={unstage}"
            );
            assert_eq!(fs::read(root.join("file.txt")).unwrap(), work);
            assert_eq!(run(&root, &["rev-parse", "HEAD"]), head);
        }
    }
}
#[test]
fn multi_hunk_offsets_crlf_unicode_and_diff_like_content_are_preserved() {
    let base = (0..40)
        .map(|i| format!("line{i:02}\r\n"))
        .collect::<String>();
    let (_temp, root, mut b, id) = fixture(base.as_bytes());
    let work = base
        .replace("line02\r\n", "++日本語\r\nextra\r\n")
        .replace("line35\r\n", "later\r\n");
    fs::write(root.join("file.txt"), &work).unwrap();
    apply(
        &mut b,
        &id,
        false,
        &["-line02\r\n", "+++日本語\r\n", "+later\r\n"],
    )
    .unwrap();
    let expected = base
        .replace("line02\r\n", "++日本語\r\n")
        .replace("line35\r\n", "line35\r\nlater\r\n");
    assert_eq!(run(&root, &["show", ":file.txt"]), expected.as_bytes());
    assert_eq!(fs::read(root.join("file.txt")).unwrap(), work.as_bytes());
    apply(&mut b, &id, true, &["+++日本語\r\n", "+later\r\n"]).unwrap();
    assert_eq!(
        run(&root, &["show", ":file.txt"]),
        base.replace("line02\r\n", "").as_bytes()
    );
}
#[test]
fn empty_tracked_file_range_boundaries_and_full_deletion_work() {
    let (_temp, root, mut b, id) = fixture(b"");
    fs::write(root.join("file.txt"), b"one\ntwo\n").unwrap();
    apply(&mut b, &id, false, &["+two\n"]).unwrap();
    assert_eq!(run(&root, &["show", ":file.txt"]), b"two\n");
    apply(&mut b, &id, true, &["+two\n"]).unwrap();
    assert_eq!(run(&root, &["show", ":file.txt"]), b"");
}
#[test]
fn stale_hash_header_selection_and_missing_eof_fail_without_index_writes() {
    let (_temp, root, mut b, id) = fixture(b"base\n");
    fs::write(root.join("file.txt"), b"changed\n").unwrap();
    let s = b.snapshot(&id).unwrap();
    let diff = b.file_diff(&id, "file.txt", None, false).unwrap();
    let before = run(&root, &["show", ":file.txt"]);
    for (rows, patch_hash) in [(vec![0], git::hash(&diff)), (vec![5], "wrong".into())] {
        assert!(b
            .mutate(
                &id,
                &s.fingerprint,
                Action::Lines {
                    path: "file.txt".into(),
                    rows,
                    unstage: false,
                    patch_hash
                }
            )
            .is_err());
        assert_eq!(run(&root, &["show", ":file.txt"]), before);
    }
    fs::write(root.join("file.txt"), b"missing EOF").unwrap();
    assert!(apply(&mut b, &id, false, &["+missing EOF\n"])
        .unwrap_err()
        .contains("final newline"));
    assert_eq!(run(&root, &["show", ":file.txt"]), before);
}

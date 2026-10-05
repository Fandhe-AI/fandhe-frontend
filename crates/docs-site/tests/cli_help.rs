//! `docs-site --help` / `-h` の CLI 契約（終了コードと出力先）。
//!
//! 正式対応した usage 出力は stdout・exit 0。それ以外の未知の引数は従来どおり
//! stderr に usage を出して非 0 終了する（fail-closed、`main.rs` 冒頭の契約）。

use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_docs-site"))
        .args(args)
        .output()
        .expect("spawn docs-site")
}

#[test]
fn help_flags_print_usage_to_stdout_and_exit_zero() {
    for flag in ["--help", "-h"] {
        let o = run(&[flag]);
        assert_eq!(o.status.code(), Some(0), "{flag}");
        let stdout = String::from_utf8_lossy(&o.stdout);
        assert!(stdout.contains("usage: docs-site"), "{flag}: {stdout}");
        assert!(stdout.contains("--help"), "{flag}");
        assert!(stdout.contains("--no-page-sections"), "{flag}");
        assert!(o.stderr.is_empty(), "{flag}: stderr must be empty");
    }
}

#[test]
fn help_does_not_build_anything() {
    // `--out` と併用しても usage だけを出し、出力先を作らない。
    let out = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cli-help-must-not-exist");
    let _ = std::fs::remove_dir_all(&out);
    let o = run(&["--out", out.to_str().unwrap(), "--help"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(!out.exists());
}

#[test]
fn help_in_value_position_prints_usage_and_builds_nothing() {
    // `--out --help` は `--help` という出力先でのビルドにせず、usage を優先する。
    // 相対パスの `--help` / `-h` が作られないことを、作業ディレクトリを隔離して確かめる。
    let cwd = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cli-help-value-position");
    let _ = std::fs::remove_dir_all(&cwd);
    std::fs::create_dir_all(&cwd).expect("create scratch cwd");
    for args in [
        &["--out", "--help"][..],
        &["--out", "-h"],
        &["--root", "--help", "--out", "dist"],
        &["--root", "-h", "--out", "dist"],
    ] {
        let o = Command::new(env!("CARGO_BIN_EXE_docs-site"))
            .args(args)
            .current_dir(&cwd)
            .output()
            .expect("spawn docs-site");
        assert_eq!(o.status.code(), Some(0), "{args:?}");
        let stdout = String::from_utf8_lossy(&o.stdout);
        assert!(stdout.contains("usage: docs-site"), "{args:?}: {stdout}");
        assert!(o.stderr.is_empty(), "{args:?}: stderr must be empty");
    }
    let leftover: Vec<_> = std::fs::read_dir(&cwd)
        .expect("read scratch cwd")
        .map(|e| e.expect("dir entry").file_name())
        .collect();
    assert!(leftover.is_empty(), "nothing must be written: {leftover:?}");
}

#[test]
fn unknown_arguments_still_fail_with_usage_on_stderr() {
    for args in [&["--bogus"][..], &["--helpx"], &["-help"], &["--help=1"]] {
        let o = run(args);
        assert_ne!(o.status.code(), Some(0), "{args:?}");
        assert!(o.stdout.is_empty(), "{args:?}: stdout must be empty");
        let stderr = String::from_utf8_lossy(&o.stderr);
        assert!(stderr.contains("unknown argument"), "{args:?}: {stderr}");
        assert!(stderr.contains("usage: docs-site"), "{args:?}");
    }
}

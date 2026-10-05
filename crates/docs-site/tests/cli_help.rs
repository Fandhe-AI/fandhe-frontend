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

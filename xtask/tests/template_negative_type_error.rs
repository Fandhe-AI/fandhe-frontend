//! `templates/default/tests/negative_type_error.rs`（TASK-4.4 / REQ-4）に対する
//! 多層防御の回帰テスト。
//!
//! `templates/default` は root workspace（リポジトリ直下 `Cargo.toml` の
//! members）に含まれないため、`cargo test --workspace` だけでは成果物テスト
//! 自体の実行漏れ・削除・弱体化を検知できない。本ファイルは xtask 側
//! （workspace メンバー）から成果物ファイルの実在と必須アサーションの記述を
//! 静的に検証し、TASK-4.1 の `template_deny_config.rs` と同一パターンの
//! 安全網として機能する。
//!
//! 外部 TOML/Rust パーサは追加しない（REQ-3・xtask 外部依存ゼロ方針。依存
//! 追加のユーザー承認を得られない自動運転では「追加しない」側に倒す）。
//! そのため検証は行・部分文字列に対する静的チェックに留める。
//! `#[ignore]` は使わない（静的検証は常時実行する）。

use std::path::PathBuf;

/// workspace ルート（`xtask/` の親ディレクトリ）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask/ には親ディレクトリ（workspace ルート）が存在する")
        .to_path_buf()
}

fn template_dir() -> PathBuf {
    workspace_root().join("templates/default")
}

#[test]
fn template_project_skeleton_exists() {
    for rel in ["Cargo.toml", "src/main.rs", "tests/negative_type_error.rs"] {
        let path = template_dir().join(rel);
        assert!(
            path.is_file(),
            "TASK-4.4 の成果物に必要なファイルが見つからない: {}",
            path.display()
        );
    }
}

#[test]
fn template_cargo_toml_is_detached_from_root_workspace() {
    let contents = std::fs::read_to_string(template_dir().join("Cargo.toml"))
        .expect("templates/default/Cargo.toml の読み込みに失敗した");
    assert!(
        contents.contains("[workspace]"),
        "templates/default/Cargo.toml に空の [workspace] テーブルがない。 \
         root workspace の members に巻き込まれると、独立した cargo \
         プロジェクトとして配布したときの `cargo check`/`cargo test` の \
         挙動を検証できなくなる: {contents}"
    );
}

#[test]
fn template_main_rs_forbids_unsafe_code() {
    let contents = std::fs::read_to_string(template_dir().join("src/main.rs"))
        .expect("templates/default/src/main.rs の読み込みに失敗した");
    assert!(
        contents.contains("#![forbid(unsafe_code)]"),
        "templates/default/src/main.rs に #![forbid(unsafe_code)] がない \
         （REQ-2 / coding-rust.md）: {contents}"
    );
}

#[test]
fn negative_type_error_test_covers_required_error_codes_and_offline_check() {
    let contents = std::fs::read_to_string(template_dir().join("tests/negative_type_error.rs"))
        .expect("templates/default/tests/negative_type_error.rs の読み込みに失敗した");

    for required in ["cargo", "check", "--offline", "E0277", "E0308", "#[test]"] {
        assert!(
            contents.contains(required),
            "negative_type_error.rs の必須要素 `{required}` が見つからない \
             （REQ-4 受け入れ基準 3 の機械的検証の弱体化を検知）"
        );
    }

    // baseline（正例）テストが削除されると、負例側の失敗が本当に注入した
    // 型不正に起因するのか、環境要因（cargo 未インストール等）なのかを
    // 切り分けられなくなるため、対照群テストの実在も固定する。
    assert!(
        contents.contains("baseline"),
        "正例ベースラインテストが negative_type_error.rs から失われている \
         （対照群がないと負例失敗の原因切り分けができなくなる）"
    );
}

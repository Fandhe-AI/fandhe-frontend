//! `fw new`（TASK-13.4 相当、イシュー #350）が展開する標準プロジェクト
//! テンプレートのコンパイル時埋め込みマニフェスト。
//!
//! `fw` は単一実行ファイル配布（Docker 想定）を目標とするため、実行時に
//! `templates/default/` のファイルシステム配置へ依存させず、
//! `include_str!` でバイナリへ埋め込む。正本は引き続き `templates/default/`
//! （`xtask/tests/template_*.rs` が正本として参照する）であり、本ファイルは
//! その写しにすぎない。両者の乖離は `cli/tests/new_e2e.rs` の
//! ドリフト検知テストが `templates/default/` を再帰走査して機械的に検出する
//! （手動同期に頼らない。`.claude/rules/ci.md` の cargo-deny pin ドリフト検知と
//! 同じ運用方針）。
//!
//! `new.rs::run_new` から呼ばれ、[`TEMPLATE_FILES`] の配列順（固定）で
//! 展開することが決定性（同一入力 → バイト単位で同一出力）を担保する。

/// テンプレート 1 ファイル分のコンパイル時定数。
///
/// `rel_path` はテンプレートルート（`templates/default/`）からの相対パスで、
/// `new.rs` がターゲットディレクトリと結合してファイルを書き出す。
/// `executable` は git の実行ビット（mode 100755）をそのまま反映したもので、
/// Unix では `new.rs` がこれをもとに 0o755 を明示設定する。
pub(crate) struct TemplateFile {
    pub(crate) rel_path: &'static str,
    pub(crate) contents: &'static str,
    pub(crate) executable: bool,
}

/// `templates/default/` の全ファイル（12 件）を git の相対パス順・実行ビット
/// どおりに埋め込んだ固定配列。
///
/// 展開順はこの配列順であり、`fw new` の出力 JSON の `files` 一覧も同じ順序で
/// 並べる契約とする（`new.rs::run_new` 参照）。
pub(crate) const TEMPLATE_FILES: &[TemplateFile] = &[
    TemplateFile {
        rel_path: ".github/workflows/deny.yml",
        contents: include_str!("../../templates/default/.github/workflows/deny.yml"),
        executable: false,
    },
    TemplateFile {
        rel_path: ".github/workflows/npm-asset-gate.yml",
        contents: include_str!("../../templates/default/.github/workflows/npm-asset-gate.yml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "Cargo.lock",
        contents: include_str!("../../templates/default/Cargo.lock"),
        executable: false,
    },
    TemplateFile {
        rel_path: "Cargo.toml",
        contents: include_str!("../../templates/default/Cargo.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "clippy.toml",
        contents: include_str!("../../templates/default/clippy.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "deny.toml",
        contents: include_str!("../../templates/default/deny.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "src/main.rs",
        contents: include_str!("../../templates/default/src/main.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tests/negative_type_error.rs",
        contents: include_str!("../../templates/default/tests/negative_type_error.rs"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/allowlist.toml",
        contents: include_str!("../../templates/default/tools/npm-asset-build/allowlist.toml"),
        executable: false,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/apply_exempt.py",
        contents: include_str!("../../templates/default/tools/npm-asset-build/apply_exempt.py"),
        executable: true,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/check_static_only.py",
        contents: include_str!(
            "../../templates/default/tools/npm-asset-build/check_static_only.py"
        ),
        executable: true,
    },
    TemplateFile {
        rel_path: "tools/npm-asset-build/install.sh",
        contents: include_str!("../../templates/default/tools/npm-asset-build/install.sh"),
        executable: true,
    },
];

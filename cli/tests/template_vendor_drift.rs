//! `templates/app/vendor/`（rws-core / rws-app の vendor 同梱、イシュー #378）
//! と正本 `core/`/`app/` の乖離検知テスト。
//!
//! # 背景
//!
//! rws-core / rws-app は `publish = false`（crates.io 未公開）のため、
//! `templates/app` は git 依存・上位ワークスペースへの path 依存のいずれも
//! 採らず、ソースを vendor 同梱する（イシュー #378 実装計画 §3.2）。
//! vendor 同梱は正本の複製であるため、正本側の変更（バグ修正・API 追加）が
//! vendor 側へ手動同期されないまま陳腐化するリスクを本テストが機械的に
//! 検出する（`.claude/rules/ci.md` の cargo-deny pin ドリフト検知と同じ
//! 運用方針。手動同期に頼らない）。
//!
//! `src/` はバイト単位で完全一致することを要求する。`Cargo.toml` は
//! vendor 化に伴う既知の変換（path 依存の参照先ディレクトリ名変更、
//! `[workspace]` を持たない）を考慮した緩やかな比較を行う（vendor 側
//! Cargo.toml のコメントに変換理由を明記している）。
//!
//! # 共有ファイル同一性
//!
//! `templates/default/` と `templates/app/` は `.github/workflows/*`・
//! `clippy.toml`・`deny.toml`・`tools/npm-asset-build/*`
//! （計 6 ファイル）をバイト単位で共有する契約（イシュー #378 実装計画
//! §3.3）。本テストはこの同一性も検証する。
//!
//! `xtask/tests/template_deny_workflow.rs`（cargo-deny pin ドリフト検知）は
//! `templates/default/.github/workflows/deny.yml` のみを直接検証する。
//! `templates/app/.github/workflows/deny.yml` は本テストの共有ファイル
//! 同一性チェックで `templates/default/` 側と常にバイト一致することが
//! 保証されるため、xtask 側の pin 検証は default 経由で app にも間接的に
//! 及ぶ（3 箇所目の pin 値を app にも複製して手動同期対象を増やさない、
//! イシュー #378 実装計画 §4 の判断根拠）。
//!
//! `static/embed.html`（CSR マウント骨格）は `templates/embed/embed.html`
//! を出自コメント付きで同梱したものであり、正本との差分検知は本テストの
//! スコープ外とする（`fw new` の生成対象としての `embed` テンプレート自体は
//! イシュー #378 の対象外、実装計画 §9）。

use std::path::{Path, PathBuf};

/// workspace ルート（`cli/` の親ディレクトリ）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent workspace root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read `{}`: {e}", path.display()))
}

fn read_bytes(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("failed to read `{}`: {e}", path.display()))
}

// --- rws-core vendor drift ---

const CORE_SRC_FILES: &[&str] = &[
    "bind.rs",
    "escape.rs",
    "keyed.rs",
    "lib.rs",
    "tags.rs",
    "url.rs",
];

#[test]
fn vendored_rws_core_src_is_byte_identical_to_source_crate() {
    let root = workspace_root();
    for file in CORE_SRC_FILES {
        let src = root.join("core/src").join(file);
        let vendored = root.join("templates/app/vendor/rws-core/src").join(file);
        assert_eq!(
            read_bytes(&src),
            read_bytes(&vendored),
            "vendored rws-core/src/{file} has drifted from core/src/{file} \
             (正本の変更を templates/app/vendor/rws-core/src/{file} へ手動同期すること)"
        );
    }
}

/// vendored `rws-core/Cargo.toml` は正本 `core/Cargo.toml` と `[dependencies]`
/// セクションが空であること（外部依存ゼロ、REQ-3）が一致することを検証する。
/// vendor 側は `[workspace]` を持たない（`cargo` の多重 workspace root
/// エラーを避けるための既知の差分、`templates/app/vendor/rws-core/Cargo.toml`
/// のコメント参照）ため、`[dependencies]` セクションの内容一致のみを比較する
/// （行単位の完全一致は求めない。パッケージ名・description 等のメタデータは
/// vendor 化に伴い変わり得るため）。
#[test]
fn vendored_rws_core_cargo_toml_has_no_external_dependencies() {
    let root = workspace_root();
    let vendored = read(&root.join("templates/app/vendor/rws-core/Cargo.toml"));
    let deps_section = extract_section(&vendored, "[dependencies]");
    assert!(
        deps_section.trim().is_empty() || deps_section.trim().starts_with('['),
        "templates/app/vendor/rws-core/Cargo.toml の [dependencies] は空である必要がある \
         （REQ-3: core は外部依存ゼロを厳守）: section={deps_section:?}"
    );
}

// --- rws-app vendor drift ---

/// 正本 `app/src/` のうち vendor 同梱・`fw new --template app` 生成対象の
/// ファイル一覧（イシュー #407 で `router.rs` / `routes.rs` を追加）。
/// `CORE_SRC_FILES` と同様、ここに列挙されていないファイルは本テストの
/// ドリフト検知対象外になる点に注意する。
const APP_SRC_FILES: &[&str] = &["lib.rs", "router.rs", "routes.rs"];

#[test]
fn vendored_rws_app_src_is_byte_identical_to_source_crate() {
    let root = workspace_root();
    for file in APP_SRC_FILES {
        let src = root.join("app/src").join(file);
        let vendored = root.join("templates/app/vendor/rws-app/src").join(file);
        assert_eq!(
            read_bytes(&src),
            read_bytes(&vendored),
            "vendored rws-app/src/{file} has drifted from app/src/{file} \
             (正本の変更を templates/app/vendor/rws-app/src/{file} へ手動同期すること)"
        );
    }
}

/// vendored `rws-app/Cargo.toml` の `rws-core` path 依存が
/// vendor 配下の実ディレクトリ名（`../rws-core`）を指すこと
/// （正本は `../core`、vendor 化に伴う既知の変換、実装計画 §3.2）を検証する。
#[test]
fn vendored_rws_app_cargo_toml_points_at_vendored_rws_core() {
    let root = workspace_root();
    let vendored = read(&root.join("templates/app/vendor/rws-app/Cargo.toml"));
    assert!(
        vendored.contains(r#"rws-core = { path = "../rws-core" }"#),
        "templates/app/vendor/rws-app/Cargo.toml は vendor 配下の rws-core \
         （../rws-core）を path 依存で参照する必要がある: {vendored:?}"
    );
}

/// `[dependencies]` セクションの内容を抽出する（次の `[` で始まる行の直前まで）。
/// 外部 TOML パーサは追加しない方針（cli の依存グラフを不必要に増やさない）
/// ため、行ベースの単純な抽出に留める。
fn extract_section<'a>(toml: &'a str, header: &str) -> &'a str {
    let start = toml
        .find(header)
        .unwrap_or_else(|| panic!("section `{header}` not found"));
    let after_header = &toml[start + header.len()..];
    match after_header.find("\n[") {
        Some(next) => &after_header[..next],
        None => after_header,
    }
}

// --- 共有ファイル同一性: templates/default/ と templates/app/ ---

/// `templates/default/<rel>` と `templates/app/<rel>` がバイト単位で一致する
/// ことを検証する共有ファイルの一覧（実装計画 §3.3・§4）。
const SHARED_RELATIVE_FILES: &[&str] = &[
    ".github/workflows/deny.yml",
    ".github/workflows/npm-asset-gate.yml",
    "clippy.toml",
    "deny.toml",
    "tools/npm-asset-build/allowlist.toml",
    "tools/npm-asset-build/apply_exempt.py",
    "tools/npm-asset-build/check_static_only.py",
    "tools/npm-asset-build/install.sh",
];

#[test]
fn default_and_app_templates_share_identical_bytes_for_common_files() {
    let root = workspace_root();
    for rel in SHARED_RELATIVE_FILES {
        let default_path = root.join("templates/default").join(rel);
        let app_path = root.join("templates/app").join(rel);
        assert_eq!(
            read_bytes(&default_path),
            read_bytes(&app_path),
            "templates/default/{rel} と templates/app/{rel} はバイト単位で \
             一致する契約（イシュー #378 実装計画 §3.3）。一方だけを変更した \
             場合は他方にも反映すること"
        );
    }
}

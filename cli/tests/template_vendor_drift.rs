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

// --- rws-interactive vendor drift（イシュー #411） ---

#[test]
fn vendored_rws_interactive_src_is_byte_identical_to_source_crate() {
    let root = workspace_root();
    let src = root.join("interactive/src/lib.rs");
    let vendored = root.join("templates/app/vendor/rws-interactive/src/lib.rs");
    assert_eq!(
        read_bytes(&src),
        read_bytes(&vendored),
        "vendored rws-interactive/src/lib.rs has drifted from interactive/src/lib.rs \
         (正本の変更を templates/app/vendor/rws-interactive/src/lib.rs へ手動同期すること)"
    );
}

/// vendored `rws-interactive/Cargo.toml` の `rws-core` path 依存が
/// vendor 配下の実ディレクトリ名（`../rws-core`）を指すこと
/// （正本は `../core`、vendor 化に伴う既知の変換）を検証する。
#[test]
fn vendored_rws_interactive_cargo_toml_points_at_vendored_rws_core() {
    let root = workspace_root();
    let vendored = read(&root.join("templates/app/vendor/rws-interactive/Cargo.toml"));
    assert!(
        vendored.contains(r#"rws-core = { path = "../rws-core" }"#),
        "templates/app/vendor/rws-interactive/Cargo.toml は vendor 配下の rws-core \
         （../rws-core）を path 依存で参照する必要がある: {vendored:?}"
    );
}

// --- rws-wasm-client vendor drift（イシュー #411） ---

const WASM_CLIENT_SRC_FILES: &[&str] = &[
    "binding.rs",
    "binding_dom.rs",
    "keyed_diff.rs",
    "keyed_dom.rs",
    "lib.rs",
    "registry.rs",
];

#[test]
fn vendored_rws_wasm_client_src_is_byte_identical_to_source_crate() {
    let root = workspace_root();
    for file in WASM_CLIENT_SRC_FILES {
        let src = root.join("wasm-client/src").join(file);
        let vendored = root
            .join("templates/app/vendor/rws-wasm-client/src")
            .join(file);
        assert_eq!(
            read_bytes(&src),
            read_bytes(&vendored),
            "vendored rws-wasm-client/src/{file} has drifted from wasm-client/src/{file} \
             (正本の変更を templates/app/vendor/rws-wasm-client/src/{file} へ手動同期すること)"
        );
    }
}

/// vendored `rws-wasm-client/Cargo.toml` の path 依存が vendor 配下の
/// 実ディレクトリ名（`../rws-core`・`../rws-app`・`../rws-interactive`）を
/// 指すこと（正本は `../core`・`../app`・`../interactive`、vendor 化に伴う
/// 既知の変換）と、`[dev-dependencies]` を持たないこと（実装計画 §2.3:
/// rws-server への vendor 連鎖を断つ意図的な除去）を検証する。
#[test]
fn vendored_rws_wasm_client_cargo_toml_points_at_vendored_paths_and_has_no_dev_dependencies() {
    let root = workspace_root();
    let vendored = read(&root.join("templates/app/vendor/rws-wasm-client/Cargo.toml"));
    for expected in [
        r#"rws-core = { path = "../rws-core" }"#,
        r#"rws-app = { path = "../rws-app" }"#,
        r#"rws-interactive = { path = "../rws-interactive" }"#,
    ] {
        assert!(
            vendored.contains(expected),
            "templates/app/vendor/rws-wasm-client/Cargo.toml must contain `{expected}`: {vendored:?}"
        );
    }
    assert!(
        !vendored.contains("[dev-dependencies]"),
        "templates/app/vendor/rws-wasm-client/Cargo.toml must not declare [dev-dependencies] \
         (dev-dependencies pulls in rws-server, breaking the vendor scope, 実装計画 §2.3): {vendored:?}"
    );
}

// --- wasm/Cargo.lock の wasm-bindgen / web-sys バージョン整合（イシュー #411） ---

/// `templates/app/wasm/Cargo.lock` の wasm-bindgen / web-sys バージョンが
/// リポジトリ本体 `Cargo.lock` の解決値と同一であることを検証する
/// （REQ-3 整理: 新規外部クレート追加ゼロ、既存解決値の参照のみという
/// 前提が崩れていないことの機械的検証。手動同期に頼らない、実装計画 §3）。
#[test]
fn wasm_lockfile_wasm_bindgen_version_matches_repo_root_lockfile() {
    let root = workspace_root();
    let repo_lock = read(&root.join("Cargo.lock"));
    let wasm_lock = read(&root.join("templates/app/wasm/Cargo.lock"));

    for pkg in ["wasm-bindgen", "web-sys"] {
        let repo_version = package_version(&repo_lock, pkg);
        let wasm_version = package_version(&wasm_lock, pkg);
        assert_eq!(
            repo_version, wasm_version,
            "templates/app/wasm/Cargo.lock の `{pkg}` バージョンがリポジトリ本体 Cargo.lock \
             と乖離している（新規外部クレート追加ゼロという REQ-3 整理の前提が崩れている \
             可能性がある。手動同期すること）"
        );
    }
}

/// `Cargo.lock` 内の `[[package]] name = "<pkg>"` に対応する `version` を
/// 抽出する（外部 TOML パーサは追加しない方針、行ベースの単純な抽出）。
fn package_version(lockfile: &str, pkg: &str) -> String {
    let marker = format!("name = \"{pkg}\"\n");
    let start = lockfile
        .find(&marker)
        .unwrap_or_else(|| panic!("package `{pkg}` not found in lockfile"));
    let after = &lockfile[start + marker.len()..];
    let version_line = after
        .lines()
        .next()
        .unwrap_or_else(|| panic!("no line after `name = \"{pkg}\"` in lockfile"));
    version_line
        .strip_prefix("version = \"")
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or_else(|| panic!("unexpected version line for `{pkg}`: {version_line:?}"))
        .to_string()
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

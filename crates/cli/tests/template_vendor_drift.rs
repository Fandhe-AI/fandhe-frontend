//! `templates/app` の crates.io バージョン依存（fandhe-frontend-core /
//! fandhe-frontend-app / fandhe-frontend-interactive / fandhe-frontend-wasm-client、
//! イシュー #412 で vendor 同梱から切替）の整合性検知テスト。
//!
//! # 背景
//!
//! `templates/app` は当初、fandhe-frontend-core / fandhe-frontend-app が
//! `publish = false`（crates.io 未公開）であったため、ソースを vendor 同梱
//! （`templates/app/vendor/`）して path 依存させていた（イシュー #378）。
//! 全 9 クレートが crates.io へ v0.1.0 で公開されたことを受け、イシュー #412
//! （`docs/design/template-vendor-to-version-switch.md`）の切替手順に従い、
//! vendor 同梱を廃止し通常の crates.io バージョン依存へ切り替えた。
//!
//! 本ファイルはこのファイル名が指す検知対象を「vendor 同梱の drift」から
//! 「バージョン依存の整合性・vendor 同梱の再発防止」へ更新する
//! （ファイル名自体は `new_template.rs`/`template_publish_copy_drift.rs` 等
//! 他ファイルからの参照を保つため維持する）。検知能力は弱体化させない
//! （`.claude/rules/coding-rust.md`）:
//!
//! - `templates/app/Cargo.toml`・`templates/app/wasm/Cargo.toml` が
//!   フレームワーク側クレートを path 依存で宣言していないこと
//! - それらのバージョン依存が正本 `crates/*/Cargo.toml` の `version` と
//!   一致すること（正本がバージョンアップした際にテンプレートが追随せず
//!   陳腐化するのを機械検知する。手動同期に頼らない）
//! - `templates/*/vendor/` ディレクトリが存在しないこと（vendor 同梱の再発を
//!   防止する）
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

/// workspace ルート（`cli/` の親の親ディレクトリ）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    // このテストバイナリは `crates/cli/` 配下でビルドされるため、2 段の
    // 親ディレクトリを辿る（イシュー #436）。
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/cli/ has a workspace root two levels up")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read `{}`: {e}", path.display()))
}

fn read_bytes(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("failed to read `{}`: {e}", path.display()))
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

/// 正本 `Cargo.toml` の `[package]` セクションから `version` の値を抽出する。
fn package_version(manifest_toml: &str) -> String {
    let section = extract_section(manifest_toml, "[package]");
    for line in section.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("version") {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                let rest = rest.trim();
                if let Some(v) = rest.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                    return v.to_string();
                }
            }
        }
    }
    panic!("`version` not found in [package] section: {section:?}");
}

/// `manifest_toml` 内で `dep_name = "X.Y.Z"`（バージョン文字列の直接指定）
/// として宣言された依存のバージョンを返す。
///
/// `{ path = "..." }` のようなテーブル形式（path 依存・vendor 同梱の典型的な
/// 書き方）は本関数のパターンにマッチしないため、path 依存が再導入された
/// 場合はここで panic し fail-closed に検知する（vendor 同梱の再発防止）。
fn version_dependency(manifest_toml: &str, dep_name: &str) -> String {
    let prefix = format!("{dep_name} = \"");
    for line in manifest_toml.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(prefix.as_str()) {
            if let Some(end) = rest.find('"') {
                return rest[..end].to_string();
            }
        }
    }
    panic!(
        "`{dep_name}` はバージョン文字列依存（`{dep_name} = \"X.Y.Z\"`）として \
         宣言されている必要がある（path 依存・vendor 同梱への回帰の可能性）: \
         {manifest_toml:?}"
    );
}

// --- templates/app のバージョン依存整合性（イシュー #412） ---

#[test]
fn templates_app_cargo_toml_declares_version_dependency_matching_source_crates() {
    let root = workspace_root();
    let app_manifest = read(&root.join("templates/app/Cargo.toml"));

    for (dep_name, source_manifest_rel) in [
        ("fandhe-frontend-core", "crates/core/Cargo.toml"),
        ("fandhe-frontend-app", "crates/app/Cargo.toml"),
    ] {
        let source_manifest = read(&root.join(source_manifest_rel));
        let expected_version = package_version(&source_manifest);
        let declared_version = version_dependency(&app_manifest, dep_name);
        assert_eq!(
            declared_version, expected_version,
            "templates/app/Cargo.toml の `{dep_name}` バージョン依存が正本 \
             {source_manifest_rel} の version（{expected_version}）と乖離している \
             （正本のバージョンアップをテンプレートへ反映すること）"
        );
    }
}

#[test]
fn templates_app_wasm_cargo_toml_declares_version_dependency_matching_source_crate() {
    let root = workspace_root();
    let wasm_manifest = read(&root.join("templates/app/wasm/Cargo.toml"));
    let source_manifest = read(&root.join("crates/wasm-client/Cargo.toml"));
    let expected_version = package_version(&source_manifest);
    let declared_version = version_dependency(&wasm_manifest, "fandhe-frontend-wasm-client");
    assert_eq!(
        declared_version, expected_version,
        "templates/app/wasm/Cargo.toml の `fandhe-frontend-wasm-client` バージョン \
         依存が正本 crates/wasm-client/Cargo.toml の version（{expected_version}）と \
         乖離している（正本のバージョンアップをテンプレートへ反映すること）"
    );
}

/// `templates/*/vendor/` ディレクトリが存在しないことを検証する
/// （vendor 同梱（イシュー #378）から crates.io バージョン依存（イシュー #412）
/// への切替が完了した状態を固定し、vendor 同梱の再発を防止する）。
#[test]
fn no_template_vendors_local_crate_sources() {
    let root = workspace_root();
    let templates_root = root.join("templates");
    let template_dirs = std::fs::read_dir(&templates_root)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", templates_root.display()));
    for entry in template_dirs.flatten() {
        let vendor_dir = entry.path().join("vendor");
        assert!(
            !vendor_dir.exists(),
            "{} が存在する。テンプレートはフレームワーク側クレートを vendor \
             同梱せず、crates.io バージョン依存を使う方針（イシュー #412、\
             docs/design/template-vendor-to-version-switch.md）",
            vendor_dir.display()
        );
    }
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
        let repo_version = package_lockfile_version(&repo_lock, pkg);
        let wasm_version = package_lockfile_version(&wasm_lock, pkg);
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
fn package_lockfile_version(lockfile: &str, pkg: &str) -> String {
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

// --- 共有ファイル同一性: templates/default/ と examples/ssr-routing/（イシュー #500） ---

/// `templates/default/<rel>` と `examples/ssr-routing/<rel>` がバイト単位で
/// 一致することを検証する共有ファイルの一覧（イシュー #500 実装計画 §3。
/// `examples/` は `templates/` とは独立した規約だが、静的解析設定
/// （`clippy.toml`）・サプライチェーン設定（`deny.toml`）は流用する契約）。
const SSR_ROUTING_SHARED_RELATIVE_FILES: &[&str] = &["clippy.toml", "deny.toml"];

#[test]
fn default_template_and_ssr_routing_example_share_identical_bytes_for_common_files() {
    let root = workspace_root();
    for rel in SSR_ROUTING_SHARED_RELATIVE_FILES {
        let default_path = root.join("templates/default").join(rel);
        let example_path = root.join("examples/ssr-routing").join(rel);
        assert_eq!(
            read_bytes(&default_path),
            read_bytes(&example_path),
            "templates/default/{rel} と examples/ssr-routing/{rel} はバイト単位で \
             一致する契約（イシュー #500 実装計画 §3）。一方だけを変更した \
             場合は他方にも反映すること"
        );
    }
}

// --- 共有ファイル同一性: templates/default/ と examples/ssg-blog/（イシュー #501） ---

/// `templates/default/<rel>` と `examples/ssg-blog/<rel>` がバイト単位で
/// 一致することを検証する共有ファイルの一覧（`SSR_ROUTING_SHARED_RELATIVE_FILES`
/// と同じ静的解析設定・サプライチェーン設定の流用契約、イシュー #501）。
const SSG_BLOG_SHARED_RELATIVE_FILES: &[&str] = &["clippy.toml", "deny.toml"];

#[test]
fn default_template_and_ssg_blog_example_share_identical_bytes_for_common_files() {
    let root = workspace_root();
    for rel in SSG_BLOG_SHARED_RELATIVE_FILES {
        let default_path = root.join("templates/default").join(rel);
        let example_path = root.join("examples/ssg-blog").join(rel);
        assert_eq!(
            read_bytes(&default_path),
            read_bytes(&example_path),
            "templates/default/{rel} と examples/ssg-blog/{rel} はバイト単位で \
             一致する契約（イシュー #501）。一方だけを変更した場合は他方にも \
             反映すること"
        );
    }
}

/// `templates/default/` と `examples/dist-server-docker/` の間でバイト単位で
/// 一致することを検証する共有ファイルの一覧（イシュー #502 実装計画 §3。
/// `SSR_ROUTING_SHARED_RELATIVE_FILES` と同じ契約を 2 例目のサンプルにも
/// 適用する）。
const DIST_SERVER_DOCKER_SHARED_RELATIVE_FILES: &[&str] = &["clippy.toml", "deny.toml"];

#[test]
fn default_template_and_dist_server_docker_example_share_identical_bytes_for_common_files() {
    let root = workspace_root();
    for rel in DIST_SERVER_DOCKER_SHARED_RELATIVE_FILES {
        let default_path = root.join("templates/default").join(rel);
        let example_path = root.join("examples/dist-server-docker").join(rel);
        assert_eq!(
            read_bytes(&default_path),
            read_bytes(&example_path),
            "templates/default/{rel} と examples/dist-server-docker/{rel} は \
             バイト単位で一致する契約（イシュー #502 実装計画 §3）。一方だけを \
             変更した場合は他方にも反映すること"
        );
    }
}

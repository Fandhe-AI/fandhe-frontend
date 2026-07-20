//! `crates/cli/templates/`（`fandhe-frontend-cli` の crates.io 公開のための
//! 同梱コピー）と正本 `templates/`（リポジトリルート）の乖離検知テスト。
//!
//! # 背景
//!
//! `crates/cli/src/new_template.rs` は `include_str!` でテンプレート群を
//! コンパイル時埋め込みするが、`include_str!` はクレートディレクトリ
//! （`crates/cli/`）の外を参照できない。ルート `templates/` を直接参照すると
//! `cargo package` / `cargo publish` の tarball 検証（クレートディレクトリ外
//! ファイルの同梱禁止）が失敗するため、`crates/cli/templates/` へ正本の
//! バイト単位同梱コピーを置いている（`templates/default/tools/npm-asset-build/`
//! が `tools/npm-asset-build/` の同梱コピーであるのと同じ「正本 + 同梱コピー
//! + ドリフト検知テスト」運用、イシュー #316）。
//!
//! 本テストは正本 `templates/{default,app,embed}/` と同梱コピー
//! `crates/cli/templates/{default,app,embed}/` を再帰走査し、両者のファイル
//! 集合とバイト内容が完全一致することを検証する。手動同期に頼らない
//! （`.claude/rules/ci.md` の cargo-deny pin ドリフト検知と同じ運用方針）。
//! `crates/cli/templates/README.md`（本コピーの出自を説明する追加ファイル、
//! 正本 `templates/` には存在しない）は比較対象から除外する。
//!
//! 同梱コピー配下の `Cargo.toml` は `Cargo.toml.embed` にリネームしている
//! （`cargo package` がネストした `Cargo.toml` を検出すると tarball の
//! ファイル列挙から機械的に除外する挙動を回避するため、`src/new_template.rs`
//! のドキュメントコメント参照）。本テストはファイル名の末尾 `.embed` を
//! 除去した論理パスで正本側と比較し、内容はバイト単位一致を要求する。

use std::collections::BTreeSet;
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

/// `root` 配下の全ファイルの相対パス集合を再帰的に収集する
/// （ディレクトリ自体は含めない。fail-closed: 読み取りエラーはテスト失敗と
/// して顕在化させる）。
fn collect_relative_files(root: &Path) -> BTreeSet<PathBuf> {
    let mut out = BTreeSet::new();
    collect_relative_files_into(root, root, &mut out);
    out
}

fn collect_relative_files_into(base: &Path, dir: &Path, out: &mut BTreeSet<PathBuf>) {
    let entries =
        std::fs::read_dir(dir).unwrap_or_else(|e| panic!("failed to read {}: {e}", dir.display()));
    for entry in entries {
        let entry =
            entry.unwrap_or_else(|e| panic!("failed to read entry in {}: {e}", dir.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_relative_files_into(base, &path, out);
        } else {
            let rel = path
                .strip_prefix(base)
                .expect("entry path must be under base")
                .to_path_buf();
            out.insert(rel);
        }
    }
}

/// `.embed` サフィックスが付いたファイル名（`Cargo.toml.embed` 等）を
/// 正本側のファイル名（`Cargo.toml`）へ正規化した論理相対パスを返す
/// （`cargo package` のネスト `Cargo.toml` 除外回避のためのリネーム、
/// `src/new_template.rs` のドキュメントコメント参照）。
fn to_logical_source_path(copy_rel: &Path) -> PathBuf {
    match copy_rel.file_name().and_then(|n| n.to_str()) {
        Some(name) if name.ends_with(".embed") => {
            let logical_name = &name[..name.len() - ".embed".len()];
            copy_rel.with_file_name(logical_name)
        }
        _ => copy_rel.to_path_buf(),
    }
}

/// 正本 `templates/<name>/` と同梱コピー `crates/cli/templates/<name>/` の
/// ファイル集合・バイト内容が完全一致することを検証する。
fn assert_copy_matches_source(template_name: &str) {
    let root = workspace_root();
    let source_dir = root.join("templates").join(template_name);
    let copy_dir = root.join("crates/cli/templates").join(template_name);

    let source_files = collect_relative_files(&source_dir);
    let copy_files_raw = collect_relative_files(&copy_dir);
    // 論理パス（`.embed` サフィックス除去後）をキーに、実際のコピー側相対
    // パスを引けるようにする。2 つの異なるコピー側パスが同じ論理パスへ
    // 縮退することは想定しない（`Cargo.toml` と `Cargo.toml.embed` が
    // 同一ディレクトリに共存するケースは正本側に存在しないため発生しない）。
    let copy_files_logical: BTreeSet<PathBuf> = copy_files_raw
        .iter()
        .map(|p| to_logical_source_path(p))
        .collect();

    assert_eq!(
        source_files, copy_files_logical,
        "templates/{template_name}/ と crates/cli/templates/{template_name}/ の \
         ファイル集合が一致しない（正本の変更を同梱コピーへ反映するか、\
         crates/cli/templates/README.md の手順に従って再同期すること）"
    );

    for copy_rel in &copy_files_raw {
        let source_rel = to_logical_source_path(copy_rel);
        let source_bytes = std::fs::read(source_dir.join(&source_rel)).unwrap_or_else(|e| {
            panic!(
                "failed to read {}: {e}",
                source_dir.join(&source_rel).display()
            )
        });
        let copy_bytes = std::fs::read(copy_dir.join(copy_rel)).unwrap_or_else(|e| {
            panic!("failed to read {}: {e}", copy_dir.join(copy_rel).display())
        });
        assert_eq!(
            source_bytes,
            copy_bytes,
            "crates/cli/templates/{template_name}/{} has drifted from templates/{template_name}/{} \
             (正本の変更を crates/cli/templates/{template_name}/{} へ手動同期すること)",
            copy_rel.display(),
            source_rel.display(),
            copy_rel.display()
        );
    }
}

#[test]
fn default_template_publish_copy_matches_source() {
    assert_copy_matches_source("default");
}

#[test]
fn app_template_publish_copy_matches_source() {
    assert_copy_matches_source("app");
}

#[test]
fn embed_template_publish_copy_matches_source() {
    assert_copy_matches_source("embed");
}

/// `crates/cli/templates/` 直下に正本には存在しない補助ファイル
/// （`README.md`）以外の想定外ファイルが紛れ込んでいないことを検証する
/// （fail-closed: 同梱コピーの範囲が静かに広がるのを防ぐ）。
#[test]
fn publish_copy_root_has_no_unexpected_top_level_entries() {
    let root = workspace_root();
    let copy_root = root.join("crates/cli/templates");
    let entries = std::fs::read_dir(&copy_root)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", copy_root.display()));
    let mut names: Vec<String> = entries
        .map(|e| {
            e.unwrap_or_else(|e| panic!("failed to read entry: {e}"))
                .file_name()
                .to_string_lossy()
                .to_string()
        })
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "README.md".to_string(),
            "app".to_string(),
            "default".to_string(),
            "embed".to_string(),
        ],
        "crates/cli/templates/ 直下に想定外のエントリがある: {names:?}"
    );
}

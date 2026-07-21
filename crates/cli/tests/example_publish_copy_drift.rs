//! `crates/cli/embedded-examples/`（`fw new --example` が展開するサンプル群の
//! crates.io 公開のための同梱コピー）と正本 `examples/`（リポジトリルート）の
//! 乖離検知テスト（イシュー #500）。
//!
//! # 背景
//!
//! `crates/cli/src/new_template.rs` は `include_str!` でサンプル群を
//! コンパイル時埋め込みするが、`include_str!` はクレートディレクトリ
//! （`crates/cli/`）の外を参照できない。ルート `examples/` を直接参照すると
//! `cargo package` / `cargo publish` の tarball 検証（クレートディレクトリ外
//! ファイルの同梱禁止）が失敗するため、`crates/cli/embedded-examples/` へ
//! 正本のバイト単位同梱コピーを置いている（`crates/cli/templates/` が
//! `templates/` の同梱コピーであるのと同じ「正本 + 同梱コピー + ドリフト
//! 検知テスト」運用、イシュー #316/#378 の先行事例を踏襲）。
//!
//! 本テストは正本 `examples/{ssr-routing}/` と同梱コピー
//! `crates/cli/embedded-examples/{ssr-routing}/` を再帰走査し、両者のファイル
//! 集合とバイト内容が完全一致することを検証する。手動同期に頼らない
//! （`.claude/rules/ci.md` の cargo-deny pin ドリフト検知と同じ運用方針）。
//! `crates/cli/embedded-examples/README.md`（本コピーの出自を説明する
//! 追加ファイル、正本 `examples/` には存在しない）は比較対象から除外する。
//!
//! 同梱コピー配下の `Cargo.toml` は `Cargo.toml.embed` にリネームしている
//! （`cargo package` がネストした `Cargo.toml` を検出すると tarball の
//! ファイル列挙から機械的に除外する挙動を回避するため、
//! `crates/cli/src/new_template.rs` のドキュメントコメント参照）。本テストは
//! ファイル名の末尾 `.embed` を除去した論理パスで正本側と比較し、内容は
//! バイト単位一致を要求する。
//!
//! `template_publish_copy_drift.rs` をモデルにしており、`EXAMPLES` の全件を
//! パラメタ化して検証する（サンプル追加時に本ファイルの拡張だけで済むよう
//! `EXAMPLE_NAMES` を単一の情報源とする）。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// [`crate::new_template::EXAMPLES`] と同期させるサンプル名の一覧
/// （本テストクレートは `fandhe-frontend-cli` の内部モジュールへアクセス
/// できない統合テストのため、独立した固定リストとして維持する。新規
/// サンプル追加時は `new_template.rs::EXAMPLES` とあわせて更新すること）。
const EXAMPLE_NAMES: &[&str] = &["ssr-routing", "ssg-blog", "dist-server-docker"];

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

/// `examples/<name>/` 配下でビルド成果物として生成されうるディレクトリ名
/// （`.gitignore` の `/examples/*/target`・`/examples/*/dist/` に対応、
/// イシュー #501 レビュー指摘）。README の「動かし方」が案内する
/// `cargo run`（`dist/` 生成）・`cargo test`（`target/` 生成）を実行した
/// ワークツリーで本テストを走らせても偽陽性ドリフト検知を起こさないよう、
/// 走査対象から除外する。同梱コピー `crates/cli/embedded-examples/` 側は
/// これらのディレクトリを含まないため非対称除外で問題ない。
const IGNORED_BUILD_ARTIFACT_DIRS: &[&str] = &["target", "dist"];

/// `root` 配下の全ファイルの相対パス集合を再帰的に収集する
/// （ディレクトリ自体は含めない。`IGNORED_BUILD_ARTIFACT_DIRS` に該当する
/// ディレクトリは丸ごとスキップする。fail-closed: 読み取りエラーはテスト
/// 失敗として顕在化させる）。
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
            let is_ignored_build_artifact = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| IGNORED_BUILD_ARTIFACT_DIRS.contains(&name));
            if is_ignored_build_artifact {
                continue;
            }
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

/// 正本 `examples/<name>/` と同梱コピー `crates/cli/embedded-examples/<name>/`
/// のファイル集合・バイト内容が完全一致することを検証する。
fn assert_copy_matches_source(example_name: &str) {
    let root = workspace_root();
    let source_dir = root.join("examples").join(example_name);
    let copy_dir = root.join("crates/cli/embedded-examples").join(example_name);

    let source_files = collect_relative_files(&source_dir);
    let copy_files_raw = collect_relative_files(&copy_dir);
    let copy_files_logical: BTreeSet<PathBuf> = copy_files_raw
        .iter()
        .map(|p| to_logical_source_path(p))
        .collect();

    assert_eq!(
        source_files, copy_files_logical,
        "examples/{example_name}/ と crates/cli/embedded-examples/{example_name}/ の \
         ファイル集合が一致しない（正本の変更を同梱コピーへ反映するか、\
         crates/cli/embedded-examples/README.md の手順に従って再同期すること）"
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
            "crates/cli/embedded-examples/{example_name}/{} has drifted from \
             examples/{example_name}/{} (正本の変更を \
             crates/cli/embedded-examples/{example_name}/{} へ手動同期すること)",
            copy_rel.display(),
            source_rel.display(),
            copy_rel.display()
        );
    }
}

#[test]
fn all_registered_examples_publish_copy_matches_source() {
    for name in EXAMPLE_NAMES {
        assert_copy_matches_source(name);
    }
}

/// `crates/cli/embedded-examples/` 直下に正本には存在しない補助ファイル
/// （`README.md`）・登録済みサンプル以外の想定外エントリが紛れ込んでいない
/// ことを検証する（fail-closed: 同梱コピーの範囲が静かに広がるのを防ぐ）。
#[test]
fn embedded_examples_root_has_no_unexpected_top_level_entries() {
    let root = workspace_root();
    let copy_root = root.join("crates/cli/embedded-examples");
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

    let mut expected: Vec<String> = EXAMPLE_NAMES.iter().map(|n| n.to_string()).collect();
    expected.push("README.md".to_string());
    expected.sort();

    assert_eq!(
        names, expected,
        "crates/cli/embedded-examples/ 直下に想定外のエントリがある: {names:?}"
    );
}

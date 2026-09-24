//! `crate::blocks` のカテゴリ別ディレクトリ分割（イシュー #2734）の構造的
//! 整合性を固定するガードテスト。
//!
//! `blocks_nav.rs`（nav.toml ⇔ レジストリ ⇔ 原稿の三方突合）・
//! `blocks_code_drift.rs`（`rust_source` ⇔ Markdown フェンスの一致）とは
//! 別の観点として、以下 2 点を検証する。
//!
//! 1. 各 [`Block`] の `rust_source` が、その `category` から機械導出される
//!    `crates/docs-site/src/blocks/<section>/<category>/` 配下を指している
//!    こと（カテゴリの付け間違い・配置間違いを検知する）。
//! 2. [`BlockCategory::ALL`] の全 66 件について、対応する
//!    `crates/docs-site/src/blocks/<section>/<category>.rs`（空雛形）または
//!    `.../<category>/mod.rs`（block を持つカテゴリ）のいずれかが実在する
//!    こと（Phase 0（本イシュー）で用意した 66 カテゴリのスキャフォールドが
//!    将来 削除・欠落しないことを保証する）。

use std::path::PathBuf;

use fandhe_frontend_docs_site::blocks::{self, BlockCategory, BlockSection};

#[path = "support/shared_site.rs"]
mod shared_site;

fn repo_root() -> PathBuf {
    shared_site::repo_root()
}

/// `BlockCategory::kebab()` の `-` を `_` へ置換した snake_case ディレクトリ
/// 名を返す。`CategoryListing`（`kebab() == "category"`）は
/// `crate::blocks::category`（型定義モジュール）と紛らわしいため
/// `category_listing` を使う唯一の例外（実装計画の命名規約）。
fn category_dir_name(category: BlockCategory) -> String {
    if category == BlockCategory::CategoryListing {
        return "category_listing".to_string();
    }
    category.kebab().replace('-', "_")
}

/// [`BlockSection`] のディレクトリ名（`blocks/mod.rs` の `mod` 宣言と一致）。
fn section_dir_name(section: BlockSection) -> &'static str {
    match section {
        BlockSection::Marketing => "marketing",
        BlockSection::Application => "application",
        BlockSection::Ecommerce => "ecommerce",
        BlockSection::Docs => "docs",
    }
}

#[test]
fn every_registered_block_rust_source_matches_its_category_directory() {
    for block in blocks::all_blocks() {
        let expected_prefix = format!(
            "crates/docs-site/src/blocks/{}/{}/",
            section_dir_name(block.category.section()),
            category_dir_name(block.category)
        );
        assert!(
            block.rust_source.starts_with(&expected_prefix),
            "block {} (category {}) の rust_source {:?} は {expected_prefix:?} 配下を指していない",
            block.path,
            block.category.label(),
            block.rust_source
        );
    }
}

#[test]
fn every_category_has_a_scaffold_file_or_directory() {
    let blocks_root = repo_root().join("crates/docs-site/src/blocks");

    for category in BlockCategory::ALL {
        let section_dir = section_dir_name(category.section());
        let dir_name = category_dir_name(*category);
        let flat_path = blocks_root.join(section_dir).join(format!("{dir_name}.rs"));
        let dir_mod_path = blocks_root.join(section_dir).join(&dir_name).join("mod.rs");

        assert!(
            flat_path.is_file() || dir_mod_path.is_file(),
            "category {} ({section_dir}/{dir_name}) は空雛形 {flat_path:?} も \
             ディレクトリ化済み {dir_mod_path:?} も実在しない",
            category.label()
        );
    }
}

/// 空雛形（`Vec::new()` を返すだけ）とディレクトリ化済み（block を 1 件
/// 以上持つ）のいずれか一方だけが存在し、両方が同時に存在しないことを
/// 固定する（「カテゴリの卒業」手順が中途半端な状態を残さないことの検証）。
#[test]
fn no_category_has_both_a_flat_scaffold_and_a_directory() {
    let blocks_root = repo_root().join("crates/docs-site/src/blocks");

    for category in BlockCategory::ALL {
        let section_dir = section_dir_name(category.section());
        let dir_name = category_dir_name(*category);
        let flat_path = blocks_root.join(section_dir).join(format!("{dir_name}.rs"));
        let dir_mod_path = blocks_root.join(section_dir).join(&dir_name).join("mod.rs");

        assert!(
            !(flat_path.is_file() && dir_mod_path.is_file()),
            "category {} が空雛形 {flat_path:?} とディレクトリ {dir_mod_path:?} を両方持っている",
            category.label()
        );
    }
}

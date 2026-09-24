//! Blocks 共通のデモ用ダミー素材ヘルパ（イシュー #2737）の契約テスト。
//!
//! `crate::blocks::dummy_assets::IMAGE_ASSETS` が実サイトビルドの
//! `out_dir` へ実際に 5 種の SVG を書き出すこと、その内容が安全
//! （`data:`/`<script`/イベントハンドラ属性を含まない）であることを
//! 実ビルド成果物に対して固定する（単体テスト `crates/docs-site/src/
//! blocks/dummy_assets.rs` の `#[cfg(test)]` は生成関数の戻り値を直接
//! 検証するのに対し、本ファイルは `build_site` を経由した書き出し経路
//! 自体を検証する）。

use std::path::{Path, PathBuf};

#[path = "support/shared_site.rs"]
mod shared_site;

fn repo_root() -> PathBuf {
    shared_site::repo_root()
}

fn build_real_site() -> &'static Path {
    shared_site::real_site().out_dir.as_path()
}

/// 生成される 5 種の basename（`RESERVED_ASSET_NAMES`・`IMAGE_ASSETS` と
/// 一致させる、`crates/docs-site/src/build.rs` 参照）。
const EXPECTED_ASSET_BASENAMES: &[&str] = &[
    "blocks-demo-product.svg",
    "blocks-demo-avatar.svg",
    "blocks-demo-logo.svg",
    "blocks-demo-screenshot.svg",
    "blocks-demo-background.svg",
];

#[test]
fn dummy_asset_svgs_are_written_when_blocks_pages_exist() {
    let _ = repo_root();
    let out = build_real_site();

    // 実サイトは Blocks ページを 1 件以上持つ（`login-01` 等）ため、
    // `has_blocks_page` が立ち 5 種すべてが書き出される契約。
    for basename in EXPECTED_ASSET_BASENAMES {
        let path = out.join("assets").join(basename);
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{path:?} should be generated: {e}"));
        assert!(!content.is_empty(), "{path:?} should not be empty");
        assert!(
            content.trim_start().starts_with("<svg") || content.contains("<svg"),
            "{path:?} should have an <svg> root element"
        );
        assert!(
            !content.contains("data:"),
            "{path:?} must not embed a data: URI (REQ-1, issue #1562 regression)"
        );
        assert!(
            !content.contains("<script"),
            "{path:?} must not contain <script"
        );
        for handler in ["onload=", "onclick=", "onerror=", "onmouseover="] {
            assert!(
                !content.contains(handler),
                "{path:?} must not contain the event handler attribute {handler}"
            );
        }
    }
}

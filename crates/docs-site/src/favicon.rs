//! docs サイトの SVG favicon（イシュー #3604）。
//!
//! # 役割・呼び出し文脈
//!
//! 生成 HTML に `<link rel="icon">` がないとブラウザは既定の
//! `/favicon.ico` をオリジン直下へ取りに行き、`base_path` の外となるため 404 になる。
//! 本モジュールは固定図形のモノグラム（角丸タイル + 白い小文字 "f"）を
//! [`fandhe_frontend_core`] のノード木 API（`el`/`render`）だけで組み立てる。
//!
//! - [`crate::build::build_site`] が [`svg`] の結果を [`REL_PATH`] へ書き出す
//! - [`crate::layout::docs_page_with_assets`] が `<head>` の link から [`REL_PATH`] を参照する
//! - リダイレクトページ（[`crate::redirect`]）はサイトクロームを持たないため参照しない
//! - Phase 2 のヘッダーのブランド表示は [`mark_node`] を再利用する
//!   （インライン利用時の `aria-hidden` 等は呼び出し側のラッパーで付ける）
//!
//! # セキュリティ（REQ-1）
//!
//! ユーザー入力を一切含まない。`format!` による文字列組み立て・`raw_html()`・
//! `<script>`/`<style>`/`<text>`/イベントハンドラ属性は使わない。中間明度の
//! 塗りタイルに白字を載せることで、light/dark どちらのタブバーでも輪郭が見える。

use fandhe_frontend_core::{el, render, Node};

/// 出力先の `assets/` 相対パス。
pub const REL_PATH: &str = "assets/favicon.svg";

/// モノグラムの SVG ノード木を返す。
#[must_use]
pub fn mark_node() -> Node {
    let glyph = |x: &'static str, y: &'static str, w: &'static str, h: &'static str| {
        el(
            "rect",
            vec![
                ("x", x),
                ("y", y),
                ("width", w),
                ("height", h),
                ("fill", "#ffffff"),
            ],
            vec![],
        )
    };
    el(
        "svg",
        vec![
            ("xmlns", "http://www.w3.org/2000/svg"),
            ("viewBox", "0 0 32 32"),
            ("role", "img"),
            ("aria-label", "fandhe-frontend"),
        ],
        vec![
            el(
                "rect",
                vec![
                    ("width", "32"),
                    ("height", "32"),
                    ("rx", "7"),
                    ("fill", "#3182ce"),
                ],
                vec![],
            ),
            // 小文字 "f": 縦棒・上のかぎ・横棒。
            glyph("13", "9", "4", "16"),
            glyph("13", "8", "9", "4"),
            glyph("9", "14", "12", "4"),
        ],
    )
}

/// 書き出し用の SVG 文字列を返す。
#[must_use]
pub fn svg() -> String {
    render(&mark_node())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_is_a_static_self_contained_document() {
        let s = svg();
        assert!(s.starts_with("<svg"));
        assert!(s.contains(r#"xmlns="http://www.w3.org/2000/svg""#));
        assert!(s.contains(r#"viewBox="0 0 32 32""#));
        for forbidden in ["<text", "<script", "<style", "foreignObject", "href", " on"] {
            assert!(!s.contains(forbidden), "{forbidden} must not appear: {s}");
        }
    }

    #[test]
    fn svg_is_deterministic_and_matches_node() {
        assert_eq!(svg(), svg());
        assert_eq!(render(&mark_node()), svg());
    }
}

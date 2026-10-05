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
//! 既定経路はユーザー入力を含まず `<text>` も使わない。指定時（#3722）は `parse_nav` が
//! 検証した `brand_mark`（英数字 1 文字）・`brand_color`（`#RRGGBB`）と、`aria-label` に入る
//! `brand`（属性エスケープ経由）だけが入る。`format!` による文字列組み立て・`raw_html()`・
//! `<script>`/`<style>`/`style` 属性/イベントハンドラ属性/外部参照は使わない。中間明度の
//! 塗りタイルに白字を載せることで、light/dark どちらのタブバーでも輪郭が見える。

use fandhe_frontend_core::{el, render, text, Node};

/// 出力先の `assets/` 相対パス。
pub const REL_PATH: &str = "assets/favicon.svg";

/// 未指定時のタイル塗り色（fandhe-frontend 自身の値）。
pub const DEFAULT_BRAND_COLOR: &str = "#3182ce";

/// `[site].brand_mark` / `[site].brand_color`（#3722）を既定値解決済みで束ねた値。
/// [`crate::nav::Site::brand_mark`] が作り、[`crate::layout::SiteChrome`] 経由で
/// ヘッダーのインライン SVG と `assets/favicon.svg` の双方へ同じ値を渡す。
/// `glyph` は ASCII 英数字 1 文字、`color` は `#RRGGBB`（いずれも `parse_nav` が検証済み）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrandMark<'a> {
    /// 指定時のみ `<text>` で描く文字。`None` は従来の rect 図案の "f"。
    pub glyph: Option<&'a str>,
    /// タイルの塗り色。
    pub color: &'a str,
}

impl Default for BrandMark<'_> {
    fn default() -> Self {
        Self {
            glyph: None,
            color: DEFAULT_BRAND_COLOR,
        }
    }
}

/// 既定（fandhe-frontend 自身）のモノグラムの SVG ノード木を返す。
#[must_use]
pub fn mark_node() -> Node {
    mark_node_for(crate::layout::DEFAULT_BRAND, &BrandMark::default())
}

/// `label`（`aria-label`）と [`BrandMark`] からモノグラムの SVG ノード木を返す。
///
/// 属性順・子ノード順は未指定時の出力を従来とバイト一致させるため固定。
/// 色は `fill` 属性、文字は汎用フォントの `<text>` の presentation attribute だけで
/// 指定し、`style` 属性・`<style>`・外部参照は持ち込まない（CSP 不変）。
#[must_use]
pub fn mark_node_for(label: &str, mark: &BrandMark<'_>) -> Node {
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
    let mut children = vec![el(
        "rect",
        vec![
            ("width", "32"),
            ("height", "32"),
            ("rx", "7"),
            ("fill", mark.color),
        ],
        vec![],
    )];
    match mark.glyph {
        // 小文字 "f": 縦棒・上のかぎ・横棒。
        None => children.extend([
            glyph("13", "9", "4", "16"),
            glyph("13", "8", "9", "4"),
            glyph("9", "14", "12", "4"),
        ]),
        Some(g) => children.push(el(
            "text",
            vec![
                ("x", "16"),
                ("y", "24"),
                ("text-anchor", "middle"),
                ("font-family", "sans-serif"),
                ("font-size", "22"),
                ("font-weight", "700"),
                ("fill", "#ffffff"),
            ],
            vec![text(g)],
        )),
    }
    el(
        "svg",
        vec![
            ("xmlns", "http://www.w3.org/2000/svg"),
            ("viewBox", "0 0 32 32"),
            ("role", "img"),
            ("aria-label", label),
        ],
        children,
    )
}

/// 既定の書き出し用 SVG 文字列を返す。
#[must_use]
pub fn svg() -> String {
    render(&mark_node())
}

/// 指定値で生成した書き出し用 SVG 文字列を返す（[`crate::build::build_site`] が使う）。
#[must_use]
pub fn svg_for(label: &str, mark: &BrandMark<'_>) -> String {
    render(&mark_node_for(label, mark))
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

    const LEGACY: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" role="img" aria-label="fandhe-frontend"><rect width="32" height="32" rx="7" fill="#3182ce"></rect><rect x="13" y="9" width="4" height="16" fill="#ffffff"></rect><rect x="13" y="8" width="9" height="4" fill="#ffffff"></rect><rect x="9" y="14" width="12" height="4" fill="#ffffff"></rect></svg>"##;

    #[test]
    fn default_svg_is_byte_identical_to_legacy_output() {
        assert_eq!(svg(), LEGACY);
        assert_eq!(
            svg_for(crate::layout::DEFAULT_BRAND, &BrandMark::default()),
            svg()
        );
    }

    #[test]
    fn color_only_changes_tile_fill() {
        let m = BrandMark {
            glyph: None,
            color: "#112233",
        };
        let s = svg_for("Acme", &m);
        assert!(s.contains(r##"rx="7" fill="#112233""##));
        assert!(!s.contains("<text"));
        assert_eq!(s.matches(r##"fill="#ffffff""##).count(), 3);
    }

    #[test]
    fn glyph_is_drawn_as_single_text_without_forbidden_parts() {
        let m = BrandMark {
            glyph: Some("A"),
            color: "#AB12cd",
        };
        let s = svg_for("Acme", &m);
        assert_eq!(s.matches("<text").count(), 1);
        assert!(s.contains(">A</text>"));
        assert!(s.contains(r#"aria-label="Acme""#));
        assert!(!s.contains("<rect x="));
        for forbidden in [
            "<script",
            "<style",
            "style=",
            "foreignObject",
            "href",
            "url(",
            " on",
        ] {
            assert!(!s.contains(forbidden), "{forbidden} must not appear: {s}");
        }
    }

    #[test]
    fn label_is_attribute_escaped() {
        let s = svg_for("A<\"b>&", &BrandMark::default());
        assert!(!s.contains("A<\"b>&"));
        assert!(s.contains("aria-label=\"A&lt;"));
    }
}

//! `grid-list-action-tiles` block（イシュー #2917。親トラッキング「Blocks
//! 目的別パーツ拡充ツリー」配下、対応表 ID R0977 の 1 件のみを主参照とする
//! 合成例。境界線を共有するアクションタイルのグリッド）。出典の固有名・
//! ファイル名は記載しない（`team_avatar_grid` 等と同じライセンス上の
//! 転記制限、対応表 ID のみを記す）。参照元はレイアウト構造のみを参照し、
//! 配色・文言・アイコンは独自に実装する。
//!
//! # 使用部品
//!
//! `item` / `icon` / `heading` / `text` / `link-overlay` の 5 部品のみを
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 境界共有の実装方式
//!
//! グリッドコンテナの `gap: 1px` + `background: var(--fandhe-color-border)`
//! でタイル間の境界線を表現する（各タイルが個別に `border` を持つと二重線
//! になるため、1px の隙間から背景色を覗かせる方式を採る）。`overflow:
//! hidden` は使わない。角丸なグリッド外枠の内側でフォーカスリング
//! （[`fandhe_frontend_pre_styled_ui::link_overlay`] の
//! `:focus-visible` + `border-radius: inherit`）が `overflow: hidden` に
//! よって切り取られるのを避けるためである。
//!
//! # 角丸の分岐（先頭・末尾タイルの外側の角のみ）
//!
//! 1 列時は `:first-child`（上 2 角）・`:last-child`（下 2 角）のみへ
//! 角丸を付ける。`@media (min-width: 40rem)`（2 列切替）では
//! `:first-child`（左上）・`:nth-child(2)`（右上）・
//! `:nth-last-child(2)`（左下）・`:last-child`（右下）の 4 隅のみへ
//! 個別に角丸を再宣言する（1 列時に設定した角を明示的に `0` へ戻す。
//! 4 隅すべてを毎回明示することで、同一詳細度のメディアクエリ内規則が
//! ソース順で後勝ちする前提に依存する）。テーマの breakpoint トークンは
//! `@media` 条件式の中では解決できないため、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Sm`（640px = 40rem）
//! と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする
//! （`team_avatar_grid`/`error_page_popular_links` と同じ判断）。
//!
//! # タイル全体のリンク化と矢印の配置
//!
//! 各タイルは [`link_overlay::root`] で `item::root` を包み、
//! [`overlay`] を最後の子として追加することで全面クリック可能にする
//! （`cta_feature_links::feature_item` と同型のパターン）。矢印
//! （`item::actions`）は `position: absolute` で `link_overlay::root`
//! （`position: relative` を持つ）を基準に右上へ配置する。`item::root`
//! 自身は `position` を持たないため、絶対配置の基準は自動的に
//! `link_overlay::root` まで遡る。
//!
//! # href の方針（`href="#"` を使わない）
//!
//! [`Block::demo`] は `fn() -> Node` のため `base_path` を受け取れず、
//! 6 タイルはいずれもサイト内に実在する索引ページへの相対パス
//! （[`error_page_popular_links`](super::super::marketing::error_page::error_page_popular_links)
//! と同じ先例）を指す。実利用時は自分のページ URL へ差し替えることを
//! `site/blocks/grid-list-action-tiles.md` の導入文で明記する。
//!
//! # アクセシブル名
//!
//! [`overlay`] へタイル見出しと同じ文言を `aria-label` として付与する。
//! `id`/`aria-labelledby` は出力しない（`demo_output_has_no_dangling_aria_
//! references_or_duplicate_ids` 契約、`crate::blocks` モジュール doc
//! 参照）。アイコン・矢印はいずれも装飾用途のため `IconProps::label` を
//! 付けない。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、タイル見出しは
//! `HeadingLevel::H3` にする（セクション見出しは持たない、Issue の仕様
//! どおり）。
//!
//! # アイコンは自作の抽象幾何図形
//!
//! `cta_feature_links::geo_icon` と同型のパターンで、実在ブランドの
//! ロゴ・商標を模さない自作の線画アイコンのみを使う。
//!
//! # media（アイコン枠）の寸法上書きの詳細度
//!
//! `item::media(ItemMediaVariant::Icon, ..)` は `pre-styled-ui.css` 側で
//! `[data-scope="item"][data-part="media"][data-variant="icon"]`（3 属性
//! セレクタ、詳細度 `(0,3,0)`）により `width`/`height`/`border-radius` を
//! 既に宣言している。block 固有フックのみの単一属性セレクタ
//! （`[data-blocks-grid-list-action-tiles-media]`、詳細度 `(0,1,0)`）では
//! 詳細度で負けて上書きできないため、[`LAYOUT_CSS`] 側も
//! `[data-scope="item"][data-part="media"]` を前置した同格の 3 属性
//! セレクタにして詳細度を揃える（`!important` は使わない）。詳細度が
//! 同格であれば、`blocks.css` は `pre-styled-ui.css` より後に `<link>`
//! される（`crates/docs-site/src/build.rs` の `extra_stylesheets` 配線
//! 順）ためソース順で本 block 側が勝つ。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。文言はすべて独自に書いた架空の
//! ものであり、実企業名・実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::item::{self, ItemMediaVariant, ItemRootProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

const TILE_ATTR: &str = "data-blocks-grid-list-action-tiles-tile";
const ITEM_ATTR: &str = "data-blocks-grid-list-action-tiles-item";
const MEDIA_ATTR: &str = "data-blocks-grid-list-action-tiles-media";
const CONTENT_ATTR: &str = "data-blocks-grid-list-action-tiles-content";
const ARROW_ATTR: &str = "data-blocks-grid-list-action-tiles-arrow";

/// タイル 1 件分のデータ（モジュール doc「href の方針」節参照）。
struct Tile {
    href: &'static str,
    title: &'static str,
    description: &'static str,
    icon_path_d: &'static str,
}

/// タイル 6 件（サイト内に実在する索引ページのみを指す）。
const TILES: [Tile; 6] = [
    Tile {
        href: "../../guides/",
        title: "ガイドを読む",
        description: "導入から実践までの手順を順番に確認できます。",
        icon_path_d: "M4 4h12v16H4zM8 8h4M8 12h4",
    },
    Tile {
        href: "../../examples/",
        title: "サンプルを試す",
        description: "実際に動くサンプルプロジェクトを確認できます。",
        icon_path_d: "M12 3l2.5 5.5L20 9l-4 4 1 6-5-3-5 3 1-6-4-4 5.5-.5z",
    },
    Tile {
        href: "../../primitives/",
        title: "Primitives を見る",
        description: "構造とアクセシビリティを担う headless 部品一覧です。",
        icon_path_d: "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
    },
    Tile {
        href: "../../themes/",
        title: "Themes を見る",
        description: "そのまま使える意匠付きの部品一覧です。",
        icon_path_d: "M4 6h16M4 12h16M4 18h10",
    },
    Tile {
        href: "../",
        title: "Blocks 一覧に戻る",
        description: "既存部品を組み合わせた合成例の一覧です。",
        icon_path_d: "M20 12H4M10 6l-6 6 6 6",
    },
    Tile {
        href: "../../api/",
        title: "API Reference を読む",
        description: "公開 API の詳細なリファレンスを確認できます。",
        icon_path_d: "m9 6 6 6-6 6",
    },
];

/// 自作の線画（stroke）アイコンを組み立てる（
/// [`super::super::marketing::cta::cta_feature_links::geo_icon`] と同型の
/// パターン。装飾用途のため `IconProps::label` は付けない）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// タイル 1 件分を組み立てる（モジュール doc「タイル全体のリンク化と矢印
/// の配置」節）。
fn action_tile(tile: &Tile) -> Node {
    link_overlay::root(
        vec![(TILE_ATTR, "")],
        vec![
            item::root(
                ItemRootProps::default(),
                vec![(ITEM_ATTR, "")],
                vec![
                    item::media(
                        ItemMediaVariant::Icon,
                        vec![(MEDIA_ATTR, "")],
                        vec![geo_icon(tile.icon_path_d)],
                    ),
                    item::content(
                        vec![(CONTENT_ATTR, "")],
                        vec![
                            heading(
                                HeadingLevel::H3,
                                &HeadingProps::default(),
                                vec![],
                                vec![text(tile.title)],
                            ),
                            styled_text::text(
                                &TextProps {
                                    variant: TextVariant::Muted,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(tile.description)],
                            ),
                        ],
                    ),
                    item::actions(vec![(ARROW_ATTR, "")], vec![geo_icon("m9 18 6-6-6-6")]),
                ],
            ),
            overlay(tile.href, vec![("aria-label", tile.title)], vec![]),
        ],
    )
}

/// `grid-list-action-tiles` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-grid-list-action-tiles-grid")],
        TILES.iter().map(action_tile).collect(),
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/grid-list-action-tiles/",
    title: "grid-list-action-tiles",
    category: BlockCategory::GridList,
    rust_source: "crates/docs-site/src/blocks/application/grid_list/grid_list_action_tiles.rs",
    demo_class: "blocks-grid-list-action-tiles",
    parts: &[
        Part {
            label: "Item",
            path: "/themes/item/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `grid_list_action_tiles` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型で private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-grid-list-action-tiles-*` と
/// `[data-blocks-grid-list-action-tiles-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`team_avatar_grid` と同じ名前空間分離）。
/// 色リテラル（`#fff`/`white` 等）は使わず、可読性の確保はすべて
/// `--fandhe-*` トークンで行う。
const LAYOUT_CSS: &str = "\
.blocks-grid-list-action-tiles-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: 1px;\n  background: var(--fandhe-color-border);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
[data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-grid-list-action-tiles-tile] {\n  background: var(--fandhe-color-bg);\n  padding: var(--fandhe-space-6);\n  border-radius: 0;\n}\n\
[data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-grid-list-action-tiles-tile]:first-child {\n  border-top-left-radius: var(--fandhe-radius-lg);\n  border-top-right-radius: var(--fandhe-radius-lg);\n}\n\
[data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-grid-list-action-tiles-tile]:last-child {\n  border-bottom-left-radius: var(--fandhe-radius-lg);\n  border-bottom-right-radius: var(--fandhe-radius-lg);\n}\n\
[data-scope=\"item\"][data-part=\"root\"][data-blocks-grid-list-action-tiles-item] {\n  flex-direction: column;\n  align-items: flex-start;\n  padding: 0;\n  border: 0;\n  border-radius: 0;\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"item\"][data-part=\"media\"][data-blocks-grid-list-action-tiles-media] {\n  width: 2.5rem;\n  height: 2.5rem;\n  border-radius: var(--fandhe-radius-md);\n}\n\
[data-scope=\"item\"][data-part=\"content\"][data-blocks-grid-list-action-tiles-content] {\n  padding-right: var(--fandhe-space-8);\n}\n\
[data-blocks-grid-list-action-tiles-arrow] {\n  position: absolute;\n  top: var(--fandhe-space-6);\n  right: var(--fandhe-space-6);\n  color: var(--fandhe-color-fg-muted);\n}\n\
@media (min-width: 40rem) {\n  .blocks-grid-list-action-tiles-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
  [data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-grid-list-action-tiles-tile]:first-child {\n    border-top-left-radius: var(--fandhe-radius-lg);\n    border-top-right-radius: 0;\n    border-bottom-left-radius: 0;\n    border-bottom-right-radius: 0;\n  }\n\
  [data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-grid-list-action-tiles-tile]:nth-child(2) {\n    border-top-left-radius: 0;\n    border-top-right-radius: var(--fandhe-radius-lg);\n    border-bottom-left-radius: 0;\n    border-bottom-right-radius: 0;\n  }\n\
  [data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-grid-list-action-tiles-tile]:nth-last-child(2) {\n    border-top-left-radius: 0;\n    border-top-right-radius: 0;\n    border-bottom-left-radius: var(--fandhe-radius-lg);\n    border-bottom-right-radius: 0;\n  }\n\
  [data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-grid-list-action-tiles-tile]:last-child {\n    border-top-left-radius: 0;\n    border-top-right-radius: 0;\n    border-bottom-left-radius: 0;\n    border-bottom-right-radius: var(--fandhe-radius-lg);\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（item/icon/heading/text/link-overlay）の anatomy を
    /// すべて実際に出力していることを固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"item\"",
            "data-scope=\"icon\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"link-overlay\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// overlay の件数（タイル 6 件分）とそれぞれの `aria-label` を固定する。
    #[test]
    fn demo_renders_six_overlays_with_aria_label() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"data-part="overlay""#).count(), 6);
        assert_eq!(html.matches("aria-label=").count(), 6);
    }

    /// 非対話・XSS の不変条件（`crate::blocks` モジュール doc）を固定する。
    #[test]
    fn demo_has_no_form_dead_links_or_unsafe_output() {
        let html = render(&demo());
        for absent in ["<form", "href=\"#\"", "src=\"data:", " id=\"", "<script"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
        for href in [
            "../../guides/",
            "../../examples/",
            "../../primitives/",
            "../../themes/",
            "../../api/",
        ] {
            assert!(
                html.contains(&format!(r#"href="{href}""#)),
                "demo should link to {href}"
            );
        }
        assert!(
            html.contains(r#"href="../""#),
            "demo should link back to blocks index"
        );
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント・境界共有・角丸分岐の
    /// 各セレクタを持ち、色リテラルを含まないことを固定する。
    #[test]
    fn layout_css_declares_breakpoint_and_corner_selectors() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("gap: 1px"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-border)"));
        assert!(LAYOUT_CSS.contains(":first-child"));
        assert!(LAYOUT_CSS.contains(":last-child"));
        assert!(LAYOUT_CSS.contains(":nth-child(2)"));
        assert!(LAYOUT_CSS.contains(":nth-last-child(2)"));
    }

    /// ルート class が `demo()` の出力へ実際に現れること。
    #[test]
    fn layout_root_class_is_present() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-grid-list-action-tiles-grid\""));
    }

    /// media（アイコン枠）の寸法上書きセレクタが `pre-styled-ui.css` の
    /// `[data-scope="item"][data-part="media"][data-variant="icon"]`
    /// （詳細度 `(0,3,0)`）と同格の 3 属性セレクタであることを固定する
    /// （モジュール doc「media（アイコン枠）の寸法上書きの詳細度」節。
    /// 単一属性セレクタへ後退すると詳細度で負けて上書きが効かなくなる
    /// 回帰を防ぐ）。
    #[test]
    fn layout_css_media_size_selector_matches_icon_variant_specificity() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"item\"][data-part=\"media\"][data-blocks-grid-list-action-tiles-media] {"
        ));
    }
}

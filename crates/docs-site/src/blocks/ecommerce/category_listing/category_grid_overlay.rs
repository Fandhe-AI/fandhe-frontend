//! `category-grid-overlay` block（イシュー #3038。親トラッキング #3024
//! 「Blocks EC」配下、対応表 ID R0036（8 枚の正方形タイル・タイル全体が
//! リンク）を主参照とし、R0035（4 枚均等・リンクなし版）・R0038（3 枚・
//! 説明と買い物リンクを重ねる版）・R0610（見出し行 + 一覧ボタン、縦長
//! 4 枚）の 3 件を集約する合成例。カテゴリ画像タイルを均等グリッドで並べ、
//! タイル下部へ名称・説明を画像へ重ねて表示する）。取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `link` / `link-overlay` / `image` / `text` の 5 部品を合成
//! する（[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # レイアウトと `@container` の理由
//!
//! 上部の見出し行（左に見出し、右に「全カテゴリへのリンク」）の下へ、
//! 同じ大きさのカテゴリ画像タイルを均等グリッドで並べる。Demo 枠の幅は
//! ビューポート幅と一致しないため、[`crate::blocks::ecommerce::store_nav::
//! store_nav_mega_menu`] と同じ判断で `@container`（コンテナクエリ）を
//! 使い、Demo 枠自身の実測幅を基準に列数を 1 → 2 → 3 → 4 と増やす
//! （`grid_list_contact_cards`/`profile_detail_datalist` と同型）。
//!
//! # `demo_class` とグリッド class を分ける理由
//!
//! [`crate::blocks::render_page`] は Demo ラッパー（子要素 1 個）へも
//! [`Block::demo_class`] を付与するため、`.blocks-category-grid-overlay`
//! 自体をグリッド化すると、ラッパー自身がグリッドの 1 トラックへ押し
//! 込まれる（`blog_overlay_cards` モジュール doc と同じ再発防止）。
//! グリッドは内側の `.blocks-category-grid-overlay-grid` へ適用する。
//!
//! # 重なり順（背景画像 → スクリム → 本文 → overlay）
//!
//! 各タイルは `link_overlay::root` の内側に (1) 背景画像（`image::image`、
//! `position: relative` の通常フロー、`link_overlay::root` の高さを確立
//! する唯一の子）、(2) 下から上へのグラデーション（`div.blocks-category-
//! grid-overlay-scrim`、`position: absolute; inset: 0`、`aria-hidden`）、
//! (3) 名称 + 説明（`div.blocks-category-grid-overlay-content`、
//! `position: absolute; inset-inline: 0; bottom: 0`）、(4) `link_overlay::
//! overlay`（`position: absolute; inset: 0`）の順で重ねる（`blog_overlay_
//! cards` と同じ構成）。
//!
//! # 背景画像を `<img>` にする理由（CSS `url()` を使わない）
//!
//! `dummy_assets::PRODUCT_SRC` はページ基準の相対パスであり、
//! `crate::blocks::stylesheet()` の CSS 内で `url()` に書くとファイルの
//! 配置階層がずれて解決先が変わる。`<img>` はページ HTML を起点に解決
//! されるため相対パスがそのまま使える（`blog_overlay_cards` と同じ理由）。
//!
//! # 配色を fg/bg の反転ペアにする理由
//!
//! スクリム + 本文の配色は `--fandhe-color-fg`（スクリムの下地）/
//! `--fandhe-color-bg`（本文の文字色）の反転ペアを使う。light/dark
//! いずれのテーマでも、明るい背景画像に対して常に高コントラストになる
//! （`blog_overlay_cards` と同じ判断）。
//!
//! # `drop_class_attr` とフックの選び方
//!
//! `link_overlay::root` / `image::image` / `heading::heading` /
//! `styled_text::text` / `link::root` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去してから合成する契約を持つ
//! ため、Demo 固有のスタイルフックは `data-blocks-category-grid-overlay-*`
//! 属性で渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の
//! `div`/`span` には `class` がそのまま効くため `.blocks-category-grid-
//! overlay-*` クラスセレクタを使う。
//!
//! # 詳細度を (0,3,0) へ引き上げる理由
//!
//! `image::image`（`[data-scope="image"][data-part="root"]`、(0,2,0)）の
//! base 宣言（`max-width: 100%; height: auto`）を上書きする必要があるため、
//! Demo 固有セレクタは `[data-scope="…"][data-part="root"][data-blocks-
//! category-grid-overlay-*]` の 3 属性セレクタ（(0,3,0)）として書く
//! （`blog_overlay_cards` と同じ判断）。
//!
//! # リンク先の方針
//!
//! `Block::demo` は `fn() -> Node` で `base_path` を受け取れないため、
//! 他 block と同じく外部の絶対 URL `https://github.com/Fandhe-AI/
//! fandhe-frontend` をリンク先として使う。`href="#"` の死リンクは使わない。
//!
//! # 見出しレベル
//!
//! Demo 内に `<h1>`/`<h2>` を置くとページ側（Markdown 原稿の `# ` および
//! `## Demo`）と重複するため、見出し行の見出しは `<h3>` にする
//! （`blog_overlay_cards` と同じ判断）。タイル自体の名称は見出しにしない
//! （見出しなし variant で h2 から h4 へレベルが飛ぶのを避けるため）。
//! アクセシブルな名前は `overlay` の `aria-label` で与える。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # フォーカスリングを内側へ出す理由
//!
//! `link_overlay::overlay` は既定 CSS として `FocusRingOffset::Outside`
//! （`outline-offset` が要素の外側へ出る）を登録しているが、本 block は
//! タイルの角丸クリップのために `[data-blocks-category-grid-overlay-tile]`
//! へ `overflow: hidden` を与えており、この前提が崩れて `:focus-visible`
//! のリングがタイル境界でクリップされる（`blog_overlay_cards` と同じ
//! 回帰パターンの事前対処）。`overlay` へ `data-blocks-category-grid-
//! overlay-overlay` を付与し、[`LAYOUT_CSS`] 側で `outline-offset` を
//! 内側へ上書きする。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// カテゴリ 1 件分のダミーデータ（架空、実在の人物・企業とは無関係）。
struct Category {
    name: &'static str,
    description: &'static str,
}

/// 正方形タイル 8 件（Variant A、R0036 主参照）。
const CATEGORIES_SQUARE: [Category; 8] = [
    Category {
        name: "キッチン用品",
        description: "毎日の調理を支える定番アイテム",
    },
    Category {
        name: "文具",
        description: "書く・まとめる・持ち運ぶ道具一式",
    },
    Category {
        name: "照明",
        description: "空間の雰囲気を整える光源",
    },
    Category {
        name: "収納",
        description: "暮らしを整える棚・箱・ラック",
    },
    Category {
        name: "テキスタイル",
        description: "肌触りにこだわった布製品",
    },
    Category {
        name: "アウトドア",
        description: "屋外での時間を快適にする道具",
    },
    Category {
        name: "植物",
        description: "部屋に緑を添える鉢植え各種",
    },
    Category {
        name: "ギフト",
        description: "贈る相手を選ばない定番の贈り物",
    },
];

/// 縦長タイル 4 件（Variant B、R0035 の枚数 + R0610 の比率）。
const CATEGORIES_PORTRAIT: [Category; 4] = [
    Category {
        name: "キッチン用品",
        description: "毎日の調理を支える定番アイテム",
    },
    Category {
        name: "文具",
        description: "書く・まとめる・持ち運ぶ道具一式",
    },
    Category {
        name: "照明",
        description: "空間の雰囲気を整える光源",
    },
    Category {
        name: "収納",
        description: "暮らしを整える棚・箱・ラック",
    },
];

/// カテゴリタイル 1 件を組み立てる（モジュール doc「重なり順」節参照）。
fn tile(cat: &Category, ratio: AspectRatio) -> Node {
    link_overlay::root(
        vec![("data-blocks-category-grid-overlay-tile", "")],
        vec![
            image::image(
                &ImageProps {
                    aspect_ratio: ratio,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-category-grid-overlay-image", "")],
            ),
            div(
                vec![
                    ("class", "blocks-category-grid-overlay-scrim"),
                    ("aria-hidden", "true"),
                ],
                vec![],
            ),
            div(
                vec![("class", "blocks-category-grid-overlay-content")],
                vec![
                    styled_text::text(
                        &TextProps::default(),
                        vec![("data-blocks-category-grid-overlay-name", "")],
                        vec![text(cat.name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(cat.description)],
                    ),
                ],
            ),
            overlay(
                REPO,
                vec![
                    ("aria-label", cat.name),
                    ("data-blocks-category-grid-overlay-overlay", ""),
                ],
                vec![],
            ),
        ],
    )
}

/// Variant A: 見出し行 + 正方形タイル 8 枚（R0036 主参照 + R0610 の見出し
/// 行）。
fn variant_square() -> Node {
    let tiles: Vec<Node> = CATEGORIES_SQUARE
        .iter()
        .map(|cat| tile(cat, AspectRatio::Square))
        .collect();
    div(
        vec![("class", "blocks-category-grid-overlay-variant")],
        vec![
            div(
                vec![("class", "blocks-category-grid-overlay-header")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("カテゴリから探す")],
                    ),
                    link::root(
                        REPO,
                        &LinkProps {
                            variant: LinkVariant::Underline,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![
                            text("すべてのカテゴリ"),
                            span(vec![("aria-hidden", "true")], vec![text(" \u{2192}")]),
                        ],
                    ),
                ],
            ),
            div(vec![("class", "blocks-category-grid-overlay-grid")], tiles),
        ],
    )
}

/// Variant B: 見出しなし + 縦長タイル 4 枚（R0035 の枚数 + R0610 の比率）。
fn variant_portrait() -> Node {
    let tiles: Vec<Node> = CATEGORIES_PORTRAIT
        .iter()
        .map(|cat| tile(cat, AspectRatio::Portrait))
        .collect();
    div(
        vec![("class", "blocks-category-grid-overlay-variant")],
        vec![div(
            vec![("class", "blocks-category-grid-overlay-grid")],
            tiles,
        )],
    )
}

/// `category-grid-overlay` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「レイアウトと `@container` の理由」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-category-grid-overlay")],
        vec![variant_square(), variant_portrait()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/category-grid-overlay/",
    title: "category-grid-overlay",
    category: BlockCategory::CategoryListing,
    rust_source: "crates/docs-site/src/blocks/ecommerce/category_listing/category_grid_overlay.rs",
    demo_class: "blocks-category-grid-overlay",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `category_grid_overlay` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。[`BLOCK`] の `layout_css`
/// （[`LayoutCss::Static`]）として自己申告し、[`crate::blocks::stylesheet`]
/// が [`crate::blocks::all_blocks`] を走査して連結する）。
///
/// セレクタは `.blocks-category-grid-overlay*` と
/// `[data-blocks-category-grid-overlay-*]`（`[data-scope=…]` と組み合わせる
/// 3 属性セレクタを含む）のみを用い、他 block や部品の素のセレクタへ
/// 影響させない（`blog_overlay_cards` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-category-grid-overlay {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-category-grid-overlay-header {\n  display: flex;\n  justify-content: space-between;\n  align-items: baseline;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-category-grid-overlay-variant {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-category-grid-overlay;\n}\n\
.blocks-category-grid-overlay-grid {\n  display: grid;\n  grid-template-columns: repeat(1, minmax(0, 1fr));\n  gap: var(--fandhe-space-4);\n}\n\
@container blocks-category-grid-overlay (min-width: 20rem) {\n  .blocks-category-grid-overlay-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@container blocks-category-grid-overlay (min-width: 30rem) {\n  .blocks-category-grid-overlay-grid {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n\
@container blocks-category-grid-overlay (min-width: 38rem) {\n  .blocks-category-grid-overlay-grid {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n}\n\
[data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-category-grid-overlay-tile] {\n  position: relative;\n  overflow: hidden;\n  border-radius: 0.5rem;\n  background: var(--fandhe-color-fg);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-category-grid-overlay-image] {\n  display: block;\n  width: 100%;\n  max-width: none;\n  object-fit: cover;\n}\n\
.blocks-category-grid-overlay-scrim {\n  position: absolute;\n  inset: 0;\n  background: linear-gradient(to top, var(--fandhe-color-fg) 0%, var(--fandhe-color-fg) 15%, transparent 70%);\n  opacity: 0.9;\n}\n\
.blocks-category-grid-overlay-content {\n  position: absolute;\n  inset-inline: 0;\n  bottom: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  padding: var(--fandhe-space-4);\n  color: var(--fandhe-color-bg);\n}\n\
[data-blocks-category-grid-overlay-name] {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
[data-scope=\"link-overlay\"][data-part=\"overlay\"][data-blocks-category-grid-overlay-overlay]:focus-visible {\n  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_dead_links() {
        let html = render(&demo());
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert!(html.contains(r#"data-scope="image""#));
        assert!(html.contains(r#"data-scope="heading""#));
        assert!(html.contains(r#"data-scope="text""#));
        assert!(html.contains(r#"data-scope="link""#));
        assert_eq!(
            html.matches("data-blocks-category-grid-overlay-tile=\"\"")
                .count(),
            12,
            "should render exactly 12 tiles (8 square + 4 portrait)"
        );
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
    }

    /// 両 variant のアスペクト比が実際に切り替わっていることの固定。
    #[test]
    fn both_aspect_ratio_variants_are_rendered() {
        let html = render(&demo());
        assert!(html.contains("fd-image--aspect-ratio-square"));
        assert!(html.contains("fd-image--aspect-ratio-portrait"));
    }

    /// [`LAYOUT_CSS`] が想定するコンテナクエリ・フォーカスリング是正を
    /// 持つこと。
    #[test]
    fn layout_css_declares_container_queries_and_inset_focus_ring() {
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert_eq!(
            LAYOUT_CSS
                .matches("@container blocks-category-grid-overlay (min-width:")
                .count(),
            3,
            "should declare exactly 3 container-query breakpoints"
        );
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"link-overlay\"][data-part=\"overlay\"][data-blocks-category-grid-overlay-overlay]:focus-visible"
        ));
        assert!(
            LAYOUT_CSS.contains("outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));")
        );
    }
}

//! `category-split-panels` block（イシュー #3040。親トラッキング #3024
//! 「Blocks EC」配下、`crate::blocks::ecommerce::category_listing`
//! カテゴリ）。
//!
//! 表示幅を左右 2 等分したパネルを 2 枚並べ、各パネルは背景画像の上に
//! 見出し・説明・買い物導線のリンクを重ねる構成を、主参照 R0826 の
//! 1 形として合成する（取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない、`category_featured_banner` と同じ転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `image` / `link` の 4 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。
//!
//! # 重ね合わせは `grid-area` の共有で行う
//!
//! [`category_featured_banner`]（[`super::category_featured_banner`]）と
//! 同じ判断で `position: absolute` は使わず、各パネル内で画像と
//! 淡い面（surface）の両方へ同じ `grid-area: 1 / 1` を与えて同一セルへ
//! 重ねる。
//!
//! # 淡い面（非反転）の配色
//!
//! 参照元 R0826 は暗い画像の上に明るい面を重ねる構成であり、
//! `category_featured_banner` の形 A（暗い半透明パネル + 明るい文字の
//! 反転ペア）とは逆に、淡い半透明面の上へ既定の前景色をそのまま置く
//! 非反転ペアにする。`--fandhe-color-bg` ベースの半透明面
//! （`color-mix(in srgb, var(--fandhe-color-bg) 88%, transparent)`）+
//! 既定の `--fandhe-color-fg` を使い、`link::root`/`styled_text::text`
//! の既定配色をどちらも上書きしない（`TextVariant::Muted` は使わない。
//! 淡い面の上ではコントラストが下がるため）。
//!
//! # ブレークポイント・`@container` を使う理由
//!
//! [`category_featured_banner`]/[`super::category_grid_overlay`]/
//! `store_nav_mega_menu` と同じ判断で、`@media`（ビューポート幅判定）
//! ではなく `@container`（コンテナクエリ）を使う。Demo ルートへ
//! `container-type: inline-size` を宣言し、表示領域自身の実測幅を基準に
//! 広い幅（`40rem` 以上、docs サイト上の Demo 枠実測上限 `43rem` でも
//! 到達する値）で 2 列、それ未満では縦積みへ切り替える。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `heading::heading`/`text::text`/`image::image`/`link::root` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-category-split-panels-*` 属性で渡す（`image` のみ、
//! 既定宣言〔`max-width: 100%; height: auto`〕を上書きするため
//! `[data-scope="image"][data-part="root"][data-blocks-category-split-
//! panels-image]` の詳細度 (0,3,0) セレクタにする）。素の `div` には
//! `class` がそのまま効くため、レイアウト用の入れ子は
//! `.blocks-category-split-panels-*` クラスセレクタを使う。レイアウト
//! root の class（`blocks-category-split-panels-layout`）は
//! [`Block::demo_class`]（`blocks-category-split-panels`）と意図的に
//! 別名にする（既存 block と同じ Bugbot 教訓の回避）。
//!
//! # link の `href` を固定の外部絶対 URL にする・可視テキストとの整合
//!
//! [`Block::demo`] は `fn() -> Node` のため `base_path` を受け取れない
//! （`category_featured_banner` と同じ制約）。2 枚のパネルの買い物導線
//! リンクは、可視テキストと遷移先がどちらも異なる実在の GitHub URL
//! （[`REPO`] と [`RELEASES`]）に固定し、`external: true` で
//! `rel="noopener noreferrer"` を付与する。死リンク `href="#"` は使わず、
//! 2 つのリンクを曖昧な重複リンクにしない（WCAG 2.4.4）。
//!
//! # `<form>` を持たない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。買い物導線リンクはいずれも `link::root` の通常の
//! ナビゲーションリンクであり、送信処理は持たない。文言はすべて架空の
//! もの（実企業名・実クレデンシャル・PII を含まない）。画像は
//! [`crate::blocks::dummy_assets`] のビルド時生成プレースホルダー SVG の
//! みを使い、`alt=""`（装飾扱い）で出力する（`data:` URI は使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps};

/// 買い物導線リンクの固定外部 URL（モジュール doc「link の `href` を
/// 固定の外部絶対 URL にする・可視テキストとの整合」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
const RELEASES: &str = "https://github.com/Fandhe-AI/fandhe-frontend/releases";

/// 1 パネル分のデータ（見出し・説明・背景画像・買い物導線リンク）。
struct Panel {
    title: &'static str,
    body: &'static str,
    image_src: &'static str,
    href: &'static str,
    link_label: &'static str,
}

const PANELS: [Panel; 2] = [
    Panel {
        title: "アウトドア用品",
        body: "軽量テントから調理器具まで、週末の遠出に必要な一式を集めました。",
        image_src: dummy_assets::BACKGROUND_SRC,
        href: REPO,
        link_label: "GitHub で見る",
    },
    Panel {
        title: "キッチン家電",
        body: "毎日の調理をすこし楽にする定番アイテムを集めました。",
        image_src: dummy_assets::PRODUCT_SRC,
        href: RELEASES,
        link_label: "リリースを見る",
    },
];

/// 1 パネル分の構成（背景画像 + 淡い面に重ねた見出し・説明・リンク）。
/// 画像と面を同じ `grid-area: 1 / 1` で重ねる（モジュール doc「重ね合わせ
/// は `grid-area` の共有で行う」節）。
fn panel(p: &Panel) -> Node {
    div(
        vec![("class", "blocks-category-split-panels-panel")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    ..ImageProps::new(p.image_src, "")
                },
                vec![("data-blocks-category-split-panels-image", "")],
            ),
            div(
                vec![("class", "blocks-category-split-panels-surface")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text(p.title)],
                    ),
                    styled_text::text(&TextProps::default(), vec![], vec![text(p.body)]),
                    link::root(
                        p.href,
                        &LinkProps {
                            external: true,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text(p.link_label)],
                    ),
                ],
            ),
        ],
    )
}

/// `category-split-panels` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-category-split-panels-layout")],
        PANELS.iter().map(panel).collect(),
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/category-split-panels/",
    title: "category-split-panels",
    category: BlockCategory::CategoryListing,
    rust_source: "crates/docs-site/src/blocks/ecommerce/category_listing/category_split_panels.rs",
    demo_class: "blocks-category-split-panels",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `category_split_panels` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「CSS の置き場」節、他 block と同型で本ファイル内
/// `const` として [`super::blocks`] から `BLOCK.layout_css` 経由で連結
/// される）。
///
/// セレクタは `.blocks-category-split-panels-*` と
/// `[data-blocks-category-split-panels-*]` のみを用い、他 block や
/// 部品の素のセレクタへ影響させない（既存 block と同じ名前空間分離）。
/// 色リテラル（`#fff`/`white`/`black` 等）は使わず、可読性の確保は
/// すべて `--fandhe-color-*` トークンと `color-mix()` で行う。
const LAYOUT_CSS: &str = "\
.blocks-category-split-panels-layout {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-category-split-panels;\n}\n\
.blocks-category-split-panels-panel {\n  display: grid;\n  min-height: 20rem;\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-category-split-panels-image] {\n  grid-area: 1 / 1;\n  display: block;\n  width: 100%;\n  height: 100%;\n  max-width: none;\n}\n\
.blocks-category-split-panels-surface {\n  grid-area: 1 / 1;\n  align-self: end;\n  margin: var(--fandhe-space-4);\n  padding: var(--fandhe-space-6);\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  align-items: start;\n  background: color-mix(in srgb, var(--fandhe-color-bg) 88%, transparent);\n  color: var(--fandhe-color-fg);\n  border-radius: var(--fandhe-radius-md);\n}\n\
@container blocks-category-split-panels (min-width: 40rem) {\n  \
.blocks-category-split-panels-layout {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  \
.blocks-category-split-panels-panel {\n    min-height: 24rem;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, RELEASES, REPO};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<img").count(), 2);
        assert_eq!(
            html.matches("data-blocks-category-split-panels-image=\"\"")
                .count(),
            2
        );
        assert!(html.contains(&format!("href=\"{REPO}\"")));
        assert!(html.contains(&format!("href=\"{RELEASES}\"")));
        for absent in [
            "<form",
            "src=\"data:",
            "href=\"#\"",
            " id=\"",
            "data-scope=\"button\"",
            "type=\"button\"",
        ] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
    }

    /// [`LAYOUT_CSS`] がコンテナクエリ（`@media` ではない）とトークン
    /// ベースの配色のみを使うこと。
    #[test]
    fn layout_css_uses_container_query_and_tokens_only() {
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-category-split-panels (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("grid-area: 1 / 1"));
        assert!(LAYOUT_CSS.contains("color-mix("));
        assert!(!LAYOUT_CSS.contains("@media"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("black"));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-category-split-panels-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-category-split-panels-layout"
        );
    }
}

//! `incentives-split-header` block（イシュー #3052。親トラッキングは
//! #3024「Blocks 目的別パーツ拡充ツリー」配下、Ecommerce / Incentives
//! カテゴリの最初の block。対応表 ID は R1015（導入 2 列 + 特典 3 点）
//! のみで、集約元は 1 件のため Demo は 1 形で足りる。取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない。
//!
//! # 使用部品
//!
//! `heading` / `text` / `image` / `icon` の 4 部品のみを合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新規 UI 部品・外部依存は追加しない。
//!
//! # レイアウト
//!
//! 上段は 2 列（左: 見出し + 本文、右: 画像）、下段はアイコン付き特典
//! 項目 3 点を横に並べる。md（48rem）未満は上段・下段とも 1 列に積む
//! （`feature_three_column_icons` と同じブレークポイント判断、テーマの
//! breakpoint トークンは `@media` 条件式の中では解決できないためリテラル
//! 値を直書きする）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、導入見出しは
//! `HeadingLevel::H3`、特典項目の見出しはそれより 1 段下げて
//! `HeadingLevel::H4` にする。
//!
//! # アイコンを装飾扱いにする理由
//!
//! 項目見出し・説明が既に意味を伝えているため、装飾アイコンは
//! [`IconProps::label`] を `None` にする（`feature_three_column_icons::
//! geo_icon` と同型の判断）。自作の単純な幾何パスのみを使い、lucide 等の
//! 著作物は複製しない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading`/`styled_text::text`/`image::image`/`icon::icon` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って
//! 除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-incentives-split-header-*` 属性で渡す。素の `div` には
//! `class` がそのまま効くため `.blocks-incentives-split-header-*`
//! クラスセレクタを使う。レイアウト root の class
//! （`blocks-incentives-split-header-layout`）は [`Block::demo_class`]
//! （`blocks-incentives-split-header`）とは意図的に別名にする（先行 block
//! で得た Bugbot 教訓の踏襲）。
//!
//! # `<form>` を持たない・依存追加なし
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。画像は [`crate::blocks::dummy_assets`] のビルド時生成
//! プレースホルダー SVG のみを使い、`alt=""`（装飾扱い）で出力する
//! （`data:` URI は使わない）。新規 UI 部品・新規外部クレート依存は追加
//! しない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 特典項目 1 件分の架空データ（見出し・説明・自作アイコンのパス）。
struct Incentive {
    title: &'static str,
    body: &'static str,
    icon_path_d: &'static str,
}

/// 特典項目 3 件（架空、実在の企業・製品とは無関係）。自作の単純な幾何
/// パスのみを使い、lucide 等の著作物は複製しない。
const INCENTIVES: [Incentive; 3] = [
    Incentive {
        title: "送料無料",
        body: "注文金額にかかわらず、国内配送はすべて無料です。",
        icon_path_d: "M3 7h11v9H3zM14 10h4l3 3v3h-7zM6.5 19a1.5 1.5 0 100-3 1.5 1.5 0 000 3zM17.5 19a1.5 1.5 0 100-3 1.5 1.5 0 000 3z",
    },
    Incentive {
        title: "30 日以内の返品",
        body: "到着から 30 日以内であれば、理由を問わず返品できます。",
        icon_path_d: "M4 4v6h6M4.5 15a8 8 0 108-11.3",
    },
    Incentive {
        title: "サポート窓口",
        body: "注文に関するお問い合わせに、専任スタッフが対応します。",
        icon_path_d: "M12 21a9 9 0 100-18 9 9 0 000 18zM12 8v5l3 3",
    },
];

/// 装飾用の自作幾何アイコン（`fill="none"` + `stroke="currentColor"` の
/// 線画、`feature_three_column_icons::geo_icon` と同型の判断）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Xl,
            ..IconProps::default()
        },
        vec![("data-blocks-incentives-split-header-icon", "")],
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

/// 左列（見出し + リード文）。
fn intro() -> Node {
    div(
        vec![("class", "blocks-incentives-split-header-intro")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("安心してお買い物いただくために")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-incentives-split-header-lead", "")],
                vec![text(
                    "送料・返品・サポートのすべてで、ご注文の不安を取り除きます。",
                )],
            ),
        ],
    )
}

/// 右列（4:3 画像、装飾扱いの `alt=""`）。
fn media() -> Node {
    div(
        vec![("class", "blocks-incentives-split-header-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Landscape,
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
            },
            vec![("data-blocks-incentives-split-header-image", "")],
        )],
    )
}

/// 特典項目 1 件（アイコン → 見出し → 説明）。
fn incentive_item(i: &Incentive) -> Node {
    div(
        vec![("class", "blocks-incentives-split-header-item")],
        vec![
            geo_icon(i.icon_path_d),
            heading::heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(i.title)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-incentives-split-header-item-desc", "")],
                vec![text(i.body)],
            ),
        ],
    )
}

/// `incentives-split-header` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-incentives-split-header-layout")],
        vec![
            div(
                vec![("class", "blocks-incentives-split-header-top")],
                vec![intro(), media()],
            ),
            div(
                vec![("class", "blocks-incentives-split-header-items")],
                INCENTIVES.iter().map(incentive_item).collect(),
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/incentives-split-header/",
    title: "incentives-split-header",
    category: BlockCategory::Incentives,
    rust_source: "crates/docs-site/src/blocks/ecommerce/incentives/incentives_split_header.rs",
    demo_class: "blocks-incentives-split-header",
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
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `incentives_split_header` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-incentives-split-header-*` と `[data-blocks-incentives-split-
/// header-*]` のみを用い、他 block や部品の素のセレクタへ影響させない
/// （`feature_three_column_icons` と同じ名前空間分離）。ブレークポイント
/// のリテラル 48rem は `recipe::Breakpoint::Md`（768px）と一致させる
/// （テーマの breakpoint トークンは `@media` 条件の中では解決できない
/// ため）。
const LAYOUT_CSS: &str = "\
.blocks-incentives-split-header-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-incentives-split-header-top {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-incentives-split-header-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  justify-content: center;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-incentives-split-header-lead] {\n  margin: 0;\n}\n\
.blocks-incentives-split-header-media {\n  min-width: 0;\n}\n\
.blocks-incentives-split-header-items {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-incentives-split-header-item {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  min-width: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-incentives-split-header-item-desc] {\n  margin: 0;\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-incentives-split-header-top {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    align-items: center;\n  }\n  \
.blocks-incentives-split-header-items {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていること
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("class=\"blocks-incentives-split-header-item\"")
                .count(),
            3
        );
        assert_eq!(
            html.matches("data-blocks-incentives-split-header-icon=\"\"")
                .count(),
            3
        );
        assert_eq!(html.matches("<img").count(), 1);
        assert!(!html.contains("<form"));
        assert!(!html.contains("<button"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 画像に同梱プレースホルダー SVG を使うこと（`data:` URI ではない）。
    #[test]
    fn media_uses_bundled_placeholder() {
        let html = render(&demo());
        assert!(html.contains(super::dummy_assets::PRODUCT_SRC));
    }

    /// [`LAYOUT_CSS`] が想定する md ブレークポイント・2 列/3 列切り替えを
    /// 持つこと。
    #[test]
    fn layout_css_declares_md_breakpoint_grids() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("repeat(3, minmax(0, 1fr))"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（`feature_three_column_icons` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-incentives-split-header-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-incentives-split-header-layout"
        );
    }
}

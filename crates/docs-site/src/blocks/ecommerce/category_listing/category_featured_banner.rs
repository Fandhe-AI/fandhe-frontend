//! `category-featured-banner` block（イシュー #3036。親トラッキング #3024
//! 「Blocks EC」配下、`crate::blocks::ecommerce::category_listing` カテゴリ
//! 最初の block。本カテゴリをイシュー #2734 の空雛形からディレクトリ化
//! する）。
//!
//! 1 カテゴリだけを大きく扱う横長バナーを、主参照 R0823（全面画像 +
//! 半透明パネル）に集約元 R0608（画像 + テキストの左右分割カード）を
//! 集約した 2 形として合成する。取得手段・ファイル名・内部コンポーネント
//! 識別子は記載しない（対応表 ID のみを記す、`super::super::super::marketing::
//! cta::cta_split_image` と同じ転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `image` / `link` / `button` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。
//!
//! # 2 形を 1 つの Demo に並記する
//!
//! [`super::super::marketing::cta::cta_split_image`] と同型に、2 形を
//! [`variant_label`] で見出しを付けながら [`demo`] 1 つの中へ縦に並べる。
//!
//! | 形 | 対応 ID | 内容 |
//! |----|---------|------|
//! | A 主参照 | R0823 | 全面画像 + 重ねた半透明パネル |
//! | B 集約元 | R0608 | 画像 + テキストの左右分割 |
//!
//! # A: 絶対配置を使わず `grid-area` の重ね合わせで画像とパネルを重ねる
//!
//! 狭幅（通常フロー）では画像の下にパネルが来るだけでよいため、広幅
//! （`48rem` 以上）でのみ root を `display: grid` にし、画像・パネルの
//! 両方へ同じ `grid-area: 1 / 1` を与えて同一セルへ重ねる
//! （`position: absolute` を使わないため、狭幅での「画像の下へ回す」
//! 挙動が素の通常フローのままで成立する）。
//!
//! # A: パネルの反転配色
//!
//! [`super::super::marketing::testimonial::testimonial_background_image`]
//! と同じ反転ペアを採用する: パネル背景を `--fandhe-color-fg` ベースの
//! 半透明、文字を `--fandhe-color-bg` ベースにする。色リテラル
//! （`#…`/`white`/`black`）は使わず `--fandhe-*` トークンと `color-mix()`
//! のみで表現する。
//!
//! # A: パネル上のボタン・リンクの反転上書き
//!
//! [`button::button`] はどの `ButtonVariant` でも独自の `background`/
//! `color` を持つため、`[data-scope="button"][data-part="root"]
//! [data-blocks-category-featured-banner-cta]` へ `background:
//! var(--fandhe-color-bg); color: var(--fandhe-color-fg);` を明示上書きし、
//! パネルの暗い半透明面でもコントラストを確保する（`cta_split_image` の
//! B と同じ判断）。[`link::root`] も同様に
//! `[data-scope="link"][data-part="root"]
//! [data-blocks-category-featured-banner-link]` へ `color:
//! var(--fandhe-color-bg);` を明示上書きする。
//!
//! # 詳細度: `[data-scope]` を含めた 3 セレクタ構成
//!
//! [`button::button`]/[`image::image`] の recipe（詳細度 (0,2,0) 程度）に
//! 確実に勝つため、上書きは `data-scope`/`data-part` を含めた 3 セレクタ
//! 構成（詳細度 (0,3,0) 以上）で行う（既存 block と同じ判断軸）。
//!
//! # link の `href` を固定の外部絶対 URL にする
//!
//! [`Block::demo`] は `fn() -> Node` のため `base_path` を受け取れない
//! （`cta_split_image` と同じ制約）。誘導リンクは実在する GitHub
//! リポジトリへの外部絶対 URL（[`REPO`]）に固定し、`external: true` で
//! `rel="noopener noreferrer"` を付与する。死リンク `href="#"` は使わない。
//!
//! # ブレークポイントをリテラルで直書きする理由
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、`48rem` をリテラルで
//! 直書きする（既存 block と同じ判断、`testimonial_background_image` と
//! 同じ値を採用）。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `heading::heading`/`text::text`/`image::image`/`link::root`/
//! `button::button` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有の
//! スタイルフックは `data-blocks-category-featured-banner-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div` には
//! `class` がそのまま効くため、レイアウト用の入れ子は従来どおり
//! `.blocks-category-featured-banner-*` クラスセレクタを使う。レイアウト
//! root の class（`blocks-category-featured-banner-layout`）は
//! [`Block::demo_class`]（`blocks-category-featured-banner`）と意図的に
//! 別名にする（既存 block と同じ Bugbot 教訓の回避）。
//!
//! # `<form>` を持たない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは `button::button` の既定 `type="button"` のまま
//! 送信先を持たず、値は一切送信されない。文言はすべて架空のもの（実
//! 企業名・実クレデンシャル・PII を含まない）。画像は
//! [`crate::blocks::dummy_assets`] のビルド時生成プレースホルダー SVG の
//! みを使い、`alt=""`（装飾扱い）で出力する（`data:` URI は使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「link の `href` を固定の外部
/// 絶対 URL にする」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 両形で共通のコピー列（小見出し・見出し・説明・CTA ボタン + 誘導
/// リンク）。`inverted` が `true` のとき（形 A のパネル上）はボタン・
/// リンクへ反転配色のフックを付与する。
fn copy_column(
    eyebrow: &'static str,
    title: &'static str,
    body: &'static str,
    cta: &'static str,
    link_label: &'static str,
    inverted: bool,
) -> Node {
    let cta_attrs = if inverted {
        vec![("data-blocks-category-featured-banner-cta", "")]
    } else {
        vec![]
    };
    let link_attrs = if inverted {
        vec![("data-blocks-category-featured-banner-link", "")]
    } else {
        vec![]
    };

    div(
        vec![("class", "blocks-category-featured-banner-copy")],
        vec![
            variant_label(eyebrow),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(&TextProps::default(), vec![], vec![text(body)]),
            div(
                vec![("class", "blocks-category-featured-banner-actions")],
                vec![
                    button::button(
                        &ButtonProps {
                            size: Size::Lg,
                            ..ButtonProps::default()
                        },
                        cta_attrs,
                        vec![text(cta)],
                    ),
                    link::root(
                        REPO,
                        &LinkProps {
                            external: true,
                            ..LinkProps::default()
                        },
                        link_attrs,
                        vec![text(link_label)],
                    ),
                ],
            ),
        ],
    )
}

/// 形 A（R0823 主参照）: 全面画像 + 重ねた半透明パネル。広幅では画像の
/// 上にパネルが重なり、狭幅では画像の下にパネルが回る（モジュール doc
/// 「A: 絶対配置を使わず `grid-area` の重ね合わせで画像とパネルを重ねる」
/// 節）。
fn variant_overlay() -> Node {
    let panel = div(
        vec![("class", "blocks-category-featured-banner-panel")],
        vec![copy_column(
            "今季の注目",
            "アウトドア用品",
            "軽量テントから調理器具まで、週末の遠出に必要な一式をまとめました。",
            "カテゴリを見る",
            "すべてのカテゴリを見る",
            true,
        )],
    );

    div(
        vec![("class", "blocks-category-featured-banner-overlay")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
                },
                vec![("data-blocks-category-featured-banner-overlay-image", "")],
            ),
            panel,
        ],
    )
}

/// 形 B（R0608 集約）: 画像 + テキストの左右分割。狭幅では縦積み、広幅
/// （`48rem` 以上）では 2 列グリッドになる。
fn variant_split() -> Node {
    div(
        vec![("class", "blocks-category-featured-banner-split")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-category-featured-banner-split-image", "")],
            ),
            copy_column(
                "人気急上昇",
                "キッチン家電",
                "毎日の調理をすこし楽にする定番アイテムを集めました。",
                "詳しく見る",
                "すべてのカテゴリを見る",
                false,
            ),
        ],
    )
}

/// `category-featured-banner` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数（モジュール doc「2 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-category-featured-banner-layout")],
        vec![
            variant_label("全面画像 + 半透明パネル（R0823 主参照）"),
            variant_overlay(),
            variant_label("画像 + テキストの左右分割（R0608 集約）"),
            variant_split(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/category-featured-banner/",
    title: "category-featured-banner",
    category: BlockCategory::CategoryListing,
    rust_source:
        "crates/docs-site/src/blocks/ecommerce/category_listing/category_featured_banner.rs",
    demo_class: "blocks-category-featured-banner",
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
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `category_featured_banner` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「CSS の置き場」節、他 block と同型で本ファイル内
/// `const` として [`super::blocks`] から `BLOCK.layout_css` 経由で連結
/// される）。
///
/// セレクタは `.blocks-category-featured-banner-*` と
/// `[data-blocks-category-featured-banner-*]` のみを用い、他 block や
/// 部品の素のセレクタへ影響させない（既存 block と同じ名前空間分離）。
/// 色リテラル（`#fff`/`white`/`black` 等）は使わず、可読性の確保は
/// すべて `--fandhe-color-*` トークンと `color-mix()` で行う。
const LAYOUT_CSS: &str = "\
.blocks-category-featured-banner-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-category-featured-banner-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  align-items: start;\n  min-width: 0;\n}\n\
.blocks-category-featured-banner-actions {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-4);\n  align-items: center;\n}\n\
.blocks-category-featured-banner-overlay {\n  display: flex;\n  flex-direction: column;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-category-featured-banner-overlay-image] {\n  display: block;\n  width: 100%;\n  height: 14rem;\n}\n\
.blocks-category-featured-banner-panel {\n  box-sizing: border-box;\n  padding: var(--fandhe-space-6);\n  background: color-mix(in srgb, var(--fandhe-color-fg) 80%, transparent);\n  color: var(--fandhe-color-bg);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-category-featured-banner-cta] {\n  background: var(--fandhe-color-bg);\n  color: var(--fandhe-color-fg);\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-category-featured-banner-link] {\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-category-featured-banner-split {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-category-featured-banner-split-image] {\n  display: block;\n  width: 100%;\n  height: 14rem;\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-category-featured-banner-overlay {\n    display: grid;\n    min-height: 24rem;\n  }\n  \
[data-scope=\"image\"][data-part=\"root\"][data-blocks-category-featured-banner-overlay-image] {\n    grid-area: 1 / 1;\n    width: 100%;\n    height: 100%;\n  }\n  \
.blocks-category-featured-banner-panel {\n    grid-area: 1 / 1;\n    align-self: end;\n    justify-self: start;\n    max-width: 28rem;\n    margin: var(--fandhe-space-6);\n  }\n  \
.blocks-category-featured-banner-split {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    gap: var(--fandhe-space-8);\n    align-items: center;\n  }\n  \
[data-scope=\"image\"][data-part=\"root\"][data-blocks-category-featured-banner-split-image] {\n    height: 100%;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
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
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<img").count(), 2);
        assert!(html.contains("type=\"button\""));
        for absent in ["<form", "src=\"data:", "href=\"#\"", " id=\""] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・重ね合わせ用の
    /// `grid-area` を持つこと（色リテラルは使わず `color-mix()` +
    /// トークン参照のみであること）。
    #[test]
    fn layout_css_declares_breakpoint_and_overlay_grid_area() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("grid-area: 1 / 1"));
        assert!(LAYOUT_CSS.contains("color-mix("));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("black"));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-category-featured-banner-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-category-featured-banner-layout"
        );
    }
}

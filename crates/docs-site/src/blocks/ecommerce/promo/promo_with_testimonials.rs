//! `promo-with-testimonials` block（イシュー #3086。Ecommerce / Promo
//! カテゴリ、`promo_offers_split` に続く本カテゴリ 4 件目）。
//!
//! # 出典に関する注記
//!
//! 主参照は対応表 ID R1195（集約元も R1195 の 1 件のみ）。`_/blocks-intake/`
//! はメイン worktree に存在せず参照ファイルは未参照のため、構造はイシューの
//! レイアウト仕様のみを根拠に独自実装する（並記すべき差分なし）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `button` / `image` / `blockquote` / `icon` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI
//! 部品は追加しない。
//!
//! # `drop_class_attr` を踏まえた CSS フックの選び方
//!
//! `heading::heading` / `styled_text::text` / `button::button` /
//! `image::image` / `blockquote::root` / `icon::icon` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、本 block 固有のフックは `data-blocks-promo-with-
//! testimonials-*` 属性で渡す。素の `div`/`section` には `class` がそのまま
//! 効くため `.blocks-promo-with-testimonials-*` クラスセレクタを使う
//! （`promo_offers_split` と同じ判断軸）。
//!
//! # 覆いを bg ベースの `color-mix` にした理由
//!
//! 背景画像の上に淡い覆い（veil）を重ね、上段の見出し・説明・CTA と下段の
//! 引用カードを読みやすくする。暗幕側を暗くするのではなく
//! `--fandhe-color-bg` を 85% 混ぜた淡い面にするのは、文字色を通常の
//! `--fandhe-color-fg`（反転なし）のまま保てるからである
//! （`testimonial_background_image` の反転ペア方式とは逆の設計判断。
//! 本 block は背景画像を装飾の薄いテクスチャ程度に留め、カード面で主な
//! コントラストを作る構成のため、全面反転は不要と判断した）。
//!
//! # 詳細度を上げて上書きする方針
//!
//! `image::image`/`blockquote::root` の recipe base 宣言（詳細度 0,2,0）に
//! 勝たせるため、本 block 固有フックは同じ属性セレクタへ前置し
//! （`[data-scope="image"][data-part="root"][data-blocks-promo-with-
//! testimonials-*]` 等、詳細度 0,3,0 以上）、ソース順
//! （`blocks.css` は `pre-styled-ui.css` より後に読み込まれる）ではなく
//! 詳細度そのもので上書きを確定させる（`testimonial_background_image` と
//! 同じ教訓）。
//!
//! # `48rem` のリテラルを直書きする理由
//!
//! `pre-styled-ui` の breakpoint トークン
//! （[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]、768px =
//! 48rem）は `SlotRecipe` 経由の変数生成専用であり、docs-site の生 CSS へ
//! 直接参照する経路を持たないため、一致するリテラル値を直書きする
//! （既存 Blocks 全件と同じ判断）。md 未満は引用カードを 1 列、`48rem`
//! 以上で 3 列へ切り替える。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。CTA ボタンは `button::button` の既定 `type="button"` の
//! まま用いる。状態を持たない静的な表示のみで、送信処理・データ取得は
//! 一切行わない。キャンペーン文言・引用・人名・役職はすべて架空のもので
//! あり、実企業名・実サービス名・実クレデンシャル・PII を含まない。画像は
//! `dummy_assets::BACKGROUND_SRC`（ビルド時生成のモノトーンプレースホルダー
//! SVG）を使い回す。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::ColorPalette;

const ROOT_CLASS: &str = "blocks-promo-with-testimonials-layout";
const BACKDROP_CLASS: &str = "blocks-promo-with-testimonials-backdrop";
const VEIL_CLASS: &str = "blocks-promo-with-testimonials-veil";
const PROMO_CLASS: &str = "blocks-promo-with-testimonials-promo";
const QUOTES_CLASS: &str = "blocks-promo-with-testimonials-quotes";

const IMAGE_ATTR: &str = "data-blocks-promo-with-testimonials-image";
const CTA_ATTR: &str = "data-blocks-promo-with-testimonials-cta";
const QUOTE_ATTR: &str = "data-blocks-promo-with-testimonials-quote";
const NAME_ATTR: &str = "data-blocks-promo-with-testimonials-name";
const ROLE_ATTR: &str = "data-blocks-promo-with-testimonials-role";

/// 引用カードの先頭に置く装飾アイコン（抽象的な引用符の図形。実在ブランド
/// の図形は使わない）。`label: None`（既定）のため `aria-hidden="true"` が
/// 付く装飾アイコンになる。
fn quote_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![("d", "M7 7h4v6l-3 5H5l2-5H7V7Zm8 0h4v6l-3 5h-3l2-5h-0V7Z")],
            vec![],
        )],
    )
}

/// 引用カード 1 件（装飾アイコン + 引用文 + 発言者名・役職）。
fn testimonial(quote: &'static str, name: &'static str, role: &'static str) -> Node {
    blockquote::root(
        BlockquoteVariant::Plain,
        ColorPalette::default(),
        vec![(QUOTE_ATTR, "")],
        vec![
            quote_icon(),
            blockquote::content(vec![], vec![text(quote)]),
            blockquote::caption(
                vec![],
                vec![div(
                    vec![],
                    vec![
                        styled_text::text(
                            &TextProps {
                                weight: fandhe_frontend_pre_styled_ui::text::TextWeight::Semibold,
                                ..TextProps::default()
                            },
                            vec![(NAME_ATTR, "")],
                            vec![text(name)],
                        ),
                        styled_text::text(
                            &TextProps {
                                size: TextSize::Sm,
                                variant: TextVariant::Muted,
                                ..TextProps::default()
                            },
                            vec![(ROLE_ATTR, "")],
                            vec![text(role)],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// 上段（見出し・説明・CTA）。
fn promo() -> Node {
    div(
        vec![("class", PROMO_CLASS)],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("秋の感謝キャンペーン開催中")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "対象商品が期間限定で特別価格に。お客様の声とともにご案内します。",
                )],
            ),
            button::button(
                &ButtonProps::default(),
                vec![(CTA_ATTR, "")],
                vec![text("キャンペーンを見る")],
            ),
        ],
    )
}

/// `promo-with-testimonials` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ契約）。
pub fn demo() -> Node {
    let backdrop = div(
        vec![("class", BACKDROP_CLASS), ("aria-hidden", "true")],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                vec![(IMAGE_ATTR, "")],
            ),
            div(vec![("class", VEIL_CLASS)], vec![]),
        ],
    );

    let quotes = div(
        vec![("class", QUOTES_CLASS)],
        (0..3)
            .map(|index| {
                testimonial(
                    dummy_assets::TESTIMONIAL_QUOTES[index],
                    dummy_assets::PERSON_NAMES[index],
                    dummy_assets::JOB_TITLES[index],
                )
            })
            .collect(),
    );

    div(vec![("class", ROOT_CLASS)], vec![backdrop, promo(), quotes])
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/promo-with-testimonials/",
    title: "promo-with-testimonials",
    category: BlockCategory::Promo,
    rust_source: "crates/docs-site/src/blocks/ecommerce/promo/promo_with_testimonials.rs",
    demo_class: "blocks-promo-with-testimonials",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Blockquote",
            path: "/themes/blockquote/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `promo_with_testimonials` 固有のレイアウト規則（`--fandhe-*` トークンと
/// `color-mix()` のみを使用）。ブレークポイントは
/// [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]（768px =
/// 48rem）と一致するリテラル値を直書きする（モジュール冒頭「`48rem` の
/// リテラルを直書きする理由」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-promo-with-testimonials {\n  padding: 0;\n  overflow: hidden;\n}\n\
.blocks-promo-with-testimonials-layout {\n  position: relative;\n  isolation: isolate;\n  display: grid;\n  gap: var(--fandhe-space-8);\n  padding: var(--fandhe-space-6) var(--fandhe-space-4);\n}\n\
.blocks-promo-with-testimonials-backdrop {\n  position: absolute;\n  inset: 0;\n  z-index: -1;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-promo-with-testimonials-image] {\n  width: 100%;\n  height: 100%;\n  display: block;\n  object-fit: cover;\n}\n\
.blocks-promo-with-testimonials-veil {\n  position: absolute;\n  inset: 0;\n  background: color-mix(in srgb, var(--fandhe-color-bg) 85%, transparent);\n}\n\
.blocks-promo-with-testimonials-promo {\n  display: grid;\n  gap: var(--fandhe-space-4);\n  max-width: 40rem;\n  justify-items: start;\n}\n\
.blocks-promo-with-testimonials-quotes {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-promo-with-testimonials-quotes [data-scope=\"blockquote\"][data-part=\"root\"][data-blocks-promo-with-testimonials-quote] {\n  display: grid;\n  gap: var(--fandhe-space-3);\n  border-inline-start: none;\n  padding: var(--fandhe-space-4);\n  border-radius: var(--fandhe-radius-lg);\n  background: color-mix(in srgb, var(--fandhe-color-bg) 94%, var(--fandhe-color-fg));\n}\n\
@media (min-width: 48rem) {\n  .blocks-promo-with-testimonials-quotes {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] が背景画像・覆い・上段（見出し/説明/CTA）・引用カード 3 件
    /// （装飾アイコン・引用文・発言者名・役職）を正しく構成し、禁止パターン
    /// （`<form>`/`type="submit"`/`href="#"`/`data:` URI/`<script`/`id="`）を
    /// 含まないこと（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_renders_promo_and_three_testimonials_and_avoids_disallowed_patterns() {
        let html = render(&demo());
        assert_eq!(
            html.matches(QUOTE_ATTR).count(),
            3,
            "should render exactly 3 blockquote cards"
        );
        assert_eq!(
            html.matches("<figcaption").count(),
            3,
            "should render exactly 3 figcaptions"
        );
        assert_eq!(
            html.matches(r#"type="button""#).count(),
            1,
            "should render exactly 1 button"
        );
        assert!(html.contains(dummy_assets::BACKGROUND_SRC));
        for index in 0..3 {
            assert!(html.contains(dummy_assets::TESTIMONIAL_QUOTES[index]));
            assert!(html.contains(dummy_assets::PERSON_NAMES[index]));
            assert!(html.contains(dummy_assets::JOB_TITLES[index]));
        }
        assert_eq!(
            html.matches(r#"aria-hidden="true""#).count(),
            4,
            "backdrop + 3 decorative quote icons should be aria-hidden"
        );
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "<script",
            "id=\"",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が全主要セレクタ・`48rem` ブレークポイント・
    /// `color-mix()` + トークン参照を満たし、色リテラルを含まないこと。
    #[test]
    fn layout_css_declares_all_selectors_and_uses_tokens_not_literals() {
        for selector in [
            ".blocks-promo-with-testimonials {",
            ".blocks-promo-with-testimonials-layout {",
            ".blocks-promo-with-testimonials-backdrop {",
            "[data-scope=\"image\"][data-part=\"root\"][data-blocks-promo-with-testimonials-image] {",
            ".blocks-promo-with-testimonials-veil {",
            ".blocks-promo-with-testimonials-promo {",
            ".blocks-promo-with-testimonials-quotes {",
            "@media (min-width: 48rem) {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("color-mix("));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-bg)"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("black"));
        assert!(!LAYOUT_CSS.contains("animation"));
        assert!(!LAYOUT_CSS.contains("transition"));
    }

    /// ルート class が [`demo`] の出力へ実際に現れ、かつ `BLOCK.demo_class`
    /// とは異なること（既存 Blocks と同じ教訓: 同一名だと `.blocks-demo`
    /// 側の共通ラッパクラスと衝突する）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains(ROOT_CLASS));
        assert_ne!(ROOT_CLASS, BLOCK.demo_class);
    }
}

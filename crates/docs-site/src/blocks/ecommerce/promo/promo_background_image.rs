//! `promo-background-image` block（イシュー #3076。親トラッキング #3024。
//! Ecommerce / Promo カテゴリ 2 件目）。
//!
//! # 出典に関する注記
//!
//! 主参照は対応表 ID R1196（全面背景画像 + 暗幕 + 中央寄せ CTA の基準形）。
//! 角丸カードへ収める形（R1198）とトップページ向けの大見出し・大余白形
//! （R1202）を並記形として集約する（取得手段・ファイル名・内部コンポー
//! ネント識別子は記載しない）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `link` / `image` / `card` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は追加しない。
//! CTA は `button::button`（遷移先を持たない `<button>`）ではなく
//! `link::root`（`<a>`）を使う。「見る」という遷移を示す文言には実在する
//! リンク先が要る（codex-review P1 指摘、イシュー #3076。`cta_split_image`
//! の「導入事例を見る」と同型の判断）。
//!
//! # 3 形を 1 つの Demo に並記する
//!
//! [`super::super::hero::hero_background_media`]・
//! [`super::super::cta::cta_split_image`] と同型に、[`variant_label`] で
//! 短いラベルを付けながら形 A（基準形）・形 B（カード形）・形 C
//! （トップページ形）を [`demo`] 1 つの中へ縦に並べる。
//!
//! 形 C はトップページでは `HeadingLevel::H1` を想定するが、Demo 内では
//! 他 2 形と同じ `HeadingLevel::H3` を使う。docs ページ本文の `<h1>` は
//! 1 個に保つ契約（`crate::blocks` モジュール doc）のため、Demo では
//! 見た目のサイズ（`HeadingSize::Xl4`）と余白の差だけを再現する
//! （原稿の差分メモにも明記する）。
//!
//! # 背景は装飾扱い（`aria-hidden` + 空 `alt`）
//!
//! `crate::blocks::dummy_assets::BACKGROUND_SRC`（ビルド時生成のモノトーン
//! 背景タイル SVG）を 3 形で使い回す。背景画像とスクリムは
//! `aria-hidden="true"` の `div` で包み、スクリーンリーダーからは
//! 読み上げ対象外にする（`promo_collection_cards`/`hero_background_media`
//! と同型）。
//!
//! # 暗幕は `color-mix` + トークンで作る
//!
//! 背景画像の上へ `--fandhe-color-fg` を `color-mix()` で半透明化した
//! スクリムを重ね、コンテンツは `--fandhe-color-bg` トークンで描く。
//! ライトテーマでは「暗幕 + 明るい文字」になるが、ダークテーマでは
//! 前景/背景の意味が反転し「明るい幕 + 暗い文字」になる既知の挙動である
//! （`hero_background_media`/`promo_collection_cards` と同じ判断、原稿の
//! 差分メモにも明記する）。色リテラル（`#`/`white` 等）は使わない。
//!
//! # 反転色 CTA も詳細度対策が要る
//!
//! `[data-blocks-promo-background-image-cta]` 単体（属性セレクタ 1 個、
//! 詳細度 0,1,0）では `link::root` の base 宣言
//! （`[data-scope="link"][data-part="root"]`、詳細度 0,2,0）に確実に
//! 負けるため、`[data-scope="link"][data-part="root"]` を前置して
//! 詳細度 0,3,0 に揃える（`cta_split_image` と同じ解法。CSS 出力順は
//! showcase → blocks の順で `<link>` するため、同値セレクタは後勝ちで
//! 確実に本 block 側が勝つ）。背景暗幕の上でもボタン然として視認できる
//! よう、`padding`/`border-radius`/`display: inline-block` を本 block 側で
//! 追加付与する（`link::root` base は装飾なしのインラインテキストリンク
//! のため）。`:hover` にも同じ指定へ `color-mix()` を加えて淡くし、recipe
//! の hover 宣言に負けないようにする。
//!
//! # B 形（カード）は `card::root` を (0,3,0) で上書きする
//!
//! `card::root` の base 宣言（`padding`/`border`）を打ち消し、背景画像・
//! スクリム・コンテンツをカードの内側全面へ敷くため、
//! `[data-scope="card"][data-part="root"][data-blocks-promo-background-
//! image-card]` へ `overflow: hidden; padding: 0; border: 0;` を当てる
//! （`promo_collection_cards` の "card 全体をリンク化" 節と同型の詳細度
//! 対策）。
//!
//! # 重なり順・中央寄せ
//!
//! 外枠に `position: relative; isolation: isolate; overflow: hidden;` を、
//! backdrop に `position: absolute; inset: 0; z-index: -1;` を指定する
//! （`hero_background_media` と同型）。コンテンツは `display: grid;
//! justify-items: center; text-align: center;` で中央寄せにする。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading`/`styled_text::text`/`link::root`/`card::root`/
//! `image::image` はいずれも `drop_class_attr` により呼び出し側 `attrs`
//! の `class` を黙って除去する契約を持つため、本 block 固有のフックは
//! `data-blocks-promo-background-image-*` 属性で渡す。素の `div` には
//! `class` がそのまま効くため、レイアウト・背景レイヤーは
//! `.blocks-promo-background-image-*` クラスセレクタで扱う。
//!
//! # `id` を使わない・`<form>` を使わない・実データを持たない
//!
//! 3 形とも `id` 属性を使わず（重複 id 回避）。`crate::blocks` モジュール
//! doc の不変条件どおり `<form>` を出力しない。CTA は `link::root` が
//! 固定の外部絶対 URL（[`REPO`]）へ遷移する実在のリンクであり、
//! `href="#"` の死リンクは使わない（`cta_split_image`/`promo_collection_
//! cards` 等、既存 block 多数と同じ方針。モジュール冒頭「使用部品」節
//! 参照）。文言はすべて架空のものであり、実企業名・実サービス名・実
//! クレデンシャル・PII・価格の断定は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// CTA のリンク先（モジュール冒頭「`id` を使わない・`<form>` を使わない・
/// 実データを持たない」節参照）。本 Demo は実データ・バックエンドを持たない
/// 静的合成例のため、`cta_split_image`/`promo_collection_cards` 等の既存
/// block と同じく固定の外部絶対 URL を死リンク回避先として使い回す。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 各形の直前に置く短い形ラベル（`hero_background_media::variant_label`
/// と同型）。
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

/// 背景画像 + 暗幕の 2 層（`aria-hidden` で装飾扱い、3 形で共通）。
fn backdrop() -> Node {
    div(
        vec![
            ("class", "blocks-promo-background-image-backdrop"),
            ("aria-hidden", "true"),
        ],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                vec![("data-blocks-promo-background-image-image", "")],
            ),
            div(
                vec![("class", "blocks-promo-background-image-scrim")],
                vec![],
            ),
        ],
    )
}

/// 中央寄せの見出し・説明・反転色 CTA（`heading_size` のみ形ごとに変える）。
fn content(heading_size: HeadingSize) -> Node {
    div(
        vec![("class", "blocks-promo-background-image-content")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: heading_size,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-promo-background-image-title", "")],
                vec![text("季節の入れ替えセール")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![("data-blocks-promo-background-image-lead", "")],
                vec![text("対象の定番アイテムが期間限定でお得になります。")],
            ),
            link::root(
                REPO,
                &LinkProps::default(),
                vec![("data-blocks-promo-background-image-cta", "")],
                vec![text("セール会場を見る")],
            ),
        ],
    )
}

/// 形 A（R1196 基準形）: 全面背景画像 + 暗幕 + 中央寄せコンテンツ。
fn variant_basic() -> Node {
    div(
        vec![("class", "blocks-promo-background-image-basic")],
        vec![backdrop(), content(HeadingSize::Xl3)],
    )
}

/// 形 B（R1198）: 角丸カードの内側全面へ背景画像・暗幕・コンテンツを敷く。
fn variant_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-promo-background-image-card", "")],
        vec![backdrop(), content(HeadingSize::Xl3)],
    )
}

/// 形 C（R1202）: トップページ向け。h1 相当の大見出し・大きめの余白
/// （Demo では `HeadingLevel::H3` のまま、サイズと余白だけを再現する。
/// モジュール冒頭「3 形を 1 つの Demo に並記する」節参照）。
fn variant_top_page() -> Node {
    div(
        vec![("class", "blocks-promo-background-image-top-page")],
        vec![backdrop(), content(HeadingSize::Xl4)],
    )
}

/// `promo-background-image` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-promo-background-image-layout")],
        vec![
            variant_label("基準形（R1196）"),
            variant_basic(),
            variant_label("カード形（R1198）"),
            variant_card(),
            variant_label("トップページ形（R1202、大見出し・大きめの余白）"),
            variant_top_page(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/promo-background-image/",
    title: "promo-background-image",
    category: BlockCategory::Promo,
    rust_source: "crates/docs-site/src/blocks/ecommerce/promo/promo_background_image.rs",
    demo_class: "blocks-promo-background-image",
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
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `promo_background_image` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節、他 block と同型）。
///
/// 生の色リテラルは使わず、可読性の確保はすべて `--fandhe-color-*`
/// トークンと `color-mix()` で行う（モジュール冒頭「暗幕は `color-mix` +
/// トークンで作る」節）。
const LAYOUT_CSS: &str = "\
.blocks-promo-background-image-layout {\n  display: grid;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-promo-background-image-basic,\n.blocks-promo-background-image-top-page {\n  position: relative;\n  isolation: isolate;\n  overflow: hidden;\n  display: grid;\n  padding: var(--fandhe-space-16) var(--fandhe-space-6);\n  border-radius: var(--fandhe-radius-lg);\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-promo-background-image-top-page {\n  padding: var(--fandhe-space-24) var(--fandhe-space-6);\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-promo-background-image-card] {\n  position: relative;\n  isolation: isolate;\n  overflow: hidden;\n  padding: 0;\n  border: 0;\n  display: grid;\n  color: var(--fandhe-color-bg);\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-promo-background-image-card] .blocks-promo-background-image-content {\n  padding: var(--fandhe-space-16) var(--fandhe-space-6);\n}\n\
.blocks-promo-background-image-backdrop {\n  position: absolute;\n  inset: 0;\n  z-index: -1;\n  overflow: hidden;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-promo-background-image-image] {\n  width: 100%;\n  height: 100%;\n  display: block;\n}\n\
.blocks-promo-background-image-scrim {\n  position: absolute;\n  inset: 0;\n  background: color-mix(in srgb, var(--fandhe-color-fg) 64%, transparent);\n}\n\
.blocks-promo-background-image-content {\n  display: grid;\n  justify-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-4);\n  max-width: 40rem;\n  margin-inline: auto;\n}\n\
[data-scope=\"heading\"][data-part=\"root\"][data-blocks-promo-background-image-title],\n[data-scope=\"text\"][data-part=\"root\"][data-blocks-promo-background-image-lead] {\n  color: inherit;\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-promo-background-image-cta] {\n  display: inline-block;\n  padding: var(--fandhe-space-3) var(--fandhe-space-6);\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-bg);\n  color: var(--fandhe-color-fg);\n  text-decoration: none;\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-promo-background-image-cta]:hover {\n  background: color-mix(in srgb, var(--fandhe-color-bg) 85%, transparent);\n  color: var(--fandhe-color-fg);\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] が 3 形（基準形/カード形/トップページ形）を静的並記し、
    /// 各 backdrop が `aria-hidden` で装飾扱いになり、禁止パターン
    /// （`<form>`/`<h1`/`<h2`/`href="#"`/`data:` URI/`id=` 等）を含まない
    /// こと（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_renders_three_variants_and_avoids_disallowed_patterns() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"aria-hidden="true""#).count(),
            3,
            "should render exactly 3 backdrops (one per variant)"
        );
        assert_eq!(
            html.matches(r#"alt="""#).count(),
            3,
            "each backdrop image should have an empty alt"
        );
        assert_eq!(
            html.matches("data-blocks-promo-background-image-cta=\"\"")
                .count(),
            3,
            "should render exactly 3 CTA links (one per variant)"
        );
        assert_eq!(
            html.matches(&format!("href=\"{REPO}\"")).count(),
            3,
            "each CTA should link to the real, existing REPO URL (not a dead href=\"#\")"
        );
        assert!(html.contains(r#"data-scope="card""#));
        assert!(html.contains(r#"data-scope="heading""#));
        assert!(html.contains(r#"data-scope="text""#));
        assert!(html.contains(r#"data-scope="link""#));
        assert!(html.contains(r#"data-scope="image""#));
        assert!(html.contains(dummy_assets::BACKGROUND_SRC));
        for hook in [
            "data-blocks-promo-background-image-image",
            "data-blocks-promo-background-image-title",
            "data-blocks-promo-background-image-lead",
            "data-blocks-promo-background-image-cta",
            "data-blocks-promo-background-image-card",
        ] {
            assert!(
                html.contains(hook),
                "demo should render the {hook} attribute"
            );
        }
        for absent in [
            "<form",
            "<h1",
            "<h2",
            "href=\"#\"",
            "src=\"data:",
            "<script",
            " id=\"",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が全主要セレクタを宣言し、色リテラルではなく
    /// トークン参照 + `color-mix()` で暗幕・反転 CTA を作ること。
    #[test]
    fn layout_css_declares_all_selectors_and_uses_tokens_not_literals() {
        for selector in [
            ".blocks-promo-background-image-layout {",
            ".blocks-promo-background-image-basic,\n.blocks-promo-background-image-top-page {",
            ".blocks-promo-background-image-top-page {",
            "[data-scope=\"card\"][data-part=\"root\"][data-blocks-promo-background-image-card] {",
            ".blocks-promo-background-image-backdrop {",
            ".blocks-promo-background-image-scrim {",
            ".blocks-promo-background-image-content {",
            "[data-scope=\"link\"][data-part=\"root\"][data-blocks-promo-background-image-cta] {",
            "[data-scope=\"link\"][data-part=\"root\"][data-blocks-promo-background-image-cta]:hover {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("color-mix("));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("animation"));
        assert!(!LAYOUT_CSS.contains("transition"));
    }
}

//! `promo-countdown` block（イシュー #3080。親 #3024。Ecommerce / Promo
//! カテゴリ 2 件目）。
//!
//! # 出典に関する注記
//!
//! 主参照は中央寄せ（背景画像 + 暗幕）の形（対応表 ID R0636 基準形）。
//! 「背景画像上の左寄せカード」（R0637）・「左に文章、右に大きな数字
//! ボックス」（R0640）を本 block が合成する 2・3 形目へ集約する（対応表
//! ID のみを記す。取得手段・ファイル名は記載しない）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `timer` / `button` / `image` / `card` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は
//! 作らない。
//!
//! # 3 形を 1 つの Demo に並記する
//!
//! `hero_background_media`・`promo_collection_cards` と同型に、
//! [`variant_label`] で見出しを付けながら形 A（中央寄せ、R0636）・
//! 形 B（背景画像上の左寄せカード、R0637）・形 C（左に文章、右に数字
//! ボックス、R0640）を [`demo`] 1 つの中へ縦に並べる。
//!
//! # カウントダウンは idle 状態の固定値（tick しない）
//!
//! docs サイトは無 JS 制約（`crate::blocks` モジュール doc 参照）のため、
//! [`Timer::countdown`] を dispatch せず idle のまま使う。idle の
//! countdown は `display_ms()` が `start_ms.saturating_sub(0)` となり
//! `start_ms` がそのまま表示値になる（`fandhe_frontend_headless_ui::timer`
//! の `display_ms`/`display_segments` 参照）。[`countdown`] ヘルパは
//! `control`/`action_trigger` を出さない静的表示専用の組み立てである。
//!
//! # 暗幕は `color-mix` + トークンで作る（`promo_collection_cards` と同型）
//!
//! 形 A・形 B の背景画像へ `--fandhe-color-fg` を `color-mix()` で半透明化
//! したスクリムを重ね、見出し・リード文・timer の値/ラベルは
//! `--fandhe-color-bg` トークンで描く。色リテラル（`#`/`white` 等）は
//! 使わず、ライトテーマでは「暗幕 + 明るい文字」、ダークテーマでは
//! 前景/背景の意味が反転し「明るい幕 + 暗い文字」になる既知の挙動である
//! （原稿の差分メモに明記、`hero_background_media`/`promo_collection_cards`
//! と同じ判断）。
//!
//! # `drop_class_attr` を踏まえた CSS フックの選び方
//!
//! `heading::heading` / `styled_text::text` / `button::button` /
//! `card::root` / `image::image` / `timer::area`/`item`/`item_value`/
//! `item_label` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、本 block 固有のフックは
//! `data-blocks-promo-countdown-*` 属性で渡す。素の `div` には `class` が
//! そのまま効くため `.blocks-promo-countdown-*` クラスセレクタを使う。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。CTA ボタンは `button::button` の既定 `type="button"` の
//! まま用いる。文言・残り時間はすべて架空のものであり、実企業名・実
//! サービス名・実クレデンシャル・PII を含まない。画像は
//! `dummy_assets::BACKGROUND_SRC`（ビルド時生成のモノトーン背景タイル
//! SVG）を使い回す。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::timer::{self, Timer, TimerUnit};

const IMAGE_ATTR: &str = "data-blocks-promo-countdown-image";
const TITLE_ATTR: &str = "data-blocks-promo-countdown-title";
const LEAD_ATTR: &str = "data-blocks-promo-countdown-lead";
const CTA_ATTR: &str = "data-blocks-promo-countdown-cta";
const CARD_ATTR: &str = "data-blocks-promo-countdown-card";
const VARIANT_ATTR: &str = "data-blocks-promo-countdown-variant";

/// セール終了までの残り時間（架空の固定値、2 日 05 時間 30 分 00 秒）。
/// idle の countdown は `display_ms()` がこの値をそのまま返す（モジュール
/// 冒頭「カウントダウンは idle 状態の固定値」節参照）。
const COUNTDOWN_START_MS: u64 = ((2 * 24 + 5) * 60 + 30) * 60 * 1000;

/// 各形の直前に置く短い形ラベル（`hero_background_media::variant_label` と
/// 同型）。
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

/// 日/時/分/秒 4 単位の idle countdown 表示（`control`/`action_trigger` を
/// 持たない静的表示専用、モジュール冒頭「カウントダウンは idle 状態の
/// 固定値」節参照）。
fn countdown(start_ms: u64, hook: &'static str) -> Node {
    let t = Timer::countdown(start_ms, 1_000);
    let (days, hours, minutes, seconds) = t.display_segments();
    let items = [
        (TimerUnit::Days, days, "日"),
        (TimerUnit::Hours, hours, "時間"),
        (TimerUnit::Minutes, minutes, "分"),
        (TimerUnit::Seconds, seconds, "秒"),
    ]
    .into_iter()
    .map(|(unit, value, label)| {
        timer::item(
            unit,
            vec![],
            vec![
                timer::item_value(unit, vec![], vec![text(timer::format_segment(value))]),
                timer::item_label(unit, vec![], vec![text(label)]),
            ],
        )
    })
    .collect();
    t.root(vec![(hook, "")], vec![t.area(vec![], items)])
}

/// 背景画像 + 暗幕の 2 層（`aria-hidden` で装飾扱い、形 A・B で共通）。
fn backdrop() -> Node {
    div(
        vec![
            ("class", "blocks-promo-countdown-backdrop"),
            ("aria-hidden", "true"),
        ],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                vec![(IMAGE_ATTR, "")],
            ),
            div(vec![("class", "blocks-promo-countdown-scrim")], vec![]),
        ],
    )
}

/// 形 A（R0636 基準形）: 背景画像 + 暗幕 + 中央寄せ。
fn variant_centered() -> Node {
    let content = div(
        vec![("class", "blocks-promo-countdown-content")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![(TITLE_ATTR, "")],
                vec![text("シーズンセール開催中")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![(LEAD_ATTR, "")],
                vec![text("会場限定の割引は終了までの時間限定です。")],
            ),
            countdown(COUNTDOWN_START_MS, "data-blocks-promo-countdown-timer"),
            button::button(
                &ButtonProps::default(),
                vec![(CTA_ATTR, "")],
                vec![text("セール会場へ")],
            ),
        ],
    );
    div(
        vec![
            ("class", "blocks-promo-countdown-root"),
            (VARIANT_ATTR, "centered"),
        ],
        vec![backdrop(), content],
    )
}

/// 形 B（R0637）: 背景画像の上に左寄せのカード。カード内に要素を置く。
fn variant_card() -> Node {
    let card_content = card::body(
        vec![("class", "blocks-promo-countdown-card-body")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![(TITLE_ATTR, "")],
                vec![text("週末限定フラッシュセール")],
            ),
            styled_text::text(
                &TextProps::default(),
                vec![(LEAD_ATTR, "")],
                vec![text("対象アイテムが数量限定で割引になります。")],
            ),
            countdown(COUNTDOWN_START_MS, "data-blocks-promo-countdown-timer-card"),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![(CTA_ATTR, "")],
                vec![text("対象商品を見る")],
            ),
        ],
    );
    div(
        vec![
            ("class", "blocks-promo-countdown-root"),
            (VARIANT_ATTR, "card"),
        ],
        vec![
            backdrop(),
            card::root(
                CardProps::from(CardVariant::Elevated),
                vec![(CARD_ATTR, "")],
                vec![card_content],
            ),
        ],
    )
}

/// 形 C（R0640）: 左に文章、右に大きな数字ボックスの 2 列。md 未満では 1 列。
fn variant_split() -> Node {
    let left = div(
        vec![("class", "blocks-promo-countdown-split-side")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![(TITLE_ATTR, "")],
                vec![text("会員限定クリアランス")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![(LEAD_ATTR, "")],
                vec![text("在庫限りの特別価格は表示の期限までです。")],
            ),
            button::button(
                &ButtonProps::default(),
                vec![(CTA_ATTR, "")],
                vec![text("今すぐ購入")],
            ),
        ],
    );
    let right = div(
        vec![("class", "blocks-promo-countdown-split-timer")],
        vec![countdown(
            COUNTDOWN_START_MS,
            "data-blocks-promo-countdown-timer-split",
        )],
    );
    div(
        vec![
            ("class", "blocks-promo-countdown-root"),
            (VARIANT_ATTR, "split"),
        ],
        vec![left, right],
    )
}

/// `promo-countdown` の Demo 本体。呼び出しごとに同一の `Node` を返す純
/// 関数（`promo_collection_cards::demo` と同型の方針）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-promo-countdown-layout")],
        vec![
            variant_label("中央寄せ（背景画像 + 暗幕）"),
            variant_centered(),
            variant_label("背景画像上の左寄せカード"),
            variant_card(),
            variant_label("左に文・右に数字ボックス"),
            variant_split(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/promo-countdown/",
    title: "promo-countdown",
    category: BlockCategory::Promo,
    rust_source: "crates/docs-site/src/blocks/ecommerce/promo/promo_countdown.rs",
    demo_class: "blocks-promo-countdown",
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
            label: "Timer",
            path: "/themes/timer/",
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
            label: "Card",
            path: "/themes/card/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `promo_countdown` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節、他 block と同型）。
///
/// 生の色リテラルは使わず、可読性の確保はすべて `--fandhe-color-*`
/// トークンと `color-mix()` で行う（モジュール冒頭「暗幕は `color-mix` +
/// トークンで作る」節）。ブレークポイントは
/// [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]（768px =
/// 48rem）と一致するリテラル値を直書きする（既存 block と同じ判断）。
const LAYOUT_CSS: &str = "\
.blocks-promo-countdown-layout {\n  display: grid;\n  gap: 2rem;\n}\n\
.blocks-promo-countdown-root {\n  position: relative;\n  isolation: isolate;\n  overflow: hidden;\n  min-height: 20rem;\n  border-radius: var(--fandhe-radius-lg);\n}\n\
[data-blocks-promo-countdown-variant=\"centered\"],\n[data-blocks-promo-countdown-variant=\"card\"] {\n  display: grid;\n  padding: var(--fandhe-space-16) var(--fandhe-space-6);\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-promo-countdown-backdrop {\n  position: absolute;\n  inset: 0;\n  z-index: -1;\n  overflow: hidden;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-promo-countdown-image] {\n  width: 100%;\n  height: 100%;\n  display: block;\n}\n\
.blocks-promo-countdown-scrim {\n  position: absolute;\n  inset: 0;\n  background: color-mix(in srgb, var(--fandhe-color-fg) 64%, transparent);\n}\n\
.blocks-promo-countdown-content {\n  display: grid;\n  gap: var(--fandhe-space-4);\n  max-width: 40rem;\n  text-align: center;\n  justify-items: center;\n  margin-inline: auto;\n}\n\
[data-blocks-promo-countdown-variant=\"centered\"] [data-scope=\"heading\"][data-part=\"root\"][data-blocks-promo-countdown-title],\n[data-blocks-promo-countdown-variant=\"centered\"] [data-scope=\"text\"][data-part=\"root\"][data-blocks-promo-countdown-lead],\n[data-blocks-promo-countdown-variant=\"centered\"] [data-scope=\"timer\"][data-part=\"item-value\"],\n[data-blocks-promo-countdown-variant=\"centered\"] [data-scope=\"timer\"][data-part=\"item-label\"] {\n  color: inherit;\n}\n\
[data-blocks-promo-countdown-variant=\"card\"] {\n  align-items: center;\n  padding: var(--fandhe-space-10) var(--fandhe-space-6);\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-promo-countdown-card] {\n  max-width: 28rem;\n  display: grid;\n}\n\
.blocks-promo-countdown-card-body {\n  display: grid;\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"timer\"][data-part=\"area\"] {\n  display: flex;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"timer\"][data-part=\"item\"] {\n  display: grid;\n  gap: var(--fandhe-space-1);\n  text-align: center;\n}\n\
[data-blocks-promo-countdown-variant=\"split\"] {\n  display: grid;\n  gap: var(--fandhe-space-8);\n  padding: var(--fandhe-space-10) var(--fandhe-space-6);\n}\n\
.blocks-promo-countdown-split-side {\n  display: grid;\n  gap: var(--fandhe-space-4);\n  align-content: start;\n}\n\
.blocks-promo-countdown-split-timer {\n  display: grid;\n  align-content: start;\n}\n\
[data-blocks-promo-countdown-variant=\"split\"] [data-scope=\"timer\"][data-part=\"item\"] {\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  padding: var(--fandhe-space-4);\n  min-width: 4.5rem;\n}\n\
[data-blocks-promo-countdown-variant=\"split\"] [data-scope=\"timer\"][data-part=\"item-value\"] {\n  --fandhe-timer-value-font-size: var(--fandhe-font-font-size-4xl);\n}\n\
@media (min-width: 48rem) {\n  [data-blocks-promo-countdown-variant=\"split\"] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    align-items: center;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] が 3 形（centered/card/split）を静的並記し、6 部品の
    /// `data-scope` を含み、禁止パターン（`<form>`/`href="#"`/`data:` URI
    /// 等）を含まないこと（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_renders_three_variants_and_avoids_disallowed_patterns() {
        let html = render(&demo());
        assert_eq!(html.matches(VARIANT_ATTR).count(), 3);
        for variant in ["centered", "card", "split"] {
            assert!(html.contains(&format!(
                r#"data-blocks-promo-countdown-variant="{variant}""#
            )));
        }
        for scope in [
            r#"data-scope="heading""#,
            r#"data-scope="text""#,
            r#"data-scope="timer""#,
            r#"data-scope="button""#,
            r#"data-scope="image""#,
            r#"data-scope="card""#,
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches(r#"role="timer""#).count(), 3);
        assert_eq!(html.matches("<button").count(), 3);
        assert_eq!(html.matches(r#"type="button""#).count(), 3);
        assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 2);
        assert!(html.contains(r#"src="../../assets/blocks-demo-background.svg""#));
        for absent in [
            "<form",
            "href=\"#\"",
            "src=\"data:",
            "<script",
            "type=\"submit\"",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
        // 固定値表示の検証: 2 日 05 時間 30 分 00 秒
        for segment in [">02<", ">05<", ">30<", ">00<"] {
            assert!(
                html.contains(segment),
                "demo should render the fixed countdown segment {segment}"
            );
        }
    }

    /// [`LAYOUT_CSS`] が色リテラル（`<`/`#`）を含まず、トークン・
    /// `color-mix()`・ブレークポイントを使うこと。
    #[test]
    fn layout_css_avoids_color_literals_and_uses_tokens() {
        for selector in [
            ".blocks-promo-countdown-layout {",
            ".blocks-promo-countdown-root {",
            ".blocks-promo-countdown-backdrop {",
            ".blocks-promo-countdown-scrim {",
            ".blocks-promo-countdown-content {",
            "[data-scope=\"card\"][data-part=\"root\"][data-blocks-promo-countdown-card] {",
            "@media (min-width: 48rem) {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("color-mix("));
        assert!(LAYOUT_CSS.contains("--fandhe-timer-value-font-size"));
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("animation"));
        assert!(!LAYOUT_CSS.contains("transition"));
    }
}

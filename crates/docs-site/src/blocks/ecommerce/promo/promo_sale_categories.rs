//! `promo-sale-categories` block（イシュー #3083。親 #3024。Ecommerce /
//! Promo カテゴリ）。
//!
//! # 出典に関する注記
//!
//! 主参照は対応表 ID R0639（集約元も R0639 の 1 件のみ、取得手段・
//! ファイル名は記載しない）。
//!
//! # 使用部品
//!
//! `heading` / `timer` / `badge` / `card` / `image` / `link-overlay` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # カウントダウンは固定値（tick 駆動しない）
//!
//! `Timer::countdown` で 2 日 13 時間 45 分 20 秒の残り時間を固定表示する
//! だけの静的掲示であり、`control`（Start/Pause/Reset 等）・
//! `action_trigger` は置かない。無 JS 制約下でボタンを押しても何も起きない
//! 操作子を出さないための判断（`crate::blocks` モジュール doc「`<form>` を
//! 持たない」不変条件と同じ考え方の一般化）。実 tick 駆動（`setInterval`）
//! は `fandhe-frontend-wasm-full::headless_timer` のスコープで、本 block の
//! スコープ外。
//!
//! `area` の既定 `aria-label`（`Timer::area_label`、英語固定書式）は
//! 本サイトの日本語文脈に合わないため、固定の日本語文字列で上書きする
//! （`Timer::area` は呼び出し側 `attrs` に `aria-label` があれば上書きしない
//! dedup 契約を持つ、`timer.rs` doc 参照）。
//!
//! # カウントダウンのピル
//!
//! 見出し行の右側に `badge`（ラベル）+ `timer`（セグメント表示）を並べた
//! 素の `div` をピル状（`border-radius: 9999px` + 枠線 + 淡色背景）にする。
//! ピル自体は装飾用の `div` であり部品ではないため `class` をそのまま使う。
//! `--fandhe-timer-value-font-size` をこのピルのスコープ内でのみ縮小し、
//! ピル内に収まる大きさへ調整する（`timer` 部品側の CSS は変更しない）。
//!
//! # カテゴリカードは全面リンク化（`promo_collection_cards` と同型）
//!
//! `card::root` の内側に `link_overlay::root` を置き、画像 + カテゴリ名を
//! 通常フローで積み、`link_overlay::overlay` でカード全体をクリック可能に
//! する。`card::root` は画像の角丸クリップのため `overflow: hidden` を
//! 持つ（[`LAYOUT_CSS`] 参照）ため、`link_overlay::overlay` 既定の
//! `FocusRingOffset::Outside` がカード境界で切れてしまう。
//! `blog_overlay_cards`/`promo_collection_cards` と同じ是正
//! （`outline-offset` を内側へ上書き）を行う。
//!
//! # グリッドの列数（1 → 48rem で 2 → 64rem で 4）
//!
//! Issue のビューポート基準（md=48rem/lg=64rem）をそのまま
//! [`LAYOUT_CSS`] へリテラル値として直書きする（`fandhe_frontend_pre_styled_ui::recipe::Breakpoint`
//! には md/lg の定数がなく 48rem/64rem が本 block 固有の基準であるため、
//! `promo_collection_cards` の sm=40rem と同じくリテラル直書きとする）。
//!
//! # `drop_class_attr` を踏まえた CSS フックの選び方
//!
//! `heading::heading` / `badge::badge` / `card::root` / `image::image` /
//! `link_overlay::root` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、本 block 固有の
//! フックは `data-blocks-promo-sale-categories-*` 属性で渡す。素の
//! `div`（見出し行・ピル・グリッド）には `class` がそのまま効くため
//! `.blocks-promo-sale-categories-*` クラスセレクタを使う。
//!
//! # リンク先の方針
//!
//! `promo_collection_cards` 等の前例と同じく、外部の絶対 URL
//! `https://github.com/Fandhe-AI/fandhe-frontend` をカードのリンク先として
//! 使う。`href="#"` の死リンクは使わない。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。カテゴリ名・残り時間はすべて架空のものであり、実企業名・
//! 実サービス名・実クレデンシャル・PII を含まない。画像は
//! `dummy_assets::PRODUCT_SRC`（ビルド時生成のモノトーンプレースホルダー
//! SVG）を使い回す。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::timer::{self, Timer, TimerUnit};

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// カウントダウンの固定残り時間（2 日 13 時間 45 分 20 秒、ミリ秒換算）。
/// `tick` を進めず idle 状態のまま [`Timer::display_segments`] で読み出す
/// だけの静的掲示に使う（モジュール doc「カウントダウンは固定値」節）。
const COUNTDOWN_START_MS: u64 = ((2 * 24 + 13) * 60 + 45) * 60 * 1000 + 20 * 1000;

/// カテゴリカード 4 件分の名前（架空、実在の企業・ブランドとは無関係）。
const CATEGORIES: [&str; 4] = ["アウター", "シューズ", "バッグ", "アクセサリー"];

/// 見出し + ピル型カウントダウンの上段。
fn header() -> Node {
    let countdown = Timer::countdown(COUNTDOWN_START_MS, 1_000);
    let (days, hours, minutes, seconds) = countdown.display_segments();
    div(
        vec![("class", "blocks-promo-sale-categories-header")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-promo-sale-categories-title", "")],
                vec![text("期間限定セール")],
            ),
            div(
                vec![("class", "blocks-promo-sale-categories-countdown")],
                vec![
                    badge::badge(
                        &BadgeProps::default(),
                        vec![("data-blocks-promo-sale-categories-badge", "")],
                        vec![text("終了まで")],
                    ),
                    countdown.area(
                        vec![
                            ("aria-label", "セール終了まで 2 日 13 時間 45 分 20 秒"),
                            ("data-blocks-promo-sale-categories-timer", ""),
                        ],
                        vec![
                            timer::item(
                                TimerUnit::Days,
                                vec![],
                                vec![timer::item_value(
                                    TimerUnit::Days,
                                    vec![],
                                    vec![text(timer::format_segment(days))],
                                )],
                            ),
                            timer::separator(vec![], vec![text(":")]),
                            timer::item(
                                TimerUnit::Hours,
                                vec![],
                                vec![timer::item_value(
                                    TimerUnit::Hours,
                                    vec![],
                                    vec![text(timer::format_segment(hours))],
                                )],
                            ),
                            timer::separator(vec![], vec![text(":")]),
                            timer::item(
                                TimerUnit::Minutes,
                                vec![],
                                vec![timer::item_value(
                                    TimerUnit::Minutes,
                                    vec![],
                                    vec![text(timer::format_segment(minutes))],
                                )],
                            ),
                            timer::separator(vec![], vec![text(":")]),
                            timer::item(
                                TimerUnit::Seconds,
                                vec![],
                                vec![timer::item_value(
                                    TimerUnit::Seconds,
                                    vec![],
                                    vec![text(timer::format_segment(seconds))],
                                )],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// カテゴリカード 1 件（画像 + カテゴリ名、カード全体をリンク化）。
fn category_card(name: &'static str) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-promo-sale-categories-card", "")],
        vec![link_overlay::root(
            vec![("data-blocks-promo-sale-categories-link", "")],
            vec![
                image::image(
                    &ImageProps {
                        fit: ImageFit::Cover,
                        ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                    },
                    vec![("data-blocks-promo-sale-categories-image", "")],
                ),
                card::body(
                    vec![("class", "blocks-promo-sale-categories-card-body")],
                    vec![heading::heading(
                        HeadingLevel::H4,
                        &HeadingProps {
                            size: HeadingSize::Md,
                            ..HeadingProps::default()
                        },
                        vec![("data-blocks-promo-sale-categories-card-name", "")],
                        vec![text(name)],
                    )],
                ),
                overlay(
                    REPO,
                    vec![
                        ("aria-label", name),
                        ("data-blocks-promo-sale-categories-overlay", ""),
                    ],
                    vec![],
                ),
            ],
        )],
    )
}

/// `promo-sale-categories` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（カウントダウンは固定値のため非決定性を持たない）。
pub fn demo() -> Node {
    let cards: Vec<Node> = CATEGORIES.iter().map(|name| category_card(name)).collect();
    div(
        vec![("class", "blocks-promo-sale-categories-layout")],
        vec![
            header(),
            div(vec![("class", "blocks-promo-sale-categories-grid")], cards),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/promo-sale-categories/",
    title: "promo-sale-categories",
    category: BlockCategory::Promo,
    rust_source: "crates/docs-site/src/blocks/ecommerce/promo/promo_sale_categories.rs",
    demo_class: "blocks-promo-sale-categories",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Timer",
            path: "/themes/timer/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `promo_sale_categories` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節、他 block と同型）。
///
/// 列数の切り替えは Issue 指定のビューポート基準（md=48rem/lg=64rem）を
/// リテラル直書きする（モジュール冒頭「グリッドの列数」節）。
const LAYOUT_CSS: &str = "\
.blocks-promo-sale-categories-layout {\n  display: grid;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-promo-sale-categories-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-promo-sale-categories-countdown {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: 9999px;\n  background: var(--fandhe-color-bg-subtle);\n  --fandhe-timer-value-font-size: var(--fandhe-font-font-size-md);\n}\n\
.blocks-promo-sale-categories-grid {\n  display: grid;\n  grid-template-columns: repeat(1, minmax(0, 1fr));\n  grid-auto-rows: 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
@media (min-width: 48rem) {\n  .blocks-promo-sale-categories-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 64rem) {\n  .blocks-promo-sale-categories-grid {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-promo-sale-categories-card] {\n  overflow: hidden;\n  padding: 0;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-promo-sale-categories-image] {\n  width: 100%;\n  display: block;\n  border-radius: var(--fandhe-radius-lg) var(--fandhe-radius-lg) 0 0;\n}\n\
.blocks-promo-sale-categories-card-body {\n  padding: var(--fandhe-space-4);\n}\n\
[data-scope=\"link-overlay\"][data-part=\"overlay\"][data-blocks-promo-sale-categories-overlay]:focus-visible {\n  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] がカウントダウン + カテゴリカード 4 枚を構成し、各カードが
    /// `link-overlay` でリンク化され、禁止パターン（`<form>`/`href="#"`/
    /// `data:` URI 等）を含まないこと（`crate::blocks` モジュール doc の
    /// 不変条件）。
    #[test]
    fn demo_renders_countdown_and_four_linked_cards_and_avoids_disallowed_patterns() {
        let html = render(&demo());

        assert!(html.contains(r#"role="timer""#));
        assert!(html.contains("02"), "should render 2 days segment");
        assert!(html.contains("13"), "should render 13 hours segment");
        assert!(html.contains("45"), "should render 45 minutes segment");
        assert!(html.contains("20"), "should render 20 seconds segment");

        for scope in [
            r#"data-scope="heading""#,
            r#"data-scope="badge""#,
            r#"data-scope="card""#,
            r#"data-scope="image""#,
            r#"data-scope="link-overlay""#,
        ] {
            assert!(html.contains(scope), "demo should render {scope}");
        }

        assert_eq!(
            html.matches("data-blocks-promo-sale-categories-card=\"\"")
                .count(),
            4,
            "should render exactly 4 category cards"
        );
        assert_eq!(
            html.matches("data-blocks-promo-sale-categories-link=\"\"")
                .count(),
            4,
            "should render exactly 4 link-overlay roots (one per card)"
        );
        assert!(html.contains(dummy_assets::PRODUCT_SRC));

        for hook in [
            "data-blocks-promo-sale-categories-title",
            "data-blocks-promo-sale-categories-badge",
            "data-blocks-promo-sale-categories-timer",
            "data-blocks-promo-sale-categories-card-name",
            "data-blocks-promo-sale-categories-overlay",
        ] {
            assert!(
                html.contains(hook),
                "demo should render the {hook} attribute"
            );
        }

        for name in CATEGORIES {
            assert!(html.contains(name), "demo should contain category {name}");
        }

        for absent in ["<form", "href=\"#\"", "src=\"data:", "<script", "<button"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が全主要セレクタ・md/lg ブレークポイント・
    /// `color-mix` 非使用（トークンのみで色を表現）を固定すること。
    #[test]
    fn layout_css_declares_all_selectors_and_breakpoints() {
        for selector in [
            ".blocks-promo-sale-categories-layout {",
            ".blocks-promo-sale-categories-header {",
            ".blocks-promo-sale-categories-countdown {",
            ".blocks-promo-sale-categories-grid {",
            "@media (min-width: 48rem) {",
            "@media (min-width: 64rem) {",
            "[data-scope=\"card\"][data-part=\"root\"][data-blocks-promo-sale-categories-card] {",
            "[data-scope=\"image\"][data-part=\"root\"][data-blocks-promo-sale-categories-image] {",
            "[data-scope=\"link-overlay\"][data-part=\"overlay\"][data-blocks-promo-sale-categories-overlay]:focus-visible {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("repeat(2,"));
        assert!(LAYOUT_CSS.contains("repeat(4,"));
        assert!(
            LAYOUT_CSS.contains("outline-offset: calc(-1"),
            "card overflow:hidden がカード内 link-overlay の focus-visible リングを\
             切り取らないよう、promo_collection_cards と同じ内側補正を持つこと"
        );
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("animation"));
        assert!(!LAYOUT_CSS.contains("transition"));
        assert!(!LAYOUT_CSS.contains("</style"));
    }
    /// ルート class が [`demo`] の出力へ実際に現れ、かつ `BLOCK.demo_class`
    /// とは異なること（同一名だと Demo ラッパー側にも同じ class が付き、
    /// [`LAYOUT_CSS`] のルート規則が二重に効く）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("blocks-promo-sale-categories-layout"));
        assert_ne!("blocks-promo-sale-categories-layout", BLOCK.demo_class);
    }
}

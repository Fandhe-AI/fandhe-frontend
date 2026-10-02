# promo-countdown

カウントダウン付きのセール告知。`heading` / `text` / `timer` / `button` /
`image` / `card` の合成例（既存部品のみ、新規部品なし）。中央寄せ（背景画像
+ 暗幕）・背景画像上の左寄せカード・左に文章 + 右に数字ボックスの 3 形を
並記します。カウントダウンは `timer` 部品を idle 状態の固定値で表示し、
docs サイトの無 JS 制約により tick しません。背景は装飾扱いの
`aria-hidden`、暗幕はテーマの色トークンを `color-mix()` で半透明化した
ものです。`<form>` は持たず、ボタンはすべて `type="button"` です。

## Rust コード

```rust
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
```

## 原案差分メモ

- 中央寄せ（背景画像 + 暗幕）を基準形とし、背景画像上の左寄せカード・
  左に文章 + 右に数字ボックスの 2 列の計 3 形へ集約した。
- カウントダウンは `timer` 部品の idle 状態（`Timer::countdown` を
  dispatch しない）で固定値を表示する。docs サイトは無 JS のため実際に
  tick する実装（`setInterval` 駆動）は持たず、見た目の実演にとどめる。
- 全画面高ではなく `min-height` を使い、Demo 枠の中に収まる高さへした。
- 暗幕は色リテラルではなく `--fandhe-color-fg` トークンを `color-mix()`
  で半透明化して作る。ライトテーマでは暗い幕の上に明るい文字が乗るが、
  ダークテーマでは前景/背景の意味が反転するため、明るい幕の上に暗い
  文字が乗る形へ反転する（意図した挙動）。
- 左に文章 + 右に数字ボックスの形では、数字ボックスを枠線 + 角丸 + 大きめの
  フォントサイズで強調し、md（48rem）未満では 1 列へ縦積みする。
- 文言・残り時間はすべて架空のものである。

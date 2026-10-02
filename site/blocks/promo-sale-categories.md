# promo-sale-categories

見出し行の右側にピル型のカウントダウンを置いた上段と、カテゴリカード
4 枚のグリッドを置いた下段で構成するブロックです。グリッドは 1 列から
始まり、`48rem` 以上で 2 列、`64rem` 以上で 4 列になります。各カードは
画像 + カテゴリ名で構成し、カード全体をリンク化します。`heading` /
`timer` / `badge` / `card` / `image` / `link-overlay` の 6 部品を合成し
ます。Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0639 です。

カウントダウンは `timer` 部品を固定値で表示するだけの静的掲示で、
実時間に合わせて進む tick 駆動は行いません。

カテゴリ名・残り時間はすべて架空のデータであり、実在の企業・ブランド・
PII・実クレデンシャルは含みません。画像はビルド時生成の同梱プレース
ホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。送信処理・
状態管理は一切持ちません。

## Rust コード

```rust
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
        vec![("class", "blocks-promo-sale-categories")],
        vec![
            header(),
            div(vec![("class", "blocks-promo-sale-categories-grid")], cards),
        ],
    )
}
```

## 原案差分メモ

集約元は主参照 R0639 の 1 件のみです。

- **R0639（主参照・唯一の集約元）**: 見出し + ピル型カウントダウン +
  カテゴリカードグリッドの構成をそのまま採用しています。集約元が 1 件
  のため並記すべき差分はありません。
- **見出しレベル**: 索引ページの `h1`/`h2` と重複しないよう、Demo 内は
  `h3`（セクション見出し）・`h4`（カード名）にしています。
- **実写画像 → 共通ダミー画像**: カテゴリ画像はビルド時生成の
  `dummy_assets::PRODUCT_SRC`（モノトーンプレースホルダー SVG）に
  差し替えています。
- **死リンク → リポジトリへの固定 URL**: カードのリンク先は
  `https://github.com/Fandhe-AI/fandhe-frontend` の固定外部 URL にして
  います。`href="#"` は使いません。
- **実時間で動くカウントダウン → 固定値の timer**: `timer` 部品は tick
  注入型の決定的な状態機械のため、実時間連動（`setInterval`）は
  `fandhe-frontend-wasm-full::headless_timer` のスコープとし、本 block
  では固定の残り時間（2 日 13 時間 45 分 20 秒）を静的表示するだけに
  留めています。
- **文言・配色・アイコン**: 出典固有の文言・配色・アイコンは持ち込まず、
  架空の文言と `--fandhe-*` トークンに置き換えています。
- **操作ボタンを省略**: 無 JS で押しても何も起きないボタンを出さない
  ため、カウントダウンの Start/Pause/Reset 等の操作子は置いていません。

関連情報: [Heading](../themes/heading.md) / [Timer](../themes/timer.md) /
[Badge](../themes/badge.md) / [Card](../themes/card.md) /
[Image](../themes/image.md) / [Link Overlay](../themes/link-overlay.md)

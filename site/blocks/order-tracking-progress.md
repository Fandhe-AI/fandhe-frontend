# order-tracking-progress

注文ヘッダ（注文番号・注文日・請求書リンク・確認ボタン）、商品カード列
（画像・名称・価格・配送先 + 配送状況の進捗バーと到達段階表示）、サマリ
（請求先・支払い情報・集計）を組み合わせた注文詳細ブロックです。
`heading` / `text` / `image` / `progress` / `steps` / `data-list` /
`button` / `link` / `card` / `separator` / `badge` の 11 部品を合成しま
す。Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

3 商品は到達段階と段階表示の形式を違えて並べています。1・2 件目は 4
段階のラベル列表示（「配送中」「発送準備中」）、3 件目は `steps` による
アイコン付きタイムライン表示（「配達完了」、全段階完了）です。

注文番号・住所・氏名・決済情報はすべて架空のデータであり、実在の
人物・企業・PII・実クレデンシャルは含みません。カード番号は末尾 4 桁の
伏字表現のみで、実在パターンは使いません。商品画像はビルド時生成の
同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。注文処理・
決済・送信先は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::Orientation;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::progress::{self, ProgressProps};
use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::steps;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 配送の 4 段階（表示ラベル）。
const STAGES: [&str; 4] = ["注文受付", "発送準備", "配送中", "配達完了"];

/// 到達段階 index（0〜3）から `Progress`（`value` は 0〜100 の 0〜3 等分）を
/// 導く。段階ラベル列（[`stage_list`]）と進捗バーが同じ `reached` から
/// 導出されるため値の食い違いが構造的に起きない
/// （モジュール冒頭「進捗バーと段階ラベルを同じ値から導く」節参照）。
fn shipment_progress(reached: usize) -> Progress {
    let value = (reached as f64) * 100.0 / ((STAGES.len() - 1) as f64);
    Progress::new(0.0, 100.0, Some(value), Orientation::Horizontal)
}

/// 4 段階の到達ラベル列。`reached` 未満の index は `data-reached` を持つ
/// （狭幅時は [`LAYOUT_CSS`] が 1 カラム＝縦並びへ切り替える）。
fn stage_list(reached: usize) -> Node {
    ul(
        vec![("class", "blocks-order-tracking-progress-stages")],
        STAGES
            .iter()
            .enumerate()
            .map(|(index, label)| {
                let mut attrs = vec![];
                if index <= reached {
                    attrs.push(("data-reached", ""));
                }
                li(attrs, vec![text(*label)])
            })
            .collect(),
    )
}

/// 段階表示の形式（[`product_card`] の `view` 引数）。R0588（`steps` に
/// よるアイコン付きタイムライン版）の差分を、既存のラベル列表示
/// （[`stage_list`]）と並記するために導入した（モジュール冒頭「構成」節
/// 参照）。
enum StageView {
    /// 4 段階を横並びのラベル列で表示する（R1122 主参照、既定）。
    Labels,
    /// `steps` によるアイコン付きタイムラインで表示する（R0588）。狭幅では
    /// 縦向きへ切り替える（[`LAYOUT_CSS`] の `@container` 規則、
    /// [`stage_timeline`] 参照）。
    Timeline,
}

/// 到達段階 index（0〜3）から `Steps` 状態機械を導く（[`stage_timeline`]
/// のみが呼ぶ内部ヘルパ）。最終段階（配達完了）到達時のみ `step` を
/// `count` に写像し、`Steps::is_completed` が真になる（モジュール冒頭
/// 「進捗バーと段階表示を同じ値から導く」節参照）。
fn shipment_steps(reached: usize, orientation: Orientation) -> Steps {
    let step = if reached + 1 >= STAGES.len() {
        STAGES.len()
    } else {
        reached
    };
    Steps::new(STAGES.len(), step, orientation)
}

/// `steps` の indicator 内アイコン。完了段階は自作のチェックマーク SVG、
/// 未完了段階は段階番号のテキスト（モジュール冒頭「アイコン」節参照）。
fn stage_icon(complete: bool, index: usize) -> Node {
    if complete {
        el(
            "svg",
            vec![
                ("viewBox", "0 0 24 24"),
                ("width", "16"),
                ("height", "16"),
                ("aria-hidden", "true"),
            ],
            vec![el(
                "path",
                vec![
                    ("d", "M5 12l4 4L19 7"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                ],
                vec![],
            )],
        )
    } else {
        text((index + 1).to_string())
    }
}

/// 4 段階の `steps` タイムライン 1 インスタンス（[`Orientation`] を構築時
/// 固定で受け取る、モジュール冒頭「`steps` タイムラインの縦横切替」節
/// 参照）。呼び出し側（[`product_card`]）が横向き・縦向きの 2 インスタンス
/// を常に両方描画し、`@container` で表示を切り替える。
fn stage_timeline(reached: usize, orientation: Orientation) -> Node {
    let state = shipment_steps(reached, orientation);
    let items = STAGES
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let complete = index < state.step();
            let trigger = steps::trigger(
                &state,
                index,
                vec![],
                vec![
                    steps::indicator(&state, index, vec![], vec![stage_icon(complete, index)]),
                    text(*label),
                ],
            );
            let mut item_children = vec![trigger];
            if index + 1 < STAGES.len() {
                item_children.push(steps::separator(&state, index, vec![], vec![]));
            }
            steps::item(&state, index, vec![], item_children)
        })
        .collect();
    let list = steps::list(&state, vec![], items);
    steps::root(
        Size::Sm,
        ColorPalette::Accent,
        &state,
        vec![("data-blocks-order-tracking-progress-timeline", "")],
        vec![list],
    )
}

/// 注文番号・注文日の見出しと、請求書リンク・確認ボタンの操作群を束ねる
/// ヘッダ行。
fn order_header(order_number: &'static str, order_date: &'static str) -> Node {
    div(
        vec![("class", "blocks-order-tracking-progress-header")],
        vec![
            div(
                vec![("class", "blocks-order-tracking-progress-identity")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(format!("注文 #{order_number}"))],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(format!("注文日 {order_date}"))],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-order-tracking-progress-actions")],
                vec![
                    link::root(
                        "https://example.com/invoices/fd-2048-113",
                        &LinkProps::default(),
                        vec![],
                        vec![text("請求書を表示")],
                    ),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("注文内容を確認")],
                    ),
                ],
            ),
        ],
    )
}

/// 商品カード 1 件（画像・名称・価格・配送先 + 配送状況・進捗バー・
/// 段階ラベル列）。
fn product_card(
    name: &'static str,
    price: &'static str,
    destination: &'static str,
    status_label: &'static str,
    eta: &'static str,
    reached: usize,
    view: StageView,
) -> Node {
    let progress_state = shipment_progress(reached);
    let progress_aria_label = format!("{name} の配送の進捗");
    card::root(
        CardProps::default(),
        vec![("data-blocks-order-tracking-progress-card", "")],
        vec![
            card::body(
                vec![("class", "blocks-order-tracking-progress-product")],
                vec![
                    image(
                        &ImageProps {
                            fit: ImageFit::Cover,
                            shape: ImageShape::Rounded,
                            ..ImageProps::new(dummy_assets::PRODUCT_SRC, name)
                        },
                        vec![("data-blocks-order-tracking-progress-image", "")],
                    ),
                    div(
                        vec![("class", "blocks-order-tracking-progress-product-info")],
                        vec![
                            heading(
                                HeadingLevel::H4,
                                &HeadingProps::default(),
                                vec![],
                                vec![text(name)],
                            ),
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(price)],
                            ),
                            styled_text::text(
                                &TextProps {
                                    variant: TextVariant::Muted,
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(format!("配送先: {destination}"))],
                            ),
                        ],
                    ),
                ],
            ),
            card::footer(
                vec![("class", "blocks-order-tracking-progress-shipping")],
                vec![
                    div(
                        vec![("class", "blocks-order-tracking-progress-status")],
                        vec![
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(format!("配送状況: {status_label}"))],
                            ),
                            badge(
                                &BadgeProps {
                                    variant: BadgeVariant::Subtle,
                                    ..BadgeProps::default()
                                },
                                vec![],
                                vec![text(eta)],
                            ),
                        ],
                    ),
                    progress::root(
                        &progress_state,
                        &ProgressProps::default(),
                        Some(status_label),
                        vec![
                            ("data-blocks-order-tracking-progress-bar", ""),
                            ("aria-label", progress_aria_label.as_str()),
                        ],
                        vec![progress_state
                            .track(vec![], vec![progress::range(&progress_state, vec![])])],
                    ),
                    match view {
                        StageView::Labels => stage_list(reached),
                        StageView::Timeline => div(
                            vec![("class", "blocks-order-tracking-progress-timeline-group")],
                            vec![
                                div(
                                    vec![(
                                        "class",
                                        "blocks-order-tracking-progress-timeline-horizontal",
                                    )],
                                    vec![stage_timeline(reached, Orientation::Horizontal)],
                                ),
                                div(
                                    vec![(
                                        "class",
                                        "blocks-order-tracking-progress-timeline-vertical",
                                    )],
                                    vec![stage_timeline(reached, Orientation::Vertical)],
                                ),
                            ],
                        ),
                    },
                ],
            ),
        ],
    )
}

/// ラベル・値の 1 行（`data_list::item` + `item-label` + `item-value`）。
fn row(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 見出し + 定義リストの 1 セクション（請求先・支払い情報・集計）。
fn summary_section(title: &'static str, orientation: DataListOrientation, rows: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-order-tracking-progress-summary-section")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(title)],
            ),
            data_list::root(
                DataListProps {
                    orientation,
                    ..DataListProps::default()
                },
                vec![],
                rows,
            ),
        ],
    )
}

/// `order-tracking-progress` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-order-tracking-progress-layout")],
        vec![
            order_header("FD-2048-113", "2026-09-24"),
            separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-order-tracking-progress-items")],
                vec![
                    product_card(
                        dummy_assets::COMPANY_NAMES[0],
                        "¥12,800",
                        "東京都渋谷区 1-2-3",
                        "配送中",
                        "9/27 到着予定",
                        2,
                        StageView::Labels,
                    ),
                    product_card(
                        dummy_assets::COMPANY_NAMES[1],
                        "¥5,600",
                        "大阪府大阪市 4-5-6",
                        "発送準備中",
                        "9/29 到着予定",
                        1,
                        StageView::Labels,
                    ),
                    product_card(
                        dummy_assets::COMPANY_NAMES[2],
                        "¥3,200",
                        "福岡県福岡市 7-8-9",
                        "配達完了",
                        "9/24 到着済み",
                        3,
                        StageView::Timeline,
                    ),
                ],
            ),
            separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-order-tracking-progress-summary")],
                vec![
                    summary_section(
                        "請求先",
                        DataListOrientation::Vertical,
                        vec![
                            row("宛名", dummy_assets::PERSON_NAMES[0]),
                            row("住所", "東京都渋谷区 1-2-3"),
                            row("メール", "haruto.fujimaki@example.com"),
                        ],
                    ),
                    summary_section(
                        "支払い情報",
                        DataListOrientation::Vertical,
                        vec![
                            row("支払方法", "クレジットカード"),
                            row("カード番号", "**** **** **** 4242"),
                            row("請求日", "2026-09-24"),
                        ],
                    ),
                    summary_section(
                        "集計",
                        DataListOrientation::Horizontal,
                        vec![
                            row("小計", "¥21,600"),
                            row("送料", "¥600"),
                            row("税", "¥2,200"),
                            row("合計", "¥24,400"),
                        ],
                    ),
                ],
            ),
        ],
    )
}
```

## 差分メモ

集約元 4 件（主参照 R1122・R1123・R0585・R0588）に対する本 block の扱い
です。

- **R1122（主参照、代表構成）**: 注文ヘッダ・商品カード列・サマリの
  3 領域構成をそのまま採用しています。
- **R1123（大画像・枠なし版）**: 本 block は `card` の枠付き・`8rem` 幅の
  画像列を既定表現として採用し、枠なし・大画像の別版は Demo に並べてい
  ません。1 block 内に枠あり/なしが混在すると差分の主眼が読み取れなく
  なるためです。
- **R0585（商品ごとに配送先と進捗を表示）**: 3 商品それぞれが
  「配送先: {destination}」行と自身の進捗バー/段階表示を個別に持つ構成
  で、すでに満たしています。
- **R0588（`steps` によるアイコン付きタイムライン表示）**: 3 件目の商品
  カードで採用しています。`steps` の `orientation` は構築時固定のため、
  横向き・縦向きの 2 インスタンスを常に描画し、コンテナ幅 40rem 未満で
  横向きを隠して縦向きを表示する `@container` 規則で切り替えています。
- **状態違いの並記**: 1〜3 件目でそれぞれ「配送中」「発送準備中」
  「配達完了」（全段階完了）の異なる到達段階を並べています。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Image](../themes/image.md) / [Progress](../themes/progress.md) /
[Steps](../themes/steps.md) / [Data List](../themes/data-list.md) /
[Button](../themes/button.md) / [Link](../themes/link.md) /
[Card](../themes/card.md) / [Separator](../themes/separator.md) /
[Badge](../themes/badge.md)

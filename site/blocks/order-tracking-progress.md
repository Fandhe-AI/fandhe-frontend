# order-tracking-progress

注文ヘッダ（注文番号・注文日・請求書リンク・確認ボタン）、商品カード列
（画像・名称・価格・配送先 + 配送状況の進捗バーと 4 段階の到達ラベル）、
サマリ（請求先・支払い情報・集計）を組み合わせた注文詳細ブロックです。
`heading` / `text` / `image` / `progress` / `data-list` / `button` /
`link` / `card` / `separator` / `badge` の 10 部品を合成します。Blocks は
既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R1122（代表構成）です。集約元 R1123（大画像・枠なし
版）・R0585（商品ごと配送先強調）・R0588（`steps` によるアイコン付き
タイムライン版）の差分並記は後続イシューで扱います。

注文番号・住所・氏名・決済情報はすべて架空のデータであり、実在の
人物・企業・PII・実クレデンシャルは含みません。カード番号は末尾 4 桁の
伏字表現のみで、実在パターンは使いません。商品画像はビルド時生成の
同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。注文処理・
決済・送信先は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::Orientation;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::progress::{self, ProgressProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
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
) -> Node {
    let progress_state = shipment_progress(reached);
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
                            ("aria-label", "配送の進捗"),
                        ],
                        vec![progress_state
                            .track(vec![], vec![progress::range(&progress_state, vec![])])],
                    ),
                    stage_list(reached),
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
                    ),
                    product_card(
                        dummy_assets::COMPANY_NAMES[1],
                        "¥5,600",
                        "大阪府大阪市 4-5-6",
                        "発送準備中",
                        "9/29 到着予定",
                        1,
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
                            row("小計", "¥18,400"),
                            row("送料", "¥600"),
                            row("税", "¥1,900"),
                            row("合計", "¥20,900"),
                        ],
                    ),
                ],
            ),
        ],
    )
}
```

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Image](../themes/image.md) / [Progress](../themes/progress.md) /
[Data List](../themes/data-list.md) / [Button](../themes/button.md) /
[Link](../themes/link.md) / [Card](../themes/card.md) /
[Separator](../themes/separator.md) / [Badge](../themes/badge.md)

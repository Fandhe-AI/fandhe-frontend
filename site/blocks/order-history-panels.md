# order-history-panels

注文ごとに枠付きパネルを縦に積む注文履歴ブロックです。主参照 R1116（サマリ
帯 + 商品行の代表構成）を軸にしています。各パネル上部はサマリ帯（注文番
号・注文日・合計 + 「注文を見る」「請求書を見る」ボタン・三点メニュー）、
下部は商品行（画像・名称・価格・説明 + 「商品を見る」リンク）です。
`card` / `data-list` / `button` / `menu` / `image` / `text` / `heading` /
`link` / `separator` / `icon` の 10 部品を合成します。Blocks は既存部品の
合成例であり、新しい UI 部品は追加しません。

コンテナ幅 40rem 未満では「注文を見る」「請求書を見る」ボタンと注文日の
行を隠し、同じ操作を提供する三点メニューのみを見せます。40rem 以上では
逆にボタン・注文日を見せてメニューを隠します。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。注文処理・
決済・送信先は一切持たず、ボタン・三点メニューはいずれも `disabled` で
固定しています（押しても何も起きない要素を操作可能に見せないため）。

注文番号・日付・商品名・価格はすべて架空のデータであり、実在の人物・
企業・商品とは無関係です。商品画像はビルド時生成の同梱プレースホルダー
SVG です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::menu;
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 商品 1 件分のダミーデータ。
struct Item {
    name: &'static str,
    description: &'static str,
    price: &'static str,
    href: &'static str,
}

/// 注文 1 件分のダミーデータ。[`menu_id`]/[`trigger_id`] は
/// `menu::content`/`menu::trigger` を紐づける注文固有の一意 id（モジュール
/// 冒頭「`menu` の id をページ内で一意にする理由」節参照）。
struct Order {
    number: &'static str,
    date: &'static str,
    total: &'static str,
    menu_id: &'static str,
    trigger_id: &'static str,
    items: &'static [Item],
}

const ORDERS: [Order; 2] = [
    Order {
        number: "FD-3102-220",
        date: "2026-09-18",
        total: "¥9,600",
        menu_id: "blocks-order-history-panels-menu-0",
        trigger_id: "blocks-order-history-panels-menu-trigger-0",
        items: &[
            Item {
                name: "ノイズキャンセリングイヤホン",
                description: "ブラック / Bluetooth 5.3",
                price: "¥8,200",
                href: "https://example.com/products/noise-cancelling-earbuds",
            },
            Item {
                name: "USB-C 急速充電ケーブル 1m",
                description: "グレー",
                price: "¥1,400",
                href: "https://example.com/products/usb-c-cable-1m",
            },
        ],
    },
    Order {
        number: "FD-3102-231",
        date: "2026-09-25",
        total: "¥17,000",
        menu_id: "blocks-order-history-panels-menu-1",
        trigger_id: "blocks-order-history-panels-menu-trigger-1",
        items: &[
            Item {
                name: "メカニカルキーボード",
                description: "茶軸 / 日本語配列",
                price: "¥12,800",
                href: "https://example.com/products/mechanical-keyboard",
            },
            Item {
                name: "静音マウス",
                description: "ホワイト",
                price: "¥3,200",
                href: "https://example.com/products/silent-mouse",
            },
            Item {
                name: "リストレスト",
                description: "低反発ウレタン",
                price: "¥1,000",
                href: "https://example.com/products/wrist-rest",
            },
        ],
    },
];

/// サマリ帯の定義リスト 1 行（ラベル + 値）。`extra_attrs` は注文日の行にだけ
/// 狭幅非表示フック（`data-blocks-order-history-panels-date`）を渡すために使う。
fn summary_row(
    label: &'static str,
    value: &'static str,
    extra_attrs: Vec<(&'static str, &'static str)>,
) -> Node {
    data_list::item(
        extra_attrs,
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 三点メニューの trigger に使う自作の線画アイコン（装飾用途、`aria-hidden`。
/// 参照元の絵柄アイコンは持ち込まない）。
fn more_actions_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![
            el(
                "circle",
                vec![("cx", "12"), ("cy", "5"), ("r", "1.6")],
                vec![],
            ),
            el(
                "circle",
                vec![("cx", "12"), ("cy", "12"), ("r", "1.6")],
                vec![],
            ),
            el(
                "circle",
                vec![("cx", "12"), ("cy", "19"), ("r", "1.6")],
                vec![],
            ),
        ],
    )
}

/// サマリ帯の三点メニュー（「注文を見る」「請求書を見る」。狭幅で隠れる
/// ボタン 2 個と同じ操作を提供する、モジュール冒頭「構成」節参照）。
fn action_menu(order: &Order) -> Node {
    let aria_label = format!("その他の操作、注文 {}", order.number);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![
            menu::trigger(
                OpenState::Closed,
                true,
                Some(order.menu_id),
                vec![
                    ("id", order.trigger_id),
                    ("aria-label", aria_label.as_str()),
                ],
                vec![more_actions_icon()],
            ),
            menu::positioner(
                OpenState::Closed,
                vec![],
                vec![menu::content(
                    OpenState::Closed,
                    Some(order.menu_id),
                    Some(order.trigger_id),
                    vec![],
                    vec![
                        menu::item("view", false, false, vec![], vec![text("注文を見る")]),
                        menu::separator(vec![], vec![]),
                        menu::item("invoice", false, false, vec![], vec![text("請求書を見る")]),
                    ],
                )],
            ),
        ],
    )
}

/// パネル上部のサマリ帯（`card::header`。定義リスト + 操作群）。
fn summary(order: &Order) -> Node {
    card::header(
        vec![("class", "blocks-order-history-panels-summary")],
        vec![
            data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Vertical,
                    ..DataListProps::default()
                },
                vec![("data-blocks-order-history-panels-summary-list", "")],
                vec![
                    summary_row("注文番号", order.number, vec![]),
                    summary_row(
                        "注文日",
                        order.date,
                        vec![("data-blocks-order-history-panels-date", "")],
                    ),
                    summary_row("合計", order.total, vec![]),
                ],
            ),
            div(
                vec![("class", "blocks-order-history-panels-actions")],
                vec![
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            size: Size::Sm,
                            disabled: true,
                            ..ButtonProps::default()
                        },
                        vec![("data-blocks-order-history-panels-action-button", "")],
                        vec![text("注文を見る")],
                    ),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            size: Size::Sm,
                            disabled: true,
                            ..ButtonProps::default()
                        },
                        vec![("data-blocks-order-history-panels-action-button", "")],
                        vec![text("請求書を見る")],
                    ),
                    action_menu(order),
                ],
            ),
        ],
    )
}

/// 商品 1 行分（画像・名称・価格・説明 + 「商品を見る」リンク）。
fn item_row(item: &Item) -> Node {
    div(
        vec![("class", "blocks-order-history-panels-item")],
        vec![
            image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, item.name)
                },
                vec![("data-blocks-order-history-panels-image", "")],
            ),
            div(
                vec![("class", "blocks-order-history-panels-item-body")],
                vec![
                    div(
                        vec![("class", "blocks-order-history-panels-item-heading")],
                        vec![
                            heading(
                                HeadingLevel::H4,
                                &HeadingProps::default(),
                                vec![],
                                vec![text(item.name)],
                            ),
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(item.price)],
                            ),
                        ],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(item.description)],
                    ),
                ],
            ),
            link::root(
                item.href,
                &LinkProps::default(),
                vec![("data-blocks-order-history-panels-item-link", "")],
                vec![text("商品を見る")],
            ),
        ],
    )
}

/// 注文 1 件分のパネル（`card::root`。サマリ帯 + 区切り線 + 商品行の並び）。
fn panel(order: &Order) -> Node {
    let mut body_children: Vec<Node> = Vec::new();
    for (index, item) in order.items.iter().enumerate() {
        if index > 0 {
            body_children.push(separator(&SeparatorProps::default(), vec![]));
        }
        body_children.push(item_row(item));
    }
    card::root(
        CardProps::default(),
        vec![("data-blocks-order-history-panels-panel", "")],
        vec![
            summary(order),
            separator(&SeparatorProps::default(), vec![]),
            card::body(
                vec![("class", "blocks-order-history-panels-items")],
                body_children,
            ),
        ],
    )
}

/// `order-history-panels` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-order-history-panels-layout")],
        ORDERS.iter().map(panel).collect(),
    )
}
```

## 差分メモ

- **R1116（主参照、狭幅時のメニュー化）**: サマリ帯のボタン 2 個と注文日
  表示を、コンテナ幅 40rem 未満で三点メニューへ集約する差分をそのまま
  採用しています。
- 配送状況の状態表示（R1118）・注文見出しの形式と商品行の「再購入」
  「類似品を見る」ボタン（R1119）は後続イシュー #3057 で追加予定です。

関連情報: [Card](../themes/card.md) / [Data List](../themes/data-list.md) /
[Button](../themes/button.md) / [Menu](../themes/menu.md) /
[Image](../themes/image.md) / [Text](../themes/text.md) /
[Heading](../themes/heading.md) / [Link](../themes/link.md) /
[Separator](../themes/separator.md) / [Icon](../themes/icon.md)

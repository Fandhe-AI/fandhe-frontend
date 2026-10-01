# cart-dialog

カートを開くボタンと、画面中央に出るダイアログ（タイトル・閉じるボタン・
数量選択/削除付き商品行・淡い面の集計・右寄せの次へボタン）で構成する
カートダイアログのブロックです。`dialog` / `button` / `image` / `text` /
`native-select` / `separator` / `data-list` の 7 部品を合成します。Blocks は
既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R1251（集約元 R0680）です。商品名・属性・価格は
すべて架空のデータであり、実在のブランド・商品・PII は含みません。
商品画像はビルド時生成の同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。ダイアログは
常に開いた状態で表示します。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole, OpenState};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::native_select::{
    native_select, FieldIds, FieldProps, NativeSelectProps,
};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// [`dialog::content`] の `id`（[`dialog::trigger`] の `controls` と対）。
const CONTENT_ID: &str = "blocks-cart-dialog-content";

/// [`dialog::title`] の `id`（[`dialog::content`] の `labelledby` と対）。
const TITLE_ID: &str = "blocks-cart-dialog-title";

/// 架空の商品行データ（商品名, 属性表示, 価格表示, 初期選択数量）。
/// 実在のブランド・商品・PII は含まない。
const CART_ITEMS: &[(&str, &str, &str, u8)] = &[
    (
        "エルゴノミック メッシュチェア",
        "カラー: グレー / サイズ: M",
        "¥24,800",
        1,
    ),
    (
        "ノイズキャンセリング ヘッドホン",
        "カラー: ブラック",
        "¥18,200",
        1,
    ),
];

/// 集計行（ラベル, 値）。最終行（合計）だけ [`summary`] 側で強調用の
/// `data-*` を追加する。
const SUMMARY_ROWS: &[(&str, &str)] = &[("小計", "¥43,000"), ("送料", "¥600"), ("合計", "¥43,600")];

/// 指定した初期選択数量 `selected` の 1〜5 の `<option>` 列を組み立てる
/// （`cart_two_column_summary.rs::qty_options` と同型）。
fn qty_options(selected: u8) -> Vec<Node> {
    (1..=5u8)
        .map(|n| {
            let mut attrs = vec![("value", n.to_string())];
            if n == selected {
                attrs.push(("selected", String::new()));
            }
            el(
                "option",
                attrs.iter().map(|(k, v)| (*k, v.as_str())).collect(),
                vec![text(n.to_string())],
            )
        })
        .collect()
}

/// 商品行 1 件（サムネイル + 名称/属性 + 価格/数量選択/削除）。`index` は
/// 0 始まりで、数量 `select` の一意な `id` の派生に使う
/// （`cart_two_column_summary.rs::item_row` と同型）。
fn item_row(index: usize, name: &str, attrs: &str, price: &str, qty: u8) -> Node {
    let field_id = format!("blocks-cart-dialog-qty-{}", index + 1);
    let qty_aria_label = format!("{name} の数量");
    let field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    div(
        vec![("class", "blocks-cart-dialog-item")],
        vec![
            image(
                &ImageProps {
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-cart-dialog-thumb", "")],
            ),
            div(
                vec![("class", "blocks-cart-dialog-item-body")],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Medium,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(attrs)],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-cart-dialog-item-controls")],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Medium,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(price)],
                    ),
                    native_select(
                        &NativeSelectProps::default(),
                        &field,
                        vec![("aria-label", qty_aria_label.as_str())],
                        qty_options(qty),
                    ),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("削除")],
                    ),
                ],
            ),
        ],
    )
}

/// 商品行と行間の `separator` を束ねたリスト。
fn item_list() -> Node {
    let mut children = Vec::new();
    for (index, (name, attrs, price, qty)) in CART_ITEMS.iter().enumerate() {
        if index > 0 {
            children.push(separator(&SeparatorProps::default(), vec![]));
        }
        children.push(item_row(index, name, attrs, price, *qty));
    }
    div(vec![("class", "blocks-cart-dialog-items")], children)
}

/// 集計行 1 件（`data_list::item` + `item-label` + `item-value`）。最終行
/// （合計）だけ強調用の `data-blocks-cart-dialog-total` を付与する
/// （`cart_two_column_summary.rs::summary_row` と同型）。
fn summary_row(label: &str, value: &str, emphasize: bool) -> Node {
    let attrs = if emphasize {
        vec![("data-blocks-cart-dialog-total", "")]
    } else {
        vec![]
    };
    data_list::item(
        attrs,
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 淡い背景面の集計（小計・送料・合計）。
fn summary() -> Node {
    let last = SUMMARY_ROWS.len() - 1;
    let rows = SUMMARY_ROWS
        .iter()
        .enumerate()
        .map(|(i, (label, value))| summary_row(label, value, i == last))
        .collect();
    div(
        vec![("data-blocks-cart-dialog-summary", "")],
        vec![data_list::root(
            DataListProps {
                orientation: DataListOrientation::Horizontal,
                ..DataListProps::default()
            },
            vec![],
            rows,
        )],
    )
}

/// `cart-dialog` の Demo 本体。カートを開くボタン（右寄せ）+ 中央に開いた
/// 静的なダイアログで構成する。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cart-dialog-stack")],
        vec![
            div(
                vec![("class", "blocks-cart-dialog-bar")],
                vec![dialog::trigger(
                    OpenState::Open,
                    Some(CONTENT_ID),
                    vec![
                        ("disabled", ""),
                        ("data-disabled", ""),
                        ("data-blocks-cart-dialog-open", ""),
                    ],
                    vec![text("カート（2）")],
                )],
            ),
            dialog::root(
                Size::Lg,
                OpenState::Open,
                vec![("data-blocks-cart-dialog-root", "")],
                vec![
                    dialog::backdrop(OpenState::Open, vec![], vec![]),
                    dialog::positioner(
                        OpenState::Open,
                        vec![],
                        vec![dialog::content(
                            OpenState::Open,
                            DialogRole::Dialog,
                            false,
                            ContentIds {
                                id: Some(CONTENT_ID),
                                labelledby: Some(TITLE_ID),
                                describedby: None,
                            },
                            vec![],
                            vec![
                                dialog::close_trigger(
                                    vec![
                                        ("aria-label", "閉じる"),
                                        ("disabled", ""),
                                        ("data-disabled", ""),
                                    ],
                                    vec![text("×")],
                                ),
                                dialog::title(
                                    Some(TITLE_ID),
                                    vec![],
                                    vec![text("ショッピングカート")],
                                ),
                                dialog::body(
                                    vec![("data-blocks-cart-dialog-body", "")],
                                    vec![item_list()],
                                ),
                                summary(),
                                dialog::footer(
                                    vec![],
                                    vec![button(
                                        &ButtonProps::default(),
                                        vec![("data-blocks-cart-dialog-next", "")],
                                        vec![text("レジに進む")],
                                    )],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}
```

## 原案差分メモ

- 開くボタン・閉じるボタンは docs サイトが JS ハイドレーションを行わない
  設計のため無 JS 下では開閉を切り替えられません。「開くボタン」「閉じる
  ボタン」を明示する issue のレイアウト仕様に合わせて両方を配置したうえで、
  ネイティブ `disabled` 属性 + `data-disabled` でフォーカス・クリック不能を
  明示しています（`store-nav-centered-logo` と同型の判断）。
- 静的なデモは閉じる機構を実際には持たず、ダイアログの外側に説明・コード・
  ナビゲーションがあるため、表示の実態と一致させて `aria-modal` は false に
  しています（`contact-dialog-form` と同じ判断）。
- 主参照（対応表 ID R1251）の中央モーダル + 集計 + 次へボタンという構成を
  主体とし、集約元（R0680、中央ダイアログ型カート）の商品行構成（画像・
  名称/属性・価格/数量選択/削除）を統合しています。R0680 単体にはない
  淡い背景面の集計（小計・送料・合計）と右寄せの次へボタンを主参照側から
  採用した点が差分です。
- コンテナ幅（Demo 枠の幅、ビューポート幅ではありません）が 36rem 未満に
  なると `@container` によってダイアログの横幅が全幅に近づき、商品行の
  コントロール・次へボタンが縦積み・全幅になります。

関連情報: [Dialog](../themes/dialog.md) / [Button](../themes/button.md) /
[Image](../themes/image.md) / [Text](../themes/text.md) /
[Native Select](../themes/native-select.md) /
[Separator](../themes/separator.md) / [Data List](../themes/data-list.md)

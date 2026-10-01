# checkout-step-sections

購入手続きを「現在入力中の節 + まだ着手していない後続節」として段階的に
見せる合成例です。一方の列に注文サマリ（商品行・編集/削除操作・集計）、
もう一方の列に簡易決済ボタン群 → 現在節（連絡先） → 後続節（配送先・
配送方法・支払い・最終確認）の見出し一覧を並べます。`field` / `input` /
`checkbox` / `button` / `heading` / `image` / `separator` / `data-list` /
`text` の 9 部品を合成します。Blocks は既存部品の合成例であり、新しい
UI 部品は追加しません。

本 Demo は無 JS の静的表示のみです。`<form>` を含まず、送信処理・データ
取得を一切行いません。現在節だけが入力可能で、後続節は見出しと「未着手」
テキストのみを持ち、入力コントロールを一切持たない構造で表しています。
編集・削除ボタン・お知らせ受信 checkbox はネイティブ disabled で固定し、
選択状態が変化しないことを構造的に保証しています。簡易決済ボタンは
架空の汎用名称のみを使い、実在ブランド名・ロゴは使いません。カード番号・
CVC 等の決済情報入力欄は置いていません（実在の決済フォームに見せない
ための判断です）。文言・金額はすべて架空のものです。

主参照は対応表 ID R0835 です（対応表 ID のみを記載し、出典の固有名は
記載しません）。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

const EMAIL_ID: &str = "blocks-checkout-step-sections-email";

/// 後続節の節名一覧（配送先 → 配送方法 → 支払い → 最終確認の順）。
const UPCOMING_STEPS: &[&str] = &["配送先", "配送方法", "支払い", "最終確認"];

/// 商品行 3 件（名称・属性・価格、`checkout_form_summary_split::
/// product_row` と同型）。
const PRODUCTS: &[(&str, &str, &str)] = &[
    ("キャンバストートバッグ", "カラー: ナチュラル", "¥6,400"),
    ("セラミックマグカップ", "カラー: ホワイト", "¥3,200"),
    ("コットンソックス 2 足組", "サイズ: M", "¥3,200"),
];

/// 呼び出しごとに [`FieldProps`] を組み立てる小さなヘルパ。
fn field_props(id: &'static str, required: bool) -> FieldProps<'static> {
    FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required,
        readonly: false,
        has_helper_text: false,
    }
}

/// 見出し 1 件（`## Demo` がページ側で `h3` を出すため `h3` に固定）。
fn section_heading(title: &'static str, attrs: Vec<(&'static str, &'static str)>) -> Node {
    heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        attrs,
        vec![text(title)],
    )
}

/// 商品行 1 件（画像 + 名称・属性 + 編集/削除ボタン + 価格）。編集/削除は
/// 押しても状態が変わらず不整合になるため disabled 固定し、商品数ぶん
/// `aria-label` を一意にする（モジュール doc「編集・削除ボタンを disabled
/// にする理由」節）。
fn product_row(name: &'static str, variant_label: &'static str, price: &'static str) -> Node {
    let edit_label = format!("{name} を編集");
    let delete_label = format!("{name} を削除");
    li(
        vec![("class", "blocks-checkout-step-sections-product-row")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    aspect_ratio: AspectRatio::Square,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-checkout-step-sections-product-image", "")],
            ),
            div(
                vec![("class", "blocks-checkout-step-sections-product-detail")],
                vec![
                    styled_text::text(&TextProps::default(), vec![], vec![text(name)]),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(variant_label)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-checkout-step-sections-product-price", "")],
                        vec![text(price)],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-checkout-step-sections-product-actions")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            size: Size::Sm,
                            disabled: true,
                            ..ButtonProps::default()
                        },
                        vec![("aria-label", edit_label.as_str())],
                        vec![text("編集")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            size: Size::Sm,
                            disabled: true,
                            ..ButtonProps::default()
                        },
                        vec![("aria-label", delete_label.as_str())],
                        vec![text("削除")],
                    ),
                ],
            ),
        ],
    )
}

/// ラベル・値の 1 行。
fn total_row(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 一方の列（注文サマリ）全体。商品行 → 区切り → 集計の順に縦積みする。
fn summary_column() -> Node {
    div(
        vec![("data-blocks-checkout-step-sections-summary", "")],
        vec![
            section_heading("ご注文内容", vec![]),
            ul(
                vec![("class", "blocks-checkout-step-sections-product-list")],
                PRODUCTS
                    .iter()
                    .map(|(name, variant, price)| product_row(name, variant, price))
                    .collect(),
            ),
            separator::separator(&SeparatorProps::default(), vec![]),
            data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Horizontal,
                    ..DataListProps::default()
                },
                vec![],
                vec![
                    total_row("小計", "¥12,800"),
                    total_row("送料", "¥600"),
                    total_row("税", "¥1,340"),
                    total_row("合計", "¥14,740"),
                ],
            ),
        ],
    )
}

/// 簡易決済ボタン 2 件（実在ブランド名は使わない架空の汎用名称）。
fn express_checkout_buttons() -> Node {
    div(
        vec![("class", "blocks-checkout-step-sections-express")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("ウォレットで支払う")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("ワンタップ決済")],
            ),
        ],
    )
}

/// 現在節（連絡先）。メールアドレス入力・お知らせ受信 checkbox・次へ進む
/// ボタンを持つ、唯一入力可能な節（モジュール doc「現在節・後続節の区別は
/// 色だけに頼らない」節）。
fn current_step() -> Node {
    let email = field_props(EMAIL_ID, true);
    let checkbox_props = CheckboxProps {
        disabled: true,
        ..CheckboxProps::default()
    };
    div(
        vec![
            ("class", "blocks-checkout-step-sections-step"),
            ("data-blocks-checkout-step-sections-step", "current"),
        ],
        vec![
            div(
                vec![("class", "blocks-checkout-step-sections-step-heading-row")],
                vec![
                    section_heading("連絡先", vec![]),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("入力中")],
                    ),
                ],
            ),
            field::root(
                &FieldRootProps {
                    orientation: FieldOrientation::Vertical,
                },
                &email,
                vec![],
                vec![
                    field::label(&email, vec![], vec![text("メールアドレス")]),
                    input::input(
                        &InputProps::default(),
                        &email,
                        vec![
                            ("type", "email"),
                            ("autocomplete", "email"),
                            ("placeholder", "you@example.com"),
                        ],
                    ),
                ],
            ),
            checkbox::root(
                Size::Sm,
                ColorPalette::Accent,
                &checkbox_props,
                vec![],
                vec![
                    checkbox::hidden_input(
                        &checkbox_props,
                        "blocks-checkout-step-sections-newsletter",
                        "on",
                        vec![],
                    ),
                    checkbox::control(
                        &checkbox_props,
                        vec![],
                        vec![checkbox::indicator(&checkbox_props, vec![], vec![])],
                    ),
                    checkbox::label(&checkbox_props, vec![], vec![text("お知らせを受け取る")]),
                ],
            ),
            button::button(
                &ButtonProps::default(),
                vec![],
                vec![text("配送先の入力へ進む")],
            ),
        ],
    )
}

/// 後続節 1 件（見出し + 「未着手」状態テキストのみ。入力コントロールを
/// 一切持たない、モジュール doc「現在節・後続節の区別は色だけに頼らない」
/// 節）。
fn upcoming_step(title: &'static str) -> Node {
    div(
        vec![
            ("class", "blocks-checkout-step-sections-step"),
            ("data-blocks-checkout-step-sections-step", "upcoming"),
        ],
        vec![
            separator::separator(&SeparatorProps::default(), vec![]),
            div(
                vec![("class", "blocks-checkout-step-sections-step-heading-row")],
                vec![
                    section_heading(
                        title,
                        vec![("data-blocks-checkout-step-sections-upcoming-heading", "")],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("未着手")],
                    ),
                ],
            ),
        ],
    )
}

/// もう一方の列（簡易決済 → 現在節 → 後続節一覧）全体。
fn steps_column() -> Node {
    let mut children = vec![
        express_checkout_buttons(),
        separator::separator(&SeparatorProps::default(), vec![]),
        styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text("またはメールアドレスで手続きを続ける")],
        ),
        current_step(),
    ];
    children.extend(UPCOMING_STEPS.iter().map(|title| upcoming_step(title)));
    div(
        vec![("class", "blocks-checkout-step-sections-steps")],
        children,
    )
}

/// `checkout-step-sections` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。DOM 順はサマリ → 手続き列（モジュール doc「DOM 順はサマリ →
/// 手続き列に固定し `order` を使わない」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-checkout-step-sections-layout")],
        vec![div(
            vec![("class", "blocks-checkout-step-sections-columns")],
            vec![summary_column(), steps_column()],
        )],
    )
}
```

## 原案差分メモ

- 後続節（配送先・配送方法・支払い・最終確認）は見出し + 「未着手」の
  状態テキストのみを持ち、入力コントロールを一切持ちません。現在の
  入力可否を色だけに頼らず、「入力中」「未着手」の状態テキストでも
  常に明示します。
- 編集・削除ボタンは押しても行・集計が変わらず不整合になるため
  `disabled` で固定し、商品名ごとに `aria-label` を一意にしています。
  お知らせ受信 checkbox も同じ理由でネイティブ disabled のままです。
- 簡易決済ボタンは架空の汎用名称（「ウォレットで支払う」「ワンタップ
  決済」）のみを使い、実在の決済サービス名・ロゴは使いません。
- 決済情報の入力欄（カード番号・CVC 等）は置きません。
- 2 カラム切り替えはコンテナクエリで判定します。コンテナは外側の
  ラッパー要素に宣言し、列の切り替えはその子要素に当てます（`@media`
  は使いません）。
- DOM 順はサマリ → 手続き列に固定しています（`order` は使いません）。
  狭幅（1 列表示）ではこの DOM 順どおりサマリが先頭に表示され、視覚順と
  キーボード操作順（Tab 移動）が常に一致します。
- 見出しレベルは `h3` に固定しています（ページ側の `## Demo` が `h2` を
  出すため）。
- 状態違い（現在節が配送先に進んだ状態等）の並記は本イシューのスコープ
  外です。

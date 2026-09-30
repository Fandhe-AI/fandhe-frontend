# checkout-form-summary-split

入力フォーム（左カラム: 連絡先 → 配送先 → 配送方法 → 支払い情報）と注文
サマリ（右カラム: 商品行・割引コード・集計・確定ボタン）を並べた、購入
手続き画面の 2 カラム合成例です。`field` / `input` /
`input-group` / `native-select` / `radio-card` / `checkbox` / `button` /
`image` / `separator` / `data-list` / `heading` の 11 部品を合成します。
Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

本 Demo は無 JS の静的表示のみです。`<form>` を含まず、送信処理・データ
取得を一切行いません。配送方法・支払い方法の radio card はネイティブ
disabled で固定し、選択状態が変化しないことを構造的に保証しています。
カード番号・CVC 等の決済情報入力欄は置いていません（実在の決済フォーム
に見せないための判断です）。文言・金額・住所はすべて架空のものです。

主参照は対応表 ID R0833、集約元は R0834/R0836/R0837/R0429/R0431 です
（対応表 ID のみを記載し、出典の固有名は記載しません）。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 配送方法 radio card のネイティブ `<input>` 共通 `name`。
const SHIPPING_METHOD_NAME: &str = "blocks-checkout-form-summary-split-shipping-method";
const SHIPPING_METHOD_LABEL_ID: &str = "blocks-checkout-form-summary-split-shipping-method-label";

/// 支払い方法 radio card のネイティブ `<input>` 共通 `name`。
const PAYMENT_METHOD_NAME: &str = "blocks-checkout-form-summary-split-payment-method";
const PAYMENT_METHOD_LABEL_ID: &str = "blocks-checkout-form-summary-split-payment-method-label";

const EMAIL_ID: &str = "blocks-checkout-form-summary-split-email";
const FIRST_NAME_ID: &str = "blocks-checkout-form-summary-split-first-name";
const LAST_NAME_ID: &str = "blocks-checkout-form-summary-split-last-name";
const ADDRESS_ID: &str = "blocks-checkout-form-summary-split-address";
const CITY_ID: &str = "blocks-checkout-form-summary-split-city";
const PREFECTURE_ID: &str = "blocks-checkout-form-summary-split-prefecture";
const POSTAL_CODE_ID: &str = "blocks-checkout-form-summary-split-postal-code";
const BILLING_NAME_ID: &str = "blocks-checkout-form-summary-split-billing-name";
const DISCOUNT_CODE_ID: &str = "blocks-checkout-form-summary-split-discount-code";

/// 呼び出しごとに [`FieldProps`] を組み立てる小さなヘルパ（モジュール doc
/// 「id / ARIA の方針」節）。
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

/// 縦積み（label 上・control 下）の共通 orientation。
fn orientation() -> FieldRootProps {
    FieldRootProps {
        orientation: FieldOrientation::Vertical,
    }
}

/// 見出し 1 件（`## Demo` がページ側で `h2` を出すため `h3` に固定、
/// 既存 block と同じ判断）。
fn section_heading(title: &'static str) -> Node {
    heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![],
        vec![text(title)],
    )
}

/// 連絡先セクション（メールアドレス + お知らせ配信 checkbox。checkbox は
/// 未チェック固定の静的表示）。
fn contact_section() -> Node {
    let email = field_props(EMAIL_ID, true);
    let checkbox_props = CheckboxProps::default();
    div(
        vec![("class", "blocks-checkout-form-summary-split-section")],
        vec![
            section_heading("連絡先"),
            field::root(
                &orientation(),
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
                        "blocks-checkout-form-summary-split-newsletter",
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
        ],
    )
}

/// 姓・名 2 欄の行（狭幅は 1 列、`40rem` 以上は 2 列。モジュール doc
/// 「id / ARIA の方針」節）。
fn name_row() -> Node {
    let first_name = field_props(FIRST_NAME_ID, true);
    let last_name = field_props(LAST_NAME_ID, true);
    div(
        vec![("class", "blocks-checkout-form-summary-split-name-row")],
        vec![
            field::root(
                &orientation(),
                &first_name,
                vec![],
                vec![
                    field::label(&first_name, vec![], vec![text("姓")]),
                    input::input(
                        &InputProps::default(),
                        &first_name,
                        vec![
                            ("type", "text"),
                            ("placeholder", "山田"),
                            ("autocomplete", "family-name"),
                        ],
                    ),
                ],
            ),
            field::root(
                &orientation(),
                &last_name,
                vec![],
                vec![
                    field::label(&last_name, vec![], vec![text("名")]),
                    input::input(
                        &InputProps::default(),
                        &last_name,
                        vec![
                            ("type", "text"),
                            ("placeholder", "太郎"),
                            ("autocomplete", "given-name"),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 都道府県 select の選択肢（一部抜粋の固定リスト、モジュール doc「都道
/// 府県 select をネイティブのまま操作可能にする理由」節）。
fn prefecture_options() -> Vec<Node> {
    [
        ("tokyo", "東京都", true),
        ("osaka", "大阪府", false),
        ("aichi", "愛知県", false),
        ("fukuoka", "福岡県", false),
    ]
    .into_iter()
    .map(|(value, label, selected)| {
        let mut attrs = vec![("value", value)];
        if selected {
            attrs.push(("selected", "selected"));
        }
        el("option", attrs, vec![text(label)])
    })
    .collect()
}

/// 配送先セクション（姓・名・住所・市区町村・都道府県・郵便番号）。
fn shipping_section() -> Node {
    let address = field_props(ADDRESS_ID, true);
    let city = field_props(CITY_ID, true);
    let prefecture = field_props(PREFECTURE_ID, true);
    let postal_code = field_props(POSTAL_CODE_ID, true);
    div(
        vec![("class", "blocks-checkout-form-summary-split-section")],
        vec![
            section_heading("配送先"),
            name_row(),
            field::root(
                &orientation(),
                &address,
                vec![],
                vec![
                    field::label(&address, vec![], vec![text("住所")]),
                    input::input(
                        &InputProps::default(),
                        &address,
                        vec![
                            ("type", "text"),
                            ("placeholder", "1-2-3 サンプル町"),
                            ("autocomplete", "address-line1"),
                        ],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-checkout-form-summary-split-city-row")],
                vec![
                    field::root(
                        &orientation(),
                        &city,
                        vec![],
                        vec![
                            field::label(&city, vec![], vec![text("市区町村")]),
                            input::input(
                                &InputProps::default(),
                                &city,
                                vec![
                                    ("type", "text"),
                                    ("placeholder", "渋谷区"),
                                    ("autocomplete", "address-level2"),
                                ],
                            ),
                        ],
                    ),
                    field::root(
                        &orientation(),
                        &prefecture,
                        vec![],
                        vec![
                            field::label(&prefecture, vec![], vec![text("都道府県")]),
                            native_select::native_select(
                                &NativeSelectProps::default(),
                                &prefecture,
                                vec![("autocomplete", "address-level1")],
                                prefecture_options(),
                            ),
                        ],
                    ),
                ],
            ),
            field::root(
                &orientation(),
                &postal_code,
                vec![],
                vec![
                    field::label(&postal_code, vec![], vec![text("郵便番号")]),
                    input::input(
                        &InputProps::default(),
                        &postal_code,
                        vec![
                            ("type", "text"),
                            ("placeholder", "150-0002"),
                            ("autocomplete", "postal-code"),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// radio card 1 件（`checked`/`disabled`/`value`/`label`/`description` から
/// 組み立てる。配送方法・支払い方法の双方で共有する、モジュール doc
/// 「配送方法・支払い方法 radio card をネイティブ disabled にする理由」
/// 節）。
fn radio_card_item(
    checked: bool,
    name: &'static str,
    value: &'static str,
    label: &'static str,
    description: &'static str,
) -> Node {
    radio_card::item(
        checked,
        true,
        value,
        vec![],
        vec![
            radio_card::item_hidden_input(checked, true, Some(name), value, vec![]),
            radio_card::item_control(
                checked,
                true,
                vec![],
                vec![
                    radio_card::item_indicator(checked, true, false, vec![]),
                    radio_card::item_content(
                        vec![],
                        vec![
                            radio_card::item_text(vec![], vec![text(label)]),
                            radio_card::item_description(vec![], vec![text(description)]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 配送方法セクション（fieldset は使わず `radio_card::label` の id を
/// `labelled_by` へ渡す構成、`card_form_footer::payment_method_field` と
/// 同型）。「通常配送」を選択済みで固定する。
fn shipping_method_section() -> Node {
    div(
        vec![("class", "blocks-checkout-form-summary-split-section")],
        vec![
            section_heading("配送方法"),
            radio_card::label(
                Some(SHIPPING_METHOD_LABEL_ID),
                vec![],
                vec![text("配送方法を選択")],
            ),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                None::<Orientation>,
                Some(SHIPPING_METHOD_LABEL_ID),
                vec![("aria-disabled", "true")],
                vec![
                    radio_card_item(
                        true,
                        SHIPPING_METHOD_NAME,
                        "standard",
                        "通常配送",
                        "3〜5 営業日でお届けします。",
                    ),
                    radio_card_item(
                        false,
                        SHIPPING_METHOD_NAME,
                        "express",
                        "お急ぎ便",
                        "1〜2 営業日でお届けします。",
                    ),
                ],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("現在の選択: 通常配送")],
            ),
        ],
    )
}

/// 支払い方法セクション（「カード払い」選択済みで固定 + 請求先の宛名
/// 欄）。カード番号等の決済情報入力欄は置かない（モジュール doc
/// 「`<form>` を持たない」節）。
fn payment_method_section() -> Node {
    let billing_name = field_props(BILLING_NAME_ID, true);
    div(
        vec![("class", "blocks-checkout-form-summary-split-section")],
        vec![
            section_heading("お支払い方法"),
            radio_card::label(
                Some(PAYMENT_METHOD_LABEL_ID),
                vec![],
                vec![text("お支払い方法を選択")],
            ),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                None::<Orientation>,
                Some(PAYMENT_METHOD_LABEL_ID),
                vec![("aria-disabled", "true")],
                vec![
                    radio_card_item(
                        true,
                        PAYMENT_METHOD_NAME,
                        "card",
                        "カード払い",
                        "登録済みのカードから引き落とします。",
                    ),
                    radio_card_item(
                        false,
                        PAYMENT_METHOD_NAME,
                        "bank",
                        "銀行振込",
                        "指定口座へお振込みいただきます。",
                    ),
                    radio_card_item(
                        false,
                        PAYMENT_METHOD_NAME,
                        "cod",
                        "代金引換",
                        "商品お受け取り時にお支払いいただきます。",
                    ),
                ],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("現在の選択: カード払い")],
            ),
            field::root(
                &orientation(),
                &billing_name,
                vec![],
                vec![
                    field::label(&billing_name, vec![], vec![text("請求先の宛名")]),
                    input::input(
                        &InputProps::default(),
                        &billing_name,
                        vec![("type", "text"), ("placeholder", "山田 太郎")],
                    ),
                ],
            ),
        ],
    )
}

/// 左カラム（入力フォーム）全体。連絡先 → 配送先 → 配送方法 → 支払い
/// 情報の順に縦積みする。
fn form_column() -> Node {
    div(
        vec![("class", "blocks-checkout-form-summary-split-form")],
        vec![
            contact_section(),
            separator::separator(&SeparatorProps::default(), vec![]),
            shipping_section(),
            separator::separator(&SeparatorProps::default(), vec![]),
            shipping_method_section(),
            separator::separator(&SeparatorProps::default(), vec![]),
            payment_method_section(),
        ],
    )
}

/// 商品行 1 件（画像 + 名前・バリエーション + 価格）。
fn product_row(name: &'static str, variant_label: &'static str, price: &'static str) -> Node {
    li(
        vec![("class", "blocks-checkout-form-summary-split-product-row")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    aspect_ratio: AspectRatio::Square,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-checkout-form-summary-split-product-image", "")],
            ),
            div(
                vec![("class", "blocks-checkout-form-summary-split-product-detail")],
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
                ],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![("data-blocks-checkout-form-summary-split-product-price", "")],
                vec![text(price)],
            ),
        ],
    )
}

/// 割引コード欄（input + 「適用」ボタンを 1 本の入力グループへ一体化、
/// `hero_email_signup::signup_group` と同型）。
fn discount_code_group() -> Node {
    let discount = field_props(DISCOUNT_CODE_ID, false);
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    field::root(
        &orientation(),
        &discount,
        vec![],
        vec![
            field::label(&discount, vec![], vec![text("割引コード")]),
            input_group::root(
                &group_props,
                vec![("data-blocks-checkout-form-summary-split-discount-group", "")],
                vec![
                    input::input(
                        &InputProps::default(),
                        &discount,
                        vec![("type", "text"), ("placeholder", "コードを入力")],
                    ),
                    input_group::addon(
                        InputGroupAlign::InlineEnd,
                        &group_props,
                        vec![],
                        vec![button::button(
                            &ButtonProps::default(),
                            vec![],
                            vec![text("適用")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// ラベル・値の 1 行（`data_list::item` + `item-label` + `item-value`）。
fn total_row(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 集計（小計・送料・税・合計）の定義リスト。
fn totals() -> Node {
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
    )
}

/// 右カラム（注文サマリ）全体。商品行 → 割引コード → 集計 → 確定ボタン
/// の順に縦積みする。
fn summary_column() -> Node {
    div(
        vec![("data-blocks-checkout-form-summary-split-summary", "")],
        vec![
            section_heading("ご注文内容"),
            ul(
                vec![("class", "blocks-checkout-form-summary-split-product-list")],
                vec![
                    product_row("キャンバストートバッグ", "カラー: ナチュラル", "¥6,400"),
                    product_row("セラミックマグカップ", "カラー: ホワイト", "¥3,200"),
                    product_row("コットンソックス 2 足組", "サイズ: M", "¥3,200"),
                ],
            ),
            separator::separator(&SeparatorProps::default(), vec![]),
            discount_code_group(),
            separator::separator(&SeparatorProps::default(), vec![]),
            totals(),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-checkout-form-summary-split-confirm", "")],
                vec![text("注文を確定する")],
            ),
        ],
    )
}

/// `checkout-form-summary-split` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。DOM 順はフォーム → サマリ（モジュール doc「狭幅
/// でサマリを先頭にする理由」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-checkout-form-summary-split-layout")],
        vec![form_column(), summary_column()],
    )
}
```

## 原案差分メモ

- 配送方法・支払い方法は `radio-card` を使い、ネイティブ `disabled` で
  固定した静的な初期状態のみを描きます（選択状態の JS 追従を行わない
  ため）。
- 支払い方法にカード番号・CVC 等の入力欄は置きません（実在の決済フォーム
  に見せないための判断）。
- 狭幅（1 列表示）ではサマリを CSS の `order` で先頭に回します。DOM 順は
  フォーム → サマリのままです。
- 見出しレベルは `h3` に固定しています（ページ側の `## Demo` が `h2` を
  出すため）。
- 簡易決済ボタン行版・サマリ反転配色版・割引バッジ・状態違いの並記は、
  本イシュー（前半）のスコープ外とし、後半（親 #3041 配下）で追加します。

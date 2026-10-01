# checkout-form-summary-split

注文サマリ（商品行・割引コード・集計・確定ボタン）と入力フォーム
（連絡先 → 配送先 → 配送方法 → 支払い情報）を並べた、購入手続き画面
の 2 カラム合成例です。集約元 6 件（R0833/R0834/R0836/R0837/R0429/
R0431、対応表 ID のみ）の差分を読み取れる 3 版を並記します。
`field` / `input` / `input-group` / `native-select` /
`radio-card` / `radio-group` / `fieldset` / `checkbox` /
`badge` / `button` / `image` / `separator` / `data-list` /
`heading` の 14 部品を合成します。Blocks は既存部品の合成例であり、
新しい UI 部品は追加しません。

- **版 A（代表構成）**: サマリ先頭（狭幅）・配送方法あり・割引コード
  入力欄あり。
- **版 B（簡易決済 + 割引適用済み + 請求先指定）**: フォーム先頭に
  簡易決済ボタン行を置き、フォーム・サマリの列順を入れ替えます
  （狭幅ではサマリが末尾）。サマリは割引コード適用済み（badge +
  集計の「割引」行）で、支払い節の後ろへ請求先住所の選択を追加
  します。
- **版 C（反転配色・閲覧専用サマリ）**: サマリ面を反転配色にし、
  操作要素（割引コード入力・確定ボタン）を持たない閲覧専用に
  します。確定ボタンはフォーム列の末尾へ移り、配送方法の節は
  持ちません。

本 Demo は無 JS の静的表示のみです。`<form>` を含まず、送信処理・
データ取得を一切行いません。配送方法・支払い方法の radio card は
ネイティブ disabled で固定し、選択状態が変化しないことを構造的に
保証しています。カード番号・CVC 等の決済情報入力欄は置いていません
（実在の決済フォームに見せないための判断です）。簡易決済ボタンは
実在ブランドの名称・ロゴ・配色を使わない汎用名です。文言・金額・
住所はすべて架空のものです。

主参照は対応表 ID R0833、集約元は R0834/R0836/R0837/R0429/R0431 です
（対応表 ID のみを記載し、出典の固有名は記載しません）。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, p, section, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation};
use fandhe_frontend_pre_styled_ui::radio_group::{self, RadioGroupProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 版ごとの構成差分（モジュール doc「状態違いの並記」節）。抽象化は本
/// 構造体 1 個に限り、trait・builder は作らない（ponytail: 1 abstraction）。
struct Variant {
    /// id・`name` を一意化する接尾辞（`"a"`/`"b"`/`"c"`）。
    suffix: &'static str,
    /// 簡易決済ボタン行（版 B）を持つか。
    express: bool,
    /// 配送方法の節を持つか（版 C は持たない）。
    shipping_method: bool,
    /// 支払い節の後ろへ独立した請求先住所セクション（`fieldset` +
    /// `radio_group`）を追加するか（版 B のみ）。`false` の版は従来どおり
    /// 支払い節内に請求先の宛名フィールドを inline する。
    billing_section: bool,
    /// サマリの割引コードが適用済みか（版 B）。`true` のとき操作用の入力欄
    /// は出さず、代わりに badge と集計の「割引」行を出す。
    discount_applied: bool,
    /// サマリに割引コード入力欄を出すか（版 A のみ）。
    show_discount_input: bool,
    /// `true` で列順を「フォーム→サマリ」（版 B、狭幅ではサマリが末尾）に
    /// する。`false` は既定の「サマリ→フォーム」（版 A/C）。
    summary_end: bool,
    /// サマリ面を反転配色の閲覧専用にするか（版 C のみ）。`true` のとき
    /// 確定ボタンはフォーム列の末尾へ移る。
    inverted: bool,
}

const VARIANT_A: Variant = Variant {
    suffix: "a",
    express: false,
    shipping_method: true,
    billing_section: false,
    discount_applied: false,
    show_discount_input: true,
    summary_end: false,
    inverted: false,
};

const VARIANT_B: Variant = Variant {
    suffix: "b",
    express: true,
    shipping_method: true,
    billing_section: true,
    discount_applied: true,
    show_discount_input: false,
    summary_end: true,
    inverted: false,
};

const VARIANT_C: Variant = Variant {
    suffix: "c",
    express: false,
    shipping_method: false,
    billing_section: false,
    discount_applied: false,
    show_discount_input: false,
    summary_end: false,
    inverted: true,
};

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
const BILLING_ADDRESS_ID: &str = "blocks-checkout-form-summary-split-billing-address";

/// 呼び出しごとに [`FieldProps`] を組み立てる小さなヘルパ（モジュール doc
/// 「id / ARIA の方針」節）。`id` は版ごとの一意な借用先（`format!` した
/// ローカル `String` 等）を受け取れるよう、`'static` ではなく汎用の
/// ライフタイムで受ける。
fn field_props(id: &str, required: bool) -> FieldProps<'_> {
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
/// 未チェック固定の静的表示のため、JS ハイドレーションなしでもネイティブ
/// input の checked 状態とカスタム indicator の表示が食い違わないよう
/// `disabled: true` でネイティブ操作を止める（#3462 レビュー指摘対応）。
/// styled checkbox の disabled の見た目（`opacity: 0.5; cursor:
/// not-allowed;`）は中和せず残し、操作できない固定表示であることを示す
/// （配送方法・支払い方法 radio card と同じ判断、#3462 codex 指摘）。
fn contact_section(suffix: &str) -> Node {
    let email_id = format!("{EMAIL_ID}-{suffix}");
    let email = field_props(&email_id, true);
    let checkbox_name = format!("blocks-checkout-form-summary-split-newsletter-{suffix}");
    let checkbox_props = CheckboxProps {
        disabled: true,
        ..CheckboxProps::default()
    };
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
                    checkbox::hidden_input(&checkbox_props, &checkbox_name, "on", vec![]),
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
fn name_row(suffix: &str) -> Node {
    let first_name_id = format!("{FIRST_NAME_ID}-{suffix}");
    let last_name_id = format!("{LAST_NAME_ID}-{suffix}");
    let first_name = field_props(&first_name_id, true);
    let last_name = field_props(&last_name_id, true);
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
fn shipping_section(suffix: &str) -> Node {
    let address_id = format!("{ADDRESS_ID}-{suffix}");
    let city_id = format!("{CITY_ID}-{suffix}");
    let prefecture_id = format!("{PREFECTURE_ID}-{suffix}");
    let postal_code_id = format!("{POSTAL_CODE_ID}-{suffix}");
    let address = field_props(&address_id, true);
    let city = field_props(&city_id, true);
    let prefecture = field_props(&prefecture_id, true);
    let postal_code = field_props(&postal_code_id, true);
    div(
        vec![("class", "blocks-checkout-form-summary-split-section")],
        vec![
            section_heading("配送先"),
            name_row(suffix),
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
    name: &str,
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
fn shipping_method_section(suffix: &str) -> Node {
    let name = format!("{SHIPPING_METHOD_NAME}-{suffix}");
    let label_id = format!("{SHIPPING_METHOD_LABEL_ID}-{suffix}");
    div(
        vec![("class", "blocks-checkout-form-summary-split-section")],
        vec![
            section_heading("配送方法"),
            radio_card::label(Some(&label_id), vec![], vec![text("配送方法を選択")]),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                None::<Orientation>,
                Some(&label_id),
                vec![("aria-disabled", "true")],
                vec![
                    radio_card_item(
                        true,
                        &name,
                        "standard",
                        "通常配送",
                        "3〜5 営業日でお届けします。",
                    ),
                    radio_card_item(
                        false,
                        &name,
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

/// 支払い方法セクション（「カード払い」選択済みで固定）。カード番号等の
/// 決済情報入力欄は置かない（モジュール doc「`<form>` を持たない」節）。
/// `include_billing_name` が `true`（版 A/C）のときのみ請求先の宛名欄を
/// inline する。`false`（版 B）のときは呼び出し側が [`billing_address_section`]
/// を別途続ける（モジュール doc「状態違いの並記」節）。
fn payment_method_section(suffix: &str, include_billing_name: bool) -> Node {
    let name = format!("{PAYMENT_METHOD_NAME}-{suffix}");
    let label_id = format!("{PAYMENT_METHOD_LABEL_ID}-{suffix}");
    let billing_name_id = format!("{BILLING_NAME_ID}-{suffix}");
    let billing_name = field_props(&billing_name_id, true);
    let mut children = vec![
        section_heading("お支払い方法"),
        radio_card::label(Some(&label_id), vec![], vec![text("お支払い方法を選択")]),
        radio_card::root(
            Size::Sm,
            ColorPalette::Accent,
            true,
            None::<Orientation>,
            Some(&label_id),
            vec![("aria-disabled", "true")],
            vec![
                radio_card_item(
                    true,
                    &name,
                    "card",
                    "カード払い",
                    "登録済みのカードから引き落とします。",
                ),
                radio_card_item(
                    false,
                    &name,
                    "bank",
                    "銀行振込",
                    "指定口座へお振込みいただきます。",
                ),
                radio_card_item(
                    false,
                    &name,
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
    ];
    if include_billing_name {
        children.push(field::root(
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
        ));
    }
    div(
        vec![("class", "blocks-checkout-form-summary-split-section")],
        children,
    )
}

/// 請求先住所の radio group 項目 1 件（`fieldset::legend` の id を
/// `radio_group::root` の `labelled_by` へ渡す構成、`form_layout_stacked::
/// notifications_section` の `push_notification_item` と同型）。
fn billing_address_item(
    checked: bool,
    props: &RadioGroupProps,
    name: &str,
    value: &'static str,
    label: &'static str,
) -> Node {
    radio_group::item(
        checked,
        props,
        value,
        vec![],
        vec![
            radio_group::item_hidden_input(checked, props, Some(name), value, vec![]),
            radio_group::item_control(checked, props, vec![]),
            radio_group::item_text(checked, props, vec![], vec![text(label)]),
        ],
    )
}

/// 請求先住所セクション（版 B 固有、R0429 の 5 節構成を近似。モジュール
/// doc「状態違いの並記」節）。`fieldset` + 読み取り専用（`disabled: true`）
/// の `radio_group` で「配送先と同じ」を選択済みにする。配送方法・支払い
/// 方法 radio card と同じ理由でネイティブ disabled にする（モジュール doc
/// 「配送方法・支払い方法 radio card をネイティブ disabled にする理由」
/// 節、radio_group 版は `radio_card::item_hidden_input` の代わりに
/// `RadioGroupProps::disabled` で表現する）。
fn billing_address_section(suffix: &str) -> Node {
    let fieldset_id = format!("{BILLING_ADDRESS_ID}-{suffix}");
    let legend_id = format!("{fieldset_id}-legend");
    let name = format!("{BILLING_ADDRESS_ID}-{suffix}");
    let fieldset_props = FieldsetProps {
        id: &fieldset_id,
        disabled: false,
        invalid: false,
        has_helper_text: false,
    };
    let radio_props = RadioGroupProps {
        disabled: true,
        ..RadioGroupProps::default()
    };
    fieldset::root(
        &FieldsetRootProps::default(),
        &fieldset_props,
        vec![("class", "blocks-checkout-form-summary-split-section")],
        vec![
            fieldset::legend(&fieldset_props, vec![], vec![text("請求先住所")]),
            radio_group::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                None::<Orientation>,
                Some(&legend_id),
                vec![],
                vec![
                    billing_address_item(true, &radio_props, &name, "same", "配送先と同じ"),
                    billing_address_item(false, &radio_props, &name, "different", "別の住所を指定"),
                ],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("現在の選択: 配送先と同じ")],
            ),
        ],
    )
}

/// 簡易決済ボタン行（版 B 固有。`checkout_step_sections::
/// express_checkout_buttons` と同じラベルを再利用し、実在ブランドの
/// 名称・ロゴ・配色を持ち込まない、モジュール doc「`<form>` を持たない」
/// 節）。
fn express_checkout_row() -> Node {
    div(
        vec![("class", "blocks-checkout-form-summary-split-express")],
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

/// 確定ボタン（`注文を確定する`、送信しない種別のまま）。版 A/B はサマリ
/// 列末尾、版 C はフォーム列末尾に置く（モジュール doc「状態違いの並記」
/// 節）。
fn confirm_button() -> Node {
    button::button(
        &ButtonProps::default(),
        vec![("data-blocks-checkout-form-summary-split-confirm", "")],
        vec![text("注文を確定する")],
    )
}

/// 右カラム（入力フォーム）全体。版 B は先頭に簡易決済ボタン行を、版 C は
/// 末尾に確定ボタンを追加する（モジュール doc「状態違いの並記」節）。
fn form_column(v: &Variant) -> Node {
    let mut children: Vec<Node> = Vec::new();
    if v.express {
        children.push(express_checkout_row());
        children.push(separator::separator(&SeparatorProps::default(), vec![]));
        children.push(styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text("またはメールアドレスで手続きを続ける")],
        ));
    }
    children.push(contact_section(v.suffix));
    children.push(separator::separator(&SeparatorProps::default(), vec![]));
    children.push(shipping_section(v.suffix));
    if v.shipping_method {
        children.push(separator::separator(&SeparatorProps::default(), vec![]));
        children.push(shipping_method_section(v.suffix));
    }
    children.push(separator::separator(&SeparatorProps::default(), vec![]));
    children.push(payment_method_section(v.suffix, !v.billing_section));
    if v.billing_section {
        children.push(separator::separator(&SeparatorProps::default(), vec![]));
        children.push(billing_address_section(v.suffix));
    }
    if v.inverted {
        children.push(separator::separator(&SeparatorProps::default(), vec![]));
        children.push(confirm_button());
    }
    div(
        vec![("class", "blocks-checkout-form-summary-split-form")],
        children,
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
/// `hero_email_signup::signup_group` と同型）。版 A のみが持つ（モジュール
/// doc「状態違いの並記」節、版 B は適用済みバッジ、版 C は閲覧専用で
/// どちらも持たない）。
fn discount_code_group(suffix: &str) -> Node {
    let discount_id = format!("{DISCOUNT_CODE_ID}-{suffix}");
    let discount = field_props(&discount_id, false);
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

/// 集計（小計・送料・税・合計）の定義リスト。`discount_applied` が `true`
/// （版 B）のとき「割引」行を挟み、送料・税・合計を割引後の金額で再計算
/// する（モジュール doc「状態違いの並記」節、架空値の整合: 小計 12,800 −
/// 割引 1,280 + 送料 600 = 課税対象 12,120、税 10% = 1,212、合計 13,332）。
fn totals(discount_applied: bool) -> Node {
    let mut rows = vec![total_row("小計", "¥12,800")];
    if discount_applied {
        rows.push(total_row("割引", "-¥1,280"));
    }
    rows.push(total_row("送料", "¥600"));
    if discount_applied {
        rows.push(total_row("税", "¥1,212"));
        rows.push(total_row("合計", "¥13,332"));
    } else {
        rows.push(total_row("税", "¥1,340"));
        rows.push(total_row("合計", "¥14,740"));
    }
    data_list::root(
        DataListProps {
            orientation: DataListOrientation::Horizontal,
            ..DataListProps::default()
        },
        vec![],
        rows,
    )
}

/// 適用済み割引コードの表示行（版 B 固有、[`badge`] で明示する。モジュール
/// doc「状態違いの並記」節）。
fn discount_applied_badge() -> Node {
    div(
        vec![(
            "class",
            "blocks-checkout-form-summary-split-discount-badge-row",
        )],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("適用中のコード")],
            ),
            badge::badge(&BadgeProps::default(), vec![], vec![text("WELCOME10")]),
        ],
    )
}

/// 左カラム（注文サマリ）全体。商品行 → （割引コード入力 or 適用済み
/// バッジ）→ 集計 → 確定ボタンの順に縦積みする。版 C（`v.inverted`）は
/// 反転配色の閲覧専用にし、割引コード入力・確定ボタンを持たない（モジュール
/// doc「反転配色の面に入力・ボタンを置かない理由」節）。
fn summary_column(v: &Variant) -> Node {
    let mut attrs = vec![("data-blocks-checkout-form-summary-split-summary", "")];
    if v.inverted {
        attrs.push(("data-blocks-checkout-form-summary-split-tone", "inverted"));
    }
    let mut children = vec![section_heading("ご注文内容")];
    if v.discount_applied {
        children.push(discount_applied_badge());
    }
    children.push(ul(
        vec![("class", "blocks-checkout-form-summary-split-product-list")],
        vec![
            product_row("キャンバストートバッグ", "カラー: ナチュラル", "¥6,400"),
            product_row("セラミックマグカップ", "カラー: ホワイト", "¥3,200"),
            product_row("コットンソックス 2 足組", "サイズ: M", "¥3,200"),
        ],
    ));
    children.push(separator::separator(&SeparatorProps::default(), vec![]));
    if v.show_discount_input {
        children.push(discount_code_group(v.suffix));
        children.push(separator::separator(&SeparatorProps::default(), vec![]));
    }
    children.push(totals(v.discount_applied));
    if !v.inverted {
        children.push(confirm_button());
    }
    div(attrs, children)
}

/// 版キャプション（`product_overview_gallery_split::caption` と同型、素の
/// `<p>` で `heading` 部品を使わない）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-checkout-form-summary-split-caption")],
        vec![text(label)],
    )
}

/// 版 1 件のレイアウト（`-layout` → `-columns` → [サマリ列, フォーム列]
/// または逆順）。`v.summary_end` で列順・列幅配分を切り替える（モジュール
/// doc「DOM 順を視覚順へ一致させる理由」節、`order` は使わない）。
fn variant_layout(v: &Variant) -> Node {
    let mut columns_attrs = vec![("class", "blocks-checkout-form-summary-split-columns")];
    if v.summary_end {
        columns_attrs.push((
            "data-blocks-checkout-form-summary-split-summary-position",
            "end",
        ));
    }
    let columns = if v.summary_end {
        vec![form_column(v), summary_column(v)]
    } else {
        vec![summary_column(v), form_column(v)]
    };
    div(
        vec![
            ("class", "blocks-checkout-form-summary-split-layout"),
            ("data-blocks-checkout-form-summary-split-variant", v.suffix),
        ],
        vec![div(columns_attrs, columns)],
    )
}

/// `checkout-form-summary-split` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。3 版をキャプション付きで縦に並べる（モジュール doc
/// 「状態違いの並記」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-checkout-form-summary-split-demo")],
        vec![
            caption("代表構成（サマリ先頭・配送方法あり・割引コード入力欄）"),
            section(vec![], vec![variant_layout(&VARIANT_A)]),
            caption("簡易決済 + 割引適用済み + 請求先指定（フォーム先頭・サマリ末尾）"),
            section(vec![], vec![variant_layout(&VARIANT_B)]),
            caption("反転配色・閲覧専用サマリ（配送方法なし・確定ボタンはフォーム末尾）"),
            section(vec![], vec![variant_layout(&VARIANT_C)]),
        ],
    )
}
```

## 原案差分メモ

- 配送方法・支払い方法は `radio-card` を使い、ネイティブ `disabled` で
  固定した静的な初期状態のみを描きます（選択状態の JS 追従を行わない
  ため）。disabled の見た目は打ち消さずに残し、現在の選択は
  「現在の選択: …」のテキストでも示します。お知らせ配信の checkbox も
  同じ扱いです。
- 2 カラム切り替えはコンテナクエリで判定します。コンテナは外側の
  ラッパー要素に宣言し、列の切り替えはその子要素に当てます。
- 支払い方法にカード番号・CVC 等の入力欄は置きません（実在の決済フォーム
  に見せないための判断）。
- DOM 順は版 A/C がサマリ → フォーム、版 B がフォーム → サマリに固定して
  います（`order` は使いません）。狭幅（1 列表示）ではこの DOM 順どおり
  版 A/C はサマリが先頭、版 B はサマリが末尾に表示され、視覚順と
  キーボード操作順（Tab 移動）が常に一致します。
- 見出しレベルは `h3` に固定しています（ページ側の `## Demo` が `h2` を
  出すため）。
- 反転配色の版 C は、サマリ面に入力・ボタンを置くとコントラストが崩れ
  うるため、閲覧専用（商品行・集計の表示のみ）に限定しています。

### 集約元 6 件との対応

- **R0833**: 版 A の骨格（連絡先・配送先・配送方法・支払い・サマリ）。
- **R0429**: 版 A の節構成、および版 B の請求先住所セクションで近似
  （5 節構成のうち請求先は独立ページ遷移ではなく支払い節の続きとして
  合成しています）。
- **R0836**: 版 B の簡易決済ボタン行。
- **R0431**: 版 B の簡易決済・割引適用（badge + 集計の「割引」行）。
- **R0837**: 版 C の反転配色サマリ。
- **R0834**: 版 C の配送方法なしの構成。

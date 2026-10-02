# promo-signup-offer

角丸カードを画像列（片側）と登録フォーム（反対側）の 2 列へ分けた、画像付き
の登録特典カードです。`card` / `heading` / `text` / `field` / `input` /
`button` / `image` / `radio-group` / `fieldset` / `link` の 10 部品を
合成します。Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0346 です。

メール登録フォームはダミー表示のみであり、`<form>` を含まず送信処理・
データ取得は一切行いません。文言はすべて架空のデータであり、実在の企業・
ブランド・PII・実クレデンシャルは含みません。画像はビルド時生成の同梱
プレースホルダー SVG です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::radio_group::{self, RadioGroupProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// リンク先の固定外部 URL（同意文リンク、モジュール doc「見出しレベル」節に
/// 準じ死リンク `href="#"` は使わない既存方針）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 興味カテゴリ fieldset の `id`。legend の id は headless
/// [`fieldset::legend`] の導出規則（`"{id}-legend"`）に一致させてリテラルで
/// 直書きする（`format!` は使わない）。
const INTEREST_FIELDSET_ID: &str = "blocks-promo-signup-offer-interest";
const INTEREST_LEGEND_ID: &str = "blocks-promo-signup-offer-interest-legend";
/// 興味カテゴリ radio group のネイティブ `<input>` の共通 `name`。
const INTEREST_NAME: &str = "blocks-promo-signup-offer-interest";

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`）。
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

/// 興味カテゴリ選択肢 1 件（`radio_group::item` 3 パーツの組み立て）。
fn interest_item(
    checked: bool,
    props: &RadioGroupProps,
    value: &'static str,
    label: &'static str,
) -> Node {
    radio_group::item(
        checked,
        props,
        value,
        vec![],
        vec![
            radio_group::item_hidden_input(checked, props, Some(INTEREST_NAME), value, vec![]),
            radio_group::item_control(checked, props, vec![]),
            radio_group::item_text(checked, props, vec![], vec![text(label)]),
        ],
    )
}

/// 形 B（R0345）が追加する興味カテゴリ fieldset + radio group（ネイティブ
/// 操作不能にする理由はモジュール doc「興味カテゴリ radio group をネイティブ
/// disabled にする理由」節）。
fn interest_fieldset() -> Node {
    let fieldset_props = FieldsetProps {
        id: INTEREST_FIELDSET_ID,
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
        vec![("data-blocks-promo-signup-offer-interest-fieldset", "")],
        vec![
            fieldset::legend(&fieldset_props, vec![], vec![text("興味のあるカテゴリ")]),
            radio_group::root(
                Size::Md,
                ColorPalette::Accent,
                true,
                None,
                Some(INTEREST_LEGEND_ID),
                vec![("data-blocks-promo-signup-offer-interest-group", "")],
                vec![
                    interest_item(true, &radio_props, "apparel", "アパレル"),
                    interest_item(false, &radio_props, "home", "生活雑貨"),
                    interest_item(false, &radio_props, "food", "食品"),
                ],
            ),
        ],
    )
}

/// 形 C（R0347）が末尾に添える同意文（text + link）。
fn consent_text() -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-promo-signup-offer-consent", "")],
        vec![
            text("登録すると"),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("利用規約")],
            ),
            text("に同意したものとみなされます。"),
        ],
    )
}

/// カード 1 枚分（画像列 + 本文列）を組み立てる。`layout` は
/// `data-blocks-promo-signup-offer-layout` へ渡す属性値
/// （`"default"`/`"reverse"`/`"centered"`）で、[`LAYOUT_CSS`] が CSS フックに
/// 使う。`email_id` は variant ごとに一意なリテラルを呼び出し側から渡す
/// （モジュール doc「id / ARIA の方針」節）。`extra` は興味カテゴリ fieldset
/// や同意文の差し込みに使う。
fn offer_card(layout: &'static str, email_id: &'static str, extra: Vec<Node>) -> Node {
    let email = FieldProps {
        id: email_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };

    let mut body_children = vec![
        image::image(
            &ImageProps::new(dummy_assets::LOGO_SRC, ""),
            vec![("data-blocks-promo-signup-offer-logo", "")],
        ),
        heading::heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl2,
                ..HeadingProps::default()
            },
            vec![],
            vec![text("先行登録で特典をゲット")],
        ),
        styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(
                "メールアドレスをご登録いただくと、次回のお買い物でお使いいただける特典をお送りします。",
            )],
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
                        ("placeholder", "you@example.com"),
                        ("autocomplete", "email"),
                    ],
                ),
            ],
        ),
    ];
    body_children.extend(extra);
    body_children.push(button::button(
        &ButtonProps::default(),
        vec![("data-blocks-promo-signup-offer-submit", "")],
        vec![text("今すぐ登録する")],
    ));

    let media = div(
        vec![("class", "blocks-promo-signup-offer-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
            },
            vec![("data-blocks-promo-signup-offer-image", "")],
        )],
    );
    let body = card::body(
        vec![("data-blocks-promo-signup-offer-body", "")],
        body_children,
    );

    card::root(
        CardProps::default(),
        vec![
            ("data-blocks-promo-signup-offer-card", ""),
            ("data-blocks-promo-signup-offer-layout", layout),
        ],
        vec![media, body],
    )
}

/// `promo-signup-offer` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（他 block と同じ契約）。4 形を縦に並べる（モジュール doc「4 形を
/// 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-promo-signup-offer-layout")],
        vec![
            variant_label("基準形"),
            offer_card("default", "blocks-promo-signup-offer-email-a", vec![]),
            variant_label("カテゴリ選択"),
            offer_card(
                "default",
                "blocks-promo-signup-offer-email-b",
                vec![interest_fieldset()],
            ),
            variant_label("左右反転 + 同意文"),
            offer_card(
                "reverse",
                "blocks-promo-signup-offer-email-c",
                vec![consent_text()],
            ),
            variant_label("中央寄せ"),
            offer_card("centered", "blocks-promo-signup-offer-email-d", vec![]),
        ],
    )
}
```

## 差分メモ

主参照は R0346（基準形）で、画像列（片側）と登録フォーム（反対側）の
2 列構成をそのまま採用しています。

- **R0345（カテゴリ選択）**: 基準形のフォーム列へ、興味のあるカテゴリを
  選ぶ fieldset + radio group を追加した形として並記しています。radio は
  先頭 1 件が選択済みの静的な初期状態で、ネイティブ操作不能にする
  `disabled: true` で固定しています（他 block と同じ判断）。
- **R0347（左右反転 + 同意文）**: `48rem` 以上で画像列へ `order: 2` を
  与えて左右を入れ替え、登録ボタンの下に利用規約への同意文を添えた形として
  並記しています。
- **R0348（中央寄せ）**: フォーム列の内容を中央寄せにした形として
  並記しています。
- **状態表示**: 無 JS のため本 block はトグル系の状態（開閉・選択等）を
  持たず、4 形は差分がある部分のみを静的に並記しています。
- **md 未満の画像非表示**: `48rem` は `Breakpoint::Md`
  （`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`）と一致する
  リテラル値です。

関連情報: [Card](../themes/card.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Field](../themes/field.md) /
[Input](../themes/input.md) / [Button](../themes/button.md) /
[Image](../themes/image.md) / [Radio Group](../themes/radio-group.md) /
[Fieldset](../themes/fieldset.md) / [Link](../themes/link.md)

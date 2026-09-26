# contact-split-form-image

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `field` / `input` /
`textarea` / `radio-group` / `fieldset` / `separator` / `button` / `image` の
10 部品を合成した、フォーム + 画像の 2 カラムお問い合わせ用ブロックです。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0065、集約元は R0859。取り込んだのは領域配置・部品構成の構造だけで、
文言・配色・装飾は持ち込んでいません。出典の固有名・ファイル名は記載しません）。

幅 `48rem`（`md`）以上では左に問い合わせフォーム、右に画像を配置した 2 列
grid になります。`48rem` 未満ではフォームのみを 1 列で表示し、画像は隠れます。
フォームは氏名（姓・名、`48rem` 以上で横並び）・メールアドレス・電話番号
（任意、補助テキスト付き）・お問い合わせ内容（本文、補助テキスト付き）・
ご予算（fieldset + radio group、4 択、`48rem` 以上で 2×2）の順に並び、
区切り線の下に右寄せの送信ボタンを配置します。

本 Demo は静的な表示例であり、`<form>` 要素を一切持たず、データの取得・
送信・状態管理を行いません。送信ボタンは `type="button"` のままで、暗黙の
submit も起きません。ご予算の選択肢は先頭 1 件のみが選択済みの静的な初期
状態で固定しています。文言はすべて独自に書いた架空のものであり、実企業名・
実クレデンシャル・PII を含みません。画像はビルド時生成のプレースホルダー
SVG（`alt=""`）です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::radio_group::{self, RadioGroupProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 予算 radio group のネイティブ `<input>` の共通 `name`（モジュール doc
/// 「id / ARIA の方針」節）。
const BUDGET_NAME: &str = "blocks-contact-split-form-image-budget";

/// 予算 [`fieldset::root`] の `id`。legend の id は headless
/// [`fieldset::legend`] の導出規則（`"{id}-legend"`）に一致させて
/// リテラルで直書きする（`format!` は使わない）。
const BUDGET_FIELDSET_ID: &str = "blocks-contact-split-form-image-budget";
const BUDGET_LEGEND_ID: &str = "blocks-contact-split-form-image-budget-legend";

/// 姓・名 2 欄の行（狭幅は 1 列、`48rem` 以上は 2 列。モジュール doc
/// 「ブレークポイント」節）。
fn name_row(orientation: &FieldRootProps) -> Node {
    let first_name = FieldProps {
        id: "blocks-contact-split-form-image-first-name",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let last_name = FieldProps {
        id: "blocks-contact-split-form-image-last-name",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    div(
        vec![("class", "blocks-contact-split-form-image-name-row")],
        vec![
            field::root(
                orientation,
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
                orientation,
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

/// メールアドレス欄（必須、補助テキストなし）。
fn email_field(orientation: &FieldRootProps) -> Node {
    let email = FieldProps {
        id: "blocks-contact-split-form-image-email",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        orientation,
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
    )
}

/// 電話番号欄（任意、補助テキスト付き。モジュール doc「id / ARIA の方針」
/// 節が述べる `has_helper_text` の実例）。
fn phone_field(orientation: &FieldRootProps) -> Node {
    let phone = FieldProps {
        id: "blocks-contact-split-form-image-phone",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: true,
    };
    field::root(
        orientation,
        &phone,
        vec![],
        vec![
            field::label(&phone, vec![], vec![text("電話番号")]),
            input::input(
                &InputProps::default(),
                &phone,
                vec![
                    ("type", "tel"),
                    ("placeholder", "090-1234-5678"),
                    ("autocomplete", "tel"),
                ],
            ),
            field::helper_text(
                &phone,
                vec![],
                vec![text("任意です。折り返しのご連絡先としてご利用します。")],
            ),
        ],
    )
}

/// お問い合わせ内容欄（必須、補助テキスト付き）。
fn message_field(orientation: &FieldRootProps) -> Node {
    let message = FieldProps {
        id: "blocks-contact-split-form-image-message",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: true,
    };
    field::root(
        orientation,
        &message,
        vec![],
        vec![
            field::label(&message, vec![], vec![text("お問い合わせ内容")]),
            textarea::textarea(
                &TextareaProps::default(),
                &message,
                false,
                vec![
                    ("rows", "5"),
                    ("placeholder", "ご質問・ご要望をご記入ください"),
                ],
                vec![],
            ),
            field::helper_text(&message, vec![], vec![text("目安は 400 文字程度です。")]),
        ],
    )
}

/// 予算選択肢 1 件（`radio_group::item` 3 パーツの組み立て、モジュール doc
/// 「id / ARIA の方針」節）。
fn budget_item(
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
            radio_group::item_hidden_input(checked, props, Some(BUDGET_NAME), value, vec![]),
            radio_group::item_control(checked, props, vec![]),
            radio_group::item_text(checked, props, vec![], vec![text(label)]),
        ],
    )
}

/// ご予算欄（fieldset + radio group、狭幅は縦積み・`48rem` 以上は 2×2。
/// 選択状態は先頭 1 件のみ固定した静的表示、モジュール doc「`<form>` を
/// 持たない」節）。ネイティブ操作で `checked` と視覚表示が食い違わない
/// よう `disabled: true` で固定する（モジュール doc「予算 radio group を
/// ネイティブ disabled にする理由」節）。
fn budget_fieldset() -> Node {
    let fieldset_props = FieldsetProps {
        id: BUDGET_FIELDSET_ID,
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
        vec![("data-blocks-contact-split-form-image-budget-fieldset", "")],
        vec![
            fieldset::legend(&fieldset_props, vec![], vec![text("ご予算")]),
            radio_group::root(
                Size::Md,
                ColorPalette::Accent,
                true,
                None,
                Some(BUDGET_LEGEND_ID),
                vec![("data-blocks-contact-split-form-image-budget-group", "")],
                vec![
                    budget_item(true, &radio_props, "under-10", "〜10 万円"),
                    budget_item(false, &radio_props, "10-to-50", "10 万〜50 万円"),
                    budget_item(false, &radio_props, "50-to-200", "50 万〜200 万円"),
                    budget_item(false, &radio_props, "over-200", "200 万円〜"),
                ],
            ),
        ],
    )
}

/// フォーム本体（`field::group` で縦積みにまとめる。モジュール doc「使用
/// 部品」節の 10 部品すべてがここへ現れる）。
fn form() -> Node {
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };
    div(
        vec![("class", "blocks-contact-split-form-image-form")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("お問い合わせ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "ご質問・ご相談をお送りください。担当者が折り返しご連絡します。",
                )],
            ),
            field::group(
                vec![],
                vec![
                    name_row(&orientation),
                    email_field(&orientation),
                    phone_field(&orientation),
                    message_field(&orientation),
                    budget_fieldset(),
                    separator::separator(&SeparatorProps::default(), vec![]),
                    div(
                        vec![("class", "blocks-contact-split-form-image-actions")],
                        vec![button::button(
                            &ButtonProps::default(),
                            vec![("data-blocks-contact-split-form-image-submit", "")],
                            vec![text("送信する")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// 画像領域（右カラム、`48rem` 以上でのみ表示・全高。モジュール doc
/// 「ブレークポイント」節）。
fn media() -> Node {
    div(
        vec![("class", "blocks-contact-split-form-image-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Auto,
                shape: ImageShape::Square,
                ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
            },
            vec![("data-blocks-contact-split-form-image-image", "")],
        )],
    )
}

/// `contact-split-form-image` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。DOM 順はフォーム → 画像（狭幅の 1 列表示でフォームが常に
/// 上に来るようにするため、画像は CSS で非表示にする）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-contact-split-form-image-layout")],
        vec![form(), media()],
    )
}
```

## 原案差分メモ

参照（主参照 R0065、集約元 R0859、出典の固有名・ファイル名は記載しません）
からの意図的な差分は次のとおりです。

- R0065 にはない補助テキスト付きの任意項目（電話番号）と、R0859 が持つ
  「予算」fieldset + radio group（4 択）の両方を 1 つの Demo へ合成しました。
- 見出しは参照側相当ではなく `h3`（ページ側が `## Demo` として `h2` を出す
  ため）にしています。
- 画像は R0859 の「右半分の全高」に合わせて `48rem` 以上でのみ表示し、
  `48rem` 未満では隠しています（`display: none`。装飾画像のため情報は
  失われません）。
- ご予算の選択状態は先頭 1 件のみを選択済みに固定した静的表示です。JS
  ハイドレーションを行わない docs サイトの制約に従い、選択の切り替えは
  扱いません。ネイティブ `<input type="radio">` はレビュー是正により
  `disabled` にしており、クリック・キーボード操作でも `checked` が
  変化しない（＝視覚表示と食い違わない）ようにしています。
- 送信ボタンは区切り線（separator）の下に右寄せで配置し、狭幅では全幅に
  なります。
- 文言（見出し・説明文・各欄のラベル・ヒント・予算の選択肢）はすべて
  独自に書き直しました。
- 画像はビルド時生成のプレースホルダー SVG（`alt=""`）で、実写風の合成は
  持ち込んでいません。
- 配色・余白・角丸は既存のテーマトークンに従っています。
- ブレークポイントは `48rem`（`md`）に固定しています。
- `<form>` は使わず、送信ボタンは `type="button"` のままです。

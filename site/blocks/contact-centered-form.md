# contact-centered-form

`heading` / `text` / `badge` / `field` / `input` / `textarea` / `checkbox` / `native-select` / `button` の 9 部品を合成した、中央寄せの問い合わせフォームレイアウトです。中央寄せのタグライン・見出し・説明文の下に、幅を絞った問い合わせフォーム（氏名・メール・会社名・電話番号・お問い合わせ内容 + 同意チェック + 送信ボタン）を配置します。

- 静的表示です。`<form>` は使わず、送信ボタンは `type="button"` のまま送信先・バリデーション・送信処理を持ちません。実際の送信処理は利用側の Rust/JS コードで実装します。
- 姓・名は 2 列グリッドに並び、会社名・メールアドレス・電話番号・お問い合わせ内容は全幅で 1 列を占めます。狭い画面（`< 48rem`）ではグリッド全体が 1 列に積み直されます。
- 電話番号は国番号（`native-select`）+ 番号本体（`input`）の横並びで表現し、任意項目です。会社名も任意項目、それ以外は必須項目です。
- 同意チェックボックスは SSR 初期状態（未チェック）を描画するのみです。docs サイトは JS ハイドレーションを行わないため、`disabled: true` を指定してネイティブ操作自体を不能にしています（操作を許すと `control`/`indicator` の見た目が同期しないまま状態だけ変化してしまうため）。表示と状態を連動させたい場合は利用者側の状態管理・ハイドレーションで実装してください。プライバシーポリシーへのリンクは遷移先を持たない死にリンクになるため出力せず、同意文言はテキストのみにしています。
- 文言・氏名・社名・メールアドレス・電話番号はすべて架空のものです。
- 集約元は 3 件です（対応表 ID R0066 を主参照、R0440・R0853 を集約元とする）。取り込んだのは領域配置・部品構成・状態の見せ方といった構造のみで、文言・配色・装飾は取り込んでいません。差分の詳細は末尾の「原案差分メモ」を参照してください。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 一意な id を組み立てる（`blocks-contact-centered-form-` 接頭辞を
/// 共通化し、フィールド追加時の綴り間違いを防ぐ）。
fn field_id(suffix: &str) -> String {
    format!("blocks-contact-centered-form-{suffix}")
}

/// 縦積み（label 上・control 下）の共通 orientation。
fn orientation() -> FieldRootProps {
    FieldRootProps {
        orientation: FieldOrientation::Vertical,
    }
}

/// 通常フィールド（`text`/`email` 等）を組み立てる。`wide` が `true` の
/// ときは全幅セル用フックを付与する（[`LAYOUT_CSS`] の
/// `[data-blocks-contact-centered-form-wide]` 参照）。
fn text_field(
    id: String,
    label_text: &'static str,
    input_type: &'static str,
    autocomplete: &'static str,
    placeholder: &'static str,
    required: bool,
    wide: bool,
) -> Node {
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required,
        readonly: false,
        has_helper_text: false,
    };
    let mut attrs = vec![("data-blocks-contact-centered-form-field", "")];
    if wide {
        attrs.push(("data-blocks-contact-centered-form-wide", ""));
    }
    field::root(
        &orientation(),
        &props,
        attrs,
        vec![
            field::label(&props, vec![], vec![text(label_text)]),
            input::input(
                &InputProps::default(),
                &props,
                vec![
                    ("type", input_type),
                    ("autocomplete", autocomplete),
                    ("placeholder", placeholder),
                ],
            ),
        ],
    )
}

/// 国番号付き電話番号フィールド（R0853 の差分。全幅・任意項目）。
fn phone_field() -> Node {
    let phone_id = field_id("phone");
    let country_id = field_id("country");
    let phone_props = FieldProps {
        id: phone_id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let country_props = FieldProps {
        id: country_id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let country_options = vec![
        el("option", vec![("value", "jp")], vec![text("+81 (JP)")]),
        el("option", vec![("value", "us")], vec![text("+1 (US)")]),
        el("option", vec![("value", "gb")], vec![text("+44 (GB)")]),
    ];

    field::root(
        &orientation(),
        &phone_props,
        vec![
            ("data-blocks-contact-centered-form-field", ""),
            ("data-blocks-contact-centered-form-wide", ""),
        ],
        vec![
            field::label(&phone_props, vec![], vec![text("電話番号（任意）")]),
            div(
                vec![("class", "blocks-contact-centered-form-phone")],
                vec![
                    native_select::native_select(
                        &NativeSelectProps::default(),
                        &country_props,
                        vec![("aria-label", "国番号")],
                        country_options,
                    ),
                    input::input(
                        &InputProps::default(),
                        &phone_props,
                        vec![
                            ("type", "tel"),
                            ("autocomplete", "tel"),
                            ("placeholder", "90-1234-5678"),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// お問い合わせ内容（全幅・必須の `textarea`）。
fn message_field() -> Node {
    let message_id = field_id("message");
    let props = FieldProps {
        id: message_id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &orientation(),
        &props,
        vec![
            ("data-blocks-contact-centered-form-field", ""),
            ("data-blocks-contact-centered-form-wide", ""),
        ],
        vec![
            field::label(&props, vec![], vec![text("お問い合わせ内容")]),
            textarea::textarea(
                &TextareaProps::default(),
                &props,
                false,
                vec![
                    ("placeholder", "ご相談内容をご記入ください。"),
                    ("data-blocks-contact-centered-form-message", ""),
                ],
                vec![],
            ),
        ],
    )
}

/// プライバシーポリシー同意チェック（R0440/R0853 の差分。SSR 初期状態
/// 〔未チェック〕を描画する、リンクは持たない。ネイティブ操作は
/// `disabled: true` で不能にする（モジュール doc「`<form>` を持たない・
/// 送信処理を持たない」節参照）。
fn consent_checkbox() -> Node {
    let props = CheckboxProps {
        required: true,
        disabled: true,
        ..CheckboxProps::default()
    };
    checkbox::root(
        Size::Md,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-contact-centered-form-consent", "")],
        vec![
            checkbox::hidden_input(&props, "contact-centered-form-consent", "on", vec![]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(
                &props,
                vec![],
                vec![text("プライバシーポリシーに同意します")],
            ),
        ],
    )
}

/// 中央寄せのタグライン + 見出し + 説明文（`header` 領域）。
fn header() -> Node {
    div(
        vec![("class", "blocks-contact-centered-form-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-contact-centered-form-tagline", "")],
                vec![text("お問い合わせ")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("ご相談・ご質問をお寄せください")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("2 営業日以内に担当者からご連絡します。")],
            ),
        ],
    )
}

/// `contact-centered-form` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let grid = div(
        vec![("class", "blocks-contact-centered-form-grid")],
        vec![
            text_field(
                field_id("first-name"),
                "姓",
                "text",
                "family-name",
                "山田",
                true,
                false,
            ),
            text_field(
                field_id("last-name"),
                "名",
                "text",
                "given-name",
                "太郎",
                true,
                false,
            ),
            text_field(
                field_id("company"),
                "会社名（任意）",
                "text",
                "organization",
                "株式会社サンプル",
                false,
                true,
            ),
            text_field(
                field_id("email"),
                "メールアドレス",
                "email",
                "email",
                "you@example.com",
                true,
                true,
            ),
            phone_field(),
            message_field(),
        ],
    );

    let form = div(
        vec![("class", "blocks-contact-centered-form-form")],
        vec![
            grid,
            consent_checkbox(),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-contact-centered-form-submit", "")],
                vec![text("送信する")],
            ),
        ],
    );

    div(
        vec![("class", "blocks-contact-centered-form-layout")],
        vec![header(), form],
    )
}
```

**原案差分メモ**

- 見出しレベルを 1 段下げました（ページ側の `## Demo` に合わせるため、セクション見出しは `h3`）。
- `<form>`/submit を持たず、送信ボタンは `type="button"` の静的表示にしました。
- R0066 の氏名・メール・本文（シンプルな中央寄せ構成）をベースに、R0440 の同意チェックと R0853 の会社名・国番号付き電話番号・項目数の多さを取り込みました。
- R0853 の同意トグルは、新しい UI 部品を追加しない制約のため `checkbox` へ置き換えました。
- 同意文言に付随するプライバシーポリシーへのリンクは出力していません（遷移先を持たない死にリンクを避けるため）。
- 国番号の `native-select` には可視ラベルを付けず、`aria-label="国番号"` でアクセシブル名を与えています（電話番号本体の `<label for>` と id が競合しないよう、国番号と電話番号は別々の id を持たせています）。
- 文言（氏名・社名・メールアドレス・お問い合わせ内容の例文）をすべて独自に書き直しました。
- 配色・余白・角丸は既存のテーマトークンに従っています。

# settings-profile-form

プロフィール設定フォームの 3 variant を並記した合成例です。`field` /
`fieldset` / `input` / `input-group` / `textarea` / `avatar` /
`file-upload` / `radio-card` / `switch` / `button` / `separator` の
11 部品を合成します。Blocks は既存部品の合成例であり、新しい UI 部品は
追加しません。

- variant A（ラベル横並び版）: コンテナ幅が十分なときラベルが左・入力欄が
  右へ並びます。
- variant B（代表構成、縦積み）: プロフィール欄に続けてパスワード変更・
  2 段階認証を設定できます。
- variant C（テーマ選択カード付き版）: 配色テーマを radio card から選べます。

氏名・メール・所在地・自己紹介・外部リンク・パスワードの文言はすべて架空の
もので、実在の人物・企業・PII は含みません。パスワード入力欄に `value` は
出力しません。写真アップロード（`file-upload`）・テーマ選択
（`radio-card`）・2 段階認証（`switch`）はいずれもネイティブ `disabled`
（file-upload の dropzone は `tabindex="-1"` + `aria-disabled="true"`）で
固定した静的表示であり、無 JS の docs サイトで操作可能に見えて実は無反応
という不整合を避けています。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。送信処理・
バリデーションは持たず、実際の実装は利用者自身の Rust/JS コードで行います。

## Rust コード

```rust
use crate::blocks::dummy_assets::PERSON_NAMES;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::file_upload::{self, FileUploadProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 一意な id を組み立てる（variant 接頭辞 + フィールド接尾辞、モジュール
/// doc「3 variant で `profile_fields` を共有する理由」節）。
fn field_id(variant: &str, suffix: &str) -> String {
    format!("blocks-settings-profile-form-{variant}-{suffix}")
}

/// 通常のテキスト系 field 1 件を組み立てる（縦積み固定、`wide` 指定は
/// 呼び出し側が [`LAYOUT_CSS`] のコンテナクエリで扱う）。
fn text_field(
    id: String,
    label_text: &'static str,
    input_type: &'static str,
    autocomplete: &'static str,
    placeholder: &'static str,
) -> Node {
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &props,
        vec![("data-blocks-settings-profile-form-field", "")],
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

/// 自己紹介欄（`textarea`、全幅）。
fn bio_field(variant: &str) -> Node {
    let id = field_id(variant, "bio");
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &props,
        vec![
            ("data-blocks-settings-profile-form-field", ""),
            ("data-blocks-settings-profile-form-wide", ""),
        ],
        vec![
            field::label(&props, vec![], vec![text("自己紹介")]),
            textarea::textarea(
                &TextareaProps::default(),
                &props,
                false,
                vec![
                    ("rows", "3"),
                    ("placeholder", "これまでの経歴をご記入ください。"),
                ],
                vec![],
            ),
        ],
    )
}

/// 外部リンク欄（`input-group` の `https://` 固定 addon + `input`、全幅）。
fn website_field(variant: &str) -> Node {
    let id = field_id(variant, "website");
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &props,
        vec![
            ("data-blocks-settings-profile-form-field", ""),
            ("data-blocks-settings-profile-form-wide", ""),
        ],
        vec![
            field::label(&props, vec![], vec![text("外部リンク")]),
            input_group::root(
                &InputGroupProps {
                    disabled: false,
                    invalid: false,
                },
                vec![],
                vec![
                    input_group::addon(
                        InputGroupAlign::InlineStart,
                        &InputGroupProps {
                            disabled: false,
                            invalid: false,
                        },
                        vec![],
                        vec![input_group::text(vec![], vec![text("https://")])],
                    ),
                    input::input(
                        &InputProps::default(),
                        &props,
                        vec![("type", "text"), ("placeholder", "example.com/you")],
                    ),
                ],
            ),
        ],
    )
}

/// 写真アップロード欄（`file_upload` + イニシャル `avatar` fallback、
/// モジュール doc「file-upload / radio-card / switch を無 JS で操作不能に
/// する理由」節）。
fn photo_upload_field() -> Node {
    let props = FileUploadProps {
        disabled: true,
        ..FileUploadProps::default()
    };
    let initials: String = PERSON_NAMES[0]
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    file_upload::root(
        Size::Md,
        &props,
        false,
        vec![
            ("data-blocks-settings-profile-form-field", ""),
            ("data-blocks-settings-profile-form-wide", ""),
        ],
        vec![
            file_upload::label(&props, vec![], vec![text("写真")]),
            div(
                vec![("class", "blocks-settings-profile-form-photo-row")],
                vec![
                    avatar::root(
                        &AvatarProps {
                            size: Size::Lg,
                            ..AvatarProps::default()
                        },
                        vec![],
                        vec![avatar::fallback(
                            ImageStatus::Error,
                            vec![],
                            vec![text(initials)],
                        )],
                    ),
                    file_upload::trigger(&props, vec![], vec![text("写真を変更")]),
                    file_upload::hidden_input("image/*", false, &props, vec![("hidden", "")]),
                ],
            ),
        ],
    )
}

/// プロフィール入力欄一式（写真・氏名・メール・所在地・自己紹介・外部
/// リンク）。3 variant が共有する（モジュール doc「3 variant で
/// `profile_fields` を共有する理由」節）。
fn profile_fields(variant: &str) -> Vec<Node> {
    vec![
        photo_upload_field(),
        text_field(
            field_id(variant, "name"),
            "氏名",
            "text",
            "name",
            "山田 太郎",
        ),
        text_field(
            field_id(variant, "email"),
            "メールアドレス",
            "email",
            "email",
            "you@example.com",
        ),
        text_field(
            field_id(variant, "location"),
            "所在地",
            "text",
            "address-level2",
            "東京都渋谷区",
        ),
        bio_field(variant),
        website_field(variant),
    ]
}

/// パスワード変更欄 1 件（`type="password"`、`value` は出さない）。
fn password_field(id: String, label_text: &'static str, autocomplete: &'static str) -> Node {
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &props,
        vec![("data-blocks-settings-profile-form-field", "")],
        vec![
            field::label(&props, vec![], vec![text(label_text)]),
            input::input(
                &InputProps::default(),
                &props,
                vec![("type", "password"), ("autocomplete", autocomplete)],
            ),
        ],
    )
}

/// パスワード変更 `fieldset`（現在のパスワード・新しいパスワード・確認の
/// 3 欄）。variant B の代表構成に含む。
fn password_fieldset() -> Node {
    let id = field_id("stacked", "password-fieldset");
    let props = FieldsetProps {
        id: id.as_str(),
        disabled: false,
        invalid: false,
        has_helper_text: false,
    };
    fieldset::root(
        &FieldsetRootProps::default(),
        &props,
        vec![("data-blocks-settings-profile-form-fieldset", "")],
        vec![
            fieldset::legend(&props, vec![], vec![text("パスワード変更")]),
            password_field(
                field_id("stacked", "current-password"),
                "現在のパスワード",
                "current-password",
            ),
            password_field(
                field_id("stacked", "new-password"),
                "新しいパスワード",
                "new-password",
            ),
            password_field(
                field_id("stacked", "confirm-password"),
                "新しいパスワード（確認）",
                "new-password",
            ),
        ],
    )
}

/// 2 段階認証 `switch` 1 件（無 JS のためネイティブ `disabled` で固定、
/// モジュール doc「file-upload / radio-card / switch を無 JS で操作不能に
/// する理由」節）。
fn two_factor_switch(name: &'static str, label_text: &'static str, checked: bool) -> Node {
    let props = SwitchProps {
        disabled: true,
        ..SwitchProps::default()
    };
    switch::root(
        Size::Md,
        ColorPalette::Accent,
        checked,
        &props,
        vec![("data-blocks-settings-profile-form-switch", "")],
        vec![
            switch::label(checked, &props, vec![], vec![text(label_text)]),
            switch::hidden_input(name, "on", checked, &props, vec![]),
            switch::control(
                checked,
                &props,
                vec![],
                vec![switch::thumb(checked, &props, vec![], vec![])],
            ),
        ],
    )
}

/// 2 段階認証 `fieldset`（認証アプリ・SMS の 2 switch）。variant B の
/// 代表構成に含む。
fn two_factor_fieldset() -> Node {
    let id = field_id("stacked", "two-factor-fieldset");
    let props = FieldsetProps {
        id: id.as_str(),
        disabled: false,
        invalid: false,
        has_helper_text: false,
    };
    fieldset::root(
        &FieldsetRootProps::default(),
        &props,
        vec![("data-blocks-settings-profile-form-fieldset", "")],
        vec![
            fieldset::legend(&props, vec![], vec![text("2 段階認証")]),
            div(
                vec![("class", "blocks-settings-profile-form-switch-list")],
                vec![
                    two_factor_switch("blocks-settings-profile-form-2fa-app", "認証アプリ", true),
                    two_factor_switch("blocks-settings-profile-form-2fa-sms", "SMS", false),
                ],
            ),
        ],
    )
}

/// テーマ選択 `radio-card` の 1 item（ネイティブ disabled、モジュール doc
/// 「file-upload / radio-card / switch を無 JS で操作不能にする理由」節）。
fn theme_item(checked: bool, value: &'static str, label: &'static str) -> Node {
    radio_card::item(
        checked,
        true,
        value,
        vec![],
        vec![
            radio_card::item_hidden_input(
                checked,
                true,
                Some("blocks-settings-profile-form-theme"),
                value,
                vec![],
            ),
            radio_card::item_control(
                checked,
                true,
                vec![],
                vec![
                    radio_card::item_indicator(checked, true, false, vec![]),
                    radio_card::item_content(
                        vec![],
                        vec![radio_card::item_text(vec![], vec![text(label)])],
                    ),
                ],
            ),
        ],
    )
}

/// テーマ選択欄（見出し + radio card 3 択）。variant C にのみ追加する。
fn theme_field() -> Node {
    let label_id = field_id("theme", "label");
    div(
        vec![
            ("class", "blocks-settings-profile-form-theme"),
            ("data-blocks-settings-profile-form-wide", ""),
        ],
        vec![
            radio_card::label(Some(label_id.as_str()), vec![], vec![text("テーマ")]),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                Some(Orientation::Horizontal),
                Some(label_id.as_str()),
                vec![("aria-disabled", "true")],
                vec![
                    theme_item(false, "light", "ライト"),
                    theme_item(true, "dark", "ダーク"),
                    theme_item(false, "system", "システムに合わせる"),
                ],
            ),
        ],
    )
}

/// フッター操作（保存のみ、または キャンセル + 保存）。
fn footer_actions(with_cancel: bool) -> Node {
    let mut children = Vec::new();
    if with_cancel {
        children.push(button::button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("キャンセル")],
        ));
    }
    children.push(button::button(
        &ButtonProps::default(),
        vec![],
        vec![text("保存")],
    ));
    div(
        vec![("class", "blocks-settings-profile-form-actions")],
        children,
    )
}

/// variant 1 件を見出し + 説明文 + フィールド群 + フッターでまとめる
/// 共通ヘルパ（モジュール doc「見出しレベル（`H3`）」節）。
fn variant_section(
    kind: &'static str,
    title: &'static str,
    description: &'static str,
    fields: Vec<Node>,
    footer: Node,
) -> Node {
    div(
        vec![
            ("class", "blocks-settings-profile-form-variant"),
            ("data-blocks-settings-profile-form-variant", kind),
        ],
        vec![
            el(
                "h3",
                vec![("class", "blocks-settings-profile-form-variant-title")],
                vec![text(title)],
            ),
            el(
                "p",
                vec![("class", "blocks-settings-profile-form-variant-description")],
                vec![text(description)],
            ),
            div(vec![("class", "blocks-settings-profile-form-grid")], fields),
            footer,
        ],
    )
}

/// variant A（横並びラベル最小版）。
fn inline_variant() -> Node {
    variant_section(
        "inline",
        "ラベル横並び版",
        "コンテナ幅が十分なときはラベルが左・入力欄が右に並びます。",
        profile_fields("inline"),
        footer_actions(false),
    )
}

/// variant B（代表構成: パスワード変更・2 段階認証を続ける縦積み版）。
fn stacked_variant() -> Node {
    let mut fields = profile_fields("stacked");
    fields.push(password_fieldset());
    fields.push(two_factor_fieldset());
    variant_section(
        "stacked",
        "代表構成（縦積み）",
        "プロフィールに続けてパスワード変更・2 段階認証を設定できます。",
        fields,
        footer_actions(true),
    )
}

/// variant C（テーマ選択カードを含む版）。
fn theme_variant() -> Node {
    let mut fields = profile_fields("theme");
    fields.push(theme_field());
    variant_section(
        "theme",
        "テーマ選択カード付き版",
        "配色テーマをカードから選べます。",
        fields,
        footer_actions(false),
    )
}

/// `settings-profile-form` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。3 variant を `separator` で区切って縦に並べる。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-profile-form-layout")],
        vec![
            inline_variant(),
            separator(&SeparatorProps::default(), vec![]),
            stacked_variant(),
            separator(&SeparatorProps::default(), vec![]),
            theme_variant(),
        ],
    )
}
```

## 原案差分メモ

- イシュー本文のレイアウト仕様（横並びラベル版・パスワード変更/2 段階認証
  続き版・テーマ選択カード版の 3 構成）のみから組み立てました。参照ファイル
  置き場 `_/blocks-intake/` はこの worktree に存在せず、構成の実物（R0262〜
  R0264）は未参照です。
- 3 variant はすべて共通の `profile_fields` ヘルパ（写真・氏名・メール・
  所在地・自己紹介・外部リンク）を共有し、variant B のみパスワード変更・
  2 段階認証の `fieldset` を、variant C のみテーマ選択 `radio-card` を
  追加しています。
- id は variant ごとに接頭辞を分けて一意化しています（重複 id・宙ぶらりん
  aria 参照の回避）。
- 見出しは `heading` 部品を使わず素の `h3` 要素にしています（Issue 指定の
  11 部品に `heading` が含まれないため）。

関連情報: [Field](../themes/field.md) / [Fieldset](../themes/fieldset.md) /
[Input](../themes/input.md) / [Input Group](../themes/input-group.md) /
[Textarea](../themes/textarea.md) / [Avatar](../themes/avatar.md) /
[File Upload](../themes/file-upload.md) /
[Radio Card](../themes/radio-card.md) / [Switch](../themes/switch.md) /
[Button](../themes/button.md) / [Separator](../themes/separator.md)

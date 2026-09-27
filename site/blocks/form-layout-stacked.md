# form-layout-stacked

`heading` / `field` / `fieldset` / `input` / `textarea` / `native-select` / `checkbox` / `radio-group` / `file-upload` / `avatar` / `button` の 11 部品を合成した、縦積みの設定フォームレイアウトです。プロフィール・個人情報・通知の 3 セクションを縦に積み、各セクションの入力欄はラベルが上・コントロールが下の縦積み配置で並びます。

- 静的表示です。`<form>` は使わず、ボタンは `type="button"` のまま送信先・バリデーション・送信処理を持ちません。実際の送信処理は利用側の Rust/JS コードで実装します。
- **プロフィール**セクション: ユーザー名・自己紹介に加え、写真アップロード欄（`file-upload` + イニシャルの `avatar` fallback）とカバー画像アップロード欄（`file-upload` の dropzone）を持ちます。
- **個人情報**セクション: 姓・名・メールアドレス・国と地域・住所・市区町村・郵便番号を持ちます。姓名・市区町村と郵便番号は広い幅（`32rem` 以上）で 2 列に並びます。
- **通知**セクション: メール通知のチェックボックス群（3 件）と、プッシュ通知のラジオボタン群（3 件、`fieldset` + `radio-group`）を持ちます。
- チェックボックス・ラジオボタンはいずれも初期状態を固定した静的表示です。docs サイトは JS ハイドレーションを行わないため、`disabled: true` を指定してネイティブ操作自体を不能にしています（操作を許すと見た目の状態と実際の値が食い違うため）。
- ファイル選択欄（写真・カバー画像）はいずれも無 JS では動作しない静的表示です。`<input type="file">` はすべて `hidden` にしており、ネイティブのファイル選択 UI は表示されません。
- 末尾にキャンセル・保存の 2 ボタンを配置しています。
- 文言・氏名・ユーザー名・メールアドレス・住所はすべて架空のものです。
- 集約元は 1 件です（対応表 ID R0968 を主参照とし、集約すべき別 variant はありません）。取り込んだのは領域配置・部品構成・状態の見せ方といった構造のみで、文言・配色・装飾は取り込んでいません。差分の詳細は末尾の「原案差分メモ」を参照してください。

## Rust コード

```rust
use crate::blocks::dummy_assets::PERSON_NAMES;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::file_upload::{self, FileUploadProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::radio_group::{self, RadioGroupProps};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 一意な id を組み立てる（`blocks-form-layout-stacked-` 接頭辞を共通化し、
/// フィールド追加時の綴り間違いを防ぐ）。
fn field_id(suffix: &str) -> String {
    format!("blocks-form-layout-stacked-{suffix}")
}

/// 縦積み（label 上・control 下）の共通 orientation。
fn orientation() -> FieldRootProps {
    FieldRootProps {
        orientation: FieldOrientation::Vertical,
    }
}

/// 通常フィールド（`has_helper_text: false`）を組み立てる。`wide` が
/// `true` のときは全幅セル用フックを付与する（[`LAYOUT_CSS`] の
/// `[data-blocks-form-layout-stacked-wide]` 参照）。
fn text_field(
    id: String,
    label_text: &'static str,
    input_type: &'static str,
    autocomplete: &'static str,
    placeholder: &'static str,
    wide: bool,
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
    let mut attrs = vec![("data-blocks-form-layout-stacked-field", "")];
    if wide {
        attrs.push(("data-blocks-form-layout-stacked-wide", ""));
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

/// 自己紹介欄（`textarea` + `helper_text`、全幅）。
fn bio_field() -> Node {
    let id = field_id("bio");
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: true,
    };
    field::root(
        &orientation(),
        &props,
        vec![
            ("data-blocks-form-layout-stacked-field", ""),
            ("data-blocks-form-layout-stacked-wide", ""),
        ],
        vec![
            field::label(&props, vec![], vec![text("自己紹介")]),
            textarea::textarea(
                &TextareaProps::default(),
                &props,
                false,
                vec![(
                    "placeholder",
                    "これまでの経歴や興味のある分野をご記入ください。",
                )],
                vec![],
            ),
            field::helper_text(&props, vec![], vec![text("プロフィールに公開されます。")]),
        ],
    )
}

/// 写真アップロード欄（`file_upload` + イニシャル `avatar` fallback）。
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
            ("data-blocks-form-layout-stacked-field", ""),
            ("data-blocks-form-layout-stacked-wide", ""),
        ],
        vec![
            file_upload::label(&props, vec![], vec![text("写真")]),
            div(
                vec![("class", "blocks-form-layout-stacked-photo-row")],
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

/// カバー画像アップロード欄（`file_upload` の `dropzone`、全幅）。
fn cover_image_upload_field() -> Node {
    let props = FileUploadProps {
        disabled: true,
        ..FileUploadProps::default()
    };
    file_upload::root(
        Size::Md,
        &props,
        false,
        vec![
            ("data-blocks-form-layout-stacked-field", ""),
            ("data-blocks-form-layout-stacked-wide", ""),
        ],
        vec![
            file_upload::label(&props, vec![], vec![text("カバー画像")]),
            file_upload::dropzone(
                &props,
                false,
                vec![("aria-label", "カバー画像をアップロード")],
                vec![
                    el(
                        "p",
                        vec![("class", "blocks-form-layout-stacked-dropzone-hint")],
                        vec![text("ここにファイルをドラッグ、または")],
                    ),
                    file_upload::trigger(&props, vec![], vec![text("ファイルを選択")]),
                    file_upload::hidden_input(
                        "image/png,image/jpeg",
                        false,
                        &props,
                        vec![("hidden", "")],
                    ),
                ],
            ),
            el(
                "p",
                vec![("class", "blocks-form-layout-stacked-dropzone-note")],
                vec![text("PNG・JPEG、最大 5MB。")],
            ),
        ],
    )
}

/// プロフィールセクション。
fn profile_section() -> Node {
    let username_id = field_id("username");
    let username_props = FieldProps {
        id: username_id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let username_field = field::root(
        &orientation(),
        &username_props,
        vec![
            ("data-blocks-form-layout-stacked-field", ""),
            ("data-blocks-form-layout-stacked-wide", ""),
        ],
        vec![
            field::label(&username_props, vec![], vec![text("ユーザー名")]),
            input::input(
                &InputProps::default(),
                &username_props,
                vec![
                    ("type", "text"),
                    ("autocomplete", "username"),
                    ("value", "haruto.fujimaki"),
                ],
            ),
        ],
    );

    section(
        "プロフィール",
        "公開プロフィールに表示される基本情報です。",
        vec![
            username_field,
            bio_field(),
            photo_upload_field(),
            cover_image_upload_field(),
        ],
    )
}

/// 個人情報セクション。
fn personal_info_section() -> Node {
    let name_row = div(
        vec![("class", "blocks-form-layout-stacked-name-row")],
        vec![
            text_field(
                field_id("first-name"),
                "姓",
                "text",
                "family-name",
                "山田",
                false,
            ),
            text_field(
                field_id("last-name"),
                "名",
                "text",
                "given-name",
                "太郎",
                false,
            ),
        ],
    );

    let country_id = field_id("country");
    let country_props = FieldProps {
        id: country_id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let country_field = field::root(
        &orientation(),
        &country_props,
        vec![
            ("data-blocks-form-layout-stacked-field", ""),
            ("data-blocks-form-layout-stacked-wide", ""),
        ],
        vec![
            field::label(&country_props, vec![], vec![text("国・地域")]),
            native_select::native_select(
                &NativeSelectProps::default(),
                &country_props,
                vec![],
                vec![
                    el(
                        "option",
                        vec![("value", "jp"), ("selected", "")],
                        vec![text("日本")],
                    ),
                    el(
                        "option",
                        vec![("value", "us")],
                        vec![text("アメリカ合衆国")],
                    ),
                    el("option", vec![("value", "gb")], vec![text("イギリス")]),
                ],
            ),
        ],
    );

    let city_postal_row = div(
        vec![("class", "blocks-form-layout-stacked-name-row")],
        vec![
            text_field(
                field_id("city"),
                "市区町村",
                "text",
                "address-level2",
                "渋谷区",
                false,
            ),
            text_field(
                field_id("postal-code"),
                "郵便番号",
                "text",
                "postal-code",
                "150-0001",
                false,
            ),
        ],
    );

    section(
        "個人情報",
        "請求書・配送先の宛先に使用します。",
        vec![
            name_row,
            text_field(
                field_id("email"),
                "メールアドレス",
                "email",
                "email",
                "you@example.com",
                true,
            ),
            country_field,
            text_field(
                field_id("address"),
                "住所",
                "text",
                "street-address",
                "神南 1-2-3",
                true,
            ),
            city_postal_row,
        ],
    )
}

/// メール通知チェックボックス 1 件を組み立てる（無 JS のためネイティブ
/// `disabled` で固定、モジュール doc「checkbox / radio group をネイティブ
/// disabled にする理由」節）。
fn email_notification_checkbox(
    name: &'static str,
    label_text: &'static str,
    checked: bool,
) -> Node {
    let props = CheckboxProps {
        checked: if checked {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
        ..CheckboxProps::default()
    };
    let value = "on";
    checkbox::root(
        Size::Md,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-form-layout-stacked-checkbox", "")],
        vec![
            checkbox::hidden_input(&props, name, value, vec![]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(&props, vec![], vec![text(label_text)]),
        ],
    )
}

/// プッシュ通知 radio group の 1 item を組み立てる。
fn push_notification_item(
    checked: bool,
    props: &RadioGroupProps,
    value: &'static str,
    label: &'static str,
) -> Node {
    radio_group::item(
        checked,
        props,
        value,
        vec![("data-blocks-form-layout-stacked-radio-item", "")],
        vec![
            radio_group::item_hidden_input(
                checked,
                props,
                Some("blocks-form-layout-stacked-push-notifications"),
                value,
                vec![],
            ),
            radio_group::item_control(checked, props, vec![]),
            radio_group::item_text(checked, props, vec![], vec![text(label)]),
        ],
    )
}

/// 通知セクション（メール通知の checkbox 群 + プッシュ通知の radio group）。
fn notifications_section() -> Node {
    let email_fieldset_id = field_id("email-notifications");
    let email_fieldset_props = FieldsetProps {
        id: email_fieldset_id.as_str(),
        disabled: false,
        invalid: false,
        has_helper_text: true,
    };
    let email_fieldset = fieldset::root(
        &FieldsetRootProps::default(),
        &email_fieldset_props,
        vec![
            ("data-blocks-form-layout-stacked-fieldset", ""),
            ("data-blocks-form-layout-stacked-wide", ""),
        ],
        vec![
            fieldset::legend(&email_fieldset_props, vec![], vec![text("メール通知")]),
            fieldset::helper_text(
                &email_fieldset_props,
                vec![],
                vec![text("重要な更新のみお知らせを受け取れます。")],
            ),
            div(
                vec![("class", "blocks-form-layout-stacked-checkbox-list")],
                vec![
                    email_notification_checkbox(
                        "blocks-form-layout-stacked-notify-updates",
                        "製品アップデート",
                        true,
                    ),
                    email_notification_checkbox(
                        "blocks-form-layout-stacked-notify-tips",
                        "使い方のヒント",
                        false,
                    ),
                    email_notification_checkbox(
                        "blocks-form-layout-stacked-notify-newsletter",
                        "ニュースレター",
                        false,
                    ),
                ],
            ),
        ],
    );

    let push_fieldset_id = field_id("push-notifications");
    let push_legend_id = format!("{push_fieldset_id}-legend");
    let push_fieldset_props = FieldsetProps {
        id: push_fieldset_id.as_str(),
        disabled: false,
        invalid: false,
        has_helper_text: false,
    };
    let push_radio_props = RadioGroupProps {
        disabled: true,
        ..RadioGroupProps::default()
    };
    let push_fieldset = fieldset::root(
        &FieldsetRootProps::default(),
        &push_fieldset_props,
        vec![
            ("data-blocks-form-layout-stacked-fieldset", ""),
            ("data-blocks-form-layout-stacked-wide", ""),
        ],
        vec![
            fieldset::legend(&push_fieldset_props, vec![], vec![text("プッシュ通知")]),
            radio_group::root(
                Size::Md,
                ColorPalette::Accent,
                true,
                None,
                Some(push_legend_id.as_str()),
                vec![("data-blocks-form-layout-stacked-radio-group", "")],
                vec![
                    push_notification_item(true, &push_radio_props, "all", "すべて通知する"),
                    push_notification_item(false, &push_radio_props, "mentions", "メンションのみ"),
                    push_notification_item(false, &push_radio_props, "none", "通知しない"),
                ],
            ),
        ],
    );

    section(
        "通知",
        "受け取る通知の種類を設定します。",
        vec![email_fieldset, push_fieldset],
    )
}

/// フッター操作（キャンセル・保存、右寄せ）。
fn footer_actions() -> Node {
    div(
        vec![("class", "blocks-form-layout-stacked-actions")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("キャンセル")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("保存")]),
        ],
    )
}

/// 1 セクション（見出し + 説明文 + 入力欄グリッド）を組み立てる共通ヘルパ。
fn section(title: &'static str, description: &'static str, fields: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-form-layout-stacked-section")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Lg,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text(title)],
            ),
            el(
                "p",
                vec![("class", "blocks-form-layout-stacked-description")],
                vec![text(description)],
            ),
            div(vec![("class", "blocks-form-layout-stacked-grid")], fields),
        ],
    )
}

/// `form-layout-stacked` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-form-layout-stacked-layout")],
        vec![
            // セクション 3 件を専用ラッパへまとめる（`LAYOUT_CSS` の
            // `.blocks-form-layout-stacked-section:last-child` が
            // footer_actions（同じく `div` の兄弟）を挟まず、実際に最後の
            // セクションだけへボーダー除去を適用できるようにするため。
            // ラップせず `:last-of-type` を使うと、タグ名一致の
            // `:last-of-type` は class を見ないため footer_actions の
            // `div` が「最後の div」になり、どのセクションにも一致しない
            // 不具合があった）。
            div(
                vec![("class", "blocks-form-layout-stacked-sections")],
                vec![
                    profile_section(),
                    personal_info_section(),
                    notifications_section(),
                ],
            ),
            footer_actions(),
        ],
    )
}
```

## 原案差分メモ

- 見出しレベルを 1 段下げました（ページ側の `## Demo` に合わせるため、セクション見出しは `h3`）。
- `<form>`/submit を持たず、ボタンはすべて `type="button"` の静的表示にしました。
- R0968 の 3 セクション構成（プロフィール・個人情報・通知）をそのまま踏襲し、各入力欄は縦積み（ラベル上・コントロール下）で配置しました。
- チェックボックス・ラジオボタンは操作しても状態が変化しない静的表示にするため、ネイティブ `disabled` で固定しました（見た目と実際の値の食い違いを避けるため）。
- ファイルアップロード欄はアップロード済みファイルの一覧を出していません（初期状態でファイルが存在しないため）。トリガー・ドロップゾーンは無 JS では何も起こさない静的表示です。
- 写真アップロード欄のアバターは画像アセットへ依存させず、イニシャルのみの fallback 表示にしています。
- 文言（ユーザー名・氏名・メールアドレス・住所などの例文）をすべて独自に書き直しました。
- 配色・余白・角丸は既存のテーマトークンに従っています。

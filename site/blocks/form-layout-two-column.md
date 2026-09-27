# form-layout-two-column

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `field` / `fieldset` /
`input` / `textarea` / `native-select` / `checkbox` / `radio-group` / `card` /
`button` 部品を合成した、見出し列＋入力列の 2 カラムフォームの実例です。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0969、集約元は R0970。出典の固有名・ファイル名は記載しません）。

各セクションを左列（見出しと説明文）・右列（入力欄）に分け、コンテナクエリ
（`min-width: 48rem`）で 2 カラム grid へ切り替える構成です。右列をカードに
入れず、パネル末尾でキャンセル・保存の 2 ボタンを共有する代表構成
（variant A「プロフィール」「個人情報」「通知」の 3 セクション）と、右列を
カードに入れ、セクションごとに独立した保存ボタンを持つ版（variant B
「アカウント」「セキュリティ」の 2 セクション）の 2 例を並べています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。ボタンはすべて `type="button"` のまま送信先を
持たず、狭い幅（`48rem` 未満）では見出しが入力欄の上に積まれます。通知の
checkbox・radio group はネイティブ `disabled` で操作を禁止した静的な初期
状態のみを描きます。文言はすべて独自に書いた架空のものであり、実企業名・
実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::radio_group::{self, RadioGroupProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// メール通知 [`fieldset::root`] の `id`。legend id は headless
/// [`fieldset::legend`] の導出規則（`"{id}-legend"`）に一致させて
/// リテラルで直書きする（モジュール doc「id / ARIA の方針」節）。
const EMAIL_NOTIF_FIELDSET_ID: &str = "blocks-form-layout-two-column-a-notif-email";

/// プッシュ通知 [`fieldset::root`] の `id` と legend id。
const PUSH_NOTIF_FIELDSET_ID: &str = "blocks-form-layout-two-column-a-notif-push";
const PUSH_NOTIF_LEGEND_ID: &str = "blocks-form-layout-two-column-a-notif-push-legend";

/// プッシュ通知 radio group のネイティブ `<input>` の共通 `name`。
const PUSH_NOTIF_RADIO_NAME: &str = "blocks-form-layout-two-column-a-notif-push";

/// 一意な id を組み立てる（`blocks-form-layout-two-column-` 接頭辞を
/// 共通化し、フィールド追加時の綴り間違いを防ぐ。`contact_centered_form` と
/// 同型のヘルパ）。
fn field_id(suffix: &str) -> String {
    format!("blocks-form-layout-two-column-{suffix}")
}

/// 縦積み（見出し・label 上、コントロール下）の共通 orientation。
fn vertical() -> FieldRootProps {
    FieldRootProps {
        orientation: FieldOrientation::Vertical,
    }
}

/// 通常フィールドの [`FieldProps`] を組み立てる小さなヘルパ。
fn simple_field(id: &str, required: bool, has_helper_text: bool) -> FieldProps<'_> {
    FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required,
        readonly: false,
        has_helper_text,
    }
}

/// セクション 1 件（左列: 見出し + 説明文、右列: 任意の内容ノード群）。
/// コンテナクエリで `48rem` 以上のときだけ 2 カラム grid へ切り替わる
/// （モジュール doc「コンテナクエリで 2 カラム化する理由」節）。
fn section(aside_title: &'static str, aside_desc: &'static str, right: Node) -> Node {
    div(
        vec![("class", "blocks-form-layout-two-column-section")],
        vec![
            div(
                vec![("class", "blocks-form-layout-two-column-aside")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Lg,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text(aside_title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(aside_desc)],
                    ),
                ],
            ),
            right,
        ],
    )
}

/// 右列の入力欄群を縦に並べる素の `div`（カードに入れない variant A 用）。
fn fields_column(fields: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-form-layout-two-column-fields")],
        fields,
    )
}

/// 「プロフィール」セクション（ユーザー名 + 自己紹介、variant A）。
fn profile_section() -> Node {
    let username_id = field_id("a-username");
    let bio_id = field_id("a-bio");
    let username = simple_field(&username_id, true, false);
    let bio = simple_field(&bio_id, false, true);
    section(
        "プロフィール",
        "他のユーザーに公開される基本情報です。",
        fields_column(vec![
            field::root(
                &vertical(),
                &username,
                vec![],
                vec![
                    field::label(&username, vec![], vec![text("ユーザー名")]),
                    input::input(
                        &InputProps::default(),
                        &username,
                        vec![("type", "text"), ("placeholder", "yamada_taro")],
                    ),
                ],
            ),
            field::root(
                &vertical(),
                &bio,
                vec![],
                vec![
                    field::label(&bio, vec![], vec![text("自己紹介")]),
                    textarea::textarea(
                        &TextareaProps::default(),
                        &bio,
                        false,
                        vec![
                            ("rows", "3"),
                            ("placeholder", "趣味や担当業務を書いてください"),
                        ],
                        vec![],
                    ),
                    field::helper_text(&bio, vec![], vec![text("200 文字程度でご記入ください。")]),
                ],
            ),
        ]),
    )
}

/// 「個人情報」セクション（氏名・メール・国/地域、variant A）。
fn personal_section() -> Node {
    let name_id = field_id("a-name");
    let email_id = field_id("a-email");
    let country_id = field_id("a-country");
    let name = simple_field(&name_id, true, false);
    let email = simple_field(&email_id, true, false);
    let country = simple_field(&country_id, false, false);
    section(
        "個人情報",
        "サポートからのご連絡に使用します。",
        fields_column(vec![
            field::root(
                &vertical(),
                &name,
                vec![],
                vec![
                    field::label(&name, vec![], vec![text("氏名")]),
                    input::input(
                        &InputProps::default(),
                        &name,
                        vec![("type", "text"), ("placeholder", "山田 太郎")],
                    ),
                ],
            ),
            field::root(
                &vertical(),
                &email,
                vec![],
                vec![
                    field::label(&email, vec![], vec![text("メールアドレス")]),
                    input::input(
                        &InputProps::default(),
                        &email,
                        vec![("type", "email"), ("placeholder", "you@example.com")],
                    ),
                ],
            ),
            field::root(
                &vertical(),
                &country,
                vec![],
                vec![
                    field::label(&country, vec![], vec![text("国・地域")]),
                    native_select::native_select(
                        &NativeSelectProps::default(),
                        &country,
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
                            el("option", vec![("value", "other")], vec![text("その他")]),
                        ],
                    ),
                ],
            ),
        ]),
    )
}

/// email 通知チェックボックス 1 件（モジュール doc「checkbox / radio group
/// をネイティブ disabled にする理由」節）。
fn notif_checkbox(props: &CheckboxProps, name: &'static str, label_text: &'static str) -> Node {
    checkbox::root(
        Size::Md,
        ColorPalette::Accent,
        props,
        vec![],
        vec![
            checkbox::hidden_input(props, name, "on", vec![]),
            checkbox::control(
                props,
                vec![],
                vec![checkbox::indicator(props, vec![], vec![])],
            ),
            checkbox::label(props, vec![], vec![text(label_text)]),
        ],
    )
}

/// push 通知 radio group の選択肢 1 件。
fn notif_radio(
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
            radio_group::item_hidden_input(
                checked,
                props,
                Some(PUSH_NOTIF_RADIO_NAME),
                value,
                vec![],
            ),
            radio_group::item_control(checked, props, vec![]),
            radio_group::item_text(checked, props, vec![], vec![text(label)]),
        ],
    )
}

/// 「通知」セクション（メール通知 checkbox 2 件 + プッシュ通知 radio group
/// 3 件、variant A）。両方とも `disabled: true` で操作を禁止する。
fn notifications_section() -> Node {
    let news = CheckboxProps {
        checked: CheckedState::Checked,
        disabled: true,
        ..CheckboxProps::default()
    };
    let product_updates = CheckboxProps {
        disabled: true,
        ..CheckboxProps::default()
    };
    let radio_props = RadioGroupProps {
        disabled: true,
        ..RadioGroupProps::default()
    };
    let email_fieldset_props = FieldsetProps {
        id: EMAIL_NOTIF_FIELDSET_ID,
        disabled: false,
        invalid: false,
        has_helper_text: false,
    };
    let push_fieldset_props = FieldsetProps {
        id: PUSH_NOTIF_FIELDSET_ID,
        disabled: false,
        invalid: false,
        has_helper_text: false,
    };
    section(
        "通知",
        "受け取る通知の種類を選べます。",
        fields_column(vec![
            fieldset::root(
                &FieldsetRootProps::default(),
                &email_fieldset_props,
                vec![],
                vec![
                    fieldset::legend(&email_fieldset_props, vec![], vec![text("メール通知")]),
                    div(
                        vec![("class", "blocks-form-layout-two-column-checkbox-group")],
                        vec![
                            notif_checkbox(
                                &news,
                                "blocks-form-layout-two-column-a-notif-news",
                                "お知らせ",
                            ),
                            notif_checkbox(
                                &product_updates,
                                "blocks-form-layout-two-column-a-notif-product",
                                "製品アップデート",
                            ),
                        ],
                    ),
                ],
            ),
            fieldset::root(
                &FieldsetRootProps::default(),
                &push_fieldset_props,
                vec![],
                vec![
                    fieldset::legend(&push_fieldset_props, vec![], vec![text("プッシュ通知")]),
                    radio_group::root(
                        Size::Md,
                        ColorPalette::Accent,
                        true,
                        None,
                        Some(PUSH_NOTIF_LEGEND_ID),
                        vec![],
                        vec![
                            notif_radio(true, &radio_props, "all", "すべて受け取る"),
                            notif_radio(false, &radio_props, "mentions", "メンションのみ"),
                            notif_radio(false, &radio_props, "none", "受け取らない"),
                        ],
                    ),
                ],
            ),
        ]),
    )
}

/// variant A（主参照 R0969。右列をカードに入れず、パネル末尾の共通操作行
/// でキャンセル・保存を行う代表構成）。
fn panel_single_save() -> Node {
    div(
        vec![
            ("class", "blocks-form-layout-two-column-panel"),
            ("data-blocks-form-layout-two-column-variant", "single-save"),
        ],
        vec![
            profile_section(),
            personal_section(),
            notifications_section(),
            div(
                vec![("class", "blocks-form-layout-two-column-actions")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("キャンセル")],
                    ),
                    button::button(&ButtonProps::default(), vec![], vec![text("変更を保存")]),
                ],
            ),
        ],
    )
}

/// カードに入れた右列 1 件（variant B。`card::footer` に保存ボタンを 1 個
/// 持たせ、セクションごとに独立させる、モジュール doc「variant A /
/// variant B の 2 例を並べる理由」節）。
fn card_fields(fields: Vec<Node>, save_label: &'static str) -> Node {
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![],
        vec![
            card::body(
                vec![("class", "blocks-form-layout-two-column-fields")],
                fields,
            ),
            card::footer(
                vec![],
                vec![button::button(
                    &ButtonProps::default(),
                    vec![],
                    vec![text(save_label)],
                )],
            ),
        ],
    )
}

/// 「アカウント」セクション（表示名 + 使用言語、variant B）。
fn account_section() -> Node {
    let name_id = field_id("b-display-name");
    let language_id = field_id("b-language");
    let name = simple_field(&name_id, true, false);
    let language = simple_field(&language_id, false, false);
    section(
        "アカウント",
        "表示名と使用言語を設定します。",
        card_fields(
            vec![
                field::root(
                    &vertical(),
                    &name,
                    vec![],
                    vec![
                        field::label(&name, vec![], vec![text("表示名")]),
                        input::input(
                            &InputProps::default(),
                            &name,
                            vec![("type", "text"), ("placeholder", "山田 太郎")],
                        ),
                    ],
                ),
                field::root(
                    &vertical(),
                    &language,
                    vec![],
                    vec![
                        field::label(&language, vec![], vec![text("使用言語")]),
                        native_select::native_select(
                            &NativeSelectProps::default(),
                            &language,
                            vec![],
                            vec![
                                el(
                                    "option",
                                    vec![("value", "ja"), ("selected", "")],
                                    vec![text("日本語")],
                                ),
                                el("option", vec![("value", "en")], vec![text("English")]),
                            ],
                        ),
                    ],
                ),
            ],
            "アカウント設定を保存",
        ),
    )
}

/// 「セキュリティ」セクション（復旧用メール + メモ、variant B）。
fn security_section() -> Node {
    let recovery_email_id = field_id("b-recovery-email");
    let notes_id = field_id("b-notes");
    let recovery_email = simple_field(&recovery_email_id, false, true);
    let notes = simple_field(&notes_id, false, false);
    section(
        "セキュリティ",
        "アカウント復旧に使う連絡先です。",
        card_fields(
            vec![
                field::root(
                    &vertical(),
                    &recovery_email,
                    vec![],
                    vec![
                        field::label(&recovery_email, vec![], vec![text("復旧用メールアドレス")]),
                        input::input(
                            &InputProps::default(),
                            &recovery_email,
                            vec![("type", "email"), ("placeholder", "backup@example.com")],
                        ),
                        field::helper_text(
                            &recovery_email,
                            vec![],
                            vec![text("主連絡先が使えないときの連絡先です。")],
                        ),
                    ],
                ),
                field::root(
                    &vertical(),
                    &notes,
                    vec![],
                    vec![
                        field::label(&notes, vec![], vec![text("メモ")]),
                        textarea::textarea(
                            &TextareaProps::default(),
                            &notes,
                            false,
                            vec![("rows", "3"), ("placeholder", "社内向けの引き継ぎ事項など")],
                            vec![],
                        ),
                    ],
                ),
            ],
            "セキュリティ設定を保存",
        ),
    )
}

/// variant B（集約元 R0970。右列をカードに入れ、セクションごとに独立した
/// 保存ボタンを持つ版）。
fn panel_section_save() -> Node {
    div(
        vec![
            ("class", "blocks-form-layout-two-column-panel"),
            ("data-blocks-form-layout-two-column-variant", "section-save"),
        ],
        vec![account_section(), security_section()],
    )
}

/// `form-layout-two-column` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。variant A/B を並べて表示する（モジュール doc「variant A /
/// variant B の 2 例を並べる理由」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-form-layout-two-column-layout")],
        vec![panel_single_save(), panel_section_save()],
    )
}
```

## 原案差分メモ

- 対応表 ID R0969（代表構成）と R0970（集約元）の差分を variant A/B として
  1 つの Demo 内に並置しています。R0970 の要素（右列をカードに入れ、
  セクションごとに独立した保存ボタンを持つ）を差分として示す構成です。
- 2 カラム化はビューポート幅ではなく、各パネルの `@container` 幅
  （`min-width: 48rem`）で判定します。`fandhe-frontend-wasm-full` の
  JS 配線を持たない docs サイトの制約上、狭幅表示の目視確認はブラウザの
  ウィンドウ幅を変える必要があります。
- checkbox・radio group は `disabled: true` で固定した静的な初期状態です。
  ラベルクリックやキーボード操作をしても状態は変化しません（JS
  ハイドレーションを行わない docs サイトで、見た目と実際の状態が
  食い違うことを避けるための構造的な禁止です）。
- Issue 本文の部品一覧に `text`（styled）を加えています。左列の説明文
  表現に必要なため追加した差分です。
- 参照元の文言・配色・アイコンは使用せず、独自に書いています。実在の
  人物・企業名・決済情報等は含みません。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Field](../themes/field.md) / [Fieldset](../themes/fieldset.md) /
[Input](../themes/input.md) / [Textarea](../themes/textarea.md) /
[Native Select](../themes/native-select.md) / [Checkbox](../themes/checkbox.md) /
[Radio Group](../themes/radio-group.md) / [Card](../themes/card.md) /
[Button](../themes/button.md)

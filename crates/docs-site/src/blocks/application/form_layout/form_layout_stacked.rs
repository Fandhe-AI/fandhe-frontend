//! `form-layout-stacked` block（イシュー #2915。親トラッキング #2892「Blocks
//! 目的別パーツ拡充ツリー」Phase 3・Application A 配下、主参照 R0968 を
//! 代表構成とする合成例。統合すべき別 variant は無い）。プロフィール・
//! 個人情報・通知の 3 セクションを縦に積んだ設定フォームで、各セクションの
//! 入力欄はラベルが上・コントロールが下の縦積み配置を取る。取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じ転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `field` / `fieldset` / `input` / `textarea` / `native-select` /
//! `checkbox` / `radio-group` / `file-upload` / `avatar` / `button` の
//! 11 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # `<form>` を持たない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。ボタンは
//! [`fandhe_frontend_pre_styled_ui::button::button`] の既定 `type="button"`
//! のまま送信先・バリデーションを持たず、実際の送信処理は利用者自身の
//! Rust/JS コードで実装する（`docs/policy/intentional-non-adoption.md`
//! §3.25）。文言はすべて架空のもの（実在の人物・企業・PII を含まない、
//! `crate::blocks::dummy_assets::PERSON_NAMES` を使用）。
//!
//! # checkbox / radio group をネイティブ disabled にする理由
//!
//! `checkbox::hidden_input`/`radio_group::item_hidden_input` は有効な
//! ネイティブ `<input>` であり、`disabled` を渡さない構成では docs サイトが
//! JS ハイドレーションを行わなくてもラベルクリック・キーボード操作で
//! ブラウザが `checked` をネイティブに切り替えてしまう。一方 `control`/
//! `indicator`/`item_control` の見た目（`data-state`）は SSR 時の `checked`
//! 引数から固定生成されるため追従せず、静的な初期状態のみという block
//! 全体の設計方針に反する（`contact_centered_form`/`contact_split_form_image`
//! と同型の判断）。[`CheckboxProps`]/[`RadioGroupProps`] の `disabled: true`
//! をすべてのパーツへ共有し、ネイティブ `disabled` 属性でフォーカス・操作を
//! 不能にして状態が二度と変化しないことを構造的に保証する。
//! `disabled_declarations()`（既定 `opacity: 0.5` + `cursor: not-allowed`）は
//! [`LAYOUT_CSS`] で中和し、通常の checkbox/radio と同じ見た目に保つ。
//!
//! # file-upload の hidden input に `hidden` を付ける理由
//!
//! `file_upload::hidden_input` はネイティブ `<input type="file">` であり、
//! 通常はクリック不可視のまま `trigger`/`dropzone` から `click()` 転送する
//! JS 配線（`fandhe-frontend-wasm-full`）を前提とする。docs サイトは JS
//! ハイドレーションを行わないため、`hidden` 存在属性を明示して要素自体を
//! 不可視・操作不能にし、無 JS でもネイティブファイル選択 UI が露出しない
//! ようにする（`hidden` は headless 側の予約キーではないため `attrs` から
//! 渡せる、`crates/headless-ui/src/file_upload.rs::HIDDEN_INPUT_RESERVED`
//! 参照）。トリガー・ドロップゾーンは無 JS では何も起こさない静的表示の
//! ままにする。アップロード済みファイルの一覧（`item_group`/`item`）は
//! 初期状態に存在しないため出力しない。
//!
//! # avatar を fallback のみにする理由
//!
//! 画像アセットへ依存させず、イニシャルの `avatar::fallback(ImageStatus::
//! Error, …)` のみを使う（`feed_upvote_cards` と同型の判断）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading`/`field`/`fieldset`/`input`/`textarea`/`native_select`/
//! `checkbox`/`radio_group`/`file_upload`/`avatar`/`button` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-form-layout-stacked-*` 属性で渡す。素の `div` には `class`
//! がそのまま効くため `.blocks-form-layout-stacked-*` クラスセレクタを使う。
//!
//! # 見出しレベル（`H3`）
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! [`HeadingLevel::H3`] にする（既存 block と同型の判断）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
    let props = FileUploadProps::default();
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
    let props = FileUploadProps::default();
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
        vec![("data-blocks-form-layout-stacked-field", "")],
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
        vec![("data-blocks-form-layout-stacked-field", "")],
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
        vec![],
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
        vec![("data-blocks-form-layout-stacked-fieldset", "")],
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
        vec![("data-blocks-form-layout-stacked-fieldset", "")],
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
            profile_section(),
            personal_info_section(),
            notifications_section(),
            footer_actions(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/form-layout-stacked/",
    title: "form-layout-stacked",
    category: BlockCategory::FormLayout,
    rust_source: "crates/docs-site/src/blocks/application/form_layout/form_layout_stacked.rs",
    demo_class: "blocks-form-layout-stacked",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Fieldset",
            path: "/themes/fieldset/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Textarea",
            path: "/themes/textarea/",
        },
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Radio Group",
            path: "/themes/radio-group/",
        },
        Part {
            label: "File Upload",
            path: "/themes/file-upload/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `form_layout_stacked` 固有のレイアウト規則（`crate::blocks` モジュール
/// doc「CSS の置き場」節）。セレクタは `.blocks-form-layout-stacked-*` と
/// `[data-blocks-form-layout-stacked-*]`、および styled `field`/`checkbox`/
/// `radio-group`/`file-upload` の `[data-scope=...]` 系セレクタへの上書き
/// のみを用い、他 block や部品の素のセレクタへ影響させない。
const LAYOUT_CSS: &str = "\
.blocks-form-layout-stacked-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  width: 100%;\n  max-width: 42rem;\n  margin-inline: auto;\n}\n\
.blocks-form-layout-stacked-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  padding-block-end: var(--fandhe-space-8);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-form-layout-stacked-section:last-of-type {\n  padding-block-end: 0;\n  border-bottom: none;\n}\n\
.blocks-form-layout-stacked-description {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-form-layout-stacked-grid {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n  container-type: inline-size;\n  container-name: blocks-form-layout-stacked;\n}\n\
.blocks-form-layout-stacked-name-row {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"field\"][data-part=\"root\"][data-blocks-form-layout-stacked-wide] {\n  grid-column: 1 / -1;\n}\n\
[data-scope=\"file-upload\"][data-part=\"root\"][data-blocks-form-layout-stacked-wide] {\n  grid-column: 1 / -1;\n}\n\
@container blocks-form-layout-stacked (min-width: 32rem) {\n  \
.blocks-form-layout-stacked-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
.blocks-form-layout-stacked-name-row {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
.blocks-form-layout-stacked-photo-row {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-form-layout-stacked-dropzone-hint {\n  margin: 0;\n}\n\
.blocks-form-layout-stacked-dropzone-note {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-form-layout-stacked-checkbox-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-form-layout-stacked-checkbox][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"radio-group\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-form-layout-stacked-actions {\n  display: flex;\n  justify-content: flex-end;\n  gap: var(--fandhe-space-2);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, BLOCK, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 11 種の部品を含むことを固定する（`crates/docs-site/
    /// tests/blocks_contract.rs` の横断検査と重複し過ぎない範囲での個別
    /// 固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"field\"",
            "data-scope=\"fieldset\"",
            "data-scope=\"checkbox\"",
            "data-scope=\"radio-group\"",
            "data-scope=\"file-upload\"",
            "data-scope=\"avatar\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"data-part="input""#));
        assert!(html.contains(r#"data-part="textarea""#));
        assert!(html.contains(r#"data-part="select""#));
    }

    /// `<form>` を出力しない・送信先を持たない静的表示で、`type="button"`
    /// がちょうど 4 個（キャンセル・保存・写真変更・カバー画像選択）で
    /// あること。
    #[test]
    fn demo_has_exactly_four_type_buttons_and_no_form() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), 4);
        assert!(!html.contains("<form"));
        assert!(!html.contains(r#"type="submit""#));
        assert!(!html.contains("action="));
        assert!(!html.contains("src=\"data:"));
    }

    /// 各テキスト系 field の `<label for>` が対応するコントロールの `id`
    /// と一致すること。
    #[test]
    fn text_field_labels_point_at_their_controls() {
        let html = render(&demo());
        for suffix in [
            "username",
            "bio",
            "first-name",
            "last-name",
            "email",
            "country",
            "address",
            "city",
            "postal-code",
        ] {
            let control_id = format!("blocks-form-layout-stacked-{suffix}-control");
            assert!(
                html.contains(&format!(r#"for="{control_id}""#)),
                "expected a label pointing at {control_id} in {html}"
            );
            assert!(
                html.contains(&format!(r#"id="{control_id}""#)),
                "expected control id {control_id} in {html}"
            );
        }
    }

    /// checkbox・radio の hidden input がすべてネイティブ `disabled` で
    /// あり、checked の初期状態がそれぞれちょうど 1 件であること。
    #[test]
    fn checkbox_and_radio_are_natively_disabled_with_single_checked_default() {
        let html = render(&demo());
        // 3 checkbox + 3 radio = 6 個の disabled hidden input。
        assert_eq!(html.matches(" disabled=\"\"").count(), 6);
        // ネイティブ `checked` 存在属性は checkbox 1 件 + radio 1 件の
        // 計 2 個のみ（`data-state="checked"` は root/control/indicator/
        // label 等の複数パーツへ伝播するため個数の固定には使わない）。
        assert_eq!(html.matches(" checked=\"\"").count(), 2);
    }

    /// `<input type="file">` がすべて `hidden` 属性を持つこと（写真・
    /// カバー画像の 2 個）。
    #[test]
    fn file_inputs_are_all_hidden() {
        let html = render(&demo());
        let file_input_tags: Vec<&str> = html
            .split('<')
            .filter(|tag| tag.starts_with("input") && tag.contains(r#"type="file""#))
            .collect();
        assert_eq!(file_input_tags.len(), 2);
        for tag in file_input_tags {
            assert!(
                tag.contains("hidden"),
                "expected hidden attribute on file input tag: {tag}"
            );
        }
    }

    /// [`LAYOUT_CSS`] がコンテナクエリ・disabled 中和規則を含み、`<` を
    /// 含まないこと。
    #[test]
    fn layout_css_declares_container_query_and_disabled_neutralization() {
        assert!(LAYOUT_CSS.contains("container-name: blocks-form-layout-stacked;"));
        assert!(LAYOUT_CSS.contains("@container blocks-form-layout-stacked (min-width: 32rem)"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-form-layout-stacked-checkbox][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-form-layout-stacked-layout\""));
        assert_ne!(BLOCK.demo_class, "blocks-form-layout-stacked-layout");
    }
}

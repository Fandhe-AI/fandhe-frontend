//! `settings-profile-form` block（イシュー #3010。Application/Settings
//! カテゴリ 11 件目）。プロフィール設定フォームの 3 variant（A: ラベル横
//! 並び・B: パスワード変更 + 2 段階認証を続ける縦積み代表構成・C: テーマ
//! 選択カードを含む構成）を 1 ページに並記する合成例。
//!
//! # 使用部品
//!
//! `field` / `fieldset` / `input` / `input-group` / `textarea` / `avatar` /
//! `file-upload` / `radio-card` / `switch` / `button` / `separator` の
//! 11 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 参照について
//!
//! Issue 本文のレイアウト仕様（横並びラベル版・パスワード変更/2 段階認証
//! 続き版・テーマ選択カード版の 3 構成）のみから組み立てた（参照ファイル
//! 置き場 `_/blocks-intake/` はローカル worktree に存在せず実物は未参照）。
//! 文言・配色・装飾は独自に書く（他 block と同じ転記制限）。
//!
//! # `<form>` を持たない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり静的な合成例。ボタンは
//! [`fandhe_frontend_pre_styled_ui::button::button`] の既定 `type="button"`
//! のまま送信先・バリデーションを持たない（`docs/policy/
//! intentional-non-adoption.md` §3.25）。氏名・メール・パスワード等の文言
//! はすべて架空（`crate::blocks::dummy_assets::PERSON_NAMES`）で、パスワード
//! 入力に `value` は出さない。
//!
//! # file-upload / radio-card / switch を無 JS で操作不能にする理由
//!
//! docs サイトは JS ハイドレーションを行わないため、`file_upload::
//! trigger`/`radio_card::item_hidden_input`/`switch::hidden_input` のような
//! 有効なネイティブ操作系要素をそのまま出すと「操作可能に見えるが無反応」
//! という不整合が生じる（`form_layout_stacked`/`card_form_footer`/
//! `pricing_seats_split` と同型の判断）。本 block は 3 パーツすべてへ
//! `disabled: true` を共有してネイティブ `disabled`（file-upload の
//! `dropzone` は `tabindex="-1"` + `aria-disabled="true"`）で操作を構造的に
//! 禁止し、`radio-card`/`switch` の disabled 見た目（`opacity: 0.5`）のみ
//! [`LAYOUT_CSS`] で中和する（file-upload の disabled 見た目は
//! `form_layout_stacked` と同じく中和しない）。
//!
//! # 3 variant で `profile_fields` を共有する理由
//!
//! variant A（横並びラベル）・B（縦積み代表構成）・C（テーマ選択カード
//! 追加）はいずれも同じプロフィール入力欄（写真・氏名・メール・所在地・
//! 自己紹介・外部リンク）を持つ。`profile_fields(variant, inline)` へ
//! 共通化し、`inline` は A のみ `true` を渡して [`LAYOUT_CSS`] のコンテナ
//! クエリでラベル位置を切り替える（B/C は `FieldOrientation::Vertical` の
//! まま）。id は `field_id(variant, suffix)` で variant ごとに接頭辞を分け、
//! `blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_
//! duplicate_ids` の重複 id 検知に通す。
//!
//! # `input-group` を外部リンク欄にのみ使う理由
//!
//! Issue 指定 11 部品に `input-group` が含まれるため、外部リンク欄
//! （`https://` 固定プレフィックスを `input_group::addon`/`text` で表示す
//! る）へ充てる（`hero_email_signup` と同型の合成契約）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `field`/`fieldset`/`input`/`input-group`/`textarea`/`avatar`/
//! `file-upload`/`radio-card`/`switch`/`button`/`separator` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のフックは `data-blocks-settings-profile-form-*`
//! 属性で渡す。素の `div`/`h3`/`p` には `class` がそのまま効くため
//! `.blocks-settings-profile-form-*` クラスセレクタを使う。
//!
//! # 見出しレベル（`H3`）
//!
//! ページ側が `## Demo` として `h2` を出すため、variant 見出しは素の
//! `h3` 要素にする（`heading` 部品は Issue 指定 11 部品に含まれないため
//! 使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-profile-form/",
    title: "settings-profile-form",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_profile_form.rs",
    demo_class: "blocks-settings-profile-form",
    parts: &[
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
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Textarea",
            path: "/themes/textarea/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "File Upload",
            path: "/themes/file-upload/",
        },
        Part {
            label: "Radio Card",
            path: "/themes/radio-card/",
        },
        Part {
            label: "Switch",
            path: "/themes/switch/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_profile_form` 固有のレイアウト規則（`crate::blocks` モジュール
/// doc「CSS の置き場」節）。セレクタは `.blocks-settings-profile-form-*` と
/// `[data-blocks-settings-profile-form-*]`、および styled `field`/
/// `fieldset`/`file-upload`/`radio-card`/`switch` の `[data-scope=...]` 系
/// セレクタへの上書きのみを用いる。
const LAYOUT_CSS: &str = "\
.blocks-settings-profile-form-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  width: 100%;\n  max-width: 42rem;\n  margin-inline: auto;\n}\n\
.blocks-settings-profile-form-variant {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  container-type: inline-size;\n  container-name: blocks-settings-profile-form;\n}\n\
.blocks-settings-profile-form-variant-title {\n  margin: 0;\n}\n\
.blocks-settings-profile-form-variant-description {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-profile-form-grid {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"field\"][data-part=\"root\"][data-blocks-settings-profile-form-wide] {\n  grid-column: 1 / -1;\n}\n\
[data-scope=\"file-upload\"][data-part=\"root\"][data-blocks-settings-profile-form-wide] {\n  grid-column: 1 / -1;\n}\n\
.blocks-settings-profile-form-theme[data-blocks-settings-profile-form-wide] {\n  grid-column: 1 / -1;\n}\n\
@container blocks-settings-profile-form (min-width: 36rem) {\n  \
[data-blocks-settings-profile-form-variant=\"inline\"] [data-scope=\"field\"][data-part=\"root\"] {\n    display: grid;\n    grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);\n    align-items: start;\n    gap: var(--fandhe-space-4);\n  }\n\
}\n\
.blocks-settings-profile-form-photo-row {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-profile-form-switch-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"switch\"][data-part=\"root\"][data-blocks-settings-profile-form-switch][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"radio-card\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-settings-profile-form-actions {\n  display: flex;\n  justify-content: flex-end;\n  gap: var(--fandhe-space-2);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, BLOCK, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// Demo が期待する 11 種の部品を含むことを固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"field\"",
            "data-scope=\"fieldset\"",
            "data-scope=\"input-group\"",
            "data-scope=\"avatar\"",
            "data-scope=\"file-upload\"",
            "data-scope=\"radio-card\"",
            "data-scope=\"switch\"",
            "data-scope=\"button\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"data-part="input""#));
        assert!(html.contains(r#"data-part="textarea""#));
    }

    /// `<form>` を出力しない・送信先を持たない静的表示であること。
    #[test]
    fn demo_has_no_form_or_submit() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains(r#"type="submit""#));
        assert!(!html.contains("action="));
        assert!(!html.contains("src=\"data:"));
    }

    /// ボタン系要素はすべて `type="button"`（キャンセル 1 個 + 保存 3 個 +
    /// file-upload trigger 3 個 = 7 個）。
    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        assert_eq!(html.matches(r#"type="button""#).count(), 7);
    }

    /// `<input type="file">` はすべて `hidden` 属性付き（3 variant で 3 個）。
    #[test]
    fn file_inputs_are_all_hidden() {
        let html = demo_html();
        let file_input_tags: Vec<&str> = html
            .split('<')
            .filter(|tag| tag.starts_with("input") && tag.contains(r#"type="file""#))
            .collect();
        assert_eq!(file_input_tags.len(), 3);
        for tag in file_input_tags {
            assert!(tag.contains("hidden"), "expected hidden attribute: {tag}");
        }
    }

    /// パスワード入力欄に `value` が出力されないこと（3 個: 現在・新規・
    /// 確認）。
    #[test]
    fn password_inputs_have_no_value() {
        let html = demo_html();
        assert_eq!(html.matches(r#"type="password""#).count(), 3);
        assert!(!html.contains(r#"type="password" value"#));
    }

    /// file-upload の trigger がネイティブ disabled であること（3 variant で
    /// 3 個）。
    #[test]
    fn file_upload_triggers_are_not_operable() {
        let html = demo_html();
        assert_eq!(html.matches(r#"data-part="trigger""#).count(), 3);
    }

    /// radio-card のテーマ選択は 1 件のみ checked（variant C の "dark"）で
    /// あること。
    #[test]
    fn theme_radio_card_has_single_checked_default() {
        let html = demo_html();
        assert_eq!(html.matches(r#"type="radio""#).count(), 3);
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    /// [`LAYOUT_CSS`] がコンテナクエリ・disabled 中和規則を含み、`<` を
    /// 含まないこと。
    #[test]
    fn layout_css_declares_container_query_and_disabled_neutralization() {
        assert!(LAYOUT_CSS.contains("container-name: blocks-settings-profile-form;"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-profile-form (min-width: 36rem)"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"radio-card\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-settings-profile-form-layout\""));
        assert_ne!(BLOCK.demo_class, "blocks-settings-profile-form-layout");
    }
}

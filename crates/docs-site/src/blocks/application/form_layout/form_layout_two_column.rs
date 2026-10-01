//! `form-layout-two-column` block（イシュー #2916。Application/Form Layout
//! カテゴリ、親トラッキング #2892「Blocks 目的別パーツ拡充」
//! 配下）。各セクションを左列（見出し + 説明文）・右列（入力欄）に分けた
//! 2 カラムフォームの合成例。対応表 ID R0969（代表構成）と R0970（右列を
//! カードに入れ、セクションごとに独立した保存ボタンを持つ版）を集約する。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`card-form-footer`〔イシュー #2899〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `field` / `fieldset` / `input` / `textarea` /
//! `native-select` / `checkbox` / `radio-group` / `card` / `button` の
//! 11 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! Issue 本文の部品一覧に `text`（styled）を加えている差分は、左列の
//! 説明文の表現に必要なためである（`site/blocks/form-layout-two-column.md`
//! 「原案差分メモ」節参照）。新しい UI 部品は追加しない。
//!
//! # variant A / variant B の 2 例を並べる理由
//!
//! 主参照 R0969（[`panel_single_save`]、右列をカードに入れず 3 セクション +
//! パネル末尾の共通操作行）に対し、集約元 R0970（[`panel_section_save`]、
//! 右列を [`card::root`] に入れセクションごとに独立した保存ボタンを持つ版）
//! を差分として並置する。`card-form-footer`/`pricing-single-split` と同型の
//! 判断で、1 つの Demo へ詰め込むより 2 例を並べる方が違いを一目で読み取れる。
//!
//! # コンテナクエリで 2 カラム化する理由
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@media` ではなく
//! `description-list-horizontal`/`form-layout-inline-labels` と同型の
//! `@container` を使う。各パネル（`.blocks-form-layout-two-column-panel`）へ
//! `container-type: inline-size; container-name:
//! blocks-form-layout-two-column;` を宣言し、`(min-width: 48rem)` のときだけ
//! セクションを `grid-template-columns: minmax(0, 1fr) minmax(0, 2fr)` の
//! 2 カラムへ切り替える。既定（狭幅）は縦積みで、見出しが入力欄の上に来る。
//!
//! # checkbox / radio group をネイティブ disabled にする理由
//!
//! [`checkbox::hidden_input`]/[`radio_group::item_hidden_input`] は有効な
//! ネイティブ `<input>` であり、`disabled` を渡さない構成では docs サイトが
//! JS ハイドレーションを行わなくてもラベルクリック・キーボード操作で
//! ブラウザが `checked` をネイティブに切り替えてしまう。一方見た目
//! （`data-state="checked"`）は SSR 時の引数から固定生成されるため追従
//! せず、選択・送信される値と支援技術が認識する状態・視覚表示が食い違う
//! （`contact_split_form_image`/`contact_centered_form` と同じ問題）。本
//! Demo は静的な初期状態のみを示す（`docs/policy/intentional-non-adoption.md`
//! §3.25、UI コンポーネント層はアプリケーションロジックを内包しない）ため、
//! [`CheckboxProps`]/[`RadioGroupProps`] の `disabled: true` を全パーツへ
//! 共有し、ネイティブ `disabled` 属性で操作を構造的に禁止する。
//! `disabled_declarations()`（既定 `opacity: 0.5` + `cursor: not-allowed`）は
//! [`LAYOUT_CSS`] で中和し、通常の checkbox/radio group と同じ見た目に保つ。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading`/`text`/`field::root`/`fieldset::root`/`input`/`textarea`/
//! `native_select`/`checkbox::root`/`radio_group::root`/`card::root`/
//! `button` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-form-layout-two-column-*` 属性で渡す。素の `div` には
//! `class` がそのまま効くため、レイアウトは
//! `.blocks-form-layout-two-column-*` クラスセレクタを使う。checkbox/
//! radio group の disabled 中和は既存 block と同型に、レイアウト用の
//! クラス祖先 + `[data-scope=...][data-part=...][data-disabled]`
//! （詳細度 0,4,0 以上）で base 宣言（詳細度 0,2,0）を上書きする。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と `fandhe_frontend_core::text`
//! が同名のため、styled 側を `styled_text` として取り込む（既存 block と
//! 同じ回避方法）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、各セクション見出しは
//! `HeadingLevel::H3` を使う（既存 block と同じ判断）。
//!
//! # id / ARIA の方針
//!
//! 通常フィールドの id は `field_id` ヘルパ（`format!` で
//! `blocks-form-layout-two-column-{suffix}` を組み立てる、
//! `contact_centered_form` と同型）で組み立てる。fieldset の id と legend
//! id（`fieldset::legend` の導出規則 `"{id}-legend"` に一致させる値）・
//! radio group の `name` はリテラル定数で固定する（`contact_split_form_image`
//! と同じ静的な突合の判断）。variant ごとに id 接頭辞を分け
//! （`-a-*`/`-b-*`）、2 インスタンス間の id 衝突を避ける。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは `button::button` の既定 `type="button"` のまま
//! 用い、送信処理・送信先は一切持たない。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R0969、集約元は R0970。文言・配色・アイコンは独自に
//! 書く（他 block と同じライセンス上の転記制限）。実在の人物・企業名・PII
//! は使わない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/form-layout-two-column/",
    title: "form-layout-two-column",
    category: BlockCategory::FormLayout,
    rust_source: "crates/docs-site/src/blocks/application/form_layout/form_layout_two_column.rs",
    demo_class: "blocks-form-layout-two-column",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
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
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `form_layout_two_column` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-form-layout-two-column-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-form-layout-two-column-panel {\n  display: flex;\n  flex-direction: column;\n  container-type: inline-size;\n  container-name: blocks-form-layout-two-column;\n}\n\
.blocks-form-layout-two-column-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  padding-block: var(--fandhe-space-6);\n}\n\
.blocks-form-layout-two-column-section + .blocks-form-layout-two-column-section {\n  border-block-start: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-form-layout-two-column-aside {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-form-layout-two-column-fields {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-form-layout-two-column-checkbox-group {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-form-layout-two-column-actions {\n  display: flex;\n  justify-content: flex-end;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-form-layout-two-column-fields [data-scope=\"checkbox\"][data-part=\"root\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-form-layout-two-column-fields [data-scope=\"radio-group\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@container blocks-form-layout-two-column (min-width: 48rem) {\n  \
.blocks-form-layout-two-column-section {\n    display: grid;\n    grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);\n    gap: var(--fandhe-space-8);\n    align-items: start;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{
        demo, EMAIL_NOTIF_FIELDSET_ID, LAYOUT_CSS, PUSH_NOTIF_FIELDSET_ID, PUSH_NOTIF_LEGEND_ID,
    };
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"field\"",
            "data-scope=\"fieldset\"",
            "data-scope=\"checkbox\"",
            "data-scope=\"radio-group\"",
            "data-scope=\"card\"",
            "data-scope=\"button\"",
            "data-scope=\"text\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains("data-part=\"input\""));
        assert!(html.contains("data-part=\"textarea\""));
        assert!(html.contains("<select"));
    }

    #[test]
    fn no_form_semantics_or_dead_links() {
        let html = demo_html();
        assert_eq!(html.matches(r#"type="button""#).count(), 4);
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("action="));
        assert!(!html.contains("href="));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn every_field_label_resolves_to_its_control() {
        let html = demo_html();
        for control_suffix in [
            "a-username",
            "a-bio",
            "a-name",
            "a-email",
            "a-country",
            "b-display-name",
            "b-language",
            "b-recovery-email",
            "b-notes",
        ] {
            let id = format!("id=\"blocks-form-layout-two-column-{control_suffix}-control\"");
            assert!(html.contains(&id), "missing control id: {id}");
        }
    }

    #[test]
    fn fieldset_legend_matches_radio_group_labelled_by() {
        let html = demo_html();
        assert!(html.contains(&format!(
            "id=\"{}\"",
            EMAIL_NOTIF_FIELDSET_ID.to_owned() + "-legend"
        )));
        assert!(html.contains(&format!("id=\"{PUSH_NOTIF_LEGEND_ID}\"")));
        assert!(html.contains(&format!("aria-labelledby=\"{PUSH_NOTIF_LEGEND_ID}\"")));
        assert_eq!(
            format!("{PUSH_NOTIF_FIELDSET_ID}-legend"),
            PUSH_NOTIF_LEGEND_ID
        );
    }

    #[test]
    fn checkbox_and_radio_hidden_inputs_are_natively_disabled() {
        let html = demo_html();
        assert_eq!(html.matches(r#"type="checkbox""#).count(), 2);
        assert_eq!(html.matches(r#"type="radio""#).count(), 3);
        // checkbox 2 件 + radio 3 件が全て disabled（モジュール doc
        // 「checkbox / radio group をネイティブ disabled にする理由」節）。
        assert_eq!(html.matches(" disabled=\"\"").count(), 5);
        assert_eq!(html.matches(" checked").count(), 2); // 通知 checkbox 1 件 + radio 1 件
    }

    #[test]
    fn both_variants_are_present() {
        let html = demo_html();
        assert!(html.contains(r#"data-blocks-form-layout-two-column-variant="single-save""#));
        assert!(html.contains(r#"data-blocks-form-layout-two-column-variant="section-save""#));
    }

    #[test]
    fn variant_b_has_two_cards_with_footer_save_buttons() {
        let html = demo_html();
        assert_eq!(html.matches("data-part=\"footer\"").count(), 2);
        assert!(html.contains("アカウント設定を保存"));
        assert!(html.contains("セキュリティ設定を保存"));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("container-name: blocks-form-layout-two-column;"));
        assert!(LAYOUT_CSS.contains("@container blocks-form-layout-two-column (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);"));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
        assert!(LAYOUT_CSS.contains("cursor: default;"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-form-layout-two-column-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-form-layout-two-column-layout"
        );
    }
}

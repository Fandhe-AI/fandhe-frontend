//! `settings-webhook-form` block（イシュー #3019。Application/Settings
//! カテゴリの Webhook 作成・編集フォーム）。(A) 代表構成・(B) ペイロード
//! 形式のラジオ群 + 詳細設定の開閉欄を持つ版・(C) セクションごとにカードへ
//! 分けた編集版の 3 variant を 1 ページに並記する合成例。
//!
//! # 使用部品
//!
//! `field` / `input` / `input-group` / `textarea` / `radio-group` /
//! `collapsible` / `switch` / `card` / `badge` / `button` の 10 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 参照について
//!
//! 対応表 ID R0380（代表構成、主参照）・R0381（ペイロード形式ラジオ +
//! 開閉式詳細設定、集約元）・R0382（セクションごとにカードへ分けた編集版、
//! 集約元）のみを記録する。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない契約（`settings_profile_form` 等と同型）。参照ファイル置き場
//! `_/blocks-intake/` はこの worktree に存在せず、実物は未参照。構成は
//! Issue 本文のレイアウト仕様のみから組み立てた。
//!
//! # `<form>` を持たない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり静的な合成例。ボタンは
//! [`fandhe_frontend_pre_styled_ui::button::button`] の既定 `type="button"`
//! のまま送信先・バリデーションを持たない（`docs/policy/
//! intentional-non-adoption.md` §3.25）。
//!
//! # 購読イベント・ペイロード形式・リトライ・詳細設定を無 JS で操作不能にする理由
//!
//! docs サイトは JS ハイドレーションを行わないため、`switch`/`radio-group`/
//! `collapsible` trigger のような有効なネイティブ操作系要素をそのまま出すと
//! 「操作可能に見えるが無反応」という不整合が生じる
//! （`settings_profile_form`/`form_layout_stacked` と同型の判断）。本 block
//! は `switch`/`radio-group`/`collapsible` trigger すべてへ `disabled: true`
//! を共有してネイティブ `disabled`（`collapsible` trigger はネイティブ
//! `disabled` 存在属性）で操作を構造的に禁止し、disabled 見た目
//! （`opacity: 0.5`）のみ [`LAYOUT_CSS`] で中和する。
//!
//! # `collapsible` を `OpenState::Open` 固定で描画する理由
//!
//! `OpenState::Closed` 固定だと [`fandhe_frontend_pre_styled_ui::collapsible::
//! content`] に `hidden` 存在属性が付き、無 JS では内容へ到達不能になる
//! （`header_floating_pill` と同じ判断）。そのため variant B の「詳細設定」
//! は常に展開した状態で描画し、トリガーは `disabled: true` で操作のみ
//! 封じる。
//!
//! # 署名シークレット欄に `value` を出さない理由
//!
//! 既存のシークレット値を画面へ平文表示しないため、`placeholder` のみを
//! 出力し `value` 属性は一切渡さない（OWASP A02 機微情報の露出対策）。
//!
//! # 3 variant で id 接頭辞を分ける理由
//!
//! variant A（`basic`）・B（`payload`）・C（`cards`）はいずれも同じ名前・
//! URL・説明・購読イベント欄を持つため、`field_id(variant, suffix)` で
//! variant ごとに id を分離する。`radio-group`/`switch` の `name` 属性も
//! 同じ接頭辞で分け、`crates/docs-site/tests/
//! blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_
//! duplicate_ids` の重複 id 検知に通す。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `field`/`input`/`input-group`/`textarea`/`radio-group`/`switch`/
//! `card`（`root` のみ）/`badge`/`button` は `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo 固有
//! のフックは `data-blocks-settings-webhook-form-*` 属性で渡す。
//! `card::header`/`card::body`/`card::title` と素の `div`/`h3`/`p` には
//! `class` がそのまま効くため `.blocks-settings-webhook-form-*` クラス
//! セレクタを使う（`settings_profile_form` と同じ判断）。
//!
//! # 見出しレベル（`H3`）
//!
//! ページ側が `## Demo` として `h2` を出すため、variant 見出しは素の
//! `h3` 要素にする（`heading` 部品は Issue 指定 10 部品に含まれないため
//! 使わない。`card::title` はカード版の見出し専用に使う）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::collapsible::{self, OpenState};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::radio_group::{self, RadioGroupProps};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 一意な id を組み立てる（variant 接頭辞 + フィールド接尾辞、モジュール
/// doc「3 variant で id 接頭辞を分ける理由」節）。
fn field_id(variant: &str, suffix: &str) -> String {
    format!("blocks-settings-webhook-form-{variant}-{suffix}")
}

/// 名前欄（`field` + `input`）。
fn name_field(variant: &str, value: Option<&'static str>) -> Node {
    let id = field_id(variant, "name");
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let mut attrs = vec![("type", "text"), ("placeholder", "注文通知")];
    if let Some(v) = value {
        attrs.push(("value", v));
    }
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &props,
        vec![("data-blocks-settings-webhook-form-field", "")],
        vec![
            field::label(&props, vec![], vec![text("名前")]),
            input::input(&InputProps::default(), &props, attrs),
        ],
    )
}

/// エンドポイント URL 欄（`input-group` の `https://` 固定 addon + `input`）。
fn url_field(variant: &str, value: Option<&'static str>) -> Node {
    let id = field_id(variant, "endpoint-url");
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let mut attrs = vec![
        ("type", "text"),
        ("placeholder", "api.example.com/webhooks/incoming"),
    ];
    if let Some(v) = value {
        attrs.push(("value", v));
    }
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &props,
        vec![("data-blocks-settings-webhook-form-field", "")],
        vec![
            field::label(&props, vec![], vec![text("エンドポイント URL")]),
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
                    input::input(&InputProps::default(), &props, attrs),
                ],
            ),
        ],
    )
}

/// 説明欄（`textarea`）。
fn description_field(variant: &str, content: Option<&'static str>) -> Node {
    let id = field_id(variant, "description");
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let children = content.map(|c| vec![text(c)]).unwrap_or_default();
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &props,
        vec![("data-blocks-settings-webhook-form-field", "")],
        vec![
            field::label(&props, vec![], vec![text("説明")]),
            textarea::textarea(
                &TextareaProps::default(),
                &props,
                false,
                vec![
                    ("rows", "2"),
                    ("placeholder", "この Webhook の用途を記入してください。"),
                ],
                children,
            ),
        ],
    )
}

/// 購読イベント 1 件（`switch` + イベント名 `badge`、モジュール doc「購読
/// イベント・ペイロード形式・リトライ・詳細設定を無 JS で操作不能にする
/// 理由」節）。
fn event_switch(
    variant: &str,
    idx: u8,
    event_key: &'static str,
    label_text: &'static str,
    checked: bool,
) -> Node {
    let name = field_id(variant, &format!("event-{idx}"));
    let props = SwitchProps {
        disabled: true,
        ..SwitchProps::default()
    };
    div(
        vec![("class", "blocks-settings-webhook-form-event-row")],
        vec![
            switch::root(
                Size::Md,
                ColorPalette::Accent,
                checked,
                &props,
                vec![("data-blocks-settings-webhook-form-switch", "")],
                vec![
                    switch::label(checked, &props, vec![], vec![text(label_text)]),
                    switch::hidden_input(name.as_str(), "on", checked, &props, vec![]),
                    switch::control(
                        checked,
                        &props,
                        vec![],
                        vec![switch::thumb(checked, &props, vec![], vec![])],
                    ),
                ],
            ),
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-settings-webhook-form-event-badge", "")],
                vec![text(event_key)],
            ),
        ],
    )
}

/// 購読イベント一式（新規注文・注文更新・決済完了・顧客削除の 4 件）。
fn events_section(
    variant: &str,
    created: bool,
    updated: bool,
    succeeded: bool,
    deleted: bool,
) -> Node {
    div(
        vec![("class", "blocks-settings-webhook-form-event-list")],
        vec![
            event_switch(variant, 0, "order.created", "新規注文", created),
            event_switch(variant, 1, "order.updated", "注文更新", updated),
            event_switch(variant, 2, "payment.succeeded", "決済完了", succeeded),
            event_switch(variant, 3, "customer.deleted", "顧客削除", deleted),
        ],
    )
}

/// ペイロード形式 `radio-group` の 1 item。
fn payload_radio_item(
    variant: &str,
    checked: bool,
    value: &'static str,
    label_text: &'static str,
) -> Node {
    let name = field_id(variant, "payload-format");
    let props = RadioGroupProps {
        disabled: true,
        ..RadioGroupProps::default()
    };
    radio_group::item(
        checked,
        &props,
        value,
        vec![("data-blocks-settings-webhook-form-radio-item", "")],
        vec![
            radio_group::item_hidden_input(checked, &props, Some(name.as_str()), value, vec![]),
            radio_group::item_control(checked, &props, vec![]),
            radio_group::item_text(checked, &props, vec![], vec![text(label_text)]),
        ],
    )
}

/// ペイロード形式欄（見出し `radio_group::label` + `json`/`form-urlencoded`
/// の 2 択、`json` を既定 checked とする）。
fn payload_radio_group(variant: &str) -> Node {
    let label_id = field_id(variant, "payload-format-label");
    let props = RadioGroupProps {
        disabled: true,
        ..RadioGroupProps::default()
    };
    div(
        vec![("data-blocks-settings-webhook-form-radio-group", "")],
        vec![
            radio_group::label(
                &props,
                Some(label_id.as_str()),
                vec![],
                vec![text("ペイロード形式")],
            ),
            radio_group::root(
                Size::Md,
                ColorPalette::Accent,
                true,
                None,
                Some(label_id.as_str()),
                vec![],
                vec![
                    payload_radio_item(variant, true, "json", "application/json"),
                    payload_radio_item(variant, false, "form", "application/x-www-form-urlencoded"),
                ],
            ),
        ],
    )
}

/// リトライ `switch`（失敗時の自動再試行）。
fn retry_switch(variant: &str, checked: bool) -> Node {
    let name = field_id(variant, "retry");
    let props = SwitchProps {
        disabled: true,
        ..SwitchProps::default()
    };
    switch::root(
        Size::Md,
        ColorPalette::Accent,
        checked,
        &props,
        vec![("data-blocks-settings-webhook-form-switch", "")],
        vec![
            switch::label(
                checked,
                &props,
                vec![],
                vec![text("失敗時に自動で再試行する")],
            ),
            switch::hidden_input(name.as_str(), "on", checked, &props, vec![]),
            switch::control(
                checked,
                &props,
                vec![],
                vec![switch::thumb(checked, &props, vec![], vec![])],
            ),
        ],
    )
}

/// タイムアウト秒欄（`input-group` の末尾「秒」固定 addon + `input`）。
fn timeout_field(variant: &str) -> Node {
    let id = field_id(variant, "timeout");
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
        vec![("data-blocks-settings-webhook-form-field", "")],
        vec![
            field::label(&props, vec![], vec![text("タイムアウト")]),
            input_group::root(
                &InputGroupProps {
                    disabled: false,
                    invalid: false,
                },
                vec![],
                vec![
                    input::input(
                        &InputProps::default(),
                        &props,
                        vec![
                            ("type", "number"),
                            ("inputmode", "numeric"),
                            ("value", "10"),
                        ],
                    ),
                    input_group::addon(
                        InputGroupAlign::InlineEnd,
                        &InputGroupProps {
                            disabled: false,
                            invalid: false,
                        },
                        vec![],
                        vec![input_group::text(vec![], vec![text("秒")])],
                    ),
                ],
            ),
        ],
    )
}

/// 署名シークレット欄（`field` + `input`。モジュール doc「署名シークレット
/// 欄に `value` を出さない理由」節、既存値は平文表示しない）。
fn secret_field(variant: &str) -> Node {
    let id = field_id(variant, "secret");
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
        vec![("data-blocks-settings-webhook-form-field", "")],
        vec![
            field::label(&props, vec![], vec![text("署名シークレット")]),
            input::input(
                &InputProps::default(),
                &props,
                vec![
                    ("type", "text"),
                    ("placeholder", "変更する場合のみ新しい値を入力してください"),
                ],
            ),
        ],
    )
}

/// 「詳細設定」`collapsible`（Open 固定、モジュール doc「`collapsible` を
/// `OpenState::Open` 固定で描画する理由」節）。署名シークレット・リトライ・
/// タイムアウトを含む。variant B にのみ追加する。
fn advanced_settings(variant: &str) -> Node {
    let panel_id = field_id(variant, "advanced-panel");
    div(
        vec![("data-blocks-settings-webhook-form-collapsible", "")],
        vec![collapsible::root(
            OpenState::Open,
            true,
            vec![],
            vec![
                collapsible::trigger(
                    OpenState::Open,
                    true,
                    Some(panel_id.as_str()),
                    vec![("data-blocks-settings-webhook-form-collapsible-trigger", "")],
                    vec![text("詳細設定")],
                ),
                collapsible::content(
                    OpenState::Open,
                    true,
                    Some(panel_id.as_str()),
                    vec![("class", "blocks-settings-webhook-form-advanced-content")],
                    vec![
                        secret_field(variant),
                        retry_switch(variant, false),
                        timeout_field(variant),
                    ],
                ),
            ],
        )],
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
        vec![("class", "blocks-settings-webhook-form-actions")],
        children,
    )
}

/// variant 1 件を見出し + 説明文 + フィールド群 + フッターでまとめる
/// 共通ヘルパ（A/B 用、モジュール doc「見出しレベル（`H3`）」節）。
fn variant_section(
    kind: &'static str,
    title: &'static str,
    description: &'static str,
    fields: Vec<Node>,
    footer: Node,
) -> Node {
    div(
        vec![
            ("class", "blocks-settings-webhook-form-variant"),
            ("data-blocks-settings-webhook-form-variant", kind),
        ],
        vec![
            el(
                "h3",
                vec![("class", "blocks-settings-webhook-form-variant-title")],
                vec![text(title)],
            ),
            el(
                "p",
                vec![("class", "blocks-settings-webhook-form-variant-description")],
                vec![text(description)],
            ),
            div(
                vec![("class", "blocks-settings-webhook-form-fields")],
                fields,
            ),
            footer,
        ],
    )
}

/// variant A（代表構成、対応表 ID R0380）。
fn basic_variant() -> Node {
    variant_section(
        "basic",
        "代表構成",
        "Webhook の基本設定です。送信先 URL と購読イベントを指定します。",
        vec![
            name_field("basic", None),
            url_field("basic", None),
            description_field("basic", None),
            events_section("basic", true, false, true, false),
        ],
        footer_actions(false),
    )
}

/// variant B（ペイロード形式のラジオ群 + 詳細設定の開閉欄を持つ版、対応表
/// ID R0381）。
fn payload_variant() -> Node {
    variant_section(
        "payload",
        "ペイロード形式 + 詳細設定版",
        "送信するペイロード形式を選べ、署名シークレット・再試行・タイムアウトなどの詳細設定を続けて設定できます。",
        vec![
            name_field("payload", None),
            url_field("payload", None),
            description_field("payload", None),
            events_section("payload", true, false, true, false),
            payload_radio_group("payload"),
            advanced_settings("payload"),
        ],
        footer_actions(false),
    )
}

/// 基本情報カード（variant C）。
fn basic_info_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-webhook-form-card", "")],
        vec![
            card::header(
                vec![("class", "blocks-settings-webhook-form-card-header")],
                vec![
                    card::title(vec![], vec![text("基本情報")]),
                    badge::badge(&BadgeProps::default(), vec![], vec![text("有効")]),
                ],
            ),
            card::body(
                vec![("class", "blocks-settings-webhook-form-card-body")],
                vec![
                    name_field("cards", Some("注文通知")),
                    url_field("cards", Some("api.example.com/webhooks/incoming")),
                    description_field(
                        "cards",
                        Some("注文・決済イベントを外部システムへ転送します。"),
                    ),
                ],
            ),
        ],
    )
}

/// 購読イベントカード（variant C）。
fn events_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-webhook-form-card", "")],
        vec![
            card::header(
                vec![("class", "blocks-settings-webhook-form-card-header")],
                vec![
                    card::title(vec![], vec![text("購読イベント")]),
                    badge::badge(&BadgeProps::default(), vec![], vec![text("2 件")]),
                ],
            ),
            card::body(
                vec![("class", "blocks-settings-webhook-form-card-body")],
                vec![events_section("cards", true, false, true, false)],
            ),
        ],
    )
}

/// 配信設定カード（variant C。ペイロード形式 + リトライ）。
fn delivery_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-webhook-form-card", "")],
        vec![
            card::header(
                vec![("class", "blocks-settings-webhook-form-card-header")],
                vec![card::title(vec![], vec![text("配信設定")])],
            ),
            card::body(
                vec![("class", "blocks-settings-webhook-form-card-body")],
                vec![payload_radio_group("cards"), retry_switch("cards", true)],
            ),
        ],
    )
}

/// variant C（セクションごとにカードへ分けた編集版、対応表 ID R0382）。
/// 既存の Webhook 設定を編集する想定のため、名前・URL・説明へ架空の既存値を
/// 入れる。
fn cards_variant() -> Node {
    div(
        vec![
            ("class", "blocks-settings-webhook-form-variant"),
            ("data-blocks-settings-webhook-form-variant", "cards"),
        ],
        vec![
            el(
                "h3",
                vec![("class", "blocks-settings-webhook-form-variant-title")],
                vec![text("セクションごとにカードへ分けた編集版")],
            ),
            el(
                "p",
                vec![("class", "blocks-settings-webhook-form-variant-description")],
                vec![text(
                    "既存の Webhook 設定を編集する構成です。入力欄にはすでに値が入っています。",
                )],
            ),
            div(
                vec![("class", "blocks-settings-webhook-form-cards")],
                vec![basic_info_card(), events_card(), delivery_card()],
            ),
            footer_actions(true),
        ],
    )
}

/// `settings-webhook-form` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。3 variant を縦に並べる。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-webhook-form-layout")],
        vec![basic_variant(), payload_variant(), cards_variant()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-webhook-form/",
    title: "settings-webhook-form",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_webhook_form.rs",
    demo_class: "blocks-settings-webhook-form",
    parts: &[
        Part {
            label: "Field",
            path: "/themes/field/",
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
            label: "Radio Group",
            path: "/themes/radio-group/",
        },
        Part {
            label: "Collapsible",
            path: "/themes/collapsible/",
        },
        Part {
            label: "Switch",
            path: "/themes/switch/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_webhook_form` 固有のレイアウト規則（`crate::blocks` モジュール
/// doc「CSS の置き場」節）。セレクタは `.blocks-settings-webhook-form-*` と
/// `[data-blocks-settings-webhook-form-*]`、および styled `switch`/
/// `radio-group`/`collapsible` の `[data-scope=...]` 系セレクタへの上書き
/// のみを用いる。
const LAYOUT_CSS: &str = "\
.blocks-settings-webhook-form-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  max-width: 42rem;\n  margin-inline: auto;\n}\n\
.blocks-settings-webhook-form-variant {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-webhook-form-variant-title {\n  margin: 0;\n}\n\
.blocks-settings-webhook-form-variant-description {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-webhook-form-fields {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-webhook-form-event-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-settings-webhook-form-event-row {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-webhook-form-advanced-content {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-webhook-form-cards {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-webhook-form-card-header {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-settings-webhook-form-card-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-webhook-form-actions {\n  display: flex;\n  justify-content: flex-end;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"switch\"][data-part=\"root\"][data-blocks-settings-webhook-form-switch][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"radio-group\"][data-part=\"item\"][data-blocks-settings-webhook-form-radio-item][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-settings-webhook-form-collapsible-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, BLOCK, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// Demo が期待する 10 種の部品を含むことを固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"field\"",
            "data-scope=\"input-group\"",
            "data-scope=\"radio-group\"",
            "data-scope=\"collapsible\"",
            "data-scope=\"switch\"",
            "data-scope=\"card\"",
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
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

    /// ボタン系要素はすべて `type="button"`（保存 3 + キャンセル 1 +
    /// collapsible trigger 1 = 5 個）。
    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        assert_eq!(html.matches(r#"type="button""#).count(), 5);
    }

    /// ペイロード形式 `radio-group` は variant B・C の 2 グループで 4 個の
    /// `type="radio"` を持ち、各グループで 1 件のみ checked。
    #[test]
    fn payload_radio_groups_have_single_checked_default() {
        let html = demo_html();
        assert_eq!(html.matches(r#"type="radio""#).count(), 4);
        assert_eq!(
            html.matches("value=\"json\" data-state=\"checked\"")
                .count(),
            2
        );
    }

    /// 署名シークレット入力欄に `value` が出力されないこと。
    #[test]
    fn secret_input_has_no_value() {
        let html = demo_html();
        assert!(html.contains("署名シークレット"));
        assert!(!html.contains("value=\"既存"));
    }

    /// `aria-controls`（collapsible trigger）と `id`（collapsible content）
    /// が対であること。
    #[test]
    fn collapsible_trigger_and_content_ids_are_paired() {
        let html = demo_html();
        assert!(
            html.contains("aria-controls=\"blocks-settings-webhook-form-payload-advanced-panel\"")
        );
        assert!(html.contains("id=\"blocks-settings-webhook-form-payload-advanced-panel\""));
    }

    /// collapsible content は `hidden` を持たない（Open 固定、モジュール doc
    /// 「`collapsible` を `OpenState::Open` 固定で描画する理由」節）。
    #[test]
    fn collapsible_content_is_not_hidden() {
        let html = demo_html();
        let content_start = html
            .find(r#"data-scope="collapsible" data-part="content""#)
            .expect("collapsible content should be present");
        let content_tag_end = html[content_start..]
            .find('>')
            .map(|offset| content_start + offset)
            .unwrap_or(html.len());
        assert!(!html[content_start..content_tag_end].contains("hidden"));
    }

    /// [`LAYOUT_CSS`] が disabled 中和規則を含み、`<` を含まないこと。
    #[test]
    fn layout_css_declares_disabled_neutralization() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"switch\"][data-part=\"root\"][data-blocks-settings-webhook-form-switch][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"radio-group\"][data-part=\"item\"][data-blocks-settings-webhook-form-radio-item][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-settings-webhook-form-collapsible-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-settings-webhook-form-layout\""));
        assert_ne!(BLOCK.demo_class, "blocks-settings-webhook-form-layout");
    }
}

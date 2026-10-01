# settings-webhook-form

Webhook の作成・編集フォームの 3 variant を並記した合成例です。`field` /
`input` / `input-group` / `textarea` / `radio-group` / `collapsible` /
`switch` / `card` / `badge` / `button` の 10 部品を合成します。対応表 ID
R0380（代表構成、主参照）・R0381（ペイロード形式ラジオ + 開閉式詳細設定）・
R0382（セクションごとにカードへ分けた編集版）を集約元とし、取得手段・
ファイル名・内部コンポーネント識別子は記載しません。参照ファイル置き場
`_/blocks-intake/` はこの worktree に存在せず、実物は未参照です。Blocks は
既存部品の合成例であり、新しい UI 部品は追加しません。

- variant A（代表構成）: 名前・エンドポイント URL・説明・購読イベント
  （4 件の switch + イベント名 badge）を設定できます。
- variant B（ペイロード形式 + 詳細設定版）: A の入力欄に加え、ペイロード
  形式（JSON / x-www-form-urlencoded）の radio group と、開閉式の
  「詳細設定」（署名シークレット・リトライ・タイムアウト秒）を持ちます。
- variant C（セクションごとにカードへ分けた編集版）: 基本情報・購読
  イベント・配信設定の 3 枚のカードへ分割した編集画面で、入力欄には
  すでに値が入っています。

URL・イベント名・説明文はすべて架空のもので、実在の企業・PII は含みません。
署名シークレット欄は既存値を平文表示しないため `value` を出力せず
`placeholder` のみです。購読イベント（`switch`）・ペイロード形式
（`radio-group`）・詳細設定の開閉（`collapsible` trigger）はいずれも
ネイティブ `disabled` で固定した静的表示であり、無 JS の docs サイトで
操作可能に見えて実は無反応という不整合を避けています。「詳細設定」の
`collapsible` は `OpenState::Open` 固定で描画し、閉状態固定時に付く
`hidden` 属性で内容へ到達不能になる問題を避けています。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。送信処理・
バリデーションは持たず、実際の実装は利用者自身の Rust/JS コードで行います。

## Rust コード

```rust
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
```

## 原案差分メモ

- イシュー本文のレイアウト仕様（代表構成・ペイロード形式 + 詳細設定版・
  カード分割編集版の 3 構成）のみから組み立てました。参照ファイル置き場
  `_/blocks-intake/` はこの worktree に存在せず、構成の実物（R0380〜R0382）
  は未参照です。
- 名前・URL・説明・購読イベントの入力欄は 3 variant 共通の構造を
  `name_field`/`url_field`/`description_field`/`events_section` ヘルパへ
  共通化し、id・`name` 属性は variant ごとに接頭辞を分けて一意化しています
  （重複 id・宙ぶらりん aria 参照の回避）。
- 見出しは `heading` 部品を使わず素の `h3` 要素にしています（Issue 指定の
  10 部品に `heading` が含まれないため）。カード版の見出しのみ
  `card::title` を使います。

関連情報: [Field](../themes/field.md) / [Input](../themes/input.md) /
[Input Group](../themes/input-group.md) / [Textarea](../themes/textarea.md) /
[Radio Group](../themes/radio-group.md) /
[Collapsible](../themes/collapsible.md) / [Switch](../themes/switch.md) /
[Card](../themes/card.md) / [Badge](../themes/badge.md) /
[Button](../themes/button.md)

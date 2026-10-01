# settings-webhook-tester

Webhook のテスト送信画面を表した設定ブロックです。宛先エンドポイントと
イベント種類を選択欄で選び、送信されるペイロードをコード表示で確認し、
送信ボタンを押す構成を縦に並べます。送信結果は成功・失敗の 2 状態を
並記し、状態バッジと応答内容を確認できます。`native-select` / `field` /
`code` / `button` / `badge` の 5 部品を合成します。Blocks は既存部品の
合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0387（代表構成）の 1 件のみで、集約元の差分はありません。
狭いコンテナ幅でもブレークポイントによる横並び切り替えは行わず、常に
縦積みのまま表示します。

宛先 URL・イベント名・応答内容はすべて架空のデータです（`example.com`
ドメイン・`evt_`/`ord_` 系の架空 ID）。署名シークレット等のクレデンシャル
様の値は出力しません。

本 Demo は無 JS の静的表示です。`<form>` を含まず、選択欄の選択操作自体は
ネイティブに機能しますが、ペイロード表示・送信ボタン・送信結果はいずれも
選択値に追従しない固定表示です。「テスト送信」ボタンは送信先・検証・
永続化の経路を一切持ちません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, h3, h4, p, pre, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// 送信されるペイロード例（架空の JSON、実クレデンシャル・PII を含まない。
/// 選択欄の選択値には追従しない静的表示、モジュール doc「無 JS のため
/// 選択は送信・ペイロードに反映されない」節参照）。
const PAYLOAD_JSON: &str = "{\n  \"event\": \"order.created\",\n  \"id\": \"evt_7f0c1a9d\",\n  \"data\": {\n    \"order_id\": \"ord_4821\",\n    \"amount\": 3200\n  }\n}";

/// 成功時の応答本文（架空値）。
const SUCCESS_BODY: &str = "{\n  \"received\": true\n}";

/// 失敗時の応答本文（架空値）。
const FAILURE_BODY: &str = "{\n  \"error\": \"internal_error\"\n}";

/// 送信結果の種別（成功・失敗の 2 状態を並記する、モジュール doc
/// 「成功・失敗の 2 状態を並記する」節参照）。
#[derive(Clone, Copy)]
enum ResultKind {
    Success,
    Failure,
}

impl ResultKind {
    /// 結果見出し（素の `h4`。`heading` 部品を使わずページ右目次への混入を
    /// 避ける判断は `settings_webhook_detail`/`settings_webhook_stats` と
    /// 同型）。
    fn heading(self) -> &'static str {
        match self {
            ResultKind::Success => "成功した場合",
            ResultKind::Failure => "失敗した場合",
        }
    }

    /// 状態バッジの色。
    fn palette(self) -> ColorPalette {
        match self {
            ResultKind::Success => ColorPalette::Success,
            ResultKind::Failure => ColorPalette::Danger,
        }
    }

    /// 状態バッジの文言（色だけでなく文言でも判別できるようにする）。
    fn label(self) -> &'static str {
        match self {
            ResultKind::Success => "成功",
            ResultKind::Failure => "失敗",
        }
    }

    /// 補足説明（架空値）。
    fn detail(self) -> &'static str {
        match self {
            ResultKind::Success => "200 OK・182ms",
            ResultKind::Failure => "500・タイムアウトまで 3 回再試行",
        }
    }

    /// 応答本文（架空値）。
    fn body(self) -> &'static str {
        match self {
            ResultKind::Success => SUCCESS_BODY,
            ResultKind::Failure => FAILURE_BODY,
        }
    }
}

/// 選択欄 1 件分（`field` + `native-select`、`id` は
/// `blocks-settings-webhook-tester-{id_suffix}`）。`options` は
/// `(value, label, selected)` の並び。
fn select_field(
    id_suffix: &'static str,
    label_text: &'static str,
    options: &[(&'static str, &'static str, bool)],
) -> Node {
    let id = format!("blocks-settings-webhook-tester-{id_suffix}");
    let field_props = FieldProps {
        id: &id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let option_nodes: Vec<Node> = options
        .iter()
        .map(|(value, label, selected)| {
            let mut attrs = vec![("value", *value)];
            if *selected {
                attrs.push(("selected", "selected"));
            }
            el("option", attrs, vec![text(*label)])
        })
        .collect();
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &field_props,
        vec![("data-blocks-settings-webhook-tester-field", "")],
        vec![
            field::label(&field_props, vec![], vec![text(label_text)]),
            native_select::native_select(
                &NativeSelectProps::default(),
                &field_props,
                vec![],
                option_nodes,
            ),
        ],
    )
}

/// 送信結果 1 件分（見出し行〔`h4` + `badge`〕+ 補足 `p` + 応答本文
/// `pre`/`code`）。
fn result_item(kind: ResultKind) -> Node {
    div(
        vec![("class", "blocks-settings-webhook-tester-result")],
        vec![
            div(
                vec![("class", "blocks-settings-webhook-tester-result-heading")],
                vec![
                    h4(vec![], vec![text(kind.heading())]),
                    badge::badge(
                        &BadgeProps {
                            variant: BadgeVariant::Subtle,
                            palette: kind.palette(),
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text(kind.label())],
                    ),
                ],
            ),
            p(vec![], vec![text(kind.detail())]),
            pre(
                vec![("class", "blocks-settings-webhook-tester-code")],
                vec![code::code(
                    &CodeProps::default(),
                    vec![],
                    vec![text(kind.body())],
                )],
            ),
        ],
    )
}

/// `settings-webhook-tester` の Demo 本体（呼び出しごとに同一の `Node` を
/// 返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-webhook-tester-layout")],
        vec![
            h3(vec![], vec![text("テスト送信")]),
            p(
                vec![("class", "blocks-settings-webhook-tester-lead")],
                vec![text(
                    "宛先とイベント種類を選び、固定のペイロードを送信します。",
                )],
            ),
            div(
                vec![("class", "blocks-settings-webhook-tester-fields")],
                vec![
                    select_field(
                        "endpoint",
                        "送信先エンドポイント",
                        &[
                            (
                                "https://hooks.example.com/ingest/8f2c91ab",
                                "https://hooks.example.com/ingest/8f2c91ab",
                                true,
                            ),
                            (
                                "https://hooks.example.com/ingest/staging",
                                "https://hooks.example.com/ingest/staging",
                                false,
                            ),
                        ],
                    ),
                    select_field(
                        "event",
                        "イベント種類",
                        &[
                            ("order.created", "order.created", true),
                            ("order.updated", "order.updated", false),
                            ("payment.succeeded", "payment.succeeded", false),
                        ],
                    ),
                ],
            ),
            h4(vec![], vec![text("送信されるペイロード")]),
            pre(
                vec![("class", "blocks-settings-webhook-tester-code")],
                vec![code::code(
                    &CodeProps::default(),
                    vec![],
                    vec![text(PAYLOAD_JSON)],
                )],
            ),
            div(
                vec![("class", "blocks-settings-webhook-tester-actions")],
                vec![button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Solid,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("テスト送信")],
                )],
            ),
            h3(vec![], vec![text("送信結果")]),
            div(
                vec![("class", "blocks-settings-webhook-tester-results")],
                vec![
                    result_item(ResultKind::Success),
                    result_item(ResultKind::Failure),
                ],
            ),
        ],
    )
}
```

## 原案差分メモ

- 集約元は対応表 ID R0387（代表構成）の 1 件のみで、差分併記はありません。
- `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
  存在しないため、参照ファイルは未参照のまま対応表 ID のみを記載し、
  構成は Issue 本文のレイアウト仕様から組み立てました
  （`settings-webhook-detail` 等の既存 block と同じ扱い）。

関連情報: [Native Select](../themes/native-select.md) /
[Field](../themes/field.md) / [Code](../themes/code.md) /
[Button](../themes/button.md) / [Badge](../themes/badge.md)

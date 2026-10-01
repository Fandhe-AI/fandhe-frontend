# settings-webhook-wizard

Webhook を「宛先 → イベント → 確認」の 3 段ステップで作るステップ式
フローの合成例です。`steps` / `field` / `input` / `checkbox` / `button` の
5 部品を合成します。対応表 ID R0383（主参照、代表構成）のみを記録し、
取得手段・ファイル名・内部コンポーネント識別子は記載しません。参照
ファイル置き場 `_/blocks-intake/` はこの worktree に存在せず、実物は
未参照です。Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

- 無 JS の docs サイトのため、3 インスタンス（destination / events /
  review）を縦に並べて各ステップの状態を静的に併記します。各インスタンスは
  現在ステップの内容のみを描画します。
- 上部の 3 段ステップは現在ステップを強調し、完了したステップにはチェック
  印を表示します。
- 現在ステップの入力欄と「前へ」「次へ」ボタンを持ちますが、押しても何も
  起きない無効な操作に見せないため、ボタン・checkbox はすべてネイティブ
  disabled です。
- 確認ステップ（review）では、それまでの入力内容（Webhook 名・エンドポ
  イント URL・選択した購読イベント）を素の `dl`/`dt`/`dd` で要約表示します。

URL・Webhook 名・購読イベント名はすべて架空のもので、実在の企業・PII は
含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。送信処理・
バリデーションは持たず、実際の実装は利用者自身の Rust/JS コードで行います。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::steps;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// Webhook 名（destination/review 共有、宛先不一致を防ぐため `const` 化）。
const WEBHOOK_NAME: &str = "注文更新通知";
/// エンドポイント URL（destination/review 共有、架空値）。
const ENDPOINT_URL: &str = "https://example.com/hooks/order-updates";
/// 購読イベント 4 件（名前, 選択済みか）。events/review 共有。
const EVENTS: [(&str, bool); 4] = [
    ("order.created", true),
    ("order.updated", false),
    ("order.cancelled", true),
    ("order.refunded", false),
];

/// 一意な id を組み立てる（`blocks-settings-webhook-wizard-<instance>-`
/// 接頭辞を共通化し、フィールド追加時の綴り間違いを防ぐ）。
fn field_id(instance: &str, suffix: &str) -> String {
    format!("blocks-settings-webhook-wizard-{instance}-{suffix}")
}

/// 完了ステップのチェック SVG（アクセシブルネーム `aria-label="完了"` を
/// SVG 要素自身へ明示付与する。`order_tracking_progress.rs::stage_icon` と
/// 同型）。
fn complete_icon() -> Node {
    el(
        "svg",
        vec![
            ("viewBox", "0 0 24 24"),
            ("width", "16"),
            ("height", "16"),
            ("role", "img"),
            ("aria-label", "完了"),
        ],
        vec![el(
            "path",
            vec![
                ("d", "M5 12l4 4L19 7"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
            ],
            vec![],
        )],
    )
}

/// steps item 1 件（番号 or 完了印 + ラベル）。トリガーボタンにしない理由は
/// モジュール doc 「steps 上部ナビを `trigger` ボタンにしない理由」節参照。
fn step_item(s: &Steps, index: usize, label: &str) -> Node {
    let item_attrs = if index == s.step() {
        vec![("aria-current", "step")]
    } else {
        vec![]
    };
    let indicator_child = if index < s.step() {
        complete_icon()
    } else {
        text((index + 1).to_string())
    };
    let mut children = vec![
        steps::indicator(s, index, vec![], vec![indicator_child]),
        el(
            "span",
            vec![("class", "blocks-settings-webhook-wizard-step-label")],
            vec![text(label)],
        ),
    ];
    if index + 1 < s.count() {
        children.push(steps::separator(s, index, vec![], vec![]));
    }
    steps::item(s, index, item_attrs, children)
}

/// 3 段ステップの見出し行。
fn step_header(s: &Steps) -> Node {
    let labels = ["宛先", "イベント", "確認"];
    let items: Vec<Node> = labels
        .iter()
        .enumerate()
        .map(|(index, label)| step_item(s, index, label))
        .collect();
    steps::root(
        Size::Md,
        ColorPalette::Accent,
        s,
        vec![],
        vec![steps::list(s, vec![], items)],
    )
}

/// 前へ / 次へ（常にネイティブ disabled。理由はモジュール doc
/// 「前へ / 次へを `button` で組み、常に disabled にする理由」節参照）。
fn nav_actions(prev_label: &'static str, next_label: &'static str) -> Node {
    div(
        vec![("class", "blocks-settings-webhook-wizard-actions")],
        vec![
            button::button(
                &ButtonProps {
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-settings-webhook-wizard-button", "")],
                vec![text(prev_label)],
            ),
            button::button(
                &ButtonProps {
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-settings-webhook-wizard-button", "")],
                vec![text(next_label)],
            ),
        ],
    )
}

/// 読み取り専用の `field`/`input` 1 件を組み立てる。
fn readonly_field<'a>(id: &'a str, label_text: &'a str, value: &'a str) -> Node {
    let props = FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: true,
        has_helper_text: false,
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &props,
        vec![("data-blocks-settings-webhook-wizard-field", "")],
        vec![
            field::label(&props, vec![], vec![text(label_text)]),
            input::input(
                &InputProps::default(),
                &props,
                vec![("type", "text"), ("value", value)],
            ),
        ],
    )
}

/// **destination**（step 0）: 名前・エンドポイント URL。
fn destination_step(instance: &str) -> Node {
    let name_id = field_id(instance, "name");
    let url_id = field_id(instance, "url");
    div(
        vec![("class", "blocks-settings-webhook-wizard-fields")],
        vec![
            readonly_field(&name_id, "Webhook 名", WEBHOOK_NAME),
            readonly_field(&url_id, "エンドポイント URL", ENDPOINT_URL),
        ],
    )
}

/// 購読イベント checkbox 1 件。
fn event_checkbox(instance: &str, name: &'static str, checked: bool) -> Node {
    let props = CheckboxProps {
        checked: if checked {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
        ..CheckboxProps::default()
    };
    let full_name = format!("blocks-settings-webhook-wizard-{instance}-event-{name}");
    checkbox::root(
        Size::Md,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-settings-webhook-wizard-checkbox", "")],
        vec![
            checkbox::hidden_input(&props, &full_name, "on", vec![]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(&props, vec![], vec![text(name)]),
        ],
    )
}

/// **events**（step 1）: 購読イベント checkbox 4 件（2 件 checked）。
fn events_step(instance: &str) -> Node {
    div(
        vec![("class", "blocks-settings-webhook-wizard-event-list")],
        EVENTS
            .iter()
            .map(|(name, checked)| event_checkbox(instance, name, *checked))
            .collect(),
    )
}

/// **review**（step 2）: 素の `dl`/`dt`/`dd` による要約。
fn review_step() -> Node {
    let selected_events: Vec<&str> = EVENTS
        .iter()
        .filter(|(_, checked)| *checked)
        .map(|(name, _)| *name)
        .collect();
    el(
        "dl",
        vec![("class", "blocks-settings-webhook-wizard-summary")],
        vec![
            el("dt", vec![], vec![text("Webhook 名")]),
            el("dd", vec![], vec![text(WEBHOOK_NAME)]),
            el("dt", vec![], vec![text("エンドポイント URL")]),
            el("dd", vec![], vec![text(ENDPOINT_URL)]),
            el("dt", vec![], vec![text("購読イベント")]),
            el("dd", vec![], vec![text(selected_events.join(", "))]),
        ],
    )
}

/// 3 ステップ共通の骨格。`step_index` の [`Steps`] を組み立て、`body` へ
/// 渡したステップ固有の中身を現在ステップパネルへ差し込む。
fn instance(
    instance_name: &'static str,
    step_index: usize,
    title: &'static str,
    description: &'static str,
    next_label: &'static str,
    body: Node,
) -> Node {
    let s = Steps::new(
        3,
        step_index,
        fandhe_frontend_pre_styled_ui::Orientation::Horizontal,
    );
    div(
        vec![
            ("class", "blocks-settings-webhook-wizard-instance"),
            (
                "data-blocks-settings-webhook-wizard-instance",
                instance_name,
            ),
        ],
        vec![
            el(
                "h3",
                vec![("class", "blocks-settings-webhook-wizard-title")],
                vec![text(title)],
            ),
            step_header(&s),
            steps::content(
                &s,
                step_index,
                vec![("class", "blocks-settings-webhook-wizard-body")],
                vec![
                    el(
                        "p",
                        vec![("class", "blocks-settings-webhook-wizard-description")],
                        vec![text(description)],
                    ),
                    body,
                ],
            ),
            nav_actions("前へ", next_label),
        ],
    )
}

/// `settings-webhook-wizard` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。3 インスタンスを縦に並べる。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-webhook-wizard-layout")],
        vec![
            instance(
                "destination",
                0,
                "1. 宛先を入力",
                "Webhook の送信先を確認します。",
                "次へ",
                destination_step("destination"),
            ),
            instance(
                "events",
                1,
                "2. イベントを選択",
                "通知を受け取るイベントを確認します。",
                "次へ",
                events_step("events"),
            ),
            instance(
                "review",
                2,
                "3. 内容を確認",
                "これまでの入力内容を確認してから作成します。",
                "作成する",
                review_step(),
            ),
        ],
    )
}
```

## 原案差分メモ

- イシュー本文のレイアウト仕様（3 段ステップ・現在ステップ強調・完了印・
  確認ステップでの入力内容要約）のみから組み立てました。参照ファイル
  置き場 `_/blocks-intake/` はこの worktree に存在せず、構成の実物
  （R0383）は未参照です。
- `onboarding_centered_steps` と同型に、3 インスタンスを縦に並べて各
  ステップの内容を静的に併記しています（無 JS のため単一インスタンスで
  状態遷移を表現できないため）。
- `steps::trigger`（上部ナビ）は使わず、「前へ」「次へ」は Issue 指定の
  5 部品に含まれる `button` で組み、常に disabled にしています。
- 確認ステップの要約には `description-list` 部品を使わず、素の `dl` 要素を
  使っています（Issue 指定 5 部品に `description-list` が含まれないため）。

関連情報: [Steps](../themes/steps.md) / [Field](../themes/field.md) /
[Input](../themes/input.md) / [Checkbox](../themes/checkbox.md) /
[Button](../themes/button.md)

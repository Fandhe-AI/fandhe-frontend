//! `settings-webhook-wizard` block（イシュー #3022。親トラッキング #2951
//! 「Blocks 目的別パーツ拡充」配下）。Webhook を「宛先 → イベント → 確認」の
//! 3 段ステップで作るステップ式フローの合成例。
//!
//! # 使用部品
//!
//! `steps` / `field` / `input` / `checkbox` / `button` の 5 部品のみを
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 参照について
//!
//! 対応表 ID R0383（主参照、代表構成）のみを記録する。`_/blocks-intake/`
//! の対応ファイルは本イシュー着手時点で本 worktree に存在しないため、
//! 実物は未参照で、Issue 本文のレイアウト仕様のみから組み立てた
//! （`onboarding_centered_steps`/`settings_webhook_form` と同じ扱い）。
//!
//! # 3 インスタンスで各ステップを併記する理由（無 JS のため静的併記）
//!
//! `Steps::new(3, step_index, Orientation::Horizontal)` を変えた 3
//! インスタンスを縦に並べ、各インスタンスは現在ステップの内容のみを描画
//! する（[`fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::
//! steps::Steps::content`] が非現在 index を `hidden` にする契約のため、
//! そもそも非表示パネルを作らない。`onboarding_centered_steps` と同型）。
//!
//! - **destination**（step 0）: 名前・エンドポイント URL の `field`/`input`
//!   （`readonly: true`、架空値）。
//! - **events**（step 1）: 購読イベントの `checkbox` 4 件（2 件 checked）。
//! - **review**（step 2）: 素の `dl`/`dt`/`dd` で、名前・URL・購読イベントの
//!   要約を表示する（`description-list` 部品は指定 5 部品に含まれないため
//!   使わない）。
//!
//! # steps 上部ナビを `trigger` ボタンにしない理由
//!
//! `steps::trigger` は実 `<button>` であり、無 JS の docs サイトでは押しても
//! 何も起きない dead control になる（`onboarding_centered_steps` と同型の
//! 判断）。番号・ラベルは `item` 直下へ [`step_item`] で静的に組み立て、
//! 完了ステップにはチェック SVG（`role="img"` + `aria-label="完了"` で
//! アクセシブルネームを明示）を、現在・未完了ステップには番号テキストを
//! 入れる。現在ステップの強調は pre-styled の `data-state="current"`
//! スタイルと `aria-current="step"` に任せ、追加 CSS は書かない。
//!
//! # 前へ / 次へを `button` で組み、常に disabled にする理由
//!
//! Issue 指定の 5 部品に `steps::prev_trigger`/`next_trigger` は含まれない
//! ため、[`fandhe_frontend_pre_styled_ui::button::button`] で組む。押しても
//! 何も起きない dead control を操作可能に見せないよう、全インスタンスで
//! `ButtonProps { disabled: true, .. }` を固定する（`onboarding_centered_steps`
//! の `nav_actions` と同じ判断）。減光は中和しない（操作不能であることを
//! 視覚的にも伝える、`list_people.rs` と同型）。
//!
//! # checkbox をネイティブ disabled にし、減光のみ中和する理由
//!
//! `checkbox::hidden_input` は有効なネイティブ `<input>` であり、`disabled`
//! を渡さない構成では JS ハイドレーションなしでもラベルクリック・
//! キーボード操作でブラウザが `checked` をネイティブに切り替えてしまう
//! 一方、`indicator` の見た目は SSR 時の `checked` 引数から固定生成される
//! ため追従しない（`form_layout_stacked.rs`/`onboarding_centered_steps.rs`
//! と同型の判断）。全 checkbox で `disabled: true` を共有し、[`LAYOUT_CSS`]
//! の `[data-scope="checkbox"][data-part="root"]
//! [data-blocks-settings-webhook-wizard-checkbox][data-disabled]` 規則で
//! `opacity: 1`/`cursor: default` に中和する。
//!
//! # `<form>` を持たない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。ボタンは `button::button` の既定
//! `type="button"` のまま送信先を持たず、実際の送信処理・バリデーションは
//! 利用者自身の Rust/JS コードで実装する（`docs/policy/
//! intentional-non-adoption.md` §3.25）。
//!
//! # id とスタイルフックの規約
//!
//! id は [`field_id`] で `blocks-settings-webhook-wizard-<instance>-<suffix>`
//! の形に一意化し、checkbox の `name` 属性も同様に分ける
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` を通す）。
//! `steps`/`field`/`input`/`checkbox`/`button` の root には
//! `data-blocks-settings-webhook-wizard-*` 属性を、素の `div`/`h3`/`p`/`dl`/
//! `span` には `.blocks-settings-webhook-wizard-*` class を使う
//! （`drop_class_attr` の契約、`onboarding_centered_steps` と同型）。
//!
//! # 見出しレベル（`H3`）
//!
//! ページ側が `## Demo` として `h2` を出すため、インスタンス見出しは素の
//! `h3` 要素にする（`heading` 部品は Issue 指定 5 部品に含まれないため
//! 使わない）。
//!
//! # ダミー素材について
//!
//! 名前・エンドポイント URL・購読イベント名はすべて独自の日本語・
//! `example.com` 配下の架空値で、実在の人物・企業・クレデンシャルとは
//! 無関係。署名シークレットのような機微欄はそもそも持たない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-webhook-wizard/",
    title: "settings-webhook-wizard",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_webhook_wizard.rs",
    demo_class: "blocks-settings-webhook-wizard",
    parts: &[
        Part {
            label: "Steps",
            path: "/themes/steps/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_webhook_wizard` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。セレクタは
/// `.blocks-settings-webhook-wizard-*` と checkbox disabled 中和セレクタの
/// みを用いる。
const LAYOUT_CSS: &str = "\
.blocks-settings-webhook-wizard-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  max-width: 42rem;\n  margin-inline: auto;\n}\n\
.blocks-settings-webhook-wizard-instance {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-webhook-wizard-title {\n  margin: 0;\n}\n\
.blocks-settings-webhook-wizard-step-label {\n  font-size: var(--fandhe-font-size-sm);\n}\n\
.blocks-settings-webhook-wizard-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-webhook-wizard-description {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-webhook-wizard-fields {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-webhook-wizard-event-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-settings-webhook-wizard-summary {\n  display: grid;\n  grid-template-columns: max-content 1fr;\n  gap: var(--fandhe-space-2) var(--fandhe-space-4);\n  margin: 0;\n}\n\
.blocks-settings-webhook-wizard-summary dt {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-webhook-wizard-summary dd {\n  margin: 0;\n}\n\
.blocks-settings-webhook-wizard-actions {\n  display: flex;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-settings-webhook-wizard-checkbox][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, BLOCK, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"steps\"",
            "data-scope=\"field\"",
            "data-scope=\"checkbox\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains(r#"data-part="input""#));
    }

    #[test]
    fn demo_wires_all_three_instance_hooks() {
        let html = demo_html();
        for instance_name in ["destination", "events", "review"] {
            assert!(html.contains(&format!(
                "data-blocks-settings-webhook-wizard-instance=\"{instance_name}\""
            )));
        }
    }

    #[test]
    fn no_form_semantics_or_unsafe_markup() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("action="));
        assert!(!html.contains(r#"type="submit""#));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn ids_have_no_duplicates_and_no_dangling_aria_references() {
        let html = demo_html();
        let mut ids: Vec<&str> = Vec::new();
        for chunk in html.split("id=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                ids.push(&chunk[..end]);
            }
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "id が重複している: {ids:?}");

        for chunk in html.split("aria-labelledby=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                let target = &chunk[..end];
                assert!(
                    ids.contains(&target),
                    "aria-labelledby={target} が参照する id が存在しない"
                );
            }
        }
    }

    #[test]
    fn buttons_are_all_type_button() {
        let html = demo_html();
        let count_open = html.matches("<button").count();
        let count_typed = html.matches(r#"type="button""#).count();
        assert!(count_open > 0);
        assert_eq!(count_open, count_typed);
        // 前へ / 次へ 2 個 × 3 インスタンス = 6 個。
        assert_eq!(count_open, 6);
    }

    #[test]
    fn checkboxes_are_all_disabled_with_expected_checked_count() {
        let html = demo_html();
        // 各インスタンスは現在ステップのみを描画するため、購読イベント
        // checkbox は events インスタンス 1 回分の 4 件のみ出現する
        // （モジュール doc 「3 インスタンスで各ステップを併記する理由」
        // 節参照）。
        assert_eq!(html.matches(r#"type="checkbox""#).count(), 4);
        assert_eq!(html.matches(" disabled=\"\"").count(), 4 + 6);
        // 2 件 checked（order.created / order.cancelled）。
        assert_eq!(html.matches("checked=\"\"").count(), 2);
    }

    #[test]
    fn current_step_and_complete_marks_are_correct() {
        let html = demo_html();
        // 3 インスタンス、各 1 個の aria-current="step"。
        assert_eq!(html.matches("aria-current=\"step\"").count(), 3);
        // 完了印: events で 1 個、review で 2 個、合計 3 個。
        assert_eq!(html.matches("aria-label=\"完了\"").count(), 3);
    }

    #[test]
    fn review_summary_matches_earlier_steps() {
        let html = demo_html();
        assert!(html.contains(super::WEBHOOK_NAME));
        assert!(html.contains(super::ENDPOINT_URL));
        assert!(html.contains("order.created, order.cancelled"));
    }

    #[test]
    fn layout_css_is_safe_and_scoped() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains("display: none"));
        assert!(LAYOUT_CSS.contains("blocks-settings-webhook-wizard"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-settings-webhook-wizard-layout\""));
        assert_ne!(BLOCK.demo_class, "blocks-settings-webhook-wizard-layout");
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }
}

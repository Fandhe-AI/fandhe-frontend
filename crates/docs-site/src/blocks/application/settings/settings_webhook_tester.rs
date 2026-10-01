//! `settings-webhook-tester` block（イシュー #3021、親 #2951。Application /
//! Settings カテゴリ）。Webhook のテスト送信画面（宛先・イベント種類の
//! 選択欄 → 送信されるペイロードのコード表示 → 送信ボタン → 送信結果の
//! 静的合成例）。集約元は R0387（代表構成、主参照）の 1 件のみ。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`settings_webhook_detail`/`settings_webhook_stats` と同じ扱い）。
//!
//! # 使用部品
//!
//! `native-select` / `field` / `code` / `button` / `badge` の 5 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、`blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//!
//! # 常に縦積み（ブレークポイントで横並びにしない）
//!
//! Issue 仕様どおり、狭幅・広幅を問わず全要素を縦に並べたまま固定する。
//! `@container`/`@media` によるレイアウト切り替えは持たない。
//!
//! # 無 JS のため選択は送信・ペイロードに反映されない
//!
//! docs サイトは JS ハイドレーションを行わないため、`native-select` の
//! 選択操作自体はネイティブに機能するが（`settings_api_keys_table.rs
//! ::expiry_select` と同じ判断で `disabled` にはしない）、ペイロード表示・
//! 送信ボタン・送信結果はいずれも選択値に追従しない静的固定である。この
//! 不整合は本 Demo が「無 JS の静的合成例」であることの一般的な制約
//! （`settings_webhook_form` 等と同型）であり、個別の注記はページ本文へ
//! 委ねる。
//!
//! # `<form>` を使わない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは [`fandhe_frontend_pre_styled_ui::button::button`] の
//! 既定 `type="button"` のまま用い、送信先・検証・永続化を一切持たない
//! （`docs/policy/intentional-non-adoption.md` §3.25）。
//!
//! # 成功・失敗の 2 状態を並記する
//!
//! Issue 仕様の「送信結果は成功と失敗の 2 状態を並記」に従い、状態は
//! `badge` の色（`ColorPalette::Success`/`Danger`）だけでなく文言
//! （「成功」/「失敗」）でも判別できるようにする（色のみに依存しない
//! A11y 配慮）。
//!
//! # `class` と `data-*` の使い分け
//!
//! `native_select::native_select` は `drop_class_attr` で呼び出し側
//! `class` を除去してから内部 variant クラスと合成するため、CSS フックは
//! `data-blocks-settings-webhook-tester-*` 属性で渡す。素の `div`/`pre`/
//! `p`/`h3`/`h4` は `class="blocks-settings-webhook-tester-*"` を使う。
//!
//! # ダミー素材について
//!
//! 宛先 URL は `example.com` ドメイン、イベント種類・応答内容・エラー値は
//! すべて架空のものであり、実在のサービス・企業・PII・実クレデンシャル
//! 形式を含まない。署名シークレット等のクレデンシャル様の値は出力しない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-webhook-tester/",
    title: "settings-webhook-tester",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_webhook_tester.rs",
    demo_class: "blocks-settings-webhook-tester",
    parts: &[
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Code",
            path: "/themes/code/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_webhook_tester` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型。ブレークポイントを持たず常に
/// 縦積みのまま固定する、モジュール doc「常に縦積み」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-settings-webhook-tester-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  max-width: 42rem;\n  margin-inline: auto;\n}\n\
.blocks-settings-webhook-tester-lead {\n  color: var(--fandhe-color-fg-muted);\n  margin: 0;\n}\n\
.blocks-settings-webhook-tester-fields {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-webhook-tester-actions {\n  display: flex;\n}\n\
.blocks-settings-webhook-tester-results {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-webhook-tester-result {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  padding: var(--fandhe-space-4);\n}\n\
.blocks-settings-webhook-tester-result-heading {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  flex-wrap: wrap;\n}\n\
.blocks-settings-webhook-tester-code {\n  margin: 0;\n  padding: var(--fandhe-space-3);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  overflow-x: auto;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        assert!(html.contains("data-scope=\"field\" data-part=\"select\""));
        assert!(html.contains("data-scope=\"badge\""));
        assert!(html.contains("data-scope=\"button\""));
        assert!(html.contains("<code"));
        assert_eq!(html.matches("<select").count(), 2);
    }

    #[test]
    fn no_form_submit_action_or_script() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("action="));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn exactly_one_type_button() {
        let html = demo_html();
        assert_eq!(html.matches("type=\"button\"").count(), 1);
    }

    #[test]
    fn selects_each_have_one_selected_option() {
        let html = demo_html();
        assert_eq!(html.matches("selected=\"selected\"").count(), 2);
    }

    #[test]
    fn success_and_failure_states_are_both_present() {
        let html = demo_html();
        assert!(html.contains("成功"));
        assert!(html.contains("失敗"));
        assert_eq!(html.matches("data-scope=\"badge\"").count(), 2);
    }

    #[test]
    fn root_class_differs_from_block_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-settings-webhook-tester-layout\""));
    }

    #[test]
    fn layout_css_is_safe() {
        assert!(!LAYOUT_CSS.contains('<'));
    }
}

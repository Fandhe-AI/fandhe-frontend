//! `settings-event-accordion` block（イシュー #2986。Application / Settings
//! カテゴリ、最初の block）。Webhook 配信のイベントログを想定した合成例:
//! イベント 1 件 = accordion 1 項目とし、見出し行に状態バッジ・種類・
//! 日時を並べ、展開部にリクエスト/レスポンスのペイロードをタブ風に切り
//! 替えて表示する。主参照 R0378。`_/blocks-intake/` の対応ファイルは本
//! イシュー着手時点で本 worktree に存在しないため、原稿・本コメントには
//! 対応表 ID のみを記す（[`super::super::profile::profile_detail_datalist`]
//! と同じ扱い）。
//!
//! # 使用部品
//!
//! `accordion` / `code` / `badge` / `heading` / `text` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。新しい UI 部品は追加しない。
//!
//! # アコーディオンは全件 open + disabled（原案との差分）
//!
//! [`super::super::faq::faq_tabbed_accordion`]・
//! [`super::super::changelog::changelog_accordion`] と同じ判断で、3 件の
//! イベントすべてを `OpenState::Open` + `AccordionProps { disabled: true,
//! .. }` で固定描画する。イシューの「先頭 1 件のみ展開」表示は、無 JS の
//! docs サイトでは閉じた項目の本文が `hidden` で到達不能になり、かつ
//! 押しても開かないボタンを残すことになるため採らない（件数を 3 件に
//! 抑えて全件展開でも読める分量にした）。差分は
//! `site/blocks/settings-event-accordion.md` の「原案差分メモ」節にも記す。
//!
//! # 実物の `tabs::tabs` は使わない
//!
//! [`super::super::faq::faq_tabbed_accordion`]（PR #3268 レビュー是正）と
//! 同じ理由で、リクエスト/レスポンスの切り替えに実物の `tabs::tabs` を
//! 使わない。非選択パネルが `hidden` で到達不能になるためである。代わりに
//! [`static_tab_list`] が `role`/`tabindex`/`<button>` を一切持たない
//! 非対話な `div` 列で選択状態を装飾として示し（`data-state="active|
//! inactive"`、`aria-hidden="true"`）、リクエスト・レスポンス両方の
//! ペイロードを見出し付きで常時可視にする。実物を呼ばない `tabs` は
//! `Block.parts` に含めない。
//!
//! # id の一意性
//!
//! accordion の id は
//! `blocks-settings-event-accordion-{index}-{trigger|content}`、静的タブ
//! 列の id は `blocks-settings-event-accordion-{index}-tablist` とし、3 件
//! を通じて重複しない（`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` が全 block
//! 横断で検証する）。
//!
//! # ダミーデータについて
//!
//! イベント id（`evt_0001` 等）・エンドポイント（`example.com` ドメイン）・
//! 金額はすべて架空である。API キー・署名・トークンを模した文字列は
//! 一切含まない（`crate::blocks` モジュール doc「セキュリティ不変条件」
//! 節、REQ-1 とは独立にペイロード内容自体を無害化する判断）。
//!
//! # `<form>` を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::code::{code, CodeProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// イベント 1 件分の型（種類・状態バッジの色 palette・状態ラベル・日時・
/// リクエスト JSON・レスポンス JSON）。架空データのみで構成する
/// （モジュール doc「ダミーデータについて」節）。
struct Event {
    event_type: &'static str,
    status_label: &'static str,
    status_palette: ColorPalette,
    occurred_at: &'static str,
    request_json: &'static str,
    response_json: &'static str,
}

/// 架空の Webhook 配信イベント 3 件（成功・失敗・再試行中）。
const EVENTS: [Event; 3] = [
    Event {
        event_type: "order.created",
        status_label: "成功",
        status_palette: ColorPalette::Success,
        occurred_at: "2026-09-28T10:12:03Z",
        request_json: "{\n  \"event\": \"order.created\",\n  \"order_id\": \"ord_1001\",\n  \"amount\": 4800,\n  \"currency\": \"JPY\"\n}",
        response_json: "{\n  \"status\": 200,\n  \"received\": true\n}",
    },
    Event {
        event_type: "payment.failed",
        status_label: "失敗",
        status_palette: ColorPalette::Danger,
        occurred_at: "2026-09-28T11:03:47Z",
        request_json: "{\n  \"event\": \"payment.failed\",\n  \"order_id\": \"ord_1002\",\n  \"reason\": \"card_declined\"\n}",
        response_json: "{\n  \"status\": 502,\n  \"error\": \"endpoint_unreachable\"\n}",
    },
    Event {
        event_type: "shipment.updated",
        status_label: "再試行中",
        status_palette: ColorPalette::Warning,
        occurred_at: "2026-09-28T12:45:19Z",
        request_json: "{\n  \"event\": \"shipment.updated\",\n  \"order_id\": \"ord_1003\",\n  \"tracking_no\": \"trk_5580\"\n}",
        response_json: "{\n  \"status\": 429,\n  \"error\": \"rate_limited\"\n}",
    },
];

/// ラベル・値の見出し行（`heading()` は `drop_class_attr` で呼び出し側
/// `class` を除去するため、色付けは呼ばない。ここでは中立見出しのみ）。
fn intro() -> Node {
    div(
        vec![("class", "blocks-settings-event-accordion-intro")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("Webhook 配信のイベントログ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("直近の配信結果を一覧表示します（架空データ）。")],
            ),
        ],
    )
}

/// 見出し行（状態バッジ・種類・日時）。日時は `margin-inline-start: auto`
/// で右寄せする（[`LAYOUT_CSS`] 参照）。
fn event_header(event: &Event) -> Node {
    div(
        vec![("class", "blocks-settings-event-accordion-header")],
        vec![
            badge(
                &BadgeProps {
                    variant: BadgeVariant::Subtle,
                    palette: event.status_palette,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text(event.status_label)],
            ),
            code(&CodeProps::default(), vec![], vec![text(event.event_type)]),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("class", "blocks-settings-event-accordion-time")],
                vec![text(event.occurred_at)],
            ),
        ],
    )
}

/// カテゴリラベルのみを装飾として示す非対話タブ列（モジュール doc「実物の
/// `tabs::tabs` は使わない」節）。`role`/`tabindex`/`<button>` を一切持た
/// ず、クリック・キーボード操作が可能に見えるトリガーを残さない。
fn static_tab_list(tablist_id: &str) -> Node {
    el(
        "div",
        vec![
            ("class", "blocks-settings-event-accordion-tablist"),
            ("id", tablist_id),
        ],
        vec![
            el(
                "div",
                vec![
                    ("class", "blocks-settings-event-accordion-tab"),
                    ("data-state", "active"),
                    ("aria-hidden", "true"),
                ],
                vec![text("リクエスト")],
            ),
            el(
                "div",
                vec![
                    ("class", "blocks-settings-event-accordion-tab"),
                    ("data-state", "inactive"),
                    ("aria-hidden", "true"),
                ],
                vec![text("レスポンス")],
            ),
        ],
    )
}

/// 見出し + JSON ペイロードの 1 ブロック（`pre` は横スクロール、
/// [`LAYOUT_CSS`] 参照）。
fn payload_block(caption: &str, json: &'static str) -> Node {
    div(
        vec![],
        vec![
            el(
                "h5",
                vec![("class", "blocks-settings-event-accordion-payload-heading")],
                vec![text(caption)],
            ),
            el(
                "pre",
                vec![("class", "blocks-settings-event-accordion-pre")],
                vec![code(
                    &CodeProps::default(),
                    vec![("data-blocks-settings-event-accordion-code", "")],
                    vec![text(json)],
                )],
            ),
        ],
    )
}

/// イベント 1 件分の accordion item（トリガー + 本文）。全件を
/// [`OpenState::Open`] + `disabled: true` で固定する（モジュール doc
/// 「アコーディオンは全件 open + disabled」節）。
fn event_item(index: usize, event: &Event) -> Node {
    let state = OpenState::Open;
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id = format!("blocks-settings-event-accordion-{index}-trigger");
    let content_id = format!("blocks-settings-event-accordion-{index}-content");
    let tablist_id = format!("blocks-settings-event-accordion-{index}-tablist");

    item(
        state,
        false,
        &props,
        vec![],
        vec![
            el(
                "h4",
                vec![("class", "blocks-settings-event-accordion-trigger-heading")],
                vec![item_trigger(
                    state,
                    false,
                    &props,
                    event.event_type,
                    Some(trigger_id.as_str()),
                    Some(content_id.as_str()),
                    vec![],
                    vec![
                        event_header(event),
                        item_indicator(state, false, &props, vec![], vec![text("▾")]),
                    ],
                )],
            ),
            item_content(
                state,
                false,
                &props,
                Some(content_id.as_str()),
                Some(trigger_id.as_str()),
                vec![],
                vec![
                    static_tab_list(&tablist_id),
                    payload_block("リクエスト", event.request_json),
                    payload_block(
                        "「レスポンス」タブを選択した場合のプレビュー",
                        event.response_json,
                    ),
                ],
            ),
        ],
    )
}

/// `settings-event-accordion` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。
pub fn demo() -> Node {
    let items: Vec<Node> = EVENTS
        .iter()
        .enumerate()
        .map(|(index, event)| event_item(index, event))
        .collect();
    div(
        vec![("class", "blocks-settings-event-accordion-layout")],
        vec![
            intro(),
            accordion::root(
                Size::Md,
                &AccordionProps::default(),
                vec![("data-blocks-settings-event-accordion-root", "")],
                items,
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-event-accordion/",
    title: "settings-event-accordion",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_event_accordion.rs",
    demo_class: "blocks-settings-event-accordion",
    parts: &[
        Part {
            label: "Accordion",
            path: "/themes/accordion/",
        },
        Part {
            label: "Code",
            path: "/themes/code/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_event_accordion` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同型）。
///
/// `[data-part="item-trigger"][data-disabled]` の中和（disabled を薄く
/// 見せない）は `faq_tabbed_accordion`/`changelog_accordion` と同型の
/// 判断。狭幅ではタブ列・見出し行が折り返し、`pre` のみが横スクロール
/// する。
const LAYOUT_CSS: &str = "\
.blocks-settings-event-accordion-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-settings-event-accordion-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-settings-event-accordion-trigger-heading {\n  margin: 0;\n  font-size: inherit;\n  font-weight: inherit;\n  width: 100%;\n}\n\
.blocks-settings-event-accordion-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  width: 100%;\n}\n\
.blocks-settings-event-accordion-time {\n  margin-inline-start: auto;\n}\n\
.blocks-settings-event-accordion-tablist {\n  display: flex;\n  gap: var(--fandhe-space-2);\n  overflow-x: auto;\n  white-space: nowrap;\n  border-bottom: 1px solid var(--fandhe-color-border);\n  margin-bottom: var(--fandhe-space-3);\n}\n\
.blocks-settings-event-accordion-tab {\n  display: inline-flex;\n  align-items: center;\n  flex-shrink: 0;\n  padding: var(--fandhe-space-2) var(--fandhe-space-3);\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n  border-bottom: 2px solid transparent;\n  margin-bottom: -1px;\n  cursor: default;\n}\n\
.blocks-settings-event-accordion-tab[data-state=\"active\"] {\n  color: var(--fandhe-color-fg);\n  border-bottom-color: var(--fandhe-color-accent);\n}\n\
.blocks-settings-event-accordion-payload-heading {\n  margin: 0 0 var(--fandhe-space-1) 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-event-accordion-pre {\n  overflow-x: auto;\n  margin: 0 0 var(--fandhe-space-4) 0;\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-settings-event-accordion-layout [data-scope=\"accordion\"][data-part=\"item-trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@container (max-width: 40rem) {\n  \
.blocks-settings-event-accordion-header {\n    flex-direction: column;\n    align-items: flex-start;\n  }\n  \
.blocks-settings-event-accordion-time {\n    margin-inline-start: 0;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, EVENTS, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"accordion\"",
            "data-scope=\"code\"",
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn no_form_submit_or_dead_links_or_scripts() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains(r#"role="tab""#));
    }

    #[test]
    fn all_accordion_items_are_open_and_disabled() {
        let html = demo_html();
        assert_eq!(
            html.matches(r#"data-part="item" data-state="open""#)
                .count(),
            EVENTS.len(),
            "html={html}"
        );
        assert_eq!(
            html.matches("item-content\" data-state=\"closed\"").count(),
            0
        );
        assert_eq!(
            html.matches(r#"aria-disabled="true""#).count(),
            EVENTS.len()
        );
        assert!(!html.contains("hidden=\"\""));
    }

    #[test]
    fn ids_are_unique_across_items() {
        let html = demo_html();
        let mut seen = std::collections::HashSet::new();
        for index in 0..EVENTS.len() {
            for suffix in ["trigger", "content", "tablist"] {
                let id = format!("blocks-settings-event-accordion-{index}-{suffix}");
                assert!(html.contains(&id), "missing id: {id}");
                assert!(seen.insert(id.clone()), "duplicate id: {id}");
            }
        }
    }

    #[test]
    fn all_event_data_is_reachable() {
        // JSON ペイロードは既定エスケープを経由するため `"` は `&quot;` に
        // 置き換わる（REQ-1）。素の JSON 文字列一致ではなく、エスケープ後
        // 形として比較する。
        let html = demo_html();
        for event in &EVENTS {
            assert!(html.contains(event.event_type));
            assert!(html.contains(event.status_label));
            assert!(html.contains(event.occurred_at));
            let escaped_request = fandhe_frontend_core::escape_html(event.request_json);
            let escaped_response = fandhe_frontend_core::escape_html(event.response_json);
            assert!(
                html.contains(&escaped_request),
                "html should contain escaped request payload"
            );
            assert!(
                html.contains(&escaped_response),
                "html should contain escaped response payload"
            );
        }
    }

    #[test]
    fn layout_css_is_safe_and_neutralizes_disabled() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS
            .contains(r#"[data-scope="accordion"][data-part="item-trigger"][data-disabled] {"#));
        assert!(LAYOUT_CSS.contains("overflow-x: auto;"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-settings-event-accordion-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-settings-event-accordion-layout"
        );
    }
}

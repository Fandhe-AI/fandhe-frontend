# settings-event-accordion

`fandhe-frontend-pre-styled-ui` の `accordion` / `code` / `badge` / `heading` /
`text` 部品を合成した、展開式の Webhook 配信イベントログです。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives 部品を
組み合わせた実例集であることに注意してください（主参照は対応表 ID R0378。
出典の固有名・ファイル名は記載しません）。

イベント 1 件を accordion 1 項目として表示します。見出し行には状態バッジ
（成功・失敗・再試行中）・イベント種類（`order.created` 等のインライン
コード表示）・発生日時を並べ、展開部にはリクエスト/レスポンス両方の
ペイロードを常時表示します。無 JS の docs サイトでは実物の `tabs` で
パネルを切り替える経路が作れないため、リクエスト/レスポンスの切り替えは
見た目のみを示す非対話のタブ列（`role`/`tabindex`/`<button>` を持たず、
クリック・キーボード操作はできません）で装飾し、実際のペイロードは
見出し付きで両方とも常時可視にしています。

本 Demo は静的な表示例です。3 件のイベントは全項目を常時展開（open）した
状態で固定し、`disabled` により開閉操作自体を無効化しています（理由は
「原案差分メモ」節を参照）。`<form>` 要素は一切持たず、データの取得・送信・
状態管理を行いません。イベント id・エンドポイント・金額はすべて独自に
書いた架空のものであり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
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

/// 見出し行（状態バッジ・種類・日時）。`item_trigger` が生成する
/// `<button>` の内部に配置されるため、`<button>` の内容モデル
/// （phrasing content 限定）を満たす `span` で組む（`div`・`styled_text::
/// text`〔`<p>`〕は不可。Codex P1・Bugbot Medium 是正）。日時は
/// `margin-inline-start: auto` で右寄せする（[`LAYOUT_CSS`] 参照）。
/// `styled_text::text` は `drop_class_attr` で呼び出し側 `class` を除去
/// するため（`faq_tabbed_accordion::category_caption` と同型の制約）、
/// 日時のミュート色・右寄せクラスは素の `span` へ直接付与し、色は
/// [`LAYOUT_CSS`] の `.blocks-settings-event-accordion-time` へ埋め込む
/// （Bugbot Medium「クラスが届かず margin-inline-start:auto が効かない」
/// 是正と同根）。
fn event_header(event: &Event) -> Node {
    span(
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
            span(
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
```

## 原案差分メモ

- イシューの原案は「先頭 1 件のみ展開・残りは閉」でしたが、無 JS の docs
  サイトでは閉じた項目のトリガーがクリック・Enter/Space に反応しない
  フォーカス可能な `<button>` として残り、本文が事実上到達不能になるため
  採用していません（`faq-tabbed-accordion` 等、先行する block が受けた
  指摘と同じ判断）。件数を 3 件に抑え、全件展開（open + disabled）でも
  読める分量にしています。
- リクエスト/レスポンスの切り替えには実物の `tabs::tabs` を使っていません。
  非選択パネルが `hidden` になり無 JS の docs サイトでは到達不能になるため
  です（`faq-tabbed-accordion` の是正と同じ判断）。代わりに非対話の視覚的
  タブ列を装飾として置き、両方のペイロードを見出し付きで常時可視にして
  います。
- イベント種類・状態・日時・ペイロードはすべて独自に書いた架空のものです。
  API キー・署名・トークンを模した文字列は含みません。
- 配色・余白・角丸は既存のテーマトークンに従っています。

関連情報: [Accordion](../themes/accordion.md) / [Code](../themes/code.md) /
[Badge](../themes/badge.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md)

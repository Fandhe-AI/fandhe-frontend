//! `api-reference-playground` block（イシュー #3102。親トラッキング #3099
//! 「Blocks Docs 区分拡充」配下、API リクエスト/レスポンスパネルの合成例）。
//!
//! # 使用部品
//!
//! `badge` / `code` / `clipboard` / `select` / `button` / `text` /
//! `empty-state` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。`visually_hidden` は補助的な合成手段のため `parts` には
//! 列挙しない（他 block と同じ判断、`hero_install_command` 等の先例）。
//!
//! # 4 版の併記
//!
//! API リクエスト/レスポンスパネルの構成差分を 1 つの Demo 内に静的に
//! 並記する（`profile_detail_datalist` 等、他 block の「差分は別
//! インスタンスで併記」パターンと同型。代表版を先頭に置く順序も同じ先例
//! に従う）。
//!
//! - **A（代表・レスポンス）**: 見出し + 3 バッジ（状態コード・所要時間・
//!   サイズ）+ コピー。本文は JSON を行番号付きで最大 12 行相当の高さに
//!   抑えてスクロールさせ、2 行を強調表示する。
//! - **B（リクエスト・インストール例）**: メソッドバッジ + 経路 + 言語選択
//!   （閉じたまま固定）+ コピー。フッター右寄せに「Try it」ボタンを置く。
//! - **C（リクエスト + エラー）**: 見出し + 3 バッジ（4xx・所要時間・
//!   サイズ）+ コピー。本文の下にエラー文を置く。
//! - **D（未送信）**: ヘッダーはメソッドバッジ + 経路のみ。本文の代わりに
//!   中央へ送信ボタン 1 つだけを置く。
//!
//! # 無 JS の静的表示であること
//!
//! docs サイトは JS を使わない（`crates/docs-site` モジュール doc の
//! 既存制約）。このため言語選択・コピー・送信・Try it の各操作は実際には
//! 動かない初期状態の固定表示に留める。[`fandhe_frontend_pre_styled_ui::
//! select::SelectProps`] へ `disabled: true` を渡し `OpenState::Closed`
//! で固定する（`card_form_footer::closed_select` と同型）。送信・Try it
//! ボタンは [`fandhe_frontend_pre_styled_ui::button::ButtonProps`] へ
//! `disabled: true` を渡し、押せそうに見せない。
//!
//! # `drop_class_attr` の契約（CSS フックの選び方）
//!
//! `badge::badge`/`code::code`/`clipboard::root`/`select::root`/
//! `button::button`/`text::text`/`empty_state::root` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、部品へのスタイルフックは
//! `data-blocks-api-reference-playground-*` 属性で渡す。素の `div`/`pre`/
//! `span` は `class` がそのまま効くため `.blocks-api-reference-playground-*`
//! クラスセレクタを使う。
//!
//! # 行番号・強調行（CSS カウンタ）
//!
//! 本文の各行は `span[data-blocks-api-reference-playground-line]` として
//! `code::code` の子に並べ、行番号は CSS カウンタ（`counter-increment`/
//! `::before { content: counter(line) }`）で付与する。選択・コピー時に
//! 行番号の文字列が本文テキストへ混ざらず、`render` 出力にも増えない
//! （[`LAYOUT_CSS`] 側の実装）。強調行は `data-highlighted` 属性 +
//! 背景色 + `border-inline-start` の両方で示し、色だけに頼らない。
//! スクロール領域（`pre`）はキーボードで届くよう `tabindex="0"` と
//! `role="region"`・`aria-label` を付与する。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンはすべて `type="button"`（`button::button` の既定）。
//!
//! # API 経路・値はすべて架空
//!
//! 経路・トークン・ホスト名はすべて架空（`api.example.com` 等の予約
//! ドメイン、実在の企業名・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps, EmptyStateVariant};
use fandhe_frontend_pre_styled_ui::select::{self, OpenState, SelectProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// メソッドバッジ（`GET`/`POST` 等。文字そのものを表示し、色だけに頼らない）。
fn method_badge(method: &'static str, palette: ColorPalette) -> Node {
    badge::badge(
        &BadgeProps {
            palette,
            ..BadgeProps::default()
        },
        vec![("data-blocks-api-reference-playground-method", "")],
        vec![text(method)],
    )
}

/// 経路の等幅表示（`code::code`）。
fn endpoint(path: &'static str) -> Node {
    code::code(
        &CodeProps::default(),
        vec![("data-blocks-api-reference-playground-endpoint", "")],
        vec![text(path)],
    )
}

/// 状態コード・所要時間・サイズの 3 バッジ（モジュール doc「4 版の併記」
/// 節）。状態コードの palette は呼び出し側が 2xx → Success・4xx → Danger
/// を選んで渡す。
fn meta_badges(
    status_label: &'static str,
    status_palette: ColorPalette,
    duration_label: &'static str,
    size_label: &'static str,
) -> Node {
    div(
        vec![("data-blocks-api-reference-playground-meta", "")],
        vec![
            badge::badge(
                &BadgeProps {
                    palette: status_palette,
                    variant: BadgeVariant::Solid,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text(status_label)],
            ),
            badge::badge(
                &BadgeProps {
                    variant: BadgeVariant::Outline,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text(duration_label)],
            ),
            badge::badge(
                &BadgeProps {
                    variant: BadgeVariant::Outline,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text(size_label)],
            ),
        ],
    )
}

/// 閉じたまま固定する言語選択（モジュール doc「無 JS の静的表示である
/// こと」節。`card_form_footer::closed_select` と同型の構造。
/// `positioner`/`content` は `hidden` のまま出力し、`aria-controls`/
/// `aria-labelledby` の参照先を宙に浮かせない）。
fn language_select(
    label_id: &'static str,
    content_id: &'static str,
    selected_label: &'static str,
    options: &[(&'static str, &'static str, bool)],
) -> Node {
    let props = SelectProps {
        disabled: true,
        ..SelectProps::default()
    };
    let items: Vec<Node> = options
        .iter()
        .map(|(value, label, selected)| {
            let state = if *selected {
                OpenState::Open
            } else {
                OpenState::Closed
            };
            select::item(
                state,
                &props,
                false,
                false,
                value,
                None,
                vec![],
                vec![select::item_text(
                    state,
                    &props,
                    false,
                    false,
                    None,
                    vec![],
                    vec![text(*label)],
                )],
            )
        })
        .collect();
    div(
        vec![("data-blocks-api-reference-playground-select", "")],
        vec![
            visually_hidden::root(
                vec![],
                vec![select::label(
                    &props,
                    Some(label_id),
                    vec![],
                    vec![text("言語")],
                )],
            ),
            select::root(
                Size::Sm,
                OpenState::Closed,
                &props,
                vec![],
                vec![
                    select::control(
                        OpenState::Closed,
                        &props,
                        vec![],
                        vec![select::trigger(
                            OpenState::Closed,
                            &props,
                            false,
                            Some(content_id),
                            Some(label_id),
                            vec![],
                            vec![
                                select::value_text(
                                    false,
                                    &props,
                                    vec![],
                                    vec![text(selected_label)],
                                ),
                                select::indicator(OpenState::Closed, &props, vec![], vec![]),
                            ],
                        )],
                    ),
                    select::positioner(
                        OpenState::Closed,
                        vec![],
                        vec![select::content(
                            OpenState::Closed,
                            Some(content_id),
                            Some(label_id),
                            None,
                            vec![],
                            items,
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// コピー操作（idle 初期状態）。モジュール doc「無 JS の静的表示である
/// こと」節: docs サイト自体は無 JS のため静的表示に留まるが、実アプリへ
/// 組み込めば `headless_clipboard` 配線によりコピー操作は機能する
/// （`hero_install_command` と同型の判断）。
fn copy_button(value: &'static str, input_id: &'static str) -> Node {
    clipboard::root(
        value,
        false,
        vec![("data-blocks-api-reference-playground-copy", "")],
        vec![
            visually_hidden::root(
                vec![],
                vec![clipboard::label(
                    false,
                    Some(input_id),
                    vec![],
                    vec![text("値をコピー")],
                )],
            ),
            clipboard::control(
                false,
                vec![],
                vec![
                    clipboard::input(value, false, vec![("id", input_id)]),
                    clipboard::trigger(
                        false,
                        vec![],
                        vec![
                            clipboard::indicator(false, false, vec![], vec![text("Copy")]),
                            clipboard::indicator(true, false, vec![], vec![text("Copied")]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 行番号付きコード本文（モジュール doc「行番号・強調行（CSS カウンタ）」
/// 節）。`highlighted` は 0 始まりの強調行インデックス集合。
fn code_body(
    body_attr: &'static str,
    aria_label: &'static str,
    lines: &[&'static str],
    highlighted: &[usize],
) -> Node {
    let line_nodes: Vec<Node> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let mut attrs = vec![("data-blocks-api-reference-playground-line", "")];
            if highlighted.contains(&i) {
                attrs.push(("data-highlighted", ""));
            }
            span(attrs, vec![text(*line)])
        })
        .collect();
    el(
        "pre",
        vec![
            (body_attr, ""),
            ("role", "region"),
            ("aria-label", aria_label),
            ("tabindex", "0"),
        ],
        vec![code::code(
            &CodeProps::default(),
            vec![("data-blocks-api-reference-playground-body-code", "")],
            line_nodes,
        )],
    )
}

/// A（代表・レスポンス、R0063 対応）。
fn panel_response() -> Node {
    const LINES: [&str; 20] = [
        "{",
        "  \"id\": \"proj_8f2a1c\",",
        "  \"name\": \"storefront-api\",",
        "  \"status\": \"active\",",
        "  \"region\": \"us-east-1\",",
        "  \"owner\": {",
        "    \"id\": \"usr_41b6\",",
        "    \"email\": \"owner@api.example.com\"",
        "  },",
        "  \"endpoints\": [",
        "    \"/v1/projects\",",
        "    \"/v1/projects/{id}\"",
        "  ],",
        "  \"rate_limit\": {",
        "    \"limit\": 1000,",
        "    \"remaining\": 998",
        "  },",
        "  \"created_at\": \"2026-01-04T09:12:00Z\",",
        "  \"updated_at\": \"2026-03-11T15:40:22Z\"",
        "}",
    ];
    div(
        vec![
            ("data-blocks-api-reference-playground-panel", ""),
            ("data-blocks-api-reference-playground-variant", "response"),
        ],
        vec![
            div(
                vec![("data-blocks-api-reference-playground-header", "")],
                vec![
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Lg,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("Response")],
                    ),
                    meta_badges("200", ColorPalette::Success, "142 ms", "1.8 KB"),
                    copy_button(
                        "{\"id\":\"proj_8f2a1c\",\"status\":\"active\"}",
                        "blocks-api-reference-playground-response-copy",
                    ),
                ],
            ),
            code_body(
                "data-blocks-api-reference-playground-body",
                "Response body",
                &LINES,
                &[2, 3],
            ),
        ],
    )
}

/// B（リクエスト・インストール例、R0056 対応）。
fn panel_request_install() -> Node {
    const LANG_LABEL_ID: &str = "blocks-api-reference-playground-request-lang-label";
    const LANG_CONTENT_ID: &str = "blocks-api-reference-playground-request-lang-content";
    const OPTIONS: [(&str, &str, bool); 3] = [
        ("curl", "cURL", true),
        ("rust", "Rust", false),
        ("js", "JavaScript", false),
    ];
    const LINES: [&str; 4] = [
        "curl -X POST https://api.example.com/v1/projects \\",
        "  -H \"Authorization: Bearer <YOUR_API_TOKEN>\" \\",
        "  -H \"Content-Type: application/json\" \\",
        "  -d '{\"name\":\"storefront-api\"}'",
    ];
    div(
        vec![
            ("data-blocks-api-reference-playground-panel", ""),
            (
                "data-blocks-api-reference-playground-variant",
                "request-install",
            ),
        ],
        vec![
            div(
                vec![("data-blocks-api-reference-playground-header", "")],
                vec![
                    method_badge("POST", ColorPalette::Accent),
                    endpoint("/v1/projects"),
                    language_select(LANG_LABEL_ID, LANG_CONTENT_ID, "cURL", &OPTIONS),
                    copy_button(
                        "curl -X POST https://api.example.com/v1/projects",
                        "blocks-api-reference-playground-request-copy",
                    ),
                ],
            ),
            code_body(
                "data-blocks-api-reference-playground-body",
                "Request body",
                &LINES,
                &[],
            ),
            div(
                vec![("data-blocks-api-reference-playground-footer", "")],
                vec![button::button(
                    &ButtonProps {
                        disabled: true,
                        ..ButtonProps::default()
                    },
                    vec![("data-blocks-api-reference-playground-try", "")],
                    vec![text("Try it")],
                )],
            ),
        ],
    )
}

/// C（リクエスト + エラー、R0062 対応）。
fn panel_request_error() -> Node {
    const LINES: [&str; 5] = [
        "{",
        "  \"name\": \"\",",
        "  \"region\": \"mars-central-1\"",
        "}",
        "",
    ];
    div(
        vec![
            ("data-blocks-api-reference-playground-panel", ""),
            (
                "data-blocks-api-reference-playground-variant",
                "request-error",
            ),
        ],
        vec![
            div(
                vec![("data-blocks-api-reference-playground-header", "")],
                vec![
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Lg,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("Request")],
                    ),
                    meta_badges("400", ColorPalette::Danger, "86 ms", "0.3 KB"),
                    copy_button(
                        "{\"name\":\"\",\"region\":\"mars-central-1\"}",
                        "blocks-api-reference-playground-error-copy",
                    ),
                ],
            ),
            code_body(
                "data-blocks-api-reference-playground-body",
                "Request body",
                &LINES,
                &[1, 2],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-api-reference-playground-error", "")],
                vec![text(
                    "Error: \"region\" must be one of the supported regions.",
                )],
            ),
        ],
    )
}

/// D（未送信、R0061 対応）。
fn panel_unsent() -> Node {
    div(
        vec![
            ("data-blocks-api-reference-playground-panel", ""),
            ("data-blocks-api-reference-playground-variant", "unsent"),
        ],
        vec![
            div(
                vec![("data-blocks-api-reference-playground-header", "")],
                vec![
                    method_badge("GET", ColorPalette::Accent),
                    endpoint("/v1/projects"),
                ],
            ),
            empty_state::root(
                &EmptyStateProps {
                    variant: EmptyStateVariant::Plain,
                    ..EmptyStateProps::default()
                },
                vec![("data-blocks-api-reference-playground-unsent", "")],
                vec![empty_state::content(
                    vec![],
                    vec![empty_state::actions(
                        vec![],
                        vec![button::button(
                            &ButtonProps {
                                disabled: true,
                                ..ButtonProps::default()
                            },
                            vec![("data-blocks-api-reference-playground-send", "")],
                            vec![text("Send request")],
                        )],
                    )],
                )],
            ),
        ],
    )
}

pub fn demo() -> Node {
    div(
        vec![("class", "blocks-api-reference-playground-layout")],
        vec![
            panel_response(),
            panel_request_install(),
            panel_request_error(),
            panel_unsent(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/api-reference-playground/",
    title: "api-reference-playground",
    category: BlockCategory::ApiReference,
    rust_source: "crates/docs-site/src/blocks/docs/api_reference/api_reference_playground.rs",
    demo_class: "blocks-api-reference-playground",
    parts: &[
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Code",
            path: "/themes/code/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Select",
            path: "/themes/select/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Empty State",
            path: "/themes/empty-state/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `api_reference_playground` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節、他 block と同型で `pub(super)` ではなく
/// `super::stylesheet` から [`Block::layout_css`] 経由で連結される）。
const LAYOUT_CSS: &str = "\
.blocks-api-reference-playground-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-api-reference-playground-panel] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  padding: var(--fandhe-space-4);\n}\n\
[data-blocks-api-reference-playground-header] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-api-reference-playground-meta] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  margin-inline-start: auto;\n}\n\
[data-blocks-api-reference-playground-copy] {\n  margin-inline-start: auto;\n}\n\
[data-blocks-api-reference-playground-meta] + [data-blocks-api-reference-playground-copy] {\n  margin-inline-start: 0;\n}\n\
[data-blocks-api-reference-playground-select] {\n  margin-inline-start: auto;\n}\n\
[data-blocks-api-reference-playground-body] {\n  margin: 0;\n  overflow: auto;\n  max-height: calc(1.5em * 12);\n  line-height: 1.5;\n  counter-reset: line;\n  font-family: var(--fandhe-font-font-mono);\n  font-size: var(--fandhe-font-font-size-sm);\n  padding: var(--fandhe-space-3);\n  background: var(--fandhe-color-bg-subtle);\n  border-radius: var(--fandhe-radius-md);\n}\n\
[data-scope=\"code\"][data-part=\"root\"][data-blocks-api-reference-playground-body-code] {\n  display: block;\n  white-space: pre;\n  background: transparent;\n  color: inherit;\n  border: 0;\n  padding: 0;\n}\n\
[data-blocks-api-reference-playground-line] {\n  display: block;\n  position: relative;\n  counter-increment: line;\n  padding-inline-start: 2.5em;\n}\n\
[data-blocks-api-reference-playground-line]::before {\n  content: counter(line);\n  position: absolute;\n  inset-inline-start: 0;\n  width: 2em;\n  text-align: right;\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-api-reference-playground-line][data-highlighted] {\n  background: var(--fandhe-color-accent-subtle);\n  border-inline-start: 2px solid var(--fandhe-color-accent-fg-subtle);\n  margin-inline-start: -2px;\n  padding-inline-start: calc(2.5em - 2px);\n}\n\
[data-blocks-api-reference-playground-error] {\n  margin: 0;\n  color: var(--fandhe-color-danger-fg-subtle);\n}\n\
[data-blocks-api-reference-playground-unsent] {\n  min-height: 8rem;\n  display: grid;\n  place-items: center;\n}\n\
[data-blocks-api-reference-playground-footer] {\n  display: flex;\n  justify-content: flex-end;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// [`LAYOUT_CSS`] が行番号・高さ制限・強調行セレクタを含み、`<` を
    /// 含まないこと（REQ-1）。
    #[test]
    fn layout_css_declares_line_numbers_and_height_limit() {
        assert!(LAYOUT_CSS.contains("max-height: calc(1.5em * 12)"));
        assert!(LAYOUT_CSS.contains("counter-increment: line"));
        assert!(
            LAYOUT_CSS.contains("[data-blocks-api-reference-playground-line][data-highlighted]")
        );
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// Demo が `<form>` を出力せず、押せないボタン・select が明示されて
    /// いること。
    #[test]
    fn demo_has_no_form_and_disabled_controls() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains("data-copied"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        // Try it / Send request ボタンと select はいずれも disabled。
        assert!(html.matches("disabled").count() >= 3);
    }

    /// 4 版すべてが固有の `data-blocks-api-reference-playground-variant`
    /// マーカーを持つこと（モジュール doc「4 版の併記」節）。
    #[test]
    fn demo_has_all_four_variant_markers() {
        let html = render(&demo());
        for variant in ["response", "request-install", "request-error", "unsent"] {
            assert!(
                html.contains(&format!(
                    "data-blocks-api-reference-playground-variant=\"{variant}\""
                )),
                "missing variant marker: {variant}"
            );
        }
    }

    /// 強調行が 1 つ以上存在すること。
    #[test]
    fn demo_has_highlighted_lines() {
        let html = render(&demo());
        assert!(html.matches("data-highlighted").count() >= 1);
    }
}

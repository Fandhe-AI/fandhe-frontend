//! `api-reference-playground` block（イシュー #3102。親トラッキング #3099
//! 「Blocks Docs 区分拡充」配下、API リクエスト/レスポンスパネルの合成例）。
//!
//! # 使用部品
//!
//! `badge` / `code` / `select` / `button` / `text` / `empty-state` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `visually_hidden` は補助的な合成手段のため `parts` には列挙しない
//! （他 block と同じ判断、`hero_install_command` 等の先例）。イシュー #3102
//! の使用部品案にあった `clipboard` は本 Demo では使わない（下記「Demo 内に
//! `clipboard` root を置かない」節）。
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
//! で固定する（`card_form_footer::closed_select` と同型）。コピー・送信・
//! Try it の各ボタンは [`fandhe_frontend_pre_styled_ui::button::ButtonProps`]
//! へ `disabled: true` を渡し、押せそうに見せない（押せる見た目で押しても
//! 反応しないボタンを Demo に残さない）。
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
//! `code::code` の子に並べ、最終行以外の各行末に実際の改行テキスト（`\n`）を
//! 置く。改行の手段はこの `\n` と親 `code` の `white-space: pre` の 1 つに
//! 統一し、行 span は `display: block` にしない（block 化と `\n` を重ねると
//! 行間に空行が入り、block 化だけに頼ると選択コピーで行が連結されるため）。
//! 行番号は CSS カウンタ（`counter-increment`/`::before { content:
//! counter(line) }`）で付与し、`::before` を固定幅の `inline-block` として
//! 各行の先頭に並べる。生成コンテンツのため選択・コピー時に行番号の文字列が
//! 本文テキストへ混ざらず、`render` 出力にも増えない（[`LAYOUT_CSS`] 側の
//! 実装）。強調行は `data-highlighted` 属性 + 背景色 +
//! `border-inline-start` の色の両方で示し、色だけに頼らない。行頭ボーダーは
//! 全行に同じ幅（2px）で確保し、通常行は透明・強調行だけ色を付ける
//! （強調行の有無で行番号・本文の横位置がずれないため）。スクロール領域
//! （`pre`）はキーボードで届くよう `tabindex="0"` と `role="region"`・
//! `aria-label` を付与する。
//!
//! # 版 C のエラー文は `TextVariant::Plain` のまま block CSS で色を付ける
//!
//! `text::text` の `Muted` 変種は recipe 側で `color` を宣言するため、
//! block 固有の属性セレクタ 1 個より詳細度が高く、danger 色が負ける。
//! そこで既定の `Plain`（`color` 宣言なし）を使い、[`LAYOUT_CSS`] の
//! `[data-blocks-api-reference-playground-error]` だけが `color` を決める。
//!
//! # Demo 内に `clipboard` root を置かない
//!
//! 本 Demo の主役は本文（コード）であり、コピーはヘッダーの付随操作である。
//! 無 JS の docs サイトで `clipboard::root` を置くと、(1) `clipboard::trigger`
//! は `disabled` を持たないため押せる見た目のまま反応しない「Copy」が残り、
//! (2) 本文と重複する `clipboard::input` を `visually_hidden` で隠しても
//! readonly input が Tab 順に残って見えない要素へフォーカスが止まる。
//! どちらも静的表示の見本として筋が通らないため、A・B・C の 3 つのコピーは
//! すべて `clipboard` scope の外側に `disabled: true` の `button::button` を
//! 置き（送信・Try it と同じ扱い）、`clipboard` root・input・`data-value` を
//! 出力しない。実アプリで本 block を使うときは [`copy_button`] を
//! `clipboard::root`（`settings_api_key_created`/`hero_install_command` の
//! 構成。コピー値は表示本文と同じ行配列を `\n` で連結する）へ置き換える。
//! 表示本文の唯一の供給元は [`RESPONSE_LINES`]/[`REQUEST_LINES`]/
//! [`ERROR_LINES`] の行配列である。
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
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps, EmptyStateVariant};
use fandhe_frontend_pre_styled_ui::select::{self, OpenState, SelectProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// A（レスポンス）の本文行。表示本文の唯一の供給元。
const RESPONSE_LINES: [&str; 20] = [
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

/// B（リクエスト）の本文行。
const REQUEST_LINES: [&str; 4] = [
    "curl -X POST https://api.example.com/v1/projects \\",
    "  -H \"Authorization: Bearer <YOUR_API_TOKEN>\" \\",
    "  -H \"Content-Type: application/json\" \\",
    "  -d '{\"name\":\"storefront-api\"}'",
];

/// C（リクエスト + エラー）の本文行。
const ERROR_LINES: [&str; 5] = [
    "{",
    "  \"name\": \"\",",
    "  \"region\": \"mars-central-1\"",
    "}",
    "",
];

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

/// コピー操作の静的表示（A・B・C 共通、押下不能）。モジュール doc「Demo 内に
/// `clipboard` root を置かない」節: 無 JS の docs サイトでは動かないため
/// `clipboard` scope の外側に `disabled: true` の `button::button` を置く。
/// 実アプリでは `clipboard::root` へ置き換える。
fn copy_button() -> Node {
    button::button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("data-blocks-api-reference-playground-copy", "")],
        vec![text("Copy")],
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
            let mut children = vec![text(*line)];
            if i + 1 < lines.len() {
                children.push(text("\n"));
            }
            span(attrs, children)
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
                    copy_button(),
                ],
            ),
            code_body(
                "data-blocks-api-reference-playground-body",
                "Response body",
                &RESPONSE_LINES,
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
                    copy_button(),
                ],
            ),
            code_body(
                "data-blocks-api-reference-playground-body",
                "Request body",
                &REQUEST_LINES,
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
                    copy_button(),
                ],
            ),
            code_body(
                "data-blocks-api-reference-playground-body",
                "Request body",
                &ERROR_LINES,
                &[1, 2],
            ),
            // 既定の `Plain` は `color` を宣言しないため、`LAYOUT_CSS` の danger 色が
            // そのまま効く（モジュール doc「版 C のエラー文」節）。
            styled_text::text(
                &TextProps::default(),
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
[data-blocks-api-reference-playground-select] + [data-blocks-api-reference-playground-copy] {\n  margin-inline-start: 0;\n}\n\
[data-blocks-api-reference-playground-body] {\n  margin: 0;\n  overflow: auto;\n  max-height: calc(1.5em * 12);\n  line-height: 1.5;\n  counter-reset: line;\n  font-family: var(--fandhe-font-font-mono);\n  font-size: var(--fandhe-font-font-size-sm);\n  padding: var(--fandhe-space-3);\n  background: var(--fandhe-color-bg-subtle);\n  border-radius: var(--fandhe-radius-md);\n}\n\
[data-scope=\"code\"][data-part=\"root\"][data-blocks-api-reference-playground-body-code] {\n  display: block;\n  white-space: pre;\n  background: transparent;\n  color: inherit;\n  border: 0;\n  padding: 0;\n}\n\
[data-blocks-api-reference-playground-line] {\n  counter-increment: line;\n  border-inline-start: 2px solid transparent;\n}\n\
[data-blocks-api-reference-playground-line]::before {\n  content: counter(line);\n  display: inline-block;\n  width: 2em;\n  margin-inline-end: 0.5em;\n  text-align: right;\n  color: var(--fandhe-color-fg-muted);\n  user-select: none;\n}\n\
[data-blocks-api-reference-playground-line][data-highlighted] {\n  background: var(--fandhe-color-accent-subtle);\n  border-inline-start-color: var(--fandhe-color-accent-fg-subtle);\n}\n\
[data-blocks-api-reference-playground-error] {\n  margin: 0;\n  color: var(--fandhe-color-danger-fg-subtle);\n}\n\
[data-blocks-api-reference-playground-unsent] {\n  min-height: 8rem;\n  display: grid;\n  place-items: center;\n}\n\
[data-blocks-api-reference-playground-footer] {\n  display: flex;\n  justify-content: flex-end;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, ERROR_LINES, LAYOUT_CSS, REQUEST_LINES, RESPONSE_LINES};
    use fandhe_frontend_core::{escape_html, render};

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
        // Copy x3 / Try it / Send request のボタンと select はいずれも disabled。
        assert!(html.matches("disabled").count() >= 6);
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

    /// 改行の手段が 1 つ（行末の `\n` + 親 `code` の `white-space: pre`）に
    /// 統一され、行 span を `display: block` にしないこと（block 化と `\n` の
    /// 重複による空行の防止。モジュール doc「行番号・強調行」節）。
    #[test]
    fn line_breaks_use_only_newline_text_without_block_lines() {
        let rule_start = LAYOUT_CSS
            .find("[data-blocks-api-reference-playground-line] {")
            .expect("line rule");
        let rule = &LAYOUT_CSS[rule_start..];
        let rule = &rule[..rule.find('}').expect("rule end")];
        assert!(
            !rule.contains("display"),
            "line span must stay inline: {rule}"
        );
        assert!(!LAYOUT_CSS.contains("position: absolute"));
        assert!(LAYOUT_CSS.contains("white-space: pre"));
        // 行 span の外側（行間）に余分な改行テキストを置かない。
        let html = render(&demo());
        assert!(!html.contains("</span>\n<span data-blocks-api-reference-playground-line"));
    }

    /// 各本文の行 span の中身を順に連結すると行配列の `\n` 連結に一致する
    /// こと（範囲選択コピーで行が連結されない）。
    #[test]
    fn displayed_bodies_keep_newlines() {
        let html = render(&demo());
        for lines in [&RESPONSE_LINES[..], &REQUEST_LINES[..], &ERROR_LINES[..]] {
            let joined = lines.join("\n");
            let mut displayed = String::new();
            for (i, line) in lines.iter().enumerate() {
                let newline = if i + 1 < lines.len() { "\n" } else { "" };
                let inner = format!("{}{newline}</span>", escape_html(line));
                assert!(html.contains(&inner), "line {i} must end with {newline:?}");
                displayed.push_str(line);
                displayed.push_str(newline);
            }
            assert_eq!(displayed, joined);
        }
    }

    /// Demo が `clipboard` root・input・コピー値を一切出力せず、A・B・C の
    /// コピーがすべて `disabled` の `button::button` であること（モジュール
    /// doc「Demo 内に `clipboard` root を置かない」節。押せる見た目で反応しない
    /// ボタンと、Tab 順に残る不可視 input の両方を排除する）。
    #[test]
    fn demo_has_no_clipboard_root_and_copy_buttons_are_disabled() {
        let html = render(&demo());
        assert!(!html.contains("data-scope=\"clipboard\""));
        // `data-value` は select の item も持つため、clipboard root 由来の
        // コピー値（`data-part="root" data-value=`）が無いことだけを見る。
        assert!(!html.contains("data-part=\"root\" data-value="));
        assert!(!html.contains("<input"));
        let marker = "data-blocks-api-reference-playground-copy";
        assert_eq!(html.matches(marker).count(), 3);
        for (i, _) in html.match_indices(marker) {
            let open = html[..i].rfind('<').expect("copy tag start");
            let close = i + html[i..].find('>').expect("copy tag end");
            let tag = &html[open..close];
            assert!(tag.starts_with("<button"), "copy must be a button: {tag}");
            assert!(tag.contains(" disabled"), "copy must be disabled: {tag}");
        }
        // 全要素のうち Tab 到達可能なのは disabled でない要素だけで、
        // `tabindex` を持つのはスクロール領域（`pre`）に限る。
        assert_eq!(
            html.matches("tabindex=").count(),
            html.matches("<pre").count()
        );
    }

    /// 全行が同じ幅（2px）の行頭ボーダーを持ち、強調行はその色だけを変える
    /// こと（強調行の有無で行番号・本文の横位置がずれない。モジュール doc
    /// 「行番号・強調行」節）。
    #[test]
    fn layout_css_keeps_line_gutter_width_constant_across_highlight() {
        let rule_start = LAYOUT_CSS
            .find("[data-blocks-api-reference-playground-line] {")
            .expect("line rule");
        let rule = &LAYOUT_CSS[rule_start..];
        let rule = &rule[..rule.find('}').expect("rule end")];
        assert!(rule.contains("border-inline-start: 2px solid transparent"));
        let hl_start = LAYOUT_CSS
            .find("[data-blocks-api-reference-playground-line][data-highlighted] {")
            .expect("highlight rule");
        let hl = &LAYOUT_CSS[hl_start..];
        let hl = &hl[..hl.find('}').expect("rule end")];
        assert!(hl.contains("border-inline-start-color:"));
        assert!(
            !hl.contains("border-inline-start:"),
            "highlight must not change border width: {hl}"
        );
    }

    /// 版 C のエラー文が `TextVariant::Muted` を使わず（recipe の `color` が
    /// block CSS に詳細度で勝つのを避ける）、block CSS が danger 色を宣言する
    /// こと（モジュール doc「版 C のエラー文」節）。
    #[test]
    fn error_text_is_plain_variant_colored_by_block_css() {
        let html = render(&demo());
        let at = html
            .find("data-blocks-api-reference-playground-error")
            .expect("error text");
        let open = html[..at].rfind('<').expect("tag start");
        let close = at + html[at..].find('>').expect("tag end");
        let tag = &html[open..close];
        assert!(
            tag.contains("data-scope=\"text\""),
            "error must be a text part: {tag}"
        );
        assert!(!tag.contains("muted"), "error must not be Muted: {tag}");
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-api-reference-playground-error] {\n  margin: 0;\n  color: var(--fandhe-color-danger-fg-subtle);\n}"
        ));
    }

    /// select と copy が並ぶヘッダーで copy 側の自動余白を打ち消し、右寄せの
    /// まとまりにすること。
    #[test]
    fn layout_css_resets_copy_margin_after_select() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-api-reference-playground-select] + [data-blocks-api-reference-playground-copy] {\n  margin-inline-start: 0;\n}"
        ));
    }
}

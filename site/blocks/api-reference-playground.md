# api-reference-playground

API のリクエスト/レスポンスパネルの合成例です。新規 UI 部品は作らず
`badge` / `code` / `select` / `button` / `text` / `empty-state` の 6 部品を
合成します。レスポンス・リクエスト・エラー・未送信の 4 版を 1 つの
Demo 内へ静的に並記して示します。docs サイトは JS を使わないため、
言語選択は閉じたまま固定し、コピー・送信・Try it の各ボタンは押せない
（`disabled`）状態で表示します。`<form>` は持たず、ボタンはすべて
`type="button"` です。API の経路・トークン・値はすべて架空のものです。

- **A（代表・レスポンス）**: 見出し「Response」+ 3 バッジ（状態コード・
  所要時間・サイズ）+ 押せないコピー。本文は JSON を行番号付きで表示し、
  12 行相当の高さを超える部分はスクロールします。2 行を強調表示します。
- **B（リクエスト・インストール例）**: メソッドバッジ + 経路 + 言語選択
  （閉じたまま固定）+ 押せないコピー。フッター右寄せに押せない「Try it」
  ボタンを置きます。
- **C（リクエスト + エラー）**: 見出し「Request」+ 3 バッジ（4xx・所要
  時間・サイズ）+ 押せないコピー。本文の下にエラー文を表示します。
- **D（未送信）**: ヘッダーはメソッドバッジ + 経路のみ。本文の代わりに
  中央へ押せない「Send request」ボタンを 1 つだけ置きます。

## Rust コード

```rust
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
```

## 原案差分メモ

- 版 A（代表）は R0063（レスポンス + 行番号 + 高さ制限）に対応します。
- 版 B は R0056（メソッド + 経路 + 言語選択 + 試行ボタン）に対応します。
- 版 C は R0062（リクエスト + 3 バッジ + エラー文）に対応します。
- 版 D は R0061（未送信状態・送信ボタンのみ）に対応します。
- 行番号は CSS カウンタ（`counter-increment`/`::before`）で付与して
  います。選択・コピー時に行番号の文字列が本文へ混ざりません。
- 本文の各行は、最終行以外の行末に実際の改行文字を持ちます。改行は
  この改行文字と `white-space: pre` だけで行い、行を `display: block`
  にしないため、表示に空行が入らず、選択してコピーしても行が連結
  されません。行番号は各行先頭の `::before`（`inline-block`）に出します。
- 全行が同じ幅の行頭ボーダー（通常行は透明）を持つため、強調行でも
  行番号と本文の横位置がずれません。
- 原案の使用部品にあった `clipboard` は使いません。docs サイトは JS を
  使わないため、`clipboard` を置くと押せる見た目のまま反応しないコピー
  ボタンと、Tab 順に残る不可視の入力欄が Demo に残るためです。A・B・C の
  コピーはいずれも `clipboard` の外側に置いた押せない（`disabled`）
  `button` で、コピー値（`data-value`）も出力しません。実アプリでは
  `copy_button` を `clipboard` へ置き換え、表示本文と同じ行配列を改行で
  連結した値をコピー値にします（`settings-api-key-created` /
  `hero-install-command` と同じ構成）。
- 版 C のエラー文は `text` の既定（`Plain`）で描画し、block 側の CSS で
  danger 色を当てています（`Muted` だと部品側の色指定が優先されるため）。
- 版 B のヘッダーでは言語選択とコピーを右寄せの 1 つのまとまりとして
  並べます（コピー側の自動余白を打ち消します）。
- 本文のスクロール領域は 12 行相当（`calc(1.5em * 12)`）で高さを
  抑えています。
- 言語選択（`select`）はすべてのインスタンスで `disabled: true` +
  `OpenState::Closed` の閉じたまま固定です。

## 関連情報

- [Badge](../themes/badge.md)
- [Code](../themes/code.md)
- [Select](../themes/select.md)
- [Button](../themes/button.md)
- [Text](../themes/text.md)
- [Empty State](../themes/empty-state.md)

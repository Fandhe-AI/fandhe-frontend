# code-block-header

`fandhe-frontend-pre-styled-ui` の `code` / `clipboard` / `button` /
`badge` / `text` の 5 部品を合成した、角丸枠の上端にヘッダー帯を持つ
コードブロックの実例です。Blocks セクションは新規部品を追加するものでは
なく、既存の Themes/Primitives 部品を組み合わせた実例集であることに注意
してください（主参照は対応表 ID R0054、集約元は R0055・R0057。出典の
固有名・ファイル名は記載しません）。

ヘッダー帯にファイル名タイトルとコピー操作だけを置いた最小構成（例 A）に
加え、折り返し・ダウンロードの補助ボタンを追加した形（例 B）、状態
バッジと本文の行番号を追加した形（例 C）の 3 通りを並べています。長い
行はページ全体ではなく枠の中で横スクロールします。

本 Demo は静的な表示例であり、`<form>` 要素は一切持ちません。クリップ
ボードは未コピー（idle）状態の固定表示で、B の補助ボタンは遷移先・
クリック処理を持たないため押下不能（`disabled`）です（実際のコピー動作
には `fandhe-frontend-wasm-full` の JS 配線が必要で、docs サイトは JS
ハイドレーションを行いません）。C の行番号は CSS カウンターで振るため
DOM にテキストを持たず、選択範囲に含めてもコピーされません。文言はすべて
独自に書いた架空のものであり、実企業名・実クレデンシャル・PII を含み
ません。

## Rust コード

```rust
use fandhe_frontend_core::{div, pre, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};
use fandhe_frontend_pre_styled_ui::Size;

/// A（R0054・主参照）で表示するコード片。1 行だけ意図的に長くし、枠内
/// 横スクロールが起きることを示す（モジュール doc「長い行は枠内で横
/// スクロールする」節）。
const SNIPPET_A: &str = "use fandhe_frontend_core::{div, text};\n\nfn demo() -> fandhe_frontend_core::Node {\n    div(vec![], vec![text(\"こんにちは\")])\n}\n\n// この行はとても長いコメントで、枠の幅を越えて横スクロールが発生することを示すためにわざと伸ばしてあります\n";

/// B（R0055）で表示するコード片。
const SNIPPET_B: &str =
    "use fandhe_frontend_core::render;\n\nfn main() {\n    println!(\"{}\", render(&demo()));\n}\n";

/// C（R0057）で表示するコード片。行番号を 1 行ずつ振るため
/// `.lines()` で分割する。
const SNIPPET_C: &str = "use fandhe_frontend_cli as _;\n\nfn check() -> bool {\n    // ビルドが通れば true\n    true\n}\n";

/// A（R0054・主参照）: ファイル名タイトル + コピー操作だけの最小構成。
fn instance_a() -> Node {
    header_instance(
        "blocks-code-block-header-a",
        SNIPPET_A,
        "demo.rs",
        None,
        false,
        false,
    )
}

/// B（R0055）: A に無効化済みの補助ボタン（折り返し・ダウンロード）を
/// 追加した構成。
fn instance_b() -> Node {
    header_instance(
        "blocks-code-block-header-b",
        SNIPPET_B,
        "main.rs",
        None,
        true,
        false,
    )
}

/// C（R0057）: ヘッダーに状態バッジを追加し、本文の左端へ行番号を振った
/// 構成。
fn instance_c() -> Node {
    header_instance(
        "blocks-code-block-header-c",
        SNIPPET_C,
        "check.rs",
        Some("成功"),
        false,
        true,
    )
}

/// A/B/C 共通のヘッダー付きコードブロックを組み立てる。
///
/// - `root_id`: 独立マウントルート識別子（モジュール doc「A/B/C を独立
///   マウントルートへ分離する理由」節参照）。
/// - `snippet`: 本文に表示するコード文字列。
/// - `title`: ヘッダー帯に表示するファイル名タイトル。
/// - `status_badge`: `Some(文言)` のときヘッダー帯の左端へ
///   `ColorPalette::Success` の状態バッジを追加する（C 専用）。
/// - `with_aux_buttons`: `true` のときコピー操作の隣へ無効化済みの補助
///   ボタン（折り返し・ダウンロード）を 2 個追加する（B 専用）。
/// - `numbered`: `true` のとき本文の各行を `span` で包み、CSS カウンター
///   で行番号を振る（C 専用、モジュール doc「行番号は CSS カウンターで
///   振る」節参照）。
fn header_instance(
    root_id: &'static str,
    snippet: &'static str,
    title: &'static str,
    status_badge: Option<&'static str>,
    with_aux_buttons: bool,
    numbered: bool,
) -> Node {
    let mut bar_left: Vec<Node> = Vec::new();
    if let Some(label) = status_badge {
        bar_left.push(badge::badge(
            &BadgeProps {
                palette: ColorPalette::Success,
                ..BadgeProps::default()
            },
            vec![],
            vec![text(label)],
        ));
    }
    bar_left.push(styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text(title)],
    ));

    let mut bar_right: Vec<Node> = vec![clipboard::root(
        snippet,
        false,
        vec![("data-blocks-code-block-header-copy", "")],
        vec![clipboard::control(
            false,
            vec![],
            vec![clipboard::trigger(
                false,
                vec![],
                vec![
                    clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                    clipboard::indicator(true, false, vec![], vec![text("コピー済み")]),
                ],
            )],
        )],
    )];
    if with_aux_buttons {
        bar_right.push(button::button(
            &ButtonProps {
                variant: ButtonVariant::Ghost,
                size: Size::Sm,
                // 遷移先・クリック処理を持たない合成例のボタンのため
                // `disabled: true` にして「押しても何も起きない」ことを
                // 明示する（モジュール doc「静的表示の制約」節、
                // `hero_install_command` と同型の判断）。
                disabled: true,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("折り返し")],
        ));
        bar_right.push(button::button(
            &ButtonProps {
                variant: ButtonVariant::Ghost,
                size: Size::Sm,
                disabled: true,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("ダウンロード")],
        ));
    }

    let body_children: Vec<Node> = if numbered {
        snippet
            .lines()
            .map(|line| {
                span(
                    vec![("data-blocks-code-block-header-line", "")],
                    vec![text(line)],
                )
            })
            .collect()
    } else {
        vec![text(snippet)]
    };

    div(
        vec![("id", root_id), ("class", "blocks-code-block-header-frame")],
        vec![
            div(
                vec![("class", "blocks-code-block-header-bar")],
                vec![
                    div(
                        vec![("data-blocks-code-block-header-bar-start", "")],
                        bar_left,
                    ),
                    div(
                        vec![("data-blocks-code-block-header-bar-end", "")],
                        bar_right,
                    ),
                ],
            ),
            pre(
                vec![("class", "blocks-code-block-header-pre"), ("tabindex", "0")],
                vec![code::code(
                    &CodeProps::default(),
                    vec![("data-blocks-code-block-header-code", "")],
                    body_children,
                )],
            ),
        ],
    )
}

pub fn demo() -> Node {
    div(
        vec![("class", "blocks-code-block-header-layout")],
        vec![instance_a(), instance_b(), instance_c()],
    )
}
```

## 原案差分メモ

- 例 A（主参照 R0054）はファイル名タイトル + コピー操作だけの最小構成を
  表します。
- 例 B（R0055）は A に補助ボタン（折り返し・ダウンロード）を追加した形で、
  いずれも実処理を持たない合成例のボタンのため `disabled` で押下不能を
  明示します。
- 例 C（R0057）はヘッダーへ状態バッジを追加し、本文の左端へ行番号を
  振った形です。行番号は DOM にテキストを置かず CSS の `counter-increment`/
  `::before { content: counter(...) }` で視覚的に振ることで、選択範囲へ
  行番号が混ざらないようにしています。
- 参照元の文言・配色・アイコンは持ち込まず、コード片・ファイル名等は
  すべて独自に作成した架空の値です。

関連情報: [Code](../themes/code.md) / [Clipboard](../themes/clipboard.md) /
[Button](../themes/button.md) / [Badge](../themes/badge.md) /
[Text](../themes/text.md)

# code-block-language-tabs

コードブロックのヘッダー帯に言語切替タブ（タブ列 + コピー操作）を備えた
ブロックです。`tabs` / `code` / `clipboard` / `text` の 4 部品を合成します。
Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0058（タブとコピーだけの基本形）で、R0059（大文字の
タイトル＋控えめなタブ）・R0060（淡色面のヘッダーを両端寄せ）を Demo の
3 インスタンス並記（A/B/C）で集約しています。docs サイトは無 JS のため
タブを切り替えられず、各インスタンスとも選択されていない言語の trigger
は `disabled` で固定しています。選択言語を変えた 3 インスタンスにより、
Rust / TOML / Shell のコードがすべてどこかで可視になります。狭い幅では
タブ列がヘッダーの 2 行目へ折り返します。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。コピー
ボタンは実アプリに組み込めば機能しますが、本 Demo 単体では静的表示に
留まります。コード・コマンドはいずれも架空のサンプルです。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 架空の Rust サンプル（`fandhe_frontend_core` の実在 API のみを使う）。
const RUST_CODE: &str = "use fandhe_frontend_core::{render, p, text};\n\nfn main() {\n    let html = render(&p(vec![], vec![text(\"Hello\")]));\n    println!(\"{html}\");\n}";

/// 架空の `Cargo.toml` 断片（具体的なバージョン番号は陳腐化を避けるため
/// 書かない）。
const TOML_CODE: &str = "[dependencies]\nfandhe-frontend-core = { version = \"...\" }";

/// 架空のシェルコマンド列。
const SHELL_CODE: &str = "cargo add fandhe-frontend-core\ncargo run";

/// 言語ごとのコードを引く。
fn code_for(lang: &str) -> &'static str {
    match lang {
        "rust" => RUST_CODE,
        "toml" => TOML_CODE,
        "shell" => SHELL_CODE,
        _ => unreachable!("code_block_language_tabs only declares rust/toml/shell"),
    }
}

/// 言語タブ列 + コピー対象コードを持つヘッダー帯 1 個を組み立てる。
///
/// - `variant`: インスタンス識別子（`"a"`/`"b"`/`"c"`、id の一意化と
///   [`LAYOUT_CSS`] 側の `data-blocks-code-block-language-tabs-variant`
///   セレクタ分岐に使う）。
/// - `title`: ヘッダーに表示するタイトル文言（`None` なら非表示、A 版）。
/// - `selected`: SSR 時点で選択表示する言語（`"rust"`/`"toml"`/`"shell"`）。
/// - `tabs_variant`/`tabs_size`: タブの見た目（B/C 版の控えめな外観差分）。
fn frame(
    variant: &str,
    title: Option<&str>,
    selected: &str,
    tabs_variant: TabsVariant,
    tabs_size: Size,
) -> Node {
    let frame_id = format!("blocks-code-block-language-tabs-{variant}");
    let tabs_id = format!("{frame_id}-tabs");

    let langs = [("rust", "Rust"), ("toml", "TOML"), ("shell", "Shell")];
    let items = langs
        .iter()
        .map(|(value, label)| TabItem {
            value,
            trigger: vec![text(*label)],
            content: vec![el(
                "pre",
                vec![("class", "blocks-code-block-language-tabs-pre")],
                vec![code::code(
                    &CodeProps::default(),
                    vec![],
                    vec![text(code_for(value))],
                )],
            )],
            // 選択されていない言語は disabled 固定（モジュール doc「無 JS
            // での扱い」節参照）。押しても選択状態・パネルが変わらない
            // dead control を残さない。
            disabled: *value != selected,
        })
        .collect();

    let props = TabsProps {
        id: tabs_id.as_str(),
        selected,
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    let tabs_node = tabs::tabs(
        tabs_variant,
        tabs_size,
        ColorPalette::default(),
        &props,
        items,
    );

    let title_node = title.map(|label| {
        div(
            vec![("class", "blocks-code-block-language-tabs-title")],
            vec![styled_text::text(
                &TextProps {
                    size: TextSize::Xs,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(label)],
            )],
        )
    });

    let copy_node = div(
        vec![("class", "blocks-code-block-language-tabs-copy")],
        vec![clipboard::root(
            code_for(selected),
            false,
            vec![],
            vec![clipboard::trigger(
                false,
                vec![("aria-label", "コードをコピー")],
                vec![
                    clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                    clipboard::indicator(true, false, vec![], vec![text("コピー済み")]),
                ],
            )],
        )],
    );

    let mut children = Vec::with_capacity(3);
    if let Some(title_node) = title_node {
        children.push(title_node);
    }
    children.push(tabs_node);
    children.push(copy_node);

    div(
        vec![
            ("id", frame_id.as_str()),
            ("class", "blocks-code-block-language-tabs-frame"),
            ("data-blocks-code-block-language-tabs-variant", variant),
        ],
        children,
    )
}

/// 版の差分を短く説明するキャプション（`ai_chat_code_preview::caption` と
/// 同型のパターン）。
fn caption(label: &str) -> Node {
    el(
        "p",
        vec![("class", "blocks-code-block-language-tabs-caption")],
        vec![text(label)],
    )
}

/// `code-block-language-tabs` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-code-block-language-tabs-layout")],
        vec![
            caption("A: タイトルなし・タブとコピーのみ（基準形）"),
            frame("a", None, "rust", TabsVariant::Line, Size::Md),
            caption("B: 大文字タイトル・控えめなタブ"),
            frame(
                "b",
                Some("依存関係に追加"),
                "toml",
                TabsVariant::Enclosed,
                Size::Sm,
            ),
            caption("C: 淡色ヘッダー帯・両端寄せ"),
            frame(
                "c",
                Some("セットアップ"),
                "shell",
                TabsVariant::Line,
                Size::Sm,
            ),
        ],
    )
}
```

## 原案差分メモ

- **A（R0058、基準形）**: タイトルなし。`tabs`（`Line`）とコピーだけの
  最小構成です。選択言語は Rust。
- **B（R0059）**: タイトルを大文字・字間広めで控えめに表示し、タブは
  `Enclosed` + 小サイズで目立たせ過ぎないようにしています。選択言語は
  TOML。
- **C（R0060）**: ヘッダー帯を淡色面にし、タイトルを左・タブとコピーを
  右へ両端寄せしています。選択言語は Shell。
- 選択されていない言語の trigger は `disabled` に固定しています。タブが
  切り替わらないため、コピー対象の値は常に表示中のコードと一致します。

関連情報: [Tabs](../themes/tabs.md) / [Code](../themes/code.md) /
[Clipboard](../themes/clipboard.md) / [Text](../themes/text.md)

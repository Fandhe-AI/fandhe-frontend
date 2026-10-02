# example-preview-tabs

`fandhe-frontend-pre-styled-ui` の `tabs` / `card` / `code` / `clipboard` /
`button` の 5 部品を合成した、部品のコード例をプレビュー/コード切替タブで
見せるカードの実例です。Blocks セクションは新規部品を追加するものでは
なく、既存の Themes/Primitives 部品を組み合わせた実例集であることに注意
してください（主参照は対応表 ID R0092、集約元は R0093。出典の固有名・
ファイル名は記載しません）。

カード上端のタブ列（プレビュー/コード）と同じ行の右端へコピー・外部で
開く操作を並べ、本体にはどちらか一方のパネルだけを表示します。docs
サイトは JS ハイドレーションを行わないため、選択していないタブは押せ
ません（`disabled`）。1 インスタンスだけだとコードパネルが一度も見えな
いため、選択状態違いの 2 インスタンス（A: プレビュー選択・B: コード
選択）を並べています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持ちません。クリップ
ボードは未コピー（idle）状態の固定表示、「外部で開く」ボタンは遷移先を
持たないため押下不能（`disabled`）です（実際のコピー・遷移には
`fandhe-frontend-wasm-full` の JS 配線が必要で、docs サイトは JS
ハイドレーションを行いません）。コード片はプレビューと同じ内容の自己
完結の Rust コードで、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, pre, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::Size;

/// コードパネルに表示するコード片。プレビュー関数（[`preview`]）が組み立てる
/// ボタン 2 個と一致させ、コピーした片だけで成り立つ自己完結の例にする。
const SNIPPET: &str = "use fandhe_frontend_core::{div, text};\nuse fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};\n\nfn demo() -> fandhe_frontend_core::Node {\n    div(\n        vec![],\n        vec![\n            button::button(&ButtonProps { disabled: true, ..ButtonProps::default() }, vec![], vec![text(\"保存する\")]),\n            button::button(\n                &ButtonProps { variant: ButtonVariant::Outline, disabled: true, ..ButtonProps::default() },\n                vec![],\n                vec![text(\"キャンセル\")],\n            ),\n        ],\n    )\n}\n";

/// プレビューパネルの中身（部品の実演）。[`SNIPPET`] と内容を一致させる。
fn preview() -> Node {
    div(
        vec![("class", "blocks-example-preview-tabs-preview")],
        vec![
            button::button(
                &ButtonProps {
                    // 遷移先・送信処理を持たない合成例のボタンのため
                    // `disabled: true` にして「押しても何も起きない」ことを
                    // 明示する（`code_block_header` と同型の判断）。
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("保存する")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("キャンセル")],
            ),
        ],
    )
}

/// A/B 共通のカード + タブ 1 インスタンスを組み立てる。
///
/// - `root_id`: 独立マウントルート識別子（モジュール doc「id」節参照）。
/// - `selected`: SSR 時点の選択状態（`"preview"`/`"code"`）。
fn instance(root_id: &'static str, selected: &'static str) -> Node {
    let tabs_id = format!("{root_id}-tabs");
    let props = TabsProps {
        id: tabs_id.as_str(),
        selected,
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    let items = vec![
        TabItem {
            value: "preview",
            trigger: vec![text("プレビュー")],
            content: vec![preview()],
            disabled: selected != "preview",
        },
        TabItem {
            value: "code",
            trigger: vec![text("コード")],
            content: vec![pre(
                vec![("data-blocks-example-preview-tabs-code-panel", "")],
                vec![code::code(
                    &CodeProps::default(),
                    vec![("data-blocks-example-preview-tabs-code", "")],
                    vec![text(SNIPPET)],
                )],
            )],
            disabled: selected != "code",
        },
    ];
    let tabs_node = tabs::tabs(
        TabsVariant::Line,
        Size::Sm,
        ColorPalette::default(),
        &props,
        items,
    );

    let actions = div(
        vec![("data-blocks-example-preview-tabs-actions", "")],
        vec![
            clipboard::root(
                SNIPPET,
                false,
                vec![],
                vec![clipboard::control(
                    false,
                    vec![],
                    vec![clipboard::trigger(
                        false,
                        vec![
                            // docs サイトは JS ハイドレーションを行わないため
                            // `navigator.clipboard` 配線が無く、押しても
                            // コピーは実行されない（モジュール doc「操作」節）。
                            // headless 層の `trigger` は `disabled` 引数を
                            // 持たない（`clipboard.rs` rustdoc「意図的非採用:
                            // disabled 視覚」参照）ため、`button::button` と
                            // 同じ 3 点セット（`disabled`/`data-disabled`/
                            // `aria-disabled`）を呼び出し側 attrs から直接
                            // 付与し、動作しないボタンを押せる状態にしない
                            // （`code_block_header` と同型の是正）。
                            ("disabled", ""),
                            ("data-disabled", ""),
                            ("aria-disabled", "true"),
                        ],
                        vec![
                            clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                            clipboard::indicator(true, false, vec![], vec![text("コピー済み")]),
                        ],
                    )],
                )],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    size: Size::Sm,
                    // リンク先を持たない合成例のため disabled のボタンにし、
                    // 死にリンク・reverse tabnabbing を避ける
                    // （`code_block_header` B と同型の判断）。
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("外部で開く")],
            ),
        ],
    );

    card::root(
        CardProps::default(),
        vec![
            ("id", root_id),
            ("data-blocks-example-preview-tabs-frame", ""),
        ],
        vec![tabs_node, actions],
    )
}

pub fn demo() -> Node {
    div(
        vec![("class", "blocks-example-preview-tabs-layout")],
        vec![
            instance("blocks-example-preview-tabs-a", "preview"),
            instance("blocks-example-preview-tabs-b", "code"),
        ],
    )
}
```

## 原案差分メモ

- 主参照 R0092（カード内のプレビュー/コード切替）をもとに、タブ列と同じ
  行の右端へコピー・外部で開く操作を並べています。
- 集約元 R0093（タブ + 右寄せの外部リンク）の差分は「コピー操作が無い」
  程度に小さく、基本仕様（右端の外部で開くボタン）にすでに含まれるため、
  3 つ目のインスタンスは追加していません。
- 参照元の文言・配色・アイコンは持ち込まず、コード片・ボタン文言等は
  すべて独自に作成した架空の値です。

関連情報: [Tabs](../themes/tabs.md) / [Card](../themes/card.md) /
[Code](../themes/code.md) / [Clipboard](../themes/clipboard.md) /
[Button](../themes/button.md)

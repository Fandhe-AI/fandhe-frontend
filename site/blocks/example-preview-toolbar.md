# example-preview-toolbar

`fandhe-frontend-pre-styled-ui` の `tabs` / `select` / `button` /
`clipboard` / `code` / `popover` の 6 部品を合成した、部品のコード例を
「プレビュー + ツールバー + コード」の 3 段構成 1 枠で見せる実例です。
Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R0094。出典の固有名・ファイル名は記載しません）。

上段にウィジェットの実演（開いた状態の `popover`）、中段にコード種別
タブ（Rust/CSS）とスタイル選択・外部実行・コピーのツールバー、下段に
選択中のコードを表示します。3 段は 1 つの枠にまとまり、狭い幅では
ツールバー右側の操作群が折り返します。

本 Demo は静的な表示例であり、`<form>` 要素は一切持ちません。コード種別
タブは Rust を選択状態に固定し、CSS タブは押下不能（`disabled`）です。
スタイル選択も閉じた状態の固定表示で `disabled` にしています。「外部で
実行」ボタンは遷移先・クリック処理を持たないため押下不能、コピーは
未コピー（idle）状態の固定表示です（実際の切り替え・コピー動作には
`fandhe-frontend-wasm-full` の JS 配線が必要で、docs サイトは JS
ハイドレーションを行いません）。文言はすべて独自に書いた架空のものであり、
実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, pre, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::popover::{self, OpenState};
use fandhe_frontend_pre_styled_ui::select::{self, SelectProps};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// プレビューに表示する popover のタイトル `id`（`content` の
/// `labelledby` と対にする）。
const PREVIEW_TITLE_ID: &str = "blocks-example-preview-toolbar-preview-title";
/// プレビューに表示する popover の `content`/`trigger` 間を結ぶ `id`。
const PREVIEW_CONTENT_ID: &str = "blocks-example-preview-toolbar-preview-content";

/// コード種別タブの `id`（`tabs::tabs` の root へ付与）。
const TABS_ID: &str = "blocks-example-preview-toolbar-tabs";

/// スタイル選択の `label`/`content` を結ぶ `id` の組。
const SELECT_LABEL_ID: &str = "blocks-example-preview-toolbar-select-label";
const SELECT_CONTENT_ID: &str = "blocks-example-preview-toolbar-select-content";

/// Rust タブのパネルに表示するコード片。プレビューの popover を組み立てる
/// 自己完結のコード片（コピーした片だけで成立する、`code_block_header`
/// と同じ方針）。
const SNIPPET_RUST: &str = "use fandhe_frontend_core::text;\nuse fandhe_frontend_pre_styled_ui::popover::{self, OpenState};\n\nfn preview() -> fandhe_frontend_core::Node {\n    let trigger = popover::trigger(\n        OpenState::Closed,\n        false,\n        Some(\"demo-content\"),\n        vec![],\n        vec![text(\"表示設定\")],\n    );\n    let content = popover::content(\n        OpenState::Closed,\n        Some(\"demo-content\"),\n        None,\n        None,\n        vec![],\n        vec![text(\"ここに設定項目を置きます\")],\n    );\n    let positioner = popover::positioner(OpenState::Closed, vec![], vec![content]);\n    popover::root(OpenState::Closed, vec![], vec![trigger, positioner])\n}\n";

/// CSS タブのパネルに表示するコード片。
const SNIPPET_CSS: &str = "[data-scope=\"popover\"][data-part=\"content\"] {\n  min-width: 16rem;\n  padding: var(--fandhe-space-4);\n}\n";

/// 無 JS で固定表示するプレビュー（開いた状態の `popover`。モジュール doc
/// 「プレビューを開いた状態で固定する理由」節参照）。
fn preview() -> Node {
    let trigger = popover::trigger(
        OpenState::Open,
        true,
        Some(PREVIEW_CONTENT_ID),
        vec![("data-blocks-example-preview-toolbar-trigger", "")],
        vec![text("表示設定")],
    );
    let content = popover::content(
        OpenState::Open,
        Some(PREVIEW_CONTENT_ID),
        Some(PREVIEW_TITLE_ID),
        None,
        vec![("data-blocks-example-preview-toolbar-content", "")],
        vec![
            popover::title(Some(PREVIEW_TITLE_ID), vec![], vec![text("表示設定")]),
            text("ウィジェットの見た目を切り替えます。"),
        ],
    );
    let positioner = popover::positioner(
        OpenState::Open,
        vec![("data-blocks-example-preview-toolbar-positioner", "")],
        vec![content],
    );
    div(
        vec![("class", "blocks-example-preview-toolbar-preview")],
        vec![popover::root(
            OpenState::Open,
            vec![],
            vec![trigger, positioner],
        )],
    )
}

/// コード種別タブ（Rust/CSS）。先頭（Rust）を選択状態に固定し、非選択
/// タブ（CSS）は `disabled: true` にする（モジュール doc「コード種別
/// タブ・スタイル選択を固定状態で置く理由」節参照）。
fn toolbar_tabs() -> Node {
    let props = TabsProps {
        id: TABS_ID,
        selected: "rust",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    let items = vec![
        TabItem {
            value: "rust",
            trigger: vec![text("Rust")],
            content: vec![pre(
                vec![("class", "blocks-example-preview-toolbar-pre")],
                vec![code::code(
                    &CodeProps::default(),
                    vec![],
                    vec![text(SNIPPET_RUST)],
                )],
            )],
            disabled: false,
        },
        TabItem {
            value: "css",
            trigger: vec![text("CSS")],
            content: vec![pre(
                vec![("class", "blocks-example-preview-toolbar-pre")],
                vec![code::code(
                    &CodeProps::default(),
                    vec![],
                    vec![text(SNIPPET_CSS)],
                )],
            )],
            disabled: true,
        },
    ];
    tabs::tabs(
        TabsVariant::Line,
        Size::Sm,
        ColorPalette::default(),
        &props,
        items,
    )
}

/// スタイル選択（閉じた状態の `select`、モジュール doc「コード種別タブ・
/// スタイル選択を固定状態で置く理由」節参照。`card_form_footer::
/// closed_select` と同型だが private ヘルパーのため本ファイルへ再実装
/// する）。
fn style_select() -> Node {
    let props = SelectProps {
        disabled: true,
        ..SelectProps::default()
    };
    div(
        vec![("class", "blocks-example-preview-toolbar-select")],
        vec![
            select::label(
                &props,
                Some(SELECT_LABEL_ID),
                vec![],
                vec![text("スタイル")],
            ),
            select::root(
                Size::Sm,
                OpenState::Closed,
                &props,
                vec![("data-blocks-example-preview-toolbar-select-root", "")],
                vec![
                    select::control(
                        OpenState::Closed,
                        &props,
                        vec![],
                        vec![select::trigger(
                            OpenState::Closed,
                            &props,
                            false,
                            Some(SELECT_CONTENT_ID),
                            Some(SELECT_LABEL_ID),
                            vec![],
                            vec![
                                select::value_text(false, &props, vec![], vec![text("既定")]),
                                select::indicator(OpenState::Closed, &props, vec![], vec![]),
                            ],
                        )],
                    ),
                    select::positioner(
                        OpenState::Closed,
                        vec![],
                        vec![select::content(
                            OpenState::Closed,
                            Some(SELECT_CONTENT_ID),
                            Some(SELECT_LABEL_ID),
                            None,
                            vec![],
                            vec![],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// ツールバー右側の操作群（スタイル選択 + 外部実行 + コピー、モジュール
/// doc「『外部で実行』・コピー操作」節参照）。
fn toolbar_actions() -> Node {
    div(
        vec![("data-blocks-example-preview-toolbar-actions", "")],
        vec![
            style_select(),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    size: Size::Sm,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-example-preview-toolbar-run", "")],
                vec![text("外部で実行")],
            ),
            clipboard::root(
                SNIPPET_RUST,
                false,
                vec![],
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
            ),
        ],
    )
}

/// `example-preview-toolbar` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。プレビュー（上段）→ コード種別タブ + 操作群（中段）→
/// タブのパネル（下段、タブ本体に内包）の順で DOM へ並べ、視覚順は
/// [`LAYOUT_CSS`] の `order` で並べ替える（モジュール doc「DOM 順と視覚順
/// の分離」節参照）。
pub fn demo() -> Node {
    div(
        vec![
            ("id", "blocks-example-preview-toolbar"),
            ("class", "blocks-example-preview-toolbar-frame"),
        ],
        vec![preview(), toolbar_tabs(), toolbar_actions()],
    )
}
```

## 原案差分メモ

- 主参照（対応表 ID R0094）の 3 段構成（プレビュー + ツールバー + コード）
  をそのまま踏襲しています。集約元が R0094 の 1 件のみのため、状態違いの
  インスタンス併記は行っていません。
- コード種別タブ・スタイル選択・プレビューの開閉状態はいずれも無 JS 制約
  のため静的に固定しています。
- 文言（タブラベル・プレビュー見出し・コード片）はすべて独自に作成した
  架空のものです。

関連情報: [Tabs](../themes/tabs.md) / [Select](../themes/select.md) /
[Button](../themes/button.md) / [Clipboard](../themes/clipboard.md) /
[Code](../themes/code.md) / [Popover](../themes/popover.md)

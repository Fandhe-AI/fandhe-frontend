//! `example-preview-toolbar` block（イシュー #3114。親トラッキング #3099
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0094 を主参照とする
//! 合成例。部品のコード例を「プレビュー + ツールバー + コード」の 3 段
//! 構成 1 枠で見せる）。取得手段・ファイル名・内部識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じ転記
//! 制限）。集約元は R0094 の 1 件のみのため、差分インスタンスの併記は
//! 行わない（原案との差分は原稿「原案差分メモ」節で扱う）。
//!
//! # 構成（3 段 1 枠）
//!
//! - **上段（プレビュー）**: 部品の実演。本 block では開いた状態の
//!   `popover` を表示する。
//! - **中段（ツールバー）**: コード種別タブ（[`toolbar_tabs`]）+
//!   スタイル選択・外部実行・コピーの操作群（[`toolbar_actions`]）。
//! - **下段（コード）**: コード種別タブのパネルとして表示する
//!   （タブの一部であり、中段と下段を分ける素の `div` は持たない）。
//!
//! 3 段は [`LAYOUT_CSS`] の flex レイアウトで 1 つの枠へまとめ、狭い幅では
//! ツールバー内の操作群が折り返す（モジュール doc「DOM 順と視覚順の分離」
//! 節参照）。
//!
//! # 使用部品
//!
//! `tabs` / `select` / `button` / `clipboard` / `code` / `popover` の 6
//! 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証
//! する）。新しい UI 部品は追加しない。
//!
//! # DOM 順と視覚順の分離（`order` プロパティ）
//!
//! [`fandhe_frontend_pre_styled_ui::tabs::tabs`] は root 1 ノードの中へ
//! タブ列（`[data-part="list"]`）とパネル（`[data-part="content"]`）を
//! 子として内包するため、呼び出し側からは分離できない。本 block は
//! tabs root を [`LAYOUT_CSS`] で `display: contents` にして、タブ列・
//! パネルを枠の直接の flex item として扱えるようにし、DOM 順
//! （プレビュー → タブ列 + パネル → 操作群）とは異なる視覚順
//! （プレビュー → タブ列 → 操作群 → パネル）を `order` プロパティで
//! 実現する。フォーカス順は DOM 順のまま「タブ → パネル → 操作」になる
//! （コピーは内容に付随する操作であるため許容する判断、
//! `code_block_header` と同型）。
//!
//! # プレビューを開いた状態で固定する理由
//!
//! docs サイトは JS ハイドレーションを行わないため、`popover` は
//! `OpenState::Open` 固定で中身を常時表示し、トリガーは
//! `disabled: true`（無 JS で押しても何も起きないため）にする
//! （`notification_tray` と同型の判断）。[`LAYOUT_CSS`] で positioner を
//! `position: static` へ中和し、通常のドキュメントフローへ戻す。
//!
//! # コード種別タブ・スタイル選択を固定状態で置く理由
//!
//! 同じく無 JS 制約のため、コード種別タブは先頭（Rust）を選択状態に
//! 固定し非選択タブ（CSS）は `disabled: true` にする
//! （`notification_tray_tabs` と同型）。スタイル選択（`select`）は
//! 閉じた状態（`OpenState::Closed`）+ `disabled: true` で固定する
//! （`card_form_footer::closed_select` と同型の判断。呼び出しヘルパーは
//! private なため、`blocks-code` マーカー内で自己完結させるべく本ファイル
//! に再実装する）。
//!
//! # 「外部で実行」・コピー操作
//!
//! 「外部で実行」ボタンは遷移先・クリック処理を持たない合成例のため
//! `disabled: true` で押下不能を明示する（`href`/`target` は持たせず、
//! 死にリンク・reverse tabnabbing を避ける）。コピーは idle 固定
//! （`data-copied` を持たない）の [`fandhe_frontend_pre_styled_ui::clipboard`]
//! を使う。
//!
//! # `class` と `data-*` の使い分け
//!
//! `button::button`/`code::code`/`clipboard::root`/`select::root` 等の
//! pre-styled パーツはいずれも `drop_class_attr` で呼び出し側 `class` を
//! 除去するため、CSS フックは `data-blocks-example-preview-toolbar-*` で
//! 渡す。素の `div`/`pre` には `.blocks-example-preview-toolbar-*`
//! クラスセレクタを使う（`code_block_header`/`notification_tray` と同型）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンはすべて既定 `type="button"` のまま用いる。
//!
//! # コード・文言はすべて無害
//!
//! コード片は本フレームワーク自身のノード木 API を使う短い自己完結の
//! Rust/CSS のみで、実在の秘密情報・トークンらしき文字列を含まない。
//! 文言はすべて独自に作成した架空のものである。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
    let rust_pre_label = "表示設定プレビューの Rust コード";
    let css_pre_label = "表示設定プレビューの CSS コード";
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
                vec![
                    ("class", "blocks-example-preview-toolbar-pre"),
                    ("tabindex", "0"),
                    ("aria-label", rust_pre_label),
                ],
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
                vec![
                    ("class", "blocks-example-preview-toolbar-pre"),
                    ("tabindex", "0"),
                    ("aria-label", css_pre_label),
                ],
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
                vec![],
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/example-preview-toolbar/",
    title: "example-preview-toolbar",
    category: BlockCategory::ExamplePreview,
    rust_source: "crates/docs-site/src/blocks/docs/example_preview/example_preview_toolbar.rs",
    demo_class: "blocks-example-preview-toolbar",
    parts: &[
        Part {
            label: "Tabs",
            path: "/themes/tabs/",
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
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Code",
            path: "/themes/code/",
        },
        Part {
            label: "Popover",
            path: "/themes/popover/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `example_preview_toolbar` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同型）。`--fandhe-*`
/// トークンのみ使用し、生値は幅・rem 指定のみに限る。
const LAYOUT_CSS: &str = "\
.blocks-example-preview-toolbar-frame {\n  display: flex;\n  flex-wrap: wrap;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n  min-width: 0;\n}\n\
.blocks-example-preview-toolbar-preview {\n  flex-basis: 100%;\n  padding: var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-scope=\"popover\"][data-part=\"positioner\"][data-blocks-example-preview-toolbar-positioner] {\n  position: static;\n}\n\
[data-scope=\"popover\"][data-part=\"trigger\"][data-blocks-example-preview-toolbar-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-example-preview-toolbar [data-scope=\"popover\"] h2 {\n  margin: 0;\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
[data-scope=\"tabs\"][data-part=\"root\"] {\n  display: contents;\n}\n\
[data-scope=\"tabs\"][data-part=\"list\"] {\n  order: 1;\n  flex: 1 1 auto;\n  min-width: 0;\n  background: var(--fandhe-color-bg-subtle);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-scope=\"tabs\"][data-part=\"content\"] {\n  order: 3;\n  flex-basis: 100%;\n}\n\
[data-blocks-example-preview-toolbar-actions] {\n  order: 2;\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  margin-inline-start: auto;\n  padding-inline: var(--fandhe-space-3);\n  background: var(--fandhe-color-bg-subtle);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"select\"][data-part=\"root\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-example-preview-toolbar-select {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-example-preview-toolbar-pre {\n  margin: 0;\n  padding: var(--fandhe-space-4);\n  overflow-x: auto;\n  font-family: var(--fandhe-font-font-mono);\n}\n\
.blocks-example-preview-toolbar-pre:focus-visible {\n  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// `<form>`・`data-copied`・死にリンク・`data:` URI がいずれも出ないこと。
    #[test]
    fn demo_has_no_form_data_copied_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert_eq!(html.matches("data-copied").count(), 0);
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("target=\"_blank\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 非選択のコード種別タブ（CSS）が disabled であること。
    #[test]
    fn non_selected_code_tab_is_disabled() {
        let html = demo_html();
        let needle_start = html.find(r#"data-value="css""#).unwrap();
        let tag_end = needle_start + html[needle_start..].find('>').unwrap();
        let trigger_html = &html[needle_start..tag_end];
        assert!(trigger_html.contains("disabled=\"\""));
    }

    /// プレビューの popover が Open 固定（positioner に
    /// `data-state="closed"` が出ない）こと。
    #[test]
    fn preview_popover_positioner_is_open() {
        let html = demo_html();
        assert!(html.contains("data-blocks-example-preview-toolbar-positioner"));
        assert!(
            !html.contains("data-scope=\"popover\" data-part=\"positioner\" data-state=\"closed\"")
        );
    }

    /// スタイル選択が閉状態かつ disabled であること。
    #[test]
    fn style_select_is_closed_and_disabled() {
        let html = demo_html();
        assert!(html.contains(r#"data-scope="select" data-part="root""#));
        assert!(html.contains(r#"data-scope="select" data-part="positioner" data-state="closed""#));
    }

    /// [`LAYOUT_CSS`] が `flex-wrap: wrap` と `display: contents` を持ち、
    /// `<` を含まないこと（`</style>` 破り防止）。
    #[test]
    fn layout_css_declares_wrap_and_contents() {
        assert!(LAYOUT_CSS.contains("flex-wrap: wrap"));
        assert!(LAYOUT_CSS.contains("display: contents"));
        assert!(!LAYOUT_CSS.contains('<'));
    }
}

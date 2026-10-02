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
//! 死にリンク・reverse tabnabbing を避ける）。コピーは
//! [`fandhe_frontend_pre_styled_ui::clipboard`] の `root`/`control`/
//! `trigger`/`indicator` を正規の構成で組み合わせているため、
//! `fandhe-frontend-wasm-full` の `headless_clipboard` 配線
//! （`crates/wasm-full/src/headless_clipboard.rs`）が実アプリの
//! `mount`/`hydrate` 時に `navigator.clipboard.writeText` を自動配線する
//! （`hero_install_command` モジュール doc「コピー配線の範囲」節と同型の
//! 構成）。無 JS の docs サイト自体では他の全部品と同じく静的表示に
//! 留まり（idle 固定、`data-copied` を持たない）、Trigger はクリック
//! しても無反応になるが、これは `headless-ui`/`clipboard` が SSR
//! マークアップのみを提供しクライアントランタイム側がコピー実行を担う
//! という `site/primitives/clipboard.md` 既定の site 全体の制約であり、
//! 本 block 固有の欠陥ではない（`clipboard::trigger` は `copied` 状態の
//! みを引数に持ち `disabled` を表現できないため、「外部で実行」ボタンと
//! 異なり無効化はしない）。実アプリへ組み込めばコピー操作は実際に
//! 機能する。
//!
//! # `class` と `data-*` の使い分け
//!
//! `button::button`/`code::code`/`clipboard::root`/`select::root` 等の
//! pre-styled パーツはいずれも `drop_class_attr` で呼び出し側 `class` を
//! 除去するため、CSS フックは `data-blocks-example-preview-toolbar-*` で
//! 渡す。素の `div`/`pre` には `.blocks-example-preview-toolbar-*`
//! クラスセレクタを使う（`code_block_header`/`notification_tray` と同型）。
//! pre-styled パーツの見た目を上書きする規則は、block 固有のフック属性と
//! レシピ属性（`[data-scope][data-part]`）を同じ要素上で連結し、レシピより
//! 詳細度を上げる（`docs_layout_sidebar_api` と同じ流儀）。
//!
//! # 下段コードパネルの `code` 装飾リセット
//!
//! `code::code` は既定（`Subtle`）でインラインコード用のピル背景・
//! padding・角丸を持つ。複数行スニペットを包む本 block では、
//! `data-blocks-example-preview-toolbar-code` を付けた `code` に限って
//! それらをリセットし、`pre` の地に溶け込むブロック表示にする
//! （`example_preview_tabs`/`code_block_header` と同型）。
//!
//! # 枠の `overflow: hidden` とフォーカスリング
//!
//! 枠は角丸の内側へ子を収めるため `overflow: hidden` を持つ。端に接する
//! タブ・コードパネル・コピー操作の外側へ描くフォーカスリングが切れない
//! よう、それらの `:focus-visible` は `outline-offset: -2px`（内側描画）に
//! 揃える（`example_preview_tabs` と同型）。
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

/// Rust タブのパネルに表示するコード片。[`preview`] と同じ popover 構成
/// （開いた状態・押下不能トリガー・タイトル + 本文）を、block 固有の
/// `id` 定数・CSS フック属性を除いて組み立てる自己完結のコード片
/// （コピーした片だけで成立する、`code_block_header` と同じ方針）。
const SNIPPET_RUST: &str = "use fandhe_frontend_core::{div, text, Node};\nuse fandhe_frontend_pre_styled_ui::popover::{self, OpenState};\n\nfn preview() -> Node {\n    let trigger = popover::trigger(\n        OpenState::Open,\n        true,\n        Some(\"demo-content\"),\n        vec![],\n        vec![text(\"表示設定\")],\n    );\n    let content = popover::content(\n        OpenState::Open,\n        Some(\"demo-content\"),\n        Some(\"demo-title\"),\n        None,\n        vec![],\n        vec![\n            popover::title(Some(\"demo-title\"), vec![], vec![text(\"表示設定\")]),\n            text(\"ウィジェットの見た目を切り替えます。\"),\n        ],\n    );\n    let positioner = popover::positioner(OpenState::Open, vec![], vec![content]);\n    div(\n        vec![],\n        vec![popover::root(OpenState::Open, vec![], vec![trigger, positioner])],\n    )\n}\n";

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
                    vec![("data-blocks-example-preview-toolbar-code", "")],
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
                    vec![("data-blocks-example-preview-toolbar-code", "")],
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
                            vec![("data-blocks-example-preview-toolbar-select-trigger", "")],
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
///
/// # 全 block 共通 CSS への連結に対する `[data-scope="..."]` セレクタのスコープ限定
///
/// 本 CSS は `crate::blocks::STYLESHEET_REL_PATH`（`assets/blocks.css`）へ
/// 全 block 共通で連結される（モジュール doc「CSS の置き場」節）。
/// `tabs::tabs` は headless 層に呼び出し側 attrs を受け取る引数を持たず
/// root/list/content へ block 固有属性を注入できないため、tabs 系 3 セレクタ
/// は `.blocks-example-preview-toolbar`（Demo ラッパへ付与される
/// `Block::demo_class`、popover の `h2` 補正規則と同じ祖先セレクタ）で
/// 限定する。button/select の disabled 中和規則は呼び出し側 attrs 経由で
/// 付与できるため、`data-blocks-example-preview-toolbar-run`/
/// `-select-trigger` を複合セレクタへ併記して限定する。select は
/// `disabled_declarations()`（`opacity`/`cursor`）を root ではなく trigger
/// へ付けるため、中和規則も trigger を対象にする（`filter_dropdown_bar`
/// の `collapsed_filter_button` と同型の判断）。祖先・属性いずれの限定も
/// 持たない生の `[data-scope="tabs"/"button"/"select"]` 複合セレクタを
/// 本 CSS に残すと、他 block の Tabs（`display: contents`/`order`）や
/// disabled Button/Select（`opacity`/`cursor`）の見た目まで書き換えてしまう
/// （Cursor Bugbot/codex-review P1 指摘の是正）。
const LAYOUT_CSS: &str = "\
.blocks-example-preview-toolbar-frame {\n  display: flex;\n  flex-wrap: wrap;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n  min-width: 0;\n}\n\
.blocks-example-preview-toolbar-preview {\n  flex-basis: 100%;\n  padding: var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-scope=\"popover\"][data-part=\"positioner\"][data-blocks-example-preview-toolbar-positioner] {\n  position: static;\n}\n\
[data-scope=\"popover\"][data-part=\"trigger\"][data-blocks-example-preview-toolbar-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-example-preview-toolbar [data-scope=\"popover\"] h2 {\n  margin: 0;\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
.blocks-example-preview-toolbar [data-scope=\"tabs\"][data-part=\"root\"] {\n  display: contents;\n}\n\
.blocks-example-preview-toolbar [data-scope=\"tabs\"][data-part=\"list\"] {\n  order: 1;\n  flex: 1 1 auto;\n  min-width: 0;\n  background: var(--fandhe-color-bg-subtle);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-example-preview-toolbar [data-scope=\"tabs\"][data-part=\"content\"] {\n  order: 3;\n  flex-basis: 100%;\n  overflow-x: auto;\n}\n\
[data-blocks-example-preview-toolbar-actions] {\n  order: 2;\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  margin-inline-start: auto;\n  padding-inline: var(--fandhe-space-3);\n  background: var(--fandhe-color-bg-subtle);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-example-preview-toolbar-run][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"select\"][data-part=\"trigger\"][data-blocks-example-preview-toolbar-select-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-example-preview-toolbar-select {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-example-preview-toolbar-pre {\n  margin: 0;\n  padding: var(--fandhe-space-4);\n  font-family: var(--fandhe-font-font-mono);\n}\n\
[data-scope=\"code\"][data-part=\"root\"][data-blocks-example-preview-toolbar-code] {\n  display: block;\n  white-space: pre;\n  background: transparent;\n  border: 0;\n  border-radius: 0;\n  padding: 0;\n  color: inherit;\n}\n\
.blocks-example-preview-toolbar [data-scope=\"tabs\"][data-part=\"trigger\"]:focus-visible,\n.blocks-example-preview-toolbar [data-scope=\"tabs\"][data-part=\"content\"]:focus-visible,\n[data-blocks-example-preview-toolbar-actions] [data-scope=\"clipboard\"][data-part=\"trigger\"]:focus-visible {\n  outline-offset: -2px;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, preview, LAYOUT_CSS, SNIPPET_RUST};
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

    /// select の disabled 中和規則が、レシピが `opacity`/`cursor` を付ける
    /// trigger を対象にし、フック属性が trigger 上に出ること。
    #[test]
    fn select_disabled_neutralizer_targets_trigger() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"select\"][data-part=\"trigger\"][data-blocks-example-preview-toolbar-select-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
        assert!(!LAYOUT_CSS.contains("[data-scope=\"select\"][data-part=\"root\"]"));
        let html = demo_html();
        let at = html
            .find("data-blocks-example-preview-toolbar-select-trigger")
            .unwrap();
        let tag_start = html[..at].rfind('<').unwrap();
        let tag = &html[tag_start..at];
        assert!(tag.contains(r#"data-scope="select" data-part="trigger""#));
        assert!(tag.contains("data-disabled"));
    }

    /// 下段の `code` 2 件にフック属性が付き、インライン装飾（背景・
    /// padding・角丸・枠）をレシピ属性連結の規則でリセットすること。
    #[test]
    fn panel_code_resets_inline_decoration() {
        let html = demo_html();
        assert_eq!(
            html.matches(r#"data-scope="code" data-part="root""#)
                .count(),
            html.matches("data-blocks-example-preview-toolbar-code")
                .count()
        );
        assert_eq!(
            html.matches("data-blocks-example-preview-toolbar-code")
                .count(),
            2
        );
        let start = LAYOUT_CSS
            .find("[data-scope=\"code\"][data-part=\"root\"][data-blocks-example-preview-toolbar-code] {")
            .unwrap();
        let rule = &LAYOUT_CSS[start..start + LAYOUT_CSS[start..].find('}').unwrap()];
        for decl in [
            "display: block;",
            "background: transparent;",
            "border: 0;",
            "border-radius: 0;",
            "padding: 0;",
        ] {
            assert!(rule.contains(decl), "{decl}");
        }
    }

    /// `overflow: hidden` の枠に接するタブ・パネル・コピー操作の
    /// フォーカスリングを内側（`outline-offset: -2px`）に描くこと。
    #[test]
    fn focus_rings_are_drawn_inset_inside_clipping_frame() {
        assert!(LAYOUT_CSS.contains("overflow: hidden"));
        let end = LAYOUT_CSS.find("{\n  outline-offset: -2px;\n}").unwrap();
        let selectors = &LAYOUT_CSS[LAYOUT_CSS[..end].rfind('}').unwrap()..end];
        for sel in [
            ".blocks-example-preview-toolbar [data-scope=\"tabs\"][data-part=\"trigger\"]:focus-visible",
            ".blocks-example-preview-toolbar [data-scope=\"tabs\"][data-part=\"content\"]:focus-visible",
            "[data-blocks-example-preview-toolbar-actions] [data-scope=\"clipboard\"][data-part=\"trigger\"]:focus-visible",
        ] {
            assert!(selectors.contains(sel), "{sel}");
        }
    }

    /// コードパネルの Rust スニペットが [`preview`] と同じ popover 構成
    /// （Open 固定・押下不能トリガー・タイトル + 本文）を組み立てること
    /// （プレビューとスニペットの乖離の再発防止）。
    #[test]
    fn rust_snippet_matches_preview_composition() {
        let preview_html = render(&preview());
        assert!(preview_html.contains("ウィジェットの見た目を切り替えます。"));
        assert!(!SNIPPET_RUST.contains("OpenState::Closed"));
        assert_eq!(SNIPPET_RUST.matches("OpenState::Open").count(), 4);
        for call in [
            "popover::trigger(\n        OpenState::Open,\n        true,",
            "popover::content(",
            "popover::title(",
            "popover::positioner(",
            "popover::root(",
            "text(\"表示設定\")",
            "text(\"ウィジェットの見た目を切り替えます。\")",
        ] {
            assert!(SNIPPET_RUST.contains(call), "{call}");
        }
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

//! `example-preview-tabs` block（イシュー #3113。親トラッキング #3099
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0092 を主参照とし
//! R0093 の構成差分を集約した合成例。部品のコード例を、プレビュー/コード
//! 切替タブで見せるカード）。取得手段・ファイル名・内部コンポーネント
//! 識別子は記載しない（`docs/design/motion-reference-adoption-policy.md`
//! §9 と同じライセンス上の転記制限）。`_/blocks-intake/` の対応ファイルは
//! 本イシュー着手時点で本 worktree に存在しないため、原稿・本コメントには
//! 対応表 ID のみを記す（`notification_tray_tabs` と同じ扱い）。
//!
//! # 使用部品
//!
//! `tabs` / `card` / `code` / `clipboard` / `button` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 実物 `tabs::tabs` を使い、選択状態違いの 2 インスタンスを並べる
//!
//! docs サイトは JS ハイドレーションを行わないため、`tabs::tabs` は
//! 非選択パネルへ `hidden` を付与したままになる。プレビュー選択の
//! 1 インスタンスだけではコードパネルを一度も可視化できないため、
//! `notification_tray_tabs`/`table_with_toolbar::status_tabs` と同じ判断で
//! 次の 2 インスタンスを並べる（1 版あたりトリガー 2 個、合計 4 個に収まる
//! ため `component_specs_overlay::feature_tabs_panel` が避けた「トリガーが
//! 多数に膨れる」懸念には当たらない）。
//!
//! - **A（R0092・主参照）**: `selected: "preview"`（静的表示の既定）。
//! - **B（R0092・状態違い）**: `selected: "code"`（コードパネルを実際に
//!   見られるようにするための版）。
//!
//! 各インスタンスとも非選択タブは `disabled: true` で固定する
//! （`table_with_toolbar::status_tabs`・`notification_tray_tabs` と同型。
//! 押しても選択状態・パネルが変わらない dead control を `disabled: false`
//! のまま放置しない）。選択中のタブは disabled にしない
//! （headless の仕様上、選択中タブを disabled にすると未選択扱いに落ちる）。
//!
//! # R0093 の差分は原稿の「原案差分メモ」で扱う
//!
//! R0093（タブ + 右寄せの外部リンク）の差分は基本仕様（右端の外部で開く
//! ボタン）にすでに含まれ、コピー操作が無い程度の小さな差分のため、
//! 3 つ目のインスタンス（コードパネルを隠した版の再発）は追加せず、
//! 差分は `site/blocks/example-preview-tabs.md` の「原案差分メモ」節へ記す。
//!
//! # ヘッダー行のレイアウト（タブ列 + 右端の操作を 1 行に収める）
//!
//! [`fandhe_frontend_pre_styled_ui::tabs::tabs`] は headless 層が root へ
//! 呼び出し側 attrs を渡す引数を持たないため（`tabs` rustdoc 参照）、
//! タブ列と同じ行の右端へ操作を差し込む場所が無い。そこでカード root
//! （[`card::root`]。フォーカスリング分の余白として `padding: 0.25rem`
//! のみ持つ）をグリッドの入れ物にし、tabs root を `display: contents`
//! にして、タブ列（1 列目・1 行目）・操作 div（[`LAYOUT_CSS`] の
//! `data-blocks-example-preview-tabs-actions`、2 列目・1 行目）・パネル
//! （`grid-column: 1 / -1` で 2 行目）をカードのグリッドへ直接並べる
//! （`hidden` の付いたパネルは `display: none` でグリッドの枠を取らない）。
//! tabs root は role を持たない素の `div` のため、`display: contents`
//! によるアクセシビリティ上の副作用は無い。tabs root は属性を受け取れない
//! ため、カード側の `data-blocks-example-preview-tabs-frame` 属性から
//! 子結合子（`>`）で選択する。カードのグリッド宣言自体は `card` recipe の
//! `[data-scope="card"][data-part="root"]`（詳細度 2）が出す
//! `display: flex` に上書きされないよう、同じ要素が持つ
//! `data-scope="card"`/`data-part="root"`/`data-blocks-example-preview-
//! tabs-frame` の 3 属性を束ねたセレクタ（詳細度 3）で宣言する。
//!
//! DOM の順序は「タブ列 → 操作 div」で、フォーカス順は「タブ → パネル →
//! 操作」になる。コピーは内容に付随する操作のため、この順序で問題ない。
//!
//! # パネルの中身
//!
//! - **プレビュー**: [`button::button`] を 2 つ（「保存する」/
//!   「キャンセル」）。どちらもクリック処理を持たないため `disabled: true`
//!   にする（`code_block_header` の補助ボタンと同じ判断）。
//! - **コード**: 素の `pre` の中に [`code::code`] を置き、プレビューを
//!   組み立てる短い Rust コード片（[`SNIPPET`]）を表示する。内容はプレビュー
//!   関数と一致させ `disabled: true` も含める（コピーした片だけで成り立つ
//!   自己完結の例、`code_block_header` のレビュー是正と同じ判断）。
//!   [`code::code`] は本来インラインコード片用の recipe（既定 `Subtle`
//!   variant で背景色・padding・角丸を持つ「pill」の見た目）のため、複数行
//!   スニペットをそのまま包むとコードブロックではなく巨大なピルに見える。
//!   `code_block_header` と同じ是正として、`[data-blocks-example-preview-
//!   tabs-code]` フックへ `display: block`/`background: transparent`/
//!   `border: 0`/`padding: 0`/`color: inherit` を [`LAYOUT_CSS`] 側から
//!   直接宣言し、Subtle variant のインライン装飾を打ち消す（PR #3552
//!   Bugbot Medium「Code panel keeps inline styles」対応）。コード
//!   パネル自体（`[data-scope="tabs"][data-part="content"]`）に
//!   `overflow-x: auto` を宣言して長い行を枠内で横スクロールする。
//!   headless 層が既にパネルへ `tabindex="0"` を付与しているため、この
//!   横スクロールはキーボードから直接到達できる（内側の `pre`
//!   〔`data-blocks-example-preview-tabs-code-panel`〕側に `tabindex` を
//!   重ねて二重のタブ停止を作らない）。カード root の `overflow: hidden`
//!   でフォーカスリングが切れないよう、パネルの `:focus-visible` は
//!   inset の outline（負の `outline-offset`）にする（`code_block_header`
//!   と同じ是正）。同じ理由でタブトリガー・コピー操作の各トリガーも
//!   `:focus-visible` を `outline-offset: -2px` の inset へ揃える
//!   （[`LAYOUT_CSS`] 末尾のルール）。
//! - **操作**: [`clipboard::root`] は未コピー（idle）状態で固定し
//!   `data-copied` は出さない。docs サイトは JS ハイドレーションを行わず
//!   `navigator.clipboard` の配線を持たないため、[`clipboard::trigger`]
//!   自体に `disabled`/`data-disabled`/`aria-disabled="true"` を直接付与し
//!   押せない状態にする（「id」節の下、ヘッダー節末尾の指摘対応）。headless
//!   層の `trigger` は `data-disabled` を CSS 側で消費しない設計（§「意図的
//!   非採用」の disabled 視覚）なので、`[data-blocks-example-preview-tabs-
//!   actions]` スコープで `[data-scope="clipboard"][data-part="trigger"]
//!   [data-disabled]` に `opacity: 0.5`/`cursor: not-allowed` を
//!   [`LAYOUT_CSS`] 側から直接宣言し、`button::button` が使う
//!   `disabled_declarations`（`crates/pre-styled-ui/src/recipe.rs`）と同じ
//!   値で見た目も押せない状態に揃える（PR #3552 Bugbot Medium「Copy
//!   trigger lacks disabled styling」対応）。
//!   [`button::button`]（Ghost/Sm/`disabled: true`）で「外部で開く」を
//!   置く。`href`/`target` は一切出さず、リンク先の無い合成例のため
//!   disabled のボタンにする（死にリンク・reverse tabnabbing を避ける、
//!   `code_block_header` B と同じ判断）。
//!
//! # id（1 root : 1 状態機械契約への対応）
//!
//! [`fandhe_frontend_pre_styled_ui::clipboard`] は「1 マウントルート :
//! 1 状態機械」契約を持つため、A/B の外枠それぞれへ一意の `id`
//! （`blocks-example-preview-tabs-a`/`-b`）を付与する
//! （`code_block_header`・`notification_tray_tabs` と同型の判断）。
//! `blocks_contract.rs` の id 重複禁止にも従う。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`card::root`]/[`code::code`]/[`button::button`]/[`clipboard::root`] は
//! いずれも `drop_class_attr` で呼び出し側 `class` を除去する契約のため、
//! 部品への CSS フックは `data-blocks-example-preview-tabs-*` 属性で渡す。
//! 素の `div`/`pre` は `class` がそのまま効くため
//! `.blocks-example-preview-tabs-*` クラスを使う
//! （`code_block_header`/`notification_tray_tabs` と同型の判断）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンはすべて `type="button"`（`button::button`/
//! `tabs::tabs` いずれも既定・固定で `type="button"`）。
//!
//! # コード・文言はすべて無害
//!
//! コード片は本フレームワーク自身のノード木 API を使う短い Rust のみで、
//! 実在の秘密情報・トークンらしき文字列・実企業名・個人情報を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/example-preview-tabs/",
    title: "example-preview-tabs",
    category: BlockCategory::ExamplePreview,
    rust_source: "crates/docs-site/src/blocks/docs/example_preview/example_preview_tabs.rs",
    demo_class: "blocks-example-preview-tabs",
    parts: &[
        Part {
            label: "Tabs",
            path: "/themes/tabs/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
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
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `example_preview_tabs` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節、他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。タブ列と右端の操作を 1 行に収める
/// グリッド配置（モジュール doc「ヘッダー行のレイアウト」節参照）と、
/// コードパネルの横スクロール・フォーカスリングをここで実装する。
///
/// ヘッダー行グリッドの `align-items: end` は、tab list・actions の 2
/// アイテムの罫線（`border-bottom`）をグリッド行の下端へ揃えるための
/// 選択（`center` だと両者の高さの違いにより罫線がずれる）。tab list の
/// `overflow-x: auto`（spec 上 `overflow-y` も自動的に `auto` 化される）
/// による選択中タブの下線（indicator の `border-bottom`）クリップ対策
/// として、`overflow-y: hidden` を明示しつつ `padding-bottom: 2px` +
/// `margin-bottom: -2px` でスクロール領域に下線の描画余地を確保し、外側
/// レイアウトへの影響を打ち消す。
const LAYOUT_CSS: &str = "\
.blocks-example-preview-tabs-layout {\n  display: flex;\n  flex-direction: column;\n  gap: 1.5rem;\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-example-preview-tabs-frame] {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr) auto;\n  align-items: end;\n  overflow: hidden;\n  min-width: 0;\n  padding: 0.25rem;\n}\n\
[data-blocks-example-preview-tabs-frame] > [data-scope=\"tabs\"][data-part=\"root\"] {\n  display: contents;\n}\n\
[data-blocks-example-preview-tabs-frame] [data-scope=\"tabs\"][data-part=\"list\"] {\n  grid-column: 1;\n  grid-row: 1;\n  min-width: 0;\n  overflow-x: auto;\n  overflow-y: hidden;\n  flex-wrap: nowrap;\n  padding-bottom: 2px;\n  margin-bottom: -2px;\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-example-preview-tabs-actions] {\n  grid-column: 2;\n  grid-row: 1;\n  display: flex;\n  flex: none;\n  align-items: center;\n  gap: 0.5rem;\n  white-space: nowrap;\n  padding-inline-end: 1rem;\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-example-preview-tabs-actions] [data-scope=\"clipboard\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 0.5;\n  cursor: not-allowed;\n}\n\
[data-blocks-example-preview-tabs-frame] [data-scope=\"tabs\"][data-part=\"content\"] {\n  grid-column: 1 / -1;\n  grid-row: 2;\n  padding: 1rem;\n  overflow-x: auto;\n}\n\
.blocks-example-preview-tabs-preview {\n  display: flex;\n  gap: 0.75rem;\n}\n\
[data-blocks-example-preview-tabs-code-panel] {\n  margin: 0;\n  font-family: var(--fandhe-font-font-mono);\n}\n\
[data-scope=\"code\"][data-part=\"root\"][data-blocks-example-preview-tabs-code] {\n  display: block;\n  white-space: pre;\n  background: transparent;\n  border: 0;\n  padding: 0;\n  color: inherit;\n}\n\
[data-blocks-example-preview-tabs-frame] [data-scope=\"tabs\"][data-part=\"content\"]:focus-visible {\n  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));\n}\n\
[data-blocks-example-preview-tabs-frame] [data-scope=\"tabs\"][data-part=\"trigger\"]:focus-visible,\n[data-blocks-example-preview-tabs-actions] [data-scope=\"clipboard\"][data-part=\"trigger\"]:focus-visible,\n[data-blocks-example-preview-tabs-actions] button:focus-visible {\n  outline-offset: -2px;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, SNIPPET};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// Demo が `<form>` を出力せず、`data-copied`・死にリンク・`data:` URI
    /// を一切持たないこと（`code_block_header` と同型の固定）。
    #[test]
    fn demo_has_no_form_data_copied_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert_eq!(html.matches("data-copied").count(), 0);
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("target=\"_blank\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// A/B がそれぞれ独立した `id` を持つこと（モジュール doc「id」節）。
    #[test]
    fn demo_has_distinct_mount_root_ids_for_a_and_b() {
        let html = demo_html();
        assert!(html.contains(r#"id="blocks-example-preview-tabs-a""#));
        assert!(html.contains(r#"id="blocks-example-preview-tabs-b""#));
    }

    /// A はコードパネルが `hidden`、B はプレビューパネルが `hidden`
    /// （選択状態の固定、モジュール doc「実物 `tabs::tabs` を使い…」節）。
    #[test]
    fn instance_a_hides_code_and_instance_b_hides_preview() {
        let html = demo_html();
        assert!(html.contains(
            r#"id="blocks-example-preview-tabs-a-tabs-content-code" role="tabpanel" aria-labelledby="blocks-example-preview-tabs-a-tabs-trigger-code" data-state="inactive" data-orientation="horizontal" tabindex="0" hidden="#
        ));
        assert!(html.contains(
            r#"id="blocks-example-preview-tabs-b-tabs-content-preview" role="tabpanel" aria-labelledby="blocks-example-preview-tabs-b-tabs-trigger-preview" data-state="inactive" data-orientation="horizontal" tabindex="0" hidden="#
        ));
        assert!(!html.contains(
            r#"id="blocks-example-preview-tabs-a-tabs-content-preview" role="tabpanel" aria-labelledby="blocks-example-preview-tabs-a-tabs-trigger-preview" data-state="inactive""#
        ));
        assert!(!html.contains(
            r#"id="blocks-example-preview-tabs-b-tabs-content-code" role="tabpanel" aria-labelledby="blocks-example-preview-tabs-b-tabs-trigger-code" data-state="inactive""#
        ));
    }

    /// 非選択タブが `disabled` になること（A/B 合わせて 2 個、モジュール doc
    /// 「実物 `tabs::tabs` を使い…」節）。タブ以外（プレビューの補助ボタン・
    /// 外部で開くボタン）も `disabled` を持つため、`data-scope="tabs"
    /// data-part="trigger"` の区画ごとに区切って数える。
    #[test]
    fn non_selected_tabs_are_disabled() {
        let html = demo_html();
        let disabled_trigger_count = html
            .split("data-scope=\"tabs\" data-part=\"trigger\"")
            .skip(1)
            .filter(|segment| {
                let end = segment.find("</button>").unwrap_or(segment.len());
                segment[..end].contains("disabled")
            })
            .count();
        assert_eq!(disabled_trigger_count, 2);
    }

    /// 「外部で開く」ボタンが disabled であること（A/B 合わせて 2 個）。
    #[test]
    fn external_open_buttons_are_disabled() {
        let html = demo_html();
        assert_eq!(html.matches("外部で開く").count(), 2);
    }

    /// コピーの clipboard トリガーが disabled であること（A/B 合わせて 2 個、
    /// モジュール doc「パネルの中身」節「操作」項。JS ハイドレーションなしで
    /// 機能しない操作を押せる状態にしない、PR #3552 codex P1 対応）。
    #[test]
    fn clipboard_triggers_are_disabled() {
        let html = demo_html();
        let disabled_clipboard_trigger_count = html
            .split("data-scope=\"clipboard\" data-part=\"trigger\"")
            .skip(1)
            .filter(|segment| {
                let end = segment.find("</button>").unwrap_or(segment.len());
                let segment = &segment[..end];
                segment.contains("disabled=\"\"")
                    && segment.contains("data-disabled")
                    && segment.contains("aria-disabled=\"true\"")
            })
            .count();
        assert_eq!(disabled_clipboard_trigger_count, 2);
    }

    /// [`SNIPPET`] がプレビューの両ボタン文言を含む自己完結の例であること
    /// （コピーした片だけで成り立つ、`code_block_header` と同型の固定）。
    #[test]
    fn snippet_matches_preview_buttons() {
        assert!(SNIPPET.contains("保存する"));
        assert!(SNIPPET.contains("キャンセル"));
        assert!(SNIPPET.contains("fn demo("));
    }

    /// [`LAYOUT_CSS`] がグリッド配置・横スクロール・inset フォーカスリングの
    /// 各セレクタを含み、HTML タグ破りを起こす `<` を含まないこと。
    #[test]
    fn layout_css_declares_grid_and_scroll_rules() {
        assert!(LAYOUT_CSS.contains("display: contents;"));
        assert!(LAYOUT_CSS.contains("overflow-x: auto;"));
        assert!(
            LAYOUT_CSS.contains("outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));")
        );
        // frame のグリッド宣言は `card` recipe の `[data-scope="card"]
        // [data-part="root"]`（詳細度 2）が出す `display: flex` に負けない
        // よう、3 属性束ねで詳細度 3 にする（PR #3552 Bugbot High 対応）。
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"card\"][data-part=\"root\"][data-blocks-example-preview-tabs-frame] {\n  display: grid;"
        ));
        // タブトリガー・clipboard トリガーの `:focus-visible` も inset に
        // 揃え、カード root の `overflow: hidden` でリングが切れないように
        // する（PR #3552 Bugbot Medium「Overflow clips header focus rings」
        // 対応）。
        assert!(LAYOUT_CSS.matches("outline-offset: -2px;").count() >= 1);
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// disabled にした clipboard トリガーが `[data-disabled]` で
    /// `opacity`/`cursor` を是正されていること（PR #3552 Bugbot Medium
    /// 「Copy trigger lacks disabled styling」対応。headless 層が
    /// `data-disabled` を CSS 側で消費しない設計のため、block 固有 CSS
    /// 側で直接宣言する）。
    #[test]
    fn layout_css_styles_disabled_clipboard_trigger() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"clipboard\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 0.5;\n  cursor: not-allowed;\n}"
        ));
    }

    /// コードタブの `[data-blocks-example-preview-tabs-code]` フックが
    /// `code::code`（既定 Subtle variant）のインライン装飾（背景色・
    /// padding・角丸）を打ち消し、複数行スニペットがピルではなくコード
    /// ブロックとして表示されること（PR #3552 Bugbot Medium「Code panel
    /// keeps inline styles」対応）。
    #[test]
    fn layout_css_resets_code_inline_chrome() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"code\"][data-part=\"root\"][data-blocks-example-preview-tabs-code] {\n  display: block;\n  white-space: pre;\n  background: transparent;\n  border: 0;\n  padding: 0;\n  color: inherit;\n}"
        ));
    }
}

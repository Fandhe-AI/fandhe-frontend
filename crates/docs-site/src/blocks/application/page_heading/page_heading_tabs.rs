//! `page-heading-tabs` block（イシュー #2934。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充」配下）。「タブ付きのページ見出し」の合成例。
//!
//! # 使用部品
//!
//! `heading` / `tab_nav` / `button` / `native_select` / `segment_group` /
//! `menu` の 6 部品のみを合成する（[`BLOCK`] の `parts` に一致させる契約）。
//! 新しい UI 部品は追加しない。
//!
//! # `tabs` は使わず `tab_nav` のみを使う
//!
//! `tabs::tabs` は非選択パネルが `hidden` になり、無 JS の docs サイトでは
//! そこへ到達する経路がない（`feature_tabs_panel` 等 3 block で過去に
//! 是正された前例）。本 block は「ページ見出し」であり切替先の本文を含ま
//! ない Demo であるため、リンク集合として無 JS で正しく機能する
//! `tab_nav::root`/`tab_nav::link` を使う。`role="tab"` は一切出力しない。
//! `aria-current="page"` は付与しない（次節「`aria-current` を付与しない」
//! 参照）。
//!
//! # `aria-current` を付与しない（イシュー #2934 codex レビュー P2 是正）
//!
//! `tab_nav::link` は `current: true` で `aria-current="page"` を固定出力
//! する API だが、本 Demo は無 JS の静的 SSR ページ内フラグメントリンクで
//! あり、利用者がリンクをクリックして別セクションへ移動しても
//! （ブラウザ側のフラグメント遷移のみで）サーバー再レンダーは起きないため
//! `aria-current` を追随更新する手段がない。固定表示のまま放置すると
//! 移動後も古いリンクを「現在位置」と支援技術へ誤って伝え続けるため、
//! `section_tabs` はすべてのリンクへ `current: false` を渡し
//! `aria-current` を一切出力しない（同期不能な状態表示は撤去する判断）。
//!
//! # 4 インスタンスで配置パターンを表現する（無 JS のため静的併記）
//!
//! 1 block・4 インスタンス縦積みの構成。
//!
//! - **below**: 見出し + 操作ボタン 2 個を上段、セクションタブを下段。
//! - **inline**: 見出し・タブ・操作ボタン 1 個を 1 行に横並び（狭幅は縦積み）。
//! - **above**: セクションタブを上段、見出し + 操作行（並び替えメニュー・
//!   表示切替・新規作成ボタン）を下段。
//! - **filter**: 見出し + 期間フィルタ（`tab_nav` は常時表示、`40rem` 未満は
//!   `native_select` を補助表示として併記する）。
//!
//! # 狭幅対応
//!
//! `below`/`inline`/`above`/`filter` はいずれもタブ列を横スクロール
//! コンテナ（`overflow-x: auto`）にし、狭幅でも `display: none` で隠さず
//! 常時表示のまま保つ（`tab_nav::link` が無 JS で機能する唯一の遷移手段の
//! ため、狭幅で到達不能にしない。イシュー #2934 codex レビュー P1 是正）。
//! `filter` の `native_select` は `40rem` 未満のみの補助表示（操作しても
//! 反映されない `disabled: true` のため、あくまで tab_nav の補足）。
//!
//! # メニュー・SegmentGroup は無 JS のため静的固定
//!
//! 並び替えメニューは閉じた状態で固定する（`page_heading_actions.rs`
//! の `overflow_menu` と同型）。表示切替の `segment_group` はネイティブ
//! 切替を構造的に禁止するため item 系パーツ全体へ `disabled: true` を渡す
//! （`form_layout_property_panel.rs::layout_section` と同型。無 JS では
//! indicator が追随せず表示と状態がずれるため）。
//!
//! # `tab_nav::link` のフラグメントリンク先を実在させる
//!
//! `site` の linkcheck（`crates/docs-site/src/linkcheck.rs`）は同一ページ内
//! アンカー（`href="#foo"`）の参照先 `id` がページ内に実在することを
//! fail-closed に検証する。`section_markers` が `section_tabs` と同じ
//! `(id, label, current)` の組から `id` 付きの小見出しを出力し、各タブの
//! リンク先を実在させる（`content_article_toc.rs::article_body` と同型の
//! 「目次リンク先に見出し `id` を実在させる」規約）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先を
//! 持たない。`tab_nav::link` の `href` はフラグメント（`#overview` 等）の
//! みで、実在しない外部 URL・`data:` URI は使わない。文言はすべて独自の
//! 架空の日本語ダミー（実企業名・実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::segment_group::{self, SegmentGroupProps};
use fandhe_frontend_pre_styled_ui::tab_nav;
use fandhe_frontend_pre_styled_ui::Size;

/// 見出し（`H1`/`H2`・`HeadingSize::Xl` 固定）。
fn heading_block(level: HeadingLevel, title: &'static str) -> Node {
    heading(
        level,
        &HeadingProps {
            size: HeadingSize::Xl,
            ..HeadingProps::default()
        },
        vec![],
        vec![text(title)],
    )
}

/// セクションタブ（横スクロールラッパ付き）。`items` は `(id, label,
/// current)` の組。`id` は `#` なしのフラグメント名で、[`section_markers`]
/// が同じ `id` を持つ見出しを出力することでリンク先を実在させる
/// （linkcheck の「同一ページ内アンカーの参照先実在」契約、モジュール doc
/// 参照）。`current` はここでは使わず（モジュール doc「`aria-current` を
/// 付与しない」節参照）、`period_filter` の初期選択導出専用として残す。
fn section_tabs(aria_label: &'static str, items: &[(&'static str, &'static str, bool)]) -> Node {
    let link_nodes: Vec<Node> = items
        .iter()
        .map(|(id, label, _current)| {
            let href = format!("#{id}");
            tab_nav::link(&href, false, vec![], vec![text(*label)])
        })
        .collect();
    div(
        vec![("data-blocks-page-heading-tabs-tab-scroller", "")],
        vec![tab_nav::root(Size::Md, aria_label, vec![], link_nodes)],
    )
}

/// `section_tabs` の各リンクが指すページ内アンカーを実在させるための、
/// 小さな見出しの並び（`content_article_toc.rs::article_body` と同型の
/// 「目次リンク先に見出し `id` を実在させる」規約）。`items` は
/// `section_tabs` と同じ `(id, label, current)` の組を渡す。
///
/// パネル内の `H1` 直下の見出しであるため `HeadingLevel::H2` を使う
/// （`H1` → `H4` への階層飛び越えの是正、イシュー #2934 レビュー指摘）。
/// 見た目は `HeadingSize::Sm` で維持する。
fn section_markers(items: &[(&'static str, &'static str, bool)]) -> Node {
    let marker_nodes: Vec<Node> = items
        .iter()
        .map(|(id, label, _)| {
            heading(
                HeadingLevel::H2,
                &HeadingProps {
                    size: HeadingSize::Sm,
                    ..HeadingProps::default()
                },
                vec![("id", id)],
                vec![text(*label)],
            )
        })
        .collect();
    div(
        vec![("data-blocks-page-heading-tabs-sections", "")],
        marker_nodes,
    )
}

/// 並び替えメニュー（無 JS のため閉じた状態で固定する。モジュール doc
/// 「メニュー・SegmentGroup は無 JS のため静的固定」節参照）。
fn sort_menu() -> Node {
    let content_id = "blocks-page-heading-tabs-sort-menu";
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id),
        vec![("aria-label", "並び替え")],
        vec![text("並び替え")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id),
        None,
        vec![],
        vec![
            menu::item("name", false, false, vec![], vec![text("名前順")]),
            menu::item("updated", false, false, vec![], vec![text("更新日順")]),
            menu::item("created", false, false, vec![], vec![text("作成日順")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// 表示切替（`segment_group`。ネイティブ切替を構造的に禁止するため item 系
/// パーツ全体を `disabled: true` にする、モジュール doc参照）。
fn view_switch() -> Node {
    let label_id = "blocks-page-heading-tabs-view-label";
    let props = SegmentGroupProps {
        disabled: true,
        ..SegmentGroupProps::default()
    };
    div(
        vec![],
        vec![
            el("span", vec![("id", label_id)], vec![text("表示")]),
            segment_group::root_with_props(
                Size::Sm,
                &props,
                None,
                Some(label_id),
                vec![],
                vec![
                    segment_group::indicator(Some((0, 2)), &props, None, vec![]),
                    segment_group::item(
                        true,
                        &props,
                        "list",
                        vec![],
                        vec![
                            segment_group::item_hidden_input(
                                true,
                                &props,
                                Some("blocks-page-heading-tabs-view"),
                                "list",
                                vec![],
                            ),
                            segment_group::item_control(true, &props, vec![]),
                            segment_group::item_text(true, &props, vec![], vec![text("一覧")]),
                        ],
                    ),
                    segment_group::item(
                        false,
                        &props,
                        "card",
                        vec![],
                        vec![
                            segment_group::item_hidden_input(
                                false,
                                &props,
                                Some("blocks-page-heading-tabs-view"),
                                "card",
                                vec![],
                            ),
                            segment_group::item_control(false, &props, vec![]),
                            segment_group::item_text(false, &props, vec![], vec![text("カード")]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 期間フィルタ。`tab_nav` は常時表示（狭幅でも `display: none` にしない。
/// モジュール doc「狭幅対応」節参照）、`native_select` は `40rem` 未満のみの
/// 補助表示。
///
/// 選択状態は `period_items`（`tab_nav` の現在位置）を単一の真実源とし、
/// `native_select` の初期選択（`option` の `selected`）をそこから導出する
/// ことで、狭幅/広幅間で表示される初期選択が食い違わないようにする。
/// 本 Demo は静的な合成例であり選択操作をフォーム送信・状態更新へ結び付ける
/// 手段を持たないため、`native_select` 自体を `disabled: true` にして
/// 「操作しても変わらないように見える」誤りを防ぐ（`view_switch` の
/// `segment_group` と同型の無 JS 対応、モジュール doc「メニュー・
/// SegmentGroup は無 JS のため静的固定」節参照）。狭幅での実際の遷移手段は
/// 常時表示の `tab_nav` が担う（イシュー #2934 codex レビュー P1 是正）。
fn period_filter() -> Node {
    let period_props = FieldProps {
        id: "blocks-page-heading-tabs-period",
        ids: FieldIds::default(),
        disabled: true,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let period_items = [
        ("period-today", "今日", false),
        ("period-7d", "7 日間", true),
        ("period-30d", "30 日間", false),
        ("period-all", "全期間", false),
    ];
    let options = [
        ("today", "今日"),
        ("7d", "7 日間"),
        ("30d", "30 日間"),
        ("all", "全期間"),
    ];
    let selected_value =
        period_items
            .iter()
            .find(|(_, _, current)| *current)
            .map_or(options[0].0, |(id, _, _)| {
                // period_items の id（`period-` 接頭辞付き）から options の value
                // （接頭辞なし）を導出する。
                id.trim_start_matches("period-")
            });
    let option_nodes: Vec<Node> = options
        .iter()
        .map(|(value, label)| {
            let mut attrs = vec![("value", *value)];
            if *value == selected_value {
                attrs.push(("selected", "selected"));
            }
            el("option", attrs, vec![text(*label)])
        })
        .collect();
    let select = native_select::native_select(
        &NativeSelectProps::default(),
        &period_props,
        vec![("aria-label", "期間")],
        option_nodes,
    );
    let tabs = section_tabs("期間の絞り込み", &period_items);
    div(
        vec![("data-blocks-page-heading-tabs-period-fields", "")],
        vec![
            div(
                vec![("data-blocks-page-heading-tabs-period-select", "")],
                vec![select],
            ),
            div(
                vec![("data-blocks-page-heading-tabs-period-tabs", "")],
                vec![tabs],
            ),
            section_markers(&period_items),
        ],
    )
}

/// below: 見出し + 操作ボタン 2 個を上段、セクションタブを下段。
fn panel_below() -> Node {
    let actions = div(
        vec![("data-blocks-page-heading-tabs-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("書き出す")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("新規作成")]),
        ],
    );
    let header_row = div(
        vec![("data-blocks-page-heading-tabs-header-row", "")],
        vec![heading_block(HeadingLevel::H1, "プロジェクト一覧"), actions],
    );
    let below_items = [
        ("overview", "概要", true),
        ("members", "メンバー", false),
        ("settings", "設定", false),
        ("history", "履歴", false),
    ];
    let tabs = section_tabs("ページ内セクション", &below_items);
    div(
        vec![
            ("data-blocks-page-heading-tabs-panel", ""),
            ("data-blocks-page-heading-tabs-variant", "below"),
        ],
        vec![header_row, tabs, section_markers(&below_items)],
    )
}

/// inline: 見出し・タブ・操作ボタンを 1 行に横並び（狭幅は縦積み）。
fn panel_inline() -> Node {
    let inline_items = [
        ("board", "ボード", true),
        ("list", "リスト", false),
        ("calendar", "カレンダー", false),
    ];
    let tabs = section_tabs("表示切り替え", &inline_items);
    let row = div(
        vec![("data-blocks-page-heading-tabs-inline-row", "")],
        vec![
            heading_block(HeadingLevel::H1, "タスク"),
            tabs,
            button::button(&ButtonProps::default(), vec![], vec![text("新規作成")]),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-tabs-panel", ""),
            ("data-blocks-page-heading-tabs-variant", "inline"),
        ],
        vec![row, section_markers(&inline_items)],
    )
}

/// above: セクションタブを上段、見出し + 操作行を下段。
fn panel_above() -> Node {
    let above_items = [
        ("docs", "ドキュメント", true),
        ("media", "メディア", false),
        ("archive", "アーカイブ", false),
    ];
    let tabs = section_tabs("コンテンツ種別", &above_items);
    let actions = div(
        vec![("data-blocks-page-heading-tabs-actions", "")],
        vec![
            sort_menu(),
            view_switch(),
            button::button(&ButtonProps::default(), vec![], vec![text("新規作成")]),
        ],
    );
    let header_row = div(
        vec![("data-blocks-page-heading-tabs-header-row", "")],
        vec![heading_block(HeadingLevel::H1, "ファイル"), actions],
    );
    div(
        vec![
            ("data-blocks-page-heading-tabs-panel", ""),
            ("data-blocks-page-heading-tabs-variant", "above"),
        ],
        vec![tabs, header_row, section_markers(&above_items)],
    )
}

/// filter: 見出し + 期間フィルタ（`tab_nav` は常時表示、狭幅は `native_select` を補助併記）。
fn panel_filter() -> Node {
    div(
        vec![
            ("data-blocks-page-heading-tabs-panel", ""),
            ("data-blocks-page-heading-tabs-variant", "filter"),
        ],
        vec![heading_block(HeadingLevel::H1, "利用ログ"), period_filter()],
    )
}

/// `page-heading-tabs` の Demo 本体（4 インスタンスを縦積みで並記する。
/// 呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-page-heading-tabs-layout")],
        vec![panel_below(), panel_inline(), panel_above(), panel_filter()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/page-heading-tabs/",
    title: "page-heading-tabs",
    category: BlockCategory::PageHeading,
    rust_source: "crates/docs-site/src/blocks/application/page_heading/page_heading_tabs.rs",
    demo_class: "blocks-page-heading-tabs",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Tab Nav",
            path: "/themes/tab-nav/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Segment Group",
            path: "/themes/segment-group/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `page_heading_tabs` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型で、本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-page-heading-tabs-*` と
/// `[data-blocks-page-heading-tabs-*]` のみを用いる。ルート class は
/// `demo_class`（`blocks-page-heading-tabs`）と別名の `-layout` にする
/// （`card_heading_toolbar.rs` 等と同じ Bugbot 教訓の回避）。
///
/// `view_switch` の `segment_group` は無 JS のため item 系パーツ全体を
/// `disabled: true` にしているが、既定の `[data-disabled]` スタイルは
/// opacity を落とすため無効化した表示切替だけが周囲より薄く見えてしまう
/// （`form_layout_property_panel.rs` と同じ既知パターン）。`item`/
/// `item-control`/`item-text` の `[data-disabled]` へ opacity を 1 に
/// 戻す中和 CSS を重ねる（イシュー #2934 レビュー指摘の是正）。
const LAYOUT_CSS: &str = "\
.blocks-page-heading-tabs-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-page-heading-tabs-panel] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-4);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
[data-blocks-page-heading-tabs-header-row] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-page-heading-tabs-inline-row] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-page-heading-tabs-actions] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-page-heading-tabs-tab-scroller] {\n  overflow-x: auto;\n  white-space: nowrap;\n  min-width: 0;\n}\n\
[data-blocks-page-heading-tabs-period-fields] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-page-heading-tabs-period-select] {\n  display: block;\n}\n\
[data-blocks-page-heading-tabs-sections] {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-4);\n  padding-top: var(--fandhe-space-2);\n  border-top: 1px dashed var(--fandhe-color-border);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-page-heading-tabs-layout [data-scope=\"segment-group\"][data-part=\"item\"][data-disabled],\n\
.blocks-page-heading-tabs-layout [data-scope=\"segment-group\"][data-part=\"item-control\"][data-disabled],\n\
.blocks-page-heading-tabs-layout [data-scope=\"segment-group\"][data-part=\"item-text\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@media (min-width: 40rem) {\n  [data-blocks-page-heading-tabs-header-row] {\n    flex-direction: row;\n    justify-content: space-between;\n    align-items: flex-start;\n  }\n  [data-blocks-page-heading-tabs-inline-row] {\n    flex-direction: row;\n    align-items: center;\n    justify-content: space-between;\n  }\n  [data-blocks-page-heading-tabs-inline-row] > [data-blocks-page-heading-tabs-tab-scroller] {\n    flex: 1 1 auto;\n  }\n  [data-blocks-page-heading-tabs-period-select] {\n    display: none;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/tab-nav/button/native-select/
    /// segment-group/menu）の anatomy をすべて実際に出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"tab-nav\"",
            "data-scope=\"button\"",
            "data-scope=\"field\" data-part=\"select\"",
            "data-scope=\"segment-group\"",
            "data-scope=\"menu\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// `tabs::tabs`（`role="tab"`/`data-scope="tabs"`）を一切使わないこと
    /// （モジュール doc「`tabs` は使わず `tab_nav` のみを使う」節参照）。
    #[test]
    fn demo_never_uses_tabs_role() {
        let html = render(&demo());
        assert!(!html.contains("data-scope=\"tabs\""));
        assert!(!html.contains("role=\"tab\""));
    }

    /// 4 インスタンス（below/inline/above/filter）がすべて出力されること。
    #[test]
    fn demo_panel_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-page-heading-tabs-panel").count(),
            4
        );
    }

    /// `tab_nav` へ `aria-current="page"` を一切付与しないこと（無 JS の
    /// 静的 SSR フラグメントリンクは移動後に追随更新できないため、同期
    /// 不能な状態表示を撤去する。モジュール doc「`aria-current` を付与
    /// しない」節参照、イシュー #2934 codex レビュー P2 是正）。
    #[test]
    fn demo_never_marks_aria_current() {
        let html = render(&demo());
        assert!(!html.contains("aria-current"));
    }

    /// `<form>` を出力しない・`raw_html`/生の `<script` を含まない（静的
    /// 合成例のセキュリティ不変条件）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
    }

    /// `LAYOUT_CSS` が block 固有セレクタのみを使うこと（他 block との
    /// 衝突防止）。
    #[test]
    fn layout_css_uses_only_scoped_selectors() {
        assert!(LAYOUT_CSS.contains(".blocks-page-heading-tabs-layout"));
        assert!(LAYOUT_CSS.contains("[data-blocks-page-heading-tabs-panel]"));
    }
}

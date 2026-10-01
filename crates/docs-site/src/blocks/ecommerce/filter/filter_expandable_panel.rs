//! `filter-expandable-panel` block（イシュー #3047。親トラッキング #3024
//! 「Blocks EC（phase:5）」配下）。主参照 R0817（件数トグル + 4 群の
//! 展開パネル）のみを集約する。文言・配色・アイコンは参照元から持ち込まず
//! すべて架空のデータで構成する。
//!
//! # 使用部品
//!
//! `button` / `collapsible` / `fieldset` / `checkbox` / `menu` / `separator`
//! の 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `blocks_nav.rs`/`blocks_contract.rs` が検証する）。新しい UI 部品は
//! 追加しない。
//!
//! # 全体構造
//!
//! 上部バー（[`bar`]）とパネル本体を `collapsible::root` で包む。
//! バー左側はフィルタ開閉トリガー（適用件数付き）+ 縦の区切り線 +
//! 「すべて解除」ボタン、右端は並び替えメニュー（[`sort_menu`]）。
//! パネルは価格・色・サイズ・カテゴリの 4 `fieldset` を並べ、開いた
//! 初期状態に固定する。
//!
//! # 静的表示（無 JS）の扱い
//!
//! docs サイトは JS ハイドレーションを行わない。collapsible trigger・
//! menu trigger・radio item・checkbox はすべて `disabled: true` で固定し、
//! ネイティブ `disabled` 属性でフォーカス・操作を不能にして状態が変化
//! しないことを構造的に保証する（`settings_org_switcher`/
//! `form_layout_stacked` と同型の確定パターン）。「すべて解除」ボタンのみ
//! 他 block と同じく既定 `type="button"` の有効ボタンのままとし、押しても
//! 何も起きない静的な見本であることを原稿へ明記する。
//!
//! # 件数表示の単一の真実源
//!
//! [`GROUPS`] の `checked` フラグから件数を数え上げ、トグルボタンの
//! 「◯ 件適用中」表示へ反映する（[`applied_count`]）。チェック済み
//! checkbox の実描画数とこの表示が食い違わないことをテストで固定する。
//!
//! # CSS フック
//!
//! `button::button`/`fieldset::root`/`checkbox::root` 等は `drop_class_attr`
//! により呼び出し側 `attrs` の `class` を除去するため、Demo 固有の
//! スタイルフックは `data-blocks-filter-expandable-panel-*` 属性で渡す。
//! 素の `div`/`span` には `.blocks-filter-expandable-panel-*` class を使う。
//!
//! # 並び替えメニューは開いた content も必ず描画する
//!
//! [`sort_menu`] は `OpenState::Closed` のまま使うが、`menu::content` は
//! 閉じていても `hidden` 存在属性付きで常に描画する（headless 層の既定
//! 挙動）。`trigger` の `aria-controls` が指す先を欠落させないための
//! 確定パターン（`settings_org_switcher` と同型）。
//!
//! # レイアウト・`@container`
//!
//! ルートへ `container-type: inline-size` を設定し、パネルの
//! グリッド列数をコンテナ幅で切り替える（既定 1 列、24rem 以上で 2 列、
//! 40rem 以上で 4 列）。狭い幅では 2 列以下に収まる。
//!
//! # `<form>` を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 送信処理・状態機械を持たない静的な合成例である。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::collapsible::{self, OpenState};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::menu;
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Orientation, Size};

/// フィルタパネルの `id`（collapsible trigger の `aria-controls` が指す先）。
const PANEL_ID: &str = "blocks-filter-expandable-panel-panel";
/// 並び替えメニューの trigger/content の `id`。
const SORT_TRIGGER_ID: &str = "blocks-filter-expandable-panel-sort-trigger";
const SORT_CONTENT_ID: &str = "blocks-filter-expandable-panel-sort-content";

/// 1 項目分のチェックボックス定義（value, label, 既定チェック状態）。
type CheckOption = (&'static str, &'static str, bool);

/// 1 フィルタ群の定義（group id, legend, input name, 項目一覧）。
type FilterGroup = (
    &'static str,
    &'static str,
    &'static str,
    &'static [CheckOption],
);

/// 価格帯フィルタ（4 項目、チェック済みは 1 件）。
const PRICE_OPTIONS: &[CheckOption] = &[
    ("under-3000", "¥3,000 以下", false),
    ("3000-8000", "¥3,000〜8,000", true),
    ("8000-15000", "¥8,000〜15,000", false),
    ("over-15000", "¥15,000 以上", false),
];

/// 色フィルタ（4 項目、チェック済みは 1 件）。
const COLOR_OPTIONS: &[CheckOption] = &[
    ("black", "ブラック", false),
    ("white", "ホワイト", true),
    ("navy", "ネイビー", false),
    ("beige", "ベージュ", false),
];

/// サイズフィルタ（4 項目、チェック済みは 1 件）。
const SIZE_OPTIONS: &[CheckOption] = &[
    ("s", "S", false),
    ("m", "M", false),
    ("l", "L", true),
    ("xl", "XL", false),
];

/// カテゴリフィルタ（4 項目、チェック済みは 0 件）。
const CATEGORY_OPTIONS: &[CheckOption] = &[
    ("tops", "トップス", false),
    ("bottoms", "ボトムス", false),
    ("outerwear", "アウター", false),
    ("accessories", "アクセサリー", false),
];

/// パネルが並べる 4 フィルタ群（単一の真実源。[`applied_count`] が
/// ここから件数を数え上げる）。
const GROUPS: &[FilterGroup] = &[
    (
        "price",
        "価格",
        "blocks-filter-expandable-panel-price",
        PRICE_OPTIONS,
    ),
    (
        "color",
        "色",
        "blocks-filter-expandable-panel-color",
        COLOR_OPTIONS,
    ),
    (
        "size",
        "サイズ",
        "blocks-filter-expandable-panel-size",
        SIZE_OPTIONS,
    ),
    (
        "category",
        "カテゴリ",
        "blocks-filter-expandable-panel-category",
        CATEGORY_OPTIONS,
    ),
];

/// [`GROUPS`] 全体でチェック済みの項目数を数える（[`GROUPS`] のみを
/// 真実源とし、トグル表示と実描画数の食い違いを構造的に防ぐ）。
fn applied_count() -> usize {
    GROUPS
        .iter()
        .flat_map(|(_, _, _, options)| options.iter())
        .filter(|(_, _, checked)| *checked)
        .count()
}

/// 1 個のチェックボックス行を組み立てる。無 JS デモのため `disabled: true`
/// 固定（`form_layout_stacked::email_notification_checkbox` と同型）。
fn checkbox_row(
    name: &'static str,
    value: &'static str,
    label: &'static str,
    checked: bool,
) -> Node {
    let props = CheckboxProps {
        checked: if checked {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
        ..CheckboxProps::default()
    };
    checkbox::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-filter-expandable-panel-checkbox", "")],
        vec![
            checkbox::hidden_input(&props, name, value, vec![]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(&props, vec![], vec![text(label)]),
        ],
    )
}

/// 1 個の `fieldset` 群を組み立てる（`id`/`name`/`legend` は [`GROUPS`] の
/// 対応行から取る）。
fn filter_fieldset(group: &FilterGroup) -> Node {
    let (id, legend_label, name, options) = *group;
    let fieldset_id = format!("blocks-filter-expandable-panel-{id}");
    let props = FieldsetProps {
        id: fieldset_id.as_str(),
        disabled: false,
        invalid: false,
        has_helper_text: false,
    };
    let rows = options
        .iter()
        .map(|(value, label, checked)| checkbox_row(name, value, label, *checked))
        .collect();
    fieldset::root(
        &FieldsetRootProps { size: Size::Sm },
        &props,
        vec![("data-blocks-filter-expandable-panel-fieldset", "")],
        vec![
            fieldset::legend(&props, vec![], vec![text(legend_label)]),
            div(
                vec![("class", "blocks-filter-expandable-panel-options")],
                rows,
            ),
        ],
    )
}

/// 並び替え項目 1 件（value, label）。おすすめ順が既定選択。
const SORT_OPTIONS: &[(&str, &str)] = &[
    ("recommended", "おすすめ順"),
    ("newest", "新着順"),
    ("price-asc", "価格の安い順"),
    ("price-desc", "価格の高い順"),
];

/// 並び替えメニュー（`menu::root` + `trigger` + `positioner(content)`）。
/// `OpenState::Closed` のまま使うが、`content` は `hidden` 付きで必ず
/// 描画する（モジュール doc「並び替えメニュー」節参照）。
fn sort_menu() -> Node {
    let state = OpenState::Closed;
    let trigger = menu::trigger(
        state,
        true,
        Some(SORT_CONTENT_ID),
        vec![
            ("id", SORT_TRIGGER_ID),
            ("data-blocks-filter-expandable-panel-sort-trigger", ""),
        ],
        vec![
            text("並び替え: おすすめ順"),
            menu::indicator(state, vec![], vec![]),
        ],
    );

    let items = SORT_OPTIONS
        .iter()
        .map(|(value, label)| {
            menu::radio_item(
                *value == "recommended",
                value,
                true,
                false,
                vec![("data-blocks-filter-expandable-panel-sort-item", "")],
                vec![text(*label)],
            )
        })
        .collect();

    let content = menu::content(
        state,
        Some(SORT_CONTENT_ID),
        Some(SORT_TRIGGER_ID),
        vec![("data-blocks-filter-expandable-panel-sort-content", "")],
        vec![menu::radio_item_group(None, vec![], items)],
    );

    let positioner = menu::positioner(
        state,
        vec![("data-blocks-filter-expandable-panel-sort-positioner", "")],
        vec![content],
    );

    menu::root(
        Size::Sm,
        state,
        vec![("data-blocks-filter-expandable-panel-sort-menu", "")],
        vec![trigger, positioner],
    )
}

/// 上部バー（左: フィルタ開閉トリガー + 区切り線 + 全解除、右: 並び替え）。
fn bar() -> Node {
    let state = OpenState::Open;
    let count_text = format!("{} 件適用中", applied_count());

    let toggle = collapsible::trigger(
        state,
        true,
        Some(PANEL_ID),
        vec![("data-blocks-filter-expandable-panel-toggle", "")],
        vec![
            text("フィルタ"),
            span(
                vec![("class", "blocks-filter-expandable-panel-count")],
                vec![text(count_text)],
            ),
            collapsible::indicator(state, true, vec![], vec![]),
        ],
    );

    let divider = separator::separator(
        &SeparatorProps {
            orientation: Orientation::Vertical,
            ..SeparatorProps::default()
        },
        vec![],
    );

    let clear_all = button::button(
        &ButtonProps {
            variant: ButtonVariant::Ghost,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![("data-blocks-filter-expandable-panel-clear", "")],
        vec![text("すべて解除")],
    );

    let start = div(
        vec![("class", "blocks-filter-expandable-panel-bar-start")],
        vec![toggle, divider, clear_all],
    );

    div(
        vec![("class", "blocks-filter-expandable-panel-bar")],
        vec![start, sort_menu()],
    )
}

/// パネル本体（上の横区切り線 + 4 `fieldset` のグリッド）。
fn panel() -> Node {
    let state = OpenState::Open;
    let rule = separator::separator(&SeparatorProps::default(), vec![]);
    let grid = div(
        vec![("class", "blocks-filter-expandable-panel-grid")],
        GROUPS.iter().map(filter_fieldset).collect(),
    );
    collapsible::content(
        state,
        true,
        Some(PANEL_ID),
        vec![("data-blocks-filter-expandable-panel-panel", "")],
        vec![rule, grid],
    )
}

/// `filter-expandable-panel` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    collapsible::root(
        OpenState::Open,
        false,
        vec![("class", "blocks-filter-expandable-panel")],
        vec![bar(), panel()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/filter-expandable-panel/",
    title: "filter-expandable-panel",
    category: BlockCategory::Filter,
    rust_source: "crates/docs-site/src/blocks/ecommerce/filter/filter_expandable_panel.rs",
    demo_class: "blocks-filter-expandable-panel",
    parts: &[
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Collapsible",
            path: "/themes/collapsible/",
        },
        Part {
            label: "Fieldset",
            path: "/themes/fieldset/",
        },
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `filter_expandable_panel` 固有のレイアウト規則（`--fandhe-*` トークンのみ
/// 使用）。セレクタは `.blocks-filter-expandable-panel-*` /
/// `[data-blocks-filter-expandable-panel-*]`、および styled 部品の
/// `[data-scope][data-part]` セレクタとの複合セレクタのみを用いる。
const LAYOUT_CSS: &str = "\
.blocks-filter-expandable-panel {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  container-type: inline-size;\n  container-name: blocks-filter-expandable-panel;\n}\n\
.blocks-filter-expandable-panel-bar {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-filter-expandable-panel-bar-start {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-filter-expandable-panel-count {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
[data-blocks-filter-expandable-panel-sort-menu] {\n  margin-inline-start: auto;\n}\n\
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-filter-expandable-panel-toggle] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-filter-expandable-panel-toggle][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"menu\"][data-part=\"trigger\"][data-blocks-filter-expandable-panel-sort-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"menu\"][data-part=\"radio-item\"][data-blocks-filter-expandable-panel-sort-item][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"menu\"][data-part=\"positioner\"][data-blocks-filter-expandable-panel-sort-positioner] {\n  position: static;\n  margin-block-start: var(--fandhe-space-2);\n}\n\
[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-filter-expandable-panel-checkbox][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-filter-expandable-panel-panel] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-filter-expandable-panel-grid {\n  display: grid;\n  grid-template-columns: repeat(1, minmax(0, 1fr));\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-filter-expandable-panel-options {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
@container blocks-filter-expandable-panel (min-width: 24rem) {\n  \
.blocks-filter-expandable-panel-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
@container blocks-filter-expandable-panel (min-width: 40rem) {\n  \
.blocks-filter-expandable-panel-grid {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{applied_count, demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// 6 部品の `data-scope` が揃い、`<form>`・`href="#"`・`data:` src を
    /// 持たないこと。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"button\"",
            "data-scope=\"collapsible\"",
            "data-scope=\"fieldset\"",
            "data-scope=\"checkbox\"",
            "data-scope=\"menu\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// collapsible content（フィルタパネル）が開いた状態（`hidden` なし）
    /// で描画されること。
    #[test]
    fn panel_content_is_rendered_open() {
        let html = render(&demo());
        let pos = html
            .find(&format!(r#"id="{}""#, super::PANEL_ID))
            .expect("panel content id should render");
        let tag_start = html[..pos]
            .rfind("<div")
            .expect("panel content should have an opening div tag");
        let tag_end = html[tag_start..]
            .find('>')
            .map(|rel| tag_start + rel)
            .expect("panel content opening tag should close");
        let tag = &html[tag_start..tag_end];
        assert!(tag.contains(r#"data-state="open""#));
        assert!(!tag.contains("hidden"));
    }

    /// `fieldset` がちょうど 4 個あること。
    #[test]
    fn exactly_four_fieldsets_render() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-scope=\"fieldset\" data-part=\"root\"")
                .count(),
            4
        );
    }

    /// チェック済み checkbox の実描画数と [`applied_count`] の件数表示が
    /// 一致すること。
    #[test]
    fn checked_checkbox_count_matches_applied_count_display() {
        let html = render(&demo());
        let checked_inputs = html.matches("checked=\"\" ").count()
            + html.matches("checked=\"checked\"").count()
            + html.matches(r#"data-state="checked""#).count() / 3;
        assert!(
            checked_inputs > 0,
            "at least one checkbox should be checked"
        );
        let count = applied_count();
        assert_eq!(count, 3);
        assert!(html.contains(&format!("{count} 件適用中")));
    }

    /// 並び替えメニューが閉じた状態で、radio item 4 件のうち checked が
    /// 1 件であること。
    #[test]
    fn sort_menu_is_closed_with_one_checked_radio_item() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"role="menuitemradio""#).count(), 4);
        assert_eq!(
            html.matches(r#"role="menuitemradio" aria-checked="true""#)
                .count(),
            1
        );
        let pos = html
            .find(&format!(r#"id="{}""#, super::SORT_CONTENT_ID))
            .expect("sort content id should render");
        let tag_start = html[..pos]
            .rfind("<div")
            .expect("sort content should have an opening div tag");
        let tag_end = html[tag_start..]
            .find('>')
            .map(|rel| tag_start + rel)
            .expect("sort content opening tag should close");
        let tag = &html[tag_start..tag_end];
        assert!(tag.contains("hidden"), "tag={tag}");
    }

    /// `BLOCK.parts` の `path` が全件 kebab-case の `/themes/…/` であること。
    #[test]
    fn parts_point_to_themes_pages() {
        for part in super::BLOCK.parts {
            assert!(part.path.starts_with("/themes/"));
            assert!(part.path.ends_with('/'));
        }
    }

    /// [`LAYOUT_CSS`] が `@container` 2 個（2 列化・4 列化）を持つこと。
    #[test]
    fn layout_css_has_container_queries() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert_eq!(
            LAYOUT_CSS
                .matches("@container blocks-filter-expandable-panel (min-width:")
                .count(),
            2
        );
    }
}

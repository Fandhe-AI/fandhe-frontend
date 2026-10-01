# filter-expandable-panel

商品一覧ページの上部に置く、開閉パネル式のフィルタバーです。
`button` / `collapsible` / `fieldset` / `checkbox` / `menu` / `separator` の
6 部品を合成します。Blocks は既存部品の合成例であり、新しい UI 部品は
追加しません。

左側に適用件数付きのフィルタ開閉トリガーと「すべて解除」ボタン、右端に
並び替えメニューを配置した上部バーの下へ、価格・色・サイズ・カテゴリの
4 つのチェック群を展開パネルとして並べます。パネルはコンテナ幅に応じて
列数を 1 → 2 → 4 と増やし、狭い幅では 2 列以下になります。

主参照は対応表 ID R0817（件数トグル + 4 群の展開パネル）のみで、集約する
別 variant はありません。文言・配色・アイコンは参照元から持ち込まず、
商品カテゴリ・価格帯・色・サイズの選択肢はすべて架空のデータです。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。フィルタ
開閉トリガー・並び替えメニュー・チェックボックス・並び替え項目は
いずれも無効化（disabled）した静的な表示に固定しています。「すべて
解除」ボタンのみ通常どおり有効な `type="button"` ボタンですが、送信先・
処理を持たない見本のため押しても何も起きません。

## Rust コード

```rust
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
///
/// ルート class（`-layout`）は [`Block::demo_class`]（`insert_generated_sections`
/// が外側 `.blocks-demo` へ付与する）とは別名にする。同名にすると
/// [`LAYOUT_CSS`] の `display: flex`・`container-type` 等のレイアウト規則が
/// 外側の共通 Demo 枠にも二重適用されてしまうため（他 block と同型の分離、
/// `docs/design/docs-site-blocks-section.md` 参照）。
pub fn demo() -> Node {
    collapsible::root(
        OpenState::Open,
        false,
        vec![("class", "blocks-filter-expandable-panel-layout")],
        vec![bar(), panel()],
    )
}
```

## 原案差分メモ

- 集約元は R0817 の 1 件のみです。
- パネルを閉じた状態や、適用中フィルタをタグ列として並記する表現は
  入れていません。初期状態をパネルが開いた状態へ固定し、1 枚の静的な
  見本として表示します。
- 並び替えメニューは閉じた状態（`おすすめ順` が選択済み）で表示し、
  トリガーを押しても開かない静的な見本です。

関連情報: [Button](../themes/button.md) / [Collapsible](../themes/collapsible.md) /
[Fieldset](../themes/fieldset.md) / [Checkbox](../themes/checkbox.md) /
[Menu](../themes/menu.md) / [Separator](../themes/separator.md)

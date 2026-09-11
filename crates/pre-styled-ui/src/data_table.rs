//! styled Data Table（shadcn/ui `Data Table` 相当。イシュー #2127、親
//! #2124、祖父トラッキング参照軸 #2001。headless 側 anatomy は #2125）。
//!
//! `fandhe_frontend_headless_ui::data_table`（#2125）が出力する
//! `data-scope="data-table"` の 8 slot（`root`/`toolbar`/`column-header`/
//! `sort-trigger`/`select-all`/`select-row`/`footer`/`selection-count`）へ、
//! shadcn/ui `Data Table` 相当（toolbar・ソート可能列ヘッダー・行選択・
//! 非表示列・footer のページング/件数表示）の意匠を重ねる薄い委譲層で
//! ある。
//!
//! # 責務境界（`.claude/rules/coding-rust.md` §UI 部品の責務境界、規則 1）
//!
//! 行の並べ替え・フィルタ・ページング処理そのもの、選択結果の保持/送信/
//! 永続化、列定義/列順の永続化はアプリケーション責務であり本モジュールは
//! 持たない（headless 側の責務境界をそのまま継承する。
//! `crates/headless-ui/src/data_table.rs` モジュール doc 参照）。本
//! モジュールが担うのは「headless が出力する `data-*` に応じて見た目
//! （hover・ソート方向アイコン・選択行背景・非表示列・loading/empty の
//! 表示状態）を切り替える」までである。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! [`crate::message_scroller`]/[`crate::questionnaire`] と同型。8 パーツ
//! すべてを同名再定義し（呼び出し側 `class` の除去は本モジュールの責務の
//! ため）、headless の型・アクション・属性ヘルパを選択的に再エクスポート
//! する。docs-site は headless-ui へ直接依存しない方針
//! （`crates/docs-site/Cargo.toml` は pre-styled-ui path 依存のみ）のため、
//! `crates/docs-site/src/showcase.rs` 等はこの再エクスポート経由で型を
//! 得る。
//!
//! # 軸を持たない理由
//!
//! `ColorPalette`/`Size` 等の見た目 variant を公開しない（本イシューの
//! スコープ外、モジュール doc 末尾「out-of-scope」節参照）。`data-loading`/
//! `data-empty`/`data-sort`/`data-state`/`data-hidden` は headless の出力を
//! [`StateCondition::Attr`]/[`StateCondition::AttrEq`] で**参照するのみ**
//! とし（`docs/design/pre-styled-ui-data-attr-vocabulary.md` §2.2「役割 B」）、
//! class ベースの [`SlotRecipe::variant`] を持たない。
//!
//! # Themes 推奨の組み立て（`column-header` slot と `table::column-header`
//! の関係）
//!
//! headless doc「イシュータイトルとの差分」節が定める契約どおり、表本体
//! （`<table>`/`<thead>`/`<tbody>`/`<tr>`）は本モジュールでは新規に作らず
//! [`crate::table`] を使う。列ヘッダー/セル/行の表示状態（ソート・列
//! 非表示・行選択）は [`column_attrs`]/[`column_header_attrs`]/
//! [`row_attrs`]（**node を作らない属性ヘルパ**、headless から再エクス
//! ポート）を [`crate::table::column_header`]/[`crate::table::cell`]/
//! [`crate::table::row`] の `attrs` へ渡すことで同一要素に 2 つの
//! `data-scope` を載せずに合成する。**本モジュール自身が持つ
//! [`column_header`]/[`sort_trigger`]（`th`/`button` ノードを実際に生成
//! するパーツ）は Primitives 経路（自前 CSS を書く利用者・headless
//! 単独利用）向けであり、Themes 経路（`table::column_header` へ委譲する
//! 側）の Demo には現れない**。[`select_all`]/[`select_row`] は
//! `<table>` セル階層に直接載る `th`/`td` であるため、Themes 経路でも
//! そのまま使う（`table` 側に対応する `attrs` パーツはない）。
//!
//! # ソート方向アイコンを raw CSS `::after` で表現する理由
//!
//! [`crate::recipe::SlotRecipe::state`] は状態条件付きの疑似要素
//! （`[data-sort="ascending"]::after`）を DSL で表現できない
//! （`SlotRecipe::pseudo_element` は状態条件と合成できない設計、
//! `docs/api/pre-styled-ui-api.md` §4f-11 参照）。[`crate::message_scroller::stylesheet`]
//! と同型に、[`crate::css::serialize_rule`] を使った raw CSS 追記で
//! `sort-trigger` の `[data-sort="ascending"|"descending"|"none"]::after`
//! を [`stylesheet`] へ連結する。`content` 値はすべて `&'static str`
//! リテラル（本モジュール冒頭「セキュリティ不変条件」節参照）。
//!
//! # 選択行背景を自前で持たない理由
//!
//! [`row_attrs`] が付与する `data-selected` は [`crate::table`] の `row`
//! slot が既に持つ state 規則（イシュー #2052、`background:
//! var(--fandhe-color-accent-subtle)`）がそのまま適用される。本モジュール
//! は `row` slot を持たない（`table::row` をそのまま使う設計のため）。
//!
//! # `select-all`/`select-row` が `--fandhe-table-*` を参照しない理由
//!
//! `crates/docs-site/tests/css_var_scope_prefix.rs` は
//! `[data-scope="X"]` を含む規則内の `--fandhe-*` が `--fandhe-X-*`・
//! theme token・共有変数のいずれかであることを要求する。`select-all`/
//! `select-row` は `data-scope="data-table"` の要素であり `table` scope
//! ではないため、独自の `--fandhe-data-table-*` 変数のみを参照する（パ
//! ディング・境界線・幅を [`crate::table`] のセル規則と視覚的に揃えたい
//! 場合は呼び出し側で `--fandhe-data-table-cell-padding` 等を上書きする）。
//!
//! # `empty`/`skeleton` の本体を持たない理由
//!
//! headless [`fandhe_frontend_headless_ui::data_table::DataTable::root`]
//! と同じ理由（`empty-state`/`skeleton` の本体は [`crate::empty_state`]/
//! [`crate::skeleton`] が担う。本モジュールは `root` の `data-empty`/
//! `data-loading`（+ `aria-busy="true"`）という表示状態のみを持つ）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`scope`/`type`）はすべて `&'static str`
//!   リテラルで固定しており、動的値が属性名スロットへ混入する経路はない。
//! - 動的値（列 id・attrs・children テキスト）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - raw CSS 追記は [`crate::css::serialize_rule`]（宣言の fail-closed
//!   検証あり）経由のみで、`content` 値は静的リテラルであり利用者入力を
//!   一切含まない。
//! - 各パーツで呼び出し側 `class` を [`drop_class_attr`] により除去する
//!   （見た目クラスのなりすまし防止、A05 対策）。
//!
//! # out-of-scope（イシュー #2127）
//!
//! - `fandhe-frontend-wasm-full` の DOM 配線（sort-trigger click →
//!   dispatch、indeterminate、`data-hidden` 反映、ページング
//!   `data-disabled`）は後続イシュー #2126 のスコープ。
//! - `ColorPalette`/`Size` 軸の追加、`column-header` slot と
//!   `table::column-header` の統合（同一要素 2 scope 禁止のため現設計を
//!   維持）は必要になれば別イシューとする。
//! - 並べ替え・フィルタ・ページング処理そのもの、選択結果の保持/送信/
//!   永続化、列定義/列順の永続化はアプリケーション責務（責務境界規則 1）。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, serialize_rule};
use crate::recipe::{
    focus_ring_declarations, hover_bg_muted, hover_surface_declarations, transition_declarations,
    FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe, StateCondition,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

// headless 型のうち本モジュールが必要とするのは props/action/属性ヘルパ
// のみ（`crate::message_scroller` と同型の規約）。パーツ関数 8 件は
// 呼び出し側 `class` の除去を担うため同名再定義する。
pub use fandhe_frontend_headless_ui::data_table::{
    column_attrs, column_header_attrs, row_attrs, ColumnHeaderProps, ColumnProps, DataTable,
    DataTableAction, DataTableProps, SortDirection,
};

/// slot 一覧（headless [`fandhe_frontend_headless_ui::data_table`] の
/// anatomy と 1:1、8 パーツ）。
const SLOTS: &[&str] = &[
    "root",
    "toolbar",
    "column-header",
    "sort-trigger",
    "select-all",
    "select-row",
    "footer",
    "selection-count",
];

/// この styled Data Table の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let root_base = vec![
        decl("display", "flex"),
        decl("flex-direction", "column"),
        decl("gap", "var(--fandhe-data-table-gap, var(--fandhe-space-3))"),
        decl("width", "100%"),
    ];

    let toolbar_base = vec![
        decl("display", "flex"),
        decl("flex-wrap", "wrap"),
        decl("align-items", "center"),
        decl("justify-content", "space-between"),
        decl("gap", "var(--fandhe-space-2)"),
    ];

    // Primitives 経路（自前 CSS 利用者）向け。Themes 経路の Demo には
    // 現れない（モジュール doc「Themes 推奨の組み立て」節参照）。UA の
    // `[hidden]{display:none}` をそのまま活かすため `display` は宣言
    // しない。
    let column_header_base = vec![
        decl("text-align", "start"),
        decl(
            "padding",
            "var(--fandhe-data-table-cell-padding, var(--fandhe-space-3) var(--fandhe-space-4))",
        ),
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
        decl("color", "var(--fandhe-color-fg-muted)"),
    ];

    let mut sort_trigger_base = vec![
        decl("display", "inline-flex"),
        decl("align-items", "center"),
        decl("gap", "var(--fandhe-space-1)"),
        decl("margin-inline", "calc(-1 * var(--fandhe-space-2))"),
        decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
        decl("background", "transparent"),
        decl("border", "0"),
        decl("border-radius", "var(--fandhe-radius-md)"),
        decl("color", "inherit"),
        decl("font", "inherit"),
        decl("cursor", "pointer"),
        hover_bg_muted(),
    ];
    sort_trigger_base.extend(transition_declarations(
        "background, color",
        MotionDuration::Fast,
    ));

    let select_cell_base = vec![
        decl(
            "padding",
            "var(--fandhe-data-table-cell-padding, var(--fandhe-space-3) var(--fandhe-space-4))",
        ),
        decl(
            "border-bottom",
            "var(--fandhe-data-table-row-border, 1px solid var(--fandhe-color-border-muted))",
        ),
        decl("vertical-align", "middle"),
        decl("width", "var(--fandhe-data-table-select-width, 2.5rem)"),
        decl("text-align", "center"),
    ];

    let footer_base = vec![
        decl("display", "flex"),
        decl("flex-wrap", "wrap"),
        decl("align-items", "center"),
        decl("justify-content", "space-between"),
        decl("gap", "var(--fandhe-space-3)"),
        decl("padding-block", "var(--fandhe-space-2)"),
    ];

    let selection_count_base = vec![
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl("color", "var(--fandhe-color-fg-muted)"),
        decl("flex", "1 1 auto"),
    ];

    SlotRecipe::new("data-table", SLOTS)
        .base("root", root_base)
        .base("toolbar", toolbar_base)
        .base("column-header", column_header_base)
        .base("sort-trigger", sort_trigger_base)
        .base("select-all", select_cell_base.clone())
        .base("select-row", select_cell_base)
        .base("footer", footer_base)
        .base("selection-count", selection_count_base)
        .state(
            "root",
            StateCondition::Attr("data-loading"),
            vec![decl("cursor", "progress")],
        )
        .state(
            "root",
            StateCondition::Attr("data-empty"),
            vec![
                decl("--fandhe-data-table-empty-min-height", "12rem"),
                decl("min-height", "var(--fandhe-data-table-empty-min-height)"),
            ],
        )
        .state(
            "sort-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "sort-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
}

/// この styled Data Table が生成する静的 CSS 全量を返す（決定的。
/// [`crate::message_scroller::stylesheet`] と同じ契約）。`sort-trigger`
/// の `[data-sort="ascending"|"descending"|"none"]::after`（ソート方向
/// アイコン）の raw CSS 追記を含む（モジュール doc「ソート方向アイコンを
/// raw CSS `::after` で表現する理由」節参照）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();

    const SORT_TRIGGER: &str = r#"[data-scope="data-table"][data-part="sort-trigger"]"#;

    // `content` 値はすべてソースコード中の `&'static str` リテラル
    // （動的値を一切受け付けない、モジュール冒頭「セキュリティ不変
    // 条件」節参照）。
    let rules: [(&str, &str, &str); 3] = [
        (r#"ascending"#, "\"\u{25b2}\"", ""),
        (r#"descending"#, "\"\u{25bc}\"", ""),
        (r#"none"#, "\"\u{2195}\"", "0.5"),
    ];
    for (direction, content_value, opacity) in rules {
        let selector = format!(r#"{SORT_TRIGGER}[data-sort="{direction}"]::after"#);
        let mut declarations = vec![decl("content", content_value)];
        if !opacity.is_empty() {
            declarations.push(decl("opacity", opacity));
        }
        if let Some(rule) = serialize_rule(&selector, &declarations) {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&rule);
        }
    }

    out
}

/// styled `root` パーツを組み立てる。見た目クラスは付与せず、呼び出し側
/// `class` を [`drop_class_attr`] で除去してから
/// [`fandhe_frontend_headless_ui::data_table::DataTable::root`] へそのまま
/// 委譲する。
#[must_use]
pub fn root<'a>(
    props: DataTableProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    DataTable::root(props, drop_class_attr(attrs), children)
}

/// styled `toolbar` パーツ（純スロット）。
#[must_use]
pub fn toolbar<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::data_table::toolbar(drop_class_attr(attrs), children)
}

/// styled `column-header` パーツ（`th`）。Primitives 経路向け（モジュール
/// doc「Themes 推奨の組み立て」節参照）。Themes 経路は
/// [`column_header_attrs`] を [`crate::table::column_header`] の `attrs`
/// へ渡す。
#[must_use]
pub fn column_header<'a>(
    table: &DataTable,
    id: &'a str,
    sortable: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    table.column_header(id, sortable, drop_class_attr(attrs), children)
}

/// styled `sort-trigger` パーツ（`button`）。
#[must_use]
pub fn sort_trigger<'a>(
    table: &DataTable,
    id: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    table.sort_trigger(id, drop_class_attr(attrs), children)
}

/// styled `select-all` パーツ（`th`）。
#[must_use]
pub fn select_all<'a>(
    state: crate::checkbox::CheckedState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    DataTable::select_all(state, drop_class_attr(attrs), children)
}

/// styled `select-row` パーツ（`td`）。
#[must_use]
pub fn select_row<'a>(
    state: crate::checkbox::CheckedState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    DataTable::select_row(state, drop_class_attr(attrs), children)
}

/// styled `footer` パーツ（純スロット）。
#[must_use]
pub fn footer<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::data_table::footer(drop_class_attr(attrs), children)
}

/// styled `selection-count` パーツ（純スロット）。
#[must_use]
pub fn selection_count<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::data_table::selection_count(drop_class_attr(attrs), children)
}

/// [`fandhe_frontend_headless_ui::menu::checkbox_item`] の薄いラッパ
/// （列表示切替 1 項目）。headless [`fandhe_frontend_headless_ui::data_table::column_toggle_item`]
/// をそのまま委譲する（`data-scope` は `menu` のまま、モジュール doc
/// headless 側参照）。
#[must_use]
pub fn column_toggle_item<'a>(
    column: &ColumnProps<'a>,
    disabled: bool,
    highlighted: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::data_table::column_toggle_item(
        column,
        disabled,
        highlighted,
        drop_class_attr(attrs),
        children,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="data-table"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains("<script"));
    }

    #[test]
    fn stylesheet_has_sort_direction_after_rules() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-sort="ascending"]::after"#));
        assert!(css.contains(r#"[data-sort="descending"]::after"#));
        assert!(css.contains(r#"[data-sort="none"]::after"#));
    }

    #[test]
    fn select_cells_do_not_reference_table_scope_vars() {
        let css = stylesheet();
        // select-all/select-row の宣言ブロックのみを雑に抽出せず、
        // モジュール全体が `--fandhe-table-` を一切含まないことを確認する
        // （`data-table` scope は独立した変数名前空間を持つ、モジュール doc
        // 「`select-all`/`select-row` が `--fandhe-table-*` を参照しない
        // 理由」節参照）。
        assert!(!css.contains("--fandhe-table-"));
    }

    #[test]
    fn root_drops_caller_class() {
        let html = render(&root(
            DataTableProps::default(),
            vec![("class", "evil")],
            vec![],
        ));
        assert!(!html.contains("class="));
    }
}

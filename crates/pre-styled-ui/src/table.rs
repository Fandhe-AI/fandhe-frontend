//! styled Table（イシュー #767）: slot recipe 静的部品。root/header/body/
//! footer/row/column-header/cell/caption の 8 パーツで
//! `table`/`thead`/`tbody`/`tfoot`/`tr`/`th`/`td`/`caption` の HTML 意味論を
//! そのまま尊重する（chakra-ui `data-display/table` 相当）。
//!
//! [`crate::card`]・[`crate::alert`] と同型の「状態機械を持たない静的
//! styled 部品」であり、`fandhe-frontend-headless-ui` 側に対応する anatomy は
//! 存在しない（[`crate::checkbox_card`]/[`crate::radio_card`] と同じく、
//! 本クレートで新規 anatomy `data-scope="table"` を定義する）。コンビニ関数
//! （全部入り `table(...)`）は提供せず、各パーツを個別に呼び出して組み立てる
//! 契約とする（[`crate::card`] と同じ判断、呼び出し例は各関数の rustdoc
//! `# Examples` を参照）。
//!
//! # variant（`variant`/`size`/`striped`）について
//!
//! [`crate::card`] と異なり 3 軸の variant を持つ（chakra-ui Table の
//! `variant`/`size`/`interactive`/`stickyHeader` のうち `interactive`/
//! `stickyHeader`・`showColumnBorder` はスコープ外、下記参照）:
//!
//! - [`TableVariant`]: `Line`（既定、行ごとの下線区切り）/ `Outline`
//!   （外枠 + 角丸）。
//! - `size`（[`crate::recipe::Size`]）: セルの padding・font-size を切り替える。
//! - `striped`（`bool`）: 縞模様表示。有効時は本文行の背景色を交互に変える。
//!
//! クラスは `root` パーツのみへ付与する（複合部品の variant 統一方針、
//! `crates/pre-styled-ui/src/lib.rs` §「複合部品の variant 統一方針」参照）。
//! `row`/`cell`/`column-header` への伝搬は `root` の variant 宣言が登録する
//! root スコープの CSS custom property（`--fandhe-table-cell-padding` 等）の
//! 通常の CSS 継承で行い、[`crate::recipe::SlotRecipe`] へ子孫セレクタ機構は
//! 追加しない（[`crate::switch`]/[`crate::breadcrumb`] と同型のパターン）。
//!
//! # striped の実装（イシュー #767・[`crate::recipe::StateCondition::NthChildEven`]）
//!
//! `striped` は常に `false`/`true` いずれかのクラスを `root` へ付与する
//! （決定性維持、[`crate::breadcrumb::BreadcrumbVariant`] と同じ「既定値も
//! 明示的に登録する」判断）。`true` 側は root スコープへ
//! `--fandhe-table-stripe-bg: var(--fandhe-color-bg-subtle)` を設定し、`false`
//! 側は `--fandhe-table-stripe-bg: transparent` を明示設定する。`row` slot の
//! [`crate::recipe::StateCondition::NthChildEven`] 規則が
//! `background: var(--fandhe-table-stripe-bg, transparent)` を消費する。
//!
//! `:nth-child(even)` は親要素（`thead`/`tbody`/`tfoot` それぞれ）内の兄弟を
//! 基準に数えるため、通常構成（1 行の `thead` + 複数行の `tbody`）では
//! `tbody` 内の行のみが交互に縞模様になる。`thead` が複数行の場合は 2 行目
//! 以降も対象になりうるが、`column-header`（`th`）は base 規則で背景色を
//! 明示するため視覚的な影響は小さい。
//!
//! # セキュリティ不変条件
//!
//! - セル値・列見出し・caption はすべて呼び出し側が渡す `children`
//!   （`fandhe_frontend_core::text()` 等）としてノード木経由で受け取り、HTML
//!   文字列の直接組み立ては行わない。出力は `render()` の既定エスケープを
//!   必ず経由する（`raw_html()` は使用しない）。
//! - variant クラス名は [`crate::recipe::SlotRecipe::variant_classes`] が
//!   `&'static str` enum 値から決定的に生成し、動的文字列合成を行わない。
//! - 呼び出し側 `attrs` に含まれる `class` は
//!   [`crate::class_attr::drop_class_attr`] で除去してから recipe 生成
//!   クラスと合成するため、`class` 属性は常に単一（呼び出し側からのクラス
//!   偽装・重複混入を防ぐ）。
//! - [`column_header`] の `scope="col"` は関数側で固定するため、呼び出し側
//!   `attrs` に `scope`（大文字小文字無視）が含まれていても除去する
//!   （[`checkbox_card`](crate::checkbox_card) の `drop_reserved` と同型の
//!   fail-closed 判断。重複 `scope` 属性による無効な HTML・意味論の後勝ち
//!   混乱を防ぐ）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - chakra-ui の `interactive`（クリック可能行のホバー装飾）・
//!   `stickyHeader`・`showColumnBorder`・`ScrollArea` 連携・`ColumnGroup`
//!   （`colgroup`/`col`）は本イシューのスコープ外（PR 本文に記録）。
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="table"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("table");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &[
    "root",
    "header",
    "body",
    "footer",
    "row",
    "column-header",
    "cell",
    "caption",
];

/// Table の見た目 variant（chakra-ui Table の `variant` を最小構成へ縮約）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableVariant {
    /// 行ごとの下線区切り（既定）。
    #[default]
    Line,
    /// 外枠 + 角丸。
    Outline,
}

impl VariantValue for TableVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Outline => "outline",
        }
    }
}

/// striped variant 値（内部専用、公開 API は `bool` のまま。
/// [`crate::table` モジュール doc](self)「striped の実装」節参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StripedVariant {
    /// 縞模様なし（既定）。
    Off,
    /// 縞模様あり。
    On,
}

impl VariantValue for StripedVariant {
    fn axis(self) -> &'static str {
        "striped"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Off => "false",
            Self::On => "true",
        }
    }
}

impl From<bool> for StripedVariant {
    fn from(b: bool) -> Self {
        if b {
            Self::On
        } else {
            Self::Off
        }
    }
}

/// Table の呼び出し側公開 props（`root` の引数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableProps {
    /// 見た目 variant（既定 [`TableVariant::Line`]）。
    pub variant: TableVariant,
    /// サイズ（既定 [`Size::Md`]）。
    pub size: Size,
    /// 縞模様表示の有無（既定 `false`）。
    pub striped: bool,
}

impl Default for TableProps {
    fn default() -> Self {
        Self {
            variant: TableVariant::default(),
            size: Size::Md,
            striped: false,
        }
    }
}

/// [`column_header`] が固定する属性名（呼び出し側 `attrs` からの偽装を
/// fail-closed で除去する対象）。
const COLUMN_HEADER_RESERVED: &[&str] = &["scope"];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（`crates/pre-styled-ui/src/checkbox_card.rs` の `drop_reserved`
/// と同型）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// Table の recipe（scope `"table"`、[`SLOTS`] の 8 パーツ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("table", SLOTS)
        .base(
            "root",
            vec![
                decl("width", "100%"),
                decl("border-collapse", "collapse"),
                decl("text-align", "left"),
            ],
        )
        .base(
            "caption",
            vec![
                decl("caption-side", "bottom"),
                decl("padding", "0.75rem 0"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .base(
            "column-header",
            vec![
                decl("padding", "var(--fandhe-table-cell-padding, 0.75rem 1rem)"),
                decl(
                    "font-size",
                    "var(--fandhe-table-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("text-align", "inherit"),
                decl(
                    "border-bottom",
                    "2px solid var(--fandhe-color-border-muted)",
                ),
                decl("background", "var(--fandhe-color-bg)"),
            ],
        )
        .base(
            "cell",
            vec![
                decl("padding", "var(--fandhe-table-cell-padding, 0.75rem 1rem)"),
                decl(
                    "font-size",
                    "var(--fandhe-table-font-size, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        .variant(
            TableVariant::Line,
            "root",
            vec![decl(
                "--fandhe-table-row-border",
                "1px solid var(--fandhe-color-border-muted)",
            )],
        )
        .variant(
            TableVariant::Outline,
            "root",
            vec![
                decl("--fandhe-table-row-border", "none"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
            ],
        )
        .default_variant(TableVariant::Line)
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-table-cell-padding", "0.5rem 0.75rem"),
                decl(
                    "--fandhe-table-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-table-cell-padding", "0.75rem 1rem"),
                decl(
                    "--fandhe-table-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-table-cell-padding", "1rem 1.25rem"),
                decl(
                    "--fandhe-table-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .default_variant(Size::Md)
        .variant(
            StripedVariant::Off,
            "root",
            vec![decl("--fandhe-table-stripe-bg", "transparent")],
        )
        .variant(
            StripedVariant::On,
            "root",
            vec![decl(
                "--fandhe-table-stripe-bg",
                "var(--fandhe-color-bg-subtle)",
            )],
        )
        .default_variant(StripedVariant::Off)
        .base(
            "row",
            vec![decl(
                "border-bottom",
                "var(--fandhe-table-row-border, none)",
            )],
        )
        .state(
            "row",
            StateCondition::NthChildEven,
            vec![decl(
                "background",
                "var(--fandhe-table-stripe-bg, transparent)",
            )],
        )
}

/// Table の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツ（`<table>`）を組み立てる。`variant`/`size`/`striped` に応じた
/// クラスを付与する唯一のパーツ（[`drop_class_attr`] により呼び出し側の
/// `class` は除去してから合成する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
///
/// let node = table::root(TableProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="table" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(props: TableProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let striped: StripedVariant = props.striped.into();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("size", props.size.value()),
        ("striped", striped.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "table", merged, children)
}

/// header パーツ（`<thead>`）を組み立てる。variant を持たないため `class` は
/// 付与せず、呼び出し側 `attrs` をそのまま連結する。
#[must_use]
pub fn header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("header", "thead", attrs, children)
}

/// body パーツ（`<tbody>`）を組み立てる。
#[must_use]
pub fn body<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("body", "tbody", attrs, children)
}

/// footer パーツ（`<tfoot>`）を組み立てる。
#[must_use]
pub fn footer<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("footer", "tfoot", attrs, children)
}

/// row パーツ（`<tr>`）を組み立てる。
#[must_use]
pub fn row<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("row", "tr", attrs, children)
}

/// column-header パーツ（`<th scope="col">`）を組み立てる。列見出しの
/// WAI-ARIA/HTML 意味論（`scope="col"`）を既定で担保する。呼び出し側 `attrs`
/// に `scope` を含めても [`drop_reserved`] により除去される（本モジュール
/// doc「セキュリティ不変条件」節参照）。
#[must_use]
pub fn column_header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![("scope", "col")];
    merged.extend(drop_reserved(attrs, COLUMN_HEADER_RESERVED));
    ANATOMY.part("column-header", "th", merged, children)
}

/// cell パーツ（`<td>`）を組み立てる。
#[must_use]
pub fn cell<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("cell", "td", attrs, children)
}

/// caption パーツ（`<caption>`）を組み立てる。呼び出し側は `<table>` の
/// 直接の子として `root` の `children` 先頭に置く必要がある（HTML 仕様上
/// `caption` は `table` の最初の子でなければならない。本関数自体は順序を
/// 強制しない）。
#[must_use]
pub fn caption<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("caption", "caption", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_variant_is_line_md_not_striped() {
        let html = render(&root(TableProps::default(), vec![], vec![]));
        assert!(html.contains("fd-table--variant-line"));
        assert!(html.contains("fd-table--size-md"));
        assert!(html.contains("fd-table--striped-false"));
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (TableVariant::Line, "fd-table--variant-line"),
            (TableVariant::Outline, "fd-table--variant-outline"),
        ] {
            let props = TableProps {
                variant,
                ..TableProps::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(class), "variant={variant:?} -> {html}");
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-table--size-sm"),
            (Size::Md, "fd-table--size-md"),
            (Size::Lg, "fd-table--size-lg"),
        ] {
            let props = TableProps {
                size,
                ..TableProps::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn striped_true_maps_to_expected_class() {
        let props = TableProps {
            striped: true,
            ..TableProps::default()
        };
        let html = render(&root(props, vec![], vec![]));
        assert!(html.contains("fd-table--striped-true"));
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&header(vec![], vec![]))
            .starts_with(r#"<thead data-scope="table" data-part="header""#));
        assert!(render(&body(vec![], vec![]))
            .starts_with(r#"<tbody data-scope="table" data-part="body""#));
        assert!(render(&footer(vec![], vec![]))
            .starts_with(r#"<tfoot data-scope="table" data-part="footer""#));
        assert!(
            render(&row(vec![], vec![])).starts_with(r#"<tr data-scope="table" data-part="row""#)
        );
        assert!(
            render(&cell(vec![], vec![])).starts_with(r#"<td data-scope="table" data-part="cell""#)
        );
        assert!(render(&caption(vec![], vec![]))
            .starts_with(r#"<caption data-scope="table" data-part="caption""#));
    }

    #[test]
    fn column_header_fixes_scope_col_and_drops_caller_scope() {
        let html = render(&column_header(vec![("scope", "row")], vec![]));
        assert!(html.starts_with(r#"<th data-scope="table" data-part="column-header""#));
        assert!(html.contains(r#"scope="col""#));
        assert!(!html.contains(r#"scope="row""#));
        assert_eq!(html.matches(r#" scope=""#).count(), 1);
    }

    #[test]
    fn composed_table_snapshot() {
        let node = root(
            TableProps::default(),
            vec![],
            vec![
                caption(vec![], vec![text("Users")]),
                header(
                    vec![],
                    vec![row(vec![], vec![column_header(vec![], vec![text("Name")])])],
                ),
                body(
                    vec![],
                    vec![row(vec![], vec![cell(vec![], vec![text("Alice")])])],
                ),
            ],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<table data-scope="table" data-part="root" class="fd-table--variant-line fd-table--size-md fd-table--striped-false">"#,
                r#"<caption data-scope="table" data-part="caption">Users</caption>"#,
                r#"<thead data-scope="table" data-part="header">"#,
                r#"<tr data-scope="table" data-part="row">"#,
                r#"<th data-scope="table" data-part="column-header" scope="col">Name</th>"#,
                r#"</tr>"#,
                r#"</thead>"#,
                r#"<tbody data-scope="table" data-part="body">"#,
                r#"<tr data-scope="table" data-part="row">"#,
                r#"<td data-scope="table" data-part="cell">Alice</td>"#,
                r#"</tr>"#,
                r#"</tbody>"#,
                r#"</table>"#,
            )
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            TableProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_cell_and_column_header_children_is_escaped() {
        let cell_html = render(&cell(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!cell_html.contains("<script>"));
        assert!(cell_html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));

        let header_html = render(&column_header(
            vec![],
            vec![text("<img src=x onerror=alert(1)>")],
        ));
        assert!(!header_html.contains("<img"));
        assert!(header_html.contains("&lt;img"));

        let caption_html = render(&caption(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!caption_html.contains("<script>"));
    }

    #[test]
    fn css_output_declares_striped_and_size_tokens() {
        let out = css();
        assert!(out.contains(":nth-child(even)"));
        assert!(out.contains("--fandhe-table-stripe-bg"));
        assert!(out.contains("--fandhe-table-cell-padding"));
        assert!(!out.contains('<'));
    }
}

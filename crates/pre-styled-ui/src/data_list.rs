//! styled DataList（イシュー #767）: slot recipe 静的部品。root/item/
//! item-label/item-value の 4 パーツで `dl`/`dt`/`dd` の定義リスト意味論を
//! そのまま尊重する（chakra-ui `data-display/data-list` 相当）。
//!
//! [`crate::card`]・[`crate::table`] と同型の「状態機械を持たない静的
//! styled 部品」であり、`fandhe-frontend-headless-ui` 側に対応する anatomy は
//! 存在しない（本クレートで新規 anatomy `data-scope="data-list"` を定義する）。
//! コンビニ関数（全部入り `data_list(...)`）は提供せず、各パーツを個別に
//! 呼び出して組み立てる契約とする（[`crate::card`] と同じ判断、呼び出し例は
//! 各関数の rustdoc `# Examples` を参照）。
//!
//! # `item` に `<div>` を使う理由
//!
//! `dl` の直接の子として許容される要素は `dt`/`dd`（グルーピング用
//! `<div>` も HTML Standard で明示的に許容されている）のみである。1 組の
//! ラベル・値をまとめる `item` パーツには `<div>` を使い、内部に
//! `item_label`（`<dt>`）と `item_value`（`<dd>`）を子として置く（`li` 等
//! `dl` に不正な要素は使わない）。
//!
//! # variant（`orientation`）について
//!
//! [`DataListOrientation`] のみを持つ 1 軸 variant（chakra-ui の
//! `variant`（`subtle`/`bold`）・`size` はスコープ外、下記参照）:
//!
//! - `Vertical`（既定）: ラベルの下に値を縦積み表示。
//! - `Horizontal`: `item` を `display: flex` にしてラベル・値を横並び表示。
//!
//! クラスは `root` パーツのみへ付与する（複合部品の variant 統一方針、
//! `crates/pre-styled-ui/src/lib.rs` §「複合部品の variant 統一方針」参照）。
//! `item` への伝搬は `root` の variant 宣言が登録する root スコープの CSS
//! custom property（`--fandhe-data-list-item-display` 等）の通常の CSS
//! 継承で行い、[`crate::recipe::SlotRecipe`] へ子孫セレクタ機構は追加しない
//! （[`crate::table`]/[`crate::switch`] と同型のパターン）。
//!
//! # セキュリティ不変条件
//!
//! - ラベル・値はすべて呼び出し側が渡す `children`
//!   （`fandhe_frontend_core::text()` 等）としてノード木経由で受け取り、HTML
//!   文字列の直接組み立ては行わない。出力は `render()` の既定エスケープを
//!   必ず経由する（`raw_html()` は使用しない）。
//! - variant クラス名は [`crate::recipe::SlotRecipe::variant_classes`] が
//!   `&'static str` enum 値から決定的に生成し、動的文字列合成を行わない。
//! - 呼び出し側 `attrs` に含まれる `class` は
//!   [`crate::class_attr::drop_class_attr`] で除去してから recipe 生成
//!   クラスと合成するため、`class` 属性は常に単一（呼び出し側からのクラス
//!   偽装・重複混入を防ぐ）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - chakra-ui の `variant`（`subtle`/`bold`）・`size` variant は本イシューの
//!   スコープ外（PR 本文に記録）。
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="data-list"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("data-list");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &["root", "item", "item-label", "item-value"];

/// DataList の並び方向 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataListOrientation {
    /// ラベルの下に値を縦積み表示（既定）。
    #[default]
    Vertical,
    /// ラベル・値を横並び表示。
    Horizontal,
}

impl VariantValue for DataListOrientation {
    fn axis(self) -> &'static str {
        "orientation"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }
}

/// DataList の呼び出し側公開 props（`root` の引数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DataListProps {
    /// 並び方向（既定 [`DataListOrientation::Vertical`]）。
    pub orientation: DataListOrientation,
}

/// DataList の recipe（scope `"data-list"`、[`SLOTS`] の 4 パーツ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("data-list", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "0.75rem"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "var(--fandhe-data-list-item-display, flex)"),
                decl(
                    "flex-direction",
                    "var(--fandhe-data-list-item-flex-direction, column)",
                ),
                decl("gap", "var(--fandhe-data-list-item-gap, 0.125rem)"),
            ],
        )
        .base(
            "item-label",
            vec![
                decl("margin", "0"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .base(
            "item-value",
            vec![
                decl("margin", "0"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-size", "var(--fandhe-font-font-size-md)"),
            ],
        )
        .variant(
            DataListOrientation::Vertical,
            "root",
            vec![
                decl("--fandhe-data-list-item-display", "flex"),
                decl("--fandhe-data-list-item-flex-direction", "column"),
                decl("--fandhe-data-list-item-gap", "0.125rem"),
            ],
        )
        .variant(
            DataListOrientation::Horizontal,
            "root",
            vec![
                decl("--fandhe-data-list-item-display", "flex"),
                decl("--fandhe-data-list-item-flex-direction", "row"),
                decl("--fandhe-data-list-item-gap", "0.5rem"),
            ],
        )
        .default_variant(DataListOrientation::Vertical)
}

/// DataList の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツ（`<dl>`）を組み立てる。`orientation` に応じたクラスを付与する
/// 唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去して
/// から合成する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::data_list::{self, DataListProps};
///
/// let node = data_list::root(DataListProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="data-list" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(props: DataListProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("orientation", props.orientation.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "dl", merged, children)
}

/// item パーツ（`<div>`）を組み立てる。variant を持たないため `class` は
/// 付与せず、呼び出し側 `attrs` をそのまま連結する（本モジュール doc
/// 「`item` に `<div>` を使う理由」参照）。
#[must_use]
pub fn item<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item", "div", attrs, children)
}

/// item-label パーツ（`<dt>`）を組み立てる。
#[must_use]
pub fn item_label<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-label", "dt", attrs, children)
}

/// item-value パーツ（`<dd>`）を組み立てる。
#[must_use]
pub fn item_value<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-value", "dd", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_orientation_is_vertical() {
        let html = render(&root(DataListProps::default(), vec![], vec![]));
        assert!(html.contains("fd-data-list--orientation-vertical"));
    }

    #[test]
    fn orientation_enumeration_maps_to_expected_classes() {
        for (orientation, class) in [
            (
                DataListOrientation::Vertical,
                "fd-data-list--orientation-vertical",
            ),
            (
                DataListOrientation::Horizontal,
                "fd-data-list--orientation-horizontal",
            ),
        ] {
            let html = render(&root(DataListProps { orientation }, vec![], vec![]));
            assert!(
                html.contains(class),
                "orientation={orientation:?} -> {html}"
            );
        }
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&item(vec![], vec![]))
            .starts_with(r#"<div data-scope="data-list" data-part="item""#));
        assert!(render(&item_label(vec![], vec![]))
            .starts_with(r#"<dt data-scope="data-list" data-part="item-label""#));
        assert!(render(&item_value(vec![], vec![]))
            .starts_with(r#"<dd data-scope="data-list" data-part="item-value""#));
    }

    #[test]
    fn composed_data_list_snapshot() {
        let node = root(
            DataListProps::default(),
            vec![],
            vec![item(
                vec![],
                vec![
                    item_label(vec![], vec![text("Name")]),
                    item_value(vec![], vec![text("Alice")]),
                ],
            )],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<dl data-scope="data-list" data-part="root" class="fd-data-list--orientation-vertical">"#,
                r#"<div data-scope="data-list" data-part="item">"#,
                r#"<dt data-scope="data-list" data-part="item-label">Name</dt>"#,
                r#"<dd data-scope="data-list" data-part="item-value">Alice</dd>"#,
                r#"</div>"#,
                r#"</dl>"#,
            )
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            DataListProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_item_label_and_value_children_is_escaped() {
        let label_html = render(&item_label(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!label_html.contains("<script>"));
        assert!(label_html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));

        let value_html = render(&item_value(
            vec![],
            vec![text("<img src=x onerror=alert(1)>")],
        ));
        assert!(!value_html.contains("<img"));
        assert!(value_html.contains("&lt;img"));
    }

    #[test]
    fn css_output_declares_orientation_tokens() {
        let out = css();
        assert!(out.contains("--fandhe-data-list-item-display"));
        assert!(out.contains("--fandhe-data-list-item-flex-direction"));
        assert!(!out.contains('<'));
    }
}

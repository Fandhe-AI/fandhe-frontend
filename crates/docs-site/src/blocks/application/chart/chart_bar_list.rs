//! `chart-bar-list` block（イシュー #2903。Application/Chart カテゴリ）。
//! 主参照 R0052（項目名 + 値の横棒ランキング 2 枚並び、
//! 値は右端揃え）を集約する。参照元は R0052 の 1 件のみで、集約元との
//! 差分を並べる対象はない（`card-form-footer` の「2 例を並べる理由」に
//! 相当する記述は本 block には存在しない）。
//!
//! # 使用部品
//!
//! `card` / `heading` / `charts`（共通 API・`ChartData`/`SortDirection`）/
//! `bar-list` の 4 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//! 新しい UI 部品は追加しない。
//!
//! # ランキング順は呼び出し側の責務
//!
//! [`fandhe_frontend_pre_styled_ui::charts::bar_list::root`] は並び順を
//! 変更しない契約を持つ（同モジュール rustdoc「本イシューのスコープ外」
//! 節）。降順ランキングにするため、本 block は [`ChartData::sort_by_series`]
//! （[`SortDirection::Descending`]）を呼び出し側で明示的に適用してから
//! `bar_list::root` へ渡す。デモデータはあえて未ソートの順で与え、ソート
//! 適用後の並び替えが実際に効いていることをテストで固定する。
//!
//! # `card::title` ではなく `heading` を使う理由
//!
//! 使用部品として `heading` を実際に合成するため、`card::header` の中身は
//! `card::title`（内部で見出しタグを持たない）ではなく
//! `heading(HeadingLevel::H3, ...)` を直接置く。見出しの階層はページ本体
//! の `h1` → Demo の `h2`（`component_page`/`blocks` 共通レイアウト）→
//! 本 block のカード見出し `h3` の順になる。
//!
//! # 狭幅での 1 列化
//!
//! [`LAYOUT_CSS`] の `.blocks-chart-bar-list-layout` は
//! `grid-template-columns: repeat(auto-fit, minmax(min(100%, 20rem), 1fr))`
//! のみで広幅 2 列・狭幅（コンテナ幅 20rem 未満相当）1 列を切り替える
//! （`card-form-footer` と同じ media query 不要の方式）。値の右寄せは
//! `bar_list` の 2 列グリッド（`minmax(0, 1fr) auto`）契約で満たされるため
//! 追加の CSS は書かない。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R0052。文言・配色・アイコンは独自に書く（他 block と
//! 同じライセンス上の転記制限）。デモデータ（流入元・よく読まれたページ）
//! は架空のものであり、実在の企業名・PII は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::charts::bar_list;
use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series, SortDirection};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};

/// ランキングカード 1 枚を組み立てる小さな helper（内部専用）。`rows` は
/// `(ラベル, 値)` の組で、表示順は未ソートのままでよい（本関数の内部で
/// 降順ソートを適用する）。
fn ranking_card(title: &str, description: &str, series_name: &str, rows: &[(&str, f64)]) -> Node {
    let categories = rows.iter().map(|(label, _)| (*label).to_string()).collect();
    let values = rows.iter().map(|(_, value)| *value).collect();
    let data = ChartData::new(categories, vec![Series::new(series_name, values)])
        .expect("chart-bar-list の固定データは常に有効な ChartData を構成する")
        .sort_by_series(series_name, SortDirection::Descending)
        .expect("series_name は直前に構築した ChartData 自身の系列名と一致する");
    let list = bar_list::root(&data, series_name)
        .expect("chart-bar-list の固定データに未知系列・負値は含まれない");

    card::root(
        CardProps::from(CardVariant::Outline),
        vec![],
        vec![
            card::header(
                vec![],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![fandhe_frontend_core::text(title)],
                    ),
                    card::description(vec![], vec![fandhe_frontend_core::text(description)]),
                ],
            ),
            card::body(vec![], vec![list]),
        ],
    )
}

/// `chart-bar-list` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let inflow = ranking_card(
        "流入元（今月）",
        "訪問数（件）",
        "visits",
        &[
            ("直接アクセス", 1180.0),
            ("検索", 3420.0),
            ("紹介リンク", 640.0),
            ("SNS", 2260.0),
            ("ニュースレター", 980.0),
        ],
    );
    let popular_pages = ranking_card(
        "よく読まれたページ",
        "閲覧数（件）",
        "views",
        &[
            ("更新履歴", 510.0),
            ("はじめに", 4380.0),
            ("お問い合わせ", 260.0),
            ("料金", 1970.0),
            ("導入事例", 1340.0),
        ],
    );
    div(
        vec![("class", "blocks-chart-bar-list-layout")],
        vec![inflow, popular_pages],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/chart-bar-list/",
    title: "chart-bar-list",
    category: BlockCategory::Chart,
    rust_source: "crates/docs-site/src/blocks/application/chart/chart_bar_list.rs",
    demo_class: "blocks-chart-bar-list",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Charts",
            path: "/themes/charts/",
        },
        Part {
            label: "Bar List",
            path: "/themes/bar-list/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `chart_bar_list` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-chart-bar-list-layout {\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(min(100%, 20rem), 1fr));\n  gap: var(--fandhe-space-6);\n  align-items: start;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"heading\"",
            "data-scope=\"bar-list\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(
            html.matches("data-scope=\"card\" data-part=\"root\"")
                .count(),
            2
        );
        assert_eq!(html.matches("<h3").count(), 2);
        assert!(html.contains("class=\"blocks-chart-bar-list-layout\""));
    }

    #[test]
    fn no_form_or_unsafe_markup() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn descending_sort_places_max_value_first_in_each_card() {
        let html = demo_html();
        // 未ソートの入力データに対し、各カードの最初の行（bar-list の
        // 最初の item）が最大値の項目になっていること（降順ソートの効果、
        // モジュール doc「ランキング順は呼び出し側の責務」節）。
        let inflow_first_item = html
            .split("data-scope=\"bar-list\" data-part=\"item\"")
            .nth(1)
            .expect("1 件目の bar-list item が存在する");
        assert!(inflow_first_item.contains("検索"));
        assert!(inflow_first_item.contains("--fandhe-bar-list-percent: 100%"));

        let popular_first_item = html
            .split("data-scope=\"bar-list\" data-part=\"item\"")
            .nth(6)
            .expect("2 件目の bar-list（5 item 分オフセット後）の 1 件目が存在する");
        assert!(popular_first_item.contains("はじめに"));
        assert!(popular_first_item.contains("--fandhe-bar-list-percent: 100%"));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains(".blocks-chart-bar-list-layout"));
        assert!(LAYOUT_CSS.contains("auto-fit"));
    }
}

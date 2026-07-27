//! styled Calendar（headless ラッパー、イシュー #835、親 #520/#546）。
//!
//! `fandhe_frontend_headless_ui::calendar` の Root / Heading / PrevTrigger /
//! NextTrigger / Table / TableHeader / TableRow / TableHeadCell / TableBody /
//! TableCell / DayTrigger 11 anatomy パーツを再エクスポートし、[`stylesheet`]
//! で既定 CSS を追加提供する。薄い委譲の根拠・スコープ外事項は
//! [`crate::select`]（本クレート内の先行例）の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`Calendar` 状態機械を再エクスポートしない理由）
//!
//! `size` variant クラス付与のため styled [`root`] を本モジュールで新設する。
//! 状態機械 [`fandhe_frontend_headless_ui::calendar::Calendar`] は**あえて**
//! 再エクスポートしない（[`crate::select`] と同じ理由）。状態管理・hydration
//! が必要な呼び出し側は `fandhe_frontend_headless_ui::calendar::Calendar` を
//! 直接 import し、実際の描画は本モジュールの styled パーツ関数を組み合わせて
//! 構築すること。
//!
//! # data-state とスタイルの連動
//!
//! `day-trigger` の `data-selected`/`data-today`/`data-outside-month`/
//! `data-disabled` に応じた見た目の切り替えを [`recipe`] へ登録する
//! （[`crate::recipe::SlotRecipe::state`]）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};

// headless 自由関数 `root`・状態機械 `Calendar` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。
pub use fandhe_frontend_headless_ui::calendar::{
    day_trigger, heading, next_trigger, prev_trigger, table, table_body, table_cell,
    table_head_cell, table_header, table_row,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
// `day_trigger` の `date` 引数型を呼び出し側（`fandhe-frontend-docs-site` 等、
// headless-ui へ直接依存しない下流クレート）がヘッドレス層への直接依存
// なしに構築できるよう、暦計算コア（イシュー #833）の値型も再エクスポート
// する（[`crate::select`] が `OpenState` を再エクスポートするのと同じ理由、
// イシュー #685）。
pub use fandhe_frontend_headless_ui::date::{PlainDate, Weekday};

/// headless `calendar` anatomy の `data-part` 一覧（`crates/headless-ui/src/calendar.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "heading",
    "prev-trigger",
    "next-trigger",
    "table",
    "table-header",
    "table-row",
    "table-head-cell",
    "table-body",
    "table-cell",
    "day-trigger",
];

/// この styled Calendar の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("calendar", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl(
                    "padding",
                    "var(--fandhe-calendar-root-padding, var(--fandhe-space-3))",
                ),
            ],
        )
        .base(
            "heading",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "space-between"),
                decl("font-weight", "600"),
            ],
        )
        .base(
            "prev-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border-radius", "0.25rem"),
            ],
        )
        .base(
            "next-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border-radius", "0.25rem"),
            ],
        )
        .base(
            "table",
            vec![decl("border-collapse", "collapse"), decl("width", "100%")],
        )
        .base(
            "table-head-cell",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("font-weight", "500"),
                decl("padding", "var(--fandhe-space-1)"),
                decl("text-align", "center"),
            ],
        )
        .base(
            "table-cell",
            vec![decl("padding", "1px"), decl("text-align", "center")],
        )
        .base(
            "day-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border-radius", "0.25rem"),
                decl(
                    "width",
                    "var(--fandhe-calendar-day-size, var(--fandhe-space-8))",
                ),
                decl(
                    "height",
                    "var(--fandhe-calendar-day-size, var(--fandhe-space-8))",
                ),
            ],
        )
        // 選択日・今日・表示月外・disabled の見た目切り替え。
        // `data-selected`/`data-today`/`data-outside-month` の出力元は
        // headless-ui（`crates/headless-ui/src/calendar.rs` の day-trigger
        // パーツ）。本モジュールは CSS セレクタとして参照するのみで、属性を
        // 出力しない（イシュー #1063、
        // `docs/design/pre-styled-ui-data-attr-vocabulary.md` 規約 A）。
        .state(
            "day-trigger",
            StateCondition::Attr("data-selected"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        .state(
            "day-trigger",
            StateCondition::Attr("data-today"),
            vec![decl("font-weight", "700")],
        )
        .state(
            "day-trigger",
            StateCondition::Attr("data-outside-month"),
            vec![decl("color", "var(--fandhe-color-fg-muted)")],
        )
        .state(
            "day-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.4")],
        )
        .state(
            "day-trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .state(
            "prev-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.4")],
        )
        .state(
            "next-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.4")],
        )
        // `size` variant（root スコープの CSS custom property）。
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-calendar-root-padding", "var(--fandhe-space-2)"),
                decl("--fandhe-calendar-day-size", "var(--fandhe-space-6)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-calendar-root-padding", "var(--fandhe-space-3)"),
                decl("--fandhe-calendar-day-size", "var(--fandhe-space-8)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-calendar-root-padding", "var(--fandhe-space-4)"),
                decl("--fandhe-calendar-day-size", "var(--fandhe-space-10)"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled Calendar が生成する静的 CSS 全量を返す（決定的。
/// [`crate::select::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::calendar::root`] へ委譲する。
#[must_use]
pub fn root<'a>(size: Size, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::calendar::root(merged, children)
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
        assert!(a.contains(r#"[data-scope="calendar"][data-part="day-trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Size::Md, vec![], vec![]));
        assert!(html.contains(r#"data-scope="calendar""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let html = render(&root(size, vec![("class", "attacker")], vec![]));
            let expected_class = format!("fd-calendar--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn stylesheet_links_data_attrs_to_style() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="calendar"][data-part="day-trigger"][data-selected]"#));
        assert!(css.contains(r#"[data-scope="calendar"][data-part="day-trigger"][data-today]"#));
        assert!(
            css.contains(r#"[data-scope="calendar"][data-part="day-trigger"][data-outside-month]"#)
        );
        assert!(css.contains(r#"[data-scope="calendar"][data-part="day-trigger"][data-disabled]"#));
    }
}

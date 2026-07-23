//! styled Carousel（headless ラッパー、イシュー #754、親 #748/#520）。
//!
//! `fandhe_frontend_headless_ui::carousel`（イシュー #754）の Root /
//! Control / PrevTrigger / NextTrigger / ItemGroup / Item / IndicatorGroup /
//! Indicator 8 anatomy パーツを再エクスポートし、[`stylesheet`] で既定 CSS を
//! 追加提供する。薄い委譲の根拠・スコープ外事項は [`crate::slider`]/
//! [`crate::segment_group`] の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Carousel` 型・
//! headless `root` を再エクスポートしない理由）
//!
//! [`crate::slider`]/[`crate::select`] と同じ理由（`size` variant クラス
//! 付与のため styled [`root`] を本モジュールで新設し、headless 自由関数
//! `root` と名前が衝突するため）で、必要な識別子のみを選択的に再エクスポート
//! する。状態機械 [`fandhe_frontend_headless_ui::carousel::Carousel`] は
//! **あえて**再エクスポートしない（[`crate::slider`]/[`crate::select`]/
//! [`crate::switch`] の状態機械非再エクスポートと同じ理由）。`Carousel` に
//! よる状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::carousel::Carousel` を直接 import し、実際の
//! 描画は本モジュールの styled [`root`]（および再エクスポート済みのパーツ
//! 関数）を組み合わせて構築すること。
//!
//! # transform ベースのスライド位置表現（`--fandhe-carousel-index`）
//!
//! headless `Carousel::item_group`（`crates/headless-ui/src/carousel.rs`）が
//! `style="--fandhe-carousel-index: <index>;"` を出力する契約に対応し、
//! `item-group` slot の recipe が
//! `transform: translateX(calc(var(--fandhe-carousel-index, 0) * -100%))`
//! （`data-orientation="vertical"` のときは `translateY`）を宣言する。
//! [`crate::recipe::SlotRecipe`] は子孫セレクタを持たないため、縦横の切替は
//! `item-group` 自身の `[data-orientation]` 属性条件で行う
//! （[`crate::segment_group`] の indicator が `--fandhe-segment-group-index`/
//! `-count` を同じ理由で `data-orientation` 条件化しているのと同型）。
//! `var()` には明示フォールバック値 `0` を書き（headless 直接利用・
//! hydrate 前の静的マークアップでも `translateX(0)` として描画される
//! fail-safe、複合部品の variant 統一方針 §2 と同じ判断）、CSS カスタム
//! プロパティ経由のみで決定的にスライド位置が定まる（JS 計測に依存しない）。
//!
//! # data-current とスタイルの連動
//!
//! `item`（現在表示中のスライド）・`indicator`（現在位置を示すドット）の
//! `data-current` 存在属性に応じた見た目の切り替えを [`recipe`] へ登録する
//! （[`crate::recipe::StateCondition::Attr`]）。
//!
//! # 複合部品の variant 統一方針（イシュー #708）適用
//!
//! `size`（Sm/Md/Lg、indicator の寸法・trigger のパディング）のみを提供し、
//! `color-palette` 軸は提供しない（carousel は選択・チェック状態を示す
//! 部品ではなく、コンテンツ送り UI であるため。方針 §3 参照）。クラスは
//! root slot のみに付与し、子孫 slot への伝搬は root スコープの CSS
//! カスタムプロパティ（`--fandhe-carousel-*`）の通常の CSS 継承で行う
//! （[`crate::slider`]/[`crate::segment_group`] と同型）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};

// headless 自由関数 `root`・状態機械 `Carousel` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。未スタイル・
// variant クラス非付与の実体・状態管理が必要な呼び出し側は
// `fandhe_frontend_headless_ui::carousel` を直接 import する。
pub use fandhe_frontend_headless_ui::carousel::{
    control, indicator, indicator_group, item, item_group, next_trigger, prev_trigger,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
// `root` の `orientation` 引数はいずれも `data_attrs` 由来で上記選択的
// 再エクスポートでは到達しない。呼び出し側が `fandhe-frontend-pre-styled-ui`
// のみに依存して呼び出せることを保証するための明示再エクスポート
// （イシュー #685 の契約、[`crate::slider`]/[`crate::segment_group`] と同型）。
pub use fandhe_frontend_headless_ui::Orientation;

/// headless `carousel` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/carousel.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "control",
    "prev-trigger",
    "next-trigger",
    "item-group",
    "item",
    "indicator-group",
    "indicator",
];

/// この styled Carousel の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("carousel", SLOTS)
        .base(
            "root",
            vec![decl("position", "relative"), decl("overflow", "hidden")],
        )
        .base(
            "control",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "prev-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "9999px"),
                decl("cursor", "pointer"),
                decl("width", "var(--fandhe-carousel-trigger-size, 2.5rem)"),
                decl("height", "var(--fandhe-carousel-trigger-size, 2.5rem)"),
            ],
        )
        .base(
            "next-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "9999px"),
                decl("cursor", "pointer"),
                decl("width", "var(--fandhe-carousel-trigger-size, 2.5rem)"),
                decl("height", "var(--fandhe-carousel-trigger-size, 2.5rem)"),
            ],
        )
        .base(
            "item-group",
            vec![
                decl("display", "flex"),
                decl("flex", "1"),
                decl(
                    "transition",
                    "transform var(--fandhe-carousel-transition-duration, 0.2s) ease",
                ),
                decl(
                    "transform",
                    "translateX(calc(var(--fandhe-carousel-index, 0) * -100%))",
                ),
            ],
        )
        .base("item", vec![decl("flex", "0 0 100%")])
        .base(
            "indicator-group",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "indicator",
            vec![
                decl("display", "inline-block"),
                decl("background", "var(--fandhe-color-bg-muted)"),
                decl("border", "none"),
                decl("border-radius", "9999px"),
                decl("cursor", "pointer"),
                decl("width", "var(--fandhe-carousel-indicator-size, 0.5rem)"),
                decl("height", "var(--fandhe-carousel-indicator-size, 0.5rem)"),
            ],
        )
        // Carousel 固有: `item-group` の縦方向スライド（[`crate::segment_group`]
        // の indicator が `data-orientation` で translateX/Y を切り替えるのと
        // 同型の判断、モジュール rustdoc「transform ベースのスライド位置表現」
        // 節参照）。
        .state(
            "item-group",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl(
                "transform",
                "translateY(calc(var(--fandhe-carousel-index, 0) * -100%))",
            )],
        )
        // 端に到達し `loop` 無効なため無効化された trigger の見た目（headless
        // `data-disabled` 存在属性、[`crate::slider`] 等と同型の減光表現）。
        .state(
            "prev-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.4"), decl("cursor", "not-allowed")],
        )
        .state(
            "next-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.4"), decl("cursor", "not-allowed")],
        )
        .state(
            "prev-trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .state(
            "next-trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        // 現在の indicator を強調する（headless `data-current` 存在属性、
        // モジュール rustdoc「data-current とスタイルの連動」節参照）。
        .state(
            "indicator",
            StateCondition::Attr("data-current"),
            vec![decl("background", "var(--fandhe-color-accent)")],
        )
        .state(
            "indicator",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        // `size` variant（root スコープの CSS custom property。Md はフォール
        // バック値と同一の現行外観を維持する）。`--fandhe-carousel-index`
        // （wasm 層/headless の位置契約）には手を触れない（モジュール
        // rustdoc 参照）。
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-carousel-trigger-size", "2rem"),
                decl("--fandhe-carousel-indicator-size", "0.375rem"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-carousel-trigger-size", "2.5rem"),
                decl("--fandhe-carousel-indicator-size", "0.5rem"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-carousel-trigger-size", "3rem"),
                decl("--fandhe-carousel-indicator-size", "0.625rem"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled Carousel が生成する静的 CSS 全量を返す（決定的。
/// [`crate::slider::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::carousel::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::carousel::{self, Orientation};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = carousel::root(Size::Md, Orientation::Horizontal, "Products", vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="carousel" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    orientation: Orientation,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::carousel::root(orientation, label, merged, children)
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
        assert!(a.contains(r#"[data-scope="carousel"][data-part="item-group"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(
            Size::Md,
            Orientation::Horizontal,
            "Products",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="carousel""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"aria-label="Products""#));
    }

    // --- size variant ---

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let html = render(&root(
                size,
                Orientation::Horizontal,
                "Products",
                vec![("class", "attacker")],
                vec![],
            ));
            let expected_class = format!("fd-carousel--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn default_variant_is_md() {
        let css = stylesheet();
        assert!(css.contains("--fandhe-carousel-trigger-size: 2.5rem;"));
        assert!(css.contains("--fandhe-carousel-indicator-size: 0.5rem;"));
    }

    // --- item-group transform ---

    #[test]
    fn item_group_transform_consumes_fandhe_carousel_index_css_var() {
        let css = stylesheet();
        assert!(
            css.contains("transform: translateX(calc(var(--fandhe-carousel-index, 0) * -100%));")
        );
    }

    #[test]
    fn item_group_switches_to_translate_y_when_vertical() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"carousel\"][data-part=\"item-group\"][data-orientation=\"vertical\"] {\n  \
             transform: translateY(calc(var(--fandhe-carousel-index, 0) * -100%));\n\
             }\n"
        ));
    }

    #[test]
    fn position_geometry_var_references_never_lack_an_explicit_fallback() {
        // fail-closed 回帰（[`crate::combobox`] と同型）: `--fandhe-carousel-index`
        // への参照はすべて明示フォールバック値を持つ（裸の `var(--x)` 禁止）。
        let css = stylesheet();
        for (idx, _) in css.match_indices("var(--fandhe-carousel-index") {
            let close = css[idx..]
                .find(')')
                .expect("every var( occurrence must be closed within the stylesheet");
            let inside = &css[idx + "var(".len()..idx + close];
            assert!(
                inside.contains(','),
                "var() reference without an explicit fallback found: var({inside})"
            );
        }
    }

    // --- data-current / data-disabled 連動 ---

    #[test]
    fn indicator_current_attr_is_styled() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="carousel"][data-part="indicator"][data-current] {"#));
        assert!(css.contains("background: var(--fandhe-color-accent);"));
    }

    #[test]
    fn disabled_triggers_are_dimmed() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="carousel"][data-part="prev-trigger"][data-disabled] {"#)
        );
        assert!(
            css.contains(r#"[data-scope="carousel"][data-part="next-trigger"][data-disabled] {"#)
        );
        assert!(css.contains("opacity: 0.4;"));
    }

    #[test]
    fn carousel_stylesheet_never_consumes_color_palette_axis() {
        // 複合部品の variant 統一方針 §3: carousel は選択・チェック状態を
        // 示す部品ではないため colorPalette 軸を提供しない。
        let css = stylesheet();
        assert!(!css.contains("--fandhe-palette"));
    }
}

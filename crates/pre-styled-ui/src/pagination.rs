//! styled Pagination（headless ラッパー、イシュー #751、親 #520/#546。
//! headless 側の保留解除は #716 → #751）。
//!
//! `fandhe_frontend_headless_ui::pagination`（#751）の Item / Ellipsis /
//! PrevTrigger / NextTrigger anatomy パーツ・[`Pagination`] 状態機械・
//! [`ItemMode`]/[`PageEntry`]/[`PaginationAction`] をそのまま再エクスポート
//! し、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠・variant
//! 統一方針は [`crate::toggle_group`]/[`crate::radio_group`] の rustdoc と
//! 同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::toggle_group::root`] と同型）を本モジュールで再定義する。
//! headless 自由関数 `root` と名前衝突するため、`pub use ...::*` ではなく
//! 必要な識別子（[`item`]/[`ellipsis`]/[`prev_trigger`]/[`next_trigger`]/
//! [`Pagination`] 等）のみを選択的に再エクスポートする。
//!
//! [`Pagination`] は inherent `root()` を持つが（`crates/headless-ui/src/pagination.rs`
//! 参照）、`crate::lib` rustdoc「複合部品の variant 統一方針」節 4 の
//! 判断（[`crate::avatar::Avatar`]・[`crate::switch::Switch`] と同じ理由）
//! により、[`Pagination`] 型自体は再エクスポートしつつ headless 自由関数
//! `root` は再エクスポートしない（未スタイル root の静かな適用漏れを防ぐ
//! fail-closed）。
//!
//! # 複合部品の variant 統一方針（root のみへクラス付与）
//!
//! `size`（[`Size`]）/`palette`（[`ColorPalette`]）はいずれも [`root`] へ
//! のみクラスを付与する。[`recipe`] が root スコープへ登録する custom
//! property（`--fandhe-pagination-item-size`/`-item-font-size`）は CSS の
//! 通常のプロパティ継承により `item`/`prev-trigger`/`next-trigger` へ伝わる
//! ため、これらの slot へ個別に variant クラスを付ける必要がない
//! （[`crate::toggle_group`]/[`crate::radio_group`] と同じ設計）。
//!
//! # `data-selected`/`aria-current` について
//!
//! headless 層の `item` は `data-state` ではなく `data-selected`（存在
//! マーカー）+ `aria-current="page"` で現在ページを表す
//! （`crates/headless-ui/src/pagination.rs` 参照）。[`recipe`] の状態規則も
//! この語彙（`StateCondition::Attr("data-selected")`）に合わせる。
//!
//! # フォーカスリング（hidden-input パターン非該当）
//!
//! `item`/`prev-trigger`/`next-trigger` はネイティブ `<button>`/`<a>` 自身が
//! 実フォーカスを直接受けるため、[`crate::toggle_group`] の `item` と同じ
//! [`StateCondition::FocusVisible`] で足りる。`data-focus-visible` 配線は
//! 不要。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層の再エクスポートと静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（`href`/`aria_label`/属性/children）へ CSS 値として流し
//! 込む経路を持たない（動的値は headless 層経由で
//! `fandhe_frontend_core::render` の既定エスケープを必ず通る、REQ-1）。
//! styled `root` は [`drop_class_attr`] により呼び出し側の `class` を除去
//! してから合成するため、`class` 属性は常に単一。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - roving focus / キーボードナビゲーションは headless 層
//!   （`crates/headless-ui/src/pagination.rs`）と同じくスコープ外
//!   （wasm keynav 層の責務）。
//! - `examples/headless-pre-styled-ui` への Pagination 追加は headless-ui
//!   0.8.0 / pre-styled-ui の crates.io 公開後の追随 PR とする（過去例:
//!   #677/#704 の追随コミットと同型）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_scale_declarations, ColorPalette, Size, SlotRecipe, StateCondition, VariantValue,
};

// headless 自由関数 `root` はあえて再エクスポートしない（本モジュール冒頭
// の rustdoc「選択的 re-export」節参照、`root` は本モジュールで styled 版
// として再定義する）。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::pagination::{
    ellipsis, item, next_trigger, prev_trigger, ItemMode, PageEntry, Pagination, PaginationAction,
};

/// headless `pagination` anatomy の `data-part` 一覧
/// (`crates/headless-ui/src/pagination.rs` の `ANATOMY.part(...)` 呼び出し
/// と同期させる契約)。
const SLOTS: &[&str] = &["root", "item", "ellipsis", "prev-trigger", "next-trigger"];

/// この styled Pagination の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("pagination", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("min-width", "var(--fandhe-pagination-item-size, 2rem)"),
                decl("height", "var(--fandhe-pagination-item-size, 2rem)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "font-size",
                    "var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("text-decoration", "none"),
                decl("cursor", "pointer"),
                decl(
                    "transition",
                    "background 0.15s, border-color 0.15s, color 0.15s",
                ),
            ],
        )
        .base(
            "ellipsis",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("min-width", "var(--fandhe-pagination-item-size, 2rem)"),
                decl("height", "var(--fandhe-pagination-item-size, 2rem)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("opacity", "0.6"),
            ],
        )
        .base(
            "prev-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("min-width", "var(--fandhe-pagination-item-size, 2rem)"),
                decl("height", "var(--fandhe-pagination-item-size, 2rem)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "font-size",
                    "var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("text-decoration", "none"),
                decl("cursor", "pointer"),
            ],
        )
        .base(
            "next-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("min-width", "var(--fandhe-pagination-item-size, 2rem)"),
                decl("height", "var(--fandhe-pagination-item-size, 2rem)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "font-size",
                    "var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("text-decoration", "none"),
                decl("cursor", "pointer"),
            ],
        )
        // 現在ページ（`data-selected` 存在マーカー、headless 層
        // `crates/headless-ui/src/pagination.rs` 参照）の見た目。
        .state(
            "item",
            StateCondition::Attr("data-selected"),
            vec![
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "border-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("color", "var(--fandhe-palette-fg)"),
            ],
        )
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        .state(
            "prev-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        .state(
            "next-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        // item/prev-trigger/next-trigger はネイティブ button/a 自身が実
        // フォーカスを受けるため、hidden-input パターンの
        // data-focus-visible 配線は不要（[`crate::toggle_group`] と同じ判断）。
        .state(
            "item",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
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
        // イシュー #1681: Xs/Xl は item-size の Sm→Md→Lg 等差進行（0.5rem
        // 刻み）を両端へ外挿。font-size は Sm=Md=sm、Lg=md の段差を踏襲し、
        // Xs=xs（1 段下）、Xl=lg（1 段上）とする。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-pagination-item-size", "1rem"),
                decl(
                    "--fandhe-pagination-item-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-pagination-item-size", "1.5rem"),
                decl(
                    "--fandhe-pagination-item-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-pagination-item-size", "2rem"),
                decl(
                    "--fandhe-pagination-item-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-pagination-item-size", "2.5rem"),
                decl(
                    "--fandhe-pagination-item-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-pagination-item-size", "3rem"),
                decl(
                    "--fandhe-pagination-item-font-size",
                    "var(--fandhe-font-font-size-lg)",
                ),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(ColorPalette::Accent);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        recipe = recipe.variant(palette, "root", palette_scale_declarations(palette));
    }
    recipe
}

/// この styled Pagination が生成する静的 CSS 全量を返す（決定的。
/// [`crate::toggle_group::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去
/// してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::pagination::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::pagination;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = pagination::root(Size::Md, ColorPalette::Accent, "pagination", vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="pagination" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    aria_label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::pagination::root(aria_label, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="pagination"][data-part="item"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_selected_item_to_accent_style() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="pagination"][data-part="item"][data-selected] {"#));
        assert!(css.contains("var(--fandhe-palette, var(--fandhe-color-accent))"));
    }

    #[test]
    fn stylesheet_links_disabled_triggers_to_not_allowed_cursor() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="pagination"][data-part="prev-trigger"][data-disabled] {"#)
        );
        assert!(
            css.contains(r#"[data-scope="pagination"][data-part="next-trigger"][data-disabled] {"#)
        );
    }

    #[test]
    fn stylesheet_links_item_to_focus_visible() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="pagination"][data-part="item"]:focus-visible {"#));
    }

    // モジュール冒頭 rustdoc「複合部品の variant 統一方針」節が謳う「root の
    // --fandhe-pagination-item-font-size は item/prev-trigger/next-trigger
    // すべてに反映される」を base スタイルの実体で保証する回帰テスト
    // （Size::Sm/Lg 指定時に Prev/Next ラベルのテキストサイズが変わらない
    // 見た目不整合の再発防止、Cursor Bugbot 指摘対応）。
    #[test]
    fn prev_and_next_trigger_inherit_item_font_size_variable() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="pagination"][data-part="prev-trigger"] {"#));
        assert!(css.contains(r#"[data-scope="pagination"][data-part="next-trigger"] {"#));
        assert!(css.contains(
            "font-size: var(--fandhe-pagination-item-font-size, var(--fandhe-font-font-size-sm))"
        ));
    }

    // --- variant クラス（root のみ） ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            "pagination",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="pagination""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains("<nav"));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            "pagination",
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-pagination--size-md"));
        assert!(html.contains("fd-pagination--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-pagination--size-sm"),
            (Size::Md, "fd-pagination--size-md"),
            (Size::Lg, "fd-pagination--size-lg"),
        ] {
            let html = render(&root(
                size,
                ColorPalette::Accent,
                "pagination",
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-pagination--color-palette-accent"),
            (ColorPalette::Info, "fd-pagination--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-pagination--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-pagination--color-palette-warning",
            ),
            (ColorPalette::Danger, "fd-pagination--color-palette-danger"),
            (
                ColorPalette::Neutral,
                "fd-pagination--color-palette-neutral",
            ),
        ] {
            let html = render(&root(Size::Md, palette, "pagination", vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn reexported_item_is_not_given_variant_classes() {
        // item は root のみへクラスが付く複合部品の variant 統一方針
        // （モジュール rustdoc 参照）。item 自体には class 属性がない。
        let html = render(&item(ItemMode::Button, false, false, vec![], vec![]));
        assert!(!html.contains("class="));
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            "pagination",
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            "pagination",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="pagination""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_aria_label_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            PAYLOAD,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_item_href_and_children_are_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&item(
            ItemMode::Link { href: PAYLOAD },
            false,
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_pagination_state_machine() {
        use fandhe_frontend_headless_ui::fandhe_frontend_interactive::{
            dispatch, render_for_hydration, Hydrate,
        };

        let mut p = Pagination::new(200, 10, 1, 1, 1);
        assert!(dispatch(&mut p, "goto", "5"));
        assert_eq!(p.page(), 5);

        let hydrate_html = render(&render_for_hydration(&p));
        assert!(hydrate_html.contains("data-hydrate-page=\"5\""));

        let restored = Pagination::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored, p);
    }
}

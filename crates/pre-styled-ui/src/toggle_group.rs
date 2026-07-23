//! styled ToggleGroup（headless ラッパー、イシュー #746、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::toggle_group`（イシュー #746）の Item
//! anatomy パーツをそのまま再エクスポートし、[`stylesheet`] で既定 CSS を
//! 追加提供する。薄い委譲の根拠は [`crate::radio_group`]/[`crate::toggle`]
//! の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::toggle::root`] と同型）を本モジュールで再定義する。headless
//! 自由関数 `root` と名前衝突するため、`pub use ...::*` ではなく必要な
//! 識別子（[`item`]/[`ToggleGroup`]/[`MultiToggleGroup`]）のみを選択的に
//! 再エクスポートする。
//!
//! [`ToggleGroup`]/[`MultiToggleGroup`] 状態機械は inherent `root()` を
//! 持たない（item 系メソッドのみ、`crates/headless-ui/src/toggle_group.rs`
//! 参照）ため、[`crate::radio_group`] の `RadioGroup` と同じく、そのまま
//! 再エクスポートしても未スタイル root の静かな適用漏れは発生しない。
//!
//! # 複合部品の variant 統一方針（root のみへクラス付与）
//!
//! `size`（[`Size`]）/`palette`（[`ColorPalette`]）はいずれも [`root`] へ
//! のみクラスを付与する。[`recipe`] が root スコープへ登録する custom
//! property（`--fandhe-toggle-group-item-padding-y`/`-item-padding-x`/
//! `-item-font-size`）は CSS の通常のプロパティ継承により `item` へ伝わる
//! ため、`item` 自身へ variant クラスを付ける必要がない（`root` が `item`
//! を内包する祖先要素であるため成立する。[`crate::radio_group`] の
//! `item-control`/`item-text` と同じ設計、`crate::lib` rustdoc
//! 「複合部品の variant 統一方針」節参照）。
//!
//! # `data-state`/`aria-pressed` 語彙について
//!
//! headless 層の `item` は [`crate::toggle::root`] と同じ `"on"`/`"off"`
//! 語彙（[`crate::state::pressed_data_state`]）を使う
//! （`crates/headless-ui/src/toggle_group.rs` 参照）。[`recipe`] の状態規則
//! もこの語彙に合わせて `data-state="on"` を条件とする。
//!
//! # フォーカスリング（hidden-input パターン非該当）
//!
//! `item` はネイティブ `<button>` 自身であり実フォーカスを直接受けるため、
//! [`crate::toggle`]/[`crate::select`] の `trigger` と同じ
//! [`StateCondition::FocusVisible`] で足りる。`data-focus-visible` 配線は
//! 不要（[`crate::toggle`] モジュール rustdoc と同じ判断）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層の再エクスポートと静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（`value`/`labelled_by`/属性/children）へ CSS 値として流し
//! 込む経路を持たない（動的値は headless 層経由で
//! `fandhe_frontend_core::render` の既定エスケープを必ず通る、REQ-1）。
//! styled `root` は [`drop_class_attr`] により呼び出し側の `class` を除去
//! してから合成するため、`class` 属性は常に単一。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - roving focus / loopFocus は headless 層（`crates/headless-ui/src/toggle_group.rs`）
//!   と同じくスコープ外（wasm keynav 層の責務）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_declarations, ColorPalette, Size, SlotRecipe, StateCondition, VariantValue,
};

// headless 自由関数 `root` はあえて再エクスポートしない（本モジュール冒頭
// の rustdoc「選択的 re-export」節参照、`root` は本モジュールで styled 版
// として再定義する）。
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::toggle_group::{item, MultiToggleGroup, ToggleGroup};

/// headless `toggle-group` anatomy の `data-part` 一覧
/// (`crates/headless-ui/src/toggle_group.rs` の `ANATOMY.part(...)` 呼び出し
/// と同期させる契約)。
const SLOTS: &[&str] = &["root", "item"];

/// この styled ToggleGroup の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("toggle-group", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        // headless 層が `data_orientation` 経由で出力する
        // `data-orientation="vertical"` では縦積みへ切り替える
        // （`crate::radio_group` の `data-orientation="horizontal"` と対称。
        // 既定は横並びのため、本コンポーネントは逆側の値のみを分岐する）。
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("flex-direction", "column")],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        .base(
            "item",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "padding",
                    "var(--fandhe-toggle-group-item-padding-y, 0.375rem) var(--fandhe-toggle-group-item-padding-x, 0.75rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-toggle-group-item-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("cursor", "pointer"),
                decl(
                    "transition",
                    "background 0.15s, border-color 0.15s, color 0.15s",
                ),
            ],
        )
        .state(
            "item",
            StateCondition::AttrEq("data-state", "on"),
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
        // item はネイティブ button 自身が実フォーカスを受けるため、
        // hidden-input パターン（switch/radio_group）の data-focus-visible
        // 配線は不要（crate::toggle rustdoc と同じ判断）。
        .state(
            "item",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-toggle-group-item-padding-y", "0.25rem"),
                decl("--fandhe-toggle-group-item-padding-x", "0.5rem"),
                decl(
                    "--fandhe-toggle-group-item-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-toggle-group-item-padding-y", "0.375rem"),
                decl("--fandhe-toggle-group-item-padding-x", "0.75rem"),
                decl(
                    "--fandhe-toggle-group-item-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-toggle-group-item-padding-y", "0.5rem"),
                decl("--fandhe-toggle-group-item-padding-x", "1rem"),
                decl(
                    "--fandhe-toggle-group-item-font-size",
                    "var(--fandhe-font-font-size-md)",
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
    ] {
        recipe = recipe.variant(palette, "root", palette_declarations(palette));
    }
    recipe
}

/// この styled ToggleGroup が生成する静的 CSS 全量を返す（決定的。
/// [`crate::toggle::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去
/// してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::toggle_group::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::toggle_group;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = toggle_group::root(
///     Size::Md,
///     ColorPalette::Accent,
///     false,
///     None,
///     None,
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"data-scope="toggle-group" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    disabled: bool,
    orientation: Option<Orientation>,
    labelled_by: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::toggle_group::root(
        disabled,
        orientation,
        labelled_by,
        merged,
        children,
    )
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
        assert!(a.contains(r#"[data-scope="toggle-group"][data-part="item"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_item_to_on_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toggle-group"][data-part="item"][data-state="on"] {"#));
        assert!(css.contains("var(--fandhe-palette, var(--fandhe-color-accent))"));
    }

    #[test]
    fn stylesheet_links_root_to_vertical_orientation() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="toggle-group"][data-part="root"][data-orientation="vertical"] {"#
        ));
        assert!(css.contains("flex-direction: column;"));
    }

    #[test]
    fn stylesheet_links_item_to_focus_visible() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toggle-group"][data-part="item"]:focus-visible {"#));
    }

    // --- variant クラス（root のみ） ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toggle-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="group""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-toggle-group--size-md"));
        assert!(html.contains("fd-toggle-group--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-toggle-group--size-sm"),
            (Size::Md, "fd-toggle-group--size-md"),
            (Size::Lg, "fd-toggle-group--size-lg"),
        ] {
            let html = render(&root(
                size,
                ColorPalette::Accent,
                false,
                None,
                None,
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (
                ColorPalette::Accent,
                "fd-toggle-group--color-palette-accent",
            ),
            (ColorPalette::Info, "fd-toggle-group--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-toggle-group--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-toggle-group--color-palette-warning",
            ),
            (
                ColorPalette::Danger,
                "fd-toggle-group--color-palette-danger",
            ),
        ] {
            let html = render(&root(Size::Md, palette, false, None, None, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn item_is_not_given_variant_classes() {
        // item は root のみへクラスが付く複合部品の variant 統一方針
        // （モジュール rustdoc 参照）。item 自体には class 属性がない。
        let html = render(&item(false, false, "bold", vec![], vec![]));
        assert!(!html.contains("class="));
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
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
            false,
            None,
            None,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toggle-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_labelled_by_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            Some(PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_item_value_and_children_are_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&item(
            false,
            false,
            PAYLOAD,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_toggle_group_state_machine() {
        use fandhe_frontend_headless_ui::fandhe_frontend_interactive::{
            dispatch, render_for_hydration, Hydrate,
        };

        let mut g = ToggleGroup::default();
        assert_eq!(g.value(), None);

        assert!(dispatch(&mut g, "toggle", "bold"));
        let hydrate_html = render(&render_for_hydration(&g));
        assert!(hydrate_html.contains("data-hydrate-selected="));
        assert!(hydrate_html.contains("bold"));

        let restored = ToggleGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_multi_toggle_group_state_machine() {
        use fandhe_frontend_headless_ui::fandhe_frontend_interactive::{dispatch, Hydrate};

        let mut g = MultiToggleGroup::default();
        assert!(dispatch(&mut g, "toggle", "bold"));
        assert!(dispatch(&mut g, "toggle", "italic"));
        assert!(g.is_pressed("bold"));
        assert!(g.is_pressed("italic"));

        let restored = MultiToggleGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }
}

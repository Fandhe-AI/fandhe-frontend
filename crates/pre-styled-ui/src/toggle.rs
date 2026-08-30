//! styled Toggle（headless ラッパー、イシュー #746、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::toggle`（イシュー #746）の Indicator
//! anatomy パーツをそのまま再エクスポートし、[`stylesheet`] で既定 CSS を
//! 追加提供する。薄い委譲の根拠は [`crate::switch`]/[`crate::radio_group`]
//! の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Toggle` 型を
//! 再エクスポートしない理由）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::switch::root`]・[`crate::avatar::root`] と同型）を本モジュール
//! で再定義する。headless 自由関数 `root` と名前衝突するため、
//! `pub use ...::*` ではなく必要な識別子（[`indicator`]/[`ToggleAction`]）
//! のみを選択的に再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::toggle::Toggle`] は**あえて**
//! 再エクスポートしない（[`crate::switch`] の `Switch` 非再エクスポートと
//! 同じ理由）。`Toggle` は `.root(disabled, attrs, children)` という
//! inherent メソッドを持つが、これは headless 自由関数 `root` へそのまま
//! 委譲するのみで `size`/`palette` variant クラスを一切付与しない未スタイル
//! の実体である。本モジュールが `Toggle` を丸ごと再エクスポートすると、
//! 呼び出し側が（styled 層のつもりで）`toggle_instance.root(...)` を呼んで
//! しまい、`size`/`palette` が付与されず見た目が静かに崩れる事故を誘発
//! する。`Toggle` による状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::toggle::Toggle` を直接 import し、実際の
//! 描画は本モジュールの styled [`root`]（および再エクスポート済みの
//! [`indicator`]）を組み合わせて構築すること。
//!
//! # `data-state` 語彙について
//!
//! headless 層は Toggle を `"on"`/`"off"` 語彙（Switch の
//! `"checked"`/`"unchecked"` とは異なる）で表現する
//! （`crates/headless-ui/src/toggle.rs` の意味論差節参照）。[`recipe`] の
//! 状態規則もこの語彙に合わせて `data-state="on"` を条件とする。
//!
//! # 実フォーカスは `root` 自身が受ける（hidden-input パターン非該当）
//!
//! [`crate::switch`]/[`crate::radio_group`] は実フォーカスが visually-hidden
//! なネイティブ `<input>` にあるため `data-focus-visible` 存在属性による
//! 間接的なフォーカスリング伝播が必要だった（イシュー #709）。[`root`] は
//! ネイティブ `<button>` 自身であり実フォーカスを直接受けるため、
//! [`crate::select`] の `trigger` と同じ [`StateCondition::FocusVisible`]
//! （`:focus-visible` 擬似クラスセレクタ）で足りる。`data-focus-visible`
//! 配線は不要。
//!
//! # `size`/`palette` variant
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-toggle-padding-x`/`-padding-y`/`-font-size` の root スコープ
//! custom property（CSS の通常のプロパティ継承・`root` 自身への直接適用で
//! 寸法を切り替える）。`palette`（[`ColorPalette`]）は既存の
//! [`crate::recipe::palette_scale_declarations`]（chakra-ui virtual token 方式、
//! #606）を `root` へ登録し、on 時の背景色を `var(--fandhe-palette, ...)`
//! 経由で切り替える。`base`/`state` 規則の `var()` にはいずれも Md サイズ・
//! Accent パレット相当のフォールバック値を書き、styled `root` を経由しない
//! headless 直接利用マークアップでも現行外観を維持する（fail-safe、
//! `crate::lib` rustdoc「複合部品の variant 統一方針」節参照）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層の再エクスポートと静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（属性/children）へ CSS 値として流し込む経路を持たない
//! （動的値は headless 層経由で `fandhe_frontend_core::render` の既定
//! エスケープを必ず通る、REQ-1）。styled `root` は [`drop_class_attr`]
//! により呼び出し側の `class` を除去してから合成するため、`class` 属性は
//! 常に単一（[`crate::switch::root`] と同型）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_scale_declarations, ColorPalette, Size, SlotRecipe, StateCondition, VariantValue,
};

// headless 自由関数 `root` はあえて再エクスポートしない（本モジュール冒頭
// の rustdoc「選択的 re-export」節参照、`root` は本モジュールで styled 版
// として再定義する）。`Toggle` 状態機械もあえて再エクスポートしない
// （同節参照）。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::toggle::{indicator, ToggleAction};

/// headless `toggle` anatomy の `data-part` 一覧（`crates/headless-ui/src/toggle.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &["root", "indicator"];

/// この styled Toggle の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("toggle", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("box-sizing", "border-box"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "padding",
                    "var(--fandhe-toggle-padding-y, 0.375rem) var(--fandhe-toggle-padding-x, 0.75rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-toggle-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("cursor", "pointer"),
                decl("transition", "background 0.15s, border-color 0.15s, color 0.15s"),
            ],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-state", "on"),
            vec![
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("border-color", "var(--fandhe-palette, var(--fandhe-color-accent))"),
                decl("color", "var(--fandhe-palette-fg)"),
            ],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        // 実フォーカスは root 自身（ネイティブ button）が受けるため、
        // hidden-input パターン（switch/radio_group）の data-focus-visible
        // 配線は不要（モジュール rustdoc 参照）。select の trigger と同じ
        // :focus-visible 擬似クラスで足りる。
        .state(
            "root",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .base("indicator", vec![decl("display", "inline-flex")])
        // indicator は off 時に非表示化する（headless 層は data-state のみを
        // 出力する最小主義パーツのため、表示切り替えは styled 層 CSS の責務。
        // `crates/headless-ui/src/toggle.rs` モジュール doc 参照）。
        .state(
            "indicator",
            StateCondition::AttrEq("data-state", "off"),
            vec![decl("display", "none")],
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-toggle-padding-y", "0.125rem"),
                decl("--fandhe-toggle-padding-x", "0.25rem"),
                decl("--fandhe-toggle-font-size", "var(--fandhe-font-font-size-xs)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-toggle-padding-y", "0.25rem"),
                decl("--fandhe-toggle-padding-x", "0.5rem"),
                decl(
                    "--fandhe-toggle-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-toggle-padding-y", "0.375rem"),
                decl("--fandhe-toggle-padding-x", "0.75rem"),
                decl(
                    "--fandhe-toggle-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-toggle-padding-y", "0.5rem"),
                decl("--fandhe-toggle-padding-x", "1rem"),
                decl(
                    "--fandhe-toggle-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-toggle-padding-y", "0.625rem"),
                decl("--fandhe-toggle-padding-x", "1.25rem"),
                decl("--fandhe-toggle-font-size", "var(--fandhe-font-font-size-lg)"),
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

/// この styled Toggle が生成する静的 CSS 全量を返す（決定的。
/// [`crate::switch::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去
/// してから合成する）。実体は [`fandhe_frontend_headless_ui::toggle::root`]
/// へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::toggle;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = toggle::root(Size::Md, ColorPalette::Accent, false, false, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="toggle" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    pressed: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::toggle::root(pressed, disabled, merged, children)
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
        assert!(a.contains(r#"[data-scope="toggle"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_root_to_on_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toggle"][data-part="root"][data-state="on"] {"#));
        assert!(css.contains("var(--fandhe-palette, var(--fandhe-color-accent))"));
    }

    #[test]
    fn stylesheet_links_root_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toggle"][data-part="root"][data-disabled] {"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn stylesheet_links_root_to_focus_visible() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toggle"][data-part="root"]:focus-visible {"#));
    }

    #[test]
    fn stylesheet_hides_indicator_when_off() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toggle"][data-part="indicator"][data-state="off"] {"#));
        assert!(css.contains("display: none;"));
    }

    // --- variant クラス ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toggle""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains("<button"));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-toggle--size-md"));
        assert!(html.contains("fd-toggle--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-toggle--size-xs"),
            (Size::Sm, "fd-toggle--size-sm"),
            (Size::Md, "fd-toggle--size-md"),
            (Size::Lg, "fd-toggle--size-lg"),
            (Size::Xl, "fd-toggle--size-xl"),
        ] {
            let html = render(&root(
                size,
                ColorPalette::Accent,
                false,
                false,
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-toggle--color-palette-accent"),
            (ColorPalette::Info, "fd-toggle--color-palette-info"),
            (ColorPalette::Success, "fd-toggle--color-palette-success"),
            (ColorPalette::Warning, "fd-toggle--color-palette-warning"),
            (ColorPalette::Danger, "fd-toggle--color-palette-danger"),
            (ColorPalette::Neutral, "fd-toggle--color-palette-neutral"),
        ] {
            let html = render(&root(Size::Md, palette, false, false, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            false,
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
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toggle""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            false,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_indicator_children_are_escaped_on_render() {
        let html = render(&indicator(
            true,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_toggle_state_machine() {
        // `Toggle` は本モジュールから再エクスポートしない（本モジュール冒頭の
        // rustdoc「`Toggle` 型を再エクスポートしない理由」参照）ため、
        // headless-ui から直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_headless_ui::toggle::Toggle;
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut t = Toggle::default();
        assert!(!t.is_pressed());

        let ssr_html = render(&t.root(false, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="off""#));

        assert!(dispatch(&mut t, "toggle", ""));
        let hydrate_html = render(&render_for_hydration(&t));
        assert!(hydrate_html.contains(r#"data-hydrate-checked="checked""#));

        let restored = Toggle::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }
}

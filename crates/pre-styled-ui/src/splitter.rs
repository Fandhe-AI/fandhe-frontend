//! styled Splitter（headless ラッパー、イシュー #826、親トラッキング #520）。
//!
//! `fandhe_frontend_headless_ui::splitter`（イシュー #826）の
//! ResizeTriggerIndicator anatomy パーツをそのまま再エクスポートし、
//! [`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠は
//! [`crate::slider`]/[`crate::switch`] の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::slider::root`] と同型）を本モジュールで再定義する。headless
//! 自由関数 `root` と名前衝突するため、`pub use ...::*` ではなく必要な識別子
//! （[`resize_trigger_indicator`]/[`SplitterAction`]/[`PanelSpec`]）のみを
//! 選択的に再エクスポートする。
//!
//! `panel`/`resize_trigger` も再エクスポートしない。動的な唯一の伝搬経路
//! （[`Splitter::size`](fandhe_frontend_headless_ui::splitter::Splitter::size)
//! から導出する `--fandhe-splitter-size` CSS custom property、モジュール doc
//! 「動的な値は 1 点のみ」参照）は本モジュールの styled [`panel`] が一元的に
//! 組み立てる。headless 自由関数 `panel` を呼び出し側が直接使うとこの唯一の
//! 経路を経由せず伸縮しない事故を誘発するため、意図的に非公開のまま
//! [`panel`]/[`resize_trigger`] 内部からのみ委譲する。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::splitter::Splitter`] も**あえて**
//! 再エクスポートしない（[`crate::slider`] の `Slider` 非再エクスポートと
//! 同じ理由）。`Splitter` は `.root(disabled, attrs, children)` 等の inherent
//! メソッドを持つが、これは headless 自由関数へそのまま委譲するのみで
//! `size`/`palette` variant クラス・`--fandhe-splitter-size` を一切付与
//! しない未スタイルの実体である。本モジュールが `Splitter` を丸ごと再
//! エクスポートすると、呼び出し側が（styled 層のつもりで）
//! `splitter_instance.root(...)`/`splitter_instance.panel(...)` を呼んで
//! しまい、見た目が静かに崩れる事故を誘発する。`Splitter` による状態管理・
//! hydration が必要な呼び出し側は `fandhe_frontend_headless_ui::splitter::Splitter`
//! を直接 import し、実際の描画は本モジュールの styled [`root`]/[`panel`]
//! （および再エクスポート済みのパーツ関数）を組み合わせて構築すること。
//!
//! # 動的な値は `--fandhe-splitter-size` の 1 点のみ（イシュー本文指定の
//! flex-basis 方式）
//!
//! [`panel`] は headless 中立な
//! [`Splitter::size`](fandhe_frontend_headless_ui::splitter::Splitter::size)
//! （0.0..=100.0 の正規化済み有限 `f64`）から [`percent_style`] が組み立てる
//! `style="--fandhe-splitter-size: <pct>%"` の 1 属性のみで伸縮を伝搬する。
//! [`recipe`] は `[data-scope="splitter"][data-part="panel"]` に
//! `flex-basis: var(--fandhe-splitter-size, auto); flex-grow: 0;
//! flex-shrink: 1; overflow: hidden;` を登録し、root の `display: flex` と
//! 組み合わせてパネル幅（高さ）を決定する。[`crate::slider`]/
//! [`crate::progress`] と同様に [`drop_style_attr`]（[`crate::progress`]
//! の同名ヘルパと同型の判断）で呼び出し側 `attrs` に含まれる `style`
//! （大文字小文字を無視）を除去してからフレームワーク側の `style` を優先
//! する（重複属性による無効な HTML 出力・後勝ちの非決定的な描画を防ぐ、
//! fail-closed）。
//!
//! # `size`/`palette` variant
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-splitter-trigger-size` の root スコープ custom property
//! （CSS の通常のプロパティ継承により `resize-trigger` へ伝わる）経由で
//! トリガーの厚みを切り替える（[`crate::slider`] の
//! `--fandhe-slider-track-height` と同型）。`palette`（[`ColorPalette`]）は
//! 既存の [`crate::recipe::palette_declarations`]（chakra-ui virtual token
//! 方式、#606）を `root` へ登録し、`resize-trigger` の強調色を
//! `var(--fandhe-palette, ...)` 経由で切り替える。
//!
//! # 縦方向（`data-orientation="vertical"`）レイアウト
//!
//! `root` は `data-orientation="vertical"` のとき `flex-direction: column`
//! を取り、`resize-trigger` はカーソルを `col-resize`/`row-resize` で
//! 切り替える（[`StateCondition::AttrEq("data-orientation", "vertical")`]）。
//!
//! # `focus-visible`（キーボードフォーカスリング）
//!
//! [`resize_trigger`] はネイティブにフォーカス可能な要素（`tabindex`）で
//! あるため、[`crate::switch`] のような hidden-input 特有の
//! `data-focus-visible` 対応は不要で、通常の `:focus-visible` 疑似クラスを
//! [`recipe`] へ直接登録する（[`StateCondition::FocusVisible`]）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく pointer ドラッグ・キーボード操作の DOM 配線、
//!   collapse/expand・`onResize`/`onCollapse` コールバックはスコープ外
//!   （[`fandhe_frontend_headless_ui::splitter`] モジュール doc 参照）。
//! - `examples/headless-pre-styled-ui`（crates.io バージョン依存）への
//!   Splitter 追加は、未公開の新バージョンを参照できないため本イシューの
//!   スコープ外とする（[`crate::number_input`]/[`crate::slider`] 冒頭
//!   rustdoc の先例どおり crates.io 公開後に追随）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_declarations, ColorPalette, Size, SlotRecipe, StateCondition, VariantValue,
};

// `Splitter` 状態機械・headless 自由関数 `root`/`panel`/`resize_trigger` は
// あえて再エクスポートしない（本モジュール冒頭の rustdoc「選択的
// re-export」節参照）。状態管理・hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::splitter::Splitter` を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::splitter::Splitter;
pub use fandhe_frontend_headless_ui::splitter::{
    resize_trigger_indicator, PanelSpec, SplitterAction,
};

/// headless `splitter` anatomy の `data-part` 一覧（`crates/headless-ui/src/splitter.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "panel",
    "resize-trigger",
    "resize-trigger-indicator",
];

/// `attrs` から `style`（ASCII 大文字小文字を無視）を除いた列を返す。
///
/// [`panel`] がフレームワーク側で `--fandhe-splitter-size` を含む `style`
/// を組み立てた後、呼び出し側 `attrs` を連結する前に使う dedup ヘルパ
/// （`crates/pre-styled-ui/src/slider.rs::drop_style_attr` と同型の判断。
/// 重複属性による無効な HTML 出力・後勝ちの非決定的な描画を防ぐ、
/// fail-closed）。
fn drop_style_attr<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("style"))
        .collect()
}

/// `percent`（[`Splitter::size`] が返す正規化済み有限 `f64`）から
/// `--fandhe-splitter-size` custom property を設定する `style` 属性値を
/// 組み立てる（動的値はこの 1 箇所のみ、モジュール doc 参照）。
fn percent_style(percent: f64) -> String {
    format!("--fandhe-splitter-size: {percent}%")
}

/// この styled Splitter の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("splitter", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("align-items", "stretch"),
                decl("width", "100%"),
            ],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("flex-direction", "column")],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.5")],
        )
        .base(
            "panel",
            vec![
                decl("flex-basis", "var(--fandhe-splitter-size, auto)"),
                decl("flex-grow", "0"),
                decl("flex-shrink", "1"),
                decl("overflow", "hidden"),
            ],
        )
        .base(
            "resize-trigger",
            vec![
                decl("flex", "0 0 var(--fandhe-splitter-trigger-size, 0.25rem)"),
                decl("background", "var(--fandhe-color-border)"),
                decl("cursor", "col-resize"),
                decl(
                    "box-shadow",
                    "inset 0 0 0 9999px var(--fandhe-palette, transparent)",
                ),
            ],
        )
        .state(
            "resize-trigger",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("cursor", "row-resize")],
        )
        .state(
            "resize-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        .state(
            "resize-trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "-2px"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("--fandhe-splitter-trigger-size", "0.125rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-splitter-trigger-size", "0.25rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-splitter-trigger-size", "0.375rem")],
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

/// この styled Splitter が生成する静的 CSS 全量を返す（決定的。
/// [`crate::slider::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は
/// 除去してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::splitter::Splitter::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_headless_ui::splitter::{PanelSpec, Splitter};
/// use fandhe_frontend_headless_ui::Orientation;
/// use fandhe_frontend_pre_styled_ui::splitter;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let s = Splitter::new(
///     &[
///         PanelSpec::new(50.0, 0.0, 100.0),
///         PanelSpec::new(50.0, 0.0, 100.0),
///     ],
///     Orientation::Horizontal,
/// );
/// let node = splitter::root(Size::Md, ColorPalette::Accent, &s, false, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="splitter" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    state: &Splitter,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    state.root(disabled, merged, children)
}

/// styled panel パーツを組み立てる。`--fandhe-splitter-size` を含む `style`
/// を付与する唯一のパーツ（[`drop_style_attr`] により呼び出し側の `style`
/// は除去してから合成する。動的値はこの 1 箇所のみ、モジュール doc「動的な
/// 値は 1 点のみ」参照）。実体は
/// [`fandhe_frontend_headless_ui::splitter::Splitter::panel`] へ委譲する。
///
/// `panel_index` は [`fandhe_frontend_headless_ui::splitter::Splitter::size`]
/// の添字（`0..panel_count()`）。範囲外の場合は `flex-basis` を出力せず
/// `auto` へフォールバックする（fail-closed。[`Splitter::size`] が `None` を
/// 返すため、[`percent_style`] を呼ばず `style` 属性自体を省略する）。
#[must_use]
pub fn panel<'a>(
    state: &Splitter,
    panel_index: usize,
    id: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let style = state.size(panel_index).map(percent_style);
    let mut merged: Vec<(&str, &str)> = Vec::with_capacity(attrs.len() + 1);
    if let Some(style) = style.as_deref() {
        merged.push(("style", style));
    }
    merged.extend(drop_style_attr(attrs));
    state.panel(id, merged, children)
}

/// styled resize-trigger パーツを組み立てる。実体は
/// [`fandhe_frontend_headless_ui::splitter::Splitter::resize_trigger`] へ
/// 委譲する（動的値の伝搬は [`panel`] の `--fandhe-splitter-size` 経由のみ
/// のため、本関数自体は追加の `style` を持たない）。
#[must_use]
pub fn resize_trigger<'a>(
    state: &Splitter,
    trigger: usize,
    panel_id: &'a str,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.resize_trigger(trigger, panel_id, disabled, attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_headless_ui::data_attrs::Orientation;

    fn default_state() -> Splitter {
        Splitter::new(
            &[
                PanelSpec::new(50.0, 0.0, 100.0),
                PanelSpec::new(50.0, 0.0, 100.0),
            ],
            Orientation::Horizontal,
        )
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="splitter"][data-part="panel"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_references_size_custom_property_as_flex_basis() {
        let css = stylesheet();
        assert!(css.contains("--fandhe-splitter-size"));
        assert!(css.contains("flex-basis: var(--fandhe-splitter-size, auto);"));
    }

    #[test]
    fn stylesheet_links_root_to_vertical_orientation() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="splitter"][data-part="root"][data-orientation="vertical"] {"#
        ));
        assert!(css.contains("flex-direction: column;"));
    }

    #[test]
    fn stylesheet_links_root_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="splitter"][data-part="root"][data-disabled] {"#));
    }

    #[test]
    fn stylesheet_links_resize_trigger_to_focus_visible() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="splitter"][data-part="resize-trigger"]:focus-visible {"#)
        );
    }

    #[test]
    fn stylesheet_contains_size_and_palette_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--color-palette-"));
        assert!(css.contains("--fandhe-splitter-trigger-size"));
    }

    // --- root ---

    #[test]
    fn root_outputs_scope_and_part() {
        let s = default_state();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="splitter""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let s = default_state();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-splitter--size-md"));
        assert!(html.contains("fd-splitter--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        let s = default_state();
        for (size, class) in [
            (Size::Sm, "fd-splitter--size-sm"),
            (Size::Md, "fd-splitter--size-md"),
            (Size::Lg, "fd-splitter--size-lg"),
        ] {
            let html = render(&root(size, ColorPalette::Accent, &s, false, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        let s = default_state();
        for (palette, class) in [
            (ColorPalette::Accent, "fd-splitter--color-palette-accent"),
            (ColorPalette::Info, "fd-splitter--color-palette-info"),
            (ColorPalette::Success, "fd-splitter--color-palette-success"),
            (ColorPalette::Warning, "fd-splitter--color-palette-warning"),
            (ColorPalette::Danger, "fd-splitter--color-palette-danger"),
        ] {
            let html = render(&root(Size::Md, palette, &s, false, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let s = default_state();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let s = default_state();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="splitter""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- panel: --fandhe-splitter-size の唯一の動的値経路 ---

    #[test]
    fn panel_outputs_size_style() {
        let s = Splitter::new(
            &[
                PanelSpec::new(30.0, 0.0, 100.0),
                PanelSpec::new(70.0, 0.0, 100.0),
            ],
            Orientation::Horizontal,
        );
        let html = render(&panel(&s, 0, "panel-a", vec![], vec![]));
        assert!(html.contains(r#"style="--fandhe-splitter-size: 30%""#));
        let html = render(&panel(&s, 1, "panel-b", vec![], vec![]));
        assert!(html.contains(r#"style="--fandhe-splitter-size: 70%""#));
    }

    #[test]
    fn panel_out_of_range_index_omits_style_attr() {
        let s = default_state();
        let html = render(&panel(&s, 99, "panel-x", vec![], vec![]));
        assert!(!html.contains("style="));
    }

    #[test]
    fn panel_caller_style_attr_is_dropped_not_duplicated() {
        let s = default_state();
        let html = render(&panel(
            &s,
            0,
            "panel-a",
            vec![("style", "attacker: 1")],
            vec![],
        ));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn resize_trigger_outputs_role_and_controls() {
        let s = default_state();
        let html = render(&resize_trigger(&s, 0, "panel-a", false, vec![], vec![]));
        assert!(html.contains(r#"role="separator""#));
        assert!(html.contains(r#"aria-controls="panel-a""#));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let s = default_state();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_resize_trigger_indicator_children_are_escaped_on_render() {
        let html = render(&resize_trigger_indicator(
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn panel_id_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let s = default_state();
        let html = render(&panel(&s, 0, PAYLOAD, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_splitter_state_machine() {
        // `Splitter` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「`Splitter` 型を再エクスポートしない理由」参照）
        // ため、headless-ui から直接 import して state machine 契約のみ
        // 検証する。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut s = default_state();
        let ssr_html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![],
            vec![],
        ));
        assert!(!ssr_html.contains("data-hydrate-"));

        assert!(dispatch(&mut s, "set", "0:70"));
        assert_eq!(s.size(0), Some(70.0));

        let hydrate_html = render(&render_for_hydration(&s));
        assert!(hydrate_html.contains(r#"data-hydrate-sizes="70,30""#));

        let restored = Splitter::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }
}

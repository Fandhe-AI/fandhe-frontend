//! styled Checkbox（headless ラッパー第 5 弾、イシュー #730、親 #520/#545、
//! `size`/`palette` variant・`data-focus-visible` フォーカスリングは
//! [`mod@switch`](crate::switch)/[`mod@radio_group`](crate::radio_group)
//! （#708/#709）と同型で最初から実装する）。
//!
//! `fandhe_frontend_headless_ui::checkbox`（イシュー #535/#595）の root /
//! control / indicator / label / hidden-input 5 anatomy パーツへ
//! [`stylesheet`] で既定 CSS を対応付ける薄い委譲層である。設計方針の根拠は
//! [`crate::switch`] rustdoc「複合部品の variant 統一方針」節（#708）と同じ。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Checkbox` 型・
//! headless `root` を再エクスポートしない理由）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::switch::root`]・[`crate::radio_group::root`] と同型）を本
//! モジュールで再定義する。headless 自由関数 `root` と名前衝突するため、
//! `pub use ...::*` ではなく必要な識別子（[`control`]/[`indicator`]/
//! [`label`]/[`hidden_input`]/[`CheckboxProps`]/[`CheckboxFlags`]/
//! [`CheckedState`]/[`CheckboxAction`]）のみを選択的に再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::checkbox::Checkbox`] は**あえて**
//! 再エクスポートしない（[`crate::switch`] の `Switch` 非再エクスポートと
//! 同じ理由、PR #695 Bugbot 指摘の前例）。`Checkbox` は
//! `.root(flags, attrs, children)` という inherent メソッドを持つが、これは
//! headless 自由関数 `root` へそのまま委譲するのみで `size`/`palette`
//! variant クラスを一切付与しない未スタイルの実体である。本モジュールが
//! `Checkbox` を丸ごと再エクスポートすると、呼び出し側が（styled 層のつもり
//! で）`checkbox_instance.root(...)` を呼んでしまい、`size`/`palette` が
//! 付与されず見た目が静かに崩れる事故を誘発する。`Checkbox` による状態
//! 管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::checkbox::Checkbox` を直接 import し、実際の
//! 描画は本モジュールの styled [`root`]（および再エクスポート済みのパーツ
//! 関数）を組み合わせて構築すること。
//!
//! # `hidden-input` は `display: none` にしない（視覚的非表示化の判断）
//!
//! headless 層の `hidden_input` は `<input type="checkbox">` で意味論・
//! フォーム送信・キーボード操作を担う実体であり、視覚的な見た目（チェック
//! ボックス自体）は `control`/`indicator` が装飾として担う。この 2 層構造を
//! 保ちつつ `hidden_input` 自体のフォーカス・タブ順・支援技術からの到達性を
//! 失わないため、[`crate::switch`]/[`crate::select`] と同じ visually-hidden
//! パターン（`position: absolute` + 1px クリップ）を採用する。
//!
//! # `indicator` の `hidden` 属性意味論を CSS が壊さない設計
//!
//! headless 層の [`indicator`] は [`CheckedState::Unchecked`] のとき `hidden`
//! 存在属性を付与して非表示化する（`crates/headless-ui/src/checkbox.rs`
//! 参照）。ブラウザ UA stylesheet は `[hidden] { display: none !important }`
//! 相当の規則を持つが、本モジュールの recipe が `indicator` の `base` へ
//! `display` 宣言を書くと詳細度次第で UA 規則を上書きしてしまい、unchecked
//! 時にもチェックマークが見えてしまう回帰を招く。そのため **`indicator` の
//! `base` には `display` 宣言を一切置かない**（`indicator_base_has_no_display_declaration`
//! テストで固定）。checked/indeterminate 時の見た目切り替えは `display`
//! ではなく `border`/`transform`/`width`/`height` の組み合わせで表現する。
//!
//! # `data-focus-visible` フォーカスリング（イシュー #709 契約の踏襲）
//!
//! 実フォーカスは `hidden-input` が受けるため、[`crate::switch`] の
//! `control` と同型のフォーカスリング条件（`StateCondition::Attr("data-focus-visible")`）
//! を `control` slot へ登録する。属性の付け外し自体は headless/wasm 層の
//! 責務（`crates/wasm-full/src/focus_visible.rs` に `("checkbox",
//! "hidden-input") => Some("root")` のマッピングが登録済み）であり、本
//! モジュールは CSS 規則の登録のみを行う。
//!
//! # `size`/`palette` variant
//!
//! [`crate::switch`] rustdoc「複合部品の variant 統一方針」節（#708）に従い、
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-checkbox-control-size`/`-check-width`/`-check-height`/
//! `-label-font-size` の root スコープ custom property（通常の CSS 継承）
//! 経由で `control`/`indicator`/`label` の寸法を切り替える。`palette`
//! （[`ColorPalette`]）は既存の [`crate::recipe::palette_scale_declarations`] を
//! `root` へ登録し、checked/indeterminate 時の `control` 背景・境界線色を
//! `var(--fandhe-palette, ...)` 経由で切り替える。base/state 規則の `var()`
//! にはいずれも Md サイズ・Accent パレット相当のフォールバック値を書き、
//! styled `root` を経由しない headless 直接利用マークアップでも現行外観を
//! 維持する（fail-safe）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_scale_declarations, ColorPalette, Size, SlotRecipe, StateCondition, VariantValue,
};

// `Checkbox` 状態機械・headless 自由関数 `root` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::checkbox::Checkbox` を直接 import する。
pub use fandhe_frontend_headless_ui::checkbox::{
    control, hidden_input, indicator, label, CheckboxAction, CheckboxFlags, CheckboxProps,
    CheckedState,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// headless `checkbox` anatomy の `data-part` 一覧（`crates/headless-ui/src/checkbox.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &["root", "control", "indicator", "label", "hidden-input"];

/// この styled Checkbox の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("checkbox", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("cursor", "pointer"),
            ],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        .base(
            "control",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("width", "var(--fandhe-checkbox-control-size, 1rem)"),
                decl("height", "var(--fandhe-checkbox-control-size, 1rem)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("flex-shrink", "0"),
                decl("transition", "background 0.15s, border-color 0.15s"),
            ],
        )
        .state(
            "control",
            StateCondition::AttrEq("data-state", "checked"),
            vec![
                decl(
                    "border-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
            ],
        )
        .state(
            "control",
            StateCondition::AttrEq("data-state", "indeterminate"),
            vec![
                decl(
                    "border-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
            ],
        )
        // イシュー #709: 実フォーカスは hidden-input が受けるため、wasm 層
        // （`fandhe-frontend-wasm-full` の focus 配線）が `control` へも
        // 付け外しする `data-focus-visible` をキーボード操作専用のフォーカス
        // リング条件として使う（`switch` の `control`
        // `StateCondition::Attr("data-focus-visible")` と同型の視覚言語、
        // モジュール rustdoc 参照）。
        .state(
            "control",
            StateCondition::Attr("data-focus-visible"),
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        // `indicator` の base に `display` 宣言を置かない（モジュール rustdoc
        // 「`indicator` の `hidden` 属性意味論を CSS が壊さない設計」節参照。
        // `indicator_base_has_no_display_declaration` テストで固定）。
        .base(
            "indicator",
            vec![
                decl("width", "var(--fandhe-checkbox-check-width, 0.25rem)"),
                decl("height", "var(--fandhe-checkbox-check-height, 0.5rem)"),
                decl(
                    "border-right",
                    "2px solid var(--fandhe-palette-fg, var(--fandhe-color-accent-fg))",
                ),
                decl(
                    "border-bottom",
                    "2px solid var(--fandhe-palette-fg, var(--fandhe-color-accent-fg))",
                ),
                decl("transform", "rotate(45deg)"),
                decl("margin-bottom", "0.1rem"),
            ],
        )
        .state(
            "indicator",
            StateCondition::AttrEq("data-state", "indeterminate"),
            vec![
                decl("transform", "none"),
                decl("border-right", "0"),
                decl(
                    "border-bottom",
                    "2px solid var(--fandhe-palette-fg, var(--fandhe-color-accent-fg))",
                ),
                decl("width", "var(--fandhe-checkbox-dash-width, 0.5rem)"),
                decl("height", "0"),
                decl("margin-bottom", "0"),
            ],
        )
        .base(
            "label",
            vec![decl(
                "font-size",
                "var(--fandhe-checkbox-label-font-size, var(--fandhe-font-font-size-sm))",
            )],
        )
        // hidden-input の視覚的非表示化（[`crate::switch`]/[`crate::select`] と
        // 同じ visually-hidden パターン。モジュール doc 参照）。
        .base(
            "hidden-input",
            vec![
                decl("position", "absolute"),
                decl("width", "1px"),
                decl("height", "1px"),
                decl("padding", "0"),
                decl("margin", "-1px"),
                decl("overflow", "hidden"),
                decl("clip", "rect(0, 0, 0, 0)"),
                decl("white-space", "nowrap"),
                decl("border", "0"),
            ],
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-checkbox-control-size", "0.7rem"),
                decl("--fandhe-checkbox-check-width", "0.15rem"),
                decl("--fandhe-checkbox-check-height", "0.3rem"),
                decl("--fandhe-checkbox-dash-width", "0.3rem"),
                decl(
                    "--fandhe-checkbox-label-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-checkbox-control-size", "0.85rem"),
                decl("--fandhe-checkbox-check-width", "0.2rem"),
                decl("--fandhe-checkbox-check-height", "0.4rem"),
                decl("--fandhe-checkbox-dash-width", "0.4rem"),
                decl(
                    "--fandhe-checkbox-label-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-checkbox-control-size", "1rem"),
                decl("--fandhe-checkbox-check-width", "0.25rem"),
                decl("--fandhe-checkbox-check-height", "0.5rem"),
                decl("--fandhe-checkbox-dash-width", "0.5rem"),
                decl(
                    "--fandhe-checkbox-label-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-checkbox-control-size", "1.25rem"),
                decl("--fandhe-checkbox-check-width", "0.3rem"),
                decl("--fandhe-checkbox-check-height", "0.6rem"),
                decl("--fandhe-checkbox-dash-width", "0.6rem"),
                decl(
                    "--fandhe-checkbox-label-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-checkbox-control-size", "1.5rem"),
                decl("--fandhe-checkbox-check-width", "0.35rem"),
                decl("--fandhe-checkbox-check-height", "0.7rem"),
                decl("--fandhe-checkbox-dash-width", "0.7rem"),
                decl(
                    "--fandhe-checkbox-label-font-size",
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

/// この styled Checkbox が生成する静的 CSS 全量を返す（決定的。
/// [`crate::switch::stylesheet`]/[`crate::radio_group::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与する
/// 唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去して
/// から合成する）。実体は [`fandhe_frontend_headless_ui::checkbox::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = checkbox::root(Size::Md, ColorPalette::Accent, &CheckboxProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="checkbox" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    props: &CheckboxProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::checkbox::root(props, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

    fn unchecked() -> CheckboxProps {
        CheckboxProps::default()
    }

    fn checked() -> CheckboxProps {
        CheckboxProps {
            checked: CheckedState::Checked,
            ..CheckboxProps::default()
        }
    }

    fn indeterminate() -> CheckboxProps {
        CheckboxProps {
            checked: CheckedState::Indeterminate,
            ..CheckboxProps::default()
        }
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="checkbox"][data-part="control"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_control_to_checked_and_indeterminate_state() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="checkbox"][data-part="control"][data-state="checked"] {
  border-color: var(--fandhe-palette, var(--fandhe-color-accent));
  background: var(--fandhe-palette, var(--fandhe-color-accent));
}"#
        ));
        assert!(css.contains(
            r#"[data-scope="checkbox"][data-part="control"][data-state="indeterminate"] {"#
        ));
    }

    #[test]
    fn stylesheet_links_control_to_focus_visible_outline() {
        // 受け入れ条件 2: switch control と同型の outline 規則。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="checkbox"][data-part="control"][data-focus-visible] {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}"#
        ));
    }

    #[test]
    fn stylesheet_links_root_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="checkbox"][data-part="root"][data-disabled] {"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn hidden_input_is_visually_hidden_not_display_none() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="checkbox"][data-part="hidden-input"] {"#));
        assert!(css.contains("clip: rect(0, 0, 0, 0);"));
        assert!(!css.contains("display: none"));
    }

    #[test]
    fn indicator_base_has_no_display_declaration() {
        // `hidden` 属性の意味論（UA stylesheet の `[hidden] { display: none }`）を
        // CSS が上書きしないことの回帰（モジュール rustdoc 参照）。
        let css = stylesheet();
        let start = css
            .find(r#"[data-scope="checkbox"][data-part="indicator"] {"#)
            .expect("indicator base block must exist");
        let end = css[start..]
            .find('}')
            .map(|i| start + i)
            .unwrap_or(css.len());
        assert!(
            !css[start..end].contains("display"),
            "indicator base block must not declare display: {}",
            &css[start..end]
        );
    }

    // --- variant クラス ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &unchecked(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="checkbox""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &unchecked(),
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-checkbox--size-md"));
        assert!(html.contains("fd-checkbox--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-checkbox--size-xs"),
            (Size::Sm, "fd-checkbox--size-sm"),
            (Size::Md, "fd-checkbox--size-md"),
            (Size::Lg, "fd-checkbox--size-lg"),
            (Size::Xl, "fd-checkbox--size-xl"),
        ] {
            let html = render(&root(
                size,
                ColorPalette::Accent,
                &unchecked(),
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-checkbox--color-palette-accent"),
            (ColorPalette::Info, "fd-checkbox--color-palette-info"),
            (ColorPalette::Success, "fd-checkbox--color-palette-success"),
            (ColorPalette::Warning, "fd-checkbox--color-palette-warning"),
            (ColorPalette::Danger, "fd-checkbox--color-palette-danger"),
            (ColorPalette::Neutral, "fd-checkbox--color-palette-neutral"),
        ] {
            let html = render(&root(Size::Md, palette, &unchecked(), vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &unchecked(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_contains_size_and_palette_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--color-palette-"));
        assert!(css.contains("--fandhe-checkbox-control-size"));
    }

    #[test]
    fn size_variants_set_label_font_size_custom_property() {
        let css = stylesheet();
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl] {
            let selector = format!(
                r#"[data-scope="checkbox"][data-part="root"].fd-checkbox--size-{}"#,
                size.value()
            );
            let start = css
                .find(&selector)
                .unwrap_or_else(|| panic!("size variant selector not found: {selector} in {css}"));
            let block_end = css[start..]
                .find('}')
                .map(|i| start + i)
                .unwrap_or(css.len());
            assert!(
                css[start..block_end].contains("--fandhe-checkbox-label-font-size"),
                "size={size:?} variant block missing --fandhe-checkbox-label-font-size: {}",
                &css[start..block_end]
            );
        }
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &unchecked(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="checkbox""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn root_reflects_checked_and_indeterminate_props() {
        let checked_html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &checked(),
            vec![],
            vec![],
        ));
        assert!(checked_html.contains(r#"data-state="checked""#));

        let indeterminate_html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &indeterminate(),
            vec![],
            vec![],
        ));
        assert!(indeterminate_html.contains(r#"data-state="indeterminate""#));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &unchecked(),
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_label_children_are_escaped_on_render() {
        let html = render(&label(
            &unchecked(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_hidden_input_name_value_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&hidden_input(&unchecked(), PAYLOAD, PAYLOAD, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_checkbox_state_machine() {
        // `Checkbox` は本モジュールから再エクスポートしない（本モジュール冒頭の
        // rustdoc「`Checkbox` 型を再エクスポートしない理由」参照）ため、
        // headless-ui から直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_headless_ui::checkbox::Checkbox;

        let mut cb = Checkbox::default();
        assert!(!cb.is_checked());

        let ssr_html = render(&cb.root(CheckboxFlags::default(), vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="unchecked""#));

        assert!(dispatch(&mut cb, "toggle", ""));
        let hydrate_html = render(&render_for_hydration(&cb));
        assert!(hydrate_html.contains(r#"data-hydrate-checked="checked""#));

        let restored = Checkbox::from_hydration_attrs(&cb.hydration_attrs()).unwrap();
        assert_eq!(restored, cb);
    }
}

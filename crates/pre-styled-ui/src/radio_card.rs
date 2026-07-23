//! styled RadioCard（イシュー #747、親 #520/#545、Phase 3 親 #736）。
//!
//! chakra-ui `forms/radio-card` 相当のカード型選択 UI。ark-ui には
//! radio-card 相当の anatomy は存在せず、chakra-ui が headless の
//! RadioGroup 状態機械の上に独自 slot recipe として実装している構図を、本
//! クレートでもそのまま踏襲する。**headless-ui には手を入れない**（受け入れ
//! 条件）。設計判断・セキュリティ不変条件は [`crate::checkbox_card`]
//! rustdoc と同型（本モジュールは RadioCard 版）。
//!
//! # anatomy は pre-styled 層で新規定義する（[`crate::card`]/[`crate::checkbox_card`] 先例準拠）
//!
//! `fandhe_frontend_headless_ui::radio_group`（イシュー #536/#595）の 6
//! anatomy パーツは `data-scope="radio-group"` に固定されており、カード型の
//! 10 パーツ構成へそのまま拡張できない。本モジュールは新規 anatomy
//! `data-scope="radio-card"` を [`fandhe_frontend_headless_ui::anatomy`] で
//! 定義する。既存 `radio-group` scope とは完全に独立するため、
//! [`crate::radio_group`] の CSS/属性契約と衝突しない。
//!
//! # 状態機械の再利用（受け入れ条件 1: 新規状態機械を作らない）
//!
//! [`fandhe_frontend_headless_ui::radio_group::RadioGroup`]（single-select
//! 状態機械、イシュー #524/#536）をそのまま利用する。本モジュールから
//! 再エクスポートしない（[`crate::radio_group`] は inherent `root()` を
//! 持たないため再エクスポートしても実害がないが、本モジュールは独自
//! anatomy を持ち `RadioGroup` に `item`/`item_control` 等の card 版
//! inherent メソッドは存在しないため、呼び出し側は
//! `fandhe_frontend_headless_ui::radio_group::RadioGroup` を直接 import し、
//! `.is_checked(value)` 等の問い合わせ結果を本モジュールのパーツ関数へ渡す
//! 契約とする）。
//!
//! # anatomy パーツ構成（chakra-ui slot 準拠、10 パーツ）
//!
//! - [`root`][]: `<div role="radiogroup">`。グループ全体、`size`/`palette`
//!   クラスを付与する唯一のパーツ。`orientation`/`labelled_by` は
//!   [`fandhe_frontend_headless_ui::radio_group::root`] と同じ任意入力。
//! - [`label`][]: `<span>`。グループ全体の見出し（[`crate::radio_group::label`]
//!   と同型、`<label>` ではなく `<span>` を採用する理由は headless 側
//!   モジュール doc 参照）。
//! - [`item`][]: `<label>`。選択肢 1 個のカード本体。
//! - [`item_control`][]: `<div>`。indicator と content を横に並べる領域。
//! - [`item_content`][]: `<div>`。text/description/addon を縦に積むコンテナ。
//! - [`item_text`][]: `<div>`。見出しテキスト。
//! - [`item_description`][]: `<div>`。補足テキスト。
//! - [`item_addon`][]: `<div>`。任意の付加コンテンツ（アイコン等）。
//! - [`item_indicator`][]: `<span>`。ラジオ円（[`crate::radio_group::item_control`]
//!   相当の box-shadow inset ドット描画）。headless radio_group の
//!   `item-control`（＝ラジオ円）と名前が同じだが、本モジュールの
//!   `item_indicator` は別 anatomy（`data-scope="radio-card"`）のパーツで
//!   あり衝突しない。
//! - [`item_hidden_input`][]: `<input type="radio">`。フォーム送信・
//!   キーボード操作・グループ内排他選択の実体（`crates/headless-ui/src/radio_group.rs`
//!   の `item_hidden_input` と同一属性契約、両ファイルを合わせて確認する契約）。
//!
//! # `item_hidden_input` の視覚的非表示化（[`crate::radio_group`] と同じ責務分担）
//!
//! headless 層は視覚的な非表示化を行わない契約（`crates/headless-ui/src/radio_group.rs`
//! 参照）のため、styled 層である本モジュールが visually-hidden パターン
//! （[`crate::radio_group::item_hidden_input`] の CSS と同一の 9 宣言）で
//! 覆い隠し、`item_indicator` をカスタムラジオ円として描画する。
//!
//! # フォーカスリング（本イシューのスコープ、§ out-of-scope 参照）
//!
//! 実フォーカスは [`item_hidden_input`] が受けるため、[`crate::radio_group`]
//! の `item` と同型の [`StateCondition::FocusWithin`]（no-JS フォールバック）
//! のみを [`item`] へ登録する。`data-focus-visible`（wasm 配線によるキー
//! ボード操作専用リング）は `crates/wasm-full/src/focus_visible.rs` の
//! `(scope, part)` マッピングに `"radio-card"` が未登録のため本イシューでは
//! 実装しない（フォローアップ、PR 本文参照）。
//!
//! # `size`/`palette` variant
//!
//! [`crate::radio_group`] rustdoc「複合部品の variant 統一方針」節（#708）と
//! 同型。`size`（[`Size`]）は [`root`] へのみクラスを付与し、[`recipe`] が
//! 登録する `--fandhe-radio-card-*` の root スコープ custom property 経由で
//! `item`/`item_indicator`/`item_text` の寸法を切り替える。`palette`
//! （[`ColorPalette`]）は [`crate::recipe::palette_declarations`] を `root`
//! へ登録し、選択済みカードの枠線・背景・ドット色を
//! `var(--fandhe-palette, ...)` 経由で切り替える。
//!
//! # セキュリティ不変条件
//!
//! `raw_html()` は使用しない。CSS 宣言値はすべて静的リテラルで、動的値
//! （`value`/`name`/attrs/children）は
//! [`fandhe_frontend_headless_ui::fandhe_frontend_core::render`] の既定
//! エスケープを必ず経由する（REQ-1）。呼び出し側 `attrs` の `class` は
//! [`drop_class_attr`] で除去してから合成し、`class` 属性は常に単一
//! （[`crate::radio_group::root`] と同型）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `fandhe-frontend-wasm-full` の focus/クリック配線（`(scope, part)` を
//!   `("radio-card", "item-hidden-input") -> "item"` へ写像し
//!   `data-focus-visible` を CSS で伝える対応、headless 配線の select
//!   アクション写像の card scope 対応）。
//! - `examples/headless-pre-styled-ui` への追随（pre-styled-ui 公開後に
//!   別 PR で対応）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_declarations, ColorPalette, Size, SlotRecipe, StateCondition, VariantValue,
};
use fandhe_frontend_headless_ui::aria::{aria_labelledby, aria_orientation, role};
pub use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::data_attrs::{data_disabled, data_orientation, data_state};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="radio-card"` を固定した本コンポーネントの anatomy
/// （既存 `data-scope="radio-group"` とは独立、モジュール冒頭 rustdoc 参照）。
const ANATOMY: Anatomy = anatomy("radio-card");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "item",
    "item-control",
    "item-content",
    "item-text",
    "item-description",
    "item-addon",
    "item-indicator",
    "item-hidden-input",
];

/// この styled RadioCard の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("radio-card", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "horizontal"),
            vec![decl("flex-direction", "row")],
        )
        .base(
            "label",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "flex"),
                decl("align-items", "flex-start"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("cursor", "pointer"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
                decl("padding", "var(--fandhe-radio-card-padding, 0.75rem)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("transition", "border-color 0.15s, box-shadow 0.15s"),
            ],
        )
        .state(
            "item",
            StateCondition::AttrEq("data-state", "checked"),
            vec![
                decl(
                    "border-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "box-shadow",
                    "0 0 0 1px var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
            ],
        )
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        // イシュー #747: 実フォーカスは item-hidden-input が受けるため、
        // 祖先 item（`<label>`）へ `:focus-within` で no-JS フォールバックの
        // リングを反映する（[`crate::radio_group`] の `item` と同型）。
        .state(
            "item",
            StateCondition::FocusWithin,
            vec![
                decl(
                    "outline",
                    "2px solid var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("outline-offset", "2px"),
            ],
        )
        .base(
            "item-control",
            vec![
                decl("display", "flex"),
                decl("align-items", "flex-start"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("flex", "1"),
            ],
        )
        .base(
            "item-content",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("flex", "1"),
            ],
        )
        .base(
            "item-text",
            vec![
                decl(
                    "font-size",
                    "var(--fandhe-radio-card-label-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .base(
            "item-description",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base("item-addon", vec![decl("display", "flex")])
        .base(
            "item-indicator",
            vec![
                decl("display", "inline-flex"),
                decl("width", "var(--fandhe-radio-card-control-size, 1rem)"),
                decl("height", "var(--fandhe-radio-card-control-size, 1rem)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "50%"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("flex-shrink", "0"),
            ],
        )
        .state(
            "item-indicator",
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
                decl(
                    "box-shadow",
                    "inset 0 0 0 var(--fandhe-radio-card-dot-inset, 3px) var(--fandhe-color-bg)",
                ),
            ],
        )
        // item-hidden-input の視覚的非表示化（[`crate::radio_group::item_hidden_input`]
        // と同一の visually-hidden パターン）。
        .base(
            "item-hidden-input",
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
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-radio-card-padding", "0.5rem"),
                decl("--fandhe-radio-card-control-size", "0.85rem"),
                decl("--fandhe-radio-card-dot-inset", "2px"),
                decl(
                    "--fandhe-radio-card-label-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-radio-card-padding", "0.75rem"),
                decl("--fandhe-radio-card-control-size", "1rem"),
                decl("--fandhe-radio-card-dot-inset", "3px"),
                decl(
                    "--fandhe-radio-card-label-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-radio-card-padding", "1rem"),
                decl("--fandhe-radio-card-control-size", "1.25rem"),
                decl("--fandhe-radio-card-dot-inset", "4px"),
                decl(
                    "--fandhe-radio-card-label-font-size",
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

/// この styled RadioCard が生成する静的 CSS 全量を返す（決定的。
/// [`crate::radio_group::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去
/// してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::radio_group::root`] と同じ属性契約
/// （`role="radiogroup"`/`orientation`/`labelled_by`）を独自 anatomy 上で
/// 組み立てる。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::radio_card;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = radio_card::root(
///     Size::Md,
///     ColorPalette::Accent,
///     false,
///     None,
///     None,
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"data-scope="radio-card" data-part="root""#));
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
    let mut merged: Vec<(&str, &str)> = vec![role("radiogroup"), ("class", class.as_str())];
    if let Some(orientation) = orientation {
        merged.push(aria_orientation(orientation));
        merged.push(data_orientation(orientation));
    }
    if let Some(id) = labelled_by {
        merged.push(aria_labelledby(id));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "div", merged, children)
}

/// label パーツ（`<span>`）。グループ全体の見出し
/// （[`crate::radio_group::label`] と同型）。
#[must_use]
pub fn label<'a>(id: Option<&'a str>, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("label", "span", merged, children)
}

/// item パーツ（`<label>`）。選択肢 1 個のカード本体。
#[must_use]
pub fn item<'a>(
    checked: bool,
    disabled: bool,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        data_state(if checked { "checked" } else { "unchecked" }),
        ("data-value", value),
    ];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item", "label", merged, children)
}

/// item-control パーツ（`<div>`）。indicator と content を横に並べる領域。
#[must_use]
pub fn item_control<'a>(
    checked: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![data_state(if checked { "checked" } else { "unchecked" })];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item-control", "div", merged, children)
}

/// item-content パーツ（`<div>`）。item-text/item-description/item-addon を
/// 縦に積むコンテナ。
#[must_use]
pub fn item_content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-content", "div", attrs, children)
}

/// item-text パーツ（`<div>`）。選択肢の見出しテキスト。
#[must_use]
pub fn item_text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-text", "div", attrs, children)
}

/// item-description パーツ（`<div>`）。選択肢の補足テキスト。
#[must_use]
pub fn item_description<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-description", "div", attrs, children)
}

/// item-addon パーツ（`<div>`）。任意の付加コンテンツ（アイコン等）。
#[must_use]
pub fn item_addon<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-addon", "div", attrs, children)
}

/// item-indicator パーツ（`<span>`）。ラジオ円
/// （[`crate::radio_group::item_control`] 相当の box-shadow inset ドット
/// 描画。名前は headless `radio_group::item_control` と異なるが、混同を
/// 避けるため本モジュールでは `item_indicator` と命名する）。
#[must_use]
pub fn item_indicator<'a>(checked: bool, disabled: bool, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![data_state(if checked { "checked" } else { "unchecked" })];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item-indicator", "span", merged, vec![])
}

/// item-hidden-input パーツ（`<input type="radio">`）。フォーム送信・
/// キーボード操作・グループ内排他選択の実体
/// （`crates/headless-ui/src/radio_group.rs::item_hidden_input` と同一属性
/// 契約、両ファイルを合わせて確認する契約、モジュール rustdoc 参照）。
#[must_use]
pub fn item_hidden_input<'a>(
    checked: bool,
    disabled: bool,
    name: Option<&'a str>,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "radio"),
        ("value", value),
        data_state(if checked { "checked" } else { "unchecked" }),
    ];
    if let Some(name) = name {
        merged.push(("name", name));
    }
    if checked {
        merged.push(("checked", ""));
    }
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("item-hidden-input", "input", merged, vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_headless_ui::radio_group::RadioGroup;
    use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="radio-card"][data-part="item-indicator"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn item_hidden_input_is_visually_hidden() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="radio-card"][data-part="item-hidden-input"]"#));
        assert!(css.contains("clip: rect(0, 0, 0, 0);"));
        assert!(css.contains("position: absolute;"));
    }

    #[test]
    fn stylesheet_links_data_state_checked_to_item_style() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="radio-card"][data-part="item"][data-state="checked"]"#)
        );
        assert!(css.contains("border-color: var(--fandhe-palette, var(--fandhe-color-accent));"));
    }

    #[test]
    fn stylesheet_links_data_state_checked_to_item_indicator_style() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="radio-card"][data-part="item-indicator"][data-state="checked"]"#
        ));
    }

    #[test]
    fn root_switches_to_row_layout_on_horizontal_orientation() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="radio-card"][data-part="root"][data-orientation="horizontal"]"#
        ));
        assert!(css.contains("flex-direction: row;"));
    }

    #[test]
    fn disabled_item_gets_not_allowed_cursor() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="radio-card"][data-part="item"][data-disabled]"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn item_focus_within_gets_accent_outline_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="radio-card"][data-part="item"]:focus-within {"#));
    }

    // --- variant クラス ---

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
        assert!(html.contains(r#"data-scope="radio-card""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="radiogroup""#));
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
        assert!(html.contains("fd-radio-card--size-md"));
        assert!(html.contains("fd-radio-card--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-radio-card--size-sm"),
            (Size::Md, "fd-radio-card--size-md"),
            (Size::Lg, "fd-radio-card--size-lg"),
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
            (ColorPalette::Accent, "fd-radio-card--color-palette-accent"),
            (ColorPalette::Info, "fd-radio-card--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-radio-card--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-radio-card--color-palette-warning",
            ),
            (ColorPalette::Danger, "fd-radio-card--color-palette-danger"),
        ] {
            let html = render(&root(Size::Md, palette, false, None, None, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn reexported_root_with_horizontal_orientation_emits_data_orientation() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            Some(Orientation::Horizontal),
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-orientation="horizontal""#));
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
        assert!(html.contains(r#"data-scope="radio-card""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn xss_payload_in_item_value_is_escaped_by_render() {
        let payload = "\"><script>alert(1)</script>";
        let html = render(&item(false, false, payload, vec![], vec![text(payload)]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn xss_payload_in_item_text_children_is_escaped_by_render() {
        let payload = "\"><img src=x onerror=alert(1)>";
        let html = render(&item_text(vec![], vec![text(payload)]));
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn item_hidden_input_name_and_value_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&item_hidden_input(
            false,
            false,
            Some(PAYLOAD),
            PAYLOAD,
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    // --- SSR/hydration 往復（受け入れ条件 1: headless RadioGroup 状態機械を再利用） ---

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_radio_group_state_machine() {
        let mut g = RadioGroup::default();
        assert_eq!(g.value(), None);

        assert!(dispatch(&mut g, "select", "red"));
        assert_eq!(g.value(), Some("red"));

        let ssr_html = render(&item_control(g.is_checked("red"), false, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="checked""#));

        let hydrate_html = render(&render_for_hydration(&g));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = RadioGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored.value(), Some("red"));
    }
}

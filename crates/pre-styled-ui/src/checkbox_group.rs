//! styled CheckboxGroup（headless ラッパー、イシュー #997、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::checkbox_group`（イシュー #997）の
//! Label / Item / ItemControl / ItemIndicator / ItemText 5 anatomy パーツと
//! [`fandhe_frontend_headless_ui::checkbox_group::CheckboxGroup`] 状態機械を
//! そのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::radio_group`] の rustdoc と同じ
//! 方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::radio_group::root`] と同型）を本モジュールで再定義する。
//! headless 自由関数 `root` と名前衝突するため、`pub use ...::*` ではなく
//! 必要な識別子（[`label`]/[`item`]/[`item_control`]/[`item_indicator`]/
//! [`item_text`]/[`CheckboxGroup`]/[`DATA_STATE_CHECKED`]/
//! [`DATA_STATE_UNCHECKED`]）のみを選択的に再エクスポートする。
//!
//! [`CheckboxGroup`] 状態機械は inherent `root()` を持たない（item 系
//! メソッドのみ、`crates/headless-ui/src/checkbox_group.rs` 参照）ため、
//! そのまま再エクスポートしても未スタイル `root` の静かな適用漏れは
//! 発生しない（[`crate::radio_group`] の `RadioGroup` 非対称処理と同じ判断）。
//!
//! # `item-hidden-input` を本モジュールが持たない理由（`checkbox::stylesheet()` 併用が必須）
//!
//! headless 層（`crates/headless-ui/src/checkbox_group.rs`）は
//! `item-hidden-input` パーツを新設せず、ネイティブ `<input type="checkbox">`
//! を [`fandhe_frontend_headless_ui::checkbox::hidden_input`] の入れ子
//! 再利用で賄う（headless 側モジュール doc「anatomy」節参照）。この設計を
//! 継承し、**本モジュールの [`recipe`] は `hidden-input` slot の
//! visually-hidden 規則を一切再宣言しない**
//! （`[data-scope="checkbox"][data-part="hidden-input"]` として
//! `crate::checkbox` の recipe に既存であり、本モジュールで重複実装すると
//! `checkbox` recipe とのドリフト・二重管理を招く）。styled CheckboxGroup を
//! 使う呼び出し側は、本モジュールの [`stylesheet`] に加えて
//! `fandhe_frontend_pre_styled_ui::checkbox::stylesheet()`
//! も併せて読み込む必要がある（`crates/docs-site/src/showcase.rs` が
//! 両方を `push_css` する実例を参照）。
//!
//! # data-state とスタイルの連動
//!
//! `item`/`item-control`/`item-indicator`（選択状態、
//! `data-state="checked"`/`"unchecked"`）の見た目の切り替えを [`recipe`] へ
//! 登録する（[`crate::recipe::SlotRecipe::state`]）。`root` の
//! `data-orientation="horizontal"` でも配置切り替えを行う（[`crate::radio_group`]
//! と同型）。
//!
//! # `:focus-within` によるフォーカスリング
//!
//! 実フォーカスは（呼び出し側が入れ子にする）
//! `fandhe_frontend_pre_styled_ui::checkbox::hidden_input` が受ける。
//! [`crate::radio_group`] と同じ理由（`item`〔`<label>`〕が hidden input の
//! 祖先であること）により、`item` へ `:focus-within` のフォーカスリングを
//! 登録する。
//!
//! # `size`/`palette` variant
//!
//! [`crate::radio_group`] rustdoc「`size`/`palette` variant」節と同じ設計
//! （`root` スコープの custom property 経由で `item-control`/`item-text` の
//! 寸法・見た目を切り替える）に従う。`size` は
//! `--fandhe-checkbox-group-control-size`/`-font-size` を、`palette` は
//! [`crate::recipe::palette_scale_declarations`] を登録する。`var()` にはいずれも
//! Md サイズ・Accent パレット相当のフォールバック値を書き、styled `root` を
//! 経由しない headless 直接利用マークアップでも現行外観を維持する
//! （fail-safe）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層の再エクスポートと静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（`value`/属性/children）へ CSS 値として流し込む経路を持たない
//! （動的値は headless 層経由で `fandhe_frontend_core::render` の既定
//! エスケープを必ず通る、REQ-1）。styled `root` は [`drop_class_attr`] に
//! より呼び出し側の `class` を除去してから合成するため、`class` 属性は常に
//! 単一（[`crate::radio_group::root`] と同型）。
//!
//! # 本イシューのスコープ外
//!
//! headless 層モジュール doc「out-of-scope」節（キーボードナビゲーション・
//! 実 DOM 配線・全選択/一部選択集約・Field 連携・`checkbox_card` を item
//! として使う構成）をそのまま継承する。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_scale_declarations, ColorPalette, Size, SlotRecipe, StateCondition, VariantValue,
};

// headless 自由関数 `root` はあえて再エクスポートしない（本モジュール冒頭
// の rustdoc「選択的 re-export」節参照、`root` は本モジュールで styled 版
// として再定義する）。
pub use fandhe_frontend_headless_ui::checkbox_group::{
    item, item_control, item_indicator, item_text, label, CheckboxGroup, DATA_STATE_CHECKED,
    DATA_STATE_UNCHECKED,
};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// headless `checkbox_group` anatomy の `data-part` 一覧（`crates/headless-ui/src/checkbox_group.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。`item-hidden-input` を
/// 含まない理由はモジュール doc「`item-hidden-input` を本モジュールが
/// 持たない理由」節参照）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "item",
    "item-control",
    "item-indicator",
    "item-text",
];

/// この styled CheckboxGroup の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("checkbox-group", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
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
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("cursor", "pointer"),
            ],
        )
        .base(
            "item-control",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("width", "var(--fandhe-checkbox-group-control-size, 1rem)"),
                decl("height", "var(--fandhe-checkbox-group-control-size, 1rem)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("flex-shrink", "0"),
                decl("transition", "background 0.15s, border-color 0.15s"),
            ],
        )
        .base(
            "item-text",
            vec![
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "font-size",
                    "var(--fandhe-checkbox-group-font-size, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        // イシュー #997: indicator の base に `display` 宣言を置かない
        // （headless 層が `data-state="unchecked"` 時に `hidden` 存在属性を
        // 付与する規約と衝突させないため。`crates/pre-styled-ui/src/checkbox.rs`
        // 「`indicator` の `hidden` 属性意味論を CSS が壊さない設計」節と
        // 同型の判断）。
        .base(
            "item-indicator",
            vec![
                // イシュー #997 Bugbot 指摘（Medium）回帰固定: 固定寸法ではなく
                // `--fandhe-checkbox-group-check-width`/`-check-height`
                // custom property（`root` の size variant が切り替える）を
                // 参照する。`crates/pre-styled-ui/src/checkbox.rs` の
                // `indicator`（`--fandhe-checkbox-check-width`/`-check-height`）
                // と同型。
                decl("width", "var(--fandhe-checkbox-group-check-width, 0.25rem)"),
                decl(
                    "height",
                    "var(--fandhe-checkbox-group-check-height, 0.5rem)",
                ),
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
        // `root` の `data-orientation="horizontal"`（headless 層が
        // `data_orientation` 経由で出力）では縦積みではなく横並びへ切り替える。
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "horizontal"),
            vec![decl("flex-direction", "row")],
        )
        // 選択済み item-control の見た目（角丸の四角、palette 色の塗り。
        // ラジオの円形〔`border-radius: 50%`〕ではない）。
        .state(
            "item-control",
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
        // `data-disabled`（headless 層が `data_disabled` 経由で `item`/
        // `item-control`/`item-indicator`/`item-text` へ出力）時の操作不能な
        // 見た目。
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        // 実フォーカスは呼び出し側が入れ子にする checkbox::hidden_input が
        // 受ける（visually-hidden 化されている）ため、祖先 `item`
        // （`<label>`）へ `:focus-within` で反映する（[`crate::radio_group`]
        // と同型のフォールバック、モジュール doc 参照）。
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
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-checkbox-group-control-size", "0.7rem"),
                decl("--fandhe-checkbox-group-check-width", "0.15rem"),
                decl("--fandhe-checkbox-group-check-height", "0.3rem"),
                decl(
                    "--fandhe-checkbox-group-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-checkbox-group-control-size", "0.85rem"),
                decl("--fandhe-checkbox-group-check-width", "0.2rem"),
                decl("--fandhe-checkbox-group-check-height", "0.4rem"),
                decl(
                    "--fandhe-checkbox-group-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-checkbox-group-control-size", "1rem"),
                decl("--fandhe-checkbox-group-check-width", "0.25rem"),
                decl("--fandhe-checkbox-group-check-height", "0.5rem"),
                decl(
                    "--fandhe-checkbox-group-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-checkbox-group-control-size", "1.25rem"),
                decl("--fandhe-checkbox-group-check-width", "0.3rem"),
                decl("--fandhe-checkbox-group-check-height", "0.6rem"),
                decl(
                    "--fandhe-checkbox-group-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-checkbox-group-control-size", "1.5rem"),
                decl("--fandhe-checkbox-group-check-width", "0.35rem"),
                decl("--fandhe-checkbox-group-check-height", "0.7rem"),
                decl(
                    "--fandhe-checkbox-group-font-size",
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

/// この styled CheckboxGroup が生成する静的 CSS 全量を返す（決定的。
/// [`crate::radio_group::stylesheet`] と同じ契約）。CheckboxGroup を実際に
/// 利用する際は `crate::checkbox::stylesheet()` も併せて読み込む必要がある
/// （モジュール doc「`item-hidden-input` を本モジュールが持たない理由」
/// 節参照）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去
/// してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::checkbox_group::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::checkbox_group;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = checkbox_group::root(
///     Size::Md,
///     ColorPalette::Accent,
///     false,
///     None,
///     None,
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"data-scope="checkbox-group" data-part="root""#));
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
    fandhe_frontend_headless_ui::checkbox_group::root(
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
        assert!(a.contains(r#"[data-scope="checkbox-group"][data-part="item-control"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_never_duplicates_checkbox_hidden_input_rules() {
        // §設計判断（モジュール doc 参照）: visually-hidden の 9 宣言は
        // `crate::checkbox` recipe の `hidden-input` slot にのみ存在し、
        // 本 stylesheet では再宣言しない（重複実装の回帰固定）。
        let css = stylesheet();
        assert!(!css.contains("hidden-input"));
        assert!(!css.contains("clip: rect(0, 0, 0, 0);"));
    }

    #[test]
    fn stylesheet_links_data_state_checked_to_item_control_style() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="checkbox-group"][data-part="item-control"][data-state="checked"]"#
        ));
        assert!(css.contains("border-color: var(--fandhe-palette, var(--fandhe-color-accent));"));
        // ラジオの円形ではなく角丸の四角（border-radius: 50% を含まない）。
        assert!(!css.contains("border-radius: 50%;"));
    }

    #[test]
    fn indicator_base_has_no_display_declaration() {
        // headless 層の `hidden` 存在属性の意味論を壊さないため
        // （`crate::checkbox` の同名テストと同型の回帰固定）。
        let css = stylesheet();
        let scope = r#"[data-scope="checkbox-group"][data-part="item-indicator"] {"#;
        let start = css.find(scope).expect("item-indicator base block missing");
        let end = css[start..]
            .find('}')
            .map(|i| start + i)
            .expect("closing brace missing");
        let block = &css[start..end];
        assert!(!block.contains("display:"));
    }

    #[test]
    fn root_switches_to_row_layout_on_horizontal_orientation() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="checkbox-group"][data-part="root"][data-orientation="horizontal"]"#
        ));
        assert!(css.contains("flex-direction: row;"));
    }

    #[test]
    fn disabled_item_gets_not_allowed_cursor() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="checkbox-group"][data-part="item"][data-disabled]"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn item_focus_within_gets_accent_outline_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="checkbox-group"][data-part="item"]:focus-within {"#));
        assert!(
            css.contains("outline: 2px solid var(--fandhe-palette, var(--fandhe-color-accent));")
        );
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
        assert!(html.contains(r#"data-scope="checkbox-group""#));
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
        assert!(html.contains("fd-checkbox-group--size-md"));
        assert!(html.contains("fd-checkbox-group--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-checkbox-group--size-xs"),
            (Size::Sm, "fd-checkbox-group--size-sm"),
            (Size::Md, "fd-checkbox-group--size-md"),
            (Size::Lg, "fd-checkbox-group--size-lg"),
            (Size::Xl, "fd-checkbox-group--size-xl"),
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
                "fd-checkbox-group--color-palette-accent",
            ),
            (ColorPalette::Info, "fd-checkbox-group--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-checkbox-group--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-checkbox-group--color-palette-warning",
            ),
            (
                ColorPalette::Danger,
                "fd-checkbox-group--color-palette-danger",
            ),
            (
                ColorPalette::Neutral,
                "fd-checkbox-group--color-palette-neutral",
            ),
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
    fn stylesheet_contains_size_and_palette_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--color-palette-"));
        assert!(css.contains("--fandhe-checkbox-group-control-size"));
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
        assert!(html.contains(r#"data-scope="checkbox-group""#));
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
        let html = render(&item_text(false, false, vec![], vec![text(payload)]));
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_checkbox_group_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut g = CheckboxGroup::default();
        assert_eq!(g.selected(), &[] as &[String]);

        assert!(dispatch(&mut g, "select", "red"));
        assert!(g.is_checked("red"));

        let ssr_html = render(&g.item_control("red", false, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="checked""#));

        let hydrate_html = render(&render_for_hydration(&g));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = CheckboxGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert!(restored.is_checked("red"));
    }

    #[test]
    fn item_indicator_hidden_state_semantics_preserved_through_reexport() {
        let unchecked = render(&item_indicator(false, false, vec![], vec![]));
        assert!(unchecked.contains(r#"hidden="""#));

        let checked = render(&item_indicator(true, false, vec![], vec![]));
        assert!(!checked.contains(r#"hidden="""#));
    }

    #[test]
    fn data_state_constants_reexported() {
        assert_eq!(DATA_STATE_CHECKED, "checked");
        assert_eq!(DATA_STATE_UNCHECKED, "unchecked");
    }
}

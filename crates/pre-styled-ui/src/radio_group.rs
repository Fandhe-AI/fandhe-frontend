//! styled RadioGroup（headless ラッパー、イシュー #683、`size`/`palette`
//! variant 拡張はイシュー #708、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::radio_group`（イシュー #558/#536）の
//! Label / Item / ItemControl / ItemText / ItemHiddenInput 5 anatomy パーツと
//! [`fandhe_frontend_headless_ui::radio_group::RadioGroup`] 状態機械を
//! そのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::select`] の rustdoc と同じ方針に
//! 従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、イシュー #708）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::avatar::root`]・[`crate::card::root`] と同型）を本モジュールで
//! 再定義する。headless 自由関数 `root` と名前衝突するため、
//! `pub use ...::*` ではなく必要な識別子（[`label`]/[`item`]/
//! [`item_control`]/[`item_text`]/[`item_hidden_input`]/[`RadioGroup`]）のみ
//! を選択的に再エクスポートする。
//!
//! [`RadioGroup`] 状態機械は inherent `root()` を持たない（item 系メソッド
//! のみ、`crates/headless-ui/src/radio_group.rs` 参照）ため、[`crate::avatar`]
//! の `Avatar` 非再エクスポートと異なり、そのまま再エクスポートしても未
//! スタイル root の静かな適用漏れは発生しない（型を経由して `size`/
//! `palette` 抜きの `root` を誤って呼んでしまう経路がない）。
//!
//! # item-hidden-input の視覚的非表示化（[`crate::select`] の hidden-select
//! と同じ責務分担）
//!
//! headless 層（`crates/headless-ui/src/radio_group.rs`）はネイティブ
//! `<input type="radio">` に `type`/`value`/`name`/`checked`/`disabled`/
//! `data-state` のみを設定し、視覚的な非表示化は行わない契約になっている。
//! styled 層である本モジュールが visually-hidden パターン（`position:
//! absolute` + 1px クリップ、[`crate::select`] の `hidden-select` 規則と
//! 同一の 9 宣言）で覆い隠し、`item-control` をカスタムラジオ円として描画
//! する。フォーム送信・キーボード操作・グループ内排他選択はネイティブ
//! semantics のまま維持される（headless 側モジュール doc 参照）。
//!
//! # data-state とスタイルの連動
//!
//! `item`/`item-control`（選択状態、`data-state="checked"`/`"unchecked"`）の
//! 見た目の切り替えを [`recipe`] へ登録する（[`crate::recipe::SlotRecipe::state`]）。
//! `root` の `data-orientation="horizontal"` でも配置切り替えを行う。
//!
//! # `:focus-within` によるフォーカスリング（イシュー #683）
//!
//! `item-hidden-input` を視覚的に隠すと、ネイティブのフォーカスリングも
//! 見えなくなる。実フォーカスは隠された `<input>` にあり、`item`
//! （`<label>`、input の祖先）へ `:focus-within` を当てるのが CSS 的に成立
//! する唯一の経路（[`crate::recipe::StateCondition`] は `Attr`/`AttrEq`/
//! `FocusVisible` のみで兄弟・子孫セレクタを持たなかったため、本イシューで
//! [`crate::recipe::StateCondition::FocusWithin`] を追加した）。
//!
//! # `data-focus-visible` によるキーボード専用フォーカスリング（イシュー #709）
//!
//! 上記 `:focus-within` は「input にフォーカスがある」ことのみを条件とし、
//! マウスクリックによるフォーカスでも発火する（chakra-ui/ark-ui が区別する
//! キーボード操作専用の `:focus-visible` 意味論とは異なる、包括的な
//! フォールバック）。これを補完するため、headless 層
//! （`fandhe_frontend_headless_ui::data_attrs::data_focus_visible`、
//! `crates/headless-ui/src/radio_group.rs` のフォーカスリング契約 doc
//! 参照）が出力し `fandhe-frontend-wasm-full` の focus 配線が `item`/
//! `item-control` へ付け外しする `data-focus-visible` を `item-control`
//! slot の状態規則として追加する。役割分担: `:focus-within`（`item`） =
//! wasm なしでも成立する no-JS フォールバック / `data-focus-visible`
//! （`item-control`） = wasm 配線時のみ有効なキーボード専用リング。両者は
//! 独立した条件として共存し、どちらか一方が成立すればリングが表示される。
//!
//! # `size`/`palette` variant（イシュー #708）
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-radio-group-control-size`/`-dot-inset`/`-font-size` の root
//! スコープ custom property（CSS の通常のプロパティ継承により `item`/
//! `item-control`/`item-text` へ伝わる。`root` はこれらのパーツを内包する
//! 祖先要素であるため、[`crate::recipe::SlotRecipe`] へ子孫セレクタ機構を
//! 追加せずに実現できる）経由で `item-control` の寸法・選択ドットの見た目を
//! 切り替える。`palette`（[`ColorPalette`]）は既存の
//! [`crate::recipe::palette_scale_declarations`]（chakra-ui virtual token 方式、
//! #606）を `root` へ登録し、checked 時の `item-control` の枠色・背景・
//! `:focus-within` のアウトライン色を `var(--fandhe-palette, ...)` 経由で
//! 切り替える。`base`/`state` 規則の `var()` にはいずれも Md サイズ・
//! Accent パレット相当のフォールバック値を書き、styled `root` を経由しない
//! headless 直接利用マークアップでも現行外観を維持する（fail-safe、
//! `crate::lib` rustdoc「複合部品の variant 統一方針」節参照）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層の再エクスポートと静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（`value`/`name`/属性/children）へ CSS 値として流し込む経路
//! を持たない（動的値は headless 層経由で `fandhe_frontend_core::render` の
//! 既定エスケープを必ず通る、REQ-1）。styled `root` は [`drop_class_attr`]
//! により呼び出し側の `class` を除去してから合成するため、`class` 属性は
//! 常に単一（[`crate::avatar::root`] と同型）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - tabs/accordion/dialog/menu/select への size（および tabs への
//!   palette）展開は本イシューの方針を第 2 弾として別途適用する
//!   （`docs/api/pre-styled-ui-api.md` の variant 表参照）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_scale_declarations, ColorPalette, Size, SlotRecipe, StateCondition, VariantValue,
};

// headless 自由関数 `root` はあえて再エクスポートしない（本モジュール冒頭
// の rustdoc「選択的 re-export」節参照、`root` は本モジュールで styled 版
// として再定義する）。
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::radio_group::{
    item, item_control, item_hidden_input, item_text, label, RadioGroup, DATA_STATE_CHECKED,
    DATA_STATE_UNCHECKED,
};

/// headless `radio_group` anatomy の `data-part` 一覧（`crates/headless-ui/src/radio_group.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "item",
    "item-control",
    "item-text",
    "item-hidden-input",
];

/// この styled RadioGroup の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("radio-group", SLOTS)
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
                decl("width", "var(--fandhe-radio-group-control-size, 1rem)"),
                decl("height", "var(--fandhe-radio-group-control-size, 1rem)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "50%"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("flex-shrink", "0"),
            ],
        )
        .base(
            "item-text",
            vec![
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "font-size",
                    "var(--fandhe-radio-group-font-size, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
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
        // `root` の `data-orientation="horizontal"`（headless 層が
        // `data_orientation` 経由で出力、`crates/headless-ui/src/radio_group.rs`
        // 参照）では縦積みではなく横並びへ切り替える。
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "horizontal"),
            vec![decl("flex-direction", "row")],
        )
        // 選択済み項目のカスタムラジオ円の見た目（アクセントカラーの外枠 +
        // 内側ドット。`box-shadow` の inset で描く。ドットの太さは
        // `--fandhe-radio-group-dot-inset` で size ごとに切り替える）。
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
                decl(
                    "box-shadow",
                    "inset 0 0 0 var(--fandhe-radio-group-dot-inset, 3px) var(--fandhe-color-bg)",
                ),
            ],
        )
        // `data-disabled`（headless 層が `data_disabled` 経由で `item`/
        // `item-control`/`item-text`/`item-hidden-input` へ出力）時の
        // 操作不能な見た目。
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        // イシュー #683: visually-hidden 化した `item-hidden-input` へ実
        // フォーカスがあるときのフォーカスリングを、祖先 `item`
        // （モジュール rustdoc 参照）へ `:focus-within` で反映する。
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
        // イシュー #709: wasm 層が付け外しする `data-focus-visible` による
        // キーボード操作専用のフォーカスリング（`:focus-within` の no-JS
        // フォールバックとは独立に共存する。モジュール rustdoc 参照）。
        .state(
            "item-control",
            StateCondition::Attr("data-focus-visible"),
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-radio-group-control-size", "0.7rem"),
                decl("--fandhe-radio-group-dot-inset", "1px"),
                decl(
                    "--fandhe-radio-group-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-radio-group-control-size", "0.85rem"),
                decl("--fandhe-radio-group-dot-inset", "2px"),
                decl(
                    "--fandhe-radio-group-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-radio-group-control-size", "1rem"),
                decl("--fandhe-radio-group-dot-inset", "3px"),
                decl(
                    "--fandhe-radio-group-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-radio-group-control-size", "1.25rem"),
                decl("--fandhe-radio-group-dot-inset", "4px"),
                decl(
                    "--fandhe-radio-group-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-radio-group-control-size", "1.5rem"),
                decl("--fandhe-radio-group-dot-inset", "5px"),
                decl(
                    "--fandhe-radio-group-font-size",
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

/// この styled RadioGroup が生成する静的 CSS 全量を返す（決定的。
/// [`crate::select::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去
/// してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::radio_group::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::radio_group;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = radio_group::root(
///     Size::Md,
///     ColorPalette::Accent,
///     false,
///     None,
///     None,
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"data-scope="radio-group" data-part="root""#));
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
    fandhe_frontend_headless_ui::radio_group::root(
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
        assert!(a.contains(r#"[data-scope="radio-group"][data-part="item-control"]"#));
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
        assert!(css.contains(r#"[data-scope="radio-group"][data-part="item-hidden-input"]"#));
        assert!(css.contains("clip: rect(0, 0, 0, 0);"));
        assert!(css.contains("position: absolute;"));
    }

    #[test]
    fn stylesheet_links_data_state_checked_to_item_control_style() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="radio-group"][data-part="item-control"][data-state="checked"]"#
        ));
        assert!(css.contains("border-color: var(--fandhe-palette, var(--fandhe-color-accent));"));
    }

    #[test]
    fn root_switches_to_row_layout_on_horizontal_orientation() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="radio-group"][data-part="root"][data-orientation="horizontal"]"#
        ));
        assert!(css.contains("flex-direction: row;"));
    }

    #[test]
    fn disabled_item_gets_not_allowed_cursor() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="radio-group"][data-part="item"][data-disabled]"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn item_focus_within_gets_accent_outline_ring() {
        // イシュー #683 受け入れ条件: visually-hidden 化した `item-hidden-input`
        // への実フォーカスが、祖先 `item` の `:focus-within` として反映される。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="radio-group"][data-part="item"]:focus-within {"#));
        assert!(
            css.contains("outline: 2px solid var(--fandhe-palette, var(--fandhe-color-accent));")
        );
    }

    // --- variant クラス（イシュー #708） ---

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
        assert!(html.contains(r#"data-scope="radio-group""#));
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
        assert!(html.contains("fd-radio-group--size-md"));
        assert!(html.contains("fd-radio-group--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-radio-group--size-xs"),
            (Size::Sm, "fd-radio-group--size-sm"),
            (Size::Md, "fd-radio-group--size-md"),
            (Size::Lg, "fd-radio-group--size-lg"),
            (Size::Xl, "fd-radio-group--size-xl"),
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
            (ColorPalette::Accent, "fd-radio-group--color-palette-accent"),
            (ColorPalette::Info, "fd-radio-group--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-radio-group--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-radio-group--color-palette-warning",
            ),
            (ColorPalette::Danger, "fd-radio-group--color-palette-danger"),
            (
                ColorPalette::Neutral,
                "fd-radio-group--color-palette-neutral",
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
        assert!(css.contains("--fandhe-radio-group-control-size"));
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
        assert!(html.contains(r#"data-scope="radio-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn xss_payload_in_item_value_is_escaped_by_render() {
        // REQ-1 回帰: `data-value`（動的値）へ与えた XSS ペイロードが
        // `render()` の既定エスケープを経由することを固定する。
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
    fn ssr_and_hydration_round_trip_via_reexported_radio_group_state_machine() {
        // 再エクスポートされた `RadioGroup`（headless の Component/Hydrate
        // 実装をそのまま継承）経由で SSR/hydration 往復を固定する
        // （[`crate::select`] の同型テストに準拠）。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut g = RadioGroup::default();
        assert_eq!(g.value(), None);

        assert!(dispatch(&mut g, "select", "red"));
        assert_eq!(g.value(), Some("red"));

        let ssr_html = render(&g.item_control("red", false, vec![]));
        assert!(ssr_html.contains(r#"data-state="checked""#));

        let hydrate_html = render(&render_for_hydration(&g));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = RadioGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored.value(), Some("red"));
    }
}

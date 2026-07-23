//! styled RadioGroup（headless ラッパー、イシュー #683、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::radio_group`（イシュー #558/#536）の
//! Root / Label / Item / ItemControl / ItemText / ItemHiddenInput 6 anatomy
//! パーツと [`fandhe_frontend_headless_ui::radio_group::RadioGroup`]
//! 状態機械をそのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加
//! 提供する。薄い委譲の根拠・スコープ外事項は [`crate::select`] の rustdoc と
//! 同じ方針に従う（variant（size 等）ごとのクラス切り替えはスコープ外、
//! `crate` rustdoc「headless ラッパーの設計」節参照）。
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
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層の再エクスポートと静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（`value`/`name`/属性/children）へ CSS 値として流し込む経路
//! を持たない（動的値は headless 層経由で `fandhe_frontend_core::render` の
//! 既定エスケープを必ず通る、REQ-1）。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

pub use fandhe_frontend_headless_ui::radio_group::*;

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
    SlotRecipe::new("radio-group", SLOTS)
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
                decl("width", "1rem"),
                decl("height", "1rem"),
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
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
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
        // 内側ドット。`box-shadow` の inset で描く）。
        .state(
            "item-control",
            StateCondition::AttrEq("data-state", "checked"),
            vec![
                decl("border-color", "var(--fandhe-color-accent)"),
                decl("background", "var(--fandhe-color-accent)"),
                decl("box-shadow", "inset 0 0 0 3px var(--fandhe-color-bg)"),
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
                decl("outline", "2px solid var(--fandhe-color-accent)"),
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
}

/// この styled RadioGroup が生成する静的 CSS 全量を返す（決定的。
/// [`crate::select::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_headless_ui::data_attrs::Orientation;

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
        assert!(css.contains("border-color: var(--fandhe-color-accent);"));
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
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(false, None, None, vec![], vec![]));
        assert!(html.contains(r#"data-scope="radio-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="radiogroup""#));
    }

    #[test]
    fn reexported_root_with_horizontal_orientation_emits_data_orientation() {
        let html = render(&root(
            false,
            Some(Orientation::Horizontal),
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-orientation="horizontal""#));
    }

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

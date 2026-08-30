//! styled Steps（headless ラッパー、イシュー #752、親 #520/#736）。
//!
//! `fandhe_frontend_headless_ui::steps`（イシュー #752）の Root / List /
//! Item / Trigger / Indicator / Separator / Content / CompletedContent /
//! PrevTrigger / NextTrigger の 10 anatomy パーツと
//! [`fandhe_frontend_headless_ui::steps::Steps`] 状態機械へ薄く委譲し、
//! [`stylesheet`] で既定 CSS（円形 indicator・区切り線・current/complete
//! 連動色）を追加提供する。薄い委譲の根拠・スコープ外事項は
//! [`crate::slider`]/[`crate::rating_group`] の rustdoc と同じ方針に従う。
//!
//! # 全パーツが `state: &Steps` を取る理由（headless 層に自由関数がない）
//!
//! [`fandhe_frontend_headless_ui::steps`] は（[`crate::slider`] の
//! `label`/`control`/`track`/`thumb` 等と異なり）自由関数を一切持たず、
//! すべて [`fandhe_frontend_headless_ui::steps::Steps`] の inherent メソッド
//! として提供される（`data-state`（complete/current/incomplete）の判定に
//! `count`/`step` の参照が毎回必要なため）。本モジュールも同型で、
//! すべての styled パーツ関数が `state: &Steps` を受け取り、内部で
//! `state.<part>(...)` へ委譲する。
//!
//! `Steps` 状態機械自体は再エクスポートしない（[`crate::switch`] の
//! `Switch` 非再エクスポートと同じ理由）。呼び出し側が
//! `state.root(...)`/`state.item(...)` を直接呼ぶと `size`/`palette`
//! variant クラスが付与されない未スタイル描画になる事故を誘発するため、
//! 状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::steps::Steps` を直接 import し、実際の
//! 描画は本モジュールの styled パーツ関数を組み合わせて構築すること。
//!
//! # `size`/`palette` variant
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-steps-indicator-size` の root スコープ custom property
//! （通常の CSS 継承により `indicator` へ伝わる）経由で寸法を切り替える
//! （[`crate::rating_group`] と同型）。`palette`（[`ColorPalette`]）は
//! 既存の [`crate::recipe::palette_declarations`]（chakra-ui virtual token
//! 方式、#606）を `root` へ登録し、current/complete の indicator/separator
//! 色を `var(--fandhe-palette, ...)` 経由で切り替える。
//!
//! # indicator/separator の状態連動色
//!
//! `indicator` は `data-state`（`"complete"`/`"current"`/`"incomplete"`）
//! に応じて塗り色を切り替える。`separator` は `data-complete`
//! （存在属性、`crates/headless-ui/src/data_attrs.rs::data_complete`）の
//! 有無で完了色に変化する。
//!
//! # `item`/`separator` のレイアウト契約（イシュー #752 PR #797 レビュー対応）
//!
//! `separator`（`flex: 1` でステップ間の接続線を描画）が実際に伸長するには
//! 親 `item`（`li`）自身も `list` の主軸方向へ伸長する必要があるため、
//! `item` にも `flex: 1` を付与する。垂直（[`fandhe_frontend_headless_ui::steps::Orientation::Vertical`]）
//! では `item` を `flex-direction: column` に切り替え、trigger の下に
//! separator（縦の接続線）が来る配置にする。この判定は `item` 自身の
//! `data-orientation` 属性（`crates/headless-ui/src/steps.rs::Steps::item`
//! が `separator`/`list`/`root` と同様に付与、本イシューで追加）を
//! [`StateCondition::AttrEq`] で条件化して行う（[`SlotRecipe`] は
//! 対象スロット自身の属性のみを条件化でき、祖先要素の属性は参照できない
//! ため、`list`/`root` の `data-orientation` だけでは `item` の垂直
//! レイアウト切り替えができない）。
//!
//! 呼び出し側は最後の `separator` を省略するのが通常の使い方（showcase・
//! 典型的な Steps 利用パターン含む）であるため、最後の item は伸ばす対象
//! を持たない。そのため `item:last-child`（[`StateCondition::LastChild`]、
//! イシュー #752 PR #797 レビュー対応）で `flex: 1`/`min-height` を打ち
//! 消し、最終ステップの後ろに余分な空白が残らないようにする。
//!
//! # `focus-visible`（キーボードフォーカスリング）
//!
//! `trigger`/`prev-trigger`/`next-trigger` はネイティブな `<button>`
//! （実フォーカスを受ける）であるため、[`crate::switch`] のような
//! hidden-input 特有の `data-focus-visible` 対応は不要で、通常の
//! `:focus-visible` 疑似クラスを [`recipe`] へ直接登録する
//! （[`StateCondition::FocusVisible`]、[`crate::slider`] の `thumb` と同型）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層への委譲と静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（`attrs`/children）へ CSS 値として流し込む経路を持たない
//! （動的値は headless 層経由で `fandhe_frontend_core::render` の既定
//! エスケープを必ず通る、REQ-1）。styled `root` は [`drop_class_attr`] に
//! より呼び出し側の `class` を除去してから合成するため、`class` 属性は
//! 常に単一（[`crate::rating_group::root`] と同型）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく `linear`（順序強制）・`isStepValid`/
//!   `isStepSkippable`・キーボード操作/roving focus・クリックの実配線は
//!   スコープ外（`fandhe_frontend_headless_ui::steps` モジュール doc 参照）。
//! - `examples/headless-pre-styled-ui`（crates.io バージョン依存）への
//!   Steps 追加は、未公開の新バージョンを参照できないため本イシューの
//!   スコープ外とする（[`crate::rating_group`] 冒頭 rustdoc の先例どおり
//!   crates.io 公開後に追随）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_scale_declarations, ColorPalette, Size, SlotRecipe, StateCondition, VariantValue,
};

// `Steps` 状態機械はあえて再エクスポートしない（本モジュール冒頭の rustdoc
// 「全パーツが `state: &Steps` を取る理由」節参照）。状態管理・hydration が
// 必要な呼び出し側は `fandhe_frontend_headless_ui::steps::Steps` を直接
// import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::steps::Steps;
pub use fandhe_frontend_headless_ui::steps::StepsAction;

/// headless `steps` anatomy の `data-part` 一覧（`crates/headless-ui/src/steps.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "list",
    "item",
    "trigger",
    "indicator",
    "separator",
    "content",
    "completed-content",
    "prev-trigger",
    "next-trigger",
];

/// この styled Steps の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("steps", SLOTS)
        .base(
            "root",
            vec![decl("display", "flex"), decl("flex-direction", "column")],
        )
        .base(
            "list",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "row"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("list-style", "none"),
                decl("margin", "0"),
                decl("padding", "0"),
            ],
        )
        .state(
            "list",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("flex-direction", "column"),
                decl("align-items", "stretch"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                // `separator`（`flex: 1`）が item 内で実際に伸長できるよう、
                // item 自身も `list` の主軸方向へ伸長させる（バグ報告:
                // イシュー #752 PR #797 cursor[bot] レビュー High severity
                // 指摘「Separators collapse to zero width」対応）。item は
                // `list`（`display: flex`）の直接の子であり、既定の
                // `flex: 0 1 auto` のままでは list の残り幅を専有しないため
                // `separator` の `flex: 1` が効かず接続線が幅ゼロになって
                // いた。
                decl("flex", "1"),
            ],
        )
        // vertical: item を列方向へ切り替え、trigger の下に separator
        // （縦の接続線）が来るようにする（イシュー #752 PR #797
        // cursor[bot] レビュー Medium severity 指摘「Vertical item layout
        // stays horizontal」対応）。`align-items: flex-start` は
        // `separator` 側の `margin-left: calc(indicator-size / 2 - 1px)`
        // （indicator 中心に接続線を揃える計算）が item 左端起点を前提と
        // しているため維持する（`align-items: center` にすると trigger
        // 幅により indicator 中心とずれる）。
        // `min-height` は separator（`flex: 1` で伸長する縦の接続線）の
        // ための確定した空きスペースを確保する（バグ報告: イシュー #752
        // PR #797 Bugbot レビュー Medium severity 指摘「Vertical
        // separators collapse to zero」対応）。item は auto-height な
        // column（内容量に応じて高さが決まる）であり `flex: 1` growth
        // だけでは分配できる余剰スペースが存在しないため、separator の
        // 高さがほぼ 0 に潰れていた。`--fandhe-steps-connector-min-height`
        // custom property で呼び出し側からの上書きも可能にする。
        .state(
            "item",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("flex-direction", "column"),
                decl("align-items", "flex-start"),
                decl(
                    "min-height",
                    "var(--fandhe-steps-connector-min-height, 2.5rem)",
                ),
            ],
        )
        // 最後の item（`<li>:last-child`）は伸ばす対象（separator）を
        // 持たないのが典型的な呼び出し方（`separator` は item 間にのみ
        // 挟むため、呼び出し側が最後の separator を省略するのが通常の
        // 使い方）であるため、`flex: 1`/`min-height` を打ち消し、最終
        // ステップの後ろに余分な空白が残らないようにする（バグ報告:
        // イシュー #752 PR #797 Bugbot レビュー Medium severity 指摘
        // 「Last step item still stretches」対応）。同一 slot への状態
        // 規則は登録順の後勝ちで上書きされる契約（[`SlotRecipe`] rustdoc
        // 参照）のため、水平・垂直いずれの直前規則よりも後に登録する。
        .state(
            "item",
            StateCondition::LastChild,
            vec![decl("flex", "none"), decl("min-height", "auto")],
        )
        .base(
            "trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("background", "none"),
                decl("border", "none"),
                decl("cursor", "pointer"),
                decl("font", "inherit"),
                decl("color", "inherit"),
                decl("padding", "0"),
            ],
        )
        .state(
            "trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        // indicator: 円形マーカー。既定（incomplete）は枠線のみの中抜き円。
        .base(
            "indicator",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("width", "var(--fandhe-steps-indicator-size, 2rem)"),
                decl("height", "var(--fandhe-steps-indicator-size, 2rem)"),
                decl("border-radius", "999px"),
                decl("border", "2px solid var(--fandhe-color-border)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("flex-shrink", "0"),
            ],
        )
        // current: 枠線・文字色をアクセントへ切り替える。
        .state(
            "indicator",
            StateCondition::AttrEq("data-state", "current"),
            vec![
                decl(
                    "border-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("color", "var(--fandhe-palette, var(--fandhe-color-accent))"),
            ],
        )
        // complete: 塗りつぶし背景へ切り替える（current との視覚的区別）。
        .state(
            "indicator",
            StateCondition::AttrEq("data-state", "complete"),
            vec![
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "border-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("color", "var(--fandhe-color-bg)"),
            ],
        )
        // separator: item 間の区切り線。既定は境界色、complete で塗り色へ。
        .base(
            "separator",
            vec![
                decl("flex", "1"),
                decl("height", "2px"),
                decl("background", "var(--fandhe-color-border)"),
            ],
        )
        .state(
            "separator",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![
                decl("width", "2px"),
                decl("height", "auto"),
                decl("align-self", "stretch"),
                decl(
                    "margin-left",
                    "calc(var(--fandhe-steps-indicator-size, 2rem) / 2 - 1px)",
                ),
            ],
        )
        .state(
            "separator",
            StateCondition::Attr("data-complete"),
            vec![decl(
                "background",
                "var(--fandhe-palette, var(--fandhe-color-accent))",
            )],
        )
        .base("content", vec![])
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("display", "none")],
        )
        .base("completed-content", vec![])
        .state(
            "completed-content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("display", "none")],
        )
        .base(
            "prev-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("font", "inherit"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-3)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
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
            "prev-trigger",
            StateCondition::Attr("disabled"),
            vec![decl("opacity", "0.5"), decl("cursor", "not-allowed")],
        )
        .base(
            "next-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("font", "inherit"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-3)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("color", "var(--fandhe-color-bg)"),
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
        .state(
            "next-trigger",
            StateCondition::Attr("disabled"),
            vec![decl("opacity", "0.5"), decl("cursor", "not-allowed")],
        )
        // イシュー #1681: Xs/Xl は Sm→Md→Lg の 0.5rem 刻み等差進行を外挿。
        .variant(
            Size::Xs,
            "root",
            vec![decl("--fandhe-steps-indicator-size", "1rem")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("--fandhe-steps-indicator-size", "1.5rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-steps-indicator-size", "2rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-steps-indicator-size", "2.5rem")],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl("--fandhe-steps-indicator-size", "3rem")],
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

/// この styled Steps が生成する静的 CSS 全量を返す（決定的。
/// [`crate::slider::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は
/// 除去してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::steps::Steps::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_headless_ui::steps::Steps;
/// use fandhe_frontend_pre_styled_ui::steps;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let s = Steps::default();
/// let node = steps::root(Size::Md, ColorPalette::Accent, &s, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="steps" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    state: &Steps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    state.root(merged, children)
}

/// styled list パーツ。実体は [`Steps::list`] へそのまま委譲する。
#[must_use]
pub fn list<'a>(state: &Steps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    state.list(attrs, children)
}

/// styled item パーツ。実体は [`Steps::item`] へそのまま委譲する。
#[must_use]
pub fn item<'a>(
    state: &Steps,
    index: usize,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.item(index, attrs, children)
}

/// styled trigger パーツ。実体は [`Steps::trigger`] へそのまま委譲する。
#[must_use]
pub fn trigger<'a>(
    state: &Steps,
    index: usize,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.trigger(index, attrs, children)
}

/// styled indicator パーツ。実体は [`Steps::indicator`] へそのまま委譲する。
#[must_use]
pub fn indicator<'a>(
    state: &Steps,
    index: usize,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.indicator(index, attrs, children)
}

/// styled separator パーツ。実体は [`Steps::separator`] へそのまま委譲する。
#[must_use]
pub fn separator<'a>(
    state: &Steps,
    index: usize,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.separator(index, attrs, children)
}

/// styled content パーツ。実体は [`Steps::content`] へそのまま委譲する。
#[must_use]
pub fn content<'a>(
    state: &Steps,
    index: usize,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.content(index, attrs, children)
}

/// styled completed-content パーツ。実体は [`Steps::completed_content`]
/// へそのまま委譲する。
#[must_use]
pub fn completed_content<'a>(
    state: &Steps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.completed_content(attrs, children)
}

/// styled prev-trigger パーツ。実体は [`Steps::prev_trigger`] へそのまま
/// 委譲する。
#[must_use]
pub fn prev_trigger<'a>(
    state: &Steps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.prev_trigger(attrs, children)
}

/// styled next-trigger パーツ。実体は [`Steps::next_trigger`] へそのまま
/// 委譲する。
#[must_use]
pub fn next_trigger<'a>(
    state: &Steps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.next_trigger(attrs, children)
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
        assert!(a.contains(r#"[data-scope="steps"][data-part="indicator"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn indicator_state_connected_selectors_present() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="steps"][data-part="indicator"][data-state="current"] {"#)
        );
        assert!(
            css.contains(r#"[data-scope="steps"][data-part="indicator"][data-state="complete"] {"#)
        );
    }

    #[test]
    fn separator_complete_selector_present() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="steps"][data-part="separator"][data-complete] {"#));
        assert!(css.contains("background: var(--fandhe-palette, var(--fandhe-color-accent));"));
    }

    #[test]
    fn content_closed_state_hides_via_display_none() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="steps"][data-part="content"][data-state="closed"] {"#));
        assert!(css.contains("display: none;"));
    }

    #[test]
    fn trigger_and_nav_triggers_link_to_focus_visible() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="steps"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains(r#"[data-scope="steps"][data-part="prev-trigger"]:focus-visible {"#));
        assert!(css.contains(r#"[data-scope="steps"][data-part="next-trigger"]:focus-visible {"#));
    }

    #[test]
    fn stylesheet_contains_size_and_palette_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--color-palette-"));
        assert!(css.contains("--fandhe-steps-indicator-size"));
    }

    // --- root ---

    #[test]
    fn root_outputs_scope_and_part() {
        let s = Steps::default();
        let html = render(&root(Size::Md, ColorPalette::Accent, &s, vec![], vec![]));
        assert!(html.contains(r#"data-scope="steps""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let s = Steps::default();
        let html = render(&root(Size::Md, ColorPalette::Accent, &s, vec![], vec![]));
        assert!(html.contains("fd-steps--size-md"));
        assert!(html.contains("fd-steps--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        let s = Steps::default();
        for (size, class) in [
            (Size::Sm, "fd-steps--size-sm"),
            (Size::Md, "fd-steps--size-md"),
            (Size::Lg, "fd-steps--size-lg"),
        ] {
            let html = render(&root(size, ColorPalette::Accent, &s, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        let s = Steps::default();
        for (palette, class) in [
            (ColorPalette::Accent, "fd-steps--color-palette-accent"),
            (ColorPalette::Info, "fd-steps--color-palette-info"),
            (ColorPalette::Success, "fd-steps--color-palette-success"),
            (ColorPalette::Warning, "fd-steps--color-palette-warning"),
            (ColorPalette::Danger, "fd-steps--color-palette-danger"),
            (ColorPalette::Neutral, "fd-steps--color-palette-neutral"),
        ] {
            let html = render(&root(Size::Md, palette, &s, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let s = Steps::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let s = Steps::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="steps""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- パーツ委譲 ---

    #[test]
    fn list_item_trigger_indicator_separator_delegate_to_headless() {
        let s = Steps::new(3, 1, Orientation::Horizontal);
        assert!(render(&list(&s, vec![], vec![])).contains(r#"data-part="list""#));
        assert!(render(&item(&s, 1, vec![], vec![])).contains(r#"data-state="current""#));
        assert!(render(&trigger(&s, 1, vec![], vec![])).contains(r#"aria-current="step""#));
        assert!(render(&indicator(&s, 0, vec![], vec![])).contains(r#"data-state="complete""#));
        assert!(render(&separator(&s, 0, vec![], vec![])).contains(r#"role="separator""#));
    }

    #[test]
    fn content_and_completed_content_delegate_to_headless() {
        let s = Steps::new(3, 3, Orientation::Horizontal);
        // 有効な content インデックスは 0..count。completed 状態
        // （step == count）では current な content は存在しないため、
        // 有効インデックスの content は closed のままであることを検証する。
        assert!(render(&content(&s, 0, vec![], vec![text("x")])).contains(r#"data-state="closed""#));
        assert!(render(&completed_content(&s, vec![], vec![])).contains(r#"data-state="open""#));
    }

    #[test]
    fn prev_and_next_trigger_delegate_to_headless() {
        let s = Steps::new(3, 0, Orientation::Horizontal);
        assert!(render(&prev_trigger(&s, vec![], vec![])).contains("disabled"));
        assert!(!render(&next_trigger(&s, vec![], vec![])).contains("disabled"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let s = Steps::default();
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_children_text_is_escaped_on_render() {
        let s = Steps::default();
        let html = render(&item(
            &s,
            0,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_steps_state_machine() {
        // `Steps` は本モジュールから再エクスポートしない（本モジュール冒頭の
        // rustdoc「全パーツが `state: &Steps` を取る理由」参照）ため、
        // headless-ui から直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut s = Steps::new(3, 0, Orientation::Horizontal);
        let ssr_html = render(&root(Size::Md, ColorPalette::Accent, &s, vec![], vec![]));
        assert!(!ssr_html.contains("data-hydrate-"));

        assert!(dispatch(&mut s, "next", ""));
        assert_eq!(s.step(), 1);

        let hydrate_html = render(&render_for_hydration(&s));
        assert!(hydrate_html.contains(r#"data-hydrate-step="1""#));

        let restored = Steps::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }
}

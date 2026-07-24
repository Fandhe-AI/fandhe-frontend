//! styled Tour（headless ラッパー、イシュー #841、親 #520/#735）。
//!
//! `fandhe_frontend_headless_ui::tour`（イシュー #841）の Root / Backdrop /
//! Spotlight / Positioner / Arrow / ArrowTip / Content / Title / Description /
//! ProgressText / CloseTrigger / ActionTrigger の 12 anatomy パーツと
//! [`fandhe_frontend_headless_ui::tour::Tour`] 状態機械へ薄く委譲し、
//! [`stylesheet`] で既定 CSS（全面オーバーレイ・スポットライト・カード状の
//! content・side/align 連動の静的配置フォールバック）を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::steps`]/[`crate::dialog`] の
//! rustdoc と同じ方針に従う。
//!
//! # 全パーツが `state: &Tour` を取る理由（headless 層に自由関数がない）
//!
//! [`fandhe_frontend_headless_ui::tour`] は（[`crate::dialog`] の
//! `backdrop`/`positioner`/`title`/`description` 等と異なり）自由関数を
//! 一切持たず、すべて [`fandhe_frontend_headless_ui::tour::Tour`] の
//! inherent メソッドとして提供される（`data-state`/`data-status`・現在
//! ステップの `placement`/`target` の参照が毎回必要なため、[`crate::steps`]
//! と同じ理由）。本モジュールも同型で、すべての styled パーツ関数が
//! `state: &Tour` を受け取り、内部で `state.<part>(...)` へ委譲する。
//!
//! `Tour` 状態機械自体は再エクスポートしない（[`crate::switch`] の `Switch`
//! 非再エクスポートと同じ理由）。呼び出し側が `state.root(...)` を直接
//! 呼ぶと `palette` variant クラスが付与されない未スタイル描画になる事故を
//! 誘発するため、状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::tour::Tour` を直接 import し、実際の描画は
//! 本モジュールの styled パーツ関数を組み合わせて構築すること。
//!
//! # `palette` variant（`size` は初版スコープ外）
//!
//! `palette`（[`ColorPalette`]）は既存の
//! [`crate::recipe::palette_declarations`]（chakra-ui virtual token方式、
//! #606）を [`root`] へ登録し、`action-trigger`/スポットライト縁取りの
//! 強調色を `var(--fandhe-palette, ...)` 経由で切り替える。`size` variant は
//! 初版スコープ外とする（イシュー #841 計画の縮約判断。overlay 系コンポー
//! ネントの寸法は呼び出し側の CSS カスタムプロパティ上書きに委ねる）。
//!
//! # overlay の stacking context・座標フォールバック（[`crate::dialog`] 前例踏襲）
//!
//! `backdrop`/`spotlight`/`positioner` は `position: fixed; inset: 0`
//! （`positioner` のみ実際には `data-side`/`data-align` 基準の静的フォール
//! バック配置、後述）のビューポート全体オーバーレイであり、`z-index` を
//! [`crate::dialog`] の値（backdrop 1000 / positioner 1001）よりさらに前面
//! に固定する（Tour は Dialog の上に重ねて案内する用途を想定するため）。
//! closed 時は headless 層が付与する `hidden` 存在属性を
//! `[data-part="..."][hidden] { display: none }` の明示規則で確実に
//! 非表示化する（[`crate::dialog`] の `positioner[hidden]` 前例と同じ
//! 詳細度対策）。
//!
//! `positioner` の実座標追従（`getBoundingClientRect` 相当）は
//! `fandhe-frontend-wasm-full` の後続イシューの責務（headless 層 rustdoc
//! 参照）であり、本モジュールは `data-side`/`data-align` に応じた
//! `position: absolute` 相当の静的フォールバック配置のみを提供する
//! （ADR §4.1、[`crate::popover`]/[`crate::menu`] の positioner と同型の
//! 「実 DOM 計測なしでも崩れない初期表示」方針）。
//!
//! # `spotlight` の CSS 変数契約
//!
//! `spotlight` は `--fandhe-tour-spotlight-x`/`-y`/`-width`/`-height` の
//! 4 つの CSS custom property（既定値つき `var()`）で位置・寸法を表現する。
//! 実測値の注入は `fandhe-frontend-wasm-full` の後続イシューが担い、本
//! モジュールは変数未設定時のフォールバック矩形（画面中央付近の固定枠）を
//! 提供するのみである。
//!
//! # `focus-visible`（キーボードフォーカスリング）
//!
//! `close-trigger`/`action-trigger` はネイティブな `<button>` であるため、
//! 通常の `:focus-visible` 疑似クラスを [`recipe`] へ直接登録する
//! （[`crate::dialog`]/[`crate::steps`] と同型）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層への委譲と静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラル
//! であり、動的値（`attrs`/children/`target`）へ CSS 値として流し込む経路
//! を持たない（動的値は headless 層経由で `fandhe_frontend_core::render`
//! の既定エスケープを必ず通る、REQ-1）。styled `root` は
//! [`drop_class_attr`] により呼び出し側の `class` を除去してから合成する
//! ため、`class` 属性は常に単一（[`crate::steps::root`] と同型）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく、対象要素の実座標追従・スポットライトへの実測値
//!   注入・`target` セレクタの実解決・クリック/キーボードの実配線は
//!   スコープ外（`fandhe_frontend_headless_ui::tour` モジュール doc 参照）。
//! - `examples/headless-pre-styled-ui` への Tour 追加は、未公開の新
//!   バージョンを参照できないため本イシューのスコープ外とする
//!   （[`crate::steps`] 冒頭 rustdoc の先例どおり crates.io 公開後に追随）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{palette_declarations, ColorPalette, SlotRecipe, StateCondition, VariantValue};

// `Tour` 状態機械はあえて再エクスポートしない（本モジュール冒頭の rustdoc
// 「全パーツが `state: &Tour` を取る理由」節参照)。状態管理・hydration が
// 必要な呼び出し側は `fandhe_frontend_headless_ui::tour::Tour` を直接
// import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::tour::Tour;
pub use fandhe_frontend_headless_ui::tour::{ContentIds, TourAction, TourStatus, TourStep};

/// headless `tour` anatomy の `data-part` 一覧（`crates/headless-ui/src/tour.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "backdrop",
    "spotlight",
    "positioner",
    "arrow",
    "arrow-tip",
    "content",
    "title",
    "description",
    "progress-text",
    "close-trigger",
    "action-trigger",
];

/// この styled Tour の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("tour", SLOTS)
        .base(
            "backdrop",
            vec![
                decl("position", "fixed"),
                decl("inset", "0"),
                decl("z-index", "1100"),
                decl("background", "rgba(0, 0, 0, 0.5)"),
            ],
        )
        .state(
            "backdrop",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .base(
            "spotlight",
            vec![
                decl("position", "fixed"),
                decl("z-index", "1101"),
                decl("top", "var(--fandhe-tour-spotlight-y, 40%)"),
                decl("left", "var(--fandhe-tour-spotlight-x, 40%)"),
                decl("width", "var(--fandhe-tour-spotlight-width, 20%)"),
                decl("height", "var(--fandhe-tour-spotlight-height, 20%)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("box-shadow", "0 0 0 max(100vw, 100vh) rgba(0, 0, 0, 0.5)"),
                decl("pointer-events", "none"),
            ],
        )
        .state(
            "spotlight",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .base(
            "positioner",
            vec![
                decl("position", "fixed"),
                decl("z-index", "1102"),
                decl("top", "50%"),
                decl("left", "50%"),
                decl("transform", "translate(-50%, -50%)"),
                decl("display", "flex"),
                decl("padding", "var(--fandhe-space-4)"),
            ],
        )
        // 実座標追従前の静的フォールバック（`data-side`/`data-align` に
        // 応じてビューポート端寄りへ寄せる、実測値注入は wasm-full 後続）。
        .state(
            "positioner",
            StateCondition::AttrEq("data-side", "top"),
            vec![
                decl("top", "var(--fandhe-space-4)"),
                decl("transform", "translateX(-50%)"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEq("data-side", "bottom"),
            vec![
                decl("top", "auto"),
                decl("bottom", "var(--fandhe-space-4)"),
                decl("transform", "translateX(-50%)"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEq("data-side", "left"),
            vec![
                decl("left", "var(--fandhe-space-4)"),
                decl("transform", "translateY(-50%)"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEq("data-side", "right"),
            vec![
                decl("left", "auto"),
                decl("right", "var(--fandhe-space-4)"),
                decl("transform", "translateY(-50%)"),
            ],
        )
        .state(
            "positioner",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .base("arrow", vec![decl("position", "relative")])
        .base(
            "arrow-tip",
            vec![
                decl("width", "0.75rem"),
                decl("height", "0.75rem"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("transform", "rotate(45deg)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("position", "relative"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl(
                    "box-shadow",
                    "var(--fandhe-shadow-lg, 0 10px 30px rgba(0, 0, 0, 0.25))",
                ),
                decl("padding", "var(--fandhe-space-6)"),
                decl("max-width", "24rem"),
            ],
        )
        .state(
            "content",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .base(
            "title",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("margin", "0 0 var(--fandhe-space-2) 0"),
            ],
        )
        .base(
            "description",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("margin", "0 0 var(--fandhe-space-4) 0"),
            ],
        )
        .base(
            "progress-text",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("margin", "0 0 var(--fandhe-space-4) 0"),
            ],
        )
        .base(
            "close-trigger",
            vec![
                decl("position", "absolute"),
                decl("top", "var(--fandhe-space-2)"),
                decl("right", "var(--fandhe-space-2)"),
                decl("cursor", "pointer"),
                decl("background", "none"),
                decl("border", "none"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .state(
            "close-trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .base(
            "action-trigger",
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
            "action-trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
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

/// この styled Tour が生成する静的 CSS 全量を返す（決定的。
/// [`crate::steps::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`palette` に応じたクラスを付与する唯一
/// のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::tour::Tour::root`] へ
/// 委譲する。
#[must_use]
pub fn root<'a>(
    palette: ColorPalette,
    state: &Tour,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    state.root(merged, children)
}

/// styled backdrop パーツ。実体は [`Tour::backdrop`] へそのまま委譲する。
#[must_use]
pub fn backdrop<'a>(state: &Tour, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    state.backdrop(attrs, children)
}

/// styled spotlight パーツ。実体は [`Tour::spotlight`] へそのまま委譲する。
#[must_use]
pub fn spotlight<'a>(state: &'a Tour, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    state.spotlight(attrs, children)
}

/// styled positioner パーツ。実体は [`Tour::positioner`] へそのまま委譲する。
#[must_use]
pub fn positioner<'a>(state: &Tour, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    state.positioner(attrs, children)
}

/// styled arrow パーツ。実体は [`Tour::arrow`] へそのまま委譲する。
#[must_use]
pub fn arrow<'a>(state: &Tour, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    state.arrow(attrs, children)
}

/// styled arrow-tip パーツ。実体は [`Tour::arrow_tip`] へそのまま委譲する。
#[must_use]
pub fn arrow_tip<'a>(state: &Tour, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    state.arrow_tip(attrs, children)
}

/// styled content パーツ。実体は [`Tour::content`] へそのまま委譲する。
#[must_use]
pub fn content<'a>(
    state: &Tour,
    ids: ContentIds<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.content(ids, attrs, children)
}

/// styled title パーツ。実体は [`Tour::title`] へそのまま委譲する。
#[must_use]
pub fn title<'a>(
    state: &Tour,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.title(id, attrs, children)
}

/// styled description パーツ。実体は [`Tour::description`] へそのまま委譲する。
#[must_use]
pub fn description<'a>(
    state: &Tour,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.description(id, attrs, children)
}

/// styled progress-text パーツ。実体は [`Tour::progress_text`] へそのまま
/// 委譲する。
#[must_use]
pub fn progress_text<'a>(
    state: &Tour,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.progress_text(attrs, children)
}

/// styled close-trigger パーツ。実体は [`Tour::close_trigger`] へそのまま
/// 委譲する。
#[must_use]
pub fn close_trigger<'a>(
    state: &Tour,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.close_trigger(attrs, children)
}

/// styled action-trigger パーツ。実体は [`Tour::action_trigger`] へそのまま
/// 委譲する。
#[must_use]
pub fn action_trigger<'a>(
    state: &Tour,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.action_trigger(attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_headless_ui::fandhe_frontend_core::{render, text};

    fn sample_tour() -> Tour {
        Tour::new(vec![TourStep {
            id: "s1".to_string(),
            target: Some("#a".to_string()),
            title: "One".to_string(),
            description: "first".to_string(),
            placement: fandhe_frontend_headless_ui::positioning::Placement::new(
                fandhe_frontend_headless_ui::positioning::Side::Bottom,
                fandhe_frontend_headless_ui::positioning::Align::Center,
            ),
        }])
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="tour"][data-part="content"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_contains_palette_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--color-palette-"));
    }

    #[test]
    fn hidden_backdrop_spotlight_positioner_content_are_display_none() {
        let css = stylesheet();
        for part in ["backdrop", "spotlight", "positioner", "content"] {
            assert!(
                css.contains(&format!(
                    r#"[data-scope="tour"][data-part="{part}"][hidden] {{"#
                )),
                "missing hidden rule for {part}"
            );
        }
    }

    #[test]
    fn root_outputs_scope_and_part_and_palette_class() {
        let s = sample_tour();
        let html = render(&root(ColorPalette::Accent, &s, vec![], vec![]));
        assert!(html.contains(r#"data-scope="tour""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains("fd-tour--color-palette-accent"));
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let s = sample_tour();
        let html = render(&root(
            ColorPalette::Accent,
            &s,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn parts_delegate_to_headless() {
        let mut s = sample_tour();
        fandhe_frontend_interactive::dispatch(&mut s, "start", "");
        assert!(render(&backdrop(&s, vec![], vec![])).contains(r#"data-part="backdrop""#));
        assert!(render(&spotlight(&s, vec![], vec![])).contains("data-target=\"#a\""));
        assert!(render(&positioner(&s, vec![], vec![])).contains(r#"data-side="bottom""#));
        assert!(render(&arrow(&s, vec![], vec![])).contains(r#"data-part="arrow""#));
        assert!(render(&arrow_tip(&s, vec![], vec![])).contains(r#"data-part="arrow-tip""#));
        assert!(render(&content(&s, ContentIds::default(), vec![], vec![]))
            .contains(r#"role="dialog""#));
        assert!(render(&title(&s, None, vec![], vec![text("t")])).contains("t"));
        assert!(render(&description(&s, None, vec![], vec![text("d")])).contains("d"));
        assert!(
            render(&progress_text(&s, vec![], vec![text("1/1")])).contains(r#"aria-live="polite""#)
        );
        assert!(render(&close_trigger(&s, vec![], vec![])).contains(r#"type="button""#));
        assert!(render(&action_trigger(&s, vec![], vec![])).contains(r#"type="button""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_tour_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut s = sample_tour();
        let ssr_html = render(&root(ColorPalette::Accent, &s, vec![], vec![]));
        assert!(!ssr_html.contains("data-hydrate-"));

        assert!(dispatch(&mut s, "start", ""));
        assert_eq!(s.status(), TourStatus::Active { step: 0 });

        let hydrate_html = render(&render_for_hydration(&s));
        assert!(hydrate_html.contains(r#"data-hydrate-status="active""#));

        let restored = Tour::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let s = sample_tour();
        let html = render(&root(
            ColorPalette::Accent,
            &s,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn title_children_text_is_escaped_on_render() {
        let s = sample_tour();
        let html = render(&title(
            &s,
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}

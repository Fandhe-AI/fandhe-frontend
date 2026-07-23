//! styled Combobox（headless ラッパー、イシュー #749、親 #520）。
//!
//! `fandhe_frontend_headless_ui::combobox`（イシュー #749）の Root / Label /
//! Control / Input / Trigger / ClearTrigger / Positioner / Content /
//! ItemGroup / ItemGroupLabel / Item / ItemText / ItemIndicator 13 anatomy
//! パーツを再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::select`] の rustdoc と同じ方針に
//! 従う（Combobox は Select の直接の姉妹コンポーネントであり、`size`
//! variant・data-state 連動・キーボード操作系属性・positioning 連携の設計は
//! すべて select 実装を踏襲する）。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Combobox` 型・
//! headless `root` を再エクスポートしない理由）
//!
//! [`crate::select`] と同じ理由（`size` variant クラス付与のため styled
//! [`root`] を本モジュールで新設し、headless 自由関数 `root` と名前が衝突
//! するため）で、必要な識別子のみを選択的に再エクスポートする。状態機械
//! [`fandhe_frontend_headless_ui::combobox::Combobox`] は**あえて**
//! 再エクスポートしない（[`crate::select`]/[`crate::switch`]/
//! [`crate::menu`] の状態機械非再エクスポートと同じ理由）。`Combobox` に
//! よる状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::combobox::Combobox` を直接 import し、実際の
//! 描画は本モジュールの styled [`root`]（および再エクスポート済みのパーツ
//! 関数）を組み合わせて構築すること。
//!
//! # data-state とスタイルの連動
//!
//! `input`/`trigger`（listbox 開閉）・`item`（選択有無、`data-state` を
//! 再利用）の `data-state` に応じた見た目の切り替えを [`recipe`] へ登録する
//! （[`crate::select`] と同じ機構、[`crate::recipe::SlotRecipe::state`]）。
//!
//! # キーボード操作系属性の反映
//!
//! `item` は [`crate::select`]/[`crate::menu`] と同じ virtual focus パターンを
//! 使い、実 DOM フォーカスは `input` に留まる（headless
//! `crates/headless-ui/src/combobox.rs` の ARIA 1.2 配線、`trigger` ではなく
//! `input` 側にフォーカスがある点が Select との差異）。ハイライト中の項目には
//! `data-highlighted` が付与されるため、highlight 表示は
//! [`crate::recipe::StateCondition::Attr`]`("data-highlighted")` で反映する。
//! `:focus-visible` はフォーカスを実際に受ける `input` へ登録する
//! （[`crate::select`] が `trigger` へ登録するのと対の判断）。
//!
//! # `--fandhe-reference-width` の消費
//!
//! [`crate::select`]/[`crate::menu`] と同じ理由（各モジュール rustdoc
//! 参照）で、`content` の `min-width` が
//! `var(--fandhe-reference-width, auto)` を参照し、listbox 幅が `control`
//! 幅へ追随する sameWidth 相当の見た目を実現する。[`crate::select`] と同じく
//! フォールバック値は `auto` を採用する（変数未設定時の SSR 静的表示での
//! 見た目変化を避けるため）。
//!
//! # 位置ジオメトリ（`--fandhe-x`/`--fandhe-y`）の消費
//!
//! [`crate::select`] と同じ理由・同じ仕組み（モジュール rustdoc 参照）で、
//! `positioner` へ `data-positioned` マーカーが付与されたときのみ確定座標
//! （viewport 座標系の `position: fixed`）へ切り替える。arrow は持たない
//! （Combobox に arrow anatomy 自体が存在しないため）ため、
//! `--fandhe-arrow-*` の消費は追加しない。
//!
//! # `input`/`control`/`positioner` の視覚配置
//!
//! `control` は入力欄・トリガー・クリアボタンを横並びにする flex コンテナで
//! あり、境界線・角丸は `control` 側へ集約する（`input` 自体は枠を持たず
//! `control` に溶け込む）。`positioner` は `position: absolute` で配置し、
//! 開いた listbox が通常のフローに残らずオーバーレイ表示になるようにする
//! （[`crate::select`] の `positioner` と同じ配置責務）。containing block を
//! 提供する `position: relative` は共通の祖先である `root` に付与する
//! （[`crate::select`] の `root`/`control`/`positioner` 配置と同型の判断、
//! PR #575 Bugbot 指摘の教訓を踏襲）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};

// headless 自由関数 `root`・状態機械 `Combobox` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。未スタイル・
// variant クラス非付与の実体・状態管理が必要な呼び出し側は
// `fandhe_frontend_headless_ui::combobox` を直接 import する。
pub use fandhe_frontend_headless_ui::combobox::{
    clear_trigger, content, control, filter_options, input, item, item_group, item_group_label,
    item_indicator, item_text, label, positioner, trigger,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
// `control`/`input`/`trigger` 等の `state` 引数はいずれも `state` モジュール
// 由来で上記選択的再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（イシュー #685 の契約、[`crate::select`] と同型）。
pub use fandhe_frontend_headless_ui::state::OpenState;

/// headless `combobox` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/combobox.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "input",
    "trigger",
    "clear-trigger",
    "positioner",
    "content",
    "item-group",
    "item-group-label",
    "item",
    "item-text",
    "item-indicator",
];

/// この styled Combobox の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("combobox", SLOTS)
        .base("root", vec![decl("position", "relative")])
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
            "control",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl(
                    "padding",
                    "var(--fandhe-combobox-control-padding, var(--fandhe-space-1) var(--fandhe-space-2))",
                ),
            ],
        )
        .base(
            "input",
            vec![
                decl("flex", "1"),
                decl("border", "none"),
                decl("outline", "none"),
                decl("background", "transparent"),
                decl("color", "inherit"),
                decl("font", "inherit"),
                decl(
                    "padding",
                    "var(--fandhe-combobox-input-padding, var(--fandhe-space-1) var(--fandhe-space-2))",
                ),
            ],
        )
        .base(
            "trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("cursor", "pointer"),
            ],
        )
        .base(
            "clear-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "positioner",
            vec![
                decl("position", "absolute"),
                decl("top", "100%"),
                decl("left", "0"),
                decl("z-index", "10"),
                decl("margin-top", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl("box-shadow", "0 4px 6px rgba(0, 0, 0, 0.15)"),
                decl(
                    "padding",
                    "var(--fandhe-combobox-content-padding, var(--fandhe-space-2))",
                ),
                decl("min-width", "var(--fandhe-reference-width, auto)"),
            ],
        )
        .base(
            "item",
            vec![
                decl(
                    "padding",
                    "var(--fandhe-combobox-item-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
                decl("cursor", "pointer"),
                decl("border-radius", "0.25rem"),
            ],
        )
        .base(
            "item-group-label",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
            ],
        )
        // Combobox 固有: input/trigger の開閉状態の見た目切り替え・item の
        // 選択済み表示（[`crate::select`] の "trigger"/"item" state 登録と
        // 同型）。
        .state(
            "control",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("border-color", "var(--fandhe-color-accent)")],
        )
        .state(
            "item",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("background", "var(--fandhe-color-bg-muted)")],
        )
        // virtual focus の highlight 表示（item は実 DOM フォーカスを受けない
        // ため `:focus-visible` ではなく `data-highlighted` で表現する。
        // 既存の選択済み表示（背景 bg-muted）とは異なる強度にして視覚的に
        // 区別する、[`crate::select`] と同じ判断）。
        .state(
            "item",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        // フォーカスは input が保持するため、`:focus-visible` は input へ
        // 登録する（[`crate::select`] が trigger へ登録するのと対の判断、
        // モジュール rustdoc 参照）。
        .state(
            "input",
            StateCondition::FocusVisible,
            vec![decl("outline", "none")],
        )
        .state(
            "control",
            StateCondition::FocusWithin,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        // wasm 層が `data-positioned` マーカーを付与したら確定座標
        // （viewport 座標系の `position: fixed`）へ切り替える（[`crate::select`]
        // と同じ契約、モジュール rustdoc 参照）。
        .state(
            "positioner",
            StateCondition::Attr("data-positioned"),
            vec![
                decl("position", "fixed"),
                decl("top", "0"),
                decl("left", "0"),
                decl("margin-top", "0"),
                decl(
                    "transform",
                    "translate3d(var(--fandhe-x, 0px), var(--fandhe-y, 0px), 0)",
                ),
            ],
        )
        // `size` variant（root スコープの CSS custom property。Md はフォール
        // バック値と同一の現行外観を維持する）。`--fandhe-reference-width`/
        // `--fandhe-x`/`--fandhe-y`（wasm positioning 契約）には手を触れない
        // （モジュール rustdoc 参照）。
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-combobox-control-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl(
                    "--fandhe-combobox-input-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl(
                    "--fandhe-combobox-item-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl("--fandhe-combobox-content-padding", "var(--fandhe-space-1)"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-combobox-control-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl(
                    "--fandhe-combobox-input-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl(
                    "--fandhe-combobox-item-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl("--fandhe-combobox-content-padding", "var(--fandhe-space-2)"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-combobox-control-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl(
                    "--fandhe-combobox-input-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl(
                    "--fandhe-combobox-item-padding",
                    "var(--fandhe-space-3) var(--fandhe-space-4)",
                ),
                decl("--fandhe-combobox-content-padding", "var(--fandhe-space-3)"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled Combobox が生成する静的 CSS 全量を返す（決定的。
/// [`crate::select::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::combobox::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::combobox::{self, OpenState};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = combobox::root(Size::Md, OpenState::Open, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="combobox" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::combobox::root(state, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::state::OpenState;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="combobox"][data-part="input"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn positioner_is_absolute_and_root_provides_containing_block() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="combobox"][data-part="positioner"]"#));
        assert!(css.contains("position: absolute;"));
        assert!(css.contains(
            "[data-scope=\"combobox\"][data-part=\"root\"] {\n  position: relative;\n}\n"
        ));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Size::Md, OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="combobox""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    // --- size variant ---

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let html = render(&root(
                size,
                OpenState::Closed,
                vec![("class", "attacker")],
                vec![],
            ));
            let expected_class = format!("fd-combobox--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn default_variant_is_md() {
        let css = stylesheet();
        assert!(css.contains(
            "padding: var(--fandhe-combobox-control-padding, var(--fandhe-space-1) var(--fandhe-space-2));"
        ));
        assert!(css.contains(
            "padding: var(--fandhe-combobox-item-padding, var(--fandhe-space-2) var(--fandhe-space-3));"
        ));
        assert!(
            css.contains("padding: var(--fandhe-combobox-content-padding, var(--fandhe-space-2));")
        );
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="combobox"][data-part="control"][data-state="open"]"#));
        assert!(css.contains(r#"[data-scope="combobox"][data-part="item"][data-state="open"]"#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_combobox_state_machine() {
        // headless `Combobox` はあえて本モジュールから再エクスポートしない
        // ため、エスケープハッチ経由で直接 import する（モジュール冒頭の
        // rustdoc「選択的 re-export」節参照、[`crate::select`] と同型）。
        use fandhe_frontend_headless_ui::combobox::Combobox;
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut c = Combobox::default();
        assert_eq!(c.open_state(), OpenState::Closed);

        let ssr_html = render(&c.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut c, "input", "vu"));
        let hydrate_html = render(&render_for_hydration(&c));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = Combobox::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored.open_state(), OpenState::Open);
        assert_eq!(restored.input_value(), "vu");
    }

    #[test]
    fn item_highlighted_attr_is_styled_and_input_has_focus_within_ring_on_control() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="combobox"][data-part="item"][data-highlighted] {"#));
        assert!(css.contains(r#"[data-scope="combobox"][data-part="control"]:focus-within {"#));
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }

    #[test]
    fn content_min_width_consumes_fandhe_reference_width_css_var() {
        let css = stylesheet();
        assert!(css.contains("min-width: var(--fandhe-reference-width, auto);"));
    }

    #[test]
    fn positioner_switches_to_fixed_geometry_when_data_positioned_marker_is_present() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"combobox\"][data-part=\"positioner\"][data-positioned] {\n  \
             position: fixed;\n  \
             top: 0;\n  \
             left: 0;\n  \
             margin-top: 0;\n  \
             transform: translate3d(var(--fandhe-x, 0px), var(--fandhe-y, 0px), 0);\n\
             }\n"
        ));
    }

    #[test]
    fn positioner_base_rule_keeps_static_ssr_fallback_geometry() {
        let css = stylesheet();
        assert!(css.contains("position: absolute;"));
        assert!(css.contains("top: 100%;"));
    }

    #[test]
    fn combobox_stylesheet_never_consumes_fandhe_arrow_geometry() {
        let css = stylesheet();
        assert!(!css.contains("--fandhe-arrow-"));
    }

    #[test]
    fn position_geometry_var_references_never_lack_an_explicit_fallback() {
        // fail-closed 回帰（[`crate::select`] と同型）: 位置ジオメトリ変数
        // （`--fandhe-x`/`--fandhe-y`）への参照はすべて明示フォールバック値を
        // 持つ（裸の `var(--x)` 禁止）。
        let css = stylesheet();
        for marker in ["var(--fandhe-x", "var(--fandhe-y"] {
            for (idx, _) in css.match_indices(marker) {
                let close = css[idx..]
                    .find(')')
                    .expect("every var( occurrence must be closed within the stylesheet");
                let inside = &css[idx + "var(".len()..idx + close];
                assert!(
                    inside.contains(','),
                    "var() reference without an explicit fallback found: var({inside})"
                );
            }
        }
    }
}

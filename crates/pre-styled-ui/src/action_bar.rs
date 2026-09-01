//! styled ActionBar（headless ラッパー、イシュー #762、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::action_bar`（イシュー #762）の Root /
//! Positioner / Content / SelectionTrigger / Separator / CloseTrigger
//! 6 anatomy パーツと [`fandhe_frontend_headless_ui::action_bar::ActionBar`]
//! 状態機械をそのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供
//! する。薄い委譲の根拠・スコープ外事項は [`crate::dialog`]/[`crate::tooltip`]
//! の rustdoc と同じ方針に従う。
//!
//! # 画面下部固定配置（chakra-ui ActionBar 相当）
//!
//! `positioner` は `position: fixed; bottom: ...; left: 50%; transform:
//! translateX(-50%)` により画面下部中央へ固定表示する。`z-index` は
//! [`crate::menu`]/[`crate::select`] の dropdown `positioner`（10）より上、
//! [`crate::dialog`] の backdrop（1000）より下（900）とする（複数選択の操作
//! バーはダイアログの上に隠れるべきではないが、モーダルよりは背面という
//! 位置付け）。
//!
//! # data-state とスタイルの連動
//!
//! `content` の開閉 `data-state`（open/closed）に応じた見た目の切り替えを
//! [`recipe`] へ登録する（[`crate::recipe::SlotRecipe::state`]、
//! [`crate::dialog`]/[`crate::tooltip`] と同じ判断）。
//!
//! # キーボード操作系属性の反映
//!
//! `selection-trigger`/`close-trigger` はフォーカス可能なボタン要素であり、
//! キーボード操作時のみのフォーカスリング（`:focus-visible`）を
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::dialog`]/[`crate::tooltip`] と同じ判断）。
//!
//! # closed 時の `positioner` は必ず非表示化する（PR #575 Bugbot 指摘対応の踏襲）
//!
//! headless 層（`crates/headless-ui/src/action_bar.rs`）は ActionBar が
//! closed のとき `positioner` に `hidden` 存在属性を付与し、UA 既定スタイル
//! `[hidden] { display: none }` によって非表示化させる契約になっている。
//! [`recipe`] の base 規則は `positioner` に `display: flex` を宣言しており、
//! この author スタイルが UA スタイルより詳細度で優先されるため `[hidden]`
//! 単体では非表示化できない（[`crate::dialog`] の PR #575 Bugbot 指摘（High）
//! と同型の不具合）。[`recipe`] に `[data-scope="action-bar"]
//! [data-part="positioner"][hidden]` に対する `display: none` の明示的な
//! 上書き規則を追加し、`display: flex` より詳細度・出現順の両方で優先させる
//! ことでこれを固定する。
//!
//! # 参考サイト基準へのスタイル調整（イシュー #1516）
//!
//! chakra-ui ActionBar（MCP `get_component_example`/`get_theme` で一次確認）
//! を基準に、7 軸チェックリスト（色/角丸/影・フォーカス・hover・
//! トランジション・タイポグラフィ）の不足を是正した。
//!
//! - **角丸・影のトークン化**: `content` の `border-radius`/`box-shadow` は
//!   生リテラルではなく [`crate::theme`] のスケールトークン
//!   （`--fandhe-radius-lg`/`--fandhe-shadow-md`）を参照する。`box-shadow`
//!   は [`crate::combobox`]/[`crate::select`]/[`crate::toast`]/
//!   [`crate::date_picker`] の overlay 系部品がいずれも `shadow-md` を使う
//!   前例に揃え、`action-bar` だけ独自の 1 段強い影（`shadow-lg`）を持たせる
//!   差別化はしない（他 overlay と同じ浮遊感で統一する判断）。
//! - **selection-trigger / close-trigger のボタン化**: chakra-ui の
//!   ActionBar は selection-trigger を破線ボーダーの小型ボタン、
//!   close-trigger を ghost の小型 close ボタンとして描く。headless 層は
//!   これらへ `<button>` 相当の役割のみ与え面（padding/border/radius/
//!   背景）を持たないため、[`recipe`] 側で面を追加した上で
//!   [`crate::recipe::hover_bg_muted`]/[`crate::recipe::
//!   hover_surface_declarations`]（イシュー #1425 共通ビジュアル言語）を
//!   `.state(_, StateCondition::Hover, ...)` として登録し、初めて hover
//!   変化を表現できるようにした。
//! - **フォーカスリングの canonical 化**: 旧実装の手書き
//!   `outline: 2px solid var(--fandhe-color-accent)` を
//!   [`crate::recipe::focus_ring_declarations`]（イシュー #1424）へ移行し、
//!   太さ・オフセット・色をテーマ側 1 箇所（`--fandhe-focus-ring-*`）で
//!   変更できるようにした。`palette` 軸を持たない部品のため
//!   [`crate::recipe::FocusRingColor::Token`] を選ぶ。
//! - **トランジション**: `content` の `data-state` open/closed 切り替えへ
//!   [`crate::recipe::transition_declarations`]（イシュー #1425）で
//!   `opacity`/`translate` の遷移を追加し、closed 側に軽い下方向オフセット
//!   （`translate: 0 0.5rem`）を加えて chakra の slide-fade 相当の出現
//!   モーションを表現する。`prefers-reduced-motion` は
//!   [`crate::theme::Theme::to_css`] 側の duration 一括無効化に委ね、本
//!   モジュールで `@media` を書かない（[`crate::segment_group`] 等の既存
//!   判断と同型）。`[hidden]` 属性による `display: none` 切り替え自体は
//!   アニメーションしない既知の制約は [`crate::dialog`] と同様。
//!
//! ## size / variant 軸を追加しない根拠
//!
//! chakra-ui ActionBar 自体が size/variant prop を持たず、内部のボタン
//! （selection-trigger/close-trigger 相当）は利用者が持ち込む `Button`
//! コンポーネントが担う設計になっている（MCP `get_component_example` で
//! 実例確認済み）。本モジュールも同じ構成（headless 層は anatomy・
//! アクセシビリティ・`data-*` のみを担い、size/variant のような見た目の
//! バリエーションは持たない）を踏襲し、`SlotRecipe::variant` 軸を追加しない
//! （`REEXPORT-GLOB-REVIEWED` 規約 B-2 とも整合）。
//!
//! ## disabled 視覚を付けない根拠
//!
//! headless `action-bar`（`crates/headless-ui/src/action_bar.rs`）は
//! `data-disabled` を一切発行しない。selection-trigger/close-trigger の
//! 無効化表現は、利用者が中に配置する `Button` 部品（[`crate::button`]）側
//! が `data-disabled`/`disabled_declarations()` で担う責務であり、
//! action-bar 自体のトリガー slot へ disabled 視覚を持ち込まない。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - Portal 描画・外側クリックでの閉鎖・アニメーション（状態機械側）は
//!   headless 層のドキュメント（`crates/headless-ui/src/action_bar.rs`）で
//!   既にスコープ外と明記済みであり、本モジュールもそれを継承する。
//! - `placement` variant（`bottom-start`/`bottom-end`）: 既定の bottom 中央
//!   固定のみ実装する。variant 追加は `SlotRecipe::variant` で後続可能。

use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, hover_bg_muted, hover_surface_declarations, transition_declarations,
    FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe, StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数・variant 型を再定義しない（規約 B-1）。variant 軸
// も提供せず（規約 B-2）、CSS 到達は [data-scope]/[data-part] 属性セレクタ
// のみに依存する（規約 B-3、イシュー #1062 規約参照）。
pub use fandhe_frontend_headless_ui::action_bar::*;
// `root`/`positioner`/`content` 等の `state` 引数・`ActionBar::new`・
// `ActionBar` の `Component::Action`（dispatch 対象）はいずれも `state`
// モジュール由来で上記 glob 再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（イシュー #685 の先例に倣う）。
pub use fandhe_frontend_headless_ui::state::{DisclosureAction, OpenState};

/// headless `action-bar` anatomy の `data-part` 一覧（`crates/headless-ui/src/action_bar.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "positioner",
    "content",
    "selection-trigger",
    "separator",
    "close-trigger",
];

/// この styled ActionBar の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("action-bar", SLOTS)
        .base(
            "positioner",
            vec![
                decl("position", "fixed"),
                decl("bottom", "var(--fandhe-space-4)"),
                decl("left", "50%"),
                decl("transform", "translateX(-50%)"),
                decl("z-index", "900"),
                decl("display", "flex"),
                decl("justify-content", "center"),
            ],
        )
        .base("content", {
            let mut declarations = vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-3)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
                decl("box-shadow", "var(--fandhe-shadow-md)"),
                decl("padding", "var(--fandhe-space-3) var(--fandhe-space-4)"),
            ];
            declarations.extend(transition_declarations(
                "opacity, translate",
                MotionDuration::Normal,
            ));
            declarations
        })
        .base("selection-trigger", {
            let mut declarations = vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-3)"),
                decl("border", "1px dashed var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "transparent"),
                decl("cursor", "pointer"),
                hover_bg_muted(),
            ];
            declarations.extend(transition_declarations("background", MotionDuration::Fast));
            declarations
        })
        .base(
            "separator",
            vec![
                decl("width", "1px"),
                decl("align-self", "stretch"),
                decl("background", "var(--fandhe-color-border)"),
            ],
        )
        .base("close-trigger", {
            let mut declarations = vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("padding", "var(--fandhe-space-1)"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "transparent"),
                decl("cursor", "pointer"),
                hover_bg_muted(),
            ];
            declarations.extend(transition_declarations("background", MotionDuration::Fast));
            declarations
        })
        // `content` の開閉状態に応じた見た目の切り替え（opacity + 軽い
        // slide-fade。上記モジュール doc「トランジション」節参照）。
        .state(
            "content",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("opacity", "1"), decl("translate", "0 0")],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("opacity", "0"), decl("translate", "0 0.5rem")],
        )
        // selection-trigger/close-trigger の hover（イシュー #1425 共通
        // ビジュアル言語。`--fandhe-hover-bg` は上記 base の `hover_bg_muted()`
        // が定義する）。
        .state(
            "selection-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "close-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // 本モジュール冒頭 rustdoc「closed 時の positioner は必ず非表示化する」
        // 節参照: positioner の base 規則が `display: flex` を宣言しており、
        // UA 既定の `[hidden] { display: none }` を上書きしてしまうため、
        // より詳細度の高い `[hidden]` 属性セレクタで明示的に上書きする。
        .state(
            "positioner",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        // キーボード操作時のみのフォーカスリング（イシュー #1424 canonical
        // ヘルパへ移行。`palette` 軸を持たないため `FocusRingColor::Token`）。
        .state(
            "selection-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "close-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
}

/// この styled ActionBar が生成する静的 CSS 全量を返す（決定的。
/// [`crate::dialog::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
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
        assert!(a.contains(r#"[data-scope="action-bar"][data-part="content"]"#));
        assert!(a.contains(r#"[data-scope="action-bar"][data-part="positioner"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn positioner_is_fixed_to_bottom_center_with_stacking_order() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="action-bar"][data-part="positioner"] {"#));
        assert!(css.contains("position: fixed;"));
        assert!(css.contains("left: 50%;"));
        assert!(css.contains("transform: translateX(-50%);"));
        assert!(css.contains("z-index: 900;"));
    }

    #[test]
    fn closed_positioner_hidden_attr_overrides_display_flex() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="action-bar"][data-part="positioner"][hidden] {"#));
        let rule_start = css
            .find(r#"[data-scope="action-bar"][data-part="positioner"][hidden] {"#)
            .expect("positioner[hidden] rule must be present");
        let rule_body = &css[rule_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        assert!(rule_body[..rule_end].contains("display: none;"));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="action-bar"][data-part="content"][data-state="open"]"#)
        );
        assert!(
            css.contains(r#"[data-scope="action-bar"][data-part="content"][data-state="closed"]"#)
        );
    }

    #[test]
    fn selection_trigger_and_close_trigger_declare_focus_visible_ring() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="action-bar"][data-part="selection-trigger"]:focus-visible {"#
        ));
        assert!(
            css.contains(r#"[data-scope="action-bar"][data-part="close-trigger"]:focus-visible {"#)
        );
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
    }

    #[test]
    fn content_uses_token_scale_radius_and_shadow() {
        let css = stylesheet();
        assert!(css.contains("border-radius: var(--fandhe-radius-lg);"));
        assert!(css.contains("box-shadow: var(--fandhe-shadow-md);"));
    }

    #[test]
    fn content_declares_transition_for_open_close_slide_fade() {
        let css = stylesheet();
        assert!(css.contains("transition-property: opacity, translate;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-normal);"));
        assert!(css.contains(r#"[data-part="content"][data-state="open"]"#));
        assert!(css.contains("translate: 0 0;"));
        assert!(css.contains("translate: 0 0.5rem;"));
    }

    #[test]
    fn selection_trigger_and_close_trigger_declare_hover_background() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(
            r#"[data-scope="action-bar"][data-part="selection-trigger"]:hover:not([data-disabled])"#
        ));
        assert!(css.contains(
            r#"[data-scope="action-bar"][data-part="close-trigger"]:hover:not([data-disabled])"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
        assert!(css.contains("--fandhe-hover-bg: var(--fandhe-color-bg-muted);"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="action-bar""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_action_bar_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut bar = ActionBar::default();
        assert_eq!(bar.state(), OpenState::Closed);

        // SSR: 状態なし初期描画には data-hydrate-* が出ない。
        let ssr_html = render(&bar.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        // dispatch で開閉し、hydration 属性へ反映されることを確認する。
        assert!(dispatch(&mut bar, "open", ""));
        let hydrate_html = render(&render_for_hydration(&bar));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        // クライアント側の改ざん耐性のある復元経路が ActionBar 経由でも機能する。
        let restored = ActionBar::from_hydration_attrs(&bar.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}

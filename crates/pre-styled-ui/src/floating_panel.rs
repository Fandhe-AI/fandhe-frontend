//! styled FloatingPanel（headless ラッパー、イシュー #827、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::floating_panel`（イシュー #827）の Root /
//! Trigger / Positioner / Content / Header / Title / Control / StageTrigger /
//! CloseTrigger / Body 10 anatomy パーツと
//! [`fandhe_frontend_headless_ui::floating_panel::FloatingPanel`] 状態機械を
//! そのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::popover`]/[`crate::dialog`] の
//! rustdoc と同じ方針に従う。
//!
//! # data-state/data-stage とスタイルの連動
//!
//! `content` の開閉 `data-state`（open/closed）に応じた見た目の切り替えを
//! [`crate::popover`] と同じ方針で登録する。加えて `body` の
//! `data-stage="minimized"` を折り畳み（`display: none`。ヘッダのみ表示）、
//! `positioner` の `data-stage="maximized"` をビューポート全面表示
//! （`transform: none; inset: 0`）へ切り替える（[`crate::recipe::SlotRecipe::state`]）。
//!
//! # キーボード操作系属性の反映
//!
//! `trigger`/`stage-trigger`/`close-trigger` はフォーカス可能なボタン要素で
//! あり、キーボード操作時のみのフォーカスリング（`:focus-visible`）を
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::dialog`]/[`crate::popover`] と同じ判断）。
//!
//! # positioner の配置と z-index tier
//!
//! `positioner` はドラッグ移動によりビューポート絶対座標へ置かれるため
//! `position: fixed; left: 0; top: 0` を基点とし、headless 側が出力する
//! `--fandhe-x`/`--fandhe-y`（[`fandhe_frontend_headless_ui::floating_panel::FloatingPanel::position_style`]）
//! を `transform: translate3d(...)` で反映する。z-index は
//! [`crate::dialog`] のモーダル層（1000/1001）未満・[`crate::menu`]/
//! [`crate::popover`] の dropdown 層（10）超の専用 tier（`900`）とする
//! （FloatingPanel は非モーダルだがトリガー付近に留まる dropdown より
//! 手前に来る想定のため、独自の中間層を割り当てる）。`positioner` は base
//! 規則で `display` を宣言しないため、closed 時に headless 層が付与する
//! `hidden` 存在属性は UA 既定 `[hidden] { display: none }` がそのまま機能
//! する（[`crate::popover`] と同じ構造判断、dialog PR #575 の不具合を構造的
//! に回避する）。
//!
//! # 参考サイト基準へのスタイル調整（イシュー #1522）
//!
//! chakra-ui FloatingPanel（MCP `get_component_example` で一次確認）を基準
//! に、7 軸チェックリスト（角丸・影のトークン化・hover・フォーカス・
//! stage-trigger/close-trigger のボタン化・maximized 時の全画面表示）の
//! 不足を是正した。直近の同種先例は [`crate::action_bar`]（#1516/PR #1790。
//! z-index 900 の同 tier・トリガーのボタン化・hover/focus-ring 導入まで
//! 構成が酷似）と [`crate::dialog`]（#1692/PR #1794。トークン化のフォール
//! バック方針）であり、両者のパターンを踏襲する。
//!
//! - **角丸・影のトークン化**: `content` の `border-radius` を
//!   `var(--fandhe-radius-lg)`（面パネルカテゴリ = `lg`、
//!   `docs/design/pre-styled-ui-scale-tokens.md` §3.1）、`box-shadow` を
//!   `var(--fandhe-shadow-md)`（overlay dropdown 型 = `md`、同 §3.2。
//!   [`crate::combobox`]/[`crate::select`]/[`crate::toast`]/
//!   [`crate::date_picker`]/[`crate::action_bar`] と同じ影に統一しダーク値
//!   切り替えも獲得）へそれぞれ変更した。`trigger` の `border-radius` も
//!   `var(--fandhe-radius-md, 0.375rem)` へトークン化する（計算値不変、
//!   [`crate::dialog`] #1692 のフォールバック方針: 単独 `stylesheet()`
//!   利用時の後方互換維持）。
//! - **trigger / stage-trigger / close-trigger の hover・トランジション**:
//!   [`crate::recipe::hover_bg_muted`]/[`crate::recipe::
//!   hover_surface_declarations`]（イシュー #1425 共通ビジュアル言語）を
//!   登録した。`trigger` は headless 層の `disabled` 引数がネイティブ
//!   `disabled` 存在属性のみを出力し `data-disabled` を発行しない
//!   （下記「disabled 視覚を付けない根拠」参照）ため、
//!   `StateCondition::HoverExceptAttr("disabled")`
//!   （`:hover:not([data-disabled]):not([disabled])`）を用いて操作不能な
//!   trigger への hover 背景適用を防ぐ（CI codex-review P1 指摘、イシュー
//!   #1522）。`stage-trigger`/`close-trigger` は headless 層が disabled
//!   引数自体を持たないため従来どおり `StateCondition::Hover`
//!   （`:hover:not([data-disabled])`）のままとする。
//!   stage-trigger/close-trigger は chakra-ui では `IconButton
//!   variant="ghost" size="2xs"` として描かれる（MCP 確認済み）ため、面
//!   （`padding`/`border-radius`/`display: inline-flex` による中央寄せ）
//!   を追加してゴーストボタンの見た目に揃えた。
//! - **フォーカスリングの canonical 化**: 3 trigger の手書き
//!   `outline: 2px solid var(--fandhe-color-accent)` を
//!   [`crate::recipe::focus_ring_declarations`]（イシュー #1424）へ移行
//!   した。`palette` 軸を持たない部品のため
//!   [`crate::recipe::FocusRingColor::Token`] を選ぶ。
//! - **maximized 時の content 全画面拡張**: `positioner`
//!   （`data-stage="maximized"`）は既に全面化していたが `content` が
//!   追随せず全画面表示が視覚的に成立していなかったため、
//!   `content[data-stage="maximized"]` へ `width: 100%; height: 100%` を
//!   追加した。`content` は `border`（1px）+ `border-radius` を持つため、
//!   `box-sizing: border-box` も併せて指定し、border 込みの外寸が
//!   `positioner` の全面領域を縦横 2px 超過してビューポートオーバーフロー
//!   を起こさないようにした（CI codex-review P1 / Cursor Bugbot 指摘、
//!   イシュー #1522）。
//!
//! ## size / variant 軸を追加しない根拠
//!
//! chakra-ui FloatingPanel 自体が size/variant prop を持たない（MCP
//! `get_component_example` で実例確認済み。`FloatingPanel.Trigger` は
//! 呼び出し側が持ち込む `Button`/`IconButton` の size/variant に委ねる
//! 設計）。本モジュールも同じ構成を踏襲し、`SlotRecipe::variant` 軸を
//! 追加しない（`REEXPORT-GLOB-REVIEWED` 規約 B-2 とも整合）。
//!
//! ## disabled 視覚を付けない根拠（data-disabled 版）
//!
//! headless `floating_panel`（`crates/headless-ui/src/floating_panel.rs`）
//! の `trigger`/`stage-trigger`/`close-trigger` はいずれも popover と異なり
//! `data-disabled` を一切発行しない（[`crate::action_bar`] #1516 の
//! 「disabled 視覚を付けない根拠」と同型）ため、`data-disabled` を狙った
//! 専用の disabled 視覚は追加しない。ただし `trigger` は `disabled: bool`
//! 引数を持ち `true` のときネイティブ `disabled` 存在属性のみを出力する
//! （`stage-trigger`/`close-trigger` は disabled 引数自体を持たない）。この
//! ネイティブ `disabled` を hover セレクタが除外できていないと、操作不能な
//! trigger にも hover 背景が表示され誤った視覚フィードバックになる（CI
//! codex-review P1 指摘、イシュー #1522）ため、上記「hover・トランジション」
//!節のとおり `trigger` のみ `HoverExceptAttr("disabled")` で `:not([disabled])`
//! を追加する。
//!
//! ## 開閉（data-state）トランジションを追加しない根拠
//!
//! closed 時は headless 層が `hidden` 存在属性を UA 既定
//! `[hidden] { display: none }` 経由で同期的に切り替えるため、opacity 等の
//! transition は視覚的に成立しない（[`crate::action_bar`] PR #1790
//! codex-review P1 指摘と同一構造）。hover 側の transition は機能するため
//! 上記のとおり追加する。`prefers-reduced-motion` は
//! `transition_declarations` が参照する `--fandhe-motion-duration-*`
//! トークンを `Theme::to_css` が一括 0ms 化する共通経路で自動充足する。
//!
//! ## z-index のトークン化を見送る根拠
//!
//! 現行の生値 `900` を維持する。スケールトークン文書は floating-panel/
//! action-bar を `sticky`（1100）へ割り当てる方針だが、同 tier の
//! action-bar が PR #1790 で生値 900 を維持しており、片方だけ 1100 へ動か
//! すと tier 内の重なり順が非対称になるため本 PR では見送る（Issue 化は
//! ユーザー承認事項のため提案に留める）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - ドラッグ移動・リサイズの実 DOM 配線: headless 層のドキュメント
//!   （`crates/headless-ui/src/floating_panel.rs`）で既にスコープ外と明記
//!   済みであり、本モジュールもそれを継承する（`--fandhe-x`/`--fandhe-y`
//!   の消費のみを提供する）。
//! - variant（size 等）ごとのクラス切り替えは他 headless ラッパーと同じく
//!   スコープ外とする。
//! - フォーカストラップ・Escape キー閉鎖・`lazyMount`・topmost 管理は
//!   headless 層のスコープ外を継承する。
//! - z-index のトークン化（`sticky` = 1100 への統一）: 上記「z-index の
//!   トークン化を見送る根拠」参照。action-bar と揃えて再評価する場合は
//!   両部品を同時に動かす必要があり、Issue 化を提案する。

use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, hover_bg_muted, hover_surface_declarations, transition_declarations,
    FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe, StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。variant 軸も提供せず
// （規約 B-2）、CSS 到達は [data-scope]/[data-part] 属性セレクタのみに依存
// する（規約 B-3、イシュー #1062 規約参照）。
pub use fandhe_frontend_headless_ui::floating_panel::*;
// `root`/`trigger` 等の `state`/`stage` 引数・`FloatingPanel::new`・
// `FloatingPanel` の `Component::Action`（dispatch 対象）はいずれも `state`
// モジュール由来で上記 glob 再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証する
// ための明示再エクスポート（`crate::popover` と同じ契約、イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{DisclosureAction, OpenState};

/// headless `floating_panel` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/floating_panel.rs` の `ANATOMY.part(...)` 呼び
/// 出しと同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を
/// 出力しない fail-closed 側の不具合として現れるため、変更時は両ファイル
/// を合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "positioner",
    "content",
    "header",
    "title",
    "control",
    "stage-trigger",
    "close-trigger",
    "body",
];

/// この styled FloatingPanel の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("floating-panel", SLOTS)
        .base("trigger", {
            let mut declarations = vec![
                decl("cursor", "pointer"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md, 0.375rem)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                hover_bg_muted(),
            ];
            declarations.extend(transition_declarations(
                "background, border-color",
                MotionDuration::Fast,
            ));
            declarations
        })
        .base(
            "positioner",
            vec![
                decl("position", "fixed"),
                decl("left", "0"),
                decl("top", "0"),
                decl("z-index", "900"),
                decl(
                    "transform",
                    "translate3d(var(--fandhe-x, 24px), var(--fandhe-y, 24px), 0)",
                ),
            ],
        )
        .base(
            "content",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
                decl("box-shadow", "var(--fandhe-shadow-md)"),
                decl("min-width", "16rem"),
            ],
        )
        .base(
            "header",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "space-between"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("padding", "var(--fandhe-space-3) var(--fandhe-space-4)"),
                decl("border-bottom", "1px solid var(--fandhe-color-border)"),
                decl("cursor", "move"),
            ],
        )
        .base(
            "title",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("margin", "0"),
            ],
        )
        .base(
            "control",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        .base("stage-trigger", {
            let mut declarations = vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("padding", "var(--fandhe-space-1)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("cursor", "pointer"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                hover_bg_muted(),
            ];
            declarations.extend(transition_declarations("background", MotionDuration::Fast));
            declarations
        })
        .base("close-trigger", {
            let mut declarations = vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("padding", "var(--fandhe-space-1)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("cursor", "pointer"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                hover_bg_muted(),
            ];
            declarations.extend(transition_declarations("background", MotionDuration::Fast));
            declarations
        })
        .base("body", vec![decl("padding", "var(--fandhe-space-4)")])
        // `content` の開閉状態に応じた見た目の切り替え（`crate::popover` と同じ判断）。
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("visibility", "hidden")],
        )
        // minimized: body（本文）を折り畳み、ヘッダのみ表示する。
        // `data-stage` の出力元は headless-ui（`crates/headless-ui/src/
        // floating_panel.rs` の stage 定数群）。本モジュールは CSS セレクタ
        // として参照するのみで、属性を出力しない（イシュー #1063、
        // `docs/design/pre-styled-ui-data-attr-vocabulary.md` 規約 A）。
        .state(
            "body",
            StateCondition::AttrEq("data-stage", "minimized"),
            vec![decl("display", "none")],
        )
        // maximized: ビューポート全面表示へ切り替える（ドラッグ座標の transform を中和）。
        .state(
            "positioner",
            StateCondition::AttrEq("data-stage", "maximized"),
            vec![decl("transform", "none"), decl("inset", "0")],
        )
        // maximized: positioner の全面化に content を追随させる（本イシュー
        // #1522 で新規追加。上記モジュール doc「maximized 時の content
        // 全画面拡張」節参照）。
        .state(
            "content",
            StateCondition::AttrEq("data-stage", "maximized"),
            vec![
                decl("width", "100%"),
                decl("height", "100%"),
                // `content` は `border`（1px）と `border-radius` を持つため、既定の
                // `content-box` のままだと width/height 100% の外寸が全面化された
                // `positioner` から縦横 2px はみ出しビューポートを超過する
                // （CI codex-review P1 / Cursor Bugbot 指摘、イシュー #1522）。
                // `border-box` で border 込みの外寸を 100% に固定しはみ出しを防ぐ。
                decl("box-sizing", "border-box"),
            ],
        )
        // trigger/stage-trigger/close-trigger の hover（イシュー #1425
        // 共通ビジュアル言語。`--fandhe-hover-bg` は上記 base の
        // `hover_bg_muted()` が定義する）。
        // `trigger` はネイティブ `disabled` 属性を native-disabled として
        // 持ちうる（headless 層 `trigger(disabled: bool, ...)`）ため、
        // `HoverExceptAttr("disabled")` で `:not([disabled])` を追加し
        // disabled 状態への hover 背景適用を防ぐ（CI codex-review P1 指摘、
        // イシュー #1522。上記モジュール doc「hover・トランジション」節・
        // 「disabled 視覚を付けない根拠（data-disabled 版）」節参照）。
        .state(
            "trigger",
            StateCondition::HoverExceptAttr("disabled"),
            hover_surface_declarations(),
        )
        // stage-trigger/close-trigger は headless 層が disabled 引数自体を
        // 持たないため、従来どおり `Hover` のままでよい。
        .state(
            "stage-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "close-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // キーボード操作時のみのフォーカスリング（イシュー #1424 canonical
        // ヘルパへ移行。`palette` 軸を持たないため `FocusRingColor::Token`）。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "stage-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "close-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
}

/// この styled FloatingPanel が生成する静的 CSS 全量を返す（決定的。
/// [`crate::popover::stylesheet`] と同じ契約）。
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
        assert!(a.contains(r#"[data-scope="floating-panel"][data-part="content"]"#));
        assert!(a.contains(r#"[data-scope="floating-panel"][data-part="trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn positioner_is_fixed_positioned_with_transform_translate() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="floating-panel"][data-part="positioner"]"#));
        assert!(css.contains("position: fixed;"));
        assert!(css.contains("translate3d(var(--fandhe-x, 24px), var(--fandhe-y, 24px), 0)"));
    }

    #[test]
    fn positioner_z_index_is_between_dropdown_and_modal_tiers() {
        let css = stylesheet();
        assert!(css.contains("z-index: 900;"));
    }

    #[test]
    fn stylesheet_links_data_state_to_content_visibility() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="floating-panel"][data-part="content"][data-state="closed"]"#
        ));
        assert!(css.contains("visibility: hidden;"));
    }

    #[test]
    fn stylesheet_links_data_stage_to_body_and_positioner() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="floating-panel"][data-part="body"][data-stage="minimized"]"#
        ));
        assert!(css.contains("display: none;"));
        assert!(css.contains(
            r#"[data-scope="floating-panel"][data-part="positioner"][data-stage="maximized"]"#
        ));
        assert!(css.contains("transform: none;"));
        assert!(css.contains("inset: 0;"));
    }

    #[test]
    fn trigger_stage_trigger_and_close_trigger_declare_focus_visible_ring() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="floating-panel"][data-part="trigger"]:focus-visible {"#)
        );
        assert!(css.contains(
            r#"[data-scope="floating-panel"][data-part="stage-trigger"]:focus-visible {"#
        ));
        assert!(css.contains(
            r#"[data-scope="floating-panel"][data-part="close-trigger"]:focus-visible {"#
        ));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
    }

    #[test]
    fn trigger_stage_trigger_and_close_trigger_declare_hover_surface() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        // `trigger` はネイティブ `disabled` 属性を持ちうる（headless 層
        // `trigger(disabled: bool, ...)`）ため、`:not([data-disabled])` に
        // 加えて `:not([disabled])` も除外条件へ含まれる（CI codex-review
        // P1 指摘、イシュー #1522。native-disabled 回帰テストは下記
        // `trigger_hover_excludes_native_disabled_attribute` 参照）。
        assert!(css.contains(
            r#"[data-scope="floating-panel"][data-part="trigger"]:hover:not([data-disabled]):not([disabled])"#
        ));
        // stage-trigger/close-trigger は headless 層が disabled 引数自体を
        // 持たないため、`data-disabled` 除外のみで従来どおり。
        assert!(css.contains(
            r#"[data-scope="floating-panel"][data-part="stage-trigger"]:hover:not([data-disabled])"#
        ));
        assert!(css.contains(
            r#"[data-scope="floating-panel"][data-part="close-trigger"]:hover:not([data-disabled])"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
        assert!(css.contains("--fandhe-hover-bg: var(--fandhe-color-bg-muted);"));
    }

    /// native-disabled（headless 層 `trigger(disabled: true, ...)` が出力する
    /// ネイティブ `disabled` 存在属性）回帰テスト（CI codex-review P1 指摘、
    /// イシュー #1522）。`trigger` の hover セレクタが `:not([disabled])` を
    /// 含まないと、操作不能な trigger にも hover 背景が適用され、disabled
    /// 状態なのに操作可能であるかのような誤った視覚フィードバックになる。
    #[test]
    fn trigger_hover_excludes_native_disabled_attribute() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="floating-panel"][data-part="trigger"]:hover:not([data-disabled]):not([disabled])"#
        ));
        // stage-trigger/close-trigger のセレクタには `:not([disabled])` が
        // 含まれないこと（disabled 引数自体を持たないため過剰な除外条件を
        // 追加しない）も併せて固定する。
        assert!(!css.contains(
            r#"[data-scope="floating-panel"][data-part="stage-trigger"]:hover:not([data-disabled]):not([disabled])"#
        ));
        assert!(!css.contains(
            r#"[data-scope="floating-panel"][data-part="close-trigger"]:hover:not([data-disabled]):not([disabled])"#
        ));
    }

    #[test]
    fn trigger_and_ghost_triggers_declare_transition() {
        let css = stylesheet();
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
        assert!(css.contains("transition-property: background, border-color;"));
        assert!(css.contains("transition-property: background;"));
    }

    #[test]
    fn content_expands_to_full_viewport_when_maximized() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="floating-panel"][data-part="content"][data-stage="maximized"]"#
        ));
        let rule_start = css
            .find(r#"[data-scope="floating-panel"][data-part="content"][data-stage="maximized"] {"#)
            .expect("content[data-stage=maximized] rule must be present");
        let rule_body = &css[rule_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        assert!(rule_body[..rule_end].contains("width: 100%;"));
        assert!(rule_body[..rule_end].contains("height: 100%;"));
        // `content` の `border`(1px)+`border-radius` により content-box のままだと
        // 全面化された `positioner` から縦横 2px はみ出すため、border-box で
        // 外寸を固定する（イシュー #1522、CI codex-review P1 / Cursor Bugbot 指摘）。
        assert!(rule_body[..rule_end].contains("box-sizing: border-box;"));
    }

    #[test]
    fn content_and_trigger_use_scale_tokens_for_radius_and_shadow() {
        let css = stylesheet();
        assert!(css.contains("border-radius: var(--fandhe-radius-lg);"));
        assert!(css.contains("box-shadow: var(--fandhe-shadow-md);"));
        assert!(css.contains("border-radius: var(--fandhe-radius-md, 0.375rem);"));
        assert!(css.contains("border-radius: var(--fandhe-radius-md);"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(OpenState::Closed, Stage::Default, vec![], vec![]));
        assert!(html.contains(r#"data-scope="floating-panel""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_floating_panel_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut p = FloatingPanel::default();
        assert_eq!(p.state(), OpenState::Closed);

        let ssr_html = render(&p.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut p, "open", ""));
        assert!(dispatch(&mut p, "maximize", ""));
        let hydrate_html = render(&render_for_hydration(&p));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));
        assert!(hydrate_html.contains(r#"data-hydrate-stage="maximized""#));

        let restored = FloatingPanel::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
        assert_eq!(restored.stage(), Stage::Maximized);
    }

    #[test]
    fn position_style_is_consumable_by_positioner_recipe_css_vars() {
        let p = FloatingPanel::new(OpenState::Open, Stage::Default, 100.0, 50.0);
        let style = p.position_style();
        let html = render(&p.positioner(vec![("style", &style)], vec![]));
        assert!(html.contains("--fandhe-x: 100px;"));
        assert!(html.contains("--fandhe-y: 50px;"));
    }
}

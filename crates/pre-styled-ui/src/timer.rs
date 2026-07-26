//! styled Timer（headless ラッパー、イシュー #836、親トラッキング #520）。
//!
//! `fandhe_frontend_headless_ui::timer`（イシュー #836）の Root / Area /
//! Item / ItemValue / ItemLabel / Separator / Control / ActionTrigger
//! 8 anatomy パーツと [`fandhe_frontend_headless_ui::timer::Timer`] 状態機械を
//! そのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::clipboard`] の rustdoc と同じ方針に
//! 従う（構造上最も近い先行例、変種なしの最小スコープ判断）。
//!
//! # セグメント表示のレイアウト
//!
//! `area` を横並び flex コンテナとし、`item` はセグメント値 + ラベルを縦積みで
//! 中央揃えする。`item-value` は `font-variant-numeric: tabular-nums` で
//! 桁の増減時の横幅ガタつき（レイアウトシフト）を防ぐ（時計表示の一般的な
//! ベストプラクティス）。`separator` は縦積みされた `item` 群の間に挟まる
//! 想定のため、`item` 自身の縦中央と揃うよう `align-self: center` を与える。
//!
//! # `data-state` に応じた見た目の切り替え
//!
//! `root` の `completed` 状態を控えめな強調色に切り替える
//! （[`crate::progress`]/[`crate::steps`] 等の完了表現と同じ判断）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - variant（size 等）ごとのクラス切り替えは他の styled 部品と同じく
//!   スコープ外とする。
//! - `setInterval` による実 tick 駆動・`navigator` 系 API 利用は
//!   `fandhe-frontend-wasm-full`（`headless_timer` モジュール）のスコープ。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。上記「本イシューの
// スコープ外」節のとおり variant 軸を持たず（規約 B-2）、CSS 到達は
// [data-scope]/[data-part] 属性セレクタのみに依存する（規約 B-3、イシュー
// #1062 規約参照）。
pub use fandhe_frontend_headless_ui::timer::*;

/// headless `timer` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/timer.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を出力しない
/// fail-closed 側の不具合として現れるため、変更時は両ファイルを合わせて
/// 確認する）。
const SLOTS: &[&str] = &[
    "root",
    "area",
    "item",
    "item-value",
    "item-label",
    "separator",
    "control",
    "action-trigger",
];

/// この styled Timer の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("timer", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("flex-direction", "column"),
            ],
        )
        .base(
            "area",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "row"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("align-items", "center"),
            ],
        )
        .base(
            "item-value",
            vec![
                decl("font-variant-numeric", "tabular-nums"),
                decl("font-size", "1.5rem"),
                decl("font-weight", "600"),
                decl("line-height", "1.2"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .base(
            "item-label",
            vec![
                decl("font-size", "0.75rem"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("text-transform", "uppercase"),
            ],
        )
        .base(
            "separator",
            vec![
                decl("font-size", "1.5rem"),
                decl("align-self", "center"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "control",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "row"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("margin-top", "var(--fandhe-space-2)"),
            ],
        )
        // 完了時は item-value を強調色へ切り替える（[`crate::progress`] 等の
        // 完了表現と同じ判断、モジュール doc 参照）。
        .state(
            "item-value",
            StateCondition::AttrEq("data-state", "completed"),
            vec![decl("color", "var(--fandhe-color-accent)")],
        )
        // キーボード操作時のみのフォーカスリング（[`crate::clipboard`]/
        // [`crate::tooltip`] 等の既存 trigger 系と同じ判断）。
        .state(
            "action-trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
}

/// この styled Timer が生成する静的 CSS 全量を返す（決定的。
/// [`crate::clipboard::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="timer"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn item_value_uses_tabular_nums_to_avoid_layout_shift() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="timer"][data-part="item-value"]"#));
        assert!(css.contains("font-variant-numeric: tabular-nums;"));
    }

    #[test]
    fn completed_state_switches_item_value_to_accent_color() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="timer"][data-part="item-value"][data-state="completed"]"#)
        );
    }

    #[test]
    fn action_trigger_declares_focus_visible_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="timer"][data-part="action-trigger"]:focus-visible {"#));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(
            true,
            10_000,
            0,
            1000,
            0,
            TimerPhase::Idle,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="timer""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_timer_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut t = Timer::countdown(2000, 500);
        assert_eq!(t.phase(), TimerPhase::Idle);

        assert!(dispatch(&mut t, "timer:start", ""));
        assert!(dispatch(&mut t, "timer:tick", "500"));

        let hydrate_html = render(&render_for_hydration(&t));
        assert!(hydrate_html.contains(r#"data-hydrate-phase="running""#));

        let restored = Timer::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }
}

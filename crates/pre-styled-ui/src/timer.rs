//! styled Timer（headless ラッパー、イシュー #836、親トラッキング #520）。
//!
//! `fandhe_frontend_headless_ui::timer`（イシュー #836）の Root / Area /
//! Item / ItemValue / ItemLabel / Separator / Control / ActionTrigger
//! 8 anatomy パーツと [`fandhe_frontend_headless_ui::timer::Timer`] 状態機械を
//! そのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::clipboard`] の rustdoc と同じ方針に
//! 従う（構造上最も近い先行例、変種なしの最小スコープ判断）。
//!
//! # 参考サイト基準への調整（イシュー #1577）
//!
//! Timer を持つ参考サイトは ark-ui のみ（chakra-ui / Radix Primitives /
//! Radix Themes には相当コンポーネントが無い）。`docs/design/
//! reference-screenshots/ark-timer-{1,2,3}.png` との比較により、以下を
//! 是正した:
//!
//! - `root` を中央揃え縦積みにし、`area`/`item` 間の余白をトークン化
//!   （旧実装は左寄せ・`control` の `margin-top` のみで余白を確保していた）。
//! - `item-value`/`item-label`/`separator` の生値（`1.5rem`/`600`/`1.2`
//!   等）をトークン参照へ置換した。
//! - `item-label` の `text-transform: uppercase` を廃止した（ark-ui は
//!   小文字のまま表示する）。
//! - `separator` の `align-self` を `flex-start`（値の行）へ揃えた（旧実装は
//!   `center` で値とラベルの中間に沈んでいた）。
//! - `action-trigger` を枠線・角丸・余白を持つ outline ボタン相当へ変更し、
//!   hover・transition を追加した（UA 既定の素の `<button>` のままだった
//!   最大の差分）。
//! - `action-trigger` の `:focus-visible` を [`crate::recipe::
//!   focus_ring_declarations`] の canonical 形へ統一した（手書き
//!   `outline: 2px solid ...` を廃止）。
//!
//! # セグメント表示のレイアウト
//!
//! `area` を横並び flex コンテナとし、`item` はセグメント値 + ラベルを縦積みで
//! 中央揃えする。`item-value` は `font-variant-numeric: tabular-nums` で
//! 桁の増減時の横幅ガタつき（レイアウトシフト）を防ぐ（時計表示の一般的な
//! ベストプラクティス）。`separator` は横並びの `area` 内で `item` 群の間に
//! 挟まる想定のため、値の行（`item-value` の基準線）に揃うよう
//! `align-self: flex-start` を与える。
//!
//! # `data-state` に応じた見た目の切り替え（root 経由の間接参照）
//!
//! [`SlotRecipe`] は子孫セレクタ機構を持たない（`recipe.rs` 参照、#708 で
//! 追加しないと確定）ため、`root` の `completed` 状態を `item-value` の
//! 色へ伝えるには custom property の間接参照を使う（[`crate::table`] の
//! `--fandhe-table-stripe-bg`・[`crate::qr_code`]・[`crate::splitter`] と
//! 同型）。`root[data-state="completed"]` が `--fandhe-timer-value-color` を
//! 強調色へ再定義し、`item-value` の base 規則がこの変数をフォールバック
//! 付きで参照する。旧実装は `item-value[data-state="completed"]` を
//! 直接参照していたが、headless 側（`crates/headless-ui/src/timer.rs`）は
//! `data-state` を root にしか出力しないため、この規則は描画された HTML に
//! 一度もマッチしない欠陥だった（本イシューで是正）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **size（variant）軸**: 唯一の参照元 ark-ui は unstyled で size prop を
//!   持たない。glob 再エクスポート規約（B-2、`REEXPORT-GLOB-REVIEWED`
//!   コメント参照）を維持し、本イシューでは追加しない。size 導入には
//!   styled `root` の新設が必要で、headless 自由関数 `root` /
//!   `Timer::root` の双方と衝突する公開 API 変更（minor バンプ・選択的
//!   re-export 化を伴う）になるため、必要になった時点で別イシューとして
//!   起票する（`docs/design/
//!   pre-styled-ui-focus-ring-and-size-conventions.md` §4 が timer を
//!   個別 issue で確定するとしている (b) 候補に分類）。
//! - **disabled 視覚**: headless `action_trigger`（`crates/headless-ui/
//!   src/timer.rs`）が `data-disabled` を出力せず、
//!   `fandhe-frontend-wasm-full` 側も付け外ししないため、CSS 側だけでは
//!   表現できない（[`crate::clipboard`] と同型のギャップ）。headless 側の
//!   属性追加は別途 Issue 提案とする。
//! - `setInterval` による実 tick 駆動・`navigator` 系 API 利用は
//!   `fandhe-frontend-wasm-full`（`headless_timer` モジュール）のスコープ。

use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, hover_bg_muted, hover_surface_declarations, transition_declarations,
    FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe, StateCondition,
};

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
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-4)"),
            ],
        )
        .base(
            "area",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "row"),
                decl("align-items", "flex-start"),
                decl("gap", "var(--fandhe-space-4)"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "item-value",
            vec![
                decl("font-variant-numeric", "tabular-nums"),
                decl("font-size", "var(--fandhe-font-font-size-2xl)"),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("line-height", "var(--fandhe-font-line-height-tight)"),
                decl(
                    "color",
                    "var(--fandhe-timer-value-color, var(--fandhe-color-fg))",
                ),
            ],
        )
        .base(
            "item-label",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "separator",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-2xl)"),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("line-height", "var(--fandhe-font-line-height-tight)"),
                decl("align-self", "flex-start"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "control",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "row"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        // outline ボタン相当（[`crate::button`] の `recipe_with_scope` は
        // class ベースの size/variant セレクタを出し glob 規約 B-2/B-3 に
        // 反するため使わず、宣言を直接並べる。モジュール doc「参考サイト
        // 基準への調整」節参照）。
        .base(
            "action-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("box-sizing", "border-box"),
                decl("min-height", "var(--fandhe-size-control-height-sm)"),
                decl("padding", "0 var(--fandhe-size-control-padding-x-sm)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-family", "var(--fandhe-font-font-body)"),
                decl("font-size", "var(--fandhe-size-control-font-size-sm)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("cursor", "pointer"),
                hover_bg_muted(),
            ]
            .into_iter()
            .chain(transition_declarations(
                "background, border-color, color",
                MotionDuration::Fast,
            ))
            .collect(),
        )
        // 完了時は item-value を強調色へ切り替える（root 経由の間接参照、
        // モジュール doc「`data-state` に応じた見た目の切り替え」節参照。
        // [`crate::progress`] 等の完了表現と同じ判断）。
        .state(
            "root",
            StateCondition::AttrEq("data-state", "completed"),
            vec![decl(
                "--fandhe-timer-value-color",
                "var(--fandhe-color-accent)",
            )],
        )
        .state(
            "action-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // キーボード操作時のみのフォーカスリング（[`crate::clipboard`]/
        // [`crate::tooltip`] 等の既存 trigger 系と同じ判断、イシュー #1424
        // canonical 形。palette 軸を持たず root は `overflow: hidden` を
        // 持たないため `FocusRingColor::Token` + `FocusRingOffset::Outside`）。
        .state(
            "action-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
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
    fn completed_state_switches_root_custom_property_consumed_by_item_value() {
        let css = stylesheet();
        // root の completed 状態が変数を定義する（headless 側は data-state
        // を root にしか出さないため、root へのマッチが必須）。
        assert!(css.contains(r#"[data-scope="timer"][data-part="root"][data-state="completed"] {"#));
        assert!(css.contains("--fandhe-timer-value-color: var(--fandhe-color-accent);"));
        // item-value の base 規則がフォールバック付きで同変数を参照する。
        assert!(css.contains("color: var(--fandhe-timer-value-color, var(--fandhe-color-fg));"));
    }

    #[test]
    fn item_label_is_not_uppercased() {
        let css = stylesheet();
        assert!(!css.contains("text-transform"));
    }

    #[test]
    fn action_trigger_declares_hover_inside_hover_media_query() {
        let css = stylesheet();
        let media_start = css
            .find("@media (hover: hover)")
            .expect("hover 宣言は @media (hover: hover) へ集約される");
        let hover_block = &css[media_start..];
        assert!(hover_block.contains(r#"[data-scope="timer"][data-part="action-trigger"]:hover"#));
        assert!(hover_block.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn action_trigger_uses_transition_tokens() {
        let css = stylesheet();
        assert!(css.contains("transition-property: background, border-color, color;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
    }

    #[test]
    fn action_trigger_declares_focus_visible_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="timer"][data-part="action-trigger"]:focus-visible {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
    }

    #[test]
    fn stylesheet_has_no_raw_color_literals() {
        let css = stylesheet();
        assert!(!css.contains('#'));
        assert!(!css.contains("rgb("));
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

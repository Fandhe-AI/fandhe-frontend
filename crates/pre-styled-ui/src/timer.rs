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
//! ベストプラクティス）。`separator` は `item` と兄弟関係で area 内に並ぶ
//! ため、`item-value` と同じ font-size/font-weight/line-height を与えて
//! 数字の行へ揃える（下記「参考サイト基準への調整」節）。
//!
//! # 参考サイト基準への調整（イシュー #1577）
//!
//! 参照サイト（ark-ui の Timer のみ。chakra-ui v3 / Radix Themes / Radix
//! Primitives に Timer は存在しない、
//! `docs/design/reference-screenshots/ark-timer-1..3.png`）と比較し、以下を
//! 是正した。
//!
//! - **値のタイポ**: 生リテラル（`font-size: 1.5rem` / `font-weight: 600` /
//!   `line-height: 1.2`）をトークン（`--fandhe-font-font-size-2xl` /
//!   `-font-weight-semibold` / `-line-height-tight`）へ載せ替えた。利用者が
//!   `item-value` の寸法を上書きできるよう `--fandhe-timer-value-font-size`
//!   custom property を介す（[`crate::stat`] の
//!   `--fandhe-stat-value-font-size` と同型）。`separator` も同じ変数を
//!   参照し常に値と揃う。
//! - **ラベル**: 参照サイトは小文字表記（"minutes" / "seconds"）のため
//!   `text-transform: uppercase` を撤去し、フォントサイズもトークン化した。
//! - **separator の位置**: 旧 `align-self: center`（`item` 全体＝値+ラベルの
//!   縦積みの中央）は値とラベルの間に落ちて数字の行とずれていた。
//!   `align-self: flex-start` へ変更し、値と同じ font-size/line-height で
//!   数字の行に揃える。
//! - **間隔**: `area` の `gap` を `space-2` から `space-4`（参照サイトの
//!   広めの項目間隔）へ、`control` の `margin-top` も `space-2` から
//!   `space-4` へ。
//! - **action-trigger**: 参照サイトの outline 風ボタン（枠線・角丸・余白・
//!   hover 面変化）に合わせ、共通ビジュアル言語
//!   （[`crate::recipe`] の `hover_bg_muted`/`hover_surface_declarations`/
//!   `disabled_declarations`/`focus_ring_declarations`/
//!   `transition_declarations`）とスケールトークン
//!   （`docs/design/pre-styled-ui-scale-tokens.md` の `control-height-sm`/
//!   `control-padding-x-sm`）へ載せ替えた。旧 `outline` 直書き 2 宣言も
//!   `focus_ring_declarations(FocusRingColor::Token,
//!   FocusRingOffset::Outside)` へ canonical 化した。
//!
//! ## dead CSS の是正（発見した実不具合）
//!
//! 旧 `.state("item-value", StateCondition::AttrEq("data-state",
//! "completed"), ...)` は一度もマッチしなかった。headless
//! `crates/headless-ui/src/timer.rs` は `data-state` を **`root` にのみ**
//! 出力し、`item`/`item-value`/`item-label` は `data-type` しか持たない。
//! wasm 配線 `crates/wasm-full/src/headless_timer.rs` も `root` の
//! `data-state`/`data-elapsed` のみを書き換え、`item-value` は本文更新
//! （`set_text_content`）のみで属性は動かさない。`SlotRecipe` は子孫
//! セレクタ機構を持たない（`crate::recipe` rustdoc、#708 で「追加しない」と
//! 確定）ため、`item-value` 側に `data-state` 条件を書いても構造的に
//! マッチし得ない。本イシューで `root[data-state]` → custom property
//! （`--fandhe-timer-value-color`）間接参照へ是正した（下記「`data-state` に
//! 応じた見た目の切り替え」節）。
//!
//! # `data-state` に応じた見た目の切り替え
//!
//! `root` の `completed`/`paused` 状態に応じて `--fandhe-timer-value-color`
//! custom property を切り替え、`item-value` の `color` がそれを
//! （`--fandhe-color-fg` へのフォールバック付きで）消費する
//! （[`crate::table`] の `--fandhe-table-stripe-bg`、[`crate::progress`] の
//! `--fandhe-progress-*` と同型の間接参照パターン）。`completed` は控えめな
//! 強調色（[`crate::progress`]/[`crate::steps`] 等の完了表現と同じ判断）、
//! `paused` は「停止中」を示す muted 色とする。`idle`/`running` は既定色の
//! まま（参照サイトも状態色差なし）。この規則は headless 層・wasm 層を
//! 一切変更しない（`data-*` 語彙は不変）。
//!
//! ## 意図的に参考サイトへ合わせない点
//!
//! 1. **size / variant 軸の非提供**: `timer` は
//!    `crates/pre-styled-ui/src/lib.rs` の規約 B 許可リストに
//!    「`stylesheet()` のみ・variant 軸なし・属性セレクタのみ」として
//!    登録済みであり、参照サイト（ark-ui は unstyled、chakra-ui v3 /
//!    Radix Themes / Radix Primitives に Timer なし）にも size/variant
//!    スケールが存在しないため変えない。size 軸を足すには規約 A への移行
//!    （`pub use ...::*` の撤去・状態機械型 `Timer` の再エクスポート撤去）が
//!    必要になり、`crates/docs-site/src/showcase.rs::timer_section`/
//!    `component_page_specs_948.rs::timer_example` が
//!    `Timer::countdown` + `dispatch` に依存しているため壊れる
//!    （[`crate::toggle_tip`] と同じ構成の判断）。
//! 2. **action-trigger のアイコン非同梱**: 参照サイトの Play 三角等の
//!    アイコンは children として呼び出し側が渡す契約であり、本モジュールは
//!    枠線・パディング・hover のみを CSS で提供する。
//! 3. **idle / running の色差なし**: 参照サイトも状態色差を持たないため。
//! 4. **`action-trigger[data-disabled]` は呼び出し側付与時のみ有効**:
//!    headless `timer::action_trigger` は `data-disabled` を出力しない
//!    （現状 grep 0 件）。CSS 規則自体は
//!    （[`crate::button`] の `recipe_with_scope` の先例に倣い）用意するが、
//!    実際に見た目へ反映されるのは呼び出し側が `attrs` で
//!    `("data-disabled", "")` を渡した場合のみ。headless 側への出力追加は
//!    別イシュー相当としてスコープ外（下記節参照）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - variant（size 等）ごとのクラス切り替えは他の styled 部品と同じく
//!   スコープ外とする（上記「意図的に参考サイトへ合わせない点」1 参照）。
//! - `setInterval` による実 tick 駆動・`navigator` 系 API 利用は
//!   `fandhe-frontend-wasm-full`（`headless_timer` モジュール）のスコープ。
//! - headless `timer::action_trigger` への `data-disabled` 出力追加
//!   （例: idle 時の Pause を無効化する状態連動）。本 PR では CSS 規則のみ
//!   用意し、属性付与は呼び出し側に委ねる（上記「意図的に参考サイトへ
//!   合わせない点」4 参照）。

use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe,
    StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。上記「意図的に参考
// サイトへ合わせない点」1 のとおり variant 軸を持たず（規約 B-2）、CSS
// 到達は [data-scope]/[data-part] 属性セレクタのみに依存する（規約 B-3、
// イシュー #1062 規約参照）。
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
                // control が area の幅へ引き伸ばされないように（イシュー
                // #1577、参考サイトは表示部と同じ幅までしかボタンが広がら
                // ない）。
                decl("align-items", "flex-start"),
            ],
        )
        .base(
            "area",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "row"),
                decl("align-items", "center"),
                // §「参考サイト基準への調整」: 参照サイトの広めの項目間隔。
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
            [
                vec![
                    decl("font-variant-numeric", "tabular-nums"),
                    decl(
                        "font-size",
                        "var(--fandhe-timer-value-font-size, var(--fandhe-font-font-size-2xl))",
                    ),
                    decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                    decl("line-height", "var(--fandhe-font-line-height-tight)"),
                    // §「data-state に応じた見た目の切り替え」: root の
                    // completed/paused が定義する custom property を
                    // フォールバック付きで消費する間接参照。
                    decl(
                        "color",
                        "var(--fandhe-timer-value-color, var(--fandhe-color-fg))",
                    ),
                ],
                transition_declarations("color", MotionDuration::Fast),
            ]
            .concat(),
        )
        .base(
            "item-label",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "separator",
            vec![
                // item（値+ラベルの縦積み）の中央ではなく数字の行に揃える。
                decl("align-self", "flex-start"),
                decl(
                    "font-size",
                    "var(--fandhe-timer-value-font-size, var(--fandhe-font-font-size-2xl))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("line-height", "var(--fandhe-font-line-height-tight)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "control",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "row"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("margin-top", "var(--fandhe-space-4)"),
            ],
        )
        // §「参考サイト基準への調整」: outline 風ボタンへ整える。
        .base(
            "action-trigger",
            [
                vec![
                    decl("cursor", "pointer"),
                    decl("display", "inline-flex"),
                    decl("align-items", "center"),
                    decl("justify-content", "center"),
                    decl("gap", "var(--fandhe-space-2)"),
                    decl(
                        "min-height",
                        "var(--fandhe-size-control-height-sm, 2.25rem)",
                    ),
                    decl(
                        "padding",
                        "0 var(--fandhe-size-control-padding-x-sm, 0.75rem)",
                    ),
                    decl("font-size", "var(--fandhe-font-font-size-sm)"),
                    decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                    decl("color", "var(--fandhe-color-fg)"),
                    decl("background", "var(--fandhe-color-bg)"),
                    decl("border", "1px solid var(--fandhe-color-border)"),
                    decl("border-radius", "var(--fandhe-radius-md)"),
                    hover_bg_muted(),
                ],
                transition_declarations("background, color, border-color", MotionDuration::Fast),
            ]
            .concat(),
        )
        // §「data-state に応じた見た目の切り替え」: dead CSS だった旧
        // item-value 直接条件を撤去し、root → custom property の間接参照へ
        // 是正（モジュール doc「dead CSS の是正」節参照）。
        .state(
            "root",
            StateCondition::AttrEq("data-state", "completed"),
            vec![decl(
                "--fandhe-timer-value-color",
                "var(--fandhe-color-accent)",
            )],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-state", "paused"),
            vec![decl(
                "--fandhe-timer-value-color",
                "var(--fandhe-color-fg-muted)",
            )],
        )
        // disabled 時の視覚フィードバック（headless が出力するのは呼び出し
        // 側が `attrs` で付与した場合のみ、モジュール doc「意図的に参考
        // サイトへ合わせない点」4 参照）。
        .state(
            "action-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // キーボード操作時のみのフォーカスリング（[`crate::clipboard`]/
        // [`crate::tooltip`] 等の既存 trigger 系と同じ判断）。
        .state(
            "action-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // hover 時の面変化（マウス操作系のみ、`@media (hover: hover)`
        // 配下）。
        .state(
            "action-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // イシュー #1632: headless `timer::action_trigger` が可視性導出の
        // ため `hidden` 属性を出力するようになった（zag.js
        // `getTriggerProps` の真偽式と同型）。base 規則が `display:
        // inline-flex` を宣言しており UA 既定の `[hidden] { display: none }`
        // を上書きしてしまうため、`action_bar.rs` の `positioner[hidden]`
        // と同じ理由でより詳細度の高い `[hidden]` 属性セレクタで明示的に
        // 上書きする。
        .state(
            "action-trigger",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
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
    fn root_completed_state_defines_value_color_custom_property() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="timer"][data-part="root"][data-state="completed"]"#));
        assert!(css.contains("--fandhe-timer-value-color: var(--fandhe-color-accent);"));
        // dead CSS だった旧セレクタは登場しない（是正の直接検証）。
        assert!(!css.contains(r#"[data-part="item-value"][data-state="completed"]"#));
    }

    #[test]
    fn root_paused_state_defines_value_color_custom_property() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="timer"][data-part="root"][data-state="paused"]"#));
        assert!(css.contains("--fandhe-timer-value-color: var(--fandhe-color-fg-muted);"));
    }

    #[test]
    fn item_value_consumes_value_color_custom_property_with_fg_fallback() {
        let css = stylesheet();
        assert!(css.contains("color: var(--fandhe-timer-value-color, var(--fandhe-color-fg));"));
    }

    #[test]
    fn action_trigger_declares_hover_gated_by_hover_media_and_not_disabled() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(
            r#"[data-scope="timer"][data-part="action-trigger"]:hover:not([data-disabled]) {"#
        ));
    }

    #[test]
    fn action_trigger_declares_canonical_focus_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="timer"][data-part="action-trigger"]:focus-visible {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
    }

    #[test]
    fn action_trigger_declares_disabled_visual() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="timer"][data-part="action-trigger"][data-disabled] {"#)
        );
    }

    #[test]
    fn no_raw_typography_literals_remain() {
        let css = stylesheet();
        assert!(!css.contains("font-size: 1.5rem;"));
        assert!(!css.contains("font-weight: 600;"));
        assert!(!css.contains("line-height: 1.2;"));
        assert!(!css.contains("text-transform: uppercase;"));
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

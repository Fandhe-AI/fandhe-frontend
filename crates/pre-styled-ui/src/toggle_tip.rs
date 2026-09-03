//! styled ToggleTip（headless ラッパー、イシュー #761、親 #520/#546）。
//!
//! `fandhe_frontend_headless_ui::toggle_tip`（イシュー #761）の Root /
//! Trigger / Positioner / Content / Arrow / ArrowTip 6 anatomy パーツと
//! [`fandhe_frontend_headless_ui::toggle_tip::ToggleTip`] 状態機械をそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠・スコープ外事項は [`crate::tooltip`]/[`crate::popover`] の rustdoc と
//! 同じ方針に従う。
//!
//! # 「見た目は Tooltip」の recipe レベルでの反映
//!
//! headless 層の rustdoc（`crates/headless-ui/src/toggle_tip.rs`）が示す
//! 「見た目は Tooltip・挙動は Popover」の要件のうち、見た目の部分は本
//! モジュールの [`recipe`] が担う。`content` の視覚系（背景色・文字色・
//! フォントサイズ・角丸・パディング・最大幅）は [`crate::tooltip`] の
//! `content` と同一の値を使う（イシュー #1548 が同時進行中のため、両者の
//! 乖離を作らないよう `font-size` を含め値を揃え続ける）。
//!
//! # data-state とスタイルの連動
//!
//! `content` の開閉 `data-state`（open/closed）に応じた見た目の切り替えを
//! [`recipe`] へ登録する（[`crate::recipe::SlotRecipe::state`]、
//! [`crate::tooltip`] と同型）。イシュー #1546 で `trigger` 側の
//! `data-state="open"` にも見た目の切り替えを追加した（下記「イシュー
//! #1546 の参照サイト比較」節参照）。
//!
//! # キーボード操作系属性の反映
//!
//! `trigger` はフォーカス可能なボタン要素であり、キーボード操作時のみの
//! フォーカスリング（`:focus-visible`）を
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::tooltip`]/[`crate::popover`] と同じ判断）。
//!
//! # positioner のオーバーレイ配置と既定表示位置
//!
//! headless 側 `root`（`crates/headless-ui/src/toggle_tip.rs`）の子として
//! `trigger`/`positioner` が並置される兄弟関係のため、containing block を
//! 提供する `position: relative` は共通祖先の `root` に付与する
//! （[`crate::tooltip`]/[`crate::popover`]/[`crate::menu`] と同じ判断）。
//! [`crate::tooltip`] と同じ層（`z-index: var(--fandhe-z-index-popover,
//! 1100)`）に配置し、トリガー要素の上側に表示する既定位置
//! （`position: absolute; bottom: 100%; left: 0`）を採る。`positioner` は
//! base 規則で `display` を宣言しないため、closed 時に headless 層が付与する
//! `hidden` 存在属性は UA 既定 `[hidden] { display: none }` がそのまま機能
//! する（[`crate::tooltip`]/[`crate::popover`] と同じ構造的な回避、dialog で
//! 発生した PR #575 Bugbot 指摘（High）と同種の不具合を避ける）。
//!
//! # `content` は `--fandhe-reference-width` を消費しない
//!
//! [`crate::tooltip`] と同じ判断（モジュール doc 参照）で、短いテキスト
//! 内容へ幅が追随すべき `content` に `sameWidth` 相当のスタイルは適用しない。
//!
//! # イシュー #1546 の参照サイト比較（7 軸チェック）
//!
//! 参照サイト（chakra-ui の ToggleTip/InfoTip = ghost アイコンボタン +
//! 小型パネル、`docs/design/reference-screenshots/chakra-toggle-tip-*.png`）
//! と比較し、共通ビジュアル言語（`crate::recipe` の
//! `focus_ring_declarations`/`disabled_declarations`/
//! `hover_surface_declarations`/`transition_declarations`）とスケール
//! トークン（`docs/design/pre-styled-ui-scale-tokens.md`）へ載せ替えた。
//!
//! - **サイズ / バリアント**: 合わせない。オーバーレイ配置系の
//!   [`crate::popover`]/[`crate::hover_card`] と同じく size/variant 軸を
//!   提供しない方針を維持する（下記スコープ外節参照）。
//! - **色**: 生色リテラルなし。新設した `content` の影も
//!   `var(--fandhe-shadow-sm)` トークン参照のみでフォールバックに生色を
//!   持ち込まない。
//! - **`data-*` 状態**: headless 層が `trigger` へ出力する `data-state`
//!   （open/closed）・`data-disabled` を初めて視覚へ反映する
//!   （`trigger[data-state="open"]` の背景/文字色反転、
//!   `trigger[data-disabled]` の `disabled_declarations()`）。
//! - **ダーク**: 反転色（`--fandhe-color-fg`/`-bg`）で自動追従する既存挙動を
//!   維持。新設の影もダーク値内蔵トークン経由。
//! - **フォーカス**: `outline` 直書き 2 宣言を
//!   `focus_ring_declarations(FocusRingColor::Token,
//!   FocusRingOffset::Outside)` へ canonical 化した（palette 軸なし）。
//! - **余白・角丸・影**: `positioner` の `z-index` を
//!   `var(--fandhe-z-index-popover, 1100)`（旧来値 1100 を fallback に据える）
//!   へ、`content` の `border-radius` を `var(--fandhe-radius-sm, 0.25rem)`
//!   （tooltip と同じ「密なインライン部品 = sm」段）へトークン化し、
//!   `content` に `box-shadow: var(--fandhe-shadow-sm)` を新設した。
//! - **hover / disabled / トランジション**: `trigger` を参考サイト同様の
//!   ghost ボタン（`background: transparent`・`border: none`・
//!   `color: var(--fandhe-color-fg-muted)`・`radius-sm`・
//!   `padding: var(--fandhe-space-1)`）へ整え、`hover_bg_muted()` +
//!   `StateCondition::Hover` → `hover_surface_declarations()` と
//!   `transition_declarations("background, color", MotionDuration::Fast)`
//!   を新設した。`data-disabled` には `disabled_declarations()` を登録する。
//!
//! ## 意図的に参考サイトへ合わせない点
//!
//! 1. `content` の配色を chakra の panel 色（非反転）へ寄せない
//!    （headless 層 rustdoc が定める「見た目は Tooltip・挙動は Popover」
//!    契約に従い、[`crate::tooltip`] の `content` と同一値を維持する）。
//! 2. size / variant 軸の追加（上記「サイズ / バリアント」参照）。
//! 3. `content` の開閉フェード演出（headless 層が closed 時に `hidden` を
//!    即座に付与するライフサイクルのため描画されない既知の未解決事項、
//!    [`crate::popover`]/[`crate::hover_card`] と同じ判断）。
//! 4. `--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`（座標ジオメトリ）は従来
//!    どおり対象外。
//! 5. `arrow`/`arrow-tip` への装飾追加（座標ジオメトリ依存のため対象外を
//!    継続）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - variant（size 等）ごとのクラス切り替えは他の headless ラッパーと同じく
//!   スコープ外とする。
//! - click-outside dismiss・Escape 閉鎖は headless 層のドキュメント
//!   （`crates/headless-ui/src/toggle_tip.rs`）で既にスコープ外と明記済みの
//!   クライアントサイド実行時挙動であり、本モジュールもそれを継承する。
//! - `--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`（座標ジオメトリ）は
//!   [`crate::tooltip`]/[`crate::menu`]/[`crate::popover`] と同じ理由で本
//!   イシューの対象外。
//! - `content`/`positioner` の開閉フェード演出（上記「意図的に参考サイトへ
//!   合わせない点」3 参照）。
//! - showcase Demo の trigger をアイコン風に変える表示変更（利用者向け
//!   説明に影響しないため今回は据え置き）。

use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe,
    StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。オーバーレイ配置系の
// [`crate::tooltip`] と同じ判断で variant 軸を提供せず（規約 B-2）、CSS
// 到達は [data-scope]/[data-part] 属性セレクタのみに依存する（規約 B-3、
// イシュー #1062 規約参照）。
pub use fandhe_frontend_headless_ui::toggle_tip::*;
// `root`/`trigger` 等の `state` 引数・`ToggleTip::new`・`ToggleTip` の
// `Component::Action`（dispatch 対象）はいずれも `state` モジュール由来で
// 上記 glob 再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（イシュー #685 の方針、[`crate::tooltip`] と同型）。
pub use fandhe_frontend_headless_ui::state::{DisclosureAction, OpenState};

/// headless `toggle_tip` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/toggle_tip.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を出力しない
/// fail-closed 側の不具合として現れるため、変更時は両ファイルを合わせて
/// 確認する）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "positioner",
    "content",
    "arrow",
    "arrow-tip",
];

/// この styled ToggleTip の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("toggle-tip", SLOTS)
        .base("root", vec![decl("position", "relative")])
        // イシュー #1546: 参考サイト（chakra InfoTip）と同じ ghost アイコン
        // ボタン相当の見た目へ整える（モジュール doc「参照サイト比較」節）。
        .base(
            "trigger",
            [
                vec![
                    decl("cursor", "pointer"),
                    decl("display", "inline-flex"),
                    decl("align-items", "center"),
                    decl("justify-content", "center"),
                    decl("background", "transparent"),
                    decl("color", "var(--fandhe-color-fg-muted)"),
                    decl("border", "none"),
                    decl("border-radius", "var(--fandhe-radius-sm)"),
                    decl("padding", "var(--fandhe-space-1)"),
                    hover_bg_muted(),
                ],
                transition_declarations("background, color", MotionDuration::Fast),
            ]
            .concat(),
        )
        .base(
            "positioner",
            vec![
                decl("position", "absolute"),
                decl("bottom", "100%"),
                decl("left", "0"),
                // §3.4: toggle-tip は popover 段。旧来値 1100 を fallback に据える。
                decl("z-index", "var(--fandhe-z-index-popover, 1100)"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("background", "var(--fandhe-color-fg)"),
                decl("color", "var(--fandhe-color-bg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("border-radius", "var(--fandhe-radius-sm, 0.25rem)"),
                // §3.2「tooltip = sm」。新設宣言のため fallback を持たない
                // （トークン未定義時は宣言が無効化され従来どおり影なしになる）。
                decl("box-shadow", "var(--fandhe-shadow-sm)"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
                decl("max-width", "20rem"),
            ],
        )
        // trigger の開状態に応じた見た目の切り替え（イシュー #1546）。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![
                decl("background", "var(--fandhe-color-bg-muted)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        // content の開閉状態に応じた見た目の切り替え（[`crate::tooltip`] と同型）。
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("visibility", "hidden")],
        )
        // disabled 時の視覚フィードバック（イシュー #1546）。
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // キーボード操作時のみのフォーカスリング。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // hover 時の視覚フィードバック（イシュー #1546）。
        .state(
            "trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
}

/// この styled ToggleTip が生成する静的 CSS 全量を返す（決定的。
/// [`crate::tooltip::stylesheet`] と同じ契約）。
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
        assert!(a.contains(r#"[data-scope="toggle-tip"][data-part="content"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn positioner_is_absolutely_positioned_for_overlay() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toggle-tip"][data-part="positioner"]"#));
        assert!(css.contains("position: absolute;"));
        assert!(css.contains("bottom: 100%;"));
    }

    #[test]
    fn root_provides_containing_block_for_positioner() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"toggle-tip\"][data-part=\"root\"] {\n  position: relative;\n}\n"
        ));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="toggle-tip"][data-part="content"][data-state="closed"]"#)
        );
    }

    #[test]
    fn trigger_declares_focus_visible_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toggle-tip"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
    }

    #[test]
    fn content_does_not_consume_reference_width_css_var() {
        // [`crate::tooltip`] と同じ判断: toggle-tip の content はテキスト
        // 内容へ幅が追随すべきであり、sameWidth 相当は不適切なため意図的に
        // --fandhe-reference-width を消費しないことを固定する。
        let css = stylesheet();
        assert!(!css.contains("--fandhe-reference-width"));
    }

    #[test]
    fn trigger_hover_surface_is_gated_by_hover_media_and_not_disabled() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover) {"));
        assert!(css.contains(
            r#"[data-scope="toggle-tip"][data-part="trigger"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn trigger_declares_disabled_visual() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toggle-tip"][data-part="trigger"][data-disabled] {"#));
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn trigger_open_state_is_visually_distinct() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="toggle-tip"][data-part="trigger"][data-state="open"] {"#)
        );
        assert!(css.contains("background: var(--fandhe-color-bg-muted);"));
        assert!(css.contains("color: var(--fandhe-color-fg);"));
    }

    #[test]
    fn positioner_z_index_uses_popover_layer_token_with_legacy_fallback() {
        let css = stylesheet();
        assert!(css.contains("z-index: var(--fandhe-z-index-popover, 1100);"));
    }

    #[test]
    fn content_radius_and_shadow_use_scale_tokens() {
        let css = stylesheet();
        assert!(css.contains("border-radius: var(--fandhe-radius-sm, 0.25rem);"));
        assert!(css.contains("box-shadow: var(--fandhe-shadow-sm);"));
    }

    #[test]
    fn stylesheet_has_no_raw_color_literals() {
        let css = stylesheet();
        assert!(!css.contains('#'));
        assert!(!css.contains("rgb("));
        assert!(!css.contains("rgba("));
    }

    #[test]
    fn trigger_declares_fast_transition() {
        let css = stylesheet();
        assert!(css.contains("transition-property: background, color;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="toggle-tip""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_toggle_tip_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut t = ToggleTip::default();
        assert_eq!(t.state(), OpenState::Closed);

        let ssr_html = render(&t.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut t, "open", ""));
        let hydrate_html = render(&render_for_hydration(&t));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        let restored = ToggleTip::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}

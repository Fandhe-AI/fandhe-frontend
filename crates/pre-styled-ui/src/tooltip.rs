//! styled Tooltip（headless ラッパー第 2 弾、イシュー #664、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::tooltip`（イシュー #533）の Root / Trigger /
//! Positioner / Content / Arrow / ArrowTip 6 anatomy パーツと
//! [`fandhe_frontend_headless_ui::tooltip::Tooltip`] 状態機械をそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠・スコープ外事項は [`crate::dialog`]/[`crate::popover`] の rustdoc と
//! 同じ方針に従う。
//!
//! # data-state とスタイルの連動（イシュー #664 受け入れ条件）
//!
//! `content` の開閉 `data-state`（open/closed）に応じた見た目の切り替えを
//! [`recipe`] へ登録する（[`crate::recipe::SlotRecipe::state`]）。
//!
//! # キーボード操作系属性の反映
//!
//! `trigger` はフォーカス可能なボタン要素であり、キーボード操作時のみの
//! フォーカスリング（`:focus-visible`）を
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::dialog`]/[`crate::popover`] と同じ判断）。
//!
//! # positioner のオーバーレイ配置と既定表示位置
//!
//! headless 側 `root`（`crates/headless-ui/src/tooltip.rs`）の子として
//! `trigger`/`positioner` が並置される兄弟関係のため、containing block を
//! 提供する `position: relative` は共通祖先の `root` に付与する
//! （[`crate::popover`]/[`crate::menu`] と同じ判断）。tooltip は一般的に
//! トリガー要素の上側に表示するため `positioner` は
//! `position: absolute; bottom: 100%; left: 0` とする。`z-index` は
//! `docs/design/pre-styled-ui-scale-tokens.md` §3.4 の割り当てに従い
//! `var(--fandhe-z-index-tooltip, 1100)` を用いる（tooltip は他のオーバーレイ
//! の上にも表示されうる補助的な説明であるため、`dialog` の `positioner`
//! （1001）より前面に来る段。旧来値 1100 を fallback に据え、`stylesheet()`
//! 単独利用者でテーマ CSS 未注入でも宣言が無効化されない後方互換方針、
//! `hover_card`/`popover`/`toggle_tip` と同じ）。`positioner` は base 規則で
//! `display` を宣言しないため、closed 時に headless 層が付与する `hidden`
//! 存在属性は UA 既定 `[hidden] { display: none }` がそのまま機能する
//! （[`crate::popover`] と同じ構造的な回避、dialog で発生した PR #575
//! Bugbot 指摘（High）と同種の不具合を避ける）。
//!
//! # `content` は `--fandhe-reference-width` を消費しない（イシュー #664 受け入れ条件 2）
//!
//! [`crate::menu`]/[`crate::select`]/[`crate::popover`] の `content` は
//! トリガー実測幅へ追随する `sameWidth` 相当のスタイルを持つが、tooltip の
//! `content` は短いテキスト内容へ幅が追随すべきであり、`sameWidth` 相当は
//! 用途として不適切なため意図的に `--fandhe-reference-width` を消費しない
//! （既存の CSS 変数規約自体には反しない選択であることをここに明記する）。
//!
//! # イシュー #1548 の参照サイト比較（7 軸チェック）
//!
//! 参照サイト（chakra-ui / Radix Themes / Radix Primitives / ark-ui、
//! `docs/design/reference-screenshots/{chakra,radixt,radixp,ark}-tooltip-*.png`）
//! と比較し、共通ビジュアル言語（`crate::recipe` の
//! `focus_ring_declarations`/`disabled_declarations`/
//! `hover_surface_declarations`/`transition_declarations`）とスケール
//! トークン（`docs/design/pre-styled-ui-scale-tokens.md`）へ載せ替えた。
//!
//! - **サイズ / バリアント**: 合わせない（下記スコープ外節参照）。
//! - **色**: 生色リテラルなし。新設した `content` の影も
//!   `var(--fandhe-shadow-sm)` トークン参照のみでフォールバックに生色を
//!   持ち込まない。
//! - **`data-*` 状態**: headless 層が `trigger` へ出力する `data-disabled` を
//!   初めて視覚へ反映する（`disabled_declarations()`）。`data-state="open"`
//!   への専用視覚は追加しない（下記「意図的に参考サイトへ合わせない点」2 参照）。
//! - **ダーク**: 反転色（`--fandhe-color-fg`/`-bg`）で自動追従する既存挙動を
//!   維持。新設の影もダーク値内蔵トークン経由。
//! - **フォーカス**: `outline` 直書き 2 宣言を
//!   `focus_ring_declarations(FocusRingColor::Token,
//!   FocusRingOffset::Outside)` へ canonical 化した（palette 軸なし）。
//! - **余白・角丸・影**: `positioner` の `z-index` を
//!   `var(--fandhe-z-index-tooltip, 1100)`（§3.4 で tooltip 専用に割り当て
//!   られた段。旧来値 1100 を fallback に据える）へ、`content` の
//!   `border-radius` を `var(--fandhe-radius-sm, 0.25rem)`
//!   （§3.1「密なインライン部品 = sm」で tooltip 名指し）へトークン化し、
//!   `content` に `box-shadow: var(--fandhe-shadow-sm)`（§3.2「tooltip = sm」）
//!   を新設した。`content` のそれ以外の宣言（反転色・`font-size-sm`・
//!   padding・`max-width: 20rem`）は [`crate::toggle_tip`] と同一値を維持する
//!   （toggle-tip 側 rustdoc の「#1548 と乖離を作らない」契約に合わせる）。
//! - **hover / disabled / トランジション**: `trigger` を参照 4 サイトと同様の
//!   枠線付きボタン（[`crate::popover`] の `trigger` と同型。headless
//!   `tooltip::trigger` は常に `button` 要素なので button 見た目が妥当）
//!   へ整え、`hover_bg_muted()` + `StateCondition::Hover` →
//!   `hover_surface_declarations()` と
//!   `transition_declarations("background, border-color",
//!   MotionDuration::Fast)` を新設した。`data-disabled` には
//!   `disabled_declarations()` を登録する。
//!
//! ## 意図的に参考サイトへ合わせない点
//!
//! 1. size / variant 軸の追加（上記「サイズ / バリアント」参照。「複合部品の
//!    variant 統一方針 方針 3」= オーバーレイの配置・寸法がコンテンツ起因の
//!    popover/tooltip には提供しない。[`crate::toggle_tip`]（#1819）も両部品
//!    共通の一括検討が必要として見送り済み）。
//! 2. `trigger[data-state="open"]` の専用視覚。tooltip の open は hover /
//!    キーボードフォーカスと同時に成立するため、hover surface と
//!    focus-visible リングが既に開状態の視覚を担い、二重強調になる
//!    （クリック開閉の [`crate::toggle_tip`]/[`crate::popover`] とは前提が
//!    異なる）。`content[data-state="closed"] { visibility: hidden }` の
//!    既存連動は維持する。
//! 3. `content` の開閉フェード演出（headless 層が closed 時に `hidden` を
//!    即座に付与するライフサイクルのため描画されない既知の未解決事項、
//!    [`crate::popover`]/[`crate::toggle_tip`] と同じ判断。
//!    `prefers-reduced-motion` は `Theme::to_css` の duration 一括 0ms 化で
//!    自動的に尊重される）。
//! 4. `content` の配色を chakra panel 色（非反転）へ寄せない（参照 4 サイト
//!    とも反転色が標準であり現状維持が正）。
//! 5. `--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`（座標ジオメトリ）は
//!    [`crate::menu`]/[`crate::popover`] と同じ理由で本イシューの対象外。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - variant（size 等）ごとのクラス切り替えは headless ラッパー第 1 弾
//!   （#551）と同じくスコープ外とする。
//! - `openDelay`/`closeDelay`/`interactive`/`closeOnEscape` は headless 層の
//!   ドキュメント（`crates/headless-ui/src/tooltip.rs`）で既にスコープ外と
//!   明記済みのクライアントサイド実行時挙動であり、本モジュールもそれを
//!   継承する。
//! - `--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`（座標ジオメトリ）は
//!   [`crate::menu`]/[`crate::popover`] と同じ理由で本イシューの対象外。
//! - `arrow`/`arrow-tip` への装飾追加（座標ジオメトリ依存のため対象外を
//!   継続）。
//! - `content`/`positioner` の開閉フェード演出（上記「意図的に参考サイトへ
//!   合わせない点」3 参照）。
//! - showcase Demo への hover / disabled 状態の追加掲示（静的掲示のため
//!   現行方針どおり据え置き）。

use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe,
    StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。variant 軸は上記
// 「複合部品の variant 統一方針」方針 3 でオーバーレイ配置系の popover/
// tooltip には提供しないと確定済み（規約 B-2）、CSS 到達は
// [data-scope]/[data-part] 属性セレクタのみに依存する（規約 B-3、イシュー
// #1062 規約参照）。
pub use fandhe_frontend_headless_ui::tooltip::*;
// `root`/`trigger` 等の `state` 引数・`Tooltip::new`・`Tooltip` の
// `Component::Action`（dispatch 対象）はいずれも `state` モジュール由来で
// 上記 glob 再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{DisclosureAction, OpenState};

/// headless `tooltip` anatomy の `data-part` 一覧（`crates/headless-ui/src/tooltip.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "positioner",
    "content",
    "arrow",
    "arrow-tip",
];

/// この styled Tooltip の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("tooltip", SLOTS)
        .base("root", vec![decl("position", "relative")])
        // イシュー #1548: 参考 4 サイトはいずれも trigger を枠線付きボタンで
        // 表現する（[`crate::popover`] の `trigger` と同型。headless
        // `tooltip::trigger` は常に `button` 要素なので button 見た目が妥当）。
        .base(
            "trigger",
            [
                vec![
                    decl("cursor", "pointer"),
                    decl("background", "var(--fandhe-color-bg)"),
                    decl("color", "var(--fandhe-color-fg)"),
                    decl("border", "1px solid var(--fandhe-color-border)"),
                    decl("border-radius", "var(--fandhe-radius-md, 0.375rem)"),
                    decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                    hover_bg_muted(),
                ],
                transition_declarations("background, border-color", MotionDuration::Fast),
            ]
            .concat(),
        )
        .base(
            "positioner",
            vec![
                decl("position", "absolute"),
                decl("bottom", "100%"),
                decl("left", "0"),
                // イシュー #1548: §3.4 の割り当てで tooltip は tooltip 段
                // （1700）。旧来値 1100 を fallback に据える。
                decl("z-index", "var(--fandhe-z-index-tooltip, 1100)"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("background", "var(--fandhe-color-fg)"),
                decl("color", "var(--fandhe-color-bg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                // イシュー #1548: §3.1「密なインライン部品 = sm」で tooltip
                // 名指し。[`crate::toggle_tip`] の content と同一値を維持する。
                decl("border-radius", "var(--fandhe-radius-sm, 0.25rem)"),
                // イシュー #1548: §3.2「tooltip = sm」。新設宣言のため
                // fallback を持たない（トークン未定義時は宣言が無効化され
                // 従来どおり影なしになる）。
                decl("box-shadow", "var(--fandhe-shadow-sm)"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
                decl("max-width", "20rem"),
            ],
        )
        // イシュー #664 受け入れ条件: `content` の開閉状態に応じた見た目の切り替え。
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("visibility", "hidden")],
        )
        // disabled 時の視覚フィードバック（イシュー #1548）。
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
        // hover 時の視覚フィードバック（イシュー #1548）。
        .state(
            "trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
}

/// この styled Tooltip が生成する静的 CSS 全量を返す（決定的。
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
        assert!(a.contains(r#"[data-scope="tooltip"][data-part="content"]"#));
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
        assert!(css.contains(r#"[data-scope="tooltip"][data-part="positioner"]"#));
        assert!(css.contains("position: absolute;"));
        assert!(css.contains("bottom: 100%;"));
    }

    #[test]
    fn root_provides_containing_block_for_positioner() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"tooltip\"][data-part=\"root\"] {\n  position: relative;\n}\n"
        ));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tooltip"][data-part="content"][data-state="closed"]"#));
    }

    #[test]
    fn trigger_declares_focus_visible_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tooltip"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
    }

    #[test]
    fn content_does_not_consume_reference_width_css_var() {
        // イシュー #664 受け入れ条件 2: tooltip の content はテキスト内容へ
        // 幅が追随すべきであり、sameWidth 相当は不適切なため意図的に
        // --fandhe-reference-width を消費しないことを固定する（モジュール
        // doc §content は --fandhe-reference-width を消費しない 参照）。
        let css = stylesheet();
        assert!(!css.contains("--fandhe-reference-width"));
    }

    #[test]
    fn trigger_hover_surface_is_gated_by_hover_media_and_not_disabled() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover) {"));
        assert!(css.contains(
            r#"[data-scope="tooltip"][data-part="trigger"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn trigger_declares_disabled_visual() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tooltip"][data-part="trigger"][data-disabled] {"#));
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn trigger_declares_fast_transition() {
        let css = stylesheet();
        assert!(css.contains("transition-property: background, border-color;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
    }

    #[test]
    fn positioner_z_index_uses_tooltip_layer_token_with_legacy_fallback() {
        let css = stylesheet();
        assert!(css.contains("z-index: var(--fandhe-z-index-tooltip, 1100);"));
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
    fn trigger_has_no_open_state_rule() {
        // イシュー #1548「意図的に参考サイトへ合わせない点」2:
        // hover surface / focus-visible リングが既に開状態の視覚を担うため、
        // trigger[data-state="open"] 専用の視覚は追加しない。
        let css = stylesheet();
        assert!(!css.contains(r#"[data-scope="tooltip"][data-part="trigger"][data-state="open"]"#));
    }

    #[test]
    fn content_visual_block_matches_toggle_tip() {
        // toggle-tip 側 rustdoc の「#1548 と乖離を作らない」契約を機械化する:
        // tooltip::content と toggle_tip::content の base ブロックは
        // data-scope 名の置換を除いて一致する。
        let tooltip_css = stylesheet();
        let toggle_tip_css = crate::toggle_tip::stylesheet();

        let extract_content_block = |css: &str, scope: &str| -> String {
            let marker = format!("[data-scope=\"{scope}\"][data-part=\"content\"] {{");
            let start = css
                .find(&marker)
                .unwrap_or_else(|| panic!("content block not found for scope {scope}"));
            let end = css[start..]
                .find("}\n")
                .map(|i| start + i + 2)
                .unwrap_or_else(|| panic!("content block not terminated for scope {scope}"));
            css[start..end].replacen(scope, "SCOPE", 1)
        };

        let tooltip_block = extract_content_block(&tooltip_css, "tooltip");
        let toggle_tip_block = extract_content_block(&toggle_tip_css, "toggle-tip");
        assert_eq!(tooltip_block, toggle_tip_block);
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="tooltip""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_tooltip_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut t = Tooltip::default();
        assert_eq!(t.state(), OpenState::Closed);

        let ssr_html = render(&t.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut t, "open", ""));
        let hydrate_html = render(&render_for_hydration(&t));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        let restored = Tooltip::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}

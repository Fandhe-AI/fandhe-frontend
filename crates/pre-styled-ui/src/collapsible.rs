//! styled Collapsible（headless ラッパー、イシュー #1682、親 #1670/#520/#546）。
//!
//! `fandhe_frontend_headless_ui::collapsible`（イシュー #529、参考サイト突合は
//! #1637）の Root / Trigger / Indicator / Content 4 anatomy パーツと
//! [`fandhe_frontend_headless_ui::collapsible::Collapsible`] 状態機械をそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠・スコープ外事項は [`crate::toggle_tip`]/[`crate::accordion`] の
//! rustdoc と同じ方針に従う。
//!
//! # 参考サイト比較（7 軸チェック、イシュー #1682）
//!
//! 参照 3 サイト（chakra-ui / Radix Primitives / ark-ui）はいずれも
//! Collapsible に `size`/`variant`/`colorPalette` prop を持たない
//! （chakra はプレーンなテキストトリガー、ark はカード状トリガー +
//! シェブロン、Radix は `IconButton` トグル）。
//!
//! - **サイズ / バリアント**: 提供しない（下記スコープ外節参照）。
//! - **色**: 生の色リテラルなし。すべて `--fandhe-*` トークン参照。
//! - **`data-*` 状態**: headless 層が `trigger`/`indicator`/`content` へ
//!   出力する `data-state`（open/closed）・`data-disabled` を視覚へ反映する
//!   （`trigger[data-state="open"]` の文字色強調、
//!   `indicator[data-state="open"]` の回転、`trigger[data-disabled]` の
//!   `disabled_declarations()`）。
//! - **ダーク**: `--fandhe-color-*` トークン経由で自動追従する。
//! - **フォーカス**: `trigger` のみ [`crate::recipe::focus_ring_declarations`]
//!   （`FocusRingOffset::Outside`。`root` は `overflow: hidden` を持たず
//!   [`crate::toggle_tip`] と同じ判断）。
//! - **余白・角丸・影**: `--fandhe-radius-md`（trigger）/`--fandhe-radius-lg`
//!   （content、chakra のボックス表現に合わせた密度差）・`--fandhe-space-*`
//!   のスケールトークンのみを使う。
//! - **hover / disabled / transition**: `trigger` に
//!   [`crate::recipe::hover_bg_muted`] +
//!   [`crate::recipe::StateCondition::Hover`] →
//!   [`crate::recipe::hover_surface_declarations`]、
//!   [`crate::recipe::transition_declarations`]（`background, color` /
//!   `transform`）、`data-disabled` に
//!   [`crate::recipe::disabled_declarations`] を登録する。
//!
//! # `content` は base で `display` を宣言しない
//!
//! headless 層（`crates/headless-ui/src/collapsible.rs`）は closed のとき
//! `content` へ `hidden` 存在属性を付与する。base 規則で `display` を宣言
//! すると UA 既定 `[hidden] { display: none }` を上書きして閉じなくなる
//! （PR #575 Bugbot 指摘・dialog で発生した不具合と同種）。
//! [`crate::toggle_tip`] の `positioner` と同じ構造的回避を採る。
//!
//! # `indicator` の `display: inline-block`
//!
//! headless 層は `indicator` を `span`（非置換インライン要素、`transform`
//! が効かない）として描画する。[`accordion`]（`item-indicator`）と同じ
//! 理由で `display: inline-block` を base へ設定し、open 時の
//! `rotate(180deg)` が実際に適用されるようにする。
//!
//! [`accordion`]: crate::accordion
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **高さアニメーション**（Radix `--radix-collapsible-content-height`・
//!   `collapsedHeight` 部分表示相当）: content 高さの実測（JS）が前提であり、
//!   レイアウト計測の関心を `headless-ui` へ持ち込まない方針
//!   （`docs/policy/intentional-non-adoption.md` §3.25）と、docs サイトの
//!   無 JS 制約に反するため非採用（headless 層 rustdoc の判断を継承）。
//! - **size / variant / colorPalette 軸の追加**: 参照 3 サイトいずれも
//!   持たないため提供しない。
//! - Themes ページ（`site/themes/collapsible.md`）・Demo・原稿・
//!   `site/nav.toml` 登録・`docs/design/component-coverage-map.md` 更新は
//!   兄弟イシュー #1683 の担当（本 PR では触らない）。

use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe,
    StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。参照 3 サイトいずれも
// size/variant/colorPalette prop を持たないため variant 軸を提供せず
// （規約 B-2）、CSS 到達は [data-scope]/[data-part] 属性セレクタのみに依存する
// （規約 B-3、イシュー #1062 規約参照）。
pub use fandhe_frontend_headless_ui::collapsible::*;
// `root`/`trigger`/`indicator`/`content` の `state` 引数・`Collapsible` の
// `Component::Action`（dispatch 対象）はいずれも `state` モジュール由来で
// 上記 glob 再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（イシュー #685 の方針、[`crate::toggle_tip`] と同型）。
pub use fandhe_frontend_headless_ui::state::{DisclosureAction, OpenState};

/// headless `collapsible` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/collapsible.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を出力しない
/// fail-closed 側の不具合として現れるため、変更時は両ファイルを合わせて
/// 確認する）。
const SLOTS: &[&str] = &["root", "trigger", "indicator", "content"];

/// この styled Collapsible の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("collapsible", SLOTS)
        .base("root", vec![decl("display", "block")])
        .base(
            "trigger",
            [
                vec![
                    decl("display", "inline-flex"),
                    decl("align-items", "center"),
                    decl("justify-content", "space-between"),
                    decl("gap", "var(--fandhe-space-2)"),
                    decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                    decl("background", "transparent"),
                    decl("color", "var(--fandhe-color-fg)"),
                    decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                    decl("border", "0"),
                    decl("border-radius", "var(--fandhe-radius-md)"),
                    decl("cursor", "pointer"),
                    decl("text-align", "left"),
                    hover_bg_muted(),
                ],
                transition_declarations("background, color", MotionDuration::Fast),
            ]
            .concat(),
        )
        .base(
            "indicator",
            [
                vec![
                    decl("display", "inline-block"),
                    decl("color", "var(--fandhe-color-fg-muted)"),
                ],
                transition_declarations("transform", MotionDuration::Normal),
            ]
            .concat(),
        )
        // イシュー #1682: closed 時に headless 層が付与する `hidden` 存在属性
        // （UA 既定 `[hidden] { display: none }`）を base 規則で上書きしない
        // よう、`display` を意図的に宣言しない（モジュール doc 参照）。
        .base(
            "content",
            vec![
                decl("margin-top", "var(--fandhe-space-2)"),
                decl("padding", "var(--fandhe-space-4)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
            ],
        )
        // 開いている trigger/indicator を強調する（[`crate::accordion`] と同型）。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("color", "var(--fandhe-color-accent)")],
        )
        .state(
            "indicator",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("transform", "rotate(180deg)")],
        )
        // headless 層が出力する data-disabled を CSS 側で消費する。
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "content",
            StateCondition::Attr("data-disabled"),
            vec![decl("color", "var(--fandhe-color-fg-muted)")],
        )
        // キーボード操作時のみのフォーカスリング。root は overflow: hidden
        // を持たないため Outside（[`crate::toggle_tip`] と同じ判断）。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // hover 時の視覚フィードバック。
        .state(
            "trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
}

/// この styled Collapsible が生成する静的 CSS 全量を返す（決定的。
/// [`crate::toggle_tip::stylesheet`] と同じ契約: 同一プロセス内の複数回
/// 呼び出しは常にバイト単位で同一の文字列を返す）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="collapsible"][data-part="trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn indicator_has_transformable_display() {
        // headless indicator は span（非置換インライン要素）のため、
        // transform を効かせるには inline-block 等の非デフォルト display
        // が必要（accordion item-indicator と同じ理由、PR #575 系）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="collapsible"][data-part="indicator"] {"#));
        assert!(css.contains("display: inline-block;"));
    }

    #[test]
    fn content_base_does_not_declare_display() {
        // closed 時の `hidden` 存在属性（UA 既定 [hidden]{display:none}）を
        // base 規則が上書きしないことを固定する（モジュール doc 参照）。
        let css = stylesheet();
        let start = css
            .find(r#"[data-scope="collapsible"][data-part="content"] {"#)
            .expect("content base ブロックが見つからない");
        let block_end = css[start..]
            .find('}')
            .map(|i| start + i)
            .expect("content base ブロックの終端が見つからない");
        let block = &css[start..block_end];
        assert!(
            !block.contains("display:"),
            "content の base ブロックが display を宣言している: {block}"
        );
    }

    #[test]
    fn stylesheet_links_trigger_and_indicator_data_state_to_open_style() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="collapsible"][data-part="trigger"][data-state="open"] {"#)
        );
        assert!(css
            .contains(r#"[data-scope="collapsible"][data-part="indicator"][data-state="open"] {"#));
        assert!(css.contains("transform: rotate(180deg);"));
    }

    #[test]
    fn trigger_declares_focus_visible_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="collapsible"][data-part="trigger"]:focus-visible {"#));
    }

    #[test]
    fn trigger_and_content_declare_disabled_visual() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="collapsible"][data-part="trigger"][data-disabled] {"#));
        assert!(css.contains("opacity: 0.5;"));
        assert!(css.contains(r#"[data-scope="collapsible"][data-part="content"][data-disabled] {"#));
    }

    #[test]
    fn trigger_hover_surface_is_gated_by_hover_media_and_not_disabled() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover) {"));
        assert!(css.contains(
            r#"[data-scope="collapsible"][data-part="trigger"]:hover:not([data-disabled]) {"#
        ));
    }

    #[test]
    fn stylesheet_has_no_raw_color_literals() {
        let css = stylesheet();
        assert!(!css.contains('#'));
        assert!(!css.contains("rgb("));
        assert!(!css.contains("rgba("));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(OpenState::Closed, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="collapsible""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_collapsible_state_machine() {
        let mut c = Collapsible::default();
        assert_eq!(c.state(), OpenState::Closed);

        let ssr_html = render(&c.root(false, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut c, "toggle", ""));
        let hydrate_html = render(&render_for_hydration(&c));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        let restored = Collapsible::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}

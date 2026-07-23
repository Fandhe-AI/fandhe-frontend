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
//! [`crate::dialog`] の `positioner`（1001）より前面に来るべき値
//! （tooltip は他のオーバーレイの上にも表示されうる補助的な説明であるため）
//! として `1100` を用いる。`positioner` は base 規則で `display` を
//! 宣言しないため、closed 時に headless 層が付与する `hidden` 存在属性は
//! UA 既定 `[hidden] { display: none }` がそのまま機能する（[`crate::popover`]
//! と同じ構造的な回避、dialog で発生した PR #575 Bugbot 指摘（High）と
//! 同種の不具合を避ける）。
//!
//! # `content` は `--fandhe-reference-width` を消費しない（イシュー #664 受け入れ条件 2）
//!
//! [`crate::menu`]/[`crate::select`]/[`crate::popover`] の `content` は
//! トリガー実測幅へ追随する `sameWidth` 相当のスタイルを持つが、tooltip の
//! `content` は短いテキスト内容へ幅が追随すべきであり、`sameWidth` 相当は
//! 用途として不適切なため意図的に `--fandhe-reference-width` を消費しない
//! （既存の CSS 変数規約自体には反しない選択であることをここに明記する）。
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

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

pub use fandhe_frontend_headless_ui::tooltip::*;

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
        .base("trigger", vec![decl("cursor", "pointer")])
        .base(
            "positioner",
            vec![
                decl("position", "absolute"),
                decl("bottom", "100%"),
                decl("left", "0"),
                decl("z-index", "1100"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("background", "var(--fandhe-color-fg)"),
                decl("color", "var(--fandhe-color-bg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("border-radius", "0.25rem"),
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
        // キーボード操作時のみのフォーカスリング。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
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
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
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

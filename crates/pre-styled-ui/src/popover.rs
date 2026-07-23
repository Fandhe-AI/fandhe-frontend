//! styled Popover（headless ラッパー第 2 弾、イシュー #664、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::popover`（イシュー #532）の Root / Trigger /
//! Anchor / Positioner / Arrow / ArrowTip / Content / Title / Description /
//! CloseTrigger / Indicator 11 anatomy パーツと
//! [`fandhe_frontend_headless_ui::popover::Popover`] 状態機械をそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠・スコープ外事項は [`crate::dialog`]/[`crate::menu`] の rustdoc と
//! 同じ方針に従う。
//!
//! # data-state とスタイルの連動（イシュー #664 受け入れ条件）
//!
//! `trigger`/`content` の開閉 `data-state`（open/closed）に応じた見た目の
//! 切り替えを [`recipe`] へ登録する（[`crate::recipe::SlotRecipe::state`]）。
//!
//! # キーボード操作系属性の反映
//!
//! `trigger`/`close-trigger` はフォーカス可能なボタン要素であり、
//! キーボード操作時のみのフォーカスリング（`:focus-visible`）を
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::dialog`] と同じ判断）。
//!
//! # positioner のオーバーレイ配置
//!
//! headless 側 `root`（`crates/headless-ui/src/popover.rs`）の子として
//! `trigger`/`positioner` が並置される兄弟関係のため、containing block を
//! 提供する `position: relative` は共通祖先の `root` に付与し、
//! `positioner` 自体は `position: absolute; top: 100%; left: 0` の
//! dropdown 型オーバーレイとする（[`crate::menu`] と同じ tier、
//! `z-index: 10`。[`crate::dialog`] のビューポート全体オーバーレイ
//! （z-index: 1000/1001）とは役割が異なる）。`positioner` は base 規則で
//! `display` を宣言しないため、closed 時に headless 層が付与する `hidden`
//! 存在属性は UA 既定 `[hidden] { display: none }` がそのまま機能する
//! （[`crate::dialog`] の `positioner` のように `display: flex` 等の base
//! 宣言で UA 既定を上書きしていないため、`[hidden]` の明示的な上書き規則は
//! 不要。dialog で発生した PR #575 Bugbot 指摘（High）と同種の不具合を
//! 構造的に回避する）。
//!
//! # `--fandhe-reference-width` の消費（イシュー #664 受け入れ条件 2）
//!
//! `crates/wasm-full/src/position.rs::reposition_one` が `positioner` の
//! `style` 属性へ書き込む `--fandhe-reference-width`（`trigger` の実測幅）を
//! `content` の `min-width` が `var(--fandhe-reference-width, auto)` として
//! 消費する（[`crate::select`] と同じフォールバック判断。popover の
//! `content` は menu/select の listbox と異なり任意の自由形式コンテンツを
//! 保持するため、`auto` フォールバックが `10rem` 固定より適切）。
//! `--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`（座標ジオメトリ）は
//! [`crate::menu`] と同じ理由で本イシューの対象外とする。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - variant（size 等）ごとのクラス切り替えは headless ラッパー第 1 弾
//!   （#551）と同じくスコープ外とする。
//! - フォーカストラップ・Escape キー閉鎖・外側クリック閉鎖・
//!   `autoFocus`/portal/modal モード・アニメーションは headless 層の
//!   ドキュメント（`crates/headless-ui/src/popover.rs`）で既にスコープ外と
//!   明記済みであり、本モジュールもそれを継承する。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

pub use fandhe_frontend_headless_ui::popover::*;

/// headless `popover` anatomy の `data-part` 一覧（`crates/headless-ui/src/popover.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "anchor",
    "positioner",
    "arrow",
    "arrow-tip",
    "content",
    "title",
    "description",
    "close-trigger",
    "indicator",
];

/// この styled Popover の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("popover", SLOTS)
        .base("root", vec![decl("position", "relative")])
        .base(
            "trigger",
            vec![
                decl("cursor", "pointer"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
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
                decl("padding", "var(--fandhe-space-4)"),
                decl("min-width", "var(--fandhe-reference-width, auto)"),
            ],
        )
        .base(
            "title",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("margin", "0 0 var(--fandhe-space-2) 0"),
            ],
        )
        .base(
            "description",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("margin", "0"),
            ],
        )
        .base(
            "close-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        // イシュー #664 受け入れ条件: `trigger`/`content` の開閉状態に応じた見た目の切り替え。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("border-color", "var(--fandhe-color-accent)")],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("visibility", "hidden")],
        )
        // キーボード操作時のみのフォーカスリング（[`crate::dialog`] と同じ判断）。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .state(
            "close-trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
}

/// この styled Popover が生成する静的 CSS 全量を返す（決定的。
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
        assert!(a.contains(r#"[data-scope="popover"][data-part="content"]"#));
        assert!(a.contains(r#"[data-scope="popover"][data-part="trigger"]"#));
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
        assert!(css.contains(r#"[data-scope="popover"][data-part="positioner"]"#));
        assert!(css.contains("position: absolute;"));
    }

    #[test]
    fn root_provides_containing_block_for_positioner() {
        // trigger/positioner は headless root の下の兄弟要素であり、trigger は
        // positioner の祖先になれない。position: relative は共通祖先の root に
        // 付与する（menu と同じ判断）。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"popover\"][data-part=\"root\"] {\n  position: relative;\n}\n"
        ));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="popover"][data-part="trigger"][data-state="open"]"#));
        assert!(css.contains(r#"[data-scope="popover"][data-part="content"][data-state="closed"]"#));
    }

    #[test]
    fn trigger_and_close_trigger_declare_focus_visible_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="popover"][data-part="trigger"]:focus-visible {"#));
        assert!(
            css.contains(r#"[data-scope="popover"][data-part="close-trigger"]:focus-visible {"#)
        );
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }

    #[test]
    fn content_consumes_reference_width_css_var() {
        // イシュー #664 受け入れ条件 2: --fandhe-reference-width を CSS
        // 継承で消費するスタイルが反映されることを固定する（SSR 静的表示では
        // auto へフォールバック。select と同じ判断）。
        let css = stylesheet();
        assert!(css.contains("min-width: var(--fandhe-reference-width, auto);"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="popover""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_popover_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut p = Popover::default();
        assert_eq!(p.state(), OpenState::Closed);

        let ssr_html = render(&p.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut p, "open", ""));
        let hydrate_html = render(&render_for_hydration(&p));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        let restored = Popover::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}

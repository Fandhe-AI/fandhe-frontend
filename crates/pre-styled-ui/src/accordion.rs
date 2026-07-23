//! styled Accordion（headless ラッパー第 1 弾、イシュー #551、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::accordion`（イシュー #527）の Root / Item /
//! ItemTrigger / ItemIndicator / ItemContent 5 anatomy パーツと
//! [`fandhe_frontend_headless_ui::accordion::Accordion`] 状態機械（single
//! モード）をそのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。
//! 薄い委譲の根拠・スコープ外事項は [`crate::dialog`] の rustdoc と同じ方針に従う
//! （`data-scope`/`data-part` セレクタへの CSS 適用のみで、パーツ関数へ手を
//! 加えない）。
//!
//! # data-state とスタイルの連動（イシュー #551 受け入れ条件）
//!
//! 項目の開閉 `data-state`（open/closed）に応じて `item-trigger`/
//! `item-indicator` の見た目を切り替える CSS を [`recipe`] へ登録する
//! ([`crate::recipe::SlotRecipe::state`]、イシュー #643。`serialize_rule` を
//! 直接呼ぶ手書きセレクタ機構は廃止した)。`item-indicator` は headless 層
//! （`crates/headless-ui/src/accordion.rs`）でデフォルト `span`（非置換インライン
//! 要素）としてレンダリングされ `transform` が効かないため、[`recipe`] の
//! base 規則で `display: inline-block` を設定し `rotate(180deg)` が実際に
//! 適用されるようにする（PR #575 Bugbot 指摘対応）。
//!
//! # キーボード操作系スタイル（イシュー #643）
//!
//! `item-trigger` は roving tabindex でフォーカス移動するボタン要素であり、
//! キーボード操作時のみのフォーカスリング（`:focus-visible`）を [`recipe`]
//! へ登録する。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

pub use fandhe_frontend_headless_ui::accordion::*;
// `item`/`item_trigger`/`item_indicator`/`item_content` の `state` 引数・
// `Accordion::item_state` 戻り値・`Accordion`/`MultiAccordion` の
// `Component::Action`（dispatch 対象）はいずれも `state` モジュール由来で
// 上記 glob 再エクスポートでは到達しない。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証するための
// 明示再エクスポート（イシュー #685）。
pub use fandhe_frontend_headless_ui::state::{MultiSelectAction, OpenState, SingleSelectAction};

/// headless `accordion` anatomy の `data-part` 一覧（`crates/headless-ui/src/accordion.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "item",
    "item-trigger",
    "item-indicator",
    "item-content",
];

/// この styled Accordion の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("accordion", SLOTS)
        .base(
            "root",
            vec![
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.5rem"),
                decl("overflow", "hidden"),
            ],
        )
        .base(
            "item",
            vec![decl(
                "border-bottom",
                "1px solid var(--fandhe-color-border-muted)",
            )],
        )
        .base(
            "item-trigger",
            vec![
                decl("display", "flex"),
                decl("width", "100%"),
                decl("padding", "var(--fandhe-space-4)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("cursor", "pointer"),
                decl("border", "0"),
                decl("text-align", "left"),
            ],
        )
        .base(
            "item-indicator",
            vec![
                decl("display", "inline-block"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "item-content",
            vec![
                decl("padding", "var(--fandhe-space-4)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        // イシュー #551 受け入れ条件: 開いている項目の trigger/indicator を強調する。
        .state(
            "item-trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("color", "var(--fandhe-color-accent)")],
        )
        .state(
            "item-indicator",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("transform", "rotate(180deg)")],
        )
        // イシュー #643: キーボード操作時のみのフォーカスリング。
        .state(
            "item-trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
}

/// この styled Accordion が生成する静的 CSS 全量を返す（決定的。[`crate::dialog::stylesheet`]
/// と同じ契約: 同一プロセス内の複数回呼び出しは常にバイト単位で同一の文字列を返す）。
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
        assert!(a.contains(r#"[data-scope="accordion"][data-part="item-trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn item_indicator_has_transformable_display() {
        // PR #575 Bugbot 指摘対応: item-indicator が transform 可能な display
        // （非デフォルトのインラインボックス）を持つことを固定する。base の
        // `display: inline-block` がないと `transform: rotate(180deg)`
        // （state_css）が実際には効かない。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="accordion"][data-part="item-indicator"] {"#));
        assert!(css.contains("display: inline-block;"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(vec![], vec![]));
        assert!(html.contains(r#"data-scope="accordion""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open() {
        // イシュー #551 受け入れ条件: 「headless 層の data-state とスタイルの
        // 連動テスト（[data-state='open'] セレクタ等）」を固定する。
        let css = stylesheet();
        assert!(css
            .contains(r#"[data-scope="accordion"][data-part="item-trigger"][data-state="open"]"#));
        assert!(css.contains(
            r#"[data-scope="accordion"][data-part="item-indicator"][data-state="open"]"#
        ));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_accordion_state_machine() {
        // イシュー #551 受け入れ条件: 「SSR / hydration 両経路の動作確認」を
        // 再エクスポートされた `Accordion`（headless の Component/Hydrate 実装を
        // そのまま継承）経由で固定する。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut a = Accordion::default();
        assert_eq!(a.expanded(), None);

        let ssr_html = render(&a.item("panel-1", false, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        assert!(dispatch(&mut a, "select", "panel-1"));
        assert_eq!(a.expanded(), Some("panel-1"));

        let hydrate_html = render(&render_for_hydration(&a));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = Accordion::from_hydration_attrs(&a.hydration_attrs()).unwrap();
        assert_eq!(restored.expanded(), Some("panel-1"));
    }

    #[test]
    fn item_trigger_declares_focus_visible_ring() {
        // イシュー #643 受け入れ条件: キーボード操作系属性（:focus-visible）
        // が recipe 経由で反映されることを固定する。
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="accordion"][data-part="item-trigger"]:focus-visible {"#)
        );
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }
}

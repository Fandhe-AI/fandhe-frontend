//! styled Navigation Menu（headless ラッパー、イシュー #993、親 #932 Phase 8）。
//!
//! `fandhe_frontend_headless_ui::navigation_menu`（イシュー #993）の Root /
//! List / Item / Trigger / Content / Link 6 anatomy パーツと
//! [`fandhe_frontend_headless_ui::navigation_menu::NavigationMenu`]
//! （[`fandhe_frontend_headless_ui::state::SingleSelect`] を埋め込んだ
//! 「高々 1 個の Trigger だけが開く」状態機械）をそのまま再エクスポートし、
//! [`stylesheet`] で既定 CSS を追加提供する（[`crate::menubar`] と同型の
//! 薄い委譲）。
//!
//! # レイアウト（トリガー行の重なり・縦ずれ回帰の予防、PR #1000 の反省）
//!
//! `item` に `position: relative` を、`content` に `position: absolute;
//! top: 100%; left: 0;` を宣言する（一般的なナビゲーションドロップダウン。
//! [`crate::menu`] の `positioner`（`absolute; top: 100%`）と同型）。
//!
//! `list` の `align-items` は **`center` ではなく `flex-start` を既定にする**。
//! トリガーの高さが揃っている通常表示では `center` と視覚的に同一だが、
//! showcase で `content` をフロー内へ中和したときに 1 項目だけ縦に伸びて
//! 他項目が縦ずれする回帰
//!（`crates/docs-site/src/showcase.rs` の `SHOWCASE_LAYOUT_CSS` が
//! `[data-scope="navigation-menu"][data-part="content"] { position: static;
//! }` で `content` を中和した際に発生しうる、PR #1000 の Menubar showcase
//! 修正 3 コミット目と同型の障害）を、`flex-start` を既定にすることで
//! 構造的に発生させない（`center` のままだと、伸びた 1 項目の高さぶん
//! flexbox が全項目を中央合わせし直し、隣接する未展開項目が上下にずれる）。
//!
//! # focus-visible リング
//!
//! `trigger` はネイティブなフォーカス可能要素（`<button>`）であり、
//! キーボード操作時のみのフォーカスリングを
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::menubar`]/[`crate::toolbar`] と同じ判断）。`link` はネイティブ
//! `<a>` であり同様にフォーカス可能だが、本モジュールでは強調は
//! `data-current`（アクティブリンク）側で表現し、`:focus-visible` は
//! `trigger` のみに登録する（headless 層が `link` へ独自の highlight
//! 状態を持たないため）。
//!
//! # 本イシューのスコープ外
//!
//! headless 層（`crates/headless-ui/src/navigation_menu.rs`）のモジュール
//! doc「スコープ外」節をそのまま継承する（`data-motion`・viewport 寸法
//! 測定、Indicator/Viewport/Sub\* パーツ・`orientation` 引数、キーボード
//! 操作の実 DOM 配線）。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

pub use fandhe_frontend_headless_ui::navigation_menu::*;
// `OpenState` は本モジュールの再エクスポート対象パーツ関数（`item`/
// `trigger`/`content`）の引数型として呼び出し側が組み立てる必要があるが、
// `navigation_menu` モジュールの glob 再エクスポートでは到達しない
// （`state` モジュール由来のため）。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証する
// ための明示再エクスポート（[`crate::menubar`] の `Orientation`/`OpenState`
// と同型のパターン）。
pub use fandhe_frontend_headless_ui::state::OpenState;

/// headless `navigation_menu` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/navigation_menu.rs` の `ANATOMY.part(...)`
/// 呼び出しと同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を
/// 出力しない fail-closed 側の不具合として現れるため、変更時は両ファイルを
/// 合わせて確認する）。
const SLOTS: &[&str] = &["root", "list", "item", "trigger", "content", "link"];

/// この styled Navigation Menu の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("navigation-menu", SLOTS)
        .base(
            "root",
            vec![decl("display", "flex"), decl("align-items", "center")],
        )
        .base(
            "list",
            vec![
                decl("display", "flex"),
                // §モジュール冒頭 rustdoc「レイアウト」節参照: showcase の
                // content 中和時の縦ずれ回帰を構造的に防ぐため center ではなく
                // flex-start を既定にする。
                decl("align-items", "flex-start"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("list-style", "none"),
                decl("margin", "0"),
                decl("padding", "0"),
            ],
        )
        .base("item", vec![decl("position", "relative")])
        .base(
            "trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("border-radius", "0.25rem"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-3)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("position", "absolute"),
                decl("top", "100%"),
                decl("left", "0"),
                decl("z-index", "10"),
                decl("margin-top", "var(--fandhe-space-1)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl("box-shadow", "0 4px 6px rgba(0, 0, 0, 0.15)"),
                decl("padding", "var(--fandhe-space-2)"),
                decl("min-width", "10rem"),
            ],
        )
        .base(
            "link",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("text-decoration", "none"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("border-radius", "0.25rem"),
            ],
        )
        // 開いている trigger を視覚的に強調する。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("background", "var(--fandhe-color-accent-subtle)")],
        )
        // アクティブリンク（現在地）を視覚的に強調する。
        .state(
            "link",
            StateCondition::Attr("data-current"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        // headless 層の navigation_menu trigger はネイティブ `disabled` 属性を
        // 付与する設計（[`crate::accordion`] の item_trigger と同型）であり、
        // disabled 項目もフォーカス順序に残す [`crate::menubar`]/[`crate::toolbar`]
        // （aria-disabled のみでネイティブ disabled を付与しない設計）とは逆に
        // フォーカス順序から除外される。ここでは視覚的にのみ操作不能を示す。
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.5"), decl("cursor", "not-allowed")],
        )
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.5"), decl("cursor", "not-allowed")],
        )
        // trigger はキーボード操作時のみのフォーカスリング。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
}

/// この styled Navigation Menu が生成する静的 CSS 全量を返す（決定的。
/// [`crate::menubar::stylesheet`] と同じ契約）。
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
        for part in SLOTS {
            let needle = format!(r#"[data-scope="navigation-menu"][data-part="{part}"]"#);
            assert!(a.contains(&needle), "missing selector for part={part}");
        }
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn trigger_open_state_is_visually_distinct() {
        let css = stylesheet();
        assert!(css
            .contains(r#"[data-scope="navigation-menu"][data-part="trigger"][data-state="open"]"#));
    }

    #[test]
    fn current_link_is_visually_distinct() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="navigation-menu"][data-part="link"][data-current]"#));
    }

    #[test]
    fn list_align_items_is_flex_start_not_center() {
        // モジュール冒頭 rustdoc「レイアウト」節参照: showcase の content
        // 中和時の縦ずれ回帰の予防策を固定する回帰テスト。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"navigation-menu\"][data-part=\"list\"] {\n  display: flex;\n  align-items: flex-start;\n  "
        ));
        // root（トリガー行そのものの縦中央揃え、通常表示のみに関わる）は
        // center のままでよい。回帰対象は list（ドロップダウン展開時に
        // 縦ずれを起こしうるコンテナ）のみであるため、list パーツの
        // セレクタブロックに絞って center が使われていないことを確認する。
        assert!(!css.contains(
            "[data-scope=\"navigation-menu\"][data-part=\"list\"] {\n  display: flex;\n  align-items: center;\n"
        ));
    }

    #[test]
    fn trigger_declares_focus_visible_ring_but_link_does_not() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="navigation-menu"][data-part="trigger"]:focus-visible {"#)
        );
        assert!(
            !css.contains(r#"[data-scope="navigation-menu"][data-part="link"]:focus-visible {"#)
        );
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root("Main", vec![], vec![]));
        assert!(html.contains(r#"data-scope="navigation-menu""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("role="));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut m = NavigationMenu::default();
        assert_eq!(m.open_value(), None);

        let ssr_html = render(&m.trigger("products", false, None, None, vec![], vec![]));
        assert!(ssr_html.contains(r#"aria-expanded="false""#));

        assert!(dispatch(&mut m, "select", "products"));
        assert_eq!(m.open_value(), Some("products"));

        let hydrate_html = render(&render_for_hydration(&m));
        assert!(hydrate_html.contains("data-hydrate-selected="));

        let restored = NavigationMenu::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored, m);
    }
}

//! styled Menubar（headless ラッパー、イシュー #992、親 #932 Phase 8）。
//!
//! `fandhe_frontend_headless_ui::menubar`（イシュー #992）の Root / Menu /
//! Trigger / Positioner / Content / Item / ItemGroup / ItemGroupLabel /
//! Separator / SubTrigger / SubContent 11 anatomy パーツと
//! [`fandhe_frontend_headless_ui::menubar::Menubar`] roving tabindex + 単一
//! 開閉状態機械をそのまま再エクスポートし、[`stylesheet`] で既定 CSS を
//! 追加提供する（[`crate::toolbar`] と同型の薄い委譲）。
//!
//! # `size`/`color-palette` variant 軸は提供しない
//!
//! [`crate::menu`]（イシュー #729）とは異なり、本モジュールは `size`/
//! `color-palette` variant を提供しない（既定 1 種の見た目のみ）。Menubar
//! は Radix Primitives 上でもトップレベルのナビゲーション構造という位置
//! 付けであり、サイズバリエーションの需要が薄いという判断（受け入れ条件・
//! 計画で確定済み）。将来 variant 需要が生じた場合は [`crate::menu`] の
//! `Size` variant パターンをそのまま踏襲できる。
//!
//! # レイアウト
//!
//! `root` は `display: flex` + `align-items: center` + `gap` の横並びを
//! 既定とし、`data-orientation="vertical"` のとき `flex-direction: column`
//! へ切り替える（headless 層が `data-orientation` を固定出力する契約、
//! `crates/headless-ui/src/menubar.rs` 参照。[`crate::toolbar`] と同判断）。
//!
//! # `menu` パーツの `position: relative`
//!
//! [`crate::menu`] の styled `root` が `position: relative`（`positioner`
//! の containing block）を担うのに対し、Menubar では 1 Menubar に複数
//! Menu が並ぶため、per-menu ラッパーである `menu` パーツがこの責務を担う
//! （headless 層の `menu` anatomy パーツ、`crates/headless-ui/src/menubar.rs`
//! 「`role="none"` の根拠と制約」参照）。
//!
//! # `content` パーツの `position: relative`（サブメニューの containing block）
//!
//! `sub-trigger`/`sub-content` は `content` の子として並ぶ兄弟パーツであり
//! （headless 層は Portal による実 DOM 移送を行わない、本 rustdoc「本イシュー
//! のスコープ外」節参照）、`sub-content` は `position: absolute; top: 0;
//! left: 100%` で自身の containing block の右上角を基準に配置される。
//! `content` 自身に `position` を明示していないと、containing block 検索は
//! さらに外側の祖先（既定では `positioner`）まで遡る。この既定状態は
//! `positioner` の padding box が実質的に `content` の外接矩形とほぼ一致する
//! ため見た目上の破綻は起きにくいが、`crates/docs-site/src/showcase.rs` の
//! `SHOWCASE_LAYOUT_CSS`（PR #1000 Bugbot 指摘 1 対応）が掲示用に `menubar`
//! の `positioner` を `position: static` へ中和すると、containing block
//! 検索は `positioner` を素通りしてさらに外側の `menu`（`position: relative`,
//! 本モジュール「`menu` パーツの `position: relative`」節参照）まで遡って
//! しまい、`sub-content` が `content` の右上角ではなく Menubar 上の
//! per-menu ラッパー（File トリガー行を含む）の右上角を基準に配置される
//! 回帰を招く（PR #1000 Bugbot 指摘 2）。[`crate::menu`] の `root` が
//! `trigger`/`positioner` 共通祖先として `position: relative` を担うのと
//! 同型の判断として、`sub-trigger`/`sub-content` の共通祖先である `content`
//! 自身に `position: relative` を宣言し、外側の祖先（`positioner`/`menu`）の
//! 中和有無に依存しない安定した containing block を確定させる。トリガー行
//! そのものを基準にした厳密な配置計算（`placement` 相当）は本 rustdoc
//! 「本イシューのスコープ外」節が示すとおり対象外のまま。
//!
//! # focus-visible リング
//!
//! `trigger` はネイティブなフォーカス可能要素（`<button>`）であり、
//! キーボード操作時のみのフォーカスリングを
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::toolbar`]/[`crate::menu`] と同じ判断）。`item`/`sub-trigger`
//! は virtual focus パターン（実 DOM フォーカスは `trigger` に留まる）の
//! ため `:focus-visible` は付けず、`data-highlighted` で表現する
//! （[`crate::menu`] の `item` と同判断）。
//!
//! # 本イシューのスコープ外
//!
//! headless 層（`crates/headless-ui/src/menubar.rs`）のモジュール doc
//! 「スコープ外」節をそのまま継承する（矢印キー実 DOM 配線・
//! CheckboxItem/RadioGroup/RadioItem/ItemIndicator/Arrow/ArrowTip・Portal の
//! 実 DOM 移送・placement 計算・skip-disabled モード）。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

pub use fandhe_frontend_headless_ui::menubar::*;
// `Orientation`/`OpenState` は本モジュールの再エクスポート対象パーツ関数
// （`root`/`menu`/`positioner`/`content`/`sub_trigger`/`sub_content` 等）の
// 引数型として呼び出し側が組み立てる必要があるが、`menubar` モジュールの
// glob 再エクスポートでは到達しない（`data_attrs`/`state` モジュール由来の
// ため）。呼び出し側が `fandhe-frontend-pre-styled-ui` のみに依存して
// 呼び出せることを保証するための明示再エクスポート（[`crate::toolbar`] の
// `Orientation` と同型のパターン）。
pub use fandhe_frontend_headless_ui::data_attrs::Orientation;
pub use fandhe_frontend_headless_ui::state::OpenState;

/// headless `menubar` anatomy の `data-part` 一覧（`crates/headless-ui/src/menubar.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "menu",
    "trigger",
    "positioner",
    "content",
    "item",
    "item-group",
    "item-group-label",
    "separator",
    "sub-trigger",
    "sub-content",
];

/// この styled Menubar の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("menubar", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("border-bottom", "1px solid var(--fandhe-color-border)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("padding", "var(--fandhe-space-1)"),
            ],
        )
        .base("menu", vec![decl("position", "relative")])
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
                decl("position", "relative"),
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
            "item",
            vec![
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("cursor", "pointer"),
                decl("border-radius", "0.25rem"),
            ],
        )
        .base(
            "item-group",
            vec![decl("display", "flex"), decl("flex-direction", "column")],
        )
        .base(
            "item-group-label",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
            ],
        )
        .base(
            "separator",
            vec![
                decl("border", "0"),
                decl("border-top", "1px solid var(--fandhe-color-border)"),
                decl("margin", "var(--fandhe-space-2) 0"),
            ],
        )
        .base(
            "sub-trigger",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "space-between"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("cursor", "pointer"),
                decl("border-radius", "0.25rem"),
            ],
        )
        .base(
            "sub-content",
            vec![
                decl("position", "absolute"),
                decl("top", "0"),
                decl("left", "100%"),
                decl("z-index", "10"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl("box-shadow", "0 4px 6px rgba(0, 0, 0, 0.15)"),
                decl("padding", "var(--fandhe-space-2)"),
                decl("min-width", "10rem"),
            ],
        )
        // root が縦向きのとき列方向へ切り替える（本モジュール冒頭 rustdoc
        // 「レイアウト」節参照）。
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("flex-direction", "column")],
        )
        // 開いている trigger / sub-trigger を視覚的に強調する。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("background", "var(--fandhe-color-accent-subtle)")],
        )
        .state(
            "sub-trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("background", "var(--fandhe-color-accent-subtle)")],
        )
        // virtual focus の highlight 表示（trigger は実フォーカスを受ける
        // ためこの規則の対象外、本モジュール冒頭 rustdoc「focus-visible
        // リング」節参照）。
        .state(
            "item",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        .state(
            "sub-trigger",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        // disabled でもフォーカス順序には残るため（headless 層の意図的な
        // 設計判断、`crates/headless-ui/src/menubar.rs` モジュール doc
        // 「スコープ外」節参照）、視覚的にのみ操作不能を示す。
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
        .state(
            "sub-trigger",
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

/// この styled Menubar が生成する静的 CSS 全量を返す（決定的。
/// [`crate::toolbar::stylesheet`] と同じ契約）。
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
            let needle = format!(r#"[data-scope="menubar"][data-part="{part}"]"#);
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
    fn root_switches_to_column_when_vertical() {
        let css = stylesheet();
        assert!(css
            .contains(r#"[data-scope="menubar"][data-part="root"][data-orientation="vertical"]"#));
        assert!(css.contains("flex-direction: column;"));
    }

    #[test]
    fn trigger_open_state_is_visually_distinct() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menubar"][data-part="trigger"][data-state="open"]"#));
    }

    #[test]
    fn content_provides_containing_block_for_sub_content() {
        // PR #1000 Bugbot 指摘 2 対応: `sub-trigger`/`sub-content` は `content`
        // の子として並ぶ兄弟パーツであり、`sub-content` の `position: absolute`
        // な配置はいずれかの祖先が containing block を提供しないと不定になる
        // （既定では `positioner` が担うが、showcase の `SHOWCASE_LAYOUT_CSS`
        // が `positioner` を `position: static` へ中和すると検索が `menu` まで
        // 遡ってしまい per-menu ラッパーの角を基準に配置される回帰が起きる、
        // 本モジュール冒頭 rustdoc「`content` パーツの `position: relative`」
        // 節参照）。`content` 自身が `position: relative;` を宣言し、外側の
        // 祖先の中和有無に依存しない containing block になっていることを
        // 固定する。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"menubar\"][data-part=\"content\"] {\n  position: relative;\n  "
        ));
    }

    #[test]
    fn trigger_declares_focus_visible_ring_but_item_does_not() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="menubar"][data-part="trigger"]:focus-visible {"#));
        assert!(!css.contains(r#"[data-scope="menubar"][data-part="item"]:focus-visible {"#));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Orientation::Horizontal, "Menubar", vec![], vec![]));
        assert!(html.contains(r#"data-scope="menubar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="menubar""#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_menubar_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut m = Menubar::new(0, 3, None, false, Orientation::Horizontal);
        assert_eq!(m.focused(), 0);

        let ssr_html = render(&m.trigger(0, false, false, None, vec![], vec![]));
        assert!(ssr_html.contains(r#"tabindex="0""#));

        assert!(dispatch(&mut m, "open", "1"));
        assert_eq!(m.open(), Some(1));

        let hydrate_html = render(&render_for_hydration(&m));
        assert!(hydrate_html.contains(r#"data-hydrate-focused="1""#));
        assert!(hydrate_html.contains(r#"data-hydrate-open="1""#));

        let restored = Menubar::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored, m);
    }
}

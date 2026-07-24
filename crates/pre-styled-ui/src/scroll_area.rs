//! styled ScrollArea（headless ラッパー、イシュー #825、親 #520/#546）。
//!
//! `fandhe_frontend_headless_ui::scroll_area`（イシュー #825）の Root /
//! Viewport / Content / Scrollbar / Thumb / Corner 6 anatomy パーツ関数を
//! そのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。
//! 状態機械を持たない自由関数のみの headless モジュールであるため、薄い
//! 委譲の根拠は [`crate::breadcrumb`]/[`crate::nav_list`] と同じ方針に従う。
//!
//! # viewport の CSS `overflow` によるスクロール表現
//!
//! headless 層は anatomy（`data-scope`/`data-part`）と `tabindex="0"` のみを
//! 出力し、実際のスクロール可能領域は本モジュールが `viewport` へ
//! `overflow: auto` を付与することで実現する（ネイティブスクロール、JS
//! 不要）。`root` は `position: relative; overflow: hidden` とし、
//! `scrollbar`（後述）を将来 `viewport` の上へ絶対配置するための
//! containing block を提供する（[`crate::popover`]/[`crate::menu`] の
//! `root: position: relative` と同じ判断）。
//!
//! `viewport` には併せて `height: 100%`/`width: 100%` を付与し、`root` の
//! サイズへ強制的に連動させる。利用側が `root` へ固定高さ（例:
//! `crates/docs-site/src/showcase.rs` の `height: 8rem` 指定）を与えた場合
//! でも、`viewport` がこの連動を持たなければ content に合わせて自然に
//! サイズが伸びてしまい、`overflow: auto` が発火せずネイティブスクロール
//! バーが表示されない不具合があった（PR #856 Bugbot 指摘）。
//!
//! # scrollbar/thumb/corner は初期実装で非表示（イシュー #825 スコープ）
//!
//! headless 層のモジュール doc（`crates/headless-ui/src/scroll_area.rs`）が
//! 明記する通り、JS によるスクロール位置追従・thumb drag は本イシューの
//! スコープ外である。`scrollbar`/`thumb`/`corner` パーツ自体は将来 JS
//! 追従を実装する際の受け皿として静的マークアップを提供するが、追従処理
//! なしに表示するとスクロール位置と無関係な固定位置のつまみが誤解を招く
//! ため、初期実装では `display: none` にしてネイティブスクロールバーの
//! 標準プロパティ（後述）による装飾で代替する。JS 追従を実装する Issue が
//! 起票された際に本 `display: none` を解除する想定。
//!
//! # ネイティブスクロールバーの装飾（`scrollbar-width`/`scrollbar-color`・`::-webkit-scrollbar`）
//!
//! `scrollbar`/`thumb`/`corner` を非表示にする代わりに、`viewport` へ
//! 標準プロパティ `scrollbar-width: thin` + `scrollbar-color`（Firefox 等）
//! を付与し、[`stylesheet`] が `recipe().css()` に続けて `::-webkit-scrollbar`
//! 系規則（Chromium/WebKit 系）を固定文字列として追記することでカスタム
//! スクロールバー表現の見た目を実現する（[`crate::spinner::css`] が
//! `@keyframes` を固定文字列追記する precedent と同型。値はすべて固定
//! リテラル + テーマ CSS 変数参照のみで構成され、動的入力は一切混入しない）。
//!
//! # variant は非提供（イシュー #825 判断）
//!
//! chakra-ui の ScrollArea が持つ `variant="hover"/"always"`・`size` 相当の
//! variant 軸は本イシューの初期実装で採用しない。`::-webkit-scrollbar` 系
//! 規則は [`crate::recipe::SlotRecipe`] の宣言 API（`{`/`}`/`;` を含む値を
//! 拒否する）で表現できず固定文字列として追記する構成のため、variant ごとに
//! 出し分けようとすると手書き定数が variant 分岐で分裂し、決定性・保守性が
//! 悪化する。必要になった時点で再評価する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - JS によるスクロール位置追従（thumb の位置・サイズをスクロール量に応じて
//!   同期する処理）・thumb の drag 操作。
//! - ネイティブスクロールバーを完全に隠して独自スクロールバーへ置き換える
//!   JS（`scrollbar-width: none` 相当のクロスブラウザ制御）。
//! - `variant`（hover/always 等）・`size` variant 軸。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

pub use fandhe_frontend_headless_ui::scroll_area::*;

/// headless `scroll_area` anatomy の `data-part` 一覧（`crates/headless-ui/src/scroll_area.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "viewport",
    "content",
    "scrollbar",
    "thumb",
    "corner",
];

/// この styled ScrollArea の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("scroll-area", SLOTS)
        .base(
            "root",
            vec![decl("position", "relative"), decl("overflow", "hidden")],
        )
        .base(
            "viewport",
            vec![
                decl("height", "100%"),
                decl("width", "100%"),
                decl("overflow", "auto"),
                decl("scrollbar-width", "thin"),
                decl("scrollbar-color", "var(--fandhe-color-border) transparent"),
            ],
        )
        .base("content", vec![decl("display", "block")])
        .base("scrollbar", vec![decl("display", "none")])
        .base("thumb", vec![decl("display", "none")])
        .base("corner", vec![decl("display", "none")])
        // キーボード操作時のみのフォーカスリング（viewport は tabindex="0"
        // を固定付与するフォーカス可能領域、`crate::dialog`/`crate::tooltip`
        // と同じ判断）。
        .state(
            "viewport",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "-2px"),
            ],
        )
}

/// この styled ScrollArea が生成する静的 CSS 全量を返す（決定的。
/// [`crate::tooltip::stylesheet`] と同じ契約）。
///
/// recipe が生成する規則群に続けて、`::-webkit-scrollbar` 系の固定 CSS
/// リテラルを追記する（[`crate::spinner::css`] の `@keyframes` 追記と同型の
/// precedent）。値はソースコード中の固定リテラル + テーマ CSS 変数参照のみで
/// 構成され、外部入力は一切混入しない。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();
    out.push_str(
        "[data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar {\n  \
         width: 0.5rem;\n  height: 0.5rem;\n}\n\
         [data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar-track {\n  \
         background: transparent;\n}\n\
         [data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar-thumb {\n  \
         background: var(--fandhe-color-border);\n  \
         border-radius: var(--fandhe-radius-full);\n}\n\
         [data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar-corner {\n  \
         background: transparent;\n}\n",
    );
    out
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
        assert!(a.contains(r#"[data-scope="scroll-area"][data-part="viewport"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn viewport_scrolls_via_overflow_auto() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="scroll-area"][data-part="viewport"] {"#));
        assert!(css.contains("overflow: auto;"));
        assert!(css.contains("scrollbar-width: thin;"));
    }

    #[test]
    fn viewport_fills_root_so_overflow_auto_actually_triggers() {
        // `root` に固定高さ（例: ショーケースの `height: 8rem`）が設定された
        // 場合でも、`viewport` が `root` の高さへ連動していなければ
        // `viewport` は content に合わせて自然にサイズが伸び、`overflow: auto`
        // が発火せずネイティブスクロールバーが表示されない不具合があった
        // （PR #856 Bugbot 指摘）。`height: 100%`/`width: 100%` により
        // `viewport` が `root`（`position: relative` の containing block）の
        // サイズへ強制的に連動することを固定する回帰テスト。
        let css = stylesheet();
        assert!(css.contains("height: 100%;"));
        assert!(css.contains("width: 100%;"));
    }

    #[test]
    fn root_provides_containing_block_and_clips_overflow() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"scroll-area\"][data-part=\"root\"] {\n  position: relative;\n  overflow: hidden;\n}\n"
        ));
    }

    #[test]
    fn scrollbar_thumb_corner_are_hidden_in_initial_implementation() {
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"scroll-area\"][data-part=\"scrollbar\"] {\n  display: none;\n}\n"
        ));
        assert!(css.contains(
            "[data-scope=\"scroll-area\"][data-part=\"thumb\"] {\n  display: none;\n}\n"
        ));
        assert!(css.contains(
            "[data-scope=\"scroll-area\"][data-part=\"corner\"] {\n  display: none;\n}\n"
        ));
    }

    #[test]
    fn stylesheet_includes_webkit_scrollbar_rules() {
        let css = stylesheet();
        assert!(css.contains("::-webkit-scrollbar {"));
        assert!(css.contains("::-webkit-scrollbar-thumb {"));
        assert!(css.contains("::-webkit-scrollbar-track {"));
        assert!(css.contains("::-webkit-scrollbar-corner {"));
        assert!(css.contains("var(--fandhe-color-border)"));
        assert!(css.contains("var(--fandhe-radius-full)"));
    }

    #[test]
    fn viewport_declares_focus_visible_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="scroll-area"][data-part="viewport"]:focus-visible {"#));
        assert!(css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(vec![], vec![]));
        assert!(html.contains(r#"data-scope="scroll-area""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn reexported_viewport_renders_with_tabindex() {
        let html = render(&viewport(vec![], vec![]));
        assert!(html.contains(r#"data-part="viewport""#));
        assert!(html.contains(r#"tabindex="0""#));
    }
}

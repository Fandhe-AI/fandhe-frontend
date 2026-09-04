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
//! # 参考サイト基準へのスタイル調整（イシュー #1584）
//!
//! chakra-ui / Radix Themes / Radix Primitives / ark-ui の Scroll Area と
//! 比較し、以下を是正した（`docs/design/component-coverage-map.md` 参照）。
//!
//! - **thumb 色のトークン化**: 固定値 `var(--fandhe-color-border)` を
//!   直接参照するのではなく、custom property
//!   `--fandhe-scroll-area-thumb-bg`（既定
//!   `var(--fandhe-color-border-emphasized, var(--fandhe-color-border))`）
//!   を介して `scrollbar-color` と `::-webkit-scrollbar-thumb` の双方が
//!   同じ値を参照する構成へ変更した。custom property は擬似要素へも
//!   継承されるため、variant 軸を新設せず（下記「variant は非提供」節）
//!   利用側の上書き（例: `root` へ `--fandhe-scroll-area-thumb-bg:
//!   transparent` を指定して chakra `variant="hover"` 相当のホバー時
//!   出現を再現する）を 1 箇所の CSS 変数指定で完結させる。
//! - **hover 強調**: `viewport` の hover 時（`StateCondition::Hover`、
//!   `@media (hover: hover)` 配下）に `--fandhe-scroll-area-thumb-bg` を
//!   `--fandhe-scroll-area-thumb-hover-bg`（既定
//!   `var(--fandhe-color-fg-subtle)`）へ再定義する。hover-reveal
//!   （既定で thumb を隠し hover 時のみ出現させる chakra
//!   `variant="hover"` の既定相当）は採用しない。タッチ端末は
//!   `hover: hover` に一致せずこの再定義が発火しないため、hover-reveal
//!   を既定にすると thumb が恒久的に不可視になり発見性を損なう
//!   （常時表示 + hover 強調の方が広い入力デバイスで安全）。
//!   [`crate::recipe::hover_surface_declarations`] は使わない
//!   （`background: var(--fandhe-hover-bg)` は面全体を塗る宣言であり、
//!   変更したいのは thumb 色のみのため）。
//! - **フォーカスリングの canonical 化**: 手書きの
//!   `outline: 2px solid var(--fandhe-color-accent); outline-offset:
//!   -2px` を [`crate::recipe::focus_ring_declarations`]
//!   （`FocusRingColor::Token`・`FocusRingOffset::Inset`）へ置換した。
//!   `palette` 軸を持たないため `Token`、`root` の `overflow: hidden`
//!   内にリングを収めるため `Inset`（[`crate::splitter`]/[`crate::listbox`]
//!   と同じ判断、`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
//!   §3 参照）。
//! - **thumb の見た目**: `border-radius: var(--fandhe-radius-full)` に
//!   加え `border: 2px solid transparent; background-clip: content-box`
//!   を付与し、トラック内側へ 2px inset した「細いつまみ」に寄せた
//!   （参照サイトの Scrollbar 実装が持つ余白表現の近似）。
//! - **スクロールバー太さの調整余地**: `::-webkit-scrollbar` の
//!   width/height を固定 `0.5rem` から custom property
//!   `--fandhe-scroll-area-scrollbar-size`（既定 `0.5rem`）へ変更した。
//!   `scrollbar-width: thin` は数値指定を受け付けない
//!   （Firefox の仕様上の制約）ため、本 custom property は
//!   Chromium/WebKit 系にのみ効く（利用側は認識しておくこと）。
//!
//! 上記 3 個の custom property はすべて `--fandhe-scroll-area-` を
//! プレフィックスとし、`crates/docs-site/tests/css_var_scope_prefix.rs`
//! の scope 一致契約（`--fandhe-<scope>-*`）を満たす。値はいずれも
//! ソースコード中の固定リテラル + テーマ変数参照のみで構成され、動的
//! 入力は混入しない。
//!
//! ## 意図的に合わせなかった点
//!
//! - **transition なし**: `scrollbar-color`・`::-webkit-scrollbar-*`・
//!   custom property の再定義はブラウザによって値の補間（transition）が
//!   行われないため、`transition` 宣言を追加しても実際には効果がない
//!   dead CSS になる。参照サイトも thumb 色の transition を実質持たない。
//! - **disabled 概念なし**: headless 層（`crates/headless-ui/src/scroll_area.rs`）
//!   が `data-disabled` を出力しないため、disabled 状態表現は不要。
//! - **thumb 既定色のコントラスト比**: 既定 `border-emphasized`
//!   （light: #b3b3b3 on #ffffff ≈ 2.0:1、dark: #525252 on #111111 ≈
//!   2.5:1）は WCAG の非テキストコントラスト基準 3:1 を下回るが、thumb の
//!   位置はネイティブスクロールバーと冗長な補助的視覚情報であり、
//!   hover・キーボードフォーカス時には `fg-subtle`（4.5:1 以上）へ
//!   到達するため意図的な差分としている。
//!
//! # variant は非提供（イシュー #825 判断、#1584 で再確認）
//!
//! chakra-ui の ScrollArea が持つ `variant="hover"/"always"`・Radix Themes
//! の `size`/`type` 相当の variant 軸は採用しない
//! （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4(d)
//! で ScrollArea は「size を持たない」部品に分類済み）。`::-webkit-scrollbar`
//! 系規則は [`crate::recipe::SlotRecipe`] の宣言 API（`{`/`}`/`;` を含む値を
//! 拒否する）で表現できず固定文字列として追記する構成のため、variant ごとに
//! 出し分けようとすると手書き定数が variant 分岐で分裂し、決定性・保守性が
//! 悪化する。上記「参考サイト基準へのスタイル調整」節のとおり、custom
//! property の上書きで chakra `variant` 相当を利用側から再現できる
//! escape hatch を用意しているため、軸追加の必要性は低いと判断する。
//! 必要になった時点で再評価する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - JS によるスクロール位置追従（thumb の位置・サイズをスクロール量に応じて
//!   同期する処理）・thumb の drag 操作。
//! - ネイティブスクロールバーを完全に隠して独自スクロールバーへ置き換える
//!   JS（`scrollbar-width: none` 相当のクロスブラウザ制御）。
//! - `variant`（hover/always 等）・`size` variant 軸。
//! - `crate::table::scroll_area`（`crates/pre-styled-ui/src/table.rs`、
//!   イシュー #1572/#1843）は別スコープ（`table`）専用のスクロール
//!   コンテナであり、thumb 色が旧来の `border` のまま本モジュールと
//!   意匠が乖離するが、本イシューでは触れない（別途 Issue 化を検討）。

use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, FocusRingColor, FocusRingOffset, SlotRecipe, StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。上記「variant は非提供
// （イシュー #825 判断、#1584 で再確認）」節のとおり variant 軸を持たず
// （規約 B-2）、CSS 到達は [data-scope]/[data-part] 属性セレクタのみに
// 依存する（規約 B-3、イシュー #1062 規約参照）。
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
            vec![
                decl("position", "relative"),
                decl("overflow", "hidden"),
                // 既定 thumb 色（イシュー #1584）。`border-emphasized`
                // 未定義な `Theme::empty()` ベースのカスタムテーマでも
                // `border` へフォールバックし、スクロールバーの視認性が
                // 失われない。custom property は inherit されるため
                // `root` で宣言することで、`viewport`（`root` の子孫要素）
                // へ継承される。`viewport` 側では通常時の値を再宣言しない
                // ことで、利用側が `root` のインライン style で
                // `--fandhe-scroll-area-thumb-bg` を上書きした場合に
                // その値が `viewport` 側の宣言に上書きされず有効になる
                // （`root` へ `transparent` を指定して hover-reveal を
                // 再現する使用例〔`showcase.rs` 参照〕が機能するための
                // 前提）。`::-webkit-scrollbar-thumb`（stylesheet() 側）も
                // この同じ custom property を参照することで、利用側が
                // 1 箇所の上書きで両ブラウザ系統の thumb 色を揃って
                // 変更できる。
                decl(
                    "--fandhe-scroll-area-thumb-bg",
                    "var(--fandhe-color-border-emphasized, var(--fandhe-color-border))",
                ),
            ],
        )
        .base(
            "viewport",
            vec![
                decl("height", "100%"),
                decl("width", "100%"),
                decl("overflow", "auto"),
                decl("scrollbar-width", "thin"),
                decl(
                    "scrollbar-color",
                    "var(--fandhe-scroll-area-thumb-bg) transparent",
                ),
            ],
        )
        .base("content", vec![decl("display", "block")])
        .base("scrollbar", vec![decl("display", "none")])
        .base("thumb", vec![decl("display", "none")])
        .base("corner", vec![decl("display", "none")])
        // キーボード操作時のみのフォーカスリング（viewport は tabindex="0"
        // を固定付与するフォーカス可能領域、`crate::dialog`/`crate::tooltip`
        // と同じ判断）。イシュー #1584 で canonical ヘルパへ移行（`Token`:
        // palette 軸なし、`Inset`: `root` の `overflow: hidden` 内にリングを
        // 収めるため、`crate::splitter`/`crate::listbox` と同じ判断）。
        .state(
            "viewport",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Inset),
        )
        // hover 時に thumb を強調する（イシュー #1584）。custom property の
        // 再定義のみで、面全体を塗る `hover_surface_declarations()` は
        // 使わない（本モジュール冒頭 doc「hover 強調」節参照）。
        .state(
            "viewport",
            StateCondition::Hover,
            vec![decl(
                "--fandhe-scroll-area-thumb-bg",
                "var(--fandhe-scroll-area-thumb-hover-bg, var(--fandhe-color-fg-subtle))",
            )],
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
         width: var(--fandhe-scroll-area-scrollbar-size, 0.5rem);\n  \
         height: var(--fandhe-scroll-area-scrollbar-size, 0.5rem);\n}\n\
         [data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar-track {\n  \
         background: transparent;\n}\n\
         [data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar-thumb {\n  \
         background: var(--fandhe-scroll-area-thumb-bg);\n  \
         border-radius: var(--fandhe-radius-full);\n  \
         border: 2px solid transparent;\n  \
         background-clip: content-box;\n}\n\
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
        // イシュー #1584 PR #1858 codex-review 指摘: 既定 thumb 色は
        // `viewport` ではなく `root` の base 宣言に含まれる（custom
        // property の inherit を利用側の `root` インライン style 上書きで
        // 妨げないため）。よって `root` のブロックはこの 3 行のみで
        // クローズしない（続けて thumb-bg 宣言がある）ことを確認する。
        assert!(css.contains(
            "[data-scope=\"scroll-area\"][data-part=\"root\"] {\n  position: relative;\n  overflow: hidden;\n  --fandhe-scroll-area-thumb-bg: var(--fandhe-color-border-emphasized, var(--fandhe-color-border));\n}\n"
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
        assert!(css.contains("var(--fandhe-scroll-area-thumb-bg)"));
        assert!(css.contains("var(--fandhe-radius-full)"));
        assert!(css.contains("var(--fandhe-scroll-area-scrollbar-size, 0.5rem)"));
    }

    #[test]
    fn viewport_declares_focus_visible_ring_via_canonical_helper() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="scroll-area"][data-part="viewport"]:focus-visible {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));"));
    }

    #[test]
    fn viewport_base_block_does_not_redeclare_thumb_bg_default() {
        // イシュー #1584 PR #1858 codex-review 指摘（Bugbot 同一箇所指摘）
        // の回帰テスト: `viewport` の通常時（`:hover` を含まない）base
        // 宣言が `--fandhe-scroll-area-thumb-bg` を再宣言すると、custom
        // property は最も詳細度の高い宣言が勝つため `root` のインライン
        // style での上書き（`showcase.rs` の hover-reveal 例が案内する
        // `--fandhe-scroll-area-thumb-bg: transparent`）が `viewport` 側の
        // 宣言に打ち消され機能しなくなる。よって `viewport` の base 宣言
        // ブロック本文には `--fandhe-scroll-area-thumb-bg` を含めない
        // （`:hover` 状態での再定義〔強調表示〕は許容し続ける）。
        let css = stylesheet();
        let viewport_base_start = css
            .find("[data-scope=\"scroll-area\"][data-part=\"viewport\"] {\n")
            .expect("viewport base ブロックが見つかりません");
        let viewport_base_end = css[viewport_base_start..]
            .find("\n}\n")
            .expect("viewport base ブロックの終端が見つかりません");
        let viewport_base_block =
            &css[viewport_base_start..viewport_base_start + viewport_base_end];
        assert!(
            !viewport_base_block.contains("--fandhe-scroll-area-thumb-bg:"),
            "viewport の通常時 base 宣言が thumb-bg 既定値を再宣言しています: {viewport_base_block}"
        );
    }

    #[test]
    fn root_thumb_bg_custom_property_has_theme_fallback() {
        // イシュー #1584 PR #1858 codex-review 指摘: 既定 thumb 色は
        // `root` の base 宣言（`viewport` ではない）。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"scroll-area\"][data-part=\"root\"] {\n  position: relative;\n  overflow: hidden;\n  --fandhe-scroll-area-thumb-bg: var(--fandhe-color-border-emphasized, var(--fandhe-color-border));\n}\n"
        ));
    }

    #[test]
    fn viewport_hover_strengthens_thumb_color_within_hover_media_query() {
        // イシュー #1584: hover 時に thumb 色を強調する。タッチ端末での
        // hover 貼り付き対策として `@media (hover: hover)` 配下（イシュー
        // #1425）へ集約出力される契約を確認する。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover) {"));
        assert!(css.contains(
            r#"[data-scope="scroll-area"][data-part="viewport"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains(
            "--fandhe-scroll-area-thumb-bg: var(--fandhe-scroll-area-thumb-hover-bg, var(--fandhe-color-fg-subtle));"
        ));
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

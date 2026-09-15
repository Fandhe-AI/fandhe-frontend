//! カスタムカーソル（cursor 部品、Motion+ Cursor 相当。opt-in、イシュー
//! #2542、親 #2530/#2476）。
//!
//! Motion+（購入者限定資料）の Cursor（ポインタに spring で追従する
//! カスタムカーソル）から着想した効果を、`docs/design/
//! motion-reference-adoption-policy.md` §9（参照は可・転写は禁止）に
//! 従って Rust/CSS で独自に再実装する。
//!
//! # マークアップ・状態機械
//!
//! [`cursor`] が返すのはネイティブカーソルを置き換える固定位置の
//! `<div>` 1 個（`root` 配下に呼び出し側が 1 個だけ配置する想定）。
//! ポインタ座標の追従・hover 対象の解決・`data-*` の書き換えは
//! `fandhe-frontend-wasm-full::cursor` が担う（3 層構成、モジュール doc
//! 参照。本モジュールは `wasm-full` に依存しないため、hover 対象へ
//! 付与する opt-in 属性（[`CURSOR_TARGET_ATTR`] 等）はこの `<div>` 自体
//! ではなく、利用者が任意の要素へ静的に付与する）。
//!
//! # `motion` feature 配下に置く理由
//!
//! [`crate`]（lib.rs）冒頭 doc「Cargo feature `motion`」節の gating 契約に
//! 従い、本モジュールは `#[cfg(feature = "motion")]` 配下にのみ存在する。
//! [`crate::theme::Theme::to_css`] 本体は変更せず、
//! [`crate::theme::Theme::to_css_with_cursor`] を別 impl ブロックとして
//! 追加し `to_css()` の出力へ [`CURSOR_CSS`] を追記するだけの opt-in
//! メソッドにする（[`crate::border_beam::Theme::to_css_with_border_beam`]
//! と同型のパターン）。
//!
//! # styled 部品の公開 CSS 関数を持たない
//!
//! [`crate::border_beam`]/[`crate::button_motion`] と同じ理由
//! （各モジュール doc「styled 部品の公開 CSS 関数を持たない」節参照）
//! により、本モジュールは `crates/pre-styled-ui/src/stylesheet.rs` の
//! `all_styled_component_css_covers_every_component_module` が要求する
//! 空引数の CSS 取得関数を持たない。
//!
//! # `data-*` 属性・CSS カスタムプロパティの命名（他クレートとの契約）
//!
//! 本モジュールの定数群は `fandhe-frontend-wasm-full::cursor`
//! （`data-*` 属性）・`fandhe-frontend-animation::cursor`（CSS カスタム
//! プロパティ名）の同名定数のリテラルの写しである（本クレートは
//! いずれにも依存しないため型共有はできない）。ドリフトは
//! `crates/pre-styled-ui/tests/cursor_attr_drift.rs` が両ソースを
//! 実行時に読んで fail-closed に検知する（`button_motion_attr_drift.rs`
//! と同型のパターン）。
//!
//! # 二重のフェイルセーフ（JS 側と CSS 側）
//!
//! `fandhe-frontend-wasm-full::cursor::wire_cursor` は
//! `prefers-reduced-motion: reduce`・`pointer: coarse` のいずれかが真な
//! ら配線自体を行わず、`root` へ [`CURSOR_ACTIVE_ATTR`] を付与しない
//! （wire しない限りネイティブカーソルは隠されない）。[`CURSOR_CSS`] は
//! これとは独立に、同じ 3 条件（`prefers-reduced-motion: reduce` /
//! `pointer: coarse` / `hover: none`）を `@media` で判定し、カーソル
//! 要素自体を非表示化し `cursor: auto` へ戻す（無 JS 環境・JS 未配線の
//! いずれでも安全側になる、Issue #2542 受け入れ条件「JS 側と CSS 側の
//! 二重のフェイルセーフ」）。

use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, Node};

/// opt-in（著者が SSR 出力に静的に付与）: カスタムカーソル要素本体への
/// マーカー（値なし存在属性）。`fandhe-frontend-wasm-full::cursor::
/// CURSOR_ATTR` の写し。
pub const CURSOR_ATTR: &str = "data-fandhe-cursor";
/// opt-in（著者が SSR 出力に静的に付与）: hover 対象へのマーカー。
/// `fandhe-frontend-wasm-full::cursor::CURSOR_TARGET_ATTR` の写し。
pub const CURSOR_TARGET_ATTR: &str = "data-fandhe-cursor-target";
/// opt-in（著者が SSR 出力に静的に付与）: hover 中にカーソルへ表示する
/// 短いラベルテキスト。`fandhe-frontend-wasm-full::cursor::
/// CURSOR_TARGET_LABEL_ATTR` の写し。
pub const CURSOR_TARGET_LABEL_ATTR: &str = "data-fandhe-cursor-target-label";
/// opt-in（著者が SSR 出力に静的に付与）: hover 中はポインタ座標では
/// なく対象の中心へ吸着する（値なし存在属性）。
/// `fandhe-frontend-wasm-full::cursor::CURSOR_TARGET_MAGNETIC_ATTR` の写し。
pub const CURSOR_TARGET_MAGNETIC_ATTR: &str = "data-fandhe-cursor-target-magnetic";
/// 状態（wasm-full が root へ 1 回だけ付与）。
/// `fandhe-frontend-wasm-full::cursor::CURSOR_ACTIVE_ATTR` の写し。
pub const CURSOR_ACTIVE_ATTR: &str = "data-fandhe-cursor-active";
/// 状態（カーソル要素へ動的に付け外す）: `"hidden"`/`"idle"`/`"hover"`。
/// `fandhe-frontend-wasm-full::cursor::CURSOR_STATE_ATTR` の写し。
pub const CURSOR_STATE_ATTR: &str = "data-fandhe-cursor-state";
/// 状態（カーソル要素へ動的に付け外す）: hover 対象の
/// [`CURSOR_TARGET_ATTR`] 値の写し。
/// `fandhe-frontend-wasm-full::cursor::CURSOR_VARIANT_ATTR` の写し。
pub const CURSOR_VARIANT_ATTR: &str = "data-fandhe-cursor-variant";
/// 状態（カーソル要素へ動的に付け外す）: hover 対象の
/// [`CURSOR_TARGET_LABEL_ATTR`] 値の写し。
/// `fandhe-frontend-wasm-full::cursor::CURSOR_LABEL_ATTR` の写し。
pub const CURSOR_LABEL_ATTR: &str = "data-fandhe-cursor-label";
/// カーソル要素の位置（CSS ピクセル、X 軸）。
/// `fandhe-frontend-animation::cursor::CURSOR_X_PROPERTY` の写し。
pub const CURSOR_X_PROPERTY: &str = "--fandhe-motion-cursor-x";
/// カーソル要素の位置（CSS ピクセル、Y 軸）。
/// `fandhe-frontend-animation::cursor::CURSOR_Y_PROPERTY` の写し。
pub const CURSOR_Y_PROPERTY: &str = "--fandhe-motion-cursor-y";

/// カスタムカーソル要素本体を組み立てる。`root` 配下に 1 個だけ配置する
/// （モジュール doc「マークアップ・状態機械」節）。装飾要素のため
/// `aria-hidden="true"` を付与し、子要素は持たない（ラベルは CSS
/// `attr()` で描画する、[`CURSOR_CSS`] 参照）。
///
/// 呼び出し側の `attrs` に `class` を含めても無視される
/// （[`crate::class_attr::drop_class_attr`]、既存 styled 部品と同じ契約）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_pre_styled_ui::cursor::cursor;
///
/// let node = cursor(vec![]);
/// ```
#[must_use]
pub fn cursor(attrs: Vec<(&str, &str)>) -> Node {
    let mut all_attrs = vec![(CURSOR_ATTR, ""), ("aria-hidden", "true")];
    all_attrs.extend(crate::class_attr::drop_class_attr(attrs));
    el("div", all_attrs, vec![])
}

/// cursor 装飾の CSS 全文（末尾改行付き）。
///
/// `<` を含まない（[`crate::stylesheet::StyleSheet::push_css`] を通る、
/// [`crate::border_beam::BORDER_BEAM_CSS`] と同じ不変条件）。
pub const CURSOR_CSS: &str = concat!(
    "[data-fandhe-cursor] {\n",
    "  position: fixed;\n",
    "  top: 0;\n",
    "  left: 0;\n",
    "  width: 20px;\n",
    "  height: 20px;\n",
    "  margin: 0;\n",
    "  border-radius: var(--fandhe-radius-full, 9999px);\n",
    "  background: var(--fandhe-color-accent);\n",
    "  z-index: var(--fandhe-z-index-max, 2147483647);\n",
    "  pointer-events: none;\n",
    "  transform: translate(var(--fandhe-motion-cursor-x, -100px), var(--fandhe-motion-cursor-y, -100px)) translate(-50%, -50%);\n",
    "  transition-property: width, height, opacity, background-color, border-radius;\n",
    "  transition-duration: var(--fandhe-motion-duration-fast);\n",
    "  transition-timing-function: var(--fandhe-motion-easing-standard);\n",
    "  opacity: 1;\n",
    "}\n",
    "[data-fandhe-cursor]:not([data-fandhe-cursor-state]) {\n",
    "  display: none;\n",
    "}\n",
    "[data-fandhe-cursor][data-fandhe-cursor-state=\"hidden\"] {\n",
    "  opacity: 0;\n",
    "}\n",
    "[data-fandhe-cursor][data-fandhe-cursor-variant=\"ring\"] {\n",
    "  width: 40px;\n",
    "  height: 40px;\n",
    "  background: transparent;\n",
    "  border: 2px solid var(--fandhe-color-accent);\n",
    "}\n",
    "[data-fandhe-cursor][data-fandhe-cursor-label]:not([data-fandhe-cursor-label=\"\"]) {\n",
    "  width: auto;\n",
    "  height: auto;\n",
    "  padding: 4px 10px;\n",
    "  border-radius: var(--fandhe-radius-full, 9999px);\n",
    "  color: var(--fandhe-color-accent-fg, #fff);\n",
    "  font-size: var(--fandhe-font-font-size-xs);\n",
    "  white-space: nowrap;\n",
    "}\n",
    // ring バリアント + ラベル同時指定時のラベル背景復元（イシュー #2542
    // レビュー指摘: ring の `background: transparent` がラベル用ピル
    // 背景を上書きしたままになり、ライト面でラベルテキストが読めない
    // 不具合の修正）。ring ルールと同じ 2 属性セレクタだが、後続の
    // ラベルルールより後（source order）に置くことでカスケードで勝つ。
    "[data-fandhe-cursor][data-fandhe-cursor-variant=\"ring\"][data-fandhe-cursor-label]:not([data-fandhe-cursor-label=\"\"]) {\n",
    "  background: var(--fandhe-color-accent);\n",
    "  border: none;\n",
    "}\n",
    "[data-fandhe-cursor][data-fandhe-cursor-label]:not([data-fandhe-cursor-label=\"\"])::after {\n",
    "  content: attr(data-fandhe-cursor-label);\n",
    "}\n",
    // `!important`: 「ネイティブカーソルを置き換える」表示契約
    // （モジュール doc「二重のフェイルセーフ」節）を、詳細度に関わらず
    // 保証する。このセレクタ自体の詳細度（0,1,0）は button recipe の
    // `[data-scope="button"][data-part="root"] { cursor: pointer }`
    // （0,2,0）に劣るため、`!important` なしでは styled button 上で
    // ネイティブカーソルが隠れずカスタムカーソルと二重表示になる
    // （PR #2583 レビュー指摘 P1-2 の是正）。無効化用 `@media` 側の
    // `cursor: auto` も同じ理由で揃える。
    "[data-fandhe-cursor-active],\n",
    "[data-fandhe-cursor-active] * {\n",
    "  cursor: none !important;\n",
    "}\n",
    "@media (prefers-reduced-motion: reduce), (pointer: coarse), (hover: none) {\n",
    "  [data-fandhe-cursor] {\n",
    "    display: none;\n",
    "  }\n",
    "  [data-fandhe-cursor-active],\n",
    "  [data-fandhe-cursor-active] * {\n",
    "    cursor: auto !important;\n",
    "  }\n",
    "}\n",
);

/// opt-in API（イシュー #2542）。[`crate::theme::Theme::to_css`] 本体は
/// 変更せず、その出力へ [`CURSOR_CSS`] を追記するだけの別 impl ブロック
/// として追加する（モジュール doc「`motion` feature 配下に置く理由」節の
/// 契約。[`crate::border_beam::Theme::to_css_with_border_beam`] と同型）。
impl crate::theme::Theme {
    /// [`crate::theme::Theme::to_css`] の出力に cursor 装飾 CSS
    /// （[`CURSOR_CSS`]）を追記して返す。
    #[must_use]
    pub fn to_css_with_cursor(&self) -> String {
        let mut out = self.to_css();
        out.push_str(CURSOR_CSS);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stylesheet::StyleSheet;
    use crate::theme::Theme;

    #[test]
    fn constants_are_stable_strings() {
        assert_eq!(CURSOR_ATTR, "data-fandhe-cursor");
        assert_eq!(CURSOR_TARGET_ATTR, "data-fandhe-cursor-target");
        assert_eq!(CURSOR_TARGET_LABEL_ATTR, "data-fandhe-cursor-target-label");
        assert_eq!(
            CURSOR_TARGET_MAGNETIC_ATTR,
            "data-fandhe-cursor-target-magnetic"
        );
        assert_eq!(CURSOR_ACTIVE_ATTR, "data-fandhe-cursor-active");
        assert_eq!(CURSOR_STATE_ATTR, "data-fandhe-cursor-state");
        assert_eq!(CURSOR_VARIANT_ATTR, "data-fandhe-cursor-variant");
        assert_eq!(CURSOR_LABEL_ATTR, "data-fandhe-cursor-label");
        assert_eq!(CURSOR_X_PROPERTY, "--fandhe-motion-cursor-x");
        assert_eq!(CURSOR_Y_PROPERTY, "--fandhe-motion-cursor-y");
    }

    #[test]
    fn does_not_use_angle_bracket_literal() {
        assert!(!CURSOR_CSS.contains('<'));
    }

    #[test]
    fn cursor_css_passes_stylesheet_push_css() {
        let mut sheet = StyleSheet::new();
        assert!(sheet.push_css(CURSOR_CSS).is_ok());
    }

    #[test]
    fn reduced_motion_media_block_hides_cursor_and_restores_native_cursor() {
        let (_, after) = CURSOR_CSS
            .split_once(
                "@media (prefers-reduced-motion: reduce), (pointer: coarse), (hover: none) {",
            )
            .expect("フェイルセーフ用 @media ブロックが見つからない");
        assert!(after.contains("display: none;"));
        assert!(after.contains("cursor: auto !important;"));
    }

    #[test]
    fn to_css_with_cursor_is_pure_append() {
        let theme = Theme::default();
        let base = theme.to_css();
        let extended = theme.to_css_with_cursor();
        assert!(extended.starts_with(&base));
        assert_eq!(extended.len(), base.len() + CURSOR_CSS.len());
    }

    #[test]
    fn cursor_renders_marker_attr_and_drops_caller_class() {
        let node = cursor(vec![("class", "attacker"), ("data-testid", "x")]);
        let rendered = fandhe_frontend_headless_ui::fandhe_frontend_core::render(&node);
        assert!(rendered.contains("data-fandhe-cursor=\"\""));
        assert!(rendered.contains("aria-hidden=\"true\""));
        assert!(rendered.contains("data-testid=\"x\""));
        assert!(!rendered.contains("class="));
    }

    #[test]
    fn cursor_escapes_attribute_values_by_default() {
        // REQ-1: `raw_html()` 以外の経路は必ず既定エスケープを経由する。
        let node = cursor(vec![("data-testid", "\"><script>alert(1)</script>")]);
        let rendered = fandhe_frontend_headless_ui::fandhe_frontend_core::render(&node);
        assert!(!rendered.contains("<script>"));
    }
}

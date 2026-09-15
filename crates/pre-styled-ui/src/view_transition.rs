//! named view transition CSS プリセット（opt-in、イシュー #2516）。
//!
//! `document.startViewTransition()` によるページ遷移の見た目を fade/slide/
//! wipe の 3 種から選べるようにする、Motion+ Curtains 相当の基本形。
//! プリセット選択のトリガー（`data-fandhe-view-transition` 属性の
//! set/remove）は `fandhe-frontend-wasm-full` 側が担い（`Runtime::
//! apply_with_view_transition_named`）、本モジュールは CSS 本体のみを持つ。
//! 両クレート間の契約は [`VIEW_TRANSITION_PRESET_ATTR`] という文字列
//! リテラルの一致のみであり、直接の Cargo 依存は発生しない（`recipe::
//! CONTENT_HEIGHT_VAR` ⇔ `wasm-full::content_height::CONTENT_HEIGHT_VAR`
//! と同型の、doc コメントで相互参照する共有定数パターン）。
//!
//! # `motion` feature 配下に置く理由
//!
//! [`crate::motion`] モジュール doc の契約をそのまま踏襲する:
//! [`crate::theme::Theme::to_css`] 本体は変更せず、その出力へ
//! [`VIEW_TRANSITION_PRESETS_CSS`] を追記するだけの別 impl ブロック
//! （[`crate::theme::Theme::to_css_with_view_transition_presets`]）として
//! 追加する（不使用時は crate サイズ・`to_css` の処理量・CSS 出力の
//! いずれも増やさない、`docs/design/motion-reference-adoption-policy.md`
//! §7 のゼロコスト方針）。
//!
//! # styled 部品の公開 CSS 関数を持たない
//!
//! [`crate::motion`] モジュール doc と同じ理由により、本モジュールは
//! `pub fn` の後ろに `css` または `stylesheet` を空引数で公開する形を
//! コメント含め一切書かない（`crates/pre-styled-ui/src/stylesheet.rs` の
//! `all_styled_component_css_covers_every_component_module` が styled
//! 部品と誤認しないようにするため）。
//!
//! # 対象範囲
//!
//! `:root[data-fandhe-view-transition="<preset>"]` 配下の
//! `::view-transition-old(root)`/`::view-transition-new(root)` のみを
//! 対象とする（ページ全体の root 遷移）。要素個別の named 遷移は
//! `crate::recipe::view_transition_name_declaration`/
//! [`crate::view_transition_name`] の責務であり、本モジュールの対象外。
//!
//! # reduced-motion（WCAG 2.3.3）
//!
//! 各 `animation-duration` は `--fandhe-motion-duration-slow` トークン
//! 経由で参照するため、[`crate::theme::Theme`] の
//! `write_reduced_motion_block`（無変更）が `prefers-reduced-motion:
//! reduce` 下で自動的に `0ms` へ上書きする既存機構がそのまま効く。
//! それに加えて、単なる 0ms 瞬間切り替えではなく UA 既定のクロスフェード
//! へ戻す UX 意図を満たすため、`@media (prefers-reduced-motion: reduce)`
//! ブロックで 3 プリセットの `::view-transition-old(root)`/
//! `::view-transition-new(root)` へ `animation: revert;` を明示的に
//! 再宣言する（CSS Conditional Rules の「同名は後勝ち」により追加の
//! `!important` は不要）。

/// [`VIEW_TRANSITION_PRESET_ATTR`] のリテラル。`concat!` はリテラル
/// トークンのみを受理し `pub const` パスを受理しないため、[`motion`]
/// モジュールの `*_keyframes_name_lit!` と同型のマクロ経由で
/// `VIEW_TRANSITION_PRESETS_CSS` の生成にも再利用する
/// （[`crate::motion`]）。
macro_rules! attr_lit {
    () => {
        "data-fandhe-view-transition"
    };
}
/// `fade` プリセットの属性値リテラル。
macro_rules! fade_value_lit {
    () => {
        "fade"
    };
}
/// `slide` プリセットの属性値リテラル。
macro_rules! slide_value_lit {
    () => {
        "slide"
    };
}
/// `wipe` プリセットの属性値リテラル。
macro_rules! wipe_value_lit {
    () => {
        "wipe"
    };
}

/// `data-fandhe-view-transition` 属性名（`fandhe-frontend-wasm-full` 側の
/// `view_transition_preset::VIEW_TRANSITION_PRESET_ATTR` と同一値。
/// 相互参照のみで Cargo 依存はしない）。
pub const VIEW_TRANSITION_PRESET_ATTR: &str = attr_lit!();

/// named view transition CSS プリセット全文（末尾改行付き）。
///
/// fade/slide/wipe 3 プリセットの `@keyframes` + 属性セレクタ規則に続けて
/// `@media (prefers-reduced-motion: reduce)` ブロックを持つ。全文字列は
/// ソースコード中の `const`/`concat!` によるコンパイル時連結のみで構成
/// され、実行時入力を一切連結しない（`<` を含まないため
/// [`crate::stylesheet::StyleSheet::push_css`] を通る）。
pub const VIEW_TRANSITION_PRESETS_CSS: &str = concat!(
    // fade プリセットが参照する `fd-motion-fade-in`/`fd-motion-fade-out` は
    // 本来 `crate::motion::KEYFRAMES_CSS`（`to_css_with_keyframes`）が持つが、
    // 本メソッドは `motion` を経由せず単独で呼ばれても fade が動く必要が
    // あるため、同名・同内容の `@keyframes` をここへ複製する（CSS の
    // 「同名は後勝ち」により両方読み込まれても値が同じなら無害）。
    "@keyframes fd-motion-fade-in {\n",
    "  from {\n    opacity: 0;\n  }\n",
    "  to {\n    opacity: 1;\n  }\n",
    "}\n",
    "@keyframes fd-motion-fade-out {\n",
    "  from {\n    opacity: 1;\n  }\n",
    "  to {\n    opacity: 0;\n  }\n",
    "}\n",
    "@keyframes fd-view-transition-slide-out-to-left {\n",
    "  to {\n    transform: translateX(-100%);\n  }\n",
    "}\n",
    "@keyframes fd-view-transition-slide-in-from-right {\n",
    "  from {\n    transform: translateX(100%);\n  }\n",
    "}\n",
    "@keyframes fd-view-transition-wipe-reveal {\n",
    "  from {\n    clip-path: inset(0 100% 0 0);\n  }\n",
    "  to {\n    clip-path: inset(0 0 0 0);\n  }\n",
    "}\n",
    ":root[",
    attr_lit!(),
    "=\"",
    fade_value_lit!(),
    // UA 既定の `::view-transition-old/new(root)` は `mix-blend-mode:
    // plus-lighter` を「疑似要素自身の CSS アニメーション」として適用して
    // おり、`animation` shorthand の上書きはそのアニメーション自体を除去
    // してしまう（静的プロパティの上書きではない）。fade（クロスフェード）
    // は同色領域で透けないよう plus-lighter 合成が必須のため、除去された
    // 分を明示的な静的宣言で補う（イシュー #2516 レビュー指摘）。
    "\"]::view-transition-old(root) {\n",
    "  animation: fd-motion-fade-out var(--fandhe-motion-duration-slow, 300ms) both;\n",
    "  mix-blend-mode: plus-lighter;\n",
    "}\n",
    ":root[",
    attr_lit!(),
    "=\"",
    fade_value_lit!(),
    "\"]::view-transition-new(root) {\n",
    "  animation: fd-motion-fade-in var(--fandhe-motion-duration-slow, 300ms) both;\n",
    "  mix-blend-mode: plus-lighter;\n",
    "}\n",
    ":root[",
    attr_lit!(),
    "=\"",
    slide_value_lit!(),
    // UA 既定の `::view-transition-old/new(root)` は `mix-blend-mode:
    // plus-lighter` を持ち、fade（クロスフェード）専用の見た目である。
    // slide/wipe は old/new を同時に重ねず片方のみが動く演出のため、
    // 既定のまま重ねると加算合成で洗い出された色になる。`normal` へ
    // 明示的に戻す（モジュール doc「対象範囲」節が触れない UA 既定
    // 上書きの根拠、イシュー #2516 レビュー指摘）。
    "\"]::view-transition-old(root) {\n",
    "  animation: fd-view-transition-slide-out-to-left var(--fandhe-motion-duration-slow, 300ms) both;\n",
    "  mix-blend-mode: normal;\n",
    "}\n",
    ":root[",
    attr_lit!(),
    "=\"",
    slide_value_lit!(),
    "\"]::view-transition-new(root) {\n",
    "  animation: fd-view-transition-slide-in-from-right var(--fandhe-motion-duration-slow, 300ms) both;\n",
    "  mix-blend-mode: normal;\n",
    "}\n",
    ":root[",
    attr_lit!(),
    "=\"",
    wipe_value_lit!(),
    "\"]::view-transition-old(root) {\n",
    "  animation: none;\n",
    "  mix-blend-mode: normal;\n",
    "}\n",
    ":root[",
    attr_lit!(),
    "=\"",
    wipe_value_lit!(),
    "\"]::view-transition-new(root) {\n",
    "  animation: fd-view-transition-wipe-reveal var(--fandhe-motion-duration-slow, 300ms) both;\n",
    "  mix-blend-mode: normal;\n",
    "}\n",
    "@media (prefers-reduced-motion: reduce) {\n",
    "  :root[",
    attr_lit!(),
    "=\"",
    fade_value_lit!(),
    "\"]::view-transition-old(root),\n",
    "  :root[",
    attr_lit!(),
    "=\"",
    fade_value_lit!(),
    "\"]::view-transition-new(root),\n",
    "  :root[",
    attr_lit!(),
    "=\"",
    slide_value_lit!(),
    "\"]::view-transition-old(root),\n",
    "  :root[",
    attr_lit!(),
    "=\"",
    slide_value_lit!(),
    "\"]::view-transition-new(root),\n",
    "  :root[",
    attr_lit!(),
    "=\"",
    wipe_value_lit!(),
    "\"]::view-transition-old(root),\n",
    "  :root[",
    attr_lit!(),
    "=\"",
    wipe_value_lit!(),
    "\"]::view-transition-new(root) {\n",
    "    animation: revert;\n",
    "  }\n",
    "}\n",
);

/// opt-in API（イシュー #2516）。[`crate::theme::Theme::to_css`] 本体は
/// 変更せず、その出力へ [`VIEW_TRANSITION_PRESETS_CSS`] を追記するだけの
/// 別 impl ブロックとして追加する（モジュール doc「`motion` feature 配下に
/// 置く理由」節の契約）。
impl crate::theme::Theme {
    /// [`crate::theme::Theme::to_css`] の出力に named view transition CSS
    /// プリセット（[`VIEW_TRANSITION_PRESETS_CSS`]）を追記して返す。
    ///
    /// `StyleSheet` 経由で個別に取り込みたい場合は
    /// `sheet.push_css(view_transition::VIEW_TRANSITION_PRESETS_CSS)` を使う。
    #[must_use]
    pub fn to_css_with_view_transition_presets(&self) -> String {
        let mut out = self.to_css();
        out.push_str(VIEW_TRANSITION_PRESETS_CSS);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_css_has_no_forbidden_angle_bracket() {
        assert!(!VIEW_TRANSITION_PRESETS_CSS.contains('<'));
    }

    #[test]
    fn presets_css_passes_stylesheet_push_css() {
        let mut sheet = crate::stylesheet::StyleSheet::new();
        assert!(sheet.push_css(VIEW_TRANSITION_PRESETS_CSS).is_ok());
    }

    #[test]
    fn to_css_with_view_transition_presets_is_pure_append() {
        let theme = crate::theme::Theme::default();
        let base = theme.to_css();
        let extended = theme.to_css_with_view_transition_presets();
        assert!(extended.starts_with(&base));
        assert_eq!(
            extended.len(),
            base.len() + VIEW_TRANSITION_PRESETS_CSS.len()
        );
    }
}

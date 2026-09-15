//! Carousel の Motion+ Carousel 相当拡張（opt-in、イシュー #2541、
//! `docs/design/motion-reference-adoption-policy.md` §4 Carousel 行）。
//!
//! `docs/design/motion-reference-adoption-policy.md` §9「ライセンス・転記
//! 制限」に従い、Motion+（購入者限定資料）の意匠から着想した表現を
//! Rust/CSS で独自に再実装する（コード片の逐語コピーはしない）。
//!
//! - **coverflow 3D**: A 群（CSS のみ）。[`COVERFLOW_CLASS`] を
//!   [`crate::carousel::root`] へ付与し、各 `item` へ
//!   [`crate::recipe::stagger_index_style`]（既存 `--fandhe-motion-
//!   stagger-index`、著者が並び順を書き込む）を付与すると 3D に並ぶ。
//! - **ドラッグ + spring スナップ**: C 群（フレームループ必須）。DOM 配線は
//!   `fandhe-frontend-wasm-full::carousel_motion`（`fandhe-frontend-
//!   animation::carousel::CarouselTrack` を消費）が担う。本モジュールは
//!   マークアップ属性・CSS のみを供給する。
//!
//! # `to_css()` 本体を変更しない理由
//!
//! [`crate::button_motion`] モジュール doc「`to_css()` 本体を変更しない
//! 理由」節と同じ契約: 既存 [`crate::carousel`] の `recipe()`/`stylesheet()`
//! 本体は一切変更せず、[`Theme::to_css_with_carousel_coverflow`] を別 impl
//! ブロックとして追加し [`carousel_motion_css()`] を追記するだけの opt-in
//! メソッドにする（pure append）。
//!
//! # styled 部品の公開 CSS 関数を持たない
//!
//! [`crate::button_motion`] モジュール doc「styled 部品の公開 CSS 関数を
//! 持たない」節と同じ理由（`stylesheet.rs` の走査対象外にするため）、
//! 本モジュールも `pub fn` の後ろに `css`/`stylesheet` を空引数で公開する
//! シグネチャを持たない（[`carousel_motion_css()`] は引数なしだが名前が
//! `css`/`stylesheet` と一致しないため走査対象にならない）。
//!
//! # `data-*` 属性の命名（他クレートとの契約）
//!
//! [`CAROUSEL_DRAG_ATTR`]/[`CAROUSEL_DRAGGING_STATE_ATTR`] のリテラル値は
//! `fandhe_frontend_wasm_full::carousel_motion` の同名定数と一致すること
//! が前提（[`crate::button_motion`] と同型のリテラルの写し）。ドリフトは
//! `crates/pre-styled-ui/tests/motion_carousel_css.rs` が wasm-full 側
//! ソースを読んで fail-closed に検知する。
//!
//! # reduced-motion（WCAG 2.3.3）
//!
//! coverflow の 3D transform は既存 `item-group`/`item` の
//! `transition-duration` トークン（`--fandhe-carousel-transition-duration`
//! → `--fandhe-motion-duration-normal`）を継承するため、
//! `Theme::to_css` 側の `prefers-reduced-motion: reduce` 一括ゼロ化
//! （既存機構）がそのまま効き、個別の `@media` は不要。ドラッグ + spring
//! スナップは `fandhe_frontend_animation::reduced_motion::
//! prefers_reduced_motion` を配線層が照会し、有効時は spring を使わず
//! 即時に整数値を書く（アニメーションさせない）。

/// opt-in: root へ付与すると coverflow 3D 表示になるクラス名。
pub const COVERFLOW_CLASS: &str = "fd-carousel--coverflow";

/// opt-in（著者が SSR 出力に静的に付与）: root へ付与するとドラッグ +
/// spring スナップを有効化するマーカー属性。値は `""`（非 loop）または
/// `"loop"`（末尾からの折り返し）のみを許可する
/// （`fandhe_frontend_wasm_full::carousel_motion::parse_loop_opt_in` が
/// 解釈する契約）。
pub const CAROUSEL_DRAG_ATTR: &str = "data-fandhe-carousel-drag";

/// 状態属性: ドラッグ中〜spring 収束完了まで root へ付与される。この間
/// [`CAROUSEL_SPRING_SNAP_CSS`] が `item-group` の `transition` を
/// `none` にし、JS（`CarouselTrack`）が毎フレーム書く値と CSS
/// `transition` の競合を防ぐ。
pub const CAROUSEL_DRAGGING_STATE_ATTR: &str = "data-fandhe-carousel-dragging";

/// coverflow 3D 表示の CSS（[`COVERFLOW_CLASS`] 配下のみに閉じる）。
///
/// `--_o`（並び順の相対オフセット、`stagger-index - carousel-index`）から
/// `translateX`/`rotateY`/`translateZ` を合成する。`abs()`/`sign()` は
/// 使わず `max(x, -x)` で絶対値を表現する（`recipe.rs` 全体の既存方針
/// ——CSS 関数の対応状況に依存しない書き方——を踏襲）。
pub const CAROUSEL_COVERFLOW_CSS: &str = concat!(
    "[data-scope=\"carousel\"][data-part=\"item-group\"].",
    "fd-carousel--coverflow",
    " {\n",
    "  transform: none;\n",
    "  perspective: var(--fandhe-carousel-coverflow-perspective, 800px);\n",
    "  transform-style: preserve-3d;\n",
    "  justify-content: center;\n",
    "  position: relative;\n",
    "}\n",
    "[data-scope=\"carousel\"][data-part=\"item\"].",
    "fd-carousel--coverflow",
    " {\n",
    "  --_o: calc(var(--fandhe-motion-stagger-index, 0) - var(--fandhe-carousel-index, 0));\n",
    "  --_abs-o: max(var(--_o), calc(-1 * var(--_o)));\n",
    "  position: absolute;\n",
    "  left: 0;\n",
    "  transform: translateX(calc(var(--_o) * var(--fandhe-carousel-coverflow-spread, 55%)))\n",
    "    rotateY(calc(clamp(-1, var(--_o), 1) * -1 * var(--fandhe-carousel-coverflow-angle, 45deg)))\n",
    "    translateZ(calc(-1 * var(--_abs-o) * var(--fandhe-carousel-coverflow-depth, 60px)));\n",
    "}\n",
    // 縦方向は translateY/rotateX に置換した同型の 1 ブロック。
    "[data-scope=\"carousel\"][data-part=\"item-group\"][data-orientation=\"vertical\"].",
    "fd-carousel--coverflow",
    " {\n",
    "  transform: none;\n",
    "}\n",
    "[data-scope=\"carousel\"][data-part=\"item\"][data-orientation=\"vertical\"].",
    "fd-carousel--coverflow",
    " {\n",
    "  --_o: calc(var(--fandhe-motion-stagger-index, 0) - var(--fandhe-carousel-index, 0));\n",
    "  --_abs-o: max(var(--_o), calc(-1 * var(--_o)));\n",
    "  position: absolute;\n",
    "  top: 0;\n",
    "  transform: translateY(calc(var(--_o) * var(--fandhe-carousel-coverflow-spread, 55%)))\n",
    "    rotateX(calc(clamp(-1, var(--_o), 1) * var(--fandhe-carousel-coverflow-angle, 45deg)))\n",
    "    translateZ(calc(-1 * var(--_abs-o) * var(--fandhe-carousel-coverflow-depth, 60px)));\n",
    "}\n",
);

/// ドラッグ + spring スナップ opt-in 配下の CSS（[`CAROUSEL_DRAG_ATTR`]
/// 付き root 配下のみに閉じる）。ドラッグ中〜settle 完了中
/// （[`CAROUSEL_DRAGGING_STATE_ATTR`]）は CSS `transition` を止め、
/// `fandhe_frontend_animation::carousel::CarouselTrack` の毎フレーム
/// 書き込みだけが見た目を駆動するようにする。
pub const CAROUSEL_SPRING_SNAP_CSS: &str = concat!(
    "[data-fandhe-carousel-drag] [data-scope=\"carousel\"][data-part=\"item-group\"] {\n",
    "  touch-action: pan-y;\n",
    "  cursor: grab;\n",
    "}\n",
    "[data-fandhe-carousel-drag][data-fandhe-carousel-dragging] [data-scope=\"carousel\"][data-part=\"item-group\"],\n",
    "[data-fandhe-carousel-drag][data-fandhe-carousel-dragging] [data-scope=\"carousel\"][data-part=\"item\"] {\n",
    "  transition: none;\n",
    "  cursor: grabbing;\n",
    "}\n",
);

/// [`CAROUSEL_COVERFLOW_CSS`] と [`CAROUSEL_SPRING_SNAP_CSS`] を連結した
/// もの（[`Theme::to_css_with_carousel_coverflow`] が追記する CSS 全体）。
#[must_use]
pub fn carousel_motion_css() -> String {
    let mut out =
        String::with_capacity(CAROUSEL_COVERFLOW_CSS.len() + CAROUSEL_SPRING_SNAP_CSS.len());
    out.push_str(CAROUSEL_COVERFLOW_CSS);
    out.push_str(CAROUSEL_SPRING_SNAP_CSS);
    out
}

impl crate::theme::Theme {
    /// [`crate::theme::Theme::to_css`] の出力に Carousel の coverflow /
    /// ドラッグ + spring スナップ用 CSS（[`carousel_motion_css()`]）を
    /// 追記して返す。
    #[must_use]
    pub fn to_css_with_carousel_coverflow(&self) -> String {
        let mut out = self.to_css();
        out.push_str(&carousel_motion_css());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carousel_motion_css_has_no_forbidden_angle_bracket_or_at_rule() {
        let css = carousel_motion_css();
        assert!(!css.contains('<'));
        assert!(!css.contains("@keyframes"));
        assert!(!css.contains("@property"));
    }

    #[test]
    fn to_css_with_carousel_coverflow_is_pure_append() {
        let theme = crate::theme::Theme::default();
        let base = theme.to_css();
        let extended = theme.to_css_with_carousel_coverflow();
        assert!(extended.starts_with(&base));
        assert_eq!(extended.len(), base.len() + carousel_motion_css().len());
    }

    #[test]
    fn coverflow_css_scoped_to_opt_in_class() {
        let css = carousel_motion_css();
        assert!(css.contains(COVERFLOW_CLASS));
        assert!(css.contains("perspective:"));
        assert!(css.contains("rotateY("));
    }

    #[test]
    fn spring_snap_css_disables_transition_only_while_dragging() {
        let css = carousel_motion_css();
        assert!(css.contains(CAROUSEL_DRAG_ATTR));
        assert!(css.contains(CAROUSEL_DRAGGING_STATE_ATTR));
        assert!(css.contains("transition: none;"));
    }
}

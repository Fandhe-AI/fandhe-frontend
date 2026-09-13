//! 共通 `@keyframes` ライブラリ（opt-in、イシュー #2382）。
//!
//! `docs/design/motion-reference-adoption-policy.md` §4 判断記録 2「keyframes
//! の A 範囲は Phase 2 で共通 `@keyframes` を opt-in 提供」の実体。
//! spinner/skeleton/marquee/progress/toast がそれぞれ個別に持つ
//! `fd-<part>-<name>` の `@keyframes` とは別に、利用者が自作部品・カスタム
//! CSS から再利用できる汎用エントランス/強調アニメーションを 1 か所に
//! まとめる。
//!
//! # `motion` feature 配下に置く理由
//!
//! [`crate`]（lib.rs）冒頭 doc「Cargo feature `motion`」節の gating 契約
//! （不使用時は crate サイズ・[`crate::theme::Theme::to_css`] の処理量・CSS
//! 出力のいずれも増やさない）に従い、本モジュールは `#[cfg(feature =
//! "motion")]` 配下にのみ存在する。[`crate::theme::Theme::to_css`] 本体は
//! 一切変更しない（走査ループへ本モジュール由来の分岐を追加しない）。
//! 代わりに [`crate::theme::Theme::to_css_with_keyframes`] を「別 impl
//! ブロック」として
//! 追加し、`to_css()` の出力へ静的 CSS を追記するだけの opt-in メソッドに
//! する。
//!
//! # styled 部品の公開 CSS 関数を持たない
//!
//! `crates/pre-styled-ui/src/stylesheet.rs` の
//! `all_styled_component_css_covers_every_component_module` は `src/` 配下の
//! 各 `.rs` ファイルを文字列走査し、`pub fn` の後ろに `css` または
//! `stylesheet` を空引数で
//! 公開するモジュールを `all_styled_component_css` 登録必須の「styled
//! 部品」とみなす（cfg 非対応の走査のため feature off でビルド不能に
//! なる）。本モジュールは styled 部品ではなく素材ライブラリのため、その
//! 関数シグネチャ文字列を doc コメント含め一切書かない。
//!
//! # reduced-motion（WCAG 2.3.3）
//!
//! [`crate::theme::Theme::to_css`] が一括生成する `prefers-reduced-motion:
//! reduce` 規則は `--fandhe-motion-duration-*` トークンを参照する宣言にしか
//! 効かない。本ライブラリはリテラル duration（例:
//! `animation: fd-motion-bounce 1s infinite`）での利用を想定するため、
//! [`KEYFRAMES_CSS`] 自身が `@media (prefers-reduced-motion: reduce)` 内で
//! 同名 `@keyframes` を再定義する（CSS Conditional Rules により条件成立時
//! のみ有効・同名は後勝ちのため、利用者がどの `animation` 宣言で参照しても
//! 構造的に効く）。`bounce`/`shake` は静止へ、`zoom-*`/`slide-from-*` は
//! transform 成分を落として opacity のみへ縮退する（前庭障害配慮）。
//! `fade-in`/`fade-out` は動きを含まないため再定義しない。

/// `fd-motion-fade-in` の名前リテラル。[`crate::skeleton`] の
/// `pulse_keyframes_name_lit!` と同型（`decl()` の値検証は `{`/`}`/`;` を
/// 拒否するため素の識別子リテラルのみを許容する）。
macro_rules! fade_in_keyframes_name_lit {
    () => {
        "fd-motion-fade-in"
    };
}
macro_rules! fade_out_keyframes_name_lit {
    () => {
        "fd-motion-fade-out"
    };
}
macro_rules! zoom_in_keyframes_name_lit {
    () => {
        "fd-motion-zoom-in"
    };
}
macro_rules! zoom_out_keyframes_name_lit {
    () => {
        "fd-motion-zoom-out"
    };
}
macro_rules! slide_from_top_keyframes_name_lit {
    () => {
        "fd-motion-slide-from-top"
    };
}
macro_rules! slide_from_bottom_keyframes_name_lit {
    () => {
        "fd-motion-slide-from-bottom"
    };
}
macro_rules! slide_from_left_keyframes_name_lit {
    () => {
        "fd-motion-slide-from-left"
    };
}
macro_rules! slide_from_right_keyframes_name_lit {
    () => {
        "fd-motion-slide-from-right"
    };
}
macro_rules! bounce_keyframes_name_lit {
    () => {
        "fd-motion-bounce"
    };
}
macro_rules! shake_keyframes_name_lit {
    () => {
        "fd-motion-shake"
    };
}

/// フェードイン（`opacity: 0` → `1`）の `@keyframes` 名。
pub const FADE_IN_KEYFRAMES_NAME: &str = fade_in_keyframes_name_lit!();
/// フェードアウト（`opacity: 1` → `0`）の `@keyframes` 名。
pub const FADE_OUT_KEYFRAMES_NAME: &str = fade_out_keyframes_name_lit!();
/// ズームイン（`opacity`/`scale` 拡大）の `@keyframes` 名。
pub const ZOOM_IN_KEYFRAMES_NAME: &str = zoom_in_keyframes_name_lit!();
/// ズームアウト（`opacity`/`scale` 縮小）の `@keyframes` 名。
pub const ZOOM_OUT_KEYFRAMES_NAME: &str = zoom_out_keyframes_name_lit!();
/// 上からスライドインする `@keyframes` 名。
pub const SLIDE_FROM_TOP_KEYFRAMES_NAME: &str = slide_from_top_keyframes_name_lit!();
/// 下からスライドインする `@keyframes` 名。
pub const SLIDE_FROM_BOTTOM_KEYFRAMES_NAME: &str = slide_from_bottom_keyframes_name_lit!();
/// 左からスライドインする `@keyframes` 名。
pub const SLIDE_FROM_LEFT_KEYFRAMES_NAME: &str = slide_from_left_keyframes_name_lit!();
/// 右からスライドインする `@keyframes` 名。
pub const SLIDE_FROM_RIGHT_KEYFRAMES_NAME: &str = slide_from_right_keyframes_name_lit!();
/// バウンス（強調・無限反復想定）の `@keyframes` 名。
pub const BOUNCE_KEYFRAMES_NAME: &str = bounce_keyframes_name_lit!();
/// シェイク（強調・無限反復想定）の `@keyframes` 名。
pub const SHAKE_KEYFRAMES_NAME: &str = shake_keyframes_name_lit!();

/// 共通 `@keyframes` ライブラリの CSS 全文（末尾改行付き）。
///
/// 10 種の `@keyframes` に続けて `@media (prefers-reduced-motion: reduce)`
/// ブロックを持つ（モジュール doc「reduced-motion」節参照）。全文字列は
/// ソースコード中の `const`/`concat!` によるコンパイル時連結のみで構成され、
/// 実行時入力を一切連結しない（`<` を含まないため
/// [`crate::stylesheet::StyleSheet::push_css`] を通る）。
pub const KEYFRAMES_CSS: &str = concat!(
    "@keyframes ", fade_in_keyframes_name_lit!(), " {\n",
    "  from {\n    opacity: 0;\n  }\n",
    "  to {\n    opacity: 1;\n  }\n",
    "}\n",
    "@keyframes ", fade_out_keyframes_name_lit!(), " {\n",
    "  from {\n    opacity: 1;\n  }\n",
    "  to {\n    opacity: 0;\n  }\n",
    "}\n",
    "@keyframes ", zoom_in_keyframes_name_lit!(), " {\n",
    "  from {\n    opacity: 0;\n    scale: 0.95;\n  }\n",
    "  to {\n    opacity: 1;\n    scale: 1;\n  }\n",
    "}\n",
    "@keyframes ", zoom_out_keyframes_name_lit!(), " {\n",
    "  from {\n    opacity: 1;\n    scale: 1;\n  }\n",
    "  to {\n    opacity: 0;\n    scale: 0.95;\n  }\n",
    "}\n",
    "@keyframes ", slide_from_top_keyframes_name_lit!(), " {\n",
    "  from {\n    opacity: 0;\n    translate: 0 calc(-1 * var(--fandhe-motion-slide-offset, 0.5rem));\n  }\n",
    "  to {\n    opacity: 1;\n    translate: 0 0;\n  }\n",
    "}\n",
    "@keyframes ", slide_from_bottom_keyframes_name_lit!(), " {\n",
    "  from {\n    opacity: 0;\n    translate: 0 var(--fandhe-motion-slide-offset, 0.5rem);\n  }\n",
    "  to {\n    opacity: 1;\n    translate: 0 0;\n  }\n",
    "}\n",
    "@keyframes ", slide_from_left_keyframes_name_lit!(), " {\n",
    "  from {\n    opacity: 0;\n    translate: calc(-1 * var(--fandhe-motion-slide-offset, 0.5rem)) 0;\n  }\n",
    "  to {\n    opacity: 1;\n    translate: 0 0;\n  }\n",
    "}\n",
    "@keyframes ", slide_from_right_keyframes_name_lit!(), " {\n",
    "  from {\n    opacity: 0;\n    translate: var(--fandhe-motion-slide-offset, 0.5rem) 0;\n  }\n",
    "  to {\n    opacity: 1;\n    translate: 0 0;\n  }\n",
    "}\n",
    "@keyframes ", bounce_keyframes_name_lit!(), " {\n",
    "  0%, 100% {\n    translate: 0 0;\n  }\n",
    "  50% {\n    translate: 0 calc(-1 * var(--fandhe-motion-bounce-height, 0.5rem));\n  }\n",
    "}\n",
    "@keyframes ", shake_keyframes_name_lit!(), " {\n",
    "  0%, 100% {\n    translate: 0 0;\n  }\n",
    "  25% {\n    translate: calc(-1 * var(--fandhe-motion-shake-distance, 0.25rem)) 0;\n  }\n",
    "  75% {\n    translate: var(--fandhe-motion-shake-distance, 0.25rem) 0;\n  }\n",
    "}\n",
    "@media (prefers-reduced-motion: reduce) {\n",
    "  @keyframes ", zoom_in_keyframes_name_lit!(), " {\n",
    "    from {\n      opacity: 0;\n    }\n",
    "    to {\n      opacity: 1;\n    }\n",
    "  }\n",
    "  @keyframes ", zoom_out_keyframes_name_lit!(), " {\n",
    "    from {\n      opacity: 1;\n    }\n",
    "    to {\n      opacity: 0;\n    }\n",
    "  }\n",
    "  @keyframes ", slide_from_top_keyframes_name_lit!(), " {\n",
    "    from {\n      opacity: 0;\n    }\n",
    "    to {\n      opacity: 1;\n    }\n",
    "  }\n",
    "  @keyframes ", slide_from_bottom_keyframes_name_lit!(), " {\n",
    "    from {\n      opacity: 0;\n    }\n",
    "    to {\n      opacity: 1;\n    }\n",
    "  }\n",
    "  @keyframes ", slide_from_left_keyframes_name_lit!(), " {\n",
    "    from {\n      opacity: 0;\n    }\n",
    "    to {\n      opacity: 1;\n    }\n",
    "  }\n",
    "  @keyframes ", slide_from_right_keyframes_name_lit!(), " {\n",
    "    from {\n      opacity: 0;\n    }\n",
    "    to {\n      opacity: 1;\n    }\n",
    "  }\n",
    "  @keyframes ", bounce_keyframes_name_lit!(), " {\n",
    "    from, to {\n      translate: 0 0;\n    }\n",
    "  }\n",
    "  @keyframes ", shake_keyframes_name_lit!(), " {\n",
    "    from, to {\n      translate: 0 0;\n    }\n",
    "  }\n",
    "}\n",
);

/// opt-in API（イシュー #2382）。[`crate::theme::Theme::to_css`] 本体は
/// 変更せず、その出力へ [`KEYFRAMES_CSS`] を追記するだけの別 impl ブロック
/// として追加する（モジュール doc「`motion` feature 配下に置く理由」節の
/// 契約）。
impl crate::theme::Theme {
    /// [`crate::theme::Theme::to_css`] の出力に共通 `@keyframes` ライブラリ
    /// （[`KEYFRAMES_CSS`]）を追記して返す。
    ///
    /// `StyleSheet` 経由で個別に取り込みたい場合は
    /// `sheet.push_css(motion::KEYFRAMES_CSS)` を使う。
    #[must_use]
    pub fn to_css_with_keyframes(&self) -> String {
        let mut out = self.to_css();
        out.push_str(KEYFRAMES_CSS);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_NAMES: [&str; 10] = [
        FADE_IN_KEYFRAMES_NAME,
        FADE_OUT_KEYFRAMES_NAME,
        ZOOM_IN_KEYFRAMES_NAME,
        ZOOM_OUT_KEYFRAMES_NAME,
        SLIDE_FROM_TOP_KEYFRAMES_NAME,
        SLIDE_FROM_BOTTOM_KEYFRAMES_NAME,
        SLIDE_FROM_LEFT_KEYFRAMES_NAME,
        SLIDE_FROM_RIGHT_KEYFRAMES_NAME,
        BOUNCE_KEYFRAMES_NAME,
        SHAKE_KEYFRAMES_NAME,
    ];

    #[test]
    fn every_keyframes_name_is_defined_in_css() {
        for name in ALL_NAMES {
            assert!(name.starts_with("fd-motion-"));
            assert!(
                KEYFRAMES_CSS.contains(&format!("@keyframes {name} {{")),
                "KEYFRAMES_CSS に @keyframes {name} が見つからない"
            );
        }
    }

    #[test]
    fn keyframes_css_has_no_forbidden_angle_bracket() {
        assert!(!KEYFRAMES_CSS.contains('<'));
    }

    #[test]
    fn to_css_with_keyframes_is_pure_append() {
        let theme = crate::theme::Theme::default();
        let base = theme.to_css();
        let extended = theme.to_css_with_keyframes();
        assert!(extended.starts_with(&base));
        assert_eq!(extended.len(), base.len() + KEYFRAMES_CSS.len());
    }
}

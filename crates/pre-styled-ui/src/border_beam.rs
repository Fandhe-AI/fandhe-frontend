//! border-beam 装飾オプション（opt-in、イシュー #2531）。
//!
//! Motion+ `components/border-beam`（`sections/pricing-sections` の
//! border-beam も実体は同一のため対象に含む、Issue #2531 本文参照）が
//! 動機だが、[`crate`] は**単体コンポーネントとして実装しない**。card /
//! pricing 等の既存要素へ任意に付与できる opt-in 装飾として、
//! [`BORDER_BEAM_CLASS`]（class 付与のみで完結）と [`BORDER_BEAM_CSS`]
//! （周回する光の CSS のみの演出）を提供する。部品 anatomy には一切
//! 触れない（例えば [`crate::card::root`] は呼び出し側の `class` 属性を
//! [`crate::class_attr::drop_class_attr`] で破棄するため、border-beam は
//! card 自体を改造せず外側を素の `<div class="fd-border-beam">` で
//! ラップして適用する。呼び出し例は [`BORDER_BEAM_CLASS`] の doc 参照）。
//!
//! # `motion` feature 配下に置く理由
//!
//! [`crate`]（lib.rs）冒頭 doc「Cargo feature `motion`」節の gating 契約
//! （不使用時は crate サイズ・[`crate::theme::Theme::to_css`] の処理量・
//! CSS 出力のいずれも増やさない）に従い、本モジュールは `#[cfg(feature =
//! "motion")]` 配下にのみ存在する。[`crate::theme::Theme::to_css`] 本体は
//! 一切変更しない（走査ループへ本モジュール由来の分岐を追加しない）。
//! 代わりに [`crate::theme::Theme::to_css_with_border_beam`] を「別 impl
//! ブロック」として追加し、`to_css()` の出力へ静的 CSS を追記するだけの
//! opt-in メソッドにする（[`crate::motion::Theme::to_css_with_keyframes`]
//! と同型のパターン）。
//!
//! # 技術選定: `@property` を使わない理由（意図的に合わせなかった点）
//!
//! Motion+ 相当の border-beam 実装は一般に `@property --angle { syntax:
//! "<angle>"; ... }` でカスタムプロパティを型登録し、`conic-gradient(from
//! var(--angle) ...)` を `@keyframes` で滑らかに回転させる（`<angle>` 等の
//! 型登録なしに custom property を `@keyframes` でアニメーションさせても
//! ブラウザは離散的にしか値を切り替えない＝滑らかに回転しない）。
//!
//! 本クレートは CSS リテラルへの `<` 使用を [`crate::stylesheet::StyleSheet::push_css`]
//! （[`crate::css::is_valid_value`] も同型）で一律拒否する不変条件を持ち、
//! `syntax: "<angle>"` はこれに抵触するため `@property` は使えない。
//! [`crate::recipe::STAGGER_INDEX_VAR`] の doc「継承と入れ子スコープの
//! 注意」節・[`crate::scroll_area`] モジュール doc「意図的に合わせなかった
//! 点」節が同じ理由で `@property` を既に 2 箇所で見送っている先例に倣う。
//!
//! 代わりに、**`transform: rotate()`**（型登録なしでもブラウザが標準で
//! 滑らかに補間する）で光源レイヤーそのものを回転させる技法を採る:
//!
//! 1. [`BORDER_BEAM_CLASS`] を付けたラッパー `<div>`（`.fd-border-beam`）に
//!    `overflow: hidden`（回転する光源レイヤーの表示域をラッパーの矩形に
//!    切り抜く）と `padding: var(--fandhe-border-beam-width, 1px)`
//!    （枠線の太さ分だけ子要素を内側へ寄せ、光が漏れる隙間を作る）を
//!    与える。
//! 2. `.fd-border-beam::before` に `conic-gradient` を背景として持つ、
//!    ラッパーの**長辺基準**の正方形レイヤーを中央揃えで配置し、
//!    `transform: rotate()` を `@keyframes` で `0deg` → `360deg` へ
//!    アニメーションする。サイズは `top: 50%; left: 50%;`（`inset: 50%`
//!    ショートハンドは使わない。`right`/`bottom` も同時に 50% へ固定して
//!    しまい、`width`/`aspect-ratio` と衝突して高さが 0 に潰れうるため
//!    ─ イシュー #2531 レビュー指摘）+ `min-width: 200%; min-height:
//!    200%; aspect-ratio: 1;`（`width`/`height` 自体は `auto` のまま）で
//!    表現する。CSS Sizing の「移行された最小サイズ」
//!    （preferred size が両軸 `auto` かつ `aspect-ratio` を持つ場合、
//!    `min-width`/`min-height` は比率を介して互いに転送される）により、
//!    実効の最小サイズは `max(own, transferred)` となる。すなわち正方形
//!    の一辺は `2 * max(ラッパーの幅, ラッパーの高さ)` に確定する
//!    （`playwright` で 200×800 / 800×200 双方の `getComputedStyle`
//!    を実測し確認済み）。この正方形の内接円半径（一辺の半分 =
//!    `max(幅, 高さ)`）は常にラッパーの半対角線長
//!    （`sqrt(幅² + 高さ²) / 2 ≤ max(幅, 高さ) * sqrt(2) / 2`）以上と
//!    なるため、幅・高さどちらが長い縦長・横長カードでも、回転角度に
//!    関わらずラッパーの四隅を含む全域を覆う（旧実装は `width: 200%` +
//!    `aspect-ratio: 1` で正方形の一辺をラッパーの**幅のみ**から決めて
//!    いたため、縦長カードで上下辺に光が届かない不具合があった）。
//!    `z-index: -1` + ラッパーの `isolation: isolate` により、ラッパーの
//!    子要素（呼び出し側が渡した実コンテンツ）より必ず下に描画される。
//! 3. 子要素は `padding` で生まれた枠線幅の隙間を除いて回転レイヤーの
//!    上に重なるため、子要素自身の背景（不透明ならなお良い）が中央を
//!    覆い、隙間だけが光る「枠線」に見える。
//!
//! **既知の制約（意図的に受け入れる）**: (a) ラッパーの `overflow:
//! hidden` は子要素の box-shadow・フォーカスリングの`outline` 等
//! “ink overflow”（要素の境界ボックスの外側にはみ出す描画）も一緒に
//! 切り抜く。装飾が必要な要素に影・フォーカスリングを持たせたい場合は
//! 呼び出し側で影の弱い variant を選ぶ等の配慮が要る（`crates/docs-site`
//! の Card Demo は `CardVariant::Outline` を使い `shadow` を持たせない）。
//! (b) 子要素の背景が透明な場合、中央にもわずかに光が透けて見える
//! （`mask` 方式と異なり厳密な「枠線のみ」にはならない）。(c)
//! ラッパー自身に角丸を与えないと、装飾する子要素が角丸を持っていても
//! 切り抜き境界（`overflow: hidden`）は直角になる。呼び出し側で
//! `.fd-border-beam` へ `border-radius`（子要素と揃えたい値）を追加の
//! `style`/class で与えることで解消できる（`border-radius` トークンは
//! 本モジュールでは公開しない。4 トークンに閉じたスコープを保つ判断、
//! Issue #2531 本文の要求に `border-radius` 軸は含まれない）。
//!
//! # styled 部品の公開 CSS 関数を持たない
//!
//! [`crate::motion`] と同じ理由（同モジュール doc「styled 部品の公開 CSS
//! 関数を持たない」節参照）により、本モジュールは
//! `crates/pre-styled-ui/src/stylesheet.rs` の
//! `all_styled_component_css_covers_every_component_module` が要求する
//! `pub fn css`/`stylesheet`（空引数）を持たない。
//!
//! # `prefers-reduced-motion: reduce` 縮退（WCAG 2.3.3）
//!
//! [`BORDER_BEAM_CSS`] はリテラル `animation`（`--fandhe-motion-duration-*`
//! トークン非経由）で周回するため、[`crate::theme::Theme::to_css`] が
//! 一括生成する reduced-motion ブロックの対象外である。[`crate::motion`]
//! と同じ理由により本モジュール自身が `@media (prefers-reduced-motion:
//! reduce)` ブロックを持ち、回転レイヤー（`::before`）を `content: none`
//! で生成しないようにし、代わりに `overflow: hidden`/`padding` を解除して
//! `--fandhe-border-beam-width` 幅の静的な `border` へフォールバックする
//! （Issue #2531 の縮退要求。子要素の box-shadow・フォーカスリングの
//! 切り抜きも同時に解除される）。

/// border-beam を付与するための class 名。呼び出し側は装飾したい要素を
/// このクラス付きの素の `<div>` でラップする（部品 anatomy 非変更・
/// 任意要素への付与という要求を両立するための設計、モジュール doc
/// 参照）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_pre_styled_ui::border_beam::BORDER_BEAM_CLASS;
/// use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::fandhe_frontend_core::{el, Node};
///
/// fn wrap_with_border_beam(inner: Node) -> Node {
///     el("div", vec![("class", BORDER_BEAM_CLASS)], vec![inner])
/// }
/// ```
pub const BORDER_BEAM_CLASS: &str = "fd-border-beam";

/// border-beam 装飾の CSS 全文（末尾改行付き）。
///
/// 4 トークン（太さ・色・アーク幅・周期）はいずれも呼び出し側の CSS
/// カスタムプロパティ上書きで調整する（[`crate::motion::KEYFRAMES_CSS`]
/// の `--fandhe-motion-slide-offset` と同型の「`var(..., <default>)`
/// フォールバック付き公開」方式。[`crate::theme::Theme`] のトークン
/// レジストリ〔`push_color` 等〕には登録しない）:
///
/// - `--fandhe-border-beam-width`（既定 `1px`）: 枠線の太さ
/// - `--fandhe-border-beam-color`（既定 `var(--fandhe-color-accent)`）: 光の色
/// - `--fandhe-border-beam-spread`（既定 `10%`）: 光弧の長さ（アーク幅）
/// - `--fandhe-border-beam-duration`（既定 `6s`）: 1 周にかかる時間
///
/// 全文字列はソースコード中の `const`/`concat!` によるコンパイル時連結
/// のみで構成され、実行時入力を一切連結しない（`<` を含まないため
/// [`crate::stylesheet::StyleSheet::push_css`] を通る。`@property` を
/// 使わない理由はモジュール doc「技術選定」節参照）。
pub const BORDER_BEAM_CSS: &str = concat!(
    ".fd-border-beam {\n",
    "  position: relative;\n",
    "  isolation: isolate;\n",
    "  overflow: hidden;\n",
    "  padding: var(--fandhe-border-beam-width, 1px);\n",
    "}\n",
    ".fd-border-beam::before {\n",
    "  content: \"\";\n",
    "  position: absolute;\n",
    "  top: 50%;\n",
    "  left: 50%;\n",
    "  min-width: 200%;\n",
    "  min-height: 200%;\n",
    "  aspect-ratio: 1;\n",
    "  z-index: -1;\n",
    "  pointer-events: none;\n",
    "  transform: translate(-50%, -50%) rotate(0deg);\n",
    "  background: conic-gradient(from 0deg, transparent, var(--fandhe-border-beam-color, var(--fandhe-color-accent)) var(--fandhe-border-beam-spread, 10%), transparent calc(var(--fandhe-border-beam-spread, 10%) * 2));\n",
    "  animation: ", "fd-border-beam-rotate", " var(--fandhe-border-beam-duration, 6s) linear infinite;\n",
    "}\n",
    "@keyframes ", "fd-border-beam-rotate", " {\n",
    "  to {\n",
    "    transform: translate(-50%, -50%) rotate(360deg);\n",
    "  }\n",
    "}\n",
    "@media (prefers-reduced-motion: reduce) {\n",
    "  .fd-border-beam {\n",
    "    overflow: visible;\n",
    "    padding: 0;\n",
    "    border: var(--fandhe-border-beam-width, 1px) solid var(--fandhe-border-beam-color, var(--fandhe-color-border-emphasized));\n",
    "  }\n",
    "  .fd-border-beam::before {\n",
    "    content: none;\n",
    "  }\n",
    "}\n",
);

/// opt-in API（イシュー #2531）。[`crate::theme::Theme::to_css`] 本体は
/// 変更せず、その出力へ [`BORDER_BEAM_CSS`] を追記するだけの別 impl
/// ブロックとして追加する（モジュール doc「`motion` feature 配下に置く
/// 理由」節の契約。[`crate::motion::Theme::to_css_with_keyframes`] と
/// 同型）。
impl crate::theme::Theme {
    /// [`crate::theme::Theme::to_css`] の出力に border-beam 装飾 CSS
    /// （[`BORDER_BEAM_CSS`]）を追記して返す。
    ///
    /// `StyleSheet` 経由で個別に取り込みたい場合は
    /// `sheet.push_css(border_beam::BORDER_BEAM_CSS)` を使う。
    #[must_use]
    pub fn to_css_with_border_beam(&self) -> String {
        let mut out = self.to_css();
        out.push_str(BORDER_BEAM_CSS);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_name_is_expected_literal_and_uses_safe_identifier_charset() {
        assert_eq!(BORDER_BEAM_CLASS, "fd-border-beam");
        assert!(BORDER_BEAM_CLASS
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-'));
    }

    #[test]
    fn css_declares_wrapper_rotating_layer_and_reduced_motion_fallback() {
        assert!(BORDER_BEAM_CSS.contains(".fd-border-beam {"));
        assert!(BORDER_BEAM_CSS.contains(".fd-border-beam::before {"));
        assert!(BORDER_BEAM_CSS.contains("@keyframes fd-border-beam-rotate {"));
        assert!(BORDER_BEAM_CSS.contains("@media (prefers-reduced-motion: reduce) {"));
    }

    #[test]
    fn light_layer_size_is_derived_from_both_wrapper_axes_not_width_alone() {
        // イシュー #2531 レビュー指摘（codex-review P1 / Cursor Bugbot
        // High）: 旧実装の `inset: 50%; width: 200%; aspect-ratio: 1;`
        // は `inset` ショートハンドが `right`/`bottom` も 50% に固定し
        // `width: 200%`/`aspect-ratio: 1` と衝突して絶対配置ボックスの
        // 高さが 0 に潰れうる。かつ一辺をラッパーの幅のみから決めるため
        // 縦長カードで上下辺に光が届かない。`min-width`/`min-height`
        // （`width`/`height` は `auto`）+ `aspect-ratio: 1` の「移行
        // された最小サイズ」により、正方形の一辺を
        // `2 * max(幅, 高さ)` へ確定させる（モジュール doc「技術選定」
        // 節参照。playwright 実測: 200×800 / 800×200 いずれも
        // `::before` は 1604×1604px の正方形になることを確認済み）。
        assert!(!BORDER_BEAM_CSS.contains("inset: 50%"));
        assert!(!BORDER_BEAM_CSS.contains("  width: 200%;\n"));
        assert!(BORDER_BEAM_CSS.contains("  top: 50%;\n"));
        assert!(BORDER_BEAM_CSS.contains("  left: 50%;\n"));
        assert!(BORDER_BEAM_CSS.contains("  min-width: 200%;\n"));
        assert!(BORDER_BEAM_CSS.contains("  min-height: 200%;\n"));
        assert!(BORDER_BEAM_CSS.contains("  aspect-ratio: 1;\n"));
    }

    #[test]
    fn does_not_use_property_registration_or_angle_bracket_literal() {
        // モジュール doc「技術選定」節: `@property … syntax: "<angle>"` は
        // `<` を含むため使えない（`recipe.rs`/`scroll_area.rs` と同じ
        // 不変条件）。`transform: rotate()` 方式で代替している。
        assert!(!BORDER_BEAM_CSS.contains("@property"));
        assert!(!BORDER_BEAM_CSS.contains('<'));
    }

    #[test]
    fn reduced_motion_block_appears_after_normal_block_and_disables_animation() {
        let normal_idx = BORDER_BEAM_CSS
            .find(".fd-border-beam::before {")
            .expect(".fd-border-beam::before ブロックが見つからない");
        let reduced_idx = BORDER_BEAM_CSS
            .find("@media (prefers-reduced-motion: reduce) {")
            .expect("reduced-motion ブロックが見つからない");
        assert!(
            reduced_idx > normal_idx,
            "reduced-motion ブロックは通常ブロックより後に出現する必要がある"
        );
        let reduced_block = &BORDER_BEAM_CSS[reduced_idx..];
        assert!(reduced_block.contains("content: none;"));
        assert!(reduced_block.contains("border: var(--fandhe-border-beam-width, 1px) solid"));
    }

    #[test]
    fn exposes_four_customizable_tokens_with_fallback_defaults() {
        for (token, default) in [
            ("--fandhe-border-beam-width", "1px"),
            ("--fandhe-border-beam-color", "var(--fandhe-color-accent)"),
            ("--fandhe-border-beam-spread", "10%"),
            ("--fandhe-border-beam-duration", "6s"),
        ] {
            assert!(
                BORDER_BEAM_CSS.contains(&format!("var({token}, {default}")),
                "BORDER_BEAM_CSS に var({token}, {default}...) が見つからない"
            );
        }
    }

    #[test]
    fn border_beam_css_has_no_forbidden_angle_bracket() {
        assert!(!BORDER_BEAM_CSS.contains('<'));
    }

    #[test]
    fn to_css_with_border_beam_is_pure_append() {
        let theme = crate::theme::Theme::default();
        let base = theme.to_css();
        let extended = theme.to_css_with_border_beam();
        assert!(extended.starts_with(&base));
        assert_eq!(extended.len(), base.len() + BORDER_BEAM_CSS.len());
    }
}

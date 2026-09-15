//! text アニメーション部品（split-text reveal / typewriter / scramble、
//! イシュー #2532）。
//!
//! Motion+ の text 系素材に相当する効果を Rust/CSS/`fandhe-frontend-
//! animation` で独自に再実装する（`docs/design/
//! motion-reference-adoption-policy.md` §9「ライセンス・転記制限」に従い、
//! 参照〔閲覧・着想〕のみで転写は行わない）。
//!
//! # 3 種の役割分担
//!
//! - [`chars`]/[`words`]: SSR のみで完結する reveal（分割 + [`crate::recipe::
//!   stagger_index_style`] による stagger + [`TEXT_REVEAL_CSS`] の
//!   `@keyframes` アニメーション）。JS ハイドレーションを必要としない
//!   （`border_beam`/`button_motion` の rolling-text と同じ A 群、
//!   `docs/design/motion-reference-adoption-policy.md` §4）。
//! - [`typewriter`]/[`scramble`]: マークアップ・CSS のみを本モジュールが
//!   供給し、実際の文字送り・乱数置換のフレームループは
//!   `fandhe_frontend_wasm_full::text_animation`（C 群、`fandhe-frontend-
//!   animation::text_animation` の `AnimationLoop` を消費）が担う
//!   （`button_motion` の hold-to-confirm と同型の責務分界）。
//!
//! # マークアップ契約（他クレートとの契約）
//!
//! reveal は SR 用の分割前テキスト（[`SR_CLASS`]、visually-hidden）+
//! `aria-hidden="true"` の分割済み表示レイヤー（[`UNITS_CLASS`]）の 2 層。
//! typewriter/scramble も同型で、表示レイヤー（[`DISPLAY_CLASS`]）の
//! `textContent` を wasm-full 側が [`crate::recipe::stagger_index_style`]
//! と同じ「固定セレクタ + `set_text_content`」方式で書き換える前提
//! （属性値へテキストを二重に持たせない、`crates/wasm-full/src/
//! text_animation.rs` 参照）。
//!
//! # `to_css()` 本体を変更しない理由・styled 部品の公開 CSS 関数を持たない
//! 理由
//!
//! [`crate::border_beam`] モジュール doc と同じ契約: [`crate::theme::Theme::
//! to_css`] 本体は変更せず [`crate::theme::Theme::to_css_with_text_reveal`]
//! を別 impl ブロックとして追加する pure append のみ。`crates/pre-styled-ui/
//! src/stylesheet.rs` の styled 部品網羅テストの対象にもしない（同 doc
//! 「styled 部品の公開 CSS 関数を持たない」節と同じ理由）。
//!
//! # `prefers-reduced-motion: reduce` 縮退（WCAG 2.3.3）
//!
//! reveal は [`TEXT_REVEAL_CSS`] 自身が `@media (prefers-reduced-motion:
//! reduce)` で `.fd-text-reveal__unit { animation: none; }` へ縮退する
//! （`animation-delay` はリテラル `calc()` のため、duration トークンの
//! 0 化だけでは `both` の `from` 状態が遅延中に残ってしまう。[`crate::
//! border_beam`]/[`crate::motion`] と同じ理由で個別 `@media` を持つ）。
//! typewriter/scramble は JS 不在・reduced-motion のいずれでも
//! [`DISPLAY_CLASS`] に目標テキストがそのまま描画されているため追加の
//! CSS は不要（reduced-motion 判定自体は wasm-full 側の責務、`magnetic`/
//! `confetti` と同型）。

use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};

use crate::recipe::stagger_index_style;

/// reveal（[`chars`]/[`words`]）のモード種別を示す属性名（値は
/// `"chars"`/`"words"`、docs-site/検証用のマーカーでありスタイル・配線の
/// いずれもこの属性値では分岐しない）。
pub const TEXT_REVEAL_ATTR: &str = "data-fandhe-text-reveal";
/// opt-in（著者が SSR 出力に静的に付与）: typewriter を有効化するマーカー。
/// 値は保持時間（ミリ秒、10 進数）の任意上書き、または空文字列（既定値
/// 使用）。`fandhe_frontend_wasm_full::text_animation::TYPEWRITER_ATTR` と
/// 値が一致する必要がある（他クレートとの契約、ドリフト検知は
/// `crates/pre-styled-ui/tests/motion_text_reveal_css.rs` 参照）。
pub const TYPEWRITER_ATTR: &str = "data-fandhe-typewriter";
/// opt-in（著者が SSR 出力に静的に付与）: scramble を有効化するマーカー。
/// [`TYPEWRITER_ATTR`] と同じ値の意味論を持つ。
pub const SCRAMBLE_ATTR: &str = "data-fandhe-scramble";

const ROOT_CLASS: &str = "fd-text-reveal";
const SR_CLASS: &str = "fd-text-reveal__sr";
const UNITS_CLASS: &str = "fd-text-reveal__units";
const WORD_CLASS: &str = "fd-text-reveal__word";
const UNIT_CLASS: &str = "fd-text-reveal__unit";
/// typewriter/scramble の表示レイヤー。wasm-full 側がこの class を持つ
/// 要素の `textContent` を書き換える（モジュール doc「マークアップ契約」
/// 節参照）。
pub const DISPLAY_CLASS: &str = "fd-text-reveal__display";

/// text-reveal 装飾 CSS 全文（末尾改行付き）。公開トークンは
/// `--fandhe-text-reveal-step`（既定 `40ms`、stagger 1 単位あたりの遅延）
/// 1 件のみ（[`crate::border_beam::BORDER_BEAM_CSS`] と同じく `Theme` の
/// トークンレジストリには登録しない）。
pub const TEXT_REVEAL_CSS: &str = concat!(
    ".fd-text-reveal__sr {\n",
    "  position: absolute;\n",
    "  width: 1px;\n",
    "  height: 1px;\n",
    "  padding: 0;\n",
    "  margin: -1px;\n",
    "  overflow: hidden;\n",
    "  clip: rect(0, 0, 0, 0);\n",
    "  white-space: nowrap;\n",
    "  border: 0;\n",
    "}\n",
    ".fd-text-reveal__word {\n",
    "  white-space: nowrap;\n",
    "  display: inline-block;\n",
    "}\n",
    ".fd-text-reveal__unit {\n",
    "  display: inline-block;\n",
    "  animation: fd-text-reveal-in var(--fandhe-motion-duration-normal) var(--fandhe-motion-easing-standard) both;\n",
    "  animation-delay: calc(var(--fandhe-motion-stagger-index, 0) * var(--fandhe-text-reveal-step, 40ms));\n",
    "}\n",
    "@keyframes fd-text-reveal-in {\n",
    "  from {\n",
    "    opacity: 0;\n",
    "    translate: 0 0.4em;\n",
    "  }\n",
    "  to {\n",
    "    opacity: 1;\n",
    "    translate: none;\n",
    "  }\n",
    "}\n",
    "@media (prefers-reduced-motion: reduce) {\n",
    "  .fd-text-reveal__unit {\n",
    "    animation: none;\n",
    "  }\n",
    "}\n",
);

/// opt-in API（[`crate::border_beam::Theme::to_css_with_border_beam`] と
/// 同型の pure append）。
impl crate::theme::Theme {
    /// [`crate::theme::Theme::to_css`] の出力に text-reveal 装飾 CSS
    /// （[`TEXT_REVEAL_CSS`]）を追記して返す。
    #[must_use]
    pub fn to_css_with_text_reveal(&self) -> String {
        let mut out = self.to_css();
        out.push_str(TEXT_REVEAL_CSS);
        out
    }
}

/// SR 用の分割前テキストレイヤー（visually-hidden）を組み立てる。
fn sr_span(content: &str) -> Node {
    el("span", vec![("class", SR_CLASS)], vec![text(content)])
}

/// 1 文字を [`UNIT_CLASS`] の `<span>` へ包み、`index`（呼び出し側が
/// 単調増加させる非空白ユニットの通し番号）を [`stagger_index_style`] で
/// `style` へ書き込む。
fn char_unit(ch: char, index: usize) -> Node {
    let style = stagger_index_style(index);
    el(
        "span",
        vec![("class", UNIT_CLASS), ("style", style.as_str())],
        vec![text(ch)],
    )
}

/// 単語ごとに [`char_unit`] を並べ、連続する単語間は素のテキストノード
/// （単一の半角スペース）で連結する（連続空白は 1 個へ正規化、モジュール
/// doc 未記載の既知の簡略化）。`index` は呼び出し側から渡し、呼び出し後の
/// 値を返す（複数呼び出しをまたいで通し番号を維持するための薄い状態渡し）。
fn char_spans_for_word(word: &str, mut index: usize) -> (Node, usize) {
    let spans: Vec<Node> = word
        .chars()
        .map(|ch| {
            let node = char_unit(ch, index);
            index += 1;
            node
        })
        .collect();
    (el("span", vec![("class", WORD_CLASS)], spans), index)
}

/// 文字単位で分割し `@keyframes fd-text-reveal-in` で 1 文字ずつ現れる
/// reveal（イシュー #2532）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::text_reveal::chars;
///
/// let html = render(&chars("Hi"));
/// assert!(html.contains("fd-text-reveal__sr"));
/// assert!(html.contains("--fandhe-motion-stagger-index: 0"));
/// assert!(html.contains("--fandhe-motion-stagger-index: 1"));
/// ```
#[must_use]
pub fn chars(content: &str) -> Node {
    let mut index = 0usize;
    let mut children: Vec<Node> = Vec::new();
    for (i, word) in content.split_whitespace().enumerate() {
        if i > 0 {
            children.push(text(" "));
        }
        let (word_node, next_index) = char_spans_for_word(word, index);
        index = next_index;
        children.push(word_node);
    }
    let units = el(
        "span",
        vec![("class", UNITS_CLASS), ("aria-hidden", "true")],
        children,
    );
    el(
        "span",
        vec![("class", ROOT_CLASS), (TEXT_REVEAL_ATTR, "chars")],
        vec![sr_span(content), units],
    )
}

/// 単語単位で分割する reveal（イシュー #2532）。[`chars`] と同じ CSS
/// （[`UNIT_CLASS`]）を単語単位で適用する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::text_reveal::words;
///
/// let html = render(&words("Hello world"));
/// assert!(html.contains("--fandhe-motion-stagger-index: 0"));
/// assert!(html.contains("--fandhe-motion-stagger-index: 1"));
/// assert_eq!(html.matches("fd-text-reveal__unit").count(), 2);
/// ```
#[must_use]
pub fn words(content: &str) -> Node {
    let mut children: Vec<Node> = Vec::new();
    for (index, word) in content.split_whitespace().enumerate() {
        if index > 0 {
            children.push(text(" "));
        }
        children.push(char_unit_word(word, index));
    }
    let units = el(
        "span",
        vec![("class", UNITS_CLASS), ("aria-hidden", "true")],
        children,
    );
    el(
        "span",
        vec![("class", ROOT_CLASS), (TEXT_REVEAL_ATTR, "words")],
        vec![sr_span(content), units],
    )
}

/// [`words`] 専用: 単語全体を 1 個の [`UNIT_CLASS`] `<span>` へ包む
/// （[`char_unit`] と同じ class・`style` 契約だが中身は 1 文字でなく単語）。
fn char_unit_word(word: &str, index: usize) -> Node {
    let style = stagger_index_style(index);
    el(
        "span",
        vec![("class", UNIT_CLASS), ("style", style.as_str())],
        vec![text(word)],
    )
}

/// typewriter/scramble 共通の骨格: [`sr_span`] + [`DISPLAY_CLASS`] 表示
/// レイヤー（初期値は `content` そのまま、JS 不在・reduced-motion でも
/// 全文が見える）。`duration_ms` が `Some` なら属性値へ 10 進数文字列と
/// して書き込み、`None` なら空文字列（wasm-full 側の既定値を使う）。
fn frame_animation_node(attr: &'static str, content: &str, duration_ms: Option<u32>) -> Node {
    let value = duration_ms.map_or_else(String::new, |ms| ms.to_string());
    let display = el(
        "span",
        vec![("class", DISPLAY_CLASS), ("aria-hidden", "true")],
        vec![text(content)],
    );
    el(
        "span",
        vec![("class", ROOT_CLASS), (attr, value.as_str())],
        vec![sr_span(content), display],
    )
}

/// typewriter（1 文字ずつ現れる）を opt-in する（イシュー #2532）。実際の
/// 文字送りは `fandhe_frontend_wasm_full::text_animation::wire_text_animation`
/// が担う（本関数はマークアップのみ）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::text_reveal::typewriter;
///
/// let html = render(&typewriter("Hello", Some(1500)));
/// assert!(html.contains(r#"data-fandhe-typewriter="1500""#));
/// assert!(html.contains("fd-text-reveal__display"));
/// ```
#[must_use]
pub fn typewriter(content: &str, duration_ms: Option<u32>) -> Node {
    frame_animation_node(TYPEWRITER_ATTR, content, duration_ms)
}

/// scramble（乱数置換から目標テキストへ収束）を opt-in する（イシュー
/// #2532）。実際の置換ループは `fandhe_frontend_wasm_full::text_animation::
/// wire_text_animation` が担う（本関数はマークアップのみ）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::text_reveal::scramble;
///
/// let html = render(&scramble("Hello", None));
/// assert!(html.contains("data-fandhe-scramble"));
/// assert!(html.contains("Hello"));
/// ```
#[must_use]
pub fn scramble(content: &str, duration_ms: Option<u32>) -> Node {
    frame_animation_node(SCRAMBLE_ATTR, content, duration_ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn text_reveal_css_has_no_forbidden_angle_bracket() {
        assert!(!TEXT_REVEAL_CSS.contains('<'));
    }

    #[test]
    fn to_css_with_text_reveal_is_pure_append() {
        let theme = crate::theme::Theme::default();
        let base = theme.to_css();
        let extended = theme.to_css_with_text_reveal();
        assert!(extended.starts_with(&base));
        assert_eq!(extended.len(), base.len() + TEXT_REVEAL_CSS.len());
    }

    #[test]
    fn chars_splits_words_with_nowrap_wrapper_and_skips_spaces_in_index() {
        let html = render(&chars("ab cd"));
        // 4 文字（空白除く）分の index が振られる: 0,1 (ab) / 2,3 (cd)。
        for i in 0..4 {
            assert!(html.contains(&format!("--fandhe-motion-stagger-index: {i}")));
        }
        assert!(html.contains("fd-text-reveal__word"));
    }

    #[test]
    fn chars_preserves_original_text_in_sr_layer_and_escapes_html() {
        let html = render(&chars("<b>hi</b>"));
        assert!(html.contains("fd-text-reveal__sr"));
        assert!(!html.contains("<b>hi</b>"));
        assert!(html.contains("&lt;b&gt;"));
    }

    #[test]
    fn words_indexes_per_word_not_per_char() {
        let html = render(&words("foo bar baz"));
        assert!(html.contains("--fandhe-motion-stagger-index: 0"));
        assert!(html.contains("--fandhe-motion-stagger-index: 1"));
        assert!(html.contains("--fandhe-motion-stagger-index: 2"));
        assert_eq!(html.matches(r#"class="fd-text-reveal__unit""#).count(), 3);
    }

    #[test]
    fn typewriter_omits_duration_attr_value_when_none() {
        let html = render(&typewriter("hi", None));
        assert!(html.contains(r#"data-fandhe-typewriter="""#));
    }

    #[test]
    fn scramble_display_layer_is_aria_hidden_with_initial_full_text() {
        let html = render(&scramble("secret", None));
        assert!(html.contains(r#"class="fd-text-reveal__display""#));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(html.contains("secret"));
    }

    #[test]
    fn units_layer_is_aria_hidden_and_sr_layer_carries_full_text_once() {
        let html = render(&chars("hi there"));
        assert_eq!(html.matches("hi there").count(), 1);
        assert!(html.contains(r#"class="fd-text-reveal__units" aria-hidden="true""#));
    }
}

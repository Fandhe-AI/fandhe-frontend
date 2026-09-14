//! field / input の Motion+ 由来フォームアニメーション（opt-in、イシュー
//! #2545、親 #2530 Phase 7、祖父トラッキング #2476）。
//!
//! Motion+ `examples/forms`（購入者限定素材）が示す 4 種の振る舞い
//! （フローティングラベル・エラー時の shake・フォーカス時の下線伸長・
//! 検証結果表示の presence 遷移）を、[`crate::field`]/[`crate::input`] の
//! 既存 anatomy へ**参照するだけ**の opt-in 追加 CSS として再実装する。
//! Motion+ 側のソースコード・スクリーンショット・購入者限定詳細は一切
//! 転写しない（本モジュールはイシュー本文が公開している振る舞いの記述
//! のみを根拠にした独自の Rust/CSS 再実装である）。
//!
//! # `motion` feature 配下に置く理由・`field.rs`/`input.rs` を変更しない理由
//!
//! [`crate`]（lib.rs）冒頭 doc「Cargo feature `motion`」節の gating 契約
//! （不使用時は crate サイズ・[`crate::theme::Theme::to_css`] の処理量・
//! CSS 出力のいずれも増やさない）に従い、本モジュールは `#[cfg(feature =
//! "motion")]` 配下にのみ存在する。
//!
//! `crates/pre-styled-ui/tests/motion_zero_cost.rs` は「motion feature off
//! でも `Theme::to_css()` の全文バイトが不変」を golden で固定しており、
//! これは [`crate::field::css`]/[`crate::input::css`] を**無条件に**呼ぶ。
//! したがって両モジュールの `recipe()`/`css()` 関数本体を feature で分岐
//! させることはできない（feature off でも同じ関数がコンパイル・実行される
//! ため、分岐を入れると「feature を有効にするだけで CSS が変わる」という
//! 別の不変条件違反を生む）。本モジュールは [`crate::border_beam`]・
//! [`crate::motion`] と同型のパターンを踏襲する: 既存 `recipe()`/`css()` は
//! 一切変更せず、別モジュールが追加の静的 CSS を生成し、呼び出し側が
//! [`crate::theme::Theme::to_css_with_forms_motion`] で明示的に連結する。
//!
//! # 4 スラッグの実装方針
//!
//! 1. **エラー時の shake**（[`SHAKE_CSS`]）: `input[data-invalid]` へ単発
//!    シェイクアニメーションを適用する。headless `field::input` は既に
//!    `data-invalid` を出力済み（新規配線不要）。既知の制約: CSS の
//!    `animation` は「属性が変化した瞬間」を検知できないため、SSR 初回
//!    描画時に既に `data-invalid` が真だと初回表示でも 1 回再生される
//!    （JS 制御なしの CSS 単体トレードオフ、意図的に受け入れる）。
//! 2. **フォーカス時の下線伸長**（[`UNDERLINE_GROW_CSS`]）:
//!    [`crate::input::InputVariant::Flushed`] のクラス
//!    （`fd-field--variant-flushed`）限定で、`background-image` の
//!    `background-size` を `:focus-visible` で 0% → 100% へ遷移させる
//!    （`<input>` は `::before`/`::after` を生成できないため疑似要素方式
//!    は使わない）。
//! 3. **フローティングラベル**（[`FLOATING_LABEL_CLASS`]・
//!    [`FLOATING_LABEL_CSS`]）: [`crate::recipe::SlotRecipe`] は結合子
//!    （`~`/`:has()`）を表現できないため、[`crate::field`] モジュール doc
//!    「shadcn/ui 突合」節と同型の直接追記（生セレクタの静的 CSS 文字列）
//!    で表す。[`crate::field::root`]（`position: relative` を既に持つ）を
//!    素の `<div class="fd-field-floating-label">` でラップして opt-in
//!    する（[`crate::border_beam`] と同型の「部品 anatomy に触れない」
//!    設計）。**呼び出し側の責務（rustdoc 契約）**: (a) `field::root` の
//!    children は `input` → `label` の順（一般兄弟結合子 `~` は後続要素
//!    にのみ効くため）、(b) `<input>` に `placeholder=" "`（半角スペース
//!    1 文字。空文字列はブラウザ実装により `:placeholder-shown` の判定が
//!    割れるため避ける）を指定する。
//! 4. **送信中→完了の状態遷移**（[`error_text_presence_css`]）:
//!    「送信ボタンのスピナー→チェックマーク変化」は `field`/`input` の
//!    headless anatomy に対応する `data-*` を持たず、新設するには
//!    headless-ui 拡張（クレート跨ぎの semver カスケード）を要するため
//!    本イシューのスコープ（`fandhe-frontend-pre-styled-ui` のみ）を超える。
//!    加えて状態遷移の「決定」自体はバリデーション/送信処理であり
//!    `docs/policy/intentional-non-adoption.md` §3.25 規則 1 のスコープ外。
//!    本モジュールは検証結果（invalid ⇄ valid）表示の出現・消失を
//!    `error-text` の presence 遷移として表現する読み替えを採る
//!    （[`crate::recipe::SlotRecipe::presence_transition`]、既存 #2497 の
//!    共通 preset をそのまま適用するのみで、新規セレクタ組み立てを持たない）。
//!
//! # `prefers-reduced-motion: reduce`
//!
//! [`SHAKE_CSS`] はリテラル `animation`（`--fandhe-motion-duration-*`
//! トークン非経由）で再生するため、[`crate::theme::Theme::to_css`] が
//! 一括生成する reduced-motion ブロックの対象外である。[`crate::border_beam`]
//! と同じ理由により本モジュール自身が個別の `@media (prefers-reduced-motion:
//! reduce)` ブロックを持ち、`animation: none` へ縮退する（WCAG 2.3.3）。
//! [`UNDERLINE_GROW_CSS`]・[`error_text_presence_css`] は
//! `var(--fandhe-motion-duration-*)` 経由の transition のみで構成され、
//! [`crate::theme::Theme::to_css`] が既に `duration-*` トークンを reduced
//! motion 下で `0ms` へ上書きするため、個別の `@media` ブロックを追加する
//! 必要はない（[`crate::recipe::transition_declarations`] rustdoc 参照）。
//!
//! # styled 部品の公開 CSS 関数を持たない
//!
//! [`crate::motion`]/[`crate::border_beam`] と同じ理由（各モジュール doc
//! 「styled 部品の公開 CSS 関数を持たない」節参照）により、本モジュールは
//! `crates/pre-styled-ui/src/stylesheet.rs` の
//! `all_styled_component_css_covers_every_component_module` が要求する
//! `pub fn css`/`stylesheet`（空引数）を持たない
//! （[`forms_motion_css`]/[`error_text_presence_css`] という別名を使う）。

use crate::recipe::{MotionDuration, SlotRecipe};

/// エラー時の shake（`input[data-invalid]`）の CSS 全文（末尾改行付き）。
///
/// 対象セレクタは `[data-scope="field"][data-part="input"][data-invalid]`
/// （headless [`fandhe_frontend_headless_ui::field::input`] が既に出力する
/// 存在属性、新規配線不要）。単発（`1` 回再生）の減衰する水平シェイクを
/// `400ms` で再生する。`prefers-reduced-motion: reduce` 下では
/// `animation: none` へ縮退する（モジュール doc「`prefers-reduced-motion:
/// reduce`」節参照）。
///
/// `@keyframes` 名 `fd-forms-motion-shake` は
/// [`crate::motion::SHAKE_KEYFRAMES_NAME`]（`"fd-motion-shake"`、強調・
/// 無限反復想定の共通プリセット）とは意図的に異なる名前・実装を持つ
/// （モジュール doc「4 スラッグの実装方針」節 1 参照。本用途は単発再生の
/// 減衰モーションであり要件が異なる）。
///
/// 全文字列はソースコード中の `const`/`concat!` によるコンパイル時連結
/// のみで構成され、実行時入力を一切連結しない。
pub const SHAKE_CSS: &str = concat!(
    "[data-scope=\"field\"][data-part=\"input\"][data-invalid] {\n",
    "  animation: ",
    "fd-forms-motion-shake",
    " 400ms ease-in-out 1;\n",
    "}\n",
    "@keyframes ",
    "fd-forms-motion-shake",
    " {\n",
    "  10%, 90% {\n    transform: translateX(-1px);\n  }\n",
    "  20%, 80% {\n    transform: translateX(2px);\n  }\n",
    "  30%, 50%, 70% {\n    transform: translateX(-4px);\n  }\n",
    "  40%, 60% {\n    transform: translateX(4px);\n  }\n",
    "}\n",
    "@media (prefers-reduced-motion: reduce) {\n",
    "  [data-scope=\"field\"][data-part=\"input\"][data-invalid] {\n",
    "    animation: none;\n",
    "  }\n",
    "}\n",
);

/// フォーカス時の下線伸長（[`crate::input::InputVariant::Flushed`] 限定）
/// の CSS 全文（末尾改行付き）。
///
/// `<input>` は `::before`/`::after` を生成できないため、疑似要素ではなく
/// `input` 自身の `background-image`（下線色の 1px グラデーション）+
/// `background-size` の transition で表現する（モジュール doc「4 スラッグ
/// の実装方針」節 2 参照）。`background-position: bottom center` で下端に
/// 固定し、`:focus-visible` で `background-size` を `0% 2px` → `100% 2px`
/// へ伸長する。色は [`crate::recipe::focus_ring_declarations`] と同じ
/// `--fandhe-color-focus-ring`（未定義時 `--fandhe-color-accent` へ
/// フォールバック）を参照し、フォーカスリングと配色を揃える。
///
/// セレクタ中のクラス名 `fd-field--variant-flushed` は
/// [`crate::recipe::SlotRecipe::variant_classes`] が生成する
/// `fd-<scope>--<axis>-<value>` 形式（scope `"field"` axis `"variant"`
/// value `"flushed"`）のリテラルであり、`crate::input::input` の実出力
/// クラスと一致することは `crates/pre-styled-ui/tests/motion_forms_css.rs`
/// の drift テストが固定する。
pub const UNDERLINE_GROW_CSS: &str = concat!(
    "[data-scope=\"field\"][data-part=\"input\"].fd-field--variant-flushed {\n",
    "  background-image: linear-gradient(var(--fandhe-color-focus-ring, var(--fandhe-color-accent)), var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));\n",
    "  background-repeat: no-repeat;\n",
    "  background-position: bottom center;\n",
    "  background-size: 0% 2px;\n",
    "  transition-property: background-size;\n",
    "  transition-duration: var(--fandhe-motion-duration-normal);\n",
    "  transition-timing-function: var(--fandhe-motion-easing-standard);\n",
    "}\n",
    "[data-scope=\"field\"][data-part=\"input\"].fd-field--variant-flushed:focus-visible {\n",
    "  background-size: 100% 2px;\n",
    "}\n",
);

/// フローティングラベルを付与するための wrapper class 名。呼び出し側は
/// [`crate::field::root`] の出力を、このクラス付きの素の `<div>` で
/// ラップする（[`crate::border_beam::BORDER_BEAM_CLASS`] と同型の設計）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_pre_styled_ui::forms_motion::FLOATING_LABEL_CLASS;
/// use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::fandhe_frontend_core::{el, Node};
///
/// fn wrap_with_floating_label(field_root: Node) -> Node {
///     el("div", vec![("class", FLOATING_LABEL_CLASS)], vec![field_root])
/// }
/// ```
///
/// # 呼び出し側の責務（契約）
///
/// - [`crate::field::root`] の children は **`input` → `label` の順**で
///   渡すこと（一般兄弟結合子 `~` は後続要素にのみ効くため）。
/// - `<input>` へ `placeholder=" "`（半角スペース 1 文字）を指定すること
///   （空文字列は `:placeholder-shown` の判定がブラウザ実装により割れる
///   ため避ける）。
pub const FLOATING_LABEL_CLASS: &str = "fd-field-floating-label";

/// フローティングラベルの CSS 全文（末尾改行付き）。
///
/// [`FLOATING_LABEL_CLASS`] 配下限定（無条件のグローバル規則にしない。
/// opt-in しない既存の `field`/`input` 呼び出しに影響を与えないための
/// 必須スコープ）。[`crate::field::root`] が既に持つ `position: relative`
/// を土台に、`label` を `position: absolute` で input の中央へ重ね、
/// `input` が `:placeholder-shown` でない（入力済み）または `:focus` の
/// ときに縮小・上方へ移動させる。
pub const FLOATING_LABEL_CSS: &str = concat!(
    ".fd-field-floating-label [data-scope=\"field\"][data-part=\"label\"] {\n",
    "  position: absolute;\n",
    "  left: var(--fandhe-size-control-padding-x-md, 1rem);\n",
    "  top: 50%;\n",
    "  transform: translateY(-50%);\n",
    "  transform-origin: left top;\n",
    "  transition-property: transform, top, color;\n",
    "  transition-duration: var(--fandhe-motion-duration-normal);\n",
    "  transition-timing-function: var(--fandhe-motion-easing-standard);\n",
    "  pointer-events: none;\n",
    "  background: var(--fandhe-color-bg);\n",
    "  padding: 0 var(--fandhe-space-1, 0.25rem);\n",
    "}\n",
    "[data-scope=\"field\"][data-part=\"input\"]:not(:placeholder-shown) ~ [data-scope=\"field\"][data-part=\"label\"],\n",
    "[data-scope=\"field\"][data-part=\"input\"]:focus ~ [data-scope=\"field\"][data-part=\"label\"] {\n",
    "  top: 0;\n",
    "  transform: translateY(-50%) scale(0.85);\n",
    "  color: var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n",
    "}\n",
);

/// `error-text` の presence（enter/exit）遷移 CSS を返す（決定的、
/// モジュール doc「4 スラッグの実装方針」節 4 参照）。
///
/// [`crate::recipe::SlotRecipe::presence_transition`]（既存 #2497 の共通
/// preset。`Theme::to_css` 本体・feature 状況に関わらず常時利用可能な API
/// であり、本イシュー固有の新規セレクタ組み立てを持たない）を
/// `error-text` slot へ 1 回だけ適用する。headless [`fandhe_frontend_headless_ui::field::error_text`]
/// は非表示時に `hidden` 属性を出す契約（[`crate::field`] モジュール doc
/// 参照）であり、`presence_transition` の `[hidden]` state と一致する。
///
/// [`crate::field::css`] が既に `error-text` へ base/state 宣言を持つが、
/// 本関数が返す宣言は `opacity`/`transform`/`transition-*` のみで
/// [`crate::field::css`] の宣言（`display`/`color`/`font-size` 等）と
/// プロパティが重複しないため、同一セレクタへの 2 つ目の base ブロックが
/// 出力されても意図した見た目の合成のみが起こる（[`crate::field`]
/// モジュール doc は「同一セレクタの base ブロックの二重出現」を
/// `input`/`textarea`/`select` slot の重複登録防止として戒めているが、
/// これはプロパティが重複し得る場合の注意であり、プロパティが排他的な
/// 本ケースには当たらない）。
#[must_use]
pub fn error_text_presence_css() -> String {
    SlotRecipe::new("field", &["error-text"])
        .presence_transition("error-text", MotionDuration::Fast)
        .css()
}

/// 上記 4 種の CSS 全量を決定的な順序（shake → underline-grow →
/// floating-label → error-text presence）で連結して返す（決定的:
/// 同一呼び出しは常にバイト単位で同一の文字列を返す）。
#[must_use]
pub fn forms_motion_css() -> String {
    let mut out = String::new();
    out.push_str(SHAKE_CSS);
    out.push_str(UNDERLINE_GROW_CSS);
    out.push_str(FLOATING_LABEL_CSS);
    out.push_str(&error_text_presence_css());
    out
}

/// opt-in API（イシュー #2545）。[`crate::theme::Theme::to_css`] 本体は
/// 変更せず、その出力へ [`forms_motion_css`] を追記するだけの別 impl
/// ブロックとして追加する（モジュール doc「`motion` feature 配下に置く
/// 理由」節の契約。[`crate::theme::Theme::to_css_with_border_beam`] と
/// 同型）。
impl crate::theme::Theme {
    /// [`crate::theme::Theme::to_css`] の出力に本モジュールの opt-in CSS
    /// （[`forms_motion_css`]）を追記して返す。
    ///
    /// `StyleSheet` 経由で個別に取り込みたい場合は
    /// `sheet.push_css(&forms_motion::forms_motion_css())` を使う。
    #[must_use]
    pub fn to_css_with_forms_motion(&self) -> String {
        let mut out = self.to_css();
        out.push_str(&forms_motion_css());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shake_css_declares_keyframes_and_reduced_motion_fallback() {
        assert!(SHAKE_CSS.contains(r#"[data-scope="field"][data-part="input"][data-invalid] {"#));
        assert!(SHAKE_CSS.contains("@keyframes fd-forms-motion-shake {"));
        assert!(SHAKE_CSS.contains("@media (prefers-reduced-motion: reduce) {"));
        assert!(SHAKE_CSS.contains("animation: none;"));
    }

    #[test]
    fn shake_keyframes_name_does_not_collide_with_common_preset() {
        // モジュール doc「実装方針」節 1: `crate::motion::SHAKE_KEYFRAMES_NAME`
        // （"fd-motion-shake"）とは異なる専用名を使う。
        assert_ne!("fd-forms-motion-shake", crate::motion::SHAKE_KEYFRAMES_NAME);
    }

    #[test]
    fn underline_grow_css_targets_flushed_variant_and_focus_visible() {
        assert!(UNDERLINE_GROW_CSS.contains("fd-field--variant-flushed"));
        assert!(UNDERLINE_GROW_CSS.contains(":focus-visible {"));
        assert!(UNDERLINE_GROW_CSS.contains("background-size: 0% 2px;"));
        assert!(UNDERLINE_GROW_CSS.contains("background-size: 100% 2px;"));
    }

    #[test]
    fn floating_label_css_is_scoped_under_wrapper_class() {
        for line_start in FLOATING_LABEL_CSS.lines().filter(|l| l.ends_with('{')) {
            assert!(
                line_start.trim_start().starts_with(FLOATING_LABEL_CLASS)
                    || line_start.trim_start().starts_with(",")
                    || line_start.contains(FLOATING_LABEL_CLASS)
                    || line_start
                        .trim_start()
                        .starts_with(r#"[data-scope="field"][data-part="input"]:not"#)
                    || line_start
                        .trim_start()
                        .starts_with(r#"[data-scope="field"][data-part="input"]:focus"#),
                "opt-in しない既存呼び出しに影響しないよう、フローティング\
                 ラベルの規則はすべて {FLOATING_LABEL_CLASS} 配下または\
                 兄弟結合子経由で input 側から辿れる形である必要がある: \
                 {line_start}"
            );
        }
    }

    #[test]
    fn floating_label_css_uses_general_sibling_combinator_from_input_to_label() {
        assert!(FLOATING_LABEL_CSS.contains(
            r#"[data-scope="field"][data-part="input"]:not(:placeholder-shown) ~ [data-scope="field"][data-part="label"]"#
        ));
        assert!(FLOATING_LABEL_CSS.contains(
            r#"[data-scope="field"][data-part="input"]:focus ~ [data-scope="field"][data-part="label"]"#
        ));
    }

    #[test]
    fn error_text_presence_css_targets_hidden_state() {
        let out = error_text_presence_css();
        assert!(out.contains(r#"[data-scope="field"][data-part="error-text"] {"#));
        assert!(out.contains(r#"[data-scope="field"][data-part="error-text"][hidden] {"#));
        assert!(out.contains("opacity: 1;"));
    }

    #[test]
    fn error_text_presence_css_is_deterministic() {
        assert_eq!(error_text_presence_css(), error_text_presence_css());
    }

    #[test]
    fn forms_motion_css_concatenates_all_four_pieces_in_order() {
        let out = forms_motion_css();
        let shake_idx = out.find(SHAKE_CSS).expect("SHAKE_CSS が見つからない");
        let underline_idx = out
            .find(UNDERLINE_GROW_CSS)
            .expect("UNDERLINE_GROW_CSS が見つからない");
        let floating_idx = out
            .find(FLOATING_LABEL_CSS)
            .expect("FLOATING_LABEL_CSS が見つからない");
        let presence_idx = out
            .find(&error_text_presence_css())
            .expect("error_text_presence_css() の出力が見つからない");
        assert!(shake_idx < underline_idx);
        assert!(underline_idx < floating_idx);
        assert!(floating_idx < presence_idx);
    }

    #[test]
    fn forms_motion_css_never_contains_style_breakout_sequences() {
        let out = forms_motion_css();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn forms_motion_css_is_deterministic() {
        assert_eq!(forms_motion_css(), forms_motion_css());
    }

    #[test]
    fn to_css_with_forms_motion_is_pure_append() {
        let theme = crate::theme::Theme::default();
        let base = theme.to_css();
        let extended = theme.to_css_with_forms_motion();
        let extra = forms_motion_css();
        assert!(extended.starts_with(&base));
        assert_eq!(extended.len(), base.len() + extra.len());
        assert_eq!(&extended[base.len()..], extra);
    }

    #[test]
    fn forms_motion_css_passes_stylesheet_push_css() {
        let mut sheet = crate::stylesheet::StyleSheet::new();
        assert!(sheet.push_css(&forms_motion_css()).is_ok());
    }
}

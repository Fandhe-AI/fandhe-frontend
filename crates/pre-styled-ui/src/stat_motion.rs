//! Stat の Motion+ 由来 variant（opt-in、イシュー #2539、親 #2528）。
//!
//! `docs/design/motion-reference-adoption-policy.md` §9「ライセンス・転記
//! 制限」に従い、Motion+（購入者限定資料）の AnimateNumber 意匠から着想
//! した数値カウントアップ動作を Rust/CSS で独自に再実装する。実際の補間・
//! `textContent` 書き込みは `fandhe-frontend-animation::count_up`、
//! `data-*` 属性の解決・トリガー判定は `fandhe-frontend-wasm-full::count_up`
//! の責務であり、本モジュールは以下のみを担う（3 層構成、`docs/design/
//! motion-reference-adoption-policy.md` §6）:
//!
//! 1. `wasm-full` 側の `data-*` 属性名リテラルの写し（[`COUNT_UP_ATTR`] 等）
//! 2. `crate::stat::value_text` へ opt-in マーカーを前置するヘルパ
//!    （[`count_up_value_text`]）
//! 3. 桁幅ジッタを抑える最小限の CSS（[`STAT_MOTION_CSS`]）
//!
//! 数値整形（桁区切り・単位・通貨）は本モジュールの責務外
//! （`.claude/rules/coding-rust.md` UI 部品の責務境界、§3.23 の一般化）。
//! 著者は整形済みの最終値をそのまま `count_up_value_text` の子テキストへ
//! 渡すだけでよく、書式の保存・数値部分の補間は `fandhe-frontend-
//! animation::count_up::NumberText` が担う。
//!
//! # `to_css()` 本体を変更しない理由
//!
//! [`crate::motion`] モジュール doc「`motion` feature 配下に置く理由」節と
//! 同じ契約: [`crate::theme::Theme::to_css`] 本体・走査ループは一切変更
//! せず、[`Theme::to_css_with_stat_motion`] を別 impl ブロックとして追加
//! し、`to_css()` の出力へ [`STAT_MOTION_CSS`] を追記するだけの opt-in
//! メソッドにする（pure append）。
//!
//! # `stat.rs` を変更しない理由
//!
//! `crate::stat::value_text` のシグネチャ（`attrs: Vec<(&str, &str)>`）へ
//! [`COUNT_UP_ATTR`] を前置するだけで opt-in を表現できるため、`stat.rs`
//! 自体（`motion` feature 非依存の既存 golden テストを持つ）へは一切
//! 手を入れない（`button_motion.rs` が `button.rs` を変更しないのと同型）。
//!
//! # styled 部品の公開 CSS 関数を持たない
//!
//! `crates/pre-styled-ui/src/stylesheet.rs` の
//! `all_styled_component_css_covers_every_component_module` の走査対象に
//! ならないよう、本モジュールは `css`/`stylesheet` という名の空引数公開
//! 関数を持たない（`button_motion.rs` と同型）。
//!
//! # `data-*` 属性の命名（他クレートとの契約）
//!
//! [`COUNT_UP_ATTR`]/[`COUNT_UP_DURATION_MS_ATTR`]/[`COUNT_UP_TRIGGER_ATTR`]/
//! [`COUNT_UP_TRIGGER_IN_VIEW`] のリテラル値は
//! `fandhe-frontend-wasm-full::count_up` の同名定数と一致することが前提
//! （本クレートは `wasm-full` に依存しないため型共有はできず、リテラルの
//! 写しとして保持する）。ドリフトは
//! `crates/pre-styled-ui/tests/stat_motion_attr_drift.rs` が wasm-full 側
//! ソースを読んで fail-closed に検知する（`button_motion_attr_drift.rs`
//! と同型）。
//!
//! # reduced-motion
//!
//! [`STAT_MOTION_CSS`] は静止したフォント調整のみで `@keyframes`/
//! `transition` を持たないため、個別の `@media (prefers-reduced-motion:
//! reduce)` ブロックは不要（`magnetic` と同じ判断）。動きの抑制は
//! `wasm-full` 側の配線が JS 段階で行う（`count_up` モジュール doc
//! 「Reduced motion」節参照）。

use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// opt-in（著者が SSR 出力に静的に付与）: カウントアップを有効化する
/// 要素マーカー。`fandhe-frontend-wasm-full::count_up::COUNT_UP_ATTR` と
/// 同一リテラル（本モジュール doc「`data-*` 属性の命名」節参照）。
pub const COUNT_UP_ATTR: &str = "data-fandhe-count-up";
/// opt-in（任意）: 補間時間（ミリ秒）を著者が上書きする属性。
pub const COUNT_UP_DURATION_MS_ATTR: &str = "data-fandhe-count-up-duration-ms";
/// opt-in（任意）: 開始トリガーを指定する属性。
pub const COUNT_UP_TRIGGER_ATTR: &str = "data-fandhe-count-up-trigger";
/// [`COUNT_UP_TRIGGER_ATTR`] の値: ビューポート進入時に開始する。
pub const COUNT_UP_TRIGGER_IN_VIEW: &str = "in-view";

/// [`crate::stat::value_text`] へ [`COUNT_UP_ATTR`] を前置して組み立てる。
/// トリガー・duration を上書きする場合は `attrs` へ
/// [`COUNT_UP_TRIGGER_ATTR`]/[`COUNT_UP_DURATION_MS_ATTR`] を追加で渡す。
///
/// `children` は [`crate::stat::value_text`] と同じ構成（数値テキストと、
/// 任意で [`crate::stat::value_unit`]/[`crate::stat::up_indicator`]/
/// [`crate::stat::down_indicator`] の兄弟）をそのまま受け取れる。配線側
/// （`fandhe-frontend-wasm-full::count_up`）は**直接の子のうち最初の数字を
/// 含むテキストノード**だけを読み書きし、兄弟の子要素は保持する（PR
/// #2580 codex-review P1 是正）。数値を `<span>` 等でさらに包むと対象外
/// （変更されない、fail-safe）になる。
#[must_use]
pub fn count_up_value_text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![(COUNT_UP_ATTR, "")];
    merged.extend(attrs);
    crate::stat::value_text(merged, children)
}

/// [`Theme::to_css_with_stat_motion`] が追記する CSS。`value-text` パーツへ
/// `font-variant-numeric: tabular-nums` を適用し、カウントアップ中の桁数
/// 変化（例 `9` → `10`）による横幅ジッタを防ぐ。
pub const STAT_MOTION_CSS: &str = concat!(
    "[data-scope=\"stat\"][data-part=\"value-text\"][data-fandhe-count-up] {\n",
    "  font-variant-numeric: tabular-nums;\n",
    "}\n",
);

/// opt-in API。[`crate::theme::Theme::to_css`] の出力へ
/// [`STAT_MOTION_CSS`] を追記して返す（pure append、[`crate::button_motion::
/// Theme::to_css_with_button_motion`] と同型）。
impl crate::theme::Theme {
    /// [`crate::theme::Theme::to_css`] の出力に Stat Motion+ variant 用
    /// CSS（[`STAT_MOTION_CSS`]）を追記して返す。
    #[must_use]
    pub fn to_css_with_stat_motion(&self) -> String {
        let mut out = self.to_css();
        out.push_str(STAT_MOTION_CSS);
        out
    }
}

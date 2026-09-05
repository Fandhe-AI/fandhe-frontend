//! Slider（単一値・連続量スライダー）headless コンポーネント（イシュー #741、
//! 親 #736、祖父 #726）。
//!
//! ark-ui の Slider
//!（`.claude/skills/ark-ui/references/components/form/slider.md`）を参考に、
//! Root / Label / Control / Track / Range / Thumb / HiddenInput / ValueText
//! の 8 anatomy パーツと、[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装する値状態機械
//! [`Slider`] を提供する。
//!
//! # `data-state` を持たない理由
//!
//! [`crate::progress::Progress`]/[`crate::switch::Switch`] の `data-state`
//! （"loading"/"checked" 等）に相当する離散的な状態区分を Slider は持たない
//! （[`crate::number_input::NumberInput`] と同じ判断: 値は連続量であり、
//! 区分らしい区分を持たない）。disabled/readonly/invalid は
//! [`SliderProps`]（描画時引数、状態機械のフィールドにしない
//! [`crate::angle_slider::AngleSliderProps`] と同型）として各パーツ関数へ
//! 渡す。
//!
//! # 参照突合（イシュー #1621、ark-ui/zag.js `slider` machine との対比）
//!
//! 一次情報は ark-ui docs（Slider ページ）と zag.js
//! `packages/machines/slider/src/*.ts`、Radix Primitives Slider（一次記録
//! `docs/design/radix-primitives-inventory.md`。姉妹部品 angle-slider の
//! 参照突合はイシュー #1601/PR #1875）。差分の是正・意図的な非追随は以下の
//! とおり（詳細は PR 本文の差分表を参照）:
//!
//! - **是正**: [`SliderProps`] を新設し `data-invalid`/`data-readonly` を
//!   root/label/control/track/range/thumb へ追加（`disabled` の描画時引数を
//!   `SliderProps` へ統合、破壊的変更）。MarkerGroup/Marker パーツ
//!   （[`marker_group`]/[`marker`]）を、姉妹部品 angle-slider（#1601）と
//!   同型のパリティで追加した（PR #1875 本文が「Slider 側にパリティ
//!   ギャップが生じた」と明記していた受け皿）。`SliderAction::IncrementLarge`/
//!   `DecrementLarge`（`"increment_large"`/`"decrement_large"`、zag の
//!   PageUp/PageDown・Shift+Arrow 相当の広域ステップ）を状態機械 dispatch
//!   契約として追加した。
//! - **意図的に追随しない**（理由付き）:
//!   - `data-focus`/`data-dragging`（zag 各パーツの実行時一時状態）:
//!     pointer ドラッグ配線がスコープ外のため出力元が存在しない
//!     （モジュール冒頭「スコープ外」節参照）。
//!   - DraggingIndicator パーツ（ark-ui 新版）: 同上の理由でスコープ外。
//!   - Root/Control/Thumb/Range の装飾用 CSS 変数
//!     （`--slider-range-start/end` 等）:
//!     `docs/policy/intentional-non-adoption.md` §3.25 規則 2（装飾・
//!     レイアウト計測は headless へ持ち込まない）に従い
//!     `fandhe-frontend-pre-styled-ui` の `--fandhe-slider-percent` が担う。
//!   - 複数 thumb（range slider、`data-index`）: #741 以来のスコープを維持。
//!   - `fandhe-frontend-wasm-full` の DOM keydown 配線（Arrow/Home/End/
//!     PageUp/PageDown）: 本イシューでは見送り、状態機械 dispatch 契約の
//!     完成のみに留めた（angle-slider #1601 と同型の判断。REQ-11 バンドル
//!     サイズ予算の逼迫が理由。PR 本文でフォローアップ Issue 化を提案）。
//!
//! # 呼び出し文脈
//!
//! SSR は [`Slider::new`] で値を正規化してから各パーツメソッド
//! （[`Slider::root`]/[`Slider::label`]/[`Slider::control`]/[`Slider::track`]/
//! [`Slider::range`]/[`Slider::thumb`]/[`Slider::hidden_input`]/
//! [`Slider::value_text`]/[`Slider::marker`]）を呼んで組み立てる。CSR/
//! hydration は [`Slider`] を経由し、dispatch（`"set"`/`"increment"`/
//! `"decrement"`/`"increment_large"`/`"decrement_large"`/`"home"`/`"end"`）
//! で状態遷移する。`fandhe-frontend-pre-styled-ui` が本モジュールを呼んで
//! スタイル済み Slider を組み立てる想定である。
//!
//! # 決定的な数値整形・step 丸め（受け入れ条件）
//!
//! - 整形は [`crate::progress`]/[`crate::number_input`] と同じ方針
//!   （`format!("{value}")`）を [`fmt_num`] として本モジュール内に個別定義
//!   する（モジュール間の相互依存を避けるための意図的な重複）。
//! - `value` は常に `min` を起点とした `step` 単位へスナップしてから
//!   `[min, max]` へ clamp する（[`snap_to_step_and_clamp`]）。スナップ後の
//!   値は [`crate::number_input`] の `round_to_step_precision`（[`step`] の
//!   小数桁数へ丸め直す）と同じ手法で浮動小数点ドリフトを除去し、
//!   `snap_to_step(snap_to_step(v)) == snap_to_step(v)`（冪等性）を保つ。
//! - 「max/min へは常に到達可能」（`value >= max`/`value <= min` はどの
//!   `step` に対しても許容する）ことを保証する。step 単位に厳密に整列しない
//!   `max`/`min` であっても、`value` が境界以上/以下のときはスナップを
//!   経由せず境界値そのものを返す（[`snap_to_step_and_clamp`]。単純な
//!   スナップ + clamp では最も近いグリッド点が境界未満/超過へ丸まり
//!   契約が破れることがあった、イシュー #741 PR #787 レビュー指摘）。
//!   `Increment`/`Decrement` も同じ関数を経由するため、off-grid な境界に
//!   着地した後の増減操作は `min` 起点の step グリッドへ再整列される。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`tabindex`）はすべて
//!   `&'static str` リテラルで固定しており、動的値が属性名スロットへ混入
//!   する経路はない（[`crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`]
//!   の既存不変条件をそのまま継承する）。
//! - 動的値（整形済み数値文字列/呼び出し側 `attrs`/children/`aria-valuetext`）
//!   は [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - 数値属性値（`aria-valuemin`/`aria-valuemax`/`aria-valuenow`/hidden-input
//!   `value`）はサーバー側で有限性検証・step 丸め・`[min, max]` へ clamp
//!   済みの `f64` の文字列表現（[`fmt_num`]）のみを出力する。任意の呼び出し
//!   側文字列をこれらの数値スロットへ直接通す経路は持たない（fail-closed
//!   正規化は [`Slider::new`] が一元的に担う）。
//! - dispatch `"set"` の payload はクライアント由来の信頼できない入力として
//!   扱い、厳密な `f64` パース + 有限性検証で fail-closed（不正値は no-op）。
//!   パース後は必ず step スナップ + `[min, max]` clamp を経由する。
//! - hydration 属性（`data-hydrate-min`/`-max`/`-step`/`-value`/
//!   `-orientation`）はクライアント側で改ざんされうる入力として扱う。
//!   [`Slider`] の [`fandhe_frontend_interactive::Hydrate`] 実装は panic
//!   せず `HydrateError` を返す（パース不能・非有限・`min >= max`・
//!   `step <= 0`・範囲外 value・未知 orientation をすべて拒否する。
//!   [`crate::progress::Progress`] と同型の fail-closed 契約）。受理した
//!   値はさらに [`snap_to_step_and_clamp`] へ通してから復元する（多層防御。値状態
//!   機械が常に step 整列済みであるという不変条件を hydration 経路でも
//!   維持する）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **range slider（複数 thumb）**: 単一値スライダーのみを引き続きスコープ
//!   とする（#741 以来のスコープを維持、モジュール冒頭「参照突合」節参照）。
//! - **pointer ドラッグ・DOM keydown 配線**: 他コンポーネント同様、
//!   クライアントランタイム（`fandhe-frontend-wasm-full`）側の後続責務と
//!   する。本モジュールは SSR 静的マークアップと dispatch 契約のみを提供
//!   する（モジュール冒頭「参照突合」節参照）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_disabled, aria_orientation};
use crate::data_attrs::{
    data_disabled, data_invalid, data_orientation, data_readonly, Orientation,
};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Slider の anatomy（`data-scope="slider"`）。
const ANATOMY: Anatomy = anatomy("slider");

/// f64 数値属性値の文字列化を一元化するヘルパ。
///
/// [`crate::progress`]/[`crate::number_input`] の同名ヘルパと同じ方針で、
/// モジュール間の相互依存を避けるため個別に定義する。
fn fmt_num(value: f64) -> String {
    format!("{value}")
}

/// `step` の小数桁数を [`fmt_num`] のシンプル表現から算出する
/// （[`crate::number_input::decimal_places`] と同型の重複実装）。
fn decimal_places(step: f64) -> i32 {
    let s = fmt_num(step);
    match s.find('.') {
        Some(idx) => (s.len() - idx - 1) as i32,
        None => 0,
    }
}

/// `value` を `step` の小数桁数へ丸める（浮動小数点ドリフト対策、
/// [`crate::number_input::round_to_step_precision`] と同型）。
fn round_to_step_precision(value: f64, step: f64) -> f64 {
    let places = decimal_places(step);
    let factor = 10f64.powi(places);
    (value * factor).round() / factor
}

/// `value` を `min` 起点で `step` 単位へスナップする。
///
/// `((value - min) / step).round()` で最も近い step 数（整数）を求め、
/// `min + steps * step` を [`round_to_step_precision`] で丸め直す。
/// この丸め直しにより `snap_to_step(snap_to_step(v), min, step) ==
/// snap_to_step(v, min, step)`（冪等性）が成り立つ（モジュール doc
/// 「決定的な数値整形・step 丸め」参照）。
///
/// `min`/`max` ちょうどの到達可能性はこの関数単体では保証しない
/// （[`snap_to_step_and_clamp`] が担う）。
fn snap_to_step(value: f64, min: f64, step: f64) -> f64 {
    let steps = ((value - min) / step).round();
    round_to_step_precision(min + steps * step, step)
}

/// [`snap_to_step`] のスナップ結果を `[min, max]` へ clamp しつつ、
/// `min`/`max` ちょうどの値は常にその境界そのものへ到達させる。
///
/// `max`（`min`）が `min` 起点の `step` グリッドに乗っていない場合、
/// 最も近いグリッド点が `max`（`min`）未満（より大きい）へ丸まることが
/// あり、[`snap_to_step`] + 単純な `clamp` だけでは「`max`/`min` は常に
/// 到達可能」というモジュール doc の契約が破れる（イシュー #741 PR #787
/// レビュー指摘、Bugbot High severity）。本関数は `value` が境界以上/以下
/// のときスナップを経由せず境界値そのものを返すことでこれを保証する。
fn snap_to_step_and_clamp(value: f64, min: f64, max: f64, step: f64) -> f64 {
    if value >= max {
        max
    } else if value <= min {
        min
    } else {
        snap_to_step(value, min, step).clamp(min, max)
    }
}

/// `min`/`max`/`step`/`value` を fail-closed に正規化する。
///
/// - `min`/`max` が非有限、または `min >= max` の場合は既定 `(0.0, 100.0)`
///   へフォールバックする（[`crate::progress::Progress`] の `normalize` と
///   同じ方針。呼び出し側の不正な入力で panic させない）。
/// - `step` が非有限、または `0.0` 以下の場合は `1.0` へフォールバックする。
/// - `value` が非有限な場合は `min` として扱う。有限な場合は
///   [`snap_to_step_and_clamp`] でスナップしてから `[min, max]` へ clamp する
///   （`max`/`min` へは常に到達可能）。
fn normalize(min: f64, max: f64, step: f64, value: f64) -> (f64, f64, f64, f64) {
    let (min, max) = if min.is_finite() && max.is_finite() && min < max {
        (min, max)
    } else {
        (0.0, 100.0)
    };
    let step = if step.is_finite() && step > 0.0 {
        step
    } else {
        1.0
    };
    let value = if value.is_finite() { value } else { min };
    let value = snap_to_step_and_clamp(value, min, max, step);
    (min, max, step, value)
}

/// [`SliderProps`] が全パーツへ一律付与する属性キー一覧。呼び出し側 `attrs`
/// にこれらと同名キーが含まれていても fail-closed で除去する対象
/// （[`crate::angle_slider::STATE_RESERVED`] と同型のパターン）。
const STATE_RESERVED: &[&str] = &["data-disabled", "data-invalid", "data-readonly"];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（[`crate::angle_slider::drop_reserved`] と同型の重複実装。
/// モジュール間の相互依存を避けるため個別に定義する）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// Slider の disabled/readonly/invalid 状態束。root/label/control/track/
/// range/thumb の全パーツへ [`data_disabled`]/[`data_invalid`]/
/// [`data_readonly`] を一律付与するために使う
/// （[`crate::angle_slider::AngleSliderProps`] と同型のパターン。破壊的
/// 変更: 従来の `disabled: bool` 引数を本 struct へ統合した）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SliderProps {
    /// 無効化状態。`true` で `data-disabled` を各パーツへ付与し、[`thumb`]
    /// を `tabindex="-1"` + `aria-disabled="true"` にする。
    pub disabled: bool,
    /// 読み取り専用状態。`true` で `data-readonly` を各パーツへ付与する。
    /// disabled と異なり [`thumb`] のフォーカス可能性は変えない
    /// （`tabindex="0"` のまま）。操作自体の抑止は
    /// `fandhe-frontend-wasm-full` 側の no-op ガードが担う想定
    /// （[`crate::angle_slider`] と同型の判断）。
    pub readonly: bool,
    /// 入力検証エラー状態。`true` で `data-invalid` を各パーツへ付与する。
    pub invalid: bool,
}

/// [`SliderProps`] から root/label/control/track/range/thumb 共通の状態
/// 属性列を組み立てる非公開ヘルパ。
fn state_attrs(props: &SliderProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs: Vec<(&'static str, &'static str)> = Vec::new();
    attrs.extend(data_disabled(props.disabled));
    attrs.extend(data_invalid(props.invalid));
    attrs.extend(data_readonly(props.readonly));
    attrs
}

/// Root パーツ（`div`）。
#[must_use]
pub fn root<'a>(
    orientation: Orientation,
    props: &SliderProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_orientation(orientation)];
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`span`）。意味論的な関連付けは呼び出し側が `id`/
/// `aria-labelledby` を `attrs` 経由で [`thumb`] へ配線する（[`thumb`]
/// は `div[role="slider"]` であり HTML `<label for>` の対象になれないため、
/// [`crate::progress::Progress::label`] と同じく `<span>` タグを使う）。
#[must_use]
pub fn label<'a>(props: &SliderProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(attrs);
    ANATOMY.part("label", "span", merged, children)
}

/// Control パーツ（`div`）。[`track`]/[`thumb`] のポインタ操作コンテナ。
#[must_use]
pub fn control<'a>(
    orientation: Orientation,
    props: &SliderProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_orientation(orientation)];
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// Track パーツ（`div`）。
#[must_use]
pub fn track<'a>(
    orientation: Orientation,
    props: &SliderProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_orientation(orientation)];
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("track", "div", merged, children)
}

/// Range パーツ（`div`）。塗りつぶし幅のスタイルは付与しない（headless
/// 中立、[`crate::progress::Progress::range`] と同型の判断）。styled 層/
/// 呼び出し側が [`Slider::percent`] を使って `attrs` 経由で `style` を渡す。
#[must_use]
pub fn range<'a>(
    orientation: Orientation,
    props: &SliderProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_orientation(orientation)];
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("range", "div", merged, children)
}

/// Thumb パーツ（`div role="slider"`）。WAI-ARIA `slider` パターンに従い
/// `aria-valuemin`/`aria-valuemax`/`aria-valuenow`/`aria-orientation` を
/// 常に出力する。`aria_valuetext` が `Some` のときのみ `aria-valuetext` を
/// 追加する。`props.disabled` が `true` のとき `tabindex="-1"` +
/// `aria-disabled` の対を出力し、それ以外（`readonly` を含む）のとき
/// `tabindex="0"`（[`crate::angle_slider::thumb`] と同型。readonly でも
/// フォーカス可能に保つのは zag の `interactive` 判定に合わせた判断）。
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn thumb<'a>(
    orientation: Orientation,
    min: &'a str,
    max: &'a str,
    now: &'a str,
    aria_valuetext: Option<&'a str>,
    props: &SliderProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("role", "slider"),
        ("aria-valuemin", min),
        ("aria-valuemax", max),
        ("aria-valuenow", now),
        aria_orientation(orientation),
    ];
    if let Some(text) = aria_valuetext {
        merged.push(("aria-valuetext", text));
    }
    if props.disabled {
        merged.push(("tabindex", "-1"));
        merged.push(aria_disabled(true));
    } else {
        merged.push(("tabindex", "0"));
    }
    merged.push(data_orientation(orientation));
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("thumb", "div", merged, children)
}

/// HiddenInput パーツ（`input type="hidden"`）。フォーム送信専用（意味論は
/// [`thumb`] の `role="slider"` が担う、[`crate::switch::hidden_input`] と
/// 同型の役割分担）。
#[must_use]
pub fn hidden_input<'a>(
    name: &'a str,
    value: &'a str,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("type", "hidden"), ("name", name), ("value", value)];
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("hidden-input", "input", merged, Vec::new())
}

/// ValueText パーツ（`span`）。表示テキストは `children`（呼び出し側が
/// 整形する。[`crate::progress::Progress::value_text`] と同型）。
#[must_use]
pub fn value_text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("value-text", "span", attrs, children)
}

/// MarkerGroup パーツ（`div`）。[`marker`] を並べるコンテナ（ark-ui の
/// MarkerGroup 相当、[`crate::angle_slider::marker_group`] と同型。装飾・
/// 位置計算は `fandhe-frontend-pre-styled-ui` 側の責務であり、本関数は
/// anatomy のみを提供する）。
#[must_use]
pub fn marker_group<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("marker-group", "div", attrs, children)
}

/// [`marker`] が全マーカーへ一律付与するキー一覧（呼び出し側 `attrs` からの
/// 偽装を fail-closed で除外する対象）。
const MARKER_RESERVED: &[&str] = &["data-value", "data-state", "data-disabled"];

/// Marker パーツ（`div`）。目盛り 1 点を表す。`min`/`max`/`value`/`current`
/// は [`normalize`] と同じ方針で fail-closed に正規化してから使う
/// （`min`/`max` が非有限または `min >= max` なら既定 `(0.0, 100.0)` へ、
/// `value`/`current` が非有限なら `min` へフォールバックする。呼び出し側の
/// 不正な入力で `f64::clamp` が panic するのを防ぐ、イシュー #1621 PR #1904
/// レビュー指摘）。`value` は正規化後の `[min, max]` へ clamp してから
/// `data-value` に出力し、正規化後の `current`（[`Slider`] の現在値）との
/// 大小関係で `data-state` を `"under-value"`/`"over-value"`/`"at-value"`
/// の 3 値リテラルへ固定する（ark-ui Marker の `data-state` と同じ語彙、
/// [`crate::angle_slider::marker`] と同型）。
#[must_use]
pub fn marker<'a>(
    value: f64,
    current: f64,
    min: f64,
    max: f64,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, MARKER_RESERVED);
    let (min, max) = if min.is_finite() && max.is_finite() && min < max {
        (min, max)
    } else {
        (0.0, 100.0)
    };
    let normalized_value = if value.is_finite() {
        value.clamp(min, max)
    } else {
        min
    };
    let normalized_current = if current.is_finite() {
        current.clamp(min, max)
    } else {
        min
    };
    let value_s = fmt_num(normalized_value);
    let state: &'static str = if normalized_value < normalized_current {
        "under-value"
    } else if normalized_value > normalized_current {
        "over-value"
    } else {
        "at-value"
    };
    let mut merged: Vec<(&str, &str)> =
        vec![("data-value", value_s.as_str()), ("data-state", state)];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("marker", "div", merged, children)
}

/// Slider のアクション（WASM 境界の文字列 dispatch と
/// [`Slider::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SliderAction {
    /// 値を設定する（[`snap_to_step_and_clamp`] でスナップ後 `[min, max]` へ
    /// clamp。`max`/`min` ちょうどの値は常に到達可能）。
    SetValue(f64),
    /// `step` 分だけ増加する（丸めた後 `[min, max]` へ clamp）。
    Increment,
    /// `step` 分だけ減少する（[`Increment`](Self::Increment) と対称）。
    Decrement,
    /// `step * `[`LARGE_STEP_MULTIPLIER`] 分だけ増加する（PageUp/
    /// Shift+ArrowUp 相当、zag の広域ステップと同型）。
    IncrementLarge,
    /// `step * `[`LARGE_STEP_MULTIPLIER`] 分だけ減少する
    /// （[`IncrementLarge`](Self::IncrementLarge) と対称）。
    DecrementLarge,
    /// 値を `min` に設定する（Home キー相当）。
    SetToMin,
    /// 値を `max` に設定する（End キー相当）。
    SetToMax,
}

/// [`SliderAction::IncrementLarge`]/[`SliderAction::DecrementLarge`] が
/// `step` に掛ける倍率（zag.js `slider` machine の PageUp/PageDown・
/// Shift+Arrow 相当の広域ステップと同じ倍率）。
const LARGE_STEP_MULTIPLIER: f64 = 10.0;

/// Slider の値状態機械（単一値、ark-ui 準拠）。
///
/// `Default` は `min=0.0, max=100.0, step=1.0, value=0.0,
/// orientation=Horizontal`（SSR の初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slider {
    min: f64,
    max: f64,
    step: f64,
    value: f64,
    orientation: Orientation,
}

impl Default for Slider {
    fn default() -> Self {
        Self::new(0.0, 100.0, 1.0, 0.0, Orientation::Horizontal)
    }
}

impl Slider {
    /// `data-hydrate-min` 属性名のフィールド部分。
    pub const FIELD_MIN: &'static str = "min";
    /// `data-hydrate-max` 属性名のフィールド部分。
    pub const FIELD_MAX: &'static str = "max";
    /// `data-hydrate-step` 属性名のフィールド部分。
    pub const FIELD_STEP: &'static str = "step";
    /// `data-hydrate-value` 属性名のフィールド部分。
    pub const FIELD_VALUE: &'static str = "value";
    /// `data-hydrate-orientation` 属性名のフィールド部分。
    pub const FIELD_ORIENTATION: &'static str = "orientation";

    /// 指定した値で [`Slider`] を生成する（[`normalize`] で fail-closed
    /// 正規化する。呼び出し側の不正な入力で panic しない）。
    #[must_use]
    pub fn new(min: f64, max: f64, step: f64, value: f64, orientation: Orientation) -> Self {
        let (min, max, step, value) = normalize(min, max, step, value);
        Self {
            min,
            max,
            step,
            value,
            orientation,
        }
    }

    /// 現在の値。
    #[must_use]
    pub fn value(&self) -> f64 {
        self.value
    }

    /// 下限値。
    #[must_use]
    pub fn min(&self) -> f64 {
        self.min
    }

    /// 上限値。
    #[must_use]
    pub fn max(&self) -> f64 {
        self.max
    }

    /// 増減の刻み幅。
    #[must_use]
    pub fn step(&self) -> f64 {
        self.step
    }

    /// 現在の向き（`data-orientation`/`aria-orientation`/hydration
    /// ラウンドトリップの対象）。
    #[must_use]
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// `[min, max]` 内の現在位置を百分率（`0.0..=100.0`）で返す
    /// （`min < max` は [`normalize`] が保証する不変条件のため常に有限）。
    #[must_use]
    pub fn percent(&self) -> f64 {
        (self.value - self.min) / (self.max - self.min) * 100.0
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        props: &SliderProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.orientation, props, attrs, children)
    }

    /// [`label`] へ委譲する利便メソッド。
    #[must_use]
    pub fn label<'a>(
        &self,
        props: &SliderProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        label(props, attrs, children)
    }

    /// [`control`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(
        &self,
        props: &SliderProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        control(self.orientation, props, attrs, children)
    }

    /// [`track`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn track<'a>(
        &self,
        props: &SliderProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        track(self.orientation, props, attrs, children)
    }

    /// [`range`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn range<'a>(
        &self,
        props: &SliderProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        range(self.orientation, props, attrs, children)
    }

    /// [`thumb`] へ現在の値・範囲を注入する利便メソッド。
    #[must_use]
    pub fn thumb<'a>(
        &self,
        aria_valuetext: Option<&'a str>,
        props: &SliderProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let min_s = fmt_num(self.min);
        let max_s = fmt_num(self.max);
        let now_s = fmt_num(self.value);
        thumb(
            self.orientation,
            min_s.as_str(),
            max_s.as_str(),
            now_s.as_str(),
            aria_valuetext,
            props,
            attrs,
            children,
        )
    }

    /// [`hidden_input`] へ現在の値を注入する利便メソッド。
    #[must_use]
    pub fn hidden_input<'a>(
        &self,
        name: &'a str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        let value_s = fmt_num(self.value);
        hidden_input(name, value_s.as_str(), disabled, attrs)
    }

    /// [`value_text`] へ委譲する利便メソッド（表示テキストは `children` で
    /// 呼び出し側が整形する）。
    #[must_use]
    pub fn value_text<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        value_text(attrs, children)
    }

    /// [`marker_group`] へ委譲する利便メソッド（状態を持たないコンテナ）。
    #[must_use]
    pub fn marker_group<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        marker_group(attrs, children)
    }

    /// [`marker`] へ現在値（`current`）・範囲（`min`/`max`）を注入する利便
    /// メソッド。`value` はマーカー自身の目盛り値。
    #[must_use]
    pub fn marker<'a>(
        &self,
        value: f64,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        marker(
            value, self.value, self.min, self.max, disabled, attrs, children,
        )
    }
}

impl Component for Slider {
    type Action = SliderAction;

    /// `SliderAction::SetValue` は非有限（`NaN`/`inf`）を fail-closed に
    /// 無視する（no-op）。[`normalize`]/[`Slider::decode_action`] が課す
    /// 「`value` は有限値」という不変条件を `update()` 単体でも維持する
    /// （[`crate::progress::Progress`]/[`crate::number_input::NumberInput`]
    /// と同型の判断）。
    fn update(&mut self, action: SliderAction) {
        match action {
            SliderAction::SetValue(v) => {
                if v.is_finite() {
                    self.value = snap_to_step_and_clamp(v, self.min, self.max, self.step);
                }
            }
            SliderAction::Increment => {
                // `next` を単純 clamp するだけでなく `snap_to_step_and_clamp`
                // へ通す。`SetToMax` 等で off-grid な値に着地した後でも、
                // 以後の Increment/Decrement が `min` 起点の step グリッドへ
                // 再整列されることを保証する（イシュー #741 PR #787 レビュー
                // 指摘、Bugbot Medium severity: 再整列しないと off-grid の
                // まま値がグリッドへ復帰できず、以後ずっとグリッド外に
                // 取り残される）。
                let next = round_to_step_precision(self.value + self.step, self.step);
                self.value = snap_to_step_and_clamp(next, self.min, self.max, self.step);
            }
            SliderAction::Decrement => {
                let next = round_to_step_precision(self.value - self.step, self.step);
                self.value = snap_to_step_and_clamp(next, self.min, self.max, self.step);
            }
            SliderAction::IncrementLarge => {
                let next = round_to_step_precision(
                    self.value + self.step * LARGE_STEP_MULTIPLIER,
                    self.step,
                );
                self.value = snap_to_step_and_clamp(next, self.min, self.max, self.step);
            }
            SliderAction::DecrementLarge => {
                let next = round_to_step_precision(
                    self.value - self.step * LARGE_STEP_MULTIPLIER,
                    self.step,
                );
                self.value = snap_to_step_and_clamp(next, self.min, self.max, self.step);
            }
            SliderAction::SetToMin => {
                self.value = self.min;
            }
            SliderAction::SetToMax => {
                self.value = self.max;
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root >
    /// control > (track > range, thumb)）。公開 UI としての利用は想定しない。
    fn view(&self) -> Node {
        let props = SliderProps::default();
        self.root(
            &props,
            Vec::new(),
            vec![self.control(
                &props,
                Vec::new(),
                vec![
                    self.track(
                        &props,
                        Vec::new(),
                        vec![self.range(&props, Vec::new(), Vec::new())],
                    ),
                    self.thumb(None, &props, Vec::new(), Vec::new()),
                ],
            )],
        )
    }

    /// `"set"`: payload を `str::parse::<f64>()` でパースし、非有限または
    /// パース不能な場合は `None`（fail-closed、dispatch は no-op）。
    /// `"increment"`/`"decrement"`/`"increment_large"`/`"decrement_large"`/
    /// `"home"`/`"end"`: payload 不使用。
    fn decode_action(name: &str, payload: &str) -> Option<SliderAction> {
        match name {
            "set" => payload
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .map(SliderAction::SetValue),
            "increment" => Some(SliderAction::Increment),
            "decrement" => Some(SliderAction::Decrement),
            "increment_large" => Some(SliderAction::IncrementLarge),
            "decrement_large" => Some(SliderAction::DecrementLarge),
            "home" => Some(SliderAction::SetToMin),
            "end" => Some(SliderAction::SetToMax),
            _ => None,
        }
    }
}

impl Hydrate for Slider {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MIN),
                fmt_num(self.min),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MAX),
                fmt_num(self.max),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STEP),
                fmt_num(self.step),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_VALUE),
                fmt_num(self.value),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ORIENTATION),
                self.orientation.as_str().to_string(),
            ),
        ]
    }

    /// クライアント改ざん入力として扱う。欠落は
    /// [`HydrateError::MissingAttr`]、パース不能・非有限・`min >= max`・
    /// `step <= 0`・範囲外 value・未知 orientation は
    /// [`HydrateError::InvalidValue`]（panic しない）。基本検証を通過した
    /// 値はさらに [`snap_to_step_and_clamp`] へ通してから復元する（モジュール
    /// doc「セキュリティ不変条件」参照。多層防御。`max`/`min` ちょうどの値は
    /// スナップで失われず境界そのものとして復元される、イシュー #741
    /// PR #787 レビュー指摘）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name))
        };

        let min_raw = find(Self::FIELD_MIN)?;
        let max_raw = find(Self::FIELD_MAX)?;
        let step_raw = find(Self::FIELD_STEP)?;
        let value_raw = find(Self::FIELD_VALUE)?;
        let orientation_raw = find(Self::FIELD_ORIENTATION)?;

        let attr_name_min = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MIN);
        let min = min_raw
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .ok_or_else(|| HydrateError::InvalidValue {
                attr: attr_name_min.clone(),
                reason: "expected a finite number".to_string(),
            })?;

        let attr_name_max = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MAX);
        let max = max_raw
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .ok_or_else(|| HydrateError::InvalidValue {
                attr: attr_name_max.clone(),
                reason: "expected a finite number".to_string(),
            })?;

        if min >= max {
            return Err(HydrateError::InvalidValue {
                attr: attr_name_min,
                reason: "expected min < max".to_string(),
            });
        }

        let attr_name_step = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STEP);
        let step = step_raw
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite() && *v > 0.0)
            .ok_or_else(|| HydrateError::InvalidValue {
                attr: attr_name_step.clone(),
                reason: "expected a finite positive number".to_string(),
            })?;

        let attr_name_value = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_VALUE);
        let value = value_raw
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .ok_or_else(|| HydrateError::InvalidValue {
                attr: attr_name_value.clone(),
                reason: "expected a finite number".to_string(),
            })?;
        if value < min || value > max {
            return Err(HydrateError::InvalidValue {
                attr: attr_name_value,
                reason: "expected value within [min, max]".to_string(),
            });
        }

        let attr_name_orientation = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ORIENTATION);
        let orientation = match orientation_raw {
            "horizontal" => Orientation::Horizontal,
            "vertical" => Orientation::Vertical,
            _ => {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name_orientation,
                    reason: "expected \"horizontal\" or \"vertical\"".to_string(),
                })
            }
        };

        let value = snap_to_step_and_clamp(value, min, max, step);

        Ok(Self {
            min,
            max,
            step,
            value,
            orientation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/data-orientation/data-disabled 出力 ---

    #[test]
    fn root_outputs_scope_part_orientation() {
        let html = render(&root(
            Orientation::Horizontal,
            &SliderProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="slider""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn root_disabled_true_adds_data_disabled() {
        let html = render(&root(
            Orientation::Horizontal,
            &SliderProps {
                disabled: true,
                ..Default::default()
            },
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn control_outputs_scope_part_orientation() {
        let html = render(&control(
            Orientation::Vertical,
            &SliderProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-part="control""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn track_outputs_scope_part_orientation() {
        let html = render(&track(
            Orientation::Horizontal,
            &SliderProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-part="track""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
    }

    #[test]
    fn range_outputs_scope_part_and_no_width_style() {
        let html = render(&range(
            Orientation::Horizontal,
            &SliderProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-part="range""#));
        assert!(!html.contains("style"));
    }

    #[test]
    fn label_outputs_scope_and_part() {
        let html = render(&label(
            &SliderProps::default(),
            vec![],
            vec![text("Volume")],
        ));
        assert!(html.contains(r#"data-scope="slider""#));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains("Volume"));
    }

    #[test]
    fn value_text_outputs_scope_and_part() {
        let html = render(&value_text(vec![], vec![text("40")]));
        assert!(html.contains(r#"data-part="value-text""#));
        assert!(html.contains("40"));
    }

    #[test]
    fn thumb_outputs_role_aria_and_tabindex() {
        let html = render(&thumb(
            Orientation::Horizontal,
            "0",
            "100",
            "40",
            None,
            &SliderProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="slider""#));
        assert!(html.contains(r#"data-part="thumb""#));
        assert!(html.contains(r#"role="slider""#));
        assert!(html.contains(r#"aria-valuemin="0""#));
        assert!(html.contains(r#"aria-valuemax="100""#));
        assert!(html.contains(r#"aria-valuenow="40""#));
        assert!(html.contains(r#"aria-orientation="horizontal""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(!html.contains("aria-disabled"));
    }

    #[test]
    fn thumb_aria_valuetext_only_when_some() {
        let without = render(&thumb(
            Orientation::Horizontal,
            "0",
            "100",
            "40",
            None,
            &SliderProps::default(),
            vec![],
            vec![],
        ));
        assert!(!without.contains("aria-valuetext"));

        let with_text = render(&thumb(
            Orientation::Horizontal,
            "0",
            "100",
            "40",
            Some("40 percent"),
            &SliderProps::default(),
            vec![],
            vec![],
        ));
        assert!(with_text.contains(r#"aria-valuetext="40 percent""#));
    }

    #[test]
    fn thumb_disabled_true_sets_tabindex_negative_one_and_aria_disabled() {
        let html = render(&thumb(
            Orientation::Horizontal,
            "0",
            "100",
            "40",
            None,
            &SliderProps {
                disabled: true,
                ..Default::default()
            },
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn hidden_input_outputs_type_name_value() {
        let html = render(&hidden_input("volume", "40", false, vec![]));
        assert!(html.contains(r#"data-scope="slider""#));
        assert!(html.contains(r#"data-part="hidden-input""#));
        assert!(html.contains(r#"type="hidden""#));
        assert!(html.contains(r#"name="volume""#));
        assert!(html.contains(r#"value="40""#));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn hidden_input_disabled_true_adds_disabled_attr() {
        let html = render(&hidden_input("volume", "40", true, vec![]));
        assert!(html.contains(r#"disabled="""#));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            Orientation::Horizontal,
            &SliderProps::default(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="slider""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn caller_supplied_state_attrs_are_dropped() {
        let html = render(&root(
            Orientation::Horizontal,
            &SliderProps::default(),
            vec![("data-disabled", "fake"), ("data-invalid", "fake")],
            vec![],
        ));
        assert!(!html.contains("fake"));
    }

    // --- MarkerGroup / Marker ---

    #[test]
    fn marker_group_outputs_scope_and_part() {
        let html = render(&marker_group(vec![], vec![]));
        assert!(html.contains(r#"data-scope="slider""#));
        assert!(html.contains(r#"data-part="marker-group""#));
    }

    #[test]
    fn marker_outputs_scope_part_value_and_state() {
        let html = render(&marker(20.0, 50.0, 0.0, 100.0, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="slider""#));
        assert!(html.contains(r#"data-part="marker""#));
        assert!(html.contains(r#"data-value="20""#));
        assert!(html.contains(r#"data-state="under-value""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn marker_data_state_over_value_when_greater_than_current() {
        let html = render(&marker(80.0, 50.0, 0.0, 100.0, false, vec![], vec![]));
        assert!(html.contains(r#"data-state="over-value""#));
    }

    #[test]
    fn marker_data_state_at_value_when_equal_to_current() {
        let html = render(&marker(50.0, 50.0, 0.0, 100.0, false, vec![], vec![]));
        assert!(html.contains(r#"data-state="at-value""#));
    }

    #[test]
    fn marker_value_is_clamped_to_range() {
        let html = render(&marker(200.0, 50.0, 0.0, 100.0, false, vec![], vec![]));
        assert!(html.contains(r#"data-value="100""#));
    }

    #[test]
    fn marker_disabled_true_adds_data_disabled() {
        let html = render(&marker(20.0, 50.0, 0.0, 100.0, true, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn marker_caller_supplied_value_and_state_are_dropped() {
        let html = render(&marker(
            20.0,
            50.0,
            0.0,
            100.0,
            false,
            vec![("data-value", "999"), ("data-state", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-value="20""#));
        assert!(html.contains(r#"data-state="under-value""#));
        assert!(!html.contains("999"));
        assert!(!html.contains("attacker"));
    }

    // --- marker の fail-closed 正規化（イシュー #1621 PR #1904 レビュー指摘） ---

    #[test]
    fn marker_min_greater_than_max_does_not_panic_and_falls_back_to_default_range() {
        // min > max は `f64::clamp` に直接渡すと panic するため、
        // `normalize` と同じ既定 (0.0, 100.0) へフォールバックすることを
        // 確認する（panic しないこと自体がこのテストの主眼）。
        let html = render(&marker(20.0, 50.0, 100.0, 0.0, false, vec![], vec![]));
        assert!(html.contains(r#"data-value="20""#));
    }

    #[test]
    fn marker_nan_min_max_does_not_panic_and_falls_back_to_default_range() {
        let html = render(&marker(
            20.0,
            50.0,
            f64::NAN,
            f64::NAN,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-value="20""#));
    }

    #[test]
    fn marker_nan_value_does_not_panic_and_falls_back_to_min() {
        let html = render(&marker(f64::NAN, 50.0, 0.0, 100.0, false, vec![], vec![]));
        // value が非有限のときは min (0.0) へフォールバックし、"NaN" を
        // 出力しない。current (50.0) より小さいため under-value。
        assert!(html.contains(r#"data-value="0""#));
        assert!(!html.contains("NaN"));
        assert!(html.contains(r#"data-state="under-value""#));
    }

    #[test]
    fn marker_nan_current_does_not_panic_and_state_stays_consistent_with_clamped_value() {
        // current が非有限のときは min へフォールバックする。value (20.0) は
        // clamp 後の current (0.0) より大きいため over-value になり、
        // clamp 前の生の current (NaN) に対する比較結果と矛盾しない。
        let html = render(&marker(20.0, f64::NAN, 0.0, 100.0, false, vec![], vec![]));
        assert!(html.contains(r#"data-value="20""#));
        assert!(!html.contains("NaN"));
        assert!(html.contains(r#"data-state="over-value""#));
    }

    // --- IncrementLarge / DecrementLarge ---

    #[test]
    fn dispatch_increment_large_steps_by_step_times_ten() {
        let mut s = Slider::new(0.0, 100.0, 1.0, 20.0, Orientation::Horizontal);
        assert!(dispatch(&mut s, "increment_large", ""));
        assert_eq!(s.value(), 30.0);
    }

    #[test]
    fn dispatch_decrement_large_steps_by_step_times_ten() {
        let mut s = Slider::new(0.0, 100.0, 1.0, 50.0, Orientation::Horizontal);
        assert!(dispatch(&mut s, "decrement_large", ""));
        assert_eq!(s.value(), 40.0);
    }

    #[test]
    fn dispatch_increment_large_clamps_at_max() {
        let mut s = Slider::new(0.0, 100.0, 1.0, 95.0, Orientation::Horizontal);
        assert!(dispatch(&mut s, "increment_large", ""));
        assert_eq!(s.value(), 100.0);
    }

    #[test]
    fn dispatch_decrement_large_clamps_at_min() {
        let mut s = Slider::new(0.0, 100.0, 1.0, 5.0, Orientation::Horizontal);
        assert!(dispatch(&mut s, "decrement_large", ""));
        assert_eq!(s.value(), 0.0);
    }

    // --- 正規化（fail-closed） ---

    #[test]
    fn new_clamps_out_of_range_value() {
        let s = Slider::new(0.0, 100.0, 1.0, 150.0, Orientation::Horizontal);
        assert_eq!(s.value(), 100.0);
        let s = Slider::new(0.0, 100.0, 1.0, -10.0, Orientation::Horizontal);
        assert_eq!(s.value(), 0.0);
    }

    #[test]
    fn new_non_finite_value_falls_back_to_min() {
        let s = Slider::new(0.0, 100.0, 1.0, f64::NAN, Orientation::Horizontal);
        assert_eq!(s.value(), 0.0);
        let s = Slider::new(10.0, 100.0, 1.0, f64::INFINITY, Orientation::Horizontal);
        assert_eq!(s.value(), 10.0);
    }

    #[test]
    fn new_non_finite_or_reversed_min_max_falls_back_to_default_range() {
        let s = Slider::new(f64::NAN, 100.0, 1.0, 5.0, Orientation::Horizontal);
        assert_eq!((s.min(), s.max()), (0.0, 100.0));
        let s = Slider::new(100.0, 0.0, 1.0, 5.0, Orientation::Horizontal);
        assert_eq!((s.min(), s.max()), (0.0, 100.0));
        let s = Slider::new(5.0, 5.0, 1.0, 5.0, Orientation::Horizontal);
        assert_eq!((s.min(), s.max()), (0.0, 100.0));
    }

    #[test]
    fn new_non_positive_or_non_finite_step_falls_back_to_one() {
        for bogus in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let s = Slider::new(0.0, 100.0, bogus, 5.0, Orientation::Horizontal);
            assert_eq!(s.step(), 1.0);
        }
    }

    #[test]
    fn new_snaps_value_to_step() {
        let s = Slider::new(0.0, 100.0, 10.0, 24.0, Orientation::Horizontal);
        assert_eq!(s.value(), 20.0);
        let s = Slider::new(0.0, 100.0, 10.0, 26.0, Orientation::Horizontal);
        assert_eq!(s.value(), 30.0);
    }

    /// イシュー #741 PR #787 レビュー指摘（Bugbot High severity）の回帰:
    /// `max`（94.0）が `min` 起点の `step`（10.0）グリッドに乗っておらず、
    /// かつ最も近いグリッド点（90.0）が `max` 未満へ丸まる場合でも、
    /// `value == max` を渡すと「max は常に到達可能」の契約どおり `max` その
    /// ものが保持される（旧実装は `snap_to_step` の丸め結果を素直に
    /// clamp するだけで `90.0` に落ちていた）。
    #[test]
    fn new_preserves_off_grid_max_when_value_equals_max() {
        let s = Slider::new(0.0, 94.0, 10.0, 94.0, Orientation::Horizontal);
        assert_eq!(s.value(), 94.0);
    }

    /// 同上の回帰を `SliderAction::SetValue` 経由（dispatch "set"）でも固定
    /// する。
    #[test]
    fn dispatch_set_reaches_off_grid_max() {
        let mut s = Slider::new(0.0, 94.0, 10.0, 0.0, Orientation::Horizontal);
        assert!(dispatch(&mut s, "set", "94"));
        assert_eq!(s.value(), 94.0);
    }

    /// 同上の回帰を hydration 経路（`from_hydration_attrs`）でも固定する
    /// （レビュー指摘が名指しした「hydration round-trip で max を失う」
    /// ケース）。
    #[test]
    fn from_hydration_attrs_preserves_off_grid_max_value() {
        let s = Slider::from_hydration_attrs(&[
            ("data-hydrate-min".to_string(), "0".to_string()),
            ("data-hydrate-max".to_string(), "94".to_string()),
            ("data-hydrate-step".to_string(), "10".to_string()),
            ("data-hydrate-value".to_string(), "94".to_string()),
            (
                "data-hydrate-orientation".to_string(),
                "horizontal".to_string(),
            ),
        ])
        .unwrap();
        assert_eq!(s.value(), 94.0);
    }

    /// イシュー #741 PR #787 レビュー指摘（Bugbot Medium severity）の回帰:
    /// `SetToMax`（"end"）で off-grid な `max` に着地した後、`Decrement`
    /// は `value - step` をそのまま採らず `min` 起点の step グリッドへ
    /// 再整列する（80.0/70.0/... の系列。旧実装は `94.0 -> 84.0 -> 74.0`
    /// と off-grid のまま推移し続けた）。
    #[test]
    fn dispatch_decrement_re_snaps_to_step_grid_after_off_grid_max() {
        let mut s = Slider::new(0.0, 94.0, 10.0, 0.0, Orientation::Horizontal);
        assert!(dispatch(&mut s, "end", ""));
        assert_eq!(s.value(), 94.0);
        assert!(dispatch(&mut s, "decrement", ""));
        assert_eq!(s.value(), 80.0);
        assert!(dispatch(&mut s, "decrement", ""));
        assert_eq!(s.value(), 70.0);
    }

    #[test]
    fn default_is_min_value_horizontal() {
        let s = Slider::default();
        assert_eq!(s.value(), 0.0);
        assert_eq!((s.min(), s.max()), (0.0, 100.0));
        assert_eq!(s.step(), 1.0);
        assert_eq!(s.orientation(), Orientation::Horizontal);
    }

    // --- percent ---

    #[test]
    fn percent_reflects_position_within_range() {
        let s = Slider::new(0.0, 100.0, 1.0, 25.0, Orientation::Horizontal);
        assert_eq!(s.percent(), 25.0);
        let s = Slider::new(0.0, 200.0, 1.0, 50.0, Orientation::Horizontal);
        assert_eq!(s.percent(), 25.0);
    }

    // --- dispatch 統合 ---

    #[test]
    fn dispatch_increment_and_decrement_step_deterministically() {
        // 受け入れ条件の回帰: min=0, max=1, step=0.1 で 10 回 increment すると
        // 浮動小数点ドリフトなしに厳密に 1.0 へ到達する。
        let mut s = Slider::new(0.0, 1.0, 0.1, 0.0, Orientation::Horizontal);
        for _ in 0..10 {
            assert!(dispatch(&mut s, "increment", ""));
        }
        assert_eq!(s.value(), 1.0);

        for _ in 0..10 {
            assert!(dispatch(&mut s, "decrement", ""));
        }
        assert_eq!(s.value(), 0.0);
    }

    #[test]
    fn dispatch_increment_clamps_at_max() {
        let mut s = Slider::new(0.0, 10.0, 1.0, 9.5, Orientation::Horizontal);
        assert!(dispatch(&mut s, "increment", ""));
        assert_eq!(s.value(), 10.0);
    }

    #[test]
    fn dispatch_decrement_clamps_at_min() {
        let mut s = Slider::new(0.0, 10.0, 1.0, 0.5, Orientation::Horizontal);
        assert!(dispatch(&mut s, "decrement", ""));
        assert_eq!(s.value(), 0.0);
    }

    #[test]
    fn dispatch_set_updates_value_and_snaps_and_clamps() {
        let mut s = Slider::new(0.0, 100.0, 10.0, 50.0, Orientation::Horizontal);
        assert!(dispatch(&mut s, "set", "24"));
        assert_eq!(s.value(), 20.0);

        assert!(dispatch(&mut s, "set", "999"));
        assert_eq!(s.value(), 100.0);
    }

    #[test]
    fn dispatch_set_rejects_invalid_payload() {
        let mut s = Slider::new(0.0, 10.0, 1.0, 5.0, Orientation::Horizontal);
        for bogus in ["abc", "NaN", "inf", "-inf", ""] {
            assert!(!dispatch(&mut s, "set", bogus));
            assert_eq!(s.value(), 5.0);
        }
    }

    #[test]
    fn dispatch_home_and_end_set_min_and_max() {
        let mut s = Slider::new(0.0, 10.0, 1.0, 5.0, Orientation::Horizontal);
        assert!(dispatch(&mut s, "home", ""));
        assert_eq!(s.value(), 0.0);
        assert!(dispatch(&mut s, "end", ""));
        assert_eq!(s.value(), 10.0);
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut s = Slider::new(0.0, 10.0, 1.0, 5.0, Orientation::Horizontal);
        assert!(!dispatch(&mut s, "no_such_action", "x"));
        assert_eq!(s.value(), 5.0);
    }

    /// イシュー #544 PR #570 レビュー指摘と同型の回帰: `decode_action` を
    /// 経由せず `SliderAction::SetValue` を直接構築して `update()` を呼んでも、
    /// 非有限値が `value` へ混入しない。
    #[test]
    fn update_rejects_non_finite_set_value_directly() {
        let mut s = Slider::new(0.0, 10.0, 1.0, 5.0, Orientation::Horizontal);
        for bogus in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            Component::update(&mut s, SliderAction::SetValue(bogus));
            assert_eq!(s.value(), 5.0);
        }
    }

    // --- SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Slider::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- hydration 経路 ---

    #[test]
    fn hydration_round_trip() {
        let s = Slider::new(0.0, 100.0, 5.0, 40.0, Orientation::Horizontal);
        let rendered = render(&render_for_hydration(&s));
        assert!(rendered.contains(r#"data-hydrate-min="0""#));
        assert!(rendered.contains(r#"data-hydrate-max="100""#));
        assert!(rendered.contains(r#"data-hydrate-step="5""#));
        assert!(rendered.contains(r#"data-hydrate-value="40""#));
        assert!(rendered.contains(r#"data-hydrate-orientation="horizontal""#));

        let restored = Slider::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }

    #[test]
    fn hydration_round_trip_vertical() {
        let s = Slider::new(0.0, 10.0, 1.0, 3.0, Orientation::Vertical);
        let restored = Slider::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
        assert_eq!(restored.orientation(), Orientation::Vertical);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Slider::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-min".to_string())
        );
    }

    #[test]
    fn from_hydration_attrs_invalid_value_does_not_panic() {
        let bogus_sets: Vec<Vec<(String, String)>> = vec![
            // min が非有限。
            vec![
                ("data-hydrate-min".to_string(), "NaN".to_string()),
                ("data-hydrate-max".to_string(), "100".to_string()),
                ("data-hydrate-step".to_string(), "1".to_string()),
                ("data-hydrate-value".to_string(), "40".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // min >= max。
            vec![
                ("data-hydrate-min".to_string(), "100".to_string()),
                ("data-hydrate-max".to_string(), "0".to_string()),
                ("data-hydrate-step".to_string(), "1".to_string()),
                ("data-hydrate-value".to_string(), "40".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // step が 0 以下。
            vec![
                ("data-hydrate-min".to_string(), "0".to_string()),
                ("data-hydrate-max".to_string(), "100".to_string()),
                ("data-hydrate-step".to_string(), "0".to_string()),
                ("data-hydrate-value".to_string(), "40".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // value が範囲外。
            vec![
                ("data-hydrate-min".to_string(), "0".to_string()),
                ("data-hydrate-max".to_string(), "100".to_string()),
                ("data-hydrate-step".to_string(), "1".to_string()),
                ("data-hydrate-value".to_string(), "150".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // orientation が未知の値。
            vec![
                ("data-hydrate-min".to_string(), "0".to_string()),
                ("data-hydrate-max".to_string(), "100".to_string()),
                ("data-hydrate-step".to_string(), "1".to_string()),
                ("data-hydrate-value".to_string(), "40".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "diagonal".to_string(),
                ),
            ],
            // value が XSS ペイロード。
            vec![
                ("data-hydrate-min".to_string(), "0".to_string()),
                ("data-hydrate-max".to_string(), "100".to_string()),
                ("data-hydrate-step".to_string(), "1".to_string()),
                (
                    "data-hydrate-value".to_string(),
                    "<script>alert(1)</script>".to_string(),
                ),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
        ];
        for attrs in bogus_sets {
            let err = Slider::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: name/attrs/children/aria-valuetext にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn hidden_input_name_payload_is_escaped_on_render() {
        let html = render(&hidden_input(ATTR_BREAK_PAYLOAD, "40", false, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            Orientation::Horizontal,
            &SliderProps::default(),
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&label(
            &SliderProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn thumb_aria_valuetext_payload_is_escaped_on_render() {
        let html = render(&thumb(
            Orientation::Horizontal,
            "0",
            "100",
            "40",
            Some(ATTR_BREAK_PAYLOAD),
            &SliderProps::default(),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn hydration_xss_payload_in_value_is_rejected_not_rendered() {
        let attrs = vec![
            ("data-hydrate-min".to_string(), "0".to_string()),
            ("data-hydrate-max".to_string(), "100".to_string()),
            ("data-hydrate-step".to_string(), "1".to_string()),
            (
                "data-hydrate-value".to_string(),
                "<script>alert(1)</script>".to_string(),
            ),
            (
                "data-hydrate-orientation".to_string(),
                "horizontal".to_string(),
            ),
        ];
        let err = Slider::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

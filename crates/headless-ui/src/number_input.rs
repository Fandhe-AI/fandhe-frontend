//! NumberInput（数値入力）headless コンポーネント（イシュー #738、親 #736。
//! 参考サイト突合はイシュー #1613）。
//!
//! ark-ui の NumberInput
//!（`.claude/skills/ark-ui/references/components/form/number-input.md`）を
//! 参考に、Root / Label / Control / Input / IncrementTrigger /
//! DecrementTrigger / ValueText の 7 anatomy パーツと、
//! [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装する値状態機械
//! [`NumberInput`] を提供する。
//!
//! # 参考サイト突合（イシュー #1613）
//!
//! ark-ui/zag.js の `number-input` machine と突合し、以下を是正した:
//!
//! - **ValueText パーツ追加**（[`value_text`]/[`NumberInput::value_text`]）。
//!   参照は 8 パーツ（root/label/control/input/increment-trigger/
//!   decrement-trigger/scrubber/value-text）だが、本モジュールは Scrubber
//!   を採らない（下記「スコープ外」節参照）ため 7 パーツとなる。
//! - **root/control/value-text に `data-readonly`**、**label に
//!   `data-required`**（zag と同じく label のみ）を追加。`input` は既存で
//!   すでに両方を持つ。
//! - **`control` に `role="group"`**、`disabled` 時 `aria-disabled="true"`、
//!   `invalid` 時 `aria-invalid="true"` を追加（`aria-disabled`/
//!   `aria-invalid` は WAI-ARIA のグローバル状態・プロパティであり `group`
//!   ロールで明示的に禁止されていない。zag.js の number-input machine も
//!   control 相当のコンテナへこれらを出力する慣行に倣う）。
//! - **`input` に `autocomplete="off"`/`autocorrect="off"`/
//!   `spellcheck="false"`/`aria-roledescription="numberfield"`** を追加。
//! - **`"home"`/`"end"` dispatch**（[`NumberInputAction::SetToMin`]/
//!   [`SetToMax`](NumberInputAction::SetToMax)）を追加（[`crate::slider::Slider`]/
//!   [`crate::angle_slider::AngleSlider`] と同型の Home/End キー相当）。
//!
//! 非追随（意図的に合わせなかった差分）:
//!
//! - **`data-focus`/`data-scrubbing`**: インタラクション状態（transient な
//!   フォーカス・ドラッグ操作）であり、[`crate::checkbox`] の
//!   hover/active/focus 非追随と同じ判断で headless 層には持ち込まない。
//! - **`pattern`/`aria-valuetext`**: zag の formatter（`Intl.NumberFormat`）に
//!   結合した値であり、数値整形は UI コンポーネント層の責務外
//!   （`.claude/rules/coding-rust.md` §3.23/§3.25）。`pattern` は負値・
//!   指数表記を `str::parse::<f64>` が受理する本実装の契約と衝突するため
//!   採らない。
//! - **修飾キー（Shift/Alt/Ctrl+Arrow）による step 倍率**: 状態機械へ
//!   倍率 API を追加しない（下記「スコープ外」節参照）。
//!
//! # `data-state` を持たない理由
//!
//! [`crate::progress::Progress`]/[`crate::switch::Switch`] の
//! `data-state`（"loading"/"checked" 等）に相当する離散的な状態区分を
//! NumberInput は持たない（値は連続量であり、区分は境界到達
//! （[`NumberInput::can_increment`]/[`NumberInput::can_decrement`]）の
//! 2 値のみ）。境界到達は各トリガーの `disabled`/`data-disabled` で表現し、
//! `data-state` 属性自体は出力しない（ark-ui 準拠、余分な状態語彙を導入しない）。
//!
//! # 呼び出し文脈
//!
//! SSR は [`NumberInput::new`] で値を正規化してから各パーツメソッド
//! （[`NumberInput::root`]/[`NumberInput::label`]/[`NumberInput::control`]/
//! [`NumberInput::input`]/[`NumberInput::increment_trigger`]/
//! [`NumberInput::decrement_trigger`]/[`NumberInput::value_text`]）を呼んで
//! 組み立てる。CSR/hydration は [`NumberInput`] を経由し、dispatch
//! （`"increment"`/`"decrement"`/`"set"`/`"clear"`/`"home"`/`"end"`）で状態
//! 遷移する。`fandhe-frontend-pre-styled-ui` が本モジュールを呼んでスタイル済み
//! NumberInput を組み立てる想定である。
//!
//! # 決定的な数値整形・パース（受け入れ条件）
//!
//! - 整形は [`crate::progress`] の `fmt_num` と同じ方針（`format!("{value}")`、
//!   Rust の `f64` `Display` はロケール非依存の shortest round-trip 表現）を
//!   [`fmt_num`] として本モジュール内に個別定義する（モジュール間の相互依存を
//!   避けるための意図的な重複、[`crate::progress`] も同型の重複を持つ）。
//! - パースは `str::parse::<f64>()`（小数点 `.` のみ・桁区切りなし・
//!   ロケール非依存）+ 有限性検証のみを用いる。
//!
//! # step 演算の決定性（浮動小数点ドリフト対策）
//!
//! `increment`/`decrement` は `value ± step` の結果を [`step`] の小数桁数
//! （[`fmt_num`] のシンプル表現から算出、[`decimal_places`]）へ丸めてから
//! `[min, max]` へ clamp する。丸めずに `f64` の加減算を繰り返すと
//! `0.1 + 0.2 != 0.3` のような蓄積誤差が生じるため（例: `min=0, max=1,
//! step=0.1` で 10 回 increment した際に `1.0` へ正確に到達しない）、本
//! モジュールは毎回 step の精度へ丸め直すことで決定的な到達を保証する。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`type`/`role`/`inputmode`/`tabindex`/
//!   `autocomplete`/`autocorrect`/`spellcheck`）は
//!   すべて `&'static str` リテラルで固定しており、動的値が属性名スロットへ
//!   混入する経路はない（[`crate::anatomy`]/[`crate::aria`]/
//!   [`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（`name`/`id`/整形済み数値文字列/呼び出し側 `attrs`/children）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - 数値属性値（`aria-valuemin`/`aria-valuemax`/`aria-valuenow`/`value`）は
//!   サーバー側で有限性検証・`[min, max]` へ clamp 済みの `f64` の文字列表現
//!   （[`fmt_num`]）のみを出力する。任意の呼び出し側文字列をこれらの数値
//!   スロットへ直接通す経路は持たない（fail-closed 正規化は
//!   [`NumberInput::new`] が一元的に担う）。
//! - dispatch `"set"` の payload はクライアント由来の信頼できない入力として
//!   扱い、厳密な `f64` パース + 有限性検証で fail-closed（不正値は no-op）。
//!   パース後は必ず `[min, max]` へ clamp する。
//! - hydration 属性（`data-hydrate-value`/`-min`/`-max`/`-step`）はクライアント
//!   側で改ざんされうる入力として扱う。[`NumberInput`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は panic せず
//!   `HydrateError` を返す（パース不能・非有限・`min > max`・`step <= 0`・
//!   範囲外 value をすべて拒否する。`min == max` はコンストラクタ
//!   ([`NumberInput::new`]) が受理する退化構成であるため hydration 側も
//!   受理する。[`crate::progress::Progress`] と同型の fail-closed 契約）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **Scrubber パーツ**: Pointer Lock API 前提（Safari 無効）のため初期実装
//!   スコープ外とする（イシュー #1613 参照突合でも非追随を継続。
//!   `docs/policy/intentional-non-adoption.md` §3.25 規則 2「参照元が
//!   primitives 層へ持ち込んでいる装飾・アニメーション・レイアウト計測の
//!   関心は headless-ui へ持ち込まない」に該当するポインタ計測の関心
//!   であるため）。
//! - **キーボード操作（ArrowUp/Down・Home/End・PageUp/Down・修飾キーによる
//!   step 倍率）の DOM 配線**: 他コンポーネント同様、クライアントランタイム
//!   （`fandhe-frontend-wasm-full`）側の後続責務とする。本モジュールは SSR
//!   静的マークアップと dispatch 契約（`"increment"`/`"decrement"`/`"set"`/
//!   `"clear"`/`"home"`/`"end"`）のみを提供する。`fandhe-frontend-wasm-full`
//!   には本コンポーネントの keydown 配線自体が存在せず（REQ-11 予算の制約）、
//!   イシュー #1613 でも新設しない。修飾キー（Shift/Alt/Ctrl+Arrow）による
//!   step 倍率も状態機械側の API を増やさない（`"increment"`/`"decrement"`
//!   は常に固定 [`Self::step`] 分のみ）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::aria_invalid;
use crate::data_attrs::{data_disabled, data_invalid, data_readonly, data_required};
use fandhe_frontend_core::{text, Node};
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// NumberInput の anatomy（`data-scope="number-input"`）。
const ANATOMY: Anatomy = anatomy("number-input");

/// f64 数値属性値の文字列化を一元化するヘルパ。
///
/// [`crate::progress`] の同名ヘルパと同じ方針（[`crate::progress`] の doc
/// 参照）で、モジュール間の相互依存を避けるため個別に定義する。
fn fmt_num(value: f64) -> String {
    format!("{value}")
}

/// `step` の小数桁数を [`fmt_num`] のシンプル表現から算出する。
///
/// 例: `step = 0.1` -> `1`、`step = 1.0` -> `0`、`step = 0.25` -> `2`。
/// 科学的記数法は通常の入力範囲（`f64` の `Display` 実装）では出現しないため
/// 考慮しない。
fn decimal_places(step: f64) -> i32 {
    let s = fmt_num(step);
    match s.find('.') {
        Some(idx) => (s.len() - idx - 1) as i32,
        None => 0,
    }
}

/// `value` を `step` の小数桁数へ丸める（浮動小数点ドリフト対策、
/// モジュール doc「step 演算の決定性」参照）。
fn round_to_step_precision(value: f64, step: f64) -> f64 {
    let places = decimal_places(step);
    let factor = 10f64.powi(places);
    (value * factor).round() / factor
}

/// `min`/`max`/`step`/`value` を fail-closed に正規化する。
///
/// - `min`/`max` が非有限な場合は [`f64::MIN`]/[`f64::MAX`] へ
///   フォールバックする（呼び出し側の不正な入力で panic させない）。
/// - `min > max` の場合は入れ替える（意図: 呼び出し側が引数を取り違えても
///   範囲としては成立させる。[`crate::progress`] の `normalize` は
///   `min >= max` を既定値へ丸めるが、NumberInput は既定値という概念が
///   希薄（`f64::MIN`/`f64::MAX` は既定として不適）なため swap を採る）。
/// - `step` が非有限、または `0.0` 以下の場合は `1.0` へフォールバックする。
/// - `value` が非有限な場合は未入力（`None`）として扱う。有限な場合は
///   `[min, max]` へ clamp する。
fn normalize(min: f64, max: f64, step: f64, value: Option<f64>) -> (f64, f64, f64, Option<f64>) {
    let min_norm = if min.is_finite() { min } else { f64::MIN };
    let max_norm = if max.is_finite() { max } else { f64::MAX };
    let (min, max) = if min_norm <= max_norm {
        (min_norm, max_norm)
    } else {
        (max_norm, min_norm)
    };
    let step = if step.is_finite() && step > 0.0 {
        step
    } else {
        1.0
    };
    let value = match value {
        Some(v) if v.is_finite() => Some(v.clamp(min, max)),
        _ => None,
    };
    (min, max, step, value)
}

/// [`root`]/[`label`]/[`control`]/[`input`]/[`value_text`] が受け取る
/// disabled/readonly/required/invalid フラグ束。4 個の独立した `bool`
/// 引数のままだと clippy `too_many_arguments`（既定閾値 7）を超えるため、
/// [`crate::checkbox::CheckboxFlags`] と同型の薄い構造体としてまとめる
/// （イシュー #1613 で `input` 専用から全パーツ共通の型へ拡張）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NumberInputFlags {
    /// ネイティブ `disabled`・`data-disabled` を付与するかどうか。
    pub disabled: bool,
    /// ネイティブ `readonly`・`data-readonly` を付与するかどうか。
    pub readonly: bool,
    /// ネイティブ `required`・`data-required` を付与するかどうか
    /// （[`label`] のみ、zag.js の number-input machine に倣う。他パーツは
    /// `required` を出力しない）。
    pub required: bool,
    /// `aria-invalid="true"`・`data-invalid` を付与するかどうか。
    pub invalid: bool,
}

/// Root パーツ（`div`）。`data-disabled`/`data-invalid`/`data-readonly` を
/// [`NumberInputFlags`] から反映する（`required` は [`label`] のみ、
/// zag.js の number-input machine に倣う）。
#[must_use]
pub fn root<'a>(
    flags: NumberInputFlags,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(flags.disabled));
    merged.extend(data_invalid(flags.invalid));
    merged.extend(data_readonly(flags.readonly));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。`input_id` を与えると `for` 属性で
/// [`input`] と関連付ける（省略時は呼び出し側が `attrs` 経由で配線する）。
/// `data-required` は 7 パーツ中 [`label`] のみが持つ（zag.js の
/// number-input machine に倣う、モジュール doc「参考サイト突合」節参照）。
#[must_use]
pub fn label<'a>(
    flags: NumberInputFlags,
    input_id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = input_id {
        merged.push(("for", id));
    }
    merged.extend(data_disabled(flags.disabled));
    merged.extend(data_invalid(flags.invalid));
    merged.extend(data_readonly(flags.readonly));
    merged.extend(data_required(flags.required));
    merged.extend(attrs);
    ANATOMY.part("label", "label", merged, children)
}

/// Control パーツ（`div`）。[`input`] とトリガーのラッパー。
///
/// `role="group"`（呼び出し側 `attrs` に同名キーがあれば省略、
/// [`has_caller_attr`] 参照）と、`disabled`/`invalid` に応じた
/// `aria-disabled`/`aria-invalid` を追加する（イシュー #1613。この 2 属性は
/// WAI-ARIA のグローバル状態・プロパティであり `group` ロールで明示的に
/// 禁止されていない。zag.js の number-input machine が control 相当の
/// コンテナへ同様に出力する慣行にも倣う）。
#[must_use]
pub fn control<'a>(
    flags: NumberInputFlags,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if !has_caller_attr(&attrs, "role") {
        merged.push(("role", "group"));
    }
    if flags.disabled {
        merged.push(("aria-disabled", "true"));
    }
    if flags.invalid {
        merged.push(aria_invalid(true));
    }
    merged.extend(data_disabled(flags.disabled));
    merged.extend(data_invalid(flags.invalid));
    merged.extend(data_readonly(flags.readonly));
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// Input パーツ（`input type="text" role="spinbutton"`）。
///
/// WAI-ARIA `spinbutton` パターンに従い `aria-valuemin`/`aria-valuemax` を
/// 常に出力し、`aria-valuenow`/`value` は現在値（`value` 引数、[`NumberInput`]
/// が [`fmt_num`] で整形済みの文字列を渡す想定）が `Some` のときのみ出力する。
/// `inputmode="decimal"` はモバイル IME に数値キーパッドを示唆するヒントで
/// あり、実際の入力検証はクライアント側（wasm-full 層）の責務。
///
/// `autocomplete="off"`（呼び出し側 `attrs` に同名キーがあれば省略）・
/// `autocorrect="off"`・`spellcheck="false"`・
/// `aria-roledescription="numberfield"` はイシュー #1613 で ark-ui/zag.js の
/// number-input machine と突合して追加した（ブラウザ・IME 由来の自動補完・
/// 自動修正・スペルチェック候補が数値入力へ誤って介入するのを防ぐ）。
#[must_use]
pub fn input<'a>(
    name: &'a str,
    id: Option<&'a str>,
    value: Option<&'a str>,
    min: &'a str,
    max: &'a str,
    flags: NumberInputFlags,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "text"),
        ("inputmode", "decimal"),
        ("role", "spinbutton"),
        ("aria-roledescription", "numberfield"),
        ("autocorrect", "off"),
        ("spellcheck", "false"),
        ("name", name),
        ("aria-valuemin", min),
        ("aria-valuemax", max),
    ];
    if !has_caller_attr(&attrs, "autocomplete") {
        merged.push(("autocomplete", "off"));
    }
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(v) = value {
        merged.push(("aria-valuenow", v));
        merged.push(("value", v));
    }
    if flags.disabled {
        merged.push(("disabled", ""));
    }
    if flags.readonly {
        merged.push(("readonly", ""));
    }
    if flags.required {
        merged.push(("required", ""));
    }
    if flags.invalid {
        merged.push(aria_invalid(true));
    }
    merged.extend(data_disabled(flags.disabled));
    merged.extend(data_invalid(flags.invalid));
    merged.extend(data_required(flags.required));
    merged.extend(data_readonly(flags.readonly));
    merged.extend(attrs);
    ANATOMY.part("input", "input", merged, Vec::new())
}

/// ValueText パーツ（`span`、イシュー #1613 で新設）。表示テキストは
/// `children`（呼び出し側が整形する。[`crate::slider::value_text`]/
/// [`crate::progress::Progress::value_text`] と同型）。`data-required` は
/// 出力しない（[`label`] のみが持つ、モジュール doc「参考サイト突合」節
/// 参照）。
#[must_use]
pub fn value_text<'a>(
    flags: NumberInputFlags,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(flags.disabled));
    merged.extend(data_invalid(flags.invalid));
    merged.extend(data_readonly(flags.readonly));
    merged.extend(attrs);
    ANATOMY.part("value-text", "span", merged, children)
}

/// `attrs` に `key`（大文字小文字を無視）が含まれるかどうかを判定する。
///
/// [`control`] の既定 `role`・[`input`] の既定 `autocomplete`・
/// [`increment_trigger`]/[`decrement_trigger`] の既定 `aria-label`
/// （いずれも呼び出し側が上書き可能、モジュール doc 参照）を、呼び出し側の
/// 指定と重複させないために使う（[`crate::progress`] の `drop_style_attr` と
/// 同型の dedup 判断、fail-closed。重複属性による無効な HTML 出力・後勝ちの
/// 非決定的な描画を防ぐ。イシュー #1613 で `has_caller_aria_label` から
/// 汎用化）。
fn has_caller_attr(attrs: &[(&str, &str)], key: &str) -> bool {
    attrs.iter().any(|(k, _)| k.eq_ignore_ascii_case(key))
}

/// IncrementTrigger パーツ（`button type="button"`）。
///
/// `tabindex="-1"` を固定付与し、キーボードフォーカス順序から除外する
/// （実際の増減操作は [`input`] へのキー入力または本ボタンへのポインタ操作、
/// ark-ui 準拠）。`disabled`（呼び出し側が [`NumberInput::can_increment`] と
/// 全体の無効化を合成した最終値）が `true` のとき、ネイティブ `disabled` と
/// `data-disabled` の双方を出力する。
#[must_use]
pub fn increment_trigger<'a>(
    input_id: Option<&'a str>,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button"), ("tabindex", "-1")];
    if let Some(id) = input_id {
        merged.push(("aria-controls", id));
    }
    if !has_caller_attr(&attrs, "aria-label") {
        merged.push(("aria-label", "increment"));
    }
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("increment-trigger", "button", merged, children)
}

/// DecrementTrigger パーツ（`button type="button"`）。[`increment_trigger`]
/// と同じ契約（既定 `aria-label` は `"decrement"`）。
#[must_use]
pub fn decrement_trigger<'a>(
    input_id: Option<&'a str>,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button"), ("tabindex", "-1")];
    if let Some(id) = input_id {
        merged.push(("aria-controls", id));
    }
    if !has_caller_attr(&attrs, "aria-label") {
        merged.push(("aria-label", "decrement"));
    }
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("decrement-trigger", "button", merged, children)
}

/// NumberInput のアクション（WASM 境界の文字列 dispatch と
/// [`NumberInput::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberInputAction {
    /// `step` 分だけ増加する（[`round_to_step_precision`] で丸めた後
    /// `[min, max]` へ clamp）。
    Increment,
    /// `step` 分だけ減少する（[`Increment`](Self::Increment) と対称）。
    Decrement,
    /// 値を設定する（`[min, max]` へ clamp して反映、step 丸めはしない）。
    Set(f64),
    /// 値を未入力状態（`None`）にする。
    Clear,
    /// 値を `min` に設定する（Home キー相当、イシュー #1613。
    /// [`crate::slider::SliderAction::SetToMin`] と同型）。
    SetToMin,
    /// 値を `max` に設定する（End キー相当、イシュー #1613。
    /// [`crate::slider::SliderAction::SetToMax`] と同型）。
    SetToMax,
}

/// NumberInput の値状態機械（ark-ui 準拠）。
///
/// `value = None` は未入力を表す。`Default` は
/// `min=f64::MIN, max=f64::MAX, step=1.0, value=None`（SSR の「未入力」
/// 初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumberInput {
    value: Option<f64>,
    min: f64,
    max: f64,
    step: f64,
}

impl Default for NumberInput {
    fn default() -> Self {
        Self::new(None, f64::MIN, f64::MAX, 1.0)
    }
}

impl NumberInput {
    /// `data-hydrate-value` 属性名のフィールド部分。
    pub const FIELD_VALUE: &'static str = "value";
    /// `data-hydrate-min` 属性名のフィールド部分。
    pub const FIELD_MIN: &'static str = "min";
    /// `data-hydrate-max` 属性名のフィールド部分。
    pub const FIELD_MAX: &'static str = "max";
    /// `data-hydrate-step` 属性名のフィールド部分。
    pub const FIELD_STEP: &'static str = "step";
    /// 未入力（`value = None`）を表す `data-hydrate-value` の予約値。
    pub const HYDRATE_VALUE_NONE: &str = "none";

    /// 指定した値で [`NumberInput`] を生成する（[`normalize`] で fail-closed
    /// 正規化する。呼び出し側の不正な入力で panic しない）。
    #[must_use]
    pub fn new(value: Option<f64>, min: f64, max: f64, step: f64) -> Self {
        let (min, max, step, value) = normalize(min, max, step, value);
        Self {
            value,
            min,
            max,
            step,
        }
    }

    /// 現在の値（`None` は未入力）。
    #[must_use]
    pub fn value(&self) -> Option<f64> {
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

    /// 現在値の整形済み文字列（未入力のときは空文字列）。
    ///
    /// イシュー #1613 で [`Self::value_text`]（[`value_text`] パーツへ
    /// 委譲する `Node` 返却の利便メソッド）を新設したため、名前衝突回避で
    /// `value_text()` から改称した（唯一の呼び出し元
    /// `crates/headless-ui/tests/number_input.rs` も追随済み）。
    #[must_use]
    pub fn formatted_value(&self) -> String {
        self.value.map(fmt_num).unwrap_or_default()
    }

    /// これ以上 increment 可能かどうか（未入力のときは常に `true`）。
    #[must_use]
    pub fn can_increment(&self) -> bool {
        match self.value {
            Some(v) => v < self.max,
            None => true,
        }
    }

    /// これ以上 decrement 可能かどうか（未入力のときは常に `true`）。
    #[must_use]
    pub fn can_decrement(&self) -> bool {
        match self.value {
            Some(v) => v > self.min,
            None => true,
        }
    }

    /// 現在値が `[min, max]` 内にあるかどうか（未入力のときは `true`）。
    /// [`NumberInput`] の不変条件（常に clamp 済み）が保たれていれば常に
    /// `true` を返すが、外部で構築ロジックが変わった際の回帰検知用に公開する。
    #[must_use]
    pub fn is_in_range(&self) -> bool {
        match self.value {
            Some(v) => v >= self.min && v <= self.max,
            None => true,
        }
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        flags: NumberInputFlags,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(flags, attrs, children)
    }

    /// [`label`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn label<'a>(
        &self,
        flags: NumberInputFlags,
        input_id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        label(flags, input_id, attrs, children)
    }

    /// [`control`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(
        &self,
        flags: NumberInputFlags,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        control(flags, attrs, children)
    }

    /// [`value_text`] へ現在値の整形済み文字列（[`Self::formatted_value`]）を
    /// テキストノードとして注入する利便メソッド（イシュー #1613）。
    #[must_use]
    pub fn value_text<'a>(&self, flags: NumberInputFlags, attrs: Vec<(&'a str, &'a str)>) -> Node {
        value_text(flags, attrs, vec![text(self.formatted_value())])
    }

    /// [`input`] へ現在の値・範囲を注入する利便メソッド。
    #[must_use]
    pub fn input<'a>(
        &self,
        name: &'a str,
        id: Option<&'a str>,
        flags: NumberInputFlags,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        let value_s = self.value.map(fmt_num);
        let min_s = fmt_num(self.min);
        let max_s = fmt_num(self.max);
        input(
            name,
            id,
            value_s.as_deref(),
            min_s.as_str(),
            max_s.as_str(),
            flags,
            attrs,
        )
    }

    /// [`increment_trigger`] へ現在の境界到達状態を注入する利便メソッド。
    /// `disabled` は呼び出し側の全体無効化フラグと [`Self::can_increment`]
    /// を OR で合成する。
    #[must_use]
    pub fn increment_trigger<'a>(
        &self,
        input_id: Option<&'a str>,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        increment_trigger(input_id, disabled || !self.can_increment(), attrs, children)
    }

    /// [`decrement_trigger`] へ現在の境界到達状態を注入する利便メソッド。
    #[must_use]
    pub fn decrement_trigger<'a>(
        &self,
        input_id: Option<&'a str>,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        decrement_trigger(input_id, disabled || !self.can_decrement(), attrs, children)
    }
}

impl Component for NumberInput {
    type Action = NumberInputAction;

    /// `NumberInputAction::Set` は非有限（`NaN`/`inf`）を fail-closed に
    /// 無視する（no-op）。[`normalize`]/[`NumberInput::decode_action`] が課す
    /// 「`value` は有限値または `None`」という不変条件を `update()` 単体でも
    /// 維持するため（[`crate::progress::Progress`] と同型の判断）。
    fn update(&mut self, action: NumberInputAction) {
        match action {
            NumberInputAction::Increment => {
                let base = self.value.unwrap_or(self.min);
                let next = round_to_step_precision(base + self.step, self.step);
                self.value = Some(next.clamp(self.min, self.max));
            }
            NumberInputAction::Decrement => {
                let base = self.value.unwrap_or(self.max);
                let next = round_to_step_precision(base - self.step, self.step);
                self.value = Some(next.clamp(self.min, self.max));
            }
            NumberInputAction::Set(v) => {
                if v.is_finite() {
                    self.value = Some(v.clamp(self.min, self.max));
                }
            }
            NumberInputAction::Clear => {
                self.value = None;
            }
            NumberInputAction::SetToMin => {
                self.value = Some(self.min);
            }
            NumberInputAction::SetToMax => {
                self.value = Some(self.max);
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root >
    /// control。`name` を要する [`input`] は含めない、[`crate::switch::Switch::view`]
    /// と同型の判断）。公開 UI としての利用は想定しない。
    fn view(&self) -> Node {
        self.root(
            NumberInputFlags::default(),
            Vec::new(),
            vec![self.control(NumberInputFlags::default(), Vec::new(), Vec::new())],
        )
    }

    /// `"increment"`/`"decrement"`/`"home"`/`"end"`: payload 不使用。`"set"`:
    /// payload を `str::parse::<f64>()` でパースし、非有限またはパース不能な
    /// 場合は `None`（fail-closed、dispatch は no-op）。`"clear"`: payload
    /// 不使用。`"home"`/`"end"`（イシュー #1613）は
    /// [`crate::slider::Slider::decode_action`]/
    /// [`crate::angle_slider::AngleSlider::decode_action`] と同型のキー名。
    fn decode_action(name: &str, payload: &str) -> Option<NumberInputAction> {
        match name {
            "increment" => Some(NumberInputAction::Increment),
            "decrement" => Some(NumberInputAction::Decrement),
            "set" => payload
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .map(NumberInputAction::Set),
            "clear" => Some(NumberInputAction::Clear),
            "home" => Some(NumberInputAction::SetToMin),
            "end" => Some(NumberInputAction::SetToMax),
            _ => None,
        }
    }
}

impl Hydrate for NumberInput {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let value_s = match self.value {
            Some(v) => fmt_num(v),
            None => Self::HYDRATE_VALUE_NONE.to_string(),
        };
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_VALUE),
                value_s,
            ),
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
        ]
    }

    /// クライアント改ざん入力として扱う。欠落は
    /// [`HydrateError::MissingAttr`]、パース不能・非有限・`min > max`・
    /// `step <= 0`・範囲外 value は [`HydrateError::InvalidValue`]（panic
    /// しない、[`crate::progress::Progress`] と同型の fail-closed 契約。
    /// `min == max` は [`NumberInput::new`] が受理する退化構成のため拒否
    /// しない）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name))
        };

        let value_raw = find(Self::FIELD_VALUE)?;
        let min_raw = find(Self::FIELD_MIN)?;
        let max_raw = find(Self::FIELD_MAX)?;
        let step_raw = find(Self::FIELD_STEP)?;

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

        // `min == max`（退化した単一値レンジ）は [`NumberInput::new`]/
        // [`normalize`] が受理する構成であるため、hydration 側も同じ境界で
        // 受理する（`min > max` のみを拒否）。ここを `min >= max` にすると
        // コンストラクタでは成立する構成が hydration では常に失敗する
        // 不変条件の食い違いが生じる。
        if min > max {
            return Err(HydrateError::InvalidValue {
                attr: attr_name_min,
                reason: "expected min <= max".to_string(),
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
        let value = if value_raw == Self::HYDRATE_VALUE_NONE {
            None
        } else {
            let v = value_raw
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| HydrateError::InvalidValue {
                    attr: attr_name_value.clone(),
                    reason: "expected a finite number or \"none\"".to_string(),
                })?;
            if v < min || v > max {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name_value,
                    reason: "expected value within [min, max]".to_string(),
                });
            }
            Some(v)
        };

        Ok(Self {
            value,
            min,
            max,
            step,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part 出力 ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(NumberInputFlags::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="number-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-invalid"));
        assert!(!html.contains("data-readonly"));
    }

    #[test]
    fn root_disabled_invalid_readonly_true_adds_data_attrs() {
        let html = render(&root(
            NumberInputFlags {
                disabled: true,
                invalid: true,
                readonly: true,
                required: true,
            },
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-readonly="""#));
        // `data-required` は label のみが出す（イシュー #1613、モジュール doc
        // 「参考サイト突合」節参照）。
        assert!(!html.contains("data-required"));
    }

    #[test]
    fn label_outputs_for_when_input_id_given() {
        let html = render(&label(
            NumberInputFlags::default(),
            Some("qty"),
            vec![],
            vec![text("Quantity")],
        ));
        assert!(html.contains(r#"data-scope="number-input""#));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains(r#"for="qty""#));
        assert!(html.contains("Quantity"));
    }

    #[test]
    fn label_omits_for_when_input_id_none() {
        let html = render(&label(NumberInputFlags::default(), None, vec![], vec![]));
        assert!(!html.contains("for="));
    }

    #[test]
    fn label_required_true_adds_data_required() {
        let html = render(&label(
            NumberInputFlags {
                required: true,
                ..NumberInputFlags::default()
            },
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-required="""#));
    }

    #[test]
    fn control_outputs_scope_and_part() {
        let html = render(&control(NumberInputFlags::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="number-input""#));
        assert!(html.contains(r#"data-part="control""#));
        assert!(html.contains(r#"role="group""#));
        assert!(!html.contains("aria-disabled"));
        assert!(!html.contains("aria-invalid"));
    }

    #[test]
    fn control_disabled_invalid_true_adds_aria_and_data_attrs() {
        let html = render(&control(
            NumberInputFlags {
                disabled: true,
                invalid: true,
                readonly: true,
                required: false,
            },
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"aria-invalid="true""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn control_caller_role_overrides_default() {
        let html = render(&control(
            NumberInputFlags::default(),
            vec![("role", "presentation")],
            vec![],
        ));
        assert_eq!(html.matches("role=").count(), 1);
        assert!(html.contains(r#"role="presentation""#));
        assert!(!html.contains(r#"role="group""#));
    }

    #[test]
    fn input_outputs_type_role_inputmode_and_valuemin_max() {
        let html = render(&input(
            "qty",
            None,
            None,
            "0",
            "100",
            NumberInputFlags::default(),
            vec![],
        ));
        assert!(html.contains(r#"data-scope="number-input""#));
        assert!(html.contains(r#"data-part="input""#));
        assert!(html.contains(r#"type="text""#));
        assert!(html.contains(r#"inputmode="decimal""#));
        assert!(html.contains(r#"role="spinbutton""#));
        assert!(html.contains(r#"name="qty""#));
        assert!(html.contains(r#"aria-valuemin="0""#));
        assert!(html.contains(r#"aria-valuemax="100""#));
        assert!(!html.contains("aria-valuenow"));
        assert!(!html.contains(r#"value="#));
        // イシュー #1613: ブラウザ/IME の自動補完・自動修正・スペル
        // チェックの誤介入を防ぐ既定属性。
        assert!(html.contains(r#"autocomplete="off""#));
        assert!(html.contains(r#"autocorrect="off""#));
        assert!(html.contains(r#"spellcheck="false""#));
        assert!(html.contains(r#"aria-roledescription="numberfield""#));
    }

    #[test]
    fn input_caller_autocomplete_overrides_default() {
        let html = render(&input(
            "qty",
            None,
            None,
            "0",
            "100",
            NumberInputFlags::default(),
            vec![("autocomplete", "on")],
        ));
        assert_eq!(html.matches("autocomplete=").count(), 1);
        assert!(html.contains(r#"autocomplete="on""#));
        assert!(!html.contains(r#"autocomplete="off""#));
    }

    #[test]
    fn input_outputs_valuenow_and_value_when_some() {
        let html = render(&input(
            "qty",
            Some("qty-input"),
            Some("40"),
            "0",
            "100",
            NumberInputFlags::default(),
            vec![],
        ));
        assert!(html.contains(r#"id="qty-input""#));
        assert!(html.contains(r#"aria-valuenow="40""#));
        assert!(html.contains(r#"value="40""#));
    }

    #[test]
    fn input_disabled_readonly_required_are_present_attrs() {
        let html = render(&input(
            "qty",
            None,
            None,
            "0",
            "100",
            NumberInputFlags {
                disabled: true,
                readonly: true,
                required: true,
                invalid: false,
            },
            vec![],
        ));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"readonly="""#));
        assert!(html.contains(r#"required="""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-required="""#));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn input_invalid_true_adds_aria_invalid_and_data_invalid() {
        let html = render(&input(
            "qty",
            None,
            None,
            "0",
            "100",
            NumberInputFlags {
                invalid: true,
                ..NumberInputFlags::default()
            },
            vec![],
        ));
        assert!(html.contains(r#"aria-invalid="true""#));
        assert!(html.contains(r#"data-invalid="""#));
    }

    #[test]
    fn increment_trigger_default_aria_label_and_tabindex() {
        let html = render(&increment_trigger(Some("qty-input"), false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="number-input""#));
        assert!(html.contains(r#"data-part="increment-trigger""#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"aria-controls="qty-input""#));
        assert!(html.contains(r#"aria-label="increment""#));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn increment_trigger_disabled_true_adds_disabled_and_data_disabled() {
        let html = render(&increment_trigger(None, true, vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn increment_trigger_caller_aria_label_overrides_default() {
        let html = render(&increment_trigger(
            None,
            false,
            vec![("aria-label", "add one")],
            vec![],
        ));
        assert_eq!(html.matches("aria-label").count(), 1);
        assert!(html.contains(r#"aria-label="add one""#));
        assert!(!html.contains(r#"aria-label="increment""#));
    }

    #[test]
    fn decrement_trigger_default_aria_label() {
        let html = render(&decrement_trigger(None, false, vec![], vec![]));
        assert!(html.contains(r#"data-part="decrement-trigger""#));
        assert!(html.contains(r#"aria-label="decrement""#));
    }

    // --- ValueText パーツ（イシュー #1613） ---

    #[test]
    fn value_text_outputs_scope_and_part() {
        let html = render(&value_text(
            NumberInputFlags::default(),
            vec![],
            vec![text("40")],
        ));
        assert!(html.contains(r#"data-scope="number-input""#));
        assert!(html.contains(r#"data-part="value-text""#));
        assert!(html.contains("40"));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn value_text_flags_add_data_attrs_but_not_required() {
        let html = render(&value_text(
            NumberInputFlags {
                disabled: true,
                invalid: true,
                readonly: true,
                required: true,
            },
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-readonly="""#));
        assert!(!html.contains("data-required"));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            NumberInputFlags::default(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="number-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- 正規化（fail-closed） ---

    #[test]
    fn new_clamps_out_of_range_value() {
        let n = NumberInput::new(Some(150.0), 0.0, 100.0, 1.0);
        assert_eq!(n.value(), Some(100.0));
        let n = NumberInput::new(Some(-10.0), 0.0, 100.0, 1.0);
        assert_eq!(n.value(), Some(0.0));
    }

    #[test]
    fn new_non_finite_value_becomes_none() {
        let n = NumberInput::new(Some(f64::NAN), 0.0, 100.0, 1.0);
        assert_eq!(n.value(), None);
        let n = NumberInput::new(Some(f64::INFINITY), 0.0, 100.0, 1.0);
        assert_eq!(n.value(), None);
    }

    #[test]
    fn new_swaps_min_max_when_reversed() {
        let n = NumberInput::new(Some(5.0), 100.0, 0.0, 1.0);
        assert_eq!((n.min(), n.max()), (0.0, 100.0));
    }

    #[test]
    fn new_non_finite_min_max_falls_back_to_f64_extremes() {
        let n = NumberInput::new(Some(5.0), f64::NAN, 100.0, 1.0);
        assert_eq!(n.min(), f64::MIN);
        let n = NumberInput::new(Some(5.0), 0.0, f64::INFINITY, 1.0);
        assert_eq!(n.max(), f64::MAX);
    }

    #[test]
    fn new_non_positive_or_non_finite_step_falls_back_to_one() {
        for bogus in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let n = NumberInput::new(Some(5.0), 0.0, 100.0, bogus);
            assert_eq!(n.step(), 1.0);
        }
    }

    #[test]
    fn default_is_no_value() {
        let n = NumberInput::default();
        assert_eq!(n.value(), None);
        assert_eq!(n.step(), 1.0);
    }

    // --- can_increment / can_decrement / is_in_range ---

    #[test]
    fn can_increment_and_decrement_reflect_bounds() {
        let n = NumberInput::new(Some(0.0), 0.0, 10.0, 1.0);
        assert!(n.can_increment());
        assert!(!n.can_decrement());

        let n = NumberInput::new(Some(10.0), 0.0, 10.0, 1.0);
        assert!(!n.can_increment());
        assert!(n.can_decrement());
    }

    #[test]
    fn can_increment_and_decrement_are_true_when_value_is_none() {
        let n = NumberInput::new(None, 0.0, 10.0, 1.0);
        assert!(n.can_increment());
        assert!(n.can_decrement());
    }

    #[test]
    fn is_in_range_true_for_clamped_or_none_value() {
        let n = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
        assert!(n.is_in_range());
        let n = NumberInput::new(None, 0.0, 10.0, 1.0);
        assert!(n.is_in_range());
    }

    // --- dispatch 統合 ---

    #[test]
    fn dispatch_increment_and_decrement_step_deterministically() {
        // 受け入れ条件の回帰: min=0, max=1, step=0.1 で 10 回 increment すると
        // 浮動小数点ドリフトなしに厳密に 1.0 へ到達する。
        let mut n = NumberInput::new(Some(0.0), 0.0, 1.0, 0.1);
        for _ in 0..10 {
            assert!(dispatch(&mut n, "increment", ""));
        }
        assert_eq!(n.value(), Some(1.0));
        assert!(!n.can_increment());

        for _ in 0..10 {
            assert!(dispatch(&mut n, "decrement", ""));
        }
        assert_eq!(n.value(), Some(0.0));
    }

    #[test]
    fn dispatch_increment_from_none_starts_at_min_plus_step() {
        let mut n = NumberInput::new(None, 0.0, 10.0, 1.0);
        assert!(dispatch(&mut n, "increment", ""));
        assert_eq!(n.value(), Some(1.0));
    }

    #[test]
    fn dispatch_decrement_from_none_starts_at_max_minus_step() {
        let mut n = NumberInput::new(None, 0.0, 10.0, 1.0);
        assert!(dispatch(&mut n, "decrement", ""));
        assert_eq!(n.value(), Some(9.0));
    }

    #[test]
    fn dispatch_increment_clamps_at_max() {
        let mut n = NumberInput::new(Some(9.5), 0.0, 10.0, 1.0);
        assert!(dispatch(&mut n, "increment", ""));
        assert_eq!(n.value(), Some(10.0));
    }

    #[test]
    fn dispatch_decrement_clamps_at_min() {
        let mut n = NumberInput::new(Some(0.5), 0.0, 10.0, 1.0);
        assert!(dispatch(&mut n, "decrement", ""));
        assert_eq!(n.value(), Some(0.0));
    }

    #[test]
    fn dispatch_set_updates_value_and_clamps() {
        let mut n = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
        assert!(dispatch(&mut n, "set", "7.5"));
        assert_eq!(n.value(), Some(7.5));

        assert!(dispatch(&mut n, "set", "999"));
        assert_eq!(n.value(), Some(10.0));
    }

    #[test]
    fn dispatch_set_rejects_invalid_payload() {
        let mut n = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
        for bogus in ["abc", "NaN", "inf", "-inf", ""] {
            assert!(!dispatch(&mut n, "set", bogus));
            assert_eq!(n.value(), Some(5.0));
        }
    }

    #[test]
    fn dispatch_clear_sets_value_to_none() {
        let mut n = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
        assert!(dispatch(&mut n, "clear", ""));
        assert_eq!(n.value(), None);
    }

    // --- "home"/"end" dispatch（イシュー #1613） ---

    #[test]
    fn dispatch_home_sets_value_to_min() {
        let mut n = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
        assert!(dispatch(&mut n, "home", ""));
        assert_eq!(n.value(), Some(0.0));
    }

    #[test]
    fn dispatch_end_sets_value_to_max() {
        let mut n = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
        assert!(dispatch(&mut n, "end", ""));
        assert_eq!(n.value(), Some(10.0));
    }

    #[test]
    fn dispatch_home_and_end_from_none_value() {
        let mut n = NumberInput::new(None, 0.0, 10.0, 1.0);
        assert!(dispatch(&mut n, "home", ""));
        assert_eq!(n.value(), Some(0.0));

        let mut n = NumberInput::new(None, 0.0, 10.0, 1.0);
        assert!(dispatch(&mut n, "end", ""));
        assert_eq!(n.value(), Some(10.0));
    }

    #[test]
    fn dispatch_home_and_end_when_min_equals_max() {
        // 退化構成（`min == max`、コンストラクタが受理する）でも "home"/"end"
        // は一貫して成立する。
        let mut n = NumberInput::new(Some(5.0), 5.0, 5.0, 1.0);
        assert!(dispatch(&mut n, "home", ""));
        assert_eq!(n.value(), Some(5.0));
        assert!(dispatch(&mut n, "end", ""));
        assert_eq!(n.value(), Some(5.0));
    }

    /// イシュー #544 PR #570 レビュー指摘と同型の回帰: `decode_action` を
    /// 経由せず `NumberInputAction::Set` を直接構築して `update()` を呼んでも、
    /// 非有限値が `value` へ混入しない（「有限値または `None`」不変条件を
    /// `update()` 単体でも維持する）。
    #[test]
    fn update_rejects_non_finite_set_value_directly() {
        let mut n = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
        for bogus in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            Component::update(&mut n, NumberInputAction::Set(bogus));
            assert_eq!(n.value(), Some(5.0));
        }
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut n = NumberInput::new(Some(5.0), 0.0, 10.0, 1.0);
        assert!(!dispatch(&mut n, "no_such_action", "x"));
        assert_eq!(n.value(), Some(5.0));
    }

    // --- SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&NumberInput::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- hydration 経路 ---

    #[test]
    fn hydration_round_trip_with_value() {
        let n = NumberInput::new(Some(40.0), 0.0, 100.0, 1.0);
        let rendered = render(&render_for_hydration(&n));
        assert!(rendered.contains(r#"data-hydrate-value="40""#));
        assert!(rendered.contains(r#"data-hydrate-min="0""#));
        assert!(rendered.contains(r#"data-hydrate-max="100""#));
        assert!(rendered.contains(r#"data-hydrate-step="1""#));

        let restored = NumberInput::from_hydration_attrs(&n.hydration_attrs()).unwrap();
        assert_eq!(restored, n);
    }

    #[test]
    fn hydration_round_trip_with_none_value() {
        let n = NumberInput::new(None, 0.0, 100.0, 1.0);
        let rendered = render(&render_for_hydration(&n));
        assert!(rendered.contains(r#"data-hydrate-value="none""#));

        let restored = NumberInput::from_hydration_attrs(&n.hydration_attrs()).unwrap();
        assert_eq!(restored, n);
    }

    /// `min == max`（退化した単一値レンジ）はコンストラクタが受理する構成
    /// である。hydration 側だけがこの構成を拒否すると、コンストラクタと
    /// hydration の間で不変条件が食い違う（`from_hydration_attrs` は
    /// `min > max` のみを拒否し `min == max` は受理する契約）。
    #[test]
    fn hydration_round_trip_when_min_equals_max() {
        let n = NumberInput::new(Some(5.0), 5.0, 5.0, 1.0);
        assert_eq!((n.min(), n.max()), (5.0, 5.0));

        let restored = NumberInput::from_hydration_attrs(&n.hydration_attrs()).unwrap();
        assert_eq!(restored, n);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = NumberInput::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-value".to_string())
        );
    }

    #[test]
    fn from_hydration_attrs_invalid_value_does_not_panic() {
        let bogus_sets: Vec<Vec<(String, String)>> = vec![
            // min が非有限。
            vec![
                ("data-hydrate-value".to_string(), "40".to_string()),
                ("data-hydrate-min".to_string(), "NaN".to_string()),
                ("data-hydrate-max".to_string(), "100".to_string()),
                ("data-hydrate-step".to_string(), "1".to_string()),
            ],
            // min > max。
            vec![
                ("data-hydrate-value".to_string(), "40".to_string()),
                ("data-hydrate-min".to_string(), "100".to_string()),
                ("data-hydrate-max".to_string(), "0".to_string()),
                ("data-hydrate-step".to_string(), "1".to_string()),
            ],
            // step が 0 以下。
            vec![
                ("data-hydrate-value".to_string(), "40".to_string()),
                ("data-hydrate-min".to_string(), "0".to_string()),
                ("data-hydrate-max".to_string(), "100".to_string()),
                ("data-hydrate-step".to_string(), "0".to_string()),
            ],
            // value が範囲外。
            vec![
                ("data-hydrate-value".to_string(), "150".to_string()),
                ("data-hydrate-min".to_string(), "0".to_string()),
                ("data-hydrate-max".to_string(), "100".to_string()),
                ("data-hydrate-step".to_string(), "1".to_string()),
            ],
            // value が XSS ペイロード。
            vec![
                (
                    "data-hydrate-value".to_string(),
                    "<script>alert(1)</script>".to_string(),
                ),
                ("data-hydrate-min".to_string(), "0".to_string()),
                ("data-hydrate-max".to_string(), "100".to_string()),
                ("data-hydrate-step".to_string(), "1".to_string()),
            ],
        ];
        for attrs in bogus_sets {
            let err = NumberInput::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: name/id/attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn input_name_payload_is_escaped_on_render() {
        let html = render(&input(
            ATTR_BREAK_PAYLOAD,
            None,
            None,
            "0",
            "100",
            NumberInputFlags::default(),
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            NumberInputFlags::default(),
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&label(
            NumberInputFlags::default(),
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    /// [`value_text`] の children も既定エスケープを経由する（イシュー #1613
    /// で新設したパーツの XSS 回帰）。
    #[test]
    fn value_text_children_payload_is_escaped_on_render() {
        let html = render(&value_text(
            NumberInputFlags::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn hydration_xss_payload_in_value_is_rejected_not_rendered() {
        let attrs = vec![
            (
                "data-hydrate-value".to_string(),
                "<script>alert(1)</script>".to_string(),
            ),
            ("data-hydrate-min".to_string(), "0".to_string()),
            ("data-hydrate-max".to_string(), "100".to_string()),
            ("data-hydrate-step".to_string(), "1".to_string()),
        ];
        let err = NumberInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

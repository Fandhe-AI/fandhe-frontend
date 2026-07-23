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
//! 区分らしい区分を持たない）。`disabled` は描画時引数（状態機械のフィールド
//! にしない、[`crate::switch::Switch`]/[`crate::number_input::NumberInput`]
//! と同型）として各パーツ関数へ渡す。
//!
//! # 呼び出し文脈
//!
//! SSR は [`Slider::new`] で値を正規化してから各パーツメソッド
//! （[`Slider::root`]/[`Slider::label`]/[`Slider::control`]/[`Slider::track`]/
//! [`Slider::range`]/[`Slider::thumb`]/[`Slider::hidden_input`]/
//! [`Slider::value_text`]）を呼んで組み立てる。CSR/hydration は [`Slider`]
//! を経由し、dispatch（`"set"`/`"increment"`/`"decrement"`/`"home"`/`"end"`）
//! で状態遷移する。`fandhe-frontend-pre-styled-ui` が本モジュールを呼んで
//! スタイル済み Slider を組み立てる想定である。
//!
//! # 決定的な数値整形・step 丸め（受け入れ条件）
//!
//! - 整形は [`crate::progress`]/[`crate::number_input`] と同じ方針
//!   （`format!("{value}")`）を [`fmt_num`] として本モジュール内に個別定義
//!   する（モジュール間の相互依存を避けるための意図的な重複）。
//! - `value` は常に `min` を起点とした `step` 単位へスナップしてから
//!   `[min, max]` へ clamp する（[`snap_to_step`]）。スナップ後の値は
//!   [`crate::number_input`] の `round_to_step_precision`（[`step`] の小数
//!   桁数へ丸め直す）と同じ手法で浮動小数点ドリフトを除去し、
//!   `snap_to_step(snap_to_step(v)) == snap_to_step(v)`（冪等性）を保つ。
//! - clamp は「max へは常に到達可能」（`value == max` はどの `step` に対して
//!   も許容する）ことを保証する。step 単位に厳密に整列しない `max` であって
//!   も、clamp が最終的に `max` そのものへ丸め込む。
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
//!   値はさらに [`snap_to_step`] へ通してから復元する（多層防御。値状態
//!   機械が常に step 整列済みであるという不変条件を hydration 経路でも
//!   維持する）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **range slider（複数 thumb）**・**Marker/MarkerGroup**: 単一値スライダー
//!   のみを初期実装スコープとする。
//! - **pointer ドラッグ・キーボード操作（Arrow/Home/End/PageUp/Down）の DOM
//!   配線**: 他コンポーネント同様、クライアントランタイム
//!   （`fandhe-frontend-wasm-full`）側の後続責務とする。本モジュールは SSR
//!   静的マークアップと dispatch 契約のみを提供する。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_disabled, aria_orientation};
use crate::data_attrs::{data_disabled, data_orientation, Orientation};
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
fn snap_to_step(value: f64, min: f64, step: f64) -> f64 {
    let steps = ((value - min) / step).round();
    round_to_step_precision(min + steps * step, step)
}

/// `min`/`max`/`step`/`value` を fail-closed に正規化する。
///
/// - `min`/`max` が非有限、または `min >= max` の場合は既定 `(0.0, 100.0)`
///   へフォールバックする（[`crate::progress::Progress`] の `normalize` と
///   同じ方針。呼び出し側の不正な入力で panic させない）。
/// - `step` が非有限、または `0.0` 以下の場合は `1.0` へフォールバックする。
/// - `value` が非有限な場合は `min` として扱う。有限な場合は
///   [`snap_to_step`] でスナップしてから `[min, max]` へ clamp する
///   （clamp は常に `max` へ到達可能）。
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
    let value = snap_to_step(value, min, step).clamp(min, max);
    (min, max, step, value)
}

/// Root パーツ（`div`）。
#[must_use]
pub fn root<'a>(
    orientation: Orientation,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_orientation(orientation)];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`span`）。装飾用パーツ（意味論的な関連付けは呼び出し側が
/// `id`/`aria-labelledby` を `attrs` 経由で [`thumb`] へ配線する。[`thumb`]
/// は `div[role="slider"]` であり HTML `<label for>` の対象になれないため、
/// [`crate::progress::Progress::label`] と同じく `<span>` タグを使う）。
#[must_use]
pub fn label<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("label", "span", attrs, children)
}

/// Control パーツ（`div`）。[`track`]/[`thumb`] のポインタ操作コンテナ。
#[must_use]
pub fn control<'a>(
    orientation: Orientation,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_orientation(orientation)];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// Track パーツ（`div`）。
#[must_use]
pub fn track<'a>(
    orientation: Orientation,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_orientation(orientation)];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("track", "div", merged, children)
}

/// Range パーツ（`div`）。塗りつぶし幅のスタイルは付与しない（headless
/// 中立、[`crate::progress::Progress::range`] と同型の判断）。styled 層/
/// 呼び出し側が [`Slider::percent`] を使って `attrs` 経由で `style` を渡す。
#[must_use]
pub fn range<'a>(
    orientation: Orientation,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_orientation(orientation)];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("range", "div", merged, children)
}

/// Thumb パーツ（`div role="slider"`）。WAI-ARIA `slider` パターンに従い
/// `aria-valuemin`/`aria-valuemax`/`aria-valuenow`/`aria-orientation` を
/// 常に出力する。`aria_valuetext` が `Some` のときのみ `aria-valuetext` を
/// 追加する。`disabled` が `true` のとき `tabindex="-1"` + `aria-disabled`
/// の対を出力し、`false` のとき `tabindex="0"`（キーボードフォーカス対象、
/// 実際の操作配線はスコープ外・モジュール doc 参照）。
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn thumb<'a>(
    orientation: Orientation,
    min: &'a str,
    max: &'a str,
    now: &'a str,
    aria_valuetext: Option<&'a str>,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
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
    if disabled {
        merged.push(("tabindex", "-1"));
        merged.push(aria_disabled(true));
    } else {
        merged.push(("tabindex", "0"));
    }
    merged.push(data_orientation(orientation));
    merged.extend(data_disabled(disabled));
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

/// Slider のアクション（WASM 境界の文字列 dispatch と
/// [`Slider::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SliderAction {
    /// 値を設定する（[`snap_to_step`] でスナップ後 `[min, max]` へ clamp）。
    SetValue(f64),
    /// `step` 分だけ増加する（丸めた後 `[min, max]` へ clamp）。
    Increment,
    /// `step` 分だけ減少する（[`Increment`](Self::Increment) と対称）。
    Decrement,
    /// 値を `min` に設定する（Home キー相当）。
    SetToMin,
    /// 値を `max` に設定する（End キー相当）。
    SetToMax,
}

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
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.orientation, disabled, attrs, children)
    }

    /// [`label`] へ委譲する利便メソッド（状態を持たない装飾用パーツ）。
    #[must_use]
    pub fn label<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        label(attrs, children)
    }

    /// [`control`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        control(self.orientation, disabled, attrs, children)
    }

    /// [`track`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn track<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        track(self.orientation, disabled, attrs, children)
    }

    /// [`range`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn range<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        range(self.orientation, disabled, attrs, children)
    }

    /// [`thumb`] へ現在の値・範囲を注入する利便メソッド。
    #[must_use]
    pub fn thumb<'a>(
        &self,
        aria_valuetext: Option<&'a str>,
        disabled: bool,
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
            disabled,
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
                    self.value = snap_to_step(v, self.min, self.step).clamp(self.min, self.max);
                }
            }
            SliderAction::Increment => {
                let next = round_to_step_precision(self.value + self.step, self.step);
                self.value = next.clamp(self.min, self.max);
            }
            SliderAction::Decrement => {
                let next = round_to_step_precision(self.value - self.step, self.step);
                self.value = next.clamp(self.min, self.max);
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
        self.root(
            false,
            Vec::new(),
            vec![self.control(
                false,
                Vec::new(),
                vec![
                    self.track(
                        false,
                        Vec::new(),
                        vec![self.range(false, Vec::new(), Vec::new())],
                    ),
                    self.thumb(None, false, Vec::new(), Vec::new()),
                ],
            )],
        )
    }

    /// `"set"`: payload を `str::parse::<f64>()` でパースし、非有限または
    /// パース不能な場合は `None`（fail-closed、dispatch は no-op）。
    /// `"increment"`/`"decrement"`/`"home"`/`"end"`: payload 不使用。
    fn decode_action(name: &str, payload: &str) -> Option<SliderAction> {
        match name {
            "set" => payload
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .map(SliderAction::SetValue),
            "increment" => Some(SliderAction::Increment),
            "decrement" => Some(SliderAction::Decrement),
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
    /// 値はさらに [`snap_to_step`] へ通してから復元する（モジュール doc
    /// 「セキュリティ不変条件」参照。多層防御）。
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

        let value = snap_to_step(value, min, step).clamp(min, max);

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
        let html = render(&root(Orientation::Horizontal, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="slider""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn root_disabled_true_adds_data_disabled() {
        let html = render(&root(Orientation::Horizontal, true, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn control_outputs_scope_part_orientation() {
        let html = render(&control(Orientation::Vertical, false, vec![], vec![]));
        assert!(html.contains(r#"data-part="control""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn track_outputs_scope_part_orientation() {
        let html = render(&track(Orientation::Horizontal, false, vec![], vec![]));
        assert!(html.contains(r#"data-part="track""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
    }

    #[test]
    fn range_outputs_scope_part_and_no_width_style() {
        let html = render(&range(Orientation::Horizontal, false, vec![], vec![]));
        assert!(html.contains(r#"data-part="range""#));
        assert!(!html.contains("style"));
    }

    #[test]
    fn label_outputs_scope_and_part() {
        let html = render(&label(vec![], vec![text("Volume")]));
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
            false,
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
            false,
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
            false,
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
            true,
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
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="slider""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
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
            false,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&label(vec![], vec![text("<script>alert(1)</script>")]));
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
            false,
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

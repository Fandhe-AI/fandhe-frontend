//! AngleSlider（円環状の単一角度値スライダー）headless コンポーネント
//! （イシュー #842、親トラッキング #520）。
//!
//! ark-ui の AngleSlider
//! （`.claude/skills/ark-ui/references/components/form/angle-slider.md`）を
//! 参考に、Root / Label / Control / Thumb / ValueText / HiddenInput の
//! 6 anatomy パーツと、[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装する整数角度状態機械
//! [`AngleSlider`] を提供する。
//!
//! # 「非採用の再導入」であること（`docs/policy/intentional-non-adoption.md` §3.22/§4）
//!
//! AngleSlider は同書 §3.22（イシュー #735）で「ポインタ座標 → 角度変換の
//! 暗黙性・非決定性・機械検証困難」を理由に意図的非採用と確定していた。
//! 本モジュールはその懸念に対し、責務を 3 層へ分離することで応える:
//!
//! 1. **本モジュール（headless 層）**: 角度値（`0..=359` の整数）の
//!    純粋状態機械のみ。ポインタ座標を一切扱わず、外部依存は
//!    `fandhe-frontend-core`/`fandhe-frontend-interactive` のみ（本クレート
//!    共通の不変条件、`crate` root doc 参照）。
//! 2. **wasm-full 層**（`crates/wasm-full/src/angle_slider.rs`）: `atan2` を
//!    含む座標 → 角度変換を `web-sys` 非依存の純粋関数として隔離し、既知
//!    座標 → 既知角度の網羅表による単体テストで決定性を固定する。
//! 3. **表示**: CSS `transform: rotate(var(--fandhe-angle))` のみ（canvas
//!    不使用、`fandhe-frontend-pre-styled-ui` 側の責務）。
//!
//! 評価軸（明示性・決定性・機械検証可能性・コンテキスト消費）の充足根拠は
//! `docs/policy/intentional-non-adoption.md` §3.22 の再導入記録、および
//! イシュー #842 の PR 本文を参照。
//!
//! # 呼び出し文脈
//!
//! SSR は [`AngleSlider::new`] で値を正規化してから各パーツメソッド
//! （[`AngleSlider::root`]/[`AngleSlider::label`]/[`AngleSlider::control`]/
//! [`AngleSlider::thumb`]/[`AngleSlider::hidden_input`]/
//! [`AngleSlider::value_text`]）を呼んで組み立てる。CSR/hydration は
//! [`AngleSlider`] を経由し、dispatch（`"set"`/`"increment"`/`"decrement"`）
//! で状態遷移する。`fandhe-frontend-pre-styled-ui` が本モジュールを呼んで
//! スタイル済み AngleSlider を組み立てる想定である。
//!
//! # 決定的な角度正規化・step 丸め（受け入れ条件）
//!
//! - 角度値は常に `0..=359` の整数（`u16`）。`360` 度は `0` 度と同一視し、
//!   受理時に `value % 360` で正規化する（[`normalize_angle`]）。
//! - `step` は `1..=359` へ clamp する（`0` は無限ループを招くため許容
//!   しない、[`crate::slider::Slider`] の `step <= 0` フォールバックと
//!   同型の判断）。
//! - `"set"`（[`AngleSliderAction::Set`]）は受理値をそのまま採用せず、
//!   `0` 起点の `step` グリッドへ最近傍スナップしてから正規化する
//!   （[`snap_angle_to_step`]、[`crate::slider::Slider::update`] の
//!   `SetValue` が常にスナップするのと同型の契約、ark-ui の
//!   `snapAngleToStep` 相当）。これにより後続の `increment`/`decrement`
//!   が意図した step の倍数へ確実に戻る。
//! - `increment`/`decrement` は `(value + step) mod 360`/
//!   `(value + 360 - step) mod 360` で計算し、`359` 度からの increment・
//!   `0` 度からの decrement のいずれも符号付き剰余を経由せず単純な非負整数
//!   演算のみでラップアラウンドする（浮動小数点を一切用いない、モジュール
//!   doc「決定性」節参照）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`tabindex`）はすべて
//!   `&'static str` リテラルで固定しており、動的値が属性名スロットへ混入
//!   する経路はない（[`crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`]
//!   の既存不変条件をそのまま継承する）。
//! - 動的値（整形済み角度文字列/呼び出し側 `attrs`/children）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - 数値属性値（`aria-valuemin`/`aria-valuemax`/`aria-valuenow`/
//!   `aria-valuetext`/hidden-input `value`）はサーバー側で正規化済みの
//!   `u16` の文字列表現（[`fmt_angle`]）のみを出力する。任意の呼び出し側
//!   文字列をこれらの数値スロットへ直接通す経路は持たない（fail-closed
//!   正規化は [`AngleSlider::new`] が一元的に担う）。
//! - dispatch `"set"` の payload はクライアント由来の信頼できない入力として
//!   扱い、厳密な `u16` パース + `0..=360` 範囲検証で fail-closed（不正値・
//!   非整数・負数・361 以上・空文字は no-op）。受理後は `360` を `0` へ
//!   丸めたうえで [`normalize_angle`] を経由する。
//! - hydration 属性（`data-hydrate-value`/`-step`）はクライアント側で
//!   改ざんされうる入力として扱う。[`AngleSlider`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は panic せず
//!   [`HydrateError`] を返す（パース不能・`0..=359` 範囲外の value・
//!   `1..=359` 範囲外の step をすべて拒否する、[`crate::progress::Progress`]
//!   と同型の fail-closed 契約）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **MarkerGroup/Marker パーツ**: [`crate::slider::Slider`]（#741）の
//!   先例に従い初期実装スコープ外。
//! - **ポインタ座標 → 角度変換・DOM 配線**: `fandhe-frontend-wasm-full`
//!   側の責務（モジュール冒頭「非採用の再導入」節参照）。本モジュールは
//!   SSR 静的マークアップと dispatch 契約のみを提供する。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::aria_disabled;
use crate::data_attrs::data_disabled;
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// AngleSlider の anatomy（`data-scope="angle-slider"`）。
const ANATOMY: Anatomy = anatomy("angle-slider");

/// 角度の上限（度）。`0..=359` が有効値域で、`360` は `0` と同一視する
/// 入力側の受理範囲を表す（`decode_action`/hydration のパース境界）。
const ANGLE_MODULUS: u32 = 360;

/// `u16` 角度値の文字列化を一元化するヘルパ（[`crate::slider::fmt_num`]
/// と同型の重複実装。モジュール間の相互依存を避けるため個別に定義する）。
fn fmt_angle(value: u16) -> String {
    format!("{value}")
}

/// 任意の `u32` 角度を `0..=359` へ正規化する（`value % 360`）。
///
/// 符号付き剰余ではなく非負整数の剰余のみを用いるため、`increment`/
/// `decrement` の呼び出し元はあらかじめ非負の被除数を用意する
/// （[`AngleSlider::update`] 参照）。
fn normalize_angle(value: u32) -> u16 {
    (value % ANGLE_MODULUS) as u16
}

/// `step` を `1..=359` へ fail-closed に clamp する。`0` はラップアラウンド
/// が停止しない無限ループを招くため許容しない
/// （[`crate::slider::Slider`] の `step <= 0` フォールバックと同型の判断）。
fn normalize_step(step: u16) -> u16 {
    step.clamp(1, (ANGLE_MODULUS - 1) as u16)
}

/// `value`/`step` を fail-closed に正規化する（[`AngleSlider::new`] が呼ぶ）。
fn normalize(value: u16, step: u16) -> (u16, u16) {
    (normalize_angle(u32::from(value)), normalize_step(step))
}

/// `value` を `0` 起点の `step` グリッドへ最近傍スナップしてから
/// `0..=359` へ正規化する（[`crate::slider::snap_to_step`] の角度版）。
///
/// `step` が `360` の約数でない場合、`0` 起点で単純に
/// `(value / step).round() * step` を計算する線形グリッド方式では
/// 最終区間（最後の step 倍数から `360`（≡ `0`）までの区間）が他の
/// 区間より短くなる（例: `step = 25` なら `350..360` の区間は長さ
/// `10` しかない）。この短い最終区間を無視して線形グリッドをそのまま
/// 延長すると、本来 `0` に最も近い角度（例: `358`）が誤って手前の通常グリッド点
/// （`350`）へスナップされ、コンポーネントの `360 ≡ 0` 契約（circular
/// wrap-around 最近接スナップ）が破れる
/// （Bugbot 指摘「Snap breaks 360-equals-0」対応）。
///
/// そのため以下の 2 候補のみを比較し、近い方へスナップする:
/// - `candidate_low`: 正規化済み角度以下の最大の step 倍数
///   （`floor(normalized / step) * step`）
/// - `candidate_high`: その次の step 倍数。ただしそれが `360` 以上に
///   達する場合は、線形グリッドをそのまま延長せず円周の閉点である
///   `360`（≡ `0`）で打ち切る
///
/// 同着（両候補との差が等しい）の場合は `candidate_high` を採用する
/// （旧実装の `f64::round()` が半数を正の無限大方向へ丸める挙動と
/// 同じ「切り上げ」寄りの決定的な tie-break）。
/// 呼び出し元（[`AngleSlider::update`] の `AngleSliderAction::Set`）は
/// この関数を経由することで、`increment`/`decrement` が加算し続ける
/// step グリッドへ常に整列した状態を保つ（ark-ui の
/// `snapAngleToStep` と同型の契約、[`crate::slider::Slider::update`] の
/// `SetValue` と同様に "set" も必ずスナップする）。
fn snap_angle_to_step(value: u16, step: u16) -> u16 {
    let step_f = f64::from(step);
    let modulus_f = f64::from(ANGLE_MODULUS);
    // `value` は `decode_action` で `0..=360` に制限済みだが、`360` を
    // `0` と同一視する契約（モジュール doc 参照）に従い、グリッド計算の
    // 起点として先に正規化する（正規化前に線形丸めを行うと `360` が
    // 契約を無視して手前の step 倍数へ丸められてしまう）。
    let normalized = f64::from(normalize_angle(u32::from(value)));
    let candidate_low = (normalized / step_f).floor() * step_f;
    let candidate_high_raw = candidate_low + step_f;
    let candidate_high = if candidate_high_raw >= modulus_f {
        modulus_f
    } else {
        candidate_high_raw
    };
    let diff_low = normalized - candidate_low;
    let diff_high = candidate_high - normalized;
    let snapped = if diff_high <= diff_low {
        candidate_high
    } else {
        candidate_low
    };
    normalize_angle(snapped as u32)
}

/// Root パーツ（`div`）。
#[must_use]
pub fn root<'a>(disabled: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`span`）。意味論的な関連付けは呼び出し側が `id`/
/// `aria-labelledby` を `attrs` 経由で [`thumb`] へ配線する
/// （[`crate::slider::label`] と同型の判断）。
#[must_use]
pub fn label<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("label", "span", attrs, children)
}

/// Control パーツ（`div`）。[`thumb`] のポインタ操作コンテナ
/// （実配線は `fandhe-frontend-wasm-full` 側の責務）。
#[must_use]
pub fn control<'a>(disabled: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// Thumb パーツ（`div role="slider"`）。WAI-ARIA `slider` パターンに従い
/// `aria-valuemin="0"`/`aria-valuemax="360"`/`aria-valuenow`/
/// `aria-valuetext="{value}deg"` を常に出力する。`disabled` が `true` の
/// とき `tabindex="-1"` + `aria-disabled` の対を出力し、`false` のとき
/// `tabindex="0"`（[`crate::slider::thumb`] と同型）。
#[must_use]
pub fn thumb<'a>(
    now: &'a str,
    value_text: &'a str,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("role", "slider"),
        ("aria-valuemin", "0"),
        ("aria-valuemax", "360"),
        ("aria-valuenow", now),
        ("aria-valuetext", value_text),
    ];
    if disabled {
        merged.push(("tabindex", "-1"));
        merged.push(aria_disabled(true));
    } else {
        merged.push(("tabindex", "0"));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("thumb", "div", merged, children)
}

/// HiddenInput パーツ（`input type="hidden"`）。フォーム送信専用（意味論は
/// [`thumb`] の `role="slider"` が担う、[`crate::slider::hidden_input`] と
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
/// 整形する。[`crate::slider::value_text`] と同型）。
#[must_use]
pub fn value_text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("value-text", "span", attrs, children)
}

/// AngleSlider のアクション（WASM 境界の文字列 dispatch と
/// [`AngleSlider::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AngleSliderAction {
    /// 値を設定する（`0..=360` を受理し、`360` は `0` へ正規化する）。
    Set(u16),
    /// `step` 分だけ時計回りに増加する（`0`/`359` 境界でラップアラウンド）。
    Increment,
    /// `step` 分だけ反時計回りに減少する
    /// （[`Increment`](Self::Increment) と対称）。
    Decrement,
}

/// AngleSlider の値状態機械（単一角度、ark-ui 準拠）。
///
/// `Default` は `value=0, step=1`（SSR の初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AngleSlider {
    value: u16,
    step: u16,
}

impl Default for AngleSlider {
    fn default() -> Self {
        Self::new(0, 1)
    }
}

impl AngleSlider {
    /// `data-hydrate-value` 属性名のフィールド部分。
    pub const FIELD_VALUE: &'static str = "value";
    /// `data-hydrate-step` 属性名のフィールド部分。
    pub const FIELD_STEP: &'static str = "step";

    /// 指定した値で [`AngleSlider`] を生成する（[`normalize`] で fail-closed
    /// 正規化する。呼び出し側の不正な入力で panic しない）。
    #[must_use]
    pub fn new(value: u16, step: u16) -> Self {
        let (value, step) = normalize(value, step);
        Self { value, step }
    }

    /// 現在の角度（度、`0..=359`）。
    #[must_use]
    pub fn angle_deg(&self) -> u16 {
        self.value
    }

    /// 増減の刻み幅（度、`1..=359`）。
    #[must_use]
    pub fn step(&self) -> u16 {
        self.step
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(disabled, attrs, children)
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
        control(disabled, attrs, children)
    }

    /// [`thumb`] へ現在の角度を注入する利便メソッド。`aria-valuetext` は
    /// `"{value}deg"` 形式で固定する（呼び出し側による上書きは提供しない。
    /// 角度の単位表記は本コンポーネントの全インスタンスで一貫させる判断）。
    #[must_use]
    pub fn thumb<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let now_s = fmt_angle(self.value);
        let text_s = format!("{}deg", self.value);
        thumb(now_s.as_str(), text_s.as_str(), disabled, attrs, children)
    }

    /// [`hidden_input`] へ現在の値を注入する利便メソッド。
    #[must_use]
    pub fn hidden_input<'a>(
        &self,
        name: &'a str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        let value_s = fmt_angle(self.value);
        hidden_input(name, value_s.as_str(), disabled, attrs)
    }

    /// [`value_text`] へ委譲する利便メソッド（表示テキストは `children` で
    /// 呼び出し側が整形する）。
    #[must_use]
    pub fn value_text<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        value_text(attrs, children)
    }
}

impl Component for AngleSlider {
    type Action = AngleSliderAction;

    fn update(&mut self, action: AngleSliderAction) {
        match action {
            AngleSliderAction::Set(v) => {
                self.value = snap_angle_to_step(v, self.step);
            }
            AngleSliderAction::Increment => {
                self.value = normalize_angle(u32::from(self.value) + u32::from(self.step));
            }
            AngleSliderAction::Decrement => {
                // 非負整数のみで表現するため、`ANGLE_MODULUS` を一度加算
                // してから引く（`self.step <= 359 < ANGLE_MODULUS` が
                // `normalize` により保証されるため、加算後の被減数は常に
                // 非負かつ `u32` で安全に表現できる）。
                self.value =
                    normalize_angle(u32::from(self.value) + ANGLE_MODULUS - u32::from(self.step));
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root >
    /// control > thumb）。公開 UI としての利用は想定しない。
    fn view(&self) -> Node {
        self.root(
            false,
            Vec::new(),
            vec![self.control(
                false,
                Vec::new(),
                vec![self.thumb(false, Vec::new(), Vec::new())],
            )],
        )
    }

    /// `"set"`: payload を `str::parse::<u16>()` でパースし、`0..=360` の
    /// 範囲外・パース不能な場合は `None`（fail-closed、dispatch は no-op）。
    /// `360` は受理するが [`Self::update`] 内で `0` へ正規化する。
    /// `"increment"`/`"decrement"`: payload 不使用。
    fn decode_action(name: &str, payload: &str) -> Option<AngleSliderAction> {
        match name {
            "set" => payload
                .parse::<u16>()
                .ok()
                .filter(|v| *v <= ANGLE_MODULUS as u16)
                .map(AngleSliderAction::Set),
            "increment" => Some(AngleSliderAction::Increment),
            "decrement" => Some(AngleSliderAction::Decrement),
            _ => None,
        }
    }
}

impl Hydrate for AngleSlider {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_VALUE),
                fmt_angle(self.value),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STEP),
                fmt_angle(self.step),
            ),
        ]
    }

    /// クライアント改ざん入力として扱う。欠落は
    /// [`HydrateError::MissingAttr`]、パース不能・範囲外（value は
    /// `0..=359`、step は `1..=359`）は [`HydrateError::InvalidValue`]
    /// （panic しない）。受理した値はさらに [`normalize`] へ通してから
    /// 復元する（多層防御、[`crate::slider::Slider`] と同型の契約）。
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
        let step_raw = find(Self::FIELD_STEP)?;

        let attr_name_value = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_VALUE);
        let value = value_raw
            .parse::<u16>()
            .ok()
            .filter(|v| *v < ANGLE_MODULUS as u16)
            .ok_or_else(|| HydrateError::InvalidValue {
                attr: attr_name_value,
                reason: "expected an integer in 0..=359".to_string(),
            })?;

        let attr_name_step = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STEP);
        let step = step_raw
            .parse::<u16>()
            .ok()
            .filter(|v| *v >= 1 && *v < ANGLE_MODULUS as u16)
            .ok_or_else(|| HydrateError::InvalidValue {
                attr: attr_name_step,
                reason: "expected an integer in 1..=359".to_string(),
            })?;

        let (value, step) = normalize(value, step);
        Ok(Self { value, step })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/data-disabled 出力 ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="angle-slider""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn root_disabled_true_adds_data_disabled() {
        let html = render(&root(true, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn control_outputs_scope_and_part() {
        let html = render(&control(false, vec![], vec![]));
        assert!(html.contains(r#"data-part="control""#));
    }

    #[test]
    fn label_outputs_scope_and_part() {
        let html = render(&label(vec![], vec![text("Angle")]));
        assert!(html.contains(r#"data-scope="angle-slider""#));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains("Angle"));
    }

    #[test]
    fn value_text_outputs_scope_and_part() {
        let html = render(&value_text(vec![], vec![text("40deg")]));
        assert!(html.contains(r#"data-part="value-text""#));
        assert!(html.contains("40deg"));
    }

    #[test]
    fn thumb_outputs_role_aria_and_tabindex() {
        let html = render(&thumb("40", "40deg", false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="angle-slider""#));
        assert!(html.contains(r#"data-part="thumb""#));
        assert!(html.contains(r#"role="slider""#));
        assert!(html.contains(r#"aria-valuemin="0""#));
        assert!(html.contains(r#"aria-valuemax="360""#));
        assert!(html.contains(r#"aria-valuenow="40""#));
        assert!(html.contains(r#"aria-valuetext="40deg""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(!html.contains("aria-disabled"));
    }

    #[test]
    fn thumb_disabled_true_sets_tabindex_negative_one_and_aria_disabled() {
        let html = render(&thumb("40", "40deg", true, vec![], vec![]));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn hidden_input_outputs_type_name_value() {
        let html = render(&hidden_input("angle", "40", false, vec![]));
        assert!(html.contains(r#"data-scope="angle-slider""#));
        assert!(html.contains(r#"data-part="hidden-input""#));
        assert!(html.contains(r#"type="hidden""#));
        assert!(html.contains(r#"name="angle""#));
        assert!(html.contains(r#"value="40""#));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn hidden_input_disabled_true_adds_disabled_attr() {
        let html = render(&hidden_input("angle", "40", true, vec![]));
        assert!(html.contains(r#"disabled="""#));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="angle-slider""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- 正規化（fail-closed） ---

    #[test]
    fn new_normalizes_360_to_zero() {
        let s = AngleSlider::new(360, 1);
        assert_eq!(s.angle_deg(), 0);
    }

    #[test]
    fn new_normalizes_over_360_via_modulus() {
        let s = AngleSlider::new(400, 1);
        assert_eq!(s.angle_deg(), 40);
    }

    #[test]
    fn new_clamps_step_to_valid_range() {
        let s = AngleSlider::new(10, 0);
        assert_eq!(s.step(), 1);
        let s = AngleSlider::new(10, 1000);
        assert_eq!(s.step(), 359);
    }

    #[test]
    fn default_is_zero_degrees_step_one() {
        let s = AngleSlider::default();
        assert_eq!(s.angle_deg(), 0);
        assert_eq!(s.step(), 1);
    }

    // --- dispatch: ラップアラウンドの決定性 ---

    #[test]
    fn dispatch_increment_wraps_past_359() {
        let mut s = AngleSlider::new(355, 10);
        assert!(dispatch(&mut s, "increment", ""));
        assert_eq!(s.angle_deg(), 5);
    }

    #[test]
    fn dispatch_decrement_wraps_before_zero() {
        let mut s = AngleSlider::new(5, 10);
        assert!(dispatch(&mut s, "decrement", ""));
        assert_eq!(s.angle_deg(), 355);
    }

    #[test]
    fn dispatch_increment_and_decrement_are_symmetric_round_trip() {
        let mut s = AngleSlider::new(0, 30);
        for _ in 0..12 {
            assert!(dispatch(&mut s, "increment", ""));
        }
        assert_eq!(s.angle_deg(), 0);
        for _ in 0..12 {
            assert!(dispatch(&mut s, "decrement", ""));
        }
        assert_eq!(s.angle_deg(), 0);
    }

    #[test]
    fn dispatch_set_normalizes_360_to_zero() {
        let mut s = AngleSlider::new(10, 1);
        assert!(dispatch(&mut s, "set", "360"));
        assert_eq!(s.angle_deg(), 0);
    }

    #[test]
    fn dispatch_set_updates_value() {
        let mut s = AngleSlider::new(0, 1);
        assert!(dispatch(&mut s, "set", "271"));
        assert_eq!(s.angle_deg(), 271);
    }

    #[test]
    fn dispatch_set_snaps_to_step_grid() {
        // step=10 のグリッド外（37 は 40 に最近傍）へ "set" しても、
        // 後続の increment/decrement が意図した step の倍数へ戻ること
        // （Bugbot 指摘「Set skips step grid snap」対応）。
        let mut s = AngleSlider::new(0, 10);
        assert!(dispatch(&mut s, "set", "37"));
        assert_eq!(s.angle_deg(), 40);
        assert!(dispatch(&mut s, "increment", ""));
        assert_eq!(s.angle_deg(), 50);
    }

    #[test]
    fn dispatch_set_snaps_down_when_closer_to_lower_grid_point() {
        let mut s = AngleSlider::new(0, 15);
        // 32 は 30 (差 2) の方が 45 (差 13) より近い。
        assert!(dispatch(&mut s, "set", "32"));
        assert_eq!(s.angle_deg(), 30);
    }

    #[test]
    fn dispatch_set_snap_wraps_near_360_to_zero() {
        // step=10 のとき 355 は 360 (=0) の方が 350 より近い。
        let mut s = AngleSlider::new(0, 10);
        assert!(dispatch(&mut s, "set", "355"));
        assert_eq!(s.angle_deg(), 0);
    }

    #[test]
    fn dispatch_set_snap_360_equals_zero_with_non_divisor_step() {
        // step=25 は 360 を割り切らない（最終区間 350..360 は長さ 10
        // しかない）。線形グリッドをそのまま延長すると 360 が手前の
        // 350 へ丸められてしまう回帰（Bugbot 指摘「Snap breaks
        // 360-equals-0」対応）。"360" は必ず "0" にスナップされる契約
        // を確認する。
        let mut s = AngleSlider::new(0, 25);
        assert!(dispatch(&mut s, "set", "360"));
        assert_eq!(s.angle_deg(), 0);
    }

    #[test]
    fn dispatch_set_snap_wraps_near_360_to_zero_with_non_divisor_step() {
        // step=25 のとき、最終区間 350..360(=0) の中点は 355。358 は
        // 手前の通常グリッド点 350 (差 8) より 0 (差 2) の方が円周上で
        // 近いため、circular wrap-around 最近接スナップで 0 になる。
        let mut s = AngleSlider::new(0, 25);
        assert!(dispatch(&mut s, "set", "358"));
        assert_eq!(s.angle_deg(), 0);
    }

    #[test]
    fn dispatch_set_snap_stays_on_lower_grid_point_before_wrap_midpoint() {
        // step=25 のとき、354 は最終区間の中点 355 未満のため、手前の
        // 通常グリッド点 350 (差 4) の方が 0 (差 6) より近い。
        let mut s = AngleSlider::new(0, 25);
        assert!(dispatch(&mut s, "set", "354"));
        assert_eq!(s.angle_deg(), 350);
    }

    #[test]
    fn dispatch_set_rejects_invalid_payload() {
        let mut s = AngleSlider::new(5, 1);
        for bogus in ["abc", "-1", "361", "9999", ""] {
            assert!(!dispatch(&mut s, "set", bogus));
            assert_eq!(s.angle_deg(), 5);
        }
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut s = AngleSlider::new(5, 1);
        assert!(!dispatch(&mut s, "no_such_action", "x"));
        assert_eq!(s.angle_deg(), 5);
    }

    // --- SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&AngleSlider::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- hydration 経路 ---

    #[test]
    fn hydration_round_trip() {
        let s = AngleSlider::new(200, 15);
        let rendered = render(&render_for_hydration(&s));
        assert!(rendered.contains(r#"data-hydrate-value="200""#));
        assert!(rendered.contains(r#"data-hydrate-step="15""#));

        let restored = AngleSlider::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = AngleSlider::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-value".to_string())
        );
    }

    #[test]
    fn from_hydration_attrs_invalid_value_does_not_panic() {
        let bogus_sets: Vec<Vec<(String, String)>> = vec![
            // value が範囲外（360 は hydration 経路では拒否。SSR 出力は
            // 常に 0..=359 のため 360 が来るのは改ざんのみ）。
            vec![
                ("data-hydrate-value".to_string(), "360".to_string()),
                ("data-hydrate-step".to_string(), "1".to_string()),
            ],
            // value が非整数。
            vec![
                ("data-hydrate-value".to_string(), "NaN".to_string()),
                ("data-hydrate-step".to_string(), "1".to_string()),
            ],
            // step が 0。
            vec![
                ("data-hydrate-value".to_string(), "40".to_string()),
                ("data-hydrate-step".to_string(), "0".to_string()),
            ],
            // value が XSS ペイロード。
            vec![
                (
                    "data-hydrate-value".to_string(),
                    "<script>alert(1)</script>".to_string(),
                ),
                ("data-hydrate-step".to_string(), "1".to_string()),
            ],
        ];
        for attrs in bogus_sets {
            let err = AngleSlider::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: name/attrs/children にペイロードを渡してもエスケープされる ---

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
    fn hydration_xss_payload_in_value_is_rejected_not_rendered() {
        let attrs = vec![
            (
                "data-hydrate-value".to_string(),
                "<script>alert(1)</script>".to_string(),
            ),
            ("data-hydrate-step".to_string(), "1".to_string()),
        ];
        let err = AngleSlider::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

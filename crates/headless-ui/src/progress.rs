//! Progress（進捗表示、Linear）headless コンポーネント（イシュー #544、親 #542）。
//!
//! ark-ui の Progress
//!（`.claude/skills/ark-ui/references/components/display/progress-linear.md`）を
//! 参考に、Root / Label / ValueText / Track / Range の 5 anatomy パーツと、
//! Phase 1（#524）の [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 抽象へ直接乗る値状態機械
//! [`Progress`] を提供する。
//!
//! # `data-state` 語彙について（[`crate::state::Disclosure`]/[`crate::state::SingleSelect`] を使わない理由）
//!
//! [`crate::state::Disclosure`]/[`crate::state::SingleSelect`] は
//! `"open"/"closed"` や選択 ID という語彙に固定されている
//! （[`crate::state::OpenState`]）。Progress は数値 `value`（`min`..=`max`、
//! または indeterminate を表す `None`）を状態として持ち、`data-state` は
//! その数値から導出する `"indeterminate"`/`"loading"`/`"complete"`
//!（Zag.js 準拠語彙）である。[`crate::switch::Switch`]（#537）と同じ判断で、
//! 本モジュールも [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装し、Phase 1 が確立した
//! dispatch 契約（未知アクション no-op）・fail-closed hydration という
//! **統合様式**にのみ準拠する。
//!
//! # 呼び出し文脈
//!
//! SSR は [`Progress::new`] で値を正規化してから各パーツメソッド
//! （[`Progress::root`]/[`Progress::label`]/[`Progress::value_text`]/
//! [`Progress::track`]/[`Progress::range`]）を呼んで組み立てる。CSR/hydration は
//! [`Progress`] を経由し、dispatch（`"set"`/`"indeterminate"`）で状態遷移する。
//! `fandhe-frontend-pre-styled-ui`（#546〜）が本モジュールを呼んでスタイル済み
//! Progress を組み立てる想定である。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`）はすべて `&'static str` リテラルで
//!   固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`crate::anatomy`]/[`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（数値属性・`aria-valuetext`・呼び出し側 `attrs`・children テキスト）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - 数値属性値（`data-value`/`data-max`/`aria-valuemin`/`aria-valuemax`/
//!   `aria-valuenow`）はサーバー側で有限性検証・`[min, max]` へ clamp 済みの
//!   `f64` の文字列表現（[`fmt_num`]）のみを出力する。任意の呼び出し側文字列を
//!   これらの数値スロットへ直接通す経路は持たない（fail-closed 正規化は
//!   [`Progress::new`] が一元的に担う）。
//! - `data-state` 値語彙（`"indeterminate"`/`"loading"`/`"complete"`）は本
//!   モジュール内で一元管理し（[`Progress::data_state`]）、パーツ関数間で
//!   分裂させない。
//! - hydration 属性（`data-hydrate-min`/`data-hydrate-max`/`data-hydrate-value`）
//!   はクライアント側で改ざんされうる入力として扱う。[`Progress`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は panic せず
//!   `HydrateError` を返す（パース不能・非有限・`min >= max`・範囲外 value を
//!   すべて拒否する）。
//! - `orientation` は `min`/`max`/`value` と同じくクライアント改ざんの
//!   対象として扱い、`data-hydrate-orientation` を経由して hydration に
//!   含める（`docs/api/hydration-state-format.md` の `<field>` 命名規約に
//!   従う）。含めない設計だと `Hydrate` ラウンドトリップ後に
//!   vertical Progress が horizontal へ静かに反転する不変条件違反が生じる
//!   ため（イシュー #544 PR #570 レビュー指摘）、他フィールドと同様に
//!   fail-closed（未知の値・欠落は `HydrateError`）で往復させる。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::role;
use crate::data_attrs::{data_orientation, data_state, Orientation};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Progress の anatomy（`data-scope="progress"`）。
const ANATOMY: Anatomy = anatomy("progress");

/// `data-state` 属性値 "indeterminate"（value が `None` のとき）。
/// Zag.js（ark-ui 基盤）準拠の値語彙。本モジュールが一元管理し、
/// パーツ関数間で分裂させない。
const DATA_STATE_INDETERMINATE: &str = "indeterminate";
/// `data-state` 属性値 "loading"（`min <= value < max` のとき）。
const DATA_STATE_LOADING: &str = "loading";
/// `data-state` 属性値 "complete"（`value == max` のとき）。
const DATA_STATE_COMPLETE: &str = "complete";

/// f64 数値属性値の文字列化を一元化するヘルパ。
///
/// Rust の `f64` の `Display` 実装は整数値を `"40"` のように小数点なしで
/// 出力する（`"40.0"` にはならない）。属性値の表記をパーツ間・
/// 状態機械間で分裂させないため、数値属性の文字列化は必ず本関数を経由する。
fn fmt_num(value: f64) -> String {
    format!("{value}")
}

/// `min`/`max`/`value` を fail-closed に正規化する。
///
/// - `min`/`max` が非有限、または `min >= max` の場合は既定 `(0.0, 100.0)`
///   へフォールバックする（呼び出し側の不正な入力で panic させない、
///   ライブラリコードの panic 回避規約に従う防御的実装）。
/// - `value` が非有限（`NaN`/`inf`）な場合は indeterminate（`None`）として
///   扱う。有限な場合は正規化後の `[min, max]` へ clamp する。
fn normalize(min: f64, max: f64, value: Option<f64>) -> (f64, f64, Option<f64>) {
    let (min, max) = if min.is_finite() && max.is_finite() && min < max {
        (min, max)
    } else {
        (0.0, 100.0)
    };
    let value = match value {
        Some(v) if v.is_finite() => Some(v.clamp(min, max)),
        _ => None,
    };
    (min, max, value)
}

/// Progress の値状態機械（Linear、ark-ui 準拠）。
///
/// `value = None` は indeterminate（不定進捗）を表す。`Default` は
/// `min=0.0, max=100.0, value=Some(0.0), orientation=Horizontal`
/// （SSR の「未開始」初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Progress {
    min: f64,
    max: f64,
    value: Option<f64>,
    orientation: Orientation,
}

impl Default for Progress {
    fn default() -> Self {
        Self::new(0.0, 100.0, Some(0.0), Orientation::Horizontal)
    }
}

impl Progress {
    /// `data-hydrate-min` 属性名のフィールド部分。
    pub const FIELD_MIN: &'static str = "min";
    /// `data-hydrate-max` 属性名のフィールド部分。
    pub const FIELD_MAX: &'static str = "max";
    /// `data-hydrate-value` 属性名のフィールド部分
    /// （値は数値文字列、または indeterminate を表す `"indeterminate"`）。
    pub const FIELD_VALUE: &'static str = "value";
    /// indeterminate（不定進捗）を表す `data-hydrate-value` の予約値。
    pub const HYDRATE_VALUE_INDETERMINATE: &str = "indeterminate";
    /// `data-hydrate-orientation` 属性名のフィールド部分。
    pub const FIELD_ORIENTATION: &'static str = "orientation";

    /// 指定した値で [`Progress`] を生成する（[`normalize`] で fail-closed
    /// 正規化する。呼び出し側の不正な入力で panic しない）。
    #[must_use]
    pub fn new(min: f64, max: f64, value: Option<f64>, orientation: Orientation) -> Self {
        let (min, max, value) = normalize(min, max, value);
        Self {
            min,
            max,
            value,
            orientation,
        }
    }

    /// 現在の値（`None` は indeterminate）。
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

    /// 現在の向き（`data-orientation`/hydration ラウンドトリップの対象）。
    #[must_use]
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// 進捗率（0.0..=100.0）。indeterminate のときは `None`。
    #[must_use]
    pub fn percent(&self) -> Option<f64> {
        self.value
            .map(|v| (v - self.min) / (self.max - self.min) * 100.0)
    }

    /// 現在の `data-state` 属性値
    /// （`"indeterminate"`/`"loading"`/`"complete"`）。
    #[must_use]
    pub fn data_state(&self) -> &'static str {
        match self.value {
            None => DATA_STATE_INDETERMINATE,
            Some(v) if v >= self.max => DATA_STATE_COMPLETE,
            Some(_) => DATA_STATE_LOADING,
        }
    }

    /// Root パーツ（`div`、`role="progressbar"`）。
    ///
    /// `aria-valuenow`/`data-value` は determinate（`value = Some(_)`）の
    /// ときのみ出力し、indeterminate では省略する（WAI-ARIA `progressbar`
    /// ロールの規定どおり）。`aria_valuetext` は `Some` のときのみ出力する。
    #[must_use]
    pub fn root<'a>(
        &self,
        aria_valuetext: Option<&str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let min_s = fmt_num(self.min);
        let max_s = fmt_num(self.max);
        let value_s = self.value.map(fmt_num);

        let mut merged: Vec<(&str, &str)> = vec![
            data_state(self.data_state()),
            ("data-max", max_s.as_str()),
            data_orientation(self.orientation),
            role("progressbar"),
            ("aria-valuemin", min_s.as_str()),
            ("aria-valuemax", max_s.as_str()),
        ];
        if let Some(v) = value_s.as_deref() {
            merged.push(("data-value", v));
            merged.push(("aria-valuenow", v));
        }
        if let Some(text) = aria_valuetext {
            merged.push(("aria-valuetext", text));
        }
        merged.extend(attrs);
        ANATOMY.part("root", "div", merged, children)
    }

    /// Label パーツ（`span`）。装飾用パーツ（意味論的なラベル関連付けは
    /// 呼び出し側が `id`/`aria-labelledby` を `attrs` 経由で配線する）。
    #[must_use]
    pub fn label<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(self.data_state())];
        merged.extend(attrs);
        ANATOMY.part("label", "span", merged, children)
    }

    /// ValueText パーツ（`span`）。表示テキストは `children`（呼び出し側が
    /// 整形する。`formatOptions`/`locale` 相当の整形機能は持たない）。
    #[must_use]
    pub fn value_text<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(self.data_state())];
        merged.extend(attrs);
        ANATOMY.part("value-text", "span", merged, children)
    }

    /// Track パーツ（`div`）。
    #[must_use]
    pub fn track<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> = vec![
            data_state(self.data_state()),
            data_orientation(self.orientation),
        ];
        merged.extend(attrs);
        ANATOMY.part("track", "div", merged, children)
    }

    /// Range パーツ（`div`）。幅スタイルは付与しない（headless 中立。
    /// styled 層/呼び出し側が [`Progress::percent`] を使って `attrs` 経由で
    /// `style` を渡す）。
    #[must_use]
    pub fn range<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> = vec![
            data_state(self.data_state()),
            data_orientation(self.orientation),
        ];
        merged.extend(attrs);
        ANATOMY.part("range", "div", merged, children)
    }
}

/// Progress のアクション（WASM 境界の文字列 dispatch と
/// [`Progress::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgressAction {
    /// 値を設定する（`[min, max]` へ clamp して反映する）。
    SetValue(f64),
    /// indeterminate（不定進捗）にする。
    SetIndeterminate,
}

impl Component for Progress {
    type Action = ProgressAction;

    /// `ProgressAction::SetValue` は非有限（`NaN`/`inf`）を fail-closed に
    /// 無視する（no-op）。[`normalize`]/[`Progress::decode_action`] が課す
    /// 「`value` は有限値または `None`」という本モジュールの不変条件を
    /// `update()` 単体でも維持するため（`decode_action` を経由しない直接
    /// `ProgressAction::SetValue` 構築・呼び出しからも同じ不変条件を守る）。
    fn update(&mut self, action: ProgressAction) {
        match action {
            ProgressAction::SetValue(v) => {
                if v.is_finite() {
                    self.value = Some(v.clamp(self.min, self.max));
                }
            }
            ProgressAction::SetIndeterminate => {
                self.value = None;
            }
        }
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > track > range）。公開 UI としての利用は想定しない
    /// （実際の UI 構築は §パーツメソッド群を呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        self.root(
            None,
            Vec::new(),
            vec![self.track(Vec::new(), vec![self.range(Vec::new(), Vec::new())])],
        )
    }

    /// `"set"`: payload を `str::parse::<f64>()` でパースし、非有限
    /// （`NaN`/`inf`）またはパース不能な場合は `None`（fail-closed、
    /// dispatch は no-op）。`"indeterminate"`: payload 不使用。
    fn decode_action(name: &str, payload: &str) -> Option<ProgressAction> {
        match name {
            "set" => payload
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .map(ProgressAction::SetValue),
            "indeterminate" => Some(ProgressAction::SetIndeterminate),
            _ => None,
        }
    }
}

impl Hydrate for Progress {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let value_s = match self.value {
            Some(v) => fmt_num(v),
            None => Self::HYDRATE_VALUE_INDETERMINATE.to_string(),
        };
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
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_VALUE),
                value_s,
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ORIENTATION),
                self.orientation.as_str().to_string(),
            ),
        ]
    }

    /// クライアント改ざん入力として扱う。欠落は
    /// [`HydrateError::MissingAttr`]、パース不能・非有限・`min >= max`・
    /// 範囲外 value・未知の `orientation` 値は
    /// [`HydrateError::InvalidValue`]（panic しない）。`orientation` も
    /// `min`/`max`/`value` と同じくラウンドトリップの対象であり
    /// （モジュール doc 参照）、`"horizontal"`/`"vertical"` 以外は拒否する。
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

        let attr_name_value = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_VALUE);
        let value = if value_raw == Self::HYDRATE_VALUE_INDETERMINATE {
            None
        } else {
            let v = value_raw
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| HydrateError::InvalidValue {
                    attr: attr_name_value.clone(),
                    reason: "expected a finite number or \"indeterminate\"".to_string(),
                })?;
            if v < min || v > max {
                return Err(HydrateError::InvalidValue {
                    attr: attr_name_value,
                    reason: "expected value within [min, max]".to_string(),
                });
            }
            Some(v)
        };

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

        Ok(Self {
            min,
            max,
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

    // --- 各パーツの data-scope/data-part/data-state 出力 ---

    #[test]
    fn root_outputs_scope_part_state_role_and_min_max() {
        let p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);
        let html = render(&p.root(None, vec![], vec![]));
        assert!(html.contains(r#"data-scope="progress""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="loading""#));
        assert!(html.contains(r#"role="progressbar""#));
        assert!(html.contains(r#"aria-valuemin="0""#));
        assert!(html.contains(r#"aria-valuemax="100""#));
        assert!(html.contains(r#"aria-valuenow="40""#));
        assert!(html.contains(r#"data-value="40""#));
        assert!(html.contains(r#"data-max="100""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
    }

    #[test]
    fn root_indeterminate_omits_valuenow_and_data_value() {
        let p = Progress::new(0.0, 100.0, None, Orientation::Horizontal);
        let html = render(&p.root(None, vec![], vec![]));
        assert!(html.contains(r#"data-state="indeterminate""#));
        assert!(!html.contains("aria-valuenow"));
        assert!(!html.contains("data-value"));
        // aria-valuemin/aria-valuemax は indeterminate でも常に出力する。
        assert!(html.contains(r#"aria-valuemin="0""#));
        assert!(html.contains(r#"aria-valuemax="100""#));
    }

    #[test]
    fn root_complete_when_value_equals_max() {
        let p = Progress::new(0.0, 100.0, Some(100.0), Orientation::Horizontal);
        let html = render(&p.root(None, vec![], vec![]));
        assert!(html.contains(r#"data-state="complete""#));
        assert!(html.contains(r#"aria-valuenow="100""#));
    }

    #[test]
    fn root_aria_valuetext_only_when_some() {
        let p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);
        let with_text = render(&p.root(Some("40 percent"), vec![], vec![]));
        assert!(with_text.contains(r#"aria-valuetext="40 percent""#));

        let without_text = render(&p.root(None, vec![], vec![]));
        assert!(!without_text.contains("aria-valuetext"));
    }

    #[test]
    fn label_outputs_scope_part_and_state() {
        let p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);
        let html = render(&p.label(vec![], vec![text("Upload progress")]));
        assert!(html.contains(r#"data-scope="progress""#));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains(r#"data-state="loading""#));
        assert!(html.contains("Upload progress"));
    }

    #[test]
    fn value_text_outputs_scope_part_and_state() {
        let p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);
        let html = render(&p.value_text(vec![], vec![text("40%")]));
        assert!(html.contains(r#"data-scope="progress""#));
        assert!(html.contains(r#"data-part="value-text""#));
        assert!(html.contains(r#"data-state="loading""#));
        assert!(html.contains("40%"));
    }

    #[test]
    fn track_and_range_output_scope_part_state_and_orientation() {
        let p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Vertical);
        let track_html = render(&p.track(vec![], vec![]));
        assert!(track_html.contains(r#"data-scope="progress""#));
        assert!(track_html.contains(r#"data-part="track""#));
        assert!(track_html.contains(r#"data-state="loading""#));
        assert!(track_html.contains(r#"data-orientation="vertical""#));

        let range_html = render(&p.range(vec![], vec![]));
        assert!(range_html.contains(r#"data-part="range""#));
        assert!(range_html.contains(r#"data-orientation="vertical""#));
        // headless 中立: 幅スタイルは付与しない。
        assert!(!range_html.contains("style"));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let p = Progress::default();
        let html = render(&p.root(
            None,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="progress""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- 正規化（fail-closed） ---

    #[test]
    fn new_clamps_out_of_range_value() {
        let p = Progress::new(0.0, 100.0, Some(150.0), Orientation::Horizontal);
        assert_eq!(p.value(), Some(100.0));
        let p = Progress::new(0.0, 100.0, Some(-10.0), Orientation::Horizontal);
        assert_eq!(p.value(), Some(0.0));
    }

    #[test]
    fn new_non_finite_value_becomes_indeterminate() {
        let p = Progress::new(0.0, 100.0, Some(f64::NAN), Orientation::Horizontal);
        assert_eq!(p.value(), None);
        let p = Progress::new(0.0, 100.0, Some(f64::INFINITY), Orientation::Horizontal);
        assert_eq!(p.value(), None);
    }

    #[test]
    fn new_invalid_min_max_falls_back_to_defaults() {
        let p = Progress::new(100.0, 0.0, Some(40.0), Orientation::Horizontal);
        assert_eq!((p.min(), p.max()), (0.0, 100.0));

        let p = Progress::new(f64::NAN, 100.0, Some(40.0), Orientation::Horizontal);
        assert_eq!((p.min(), p.max()), (0.0, 100.0));

        let p = Progress::new(0.0, f64::INFINITY, Some(40.0), Orientation::Horizontal);
        assert_eq!((p.min(), p.max()), (0.0, 100.0));
    }

    #[test]
    fn percent_reflects_position_within_range() {
        let p = Progress::new(0.0, 200.0, Some(50.0), Orientation::Horizontal);
        assert_eq!(p.percent(), Some(25.0));
        let p = Progress::new(0.0, 100.0, None, Orientation::Horizontal);
        assert_eq!(p.percent(), None);
    }

    // --- Progress: dispatch 統合 ---

    #[test]
    fn progress_default_is_loading_at_zero() {
        let p = Progress::default();
        assert_eq!(p.value(), Some(0.0));
        assert_eq!(p.data_state(), DATA_STATE_LOADING);
    }

    #[test]
    fn progress_dispatch_set_updates_value_and_clamps() {
        let mut p = Progress::default();
        assert!(dispatch(&mut p, "set", "75.5"));
        assert_eq!(p.value(), Some(75.5));

        assert!(dispatch(&mut p, "set", "120"));
        assert_eq!(p.value(), Some(100.0));
    }

    #[test]
    fn progress_dispatch_indeterminate() {
        let mut p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);
        assert!(dispatch(&mut p, "indeterminate", ""));
        assert_eq!(p.value(), None);
    }

    #[test]
    fn progress_dispatch_set_rejects_invalid_payload() {
        let mut p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);
        for bogus in ["abc", "NaN", "inf", "-inf", ""] {
            assert!(!dispatch(&mut p, "set", bogus));
            assert_eq!(p.value(), Some(40.0));
        }
    }

    /// イシュー #544 PR #570 レビュー指摘: `decode_action` を経由せず
    /// `ProgressAction::SetValue` を直接構築して `update()` を呼んでも、
    /// 非有限値（`NaN`/`inf`）が `value` へ混入して
    /// `aria-valuenow`/`data-value`/`percent`/`data-state` を汚染しない
    /// （「有限値または `None`」不変条件を `update()` 単体でも維持する）。
    #[test]
    fn progress_update_rejects_non_finite_set_value_directly() {
        let mut p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);
        for bogus in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            Component::update(&mut p, ProgressAction::SetValue(bogus));
            assert_eq!(p.value(), Some(40.0));
            assert_eq!(p.data_state(), DATA_STATE_LOADING);
            assert_eq!(p.percent(), Some(40.0));
        }
    }

    #[test]
    fn progress_dispatch_ignores_unknown_action() {
        let mut p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);
        assert!(!dispatch(&mut p, "no_such_action", "x"));
        assert_eq!(p.value(), Some(40.0));
    }

    // --- Progress: SSR 状態なし初期描画 ---

    #[test]
    fn progress_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Progress::default().view());
        assert!(rendered.contains(r#"data-state="loading""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- Progress: hydration 経路 ---

    #[test]
    fn progress_hydration_round_trip_determinate() {
        let p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);
        let rendered = render(&render_for_hydration(&p));
        assert!(rendered.contains(r#"data-hydrate-min="0""#));
        assert!(rendered.contains(r#"data-hydrate-max="100""#));
        assert!(rendered.contains(r#"data-hydrate-value="40""#));
        assert!(rendered.contains(r#"data-hydrate-orientation="horizontal""#));

        let restored = Progress::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored, p);
    }

    #[test]
    fn progress_hydration_round_trip_indeterminate() {
        let p = Progress::new(0.0, 100.0, None, Orientation::Horizontal);
        let rendered = render(&render_for_hydration(&p));
        assert!(rendered.contains(r#"data-hydrate-value="indeterminate""#));

        let restored = Progress::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored, p);
    }

    /// イシュー #544 PR #570 レビュー指摘: hydration ラウンドトリップ後に
    /// vertical Progress が horizontal へ静かに反転しないことを保証する
    /// （`orientation` も他フィールドと同じくラウンドトリップ対象）。
    #[test]
    fn progress_hydration_round_trip_preserves_vertical_orientation() {
        let p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Vertical);
        let rendered = render(&render_for_hydration(&p));
        assert!(rendered.contains(r#"data-hydrate-orientation="vertical""#));

        let restored = Progress::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored.orientation(), Orientation::Vertical);
        assert_eq!(restored, p);
    }

    #[test]
    fn progress_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Progress::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-min".to_string())
        );
    }

    #[test]
    fn progress_from_hydration_attrs_invalid_value_does_not_panic() {
        let bogus_sets: Vec<Vec<(String, String)>> = vec![
            // min が非有限。
            vec![
                ("data-hydrate-min".to_string(), "NaN".to_string()),
                ("data-hydrate-max".to_string(), "100".to_string()),
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
                ("data-hydrate-value".to_string(), "150".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // value が XSS ペイロード。
            vec![
                ("data-hydrate-min".to_string(), "0".to_string()),
                ("data-hydrate-max".to_string(), "100".to_string()),
                (
                    "data-hydrate-value".to_string(),
                    "<script>alert(1)</script>".to_string(),
                ),
                (
                    "data-hydrate-orientation".to_string(),
                    "horizontal".to_string(),
                ),
            ],
            // orientation が未知の語彙。
            vec![
                ("data-hydrate-min".to_string(), "0".to_string()),
                ("data-hydrate-max".to_string(), "100".to_string()),
                ("data-hydrate-value".to_string(), "40".to_string()),
                (
                    "data-hydrate-orientation".to_string(),
                    "diagonal".to_string(),
                ),
            ],
        ];
        for attrs in bogus_sets {
            let err = Progress::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: aria-valuetext/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn aria_valuetext_payload_is_escaped_on_render() {
        let p = Progress::default();
        let html = render(&p.root(Some(ATTR_BREAK_PAYLOAD), vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let p = Progress::default();
        let html = render(&p.root(None, vec![("data-testid", ATTR_BREAK_PAYLOAD)], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let p = Progress::default();
        let html = render(&p.label(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}

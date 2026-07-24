//! DateInput（年・月・日セグメント入力）headless コンポーネント
//! （イシュー #834、親トラッキング #832）。
//!
//! ark-ui の DateInput 相当を、Root / Label / Control / SegmentGroup /
//! Segment / HiddenInput の 6 anatomy パーツと、セグメントごとの値・
//! フォーカス位置を持つ独自の値状態機械 [`DateInput`] として提供する。
//! 暦計算そのもの（うるう年・月ごとの日数・ISO 8601 パース）は
//! [`crate::date`]（イシュー #833）へ全面的に委譲し、本モジュールは
//! セグメント単位の SSR マークアップ・dispatch・hydration のみを担う。
//!
//! # `date_input::segment_group` と `crate::segment_group` の違い
//!
//! 本モジュールの [`segment_group`] は `data-scope="date-input"` 内の
//! 1 パーツ（Year/Month/Day セグメントのコンテナ）であり、独立した
//! headless コンポーネントである [`crate::segment_group::SegmentGroup`]
//! （segmented control、`data-scope="segment-group"`）とは anatomy の
//! スコープも状態機械も完全に別物である。同名だが無関係の 2 者である
//! ことを呼び出し側は混同しないこと。
//!
//! # 独自状態機械にした理由
//!
//! [`crate::state`] の既存語彙（`Disclosure`/`SingleSelect`/`MultiSelect`/
//! `Checkable`/`TextInput`）はいずれも「年・月・日の 3 つの数値スロット +
//! フォーカス位置」という形の状態を表現できない。[`crate::pin_input::PinInput`]/
//! [`crate::number_input::NumberInput`] と同じ判断（両モジュール rustdoc
//! 参照）で、本モジュールも [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装する。
//!
//! # 呼び出し文脈
//!
//! SSR は [`DateInput::new`] で年月日・`min`/`max` を指定してから各パーツ
//! メソッド（[`DateInput::root`]/[`DateInput::label`]/[`DateInput::control`]/
//! [`DateInput::segment_group`]/[`DateInput::segment`]/[`DateInput::hidden_input`]）
//! を呼んで組み立てる。CSR/hydration は [`DateInput`] を経由し、dispatch
//! （`"increment"`/`"decrement"`/`"focus"`/`"set-segment"`/`"set"`/`"clear"`）
//! で状態遷移する。`fandhe-frontend-pre-styled-ui` が本モジュールを呼んで
//! スタイル済み DateInput を組み立てる想定である。
//!
//! # fail-closed な日付検証（受け入れ条件）
//!
//! 年・月・日の各セグメントは個別に `[0, 9999]`/`[1, 12]`/`[1, 31]` へ
//! クランプされた `Option` として保持される（[`normalize_segments`]）。3
//! セグメントすべてが充足したときのみ [`crate::date::PlainDate::new`] で
//! 実在する日付か検証し、`2/30` のような存在しない日付は
//! [`DateInput::value`] が `None` を返す（[`DateInput::is_invalid`] が
//! `true` になり [`hidden_input`] は空値を出力する。**セグメント自体の値は
//! 破棄しない**。これは Web フォームで「不正な入力を可視化しつつユーザーに
//! 訂正させる」UX を優先する意図的な設計であり、hydration 側もこの部分的に
//! 不正な状態をそのまま復元する契約とする、下記「hydration 契約」参照）。
//!
//! # hydration 契約
//!
//! - `data-hydrate-year`/`-month`/`-day` は各セグメントの値
//!   （未入力は予約値 `"none"`）を運ぶ。**構造的に妥当な範囲
//!   （年 `0..=9999`・月 `1..=12`・日 `1..=31`）を外れる値、パース不能な
//!   文字列のみを [`HydrateError::InvalidValue`] として拒否する**。年月日の
//!   組み合わせとして存在しない日付（`2024-02-30` 等）は構造的には妥当な
//!   3 整数の組であるため hydration としては受理し、[`DateInput::is_invalid`]
//!   が `true` を返す状態としてそのまま復元する（モジュール doc「fail-closed
//!   な日付検証」参照。この点は「値そのものが破損している」
//!   [`crate::number_input::NumberInput`] の非有限値拒否とは性質が異なる）。
//! - `data-hydrate-min`/`-max` は ISO 8601 文字列（未指定は予約値
//!   `"none"`）。パース不能な文字列は拒否し、両方が指定されていて
//!   `min > max` の場合も拒否する（fail-closed、値の入れ替えはしない。
//!   [`crate::number_input::NumberInput`] の hydration が `min > max` を
//!   拒否するのと同じ契約）。
//! - `focused`（キーボードフォーカス位置という ephemeral な DOM 状態）は
//!   [`crate::pin_input::PinInput`] と同じ理由で hydration では運ばない。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`inputmode`/`tabindex`/`type`）は
//!   すべて `&'static str` リテラルで固定しており、動的値が属性名スロットへ
//!   混入する経路はない（[`crate::anatomy`]/[`crate::aria`]/
//!   [`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（`name`/`id`/整形済みセグメント文字列/呼び出し側 `attrs`/
//!   children）は [`fandhe_frontend_core::render`] の既定エスケープを
//!   必ず経由する。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - dispatch `"set-segment"`/`"set"`/`"focus"` の payload はクライアント
//!   由来の信頼できない入力として厳密パース + 範囲検証する
//!   （[`DateInput::decode_action`]）。パース失敗・範囲外・未知セグメント名は
//!   すべて no-op（fail-closed）。
//! - hidden input の値は年月日 3 セグメントがすべて充足し、かつ
//!   [`crate::date::PlainDate::new`] が受理する実在の日付であり、かつ
//!   `min`/`max` の範囲内にあるときのみ ISO 8601 文字列として出力する
//!   （[`DateInput::value`] が一元的に判定する）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - hour/minute/second セグメント（granularity。[`crate::date`] が
//!   date-only モデルであるため）。
//! - `selectionMode: range`（範囲選択）。
//! - locale 依存の桁順・区切り文字整形（決定性優先で year→month→day の
//!   ISO 固定順のみを提供する）。
//! - キーボード操作（ArrowUp/Down 等）の実 DOM 配線（他コンポーネント同様、
//!   `fandhe-frontend-wasm-full` 側の後続責務）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::aria_invalid;
use crate::data_attrs::{data_disabled, data_invalid, data_readonly};
use crate::date::{days_in_month, DateError, PlainDate};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// DateInput の anatomy（`data-scope="date-input"`）。
const ANATOMY: Anatomy = anatomy("date-input");

/// `data-placeholder` 存在属性。セグメントが未入力のときのみ出力する
/// （[`crate::data_attrs::data_disabled`] と同じ「存在で真を表す」規約）。
/// DateInput 固有の語彙であるため、本モジュール内で個別に定義する
/// （[`crate::pin_input`] の `data_complete` と同型の判断）。
fn data_placeholder(placeholder: bool) -> Option<(&'static str, &'static str)> {
    placeholder.then_some(("data-placeholder", ""))
}

/// DateInput が扱う 3 種のセグメント（year/month/day、ISO 固定順）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DateSegment {
    /// 年（`0000..=9999`）。
    Year,
    /// 月（`1..=12`）。
    Month,
    /// 日（`1..=`当該年月の日数）。
    Day,
}

impl DateSegment {
    /// hydration・dispatch payload で使う固定語彙（小文字英字のみ）。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Year => "year",
            Self::Month => "month",
            Self::Day => "day",
        }
    }

    /// [`Self::as_str`] の逆変換。未知の値は `None`（fail-closed）。
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "year" => Some(Self::Year),
            "month" => Some(Self::Month),
            "day" => Some(Self::Day),
            _ => None,
        }
    }

    /// `aria-label` に使う英語ラベル（ユーザー向け文字列は英語、
    /// `.claude/rules/japanese-style.md` 参照）。
    #[must_use]
    const fn aria_label(self) -> &'static str {
        match self {
            Self::Year => "Year",
            Self::Month => "Month",
            Self::Day => "Day",
        }
    }

    /// 未入力時に表示するプレースホルダ文字列。
    #[must_use]
    const fn placeholder(self) -> &'static str {
        match self {
            Self::Year => "yyyy",
            Self::Month => "mm",
            Self::Day => "dd",
        }
    }
}

/// Root パーツ（`div`）。
#[must_use]
pub fn root<'a>(
    disabled: bool,
    invalid: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(disabled));
    merged.extend(data_invalid(invalid));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。`control_id` を与えると `for` 属性で関連付ける
/// （省略時は呼び出し側が `attrs` 経由で配線する、[`crate::number_input::label`]
/// と同じ契約）。
#[must_use]
pub fn label<'a>(
    disabled: bool,
    invalid: bool,
    control_id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = control_id {
        merged.push(("for", id));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(data_invalid(invalid));
    merged.extend(attrs);
    ANATOMY.part("label", "label", merged, children)
}

/// Control パーツ（`div`）。[`segment_group`] と [`hidden_input`] のラッパー。
#[must_use]
pub fn control<'a>(
    disabled: bool,
    invalid: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(disabled));
    merged.extend(data_invalid(invalid));
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// SegmentGroup パーツ（`div`）。Year/Month/Day の [`segment`] を並べる
/// コンテナ（装飾用パーツ、状態を持たない）。
#[must_use]
pub fn segment_group<'a>(
    disabled: bool,
    invalid: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(disabled));
    merged.extend(data_invalid(invalid));
    merged.extend(attrs);
    ANATOMY.part("segment-group", "div", merged, children)
}

/// [`segment`]/[`DateInput::segment`] が受け取る disabled/invalid/readonly
/// フラグ束。3 個の独立した `bool` 引数のままだと他の引数と合わせて clippy
/// `too_many_arguments`（既定閾値 7）を超えるため、
/// [`crate::number_input::NumberInputFlags`] と同型の薄い構造体としてまとめる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DateSegmentFlags {
    /// `data-disabled` を付与し、`tabindex` を出力しないかどうか。
    pub disabled: bool,
    /// `aria-invalid="true"`・`data-invalid` を付与するかどうか。
    pub invalid: bool,
    /// `data-readonly` を付与するかどうか。
    pub readonly: bool,
}

/// Segment パーツ（`div role="spinbutton"`）。年/月/日 1 個分の編集可能単位。
///
/// WAI-ARIA `spinbutton` パターンに従い `aria-valuemin`/`aria-valuemax` を
/// 常に出力し、`aria-valuenow` は `value` が `Some` のときのみ出力する
/// （[`crate::number_input::input`] と同じ方針）。未入力時は
/// [`DateSegment::placeholder`] をテキストとして表示し `data-placeholder`
/// を付与する。
#[must_use]
pub fn segment<'a>(
    kind: DateSegment,
    value: Option<&'a str>,
    min: &'a str,
    max: &'a str,
    flags: DateSegmentFlags,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![
        ("role", "spinbutton"),
        ("inputmode", "numeric"),
        ("aria-label", kind.aria_label()),
        ("aria-valuemin", min),
        ("aria-valuemax", max),
    ];
    if let Some(v) = value {
        merged.push(("aria-valuenow", v));
    }
    if !flags.disabled {
        merged.push(("tabindex", "0"));
    }
    if flags.invalid {
        merged.push(aria_invalid(true));
    }
    merged.extend(data_disabled(flags.disabled));
    merged.extend(data_invalid(flags.invalid));
    merged.extend(data_readonly(flags.readonly));
    merged.extend(data_placeholder(value.is_none()));
    merged.extend(attrs);
    let text_content = fandhe_frontend_core::text(value.unwrap_or(kind.placeholder()));
    ANATOMY.part("segment", "div", merged, vec![text_content])
}

/// HiddenInput パーツ（`input type="hidden"`）。フォーム送信時に確定済み
/// 日付を ISO 8601 文字列として運ぶ（各 [`segment`] は `name` を持たない
/// ため、実際の送信値はこのパーツが唯一担う。[`crate::pin_input::hidden_input`]
/// と同型の契約）。
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
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("hidden-input", "input", merged, Vec::new())
}

/// [`DateInput`] に対する型付きアクション（WASM 境界の文字列 dispatch と
/// [`DateInput::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateInputAction {
    /// 現在フォーカス中のセグメントを 1 つ増やす（未フォーカスなら no-op）。
    Increment,
    /// 現在フォーカス中のセグメントを 1 つ減らす（未フォーカスなら no-op）。
    Decrement,
    /// 指定セグメントへフォーカスを移す。
    Focus(DateSegment),
    /// 指定セグメントへ値を直接設定する（クランプ済み）。
    SetSegment(DateSegment, i32),
    /// ISO 8601 文字列（[`PlainDate::parse_iso`] で検証済み）を丸ごと設定する。
    Set(PlainDate),
    /// 全セグメントを未入力に戻す。
    Clear,
}

/// 年/月/日それぞれの構造的な値域へクランプする（実在する日付かどうかの
/// 検証はここでは行わない。モジュール doc「fail-closed な日付検証」参照）。
fn clamp_year(year: i32) -> i32 {
    year.clamp(0, 9999)
}
fn clamp_month(month: i32) -> u8 {
    month.clamp(1, 12) as u8
}
fn clamp_day(day: i32) -> u8 {
    day.clamp(1, 31) as u8
}

/// `min`/`max` の指定順が逆転している場合は hydration と異なり SSR 構築時
/// のみ入れ替える（[`crate::number_input::normalize`] と同じ判断: 呼び出し側の
/// 引数取り違えを許容するのはプログラム構築 API 側のみで、hydration は
/// クライアント改ざんを想定するため入れ替えず拒否する）。
fn normalize_min_max(
    min: Option<PlainDate>,
    max: Option<PlainDate>,
) -> (Option<PlainDate>, Option<PlainDate>) {
    match (min, max) {
        (Some(a), Some(b)) if a > b => (Some(b), Some(a)),
        other => other,
    }
}

/// DateInput の値状態機械（ark-ui 準拠）。
///
/// `year`/`month`/`day` はそれぞれ `None`（未入力）または構造的値域内の値。
/// `focused` はキーボードフォーカス位置という ephemeral な DOM 状態であり
/// hydration では運ばない（モジュール doc 参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateInput {
    year: Option<i32>,
    month: Option<u8>,
    day: Option<u8>,
    min: Option<PlainDate>,
    max: Option<PlainDate>,
    focused: Option<DateSegment>,
}

impl Default for DateInput {
    /// 既定は全セグメント未入力・範囲制限なし。
    fn default() -> Self {
        Self::new(None, None, None, None, None)
    }
}

impl DateInput {
    /// `data-hydrate-year` 属性名のフィールド部分。
    pub const FIELD_YEAR: &'static str = "year";
    /// `data-hydrate-month` 属性名のフィールド部分。
    pub const FIELD_MONTH: &'static str = "month";
    /// `data-hydrate-day` 属性名のフィールド部分。
    pub const FIELD_DAY: &'static str = "day";
    /// `data-hydrate-min` 属性名のフィールド部分。
    pub const FIELD_MIN: &'static str = "min";
    /// `data-hydrate-max` 属性名のフィールド部分。
    pub const FIELD_MAX: &'static str = "max";
    /// 未入力（`None`）を表す hydration 属性の予約値。
    pub const HYDRATE_NONE: &'static str = "none";

    /// 指定した年月日・範囲で [`DateInput`] を生成する（[`clamp_year`]/
    /// [`clamp_month`]/[`clamp_day`]/[`normalize_min_max`] で fail-closed
    /// 正規化する。呼び出し側の不正な入力で panic しない）。
    #[must_use]
    pub fn new(
        year: Option<i32>,
        month: Option<u8>,
        day: Option<u8>,
        min: Option<PlainDate>,
        max: Option<PlainDate>,
    ) -> Self {
        let (min, max) = normalize_min_max(min, max);
        Self {
            year: year.map(clamp_year),
            month: month.map(|m| clamp_month(i32::from(m))),
            day: day.map(|d| clamp_day(i32::from(d))),
            min,
            max,
            focused: None,
        }
    }

    /// 年セグメントの現在値。
    #[must_use]
    pub fn year(&self) -> Option<i32> {
        self.year
    }

    /// 月セグメントの現在値。
    #[must_use]
    pub fn month(&self) -> Option<u8> {
        self.month
    }

    /// 日セグメントの現在値。
    #[must_use]
    pub fn day(&self) -> Option<u8> {
        self.day
    }

    /// 下限日付（未設定なら `None`）。
    #[must_use]
    pub fn min(&self) -> Option<PlainDate> {
        self.min
    }

    /// 上限日付（未設定なら `None`）。
    #[must_use]
    pub fn max(&self) -> Option<PlainDate> {
        self.max
    }

    /// 現在フォーカス中のセグメント。
    #[must_use]
    pub fn focused(&self) -> Option<DateSegment> {
        self.focused
    }

    /// 3 セグメントすべてが充足しているかどうか。
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.year.is_some() && self.month.is_some() && self.day.is_some()
    }

    /// 確定済みの日付（3 セグメント充足かつ実在する日付のときのみ
    /// `Some`。モジュール doc「fail-closed な日付検証」参照）。
    #[must_use]
    pub fn value(&self) -> Option<PlainDate> {
        let (y, m, d) = (self.year?, self.month?, self.day?);
        PlainDate::new(y, m, d).ok()
    }

    /// invalid かどうか（3 セグメント充足だが実在しない日付、または
    /// `min`/`max` の範囲外）。未充足（入力途中）は invalid 扱いしない。
    #[must_use]
    pub fn is_invalid(&self) -> bool {
        if self.is_complete() && self.value().is_none() {
            return true;
        }
        match self.value() {
            Some(v) => {
                if let Some(min) = self.min {
                    if v < min {
                        return true;
                    }
                }
                if let Some(max) = self.max {
                    if v > max {
                        return true;
                    }
                }
                false
            }
            None => false,
        }
    }

    /// 年セグメントのゼロ埋め表示文字列（未入力は `None`）。
    #[must_use]
    pub fn year_text(&self) -> Option<String> {
        self.year.map(|y| format!("{y:04}"))
    }

    /// 月セグメントのゼロ埋め表示文字列（未入力は `None`）。
    #[must_use]
    pub fn month_text(&self) -> Option<String> {
        self.month.map(|m| format!("{m:02}"))
    }

    /// 日セグメントのゼロ埋め表示文字列（未入力は `None`）。
    #[must_use]
    pub fn day_text(&self) -> Option<String> {
        self.day.map(|d| format!("{d:02}"))
    }

    /// 日セグメントの構造的上限（年月の両方が判明していれば当該月の日数、
    /// そうでなければ広めの `31`）。
    fn day_max(&self) -> u8 {
        match (self.year, self.month) {
            (Some(y), Some(m)) => days_in_month(y, m).unwrap_or(31),
            _ => 31,
        }
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(disabled, self.is_invalid(), attrs, children)
    }

    /// [`label`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn label<'a>(
        &self,
        disabled: bool,
        control_id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        label(disabled, self.is_invalid(), control_id, attrs, children)
    }

    /// [`control`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        control(disabled, self.is_invalid(), attrs, children)
    }

    /// [`segment_group`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn segment_group<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        segment_group(disabled, self.is_invalid(), attrs, children)
    }

    /// [`segment`] へ現在の状態を注入する利便メソッド。`readonly` は
    /// 呼び出し側が全体設定として渡す（[`crate::number_input::input`] の
    /// `NumberInputFlags::readonly` と同型）。
    #[must_use]
    pub fn segment<'a>(
        &self,
        kind: DateSegment,
        disabled: bool,
        readonly: bool,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        let flags = DateSegmentFlags {
            disabled,
            invalid: self.is_invalid(),
            readonly,
        };
        match kind {
            DateSegment::Year => {
                segment(kind, self.year_text().as_deref(), "0", "9999", flags, attrs)
            }
            DateSegment::Month => {
                segment(kind, self.month_text().as_deref(), "1", "12", flags, attrs)
            }
            DateSegment::Day => {
                let max = self.day_max().to_string();
                segment(kind, self.day_text().as_deref(), "1", &max, flags, attrs)
            }
        }
    }

    /// [`hidden_input`] へ現在の確定値を注入する利便メソッド（invalid の
    /// ときは空値、モジュール doc「セキュリティ不変条件」参照）。
    #[must_use]
    pub fn hidden_input<'a>(
        &self,
        name: &'a str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        let value = self.value().map(|d| d.to_iso_string()).unwrap_or_default();
        hidden_input(name, &value, disabled, attrs)
    }
}

impl Component for DateInput {
    type Action = DateInputAction;

    fn update(&mut self, action: DateInputAction) {
        match action {
            DateInputAction::Increment => {
                let Some(kind) = self.focused else { return };
                match kind {
                    DateSegment::Year => {
                        let base = self.year.unwrap_or(0);
                        self.year = Some(clamp_year(base.saturating_add(1)));
                    }
                    DateSegment::Month => {
                        let base = i32::from(self.month.unwrap_or(1));
                        self.month = Some(clamp_month(base.saturating_add(1)));
                    }
                    DateSegment::Day => {
                        let base = i32::from(self.day.unwrap_or(1));
                        let max = i32::from(self.day_max());
                        self.day = Some(base.saturating_add(1).min(max).max(1) as u8);
                    }
                }
            }
            DateInputAction::Decrement => {
                let Some(kind) = self.focused else { return };
                match kind {
                    DateSegment::Year => {
                        let base = self.year.unwrap_or(9999);
                        self.year = Some(clamp_year(base.saturating_sub(1)));
                    }
                    DateSegment::Month => {
                        let base = i32::from(self.month.unwrap_or(12));
                        self.month = Some(clamp_month(base.saturating_sub(1)));
                    }
                    DateSegment::Day => {
                        let base = i32::from(self.day.unwrap_or(self.day_max()));
                        self.day = Some(base.saturating_sub(1).max(1) as u8);
                    }
                }
            }
            DateInputAction::Focus(kind) => {
                self.focused = Some(kind);
            }
            DateInputAction::SetSegment(kind, value) => match kind {
                DateSegment::Year => self.year = Some(clamp_year(value)),
                DateSegment::Month => self.month = Some(clamp_month(value)),
                DateSegment::Day => {
                    let max = i32::from(self.day_max());
                    self.day = Some(value.clamp(1, max) as u8);
                }
            },
            DateInputAction::Set(date) => {
                self.year = Some(date.year());
                self.month = Some(date.month());
                self.day = Some(date.day());
            }
            DateInputAction::Clear => {
                self.year = None;
                self.month = None;
                self.day = None;
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root >
    /// control > segment-group。`name` を要する [`hidden_input`] は含めない、
    /// [`crate::number_input::NumberInput::view`] と同型の判断）。
    fn view(&self) -> Node {
        self.root(
            false,
            Vec::new(),
            vec![self.control(
                false,
                Vec::new(),
                vec![self.segment_group(false, Vec::new(), Vec::new())],
            )],
        )
    }

    /// `"increment"`/`"decrement"`: payload 不使用。`"focus"`: payload は
    /// [`DateSegment::parse`] で厳密パース。`"set-segment"`: payload は
    /// `"<kind>:<value>"` 形式（例 `"year:2026"`）でパースし、未知
    /// kind・非整数値は `None`（fail-closed、dispatch は no-op）。`"set"`:
    /// payload を [`PlainDate::parse_iso`] で厳密検証。`"clear"`: payload
    /// 不使用。
    fn decode_action(name: &str, payload: &str) -> Option<DateInputAction> {
        match name {
            "increment" => Some(DateInputAction::Increment),
            "decrement" => Some(DateInputAction::Decrement),
            "focus" => DateSegment::parse(payload).map(DateInputAction::Focus),
            "set-segment" => {
                let (kind_s, value_s) = payload.split_once(':')?;
                let kind = DateSegment::parse(kind_s)?;
                let value: i32 = value_s.parse().ok()?;
                Some(DateInputAction::SetSegment(kind, value))
            }
            "set" => PlainDate::parse_iso(payload).ok().map(DateInputAction::Set),
            "clear" => Some(DateInputAction::Clear),
            _ => None,
        }
    }
}

impl Hydrate for DateInput {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let year_s = self
            .year
            .map(|y| y.to_string())
            .unwrap_or_else(|| Self::HYDRATE_NONE.to_string());
        let month_s = self
            .month
            .map(|m| m.to_string())
            .unwrap_or_else(|| Self::HYDRATE_NONE.to_string());
        let day_s = self
            .day
            .map(|d| d.to_string())
            .unwrap_or_else(|| Self::HYDRATE_NONE.to_string());
        let min_s = self
            .min
            .map(|d| d.to_iso_string())
            .unwrap_or_else(|| Self::HYDRATE_NONE.to_string());
        let max_s = self
            .max
            .map(|d| d.to_iso_string())
            .unwrap_or_else(|| Self::HYDRATE_NONE.to_string());
        vec![
            (format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_YEAR), year_s),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MONTH),
                month_s,
            ),
            (format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_DAY), day_s),
            (format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MIN), min_s),
            (format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MAX), max_s),
        ]
    }

    /// クライアント改ざん入力として扱う。構造的範囲外・パース不能は
    /// [`HydrateError::InvalidValue`]（panic しない）。実在しない日付
    /// （2/30 等）は構造的には妥当な 3 整数のためそのまま受理する
    /// （モジュール doc「hydration 契約」参照）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name))
        };

        let parse_segment =
            |field: &str, raw: &str, lo: i64, hi: i64| -> Result<Option<i32>, HydrateError> {
                if raw == Self::HYDRATE_NONE {
                    return Ok(None);
                }
                let attr_name = format!("{HYDRATE_ATTR_PREFIX}{field}");
                let value: i64 = raw.parse().map_err(|_| HydrateError::InvalidValue {
                    attr: attr_name.clone(),
                    reason: "expected an integer or \"none\"".to_string(),
                })?;
                if value < lo || value > hi {
                    return Err(HydrateError::InvalidValue {
                        attr: attr_name,
                        reason: format!("expected a value within {lo}..={hi}"),
                    });
                }
                Ok(Some(value as i32))
            };

        let year_raw = find(Self::FIELD_YEAR)?;
        let month_raw = find(Self::FIELD_MONTH)?;
        let day_raw = find(Self::FIELD_DAY)?;
        let min_raw = find(Self::FIELD_MIN)?;
        let max_raw = find(Self::FIELD_MAX)?;

        let year = parse_segment(Self::FIELD_YEAR, year_raw, 0, 9999)?;
        let month = parse_segment(Self::FIELD_MONTH, month_raw, 1, 12)?.map(|m| m as u8);
        let day = parse_segment(Self::FIELD_DAY, day_raw, 1, 31)?.map(|d| d as u8);

        let parse_date = |field: &str, raw: &str| -> Result<Option<PlainDate>, HydrateError> {
            if raw == Self::HYDRATE_NONE {
                return Ok(None);
            }
            let attr_name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            PlainDate::parse_iso(raw)
                .map(Some)
                .map_err(|_: DateError| HydrateError::InvalidValue {
                    attr: attr_name,
                    reason: "expected a strict YYYY-MM-DD date or \"none\"".to_string(),
                })
        };

        let min = parse_date(Self::FIELD_MIN, min_raw)?;
        let max = parse_date(Self::FIELD_MAX, max_raw)?;

        if let (Some(min_v), Some(max_v)) = (min, max) {
            if min_v > max_v {
                return Err(HydrateError::InvalidValue {
                    attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MIN),
                    reason: "expected min <= max".to_string(),
                });
            }
        }

        Ok(Self {
            year,
            month,
            day,
            min,
            max,
            // フォーカス位置は ephemeral な DOM 状態のため運ばない
            // （モジュール doc 参照）。復元直後は常に未設定。
            focused: None,
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
        let html = render(&root(false, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="date-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-invalid"));
    }

    #[test]
    fn root_disabled_invalid_true_adds_data_attrs() {
        let html = render(&root(true, true, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-invalid="""#));
    }

    #[test]
    fn label_outputs_for_when_control_id_given() {
        let html = render(&label(
            false,
            false,
            Some("dob"),
            vec![],
            vec![text("Date of birth")],
        ));
        assert!(html.contains(r#"data-scope="date-input""#));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains(r#"for="dob""#));
    }

    #[test]
    fn control_and_segment_group_output_scope_and_part() {
        let html = render(&control(false, false, vec![], vec![]));
        assert!(html.contains(r#"data-part="control""#));
        let html = render(&segment_group(false, false, vec![], vec![]));
        assert!(html.contains(r#"data-part="segment-group""#));
    }

    #[test]
    fn segment_outputs_role_spinbutton_and_valuemin_max() {
        let html = render(&segment(
            DateSegment::Year,
            None,
            "0",
            "9999",
            DateSegmentFlags::default(),
            vec![],
        ));
        assert!(html.contains(r#"data-scope="date-input""#));
        assert!(html.contains(r#"data-part="segment""#));
        assert!(html.contains(r#"role="spinbutton""#));
        assert!(html.contains(r#"inputmode="numeric""#));
        assert!(html.contains(r#"aria-label="Year""#));
        assert!(html.contains(r#"aria-valuemin="0""#));
        assert!(html.contains(r#"aria-valuemax="9999""#));
        assert!(!html.contains("aria-valuenow"));
        assert!(html.contains(r#"data-placeholder="""#));
        assert!(html.contains("yyyy"));
    }

    #[test]
    fn segment_outputs_valuenow_and_no_placeholder_when_some() {
        let html = render(&segment(
            DateSegment::Month,
            Some("07"),
            "1",
            "12",
            DateSegmentFlags::default(),
            vec![],
        ));
        assert!(html.contains(r#"aria-valuenow="07""#));
        assert!(!html.contains("data-placeholder"));
        assert!(html.contains("07"));
        assert!(html.contains(r#"aria-label="Month""#));
    }

    #[test]
    fn segment_disabled_omits_tabindex_and_adds_data_disabled() {
        let html = render(&segment(
            DateSegment::Day,
            None,
            "1",
            "31",
            DateSegmentFlags {
                disabled: true,
                ..DateSegmentFlags::default()
            },
            vec![],
        ));
        assert!(!html.contains("tabindex"));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn segment_enabled_has_tabindex_zero() {
        let html = render(&segment(
            DateSegment::Day,
            None,
            "1",
            "31",
            DateSegmentFlags::default(),
            vec![],
        ));
        assert!(html.contains(r#"tabindex="0""#));
    }

    #[test]
    fn segment_invalid_adds_aria_invalid_and_data_invalid() {
        let html = render(&segment(
            DateSegment::Day,
            Some("30"),
            "1",
            "31",
            DateSegmentFlags {
                invalid: true,
                ..DateSegmentFlags::default()
            },
            vec![],
        ));
        assert!(html.contains(r#"aria-invalid="true""#));
        assert!(html.contains(r#"data-invalid="""#));
    }

    #[test]
    fn segment_readonly_adds_data_readonly() {
        let html = render(&segment(
            DateSegment::Day,
            None,
            "1",
            "31",
            DateSegmentFlags {
                readonly: true,
                ..DateSegmentFlags::default()
            },
            vec![],
        ));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn hidden_input_outputs_type_hidden_name_and_value() {
        let html = render(&hidden_input("dob", "2026-07-22", false, vec![]));
        assert!(html.contains(r#"data-part="hidden-input""#));
        assert!(html.contains(r#"type="hidden""#));
        assert!(html.contains(r#"name="dob""#));
        assert!(html.contains(r#"value="2026-07-22""#));
    }

    #[test]
    fn hidden_input_disabled_true_outputs_native_disabled() {
        let html = render(&hidden_input("dob", "", true, vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            false,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="date-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- DateSegment ---

    #[test]
    fn segment_as_str_and_parse_round_trip() {
        for kind in [DateSegment::Year, DateSegment::Month, DateSegment::Day] {
            assert_eq!(DateSegment::parse(kind.as_str()), Some(kind));
        }
        assert_eq!(DateSegment::parse("unknown"), None);
        assert_eq!(DateSegment::parse("<script>"), None);
    }

    // --- 正規化・fail-closed 構築 ---

    #[test]
    fn new_clamps_out_of_range_segments() {
        let d = DateInput::new(Some(20000), Some(99), Some(200), None, None);
        assert_eq!(d.year(), Some(9999));
        assert_eq!(d.month(), Some(12));
        assert_eq!(d.day(), Some(31));

        let d = DateInput::new(Some(-5), Some(0), Some(0), None, None);
        assert_eq!(d.year(), Some(0));
        assert_eq!(d.month(), Some(1));
        assert_eq!(d.day(), Some(1));
    }

    #[test]
    fn new_swaps_min_max_when_reversed() {
        let min = PlainDate::new(2026, 1, 1).unwrap();
        let max = PlainDate::new(2026, 12, 31).unwrap();
        let d = DateInput::new(None, None, None, Some(max), Some(min));
        assert_eq!(d.min(), Some(min));
        assert_eq!(d.max(), Some(max));
    }

    #[test]
    fn default_is_all_none() {
        let d = DateInput::default();
        assert_eq!(d.year(), None);
        assert_eq!(d.month(), None);
        assert_eq!(d.day(), None);
        assert!(!d.is_complete());
        assert!(!d.is_invalid());
        assert_eq!(d.value(), None);
    }

    // --- fail-closed 日付検証（受け入れ条件） ---

    #[test]
    fn value_is_none_for_nonexistent_date_but_segments_are_kept() {
        // 2024-02-30 は存在しない日付。
        let d = DateInput::new(Some(2024), Some(2), Some(30), None, None);
        assert_eq!(d.value(), None);
        assert!(d.is_invalid());
        // セグメント自体の値は破棄されない（モジュール doc 参照）。
        assert_eq!(d.year(), Some(2024));
        assert_eq!(d.month(), Some(2));
        assert_eq!(d.day(), Some(30));
    }

    #[test]
    fn value_is_some_for_valid_date() {
        let d = DateInput::new(Some(2026), Some(7), Some(22), None, None);
        assert_eq!(d.value(), Some(PlainDate::new(2026, 7, 22).unwrap()));
        assert!(!d.is_invalid());
    }

    #[test]
    fn is_invalid_false_while_incomplete() {
        let d = DateInput::new(Some(2026), None, None, None, None);
        assert!(!d.is_complete());
        assert!(!d.is_invalid());
    }

    #[test]
    fn is_invalid_true_when_out_of_min_max_range() {
        let min = PlainDate::new(2026, 1, 1).unwrap();
        let max = PlainDate::new(2026, 12, 31).unwrap();
        let d = DateInput::new(Some(2025), Some(12), Some(31), Some(min), Some(max));
        assert!(d.is_invalid());
        let d = DateInput::new(Some(2027), Some(1), Some(1), Some(min), Some(max));
        assert!(d.is_invalid());
        let d = DateInput::new(Some(2026), Some(6), Some(15), Some(min), Some(max));
        assert!(!d.is_invalid());
    }

    // --- dispatch 統合 ---

    #[test]
    fn increment_decrement_are_no_op_without_focus() {
        let mut d = DateInput::default();
        assert!(dispatch(&mut d, "increment", ""));
        assert_eq!(d.year(), None);
        assert!(dispatch(&mut d, "decrement", ""));
        assert_eq!(d.year(), None);
    }

    #[test]
    fn focus_then_increment_updates_focused_segment_only() {
        let mut d = DateInput::default();
        assert!(dispatch(&mut d, "focus", "month"));
        assert_eq!(d.focused(), Some(DateSegment::Month));
        assert!(dispatch(&mut d, "increment", ""));
        assert_eq!(d.month(), Some(2));
        assert_eq!(d.year(), None);
        assert_eq!(d.day(), None);
    }

    #[test]
    fn increment_clamps_at_segment_boundaries() {
        let mut d = DateInput::new(Some(9999), Some(12), Some(31), None, None);
        dispatch(&mut d, "focus", "year");
        dispatch(&mut d, "increment", "");
        assert_eq!(d.year(), Some(9999));
        dispatch(&mut d, "focus", "month");
        dispatch(&mut d, "increment", "");
        assert_eq!(d.month(), Some(12));
        dispatch(&mut d, "focus", "day");
        dispatch(&mut d, "increment", "");
        assert_eq!(d.day(), Some(31));
    }

    #[test]
    fn decrement_clamps_at_segment_lower_bound() {
        let mut d = DateInput::new(Some(0), Some(1), Some(1), None, None);
        dispatch(&mut d, "focus", "year");
        dispatch(&mut d, "decrement", "");
        assert_eq!(d.year(), Some(0));
        dispatch(&mut d, "focus", "month");
        dispatch(&mut d, "decrement", "");
        assert_eq!(d.month(), Some(1));
        dispatch(&mut d, "focus", "day");
        dispatch(&mut d, "decrement", "");
        assert_eq!(d.day(), Some(1));
    }

    #[test]
    fn day_increment_respects_days_in_month() {
        // 2024 年 2 月はうるう年で 29 日まで。
        let mut d = DateInput::new(Some(2024), Some(2), Some(29), None, None);
        dispatch(&mut d, "focus", "day");
        dispatch(&mut d, "increment", "");
        assert_eq!(d.day(), Some(29));
    }

    #[test]
    fn focus_rejects_unknown_segment_as_no_op() {
        let mut d = DateInput::default();
        assert!(!dispatch(&mut d, "focus", "hour"));
        assert_eq!(d.focused(), None);
    }

    #[test]
    fn set_segment_updates_single_segment_and_clamps() {
        let mut d = DateInput::default();
        assert!(dispatch(&mut d, "set-segment", "year:2026"));
        assert_eq!(d.year(), Some(2026));
        assert!(dispatch(&mut d, "set-segment", "month:99"));
        assert_eq!(d.month(), Some(12));
    }

    #[test]
    fn set_segment_rejects_malformed_payload_as_no_op() {
        let mut d = DateInput::default();
        for bogus in ["year", "year:", "year:abc", "bogus:1", ":1", ""] {
            assert!(!dispatch(&mut d, "set-segment", bogus));
        }
        assert_eq!(d.year(), None);
    }

    #[test]
    fn set_parses_strict_iso_and_updates_all_segments() {
        let mut d = DateInput::default();
        assert!(dispatch(&mut d, "set", "2026-07-22"));
        assert_eq!(
            (d.year(), d.month(), d.day()),
            (Some(2026), Some(7), Some(22))
        );
    }

    #[test]
    fn set_rejects_invalid_iso_as_no_op() {
        let mut d = DateInput::default();
        for bogus in ["2024-02-30", "2026-13-01", "2026/07/22", "not-a-date", ""] {
            assert!(!dispatch(&mut d, "set", bogus));
        }
        assert_eq!(d.year(), None);
    }

    #[test]
    fn clear_resets_all_segments() {
        let mut d = DateInput::new(Some(2026), Some(7), Some(22), None, None);
        assert!(dispatch(&mut d, "clear", ""));
        assert_eq!((d.year(), d.month(), d.day()), (None, None, None));
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut d = DateInput::new(Some(2026), Some(7), Some(22), None, None);
        assert!(!dispatch(&mut d, "no_such_action", "x"));
        assert_eq!(d.year(), Some(2026));
    }

    // --- SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&DateInput::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- hydration 経路 ---

    #[test]
    fn hydration_round_trip_with_full_date() {
        let d = DateInput::new(Some(2026), Some(7), Some(22), None, None);
        let rendered = render(&render_for_hydration(&d));
        assert!(rendered.contains(r#"data-hydrate-year="2026""#));
        assert!(rendered.contains(r#"data-hydrate-month="7""#));
        assert!(rendered.contains(r#"data-hydrate-day="22""#));
        assert!(rendered.contains(r#"data-hydrate-min="none""#));
        assert!(rendered.contains(r#"data-hydrate-max="none""#));

        let restored = DateInput::from_hydration_attrs(&d.hydration_attrs()).unwrap();
        assert_eq!(restored, d);
    }

    #[test]
    fn hydration_round_trip_with_none_segments() {
        let d = DateInput::default();
        let restored = DateInput::from_hydration_attrs(&d.hydration_attrs()).unwrap();
        assert_eq!(restored, d);
    }

    #[test]
    fn hydration_round_trip_with_min_max() {
        let min = PlainDate::new(2026, 1, 1).unwrap();
        let max = PlainDate::new(2026, 12, 31).unwrap();
        let d = DateInput::new(Some(2026), Some(6), Some(1), Some(min), Some(max));
        let rendered = render(&render_for_hydration(&d));
        assert!(rendered.contains(r#"data-hydrate-min="2026-01-01""#));
        assert!(rendered.contains(r#"data-hydrate-max="2026-12-31""#));
        let restored = DateInput::from_hydration_attrs(&d.hydration_attrs()).unwrap();
        assert_eq!(restored, d);
    }

    /// 実在しない日付（2/30 等）は hydration でも受理し、そのまま復元する
    /// （モジュール doc「hydration 契約」参照。値そのものが破損しているわけ
    /// ではなく、UI 上「不正な入力」を可視化して訂正させる意図的な設計）。
    #[test]
    fn hydration_accepts_structurally_valid_but_nonexistent_date() {
        let attrs = vec![
            ("data-hydrate-year".to_string(), "2024".to_string()),
            ("data-hydrate-month".to_string(), "2".to_string()),
            ("data-hydrate-day".to_string(), "30".to_string()),
            ("data-hydrate-min".to_string(), "none".to_string()),
            ("data-hydrate-max".to_string(), "none".to_string()),
        ];
        let restored = DateInput::from_hydration_attrs(&attrs).unwrap();
        assert_eq!(restored.year(), Some(2024));
        assert_eq!(restored.day(), Some(30));
        assert!(restored.is_invalid());
        assert_eq!(restored.value(), None);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = DateInput::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-year".to_string())
        );
    }

    #[test]
    fn from_hydration_attrs_out_of_structural_range_does_not_panic() {
        let base = |year: &str, month: &str, day: &str| {
            vec![
                ("data-hydrate-year".to_string(), year.to_string()),
                ("data-hydrate-month".to_string(), month.to_string()),
                ("data-hydrate-day".to_string(), day.to_string()),
                ("data-hydrate-min".to_string(), "none".to_string()),
                ("data-hydrate-max".to_string(), "none".to_string()),
            ]
        };
        for attrs in [
            base("10000", "1", "1"),
            base("2026", "13", "1"),
            base("2026", "1", "32"),
            base("not-a-number", "1", "1"),
        ] {
            let err = DateInput::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    #[test]
    fn from_hydration_attrs_rejects_min_greater_than_max() {
        let attrs = vec![
            ("data-hydrate-year".to_string(), "none".to_string()),
            ("data-hydrate-month".to_string(), "none".to_string()),
            ("data-hydrate-day".to_string(), "none".to_string()),
            ("data-hydrate-min".to_string(), "2026-12-31".to_string()),
            ("data-hydrate-max".to_string(), "2026-01-01".to_string()),
        ];
        let err = DateInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_rejects_malformed_min_max_iso() {
        let attrs = vec![
            ("data-hydrate-year".to_string(), "none".to_string()),
            ("data-hydrate-month".to_string(), "none".to_string()),
            ("data-hydrate-day".to_string(), "none".to_string()),
            ("data-hydrate-min".to_string(), "2026/01/01".to_string()),
            ("data-hydrate-max".to_string(), "none".to_string()),
        ];
        let err = DateInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn hydration_does_not_carry_focused_state() {
        let mut d = DateInput::default();
        dispatch(&mut d, "focus", "day");
        let restored = DateInput::from_hydration_attrs(&d.hydration_attrs()).unwrap();
        assert_eq!(restored.focused(), None);
    }

    // --- XSS 回帰: name/attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn hidden_input_name_payload_is_escaped_on_render() {
        let html = render(&hidden_input(
            ATTR_BREAK_PAYLOAD,
            "2026-07-22",
            false,
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            false,
            false,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&label(
            false,
            false,
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn hydration_xss_payload_in_year_is_rejected_not_rendered() {
        let attrs = vec![
            (
                "data-hydrate-year".to_string(),
                "<script>alert(1)</script>".to_string(),
            ),
            ("data-hydrate-month".to_string(), "none".to_string()),
            ("data-hydrate-day".to_string(), "none".to_string()),
            ("data-hydrate-min".to_string(), "none".to_string()),
            ("data-hydrate-max".to_string(), "none".to_string()),
        ];
        let err = DateInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn dispatch_set_segment_xss_payload_is_no_op() {
        let mut d = DateInput::default();
        assert!(!dispatch(
            &mut d,
            "set-segment",
            "year:<script>alert(1)</script>"
        ));
        assert_eq!(d.year(), None);
    }
}

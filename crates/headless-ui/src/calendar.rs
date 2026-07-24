//! Calendar（月表示・日付選択）headless コンポーネント（イシュー #835、親 #832）。
//!
//! ark-ui/chakra-ui の Calendar・DatePicker（`.claude/skills/chakra-ui`
//! `date-time/calendar.md` 等）を参考に、Root / Heading / PrevTrigger /
//! NextTrigger / Table / TableHeader / TableRow / TableHeadCell / TableBody /
//! TableCell / DayTrigger の 11 anatomy パーツと、月表示・選択・範囲制約を
//! 持つ状態機械 [`Calendar`] を提供する。暦計算は依存イシュー #833
//! （[`crate::date`]）へ全委譲し、本モジュールは描画・状態遷移のみを担う。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数を直接呼んで組み立てる。CSR/hydration は
//! [`Calendar`]（[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"prev-month"`/`"next-month"`/`"select"`/`"clear-selection"`）で
//! 月表示・選択の状態遷移をする。[`crate::date_picker::DatePicker`]
//! （#835）が popover コンテンツとして本コンポーネントを合成する。
//! `fandhe-frontend-pre-styled-ui` が本モジュールを呼んでスタイル済み
//! Calendar を組み立てる想定である。
//!
//! # 決定性の不変条件（[`crate::date`] モジュール doc の契約を継承）
//!
//! - **現在時刻を一切取得しない**: `SystemTime`・`Instant`・`js_sys` 等の
//!   時刻取得 API を本モジュールに追加してはならない。「今日」は
//!   [`Calendar::new`] の `today` 引数として**呼び出し側が明示的に渡す**。
//!   同一入力から常に同一出力を返す決定性は `tests/calendar.rs` の機械検査
//!   （`include_str!` によるソース走査）で恒久的に強制する。
//! - **fail-closed**: `min > max` は [`Calendar::new`] が拒否する。範囲外・
//!   不正な選択操作は状態を変更しない（`panic!`/`unwrap()` を使わない、
//!   `.claude/rules/coding-rust.md`）。年 `0000`/`9999` 境界で月グリッドの
//!   前後月展開が [`DateError::OutOfRange`] を返す極端な境界では、
//!   [`Calendar::weeks`] が `Err` を返し、呼び出し側（[`table_body_from_grid`]
//!   経由）は空の `tbody` を描画するに留める（panic しない）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`disabled`/`id`）はすべて
//!   `&'static str` リテラルで固定しており、動的値が属性名スロットへ混入する
//!   経路はない（[`crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の
//!   既存不変条件をそのまま継承する）。
//! - 動的値（曜日ラベル・`aria-label`・呼び出し側 `attrs`/`children`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   [`crate::date::PlainDate::to_iso_string`] の出力（ASCII 数字とハイフン
//!   のみ）であっても、この既定エスケープ経路を省略しない
//!   （[`crate::date`] モジュール doc の契約）。`raw_html()` は使用せず、
//!   HTML 文字列を直接組み立てない。
//!
//! # out-of-scope（本イシュー #835 のスコープ外）
//!
//! - キーボードナビゲーション（矢印キーでの gridcell フォーカス移動・
//!   roving tabindex）の実 DOM 配線: wasm 配線イシューのスコープ。
//! - 範囲選択（range mode）・複数月表示（multi-month）・年/月ビュー切替。
//! - [`crate::date_picker::DatePicker`] との連携以外の DateInput（セグメント
//!   式入力）との配線: 別イシューのスコープ。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_current, aria_disabled, aria_labelledby, aria_selected, role, AriaCurrent};
use crate::data_attrs::data_disabled;
use crate::date::{
    days_in_month, month_grid, DateError, MonthGrid, PlainDate, Weekday, MAX_YEAR, MIN_YEAR,
};
use fandhe_frontend_core::{text, Node};
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Calendar の anatomy（`data-scope="calendar"`）。
const ANATOMY: Anatomy = anatomy("calendar");

/// Root パーツ（`div`）。
#[must_use]
pub fn root<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("root", "div", attrs, children)
}

/// Heading パーツ（`div`）。表示中の年月ラベル（テキストは呼び出し側が
/// `children` で渡す）。`id` が `Some` のとき [`table`] の `aria-labelledby`
/// 先として使う。
#[must_use]
pub fn heading<'a>(
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("heading", "div", merged, children)
}

/// PrevTrigger パーツ（`button`）。前月へ移動するトリガー。`disabled` が
/// `true`（範囲下限に到達）のときネイティブ `disabled` + `aria-disabled` +
/// `data-disabled` を付与する。フォーム内配置時の意図しない submit を
/// 防ぐため `type="button"` を固定で付与する。
#[must_use]
pub fn prev_trigger<'a>(
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    if disabled {
        merged.push(aria_disabled(true));
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("prev-trigger", "button", merged, children)
}

/// NextTrigger パーツ（`button`）。[`prev_trigger`] と対称（範囲上限到達時に
/// 無効化）。
#[must_use]
pub fn next_trigger<'a>(
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    if disabled {
        merged.push(aria_disabled(true));
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("next-trigger", "button", merged, children)
}

/// Table パーツ（`table`）。WAI-ARIA APG の grid パターンに従い
/// `role="grid"` を固定付与する。`labelledby` が `Some` のとき [`heading`]
/// の `id` と対で `aria-labelledby` 関連付けを成立させる。
#[must_use]
pub fn table<'a>(
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("grid")];
    if let Some(labelledby) = labelledby {
        merged.push(aria_labelledby(labelledby));
    }
    merged.extend(attrs);
    ANATOMY.part("table", "table", merged, children)
}

/// TableHeader パーツ（`thead`）。`data-part="table-header"`（ark-ui 準拠の
/// kebab-case）。
#[must_use]
pub fn table_header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("table-header", "thead", attrs, children)
}

/// TableRow パーツ（`tr`）。`role="row"` を固定付与する。曜日見出し行・
/// 日付行の両方で共用する。
#[must_use]
pub fn table_row<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("row")];
    merged.extend(attrs);
    ANATOMY.part("table-row", "tr", merged, children)
}

/// TableHeadCell パーツ（`th`）。`data-part="table-head-cell"`（ark-ui 準拠の
/// kebab-case）。`role="columnheader"` を固定付与する。曜日ラベル自体は
/// 呼び出し側が `children` で渡す（i18n 対応・既定エスケープ経由）。
#[must_use]
pub fn table_head_cell<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("columnheader")];
    merged.extend(attrs);
    ANATOMY.part("table-head-cell", "th", merged, children)
}

/// TableBody パーツ（`tbody`）。`data-part="table-body"`（ark-ui 準拠の
/// kebab-case）。
#[must_use]
pub fn table_body<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("table-body", "tbody", attrs, children)
}

/// TableCell パーツ（`td`）。`role="gridcell"` を固定付与する。`selected` を
/// `aria-selected` へ反映する（[`day_trigger`] とセットで選択状態を二重に
/// 表現する、WAI-ARIA grid パターンの慣行）。
#[must_use]
pub fn table_cell<'a>(selected: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("gridcell"), aria_selected(selected)];
    merged.extend(attrs);
    ANATOMY.part("table-cell", "td", merged, children)
}

/// DayTrigger パーツ（`button`）。1 個の日付セルを表す。
///
/// `aria-label` に ISO 8601 表記（[`PlainDate::to_iso_string`]）を固定付与
/// する。選択日には `data-selected` を、今日には `data-today` +
/// `aria-current="date"`（[`AriaCurrent::Date`]）を、表示月外の日付には
/// `data-outside-month` を、min/max 範囲外には `data-disabled` +
/// ネイティブ `disabled` + `aria-disabled` を付与する。フォーム内配置時の
/// 意図しない submit を防ぐため `type="button"` を固定で付与する。
#[must_use]
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
pub fn day_trigger<'a>(
    date: PlainDate,
    selected: bool,
    today: bool,
    outside_month: bool,
    disabled: bool,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    // `aria-label`（ISO 表記）は呼び出しスコープで所有する `String` のため、
    // `el` へ渡す直前に借用参照へ揃えて 1 つの `&str` Vec として組み立てる
    // （動的値は依然として `render()` の既定エスケープを経由する）。
    let iso = date.to_iso_string();
    let mut merged: Vec<(&str, &str)> = vec![("type", "button"), ("aria-label", &iso)];
    if selected {
        merged.push(("data-selected", ""));
    }
    if today {
        merged.push(("data-today", ""));
        merged.push(aria_current(AriaCurrent::Date));
    }
    if outside_month {
        merged.push(("data-outside-month", ""));
    }
    if disabled {
        merged.push(("data-disabled", ""));
        merged.push(("disabled", ""));
        merged.push(aria_disabled(true));
    }
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("day-trigger", "button", merged, children)
}

/// [`Calendar`] に対する型付きアクション。
///
/// WASM 境界の文字列 dispatch（`name`/`payload`）とは
/// [`Calendar::decode_action`] で接続する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarAction {
    /// 表示月を 1 月前へ移動する（範囲下限と全く交差しない場合は無移動）。
    PrevMonth,
    /// 表示月を 1 月後へ移動する（範囲上限と全く交差しない場合は無移動）。
    NextMonth,
    /// 指定した日付を選択する（min/max 範囲外なら無視、fail-closed）。
    Select(PlainDate),
    /// 選択を解除する。
    ClearSelection,
}

/// 月表示・選択・範囲制約を持つ Calendar の状態機械。
///
/// 「今日」（`today`）はコンストラクタで呼び出し側が明示的に渡す必須
/// フィールドであり、本型・本モジュールのどこからも現在時刻 API を呼ばない
/// （モジュール doc §決定性の不変条件参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Calendar {
    view_year: i32,
    view_month: u8,
    selected: Option<PlainDate>,
    today: PlainDate,
    min: Option<PlainDate>,
    max: Option<PlainDate>,
    week_start: Weekday,
}

impl Calendar {
    /// 表示年月・選択・範囲制約・週開始曜日を指定して構築する。
    ///
    /// # Errors
    ///
    /// - `view_year` が `0000..=9999`（[`PlainDate::new`] と同一のサポート
    ///   範囲）の範囲外なら [`DateError::OutOfRange`]。`view_year()`/モジュール
    ///   doc は本コンストラクタを経由した値のみがこの範囲であることを前提と
    ///   する（ハイドレーション経路で復元される `view_year` を含む）。
    /// - `view_month` が `1..=12` の範囲外なら [`DateError::InvalidDate`]。
    /// - `min`/`max` がともに `Some` で `min > max` なら
    ///   [`DateError::InvalidDate`]（範囲制約として矛盾するため fail-closed
    ///   に拒否する）。
    pub fn new(
        view_year: i32,
        view_month: u8,
        today: PlainDate,
        selected: Option<PlainDate>,
        min: Option<PlainDate>,
        max: Option<PlainDate>,
        week_start: Weekday,
    ) -> Result<Self, DateError> {
        // view_year の範囲検証（PlainDate::new と同一の 0000..=9999 範囲、
        // Bugbot 指摘: この検証がなければ範囲外の view_year がハイドレーション
        // 経由で復元されうる）。
        if !(MIN_YEAR..=MAX_YEAR).contains(&view_year) {
            return Err(DateError::OutOfRange);
        }
        // view_month の妥当性検証（days_in_month が month の 1..=12 範囲を
        // 検証する）。
        let _ = days_in_month(view_year, view_month)?;
        if let (Some(min), Some(max)) = (min, max) {
            if min > max {
                return Err(DateError::InvalidDate);
            }
        }
        Ok(Self {
            view_year,
            view_month,
            selected,
            today,
            min,
            max,
            week_start,
        })
    }

    /// 表示中の年（`0000..=9999`）。
    #[must_use]
    pub const fn view_year(&self) -> i32 {
        self.view_year
    }

    /// 表示中の月（`1..=12`）。
    #[must_use]
    pub const fn view_month(&self) -> u8 {
        self.view_month
    }

    /// 現在選択中の日付。
    #[must_use]
    pub const fn selected(&self) -> Option<PlainDate> {
        self.selected
    }

    /// 「今日」として構築時に渡された日付。
    #[must_use]
    pub const fn today(&self) -> PlainDate {
        self.today
    }

    /// 指定した日付が min/max 範囲外かどうか。
    #[must_use]
    pub fn is_disabled(&self, date: PlainDate) -> bool {
        if let Some(min) = self.min {
            if date < min {
                return true;
            }
        }
        if let Some(max) = self.max {
            if date > max {
                return true;
            }
        }
        false
    }

    /// 表示中の月グリッド（[`crate::date::month_grid`] への委譲）。
    ///
    /// # Errors
    ///
    /// 前後月への展開が年 `0000`/`9999` 境界を逸脱する極端な場合に
    /// [`DateError::OutOfRange`] を返す（[`crate::date::month_grid`] の
    /// 契約をそのまま継承）。
    pub fn weeks(&self) -> Result<MonthGrid, DateError> {
        month_grid(self.view_year, self.view_month, self.week_start)
    }

    /// 前月への移動が可能か（範囲下限と交差しなくなる直前かどうか）。
    #[must_use]
    pub fn can_go_prev(&self) -> bool {
        self.would_move(-1)
    }

    /// 翌月への移動が可能か。
    #[must_use]
    pub fn can_go_next(&self) -> bool {
        self.would_move(1)
    }

    /// `delta`（月単位、`-1`/`1`）移動した場合の表示年月を返す。範囲外・
    /// min/max と全く交差しない場合は `None`（fail-closed、無移動）。
    fn resolve_move(&self, delta: i32) -> Option<(i32, u8)> {
        let mut y = self.view_year;
        let mut m = i32::from(self.view_month) + delta;
        while m < 1 {
            m += 12;
            y -= 1;
        }
        while m > 12 {
            m -= 12;
            y += 1;
        }
        if !(0..=9999).contains(&y) {
            return None;
        }
        let m_u8 = u8::try_from(m).ok()?;
        let last_day_number = days_in_month(y, m_u8).ok()?;
        let first = PlainDate::new(y, m_u8, 1).ok()?;
        let last = PlainDate::new(y, m_u8, last_day_number).ok()?;
        if let Some(max) = self.max {
            if first > max {
                return None;
            }
        }
        if let Some(min) = self.min {
            if last < min {
                return None;
            }
        }
        Some((y, m_u8))
    }

    fn would_move(&self, delta: i32) -> bool {
        self.resolve_move(delta).is_some()
    }

    fn move_month(&mut self, delta: i32) {
        if let Some((y, m)) = self.resolve_move(delta) {
            self.view_year = y;
            self.view_month = m;
        }
    }

    /// [`root`] の利便メソッド。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        root(attrs, children)
    }

    /// [`prev_trigger`] へ現在の範囲端到達有無を注入する利便メソッド。
    #[must_use]
    pub fn prev_trigger<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        prev_trigger(!self.can_go_prev(), attrs, children)
    }

    /// [`next_trigger`] へ現在の範囲端到達有無を注入する利便メソッド。
    #[must_use]
    pub fn next_trigger<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        next_trigger(!self.can_go_next(), attrs, children)
    }

    /// 現在の月グリッドから `tbody` を組み立てる。[`weeks`] が `Err` の
    /// （年 `0000`/`9999` 境界の極端な場合）は panic せず空の `tbody` を
    /// 返す（モジュール doc §決定性の不変条件参照）。
    #[must_use]
    pub fn table_body_from_grid<'a>(&self, attrs: Vec<(&'a str, &'a str)>) -> Node {
        let Ok(grid) = self.weeks() else {
            return table_body(attrs, Vec::new());
        };
        let rows: Vec<Node> = grid
            .weeks()
            .iter()
            .map(|week| {
                let cells: Vec<Node> = week
                    .iter()
                    .map(|date| {
                        let date = *date;
                        let is_selected = self.selected == Some(date);
                        let is_today = date == self.today;
                        let is_outside =
                            date.month() != self.view_month || date.year() != self.view_year;
                        let disabled = self.is_disabled(date);
                        let day_label = date.day().to_string();
                        table_cell(
                            is_selected,
                            Vec::new(),
                            vec![day_trigger(
                                date,
                                is_selected,
                                is_today,
                                is_outside,
                                disabled,
                                None,
                                Vec::new(),
                                vec![text(&day_label)],
                            )],
                        )
                    })
                    .collect();
                table_row(Vec::new(), cells)
            })
            .collect();
        table_body(attrs, rows)
    }
}

impl Component for Calendar {
    type Action = CalendarAction;

    fn update(&mut self, action: CalendarAction) {
        match action {
            CalendarAction::PrevMonth => self.move_month(-1),
            CalendarAction::NextMonth => self.move_month(1),
            CalendarAction::Select(date) => {
                if !self.is_disabled(date) {
                    self.selected = Some(date);
                }
            }
            CalendarAction::ClearSelection => self.selected = None,
        }
    }

    /// 共通契約（root のみ）を表す最小正準ビュー。実際の UI 構築は
    /// パーツ関数群・[`table_body_from_grid`] を呼び出し側が組み合わせる
    /// （[`crate::select::Select`] と同じ位置付け）。
    fn view(&self) -> Node {
        self.root(Vec::new(), Vec::new())
    }

    fn decode_action(name: &str, payload: &str) -> Option<CalendarAction> {
        match name {
            "prev-month" => Some(CalendarAction::PrevMonth),
            "next-month" => Some(CalendarAction::NextMonth),
            "select" => PlainDate::parse_iso(payload)
                .ok()
                .map(CalendarAction::Select),
            "clear-selection" => Some(CalendarAction::ClearSelection),
            _ => None,
        }
    }
}

/// hydration 属性名のフィールド部分（`docs/api/hydration-state-format.md`
/// の `<field>` 命名規約に従う）。
const FIELD_VIEW_YEAR: &str = "view-year";
const FIELD_VIEW_MONTH: &str = "view-month";
const FIELD_SELECTED: &str = "selected";
const FIELD_TODAY: &str = "today";
const FIELD_MIN: &str = "min";
const FIELD_MAX: &str = "max";
const FIELD_WEEK_START: &str = "week-start";

/// 空文字列を「値なし」（`Option::None`）の規約とする ISO 表記のエンコード。
fn encode_optional_date(date: Option<PlainDate>) -> String {
    date.map(|d| d.to_iso_string()).unwrap_or_default()
}

/// [`encode_optional_date`] の逆写像。空文字列は `Ok(None)`、それ以外は
/// [`PlainDate::parse_iso`] の fail-closed 検証を経由する。
fn decode_optional_date(attr: &str, raw: &str) -> Result<Option<PlainDate>, HydrateError> {
    if raw.is_empty() {
        return Ok(None);
    }
    PlainDate::parse_iso(raw)
        .map(Some)
        .map_err(|_| HydrateError::InvalidValue {
            attr: attr.to_string(),
            reason: "expected empty string or strict YYYY-MM-DD".to_string(),
        })
}

impl Hydrate for Calendar {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{FIELD_VIEW_YEAR}"),
                self.view_year.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{FIELD_VIEW_MONTH}"),
                self.view_month.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{FIELD_SELECTED}"),
                encode_optional_date(self.selected),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{FIELD_TODAY}"),
                self.today.to_iso_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{FIELD_MIN}"),
                encode_optional_date(self.min),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{FIELD_MAX}"),
                encode_optional_date(self.max),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{FIELD_WEEK_START}"),
                self.week_start.iso_number().to_string(),
            ),
        ]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name))
        };

        let view_year_attr = format!("{HYDRATE_ATTR_PREFIX}{FIELD_VIEW_YEAR}");
        let view_year: i32 =
            find(FIELD_VIEW_YEAR)?
                .parse()
                .map_err(|_| HydrateError::InvalidValue {
                    attr: view_year_attr.clone(),
                    reason: "expected an integer".to_string(),
                })?;

        let view_month_attr = format!("{HYDRATE_ATTR_PREFIX}{FIELD_VIEW_MONTH}");
        let view_month: u8 =
            find(FIELD_VIEW_MONTH)?
                .parse()
                .map_err(|_| HydrateError::InvalidValue {
                    attr: view_month_attr.clone(),
                    reason: "expected an integer in 1..=12".to_string(),
                })?;

        let selected_attr = format!("{HYDRATE_ATTR_PREFIX}{FIELD_SELECTED}");
        let selected = decode_optional_date(&selected_attr, find(FIELD_SELECTED)?)?;

        let today_attr = format!("{HYDRATE_ATTR_PREFIX}{FIELD_TODAY}");
        let today =
            PlainDate::parse_iso(find(FIELD_TODAY)?).map_err(|_| HydrateError::InvalidValue {
                attr: today_attr.clone(),
                reason: "expected strict YYYY-MM-DD".to_string(),
            })?;

        let min_attr = format!("{HYDRATE_ATTR_PREFIX}{FIELD_MIN}");
        let min = decode_optional_date(&min_attr, find(FIELD_MIN)?)?;

        let max_attr = format!("{HYDRATE_ATTR_PREFIX}{FIELD_MAX}");
        let max = decode_optional_date(&max_attr, find(FIELD_MAX)?)?;

        let week_start_attr = format!("{HYDRATE_ATTR_PREFIX}{FIELD_WEEK_START}");
        let week_start_raw: u8 =
            find(FIELD_WEEK_START)?
                .parse()
                .map_err(|_| HydrateError::InvalidValue {
                    attr: week_start_attr.clone(),
                    reason: "expected an integer in 1..=7".to_string(),
                })?;
        let week_start =
            Weekday::from_iso_number(week_start_raw).map_err(|_| HydrateError::InvalidValue {
                attr: week_start_attr,
                reason: "expected an integer in 1..=7".to_string(),
            })?;

        Calendar::new(view_year, view_month, today, selected, min, max, week_start).map_err(|_| {
            HydrateError::InvalidValue {
                attr: view_year_attr,
                reason: "invalid combination of view-year/view-month/min/max".to_string(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_interactive::dispatch;

    fn today_2026_07() -> PlainDate {
        PlainDate::new(2026, 7, 1).unwrap()
    }

    // --- 決定性: 同一入力から常に同一出力 ---

    #[test]
    fn month_grid_known_layout_2026_07_monday_start() {
        let cal =
            Calendar::new(2026, 7, today_2026_07(), None, None, None, Weekday::Monday).unwrap();
        let grid = cal.weeks().unwrap();
        // 2026-07-01 は水曜日。月曜始まりだと最初の週は 06-29(月)始まり。
        assert_eq!(grid.weeks()[0][0], PlainDate::new(2026, 6, 29).unwrap());
        assert_eq!(grid.weeks()[0][2], PlainDate::new(2026, 7, 1).unwrap());
        let last_week = grid.weeks().last().unwrap();
        assert_eq!(last_week[4], PlainDate::new(2026, 7, 31).unwrap());
    }

    #[test]
    fn month_grid_sunday_start_shifts_layout() {
        let cal =
            Calendar::new(2026, 7, today_2026_07(), None, None, None, Weekday::Sunday).unwrap();
        let grid = cal.weeks().unwrap();
        assert_eq!(grid.weeks()[0][0], PlainDate::new(2026, 6, 28).unwrap());
    }

    // --- 選択・範囲クランプ決定性 ---

    #[test]
    fn select_within_range_updates_selected() {
        let mut cal =
            Calendar::new(2026, 7, today_2026_07(), None, None, None, Weekday::Monday).unwrap();
        let d = PlainDate::new(2026, 7, 15).unwrap();
        cal.update(CalendarAction::Select(d));
        assert_eq!(cal.selected(), Some(d));
    }

    #[test]
    fn select_outside_min_max_range_is_ignored() {
        let min = PlainDate::new(2026, 7, 10).unwrap();
        let max = PlainDate::new(2026, 7, 20).unwrap();
        let mut cal = Calendar::new(
            2026,
            7,
            today_2026_07(),
            None,
            Some(min),
            Some(max),
            Weekday::Monday,
        )
        .unwrap();
        cal.update(CalendarAction::Select(PlainDate::new(2026, 7, 5).unwrap()));
        assert_eq!(cal.selected(), None);
        cal.update(CalendarAction::Select(PlainDate::new(2026, 7, 25).unwrap()));
        assert_eq!(cal.selected(), None);
        cal.update(CalendarAction::Select(PlainDate::new(2026, 7, 15).unwrap()));
        assert_eq!(cal.selected(), Some(PlainDate::new(2026, 7, 15).unwrap()));
    }

    #[test]
    fn clear_selection_resets_to_none() {
        let mut cal =
            Calendar::new(2026, 7, today_2026_07(), None, None, None, Weekday::Monday).unwrap();
        cal.update(CalendarAction::Select(PlainDate::new(2026, 7, 15).unwrap()));
        cal.update(CalendarAction::ClearSelection);
        assert_eq!(cal.selected(), None);
    }

    #[test]
    fn new_rejects_min_greater_than_max() {
        let min = PlainDate::new(2026, 7, 20).unwrap();
        let max = PlainDate::new(2026, 7, 10).unwrap();
        let err = Calendar::new(
            2026,
            7,
            today_2026_07(),
            None,
            Some(min),
            Some(max),
            Weekday::Monday,
        )
        .unwrap_err();
        assert_eq!(err, DateError::InvalidDate);
    }

    #[test]
    fn new_rejects_view_year_out_of_range() {
        // Bugbot 指摘（PR #865）: view_year は PlainDate::new と同一の
        // 0000..=9999 範囲で検証されるべきで、範囲外はハイドレーション経由で
        // 不正な view year が復元される事態を防ぐため拒否する。
        let err_negative =
            Calendar::new(-1, 7, today_2026_07(), None, None, None, Weekday::Monday).unwrap_err();
        assert_eq!(err_negative, DateError::OutOfRange);

        let err_too_large = Calendar::new(
            10_000,
            7,
            today_2026_07(),
            None,
            None,
            None,
            Weekday::Monday,
        )
        .unwrap_err();
        assert_eq!(err_too_large, DateError::OutOfRange);
    }

    #[test]
    fn prev_next_month_moves_view() {
        let mut cal =
            Calendar::new(2026, 7, today_2026_07(), None, None, None, Weekday::Monday).unwrap();
        cal.update(CalendarAction::NextMonth);
        assert_eq!((cal.view_year(), cal.view_month()), (2026, 8));
        cal.update(CalendarAction::PrevMonth);
        cal.update(CalendarAction::PrevMonth);
        assert_eq!((cal.view_year(), cal.view_month()), (2026, 6));
    }

    #[test]
    fn prev_next_month_crosses_year_boundary() {
        let mut cal =
            Calendar::new(2026, 1, today_2026_07(), None, None, None, Weekday::Monday).unwrap();
        cal.update(CalendarAction::PrevMonth);
        assert_eq!((cal.view_year(), cal.view_month()), (2025, 12));

        let mut cal2 =
            Calendar::new(2026, 12, today_2026_07(), None, None, None, Weekday::Monday).unwrap();
        cal2.update(CalendarAction::NextMonth);
        assert_eq!((cal2.view_year(), cal2.view_month()), (2027, 1));
    }

    #[test]
    fn prev_month_clamped_when_min_excludes_previous_month() {
        let min = PlainDate::new(2026, 7, 1).unwrap();
        let mut cal = Calendar::new(
            2026,
            7,
            today_2026_07(),
            None,
            Some(min),
            None,
            Weekday::Monday,
        )
        .unwrap();
        cal.update(CalendarAction::PrevMonth);
        assert_eq!(
            (cal.view_year(), cal.view_month()),
            (2026, 7),
            "min が当月全体を含むため前月へは移動しない"
        );
        assert!(!cal.can_go_prev());
    }

    #[test]
    fn next_month_clamped_when_max_excludes_next_month() {
        let max = PlainDate::new(2026, 7, 31).unwrap();
        let mut cal = Calendar::new(
            2026,
            7,
            today_2026_07(),
            None,
            None,
            Some(max),
            Weekday::Monday,
        )
        .unwrap();
        cal.update(CalendarAction::NextMonth);
        assert_eq!((cal.view_year(), cal.view_month()), (2026, 7));
        assert!(!cal.can_go_next());
    }

    #[test]
    fn year_0000_january_prev_month_is_no_op() {
        let cal = Calendar::new(
            0,
            1,
            PlainDate::new(0, 1, 1).unwrap(),
            None,
            None,
            None,
            Weekday::Monday,
        )
        .unwrap();
        assert!(!cal.can_go_prev());
        let mut cal_mut = cal;
        cal_mut.update(CalendarAction::PrevMonth);
        assert_eq!((cal_mut.view_year(), cal_mut.view_month()), (0, 1));
    }

    #[test]
    fn year_9999_december_next_month_is_no_op() {
        let cal = Calendar::new(
            9999,
            12,
            PlainDate::new(9999, 12, 31).unwrap(),
            None,
            None,
            None,
            Weekday::Monday,
        )
        .unwrap();
        assert!(!cal.can_go_next());
        let mut cal_mut = cal;
        cal_mut.update(CalendarAction::NextMonth);
        assert_eq!((cal_mut.view_year(), cal_mut.view_month()), (9999, 12));
    }

    #[test]
    fn table_body_from_grid_does_not_panic_at_year_boundary() {
        // 年 0000 1 月・日曜始まりは前月展開が year -1 へ抜けるため
        // month_grid が Err を返し得る境界（weeks() が Err のフォールバック
        // 経路を機械的に検証する）。
        let cal = Calendar::new(
            0,
            1,
            PlainDate::new(0, 1, 1).unwrap(),
            None,
            None,
            None,
            Weekday::Sunday,
        )
        .unwrap();
        let html = render(&cal.table_body_from_grid(Vec::new()));
        assert!(html.contains("table-body") || html.contains("tbody"));
    }

    // --- ARIA grid パターン ---

    #[test]
    fn table_has_role_grid_and_labelledby() {
        let html = render(&table(Some("cal-heading"), vec![], vec![]));
        assert!(html.contains(r#"role="grid""#));
        assert!(html.contains(r#"aria-labelledby="cal-heading""#));
    }

    #[test]
    fn table_row_has_role_row() {
        let html = render(&table_row(vec![], vec![]));
        assert!(html.contains(r#"role="row""#));
    }

    #[test]
    fn table_cell_has_role_gridcell_and_aria_selected() {
        let selected = render(&table_cell(true, vec![], vec![]));
        assert!(selected.contains(r#"role="gridcell""#));
        assert!(selected.contains(r#"aria-selected="true""#));

        let unselected = render(&table_cell(false, vec![], vec![]));
        assert!(unselected.contains(r#"aria-selected="false""#));
    }

    #[test]
    fn day_trigger_marks_selected_today_outside_disabled() {
        let d = PlainDate::new(2026, 7, 15).unwrap();
        let html = render(&day_trigger(
            d,
            true,
            true,
            true,
            true,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-label="2026-07-15""#));
        assert!(html.contains(r#"data-selected="""#));
        assert!(html.contains(r#"data-today="""#));
        assert!(html.contains(r#"aria-current="date""#));
        assert!(html.contains(r#"data-outside-month="""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn day_trigger_plain_omits_all_state_attrs() {
        let d = PlainDate::new(2026, 7, 15).unwrap();
        let html = render(&day_trigger(
            d,
            false,
            false,
            false,
            false,
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("data-selected"));
        assert!(!html.contains("data-today"));
        assert!(!html.contains("aria-current"));
        assert!(!html.contains("data-outside-month"));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("disabled"));
    }

    // --- dispatch 統合 ---

    #[test]
    fn dispatch_select_and_clear_selection() {
        let mut cal =
            Calendar::new(2026, 7, today_2026_07(), None, None, None, Weekday::Monday).unwrap();
        assert!(dispatch(&mut cal, "select", "2026-07-15"));
        assert_eq!(cal.selected(), Some(PlainDate::new(2026, 7, 15).unwrap()));
        assert!(dispatch(&mut cal, "clear-selection", ""));
        assert_eq!(cal.selected(), None);
    }

    #[test]
    fn dispatch_select_invalid_iso_payload_is_ignored() {
        let mut cal =
            Calendar::new(2026, 7, today_2026_07(), None, None, None, Weekday::Monday).unwrap();
        assert!(!dispatch(&mut cal, "select", "not-a-date"));
        assert_eq!(cal.selected(), None);
    }

    #[test]
    fn dispatch_prev_next_month() {
        let mut cal =
            Calendar::new(2026, 7, today_2026_07(), None, None, None, Weekday::Monday).unwrap();
        assert!(dispatch(&mut cal, "next-month", ""));
        assert_eq!((cal.view_year(), cal.view_month()), (2026, 8));
        assert!(dispatch(&mut cal, "prev-month", ""));
        assert_eq!((cal.view_year(), cal.view_month()), (2026, 7));
    }

    // --- hydration ---

    #[test]
    fn hydration_round_trip() {
        let mut cal = Calendar::new(
            2026,
            7,
            today_2026_07(),
            None,
            Some(PlainDate::new(2026, 1, 1).unwrap()),
            Some(PlainDate::new(2026, 12, 31).unwrap()),
            Weekday::Sunday,
        )
        .unwrap();
        cal.update(CalendarAction::Select(PlainDate::new(2026, 7, 15).unwrap()));
        let attrs = cal.hydration_attrs();
        let restored = Calendar::from_hydration_attrs(&attrs).unwrap();
        assert_eq!(restored, cal);
    }

    #[test]
    fn hydration_round_trip_without_selection_or_range() {
        let cal =
            Calendar::new(2026, 7, today_2026_07(), None, None, None, Weekday::Monday).unwrap();
        let attrs = cal.hydration_attrs();
        let restored = Calendar::from_hydration_attrs(&attrs).unwrap();
        assert_eq!(restored, cal);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Calendar::from_hydration_attrs(&[]).unwrap_err();
        assert!(matches!(err, HydrateError::MissingAttr(_)));
    }

    #[test]
    fn from_hydration_attrs_invalid_value_does_not_panic() {
        let bogus = vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{FIELD_VIEW_YEAR}"),
                "2026".to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{FIELD_VIEW_MONTH}"),
                "not-a-number".to_string(),
            ),
        ];
        let err = Calendar::from_hydration_attrs(&bogus).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰 ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn heading_children_text_is_escaped_on_render() {
        let html = render(&heading(
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(vec![("data-testid", ATTR_BREAK_PAYLOAD)], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn day_trigger_id_payload_is_escaped_on_render() {
        let d = PlainDate::new(2026, 7, 15).unwrap();
        let html = render(&day_trigger(
            d,
            false,
            false,
            false,
            false,
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="calendar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }
}

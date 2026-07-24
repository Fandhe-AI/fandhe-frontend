//! DatePicker（トリガー起点の日付選択オーバーレイ）headless コンポーネント
//! （イシュー #835、親 #832）。
//!
//! ark-ui/chakra-ui の DatePicker（`.claude/skills/ark-ui`
//! `date-time/date-picker.md` 等）を参考に、Root / Label / Control / Input /
//! Trigger / ClearTrigger / Positioner / Content の 8 anatomy パーツを提供
//! する。**positioner/content の開閉・配置は [`crate::popover`] と同一の
//! 基盤（[`crate::state::Disclosure`]）を再利用しており、独自のオーバーレイ
//! 機構を持たない**（[`crate::popover::Popover`] と並ぶ利用例）。[`content`]
//! 内部に [`crate::calendar::Calendar`] を合成して月表示・日付選択 UI を
//! 組み立てる想定である。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数を直接呼んで組み立てる。CSR/hydration は
//! [`DatePicker`]（[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"open"`/`"close"`/`"toggle"`/`"select"`/`"clear-selection"`/
//! `"prev-month"`/`"next-month"`）で開閉・月表示・選択の状態遷移をする。
//!
//! # DateInput（#834）との責務境界
//!
//! 本コンポーネントはセグメント式の DateInput に依存せず、ISO 8601
//! （`YYYY-MM-DD`）値を持つネイティブ `<input>` パーツ（[`input`]）だけで
//! 完結する。セグメント式 DateInput との連携強化は #834 側の作業である。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`disabled`/`id`/`value`）は
//!   すべて `&'static str` リテラルで固定しており、動的値が属性名スロットへ
//!   混入する経路はない。
//! - 動的値（`value`/`id`/`controls`/`labelledby`/呼び出し側
//!   `attrs`/`children`）は [`fandhe_frontend_core::render`] の既定エスケープ
//!   を必ず経由する。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - 文字列からの日付取り込みは [`crate::date::PlainDate::parse_iso`] の
//!   fail-closed（`Err` で状態不変）に限定する。
//!
//! # out-of-scope（本イシュー #835 のスコープ外）
//!
//! - フォーカストラップ・`closeOnEscape`・`closeOnInteractOutside`・portal:
//!   クライアントランタイム側の領域（[`crate::popover`] と同じ判断）。
//! - DateInput（#834）との配線・range mode・複数月表示。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_controls, aria_expanded, aria_haspopup, aria_labelledby, role, AriaPopup};
use crate::calendar::{Calendar, CalendarAction};
use crate::data_attrs::data_disabled;
use crate::date::{DateError, PlainDate};
use crate::state::{Disclosure, DisclosureAction, OpenState};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// DatePicker の anatomy（`data-scope="date-picker"`）。
const ANATOMY: Anatomy = anatomy("date-picker");

/// Root パーツ（`div`）。開閉状態を `data-*` へ反映する。
#[must_use]
pub fn root<'a>(state: OpenState, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![crate::data_attrs::data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。`id` が `Some` のとき [`trigger`]/[`content`] の
/// `labelledby` と対で `aria-labelledby` 関連付けを成立させる。
#[must_use]
pub fn label<'a>(id: Option<&'a str>, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("label", "label", merged, children)
}

/// Control パーツ（`div`）。[`input`]/[`trigger`]/[`clear_trigger`] をまとめる
/// コンテナ。開閉状態を `data-*` へ反映するのみの最小主義な装飾用パーツ。
#[must_use]
pub fn control<'a>(state: OpenState, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![crate::data_attrs::data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// Input パーツ（ネイティブ `input`）。
///
/// `value` は [`PlainDate::to_iso_string`] 由来の ISO 8601 表記のみを渡す
/// 契約とする（モジュール doc §DateInput との責務境界参照）。`type="text"`
/// を固定し、`disabled` はネイティブ `disabled` + `data-disabled` の両方へ
/// 反映する。
#[must_use]
pub fn input<'a>(
    value: Option<&'a str>,
    disabled: bool,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "text")];
    if let Some(value) = value {
        merged.push(("value", value));
    }
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(data_disabled(disabled));
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("input", "input", merged, Vec::new())
}

/// Trigger パーツ（`button`）。popover を開閉するトリガー
/// （[`crate::popover::trigger`] と同型、`aria-haspopup="dialog"` 固定）。
#[must_use]
pub fn trigger<'a>(
    state: OpenState,
    disabled: bool,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_haspopup(AriaPopup::Dialog),
        aria_expanded(state.is_open()),
        crate::data_attrs::data_state(state.as_data_state()),
    ];
    if let Some(id) = controls {
        merged.push(aria_controls(id));
    }
    merged.extend(data_disabled(disabled));
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// ClearTrigger パーツ（`button`）。[`crate::popover::close_trigger`] と同型。
#[must_use]
pub fn clear_trigger<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    merged.extend(attrs);
    ANATOMY.part("clear-trigger", "button", merged, children)
}

/// Positioner パーツ（`div`）。[`crate::popover::positioner`] と同型
/// （位置決めロジックは [`crate::positioning`] へ委譲、本関数は開閉に応じた
/// `hidden` のみを担う）。
#[must_use]
pub fn positioner<'a>(
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![crate::data_attrs::data_state(state.as_data_state())];
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("positioner", "div", merged, children)
}

/// Content パーツ（`div`）。`role="dialog"` を固定付与し、内部に
/// [`crate::calendar::Calendar`] のパーツ関数群を合成する想定
/// （[`crate::popover::content`] と同型）。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    id: Option<&'a str>,
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("dialog"),
        crate::data_attrs::data_state(state.as_data_state()),
    ];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(labelledby) = labelledby {
        merged.push(aria_labelledby(labelledby));
    }
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("content", "div", merged, children)
}

/// [`DatePicker`] に対する型付きアクション。
///
/// WASM 境界の文字列 dispatch（`name`/`payload`）とは
/// [`DatePicker::decode_action`] で接続する。[`crate::state::Disclosure`]
/// （popover 開閉）と [`crate::calendar::CalendarAction`]（月表示・選択）を
/// 合成する。`Select` は ark-ui の `closeOnSelect` 既定 `true` に準拠し、
/// 選択と同時に popover を閉じる（[`crate::select::Select`] と同じ判断）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatePickerAction {
    /// popover を開く。
    Open,
    /// popover を閉じる。
    Close,
    /// popover の開閉を反転する。
    Toggle,
    /// 表示月を 1 月前へ移動する。
    PrevMonth,
    /// 表示月を 1 月後へ移動する。
    NextMonth,
    /// 指定した日付を選択する（選択と同時に popover を閉じる）。
    Select(PlainDate),
    /// 選択を解除する（popover の開閉状態は変えない）。
    ClearSelection,
}

/// [`Disclosure`]（popover 開閉）+ [`Calendar`]（月表示・選択）を埋め込んだ
/// DatePicker の状態機械。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatePicker {
    disclosure: Disclosure,
    calendar: Calendar,
}

impl DatePicker {
    /// 指定した [`Calendar`] を埋め込み、popover は閉状態で構築する
    /// （SSR の状態なし初期描画に対応する既定値）。
    #[must_use]
    pub fn new(calendar: Calendar) -> Self {
        Self {
            disclosure: Disclosure::default(),
            calendar,
        }
    }

    /// 現在の popover 開閉状態。
    #[must_use]
    pub fn open_state(&self) -> OpenState {
        self.disclosure.state()
    }

    /// popover が開いているかどうか。
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.disclosure.state().is_open()
    }

    /// 埋め込み [`Calendar`] への参照。
    #[must_use]
    pub const fn calendar(&self) -> &Calendar {
        &self.calendar
    }

    /// 現在選択中の日付（[`Calendar::selected`] への委譲）。
    #[must_use]
    pub const fn selected(&self) -> Option<PlainDate> {
        self.calendar.selected()
    }

    /// [`root`] へ現在の開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        root(self.open_state(), attrs, children)
    }

    /// [`control`] へ現在の開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        control(self.open_state(), attrs, children)
    }

    /// [`trigger`] へ現在の開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        disabled: bool,
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(self.open_state(), disabled, controls, attrs, children)
    }

    /// [`positioner`] へ現在の開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn positioner<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        positioner(self.open_state(), attrs, children)
    }

    /// [`content`] へ現在の開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn content<'a>(
        &self,
        id: Option<&'a str>,
        labelledby: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(self.open_state(), id, labelledby, attrs, children)
    }
}

impl Component for DatePicker {
    type Action = DatePickerAction;

    fn update(&mut self, action: DatePickerAction) {
        match action {
            DatePickerAction::Open => self.disclosure.update(DisclosureAction::Open),
            DatePickerAction::Close => self.disclosure.update(DisclosureAction::Close),
            DatePickerAction::Toggle => self.disclosure.update(DisclosureAction::Toggle),
            DatePickerAction::PrevMonth => self.calendar.update(CalendarAction::PrevMonth),
            DatePickerAction::NextMonth => self.calendar.update(CalendarAction::NextMonth),
            DatePickerAction::Select(date) => {
                self.calendar.update(CalendarAction::Select(date));
                // ark-ui の closeOnSelect 既定 true に準拠する
                // （モジュール doc §DatePickerAction 参照）。
                self.disclosure.update(DisclosureAction::Close);
            }
            DatePickerAction::ClearSelection => {
                self.calendar.update(CalendarAction::ClearSelection);
            }
        }
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（[`crate::select::Select`] と同じ位置付け）。
    fn view(&self) -> Node {
        let state = self.open_state();
        self.root(
            Vec::new(),
            vec![
                trigger(state, false, None, Vec::new(), Vec::new()),
                positioner(
                    state,
                    Vec::new(),
                    vec![content(state, None, None, Vec::new(), Vec::new())],
                ),
            ],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<DatePickerAction> {
        match name {
            "open" => Some(DatePickerAction::Open),
            "close" => Some(DatePickerAction::Close),
            "toggle" => Some(DatePickerAction::Toggle),
            "prev-month" => Some(DatePickerAction::PrevMonth),
            "next-month" => Some(DatePickerAction::NextMonth),
            "select" => PlainDate::parse_iso(payload)
                .ok()
                .map(DatePickerAction::Select),
            "clear-selection" => Some(DatePickerAction::ClearSelection),
            _ => None,
        }
    }
}

impl Hydrate for DatePicker {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let mut attrs = self.disclosure.hydration_attrs();
        attrs.extend(self.calendar.hydration_attrs());
        attrs
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            disclosure: Disclosure::from_hydration_attrs(attrs)?,
            calendar: Calendar::from_hydration_attrs(attrs)?,
        })
    }
}

/// [`DatePicker::new`] で埋め込む [`Calendar`] を組み立てる際の共通エラー
/// 経路。呼び出し側（テスト・利用者コード）の利便のため
/// [`Calendar::new`] の [`DateError`] をそのまま再輸出する。
pub type DatePickerBuildError = DateError;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::Weekday;
    use fandhe_frontend_core::render;
    use fandhe_frontend_interactive::dispatch;

    fn sample_calendar() -> Calendar {
        Calendar::new(
            2026,
            7,
            PlainDate::new(2026, 7, 1).unwrap(),
            None,
            None,
            None,
            Weekday::Monday,
        )
        .unwrap()
    }

    #[test]
    fn default_is_closed_and_unselected() {
        let dp = DatePicker::new(sample_calendar());
        assert_eq!(dp.open_state(), OpenState::Closed);
        assert_eq!(dp.selected(), None);
    }

    #[test]
    fn dispatch_open_close_toggle() {
        let mut dp = DatePicker::new(sample_calendar());
        assert!(dispatch(&mut dp, "open", ""));
        assert!(dp.is_open());
        assert!(dispatch(&mut dp, "close", ""));
        assert!(!dp.is_open());
        assert!(dispatch(&mut dp, "toggle", ""));
        assert!(dp.is_open());
    }

    #[test]
    fn dispatch_select_updates_calendar_and_closes_popover() {
        let mut dp = DatePicker::new(sample_calendar());
        dispatch(&mut dp, "open", "");
        assert!(dp.is_open());

        assert!(dispatch(&mut dp, "select", "2026-07-15"));
        assert_eq!(dp.selected(), Some(PlainDate::new(2026, 7, 15).unwrap()));
        assert!(
            !dp.is_open(),
            "closeOnSelect: 選択と同時に popover を閉じる"
        );
    }

    #[test]
    fn dispatch_clear_selection_does_not_close_popover() {
        let mut dp = DatePicker::new(sample_calendar());
        dispatch(&mut dp, "select", "2026-07-15");
        dispatch(&mut dp, "open", "");
        assert!(dispatch(&mut dp, "clear-selection", ""));
        assert_eq!(dp.selected(), None);
        assert!(dp.is_open());
    }

    #[test]
    fn dispatch_prev_next_month_delegates_to_calendar() {
        let mut dp = DatePicker::new(sample_calendar());
        assert!(dispatch(&mut dp, "next-month", ""));
        assert_eq!(
            (dp.calendar().view_year(), dp.calendar().view_month()),
            (2026, 8)
        );
    }

    #[test]
    fn dispatch_select_invalid_iso_payload_is_ignored() {
        let mut dp = DatePicker::new(sample_calendar());
        assert!(!dispatch(&mut dp, "select", "not-a-date"));
        assert_eq!(dp.selected(), None);
    }

    // --- ARIA / anatomy ---

    #[test]
    fn trigger_has_haspopup_dialog_and_aria_expanded() {
        let html = render(&trigger(OpenState::Closed, false, None, vec![], vec![]));
        assert!(html.contains(r#"aria-haspopup="dialog""#));
        assert!(html.contains(r#"aria-expanded="false""#));
    }

    #[test]
    fn content_has_role_dialog_and_hidden_when_closed() {
        let closed = render(&content(OpenState::Closed, None, None, vec![], vec![]));
        assert!(closed.contains(r#"role="dialog""#));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&content(OpenState::Open, None, None, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn input_value_reflects_iso_string() {
        let html = render(&input(Some("2026-07-15"), false, None, vec![]));
        assert!(html.contains(r#"value="2026-07-15""#));
        assert!(html.contains(r#"type="text""#));
    }

    #[test]
    fn input_disabled_true_adds_native_and_data_disabled() {
        let html = render(&input(None, true, None, vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    // --- hydration ---

    #[test]
    fn hydration_round_trip() {
        let mut dp = DatePicker::new(sample_calendar());
        dispatch(&mut dp, "open", "");
        dispatch(&mut dp, "select", "2026-07-15");
        dispatch(&mut dp, "open", "");

        let attrs = dp.hydration_attrs();
        let restored = DatePicker::from_hydration_attrs(&attrs).unwrap();
        assert_eq!(restored, dp);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = DatePicker::from_hydration_attrs(&[]).unwrap_err();
        assert!(matches!(err, HydrateError::MissingAttr(_)));
    }

    // --- XSS 回帰 ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn trigger_controls_payload_is_escaped_on_render() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn input_value_payload_is_escaped_on_render() {
        let html = render(&input(Some(ATTR_BREAK_PAYLOAD), false, None, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            OpenState::Closed,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="date-picker""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }
}

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
//! # 参照突合（イシュー #1627、ark-ui/chakra-ui `date-picker` との対比）
//!
//! 一次情報は `.claude/skills/ark-ui`/`.claude/skills/chakra-ui` 配下の
//! date-picker 参照ファイルと、直近の同型 precedent（color-picker #1604・
//! combobox #1605）。差分の是正・意図的な非追随は以下のとおり:
//!
//! - **是正**: [`DatePickerProps`]（`disabled`/`readonly`/`invalid`/
//!   `required`）を新設し、root/label/control/input/trigger/clear-trigger
//!   の 6 パーツへ `data-disabled`/`data-invalid`/`data-readonly` を一律
//!   付与、label にのみ `data-required` を追加で付与する（[`crate::combobox::ComboboxProps`]
//!   と同型のパターン）。[`label`] に `for_`（ark `htmlFor` 準拠）を追加し、
//!   `id` が `Some` のとき [`content`] の `labelledby` と対で
//!   `aria-labelledby` 関連付け、`for_` が `Some` のとき [`input`] の `id`
//!   と対でネイティブ `label[for]` 関連付けを成立させる。[`input`] は
//!   `props.disabled`/`props.readonly`/`props.required` をそれぞれネイティブ
//!   `disabled`/`readonly`/`required` 存在属性へ反映し、`props.invalid` の
//!   ときのみ `aria-invalid="true"` を追加する（[`crate::combobox::input`]
//!   と同型）。呼び出し側 `attrs` からの状態系 `data-*` 上書きは
//!   [`drop_reserved`] が fail-closed に除去する。
//! - **意図的に追随しない**（理由付き）:
//!   - ark-ui の View/ViewControl/PrevTrigger/NextTrigger/ViewTrigger/
//!     RangeText/Table 系/TableCellTrigger/MonthSelect/YearSelect/
//!     PresetTrigger/WeekNumber\*/ValueText はグリッド系（Table 系相当）が
//!     [`content`] へ合成する [`crate::calendar::Calendar`] 側（11 パーツ）
//!     に既に存在し、年月ビュー切替・プリセット・週番号は
//!     [`crate::calendar`] のモジュール doc で明示的にスコープ外のため
//!     本イシューでも非追随を継続する。
//!   - `data-view`（ビュー切替非対応のため非追随）。
//!   - `data-placement`/`data-side`（[`crate::combobox`]/
//!     [`crate::color_picker`] と同様、JS ランタイムのレイアウト計測属性は
//!     `docs/policy/intentional-non-adoption.md` §3.25 規則 2 に従い非採用）。
//!   - `content` の `role="dialog"`/`trigger` の `aria-haspopup="dialog"` は
//!     WAI-ARIA APG「Date Picker Dialog」パターンと [`crate::popover`]
//!     基盤との整合を優先し現状維持する。
//! - **スコープ外**（`.claude/rules/out-of-scope-tracking.md` 対応）:
//!   - `fandhe-frontend-wasm-full` への `date-picker` scope 配線
//!     （trigger click・clear・Escape・外側クリック閉鎖が CSR で未動作。
//!     `MAPPING_TABLE`/`OverlayKind::from_scope` に `"date-picker"` が
//!     登録されていない）。
//!   - `readonly` 時の操作抑止（wasm-full 側の dispatch 拒否）。
//!
//! # DateInput（#834）との責務境界
//!
//! 本コンポーネントはセグメント式の DateInput に依存せず、ISO 8601
//! （`YYYY-MM-DD`）値を持つネイティブ `<input>` パーツ（[`input`]）だけで
//! 完結する。セグメント式 DateInput との連携強化は #834 側の作業である。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`disabled`/`readonly`/
//!   `required`/`id`/`value`/`for`）はすべて `&'static str` リテラルで
//!   固定しており、動的値が属性名スロットへ混入する経路はない。
//! - 動的値（`value`/`id`/`for_`/`controls`/`labelledby`/呼び出し側
//!   `attrs`/`children`）は [`fandhe_frontend_core::render`] の既定エスケープ
//!   を必ず経由する。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - 文字列からの日付取り込みは [`crate::date::PlainDate::parse_iso`] の
//!   fail-closed（`Err` で状態不変）に限定する。
//! - 呼び出し側 `attrs` による `data-scope`/`data-part`/状態系 `data-*`
//!   属性の上書きは [`Anatomy::part`] と [`drop_reserved`] が fail-closed に
//!   破棄する（[`crate::combobox`] と同型のパターン）。
//!
//! # out-of-scope（本イシュー #835 のスコープ外）
//!
//! - フォーカストラップ・`closeOnEscape`・`closeOnInteractOutside`・portal:
//!   クライアントランタイム側の領域（[`crate::popover`] と同じ判断）。
//! - DateInput（#834）との配線・range mode・複数月表示。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{
    aria_controls, aria_expanded, aria_haspopup, aria_invalid, aria_labelledby, role, AriaPopup,
};
use crate::calendar::{Calendar, CalendarAction};
use crate::data_attrs::{data_disabled, data_invalid, data_readonly, data_required, data_state};
use crate::date::{DateError, PlainDate};
use crate::state::{Disclosure, DisclosureAction, OpenState};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// DatePicker の anatomy（`data-scope="date-picker"`）。
const ANATOMY: Anatomy = anatomy("date-picker");

/// DatePicker の disabled/readonly/invalid/required 状態束。
/// root/label/control/input/trigger/clear-trigger の全パーツへ
/// [`data_disabled`]/[`data_invalid`]/[`data_readonly`] を一律付与し、
/// label にのみ [`data_required`] を追加で付与するために使う
/// （[`crate::combobox::ComboboxProps`] と同型のパターン）。状態機械
/// [`DatePicker`] にはフィールドを持たせず、呼び出しごとに
/// `&DatePickerProps` を渡す設計とする（hydration 属性面を拡張しない）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DatePickerProps {
    /// 無効化状態。`true` で `data-disabled` を各パーツへ付与し、
    /// [`input`]/[`trigger`]/[`clear_trigger`] にはネイティブ `disabled`
    /// 存在属性も追加する。
    pub disabled: bool,
    /// 読み取り専用状態。`true` で `data-readonly` を各パーツへ付与し、
    /// [`input`] にはネイティブ `readonly` 存在属性も追加する。操作自体の
    /// 抑止は `fandhe-frontend-wasm-full` 側の責務（モジュール冒頭
    /// 「スコープ外」節参照）。
    pub readonly: bool,
    /// 入力検証エラー状態。`true` で `data-invalid` を各パーツへ、
    /// [`input`] には追加で `aria-invalid="true"` を付与する。
    pub invalid: bool,
    /// 入力必須状態。`true` で [`label`] に `data-required` を、[`input`]
    /// にはネイティブ `required` 存在属性を付与する。
    pub required: bool,
}

/// [`DatePickerProps`] から root/label/control/input/trigger/clear-trigger
/// 共通の状態属性列を組み立てる非公開ヘルパ（disabled/invalid/readonly の
/// 3 属性、[`crate::combobox::state_attrs`] と同型）。
fn state_attrs(props: &DatePickerProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs: Vec<(&'static str, &'static str)> = Vec::new();
    attrs.extend(data_disabled(props.disabled));
    attrs.extend(data_invalid(props.invalid));
    attrs.extend(data_readonly(props.readonly));
    attrs
}

/// [`DatePickerProps`] が全パーツへ一律付与する属性キー一覧。呼び出し側
/// `attrs` にこれらと同名キーが含まれていても fail-closed で除去する対象。
const STATE_RESERVED: &[&str] = &["data-disabled", "data-invalid", "data-readonly"];

/// [`root`]/[`control`]/[`trigger`] が固定付与するキー一覧（[`STATE_RESERVED`]
/// に `data-state` を加えたもの）。
const STATEFUL_CONTAINER_RESERVED: &[&str] = &[
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-state",
];

/// [`label`] が固定付与するキー一覧（[`STATE_RESERVED`] に `data-required`
/// を加えたもの）。
const LABEL_RESERVED: &[&str] = &[
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-required",
];

/// [`clear_trigger`] が固定付与するキー一覧（`data-state` を持たないため
/// [`STATE_RESERVED`] と同じ集合、意味的に別名を与える）。
const CLEAR_TRIGGER_RESERVED: &[&str] = STATE_RESERVED;

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（[`crate::combobox::drop_reserved`]/
/// [`crate::color_picker::drop_reserved`] と同型の重複実装。モジュール間の
/// 相互依存を避けるため個別に定義する）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// Root パーツ（`div`）。開閉状態と [`DatePickerProps`] の状態束を `data-*`
/// へ反映する。
#[must_use]
pub fn root<'a>(
    state: OpenState,
    props: &DatePickerProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATEFUL_CONTAINER_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。`id` が `Some` のとき [`trigger`]/[`content`] の
/// `labelledby` と対で `aria-labelledby` 関連付けを成立させる。`for_` が
/// `Some` のとき [`input`] の `id` と対でネイティブ `label[for]` 関連付けを
/// 成立させる（ark-ui の `htmlFor` 準拠、[`crate::combobox::label`] と同型、
/// イシュー #1627 参照突合）。[`DatePickerProps`] の状態束 + `data-required`
/// を付与する。
#[must_use]
pub fn label<'a>(
    props: &DatePickerProps,
    id: Option<&'a str>,
    for_: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, LABEL_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(data_required(props.required));
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(for_) = for_ {
        merged.push(("for", for_));
    }
    merged.extend(attrs);
    ANATOMY.part("label", "label", merged, children)
}

/// Control パーツ（`div`）。[`input`]/[`trigger`]/[`clear_trigger`] をまとめる
/// コンテナ。開閉状態と [`DatePickerProps`] の状態束を `data-*` へ反映する。
#[must_use]
pub fn control<'a>(
    state: OpenState,
    props: &DatePickerProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATEFUL_CONTAINER_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// Input パーツ（ネイティブ `input`）。
///
/// `value` は [`PlainDate::to_iso_string`] 由来の ISO 8601 表記のみを渡す
/// 契約とする（モジュール doc §DateInput との責務境界参照）。`type="text"`
/// を固定する。[`DatePickerProps`] の状態束を付与し、`props.disabled`/
/// `props.readonly`/`props.required` はそれぞれネイティブ
/// `disabled`/`readonly`/`required` 存在属性へも反映する。`props.invalid`
/// のときのみ `aria-invalid="true"` を追加する（valid のときは省略、
/// [`crate::combobox::input`] と同型の判断、イシュー #1627 参照突合）。
#[must_use]
pub fn input<'a>(
    value: Option<&'a str>,
    props: &DatePickerProps,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "text")];
    if let Some(value) = value {
        merged.push(("value", value));
    }
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(state_attrs(props));
    if props.disabled {
        merged.push(("disabled", ""));
    }
    if props.readonly {
        merged.push(("readonly", ""));
    }
    if props.required {
        merged.push(("required", ""));
    }
    if props.invalid {
        merged.push(aria_invalid(true));
    }
    merged.extend(attrs);
    ANATOMY.part("input", "input", merged, Vec::new())
}

/// Trigger パーツ（`button`）。popover を開閉するトリガー
/// （[`crate::popover::trigger`] と同型、`aria-haspopup="dialog"` 固定）。
/// [`DatePickerProps`] の状態束を付与し、`props.disabled` のときのみ
/// `disabled` ネイティブ属性を追加する。
#[must_use]
pub fn trigger<'a>(
    state: OpenState,
    props: &DatePickerProps,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATEFUL_CONTAINER_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_haspopup(AriaPopup::Dialog),
        aria_expanded(state.is_open()),
        data_state(state.as_data_state()),
    ];
    if let Some(id) = controls {
        merged.push(aria_controls(id));
    }
    merged.extend(state_attrs(props));
    if props.disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// ClearTrigger パーツ（`button`）。[`crate::popover::close_trigger`] と同型。
/// [`DatePickerProps`] の状態束を付与し、`props.disabled` のときのみ
/// `disabled` ネイティブ属性を追加する。
#[must_use]
pub fn clear_trigger<'a>(
    props: &DatePickerProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, CLEAR_TRIGGER_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    merged.extend(state_attrs(props));
    if props.disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("clear-trigger", "button", merged, children)
}

/// Positioner パーツ（`div`）。[`crate::popover::positioner`] と同型
/// （位置決めロジックは [`crate::positioning`] へ委譲、本関数は開閉に応じた
/// `hidden` のみを担う）。`data-placement` は意図的に非追随（モジュール doc
/// 参照突合節参照）。
#[must_use]
pub fn positioner<'a>(
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("positioner", "div", merged, children)
}

/// Content パーツ（`div`）。`role="dialog"` を固定付与し、内部に
/// [`crate::calendar::Calendar`] のパーツ関数群を合成する想定
/// （[`crate::popover::content`] と同型）。`role`/`aria-haspopup` は
/// WAI-ARIA APG「Date Picker Dialog」パターンとの整合を優先し現状維持する
/// （モジュール doc 参照突合節参照）。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    id: Option<&'a str>,
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![role("dialog"), data_state(state.as_data_state())];
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
    pub fn root<'a>(
        &self,
        props: &DatePickerProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.open_state(), props, attrs, children)
    }

    /// [`control`] へ現在の開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(
        &self,
        props: &DatePickerProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        control(self.open_state(), props, attrs, children)
    }

    /// [`trigger`] へ現在の開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        props: &DatePickerProps,
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(self.open_state(), props, controls, attrs, children)
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
        let props = DatePickerProps::default();
        self.root(
            &props,
            Vec::new(),
            vec![
                trigger(state, &props, None, Vec::new(), Vec::new()),
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

    fn all_true_props() -> DatePickerProps {
        DatePickerProps {
            disabled: true,
            readonly: true,
            invalid: true,
            required: true,
        }
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
        let props = DatePickerProps::default();
        let html = render(&trigger(OpenState::Closed, &props, None, vec![], vec![]));
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
        let props = DatePickerProps::default();
        let html = render(&input(Some("2026-07-15"), &props, None, vec![]));
        assert!(html.contains(r#"value="2026-07-15""#));
        assert!(html.contains(r#"type="text""#));
    }

    #[test]
    fn input_disabled_true_adds_native_and_data_disabled() {
        let props = DatePickerProps {
            disabled: true,
            ..Default::default()
        };
        let html = render(&input(None, &props, None, vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    // --- DatePickerProps 網羅（イシュー #1627） ---

    #[test]
    fn default_props_emit_no_state_attrs_anywhere() {
        let props = DatePickerProps::default();
        let root_html = render(&root(OpenState::Closed, &props, vec![], vec![]));
        assert!(!root_html.contains("data-disabled"));
        assert!(!root_html.contains("data-invalid"));
        assert!(!root_html.contains("data-readonly"));

        let label_html = render(&label(&props, None, None, vec![], vec![]));
        assert!(!label_html.contains("data-required"));

        let input_html = render(&input(None, &props, None, vec![]));
        assert!(!input_html.contains("disabled"));
        assert!(!input_html.contains("readonly"));
        assert!(!input_html.contains("required"));
        assert!(!input_html.contains("aria-invalid"));
    }

    #[test]
    fn all_true_props_reach_root_label_control_input_trigger_clear_trigger() {
        let props = all_true_props();
        let state = OpenState::Closed;

        let root_html = render(&root(state, &props, vec![], vec![]));
        assert!(root_html.contains(r#"data-disabled="""#));
        assert!(root_html.contains(r#"data-invalid="""#));
        assert!(root_html.contains(r#"data-readonly="""#));

        let label_html = render(&label(&props, None, None, vec![], vec![]));
        assert!(label_html.contains(r#"data-disabled="""#));
        assert!(label_html.contains(r#"data-invalid="""#));
        assert!(label_html.contains(r#"data-readonly="""#));
        assert!(label_html.contains(r#"data-required="""#));

        let control_html = render(&control(state, &props, vec![], vec![]));
        assert!(control_html.contains(r#"data-disabled="""#));
        assert!(control_html.contains(r#"data-invalid="""#));
        assert!(control_html.contains(r#"data-readonly="""#));

        let input_html = render(&input(None, &props, None, vec![]));
        assert!(input_html.contains(r#"data-disabled="""#));
        assert!(input_html.contains(r#"data-invalid="""#));
        assert!(input_html.contains(r#"data-readonly="""#));
        assert!(input_html.contains(r#"disabled="""#));
        assert!(input_html.contains(r#"readonly="""#));
        assert!(input_html.contains(r#"required="""#));
        assert!(input_html.contains(r#"aria-invalid="true""#));

        let trigger_html = render(&trigger(state, &props, None, vec![], vec![]));
        assert!(trigger_html.contains(r#"data-disabled="""#));
        assert!(trigger_html.contains(r#"data-invalid="""#));
        assert!(trigger_html.contains(r#"data-readonly="""#));
        assert!(trigger_html.contains(r#"disabled="""#));

        let clear_trigger_html = render(&clear_trigger(&props, vec![], vec![]));
        assert!(clear_trigger_html.contains(r#"data-disabled="""#));
        assert!(clear_trigger_html.contains(r#"data-invalid="""#));
        assert!(clear_trigger_html.contains(r#"data-readonly="""#));
        assert!(clear_trigger_html.contains(r#"disabled="""#));
    }

    #[test]
    fn label_for_and_id_produce_both_attributes() {
        let props = DatePickerProps::default();
        let html = render(&label(
            &props,
            Some("dp-label"),
            Some("dp-input"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="dp-label""#));
        assert!(html.contains(r#"for="dp-input""#));
    }

    #[test]
    fn caller_supplied_state_data_attrs_are_dropped_case_insensitively() {
        let props = all_true_props();
        let html = render(&root(
            OpenState::Closed,
            &props,
            vec![
                ("DATA-DISABLED", "attacker"),
                ("Data-Invalid", "attacker"),
                ("data-readonly", "attacker"),
                ("data-state", "attacker"),
            ],
            vec![],
        ));
        assert!(!html.contains("attacker"));
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
        let props = DatePickerProps::default();
        let html = render(&trigger(
            OpenState::Closed,
            &props,
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn input_value_payload_is_escaped_on_render() {
        let props = DatePickerProps::default();
        let html = render(&input(Some(ATTR_BREAK_PAYLOAD), &props, None, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn label_for_payload_is_escaped_on_render() {
        let props = DatePickerProps::default();
        let html = render(&label(
            &props,
            None,
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let props = DatePickerProps::default();
        let html = render(&root(
            OpenState::Closed,
            &props,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="date-picker""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }
}

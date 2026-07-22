//! Select（リストボックス選択）headless コンポーネント（イシュー #541、親 #539）。
//!
//! ark-ui の Select
//!（`.claude/skills/ark-ui/references/components/collections/select.md`）を
//! 参考に、Root / Label / Control / Trigger / ValueText / ClearTrigger /
//! Indicator / Positioner / Content / ItemGroup / ItemGroupLabel / Item /
//! ItemText / ItemIndicator / HiddenSelect の 15 anatomy パーツと、Phase 1
//! （#524）の [`crate::state::Disclosure`]（listbox の開閉）+
//! [`crate::state::SingleSelect`]（選択値）を合成した状態機械 [`Select`] を
//! 提供する。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`label`]/[`control`]/[`trigger`]/
//! [`value_text`]/[`clear_trigger`]/[`indicator`]/[`positioner`]/[`content`]/
//! [`item_group`]/[`item_group_label`]/[`item`]/[`item_text`]/
//! [`item_indicator`]/[`hidden_select`]、いずれも純粋関数で完結）を直接呼んで
//! 組み立てる。CSR/hydration は [`Select`]（[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"open"`/`"close"`/`"toggle"`/`"select"`/`"deselect"`）で listbox 開閉と
//! 選択値の状態遷移をする。`fandhe-frontend-pre-styled-ui`（#546〜）が本
//! モジュールを呼んでスタイル済み Select を組み立てる想定である。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`hidden`/`disabled`/`id`/`value`/
//!   `selected`/`tabindex`/`name`）はすべて `&'static str` リテラルで固定して
//!   おり、動的値が属性名スロットへ混入する経路はない（[`mod@crate::anatomy`]/
//!   [`crate::aria`]/[`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（選択値 `value`/`id`/`controls`/`labelledby`/`name`/option の
//!   ラベルテキスト/呼び出し側 `attrs`/`children`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は [`crate::state::OpenState`]
//!   に一元化し、本モジュールで独自の値を作らない。[`item`]/[`item_indicator`]
//!   の `data-state` も同語彙を選択有無の表現に再利用する（[`crate::accordion`]
//!   の `SingleSelect::item_data_state()` と同じ契約）。
//! - hydration 属性（`data-hydrate-state`/`data-hydrate-selected`）はクライアント
//!   側で改ざんされうる入力として扱う。[`Select`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は [`crate::state::Disclosure`]/
//!   [`crate::state::SingleSelect`] へ全委譲することで、panic せず
//!   `HydrateError` を返す既存保証をそのまま継承する。
//! - dispatch payload（選択値）は改ざんされうるクライアント入力として扱い、
//!   HTML として解釈せず値として保持する（[`crate::state::SingleSelect`] の
//!   既存契約を継承）。
//!
//! # out-of-scope（本イシュー #541 のスコープ外）
//!
//! - **`data-state` の `checked`/`unchecked` 語彙の導入**: ark-ui は item に
//!   `checked`/`unchecked` を使うが、本リポジトリの既存不変条件
//!   （`data-state` 値語彙は [`crate::state::OpenState`] に一元化する）に
//!   合わせ、選択有無も `"open"`/`"closed"` で表現する。語彙導入の是非は
//!   form 系（#535 Checkbox）の判断に委ねる。
//! - **multiple 選択**: 高々 1 個の選択のみ扱う（[`crate::state::SingleSelect`]
//!   の既存スコープをそのまま継承）。
//! - **highlight（`data-highlighted`/`aria-activedescendant`）・typeahead・
//!   キーボードナビゲーション**: CSR 挙動層の責務であり、wasm 層の将来
//!   イシューのスコープ。
//! - **位置決めロジック（Floating UI 相当）**: [`positioner`] は CSS フック
//!   （data-* セレクタ）のみを提供する（Popover/Tooltip と同じ判断）。
//! - **`closeOnSelect` 以外の close 制御・lazyMount・portal**: クライアント
//!   ランタイム側のイベント処理・DOM 操作であり、wasm 層の将来イシューの
//!   スコープ。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{
    aria_controls, aria_expanded, aria_haspopup, aria_labelledby, aria_selected, role, AriaPopup,
};
use crate::data_attrs::{data_disabled, data_state};
use crate::state::{Disclosure, OpenState, SingleSelect, SingleSelectAction};
use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Select の anatomy（`data-scope="select"`）。
const ANATOMY: Anatomy = anatomy("select");

/// Root パーツ（`div`）。listbox の開閉状態を `data-*` へ反映する。
#[must_use]
pub fn root<'a>(state: OpenState, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。`id` が `Some` のとき [`content`]/[`trigger`] の
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

/// Control パーツ（`div`）。トリガー・値表示・クリアボタン等をまとめる
/// コンテナ。開閉状態を `data-*` へ反映するのみの最小主義な装飾用パーツ。
#[must_use]
pub fn control<'a>(state: OpenState, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// Trigger パーツ（`button`）。
///
/// フォーム内配置時の意図しない submit を防ぐため `type="button"` を固定で
/// 付与する（A05 セキュリティ設定ミス対策、既存コンポーネントと同判断）。
/// `aria-haspopup="listbox"` を固定付与し、`controls` が `Some` のとき
/// `aria-controls` で [`content`] と、`labelledby` が `Some` のとき
/// `aria-labelledby` で [`label`] と関連付ける。`disabled` はネイティブ
/// `disabled` 存在属性と `data-disabled` の両方へ反映する。
#[must_use]
pub fn trigger<'a>(
    state: OpenState,
    disabled: bool,
    controls: Option<&'a str>,
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_haspopup(AriaPopup::Listbox),
        aria_expanded(state.is_open()),
        data_state(state.as_data_state()),
    ];
    if let Some(controls) = controls {
        merged.push(aria_controls(controls));
    }
    if let Some(labelledby) = labelledby {
        merged.push(aria_labelledby(labelledby));
    }
    merged.extend(data_disabled(disabled));
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// ValueText パーツ（`span`）。`data-part="value-text"`（ark-ui 準拠の
/// kebab-case）。プレースホルダー表示中（未選択）のときのみ
/// `data-placeholder-shown` 存在属性を付与する。
#[must_use]
pub fn value_text<'a>(
    placeholder_shown: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if placeholder_shown {
        merged.push(("data-placeholder-shown", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("value-text", "span", merged, children)
}

/// ClearTrigger パーツ（`button`）。`data-part="clear-trigger"`（ark-ui 準拠の
/// kebab-case）。[`trigger`] と同じくフォーム内配置時の意図しない submit を
/// 防ぐため `type="button"` を固定で付与する。アクセシブルネーム
/// （`aria-label` 等）は本関数の `attrs` を通じて呼び出し側が付与する責務と
/// する（[`crate::popover::close_trigger`] と同じ判断）。
#[must_use]
pub fn clear_trigger<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    merged.extend(attrs);
    ANATOMY.part("clear-trigger", "button", merged, children)
}

/// Indicator パーツ（`span`）。開閉状態のみを `data-state` へ反映する
/// 最小主義な装飾用パーツ（アイコン等は呼び出し側の `attrs`/`children` が担う）。
#[must_use]
pub fn indicator<'a>(
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("indicator", "span", merged, children)
}

/// Positioner パーツ（`div`）。位置決めロジックのコンテナ。開閉状態を
/// `data-*` へ反映し、closed のとき `hidden` 存在属性を付与することで
/// [`content`] を含めて SSR/no-JS マークアップから閉状態を表現する
/// （Popover/Tooltip の `positioner` と同じ判断）。位置決め計算自体は
/// スコープ外（モジュール doc §out-of-scope 参照）。
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

/// Content パーツ（`div`）。
///
/// `role="listbox"` を固定付与する。`id` が `Some` のとき [`trigger`] の
/// `controls` と対で関連付ける。`labelledby` が `Some` のとき
/// `aria-labelledby` で [`label`] と関連付ける。closed のとき `hidden`
/// 存在属性を付与し、JS なしの SSR でも閉状態を表現する。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    id: Option<&'a str>,
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![role("listbox"), data_state(state.as_data_state())];
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

/// ItemGroup パーツ（`div`）。`data-part="item-group"`（ark-ui 準拠の
/// kebab-case）。`labelledby` が `Some` のときのみ `role="group"` と
/// `aria-labelledby` をセットで付与する（名前なし group を作らないため、
/// [`crate::accordion::item_content`] の `labelled_by` と同じ判断）。
#[must_use]
pub fn item_group<'a>(
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(labelledby) = labelledby {
        merged.push(role("group"));
        merged.push(aria_labelledby(labelledby));
    }
    merged.extend(attrs);
    ANATOMY.part("item-group", "div", merged, children)
}

/// ItemGroupLabel パーツ（`div`）。`data-part="item-group-label"`（ark-ui
/// 準拠の kebab-case）。`id` が `Some` のとき [`item_group`] の `labelledby`
/// と対で関連付ける。
#[must_use]
pub fn item_group_label<'a>(
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("item-group-label", "div", merged, children)
}

/// Item パーツ（`div`）。1 個の選択肢の選択状態・disabled 状態を `data-*`/ARIA
/// へ反映する。
///
/// `role="option"` を固定付与する。`data-state` は選択有無を
/// [`crate::state::OpenState`] の既存語彙（`"open"`/`"closed"`）で表現する
/// （モジュール doc §out-of-scope 参照。ark-ui の `checked`/`unchecked` は
/// 採用しない）。`value` は `data-value` として動的値のまま出力し、
/// `render()` の既定エスケープを必ず経由する。
#[must_use]
pub fn item<'a>(
    selected_state: OpenState,
    disabled: bool,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("option"),
        aria_selected(selected_state.is_open()),
        data_state(selected_state.as_data_state()),
        ("data-value", value),
    ];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item", "div", merged, children)
}

/// ItemText パーツ（`span`）。`data-part="item-text"`（ark-ui 準拠の
/// kebab-case）。`id` が `Some` のとき呼び出し側が `aria-labelledby` 等と
/// 関連付けるための識別子として使える。
#[must_use]
pub fn item_text<'a>(
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("item-text", "span", merged, children)
}

/// ItemIndicator パーツ（`span`）。`data-part="item-indicator"`（ark-ui 準拠の
/// kebab-case）。選択状態を `data-state` へ反映し、非選択のとき `hidden`
/// 存在属性を付与する（チェックマーク等のアイコンを非選択時に隠す用途、
/// [`crate::accordion::item_content`] の `hidden` 判断と同型）。
#[must_use]
pub fn item_indicator<'a>(
    selected_state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(selected_state.as_data_state())];
    if !selected_state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("item-indicator", "span", merged, children)
}

/// HiddenSelect パーツ（`select`）。フォーム統合用のネイティブ `<select>`。
///
/// `aria-hidden="true"` + `tabindex="-1"` を固定付与し、支援技術・フォーカスの
/// 両方から二重露出しないようにする（A05 セキュリティ設定ミス対策の一環。
/// 視覚的な UI は [`trigger`]/[`content`] 側が担い、本パーツはフォーム
/// 送信のためだけに存在する）。`options` は `(value, label)` の列であり、
/// 各要素は `el("option", ..)` として組み立てる。`selected` と `value` が
/// 一致する option にのみ `selected` 存在属性を付与する。値・ラベルは
/// いずれも動的だが `render()` の既定エスケープを必ず経由する。
#[must_use]
pub fn hidden_select<'a>(
    selected: Option<&'a str>,
    name: Option<&'a str>,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    options: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("aria-hidden", "true"), ("tabindex", "-1")];
    if let Some(name) = name {
        merged.push(("name", name));
    }
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);

    let option_nodes: Vec<Node> = options
        .into_iter()
        .map(|(value, option_label)| {
            let mut option_attrs: Vec<(&'a str, &'a str)> = vec![("value", value)];
            if selected == Some(value) {
                option_attrs.push(("selected", ""));
            }
            el("option", option_attrs, vec![text(option_label)])
        })
        .collect();

    ANATOMY.part("hidden-select", "select", merged, option_nodes)
}

/// [`Select`] に対する型付きアクション。
///
/// WASM 境界の文字列 dispatch（`name`/`payload`）とは
/// [`Select::decode_action`] で接続する。[`crate::state::Disclosure`] の
/// `"open"`/`"close"`/`"toggle"` と [`crate::state::SingleSelect`] の
/// `"select"`/`"deselect"` を合成するが、両者の dispatch 名は
/// `"toggle"`（listbox 開閉/選択トグルの二重定義）で衝突するため、
/// [`SingleSelect::decode_action`] へは委譲せず本 enum が独自にデコードする。
/// `"toggle"` は「listbox 開閉のトグル」（トリガークリックの自然な意味論）に
/// 割り当て、[`SingleSelect`] 側の `Toggle` 意味論は採用しない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectAction {
    /// listbox を開く。
    Open,
    /// listbox を閉じる。
    Close,
    /// listbox の開閉を反転する。
    Toggle,
    /// 指定した項目値を選択する（ark-ui の `closeOnSelect` 既定 `true` に
    /// 準拠し、選択と同時に listbox を閉じる）。
    Select(String),
    /// 選択を解除する（[`clear_trigger`] 相当）。
    Deselect,
}

/// [`Disclosure`]（listbox の開閉）+ [`SingleSelect`]（選択値）を埋め込んだ
/// Select の状態機械。
///
/// `data-state`/`aria-selected`/`aria-expanded` と実際の状態の整合を
/// 型レベルで保証する入口として、状態を取る各パーツ関数（[`root`]/
/// [`control`]/[`trigger`]/[`value_text`]/[`indicator`]/[`positioner`]/
/// [`content`]/[`item`]/[`item_indicator`]/[`hidden_select`]）へ現在状態を
/// 注入する利便メソッドを提供する。状態を取らないパーツ（[`label`]/
/// [`clear_trigger`]/[`item_group`]/[`item_group_label`]/[`item_text`]）は
/// 自由関数のみを提供し、`Select` のメソッドとしては公開しない。SSR での
/// 自由関数直接利用（本型を経由しない構成）も引き続き可能。`Default` は
/// closed・未選択（SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Select {
    disclosure: Disclosure,
    selection: SingleSelect,
}

impl Select {
    /// 現在の listbox 開閉状態。
    #[must_use]
    pub fn open_state(&self) -> OpenState {
        self.disclosure.state()
    }

    /// listbox が開いているかどうか。
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.disclosure.state().is_open()
    }

    /// 現在選択中の項目値（未選択なら `None`）。
    #[must_use]
    pub fn selected(&self) -> Option<&str> {
        self.selection.selected()
    }

    /// 指定した項目値が選択中かどうか。
    #[must_use]
    pub fn is_selected(&self, value: &str) -> bool {
        self.selection.is_selected(value)
    }

    /// 項目 `value` の現在の選択状態（選択中なら [`OpenState::Open`]、
    /// それ以外は [`OpenState::Closed`]。[`item`]/[`item_indicator`] の
    /// `data-state` 語彙と一致させるための変換）。
    #[must_use]
    pub fn item_state(&self, value: &str) -> OpenState {
        if self.is_selected(value) {
            OpenState::Open
        } else {
            OpenState::Closed
        }
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
        labelledby: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(
            self.open_state(),
            disabled,
            controls,
            labelledby,
            attrs,
            children,
        )
    }

    /// [`value_text`] へ現在の選択有無（未選択ならプレースホルダー表示）を
    /// 注入する利便メソッド。
    #[must_use]
    pub fn value_text<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        value_text(self.selected().is_none(), attrs, children)
    }

    /// [`indicator`] へ現在の開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn indicator<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        indicator(self.open_state(), attrs, children)
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

    /// [`item`] へ項目 `value` の現在の選択状態を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        value: &'a str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(self.item_state(value), disabled, value, attrs, children)
    }

    /// [`item_indicator`] へ項目 `value` の現在の選択状態を注入する利便メソッド。
    #[must_use]
    pub fn item_indicator<'a>(
        &self,
        value: &str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_indicator(self.item_state(value), attrs, children)
    }

    /// [`hidden_select`] へ現在の選択値を注入する利便メソッド。
    #[must_use]
    pub fn hidden_select<'a>(
        &'a self,
        name: Option<&'a str>,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        options: Vec<(&'a str, &'a str)>,
    ) -> Node {
        hidden_select(self.selected(), name, disabled, attrs, options)
    }
}

impl Component for Select {
    type Action = SelectAction;

    fn update(&mut self, action: SelectAction) {
        match action {
            SelectAction::Open => self.disclosure.update(crate::state::DisclosureAction::Open),
            SelectAction::Close => self
                .disclosure
                .update(crate::state::DisclosureAction::Close),
            SelectAction::Toggle => self
                .disclosure
                .update(crate::state::DisclosureAction::Toggle),
            SelectAction::Select(value) => {
                self.selection.update(SingleSelectAction::Select(value));
                // ark-ui の closeOnSelect 既定 true に準拠し、選択と同時に
                // listbox を閉じる（モジュール doc §セキュリティ不変条件・
                // SelectAction rustdoc 参照）。
                self.disclosure
                    .update(crate::state::DisclosureAction::Close);
            }
            SelectAction::Deselect => self.selection.update(SingleSelectAction::Deselect),
        }
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > trigger + positioner > content、children 空・id なし）。
    /// [`crate::popover::Popover`] と同じ位置付けであり、公開 UI としての
    /// 利用は想定しない（実際の UI 構築は §パーツ関数群を呼び出し側が
    /// 組み合わせる）。
    fn view(&self) -> Node {
        let state = self.open_state();
        self.root(
            Vec::new(),
            vec![
                trigger(state, false, None, None, Vec::new(), Vec::new()),
                positioner(
                    state,
                    Vec::new(),
                    vec![content(state, None, None, Vec::new(), Vec::new())],
                ),
            ],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<SelectAction> {
        match name {
            "open" => Some(SelectAction::Open),
            "close" => Some(SelectAction::Close),
            "toggle" => Some(SelectAction::Toggle),
            "select" => Some(SelectAction::Select(payload.to_string())),
            "deselect" => Some(SelectAction::Deselect),
            _ => None,
        }
    }
}

impl Hydrate for Select {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let mut attrs = self.disclosure.hydration_attrs();
        attrs.extend(self.selection.hydration_attrs());
        attrs
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            disclosure: Disclosure::from_hydration_attrs(attrs)?,
            selection: SingleSelect::from_hydration_attrs(attrs)?,
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
    fn root_outputs_scope_part_and_state() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="select""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn label_id_some_outputs_id() {
        let html = render(&label(Some("select-label-1"), vec![], vec![text("Fruit")]));
        assert!(html.contains(r#"<label"#));
        assert!(html.contains(r#"id="select-label-1""#));
    }

    #[test]
    fn control_outputs_scope_part_and_state() {
        let html = render(&control(OpenState::Open, vec![], vec![]));
        assert!(html.contains(r#"data-part="control""#));
        assert!(html.contains(r#"data-state="open""#));
    }

    #[test]
    fn trigger_has_type_button_haspopup_listbox_and_aria_expanded() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-haspopup="listbox""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(!html.contains("aria-controls"));
        assert!(!html.contains("aria-labelledby"));
        assert!(!html.contains("disabled"));

        let html_open = render(&trigger(OpenState::Open, false, None, None, vec![], vec![]));
        assert!(html_open.contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn trigger_controls_and_labelledby_some_outputs_both() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            Some("select-content-1"),
            Some("select-label-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-controls="select-content-1""#));
        assert!(html.contains(r#"aria-labelledby="select-label-1""#));
    }

    #[test]
    fn trigger_disabled_true_adds_native_and_data_disabled() {
        let html = render(&trigger(
            OpenState::Closed,
            true,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"disabled="""#));
    }

    #[test]
    fn trigger_disabled_false_omits_both_disabled_attrs() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains(r#" disabled"#));
    }

    #[test]
    fn value_text_placeholder_shown_only_when_true() {
        let placeholder = render(&value_text(true, vec![], vec![text("Select a fruit")]));
        assert!(placeholder.contains(r#"data-placeholder-shown="""#));

        let with_value = render(&value_text(false, vec![], vec![text("Apple")]));
        assert!(!with_value.contains("data-placeholder-shown"));
    }

    #[test]
    fn clear_trigger_has_type_button_and_kebab_case_part() {
        let html = render(&clear_trigger(vec![], vec![]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-part="clear-trigger""#));
    }

    #[test]
    fn indicator_outputs_scope_part_and_state_only() {
        let html = render(&indicator(OpenState::Open, vec![], vec![text("v")]));
        assert!(html.contains(r#"data-scope="select""#));
        assert!(html.contains(r#"data-part="indicator""#));
        assert!(html.contains(r#"data-state="open""#));
    }

    #[test]
    fn positioner_closed_has_hidden_attr_open_does_not() {
        let closed = render(&positioner(OpenState::Closed, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&positioner(OpenState::Open, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_has_role_listbox_and_state() {
        let html = render(&content(OpenState::Open, None, None, vec![], vec![]));
        assert!(html.contains(r#"role="listbox""#));
        assert!(html.contains(r#"data-state="open""#));
    }

    #[test]
    fn content_closed_has_hidden_attr_open_does_not() {
        let closed = render(&content(OpenState::Closed, None, None, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&content(OpenState::Open, None, None, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_id_and_labelledby_some_outputs_both() {
        let html = render(&content(
            OpenState::Open,
            Some("select-content-1"),
            Some("select-label-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="select-content-1""#));
        assert!(html.contains(r#"aria-labelledby="select-label-1""#));
    }

    #[test]
    fn item_group_labelledby_some_outputs_role_group_and_aria_labelledby_together() {
        let html = render(&item_group(Some("group-label-1"), vec![], vec![]));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"aria-labelledby="group-label-1""#));
    }

    #[test]
    fn item_group_labelledby_none_omits_role_and_aria_labelledby() {
        let html = render(&item_group(None, vec![], vec![]));
        assert!(!html.contains("role="));
        assert!(!html.contains("aria-labelledby"));
    }

    #[test]
    fn item_group_label_id_some_outputs_id() {
        let html = render(&item_group_label(Some("group-label-1"), vec![], vec![]));
        assert!(html.contains(r#"id="group-label-1""#));
    }

    #[test]
    fn item_has_role_option_aria_selected_and_data_value() {
        let html = render(&item(OpenState::Open, false, "vue", vec![], vec![]));
        assert!(html.contains(r#"role="option""#));
        assert!(html.contains(r#"aria-selected="true""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-value="vue""#));

        let unselected = render(&item(OpenState::Closed, false, "react", vec![], vec![]));
        assert!(unselected.contains(r#"aria-selected="false""#));
        assert!(unselected.contains(r#"data-state="closed""#));
    }

    #[test]
    fn item_disabled_true_adds_data_disabled() {
        let html = render(&item(OpenState::Closed, true, "svelte", vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_text_id_some_outputs_id() {
        let html = render(&item_text(Some("item-text-1"), vec![], vec![text("Vue")]));
        assert!(html.contains(r#"id="item-text-1""#));
    }

    #[test]
    fn item_indicator_selected_shown_unselected_hidden() {
        let selected = render(&item_indicator(OpenState::Open, vec![], vec![text("✓")]));
        assert!(!selected.contains("hidden"));
        assert!(selected.contains(r#"data-state="open""#));

        let unselected = render(&item_indicator(OpenState::Closed, vec![], vec![]));
        assert!(unselected.contains(r#"hidden="""#));
        assert!(unselected.contains(r#"data-state="closed""#));
    }

    #[test]
    fn hidden_select_has_native_select_and_aria_hidden_tabindex() {
        let html = render(&hidden_select(
            Some("vue"),
            Some("framework"),
            false,
            vec![],
            vec![("vue", "Vue"), ("react", "React")],
        ));
        assert!(html.contains(r#"<select"#));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"name="framework""#));
        assert!(html.contains(r#"<option value="vue" selected="">Vue</option>"#));
        assert!(html.contains(r#"<option value="react">React</option>"#));
    }

    #[test]
    fn hidden_select_name_none_omits_name_attr() {
        let html = render(&hidden_select(None, None, false, vec![], vec![]));
        assert!(!html.contains("name="));
    }

    #[test]
    fn hidden_select_disabled_true_adds_native_disabled() {
        let html = render(&hidden_select(None, None, true, vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            OpenState::Closed,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="select""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- Select: dispatch 統合 ---

    #[test]
    fn select_default_is_closed_and_unselected() {
        let s = Select::default();
        assert_eq!(s.open_state(), OpenState::Closed);
        assert_eq!(s.selected(), None);
    }

    #[test]
    fn select_dispatch_open_close_toggle() {
        let mut s = Select::default();
        assert!(dispatch(&mut s, "open", ""));
        assert!(s.is_open());
        assert!(dispatch(&mut s, "close", ""));
        assert!(!s.is_open());
        assert!(dispatch(&mut s, "toggle", ""));
        assert!(s.is_open());
        assert!(dispatch(&mut s, "toggle", ""));
        assert!(!s.is_open());
    }

    #[test]
    fn select_dispatch_select_updates_value_and_closes_listbox() {
        let mut s = Select::default();
        dispatch(&mut s, "open", "");
        assert!(s.is_open());

        assert!(dispatch(&mut s, "select", "vue"));
        assert_eq!(s.selected(), Some("vue"));
        assert!(!s.is_open(), "closeOnSelect: 選択と同時に listbox を閉じる");
    }

    #[test]
    fn select_dispatch_deselect_clears_selection() {
        let mut s = Select::default();
        dispatch(&mut s, "select", "vue");
        assert!(dispatch(&mut s, "deselect", ""));
        assert_eq!(s.selected(), None);
    }

    #[test]
    fn select_dispatch_ignores_unknown_action() {
        let mut s = Select::default();
        dispatch(&mut s, "select", "vue");
        assert!(!dispatch(&mut s, "no_such_action", "x"));
        assert_eq!(s.selected(), Some("vue"));
    }

    // --- Select: 利便メソッド経由の描画が状態機械と一致 ---

    #[test]
    fn select_convenience_methods_reflect_state() {
        let mut s = Select::default();
        dispatch(&mut s, "open", "");
        dispatch(&mut s, "select", "vue");

        // select の副作用で listbox は閉じるため、比較用に再度開く。
        dispatch(&mut s, "open", "");

        let item_vue = render(&s.item("vue", false, vec![], vec![]));
        assert!(item_vue.contains(r#"aria-selected="true""#));

        let item_react = render(&s.item("react", false, vec![], vec![]));
        assert!(item_react.contains(r#"aria-selected="false""#));

        let value_text_html = render(&s.value_text(vec![], vec![]));
        assert!(!value_text_html.contains("data-placeholder-shown"));
    }

    #[test]
    fn select_value_text_shows_placeholder_when_unselected() {
        let s = Select::default();
        let html = render(&s.value_text(vec![], vec![text("Select a fruit")]));
        assert!(html.contains(r#"data-placeholder-shown="""#));
    }

    // --- Select: SSR 状態なし初期描画 ---

    #[test]
    fn select_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Select::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- Select: hydration 経路 ---

    #[test]
    fn select_hydration_round_trip_open_and_selected() {
        let mut s = Select::default();
        dispatch(&mut s, "open", "");
        dispatch(&mut s, "select", "vue");
        // select が listbox を閉じるため、開いた状態を保つには再 open する。
        dispatch(&mut s, "open", "");

        let rendered = render(&render_for_hydration(&s));
        assert!(rendered.contains(r#"data-hydrate-state="open""#));
        assert!(rendered.contains("data-hydrate-selected="));

        let restored = Select::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }

    #[test]
    fn select_hydration_round_trip_closed_and_unselected() {
        let s = Select::default();
        let restored = Select::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }

    #[test]
    fn select_from_hydration_attrs_missing_state_attr_does_not_panic() {
        let err = Select::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-state".to_string())
        );
    }

    #[test]
    fn select_from_hydration_attrs_invalid_state_value_does_not_panic() {
        for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
            let attrs = vec![
                ("data-hydrate-state".to_string(), bogus.to_string()),
                (
                    "data-hydrate-selected".to_string(),
                    fandhe_frontend_interactive::codec::encode_list(&[]),
                ),
            ];
            let err = Select::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    #[test]
    fn select_from_hydration_attrs_invalid_selected_value_does_not_panic() {
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["a".to_string(), "b".to_string()]);
        let attrs = vec![
            ("data-hydrate-state".to_string(), "closed".to_string()),
            ("data-hydrate-selected".to_string(), bogus),
        ];
        let err = Select::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰: 動的値にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn trigger_controls_and_labelledby_payload_is_escaped_on_render() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            Some(ATTR_BREAK_PAYLOAD),
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_value_payload_is_escaped_on_render() {
        let html = render(&item(
            OpenState::Closed,
            false,
            ATTR_BREAK_PAYLOAD,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn hidden_select_option_label_and_value_payload_is_escaped_on_render() {
        let html = render(&hidden_select(
            None,
            None,
            false,
            vec![],
            vec![(ATTR_BREAK_PAYLOAD, "<script>alert(1)</script>")],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&quot;"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            OpenState::Closed,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&indicator(
            OpenState::Open,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn select_dispatch_select_payload_is_escaped_on_render() {
        let mut s = Select::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut s, "select", payload));

        let rendered = render(&render_for_hydration(&s));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn select_view_root_is_element_for_render_for_hydration() {
        // render_for_hydration はルートが Node::Element であることを前提に
        // 属性を合成する（`crates/interactive/src/lib.rs` 参照）。本型の
        // view() が常に Element を返すことを固定する回帰テスト。
        let node = Select::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }
}

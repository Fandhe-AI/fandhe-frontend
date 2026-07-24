//! Listbox（常時展開のリスト選択）headless コンポーネント（イシュー #750、
//! 親 #748）。
//!
//! ark-ui の Listbox
//!（`.claude/skills/ark-ui/references/components/collections/listbox.md`）/
//! chakra-ui の Listbox 相当を参考に、Root / Label / Content / ItemGroup /
//! ItemGroupLabel / Item / ItemText / ItemIndicator / ValueText の 9 anatomy
//! パーツと、single モード [`state::SingleSelect`] を埋め込んだ [`Listbox`]、
//! multiple モード [`state::MultiSelect`] を埋め込んだ [`MultiListbox`] の
//! 2 状態機械を提供する。
//!
//! # [`crate::select::Select`] との責務境界（イシュー #750 明示要件）
//!
//! [`crate::select::Select`] は**ポップアップ型**の選択コンポーネントであり、
//! [`state::Disclosure`]（listbox の開閉）+ [`state::SingleSelect`]（選択値）
//! の合成、trigger/positioner/hidden-select（フォーム送信対応）を持つ。
//! 対して本モジュールの Listbox/MultiListbox は**常時展開**（開閉状態を
//! 一切持たない）であり、trigger/positioner を持たない。ポップアップ選択
//! （クリックで開閉するドロップダウン）が必要な場合は [`crate::select`] を
//! 使うこと。逆に「常に見えているリストから 1 個または複数個を選ぶ」用途
//! （フィルタ UI・複数選択パネル等）には本モジュールを使う。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`label`]/[`content`]/
//! [`item_group`]/[`item_group_label`]/[`item`]/[`item_text`]/
//! [`item_indicator`]/[`value_text`]、いずれも純粋関数で完結）を直接呼んで
//! 組み立てる。CSR/hydration は [`Listbox`]（single）/ [`MultiListbox`]
//! （multiple）（いずれも [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"select"`/`"deselect"`/`"toggle"`）で選択状態を遷移する。
//! `fandhe-frontend-pre-styled-ui`（#750）が本モジュールを呼んでスタイル済み
//! Listbox を組み立てる想定である。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`id`/`tabindex`）はすべて `&'static str`
//!   リテラルで固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`mod@crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の既存
//!   不変条件をそのまま継承する）。
//! - 動的値（項目値 `value`/`id`/`labelledby`/`activedescendant`/呼び出し側
//!   `attrs`/`children`）は [`fandhe_frontend_core::render`] の既定エスケープを
//!   必ず経由する。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は
//!   [`crate::state::OpenState`] に一元化し、選択有無の表現に再利用する
//!   （[`crate::select`] の `item`/`item_indicator` と同じ契約）。
//! - hydration 属性（`data-hydrate-selected`）はクライアント側で改ざんされ
//!   うる入力として扱う。[`Listbox`]/[`MultiListbox`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は
//!   [`state::SingleSelect`]/[`state::MultiSelect`] へ全委譲することで、
//!   panic せず `HydrateError` を返す既存保証をそのまま継承する（single は
//!   2 件以上、multiple は重複値を fail-closed 拒否）。
//! - dispatch payload（選択値）は改ざんされうるクライアント入力として扱い、
//!   HTML として解釈せず値として保持する（[`state::SingleSelect`]/
//!   [`state::MultiSelect`] の既存契約を継承）。
//!
//! # out-of-scope（本イシュー #750 のスコープ外、PR 本文で別イシュー化を提案）
//!
//! - **`"extended"` selection mode**（Cmd/Ctrl 修飾による範囲・追加選択）:
//!   本モジュールは single（[`Listbox`]）/ multiple（[`MultiListbox`]）の
//!   2 モードのみ提供する。
//! - **キーボードナビゲーション・typeahead・loopFocus の実 DOM 配線**:
//!   [`item`] の `highlighted` 引数・[`content`] の `activedescendant` 引数は
//!   `data-highlighted`/`aria-activedescendant` の SSR 静的表現のみを提供し、
//!   実際の移動・typeahead は wasm 層（`fandhe-frontend-wasm-full`）の将来
//!   イシューのスコープ（[`crate::select`] §out-of-scope と同じ判断）。
//! - **chakra 固有の `Input`（フィルタ入力）/`Empty` パーツ**: 未提供。
//! - **フォーム送信用 hidden input（[`crate::select::hidden_select`] 相当）**:
//!   常時展開の Listbox はネイティブ `<select>` によるフォーム統合を前提と
//!   しないため未提供。
//! - **grid collection**（2 次元ナビゲーション）: 未提供。
//! - **[`value_text`] の `data-bind-text` 束縛**（[`crate::select::VALUE_TEXT_FIELD`]
//!   相当）: wasm 配線とセットで行う後続イシューのスコープ。
//! - **`examples/headless-pre-styled-ui` への節追加**: 別イシューで対応する。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{
    aria_activedescendant, aria_disabled, aria_labelledby, aria_multiselectable, aria_selected,
    role,
};
use crate::data_attrs::{data_disabled, data_highlighted, data_state};
use crate::state::{MultiSelect, MultiSelectAction, OpenState, SingleSelect, SingleSelectAction};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Listbox の anatomy（`data-scope="listbox"`）。
const ANATOMY: Anatomy = anatomy("listbox");

/// Root パーツ（`div`）。選択有無・disabled を `data-*` へ反映する。
#[must_use]
pub fn root<'a>(
    selection_state: OpenState,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(selection_state.as_data_state())];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。`id` が `Some` のとき [`content`] の `labelledby`
/// と対で `aria-labelledby` 関連付けを成立させる。
#[must_use]
pub fn label<'a>(id: Option<&'a str>, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("label", "label", merged, children)
}

/// Content パーツ（`div`）。
///
/// `role="listbox"` を固定付与する。`multiple` が `true` のとき
/// `aria-multiselectable="true"` を付与する（single モードでは属性自体を
/// 省略し、既定の単一選択セマンティクスに任せる）。`labelledby` が `Some`
/// のとき [`label`] と関連付ける。`activedescendant` が `Some` のとき
/// `aria-activedescendant` を付与し、値は現在ハイライト中の [`item`] の
/// `id` と対応させる（[`crate::select::content`] と同じ SSR 静的表現、
/// イシュー #599 の踏襲）。`tabindex="0"` を固定付与し、DOM フォーカスを
/// 本パーツ自身が受ける（[`crate::select`] のように trigger が別途フォーカス
/// を受けるポップアップ型とは異なる、モジュール doc §責務境界参照）。
#[must_use]
pub fn content<'a>(
    multiple: bool,
    id: Option<&'a str>,
    labelledby: Option<&'a str>,
    activedescendant: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("listbox"), ("tabindex", "0")];
    if multiple {
        merged.push(aria_multiselectable(true));
    }
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(labelledby) = labelledby {
        merged.push(aria_labelledby(labelledby));
    }
    if let Some(activedescendant) = activedescendant {
        merged.push(aria_activedescendant(activedescendant));
    }
    merged.extend(attrs);
    ANATOMY.part("content", "div", merged, children)
}

/// ItemGroup パーツ（`div`）。`data-part="item-group"`（ark-ui 準拠の
/// kebab-case）。`labelledby` が `Some` のときのみ `role="group"` と
/// `aria-labelledby` をセットで付与する（名前なし group を作らないため、
/// [`crate::select::item_group`] と同じ判断）。
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
/// （[`crate::select::item`] と同じ判断。ark-ui の `checked`/`unchecked` は
/// 採用しない）。`value` は `data-value` として動的値のまま出力し、
/// `render()` の既定エスケープを必ず経由する。`disabled` が `true` のとき
/// `aria-disabled="true"` と `data-disabled` を対で付与する（本パーツは
/// `div[role="option"]` でありネイティブの `disabled` 属性を持たないため、
/// 支援技術へは ARIA 経由でのみ伝達できる。[`crate::select::item`] の
/// PR #568 Bugbot 対応と同じ契約）。
///
/// `highlighted`（キーボードナビゲーション等によるフォーカス位置）は
/// クライアントランタイムの領域だが、SSR でも `data-highlighted` を出力
/// できるよう `bool` 引数として受ける（状態機械には持たせない。
/// [`crate::select::item`] と同じ契約）。`id` が `Some` のとき、[`content`]
/// の `activedescendant` 引数の参照先として使う識別子になる。
#[must_use]
pub fn item<'a>(
    selected_state: OpenState,
    disabled: bool,
    highlighted: bool,
    value: &'a str,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("option"),
        aria_selected(selected_state.is_open()),
        data_state(selected_state.as_data_state()),
        ("data-value", value),
    ];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if disabled {
        merged.push(aria_disabled(true));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(data_highlighted(highlighted));
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
/// [`crate::select::item_indicator`] と同型）。
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

/// ValueText パーツ（`span`）。`data-part="value-text"`（ark-ui 準拠の
/// kebab-case）。選択項目が 1 件もないときのみ `data-placeholder-shown`
/// 存在属性を付与する（[`crate::select::value_text`] と同じ判断）。
///
/// [`crate::select::VALUE_TEXT_FIELD`] 相当の `data-bind-text` 束縛は
/// wasm 配線とセットで行う後続イシューのスコープであり、本パーツには
/// 付与しない（モジュール doc §out-of-scope 参照）。
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

/// [`state::SingleSelect`]（高々 1 個選択）を埋め込んだ single モード
/// Listbox の状態機械。
///
/// 状態を取る各パーツ関数（[`root`]/[`item`]/[`item_indicator`]/
/// [`value_text`]）へ現在状態を注入する利便メソッドを提供する。状態を
/// 取らないパーツ（[`label`]/[`content`]/[`item_group`]/
/// [`item_group_label`]/[`item_text`]）は自由関数のみを提供し、`Listbox`
/// のメソッドとしては公開しない（[`crate::select::Select`] と同じ設計）。
/// SSR での自由関数直接利用（本型を経由しない構成）も引き続き可能。
/// `Default` は未選択（SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Listbox {
    selection: SingleSelect,
}

impl Listbox {
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

    /// ルート全体の選択有無を表す [`OpenState`]（いずれかの項目が選択中
    /// なら `Open`、未選択なら `Closed`）。
    #[must_use]
    fn root_state(&self) -> OpenState {
        if self.selection.selected().is_some() {
            OpenState::Open
        } else {
            OpenState::Closed
        }
    }

    /// [`root`] へ現在の選択有無を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.root_state(), disabled, attrs, children)
    }

    /// [`item`] へ項目 `value` の現在の選択状態を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        value: &'a str,
        disabled: bool,
        highlighted: bool,
        id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(
            self.item_state(value),
            disabled,
            highlighted,
            value,
            id,
            attrs,
            children,
        )
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

    /// [`value_text`] へ現在の選択有無（未選択ならプレースホルダー表示）を
    /// 注入する利便メソッド。
    #[must_use]
    pub fn value_text<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        value_text(self.selected().is_none(), attrs, children)
    }
}

impl Component for Listbox {
    type Action = SingleSelectAction;

    fn update(&mut self, action: SingleSelectAction) {
        self.selection.update(action);
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root、children 空）。[`state::SingleSelect::view`] と同じ
    /// 位置付けであり、公開 UI としての利用は想定しない。
    fn view(&self) -> Node {
        self.root(false, Vec::new(), Vec::new())
    }

    fn decode_action(name: &str, payload: &str) -> Option<SingleSelectAction> {
        SingleSelect::decode_action(name, payload)
    }
}

impl Hydrate for Listbox {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.selection.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            selection: SingleSelect::from_hydration_attrs(attrs)?,
        })
    }
}

/// [`state::MultiSelect`]（0 個以上の同時選択）を埋め込んだ multiple モード
/// Listbox の状態機械。
///
/// [`Listbox`]（single モード）と対称の API を提供する。「複数項目が
/// 同時に選択される」ことを許すため [`Self::selected`] は `&[String]` を
/// 返す。型を分ける理由は [`crate::accordion::MultiAccordion`] rustdoc と
/// 同じ（dispatch 契約の静的確定・hydration の fail-closed 性維持）。
/// `Default` は空選択（SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MultiListbox {
    selection: MultiSelect,
}

impl MultiListbox {
    /// 現在選択中の項目値（選択順）。
    #[must_use]
    pub fn selected(&self) -> &[String] {
        self.selection.selected()
    }

    /// 指定した項目値が選択中かどうか。
    #[must_use]
    pub fn is_selected(&self, value: &str) -> bool {
        self.selection.is_selected(value)
    }

    /// 項目 `value` の現在の [`OpenState`]。
    #[must_use]
    pub fn item_state(&self, value: &str) -> OpenState {
        if self.is_selected(value) {
            OpenState::Open
        } else {
            OpenState::Closed
        }
    }

    /// ルート全体の選択有無を表す [`OpenState`]（いずれかの項目が選択中
    /// なら `Open`、全未選択なら `Closed`）。
    #[must_use]
    fn root_state(&self) -> OpenState {
        if self.selection.selected().is_empty() {
            OpenState::Closed
        } else {
            OpenState::Open
        }
    }

    /// [`root`] へ現在の選択有無を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.root_state(), disabled, attrs, children)
    }

    /// [`item`] へ項目 `value` の現在の選択状態を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        value: &'a str,
        disabled: bool,
        highlighted: bool,
        id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(
            self.item_state(value),
            disabled,
            highlighted,
            value,
            id,
            attrs,
            children,
        )
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

    /// [`value_text`] へ現在の選択有無（未選択ならプレースホルダー表示）を
    /// 注入する利便メソッド。
    #[must_use]
    pub fn value_text<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        value_text(self.selection.selected().is_empty(), attrs, children)
    }
}

impl Component for MultiListbox {
    type Action = MultiSelectAction;

    fn update(&mut self, action: MultiSelectAction) {
        self.selection.update(action);
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root、children 空）。[`state::MultiSelect::view`] と同じ
    /// 位置付け。
    fn view(&self) -> Node {
        self.root(false, Vec::new(), Vec::new())
    }

    fn decode_action(name: &str, payload: &str) -> Option<MultiSelectAction> {
        MultiSelect::decode_action(name, payload)
    }
}

impl Hydrate for MultiListbox {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.selection.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            selection: MultiSelect::from_hydration_attrs(attrs)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/data-state/ARIA 出力 ---

    #[test]
    fn root_outputs_scope_part_state_and_disabled() {
        let html = render(&root(OpenState::Closed, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="listbox""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(!html.contains("data-disabled"));

        let html_disabled = render(&root(OpenState::Open, true, vec![], vec![]));
        assert!(html_disabled.contains(r#"data-state="open""#));
        assert!(html_disabled.contains(r#"data-disabled="""#));
    }

    #[test]
    fn label_id_some_outputs_id() {
        let html = render(&label(Some("listbox-label-1"), vec![], vec![text("Fruit")]));
        assert!(html.contains(r#"<label"#));
        assert!(html.contains(r#"id="listbox-label-1""#));
    }

    #[test]
    fn content_has_role_listbox_and_tabindex() {
        let html = render(&content(false, None, None, None, vec![], vec![]));
        assert!(html.contains(r#"role="listbox""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(!html.contains("aria-multiselectable"));
        assert!(!html.contains("aria-labelledby"));
        assert!(!html.contains("aria-activedescendant"));
    }

    #[test]
    fn content_multiple_true_outputs_aria_multiselectable_true() {
        let html = render(&content(true, None, None, None, vec![], vec![]));
        assert!(html.contains(r#"aria-multiselectable="true""#));
    }

    #[test]
    fn content_id_labelledby_activedescendant_some_outputs_all() {
        let html = render(&content(
            false,
            Some("listbox-content-1"),
            Some("listbox-label-1"),
            Some("listbox-item-2"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="listbox-content-1""#));
        assert!(html.contains(r#"aria-labelledby="listbox-label-1""#));
        assert!(html.contains(r#"aria-activedescendant="listbox-item-2""#));
    }

    #[test]
    fn item_group_labelledby_none_omits_role_and_labelledby() {
        let html = render(&item_group(None, vec![], vec![]));
        assert!(!html.contains("role"));
        assert!(!html.contains("aria-labelledby"));
    }

    #[test]
    fn item_group_labelledby_some_outputs_role_group_and_labelledby() {
        let html = render(&item_group(Some("group-label-1"), vec![], vec![]));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"aria-labelledby="group-label-1""#));
    }

    #[test]
    fn item_group_label_id_some_outputs_id() {
        let html = render(&item_group_label(
            Some("group-label-1"),
            vec![],
            vec![text("Citrus")],
        ));
        assert!(html.contains(r#"id="group-label-1""#));
    }

    #[test]
    fn item_has_role_option_aria_selected_and_data_state() {
        let html = render(&item(
            OpenState::Open,
            false,
            false,
            "apple",
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="option""#));
        assert!(html.contains(r#"aria-selected="true""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-value="apple""#));
        assert!(!html.contains("aria-disabled"));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-highlighted"));

        let html_closed = render(&item(
            OpenState::Closed,
            false,
            false,
            "banana",
            None,
            vec![],
            vec![],
        ));
        assert!(html_closed.contains(r#"aria-selected="false""#));
        assert!(html_closed.contains(r#"data-state="closed""#));
    }

    #[test]
    fn item_disabled_true_adds_aria_and_data_disabled_pair() {
        let html = render(&item(
            OpenState::Closed,
            true,
            false,
            "banana",
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-disabled="true""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_highlighted_true_adds_data_highlighted() {
        let html = render(&item(
            OpenState::Closed,
            false,
            true,
            "banana",
            Some("listbox-item-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-highlighted="""#));
        assert!(html.contains(r#"id="listbox-item-1""#));
    }

    #[test]
    fn item_text_id_some_outputs_id() {
        let html = render(&item_text(Some("item-text-1"), vec![], vec![text("Apple")]));
        assert!(html.contains(r#"id="item-text-1""#));
    }

    #[test]
    fn item_indicator_hides_when_not_selected() {
        let selected = render(&item_indicator(OpenState::Open, vec![], vec![]));
        assert!(!selected.contains("hidden"));
        assert!(selected.contains(r#"data-state="open""#));

        let unselected = render(&item_indicator(OpenState::Closed, vec![], vec![]));
        assert!(unselected.contains(r#"hidden="""#));
        assert!(unselected.contains(r#"data-state="closed""#));
    }

    #[test]
    fn value_text_placeholder_shown_only_when_true() {
        let placeholder = render(&value_text(true, vec![], vec![text("Select a fruit")]));
        assert!(placeholder.contains(r#"data-placeholder-shown="""#));

        let with_value = render(&value_text(false, vec![], vec![text("Apple")]));
        assert!(!with_value.contains("data-placeholder-shown"));
    }

    // --- Listbox（single モード）: dispatch 統合 ---

    #[test]
    fn listbox_select_and_deselect_via_dispatch() {
        let mut l = Listbox::default();
        assert_eq!(l.selected(), None);

        assert!(dispatch(&mut l, "select", "apple"));
        assert_eq!(l.selected(), Some("apple"));
        assert!(l.is_selected("apple"));

        assert!(dispatch(&mut l, "select", "banana"));
        assert_eq!(l.selected(), Some("banana"));
        assert!(!l.is_selected("apple"));

        assert!(dispatch(&mut l, "deselect", ""));
        assert_eq!(l.selected(), None);
    }

    #[test]
    fn listbox_toggle_via_dispatch() {
        let mut l = Listbox::default();
        assert!(dispatch(&mut l, "toggle", "apple"));
        assert_eq!(l.selected(), Some("apple"));

        assert!(dispatch(&mut l, "toggle", "apple"));
        assert_eq!(l.selected(), None);
    }

    #[test]
    fn listbox_convenience_methods_render_matching_state() {
        let mut l = Listbox::default();
        dispatch(&mut l, "select", "apple");

        let root_html = render(&l.root(false, vec![], vec![]));
        assert!(root_html.contains(r#"data-state="open""#));

        let selected_item_html = render(&l.item("apple", false, false, None, vec![], vec![]));
        assert!(selected_item_html.contains(r#"aria-selected="true""#));

        let unselected_item_html = render(&l.item("banana", false, false, None, vec![], vec![]));
        assert!(unselected_item_html.contains(r#"aria-selected="false""#));

        let indicator_html = render(&l.item_indicator("apple", vec![], vec![]));
        assert!(!indicator_html.contains("hidden"));

        let value_text_html = render(&l.value_text(vec![], vec![text("Apple")]));
        assert!(!value_text_html.contains("data-placeholder-shown"));
    }

    #[test]
    fn listbox_ssr_stateless_initial_render() {
        let rendered = render(&Listbox::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
    }

    #[test]
    fn listbox_hydration_round_trip() {
        let mut l = Listbox::default();
        dispatch(&mut l, "select", "apple");

        let attrs = l.hydration_attrs();
        assert!(attrs.iter().any(|(k, _)| k.contains("hydrate-selected")));

        let restored = Listbox::from_hydration_attrs(&attrs).unwrap();
        assert_eq!(restored, l);
        assert_eq!(restored.selected(), Some("apple"));
    }

    #[test]
    fn listbox_hydration_rejects_more_than_one_selected_value() {
        // 改ざん入力（single Listbox に 2 件以上のリスト）を fail-closed で
        // 拒否する（SingleSelect の既存保証を Listbox 経由でも固定）。
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["apple".to_string(), "banana".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = Listbox::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn listbox_hydration_missing_attr_is_rejected() {
        let err = Listbox::from_hydration_attrs(&[]).unwrap_err();
        assert!(matches!(err, HydrateError::MissingAttr(_)));
    }

    // --- MultiListbox（multiple モード）: dispatch 統合 ---

    #[test]
    fn multi_listbox_select_deselect_and_toggle_via_dispatch() {
        let mut m = MultiListbox::default();
        assert_eq!(m.selected(), &[] as &[String]);

        assert!(dispatch(&mut m, "select", "apple"));
        assert!(dispatch(&mut m, "select", "banana"));
        assert_eq!(m.selected(), &["apple".to_string(), "banana".to_string()]);
        assert!(m.is_selected("apple"));
        assert!(m.is_selected("banana"));

        assert!(dispatch(&mut m, "deselect", "apple"));
        assert_eq!(m.selected(), &["banana".to_string()]);

        assert!(dispatch(&mut m, "toggle", "banana"));
        assert_eq!(m.selected(), &[] as &[String]);
        assert!(dispatch(&mut m, "toggle", "banana"));
        assert_eq!(m.selected(), &["banana".to_string()]);
    }

    #[test]
    fn multi_listbox_convenience_methods_render_matching_state() {
        let mut m = MultiListbox::default();
        dispatch(&mut m, "select", "apple");
        dispatch(&mut m, "select", "banana");

        let root_html = render(&m.root(false, vec![], vec![]));
        assert!(root_html.contains(r#"data-state="open""#));

        let selected_item_html = render(&m.item("apple", false, false, None, vec![], vec![]));
        assert!(selected_item_html.contains(r#"aria-selected="true""#));

        let value_text_html = render(&m.value_text(vec![], vec![text("2 selected")]));
        assert!(!value_text_html.contains("data-placeholder-shown"));
    }

    #[test]
    fn multi_listbox_ssr_stateless_initial_render() {
        let rendered = render(&MultiListbox::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
    }

    #[test]
    fn multi_listbox_hydration_round_trip() {
        let mut m = MultiListbox::default();
        dispatch(&mut m, "select", "apple");
        dispatch(&mut m, "select", "banana");

        let attrs = m.hydration_attrs();
        let restored = MultiListbox::from_hydration_attrs(&attrs).unwrap();
        assert_eq!(restored, m);
        assert_eq!(
            restored.selected(),
            &["apple".to_string(), "banana".to_string()]
        );
    }

    #[test]
    fn multi_listbox_hydration_rejects_duplicate_selected_values() {
        // 改ざん入力（MultiListbox に重複値）を fail-closed で拒否する
        // （MultiSelect の既存保証を MultiListbox 経由でも固定）。
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["apple".to_string(), "apple".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = MultiListbox::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- render_for_hydration の疎通確認 ---

    #[test]
    fn listbox_render_for_hydration_includes_hydrate_attrs() {
        let mut l = Listbox::default();
        dispatch(&mut l, "select", "apple");
        let html = render(&render_for_hydration(&l));
        assert!(html.contains("data-hydrate-selected"));
    }

    #[test]
    fn multi_listbox_render_for_hydration_includes_hydrate_attrs() {
        let mut m = MultiListbox::default();
        dispatch(&mut m, "select", "apple");
        let html = render(&render_for_hydration(&m));
        assert!(html.contains("data-hydrate-selected"));
    }
}

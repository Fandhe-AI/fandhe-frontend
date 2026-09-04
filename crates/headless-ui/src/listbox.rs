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
//! - [`ListboxProps`] が全パーツへ一律付与する固定キー（`data-orientation`/
//!   `data-disabled`）は呼び出し側 `attrs` から `drop_reserved` により
//!   fail-closed に除外してから合成する（[`crate::checkbox`]/
//!   [`crate::angle_slider`] と同型のパターン、A01/A05 属性偽装防止）。
//!
//! # 参照突合（イシュー #1611）
//!
//! ark-ui/chakra-ui（実体は zag.js `@zag-js/listbox`）の anatomy・`data-*`
//! 語彙・キーボード操作と突合し、以下を是正した。
//!
//! - **[`ListboxProps`]（新設）**: zag の root-level `disabled`/
//!   `orientation` に対応する。root/label/content/item-group/item/
//!   value-text へ `data-disabled`（root 由来）・`data-orientation`
//!   （root/content/item-group/item）を一律付与する。
//! - **root disabled の item への伝播**: zag の `getItemState`
//!   （`disabled || itemDisabled`）に合わせ、[`item`] の有効 disabled は
//!   `props.disabled || disabled` で判定する（従来は root の disabled が
//!   item へ伝播せず、`crates/pre-styled-ui/src/listbox.rs::stylesheet`
//!   が raw CSS の hover 抑止規則で補っていた問題の headless 側是正）。
//! - **[`item`] への `data-selected`**（選択時のみ存在属性）を追加し、
//!   ark 互換の属性セレクタを提供する。
//! - **[`item_text`] への `data-state`/`data-disabled`/`data-highlighted`
//!   の 3 状態属性**を追加する（[`item`] の先頭 3 引数と同型）。
//! - **[`item_group_label`] への `role="presentation"`**、**[`item_indicator`]
//!   への `aria-hidden="true"`** を固定付与する。
//!
//! 一方、以下は zag との差分を**意図的に維持**する。
//!
//! - [`item`] の `data-state` は `"open"`/`"closed"` を維持する（[`crate::select`]
//!   と共有する既存語彙。Themes recipe が `item[data-state="open"]` を
//!   参照するため、zag の `checked`/`unchecked` は採用しない）。
//! - [`root`] の `data-state`（選択有無）・[`value_text`] の
//!   `data-placeholder-shown` は zag に無い追加分として維持する。
//! - `data-empty`/`data-layout`/`--column-count`/`data-activedescendant`
//!   （collection 抽象・grid・aria との重複）は非採用のまま（下記
//!   out-of-scope 節参照）。
//!
//! # [`crate::select::Select`] との責務境界（イシュー #750 明示要件）
//!
//! # out-of-scope（本イシュー #750/#1611 のスコープ外、PR 本文で別イシュー化を提案）
//!
//! - **`"extended"` selection mode**（Cmd/Ctrl 修飾による範囲・追加選択）:
//!   本モジュールは single（[`Listbox`]）/ multiple（[`MultiListbox`]）の
//!   2 モードのみ提供する。
//! - **Ctrl/Cmd+A による全選択**・**`deselectable` prop による Escape
//!   解除**: zag が持つが本モジュールは提供しない（ダイアログ内 Listbox が
//!   親の Escape 閉鎖を奪わない既存設計を維持するため）。
//! - **キーボードナビゲーション・typeahead・loopFocus の実 DOM 配線**:
//!   イシュー #1070 で `fandhe-frontend-wasm-full` の `keynav` モジュールへ
//!   実装済み（Arrow/Home/End・typeahead・Enter/Space の決定は content 直下
//!   highlight 項目への click 合成）。`loopFocus` は content への
//!   `data-loop-focus="true"` オプトインで有効になる（欠落時は非循環既定、
//!   `keynav` モジュール doc §Listbox 参照）。本モジュールは引き続き
//!   [`item`] の `highlighted` 引数・[`content`] の `activedescendant` 引数を
//!   通じて `data-highlighted`/`aria-activedescendant` の SSR 静的表現のみを
//!   提供し、実 DOM 配線自体は持たない（責務境界は不変）。**Enter/Space →
//!   選択 dispatch の接続**（`fandhe-frontend-wasm-full` の
//!   `headless.rs::MAPPING_TABLE` に listbox `item` 行が無く未接続）は
//!   本イシューのスコープ外として別イシュー化を提案する（イシュー #1611
//!   PR 本文参照）。
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
    aria_activedescendant, aria_disabled, aria_hidden, aria_labelledby, aria_multiselectable,
    aria_selected, role,
};
use crate::data_attrs::{
    data_disabled, data_highlighted, data_orientation, data_state, Orientation,
};
use crate::state::{MultiSelect, MultiSelectAction, OpenState, SingleSelect, SingleSelectAction};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Listbox の anatomy（`data-scope="listbox"`）。
const ANATOMY: Anatomy = anatomy("listbox");

/// root/label/content/item-group/item/value-text が共有するリスト全体の
/// 状態（イシュー #1611、[`crate::checkbox::CheckboxProps`]/
/// [`crate::angle_slider::AngleSliderProps`] と同型のパターン）。
///
/// zag の root-level `disabled`/`orientation` に対応する。`Default` は
/// 非 disabled・vertical（zag の既定と一致）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListboxProps {
    /// 無効化状態。`true` で `data-disabled` を root/label/content/
    /// item-group/value-text へ、`item` へは `disabled || item 個別の
    /// disabled` として伝播する（モジュール冒頭「参照突合」節参照）。
    pub disabled: bool,
    /// 向き。`data-orientation` として root/content/item-group/item へ
    /// 出力する。`horizontal` のとき `fandhe-frontend-wasm-full` の
    /// keynav が ArrowLeft/ArrowRight を受理する
    /// （`crates/wasm-full/src/keynav.rs` §Listbox 参照）。
    pub orientation: Orientation,
}

impl Default for ListboxProps {
    /// 非 disabled・vertical（zag の既定と一致）。`Orientation` は
    /// `Default` を実装していないため手書きで定義する。
    fn default() -> Self {
        Self {
            disabled: false,
            orientation: Orientation::Vertical,
        }
    }
}

/// [`ListboxProps`] から root/content/item-group/item 共通の
/// `data-orientation`・`data-disabled` 属性列を組み立てる非公開ヘルパ
/// （[`label`]/[`value_text`] は `data-disabled` のみを個別に付与する）。
fn root_state_attrs(props: &ListboxProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs: Vec<(&'static str, &'static str)> = vec![data_orientation(props.orientation)];
    attrs.extend(data_disabled(props.disabled));
    attrs
}

/// [`root`] が固定付与するキー一覧（[`root_state_attrs`] が出力する
/// `data-orientation`/`data-disabled` に `data-state` を加えたもの）。
const ROOT_RESERVED: &[&str] = &["data-state", "data-orientation", "data-disabled"];

/// [`content`] が固定付与するキー一覧。
const CONTENT_RESERVED: &[&str] = &[
    "data-orientation",
    "data-disabled",
    "role",
    "tabindex",
    "aria-multiselectable",
    "id",
    "aria-labelledby",
    "aria-activedescendant",
];

/// [`item`] が固定付与するキー一覧。
const ITEM_RESERVED: &[&str] = &[
    "role",
    "aria-selected",
    "data-state",
    "data-selected",
    "data-value",
    "data-orientation",
    "data-disabled",
    "data-highlighted",
    "aria-disabled",
    "id",
];

/// [`item_text`] が固定付与するキー一覧。
const ITEM_TEXT_RESERVED: &[&str] = &["data-state", "data-disabled", "data-highlighted", "id"];

/// [`item_group_label`] が固定付与するキー一覧。
const ITEM_GROUP_LABEL_RESERVED: &[&str] = &["role", "id"];

/// [`item_indicator`] が固定付与するキー一覧。
const ITEM_INDICATOR_RESERVED: &[&str] = &["aria-hidden", "data-state", "hidden"];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（[`crate::checkbox::drop_reserved`]/[`crate::angle_slider::drop_reserved`]
/// と同型の重複実装。モジュール間の相互依存を避けるため個別に定義する）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// Root パーツ（`div`）。選択有無を `data-state` へ、[`ListboxProps`] を
/// `data-orientation`/`data-disabled` へ反映する。
#[must_use]
pub fn root<'a>(
    selection_state: OpenState,
    props: &ListboxProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(selection_state.as_data_state())];
    merged.extend(root_state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。`id` が `Some` のとき [`content`] の `labelledby`
/// と対で `aria-labelledby` 関連付けを成立させる。[`ListboxProps::disabled`]
/// を `data-disabled` へ反映する。
#[must_use]
pub fn label<'a>(
    props: &ListboxProps,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, &["data-disabled"]);
    let mut merged: Vec<(&'a str, &'a str)> = data_disabled(props.disabled).into_iter().collect();
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
/// [`ListboxProps::orientation`] を `data-orientation` へ反映する
/// （`fandhe-frontend-wasm-full` の keynav が ArrowLeft/ArrowRight を受理
/// するかどうかの判定に使う呼び出し側オプトインだった属性を、イシュー
/// #1611 で常時出力へ変更した）。[`ListboxProps::disabled`] を
/// `data-disabled` へ反映する（[`root_state_attrs`] 経由。root/label/
/// content/item-group/item/value-text へ一律付与する契約、モジュール
/// doc §セキュリティ不変条件参照）。
#[must_use]
pub fn content<'a>(
    multiple: bool,
    props: &ListboxProps,
    id: Option<&'a str>,
    labelledby: Option<&'a str>,
    activedescendant: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, CONTENT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = root_state_attrs(props);
    merged.push(role("listbox"));
    merged.push(("tabindex", "0"));
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
/// [`crate::select::item_group`] と同じ判断）。[`ListboxProps`] を
/// `data-disabled`/`data-orientation` へ反映する。
#[must_use]
pub fn item_group<'a>(
    props: &ListboxProps,
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(
        attrs,
        &[
            "role",
            "aria-labelledby",
            "data-disabled",
            "data-orientation",
        ],
    );
    let mut merged: Vec<(&'a str, &'a str)> = root_state_attrs(props);
    if let Some(labelledby) = labelledby {
        merged.push(role("group"));
        merged.push(aria_labelledby(labelledby));
    }
    merged.extend(attrs);
    ANATOMY.part("item-group", "div", merged, children)
}

/// ItemGroupLabel パーツ（`div`）。`data-part="item-group-label"`（ark-ui
/// 準拠の kebab-case）。`id` が `Some` のとき [`item_group`] の `labelledby`
/// と対で関連付ける。`role="presentation"` を固定付与する（zag の
/// ItemGroupLabel anatomy に合わせる、イシュー #1611 参照突合）。
#[must_use]
pub fn item_group_label<'a>(
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_GROUP_LABEL_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("presentation")];
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
/// 採用しない）。選択時のみ `data-selected` 存在属性を追加する（ark 互換の
/// 属性セレクタ、イシュー #1611 参照突合）。`value` は `data-value` として
/// 動的値のまま出力し、`render()` の既定エスケープを必ず経由する。
///
/// 有効な disabled は `props.disabled || disabled`（zag の `getItemState`
/// と同じ「root disabled が子 item へ伝播する」契約、モジュール冒頭
/// 「参照突合」節参照）で判定し、`true` のとき `aria-disabled="true"` と
/// `data-disabled` を対で付与する（本パーツは `div[role="option"]` であり
/// ネイティブの `disabled` 属性を持たないため、支援技術へは ARIA 経由での
/// み伝達できる。[`crate::select::item`] の PR #568 Bugbot 対応と同じ契約）。
/// [`ListboxProps::orientation`] を `data-orientation` へ反映する。
///
/// `highlighted`（キーボードナビゲーション等によるフォーカス位置）は
/// クライアントランタイムの領域だが、SSR でも `data-highlighted` を出力
/// できるよう `bool` 引数として受ける（状態機械には持たせない。
/// [`crate::select::item`] と同じ契約）。`id` が `Some` のとき、[`content`]
/// の `activedescendant` 引数の参照先として使う識別子になる。
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn item<'a>(
    selected_state: OpenState,
    props: &ListboxProps,
    disabled: bool,
    highlighted: bool,
    value: &'a str,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_RESERVED);
    let effective_disabled = props.disabled || disabled;
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        data_orientation(props.orientation),
        role("option"),
        aria_selected(selected_state.is_open()),
        data_state(selected_state.as_data_state()),
        ("data-value", value),
    ];
    if selected_state.is_open() {
        merged.push(("data-selected", ""));
    }
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if effective_disabled {
        merged.push(aria_disabled(true));
    }
    merged.extend(data_disabled(effective_disabled));
    merged.extend(data_highlighted(highlighted));
    merged.extend(attrs);
    ANATOMY.part("item", "div", merged, children)
}

/// ItemText パーツ（`span`）。`data-part="item-text"`（ark-ui 準拠の
/// kebab-case）。`id` が `Some` のとき呼び出し側が `aria-labelledby` 等と
/// 関連付けるための識別子として使える。
///
/// `selected_state`/`disabled`/`highlighted` の 3 状態属性（[`item`] の
/// 先頭 3 引数と同型）を `data-state`/`data-disabled`/`data-highlighted`
/// として出力する（イシュー #1611 参照突合。`crate::pre_styled_ui` の
/// `SlotRecipe::state` は自パーツ属性しか条件にできないため、従来は
/// item-text を選択/highlight 状態で装飾できなかった問題を是正する）。
///
/// 有効な disabled は [`item`] と同じ `props.disabled || disabled`
/// （zag の `getItemState` と同じ「root disabled が子 item へ伝播する」
/// 契約）を本関数の内部で計算する。`item` 呼び出し側が有効値を
/// 再計算して渡す必要はなく、`props.disabled=true` かつ item-level
/// `disabled=false` でも親 [`item`] と同じ状態を反映する（PR #1888
/// codex-review 指摘: 従来は `disabled` を素通ししていたため
/// `props.disabled` が伝播せず契約が破れていた）。
#[must_use]
pub fn item_text<'a>(
    selected_state: OpenState,
    props: &ListboxProps,
    disabled: bool,
    highlighted: bool,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_TEXT_RESERVED);
    let effective_disabled = props.disabled || disabled;
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(selected_state.as_data_state())];
    merged.extend(data_disabled(effective_disabled));
    merged.extend(data_highlighted(highlighted));
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("item-text", "span", merged, children)
}

/// ItemIndicator パーツ（`span`）。`data-part="item-indicator"`（ark-ui 準拠の
/// kebab-case）。選択状態を `data-state` へ反映し、非選択のとき `hidden`
/// 存在属性を付与する（チェックマーク等のアイコンを非選択時に隠す用途、
/// [`crate::select::item_indicator`] と同型）。`aria-hidden="true"` を固定
/// 付与する（装飾アイコンであり `item` 自身の `aria-selected` が選択状態を
/// 既に伝達するため、支援技術の二重読み上げを防ぐ。イシュー #1611 参照突合、
/// zag の ItemIndicator anatomy に合わせる）。
#[must_use]
pub fn item_indicator<'a>(
    selected_state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_INDICATOR_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        aria_hidden(true),
        data_state(selected_state.as_data_state()),
    ];
    if !selected_state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("item-indicator", "span", merged, children)
}

/// ValueText パーツ（`span`）。`data-part="value-text"`（ark-ui 準拠の
/// kebab-case）。選択項目が 1 件もないときのみ `data-placeholder-shown`
/// 存在属性を付与する（[`crate::select::value_text`] と同じ判断）。
/// [`ListboxProps::disabled`] を `data-disabled` へ反映する。
///
/// [`crate::select::VALUE_TEXT_FIELD`] 相当の `data-bind-text` 束縛は
/// wasm 配線とセットで行う後続イシューのスコープであり、本パーツには
/// 付与しない（モジュール doc §out-of-scope 参照）。
#[must_use]
pub fn value_text<'a>(
    placeholder_shown: bool,
    props: &ListboxProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, &["data-placeholder-shown", "data-disabled"]);
    let mut merged: Vec<(&'a str, &'a str)> = data_disabled(props.disabled).into_iter().collect();
    if placeholder_shown {
        merged.push(("data-placeholder-shown", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("value-text", "span", merged, children)
}

/// [`state::SingleSelect`]（高々 1 個選択）を埋め込んだ single モード
/// Listbox の状態機械。
///
/// 状態を取る各パーツ関数（[`root`]/[`item`]/[`item_text`]/
/// [`item_indicator`]/[`value_text`]）へ現在状態を注入する利便メソッドを
/// 提供する。状態を取らないパーツ（[`label`]/[`content`]/[`item_group`]/
/// [`item_group_label`]）は自由関数のみを提供し、`Listbox` のメソッドとし
/// ては公開しない（[`crate::select::Select`] と同じ設計）。SSR での自由
/// 関数直接利用（本型を経由しない構成）も引き続き可能。`Default` は未選択
/// （SSR の状態なし初期描画に対応する既定値）。
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
        props: &ListboxProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.root_state(), props, attrs, children)
    }

    /// [`item`] へ項目 `value` の現在の選択状態を注入する利便メソッド。
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn item<'a>(
        &self,
        value: &'a str,
        props: &ListboxProps,
        disabled: bool,
        highlighted: bool,
        id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(
            self.item_state(value),
            props,
            disabled,
            highlighted,
            value,
            id,
            attrs,
            children,
        )
    }

    /// [`item_text`] へ項目 `value` の現在の選択状態を注入する利便メソッド。
    /// `props` は [`Self::item`] へ渡したものと同じ [`ListboxProps`] を
    /// 渡す想定（有効 disabled の計算は [`item_text`] 内部が担う、PR #1888
    /// codex-review 指摘参照）。
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn item_text<'a>(
        &self,
        value: &str,
        props: &ListboxProps,
        disabled: bool,
        highlighted: bool,
        id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_text(
            self.item_state(value),
            props,
            disabled,
            highlighted,
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
    pub fn value_text<'a>(
        &self,
        props: &ListboxProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        value_text(self.selected().is_none(), props, attrs, children)
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
        self.root(&ListboxProps::default(), Vec::new(), Vec::new())
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
        props: &ListboxProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.root_state(), props, attrs, children)
    }

    /// [`item`] へ項目 `value` の現在の選択状態を注入する利便メソッド。
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn item<'a>(
        &self,
        value: &'a str,
        props: &ListboxProps,
        disabled: bool,
        highlighted: bool,
        id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(
            self.item_state(value),
            props,
            disabled,
            highlighted,
            value,
            id,
            attrs,
            children,
        )
    }

    /// [`item_text`] へ項目 `value` の現在の選択状態を注入する利便メソッド。
    /// `props` は [`Self::item`] へ渡したものと同じ [`ListboxProps`] を
    /// 渡す想定（有効 disabled の計算は [`item_text`] 内部が担う、PR #1888
    /// codex-review 指摘参照）。
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn item_text<'a>(
        &self,
        value: &str,
        props: &ListboxProps,
        disabled: bool,
        highlighted: bool,
        id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_text(
            self.item_state(value),
            props,
            disabled,
            highlighted,
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
    pub fn value_text<'a>(
        &self,
        props: &ListboxProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        value_text(self.selection.selected().is_empty(), props, attrs, children)
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
        self.root(&ListboxProps::default(), Vec::new(), Vec::new())
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
    fn root_outputs_scope_part_state_orientation_and_disabled() {
        let html = render(&root(
            OpenState::Closed,
            &ListboxProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="listbox""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(!html.contains("data-disabled"));

        let props = ListboxProps {
            disabled: true,
            orientation: Orientation::Horizontal,
        };
        let html_disabled = render(&root(OpenState::Open, &props, vec![], vec![]));
        assert!(html_disabled.contains(r#"data-state="open""#));
        assert!(html_disabled.contains(r#"data-orientation="horizontal""#));
        assert!(html_disabled.contains(r#"data-disabled="""#));
    }

    #[test]
    fn root_drops_caller_supplied_reserved_keys() {
        let html = render(&root(
            OpenState::Closed,
            &ListboxProps::default(),
            vec![("data-state", "attacker"), ("DATA-ORIENTATION", "attacker")],
            vec![],
        ));
        assert!(!html.contains("attacker"));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn label_id_some_outputs_id_and_reflects_disabled() {
        let html = render(&label(
            &ListboxProps::default(),
            Some("listbox-label-1"),
            vec![],
            vec![text("Fruit")],
        ));
        assert!(html.contains(r#"<label"#));
        assert!(html.contains(r#"id="listbox-label-1""#));
        assert!(!html.contains("data-disabled"));

        let props = ListboxProps {
            disabled: true,
            ..ListboxProps::default()
        };
        let html_disabled = render(&label(&props, None, vec![], vec![]));
        assert!(html_disabled.contains(r#"data-disabled="""#));
    }

    #[test]
    fn content_has_role_listbox_tabindex_and_orientation() {
        let html = render(&content(
            false,
            &ListboxProps::default(),
            None,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="listbox""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(!html.contains("aria-multiselectable"));
        assert!(!html.contains("aria-labelledby"));
        assert!(!html.contains("aria-activedescendant"));
    }

    #[test]
    fn content_multiple_true_outputs_aria_multiselectable_true() {
        let html = render(&content(
            true,
            &ListboxProps::default(),
            None,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-multiselectable="true""#));
    }

    #[test]
    fn content_horizontal_orientation_reflects_in_data_orientation() {
        let props = ListboxProps {
            orientation: Orientation::Horizontal,
            ..ListboxProps::default()
        };
        let html = render(&content(false, &props, None, None, None, vec![], vec![]));
        assert!(html.contains(r#"data-orientation="horizontal""#));
    }

    #[test]
    fn content_id_labelledby_activedescendant_some_outputs_all() {
        let html = render(&content(
            false,
            &ListboxProps::default(),
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
    fn content_reflects_disabled_from_props() {
        let props = ListboxProps {
            disabled: true,
            ..ListboxProps::default()
        };
        let html = render(&content(false, &props, None, None, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn content_omits_data_disabled_when_not_disabled() {
        let html = render(&content(
            false,
            &ListboxProps::default(),
            None,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn content_drops_caller_supplied_data_disabled() {
        let html = render(&content(
            false,
            &ListboxProps::default(),
            None,
            None,
            None,
            vec![("data-disabled", "attacker"), ("DATA-DISABLED", "attacker")],
            vec![],
        ));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn item_group_labelledby_none_omits_role_and_labelledby() {
        let html = render(&item_group(&ListboxProps::default(), None, vec![], vec![]));
        assert!(!html.contains("role"));
        assert!(!html.contains("aria-labelledby"));
        assert!(html.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn item_group_labelledby_some_outputs_role_group_and_labelledby() {
        let html = render(&item_group(
            &ListboxProps::default(),
            Some("group-label-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"aria-labelledby="group-label-1""#));
    }

    #[test]
    fn item_group_reflects_disabled_from_props() {
        let props = ListboxProps {
            disabled: true,
            ..ListboxProps::default()
        };
        let html = render(&item_group(&props, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_group_label_id_some_outputs_id_and_fixed_role_presentation() {
        let html = render(&item_group_label(
            Some("group-label-1"),
            vec![],
            vec![text("Citrus")],
        ));
        assert!(html.contains(r#"id="group-label-1""#));
        assert!(html.contains(r#"role="presentation""#));
    }

    #[test]
    fn item_group_label_drops_caller_supplied_role() {
        let html = render(&item_group_label(None, vec![("role", "attacker")], vec![]));
        assert!(!html.contains("attacker"));
        assert!(html.contains(r#"role="presentation""#));
    }

    #[test]
    fn item_has_role_option_aria_selected_data_state_and_orientation() {
        let html = render(&item(
            OpenState::Open,
            &ListboxProps::default(),
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
        assert!(html.contains(r#"data-selected="""#));
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(!html.contains("aria-disabled"));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-highlighted"));

        let html_closed = render(&item(
            OpenState::Closed,
            &ListboxProps::default(),
            false,
            false,
            "banana",
            None,
            vec![],
            vec![],
        ));
        assert!(html_closed.contains(r#"aria-selected="false""#));
        assert!(html_closed.contains(r#"data-state="closed""#));
        assert!(!html_closed.contains("data-selected"));
    }

    #[test]
    fn item_disabled_true_adds_aria_and_data_disabled_pair() {
        let html = render(&item(
            OpenState::Closed,
            &ListboxProps::default(),
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
    fn item_inherits_disabled_from_root_props_even_when_item_level_disabled_is_false() {
        // 参照突合（イシュー #1611）: zag の getItemState（disabled ||
        // itemDisabled）と同じく root disabled が個々の item へ伝播する。
        let props = ListboxProps {
            disabled: true,
            ..ListboxProps::default()
        };
        let html = render(&item(
            OpenState::Closed,
            &props,
            false,
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
            &ListboxProps::default(),
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
    fn item_drops_caller_supplied_reserved_keys() {
        let html = render(&item(
            OpenState::Open,
            &ListboxProps::default(),
            false,
            false,
            "apple",
            None,
            vec![("data-selected", "attacker"), ("ROLE", "attacker")],
            vec![],
        ));
        assert!(!html.contains("attacker"));
        assert!(html.contains(r#"role="option""#));
    }

    #[test]
    fn item_text_id_some_outputs_id_and_three_state_attrs() {
        let html = render(&item_text(
            OpenState::Open,
            &ListboxProps::default(),
            false,
            true,
            Some("item-text-1"),
            vec![],
            vec![text("Apple")],
        ));
        assert!(html.contains(r#"id="item-text-1""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-highlighted="""#));
        assert!(!html.contains("data-disabled"));

        let html_disabled = render(&item_text(
            OpenState::Closed,
            &ListboxProps::default(),
            true,
            false,
            None,
            vec![],
            vec![],
        ));
        assert!(html_disabled.contains(r#"data-state="closed""#));
        assert!(html_disabled.contains(r#"data-disabled="""#));
        assert!(!html_disabled.contains("data-highlighted"));
    }

    #[test]
    fn item_text_reflects_root_disabled_propagation() {
        // PR #1888 codex-review 指摘の回帰: props.disabled=true かつ
        // item-level disabled=false でも item_text は有効 disabled
        // （props.disabled || disabled）を反映しなければならない。
        let root_disabled = ListboxProps {
            disabled: true,
            ..ListboxProps::default()
        };
        let html = render(&item_text(
            OpenState::Open,
            &root_disabled,
            false,
            false,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_indicator_hides_when_not_selected_and_has_aria_hidden() {
        let selected = render(&item_indicator(OpenState::Open, vec![], vec![]));
        assert!(!selected.contains("hidden=\"\""));
        assert!(selected.contains(r#"data-state="open""#));
        assert!(selected.contains(r#"aria-hidden="true""#));

        let unselected = render(&item_indicator(OpenState::Closed, vec![], vec![]));
        assert!(unselected.contains(r#"hidden="""#));
        assert!(unselected.contains(r#"data-state="closed""#));
        assert!(unselected.contains(r#"aria-hidden="true""#));
    }

    #[test]
    fn item_indicator_drops_caller_supplied_aria_hidden() {
        let html = render(&item_indicator(
            OpenState::Open,
            vec![("aria-hidden", "false")],
            vec![],
        ));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(!html.contains(r#"aria-hidden="false""#));
    }

    #[test]
    fn value_text_placeholder_shown_only_when_true_and_reflects_disabled() {
        let placeholder = render(&value_text(
            true,
            &ListboxProps::default(),
            vec![],
            vec![text("Select a fruit")],
        ));
        assert!(placeholder.contains(r#"data-placeholder-shown="""#));
        assert!(!placeholder.contains("data-disabled"));

        let with_value = render(&value_text(
            false,
            &ListboxProps::default(),
            vec![],
            vec![text("Apple")],
        ));
        assert!(!with_value.contains("data-placeholder-shown"));

        let props = ListboxProps {
            disabled: true,
            ..ListboxProps::default()
        };
        let disabled_html = render(&value_text(false, &props, vec![], vec![]));
        assert!(disabled_html.contains(r#"data-disabled="""#));
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
        let props = ListboxProps::default();

        let root_html = render(&l.root(&props, vec![], vec![]));
        assert!(root_html.contains(r#"data-state="open""#));

        let selected_item_html =
            render(&l.item("apple", &props, false, false, None, vec![], vec![]));
        assert!(selected_item_html.contains(r#"aria-selected="true""#));

        let unselected_item_html =
            render(&l.item("banana", &props, false, false, None, vec![], vec![]));
        assert!(unselected_item_html.contains(r#"aria-selected="false""#));

        let item_text_html =
            render(&l.item_text("apple", &props, false, false, None, vec![], vec![]));
        assert!(item_text_html.contains(r#"data-state="open""#));

        let indicator_html = render(&l.item_indicator("apple", vec![], vec![]));
        assert!(!indicator_html.contains("hidden=\"\""));

        let value_text_html = render(&l.value_text(&props, vec![], vec![text("Apple")]));
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
        let props = ListboxProps::default();

        let root_html = render(&m.root(&props, vec![], vec![]));
        assert!(root_html.contains(r#"data-state="open""#));

        let selected_item_html =
            render(&m.item("apple", &props, false, false, None, vec![], vec![]));
        assert!(selected_item_html.contains(r#"aria-selected="true""#));

        let value_text_html = render(&m.value_text(&props, vec![], vec![text("2 selected")]));
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

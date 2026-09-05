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
//!   ラベルテキスト/[`item`] の `id`/[`content`] の `activedescendant`/
//!   呼び出し側 `attrs`/`children`）は [`fandhe_frontend_core::render`] の
//!   既定エスケープを必ず経由する。`raw_html()` は使用せず、HTML 文字列を
//!   直接組み立てない。
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
//! - **highlight 移動・typeahead・キーボードナビゲーション**: [`item`] の
//!   `highlighted` 引数・[`content`] の `activedescendant` 引数は
//!   `data-highlighted`/`aria-activedescendant` の SSR 静的表現（イシュー
//!   #599）を提供するのみであり、ハイライト位置の移動・typeahead・
//!   キーボード操作自体は引き続き CSR 挙動層（wasm 層の Phase 1
//!   キーボードナビゲーション実装）のスコープである。
//! - **trigger の combobox 化**（`role="combobox"` + trigger 側
//!   `aria-activedescendant` の select-only combobox パターン）: 現行の
//!   anatomy（`trigger` は素の `button`、`aria-haspopup="listbox"` のみ）の
//!   変更を伴うため別イシューのスコープとする（#599 では [`content`]
//!   （`role="listbox"`）側にのみ `aria-activedescendant` を配線する）。
//! - **`closeOnSelect` 以外の close 制御・lazyMount・portal**: クライアント
//!   ランタイム側のイベント処理・DOM 操作であり、wasm 層の将来イシューの
//!   スコープ。
//!
//! 位置決めロジック（Floating UI 相当の placement / `sameWidth` / CSS 変数
//! 出力）は本イシュー（#541）時点ではスコープ外だったが、イシュー #590
//! （親 #588）で [`crate::positioning`] として実装済みである。詳細は
//! [`positioner`] の doc を参照（Select は arrow を持たないため
//! `data-side`/`data-align` のみを出力する、ADR §4.2）。
//!
//! # 参照突合（イシュー #1619）
//!
//! ark-ui Select（zag.js）/ Radix Primitives Select の Data Attributes・
//! Keyboard Support 表と突合し、以下を是正した（同 Phase の combobox #1605・
//! listbox #1611・radio-group #1616 と同型のパターン）。
//!
//! - **[`SelectProps`]（新設）**: `disabled`/`readonly`/`invalid`/`required`
//!   の状態束を root/label/control/trigger/value-text/clear-trigger/
//!   indicator/item-group へ一律付与する（[`crate::combobox::ComboboxProps`]
//!   と同型）。呼び出し側 `attrs` に同名キーが混入していても
//!   [`drop_reserved`] で fail-closed に除去する。
//! - **[`trigger`] の `data-placeholder-shown`**: ark-ui/Radix 双方が trigger
//!   に持つ属性で、未選択時のスタイル分岐を [`value_text`] だけでなく
//!   trigger 自体でも可能にする。
//! - **[`item`] の root disabled 伝播・`data-selected`**: 有効 disabled は
//!   `props.disabled || disabled`（zag の `getItemState` 準拠、
//!   [`crate::listbox`] と同じ契約）。選択時のみ `data-selected` 存在属性を
//!   追加する（ark 互換セレクタ、`data-state` の `checked`/`unchecked` 化は
//!   非採用のまま）。
//! - **[`item_text`] の 3 状態属性**: [`item`] の先頭 3 引数
//!   （selected_state/disabled/highlighted）を同型で受け取り
//!   `data-state`/`data-disabled`/`data-highlighted` を出力する
//!   （[`crate::listbox::item_text`] と同型。select と listbox は
//!   [`SingleSelect`] 語彙を共有する兄弟のため combobox ではなく listbox の
//!   判断に揃える）。
//! - **[`item_group_label`] の `role="presentation"`**・**[`item_indicator`]
//!   の `aria-hidden="true"`**: zag の anatomy に合わせて固定付与する
//!   （[`crate::listbox`] と同型）。
//! - **[`hidden_select`] の `required`**: `disabled: bool` 引数を
//!   `&SelectProps` へ置換し、`props.required` でネイティブ `required` を
//!   追加付与する（`readonly` は `<select readonly>` が無効な HTML のため
//!   非採用、[`crate::field::select`] と同じ結論）。
//! - **readonly 中のキーボード操作抑止**: `fandhe-frontend-wasm-full` の
//!   `keynav` モジュールが trigger の `data-readonly` を確認して no-op に
//!   する（combobox #1605 の codex-review P1 是正と同型）。
//!
//! **意図的非追随**: anatomy 15 パーツは ark-ui と完全一致のため追加・削除
//! なし。Radix 固有の Portal/Viewport/ScrollButton（レイアウト計測の関心、
//! `docs/policy/intentional-non-adoption.md` §3.25 規則 2）・Arrow（Select は
//! arrow を持たない、ADR §4.2）・Icon/Value（Indicator/ValueText と同義）・
//! Separator（別部品の責務）は追加しない。`data-focus`（DOM ローカル
//! focus、§3.25 規則 2）・`data-placement`/`data-side`（[`crate::positioning`]
//! 経由で既に提供）・`data-activedescendant`（`aria-activedescendant` と
//! 重複）も追加しない。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{
    aria_activedescendant, aria_controls, aria_disabled, aria_expanded, aria_haspopup, aria_hidden,
    aria_labelledby, aria_selected, role, AriaPopup,
};
use crate::data_attrs::{
    data_disabled, data_highlighted, data_invalid, data_readonly, data_required, data_state,
};
use crate::state::{Disclosure, OpenState, SingleSelect, SingleSelectAction};
use fandhe_frontend_core::{el, text, Node, BIND_TEXT_ATTR};
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Select の anatomy（`data-scope="select"`）。
const ANATOMY: Anatomy = anatomy("select");

/// [`value_text`] が発行する `data-bind-text` 束縛点のフィールド名
/// （イシュー #642）。
///
/// `fandhe-frontend-wasm-full` の `headless_select` モジュール（クライアント
/// 側で select dispatch 後にラベルを再同期する配線層）が
/// [`fandhe_frontend_core::BIND_TEXT_ATTR`] 経由でこの値と一致する
/// field を対象に `set_text_content` する契約であり、本定数が両クレート間の
/// 唯一の合わせ込み箇所である（ドリフト検知は wasm-full 側の native テスト
/// が担う）。値そのものは変更しても外部から不透明な識別子であり HTML
/// として解釈されない。
pub const VALUE_TEXT_FIELD: &str = "select-value-text";

/// Select の disabled/readonly/invalid/required 状態束（イシュー #1619
/// 参照突合）。root/label/control/trigger/value-text/clear-trigger/
/// indicator/item-group へ [`data_disabled`]/[`data_invalid`]/
/// [`data_readonly`] を一律付与し、[`label`] にのみ [`data_required`] を
/// 追加で付与するために使う（[`crate::combobox::ComboboxProps`]/
/// [`crate::listbox::ListboxProps`] と同型のパターン）。状態機械 [`Select`]
/// にはフィールドを持たせず、呼び出しごとに `&SelectProps` を渡す
/// （hydration 属性面を拡張しない設計）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SelectProps {
    /// 無効化状態。`true` で `data-disabled` を各パーツへ付与し、
    /// [`item`]/[`item_text`] へは `props.disabled || 個別の disabled` として
    /// 伝播する（zag の `getItemState` 準拠、モジュール冒頭「参照突合」節
    /// 参照）。
    pub disabled: bool,
    /// 読み取り専用状態。`true` で `data-readonly` を各パーツへ付与する。
    /// trigger の keydown/click 経路は `fandhe-frontend-wasm-full` が
    /// `data-readonly` を確認して no-op にする（モジュール冒頭「参照突合」
    /// 節参照）。
    pub readonly: bool,
    /// 入力検証エラー状態。`true` で `data-invalid` を各パーツへ付与する。
    pub invalid: bool,
    /// 入力必須状態。`true` で [`label`] に `data-required` を、
    /// [`hidden_select`] にはネイティブ `required` 存在属性を付与する。
    pub required: bool,
}

/// [`SelectProps`] から root/label/control/trigger/value-text/
/// clear-trigger/indicator/item-group 共通の状態属性列を組み立てる非公開
/// ヘルパ（disabled/invalid/readonly の 3 属性、[`crate::combobox::state_attrs`]
/// と同型）。
fn state_attrs(props: &SelectProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs: Vec<(&'static str, &'static str)> = Vec::new();
    attrs.extend(data_disabled(props.disabled));
    attrs.extend(data_invalid(props.invalid));
    attrs.extend(data_readonly(props.readonly));
    attrs
}

/// [`root`]/[`control`] が固定付与するキー一覧（[`SelectProps`] の
/// `data-disabled`/`data-invalid`/`data-readonly` に `data-state` を
/// 加えたもの、[`crate::combobox::STATEFUL_CONTAINER_RESERVED`] と同型）。
const STATEFUL_CONTAINER_RESERVED: &[&str] = &[
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-state",
];

/// [`label`] が固定付与するキー一覧（[`SelectProps`] の状態束に
/// `data-required` を加えたもの）。
const LABEL_RESERVED: &[&str] = &[
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-required",
];

/// [`trigger`] が固定付与するキー一覧（[`STATEFUL_CONTAINER_RESERVED`] に
/// `data-placeholder-shown` を加えたもの）。
const TRIGGER_RESERVED: &[&str] = &[
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-state",
    "data-placeholder-shown",
];

/// [`item_group`] が固定付与するキー一覧。
const ITEM_GROUP_RESERVED: &[&str] = &[
    "role",
    "aria-labelledby",
    "data-disabled",
    "data-invalid",
    "data-readonly",
];

/// [`item_group_label`] が固定付与するキー一覧。
const ITEM_GROUP_LABEL_RESERVED: &[&str] = &["role", "id"];

/// [`item`] が固定付与するキー一覧。
const ITEM_RESERVED: &[&str] = &[
    "role",
    "aria-selected",
    "data-state",
    "data-selected",
    "data-value",
    "aria-disabled",
    "data-disabled",
    "data-highlighted",
    "id",
];

/// [`item_text`] が固定付与するキー一覧。
const ITEM_TEXT_RESERVED: &[&str] = &["data-state", "data-disabled", "data-highlighted", "id"];

/// [`item_indicator`] が固定付与するキー一覧。
const ITEM_INDICATOR_RESERVED: &[&str] = &["aria-hidden", "data-state", "hidden"];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（[`crate::combobox::drop_reserved`]/[`crate::listbox::drop_reserved`]
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

/// Root パーツ（`div`）。listbox の開閉状態と [`SelectProps`] の状態束を
/// `data-*` へ反映する。
#[must_use]
pub fn root<'a>(
    state: OpenState,
    props: &SelectProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATEFUL_CONTAINER_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。`id` が `Some` のとき [`content`]/[`trigger`] の
/// `labelledby` と対で `aria-labelledby` 関連付けを成立させる。
/// [`SelectProps`] の状態束 + `data-required` を付与する（イシュー #1619
/// 参照突合。ark-ui の `data-required` は Label のみが持つ）。
#[must_use]
pub fn label<'a>(
    props: &SelectProps,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, LABEL_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(data_required(props.required));
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("label", "label", merged, children)
}

/// Control パーツ（`div`）。トリガー・値表示・クリアボタン等をまとめる
/// コンテナ。開閉状態と [`SelectProps`] の状態束を `data-*` へ反映する
/// 最小主義な装飾用パーツ。
#[must_use]
pub fn control<'a>(
    state: OpenState,
    props: &SelectProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATEFUL_CONTAINER_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// Trigger パーツ（`button`）。
///
/// フォーム内配置時の意図しない submit を防ぐため `type="button"` を固定で
/// 付与する（A05 セキュリティ設定ミス対策、既存コンポーネントと同判断）。
/// `aria-haspopup="listbox"` を固定付与し、`controls` が `Some` のとき
/// `aria-controls` で [`content`] と、`labelledby` が `Some` のとき
/// `aria-labelledby` で [`label`] と関連付ける。[`SelectProps`] の状態束を
/// 付与し、`props.disabled` のときネイティブ `disabled` 存在属性も追加する。
/// `placeholder_shown` が `true` のとき `data-placeholder-shown` を付与する
/// （ark-ui/Radix 双方が trigger に持つ属性、イシュー #1619 参照突合。
/// [`value_text`] と併用する）。
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn trigger<'a>(
    state: OpenState,
    props: &SelectProps,
    placeholder_shown: bool,
    controls: Option<&'a str>,
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, TRIGGER_RESERVED);
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
    if placeholder_shown {
        merged.push(("data-placeholder-shown", ""));
    }
    merged.extend(state_attrs(props));
    if props.disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// ValueText パーツ（`span`）。`data-part="value-text"`（ark-ui 準拠の
/// kebab-case）。プレースホルダー表示中（未選択）のときのみ
/// `data-placeholder-shown` 存在属性を付与する。[`SelectProps`] の
/// `disabled`/`invalid` を `data-*` へ反映する（`data-readonly` は本パーツが
/// 操作対象ではないため付与しない、ark-ui の Data Attributes 表準拠）。
///
/// [`VALUE_TEXT_FIELD`] を field とする `data-bind-text` 束縛マーカー
/// （[`fandhe_frontend_core::BIND_TEXT_ATTR`]）を常時付与する（イシュー
/// #642）。これにより `fandhe-frontend-wasm-full` の `headless_select` 配線層
/// が select/deselect dispatch 後にラベルテキストを
/// `fandhe_frontend_wasm_client::binding_dom::BindingTable::apply_dirty`
/// 経由（`set_text_content` のみ、`innerHTML` 不使用）で再同期できる。
/// 呼び出し側 `attrs` に同名マーカーが混入していても
/// `fandhe_frontend_core::bind::bind_text` と同じ「retain で除去してから
/// 末尾へ 1 個だけ付与する」契約に従い、`data-bind-text` の重複（先頭のみ
/// 有効になり残りが黙って無視される不整合）を防ぐ。
#[must_use]
pub fn value_text<'a>(
    placeholder_shown: bool,
    props: &SelectProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(
        attrs,
        &["data-disabled", "data-invalid", "data-placeholder-shown"],
    );
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if placeholder_shown {
        merged.push(("data-placeholder-shown", ""));
    }
    merged.extend(data_disabled(props.disabled));
    merged.extend(data_invalid(props.invalid));
    let mut attrs = attrs;
    attrs.retain(|(name, _)| *name != BIND_TEXT_ATTR);
    merged.extend(attrs);
    merged.push((BIND_TEXT_ATTR, VALUE_TEXT_FIELD));
    ANATOMY.part("value-text", "span", merged, children)
}

/// ClearTrigger パーツ（`button`）。`data-part="clear-trigger"`（ark-ui 準拠の
/// kebab-case）。[`trigger`] と同じくフォーム内配置時の意図しない submit を
/// 防ぐため `type="button"` を固定で付与する。アクセシブルネーム
/// （`aria-label` 等）は本関数の `attrs` を通じて呼び出し側が付与する責務と
/// する（[`crate::popover::close_trigger`] と同じ判断）。[`SelectProps`] の
/// `invalid` を `data-invalid` へ反映し、`props.disabled` のときのみ
/// ネイティブ `disabled` 存在属性と `data-disabled` を追加する（無効な
/// select はクリアもできない安全側の判断、[`crate::combobox::clear_trigger`]
/// と同型、イシュー #1619 参照突合）。
#[must_use]
pub fn clear_trigger<'a>(
    props: &SelectProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, &["data-disabled", "data-invalid", "disabled"]);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    merged.extend(data_invalid(props.invalid));
    if props.disabled {
        merged.push(("disabled", ""));
        merged.extend(data_disabled(true));
    }
    merged.extend(attrs);
    ANATOMY.part("clear-trigger", "button", merged, children)
}

/// Indicator パーツ（`span`）。開閉状態と [`SelectProps`] の状態束を
/// `data-*` へ反映する（アイコン等は呼び出し側の `attrs`/`children` が担う）。
#[must_use]
pub fn indicator<'a>(
    state: OpenState,
    props: &SelectProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATEFUL_CONTAINER_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("indicator", "span", merged, children)
}

/// Positioner パーツ（`div`）。位置決めロジックのコンテナ。開閉状態を
/// `data-*` へ反映し、closed のとき `hidden` 存在属性を付与することで
/// [`content`] を含めて SSR/no-JS マークアップから閉状態を表現する
/// （Popover/Tooltip の `positioner` と同じ判断）。placement 計算自体は
/// [`crate::positioning::compute_position`]（#590）が担い、算出された
/// `style`（`--fandhe-*` CSS 変数、arrow 座標は含まない）・`data-side`/
/// `data-align` は呼び出し側が `attrs` 経由で渡す。
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
///
/// `activedescendant` が `Some` のとき `aria-activedescendant` を付与し、
/// 値は現在ハイライト中の [`item`] の `id` と対応させる（イシュー #599）。
/// `aria-activedescendant` は composite ロール（`listbox`/`combobox` 等）に
/// のみ有効な属性であり、本パーツが `role="listbox"` を持つため配線先に
/// 選んでいる（[`trigger`] は素の `button` のため付与しない。モジュール doc
/// §out-of-scope 参照）。ハイライト位置自体の移動・キーボードナビゲーション
/// は CSR 挙動層（wasm 層）のスコープであり、本関数は SSR 静的表現のみを
/// 提供する。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    id: Option<&'a str>,
    labelledby: Option<&'a str>,
    activedescendant: Option<&'a str>,
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
    if let Some(activedescendant) = activedescendant {
        merged.push(aria_activedescendant(activedescendant));
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
/// [`SelectProps`] の状態束を `data-*` へ反映する（[`crate::listbox::item_group`]
/// と同型、イシュー #1619 参照突合）。
#[must_use]
pub fn item_group<'a>(
    props: &SelectProps,
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_GROUP_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = state_attrs(props);
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
/// ItemGroupLabel anatomy に合わせる、[`crate::listbox::item_group_label`]
/// と同型、イシュー #1619 参照突合）。
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
/// （モジュール doc §out-of-scope 参照。ark-ui の `checked`/`unchecked` は
/// 採用しない）。選択時のみ `data-selected` 存在属性を追加する（ark 互換の
/// 属性セレクタ、イシュー #1619 参照突合）。`value` は `data-value` として
/// 動的値のまま出力し、`render()` の既定エスケープを必ず経由する。
///
/// 有効な disabled は `props.disabled || disabled`（zag の `getItemState`
/// と同じ「root disabled が子 item へ伝播する」契約、
/// [`crate::listbox::item`] と同型）で判定し、`true` のとき
/// `aria-disabled="true"` と `data-disabled` を対で付与する（本パーツは
/// `div[role="option"]` でありネイティブの `disabled` 属性を持たないため、
/// 支援技術へは ARIA 経由でのみ伝達できる。[`crate::menu::item`] と同じ
/// 判断）。
///
/// `highlighted`（キーボードナビゲーション等によるフォーカス位置）は
/// クライアントランタイムの領域だが、SSR でも `data-highlighted` を出力
/// できるよう `bool` 引数として受ける（状態機械には持たせない。
/// [`crate::menu::item`] と同じ契約、イシュー #599）。`id` が `Some` の
/// とき、[`content`] の `activedescendant` 引数の参照先として使う識別子
/// になる（`aria-activedescendant` は対象要素の `id` を参照する属性のため）。
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn item<'a>(
    selected_state: OpenState,
    props: &SelectProps,
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
/// として出力する（[`crate::listbox::item_text`] と同型、イシュー #1619
/// 参照突合）。有効な disabled は [`item`] と同じ `props.disabled ||
/// disabled` を本関数の内部で計算する。
#[must_use]
pub fn item_text<'a>(
    selected_state: OpenState,
    props: &SelectProps,
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
/// [`crate::accordion::item_content`] の `hidden` 判断と同型）。
/// `aria-hidden="true"` を固定付与する（装飾アイコンであり `item` 自身の
/// `aria-selected` が選択状態を既に伝達するため、支援技術の二重読み上げを
/// 防ぐ。[`crate::listbox::item_indicator`] と同型、イシュー #1619 参照突合）。
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

/// HiddenSelect パーツ（`select`）。フォーム統合用のネイティブ `<select>`。
///
/// `aria-hidden="true"` + `tabindex="-1"` を固定付与し、支援技術・フォーカスの
/// 両方から二重露出しないようにする（A05 セキュリティ設定ミス対策の一環。
/// 視覚的な UI は [`trigger`]/[`content`] 側が担い、本パーツはフォーム
/// 送信のためだけに存在する）。`options` は `(value, label)` の列であり、
/// 各要素は `el("option", ..)` として組み立てる。`selected` と `value` が
/// 一致する option にのみ `selected` 存在属性を付与する。値・ラベルは
/// いずれも動的だが `render()` の既定エスケープを必ず経由する。
///
/// `selected` が `None`（未選択・deselect 後・placeholder 表示中）のときは
/// `value=""` の非表示 placeholder option（`selected` かつ `disabled` 存在
/// 属性つき）を先頭へ挿入する。HTML の `<select>` 仕様上、選択済み option が
/// 一つもない場合ブラウザは自動的に先頭の有効 option を選択済み扱いにして
/// フォーム送信してしまうため、これを行わないと未選択状態にもかかわらず
/// 先頭の実 option 値が送信され、呼び出し側（`fandhe-frontend-pre-styled-ui`
/// 等）が前提とする「未選択なら空値」というフォーム連携契約が壊れる。
///
/// `disabled: bool` 引数を `&SelectProps` へ置換し、`props.disabled`/
/// `props.required` をそれぞれネイティブ `disabled`/`required` 存在属性へ
/// 反映する（イシュー #1619 参照突合。`readonly` は `<select readonly>` が
/// 無効な HTML のため非採用、[`crate::field::select`] と同じ結論）。
#[must_use]
pub fn hidden_select<'a>(
    selected: Option<&'a str>,
    name: Option<&'a str>,
    props: &SelectProps,
    attrs: Vec<(&'a str, &'a str)>,
    options: Vec<(&'a str, &'a str)>,
) -> Node {
    let attrs = drop_reserved(
        attrs,
        &["aria-hidden", "tabindex", "name", "disabled", "required"],
    );
    let mut merged: Vec<(&'a str, &'a str)> = vec![("aria-hidden", "true"), ("tabindex", "-1")];
    if let Some(name) = name {
        merged.push(("name", name));
    }
    if props.disabled {
        merged.push(("disabled", ""));
    }
    if props.required {
        merged.push(("required", ""));
    }
    merged.extend(attrs);

    let mut option_nodes: Vec<Node> = Vec::with_capacity(options.len() + 1);
    if selected.is_none() {
        option_nodes.push(el(
            "option",
            vec![("value", ""), ("selected", ""), ("disabled", "")],
            vec![],
        ));
    }
    option_nodes.extend(options.into_iter().map(|(value, option_label)| {
        let mut option_attrs: Vec<(&'a str, &'a str)> = vec![("value", value)];
        if selected == Some(value) {
            option_attrs.push(("selected", ""));
        }
        el("option", option_attrs, vec![text(option_label)])
    }));

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
    pub fn root<'a>(
        &self,
        props: &SelectProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.open_state(), props, attrs, children)
    }

    /// [`control`] へ現在の開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(
        &self,
        props: &SelectProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        control(self.open_state(), props, attrs, children)
    }

    /// [`trigger`] へ現在の開閉状態と未選択有無（プレースホルダー表示判定）を
    /// 注入する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        props: &SelectProps,
        controls: Option<&'a str>,
        labelledby: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(
            self.open_state(),
            props,
            self.selected().is_none(),
            controls,
            labelledby,
            attrs,
            children,
        )
    }

    /// [`value_text`] へ現在の選択有無（未選択ならプレースホルダー表示）を
    /// 注入する利便メソッド。
    #[must_use]
    pub fn value_text<'a>(
        &self,
        props: &SelectProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        value_text(self.selected().is_none(), props, attrs, children)
    }

    /// [`indicator`] へ現在の開閉状態を注入する利便メソッド。
    #[must_use]
    pub fn indicator<'a>(
        &self,
        props: &SelectProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        indicator(self.open_state(), props, attrs, children)
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
        activedescendant: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(
            self.open_state(),
            id,
            labelledby,
            activedescendant,
            attrs,
            children,
        )
    }

    /// [`item`] へ項目 `value` の現在の選択状態を注入する利便メソッド。
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn item<'a>(
        &self,
        value: &'a str,
        props: &SelectProps,
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
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn item_text<'a>(
        &self,
        value: &str,
        props: &SelectProps,
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

    /// [`hidden_select`] へ現在の選択値を注入する利便メソッド。
    #[must_use]
    pub fn hidden_select<'a>(
        &'a self,
        name: Option<&'a str>,
        props: &SelectProps,
        attrs: Vec<(&'a str, &'a str)>,
        options: Vec<(&'a str, &'a str)>,
    ) -> Node {
        hidden_select(self.selected(), name, props, attrs, options)
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
        let props = SelectProps::default();
        self.root(
            &props,
            Vec::new(),
            vec![
                trigger(
                    state,
                    &props,
                    self.selected().is_none(),
                    None,
                    None,
                    Vec::new(),
                    Vec::new(),
                ),
                positioner(
                    state,
                    Vec::new(),
                    vec![content(state, None, None, None, Vec::new(), Vec::new())],
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

    // --- positioning（#590）接続: Select は arrow を持たないため
    // has_arrow=false で呼び出す（ADR §4.2） ---

    #[test]
    fn positioner_accepts_computed_style_and_placement_attrs_via_attrs() {
        use crate::positioning::{
            compute_position, css_vars_style, placement_attrs, Align, Placement, PositioningConfig,
            Rect, Side, Size,
        };

        let anchor = Rect {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 20.0,
        };
        let floating = Size {
            width: 200.0,
            height: 80.0,
        };
        let viewport = Size {
            width: 800.0,
            height: 600.0,
        };
        let config = PositioningConfig {
            placement: Placement::new(Side::Bottom, Align::Center),
            offset: 0.0,
            flip: true,
            shift: true,
            same_width: true,
        };
        let resolved = compute_position(anchor, floating, viewport, &config, false);
        let style = css_vars_style(&resolved, anchor.width, config.same_width);
        let mut attrs: Vec<(&str, &str)> = vec![("style", &style)];
        attrs.extend(placement_attrs(resolved.placement));

        let html = render(&positioner(OpenState::Open, attrs, vec![]));
        assert!(html.contains("--fandhe-reference-width:"));
        assert!(!html.contains("--fandhe-arrow-x:"));
        assert!(html.contains(r#"data-side="bottom""#));
        assert!(html.contains(r#"data-align="center""#));
    }

    // --- 各パーツの data-scope/data-part/data-state 出力 ---

    #[test]
    fn root_outputs_scope_part_and_state() {
        let html = render(&root(
            OpenState::Closed,
            &SelectProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="select""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn label_id_some_outputs_id() {
        let html = render(&label(
            &SelectProps::default(),
            Some("select-label-1"),
            vec![],
            vec![text("Fruit")],
        ));
        assert!(html.contains(r#"<label"#));
        assert!(html.contains(r#"id="select-label-1""#));
    }

    #[test]
    fn control_outputs_scope_part_and_state() {
        let html = render(&control(
            OpenState::Open,
            &SelectProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-part="control""#));
        assert!(html.contains(r#"data-state="open""#));
    }

    #[test]
    fn trigger_has_type_button_haspopup_listbox_and_aria_expanded() {
        let html = render(&trigger(
            OpenState::Closed,
            &SelectProps::default(),
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

        let html_open = render(&trigger(
            OpenState::Open,
            &SelectProps::default(),
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html_open.contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn trigger_controls_and_labelledby_some_outputs_both() {
        let html = render(&trigger(
            OpenState::Closed,
            &SelectProps::default(),
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
        let props = SelectProps {
            disabled: true,
            ..SelectProps::default()
        };
        let html = render(&trigger(
            OpenState::Closed,
            &props,
            false,
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
            &SelectProps::default(),
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
    fn trigger_placeholder_shown_true_adds_marker_false_omits() {
        // ark-ui/Radix 双方が trigger に持つ属性（イシュー #1619 参照突合）。
        let shown = render(&trigger(
            OpenState::Closed,
            &SelectProps::default(),
            true,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(shown.contains(r#"data-placeholder-shown="""#));

        let not_shown = render(&trigger(
            OpenState::Closed,
            &SelectProps::default(),
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(!not_shown.contains("data-placeholder-shown"));
    }

    #[test]
    fn trigger_invalid_and_readonly_output_data_attrs() {
        let invalid_props = SelectProps {
            invalid: true,
            ..SelectProps::default()
        };
        let invalid_html = render(&trigger(
            OpenState::Closed,
            &invalid_props,
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(invalid_html.contains(r#"data-invalid="""#));

        let readonly_props = SelectProps {
            readonly: true,
            ..SelectProps::default()
        };
        let readonly_html = render(&trigger(
            OpenState::Closed,
            &readonly_props,
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(readonly_html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn label_required_and_state_attrs() {
        let props = SelectProps {
            required: true,
            disabled: true,
            invalid: true,
            readonly: true,
        };
        let html = render(&label(&props, Some("l1"), vec![], vec![text("Fruit")]));
        assert!(html.contains(r#"data-required="""#));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-readonly="""#));

        let default_html = render(&label(&SelectProps::default(), None, vec![], vec![]));
        assert!(!default_html.contains("data-required"));
    }

    #[test]
    fn value_text_placeholder_shown_only_when_true() {
        let placeholder = render(&value_text(
            true,
            &SelectProps::default(),
            vec![],
            vec![text("Select a fruit")],
        ));
        assert!(placeholder.contains(r#"data-placeholder-shown="""#));

        let with_value = render(&value_text(
            false,
            &SelectProps::default(),
            vec![],
            vec![text("Apple")],
        ));
        assert!(!with_value.contains("data-placeholder-shown"));
    }

    #[test]
    fn value_text_disabled_and_invalid_output_data_attrs() {
        let props = SelectProps {
            disabled: true,
            invalid: true,
            ..SelectProps::default()
        };
        let html = render(&value_text(false, &props, vec![], vec![text("Apple")]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(!html.contains("data-readonly"));
    }

    #[test]
    fn value_text_carries_bind_text_marker_for_client_sync() {
        // イシュー #642: wasm-full の headless_select 配線層が select/deselect
        // dispatch 後にこのマーカーを頼りにラベルを再同期する。field 名が
        // VALUE_TEXT_FIELD と一致することが両クレート間の契約。
        let html = render(&value_text(
            false,
            &SelectProps::default(),
            vec![],
            vec![text("Apple")],
        ));
        assert!(html.contains(&format!(r#"data-bind-text="{VALUE_TEXT_FIELD}""#)));
    }

    #[test]
    fn value_text_normalizes_caller_supplied_duplicate_bind_text_marker() {
        // 呼び出し側が誤って同名マーカーを attrs へ渡しても 1 個に正規化され、
        // 先頭のみ有効になる不整合（HTML パース時の属性重複）を防ぐ。
        let html = render(&value_text(
            false,
            &SelectProps::default(),
            vec![("data-bind-text", "caller-supplied-stale-field")],
            vec![text("Apple")],
        ));
        assert_eq!(html.matches("data-bind-text").count(), 1);
        assert!(html.contains(&format!(r#"data-bind-text="{VALUE_TEXT_FIELD}""#)));
        assert!(!html.contains("caller-supplied-stale-field"));
    }

    #[test]
    fn clear_trigger_has_type_button_and_kebab_case_part() {
        let html = render(&clear_trigger(&SelectProps::default(), vec![], vec![]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-part="clear-trigger""#));
    }

    #[test]
    fn clear_trigger_disabled_true_adds_native_and_data_disabled() {
        let props = SelectProps {
            disabled: true,
            ..SelectProps::default()
        };
        let html = render(&clear_trigger(&props, vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn clear_trigger_invalid_outputs_data_invalid() {
        let props = SelectProps {
            invalid: true,
            ..SelectProps::default()
        };
        let html = render(&clear_trigger(&props, vec![], vec![]));
        assert!(html.contains(r#"data-invalid="""#));
    }

    #[test]
    fn indicator_outputs_scope_part_and_state_only() {
        let html = render(&indicator(
            OpenState::Open,
            &SelectProps::default(),
            vec![],
            vec![text("v")],
        ));
        assert!(html.contains(r#"data-scope="select""#));
        assert!(html.contains(r#"data-part="indicator""#));
        assert!(html.contains(r#"data-state="open""#));
    }

    #[test]
    fn indicator_state_attrs_reflect_props() {
        let props = SelectProps {
            disabled: true,
            invalid: true,
            readonly: true,
            required: false,
        };
        let html = render(&indicator(OpenState::Open, &props, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-readonly="""#));
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
        let html = render(&content(OpenState::Open, None, None, None, vec![], vec![]));
        assert!(html.contains(r#"role="listbox""#));
        assert!(html.contains(r#"data-state="open""#));
    }

    #[test]
    fn content_closed_has_hidden_attr_open_does_not() {
        let closed = render(&content(
            OpenState::Closed,
            None,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&content(OpenState::Open, None, None, None, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_id_and_labelledby_some_outputs_both() {
        let html = render(&content(
            OpenState::Open,
            Some("select-content-1"),
            Some("select-label-1"),
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="select-content-1""#));
        assert!(html.contains(r#"aria-labelledby="select-label-1""#));
    }

    #[test]
    fn content_activedescendant_some_outputs_attr_none_omits() {
        let html = render(&content(
            OpenState::Open,
            None,
            None,
            Some("item-vue"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-activedescendant="item-vue""#));

        let without = render(&content(OpenState::Open, None, None, None, vec![], vec![]));
        assert!(!without.contains("aria-activedescendant"));
    }

    #[test]
    fn item_group_labelledby_some_outputs_role_group_and_aria_labelledby_together() {
        let html = render(&item_group(
            &SelectProps::default(),
            Some("group-label-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"aria-labelledby="group-label-1""#));
    }

    #[test]
    fn item_group_labelledby_none_omits_role_and_aria_labelledby() {
        let html = render(&item_group(&SelectProps::default(), None, vec![], vec![]));
        assert!(!html.contains("role="));
        assert!(!html.contains("aria-labelledby"));
    }

    #[test]
    fn item_group_disabled_true_outputs_data_disabled() {
        let props = SelectProps {
            disabled: true,
            ..SelectProps::default()
        };
        let html = render(&item_group(&props, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_group_label_id_some_outputs_id_and_fixed_role_presentation() {
        let html = render(&item_group_label(Some("group-label-1"), vec![], vec![]));
        assert!(html.contains(r#"id="group-label-1""#));
        assert!(html.contains(r#"role="presentation""#));
    }

    #[test]
    fn item_group_label_drops_caller_supplied_role() {
        let html = render(&item_group_label(None, vec![("role", "attacker")], vec![]));
        assert!(html.contains(r#"role="presentation""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn item_has_role_option_aria_selected_and_data_value() {
        let html = render(&item(
            OpenState::Open,
            &SelectProps::default(),
            false,
            false,
            "vue",
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="option""#));
        assert!(html.contains(r#"aria-selected="true""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-value="vue""#));
        assert!(html.contains(r#"data-selected="""#));

        let unselected = render(&item(
            OpenState::Closed,
            &SelectProps::default(),
            false,
            false,
            "react",
            None,
            vec![],
            vec![],
        ));
        assert!(unselected.contains(r#"aria-selected="false""#));
        assert!(unselected.contains(r#"data-state="closed""#));
        assert!(!unselected.contains("data-selected"));
    }

    #[test]
    fn item_disabled_true_adds_data_disabled() {
        let html = render(&item(
            OpenState::Closed,
            &SelectProps::default(),
            true,
            false,
            "svelte",
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_root_disabled_propagates_even_when_item_disabled_false() {
        // zag の getItemState 準拠: root disabled が子 item へ伝播する
        // （[`crate::listbox::item`] と同型、イシュー #1619 参照突合）。
        let props = SelectProps {
            disabled: true,
            ..SelectProps::default()
        };
        let html = render(&item(
            OpenState::Closed,
            &props,
            false,
            false,
            "svelte",
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn item_disabled_true_adds_aria_disabled() {
        // `div[role="option"]` はネイティブの `disabled` を持たないため、
        // 支援技術へは `aria-disabled` 経由でのみ伝達できる（Bugbot 指摘:
        // crates/headless-ui/src/select.rs#L277-L294、`menu::item` と同じ
        // 契約）。
        let html = render(&item(
            OpenState::Closed,
            &SelectProps::default(),
            true,
            false,
            "svelte",
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-disabled="true""#));

        let enabled = render(&item(
            OpenState::Closed,
            &SelectProps::default(),
            false,
            false,
            "svelte",
            None,
            vec![],
            vec![],
        ));
        assert!(!enabled.contains("aria-disabled"));
    }

    #[test]
    fn item_highlighted_true_adds_data_highlighted_false_omits() {
        let highlighted = render(&item(
            OpenState::Closed,
            &SelectProps::default(),
            false,
            true,
            "svelte",
            None,
            vec![],
            vec![],
        ));
        assert!(highlighted.contains(r#"data-highlighted="""#));

        let not_highlighted = render(&item(
            OpenState::Closed,
            &SelectProps::default(),
            false,
            false,
            "svelte",
            None,
            vec![],
            vec![],
        ));
        assert!(!not_highlighted.contains("data-highlighted"));
    }

    #[test]
    fn item_id_some_outputs_id_none_omits() {
        let with_id = render(&item(
            OpenState::Closed,
            &SelectProps::default(),
            false,
            false,
            "svelte",
            Some("item-svelte"),
            vec![],
            vec![],
        ));
        assert!(with_id.contains(r#"id="item-svelte""#));

        let without_id = render(&item(
            OpenState::Closed,
            &SelectProps::default(),
            false,
            false,
            "svelte",
            None,
            vec![],
            vec![],
        ));
        assert!(!without_id.contains(" id="));
    }

    #[test]
    fn item_text_id_some_outputs_id() {
        let html = render(&item_text(
            OpenState::Closed,
            &SelectProps::default(),
            false,
            false,
            Some("item-text-1"),
            vec![],
            vec![text("Vue")],
        ));
        assert!(html.contains(r#"id="item-text-1""#));
    }

    #[test]
    fn item_text_outputs_three_state_attrs() {
        let html = render(&item_text(
            OpenState::Open,
            &SelectProps::default(),
            false,
            true,
            None,
            vec![],
            vec![text("Vue")],
        ));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-highlighted="""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn item_text_root_disabled_propagates() {
        let props = SelectProps {
            disabled: true,
            ..SelectProps::default()
        };
        let html = render(&item_text(
            OpenState::Closed,
            &props,
            false,
            false,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_indicator_selected_shown_unselected_hidden() {
        let selected = render(&item_indicator(OpenState::Open, vec![], vec![text("✓")]));
        assert!(!selected.contains(r#" hidden"#));
        assert!(selected.contains(r#"data-state="open""#));
        assert!(selected.contains(r#"aria-hidden="true""#));

        let unselected = render(&item_indicator(OpenState::Closed, vec![], vec![]));
        assert!(unselected.contains(r#" hidden="""#));
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
    fn hidden_select_has_native_select_and_aria_hidden_tabindex() {
        let html = render(&hidden_select(
            Some("vue"),
            Some("framework"),
            &SelectProps::default(),
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
        let html = render(&hidden_select(
            None,
            None,
            &SelectProps::default(),
            vec![],
            vec![],
        ));
        assert!(!html.contains("name="));
    }

    #[test]
    fn hidden_select_disabled_true_adds_native_disabled() {
        let props = SelectProps {
            disabled: true,
            ..SelectProps::default()
        };
        let html = render(&hidden_select(None, None, &props, vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
    }

    #[test]
    fn hidden_select_required_true_adds_native_required() {
        let props = SelectProps {
            required: true,
            ..SelectProps::default()
        };
        let html = render(&hidden_select(None, None, &props, vec![], vec![]));
        assert!(html.contains(r#"required="""#));
    }

    #[test]
    fn hidden_select_unselected_does_not_default_to_first_option() {
        // 未選択（selected == None）のとき、ブラウザの暗黙選択（先頭 option
        // の自動選択）に頼らず、選択済み placeholder option を明示挿入する
        // ことで先頭の実 option 値が誤ってフォーム送信されないことを保証する
        // 回帰テスト（Bugbot 指摘: crates/headless-ui/src/select.rs#L356-L366）。
        let html = render(&hidden_select(
            None,
            Some("framework"),
            &SelectProps::default(),
            vec![],
            vec![("vue", "Vue"), ("react", "React")],
        ));
        assert!(html.contains(r#"<option value="" selected="" disabled=""></option>"#));
        assert!(!html.contains(r#"<option value="vue" selected="">Vue</option>"#));
        assert!(!html.contains(r#"<option value="react" selected="">React</option>"#));
    }

    #[test]
    fn hidden_select_selected_some_omits_placeholder_option() {
        let html = render(&hidden_select(
            Some("vue"),
            None,
            &SelectProps::default(),
            vec![],
            vec![("vue", "Vue"), ("react", "React")],
        ));
        assert!(!html.contains(r#"<option value="" selected="""#));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            OpenState::Closed,
            &SelectProps::default(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="select""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- 呼び出し側 attrs による reserved キー偽装除去（イシュー #1619） ---

    #[test]
    fn caller_supplied_reserved_state_keys_are_dropped_on_root() {
        let props = SelectProps {
            disabled: true,
            ..SelectProps::default()
        };
        let html = render(&root(
            OpenState::Closed,
            &props,
            vec![("data-disabled", "attacker")],
            vec![],
        ));
        assert_eq!(html.matches("data-disabled").count(), 1);
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn caller_supplied_reserved_placeholder_shown_is_dropped_on_trigger() {
        let html = render(&trigger(
            OpenState::Closed,
            &SelectProps::default(),
            true,
            None,
            None,
            vec![("data-placeholder-shown", "attacker")],
            vec![],
        ));
        assert_eq!(html.matches("data-placeholder-shown").count(), 1);
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

        let props = SelectProps::default();
        let item_vue = render(&s.item("vue", &props, false, false, None, vec![], vec![]));
        assert!(item_vue.contains(r#"aria-selected="true""#));

        let item_react = render(&s.item("react", &props, false, false, None, vec![], vec![]));
        assert!(item_react.contains(r#"aria-selected="false""#));

        let value_text_html = render(&s.value_text(&props, vec![], vec![]));
        assert!(!value_text_html.contains("data-placeholder-shown"));

        // select 済みなので trigger の data-placeholder-shown は出ない。
        let trigger_html = render(&s.trigger(&props, None, None, vec![], vec![]));
        assert!(!trigger_html.contains("data-placeholder-shown"));
    }

    #[test]
    fn select_value_text_shows_placeholder_when_unselected() {
        let s = Select::default();
        let html = render(&s.value_text(
            &SelectProps::default(),
            vec![],
            vec![text("Select a fruit")],
        ));
        assert!(html.contains(r#"data-placeholder-shown="""#));
    }

    #[test]
    fn select_trigger_shows_placeholder_when_unselected() {
        // Select::trigger は self.selected().is_none() を placeholder_shown へ
        // 注入する（イシュー #1619 参照突合）。
        let s = Select::default();
        let html = render(&s.trigger(&SelectProps::default(), None, None, vec![], vec![]));
        assert!(html.contains(r#"data-placeholder-shown="""#));
    }

    #[test]
    fn select_convenience_value_text_carries_bind_text_marker() {
        // `Select::value_text` は自由関数 `value_text` へ委譲するのみだが、
        // イシュー #642 の束縛マーカー付与が便宜メソッド経由でも確実に
        // 効くことを固定する（委譲経路の回帰防止）。
        let s = Select::default();
        let html = render(&s.value_text(
            &SelectProps::default(),
            vec![],
            vec![text("Select a fruit")],
        ));
        assert!(html.contains(&format!(r#"data-bind-text="{VALUE_TEXT_FIELD}""#)));
    }

    #[test]
    fn select_convenience_item_text_and_hidden_select() {
        let mut s = Select::default();
        dispatch(&mut s, "select", "vue");
        let props = SelectProps::default();

        let item_text_html =
            render(&s.item_text("vue", &props, false, false, None, vec![], vec![]));
        assert!(item_text_html.contains(r#"data-state="open""#));

        let hidden_html = render(&s.hidden_select(None, &props, vec![], vec![("vue", "Vue")]));
        assert!(hidden_html.contains(r#"<option value="vue" selected="">Vue</option>"#));
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
            &SelectProps::default(),
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
            &SelectProps::default(),
            false,
            false,
            ATTR_BREAK_PAYLOAD,
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_id_payload_is_escaped_on_render() {
        let html = render(&item(
            OpenState::Closed,
            &SelectProps::default(),
            false,
            false,
            "svelte",
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn content_activedescendant_payload_is_escaped_on_render() {
        let html = render(&content(
            OpenState::Open,
            None,
            None,
            Some(ATTR_BREAK_PAYLOAD),
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
            &SelectProps::default(),
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
            &SelectProps::default(),
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&indicator(
            OpenState::Open,
            &SelectProps::default(),
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

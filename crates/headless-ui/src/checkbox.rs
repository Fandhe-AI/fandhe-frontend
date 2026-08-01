//! Checkbox コンポーネント（ark-ui Checkbox 相当の anatomy、イシュー #535）。
//!
//! 親トラッキング #520・Phase 2・form 系イシュー #534 配下。SSR/SSG から
//! [`fandhe_frontend_core::render`] 経由で呼ばれる、[`CheckedState`] を受け取る
//! 純粋な描画関数群として実装する（REQ-5: マクロ DSL 非採用）。
//!
//! # anatomy
//!
//! ark-ui の Checkbox anatomy に倣い、以下 5 パーツで構成する。
//!
//! - [`root`][]: `<label>`。他パーツを包む起点。
//! - [`control`][]: `<div aria-hidden="true">`。視覚的なチェックボックス表現。
//!   アクセシビリティ実体は [`hidden_input`] が担うため支援技術からは隠す。
//! - [`indicator`][]: `<div>`。チェックマーク等の視覚的インジケータ。
//!   [`CheckedState::Unchecked`] のときは `hidden` 存在属性を付与する。
//! - [`label`][]: `<span>`。ラベルテキストを包むパーツ。
//! - [`hidden_input`][]: `<input type="checkbox">`。フォーム送信・
//!   アクセシビリティの実体を担うネイティブ input。
//!
//! 全パーツに `data-scope="checkbox"` と `data-part="<part>"` に加え、
//! [`CheckedState::as_data_state`] 由来の `data-state`、および
//! `disabled`/`invalid`/`required`/`readonly` の存在属性（[`crate::data_attrs`]）を
//! 一律付与する（ark-ui の「全パーツが data-state を持つ」規約）。
//!
//! # 状態機械 / dispatch 統合との境界
//!
//! パーツ関数群（[`root`]/[`control`]/[`indicator`]/[`label`]/
//! [`hidden_input`]）は [`CheckedState`] を受け取って HTML を組み立てるだけの
//! 純粋関数であり、状態そのものは保持しない（SSR/SSG 初期描画で完結する。
//! 元 #535 の実装方針をそのまま維持する）。
//!
//! クリックトグル等の動的状態遷移（`Component`/`Hydrate` + `dispatch`
//! 統合）は [`Checkbox`] が担う。[`Checkbox`] は
//! [`crate::state::Checkable`]（イシュー #524 で確立し #595 で Switch から
//! 共通化昇格した 2 値チェック状態機械）を埋め込み、dispatch 語彙
//! （`"check"`/`"uncheck"`/`"toggle"`）・fail-closed hydration を
//! [`crate::switch::Switch`] と揃える。indeterminate（3 値目）は
//! [`Checkable`](crate::state::Checkable) のスコープ外のため、[`Checkbox`]
//! の dispatch/hydration 経路では表現できない — インタラクティブな
//! tri-state 対応（プログラム的な indeterminate 設定の dispatch/hydration
//! 化）は #595 の out-of-scope（PR 本文参照）。SSR 静的 props
//! （[`CheckedState::Indeterminate`]）としての表現は本モジュールのパーツ
//! 関数群で引き続き可能。
//!
//! # フォーカスリング契約（`data-focus-visible`、イシュー #709）
//!
//! 実フォーカスは [`hidden_input`]（visually-hidden なネイティブ
//! `<input type="checkbox">`）が受けるため、[`switch`](crate::switch) と
//! 同型の hidden-input パターンに該当し、[`crate::data_attrs::data_focus_visible`]
//! を [`root`]/[`control`] へ出力できる（契約は同関数の doc を参照）。
//! `fandhe-frontend-pre-styled-ui` に Checkbox の styled ラッパーは
//! イシュー #709 時点で未実装のため、CSS 側の recipe 追加は本イシューの
//! 対象外（本 doc は headless 層の契約のみを先行して確立する）。
//!
//! # セキュリティ不変条件
//!
//! - 全パーツは [`crate::anatomy::Anatomy::part`]（内部で
//!   [`fandhe_frontend_core::el`] へ委譲）を経由する薄い委譲であり、独自の
//!   エスケープ処理・HTML 文字列直接組み立て・`raw_html()` 使用は行わない
//!   （`docs/api/component-api.md` §4 定義規則 1・2 準拠）。
//! - フレームワークが固定する属性（`data-scope`/`data-part`/`data-state`/
//!   `type`/`checked`/`aria-checked`/`aria-hidden`/`hidden`/`disabled`/
//!   `required`）は呼び出し側 `attrs` に同名キー（ASCII 大文字小文字無視）が
//!   含まれていても fail-closed で除去し、フレームワーク値を優先する
//!   （`Anatomy::part` の `data-scope`/`data-part` フィルタと同型の防御。
//!   [`fandhe_frontend_core::el`] は重複属性を除去しないため、パーツ側で
//!   多重定義を防ぐ責務を負う）。
//! - `name`/`value`・呼び出し側 `attrs` の動的値は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する
//!   （`tests/checkbox_escape.rs` で回帰固定）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_checked, aria_hidden, AriaChecked};
use crate::data_attrs::{data_disabled, data_invalid, data_readonly, data_required, data_state};
use crate::state::Checkable;
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// このコンポーネントの anatomy（`data-scope="checkbox"` を固定）。
const ANATOMY: Anatomy = anatomy("checkbox");

/// チェック状態（3 値）。`data-state` 値と `aria-checked` 値の唯一の情報源。
///
/// 任意文字列ではなく enum で値語彙を固定することで、`data-state`/
/// `aria-checked` を偽装・不整合な値にする経路を型で塞ぐ
/// （[`crate::data_attrs::Orientation`] と同型の判断）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CheckedState {
    /// 未チェック（既定値）。
    #[default]
    Unchecked,
    /// チェック済み。
    Checked,
    /// 不定状態（tri-state）。
    Indeterminate,
}

/// `"indeterminate"` の `data-state` 値。3 値目のみ本モジュール local で
/// 定義する（[`crate::state::DATA_STATE_CHECKED`]/
/// [`crate::state::DATA_STATE_UNCHECKED`] は 2 値の共通機械
/// [`crate::state::Checkable`] が管理する値であり、`"indeterminate"` を
/// 含まない。§設計判断はモジュール冒頭「状態機械 / dispatch 統合との境界」
/// 節・イシュー #595 参照）。
const DATA_STATE_INDETERMINATE: &str = "indeterminate";

impl CheckedState {
    /// `data-state` の属性値文字列を返す。`Unchecked`/`Checked` は
    /// [`crate::state::Checkable`] が一元管理する共通値語彙
    /// （[`crate::state::checked_data_state`]）を再利用し、`Indeterminate`
    /// のみ本モジュール固有の値を返す（3 値のうち 2 値は Switch/RadioGroup
    /// と共有、1 値は Checkbox 固有というイシュー #595 の設計判断を反映）。
    #[must_use]
    pub const fn as_data_state(self) -> &'static str {
        match self {
            Self::Unchecked => crate::state::DATA_STATE_UNCHECKED,
            Self::Checked => crate::state::DATA_STATE_CHECKED,
            Self::Indeterminate => DATA_STATE_INDETERMINATE,
        }
    }

    /// [`crate::aria::AriaChecked`] へ写像する。`Indeterminate` は
    /// `AriaChecked::Mixed`（`aria-checked="mixed"`）に対応する。
    #[must_use]
    pub const fn to_aria(self) -> AriaChecked {
        match self {
            Self::Unchecked => AriaChecked::False,
            Self::Checked => AriaChecked::True,
            Self::Indeterminate => AriaChecked::Mixed,
        }
    }
}

/// SSR 初期描画に必要な Checkbox の宣言的状態。
///
/// `Default` は [`CheckedState::Unchecked`] かつ全 `bool` フラグ `false`。
/// クリックトグル等の動的遷移は本構造体の責務外（#524 スコープ、モジュール
/// 冒頭コメント参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CheckboxProps {
    /// チェック状態（3 値）。
    pub checked: CheckedState,
    /// 無効化状態。`true` で `data-disabled`/`disabled`/`aria-disabled` 相当の
    /// 存在属性を各パーツへ付与する。
    pub disabled: bool,
    /// 入力検証エラー状態。`true` で `data-invalid`/`aria-invalid="true"` を
    /// 付与する。
    pub invalid: bool,
    /// 必須入力状態。`true` で `data-required`/`required` を付与する。
    pub required: bool,
    /// 読み取り専用状態。`true` で `data-readonly` を付与する
    /// （ネイティブ `readonly` 属性はチェックボックスに意味を持たないため
    /// 付与しない。ark-ui も `data-readonly` のみを付与する）。
    pub readonly: bool,
}

/// [`Checkbox`] の利便メソッドが受け取る disabled/invalid/required/readonly
/// フラグ束（`checked` は含まない — [`Checkbox::props`] が
/// `self.checkable.is_checked()` から自動算出するため呼び出し側が渡す
/// 必要はない）。4 個の独立した `bool` 引数のままだと各利便メソッドの
/// 引数数が clippy `too_many_arguments`（既定閾値 7）を超えるため、
/// [`CheckboxProps`] と対になる薄い構造体としてまとめる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CheckboxFlags {
    /// [`CheckboxProps::disabled`] 参照。
    pub disabled: bool,
    /// [`CheckboxProps::invalid`] 参照。
    pub invalid: bool,
    /// [`CheckboxProps::required`] 参照。
    pub required: bool,
    /// [`CheckboxProps::readonly`] 参照。
    pub readonly: bool,
}

/// 全パーツ共通の `data-state`/`data-disabled`/`data-invalid`/`data-required`/
/// `data-readonly` 属性列を組み立てる非公開ヘルパ。
fn state_attrs(props: &CheckboxProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs: Vec<(&'static str, &'static str)> =
        vec![data_state(props.checked.as_data_state())];
    attrs.extend(data_disabled(props.disabled));
    attrs.extend(data_invalid(props.invalid));
    attrs.extend(data_required(props.required));
    attrs.extend(data_readonly(props.readonly));
    attrs
}

/// [`state_attrs`] が全パーツへ一律付与する属性キー一覧。呼び出し側 `attrs`
/// にこれらと同名キーが含まれていても fail-closed で除去する対象
/// （モジュール冒頭「セキュリティ不変条件」参照）。`root`/`control`/
/// `indicator`/`label`/`hidden_input` の全パーツが `state_attrs` を
/// マージするため、各パーツ個別の予約リストとは別にこの共通リストを
/// 必ず適用する（適用漏れは `data-state` 等の重複属性・状態偽装を招く）。
const STATE_RESERVED: &[&str] = &[
    "data-state",
    "data-disabled",
    "data-invalid",
    "data-required",
    "data-readonly",
];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する。`Anatomy::part` の `data-scope`/`data-part` フィルタと同型の
/// fail-closed 防御であり、各パーツが追加で持つ固定属性（`type`/`checked`
/// 等）を呼び出し側の偽装値から守るために使う。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// `Root` パーツ（`<label>`）。他パーツを包む起点。
///
/// `<label>` が input を包む構造のため `for`/`id` の明示連結は不要だが、
/// 利用者が分離したい場合は `attrs` で `for`/`id` を渡せる。
#[must_use]
pub fn root<'a>(
    props: &CheckboxProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(attrs);
    ANATOMY.part("root", "label", merged, children)
}

/// `Control` パーツ（`<div aria-hidden="true">`）。視覚的なチェックボックス
/// 表現を包む。アクセシビリティ実体は [`hidden_input`] が担うため、支援技術
/// からは `aria-hidden="true"` で隠す（WAI-ARIA Checkbox パターン準拠。
/// 二重読み上げの防止）。
#[must_use]
pub fn control<'a>(
    props: &CheckboxProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(drop_reserved(attrs, STATE_RESERVED), &["aria-hidden"]);
    let mut merged = state_attrs(props);
    merged.push(aria_hidden(true));
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// `Indicator` パーツ（`<div>`）。チェックマーク等の視覚的インジケータ。
/// [`CheckedState::Unchecked`] のときは `hidden` 存在属性を付与し、
/// `Checked`/`Indeterminate` のときは表示する。
#[must_use]
pub fn indicator<'a>(
    props: &CheckboxProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(drop_reserved(attrs, STATE_RESERVED), &["hidden"]);
    let mut merged = state_attrs(props);
    if props.checked == CheckedState::Unchecked {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("indicator", "div", merged, children)
}

/// `Label` パーツ（`<span>`）。ラベルテキストを包む。
#[must_use]
pub fn label<'a>(
    props: &CheckboxProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(attrs);
    ANATOMY.part("label", "span", merged, children)
}

/// フレームワークが `hidden_input` に固定する属性キー一覧
/// （呼び出し側 `attrs` からの偽装を fail-closed で除外する対象）。
const HIDDEN_INPUT_RESERVED: &[&str] = &[
    "type",
    "checked",
    "aria-checked",
    "aria-invalid",
    "name",
    "value",
    "disabled",
    "required",
];

/// `HiddenInput` パーツ（`<input type="checkbox">`）。フォーム送信・
/// アクセシビリティの実体を担うネイティブ input。
///
/// `name`/`value` は暗黙の既定値を持たず、呼び出し側が明示する
/// （フォームフィールド名・送信値の偽装防止・明示性の確保）。
///
/// - [`CheckedState::Checked`] のとき `checked` 存在属性を付与する。
/// - [`CheckedState::Indeterminate`] のとき `aria-checked="mixed"` を付与する
///   （ネイティブ `checked` 属性では表現できない不定状態の補完。
///   `indeterminate` プロパティは実行時 DOM 操作が必要なため、SSR 初期描画
///   では ARIA での補完にとどめる）。
/// - `props.disabled`/`props.required` はネイティブ存在属性へ反映する。
/// - `props.invalid` のとき `aria-invalid="true"` を付与する。
#[must_use]
pub fn hidden_input<'a>(
    props: &CheckboxProps,
    name: &'a str,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let attrs = drop_reserved(drop_reserved(attrs, STATE_RESERVED), HIDDEN_INPUT_RESERVED);
    let mut merged = state_attrs(props);
    merged.push(("type", "checkbox"));
    merged.push(("name", name));
    merged.push(("value", value));
    if props.checked == CheckedState::Checked {
        merged.push(("checked", ""));
    }
    if props.checked == CheckedState::Indeterminate {
        merged.push(aria_checked(AriaChecked::Mixed));
    }
    if props.invalid {
        merged.push(("aria-invalid", "true"));
    }
    if props.disabled {
        merged.push(("disabled", ""));
    }
    if props.required {
        merged.push(("required", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("hidden-input", "input", merged, vec![])
}

/// Checkbox のアクション（WASM 境界の文字列 dispatch と
/// [`Checkbox::decode_action`] で接続する）。payload は使用しない。
///
/// [`crate::state::CheckableAction`] の互換 re-export（[`crate::switch::SwitchAction`]
/// と同じ様式。indeterminate へ遷移するアクションは存在しない —
/// モジュール冒頭「状態機械 / dispatch 統合との境界」節参照）。
pub use crate::state::CheckableAction as CheckboxAction;

/// [`crate::state::Checkable`]（#524 で確立・#595 で Switch から共通化
/// 昇格した 2 値チェック状態機械）を埋め込んだ Checkbox の動的状態機械。
///
/// [`CheckboxProps`]（SSR 静的 props、indeterminate を含む 3 値）とは異なり、
/// 本型は dispatch/hydration で遷移可能な checked/unchecked の 2 値のみを
/// 保持する。各パーツ関数（[`root`]/[`control`]/[`indicator`]/[`label`]/
/// [`hidden_input`]）へ現在の checked 状態から導いた [`CheckboxProps`] を
/// 注入する利便メソッドを提供する（[`crate::switch::Switch`] と同じ利便
/// メソッド様式）。`Default` は unchecked（SSR の状態なし初期描画に対応する
/// 既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Checkbox {
    checkable: Checkable,
}

impl Checkbox {
    /// `data-hydrate-checked` 属性名のフィールド部分
    /// （[`Checkable::FIELD_CHECKED`] と同一値。
    /// `docs/api/hydration-state-format.md` の `<field>` 命名規約に従う）。
    pub const FIELD_CHECKED: &'static str = Checkable::FIELD_CHECKED;

    /// 指定した初期状態で Checkbox を生成する。
    #[must_use]
    pub fn new(checked: bool) -> Self {
        Self {
            checkable: Checkable::new(checked),
        }
    }

    /// 現在チェックされているかどうか。
    #[must_use]
    pub fn is_checked(&self) -> bool {
        self.checkable.is_checked()
    }

    /// 現在の `data-state` 属性値（`"checked"`/`"unchecked"`）。
    #[must_use]
    pub fn data_state(&self) -> &'static str {
        self.checkable.data_state()
    }

    /// 現在の checked 状態と呼び出し側の [`CheckboxFlags`] から
    /// [`CheckboxProps`] を組み立てる非公開ヘルパ。indeterminate は本型の
    /// スコープ外のため常に `Checked`/`Unchecked` のいずれかになる。
    fn props(&self, flags: CheckboxFlags) -> CheckboxProps {
        CheckboxProps {
            checked: if self.is_checked() {
                CheckedState::Checked
            } else {
                CheckedState::Unchecked
            },
            disabled: flags.disabled,
            invalid: flags.invalid,
            required: flags.required,
            readonly: flags.readonly,
        }
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        flags: CheckboxFlags,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(&self.props(flags), attrs, children)
    }

    /// [`control`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(
        &self,
        flags: CheckboxFlags,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        control(&self.props(flags), attrs, children)
    }

    /// [`indicator`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn indicator<'a>(
        &self,
        flags: CheckboxFlags,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        indicator(&self.props(flags), attrs, children)
    }

    /// [`label`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn label<'a>(
        &self,
        flags: CheckboxFlags,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        label(&self.props(flags), attrs, children)
    }

    /// [`hidden_input`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn hidden_input<'a>(
        &self,
        name: &'a str,
        value: &'a str,
        flags: CheckboxFlags,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        hidden_input(&self.props(flags), name, value, attrs)
    }
}

impl Component for Checkbox {
    type Action = CheckboxAction;

    fn update(&mut self, action: CheckboxAction) {
        self.checkable.update(action);
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > control(indicator)、`name`/`value` を要する
    /// [`hidden_input`] は含めない。[`crate::switch::Switch::view`] と同じ
    /// 位置付け）。公開 UI としての利用は想定しない（実際の UI 構築は
    /// §パーツ関数群・利便メソッドを呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        let props = self.props(CheckboxFlags::default());
        root(
            &props,
            Vec::new(),
            vec![control(
                &props,
                Vec::new(),
                vec![indicator(&props, Vec::new(), Vec::new())],
            )],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<CheckboxAction> {
        Checkable::decode_action(name, payload)
    }
}

impl Hydrate for Checkbox {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.checkable.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            checkable: Checkable::from_hydration_attrs(attrs)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{el, render, text};

    fn unchecked() -> CheckboxProps {
        CheckboxProps::default()
    }

    fn checked() -> CheckboxProps {
        CheckboxProps {
            checked: CheckedState::Checked,
            ..CheckboxProps::default()
        }
    }

    fn indeterminate() -> CheckboxProps {
        CheckboxProps {
            checked: CheckedState::Indeterminate,
            ..CheckboxProps::default()
        }
    }

    #[test]
    fn checked_state_as_data_state_matches_ark_ui_vocabulary() {
        assert_eq!(CheckedState::Unchecked.as_data_state(), "unchecked");
        assert_eq!(CheckedState::Checked.as_data_state(), "checked");
        assert_eq!(CheckedState::Indeterminate.as_data_state(), "indeterminate");
    }

    #[test]
    fn checked_state_to_aria_maps_tri_state() {
        assert_eq!(CheckedState::Unchecked.to_aria(), AriaChecked::False);
        assert_eq!(CheckedState::Checked.to_aria(), AriaChecked::True);
        assert_eq!(CheckedState::Indeterminate.to_aria(), AriaChecked::Mixed);
    }

    #[test]
    fn default_props_are_unchecked_and_all_false() {
        assert_eq!(CheckboxProps::default().checked, CheckedState::Unchecked);
        assert!(!CheckboxProps::default().disabled);
        assert!(!CheckboxProps::default().invalid);
        assert!(!CheckboxProps::default().required);
        assert!(!CheckboxProps::default().readonly);
    }

    #[test]
    fn root_renders_label_with_state_attrs() {
        let node = root(&unchecked(), vec![("id", "cb1")], vec![text("Accept")]);
        assert_eq!(
            render(&node),
            r#"<label data-scope="checkbox" data-part="root" data-state="unchecked" id="cb1">Accept</label>"#
        );
    }

    #[test]
    fn root_reflects_checked_and_disabled_state_attrs() {
        let mut props = checked();
        props.disabled = true;
        props.invalid = true;
        props.required = true;
        props.readonly = true;
        let node = root(&props, vec![], vec![]);
        assert_eq!(
            render(&node),
            r#"<label data-scope="checkbox" data-part="root" data-state="checked" data-disabled="" data-invalid="" data-required="" data-readonly=""></label>"#
        );
    }

    #[test]
    fn root_drops_caller_supplied_state_attrs_case_insensitively() {
        // レビュー指摘: state_attrs() 系キーが drop_reserved の対象外で
        // 重複属性・状態偽装が起きていた回帰を固定する。
        let node = root(
            &checked(),
            vec![
                ("data-state", "unchecked"),
                ("Data-Disabled", ""),
                ("DATA-INVALID", ""),
                ("data-required", ""),
                ("data-readonly", ""),
            ],
            vec![],
        );
        assert_eq!(
            render(&node),
            r#"<label data-scope="checkbox" data-part="root" data-state="checked"></label>"#
        );
    }

    #[test]
    fn control_drops_caller_supplied_state_attrs() {
        let node = control(&checked(), vec![("data-state", "unchecked")], vec![]);
        assert_eq!(
            render(&node),
            r#"<div data-scope="checkbox" data-part="control" data-state="checked" aria-hidden="true"></div>"#
        );
    }

    #[test]
    fn indicator_drops_caller_supplied_state_attrs() {
        let node = indicator(&checked(), vec![("data-disabled", "")], vec![]);
        assert_eq!(
            render(&node),
            r#"<div data-scope="checkbox" data-part="indicator" data-state="checked"></div>"#
        );
    }

    #[test]
    fn label_drops_caller_supplied_state_attrs() {
        let node = label(&checked(), vec![("data-invalid", "")], vec![text("x")]);
        assert_eq!(
            render(&node),
            r#"<span data-scope="checkbox" data-part="label" data-state="checked">x</span>"#
        );
    }

    #[test]
    fn hidden_input_drops_caller_supplied_state_attrs() {
        let node = hidden_input(
            &checked(),
            "terms",
            "on",
            vec![("data-state", "unchecked"), ("data-required", "")],
        );
        assert_eq!(
            render(&node),
            r#"<input data-scope="checkbox" data-part="hidden-input" data-state="checked" type="checkbox" name="terms" value="on" checked="">"#
        );
    }

    #[test]
    fn control_is_aria_hidden_div() {
        let node = control(&unchecked(), vec![], vec![]);
        assert_eq!(
            render(&node),
            r#"<div data-scope="checkbox" data-part="control" data-state="unchecked" aria-hidden="true"></div>"#
        );
    }

    #[test]
    fn control_drops_caller_supplied_aria_hidden_case_insensitively() {
        let node = control(&unchecked(), vec![("Aria-Hidden", "false")], vec![]);
        assert_eq!(
            render(&node),
            r#"<div data-scope="checkbox" data-part="control" data-state="unchecked" aria-hidden="true"></div>"#
        );
    }

    #[test]
    fn indicator_is_hidden_when_unchecked() {
        let node = indicator(&unchecked(), vec![], vec![]);
        assert_eq!(
            render(&node),
            r#"<div data-scope="checkbox" data-part="indicator" data-state="unchecked" hidden=""></div>"#
        );
    }

    #[test]
    fn indicator_is_visible_when_checked_or_indeterminate() {
        let checked_node = indicator(&checked(), vec![], vec![]);
        assert_eq!(
            render(&checked_node),
            r#"<div data-scope="checkbox" data-part="indicator" data-state="checked"></div>"#
        );

        let indeterminate_node = indicator(&indeterminate(), vec![], vec![]);
        assert_eq!(
            render(&indeterminate_node),
            r#"<div data-scope="checkbox" data-part="indicator" data-state="indeterminate"></div>"#
        );
    }

    #[test]
    fn indicator_drops_caller_supplied_hidden_case_insensitively() {
        let node = indicator(&checked(), vec![("HIDDEN", "")], vec![]);
        assert_eq!(
            render(&node),
            r#"<div data-scope="checkbox" data-part="indicator" data-state="checked"></div>"#
        );
    }

    #[test]
    fn label_wraps_text_span() {
        let node = label(&unchecked(), vec![], vec![text("Accept terms")]);
        assert_eq!(
            render(&node),
            r#"<span data-scope="checkbox" data-part="label" data-state="unchecked">Accept terms</span>"#
        );
    }

    #[test]
    fn hidden_input_unchecked_has_no_checked_attr() {
        let node = hidden_input(&unchecked(), "terms", "on", vec![]);
        assert_eq!(
            render(&node),
            r#"<input data-scope="checkbox" data-part="hidden-input" data-state="unchecked" type="checkbox" name="terms" value="on">"#
        );
    }

    #[test]
    fn hidden_input_checked_has_checked_attr() {
        let node = hidden_input(&checked(), "terms", "on", vec![]);
        assert_eq!(
            render(&node),
            r#"<input data-scope="checkbox" data-part="hidden-input" data-state="checked" type="checkbox" name="terms" value="on" checked="">"#
        );
    }

    #[test]
    fn hidden_input_indeterminate_has_aria_checked_mixed() {
        let node = hidden_input(&indeterminate(), "terms", "on", vec![]);
        assert_eq!(
            render(&node),
            r#"<input data-scope="checkbox" data-part="hidden-input" data-state="indeterminate" type="checkbox" name="terms" value="on" aria-checked="mixed">"#
        );
    }

    #[test]
    fn hidden_input_reflects_invalid_disabled_required() {
        let mut props = unchecked();
        props.invalid = true;
        props.disabled = true;
        props.required = true;
        let node = hidden_input(&props, "terms", "on", vec![]);
        assert_eq!(
            render(&node),
            r#"<input data-scope="checkbox" data-part="hidden-input" data-state="unchecked" data-disabled="" data-invalid="" data-required="" type="checkbox" name="terms" value="on" aria-invalid="true" disabled="" required="">"#
        );
    }

    #[test]
    fn hidden_input_drops_caller_supplied_reserved_attrs_case_insensitively() {
        let node = hidden_input(
            &checked(),
            "terms",
            "on",
            vec![
                ("Type", "text"),
                ("CHECKED", "false"),
                ("aria-checked", "false"),
                ("Name", "attacker"),
                ("VALUE", "attacker"),
                ("Disabled", "false"),
                ("Required", "false"),
                ("id", "terms-input"),
            ],
        );
        // フレームワーク値（checkbox/on/checked）が勝ち、呼び出し側の偽装値は落ちる。
        // id のような非予約属性は通過する。
        assert_eq!(
            render(&node),
            r#"<input data-scope="checkbox" data-part="hidden-input" data-state="checked" type="checkbox" name="terms" value="on" checked="" id="terms-input">"#
        );
    }

    #[test]
    fn parts_match_direct_el_call_shape() {
        // ANATOMY.part への薄い委譲であることを固定する（el() 直接呼び出しと同じ出力形）。
        let via_control = control(&unchecked(), vec![("id", "c1")], vec![]);
        let via_el = el(
            "div",
            vec![
                ("data-scope", "checkbox"),
                ("data-part", "control"),
                ("data-state", "unchecked"),
                ("aria-hidden", "true"),
                ("id", "c1"),
            ],
            vec![],
        );
        assert_eq!(render(&via_control), render(&via_el));
    }

    // --- Checkbox: dispatch 統合（イシュー #595） ---

    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    #[test]
    fn checkbox_default_is_unchecked() {
        assert!(!Checkbox::default().is_checked());
    }

    #[test]
    fn checkbox_dispatch_check_uncheck_toggle() {
        let mut cb = Checkbox::default();

        assert!(dispatch(&mut cb, "check", ""));
        assert!(cb.is_checked());

        assert!(dispatch(&mut cb, "uncheck", ""));
        assert!(!cb.is_checked());

        assert!(dispatch(&mut cb, "toggle", ""));
        assert!(cb.is_checked());
        assert!(dispatch(&mut cb, "toggle", ""));
        assert!(!cb.is_checked());
    }

    #[test]
    fn checkbox_dispatch_ignores_unknown_action() {
        let mut cb = Checkbox::new(true);
        assert!(!dispatch(&mut cb, "no_such_action", "x"));
        assert!(cb.is_checked());
    }

    #[test]
    fn checkbox_convenience_methods_reflect_state_and_flags() {
        let mut cb = Checkbox::default();
        assert!(dispatch(&mut cb, "check", ""));

        let all_flags = CheckboxFlags {
            disabled: true,
            invalid: true,
            required: true,
            readonly: true,
        };
        let root_html = render(&cb.root(all_flags, vec![], vec![]));
        assert!(root_html.contains(r#"data-state="checked""#));
        assert!(root_html.contains(r#"data-disabled="""#));
        assert!(root_html.contains(r#"data-invalid="""#));
        assert!(root_html.contains(r#"data-required="""#));
        assert!(root_html.contains(r#"data-readonly="""#));

        let hidden_input_html =
            render(&cb.hidden_input("terms", "on", CheckboxFlags::default(), vec![]));
        assert!(hidden_input_html.contains(r#"checked="""#));
    }

    #[test]
    fn checkbox_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Checkbox::default().view());
        assert!(rendered.contains(r#"data-state="unchecked""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    #[test]
    fn checkbox_hydration_round_trip() {
        let cb = Checkbox::new(true);
        let rendered = render(&render_for_hydration(&cb));
        assert!(rendered.contains(r#"data-hydrate-checked="checked""#));

        let restored = Checkbox::from_hydration_attrs(&cb.hydration_attrs()).unwrap();
        assert_eq!(restored, cb);
    }

    #[test]
    fn checkbox_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Checkbox::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            fandhe_frontend_interactive::HydrateError::MissingAttr(
                "data-hydrate-checked".to_string()
            )
        );
    }

    #[test]
    fn checkbox_from_hydration_attrs_rejects_indeterminate_and_unknown_values() {
        // 共通機械 Checkable は 2 値のみを扱うため、"indeterminate" を含む
        // 未知値は改ざん入力として一律拒否する（§設計判断 2.2 参照）。
        for bogus in ["indeterminate", "CHECKED", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-checked".to_string(), bogus.to_string())];
            let err = Checkbox::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(
                err,
                fandhe_frontend_interactive::HydrateError::InvalidValue { .. }
            ));
        }
    }
}

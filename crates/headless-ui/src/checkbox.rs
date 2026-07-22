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
//! 本モジュールは [`CheckedState`] を受け取って HTML を組み立てるだけであり、
//! 状態そのものは保持しない（SSR/SSG 初期描画で完結する）。クリックによる
//! トグル等の動的状態遷移（`Component`/`Hydrate` + `dispatch` 統合）は
//! **既存 open イシュー #524**（開閉状態機械の `fandhe-frontend-interactive`
//! 連携共通化）のスコープであり、本モジュールには含めない。
//! `crates/headless-ui/Cargo.toml` は `fandhe-frontend-interactive` に依存しない
//! （#524 未完のため）。
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
use fandhe_frontend_core::Node;

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

impl CheckedState {
    /// `data-state` の属性値文字列を返す。
    #[must_use]
    pub const fn as_data_state(self) -> &'static str {
        match self {
            Self::Unchecked => "unchecked",
            Self::Checked => "checked",
            Self::Indeterminate => "indeterminate",
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
            r#"<input data-scope="checkbox" data-part="hidden-input" data-state="checked" type="checkbox" name="terms" value="on" checked=""></input>"#
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
            r#"<input data-scope="checkbox" data-part="hidden-input" data-state="unchecked" type="checkbox" name="terms" value="on"></input>"#
        );
    }

    #[test]
    fn hidden_input_checked_has_checked_attr() {
        let node = hidden_input(&checked(), "terms", "on", vec![]);
        assert_eq!(
            render(&node),
            r#"<input data-scope="checkbox" data-part="hidden-input" data-state="checked" type="checkbox" name="terms" value="on" checked=""></input>"#
        );
    }

    #[test]
    fn hidden_input_indeterminate_has_aria_checked_mixed() {
        let node = hidden_input(&indeterminate(), "terms", "on", vec![]);
        assert_eq!(
            render(&node),
            r#"<input data-scope="checkbox" data-part="hidden-input" data-state="indeterminate" type="checkbox" name="terms" value="on" aria-checked="mixed"></input>"#
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
            r#"<input data-scope="checkbox" data-part="hidden-input" data-state="unchecked" data-disabled="" data-invalid="" data-required="" type="checkbox" name="terms" value="on" aria-invalid="true" disabled="" required=""></input>"#
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
            r#"<input data-scope="checkbox" data-part="hidden-input" data-state="checked" type="checkbox" name="terms" value="on" checked="" id="terms-input"></input>"#
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
}

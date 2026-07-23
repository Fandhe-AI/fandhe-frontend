//! PasswordInput（表示切替トリガー付きパスワード入力）headless コンポーネント
//! （イシュー #740、親 #736）。
//!
//! ark-ui の PasswordInput
//!（`.claude/skills/ark-ui/references/components/form/password-input.md`）を
//! 参考に、Root / Label / Control / Input / VisibilityTrigger / Indicator の
//! 6 anatomy パーツと、表示切替（visibility）状態機械 [`PasswordInput`] を
//! 提供する。
//!
//! # セキュリティ不変条件（本コンポーネント固有の中核要件）
//!
//! 本モジュールが扱うのは `type=password`/`type=text` の**表示切替状態機械と
//! DOM 属性の切替のみ**であり、パスワード値そのものは一切扱わない。
//!
//! - [`input`] は `value` 引数を持たず、出力 HTML に `value=` 属性が現れる
//!   経路を持たない（呼び出し側がフォーム送信で値を扱う場合はブラウザの
//!   ネイティブ input value に委ねる。本コンポーネントは常にそれを読み出し・
//!   保持・出力しない）。
//! - [`PasswordInput`] 状態機械は `visible`（bool、表示中かどうか）のみを
//!   フィールドに持ち、パスワード値を一切保持しない。`Debug`/`Hydrate` の
//!   出力・エラーメッセージ・ログのいずれにもパスワード値は現れない
//!   （現れる余地となるフィールド自体が存在しない）。
//! - hydration 属性 `data-hydrate-visible` はクライアント側で改ざんされうる
//!   入力として扱い、`from_hydration_attrs` は panic せず `HydrateError` で
//!   未知値を拒否する（fail-closed、`docs/api/hydration-state-format.md` 準拠）。
//!
//! # `data-state` 語彙について（`Checkable` を使わない理由）
//!
//! [`crate::state::Checkable`] は `"checked"`/`"unchecked"` 語彙に固定されて
//! おり（[`crate::switch::Switch`] の rustdoc 参照）、本コンポーネントが必要
//! とする `"visible"`/`"hidden"` 語彙とは異なる。3 値目以上の追加ではなく
//! 別語彙のため、[`crate::switch::Switch`] の #595 昇格前と同型でモジュール
//! 内に個別実装する（[`crate::avatar::Avatar`] の 3 値ステータス機械と同じ
//! 判断: 既存 2 値共通機械のいずれにも値語彙が一致しない場合は個別実装する
//! 方針）。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`label`]/[`control`]/[`input`]/
//! [`visibility_trigger`]/[`indicator`]、純粋関数で完結）を直接呼んで組み
//! 立てる。CSR/hydration は [`PasswordInput`] を経由し、dispatch
//! （`"show"`/`"hide"`/`"toggle"`）で状態遷移する。クライアント側の
//! click → dispatch 配線（`fandhe-frontend-wasm-full`）は既存の汎用 dispatch
//! 機構の利用側責務であり本イシューのスコープ外（[`crate::switch::Switch`]
//! と同じ切り分け）。`fandhe-frontend-pre-styled-ui`（#546〜）が本モジュール
//! を呼んでスタイル済み PasswordInput を組み立てる想定である。
//!
//! # トリガーのアクセシビリティ表現
//!
//! [`visibility_trigger`] は `button type="button"`（フォーム submit 誤発火
//! 防止）+ `aria-pressed`（トグルボタンパターン、表示中で `"true"`）+
//! `aria-controls`（対象 [`input`] の id）で意味論を担う。`aria-label`
//! （「パスワードを表示」等の文言）は呼び出し側が `attrs` へ付与する
//! （本コンポーネントは固定文言を持たない。国際化はアプリ側の責務）。
//! [`indicator`] は装飾専用のため `aria-hidden="true"` を固定付与し、
//! 支援技術の重複読み上げを防ぐ（trigger の `aria-pressed` が意味論を担う）。
//!
//! # 1 PasswordInput = 1 Input の id 導出
//!
//! [`PasswordInputProps::id`] から [`input`] の id を `"{id}-input"` として
//! 決定的に導出し、[`label`] の `for`・[`visibility_trigger`] の
//! `aria-controls` へも同じ値を一貫伝播する（`crate::field` の派生 id 方針と
//! 同型。個別箇所での再導出は行わない）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - クライアント側の click → dispatch 配線（wasm-full）。
//! - crates.io への新バージョン公開・`examples/headless-pre-styled-ui` への
//!   PasswordInput 追加（#608/#609 と同じ後続分離）。
//! - ark-ui の `ignorePasswordManagers` 相当（`data-1p-ignore` 等の静的属性）
//!   は本イシューでは見送り、需要が判明したら別途 Issue 化する。
//!
//! # セキュリティ不変条件（共通部分）
//!
//! - 属性名（`data-*`/`aria-*`/`type`/`autocomplete`/`disabled`/`required`）
//!   はすべて `&'static str` リテラルで固定しており、動的値が属性名スロット
//!   へ混入する経路はない（[`crate::anatomy`]/[`crate::aria`]/
//!   [`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（`id`/呼び出し側 `attrs`/`children`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - [`input`] の `type` 属性値は `visible` の bool から `"password"`/
//!   `"text"` の 2 値のみを決定的に導出し、呼び出し側文字列を通さない。
//! - [`crate::anatomy::Anatomy::part`] の `data-scope`/`data-part` 偽装除去
//!   （fail-closed）を継承し回帰テストで固定する。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_controls, aria_hidden, aria_invalid, aria_pressed};
use crate::data_attrs::{data_disabled, data_invalid, data_required, data_state};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// PasswordInput の anatomy（`data-scope="password-input"`）。
const ANATOMY: Anatomy = anatomy("password-input");

/// `data-state`/`data-hydrate-visible` 属性値 "visible"。
const DATA_STATE_VISIBLE: &str = "visible";
/// `data-state`/`data-hydrate-visible` 属性値 "hidden"。
const DATA_STATE_HIDDEN: &str = "hidden";

/// `visible` から `data-state`/`data-hydrate-visible` の属性値文字列へ
/// 変換する（[`crate::state::checked_data_state`] の visibility 版）。
#[must_use]
pub const fn visible_data_state(visible: bool) -> &'static str {
    if visible {
        DATA_STATE_VISIBLE
    } else {
        DATA_STATE_HIDDEN
    }
}

/// `data-state`/`data-hydrate-visible` 属性値から `visible` を復元する。
///
/// 未知の値（改ざん・タイポを含む）は `None` を返す（安全側、呼び出し元が
/// [`HydrateError::InvalidValue`] へ変換する）。
#[must_use]
fn visible_from_data_state(s: &str) -> Option<bool> {
    match s {
        DATA_STATE_VISIBLE => Some(true),
        DATA_STATE_HIDDEN => Some(false),
        _ => None,
    }
}

/// パスワードマネージャ連携時の自動補完ヒント（ark-ui `autoComplete` 相当）。
///
/// 自由文字列にせず固定語彙の enum とすることで、`autocomplete` 属性値汚染
/// （任意文字列の混入）を型レベルで防ぐ（`.claude/rules/security.md` A05）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordAutocomplete {
    /// ログインフォーム等、既存パスワードの入力（`autocomplete="current-password"`）。
    CurrentPassword,
    /// 登録・パスワード変更フォーム等、新規パスワードの入力
    /// （`autocomplete="new-password"`）。
    NewPassword,
}

impl PasswordAutocomplete {
    /// `autocomplete` 属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CurrentPassword => "current-password",
            Self::NewPassword => "new-password",
        }
    }
}

/// `password_input` モジュールの各パーツ関数へ共通で渡す props
/// （[`crate::field::FieldProps`] と同型の様式）。
#[derive(Debug, Clone, Copy)]
pub struct PasswordInputProps<'a> {
    /// ベース id。[`input`] の id（`"{id}-input"`）の決定的導出に使う。
    /// 「1 PasswordInput = 1 Input」が呼び出し側の契約である。
    pub id: &'a str,
    /// フィールド全体の無効化。`true` のとき [`input`]/[`visibility_trigger`]
    /// にネイティブ `disabled` 存在属性・`data-disabled` を付与する。
    pub disabled: bool,
    /// 入力値が不正であることを示す。`true` のとき [`root`]/[`control`]/
    /// [`input`] に `data-invalid` を、[`input`] に `aria-invalid="true"` を
    /// 付与する。
    pub invalid: bool,
    /// 必須入力。`true` のとき [`input`] にネイティブ `required` 存在属性・
    /// `data-required` を付与する。
    pub required: bool,
    /// パスワードマネージャ連携の自動補完ヒント。
    pub autocomplete: PasswordAutocomplete,
}

/// [`PasswordInputProps::id`] から [`input`] の id（`"{id}-input"`）を導出
/// する（[`label`] の `for`・[`visibility_trigger`] の `aria-controls` も
/// この値を参照する）。
#[must_use]
fn control_input_id(id: &str) -> String {
    format!("{id}-input")
}

/// Root パーツ（`div`）。visible/invalid/disabled 状態を `data-*` へ反映する。
#[must_use]
pub fn root(
    visible: bool,
    props: &PasswordInputProps<'_>,
    attrs: Vec<(&str, &str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![data_state(visible_data_state(visible))];
    merged.extend(data_invalid(props.invalid));
    merged.extend(data_disabled(props.disabled));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。[`input`] と同じ派生 id へ `for` で関連付ける。
#[must_use]
pub fn label(
    props: &PasswordInputProps<'_>,
    attrs: Vec<(&str, &str)>,
    children: Vec<Node>,
) -> Node {
    let input_id = control_input_id(props.id);
    let mut merged: Vec<(&str, &str)> = vec![("for", input_id.as_str())];
    merged.extend(attrs);
    ANATOMY.part("label", "label", merged, children)
}

/// Control パーツ（`div`）。[`input`]/[`visibility_trigger`] を内包する枠。
#[must_use]
pub fn control(
    visible: bool,
    props: &PasswordInputProps<'_>,
    attrs: Vec<(&str, &str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![data_state(visible_data_state(visible))];
    merged.extend(data_invalid(props.invalid));
    merged.extend(data_disabled(props.disabled));
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// Input パーツ（`input`）。`type` を `visible` に応じて `"text"`/`"password"`
/// に切替える。
///
/// **`value` 引数を持たない**（本モジュールのセキュリティ不変条件、モジュール
/// doc 参照）。パスワード値の表示・保持・出力は一切行わない。
#[must_use]
pub fn input(visible: bool, props: &PasswordInputProps<'_>, attrs: Vec<(&str, &str)>) -> Node {
    let input_id = control_input_id(props.id);
    let ty = if visible { "text" } else { "password" };
    let mut merged: Vec<(&str, &str)> = vec![
        ("type", ty),
        ("id", input_id.as_str()),
        ("autocomplete", props.autocomplete.as_str()),
    ];
    if props.disabled {
        merged.push(("disabled", ""));
    }
    if props.required {
        merged.push(("required", ""));
    }
    if props.invalid {
        merged.push(aria_invalid(true));
    }
    merged.extend(data_invalid(props.invalid));
    merged.extend(data_disabled(props.disabled));
    merged.extend(data_required(props.required));
    merged.extend(attrs);
    ANATOMY.part("input", "input", merged, Vec::new())
}

/// VisibilityTrigger パーツ（`button type="button"`）。
///
/// `aria-pressed`（トグルボタンパターン、`visible` で `"true"`）+
/// `aria-controls`（[`input`] の id）で意味論を担う。`aria-label` は呼び出し
/// 側が `attrs` へ付与する（モジュール doc 参照）。
#[must_use]
pub fn visibility_trigger(
    visible: bool,
    props: &PasswordInputProps<'_>,
    attrs: Vec<(&str, &str)>,
    children: Vec<Node>,
) -> Node {
    let input_id = control_input_id(props.id);
    let mut merged: Vec<(&str, &str)> = vec![
        ("type", "button"),
        aria_pressed(visible),
        aria_controls(input_id.as_str()),
        data_state(visible_data_state(visible)),
    ];
    merged.extend(data_disabled(props.disabled));
    if props.disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("visibility-trigger", "button", merged, children)
}

/// Indicator パーツ（`span`）。装飾専用のため `aria-hidden="true"` を固定
/// 付与する（[`visibility_trigger`] の `aria-pressed` が意味論を担う）。
#[must_use]
pub fn indicator(visible: bool, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&str, &str)> =
        vec![data_state(visible_data_state(visible)), aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("indicator", "span", merged, children)
}

/// PasswordInput のアクション（WASM 境界の文字列 dispatch と
/// [`PasswordInput::decode_action`] で接続する）。payload は使用しない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordInputAction {
    /// 表示する（`type="text"` にする）。
    Show,
    /// 隠す（`type="password"` にする）。
    Hide,
    /// 表示状態を反転する。
    Toggle,
}

/// PasswordInput の表示切替（visibility）状態機械。
///
/// `data-state` と実際の表示状態の整合を型レベルで保証する入口として、各
/// パーツ関数（[`root`]/[`control`]/[`input`]/[`visibility_trigger`]/
/// [`indicator`]）へ `self.is_visible()` を注入する利便メソッドを提供する。
/// SSR での自由関数直接利用（本型を経由しない構成）も引き続き可能。
///
/// `Default` は Hidden（パスワードは隠すのが安全側の既定、
/// `.claude/rules/security.md` A05 準拠）。
///
/// パスワード値は一切保持しない（`visible: bool` のみのフィールド。モジュール
/// doc のセキュリティ不変条件参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PasswordInput {
    visible: bool,
}

impl PasswordInput {
    /// `data-hydrate-visible` 属性名のフィールド部分
    /// （`docs/api/hydration-state-format.md` の `<field>` 命名規約に従う）。
    pub const FIELD_VISIBLE: &'static str = "visible";

    /// 指定した初期状態で PasswordInput を生成する。
    #[must_use]
    pub fn new(visible: bool) -> Self {
        Self { visible }
    }

    /// 現在表示中かどうか。
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// 現在の `data-state` 属性値（`"visible"`/`"hidden"`）。
    #[must_use]
    pub fn data_state(&self) -> &'static str {
        visible_data_state(self.visible)
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root(
        &self,
        props: &PasswordInputProps<'_>,
        attrs: Vec<(&str, &str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.visible, props, attrs, children)
    }

    /// [`control`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn control(
        &self,
        props: &PasswordInputProps<'_>,
        attrs: Vec<(&str, &str)>,
        children: Vec<Node>,
    ) -> Node {
        control(self.visible, props, attrs, children)
    }

    /// [`input`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn input(&self, props: &PasswordInputProps<'_>, attrs: Vec<(&str, &str)>) -> Node {
        input(self.visible, props, attrs)
    }

    /// [`visibility_trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn visibility_trigger(
        &self,
        props: &PasswordInputProps<'_>,
        attrs: Vec<(&str, &str)>,
        children: Vec<Node>,
    ) -> Node {
        visibility_trigger(self.visible, props, attrs, children)
    }

    /// [`indicator`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn indicator(&self, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
        indicator(self.visible, attrs, children)
    }
}

impl Component for PasswordInput {
    type Action = PasswordInputAction;

    fn update(&mut self, action: PasswordInputAction) {
        self.visible = match action {
            PasswordInputAction::Show => true,
            PasswordInputAction::Hide => false,
            PasswordInputAction::Toggle => !self.visible,
        };
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > control(input + visibility_trigger)、id は固定の
    /// プレースホルダを使う）。公開 UI としての利用は想定しない（実際の UI
    /// 構築は §パーツ関数群を呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        let props = PasswordInputProps {
            id: "password-input",
            disabled: false,
            invalid: false,
            required: false,
            autocomplete: PasswordAutocomplete::CurrentPassword,
        };
        self.root(
            &props,
            Vec::new(),
            vec![control(
                self.visible,
                &props,
                Vec::new(),
                vec![
                    input(self.visible, &props, Vec::new()),
                    visibility_trigger(self.visible, &props, Vec::new(), Vec::new()),
                ],
            )],
        )
    }

    fn decode_action(name: &str, _payload: &str) -> Option<PasswordInputAction> {
        match name {
            "show" => Some(PasswordInputAction::Show),
            "hide" => Some(PasswordInputAction::Hide),
            "toggle" => Some(PasswordInputAction::Toggle),
            _ => None,
        }
    }
}

impl Hydrate for PasswordInput {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_VISIBLE),
            self.data_state().to_string(),
        )]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let attr_name = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_VISIBLE);
        let raw = attrs
            .iter()
            .find(|(k, _)| *k == attr_name)
            .map(|(_, v)| v.as_str())
            .ok_or_else(|| HydrateError::MissingAttr(attr_name.clone()))?;
        let visible = visible_from_data_state(raw).ok_or_else(|| HydrateError::InvalidValue {
            attr: attr_name.clone(),
            reason: "expected \"visible\" or \"hidden\"".to_string(),
        })?;
        Ok(Self { visible })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    fn default_props(id: &str) -> PasswordInputProps<'_> {
        PasswordInputProps {
            id,
            disabled: false,
            invalid: false,
            required: false,
            autocomplete: PasswordAutocomplete::CurrentPassword,
        }
    }

    // --- 各パーツの data-scope/data-part/data-state 出力 ---

    #[test]
    fn root_outputs_scope_part_and_state() {
        let props = default_props("pw");
        let html = render(&root(false, &props, vec![], vec![]));
        assert!(html.contains(r#"data-scope="password-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="hidden""#));
        assert!(!html.contains("data-invalid"));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn root_visible_true_outputs_visible_state() {
        let props = default_props("pw");
        let html = render(&root(true, &props, vec![], vec![]));
        assert!(html.contains(r#"data-state="visible""#));
    }

    #[test]
    fn root_invalid_and_disabled_add_data_attrs() {
        let mut props = default_props("pw");
        props.invalid = true;
        props.disabled = true;
        let html = render(&root(false, &props, vec![], vec![]));
        assert!(html.contains(r#"data-invalid=""#));
        assert!(html.contains(r#"data-disabled=""#));
    }

    #[test]
    fn label_outputs_for_matching_input_id() {
        let props = default_props("pw");
        let html = render(&label(&props, vec![], vec![text("Password")]));
        assert!(html.contains(r#"data-scope="password-input""#));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains(r#"for="pw-input""#));
        assert!(html.contains("Password"));
    }

    #[test]
    fn control_outputs_scope_part_and_state() {
        let props = default_props("pw");
        let html = render(&control(true, &props, vec![], vec![]));
        assert!(html.contains(r#"data-scope="password-input""#));
        assert!(html.contains(r#"data-part="control""#));
        assert!(html.contains(r#"data-state="visible""#));
    }

    #[test]
    fn input_type_switches_between_password_and_text() {
        let props = default_props("pw");
        let hidden_html = render(&input(false, &props, vec![]));
        assert!(hidden_html.contains(r#"type="password""#));
        assert!(hidden_html.contains(r#"id="pw-input""#));
        assert!(hidden_html.contains(r#"autocomplete="current-password""#));

        let visible_html = render(&input(true, &props, vec![]));
        assert!(visible_html.contains(r#"type="text""#));
    }

    #[test]
    fn input_new_password_autocomplete_variant() {
        let mut props = default_props("pw");
        props.autocomplete = PasswordAutocomplete::NewPassword;
        let html = render(&input(false, &props, vec![]));
        assert!(html.contains(r#"autocomplete="new-password""#));
    }

    #[test]
    fn input_disabled_required_are_present_attrs() {
        let mut props = default_props("pw");
        props.disabled = true;
        props.required = true;
        props.invalid = true;
        let html = render(&input(false, &props, vec![]));
        assert!(html.contains(r#"disabled=""#));
        assert!(html.contains(r#"required=""#));
        assert!(html.contains(r#"aria-invalid="true""#));
        assert!(html.contains(r#"data-disabled=""#));
        assert!(html.contains(r#"data-required=""#));
        assert!(html.contains(r#"data-invalid=""#));
    }

    #[test]
    fn input_omits_boolean_attrs_when_false() {
        let props = default_props("pw");
        let html = render(&input(false, &props, vec![]));
        assert!(!html.contains(r#"disabled=""#));
        assert!(!html.contains(r#"required=""#));
        assert!(!html.contains("aria-invalid"));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-required"));
        assert!(!html.contains("data-invalid"));
    }

    #[test]
    fn input_never_outputs_value_attribute() {
        // セキュリティ不変条件: value 引数自体を持たないため、あらゆる状態
        // 組み合わせで出力 HTML に value= が現れないことを固定する。
        let mut props = default_props("pw");
        for disabled in [false, true] {
            for required in [false, true] {
                for invalid in [false, true] {
                    props.disabled = disabled;
                    props.required = required;
                    props.invalid = invalid;
                    for visible in [false, true] {
                        let html = render(&input(visible, &props, vec![]));
                        assert!(!html.contains("value="), "unexpected value= in {html}");
                    }
                }
            }
        }
    }

    #[test]
    fn visibility_trigger_outputs_aria_pressed_and_controls() {
        let props = default_props("pw");
        let hidden_html = render(&visibility_trigger(false, &props, vec![], vec![]));
        assert!(hidden_html.contains(r#"type="button""#));
        assert!(hidden_html.contains(r#"aria-pressed="false""#));
        assert!(hidden_html.contains(r#"aria-controls="pw-input""#));
        assert!(hidden_html.contains(r#"data-state="hidden""#));

        let visible_html = render(&visibility_trigger(true, &props, vec![], vec![]));
        assert!(visible_html.contains(r#"aria-pressed="true""#));
        assert!(visible_html.contains(r#"data-state="visible""#));
    }

    #[test]
    fn visibility_trigger_disabled_adds_native_and_data_attr() {
        let mut props = default_props("pw");
        props.disabled = true;
        let html = render(&visibility_trigger(false, &props, vec![], vec![]));
        assert!(html.contains(r#"disabled=""#));
        assert!(html.contains(r#"data-disabled=""#));
    }

    #[test]
    fn indicator_outputs_scope_part_state_and_aria_hidden() {
        let html = render(&indicator(true, vec![], vec![]));
        assert!(html.contains(r#"data-scope="password-input""#));
        assert!(html.contains(r#"data-part="indicator""#));
        assert!(html.contains(r#"data-state="visible""#));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側 attrs の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let props = default_props("pw");
        let html = render(&root(
            false,
            &props,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="password-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- PasswordInput: dispatch 統合 ---

    #[test]
    fn password_input_default_is_hidden() {
        assert!(!PasswordInput::default().is_visible());
    }

    #[test]
    fn password_input_dispatch_toggle_changes_data_state() {
        let props = default_props("pw");
        let mut p = PasswordInput::default();
        assert!(render(&p.root(&props, vec![], vec![])).contains(r#"data-state="hidden""#));

        assert!(dispatch(&mut p, "toggle", ""));
        assert!(render(&p.root(&props, vec![], vec![])).contains(r#"data-state="visible""#));
        assert!(render(&p.control(&props, vec![], vec![])).contains(r#"data-state="visible""#));
        assert!(render(&p.input(&props, vec![])).contains(r#"type="text""#));
        assert!(render(&p.visibility_trigger(&props, vec![], vec![]))
            .contains(r#"aria-pressed="true""#));
        assert!(render(&p.indicator(vec![], vec![])).contains(r#"data-state="visible""#));
    }

    #[test]
    fn password_input_dispatch_show_and_hide() {
        let mut p = PasswordInput::default();
        assert!(dispatch(&mut p, "show", ""));
        assert!(p.is_visible());
        assert!(dispatch(&mut p, "hide", ""));
        assert!(!p.is_visible());
    }

    #[test]
    fn password_input_dispatch_ignores_unknown_action() {
        let mut p = PasswordInput::new(true);
        assert!(!dispatch(&mut p, "no_such_action", "x"));
        assert!(p.is_visible());
    }

    // --- PasswordInput: SSR 状態なし初期描画 ---

    #[test]
    fn password_input_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&PasswordInput::default().view());
        assert!(rendered.contains(r#"data-state="hidden""#));
        assert!(!rendered.contains("data-hydrate-"));
        assert!(!rendered.contains("value="));
    }

    // --- PasswordInput: hydration 経路 ---

    #[test]
    fn password_input_hydration_round_trip() {
        let p = PasswordInput::new(true);
        let rendered = render(&render_for_hydration(&p));
        assert!(rendered.contains(r#"data-hydrate-visible="visible""#));

        let restored = PasswordInput::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored, p);
    }

    #[test]
    fn password_input_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = PasswordInput::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-visible".to_string())
        );
    }

    #[test]
    fn password_input_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["VISIBLE", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-visible".to_string(), bogus.to_string())];
            let err = PasswordInput::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: id/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn id_payload_is_escaped_on_render() {
        let props = default_props(ATTR_BREAK_PAYLOAD);
        let html = render(&label(&props, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let props = default_props("pw");
        let html = render(&root(
            false,
            &props,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&indicator(
            true,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn password_input_xss_payload_in_hydration_visible_is_rejected_not_rendered() {
        // data-hydrate-visible はサーバーが data_state() から生成する固定語彙
        // のみを出力するため攻撃者が任意値を注入する経路はないが、クライアント
        // 改ざん入力の復元経路（from_hydration_attrs）が未知値を拒否することを
        // PasswordInput 経由でも固定する。
        let attrs = vec![(
            "data-hydrate-visible".to_string(),
            "<script>alert(1)</script>".to_string(),
        )];
        let err = PasswordInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

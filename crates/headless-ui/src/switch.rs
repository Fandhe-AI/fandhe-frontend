//! Switch（オン/オフ切り替え）headless コンポーネント（イシュー #537、親 #534）。
//!
//! ark-ui の Switch
//!（`.claude/skills/ark-ui/references/components/form/switch.md`）を
//! 参考に、Root / Control / Thumb / Label / HiddenInput の 5 anatomy パーツと、
//! Phase 1（#524）の [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 抽象へ直接乗る開閉状態機械
//! [`Switch`] を提供する。
//!
//! # `data-state` 語彙について（[`crate::state::Disclosure`] を使わない理由）
//!
//! [`crate::state::Disclosure`] の `data-state` 語彙は `"open"`/`"closed"` に
//! 固定されている（[`crate::state::OpenState`]）。Switch は ark-ui 準拠で
//! `"checked"`/`"unchecked"` を使うため、[`Disclosure`](crate::state::Disclosure)
//! を埋め込むと表示用 `data-state` と hydration 属性の語彙が分裂する。
//! これを避けるため、本モジュールは [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装し、Phase 1 が確立した
//! dispatch 契約（未知アクション no-op）・fail-closed hydration という
//! **統合様式**にのみ準拠する（`"checked"/"unchecked"` 状態機械の
//! `state.rs` への共通化昇格は Checkbox 実装後の別イシューとする）。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`control`]/[`thumb`]/[`label`]/
//! [`hidden_input`]、純粋関数で完結）を直接呼んで組み立てる。CSR/hydration は
//! [`Switch`] を経由し、dispatch（`"check"`/`"uncheck"`/`"toggle"`）で状態
//! 遷移する。`fandhe-frontend-pre-styled-ui`（#546〜）が本モジュールを呼んで
//! スタイル済み Switch を組み立てる想定である。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`type`/`role`/`name`/`checked`/`disabled`/
//!   `required`）はすべて `&'static str` リテラルで固定しており、動的値が
//!   属性名スロットへ混入する経路はない（[`crate::anatomy`]/[`crate::aria`]/
//!   [`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（`name`/`value`/呼び出し側 `attrs`/`children` テキスト）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"checked"`/`"unchecked"`）は本モジュール内で
//!   一元管理し（[`state_str`]）、パーツ関数間で分裂させない。
//! - hidden input は `<input type="checkbox" role="switch">`（WAI-ARIA APG
//!   の「Switch Example Using HTML Checkbox Input」パターン）。native の
//!   `checked` 状態がブラウザによって `aria-checked` へマップされるため、
//!   本モジュールは `aria-checked` を明示付与しない（二重読み上げ防止。
//!   `radio_group` の native input 方針と同型）。
//! - hydration 属性（`data-hydrate-checked`）はクライアント側で改ざんされ
//!   うる入力として扱う。[`Switch`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は panic せず
//!   `HydrateError` を返す。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::aria_hidden;
use crate::data_attrs::{data_disabled, data_required, data_state};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Switch の anatomy（`data-scope="switch"`）。
const ANATOMY: Anatomy = anatomy("switch");

/// `data-state` 属性値 "checked"（ark-ui Switch 準拠の値語彙。本モジュールが
/// 一元管理し、パーツ関数間で分裂させない）。
const DATA_STATE_CHECKED: &str = "checked";
/// `data-state` 属性値 "unchecked"。[`DATA_STATE_CHECKED`] 参照。
const DATA_STATE_UNCHECKED: &str = "unchecked";

/// `checked` から `data-state`/`data-hydrate-checked` の属性値文字列へ変換する。
const fn state_str(checked: bool) -> &'static str {
    if checked {
        DATA_STATE_CHECKED
    } else {
        DATA_STATE_UNCHECKED
    }
}

/// `data-state`/`data-hydrate-checked` 属性値から `checked` を復元する。
///
/// 未知の値（改ざん・タイポ）は `None` を返す（安全側、呼び出し元が
/// [`HydrateError::InvalidValue`] へ変換する）。
fn checked_from_state_str(s: &str) -> Option<bool> {
    match s {
        DATA_STATE_CHECKED => Some(true),
        DATA_STATE_UNCHECKED => Some(false),
        _ => None,
    }
}

/// Root パーツ（`label`）。
///
/// 内包する [`hidden_input`] との暗黙のラベル関連付けを成立させるため
/// `<label>` 要素を使う（`for`/`id` の配線が不要になる。ark-ui と同じ方針）。
/// checked/disabled 状態を `data-*` へ反映する。
#[must_use]
pub fn root<'a>(
    checked: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state_str(checked))];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("root", "label", merged, children)
}

/// Control パーツ（`span`）。トラック/つまみの見た目を担う装飾用パーツ。
///
/// 意味論（オン/オフ）は [`hidden_input`] の native checkbox が担うため、
/// `aria-hidden="true"` を固定付与し、支援技術の重複読み上げを防ぐ
/// （`radio_group` の control と同じ最小主義）。
#[must_use]
pub fn control<'a>(
    checked: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![data_state(state_str(checked)), aria_hidden(true)];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("control", "span", merged, children)
}

/// Thumb パーツ（`span`）。checked 状態のみを `data-state` へ反映する
/// 最小主義な装飾用パーツ（[`crate::collapsible::indicator`] と同じ最小主義）。
#[must_use]
pub fn thumb<'a>(checked: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state_str(checked))];
    merged.extend(attrs);
    ANATOMY.part("thumb", "span", merged, children)
}

/// Label パーツ（`span`）。ラベルテキストを表示する装飾用パーツ
/// （意味論的なラベル関連付けは [`root`] の `<label>` 要素が担う）。
#[must_use]
pub fn label<'a>(checked: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state_str(checked))];
    merged.extend(attrs);
    ANATOMY.part("label", "span", merged, children)
}

/// HiddenInput パーツ（`input type="checkbox" role="switch"`）。
///
/// WAI-ARIA APG の「Switch Example Using HTML Checkbox Input」パターンに
/// 従い、native checkbox の checked 状態でオン/オフの意味論・フォーム送信
/// を担う（`aria-checked` は自動マップされるため明示付与しない）。
/// `checked`/`disabled`/`required` は存在属性として `true` のときのみ
/// 出力する（ark-ui 流の boolean 属性規約、[`crate::data_attrs`] と同型）。
#[must_use]
pub fn hidden_input<'a>(
    name: &'a str,
    value: &'a str,
    checked: bool,
    disabled: bool,
    required: bool,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "checkbox"),
        ("role", "switch"),
        ("name", name),
        ("value", value),
    ];
    if checked {
        merged.push(("checked", ""));
    }
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_required(required));
    if required {
        merged.push(("required", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("hidden-input", "input", merged, Vec::new())
}

/// Switch のアクション（WASM 境界の文字列 dispatch と
/// [`Switch::decode_action`] で接続する）。payload は使用しない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchAction {
    /// オンにする。
    Check,
    /// オフにする。
    Uncheck,
    /// オン/オフを反転する。
    Toggle,
}

/// Switch の開閉（オン/オフ）状態機械。
///
/// `data-state` と実際のチェック状態の整合を型レベルで保証する入口として、
/// 各パーツ関数（[`root`]/[`control`]/[`thumb`]/[`label`]/[`hidden_input`]）へ
/// `self.is_checked()` を注入する利便メソッドを提供する。SSR での自由関数
/// 直接利用（本型を経由しない構成）も引き続き可能。`Default` は未チェック
/// （SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Switch {
    checked: bool,
}

impl Switch {
    /// `data-hydrate-checked` 属性名のフィールド部分
    /// （`docs/api/hydration-state-format.md` の `<field>` 命名規約に従う）。
    pub const FIELD_CHECKED: &'static str = "checked";

    /// 指定した初期状態で Switch を生成する。
    #[must_use]
    pub fn new(checked: bool) -> Self {
        Self { checked }
    }

    /// 現在チェックされているかどうか。
    #[must_use]
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// 現在の `data-state` 属性値（`"checked"`/`"unchecked"`）。
    #[must_use]
    pub fn data_state(&self) -> &'static str {
        state_str(self.checked)
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.checked, disabled, attrs, children)
    }

    /// [`control`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        control(self.checked, disabled, attrs, children)
    }

    /// [`thumb`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn thumb<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        thumb(self.checked, attrs, children)
    }

    /// [`label`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn label<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        label(self.checked, attrs, children)
    }

    /// [`hidden_input`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn hidden_input<'a>(
        &self,
        name: &'a str,
        value: &'a str,
        disabled: bool,
        required: bool,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        hidden_input(name, value, self.checked, disabled, required, attrs)
    }
}

impl Component for Switch {
    type Action = SwitchAction;

    fn update(&mut self, action: SwitchAction) {
        self.checked = match action {
            SwitchAction::Check => true,
            SwitchAction::Uncheck => false,
            SwitchAction::Toggle => !self.checked,
        };
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > control(thumb)、`name`/`value` を要する
    /// [`hidden_input`] は含めない）。公開 UI としての利用は想定しない
    /// （実際の UI 構築は §パーツ関数群を呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        self.root(
            false,
            Vec::new(),
            vec![control(
                self.checked,
                false,
                Vec::new(),
                vec![thumb(self.checked, Vec::new(), Vec::new())],
            )],
        )
    }

    fn decode_action(name: &str, _payload: &str) -> Option<SwitchAction> {
        match name {
            "check" => Some(SwitchAction::Check),
            "uncheck" => Some(SwitchAction::Uncheck),
            "toggle" => Some(SwitchAction::Toggle),
            _ => None,
        }
    }
}

impl Hydrate for Switch {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_CHECKED),
            self.data_state().to_string(),
        )]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let attr_name = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_CHECKED);
        let raw = attrs
            .iter()
            .find(|(k, _)| *k == attr_name)
            .map(|(_, v)| v.as_str())
            .ok_or_else(|| HydrateError::MissingAttr(attr_name.clone()))?;
        let checked = checked_from_state_str(raw).ok_or_else(|| HydrateError::InvalidValue {
            attr: attr_name.clone(),
            reason: "expected \"checked\" or \"unchecked\"".to_string(),
        })?;
        Ok(Self { checked })
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
        let html = render(&root(false, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert!(html.contains("<label"));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn root_checked_true_outputs_checked_state() {
        let html = render(&root(true, false, vec![], vec![]));
        assert!(html.contains(r#"data-state="checked""#));
    }

    #[test]
    fn root_disabled_true_adds_data_disabled() {
        let html = render(&root(true, true, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn control_outputs_scope_part_state_and_aria_hidden() {
        let html = render(&control(true, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="control""#));
        assert!(html.contains(r#"data-state="checked""#));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(!html.contains("aria-checked"));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn control_disabled_true_adds_data_disabled() {
        let html = render(&control(false, true, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn thumb_outputs_scope_part_and_state_only() {
        let html = render(&thumb(true, vec![], vec![]));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="thumb""#));
        assert!(html.contains(r#"data-state="checked""#));
    }

    #[test]
    fn label_outputs_scope_part_and_state() {
        let html = render(&label(false, vec![], vec![text("Airplane mode")]));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert!(html.contains("Airplane mode"));
    }

    #[test]
    fn hidden_input_outputs_type_role_name_value() {
        let html = render(&hidden_input("wifi", "on", false, false, false, vec![]));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="hidden-input""#));
        assert!(html.contains("<input"));
        assert!(html.contains(r#"type="checkbox""#));
        assert!(html.contains(r#"role="switch""#));
        assert!(html.contains(r#"name="wifi""#));
        assert!(html.contains(r#"value="on""#));
        assert!(!html.contains("checked"));
        assert!(!html.contains("disabled"));
        assert!(!html.contains("required"));
    }

    #[test]
    fn hidden_input_checked_disabled_required_are_present_attrs() {
        let html = render(&hidden_input("wifi", "on", true, true, true, vec![]));
        assert!(html.contains(r#"checked="""#));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"required="""#));
        assert!(html.contains(r#"data-required="""#));
    }

    #[test]
    fn hidden_input_omits_boolean_attrs_when_false() {
        let html = render(&hidden_input("wifi", "on", false, false, false, vec![]));
        assert!(!html.contains(r#"checked="""#));
        assert!(!html.contains(r#"disabled="""#));
        assert!(!html.contains(r#"required="""#));
        assert!(!html.contains("data-required"));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側 attrs の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            false,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- Switch: dispatch 統合 ---

    #[test]
    fn switch_default_is_unchecked() {
        assert!(!Switch::default().is_checked());
    }

    #[test]
    fn switch_dispatch_toggle_changes_data_state() {
        let mut s = Switch::default();
        assert!(render(&s.root(false, vec![], vec![])).contains(r#"data-state="unchecked""#));

        assert!(dispatch(&mut s, "toggle", ""));
        assert!(render(&s.root(false, vec![], vec![])).contains(r#"data-state="checked""#));
        assert!(render(&s.control(false, vec![], vec![])).contains(r#"data-state="checked""#));
        assert!(render(&s.thumb(vec![], vec![])).contains(r#"data-state="checked""#));
        assert!(render(&s.label(vec![], vec![])).contains(r#"data-state="checked""#));
        assert!(
            render(&s.hidden_input("wifi", "on", false, false, vec![])).contains(r#"checked="""#)
        );
    }

    #[test]
    fn switch_dispatch_check_and_uncheck() {
        let mut s = Switch::default();
        assert!(dispatch(&mut s, "check", ""));
        assert!(s.is_checked());
        assert!(dispatch(&mut s, "uncheck", ""));
        assert!(!s.is_checked());
    }

    #[test]
    fn switch_dispatch_ignores_unknown_action() {
        let mut s = Switch::new(true);
        assert!(!dispatch(&mut s, "no_such_action", "x"));
        assert!(s.is_checked());
    }

    // --- Switch: SSR 状態なし初期描画 ---

    #[test]
    fn switch_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Switch::default().view());
        assert!(rendered.contains(r#"data-state="unchecked""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- Switch: hydration 経路 ---

    #[test]
    fn switch_hydration_round_trip() {
        let s = Switch::new(true);
        let rendered = render(&render_for_hydration(&s));
        assert!(rendered.contains(r#"data-hydrate-checked="checked""#));

        let restored = Switch::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }

    #[test]
    fn switch_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Switch::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-checked".to_string())
        );
    }

    #[test]
    fn switch_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["CHECKED", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-checked".to_string(), bogus.to_string())];
            let err = Switch::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: name/value/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn hidden_input_name_value_payload_is_escaped_on_render() {
        let html = render(&hidden_input(
            ATTR_BREAK_PAYLOAD,
            ATTR_BREAK_PAYLOAD,
            false,
            false,
            false,
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            false,
            false,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&label(
            true,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn switch_xss_payload_in_hydration_checked_is_rejected_not_rendered() {
        // data-hydrate-checked はサーバーが state_str() から生成する固定語彙の
        // みを出力するため攻撃者が任意値を注入する経路はないが、クライアント
        // 改ざん入力の復元経路（from_hydration_attrs）が未知値を拒否することを
        // Switch 経由でも固定する。
        let attrs = vec![(
            "data-hydrate-checked".to_string(),
            "<script>alert(1)</script>".to_string(),
        )];
        let err = Switch::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

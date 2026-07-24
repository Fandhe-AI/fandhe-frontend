//! Toggle（押下状態を持つ 2 状態ボタン）headless コンポーネント
//! （イシュー #746、Phase 3「headless + styled」一括方式）。
//!
//! ark-ui の Toggle
//!（`.claude/skills/ark-ui/references/components/disclosure/toggle.md`）を
//! 参考に、Root / Indicator の 2 anatomy パーツと、[`crate::state::Checkable`]
//! を埋め込んだ押下状態機械 [`Toggle`] を提供する。
//!
//! # Switch との意味論差
//!
//! [`crate::switch::Switch`] と同じ [`crate::state::Checkable`]
//! （checked/unchecked の 2 値状態機械）を埋め込むが、表す意味論・公開 HTML
//! は異なる:
//!
//! - **Switch**: 「オン/オフ設定」を表す。ネイティブ `<input
//!   type="checkbox" role="switch">`（[`crate::switch::hidden_input`]）を
//!   持ち、フォーム送信に参加する。`data-state` は `"checked"`/`"unchecked"`
//!   （[`crate::state::checked_data_state`]）。
//! - **Toggle**: 「ボタンの押下状態」を表す。[`root`] 自身がネイティブ
//!   `<button type="button">` であり、hidden input を持たずフォーム送信に
//!   参加しない（ark-ui 準拠、ボタンは値を持たない）。`data-state` は
//!   `"on"`/`"off"`（[`crate::state::pressed_data_state`]）で、`aria-pressed`
//!   （`"true"`/`"false"`）と `data-pressed`（存在属性）を併記する。
//!
//! 内部の状態機械は両者とも同型（checked/unchecked の 2 値）であるため、
//! [`state::Checkable`] を再利用し状態機械の分裂を防ぐ（イシュー #595 の
//! 共通化方針を踏襲）。ただし公開 HTML の `data-state` 語彙は分離するため、
//! [`Toggle`] は [`crate::state::checked_data_state`] を直接使わず
//! [`crate::state::pressed_data_state`] へ変換して出力する。
//!
//! **hydration ワイヤ値についての注記**: [`Toggle`] の
//! [`fandhe_frontend_interactive::Hydrate`] 実装は [`Checkable`] へ全委譲する
//! ため、`data-hydrate-checked` 属性の値語彙は `Checkable` 由来の
//! `"checked"`/`"unchecked"` のままである（表示語彙の `"on"`/`"off"` とは
//! 異なる）。これは状態機械分裂の防止（イシュー #595）を優先した設計判断で
//! あり、公開 HTML（`data-state`/`aria-pressed`/`data-pressed`）とハイドレー
//! ション属性（`data-hydrate-checked`）の語彙が異なる点に注意する
//! （[`Switch`](crate::switch::Switch) の hydration も同一のワイヤ値を使う）。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`indicator`]、純粋関数で完結）を
//! 直接呼んで組み立てる。CSR/hydration は [`Toggle`] を経由し、dispatch
//! （`"check"`/`"uncheck"`/`"toggle"`）で状態遷移する。
//! `fandhe-frontend-pre-styled-ui`（イシュー #746）が本モジュールを呼んで
//! スタイル済み Toggle を組み立てる想定である。
//!
//! # out-of-scope（本イシュー #746 のスコープ外）
//!
//! - **キーボードナビゲーション**: [`root`] はネイティブ `<button>` のため
//!   単体では Tab/Space/Enter がブラウザ既定動作で成立するが、複数 Toggle
//!   間の roving focus（矢印キー移動）は不要（単体コンポーネントのため）。
//! - **`indicator` の表示切り替え**: [`indicator`] は `data-state` のみを
//!   出力する最小主義パーツであり、on/off に応じた表示/非表示の切り替えは
//!   `fandhe-frontend-pre-styled-ui` の CSS（`[data-state="off"]` セレクタ）
//!   の責務とする（Collapsible の `indicator` と同じ最小主義）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`type`）はすべて `&'static str` リテラルで
//!   固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の既存不変
//!   条件をそのまま継承する）。
//! - 動的値（呼び出し側 `attrs`/`children` テキスト）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"on"`/`"off"`）は [`crate::state`]
//!   （[`crate::state::pressed_data_state`]）が一元管理し、本モジュールは
//!   パーツ関数間で分裂させない。
//! - hydration 属性（`data-hydrate-checked`）はクライアント側で改ざんされ
//!   うる入力として扱う。[`Toggle`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は [`crate::state::Checkable`]
//!   へ全委譲することで、panic せず `HydrateError` を返す既存保証をそのまま
//!   継承する。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::aria_pressed;
use crate::data_attrs::{data_disabled, data_pressed, data_state};
use crate::state::{pressed_data_state, Checkable};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Toggle の anatomy（`data-scope="toggle"`）。
const ANATOMY: Anatomy = anatomy("toggle");

/// Root パーツ（`button type="button"`）。
///
/// `aria-pressed`・`data-state`（`"on"`/`"off"`）・`data-pressed`（存在
/// 属性）・`disabled`/`data-disabled` を出力する。ネイティブ `<button>` の
/// ため、フォーカス・クリック・Space/Enter キー操作はブラウザ既定動作で
/// 成立する（hidden input を介さない点が [`crate::switch::root`] との違い、
/// モジュール doc 参照）。
#[must_use]
pub fn root<'a>(
    pressed: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_pressed(pressed),
        data_state(pressed_data_state(pressed)),
    ];
    merged.extend(data_pressed(pressed));
    merged.extend(data_disabled(disabled));
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("root", "button", merged, children)
}

/// Indicator パーツ（`span`）。`data-state` のみを反映する最小主義な装飾用
/// パーツ（[`crate::collapsible::indicator`] と同じ最小主義。on/off に
/// 応じた表示/非表示切り替えは styled 層 CSS の責務、モジュール doc 参照）。
#[must_use]
pub fn indicator<'a>(pressed: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(pressed_data_state(pressed))];
    merged.extend(attrs);
    ANATOMY.part("indicator", "span", merged, children)
}

/// Toggle のアクション（WASM 境界の文字列 dispatch と
/// [`Toggle::decode_action`] で接続する）。payload は使用しない。
///
/// [`crate::state::CheckableAction`] の互換 re-export（[`crate::switch::SwitchAction`]
/// と同型。[`crate::state::Checkable`] を埋め込んだ結果としてアクション語彙
/// も共有する。「オン/オフ」ではなく「押下 (Pressed)」の意味論だが、遷移の
/// 形が同一のため型を分離する理由がない）。
pub use crate::state::CheckableAction as ToggleAction;

/// Toggle の押下状態機械。
///
/// [`crate::state::Checkable`] をフィールドとして埋め込み（[`crate::switch::Switch`]
/// と同じ様式）、`data-state`（`"on"`/`"off"`）と実際の押下状態の整合を型
/// レベルで保証する入口として、各パーツ関数（[`root`]/[`indicator`]）へ
/// `self.is_pressed()` を注入する利便メソッドを提供する。SSR での自由関数
/// 直接利用（本型を経由しない構成）も引き続き可能。`Default` は未押下
/// （SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Toggle {
    checkable: Checkable,
}

impl Toggle {
    /// `data-hydrate-checked` 属性名のフィールド部分（[`Checkable::FIELD_CHECKED`]
    /// と同一値。モジュール doc「hydration ワイヤ値についての注記」参照）。
    pub const FIELD_CHECKED: &'static str = Checkable::FIELD_CHECKED;

    /// 指定した初期状態で Toggle を生成する。
    #[must_use]
    pub fn new(pressed: bool) -> Self {
        Self {
            checkable: Checkable::new(pressed),
        }
    }

    /// 現在押下されているかどうか。
    #[must_use]
    pub fn is_pressed(&self) -> bool {
        self.checkable.is_checked()
    }

    /// 現在の `data-state` 属性値（`"on"`/`"off"`）。
    #[must_use]
    pub fn data_state(&self) -> &'static str {
        pressed_data_state(self.checkable.is_checked())
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.checkable.is_checked(), disabled, attrs, children)
    }

    /// [`indicator`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn indicator<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        indicator(self.checkable.is_checked(), attrs, children)
    }
}

impl Component for Toggle {
    type Action = ToggleAction;

    fn update(&mut self, action: ToggleAction) {
        self.checkable.update(action);
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > indicator）。公開 UI としての利用は想定しない
    /// （実際の UI 構築は §パーツ関数群を呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        self.root(
            false,
            Vec::new(),
            vec![indicator(
                self.checkable.is_checked(),
                Vec::new(),
                Vec::new(),
            )],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<ToggleAction> {
        Checkable::decode_action(name, payload)
    }
}

impl Hydrate for Toggle {
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
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/data-state/ARIA 出力 ---

    #[test]
    fn root_outputs_scope_part_type_and_off_state() {
        let html = render(&root(false, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="toggle""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains("<button"));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-state="off""#));
        assert!(html.contains(r#"aria-pressed="false""#));
        assert!(!html.contains("data-pressed"));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn root_pressed_true_outputs_on_state_and_data_pressed() {
        let html = render(&root(true, false, vec![], vec![]));
        assert!(html.contains(r#"data-state="on""#));
        assert!(html.contains(r#"aria-pressed="true""#));
        assert!(html.contains(r#"data-pressed="""#));
    }

    #[test]
    fn root_disabled_true_adds_data_disabled_and_native_disabled() {
        let html = render(&root(false, true, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"disabled=""#));
    }

    #[test]
    fn indicator_outputs_scope_part_and_state_only() {
        let html = render(&indicator(true, vec![], vec![]));
        assert!(html.contains(r#"data-scope="toggle""#));
        assert!(html.contains(r#"data-part="indicator""#));
        assert!(html.contains(r#"data-state="on""#));
        assert!(!html.contains("aria-pressed"));
    }

    #[test]
    fn indicator_carries_children() {
        let html = render(&indicator(false, vec![], vec![text("Bold")]));
        assert!(html.contains("Bold"));
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
        assert!(html.contains(r#"data-scope="toggle""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- Toggle: dispatch 統合 ---

    #[test]
    fn toggle_default_is_unpressed() {
        assert!(!Toggle::default().is_pressed());
    }

    #[test]
    fn toggle_dispatch_toggle_changes_data_state() {
        let mut t = Toggle::default();
        assert!(render(&t.root(false, vec![], vec![])).contains(r#"data-state="off""#));

        assert!(dispatch(&mut t, "toggle", ""));
        assert!(render(&t.root(false, vec![], vec![])).contains(r#"data-state="on""#));
        assert!(render(&t.indicator(vec![], vec![])).contains(r#"data-state="on""#));
    }

    #[test]
    fn toggle_dispatch_check_and_uncheck() {
        let mut t = Toggle::default();
        assert!(dispatch(&mut t, "check", ""));
        assert!(t.is_pressed());
        assert!(dispatch(&mut t, "uncheck", ""));
        assert!(!t.is_pressed());
    }

    #[test]
    fn toggle_dispatch_ignores_unknown_action() {
        let mut t = Toggle::new(true);
        assert!(!dispatch(&mut t, "no_such_action", "x"));
        assert!(t.is_pressed());
    }

    // --- Toggle: SSR 状態なし初期描画 ---

    #[test]
    fn toggle_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Toggle::default().view());
        assert!(rendered.contains(r#"data-state="off""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    #[test]
    fn toggle_view_root_is_element_for_render_for_hydration() {
        let node = Toggle::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    // --- Toggle: hydration 経路（ワイヤ値は Checkable 由来の checked/unchecked） ---

    #[test]
    fn toggle_hydration_round_trip() {
        let t = Toggle::new(true);
        let rendered = render(&render_for_hydration(&t));
        assert!(rendered.contains(r#"data-hydrate-checked="checked""#));

        let restored = Toggle::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn toggle_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Toggle::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-checked".to_string())
        );
    }

    #[test]
    fn toggle_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["ON", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-checked".to_string(), bogus.to_string())];
            let err = Toggle::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: 呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

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
        let html = render(&indicator(
            true,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn toggle_xss_payload_in_hydration_checked_is_rejected_not_rendered() {
        // data-hydrate-checked はサーバーが state_str() から生成する固定語彙の
        // みを出力するため攻撃者が任意値を注入する経路はないが、クライアント
        // 改ざん入力の復元経路（from_hydration_attrs）が未知値を拒否することを
        // Toggle 経由でも固定する。
        let attrs = vec![(
            "data-hydrate-checked".to_string(),
            "<script>alert(1)</script>".to_string(),
        )];
        let err = Toggle::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

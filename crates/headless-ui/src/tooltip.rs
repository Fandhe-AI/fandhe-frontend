//! Tooltip（吹き出しヒント）headless コンポーネント（イシュー #533、親 #530）。
//!
//! ark-ui の Tooltip
//!（`.claude/skills/ark-ui/references/components/overlays/tooltip.md`）を
//! 参考に、Root / Trigger / Positioner / Content / Arrow / ArrowTip の
//! 6 anatomy パーツと、Phase 1（#524）の [`crate::state::Disclosure`] を
//! 埋め込んだ開閉状態機械 [`Tooltip`] を提供する。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`trigger`]/[`positioner`]/
//! [`content`]/[`arrow`]/[`arrow_tip`]、純粋関数で完結）を直接呼んで
//! 組み立てる。CSR/hydration は [`Tooltip`]
//! （[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"open"`/`"close"`/`"toggle"`）で状態遷移する。`fandhe-frontend-pre-styled-ui`
//! （#546〜）が本モジュールを呼んでスタイル済み Tooltip を組み立てる想定である。
//!
//! # WAI-ARIA tooltip パターンと [`crate::collapsible`] との違い
//!
//! Tooltip は WAI-ARIA tooltip パターンに従い、[`trigger`] は
//! `aria-describedby` で [`content`] と関連付ける（`role="tooltip"` は
//! `content` 側が持つ）。Disclosure 系（[`crate::collapsible`] 等）が使う
//! `aria-expanded`/`aria-controls` は tooltip パターンでは使用しない
//! （trigger 自体が展開可能なウィジェットではなく、tooltip は補助的な
//! 説明の開示であるため）。
//!
//! # スコープ外（out-of-scope）
//!
//! `openDelay`/`closeDelay`（表示・非表示までの遅延タイマー）・
//! `interactive`（tooltip 内へのポインタ移動時の維持）・`closeOnEscape`・
//! `positioning`（フローティング位置計算）は、タイマーやポインタ座標などの
//! クライアントサイド実行時挙動であり、headless な anatomy/状態機械を
//! 提供する本イシューのスコープ外とする（`fandhe-frontend-wasm-full`/
//! `fandhe-frontend-wasm-thin` 層または styled 層側の実装課題として別途
//! 検討する）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`hidden`/`disabled`/`id`）は
//!   すべて `&'static str` リテラルで固定しており、動的値が属性名スロットへ
//!   混入する経路はない（[`crate::anatomy`]/[`crate::aria`]/
//!   [`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（`describedby`/`id`/呼び出し側 `attrs`/`children` テキスト）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は [`crate::state::OpenState`]
//!   に一元化し、本モジュールで独自の値を作らない。
//! - hydration 属性（`data-hydrate-state`）はクライアント側で改ざんされうる
//!   入力として扱う。[`Tooltip`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は
//!   [`crate::state::Disclosure`] へ全委譲することで、panic せず
//!   `HydrateError` を返す既存保証をそのまま継承する。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_describedby, aria_hidden, role};
use crate::data_attrs::{data_disabled, data_state};
use crate::state::{Disclosure, DisclosureAction, OpenState};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Tooltip の anatomy（`data-scope="tooltip"`）。
const ANATOMY: Anatomy = anatomy("tooltip");

/// Root パーツ（`div`）。開閉状態を `data-state` へ反映する。
#[must_use]
pub fn root<'a>(state: OpenState, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Trigger パーツ（`button`）。
///
/// フォーム内配置時の意図しない submit を防ぐため `type="button"` を固定で
/// 付与する（A05 セキュリティ設定ミス対策）。`describedby` が `Some` のとき
/// `aria-describedby` で [`content`] と関連付ける（WAI-ARIA tooltip
/// パターン。`aria-expanded`/`aria-controls` は使用しない。
/// [`crate::collapsible::trigger`] との違いはモジュール doc §WAI-ARIA
/// tooltip パターン参照）。`disabled` はネイティブ `disabled` 存在属性と
/// `data-disabled` の両方へ反映する。
#[must_use]
pub fn trigger<'a>(
    state: OpenState,
    disabled: bool,
    describedby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("type", "button"), data_state(state.as_data_state())];
    if let Some(id) = describedby {
        merged.push(aria_describedby(id));
    }
    merged.extend(data_disabled(disabled));
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// Positioner パーツ（`div`）。フローティング位置計算はスコープ外
/// （モジュール doc §スコープ外参照）であり、本関数は `data-scope`/
/// `data-part` のみを付与する位置決めラッパーである。
#[must_use]
pub fn positioner<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("positioner", "div", attrs, children)
}

/// Content パーツ（`div`）。
///
/// `role="tooltip"`（[`crate::aria::role`] 使用）を固定で付与する。closed
/// のとき `hidden` 存在属性を付与し、JS なしの SSR でも閉状態を表現する
/// （アニメーション対応の CSS 変数出力等はスコープ外。モジュール doc
/// §スコープ外参照）。`id` が `Some` のとき [`trigger`] の `describedby`
/// と対で `aria-describedby` 関連付けを成立させる。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![role("tooltip"), data_state(state.as_data_state())];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("content", "div", merged, children)
}

/// Arrow パーツ（`div`）。装飾用のみであり、スクリーンリーダーへ読み上げ
/// させないため `aria-hidden="true"` を固定で付与する。
#[must_use]
pub fn arrow<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("arrow", "div", merged, children)
}

/// ArrowTip パーツ（`div`）。[`arrow`] 同様に装飾用のみであり、
/// `aria-hidden="true"` を固定で付与する。
#[must_use]
pub fn arrow_tip<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("arrow-tip", "div", merged, children)
}

/// [`Disclosure`]（#524）を埋め込んだ Tooltip の状態機械。
///
/// `data-state` と実際の開閉状態の整合を型レベルで保証する入口として、
/// 各パーツ関数（[`root`]/[`trigger`]/[`content`]）へ `self.state()` を
/// 注入する利便メソッドを提供する（[`positioner`]/[`arrow`]/[`arrow_tip`]
/// は状態非依存のため利便メソッドを持たない）。SSR での自由関数直接利用
/// （本型を経由しない構成）も引き続き可能。`Default` は
/// [`OpenState::Closed`]（SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tooltip {
    disclosure: Disclosure,
}

impl Tooltip {
    /// 指定した初期状態で Tooltip を生成する。
    #[must_use]
    pub fn new(initial: OpenState) -> Self {
        Self {
            disclosure: Disclosure::new(initial),
        }
    }

    /// 現在の開閉状態。
    #[must_use]
    pub fn state(&self) -> OpenState {
        self.disclosure.state()
    }

    /// 現在の `data-state` 属性値（`"open"`/`"closed"`）。
    #[must_use]
    pub fn data_state(&self) -> &'static str {
        self.disclosure.data_state()
    }

    /// 開いているかどうか。
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.disclosure.state().is_open()
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        root(self.state(), attrs, children)
    }

    /// [`trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        disabled: bool,
        describedby: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(self.state(), disabled, describedby, attrs, children)
    }

    /// [`content`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn content<'a>(
        &self,
        id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(self.state(), id, attrs, children)
    }
}

impl Component for Tooltip {
    type Action = DisclosureAction;

    fn update(&mut self, action: DisclosureAction) {
        self.disclosure.update(action);
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > trigger + positioner > content、children 空・id
    /// なし）。[`Disclosure::view`] と同じ位置付けであり、公開 UI としての
    /// 利用は想定しない（実際の UI 構築は §パーツ関数群を呼び出し側が
    /// 組み合わせる）。
    fn view(&self) -> Node {
        let state = self.state();
        self.root(
            Vec::new(),
            vec![
                trigger(state, false, None, Vec::new(), Vec::new()),
                positioner(
                    Vec::new(),
                    vec![content(state, None, Vec::new(), Vec::new())],
                ),
            ],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<DisclosureAction> {
        Disclosure::decode_action(name, payload)
    }
}

impl Hydrate for Tooltip {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.disclosure.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            disclosure: Disclosure::from_hydration_attrs(attrs)?,
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
        assert!(html.contains(r#"data-scope="tooltip""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn trigger_has_type_button_and_no_aria_expanded() {
        let html = render(&trigger(OpenState::Closed, false, None, vec![], vec![]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-state="closed""#));
        // tooltip パターンでは aria-expanded/aria-controls を使わない
        // （collapsible との違い、モジュール doc 参照）。
        assert!(!html.contains("aria-expanded"));
        assert!(!html.contains("aria-controls"));
        assert!(!html.contains("aria-describedby"));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn trigger_describedby_some_outputs_aria_describedby() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            Some("tooltip-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-describedby="tooltip-1""#));
    }

    #[test]
    fn trigger_disabled_true_adds_native_and_data_disabled() {
        let html = render(&trigger(OpenState::Closed, true, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"disabled="""#));
    }

    #[test]
    fn trigger_disabled_false_omits_both_disabled_attrs() {
        let html = render(&trigger(OpenState::Closed, false, None, vec![], vec![]));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains(r#" disabled"#));
    }

    #[test]
    fn positioner_outputs_scope_and_part_only() {
        let html = render(&positioner(vec![], vec![]));
        assert!(html.contains(r#"data-scope="tooltip""#));
        assert!(html.contains(r#"data-part="positioner""#));
    }

    #[test]
    fn content_has_role_tooltip() {
        let html = render(&content(OpenState::Open, None, vec![], vec![]));
        assert!(html.contains(r#"role="tooltip""#));
    }

    #[test]
    fn content_closed_has_hidden_attr_open_does_not() {
        let closed = render(&content(OpenState::Closed, None, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&content(OpenState::Open, None, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_id_some_outputs_id_attribute() {
        let html = render(&content(OpenState::Open, Some("tooltip-1"), vec![], vec![]));
        assert!(html.contains(r#"id="tooltip-1""#));
    }

    #[test]
    fn arrow_has_aria_hidden_true() {
        let html = render(&arrow(vec![], vec![]));
        assert!(html.contains(r#"data-part="arrow""#));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    #[test]
    fn arrow_tip_has_kebab_case_part_and_aria_hidden_true() {
        let html = render(&arrow_tip(vec![], vec![]));
        assert!(html.contains(r#"data-part="arrow-tip""#));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            OpenState::Closed,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="tooltip""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- trigger + content の aria-describedby/id 対応 ---

    #[test]
    fn trigger_describedby_and_content_id_correspond() {
        let trigger_html = render(&trigger(OpenState::Open, false, Some("t1"), vec![], vec![]));
        let content_html = render(&content(OpenState::Open, Some("t1"), vec![], vec![]));
        assert!(trigger_html.contains(r#"aria-describedby="t1""#));
        assert!(content_html.contains(r#"id="t1""#));
    }

    // --- Tooltip: dispatch 統合 ---

    #[test]
    fn tooltip_default_is_closed() {
        assert_eq!(Tooltip::default().state(), OpenState::Closed);
    }

    #[test]
    fn tooltip_dispatch_toggle_changes_data_state() {
        let mut t = Tooltip::default();
        assert!(render(&t.root(vec![], vec![])).contains(r#"data-state="closed""#));

        assert!(dispatch(&mut t, "toggle", ""));
        assert!(render(&t.root(vec![], vec![])).contains(r#"data-state="open""#));
        assert!(render(&t.content(None, vec![], vec![])).contains(r#"data-state="open""#));
        assert!(!render(&t.content(None, vec![], vec![])).contains("hidden"));
    }

    #[test]
    fn tooltip_dispatch_open_and_close() {
        let mut t = Tooltip::default();
        assert!(dispatch(&mut t, "open", ""));
        assert_eq!(t.state(), OpenState::Open);
        assert!(dispatch(&mut t, "close", ""));
        assert_eq!(t.state(), OpenState::Closed);
    }

    #[test]
    fn tooltip_dispatch_ignores_unknown_action() {
        let mut t = Tooltip::new(OpenState::Open);
        assert!(!dispatch(&mut t, "no_such_action", "x"));
        assert_eq!(t.state(), OpenState::Open);
    }

    // --- Tooltip: SSR 状態なし初期描画 ---

    #[test]
    fn tooltip_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Tooltip::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- Tooltip: hydration 経路 ---

    #[test]
    fn tooltip_hydration_round_trip() {
        let t = Tooltip::new(OpenState::Open);
        let rendered = render(&render_for_hydration(&t));
        assert!(rendered.contains(r#"data-hydrate-state="open""#));

        let restored = Tooltip::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn tooltip_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Tooltip::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-state".to_string())
        );
    }

    #[test]
    fn tooltip_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
            let err = Tooltip::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: describedby/id/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn trigger_describedby_payload_is_escaped_on_render() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn content_id_payload_is_escaped_on_render() {
        let html = render(&content(
            OpenState::Open,
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
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
        let html = render(&content(
            OpenState::Open,
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn tooltip_xss_payload_in_hydration_state_is_rejected_not_rendered() {
        // data-hydrate-state はサーバーが state.as_data_state() から生成する
        // 固定語彙のみを出力するため攻撃者が任意値を注入する経路はないが、
        // クライアント改ざん入力の復元経路（from_hydration_attrs）が
        // 未知値を拒否することを Tooltip 経由でも固定する。
        let attrs = vec![(
            "data-hydrate-state".to_string(),
            "<script>alert(1)</script>".to_string(),
        )];
        let err = Tooltip::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

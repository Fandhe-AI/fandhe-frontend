//! ActionBar（複数選択操作バー）headless コンポーネント（イシュー #762、親トラッキング #520）。
//!
//! chakra-ui の ActionBar（`.claude/skills/chakra-ui` overlays/action-bar 相当）を
//! 参考に、Root / Positioner / Content / SelectionTrigger / Separator /
//! CloseTrigger の 6 anatomy パーツと、Phase 1（#524）の
//! [`crate::state::Disclosure`] を埋め込んだ開閉状態機械 [`ActionBar`] を
//! 提供する。構造上最も近い先行例は [`crate::dialog::Dialog`]（`Disclosure`
//! 埋め込み + positioner/close-trigger 構成）であり、本モジュールはそのパターン
//! に完全準拠する。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（純粋関数で完結）を直接呼んで組み立てる。
//! CSR/hydration は [`ActionBar`]（[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"open"`/`"close"`/`"toggle"`）で状態遷移する。
//! `fandhe-frontend-pre-styled-ui` が本モジュールを呼んでスタイル済み
//! ActionBar を組み立てる想定である。
//!
//! # 選択件数から open を導出する糖衣 API は持たない
//!
//! chakra-ui は `open={selection.size > 0}` のように選択件数から開閉を導出
//! するが、本モジュールはそれを行わない。「選択操作 → 開閉状態の決定」は
//! 呼び出し側（アプリケーション状態を持つ層）の責務とし、状態機械へは
//! [`crate::state::Disclosure`] の既存契約（`"open"`/`"close"`/`"toggle"`
//! dispatch）のみを持ち込む（既存 Disclosure 契約の一貫性優先）。
//!
//! # スコープ外（out-of-scope-tracking 対応）
//!
//! - Portal 描画・外側クリックでの閉鎖（`closeOnInteractOutside`）・
//!   アニメーションは JS ランタイム側の責務であり本イシューのスコープ外
//!   （SSR/属性出力のみ、[`crate::dialog`] と同じ判断）。
//! - `placement` variant（`bottom-start`/`bottom-end` 等）: 既定の bottom
//!   中央固定のみ実装する。variant 追加は styled 層の `SlotRecipe::variant`
//!   で後続可能。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`hidden`）はすべて `&'static str`
//!   リテラルで固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`mod@crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の既存
//!   不変条件をそのまま継承する）。
//! - 動的値（`label`・呼び出し側 `attrs`/`children` テキスト）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は [`crate::state::OpenState`]
//!   に一元化し、本モジュールで独自の値を作らない。
//! - hydration 属性（`data-hydrate-state`）はクライアント側で改ざんされうる
//!   入力として扱う。[`ActionBar`] の [`fandhe_frontend_interactive::Hydrate`]
//!   実装は [`crate::state::Disclosure`] へ全委譲することで、panic せず
//!   `HydrateError` を返す既存保証をそのまま継承する。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_label, aria_orientation, role};
use crate::data_attrs::{data_state, Orientation};
use crate::state::{Disclosure, DisclosureAction, OpenState};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// ActionBar の anatomy（`data-scope="action-bar"`）。
const ANATOMY: Anatomy = anatomy("action-bar");

/// Root パーツ（`div`）。開閉状態を `data-*` へ反映する。
#[must_use]
pub fn root<'a>(state: OpenState, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Positioner パーツ（`div`）。[`content`] の画面下部固定配置用ラッパー。
///
/// closed のとき `hidden` 存在属性を付与する。
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

/// Content パーツ（`div`）。ActionBar 本体。
///
/// `role="toolbar"` + `aria-label`（`label` は選択操作バーの読み上げ名、
/// 呼び出し側が渡す必須引数）を付与する。closed のとき `hidden` 存在属性を
/// 付与する。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("toolbar"),
        aria_label(label),
        data_state(state.as_data_state()),
    ];
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("content", "div", merged, children)
}

/// SelectionTrigger パーツ（`button`）。選択件数表示・選択解除等の操作を
/// 想定するボタン。選択件数テキスト（例: "3 selected"）は呼び出し側が
/// `children` の `text()` で渡す（既定エスケープ経由）。
///
/// フォーム内配置時の意図しない submit を防ぐため `type="button"` を固定で
/// 付与する（A05 セキュリティ設定ミス対策、[`crate::dialog::trigger`] と
/// 同じ判断）。
#[must_use]
pub fn selection_trigger<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    merged.extend(attrs);
    ANATOMY.part("selection-trigger", "button", merged, children)
}

/// Separator パーツ（`div`）。ツールバー内のボタン群を視覚的に区切る。
///
/// `role="separator"` + `aria-orientation="vertical"` を固定で付与する
/// （ActionBar のボタン列は横並びであり、区切り線は縦向きになる）。
#[must_use]
pub fn separator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![role("separator"), aria_orientation(Orientation::Vertical)];
    merged.extend(attrs);
    ANATOMY.part("separator", "div", merged, children)
}

/// CloseTrigger パーツ（`button`）。ラベル（`aria-label`/children）は
/// 呼び出し側が `attrs`/`children` で付与する。
///
/// [`selection_trigger`] と同じく `type="button"` を固定で付与する。
#[must_use]
pub fn close_trigger<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    merged.extend(attrs);
    ANATOMY.part("close-trigger", "button", merged, children)
}

/// [`Disclosure`]（#524）を埋め込んだ ActionBar の状態機械。
///
/// `data-state` と実際の開閉状態の整合を型レベルで保証する入口として、
/// 各パーツ関数（[`root`]/[`positioner`]/[`content`]）へ `self.state()` を
/// 注入する利便メソッドを提供する。SSR での自由関数直接利用（本型を経由
/// しない構成）も引き続き可能。`Default` は [`OpenState::Closed`]（選択なし
/// の初期状態に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActionBar {
    disclosure: Disclosure,
}

impl ActionBar {
    /// 指定した初期状態で ActionBar を生成する。
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

    /// [`positioner`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn positioner<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        positioner(self.state(), attrs, children)
    }

    /// [`content`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn content<'a>(
        &self,
        label: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(self.state(), label, attrs, children)
    }
}

impl Component for ActionBar {
    type Action = DisclosureAction;

    fn update(&mut self, action: DisclosureAction) {
        self.disclosure.update(action);
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > positioner(content(selection-trigger + separator +
    /// close-trigger))、children 空）。[`crate::dialog::Dialog::view`] と
    /// 同じ位置付けであり、公開 UI としての利用は想定しない（実際の UI
    /// 構築は §パーツ関数群を呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        let state = self.state();
        self.root(
            Vec::new(),
            vec![positioner(
                state,
                Vec::new(),
                vec![content(
                    state,
                    "",
                    Vec::new(),
                    vec![
                        selection_trigger(Vec::new(), Vec::new()),
                        separator(Vec::new(), Vec::new()),
                        close_trigger(Vec::new(), Vec::new()),
                    ],
                )],
            )],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<DisclosureAction> {
        Disclosure::decode_action(name, payload)
    }
}

impl Hydrate for ActionBar {
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
        assert!(html.contains(r#"data-scope="action-bar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn positioner_closed_has_hidden_open_does_not() {
        let closed = render(&positioner(OpenState::Closed, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));
        assert!(closed.contains(r#"data-state="closed""#));

        let open = render(&positioner(OpenState::Open, vec![], vec![]));
        assert!(!open.contains("hidden"));
        assert!(open.contains(r#"data-state="open""#));
    }

    #[test]
    fn content_has_role_toolbar_and_aria_label() {
        let html = render(&content(OpenState::Open, "3 selected", vec![], vec![]));
        assert!(html.contains(r#"role="toolbar""#));
        assert!(html.contains(r#"aria-label="3 selected""#));
        assert!(html.contains(r#"data-part="content""#));
    }

    #[test]
    fn content_closed_has_hidden_open_does_not() {
        let closed = render(&content(OpenState::Closed, "label", vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&content(OpenState::Open, "label", vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn selection_trigger_has_type_button() {
        let html = render(&selection_trigger(vec![], vec![text("3 selected")]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-part="selection-trigger""#));
        assert!(html.contains("3 selected"));
    }

    #[test]
    fn separator_has_role_and_vertical_orientation() {
        let html = render(&separator(vec![], vec![]));
        assert!(html.contains(r#"role="separator""#));
        assert!(html.contains(r#"aria-orientation="vertical""#));
        assert!(html.contains(r#"data-part="separator""#));
    }

    #[test]
    fn close_trigger_has_type_button() {
        let html = render(&close_trigger(vec![], vec![text("Close")]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-part="close-trigger""#));
        assert!(html.contains("Close"));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            OpenState::Closed,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="action-bar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- ActionBar: dispatch 統合 ---

    #[test]
    fn action_bar_default_is_closed() {
        assert_eq!(ActionBar::default().state(), OpenState::Closed);
    }

    #[test]
    fn action_bar_dispatch_toggle_changes_data_state() {
        let mut bar = ActionBar::default();
        assert!(render(&bar.root(vec![], vec![])).contains(r#"data-state="closed""#));

        assert!(dispatch(&mut bar, "toggle", ""));
        assert!(render(&bar.root(vec![], vec![])).contains(r#"data-state="open""#));
        assert!(render(&bar.positioner(vec![], vec![])).contains(r#"data-state="open""#));
        assert!(!render(&bar.positioner(vec![], vec![])).contains("hidden=\"\""));
    }

    #[test]
    fn action_bar_dispatch_open_and_close() {
        let mut bar = ActionBar::default();
        assert!(dispatch(&mut bar, "open", ""));
        assert_eq!(bar.state(), OpenState::Open);
        assert!(dispatch(&mut bar, "close", ""));
        assert_eq!(bar.state(), OpenState::Closed);
    }

    #[test]
    fn action_bar_dispatch_ignores_unknown_action() {
        let mut bar = ActionBar::new(OpenState::Open);
        assert!(!dispatch(&mut bar, "no_such_action", "x"));
        assert_eq!(bar.state(), OpenState::Open);
    }

    // --- ActionBar: SSR 状態なし初期描画 ---

    #[test]
    fn action_bar_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&ActionBar::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    #[test]
    fn action_bar_view_root_is_element_for_render_for_hydration() {
        let node = ActionBar::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    // --- ActionBar: hydration 経路 ---

    #[test]
    fn action_bar_hydration_round_trip() {
        let bar = ActionBar::new(OpenState::Open);
        let rendered = render(&render_for_hydration(&bar));
        assert!(rendered.contains(r#"data-hydrate-state="open""#));

        let restored = ActionBar::from_hydration_attrs(&bar.hydration_attrs()).unwrap();
        assert_eq!(restored, bar);
    }

    #[test]
    fn action_bar_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = ActionBar::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-state".to_string())
        );
    }

    #[test]
    fn action_bar_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
            let err = ActionBar::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: label/attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn content_label_payload_is_escaped_on_render() {
        let html = render(&content(
            OpenState::Open,
            ATTR_BREAK_PAYLOAD,
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
        let html = render(&close_trigger(
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn action_bar_xss_payload_in_hydration_state_is_rejected_not_rendered() {
        // data-hydrate-state はサーバーが state.as_data_state() から生成する
        // 固定語彙のみを出力するため攻撃者が任意値を注入する経路はないが、
        // クライアント改ざん入力の復元経路（from_hydration_attrs）が
        // 未知値を拒否することを ActionBar 経由でも固定する。
        let attrs = vec![(
            "data-hydrate-state".to_string(),
            "<script>alert(1)</script>".to_string(),
        )];
        let err = ActionBar::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

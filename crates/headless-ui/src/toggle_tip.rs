//! ToggleTip（クリック開閉の小型ヒント）headless コンポーネント（イシュー
//! #761、親トラッキング #520）。
//!
//! chakra-ui の ToggleTip
//!（`overlays/toggle-tip.md`）を参考に、Root / Trigger / Positioner /
//! Content / Arrow / ArrowTip の 6 anatomy パーツと、Phase 1（#524）の
//! [`crate::state::Disclosure`] を埋め込んだ開閉状態機械 [`ToggleTip`] を
//! 提供する。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`trigger`]/[`positioner`]/
//! [`content`]/[`arrow`]/[`arrow_tip`]、純粋関数で完結）を直接呼んで
//! 組み立てる。CSR/hydration は [`ToggleTip`]
//! （[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"open"`/`"close"`/`"toggle"`）で状態遷移する。`fandhe-frontend-pre-styled-ui`
//! が本モジュールを呼んでスタイル済み ToggleTip を組み立てる想定である。
//!
//! # ToggleTip・[`crate::tooltip`]・[`crate::popover`] の 3 者境界
//!
//! chakra-ui は ToggleTip を「見た目は Tooltip（小型・非モーダル）、挙動は
//! Popover（クリックで開閉し、明示的に閉じるまで持続）」の変種と位置づける。
//! 本クレートでの ARIA 表現もこの立ち位置に従い、[`crate::tooltip`]・
//! [`crate::popover`] のいずれとも異なる：
//!
//! - **[`crate::tooltip::Tooltip`]**（hover/focus 由来）: [`trigger`] は
//!   `aria-describedby` で [`content`] と関連付け、[`content`] は
//!   `role="tooltip"` を持つ。`aria-expanded`/`aria-controls` は使わない
//!   （trigger 自体が展開可能なウィジェットではないため）。
//! - **[`crate::popover::Popover`]**（クリック起点のオーバーレイ）: [`trigger`]
//!   は `aria-haspopup="dialog"` + `aria-expanded` + `aria-controls` を持つ。
//!   [`content`] は Title/Description/CloseTrigger を伴う対話的なダイアログ
//!   相当。
//! - **[`ToggleTip`]（本モジュール）**: [`trigger`] は `aria-expanded`（状態
//!   連動）+ `aria-controls`（`controls` が `Some` のとき）を持つが、
//!   `aria-haspopup` は付与しない（[`content`] は Title/Description/
//!   CloseTrigger を伴う dialog ではなく、簡潔な非対話テキストであるため）。
//!   [`content`] は `role="tooltip"` を持たない（WAI-ARIA の tooltip role は
//!   hover/focus + `aria-describedby` 前提であり、click 起点の disclosure
//!   パターンには適合しないため）。
//!
//! # anatomy: chakra-ui の 4 パーツ + Arrow/ArrowTip
//!
//! chakra-ui の sub-parts は Root/Trigger/Content/Positioner の 4 種だが、
//! 本モジュールは [`crate::tooltip`] と同じ視覚表現（矢印つき吹き出し）を
//! styled 層で再現できるよう、装飾専用（`aria-hidden="true"`）の
//! [`arrow`]/[`arrow_tip`] を tooltip と同型で加える。
//!
//! # スコープ外（out-of-scope）
//!
//! click-outside dismiss（トリガー外クリックでの自動閉鎖）・Escape 閉鎖は、
//! クライアントサイド実行時のイベント処理であり、headless な anatomy/
//! 状態機械を提供する本イシューのスコープ外とする（`fandhe-frontend-wasm-full`
//! の `overlay` モジュールへの `"toggle-tip"` scope 登録は別イシュー提案）。
//!
//! フローティング位置計算（Floating UI 相当の placement / CSS 変数出力）は
//! イシュー #590（親 #588）で [`crate::positioning`] として実装済みである。
//! [`positioner`]/[`arrow`]/[`arrow_tip`] は [`crate::tooltip`] と同じく
//! `attrs` 経由で `style`/`data-side`/`data-align` を受け取る薄いラッパーの
//! ままであり、計算自体は `fandhe-frontend-wasm-full`（`position` モジュール）
//! が [`crate::positioning::compute_position`] を呼び出して行う（本モジュール
//! 自体は `web-sys` 非依存を維持する）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`type`/`hidden`/`disabled`/`id`）はすべて
//!   `&'static str` リテラルで固定しており、動的値が属性名スロットへ混入する
//!   経路はない（[`crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の
//!   既存不変条件をそのまま継承する）。
//! - 動的値（`controls`/`id`/呼び出し側 `attrs`/`children` テキスト）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は [`crate::state::OpenState`]
//!   に一元化し、本モジュールで独自の値を作らない。
//! - hydration 属性（`data-hydrate-state`）はクライアント側で改ざんされうる
//!   入力として扱う。[`ToggleTip`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は
//!   [`crate::state::Disclosure`] へ全委譲することで、panic せず
//!   `HydrateError` を返す既存保証をそのまま継承する。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_controls, aria_expanded, aria_hidden};
use crate::data_attrs::{data_disabled, data_state};
use crate::state::{Disclosure, DisclosureAction, OpenState};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// ToggleTip の anatomy（`data-scope="toggle-tip"`）。
const ANATOMY: Anatomy = anatomy("toggle-tip");

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
/// 付与する（A05 セキュリティ設定ミス対策）。`aria-expanded` を状態連動で
/// 常に出力し、`controls` が `Some` のとき `aria-controls` で [`content`]
/// と関連付ける。モジュール doc §3 者境界の通り `aria-haspopup` は付与しない
/// （[`crate::popover::trigger`] との違い）。`disabled` はネイティブ
/// `disabled` 存在属性と `data-disabled` の両方へ反映する。
#[must_use]
pub fn trigger<'a>(
    state: OpenState,
    disabled: bool,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_expanded(state.is_open()),
        data_state(state.as_data_state()),
    ];
    if let Some(id) = controls {
        merged.push(aria_controls(id));
    }
    merged.extend(data_disabled(disabled));
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// Positioner パーツ（`div`）。位置計算自体は本関数の責務ではなく
/// [`crate::positioning::compute_position`]（#590）が担う。本関数は
/// `data-scope`/`data-part` に加え、呼び出し側が `attrs` 経由で渡す
/// `style`（`--fandhe-*` CSS 変数）・`data-side`/`data-align` をそのまま
/// 透過させる薄いラッパーである（[`crate::tooltip::positioner`] と同型、
/// モジュール doc §スコープ外参照）。
///
/// `state` から `data-state` を出力し、`fandhe-frontend-wasm-full` の
/// `reposition_all` が使う `[data-part="positioner"][data-state="open"]`
/// セレクタへ追加実装なしでマッチする（scope 非依存セレクタ、
/// [`crate::tooltip::positioner`] のレビュー指摘と同じ判断）。closed の
/// とき `hidden` 存在属性を付与し、arrow/arrow_tip が positioner 内に
/// ネストされる anatomy 構造上、closed 時にポインタ層を SSR/no-JS
/// マークアップへ表示させない（[`crate::popover::positioner`] と同じ判断）。
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
/// モジュール doc §3 者境界の通り `role="tooltip"` は付与しない（click
/// 起点の disclosure パターンであり、WAI-ARIA tooltip role の hover/focus
/// 前提に適合しないため）。closed のとき `hidden` 存在属性を付与し、JS
/// なしの SSR でも閉状態を表現する。`id` が `Some` のとき [`trigger`] の
/// `controls` と対で `aria-controls` 関連付けを成立させる。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
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

/// [`Disclosure`]（#524）を埋め込んだ ToggleTip の状態機械。
///
/// `data-state` と実際の開閉状態の整合を型レベルで保証する入口として、
/// 各パーツ関数（[`root`]/[`trigger`]/[`content`]/[`positioner`]）へ
/// `self.state()` を注入する利便メソッドを提供する（[`arrow`]/[`arrow_tip`]
/// は状態非依存のため利便メソッドを持たない）。SSR での自由関数直接利用
/// （本型を経由しない構成）も引き続き可能。`Default` は
/// [`OpenState::Closed`]（SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToggleTip {
    disclosure: Disclosure,
}

impl ToggleTip {
    /// 指定した初期状態で ToggleTip を生成する。
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
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(self.state(), disabled, controls, attrs, children)
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
        id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(self.state(), id, attrs, children)
    }
}

impl Component for ToggleTip {
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
                    state,
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

impl Hydrate for ToggleTip {
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

    // --- positioning（#590）接続 ---

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
            placement: Placement::new(Side::Top, Align::Start),
            offset: 0.0,
            flip: true,
            shift: true,
            same_width: false,
        };
        let resolved = compute_position(anchor, floating, viewport, &config, true);
        let style = css_vars_style(&resolved, anchor.width, config.same_width);
        let mut attrs: Vec<(&str, &str)> = vec![("style", &style)];
        attrs.extend(placement_attrs(resolved.placement));

        let html = render(&positioner(OpenState::Open, attrs, vec![]));
        assert!(html.contains("--fandhe-arrow-x:"));
        assert!(html.contains(r#"data-side="top""#));
        assert!(html.contains(r#"data-align="start""#));
    }

    // --- 各パーツの data-scope/data-part/data-state 出力 ---

    #[test]
    fn root_outputs_scope_part_and_state() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="toggle-tip""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn trigger_has_type_button_and_aria_expanded_but_no_haspopup() {
        let html = render(&trigger(OpenState::Closed, false, None, vec![], vec![]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(html.contains(r#"data-state="closed""#));
        // 3 者境界（モジュール doc）: Popover と異なり aria-haspopup は
        // 付与しない。Tooltip と異なり aria-describedby も使わない。
        assert!(!html.contains("aria-haspopup"));
        assert!(!html.contains("aria-describedby"));
        assert!(!html.contains("aria-controls"));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn trigger_aria_expanded_reflects_state() {
        let closed = render(&trigger(OpenState::Closed, false, None, vec![], vec![]));
        assert!(closed.contains(r#"aria-expanded="false""#));

        let open = render(&trigger(OpenState::Open, false, None, vec![], vec![]));
        assert!(open.contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn trigger_controls_some_outputs_aria_controls() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            Some("toggle-tip-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-controls="toggle-tip-1""#));
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
    fn positioner_outputs_scope_part_and_state() {
        let html = render(&positioner(OpenState::Open, vec![], vec![]));
        assert!(html.contains(r#"data-scope="toggle-tip""#));
        assert!(html.contains(r#"data-part="positioner""#));
        assert!(html.contains(r#"data-state="open""#));
    }

    #[test]
    fn positioner_closed_has_hidden_attr_open_does_not() {
        let closed = render(&positioner(OpenState::Closed, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&positioner(OpenState::Open, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_has_no_role_tooltip() {
        // 3 者境界（モジュール doc）: click 起点の disclosure パターンであり
        // WAI-ARIA tooltip role の hover/focus 前提に適合しないため付与しない。
        let html = render(&content(OpenState::Open, None, vec![], vec![]));
        assert!(!html.contains("role="));
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
        let html = render(&content(
            OpenState::Open,
            Some("toggle-tip-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="toggle-tip-1""#));
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
        assert!(html.contains(r#"data-scope="toggle-tip""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- trigger + content の aria-controls/id 対応 ---

    #[test]
    fn trigger_controls_and_content_id_correspond() {
        let trigger_html = render(&trigger(OpenState::Open, false, Some("t1"), vec![], vec![]));
        let content_html = render(&content(OpenState::Open, Some("t1"), vec![], vec![]));
        assert!(trigger_html.contains(r#"aria-controls="t1""#));
        assert!(content_html.contains(r#"id="t1""#));
    }

    // --- ToggleTip: dispatch 統合 ---

    #[test]
    fn toggle_tip_default_is_closed() {
        assert_eq!(ToggleTip::default().state(), OpenState::Closed);
    }

    #[test]
    fn toggle_tip_dispatch_toggle_changes_data_state() {
        let mut t = ToggleTip::default();
        assert!(render(&t.root(vec![], vec![])).contains(r#"data-state="closed""#));

        assert!(dispatch(&mut t, "toggle", ""));
        assert!(render(&t.root(vec![], vec![])).contains(r#"data-state="open""#));
        assert!(render(&t.content(None, vec![], vec![])).contains(r#"data-state="open""#));
        assert!(!render(&t.content(None, vec![], vec![])).contains("hidden"));
        assert!(render(&t.trigger(false, None, vec![], vec![])).contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn toggle_tip_dispatch_open_and_close() {
        let mut t = ToggleTip::default();
        assert!(dispatch(&mut t, "open", ""));
        assert_eq!(t.state(), OpenState::Open);
        assert!(dispatch(&mut t, "close", ""));
        assert_eq!(t.state(), OpenState::Closed);
    }

    #[test]
    fn toggle_tip_dispatch_ignores_unknown_action() {
        let mut t = ToggleTip::new(OpenState::Open);
        assert!(!dispatch(&mut t, "no_such_action", "x"));
        assert_eq!(t.state(), OpenState::Open);
    }

    // --- ToggleTip: SSR 状態なし初期描画 ---

    #[test]
    fn toggle_tip_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&ToggleTip::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- ToggleTip: hydration 経路 ---

    #[test]
    fn toggle_tip_hydration_round_trip() {
        let t = ToggleTip::new(OpenState::Open);
        let rendered = render(&render_for_hydration(&t));
        assert!(rendered.contains(r#"data-hydrate-state="open""#));

        let restored = ToggleTip::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn toggle_tip_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = ToggleTip::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-state".to_string())
        );
    }

    #[test]
    fn toggle_tip_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
            let err = ToggleTip::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: controls/id/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn trigger_controls_payload_is_escaped_on_render() {
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
    fn toggle_tip_xss_payload_in_hydration_state_is_rejected_not_rendered() {
        // data-hydrate-state はサーバーが state.as_data_state() から生成する
        // 固定語彙のみを出力するため攻撃者が任意値を注入する経路はないが、
        // クライアント改ざん入力の復元経路（from_hydration_attrs）が
        // 未知値を拒否することを ToggleTip 経由でも固定する。
        let attrs = vec![(
            "data-hydrate-state".to_string(),
            "<script>alert(1)</script>".to_string(),
        )];
        let err = ToggleTip::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

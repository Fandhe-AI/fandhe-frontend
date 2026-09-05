//! Collapsible（開閉パネル）headless コンポーネント（イシュー #529、親 #526。
//! anatomy / `data-*` / キーボード操作の参考サイト突合はイシュー #1637）。
//!
//! ark-ui の Collapsible
//!（`.claude/skills/ark-ui/references/components/disclosure/collapsible.md`）と
//! Radix Primitives Collapsible（`docs/design/radix-primitives-inventory.md`）を
//! 参考に、Root / Trigger / Indicator / Content の 4 anatomy パーツと、
//! Phase 1（#524）の [`crate::state::Disclosure`] を埋め込んだ開閉状態機械
//! [`Collapsible`] を提供する。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`trigger`]/[`indicator`]/[`content`]、
//! 純粋関数で完結）を直接呼んで組み立てる。CSR/hydration は [`Collapsible`]
//! （[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"open"`/`"close"`/`"toggle"`）で状態遷移する。`fandhe-frontend-pre-styled-ui`
//! （#546〜）が本モジュールを呼んでスタイル済み Collapsible を組み立てる想定である。
//!
//! # 参考サイトとの意図的な差分（イシュー #1637）
//!
//! - **`content` の `data-collapsible` 存在属性は採用しない**: ark-ui は
//!   `data-collapsible` を付与するが、本クレートは [`crate::anatomy::Anatomy::part`]
//!   が常に `data-scope="collapsible" data-part="content"` を出力するため、
//!   同じ情報を重複して持たせるだけで状態値でもない。冗長な属性は追加しない。
//! - **サイズ計測・アニメーション系（`data-has-collapsed-size`・
//!   `--height`/`--width`/`--collapsed-height`/`--collapsed-width` 等の CSS 変数・
//!   `--radix-collapsible-content-*`）は headless-ui へ持ち込まない**: レイアウト
//!   計測・アニメーションの関心であり、`docs/policy/intentional-non-adoption.md`
//!   §3.25 の判断軸（規則 2）に従い、必要なら上層の `fandhe-frontend-pre-styled-ui`
//!   の責務とする（Themes 側イシュー #1670 の計画対象）。
//! - **`root` の `data-disabled` は維持する**: ark-ui の Root には無いが、
//!   Radix Primitives の Root には存在し、本クレートの既存契約（wasm-full の
//!   祖先 root disabled 判定）が依存しているため据え置く。
//! - **キーボード操作は Space / Enter のみ**: [`trigger`] はネイティブ
//!   `<button type="button">` として描画され、ブラウザ標準の Space/Enter →
//!   click 発火と `fandhe-frontend-wasm-full` の `MAPPING_TABLE`（`(collapsible,
//!   trigger)` クリック →`"toggle"` dispatch）の組み合わせで開閉が成立する。
//!   本モジュールは独自の keydown ハンドラを持たない（ark-ui/Radix いずれの
//!   Keyboard Support 表も Space/Enter のみを掲げており、追加のキー操作は無い）。
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
//! - 呼び出し側 `attrs` は各パーツの固定属性（`data-state`/`data-disabled`/
//!   `aria-expanded`/`aria-controls`/`type`/`disabled`/`hidden`/`id`）を
//!   [`drop_reserved`] で除外してから merge する。表示状態と実際の DOM 属性の
//!   なりすまし（A05）を防ぐ（`crate::progress`/`crate::switch`/
//!   `crate::toggle_group` と同型の防御、イシュー #1637）。
//! - hydration 属性（`data-hydrate-state`）はクライアント側で改ざんされうる
//!   入力として扱う。[`Collapsible`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は
//!   [`crate::state::Disclosure`] へ全委譲することで、panic せず
//!   `HydrateError` を返す既存保証をそのまま継承する。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_controls, aria_expanded};
use crate::data_attrs::{data_disabled, data_state};
use crate::state::{Disclosure, DisclosureAction, OpenState};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Collapsible の anatomy（`data-scope="collapsible"`）。
const ANATOMY: Anatomy = anatomy("collapsible");

/// Root パーツで固定出力する属性名（呼び出し側 `attrs` からの偽装を防ぐため
/// [`drop_reserved`] で除外する対象）。
const ROOT_RESERVED: &[&str] = &["data-state", "data-disabled"];

/// Root パーツ（`div`）。開閉状態・disabled 状態を `data-*` へ反映する。
#[must_use]
pub fn root<'a>(
    state: OpenState,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

const TRIGGER_RESERVED: &[&str] = &[
    "type",
    "aria-expanded",
    "aria-controls",
    "data-state",
    "data-disabled",
    "disabled",
];

/// Trigger パーツ（`button`）。
///
/// フォーム内配置時の意図しない submit を防ぐため `type="button"` を固定で
/// 付与する（A05 セキュリティ設定ミス対策）。`controls` が `Some` のとき
/// `aria-controls` で [`content`] と関連付ける。`disabled` はネイティブ
/// `disabled` 存在属性と `data-disabled` の両方へ反映する。
#[must_use]
pub fn trigger<'a>(
    state: OpenState,
    disabled: bool,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, TRIGGER_RESERVED);
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

const INDICATOR_RESERVED: &[&str] = &["data-state", "data-disabled"];

/// Indicator パーツ（`span`）。開閉状態を `data-state` へ、disabled 状態を
/// `data-disabled` へ反映する装飾用パーツ（アイコン等は呼び出し側の
/// `attrs`/`children` が担う）。`data-disabled` は ark-ui Indicator の
/// Data Attributes 表準拠で追加した（イシュー #1637。ネイティブ `disabled`
/// 存在属性は `span` に無効なため付与しない）。
#[must_use]
pub fn indicator<'a>(
    state: OpenState,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, INDICATOR_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("indicator", "span", merged, children)
}

const CONTENT_RESERVED: &[&str] = &["data-state", "data-disabled", "id", "hidden"];

/// Content パーツ（`div`）。
///
/// closed のとき `hidden` 存在属性を付与し、JS なしの SSR でも閉状態を表現
/// する（アニメーション対応の `open`/`visible` 分離・CSS 変数出力はスコープ外。
/// 本モジュールのモジュール doc「§参考サイトとの意図的な差分（イシュー
/// #1637）」参照）。`id` が `Some` のとき [`trigger`] の `controls` と対で
/// `aria-controls` 関連付けを成立させる。`data-disabled` は ark-ui/Radix の
/// Content 双方の Data Attributes 表準拠で追加した（イシュー #1637。
/// ネイティブ `disabled` 存在属性は `div` に無効なため付与しない）。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    disabled: bool,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, CONTENT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(data_disabled(disabled));
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("content", "div", merged, children)
}

/// [`Disclosure`]（#524）を埋め込んだ Collapsible の状態機械。
///
/// `data-state` と実際の開閉状態の整合を型レベルで保証する入口として、
/// 各パーツ関数（[`root`]/[`trigger`]/[`indicator`]/[`content`]）へ
/// `self.state()` を注入する利便メソッドを提供する。SSR での自由関数直接
/// 利用（本型を経由しない構成）も引き続き可能。`Default` は
/// [`OpenState::Closed`]（SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Collapsible {
    disclosure: Disclosure,
}

impl Collapsible {
    /// 指定した初期状態で Collapsible を生成する。
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
    pub fn root<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.state(), disabled, attrs, children)
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

    /// [`indicator`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn indicator<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        indicator(self.state(), disabled, attrs, children)
    }

    /// [`content`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn content<'a>(
        &self,
        disabled: bool,
        id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(self.state(), disabled, id, attrs, children)
    }
}

impl Component for Collapsible {
    type Action = DisclosureAction;

    fn update(&mut self, action: DisclosureAction) {
        self.disclosure.update(action);
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > trigger + content、children 空・id なし）。
    /// [`Disclosure::view`] と同じ位置付けであり、公開 UI としての利用は
    /// 想定しない（実際の UI 構築は §パーツ関数群を呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        let state = self.state();
        self.root(
            false,
            Vec::new(),
            vec![
                trigger(state, false, None, Vec::new(), Vec::new()),
                content(state, false, None, Vec::new(), Vec::new()),
            ],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<DisclosureAction> {
        Disclosure::decode_action(name, payload)
    }
}

impl Hydrate for Collapsible {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.disclosure.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            disclosure: Disclosure::from_hydration_attrs(attrs)?,
        })
    }
}

/// 呼び出し側 `attrs` から、各パーツが固定出力する属性名（大文字小文字を
/// 区別しない）を除外する（`crate::progress::drop_reserved` と同型の重複
/// 実装。モジュール間の相互依存を避けるため個別に定義する）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/data-state 出力 ---

    #[test]
    fn root_outputs_scope_part_and_state() {
        let html = render(&root(OpenState::Closed, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="collapsible""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn root_disabled_true_adds_data_disabled() {
        let html = render(&root(OpenState::Open, true, vec![], vec![]));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn trigger_has_type_button_and_aria_expanded() {
        let html = render(&trigger(OpenState::Closed, false, None, vec![], vec![]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(!html.contains("aria-controls"));
        assert!(!html.contains("disabled"));

        let html_open = render(&trigger(OpenState::Open, false, None, vec![], vec![]));
        assert!(html_open.contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn trigger_controls_some_outputs_aria_controls() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            Some("panel-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-controls="panel-1""#));
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
    fn indicator_outputs_scope_part_and_state_only() {
        let html = render(&indicator(OpenState::Open, false, vec![], vec![text("+")]));
        assert!(html.contains(r#"data-scope="collapsible""#));
        assert!(html.contains(r#"data-part="indicator""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(!html.contains("data-disabled"));
        assert!(html.contains('+'));
    }

    #[test]
    fn indicator_disabled_true_adds_data_disabled() {
        let html = render(&indicator(OpenState::Open, true, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn indicator_disabled_false_omits_data_disabled() {
        let html = render(&indicator(OpenState::Open, false, vec![], vec![]));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn content_closed_has_hidden_attr_open_does_not() {
        let closed = render(&content(OpenState::Closed, false, None, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&content(OpenState::Open, false, None, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_id_some_outputs_id_attribute() {
        let html = render(&content(
            OpenState::Open,
            false,
            Some("panel-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="panel-1""#));
    }

    #[test]
    fn content_disabled_true_adds_data_disabled_without_native_disabled() {
        let html = render(&content(OpenState::Open, true, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(!html.contains(r#" disabled"#));
    }

    #[test]
    fn content_does_not_emit_data_collapsible() {
        let html = render(&content(OpenState::Open, false, None, vec![], vec![]));
        assert!(!html.contains("data-collapsible"));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            OpenState::Closed,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="collapsible""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    /// 呼び出し側 `attrs` が `data-state`/`data-disabled` へなりすましても
    /// フレームワーク側の実値で上書きされる（表示状態と実際の DOM 属性の
    /// 不整合を防ぐ、イシュー #1637）。
    #[test]
    fn root_caller_reserved_attrs_are_dropped() {
        let html = render(&root(
            OpenState::Open,
            false,
            vec![("data-state", "closed"), ("data-disabled", "spoofed")],
            vec![],
        ));
        assert!(html.contains(r#"data-state="open""#));
        assert!(!html.contains("closed"));
        assert!(!html.contains("spoofed"));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn trigger_caller_reserved_attrs_are_dropped() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            Some("real"),
            vec![
                ("type", "submit"),
                ("aria-expanded", "true"),
                ("aria-controls", "spoofed"),
                ("data-state", "open"),
                ("data-disabled", "spoofed"),
                ("disabled", "spoofed"),
            ],
            vec![],
        ));
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains(r#"type="submit""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(html.contains(r#"aria-controls="real""#));
        assert!(!html.contains("spoofed"));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains(r#" disabled"#));
    }

    #[test]
    fn indicator_caller_reserved_attrs_are_dropped() {
        let html = render(&indicator(
            OpenState::Open,
            false,
            vec![("data-state", "closed"), ("data-disabled", "spoofed")],
            vec![],
        ));
        assert!(html.contains(r#"data-state="open""#));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("spoofed"));
    }

    #[test]
    fn content_caller_reserved_attrs_are_dropped() {
        let html = render(&content(
            OpenState::Open,
            false,
            Some("real"),
            vec![
                ("data-state", "closed"),
                ("data-disabled", "spoofed"),
                ("id", "attacker"),
                ("hidden", "spoofed"),
            ],
            vec![],
        ));
        assert!(html.contains(r#"data-state="open""#));
        assert!(!html.contains("data-disabled"));
        assert!(html.contains(r#"id="real""#));
        assert!(!html.contains("attacker"));
        assert!(!html.contains("hidden"));
        assert!(!html.contains("spoofed"));
    }

    // --- trigger + content の aria-controls/id 対応 ---

    #[test]
    fn trigger_controls_and_content_id_correspond() {
        let trigger_html = render(&trigger(OpenState::Open, false, Some("c1"), vec![], vec![]));
        let content_html = render(&content(OpenState::Open, false, Some("c1"), vec![], vec![]));
        assert!(trigger_html.contains(r#"aria-controls="c1""#));
        assert!(content_html.contains(r#"id="c1""#));
    }

    // --- Collapsible: dispatch 統合 ---

    #[test]
    fn collapsible_default_is_closed() {
        assert_eq!(Collapsible::default().state(), OpenState::Closed);
    }

    #[test]
    fn collapsible_dispatch_toggle_changes_data_state() {
        let mut c = Collapsible::default();
        assert!(render(&c.root(false, vec![], vec![])).contains(r#"data-state="closed""#));

        assert!(dispatch(&mut c, "toggle", ""));
        assert!(render(&c.root(false, vec![], vec![])).contains(r#"data-state="open""#));
        assert!(render(&c.content(false, None, vec![], vec![])).contains(r#"data-state="open""#));
        assert!(!render(&c.content(false, None, vec![], vec![])).contains("hidden"));
    }

    #[test]
    fn collapsible_dispatch_open_and_close() {
        let mut c = Collapsible::default();
        assert!(dispatch(&mut c, "open", ""));
        assert_eq!(c.state(), OpenState::Open);
        assert!(dispatch(&mut c, "close", ""));
        assert_eq!(c.state(), OpenState::Closed);
    }

    #[test]
    fn collapsible_dispatch_ignores_unknown_action() {
        let mut c = Collapsible::new(OpenState::Open);
        assert!(!dispatch(&mut c, "no_such_action", "x"));
        assert_eq!(c.state(), OpenState::Open);
    }

    // --- Collapsible: SSR 状態なし初期描画 ---

    #[test]
    fn collapsible_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Collapsible::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- Collapsible: hydration 経路 ---

    #[test]
    fn collapsible_hydration_round_trip() {
        let c = Collapsible::new(OpenState::Open);
        let rendered = render(&render_for_hydration(&c));
        assert!(rendered.contains(r#"data-hydrate-state="open""#));

        let restored = Collapsible::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored, c);
    }

    #[test]
    fn collapsible_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Collapsible::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-state".to_string())
        );
    }

    #[test]
    fn collapsible_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
            let err = Collapsible::from_hydration_attrs(&attrs).unwrap_err();
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
            false,
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
            false,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&indicator(
            OpenState::Open,
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn collapsible_xss_payload_in_hydration_state_is_rejected_not_rendered() {
        // data-hydrate-state はサーバーが state.as_data_state() から生成する
        // 固定語彙のみを出力するため攻撃者が任意値を注入する経路はないが、
        // クライアント改ざん入力の復元経路（from_hydration_attrs）が
        // 未知値を拒否することを Collapsible 経由でも固定する。
        let attrs = vec![(
            "data-hydrate-state".to_string(),
            "<script>alert(1)</script>".to_string(),
        )];
        let err = Collapsible::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

//! Popover（トリガー起点のオーバーレイ）headless コンポーネント（イシュー #532、親 #530）。
//!
//! ark-ui の Popover
//!（`.claude/skills/ark-ui/references/components/overlays/popover.md`）を
//! 参考に、Root / Trigger / Anchor / Positioner / Arrow / ArrowTip / Content /
//! Title / Description / CloseTrigger / Indicator の 11 anatomy パーツと、
//! Phase 1（#524）の [`crate::state::Disclosure`] を埋め込んだ開閉状態機械
//! [`Popover`] を提供する。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`trigger`]/[`anchor`]/
//! [`positioner`]/[`arrow`]/[`arrow_tip`]/[`content`]/[`title`]/[`description`]/
//! [`close_trigger`]/[`indicator`]、純粋関数で完結）を直接呼んで組み立てる。
//! CSR/hydration は [`Popover`]（[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"open"`/`"close"`/`"toggle"`）で状態遷移する。`fandhe-frontend-pre-styled-ui`
//! （#546〜）が本モジュールを呼んでスタイル済み Popover を組み立てる想定である。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`hidden`/`disabled`/`id`）はすべて
//!   `&'static str` リテラルで固定しており、動的値が属性名スロットへ混入する
//!   経路はない（[`crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の
//!   既存不変条件をそのまま継承する）。
//! - 動的値（`controls`/`id`/`labelledby`/`describedby`/呼び出し側 `attrs`/
//!   `children` テキスト）は [`fandhe_frontend_core::render`] の既定エスケープを
//!   必ず経由する。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は [`crate::state::OpenState`]
//!   に一元化し、本モジュールで独自の値を作らない。
//! - hydration 属性（`data-hydrate-state`）はクライアント側で改ざんされうる
//!   入力として扱う。[`Popover`] の [`fandhe_frontend_interactive::Hydrate`]
//!   実装は [`crate::state::Disclosure`] へ全委譲することで、panic せず
//!   `HydrateError` を返す既存保証をそのまま継承する。
//!
//! # スコープ外（ark-ui Popover のクライアントサイド機能）
//!
//! - フォーカストラップ / `autoFocus` / `closeOnEscape` /
//!   `closeOnInteractOutside` / portal / modal モード / `lazyMount`:
//!   クライアントランタイム側のイベント処理・DOM 操作であり、wasm 層の
//!   将来イシューのスコープ。
//!
//! 位置決めロジック（Floating UI 相当の placement / `sameWidth` / CSS 変数
//! 出力）は本イシュー（#532）時点ではスコープ外だったが、Tooltip（#533）との
//! 共通化検討を経てイシュー #590（親 #588）で [`crate::positioning`] として
//! 実装済みである。詳細は [`positioner`] の doc を参照。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{
    aria_controls, aria_describedby, aria_expanded, aria_haspopup, aria_labelledby, role, AriaPopup,
};
use crate::data_attrs::{data_disabled, data_state};
use crate::state::{Disclosure, DisclosureAction, OpenState};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Popover の anatomy（`data-scope="popover"`）。
const ANATOMY: Anatomy = anatomy("popover");

/// Root パーツ（`div`）。開閉状態を `data-*` へ反映する。
#[must_use]
pub fn root<'a>(state: OpenState, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Trigger パーツ（`button`）。
///
/// フォーム内配置時の意図しない submit を防ぐため `type="button"` を固定で
/// 付与する（A05 セキュリティ設定ミス対策、Collapsible の `trigger` と同判断）。
/// `aria-haspopup="dialog"` を固定付与し、`controls` が `Some` のとき
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
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_haspopup(AriaPopup::Dialog),
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

/// Anchor パーツ（`div`）。位置決めの代替参照要素。anatomy 属性のみを付与する
/// 最小主義な装飾用パーツ（位置決めロジック自体はスコープ外、モジュール doc
/// §スコープ外参照）。
#[must_use]
pub fn anchor<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("anchor", "div", attrs, children)
}

/// Positioner パーツ（`div`）。位置決めロジックのコンテナ。開閉状態を
/// `data-*` へ反映する。placement 計算自体は
/// [`crate::positioning::compute_position`]（#590）が担い、算出された
/// `style`（`--fandhe-*` CSS 変数）・`data-side`/`data-align` は呼び出し側が
/// `attrs` 経由で渡す（`fandhe-frontend-wasm-full` の `position` モジュールが
/// 実 DOM 計測を行ったうえで計算する。本関数自体は `web-sys` 非依存の
/// ままである）。
///
/// anatomy 上 [`arrow`]/[`arrow_tip`] は [`content`] と並んで本パーツ内に
/// 配置される想定であり、closed のとき `hidden` 存在属性を本パーツへ付与
/// することで、[`content`] だけでなく arrow 等のポインタ層も含めて
/// SSR/no-JS マークアップから隠す（[`Dialog`](crate::dialog::Dialog) の
/// `positioner` と同じ判断、イシュー #532 レビュー指摘）。
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

/// Arrow パーツ（`div`）。視覚的なポインター要素。anatomy 属性のみを付与する
/// 装飾用パーツ。
#[must_use]
pub fn arrow<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("arrow", "div", attrs, children)
}

/// ArrowTip パーツ（`div`）。`data-part="arrow-tip"`（ark-ui 準拠の kebab-case）。
#[must_use]
pub fn arrow_tip<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("arrow-tip", "div", attrs, children)
}

/// Content パーツ（`div`）。
///
/// `role="dialog"` を固定付与する。`labelledby`/`describedby` が `Some` の
/// とき [`title`]/[`description`] の `id` と対で関連付ける。closed のとき
/// `hidden` 存在属性を付与し、JS なしの SSR でも閉状態を表現する
/// （Collapsible の `content` と同じ判断）。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    id: Option<&'a str>,
    labelledby: Option<&'a str>,
    describedby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![role("dialog"), data_state(state.as_data_state())];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(labelledby) = labelledby {
        merged.push(aria_labelledby(labelledby));
    }
    if let Some(describedby) = describedby {
        merged.push(aria_describedby(describedby));
    }
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("content", "div", merged, children)
}

/// Title パーツ（`h2`）。`id` が `Some` のとき [`content`] の `labelledby` と
/// 対で `aria-labelledby` 関連付けを成立させる。
#[must_use]
pub fn title<'a>(id: Option<&'a str>, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("title", "h2", merged, children)
}

/// Description パーツ（`p`）。`id` が `Some` のとき [`content`] の
/// `describedby` と対で `aria-describedby` 関連付けを成立させる。
#[must_use]
pub fn description<'a>(
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("description", "p", merged, children)
}

/// CloseTrigger パーツ（`button`）。`data-part="close-trigger"`（ark-ui 準拠の
/// kebab-case）。[`trigger`] と同じくフォーム内配置時の意図しない submit を
/// 防ぐため `type="button"` を固定で付与する。アクセシブルネーム
/// （`aria-label` 等）は本関数の `attrs` を通じて呼び出し側が付与する責務と
/// する（本関数はテキスト内容に依存しないアイコンボタン等の用途も想定し、
/// 既定のアクセシブルネームを強制しない）。
#[must_use]
pub fn close_trigger<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    merged.extend(attrs);
    ANATOMY.part("close-trigger", "button", merged, children)
}

/// Indicator パーツ（`span`）。開閉状態のみを `data-state` へ反映する
/// 最小主義な装飾用パーツ（アイコン等は呼び出し側の `attrs`/`children` が担う。
/// Collapsible の `indicator` と同じ最小主義に揃える）。
#[must_use]
pub fn indicator<'a>(
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("indicator", "span", merged, children)
}

/// [`Disclosure`]（#524）を埋め込んだ Popover の状態機械。
///
/// `data-state` と実際の開閉状態の整合を型レベルで保証する入口として、
/// 状態を持つ各パーツ関数（[`root`]/[`trigger`]/[`positioner`]/[`content`]/
/// [`indicator`]）へ `self.state()` を注入する利便メソッドを提供する。
/// 状態を取らないパーツ（[`anchor`]/[`arrow`]/[`arrow_tip`]/[`title`]/
/// [`description`]/[`close_trigger`]）は自由関数のみを提供し、`Popover` の
/// メソッドとしては公開しない。SSR での自由関数直接利用（本型を経由しない
/// 構成）も引き続き可能。`Default` は [`OpenState::Closed`]（SSR の状態なし
/// 初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Popover {
    disclosure: Disclosure,
}

impl Popover {
    /// 指定した初期状態で Popover を生成する。
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
        labelledby: Option<&'a str>,
        describedby: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(self.state(), id, labelledby, describedby, attrs, children)
    }

    /// [`indicator`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn indicator<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        indicator(self.state(), attrs, children)
    }
}

impl Component for Popover {
    type Action = DisclosureAction;

    fn update(&mut self, action: DisclosureAction) {
        self.disclosure.update(action);
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > trigger + positioner > content、children 空・id なし）。
    /// [`Disclosure::view`]・Collapsible の `view` と同じ位置付けであり、
    /// 公開 UI としての利用は想定しない（実際の UI 構築は §パーツ関数群を
    /// 呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        let state = self.state();
        self.root(
            Vec::new(),
            vec![
                trigger(state, false, None, Vec::new(), Vec::new()),
                positioner(
                    state,
                    Vec::new(),
                    vec![content(state, None, None, None, Vec::new(), Vec::new())],
                ),
            ],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<DisclosureAction> {
        Disclosure::decode_action(name, payload)
    }
}

impl Hydrate for Popover {
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

    // --- 各パーツの data-scope/data-part 出力 ---

    #[test]
    fn root_outputs_scope_part_and_state() {
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="popover""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn trigger_has_type_button_haspopup_dialog_and_aria_expanded() {
        let html = render(&trigger(OpenState::Closed, false, None, vec![], vec![]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-haspopup="dialog""#));
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
            Some("popover-content-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-controls="popover-content-1""#));
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
    fn anchor_outputs_scope_and_part_only() {
        let html = render(&anchor(vec![], vec![]));
        assert!(html.contains(r#"data-scope="popover""#));
        assert!(html.contains(r#"data-part="anchor""#));
    }

    #[test]
    fn positioner_outputs_scope_part_and_state() {
        let html = render(&positioner(OpenState::Open, vec![], vec![]));
        assert!(html.contains(r#"data-part="positioner""#));
        assert!(html.contains(r#"data-state="open""#));
    }

    #[test]
    fn positioner_closed_has_hidden_attr_open_does_not() {
        // anatomy 上 arrow/arrow_tip は content と並んで positioner 内に
        // あるため、positioner 自体を hidden にしないと closed でも
        // ポインタ層（arrow）が SSR/no-JS マークアップに表示され続ける
        // （イシュー #532 レビュー指摘、Bugbot f6f5796c-8365-4534-8e07-38cc499b2449）。
        let closed = render(&positioner(OpenState::Closed, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&positioner(OpenState::Open, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    // --- positioning（#590）接続: positioner/arrow が attrs 経由で
    // style/data-side/data-align を透過し、既定エスケープを経由することを
    // 確認する ---

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
            same_width: false,
        };
        let resolved = compute_position(anchor, floating, viewport, &config, true);
        let style = css_vars_style(&resolved, anchor.width, config.same_width);
        let mut attrs: Vec<(&str, &str)> = vec![("style", &style)];
        attrs.extend(placement_attrs(resolved.placement));

        let html = render(&positioner(OpenState::Open, attrs, vec![]));
        assert!(html.contains("--fandhe-x:"));
        // same_width: false のため --fandhe-reference-width は出力されない
        // （イシュー #622 レビュー指摘の回帰）。
        assert!(!html.contains("--fandhe-reference-width"));
        assert!(html.contains(r#"data-side="bottom""#));
        assert!(html.contains(r#"data-align="center""#));
    }

    #[test]
    fn arrow_outputs_scope_and_part_only() {
        let html = render(&arrow(vec![], vec![]));
        assert!(html.contains(r#"data-part="arrow""#));
    }

    #[test]
    fn arrow_tip_outputs_kebab_case_part() {
        let html = render(&arrow_tip(vec![], vec![]));
        assert!(html.contains(r#"data-part="arrow-tip""#));
    }

    #[test]
    fn content_has_role_dialog_and_state() {
        let html = render(&content(OpenState::Open, None, None, None, vec![], vec![]));
        assert!(html.contains(r#"role="dialog""#));
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
    fn content_id_labelledby_describedby_some_outputs_all() {
        let html = render(&content(
            OpenState::Open,
            Some("content-1"),
            Some("title-1"),
            Some("desc-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="content-1""#));
        assert!(html.contains(r#"aria-labelledby="title-1""#));
        assert!(html.contains(r#"aria-describedby="desc-1""#));
    }

    #[test]
    fn title_and_description_id_some_outputs_id() {
        let title_html = render(&title(Some("title-1"), vec![], vec![text("hi")]));
        assert!(title_html.contains(r#"<h2"#));
        assert!(title_html.contains(r#"id="title-1""#));

        let desc_html = render(&description(Some("desc-1"), vec![], vec![text("hi")]));
        assert!(desc_html.contains(r#"<p"#));
        assert!(desc_html.contains(r#"id="desc-1""#));
    }

    #[test]
    fn close_trigger_has_type_button_and_kebab_case_part() {
        let html = render(&close_trigger(vec![], vec![]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-part="close-trigger""#));
    }

    #[test]
    fn indicator_outputs_scope_part_and_state_only() {
        let html = render(&indicator(OpenState::Open, vec![], vec![text("+")]));
        assert!(html.contains(r#"data-scope="popover""#));
        assert!(html.contains(r#"data-part="indicator""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains('+'));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            OpenState::Closed,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="popover""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- trigger + content の aria-controls/id 対応、title/description との対応 ---

    #[test]
    fn trigger_controls_and_content_id_correspond() {
        let trigger_html = render(&trigger(OpenState::Open, false, Some("c1"), vec![], vec![]));
        let content_html = render(&content(
            OpenState::Open,
            Some("c1"),
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(trigger_html.contains(r#"aria-controls="c1""#));
        assert!(content_html.contains(r#"id="c1""#));
    }

    #[test]
    fn content_labelledby_describedby_and_title_description_id_correspond() {
        let content_html = render(&content(
            OpenState::Open,
            None,
            Some("t1"),
            Some("d1"),
            vec![],
            vec![],
        ));
        let title_html = render(&title(Some("t1"), vec![], vec![]));
        let desc_html = render(&description(Some("d1"), vec![], vec![]));
        assert!(content_html.contains(r#"aria-labelledby="t1""#));
        assert!(content_html.contains(r#"aria-describedby="d1""#));
        assert!(title_html.contains(r#"id="t1""#));
        assert!(desc_html.contains(r#"id="d1""#));
    }

    // --- Popover: dispatch 統合 ---

    #[test]
    fn popover_default_is_closed() {
        assert_eq!(Popover::default().state(), OpenState::Closed);
    }

    #[test]
    fn popover_dispatch_toggle_changes_data_state() {
        let mut p = Popover::default();
        assert!(render(&p.root(vec![], vec![])).contains(r#"data-state="closed""#));

        assert!(dispatch(&mut p, "toggle", ""));
        assert!(render(&p.root(vec![], vec![])).contains(r#"data-state="open""#));
        assert!(render(&p.trigger(false, None, vec![], vec![])).contains(r#"aria-expanded="true""#));
        assert!(render(&p.positioner(vec![], vec![])).contains(r#"data-state="open""#));
        let content_html = render(&p.content(None, None, None, vec![], vec![]));
        assert!(content_html.contains(r#"data-state="open""#));
        assert!(!content_html.contains("hidden"));
        assert!(render(&p.indicator(vec![], vec![])).contains(r#"data-state="open""#));
    }

    #[test]
    fn popover_dispatch_open_and_close() {
        let mut p = Popover::default();
        assert!(dispatch(&mut p, "open", ""));
        assert_eq!(p.state(), OpenState::Open);
        assert!(dispatch(&mut p, "close", ""));
        assert_eq!(p.state(), OpenState::Closed);
    }

    #[test]
    fn popover_dispatch_ignores_unknown_action() {
        let mut p = Popover::new(OpenState::Open);
        assert!(!dispatch(&mut p, "no_such_action", "x"));
        assert_eq!(p.state(), OpenState::Open);
    }

    // --- Popover: SSR 状態なし初期描画 ---

    #[test]
    fn popover_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Popover::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- Popover: hydration 経路 ---

    #[test]
    fn popover_hydration_round_trip() {
        let p = Popover::new(OpenState::Open);
        let rendered = render(&render_for_hydration(&p));
        assert!(rendered.contains(r#"data-hydrate-state="open""#));

        let restored = Popover::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored, p);
    }

    #[test]
    fn popover_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Popover::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-state".to_string())
        );
    }

    #[test]
    fn popover_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
            let err = Popover::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: controls/id/labelledby/describedby/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

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
    fn content_id_labelledby_describedby_payload_is_escaped_on_render() {
        let html = render(&content(
            OpenState::Open,
            Some(ATTR_BREAK_PAYLOAD),
            Some(ATTR_BREAK_PAYLOAD),
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
        let html = render(&indicator(
            OpenState::Open,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn popover_xss_payload_in_hydration_state_is_rejected_not_rendered() {
        // data-hydrate-state はサーバーが state.as_data_state() から生成する
        // 固定語彙のみを出力するため攻撃者が任意値を注入する経路はないが、
        // クライアント改ざん入力の復元経路（from_hydration_attrs）が
        // 未知値を拒否することを Popover 経由でも固定する。
        let attrs = vec![(
            "data-hydrate-state".to_string(),
            "<script>alert(1)</script>".to_string(),
        )];
        let err = Popover::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

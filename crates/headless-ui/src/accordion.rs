//! Accordion（開閉可能な項目リスト）headless コンポーネント（イシュー #527、親 #526）。
//!
//! ark-ui の Accordion
//!（`.claude/skills/ark-ui/references/components/disclosure/accordion.md`）を
//! 参考に、Root / Item / ItemTrigger / ItemIndicator / ItemContent の 5
//! anatomy パーツと、Phase 1（#524）の [`crate::state::SingleSelect`] を
//! 埋め込んだ「高々 1 項目が開く」状態機械 [`Accordion`]、および
//! [`crate::state::MultiSelect`]（イシュー #594）を埋め込んだ「複数項目が
//! 同時に開く」状態機械 [`MultiAccordion`] を提供する。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`item`]/[`item_trigger`]/
//! [`item_indicator`]/[`item_content`]、いずれも純粋関数で完結）を直接呼んで
//! 組み立てる。各パーツは項目ごとの [`crate::state::OpenState`] を引数で
//! 受け取るため single/multiple のどちらのモードでも共用できる。
//!
//! CSR/hydration は用途に応じて [`Accordion`] または [`MultiAccordion`]
//! （いずれも [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を使い分ける。
//! [`Accordion`] は「高々 1 項目が開く」single モード
//! （dispatch: `"select"`/`"deselect"`/`"toggle"`、`"deselect"` は payload
//! なしで全解除）を、[`MultiAccordion`] は「複数項目が同時に開く」
//! multiple モード（dispatch: `"select"`/`"deselect"`/`"toggle"`、
//! `"deselect"` は項目値 payload 必須で当該項目のみ解除）を提供する。
//! `fandhe-frontend-pre-styled-ui`（#546〜）が本モジュールを呼んで
//! スタイル済み Accordion を組み立てる想定である。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`type`/`role`/`hidden`/`disabled`/`id`）は
//!   すべて `&'static str` リテラルで固定しており、動的値が属性名スロットへ
//!   混入する経路はない（[`mod@crate::anatomy`]/[`crate::aria`]/
//!   [`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（`value`/`id`/`controls`/`labelled_by`/呼び出し側 `attrs`/
//!   `children` テキスト）は [`fandhe_frontend_core::render`] の既定
//!   エスケープを必ず経由する。`raw_html()` は使用せず、HTML 文字列を
//!   直接組み立てない（`id` の `format!` 利用は Tabs（#528）と同じく属性値
//!   という**データ**の組み立てであり、マークアップ自体の文字列化ではない）。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は [`crate::state::OpenState`]
//!   に一元化し、本モジュールで独自の値を作らない。
//! - hydration 属性（`data-hydrate-selected`）はクライアント側で改ざんされ
//!   うる入力として扱う。[`Accordion`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は
//!   [`crate::state::SingleSelect`] へ全委譲することで、panic せず
//!   `HydrateError` を返す既存保証をそのまま継承する。
//!
//! # out-of-scope（本イシュー #527/#594 のスコープ外）
//!
//! - **全項目一括 close（`MultiSelect` の payload なし deselect 相当）**:
//!   [`crate::state::MultiSelectAction::Deselect`] は項目単位（payload
//!   必須）のみを提供する。「どれを閉じるか」の指定なしに全解除する
//!   アクションはイシュー #594 の dispatch 契約に含まれないため未実装。
//! - **orientation / キーボードナビゲーション**: SSR 静的マークアップに
//!   寄与しない CSR 挙動層の責務のため未提供（Tabs の `data-orientation`
//!   と異なり、Accordion の orientation は本イシューのスコープ外のまま）。
//!   `data-orientation` が必要な呼び出し側は各パーツの `attrs` 引数で
//!   付与できる（既定エスケープ経由のまま、迂回経路ではない）。
//! - **lazyMount / unmountOnExit / CSS 変数（`--height` 等）**: アニメーション
//!   対応はスコープ外（[`item_content`] は `hidden` 存在属性のみで closed を
//!   表現する）。
//! - **heading 要素でのラップ**: `<h3>` 等での [`item_trigger`] のラップは
//!   呼び出し側が `children` で自由に表現できるため、本モジュールは専用
//!   パーツを持たない。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_controls, aria_expanded, aria_labelledby, role};
use crate::data_attrs::{data_disabled, data_state};
use crate::state::{MultiSelect, MultiSelectAction, OpenState, SingleSelect, SingleSelectAction};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Accordion の anatomy（`data-scope="accordion"`）。
const ANATOMY: Anatomy = anatomy("accordion");

/// Root パーツ（`div`）。状態非依存（項目の開閉状態は各 [`item`] 側が持つ）。
#[must_use]
pub fn root<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("root", "div", attrs, children)
}

/// Item パーツ（`div`）。項目 1 個の開閉状態・disabled 状態を `data-*` へ反映する。
#[must_use]
pub fn item<'a>(
    state: OpenState,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item", "div", merged, children)
}

/// ItemTrigger パーツ（`button`）。
///
/// フォーム内配置時の意図しない submit を防ぐため `type="button"` を固定で
/// 付与する（A05 セキュリティ設定ミス対策。Collapsible 実装（イシュー #529）
/// と同じ判断を踏襲する）。`controls` が `Some` のとき
/// `aria-controls` で [`item_content`] と関連付ける。`disabled` はネイティブ
/// `disabled` 存在属性と `data-disabled` の両方へ反映する。
#[must_use]
pub fn item_trigger<'a>(
    state: OpenState,
    disabled: bool,
    id: Option<&'a str>,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_expanded(state.is_open()),
        data_state(state.as_data_state()),
    ];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(controls) = controls {
        merged.push(aria_controls(controls));
    }
    merged.extend(data_disabled(disabled));
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("item-trigger", "button", merged, children)
}

/// ItemIndicator パーツ（`span`）。開閉状態のみを `data-state` へ反映する
/// 最小主義な装飾用パーツ（アイコン等は呼び出し側の `attrs`/`children` が
/// 担う。Collapsible の `indicator` と同じ最小主義に揃える）。
#[must_use]
pub fn item_indicator<'a>(
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("item-indicator", "span", merged, children)
}

/// ItemContent パーツ（`div`）。
///
/// closed のとき `hidden` 存在属性を付与し、JS なしの SSR でも閉状態を表現
/// する（アニメーション対応の CSS 変数出力等はスコープ外、モジュール doc
/// §out-of-scope 参照）。`id` が `Some` のとき [`item_trigger`] の `controls`
/// と対で `aria-controls` 関連付けを成立させる。`labelled_by` が `Some` の
/// ときのみ `role="region"` と `aria-labelledby` をセットで付与する
/// （名前なし region を作らないため、`labelled_by` が `None` の場合は
/// どちらも出力しない）。
#[must_use]
pub fn item_content<'a>(
    state: OpenState,
    id: Option<&'a str>,
    labelled_by: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(labelled_by) = labelled_by {
        merged.push(role("region"));
        merged.push(aria_labelledby(labelled_by));
    }
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("item-content", "div", merged, children)
}

/// [`SingleSelect`]（#524）を埋め込んだ Accordion（single モード）の状態機械。
///
/// 「高々 1 項目が開く」制約を型レベルで保証する入口として、[`Self::item_state`]
/// が各項目値の [`OpenState`] を決定し、各パーツ関数（[`item`]/
/// [`item_trigger`]/[`item_indicator`]/[`item_content`]）へ注入する利便
/// メソッドを提供する（[`root`] は状態非依存のため利便メソッドを持たない）。
/// SSR での自由関数直接利用（本型を経由しない構成。複数項目同時 open の
/// 表現を含む）も引き続き可能。`Default` は未選択（全項目 closed。SSR の
/// 状態なし初期描画に対応する既定値）。
///
/// collapsible な挙動（開いた項目を再クリックで閉じる）が必要な呼び出し側は
/// dispatch アクション名 `"toggle"`（[`SingleSelectAction::Toggle`]）を、
/// 常に何か 1 項目を開いたままにしたい場合は `"select"`
/// （[`SingleSelectAction::Select`]）を使い分ける。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Accordion {
    select: SingleSelect,
}

impl Accordion {
    /// 現在開いている項目値（未選択なら `None`）。
    #[must_use]
    pub fn expanded(&self) -> Option<&str> {
        self.select.selected()
    }

    /// 指定した項目値が開いているかどうか。
    #[must_use]
    pub fn is_open(&self, value: &str) -> bool {
        self.select.is_selected(value)
    }

    /// 項目 `value` の現在の [`OpenState`]。
    #[must_use]
    pub fn item_state(&self, value: &str) -> OpenState {
        if self.is_open(value) {
            OpenState::Open
        } else {
            OpenState::Closed
        }
    }

    /// [`item`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        value: &str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(self.item_state(value), disabled, attrs, children)
    }

    /// [`item_trigger`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item_trigger<'a>(
        &self,
        value: &str,
        disabled: bool,
        id: Option<&'a str>,
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_trigger(
            self.item_state(value),
            disabled,
            id,
            controls,
            attrs,
            children,
        )
    }

    /// [`item_indicator`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item_indicator<'a>(
        &self,
        value: &str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_indicator(self.item_state(value), attrs, children)
    }

    /// [`item_content`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item_content<'a>(
        &self,
        value: &str,
        id: Option<&'a str>,
        labelled_by: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_content(self.item_state(value), id, labelled_by, attrs, children)
    }
}

impl Component for Accordion {
    type Action = SingleSelectAction;

    fn update(&mut self, action: SingleSelectAction) {
        self.select.update(action);
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root、children
    /// 空）。[`SingleSelect::view`] と同じ位置付けであり、公開 UI としての
    /// 利用は想定しない（実際の UI 構築は §パーツ関数群を呼び出し側が
    /// 組み合わせる）。
    fn view(&self) -> Node {
        root(Vec::new(), Vec::new())
    }

    fn decode_action(name: &str, payload: &str) -> Option<SingleSelectAction> {
        SingleSelect::decode_action(name, payload)
    }
}

impl Hydrate for Accordion {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.select.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            select: SingleSelect::from_hydration_attrs(attrs)?,
        })
    }
}

/// [`MultiSelect`]（イシュー #594）を埋め込んだ Accordion（multiple モード）
/// の状態機械。
///
/// [`Accordion`]（single モード）と対称の API を提供する。「複数項目が同時に
/// 開く」ことを許すため [`Self::expanded`] は `&[String]` を返す。
/// [`Component::Action`] は関連型が 1 つのため、single/multiple 双方を 1 型で
/// 扱おうとすると dispatch 契約（`"deselect"` の payload 有無）が衝突する。
/// 型を分けることで hydration の解釈（2 件以上のリストを拒否/受理のどちらで
/// 扱うか）も静的に確定し、fail-closed 性を保つ（詳細は
/// `docs/design`（該当があれば）または本イシューの実装計画を参照）。
/// `Default` は全項目 closed（SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MultiAccordion {
    select: MultiSelect,
}

impl MultiAccordion {
    /// 現在開いている項目値（選択順）。
    #[must_use]
    pub fn expanded(&self) -> &[String] {
        self.select.selected()
    }

    /// 指定した項目値が開いているかどうか。
    #[must_use]
    pub fn is_open(&self, value: &str) -> bool {
        self.select.is_selected(value)
    }

    /// 項目 `value` の現在の [`OpenState`]。
    #[must_use]
    pub fn item_state(&self, value: &str) -> OpenState {
        if self.is_open(value) {
            OpenState::Open
        } else {
            OpenState::Closed
        }
    }

    /// [`item`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        value: &str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(self.item_state(value), disabled, attrs, children)
    }

    /// [`item_trigger`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item_trigger<'a>(
        &self,
        value: &str,
        disabled: bool,
        id: Option<&'a str>,
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_trigger(
            self.item_state(value),
            disabled,
            id,
            controls,
            attrs,
            children,
        )
    }

    /// [`item_indicator`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item_indicator<'a>(
        &self,
        value: &str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_indicator(self.item_state(value), attrs, children)
    }

    /// [`item_content`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item_content<'a>(
        &self,
        value: &str,
        id: Option<&'a str>,
        labelled_by: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_content(self.item_state(value), id, labelled_by, attrs, children)
    }
}

impl Component for MultiAccordion {
    type Action = MultiSelectAction;

    fn update(&mut self, action: MultiSelectAction) {
        self.select.update(action);
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root、children
    /// 空）。[`MultiSelect::view`] と同じ位置付け。
    fn view(&self) -> Node {
        root(Vec::new(), Vec::new())
    }

    fn decode_action(name: &str, payload: &str) -> Option<MultiSelectAction> {
        MultiSelect::decode_action(name, payload)
    }
}

impl Hydrate for MultiAccordion {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.select.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            select: MultiSelect::from_hydration_attrs(attrs)?,
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
    fn root_outputs_scope_and_part_only() {
        let html = render(&root(vec![], vec![]));
        assert_eq!(
            html,
            r#"<div data-scope="accordion" data-part="root"></div>"#
        );
    }

    #[test]
    fn item_outputs_scope_part_and_state() {
        let html = render(&item(OpenState::Closed, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="accordion""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn item_disabled_true_adds_data_disabled() {
        let html = render(&item(OpenState::Open, true, vec![], vec![]));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_trigger_has_type_button_and_aria_expanded() {
        let html = render(&item_trigger(
            OpenState::Closed,
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains("<button"));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(!html.contains("aria-controls"));
        assert!(!html.contains(" id="));
        assert!(!html.contains("disabled"));

        let html_open = render(&item_trigger(
            OpenState::Open,
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html_open.contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn item_trigger_id_and_controls_some_outputs_both_attributes() {
        let html = render(&item_trigger(
            OpenState::Closed,
            false,
            Some("t-trigger-a"),
            Some("t-content-a"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="t-trigger-a""#));
        assert!(html.contains(r#"aria-controls="t-content-a""#));
    }

    #[test]
    fn item_trigger_disabled_true_adds_native_and_data_disabled() {
        let html = render(&item_trigger(
            OpenState::Closed,
            true,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"disabled="""#));
    }

    #[test]
    fn item_trigger_disabled_false_omits_both_disabled_attrs() {
        let html = render(&item_trigger(
            OpenState::Closed,
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains(" disabled"));
    }

    #[test]
    fn item_indicator_outputs_scope_part_and_state_only() {
        let html = render(&item_indicator(OpenState::Open, vec![], vec![text("+")]));
        assert!(html.contains(r#"data-scope="accordion""#));
        assert!(html.contains(r#"data-part="item-indicator""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains('+'));
    }

    #[test]
    fn item_content_closed_has_hidden_attr_open_does_not() {
        let closed = render(&item_content(OpenState::Closed, None, None, vec![], vec![]));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&item_content(OpenState::Open, None, None, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn item_content_id_some_outputs_id_attribute() {
        let html = render(&item_content(
            OpenState::Open,
            Some("t-content-a"),
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"id="t-content-a""#));
    }

    #[test]
    fn item_content_labelled_by_some_outputs_role_region_and_aria_labelledby_together() {
        let html = render(&item_content(
            OpenState::Open,
            None,
            Some("t-trigger-a"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="region""#));
        assert!(html.contains(r#"aria-labelledby="t-trigger-a""#));
    }

    #[test]
    fn item_content_labelled_by_none_omits_role_and_aria_labelledby() {
        let html = render(&item_content(OpenState::Open, None, None, vec![], vec![]));
        assert!(!html.contains("role="));
        assert!(!html.contains("aria-labelledby"));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&item(
            OpenState::Closed,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="accordion""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(!html.contains("attacker"));
    }

    // --- root > item > trigger(+indicator) + content の組み立てテスト（相互参照込みスナップショット） ---

    #[test]
    fn full_assembly_trigger_and_content_id_cross_reference() {
        let node = root(
            vec![],
            vec![item(
                OpenState::Open,
                false,
                vec![],
                vec![
                    item_trigger(
                        OpenState::Open,
                        false,
                        Some("t-trigger-a"),
                        Some("t-content-a"),
                        vec![],
                        vec![item_indicator(OpenState::Open, vec![], vec![text("+")])],
                    ),
                    item_content(
                        OpenState::Open,
                        Some("t-content-a"),
                        Some("t-trigger-a"),
                        vec![],
                        vec![text("panel A")],
                    ),
                ],
            )],
        );
        assert_eq!(
            render(&node),
            concat!(
                r#"<div data-scope="accordion" data-part="root">"#,
                r#"<div data-scope="accordion" data-part="item" data-state="open">"#,
                r#"<button data-scope="accordion" data-part="item-trigger" type="button" aria-expanded="true" data-state="open" id="t-trigger-a" aria-controls="t-content-a">"#,
                r#"<span data-scope="accordion" data-part="item-indicator" data-state="open">+</span>"#,
                r#"</button>"#,
                r#"<div data-scope="accordion" data-part="item-content" data-state="open" id="t-content-a" role="region" aria-labelledby="t-trigger-a">panel A</div>"#,
                r#"</div>"#,
                r#"</div>"#,
            )
        );
    }

    // --- Accordion: dispatch 統合（single モード） ---

    #[test]
    fn accordion_default_is_all_closed() {
        let a = Accordion::default();
        assert_eq!(a.expanded(), None);
        assert!(!a.is_open("a"));
        assert!(!a.is_open("b"));
    }

    #[test]
    fn accordion_dispatch_select_opens_at_most_one_item() {
        let mut a = Accordion::default();
        assert!(dispatch(&mut a, "select", "a"));
        assert!(a.is_open("a"));
        assert!(!a.is_open("b"));

        assert!(dispatch(&mut a, "select", "b"));
        assert!(!a.is_open("a"));
        assert!(a.is_open("b"));
    }

    #[test]
    fn accordion_dispatch_toggle_opens_then_closes_collapsible_style() {
        let mut a = Accordion::default();
        assert!(dispatch(&mut a, "toggle", "a"));
        assert!(a.is_open("a"));

        assert!(dispatch(&mut a, "toggle", "a"));
        assert!(!a.is_open("a"));
        assert_eq!(a.expanded(), None);
    }

    #[test]
    fn accordion_dispatch_deselect_closes_all() {
        let mut a = Accordion::default();
        dispatch(&mut a, "select", "a");
        assert!(dispatch(&mut a, "deselect", ""));
        assert_eq!(a.expanded(), None);
    }

    #[test]
    fn accordion_dispatch_ignores_unknown_action() {
        let mut a = Accordion::default();
        dispatch(&mut a, "select", "a");
        assert!(!dispatch(&mut a, "no_such_action", "b"));
        assert!(a.is_open("a"));
    }

    // --- Accordion: 利便メソッド経由の描画が状態機械と一致 ---

    #[test]
    fn accordion_convenience_methods_reflect_state() {
        let mut a = Accordion::default();
        dispatch(&mut a, "select", "a");

        let trigger_a = render(&a.item_trigger("a", false, None, None, vec![], vec![]));
        assert!(trigger_a.contains(r#"aria-expanded="true""#));
        assert!(trigger_a.contains(r#"data-state="open""#));

        let trigger_b = render(&a.item_trigger("b", false, None, None, vec![], vec![]));
        assert!(trigger_b.contains(r#"aria-expanded="false""#));
        assert!(trigger_b.contains(r#"data-state="closed""#));

        let content_a = render(&a.item_content("a", None, None, vec![], vec![]));
        assert!(!content_a.contains("hidden"));

        let content_b = render(&a.item_content("b", None, None, vec![], vec![]));
        assert!(content_b.contains(r#"hidden="""#));
    }

    // --- Accordion: SSR 状態なし初期描画 ---

    #[test]
    fn accordion_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Accordion::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- Accordion: hydration 経路 ---

    #[test]
    fn accordion_hydration_round_trip_selected() {
        let mut a = Accordion::default();
        dispatch(&mut a, "select", "tab-1");
        let rendered = render(&render_for_hydration(&a));
        // codec::encode_list は区切り文字を先頭に付与するエンコードのため、
        // 属性値は選択値そのままの文字列（"tab-1"）とは一致しない。属性が
        // 実際に出力され値に選択値が含まれることのみを確認する
        // （エンコード形式の詳細は `crate::state::SingleSelect` の責務）。
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("tab-1"));

        let restored = Accordion::from_hydration_attrs(&a.hydration_attrs()).unwrap();
        assert_eq!(restored, a);
    }

    #[test]
    fn accordion_hydration_round_trip_unselected() {
        let a = Accordion::default();
        let restored = Accordion::from_hydration_attrs(&a.hydration_attrs()).unwrap();
        assert_eq!(restored, a);
    }

    #[test]
    fn accordion_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Accordion::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-selected".to_string())
        );
    }

    #[test]
    fn accordion_from_hydration_attrs_invalid_value_does_not_panic() {
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["a".to_string(), "b".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = Accordion::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰: value/id/controls/labelled_by/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn item_trigger_id_and_controls_payload_is_escaped_on_render() {
        let html = render(&item_trigger(
            OpenState::Closed,
            false,
            Some(ATTR_BREAK_PAYLOAD),
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_content_id_and_labelled_by_payload_is_escaped_on_render() {
        let html = render(&item_content(
            OpenState::Open,
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
        let html = render(&root(vec![("data-testid", ATTR_BREAK_PAYLOAD)], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&item_indicator(
            OpenState::Open,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn accordion_dispatch_select_payload_is_escaped_on_render() {
        let mut a = Accordion::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut a, "select", payload));

        let rendered = render(&render_for_hydration(&a));
        // 正の確認: data-hydrate-selected 属性が実際に出力へ載っていること
        // （不在アサーションのみだと属性ごと消えた場合にも誤って合格しうる）。
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn accordion_xss_payload_in_hydration_selected_is_rejected_not_rendered() {
        // 改ざん耐性: from_hydration_attrs は未知/不正な値を panic せず拒否する
        // （SingleSelect の既存保証を Accordion 経由でも固定する）。
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["<script>alert(1)</script>".to_string(), "b".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = Accordion::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn accordion_view_root_is_element_for_render_for_hydration() {
        // render_for_hydration はルートが Node::Element であることを前提に
        // 属性を合成する（`crates/interactive/src/lib.rs` 参照）。本型の
        // view() が常に Element を返すことを固定する回帰テスト。
        let node = Accordion::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    // --- MultiAccordion: dispatch 統合（multiple モード） ---

    #[test]
    fn multi_accordion_default_is_all_closed() {
        let a = MultiAccordion::default();
        assert_eq!(a.expanded(), &[] as &[String]);
        assert!(!a.is_open("a"));
        assert!(!a.is_open("b"));
    }

    #[test]
    fn multi_accordion_dispatch_select_opens_multiple_items_simultaneously() {
        let mut a = MultiAccordion::default();
        assert!(dispatch(&mut a, "select", "a"));
        assert!(a.is_open("a"));
        assert!(!a.is_open("b"));

        // 複数項目同時 open が本型の存在理由（Accordion は select 2 回目で
        // 前項目が閉じるが、MultiAccordion は両方 open のまま維持する）。
        assert!(dispatch(&mut a, "select", "b"));
        assert!(a.is_open("a"));
        assert!(a.is_open("b"));
    }

    #[test]
    fn multi_accordion_dispatch_toggle_and_deselect_close_only_target_item() {
        let mut a = MultiAccordion::default();
        dispatch(&mut a, "select", "a");
        dispatch(&mut a, "select", "b");

        assert!(dispatch(&mut a, "deselect", "a"));
        assert!(!a.is_open("a"));
        assert!(a.is_open("b"));

        assert!(dispatch(&mut a, "toggle", "b"));
        assert!(!a.is_open("b"));
        assert_eq!(a.expanded(), &[] as &[String]);
    }

    #[test]
    fn multi_accordion_dispatch_ignores_unknown_action() {
        let mut a = MultiAccordion::default();
        dispatch(&mut a, "select", "a");
        assert!(!dispatch(&mut a, "no_such_action", "b"));
        assert!(a.is_open("a"));
    }

    // --- MultiAccordion: 利便メソッド経由の描画が状態機械と一致（複数同時 open） ---

    #[test]
    fn multi_accordion_convenience_methods_reflect_state_for_two_open_items() {
        let mut a = MultiAccordion::default();
        dispatch(&mut a, "select", "a");
        dispatch(&mut a, "select", "b");

        let trigger_a = render(&a.item_trigger("a", false, None, None, vec![], vec![]));
        assert!(trigger_a.contains(r#"aria-expanded="true""#));
        let trigger_b = render(&a.item_trigger("b", false, None, None, vec![], vec![]));
        assert!(trigger_b.contains(r#"aria-expanded="true""#));
        let trigger_c = render(&a.item_trigger("c", false, None, None, vec![], vec![]));
        assert!(trigger_c.contains(r#"aria-expanded="false""#));

        let content_a = render(&a.item_content("a", None, None, vec![], vec![]));
        assert!(!content_a.contains("hidden"));
        let content_b = render(&a.item_content("b", None, None, vec![], vec![]));
        assert!(!content_b.contains("hidden"));
        let content_c = render(&a.item_content("c", None, None, vec![], vec![]));
        assert!(content_c.contains(r#"hidden="""#));
    }

    // --- MultiAccordion: SSR 状態なし初期描画 ---

    #[test]
    fn multi_accordion_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&MultiAccordion::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- MultiAccordion: hydration 経路（複数同時 open のラウンドトリップ） ---

    #[test]
    fn multi_accordion_hydration_round_trip_multiple_selected() {
        let mut a = MultiAccordion::default();
        dispatch(&mut a, "select", "tab-1");
        dispatch(&mut a, "select", "tab-2");
        let rendered = render(&render_for_hydration(&a));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("tab-1"));
        assert!(rendered.contains("tab-2"));

        let restored = MultiAccordion::from_hydration_attrs(&a.hydration_attrs()).unwrap();
        assert_eq!(restored, a);
    }

    #[test]
    fn multi_accordion_hydration_round_trip_unselected() {
        let a = MultiAccordion::default();
        let restored = MultiAccordion::from_hydration_attrs(&a.hydration_attrs()).unwrap();
        assert_eq!(restored, a);
    }

    #[test]
    fn multi_accordion_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = MultiAccordion::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-selected".to_string())
        );
    }

    #[test]
    fn multi_accordion_from_hydration_attrs_duplicate_value_rejected_not_panicking() {
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["a".to_string(), "a".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = MultiAccordion::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰: MultiAccordion の dispatch payload/hydration 経路 ---

    #[test]
    fn multi_accordion_dispatch_select_payload_is_escaped_on_render() {
        let mut a = MultiAccordion::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut a, "select", payload));

        let rendered = render(&render_for_hydration(&a));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn multi_accordion_xss_payload_in_hydration_selected_is_rejected_not_rendered() {
        // 改ざん耐性: from_hydration_attrs は不正な値（重複）を panic せず
        // 拒否する（MultiSelect の既存保証を MultiAccordion 経由でも固定）。
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&[
            "<script>alert(1)</script>".to_string(),
            "<script>alert(1)</script>".to_string(),
        ]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = MultiAccordion::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn multi_accordion_view_root_is_element_for_render_for_hydration() {
        // render_for_hydration はルートが Node::Element であることを前提に
        // 属性を合成する（`crates/interactive/src/lib.rs` 参照）。本型の
        // view() が常に Element を返すことを固定する回帰テスト。
        let node = MultiAccordion::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }
}

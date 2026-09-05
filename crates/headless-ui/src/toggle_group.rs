//! ToggleGroup（複数トグルボタンのグループ）headless コンポーネント
//! （イシュー #746、Phase 3「headless + styled」一括方式）。
//!
//! ark-ui の Toggle Group
//!（`.claude/skills/ark-ui/references/components/disclosure/toggle-group.md`）を
//! 参考に、Root / Item の 2 anatomy パーツと、Phase 1（#524）の
//! [`crate::state::SingleSelect`] を埋め込んだ「高々 1 項目が押下される」
//! 状態機械 [`ToggleGroup`]、および [`crate::state::MultiSelect`]
//! （イシュー #594）を埋め込んだ「複数項目が同時に押下される」状態機械
//! [`MultiToggleGroup`] を提供する（構成は [`crate::accordion::Accordion`]/
//! [`crate::accordion::MultiAccordion`] のひな型を踏襲する）。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`item`]、いずれも純粋関数で
//! 完結）を直接呼んで組み立てる。CSR/hydration は用途に応じて
//! [`ToggleGroup`] または [`MultiToggleGroup`]（いずれも
//! [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を使い分ける。両者とも
//! dispatch は `"toggle"` のみを受理する（deselectable 既定 true。§out-of-scope
//! 参照）。`fandhe-frontend-pre-styled-ui`（イシュー #746）が本モジュールを
//! 呼んでスタイル済み ToggleGroup を組み立てる想定である。
//!
//! # `item` の意味論（[`crate::toggle::Toggle`] との関係）
//!
//! [`item`] は単体の [`crate::toggle::root`] と同じ「押下状態を持つ
//! ネイティブ `<button type=\"button\">`」であり、`aria-pressed`/`data-state`
//! （`"on"`/`"off"`、[`crate::state::pressed_data_state`]）の語彙も揃える
//! （ToggleGroup の各項目は独立した Toggle の集合という ark-ui の位置付け
//! に従う）。[`root`] のみが `role="group"` を持つグループ化コンテナである
//! 点が [`crate::radio_group::root`]（`role="radiogroup"`）との違い（ボタン
//! グループであり input グループではないため）。
//!
//! # 参照突合（イシュー #1630）
//!
//! ark-ui（Zag `toggle-group`）/ Radix Primitives Toggle Group の
//! Anatomy・Data Attributes・Keyboard Interactions 表と突合し、以下を
//! 是正した:
//!
//! - [`ToggleGroupProps`]（`disabled`/`orientation`/`roving_focus`）を新設。
//!   [`root`]/[`item`] が共通で参照するグループ状態束であり、`disabled` は
//!   root の `data-disabled` に加えて全 item への伝播（`props.disabled ||
//!   item 個別の disabled`）を担う（ark-ui/Radix とも Root disabled は
//!   全 item を無効化する契約のため）。
//! - [`item`] へ `data-orientation`（`props.orientation` が `Some` のときの
//!   み root と同値）を追加した（ark-ui/Radix とも Item に
//!   `data-orientation` を持つが、既存実装は root のみへ付与していた）。
//! - [`ToggleGroupProps::roving_focus`]（既定 **false**）と [`item`] の
//!   `focused` 引数により、opt-in で roving tabindex（`tabindex="0"`/
//!   `"-1"`）を SSR 側から出力できるようにした（[`crate::toolbar::root`]
//!   の `roving_tabindex` と同型のヘルパを本モジュール内に複製する。
//!   共通化は out-of-scope として記録、モジュール末尾「意図的に合わせな
//!   かった点」参照）。既定を ark/Radix の `true` ではなく **false** にした
//!   のは、既存呼び出し（styled 層・docs-site showcase・golden テスト）の
//!   出力バイト一致を優先し、no-JS SSR で全 item を Tab 到達可能に保つ
//!   ため（意図的差分）。矢印キーによる実際のフォーカス移動 DOM 配線は
//!   従来どおり `fandhe-frontend-wasm-full` の `keynav` モジュール
//!   （イシュー #1075）が担い、`roving_focus` はその配線が book-keeping
//!   する `tabindex` の SSR 初期値を与えるのみである。
//! - [`drop_reserved`]（[`crate::radio_group`] の同名ヘルパと同型）を追加
//!   し、呼び出し側 `attrs` からの `tabindex`/`data-value`/`aria-pressed`/
//!   `data-orientation` 等の偽装を fail-closed に除去する。
//!
//! 意図的に合わせなかった点（差分メモ、Issue コメントへ転記）:
//!
//! - **root の `data-orientation` は `Option`（ark 既定 `horizontal` の
//!   常時出力へは合わせない）**: `None`（欠落）のとき
//!   `fandhe-frontend-wasm-full` の `keynav::toggle_group_next_index`/
//!   `radio_next_index` が両軸の矢印キーを受理する既存契約のため。
//! - **`data-focus`（ark）は付与しない**: pointer/focus のローカル操作
//!   状態は SSR 静的出力に持たせない
//!   （`docs/policy/intentional-non-adoption.md` §3.25 規則 2、
//!   [`crate::radio_group`]/[`crate::toggle`] と同じ契約）。
//! - **`deselectable=false`/`loopFocus=false`（ark オプション）は未提供**:
//!   モジュール冒頭 §out-of-scope 参照。
//! - **`roving_tabindex`/`drop_reserved` を [`crate::toolbar`] と共通化
//!   しない**: `toolbar` 側のヘルパは private のため複製した。共通ヘルパ
//!   crate 抽出は本イシューのスコープ外（後続 Issue 化を検討）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`type`/`role`）はすべて `&'static str`
//!   リテラルで固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の既存不変
//!   条件をそのまま継承する）。
//! - 動的値（`value`/`labelled_by`/呼び出し側 `attrs`/`children` テキスト/
//!   dispatch payload/hydration 属性）は [`fandhe_frontend_core::render`] の
//!   既定エスケープを必ず経由する（REQ-1）。本モジュールは `raw_html()` を
//!   使用しない。
//! - `data-state` 値語彙（`"on"`/`"off"`）は [`crate::state`]
//!   （[`crate::state::pressed_data_state`]）が一元管理し、本モジュールで
//!   独自の値を作らない。
//! - [`drop_reserved`] は呼び出し側 `attrs` が `role`/`data-orientation`/
//!   `aria-labelledby`/`data-disabled`（root）、`type`/`aria-pressed`/
//!   `data-state`/`data-pressed`/`data-disabled`/`data-value`/
//!   `data-orientation`/`disabled`/`tabindex`（item）を偽装することを
//!   ASCII 大文字小文字無視で fail-closed に防ぐ（[`crate::anatomy::Anatomy::part`]
//!   の `data-scope`/`data-part` 除去と同型の防御、イシュー #1630 で追加）。
//! - [`ToggleGroup::decode_action`]/[`MultiToggleGroup::decode_action`] は
//!   クライアント由来の文字列アクション名を `"toggle"` のみに絞る
//!   （fail-closed）。
//! - hydration 属性（`data-hydrate-selected`）はクライアント側で改ざんされ
//!   うる入力として扱う。[`ToggleGroup`]/[`MultiToggleGroup`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は
//!   [`crate::state::SingleSelect`]/[`crate::state::MultiSelect`] へ全委譲
//!   することで、panic せず `HydrateError` を返す既存保証をそのまま継承する。
//!
//! # out-of-scope（本イシュー #746/#1630 のスコープ外）
//!
//! - **roving focus の実 DOM 配線（矢印キーによるフォーカス移動）**: SSR
//!   静的マークアップに寄与しない CSR 挙動層（`fandhe-frontend-wasm-full`
//!   の `keynav` モジュール）の責務のため未提供（Tabs（#528/#582）・
//!   RadioGroup（#536）と同じ判断）。`crates/wasm-full/src/keynav.rs`
//!   （イシュー #1075）で実装済み（`toggle_group_next_index` は
//!   `radio_next_index` を共有する）。[`ToggleGroupProps::roving_focus`]/
//!   [`item`] の `focused` 引数（イシュー #1630）は SSR 側の `tabindex`
//!   初期値の book-keeping のみを提供し、キー入力のハンドリング自体は
//!   引き続き `keynav` の責務である。
//! - **single モードの deselectable=false（ark-ui オプション）**: 常時
//!   deselectable（クリックで選択解除可能）のみを提供する。「常に 1 個は
//!   選択されている」制約が必要な用途は本イシューのスコープ外
//!   （`out-of-scope-tracking.md` に従い後続 Issue 化を検討）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_labelledby, aria_pressed, role};
use crate::data_attrs::{data_disabled, data_orientation, data_pressed, data_state, Orientation};
use crate::state::{
    pressed_data_state, MultiSelect, MultiSelectAction, SingleSelect, SingleSelectAction,
};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// ToggleGroup の anatomy（`data-scope="toggle-group"`）。
const ANATOMY: Anatomy = anatomy("toggle-group");

/// ToggleGroup 全体へ宣言的に反映する状態束（イシュー #1630 で新設）。
///
/// `Default` は「無効化なし・orientation 未指定・roving focus 無効」（SSR
/// の状態なし初期描画に対応する既定値）。[`root`]/[`item`] の双方が本構造体
/// を参照する（[`crate::radio_group::RadioGroupProps`] と同型のパターン）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToggleGroupProps {
    /// グループ全体の無効化状態。`true` で [`root`] の `data-disabled` に
    /// 加えて全 [`item`] へも `data-disabled`/ネイティブ `disabled` として
    /// 伝播する（ark-ui/Radix の Root disabled が全 item を無効化する契約、
    /// モジュール doc「参照突合」節参照）。
    pub disabled: bool,
    /// `Some` のときのみ [`root`]/[`item`] 双方へ `data-orientation` を
    /// 付与する（`None` のときは `fandhe-frontend-wasm-full` の keynav が
    /// 両軸の矢印キーを受理する既存契約を維持する。モジュール doc「意図的
    /// に合わせなかった点」参照）。
    pub orientation: Option<Orientation>,
    /// `true` のとき [`item`] が `focused` 引数に応じて `tabindex="0"`/
    /// `"-1"`（roving tabindex）を出力する。既定 `false`（SSR 初期描画では
    /// tabindex を出力せず、no-JS でも全 item が Tab で到達可能なまま）。
    pub roving_focus: bool,
}

/// [`root`] 固有の固定属性キー一覧（呼び出し側 `attrs` からの偽装除去対象、
/// [`crate::radio_group::ROOT_RESERVED`] と同型のパターン）。
const ROOT_RESERVED: &[&str] = &[
    "role",
    "data-orientation",
    "aria-labelledby",
    "data-disabled",
];

/// [`item`] 固有の固定属性キー一覧（呼び出し側 `attrs` からの偽装除去対象）。
const ITEM_RESERVED: &[&str] = &[
    "type",
    "aria-pressed",
    "data-state",
    "data-pressed",
    "data-disabled",
    "data-value",
    "data-orientation",
    "disabled",
    "tabindex",
];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する。`Anatomy::part` の `data-scope`/`data-part` フィルタと同型の
/// fail-closed 防御であり、各パーツが追加で持つ固定属性の呼び出し側からの
/// 偽装を防ぐ（`crate::radio_group` の同名ヘルパと同型、イシュー #1630）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// focus 対象なら `tabindex="0"`、そうでなければ `tabindex="-1"` を返す
/// roving tabindex の共通ヘルパ（[`crate::toolbar::root`] の同名関数と
/// 同型。private のため複製する、モジュール doc「意図的に合わせなかった
/// 点」参照）。
fn roving_tabindex(focused: bool) -> (&'static str, &'static str) {
    if focused {
        ("tabindex", "0")
    } else {
        ("tabindex", "-1")
    }
}

/// Root パーツ（`div`、`role="group"`）。
///
/// `labelled_by` が `Some` のときのみ `aria-labelledby` を付与する
/// （[`crate::radio_group::root`] と同じ「名前なしの関連付けを作らない」
/// 方針）。`props.orientation` が `Some` のときのみ `data-orientation` を
/// 付与する。`role="group"` には WAI-ARIA 上 `aria-orientation` は許可され
/// ていない（`radiogroup`/`toolbar` 等とは異なる）ため、CSS 用途の
/// `data-orientation` のみを出力し `aria-orientation` は付与しない
/// （イシュー #746 PR #791 Bugbot 指摘対応）。
#[must_use]
pub fn root<'a>(
    props: &ToggleGroupProps,
    labelled_by: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("group")];
    if let Some(orientation) = props.orientation {
        merged.push(data_orientation(orientation));
    }
    if let Some(id) = labelled_by {
        merged.push(aria_labelledby(id));
    }
    merged.extend(data_disabled(props.disabled));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Item パーツ（`button type="button"`）。グループ内の押下可能な選択肢
/// 1 個。ネイティブ `<button>` のため、フォーカス・クリック・Space/Enter
/// キー操作はブラウザ既定動作で成立する（[`crate::toggle::root`] と同型）。
///
/// `value` は `data-value` として動的値のまま出力し、`render()` の既定
/// エスケープを必ず経由する（REQ-1、[`crate::radio_group::item`] と同型）。
/// `props.disabled || disabled` を実効無効化状態として扱う（Root disabled
/// の伝播、モジュール doc「参照突合」節参照）。`props.orientation` が
/// `Some` のときのみ root と同値の `data-orientation` を出力する。
/// `props.roving_focus` が `true` のときのみ `focused` に応じて
/// `tabindex="0"`/`"-1"` を出力する（`false` のときは `focused` を無視し
/// `tabindex` を出力しない、モジュール doc「参照突合」節参照）。
#[must_use]
pub fn item<'a>(
    props: &ToggleGroupProps,
    pressed: bool,
    focused: bool,
    disabled: bool,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_RESERVED);
    let effective_disabled = props.disabled || disabled;
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_pressed(pressed),
        data_state(pressed_data_state(pressed)),
        ("data-value", value),
    ];
    if let Some(orientation) = props.orientation {
        merged.push(data_orientation(orientation));
    }
    merged.extend(data_pressed(pressed));
    merged.extend(data_disabled(effective_disabled));
    if effective_disabled {
        merged.push(("disabled", ""));
    }
    if props.roving_focus {
        merged.push(roving_tabindex(focused));
    }
    merged.extend(attrs);
    ANATOMY.part("item", "button", merged, children)
}

/// [`SingleSelect`]（#524）を埋め込んだ ToggleGroup（single モード）の
/// 状態機械。「高々 1 項目が押下される」制約を型レベルで保証する。
///
/// 常時 deselectable（選択中の項目を再度クリックすると解除される）のみを
/// 提供する（モジュール doc §out-of-scope 参照）。`Default` は未選択（SSR
/// の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ToggleGroup {
    select: SingleSelect,
}

impl ToggleGroup {
    /// 現在押下中の項目値（未選択なら `None`）。
    #[must_use]
    pub fn value(&self) -> Option<&str> {
        self.select.selected()
    }

    /// 指定した項目値が押下中かどうか。
    #[must_use]
    pub fn is_pressed(&self, value: &str) -> bool {
        self.select.is_selected(value)
    }

    /// [`item`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        props: &ToggleGroupProps,
        value: &'a str,
        focused: bool,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(
            props,
            self.is_pressed(value),
            focused,
            disabled,
            value,
            attrs,
            children,
        )
    }
}

impl Component for ToggleGroup {
    type Action = SingleSelectAction;

    /// 型付き API（プログラム的な呼び出し）では [`SingleSelectAction::Select`]/
    /// [`SingleSelectAction::Deselect`] も許す。クライアント由来の文字列
    /// dispatch 境界は `"toggle"` のみを受理する（[`Self::decode_action`]）。
    fn update(&mut self, action: SingleSelectAction) {
        self.select.update(action);
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（[`root`]、
    /// children 空。[`crate::accordion::Accordion::view`] と同じ位置付け）。
    fn view(&self) -> Node {
        root(&ToggleGroupProps::default(), None, Vec::new(), Vec::new())
    }

    /// クライアント由来の文字列アクション名を `"toggle"` のみに絞る
    /// （fail-closed。deselectable 既定 true、モジュール doc 参照）。
    fn decode_action(name: &str, payload: &str) -> Option<SingleSelectAction> {
        match name {
            "toggle" => Some(SingleSelectAction::Toggle(payload.to_string())),
            _ => None,
        }
    }
}

impl Hydrate for ToggleGroup {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.select.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            select: SingleSelect::from_hydration_attrs(attrs)?,
        })
    }
}

/// [`MultiSelect`]（#594）を埋め込んだ ToggleGroup（multiple モード）の
/// 状態機械。「0 個以上の項目が同時に押下される」ことを許す
/// （[`crate::accordion::MultiAccordion`] と同じひな型）。`Default` は
/// 全未選択（SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MultiToggleGroup {
    select: MultiSelect,
}

impl MultiToggleGroup {
    /// 現在押下中の項目値（押下順）。
    #[must_use]
    pub fn values(&self) -> &[String] {
        self.select.selected()
    }

    /// 指定した項目値が押下中かどうか。
    #[must_use]
    pub fn is_pressed(&self, value: &str) -> bool {
        self.select.is_selected(value)
    }

    /// [`item`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        props: &ToggleGroupProps,
        value: &'a str,
        focused: bool,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(
            props,
            self.is_pressed(value),
            focused,
            disabled,
            value,
            attrs,
            children,
        )
    }
}

impl Component for MultiToggleGroup {
    type Action = MultiSelectAction;

    /// 型付き API では [`MultiSelectAction::Select`]/[`MultiSelectAction::Deselect`]
    /// も許す。クライアント由来の文字列 dispatch 境界は `"toggle"` のみを
    /// 受理する（[`Self::decode_action`]）。
    fn update(&mut self, action: MultiSelectAction) {
        self.select.update(action);
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー
    /// （[`crate::accordion::MultiAccordion::view`] と同じ位置付け）。
    fn view(&self) -> Node {
        root(&ToggleGroupProps::default(), None, Vec::new(), Vec::new())
    }

    /// クライアント由来の文字列アクション名を `"toggle"` のみに絞る
    /// （fail-closed）。
    fn decode_action(name: &str, payload: &str) -> Option<MultiSelectAction> {
        match name {
            "toggle" => Some(MultiSelectAction::Toggle(payload.to_string())),
            _ => None,
        }
    }
}

impl Hydrate for MultiToggleGroup {
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

    // --- 各パーツの data-scope/data-part/ARIA/data-state 出力 ---

    #[test]
    fn root_outputs_group_role() {
        let html = render(&root(&ToggleGroupProps::default(), None, vec![], vec![]));
        assert!(html.contains(r#"data-scope="toggle-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="group""#));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("aria-labelledby"));
        assert!(!html.contains("orientation"));
    }

    #[test]
    fn root_disabled_true_adds_data_disabled() {
        let props = ToggleGroupProps {
            disabled: true,
            ..ToggleGroupProps::default()
        };
        let html = render(&root(&props, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn root_labelled_by_some_outputs_aria_labelledby() {
        let html = render(&root(
            &ToggleGroupProps::default(),
            Some("group-label"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-labelledby="group-label""#));
    }

    #[test]
    fn root_orientation_some_outputs_data_orientation_without_aria() {
        let props = ToggleGroupProps {
            orientation: Some(Orientation::Horizontal),
            ..ToggleGroupProps::default()
        };
        let html = render(&root(&props, None, vec![], vec![]));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        // role="group" には aria-orientation は許可されていない
        // （aria-allowed-attr 違反、イシュー #746 PR #791 Bugbot 指摘対応）。
        assert!(!html.contains("aria-orientation"));
    }

    #[test]
    fn caller_root_attrs_cannot_override_role_orientation_labelledby_disabled() {
        let props = ToggleGroupProps {
            disabled: true,
            orientation: Some(Orientation::Horizontal),
            ..ToggleGroupProps::default()
        };
        let html = render(&root(
            &props,
            Some("group-label"),
            vec![
                ("role", "attacker"),
                ("data-orientation", "attacker"),
                ("aria-labelledby", "attacker"),
                ("data-disabled", "attacker"),
                ("DATA-DISABLED", "attacker-case"),
            ],
            vec![],
        ));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(html.contains(r#"aria-labelledby="group-label""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn item_reflects_pressed_state_and_disabled() {
        let props = ToggleGroupProps::default();
        let pressed = render(&item(&props, true, false, false, "bold", vec![], vec![]));
        assert!(pressed.contains("<button"));
        assert!(pressed.contains(r#"type="button""#));
        assert!(pressed.contains(r#"aria-pressed="true""#));
        assert!(pressed.contains(r#"data-state="on""#));
        assert!(pressed.contains(r#"data-pressed="""#));
        assert!(pressed.contains(r#"data-value="bold""#));
        assert!(!pressed.contains("data-disabled"));

        let unpressed_disabled =
            render(&item(&props, false, false, true, "italic", vec![], vec![]));
        assert!(unpressed_disabled.contains(r#"aria-pressed="false""#));
        assert!(unpressed_disabled.contains(r#"data-state="off""#));
        assert!(!unpressed_disabled.contains("data-pressed"));
        assert!(unpressed_disabled.contains(r#"data-disabled="""#));
        assert!(unpressed_disabled.contains(r#"disabled=""#));
    }

    #[test]
    fn item_outputs_data_orientation_matching_root_when_some() {
        let props = ToggleGroupProps {
            orientation: Some(Orientation::Vertical),
            ..ToggleGroupProps::default()
        };
        let html = render(&item(&props, false, false, false, "bold", vec![], vec![]));
        assert!(html.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn item_omits_data_orientation_when_props_orientation_none() {
        let html = render(&item(
            &ToggleGroupProps::default(),
            false,
            false,
            false,
            "bold",
            vec![],
            vec![],
        ));
        assert!(!html.contains("data-orientation"));
    }

    #[test]
    fn item_inherits_disabled_from_root_props() {
        let props = ToggleGroupProps {
            disabled: true,
            ..ToggleGroupProps::default()
        };
        // item 自身は disabled=false のまま渡しても、props.disabled の
        // 伝播により無効化される（Root disabled の伝播、モジュール doc
        // 「参照突合」節参照）。
        let html = render(&item(&props, false, false, false, "bold", vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"disabled=""#));
    }

    #[test]
    fn item_roving_focus_false_omits_tabindex_regardless_of_focused() {
        let props = ToggleGroupProps::default();
        let focused = render(&item(&props, false, true, false, "bold", vec![], vec![]));
        let unfocused = render(&item(&props, false, false, false, "bold", vec![], vec![]));
        assert!(!focused.contains("tabindex"));
        assert!(!unfocused.contains("tabindex"));
    }

    #[test]
    fn item_roving_focus_true_reflects_focused_as_tabindex() {
        let props = ToggleGroupProps {
            roving_focus: true,
            ..ToggleGroupProps::default()
        };
        let focused = render(&item(&props, false, true, false, "bold", vec![], vec![]));
        let unfocused = render(&item(&props, false, false, false, "bold", vec![], vec![]));
        assert!(focused.contains(r#"tabindex="0""#));
        assert!(unfocused.contains(r#"tabindex="-1""#));
    }

    #[test]
    fn item_does_not_output_data_focus() {
        // pointer/focus のローカル操作状態は SSR 静的出力に持たせない
        // （`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。
        let props = ToggleGroupProps::default();
        let html = render(&item(&props, true, true, false, "bold", vec![], vec![]));
        assert!(!html.contains("data-focus"));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_attrs_cannot_override_anatomy_scope_and_part() {
        let html = render(&item(
            &ToggleGroupProps::default(),
            true,
            false,
            false,
            "bold",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toggle-group""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn caller_item_attrs_cannot_override_reserved_keys() {
        let props = ToggleGroupProps {
            roving_focus: true,
            orientation: Some(Orientation::Horizontal),
            ..ToggleGroupProps::default()
        };
        let html = render(&item(
            &props,
            true,
            true,
            false,
            "bold",
            vec![
                ("type", "submit"),
                ("aria-pressed", "false"),
                ("data-state", "off"),
                ("data-pressed", "attacker"),
                ("data-disabled", "attacker"),
                ("data-value", "attacker"),
                ("data-orientation", "attacker"),
                ("disabled", "attacker"),
                ("tabindex", "99"),
                ("TABINDEX", "99"),
            ],
            vec![],
        ));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-pressed="true""#));
        assert!(html.contains(r#"data-state="on""#));
        assert!(html.contains(r#"data-value="bold""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(!html.contains("attacker"));
        assert!(!html.contains("submit"));
        assert!(!html.contains("99"));
        assert!(!html.contains(r#"disabled="attacker""#));
    }

    // --- root > item*2 の組み立て ---

    #[test]
    fn full_assembly_root_with_two_items() {
        let props = ToggleGroupProps::default();
        let node = root(
            &props,
            Some("group-label"),
            vec![],
            vec![
                item(&props, true, false, false, "bold", vec![], vec![text("B")]),
                item(
                    &props,
                    false,
                    false,
                    false,
                    "italic",
                    vec![],
                    vec![text("I")],
                ),
            ],
        );
        let html = render(&node);
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"data-value="bold""#));
        assert!(html.contains(r#"data-value="italic""#));
        assert!(html.contains("B"));
        assert!(html.contains("I"));
    }

    // --- ToggleGroup（single）: dispatch 統合、"toggle" のみ受理 ---

    #[test]
    fn toggle_group_default_is_unpressed() {
        let g = ToggleGroup::default();
        assert_eq!(g.value(), None);
        assert!(!g.is_pressed("bold"));
    }

    #[test]
    fn toggle_group_dispatch_toggle_selects_and_deselects_at_most_one_item() {
        let mut g = ToggleGroup::default();
        assert!(dispatch(&mut g, "toggle", "bold"));
        assert!(g.is_pressed("bold"));

        // 選択中の項目を再度 toggle すると解除される（deselectable 既定）。
        assert!(dispatch(&mut g, "toggle", "bold"));
        assert!(!g.is_pressed("bold"));

        assert!(dispatch(&mut g, "toggle", "bold"));
        assert!(dispatch(&mut g, "toggle", "italic"));
        assert!(!g.is_pressed("bold"));
        assert!(g.is_pressed("italic"));
    }

    #[test]
    fn toggle_group_dispatch_ignores_select_deselect_and_unknown_action() {
        let mut g = ToggleGroup::default();
        assert!(!dispatch(&mut g, "select", "bold"));
        assert!(!dispatch(&mut g, "deselect", ""));
        assert!(!dispatch(&mut g, "no_such_action", "bold"));
        assert!(!g.is_pressed("bold"));
    }

    #[test]
    fn toggle_group_typed_update_select_and_deselect() {
        let mut g = ToggleGroup::default();
        g.update(SingleSelectAction::Select("bold".to_string()));
        assert!(g.is_pressed("bold"));
        g.update(SingleSelectAction::Deselect);
        assert_eq!(g.value(), None);
    }

    #[test]
    fn toggle_group_convenience_method_reflects_state() {
        let mut g = ToggleGroup::default();
        dispatch(&mut g, "toggle", "bold");
        let props = ToggleGroupProps::default();
        let item_bold = render(&g.item(&props, "bold", false, false, vec![], vec![]));
        assert!(item_bold.contains(r#"data-state="on""#));
        let item_italic = render(&g.item(&props, "italic", false, false, vec![], vec![]));
        assert!(item_italic.contains(r#"data-state="off""#));
    }

    #[test]
    fn toggle_group_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&ToggleGroup::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    #[test]
    fn toggle_group_hydration_round_trip() {
        let mut g = ToggleGroup::default();
        dispatch(&mut g, "toggle", "bold");
        let rendered = render(&render_for_hydration(&g));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("bold"));

        let restored = ToggleGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }

    #[test]
    fn toggle_group_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = ToggleGroup::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-selected".to_string())
        );
    }

    #[test]
    fn toggle_group_from_hydration_attrs_invalid_value_does_not_panic() {
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["a".to_string(), "b".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = ToggleGroup::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- MultiToggleGroup: dispatch 統合、"toggle" のみ受理、複数同時押下 ---

    #[test]
    fn multi_toggle_group_default_is_empty() {
        let g = MultiToggleGroup::default();
        assert!(g.values().is_empty());
        assert!(!g.is_pressed("bold"));
    }

    #[test]
    fn multi_toggle_group_dispatch_toggle_allows_multiple_simultaneous_items() {
        let mut g = MultiToggleGroup::default();
        assert!(dispatch(&mut g, "toggle", "bold"));
        assert!(dispatch(&mut g, "toggle", "italic"));
        assert!(g.is_pressed("bold"));
        assert!(g.is_pressed("italic"));

        assert!(dispatch(&mut g, "toggle", "bold"));
        assert!(!g.is_pressed("bold"));
        assert!(g.is_pressed("italic"));
    }

    #[test]
    fn multi_toggle_group_dispatch_ignores_select_deselect_and_unknown_action() {
        let mut g = MultiToggleGroup::default();
        assert!(!dispatch(&mut g, "select", "bold"));
        assert!(!dispatch(&mut g, "deselect", "bold"));
        assert!(!dispatch(&mut g, "no_such_action", "bold"));
        assert!(!g.is_pressed("bold"));
    }

    #[test]
    fn multi_toggle_group_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&MultiToggleGroup::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    #[test]
    fn multi_toggle_group_hydration_round_trip() {
        let mut g = MultiToggleGroup::default();
        dispatch(&mut g, "toggle", "bold");
        dispatch(&mut g, "toggle", "italic");
        let restored = MultiToggleGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }

    #[test]
    fn multi_toggle_group_from_hydration_attrs_rejects_duplicate_items() {
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["bold".to_string(), "bold".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = MultiToggleGroup::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰: value/labelled_by/呼び出し側 attrs/children/dispatch payload/hydration ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn root_labelled_by_payload_is_escaped_on_render() {
        let html = render(&root(
            &ToggleGroupProps::default(),
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_value_payload_is_escaped_on_render() {
        let html = render(&item(
            &ToggleGroupProps::default(),
            false,
            false,
            false,
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
            &ToggleGroupProps::default(),
            None,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&item(
            &ToggleGroupProps::default(),
            false,
            false,
            false,
            "bold",
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn toggle_group_dispatch_payload_is_escaped_on_render() {
        let mut g = ToggleGroup::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut g, "toggle", payload));

        let rendered = render(&render_for_hydration(&g));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
    }

    #[test]
    fn toggle_group_xss_payload_in_hydration_selected_is_rejected_not_rendered() {
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["<script>alert(1)</script>".to_string(), "b".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = ToggleGroup::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn multi_toggle_group_dispatch_payload_is_escaped_on_render() {
        let mut g = MultiToggleGroup::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut g, "toggle", payload));

        let rendered = render(&render_for_hydration(&g));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
    }

    #[test]
    fn multi_toggle_group_xss_payload_in_hydration_selected_is_rejected_not_rendered() {
        // MultiSelect は「重複値」を不変条件違反として拒否する（一意な
        // script ペイロード単体は不正値ではない。`MultiAccordion` の同型
        // テスト・`crates/headless-ui/src/state.rs` の `MultiSelect::Hydrate`
        // 実装参照）。改ざん耐性の固定は重複ペイロードで検証する。
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&[
            "<script>alert(1)</script>".to_string(),
            "<script>alert(1)</script>".to_string(),
        ]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = MultiToggleGroup::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

//! RadioGroup: ark-ui の Radio Group
//!（`.claude/skills/ark-ui/references/components/form/radio-group.md`）を
//! 参考にした headless ラジオグループ（イシュー #536、親トラッキング #534、
//! Phase 2 親 #525）。
//!
//! Root / Label / Item / ItemControl / ItemText / ItemHiddenInput の 6
//! anatomy パーツと、Phase 1（#524）の [`crate::state::SingleSelect`] を
//! 埋め込んだ「高々 1 項目が選択される」状態機械 [`RadioGroup`] を提供する
//! （構成は [`crate::accordion::Accordion`] のひな型を踏襲する）。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`label`]/[`item`]/
//! [`item_control`]/[`item_text`]/[`item_hidden_input`]、いずれも純粋関数で
//! 完結）を直接呼んで組み立てる。CSR/hydration は [`RadioGroup`]
//!（[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"select"`）で「高々 1 項目が選択される」状態遷移をする。
//! `fandhe-frontend-pre-styled-ui`（#546〜）が本モジュールを呼んでスタイル
//! 済み RadioGroup を組み立てる想定である。
//!
//! # data-state 語彙（`"checked"`/`"unchecked"`）
//!
//! WAI-ARIA radio パターンの状態語彙は [`crate::state::OpenState`] の
//! `"open"`/`"closed"` とは異なる（「開閉」ではなく「選択」を表すため）。
//! [`DATA_STATE_CHECKED`]/[`DATA_STATE_UNCHECKED`] は Checkbox（#535）/
//! Switch（#537）と共有する値語彙であり、イシュー #595 で
//! [`crate::state::DATA_STATE_CHECKED`]/[`crate::state::DATA_STATE_UNCHECKED`]
//! （共通機械 [`crate::state::Checkable`] が使う値定数）へ共通化した。本
//! モジュールの `DATA_STATE_CHECKED`/`DATA_STATE_UNCHECKED` はその
//! 互換 re-export であり、既存公開パス `radio_group::DATA_STATE_CHECKED`
//! を維持する。状態機械そのもの（「選択値」を持つ [`SingleSelect`]）は
//! 2 値の [`crate::state::Checkable`] へ写像できないため、引き続き
//! [`SingleSelect`] を埋め込む（値語彙の共通化のみが #595 の対象）。
//!
//! # ネイティブ semantics
//!
//! [`item_hidden_input`] が生成するネイティブ `<input type="radio">` が
//! チェック状態・フォーム送信・キーボード操作・グループ内排他選択を担う。
//! そのため装飾パーツ（[`item_control`]）には `role="radio"` /
//! `aria-checked` を重複付与しない（二重読み上げ防止、Accordion の
//! `item_control`/`item_indicator` と同じ最小主義）。[`item`] は `<label>`
//! を採用し（ark-ui「Item renders as `<label>`」準拠）、内包する
//! [`item_hidden_input`] とのネイティブ関連付け（クリック委譲）が JS
//! なしで成立する。
//!
//! [`label`] は RadioGroup 全体の見出しであり、`<label>` ではなく `<span>`
//! を採用する（`<label>` は labelable な単一コントロール専用要素であり、
//! グループ見出しには不適。関連付けは [`root`] の `aria-labelledby` で
//! 成立させる）。
//!
//! # セキュリティ不変条件
//!
//! 各関数は属性 Vec を組み立てて [`crate::anatomy::Anatomy::part`]（内部で
//! [`fandhe_frontend_core::el`] を 1 回呼ぶ）へ委譲するだけであり、独自の
//! エスケープ処理・HTML 文字列直接組み立てを持たない。動的値（`value` /
//! `name` / `id` / `labelled_by` / 呼び出し側 `attrs` / `children` テキスト /
//! dispatch payload / hydration 属性）は [`fandhe_frontend_core::render`] の
//! 既定エスケープを必ず経由する（REQ-1）。本モジュールは `raw_html()` を
//! 使用しない。[`RadioGroup::decode_action`] はクライアント由来の文字列
//! アクション名を `"select"` のみに絞る（fail-closed。改ざんされうる
//! dispatch 境界からの選択解除ジェスチャは受理しない）。hydration 属性は
//! [`crate::state::SingleSelect`] の [`fandhe_frontend_interactive::Hydrate`]
//! 実装へ全委譲し、panic せず `HydrateError` を返す既存保証をそのまま
//! 継承する。
//!
//! # out-of-scope（本イシュー #536 のスコープ外）
//!
//! - **Indicator パーツ**: 選択項目へ追従する浮動ビジュアルインジケータの
//!   位置計算は CSR 挙動層の責務のため未提供（Accordion での「Tabs
//!   indicator 除外と同じ判断」を踏襲）。
//! - **キーボードナビゲーション（矢印キー・roving tabindex）**: SSR 静的
//!   マークアップに寄与しない CSR 挙動層のため未提供。
//! - **Field（#538）との `aria-describedby` / `data-invalid` 連携**: #538 の
//!   スコープ。
//!
//! `"checked"`/`"unchecked"` 語彙の共通化（Checkbox #535 / Switch #537 と
//! 揃える）は #595 で解消済み（本モジュール冒頭「data-state 語彙」節参照）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_labelledby, aria_orientation, role};
use crate::data_attrs::{data_disabled, data_orientation, data_state, Orientation};
use crate::state::{checked_data_state, SingleSelect, SingleSelectAction};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// `data-state` 属性値 "checked"。WAI-ARIA radio パターンの選択語彙
/// （[`crate::state::OpenState`] の `"open"`/`"closed"` とは別語彙。
/// モジュール doc 参照）。[`crate::state::DATA_STATE_CHECKED`] の互換
/// re-export（イシュー #595 で共通化。既存公開パス
/// `radio_group::DATA_STATE_CHECKED` を維持する）。
pub use crate::state::DATA_STATE_CHECKED;
/// `data-state` 属性値 "unchecked"。[`DATA_STATE_CHECKED`] 参照。
pub use crate::state::DATA_STATE_UNCHECKED;

/// RadioGroup の anatomy（`data-scope="radio-group"` 固定）。
const ANATOMY: Anatomy = anatomy("radio-group");

/// Root パーツ（`div`、`role="radiogroup"`）。
///
/// `labelled_by` が `Some` のときのみ `aria-labelledby` を付与する（[`label`]
/// パーツの `id` と対で使う想定。名前なしの関連付けを作らないため `None`
/// のときは属性ごと出力しない）。`orientation` が `Some` のときのみ
/// `data-orientation`/`aria-orientation` を付与する（キーボード操作方向の
/// ヒントであり必須ではないため任意入力とする）。
#[must_use]
pub fn root<'a>(
    disabled: bool,
    orientation: Option<Orientation>,
    labelled_by: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("radiogroup")];
    if let Some(orientation) = orientation {
        merged.push(aria_orientation(orientation));
        merged.push(data_orientation(orientation));
    }
    if let Some(id) = labelled_by {
        merged.push(aria_labelledby(id));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`span`）。RadioGroup 全体の見出し。`id` が `Some` のとき
/// [`root`] の `labelled_by` と対で使う `id` 属性を出力する（関連付け自体は
/// 呼び出し側の責務。`<label>` ではなく `<span>` を採用する理由はモジュール
/// doc 参照）。
#[must_use]
pub fn label<'a>(id: Option<&'a str>, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("label", "span", merged, children)
}

/// Item パーツ（`label`）。選択肢 1 個のラップ要素。ネイティブ `<label>`
/// により、この要素内の [`item_hidden_input`] へのクリック委譲（フォーカス・
/// 選択）が JS なしで機能する。
///
/// `value` は `data-value` として動的値のまま出力し、`render()` の既定
/// エスケープを必ず経由する（REQ-1）。イシュー #580:
/// `fandhe-frontend-wasm-full` の headless 配線基盤（`wasm-full/src/headless.rs`）が
/// `(scope, part) = ("radio-group", "item")` クリックを `"select"` アクションへ
/// 写像する際の payload 源として参照する契約。[`item`] はネイティブ
/// `<label>` のため、内包する [`item_hidden_input`] へのクリック転送で同一
/// クリックが 2 回配線に届き得るが、`"select"`（同一値）は冪等のため実害は
/// ない（モジュール doc 参照）。
#[must_use]
pub fn item<'a>(
    checked: bool,
    disabled: bool,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        data_state(checked_data_state(checked)),
        ("data-value", value),
    ];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item", "label", merged, children)
}

/// ItemControl パーツ（`span`、視覚的なラジオボタンの外枠）。
///
/// チェック状態のセマンティクスは [`item_hidden_input`] のネイティブ
/// `<input type="radio">` が担うため、本要素へ `role="radio"` /
/// `aria-checked` は付与しない（二重読み上げ防止、モジュール doc 参照）。
#[must_use]
pub fn item_control<'a>(checked: bool, disabled: bool, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(checked_data_state(checked))];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item-control", "span", merged, vec![])
}

/// ItemText パーツ（`span`）。選択肢のラベルテキスト。
#[must_use]
pub fn item_text<'a>(
    checked: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(checked_data_state(checked))];
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item-text", "span", merged, children)
}

/// ItemHiddenInput パーツ（`input`）。選択肢のネイティブ
/// `<input type="radio">`。フォーム送信・キーボード操作・グループ内排他
/// 選択（同一 `name` の `<input>` 間）をブラウザのネイティブ semantics に
/// 委ねる（headless SSR として JS なしで自立する。ark-ui「Must include
/// ItemHiddenInput for proper form integration」準拠）。children を持たない
/// 固定パーツ。
///
/// `type="radio"` はリテラル固定。`name`/`value` は動的値だが
/// [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
/// `checked`/`disabled` は true のときのみ存在属性として出力する（ark-ui
/// 流の存在属性規約、[`crate::data_attrs`] と同型）。
#[must_use]
pub fn item_hidden_input<'a>(
    checked: bool,
    disabled: bool,
    name: Option<&'a str>,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "radio"),
        ("value", value),
        data_state(checked_data_state(checked)),
    ];
    if let Some(name) = name {
        merged.push(("name", name));
    }
    if checked {
        merged.push(("checked", ""));
    }
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("item-hidden-input", "input", merged, vec![])
}

/// [`SingleSelect`]（#524）を埋め込んだ RadioGroup（single モード）の状態機械。
///
/// 「高々 1 項目が選択される」制約を型レベルで保証する入口として、
/// [`Self::is_checked`]/[`Self::item_checked_data_state`] が各項目値の
/// チェック状態を決定し、各パーツ関数（[`item`]/[`item_control`]/
/// [`item_text`]/[`item_hidden_input`]）へ注入する利便メソッドを提供する
/// （[`root`]/[`label`] は状態非依存のため利便メソッドを持たない）。SSR
/// での自由関数直接利用（本型を経由しない構成）も引き続き可能。`Default`
/// は未選択（SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RadioGroup {
    select: SingleSelect,
}

impl RadioGroup {
    /// 現在選択中の項目値（未選択なら `None`）。
    #[must_use]
    pub fn value(&self) -> Option<&str> {
        self.select.selected()
    }

    /// 指定した項目値が選択中かどうか。
    #[must_use]
    pub fn is_checked(&self, value: &str) -> bool {
        self.select.is_selected(value)
    }

    /// 項目 `value` の現在の `data-state` 値（`"checked"`/`"unchecked"`）。
    #[must_use]
    pub fn item_checked_data_state(&self, value: &str) -> &'static str {
        checked_data_state(self.is_checked(value))
    }

    /// [`item`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        value: &'a str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(self.is_checked(value), disabled, value, attrs, children)
    }

    /// [`item_control`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item_control<'a>(
        &self,
        value: &str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        item_control(self.is_checked(value), disabled, attrs)
    }

    /// [`item_text`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item_text<'a>(
        &self,
        value: &str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_text(self.is_checked(value), disabled, attrs, children)
    }

    /// [`item_hidden_input`] へ項目 `value` の現在状態を注入する利便メソッド。
    #[must_use]
    pub fn item_hidden_input<'a>(
        &self,
        value: &'a str,
        disabled: bool,
        name: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        item_hidden_input(self.is_checked(value), disabled, name, value, attrs)
    }
}

impl Component for RadioGroup {
    type Action = SingleSelectAction;

    /// 型付き API（プログラム的な呼び出し）では [`SingleSelectAction::Deselect`]
    /// による選択解除も許す（フォームリセット等の用途）。クライアント由来の
    /// 文字列 dispatch 境界で選択解除を受理しないこと（[`Self::decode_action`]）
    /// とは別軸の制約である。
    fn update(&mut self, action: SingleSelectAction) {
        self.select.update(action);
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（[`root`]、
    /// children 空。[`crate::accordion::Accordion::view`] と同じ位置付け）。
    fn view(&self) -> Node {
        root(false, None, None, Vec::new(), Vec::new())
    }

    /// クライアント由来の文字列アクション名を `"select"` のみに絞る
    /// （fail-closed）。WAI-ARIA radio パターンには選択解除ジェスチャが
    /// 存在しないため、`"toggle"`/`"deselect"`/未知アクションはすべて
    /// no-op とする（モジュール doc §セキュリティ不変条件参照）。
    fn decode_action(name: &str, payload: &str) -> Option<SingleSelectAction> {
        match name {
            "select" => Some(SingleSelectAction::Select(payload.to_string())),
            _ => None,
        }
    }
}

impl Hydrate for RadioGroup {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.select.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            select: SingleSelect::from_hydration_attrs(attrs)?,
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
    fn root_outputs_radiogroup_role() {
        let html = render(&root(false, None, None, vec![], vec![]));
        assert!(html.contains(r#"data-scope="radio-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="radiogroup""#));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("aria-labelledby"));
        assert!(!html.contains("orientation"));
    }

    #[test]
    fn root_disabled_true_adds_data_disabled() {
        let html = render(&root(true, None, None, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn root_labelled_by_some_outputs_aria_labelledby() {
        let html = render(&root(false, None, Some("group-label"), vec![], vec![]));
        assert!(html.contains(r#"aria-labelledby="group-label""#));
    }

    #[test]
    fn root_orientation_some_outputs_data_and_aria_orientation() {
        let html = render(&root(
            false,
            Some(Orientation::Vertical),
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(html.contains(r#"aria-orientation="vertical""#));
    }

    #[test]
    fn root_orientation_none_omits_orientation_attrs() {
        let html = render(&root(false, None, None, vec![], vec![]));
        assert!(!html.contains("orientation"));
    }

    #[test]
    fn label_id_some_outputs_id_and_children() {
        let html = render(&label(
            Some("group-label"),
            vec![],
            vec![text("Choose one")],
        ));
        assert_eq!(
            html,
            r#"<span data-scope="radio-group" data-part="label" id="group-label">Choose one</span>"#
        );
    }

    #[test]
    fn label_id_none_omits_id() {
        let html = render(&label(None, vec![], vec![]));
        assert!(!html.contains(" id="));
    }

    #[test]
    fn item_reflects_checked_state_and_disabled() {
        let checked = render(&item(true, false, "red", vec![], vec![]));
        assert!(checked.contains(r#"data-state="checked""#));
        assert!(checked.contains(r#"data-value="red""#));
        assert!(!checked.contains("data-disabled"));

        let unchecked_disabled = render(&item(false, true, "blue", vec![], vec![]));
        assert!(unchecked_disabled.contains(r#"data-state="unchecked""#));
        assert!(unchecked_disabled.contains(r#"data-value="blue""#));
        assert!(unchecked_disabled.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_control_carries_state_without_radio_role() {
        let html = render(&item_control(true, false, vec![]));
        assert!(html.contains(r#"data-part="item-control""#));
        assert!(html.contains(r#"data-state="checked""#));
        assert!(!html.contains("role=\"radio\""));
        assert!(!html.contains("aria-checked"));
    }

    #[test]
    fn item_text_carries_state_and_children() {
        let html = render(&item_text(false, false, vec![], vec![text("Option A")]));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert!(html.contains("Option A"));
    }

    #[test]
    fn item_hidden_input_is_native_radio_with_presence_attrs() {
        let checked = render(&item_hidden_input(
            true,
            false,
            Some("color"),
            "red",
            vec![],
        ));
        assert!(checked.contains(r#"type="radio""#));
        assert!(checked.contains(r#"name="color""#));
        assert!(checked.contains(r#"value="red""#));
        assert!(checked.contains(r#"checked="""#));
        assert!(!checked.contains("disabled"));

        let unchecked_disabled = render(&item_hidden_input(
            false,
            true,
            Some("color"),
            "blue",
            vec![],
        ));
        assert!(!unchecked_disabled.contains(r#"checked=""#));
        assert!(unchecked_disabled.contains(r#"disabled="""#));
    }

    #[test]
    fn item_hidden_input_name_none_omits_name_attribute() {
        let html = render(&item_hidden_input(false, false, None, "red", vec![]));
        assert!(!html.contains("name="));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_attrs_cannot_override_anatomy_scope_and_part() {
        let html = render(&item(
            true,
            false,
            "red",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="radio-group""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(!html.contains("attacker"));
    }

    // --- root > label + item(item_control + item_text + item_hidden_input) の組み立て ---

    #[test]
    fn full_assembly_label_and_root_id_cross_reference_with_two_items() {
        let node = root(
            false,
            None,
            Some("group-label"),
            vec![],
            vec![
                label(Some("group-label"), vec![], vec![text("Color")]),
                item(
                    true,
                    false,
                    "red",
                    vec![],
                    vec![
                        item_hidden_input(true, false, Some("color"), "red", vec![]),
                        item_control(true, false, vec![]),
                        item_text(true, false, vec![], vec![text("Red")]),
                    ],
                ),
                item(
                    false,
                    false,
                    "blue",
                    vec![],
                    vec![
                        item_hidden_input(false, false, Some("color"), "blue", vec![]),
                        item_control(false, false, vec![]),
                        item_text(false, false, vec![], vec![text("Blue")]),
                    ],
                ),
            ],
        );
        assert_eq!(
            render(&node),
            concat!(
                r#"<div data-scope="radio-group" data-part="root" role="radiogroup" aria-labelledby="group-label">"#,
                r#"<span data-scope="radio-group" data-part="label" id="group-label">Color</span>"#,
                r#"<label data-scope="radio-group" data-part="item" data-state="checked" data-value="red">"#,
                r#"<input data-scope="radio-group" data-part="item-hidden-input" type="radio" value="red" data-state="checked" name="color" checked=""></input>"#,
                r#"<span data-scope="radio-group" data-part="item-control" data-state="checked"></span>"#,
                r#"<span data-scope="radio-group" data-part="item-text" data-state="checked">Red</span>"#,
                r#"</label>"#,
                r#"<label data-scope="radio-group" data-part="item" data-state="unchecked" data-value="blue">"#,
                r#"<input data-scope="radio-group" data-part="item-hidden-input" type="radio" value="blue" data-state="unchecked" name="color"></input>"#,
                r#"<span data-scope="radio-group" data-part="item-control" data-state="unchecked"></span>"#,
                r#"<span data-scope="radio-group" data-part="item-text" data-state="unchecked">Blue</span>"#,
                r#"</label>"#,
                r#"</div>"#,
            )
        );
    }

    // --- RadioGroup: dispatch 統合（single モード、"select" のみ受理） ---

    #[test]
    fn radio_group_default_is_unchecked() {
        let g = RadioGroup::default();
        assert_eq!(g.value(), None);
        assert!(!g.is_checked("red"));
        assert!(!g.is_checked("blue"));
    }

    #[test]
    fn radio_group_dispatch_select_checks_at_most_one_item() {
        let mut g = RadioGroup::default();
        assert!(dispatch(&mut g, "select", "red"));
        assert!(g.is_checked("red"));
        assert!(!g.is_checked("blue"));

        assert!(dispatch(&mut g, "select", "blue"));
        assert!(!g.is_checked("red"));
        assert!(g.is_checked("blue"));
    }

    #[test]
    fn radio_group_dispatch_ignores_toggle_and_deselect_and_unknown_action() {
        let mut g = RadioGroup::default();
        dispatch(&mut g, "select", "red");

        assert!(!dispatch(&mut g, "toggle", "red"));
        assert!(g.is_checked("red"));

        assert!(!dispatch(&mut g, "deselect", ""));
        assert!(g.is_checked("red"));

        assert!(!dispatch(&mut g, "no_such_action", "blue"));
        assert!(g.is_checked("red"));
    }

    #[test]
    fn radio_group_typed_update_deselect_clears_selection() {
        let mut g = RadioGroup::default();
        g.update(SingleSelectAction::Select("red".to_string()));
        assert!(g.is_checked("red"));

        g.update(SingleSelectAction::Deselect);
        assert_eq!(g.value(), None);
    }

    // --- RadioGroup: 利便メソッド経由の描画が状態機械と一致 ---

    #[test]
    fn radio_group_convenience_methods_reflect_state() {
        let mut g = RadioGroup::default();
        dispatch(&mut g, "select", "red");

        let item_red = render(&g.item("red", false, vec![], vec![]));
        assert!(item_red.contains(r#"data-state="checked""#));

        let item_blue = render(&g.item("blue", false, vec![], vec![]));
        assert!(item_blue.contains(r#"data-state="unchecked""#));

        let input_red = render(&g.item_hidden_input("red", false, Some("color"), vec![]));
        assert!(input_red.contains(r#"checked="""#));

        let input_blue = render(&g.item_hidden_input("blue", false, Some("color"), vec![]));
        assert!(!input_blue.contains(r#"checked=""#));
    }

    // --- RadioGroup: SSR 状態なし初期描画 ---

    #[test]
    fn radio_group_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&RadioGroup::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    #[test]
    fn radio_group_view_root_is_element_for_render_for_hydration() {
        // render_for_hydration はルートが Node::Element であることを前提に
        // 属性を合成する（`crates/interactive/src/lib.rs` 参照）。
        let node = RadioGroup::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    // --- RadioGroup: hydration 経路 ---

    #[test]
    fn radio_group_hydration_round_trip_checked() {
        let mut g = RadioGroup::default();
        dispatch(&mut g, "select", "red");
        let rendered = render(&render_for_hydration(&g));
        // codec::encode_list は区切り文字を先頭に付与するエンコードのため、
        // 属性値は選択値そのままの文字列（"red"）とは一致しない。属性が
        // 実際に出力され値に選択値が含まれることのみを確認する。
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("red"));

        let restored = RadioGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }

    #[test]
    fn radio_group_hydration_round_trip_unchecked() {
        let g = RadioGroup::default();
        let restored = RadioGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }

    #[test]
    fn radio_group_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = RadioGroup::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-selected".to_string())
        );
    }

    #[test]
    fn radio_group_from_hydration_attrs_invalid_value_does_not_panic() {
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["red".to_string(), "blue".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = RadioGroup::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰: value/name/id/labelled_by/呼び出し側 attrs/children/dispatch payload/hydration にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn root_labelled_by_payload_is_escaped_on_render() {
        let html = render(&root(false, None, Some(ATTR_BREAK_PAYLOAD), vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn label_id_payload_is_escaped_on_render() {
        let html = render(&label(Some(ATTR_BREAK_PAYLOAD), vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_hidden_input_name_and_value_payload_is_escaped_on_render() {
        let html = render(&item_hidden_input(
            false,
            false,
            Some(ATTR_BREAK_PAYLOAD),
            ATTR_BREAK_PAYLOAD,
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            false,
            None,
            None,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&item_text(
            false,
            false,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn radio_group_dispatch_select_payload_is_escaped_on_render() {
        let mut g = RadioGroup::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut g, "select", payload));

        let rendered = render(&render_for_hydration(&g));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn radio_group_xss_payload_in_hydration_selected_is_rejected_not_rendered() {
        // 改ざん耐性: from_hydration_attrs は未知/不正な値を panic せず拒否する
        // （SingleSelect の既存保証を RadioGroup 経由でも固定する）。
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["<script>alert(1)</script>".to_string(), "b".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = RadioGroup::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

//! SegmentGroup（segmented control）: ark-ui の Segment Group
//!（`.claude/skills/ark-ui/references/components/form/segment-group.md`）/
//! chakra-ui の Segmented Control 相当の headless セグメント UI
//!（イシュー #743、親トラッキング #520）。
//!
//! # `radio_group` への委譲（責務境界、必読）
//!
//! WAI-ARIA 上、segmented control は radio パターン（単一選択・排他制御）
//! そのものである。本モジュールは状態機械・dispatch 契約・hydration の
//! **すべてを [`crate::radio_group::RadioGroup`]（[`crate::state::SingleSelect`]
//! を埋め込んだ既存実装）へ全委譲**し、独自の状態機械を新設しない。
//! 本モジュールが固有に持つのは以下の 2 点のみ:
//!
//! 1. segment 用 anatomy（`data-scope="segment-group"`。ark-ui の Root /
//!    Indicator / Item / ItemText / ItemControl / ItemHiddenInput 6 パーツ）。
//! 2. [`indicator`] の SSR 決定的な位置表現（下記「Indicator の位置表現」節）。
//!
//! [`SegmentGroup::update`]/[`SegmentGroup::decode_action`]/
//! [`SegmentGroup::hydration_attrs`]/[`SegmentGroup::from_hydration_attrs`]
//! はすべて内部の [`crate::radio_group::RadioGroup`] へそのまま委譲する
//! （dispatch `"select"` のみ受理する fail-closed 契約、hydration が panic
//! せず `HydrateError` を返す既存保証を継承する。詳細は
//! `crates/headless-ui/src/radio_group.rs` module doc 参照）。
//!
//! # data-state 語彙（`"checked"`/`"unchecked"`）
//!
//! `radio_group` と同一の値語彙（[`crate::state::DATA_STATE_CHECKED`]/
//! [`crate::state::DATA_STATE_UNCHECKED`]）を [`item`]/[`item_control`]/
//! [`item_text`]/[`item_hidden_input`] へ、[`indicator`] にも
//! （選択有無の表現として）出力する。
//!
//! # ネイティブ semantics
//!
//! [`item_hidden_input`] が生成するネイティブ `<input type="radio">` が
//! チェック状態・フォーム送信・キーボード操作・グループ内排他選択を担う
//! （`radio_group` と同型）。[`item`] は `<label>` を採用し、内包する
//! [`item_hidden_input`] とのクリック委譲が JS なしで成立する。
//! [`item_control`] には `role="radio"`/`aria-checked` を重複付与しない
//! （二重読み上げ防止、`radio_group::item_control` と同じ最小主義）。
//!
//! # Indicator の位置表現（SSR 決定的、JS 計測なし）
//!
//! ark-ui の Indicator は CSR 実測（`getBoundingClientRect` 等）で追従する
//! が、本フレームワークは AI 前提の明示性・決定性を優先し、SSR 静的
//! マークアップのみで位置を表現する。[`indicator`] は選択項目の
//! `(index, item_count)` を受け取り、`style` 属性へ CSS カスタム
//! プロパティ 2 種のみを出力する:
//!
//! ```text
//! --fandhe-segment-group-index: <index>; --fandhe-segment-group-count: <count>;
//! ```
//!
//! 値は `usize` の Display 整形のみから組み立て、ユーザー文字列を CSS 値へ
//! 流し込む経路は存在しない（[`crate::positioning::css_vars_style`] と同型の
//! 安全設計。詳細は [`indicator`] の doc 参照）。等幅セグメントの前提で
//! styled 層（`fandhe-frontend-pre-styled-ui`）がこの 2 変数から
//! `width: calc(100% / var(--fandhe-segment-group-count))` と
//! `transform: translateX(calc(100% * var(--fandhe-segment-group-index)))`
//! （vertical では translateY）を導出する想定。
//!
//! # セキュリティ不変条件
//!
//! 各パーツ関数は属性 Vec を組み立てて [`crate::anatomy::Anatomy::part`]
//! （内部で [`fandhe_frontend_core::el`] を 1 回呼ぶ）へ委譲するのみであり、
//! 独自のエスケープ処理・HTML 文字列直接組み立てを持たない。動的値
//! （`value`/`name`/`labelled_by`/呼び出し側 `attrs`/`children` テキスト/
//! dispatch payload/hydration 属性）は [`fandhe_frontend_core::render`] の
//! 既定エスケープを必ず経由する（REQ-1）。本モジュールは `raw_html()` を
//! 使用しない。[`indicator`] の `style` 属性値は `usize` の整形のみで
//! 合成し、CSS インジェクション経路を作らない。
//!
//! # out-of-scope（本イシュー #743 のスコープ外）
//!
//! - **`fandhe-frontend-wasm-full` の CSR 配線**: `(scope, part) =
//!   ("segment-group", "item") -> "select"` の静的マッピング表追加・
//!   focus_visible 配線・dispatch 後の indicator CSS 変数の動的更新は未着手
//!   （別イシューでの追跡を提案する）。
//! - **矢印キーによる roving tabindex**: SSR 静的マークアップに寄与しない
//!   CSR 挙動層のため未提供（`radio_group` と同じ判断）。
//! - **chakra-ui 拡張の `Label`/`Items` sub-parts**: ark-ui anatomy に存在
//!   しないため採用しない（外部ラベル関連付けは [`root`] の `labelled_by`
//!   で成立させる）。
//! - **`readOnly`・`xs` サイズ**: styled 層（pre-styled-ui）のスコープ外。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_hidden, aria_labelledby, aria_orientation, role};
use crate::data_attrs::{data_disabled, data_orientation, data_state, Orientation};
use crate::radio_group::RadioGroup;
use crate::state::checked_data_state;
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// `data-state` 属性値 "checked"。[`crate::radio_group::DATA_STATE_CHECKED`]
/// の互換 re-export（値語彙は `radio_group` と共有。モジュール doc
/// 「data-state 語彙」節参照）。
pub use crate::state::DATA_STATE_CHECKED;
/// `data-state` 属性値 "unchecked"。[`DATA_STATE_CHECKED`] 参照。
pub use crate::state::DATA_STATE_UNCHECKED;

/// SegmentGroup の anatomy（`data-scope="segment-group"` 固定）。
const ANATOMY: Anatomy = anatomy("segment-group");

/// Root パーツ（`div`、`role="radiogroup"`）。`radio_group::root` と同型の
/// 引数・出力契約（`labelled_by`/`orientation` はいずれも `Some` のときのみ
/// 対応する属性を出力する）。
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

/// Indicator パーツ（`span`、装飾のため `aria-hidden="true"` 固定）。
///
/// `position` が `Some((index, count))` のとき `data-state="checked"` と
/// ともに `style` 属性へ `--fandhe-segment-group-index`/
/// `--fandhe-segment-group-count`（`usize` の Display 整形のみ、モジュール
/// doc「Indicator の位置表現」参照）を出力する。`None`（未選択）のときは
/// `data-state="unchecked"` のみを出力し、`style` 属性は付与しない（styled
/// 層が未選択時にインジケータを非表示にできるようにする）。
///
/// `orientation` が `Some` のときは `data-orientation` も出力し、styled 層が
/// 縦横で `translateX`/`translateY` を切り替えられるようにする（`SlotRecipe`
/// は子孫セレクタを持たないため、`root` ではなく `indicator` 自身の属性で
/// 条件化する必要がある）。
#[must_use]
pub fn indicator<'a>(
    position: Option<(usize, usize)>,
    orientation: Option<Orientation>,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let style: Option<String> = position.map(|(index, count)| {
        format!("--fandhe-segment-group-index: {index}; --fandhe-segment-group-count: {count};")
    });

    // `merged` は `style`（関数ローカルの `String`）由来の借用と `attrs`
    // （呼び出し側 `'a`）由来の借用を混在させるため、`'a` へ明示的に紐付けず
    // 短い局所ライフタイムへ推論させる（`&str` の共変性により `'a: 'local`
    // の要素は自然に混在できる）。`ANATOMY.part` 呼び出しの間だけ生存すれば
    // 十分であり、戻り値の `Node` は所有権を持つため呼び出し後の借用は残らない。
    let mut merged: Vec<(&str, &str)> = vec![
        aria_hidden(true),
        data_state(checked_data_state(position.is_some())),
    ];
    if let Some(orientation) = orientation {
        merged.push(data_orientation(orientation));
    }
    if let Some(style) = &style {
        merged.push(("style", style.as_str()));
    }
    merged.extend(attrs);
    ANATOMY.part("indicator", "span", merged, vec![])
}

/// Item パーツ（`label`）。選択肢 1 個のラップ要素。[`crate::radio_group::item`]
/// と同型（ネイティブ `<label>` によりクリック委譲が JS なしで機能する）。
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

/// ItemControl パーツ（`span`、視覚的な選択枠）。チェック状態のセマンティクス
/// は [`item_hidden_input`] のネイティブ `<input type="radio">` が担うため
/// `role="radio"`/`aria-checked` を付与しない（`radio_group::item_control`
/// と同じ二重読み上げ防止の最小主義）。
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
/// 選択をブラウザのネイティブ semantics に委ねる（`radio_group::item_hidden_input`
/// と同型）。`type="radio"` はリテラル固定。`name`/`value` は動的値だが
/// [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
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

/// 状態機械・dispatch 契約・hydration のすべてを [`RadioGroup`]
/// （[`crate::state::SingleSelect`]）へ全委譲する SegmentGroup（モジュール
/// doc「`radio_group` への委譲」節参照）。本型が固有に持つのは segment
/// anatomy への注入用の利便メソッドのみ。`Default` は未選択（SSR の状態
/// なし初期描画に対応する既定値）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SegmentGroup {
    radio: RadioGroup,
}

impl SegmentGroup {
    /// 現在選択中の項目値（未選択なら `None`）。[`RadioGroup::value`] へ委譲。
    #[must_use]
    pub fn value(&self) -> Option<&str> {
        self.radio.value()
    }

    /// 指定した項目値が選択中かどうか。[`RadioGroup::is_checked`] へ委譲。
    #[must_use]
    pub fn is_checked(&self, value: &str) -> bool {
        self.radio.is_checked(value)
    }

    /// `values` の中で現在選択中の項目の `(index, count)`。未選択、または
    /// 選択値が `values` に含まれない場合は `None`（[`indicator`] へそのまま
    /// 渡せる形。選択値解決は呼び出し側の `values` 順序に依存するため、
    /// `values` は呼び出し側が描画する項目順と一致させる必要がある）。
    #[must_use]
    pub fn indicator_position(&self, values: &[&str]) -> Option<(usize, usize)> {
        let selected = self.value()?;
        let index = values.iter().position(|v| *v == selected)?;
        Some((index, values.len()))
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

    /// [`indicator`] へ `values` から解決した現在の選択位置を注入する利便
    /// メソッド（[`Self::indicator_position`] 参照）。
    #[must_use]
    pub fn indicator<'a>(
        &self,
        values: &[&str],
        orientation: Option<Orientation>,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        indicator(self.indicator_position(values), orientation, attrs)
    }
}

impl Component for SegmentGroup {
    type Action = <RadioGroup as Component>::Action;

    /// [`RadioGroup::update`] へ全委譲（モジュール doc 参照）。
    fn update(&mut self, action: Self::Action) {
        self.radio.update(action);
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（[`root`]、
    /// children 空。[`RadioGroup::view`] と同じ位置付け）。
    fn view(&self) -> Node {
        root(false, None, None, Vec::new(), Vec::new())
    }

    /// [`RadioGroup::decode_action`] へ全委譲（`"select"` のみ受理する
    /// fail-closed 契約をそのまま継承する。モジュール doc 参照）。
    fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
        RadioGroup::decode_action(name, payload)
    }
}

impl Hydrate for SegmentGroup {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.radio.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            radio: RadioGroup::from_hydration_attrs(attrs)?,
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
        assert!(html.contains(r#"data-scope="segment-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="radiogroup""#));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("aria-labelledby"));
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
    fn indicator_some_position_outputs_state_and_css_vars() {
        let html = render(&indicator(Some((1, 3)), None, vec![]));
        assert!(html.contains(r#"data-scope="segment-group""#));
        assert!(html.contains(r#"data-part="indicator""#));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(html.contains(r#"data-state="checked""#));
        assert!(html.contains("--fandhe-segment-group-index: 1;"));
        assert!(html.contains("--fandhe-segment-group-count: 3;"));
    }

    #[test]
    fn indicator_none_position_omits_style_and_is_unchecked() {
        let html = render(&indicator(None, None, vec![]));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert!(!html.contains("style="));
        assert!(!html.contains("--fandhe-segment-group-index"));
    }

    #[test]
    fn indicator_orientation_some_outputs_data_orientation() {
        let html = render(&indicator(
            Some((0, 2)),
            Some(Orientation::Vertical),
            vec![],
        ));
        assert!(html.contains(r#"data-orientation="vertical""#));
    }

    #[test]
    fn item_reflects_checked_state_and_disabled() {
        let checked = render(&item(true, false, "list", vec![], vec![]));
        assert!(checked.contains(r#"data-state="checked""#));
        assert!(checked.contains(r#"data-value="list""#));
        assert!(!checked.contains("data-disabled"));

        let unchecked_disabled = render(&item(false, true, "grid", vec![], vec![]));
        assert!(unchecked_disabled.contains(r#"data-state="unchecked""#));
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
        let html = render(&item_text(false, false, vec![], vec![text("List")]));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert!(html.contains("List"));
    }

    #[test]
    fn item_hidden_input_is_native_radio_with_presence_attrs() {
        let checked = render(&item_hidden_input(
            true,
            false,
            Some("view"),
            "list",
            vec![],
        ));
        assert!(checked.contains(r#"type="radio""#));
        assert!(checked.contains(r#"name="view""#));
        assert!(checked.contains(r#"value="list""#));
        assert!(checked.contains(r#"checked="""#));

        let unchecked_disabled = render(&item_hidden_input(
            false,
            true,
            Some("view"),
            "grid",
            vec![],
        ));
        assert!(!unchecked_disabled.contains(r#"checked=""#));
        assert!(unchecked_disabled.contains(r#"disabled="""#));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_attrs_cannot_override_anatomy_scope_and_part() {
        let html = render(&item(
            true,
            false,
            "list",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="segment-group""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(!html.contains("attacker"));
    }

    // --- root > indicator + item(item_control + item_text + item_hidden_input) の組み立て ---

    #[test]
    fn full_assembly_root_with_indicator_and_two_items() {
        let node = root(
            false,
            None,
            None,
            vec![],
            vec![
                indicator(Some((0, 2)), None, vec![]),
                item(
                    true,
                    false,
                    "list",
                    vec![],
                    vec![
                        item_hidden_input(true, false, Some("view"), "list", vec![]),
                        item_control(true, false, vec![]),
                        item_text(true, false, vec![], vec![text("List")]),
                    ],
                ),
                item(
                    false,
                    false,
                    "grid",
                    vec![],
                    vec![
                        item_hidden_input(false, false, Some("view"), "grid", vec![]),
                        item_control(false, false, vec![]),
                        item_text(false, false, vec![], vec![text("Grid")]),
                    ],
                ),
            ],
        );
        assert_eq!(
            render(&node),
            concat!(
                r#"<div data-scope="segment-group" data-part="root" role="radiogroup">"#,
                r#"<span data-scope="segment-group" data-part="indicator" aria-hidden="true" data-state="checked" style="--fandhe-segment-group-index: 0; --fandhe-segment-group-count: 2;"></span>"#,
                r#"<label data-scope="segment-group" data-part="item" data-state="checked" data-value="list">"#,
                r#"<input data-scope="segment-group" data-part="item-hidden-input" type="radio" value="list" data-state="checked" name="view" checked="">"#,
                r#"<span data-scope="segment-group" data-part="item-control" data-state="checked"></span>"#,
                r#"<span data-scope="segment-group" data-part="item-text" data-state="checked">List</span>"#,
                r#"</label>"#,
                r#"<label data-scope="segment-group" data-part="item" data-state="unchecked" data-value="grid">"#,
                r#"<input data-scope="segment-group" data-part="item-hidden-input" type="radio" value="grid" data-state="unchecked" name="view">"#,
                r#"<span data-scope="segment-group" data-part="item-control" data-state="unchecked"></span>"#,
                r#"<span data-scope="segment-group" data-part="item-text" data-state="unchecked">Grid</span>"#,
                r#"</label>"#,
                r#"</div>"#,
            )
        );
    }

    // --- SegmentGroup: dispatch 統合（radio_group への委譲） ---

    #[test]
    fn segment_group_default_is_unchecked() {
        let g = SegmentGroup::default();
        assert_eq!(g.value(), None);
        assert!(!g.is_checked("list"));
    }

    #[test]
    fn segment_group_dispatch_select_checks_at_most_one_item() {
        let mut g = SegmentGroup::default();
        assert!(dispatch(&mut g, "select", "list"));
        assert!(g.is_checked("list"));
        assert!(!g.is_checked("grid"));

        assert!(dispatch(&mut g, "select", "grid"));
        assert!(!g.is_checked("list"));
        assert!(g.is_checked("grid"));
    }

    #[test]
    fn segment_group_dispatch_ignores_toggle_and_deselect_and_unknown_action() {
        let mut g = SegmentGroup::default();
        dispatch(&mut g, "select", "list");

        assert!(!dispatch(&mut g, "toggle", "list"));
        assert!(g.is_checked("list"));

        assert!(!dispatch(&mut g, "deselect", ""));
        assert!(g.is_checked("list"));

        assert!(!dispatch(&mut g, "no_such_action", "grid"));
        assert!(g.is_checked("list"));
    }

    // --- SegmentGroup: 利便メソッド経由の描画が状態機械と一致 ---

    #[test]
    fn segment_group_convenience_methods_reflect_state() {
        let mut g = SegmentGroup::default();
        dispatch(&mut g, "select", "list");

        let item_list = render(&g.item("list", false, vec![], vec![]));
        assert!(item_list.contains(r#"data-state="checked""#));

        let item_grid = render(&g.item("grid", false, vec![], vec![]));
        assert!(item_grid.contains(r#"data-state="unchecked""#));

        let input_list = render(&g.item_hidden_input("list", false, Some("view"), vec![]));
        assert!(input_list.contains(r#"checked="""#));
    }

    #[test]
    fn segment_group_indicator_position_resolves_selected_index() {
        let mut g = SegmentGroup::default();
        assert_eq!(g.indicator_position(&["list", "grid", "table"]), None);

        dispatch(&mut g, "select", "grid");
        assert_eq!(
            g.indicator_position(&["list", "grid", "table"]),
            Some((1, 3))
        );
    }

    #[test]
    fn segment_group_indicator_convenience_method_reflects_position() {
        let mut g = SegmentGroup::default();
        dispatch(&mut g, "select", "grid");

        let html = render(&g.indicator(&["list", "grid"], None, vec![]));
        assert!(html.contains("--fandhe-segment-group-index: 1;"));
        assert!(html.contains("--fandhe-segment-group-count: 2;"));
    }

    #[test]
    fn segment_group_indicator_position_none_when_selected_value_not_in_values() {
        let mut g = SegmentGroup::default();
        dispatch(&mut g, "select", "unknown-value");
        assert_eq!(g.indicator_position(&["list", "grid"]), None);
    }

    // --- SegmentGroup: SSR 状態なし初期描画・hydration 経路 ---

    #[test]
    fn segment_group_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&SegmentGroup::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    #[test]
    fn segment_group_hydration_round_trip_checked() {
        let mut g = SegmentGroup::default();
        dispatch(&mut g, "select", "list");
        let rendered = render(&render_for_hydration(&g));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("list"));

        let restored = SegmentGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }

    #[test]
    fn segment_group_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = SegmentGroup::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-selected".to_string())
        );
    }

    // --- XSS 回帰: value/name/labelled_by/呼び出し側 attrs/children/dispatch payload/hydration へのペイロード ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn root_labelled_by_payload_is_escaped_on_render() {
        let html = render(&root(false, None, Some(ATTR_BREAK_PAYLOAD), vec![], vec![]));
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
    fn segment_group_dispatch_select_payload_is_escaped_on_render() {
        let mut g = SegmentGroup::default();
        let payload = "\"><script>alert(1)</script>";
        assert!(dispatch(&mut g, "select", payload));

        let rendered = render(&render_for_hydration(&g));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(!rendered.contains(r#""><script"#));
    }

    #[test]
    fn segment_group_xss_payload_in_hydration_selected_is_rejected_not_rendered() {
        // 改ざん耐性: from_hydration_attrs は未知/不正な値を panic せず拒否する
        // （SingleSelect/RadioGroup の既存保証を SegmentGroup 経由でも固定する）。
        use fandhe_frontend_interactive::codec;
        let bogus = codec::encode_list(&["<script>alert(1)</script>".to_string(), "b".to_string()]);
        let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
        let err = SegmentGroup::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

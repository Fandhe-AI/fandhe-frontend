//! Tabs コンポーネント（イシュー #528、Phase 2 #526〜#544 の一部）。
//!
//! WAI-ARIA APG の Tabs パターン（`role="tablist"`/`"tab"`/`"tabpanel"`・
//! `aria-selected`・`aria-controls`/`aria-labelledby` の相互参照・roving
//! tabindex）に準拠したマークアップを、[`fandhe_frontend_core::Node`] 木として
//! 組み立てる。anatomy は ark-ui 相当の `root` / `list` / `trigger` / `content` /
//! `indicator`（#601、[`TabsProps::indicator`] で opt-in）の 5 パーツ構成。
//! `indicator` は選択タブの位置を示す装飾パーツで、Phase 1 の positioner
//! （`crate::popover` の判断: 位置決めロジック＝計測は SSR の責務外）と同じ
//! 整理に基づき、SSR では `data-*` フックと CSS 変数の**初期値**のみを出力する
//! （動的計測・`--transition-*` 系は wasm/CSR 層の後続責務）。
//!
//! # 呼び出し文脈
//!
//! - 上層の [`crate::anatomy::Anatomy`]（#523）・[`crate::aria`]（#523）・
//!   [`crate::data_attrs`]（#523）へ薄く委譲するのみで、独自の出力経路・
//!   独自のエスケープ処理は持たない。
//! - クリック/キーボード操作の実挙動・[`crate::state::SingleSelect`] との
//!   状態機械連携は本イシューのスコープ外（後続イシュー・wasm 層の責務）。
//!   [`tabs`] は SSR 時点の静的な選択状態（[`TabsProps::selected`]）から
//!   決定的にマークアップを組み立てるのみである。
//! - styled 層（`fandhe-frontend-pre-styled-ui`、#546）は本モジュールが
//!   出力する `data-scope="tabs"`/`data-part="..."` セレクタを前提にスタイルを
//!   当てる。
//!
//! # セキュリティ不変条件
//!
//! - `value`/`id`/ラベル等の動的値はすべて [`fandhe_frontend_core::el`] の
//!   属性値・子ノードとして渡り、[`fandhe_frontend_core::render`] の既定
//!   エスケープ（REQ-1）を必ず経由する。本モジュールは `raw_html()` を
//!   使用しない。
//! - 属性名はすべて `&'static str` リテラルで固定されており、動的値が
//!   属性名スロットへ混入する経路はない。
//! - `id` 属性値は `format!` で組み立てるが、これは属性値という**データ**の
//!   組み立てであり、`docs/api/component-api.md`・`.claude/rules/coding-rust.md`
//!   が禁止する「HTML 文字列の直接組み立て」（`format!("<div>{}</div>", ..)`
//!   のようなマークアップ自体の文字列化）ではない。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{
    aria_controls, aria_disabled, aria_hidden, aria_labelledby, aria_orientation, aria_selected,
    role,
};
use crate::data_attrs::{data_disabled, data_orientation, data_state, Orientation};
use fandhe_frontend_core::Node;
use std::collections::HashMap;

/// `data-scope="tabs"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("tabs");

/// `data-state` 属性値 "active"（選択中の trigger/content が持つ値）。
const DATA_STATE_ACTIVE: &str = "active";
/// `data-state` 属性値 "inactive"（非選択の trigger/content が持つ値）。
const DATA_STATE_INACTIVE: &str = "inactive";

/// `indicator` パーツの `style` 属性の初期値（#601）。
///
/// ark-ui / Zag.js の Tabs Indicator が公開する CSS 変数と同名
/// （`--left`/`--top`/`--width`/`--height`）で、styled 層・利用者 CSS が
/// `var(--left)` 等でそのまま参照できるようにする。値は `0px` 固定の
/// `&'static str` リテラルであり、`format!` を経由しない（動的値の混入経路
/// ゼロ）。実測に基づく動的な位置・サイズの反映（Zag の
/// `setIndicatorRect` 相当）は wasm/CSR 層の後続責務であり、本モジュールは
/// SSR 時点の決定的な初期値のみを出力する。
const INDICATOR_STYLE_INITIAL: &str = "--left: 0px; --top: 0px; --width: 0px; --height: 0px";

/// タブ活性化のタイミング(WAI-ARIA APG Tabs パターンの `automatic`/`manual`
/// activation の区別)。SSR 出力(`data-activation-mode`)としては本 enum の
/// 固定 2 値のみを語彙とし、`crates/wasm-full/src/keynav.rs`(イシュー #582)が
/// この属性を読んでキーボード操作時の挙動を分岐する契約となる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActivationMode {
    /// フォーカス移動と同時にタブを活性化する(既定)。
    #[default]
    Automatic,
    /// フォーカス移動のみを行い、Enter/Space(ネイティブ button の click)で
    /// 活性化する。
    Manual,
}

impl ActivationMode {
    /// `data-activation-mode` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Manual => "manual",
        }
    }
}

/// タブ 1 枚の定義（trigger ラベルと対応する content パネル）。
///
/// `value` は同一 [`tabs`] 呼び出し内で一意であることが呼び出し側の契約
/// である（[`TabsProps::id`] と組み合わせて trigger/content の `id` を
/// 決定的に生成するため）。重複した場合の挙動は [`tabs`] の rustdoc を
/// 参照（選択判定は先勝ち・fail-closed で panic しない。`id` の一意性は
/// 2 件目以降の重複 `value` に出現順インデックスを付与して機械的に確保する。
/// 呼び出し側は `value` を一意にすることが望ましい契約であることに変わりはない）。
pub struct TabItem<'a> {
    /// タブの識別 value。
    pub value: &'a str,
    /// trigger（タブボタン）の子ノード。
    pub trigger: Vec<Node>,
    /// content（タブパネル）の子ノード。
    pub content: Vec<Node>,
    /// 無効タブ。`true` のとき `disabled` 属性・`data-disabled`・
    /// `aria-disabled="true"` を trigger に付与し、roving tabindex の
    /// フォールバック候補からも除外する。
    pub disabled: bool,
}

/// [`tabs`] 全体の設定。
pub struct TabsProps<'a> {
    /// ベース id。trigger/content の決定的 id 生成
    /// （`"{id}-trigger-{value}"`/`"{id}-content-{value}"`）に使う。
    pub id: &'a str,
    /// 選択中タブの value（SSR 時点の静的選択状態）。
    ///
    /// どの [`TabItem::value`] とも一致しない場合は「全タブ非選択」として
    /// 描画する（[`tabs`] の rustdoc 参照）。
    pub selected: &'a str,
    /// 向き。`data-orientation`（root/list/trigger/content 共通）・
    /// list の `aria-orientation` の双方に反映する。
    pub orientation: Orientation,
    /// タブ活性化のタイミング。`list` パーツへ `data-activation-mode` として
    /// 出力し、`crates/wasm-full/src/keynav.rs`（イシュー #582）がキーボード
    /// 操作時の活性化挙動（automatic: フォーカス移動と同時に活性化 / manual:
    /// Enter・Space で活性化）を分岐するために読む。
    pub activation_mode: ActivationMode,
    /// roving tabindex のフォーカス循環（Arrow キーで端から反対端へ移動する
    /// か）。`list` パーツへ `data-loop-focus`（`"true"`/`"false"`）として
    /// 出力する。ark-ui の既定に合わせ `true` を既定値とする
    /// （[`Default`] 実装は持たないため、呼び出し側が明示的に指定する）。
    pub loop_focus: bool,
    /// `indicator` パーツ（選択タブの位置を示す装飾要素）を出力するかどうか
    /// （#601、既定 `false` で既存出力を変えない opt-in）。
    ///
    /// `true` の場合、`list` の最終子として `data-part="indicator"` の
    /// `<span>` を追加する。`data-state`（active タブがあれば `"active"`、
    /// なければ `"inactive"`）・`data-orientation`・`aria-hidden="true"`
    /// （装飾要素のため a11y ツリーから除外し、`role="tablist"` の子は
    /// tab のみという APG の前提を壊さない）・`style` 属性（CSS 変数の
    /// 初期値、`INDICATOR_STYLE_INITIAL`）を持つ。inactive な場合は
    /// 指す対象がないため `hidden` を付与する（hydration 前に誤表示
    /// しないための fail-safe）。動的な位置・サイズの計測
    /// （Zag の `setIndicatorRect` 相当）は wasm/CSR 層の後続責務であり、
    /// 本パーツは SSR 時点の決定的な初期値のみを出力する（Phase 1 の
    /// positioner、`crate::popover` と同じ整理）。
    pub indicator: bool,
}

/// Tabs 全体を 1 つの [`Node`] 木として組み立てる。
///
/// anatomy: `root`（div）> `list`（div, `role="tablist"`）> `trigger`
/// （button, `role="tab"`）* と、`root` の子として `content`（div,
/// `role="tabpanel"`）* が並ぶ（`list` の外側の兄弟）。
///
/// # 選択状態の決定則（panic せず決定的に描画する）
///
/// - `props.selected` と一致する最初の [`TabItem::value`] のみを active と
///   する。2 件目以降の重複 `value` はすべて inactive として描画する
///   （先勝ち）。
/// - `id` の一意性（REQ 相当、レビュー指摘 PR #560）: 重複 `value` を持つ
///   複数の [`TabItem`] がそのまま `"{id}-trigger-{value}"` から `id` を
///   導出すると HTML 上で `id` が衝突し、`aria-controls`/`aria-labelledby`
///   の参照先が曖昧になる。これを防ぐため、同一 `value` の 2 件目以降には
///   その `value` 内での出現順インデックス（0 始まり、初回は付与しない）を
///   `"{id}-trigger-{value}-{n}"`/`"{id}-content-{value}-{n}"` として
///   付与し、`id` を一意に保つ（描画自体は先勝ち・fail-closed のまま拒否
///   しない）。
/// - `props.selected` がどの `value` とも一致しない場合、全 trigger/content
///   が inactive として描画される。
/// - `props.selected` が一致した item が disabled の場合も同様に「未選択」
///   として扱い、全 trigger/content が inactive（`aria-selected="false"`・
///   `data-state="inactive"`・全 panel `hidden`）として描画される。disabled
///   item を active のままにすると、パネルは表示され続けるのに roving
///   tabindex は別の trigger へ移り、selected-tab ↔ visible-panel ↔
///   tabbable-tab の対応（WAI-ARIA APG）が崩れるため（レビュー指摘、PR #560）。
/// - roving tabindex（WAI-ARIA APG: tablist 内に常に `tabindex="0"` を
///   ちょうど 1 つ）は、active な trigger（上記の通り disabled では
///   あり得ない）があればそれに `tabindex="0"` を与える。active がない
///   場合は最初の非 disabled trigger に与える。全 trigger が disabled、
///   または `items` が空の場合は誰にも `0` を与えない（全て `-1`、あるいは
///   trigger 自体が存在しない）。
/// - `items` が空の場合は `root`/`list` のみを描画する（panic しない）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_headless_ui::data_attrs::Orientation;
/// use fandhe_frontend_headless_ui::{tabs, TabItem, TabsProps};
///
/// let node = tabs(
///     &TabsProps {
///         id: "t",
///         selected: "a",
///         orientation: Orientation::Horizontal,
///         activation_mode: fandhe_frontend_headless_ui::tabs::ActivationMode::Automatic,
///         loop_focus: true,
///         indicator: false,
///     },
///     vec![
///         TabItem { value: "a", trigger: vec![text("A")], content: vec![text("panel A")], disabled: false },
///         TabItem { value: "b", trigger: vec![text("B")], content: vec![text("panel B")], disabled: false },
///     ],
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"aria-selected="true""#));
/// assert!(html.contains(r#"id="t-trigger-a""#));
/// assert!(html.contains(r#"aria-controls="t-content-a""#));
/// ```
#[must_use]
pub fn tabs(props: &TabsProps<'_>, items: Vec<TabItem<'_>>) -> Node {
    let data_orientation_attr = data_orientation(props.orientation);
    let aria_orientation_attr = aria_orientation(props.orientation);

    // 選択判定: value が一致する最初の item のみ active（先勝ち、fail-closed）。
    // ただしその item が disabled の場合は「未選択」として扱う（レビュー指摘、PR #560）:
    // disabled item を selected のまま active/aria-selected="true" にすると、パネルは
    // 表示され続けるのに roving tabindex="0" は別の非 disabled trigger へ移る
    // （selected-tab ↔ visible-panel ↔ tabbable-tab の対応が崩れ、Tab 移動で到達する
    // trigger と表示中のパネルが食い違う）。selected が unmatched のケースと同様に
    // 「全 trigger/panel が inactive」として決定的に描画することで、この不整合を防ぐ。
    let active_index = items
        .iter()
        .position(|item| item.value == props.selected)
        .filter(|&index| !items[index].disabled);

    // roving tabindex: active（かつ非 disabled、上記フィルタ済み）があればそれが 0。
    // active が無い場合（selected が unmatched、または selected が disabled item を指す
    // ケース）は最初の非 disabled item にフォールバックする。
    // 該当なし（items 空・全 disabled）なら誰にも 0 を与えない。
    let tabbable_index = active_index.or_else(|| items.iter().position(|item| !item.disabled));

    let mut list_children: Vec<Node> = Vec::with_capacity(items.len());
    let mut root_extra_children: Vec<Node> = Vec::with_capacity(items.len());
    // `value` の出現回数を追跡し、重複 `value` の 2 件目以降に一意化サフィックスを
    // 付与するために使う（レビュー指摘、PR #560）。初回出現（0 件目）はサフィックス
    // なしのまま既存の `id` 形式（"{id}-trigger-{value}"）を維持し、後方互換を保つ。
    let mut value_occurrence: HashMap<&str, u32> = HashMap::with_capacity(items.len());

    for (index, item) in items.into_iter().enumerate() {
        let is_active = active_index == Some(index);
        let is_tabbable = tabbable_index == Some(index);
        let occurrence = value_occurrence.entry(item.value).or_insert(0);
        let occurrence_index = *occurrence;
        *occurrence += 1;
        let (trigger_id, content_id) = if occurrence_index == 0 {
            (
                format!("{}-trigger-{}", props.id, item.value),
                format!("{}-content-{}", props.id, item.value),
            )
        } else {
            // 同一 value の重複衝突を避けるため出現順インデックスを付与する。
            (
                format!("{}-trigger-{}-{}", props.id, item.value, occurrence_index),
                format!("{}-content-{}-{}", props.id, item.value, occurrence_index),
            )
        };
        let data_state_value = if is_active {
            DATA_STATE_ACTIVE
        } else {
            DATA_STATE_INACTIVE
        };
        let tabindex_value: &str = if is_tabbable { "0" } else { "-1" };

        let mut trigger_attrs: Vec<(&str, &str)> = vec![
            ("type", "button"),
            ("id", trigger_id.as_str()),
            role("tab"),
            aria_selected(is_active),
            aria_controls(content_id.as_str()),
            data_state(data_state_value),
            data_orientation_attr,
            ("tabindex", tabindex_value),
            // イシュー #580: `fandhe-frontend-wasm-full` の headless 配線基盤
            // （`wasm-full/src/headless.rs`）が `(scope, part) = ("tabs", "trigger")`
            // クリックを `"select"` アクションへ写像する際の payload 源として
            // `data-value` を参照する。動的値だが `ANATOMY.part` 経由で
            // `render()` の既定エスケープを必ず経由する（REQ-1）。
            ("data-value", item.value),
        ];
        if item.disabled {
            trigger_attrs.push(("disabled", ""));
            trigger_attrs.extend(data_disabled(true));
            trigger_attrs.push(aria_disabled(true));
        }
        list_children.push(ANATOMY.part("trigger", "button", trigger_attrs, item.trigger));

        let mut content_attrs: Vec<(&str, &str)> = vec![
            ("id", content_id.as_str()),
            role("tabpanel"),
            aria_labelledby(trigger_id.as_str()),
            data_state(data_state_value),
            data_orientation_attr,
            ("tabindex", "0"),
        ];
        if !is_active {
            content_attrs.push(("hidden", ""));
        }
        root_extra_children.push(ANATOMY.part("content", "div", content_attrs, item.content));
    }

    if props.indicator {
        // indicator は選択タブの位置を示す装飾パーツ（#601）。装飾要素のため
        // aria-hidden="true" で a11y ツリーから除外し、role="tablist" の子は
        // tab のみという APG の前提を壊さない。data-state は「active タブが
        // 存在するか」で決まる（個々の item の active_index とは独立。indicator
        // は選択状態そのものではなく「示す対象があるか」を表す）。
        let indicator_data_state = if active_index.is_some() {
            DATA_STATE_ACTIVE
        } else {
            DATA_STATE_INACTIVE
        };
        let mut indicator_attrs: Vec<(&str, &str)> = vec![
            data_state(indicator_data_state),
            data_orientation_attr,
            aria_hidden(true),
        ];
        if active_index.is_none() {
            // 指す対象（active タブ）がない場合、hydration 前に位置不定の
            // indicator が誤って表示され続けないための fail-safe。
            indicator_attrs.push(("hidden", ""));
        }
        indicator_attrs.push(("style", INDICATOR_STYLE_INITIAL));
        list_children.push(ANATOMY.part("indicator", "span", indicator_attrs, vec![]));
    }

    let loop_focus_value: &str = if props.loop_focus { "true" } else { "false" };
    let list_attrs: Vec<(&str, &str)> = vec![
        role("tablist"),
        aria_orientation_attr,
        data_orientation_attr,
        ("data-activation-mode", props.activation_mode.as_str()),
        ("data-loop-focus", loop_focus_value),
    ];
    let list_node = ANATOMY.part("list", "div", list_attrs, list_children);

    let mut root_children = Vec::with_capacity(1 + root_extra_children.len());
    root_children.push(list_node);
    root_children.extend(root_extra_children);

    let root_attrs: Vec<(&str, &str)> = vec![("id", props.id), data_orientation_attr];
    ANATOMY.part("root", "div", root_attrs, root_children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    fn item<'a>(value: &'a str, disabled: bool) -> TabItem<'a> {
        TabItem {
            value,
            trigger: vec![text(value)],
            content: vec![text(value)],
            disabled,
        }
    }

    fn props<'a>(id: &'a str, selected: &'a str) -> TabsProps<'a> {
        TabsProps {
            id,
            selected,
            orientation: Orientation::Horizontal,
            activation_mode: ActivationMode::Automatic,
            loop_focus: true,
            indicator: false,
        }
    }

    #[test]
    fn two_tabs_first_selected_full_html_snapshot() {
        let node = tabs(&props("t", "a"), vec![item("a", false), item("b", false)]);
        assert_eq!(
            render(&node),
            concat!(
                r#"<div data-scope="tabs" data-part="root" id="t" data-orientation="horizontal">"#,
                r#"<div data-scope="tabs" data-part="list" role="tablist" aria-orientation="horizontal" data-orientation="horizontal" data-activation-mode="automatic" data-loop-focus="true">"#,
                r#"<button data-scope="tabs" data-part="trigger" type="button" id="t-trigger-a" role="tab" aria-selected="true" aria-controls="t-content-a" data-state="active" data-orientation="horizontal" tabindex="0" data-value="a">a</button>"#,
                r#"<button data-scope="tabs" data-part="trigger" type="button" id="t-trigger-b" role="tab" aria-selected="false" aria-controls="t-content-b" data-state="inactive" data-orientation="horizontal" tabindex="-1" data-value="b">b</button>"#,
                r#"</div>"#,
                r#"<div data-scope="tabs" data-part="content" id="t-content-a" role="tabpanel" aria-labelledby="t-trigger-a" data-state="active" data-orientation="horizontal" tabindex="0">a</div>"#,
                r#"<div data-scope="tabs" data-part="content" id="t-content-b" role="tabpanel" aria-labelledby="t-trigger-b" data-state="inactive" data-orientation="horizontal" tabindex="0" hidden="">b</div>"#,
                r#"</div>"#,
            )
        );
    }

    #[test]
    fn selected_trigger_has_true_active_zero_others_have_false_inactive_minus_one() {
        let node = tabs(&props("t", "a"), vec![item("a", false), item("b", false)]);
        let html = render(&node);
        assert!(html.contains(r#"aria-selected="true""#));
        assert!(html.contains(r#"data-state="active""#));
        assert!(html.contains(r#"aria-selected="false""#));
        assert!(html.contains(r#"data-state="inactive""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(html.contains(r#"tabindex="-1""#));
    }

    #[test]
    fn trigger_and_content_id_cross_reference() {
        let node = tabs(&props("t", "a"), vec![item("a", false), item("b", false)]);
        let html = render(&node);
        assert!(html.contains(r#"id="t-trigger-a""#));
        assert!(html.contains(r#"aria-controls="t-content-a""#));
        assert!(html.contains(r#"id="t-content-a""#));
        assert!(html.contains(r#"aria-labelledby="t-trigger-a""#));
    }

    #[test]
    fn inactive_content_is_hidden_active_content_is_not() {
        let node = tabs(&props("t", "a"), vec![item("a", false), item("b", false)]);
        let html = render(&node);
        assert!(html.contains(
            r#"id="t-content-b" role="tabpanel" aria-labelledby="t-trigger-b" data-state="inactive" data-orientation="horizontal" tabindex="0" hidden="""#
        ));
        assert!(!html.contains(
            r#"id="t-content-a" role="tabpanel" aria-labelledby="t-trigger-a" data-state="active" data-orientation="horizontal" tabindex="0" hidden"#
        ));
    }

    #[test]
    fn disabled_item_gets_disabled_data_disabled_aria_disabled_and_tabindex_minus_one() {
        let node = tabs(&props("t", "a"), vec![item("a", false), item("b", true)]);
        let html = render(&node);
        assert!(html.contains(
            r#"id="t-trigger-b" role="tab" aria-selected="false" aria-controls="t-content-b" data-state="inactive" data-orientation="horizontal" tabindex="-1" data-value="b" disabled="" data-disabled="" aria-disabled="true""#
        ));
    }

    #[test]
    fn selected_matching_disabled_item_is_treated_as_unselected() {
        // props.selected が disabled item の value と一致するケース（PR #560 レビュー指摘）:
        // disabled item を active のままにすると、パネルは表示され続けるのに
        // tabindex="0" は別の trigger へ移り、selected-tab ↔ visible-panel ↔
        // tabbable-tab の対応が崩れる。そのため「未選択」（全 inactive・全 panel
        // hidden）として決定的に描画する。
        let node = tabs(&props("t", "a"), vec![item("a", true), item("b", false)]);
        let html = render(&node);
        // a は disabled のため未選択扱い: aria-selected="false"・data-state="inactive"・tabindex="-1"。
        assert!(html.contains(
            r#"id="t-trigger-a" role="tab" aria-selected="false" aria-controls="t-content-a" data-state="inactive" data-orientation="horizontal" tabindex="-1" data-value="a" disabled="" data-disabled="" aria-disabled="true""#
        ));
        // b は最初の非 disabled item なので tabindex="0" を得る（inactive のまま）。
        assert!(html.contains(
            r#"id="t-trigger-b" role="tab" aria-selected="false" aria-controls="t-content-b" data-state="inactive" data-orientation="horizontal" tabindex="0" data-value="b""#
        ));
        // どの trigger も active でないため、両方の panel が hidden
        // （表示中パネルに対応する trigger が unreachable になる状態を防ぐ）。
        assert_eq!(html.matches(r#"hidden="""#).count(), 2);
        assert!(!html.contains(r#"aria-selected="true""#));
    }

    #[test]
    fn trigger_outputs_data_value_matching_item_value() {
        // イシュー #580: `fandhe-frontend-wasm-full` の headless 配線基盤が
        // `(scope, part) = ("tabs", "trigger")` クリックの select payload 源として
        // `data-value` を参照する契約を固定する回帰テスト。
        let node = tabs(&props("t", "a"), vec![item("a", false), item("b", false)]);
        let html = render(&node);
        assert!(html.contains(r#"data-value="a""#));
        assert!(html.contains(r#"data-value="b""#));
    }

    #[test]
    fn trigger_data_value_payload_is_escaped_on_render() {
        let payload_value = "\"><script>alert(1)</script>";
        let node = tabs(
            &props("t", "unmatched"),
            vec![TabItem {
                value: payload_value,
                trigger: vec![text("t")],
                content: vec![text("c")],
                disabled: false,
            }],
        );
        let html = render(&node);
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn vertical_orientation_reflects_in_aria_and_data_orientation() {
        let node = tabs(
            &TabsProps {
                id: "t",
                selected: "a",
                orientation: Orientation::Vertical,
                activation_mode: ActivationMode::Automatic,
                loop_focus: true,
                indicator: false,
            },
            vec![item("a", false)],
        );
        let html = render(&node);
        assert!(html.contains(r#"aria-orientation="vertical""#));
        // list/trigger/content/root いずれも data-orientation="vertical" を持つ。
        assert_eq!(html.matches(r#"data-orientation="vertical""#).count(), 4);
    }

    #[test]
    fn selected_not_matching_any_value_makes_all_inactive_and_first_enabled_gets_tabindex_zero() {
        let node = tabs(
            &props("t", "does-not-exist"),
            vec![item("a", false), item("b", false)],
        );
        let html = render(&node);
        assert!(!html.contains(r#"aria-selected="true""#));
        assert!(!html.contains(r#"data-state="active""#));
        // a が最初の非 disabled item なので tabindex="0" を得る。
        assert!(html.contains(
            r#"id="t-trigger-a" role="tab" aria-selected="false" aria-controls="t-content-a" data-state="inactive" data-orientation="horizontal" tabindex="0" data-value="a""#
        ));
        assert!(html.contains(
            r#"id="t-trigger-b" role="tab" aria-selected="false" aria-controls="t-content-b" data-state="inactive" data-orientation="horizontal" tabindex="-1" data-value="b""#
        ));
        // 全 content が hidden。
        assert_eq!(html.matches(r#"hidden="""#).count(), 2);
    }

    #[test]
    fn selected_not_matching_any_value_and_all_disabled_gives_no_tabbable_trigger() {
        let node = tabs(
            &props("t", "does-not-exist"),
            vec![item("a", true), item("b", true)],
        );
        let html = render(&node);
        assert_eq!(html.matches(r#"tabindex="0""#).count(), 2); // content 2 件分のみ（trigger 分は 0 件）
        assert_eq!(html.matches(r#"tabindex="-1""#).count(), 2);
    }

    #[test]
    fn duplicate_value_first_wins_second_is_inactive() {
        let node = tabs(&props("t", "a"), vec![item("a", false), item("a", false)]);
        let html = render(&node);
        assert_eq!(html.matches(r#"data-state="active""#).count(), 2); // trigger 1 + content 1 のみ（2 件目の重複 item は inactive）
        assert_eq!(html.matches(r#"aria-selected="true""#).count(), 1);
        assert_eq!(html.matches(r#"aria-selected="false""#).count(), 1);
    }

    #[test]
    fn duplicate_value_gets_unique_ids_via_occurrence_suffix() {
        // レビュー指摘（PR #560, Cursor Bugbot）: 重複 value を持つ TabItem が
        // 同一 id を生成すると aria-controls/aria-labelledby の参照先が曖昧になる。
        // 2 件目以降には出現順インデックスを付与して id を一意に保つ。
        let node = tabs(&props("t", "a"), vec![item("a", false), item("a", false)]);
        let html = render(&node);
        // 1 件目（初回出現）は従来どおりサフィックスなし。
        assert!(html.contains(r#"id="t-trigger-a""#));
        assert!(html.contains(r#"id="t-content-a""#));
        // 2 件目（重複）は出現順インデックス "-1" が付与され、id が一意になる。
        assert!(html.contains(r#"id="t-trigger-a-1""#));
        assert!(html.contains(r#"id="t-content-a-1""#));
        // 2 件目の aria-controls/aria-labelledby も一意化後の id を正しく参照する。
        assert!(html.contains(r#"aria-controls="t-content-a-1""#));
        assert!(html.contains(r#"aria-labelledby="t-trigger-a-1""#));
        // "id=" の総出現回数がちょうど 5 件（root 1 + trigger 2 + content 2）で、
        // 衝突（同一 id 文字列の重複出現）がないことを裏付ける。
        assert_eq!(html.matches(" id=\"").count(), 5);
    }

    #[test]
    fn triple_duplicate_value_gets_sequential_occurrence_suffixes() {
        let node = tabs(
            &props("t", "does-not-exist"),
            vec![item("a", false), item("a", false), item("a", false)],
        );
        let html = render(&node);
        assert!(html.contains(r#"id="t-trigger-a""#));
        assert!(html.contains(r#"id="t-trigger-a-1""#));
        assert!(html.contains(r#"id="t-trigger-a-2""#));
        assert!(html.contains(r#"id="t-content-a""#));
        assert!(html.contains(r#"id="t-content-a-1""#));
        assert!(html.contains(r#"id="t-content-a-2""#));
    }

    #[test]
    fn empty_items_renders_root_and_list_only_without_panicking() {
        let node = tabs(&props("t", "a"), vec![]);
        assert_eq!(
            render(&node),
            concat!(
                r#"<div data-scope="tabs" data-part="root" id="t" data-orientation="horizontal">"#,
                r#"<div data-scope="tabs" data-part="list" role="tablist" aria-orientation="horizontal" data-orientation="horizontal" data-activation-mode="automatic" data-loop-focus="true"></div>"#,
                r#"</div>"#,
            )
        );
    }

    // --- XSS 回帰: value/id/子ノードに攻撃者制御文字列が入っても既定エスケープが効く ---

    #[test]
    fn xss_payload_in_value_and_children_is_escaped_on_render() {
        let payload_value = "x\" onmouseover=\"alert(1)";
        let node = tabs(
            &props("t", payload_value),
            vec![TabItem {
                value: payload_value,
                trigger: vec![text("<script>alert(1)</script>")],
                content: vec![text("<script>alert(2)</script>")],
                disabled: false,
            }],
        );
        let html = render(&node);
        assert!(!html.contains("<script>alert"));
        assert!(!html.contains("onmouseover=\"alert"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"));
        assert!(html.contains("&quot;"));
    }

    // --- イシュー #582: activation_mode/loop_focus の SSR 属性出力 ---

    #[test]
    fn manual_activation_mode_and_loop_focus_false_are_reflected_in_list_attrs() {
        let node = tabs(
            &TabsProps {
                id: "t",
                selected: "a",
                orientation: Orientation::Horizontal,
                activation_mode: ActivationMode::Manual,
                loop_focus: false,
                indicator: false,
            },
            vec![item("a", false), item("b", false)],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-activation-mode="manual""#));
        assert!(html.contains(r#"data-loop-focus="false""#));
    }

    #[test]
    fn default_activation_mode_and_loop_focus_true_are_reflected_in_list_attrs() {
        let node = tabs(&props("t", "a"), vec![item("a", false)]);
        let html = render(&node);
        assert!(html.contains(r#"data-activation-mode="automatic""#));
        assert!(html.contains(r#"data-loop-focus="true""#));
    }

    #[test]
    fn activation_mode_as_str_returns_expected_literals() {
        assert_eq!(ActivationMode::Automatic.as_str(), "automatic");
        assert_eq!(ActivationMode::Manual.as_str(), "manual");
        assert_eq!(ActivationMode::default(), ActivationMode::Automatic);
    }

    // --- イシュー #601: indicator パーツ ---

    #[test]
    fn indicator_false_omits_indicator_part() {
        // 既定（indicator: false）では data-part="indicator" が出力されない
        // （既存出力のスナップショットが不変であることの裏付け）。
        let node = tabs(&props("t", "a"), vec![item("a", false), item("b", false)]);
        let html = render(&node);
        assert!(!html.contains(r#"data-part="indicator""#));
    }

    #[test]
    fn indicator_true_with_active_tab_full_html_snapshot() {
        let node = tabs(
            &TabsProps {
                indicator: true,
                ..props("t", "a")
            },
            vec![item("a", false), item("b", false)],
        );
        assert_eq!(
            render(&node),
            concat!(
                r#"<div data-scope="tabs" data-part="root" id="t" data-orientation="horizontal">"#,
                r#"<div data-scope="tabs" data-part="list" role="tablist" aria-orientation="horizontal" data-orientation="horizontal" data-activation-mode="automatic" data-loop-focus="true">"#,
                r#"<button data-scope="tabs" data-part="trigger" type="button" id="t-trigger-a" role="tab" aria-selected="true" aria-controls="t-content-a" data-state="active" data-orientation="horizontal" tabindex="0">a</button>"#,
                r#"<button data-scope="tabs" data-part="trigger" type="button" id="t-trigger-b" role="tab" aria-selected="false" aria-controls="t-content-b" data-state="inactive" data-orientation="horizontal" tabindex="-1">b</button>"#,
                r#"<span data-scope="tabs" data-part="indicator" data-state="active" data-orientation="horizontal" aria-hidden="true" style="--left: 0px; --top: 0px; --width: 0px; --height: 0px"></span>"#,
                r#"</div>"#,
                r#"<div data-scope="tabs" data-part="content" id="t-content-a" role="tabpanel" aria-labelledby="t-trigger-a" data-state="active" data-orientation="horizontal" tabindex="0">a</div>"#,
                r#"<div data-scope="tabs" data-part="content" id="t-content-b" role="tabpanel" aria-labelledby="t-trigger-b" data-state="inactive" data-orientation="horizontal" tabindex="0" hidden="">b</div>"#,
                r#"</div>"#,
            )
        );
    }

    #[test]
    fn indicator_true_with_unmatched_selected_is_inactive_and_hidden() {
        let node = tabs(
            &TabsProps {
                indicator: true,
                ..props("t", "does-not-exist")
            },
            vec![item("a", false), item("b", false)],
        );
        let html = render(&node);
        assert!(html.contains(
            r#"<span data-scope="tabs" data-part="indicator" data-state="inactive" data-orientation="horizontal" aria-hidden="true" hidden="" style="--left: 0px; --top: 0px; --width: 0px; --height: 0px"></span>"#
        ));
    }

    #[test]
    fn indicator_true_with_selected_matching_disabled_item_is_inactive_and_hidden() {
        // selected が disabled item を指す場合も「未選択」扱い（既存の選択決定則を継承）。
        let node = tabs(
            &TabsProps {
                indicator: true,
                ..props("t", "a")
            },
            vec![item("a", true), item("b", false)],
        );
        let html = render(&node);
        assert!(html.contains(
            r#"<span data-scope="tabs" data-part="indicator" data-state="inactive" data-orientation="horizontal" aria-hidden="true" hidden="" style="--left: 0px; --top: 0px; --width: 0px; --height: 0px"></span>"#
        ));
    }

    #[test]
    fn indicator_true_with_empty_items_does_not_panic() {
        let node = tabs(
            &TabsProps {
                indicator: true,
                ..props("t", "a")
            },
            vec![],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-part="indicator""#));
        assert!(html.contains(r#"data-state="inactive""#));
        assert!(html.contains("hidden"));
    }

    #[test]
    fn indicator_true_with_vertical_orientation_reflects_data_orientation() {
        let node = tabs(
            &TabsProps {
                orientation: Orientation::Vertical,
                indicator: true,
                ..props("t", "a")
            },
            vec![item("a", false)],
        );
        let html = render(&node);
        // list/trigger/content/root/indicator の 5 パーツすべてが
        // data-orientation="vertical" を持つ。
        assert_eq!(html.matches(r#"data-orientation="vertical""#).count(), 5);
        assert!(html.contains(r#"data-part="indicator""#));
    }

    #[test]
    fn indicator_style_attribute_is_fixed_literal_with_no_dynamic_value() {
        // 動的値が混入しない不変条件の固定（#601 セキュリティ考慮）。
        let node = tabs(
            &TabsProps {
                indicator: true,
                ..props("t", "a")
            },
            vec![item("a", false)],
        );
        let html = render(&node);
        assert!(html.contains(r#"style="--left: 0px; --top: 0px; --width: 0px; --height: 0px""#));
    }

    #[test]
    fn xss_payload_in_value_with_indicator_true_does_not_leak_into_indicator_attrs() {
        // XSS 回帰（indicator 版）: payload は選択判定にのみ使われ、indicator の
        // 属性（特に style）は固定リテラルのままで payload が現れないことを確認する。
        let payload_value = "x\" onmouseover=\"alert(1)";
        let node = tabs(
            &TabsProps {
                indicator: true,
                ..props("t", payload_value)
            },
            vec![TabItem {
                value: payload_value,
                trigger: vec![text("<script>alert(1)</script>")],
                content: vec![text("<script>alert(2)</script>")],
                disabled: false,
            }],
        );
        let html = render(&node);
        assert!(!html.contains("<script>alert"));
        assert!(!html.contains("onmouseover=\"alert"));
        assert!(html.contains(r#"style="--left: 0px; --top: 0px; --width: 0px; --height: 0px""#));
        // indicator は active（selected が一致・disabled でない）なので data-state="active"。
        assert!(html.contains(
            r#"<span data-scope="tabs" data-part="indicator" data-state="active" data-orientation="horizontal" aria-hidden="true" style="--left: 0px; --top: 0px; --width: 0px; --height: 0px"></span>"#
        ));
    }
}

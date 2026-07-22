//! Tabs コンポーネント（イシュー #528、Phase 2 #526〜#544 の一部）。
//!
//! WAI-ARIA APG の Tabs パターン（`role="tablist"`/`"tab"`/`"tabpanel"`・
//! `aria-selected`・`aria-controls`/`aria-labelledby` の相互参照・roving
//! tabindex）に準拠したマークアップを、[`fandhe_frontend_core::Node`] 木として
//! 組み立てる。anatomy は ark-ui 相当の `root` / `list` / `trigger` / `content`
//! の 4 パーツ構成（`indicator` は静的 SSR マークアップに意味を持たないため
//! 本イシューのスコープ外。要否は #524/#546 側の判断に委ねる）。
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
    aria_controls, aria_disabled, aria_labelledby, aria_orientation, aria_selected, role,
};
use crate::data_attrs::{data_disabled, data_orientation, data_state, Orientation};
use fandhe_frontend_core::Node;

/// `data-scope="tabs"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("tabs");

/// `data-state` 属性値 "active"（選択中の trigger/content が持つ値）。
const DATA_STATE_ACTIVE: &str = "active";
/// `data-state` 属性値 "inactive"（非選択の trigger/content が持つ値）。
const DATA_STATE_INACTIVE: &str = "inactive";

/// タブ 1 枚の定義（trigger ラベルと対応する content パネル）。
///
/// `value` は同一 [`tabs`] 呼び出し内で一意であることが呼び出し側の契約
/// である（[`TabsProps::id`] と組み合わせて trigger/content の `id` を
/// 決定的に生成するため）。重複した場合の挙動は [`tabs`] の rustdoc を
/// 参照（先勝ち・fail-closed、panic しない）。
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
/// - `props.selected` がどの `value` とも一致しない場合、全 trigger/content
///   が inactive として描画される。
/// - roving tabindex（WAI-ARIA APG: tablist 内に常に `tabindex="0"` を
///   ちょうど 1 つ）は、active な trigger があり、かつそれが disabled で
///   なければそれに `tabindex="0"` を与える。active がない場合、または
///   active な item が disabled の場合は最初の非 disabled trigger に与える
///   （disabled item に `tabindex="0"` と `disabled` が同居する状態を避ける）。
///   全 trigger が disabled、または `items` が空の場合は誰にも `0` を
///   与えない（全て `-1`、あるいは trigger 自体が存在しない）。
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
///     &TabsProps { id: "t", selected: "a", orientation: Orientation::Horizontal },
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
    let active_index = items.iter().position(|item| item.value == props.selected);

    // roving tabindex: active かつ非 disabled ならそれが 0。
    // active が disabled（selected が disabled item の value と一致するケースを含む）、
    // または active 自体が無い場合は最初の非 disabled item にフォールバックする。
    // disabled かつ tabindex="0" が同一要素に同居する状態を避けるための意図的なガード。
    // 該当なし（items 空・全 disabled）なら誰にも 0 を与えない。
    let tabbable_index = active_index
        .filter(|&index| !items[index].disabled)
        .or_else(|| items.iter().position(|item| !item.disabled));

    let mut list_children: Vec<Node> = Vec::with_capacity(items.len());
    let mut root_extra_children: Vec<Node> = Vec::with_capacity(items.len());

    for (index, item) in items.into_iter().enumerate() {
        let is_active = active_index == Some(index);
        let is_tabbable = tabbable_index == Some(index);
        let trigger_id = format!("{}-trigger-{}", props.id, item.value);
        let content_id = format!("{}-content-{}", props.id, item.value);
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

    let list_attrs: Vec<(&str, &str)> = vec![
        role("tablist"),
        aria_orientation_attr,
        data_orientation_attr,
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
        }
    }

    #[test]
    fn two_tabs_first_selected_full_html_snapshot() {
        let node = tabs(&props("t", "a"), vec![item("a", false), item("b", false)]);
        assert_eq!(
            render(&node),
            concat!(
                r#"<div data-scope="tabs" data-part="root" id="t" data-orientation="horizontal">"#,
                r#"<div data-scope="tabs" data-part="list" role="tablist" aria-orientation="horizontal" data-orientation="horizontal">"#,
                r#"<button data-scope="tabs" data-part="trigger" type="button" id="t-trigger-a" role="tab" aria-selected="true" aria-controls="t-content-a" data-state="active" data-orientation="horizontal" tabindex="0">a</button>"#,
                r#"<button data-scope="tabs" data-part="trigger" type="button" id="t-trigger-b" role="tab" aria-selected="false" aria-controls="t-content-b" data-state="inactive" data-orientation="horizontal" tabindex="-1">b</button>"#,
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
            r#"id="t-trigger-b" role="tab" aria-selected="false" aria-controls="t-content-b" data-state="inactive" data-orientation="horizontal" tabindex="-1" disabled="" data-disabled="" aria-disabled="true""#
        ));
    }

    #[test]
    fn selected_matching_disabled_item_does_not_get_tabindex_zero() {
        // props.selected が disabled item の value と一致するケース（レビュー指摘）:
        // active な item であっても disabled なら tabindex="0" を与えず、
        // 最初の非 disabled item にフォールバックする。
        // disabled と tabindex="0" が同一要素に同居する状態を避ける。
        let node = tabs(&props("t", "a"), vec![item("a", true), item("b", false)]);
        let html = render(&node);
        // a は active（aria-selected="true"）だが disabled のため tabindex="-1"。
        assert!(html.contains(
            r#"id="t-trigger-a" role="tab" aria-selected="true" aria-controls="t-content-a" data-state="active" data-orientation="horizontal" tabindex="-1" disabled="" data-disabled="" aria-disabled="true""#
        ));
        // b は最初の非 disabled item なので tabindex="0" を得る（inactive のまま）。
        assert!(html.contains(
            r#"id="t-trigger-b" role="tab" aria-selected="false" aria-controls="t-content-b" data-state="inactive" data-orientation="horizontal" tabindex="0""#
        ));
    }

    #[test]
    fn vertical_orientation_reflects_in_aria_and_data_orientation() {
        let node = tabs(
            &TabsProps {
                id: "t",
                selected: "a",
                orientation: Orientation::Vertical,
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
            r#"id="t-trigger-a" role="tab" aria-selected="false" aria-controls="t-content-a" data-state="inactive" data-orientation="horizontal" tabindex="0""#
        ));
        assert!(html.contains(
            r#"id="t-trigger-b" role="tab" aria-selected="false" aria-controls="t-content-b" data-state="inactive" data-orientation="horizontal" tabindex="-1""#
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
    fn empty_items_renders_root_and_list_only_without_panicking() {
        let node = tabs(&props("t", "a"), vec![]);
        assert_eq!(
            render(&node),
            concat!(
                r#"<div data-scope="tabs" data-part="root" id="t" data-orientation="horizontal">"#,
                r#"<div data-scope="tabs" data-part="list" role="tablist" aria-orientation="horizontal" data-orientation="horizontal"></div>"#,
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
}

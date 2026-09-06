//! `tree_view`（イシュー #753）を ark-ui docs / zag.js
//! `packages/machines/tree-view/src/tree-view.connect.ts` と突合した契約を
//! fail-closed に固定する統合テスト（イシュー #1667、
//! `splitter_reference_parity.rs` と同型の立て付け）。Radix Primitives に
//! Tree View 相当は存在しないため突合対象に含めない
//! （`docs/design/component-coverage-map.md` 参照）。
//!
//! # 突合結果（詳細は `crate::tree_view` モジュール doc「参照突合
//! （イシュー #1667）」節参照）
//!
//! - **是正**: [`branch`] へ `data-branch`（= ノード値）を追加、
//!   [`branch_control`] へ `data-value`/`data-depth` を追加、
//!   [`branch_indicator`] へ `data-disabled`/`data-selected`/
//!   `aria-hidden="true"` を追加、[`branch_text`] へ `data-state`/
//!   `data-disabled` を追加、[`branch_content`] へ `data-depth`/`data-value`
//!   を追加、[`branch_indent_guide`] へ `data-depth` を追加、[`item_text`]
//!   へ `data-selected`/`data-disabled` を追加、[`item_indicator`] へ
//!   `data-disabled`/`aria-hidden="true"`/非選択時 `hidden` を追加。
//! - **非追随**: `data-focus`/`data-renaming`/`data-checked`/
//!   `data-indeterminate`/`data-loading`/`aria-busy`、`data-path`/
//!   `data-ownedby`/`id`/`dir`/`tabindex`/`--depth` style、
//!   `aria-multiselectable`、`aria-current="true"`（zag の item）、
//!   `role="button"` on branch-control、branch-trigger /
//!   node-checkbox / node-rename-input。
//! - **APG superset として維持**: `aria-posinset`/`aria-setsize`
//!   （zag は出力しない）、disabled 時も明示する `aria-selected="false"`
//!   （zag は disabled 時に省略）。
//! - **`data-depth` の起点**: zag.js は 1 起点だが、本実装は 0 起点を
//!   意図的に維持する（`aria-level` が 1 起点の深さを既に担う。
//!   `fandhe-frontend-wasm-full` `keynav.rs` の `depth == 0` ルート判定との
//!   整合のため）。
//!
//! 公開 API（`fandhe_frontend_headless_ui::tree_view`）のみを使い、
//! `crate` 内部実装には依存しない。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::tree_view::{self, TreeItemProps};
use fandhe_frontend_headless_ui::OpenState;

/// `tag>` までを抜き出し開始タグのみを対象に属性を検査するヘルパ
/// （`splitter_reference_parity.rs` と同型のパターン）。
fn tag_slice<'a>(html: &'a str, needle: &str) -> &'a str {
    let start = html.find(needle).unwrap_or_else(|| {
        panic!("needle {needle:?} not found in html: {html}");
    });
    let end = html[start..].find('>').unwrap() + start;
    &html[start..end]
}

fn sample_props(selected: bool, disabled: bool) -> TreeItemProps<'static> {
    TreeItemProps {
        value: "src",
        selected,
        disabled,
        level: "1",
        posinset: "1",
        setsize: "2",
        depth: "0",
    }
}

/// ark-ui docs の Data Attributes 表: branch は `data-value`/`data-depth`/
/// `data-branch`（イシュー #1667 で追加）を持つ。`data-path`/`data-ownedby`/
/// `id` は zag.js 固有の内部識別子であり非追随。
#[test]
fn branch_has_data_value_depth_and_branch() {
    let html = render(&tree_view::branch(
        OpenState::Closed,
        sample_props(false, false),
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="branch""#);
    assert!(tag.contains(r#"data-value="src""#));
    assert!(tag.contains(r#"data-branch="src""#));
    assert!(tag.contains(r#"data-depth="0""#));
    assert!(!tag.contains("data-path"));
    assert!(!tag.contains("data-ownedby"));
    assert!(!tag.contains(" id="));
    assert!(!tag.contains(" dir="));
    assert!(!tag.contains("data-focus"));
    assert!(!tag.contains("aria-current"));
}

/// APG superset として維持: disabled 時も `aria-selected="false"`（zag は
/// disabled 時に省略）を出力し続ける。
#[test]
fn branch_disabled_still_emits_aria_selected_false() {
    let html = render(&tree_view::branch(
        OpenState::Closed,
        sample_props(false, true),
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="branch""#);
    assert!(tag.contains(r#"aria-selected="false""#));
    assert!(tag.contains(r#"aria-disabled="true""#));
}

/// ark-ui docs: branch-control は `data-value`/`data-depth`/`data-state`/
/// `data-selected`/`data-disabled` を持つが `role="button"` は持たない
/// （APG ではフォーカス可能要素は treeitem（branch）が担う）。
#[test]
fn branch_control_has_data_value_and_depth_but_no_role() {
    let html = render(&tree_view::branch_control(
        OpenState::Open,
        sample_props(true, false),
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="branch-control""#);
    assert!(tag.contains(r#"data-value="src""#));
    assert!(tag.contains(r#"data-depth="0""#));
    assert!(tag.contains(r#"data-state="open""#));
    assert!(tag.contains(r#"data-selected="""#));
    assert!(!tag.contains("role="));
    assert!(!html.contains("<button"));
}

/// ark-ui docs: branch-indicator は `data-state`/`data-disabled`/
/// `data-selected`/`aria-hidden="true"` を持つ（装飾アイコンを支援技術から
/// 隠す）。
#[test]
fn branch_indicator_has_data_state_disabled_selected_and_aria_hidden() {
    let html = render(&tree_view::branch_indicator(
        OpenState::Closed,
        sample_props(true, true),
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="branch-indicator""#);
    assert!(tag.contains(r#"data-state="closed""#));
    assert!(tag.contains(r#"data-selected="""#));
    assert!(tag.contains(r#"data-disabled="""#));
    assert!(tag.contains(r#"aria-hidden="true""#));
}

/// ark-ui docs: branch-text は `data-state`/`data-disabled` を持つ。
#[test]
fn branch_text_has_data_state_and_disabled() {
    let html = render(&tree_view::branch_text(
        OpenState::Open,
        sample_props(false, true),
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="branch-text""#);
    assert!(tag.contains(r#"data-state="open""#));
    assert!(tag.contains(r#"data-disabled="""#));
}

/// ark-ui docs: branch-content は `role="group"`/`data-state`/`data-depth`/
/// `data-value` を持つ。
#[test]
fn branch_content_has_role_group_depth_and_value() {
    let html = render(&tree_view::branch_content(
        OpenState::Closed,
        sample_props(false, false),
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="branch-content""#);
    assert!(tag.contains(r#"role="group""#));
    assert!(tag.contains(r#"data-state="closed""#));
    assert!(tag.contains(r#"data-depth="0""#));
    assert!(tag.contains(r#"data-value="src""#));
    assert!(tag.contains(r#"hidden="""#));
}

/// ark-ui docs: branch-indent-guide は `data-depth` を持つ。
#[test]
fn branch_indent_guide_has_data_depth() {
    let props = TreeItemProps {
        depth: "3",
        ..sample_props(false, false)
    };
    let html = render(&tree_view::branch_indent_guide(props, vec![], vec![]));
    let tag = tag_slice(&html, r#"data-part="branch-indent-guide""#);
    assert!(tag.contains(r#"data-depth="3""#));
}

/// ark-ui docs: item は `aria-current` を出力しない（`aria-selected` が
/// 選択の正、zag との非追随）。
#[test]
fn item_does_not_emit_aria_current() {
    let html = render(&tree_view::item(
        TreeItemProps {
            value: "file.txt",
            ..sample_props(true, false)
        },
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="item""#);
    assert!(tag.contains(r#"aria-selected="true""#));
    assert!(!tag.contains("aria-current"));
    assert!(!tag.contains(" tabindex="));
}

/// ark-ui docs: item-text は `data-selected`/`data-disabled` を持つ。
#[test]
fn item_text_has_data_selected_and_disabled() {
    let html = render(&tree_view::item_text(
        sample_props(true, true),
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="item-text""#);
    assert!(tag.contains(r#"data-selected="""#));
    assert!(tag.contains(r#"data-disabled="""#));
}

/// ark-ui docs: item-indicator は `data-disabled`/`aria-hidden="true"` を
/// 持ち、非選択時は `hidden` 存在属性を持つ。
#[test]
fn item_indicator_hidden_when_not_selected_and_aria_hidden_always() {
    let unselected = render(&tree_view::item_indicator(
        sample_props(false, true),
        vec![],
        vec![],
    ));
    let tag = tag_slice(&unselected, r#"data-part="item-indicator""#);
    assert!(tag.contains(r#"aria-hidden="true""#));
    assert!(tag.contains(r#"data-disabled="""#));
    assert!(tag.contains(r#" hidden="""#));

    let selected = render(&tree_view::item_indicator(
        sample_props(true, false),
        vec![],
        vec![],
    ));
    let tag_selected = tag_slice(&selected, r#"data-part="item-indicator""#);
    assert!(!tag_selected.contains(r#" hidden="""#));
    assert!(tag_selected.contains(r#"data-selected="""#));
}

/// `drop_reserved`（イシュー #1667 で全パーツへ導入）: 呼び出し側 `attrs`
/// が予約キーを偽装・重複出力できないことを固定する（`branch` を代表例に
/// する。全パーツのユニットテストは `crate::tree_view::tests` 側で網羅）。
#[test]
fn caller_supplied_reserved_keys_are_dropped() {
    let html = render(&tree_view::branch(
        OpenState::Closed,
        sample_props(false, false),
        vec![
            ("role", "attacker"),
            ("data-value", "attacker"),
            ("data-branch", "attacker"),
            ("aria-expanded", "attacker"),
        ],
        vec![],
    ));
    assert!(!html.contains("attacker"));
}

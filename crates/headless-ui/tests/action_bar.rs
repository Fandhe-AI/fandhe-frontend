//! `action_bar`（イシュー #762）の公開 API 経由統合テスト。
//!
//! `crates/headless-ui/src/action_bar.rs` 側のユニットテストが値ごとの
//! 詳細な属性検証を行っているのに対し、本ファイルは参考サイトとの突合
//! 契約（イシュー #1647）に絞る。ark-ui・Radix Primitives のいずれにも
//! ActionBar 相当は存在せず、chakra-ui の ActionBar は独自の状態機械を
//! 持たず Ark Popover（zag.js `popover.connect`）をそのまま再利用して
//! いる。このため本ファイルは zag.js popover の content/close-trigger 属性
//! 仕様を基準に、6 anatomy パーツの過不足・`data-*`/`role`/`aria-*` の
//! 一致点と意図的差分（モジュール doc「参照基準」節参照）の双方を
//! fail-closed に固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::action_bar::{
    close_trigger, content, positioner, root, selection_trigger, separator,
    CLOSE_TRIGGER_ARIA_LABEL,
};
use fandhe_frontend_headless_ui::state::OpenState;

/// anatomy は root/positioner/content/selection-trigger/separator/
/// close-trigger の 6 パーツ固定（chakra-ui `action-bar.tsx` の
/// エクスポート構成に一致、増減なし）。
#[test]
fn reference_anatomy_has_six_parts() {
    let html = render(&root(
        OpenState::Open,
        vec![],
        vec![positioner(
            OpenState::Open,
            vec![],
            vec![content(
                OpenState::Open,
                "3 selected",
                vec![],
                vec![
                    selection_trigger(vec![], vec![text("3 selected")]),
                    separator(vec![], vec![]),
                    close_trigger(vec![], vec![text("Close")]),
                ],
            )],
        )],
    ));

    for part in [
        "root",
        "positioner",
        "content",
        "selection-trigger",
        "separator",
        "close-trigger",
    ] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "anatomy パーツ {part} が出力される: {html}"
        );
    }
    assert_eq!(
        html.matches("data-part=").count(),
        6,
        "action-bar の anatomy は 6 パーツのみ: {html}"
    );
}

/// content パーツの `role` は参考基準（zag.js popover、非モーダル）に合わせ
/// `"dialog"` を出力し、旧 `"toolbar"`（roving tabindex を伴わない不完全な
/// 適用だった）は出力しない。`aria-modal` も非モーダルのため付与しない
/// （イシュー #1647 是正、**破壊的変更**）。
#[test]
fn content_role_matches_reference_dialog_not_toolbar() {
    let html = render(&content(OpenState::Open, "3 selected", vec![], vec![]));
    assert!(html.contains(r#"role="dialog""#));
    assert!(!html.contains(r#"role="toolbar""#));
    assert!(!html.contains("aria-modal"));
}

/// content パーツは開状態のときのみ `data-expanded` を出力する（zag.js
/// popover の content と同じ語彙）。
#[test]
fn content_data_expanded_matches_open_state() {
    let open = render(&content(OpenState::Open, "label", vec![], vec![]));
    assert!(open.contains("data-expanded"));

    let closed = render(&content(OpenState::Closed, "label", vec![], vec![]));
    assert!(!closed.contains("data-expanded"));
}

/// content パーツは `tabindex="-1"` を固定で出力する（chakra
/// `autoFocus: false` に対応。開時にフォーカスを自動移動しない）。
#[test]
fn content_has_tabindex_minus_one() {
    let html = render(&content(OpenState::Open, "label", vec![], vec![]));
    assert!(html.contains(r#"tabindex="-1""#));
}

/// close-trigger パーツは呼び出し側が `aria-label` を指定しなければ
/// 参考基準（zag.js popover `translations.closeTrigger` 既定値 `"close"`）
/// に合わせた既定値を出力する。
#[test]
fn close_trigger_has_reference_default_aria_label() {
    let html = render(&close_trigger(vec![], vec![]));
    assert!(html.contains(&format!(r#"aria-label="{CLOSE_TRIGGER_ARIA_LABEL}""#)));
}

/// 参考基準に存在しない `data-placement`/`data-side`（placement variant）は
/// 本実装のスコープ外のまま出力しない（`docs/policy/intentional-non-adoption.md`
/// §3.25 規則 2）。
#[test]
fn placement_variant_data_attrs_are_not_emitted() {
    let html = render(&content(OpenState::Open, "label", vec![], vec![]));
    assert!(!html.contains("data-placement"));
    assert!(!html.contains("data-side"));
}

/// Anatomy::part の fail-closed 契約（呼び出し側による `data-scope`/
/// `data-part` 偽装除去）が root/content/close-trigger の全パーツで
/// 効いていることを統合テストとして固定する。
#[test]
fn caller_supplied_scope_and_part_are_dropped_across_parts() {
    let attack_attrs = vec![("data-scope", "attacker"), ("data-part", "attacker")];

    let root_html = render(&root(OpenState::Closed, attack_attrs.clone(), vec![]));
    assert!(root_html.contains(r#"data-scope="action-bar""#));
    assert!(root_html.contains(r#"data-part="root""#));

    let content_html = render(&content(
        OpenState::Open,
        "label",
        attack_attrs.clone(),
        vec![],
    ));
    assert!(content_html.contains(r#"data-scope="action-bar""#));
    assert!(content_html.contains(r#"data-part="content""#));

    let close_html = render(&close_trigger(attack_attrs, vec![]));
    assert!(close_html.contains(r#"data-scope="action-bar""#));
    assert!(close_html.contains(r#"data-part="close-trigger""#));

    for html in [root_html, content_html, close_html] {
        assert!(!html.contains("attacker"));
    }
}

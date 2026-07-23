//! Pagination（イシュー #751）の統合テスト。
//!
//! `crates/headless-ui/src/pagination.rs` の inline unit tests がパーツ単体の
//! 属性出力・`page_range` の決定性/境界・状態機械の正規化/dispatch/hydration
//! を固定するのに対し、本ファイルは
//! `root(items + ellipsis を page_entries() から組み立て + prev/next trigger)`
//! という全体の組み立てにおける `aria-current`/`data-selected` の一意性・
//! 端到達時のトリガー disabled 連動・SSR/hydration 両経路をクレート外部から
//! （公開 API のみを使って）固定する（`tests/number_input.rs` と同粒度）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::pagination::{self, ItemMode, PageEntry, Pagination};
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate, HydrateError};

/// `page_entries()` を走査して `nav > item*/ellipsis* + prev/next` を組み立てる
/// ヘルパ（`fandhe-frontend-pre-styled-ui` の styled ラッパーが行う想定の
/// 組み立てを外部クレート視点で再現する）。
fn assemble(p: &Pagination) -> String {
    let mut items = Vec::new();
    for entry in p.page_entries() {
        match entry {
            PageEntry::Page(n) => items.push(p.item(
                ItemMode::Button,
                n,
                false,
                vec![],
                vec![text(n.to_string())],
            )),
            PageEntry::Ellipsis => items.push(pagination::ellipsis(vec![], vec![])),
        }
    }
    let mut children = vec![p.prev_trigger(ItemMode::Button, vec![], vec![text("Prev")])];
    children.extend(items);
    children.push(p.next_trigger(ItemMode::Button, vec![], vec![text("Next")]));

    render(&p.root("pagination", vec![], children))
}

#[test]
fn full_assembly_wires_root_items_ellipsis_and_triggers() {
    let p = Pagination::new(200, 10, 1, 1, 10);
    let html = assemble(&p);

    assert!(html.contains("<nav"));
    assert!(html.contains(r#"data-scope="pagination""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="item""#));
    assert!(html.contains(r#"data-part="ellipsis""#));
    assert!(html.contains(r#"data-part="prev-trigger""#));
    assert!(html.contains(r#"data-part="next-trigger""#));

    // 現在ページ(10)の aria-current="page" はちょうど 1 回のみ出現する。
    assert_eq!(html.matches(r#"aria-current="page""#).count(), 1);
    assert_eq!(html.matches("data-selected").count(), 1);
}

#[test]
fn prev_trigger_disabled_only_at_first_page() {
    let first = Pagination::new(50, 10, 1, 1, 1);
    let html = assemble(&first);
    let prev_start = html.find(r#"data-part="prev-trigger""#).unwrap();
    let prev_end = html[prev_start..].find('>').unwrap() + prev_start;
    assert!(html[prev_start..prev_end].contains("disabled"));

    let middle = Pagination::new(50, 10, 1, 1, 3);
    let html = assemble(&middle);
    let prev_start = html.find(r#"data-part="prev-trigger""#).unwrap();
    let prev_end = html[prev_start..].find('>').unwrap() + prev_start;
    assert!(!html[prev_start..prev_end].contains("disabled"));
}

#[test]
fn next_trigger_disabled_only_at_last_page() {
    let last = Pagination::new(50, 10, 1, 1, 5);
    let html = assemble(&last);
    let next_start = html.rfind(r#"data-part="next-trigger""#).unwrap();
    let next_end = html[next_start..].find('>').unwrap() + next_start;
    assert!(html[next_start..next_end].contains("disabled"));

    let middle = Pagination::new(50, 10, 1, 1, 3);
    let html = assemble(&middle);
    let next_start = html.rfind(r#"data-part="next-trigger""#).unwrap();
    let next_end = html[next_start..].find('>').unwrap() + next_start;
    assert!(!html[next_start..next_end].contains("disabled"));
}

#[test]
fn dispatch_goto_then_ssr_hydration_round_trip() {
    let mut p = Pagination::new(200, 10, 2, 1, 1);
    assert!(dispatch(&mut p, "goto", "12"));
    assert_eq!(p.page(), 12);

    let hydrate_html = render(&render_for_hydration(&p));
    assert!(hydrate_html.contains(r#"data-hydrate-page="12""#));

    let restored = Pagination::from_hydration_attrs(&p.hydration_attrs()).unwrap();
    assert_eq!(restored, p);
}

#[test]
fn from_hydration_attrs_out_of_range_page_does_not_panic() {
    let attrs = vec![
        ("data-hydrate-count".to_string(), "10".to_string()),
        ("data-hydrate-page-size".to_string(), "10".to_string()),
        ("data-hydrate-sibling-count".to_string(), "1".to_string()),
        ("data-hydrate-boundary-count".to_string(), "1".to_string()),
        ("data-hydrate-page".to_string(), "2".to_string()), // total_pages=1 のため範囲外
    ];
    let err = Pagination::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

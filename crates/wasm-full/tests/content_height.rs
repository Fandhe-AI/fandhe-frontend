//! `fandhe_frontend_wasm_full::content_height`（イシュー #2191、親
//! トラッキング #2189）の native 統合テスト。
//!
//! 純粋層（[`format_content_height`]/[`target_selector`]）の書式契約に
//! 加え、[`TARGETS`] 静的表と `fandhe-frontend-headless-ui` 実出力の
//! `(data-scope, data-part)` がドリフトしていないことを、実際の
//! headless-ui コンポーネントを `render()` した HTML 中の文字列一致で
//! 機械検知する（`tests/headless_wiring.rs` と同型のドリフト検知
//! パターン）。あわせて `hidden` 契約（closed のとき `hidden` を持つ）が
//! 崩れていないことも同一の出力から再確認する。
//!
//! `web_sys::Element` を組み立てての実測・CSS 変数書き込み（配線層
//! `content_height::sync_content_height`）は wasm32 専用のため本ファイル
//! では検証できない（実ブラウザ回帰は `tests/content_height_browser.rs`
//! に委ねる）。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::accordion::{self, AccordionProps};
use fandhe_frontend_headless_ui::collapsible;
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_wasm_full::content_height::{
    format_content_height, is_target, target_selector, TARGETS,
};

// --- 純粋層の書式契約 ---

#[test]
fn format_content_height_negative_is_none() {
    assert_eq!(format_content_height(-1), None);
    assert_eq!(format_content_height(i32::MIN), None);
}

#[test]
fn format_content_height_zero_is_0px() {
    assert_eq!(format_content_height(0), Some("0px".to_string()));
}

#[test]
fn format_content_height_positive_is_npx() {
    assert_eq!(format_content_height(1), Some("1px".to_string()));
    assert_eq!(format_content_height(240), Some("240px".to_string()));
}

#[test]
fn target_selector_is_static_and_stable() {
    let selector = target_selector();
    assert_eq!(
        selector,
        r#"[data-scope="collapsible"][data-part="content"],[data-scope="accordion"][data-part="item-content"]"#
    );
    // 2 回呼んでも同一（副作用・グローバル状態を持たない純粋関数である
    // ことの確認）。
    assert_eq!(selector, target_selector());
}

// --- TARGETS 表と headless-ui 実出力のドリフト検知 ---

/// `html` が `data-scope="{scope}"`/`data-part="{part}"` を両方含むことを
/// 確認する（`headless_wiring.rs::assert_scope_part_present` と同型）。
fn assert_scope_part_present(html: &str, scope: &str, part: &str) {
    assert!(
        html.contains(&format!(r#"data-scope="{scope}""#)),
        "data-scope=\"{scope}\" が出力に含まれない（headless-ui 側の scope 変更で\
         wasm-full::content_height::TARGETS がドリフトした可能性）: {html}"
    );
    assert!(
        html.contains(&format!(r#"data-part="{part}""#)),
        "data-part=\"{part}\" が出力に含まれない（headless-ui 側の part 変更で\
         wasm-full::content_height::TARGETS がドリフトした可能性）: {html}"
    );
}

#[test]
fn targets_table_matches_collapsible_content_output() {
    let (scope, part) = TARGETS[0];
    assert_eq!((scope, part), ("collapsible", "content"));

    let html = render(&collapsible::content(
        OpenState::Open,
        false,
        None,
        vec![],
        vec![],
    ));
    assert_scope_part_present(&html, scope, part);
    assert!(is_target(scope, part));
}

#[test]
fn targets_table_matches_accordion_item_content_output() {
    let (scope, part) = TARGETS[1];
    assert_eq!((scope, part), ("accordion", "item-content"));

    let props = AccordionProps::default();
    let html = render(&accordion::item_content(
        OpenState::Open,
        false,
        &props,
        None,
        None,
        vec![],
        vec![],
    ));
    assert_scope_part_present(&html, scope, part);
    assert!(is_target(scope, part));
}

// --- hidden 契約の再確認（本イシューは headless-ui 非変更が前提） ---

#[test]
fn collapsible_content_closed_has_hidden_open_does_not() {
    let closed = render(&collapsible::content(
        OpenState::Closed,
        false,
        None,
        vec![],
        vec![],
    ));
    assert!(
        closed.contains("hidden"),
        "closed の content は hidden を持つこと: {closed}"
    );

    let open = render(&collapsible::content(
        OpenState::Open,
        false,
        None,
        vec![],
        vec![],
    ));
    assert!(
        !open.contains("hidden"),
        "open の content は hidden を持たないこと: {open}"
    );
}

#[test]
fn accordion_item_content_closed_has_hidden_open_does_not() {
    let props = AccordionProps::default();
    let closed = render(&accordion::item_content(
        OpenState::Closed,
        false,
        &props,
        None,
        None,
        vec![],
        vec![],
    ));
    assert!(
        closed.contains("hidden"),
        "closed の item-content は hidden を持つこと: {closed}"
    );

    let open = render(&accordion::item_content(
        OpenState::Open,
        false,
        &props,
        None,
        None,
        vec![],
        vec![],
    ));
    assert!(
        !open.contains("hidden"),
        "open の item-content は hidden を持たないこと: {open}"
    );
}

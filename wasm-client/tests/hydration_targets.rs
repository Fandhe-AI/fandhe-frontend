//! `rws-wasm-client` 純粋ロジック層のネイティブ回帰テスト（TASK-6.2c、#49）。
//!
//! `rlib` として通常の Rust クレート経由でリンクし、wasm ビルド・実ブラウザを
//! 介さずに以下を確認する（`docs/hydration-api.md` 第 7 節・テスト階層 1）。
//!
//! - SSR（`rws_app` の各ページ関数 → `rws_core::render`）と CSR 用純粋関数
//!   （[`rws_wasm_client::render_list_page_html`] 等）の出力が完全一致すること
//!   （REQ-6「CSR が SSR/SSG と同一関数を呼ぶ」契約）。
//! - `demo_items()[1]` の XSS ペイロードが CSR 経路でもエスケープされること
//!   （REQ-1 の CSR 経路での回帰）。
//! - ハイドレーション対象特定関数（[`rws_wasm_client::find_hydrate_target_kinds`] /
//!   [`rws_wasm_client::find_list_nav_targets`]）が `rws_app` の DOM 契約
//!   （`data-hydrate` / `data-nav`）と整合すること。

use rws_app::{demo_items, detail_page, list_page};
use rws_core::render;
use rws_wasm_client::{
    find_hydrate_target_kinds, find_list_nav_targets, render_detail_page_html,
    render_list_page_html, LIKE_HYDRATE_VALUE,
};

#[test]
fn render_list_page_html_matches_ssr_output() {
    let ssr = render(&list_page(&demo_items()));
    assert_eq!(render_list_page_html(), ssr);
}

#[test]
fn render_detail_page_html_matches_ssr_output_for_existing_item() {
    let items = demo_items();
    let item = items.iter().find(|it| it.id == "1");
    let ssr = render(&detail_page(item));
    assert_eq!(render_detail_page_html("1"), ssr);
}

#[test]
fn render_detail_page_html_handles_missing_item_without_panic() {
    let html = render_detail_page_html("does-not-exist");
    assert!(html.contains("見つかりません"));
}

/// REQ-1 の CSR 経路回帰: `demo_items()[1]`（意図的な XSS ペイロード）が
/// `render_list_page_html` / `render_detail_page_html` の出力でもエスケープ
/// されており、生の `<script>` タグとして出力されないことを確認する。
#[test]
fn csr_pure_functions_escape_xss_payload_in_demo_items() {
    let list_html = render_list_page_html();
    assert!(
        !list_html.contains("<script>alert"),
        "CSR 用一覧 HTML で XSS ペイロードがエスケープされずに出力された: {list_html}"
    );
    assert!(list_html.contains("&lt;script&gt;alert"));

    let detail_html = render_detail_page_html("2");
    assert!(
        !detail_html.contains("<script>alert"),
        "CSR 用詳細 HTML で XSS ペイロードがエスケープされずに出力された: {detail_html}"
    );
    assert!(detail_html.contains("&lt;script&gt;alert"));
}

#[test]
fn find_hydrate_target_kinds_locates_like_button_for_existing_item() {
    assert_eq!(
        find_hydrate_target_kinds("1"),
        vec![LIKE_HYDRATE_VALUE.to_string()]
    );
}

#[test]
fn find_hydrate_target_kinds_is_empty_for_missing_item() {
    // 404 相当ノード（見つかりません）には data-hydrate 属性を持つ要素がない。
    assert!(find_hydrate_target_kinds("does-not-exist").is_empty());
}

#[test]
fn find_list_nav_targets_lists_all_item_hrefs() {
    let targets = find_list_nav_targets();
    assert_eq!(
        targets,
        vec![
            "/items/1".to_string(),
            "/items/2".to_string(),
            "/items/3".to_string(),
        ]
    );
}

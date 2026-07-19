//! `fandhe-frontend-wasm-client` 純粋ロジック層のネイティブ回帰テスト（TASK-6.2c、#49）。
//!
//! `rlib` として通常の Rust クレート経由でリンクし、wasm ビルド・実ブラウザを
//! 介さずに以下を確認する（`docs/api/hydration-api.md` 第 7 節・テスト階層 1）。
//!
//! - SSR（`fandhe_frontend_app` の各ページ関数 → `fandhe_frontend_core::render`）と CSR 用純粋関数
//!   （[`fandhe_frontend_wasm_client::render_list_page_html`] 等）の出力が完全一致すること
//!   （REQ-6「CSR が SSR/SSG と同一関数を呼ぶ」契約）。
//! - `demo_items()[1]` の XSS ペイロードが CSR 経路でもエスケープされること
//!   （REQ-1 の CSR 経路での回帰）。
//! - ハイドレーション対象特定関数（[`fandhe_frontend_wasm_client::find_hydrate_target_kinds`] /
//!   [`fandhe_frontend_wasm_client::find_list_nav_targets`]）が `fandhe_frontend_app` の DOM 契約
//!   （`data-hydrate` / `data-nav`）と整合すること。
//! - イシュー #375（`fandhe_frontend_app::demo_items()` 直呼び出しから
//!   `fandhe_frontend_app::Loader` 経由への移行）: loader 経路の決定性・fail-closed 挙動
//!   （[`loader_error_view`] への収束・機微情報の非混入）を追加固定する。

use fandhe_frontend_app::{demo_items, detail_page, list_page, Item, Loader};
use fandhe_frontend_core::render;
use fandhe_frontend_wasm_client::{
    find_hydrate_target_kinds, find_list_nav_targets, loader_error_view, render_detail_page_html,
    render_list_page_html, resolve_detail_node, resolve_list_node, LIKE_HYDRATE_VALUE,
};
use std::fmt;

/// `AlwaysOkListLoader` 用のダミー `Error` 型（イシュー #375、Bugbot 指摘対応）。
///
/// 値を構築する経路を持たない never 型相当の enum。`resolve_list_node` が
/// `Loader::Error = Infallible` 以外の成功パスでも正しく型接続すること
/// （ドキュメント記載どおりの検証）を、実際に `Infallible` から区別できる
/// 型で確認する。
#[derive(Debug)]
enum NeverConstructed {}

impl fmt::Display for NeverConstructed {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

/// 一覧 loader が必ず失敗するテストフィクスチャ。`Error` にダミーの機微情報
/// 風文字列（実クレデンシャルは使わない）を含め、[`resolve_list_node`] の
/// 出力へ一切混入しないことを検証する（`server/src/ssr.rs::FailingListLoader`
/// と同型、イシュー #375）。
struct FailingListLoader;

impl Loader for FailingListLoader {
    type Input = ();
    type Output = Vec<Item>;
    type Error = String;

    fn load(&self, _input: &()) -> Result<Vec<Item>, String> {
        Err("db_password=dummy-secret /internal/path".to_string())
    }
}

/// 詳細 loader 版の失敗フィクスチャ（[`FailingListLoader`] と同様）。
struct FailingDetailLoader;

impl Loader for FailingDetailLoader {
    type Input = String;
    type Output = Option<Item>;
    type Error = String;

    fn load(&self, _input: &String) -> Result<Option<Item>, String> {
        Err("db_password=dummy-secret /internal/path".to_string())
    }
}

/// 常に成功する参照実装と同一入力を返すが `Error` 型を `Infallible` 以外に
/// している以外は等価な loader。`resolve_*_node` が失敗系だけでなく成功系も
/// 型に対して正しく動作すること（`where` 束縛の型接続確認）を補強する
/// （イシュー #375、Bugbot 指摘対応: `Error = Infallible` のままでは
/// `DemoItemsLoader` と型が重複し非 `Infallible` の成功パスを一度も
/// カバーしないため、[`NeverConstructed`] へ変更）。
struct AlwaysOkListLoader;

impl Loader for AlwaysOkListLoader {
    type Input = ();
    type Output = Vec<Item>;
    type Error = NeverConstructed;

    fn load(&self, _input: &()) -> Result<Vec<Item>, NeverConstructed> {
        Ok(demo_items())
    }
}

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

/// イシュー #375: [`resolve_list_node`] が別 loader 実装（`AlwaysOkListLoader`）
/// を渡しても、参照 loader（`DemoItemsLoader`）と同じ入力データからは同じ
/// ノード木（＝同じ `render` 出力）を返すこと（`where` 束縛の型接続確認）。
#[test]
fn resolve_list_node_is_deterministic_across_equivalent_loaders() {
    let via_default = render(&resolve_list_node(&fandhe_frontend_app::DemoItemsLoader));
    let via_custom = render(&resolve_list_node(&AlwaysOkListLoader));
    assert_eq!(via_default, via_custom);
    assert_eq!(via_default, render_list_page_html());
}

/// イシュー #375 受け入れ条件: 一覧 loader が失敗した場合、
/// [`resolve_list_node`] が [`loader_error_view`] へ収束し、`Loader::Error`
/// に含めたダミー機微文字列が出力へ一切混入しないこと（fail-closed の直接
/// 証明、`server/src/ssr.rs` の同型テストと対をなす）。
#[test]
fn resolve_list_node_falls_back_to_loader_error_view_and_leaks_nothing() {
    let node = resolve_list_node(&FailingListLoader);
    assert_eq!(render(&node), render(&loader_error_view()));

    let html = render(&node);
    assert!(!html.contains("db_password"));
    assert!(!html.contains("dummy-secret"));
    assert!(!html.contains("/internal/path"));
}

/// 詳細画面版の fail-closed 回帰（[`resolve_list_node_falls_back_to_loader_error_view_and_leaks_nothing`]
/// と同様）。
#[test]
fn resolve_detail_node_falls_back_to_loader_error_view_and_leaks_nothing() {
    let node = resolve_detail_node(&FailingDetailLoader, "1");
    assert_eq!(render(&node), render(&loader_error_view()));

    let html = render(&node);
    assert!(!html.contains("db_password"));
    assert!(!html.contains("dummy-secret"));
    assert!(!html.contains("/internal/path"));
}

/// 対照: 未知の id は `Loader::Error` ではなく `Output = None`（404 相当）で
/// あり、loader 自体は成功する。[`resolve_detail_node`] は
/// [`loader_error_view`] ではなく従来どおり `detail_page(None)` 相当の
/// ノードを返すことを固定する（`Error` と「見つからない」の区別、
/// 設計書 §3.3）。
#[test]
fn resolve_detail_node_returns_not_found_node_not_error_view_for_unknown_id() {
    let node = resolve_detail_node(&fandhe_frontend_app::DemoItemDetailLoader, "does-not-exist");
    assert_ne!(render(&node), render(&loader_error_view()));
    assert_eq!(render(&node), render(&detail_page(None)));
}

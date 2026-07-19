//! `rws_wasm_full::csr`（TASK-CSR-loader・#349）の native テスト。
//!
//! `csr` モジュールは DOM 非依存の純粋層（`resolve_list_node`/
//! `resolve_detail_node`/`loader_error_view`）であるため、`wasm32` ターゲット・
//! 実 DOM を介さず native の `cargo test --workspace` から直接検証できる
//! （`wasm-full/tests/runtime_headless.rs` と同じ 2 層構成方針）。
//!
//! # 検証内容
//!
//! 1. CSR ≡ 直呼び: `resolve_list_node`/`resolve_detail_node` の出力が
//!    `rws_app::list_page`/`detail_page` を直接呼んだ場合とバイト一致する
//!    （三モード整合の native 側前提。実ブラウザ側の等価検証は
//!    `wasm-full/tests/three_mode_browser.rs` が担う）。
//! 2. 決定性: 同一入力での反復呼び出しが完全一致する。
//! 3. fail-closed: `Error` に機微情報風文字列を含む loader でも、出力が
//!    [`loader_error_view`] の render 結果と一致し、機微文字列を含まず、
//!    固定文言のみを含む（`server/src/ssr.rs` の `FailingListLoader`
//!    パターンを踏襲、`.claude/rules/security.md`「機微情報の露出」）。
//! 4. XSS 回帰: `demo_items()` の XSS ペイロード item（id="2"）が loader
//!    経由でも既定エスケープされ、生の `<script>` タグが出力に現れない
//!    （REQ-1）。

use rws_app::{
    demo_items, detail_page, list_page, DemoItemDetailLoader, DemoItemsLoader, Item, Loader,
};
use rws_core::render;
use rws_wasm_full::csr::{loader_error_view, resolve_detail_node, resolve_list_node};

/// 検証 1（一覧）: `resolve_list_node` の出力は `list_page` を直接呼んだ
/// 場合とバイト一致する。
#[test]
fn resolve_list_node_matches_direct_list_page_call() {
    let via_loader = render(&resolve_list_node(&DemoItemsLoader));
    let direct = render(&list_page(&demo_items()));
    assert_eq!(via_loader, direct);
}

/// 検証 1（詳細、正常系）: 既知 id の詳細解決が `detail_page(Some(_))` と
/// 一致する。
#[test]
fn resolve_detail_node_matches_direct_detail_page_call() {
    let via_loader = render(&resolve_detail_node(&DemoItemDetailLoader, "1"));
    let expected_item = demo_items().into_iter().find(|it| it.id == "1");
    let direct = render(&detail_page(expected_item.as_ref()));
    assert_eq!(via_loader, direct);
}

/// 検証 1（詳細、未知 id = 404 相当）: `Output = None` はエラー扱いにせず
/// `detail_page(None)` の固定文言のまま描画される（設計書 §3.3、見つからない
/// を `Error` ではなく `Output` の一部として表現する契約）。
#[test]
fn resolve_detail_node_matches_direct_call_for_unknown_id() {
    let via_loader = render(&resolve_detail_node(
        &DemoItemDetailLoader,
        "does-not-exist",
    ));
    let direct = render(&detail_page(None));
    assert_eq!(via_loader, direct);
}

/// 検証 2: 同一入力での反復呼び出しが完全一致する。
#[test]
fn resolve_list_node_is_deterministic_across_repeated_calls() {
    let first = render(&resolve_list_node(&DemoItemsLoader));
    let second = render(&resolve_list_node(&DemoItemsLoader));
    assert_eq!(first, second);
}

/// 検証 4: `demo_items()` の XSS ペイロード item（id="2"）が loader 経由でも
/// 既定エスケープされ、生の `<script>` タグが出力に現れない。
#[test]
fn resolve_detail_node_escapes_xss_payload_title() {
    let html = render(&resolve_detail_node(&DemoItemDetailLoader, "2"));
    assert!(
        !html.contains("<script>"),
        "XSS ペイロード item の title が既定エスケープされずに出力へ混入した: {html}"
    );
    assert!(
        html.contains("&lt;script&gt;"),
        "既定エスケープ済みの文字列が出力に含まれていること: {html}"
    );
}

/// 検証 3 用の失敗フィクスチャ（一覧）: `Error` に機微情報風文字列を
/// 含める（`server/src/ssr.rs::tests::FailingListLoader` と同じ意図）。
struct FailingListLoader;

impl Loader for FailingListLoader {
    type Input = ();
    type Output = Vec<Item>;
    type Error = String;

    fn load(&self, _input: &()) -> Result<Vec<Item>, String> {
        Err("secret://db-password@internal-host".to_string())
    }
}

/// 検証 3 用の失敗フィクスチャ（詳細）。
struct FailingDetailLoader;

impl Loader for FailingDetailLoader {
    type Input = String;
    type Output = Option<Item>;
    type Error = String;

    fn load(&self, _input: &String) -> Result<Option<Item>, String> {
        Err("secret://db-password@internal-host".to_string())
    }
}

/// 検証 3（一覧）: loader が失敗しても機微情報風文字列は出力へ混入せず、
/// [`loader_error_view`] の render 結果と一致する固定文言のみを返す。
#[test]
fn resolve_list_node_converts_loader_error_to_fixed_error_view_without_leaking_error_value() {
    let html = render(&resolve_list_node(&FailingListLoader));
    let expected = render(&loader_error_view());
    assert_eq!(html, expected);
    assert!(
        !html.contains("secret://db-password@internal-host"),
        "loader の Error 値（機微情報風文字列）が出力へ混入してはならない: {html}"
    );
    assert!(
        html.contains("Something went wrong"),
        "固定文言のエラービューを返すこと: {html}"
    );
}

/// 検証 3（詳細）: 上記と同様、詳細解決の失敗経路でも機微情報が漏れない。
#[test]
fn resolve_detail_node_converts_loader_error_to_fixed_error_view_without_leaking_error_value() {
    let html = render(&resolve_detail_node(&FailingDetailLoader, "1"));
    let expected = render(&loader_error_view());
    assert_eq!(html, expected);
    assert!(
        !html.contains("secret://db-password@internal-host"),
        "loader の Error 値（機微情報風文字列）が出力へ混入してはならない: {html}"
    );
}

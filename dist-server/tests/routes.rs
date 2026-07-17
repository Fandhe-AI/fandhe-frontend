//! `rws-dist-server` の公開 API（`routes::route_request`）に対する統合テスト。
//!
//! `src/routes.rs` 内のユニットテストと重複する観点も含むが、こちらは
//! クレート外部（`tests/`）から `pub` 経路のみを叩くことで、`route_request`
//! が公開契約として意図どおり使えることを固定する（内部実装のリファクタで
//! `pub` 境界を誤って壊した場合に検知する回帰テスト）。
//!
//! 起動（TCP bind・実プロセス）を伴う検証は TASK-9.1c（イシュー #97）の
//! スコープであり、本テストはハンドラレベル（`route_request` 呼び出し）に
//! 留める。

use rws_dist_server::routes::route_request;

#[test]
fn root_path_serves_list_page_with_escaped_xss_payload() {
    let response = route_request("/");
    assert_eq!(response.status, 200);
    let html = String::from_utf8(response.body).expect("HTML body is UTF-8");
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn known_item_id_returns_200_and_unknown_id_returns_404() {
    let ok = route_request("/items/2");
    assert_eq!(ok.status, 200);

    let not_found = route_request("/items/does-not-exist");
    assert_eq!(not_found.status, 404);
}

#[test]
fn static_asset_is_served_with_expected_content_type() {
    let response = route_request("/static/view-transitions.js");
    assert_eq!(response.status, 200);
    assert_eq!(response.content_type, "text/javascript; charset=utf-8");
}

#[test]
fn path_traversal_attempts_against_static_assets_return_404() {
    assert_eq!(route_request("/static/../Cargo.toml").status, 404);
    assert_eq!(route_request("/static/..%2FCargo.toml").status, 404);
}

#[test]
fn unmatched_path_returns_404() {
    assert_eq!(route_request("/totally/unknown/path").status, 404);
}

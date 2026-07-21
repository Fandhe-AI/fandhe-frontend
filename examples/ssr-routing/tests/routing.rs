//! `examples/ssr-routing` の integration test。
//!
//! `fandhe_frontend_server::ssr::respond_with` を独自 loader（`crates/src/main.rs`
//! と同型の固定データ）で直接検証し、一覧・詳細の 200/404・未一致パスの
//! `None`・既定エスケープ（REQ-1）の回帰を固定する。`src/main.rs` はバイナリ
//! クレートのため本ファイルからは `use` できず、loader・固定データを
//! テスト内に独立して定義する（依存クレートの公開 API のみを使う点で
//! 利用者向けサンプルとしての実演性は変わらない）。

use fandhe_frontend_app::{Item, Loader};
use fandhe_frontend_server::ssr::respond_with;
use std::convert::Infallible;

struct TestItemsLoader(Vec<Item>);

impl Loader for TestItemsLoader {
    type Input = ();
    type Output = Vec<Item>;
    type Error = Infallible;

    fn load(&self, _input: &()) -> Result<Vec<Item>, Infallible> {
        Ok(self.0.clone())
    }
}

struct TestItemDetailLoader(Vec<Item>);

impl Loader for TestItemDetailLoader {
    type Input = String;
    type Output = Option<Item>;
    type Error = Infallible;

    fn load(&self, id: &String) -> Result<Option<Item>, Infallible> {
        Ok(self.0.iter().find(|it| &it.id == id).cloned())
    }
}

fn fixture_items() -> Vec<Item> {
    vec![Item {
        id: "1".to_string(),
        title: "Example Item".to_string(),
        body: "Example body.".to_string(),
    }]
}

#[test]
fn respond_with_list_route_returns_200() {
    let items = fixture_items();
    let list_loader = TestItemsLoader(items.clone());
    let detail_loader = TestItemDetailLoader(items);

    let response = respond_with(&list_loader, &detail_loader, "/").expect("\"/\" should resolve");

    assert_eq!(response.status, 200);
    assert_eq!(response.content_type, "text/html; charset=utf-8");
}

#[test]
fn respond_with_known_detail_route_returns_200_with_title() {
    let items = fixture_items();
    let list_loader = TestItemsLoader(items.clone());
    let detail_loader = TestItemDetailLoader(items);

    let response = respond_with(&list_loader, &detail_loader, "/items/1")
        .expect("\"/items/1\" should resolve");

    assert_eq!(response.status, 200);
    assert!(
        response.body.contains("Example Item"),
        "body should contain the item title"
    );
}

#[test]
fn respond_with_unknown_id_returns_404() {
    let items = fixture_items();
    let list_loader = TestItemsLoader(items.clone());
    let detail_loader = TestItemDetailLoader(items);

    let response = respond_with(&list_loader, &detail_loader, "/items/999")
        .expect("\"/items/999\" should resolve to the detail route");

    assert_eq!(response.status, 404);
}

#[test]
fn respond_with_unmatched_path_returns_none() {
    let items = fixture_items();
    let list_loader = TestItemsLoader(items.clone());
    let detail_loader = TestItemDetailLoader(items);

    // `respond_with` の契約上、未一致パスは呼び出し側（`src/main.rs` の
    // `not_found_response`）が 404 応答を組み立てる責務を持つ。
    assert!(respond_with(&list_loader, &detail_loader, "/unknown").is_none());
}

/// 既定エスケープ回帰（REQ-1）: `<script>` を含む title が実体参照化されて
/// 出力され、生の `<script>` タグとしては現れないことを固定する
/// （`crates/server/tests/ssg_generic_routes.rs` のパターンに準拠）。
#[test]
fn respond_with_escapes_item_title() {
    let items = vec![Item {
        id: "1".to_string(),
        title: "<script>alert(1)</script>".to_string(),
        body: "escape regression fixture".to_string(),
    }];
    let list_loader = TestItemsLoader(items.clone());
    let detail_loader = TestItemDetailLoader(items);

    let response = respond_with(&list_loader, &detail_loader, "/items/1")
        .expect("\"/items/1\" should resolve");

    assert!(!response.body.contains("<script>alert"));
    assert!(response.body.contains("&lt;script&gt;"));
}

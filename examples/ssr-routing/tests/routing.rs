//! `examples/ssr-routing` の integration test。
//!
//! `fandhe_frontend_server::ssr::respond_with` を独自 loader（`examples/ssr-routing/src/main.rs`
//! と同型の固定データ）で直接検証し、一覧・詳細の 200/404・未一致パスの
//! `None`・既定エスケープ（REQ-1）の回帰を固定する。`src/main.rs` はバイナリ
//! クレートのため本ファイルからは `use` できず、loader・固定データを
//! テスト内に独立して定義する（依存クレートの公開 API のみを使う点で
//! 利用者向けサンプルとしての実演性は変わらない）。
//!
//! `hello_router` / `hello_response` / `resolve_response`（`src/main.rs` 内部
//! 関数、非公開）は上記と同じ理由で直接 `use` できないため、末尾の
//! `hello_route_*` / `cli_*` 系テストではビルド済みバイナリを
//! `env!("CARGO_BIN_EXE_<name>")`（cargo 標準機能、追加依存なし）で
//! サブプロセス起動し、標準出力を検証することで実際の CLI 経路
//! （`main` → `resolve_response` → `hello_router`/`hello_response` または
//! `respond_with` → `not_found_response`）をブラックボックスにカバーする。

use fandhe_frontend_app::{Item, Loader};
use fandhe_frontend_server::ssr::respond_with;
use std::convert::Infallible;
use std::process::Command;

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

/// `src/main.rs` のバイナリを引数付きで起動し、標準出力（ステータス行 +
/// `Content-Type:` 行 + 空行 + body）を丸ごと返す。
fn run_cli(path: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_fandhe-frontend-example-ssr-routing"))
        .arg(path)
        .output()
        .expect("binary should spawn and run to completion");
    assert!(output.status.success(), "CLI should always exit 0");
    String::from_utf8(output.stdout).expect("CLI stdout should be valid UTF-8")
}

/// `resolve_response` の判定順序 (1)（`hello_router`/`hello_response`）を
/// CLI 経由で固定する。独自ルート `/hello/:name` は `respond_with` の
/// 一覧・詳細画面表とは独立して解決される。
#[test]
fn hello_route_returns_200_with_greeting() {
    let stdout = run_cli("/hello/world");

    assert!(stdout.starts_with("200\n"), "stdout was: {stdout}");
    assert!(stdout.contains("Content-Type: text/html; charset=utf-8"));
    assert!(stdout.contains("Hello, world!"));
}

/// 既定エスケープ回帰（REQ-1）: `/hello/:name` の `name`（URL デコードされ
/// ていない生文字列、`Router::Params` の契約）が `text()` 経由でエスケープ
/// されて出力され、生の `<script>` タグとしては現れないことを CLI 経由で
/// 固定する。
#[test]
fn hello_route_escapes_name() {
    // `Command::arg` はシェルを経由しない生の argv 要素として渡すため、
    // `<script>` はメタ文字として解釈されず `name` パラメータへそのまま渡る
    // （`Router::Params` の「URL デコードされていない生文字列」契約の実演）。
    let stdout = run_cli("/hello/<script>");

    assert!(!stdout.contains("Hello, <script>"), "stdout was: {stdout}");
    assert!(stdout.contains("Hello, &lt;script&gt;!"));
}

/// `resolve_response` の判定順序 (2)（`respond_with` 経由の一覧画面）を
/// CLI 経由で固定する。引数省略時は `"/"` が既定パスになる（`main` の契約）。
#[test]
fn cli_default_path_returns_200_list_page() {
    let output = Command::new(env!("CARGO_BIN_EXE_fandhe-frontend-example-ssr-routing"))
        .output()
        .expect("binary should spawn and run to completion");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("CLI stdout should be valid UTF-8");
    assert!(stdout.starts_with("200\n"), "stdout was: {stdout}");
}

/// `resolve_response` の判定順序 (3)（いずれにも一致しない場合の
/// `not_found_response` フォールバック）を CLI 経由で固定する。
#[test]
fn cli_unmatched_path_returns_404() {
    let stdout = run_cli("/no-such-route");

    assert!(stdout.starts_with("404\n"), "stdout was: {stdout}");
    assert!(stdout.contains("The requested path was not found."));
}

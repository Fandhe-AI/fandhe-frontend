//! `fandhe-frontend-example-ssr-routing`: SSR + ルーティングの正本サンプル
//! （イシュー #499、examples 規約の初例）。
//!
//! # 役割・呼び出し文脈
//!
//! `templates/default` / `templates/app`（`fw new` が展開する「生成の雛形」）
//! とは異なり、本サンプルは crates.io へ公開済みの
//! `fandhe-frontend-core` / `fandhe-frontend-app` / `fandhe-frontend-server`
//! （いずれも v0.1.0）をバージョン依存として実際に使う「正本」であり、
//! 利用者・AI エージェントが SSR エントリを自作して構成がドリフトするのを
//! 防ぐための参照実装として存在する。以降 `examples/` に追加される各サンプル
//! も本ディレクトリの構成（`Cargo.toml` / `structure.toml` / `clippy.toml` /
//! `deny.toml` / `README.md` / `src/` / `tests/`）に従うことを想定する。
//!
//! CLI として `argv[1]`（省略時は `"/"`）をリクエストパスとみなし、
//! ステータス・Content-Type・HTML を標準出力へ書き出す。実行時のファイル
//! 読み書き・ネットワーク I/O は行わない（攻撃面を持たない）。
//!
//! # 学べること
//!
//! - `fandhe_frontend_app::Loader` trait の自作実装（`fandhe_frontend_app::DemoItemsLoader` /
//!   `DemoItemDetailLoader` への決め打ちを避けた最小サンプル）
//! - `fandhe_frontend_server::ssr::respond_with` による一覧・詳細画面の SSR 応答組み立て
//! - `fandhe_frontend_app::router::Router` を独自ルート（`/hello/:name`）に直接使う実演
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! HTML はすべて `fandhe_frontend_core` のノード木 API（`el` / `p` / `text` /
//! `fandhe_frontend_app::page_shell`）で組み立て、`format!` によるタグ文字列の
//! 直接組み立て・`raw_html()` は一切使わない。`Router::Params` は URL デコード
//! されていない生文字列を返す契約（`fandhe_frontend_app::router` rustdoc）であり、
//! `/hello/:name` の `name` は必ず `text()` 経由でノード木へ載せて既定
//! エスケープを通す（[`hello_response`] 参照）。

#![forbid(unsafe_code)]

use fandhe_frontend_app::router::Router;
use fandhe_frontend_app::{page_shell, Item, Loader};
use fandhe_frontend_core::{el, p, text};
use fandhe_frontend_server::ssr::respond_with;
use std::convert::Infallible;

/// 一覧画面（`/`）向けの loader 実装。
///
/// `fandhe_frontend_app::DemoItemsLoader` は使わず、本サンプル専用の固定データ
/// （[`example_items`]）を返す最小実装として自前定義する（利用者が自身の
/// データソースに差し替える際の雛形）。固定データの解決は失敗しないため
/// `Error = Infallible` とする。
struct ExampleItemsLoader;

impl Loader for ExampleItemsLoader {
    type Input = ();
    type Output = Vec<Item>;
    type Error = Infallible;

    fn load(&self, _input: &()) -> Result<Vec<Item>, Infallible> {
        Ok(example_items())
    }
}

/// 詳細画面（`/items/:id`）向けの loader 実装。
///
/// id が [`example_items`] に存在しない場合は `Output = None` を返す
/// （404 相当を `Error` ではなく `Output` の一部として表現する契約は
/// `fandhe_frontend_app::Loader` rustdoc §3.3 に従う）。
struct ExampleItemDetailLoader;

impl Loader for ExampleItemDetailLoader {
    type Input = String;
    type Output = Option<Item>;
    type Error = Infallible;

    fn load(&self, id: &String) -> Result<Option<Item>, Infallible> {
        Ok(example_items().into_iter().find(|it| &it.id == id))
    }
}

/// 本サンプル専用の固定データ。2 件目のタイトルへ意図的な XSS ペイロードを
/// 含め、`tests/routing.rs` の既定エスケープ回帰テスト（REQ-1）の入力にも
/// 使う（`fandhe_frontend_app::demo_items()` の実証パターンを踏襲）。
fn example_items() -> Vec<Item> {
    vec![
        Item {
            id: "1".to_string(),
            title: "SSR + ルーティング正本サンプルへようこそ".to_string(),
            body:
                "fandhe_frontend_server::ssr::respond_with と fandhe_frontend_app::router::Router \
                の実演です。"
                    .to_string(),
        },
        Item {
            id: "2".to_string(),
            title: "<script>alert('xss')</script>".to_string(),
            body: "このタイトルは既定エスケープ（REQ-1）を実演する意図的な XSS ペイロードです。"
                .to_string(),
        },
    ]
}

/// `/hello/:name` を単独ルーターとして持つ [`Router`] を組み立てる。
///
/// `respond_with`（`fandhe_frontend_app::routes` の単一ルート表）が関知しない
/// 独自ルートを追加したい呼び出し元向けの実演であり、パターン文字列は
/// 静的リテラルのため `route()` の `Result` は `expect` で確定的に展開する
/// （`RouterError` はパターン記述ミスのみを表す開発時エラーであり、
/// 実行時入力には依存しない）。
fn hello_router() -> Router<()> {
    Router::new()
        .route("/hello/:name", ())
        .expect("\"/hello/:name\" is a statically valid route pattern")
}

/// `/hello/:name` 一致時の 200 応答を組み立てる。
///
/// `params.get(\"name\")` は [`Router`] の契約どおり URL デコードされていない
/// 生文字列であり、HTML への出力は必ず [`text`] 経由（既定エスケープ）で行う。
/// `format!` はタグ文字列の組み立てには使わず、`text()` へ渡す前のプレーン
/// テキスト整形にのみ使う（`coding-rust.md`「HTML 文字列の直接組み立て禁止」
/// の対象外）。
fn hello_response(name: &str) -> (u16, &'static str, String) {
    let body = page_shell(
        "Hello",
        el(
            "main",
            vec![],
            vec![p(vec![], vec![text(format!("Hello, {name}!"))])],
        ),
    );
    (200, "text/html; charset=utf-8", body)
}

/// `respond_with` が `None`（未一致パス）を返した場合の固定文言 404 応答。
///
/// `fandhe_frontend_server::ssr::respond_with` の契約上、未一致パスは呼び出し側が
/// 404 応答を組み立てる責務を持つ（`crates/server/src/ssr.rs` rustdoc）。
/// 内部パス・スタックトレース等は含めない（fail-closed、`security.md`
/// 「機微情報の露出」）。
fn not_found_response() -> (u16, &'static str, String) {
    let body = page_shell(
        "Not Found",
        el(
            "main",
            vec![],
            vec![p(vec![], vec![text("The requested path was not found.")])],
        ),
    );
    (404, "text/html; charset=utf-8", body)
}

/// リクエストパスをステータス・Content-Type・HTML 文字列へ解決する。
///
/// 判定順序: (1) [`hello_router`]（独自ルート実演）→ (2) `respond_with`
/// （一覧・詳細画面、`fandhe_frontend_app::routes` の単一ルート表）→
/// (3) [`not_found_response`]（いずれにも一致しない場合の 404 フォールバック）。
fn resolve_response(path: &str) -> (u16, &'static str, String) {
    if let Some(route_match) = hello_router().resolve(path) {
        let name = route_match.params.get("name").unwrap_or("world");
        return hello_response(name);
    }

    match respond_with(&ExampleItemsLoader, &ExampleItemDetailLoader, path) {
        Some(response) => (response.status, response.content_type, response.body),
        None => not_found_response(),
    }
}

/// CLI エントリポイント。`argv[1]`（省略時は `"/"`）をリクエストパスとして
/// [`resolve_response`] を呼び、結果を標準出力へ書き出す。404 応答も正常な
/// SSR 結果として扱い、終了コードは常に 0。
fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "/".to_string());
    let (status, content_type, body) = resolve_response(&path);

    println!("{status}");
    println!("Content-Type: {content_type}");
    println!();
    println!("{body}");
}

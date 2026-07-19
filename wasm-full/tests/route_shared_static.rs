//! server ↔ wasm-full のルート定義単一化の静的検証（イシュー #407）。
//!
//! 従来の `route_sync_static.rs`（イシュー #374）は「両ファイルに同じ
//! パターン・タイトルリテラルが存在する」ことのみを検知するドリフト
//! **検知**テストだったが、本テストは検知対象を弱体化させず**上回る**形へ
//! 置き換える（受け入れ条件 2）:
//!
//! 1. **単一定義の強制**: `server/src/ssr.rs`・`wasm-full/src/nav.rs`
//!    いずれにもルートパターンリテラル（`"/"`・`"/items/:id"`）・ページ
//!    タイトルリテラル（`"記事一覧"`・`"記事詳細"`）が**再定義されていない**
//!    こと、双方が `fandhe_frontend_app::routes` の共有 API を参照していることを固定する
//!    （従来はリテラルの**存在**確認だったが、本テストは**非再定義**を固定する
//!    ことでドリフトそのものを構造的に不可能にする）。
//! 2. **意味論の直接固定**: `fandhe_frontend_app::routes::resolve` に対する v1 仕様
//!    ベクトルテストは `app/src/routes.rs` の `#[cfg(test)]`（クエリ除去・
//!    末尾スラッシュ厳格一致・空セグメント拒否・`:id` 捕捉・XSS ペイロード風
//!    パス）が担う。server / wasm-full は同一エンジン
//!    （[`fandhe_frontend_app::router::Router`]）・同一ルート表を共有するため、
//!    従来のような「2 実装の意味論が一致するか」の等価性テストは不要（B-1、
//!    `docs/design/route-definition-sharing.md`）。
//! 3. **タイトル整合**: SSR 出力（`server/src/ssr.rs::respond_with`）と
//!    クライアント遷移（`wasm-full/src/nav.rs::resolve_route_view_with`）が
//!    同一ルートで同一タイトルを返すことは、双方が
//!    `fandhe_frontend_app::routes::title` を直接呼ぶ構造（検証 1）そのものが保証する。
//!    `wasm-full/tests/nav_native.rs`（無修正）・`server/src/ssr.rs` の
//!    既存テスト（無修正）が実際の戻り値を固定する。

use std::fs;
use std::path::{Path, PathBuf};

/// workspace ルート（`wasm-full/` の親ディレクトリ）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("wasm-full/ は workspace ルート直下に存在する前提")
        .to_path_buf()
}

/// 行コメント（`//`）・ブロックコメント（`/* */`）を除去する簡易フィルタ。
/// `core/tests/no_branching_across_modes.rs::strip_comments` の複製
/// （integration test はファイル間でコード共有できないため複製する。
/// 複製理由の詳細は同ファイルの `collect_rs_files` コメント参照）。
fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'/') {
            for c2 in chars.by_ref() {
                if c2 == '\n' {
                    out.push('\n');
                    break;
                }
            }
        } else if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut prev = ' ';
            for c2 in chars.by_ref() {
                if prev == '*' && c2 == '/' {
                    break;
                }
                prev = c2;
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn read_stripped(path: &Path) -> String {
    let src =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("{path:?} の読み取りに失敗した: {e}"));
    strip_comments(&src)
}

/// 単一定義（イシュー #407）の対象となるルートパターンリテラル。
/// `app/src/routes.rs` にのみ存在してよく、`server/src/ssr.rs`・
/// `wasm-full/src/nav.rs` には再定義されていてはならない。
const ROUTE_PATTERN_LITERALS: &[&str] = &["\"/\"", "\"/items/:id\""];

/// 単一定義の対象となるページタイトルリテラル。同様に `app/src/routes.rs`
/// にのみ存在してよい。
const PAGE_TITLE_LITERALS: &[&str] = &["\"記事一覧\"", "\"記事詳細\""];

/// 検証 1a: ルートパターン・ページタイトルの単一定義元である
/// `app/src/routes.rs` に、期待どおりのリテラルが実在すること（本テストの
/// 前提が崩れていないことの自己チェック）。
#[test]
fn app_routes_defines_the_shared_route_table_literals() {
    let root = workspace_root();
    let routes_path = root.join("app/src/routes.rs");
    let stripped = read_stripped(&routes_path);

    for pattern in ROUTE_PATTERN_LITERALS {
        assert!(
            stripped.contains(&format!(".route({pattern}")),
            "{routes_path:?} に `.route({pattern}` が見つからない（単一定義元の前提が崩れている）"
        );
    }
    for title in PAGE_TITLE_LITERALS {
        assert!(
            stripped.contains(title),
            "{routes_path:?} にタイトルリテラル {title} が見つからない（単一定義元の前提が崩れている）"
        );
    }
}

/// 検証 1b（受け入れ条件 2 の核心）: `server/src/ssr.rs` にルートパターン・
/// ページタイトルリテラルが**再定義されていない**こと、かつ
/// `fandhe_frontend_app::routes` を参照していることを固定する。
#[test]
fn server_ssr_does_not_redefine_route_literals_and_references_shared_routes() {
    let root = workspace_root();
    let ssr_path = root.join("server/src/ssr.rs");
    let stripped = read_stripped(&ssr_path);

    for pattern in ROUTE_PATTERN_LITERALS {
        assert!(
            !stripped.contains(&format!(".route({pattern}")),
            "{ssr_path:?} にルートパターンリテラル `.route({pattern}` が再定義されている。\
             fandhe_frontend_app::routes の単一定義（イシュー #407）に反する"
        );
    }
    for title in PAGE_TITLE_LITERALS {
        assert!(
            !stripped.contains(title),
            "{ssr_path:?} にページタイトルリテラル {title} が再定義されている。\
             fandhe_frontend_app::routes::title の単一定義（イシュー #407）に反する"
        );
    }
    assert!(
        stripped.contains("fandhe_frontend_app::routes"),
        "{ssr_path:?} が fandhe_frontend_app::routes を参照していない（共有ルート定義を経由していない）"
    );
}

/// 検証 1c: `wasm-full/src/nav.rs` も同様に、ルートパターン・タイトル
/// リテラルを再定義せず `fandhe_frontend_app::routes` を参照していることを固定する。
#[test]
fn wasm_full_nav_does_not_redefine_route_literals_and_references_shared_routes() {
    let root = workspace_root();
    let nav_path = root.join("wasm-full/src/nav.rs");
    let stripped = read_stripped(&nav_path);

    for pattern in ROUTE_PATTERN_LITERALS {
        assert!(
            !stripped.contains(&format!(".route({pattern}")),
            "{nav_path:?} にルートパターンリテラル `.route({pattern}` が再定義されている。\
             fandhe_frontend_app::routes の単一定義（イシュー #407）に反する"
        );
    }
    for title in PAGE_TITLE_LITERALS {
        assert!(
            !stripped.contains(title),
            "{nav_path:?} にページタイトルリテラル {title} が再定義されている。\
             fandhe_frontend_app::routes::title の単一定義（イシュー #407）に反する"
        );
    }
    assert!(
        stripped.contains("fandhe_frontend_app::routes"),
        "{nav_path:?} が fandhe_frontend_app::routes を参照していない（共有ルート定義を経由していない）"
    );
}

/// 検証 2: server・wasm-full 双方のルーティングエンジンが同一クレート
/// （`fandhe_frontend_app::router::Router`、`app/src/routes.rs` 経由）に収束しており、
/// `fandhe-frontend-server` 独自の `Router` 実体（旧 `server/src/router.rs`）が
/// 復活していないこと。`server/src/router.rs` は再エクスポートのみを
/// 許容し、独自の `struct Router` 定義（`pub struct Router<H>` 等）を
/// 持たないことを固定する。
#[test]
fn server_router_module_is_a_reexport_shim_not_a_duplicate_engine() {
    let root = workspace_root();
    let router_path = root.join("server/src/router.rs");
    let stripped = read_stripped(&router_path);

    assert!(
        !stripped.contains("struct Router"),
        "{router_path:?} に `struct Router` の独自定義が存在する。\
         エンジンは fandhe_frontend_app::router へ一本化されている必要がある（イシュー #407 案 B-1）"
    );
    assert!(
        stripped.contains("pub use fandhe_frontend_app::router"),
        "{router_path:?} が fandhe_frontend_app::router を再エクスポートしていない"
    );
}

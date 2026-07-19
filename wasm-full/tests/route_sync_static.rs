//! server ↔ wasm-full のルート定義ドリフト検知（イシュー #374）。
//!
//! `rws-wasm-full` は `rws-server` へ依存できない（`structure.toml` の
//! `server.allowed_dependents = ["dist-server"]`）ため、`wasm-full/src/nav.rs`
//! はクライアント側のルート表を独自に定義する（`server/src/ssr.rs` の
//! `Router` とは別実装）。本テストは両者のルートパターンリテラル・
//! ページタイトルリテラルが乖離しないことを `std::fs` のみで workspace 内
//! ソースを走査する**静的解析テスト**として固定する
//! （`core/tests/no_branching_across_modes.rs` と同方式）。
//!
//! `fw structure` の `rws-router-v1` 抽出器（`cli/src/routes.rs`）は
//! `server/` 内の `.route("<literal>")` 文字列リテラルを走査する実装のため、
//! ルートパターンを定数へ括り出して両クレートから共有する方式は取れない
//! （#374 実装計画 §2「現状調査結果」参照）。本テストがドリフト防止の代替
//! 機構を担う。

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

/// `server/src/ssr.rs::build_page_router()` が登録するルートパターン
/// （`.route("<pattern>")` の第 1 引数リテラル）。
const SERVER_ROUTE_PATTERNS: &[&str] = &["/", "/items/:id"];

/// `server/src/ssr.rs::respond_with` が使うページタイトルリテラル。
const SERVER_PAGE_TITLES: &[&str] = &["記事一覧", "記事詳細"];

/// 検証 1: `server/src/ssr.rs` に [`SERVER_ROUTE_PATTERNS`] の各パターンが
/// `.route("<pattern>")` の形で登録されていることを確認する（本テスト自体が
/// server 側の実装を変更しても追従できるよう、期待値をハードコードしすぎず
/// 「両ファイルに同じリテラルが存在する」ことを主眼に置く）。
#[test]
fn server_route_patterns_are_registered_as_expected() {
    let root = workspace_root();
    let ssr_path = root.join("server/src/ssr.rs");
    let stripped = read_stripped(&ssr_path);

    for pattern in SERVER_ROUTE_PATTERNS {
        let needle = format!(".route(\"{pattern}\"");
        assert!(
            stripped.contains(&needle),
            "{ssr_path:?} に `{needle}` が見つからない。SERVER_ROUTE_PATTERNS の \
             前提が崩れている（本テスト自体の更新が必要な可能性がある）"
        );
    }
}

/// 検証 2: `wasm-full/src/nav.rs` のルート解決（`resolve_path`）が
/// [`SERVER_ROUTE_PATTERNS`] と同じ 2 ルート（`/`・`/items/:id` 相当の
/// セグメント構造）を認識することを確認する。パターン文字列そのものを
/// 走査するのではなく、`nav.rs` 内の `match segments.as_slice()` 分岐に
/// 対応するリテラルが両方存在することを固定する（`nav.rs` は `Router` の
/// パターン文字列 DSL を持たないため、server 側と同じ文字列走査はできない）。
#[test]
fn nav_route_table_covers_the_same_two_routes_as_server() {
    let root = workspace_root();
    let nav_path = root.join("wasm-full/src/nav.rs");
    let stripped = read_stripped(&nav_path);

    // `/` 相当: `ClientRoute::List` を返す分岐が存在すること。
    assert!(
        stripped.contains("ClientRoute::List"),
        "{nav_path:?} に `ClientRoute::List`（`/` 相当のルート）への解決が \
         見つからない。server 側の `/` ルート登録とドリフトしている"
    );
    // `/items/:id` 相当: `["items", id]` セグメント一致で `Detail` を返す
    // 分岐が存在すること。
    assert!(
        stripped.contains("[\"items\", id]") && stripped.contains("ClientRoute::Detail"),
        "{nav_path:?} に `/items/:id` 相当のセグメント一致（[\"items\", id] → \
         ClientRoute::Detail）が見つからない。server 側の `/items/:id` \
         ルート登録とドリフトしている"
    );
}

/// 検証 3: `server/src/ssr.rs`・`wasm-full/src/nav.rs` の双方に
/// [`SERVER_PAGE_TITLES`] の各タイトルリテラルが出現することを確認する
/// （`<title>` 相当の表示文言が SSR とクライアント遷移で食い違わないことの
/// 固定、受け入れ条件 4「三モード整合」の一部）。
#[test]
fn page_titles_are_shared_literally_between_server_and_nav() {
    let root = workspace_root();
    let ssr_path = root.join("server/src/ssr.rs");
    let nav_path = root.join("wasm-full/src/nav.rs");

    for path in [&ssr_path, &nav_path] {
        let stripped = read_stripped(path);
        for title in SERVER_PAGE_TITLES {
            assert!(
                stripped.contains(title),
                "{path:?} にページタイトルリテラル {title:?} が見つからない。\
                 server ↔ wasm-full 間でタイトル文言がドリフトしている（受け入れ条件 4）"
            );
        }
    }
}

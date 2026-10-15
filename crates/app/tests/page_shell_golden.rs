//! `page_shell` 出力のバイト一致固定テスト（イシュー #3819）。
//!
//! `page_shell` を `Document` 経由の実装へ置き換えた際に出力が 1 バイトも変わらないことの証人。
//! 期待値は置き換え前の実装が返した文字列をそのままリテラル化したもので、`Document` 側の
//! 変更に追随して書き換えてはならない（SSR / SSG / dist-server が同一出力を前提とする）。
use fandhe_frontend_app::{demo_items, detail_page, list_page, page_shell};
use fandhe_frontend_core::{p, text};

#[test]
fn list_page_shell_is_byte_identical() {
    let expected = "<!DOCTYPE html>\n<html lang=\"ja\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><style>@view-transition { navigation: auto; }</style><title>記事一覧</title><link rel=\"stylesheet\" href=\"/static/style.css\"></head><body><div id=\"app-root\" data-fandhe-frontend=\"root\"><h1>記事一覧</h1><main><ul data-testid=\"item-list\"><li><a href=\"/items/1\" data-nav=\"/items/1\">Rust 製フロントエンド基盤の構想</a></li><li><a href=\"/items/2\" data-nav=\"/items/2\">&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;&lt;img src=x onerror=alert(1)&gt;</a></li><li><a href=\"/items/3\" data-nav=\"/items/3\">View Transitions API の薄いラッパー評価</a></li></ul></main></div><script type=\"module\" src=\"/static/hydrate.js\"></script></body></html>";
    assert_eq!(page_shell("記事一覧", list_page(&demo_items())), expected);
}

#[test]
fn detail_not_found_page_shell_is_byte_identical() {
    let expected = "<!DOCTYPE html>\n<html lang=\"ja\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><style>@view-transition { navigation: auto; }</style><title>記事</title><link rel=\"stylesheet\" href=\"/static/style.css\"></head><body><div id=\"app-root\" data-fandhe-frontend=\"root\"><h1>見つかりません</h1><main><p>指定された記事は存在しません。</p></main></div><script type=\"module\" src=\"/static/hydrate.js\"></script></body></html>";
    assert_eq!(page_shell("記事", detail_page(None)), expected);
}

#[test]
fn escaped_title_page_shell_is_byte_identical() {
    let expected = "<!DOCTYPE html>\n<html lang=\"ja\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><style>@view-transition { navigation: auto; }</style><title>a&lt;&gt;&amp;&#x27;&quot;日本</title><link rel=\"stylesheet\" href=\"/static/style.css\"></head><body><p>x&amp;y</p><script type=\"module\" src=\"/static/hydrate.js\"></script></body></html>";
    assert_eq!(
        page_shell("a<>&'\"日本", p(vec![], vec![text("x&y")])),
        expected
    );
}

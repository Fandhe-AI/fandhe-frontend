//! トップページに表示する SSR コード例（`snippets/landing_ssr.rs`、イシュー #3615）の
//! コンパイル・実行担保。
//!
//! `landing.rs` は同じファイルを `include_str!` で表示に使い、本テストは
//! `#[path]` で実モジュールとしてコンパイルする。表示と検証が同一ファイルなので
//! ドリフトせず、app / core の API 変更はここのコンパイルエラーで検知される。

#[path = "../snippets/landing_ssr.rs"]
mod snippet;

#[test]
fn snippet_renders_escaped_document() {
    let html = snippet::hello_page("<script>alert(1)</script>");
    assert!(html.starts_with("<!DOCTYPE html>"), "{html}");
    assert!(html.contains("Hello, &lt;script&gt;alert(1)&lt;/script&gt;!"));
    assert!(!html.contains("<script>alert"));
}

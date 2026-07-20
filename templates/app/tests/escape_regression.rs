//! `templates/app`（`fw new --template app` 拡張プロジェクトテンプレート、
//! イシュー #378）の XSS 回帰テスト。
//!
//! REQ-1（既定エスケープ）の実証: `fandhe_frontend_app::demo_items()`（crates.io
//! バージョン依存、イシュー #412 で vendor 同梱から切替）は `items()[1]` に意図的な XSS ペイロード
//! （`<script>alert('xss')</script>` 等）を含む。本テストは `list_page` /
//! `detail_page` を経由して `fandhe_frontend_core::render` した出力にそのペイロードが
//! 生タグとして現れないことを固定する。`fw gate` の `test` チェック
//! （`cargo test -p fandhe-frontend-template-app`）が常時実行する回帰テストであり、
//! `.claude/rules/coding-rust.md`「XSS 回帰テストは削除・弱体化しない」を
//! 本プロジェクトテンプレートでも踏襲する。

use fandhe_frontend_app::{demo_items, detail_page, list_page};
use fandhe_frontend_core::render;

/// PoC-2/PoC-3 由来の意図的な XSS ペイロード（`app/src/lib.rs::demo_items`）
/// が `list_page` 経由でエスケープされずに出力へ混入しないことを確認する。
#[test]
fn list_page_escapes_xss_payload_in_demo_items() {
    let items = demo_items();
    let html = render(&list_page(&items));
    assert!(
        !html.contains("<script>"),
        "生スクリプトタグがエスケープされずに list_page の出力へ混入した: {html}"
    );
    assert!(
        html.contains("&lt;script&gt;"),
        "list_page の出力にエスケープ済みペイロードが見つからない: {html}"
    );
}

/// 同じペイロードを持つ項目を `detail_page` 経由で描画した場合も
/// エスケープが維持されることを確認する（`list_page` とは別の描画経路）。
#[test]
fn detail_page_escapes_xss_payload_in_demo_items() {
    let items = demo_items();
    let payload_item = items
        .iter()
        .find(|it| it.title.contains("<script>"))
        .expect("demo_items() には XSS ペイロードを含む項目が含まれる前提");

    let html = render(&detail_page(Some(payload_item)));
    assert!(
        !html.contains("<script>alert"),
        "生スクリプトタグがエスケープされずに detail_page の出力へ混入した: {html}"
    );
    assert!(
        html.contains("&lt;script&gt;"),
        "detail_page の出力にエスケープ済みペイロードが見つからない: {html}"
    );
}

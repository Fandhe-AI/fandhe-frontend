//! `rws-wasm-client` 側の三モード（SSR/SSG/CSR）整合テスト（イシュー #375）。
//!
//! # 責務境界（`server/tests/three_mode_integration.rs` との役割分担）
//!
//! `server/tests/three_mode_integration.rs`（TASK-6.1d、#45）は SSR/SSG が
//! バイト一致すること、および「CSR を模した直接呼び出し」
//! （`rws_core::render(&rws_app::list_page(...))`）が SSR 出力へ埋め込まれる
//! ことを固定するが、実際の `rws-wasm-client` の公開関数
//! （[`rws_wasm_client::render_list_page_html`] 等）は使っていない
//! （同ファイルのコメント「CSR を模した直接呼び出し（rws-wasm-client 相当）」
//! 参照）。
//!
//! 本ファイルは、イシュー #375 で `rws-wasm-client` を `rws_app::Loader`
//! 経由へ移行したことに伴い、**実際の `rws-wasm-client` 公開関数**を SSR
//! （[`rws_server::ssr::respond`]）・SSG（[`rws_server::ssg::generate`]）の
//! 出力と native で直接突き合わせる。`rws-server` を dev-dependency として
//! 追加している（`wasm-client/Cargo.toml` 参照。ワークスペース内 path 依存
//! かつ `rws-server` 自体は外部依存ゼロのため REQ-3 の依存グラフ計測
//! （Normal のみ対象）に影響しない）。
//!
//! 実ブラウザでの三モード検証は本ファイルの対象外とする
//! （`wasm-full/tests/three_mode_browser.rs` が実 DOM 経路を別途カバー済み。
//! `wasm-client` 側は native バイト一致で契約を固定する方針を
//! `docs/design/loader-trait-design.md` に記録する）。

use rws_server::ssg::generate;
use rws_server::ssr::respond;
use rws_wasm_client::{render_detail_page_html, render_list_page_html};
use std::fs;

// `server/tests/support/temp_dir.rs` と同型のヘルパー（別クレートのため
// `#[cfg(test)]` アイテムを共有できず複製、`wasm-client/tests/support/temp_dir.rs`
// 参照）。
include!("support/temp_dir.rs");

/// REQ-6 中核: `rws-wasm-client` の CSR 用一覧 HTML（[`render_list_page_html`]）
/// が SSR（`respond("/")`）の応答本文にそのまま埋め込まれていること。
///
/// `render_list_page_html()` は `page_shell` で包む前の `list_page` 単体の
/// レンダリング結果であり、SSR 側は `page_shell` でラップされた全体 HTML を
/// 返すため（`app/src/lib.rs::page_shell`）、両者は完全一致ではなく
/// 部分文字列一致で検証する（`server/tests/three_mode_integration.rs` と
/// 同じ比較方式）。
#[test]
fn wasm_client_list_html_is_embedded_in_ssr_output() {
    let csr_html = render_list_page_html();
    let ssr_response = respond("/").expect("\"/\" should match");
    assert_eq!(ssr_response.status, 200);
    assert!(
        ssr_response.body.contains(&csr_html),
        "SSR body should embed the exact rws-wasm-client render_list_page_html() output"
    );
}

/// 詳細ページ版。`rws_app::demo_items()` の全 id について、CSR
/// （[`render_detail_page_html`]）の出力が SSR 応答本文へそのまま埋め込まれる
/// ことを固定する。
#[test]
fn wasm_client_detail_html_is_embedded_in_ssr_output_for_every_item() {
    for item in rws_app::demo_items() {
        let csr_html = render_detail_page_html(&item.id);
        let ssr_response =
            respond(&format!("/items/{}", item.id)).expect("item detail should match");
        assert_eq!(ssr_response.status, 200);
        assert!(
            ssr_response.body.contains(&csr_html),
            "SSR detail body for id {:?} should embed the exact rws-wasm-client \
             render_detail_page_html() output",
            item.id
        );
    }
}

/// REQ-6: SSG（`generate`）が書き出す静的ファイルにも、`rws-wasm-client` の
/// CSR HTML がそのまま埋め込まれていること（SSR/SSG がバイト一致する契約
/// （`server/tests/ssr_ssg_parity.rs`）と組み合わせ、三モードすべてが同一
/// データ源に接続されていることを固定する）。
#[test]
fn wasm_client_html_is_embedded_in_ssg_output() {
    let dir = TempDir::new("three-mode");
    generate(&dir.0).expect("generate should succeed");

    let index_html = fs::read_to_string(dir.0.join("index.html")).expect("index.html should exist");
    assert!(index_html.contains(&render_list_page_html()));

    for item in rws_app::demo_items() {
        let detail_html = fs::read_to_string(dir.0.join("items").join(&item.id).join("index.html"))
            .expect("item detail file should exist");
        assert!(detail_html.contains(&render_detail_page_html(&item.id)));
    }
}

/// REQ-1 三経路回帰: `demo_items()[1]`（意図的な XSS ペイロード、id "2"）が
/// SSR・SSG・CSR（`rws-wasm-client`）のいずれの経路でも既定エスケープされ、
/// 生の `<script>`/`onerror` として出力されないこと。
#[test]
fn xss_payload_is_escaped_across_ssr_ssg_and_wasm_client_csr_paths() {
    let dir = TempDir::new("three-mode-xss");
    generate(&dir.0).expect("generate should succeed");

    // CSR（rws-wasm-client 実関数）。`<img src=x onerror=alert(1)>` は
    // エスケープ後も属性文字列としての "onerror=alert" 自体は残る（無害な
    // テキストとして表示されるのみ）ため、タグ自体が生の `<img` として出現
    // しないことを検証する（`<` がエスケープされていれば `<img` は現れない）。
    let csr_list_html = render_list_page_html();
    assert!(!csr_list_html.contains("<script>alert"));
    assert!(csr_list_html.contains("&lt;script&gt;alert"));
    assert!(!csr_list_html.contains("<img src=x onerror"));

    let csr_detail_html = render_detail_page_html("2");
    assert!(!csr_detail_html.contains("<script>alert"));
    assert!(csr_detail_html.contains("&lt;script&gt;alert"));
    assert!(!csr_detail_html.contains("<img src=x onerror"));

    // SSR。
    let ssr_response = respond("/items/2").expect("\"/items/2\" should match");
    assert!(!ssr_response.body.contains("<script>alert"));
    assert!(ssr_response.body.contains("&lt;script&gt;alert"));
    assert!(!ssr_response.body.contains("<img src=x onerror"));

    // SSG。
    let ssg_detail = fs::read_to_string(dir.0.join("items").join("2").join("index.html"))
        .expect("item 2 detail file should exist");
    assert!(!ssg_detail.contains("<script>alert"));
    assert!(ssg_detail.contains("&lt;script&gt;alert"));
    assert!(!ssg_detail.contains("<img src=x onerror"));

    // 三経路がすべて同一の CSR フラグメントを含むこと（データ源の単一性）。
    assert!(ssr_response.body.contains(&csr_detail_html));
    assert!(ssg_detail.contains(&csr_detail_html));
}

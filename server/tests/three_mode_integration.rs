//! 三モード統合テスト（TASK-6.1d、イシュー #45）。
//!
//! REQ-6（単一コアによる SSR/SSG/CSR 描画）の受け入れ基準を、rws-server の
//! 公開 API（[`rws_server::ssr::respond`] / [`rws_server::ssg::generate`]）と
//! rws-app のコンポーネント関数（[`rws_app::list_page`] 等 + `rws_core::render`）
//! を実際に組み合わせて固定する。
//!
//! # スコープ（`docs/app-api.md` 第 9 節参照）
//!
//! - 実ブラウザでの `mount_csr` / ハイドレーション実証は TASK-6.2（#46）・
//!   TASK-6.3 のスコープであり、本テストでは扱わない。ここでの「CSR 同一
//!   関数性」検証は、`rws_core::render(&list_page(...))` を SSR とは別に
//!   呼び出しても同一関数・同一入力である限り同一 HTML が得られることの
//!   固定（モード非依存契約の直接証明）に限定する。
//! - SSR/SSG 完全一致の CI 回帰拡充（`server/tests/ssr_ssg_parity.rs`）は
//!   TASK-6.4（#50）のスコープであり別イシューで扱う。本テストは
//!   TASK-6.1d の受け入れ基準（三モード統合の最小固定）のみを担う。

use rws_app::{demo_items, detail_page, list_page, page_shell};
use rws_core::render;
use rws_server::ssg::generate;
use rws_server::ssr::respond;
use std::fs;
use std::path::PathBuf;

/// `tempfile` 等の外部クレートを追加せず、`std::env::temp_dir()` +
/// プロセス固有サフィックスで一時ディレクトリを代用する
/// （REQ-3: `rws-server` は外部依存ゼロを維持する）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "rws-server-three-mode-test-{tag}-{}-{unique}",
            std::process::id()
        ));
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // 後片付け失敗はテスト失敗にしない（一時ディレクトリの残留は
        // テスト結果の正当性に影響しない）。
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// REQ-6 受け入れ基準 1: SSR（`respond`）と SSG（`generate`）が同一ボディを
/// 出力すること（バイト一致）。
#[test]
fn ssr_and_ssg_bodies_match_byte_for_byte() {
    let dir = TempDir::new("parity");
    generate(&dir.0).expect("generate should succeed");

    let ssr_index = respond("/").expect("\"/\" should match").body;
    let ssg_index = fs::read_to_string(dir.0.join("index.html")).expect("index.html should exist");
    assert_eq!(ssr_index, ssg_index);

    for item in demo_items() {
        let ssr_detail = respond(&format!("/items/{}", item.id))
            .expect("item detail should match")
            .body;
        let ssg_detail = fs::read_to_string(dir.0.join("items").join(&item.id).join("index.html"))
            .expect("item detail file should exist");
        assert_eq!(ssr_detail, ssg_detail);
    }
}

/// REQ-6 受け入れ基準 2: CSR も同一関数（`list_page`/`detail_page`）を
/// 呼び出すことで、SSR 出力の一部として同一 HTML が現れること
/// （モード非依存契約: 同一入力なら常に同一 `Node` 木 → 同一 HTML）。
#[test]
fn csr_style_direct_calls_produce_html_embedded_in_ssr_output() {
    let items = demo_items();

    // CSR を模した直接呼び出し（rws-wasm-client 相当。実ブラウザでの
    // mount_csr 実証は TASK-6.2 のスコープ、上記モジュール doc 参照）。
    let csr_list_html = render(&list_page(&items));
    let ssr_response = respond("/").expect("\"/\" should match");
    assert!(
        ssr_response.body.contains(&csr_list_html),
        "SSR page_shell body should embed the exact same list_page HTML that a CSR caller would render"
    );

    // 反復呼び出しの完全一致（副作用・グローバル状態を持たないことの確認）。
    let csr_list_html_again = render(&list_page(&items));
    assert_eq!(csr_list_html, csr_list_html_again);

    let item = items.iter().find(|it| it.id == "1");
    let csr_detail_html = render(&detail_page(item));
    let ssr_detail_response = respond("/items/1").expect("\"/items/1\" should match");
    assert!(ssr_detail_response.body.contains(&csr_detail_html));
}

/// REQ-1: XSS ペイロード（`demo_items()[1]`）が SSR・SSG・CSR いずれの経路でも
/// 既定エスケープされ、`<script>` として解釈されないこと（三経路回帰）。
#[test]
fn xss_payload_is_escaped_across_ssr_ssg_and_csr_paths() {
    let dir = TempDir::new("xss");
    generate(&dir.0).expect("generate should succeed");

    let payload_item = demo_items()
        .into_iter()
        .find(|it| it.id == "2")
        .expect("demo_items()[1] (id \"2\") carries the XSS payload fixture");

    // SSR 経路。
    let ssr_detail = respond(&format!("/items/{}", payload_item.id))
        .expect("payload item should match")
        .body;
    assert!(!ssr_detail.contains("<script>alert"));
    assert!(ssr_detail.contains("&lt;script&gt;alert"));

    // SSG 経路（SSR と同一ボディのはずだが、独立して固定する）。
    let ssg_detail = fs::read_to_string(
        dir.0
            .join("items")
            .join(&payload_item.id)
            .join("index.html"),
    )
    .expect("payload item file should exist");
    assert!(!ssg_detail.contains("<script>alert"));
    assert!(ssg_detail.contains("&lt;script&gt;alert"));

    // CSR 経路（page_shell を経由しない list_page 単体でも既定エスケープされる）。
    let csr_list_html = render(&list_page(&demo_items()));
    assert!(!csr_list_html.contains("<script>alert"));
    assert!(csr_list_html.contains("&lt;script&gt;alert"));

    // page_shell（SSR/SSG/CSR いずれのモードからも呼ばれる文書骨格）経由でも
    // 同様にエスケープされることを固定する。
    let shell_html = page_shell("記事詳細", detail_page(Some(&payload_item)));
    assert!(!shell_html.contains("<script>alert"));
    assert!(shell_html.contains("&lt;script&gt;alert"));
}

/// 未知 id の 404 経路: 内部パス等の機微情報を含まない固定文言のみ返すこと。
#[test]
fn unknown_item_id_returns_404_without_leaking_internal_details() {
    let response = respond("/items/does-not-exist").expect("pattern should still match");
    assert_eq!(response.status, 404);
    assert!(!response.body.contains("Cargo"));
    assert!(!response.body.contains("/home/"));
    assert!(response.body.contains("見つかりません"));
}

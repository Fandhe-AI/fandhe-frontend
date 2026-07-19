//! 単一バイナリ配布経路での XSS エスケープ維持を検証する統合テスト
//! （TASK-9.4、イシュー #104。REQ-9 受け入れ基準「単一バイナリでの XSS
//! エスケープ（REQ-1）が維持されること」に対応）。
//!
//! # レイヤー責務・既存テストとの違い
//!
//! - `core/tests/xss_escape.rs`: `fandhe_frontend_core::render()` レベルのエスケープ
//!   固定（HTML 文字列を組み立てる最内層）。
//! - `dist-server/tests/routes.rs`: `routes::route_request()`（ハンドラ
//!   レベル）の公開契約固定。プロセス起動・TCP・hyper は経由しない。
//! - `dist-server/tests/boot.rs`（TASK-9.1c）: 実バイナリ起動 + TCP 経由の
//!   起動検証が主目的で、XSS 確認は `/` の部分一致 1 本に留まる。
//! - **本ファイル**: PoC-4（`docs/spec/03-poc/single-binary-distribution/`）の
//!   `curl http://127.0.0.1:3101/items/2` による手動実測を自動化し、実バイナリ・
//!   プロセス境界・hyper トランスポート層を貫通した体系的なエスケープ維持
//!   検証として固定する。一覧・詳細ページ双方、in-process SSR 出力とのバイト
//!   列完全一致、percent-encoded ペイロードの非反射までを扱う。
//!
//! # モードカバレッジ
//!
//! - 既定 `cargo test -p fandhe-frontend-dist-server`: debug ビルド（`assets::AssetMode::
//!   DevFilesystem`）で実行される。
//! - CI の `dist-server-embedded-mode` ジョブ（`cargo test -p fandhe-frontend-dist-server
//!   --features force-embed`）: `force-embed` フィーチャーにより
//!   `assets::AssetMode::Embedded`（release・単一バイナリ配布と同じ配信経路）
//!   に固定した状態で同じテストが実行される。ページ本文の生成
//!   （`fandhe_frontend_server::ssr::respond`）はアセットモードに依存しないため、いずれの
//!   モードでも同一のアサーションが成立する。
//!
//! # 削除・弱体化の禁止
//!
//! `coding-rust.md`「XSS 回帰テスト（SSR / SSG / CSR / WASM の各経路）は
//! 削除・弱体化しない」「テストの `#[ignore]` 追加でごまかさない」に従い、
//! 本ファイルのテストは今後の変更でも維持すること。
//!
//! 依存関係: `fandhe_frontend_server` は `dist-server/Cargo.toml` の通常 `[dependencies]`
//! （`fandhe-frontend-app` 経由の SSR コア）であり、テストから `use` しても新規
//! dev-dependency の追加にはあたらない。`[dev-dependencies]` は本タスクでも
//! 空のまま維持する（REQ-3）。

mod support;

use support::{response_body, send_http_request, spawn_and_wait_for_port, status_code};

/// `demo_items()[1]`（id="2"）の title に埋め込まれた XSS ペイロード
/// （`app/src/lib.rs` 参照）と、`fandhe_frontend_core::escape` の写像
/// （`&`→`&amp;` / `<`→`&lt;` / `>`→`&gt;` / `"`→`&quot;` / `'`→`&#x27;`）
/// に基づく、期待されるエスケープ済み表現。
const ESCAPED_SCRIPT_TAG: &str = "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;";
const ESCAPED_IMG_TAG: &str = "&lt;img src=x onerror=alert(1)&gt;";

/// 上記の生（未エスケープ）表現。応答に含まれてはならない。
const RAW_SCRIPT_TAG: &str = "<script>alert('xss')</script>";
const RAW_IMG_TAG: &str = "<img src=x onerror=alert(1)>";

#[test]
fn detail_page_via_binary_escapes_xss_payload() {
    let (_guard, port) = spawn_and_wait_for_port(
        std::path::Path::new(env!("CARGO_BIN_EXE_dist-server")),
        None,
    );

    let response = send_http_request(port, "GET", "/items/2");

    assert_eq!(status_code(&response), 200);
    let body = response_body(&response);
    assert!(
        body.contains(ESCAPED_SCRIPT_TAG),
        "detail page must contain the escaped <script> representation (REQ-1): {body}"
    );
    assert!(
        body.contains(ESCAPED_IMG_TAG),
        "detail page must contain the escaped <img> representation (REQ-1): {body}"
    );
    assert!(
        !body.contains(RAW_SCRIPT_TAG),
        "detail page must not contain the raw <script> payload (REQ-1 regression guard): {body}"
    );
    assert!(
        !body.contains(RAW_IMG_TAG),
        "detail page must not contain the raw <img> payload (REQ-1 regression guard): {body}"
    );
}

#[test]
fn list_page_via_binary_escapes_xss_payload() {
    let (_guard, port) = spawn_and_wait_for_port(
        std::path::Path::new(env!("CARGO_BIN_EXE_dist-server")),
        None,
    );

    let response = send_http_request(port, "GET", "/");

    assert_eq!(status_code(&response), 200);
    let body = response_body(&response);
    assert!(
        body.contains(ESCAPED_SCRIPT_TAG),
        "list page must contain the escaped <script> representation (REQ-1): {body}"
    );
    assert!(
        body.contains(ESCAPED_IMG_TAG),
        "list page must contain the escaped <img> representation (REQ-1): {body}"
    );
    assert!(
        !body.contains(RAW_SCRIPT_TAG),
        "list page must not contain the raw <script> payload (REQ-1 regression guard): {body}"
    );
    assert!(
        !body.contains(RAW_IMG_TAG),
        "list page must not contain the raw <img> payload (REQ-1 regression guard): {body}"
    );
}

/// トランスポート層（hyper・`routes.rs` の詰め替え）がエスケープ済み HTML を
/// 一切変換しないことの構造的証明。実バイナリ・TCP 越しの応答本文が、
/// 同一プロセス内で `fandhe_frontend_server::ssr::respond` を直接呼んだ場合の出力と
/// バイト列完全一致することを検証する（`route_request`・`main.rs` の詰め
/// 替えが余分な変換を挟まないことの回帰テスト）。
#[test]
fn http_body_matches_in_process_ssr_output_byte_for_byte() {
    let (_guard, port) = spawn_and_wait_for_port(
        std::path::Path::new(env!("CARGO_BIN_EXE_dist-server")),
        None,
    );

    for path in ["/", "/items/2"] {
        let response = send_http_request(port, "GET", path);
        assert_eq!(status_code(&response), 200);
        let http_body = response_body(&response);

        let in_process = fandhe_frontend_server::ssr::respond(path)
            .unwrap_or_else(|| panic!("in-process ssr::respond must resolve path {path}"));

        assert_eq!(
            http_body.as_bytes(),
            in_process.body.as_bytes(),
            "HTTP body for {path} must match in-process SSR output byte-for-byte"
        );
    }
}

/// 反射型 XSS の否定: percent-encode されたペイロードを未知の `id` として
/// 送っても、応答（固定文言の 404 本文）に生ペイロードもエコーバック
/// （percent-encoded のまま）も含まれないことを固定する。
///
/// ルーターは percent-decode を行わない設計（`server/src/router.rs`、
/// out-of-scope-tracking 済み事項）のため、当該 id は「未知の id」として
/// 404 に解決される。生の `<` `>` を含むリクエストラインは hyper 側で
/// 400 になり得るため、本テストは percent-encoded 形のみを送る。
#[test]
fn unknown_item_id_with_payload_is_not_reflected() {
    let (_guard, port) = spawn_and_wait_for_port(
        std::path::Path::new(env!("CARGO_BIN_EXE_dist-server")),
        None,
    );

    let response = send_http_request(port, "GET", "/items/%3Cscript%3Ealert(1)%3C%2Fscript%3E");

    // 未知の id は `fandhe_frontend_server::ssr::respond` が 404 ステータス +
    // `detail_page(None)` の固定文言 HTML（「見つかりません」等）を返す契約
    // （`server/src/ssr.rs` の doc 参照）。`routes::not_found()` の
    // プレーンテキスト固定文言（`/static/` 未一致・完全未一致パス用）とは
    // 別経路のため、ここでは「固定文言 HTML であること」自体は
    // `server/tests`（TASK-6.1c 系）の責務とし、本テストは「入力ペイロードが
    // どちらの形でも反射されないこと」のみを検証する。
    assert_eq!(status_code(&response), 404);
    assert!(
        !response.contains("<script>alert(1)</script>"),
        "response must not reflect the raw payload anywhere: {response}"
    );
    assert!(
        !response.contains("%3Cscript%3E"),
        "response must not echo back the percent-encoded payload anywhere: {response}"
    );
}

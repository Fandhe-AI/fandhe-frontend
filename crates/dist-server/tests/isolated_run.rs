//! REQ-9（単一バイナリ配布）の受け入れ基準「単一バイナリが外部ファイル・
//! Node ランタイム・依存インストールなしで自己完結して動作すること」
//! （`docs/spec/04-requirements.md`）を製品版 `dist-server` に対して固定する
//! 回帰テスト（TASK-9.2a、イシュー #99。親イシュー #98）。
//!
//! PoC-4 で手動実証された「ビルド成果物をソースツリーと無関係なディレクトリ
//! へコピーして起動する」手順（`docs/reports/isolated-run-acceptance-report.md` 第 4
//! 節の検証手順様式）を自動化する。プロセス起動・HTTP 送受信の共通ヘルパは
//! `tests/support/mod.rs`（`tests/boot.rs` と共有）を使う。
//!
//! # なぜファイル全体をコンパイル時ゲートするか（`AssetMode::DevFilesystem` の無効化）
//!
//! `dist-server/src/assets.rs` の `DevFilesystem` モード（debug かつ
//! `force-embed` 無効）は、コンパイル時に埋め込まれた `CARGO_MANIFEST_DIR`
//! の絶対パスから `static/` を読む（`assets.rs` の `dev_fs::static_root`
//! 参照）。そのため debug バイナリを隔離ディレクトリへコピーしても、
//! バイナリはソースツリー上の `static/` を（隔離ディレクトリではなく）
//! 引き続き読めてしまい、「外部ファイル非依存」の検証にならない
//! （偽陽性）。
//!
//! したがって本ファイル全体を
//! `#![cfg(any(not(debug_assertions), feature = "force-embed"))]` で
//! ゲートし、`AssetMode::Embedded`（release、または `force-embed` 有効な
//! debug ビルド）でのみコンパイル・実行される設計とする
//! （`tests/wasm_assets.rs` の `#![cfg(wasm_assets_embedded)]` と同じ、
//! 実行時スキップ（`#[ignore]`）ではなくコンパイル時ゲートを選ぶ方針 —
//! `.claude/rules/coding-rust.md`「テストの `#[ignore]` 追加でごまかさない」
//! に対応）。これにより:
//!
//! - 既存 CI `test` ジョブ（`cargo test --workspace --locked`、debug・
//!   DevFilesystem）では本ファイルはコンパイル対象外（空テストバイナリ）
//!   となり、偽陽性を生まない
//! - `dist-server-embedded-mode` ジョブ（`cargo test -p fandhe-frontend-dist-server
//!   --features force-embed --locked`、`.github/workflows/ci.yml`）では
//!   本ファイルが実行され、Embedded モード = release と同一の配信経路が
//!   実際に外部ファイル非依存であることを固定する
//! - ローカルでは `--release` でも実行可能（PoC-4 手順と等価な真の本番
//!   プロファイル検証）
#![cfg(any(not(debug_assertions), feature = "force-embed"))]

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use support::{send_http_request, spawn_and_wait_for_port, status_code};
// `response_body_bytes` / `send_http_request_bytes` / `status_code_bytes` は
// `isolated_wasm_assets_served`（下記 `#[cfg(wasm_assets_embedded)]`）専用の
// バイト列 API のため、同じ cfg でのみ import する。WASM ビルドステージを
// オプトアウトしたジョブ（`dist-server-embedded-mode`、`FANDHE_FRONTEND_WASM_BUILD=0`）
// では未使用 import になり `-D warnings` に抵触するため。
#[cfg(wasm_assets_embedded)]
use support::{response_body_bytes, send_http_request_bytes, status_code_bytes};

/// 統合テストのスクラッチ基点。`CARGO_TARGET_TMPDIR` は cargo が統合テスト
/// バイナリの**コンパイル時のみ**設定する（Cargo Book）ため `env!` で確定し、
/// 実行時 env による明示上書きのみ許容する。`/tmp` へは一切フォールバック
/// しない（イシュー #637 の事実誤認の再発防止、#658、`cli/tests/support/mod.rs`
/// と同一パターン）。
fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = fs::create_dir_all(&root);
    root
}

/// 隔離ディレクトリ（`<target>/tmp` 配下）を確実に片付ける Drop ガード。
///
/// 検証対象バイナリ以外の何も置かないディレクトリのため、テストの成否に
/// 関わらず終了時に削除してよい。削除失敗（既に削除済み等）はテスト結果に
/// 影響しないため無視する（`ChildGuard` と同じ「Drop での後始末はベスト
/// エフォート」方針、`tests/support/mod.rs` 参照）。
struct IsolatedDirGuard(PathBuf);

impl Drop for IsolatedDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// プロセス内で `create_isolated_binary` が呼ばれるたびに 1 ずつ増える
/// カウンタ。ディレクトリ名の一意性を保証する主要な手段（下記 doc 参照）。
static ISOLATED_DIR_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// `<target>/tmp` 配下に一意な隔離ディレクトリを作成し、
/// `CARGO_BIN_EXE_dist-server` をその中へコピーする。
///
/// ディレクトリ名にプロセス ID・ナノ秒タイムスタンプ・プロセス内カウンタを
/// 含める。`cargo test` は既定でテスト関数をプロセス内の複数スレッドへ
/// 並列実行するため、プロセス ID は全テストで共通であり、ナノ秒タイム
/// スタンプ（+ スレッド ID）のみでは環境によってクロックの実効解像度が粗く
/// 衝突し得ることを実測した（衝突すると、片方のスレッドがコピー中の
/// バイナリをもう片方が実行しようとして `ETXTBSY`（"Text file busy"）で
/// spawn に失敗する）。`AtomicU64` の `fetch_add` は同一プロセス内で
/// 呼び出しごとに厳密に異なる値を返すため、これを主たる一意性の根拠とし、
/// プロセス ID・タイムスタンプは並列 CI ジョブ間・過去の実行残骸との衝突
/// 回避を補強する目的で組み合わせる。
fn create_isolated_binary() -> (IsolatedDirGuard, PathBuf) {
    let sequence = ISOLATED_DIR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let unique = format!(
        "fandhe-frontend-isolated-run-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock must be after UNIX_EPOCH")
            .as_nanos(),
        sequence
    );
    let dir = scratch_root().join(unique);
    fs::create_dir_all(&dir).expect("isolated directory must be creatable under scratch_root");

    let source_binary = Path::new(env!("CARGO_BIN_EXE_dist-server"));
    let binary_name = source_binary
        .file_name()
        .expect("CARGO_BIN_EXE_dist-server must have a file name");
    let copied_binary = dir.join(binary_name);

    // `fs::copy` は Unix ではパーミッションビット（実行権限含む）を保存する
    // ため、コピー後もそのまま `Command::new` で起動できる。
    fs::copy(source_binary, &copied_binary).expect("dist-server binary must be copyable");

    (IsolatedDirGuard(dir), copied_binary)
}

#[test]
fn isolated_binary_boots_without_source_tree() {
    let (_dir_guard, binary) = create_isolated_binary();
    let cwd = binary
        .parent()
        .expect("copied binary must have a parent directory");

    // `spawn_and_wait_for_port` 自体が「listening on 行が出力される
    // （= 正常起動する）」ことを確認する。ここでの成功自体が「外部ファイル
    // 非依存で起動できる」ことの直接的な証拠となる。
    let (_guard, _port) = spawn_and_wait_for_port(&binary, Some(cwd));
}

#[test]
fn isolated_get_root_returns_ssr_html_with_escaped_payload() {
    let (_dir_guard, binary) = create_isolated_binary();
    let cwd = binary
        .parent()
        .expect("copied binary must have a parent directory")
        .to_path_buf();
    let (_guard, port) = spawn_and_wait_for_port(&binary, Some(&cwd));

    let response = send_http_request(port, "GET", "/");

    assert_eq!(status_code(&response), 200);
    assert!(
        response.contains("&lt;script&gt;"),
        "isolated binary must escape the XSS payload by default (REQ-1): {response}"
    );
    assert!(
        !response.contains("<script>"),
        "isolated binary must not serve the raw, unescaped XSS payload (REQ-1 regression guard): {response}"
    );
}

#[test]
fn isolated_get_item_detail_keeps_default_escaping() {
    let (_dir_guard, binary) = create_isolated_binary();
    let cwd = binary
        .parent()
        .expect("copied binary must have a parent directory")
        .to_path_buf();
    let (_guard, port) = spawn_and_wait_for_port(&binary, Some(&cwd));

    // item id "2" は `dist-server/tests/routes.rs`
    // (`known_item_id_returns_200_and_unknown_id_returns_404`) が固定する
    // 既知の item id と同一のものを使う。`demo_items()[1]`（id "2"）の
    // title は意図的な XSS ペイロード（`<script>...`）であり
    // (`dist-server/src/routes.rs` 参照)、詳細ページでも既定エスケープ
    // （REQ-1）が保たれることをここで固定する。ステータスコードのみの
    // 検証では、詳細ページのレンダリング経路でエスケープが漏れる
    // リグレッションを検知できない（Cursor Bugbot 指摘、PR #252）。
    let response = send_http_request(port, "GET", "/items/2");

    assert_eq!(status_code(&response), 200);
    assert!(
        response.contains("&lt;script&gt;"),
        "isolated binary must escape the item detail XSS payload by default (REQ-1): {response}"
    );
    assert!(
        !response.contains("<script>"),
        "isolated binary must not serve the raw, unescaped item detail XSS payload (REQ-1 regression guard): {response}"
    );
}

#[test]
fn isolated_static_assets_served_from_embedded_table() {
    let (_dir_guard, binary) = create_isolated_binary();
    let cwd = binary
        .parent()
        .expect("copied binary must have a parent directory")
        .to_path_buf();
    let (_guard, port) = spawn_and_wait_for_port(&binary, Some(&cwd));

    let response = send_http_request(port, "GET", "/static/view-transitions.js");

    assert_eq!(
        status_code(&response),
        200,
        "static asset must be served from the compile-time embedded table, not the (absent) source tree: {response}"
    );
}

/// WASM ビルドステージ（`dist-server/build.rs`、TASK-10.2b・イシュー #110）
/// が実際に埋め込みテーブルへ合流したときのみコンパイルされる
/// （`tests/wasm_assets.rs` と同じ cfg ゲート）。`dist-server-embedded-mode`
/// ジョブは `FANDHE_FRONTEND_WASM_BUILD: "0"` で WASM ステージをオプトアウトしている
/// ため、本テストは当該ジョブではコンパイル対象外となり偽陽性を出さない
/// （実装計画 §2.3 参照）。
#[cfg(wasm_assets_embedded)]
#[test]
fn isolated_wasm_assets_served() {
    let (_dir_guard, binary) = create_isolated_binary();
    let cwd = binary
        .parent()
        .expect("copied binary must have a parent directory")
        .to_path_buf();
    let (_guard, port) = spawn_and_wait_for_port(&binary, Some(&cwd));

    let js_response = send_http_request(port, "GET", "/static/wasm/fandhe_frontend_wasm_full.js");
    assert_eq!(status_code(&js_response), 200);

    let wasm_response = send_http_request_bytes(
        port,
        "GET",
        "/static/wasm/fandhe_frontend_wasm_full_bg.wasm",
    );
    assert_eq!(status_code_bytes(&wasm_response), 200);
    let body = response_body_bytes(&wasm_response);
    assert!(
        body.len() >= 4 && &body[..4] == b"\0asm",
        "isolated binary must serve the embedded WASM binary intact (magic number check)"
    );
}

#[test]
fn isolated_path_traversal_still_returns_404() {
    let (_dir_guard, binary) = create_isolated_binary();
    let cwd = binary
        .parent()
        .expect("copied binary must have a parent directory")
        .to_path_buf();
    let (_guard, port) = spawn_and_wait_for_port(&binary, Some(&cwd));

    let response = send_http_request(port, "GET", "/static/../Cargo.toml");

    assert_eq!(
        status_code(&response),
        404,
        "path traversal must still be rejected when running in isolation (OWASP A01): {response}"
    );
}

#[test]
fn isolated_dir_stays_clean() {
    let (dir_guard, binary) = create_isolated_binary();
    let cwd = binary
        .parent()
        .expect("copied binary must have a parent directory")
        .to_path_buf();
    let (_guard, port) = spawn_and_wait_for_port(&binary, Some(&cwd));

    // 複数エンドポイントへアクセスした後も、隔離ディレクトリにバイナリ以外の
    // ファイルが生成されていないことを確認する。Embedded モードは実行時に
    // ファイルシステムへ書き込む経路を持たないため（`assets.rs` の
    // モジュール doc 参照）、これは「外部ファイル非依存」の裏付けとなる。
    let _ = send_http_request(port, "GET", "/");
    let _ = send_http_request(port, "GET", "/static/view-transitions.js");

    let entries: Vec<_> = fs::read_dir(&dir_guard.0)
        .expect("isolated directory must still be readable")
        .map(|entry| entry.expect("dir entry must be readable").file_name())
        .collect();

    assert_eq!(
        entries.len(),
        1,
        "isolated directory must contain only the copied binary, found: {entries:?}"
    );
    assert_eq!(
        entries[0],
        binary.file_name().expect("binary must have a file name")
    );
}

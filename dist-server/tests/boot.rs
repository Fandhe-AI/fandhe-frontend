//! `rws-dist-server` バイナリの実プロセス起動検証（TASK-9.1c、イシュー #97）。
//!
//! `tests/routes.rs` はハンドラレベル（`routes::route_request` 呼び出し）に
//! 留め、実際の TCP bind・hyper 接続処理は検証しない設計だった
//! （`tests/routes.rs` 冒頭 doc 参照）。本ファイルはそのギャップを埋め、
//! `env!("CARGO_BIN_EXE_dist-server")` で実バイナリを子プロセスとして起動し、
//! 素の `TcpStream` で HTTP/1.1 リクエストを送ることで、`main.rs`（トランス
//! ポート層）と `routes::route_request`（ルーティングコア）の結合が実際の
//! プロセス境界を越えて機能することを固定する。
//!
//! プロセス起動・HTTP 送受信の共通ヘルパは `tests/support/mod.rs`
//! （TASK-9.2a、イシュー #99 で `tests/isolated_run.rs` と共有するために
//! 抽出）に切り出されている。本ファイルはソースツリー上のバイナリを
//! 通常のカレントディレクトリ（`cwd = None`）で起動する従来どおりの
//! シナリオのみを担う（隔離ディレクトリでの起動検証は `isolated_run.rs`）。
//!
//! 外部 dev-dependency（reqwest 等）は追加しない（`Cargo.toml` の
//! `[dev-dependencies]` は空のまま — REQ-3 の趣旨、`dist-server/Cargo.toml`
//! 冒頭コメント参照）。プロセス起動・HTTP 通信はすべて `std` のみで行う。
//!
//! # #97 完了時点でのカバレッジ（`docs/dist-server-design.md` 8 節との対応）
//!
//! 親イシュー #94（PR #244）の前倒し実装により、本ファイルは元々 4 観点
//! （`/` 200・エスケープ済み HTML／静的アセット 200／パストラバーサル 404／
//! bind 失敗時の非 0 終了・固定 stderr）のみを実プロセスレベルで固定して
//! いた。#97 では設計書 8 節に列挙されながら未実装だった残り 3 観点
//! （`/items/:id` の既知 ID 200・未知 ID 404、未知パスの 404、静的アセットの
//! Content-Type）を本ファイルへ追補し、設計書 8 節の観点をすべて実プロセス
//! 境界（子プロセス起動 + 素の TCP）で固定した。
//!
//! スコープ境界（重複実装を避けるための担当分け）:
//! - `RWS_BIND_ADDR` によるバインドアドレス自体の切り替え検証 → `tests/bind_addr.rs`（#162）
//! - XSS エスケープの単一バイナリ経路での本格的な統合検証（複数ペイロード・
//!   属性コンテキスト等の網羅）→ TASK-9.4（#104、本ファイルは疎通レベルの
//!   軽い回帰のみを担う）
//! - 隔離ディレクトリへコピーして起動する変種 → `tests/isolated_run.rs`（TASK-9.2、#98）

mod support;

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::Duration;
use support::{
    send_http_request, spawn_and_wait_for_port, status_code, wait_with_timeout, ChildGuard,
};

#[test]
fn get_root_returns_200_with_escaped_xss_payload() {
    let (_guard, port) = spawn_and_wait_for_port(
        std::path::Path::new(env!("CARGO_BIN_EXE_dist-server")),
        None,
    );

    let response = send_http_request(port, "GET", "/");

    assert_eq!(status_code(&response), 200);
    assert!(
        response.contains("&lt;script&gt;"),
        "list page must escape the XSS payload by default (REQ-1): {response}"
    );
    assert!(
        !response.contains("<script>"),
        "list page must not contain the raw, unescaped XSS payload (REQ-1 regression guard): {response}"
    );
}

#[test]
fn get_static_asset_returns_200_with_expected_content_type() {
    let (_guard, port) = spawn_and_wait_for_port(
        std::path::Path::new(env!("CARGO_BIN_EXE_dist-server")),
        None,
    );

    let response = send_http_request(port, "GET", "/static/view-transitions.js");

    assert_eq!(status_code(&response), 200);
    // hyper 1.x は応答ヘッダ名を小文字で書き出すため、大文字小文字非依存で
    // 照合する（`dist-server/src/mime.rs` が返す固定 MIME、
    // `tests/routes.rs` の `static_asset_is_served_with_expected_content_type`
    // と同じ期待値をプロセス境界越しに固定する）。
    assert!(
        response
            .to_lowercase()
            .contains("content-type: text/javascript; charset=utf-8"),
        "static asset response must set the expected Content-Type header: {response}"
    );
}

#[test]
fn get_known_item_detail_returns_200_with_escaped_xss_payload() {
    let (_guard, port) = spawn_and_wait_for_port(
        std::path::Path::new(env!("CARGO_BIN_EXE_dist-server")),
        None,
    );

    // id "2" は `rws_app::demo_items()` 内の XSS ペイロード付き既知アイテム
    // （`server/src/ssr.rs` 等、他の統合テストと共通の固定 ID）。
    let response = send_http_request(port, "GET", "/items/2");

    assert_eq!(status_code(&response), 200);
    assert!(
        response.contains("&lt;script&gt;"),
        "detail page must escape the XSS payload by default (REQ-1): {response}"
    );
    assert!(
        !response.contains("<script>"),
        "detail page must not contain the raw, unescaped XSS payload (REQ-1 regression guard): {response}"
    );
}

#[test]
fn get_unknown_item_id_returns_404_with_fixed_body() {
    let (_guard, port) = spawn_and_wait_for_port(
        std::path::Path::new(env!("CARGO_BIN_EXE_dist-server")),
        None,
    );

    let response = send_http_request(port, "GET", "/items/does-not-exist");

    assert_eq!(status_code(&response), 404);
    assert!(
        response.contains("404 Not Found"),
        "unknown item id must return the fixed 404 body: {response}"
    );
    // 機微情報（内部パス・ソースツリー由来の識別子・panic 文言等）が
    // 応答本文へ漏れないことを固定する（`security.md` A09）。
    assert!(
        !response.to_lowercase().contains("panic") && !response.contains("src/"),
        "404 response must not leak internal details: {response}"
    );
}

#[test]
fn unknown_path_returns_404_with_fixed_body() {
    let (_guard, port) = spawn_and_wait_for_port(
        std::path::Path::new(env!("CARGO_BIN_EXE_dist-server")),
        None,
    );

    let response = send_http_request(port, "GET", "/no-such-page");

    assert_eq!(status_code(&response), 404);
    assert!(
        response.contains("404 Not Found"),
        "unknown path must return the fixed 404 body: {response}"
    );
    assert!(
        !response.to_lowercase().contains("panic") && !response.contains("src/"),
        "404 response must not leak internal details: {response}"
    );
}

#[test]
fn path_traversal_against_static_assets_returns_404() {
    let (_guard, port) = spawn_and_wait_for_port(
        std::path::Path::new(env!("CARGO_BIN_EXE_dist-server")),
        None,
    );

    let response = send_http_request(port, "GET", "/static/../Cargo.toml");

    assert_eq!(status_code(&response), 404);
}

#[test]
fn bind_conflict_exits_non_zero_with_fixed_stderr_message() {
    // 先に素の `TcpListener` でポートを 1 つ占有し、`dist-server` に同じ
    // アドレスを明示指定させて bind 失敗を再現する。
    let occupied = std::net::TcpListener::bind("127.0.0.1:0").expect("must bind a probe port");
    let occupied_addr = occupied.local_addr().expect("must read local_addr");

    let mut child = Command::new(env!("CARGO_BIN_EXE_dist-server"))
        .env("RWS_BIND_ADDR", occupied_addr.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("dist-server binary must spawn");

    // stderr は `wait()` より先に（別スレッドで）ドレインする。子プロセスの
    // 出力量がパイプバッファ（通常 64KB 程度）を超えた場合、`wait()` を先に
    // 呼ぶと「子は書き込みブロック・親は wait 待ち」の典型的なデッドロックに
    // 陥り得る（read_listening_port と同様の対策、レビュー指摘対応）。
    let stderr_pipe = child
        .stderr
        .take()
        .expect("stderr must be piped for spawned child");
    let stderr_reader = std::thread::spawn(move || {
        let mut stderr = String::new();
        let mut stderr_pipe = stderr_pipe;
        stderr_pipe
            .read_to_string(&mut stderr)
            .expect("stderr must be readable as UTF-8");
        stderr
    });

    // `wait_with_timeout` を呼ぶ前に `ChildGuard` でラップする。bind 失敗が
    // 起きない、または起動が停滞して `wait_with_timeout` が panic した場合
    // でも、guard の `Drop` が子プロセスの kill/wait を保証する（レビュー
    // 指摘対応、`spawn_and_wait_for_port` と同様の方針）。
    let mut guard = ChildGuard(child);

    let status = wait_with_timeout(&mut guard.0, Duration::from_secs(5));
    assert!(
        !status.success(),
        "dist-server must exit non-zero when the bind address is already in use"
    );

    let stderr = stderr_reader
        .join()
        .expect("stderr reader thread must not panic");

    // 機微情報（内部パス・スタックトレース等）を含まない固定文言のみで
    // あることを確認する（`security.md` A09、`main.rs` の doc 参照）。
    assert!(
        stderr.contains("rws-dist-server: failed to bind"),
        "stderr must contain the fixed bind-failure message: {stderr}"
    );
    assert!(
        !stderr.to_lowercase().contains("panic"),
        "bind failure must not panic: {stderr}"
    );

    drop(occupied);
}

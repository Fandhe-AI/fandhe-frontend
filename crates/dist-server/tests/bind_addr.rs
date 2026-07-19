//! `FANDHE_FRONTEND_BIND_ADDR` によるバインドアドレス・ポート切り替えの受け入れ基準検証
//! （イシュー #162）。
//!
//! `docs/spec/04-requirements.md`（REQ-9、画面・インターフェース要件、
//! PoC-4 踏襲）は「`FANDHE_FRONTEND_BIND_ADDR`（またはフレームワーク標準の環境変数）で
//! バインドアドレスを切り替え可能」であることを要求している。実装自体は
//! TASK-9.1（イシュー #94）で完了済みで、`tests/boot.rs` も
//! `FANDHE_FRONTEND_BIND_ADDR=127.0.0.1:0`（OS 割当ポート）での起動・bind 競合時の非 0
//! 終了は検証している。しかし「環境変数に指定した値どおりにバインド
//! **アドレス**・**ポート**が切り替わる」ことを直接検証するテストは
//! 存在しなかった。本ファイルはそのギャップを埋め、`main.rs` が stderr へ
//! 出力する `listening on <addr>` 行の `<addr>` が `FANDHE_FRONTEND_BIND_ADDR` の指定値と
//! 一致すること、かつ実際にそのアドレス・ポートで接続を受け付けることを
//! 検証する。
//!
//! `boot.rs` との棲み分け: `boot.rs` は起動・ルーティングの結合（固定の
//! `127.0.0.1:0` で起動し、ハンドラ挙動を検証）を担う。本ファイルは
//! bind 設定そのものの切り替えを検証対象とする。
//!
//! 外部 dev-dependency（reqwest 等）は追加しない（`Cargo.toml` の
//! `[dev-dependencies]` は空のまま — REQ-3 の趣旨、`dist-server/Cargo.toml`
//! 冒頭コメント参照）。プロセス起動・HTTP 通信はすべて `std` のみで行う。
//!
//! セキュリティ上の注意（`security.md` A05）: 本テストが bind するのは
//! ループバック域（`127.0.0.1` / `127.0.0.2`）のみであり、`0.0.0.0` へは
//! 一切 bind しない（CI・開発機のポートを外部へ露出させないため）。

mod support;

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use support::{parse_listening_addr, send_http_request, status_code, ChildGuard};
// `spawn_with_bind_addr` と `TcpStream` は `bind_addr_env_switches_address`
// （Linux 限定、下記 `#[cfg(target_os = "linux")]` 参照）専用。非 Linux
// ビルドでは未使用 import になるため cfg で揃える。
#[cfg(target_os = "linux")]
use std::net::TcpStream;
#[cfg(target_os = "linux")]
use support::spawn_with_bind_addr;

/// bind 対象ポートの探索・起動リトライの上限回数。
///
/// 空きポートを見つけてからリスナーを drop し、その直後に子プロセスへ同じ
/// ポートを指定して bind させるまでの間（TOCTOU）に、同一マシン上の別プロセス
/// （並列実行される他のテストを含む）がそのポートを奪う可能性がゼロではない。
/// 稀な競合に備え、bind に失敗した場合は新しい空きポートで数回だけ再試行する。
const MAX_PORT_RETRIES: u32 = 3;

/// [`spawn_with_bind_addr`] を直接使わず本ファイル専用に実装した起動ヘルパ。
///
/// `support::spawn_with_bind_addr`（延いては内部の `read_listening_addr`）は
/// 「`listening on` 行が出るまで待ち、出なければ 5 秒後に panic する」契約
/// になっている（bind 成功を前提とする `boot.rs` 等の既存呼び出し側に合わせた
/// 設計）。本関数は bind **失敗**（`main.rs` が出す
/// `"fandhe-frontend-dist-server: failed to bind"` 行）を正常系の一つとして区別し、
/// 呼び出し側（TOCTOU リトライループ）へ `Err` として返す必要があるため、
/// 共有ヘルパには委譲せず個別に実装する。
fn try_spawn_with_bind_addr(binary: &Path, bind_addr: &str) -> Result<(ChildGuard, String), ()> {
    let mut child = Command::new(binary)
        .env("FANDHE_FRONTEND_BIND_ADDR", bind_addr)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("dist-server binary must spawn");

    let stderr = child
        .stderr
        .take()
        .expect("stderr must be piped for spawned child");
    // `spawn_and_wait_for_port` と同じ理由で、panic し得る読み取りより前に
    // `ChildGuard` でラップする（panic 時も子プロセスの kill/wait を保証）。
    let guard = ChildGuard(child);

    let (tx, rx) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if tx.send(line.clone()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            panic!("dist-server did not print a listening/failure line within timeout");
        }
        match rx.recv_timeout(remaining) {
            Ok(line) => {
                let trimmed = line.trim_end();
                if let Some(addr) = parse_listening_addr(trimmed) {
                    return Ok((guard, addr.to_string()));
                }
                if trimmed.starts_with("fandhe-frontend-dist-server: failed to bind") {
                    return Err(());
                }
                // それ以外の行（`assets=` 等）は無視して読み取りを続ける。
            }
            Err(mpsc::RecvTimeoutError::Timeout | mpsc::RecvTimeoutError::Disconnected) => {
                panic!("dist-server did not print a listening/failure line within timeout");
            }
        }
    }
}

/// `FANDHE_FRONTEND_BIND_ADDR` に環境変数経由で明示指定したポート番号が、実際の
/// バインドポートとして反映されることを検証する（既定値 3100 でも OS 任意
/// 割当でもなく、環境変数の値そのものが使われたことの直接証明）。
#[test]
fn bind_addr_env_switches_port() {
    let binary = Path::new(env!("CARGO_BIN_EXE_dist-server"));

    for attempt in 1..=MAX_PORT_RETRIES {
        // ポート 0 で bind して OS に空きポートを割り当てさせ、`local_addr()`
        // で番号を読み取ったうえでリスナーを即座に drop する。drop してから
        // 子プロセスを起動するまでの短い窓の間だけ、他プロセスにポートを
        // 奪われる可能性がある（TOCTOU、上記 `MAX_PORT_RETRIES` 参照）。
        let probe = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("must bind a probe port to find a free one");
        let free_port = probe.local_addr().expect("must read local_addr").port();
        drop(probe);

        let requested_addr = format!("127.0.0.1:{free_port}");
        let (guard, reported_addr) = match try_spawn_with_bind_addr(binary, &requested_addr) {
            Ok(result) => result,
            Err(()) => {
                // TOCTOU でポートを奪われた場合のみここに来る。次のポートで
                // 再試行する（最終試行での失敗は下の assert で検出させる）。
                continue;
            }
        };

        assert_eq!(
            reported_addr, requested_addr,
            "attempt {attempt}: reported bind address must equal the requested one"
        );

        // 環境変数で指定したとおりのポートで実際に待ち受けていることを、
        // ログの見かけだけでなく実際の HTTP 接続で確認する。
        let response = send_http_request(free_port, "GET", "/");
        assert_eq!(
            status_code(&response),
            200,
            "dist-server must accept connections on the FANDHE_FRONTEND_BIND_ADDR-specified port"
        );

        drop(guard);
        return;
    }

    panic!(
        "dist-server failed to bind a freshly-probed port after {MAX_PORT_RETRIES} attempts \
         (persistent TOCTOU collision is unexpected)"
    );
}

/// #437 の fail-open 回帰テスト: 改名前の環境変数名 `RWS_BIND_ADDR` のみを
/// 設定しても新コードからは一切読み取られず、既定のループバックアドレス
/// （`DEFAULT_BIND_ADDR` = `127.0.0.1:3100`）へフォールバックすることを
/// 検証する（`security.md` A05「意図しない広いバインドが発生しないこと」の
/// 直接証明）。
///
/// `127.0.0.2:0`（非既定ホスト・OS 割当ポート）を旧名で指定しても
/// 反映されない ―― すなわち stderr の最初の意味のある行が
/// 「`listening on 127.0.0.1:3100`」（既定アドレスへの成功）または
/// 「`failed to bind 127.0.0.1:3100`」（共有 runner で 3100 番ポートが
/// 既に使用中の場合の失敗）のいずれかであることを assert する。両者の
/// いずれであっても「旧名が無視され既定アドレス 127.0.0.1:3100 が使われた」
/// ことの証明になり、固定ポート 3100 の TOCTOU 競合による flaky 化を避けつつ
/// 決定的に検証できる。
#[test]
fn legacy_rws_bind_addr_name_is_ignored_and_falls_back_to_default() {
    let binary = Path::new(env!("CARGO_BIN_EXE_dist-server"));

    let mut command = Command::new(binary);
    command
        .env_remove("FANDHE_FRONTEND_BIND_ADDR")
        .env("RWS_BIND_ADDR", "127.0.0.2:0")
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("dist-server binary must spawn");

    let stderr = child
        .stderr
        .take()
        .expect("stderr must be piped for spawned child");
    let guard = ChildGuard(child);

    let (tx, rx) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if tx.send(line.clone()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let deadline = Instant::now() + Duration::from_secs(5);
    let observed = loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            panic!("dist-server did not print a listening/failure line within timeout");
        }
        match rx.recv_timeout(remaining) {
            Ok(line) => {
                let trimmed = line.trim_end().to_string();
                if trimmed.starts_with("fandhe-frontend-dist-server: listening on")
                    || trimmed.starts_with("fandhe-frontend-dist-server: failed to bind")
                {
                    break trimmed;
                }
                // それ以外の行（`assets=` 等）は無視して読み取りを続ける。
            }
            Err(mpsc::RecvTimeoutError::Timeout | mpsc::RecvTimeoutError::Disconnected) => {
                panic!("dist-server did not print a listening/failure line within timeout");
            }
        }
    };

    drop(guard);
    let listened_on_default =
        observed == "fandhe-frontend-dist-server: listening on 127.0.0.1:3100";
    let failed_to_bind_default =
        observed.starts_with("fandhe-frontend-dist-server: failed to bind 127.0.0.1:3100");
    assert!(
        listened_on_default || failed_to_bind_default,
        "legacy RWS_BIND_ADDR must be ignored and dist-server must attempt only the default \
         127.0.0.1:3100 address (never 127.0.0.2, the legacy-name value): observed {observed:?}"
    );
}

/// `FANDHE_FRONTEND_BIND_ADDR` に指定したホスト部（`127.0.0.1` 以外のループバック
/// アドレス）が実際のバインドアドレスとして反映されることを検証する。
///
/// Linux はループバック `/8`（`127.0.0.0/8`）全域が追加設定なしで bind 可能
/// なため `127.0.0.2` を使う。macOS 等ではエイリアス設定が必要になり得る
/// ため、CI（ubuntu-latest）が常時実行する Linux 限定で検証する。
#[cfg(target_os = "linux")]
#[test]
fn bind_addr_env_switches_address() {
    let binary = Path::new(env!("CARGO_BIN_EXE_dist-server"));

    let (_guard, reported_addr) = spawn_with_bind_addr(binary, None, "127.0.0.2:0");

    // ホスト部が明示指定どおり `127.0.0.2` であることを assert する
    // （ログの見かけ側の証明）。
    let host = reported_addr
        .rsplit_once(':')
        .map(|(host, _port)| host)
        .expect("listening address must contain a host:port separator");
    assert_eq!(
        host, "127.0.0.2",
        "listening address host must match the FANDHE_FRONTEND_BIND_ADDR-specified host: {reported_addr}"
    );

    let port: u16 = reported_addr
        .rsplit(':')
        .next()
        .and_then(|port_str| port_str.parse().ok())
        .expect("listening address must end with a valid port");

    // 実際に `127.0.0.2` へ到達できることを assert する（否定側証明の前提と
    // なる、bind アドレスが確かに切り替わったことの肯定側証明）。
    let response = support::send_http_request_to("127.0.0.2", port, "GET", "/");
    assert_eq!(
        status_code(&response),
        200,
        "dist-server must accept connections on the FANDHE_FRONTEND_BIND_ADDR-specified host"
    );

    // 否定側証明: 同じポートの `127.0.0.1` には bind していない（ログ上の
    // 見かけではなく、実際に bind アドレスが切り替わったことの証明）。
    // 万一無関係な別プロセスが同じポートで `127.0.0.1` 側に待ち受けている
    // 稀な衝突があり得るが、テスト環境（CI ジョブ・ローカル開発機）が生成
    // した空きポートに対してその衝突が起きる可能性は無視できるほど低い。
    let connect_timeout = Duration::from_millis(500);
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}")
        .parse()
        .expect("127.0.0.1:<port> must parse as a socket address");
    let connect_result = TcpStream::connect_timeout(&addr, connect_timeout);
    assert!(
        connect_result.is_err(),
        "dist-server bound to 127.0.0.2 must not also accept connections on 127.0.0.1: {reported_addr}"
    );

    assert!(
        parse_listening_addr(&format!(
            "fandhe-frontend-dist-server: listening on {reported_addr}"
        ))
        .is_some(),
        "listening line format contract must still hold for the reported address"
    );
}

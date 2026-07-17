//! `dist-server` の実プロセス起動検証（`tests/boot.rs`・`tests/isolated_run.rs`）
//! が共有するヘルパ群（TASK-9.2a、イシュー #99）。
//!
//! 元々は `tests/boot.rs`（TASK-9.1c、イシュー #97）に単体で実装されていたが、
//! `isolated_run.rs`（隔離ディレクトリへコピーしたバイナリを起動する変種）が
//! 同じプロセス管理・HTTP 送受信の仕組みを必要としたため、共通部分をこの
//! `tests/support/mod.rs` へ抽出した。両テストファイルからは `mod support;`
//! （`#[path = "support/mod.rs"] mod support;` 不要 — ディレクトリ名がモジュール
//! 名と一致するため通常の `mod` 宣言で解決される）で利用する。
//!
//! 外部 dev-dependency（reqwest 等）は追加しない（`dist-server/Cargo.toml` の
//! `[dev-dependencies]` は空のまま — REQ-3 の趣旨）。プロセス起動・HTTP 通信は
//! すべて `std` のみで行う。
//!
//! # integration test ハーネスの制約について
//!
//! `tests/` 配下の各 `.rs` ファイルは cargo によって独立したテストバイナリへ
//! コンパイルされる。共通モジュールを `mod support;` で複数のテストバイナリ
//! （`boot.rs`・`isolated_run.rs`）から読み込むと、本ファイル中の未使用関数が
//! テストバイナリごとに `dead_code` 警告の対象になり得る（各バイナリが使う
//! 関数の組み合わせが異なるため）。呼び出し側で全関数を使い切らない場合に
//! 備え、個々の関数へ `#[allow(dead_code)]` は付けず、利用側の `mod support;`
//! 宣言に本モジュール全体の未使用を許容する属性は付与しない方針とする
//! （現状すべての公開関数が boot.rs・isolated_run.rs のいずれかから使われて
//! おり、警告は発生しない）。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// 子プロセスを確実に終了させる Drop ガード。
///
/// テストがアサート失敗・panic で早期リターンしても、`dist-server` の
/// 子プロセスがゾンビ化・ポート占有したまま CI 上に残らないようにする
/// （実装計画のセキュリティ考慮「プロセス管理」参照）。
pub struct ChildGuard(pub Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        // 終了操作の失敗（既に終了済み等）はテスト結果に影響しないため無視する。
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// `binary` を起動し、`RWS_BIND_ADDR=127.0.0.1:0`（OS 割当ポート）で bind
/// させたうえで stderr の `listening on` 行から実際のポートを読み取る。
///
/// `cwd` を指定すると子プロセスの作業ディレクトリを固定する
/// （`isolated_run.rs` がソースツリーと無関係な隔離ディレクトリを
/// カレントディレクトリとして起動する用途）。`None` の場合は親プロセスの
/// カレントディレクトリを継承する（`boot.rs` の従来動作と同一）。
///
/// タイムアウト（5 秒）以内に当該行が出力されない場合は panic する
/// （テストコードでの panic は `coding-rust.md` の対象外 — ライブラリコード
/// のみ禁止）。
pub fn spawn_and_wait_for_port(binary: &Path, cwd: Option<&Path>) -> (ChildGuard, u16) {
    let mut command = Command::new(binary);
    command
        .env("RWS_BIND_ADDR", "127.0.0.1:0")
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }

    let mut child = spawn_with_etxtbsy_retry(&mut command);

    let stderr = child
        .stderr
        .take()
        .expect("stderr must be piped for spawned child");
    let reader = BufReader::new(stderr);

    // `read_listening_port` の呼び出し（タイムアウト・panic し得る）より前に
    // `ChildGuard` でラップする。ラップを後回しにすると、その間に panic や
    // 早期リターンが起きた場合、生の `Child` が guard で保護されないまま
    // drop されてしまう（`std::process::Child` は drop 時に自動終了しない
    // ため、子プロセスが起動したままゾンビ化・ポート占有し得る）。
    let guard = ChildGuard(child);

    let port = read_listening_port(reader);
    (guard, port)
}

/// `Command::spawn` を、`ETXTBSY`（"Text file busy"）に限り短い待機を挟んで
/// リトライする。
///
/// `isolated_run.rs` は「`fs::copy` でバイナリをコピー → 直後に spawn」を
/// 各テストが並列（`cargo test` 既定のマルチスレッド実行）に行う。実測で、
/// コピー先パスは呼び出しごとに一意（衝突なし）であるにもかかわらず、
/// 別スレッドが同時に別のバイナリを fork+exec している最中だと、Linux の
/// カーネルが無関係のパスに対して一時的に `ETXTBSY` を返すことがある
/// （fork 直後・exec 前の書き込み可能な fd 継承に起因する既知の過渡的事象。
/// Go の `exec_test.go` 等でも同種のリトライ対策が取られている）。
/// 対象ファイル自体の破損やロックではなく一過性のカーネル挙動のため、
/// 短い待機を挟んだ再試行で解消する。`ETXTBSY` 以外のエラーは即座に
/// panic する（真の起動失敗を握り潰さないため）。
fn spawn_with_etxtbsy_retry(command: &mut Command) -> Child {
    const MAX_ATTEMPTS: u32 = 20;
    const RETRY_DELAY: Duration = Duration::from_millis(25);

    for attempt in 1..=MAX_ATTEMPTS {
        match command.spawn() {
            Ok(child) => return child,
            Err(err) if err.kind() == std::io::ErrorKind::ExecutableFileBusy => {
                if attempt == MAX_ATTEMPTS {
                    panic!(
                        "dist-server binary must spawn (still ETXTBSY after {MAX_ATTEMPTS} attempts): {err}"
                    );
                }
                std::thread::sleep(RETRY_DELAY);
            }
            Err(err) => panic!("dist-server binary must spawn: {err}"),
        }
    }

    unreachable!("loop above always returns or panics");
}

/// stderr から `"rws-dist-server: listening on 127.0.0.1:<port>"` 行を探し、
/// `<port>` を返す。5 秒待っても見つからなければ panic する。
///
/// `BufReader::read_line` は子プロセスの `ChildStderr` パイプに対する
/// ブロッキング呼び出しで、読み取り自体にタイムアウトを設定する手段が
/// 標準ライブラリにはない（`TcpStream::set_read_timeout` 相当が存在しない）。
/// そのため実際の読み取りは別スレッドへ切り出し、本スレッドは
/// `mpsc::Receiver::recv_timeout` でデッドラインまで待つことでタイムアウトを
/// 実効化する。
fn read_listening_port(reader: BufReader<std::process::ChildStderr>) -> u16 {
    let (tx, rx) = mpsc::channel::<String>();

    // 読み取りスレッドは検出後も本体側から join しない（detach する）。
    // 本関数はポートが見つかり次第 return するため、join すると子プロセスの
    // 後続出力を待ち続けてしまい、タイムアウト対策そのものが無意味になる。
    // 受信側が既に return してチャネルが閉じている場合は素直に終了する。
    std::thread::spawn(move || {
        let mut reader = reader;
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF: プロセスが起動前に終了した
                Ok(_) => {
                    if tx.send(line.clone()).is_err() {
                        break; // 受信側は既に return 済み
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
            break;
        }
        match rx.recv_timeout(remaining) {
            Ok(line) => {
                if let Some(port) = parse_listening_port(line.trim_end()) {
                    return port;
                }
            }
            // タイムアウト・読み取りスレッド終了（EOF/エラー）はいずれも
            // 「該当行が見つからなかった」扱いとして panic へフォールスルーする。
            Err(mpsc::RecvTimeoutError::Timeout) => break,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    panic!("dist-server did not print a \"listening on\" line with a port within timeout");
}

/// `child.wait()` を無期限にブロックさせず、タイムアウト（5 秒）付きで
/// 子プロセスの終了を待つ。
///
/// bind 失敗を検証するテスト用。bind 失敗のパスでは子プロセスは即座に
/// 終了するはずだが、想定外に起動が停滞した場合でも `wait()` を無期限に
/// 呼ぶとテストごと CI をハングさせてしまう。`try_wait()` によるポーリング
/// でデッドラインを実効化し、タイムアウト時は panic する（呼び出し元は
/// `ChildGuard` でラップ済みであることが前提 — panic 後も `Drop` で子
/// プロセスの kill/wait が保証される）。
///
/// `isolated_run.rs` のテストバイナリでは未使用（bind 失敗検証は
/// `boot.rs` の `bind_conflict_exits_non_zero_with_fixed_stderr_message`
/// 専用のシナリオ）のため `#[allow(dead_code)]` で抑止する。
#[allow(dead_code)]
pub fn wait_with_timeout(child: &mut Child, timeout: Duration) -> std::process::ExitStatus {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child
            .try_wait()
            .expect("try_wait on dist-server child must not error")
        {
            return status;
        }
        if Instant::now() >= deadline {
            panic!("dist-server did not exit within timeout after a bind conflict");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// `"rws-dist-server: listening on 127.0.0.1:PORT"` 形式の 1 行から
/// ポート番号を抽出する（`main.rs` の起動ログ契約に対応）。
pub fn parse_listening_port(line: &str) -> Option<u16> {
    let addr = line.strip_prefix("rws-dist-server: listening on ")?;
    let port_str = addr.rsplit(':').next()?;
    port_str.parse::<u16>().ok()
}

/// `127.0.0.1:port` へ TCP 接続し、素の HTTP/1.1 リクエストを送って
/// レスポンス全体（ヘッダ + ボディ）を文字列で返す。
pub fn send_http_request(port: u16, method: &str, path: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("must connect to dist-server");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set_read_timeout must succeed");

    let request =
        format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .expect("request must be written");

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("response must be readable as UTF-8");
    response
}

/// レスポンス文字列の先頭ステータス行（`HTTP/1.1 200 OK` 等）からステータス
/// コードを取り出す。
pub fn status_code(response: &str) -> u16 {
    response
        .lines()
        .next()
        .and_then(|status_line| status_line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .expect("response must start with a valid HTTP status line")
}

/// [`send_http_request`] のバイト列版。
///
/// WASM バイナリ（`.wasm`）の応答本文は有効な UTF-8 とは限らない
/// （`\0asm` マジックナンバーを含む）ため、`send_http_request` の
/// `read_to_string`（UTF-8 前提）では読み取りに失敗する。
/// `isolated_run.rs` の WASM アセット検証はこちらを使う。
///
/// `tests/` 配下は各ファイルが独立したテストバイナリへコンパイルされる
/// （`mod support;` を宣言するファイルごとに本モジュールが再コンパイル
/// される）。`boot.rs` はこの関数を呼ばないため、`boot.rs` のテスト
/// バイナリでは常に未使用になり `dead_code` 警告の対象となる。実際には
/// `isolated_run.rs`（`wasm_assets_embedded` cfg 有効時）から使われる
/// ため `#[allow(dead_code)]` で抑止する。
#[allow(dead_code)]
pub fn send_http_request_bytes(port: u16, method: &str, path: &str) -> Vec<u8> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("must connect to dist-server");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set_read_timeout must succeed");

    let request =
        format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .expect("request must be written");

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .expect("response must be readable");
    response
}

/// バイト列レスポンスの先頭ステータス行からステータスコードを取り出す
/// （[`status_code`] のバイト列版）。ステータス行自体は常に ASCII のため
/// `str::from_utf8` で先頭行だけを取り出せば十分。
///
/// `boot.rs` のテストバイナリでは未使用（`send_http_request_bytes` と同じ
/// 理由、上記 doc 参照）のため `#[allow(dead_code)]` で抑止する。
#[allow(dead_code)]
pub fn status_code_bytes(response: &[u8]) -> u16 {
    let header_end = find_header_end(response).unwrap_or(response.len());
    let header_bytes = &response[..header_end];
    let header_str =
        std::str::from_utf8(header_bytes).expect("HTTP header section must be ASCII/UTF-8");
    header_str
        .lines()
        .next()
        .and_then(|status_line| status_line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .expect("response must start with a valid HTTP status line")
}

/// バイト列レスポンスからヘッダ部分を除いた本文（ボディ）を取り出す。
/// `Content-Length` は解釈せず、`Connection: close` 前提（本テスト群の
/// リクエストは常に付与している）で接続断までを読み切った
/// [`send_http_request_bytes`] の出力をヘッダ区切り（`\r\n\r\n`）で
/// 単純分割するのみで十分。
///
/// `boot.rs` のテストバイナリでは未使用（`send_http_request_bytes` と同じ
/// 理由、上記 doc 参照）のため `#[allow(dead_code)]` で抑止する。
#[allow(dead_code)]
pub fn response_body_bytes(response: &[u8]) -> &[u8] {
    match find_header_end(response) {
        Some(header_end) => &response[header_end + 4..],
        None => &[],
    }
}

/// `\r\n\r\n`（ヘッダ / ボディ区切り）の開始インデックスを探す。
#[allow(dead_code)]
fn find_header_end(response: &[u8]) -> Option<usize> {
    response.windows(4).position(|window| window == b"\r\n\r\n")
}

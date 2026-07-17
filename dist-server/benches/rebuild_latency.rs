//! TASK-10.4a（イシュー #119、REQ-10）: 本番ビルドのアセット変更反映
//! （差分ビルド）が 5 秒以内であることを継続計測する `[[bench]]`。
//!
//! # 背景・契約
//!
//! REQ-10 の受け入れ基準（`docs/spec/04-requirements.md`）「本番ビルドの
//! アセット変更反映（差分ビルド）が 5 秒以内」を、PoC-4 実測方法
//! （`docs/spec/03-poc/single-binary-distribution/README.md`: `static/` の
//! アセットを変更 → `cargo build --release` の壁時計時間を計測）に沿って
//! 自動化する。判定・サマリ整形は [`rws_dist_server::bench_support`]
//! （`dist-server/src/bench_support.rs`）へ切り出し、そちらの
//! `#[cfg(test)]` で書式契約を固定する（本ファイルは `test = false` の
//! ため `cargo test` からは実行されない）。
//!
//! # 実行方法
//!
//! ```text
//! cargo bench -p rws-dist-server --bench rebuild_latency
//! ```
//!
//! wasm ツールチェーン未整備環境では `RWS_WASM_BUILD=0` を付与すると
//! `dist-server/build.rs` の WASM ビルドステージをスキップできる
//! （`build.rs` の `wasm_build_enabled` 参照。本ベンチは環境変数を
//! 読み替えず、子プロセスへそのまま継承させるだけ）。
//!
//! # 計測プロトコル
//!
//! 1. ウォームビルド（計測対象外）で依存クレートのビルドキャッシュを温める。
//! 2. `static/` 配下の専用プローブファイルへ一意なマーカーを書き込み、
//!    同一の `cargo build --release --locked` コマンドの壁時計時間を
//!    N=3 回計測する。
//! 3. 各サンプル後、生成バイナリのバイト列に当該マーカーが含まれることを
//!    検査し、「アセット変更が実際に反映された」ことをビルド成功以上に
//!    強く確認する（マーカー不在は fail-closed で終了コード 1）。
//! 4. 最大値を [`rws_dist_server::bench_support::LIMIT_SECONDS`] と比較し、
//!    1 行サマリを stdout へ出力して判定に応じた終了コードを返す。
//!
//! # ネストビルドのロック回避
//!
//! 本ベンチが起動する子 `cargo build` は `--target-dir
//! target/rebuild-latency-bench`（親 `cargo bench` の `target/` とは別ディレクトリ）
//! を使う。`dist-server/build.rs` が WASM ステージ用に `target/wasm-dist` を
//! 分離しているのと同じ考え方で、親プロセスが保持する `target/` ディレクトリ
//! ロックとの競合・デッドロックを避ける。
//!
//! # セキュリティ（OWASP A03 インジェクション対策）
//!
//! 子プロセス起動は固定引数・固定パスのみで構成し、外部入力を文字列連結で
//! シェルへ渡す経路は存在しない（`Command` の引数は配列として個別に渡され、
//! シェル展開を経由しない）。`RWS_WASM_BUILD` は値を解釈せず子プロセスへ
//! 透過するのみで、コマンドライン組み立てには一切関与しない。

use rws_dist_server::bench_support::{format_summary_line, judge, Sample};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// 計測サンプル数（PoC-4 の実測プロトコルに合わせた反復回数）。
const SAMPLE_COUNT: usize = 3;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("dist-server/ has a parent directory (workspace root)")
        .to_path_buf();

    // 親 `cargo bench` の `target/` とは別ディレクトリにし、ディレクトリ
    // ロックの競合を避ける（`build.rs` が `target/wasm-dist` を分離するのと
    // 同じ理由）。
    let bench_target_dir = workspace_root.join("target").join("rebuild-latency-bench");
    // プローブファイルは `static/` 配下の専用ファイル（`view-transitions.js`
    // 等の既存アセットには一切触れない）。パスは固定の定数結合のみで、
    // 外部入力からの組み立てを行わない（OWASP A01 パストラバーサル対策）。
    let probe_path = workspace_root
        .join("static")
        .join("rebuild-latency-probe.txt");

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    // ウォームビルド: 依存クレートのビルドキャッシュを温める（計測対象外）。
    // 失敗した場合はこの時点で fail-closed とする。
    if let Err(message) = run_build(&cargo, &workspace_root, &bench_target_dir) {
        eprintln!("rebuild-latency: warm build failed: {message}");
        std::process::exit(1);
    }

    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut failure: Option<String> = None;

    for i in 1..=SAMPLE_COUNT {
        // Drop ガードでプローブファイルを必ず削除する（panic 時含む）。
        // `static/` は本番アセット埋め込みテーブルの走査対象であり、
        // 一時ファイルの残留は次回ビルド・配信物の汚染につながるため。
        let marker = format!(
            "rebuild-latency-probe iteration={i} epoch_nanos={}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let _guard = match ProbeGuard::write(&probe_path, &marker) {
            Ok(guard) => guard,
            Err(message) => {
                failure = Some(format!("failed to write probe file: {message}"));
                break;
            }
        };

        let start = Instant::now();
        let build_result = run_build(&cargo, &workspace_root, &bench_target_dir);
        let elapsed = start.elapsed().as_secs_f64();

        if let Err(message) = build_result {
            failure = Some(format!("sample {i} build failed: {message}"));
            break;
        }

        let binary_path = bench_target_dir.join("release").join("dist-server");
        match binary_contains_marker(&binary_path, &marker) {
            Ok(true) => {}
            Ok(false) => {
                failure = Some(format!(
                    "sample {i}: rebuilt binary does not contain the probe marker \
                     (asset change was not reflected)"
                ));
                break;
            }
            Err(message) => {
                failure = Some(format!(
                    "sample {i}: failed to verify probe marker: {message}"
                ));
                break;
            }
        }

        samples.push(Sample { seconds: elapsed });
    }

    if let Some(message) = failure {
        eprintln!("rebuild-latency: {message}");
        std::process::exit(1);
    }

    let result = judge(&samples);
    let is_pass = result.is_pass();
    println!("{}", format_summary_line(&result));

    if !is_pass {
        std::process::exit(1);
    }
}

/// `cargo build --release --locked -p rws-dist-server --target-dir
/// <bench_target_dir>` を実行する。エラー時は内部パス等の機微情報を含まない
/// メッセージを返す（`security.md` 「機微情報の露出」観点）。
fn run_build(cargo: &str, workspace_root: &Path, target_dir: &Path) -> Result<(), String> {
    let status = Command::new(cargo)
        .current_dir(workspace_root)
        .args([
            "build",
            "--release",
            "--locked",
            "-p",
            "rws-dist-server",
            "--target-dir",
        ])
        .arg(target_dir)
        .status()
        .map_err(|e| format!("failed to spawn `cargo build`: {e}"))?;

    if !status.success() {
        return Err(
            "`cargo build --release --locked -p rws-dist-server` exited non-zero".to_string(),
        );
    }
    Ok(())
}

/// `binary_path` のバイト列に `marker` の UTF-8 バイト列がそのまま含まれるかを
/// 検査する。埋め込みテーブル（`build.rs` 生成・`include_bytes!` 経由）は
/// プローブファイルの内容をそのままバイナリへ焼き込むため、単純な部分列
/// 一致で「アセット変更が実際に反映されたか」を確認できる。
fn binary_contains_marker(binary_path: &Path, marker: &str) -> Result<bool, String> {
    let bytes = fs::read(binary_path).map_err(|e| format!("failed to read rebuilt binary: {e}"))?;
    let needle = marker.as_bytes();
    Ok(bytes.windows(needle.len()).any(|window| window == needle))
}

/// プローブファイルの RAII 削除ガード。書き込みから削除までのスコープを
/// 1 箇所に閉じ込め、panic 経路（`std::process::exit` を除く通常の巻き戻し）
/// でも `static/` 配下に一時ファイルを残さないようにする。
///
/// # 注意（`std::process::exit` との相互作用）
///
/// `main` 内の `std::process::exit` はスタック巻き戻しを行わないため、
/// exit 呼び出し時点で本ガードの `Drop` は実行されない。ただし exit を
/// 呼ぶ経路（ビルド失敗・マーカー不在・しきい値超過）はいずれもループ内で
/// ガードのスコープを抜けた後（`for` イテレーション終端で `_guard` が
/// 明示的にドロップされた後）に到達するため、実運用上プローブファイルは
/// 各イテレーション終了時点で既に削除済みである。次回実行時も上書き→削除の
/// サイクルで自己回復する（実装計画「検証方法」3 節）。
struct ProbeGuard {
    path: PathBuf,
}

impl ProbeGuard {
    fn write(path: &Path, content: &str) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create parent directory: {e}"))?;
        }
        fs::write(path, content).map_err(|e| format!("failed to write probe file: {e}"))?;
        Ok(Self {
            path: path.to_path_buf(),
        })
    }
}

impl Drop for ProbeGuard {
    fn drop(&mut self) {
        // 削除失敗（既に無い等)は無視する。プローブファイルは一時的な
        // 非配信対象データであり、削除失敗を理由にベンチ全体を失敗させる
        // 必要はない（次回実行時の上書き→削除で自己回復する）。
        let _ = fs::remove_file(&self.path);
    }
}

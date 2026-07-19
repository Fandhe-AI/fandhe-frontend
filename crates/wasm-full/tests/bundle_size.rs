//! REQ-11（`docs/spec/04-requirements.md`）の受け入れ基準「最小インタラクティブ
//! コンポーネントの WASM バンドルサイズ（gzip 後）が 200KB 以内であること」
//! （PoC-5 実績: WASM 完全方式 gzip 合計 27,703 B ≒ 27.1KB）を CI で継続計測する
//! テスト（TASK-11.6、イシュー #89）。
//!
//! # 計測経路と製品ビルドとの契約
//!
//! `dist-server/build.rs`（TASK-10.2b・イシュー #110）が本番配布物として実行する
//! のと同一のコマンド列
//! （`cargo build -p fandhe-frontend-wasm-full --target wasm32-unknown-unknown --release`
//! → `wasm-bindgen --target web --no-typescript --out-dir <dir>`）を、本テストは
//! `wasm-full` クレート自身の native 統合テストとして再現する。`dist-server` に
//! 依存させない（`wasm-full` 単体で TASK-11.6 の受け入れ基準を検証できる）ため、
//! ビルド出力先は `dist-server/build.rs` とは独立した
//! `target/bundle-size-check/` を使う（同一 `target/wasm-dist` を共有すると
//! `dist-server` のビルドと並行実行した際にディレクトリロックが競合しうる、
//! `dist-server/build.rs` の `run_wasm_build` コメント参照）。
//!
//! # 計測定義（PoC-5 と同一、`docs/spec/03-poc/wasm-runtime-split/README.md`
//! 「2. バンドルサイズの実測」節参照）
//!
//! `wasm-bindgen --target web` の出力ディレクトリに含まれる全ファイル
//! （`fandhe_frontend_wasm_full_bg.wasm`・`fandhe_frontend_wasm_full.js`）を**各ファイル個別に
//! `gzip -9` した圧縮後バイト数の合算**とする（実配信時は個別ファイルとして
//! 転送されるため、連結後の圧縮ではなく個別圧縮の合算を採用し、実配信バイト数を
//! 過小評価しないようにする PoC-5 の方針を踏襲する）。
//!
//! アプリ側グルー JS は製品側に未整備（イシュー #156、LOC 検証は別スコープ）
//! のため、現時点の計測対象には含めない。グルー JS が製品に追加された時点で
//! 本テストの計測対象へ含めることを検討する（`.claude/rules/
//! out-of-scope-tracking.md` の追跡対象）。
//!
//! # fail-closed 方針（`xtask/src/check_deps.rs` 等と同一の運用原則）
//!
//! しきい値（[`REQ11_BUNDLE_SIZE_LIMIT_BYTES`]）はコード定数のみが正であり、
//! 緩和用の環境変数・CLI 引数は設けない。wasm32 ターゲット・
//! `wasm-bindgen-cli`・`gzip` の不在やビルド失敗は、しきい値超過と同様に
//! テスト失敗として扱う（`unwrap`/`expect`/`assert!` によるフェイルクローズ。
//! `.claude/rules/coding-rust.md` が定めるとおりテストコードでの `unwrap`/
//! `expect` は許容される）。
//!
//! 唯一の明示的スキップ経路は `RWS_WASM_BUILD=0`（`skip`/`false` も同義、
//! 大文字小文字を区別しない）で、`dist-server/build.rs::wasm_build_enabled` と
//! 同一契約。wasm ツールチェーンが常設されない環境（例: 本リポジトリの
//! `forbid-unsafe` self-hosted ジョブ）向けの逃げ道であり、既定は有効
//! （フェイルクローズ側）を維持する。

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// REQ-11 受け入れ基準が定めるバンドルサイズの上限（gzip 後バイト数）。
///
/// 200KB = 200_000 バイト（10 進 KB。`xtask/src/check_image_size.rs` の
/// 10 進採用と整合し、2 進 KiB より厳しい安全側の定義）。上限緩和は
/// この定数を変更する PR（レビュー必須）以外の経路を設けない。
pub const REQ11_BUNDLE_SIZE_LIMIT_BYTES: u64 = 200_000;

/// 1 回の計測結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleSizeMeasurement {
    /// `wasm-bindgen` 出力ディレクトリ内の各ファイルを個別に `gzip -9` した
    /// 圧縮後バイト数の合算（PoC-5 と同一の計測定義）。
    pub total_gzip_bytes: u64,
    /// 計測対象ファイル数（空ディレクトリの合計 0 B が偽 PASS になる事故を
    /// 防ぐためのガードに使う、[`format_report`] にも含めて可視化する）。
    pub file_count: usize,
}

/// 上限判定結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// gzip 後合計サイズが上限以内。
    Pass(BundleSizeMeasurement, u64),
    /// gzip 後合計サイズが上限を超過。
    Fail(BundleSizeMeasurement, u64),
}

impl CheckResult {
    /// CI（`.github/workflows/ci.yml` の `bundle-size` ジョブ）が終了コードを
    /// 決定する際に参照する契約: `Pass` のみ成功、それ以外は失敗として扱う。
    pub fn is_pass(&self) -> bool {
        matches!(self, CheckResult::Pass(_, _))
    }
}

/// 実測値 `measurement` を上限 `limit_bytes` に照らして判定する純粋関数。
///
/// I/O を一切行わないため単体テストで境界値（ちょうど上限 / +1 / 0）を
/// 直接検証できる（`xtask/src/check_image_size.rs::judge` と同一パターン）。
pub fn judge(measurement: BundleSizeMeasurement, limit_bytes: u64) -> CheckResult {
    if measurement.total_gzip_bytes <= limit_bytes {
        CheckResult::Pass(measurement, limit_bytes)
    } else {
        CheckResult::Fail(measurement, limit_bytes)
    }
}

/// CI ログから機械抽出可能な 1 行サマリを整形する。
///
/// 書式
/// `bundle-size: total_gzip_bytes=<n>/<limit> files=<k> result=<PASS|FAIL>` は
/// `.github/workflows/ci.yml` の `bundle-size` ジョブが
/// `grep '^bundle-size:'` で抽出する契約であり、本ファイルの
/// `format_report_*` 単体テストで固定する。安易に変更しない。
pub fn format_report(result: &CheckResult) -> String {
    let (measurement, limit_bytes, verdict) = match result {
        CheckResult::Pass(m, limit) => (m, limit, "PASS"),
        CheckResult::Fail(m, limit) => (m, limit, "FAIL"),
    };
    format!(
        "bundle-size: total_gzip_bytes={}/{} files={} result={verdict}",
        measurement.total_gzip_bytes, limit_bytes, measurement.file_count
    )
}

/// `wasm-full/` の親ディレクトリ（ワークスペースルート）を返す。
///
/// `dist-server/build.rs::main` と同一の解決方法（`CARGO_MANIFEST_DIR` から
/// 一段上がる）。`cargo test` 実行時は `CARGO_MANIFEST_DIR` が本クレート
/// （`wasm-full/`）を指すことを前提とする。
fn workspace_root() -> PathBuf {
    // `crates/wasm-full/` から 2 段上でワークスペースルートに到達する
    // （イシュー #436、`crates/` 配下移設）。
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by `cargo test`"),
    );
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/wasm-full/ has a workspace root two levels up")
        .to_path_buf()
}

/// WASM ビルドステージが有効かどうかを環境変数 `RWS_WASM_BUILD` から判定する。
///
/// `dist-server/build.rs::wasm_build_enabled` と同一契約（`0`/`skip`/`false`
/// のいずれかで無効化、既定は有効）。両ファイルで判定ロジックを重複させて
/// いるのは、本テストが `dist-server` に依存させたくない（`wasm-full` 単体で
/// 完結させたい）ためで、契約の変更時は両方を合わせて更新すること。
fn wasm_build_enabled() -> bool {
    match env::var("RWS_WASM_BUILD") {
        Ok(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            !(normalized == "0" || normalized == "skip" || normalized == "false")
        }
        Err(_) => true,
    }
}

/// ネストした `cargo build -p fandhe-frontend-wasm-full --target wasm32-unknown-unknown
/// --release --locked` を実行し、生成された `.wasm` バイナリの絶対パスを返す。
///
/// `--target-dir` を `target/bundle-size-check/`（本テスト専用）にすることで、
/// 本テストを実行している外側の `cargo test` プロセスが保持する `target/`
/// ディレクトリロックとのデッドロックを避ける（`dist-server/build.rs::
/// run_wasm_build` が `target/wasm-dist` を使う理由と同一）。
///
/// # 環境の分離（決定性確保、`dist-server/build.rs::run_wasm_build` と同一契約）
///
/// `Command::env_clear()` で外部環境を一旦すべて遮断し、ビルドに最低限必要な
/// 変数（`PATH`/`HOME`/`CARGO_HOME`/`RUSTUP_HOME`/`RUSTUP_TOOLCHAIN`）のみを
/// 明示的に許可リストで引き継ぐ。これを怠ると、本テストを起動した外側の
/// `cargo test` プロセスの環境（例: CI の `RUSTFLAGS='-F unsafe_code'`）が
/// ネストビルドへそのまま伝播し、ここで計測する `.wasm`/glue 成果物が
/// `dist-server/build.rs`（REQ-11 サイズ計測の配布基準）が実際に生成する
/// ものと異なりうる（Cursor Bugbot 指摘、PR #248）。
fn build_wasm_full_release(workspace_root: &Path) -> PathBuf {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let target_dir = workspace_root.join("target").join("bundle-size-check");

    let mut command = Command::new(&cargo);
    command.env_clear();
    for key in [
        "PATH",
        "HOME",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "RUSTUP_TOOLCHAIN",
    ] {
        if let Ok(value) = env::var(key) {
            command.env(key, value);
        }
    }
    command
        .current_dir(workspace_root)
        .args([
            "build",
            "-p",
            "fandhe-frontend-wasm-full",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "--locked",
            "--target-dir",
        ])
        .arg(&target_dir);

    let status = command
        .status()
        .expect("failed to spawn nested `cargo build -p fandhe-frontend-wasm-full`");
    assert!(
        status.success(),
        "nested `cargo build -p fandhe-frontend-wasm-full --target wasm32-unknown-unknown --release` failed. \
         Ensure the wasm32-unknown-unknown target is installed: rustup target add wasm32-unknown-unknown"
    );

    target_dir
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("fandhe_frontend_wasm_full.wasm")
}

/// ワークスペースの `Cargo.lock` を std の文字列処理でパースし、解決済みの
/// `wasm-bindgen` クレートのバージョンを取得する。
///
/// `dist-server/build.rs::expected_wasm_bindgen_version` と同一の実装（TOML
/// パーサクレートを追加しない方針、`.claude/rules/coding-rust.md` の依存上限・
/// `core` 外部依存ゼロの精神を本テストにも適用）。契約の変更時は両方を
/// 合わせて更新すること。
fn expected_wasm_bindgen_version(workspace_root: &Path) -> String {
    let lock_path = workspace_root.join("Cargo.lock");
    let content = fs::read_to_string(&lock_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", lock_path.display()));

    for block in content.split("[[package]]") {
        let mut name = None;
        let mut version = None;
        for line in block.lines() {
            let line = line.trim();
            if let Some(value) = line.strip_prefix("name = ") {
                name = Some(value.trim_matches('"'));
            } else if let Some(value) = line.strip_prefix("version = ") {
                version = Some(value.trim_matches('"'));
            }
            if name.is_some() && version.is_some() {
                break;
            }
        }
        if name == Some("wasm-bindgen") {
            return version
                .unwrap_or_else(|| {
                    panic!("found a wasm-bindgen entry in Cargo.lock but it has no version field")
                })
                .to_string();
        }
    }

    panic!(
        "wasm-bindgen package not found in Cargo.lock (is wasm-full's dependency on it intact?)"
    );
}

/// インストール済み `wasm-bindgen-cli` のバージョン文字列を返す
/// （`dist-server/build.rs::installed_wasm_bindgen_cli_version` と同一実装）。
fn installed_wasm_bindgen_cli_version() -> String {
    let output = Command::new("wasm-bindgen")
        .arg("--version")
        .output()
        .expect(
            "wasm-bindgen-cli not found on PATH. Install it with: \
             cargo install wasm-bindgen-cli --version <version-matching-Cargo.lock> --locked",
        );
    assert!(
        output.status.success(),
        "`wasm-bindgen --version` exited with a non-zero status"
    );

    // 出力形式は `wasm-bindgen <version>`。末尾トークンをバージョンとして扱う。
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .rsplit(' ')
        .next()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| panic!("unexpected `wasm-bindgen --version` output format"))
        .to_string()
}

/// `Cargo.lock` が解決した `wasm-bindgen` バージョンと、PATH 上の
/// `wasm-bindgen-cli` バージョンが一致することを検証する。
///
/// `dist-server/build.rs::run_wasm_stage` の同種チェックを再現し fail-closed
/// にする（Cursor Bugbot 指摘、PR #248）: バージョン不一致のまま
/// `wasm-bindgen` を実行すると、実際の製品ビルド（`dist-server/build.rs`）は
/// ここで拒否するにもかかわらず、本テストはビルドを継続して 200KB ゲートを
/// 通過させてしまいうる（＝計測対象と配布物が乖離した状態を見逃す）。
fn verify_wasm_bindgen_version_matches_lockfile(workspace_root: &Path) {
    let expected_version = expected_wasm_bindgen_version(workspace_root);
    let installed_version = installed_wasm_bindgen_cli_version();
    assert_eq!(
        expected_version, installed_version,
        "wasm-bindgen-cli version mismatch: Cargo.lock resolves wasm-bindgen {expected_version}, \
         but `wasm-bindgen --version` reports {installed_version}. \
         Install the matching CLI with: cargo install wasm-bindgen-cli --version {expected_version} --locked"
    );
}

/// `wasm-bindgen --target web --no-typescript` を実行し、生成された JS グルー
/// コード・`_bg.wasm` を出力したディレクトリの絶対パスを返す。
///
/// `dist-server/build.rs::run_wasm_bindgen` と同一のフラグ構成（製品配布物と
/// 同一構成のバンドルを計測対象にするため）。出力先は本テスト専用の
/// `target/bundle-size-check/wasm-assets/`。
fn run_wasm_bindgen_for_bundle_size(wasm_binary_path: &Path, workspace_root: &Path) -> PathBuf {
    let out_dir = workspace_root
        .join("target")
        .join("bundle-size-check")
        .join("wasm-assets");
    fs::create_dir_all(&out_dir)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", out_dir.display()));

    let status = Command::new("wasm-bindgen")
        .args(["--target", "web", "--no-typescript", "--out-dir"])
        .arg(&out_dir)
        .arg(wasm_binary_path)
        .status()
        .expect(
            "wasm-bindgen-cli not found on PATH. Install it with: \
             cargo install wasm-bindgen-cli --version <version-matching-Cargo.lock> --locked",
        );
    assert!(
        status.success(),
        "wasm-bindgen failed to generate JS bindings for fandhe-frontend-wasm-full"
    );

    out_dir
}

/// `dir` 直下のファイル一覧（絶対パス）を返す。サブディレクトリは走査しない
/// （`wasm-bindgen --target web --no-typescript` の出力は現状フラットな構成。
/// 将来 `snippets/`（インライン JS スニペット）等のサブディレクトリが
/// 生成されるようになった場合、本関数はそれらを計測対象から漏らす
/// ＝合計を過小評価する側にのみ倒れる。過大評価にはならないため
/// fail-closed の安全側からは外れないが、計測対象を広げる際は本関数を
/// 再帰走査に変更すること）。
fn collect_output_files(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", dir.display()))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect()
}

/// `path` の内容をシステムの `gzip -9 -c` で圧縮し、圧縮後バイト数を返す。
///
/// 外部クレート（`flate2` 等）を追加しない設計判断（依存追加は事前承認が
/// 必要であり、自動計測のためだけに追加しない。REQ-3 依存上限にも
/// 影響させない）。`Command::arg` で個別引数として渡すため、ファイル名に
/// シェルメタ文字が含まれてもシェル解釈は発生しない（security.md
/// 「インジェクション」観点、`xtask/src/check_image_size.rs::measure` と
/// 同一の安全設計）。
fn gzip_compressed_size(path: &Path) -> u64 {
    let output = Command::new("gzip")
        .args(["-9", "-c"])
        .arg(path)
        .output()
        .expect("failed to spawn `gzip` (system `gzip` command is required for this test)");
    assert!(
        output.status.success(),
        "`gzip -9 -c {}` exited non-zero",
        path.display()
    );
    output.stdout.len() as u64
}

#[cfg(test)]
mod judge_and_format_report_tests {
    use super::*;

    fn measurement(total_gzip_bytes: u64, file_count: usize) -> BundleSizeMeasurement {
        BundleSizeMeasurement {
            total_gzip_bytes,
            file_count,
        }
    }

    #[test]
    fn judge_passes_when_zero_bytes() {
        let result = judge(measurement(0, 0), REQ11_BUNDLE_SIZE_LIMIT_BYTES);
        assert!(result.is_pass());
    }

    #[test]
    fn judge_passes_when_exactly_at_limit() {
        let result = judge(
            measurement(REQ11_BUNDLE_SIZE_LIMIT_BYTES, 2),
            REQ11_BUNDLE_SIZE_LIMIT_BYTES,
        );
        assert!(result.is_pass());
    }

    #[test]
    fn judge_fails_when_one_byte_over_limit() {
        let result = judge(
            measurement(REQ11_BUNDLE_SIZE_LIMIT_BYTES + 1, 2),
            REQ11_BUNDLE_SIZE_LIMIT_BYTES,
        );
        assert!(!result.is_pass());
    }

    #[test]
    fn judge_passes_at_poc5_measured_value() {
        // PoC-5 実績（gzip 合計 27,703 B）を回帰の基準値として固定する。
        let result = judge(measurement(27_703, 2), REQ11_BUNDLE_SIZE_LIMIT_BYTES);
        assert!(result.is_pass());
    }

    #[test]
    fn format_report_matches_fixed_format_for_pass() {
        let result = judge(measurement(27_703, 2), REQ11_BUNDLE_SIZE_LIMIT_BYTES);
        assert_eq!(
            format_report(&result),
            "bundle-size: total_gzip_bytes=27703/200000 files=2 result=PASS"
        );
    }

    #[test]
    fn format_report_matches_fixed_format_for_fail() {
        let result = judge(measurement(300_000, 2), REQ11_BUNDLE_SIZE_LIMIT_BYTES);
        assert_eq!(
            format_report(&result),
            "bundle-size: total_gzip_bytes=300000/200000 files=2 result=FAIL"
        );
    }
}

/// TASK-11.6・REQ-11 の受け入れ基準本体。`RWS_WASM_BUILD` が明示的に無効化
/// されていない限り、製品ビルドと同一のコマンド列で `fandhe-frontend-wasm-full` を
/// ビルド・`wasm-bindgen` 変換し、実測 gzip 合計サイズが 200KB 以内であることを
/// アサートする（fail-closed。詳細はファイル冒頭の doc comment 参照）。
#[test]
fn wasm_full_bundle_gzip_size_within_req11_limit() {
    if !wasm_build_enabled() {
        eprintln!(
            "bundle-size: skipped (RWS_WASM_BUILD is disabled; wasm toolchain not assumed present)"
        );
        return;
    }

    let workspace_root = workspace_root();
    // `dist-server/build.rs::run_wasm_stage` と同じく、glue コード生成に不整合が
    // 出ないよう `wasm-bindgen` 実行前にバージョン一致を fail-closed で検証する。
    verify_wasm_bindgen_version_matches_lockfile(&workspace_root);
    let wasm_binary_path = build_wasm_full_release(&workspace_root);
    let assets_dir = run_wasm_bindgen_for_bundle_size(&wasm_binary_path, &workspace_root);

    let files = collect_output_files(&assets_dir);
    // 空計測ガード: 出力ディレクトリが空（≒ビルドが実質何も生成していない）
    // 場合に合計 0 B で偽 PASS になる事故を防ぐ。少なくとも `.wasm` 本体が
    // 1 つ存在することを要求する。
    let has_wasm_file = files
        .iter()
        .any(|path| path.extension().and_then(|ext| ext.to_str()) == Some("wasm"));
    assert!(
        has_wasm_file,
        "expected at least one .wasm file in wasm-bindgen output directory {}, found: {:?}",
        assets_dir.display(),
        files
    );

    let total_gzip_bytes: u64 = files.iter().map(|path| gzip_compressed_size(path)).sum();
    let measurement = BundleSizeMeasurement {
        total_gzip_bytes,
        file_count: files.len(),
    };

    let result = judge(measurement, REQ11_BUNDLE_SIZE_LIMIT_BYTES);
    let report = format_report(&result);
    // `cargo test -- --nocapture` で標準出力へ、`.github/workflows/ci.yml` の
    // `bundle-size` ジョブが `grep '^bundle-size:'` で抽出する 1 行サマリ。
    println!("{report}");
    assert!(result.is_pass(), "{report}");
}

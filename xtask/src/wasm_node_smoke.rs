//! `wasm-node-smoke` サブコマンド: `docs/wasm-build-integration.md` §6.4 が
//! 手順として文書化していた「`rws-wasm-thin`（`web-sys` 非依存な薄い JS グルー
//! 方式ランタイム）を Node.js から `require()` して素早くロジック確認する」
//! 開発フローを、イシュー #297（TASK-10.2 残課題、出典 PR #220 §10 スコープ外
//! 節）で `cargo xtask` サブコマンドとして自動化したもの。
//!
//! # 呼び出し元
//!
//! `.github/workflows/ci.yml` の `wasm-node-smoke` ジョブが `cargo xtask
//! wasm-node-smoke` を実行し、CI ゲートとして使う。開発者は手元でも同じ
//! コマンドを実行できる（`--build-only` で node 実行をスキップし wasm32
//! ビルドのみ確認可能）。
//!
//! # 処理の流れ
//!
//! 1. [`resolve_expected_wasm_bindgen_version`]（`Cargo.lock` 解決済み
//!    `wasm-bindgen` クレートのバージョン抽出）と
//!    [`installed_wasm_bindgen_cli_version`]（`wasm-bindgen --version`）を
//!    突き合わせ、完全一致を要求する。これは `dist-server/build.rs::
//!    expected_wasm_bindgen_version` と同一の契約（バージョン不一致の CLI が
//!    生成する JS グルーコードは壊れ得るため）。固定値ドリフトの前倒し検出は
//!    `xtask/tests/wasm_bindgen_version_sync.rs` が別途 `cargo test` 時点で
//!    行う。
//! 2. `--build-only` 指定時を除き [`check_node_available`] で `node` の存在を
//!    確認する。
//! 3. [`run_wasm32_build`] で `cargo build --target wasm32-unknown-unknown -p
//!    rws-wasm-thin`（debug プロファイル）を実行する。
//! 4. [`cargo_metadata_target_directory`] で `cargo metadata` の
//!    `target_directory` を取得し、成果物パス・`wasm-bindgen` 出力先を
//!    そこから解決する（共有 `CARGO_TARGET_DIR`、`.claude/rules/ci.md`
//!    環境下でも正しく動作させるため、`target/` を決め打ちしない）。
//! 5. [`run_wasm_bindgen_nodejs`] で `--target nodejs` のバインディングを
//!    `<target_directory>/wasm-node/thin` へ生成する（出力先は定数固定。
//!    §6.4 不変条件「`static/`・埋め込み入力への混入禁止」をコードで強制）。
//! 6. `--build-only` 時を除き [`run_node_check`] で `node -e "<固定スクリプト>"`
//!    を実行し、`require()` → `initial_html()` が非空文字列を返すこと、
//!    および `apply("set_draft", "<script>alert(1)</script>")` の戻り値に
//!    生の `<script>` タグが含まれない（既定エスケープ済み、REQ-1）ことを
//!    検証する。スクリプト本文は定数文字列でユーザー入力を連結しない
//!    （payload はスクリプト内のリテラルであり、外部入力ではない）。
//!
//! # 契約（`xtask/src/main.rs` の他サブコマンドと統一）
//!
//! - 終了コード 0: 全ステップ成功（PASS）
//! - 終了コード 1: ツール不在・バージョン不一致・ビルド失敗・bindgen 失敗・
//!   node 実行失敗・エスケープ検証失敗のいずれも fail-closed
//! - 終了コード 2: 不明な引数（`--build-only` 以外は受け付けない）
//! - stdout 1 行サマリ: [`format_report`] が生成する
//!   `wasm-node-smoke: package=rws-wasm-thin target=nodejs mode=<full|build-only>
//!   result=<PASS|FAIL>` 形式。CI が `grep '^wasm-node-smoke:'` で抽出できる。
//!
//! 判定対象クレート（[`PACKAGE_NAME`]）・出力先（[`wasm_bindgen_out_dir`]）は
//! 定数固定であり、判定を弱める CLI 引数・環境変数は設けない
//! （`check-loc`/`check-core-deps` と同じ設計原則）。

use crate::json::{self, Json};
use std::env;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// スモーク対象のクレート（`web-sys` 非依存な薄い JS グルー方式の参照実装、
/// `wasm-thin/Cargo.toml` 参照）。CLI 引数での差し替えは意図的にサポートしない。
pub const PACKAGE_NAME: &str = "rws-wasm-thin";

/// `cargo build` が生成する `.wasm` ファイル名（クレート名のハイフンが
/// アンダースコアに正規化される cargo の既定挙動に一致させる）。
pub const WASM_ARTIFACT_STEM: &str = "rws_wasm_thin";

/// `--build-only` の有無を表すモード。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmokeMode {
    /// wasm32 ビルド + bindgen + node 実行確認まで行う（既定）。
    Full,
    /// wasm32 ビルド + bindgen までに留め、node 実行はスキップする。
    BuildOnly,
}

impl SmokeMode {
    /// [`format_report`] のサマリ行に埋め込むラベル（`full`/`build-only`）。
    pub fn label(self) -> &'static str {
        match self {
            SmokeMode::Full => "full",
            SmokeMode::BuildOnly => "build-only",
        }
    }
}

/// このサブコマンド専用のエラー型。fail-closed の観点から、呼び出し元
/// （`xtask/src/main.rs::run_wasm_node_smoke`）はすべての `Err` を
/// 終了コード 1 に落とし込む。
#[derive(Debug)]
pub struct SmokeError(String);

impl fmt::Display for SmokeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SmokeError {
    fn new(message: impl Into<String>) -> Self {
        SmokeError(message.into())
    }
}

/// `wasm-node-smoke: package=<pkg> target=nodejs mode=<mode> result=<PASS|FAIL>\n`
/// 形式の 1 行サマリを生成する。CI（`.github/workflows/ci.yml` の
/// `wasm-node-smoke` ジョブ）はこの行を `grep '^wasm-node-smoke:'` で抽出する
/// 契約（`check_loc::format_loc_report` 等、他サブコマンドと同一パターン）。
pub fn format_report(mode: SmokeMode, passed: bool) -> String {
    let verdict = if passed { "PASS" } else { "FAIL" };
    format!(
        "wasm-node-smoke: package={PACKAGE_NAME} target=nodejs mode={} result={verdict}\n",
        mode.label()
    )
}

/// `Cargo.lock` の内容から `[[package]] name = "wasm-bindgen"` ブロックの
/// `version` を厳密抽出する（`wasm-bindgen-backend`/`wasm-bindgen-macro` 等の
/// 前方一致する別パッケージを誤って拾わないよう完全一致で探す）。
///
/// `xtask/tests/wasm_bindgen_version_sync.rs` の同名ロジックと同型だが、
/// あちらはテスト用に `panic!` で早期失敗する一方、本関数は呼び出し元
/// （`run_wasm_node_smoke`）が fail-closed で終了コード 1 に変換できるよう
/// `Result` を返す。
pub fn extract_wasm_bindgen_version_from_lock(contents: &str) -> Result<String, SmokeError> {
    let lines: Vec<&str> = contents.lines().collect();
    let name_line_index = lines
        .iter()
        .position(|line| line.trim() == "name = \"wasm-bindgen\"")
        .ok_or_else(|| {
            SmokeError::new(
                "Cargo.lock に [[package]] name = \"wasm-bindgen\" ブロックが見つからない",
            )
        })?;

    let version_line = lines.get(name_line_index + 1).ok_or_else(|| {
        SmokeError::new("Cargo.lock の wasm-bindgen ブロックに version 行が続いていない")
    })?;

    version_line
        .trim()
        .strip_prefix("version = \"")
        .and_then(|rest| rest.strip_suffix('"'))
        .map(str::to_owned)
        .ok_or_else(|| {
            SmokeError::new(format!(
                "Cargo.lock の wasm-bindgen ブロックで name の直後が version 行に \
                 なっていない: {version_line}"
            ))
        })
}

/// ワークスペースルート（カレントディレクトリ）の `Cargo.lock` を読み、
/// [`extract_wasm_bindgen_version_from_lock`] でバージョンを取り出す。
///
/// `cargo xtask` は他サブコマンド（`check_loc::measure_file` 等）と同様、
/// ワークスペースルートから実行される前提でカレントディレクトリ相対の
/// パスを使う。
fn resolve_expected_wasm_bindgen_version() -> Result<String, SmokeError> {
    let contents = std::fs::read_to_string("Cargo.lock")
        .map_err(|e| SmokeError::new(format!("failed to read Cargo.lock: {e}")))?;
    extract_wasm_bindgen_version_from_lock(&contents)
}

/// `wasm-bindgen --version` の出力（例: `wasm-bindgen 0.2.126`）からバージョン
/// 文字列のみを取り出す。
pub fn parse_wasm_bindgen_version_output(output: &str) -> Result<String, SmokeError> {
    output
        .split_whitespace()
        .next_back()
        .map(str::to_owned)
        .ok_or_else(|| {
            SmokeError::new(format!(
                "unexpected `wasm-bindgen --version` output: {output:?}"
            ))
        })
}

/// `wasm-bindgen --version` を実行し、インストール済み CLI のバージョンを
/// 取得する。CLI 不在・実行失敗は fail-closed で `Err` を返す
/// （`dist-server/build.rs::installed_wasm_bindgen_cli_version` と同型の契約）。
fn installed_wasm_bindgen_cli_version() -> Result<String, SmokeError> {
    let output = Command::new("wasm-bindgen")
        .arg("--version")
        .output()
        .map_err(|e| {
            SmokeError::new(format!(
                "failed to spawn `wasm-bindgen --version`: {e}. \
                 Install wasm-bindgen-cli matching Cargo.lock's wasm-bindgen version."
            ))
        })?;
    if !output.status.success() {
        return Err(SmokeError::new(format!(
            "`wasm-bindgen --version` exited with {status}",
            status = output.status
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_wasm_bindgen_version_output(stdout.trim())
}

/// `Cargo.lock` 解決済みバージョンとインストール済み CLI のバージョンが
/// 完全一致することを検証する（`dist-server/build.rs::expected_wasm_bindgen_version`
/// と同一契約）。不一致・取得失敗はいずれも `Err`。
fn verify_wasm_bindgen_version() -> Result<(), SmokeError> {
    let expected = resolve_expected_wasm_bindgen_version()?;
    let installed = installed_wasm_bindgen_cli_version()?;
    if expected != installed {
        return Err(SmokeError::new(format!(
            "wasm-bindgen-cli version mismatch: Cargo.lock resolves wasm-bindgen {expected}, \
             but `wasm-bindgen --version` reports {installed}. \
             Install the matching CLI with: cargo install wasm-bindgen-cli --version {expected} --locked"
        )));
    }
    Ok(())
}

/// `node --version` の実行で Node.js の存在を確認する。`--build-only`
/// 指定時はこの検査自体をスキップする（呼び出し元 `run` が判定）。
fn check_node_available() -> Result<(), SmokeError> {
    let output = Command::new("node")
        .arg("--version")
        .output()
        .map_err(|e| {
            SmokeError::new(format!(
                "failed to spawn `node --version`: {e}. \
             Node.js is required for the node execution check (use --build-only to skip it)."
            ))
        })?;
    if !output.status.success() {
        return Err(SmokeError::new(format!(
            "`node --version` exited with {status}",
            status = output.status
        )));
    }
    Ok(())
}

/// `cargo metadata --no-deps --format-version 1 --locked` の `target_directory`
/// を取得する。共有 `CARGO_TARGET_DIR`（`.claude/rules/ci.md`）環境でも
/// 正しく解決させるため、`target/` を決め打ちしない。
fn cargo_metadata_target_directory() -> Result<PathBuf, SmokeError> {
    let cargo_bin = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(cargo_bin)
        .args(["metadata", "--no-deps", "--format-version", "1", "--locked"])
        .output()
        .map_err(|e| SmokeError::new(format!("failed to run cargo metadata: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SmokeError::new(format!(
            "cargo metadata exited with {status}: {stderr}",
            status = output.status
        )));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| SmokeError::new(format!("cargo metadata output is not valid UTF-8: {e}")))?;
    let parsed = json::parse(&stdout)
        .map_err(|e| SmokeError::new(format!("failed to parse cargo metadata output: {e}")))?;
    extract_target_directory(&parsed)
}

/// `cargo metadata` の JSON から `target_directory` フィールドを取り出す
/// 純粋関数（`cargo_metadata_target_directory` から分離してユニットテスト可能にする）。
pub fn extract_target_directory(metadata: &Json) -> Result<PathBuf, SmokeError> {
    metadata
        .get("target_directory")
        .and_then(Json::as_str)
        .map(PathBuf::from)
        .ok_or_else(|| {
            SmokeError::new("cargo metadata output is missing string field `target_directory`")
        })
}

/// `<target_directory>/wasm32-unknown-unknown/debug/rws_wasm_thin.wasm`
/// （debug プロファイル固定、§6.4 の実績コマンド踏襲）を解決する。
fn wasm_artifact_path(target_directory: &Path) -> PathBuf {
    target_directory
        .join("wasm32-unknown-unknown")
        .join("debug")
        .join(format!("{WASM_ARTIFACT_STEM}.wasm"))
}

/// `wasm-bindgen --target nodejs` の出力先。`target_directory` 配下に固定し
/// CLI 引数で差し替え不可とする（§6.4 不変条件「`static/`・埋め込み入力への
/// 混入禁止」をコードで強制。`check-core-deps`/`check-loc` と同じ「判定対象は
/// 引数で差し替え不可」原則）。
fn wasm_bindgen_out_dir(target_directory: &Path) -> PathBuf {
    target_directory.join("wasm-node").join("thin")
}

/// `cargo build --target wasm32-unknown-unknown -p rws-wasm-thin`
/// （debug プロファイル、§6.4 の実績コマンド踏襲）を実行する。
fn run_wasm32_build() -> Result<(), SmokeError> {
    let cargo_bin = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo_bin)
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "-p",
            PACKAGE_NAME,
        ])
        .status()
        .map_err(|e| {
            SmokeError::new(format!(
                "failed to spawn `cargo build -p {PACKAGE_NAME}`: {e}"
            ))
        })?;
    if !status.success() {
        return Err(SmokeError::new(format!(
            "`cargo build --target wasm32-unknown-unknown -p {PACKAGE_NAME}` failed. \
             Ensure the wasm32-unknown-unknown target is installed: \
             rustup target add wasm32-unknown-unknown"
        )));
    }
    Ok(())
}

/// `wasm-bindgen --target nodejs --out-dir <out_dir> <wasm_path>` を実行する。
fn run_wasm_bindgen_nodejs(wasm_path: &Path, out_dir: &Path) -> Result<(), SmokeError> {
    std::fs::create_dir_all(out_dir)
        .map_err(|e| SmokeError::new(format!("failed to create {}: {e}", out_dir.display())))?;

    let status = Command::new("wasm-bindgen")
        .args(["--target", "nodejs", "--out-dir"])
        .arg(out_dir)
        .arg(wasm_path)
        .status()
        .map_err(|e| SmokeError::new(format!("failed to spawn wasm-bindgen: {e}")))?;
    if !status.success() {
        return Err(SmokeError::new(
            "wasm-bindgen failed to generate nodejs bindings for rws-wasm-thin".to_string(),
        ));
    }
    Ok(())
}

/// `node -e "<固定スクリプト>"` を実行し、`require()` → `initial_html()` が
/// 非空文字列を返すこと、および `apply("set_draft", "<script>alert(1)</script>")`
/// の戻り値に生の `<script>` タグが含まれない（既定エスケープ済み、REQ-1）こと
/// を検証する。
///
/// スクリプト本文は定数文字列（[`NODE_CHECK_SCRIPT`]）でユーザー入力を連結
/// しない。`require()` に渡す対象は `wasm-bindgen --target nodejs` が
/// `--out-dir` 直下に生成するエントリ JS ファイル自体（`wasm-bindgen` は
/// `package.json`/`index.js` を生成しないため、ディレクトリを直接 `require()`
/// できない）。このパス（信頼できる自プログラムの計算結果であり外部入力
/// ではない）のみを `-- <path>` の形で追加引数として渡し、スクリプト側は
/// `process.argv[1]` から読み取る（security.md A03: インジェクション対策
/// として、文字列補間ではなく argv 経由で渡す）。
fn run_node_check(out_dir: &Path) -> Result<(), SmokeError> {
    let entry_js = out_dir.join(format!("{WASM_ARTIFACT_STEM}.js"));
    let status = Command::new("node")
        .arg("-e")
        .arg(NODE_CHECK_SCRIPT)
        .arg("--")
        .arg(&entry_js)
        .status()
        .map_err(|e| SmokeError::new(format!("failed to spawn node: {e}")))?;
    if !status.success() {
        return Err(SmokeError::new(
            "node execution check failed (see above output for details)".to_string(),
        ));
    }
    Ok(())
}

/// [`run_node_check`] が `node -e` に渡す固定スクリプト。ユーザー入力は含まず、
/// `apply()` へのエスケープ検証ペイロード（`<script>alert(1)</script>`）も
/// スクリプト内のリテラルである。
const NODE_CHECK_SCRIPT: &str = r#"
const entryJsPath = process.argv[1];
const mod = require(entryJsPath);

const html = mod.initial_html();
if (typeof html !== 'string' || html.length === 0) {
  console.error('wasm-node-smoke: initial_html() returned an empty or non-string value');
  process.exit(1);
}

const payload = '<script>alert(1)</script>';
const applied = mod.apply('set_draft', payload);
if (typeof applied !== 'string') {
  console.error('wasm-node-smoke: apply() did not return a string');
  process.exit(1);
}
if (applied.includes('<script>')) {
  console.error('wasm-node-smoke: apply() output contains an unescaped <script> tag (default-escape regression, REQ-1)');
  process.exit(1);
}

console.log('wasm-node-smoke: node execution check passed');
"#;

/// `wasm-node-smoke` サブコマンド本体。`xtask/src/main.rs::run_wasm_node_smoke`
/// が引数を解釈した後の [`SmokeMode`] を受け取り、判定結果を返す。
///
/// `Ok(true)`: 全ステップ成功（PASS）。`Ok(false)` は本関数からは返らず、
/// 失敗は常に `Err` として伝播する（呼び出し元が [`format_report`] で FAIL の
/// 1 行サマリを出力してから終了コード 1 にする）。
pub fn run(mode: SmokeMode) -> Result<(), SmokeError> {
    // 1. バージョン整合検査（ビルド前に高速失敗、fail-closed）。
    verify_wasm_bindgen_version()?;

    // 2. node 実行が必要なモードでは事前に存在確認する（wasm32 ビルドという
    //    重い処理の前に高速失敗させるため、ビルドより先に検査する）。
    if matches!(mode, SmokeMode::Full) {
        check_node_available()?;
    }

    // 3. wasm32 ビルド。
    run_wasm32_build()?;

    // 4. 成果物パス・出力先の解決（共有 CARGO_TARGET_DIR 環境に対応するため
    //    cargo metadata の target_directory を都度取得する）。
    let target_directory = cargo_metadata_target_directory()?;
    let wasm_path = wasm_artifact_path(&target_directory);
    let out_dir = wasm_bindgen_out_dir(&target_directory);

    // 5. --target nodejs バインディング生成。
    run_wasm_bindgen_nodejs(&wasm_path, &out_dir)?;

    // 6. node 実行確認（--build-only 時はスキップ）。
    if matches!(mode, SmokeMode::Full) {
        run_node_check(&out_dir)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_report_matches_summary_contract_pass_full() {
        let report = format_report(SmokeMode::Full, true);
        assert_eq!(
            report,
            "wasm-node-smoke: package=rws-wasm-thin target=nodejs mode=full result=PASS\n"
        );
    }

    #[test]
    fn format_report_matches_summary_contract_fail_build_only() {
        let report = format_report(SmokeMode::BuildOnly, false);
        assert_eq!(
            report,
            "wasm-node-smoke: package=rws-wasm-thin target=nodejs mode=build-only result=FAIL\n"
        );
    }

    #[test]
    fn extract_wasm_bindgen_version_from_lock_finds_exact_match() {
        let contents = r#"
[[package]]
name = "wasm-bindgen-backend"
version = "9.9.9"

[[package]]
name = "wasm-bindgen"
version = "0.2.126"

[[package]]
name = "wasm-bindgen-macro"
version = "0.2.126"
"#;
        let version = extract_wasm_bindgen_version_from_lock(contents).unwrap();
        assert_eq!(version, "0.2.126");
    }

    #[test]
    fn extract_wasm_bindgen_version_from_lock_errors_when_missing() {
        let contents = r#"
[[package]]
name = "serde"
version = "1.0.0"
"#;
        let err = extract_wasm_bindgen_version_from_lock(contents).unwrap_err();
        assert!(err.to_string().contains("wasm-bindgen"));
    }

    #[test]
    fn parse_wasm_bindgen_version_output_extracts_last_token() {
        let version = parse_wasm_bindgen_version_output("wasm-bindgen 0.2.126").unwrap();
        assert_eq!(version, "0.2.126");
    }

    #[test]
    fn parse_wasm_bindgen_version_output_errors_on_empty_input() {
        assert!(parse_wasm_bindgen_version_output("").is_err());
    }

    #[test]
    fn extract_target_directory_reads_string_field() {
        let metadata = Json::Object(vec![(
            "target_directory".to_string(),
            Json::String("/workspace/target".to_string()),
        )]);
        let dir = extract_target_directory(&metadata).unwrap();
        assert_eq!(dir, PathBuf::from("/workspace/target"));
    }

    #[test]
    fn extract_target_directory_errors_when_missing() {
        let metadata = Json::Object(vec![]);
        assert!(extract_target_directory(&metadata).is_err());
    }

    #[test]
    fn wasm_artifact_path_uses_debug_profile_and_stem() {
        let path = wasm_artifact_path(Path::new("/workspace/target"));
        assert_eq!(
            path,
            PathBuf::from("/workspace/target/wasm32-unknown-unknown/debug/rws_wasm_thin.wasm")
        );
    }

    #[test]
    fn wasm_bindgen_out_dir_is_fixed_under_target_directory() {
        let dir = wasm_bindgen_out_dir(Path::new("/workspace/target"));
        assert_eq!(dir, PathBuf::from("/workspace/target/wasm-node/thin"));
    }

    #[test]
    fn smoke_mode_label_matches_cli_flag_vocabulary() {
        assert_eq!(SmokeMode::Full.label(), "full");
        assert_eq!(SmokeMode::BuildOnly.label(), "build-only");
    }
}

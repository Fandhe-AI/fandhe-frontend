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
//! → `wasm-bindgen --target web --no-typescript --remove-name-section
//! --remove-producers-section --out-dir <dir>` → 有効時は `wasm-opt -Os`、
//! イシュー #1971）を、本テストは `wasm-full` クレート自身の native 統合
//! テストとして再現する。`dist-server` に依存させない（`wasm-full` 単体で
//! TASK-11.6 の受け入れ基準を検証できる）ため、ビルド出力先は
//! `dist-server/build.rs` とは独立した `target/bundle-size-check/` を使う
//! （同一 `target/wasm-dist` を共有すると `dist-server` のビルドと並行実行
//! した際にディレクトリロックが競合しうる、`dist-server/build.rs` の
//! `run_wasm_build` コメント参照）。`wasm-opt` の適用可否は
//! `dist-server/build.rs::detect_wasm_opt` と同一の soft-skip 判定
//! （PATH 上に見つかった場合のみ適用）を独立実装として複製しており、
//! `dist-server` 側の後処理契約を変更する場合は本ファイルも合わせて
//! 更新すること（そうしないと本テストが未適用構成を測り続け、実配布物と
//! 乖離した数値を「REQ-11 準拠」として報告してしまう）。
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
//! しきい値（[`REQ11_BUNDLE_SIZE_LIMIT_BYTES`] と警告しきい値
//! [`REQ11_BUNDLE_SIZE_WARN_BYTES`]）はいずれもコード定数のみが正であり、
//! 緩和用の環境変数・CLI 引数は設けない。wasm32 ターゲット・
//! `wasm-bindgen-cli`・`gzip` の不在やビルド失敗は、しきい値超過と同様に
//! テスト失敗として扱う（`unwrap`/`expect`/`assert!` によるフェイルクローズ。
//! `.claude/rules/coding-rust.md` が定めるとおりテストコードでの `unwrap`/
//! `expect` は許容される）。
//!
//! # 判定契約（PASS / PASS+警告 / FAIL の三値、イシュー #1968）
//!
//! [`judge`] は実測が上限（[`REQ11_BUNDLE_SIZE_LIMIT_BYTES`]）を**超えた**
//! ときのみ [`CheckResult::Fail`] を返す（[`CheckResult::is_pass`] が
//! `false`＝CI 終了コード非 0）。実測が警告しきい値
//! （[`REQ11_BUNDLE_SIZE_WARN_BYTES`] = 上限の 95%）を**超え**、かつ上限以内
//! のときは [`CheckResult::PassWithWarning`] を返し、[`is_pass`] は `true`
//! のまま（CI は失敗させない）[`CheckResult::is_warning`] のみ `true` に
//! なる。[`format_report`] はこの区別を 1 行サマリ末尾の ` warn=above-95pct`
//! （[`REQ11_BUNDLE_SIZE_WARN_TAG`]）の有無で表現し、`.github/workflows/
//! ci.yml` の `bundle-size` ジョブがこのタグを `grep` して
//! `::warning::` ワークフローコマンドへ変換する（アノテーション発火自体は
//! ci.yml 側の責務であり、本ファイルはタグ付き 1 行サマリの出力までを担う）。
//!
//! 唯一の明示的スキップ経路は `FANDHE_FRONTEND_WASM_BUILD=0`（`skip`/`false` も同義、
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

/// REQ-11 上限の 95% にあたる警告しきい値（gzip 後バイト数、イシュー
/// #1968）。超過時は [`CheckResult::Fail`] にはせず
/// [`CheckResult::PassWithWarning`] として可視化するのみで、上限自体
/// （[`REQ11_BUNDLE_SIZE_LIMIT_BYTES`]）と同様に緩和用の環境変数・CLI 引数は
/// 設けない。
pub const REQ11_BUNDLE_SIZE_WARN_BYTES: u64 = 190_000;

/// 警告しきい値超過を示す 1 行サマリ末尾のタグ。`.github/workflows/ci.yml`
/// の `bundle-size` ジョブがこの文字列を `grep` して `::warning::` を出す
/// 唯一の正（[`format_report`] 側の出力とここでの定義がずれると ci.yml の
/// grep が発火しなくなるため、変更時は両方を合わせて更新すること）。
pub const REQ11_BUNDLE_SIZE_WARN_TAG: &str = "above-95pct";

// 警告しきい値が上限以上になる誤設定（警告が上限超過後にしか出ない、また
// は FAIL より先に警告が発火しない状態）をコンパイル時に fail-closed で
// 弾く不変条件。
const _: () = assert!(REQ11_BUNDLE_SIZE_WARN_BYTES < REQ11_BUNDLE_SIZE_LIMIT_BYTES);

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

/// 上限判定結果（PASS / PASS+警告 / FAIL の三値、イシュー #1968）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// gzip 後合計サイズが警告しきい値以内。
    Pass(BundleSizeMeasurement, u64),
    /// gzip 後合計サイズが上限以内だが警告しきい値（上限の 95%）を超過。
    /// CI の終了コードには影響させない（[`CheckResult::is_pass`] は
    /// `true`）が、[`CheckResult::is_warning`] で検知できる。
    PassWithWarning(BundleSizeMeasurement, u64, u64),
    /// gzip 後合計サイズが上限を超過。
    Fail(BundleSizeMeasurement, u64),
}

impl CheckResult {
    /// CI（`.github/workflows/ci.yml` の `bundle-size` ジョブ）が終了コードを
    /// 決定する際に参照する契約: `Pass` と `PassWithWarning` は成功、
    /// `Fail` のみ失敗として扱う（警告は FAIL にしない）。
    pub fn is_pass(&self) -> bool {
        matches!(
            self,
            CheckResult::Pass(_, _) | CheckResult::PassWithWarning(_, _, _)
        )
    }

    /// 警告しきい値（上限の 95%）を超過しているかどうか。`PassWithWarning`
    /// のみ `true`（`ci.yml` 側が `::warning::` を出すかどうかの判断材料）。
    pub fn is_warning(&self) -> bool {
        matches!(self, CheckResult::PassWithWarning(_, _, _))
    }
}

/// 実測値 `measurement` を上限 `limit_bytes`・警告しきい値 `warn_bytes` に
/// 照らして判定する純粋関数。
///
/// I/O を一切行わないため単体テストで境界値（ちょうど上限 / +1 / 警告
/// しきい値ちょうど / +1 / 0）を直接検証できる
/// （`xtask/src/check_image_size.rs::judge` と同一パターン）。
pub fn judge(measurement: BundleSizeMeasurement, limit_bytes: u64, warn_bytes: u64) -> CheckResult {
    if measurement.total_gzip_bytes > limit_bytes {
        CheckResult::Fail(measurement, limit_bytes)
    } else if measurement.total_gzip_bytes > warn_bytes {
        CheckResult::PassWithWarning(measurement, limit_bytes, warn_bytes)
    } else {
        CheckResult::Pass(measurement, limit_bytes)
    }
}

/// CI ログから機械抽出可能な 1 行サマリを整形する。
///
/// 書式
/// `bundle-size: total_gzip_bytes=<n>/<limit> files=<k> result=<PASS|FAIL>`
/// （PASS かつ警告しきい値超過のときのみ末尾に
/// ` warn=above-95pct`〔[`REQ11_BUNDLE_SIZE_WARN_TAG`]〕が付く）は
/// `.github/workflows/ci.yml` の `bundle-size` ジョブが
/// `grep '^bundle-size:'`／`grep '^bundle-size:.*warn=above-95pct'` で
/// 抽出する契約であり、本ファイルの `format_report_*` 単体テストで固定する。
/// 安易に変更しない。
pub fn format_report(result: &CheckResult) -> String {
    let (measurement, limit_bytes, verdict, is_warning) = match result {
        CheckResult::Pass(m, limit) => (m, limit, "PASS", false),
        CheckResult::PassWithWarning(m, limit, _warn) => (m, limit, "PASS", true),
        CheckResult::Fail(m, limit) => (m, limit, "FAIL", false),
    };
    let mut line = format!(
        "bundle-size: total_gzip_bytes={}/{} files={} result={verdict}",
        measurement.total_gzip_bytes, limit_bytes, measurement.file_count
    );
    if is_warning {
        line.push_str(" warn=");
        line.push_str(REQ11_BUNDLE_SIZE_WARN_TAG);
    }
    line
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

/// WASM ビルドステージが有効かどうかを環境変数 `FANDHE_FRONTEND_WASM_BUILD`
/// から判定する薄いラッパ。判定本体は [`wasm_build_enabled_for`]（純関数）に
/// 分離してあり、`dist-server/src/wasm_build_gate.rs::wasm_build_enabled_for`
/// と同一契約（`0`/`skip`/`false` のいずれかで無効化、既定は有効）。
/// 両ファイルで判定ロジックを重複させているのは、本テストが `dist-server` に
/// 依存させたくない（`wasm-full` 単体で完結させたい）ためで、契約の変更時は
/// 両方を合わせて更新すること（#437 で `RWS_WASM_BUILD` →
/// `FANDHE_FRONTEND_WASM_BUILD` へ改名した際も両ファイルを同時更新した）。
fn wasm_build_enabled() -> bool {
    wasm_build_enabled_for(env::var("FANDHE_FRONTEND_WASM_BUILD").ok().as_deref())
}

/// [`wasm_build_enabled`] の判定本体。環境変数の実読み取りを行わない純関数と
/// することで、環境変数のミューテーションを伴わない決定的なユニットテスト
/// （`None`＝未設定・`Some("0")` 等）を可能にする。
fn wasm_build_enabled_for(env_value: Option<&str>) -> bool {
    match env_value {
        Some(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            !(normalized == "0" || normalized == "skip" || normalized == "false")
        }
        None => true,
    }
}

#[cfg(test)]
mod wasm_build_enabled_tests {
    use super::wasm_build_enabled_for;

    /// 未設定（新名 `FANDHE_FRONTEND_WASM_BUILD` を一切指定しない状態）は
    /// 既定で有効（安全側）であることを固定する回帰テスト（#437）。
    #[test]
    fn unset_defaults_to_enabled() {
        assert!(wasm_build_enabled_for(None));
    }

    #[test]
    fn explicit_disable_values_disable_the_stage() {
        for value in ["0", "skip", "false", "SKIP", "FALSE"] {
            assert!(
                !wasm_build_enabled_for(Some(value)),
                "expected {value:?} to disable the wasm build stage"
            );
        }
    }

    #[test]
    fn other_values_keep_the_stage_enabled() {
        for value in ["1", "true", "yes", ""] {
            assert!(
                wasm_build_enabled_for(Some(value)),
                "expected {value:?} to keep the wasm build stage enabled"
            );
        }
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

/// `wasm-bindgen` に渡す固定引数。`dist-server/build.rs::WASM_BINDGEN_ARGS` と
/// 同一配列を独立実装として複製している（本テストは `dist-server` に
/// 依存させない設計、ファイル冒頭「計測経路と製品ビルドとの契約」参照）。
/// フラグ構成を変更する場合は両ファイルを揃えて更新すること（イシュー
/// #1971。片方だけ更新すると計測側が未適用構成を測り続け、期待値だけを
/// 実配布物と乖離した値へ更新してしまう）。
const WASM_BINDGEN_ARGS: &[&str] = &[
    "--target",
    "web",
    "--no-typescript",
    "--remove-name-section",
    "--remove-producers-section",
];

/// `wasm-bindgen` を実行し、生成された JS グルーコード・`_bg.wasm` を出力した
/// ディレクトリの絶対パスを返す。
///
/// `dist-server/build.rs::run_wasm_bindgen` と同一のフラグ構成
/// （[`WASM_BINDGEN_ARGS`]、製品配布物と同一構成のバンドルを計測対象にする
/// ため）。出力先は本テスト専用の `target/bundle-size-check/wasm-assets/`。
fn run_wasm_bindgen_for_bundle_size(wasm_binary_path: &Path, workspace_root: &Path) -> PathBuf {
    let out_dir = workspace_root
        .join("target")
        .join("bundle-size-check")
        .join("wasm-assets");
    fs::create_dir_all(&out_dir)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", out_dir.display()));

    let status = Command::new("wasm-bindgen")
        .args(WASM_BINDGEN_ARGS)
        .arg("--out-dir")
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

/// バイト列が WASM バイナリのマジックナンバー（`\0asm`、4 バイト）で始まるか
/// を判定する。`dist-server/src/wasm_stage_cache.rs::looks_like_wasm` と同一
/// 実装（本テストを `dist-server` に依存させない設計のための複製）。
fn looks_like_wasm(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && &bytes[..4] == b"\0asm"
}

/// PATH 上の `wasm-opt`（binaryen）を検出する。
/// `dist-server/build.rs::detect_wasm_opt` と同一の 3 分岐契約
/// （未検出は `Ok(None)`＝soft-skip、検出したが実行失敗は `Err`＝hard fail、
/// 検出・正常終了は `Ok(Some(バージョン文字列))`）。契約を変更する場合は
/// 両ファイルを揃えて更新すること（イシュー #1971）。
fn detect_wasm_opt() -> Result<Option<String>, String> {
    let output = match Command::new("wasm-opt").arg("--version").output() {
        Ok(output) => output,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(format!(
                "failed to spawn `wasm-opt --version` (found on PATH but could not execute it): {err}"
            ));
        }
    };

    if !output.status.success() {
        return Err("`wasm-opt --version` exited with a non-zero status".to_string());
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        return Err("`wasm-opt --version` produced no output".to_string());
    }

    Ok(Some(version))
}

/// `wasm-opt -Os` を `wasm_binary_path` の file stem から求めた
/// `<stem>_bg.wasm`（`assets_dir` 配下）に適用し、意味論を変えずにサイズのみ
/// 縮める。`dist-server/build.rs::run_wasm_opt` と同一の soft-skip 設計
/// （一時ファイル経由の atomic 置換・失敗時 hard fail）を、本テスト専用の
/// `target/bundle-size-check/wasm-opt-tmp/` を使って再現する。
///
/// 呼び出し元（[`wasm_full_bundle_gzip_size_within_req11_limit`]）は
/// [`detect_wasm_opt`] が `Some` を返した場合のみ本関数を呼ぶ。
fn apply_wasm_opt(wasm_binary_path: &Path, assets_dir: &Path, workspace_root: &Path) {
    let stem = wasm_binary_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_else(|| {
            panic!(
                "could not determine file stem of {}",
                wasm_binary_path.display()
            )
        });
    let bg_wasm = assets_dir.join(format!("{stem}_bg.wasm"));

    let tmp_dir = workspace_root
        .join("target")
        .join("bundle-size-check")
        .join("wasm-opt-tmp");
    fs::create_dir_all(&tmp_dir)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", tmp_dir.display()));
    let tmp_path = tmp_dir.join(format!("{stem}_bg.wasm"));

    let status = Command::new("wasm-opt")
        .arg("-Os")
        .arg(&bg_wasm)
        .arg("-o")
        .arg(&tmp_path)
        .status()
        .expect("failed to spawn wasm-opt");
    if !status.success() {
        let _ = fs::remove_file(&tmp_path);
        panic!("wasm-opt failed while optimizing {}", bg_wasm.display());
    }

    let optimized = fs::read(&tmp_path)
        .unwrap_or_else(|e| panic!("failed to read wasm-opt output {}: {e}", tmp_path.display()));
    if !looks_like_wasm(&optimized) {
        let _ = fs::remove_file(&tmp_path);
        panic!(
            "wasm-opt produced an unexpected (empty or non-wasm) output at {}",
            tmp_path.display()
        );
    }

    fs::rename(&tmp_path, &bg_wasm).unwrap_or_else(|e| {
        panic!(
            "failed to replace {} with wasm-opt output: {e}",
            bg_wasm.display()
        )
    });
}

/// `wasm-opt` 適用結果の 1 行サマリを整形する純粋関数。
/// `format_report` とは別行（イシュー #1971。`ci.yml` 側の grep 契約は
/// `bundle-size:` 行を変えないため、`bundle-size-wasm-opt:` という別の
/// プレフィックスを使う）。
pub fn format_wasm_opt_report(wasm_opt_version: Option<&str>) -> String {
    match wasm_opt_version {
        Some(version) => format!("bundle-size-wasm-opt: result=applied version={version}"),
        None => "bundle-size-wasm-opt: result=skipped reason=not-found".to_string(),
    }
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
        let result = judge(
            measurement(0, 0),
            REQ11_BUNDLE_SIZE_LIMIT_BYTES,
            REQ11_BUNDLE_SIZE_WARN_BYTES,
        );
        assert!(result.is_pass());
        assert!(!result.is_warning());
    }

    #[test]
    fn judge_passes_when_exactly_at_limit() {
        // ちょうど上限は警告しきい値（190,000 B）も超えているため、PASS で
        // あると同時に警告扱いになる（イシュー #1968）。
        let result = judge(
            measurement(REQ11_BUNDLE_SIZE_LIMIT_BYTES, 2),
            REQ11_BUNDLE_SIZE_LIMIT_BYTES,
            REQ11_BUNDLE_SIZE_WARN_BYTES,
        );
        assert!(result.is_pass());
        assert!(result.is_warning());
    }

    #[test]
    fn judge_fails_when_one_byte_over_limit() {
        let result = judge(
            measurement(REQ11_BUNDLE_SIZE_LIMIT_BYTES + 1, 2),
            REQ11_BUNDLE_SIZE_LIMIT_BYTES,
            REQ11_BUNDLE_SIZE_WARN_BYTES,
        );
        assert!(!result.is_pass());
        assert!(!result.is_warning());
    }

    #[test]
    fn judge_passes_at_poc5_measured_value() {
        // PoC-5 実績（gzip 合計 27,703 B）を回帰の基準値として固定する。
        let result = judge(
            measurement(27_703, 2),
            REQ11_BUNDLE_SIZE_LIMIT_BYTES,
            REQ11_BUNDLE_SIZE_WARN_BYTES,
        );
        assert!(result.is_pass());
        assert!(!result.is_warning());
    }

    /// 警告しきい値が上限の 95% ちょうどであることを固定する回帰テスト
    /// （イシュー #1968）。
    #[test]
    fn warn_threshold_is_95_percent_of_limit() {
        assert_eq!(
            REQ11_BUNDLE_SIZE_WARN_BYTES,
            REQ11_BUNDLE_SIZE_LIMIT_BYTES * 95 / 100
        );
    }

    #[test]
    fn judge_passes_without_warning_when_exactly_at_warn_threshold() {
        let result = judge(
            measurement(REQ11_BUNDLE_SIZE_WARN_BYTES, 2),
            REQ11_BUNDLE_SIZE_LIMIT_BYTES,
            REQ11_BUNDLE_SIZE_WARN_BYTES,
        );
        assert!(result.is_pass());
        assert!(!result.is_warning());
    }

    #[test]
    fn judge_warns_when_one_byte_over_warn_threshold() {
        let result = judge(
            measurement(REQ11_BUNDLE_SIZE_WARN_BYTES + 1, 2),
            REQ11_BUNDLE_SIZE_LIMIT_BYTES,
            REQ11_BUNDLE_SIZE_WARN_BYTES,
        );
        assert!(result.is_pass());
        assert!(result.is_warning());
    }

    #[test]
    fn format_report_matches_fixed_format_for_pass() {
        let result = judge(
            measurement(27_703, 2),
            REQ11_BUNDLE_SIZE_LIMIT_BYTES,
            REQ11_BUNDLE_SIZE_WARN_BYTES,
        );
        assert_eq!(
            format_report(&result),
            "bundle-size: total_gzip_bytes=27703/200000 files=2 result=PASS"
        );
    }

    #[test]
    fn format_report_matches_fixed_format_for_pass_with_warning() {
        let result = judge(
            measurement(195_000, 2),
            REQ11_BUNDLE_SIZE_LIMIT_BYTES,
            REQ11_BUNDLE_SIZE_WARN_BYTES,
        );
        assert_eq!(
            format_report(&result),
            "bundle-size: total_gzip_bytes=195000/200000 files=2 result=PASS warn=above-95pct"
        );
    }

    #[test]
    fn format_report_matches_fixed_format_for_fail() {
        let result = judge(
            measurement(300_000, 2),
            REQ11_BUNDLE_SIZE_LIMIT_BYTES,
            REQ11_BUNDLE_SIZE_WARN_BYTES,
        );
        assert_eq!(
            format_report(&result),
            "bundle-size: total_gzip_bytes=300000/200000 files=2 result=FAIL"
        );
    }

    /// `format_wasm_opt_report` の書式固定（イシュー #1971）。`applied` は
    /// `wasm-opt --version` の出力をそのまま含む。
    #[test]
    fn format_wasm_opt_report_matches_fixed_format_for_applied() {
        assert_eq!(
            format_wasm_opt_report(Some("wasm-opt version 129")),
            "bundle-size-wasm-opt: result=applied version=wasm-opt version 129"
        );
    }

    #[test]
    fn format_wasm_opt_report_matches_fixed_format_for_skipped() {
        assert_eq!(
            format_wasm_opt_report(None),
            "bundle-size-wasm-opt: result=skipped reason=not-found"
        );
    }
}

/// TASK-11.6・REQ-11 の受け入れ基準本体。`FANDHE_FRONTEND_WASM_BUILD` が明示的に無効化
/// されていない限り、製品ビルドと同一のコマンド列で `fandhe-frontend-wasm-full` を
/// ビルド・`wasm-bindgen` 変換し、実測 gzip 合計サイズが 200KB 以内であることを
/// アサートする（fail-closed。詳細はファイル冒頭の doc comment 参照）。
#[test]
fn wasm_full_bundle_gzip_size_within_req11_limit() {
    if !wasm_build_enabled() {
        eprintln!(
            "bundle-size: skipped (FANDHE_FRONTEND_WASM_BUILD is disabled; wasm toolchain not assumed present)"
        );
        return;
    }

    let workspace_root = workspace_root();
    // `dist-server/build.rs::run_wasm_stage` と同じく、glue コード生成に不整合が
    // 出ないよう `wasm-bindgen` 実行前にバージョン一致を fail-closed で検証する。
    verify_wasm_bindgen_version_matches_lockfile(&workspace_root);
    let wasm_binary_path = build_wasm_full_release(&workspace_root);
    let assets_dir = run_wasm_bindgen_for_bundle_size(&wasm_binary_path, &workspace_root);

    // `dist-server/build.rs::run_wasm_stage` と同一の後処理契約（イシュー
    // #1971）: PATH 上に `wasm-opt` が見つかった場合のみ追加適用する
    // soft-skip。「dist-server と同一構成を計測する」契約（ファイル冒頭
    // 「計測経路と製品ビルドとの契約」参照）を保つため、製品ビルド側が
    // 適用する後処理はここでも同じ条件で適用する。
    let wasm_opt_version =
        detect_wasm_opt().unwrap_or_else(|message| panic!("wasm-opt detection failed: {message}"));
    if let Some(version) = &wasm_opt_version {
        apply_wasm_opt(&wasm_binary_path, &assets_dir, &workspace_root);
        eprintln!("bundle-size: applied wasm-opt -Os ({version})");
    }
    println!("{}", format_wasm_opt_report(wasm_opt_version.as_deref()));

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

    let result = judge(
        measurement,
        REQ11_BUNDLE_SIZE_LIMIT_BYTES,
        REQ11_BUNDLE_SIZE_WARN_BYTES,
    );
    let report = format_report(&result);
    // `cargo test -- --nocapture` で標準出力へ、`.github/workflows/ci.yml` の
    // `bundle-size` ジョブが `grep '^bundle-size:'` で抽出する 1 行サマリ。
    println!("{report}");
    assert!(result.is_pass(), "{report}");
}

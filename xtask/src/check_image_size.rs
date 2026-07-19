//! REQ-9（単一バイナリ配布と Docker イメージ最小化、`docs/spec/04-requirements.md`）の
//! 受け入れ基準「`scratch` または `distroless` ベースの Docker イメージサイズが
//! 50MB 以内であること（PoC-4 実績: 2.19MB）」を CI で継続計測するモジュール
//! （TASK-9.3b、イシュー #103）。
//!
//! `check_deps.rs` / `check_loc.rs` と同じ運用原則を踏襲する: しきい値
//! （[`REQ9_IMAGE_SIZE_LIMIT_BYTES`]）はコード定数のみが正であり、CLI 引数
//! （`--limit-mb`）は動作確認・段階導入のための上書きを許容するが、既定値は
//! 常にこの定数を使う（`xtask/src/main.rs` の `run_check_image_size` 参照）。
//! 計測失敗（docker 不在・`docker image inspect` 失敗・出力パース失敗）は
//! しきい値超過と同様に fail-closed（終了コード 1）として扱う
//! （security.md「フェイルセーフ」参照）。
//!
//! 計測対象は `docker image inspect --format {{.Size}} <TAG>` が返す
//! 非圧縮イメージサイズ（バイト）。ビルド自体（`docker build`）はこのモジュールの
//! 責務外で、呼び出し元（CI ワークフロー・利用者）が事前に行う契約とする。

use std::fmt;
use std::process::Command;

/// REQ-9 受け入れ基準が定めるイメージサイズの上限（バイト）。
///
/// 50MB = 50_000_000 バイト（10 進 MB）。`docker images` / `docker image inspect`
/// が表示・返却する単位系（10 進）と一致させ、2 進 MiB（52_428_800 バイト）より
/// 厳しい安全側の定義を採用する。上限緩和のためのコード変更以外の経路
/// （環境変数等）は意図的に設けない。
pub const REQ9_IMAGE_SIZE_LIMIT_BYTES: u64 = 50_000_000;

/// 1 イメージのサイズ計測結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageSizeMeasurement {
    /// 計測対象イメージのタグ・参照名（呼び出し元が指定した文字列そのもの）。
    pub image: String,
    /// `docker image inspect --format {{.Size}}` が返す非圧縮サイズ（バイト）。
    pub size_bytes: u64,
}

/// 上限判定結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// イメージサイズが上限以内。
    Pass(ImageSizeMeasurement, u64),
    /// イメージサイズが上限を超過。
    Fail(ImageSizeMeasurement, u64),
}

impl CheckResult {
    /// CI（`.github/workflows/image-size.yml`）が終了コードを決定する際に
    /// 参照する契約: `Pass` のみ成功、それ以外は失敗として扱う。
    pub fn is_pass(&self) -> bool {
        matches!(self, CheckResult::Pass(_, _))
    }
}

/// 実測値 `measurement` を上限 `limit_bytes` に照らして判定する純粋関数。
///
/// I/O を一切行わないため単体テストで境界値（ちょうど上限 / +1 / 0）を
/// 直接検証できる。上限は呼び出し元が渡す値をそのまま使う
/// （既定値は [`REQ9_IMAGE_SIZE_LIMIT_BYTES`]、`xtask/src/main.rs` 参照）。
pub fn judge(measurement: ImageSizeMeasurement, limit_bytes: u64) -> CheckResult {
    if measurement.size_bytes <= limit_bytes {
        CheckResult::Pass(measurement, limit_bytes)
    } else {
        CheckResult::Fail(measurement, limit_bytes)
    }
}

/// CI ログから機械抽出可能な 1 行サマリを整形する。
///
/// 書式（`image-size: image=<tag> size_bytes=<n>/<limit> size_mb=<x.xx> result=<PASS|FAIL>`）は
/// `.github/workflows/image-size.yml` が `grep '^image-size:'` で抽出する契約であり、
/// `xtask/tests/cli_check_image_size.rs` で固定する。安易に変更しない。
pub fn format_report(result: &CheckResult) -> String {
    let (measurement, limit_bytes, verdict) = match result {
        CheckResult::Pass(m, limit) => (m, limit, "PASS"),
        CheckResult::Fail(m, limit) => (m, limit, "FAIL"),
    };
    let size_mb = measurement.size_bytes as f64 / 1_000_000.0;
    format!(
        "image-size: image={} size_bytes={}/{} size_mb={size_mb:.2} result={verdict}\n",
        measurement.image, measurement.size_bytes, limit_bytes
    )
}

/// 計測失敗を表すエラー。fail-closed の観点から、docker 不在・`inspect` の
/// 非ゼロ終了・出力のパース失敗のいずれも呼び出し元（`run_check_image_size`）で
/// 終了コード 1 に落とし込む。
#[derive(Debug)]
pub enum CheckImageSizeError {
    /// `docker` コマンドの起動自体に失敗した（未インストール等）。
    Spawn(std::io::Error),
    /// `docker image inspect` が非ゼロ終了コードを返した。
    CommandFailed { stderr: String },
    /// 標準出力を UTF-8 として解釈できなかった。
    InvalidUtf8,
    /// 標準出力を非負整数（バイト数）としてパースできなかった。
    InvalidSize { raw: String },
}

impl fmt::Display for CheckImageSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckImageSizeError::Spawn(e) => {
                write!(f, "failed to spawn `docker image inspect`: {e}")
            }
            CheckImageSizeError::CommandFailed { stderr } => {
                write!(f, "`docker image inspect` exited non-zero: {stderr}")
            }
            CheckImageSizeError::InvalidUtf8 => {
                write!(f, "`docker image inspect` output was not valid UTF-8")
            }
            CheckImageSizeError::InvalidSize { raw } => {
                write!(f, "could not parse image size from output: `{raw}`")
            }
        }
    }
}

impl std::error::Error for CheckImageSizeError {}

/// `image`（`docker build -t <tag> .` 等で作成済みのイメージ参照）のサイズを
/// `docker image inspect --format {{.Size}}` で計測する。
///
/// `image` は `Command::arg` で個別引数として渡し、シェルを経由しないため、
/// 呼び出し元がタグ文字列にシェルメタ文字を含めてもコマンドインジェクションには
/// つながらない（security.md「インジェクション」参照）。
pub fn measure(image: &str) -> Result<ImageSizeMeasurement, CheckImageSizeError> {
    let output = Command::new("docker")
        .arg("image")
        .arg("inspect")
        .arg("--format")
        .arg("{{.Size}}")
        .arg(image)
        .output()
        .map_err(CheckImageSizeError::Spawn)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(CheckImageSizeError::CommandFailed { stderr });
    }

    let stdout = String::from_utf8(output.stdout).map_err(|_| CheckImageSizeError::InvalidUtf8)?;
    let trimmed = stdout.trim();
    let size_bytes: u64 = trimmed
        .parse()
        .map_err(|_| CheckImageSizeError::InvalidSize {
            raw: trimmed.to_string(),
        })?;

    Ok(ImageSizeMeasurement {
        image: image.to_string(),
        size_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measurement(size_bytes: u64) -> ImageSizeMeasurement {
        ImageSizeMeasurement {
            image: "fandhe-frontend-dist-server:ci".to_string(),
            size_bytes,
        }
    }

    #[test]
    fn exactly_at_limit_passes() {
        let m = measurement(REQ9_IMAGE_SIZE_LIMIT_BYTES);
        assert!(judge(m, REQ9_IMAGE_SIZE_LIMIT_BYTES).is_pass());
    }

    #[test]
    fn one_byte_over_limit_fails() {
        let m = measurement(REQ9_IMAGE_SIZE_LIMIT_BYTES + 1);
        assert!(!judge(m, REQ9_IMAGE_SIZE_LIMIT_BYTES).is_pass());
    }

    #[test]
    fn zero_bytes_passes() {
        let m = measurement(0);
        assert!(judge(m, REQ9_IMAGE_SIZE_LIMIT_BYTES).is_pass());
    }

    #[test]
    fn poc4_scale_measurement_passes() {
        // PoC-4 実績（2.19MB）近傍のサイズで PASS することを確認する。
        let m = measurement(2_190_000);
        assert!(judge(m, REQ9_IMAGE_SIZE_LIMIT_BYTES).is_pass());
    }

    #[test]
    fn format_report_matches_summary_contract_pass() {
        let m = measurement(2_190_000);
        let report = format_report(&judge(m, REQ9_IMAGE_SIZE_LIMIT_BYTES));
        assert_eq!(
            report,
            "image-size: image=fandhe-frontend-dist-server:ci size_bytes=2190000/50000000 size_mb=2.19 result=PASS\n"
        );
    }

    #[test]
    fn format_report_matches_summary_contract_fail() {
        let m = measurement(60_000_000);
        let report = format_report(&judge(m, REQ9_IMAGE_SIZE_LIMIT_BYTES));
        assert_eq!(
            report,
            "image-size: image=fandhe-frontend-dist-server:ci size_bytes=60000000/50000000 size_mb=60.00 result=FAIL\n"
        );
    }

    #[test]
    fn measure_fails_closed_for_nonexistent_image() {
        // 存在しないイメージ名は `docker image inspect` が非ゼロ終了する
        // （もしくは docker 自体が未インストールなら Spawn エラー）ため、
        // いずれの経路でも Err を返し fail-closed であることを確認する。
        let result = measure("fandhe-frontend-image-size-test-does-not-exist:__missing__");
        assert!(result.is_err());
    }
}

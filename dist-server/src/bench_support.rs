//! `benches/rebuild_latency.rs`（TASK-10.4a、イシュー #119、REQ-10）の判定・整形ロジック。
//!
//! REQ-10 の受け入れ基準「本番ビルドのアセット変更反映（差分ビルド）が
//! 5 秒以内」（`docs/spec/04-requirements.md`、PoC-4 実測 0.571〜0.597 秒）を
//! 継続計測するベンチマークの純粋関数部分をここへ切り出す。
//!
//! ベンチ本体（`benches/rebuild_latency.rs`）は `[[bench]] test = false` の
//! ため `cargo test` では実行されない。しきい値判定・サマリ整形はここへ
//! 分離し、`cargo test -p rws-dist-server` から直接ユニットテストで固定する
//! （`xtask::check_loc` の `format_loc_report` 契約テストと同型のパターン）。
//!
//! CI（`.github/workflows/ci.yml` の `rebuild-latency` ジョブ）は本モジュールが
//! 定義する 1 行サマリを `grep '^rebuild-latency:'` で抽出し
//! `$GITHUB_STEP_SUMMARY` へ転記する契約であり、書式を変更する場合は
//! 当該ジョブも合わせて更新すること。
//!
//! ライブラリの公開 API 面を汚さないよう `#[doc(hidden)]` とし、ベンチ・
//! テストからのみ参照される内部ユーティリティであることを明示する。
//!
//! # 判定統計量: min-of-N（イシュー #294）
//!
//! 共有 self-hosted runner（6 並列）上では他ジョブとの CPU 競合により、
//! N=3 サンプルのうち 1 つだけが環境ノイズで跳ねることがある
//! （実測: PR #291 で 5.494s → rerun で 2.x〜4.x 秒台に収束）。従来は
//! 最大値（max）を `LIMIT_SECONDS` と比較していたため、この種の間欠的な
//! ノイズだけで FAIL していた。
//!
//! 本モジュールは判定基準を **最小値（min）** に変更する。恒常的な性能
//! リグレッション（全サンプルが 5 秒を超える）は min でも確実に検出でき、
//! 受け入れ基準「リグレッション検出力の維持」を満たす一方、1 サンプルのみの
//! 環境ノイズによる誤 FAIL は吸収できる。トレードオフ（間欠的にのみ遅くなる
//! 製品リグレッションは min では検出できない）への緩和として、サマリには
//! 引き続き全サンプル値と `max_s`（最大値）を出力し、Step Summary 上で
//! 目視追跡できるようにする（`.claude/rules/security.md` 「セキュリティ設定
//! ミス」観点: しきい値緩和ではなく判定統計量の変更であり、`LIMIT_SECONDS`
//! 自体・fail-closed 方針は変更しない）。

/// REQ-10 受け入れ基準が定める差分ビルド反映時間の上限（秒）。
///
/// しきい値はこの定数のみが正であり、CLI 引数・環境変数による緩和経路は
/// 設けない（`deps-check`/`loc-check` ジョブと同一の運用原則、
/// `.claude/rules/security.md` 「セキュリティ設定ミス」観点）。
pub const LIMIT_SECONDS: f64 = 5.0;

/// 1 サンプル（プローブ変更 → 差分ビルド完了）の壁時計計測結果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// 計測秒数（`std::time::Instant` 由来）。
    pub seconds: f64,
}

/// [`LIMIT_SECONDS`] に照らした判定結果。
///
/// 判定は最小値（`min_seconds`）基準（イシュー #294、min-of-N 採用の
/// 根拠はモジュールドキュメント参照）。`max_seconds` は判定には使わないが、
/// 間欠的な製品リグレッションを Step Summary 上で目視追跡できるよう
/// 観測性のために保持する。
#[derive(Debug, Clone, PartialEq)]
pub enum CheckResult {
    /// 全サンプルの最小値がしきい値以内。
    Pass {
        samples: Vec<Sample>,
        min_seconds: f64,
        max_seconds: f64,
    },
    /// 最小値がしきい値を超過（＝全サンプルが超過、恒常的なリグレッション）。
    Fail {
        samples: Vec<Sample>,
        min_seconds: f64,
        max_seconds: f64,
    },
}

impl CheckResult {
    /// `benches/rebuild_latency.rs` が終了コードを決定する際に参照する契約:
    /// `Pass` のみ成功、それ以外は失敗として扱う（fail-closed）。
    pub fn is_pass(&self) -> bool {
        matches!(self, CheckResult::Pass { .. })
    }
}

/// `samples`（計測順）を [`LIMIT_SECONDS`] に照らして判定する純粋関数。
///
/// I/O を一切行わないため、境界値（ちょうど 5.0 秒 / 5.001 秒）を単体テストで
/// 直接検証できる。空スライスは呼び出し元の契約違反であり `Fail`
/// （min=0.0 / max=0.0）として扱う（fail-closed。計測が 1 件も取れなかった
/// 場合に PASS 側へ倒れないようにするため）。
///
/// 判定基準は最小値（min-of-N、イシュー #294）。共有 runner の CPU 競合に
/// よる間欠的な単一サンプルの跳ねを吸収しつつ、全サンプルが超過する
/// 恒常的なリグレッションは引き続き検出する（モジュールドキュメント参照）。
pub fn judge(samples: &[Sample]) -> CheckResult {
    let min_seconds = samples.iter().map(|s| s.seconds).fold(f64::MAX, f64::min);
    let max_seconds = samples.iter().map(|s| s.seconds).fold(f64::MIN, f64::max);
    let (min_seconds, max_seconds) = if samples.is_empty() {
        (0.0, 0.0)
    } else {
        (min_seconds, max_seconds)
    };
    let samples_vec = samples.to_vec();

    if !samples.is_empty() && min_seconds <= LIMIT_SECONDS {
        CheckResult::Pass {
            samples: samples_vec,
            min_seconds,
            max_seconds,
        }
    } else {
        CheckResult::Fail {
            samples: samples_vec,
            min_seconds,
            max_seconds,
        }
    }
}

/// CI ログから機械抽出可能な 1 行サマリを整形する。
///
/// 書式（`rebuild-latency: samples=<n> s1=<x> s2=<x> ... min_s=<x> max_s=<x>
/// limit_s=<x> result=<PASS|FAIL>`）は `.github/workflows/ci.yml` の
/// `rebuild-latency` ジョブが `grep '^rebuild-latency:'` で抽出する契約であり、
/// `#[cfg(test)]` 下のユニットテストで固定する。安易に変更しない。
///
/// `min_s`（イシュー #294 で判定基準に採用）と `max_s`（観測性のため保持。
/// 間欠的な製品リグレッションの目視追跡用）の両方を出力する。
pub fn format_summary_line(result: &CheckResult) -> String {
    let (samples, min_seconds, max_seconds, verdict) = match result {
        CheckResult::Pass {
            samples,
            min_seconds,
            max_seconds,
        } => (samples, *min_seconds, *max_seconds, "PASS"),
        CheckResult::Fail {
            samples,
            min_seconds,
            max_seconds,
        } => (samples, *min_seconds, *max_seconds, "FAIL"),
    };

    let mut line = format!("rebuild-latency: samples={}", samples.len());
    for (i, sample) in samples.iter().enumerate() {
        line.push_str(&format!(" s{}={:.3}", i + 1, sample.seconds));
    }
    line.push_str(&format!(
        " min_s={min_seconds:.3} max_s={max_seconds:.3} limit_s={LIMIT_SECONDS:.1} result={verdict}"
    ));
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(seconds: f64) -> Sample {
        Sample { seconds }
    }

    #[test]
    fn all_samples_within_limit_pass() {
        let samples = vec![s(0.5), s(0.6), s(0.55)];
        assert!(judge(&samples).is_pass());
    }

    #[test]
    fn sample_exactly_at_limit_passes() {
        let samples = vec![s(LIMIT_SECONDS)];
        assert!(judge(&samples).is_pass());
    }

    #[test]
    fn all_samples_over_limit_fails() {
        // 全サンプルが超過 = 恒常的なリグレッション。min-of-N でも検出できる
        // ことを固定する（受け入れ基準「リグレッション検出力の維持」）。
        let samples = vec![s(5.1), s(6.0), s(5.5)];
        assert!(!judge(&samples).is_pass());
    }

    #[test]
    fn single_flaky_sample_over_limit_still_passes() {
        // イシュー #294 の実測値（PR #291: 5.494s の跳ね）を再現する
        // フレーク耐性の回帰テスト。1 サンプルのみが CPU 競合で跳ねても、
        // 残りが十分速ければ min 基準で PASS すること。
        let samples = vec![s(0.5), s(5.494), s(0.6)];
        assert!(judge(&samples).is_pass());
    }

    #[test]
    fn min_exactly_at_limit_passes() {
        let samples = vec![s(LIMIT_SECONDS), s(9.9), s(8.0)];
        assert!(judge(&samples).is_pass());
    }

    #[test]
    fn empty_samples_fail_closed() {
        let result = judge(&[]);
        assert!(!result.is_pass());
        match result {
            CheckResult::Fail {
                min_seconds,
                max_seconds,
                ..
            } => {
                assert_eq!(min_seconds, 0.0);
                assert_eq!(max_seconds, 0.0);
            }
            CheckResult::Pass { .. } => panic!("empty samples must not pass"),
        }
    }

    #[test]
    fn format_summary_line_matches_contract_on_pass() {
        let samples = vec![s(0.571), s(0.597), s(0.58)];
        let report = format_summary_line(&judge(&samples));
        assert_eq!(
            report,
            "rebuild-latency: samples=3 s1=0.571 s2=0.597 s3=0.580 min_s=0.571 max_s=0.597 limit_s=5.0 result=PASS"
        );
    }

    #[test]
    fn format_summary_line_matches_contract_on_fail() {
        let samples = vec![s(6.0), s(6.2)];
        let report = format_summary_line(&judge(&samples));
        assert_eq!(
            report,
            "rebuild-latency: samples=2 s1=6.000 s2=6.200 min_s=6.000 max_s=6.200 limit_s=5.0 result=FAIL"
        );
    }
}

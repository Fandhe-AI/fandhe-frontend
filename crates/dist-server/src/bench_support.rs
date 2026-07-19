//! `benches/rebuild_latency.rs`（TASK-10.4a、イシュー #119、REQ-10）の判定・整形ロジック。
//!
//! REQ-10 の受け入れ基準「本番ビルドのアセット変更反映（差分ビルド）が
//! 5 秒以内」（`docs/spec/04-requirements.md`、PoC-4 実測 0.571〜0.597 秒）を
//! 継続計測するベンチマークの純粋関数部分をここへ切り出す。
//!
//! ベンチ本体（`benches/rebuild_latency.rs`）は `[[bench]] test = false` の
//! ため `cargo test` では実行されない。しきい値判定・サマリ整形はここへ
//! 分離し、`cargo test -p fandhe-frontend-dist-server` から直接ユニットテストで固定する
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
//! # 判定統計量: median-of-N（イシュー #294）
//!
//! 共有 self-hosted runner（6 並列）上では他ジョブとの CPU 競合により、
//! N=3 サンプルのうち 1 つだけが環境ノイズで跳ねることがある
//! （実測: PR #291 で 5.494s → rerun で 2.x〜4.x 秒台に収束）。従来は
//! 最大値（max）を `LIMIT_SECONDS` と比較していたため、この種の間欠的な
//! ノイズだけで FAIL していた。
//!
//! 本モジュールは判定基準を **中央値（median）** に変更する（レビュー
//! 指摘: 最小値（min）を採用すると N=3 中 1 サンプルでもしきい値以内なら
//! 残り 2 サンプルが超過していても PASS してしまい、間欠的（多数が遅い）
//! リグレッションの検出力が失われるため）。median は 1 サンプルのみの
//! 環境ノイズ（少数派）を吸収しつつ、N 中の過半数が恒常的に遅い場合は
//! 引き続き FAIL として検出する。トレードオフとして、**少数派**
//! （N=3 なら 1 サンプルのみ）が間欠的に遅くなる製品リグレッションは
//! median でも検出できない（min ほど寛容ではないが、ゼロではない）。
//! サマリには引き続き全サンプル値と `min_s`・`max_s` を出力し、Step
//! Summary 上で目視追跡できるようにする（`.claude/rules/security.md`
//! 「セキュリティ設定ミス」観点: しきい値緩和ではなく判定統計量の変更で
//! あり、`LIMIT_SECONDS` 自体・fail-closed 方針は変更しない）。
//!
//! ## 偶数 N の中央値定義
//!
//! サンプル数が偶数の場合は「上側中央値」（昇順ソート後、`len / 2`
//! 番目＝ 0 始まりで中央より高い側の値）を採用する。しきい値超過方向を
//! 判定に使う fail-closed の考え方（`judge` 関数の空スライス処理と同じ
//! 方針）と整合させるため、平均を取らず単純に高い方へ倒す。

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
/// 判定は中央値（`median_seconds`）基準（イシュー #294、median-of-N 採用の
/// 根拠はモジュールドキュメント参照）。`min_seconds` / `max_seconds` は
/// 判定には使わないが、Step Summary 上で分布を目視追跡できるよう観測性の
/// ために保持する。
#[derive(Debug, Clone, PartialEq)]
pub enum CheckResult {
    /// 全サンプルの中央値がしきい値以内。
    Pass {
        samples: Vec<Sample>,
        median_seconds: f64,
        min_seconds: f64,
        max_seconds: f64,
    },
    /// 中央値がしきい値を超過（＝過半数のサンプルが超過、恒常的なリグレッション）。
    Fail {
        samples: Vec<Sample>,
        median_seconds: f64,
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
/// （median=0.0 / min=0.0 / max=0.0）として扱う（fail-closed。計測が
/// 1 件も取れなかった場合に PASS 側へ倒れないようにするため）。
///
/// 判定基準は中央値（median-of-N、イシュー #294）。共有 runner の CPU 競合に
/// よる少数派（N=3 なら 1 サンプル）の跳ねは吸収しつつ、過半数のサンプルが
/// 超過する恒常的なリグレッションは引き続き検出する（モジュールドキュメント
/// 参照）。偶数 N は上側中央値（`sorted[len / 2]`）を採る。
pub fn judge(samples: &[Sample]) -> CheckResult {
    let min_seconds = samples.iter().map(|s| s.seconds).fold(f64::MAX, f64::min);
    let max_seconds = samples.iter().map(|s| s.seconds).fold(f64::MIN, f64::max);
    let mut sorted_seconds: Vec<f64> = samples.iter().map(|s| s.seconds).collect();
    sorted_seconds.sort_by(|a, b| a.partial_cmp(b).expect("sample seconds must be finite"));
    let median_seconds = sorted_seconds.get(sorted_seconds.len() / 2).copied();

    let (median_seconds, min_seconds, max_seconds) = match median_seconds {
        Some(median_seconds) => (median_seconds, min_seconds, max_seconds),
        None => (0.0, 0.0, 0.0),
    };
    let samples_vec = samples.to_vec();

    if !samples.is_empty() && median_seconds <= LIMIT_SECONDS {
        CheckResult::Pass {
            samples: samples_vec,
            median_seconds,
            min_seconds,
            max_seconds,
        }
    } else {
        CheckResult::Fail {
            samples: samples_vec,
            median_seconds,
            min_seconds,
            max_seconds,
        }
    }
}

/// CI ログから機械抽出可能な 1 行サマリを整形する。
///
/// 書式（`rebuild-latency: samples=<n> s1=<x> s2=<x> ... median_s=<x>
/// min_s=<x> max_s=<x> limit_s=<x> result=<PASS|FAIL>`）は
/// `.github/workflows/ci.yml` の `rebuild-latency` ジョブが
/// `grep '^rebuild-latency:'` で抽出する契約であり、`#[cfg(test)]` 下の
/// ユニットテストで固定する。安易に変更しない。
///
/// `median_s`（イシュー #294 で判定基準に採用）に加え、`min_s` / `max_s`
/// （観測性のため保持。分布の目視追跡用）も出力する。
pub fn format_summary_line(result: &CheckResult) -> String {
    let (samples, median_seconds, min_seconds, max_seconds, verdict) = match result {
        CheckResult::Pass {
            samples,
            median_seconds,
            min_seconds,
            max_seconds,
        } => (samples, *median_seconds, *min_seconds, *max_seconds, "PASS"),
        CheckResult::Fail {
            samples,
            median_seconds,
            min_seconds,
            max_seconds,
        } => (samples, *median_seconds, *min_seconds, *max_seconds, "FAIL"),
    };

    let mut line = format!("rebuild-latency: samples={}", samples.len());
    for (i, sample) in samples.iter().enumerate() {
        line.push_str(&format!(" s{}={:.3}", i + 1, sample.seconds));
    }
    line.push_str(&format!(
        " median_s={median_seconds:.3} min_s={min_seconds:.3} max_s={max_seconds:.3} limit_s={LIMIT_SECONDS:.1} result={verdict}"
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
        // 全サンプルが超過 = 恒常的なリグレッション。median-of-N でも検出
        // できることを固定する（受け入れ基準「リグレッション検出力の維持」）。
        let samples = vec![s(5.1), s(6.0), s(5.5)];
        assert!(!judge(&samples).is_pass());
    }

    #[test]
    fn single_flaky_sample_over_limit_still_passes() {
        // イシュー #294 の実測値（PR #291: 5.494s の跳ね）を再現する
        // フレーク耐性の回帰テスト。N=3 中 1 サンプル（少数派）のみが
        // CPU 競合で跳ねても、残り 2 サンプルが十分速ければ median 基準で
        // PASS すること。
        let samples = vec![s(0.5), s(5.494), s(0.6)];
        assert!(judge(&samples).is_pass());
    }

    #[test]
    fn majority_over_limit_with_one_fast_sample_fails() {
        // レビュー指摘（イシュー #294）: min-of-N では「N=3 中 2 つが超過・
        // 1 つだけ高速」でも PASS してしまい、間欠的（多数決で恒常的寄り）な
        // リグレッションを見逃す。median-of-N はこのケースを FAIL として
        // 検出できることを固定する。
        let samples = vec![s(6.0), s(6.0), s(4.9)];
        assert!(!judge(&samples).is_pass());
    }

    #[test]
    fn median_over_limit_with_min_within_limit_fails() {
        // 旧 min-of-N 時代は `[5.0, 9.9, 8.0]`（min=5.0 のみでしきい値以内）が
        // PASS していた。過半数（8.0・9.9）が超過している以上、median-of-N
        // では FAIL すべきことを固定する（min-of-N が許容していた検出力低下
        // が解消されたことの回帰テスト）。
        let samples = vec![s(LIMIT_SECONDS), s(9.9), s(8.0)];
        assert!(!judge(&samples).is_pass());
    }

    #[test]
    fn empty_samples_fail_closed() {
        let result = judge(&[]);
        assert!(!result.is_pass());
        match result {
            CheckResult::Fail {
                median_seconds,
                min_seconds,
                max_seconds,
                ..
            } => {
                assert_eq!(median_seconds, 0.0);
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
            "rebuild-latency: samples=3 s1=0.571 s2=0.597 s3=0.580 median_s=0.580 min_s=0.571 max_s=0.597 limit_s=5.0 result=PASS"
        );
    }

    #[test]
    fn format_summary_line_matches_contract_on_fail() {
        let samples = vec![s(6.0), s(6.2)];
        let report = format_summary_line(&judge(&samples));
        assert_eq!(
            report,
            "rebuild-latency: samples=2 s1=6.000 s2=6.200 median_s=6.200 min_s=6.000 max_s=6.200 limit_s=5.0 result=FAIL"
        );
    }

    #[test]
    fn even_sample_count_uses_upper_median() {
        // 偶数 N（この場合 2）の中央値定義（モジュールドキュメント参照:
        // 上側中央値 `sorted[len / 2]`）を固定する。`[4.0, 6.0]` は下側
        // （4.0）ならしきい値以内で PASS してしまうが、上側（6.0）を採る
        // ため FAIL することを確認する。
        let samples = vec![s(4.0), s(6.0)];
        assert!(!judge(&samples).is_pass());
    }
}

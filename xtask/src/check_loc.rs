//! REQ-8（View Transitions API のネイティブ活用、`docs/spec/04-requirements.md`）の
//! 受け入れ基準「同一文書内更新向けラッパー関数の実効行数（コメント・空行を除く）が
//! 10 行以内であること」を機械的に強制するモジュール（TASK-8.2b、イシュー #62）。
//!
//! ルーブリックの定義元は PoC-3（`docs/spec/03-poc/rendering-web-standards/README.md`）
//! の「薄い = 実効行数 0〜10 行（コメント・空行を除く）」。対になる TASK-8.2a
//! （イシュー #61）が実装する `static/view-transitions.js` の `withViewTransition`
//! ラッパーが対象で、本モジュールはそのしきい値超過を CI で検知する。
//!
//! `check_deps.rs`（TASK-3.1）と同じ運用原則を踏襲する: しきい値・対象ファイルは
//! [`MAX_EFFECTIVE_LOC`] / [`LOC_CHECK_TARGETS`] のコード定数のみが正であり、
//! CLI 引数・環境変数による緩和経路は設けない（`xtask/src/main.rs` の
//! `run_check_loc` は引数を一切取らない契約）。対象ファイルの不在・読み取り失敗も
//! しきい値超過と同様に fail-closed（終了コード 1）として扱う。

use std::fmt;
use std::fs;

/// LOC チェックの対象ファイル（workspace ルートからの相対パス）。
///
/// TASK-8.2a（イシュー #61）が実装する `static/view-transitions.js` の
/// `withViewTransition` ラッパーのみを対象とする。将来的に他のグルー JS
/// （`hydrate.js` 等）へ対象を広げる場合は、勝手に追加せず別イシューとして
/// 提案すること（`out-of-scope-tracking.md`）。
pub const LOC_CHECK_TARGETS: &[&str] = &["static/view-transitions.js"];

/// REQ-8 受け入れ基準が定める実効 LOC の上限（コメント・空行を除く）。
pub const MAX_EFFECTIVE_LOC: usize = 10;

/// 1 ファイルの実効 LOC 計測結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocMeasurement {
    /// 計測対象ファイルパス（[`LOC_CHECK_TARGETS`] の要素そのもの）。
    pub file: String,
    /// コメント・空行を除いた実効行数。
    pub effective_loc: usize,
}

/// [`MAX_EFFECTIVE_LOC`] に照らした判定結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// 実効 LOC がしきい値以内。
    Pass(LocMeasurement),
    /// 実効 LOC がしきい値を超過。
    Fail(LocMeasurement),
}

impl CheckResult {
    /// CI（`.github/workflows/ci.yml` の `loc-check` ジョブ）が終了コードを
    /// 決定する際に参照する契約: `Pass` のみ成功、それ以外は失敗として扱う。
    pub fn is_pass(&self) -> bool {
        matches!(self, CheckResult::Pass(_))
    }
}

/// 実測値 `measurement` を [`MAX_EFFECTIVE_LOC`] に照らして判定する純粋関数。
///
/// I/O を一切行わないため単体テストで境界値（ちょうど 10 行 / 11 行）を
/// 直接検証できる。
pub fn judge(measurement: LocMeasurement) -> CheckResult {
    if measurement.effective_loc <= MAX_EFFECTIVE_LOC {
        CheckResult::Pass(measurement)
    } else {
        CheckResult::Fail(measurement)
    }
}

/// CI ログから機械抽出可能な 1 行サマリを整形する。
///
/// 書式（`loc-check: file=<path> effective_loc=<n>/<limit> result=<PASS|FAIL>`）は
/// `.github/workflows/ci.yml` の `loc-check` ジョブが `grep '^loc-check:'` で
/// 抽出する契約であり、`xtask/tests/cli_check_loc.rs` で固定する。安易に変更しない。
pub fn format_loc_report(result: &CheckResult) -> String {
    let (measurement, verdict) = match result {
        CheckResult::Pass(m) => (m, "PASS"),
        CheckResult::Fail(m) => (m, "FAIL"),
    };
    format!(
        "loc-check: file={} effective_loc={}/{} result={verdict}\n",
        measurement.file, measurement.effective_loc, MAX_EFFECTIVE_LOC
    )
}

/// ファイル I/O 由来のエラー。fail-closed の観点から、対象ファイル不在・読み取り
/// 失敗のいずれも呼び出し元（`run_check_loc`）で終了コード 1 に落とし込む。
#[derive(Debug)]
pub struct CheckLocError {
    file: String,
    source: std::io::Error,
}

impl fmt::Display for CheckLocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to read `{}`: {}", self.file, self.source)
    }
}

impl std::error::Error for CheckLocError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// `path`（workspace ルートからの相対パス、または呼び出し側 CWD からの相対パス）を
/// 読み込み、実効 LOC を計測する。
///
/// `xtask/tests/cli_check_loc.rs` はテスト専用ディレクトリに `current_dir` を
/// 設定して本関数（経由の CLI）を検証するため、パスは常に相対パスとして扱う。
pub fn measure_file(path: &str) -> Result<LocMeasurement, CheckLocError> {
    let source = fs::read_to_string(path).map_err(|source| CheckLocError {
        file: path.to_string(),
        source,
    })?;
    Ok(LocMeasurement {
        file: path.to_string(),
        effective_loc: count_effective_loc(&source),
    })
}

/// `source` のうち、空行・コメント行（`//` 行コメント、複数行にまたがる
/// `/* */` ブロックコメントを含む）を除いた実効行数を数える。
///
/// 完全な JS 字句解析は行わない 1 パスの簡易状態機械であり、
/// 文字列リテラル内に `//` や `/*` が現れるケースを正しく除外できない。
///
/// コメント開始位置より手前に非空白の実コードがある行は、その位置の判定が
/// 文字列リテラル内であっても必ずカウントされる（例: `const url = "http://x";`）。
/// これは「実効 LOC を過大に計上する」方向（ゲートを弱めない方向）に倒れる。
///
/// 一方、文字列リテラル内に `/*` が現れ、同一行に対応する `*/` が無い場合
/// （例: `const x = "/*";`）は、状態機械が誤ってブロックコメント継続中と
/// 認識し、後続行が実コードでも `*/` 相当の文字列に出会うまで未カウントに
/// なり得る（過小計上）。この経路は「実効 LOC を過大評価しない」方向の
/// 誤判定であり、完全な字句解析を行わない設計上のトレードオフとして許容する
/// （対象はレビュー済みの薄いラッパー 1 関数のみであり、実運用でこの構文は
/// 想定していない）。
fn count_effective_loc(source: &str) -> usize {
    let mut count = 0;
    // ブロックコメントは行をまたぐため、直前行から継続中かどうかを状態として持つ。
    let mut in_block_comment = false;

    for line in source.lines() {
        let mut rest = line;
        let mut has_code = false;

        loop {
            if in_block_comment {
                match rest.find("*/") {
                    Some(idx) => {
                        rest = &rest[idx + 2..];
                        in_block_comment = false;
                        // ブロックコメント終端後に同一行内のコード・別コメントが
                        // 続く可能性があるため、ループを継続して残りを再評価する。
                    }
                    None => break, // 行全体がブロックコメント内。この行に実コードなし。
                }
            } else {
                let line_comment = rest.find("//");
                let block_comment = rest.find("/*");
                let next = match (line_comment, block_comment) {
                    (None, None) => None,
                    (Some(l), None) => Some((l, false)),
                    (None, Some(b)) => Some((b, true)),
                    (Some(l), Some(b)) => Some(if l < b { (l, false) } else { (b, true) }),
                };
                match next {
                    None => {
                        if !rest.trim().is_empty() {
                            has_code = true;
                        }
                        break;
                    }
                    Some((pos, is_block)) => {
                        if !rest[..pos].trim().is_empty() {
                            has_code = true;
                        }
                        if is_block {
                            rest = &rest[pos + 2..];
                            in_block_comment = true;
                        } else {
                            break; // `//` は行末までコメント。
                        }
                    }
                }
            }

            if rest.is_empty() {
                break;
            }
        }

        if has_code {
            count += 1;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_lines_only_are_not_counted() {
        assert_eq!(count_effective_loc("\n\n   \n\t\n"), 0);
    }

    #[test]
    fn line_comments_only_are_not_counted() {
        let src = "// header comment\n// another comment\n";
        assert_eq!(count_effective_loc(src), 0);
    }

    #[test]
    fn multiline_block_comment_is_not_counted() {
        let src = "/*\n multi\n line comment\n*/\nconst x = 1;\n";
        assert_eq!(count_effective_loc(src), 1);
    }

    #[test]
    fn code_after_block_comment_close_on_same_line_is_counted() {
        let src = "/* header */ const x = 1;\n";
        assert_eq!(count_effective_loc(src), 1);
    }

    #[test]
    fn code_before_trailing_line_comment_is_counted() {
        let src = "const x = 1; // trailing comment\n";
        assert_eq!(count_effective_loc(src), 1);
    }

    #[test]
    fn double_slash_inside_string_literal_is_conservatively_counted() {
        // 文字列内の `//`（URL 等）は素朴な走査ではコメント開始と誤認されるが、
        // その手前に実コードがあるため過大計上側（カウントする側）に倒れる。
        let src = "const url = \"http://example.com\";\n";
        assert_eq!(count_effective_loc(src), 1);
    }

    #[test]
    fn slash_star_inside_string_literal_is_a_known_undercount_caveat() {
        // 文字列内の `/*`（対応する `*/` が同一行に無い場合）は状態機械が
        // 誤ってブロックコメント継続中と認識し、後続の実コード行が未カウントに
        // なり得る（`count_effective_loc` の rustdoc 参照）。完全な字句解析を
        // 行わない設計上のトレードオフとして許容し、実際の挙動を固定するために
        // 明示的にテストする（対象はレビュー済みの薄いラッパーのみで、実運用で
        // この構文は想定していない）。
        let src = "const x = \"/*\";\nconst y = 2;\n";
        assert_eq!(count_effective_loc(src), 1);
    }

    #[test]
    fn exactly_ten_lines_passes() {
        let m = LocMeasurement {
            file: "static/view-transitions.js".to_string(),
            effective_loc: 10,
        };
        assert!(judge(m).is_pass());
    }

    #[test]
    fn eleven_lines_fails() {
        let m = LocMeasurement {
            file: "static/view-transitions.js".to_string(),
            effective_loc: 11,
        };
        assert!(!judge(m).is_pass());
    }

    #[test]
    fn format_loc_report_matches_summary_contract() {
        let m = LocMeasurement {
            file: "static/view-transitions.js".to_string(),
            effective_loc: 6,
        };
        let report = format_loc_report(&judge(m));
        assert_eq!(
            report,
            "loc-check: file=static/view-transitions.js effective_loc=6/10 result=PASS\n"
        );
    }

    #[test]
    fn measure_file_reports_io_error_for_missing_file() {
        let err = measure_file("static/does-not-exist.js").unwrap_err();
        assert!(err.to_string().contains("static/does-not-exist.js"));
    }
}

//! `.github/workflows/ci.yml` の `ci-complete` 集約ジョブ（ruleset
//! `main-protection` の必須チェック集約先、`.claude/rules/ci.md` §
//! 「`ci-complete` 集約ジョブと ruleset 必須チェック」）が持つ 3 つの
//! 不変条件を fail-closed に機械検知する契約テスト（イシュー #2324）。
//!
//! ## 背景
//!
//! `ci-complete` は ci.yml の全ジョブを `needs:` に列挙し `if: always()`
//! で結果を集約する。ジョブを追加して `needs:` へ入れ忘れると、その
//! ジョブの失敗が必須チェックへ反映されないサイレントな fail-open になる。
//! これまでこの整合性は ci.yml のコメント（「レビューで必ず確認する」）
//! に頼る人手レビューのみで担保されていた。本テストはこれを
//! `cargo test -p xtask` の時点で検知する。
//!
//! ## 契約の 3 点
//!
//! 1. **`needs:` の網羅性**: `jobs:` 直下の全トップレベルジョブ（自身を
//!    除く）と `ci-complete` の `needs:` 列挙が集合として完全一致する
//!    （追加漏れ・改名の取り残し・重複のいずれも違反）。
//! 2. **`if: always()`**: `ci-complete` 自身がちょうど 1 個の
//!    `if: always()`（リテラル一致。`${{ always() }}` 等の表記揺れは
//!    意図的に非受理）を持つ。
//! 3. **`skipped` 許容リストの整合**: 集約ステップの jq 式が許容する
//!    `skipped` 結果は、本ファイル内の定数 [`SKIPPED_ALLOWLIST`] と
//!    1 対 1 で一致し、許容対象ジョブは実際にジョブレベル `if:` を
//!    持ち、逆に `ci-complete` 以外でジョブレベル `if:`（値が
//!    `always()` でない）を持つジョブは全て許容リストに含まれる。
//!
//! `SKIPPED_ALLOWLIST` は ci.yml から自動導出せず、本ファイル内の定数と
//! して固定する。ci.yml 側だけで `if:` と jq 式を同時に書き換えても
//! この定数が追随しない限り FAIL するようにするための、意図的な
//! 第 3 の摩擦点である（`.claude/rules/ci.md` の「許容リストへの明示
//! 追加が必要」という運用注記の機械化）。
//!
//! ## 反転判定（fail-closed）の方針
//!
//! `workflow_runner_policy.rs` / `workflow_shared_target_contract.rs` と
//! 同じ流儀を踏襲する: 許容する表記を列挙してそれ以外を通す形ではなく、
//! 「唯一の正規形」に一致しない・認識できない表記はすべて違反として
//! 報告する。クォート付きキー・flow sequence（`needs: [a, b]`）・
//! `${{ }}` 式・アンカー等はいずれも未対応の表記として違反側へ倒れる。
//!
//! ## 外部 YAML パーサ不採用
//!
//! 行ベースの文字列走査に留め、外部クレートへの依存は追加しない
//! （REQ-3・xtask 外部依存ゼロ方針）。
//!
//! ## スコープ外
//!
//! 本契約は `.github/workflows/ci.yml` の `ci-complete` ジョブのみを
//! 対象とする。他ワークフロー（`docs-site.yml` 等）への同種契約の拡張は
//! 別途判断する（`.claude/rules/out-of-scope-tracking.md` 参照）。

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// workspace ルート（`xtask/` の親ディレクトリ）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/xtask/ から 2 段上でワークスペースルートに到達する（イシュー #436）")
        .to_path_buf()
}

fn ci_workflow_path() -> PathBuf {
    workspace_root().join(".github/workflows/ci.yml")
}

fn read_ci_workflow() -> String {
    std::fs::read_to_string(ci_workflow_path())
        .unwrap_or_else(|e| panic!("{:?} の読み込みに失敗した: {e}", ci_workflow_path()))
}

/// 1 行からコメント部分を切り落とす。`workflow_runner_policy.rs` /
/// `workflow_shared_target_contract.rs` と同一ロジック（意図的な複製。
/// 3 ファイル共有のヘルパモジュールは持たない自己完結スタイルを踏襲）。
fn strip_comment(line: &str) -> String {
    let trimmed_start = line.trim_start();
    if trimmed_start.starts_with('#') {
        return String::new();
    }

    let mut result = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut prev_char: Option<char> = None;

    for ch in line.chars() {
        if ch == '\'' && !in_double {
            in_single = !in_single;
            result.push(ch);
            prev_char = Some(ch);
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            result.push(ch);
            prev_char = Some(ch);
            continue;
        }
        if ch == '#' && !in_single && !in_double {
            let boundary_ok = match prev_char {
                None => true,
                Some(c) => c.is_whitespace(),
            };
            if boundary_ok {
                break;
            }
        }
        result.push(ch);
        prev_char = Some(ch);
    }

    result
}

/// `ci-complete` の集約ステップが `skipped` を許容してよいジョブ名の
/// 唯一の正本。ci.yml から自動導出しない（意図は本ファイル冒頭の doc
/// コメント参照）。
const SKIPPED_ALLOWLIST: &[&str] = &["version-bump-guard"];

/// `jobs:` 直下（インデント 2）のジョブ名行かどうかを判定する。
///
/// 唯一の正規形 `  <name>:`（行頭ちょうど 2 個の半角スペース、名前は
/// 英数字・`_`・`-` のみ、末尾は `:` のみ）にのみ一致する。クォート付き
/// キー・アンカー・flow mapping・末尾の余分な値はすべて `None`（＝反転
/// 判定により違反側）。
fn job_name_at_indent2(stripped_line: &str) -> Option<String> {
    let trimmed_end = stripped_line.trim_end();
    if !trimmed_end.starts_with("  ") {
        return None;
    }
    // ちょうどインデント 2（3 文字目が空白ならインデント 3 以上）。
    if trimmed_end.as_bytes().get(2) == Some(&b' ') {
        return None;
    }
    let rest = &trimmed_end[2..];
    let name = rest.strip_suffix(':')?;
    if name.is_empty() {
        return None;
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }
    Some(name.to_string())
}

/// インデント 4 の `<key>: <value>` 行を `(key, value)` として抽出する。
/// `needs:` / `if:` の判定にのみ使う（他の任意のジョブ直下キーも構文上
/// 拾えるが、本テストは `needs`/`if` 以外のキーには関心を持たない）。
fn key_at_indent4(stripped_line: &str) -> Option<(String, String)> {
    let trimmed_end = stripped_line.trim_end();
    if !trimmed_end.starts_with("    ") {
        return None;
    }
    if trimmed_end.as_bytes().get(4) == Some(&b' ') {
        return None;
    }
    let rest = &trimmed_end[4..];
    let colon_pos = rest.find(':')?;
    let key = &rest[..colon_pos];
    if key.is_empty()
        || !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }
    let value = rest[colon_pos + 1..].trim().to_string();
    Some((key.to_string(), value))
}

/// インデント 4 の `<key>: <value>` 行から、キー部分の文字種妥当性を検証
/// せずに生のキー文字列（コロン直前まで）だけを抽出する。`key_at_indent4`
/// はクォート付きキー等の未対応表記を `None` として握りつぶすため、
/// 「`if` に正規化できるが唯一の正規形と一致しない」表記（イシュー #2324
/// の P1 是正、下記 [`classify_if_like_key`] 参照）を検知する下請けとして
/// 別に用意する。
fn raw_key_at_indent4(stripped_line: &str) -> Option<String> {
    let trimmed_end = stripped_line.trim_end();
    if !trimmed_end.starts_with("    ") {
        return None;
    }
    if trimmed_end.as_bytes().get(4) == Some(&b' ') {
        return None;
    }
    let rest = &trimmed_end[4..];
    let colon_pos = rest.find(':')?;
    Some(rest[..colon_pos].to_string())
}

/// 前後を同じ引用符（`"..."` または `'...'`）で囲まれている場合のみ、
/// その引用符を剥がす。囲まれていなければそのまま返す。
fn strip_matching_quotes(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return &s[1..s.len() - 1];
        }
    }
    s
}

/// ジョブ本文 1 件分（`block_start..block_end`）を走査し、ジョブレベル
/// `if:` の状態を判定する。
///
/// - `canonical`: 唯一の正規形 `    if: <value>`（クォート無しキー）で
///   見つかった最初の 1 件の `(行番号, value)`。
/// - `non_canonical`: キーを引用符除去すると `if` に一致するが、唯一の
///   正規形とは一致しない行（クォート付きキー `"if"`/`'if'` 等）の
///   `(行番号, 生テキスト)` 一覧。`key_at_indent4` はこれらを黙って
///   `None` にして読み飛ばしてしまう（イシュー #2324 の P1 指摘）ため、
///   反転判定により呼び出し側で違反として報告する材料として集める。
struct JobIfScan {
    canonical: Option<(usize, String)>,
    non_canonical: Vec<(usize, String)>,
}

fn scan_job_if(stripped: &[String], block_start: usize, block_end: usize) -> JobIfScan {
    let mut canonical = None;
    let mut non_canonical = Vec::new();
    for (i, line) in stripped
        .iter()
        .enumerate()
        .take(block_end)
        .skip(block_start)
    {
        if line.is_empty() {
            continue;
        }
        if let Some((key, value)) = key_at_indent4(line) {
            if key == "if" && canonical.is_none() {
                canonical = Some((i, value));
            }
            continue;
        }
        if let Some(raw_key) = raw_key_at_indent4(line) {
            if strip_matching_quotes(raw_key.trim()) == "if" {
                non_canonical.push((i, line.trim().to_string()));
            }
        }
    }
    JobIfScan {
        canonical,
        non_canonical,
    }
}

/// `needs:` 配下の block sequence 要素（インデント 6 の `- <name>`）の
/// 分類結果。
enum NeedsLine {
    /// 妥当な要素（ジョブ名として使える文字種のみ）。
    Item(String),
    /// インデント 6 の `- ` 行だが要素の中身が想定外の表記
    /// （クォート・`${{ }}` 式・アンカー等）。
    InvalidItem(String),
    /// block sequence 要素ではない行（列挙の終端）。
    Other,
}

fn classify_needs_line(stripped_line: &str) -> NeedsLine {
    let trimmed_end = stripped_line.trim_end();
    if !trimmed_end.starts_with("      ") {
        return NeedsLine::Other;
    }
    if trimmed_end.as_bytes().get(6) == Some(&b' ') {
        return NeedsLine::Other;
    }
    let rest = &trimmed_end[6..];
    let Some(name) = rest.strip_prefix("- ") else {
        return NeedsLine::Other;
    };
    if !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        NeedsLine::Item(name.to_string())
    } else {
        NeedsLine::InvalidItem(name.to_string())
    }
}

/// `jobs:` 直下 1 ジョブの走査結果。
struct Job {
    name: String,
    /// ジョブ名行自体の 0-indexed 行番号。
    name_line: usize,
    /// ジョブ本文（ジョブ名行の次行）の開始 0-indexed 行番号。
    block_start: usize,
    /// ジョブ本文の終端（次のトップレベルジョブ行、または `jobs:`
    /// セクション終端）の 0-indexed 行番号（exclusive）。
    block_end: usize,
}

/// ci.yml（または同型のフィクスチャ）の内容から `ci-complete` の
/// `needs:` 網羅性契約を検証する。違反があれば全件を収集して返す
/// （1 件見つけて即座に打ち切らない。是正が 1 パスで終わるようにする
/// ため）。
fn check_ci_complete_needs_contract(contents: &str) -> Result<(), Vec<String>> {
    let mut violations: Vec<String> = Vec::new();
    let stripped: Vec<String> = contents.lines().map(strip_comment).collect();
    let total = stripped.len();

    // --- `jobs:` セクションの切り出し ---
    let jobs_occurrences: Vec<usize> = stripped
        .iter()
        .enumerate()
        .filter(|(_, l)| l.as_str() == "jobs:")
        .map(|(i, _)| i)
        .collect();
    if jobs_occurrences.is_empty() {
        violations.push("トップレベルの `jobs:` キーが見つからない".to_string());
        return Err(violations);
    }
    if jobs_occurrences.len() > 1 {
        violations.push(format!(
            "トップレベルの `jobs:` キーが {} 回出現している（1 回のみ想定）",
            jobs_occurrences.len()
        ));
    }
    let jobs_line = jobs_occurrences[0];

    let mut section_end = total;
    for (i, raw) in stripped.iter().enumerate().skip(jobs_line + 1) {
        if raw.is_empty() {
            // 空行、またはコメントのみの行（`strip_comment` で消える）。
            continue;
        }
        if !raw.starts_with(' ') {
            section_end = i;
            break;
        }
    }

    // --- `jobs:` 直下（インデント 2）のトップレベルジョブ名の抽出 ---
    let mut jobs: Vec<Job> = Vec::new();
    let mut seen_names: HashSet<String> = HashSet::new();
    for (i, line) in stripped
        .iter()
        .enumerate()
        .take(section_end)
        .skip(jobs_line + 1)
    {
        if line.is_empty() {
            continue;
        }
        let leading = line.len() - line.trim_start_matches(' ').len();
        if leading != 2 {
            continue;
        }
        match job_name_at_indent2(line) {
            Some(name) => {
                if !seen_names.insert(name.clone()) {
                    violations.push(format!(
                        "ci.yml:{}: ジョブ名 `{name}` が `jobs:` 直下で重複している",
                        i + 1
                    ));
                }
                jobs.push(Job {
                    name,
                    name_line: i,
                    block_start: i + 1,
                    block_end: section_end,
                });
            }
            None => {
                violations.push(format!(
                    "ci.yml:{}: `jobs:` 直下（インデント 2）に想定外の表記がある。\
                     クォート付きキー・アンカー・flow mapping 等は未対応であり、\
                     反転判定により違反として扱う: {}",
                    i + 1,
                    line.trim()
                ));
            }
        }
    }

    // block_end を次のジョブ行の直前に確定する。
    for idx in 0..jobs.len() {
        let end = if idx + 1 < jobs.len() {
            jobs[idx + 1].name_line
        } else {
            section_end
        };
        jobs[idx].block_end = end;
    }

    if jobs.is_empty() {
        violations.push("`jobs:` 直下にトップレベルジョブが 1 件も見つからなかった".to_string());
        return Err(violations);
    }

    // --- `ci-complete` ブロックの特定 ---
    let Some(ci_complete_idx) = jobs.iter().position(|j| j.name == "ci-complete") else {
        violations.push("`ci-complete` ジョブが `jobs:` 直下に見つからない".to_string());
        return Err(violations);
    };
    let (block_start, block_end) = (
        jobs[ci_complete_idx].block_start,
        jobs[ci_complete_idx].block_end,
    );

    // --- `needs:` キーの特定と要素の抽出 ---
    let mut needs_key_lines: Vec<usize> = Vec::new();
    let mut if_key_lines: Vec<(usize, String)> = Vec::new();
    for (i, line) in stripped
        .iter()
        .enumerate()
        .take(block_end)
        .skip(block_start)
    {
        if line.is_empty() {
            continue;
        }
        if let Some((key, value)) = key_at_indent4(line) {
            match key.as_str() {
                "needs" => needs_key_lines.push(i),
                "if" => if_key_lines.push((i, value)),
                _ => {}
            }
        }
    }

    let mut needs_items: Vec<(usize, String)> = Vec::new();
    match needs_key_lines.len() {
        1 => {
            let needs_line = needs_key_lines[0];
            let (_, value) = key_at_indent4(&stripped[needs_line])
                .expect("needs_key_lines は key_at_indent4 が Some を返した行のみを含む");
            if !value.is_empty() {
                violations.push(format!(
                    "ci.yml:{}: `ci-complete` の `needs:` の値が空でない（block sequence 形\
                     以外は本テスト非対応であり、反転判定により違反として扱う）: {value}",
                    needs_line + 1
                ));
            }
            let mut i = needs_line + 1;
            while i < block_end {
                let line = &stripped[i];
                if line.is_empty() {
                    i += 1;
                    continue;
                }
                match classify_needs_line(line) {
                    NeedsLine::Item(name) => {
                        needs_items.push((i, name));
                        i += 1;
                    }
                    NeedsLine::InvalidItem(raw) => {
                        violations.push(format!(
                            "ci.yml:{}: `ci-complete` の `needs:` 要素が想定外の表記（クォート・\
                             `${{{{ }}}}` 式・アンカー等）であり、反転判定により違反として扱う: {raw}",
                            i + 1
                        ));
                        i += 1;
                    }
                    NeedsLine::Other => break,
                }
            }
        }
        0 => {
            violations.push("`ci-complete` に `needs:` キーが見つからない".to_string());
        }
        n => {
            violations.push(format!(
                "`ci-complete` に `needs:` キーが {n} 回出現している（1 回のみ想定）"
            ));
        }
    }

    // --- needs 要素の重複検知 ---
    let mut needs_name_counts: HashMap<&str, usize> = HashMap::new();
    for (_, name) in &needs_items {
        *needs_name_counts.entry(name.as_str()).or_insert(0) += 1;
    }
    for (name, count) in &needs_name_counts {
        if *count > 1 {
            violations.push(format!(
                "`ci-complete` の `needs:` にジョブ `{name}` が {count} 回重複して列挙されている"
            ));
        }
    }
    let needs_names: HashSet<&str> = needs_name_counts.keys().copied().collect();

    // --- 集合比較: 定義済みジョブ（ci-complete 自身を除く）⇔ needs ---
    let defined_names: HashSet<&str> = jobs
        .iter()
        .filter(|j| j.name != "ci-complete")
        .map(|j| j.name.as_str())
        .collect();

    let mut missing: Vec<&&str> = defined_names.difference(&needs_names).collect();
    missing.sort();
    for name in missing {
        violations.push(format!(
            "ジョブ `{name}` が `ci-complete` の `needs:` に含まれていない（追加漏れ。\
             `.github/workflows/ci.yml` の `ci-complete` の `needs:` へ追加すること）"
        ));
    }

    let mut extra: Vec<&&str> = needs_names.difference(&defined_names).collect();
    extra.sort();
    for name in extra {
        violations.push(format!(
            "`ci-complete` の `needs:` にジョブ `{name}` が列挙されているが、`jobs:` 直下には\
             存在しない（改名の取り残しの可能性）"
        ));
    }

    if needs_names.contains("ci-complete") {
        violations
            .push("`ci-complete` の `needs:` に `ci-complete` 自身が含まれている".to_string());
    }

    // --- `if: always()` 契約 ---
    match if_key_lines.len() {
        1 => {
            let (line_no, value) = &if_key_lines[0];
            if value != "always()" {
                violations.push(format!(
                    "ci.yml:{}: `ci-complete` の `if:` が `always()` と一致しない\
                     （`${{{{ always() }}}}` 等の表記揺れは意図的に非受理）: {value}",
                    line_no + 1
                ));
            }
        }
        0 => violations.push("`ci-complete` に `if:` キーが見つからない".to_string()),
        n => violations.push(format!(
            "`ci-complete` に `if:` キーが {n} 回出現している（1 回のみ想定）"
        )),
    }

    // --- skipped 許容リストの整合（ci-complete ブロック内テキスト走査） ---
    let block_text: String = stripped[block_start..block_end].join("\n");
    if !block_text.contains("toJSON(needs)") {
        violations.push(
            "`ci-complete` の集約ステップに `toJSON(needs)` が見つからない（`needs` の結果を\
             丸ごと検証する契約が失われている）"
                .to_string(),
        );
    }
    // `SKIPPED_ALLOWLIST` から jq の判定式全体（`map(select((...) | not))`
    // の `(...)` 内側）を機械的に組み立て、集約ステップのテキストと
    // **部分文字列の存在ではなく式全体の完全一致**で照合する（イシュー
    // #2324 の P1 是正）。部分文字列一致（旧実装）は `\"skipped\"` の
    // 出現回数や個別の判定式の存在だけを見るため、既存の許容式へ
    // `or true` 等の無関係な追加節を継ぎ足す改変（許容外ジョブの
    // 失敗・skip を丸ごと PASS 扱いにする fail-open 化）があっても、
    // 元の判定式が部分文字列として残っている限り検知できない。
    // `map(select((` と `) | not))` という唯一の正規形の境界で予測式を
    // 完全一致検証することで、境界内へのどんな追加節も差分として検知
    // する。
    let allowlist_clause = SKIPPED_ALLOWLIST
        .iter()
        .map(|name| format!("(.key == \"{name}\" and .value.result == \"skipped\")"))
        .collect::<Vec<_>>()
        .join(" or ");
    let expected_predicate = if allowlist_clause.is_empty() {
        ".value.result == \"success\"".to_string()
    } else {
        format!(".value.result == \"success\" or {allowlist_clause}")
    };
    const PREDICATE_PREFIX: &str = "map(select((";
    const PREDICATE_SUFFIX: &str = ") | not))";
    match block_text.find(PREDICATE_PREFIX) {
        None => violations.push(format!(
            "`ci-complete` の集約ステップに唯一の正規形 `{PREDICATE_PREFIX}...{PREDICATE_SUFFIX}` \
             が見つからない（反転判定により違反として扱う）"
        )),
        Some(prefix_idx) => {
            let after_prefix = &block_text[prefix_idx + PREDICATE_PREFIX.len()..];
            match after_prefix.find(PREDICATE_SUFFIX) {
                None => violations.push(format!(
                    "`ci-complete` の集約ステップの `{PREDICATE_PREFIX}` に対応する \
                     `{PREDICATE_SUFFIX}` が見つからない（反転判定により違反として扱う）"
                )),
                Some(suffix_idx) => {
                    let actual_predicate = &after_prefix[..suffix_idx];
                    if actual_predicate != expected_predicate {
                        violations.push(format!(
                            "`ci-complete` の集約ステップの skipped 許容式が `SKIPPED_ALLOWLIST` \
                             から構成した期待式と完全一致しない（部分文字列一致ではなく式全体の\
                             一致を要求する）。\n  期待: {expected_predicate}\n  実際: {actual_predicate}"
                        ));
                    }
                }
            }
        }
    }

    // --- 許容リストの各ジョブが実在し、ジョブレベル if: を持つこと ---
    for name in SKIPPED_ALLOWLIST {
        match jobs.iter().find(|j| j.name == *name) {
            None => violations.push(format!(
                "`SKIPPED_ALLOWLIST` のジョブ `{name}` が `jobs:` 直下に存在しない（stale な\
                 許容リストの可能性）"
            )),
            Some(job) => {
                let scan = scan_job_if(&stripped, job.block_start, job.block_end);
                // クォート付きキー等、`if` に正規化できるが唯一の正規形と
                // 一致しない表記はそれ自体を違反として報告する（黙って
                // 「if: が無い」と誤判定しない。イシュー #2324 の P1 是正）。
                for (i, raw) in &scan.non_canonical {
                    violations.push(format!(
                        "ci.yml:{}: `SKIPPED_ALLOWLIST` のジョブ `{name}` の `if:` 相当の\
                         キーが唯一の正規形（クォート無し `if:`）に一致しない表記になっている\
                         （反転判定により違反として扱う）: {raw}",
                        i + 1
                    ));
                }
                if scan.canonical.is_none() && scan.non_canonical.is_empty() {
                    violations.push(format!(
                        "`SKIPPED_ALLOWLIST` のジョブ `{name}` にジョブレベル `if:` が無い（\
                         条件付きジョブでなくなったのに許容リストへ残っている stale な状態の\
                         可能性）"
                    ));
                }
            }
        }
    }

    // --- ci-complete 以外で条件付き（if != always()）なジョブは許容リストに含まれること ---
    for job in &jobs {
        if job.name == "ci-complete" {
            continue;
        }
        let scan = scan_job_if(&stripped, job.block_start, job.block_end);
        // クォート付きキー等、認識できない `if:` 相当の表記はそれ自体を
        // 違反として報告する（`key_at_indent4` が黙って読み飛ばし、
        // 許容リスト外の条件付きジョブが検知漏れになる迂回を塞ぐ。
        // イシュー #2324 の P1 是正）。
        for (i, raw) in &scan.non_canonical {
            violations.push(format!(
                "ci.yml:{}: ジョブ `{}` の `if:` 相当のキーが唯一の正規形（クォート無し \
                 `if:`）に一致しない表記になっている（反転判定により違反として扱う）: {raw}",
                i + 1,
                job.name
            ));
        }
        if let Some((i, value)) = &scan.canonical {
            if value != "always()" && !SKIPPED_ALLOWLIST.contains(&job.name.as_str()) {
                violations.push(format!(
                    "ci.yml:{}: ジョブ `{}` がジョブレベル `if:`（値: {value}）を持つが、\
                     `SKIPPED_ALLOWLIST` に含まれていない（`ci-complete` が skip を fail\
                     として扱うため、条件付きジョブを増やす場合は本テストの \
                     `SKIPPED_ALLOWLIST` への追加が必要）",
                    i + 1,
                    job.name
                ));
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

#[test]
fn ci_complete_needs_covers_all_jobs_exactly() {
    let contents = read_ci_workflow();
    if let Err(violations) = check_ci_complete_needs_contract(&contents) {
        panic!(
            "`.github/workflows/ci.yml` の `ci-complete` needs 網羅性契約に違反している\
             （イシュー #2324）。ジョブを追加した場合は `ci-complete` の `needs:` へ\
             追加すること。違反一覧:\n{}",
            violations.join("\n")
        );
    }
}

/// `strip_comment` の暴走や section 検出の失敗で空集合同士が一致して\
/// vacuous PASS になっていないかを確認する（`workflow_runner_policy.rs::\
/// workflow_scan_is_not_vacuous` と同型）。
#[test]
fn ci_complete_scan_is_not_vacuous() {
    let contents = read_ci_workflow();
    let stripped: Vec<String> = contents.lines().map(strip_comment).collect();
    let job_count = stripped
        .iter()
        .filter(|l| job_name_at_indent2(l).is_some())
        .count();
    assert!(
        job_count >= 2,
        "`jobs:` 直下のトップレベルジョブが 2 件未満だった（走査が壊れている可能性）"
    );
    assert!(
        stripped.iter().any(|l| l.trim() == "ci-complete:"),
        "`ci-complete:` 行が走査対象に見つからない"
    );
    let needs_item_count = stripped
        .iter()
        .filter(|l| matches!(classify_needs_line(l), NeedsLine::Item(_)))
        .count();
    assert!(
        needs_item_count >= 1,
        "`needs:` の block sequence 要素が 1 件も見つからなかった（走査が壊れている可能性）"
    );
}

#[cfg(test)]
mod fixture_tests {
    use super::{check_ci_complete_needs_contract, SKIPPED_ALLOWLIST};

    /// 契約を満たす最小の PASS フィクスチャ。実 ci.yml と同じ構造
    /// （ジョブ 3 件 + `ci-complete`、うち 1 件が `if:` 付きの許容
    /// リスト対象）を最小化したもの。以降の FAIL テストはこれを 1 箇所
    /// だけ変異させて構成する。
    fn base_fixture() -> String {
        concat!(
            "name: CI\n",
            "on:\n",
            "  pull_request:\n",
            "  push:\n",
            "jobs:\n",
            "  job-a:\n",
            "    runs-on: ubuntu-latest\n",
            "    steps:\n",
            "      - run: \"true\"\n",
            "  job-b:\n",
            "    runs-on: ubuntu-latest\n",
            "    steps:\n",
            "      - run: \"true\"\n",
            "  version-bump-guard:\n",
            "    if: ${{ github.event_name == 'pull_request' }}\n",
            "    runs-on: ubuntu-latest\n",
            "    steps:\n",
            "      - run: \"true\"\n",
            "  ci-complete:\n",
            "    name: ci-complete\n",
            "    if: always()\n",
            "    runs-on: ubuntu-latest\n",
            "    needs:\n",
            "      - job-a\n",
            "      - job-b\n",
            "      - version-bump-guard\n",
            "    steps:\n",
            "      - name: check\n",
            "        env:\n",
            "          RESULTS: ${{ toJSON(needs) }}\n",
            "        run: |\n",
            "          echo \"${RESULTS}\" | jq -e 'to_entries | map(select((.value.result == \"success\" or (.key == \"version-bump-guard\" and .value.result == \"skipped\")) | not)) | length == 0' > /dev/null\n",
        )
        .to_string()
    }

    fn assert_violations_contain(contents: &str, needle: &str) {
        match check_ci_complete_needs_contract(contents) {
            Ok(()) => panic!("契約違反を検知できなかった（想定: `{needle}` を含む違反）"),
            Err(violations) => {
                let joined = violations.join("\n");
                assert!(
                    joined.contains(needle),
                    "違反メッセージに `{needle}` が含まれていない。実際の違反一覧:\n{joined}"
                );
            }
        }
    }

    #[test]
    fn allowlist_constant_matches_fixture() {
        // フィクスチャは SKIPPED_ALLOWLIST と同じ 1 件（version-bump-guard）
        // を前提に組んでいる。定数を変更した場合はこのテストが即座に
        // フィクスチャとの前提ズレを知らせる。
        assert_eq!(SKIPPED_ALLOWLIST, &["version-bump-guard"]);
    }

    #[test]
    fn pass_minimal_fixture_is_ok() {
        assert_eq!(check_ci_complete_needs_contract(&base_fixture()), Ok(()));
    }

    #[test]
    fn fail_needs_missing_one_job() {
        let contents = base_fixture().replace("      - job-b\n", "");
        assert_violations_contain(&contents, "job-b");
    }

    #[test]
    fn fail_new_job_not_in_needs() {
        let contents = base_fixture().replace(
            "  ci-complete:\n",
            "  job-c:\n    runs-on: ubuntu-latest\n    steps:\n      - run: \"true\"\n  ci-complete:\n",
        );
        assert_violations_contain(&contents, "job-c");
    }

    #[test]
    fn fail_needs_references_nonexistent_job() {
        let contents = base_fixture().replace(
            "      - version-bump-guard\n",
            "      - version-bump-guard\n      - job-x\n",
        );
        assert_violations_contain(&contents, "job-x");
    }

    #[test]
    fn fail_needs_duplicate_element() {
        let contents = base_fixture().replace(
            "      - job-a\n      - job-b\n",
            "      - job-a\n      - job-a\n      - job-b\n",
        );
        assert_violations_contain(&contents, "job-a");
    }

    #[test]
    fn fail_needs_flow_sequence_form() {
        let contents = base_fixture().replace(
            "    needs:\n      - job-a\n      - job-b\n      - version-bump-guard\n",
            "    needs: [job-a, job-b, version-bump-guard]\n",
        );
        // block sequence 形以外は非対応（値が空でない違反）として検知され、
        // 結果として 3 ジョブとも needs 追加漏れとしても報告される。
        assert_violations_contain(&contents, "job-a");
    }

    #[test]
    fn fail_needs_element_quoted_or_expression() {
        let contents = base_fixture().replace("      - job-a\n", "      - \"job-a\"\n");
        assert_violations_contain(&contents, "job-a");
    }

    #[test]
    fn fail_if_always_missing() {
        let contents = base_fixture().replace(
            "  ci-complete:\n    name: ci-complete\n    if: always()\n",
            "  ci-complete:\n    name: ci-complete\n",
        );
        assert_violations_contain(&contents, "if:");
    }

    #[test]
    fn fail_if_expression_wrapped() {
        let contents = base_fixture().replace("if: always()", "if: ${{ always() }}");
        assert_violations_contain(&contents, "always()");
    }

    #[test]
    fn fail_if_success_not_always() {
        let contents = base_fixture().replace("if: always()", "if: success()");
        assert_violations_contain(&contents, "always()");
    }

    #[test]
    fn fail_skipped_allowlist_extra_job() {
        let contents = base_fixture().replace(
            "(.key == \"version-bump-guard\" and .value.result == \"skipped\")",
            "(.key == \"version-bump-guard\" and .value.result == \"skipped\") or (.key == \"job-a\" and .value.result == \"skipped\")",
        );
        assert_violations_contain(&contents, "skipped");
    }

    #[test]
    fn fail_skipped_allowlist_stale_missing_if() {
        let contents =
            base_fixture().replace("    if: ${{ github.event_name == 'pull_request' }}\n", "");
        assert_violations_contain(&contents, "version-bump-guard");
    }

    /// イシュー #2324 の P1 是正 1 点目: 許容式へ無関係な `or true` 節を
    /// 継ぎ足す改変（許容外ジョブの失敗・skip を丸ごと PASS 扱いにする
    /// fail-open 化）は、元の判定式が部分文字列として残っているため
    /// 旧実装（`\"skipped\"` 出現回数・部分文字列存在チェック）では検知
    /// できなかった。式全体の完全一致検証で検知できることを固定する。
    #[test]
    fn fail_skipped_predicate_or_true_bypass_is_violation() {
        let contents = base_fixture().replace(
            "(.key == \"version-bump-guard\" and .value.result == \"skipped\")) | not))",
            "(.key == \"version-bump-guard\" and .value.result == \"skipped\") or true) | not))",
        );
        assert_violations_contain(&contents, "完全一致");
    }

    #[test]
    fn fail_conditional_job_not_in_allowlist() {
        let contents = base_fixture().replace(
            "  job-b:\n    runs-on: ubuntu-latest\n",
            "  job-b:\n    if: success()\n    runs-on: ubuntu-latest\n",
        );
        assert_violations_contain(&contents, "job-b");
    }

    /// イシュー #2324 の P1 是正 2 点目: 許容リスト外のジョブへクォート
    /// 付きキー `"if"` を追加すると、`key_at_indent4` が `None` を返して
    /// 黙って読み飛ばすため、旧実装では条件付きジョブとして検知されな
    /// かった（`SKIPPED_ALLOWLIST` への追加漏れがすり抜ける）。非正規形
    /// の `if` 相当キーそれ自体を違反として検知することを固定する。
    #[test]
    fn fail_conditional_job_quoted_if_key_is_violation() {
        let contents = base_fixture().replace(
            "  job-b:\n    runs-on: ubuntu-latest\n",
            "  job-b:\n    \"if\": success()\n    runs-on: ubuntu-latest\n",
        );
        assert_violations_contain(&contents, "job-b");
    }

    /// 同じクォート付きキーの迂回が `SKIPPED_ALLOWLIST` 側ジョブ（既に
    /// 許容リストに載っている想定）で起きた場合も、「if: が無い」との
    /// 誤判定ではなく非正規形の違反として検知することを固定する。
    #[test]
    fn fail_allowlist_job_quoted_if_key_is_violation() {
        let contents = base_fixture().replace(
            "    if: ${{ github.event_name == 'pull_request' }}\n",
            "    \"if\": ${{ github.event_name == 'pull_request' }}\n",
        );
        assert_violations_contain(&contents, "version-bump-guard");
    }

    #[test]
    fn fail_tojson_needs_missing() {
        let contents = base_fixture().replace("${{ toJSON(needs) }}", "nope");
        assert_violations_contain(&contents, "toJSON");
    }

    #[test]
    fn fail_quoted_job_key_is_violation() {
        let contents = base_fixture().replace("  job-a:\n", "  \"job-a\":\n");
        assert_violations_contain(&contents, "job-a");
    }

    #[test]
    fn pass_top_level_key_after_jobs_has_no_phantom_job() {
        // `jobs:` セクションの後ろに別トップレベルキー（`defaults:`）が
        // 続き、その配下にインデント 2 のキー（`run:`）があっても、
        // セクション境界の外なので幻ジョブとして拾われない。
        let contents = format!("{}defaults:\n  run:\n    shell: bash\n", base_fixture());
        assert_eq!(check_ci_complete_needs_contract(&contents), Ok(()));
    }

    #[test]
    fn pass_indent2_comment_line_not_treated_as_job() {
        // `jobs:` 直下のインデント 2 のコメント行（末尾が `:` でも）は
        // `strip_comment` で消えるため、ジョブ名の想定外表記として
        // 誤検知されない（ci.yml L1075 の実例に基づく境界テスト）。
        let contents = base_fixture().replace(
            "  job-a:\n",
            "  # イシュー #999（テスト用のコメント行）:\n  job-a:\n",
        );
        assert_eq!(check_ci_complete_needs_contract(&contents), Ok(()));
    }
}

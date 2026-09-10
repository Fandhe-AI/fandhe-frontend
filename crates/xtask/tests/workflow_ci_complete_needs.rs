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
//! 3. **`skipped` 許容リストの整合**: 集約ステップの `RESULTS` への
//!    `toJSON(needs)` 束縛・`echo "${RESULTS}" | jq -e '...' > /dev/null`
//!    という入力配線からプログラム全体（`-e` フラグ・select 述語・
//!    最終判定〔`| length == 0`〕を含む）まで、行全体の完全一致で
//!    検証する。jq 式が許容する `skipped` 結果は、本ファイル内の定数
//!    [`SKIPPED_ALLOWLIST`] と 1 対 1 で一致し、許容対象ジョブは実際に
//!    ジョブレベル `if:` を持ち、逆に `ci-complete` 以外でジョブレベル
//!    `if:`（値が `always()` でない）を持つジョブは全て許容リストに
//!    含まれる。加えて、`RESULTS` 束縛と jq 呼び出し行が同一ステップに
//!    属すること・そのステップの `run:` ブロックが jq 呼び出し行以外の
//!    コマンドを含まないこと・ステップレベル `if:` が不在または
//!    `always()` であること・`continue-on-error:` が不在であることを
//!    検証する（イシュー #2324 の PR #2335 codex-review 再指摘 1 点目。
//!    ブロック全体からの個別検索だけでは検知できない実行コンテキストの
//!    迂回を塞ぐ）。
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

/// インデント `indent` の `<key>: <value>` 行を `(key, value)` として
/// 抽出する（`key_at_indent4` の任意インデント版）。ジョブ本文の基準
/// インデントはジョブごとに異なり得る（`block_base_indent` 参照）ため、
/// `scan_job_if` はこちらを使う。
fn key_at_indent(stripped_line: &str, indent: usize) -> Option<(String, String)> {
    let trimmed_end = stripped_line.trim_end();
    let leading = trimmed_end.len() - trimmed_end.trim_start_matches(' ').len();
    if leading != indent {
        return None;
    }
    let rest = &trimmed_end[indent..];
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

/// インデント `indent` の `<key>: <value>` 行から、キー文字種の妥当性を
/// 検証せずに生のキー文字列だけを抽出する（`key_at_indent` の姉妹関数。
/// 用途は `key_at_indent` と同じで、`scan_job_if` が「`if` に正規化できる
/// が唯一の正規形と一致しない」表記を検知する下請けとして使う）。
fn raw_key_at_indent(stripped_line: &str, indent: usize) -> Option<String> {
    let trimmed_end = stripped_line.trim_end();
    let leading = trimmed_end.len() - trimmed_end.trim_start_matches(' ').len();
    if leading != indent {
        return None;
    }
    let rest = &trimmed_end[indent..];
    let colon_pos = rest.find(':')?;
    Some(rest[..colon_pos].to_string())
}

/// ジョブ本文（`block_start..block_end`）内の非空行の最小インデント幅を
/// 「そのジョブ本文直下キーの基準インデント」として返す。
///
/// 従来は `key_at_indent4` によりジョブ本文のキーは常にインデント 4
/// という前提で走査していたが、YAML 仕様上は
/// 兄弟キー同士のインデントが揃っていれば値は自由（親より深ければ何桁でも
/// 妥当）である。許容リスト外ジョブの本文全体をインデント 6（等）へ
/// 揃えたうえで `if:` をそこに書くと、インデント 4 固定の走査では
/// 「`if:` が存在しない」と読み飛ばしてしまい、条件付きジョブの検知が
/// すり抜ける（イシュー #2324 の P1 指摘）。ジョブ直下キー（`needs:`
/// 等）は他のどの行よりも浅いインデントで書かれる（それより深い行は
/// ネストされた値・block sequence 要素）ため、非空行の最小インデントが
/// 基準インデントに一致する。
fn block_base_indent(stripped: &[String], block_start: usize, block_end: usize) -> Option<usize> {
    stripped[block_start..block_end]
        .iter()
        // `strip_comment` はコメントのみの行・空行を `String::new()` に
        // するが、半角スペースのみの行（YAML 上は無意味な空行）は
        // そのまま残す。`!line.is_empty()` だけではこの空白のみの行を
        // 除外できず、最小インデントが 1 のような小さい値に引き下がって
        // しまい、通常インデント 4 にある `if:` を `scan_job_if` が
        // 見落とす（イシュー #2324 の PR #2335 codex-review 再指摘）。
        // `trim()` した結果が空の行は判定対象から除外する。
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start_matches(' ').len())
        .min()
}

/// ジョブ本文 1 件分（`block_start..block_end`）を走査し、ジョブレベル
/// `if:` の状態を判定する。
///
/// 基準インデントは `block_base_indent` でジョブごとに動的に決定する
/// （固定インデント 4 決め打ちにしない。イシュー #2324 の P1 是正 2 点目。
/// 上記 doc コメント参照）。ブロックが空（非空行が 1 行もない）場合のみ
/// 実 ci.yml の慣例であるインデント 4 へフォールバックする。
///
/// - `canonical`: 基準インデントちょうどの `if: <value>`（クォート無し
///   キー）で見つかった最初の 1 件の `(行番号, value)`。
/// - `non_canonical`: キーを引用符除去すると `if` に一致するが、
///   クォート無しキーとは一致しない行（クォート付きキー `"if"`/`'if'`
///   等）の `(行番号, 生テキスト)` 一覧。`key_at_indent` はこれらを
///   黙って `None` にして読み飛ばしてしまうため、反転判定により
///   呼び出し側で違反として報告する材料として集める。
struct JobIfScan {
    canonical: Option<(usize, String)>,
    non_canonical: Vec<(usize, String)>,
}

fn scan_job_if(stripped: &[String], block_start: usize, block_end: usize) -> JobIfScan {
    let indent = block_base_indent(stripped, block_start, block_end).unwrap_or(4);
    let KeyScan {
        canonical,
        non_canonical,
    } = scan_key_at_indent(stripped, block_start, block_end, indent, "if");
    JobIfScan {
        canonical,
        non_canonical,
    }
}

/// 汎用: 基準インデントちょうどの `<key>: <value>` を走査する結果。
/// `scan_job_if` の「唯一の正規形（クォート無しキー）」と「`key` に
/// 正規化できるが唯一の正規形とは一致しない表記（クォート付きキー等）」
/// を区別する枠組みを、任意のキー名・任意の（呼び出し側が確定させた）
/// インデントへ一般化したもの（イシュー #2324 の PR #2335 codex-review
/// 再指摘 1 点目: 集約ステップの実行コンテキスト検証で `if:` /
/// `continue-on-error:` の 2 キーを同じ枠組みで扱うために新設）。
struct KeyScan {
    /// 見つかった最初の 1 件の `(行番号, value)`。
    canonical: Option<(usize, String)>,
    /// キーを引用符除去すると対象キー名に一致するが、クォート無しキー
    /// とは一致しない行（クォート付きキー `"if"`/`'if'` 等）の
    /// `(行番号, 生テキスト)` 一覧。`key_at_indent` はこれらを黙って
    /// `None` にして読み飛ばしてしまうため、反転判定により呼び出し側で
    /// 違反として報告する材料として集める。
    non_canonical: Vec<(usize, String)>,
}

fn scan_key_at_indent(
    stripped: &[String],
    block_start: usize,
    block_end: usize,
    indent: usize,
    key_name: &str,
) -> KeyScan {
    let mut canonical = None;
    let mut non_canonical = Vec::new();
    for (i, line) in stripped
        .iter()
        .enumerate()
        .take(block_end)
        .skip(block_start)
    {
        if line.trim().is_empty() {
            continue;
        }
        if let Some((key, value)) = key_at_indent(line, indent) {
            if key == key_name && canonical.is_none() {
                canonical = Some((i, value));
            }
            continue;
        }
        if let Some(raw_key) = raw_key_at_indent(line, indent) {
            if strip_matching_quotes(raw_key.trim()) == key_name {
                non_canonical.push((i, line.trim().to_string()));
            }
        }
    }
    KeyScan {
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

/// `steps:` 配下のトップレベルステップ項目 1 件分の境界。
///
/// `start` はステップ項目マーカー行（`      - name: ...` 等）自体の
/// 0-indexed 行番号、`end` は次のステップ項目、または `steps:` を含む
/// 親ブロックの終端の 0-indexed 行番号（exclusive）。イシュー #2324 の
/// PR #2335 codex-review 再指摘 1 点目（集約処理の実行コンテキスト検証）
/// のため新設: `RESULTS` の束縛と jq 呼び出しをブロック全体から個別に
/// 検索するだけでは、`run` の先頭に無関係なコマンドを追加する・
/// 集約ステップへ `if:`/`continue-on-error:` を追加する迂回を検知できない
/// （それぞれ別の場所を検索するだけで「同一ステップに属するか」「その
/// ステップの実行条件・失敗伝播はどうか」を見ていないため）。ステップ
/// 境界を明示的に切り出すことで、これらを 1 つのステップ本文に対する
/// 検証としてまとめて行える。
struct StepBlock {
    start: usize,
    end: usize,
}

/// `steps:` キーの次行（`steps_start`）から `block_end` までを走査し、
/// `item_indent`（ステップ項目マーカー `- ` の先頭インデント）ちょうどの
/// `- ` から始まる行をステップ境界として切り出す。
///
/// `ci-complete` トップレベルキー（`needs:`/`if:`/`steps:`）が固定
/// インデント 4 という本ファイルの既存前提（`key_at_indent4` 使用箇所）
/// に合わせ、`item_indent` はステップ項目マーカー行の実インデントを
/// 呼び出し側が確定させて渡す（本ファイルでは `steps:` のインデント
/// 4 + 2 = 6 を渡す運用）。
fn scan_top_level_steps(
    stripped: &[String],
    steps_start: usize,
    block_end: usize,
    item_indent: usize,
) -> Vec<StepBlock> {
    let mut starts: Vec<usize> = Vec::new();
    for (i, line) in stripped
        .iter()
        .enumerate()
        .take(block_end)
        .skip(steps_start)
    {
        if line.trim().is_empty() {
            continue;
        }
        let leading = line.len() - line.trim_start_matches(' ').len();
        if leading != item_indent {
            continue;
        }
        if line[item_indent..].starts_with("- ") {
            starts.push(i);
        }
    }
    starts
        .iter()
        .enumerate()
        .map(|(idx, &start)| {
            let end = starts.get(idx + 1).copied().unwrap_or(block_end);
            StepBlock { start, end }
        })
        .collect()
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

    // --- `needs:`/`if:`/`steps:` キーの特定と要素の抽出 ---
    let mut needs_key_lines: Vec<usize> = Vec::new();
    let mut if_key_lines: Vec<(usize, String)> = Vec::new();
    let mut steps_key_lines: Vec<usize> = Vec::new();
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
                "steps" => steps_key_lines.push(i),
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
    //
    // `SKIPPED_ALLOWLIST` から jq プログラム全体（`jq -e '...'` の
    // `'...'` 内側、`to_entries` から `| length == 0` までの全文）と、
    // それを実行するシェル行全体（`RESULTS` への束縛・`echo "${RESULTS}"`
    // による入力・jq への配線を含む）を機械的に組み立て、集約ステップの
    // テキストと **部分文字列の存在ではなく行全体の完全一致** で照合する
    // （イシュー #2324 の P1 是正。PR #2335 の codex-review 再指摘、および
    // 自己レビュー追補を受け、select 述語の内側だけでなくプログラム全体・
    // 入力配線（`env: RESULTS` と `echo "${RESULTS}" |` のパイプ）まで
    // 検証範囲を拡張した）。
    //
    // 旧実装は `map(select((...) | not))` の `(...)` 内側（select 述語）
    // だけを完全一致検証しており、その外側（`-e` フラグの有無・
    // `| length == 0` という最終判定・そもそも `jq` へ何を流し込んでいる
    // か）は一切検証していなかった。このため、述語自体はそのままに
    // (a) 末尾へ `or true` を継ぎ足す、(b) `| length == 0` を
    // `| length >= 0` へ緩める、(c) `-e` フラグを外す、(d) `env:` の
    // `RESULTS` を `toJSON(needs)` から別式（例: `toJSON(github)`）へ
    // すり替える、(e) `echo "${RESULTS}" |` を `echo '{}' |` 等の無関係な
    // 固定入力へすり替える（`RESULTS` の束縛自体は残したまま jq への
    // 入力経路だけ差し替えるため、`toJSON(needs)` の部分文字列存在
    // チェックだけでは検知できない）といった改変を個別に検知できな
    // かった。`env:` 配下の `RESULTS` 行・`jq` を呼ぶシェル行の双方を
    // 行全体の完全一致で照合することで、これらすべての迂回を検知する。
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
    let expected_program =
        format!("to_entries | map(select(({expected_predicate}) | not)) | length == 0");

    // `env:` 配下で `RESULTS` を `toJSON(needs)` へ束縛する行が、唯一の
    // 正規形でちょうど 1 回だけ存在すること（環境変数名を変える・
    // 束縛先の式を変える・複数束縛して未使用の別名を紛れ込ませる等の
    // 迂回をすべて違反側へ倒す反転判定）。
    const EXPECTED_RESULTS_BINDING: &str = "RESULTS: ${{ toJSON(needs) }}";
    let results_binding_lines: Vec<usize> = stripped[block_start..block_end]
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim() == EXPECTED_RESULTS_BINDING)
        .map(|(i, _)| block_start + i)
        .collect();
    match results_binding_lines.len() {
        1 => {}
        0 => violations.push(format!(
            "`ci-complete` の集約ステップに唯一の正規形の `env:` 束縛 \
             `{EXPECTED_RESULTS_BINDING}` が見つからない（`needs` の結果を丸ごと束縛する\
             契約が失われている、または表記が非正規形になっている。反転判定により違反として\
             扱う）"
        )),
        n => violations.push(format!(
            "`ci-complete` の集約ステップに `{EXPECTED_RESULTS_BINDING}` が {n} 回出現している\
             （1 回のみ想定）"
        )),
    }

    // `RESULTS` を jq へ渡すシェル行全体（`echo "${RESULTS}" | jq -e '...'
    // > /dev/null`）が、唯一の正規形でちょうど 1 回だけ存在すること。
    // 行全体一致にすることで、`RESULTS` の束縛は正規のまま入力だけ固定値
    // へすり替える迂回（`echo '{}' | jq ...`）・`-e` フラグの有無・
    // 出力先の変更・述語や最終判定の緩和のいずれも単一の照合で検知する。
    let expected_jq_line =
        format!("echo \"${{RESULTS}}\" | jq -e '{expected_program}' > /dev/null");
    let jq_lines: Vec<(usize, String)> = stripped[block_start..block_end]
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim_start().starts_with("echo \"${RESULTS}\""))
        .map(|(i, line)| (block_start + i, line.trim().to_string()))
        .collect();
    match jq_lines.len() {
        1 => {
            let (line_no, actual) = &jq_lines[0];
            if actual != &expected_jq_line {
                violations.push(format!(
                    "ci.yml:{}: `ci-complete` の集約ステップの `RESULTS` を jq へ渡すシェル行が\
                     期待する正規形と完全一致しない（部分文字列一致ではなく `echo` の入力・\
                     パイプ・`-e` フラグ・select 述語・最終判定〔`| length == 0`〕・出力先を\
                     含む行全体の一致を要求する）。\n  期待: {expected_jq_line}\n  \
                     実際: {actual}",
                    line_no + 1
                ));
            }
        }
        0 => violations.push(format!(
            "`ci-complete` の集約ステップに `RESULTS` を jq へ渡す唯一の正規形の行 \
             `{expected_jq_line}` が見つからない（`echo \"${{RESULTS}}\"` で始まる行が存在\
             しない。`needs` の結果を丸ごと検証する契約が失われている、または `RESULTS` の\
             入力経路が固定値・別の式へすり替えられている可能性がある。反転判定により違反\
             として扱う）"
        )),
        n => violations.push(format!(
            "`ci-complete` の集約ステップに `echo \"${{RESULTS}}\"` で始まる行が {n} 回出現\
             している（1 回のみ想定）"
        )),
    }

    // --- 集約ステップの実行コンテキスト検証 ---
    //
    // 上記 2 点（`RESULTS` 束縛・jq 呼び出し行）はブロック全体から
    // 個別に検索するだけであり、(a) 両者が同一ステップに属すること、
    // (b) そのステップの `run:` ブロックが jq 呼び出し行以外のコマンドを
    // 含まないこと（`RESULTS='{}'` 等を先頭に追加して入力だけをすり替え、
    // jq 呼び出し行自体は無傷のまま集約を無力化する迂回。`{}` を
    // `to_entries` すると `[]` になり `length == 0` が常に真になる）、
    // (c) ステップの実行条件（`if:`）が不在または `always()` であること、
    // (d) 失敗伝播を無効化する `continue-on-error:` が存在しないこと、の
    // いずれも検証していなかった（イシュー #2324 の PR #2335
    // codex-review 再指摘 1 点目）。`steps:` 配下のステップ境界を切り出し、
    // `RESULTS` 束縛・jq 呼び出し行の双方を含む単一のステップへスコープ
    // した検証を行う。
    let mut steps_start: Option<usize> = None;
    match steps_key_lines.len() {
        1 => steps_start = Some(steps_key_lines[0] + 1),
        0 => violations.push("`ci-complete` に `steps:` キーが見つからない".to_string()),
        n => violations.push(format!(
            "`ci-complete` に `steps:` キーが {n} 回出現している（1 回のみ想定）"
        )),
    }

    if let Some(steps_start) = steps_start {
        // `steps:` はインデント 4（`key_at_indent4` 前提）で見つかって
        // いるため、ステップ項目マーカー `- ` はインデント 6 に固定する
        // （`.github/workflows/ci.yml` の実際の慣例と一致）。
        const STEP_ITEM_INDENT: usize = 6;
        let step_blocks = scan_top_level_steps(&stripped, steps_start, block_end, STEP_ITEM_INDENT);

        if let (Some(&binding_line), Some((jq_line_no, _))) =
            (results_binding_lines.first(), jq_lines.first())
        {
            let binding_step = step_blocks
                .iter()
                .find(|s| s.start < binding_line && binding_line < s.end);
            let jq_step = step_blocks
                .iter()
                .find(|s| s.start < *jq_line_no && *jq_line_no < s.end);

            match (binding_step, jq_step) {
                (Some(b), Some(j)) if b.start == j.start => {
                    let step_body_start = b.start + 1;
                    let step_body_end = b.end;
                    // ステップ直下キーの基準インデントは、ステップ項目
                    // マーカー行自体を除いた本文の最小インデントとして
                    // 動的に決定する（P1-2 是正と同じ流儀。空白のみの
                    // 行は `block_base_indent` 内で除外済み）。
                    let step_key_indent =
                        block_base_indent(&stripped, step_body_start, step_body_end)
                            .unwrap_or(STEP_ITEM_INDENT + 2);

                    // (c) 実行条件: 不在または `always()` のみ許容。
                    let if_scan = scan_key_at_indent(
                        &stripped,
                        step_body_start,
                        step_body_end,
                        step_key_indent,
                        "if",
                    );
                    for (i, raw) in &if_scan.non_canonical {
                        violations.push(format!(
                            "ci.yml:{}: `ci-complete` の集約ステップの `if:` 相当のキーが\
                             唯一の正規形（クォート無し `if:`）に一致しない表記になっている\
                             （反転判定により違反として扱う）: {raw}",
                            i + 1
                        ));
                    }
                    if let Some((i, value)) = &if_scan.canonical {
                        if value != "always()" {
                            violations.push(format!(
                                "ci.yml:{}: `ci-complete` の集約ステップに `if: {value}`\
                                 という実行条件が設定されている（依存ジョブが失敗していても\
                                 ステップ自体が実行されず集約が成功する迂回を防ぐため、\
                                 集約ステップは `if:` 不在または `always()` のみ許容する）",
                                i + 1
                            ));
                        }
                    }

                    // (d) 失敗伝播: `continue-on-error:` は不在のみ許容
                    //     （値に関わらず存在自体を違反とする）。
                    let coe_scan = scan_key_at_indent(
                        &stripped,
                        step_body_start,
                        step_body_end,
                        step_key_indent,
                        "continue-on-error",
                    );
                    for (i, raw) in &coe_scan.non_canonical {
                        violations.push(format!(
                            "ci.yml:{}: `ci-complete` の集約ステップに `continue-on-error:` \
                             相当のキーが存在する（表記: {raw}）。失敗伝播を無効化する迂回を\
                             防ぐため、集約ステップは `continue-on-error:` を持たないことを\
                             要求する",
                            i + 1
                        ));
                    }
                    if let Some((i, value)) = &coe_scan.canonical {
                        violations.push(format!(
                            "ci.yml:{}: `ci-complete` の集約ステップに `continue-on-error: \
                             {value}` が設定されている（値に関わらず、失敗伝播を無効化する\
                             迂回を防ぐため集約ステップは `continue-on-error:` を持たないこと\
                             を要求する）",
                            i + 1
                        ));
                    }

                    // (b) run ブロックの内容: jq 呼び出し行以外のコマンド
                    //     を許容しない。
                    let run_scan = scan_key_at_indent(
                        &stripped,
                        step_body_start,
                        step_body_end,
                        step_key_indent,
                        "run",
                    );
                    for (i, raw) in &run_scan.non_canonical {
                        violations.push(format!(
                            "ci.yml:{}: `ci-complete` の集約ステップの `run:` 相当のキーが\
                             唯一の正規形（クォート無し `run:`）に一致しない表記になっている\
                             （反転判定により違反として扱う）: {raw}",
                            i + 1
                        ));
                    }
                    match &run_scan.canonical {
                        None => violations.push(
                            "`ci-complete` の集約ステップに `run: |` が見つからない（jq 呼び\
                             出し行を実行する run ブロックの契約が失われている）"
                                .to_string(),
                        ),
                        Some((run_line, run_value)) => {
                            if run_value != "|" {
                                violations.push(format!(
                                    "ci.yml:{}: `ci-complete` の集約ステップの `run:` が唯一の\
                                     正規形 `run: |`（リテラルブロックスカラー）と一致しない: \
                                     {run_value}",
                                    run_line + 1
                                ));
                            } else {
                                let mut content_end = step_body_end;
                                for (i, line) in stripped
                                    .iter()
                                    .enumerate()
                                    .take(step_body_end)
                                    .skip(run_line + 1)
                                {
                                    if line.trim().is_empty() {
                                        continue;
                                    }
                                    let leading = line.len() - line.trim_start_matches(' ').len();
                                    if leading <= step_key_indent {
                                        content_end = i;
                                        break;
                                    }
                                }
                                let content_lines: Vec<(usize, &str)> = stripped
                                    [run_line + 1..content_end]
                                    .iter()
                                    .enumerate()
                                    .map(|(offset, line)| (run_line + 1 + offset, line.as_str()))
                                    .filter(|(_, line)| !line.trim().is_empty())
                                    .collect();
                                match content_lines.len() {
                                    1 => {
                                        let (i, actual) = content_lines[0];
                                        if actual.trim() != expected_jq_line {
                                            violations.push(format!(
                                                "ci.yml:{}: `ci-complete` の集約ステップの \
                                                 `run:` ブロックの唯一の行が期待する正規形と\
                                                 完全一致しない。\n  期待: {expected_jq_line}\n  \
                                                 実際: {}",
                                                i + 1,
                                                actual.trim()
                                            ));
                                        }
                                    }
                                    n => violations.push(format!(
                                        "`ci-complete` の集約ステップの `run:` ブロックに \
                                         {n} 行の非空行がある（依存ジョブの失敗を隠す前処理・\
                                         後処理の追加を検知するため、run ブロックは `RESULTS` \
                                         を jq へ渡す行のみを許容する。1 回のみ想定）"
                                    )),
                                }
                            }
                        }
                    }
                }
                _ => {
                    violations.push(
                        "`ci-complete` の集約ステップにおいて、`RESULTS` の束縛と `RESULTS` \
                         を jq へ渡す行が同一のステップに属していない（反転判定により違反\
                         として扱う）"
                            .to_string(),
                    );
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

    /// イシュー #2324 の PR #2335 codex-review 再指摘 1 点目: select 述語
    /// 自体は変更せず、末尾の最終判定だけを `| length == 0` から
    /// `| length >= 0` へ緩めると、`map(select(...))` が非空（＝許容外の
    /// 失敗・skip が実在）でも集約ステップが PASS してしまう fail-open に
    /// なる。旧実装は select 述語の内側だけを完全一致検証しており、この
    /// 末尾緩和を検知できなかった。最終判定まで含めて検証することを
    /// 固定する。
    #[test]
    fn fail_skipped_length_check_relaxed_is_violation() {
        let contents = base_fixture().replace("| length == 0", "| length >= 0");
        assert_violations_contain(&contents, "完全一致");
    }

    /// PR #2335 の自己レビュー追補: `length_check_ok` が「`| length == 0`
    /// の直後が英数字でなければ受理」という緩い判定だったため、
    /// `| length == 0 or true'`（jq 上 `(length == 0) or true` と解釈され
    /// 常に真になる）という迂回は、直後の文字が空白であり英数字でない
    /// ため誤って PASS してしまっていた（select 述語内側への `or true`
    /// 迂回と同型だが、プログラム末尾側で起きる変種）。jq プログラム
    /// 全体（`jq -e '...'`）を単一の完全一致で照合する現行実装では
    /// この迂回も自動的に検知されることを固定する。
    #[test]
    fn fail_skipped_length_check_or_true_appended_is_violation() {
        let contents = base_fixture().replace("| length == 0'", "| length == 0 or true'");
        assert_violations_contain(&contents, "完全一致");
    }

    /// jq プログラム全体を検証する現行実装は `-e` フラグの有無も検証
    /// 対象に含む。`-e` 無しの jq はフィルタ結果の真偽に関わらず構文的に
    /// 成功すれば終了コード 0 になり得るため、フラグを外す改変も集約を
    /// vacuous にする迂回として検知できることを固定する。
    #[test]
    fn fail_jq_e_flag_removed_is_violation() {
        let contents = base_fixture().replace("jq -e '", "jq '");
        assert_violations_contain(&contents, "jq -e '");
    }

    /// 自己レビュー追補（P1-1 の入力配線側）: `RESULTS` の束縛（`env:`）は
    /// 正規のまま残し、jq への入力だけ無関係な固定値へすり替える迂回
    /// （`echo "${RESULTS}" | jq ...` → `echo '{}' | jq ...`）は、`{}` を
    /// `to_entries` すると `[]` になり `length == 0` が常に真になるため
    /// 集約が vacuous になる。旧実装（`toJSON(needs)` の部分文字列存在
    /// チェックのみ）はこの迂回を検知できなかった（`RESULTS` の束縛自体は
    /// 変更されないため）。`echo "${RESULTS}" | jq ...` という行全体の
    /// 完全一致検証で検知できることを固定する。
    #[test]
    fn fail_jq_input_decoy_is_violation() {
        let contents =
            base_fixture().replace("echo \"${RESULTS}\" | jq -e '", "echo '{}' | jq -e '");
        assert_violations_contain(&contents, "見つからない");
    }

    /// 自己レビュー追補（P1-1 の入力配線側、その 2）: `RESULTS` を
    /// `toJSON(needs)` ではなく無関係な式（`toJSON(github)`）へ束縛し、
    /// 本来の `toJSON(needs)` は使われない別の環境変数（`UNUSED`）へ
    /// 束縛し直す迂回は、`block_text.contains("toJSON(needs)")` という
    /// 旧実装の部分文字列チェックでは（`UNUSED` 側に文字列が残るため）
    /// 検知できなかった。`RESULTS: ${{ toJSON(needs) }}` という行全体の
    /// 完全一致検証（`UNUSED` 側は無視する）で検知できることを固定する。
    #[test]
    fn fail_tojson_needs_bound_to_unused_env_var() {
        let contents = base_fixture().replace(
            "          RESULTS: ${{ toJSON(needs) }}\n",
            "          RESULTS: ${{ toJSON(github) }}\n          UNUSED: ${{ toJSON(needs) }}\n",
        );
        assert_violations_contain(&contents, "RESULTS: ${{ toJSON(needs) }}");
    }

    /// イシュー #2324 の PR #2335 codex-review 再指摘 2 点目:
    /// `key_at_indent4`（固定インデント 4）で
    /// `scan_job_if` を実装していた旧実装は、許容リスト外ジョブの本文
    /// 全体を（YAML として妥当な）インデント 6 へ揃えたうえで同じ階層に
    /// `if: success()` を書く迂回を検知できなかった（ジョブレベル `if:`
    /// が「存在しない」と誤って読み飛ばされ、`SKIPPED_ALLOWLIST` への
    /// 追加漏れがすり抜ける）。`block_base_indent` でジョブごとに実際の
    /// 基準インデントを動的検出することで検知できることを固定する。
    #[test]
    fn fail_conditional_job_reindented_body_if_is_violation() {
        let contents = base_fixture().replace(
            "  job-b:\n    runs-on: ubuntu-latest\n    steps:\n      - run: \"true\"\n",
            "  job-b:\n      runs-on: ubuntu-latest\n      if: success()\n      steps:\n        - run: \"true\"\n",
        );
        // 「job-b」という文字列一致だけでなく、実際に条件付きジョブ検知
        // 経路（`SKIPPED_ALLOWLIST` 追加漏れ）が発火したことを確認する
        // （job-b への別種の言及で偶然一致する誤判定を防ぐ）。
        assert_violations_contain(&contents, "SKIPPED_ALLOWLIST");
        assert_violations_contain(&contents, "job-b");
    }

    /// 自己レビュー追補（P1-2 続報。PR #2335 codex-review 再指摘 2 点目）:
    /// `strip_comment` はコメントのみの行・完全な空行を `String::new()`
    /// にするが、半角スペース 1 個だけの行（YAML 上は無意味な空行）は
    /// そのまま残す。旧実装の `block_base_indent` は `!line.is_empty()`
    /// だけで除外していたため、ジョブ本文にこの種の空白のみの行が
    /// 1 行あるだけで最小インデントが 1 に引き下がり、通常インデント 4
    /// にある `if: success()` を `scan_job_if` が見落としていた
    /// （`SKIPPED_ALLOWLIST` 追加漏れが検知漏れになる）。`trim()` した
    /// 結果が空の行を除外してから最小インデントを求めることで検知
    /// できることを固定する。
    #[test]
    fn fail_conditional_job_with_whitespace_only_line_is_violation() {
        let contents = base_fixture().replace(
            "  job-b:\n    runs-on: ubuntu-latest\n    steps:\n      - run: \"true\"\n",
            "  job-b:\n \n    runs-on: ubuntu-latest\n    if: success()\n    steps:\n      - run: \"true\"\n",
        );
        assert_violations_contain(&contents, "SKIPPED_ALLOWLIST");
        assert_violations_contain(&contents, "job-b");
    }

    /// 上記の逆側: 本文全体をインデント 6 へ揃えたジョブでも、それが
    /// 既に `SKIPPED_ALLOWLIST` に載っているジョブ（`version-bump-guard`）
    /// であれば、動的検出した基準インデントで正しく `if:` を検知し PASS
    /// することを固定する（過剰検知〔インデント一般化が誤って正規ジョブ
    /// まで弾く〕がないことの確認）。
    #[test]
    fn pass_allowlist_job_reindented_body_if_is_detected() {
        let contents = base_fixture().replace(
            "  version-bump-guard:\n    if: ${{ github.event_name == 'pull_request' }}\n    runs-on: ubuntu-latest\n    steps:\n      - run: \"true\"\n",
            "  version-bump-guard:\n      if: ${{ github.event_name == 'pull_request' }}\n      runs-on: ubuntu-latest\n      steps:\n        - run: \"true\"\n",
        );
        assert_eq!(check_ci_complete_needs_contract(&contents), Ok(()));
    }

    /// 自己レビュー追補（P1-1 続報。PR #2335 codex-review 再指摘 1 点目）:
    /// `RESULTS` の束縛・jq 呼び出し行はブロック全体から個別に検索する
    /// だけであり、両者が同一ステップ・同一 `run:` ブロックに属するかを
    /// 見ていなかった。`run: |` ブロックの先頭へ `RESULTS='{}'` を追加
    /// して jq への入力を無害化する（`{}` の `to_entries` は `[]` になり
    /// `length == 0` が常に真になる）迂回は、既存の jq 呼び出し行自体は
    /// 変更されないため検知できなかった。`run:` ブロックの非空行が
    /// jq 呼び出し行 1 行のみであることを要求する検証で検知できることを
    /// 固定する。
    #[test]
    fn fail_run_block_prepends_results_override_is_violation() {
        let contents = base_fixture().replace(
            "        run: |\n          echo \"${RESULTS}\"",
            "        run: |\n          RESULTS='{}'\n          echo \"${RESULTS}\"",
        );
        // 「run:」という部分文字列は `run: |` が見つからない場合の違反
        // メッセージ等にも登場するため、ここでは count-mismatch 分岐に
        // 固有の文言で検知経路を確認する（弱いニードルによる誤判定を
        // 防ぐ）。
        assert_violations_contain(&contents, "行の非空行がある");
    }

    /// 自己レビュー追補（P1-1 続報、その 2）: 集約ステップ自体へ
    /// `if: success()` を追加すると、依存ジョブが失敗した場合にこの
    /// ステップが実行されず（`ci-complete` ジョブの `if: always()` は
    /// ジョブ自体の実行条件であり、ステップ側の `if:` はさらにその内側
    /// で判定される）、集約チェックが走らないまま暗黙に緑化される。
    /// ステップの実行条件は不在または `always()` のみ許容することを
    /// 固定する。
    #[test]
    fn fail_step_if_success_is_violation() {
        let contents = base_fixture().replace(
            "      - name: check\n        env:\n",
            "      - name: check\n        if: success()\n        env:\n",
        );
        assert_violations_contain(&contents, "if: success()");
    }

    /// 自己レビュー追補（P1-1 続報、その 3）: 集約ステップへ
    /// `continue-on-error: true` を追加すると、jq が非 0 終了しても
    /// ステップとしては成功扱いになり、依存ジョブの失敗が握り消される。
    /// `continue-on-error:` は値に関わらず不在のみ許容することを固定
    /// する。
    #[test]
    fn fail_step_continue_on_error_true_is_violation() {
        let contents = base_fixture().replace(
            "      - name: check\n        env:\n",
            "      - name: check\n        continue-on-error: true\n        env:\n",
        );
        assert_violations_contain(&contents, "continue-on-error");
    }

    /// `if: success()` の逆側: 集約ステップに `if: always()` を明示して
    /// も（`ci-complete` ジョブ自身の `if: always()` と同じ値を重複して
    /// ステップへ書く場合等）過剰検知しないことを固定する（P1-1 是正の
    /// 「不在または `always()` のみ許容」という判定条件のうち、
    /// `always()` 側が実際に PASS することの確認）。
    #[test]
    fn pass_step_if_always_is_ok() {
        let contents = base_fixture().replace(
            "      - name: check\n        env:\n",
            "      - name: check\n        if: always()\n        env:\n",
        );
        assert_eq!(check_ci_complete_needs_contract(&contents), Ok(()));
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

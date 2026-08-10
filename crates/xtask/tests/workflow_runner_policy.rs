//! `.claude/rules/ci.md` の Runner 方針（ホステッドランナー既定・
//! `runs-on: self-hosted` 禁止。トラッキング #1220、2026-08-07 方針転換／
//! さらに `runs-on` は `ubuntu-latest` 単一に限定。ユーザー指示 2026-08-10）を
//! `.github/workflows/*.yml` に対して fail-closed に機械強制する契約テスト
//! （イシュー #1239）。
//!
//! ## 契約の意味論（その 2: `runs-on` は `ubuntu-latest` のみ）
//!
//! 非コメント行の `runs-on` キーの値が、リテラル `ubuntu-latest`
//! （クォート有無は不問）と完全一致することを契約とする。キーと `:` の
//! 間の空白（`runs-on : windows-latest`。YAML では合法）も正規化して
//! 検査する（正規化を怠ると空白 1 つで検知を回避できる fail-open に
//! なる。PR #1301 の codex レビュー P0 指摘）。
//! `windows-latest` / `macos-latest` / `ubuntu-24.04-arm` 等の他 OS・他
//! イメージはもちろん、`${{ matrix.os }}` のような非リテラル指定、
//! block sequence / `labels:` mapping 形もすべて不一致＝ FAIL とする
//! （実 runner を YAML の 1 行から決定的に読み取れない書き方を許すと、
//! 間接指定経由で他 OS が入り込む経路がサイレントに開くため）。
//!
//! ### 本テストの射程外（codex-review 例外との両立）
//!
//! reusable workflow の呼び出しジョブ（job-level `uses:`）は `runs-on` を
//! 持たないため、本テストの走査対象に現れない。`.claude/rules/ci.md` が
//! 唯一の例外として承認している `codex-review.yml` の codex 実行ジョブ
//! （runner は呼び出し先の `runner-label` 入力既定値で決まる）は、この
//! 射程外で成立している。今回の `ubuntu-latest` 単一化は OS・イメージ
//! 選択に関する規則であり、既承認の codex 例外を撤回するものではない。
//!
//! ## 契約の意味論（その 1: `self-hosted` 禁止）
//!
//! 「`.github/workflows/` 配下の全ワークフロー YAML について、コメントを
//! 除去した後の内容に `self-hosted` トークンが一切現れない」ことを契約と
//! する。`runs-on:` キーの値のみを解析対象にする案よりも意図的に広い
//! 走査範囲を取る。理由は、`runs-on: ${{ matrix.os }}` + `matrix.os:
//! [self-hosted]` のような間接指定や、スカラー / flow sequence / block
//! sequence / `labels:` mapping 形などの YAML 表記揺れを、行ベース走査の
//! まま漏れなく検知するため（`runs-on` の値解析に限定すると表記揺れ
//! ごとに検知器を追随させる保守が必要になり、追随漏れがサイレント PASS
//! になる）。副作用として `run:` スクリプト内の文字列リテラル等で
//! `self-hosted` と書いても FAIL するが、方針上そのような記述は不要で
//! あり、歴史的経緯の言及はコメントに書けば足りるため、この厳格性は
//! 意図的である。
//!
//! ## コメントとして誤検知しない対象
//!
//! `.github/workflows/` には旧 self-hosted 方針時代の経緯コメント
//! （`ci.yml` / `image-size.yml` / `musl-smoke.yml` 等の移行経緯コメント）が
//! 多数残っており、これらは正当な歴史記録として維持される
//! （`.claude/rules/ci.md` 参照）。本契約はコメントを除去したうえで走査
//! するため、これらの行を誤検知しない。
//!
//! ## 外部 YAML パーサ不採用
//!
//! `workflow_shared_target_contract.rs` / `template_deny_workflow.rs` と
//! 同じく、行ベースの文字列走査に留め外部クレートへの依存を追加しない
//! （REQ-3・xtask 外部依存ゼロ方針）。
//!
//! ## スコープ外
//!
//! `templates/default/.github/workflows/`（`fw new` が生成するユーザー
//! プロジェクト向けテンプレート）は本契約の対象外とする。ユーザー
//! プロジェクトの runner 方針は本リポジトリ自身の CI 運用（本契約の
//! 対象）とは別論点であり、拡張するかは別途判断する
//! （`.claude/rules/out-of-scope-tracking.md` 参照）。

use std::path::PathBuf;

/// workspace ルート（`xtask/` の親ディレクトリ）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/xtask/ から 2 段上でワークスペースルートに到達する（イシュー #436）")
        .to_path_buf()
}

fn workflows_dir() -> PathBuf {
    workspace_root().join(".github/workflows")
}

/// 1 行からコメント部分を切り落とす。
///
/// トリム後に `#` で始まる行は行全体をコメントとして扱い空文字列を返す。
/// 行中コメントは、シングル/ダブルクォートの外側かつ直前が空白（または
/// 行頭）の `#` 以降を切り落とす。クォート状態は行ごとにリセットする
/// （`run: |` の複数行シェルブロックに崩れたクォートが含まれていても、
/// 次の行の判定を汚染しないため。ファイル全体で 1 パスの状態機械にすると
/// シェルコード中の不釣り合いなクォートで `#` 除去が誤作動し、正当な
/// 歴史記録コメントを非コメット扱いする過検知が起き得る）。
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

/// ファイル内容から `self-hosted` の非コメント出現箇所を検出する。
///
/// 戻り値は (1-indexed 行番号, 元の行内容) のリスト。
fn find_self_hosted_violations(content: &str) -> Vec<(usize, String)> {
    content
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            let stripped = strip_comment(line);
            if stripped.contains("self-hosted") {
                Some((idx + 1, line.to_string()))
            } else {
                None
            }
        })
        .collect()
}

/// 方針上唯一許容される runner ラベル（`.claude/rules/ci.md` Runner 方針、
/// ユーザー指示 2026-08-10）。
const ALLOWED_RUNNER: &str = "ubuntu-latest";

/// 1 行が `runs-on` キーであればその値（コロン以降のトリム済み文字列）を
/// 返す。
///
/// YAML はキーと `:` の間の空白を許すため（`runs-on : windows-latest` も
/// 合法）、`strip_prefix("runs-on:")` だけで判定すると空白を 1 つ入れる
/// だけで検知を回避できる fail-open になる（codex レビュー P0 指摘、
/// PR #1301）。キー名一致のあと空白を読み飛ばしてから `:` を要求する形で
/// 正規化し、この抜け道を塞ぐ。`runs-on-foo:` のような別キーは `:` 以外の
/// 文字に当たるため `None` を返す。
fn runs_on_value(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix("runs-on")?;
    let rest = rest.trim_start();
    Some(rest.strip_prefix(':')?.trim())
}

/// ファイル内容から `runs-on` の値が `ubuntu-latest` 以外である箇所を
/// 検出する。
///
/// 戻り値は (1-indexed 行番号, 元の行内容) のリスト。値が空（block
/// sequence 形・`labels:` mapping 形のように次行以降へ続く書き方）の場合も
/// 「1 行から runner を決定的に読み取れない」ため違反として扱う
/// （fail-closed）。
fn find_non_ubuntu_runs_on(content: &str) -> Vec<(usize, String)> {
    content
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            let stripped = strip_comment(line);
            let value = runs_on_value(&stripped)?;
            // クォート表記（`"ubuntu-latest"` / `'ubuntu-latest'`）を正規化する。
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                .unwrap_or(value);
            if value == ALLOWED_RUNNER {
                None
            } else {
                Some((idx + 1, line.to_string()))
            }
        })
        .collect()
}

/// `.github/workflows/*.yml` を列挙する。
///
/// `.yaml` 拡張子は本リポジトリでは使われていないが、将来の追加を
/// 見逃さないよう `.yml` / `.yaml` の双方を対象にする。IO エラーは
/// fail-closed に即 panic する（環境エラーとテストのすり抜けを区別
/// しない設計は他契約テストと同型）。
fn workflow_files() -> Vec<PathBuf> {
    let dir = workflows_dir();
    let entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!(".github/workflows/ の読み込みに失敗した: {dir:?}: {e}"));

    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && matches!(
                    path.extension().and_then(|ext| ext.to_str()),
                    Some("yml") | Some("yaml")
                )
        })
        .collect();
    files.sort();
    files
}

#[test]
fn workflows_have_no_self_hosted_outside_comments() {
    let mut violations: Vec<String> = Vec::new();

    for path in workflow_files() {
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{path:?} の読み込みに失敗した: {e}"));
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<unknown>");

        for (line_no, line) in find_self_hosted_violations(&content) {
            violations.push(format!("{file_name}:{line_no}: {}", line.trim()));
        }
    }

    assert!(
        violations.is_empty(),
        "runs-on の self-hosted 残置を検知した（`.claude/rules/ci.md` \
         Runner 方針違反）。ホステッドランナーへ移行するか、歴史的経緯の \
         記述はコメントへ移すこと:\n{}",
        violations.join("\n")
    );
}

#[test]
fn workflows_run_only_on_ubuntu_latest() {
    let mut violations: Vec<String> = Vec::new();

    for path in workflow_files() {
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{path:?} の読み込みに失敗した: {e}"));
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<unknown>");

        for (line_no, line) in find_non_ubuntu_runs_on(&content) {
            violations.push(format!("{file_name}:{line_no}: {}", line.trim()));
        }
    }

    assert!(
        violations.is_empty(),
        "runs-on に `{ALLOWED_RUNNER}` 以外の指定を検知した（`.claude/rules/ci.md` \
         Runner 方針違反）。windows-latest / macos-latest 等の他 OS・他イメージ、\
         および `${{{{ matrix.os }}}}` 等の非リテラル指定は使用しない:\n{}",
        violations.join("\n")
    );
}

/// 検知器自体が何も読まずに PASS してしまう退行（glob ミス・ディレクトリ
/// 移動・`strip_comment` の暴走等）を防ぐための非空性チェック。
#[test]
fn workflow_scan_is_not_vacuous() {
    let files = workflow_files();
    assert!(
        !files.is_empty(),
        ".github/workflows/ 配下にワークフロー YAML が 1 件も見つからなかった"
    );
    assert!(
        files
            .iter()
            .any(|p| p.file_name().and_then(|n| n.to_str()) == Some("ci.yml")),
        "ci.yml が走査対象に含まれていない"
    );

    let mut runs_on_count = 0usize;
    for path in &files {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{path:?} の読み込みに失敗した: {e}"));
        for line in content.lines() {
            let stripped = strip_comment(line);
            if runs_on_value(&stripped).is_some() {
                runs_on_count += 1;
            }
        }
    }
    assert!(
        runs_on_count > 0,
        "走査対象の非コメント行に runs-on: キーが 1 件も見つからなかった \
         （strip_comment がコメント除去を暴走させ正当な行まで消していないか確認する）"
    );
}

#[test]
fn detects_scalar_self_hosted() {
    let content = "jobs:\n  test:\n    runs-on: self-hosted\n";
    let violations = find_self_hosted_violations(content);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].0, 3);
}

#[test]
fn detects_flow_sequence_self_hosted() {
    let content = "jobs:\n  test:\n    runs-on: [self-hosted, Windows]\n";
    let violations = find_self_hosted_violations(content);
    assert_eq!(violations.len(), 1);
}

#[test]
fn detects_block_sequence_self_hosted() {
    let content = "jobs:\n  test:\n    runs-on:\n      - self-hosted\n      - linux\n";
    let violations = find_self_hosted_violations(content);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].0, 4);
}

#[test]
fn detects_labels_mapping_self_hosted() {
    let content = "jobs:\n  test:\n    runs-on:\n      group: pool\n      labels: [self-hosted]\n";
    let violations = find_self_hosted_violations(content);
    assert_eq!(violations.len(), 1);
}

#[test]
fn detects_matrix_indirect_self_hosted() {
    let content = "jobs:\n  test:\n    strategy:\n      matrix:\n        os: [self-hosted]\n    runs-on: ${{ matrix.os }}\n";
    let violations = find_self_hosted_violations(content);
    assert_eq!(violations.len(), 1);
}

#[test]
fn detects_quoted_self_hosted() {
    let content = "jobs:\n  test:\n    runs-on: \"self-hosted\"\n";
    let violations = find_self_hosted_violations(content);
    assert_eq!(violations.len(), 1);
}

#[test]
fn does_not_flag_full_line_comment() {
    // 旧 fw-new-windows-verify.yml（イシュー #413、2026-08-10 に削除）が持って
    // いた引用コメントと同型のパターン。歴史的経緯の記述は引き続き許容される。
    let content = "# （`runs-on: [self-hosted, Windows]`。Windows ラベルの self-hosted runner が\n";
    let violations = find_self_hosted_violations(content);
    assert!(violations.is_empty());
    assert!(find_non_ubuntu_runs_on(content).is_empty());
}

#[test]
fn detects_windows_runner() {
    let content = "jobs:\n  verify:\n    runs-on: windows-latest\n";
    let violations = find_non_ubuntu_runs_on(content);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].0, 3);
}

#[test]
fn detects_macos_runner() {
    let content = "jobs:\n  verify:\n    runs-on: macos-latest\n";
    assert_eq!(find_non_ubuntu_runs_on(content).len(), 1);
}

#[test]
fn detects_non_latest_ubuntu_image() {
    // `ubuntu-24.04` / `ubuntu-24.04-arm` のようなイメージ固定・arm 変種も、
    // 「ubuntu-latest だけ」の規則に反するため検知する。
    let content = "jobs:\n  a:\n    runs-on: ubuntu-24.04\n  b:\n    runs-on: ubuntu-24.04-arm\n";
    assert_eq!(find_non_ubuntu_runs_on(content).len(), 2);
}

#[test]
fn detects_matrix_indirect_runner() {
    let content = "jobs:\n  test:\n    strategy:\n      matrix:\n        os: [ubuntu-latest, windows-latest]\n    runs-on: ${{ matrix.os }}\n";
    let violations = find_non_ubuntu_runs_on(content);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].0, 6);
}

#[test]
fn detects_block_sequence_runner() {
    // 値が次行以降へ続く書き方は 1 行から runner を確定できないため違反扱い。
    let content = "jobs:\n  test:\n    runs-on:\n      - ubuntu-latest\n";
    assert_eq!(find_non_ubuntu_runs_on(content).len(), 1);
}

#[test]
fn detects_runner_with_whitespace_before_colon() {
    // YAML はキーと `:` の間の空白を許す。空白 1 つで検知を回避できると
    // fail-open になるため、この表記でも違反として検出する
    // （PR #1301 の codex レビュー P0 指摘の回帰テスト）。
    let content = "jobs:\n  a:\n    runs-on : windows-latest\n  b:\n    runs-on\t: macos-latest\n";
    let violations = find_non_ubuntu_runs_on(content);
    assert_eq!(violations.len(), 2);
    assert_eq!(violations[0].0, 3);
    assert_eq!(violations[1].0, 5);
}

#[test]
fn accepts_ubuntu_latest_with_whitespace_before_colon() {
    let content = "jobs:\n  test:\n    runs-on : ubuntu-latest\n";
    assert!(find_non_ubuntu_runs_on(content).is_empty());
}

#[test]
fn does_not_treat_other_keys_as_runs_on() {
    // `runs-on` を接頭辞に持つ別キー・別値を誤検知しない
    // （キー名一致の後に空白を読み飛ばしても `:` 以外なら非対象）。
    let content = "jobs:\n  test:\n    runs-on-note: windows-latest\n    steps:\n      - run: echo runs-on windows-latest\n";
    assert!(find_non_ubuntu_runs_on(content).is_empty());
}

#[test]
fn accepts_quoted_ubuntu_latest() {
    let content = "jobs:\n  a:\n    runs-on: \"ubuntu-latest\"\n  b:\n    runs-on: 'ubuntu-latest'\n  c:\n    runs-on: ubuntu-latest # 旧: windows-latest\n";
    assert!(find_non_ubuntu_runs_on(content).is_empty());
}

#[test]
fn does_not_flag_trailing_comment() {
    let content = "jobs:\n  test:\n    runs-on: ubuntu-latest # 旧: self-hosted\n";
    let violations = find_self_hosted_violations(content);
    assert!(violations.is_empty());
}

#[test]
fn does_not_flag_workflow_without_self_hosted() {
    let content =
        "jobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hello\n";
    let violations = find_self_hosted_violations(content);
    assert!(violations.is_empty());
}

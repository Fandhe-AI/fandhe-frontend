//! `.claude/rules/ci.md` の Runner 方針（ホステッドランナー既定・
//! `runs-on: self-hosted` 禁止。トラッキング #1220、2026-08-07 方針転換）を
//! `.github/workflows/*.yml` に対して fail-closed に機械強制する契約テスト
//! （イシュー #1239）。
//!
//! ## 契約の意味論
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
//! `.github/workflows/` には旧 self-hosted 方針時代の経緯コメント（例:
//! `fw-new-windows-verify.yml` の `# runs-on: [self-hosted, Windows]` 引用、
//! `ci.yml` / `image-size.yml` / `musl-smoke.yml` 等の移行経緯コメント）が
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
            if stripped.trim_start().starts_with("runs-on:") {
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
    // fw-new-windows-verify.yml 18 行目の実在パターン（引用コメント）。
    let content = "# （`runs-on: [self-hosted, Windows]`。Windows ラベルの self-hosted runner が\n";
    let violations = find_self_hosted_violations(content);
    assert!(violations.is_empty());
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

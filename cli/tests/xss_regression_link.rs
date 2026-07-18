//! TASK-13.3c（#141）: `fw gate`（`cli/src/gate.rs`, TASK-13.3・#138）の
//! `test` チェックが XSS 回帰テスト（TASK-1.2、`core/tests/xss_escape.rs`
//! 相当）の合否をそのまま反映し、エスケープ実装の退行を BLOCKED に導く
//! ことを固定する統合テスト。
//!
//! # 連携の内容（イシュー #141 の核心）
//!
//! `fw gate` の `test` チェック（`gate.rs::run_cargo_test`）は
//! `cargo test --locked -p <宣言クレート>` を実行するだけであり、
//! 「XSS 回帰テストが失敗したら BLOCKED になる」こと自体は `cargo test`
//! の標準的な合否伝播にすぎない。しかし本リポジトリのフィクスチャ
//! （`negative_cases.rs`）にはテストコード自体が存在しないケースしかなく、
//! 「REQ-1 のエスケープ実装が退行した場合に `test` チェック経由で検知され、
//! かつ `default_escape_check`（raw_html 検出）とは独立した経路で
//! ブロックされる」という連携そのものを固定するテストが無かった
//! （計画 §2-2 のギャップ）。本ファイルはこのギャップを埋める。
//!
//! # 正例・負例
//!
//! - 正例（[`fixture_with_passing_xss_regression_test_passes_test_check`]）:
//!   `core/src/escape.rs` と同じ 5 文字置換のエスケープ実装を持つフィクス
//!   チャが `test` チェックを含む全チェックを通過することを確認する。
//! - 負例（[`escaping_regression_fails_test_check_and_blocks_gate`]）:
//!   同フィクスチャのエスケープ実装を素通し（入力をそのまま返す）へ退行
//!   させ、`test` チェックのみが failed になって `fw gate` 全体が BLOCKED
//!   になることを確認する。`default_escape_check` が引き続き passed で
//!   あることを併せて検証し、「検知が `test` チェック経由であること」
//!   （raw_html 検出経由ではないこと）を保証する。
//!
//! フィクスチャ書き出し・`fw` 起動・JSON レポート判定の共通ヘルパーは
//! `negative_cases.rs`（TASK-13.5・#148）と共用する（`tests/support/mod.rs`
//! 冒頭コメント参照）。

mod support;

use support::{check_passed, run_fw_gate, write_xss_case_project};

/// `core/src/escape.rs` の既定エスケープ仕様と同一の 5 文字置換
/// （`&` を最初に処理する順序契約も含む）を実装したフィクスチャ用
/// `lib.rs`。
fn passing_escape_lib_rs() -> &'static str {
    r#"//! TASK-13.3c フィクスチャ用の最小エスケープ実装
//! （`core/src/escape.rs` の 5 文字置換仕様と同一の契約を持つ）。

/// 既定のエスケープ規則に従って HTML エンティティ化した新しい `String` を返す。
/// `&` を最初に処理し、後続のエンティティ化で生成された `&` を再エスケープ
/// しない（`core/src/escape.rs` と同じ処理順序契約）。
pub fn escape_html_content(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            other => out.push(other),
        }
    }
    out
}
"#
}

/// 正例（TASK-1.2 連携が壊れていないことの対照群）。
///
/// エスケープ実装が仕様どおりのフィクスチャで `fw gate` を実行し、
/// `type_check` / `default_escape_check` / `lint` / `test` の 4 チェック
/// が通過することを確認する。後続の負例テストが「注入した退行」に起因して
/// `test` チェックのみ failed になっていることを保証する基盤であり、
/// 本テストが落ちる場合は負例側の失敗を環境要因と区別できない。
///
/// Cursor Bugbot（PR #281, review 4727301533）指摘: 本テストは従来
/// `type_check` / `default_escape_check` / `lint` / `test` の 4 チェックの
/// `passed` のみをアサートし、`fw gate` 全体の終了コード（`gate_result`）を
/// 見ていなかった。そのため cargo-deny 導入環境で `policy` チェックのみが
/// BLOCKED でも本テストは（4 チェックが passed のままなので）成功してしまい、
/// 「正例フィクスチャは `fw gate` を無条件に通過する」という正例側
/// （positive control）の前提が実際には保証されていなかった。
///
/// PR #281 CI（`forbid-unsafe` ジョブ）指摘: `policy` チェック
/// （`gate.rs::policy_check`）は `deny.toml` が存在しても cargo-deny 本体が
/// 未導入の環境では起動自体に失敗し fail-closed で failed になる
/// （`negative_cases.rs::baseline_fixture_passes_core_checks` と同じ契約）。
/// cargo-deny のインストールステップは `test` ジョブ（TASK-13.3c）にのみ
/// 存在し `forbid-unsafe` ジョブには存在しないため、`gate_result` /
/// 終了コードを cargo-deny 導入有無に関わらず PASS/0 に固定してしまうと
/// `forbid-unsafe` ジョブでは常に失敗する。ここでは `negative_cases.rs` と
/// 同一パターンで `cargo_deny_available()` により分岐し、導入環境では
/// `policy` の passed と `gate_result` の終了コード（0 = PASS）まで検証し、
/// 未導入環境では「`policy` のみ fail-closed で failed になり `gate_result`
/// は BLOCKED（終了コード 1）」という契約自体を検証する。いずれの分岐でも
/// `type_check` / `default_escape_check` / `lint` / `test` の 4 チェックが
/// 通過することは共通のアサーションとして維持する。
#[test]
fn fixture_with_passing_xss_regression_test_passes_test_check() {
    let project = write_xss_case_project("passing", passing_escape_lib_rs());
    let (code, stdout, stderr) = run_fw_gate(&project);

    assert_eq!(
        check_passed(&stdout, "type_check"),
        Some(true),
        "正例フィクスチャで type_check が失敗した: stdout={stdout} stderr={stderr}"
    );
    assert_eq!(
        check_passed(&stdout, "default_escape_check"),
        Some(true),
        "正例フィクスチャで default_escape_check が失敗した（raw_html 未使用のはず）: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "lint"),
        Some(true),
        "正例フィクスチャで lint が失敗した: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "test"),
        Some(true),
        "正例フィクスチャで XSS 回帰テスト（test チェック）が失敗した: stdout={stdout} stderr={stderr}"
    );

    if support::cargo_deny_available() {
        assert_eq!(
            check_passed(&stdout, "policy"),
            Some(true),
            "cargo-deny 導入環境で正例フィクスチャの policy チェックが失敗した: stdout={stdout} stderr={stderr}"
        );
        assert_eq!(
            code, 0,
            "cargo-deny 導入環境では正例フィクスチャは fw gate を PASS するはず: stdout={stdout} stderr={stderr}"
        );
        assert!(
            stdout.contains("\"gate_result\":\"PASS\""),
            "cargo-deny 導入環境で正例フィクスチャの gate_result が PASS でない: stdout={stdout}"
        );
    } else {
        // cargo-deny 未導入環境（例: forbid-unsafe ジョブ）では policy のみ
        // fail-closed で failed になり、他の 4 チェックは通過したまま全体
        // として BLOCKED になる、という negative_cases.rs と同一の
        // fail-closed 契約を確認する（cargo-deny 不在は「エスケープ退行」
        // ではないため、この分岐に来ても対照群としての妥当性は損なわれない）。
        assert_eq!(
            check_passed(&stdout, "policy"),
            Some(false),
            "cargo-deny 未導入環境では policy は fail-closed で failed のはず: stdout={stdout} stderr={stderr}"
        );
        assert_eq!(
            code, 1,
            "cargo-deny 未導入環境では policy の fail-closed により BLOCKED（終了コード 1）のはず: stdout={stdout} stderr={stderr}"
        );
        assert!(
            stdout.contains("\"gate_result\":\"BLOCKED\""),
            "cargo-deny 未導入環境で正例フィクスチャの gate_result が BLOCKED でない: stdout={stdout}"
        );
    }
}

/// 負例（TASK-1.2 連携の核心アサーション）。
///
/// エスケープ実装を `input.to_string()`（素通し）へ退行させ、
/// `tests/xss_escape.rs` の代表ペイロード検証が失敗することで
/// `test` チェックが failed になり、`fw gate` 全体が BLOCKED になることを
/// 確認する。`type_check`・`default_escape_check` は無関係のまま通過する
/// ことも確認し、「検知が `test` チェック（= XSS 回帰テスト）経由であって
/// `default_escape_check`（raw_html 静的検出）経由ではないこと」という
/// 連携の特定性を保証する（イシュー #141 の受け入れ基準そのもの）。
#[test]
fn escaping_regression_fails_test_check_and_blocks_gate() {
    let regressed = support::replace_unique(
        passing_escape_lib_rs(),
        "pub fn escape_html_content(input: &str) -> String {\n    let mut out = String::with_capacity(input.len());\n    for c in input.chars() {\n        match c {\n            '&' => out.push_str(\"&amp;\"),\n            '<' => out.push_str(\"&lt;\"),\n            '>' => out.push_str(\"&gt;\"),\n            '\"' => out.push_str(\"&quot;\"),\n            '\\'' => out.push_str(\"&#x27;\"),\n            other => out.push(other),\n        }\n    }\n    out\n}",
        "pub fn escape_html_content(input: &str) -> String {\n    // TASK-13.3c 負例: エスケープを退行させ素通しにする。\n    input.to_string()\n}",
    );

    let project = write_xss_case_project("regressed", &regressed);
    let (code, stdout, stderr) = run_fw_gate(&project);

    assert_eq!(
        code, 1,
        "エスケープ退行が fw gate を通過してしまった（BLOCKED になるはず）: stdout={stdout} stderr={stderr}"
    );
    assert!(
        stdout.contains("\"gate_result\":\"BLOCKED\""),
        "stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "test"),
        Some(false),
        "エスケープ退行により XSS 回帰テスト（test チェック）が failed であるはず: stdout={stdout} stderr={stderr}"
    );
    assert_eq!(
        check_passed(&stdout, "type_check"),
        Some(true),
        "エスケープ退行とは無関係な type_check は通過するはず（コンパイル自体は成立、ブロック理由の特定性）: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "default_escape_check"),
        Some(true),
        "default_escape_check は raw_html 非使用のため通過し続けるはず \
         （= 検知が test チェック経由であることの核心アサーション）: stdout={stdout}"
    );
}

//! TASK-13.4b（#145）シナリオ 1（バグ修正）の回帰テスト。
//!
//! # 契約
//!
//! PoC-7（`docs/spec/03-poc/ai-self-maintenance/scenarios/
//! bugfix-escape-regression/{impact.json, gate-before-fix.json,
//! gate-after-fix.json}`）が実測した「`escape_html` のシングルクォート
//! エスケープ欠落」回帰を、製品 CLI（`fw impact`/`fw gate`、実バイナリ・実
//! ツールチェーン起動込み）に対して再現する。REQ-1（既定エスケープ）と
//! REQ-13（検証・制約の強制）の交点である「エスケープ回帰がゲートで確実に
//! ブロックされること」を、`impact` による事前判定 → 欠陥混入 → BLOCKED →
//! 修正 → PASS というライフサイクル全体で担保する。
//!
//! 本ファイルの削除・弱体化（アサーション削除・`#[ignore]` 付与等）は
//! 上記ライフサイクル全体の回帰保証を失わせるため行わない
//! （coding-rust.md「テストの `#[ignore]` 追加でごまかさない」）。
//!
//! # ヘルメチック性
//!
//! `common.rs` のフィクスチャは外部依存ゼロの path 依存クレートのみで構成し、
//! `cargo generate-lockfile --offline` で決定的にロックファイルを生成する
//! ため、本ファイルの全テストはネットワークアクセスを行わない。

use crate::common::{
    cargo_deny_available, check_passed, replace_unique, run_fw, scenario1_core_lib_rs,
    write_scenario1_project, SINGLE_QUOTE_ESCAPE_ARM, SINGLE_QUOTE_ESCAPE_ARM_REGRESSED,
};

/// ケース 0（ベースライン対照）。
///
/// 無改変のフィクスチャが `fw gate` の `type_check`/`default_escape_check`/
/// `lint`/`test` の 4 チェックすべてを通過することを確認する対照群。後続の
/// `gate_blocks_escape_regression_and_passes_after_fix` が「注入した欠陥」に
/// 起因して BLOCKED になっていることを保証する基盤であり、このテストが
/// 落ちる場合はシナリオ側の失敗を環境要因と区別できない。
///
/// `policy` チェックのみ、cargo-deny の導入有無で環境ごとに挙動が変わる
/// （`negative_cases.rs::baseline_fixture_passes_core_checks` と同じ吸収方針）。
#[test]
fn baseline_passes_gate() {
    let project = write_scenario1_project("baseline", scenario1_core_lib_rs());
    let (code, stdout, stderr) = run_fw("gate", &[], &project);

    assert_eq!(
        check_passed(&stdout, "type_check"),
        Some(true),
        "ベースラインで type_check が失敗した（対照群が壊れている）: stdout={stdout} stderr={stderr}"
    );
    assert_eq!(
        check_passed(&stdout, "default_escape_check"),
        Some(true),
        "ベースラインで default_escape_check が失敗した: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "lint"),
        Some(true),
        "ベースラインで lint が失敗した: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "test"),
        Some(true),
        "ベースラインで test が失敗した（text_node_is_escaped_by_default 回帰）: stdout={stdout}"
    );

    if cargo_deny_available() {
        assert_eq!(
            code, 0,
            "cargo-deny 導入環境ではベースラインは PASS するはず: stdout={stdout}"
        );
        assert!(
            stdout.contains("\"gate_result\":\"PASS\""),
            "stdout={stdout}"
        );
        assert_eq!(
            check_passed(&stdout, "policy"),
            Some(true),
            "stdout={stdout}"
        );
    } else {
        // cargo-deny 未導入環境（本リポジトリ CI 相当）では policy のみ
        // fail-closed で failed になり、他の 4 チェックは通過したまま全体
        // として BLOCKED になる、という fail-closed 契約を確認する。
        assert_eq!(
            code, 1,
            "cargo-deny 未導入環境では policy の fail-closed により BLOCKED (終了コード 1) のはず: stdout={stdout}"
        );
        assert!(
            stdout.contains("\"gate_result\":\"BLOCKED\""),
            "stdout={stdout}"
        );
        assert_eq!(
            check_passed(&stdout, "policy"),
            Some(false),
            "stdout={stdout}"
        );
    }
}

/// `fw impact render` がシナリオ 1 の影響範囲を機械的に提示することを検証
/// する（REQ-13 受け入れ基準: impact が影響範囲・`breaking_risk`・
/// `requires_human_approval` を提示する）。
///
/// `render` は `core/src/lib.rs`（`fandhe-frontend-core`）で定義され、`app/src/lib.rs`
/// （`fandhe-frontend-app`）・`wasm-client/src/lib.rs`（`fandhe-frontend-wasm-client`）の双方から
/// 呼ばれる。`fandhe-frontend-wasm-client` は `cli/src/impact.rs::CLIENT_BOUNDARY_CRATES`
/// と完全一致するため、影響クレート数（2 件）が `high` 判定の閾値
/// （3 件以上）未満であっても、クライアント境界への波及により
/// `breaking_risk: high`・`requires_human_approval: true` となる
/// （`judge_breaking_risk`/`requires_human_approval`、
/// `docs/design/impact-analysis-design.md` §3.4/§3.5 のスキーマに準拠）。
/// ルート定義を含まないフィクスチャのため `affected_routes` は空。
#[test]
fn impact_reports_high_risk_for_render() {
    let project = write_scenario1_project("impact", scenario1_core_lib_rs());
    let (code, stdout, stderr) = run_fw("impact", &["render"], &project);

    assert_eq!(
        code, 0,
        "render の定義が一意に見つかる想定であり `fw impact` は正常終了するはず: stdout={stdout} stderr={stderr}"
    );
    assert!(stdout.contains("\"symbol\":\"render\""), "stdout={stdout}");
    assert!(
        stdout.contains("\"defined_in_crate\":\"fandhe-frontend-core\""),
        "render は core/src/lib.rs（fandhe-frontend-core）で定義されるはず: stdout={stdout}"
    );
    assert!(
        stdout.contains("\"ambiguous\":false"),
        "render の定義は core 1 箇所のみのはず: stdout={stdout}"
    );
    assert!(
        stdout.contains("\"file\":\"app/src/lib.rs\""),
        "app/src/lib.rs が affected_files に含まれるはず: stdout={stdout}"
    );
    assert!(
        stdout.contains("\"file\":\"wasm-client/src/lib.rs\""),
        "wasm-client/src/lib.rs が affected_files に含まれるはず: stdout={stdout}"
    );
    assert!(
        stdout.contains("\"affected_crates\":[\"fandhe-frontend-app\",\"fandhe-frontend-wasm-client\"]"),
        "affected_crates は fandhe-frontend-app・fandhe-frontend-wasm-client の 2 件のはず: stdout={stdout}"
    );
    assert!(
        stdout.contains("\"affected_routes\":[]"),
        "本フィクスチャはルート定義を含まないため affected_routes は空のはず: stdout={stdout}"
    );
    assert!(
        stdout.contains("\"breaking_risk\":\"high\""),
        "fandhe-frontend-wasm-client（クライアント境界）への波及により high 判定のはず: stdout={stdout}"
    );
    assert!(
        stdout.contains("\"requires_human_approval\":true"),
        "breaking_risk が high のため人間承認が必要なはず: stdout={stdout}"
    );
}

/// ケース本体（PoC-7 `bugfix-escape-regression` シナリオ再現）。
///
/// (a) `escape_html` のシングルクォートエスケープ arm を「無変換で素通しする」
/// 内容へ [`replace_unique`] で置換し、`fw gate` が `test` チェックの失敗
/// （`text_node_is_escaped_by_default` の回帰）により BLOCKED になることを
/// 確認する。`type_check`/`lint`/`default_escape_check` は無関係のまま通過
/// することも確認し、ブロック理由の特定性（エスケープロジックの意味的な
/// バグであってコンパイル失敗・lint 違反ではないこと）を保証する。
///
/// (b) ベースライン内容へ書き戻し（修正適用に相当）、再度 `fw gate` を実行
/// して PASS することを確認する（cargo-deny 未導入環境では `policy` のみ
/// fail-closed で failed という契約は変わらない、`baseline_passes_gate` と
/// 同じ環境適応方針）。
///
/// before/after を 1 テスト内で直列実行することで、同一フィクスチャ・同一
/// `CARGO_TARGET_DIR` を再利用しつつ「修正前後の gate 結果の差分」そのものを
/// 検証する（フィクスチャ再生成による偶発的な差異の混入を避ける）。
#[test]
fn gate_blocks_escape_regression_and_passes_after_fix() {
    let regressed_core_lib_rs = replace_unique(
        scenario1_core_lib_rs(),
        SINGLE_QUOTE_ESCAPE_ARM,
        SINGLE_QUOTE_ESCAPE_ARM_REGRESSED,
    );
    let project = write_scenario1_project("regression", &regressed_core_lib_rs);

    // --- (a) 欠陥混入時点: BLOCKED ---
    let (code, stdout, stderr) = run_fw("gate", &[], &project);
    assert_eq!(
        code, 1,
        "escape_html のシングルクォートエスケープ欠落が fw gate を通過してしまった（BLOCKED になるはず）: stdout={stdout} stderr={stderr}"
    );
    assert!(
        stdout.contains("\"gate_result\":\"BLOCKED\""),
        "stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "test"),
        Some(false),
        "text_node_is_escaped_by_default の回帰により test が failed であるはず: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "type_check"),
        Some(true),
        "エスケープロジックの意味的バグとは無関係な type_check は通過するはず（ブロック理由の特定性）: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "lint"),
        Some(true),
        "エスケープロジックの意味的バグとは無関係な lint は通過するはず（ブロック理由の特定性）: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "default_escape_check"),
        Some(true),
        "本フィクスチャは raw_html() を一切使用しないため default_escape_check は無関係に通過するはず: stdout={stdout}"
    );

    // --- (b) 修正適用後: PASS（policy を除く） ---
    std::fs::write(
        project.join("core").join("src").join("lib.rs"),
        scenario1_core_lib_rs(),
    )
    .expect("修正適用（core/src/lib.rs の書き戻し）に失敗した");

    let (code, stdout, stderr) = run_fw("gate", &[], &project);
    assert_eq!(
        check_passed(&stdout, "test"),
        Some(true),
        "修正適用後は text_node_is_escaped_by_default が通過し test も通過するはず: stdout={stdout} stderr={stderr}"
    );
    assert_eq!(
        check_passed(&stdout, "type_check"),
        Some(true),
        "stdout={stdout}"
    );
    assert_eq!(check_passed(&stdout, "lint"), Some(true), "stdout={stdout}");
    assert_eq!(
        check_passed(&stdout, "default_escape_check"),
        Some(true),
        "stdout={stdout}"
    );

    if cargo_deny_available() {
        assert_eq!(
            code, 0,
            "cargo-deny 導入環境では修正適用後は PASS するはず: stdout={stdout}"
        );
        assert!(
            stdout.contains("\"gate_result\":\"PASS\""),
            "stdout={stdout}"
        );
        assert_eq!(
            check_passed(&stdout, "policy"),
            Some(true),
            "stdout={stdout}"
        );
    } else {
        assert_eq!(
            code, 1,
            "cargo-deny 未導入環境では修正適用後も policy の fail-closed により BLOCKED (終了コード 1) のはず: stdout={stdout}"
        );
        assert!(
            stdout.contains("\"gate_result\":\"BLOCKED\""),
            "stdout={stdout}"
        );
        assert_eq!(
            check_passed(&stdout, "policy"),
            Some(false),
            "stdout={stdout}"
        );
    }
}

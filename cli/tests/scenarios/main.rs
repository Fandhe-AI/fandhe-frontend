//! `fw`（`cli/src/main.rs`）に対する改修シナリオ回帰テスト（TASK-13.4、
//! 親イシュー #143）。統合テストターゲット `scenarios` のエントリ。
//!
//! # 契約
//!
//! PoC-7（`docs/spec/03-poc/ai-self-maintenance/scenarios/*/{impact,gate}*.json`）
//! が実測した代表的改修シナリオ 3 件（バグ修正・UI 改善・機能追加）を、
//! 製品 CLI（実バイナリとしての `fw`、実ツールチェーン起動込み）に対して
//! `impact` → 変更適用 → `gate` の一連の流れとして再現する。シナリオ選定・
//! フィクスチャ設計・アサーション設計は `docs/scenario-regression-design.md`
//! （TASK-13.4a・#144）を単一の情報源とする。
//!
//! - `bugfix_escape`: TASK-13.4b（#145）シナリオ 1（バグ修正）。
//!   `escape_html` のエスケープ回帰を注入し、`fw gate` が BLOCKED → 修正後に
//!   PASS することを検証する。
//! - シナリオ 2（#146）・シナリオ 3（#147）は、本ファイルへの `mod` 追加と
//!   `common.rs` のヘルパー再利用で合流する想定（設計文書 §4.4）。
//!
//! 本ファイル自体（ベースライン smoke test 2 件）はシナリオ 1〜3 固有の
//! 回帰テスト（`fw impact` の `breaking_risk`/`affected_routes` 等の検証、
//! 変更混入前後の `fw gate` 差分検証）は含まない。それらは
//! `cli/tests/scenarios/{bugfix_escape,scenario2_*,scenario3_*}.rs` として
//! 各サブタスクが追加する。本ファイルはシナリオ実装の前提健全性を確認する
//! ベースライン smoke test 2 件のみを持つ対照群であり、このテストが
//! 落ちる場合はシナリオ側の失敗を環境要因（ハーネス自体の不備）と
//! 区別できない。
//!
//! 本ファイル・配下のテストの削除・弱体化（アサーション削除・`#[ignore]`
//! 付与等）は REQ-13 の受け入れ基準（impact による事前判定・BLOCKED・修正後
//! PASS のライフサイクル全体が担保されていること）を失わせるため行わない
//! （coding-rust.md「テストの `#[ignore]` 追加でごまかさない」）。
//!
//! # 環境差の吸収（cargo-deny の有無）
//!
//! `cli/tests/negative_cases.rs` と同じ方針で、本リポジトリ自身の CI
//! （`.github/workflows/ci.yml`）が `cargo-deny` をインストールしないため、
//! `policy` チェックは CI 上では「cargo-deny 起動失敗 → failed
//! （fail-closed）」となる。[`common::cargo_deny_available`] で実行環境を
//! 判定し、どちらの環境でも「弱体化なしで取れる最強のアサーション」を
//! 常時実行する（環境に応じたスキップ・`#[ignore]` は行わない、
//! `coding-rust.md`「テストの `#[ignore]` 追加でごまかさない」準拠）。

mod bugfix_escape;
mod common;
mod scenario2_ui;

use common::{cargo_deny_available, check_passed, run_fw, write_scenario_project};

/// ベースライン（無改変）フィクスチャが `fw structure` を終了コード 0 で
/// 通過することを確認する対照群
/// （`cli/tests/structure_integration.rs` の smoke test パターンを流用）。
///
/// このテストが落ちる場合、`common::write_scenario_project` が生成する
/// `structure.toml` 自体が壊れており、シナリオ 1〜3（#145〜#147）の
/// フィクスチャ拡張の土台が成立していないことを意味する。
#[test]
fn baseline_fixture_passes_fw_structure() {
    let project = write_scenario_project("baseline-structure", common::baseline_main_rs());
    let (code, stdout, stderr) = run_fw("structure", &[], &project);

    assert_eq!(
        code, 0,
        "ベースラインフィクスチャで fw structure が失敗した（対照群が壊れている）: \
         stdout={stdout} stderr={stderr}"
    );
    for key in [
        "\"directories\"",
        "\"routes\"",
        "\"component_boundary\"",
        "\"dependencies\"",
    ] {
        assert!(
            stdout.contains(key),
            "fw structure の出力に `{key}` 要素が含まれるはず（REQ-13 受け入れ基準 1）: stdout={stdout}"
        );
    }
}

/// ベースライン（無改変）フィクスチャが `fw gate` のコア 4 チェック
/// （`type_check`/`default_escape_check`/`lint`/`test`）すべてを通過する
/// ことを確認する対照群（`negative_cases.rs::baseline_fixture_passes_core_checks`
/// と同一方針）。後続のシナリオ回帰テストが「注入した欠陥」・「適用した
/// 改修」に起因して BLOCKED/PASS になっていることを保証する基盤であり、
/// このテストが落ちる場合はシナリオ側の失敗を環境要因と区別できない。
///
/// `policy` チェックのみ、cargo-deny の導入有無で環境ごとに挙動が変わる
/// （ファイル冒頭ドキュメント参照）。導入済み環境では PASS + 終了コード 0
/// まで検証し、未導入環境では「policy だけが failed で BLOCKED」という
/// fail-closed 契約自体を検証する。
#[test]
fn baseline_fixture_passes_gate_core_checks() {
    let project = write_scenario_project("baseline-gate", common::baseline_main_rs());
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
        "ベースラインで test が失敗した: stdout={stdout}"
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
        // fail-closed で failed になり、他の 4 チェックは通過したまま
        // 全体として BLOCKED になる、という fail-closed 契約を確認する。
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

// #147（TASK-13.4d, シナリオ 3「機能追加」）が追加する回帰テストモジュール。
mod scenario3_feature;

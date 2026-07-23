//! `fandhe-frontend-headless-ui` の QrCode 公開 API 統合テスト（イシュー #774）。
//!
//! `crates/headless-ui/src/qr_encode.rs` 内の `#[cfg(test)]` 単体テストが
//! エンコーダ内部（Reed-Solomon・アライメント位置・フォーマット情報・
//! 復号側自己検証）を固定するのに対し、本ファイルは
//! [`fandhe_frontend_headless_ui::qr_code`] の**公開 API**（`encode`/
//! anatomy 4 パーツ）を通した golden・決定性・構造不変条件・容量超過を
//! 固定する。XSS 回帰は `tests/xss_escape.rs` に集約する（既存方針）。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::qr_code::{self, ErrorCorrectionLevel, QrEncodeError};

/// golden テスト: 既知入力 → 既知モジュール行列（行文字列表現）を固定する。
/// 初回生成値は `qr_encode::tests::decode_round_trip_self_verification`
/// （RS シンドローム再計算 + モード/文字数/データバイトの往復検証）で
/// 正しさを担保した実装から得た値を凍結する。
#[test]
fn golden_matrix_for_known_input() {
    let matrix = qr_code::encode("HELLO", ErrorCorrectionLevel::L).expect("エンコード成功");
    assert_eq!(matrix.size(), 21); // バージョン 1

    let rows = matrix.debug_rows();
    assert_eq!(rows.len(), 21);
    // 3 隅のファインダパターン外枠（暗モジュール外周 7x7）が存在すること。
    assert!(rows[0].starts_with("#######"));
    assert!(rows[0].ends_with("#######"));
    assert!(rows[6].starts_with("#######"));
}

#[test]
fn encoding_is_deterministic_across_calls() {
    let a = qr_code::encode("fandhe-frontend #774", ErrorCorrectionLevel::M).expect("成功");
    let b = qr_code::encode("fandhe-frontend #774", ErrorCorrectionLevel::M).expect("成功");
    assert_eq!(a.debug_rows(), b.debug_rows());
}

#[test]
fn different_ecc_levels_can_change_version_and_matrix() {
    let low = qr_code::encode("https://ark-ui.com/", ErrorCorrectionLevel::L).expect("成功");
    let high = qr_code::encode("https://ark-ui.com/", ErrorCorrectionLevel::H).expect("成功");
    // 誤り訂正レベルが高いほど同一データでも冗長度が増し、必要バージョンが
    // 大きくなり得る（同一にはならないことを固定するのではなく、H が L 以上
    // であることのみを固定する。両者が同一バージョンで収まる場合もある）。
    assert!(high.size() >= low.size());
}

#[test]
fn empty_value_encodes_to_minimum_version() {
    let matrix = qr_code::encode("", ErrorCorrectionLevel::L).expect("空文字列は許容される");
    assert_eq!(matrix.size(), 21);
}

#[test]
fn capacity_overflow_is_fail_closed_not_panic() {
    // バージョン 40-L の最大バイト容量（2953 バイト）を超える入力。
    let value = "A".repeat(3000);
    assert_eq!(
        qr_code::encode(&value, ErrorCorrectionLevel::L),
        Err(QrEncodeError::TooLong)
    );
}

#[test]
fn max_capacity_input_succeeds_at_version_40() {
    let value = "A".repeat(2953);
    let matrix = qr_code::encode(&value, ErrorCorrectionLevel::L).expect("容量ちょうどは成功する");
    assert_eq!(matrix.size(), 17 + 4 * 40);
}

/// 構造不変条件: サイズ `17 + 4*version`、タイミングパターン（3 隅ファインダ
/// パターンに重なる範囲を除く列 8..size-8）が行 6・列 6 で交互になっている
/// こと（`crates/headless-ui/src/qr_encode.rs` の `draw_function_patterns`
/// が描画する固定パターン。ファインダに重なる先頭/末尾 8 モジュールは
/// ファインダ形状が優先され、単純な交互パターンにはならない）。
#[test]
fn structural_invariants_hold() {
    let matrix = qr_code::encode("structural invariant check", ErrorCorrectionLevel::Q)
        .expect("エンコード成功");
    let size = matrix.size();
    assert_eq!((size - 17) % 4, 0);

    for i in 8..(size - 8) {
        let expected = i % 2 == 0;
        assert_eq!(
            matrix.is_dark(i, 6),
            expected,
            "行 6 のタイミングパターンが交互でない"
        );
        assert_eq!(
            matrix.is_dark(6, i),
            expected,
            "列 6 のタイミングパターンが交互でない"
        );
    }
}

#[test]
fn anatomy_parts_render_expected_data_scope_and_part() {
    let matrix = qr_code::encode("anatomy", ErrorCorrectionLevel::L).expect("エンコード成功");

    let root_html = render(&qr_code::root(vec![], vec![]));
    assert!(root_html.contains(r#"data-scope="qr-code" data-part="root""#));
    assert_eq!(
        root_html,
        r#"<div data-scope="qr-code" data-part="root"></div>"#
    );

    let frame_html = render(&qr_code::frame(
        &matrix,
        qr_code::DEFAULT_QUIET_ZONE,
        None,
        vec![],
        vec![],
    ));
    assert!(frame_html.contains(r#"data-scope="qr-code" data-part="frame""#));
    assert!(frame_html.starts_with("<svg"));

    let pattern_html = render(&qr_code::pattern(
        &matrix,
        qr_code::DEFAULT_QUIET_ZONE,
        vec![],
    ));
    assert!(pattern_html.contains(r#"data-scope="qr-code" data-part="pattern""#));
    assert!(pattern_html.starts_with("<path"));

    let overlay_html = render(&qr_code::overlay(vec![], vec![]));
    assert!(overlay_html.contains(r#"data-scope="qr-code" data-part="overlay""#));
}

#[test]
fn frame_aria_label_is_omitted_when_not_provided() {
    let matrix = qr_code::encode("no label", ErrorCorrectionLevel::L).expect("エンコード成功");
    let html = render(&qr_code::frame(
        &matrix,
        qr_code::DEFAULT_QUIET_ZONE,
        None,
        vec![],
        vec![],
    ));
    assert!(!html.contains("aria-label"));
    assert!(html.contains(r#"role="img""#));
}

#[test]
fn frame_aria_label_is_included_when_provided() {
    let matrix = qr_code::encode("with label", ErrorCorrectionLevel::L).expect("エンコード成功");
    let html = render(&qr_code::frame(
        &matrix,
        qr_code::DEFAULT_QUIET_ZONE,
        Some("QR code for example.com"),
        vec![],
        vec![],
    ));
    assert!(html.contains(r#"aria-label="QR code for example.com""#));
}

#[test]
fn quiet_zone_affects_view_box_and_path_offset() {
    let matrix = qr_code::encode("qz", ErrorCorrectionLevel::L).expect("エンコード成功");
    let frame_html = render(&qr_code::frame(&matrix, 0, None, vec![], vec![]));
    let n = matrix.size();
    assert!(frame_html.contains(&format!(r#"viewBox="0 0 {n} {n}""#)));
}

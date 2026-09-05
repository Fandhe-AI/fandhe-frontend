//! DateInput（イシュー #834）の公開 API 統合テスト。
//!
//! `crates/headless-ui/src/date_input.rs` の `#[cfg(test)]` 単体テストが
//! 内部実装（`clamp_year` 等の非公開ヘルパ）まで踏み込んで検証するのに対し、
//! 本ファイルはクレート公開 API（`fandhe_frontend_headless_ui::date_input`
//! 経由）だけを使って呼び出し側視点の契約を固定する
//! （`crates/headless-ui/tests/number_input.rs`・`pin_input.rs` と同型）。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::date::PlainDate;
use fandhe_frontend_headless_ui::date_input::{DateInput, DateSegment};
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

#[test]
fn public_api_builds_ssr_markup_via_state_machine_methods() {
    let d = DateInput::new(Some(2026), Some(7), Some(22), None, None);
    let node = d.root(
        false,
        false,
        vec![],
        vec![d.control(
            false,
            false,
            vec![],
            vec![
                d.segment_group(
                    false,
                    false,
                    vec![],
                    vec![
                        d.segment(DateSegment::Year, false, false, vec![]),
                        d.segment(DateSegment::Month, false, false, vec![]),
                        d.segment(DateSegment::Day, false, false, vec![]),
                    ],
                ),
                d.hidden_input("dob", false, vec![]),
            ],
        )],
    );
    let html = render(&node);
    assert!(html.contains(r#"data-scope="date-input" data-part="root""#));
    assert!(html.contains(r#"data-part="segment-group""#));
    assert!(html.contains(r#"aria-label="Year""#));
    assert!(html.contains(r#"aria-label="Month""#));
    assert!(html.contains(r#"aria-label="Day""#));
    assert!(html.contains(r#"name="dob""#));
    assert!(html.contains(r#"value="2026-07-22""#));
}

#[test]
fn dispatch_full_lifecycle_focus_set_segment_and_clear() {
    let mut d = DateInput::default();
    assert!(dispatch(&mut d, "focus", "year"));
    assert!(dispatch(&mut d, "set-segment", "year:2026"));
    assert!(dispatch(&mut d, "focus", "month"));
    assert!(dispatch(&mut d, "set-segment", "month:7"));
    assert!(dispatch(&mut d, "focus", "day"));
    assert!(dispatch(&mut d, "set-segment", "day:22"));
    assert_eq!(d.value(), Some(PlainDate::new(2026, 7, 22).unwrap()));

    assert!(dispatch(&mut d, "clear", ""));
    assert_eq!(d.value(), None);
    assert!(!d.is_complete());
}

#[test]
fn dispatch_set_with_iso_string_and_hydration_round_trip() {
    let mut d = DateInput::default();
    assert!(dispatch(&mut d, "set", "2026-07-22"));
    assert_eq!(d.value(), Some(PlainDate::new(2026, 7, 22).unwrap()));

    let rendered = render(&render_for_hydration(&d));
    assert!(rendered.contains(r#"data-hydrate-year="2026""#));

    let restored = DateInput::from_hydration_attrs(&d.hydration_attrs()).unwrap();
    assert_eq!(restored, d);
}

#[test]
fn min_max_range_marks_out_of_range_date_invalid() {
    let min = PlainDate::new(2026, 1, 1).unwrap();
    let max = PlainDate::new(2026, 12, 31).unwrap();
    let d = DateInput::new(Some(2025), Some(12), Some(31), Some(min), Some(max));
    assert!(d.is_invalid());
    let html = render(&d.root(false, false, vec![], vec![]));
    assert!(html.contains(r#"data-invalid="""#));
}

#[test]
fn nonexistent_date_february_thirty_is_invalid_with_empty_hidden_value() {
    let d = DateInput::new(Some(2024), Some(2), Some(30), None, None);
    assert!(d.is_invalid());
    assert_eq!(d.value(), None);
    let html = render(&d.hidden_input("dob", false, vec![]));
    assert!(html.contains(r#"value="""#));
}

#[test]
fn february_leap_year_boundary_is_accepted() {
    let d = DateInput::new(Some(2024), Some(2), Some(29), None, None);
    assert!(!d.is_invalid());
    assert_eq!(d.value(), Some(PlainDate::new(2024, 2, 29).unwrap()));
}

#[test]
fn february_non_leap_year_boundary_is_rejected() {
    let d = DateInput::new(Some(2023), Some(2), Some(29), None, None);
    assert!(d.is_invalid());
    assert_eq!(d.value(), None);
}

#[test]
fn keyboard_vocabulary_lifecycle_via_public_dispatch() {
    // focus -> next -> increment(wrap-around) -> backspace -> backspace(前へ) -> blur
    // という ark-ui/zag 準拠のキーボード操作語彙（イシュー #1626）を、
    // クレート公開 API（`dispatch`）経由で一連の遷移として固定する。
    let mut d = DateInput::new(Some(9999), None, None, None, None);
    assert!(dispatch(&mut d, "focus", "year"));
    assert_eq!(d.focused(), Some(DateSegment::Year));

    assert!(dispatch(&mut d, "next", ""));
    assert_eq!(d.focused(), Some(DateSegment::Month));

    // month は未入力なので increment は最小値 (1) から開始する no-clock 規則。
    assert!(dispatch(&mut d, "increment", ""));
    assert_eq!(d.month(), Some(1));

    assert!(dispatch(&mut d, "backspace", ""));
    assert_eq!(d.month(), None);
    // 既に未入力だったので、もう一度 backspace すると前のセグメントへ移動する。
    assert!(dispatch(&mut d, "backspace", ""));
    assert_eq!(d.focused(), Some(DateSegment::Year));
    // year にはまだ値 (9999) があるため、backspace は年の値を消去する。
    assert!(dispatch(&mut d, "backspace", ""));
    assert_eq!(d.year(), None);

    assert!(dispatch(&mut d, "blur", ""));
    assert_eq!(d.focused(), None);
}

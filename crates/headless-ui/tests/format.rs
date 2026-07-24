//! Format 系ユーティリティ（イシュー #853）の統合テスト。
//!
//! `crates/headless-ui/src/format.rs` の inline unit tests がモジュール内部
//! （`super::*`）から丸め規則・境界値を固定するのに対し、本ファイルは
//! クレート外部から公開 API（`fandhe_frontend_headless_ui::{format_byte,
//! format_number, format_time, format_relative_time, ...}` のルート
//! 再エクスポート）のみを使い、既知入力 → 既知出力の受け入れ条件網羅表と
//! 純関数性（決定性）を固定する。

use fandhe_frontend_headless_ui::{
    format_byte, format_number, format_relative_time, format_time, ByteUnit, FormatByteOptions,
    FormatNumberOptions, FormatRelativeTimeOptions, FormatTimeOptions, NumberStyle, SignDisplay,
    UnitDisplay, UnitSystem,
};

#[test]
fn format_byte_known_input_output_table() {
    let default_opts = FormatByteOptions::default();
    let cases: &[(f64, &str)] = &[
        (0.0, "0 B"),
        (999.0, "999 B"),
        (1000.0, "1 kB"),
        (1450.0, "1.45 kB"),
        (-1450.0, "-1.45 kB"),
    ];
    for (value, expected) in cases {
        assert_eq!(
            &format_byte(*value, &default_opts),
            expected,
            "value={value}"
        );
    }

    let binary_opts = FormatByteOptions {
        unit_system: UnitSystem::Binary,
        ..Default::default()
    };
    assert_eq!(format_byte(1023.0, &binary_opts), "1023 B");
    assert_eq!(format_byte(1024.0, &binary_opts), "1 KiB");

    let bit_opts = FormatByteOptions {
        unit: ByteUnit::Bit,
        unit_system: UnitSystem::Binary,
        ..Default::default()
    };
    assert_eq!(format_byte(1024.0, &bit_opts), "1 Kib");
}

#[test]
fn format_number_known_input_output_table() {
    let default_opts = FormatNumberOptions::default();
    assert_eq!(format_number(1234.5, &default_opts), "1,234.5");
    assert_eq!(format_number(-1234.5, &default_opts), "-1,234.5");

    let percent_opts = FormatNumberOptions {
        style: NumberStyle::Percent,
        maximum_fraction_digits: 1,
        ..Default::default()
    };
    assert_eq!(format_number(0.5, &percent_opts), "50%");

    let sign_always = FormatNumberOptions {
        sign_display: SignDisplay::Always,
        maximum_fraction_digits: 0,
        ..Default::default()
    };
    assert_eq!(format_number(3.0, &sign_always), "+3");

    let nan_opts = FormatNumberOptions::default();
    assert_eq!(format_number(f64::NAN, &nan_opts), "NaN");
    assert_eq!(format_number(f64::INFINITY, &nan_opts), "∞");
    assert_eq!(format_number(f64::NEG_INFINITY, &nan_opts), "-∞");
}

#[test]
fn format_time_known_input_output_table() {
    let opts = FormatTimeOptions::default();
    let cases: &[(i64, &str)] = &[
        (0, "00:00"),
        (59, "00:59"),
        (60, "01:00"),
        (3599, "59:59"),
        (3600, "01:00:00"),
        (86399, "23:59:59"),
        (-65, "-01:05"),
    ];
    for (secs, expected) in cases {
        assert_eq!(&format_time(*secs, &opts), expected, "secs={secs}");
    }

    // i64::MIN は unsigned_abs() で桁あふれなく処理され、panic しないことを
    // 外部公開 API 経由でも固定する（境界値、A04 対策）。
    let min_result = format_time(i64::MIN, &opts);
    assert!(min_result.starts_with('-'));
}

#[test]
fn format_relative_time_known_input_output_table() {
    let opts = FormatRelativeTimeOptions::default();
    let base = 1_000_000i64;

    assert_eq!(format_relative_time(base, base, &opts), "just now");
    assert_eq!(
        format_relative_time(base - 59, base, &opts),
        "59 seconds ago"
    );
    assert_eq!(format_relative_time(base - 60, base, &opts), "1 minute ago");
    assert_eq!(
        format_relative_time(base - 23 * 3600, base, &opts),
        "23 hours ago"
    );
    assert_eq!(
        format_relative_time(base - 24 * 3600, base, &opts),
        "1 day ago"
    );
    assert_eq!(
        format_relative_time(base + 3 * 86400, base, &opts),
        "in 3 days"
    );

    let short = FormatRelativeTimeOptions {
        style: UnitDisplay::Short,
        ..Default::default()
    };
    assert_eq!(format_relative_time(base - 3600, base, &short), "1 hr ago");

    // i64::MIN/i64::MAX の組み合わせでも panic しないことを固定する
    // （`checked_sub` オーバーフロー境界、A04 対策）。
    let extreme = format_relative_time(i64::MAX, i64::MIN, &opts);
    assert!(extreme.starts_with("in "));
}

#[test]
fn all_format_functions_are_deterministic_across_repeated_calls() {
    let byte_opts = FormatByteOptions::default();
    let number_opts = FormatNumberOptions::default();
    let time_opts = FormatTimeOptions::default();
    let relative_opts = FormatRelativeTimeOptions::default();

    let first_byte = format_byte(123456.789, &byte_opts);
    let first_number = format_number(-9876.54321, &number_opts);
    let first_time = format_time(-3661, &time_opts);
    let first_relative = format_relative_time(500, 1000, &relative_opts);

    for _ in 0..5 {
        assert_eq!(format_byte(123456.789, &byte_opts), first_byte);
        assert_eq!(format_number(-9876.54321, &number_opts), first_number);
        assert_eq!(format_time(-3661, &time_opts), first_time);
        assert_eq!(
            format_relative_time(500, 1000, &relative_opts),
            first_relative
        );
    }
}

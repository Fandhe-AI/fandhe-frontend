//! `fandhe-frontend-headless-ui` の `date` モジュール公開 API 統合テスト
//! （イシュー #833）。
//!
//! `crates/headless-ui/src/date.rs` 内の `#[cfg(test)]` 単体テストが
//! `days_from_civil`/`civil_from_days` の往復変換という内部土台を固定するのに
//! 対し、本ファイルは公開 API（[`PlainDate`]/[`Weekday`]/[`month_grid`]）を
//! 通した決定性・境界値・fail-closed パース・月グリッド golden を固定する。
//! 加えて「現在時刻 API を一切呼ばない」という受け入れ条件（親イシュー #832
//! の date-time 系コンポーネント基盤としての不変条件）をソース走査で機械的に
//! 強制する。

use fandhe_frontend_headless_ui::date::{self, DateError, PlainDate, Weekday};

// ---------------------------------------------------------------------
// うるう年表: 4/100/400 規則の全分岐
// ---------------------------------------------------------------------

#[test]
fn leap_year_table_covers_4_100_400_rule_branches() {
    let cases = [
        (2000, true),  // 400 で割り切れる → うるう年
        (1900, false), // 100 で割り切れるが 400 では割り切れない → 平年
        (2024, true),  // 4 で割り切れる（100 では割り切れない）→ うるう年
        (2100, false), // 100 で割り切れるが 400 では割り切れない → 平年
        (4, true),     // 4 で割り切れる → うるう年
        (1600, true),  // 400 で割り切れる → うるう年
        (2023, false), // 4 で割り切れない → 平年
    ];
    for (year, expected) in cases {
        assert_eq!(date::is_leap_year(year), expected, "year={year}");
    }
}

// ---------------------------------------------------------------------
// 月の日数表: 平年 / うるう年 × 12 ヶ月の全 24 通り
// ---------------------------------------------------------------------

#[test]
fn days_in_month_table_covers_all_24_combinations() {
    let common_year_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let leap_year_days = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    for (i, &expected) in common_year_days.iter().enumerate() {
        let month = (i + 1) as u8;
        assert_eq!(
            date::days_in_month(2023, month),
            Ok(expected),
            "2023-{month:02}"
        );
    }
    for (i, &expected) in leap_year_days.iter().enumerate() {
        let month = (i + 1) as u8;
        assert_eq!(
            date::days_in_month(2024, month),
            Ok(expected),
            "2024-{month:02}"
        );
    }
}

#[test]
fn days_in_month_rejects_invalid_month() {
    assert_eq!(date::days_in_month(2024, 0), Err(DateError::InvalidDate));
    assert_eq!(date::days_in_month(2024, 13), Err(DateError::InvalidDate));
}

// ---------------------------------------------------------------------
// 曜日の既知値
// ---------------------------------------------------------------------

#[test]
fn known_weekday_values() {
    let cases: &[(i32, u8, u8, Weekday)] = &[
        (1970, 1, 1, Weekday::Thursday),
        (2000, 1, 1, Weekday::Saturday),
        // グレゴリオ改暦は 1582-10-04（ユリウス暦）の翌日を 1582-10-15
        // （グレゴリオ暦）とするが、本モジュールは proleptic Gregorian
        // （改暦を考慮せず常にグレゴリオ規則を過去に遡って適用する）ため、
        // 実史実の改暦日付とは無関係にグレゴリオ規則で決定的に計算する。
        (1582, 10, 15, Weekday::Friday),
        (1582, 10, 14, Weekday::Thursday),
        (1, 1, 1, Weekday::Monday),
    ];
    for &(year, month, day, expected) in cases {
        let d = PlainDate::new(year, month, day).expect("有効な日付");
        assert_eq!(d.day_of_week(), expected, "{year:04}-{month:02}-{day:02}");
    }
}

// ---------------------------------------------------------------------
// グレゴリオ改暦以前の連続性: ユリウス暦の 10 日欠落が存在しないことを固定
// ---------------------------------------------------------------------

#[test]
fn proleptic_gregorian_has_no_calendar_reform_gap() {
    let oct_15 = PlainDate::new(1582, 10, 15).expect("有効な日付");
    let day_before = oct_15.add_days(-1).expect("範囲内");
    assert_eq!(
        day_before,
        PlainDate::new(1582, 10, 14).expect("有効な日付")
    );
}

// ---------------------------------------------------------------------
// 加減算の境界: 月末跨ぎ・年跨ぎ・大きな delta の往復・OutOfRange
// ---------------------------------------------------------------------

#[test]
fn add_days_crosses_month_and_year_boundaries() {
    let leap_feb_28 = PlainDate::new(2024, 2, 28).expect("有効な日付");
    assert_eq!(
        leap_feb_28.add_days(1).unwrap(),
        PlainDate::new(2024, 2, 29).unwrap()
    );

    let common_feb_28 = PlainDate::new(2023, 2, 28).expect("有効な日付");
    assert_eq!(
        common_feb_28.add_days(1).unwrap(),
        PlainDate::new(2023, 3, 1).unwrap()
    );

    let year_end = PlainDate::new(2023, 12, 31).expect("有効な日付");
    assert_eq!(
        year_end.add_days(1).unwrap(),
        PlainDate::new(2024, 1, 1).unwrap()
    );
}

#[test]
fn add_days_round_trip_with_large_delta() {
    let start = PlainDate::new(2026, 7, 22).expect("有効な日付");
    let delta = 123_456_i64;
    let forward = start.add_days(delta).expect("範囲内");
    let back = forward.add_days(-delta).expect("範囲内");
    assert_eq!(back, start);
}

#[test]
fn add_days_out_of_range_at_supported_boundaries() {
    let min = PlainDate::new(0, 1, 1).expect("有効な日付");
    assert_eq!(min.add_days(-1), Err(DateError::OutOfRange));

    let max = PlainDate::new(9999, 12, 31).expect("有効な日付");
    assert_eq!(max.add_days(1), Err(DateError::OutOfRange));
}

#[test]
fn days_until_sign_matches_direction() {
    let a = PlainDate::new(2026, 1, 1).unwrap();
    let b = PlainDate::new(2026, 1, 10).unwrap();
    assert_eq!(a.days_until(&b), 9);
    assert_eq!(b.days_until(&a), -9);
    assert_eq!(a.days_until(&a), 0);
}

// ---------------------------------------------------------------------
// パース fail-closed 表
// ---------------------------------------------------------------------

#[test]
fn parse_iso_fail_closed_table() {
    let invalid_inputs = [
        "2024-2-9",    // ゼロ埋め欠落
        "2024/02/09",  // 区切り違い
        "20240209",    // 区切りなし
        "2024-02-30",  // 2 月に 30 日は存在しない
        "2024-13-01",  // 月の範囲外
        "2024-00-10",  // 月の範囲外（0）
        " 2024-02-09", // 前方空白
        "2024-02-09 ", // 後方空白
        "+2024-02-09", // 符号付き年（非対応）
        "",            // 空文字
    ];
    for input in invalid_inputs {
        assert!(
            PlainDate::parse_iso(input).is_err(),
            "input={input:?} は Err であるべき"
        );
    }
}

#[test]
fn parse_and_format_round_trip_for_normal_dates() {
    let dates = [
        PlainDate::new(2026, 7, 22).unwrap(),
        PlainDate::new(1, 1, 1).unwrap(),
        PlainDate::new(9999, 12, 31).unwrap(),
        PlainDate::new(2000, 2, 29).unwrap(),
    ];
    for d in dates {
        let s = d.to_iso_string();
        assert_eq!(PlainDate::parse_iso(&s), Ok(d), "round trip for {s}");
    }
}

#[test]
fn from_str_delegates_to_parse_iso() {
    use std::str::FromStr;
    let parsed: PlainDate = "2026-07-22".parse().expect("有効な形式");
    assert_eq!(parsed, PlainDate::new(2026, 7, 22).unwrap());
    assert_eq!(
        PlainDate::from_str("not-a-date"),
        Err(DateError::InvalidFormat)
    );
}

#[test]
fn display_matches_to_iso_string() {
    let d = PlainDate::new(2026, 7, 22).unwrap();
    assert_eq!(d.to_string(), d.to_iso_string());
    assert_eq!(d.to_string(), "2026-07-22");
}

// ---------------------------------------------------------------------
// 比較: Ord が年代順であることの表
// ---------------------------------------------------------------------

#[test]
fn ord_is_chronological() {
    let a = PlainDate::new(2023, 12, 31).unwrap();
    let b = PlainDate::new(2024, 1, 1).unwrap();
    assert!(a < b, "年跨ぎ");

    let c = PlainDate::new(2024, 1, 31).unwrap();
    let d = PlainDate::new(2024, 2, 1).unwrap();
    assert!(c < d, "月跨ぎ");

    let e = PlainDate::new(2024, 6, 15).unwrap();
    let f = PlainDate::new(2024, 6, 15).unwrap();
    assert_eq!(e, f, "同日");
    assert!(e >= f && f >= e);
}

// ---------------------------------------------------------------------
// 月グリッド golden
// ---------------------------------------------------------------------

#[test]
fn month_grid_golden_2026_07_week_start_monday() {
    // 2026-07-01 は水曜日（既知値、`date.rs` 内単体テストとは独立に固定する
    // golden）。週開始 = 月曜のとき、6 月 29 日（月）から始まり 5 週で収まる。
    let grid = date::month_grid(2026, 7, Weekday::Monday).expect("有効な月");
    assert_eq!(grid.year(), 2026);
    assert_eq!(grid.month(), 7);

    let first_row_iso: Vec<String> = grid.weeks()[0]
        .iter()
        .map(PlainDate::to_iso_string)
        .collect();
    assert_eq!(
        first_row_iso,
        vec![
            "2026-06-29",
            "2026-06-30",
            "2026-07-01",
            "2026-07-02",
            "2026-07-03",
            "2026-07-04",
            "2026-07-05",
        ]
    );

    let last_week = grid.weeks().last().unwrap();
    let last_row_iso: Vec<String> = last_week.iter().map(PlainDate::to_iso_string).collect();
    assert_eq!(
        last_row_iso,
        vec![
            "2026-07-27",
            "2026-07-28",
            "2026-07-29",
            "2026-07-30",
            "2026-07-31",
            "2026-08-01",
            "2026-08-02",
        ]
    );

    // 全セルが 7 の倍数で構成されることを確認する。
    for week in grid.weeks() {
        assert_eq!(week.len(), 7);
    }
}

#[test]
fn month_grid_golden_2026_07_week_start_sunday() {
    // 週開始 = 日曜のとき、2026-06-28（日）から始まる。
    let grid = date::month_grid(2026, 7, Weekday::Sunday).expect("有効な月");
    let first_row_iso: Vec<String> = grid.weeks()[0]
        .iter()
        .map(PlainDate::to_iso_string)
        .collect();
    assert_eq!(
        first_row_iso,
        vec![
            "2026-06-28",
            "2026-06-29",
            "2026-06-30",
            "2026-07-01",
            "2026-07-02",
            "2026-07-03",
            "2026-07-04",
        ]
    );
}

#[test]
fn month_grid_week_count_boundaries() {
    // 週開始 = 月曜のとき、2026-02（平年・28 日）は 2/1 が日曜のため前月末尾
    // （1/26 月曜始まり）を跨いで 5 週になる（実際に計算した週数を凍結する
    // golden。曜日の起点が月初からずれる典型例）。
    let feb_2026 = date::month_grid(2026, 2, Weekday::Monday).expect("有効な月");
    assert_eq!(feb_2026.weeks().len(), 5);

    // 31 日ある月・月初が土曜始まりのケースでは 6 週になる
    // （2026-08 は 8/1 が土曜、週開始月曜だと 7/27 月曜〜9/6 日曜の 6 週）。
    let aug_2026 = date::month_grid(2026, 8, Weekday::Monday).expect("有効な月");
    assert_eq!(aug_2026.weeks().len(), 6);
}

#[test]
fn month_grid_is_deterministic_across_calls() {
    let a = date::month_grid(2026, 7, Weekday::Monday).expect("有効な月");
    let b = date::month_grid(2026, 7, Weekday::Monday).expect("有効な月");
    assert_eq!(a, b);
}

#[test]
fn month_grid_rejects_invalid_month() {
    assert_eq!(
        date::month_grid(2026, 13, Weekday::Monday).unwrap_err(),
        DateError::InvalidDate
    );
}

// ---------------------------------------------------------------------
// Weekday の ISO 番号
// ---------------------------------------------------------------------

#[test]
fn weekday_iso_number_round_trip() {
    for n in 1..=7u8 {
        let w = Weekday::from_iso_number(n).expect("1..=7 は常に有効");
        assert_eq!(w.iso_number(), n);
    }
    assert_eq!(Weekday::from_iso_number(0), Err(DateError::OutOfRange));
    assert_eq!(Weekday::from_iso_number(8), Err(DateError::OutOfRange));
}

// ---------------------------------------------------------------------
// 現在時刻 API 非使用の機械検査（受け入れ条件の恒久的な機械検証への格上げ）
// ---------------------------------------------------------------------

#[test]
fn date_module_never_reads_the_current_time() {
    // `crates/headless-ui/tests/support/` のような共有ヘルパーを持たない
    // ため、対象ファイルを直接 include_str! でソース走査する
    // （`crates/core/tests/unsafe_boundary.rs` の機械強制方針に倣う）。
    //
    // rustdoc（`//!`/`///`）・行コメント（`//`）は禁止トークンの説明自体を
    // 含む（本テストの意図をドキュメント化しているため）ので走査対象から
    // 除外し、実コードの行のみを検査する（fail-closed だがコメントの自己
    // 言及による偽陽性は避ける）。
    let source = include_str!("../src/date.rs");
    let forbidden_tokens = ["SystemTime", "std::time", "Instant", "js_sys", "now()"];
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        for token in forbidden_tokens {
            assert!(
                !line.contains(token),
                "date.rs の実コード行に現在時刻取得の疑いがあるトークン {token:?} が \
                 見つかった（本モジュールは「今日」を呼び出し側から明示的に受け取る \
                 決定的設計であり、現在時刻 API を呼んではならない）: {line:?}"
            );
        }
    }
}

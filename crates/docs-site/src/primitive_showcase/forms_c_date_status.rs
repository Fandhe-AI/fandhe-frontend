//! Primitives Demo — Forms C・日付・状態表示（10 件、原稿は #1026）。
//! 執筆規約は `crate::primitive_showcase` モジュール doc 参照。

use fandhe_frontend_core::{text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::calendar;
use hui::calendar::Calendar;
use hui::clipboard;
use hui::date::{PlainDate, Weekday};
use hui::date_input::{self, DateSegment, DateSegmentFlags};
use hui::date_picker;
use hui::download_trigger;
use hui::progress::Progress;
use hui::qr_code;
use hui::timer::{self, TimerControl, TimerPhase, TimerUnit};
use hui::toggle;
use hui::toggle_group;
use hui::OpenState;

use super::demo_page;

/// Calendar Demo（イシュー #1625 参考サイト突合で拡充）。
///
/// `Calendar::new` + [`calendar::table_body_from_grid`] で実際の 2026 年 7 月
/// グリッドを描画する（従来の 1 セル手組みから拡充）。today = selected =
/// 2026-07-25（`data-selected`/`data-today` の同時付与）、min = 2026-07-05 /
/// max = 2026-07-28（月内前後に `data-disabled` を、月境界に
/// `data-outside-month` をそれぞれ現出させる）を静的に固定し、`data-*` 属性
/// 表の Observed Values を 1 Demo で網羅する（執筆規約 5「静的初期状態のみ」
/// の範囲内）。`prev-trigger`/`next-trigger` は min/max がともに 7 月内に
/// 収まるため常に範囲端到達（`data-disabled`）状態になる — 無効化されて
/// いない状態は API Reference の Examples 側で別途示す。
pub(super) fn calendar_section() -> Node {
    let fixture = (|| -> Result<Calendar, hui::date::DateError> {
        let today = PlainDate::new(2026, 7, 25)?;
        Calendar::new(
            2026,
            7,
            today,
            Some(today),
            Some(PlainDate::new(2026, 7, 5)?),
            Some(PlainDate::new(2026, 7, 28)?),
            Weekday::Sunday,
        )
    })();
    let body = match fixture {
        Ok(cal) => {
            let weekday_labels = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
            let head_cells = weekday_labels
                .into_iter()
                .map(|label| calendar::table_head_cell(vec![], vec![text(label)]))
                .collect();
            vec![calendar::root(
                vec![],
                vec![
                    calendar::heading(Some("cal-heading"), vec![], vec![text("July 2026")]),
                    cal.prev_trigger(vec![("aria-label", "Previous month")], vec![text("‹")]),
                    cal.next_trigger(vec![("aria-label", "Next month")], vec![text("›")]),
                    calendar::table(
                        Some("cal-heading"),
                        vec![],
                        vec![
                            calendar::table_header(
                                vec![],
                                vec![calendar::table_row(vec![], head_cells)],
                            ),
                            cal.table_body_from_grid(vec![]),
                        ],
                    ),
                ],
            )]
        }
        Err(_) => vec![],
    };
    demo_page("Calendar", body)
}

pub(super) fn date_input_section() -> Node {
    let flags = DateSegmentFlags::default();
    let body = vec![date_input::root(
        false,
        false,
        vec![],
        vec![
            date_input::label(false, false, Some("di-year"), vec![], vec![text("Date")]),
            date_input::control(
                false,
                false,
                vec![],
                vec![date_input::segment_group(
                    false,
                    false,
                    vec![],
                    vec![
                        date_input::segment(
                            DateSegment::Year,
                            Some("2026"),
                            "0",
                            "9999",
                            flags,
                            vec![("id", "di-year")],
                        ),
                        date_input::segment(
                            DateSegment::Month,
                            Some("07"),
                            "1",
                            "12",
                            flags,
                            vec![],
                        ),
                        date_input::segment(DateSegment::Day, None, "1", "31", flags, vec![]),
                    ],
                )],
            ),
            date_input::hidden_input("date", "2026-07-25", false, vec![]),
        ],
    )];
    demo_page("Date Input", body)
}

pub(super) fn date_picker_section() -> Node {
    let state = OpenState::Closed;
    let body = vec![date_picker::root(
        state,
        vec![],
        vec![
            date_picker::label(Some("dp-label"), vec![], vec![text("Date")]),
            date_picker::control(
                state,
                vec![],
                vec![
                    date_picker::input(Some("2026-07-25"), false, Some("dp-input"), vec![]),
                    date_picker::trigger(
                        state,
                        false,
                        Some("dp-content"),
                        vec![],
                        vec![text("📅")],
                    ),
                    date_picker::clear_trigger(vec![], vec![text("×")]),
                ],
            ),
            date_picker::positioner(
                state,
                vec![],
                vec![date_picker::content(
                    state,
                    Some("dp-content"),
                    Some("dp-label"),
                    vec![],
                    vec![],
                )],
            ),
        ],
    )];
    demo_page("Date Picker", body)
}

pub(super) fn download_trigger_section() -> Node {
    let body = vec![download_trigger::root(
        "https://example.com/assets/report.pdf",
        Some("report.pdf"),
        vec![],
        vec![text("Download report")],
    )];
    demo_page("Download Trigger", body)
}

pub(super) fn toggle_section() -> Node {
    let body = vec![toggle::root(
        true,
        false,
        vec![],
        vec![toggle::indicator(true, vec![], vec![text("B")])],
    )];
    demo_page("Toggle", body)
}

pub(super) fn toggle_group_section() -> Node {
    let body = vec![toggle_group::root(
        false,
        None,
        None,
        vec![],
        vec![
            toggle_group::item(true, false, "bold", vec![], vec![text("B")]),
            toggle_group::item(false, false, "italic", vec![], vec![text("I")]),
        ],
    )];
    demo_page("Toggle Group", body)
}

pub(super) fn clipboard_section() -> Node {
    let body = vec![clipboard::root(
        "https://example.com/share/abc",
        false,
        vec![],
        vec![
            clipboard::label(vec![], vec![text("Share link")]),
            clipboard::control(
                false,
                vec![],
                vec![
                    clipboard::input("https://example.com/share/abc", false, vec![]),
                    clipboard::trigger(
                        false,
                        vec![],
                        vec![
                            clipboard::indicator(false, false, vec![], vec![text("Copy")]),
                            clipboard::indicator(true, false, vec![], vec![text("Copied")]),
                        ],
                    ),
                ],
            ),
            clipboard::value_text(vec![], vec![text("https://example.com/share/abc")]),
        ],
    )];
    demo_page("Clipboard", body)
}

pub(super) fn timer_section() -> Node {
    let body = vec![timer::root(
        true,
        0,
        60_000,
        1_000,
        15_000,
        TimerPhase::Running,
        vec![],
        vec![
            timer::area(
                vec![],
                vec![
                    timer::item(
                        TimerUnit::Minutes,
                        vec![],
                        vec![
                            timer::item_value(TimerUnit::Minutes, vec![], vec![text("00")]),
                            timer::item_label(TimerUnit::Minutes, vec![], vec![text("min")]),
                        ],
                    ),
                    timer::separator(vec![], vec![text(":")]),
                    timer::item(
                        TimerUnit::Seconds,
                        vec![],
                        vec![
                            timer::item_value(TimerUnit::Seconds, vec![], vec![text("45")]),
                            timer::item_label(TimerUnit::Seconds, vec![], vec![text("sec")]),
                        ],
                    ),
                ],
            ),
            timer::control(
                vec![],
                vec![
                    timer::action_trigger(TimerControl::Pause, vec![], vec![text("Pause")]),
                    timer::action_trigger(TimerControl::Reset, vec![], vec![text("Reset")]),
                ],
            ),
        ],
    )];
    demo_page("Timer", body)
}

pub(super) fn progress_section() -> Node {
    let progress = Progress::new(
        0.0,
        100.0,
        Some(40.0),
        hui::data_attrs::Orientation::Horizontal,
    );
    let body = vec![progress.root(
        Some("40%"),
        vec![],
        vec![
            progress.label(vec![], vec![text("Uploading")]),
            progress.track(vec![], vec![progress.range(vec![], vec![])]),
            progress.value_text(vec![], vec![text("40%")]),
        ],
    )];
    demo_page("Progress", body)
}

pub(super) fn qr_code_section() -> Node {
    let matrix = qr_code::encode("https://example.com", qr_code::ErrorCorrectionLevel::L);
    let body = match matrix {
        Ok(matrix) => vec![qr_code::root(
            vec![],
            vec![qr_code::frame(
                &matrix,
                2,
                Some("QR code for https://example.com"),
                vec![],
                vec![
                    qr_code::pattern(&matrix, 2, vec![]),
                    qr_code::overlay(vec![], vec![]),
                ],
            )],
        )],
        Err(_) => vec![],
    };
    demo_page("QR Code", body)
}

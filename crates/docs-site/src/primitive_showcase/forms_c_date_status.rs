//! Primitives Demo — Forms C・日付・状態表示（10 件、原稿は #1026）。
//! 執筆規約は `crate::primitive_showcase` モジュール doc 参照。

use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::calendar;
use hui::clipboard;
use hui::date::PlainDate;
use hui::date_input::{DateInput, DateSegment};
use hui::date_picker;
use hui::download_trigger;
use hui::progress::Progress;
use hui::qr_code;
use hui::timer::{self, TimerControl, TimerPhase, TimerUnit};
use hui::toggle;
use hui::toggle_group;
use hui::OpenState;

use super::demo_page;

pub(super) fn calendar_section() -> Node {
    let today = PlainDate::new(2026, 7, 25);
    let body = match today {
        Ok(today) => vec![calendar::root(
            vec![],
            vec![
                calendar::heading(Some("cal-heading"), vec![], vec![text("July 2026")]),
                calendar::prev_trigger(false, vec![], vec![text("‹")]),
                calendar::next_trigger(false, vec![], vec![text("›")]),
                calendar::table(
                    Some("cal-heading"),
                    vec![],
                    vec![
                        calendar::table_header(
                            vec![],
                            vec![calendar::table_row(
                                vec![],
                                vec![calendar::table_head_cell(vec![], vec![text("Su")])],
                            )],
                        ),
                        calendar::table_body(
                            vec![],
                            vec![calendar::table_row(
                                vec![],
                                vec![calendar::table_cell(
                                    true,
                                    vec![],
                                    vec![calendar::day_trigger(
                                        today,
                                        true,
                                        true,
                                        false,
                                        false,
                                        Some("cal-day-25"),
                                        vec![],
                                        vec![text("25")],
                                    )],
                                )],
                            )],
                        ),
                    ],
                ),
            ],
        )],
        Err(_) => vec![],
    };
    demo_page("Calendar", body)
}

/// DateInput の Demo（Anatomy・`data-*` 属性表の機械導出元）を 5 状態
/// （入力済み・focused / 未入力 / disabled / readonly / invalid）分
/// 用意する（イシュー #1626）。各インスタンスは `id_prefix` で id を
/// 一意にし、`label` の id を `segment_group` へ `aria-labelledby` として
/// 配線する例を含める。フォーカス状態は状態機械 [`DateInput`] を
/// `"focus"` dispatch してから描画することで作る（headless 側の
/// `DateInputProps::focused` は状態機械が導出する契約であり、パーツ関数へ
/// 直接 `true` を渡す用途を想定していないため）。
fn date_input_instance(
    id_prefix: &str,
    label_text: &str,
    mut state: DateInput,
    disabled: bool,
    readonly: bool,
    focus_year: bool,
) -> Node {
    if focus_year {
        hui::fandhe_frontend_interactive::dispatch(&mut state, "focus", "year");
    }
    let label_id = format!("{id_prefix}-label");
    let group_id = format!("{id_prefix}-group");
    state.root(
        disabled,
        readonly,
        vec![],
        vec![
            state.label(
                disabled,
                readonly,
                None,
                vec![("id", label_id.as_str())],
                vec![text(label_text)],
            ),
            state.control(
                disabled,
                readonly,
                vec![],
                vec![
                    state.segment_group(
                        disabled,
                        readonly,
                        vec![
                            ("id", group_id.as_str()),
                            ("aria-labelledby", label_id.as_str()),
                        ],
                        vec![
                            state.segment(DateSegment::Year, disabled, readonly, vec![]),
                            state.segment(DateSegment::Month, disabled, readonly, vec![]),
                            state.segment(DateSegment::Day, disabled, readonly, vec![]),
                        ],
                    ),
                    state.hidden_input(&format!("{id_prefix}-value"), disabled, vec![]),
                ],
            ),
        ],
    )
}

pub(super) fn date_input_section() -> Node {
    let body = vec![
        date_input_instance(
            "di-filled",
            "Date (filled, focused)",
            DateInput::new(Some(2026), Some(7), Some(22), None, None),
            false,
            false,
            true,
        ),
        date_input_instance(
            "di-empty",
            "Date (empty)",
            DateInput::default(),
            false,
            false,
            false,
        ),
        date_input_instance(
            "di-disabled",
            "Date (disabled)",
            DateInput::new(Some(2026), Some(1), Some(1), None, None),
            true,
            false,
            false,
        ),
        date_input_instance(
            "di-readonly",
            "Date (readonly)",
            DateInput::new(Some(2026), Some(7), Some(22), None, None),
            false,
            true,
            false,
        ),
        date_input_instance(
            "di-invalid",
            "Date (invalid, Feb 30)",
            // 2024-02-30 は実在しない日付。DateInput::is_invalid() が true になる。
            DateInput::new(Some(2024), Some(2), Some(30), None, None),
            false,
            false,
            false,
        ),
    ];
    demo_page("Date Input", body)
}

/// [`date_picker_section`] の内部ヘルパ。`id_prefix` ごとに id を一意化し、
/// `props`（disabled/readonly/invalid/required）の異なる組み合わせで
/// 1 インスタンスを組み立てる（イシュー #1627、combobox 系デモの
/// 多インスタンス化と同型のパターン）。
fn date_picker_instance(
    id_prefix: &str,
    state: OpenState,
    props: &date_picker::DatePickerProps,
    label_text: &str,
    value: &str,
) -> Node {
    let label_id = format!("dp-{id_prefix}-label");
    let input_id = format!("dp-{id_prefix}-input");
    let content_id = format!("dp-{id_prefix}-content");
    date_picker::root(
        state,
        props,
        vec![],
        vec![
            date_picker::label(
                props,
                Some(label_id.as_str()),
                Some(input_id.as_str()),
                vec![],
                vec![text(label_text)],
            ),
            date_picker::control(
                state,
                props,
                vec![],
                vec![
                    date_picker::input(Some(value), props, Some(input_id.as_str()), vec![]),
                    date_picker::trigger(
                        state,
                        props,
                        Some(content_id.as_str()),
                        vec![],
                        vec![text("📅")],
                    ),
                    date_picker::clear_trigger(props, vec![], vec![text("×")]),
                ],
            ),
            date_picker::positioner(
                state,
                vec![],
                vec![date_picker::content(
                    state,
                    Some(content_id.as_str()),
                    Some(label_id.as_str()),
                    vec![],
                    vec![],
                )],
            ),
        ],
    )
}

pub(super) fn date_picker_section() -> Node {
    let default_props = date_picker::DatePickerProps::default();
    let disabled_props = date_picker::DatePickerProps {
        disabled: true,
        ..Default::default()
    };
    let invalid_required_props = date_picker::DatePickerProps {
        invalid: true,
        required: true,
        ..Default::default()
    };
    let readonly_props = date_picker::DatePickerProps {
        readonly: true,
        ..Default::default()
    };
    let body = vec![
        date_picker_instance(
            "closed",
            OpenState::Closed,
            &default_props,
            "Date",
            "2026-07-25",
        ),
        date_picker_instance(
            "open",
            OpenState::Open,
            &default_props,
            "Date (open)",
            "2026-07-25",
        ),
        date_picker_instance(
            "disabled",
            OpenState::Closed,
            &disabled_props,
            "Date (disabled)",
            "2026-07-25",
        ),
        date_picker_instance(
            "invalid",
            OpenState::Closed,
            &invalid_required_props,
            "Date (invalid, required)",
            "2026-07-25",
        ),
        date_picker_instance(
            "readonly",
            OpenState::Closed,
            &readonly_props,
            "Date (readonly)",
            "2026-07-25",
        ),
    ];
    demo_page("Date Picker", body)
}

/// 参考サイト（ark-ui/chakra-ui）の DownloadTrigger デモは basic / async
/// data / svg の複数変種を並べる。本 Demo も `file_name` あり/なしの両変種
/// を並べ、`download` 属性の 2 通りの挙動（ファイル名ヒントの有無）を
/// 可視化する（イシュー #1628）。
pub(super) fn download_trigger_section() -> Node {
    // root（`a[download]`）はインライン要素であり、複数バリアントを
    // そのまま隣接配置するとラベルが視覚的に連結してしまう
    // （run-on text）。各バリアントを個別の `div`（ブロック要素）で
    // ラップし、視覚的に分離する（イシュー #1628 Bugbot 指摘対応）。
    let body = vec![
        div(
            vec![],
            vec![download_trigger::root(
                "https://example.com/assets/report.pdf",
                Some("report.pdf"),
                vec![],
                vec![text("Download report")],
            )],
        ),
        div(
            vec![],
            vec![download_trigger::root(
                "https://example.com/assets/data.csv",
                None,
                vec![],
                vec![text("Download data.csv")],
            )],
        ),
    ];
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

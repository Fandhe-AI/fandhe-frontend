//! `date_picker` モジュールの統合テスト（イシュー #835）。
//!
//! popover 開閉合成・input 値・dispatch の単体テストは
//! `crates/headless-ui/src/date_picker.rs` の `#[cfg(test)]` に集約済み。
//! 本ファイルは公開 API 経由の統合利用・現在時刻 API 非使用の機械検査のみ
//! を担う。

use fandhe_frontend_headless_ui::date::{PlainDate, Weekday};
use fandhe_frontend_headless_ui::date_picker::{content, input, trigger};
use fandhe_frontend_headless_ui::fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::fandhe_frontend_interactive::dispatch;
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_headless_ui::{Calendar, DatePicker};

fn sample_calendar() -> Calendar {
    Calendar::new(
        2026,
        7,
        PlainDate::new(2026, 7, 22).unwrap(),
        None,
        None,
        None,
        Weekday::Monday,
    )
    .unwrap()
}

#[test]
fn public_api_open_select_and_render_composed_popover() {
    let mut dp = DatePicker::new(sample_calendar());
    dispatch(&mut dp, "open", "");
    assert!(dp.is_open());

    let trigger_html = render(&dp.trigger(false, None, vec![], vec![]));
    assert!(trigger_html.contains(r#"aria-expanded="true""#));

    dispatch(&mut dp, "select", "2026-07-22");
    assert_eq!(dp.selected(), Some(PlainDate::new(2026, 7, 22).unwrap()));
    assert!(!dp.is_open());
}

#[test]
fn public_api_input_reflects_selected_iso_value() {
    let mut dp = DatePicker::new(sample_calendar());
    dispatch(&mut dp, "select", "2026-07-22");
    let value = dp.selected().map(|d| d.to_iso_string());
    let html = render(&input(value.as_deref(), false, None, vec![]));
    assert!(html.contains(r#"value="2026-07-22""#));
}

#[test]
fn public_api_content_and_trigger_share_state() {
    let html_open = render(&content(OpenState::Open, None, None, vec![], vec![]));
    assert!(!html_open.contains("hidden"));
    let html_trigger = render(&trigger(OpenState::Open, false, None, vec![], vec![]));
    assert!(html_trigger.contains(r#"aria-expanded="true""#));
}

// ---------------------------------------------------------------------
// 現在時刻 API 非使用の機械検査（`crates/headless-ui/tests/date.rs` と同型）
// ---------------------------------------------------------------------

#[test]
fn date_picker_module_never_reads_the_current_time() {
    let source = include_str!("../src/date_picker.rs");
    let forbidden_tokens = ["SystemTime", "std::time", "Instant", "js_sys", "now()"];
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        for token in forbidden_tokens {
            assert!(
                !line.contains(token),
                "date_picker.rs の実コード行に現在時刻取得の疑いがあるトークン {token:?} が \
                 見つかった: {line:?}"
            );
        }
    }
}

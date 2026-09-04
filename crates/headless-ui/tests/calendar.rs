//! `calendar` モジュールの統合テスト（イシュー #835）。
//!
//! 決定性・ARIA grid パターン・範囲クランプの単体テストは
//! `crates/headless-ui/src/calendar.rs` の `#[cfg(test)]` に集約済み
//! （`docs/api/component-api.md` の既存パターンに従う）。本ファイルは
//! それらでは検証できない横断的な観点（現在時刻 API 非使用の機械検査・
//! 公開 API 経由の統合利用）のみを担う。

use fandhe_frontend_headless_ui::calendar::{
    day_trigger, heading, next_trigger, prev_trigger, root, table, table_body, table_cell,
    table_head_cell, table_header, table_row, Calendar, CalendarAction,
};
use fandhe_frontend_headless_ui::date::{PlainDate, Weekday};
use fandhe_frontend_headless_ui::fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::fandhe_frontend_interactive::{dispatch, Component};

fn today() -> PlainDate {
    PlainDate::new(2026, 7, 22).unwrap()
}

/// イシュー #1625 の参考サイト突合テスト共通フィクスチャ。
///
/// 表示月 2026-07、today = selected = 2026-07-25（同日、`data-selected` と
/// `data-today` の同時付与を検証するため）、min = 2026-07-05 / max =
/// 2026-07-28（月内に disabled セルを作り月境界の `data-outside-month` と
/// 区別できるようにするため）、週開始は日曜日。
fn reference_alignment_fixture() -> Calendar {
    Calendar::new(
        2026,
        7,
        PlainDate::new(2026, 7, 25).unwrap(),
        Some(PlainDate::new(2026, 7, 25).unwrap()),
        Some(PlainDate::new(2026, 7, 5).unwrap()),
        Some(PlainDate::new(2026, 7, 28).unwrap()),
        Weekday::Sunday,
    )
    .unwrap()
}

/// [`reference_alignment_fixture`] の 11 パーツをすべて合成した SSR 出力。
/// anatomy・`data-*` 語彙・ARIA の突合テストが共通で使う完全な HTML 断片。
fn render_full_calendar(cal: &Calendar) -> String {
    let body = cal.table_body_from_grid(Vec::new());
    let thead = table_header(
        Vec::new(),
        vec![table_row(
            Vec::new(),
            vec![table_head_cell(Vec::new(), vec![])],
        )],
    );
    let tbl = table(Some("cal-heading"), Vec::new(), vec![thead, body]);
    let node = root(
        Vec::new(),
        vec![
            heading(Some("cal-heading"), Vec::new(), Vec::new()),
            cal.prev_trigger(vec![("aria-label", "Previous month")], Vec::new()),
            cal.next_trigger(vec![("aria-label", "Next month")], Vec::new()),
            tbl,
        ],
    );
    render(&node)
}

#[test]
fn public_api_month_grid_and_selection_integration() {
    let mut cal = Calendar::new(2026, 7, today(), None, None, None, Weekday::Monday).unwrap();
    assert!(dispatch(&mut cal, "select", "2026-07-22"));
    assert_eq!(cal.selected(), Some(today()));

    let html = render(&cal.table_body_from_grid(Vec::new()));
    assert!(html.contains(r#"data-scope="calendar""#));
    assert!(html.contains(r#"data-part="table-body""#));
    assert!(html.contains(r#"data-selected"#));
}

#[test]
fn public_api_parts_compose_into_a_grid_table() {
    let d = today();
    let cell = table_cell(
        true,
        vec![],
        vec![day_trigger(
            d,
            true,
            true,
            false,
            false,
            None,
            vec![],
            vec![],
        )],
    );
    let row = table_row(vec![], vec![cell]);
    let body = table_body(vec![], vec![row]);
    let html = render(&body);
    assert!(html.contains(r#"role="gridcell""#));
    assert!(html.contains(r#"aria-current="date""#));
}

#[test]
fn calendar_action_variants_round_trip_via_dispatch() {
    let mut cal = Calendar::new(2026, 7, today(), None, None, None, Weekday::Monday).unwrap();
    cal.update(CalendarAction::NextMonth);
    assert_eq!((cal.view_year(), cal.view_month()), (2026, 8));
}

// ---------------------------------------------------------------------
// 参考サイト（ark-ui/zag date-picker・chakra-ui Calendar）突合テスト
// （イシュー #1625）
// ---------------------------------------------------------------------

/// 11 anatomy パーツの `data-part` 集合が突合結果（本ファイル冒頭コメント・
/// PR 差分メモ参照）と完全一致することを検証する。増減があれば必ず失敗する
/// fail-closed 契約（意図的な追加・削除は本テストの期待集合も同時更新する
/// こと）。
#[test]
fn anatomy_parts_match_reference_alignment_exactly() {
    let cal = reference_alignment_fixture();
    let html = render_full_calendar(&cal);

    let mut found: Vec<String> = Vec::new();
    let mut rest = html.as_str();
    while let Some(idx) = rest.find(r#"data-part=""#) {
        let after = &rest[idx + r#"data-part=""#.len()..];
        let end = after.find('"').expect("data-part 属性値は必ず閉じる");
        found.push(after[..end].to_string());
        rest = &after[end..];
    }
    found.sort();
    found.dedup();

    let mut expected = vec![
        "root",
        "heading",
        "prev-trigger",
        "next-trigger",
        "table",
        "table-header",
        "table-row",
        "table-head-cell",
        "table-body",
        "table-cell",
        "day-trigger",
    ];
    expected.sort_unstable();

    assert_eq!(
        found,
        expected.into_iter().map(str::to_string).collect::<Vec<_>>(),
        "calendar の data-part 集合が参考サイト突合結果と一致しない（#1625）"
    );
    assert!(html.contains(r#"data-scope="calendar""#));
}

/// [`Calendar::table_body_from_grid`] 出力の `data-*` 語彙が突合結果どおり
/// であることを検証する（`data-selected`/`data-today`/`data-outside-month`/
/// `data-disabled`/`data-value`、および `aria-current="date"` の付与条件）。
#[test]
fn day_trigger_data_vocabulary_matches_reference() {
    let cal = reference_alignment_fixture();
    let html = render(&cal.table_body_from_grid(Vec::new()));

    // today = selected = 2026-07-25 のセルは両方の data-* を同時に持つ。
    assert!(html.contains(r#"data-value="2026-07-25""#));
    assert!(html.contains("data-selected"));
    assert!(html.contains("data-today"));
    assert!(html.contains(r#"aria-current="date""#));

    // 表示月 7 月の前後（6 月末・8 月頭）は data-outside-month を持つ。
    assert!(html.contains("data-outside-month"));

    // min=2026-07-05 未満・max=2026-07-28 超過の日付は data-disabled + 無効化。
    assert!(html.contains(r#"data-value="2026-07-04""#));
    assert!(html.contains("data-disabled"));
}

/// ark-ui/zag には存在するが本実装が意図的に採用しなかった `data-*`（DOM
/// ローカル状態・ビュー概念・locale 依存・range mode 等）が出力に一切
/// 現れないことを固定する（#1625 突合結果 §2.2 の判断を機械検査する）。
#[test]
fn intentionally_omitted_attributes_are_absent() {
    let cal = reference_alignment_fixture();
    let html = render_full_calendar(&cal);

    let must_be_absent = [
        "data-focus",
        "data-view",
        "data-weekend",
        "data-unavailable",
        "data-selectable",
        "data-range-start",
        "data-range-end",
        "data-in-range",
        "data-outside-range",
        "data-motion",
        "data-hover",
        "data-active",
    ];
    for token in must_be_absent {
        assert!(
            !html.contains(token),
            "calendar の出力に不採用のはずの {token:?} が含まれていた（#1625 突合結果 §2.2）"
        );
    }
}

/// WAI-ARIA APG の grid パターンに沿った role/aria の割り当てを検証する。
#[test]
fn roles_and_aria_follow_apg_grid_pattern() {
    let cal = reference_alignment_fixture();
    let html = render_full_calendar(&cal);

    assert!(html.contains(r#"role="grid""#));
    assert!(html.contains(r#"role="row""#));
    assert!(html.contains(r#"role="columnheader""#));
    assert!(html.contains(r#"role="gridcell""#));
    assert!(html.contains(r#"aria-selected="true""#) || html.contains(r#"aria-selected="false""#));
    assert!(html.contains(r#"id="cal-heading""#));
    assert!(html.contains(r#"aria-labelledby="cal-heading""#));

    // thead に aria-hidden は付与しない（WAI-ARIA APG の例に従い columnheader
    // として公開する、#1625 突合結果 §2.2）。
    let thead_open = html.find("<thead").expect("thead 開始タグが見つかるはず");
    let thead_close = html[thead_open..]
        .find('>')
        .expect("thead 開始タグは閉じるはず");
    let thead_tag = &html[thead_open..thead_open + thead_close];
    assert!(!thead_tag.contains("aria-hidden"));
}

/// トリガー系パーツがネイティブ `button`（`type="button"`）であること、
/// 無効化状態の三点セット（`disabled`/`aria-disabled`/`data-disabled`）、
/// 呼び出し側 `attrs` 経由の `aria-label` が既定エスケープを経由することを
/// 検証する。
#[test]
fn triggers_are_native_buttons_and_accept_caller_aria_label() {
    let disabled_prev = prev_trigger(true, Vec::new(), Vec::new());
    let html = render(&disabled_prev);
    assert!(html.contains(r#"type="button""#));
    assert!(html.contains("disabled"));
    assert!(html.contains(r#"aria-disabled="true""#));
    assert!(html.contains("data-disabled"));

    let enabled_next = next_trigger(false, vec![("aria-label", "Next month <icon>")], Vec::new());
    let html = render(&enabled_next);
    assert!(html.contains(r#"type="button""#));
    // `<` は既定エスケープ経由で `&lt;` になる（REQ-1）。
    assert!(html.contains("Next month &lt;icon&gt;"));
    assert!(!html.contains("Next month <icon>"));

    let day = day_trigger(
        today(),
        false,
        false,
        false,
        false,
        None,
        Vec::new(),
        Vec::new(),
    );
    let html = render(&day);
    assert!(html.contains(r#"type="button""#));
}

/// `Calendar::decode_action`（`fandhe_frontend_interactive::Component` 実装）
/// の dispatch 名が `crates/wasm-full` の `MAPPING_TABLE` 契約
/// （`"prev-month"`/`"next-month"`/`"select"`/`"clear-selection"`）と一致
/// することを headless 側から固定する。未知の name・不正な payload は
/// `None`（fail-closed）を返す。
#[test]
fn dispatch_names_match_wasm_full_mapping_table() {
    assert_eq!(
        Calendar::decode_action("prev-month", ""),
        Some(CalendarAction::PrevMonth)
    );
    assert_eq!(
        Calendar::decode_action("next-month", ""),
        Some(CalendarAction::NextMonth)
    );
    assert_eq!(
        Calendar::decode_action("select", "2026-07-25"),
        Some(CalendarAction::Select(PlainDate::new(2026, 7, 25).unwrap()))
    );
    assert_eq!(
        Calendar::decode_action("clear-selection", ""),
        Some(CalendarAction::ClearSelection)
    );

    assert_eq!(Calendar::decode_action("unknown-action", ""), None);
    assert_eq!(Calendar::decode_action("select", "not-a-date"), None);
}

// ---------------------------------------------------------------------
// 現在時刻 API 非使用の機械検査（`crates/headless-ui/tests/date.rs` と同型）
// ---------------------------------------------------------------------

#[test]
fn calendar_module_never_reads_the_current_time() {
    let source = include_str!("../src/calendar.rs");
    let forbidden_tokens = ["SystemTime", "std::time", "Instant", "js_sys", "now()"];
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        for token in forbidden_tokens {
            assert!(
                !line.contains(token),
                "calendar.rs の実コード行に現在時刻取得の疑いがあるトークン {token:?} が \
                 見つかった（「今日」は呼び出し側から明示的に受け取る決定的設計であり、\
                 現在時刻 API を呼んではならない）: {line:?}"
            );
        }
    }
}

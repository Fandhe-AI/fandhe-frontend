//! pre-styled-only 部品が「出力」する非 anatomy `data-*` 語彙（イシュー
//! #1063）の固定契約テスト。
//!
//! # 本ファイルのスコープ
//!
//! `docs/design/pre-styled-ui-data-attr-vocabulary.md` §2.1 が洗い出した
//! 5 語彙 6 出力箇所（`data-current`/`data-loading`/`data-action`/
//! `data-value`/`data-series`）について、付与条件・非付与条件・予約キー
//! 偽装除去をレンダリング結果で直接固定する。ソースを正規表現で走査する
//! fail-closed スキャナは同文書 §3.4 の判断により採用しない代わりに、本
//! ファイルが「レンダリング結果を決定的に固定する」代替手段を担う。
//!
//! XSS 回帰（`data-value`/`data-action`/`data-series` の動的値経路）は
//! `crates/pre-styled-ui/tests/xss_escape_styled.rs` が `payloads::all()`
//! で既に網羅している（(25) charts 経路・tag close_trigger 経路・
//! radio_card::item 経路）。本ファイルはそれを重複させず、単一ペイロードの
//! 最小回帰のみを追加して既定エスケープ（REQ-1）の迂回がないことを補強する。
//!
//! # 削除・弱体化の禁止
//!
//! `.claude/rules/coding-rust.md` の規約により、本ファイルのテストは
//! 以後の削除・弱体化・`#[ignore]` 化を禁止する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
use fandhe_frontend_pre_styled_ui::charts::radar_chart::{self, RadarChartProps};
use fandhe_frontend_pre_styled_ui::charts::scatter_chart::{
    self, ScatterChartProps, ScatterData, ScatterSeries,
};
use fandhe_frontend_pre_styled_ui::radio_card;
use fandhe_frontend_pre_styled_ui::tab_nav;
use fandhe_frontend_pre_styled_ui::tag;

/// XSS 回帰用の最小ペイロード（`xss_escape_styled.rs` の既存様式に合わせる）。
const XSS_PAYLOAD: &str = "\"><script>alert(1)</script>";

fn assert_no_raw_payload(html: &str, context_label: &str) {
    assert!(
        !html.contains("<script>alert(1)</script>"),
        "{context_label}: 生ペイロードが出力に残っている: html={html}"
    );
}

/// `data-loading`（`button.rs`）: `loading: true` のときのみ付与し、
/// `aria-busy="true"` を併記する。`false` のときは非出力（既定エスケープ
/// 経路とは独立の存在属性であり動的値を運ばないため XSS 対象外）。
#[test]
fn button_data_loading_is_gated_by_loading_flag() {
    let loading = ButtonProps {
        loading: true,
        ..ButtonProps::default()
    };
    let html = render(&button(&loading, vec![], vec![text("Save")]));
    assert!(html.contains(r#"data-loading="""#));
    assert!(html.contains(r#"aria-busy="true""#));

    let not_loading = ButtonProps::default();
    let html = render(&button(&not_loading, vec![], vec![text("Save")]));
    assert!(!html.contains("data-loading"));
    assert!(!html.contains("aria-busy"));
}

/// `data-action`（`tag.rs::close_trigger`）: `Some` のときのみ付与し、値を
/// そのまま反映する。`None` のときは非出力。動的値のため XSS 回帰も固定する
/// （headless `timer::action_trigger` と共有する意味論、規約 B-2）。
#[test]
fn tag_close_trigger_data_action_is_gated_by_action_option() {
    let html = render(&tag::close_trigger(Some("dismiss"), vec![], vec![]));
    assert!(html.contains(r#"data-action="dismiss""#));

    let html = render(&tag::close_trigger(None, vec![], vec![]));
    assert!(!html.contains("data-action"));

    let html = render(&tag::close_trigger(Some(XSS_PAYLOAD), vec![], vec![]));
    assert_no_raw_payload(&html, "tag::close_trigger data-action 属性値コンテキスト");
}

/// `data-value`（`radio_card.rs::item`）: 常に付与し値は呼び出し側の
/// `value` 引数をそのまま反映する。`ITEM_RESERVED` により呼び出し側 `attrs`
/// 経由の `data-value` 偽装は除去される（既存の `item_drops_caller_
/// supplied_reserved_attrs` と同型の防御を本ファイルからも再固定する）。
/// 動的値のため XSS 回帰も固定する（headless 5 部品と共有する意味論、
/// 規約 B-2）。
#[test]
fn radio_card_item_data_value_reflects_value_and_drops_spoofed_attr() {
    let html = render(&radio_card::item(false, false, "red", vec![], vec![]));
    assert!(html.contains(r#"data-value="red""#));

    let html = render(&radio_card::item(
        false,
        false,
        "red",
        vec![("data-value", "attacker")],
        vec![],
    ));
    assert!(html.contains(r#"data-value="red""#));
    assert!(!html.contains("attacker"));

    let html = render(&radio_card::item(false, false, XSS_PAYLOAD, vec![], vec![]));
    assert_no_raw_payload(&html, "radio_card::item data-value 属性値コンテキスト");
}

/// `data-series`（`charts/radar_chart.rs`/`charts/scatter_chart.rs`）:
/// 系列名をそのまま反映する。charts は pre-styled-only（headless-ui に
/// 対応部品なし）語彙の代表として固定する。動的値のため XSS 回帰も固定する。
#[test]
fn charts_data_series_reflects_series_name() {
    let scatter_data = ScatterData::new(vec![ScatterSeries::new("s1", vec![(0.0, 0.0)])])
        .expect("valid scatter series");
    let html = render(
        &scatter_chart::root(&scatter_data, ScatterChartProps::default(), "label")
            .expect("valid scatter chart"),
    );
    assert!(html.contains(r#"data-series="s1""#));

    let radar_data = ChartData::new(
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
        vec![Series::new("s1", vec![1.0, 2.0, 3.0])],
    )
    .expect("valid radar chart data");
    let html = render(
        &radar_chart::root(&radar_data, RadarChartProps::default(), "label")
            .expect("valid radar chart"),
    );
    assert!(html.contains(r#"data-series="s1""#));

    let scatter_data_payload =
        ScatterData::new(vec![ScatterSeries::new(XSS_PAYLOAD, vec![(0.0, 0.0)])])
            .expect("valid scatter series");
    let html = render(
        &scatter_chart::root(&scatter_data_payload, ScatterChartProps::default(), "label")
            .expect("valid scatter chart"),
    );
    assert_no_raw_payload(&html, "scatter_chart::root data-series 属性値コンテキスト");
}

/// `data-current`（`tab_nav.rs::link`）: `current: true` のときのみ付与する。
/// イシュー #1063 でヘルパ（`fandhe_frontend_headless_ui::data_attrs::
/// data_current`）経由化した後も出力が完全に不変であることを固定する
/// （生タプルでの `("data-current", "")` と等価）。
#[test]
fn tab_nav_link_data_current_is_gated_by_current_flag() {
    let html = render(&tab_nav::link("/docs", true, vec![], vec![text("Docs")]));
    assert!(html.contains(r#"data-current="""#));
    assert!(html.contains(r#"aria-current="page""#));

    let html = render(&tab_nav::link("/docs", false, vec![], vec![text("Docs")]));
    assert!(!html.contains("data-current"));
    assert!(!html.contains("aria-current"));
}

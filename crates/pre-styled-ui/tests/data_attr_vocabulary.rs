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
use fandhe_frontend_headless_ui::progress::Progress;
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
use fandhe_frontend_pre_styled_ui::charts::radar_chart::{self, RadarChartProps};
use fandhe_frontend_pre_styled_ui::charts::scatter_chart::{
    self, ScatterChartProps, ScatterData, ScatterSeries,
};
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps, FieldRootProps};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::progress::{self, Orientation, ProgressProps};
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

/// `field.rs`（イシュー #1684）は独自の `data-*` を一切出力しない
/// （`docs/design/pre-styled-ui-data-attr-vocabulary.md` §3.1 規約 A・
/// 役割 B）。styled `root` 出力に現れる `data-disabled`/`data-invalid`/
/// `data-required`/`data-readonly` はすべて headless
/// `fandhe_frontend_headless_ui::field::root` が [`FieldProps`] の 4
/// フラグから生成するものであり、`field::css()` はその属性を CSS
/// セレクタとして**参照する**だけで自前出力はしない、という事実を固定
/// する。
#[test]
fn field_root_data_attrs_are_headless_sourced_not_self_emitted() {
    fn field(id: &str) -> FieldProps<'_> {
        FieldProps {
            id,
            ids: FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        }
    }

    // 全フラグ false のとき、4 種の data-* はいずれも出力されない。
    let f = field("f");
    let html = render(&field::root(&FieldRootProps::default(), &f, vec![], vec![]));
    assert!(!html.contains("data-disabled"));
    assert!(!html.contains("data-invalid"));
    assert!(!html.contains("data-required"));
    assert!(!html.contains("data-readonly"));

    // 全フラグ true のとき、4 種すべてが headless `field::root` 経由で
    // 出力される（styled `root` 自身は data-* を組み立てない）。
    let f = FieldProps {
        id: "f",
        ids: FieldIds::default(),
        disabled: true,
        invalid: true,
        required: true,
        readonly: true,
        has_helper_text: false,
    };
    let html = render(&field::root(&FieldRootProps::default(), &f, vec![], vec![]));
    assert!(html.contains("data-disabled"));
    assert!(html.contains("data-invalid"));
    assert!(html.contains("data-required"));
    assert!(html.contains("data-readonly"));

    // `field::css()` は `[data-disabled]` を参照する state 規則を持つが、
    // 自前で `data-*` を組み立てて出力する経路（属性タプルの直接構築）を
    // 持たないことを、CSS 出力側からも確認する（属性セレクタとしての
    // 参照は許容、自前出力はしないという役割 B の境界を固定）。
    let css = field::css();
    assert!(css.contains("[data-disabled]"));
}

/// `fieldset.rs`（イシュー #1686）は独自の `data-*` を一切出力しない
/// （`docs/design/pre-styled-ui-data-attr-vocabulary.md` §3.1 規約 A・
/// 役割 B）。styled `root` 出力に現れる `data-disabled`/`data-invalid` は
/// すべて headless `fandhe_frontend_headless_ui::fieldset::root` が
/// [`FieldsetProps`] の 2 フラグから生成するものであり、`fieldset::css()`
/// はその属性を CSS セレクタとして**参照する**だけで自前出力はしない、
/// という事実を固定する（`field_root_data_attrs_are_headless_sourced_not_self_emitted`
/// と同型）。
#[test]
fn fieldset_root_data_attrs_are_headless_sourced_not_self_emitted() {
    fn fieldset_props(id: &str) -> FieldsetProps<'_> {
        FieldsetProps {
            id,
            disabled: false,
            invalid: false,
            has_helper_text: false,
        }
    }

    // 全フラグ false のとき、2 種の data-* はいずれも出力されない。
    let f = fieldset_props("f");
    let html = render(&fieldset::root(
        &FieldsetRootProps::default(),
        &f,
        vec![],
        vec![],
    ));
    assert!(!html.contains("data-disabled"));
    assert!(!html.contains("data-invalid"));

    // 全フラグ true のとき、2 種すべてが headless `fieldset::root` 経由で
    // 出力される（styled `root` 自身は data-* を組み立てない）。
    let f = FieldsetProps {
        id: "f",
        disabled: true,
        invalid: true,
        has_helper_text: false,
    };
    let html = render(&fieldset::root(
        &FieldsetRootProps::default(),
        &f,
        vec![],
        vec![],
    ));
    assert!(html.contains("data-disabled"));
    assert!(html.contains("data-invalid"));

    // `fieldset::css()` は `[data-disabled]` を参照する state 規則を持つが、
    // 自前で `data-*` を組み立てて出力する経路（属性タプルの直接構築）を
    // 持たないことを、CSS 出力側からも確認する。`[data-invalid]` は参照
    // しない（`legend` は invalid による色変更を持たない、モジュール doc
    // 「意図的非採用」節参照）。
    let css = fieldset::css();
    assert!(css.contains("[data-disabled]"));
    assert!(!css.contains("[data-invalid]"));
}

/// `progress.rs`（イシュー #763/#1564/#1688）は pre-styled-only の `data-*`
/// を一切出力しない（`docs/design/pre-styled-ui-data-attr-vocabulary.md`
/// §3.1 規約 A・役割 B、`field_root_data_attrs_are_headless_sourced_not_self_emitted`/
/// `fieldset_root_data_attrs_are_headless_sourced_not_self_emitted` と同型）。
/// styled `root`/`range` の出力に現れる `data-state`/`data-orientation` は
/// すべて headless `fandhe_frontend_headless_ui::progress::Progress` の
/// inherent メソッド由来であり、`progress::stylesheet()` はその属性を CSS
/// セレクタとして**参照する**だけで自前出力はしない、という事実を固定する。
/// linear（root/label/value-text/track/range）は `data-orientation` を持つが
/// circular 3 parts（circle/circle-track/circle-range）は持たない（headless
/// 側 rustdoc「data-orientation を持たない」節、`crates/headless-ui/src/progress.rs`
/// 参照）非対称も合わせて固定する。
#[test]
fn progress_parts_data_attrs_are_headless_sourced_not_self_emitted() {
    let determinate = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);
    let complete = Progress::new(0.0, 100.0, Some(100.0), Orientation::Horizontal);
    let indeterminate = Progress::new(0.0, 100.0, None, Orientation::Horizontal);

    // styled root: headless 経由で data-state が determinate/complete/
    // indeterminate の 3 状態を切り替える。
    let root_loading = render(&progress::root(
        &determinate,
        &ProgressProps::default(),
        None,
        vec![],
        vec![],
    ));
    assert!(root_loading.contains(r#"data-state="loading""#));
    assert!(root_loading.contains(r#"data-orientation="horizontal""#));

    let root_complete = render(&progress::root(
        &complete,
        &ProgressProps::default(),
        None,
        vec![],
        vec![],
    ));
    assert!(root_complete.contains(r#"data-state="complete""#));

    let root_indeterminate = render(&progress::root(
        &indeterminate,
        &ProgressProps::default(),
        None,
        vec![],
        vec![],
    ));
    assert!(root_indeterminate.contains(r#"data-state="indeterminate""#));

    // styled range も同じく headless 由来（determinate/indeterminate）。
    let range_loading = render(&progress::range(&determinate, vec![]));
    assert!(range_loading.contains(r#"data-state="loading""#));
    assert!(range_loading.contains(r#"data-orientation="horizontal""#));
    let range_indeterminate = render(&progress::range(&indeterminate, vec![]));
    assert!(range_indeterminate.contains(r#"data-state="indeterminate""#));

    // circular 3 parts は headless の inherent メソッドを直接呼ぶ
    // （styled ラッパーを経由しない、モジュール冒頭 rustdoc 参照）。
    // data-state は持つが data-orientation は持たない非対称を固定する。
    let circle_html = render(&indeterminate.circle(vec![], vec![]));
    assert!(circle_html.contains(r#"data-state="indeterminate""#));
    assert!(!circle_html.contains("data-orientation"));
    let circle_track_html = render(&indeterminate.circle_track(vec![], vec![]));
    assert!(circle_track_html.contains(r#"data-state="indeterminate""#));
    assert!(!circle_track_html.contains("data-orientation"));
    let circle_range_html = render(&indeterminate.circle_range(vec![], vec![]));
    assert!(circle_range_html.contains(r#"data-state="indeterminate""#));
    assert!(!circle_range_html.contains("data-orientation"));

    // `progress::stylesheet()` は `[data-state="indeterminate"]`/
    // `[data-orientation="vertical"]` を state セレクタとして参照するが、
    // 自前で data-* タプルを組み立てて出力する経路（属性タプルの直接
    // 構築）は持たない（CSS 出力側からの確認、他 2 テストと同型）。
    let css = progress::stylesheet();
    assert!(css.contains(r#"[data-state="indeterminate"]"#));
    assert!(css.contains(r#"[data-orientation="vertical"]"#));
}

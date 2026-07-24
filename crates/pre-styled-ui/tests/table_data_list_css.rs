//! styled Table / DataList（イシュー #767）の決定的 CSS 出力の固定テスト。
//!
//! `crates/pre-styled-ui/tests/form_controls_css.rs`（Input/Textarea/
//! NativeSelect、#737）は `css()` の全文をバイト単位で固定する golden
//! fixture 形式だが、本ファイルは breadcrumb（`src/breadcrumb.rs` の
//! `stylesheet_is_deterministic_and_contains_variant_selectors_and_tokens`）
//! と同型の「決定性 + 重要規則の存在確認」形式を採る。Table の recipe は
//! `variant`/`size`/`striped` の 3 軸が絡み合い、全文バイト一致固定は軸を
//! 1 つ調整するだけで無関係な差分が大量発生し、レビュー・保守コストに見合わ
//! ないと判断したため（決定性・fail-closed の不変条件はどちらの形式でも
//! 等しく検証できる）。
//!
//! 固定する観点:
//! 1. `css()` の呼び出しは決定的（複数回呼んでもバイト一致）。
//! 2. striped の `:nth-child(even)` 状態規則・`--fandhe-table-stripe-bg`
//!    custom property が出力に含まれる。
//! 3. size 3 値（sm/md/lg）の custom property 切り替えクラスが出力に含まれる。
//! 4. `variant`（line/outline）のクラスセレクタが出力に含まれる。
//! 5. `<` を一切含まない（`<style>` RAWTEXT 脱出防止、`StyleSheet::push_css`
//!    が要求する不変条件と同型）。
//! 6. recipe が生成するセレクタが、実際に本クレートがレンダリングする
//!    `data-scope`/`data-part` 属性と一致する（`recipe_css.rs` の
//!    `base_selectors_match_actual_headless_markup` と同型の接続照合）。

use fandhe_frontend_core::render;
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::Size;

#[test]
fn table_css_is_deterministic() {
    assert_eq!(table::css(), table::css());
}

#[test]
fn table_css_contains_striped_state_rule_and_custom_property() {
    let css = table::css();
    assert!(css.contains(r#"[data-scope="table"][data-part="row"]:nth-child(even) {"#));
    assert!(css.contains("background: var(--fandhe-table-stripe-bg, transparent);"));
    assert!(css.contains("--fandhe-table-stripe-bg: var(--fandhe-color-bg-subtle);"));
    assert!(css.contains("--fandhe-table-stripe-bg: transparent;"));
}

#[test]
fn table_css_contains_size_custom_property_variants() {
    let css = table::css();
    for (class, padding) in [
        ("fd-table--size-sm", "0.5rem 0.75rem"),
        ("fd-table--size-md", "0.75rem 1rem"),
        ("fd-table--size-lg", "1rem 1.25rem"),
    ] {
        let selector = format!(r#"[data-scope="table"][data-part="root"].{class} {{"#);
        assert!(
            css.contains(&selector),
            "missing selector: {selector}\n{css}"
        );
        assert!(
            css.contains(&format!("--fandhe-table-cell-padding: {padding};")),
            "missing padding decl for {class}\n{css}"
        );
    }
}

#[test]
fn table_css_contains_variant_selectors() {
    let css = table::css();
    assert!(css.contains(r#"[data-scope="table"][data-part="root"].fd-table--variant-line {"#));
    assert!(css.contains(r#"[data-scope="table"][data-part="root"].fd-table--variant-outline {"#));
}

/// `root` は `border-collapse: separate`（イシュー #767 PR #811 の Outline
/// variant 角丸修正で導入）であり、CSS 表モデル仕様上 `row`（`tr`）への
/// border 指定はブラウザに無視される。Line variant の行区切り線
/// （`--fandhe-table-row-border`）は `row` ではなく `cell`（`td`）側の
/// `border-bottom` として出力されなければならない（PR #811 Bugbot 追加指摘:
/// "Line row borders ignored" の回帰防止）。
#[test]
fn table_css_puts_row_border_on_cell_not_row() {
    let css = table::css();
    assert!(
        css.contains(
            r#"[data-scope="table"][data-part="cell"] {
  padding: var(--fandhe-table-cell-padding, 0.75rem 1rem);
  font-size: var(--fandhe-table-font-size, var(--fandhe-font-font-size-sm));
  border-bottom: var(--fandhe-table-row-border, none);
}"#
        ),
        "cell base rule must set border-bottom via --fandhe-table-row-border\n{css}"
    );
    // `row`（`tr`）の base 規則自体が出力されないこと（`separate` モデルでは
    // 無効なため、ここに書いても意味がない不変条件を CSS 出力レベルで固定する）。
    assert!(!css.contains(r#"[data-scope="table"][data-part="row"] {"#));
}

/// Outline variant は `border-radius` に加え `overflow: hidden` を
/// `root` に持たなければならない。`border-collapse: separate` では
/// `column-header` の不透明背景・striped 偶数行の背景が `root` の角丸に
/// 追従してクリップされず、矩形の角のまま描画される（イシュー #767
/// PR #811 Bugbot 指摘: "Outline corners not clipped" の回帰防止）。
#[test]
fn table_css_outline_variant_clips_descendants_to_border_radius() {
    let css = table::css();
    let outline_rule_start = css
        .find(r#"[data-scope="table"][data-part="root"].fd-table--variant-outline {"#)
        .expect("Outline variant の root 規則が css() 出力に存在すること");
    let outline_rule_end = css[outline_rule_start..]
        .find('}')
        .map(|offset| outline_rule_start + offset)
        .expect("Outline variant の root 規則が `}` で閉じられていること");
    let outline_rule = &css[outline_rule_start..outline_rule_end];
    assert!(
        outline_rule.contains("border-radius:"),
        "Outline variant の root 規則に border-radius が存在すること\n{outline_rule}"
    );
    assert!(
        outline_rule.contains("overflow: hidden;"),
        "Outline variant の root 規則に overflow: hidden がなく、\
         column-header/striped 偶数行の背景が角丸からはみ出す\n{outline_rule}"
    );
}

#[test]
fn table_css_never_contains_style_breakout_sequences() {
    let css = table::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn table_recipe_selectors_match_actual_rendered_markup() {
    let html = render(&table::root(
        TableProps {
            variant: TableVariant::Outline,
            size: Size::Lg,
            striped: true,
        },
        vec![],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="table" data-part="root""#));
    assert!(html.contains("fd-table--variant-outline"));
    assert!(html.contains("fd-table--size-lg"));
    assert!(html.contains("fd-table--striped-true"));

    let cell_html = render(&table::cell(vec![], vec![]));
    assert!(cell_html.starts_with(r#"<td data-scope="table" data-part="cell""#));

    let row_html = render(&table::row(vec![], vec![]));
    assert!(row_html.starts_with(r#"<tr data-scope="table" data-part="row""#));
}

#[test]
fn data_list_css_is_deterministic() {
    assert_eq!(data_list::css(), data_list::css());
}

#[test]
fn data_list_css_contains_orientation_custom_properties() {
    let css = data_list::css();
    assert!(css.contains(
        r#"[data-scope="data-list"][data-part="root"].fd-data-list--orientation-vertical {"#
    ));
    assert!(css.contains(
        r#"[data-scope="data-list"][data-part="root"].fd-data-list--orientation-horizontal {"#
    ));
    assert!(css.contains("--fandhe-data-list-item-flex-direction: column;"));
    assert!(css.contains("--fandhe-data-list-item-flex-direction: row;"));
}

#[test]
fn data_list_css_never_contains_style_breakout_sequences() {
    let css = data_list::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn data_list_recipe_selectors_match_actual_rendered_markup() {
    let html = render(&data_list::root(
        DataListProps {
            orientation: DataListOrientation::Horizontal,
        },
        vec![],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="data-list" data-part="root""#));
    assert!(html.contains("fd-data-list--orientation-horizontal"));

    let item_html = render(&data_list::item(vec![], vec![]));
    assert!(item_html.starts_with(r#"<div data-scope="data-list" data-part="item""#));
}

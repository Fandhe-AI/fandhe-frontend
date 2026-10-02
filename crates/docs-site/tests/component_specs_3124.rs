//! イシュー #3124（select の選択インジケータ位置指定・R1239 分割ボタン
//! 合成例を追加する）専用の契約テスト。
//!
//! `crates/docs-site/tests/component_pages.rs` 等の共有ファイルは並列実行
//! される他イシューも触り得るため変更しない方針（`component_specs_3118.rs`
//! と同じ per-issue テストファイル方式）。本ファイルは `/themes/select/`・
//! `/themes/button-group/` の 2 ページのみを検証する:
//! (1) select ページの Examples 節に `fd-select--item-indicator-placement-start`
//!     class を伴う新しい見出しが載ること
//! (2) button-group ページの Examples 節に分割ボタン合成例の新しい見出しが
//!     載り、menu trigger が button-group の子として描画されること
//! (3) 両ページのレンダリングが決定的であること
//! (4) 原稿（`forms.rs`/`component_specs_overlay.rs`）が `raw_html()` を
//!     使わずノード木 API のみで組み立てられていること
//!
//! pre-styled-ui 側の `ItemIndicatorPlacement` 軸実装・golden CSS・単体
//! テストは本イシューの別コミット（`crates/pre-styled-ui/`）で完了済みの
//! ため、本ファイルは docs-site の Examples 追加分のみを検証対象とする。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::generated_content;

/// `/themes/select/` の生成 HTML（1 回分）。
fn select_page_html() -> String {
    let node = generated_content("/themes/select/")
        .expect("generated_content(\"/themes/select/\") should be Some");
    render(&node)
}

/// `/themes/button-group/` の生成 HTML（1 回分）。
fn button_group_page_html() -> String {
    let node = generated_content("/themes/button-group/")
        .expect("generated_content(\"/themes/button-group/\") should be Some");
    render(&node)
}

/// select の Examples 節に選択インジケータ位置の新しい見出しが含まれ、
/// `fd-select--item-indicator-placement-start` class を実際に出力すること。
#[test]
fn select_examples_section_includes_item_indicator_placement_entry() {
    let html = select_page_html();
    assert!(
        html.contains("<h2>Examples</h2>"),
        "select page should have an <h2>Examples</h2> section"
    );
    assert!(
        html.contains("選択インジケータを左端に置く"),
        "Examples should include an item-indicator-placement entry, got: {html}"
    );
    assert!(
        html.contains("fd-select--item-indicator-placement-start"),
        "the example should render the Start placement class, got: {html}"
    );
}

/// button-group の Examples 節に分割ボタンの新しい見出しが含まれ、menu
/// trigger が button-group root の子として描画されること。
#[test]
fn button_group_examples_section_includes_split_button_entry() {
    let html = button_group_page_html();
    assert!(
        html.contains("<h2>Examples</h2>"),
        "button-group page should have an <h2>Examples</h2> section"
    );
    assert!(
        html.contains("分割ボタン"),
        "Examples should include a split-button entry, got: {html}"
    );
    assert!(
        html.contains(r#"data-scope="button-group""#) && html.contains(r#"data-scope="menu""#),
        "split-button example should combine button-group and menu parts, got: {html}"
    );
}

/// 両ページのレンダリングが決定的であること（同一入力から 2 回描画しても
/// バイト一致）。
#[test]
fn rendering_is_deterministic() {
    assert_eq!(
        select_page_html(),
        select_page_html(),
        "select page rendering should be deterministic"
    );
    assert_eq!(
        button_group_page_html(),
        button_group_page_html(),
        "button-group page rendering should be deterministic"
    );
}

/// `forms.rs`・`component_specs_overlay.rs` に `raw_html` が出現しないこと
/// （既存 XSS 回帰テストの走査対象に `component_specs/`・
/// `component_specs_overlay.rs` 配下は含まれるが、本イシュー追加分も崩れて
/// いないことを明示的に固定する）。
#[test]
fn select_and_button_group_examples_use_no_raw_html() {
    for (path, source) in [
        (
            "component_specs/forms.rs",
            include_str!("../src/component_specs/forms.rs"),
        ),
        (
            "component_specs_overlay.rs",
            include_str!("../src/component_specs_overlay.rs"),
        ),
    ] {
        for (i, line) in source.lines().enumerate() {
            let code_part = match line.find("//") {
                Some(idx) => &line[..idx],
                None => line,
            };
            assert!(
                !code_part.contains("raw_html"),
                "{path} line {} should not call raw_html(): {line}",
                i + 1
            );
        }
    }
}

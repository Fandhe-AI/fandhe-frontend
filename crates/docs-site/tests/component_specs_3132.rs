//! イシュー #3132（pre-styled-ui の `calendar` に枠線なし variant と大セル
//! 月表示を追加する）専用の契約テスト。
//!
//! `crates/docs-site/tests/component_pages.rs` /
//! `crates/docs-site/tests/component_page_specs_948.rs` は並列実行される
//! 他イシューも触り得る共有ファイルのため変更しない方針
//! （`crates/docs-site/tests/component_specs_3125.rs` と同じ per-issue
//! テストファイル方式）。本ファイルは `/themes/calendar/` 1 ページのみを
//! 検証する: (1) Examples 節に新しい見出し 2 件が載ること、(2)
//! `fd-calendar--variant-plain` と `fd-calendar--cell-size-large` が
//! それぞれ size class と同じ `class` 属性内に並ぶこと、(3) レンダリングが
//! 決定的であること、(4) 生の `<script` を含まないこと、(5)
//! `component_page_specs_948.rs` が `raw_html()` を使わないこと。
//!
//! pre-styled-ui 側の variant/cell-size 軸実装・golden CSS・単体テストは
//! 本イシューの同一 PR 内で完了済みのため、本ファイルは docs-site の
//! Examples 追加分のみを検証対象とする。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::generated_content;

/// `/themes/calendar/` の生成 HTML（1 回分）。
fn calendar_page_html() -> String {
    let node = generated_content("/themes/calendar/")
        .expect("generated_content(\"/themes/calendar/\") should be Some");
    render(&node)
}

/// Examples 節に Plain variant / Large cell-size の新しい見出しが含まれること。
#[test]
fn examples_section_includes_plain_and_large_entries() {
    let html = calendar_page_html();
    assert!(
        html.contains("<h2>Examples</h2>"),
        "calendar page should have an <h2>Examples</h2> section"
    );
    assert!(
        html.contains("枠線なし（Plain variant）"),
        "Examples should include a Plain variant entry, got: {html}"
    );
    assert!(
        html.contains("大セル月表示（Large cell-size）"),
        "Examples should include a Large cell-size entry, got: {html}"
    );
}

/// Plain variant 例の root が `class="fd-calendar--size-md fd-calendar--variant-plain"`
/// を出力すること。
#[test]
fn plain_example_renders_variant_class_alongside_size_class() {
    let html = calendar_page_html();
    assert!(
        html.contains(r#"class="fd-calendar--size-md fd-calendar--variant-plain""#),
        "plain example should render size + variant classes together, got: {html}"
    );
}

/// Large cell-size 例の root が `class="fd-calendar--size-sm fd-calendar--cell-size-large"`
/// を出力すること。
#[test]
fn large_cell_size_example_renders_cell_size_class_alongside_size_class() {
    let html = calendar_page_html();
    assert!(
        html.contains(r#"class="fd-calendar--size-sm fd-calendar--cell-size-large""#),
        "large cell-size example should render size + cell-size classes together, got: {html}"
    );
}

/// レンダリングが決定的であること（同一入力から 2 回描画してもバイト一致）。
#[test]
fn rendering_is_deterministic() {
    let first = calendar_page_html();
    let second = calendar_page_html();
    assert_eq!(
        first, second,
        "calendar page rendering should be deterministic"
    );
}

/// 生の `<script` を含まないこと（XSS 回帰の明示的固定）。
#[test]
fn calendar_page_contains_no_raw_script_tag() {
    let html = calendar_page_html();
    assert!(
        !html.contains("<script"),
        "calendar page should not contain a raw <script tag, got: {html}"
    );
}

/// `component_page_specs_948.rs` に `raw_html` が出現しないこと（既存 XSS
/// 回帰テストの走査対象に同ファイルは含まれるが、本イシュー追加分も
/// 崩れていないことを明示的に固定する）。
#[test]
fn calendar_examples_use_no_raw_html() {
    let source = include_str!("../src/component_page_specs_948.rs");
    for (i, line) in source.lines().enumerate() {
        let code_part = match line.find("//") {
            Some(idx) => &line[..idx],
            None => line,
        };
        assert!(
            !code_part.contains("raw_html"),
            "component_page_specs_948.rs line {} should not call raw_html(): {line}",
            i + 1
        );
    }
}

//! イシュー #3126（pre-styled-ui の `tab_nav` にカード状バー variant
//! `TabNavVariant::Bar` を追加する）専用の契約テスト。
//!
//! `crates/docs-site/tests/component_pages.rs` /
//! `crates/docs-site/tests/component_page_specs_948.rs` は並列実行される
//! 他イシューも触り得る共有ファイルのため変更しない方針
//! （`crates/docs-site/tests/component_specs_3125.rs` と同じ per-issue
//! テストファイル方式）。本ファイルは `/themes/tab-nav/` 1 ページのみを
//! 検証する: (1) Examples 節に「Bar variant」の見出しが載ること、(2)
//! `class="fd-tab-nav--size-md fd-tab-nav--variant-bar"` が出力されること、
//! (3) レンダリングが決定的であること、(4) 生の `<script` を含まないこと、
//! (5) 原稿（`component_specs_nav_data.rs`）が `raw_html()` を使わないこと。
//!
//! pre-styled-ui 側の bar variant 実装・golden CSS・単体テストは本イシュー
//! の同一 PR 内で完了済みのため、本ファイルは docs-site の Examples 追加分
//! のみを検証対象とする。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::generated_content;

/// `/themes/tab-nav/` の生成 HTML（1 回分）。
fn tab_nav_page_html() -> String {
    let node = generated_content("/themes/tab-nav/")
        .expect("generated_content(\"/themes/tab-nav/\") should be Some");
    render(&node)
}

/// Examples 節に bar variant の新しい見出しが含まれること。
#[test]
fn examples_section_includes_bar_entry() {
    let html = tab_nav_page_html();
    assert!(
        html.contains("<h2>Examples</h2>"),
        "tab-nav page should have an <h2>Examples</h2> section"
    );
    assert!(
        html.contains("Bar variant"),
        "Examples should include a Bar variant entry, got: {html}"
    );
}

/// Bar variant 例の root が `class="fd-tab-nav--size-md fd-tab-nav--variant-bar"`
/// を出力すること。
#[test]
fn bar_example_renders_variant_class_alongside_size_class() {
    let html = tab_nav_page_html();
    assert!(
        html.contains(r#"class="fd-tab-nav--size-md fd-tab-nav--variant-bar""#),
        "bar example should render size + variant classes together, got: {html}"
    );
}

/// レンダリングが決定的であること（同一入力から 2 回描画してもバイト一致）。
#[test]
fn rendering_is_deterministic() {
    let first = tab_nav_page_html();
    let second = tab_nav_page_html();
    assert_eq!(
        first, second,
        "tab-nav page rendering should be deterministic"
    );
}

/// 生の `<script` を含まないこと（XSS 回帰の明示的固定）。
#[test]
fn tab_nav_page_contains_no_raw_script_tag() {
    let html = tab_nav_page_html();
    assert!(
        !html.contains("<script"),
        "tab-nav page should not contain a raw <script tag, got: {html}"
    );
}

/// `component_specs_nav_data.rs` に `raw_html` が出現しないこと（既存 XSS
/// 回帰テストの走査対象に `component_specs_nav_data.rs` は含まれるが、
/// 本イシュー追加分も崩れていないことを明示的に固定する）。
#[test]
fn tab_nav_examples_use_no_raw_html() {
    let source = include_str!("../src/component_specs_nav_data.rs");
    for (i, line) in source.lines().enumerate() {
        let code_part = match line.find("//") {
            Some(idx) => &line[..idx],
            None => line,
        };
        assert!(
            !code_part.contains("raw_html"),
            "component_specs_nav_data.rs line {} should not call raw_html(): {line}",
            i + 1
        );
    }
}

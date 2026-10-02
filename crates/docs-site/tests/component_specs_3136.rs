//! イシュー #3136（pre-styled-ui の `pagination` に連結表示 variant
//! `PaginationVariant::Attached` を追加する）専用の契約テスト。
//!
//! `crates/docs-site/tests/component_pages.rs` /
//! `crates/docs-site/tests/component_page_specs_948.rs` は並列実行される
//! 他イシューも触り得る共有ファイルのため変更しない方針
//! （`crates/docs-site/tests/component_specs_3126.rs` と同じ per-issue
//! テストファイル方式）。本ファイルは `/themes/pagination/` 1 ページのみを
//! 検証する: (1) Examples 節に「Attached」の見出しが載ること、(2)
//! `fd-pagination--variant-attached` が出力されること、(3) レンダリングが
//! 決定的であること、(4) 生の `<script` を含まないこと、(5) 原稿
//! （`component_specs_nav_data.rs`）が `raw_html()` を使わないこと。
//!
//! pre-styled-ui 側の attached variant 実装・golden CSS・単体テストは本
//! イシューの同一 PR 内で完了済みのため、本ファイルは docs-site の
//! Examples 追加分のみを検証対象とする。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::generated_content;

/// `/themes/pagination/` の生成 HTML（1 回分）。
fn pagination_page_html() -> String {
    let node = generated_content("/themes/pagination/")
        .expect("generated_content(\"/themes/pagination/\") should be Some");
    render(&node)
}

/// Examples 節に Attached の新しい見出しが含まれること。
#[test]
fn examples_section_includes_attached_entry() {
    let html = pagination_page_html();
    assert!(
        html.contains("<h2>Examples</h2>"),
        "pagination page should have an <h2>Examples</h2> section"
    );
    assert!(
        html.contains("Attached"),
        "Examples should include an Attached entry, got: {html}"
    );
}

/// Attached 例の root が `fd-pagination--variant-attached` を出力すること。
#[test]
fn attached_example_renders_variant_class() {
    let html = pagination_page_html();
    assert!(
        html.contains("fd-pagination--variant-attached"),
        "attached example should render the variant class, got: {html}"
    );
}

/// レンダリングが決定的であること（同一入力から 2 回描画してもバイト一致）。
#[test]
fn rendering_is_deterministic() {
    let first = pagination_page_html();
    let second = pagination_page_html();
    assert_eq!(
        first, second,
        "pagination page rendering should be deterministic"
    );
}

/// 生の `<script` を含まないこと（XSS 回帰の明示的固定）。
#[test]
fn pagination_page_contains_no_raw_script_tag() {
    let html = pagination_page_html();
    assert!(
        !html.contains("<script"),
        "pagination page should not contain a raw <script tag, got: {html}"
    );
}

/// `component_specs_nav_data.rs` に `raw_html` が出現しないこと（既存 XSS
/// 回帰テストの走査対象に `component_specs_nav_data.rs` は含まれるが、
/// 本イシュー追加分も崩れていないことを明示的に固定する）。
#[test]
fn pagination_examples_use_no_raw_html() {
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

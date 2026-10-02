//! イシュー #3140（pre-styled-ui の `progress` にマイルストーンの目盛り
//! ラベル `marker_group`/`marker` を追加する）専用の契約テスト。
//!
//! `crates/docs-site/tests/component_pages.rs` /
//! `crates/docs-site/tests/component_page_specs_948.rs` は並列実行される
//! 他イシューも触り得る共有ファイルのため変更しない方針
//! （`crates/docs-site/tests/component_specs_3136.rs` と同じ per-issue
//! テストファイル方式）。本ファイルは `/themes/progress/` 1 ページのみを
//! 検証する: (1) Examples 節に「Milestone labels」の見出しが載ること、(2)
//! `data-part="marker-group"`/`data-part="marker"`/
//! `data-state="under-value"`/`data-state="over-value"` が出力されること、
//! (3) レンダリングが決定的であること、(4) 生の `<script` を含まないこと、
//! (5) 原稿（`component_specs_nav_data.rs`）が `raw_html()` を使わないこと。
//!
//! pre-styled-ui 側の marker-group/marker 実装・golden CSS・単体テストは本
//! イシューの同一 PR 内で完了済みのため、本ファイルは docs-site の
//! Examples 追加分のみを検証対象とする。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::generated_content;

/// `/themes/progress/` の生成 HTML（1 回分）。
fn progress_page_html() -> String {
    let node = generated_content("/themes/progress/")
        .expect("generated_content(\"/themes/progress/\") should be Some");
    render(&node)
}

/// Examples 節に Milestone labels の新しい見出しが含まれること。
#[test]
fn examples_section_includes_milestone_labels_entry() {
    let html = progress_page_html();
    assert!(
        html.contains("<h2>Examples</h2>"),
        "progress page should have an <h2>Examples</h2> section"
    );
    assert!(
        html.contains("Milestone labels"),
        "Examples should include a Milestone labels entry, got: {html}"
    );
}

/// Milestone labels 例が marker-group/marker の `data-part` と、到達済み
/// （under-value）・未到達（over-value）の `data-state` を出力すること。
#[test]
fn milestone_labels_example_renders_marker_data_attrs() {
    let html = progress_page_html();
    assert!(
        html.contains(r#"data-part="marker-group""#),
        "milestone labels example should render marker-group, got: {html}"
    );
    assert!(
        html.contains(r#"data-part="marker""#),
        "milestone labels example should render marker, got: {html}"
    );
    assert!(
        html.contains(r#"data-state="under-value""#),
        "milestone labels example should render an under-value marker, got: {html}"
    );
    assert!(
        html.contains(r#"data-state="over-value""#),
        "milestone labels example should render an over-value marker, got: {html}"
    );
}

/// レンダリングが決定的であること（同一入力から 2 回描画してもバイト一致）。
#[test]
fn rendering_is_deterministic() {
    let first = progress_page_html();
    let second = progress_page_html();
    assert_eq!(
        first, second,
        "progress page rendering should be deterministic"
    );
}

/// 生の `<script` を含まないこと（XSS 回帰の明示的固定）。
#[test]
fn progress_page_contains_no_raw_script_tag() {
    let html = progress_page_html();
    assert!(
        !html.contains("<script"),
        "progress page should not contain a raw <script tag, got: {html}"
    );
}

/// `component_specs_nav_data.rs` に `raw_html` が出現しないこと（既存 XSS
/// 回帰テストの走査対象に `component_specs_nav_data.rs` は含まれるが、
/// 本イシュー追加分も崩れていないことを明示的に固定する）。
#[test]
fn progress_examples_use_no_raw_html() {
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

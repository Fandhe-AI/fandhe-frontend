//! イシュー #3121（select に共通 shape 軸（pill / circle）を追加する）専用の
//! 契約テスト。
//!
//! `crates/docs-site/tests/component_pages.rs` 等の共有ファイルは変更せず、
//! 本ファイルが `/themes/select/` 1 ページのみを検証する（per-issue テスト
//! ファイル方式、`crates/docs-site/tests/component_specs_3118.rs` と同型）。
//!
//! select の pill 形状自体はイシュー #3117（PR #3478）で実装済み、淡色背景
//! variant（`SelectVariant::Subtle`）は本イシューで `crates/pre-styled-ui`
//! へ追加した。本ファイルは docs-site の Examples 追加分を検証する: (1)
//! Examples 節に新しい見出し 2 件が載ること、(2) pill 形状の例が
//! `fd-select--shape-pill` を出力すること、(3) Subtle variant の例が
//! `fd-select--shape-pill` と `fd-select--variant-subtle` を同じ `class` 属性
//! （同一 root 要素）に持つこと、(4) レンダリングが決定的であること、(5)
//! 生の `<script` を含まないこと（既定エスケープの回帰確認）、(6) 原稿
//! （`forms.rs`）が `raw_html()` を使わずノード木 API のみで組み立てられて
//! いること。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::generated_content;

/// `/themes/select/` の生成 HTML（1 回分）。
fn select_page_html() -> String {
    let node = generated_content("/themes/select/")
        .expect("generated_content(\"/themes/select/\") should be Some");
    render(&node)
}

/// Examples 節に pill / Subtle variant の新しい見出しが含まれること。
#[test]
fn examples_section_includes_shape_and_variant_entries() {
    let html = select_page_html();
    assert!(
        html.contains("<h2>Examples</h2>"),
        "select page should have an <h2>Examples</h2> section"
    );
    assert!(
        html.contains("pill 形状（shape 軸）"),
        "Examples should include a pill shape entry, got: {html}"
    );
    assert!(
        html.contains("淡色背景の pill（Subtle variant）"),
        "Examples should include a Subtle variant entry, got: {html}"
    );
}

/// pill 形状の例が `fd-select--shape-pill` を出力すること。
#[test]
fn pill_example_renders_pill_shaped_select() {
    let html = select_page_html();
    assert!(
        html.contains("fd-select--shape-pill"),
        "expected a pill-shaped select, got: {html}"
    );
}

/// Subtle variant の例が `fd-select--shape-pill` と
/// `fd-select--variant-subtle` を同じ `class` 属性へ併記すること
/// （R1371 想定の組み合わせ）。
#[test]
fn subtle_example_combines_pill_shape_and_subtle_variant_on_same_root() {
    let html = select_page_html();
    assert!(
        html.contains(r#"class="fd-select--size-md fd-select--shape-pill fd-select--variant-subtle""#),
        "Subtle variant example should combine shape-pill and variant-subtle on the same root element, got: {html}"
    );
}

/// レンダリングが決定的であること（同一入力から 2 回描画してもバイト一致）。
#[test]
fn rendering_is_deterministic() {
    let first = select_page_html();
    let second = select_page_html();
    assert_eq!(
        first, second,
        "select page rendering should be deterministic"
    );
}

/// 既定エスケープの回帰確認: 生の `<script` がページに出現しないこと。
#[test]
fn page_never_contains_raw_script_tag() {
    let html = select_page_html();
    assert!(
        !html.contains("<script"),
        "select page should never contain a raw <script tag, got: {html}"
    );
}

/// `forms.rs` に `raw_html` が出現しないこと（既存 XSS 回帰テストの走査
/// 対象に `component_specs/` 配下は含まれるが、本イシュー追加分も崩れて
/// いないことを明示的に固定する）。
#[test]
fn select_examples_use_no_raw_html() {
    let source = include_str!("../src/component_specs/forms.rs");
    for (i, line) in source.lines().enumerate() {
        let code_part = match line.find("//") {
            Some(idx) => &line[..idx],
            None => line,
        };
        assert!(
            !code_part.contains("raw_html"),
            "component_specs/forms.rs line {} should not call raw_html(): {line}",
            i + 1
        );
    }
}

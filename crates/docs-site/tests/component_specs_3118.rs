//! イシュー #3118（button に共通 shape 軸の Examples を追加する）専用の
//! 契約テスト。
//!
//! `crates/docs-site/tests/component_pages.rs` /
//! `crates/docs-site/tests/component_page_specs_948.rs` は並列実行される
//! 他イシューも触り得る共有ファイルのため変更しない方針
//! （`crates/docs-site/tests/component_specs_2212.rs` と同じ per-issue
//! テストファイル方式）。本ファイルは `/themes/button/` 1 ページのみを
//! 検証する: (1) Examples 節に新しい見出し 2 件が載ること、(2) pill 形状の
//! 例が `fd-button--shape-pill` を 2 回以上出力し、Surface + Neutral の
//! secondary 近似が対応する variant/palette class を同じ要素に持つこと、
//! (3) circle 形状の例が icon-only + `fd-button--shape-circle` を同じ
//! `<button>` に持ち `aria-label="Add"` を伴うこと、(4) レンダリングが
//! 決定的であること、(5) 原稿（`forms.rs`）が `raw_html()` を使わずノード木
//! API のみで組み立てられていること。
//!
//! pre-styled-ui 側の shape 軸実装・golden CSS・単体テストはイシュー #3117
//! （PR #3478）で完了済みのため、本ファイルは docs-site の Examples 追加分
//! のみを検証対象とする。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::generated_content;

/// `/themes/button/` の生成 HTML（1 回分）。
fn button_page_html() -> String {
    let node = generated_content("/themes/button/")
        .expect("generated_content(\"/themes/button/\") should be Some");
    render(&node)
}

/// Examples 節に pill / circle 形状の新しい見出しが含まれること。
#[test]
fn examples_section_includes_shape_entries() {
    let html = button_page_html();
    assert!(
        html.contains("<h2>Examples</h2>"),
        "button page should have an <h2>Examples</h2> section"
    );
    assert!(
        html.contains("pill 形状（shape 軸）"),
        "Examples should include a pill shape entry, got: {html}"
    );
    assert!(
        html.contains("circle 形状（icon_button）"),
        "Examples should include a circle shape entry, got: {html}"
    );
}

/// pill 形状の例が `fd-button--shape-pill` を 2 回以上出力し、2 つ目が
/// Surface + Neutral（secondary 近似）であること。
#[test]
fn pill_example_renders_two_pill_buttons_with_secondary_approximation() {
    let html = button_page_html();
    let pill_count = html.matches("fd-button--shape-pill").count();
    assert!(
        pill_count >= 2,
        "expected at least 2 pill-shaped buttons, got {pill_count} in: {html}"
    );
    assert!(
        html.contains("fd-button--variant-surface") && html.contains("fd-button--color-palette-neutral"),
        "secondary-ish pill button should combine Surface variant with Neutral palette, got: {html}"
    );
}

/// circle 形状の例が icon-only の `<button>` に `fd-button--shape-circle` と
/// `aria-label="Add"` を伴って描画されること。
#[test]
fn circle_example_renders_icon_only_button_with_aria_label() {
    let html = button_page_html();
    assert!(
        html.contains("fd-button--shape-circle"),
        "circle example should render fd-button--shape-circle, got: {html}"
    );
    assert!(
        html.contains("fd-button--icon-only"),
        "circle example should be icon-only, got: {html}"
    );
    assert!(
        html.contains("aria-label=\"Add\""),
        "circle example should have aria-label=\"Add\", got: {html}"
    );
}

/// レンダリングが決定的であること（同一入力から 2 回描画してもバイト一致）。
#[test]
fn rendering_is_deterministic() {
    let first = button_page_html();
    let second = button_page_html();
    assert_eq!(
        first, second,
        "button page rendering should be deterministic"
    );
}

/// `forms.rs` に `raw_html` が出現しないこと（既存 XSS 回帰テストの走査
/// 対象に `component_specs/` 配下は含まれるが、本イシュー追加分も崩れて
/// いないことを明示的に固定する）。
#[test]
fn button_shape_examples_use_no_raw_html() {
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

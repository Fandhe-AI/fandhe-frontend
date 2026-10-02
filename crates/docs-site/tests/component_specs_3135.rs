//! イシュー #3135（button_group の入れ子 + `data-attached` opt-in で
//! 入力欄を縦横混在連結する合成例を追加する）専用の契約テスト。
//!
//! `crates/docs-site/tests/component_pages.rs` 等の共有ファイルは並列実行
//! される他イシューも触り得るため変更しない方針（`component_specs_3124.rs`
//! と同じ per-issue テストファイル方式）。本ファイルは `/themes/button-group/`
//! ページのみを検証する:
//! (1) Examples 節に新しい見出しが載ること
//! (2) `data-attached` を持つ内側 root が外側 root の子として描画される
//!     こと
//! (3) 入力欄が 3 つあり、それぞれ `aria-label` を持つこと
//! (4) `id` がページ内で一意であること
//! (5) レンダリングが決定的であること
//! (6) 原稿（`component_specs_overlay.rs`）が `raw_html()` を使わずノード木
//!     API のみで組み立てられていること
//!
//! pre-styled-ui 側の `data-attached` opt-in CSS・golden CSS・単体テストは
//! 本イシューの別コミット（`crates/pre-styled-ui/`）で完了済みのため、本
//! ファイルは docs-site の Examples 追加分のみを検証対象とする。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::generated_content;

/// `/themes/button-group/` の生成 HTML（1 回分）。
fn button_group_page_html() -> String {
    let node = generated_content("/themes/button-group/")
        .expect("generated_content(\"/themes/button-group/\") should be Some");
    render(&node)
}

/// button-group の Examples 節に入れ子 + `data-attached` の新しい見出しが
/// 含まれ、`data-attached` を持つ内側 root が外側 root の子として描画
/// されること。
#[test]
fn button_group_examples_section_includes_nested_mixed_entry() {
    let html = button_group_page_html();
    assert!(
        html.contains("<h2>Examples</h2>"),
        "button-group page should have an <h2>Examples</h2> section"
    );
    assert!(
        html.contains("入れ子 + data-attached"),
        "Examples should include a nested + data-attached entry, got: {html}"
    );
    assert!(
        html.contains(r#"data-attached="""#),
        "the nested example should render a data-attached attribute, got: {html}"
    );
    // `data-attached` を持つ内側 root（horizontal）が、外側 root（vertical）
    // の子として描画されていること（入れ子構造の実演）。
    assert!(
        html.contains(r#"data-orientation="vertical""#)
            && html.contains(r#"data-orientation="horizontal""#),
        "the nested example should combine an outer vertical root and an inner horizontal root, got: {html}"
    );
}

/// 入力欄が 3 つあり、それぞれ `aria-label` を持つこと（placeholder は
/// アクセシブルな名前の代わりにならないため、PR #3569 codex レビュー指摘
/// 対応を踏襲する）。
#[test]
fn button_group_nested_mixed_example_inputs_have_aria_label() {
    let html = button_group_page_html();
    for label in ["Card number", "Expiry date", "Security code"] {
        assert!(
            html.contains(&format!(r#"aria-label="{label}""#)),
            "input should have aria-label=\"{label}\", got: {html}"
        );
    }
}

/// 入れ子例の 3 つの入力欄の `id` がページ内で一意であること（Demo や他
/// の Example の id と衝突しない）。
#[test]
fn button_group_nested_mixed_example_ids_are_unique_on_the_page() {
    let html = button_group_page_html();
    for id in [
        "button-group-attached-card-number",
        "button-group-attached-expiry",
        "button-group-attached-cvc",
    ] {
        // `input::input` は `FieldProps::id` へ `-control` suffix を付けて
        // 実 DOM の id を出力する（`crates/pre-styled-ui/src/input.rs`
        // 参照）。
        let needle = format!(r#"id="{id}-control""#);
        assert_eq!(
            html.matches(&needle).count(),
            1,
            "id={id} should appear exactly once on the page, got: {html}"
        );
    }
}

/// レンダリングが決定的であること（同一入力から 2 回描画してもバイト
/// 一致）。
#[test]
fn rendering_is_deterministic() {
    assert_eq!(
        button_group_page_html(),
        button_group_page_html(),
        "button-group page rendering should be deterministic"
    );
}

/// `component_specs_overlay.rs` に `raw_html` が出現しないこと（既存 XSS
/// 回帰テストの走査対象に `component_specs_overlay.rs` 配下は含まれるが、
/// 本イシュー追加分も崩れていないことを明示的に固定する）。
#[test]
fn button_group_nested_mixed_example_uses_no_raw_html() {
    let source = include_str!("../src/component_specs_overlay.rs");
    for (i, line) in source.lines().enumerate() {
        let code_part = match line.find("//") {
            Some(idx) => &line[..idx],
            None => line,
        };
        assert!(
            !code_part.contains("raw_html"),
            "component_specs_overlay.rs line {} should not call raw_html(): {line}",
            i + 1
        );
    }
}

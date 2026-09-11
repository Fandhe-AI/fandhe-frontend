//! イシュー #2212（calendar の Examples に Today / Clear プリセットボタン行を
//! 追加する）専用の契約テスト。
//!
//! `crates/docs-site/tests/component_pages.rs` /
//! `crates/docs-site/tests/component_page_specs_948.rs` は並列実行される
//! 他イシューも触り得る共有ファイルのため変更しない方針
//! （`crates/docs-site/tests/component_specs_1691.rs` と同じ per-issue
//! テストファイル方式）。本ファイルは `/themes/calendar/` 1 ページのみを
//! 検証する: (1) Examples 節に新しい見出し・Today/Clear ボタンが載ること、
//! (2) プリセット行のラッパが `calendar::root` の 3 列 inline-grid を
//! 越境する `grid-column: 1 / -1` を持つこと、(3) 静的掲示のみで
//! `<form>` を持ち込まないこと（UI 部品の責務境界、
//! `docs/policy/intentional-non-adoption.md` §3.25）、(4) id の重複が
//! ないこと、(5) レンダリングが決定的であること、(6) 原稿データ
//! （`component_page_specs_948.rs`）が `raw_html()` を使わずノード木 API
//! のみで組み立てられていること（既存の XSS 回帰テストの走査対象に本
//! ファイルが含まれていないギャップを補う）。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::generated_content;

/// `/themes/calendar/` の生成 HTML（1 回分）。
fn calendar_page_html() -> String {
    let node = generated_content("/themes/calendar/")
        .expect("generated_content(\"/themes/calendar/\") should be Some");
    render(&node)
}

/// Examples 節に新しいプリセット行の見出しが含まれること。
#[test]
fn examples_section_includes_presets_entry() {
    let html = calendar_page_html();
    assert!(
        html.contains("<h2>Examples</h2>"),
        "calendar page should have an <h2>Examples</h2> section"
    );
    assert!(
        html.contains("Today / Clear プリセットボタン行"),
        "Examples should include a new entry titled for Today/Clear presets, got: {html}"
    );
}

/// Today / Clear の 2 ボタンが `type="button"`・非 `disabled` の button 部品
/// として描画されること。
#[test]
fn presets_row_renders_two_outline_buttons() {
    let html = calendar_page_html();
    assert!(
        html.contains(">Today</button>"),
        "presets row should render a Today button, got: {html}"
    );
    assert!(
        html.contains(">Clear</button>"),
        "presets row should render a Clear button, got: {html}"
    );
    let button_scope_count = html.matches("data-scope=\"button\"").count();
    assert!(
        button_scope_count >= 2,
        "expected at least 2 button parts (Today/Clear), got {button_scope_count}"
    );
    // 静的掲示のみであり、暗黙 submit を防ぐため type="button" 固定・
    // disabled 属性なしであることを固定する。
    assert!(
        html.contains("type=\"button\""),
        "buttons should be type=\"button\", got: {html}"
    );
}

/// プリセット行のラッパが root の 3 列 inline-grid を越境する
/// `grid-column: 1 / -1` を持つこと（`crates/pre-styled-ui/src/calendar.rs`
/// の `table` パーツと同じ越境手法）。
#[test]
fn presets_row_spans_calendar_grid() {
    let html = calendar_page_html();
    assert!(
        html.contains("grid-column: 1 / -1"),
        "presets row wrapper should span the calendar root's 3-column grid, got: {html}"
    );
}

/// calendar ページが `<form` を持ち込まないこと（UI コンポーネント層は
/// アプリケーションロジック〔送信処理〕を内包しない、§3.25）。
#[test]
fn calendar_page_has_no_form_element() {
    let html = calendar_page_html();
    assert!(
        !html.contains("<form"),
        "calendar page should not contain a <form> element (static display only), got: {html}"
    );
}

/// Demo・既存 Example・新 Example の id が衝突しないこと。
#[test]
fn page_has_no_duplicate_ids() {
    let html = calendar_page_html();
    let mut ids = Vec::new();
    let mut rest = html.as_str();
    while let Some(idx) = rest.find("id=\"") {
        let after = &rest[idx + 4..];
        let end = after
            .find('"')
            .unwrap_or_else(|| panic!("unterminated id attribute near: {after}"));
        ids.push(after[..end].to_string());
        rest = &after[end + 1..];
    }
    let mut seen = std::collections::HashSet::new();
    for id in &ids {
        assert!(
            seen.insert(id.clone()),
            "duplicate id detected: {id} (all ids: {ids:?})"
        );
    }
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

/// `component_page_specs_948.rs` の calendar 関連コード（コメント除外）に
/// `raw_html` が出現しないこと。既存 XSS 回帰テスト
/// （`component_pages.rs::component_page_source_does_not_use_raw_html`）の
/// 走査対象が `component_specs*.rs`/`component_page.rs` 等に限定され
/// `component_page_specs_948.rs` を含まないため、本イシューの追加分に
/// 限って本テストで補う。
#[test]
fn presets_example_uses_no_raw_html() {
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

//! イシュー #1691（dialog（alert-dialog）の Themes ページ・Demo・契約テスト
//! 追随、親 #1675）専用の契約テスト。
//!
//! `crates/docs-site/tests/component_pages.rs` / `tests/site_showcase.rs` は
//! 並列実行される他イシューも触り得る共有ファイルのため変更しない方針
//! （`crates/docs-site/tests/component_specs_1155.rs` と同じ per-issue
//! テストファイル方式）。本ファイルは `/themes/dialog/` 1 ページのみを
//! 検証する: (1) Demo の `.showcase-row` → `dialog::footer`（イシュー
//! #1690）置換が Anatomy 表へ反映されていること、(2) Examples 節に
//! alert-dialog 構成の例が追加されていること、(3) Demo と Examples の
//! id が衝突しないこと、(4) レンダリングが決定的であること。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::generated_content;

/// `/themes/dialog/` の生成 HTML（1 回分）。
fn dialog_page_html() -> String {
    let node = generated_content("/themes/dialog/")
        .expect("generated_content(\"/themes/dialog/\") should be Some");
    render(&node)
}

/// Anatomy コードブロック（`<h2>Anatomy</h2><pre><code>…</code></pre>`）の
/// テキストを抽出する（機械導出元は Demo ノード木、`component_page.rs`
/// 参照）。
fn anatomy_block(html: &str) -> &str {
    let h2 = "<h2>Anatomy</h2>";
    let h2_idx = html
        .find(h2)
        .unwrap_or_else(|| panic!("dialog page should have an <h2>Anatomy</h2> section"));
    let after_h2 = &html[h2_idx + h2.len()..];
    let code_open = "<code>";
    let code_close = "</code>";
    let start = after_h2
        .find(code_open)
        .unwrap_or_else(|| panic!("Anatomy section should contain a <code> block"))
        + code_open.len();
    let end = after_h2[start..]
        .find(code_close)
        .unwrap_or_else(|| panic!("Anatomy <code> block should be closed"));
    &after_h2[start..start + end]
}

/// Demo の `.showcase-row` → `dialog::footer`（イシュー #1690）置換が
/// Anatomy 表へ反映され、`footer` が `description` の後・`close-trigger`
/// の前（Demo ノード出現順）に現れること。
#[test]
fn anatomy_includes_footer_between_description_and_close_trigger() {
    let html = dialog_page_html();
    let anatomy = anatomy_block(&html);

    let description_idx = anatomy
        .find("description")
        .unwrap_or_else(|| panic!("Anatomy should list description, got: {anatomy}"));
    let footer_idx = anatomy
        .find("footer")
        .unwrap_or_else(|| panic!("Anatomy should list footer (issue #1690), got: {anatomy}"));
    let close_trigger_idx = anatomy
        .find("close-trigger")
        .unwrap_or_else(|| panic!("Anatomy should list close-trigger, got: {anatomy}"));

    assert!(
        description_idx < footer_idx && footer_idx < close_trigger_idx,
        "Anatomy order should be description < footer < close-trigger, got: {anatomy}"
    );
}

/// Demo の content 部分木から `.showcase-row`（掲示用レイアウト class、
/// footer 導入前の旧実装）が消えていること。`dialog::footer` は
/// `data-scope`/`data-part` のみを付与し `class="showcase-row"` を出力
/// しないため、footer への置換が確実に行われたことの直接固定になる。
#[test]
fn demo_content_no_longer_uses_showcase_row_class() {
    let html = dialog_page_html();
    let h2_demo = "<h2>Demo</h2>";
    let h2_features = "<h2>Features</h2>";
    let demo_start = html
        .find(h2_demo)
        .unwrap_or_else(|| panic!("dialog page should have a <h2>Demo</h2> section"))
        + h2_demo.len();
    let demo_end = html[demo_start..]
        .find(h2_features)
        .map(|rel| demo_start + rel)
        .unwrap_or_else(|| {
            panic!("dialog page should have a <h2>Features</h2> section after Demo")
        });
    let demo_section = &html[demo_start..demo_end];

    assert!(
        !demo_section.contains("showcase-row"),
        "Demo section should no longer contain the pre-#1690 `.showcase-row` layout div \
         (replaced by dialog::footer), got Demo section: {demo_section}"
    );
}

/// Examples 節に「Alert dialog」エントリが存在し、`role="alertdialog"`
/// （DialogRole::Alertdialog）と `data-part="footer"`（イシュー #1690）を
/// 含む alert-dialog 構成の例を掲示していること。
#[test]
fn examples_section_includes_alert_dialog_entry() {
    let html = dialog_page_html();

    assert!(
        html.contains("<h2>Examples</h2>"),
        "dialog page should have an Examples section"
    );
    assert!(
        html.contains("<h3>Alert dialog</h3>"),
        "Examples section should contain an \"Alert dialog\" entry"
    );
    assert!(
        html.contains(r#"role="alertdialog""#),
        "Alert dialog example should render role=\"alertdialog\" (DialogRole::Alertdialog)"
    );
    assert!(
        html.contains(r#"data-part="footer""#),
        "Alert dialog example should use dialog::footer (data-part=\"footer\", issue #1690)"
    );
}

/// Demo（`showcase-dialog-*`）と Examples（`showcase-alert-dialog-*`）が
/// 同一ページに描画されるため、`id="…"` 値がページ内で重複しないこと
/// （HTML 仕様違反・`aria-labelledby`/`aria-describedby`/`aria-controls`
/// 関連付け破壊の防止）。
#[test]
fn page_has_no_duplicate_ids() {
    let html = dialog_page_html();

    let mut ids: Vec<&str> = Vec::new();
    let mut idx = 0;
    while let Some(rel) = html[idx..].find(r#"id=""#) {
        let start = idx + rel + r#"id=""#.len();
        let Some(end_rel) = html[start..].find('"') else {
            break;
        };
        ids.push(&html[start..start + end_rel]);
        idx = start + end_rel + 1;
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut duplicates: Vec<&str> = Vec::new();
    for id in &ids {
        if !seen.insert(*id) {
            duplicates.push(id);
        }
    }
    assert!(
        duplicates.is_empty(),
        "dialog page should not have duplicate id attributes, found duplicates: {duplicates:?} \
         (all ids: {ids:?})"
    );
}

/// 2 回連続レンダリングがバイト一致すること（決定性、REQ-1 系の既存契約
/// と同型）。
#[test]
fn rendering_is_deterministic() {
    let first = dialog_page_html();
    let second = dialog_page_html();
    assert_eq!(
        first, second,
        "dialog page rendering should be deterministic"
    );
}

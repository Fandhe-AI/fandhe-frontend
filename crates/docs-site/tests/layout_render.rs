//! `fandhe-frontend-docs-site::layout` の統合テスト（イシュー #469）。
//!
//! 受け入れ条件（完全文書組み立て・見出しアンカー抽出・アセットパス正規化）
//! と、XSS 回帰・決定性（REQ-6 のモード非依存性契約に倣う）を検証する。
//! `fandhe_frontend_server::ssg::generate_pages()` が `render()` 結果へ
//! `<!DOCTYPE html>` を前置する契約であるため、本テストは `layout::docs_page`
//! が返す `Node` に対する `render()` 出力のみを検証し DOCTYPE の有無は
//! 検証しない（DOCTYPE 前置は #470 でエントリ接続後に検証する）。

use fandhe_frontend_core::{h2, h3, li, p, render, text, ul};
use fandhe_frontend_docs_site::layout::{asset_href, docs_page, toc_nav, with_heading_anchors};

fn sample_sidebar() -> fandhe_frontend_core::Node {
    ul(vec![], vec![li(vec![], vec![text("はじめに")])])
}

#[test]
fn docs_page_renders_a_single_complete_document() {
    let body = p(vec![], vec![text("本文です。")]);
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);

    assert!(html.starts_with("<html lang=\"ja\">"));
    assert!(html.contains("<head>"));
    assert!(html.contains("<title>タイトル</title>"));
    assert!(html.contains("はじめに"));
    assert!(html.contains("本文です。"));
    assert!(html.contains(r#"class="docs-sidebar""#));
    assert!(html.contains(r#"class="docs-content""#));
    assert!(html.contains(r#"href="/assets/site.css""#));
}

#[test]
fn heading_anchors_are_extracted_in_document_order_with_correct_levels() {
    let body = fandhe_frontend_core::div(
        vec![],
        vec![
            h2(vec![], vec![text("導入")]),
            p(vec![], vec![text("前置き")]),
            h3(vec![], vec![text("詳細")]),
        ],
    );
    let (annotated, entries) = with_heading_anchors(body);

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].level, 2);
    assert_eq!(entries[0].title, "導入");
    assert_eq!(entries[1].level, 3);
    assert_eq!(entries[1].title, "詳細");

    let html = render(&annotated);
    assert!(html.contains(&format!(r#"<h2 id="{}">導入</h2>"#, entries[0].id)));
    assert!(html.contains(&format!(r#"<h3 id="{}">詳細</h3>"#, entries[1].id)));
}

#[test]
fn duplicate_heading_titles_get_deterministic_unique_ids() {
    let body = fandhe_frontend_core::div(
        vec![],
        vec![
            h2(vec![], vec![text("概要")]),
            h2(vec![], vec![text("概要")]),
        ],
    );
    let (_, entries) = with_heading_anchors(body);

    assert_eq!(entries[0].id, "概要");
    assert_eq!(entries[1].id, "概要-2");
    assert_ne!(entries[0].id, entries[1].id);
}

#[test]
fn existing_heading_id_is_respected_and_not_overwritten() {
    let body = fandhe_frontend_core::div(
        vec![],
        vec![fandhe_frontend_core::el(
            "h2",
            vec![("id", "custom-anchor")],
            vec![text("見出し")],
        )],
    );
    let (annotated, entries) = with_heading_anchors(body);

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, "custom-anchor");
    let html = render(&annotated);
    assert!(html.contains(r#"<h2 id="custom-anchor">見出し</h2>"#));
}

#[test]
fn raw_html_children_are_not_concatenated_into_heading_title() {
    // docs-site クレートは raw_html() を使わない方針だが、混入時でも
    // TOC タイトルへ生 HTML 断片を取り込まない防御的実装を検証する。
    // raw_html() 呼び出しは clippy::disallowed_methods 対象のため、ここでは
    // 検証対象の `Node::RawHtml` バリアントを直接構築する（呼び出し経路の
    // レビューを要さない、列挙子の直接構築）。
    let body = fandhe_frontend_core::el(
        "h2",
        vec![],
        vec![
            text("見出し"),
            fandhe_frontend_core::Node::RawHtml("<b>強調</b>".to_string()),
        ],
    );
    let (_, entries) = with_heading_anchors(body);

    assert_eq!(entries[0].title, "見出し");
}

#[test]
fn no_headings_means_no_toc_nav_and_no_toc_section_in_document() {
    let body = p(vec![], vec![text("見出しのない本文")]);
    let (annotated, entries) = with_heading_anchors(body.clone());
    assert!(entries.is_empty());
    assert!(toc_nav(&entries).is_none());

    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);
    assert!(!html.contains(r#"class="docs-toc""#));
    let _ = annotated;
}

#[test]
fn toc_nav_links_use_anchor_hrefs_matching_injected_ids() {
    let body = fandhe_frontend_core::div(vec![], vec![h2(vec![], vec![text("導入")])]);
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);

    assert!(html.contains(r#"class="docs-toc""#));
    // id 属性値と一致する #<id> アンカーが目次に出力されること。
    let id_marker = r#"<h2 id=""#;
    let start = html.find(id_marker).expect("h2 with injected id");
    let after = &html[start + id_marker.len()..];
    let end = after.find('"').expect("closing quote of id attr");
    let id = &after[..end];
    assert!(html.contains(&format!("href=\"#{id}\"")));
}

#[test]
fn asset_href_normalizes_base_path_variants() {
    assert_eq!(asset_href("", "assets/site.css"), "/assets/site.css");
    assert_eq!(
        asset_href("/fandhe-frontend", "assets/site.css"),
        "/fandhe-frontend/assets/site.css"
    );
    assert_eq!(
        asset_href("/fandhe-frontend/", "assets/site.css"),
        "/fandhe-frontend/assets/site.css"
    );
    assert_eq!(asset_href("", ""), "/");
    assert_eq!(asset_href("/fandhe-frontend", ""), "/fandhe-frontend/");
}

#[test]
fn docs_page_output_is_deterministic_for_identical_input() {
    let make = || {
        let body = fandhe_frontend_core::div(
            vec![],
            vec![
                h2(vec![], vec![text("導入")]),
                p(vec![], vec![text("本文")]),
            ],
        );
        docs_page("タイトル", "/fandhe-frontend", sample_sidebar(), body)
    };
    assert_eq!(render(&make()), render(&make()));
}

#[test]
fn xss_payloads_in_title_headings_and_sidebar_are_escaped() {
    let payload = "<script>alert(1)</script>";
    let attr_payload = "\"><img src=x onerror=alert(1)>";

    let sidebar = ul(vec![], vec![li(vec![], vec![text(attr_payload)])]);
    let body = fandhe_frontend_core::div(vec![], vec![h2(vec![], vec![text(payload)])]);
    let node = docs_page(payload, "", sidebar, body);
    let html = render(&node);

    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(!html.contains("<img src=x onerror=alert(1)>"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(html.contains("&quot;&gt;&lt;img"));

    // 悪意ある見出しから生成した slug は英数字と `-` のみに正規化され、
    // 属性値エスケープを経由する（id 属性は render_into 側で常に
    // escape_html_into を通る。生成 slug 自体に `"` `<` `>` を含まない）。
    let (_, entries) = with_heading_anchors(fandhe_frontend_core::div(
        vec![],
        vec![h2(vec![], vec![text(payload)])],
    ));
    let id = &entries[0].id;
    assert!(id.chars().all(|c| c.is_alphanumeric() || c == '-'));
}

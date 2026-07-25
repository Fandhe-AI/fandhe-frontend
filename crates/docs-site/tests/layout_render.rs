//! `fandhe-frontend-docs-site::layout` の統合テスト（イシュー #469）。
//!
//! 受け入れ条件（完全文書組み立て・見出しアンカー抽出・アセットパス正規化）
//! と、XSS 回帰・決定性（REQ-6 のモード非依存性契約に倣う）を検証する。
//! `fandhe_frontend_server::ssg::generate_pages()` が `render()` 結果へ
//! `<!DOCTYPE html>` を前置する契約であるため、本テストは `layout::docs_page`
//! が返す `Node` に対する `render()` 出力のみを検証し DOCTYPE の有無は
//! 検証しない（DOCTYPE 前置は #470 でエントリ接続後に検証する）。

use fandhe_frontend_core::{h2, h3, li, p, render, text, ul};
use fandhe_frontend_docs_site::layout::{
    asset_href, docs_page, docs_page_with_assets, toc_nav, with_heading_anchors,
};
use fandhe_frontend_docs_site::nav::{header_nav, parse_nav};
use fandhe_frontend_docs_site::script;

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

/// イシュー #776: SkipNav の `link` は `<body>` 先頭（`docs-header` より前）、
/// `content`（スキップ先ターゲット）は `main` 内の本文（`docs-content`）
/// より前に出力される。専用 CSS（`assets/skip-nav.css`）への `<link>` も
/// 全ページへ無条件に付与される（`crate::skip_nav` モジュール doc 参照）。
#[test]
fn docs_page_inserts_skip_nav_link_before_header_and_content_before_main_body() {
    let body = p(vec![], vec![text("本文です。")]);
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);

    assert!(html.contains(r#"data-scope="skip-nav""#));
    assert!(html.contains(r#"href="/assets/skip-nav.css""#));

    let body_start = html.find("<body>").expect("body tag should exist");
    let skip_link_pos = html
        .find(r#"data-part="link""#)
        .expect("skip-nav link should exist");
    let header_pos = html
        .find(r#"class="docs-header""#)
        .expect("header should exist");
    let skip_content_pos = html
        .find(r#"data-part="content""#)
        .expect("skip-nav content target should exist");
    let article_pos = html
        .find(r#"class="docs-content""#)
        .expect("article should exist");

    assert!(
        body_start < skip_link_pos,
        "skip-nav link should be inside body"
    );
    assert!(
        skip_link_pos < header_pos,
        "skip-nav link should precede docs-header"
    );
    assert!(
        skip_content_pos < article_pos,
        "skip-nav content target should precede the docs-content article"
    );
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
fn headings_inside_data_scope_subtrees_are_excluded_from_anchors_and_toc() {
    // headless-ui コンポーネントの anatomy（`data-scope` 属性を持つ要素）
    // 配下の見出しは部品構造であり文書アウトラインではない（Accordion の
    // item trigger を包む h3 等）。アンカー注入も TOC 収集も行わないことを
    // 固定する（showcase ページの TOC 混入回帰防止、Bugbot 指摘）。
    let body = fandhe_frontend_core::div(
        vec![],
        vec![
            h2(vec![], vec![text("Accordion")]),
            fandhe_frontend_core::div(
                vec![("data-scope", "accordion"), ("data-part", "item")],
                vec![h3(vec![], vec![text("trigger の質問見出し")])],
            ),
        ],
    );
    let (annotated, entries) = with_heading_anchors(body);

    // TOC はセクション見出し（h2）のみ。部品内 h3 は収集されない。
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].level, 2);
    assert_eq!(entries[0].title, "Accordion");

    // 部品内 h3 には id が注入されず、元のマークアップのまま保たれる。
    let html = render(&annotated);
    assert!(html.contains("<h3>trigger の質問見出し</h3>"));
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
fn existing_id_colliding_with_autogenerated_slug_is_made_unique() {
    // 著者指定 id が既に自動生成スラグに確保済みの値と衝突するケース
    // （Cursor Bugbot 指摘 BUGBOT_BUG_ID: 6aa791a9-b7d6-4155-843e-3814b6b74504）。
    // 衝突を検出せず両見出しが同一 id を持つと、TOC・静的 `#...` リンクが
    // 最初の見出ししか指さなくなる。
    let body = fandhe_frontend_core::div(
        vec![],
        vec![
            h2(vec![], vec![text("概要")]),
            fandhe_frontend_core::el("h2", vec![("id", "概要")], vec![text("別の概要")]),
        ],
    );
    let (annotated, entries) = with_heading_anchors(body);

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].id, "概要");
    // 著者指定 id は尊重されつつ、衝突時のみ一意化される。
    assert_ne!(entries[1].id, "概要");
    assert!(entries[1].id.starts_with("概要-"));

    let html = render(&annotated);
    assert!(html.contains(&format!(r#"<h2 id="{}">概要</h2>"#, entries[0].id)));
    assert!(html.contains(&format!(r#"<h2 id="{}">別の概要</h2>"#, entries[1].id)));
    // 衝突後の id 属性は 1 つのみ出力されること（重複属性が残らないこと）。
    let second_open = html.rfind("<h2 id=").expect("second heading tag");
    assert_eq!(html[second_open..].matches(" id=").count(), 1);
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
    // イシュー #907: 見出しの無いページでは右目次カラム（第 3 子 aside）自体を
    // 出力しない（設計文書 §3.3 の方針）。
    assert!(!html.contains(r#"class="docs-toc-aside""#));
    // Bugbot 指摘（PR #916）是正の回帰テスト: `aside.docs-toc-aside` が無い
    // ページの `div.docs-container` には `docs-container--no-toc` 修飾 class
    // を付与し、`min-width: 1200px` の 3 カラム grid で右目次列のグリッド
    // トラックを収縮させる（`crate::site_theme::STRUCTURAL_CSS` 参照）。
    assert!(html.contains(r#"class="docs-container docs-container--no-toc""#));
    let _ = annotated;
}

/// [`no_headings_means_no_toc_nav_and_no_toc_section_in_document`] の対:
/// 見出しが存在するページでは `docs-container--no-toc` 修飾 class を付与
/// しない（Bugbot 指摘、PR #916 是正）。
#[test]
fn headings_present_means_container_has_no_toc_modifier_class() {
    let body = fandhe_frontend_core::div(
        vec![],
        vec![
            h2(vec![], vec![text("見出し")]),
            p(vec![], vec![text("本文")]),
        ],
    );
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);
    assert!(html.contains(r#"class="docs-toc-aside""#));
    assert!(html.contains(r#"class="docs-container""#));
    assert!(!html.contains("docs-container--no-toc"));
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

/// イシュー #907: 3 カラム骨格の DOM 出現順（左ナビ / 中央コンテンツ /
/// 右目次）を固定する回帰テスト。設計文書 §3.1/§3.3 の「`nav.docs-toc` を
/// `aside.docs-toc-aside` として `main.docs-main` の外（第 3 子）へ移設する」
/// 変更に伴い、`docs-sidebar` < `docs-content` < `docs-toc-aside` の順で
/// 出現することを検証する。
#[test]
fn docs_page_emits_three_columns_in_left_nav_center_content_right_toc_order() {
    let body = fandhe_frontend_core::div(vec![], vec![h2(vec![], vec![text("導入")])]);
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);

    let sidebar_pos = html
        .find(r#"class="docs-sidebar""#)
        .expect("docs-sidebar should exist");
    let content_pos = html
        .find(r#"class="docs-content""#)
        .expect("docs-content should exist");
    let toc_aside_pos = html
        .find(r#"class="docs-toc-aside""#)
        .expect("docs-toc-aside should exist when headings are present");
    let toc_nav_pos = html
        .find(r#"class="docs-toc""#)
        .expect("docs-toc nav should exist inside the toc aside");

    assert!(
        sidebar_pos < content_pos,
        "left nav column should precede center content column"
    );
    assert!(
        content_pos < toc_aside_pos,
        "center content column should precede right toc column"
    );
    assert!(
        toc_aside_pos < toc_nav_pos,
        "nav.docs-toc should be nested inside aside.docs-toc-aside"
    );
}

/// イシュー #907（レビュー指摘、commit e01a23d）: `< 768px` の左ナビ折りたたみを
/// タッチ操作でも開閉できるようにするチェックボックスハック
/// （`input#docs-sidebar-toggle` + `label[for=docs-sidebar-toggle]`）の
/// markup・id/for 紐付け・DOM 順を固定する回帰テスト。`site.css` の CSS
/// 一般兄弟結合子 `.docs-sidebar-toggle:checked ~ nav.sidebar` が機能する
/// ためには `input` が `label`・`nav`（`sidebar` 引数のルート要素）より
/// 先に出現する必要があり、markup の並び順が誤って変更された場合に
/// この回帰テストが検知する。
#[test]
fn docs_sidebar_toggle_checkbox_and_label_are_wired_before_sidebar_nav() {
    let body = p(vec![], vec![text("本文です。")]);
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);

    assert!(html.contains(r#"type="checkbox""#));
    assert!(html.contains(r#"id="docs-sidebar-toggle""#));
    assert!(html.contains(r#"class="docs-sidebar-toggle""#));
    assert!(html.contains(r#"for="docs-sidebar-toggle""#));
    assert!(html.contains(r#"class="docs-sidebar-toggle-label""#));

    let toggle_pos = html
        .find(r#"id="docs-sidebar-toggle""#)
        .expect("sidebar toggle checkbox should exist");
    let label_pos = html
        .find(r#"for="docs-sidebar-toggle""#)
        .expect("sidebar toggle label should exist");
    let sidebar_nav_pos = html
        .find("はじめに")
        .expect("sidebar nav content should exist");

    assert!(
        toggle_pos < label_pos,
        "checkbox input must precede its label for the CSS general sibling combinator to apply"
    );
    assert!(
        label_pos < sidebar_nav_pos,
        "label must precede nav.sidebar so `.docs-sidebar-toggle:checked ~ nav.sidebar` matches"
    );
}

#[test]
fn toc_nav_items_carry_level_class_distinguishing_h2_and_h3() {
    // Bugbot 指摘 b0e41098: toc_nav が TocEntry::level を無視してフラットな
    // <li> を出すと h2/h3 の階層がマークアップから読み取れなくなる。
    // レベルクラス（docs-toc-level-2 / docs-toc-level-3）で区別できることを
    // 確認する回帰テスト。
    let body = fandhe_frontend_core::div(
        vec![],
        vec![
            h2(vec![], vec![text("導入")]),
            h3(vec![], vec![text("背景")]),
        ],
    );
    let (_, entries) = with_heading_anchors(body.clone());
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].level, 2);
    assert_eq!(entries[1].level, 3);

    let toc = toc_nav(&entries).expect("toc_nav must return Some for non-empty entries");
    let html = render(&toc);
    assert!(html.contains(r#"class="docs-toc-level-2""#));
    assert!(html.contains(r#"class="docs-toc-level-3""#));
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

// ---- ヘッダーナビ（イシュー #908） ----

/// `docs_page`（`header_nav` を渡さない従来経路）でもブランドリンクに
/// `docs-brand` class が付き、ヘッダーナビ（`docs-header-nav`）は出力
/// されないことを固定する。
#[test]
fn docs_page_without_header_nav_has_brand_class_and_no_header_nav() {
    let body = p(vec![], vec![text("本文です。")]);
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);

    assert!(html.contains(r#"class="docs-brand""#));
    assert!(!html.contains("docs-header-nav"));
}

fn sample_nav_toml() -> &'static str {
    r#"
[site]
title = "Fixture"
base_path = ""

[[section]]
title = "Getting Started"

[[section.page]]
title = "Intro"
source = "site/index.md"
path = "/"

[[section]]
title = "Guides"

[[section.page]]
title = "Advanced"
source = "site/index.md"
path = "/advanced/"
"#
}

/// `docs_page_with_assets(..., Some(header_nav))` で `a.docs-brand` →
/// `nav.docs-header-nav` の順に header 内へ出力されることを固定する
/// （設計文書 §3.5 の DOM 契約）。
#[test]
fn docs_page_with_assets_places_brand_before_header_nav_inside_header() {
    let nav = parse_nav(sample_nav_toml()).expect("fixture nav.toml should parse");
    let body = p(vec![], vec![text("本文です。")]);
    let node = docs_page_with_assets(
        "タイトル",
        "",
        sample_sidebar(),
        body,
        &[],
        Some(header_nav(&nav, "/")),
    );
    let html = render(&node);

    let header_start = html
        .find(r#"class="docs-header""#)
        .expect("docs-header should exist");
    let brand_pos = html
        .find(r#"class="docs-brand""#)
        .expect("docs-brand should exist");
    let header_nav_pos = html
        .find(r#"class="docs-header-nav""#)
        .expect("docs-header-nav should exist");

    assert!(
        header_start < brand_pos,
        "brand link should be inside header"
    );
    assert!(
        brand_pos < header_nav_pos,
        "brand link should precede header nav within the header"
    );

    // セクションタイトル・ページタイトルが両方出力される。
    assert!(html.contains("Getting Started"));
    assert!(html.contains("Advanced"));
}

/// SkipNav リンクは `header_nav` を渡してもなお header より前に残る
/// （既存 SkipNav 不変条件、イシュー #776 が固定した DOM 順の維持確認）。
#[test]
fn docs_page_with_assets_keeps_skip_nav_before_header_when_header_nav_present() {
    let nav = parse_nav(sample_nav_toml()).expect("fixture nav.toml should parse");
    let body = p(vec![], vec![text("本文です。")]);
    let node = docs_page_with_assets(
        "タイトル",
        "",
        sample_sidebar(),
        body,
        &[],
        Some(header_nav(&nav, "/")),
    );
    let html = render(&node);

    let skip_link_pos = html
        .find(r#"data-part="link""#)
        .expect("skip-nav link should exist");
    let header_pos = html
        .find(r#"class="docs-header""#)
        .expect("header should exist");

    assert!(
        skip_link_pos < header_pos,
        "skip-nav link should still precede docs-header"
    );
}

// ---- View Transitions（イシュー #912 回帰検証） ----

/// `docs_page` が `<head>` へ View Transitions の opt-in 宣言
/// （`@view-transition { navigation: auto; }`）を無条件で出力することを
/// 固定する（`crate::layout::docs_page_with_assets` 参照）。この静的テストは
/// opt-in 宣言の**存在**のみを固定し、遷移が実際に走るかは実ブラウザ確認の
/// 責務（`docs/reports/docs-site-redesign-regression-report.md` 参照）。
#[test]
fn docs_page_emits_view_transition_opt_in_style_in_head() {
    let body = p(vec![], vec![text("本文です。")]);
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);

    assert!(html.contains("<style>@view-transition { navigation: auto; }</style>"));
    let head_end = html.find("</head>").expect("head should exist");
    let style_pos = html
        .find("@view-transition { navigation: auto; }")
        .expect("view-transition opt-in style should exist");
    assert!(
        style_pos < head_end,
        "view-transition opt-in style should be inside <head>"
    );
}

/// [`docs_page_emits_view_transition_opt_in_style_in_head`] の対:
/// `docs_page_with_assets`（ショーケースページ等、`extra_stylesheets`・
/// `header_nav` を渡す経路）でも同じ opt-in 宣言が出ることを固定する
/// （配線分岐で opt-in が抜け落ちる回帰の防止）。
#[test]
fn docs_page_with_assets_emits_view_transition_opt_in_style_in_head() {
    let nav = parse_nav(sample_nav_toml()).expect("fixture nav.toml should parse");
    let body = p(vec![], vec![text("本文です。")]);
    let node = docs_page_with_assets(
        "タイトル",
        "",
        sample_sidebar(),
        body,
        &["assets/pre-styled-ui.css"],
        Some(header_nav(&nav, "/")),
    );
    let html = render(&node);

    assert!(html.contains("<style>@view-transition { navigation: auto; }</style>"));
}

// ---- SkipNav の href/id 対応・フォーカス順（イシュー #912 回帰検証） ----

/// SkipNav の `link` の `href` が `content` の `id` と一致し、`content` に
/// `tabindex="-1"` が付くことを固定する（`ps_skip_nav::DEFAULT_ID` の配線
/// ずれ・href の取り違えを検知する）。
#[test]
fn docs_page_skip_nav_link_href_matches_content_target_id() {
    let body = p(vec![], vec![text("本文です。")]);
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);

    // `link` の href（`#<id>`）を実出力から抽出し、`content` の `id` 属性値
    // へ実際に対応していることを検証する（`ps_skip_nav::DEFAULT_ID` を
    // 定数として二重にハードコードせず、実出力どうしの整合を見る）。
    let href_marker = "href=\"#";
    let href_start = html
        .find(href_marker)
        .expect("skip-nav link href should exist")
        + href_marker.len();
    let href_rest = &html[href_start..];
    let href_end = href_rest.find('"').expect("closing quote of href attr");
    let target_id = &href_rest[..href_end];

    assert!(
        html.contains(&format!(r#"id="{target_id}""#)),
        "content target should carry an id matching the skip-nav link href (#{target_id})"
    );
    assert!(html.contains(r#"tabindex="-1""#));
}

/// SkipNav の `link` が `<body>` 内で最初のフォーカス可能要素であることを
/// 固定する（WCAG 2.1 SC 2.4.1「ブロックのスキップ」。`header_nav` の
/// 有無双方で検証する）。
#[test]
fn docs_page_skip_nav_link_is_first_focusable_element_in_body() {
    fn assert_skip_link_is_first_focusable(html: &str) {
        let body_start = html.find("<body>").expect("body tag should exist") + "<body>".len();
        // `data-part="link"` の位置ではなく開始タグそのもの（`<a `）の位置を
        // 使う: 属性出力順は `data-scope`→`data-part`→`href`
        // （`fandhe_frontend_headless_ui::anatomy::Anatomy::part` 参照）のため
        // `data-part="link"` は skip-nav リンク自身のタグの**内部**に現れる。
        // これを境界に使うと skip-nav リンク自身の `<a ` が「先行するフォーカス
        // 可能要素」として誤検知（偽陽性）される。
        let skip_link_pos = html
            .find(r#"<a data-scope="skip-nav" data-part="link""#)
            .expect("skip-nav link tag should exist");
        let between = &html[body_start..skip_link_pos];
        for needle in [
            "<a ",
            "<button",
            "<input",
            "<select",
            "<textarea",
            "tabindex=",
        ] {
            assert!(
                !between.contains(needle),
                "no focusable element ({needle:?}) should precede the skip-nav link in body"
            );
        }
    }

    let body = p(vec![], vec![text("本文です。")]);
    let node_without_header_nav = docs_page("タイトル", "", sample_sidebar(), body.clone());
    assert_skip_link_is_first_focusable(&render(&node_without_header_nav));

    let nav = parse_nav(sample_nav_toml()).expect("fixture nav.toml should parse");
    let node_with_header_nav = docs_page_with_assets(
        "タイトル",
        "",
        sample_sidebar(),
        body,
        &[],
        Some(header_nav(&nav, "/")),
    );
    assert_skip_link_is_first_focusable(&render(&node_with_header_nav));
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

// ---- テーマトグル・GitHub リンク（イシュー #951） ----

/// 最重要のエスケープ往復検証: `render()` 結果が
/// `script::INLINE_THEME_BOOTSTRAP` を**逐語で**含む。これが破れる場合
/// （実体参照化される等）は `<script>` の中身が壊れて構文エラーになる
/// ことを意味する（`crate::script` モジュール doc の不変条件参照）。
#[test]
fn docs_page_head_contains_inline_theme_bootstrap_verbatim_and_unescaped() {
    let body = p(vec![], vec![text("本文です。")]);
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);

    assert!(html.contains(script::INLINE_THEME_BOOTSTRAP));
    // エスケープ痕跡（実体参照化された `<script>` の中身）が残っていない
    // ことも併せて確認する。
    assert!(!html.contains("&#x27;"));
    assert!(!html.contains("&amp;"));
    assert!(!html.contains("&quot;"));

    let script_start = html
        .find(script::INLINE_THEME_BOOTSTRAP)
        .expect("INLINE_THEME_BOOTSTRAP should appear in <head>");
    let head_end = html.find("</head>").expect("</head> should exist");
    assert!(
        script_start < head_end,
        "インライン script は </head> より前に出力される必要がある"
    );
}

/// `<script src="…/assets/site.js" defer>` が `<head>` に出力される
/// （`base_path` なし・あり双方）。`base_path` 付きでは
/// `layout::asset_href` が単一実装点として `/fandhe-frontend/assets/site.js`
/// を組み立てることを固定する。
#[test]
fn docs_page_head_contains_deferred_site_js_script_tag() {
    let body = p(vec![], vec![text("本文です。")]);

    let html = render(&docs_page("タイトル", "", sample_sidebar(), body.clone()));
    assert!(html.contains(r#"<script src="/assets/site.js" defer="">"#));

    let html_with_base = render(&docs_page(
        "タイトル",
        "/fandhe-frontend",
        sample_sidebar(),
        body,
    ));
    assert!(html_with_base.contains(r#"<script src="/fandhe-frontend/assets/site.js" defer="">"#));
}

/// テーマトグル `button` の必須属性（`type="button"` / 既定 `hidden` /
/// `aria-label` / `aria-pressed="false"`）と、GitHub リンクの `href`・
/// `rel="noopener noreferrer"`（tabnabbing 対策、OWASP A05）を固定する。
#[test]
fn docs_page_header_actions_have_required_attributes() {
    let body = p(vec![], vec![text("本文です。")]);
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);

    assert!(html.contains(r#"class="docs-theme-toggle""#));
    assert!(html.contains(r#"type="button""#));
    assert!(html.contains(r#"hidden="""#));
    assert!(html.contains(r#"aria-label="Toggle color theme""#));
    assert!(html.contains(r#"aria-pressed="false""#));

    assert!(html.contains(r#"class="docs-github-link""#));
    assert!(html.contains(r#"href="https://github.com/Fandhe-AI/fandhe-frontend""#));
    assert!(html.contains(r#"target="_blank""#));
    assert!(html.contains(r#"rel="noopener noreferrer""#));
}

/// ヘッダー内 DOM 順: `docs-brand` < `docs-header-nav`（`header_nav` あり）
/// < `docs-header-actions`。`header_nav` が `None` の場合でも
/// `docs-brand` < `docs-header-actions` の順は不変（`docs_page`
/// （従来経路）でも 3 要素すべてが出現する）。
#[test]
fn docs_page_header_dom_order_places_actions_after_brand_and_nav() {
    let body = p(vec![], vec![text("本文です。")]);

    let node_without_header_nav = docs_page("タイトル", "", sample_sidebar(), body.clone());
    let html_without = render(&node_without_header_nav);
    let brand_pos = html_without
        .find(r#"class="docs-brand""#)
        .expect("docs-brand should appear");
    let actions_pos = html_without
        .find(r#"class="docs-header-actions""#)
        .expect("docs-header-actions should appear even without header_nav");
    assert!(brand_pos < actions_pos);
    assert!(html_without.contains(r#"class="docs-github-link""#));
    assert!(html_without.contains(r#"class="docs-theme-toggle""#));

    let nav = parse_nav(sample_nav_toml()).expect("fixture nav.toml should parse");
    let node_with_header_nav = docs_page_with_assets(
        "タイトル",
        "",
        sample_sidebar(),
        body,
        &[],
        Some(header_nav(&nav, "/")),
    );
    let html_with = render(&node_with_header_nav);
    let brand_pos = html_with
        .find(r#"class="docs-brand""#)
        .expect("docs-brand should appear");
    let nav_pos = html_with
        .find(r#"class="docs-header-nav""#)
        .expect("docs-header-nav should appear");
    let actions_pos = html_with
        .find(r#"class="docs-header-actions""#)
        .expect("docs-header-actions should appear");
    assert!(brand_pos < nav_pos);
    assert!(nav_pos < actions_pos);
}

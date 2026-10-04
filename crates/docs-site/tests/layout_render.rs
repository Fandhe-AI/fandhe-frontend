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
    asset_href, docs_page, docs_page_with_assets, docs_page_with_layout, toc_inline, toc_nav,
    with_heading_anchors, PageLayout, TocEntry, TOC_HEADING_ID,
};
use fandhe_frontend_docs_site::nav::{header_nav, parse_nav};
use fandhe_frontend_docs_site::search_index;

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

/// イシュー #3678: meta CSP は charset・viewport の直後、`title` と最初の
/// `link`/`script` より前にちょうど 1 個出る（Docs・Landing 共通）。タイトルへ
/// 注入を試みても CSP meta は変わらない。
#[test]
fn csp_meta_is_emitted_once_right_after_viewport_for_docs_and_landing() {
    use fandhe_frontend_docs_site::csp::CONTENT_SECURITY_POLICY;
    let expected = format!(
        r#"<meta http-equiv="Content-Security-Policy" content="{}">"#,
        fandhe_frontend_core::escape_html(CONTENT_SECURITY_POLICY)
    );
    let body = p(vec![], vec![text("本文です。")]);
    let docs = render(&docs_page(
        r#""><script>x</script>"#,
        "",
        sample_sidebar(),
        body.clone(),
    ));
    let landing = render(&docs_page_with_layout(
        "T",
        "",
        sample_sidebar(),
        body,
        &[],
        None,
        None,
        None,
        PageLayout::Landing,
    ));
    for html in [docs, landing] {
        assert_eq!(html.matches(&expected).count(), 1);
        let viewport = html.find(r#"<meta name="viewport""#).unwrap();
        let csp = html.find(&expected).unwrap();
        assert!(viewport < csp);
        assert!(csp < html.find("<title>").unwrap());
        assert!(csp < html.find("<link").unwrap());
        assert!(!html.contains("<script>x"));
    }
}

/// codex P1（PR #3688）: ナビ drawer を渡さない `docs_page` 系の出力は、768px 未満でも
/// サイドバーを残すため `data-no-nav-drawer` を container へ付ける。drawer ありでは付けない。
#[test]
fn container_marks_missing_nav_drawer_so_sidebar_stays_visible() {
    let body = p(vec![], vec![text("本文です。")]);
    let without = render(&docs_page("T", "", sample_sidebar(), body.clone()));
    assert!(without.contains(r#"class="docs-container docs-container--no-toc" data-no-nav-drawer"#));
    let with = render(&docs_page_with_layout(
        "T",
        "",
        sample_sidebar(),
        body,
        &[],
        None,
        Some(ul(vec![], vec![])),
        None,
        PageLayout::Docs,
    ));
    assert!(!with.contains("data-no-nav-drawer"));
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
    // イシュー #1080: 見出しの無いページでは折りたたみ目次
    // （`nav.docs-toc-inline`）も出力しない（右目次と出現条件が一致する）。
    assert!(toc_inline(&entries).is_none());
    assert!(!html.contains(r#"class="docs-toc-inline""#));
    let _ = annotated;
}

/// イシュー #1080: `< 1200px` で右目次カラムが消える代替として、
/// `main.docs-main` の第 1 子に折りたたみ目次を置き、SkipNav のスキップ先
/// ターゲット（`data-part="content"`）・本文（`article.docs-content`）より
/// 前に出現することを固定する（DOM 順の受け入れ条件 1）。
#[test]
fn inline_toc_precedes_skip_nav_target_and_article() {
    let body = fandhe_frontend_core::div(vec![], vec![h2(vec![], vec![text("導入")])]);
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);

    let inline_toc_pos = html
        .find(r#"class="docs-toc-inline""#)
        .expect("docs-toc-inline should exist when headings are present");
    let skip_content_pos = html
        .find(r#"data-part="content""#)
        .expect("skip-nav content target should exist");
    let article_pos = html
        .find(r#"class="docs-content""#)
        .expect("article should exist");

    assert!(
        inline_toc_pos < skip_content_pos,
        "inline toc should precede the SkipNav content target"
    );
    assert!(
        skip_content_pos < article_pos,
        "SkipNav content target should still precede the docs-content article"
    );
}

/// イシュー #1080（WCAG 2.1 SC 2.4.1 回帰、`docs_page_skip_nav_link_is_first_focusable_element_in_body`
/// の対）: 折りたたみ目次の `<summary>`（ネイティブにフォーカス可能な要素）を
/// 持つページであっても、SkipNav の `link`（`<body>` 内で最初にフォーカス
/// 可能な要素という既存契約）より前に他のフォーカス可能要素が出現しない
/// ことを固定する。既存の `docs_page_skip_nav_link_is_first_focusable_element_in_body`
/// は見出しの無いフィクスチャ（`toc_inline` が `None` を返す）のみを使うため
/// この経路を検証しておらず、本テストが見出しありページで明示的に補う。
#[test]
fn inline_toc_summary_does_not_precede_the_skip_nav_link() {
    let body = fandhe_frontend_core::div(vec![], vec![h2(vec![], vec![text("導入")])]);
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);

    // 折りたたみ目次の <summary> が実在すること（この経路を検証している
    // ことの前提確認。存在しなければ以下のフォーカス順検証は空振りする）。
    assert!(html.contains(r#"class="docs-toc-inline-summary""#));

    let body_start = html.find("<body>").expect("body tag should exist") + "<body>".len();
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
        "<summary",
        "tabindex=",
    ] {
        assert!(
            !between.contains(needle),
            "no focusable element ({needle:?}), including the inline toc's <summary>, should precede the skip-nav link in body"
        );
    }
}

/// イシュー #1080: 折りたたみ目次は素の `<details>`/`<summary>`
/// ディスクロージャで、本文中の見出しへ注入された `id` と同じ `href="#<id>"`
/// のアンカーを持ち、既定では閉（`open` 属性を持たない）ことを固定する
/// （JS ハイドレーションなしで動作する受け入れ条件 1 の markup 側）。
#[test]
fn inline_toc_is_a_details_disclosure_with_anchor_links() {
    let body = fandhe_frontend_core::div(vec![], vec![h2(vec![], vec![text("導入")])]);
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);

    let nav_start = html
        .find(r#"<nav class="docs-toc-inline""#)
        .expect("nav.docs-toc-inline should exist");
    let toc_aside_start = html
        .find(r#"class="docs-toc-aside""#)
        .expect("docs-toc-aside should exist");
    let inline_block = &html[nav_start..toc_aside_start];

    assert!(inline_block.contains("<details>"));
    assert!(!inline_block.contains("<details open"));
    assert!(inline_block.contains(r#"class="docs-toc-inline-summary""#));
    // イシュー #3610: summary 内に装飾 svg アイコン（支援技術へは隠す）。
    let summary_start = inline_block.find("<summary").expect("summary");
    let summary_end = inline_block.find("</summary>").expect("summary end");
    let summary = &inline_block[summary_start..summary_end];
    assert!(summary.contains(r#"class="docs-toc-inline-icon""#));
    assert!(summary.contains(r#"aria-hidden="true""#));
    assert!(summary.contains(r#"focusable="false""#));

    let id_marker = r#"<h2 id=""#;
    let start = html.find(id_marker).expect("h2 with injected id");
    let after = &html[start + id_marker.len()..];
    let end = after.find('"').expect("closing quote of id attr");
    let id = &after[..end];
    assert!(inline_block.contains(&format!("href=\"#{id}\"")));
}

/// イシュー #1080（アンチマージガード）: 折りたたみ目次は
/// `crate::script::SITE_JS` のスクロールスパイが `document.querySelector`
/// で掴む `class="docs-toc"`（右目次のみが持つセレクタ）を共有しない。
/// フルページ HTML 中の `class="docs-toc"` 完全一致の出現数がちょうど 1
/// （右目次のみ）であることを固定し、将来の「クラス統合による簡素化」で
/// 折りたたみ目次側へこのクラスが漏れ、`>= 1200px` で非表示のはずの
/// 折りたたみ目次側へ observer が吸着する回帰（右カラムの現在地
/// ハイライト #950 が無音で死ぬ）を検知する。
#[test]
fn inline_toc_does_not_carry_the_scrollspy_class() {
    let body = fandhe_frontend_core::div(vec![], vec![h2(vec![], vec![text("導入")])]);
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);

    let scrollspy_class_count = html.matches(r#"class="docs-toc""#).count();
    assert_eq!(
        scrollspy_class_count, 1,
        "exactly one element (the right toc column's nav) should carry class=\"docs-toc\""
    );
    assert!(html.contains(r#"class="docs-toc-inline""#));
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

/// イシュー #3674: ナビ drawer のチェックボックスハック
/// （`input#docs-nav-drawer-toggle` + `label[for]` + `nav.docs-nav-drawer`）の
/// markup・id/for 紐付け・DOM 順を固定する回帰テスト。CSS の一般兄弟結合子
/// `.docs-nav-drawer-toggle:checked ~ .docs-nav-drawer` が機能するには、`input` が
/// drawer より前の兄弟（同じ `div.docs-header-inner` の子）である必要がある。
/// `nav_drawer: None` のときは toggle・label・drawer のいずれも出力しない。
#[test]
fn nav_drawer_toggle_is_wired_before_drawer_in_header_inner() {
    let body = p(vec![], vec![text("本文です。")]);
    let drawer = fandhe_frontend_core::el(
        "nav",
        vec![("class", "docs-nav-drawer")],
        vec![text("ドロワー本文")],
    );
    let html = render(&docs_page_with_layout(
        "タイトル",
        "",
        sample_sidebar(),
        body.clone(),
        &[],
        None,
        Some(drawer),
        None,
        PageLayout::Docs,
    ));

    assert!(html.contains(r#"type="checkbox""#));
    assert!(html.contains(r#"id="docs-nav-drawer-toggle""#));
    assert!(html.contains(r#"class="docs-nav-drawer-toggle""#));
    assert!(html.contains(r#"autocomplete="off""#));
    assert!(html.contains(r#"for="docs-nav-drawer-toggle""#));
    assert!(html.contains(r#"class="docs-nav-drawer-toggle-label""#));

    let toggle_pos = html.find(r#"id="docs-nav-drawer-toggle""#).expect("toggle");
    let label_pos = html.find(r#"for="docs-nav-drawer-toggle""#).expect("label");
    let drawer_pos = html.find("ドロワー本文").expect("drawer");
    let actions_pos = html.find("docs-header-actions").expect("actions");
    let inner_end = html.find("</header>").expect("header end");
    assert!(actions_pos < toggle_pos, "toggle follows the actions group");
    assert!(toggle_pos < label_pos && label_pos < drawer_pos);
    assert!(drawer_pos < inner_end, "drawer lives inside the header");

    // 旧サイドバー Menu トグルは aside から取り除かれている。
    assert!(!html.contains("docs-sidebar-toggle"));

    // `None` なら toggle・label は出ず、id も増えない。
    let plain = render(&docs_page_with_layout(
        "タイトル",
        "",
        sample_sidebar(),
        body,
        &[],
        None,
        None,
        None,
        PageLayout::Docs,
    ));
    assert!(!plain.contains("docs-nav-drawer"));
    assert!(!plain.contains(r#"type="checkbox""#));
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

/// イシュー #950 受入条件 1: 右目次の先頭に "On this page" 見出し
/// （`h2.docs-toc-title`）が `<ul` より前に出力される。
#[test]
fn toc_nav_emits_on_this_page_heading_before_the_list() {
    let entries = vec![TocEntry {
        level: 2,
        id: "intro".to_string(),
        title: "導入".to_string(),
    }];
    let toc = toc_nav(&entries).expect("toc_nav must return Some for non-empty entries");
    let html = render(&toc);

    assert!(html.contains(r#"class="docs-toc-title""#));
    assert!(html.contains("On this page"));
    let title_pos = html
        .find(r#"class="docs-toc-title""#)
        .expect("docs-toc-title should exist");
    let ul_pos = html.find("<ul>").expect("ul should exist");
    assert!(
        title_pos < ul_pos,
        "docs-toc-title heading should precede the list"
    );
}

/// `nav.docs-toc` は自身の見出しへ `aria-labelledby` で紐付き、対応する
/// `id` を持つ（ランドマークへ名前を与える。WCAG 2.4.1 相当）。
#[test]
fn toc_nav_is_labelled_by_its_heading() {
    let entries = vec![TocEntry {
        level: 2,
        id: "intro".to_string(),
        title: "導入".to_string(),
    }];
    let toc = toc_nav(&entries).expect("toc_nav must return Some for non-empty entries");
    let html = render(&toc);

    let expected_labelledby = format!(r#"aria-labelledby="{TOC_HEADING_ID}""#);
    let expected_id = format!(r#"id="{TOC_HEADING_ID}""#);
    assert!(html.contains(&expected_labelledby));
    assert!(html.contains(&expected_id));
}

/// イシュー #950 受入条件 2: `TOC_MAX_LEVEL` を超える見出し（`h4` 相当）は
/// 目次から除外される（将来 `heading_level` が `h4` 以降を拾うようになっても
/// 右目次は 2 段で頭打ちになる fail-closed なガード）。
#[test]
fn toc_nav_drops_entries_deeper_than_the_max_level() {
    let entries = vec![
        TocEntry {
            level: 2,
            id: "intro".to_string(),
            title: "導入".to_string(),
        },
        TocEntry {
            level: 3,
            id: "detail".to_string(),
            title: "詳細".to_string(),
        },
        TocEntry {
            level: 4,
            id: "too-deep".to_string(),
            title: "深すぎる見出し".to_string(),
        },
    ];
    let toc = toc_nav(&entries).expect("toc_nav must return Some when shallow entries remain");
    let html = render(&toc);

    assert!(html.contains(r##"href="#intro""##));
    assert!(html.contains(r##"href="#detail""##));
    assert!(!html.contains(r##"href="#too-deep""##));
    assert!(!html.contains("深すぎる見出し"));
}

/// 全エントリが `TOC_MAX_LEVEL` を超える場合、深さフィルタ適用後は空になり
/// `toc_nav` は `None` を返す（`docs_page_with_assets` の `has_toc` 判定・
/// `docs-container--no-toc` 修飾が自動的に整合する前提）。
#[test]
fn toc_nav_returns_none_when_every_entry_exceeds_the_max_level() {
    let entries = vec![TocEntry {
        level: 4,
        id: "too-deep".to_string(),
        title: "深すぎる見出し".to_string(),
    }];
    assert!(toc_nav(&entries).is_none());
}

/// イシュー #950: 右目次見出しの id（[`TOC_HEADING_ID`]）は本文走査より前に
/// 予約される。本文側の見出しが偶然同じ slug へ解決される入力を与えても、
/// 既存の「衝突時は `unique_slug` で採番し直す」分岐により
/// `docs-toc-heading-2` へ回避され、id 重複が起こらないことを固定する。
#[test]
fn with_heading_anchors_reserves_the_toc_heading_id() {
    let body = fandhe_frontend_core::div(vec![], vec![h2(vec![], vec![text("Docs toc heading")])]);
    let (_, entries) = with_heading_anchors(body);
    assert_eq!(entries.len(), 1);
    assert_ne!(entries[0].id, TOC_HEADING_ID);
    assert_eq!(entries[0].id, format!("{TOC_HEADING_ID}-2"));
}

/// イシュー #950 受入条件 4: 現在地ハイライト（`aria-current="location"`）は
/// `crate::script::SITE_JS` が実行時にのみ付与する契約であり、SSG が出力する
/// 静的 markup（`toc_nav` の戻り値）には一切含まれない
/// （JS 無効・読み込み失敗時は通常のリンク表示のまま機能する
/// progressive enhancement の Rust 側固定）。
#[test]
fn toc_nav_emits_no_aria_current_without_javascript() {
    let entries = vec![TocEntry {
        level: 2,
        id: "intro".to_string(),
        title: "導入".to_string(),
    }];
    let toc = toc_nav(&entries).expect("toc_nav must return Some for non-empty entries");
    let html = render(&toc);
    assert!(!html.contains("aria-current"));
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
index_path = "/"

[[section.page]]
title = "Intro"
source = "site/index.md"
path = "/"

[[section]]
title = "Guides"
index_path = "/advanced/"

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
        None,
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

    // セクションタイトルがトリガーとして出力される。ページタイトル（`Advanced`）は
    // セクション別 popup の廃止（イシュー #3701）によりヘッダーナビへは出ない。
    assert!(html.contains("Getting Started"));
    assert!(html.contains("Guides"));
    assert!(!html.contains("Advanced"));
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
        None,
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

/// `docs_page` がインライン `<style>` を出さず、site.css の `<link>` を head 内に
/// 持つことを固定する（CSP `style-src 'self'` 対応、#3677）。`@view-transition`
/// の opt-in 自体は site.css 側で、`site_theme` の単体テストと `site_build.rs`
/// が固定する。
#[test]
fn docs_page_emits_no_inline_style_and_links_site_css_in_head() {
    let body = p(vec![], vec![text("本文です。")]);
    let node = docs_page("タイトル", "", sample_sidebar(), body);
    let html = render(&node);

    assert!(!html.contains("<style"));
    let head_end = html.find("</head>").expect("head should exist");
    let link_pos = html
        .find("assets/site.css")
        .expect("site.css link should exist");
    assert!(link_pos < head_end, "site.css link should be inside <head>");
}

/// 上の対: `docs_page_with_assets` 経路でもインライン `<style>` が出ない。
#[test]
fn docs_page_with_assets_emits_no_inline_style() {
    let nav = parse_nav(sample_nav_toml()).expect("fixture nav.toml should parse");
    let body = p(vec![], vec![text("本文です。")]);
    let node = docs_page_with_assets(
        "タイトル",
        "",
        sample_sidebar(),
        body,
        &["assets/pre-styled-ui.css"],
        Some(header_nav(&nav, "/")),
        None,
    );
    let html = render(&node);

    assert!(!html.contains("<style"));
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
        None,
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

/// イシュー #3676: テーマ初期化は本文を持たない同期の外部 `<script src>`
/// （`defer`/`async` なし）として `<head>` 内・全 stylesheet より前に出力され、
/// インライン `<script>` は出力されない（`base_path` なし・あり双方）。
#[test]
fn docs_page_head_loads_theme_init_synchronously_before_stylesheets() {
    let body = p(vec![], vec![text("本文です。")]);
    for (base, expected) in [
        ("", r#"<script src="/assets/theme-init.js"></script>"#),
        (
            "/fandhe-frontend",
            r#"<script src="/fandhe-frontend/assets/theme-init.js"></script>"#,
        ),
    ] {
        let html = render(&docs_page("タイトル", base, sample_sidebar(), body.clone()));
        let pos = html
            .find(expected)
            .unwrap_or_else(|| panic!("{expected} should appear (base={base:?})"));
        let first_css = html
            .find(r#"<link rel="stylesheet""#)
            .expect("stylesheet link");
        let head_end = html.find("</head>").expect("</head> should exist");
        assert!(pos < first_css, "theme-init.js は stylesheet より前");
        assert!(pos < head_end, "theme-init.js は </head> より前");
        assert!(!html.contains("<script>"), "インライン script は出力しない");
    }
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
    let nav_node = header_nav(&nav, "/");
    // `.docs-header-nav` 単体を独立レンダリングし、その完全なシリアライズ
    // 結果（開始〜自身の閉じタグまで）を後段の隣接判定に使う。
    let nav_html = render(&nav_node);
    let node_with_header_nav = docs_page_with_assets(
        "タイトル",
        "",
        sample_sidebar(),
        body,
        &[],
        Some(header_nav(&nav, "/")),
        None,
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

    // CSS の隣接セレクタ `.docs-header-nav + .docs-header-actions`（イシュー
    // #951、`crate::site_theme::STRUCTURAL_CSS` 参照）が実際にマッチする
    // ためには、`.docs-header-nav` の閉じタグ直後に間を置かず
    // `.docs-header-actions` が続く必要がある（同順序であるだけでは
    // 不十分。両者の間に別要素・テキストノードが挟まると隣接セレクタは
    // 不成立になる）。単体レンダリングした `nav_html`（開始〜自身の閉じ
    // タグまでの完全なシリアライズ）に続けて `.docs-header-actions` の
    // 開始タグが直接連結されていることを確認することで、その厳密な隣接を
    // 固定する。
    let expected_adjacent = format!("{nav_html}<div class=\"docs-header-actions\"");
    assert!(
        html_with.contains(&expected_adjacent),
        ".docs-header-nav の直後に間を置かず .docs-header-actions が続く必要がある \
         （CSS 隣接セレクタ .docs-header-nav + .docs-header-actions が成立する前提、\
         イシュー #951）"
    );
}

/// ヘッダー左端揃え（イシュー #949）: `header.docs-header` 直下の
/// `div.docs-header-inner` が brand/[header_nav]/actions を包むこと、
/// および DOM 出現順（`docs-header` < `docs-header-inner` < `docs-brand`
/// < `docs-header-nav` < `docs-header-actions`）を固定する。`header_nav`
/// が `None`（`docs_page` 経路）でも `docs-header-inner` が出現することを
/// 併せて確認する。
#[test]
fn docs_page_wraps_header_children_in_inner_container() {
    let body = p(vec![], vec![text("本文です。")]);

    // `docs_page`（header_nav なし）経路。
    let node_without_header_nav = docs_page("タイトル", "", sample_sidebar(), body.clone());
    let html_without = render(&node_without_header_nav);
    assert!(html_without.contains(r#"class="docs-header-inner""#));
    let header_pos = html_without
        .find(r#"class="docs-header""#)
        .expect("docs-header should appear");
    let inner_pos = html_without
        .find(r#"class="docs-header-inner""#)
        .expect("docs-header-inner should appear");
    let brand_pos = html_without
        .find(r#"class="docs-brand""#)
        .expect("docs-brand should appear");
    let actions_pos = html_without
        .find(r#"class="docs-header-actions""#)
        .expect("docs-header-actions should appear");
    assert!(header_pos < inner_pos);
    assert!(inner_pos < brand_pos);
    assert!(brand_pos < actions_pos);

    // `docs_page_with_assets`（header_nav あり）経路。
    let nav = parse_nav(sample_nav_toml()).expect("fixture nav.toml should parse");
    let node_with_header_nav = docs_page_with_assets(
        "タイトル",
        "",
        sample_sidebar(),
        body,
        &[],
        Some(header_nav(&nav, "/")),
        None,
    );
    let html_with = render(&node_with_header_nav);
    let header_pos = html_with
        .find(r#"class="docs-header""#)
        .expect("docs-header should appear");
    let inner_pos = html_with
        .find(r#"class="docs-header-inner""#)
        .expect("docs-header-inner should appear");
    let brand_pos = html_with
        .find(r#"class="docs-brand""#)
        .expect("docs-brand should appear");
    let nav_pos = html_with
        .find(r#"class="docs-header-nav""#)
        .expect("docs-header-nav should appear");
    let actions_pos = html_with
        .find(r#"class="docs-header-actions""#)
        .expect("docs-header-actions should appear");
    assert!(header_pos < inner_pos);
    assert!(inner_pos < brand_pos);
    assert!(brand_pos < nav_pos);
    assert!(nav_pos < actions_pos);
}

/// イシュー #958 の必須検証（設計 §4-2）: `data-search-index` 属性値が
/// `layout::asset_href(base_path, search_index::REL_PATH)` と一致すること。
/// `base_path` を `""` と `"/fandhe-frontend"` の 2 通りで確認する
/// （linkcheck は `data-*` 属性を検証しないため、URL 誤りを検知する唯一の
/// 機械的手段）。
#[test]
fn docs_page_search_input_data_attribute_matches_asset_href() {
    let body = p(vec![], vec![text("本文です。")]);

    let html_root = render(&docs_page("タイトル", "", sample_sidebar(), body.clone()));
    let expected_root = asset_href("", search_index::REL_PATH);
    assert!(html_root.contains(&format!(r#"data-search-index="{expected_root}""#)));

    let html_base = render(&docs_page(
        "タイトル",
        "/fandhe-frontend",
        sample_sidebar(),
        body,
    ));
    let expected_base = asset_href("/fandhe-frontend", search_index::REL_PATH);
    assert!(html_base.contains(&format!(r#"data-search-index="{expected_base}""#)));
    assert_eq!(expected_base, "/fandhe-frontend/assets/search-index.json");
}

/// 検索ブロック（`div.docs-search`）が `div.docs-header-actions` の第 1 子
/// （GitHub リンク・テーマトグルより前）であることを固定する（イシュー #958）。
#[test]
fn docs_page_search_block_is_the_first_header_action() {
    let body = p(vec![], vec![text("本文です。")]);
    let html = render(&docs_page("タイトル", "", sample_sidebar(), body));

    let search_pos = html
        .find(r#"class="docs-search""#)
        .expect("docs-search should appear");
    let github_pos = html
        .find(r#"class="docs-github-link""#)
        .expect("docs-github-link should appear");
    let toggle_pos = html
        .find(r#"class="docs-theme-toggle""#)
        .expect("docs-theme-toggle should appear");
    assert!(search_pos < github_pos);
    assert!(search_pos < toggle_pos);
}

/// SSG 出力時点で検索ブロック・結果一覧の双方が既定 `hidden` であることを
/// 固定する（JS 無効時の受入条件、`crate::script::SITE_JS` が配線完了後に
/// のみ除去する契約の HTML 側の裏付け）。
#[test]
fn docs_page_search_elements_are_hidden_by_default() {
    let body = p(vec![], vec![text("本文です。")]);
    let html = render(&docs_page("タイトル", "", sample_sidebar(), body));

    let search_start = html
        .find(r#"<div class="docs-search""#)
        .expect("div.docs-search should appear");
    let search_tag_end = html[search_start..]
        .find('>')
        .expect("div.docs-search opening tag should close");
    let search_tag = &html[search_start..search_start + search_tag_end];
    assert!(search_tag.contains("hidden"));

    let results_start = html
        .find(r#"id="docs-search-results""#)
        .expect("ul#docs-search-results should appear");
    let results_tag_start = html[..results_start].rfind('<').unwrap();
    let results_tag_end = html[results_tag_start..]
        .find('>')
        .expect("ul#docs-search-results opening tag should close");
    let results_tag = &html[results_tag_start..results_tag_start + results_tag_end];
    assert!(results_tag.contains("hidden"));
}

/// `docs_page`（`header_nav: None`）経路でも検索ブロックが出力されることを
/// 固定する（`docs-header-actions` 本体と同じ「無条件出力」契約、イシュー #958）。
#[test]
fn docs_page_without_header_nav_still_includes_search_block() {
    let body = p(vec![], vec![text("本文です。")]);
    let html = render(&docs_page("タイトル", "", sample_sidebar(), body));
    assert!(html.contains(r#"class="docs-search""#));
    assert!(html.contains(r#"class="docs-search-input""#));
    assert!(html.contains(r#"id="docs-search-results""#));
}

/// 検索入力の WAI-ARIA combobox 配線（`role="combobox"`・`aria-controls`）が
/// `ul#docs-search-results` の `id` と一致することを固定する（イシュー #958）。
#[test]
fn docs_page_search_input_combobox_attributes_reference_results_list_id() {
    let body = p(vec![], vec![text("本文です。")]);
    let html = render(&docs_page("タイトル", "", sample_sidebar(), body));
    assert!(html.contains(r#"role="combobox""#));
    assert!(html.contains(r#"aria-controls="docs-search-results""#));
    assert!(html.contains(r#"id="docs-search-results""#));
}

/// 検索ブロックは `<form>` で包まない（JS 無効時に Enter キーでのフォーム
/// 送信を誘発しないため、モジュール doc・設計文書 §4-1 参照）。
#[test]
fn docs_page_output_contains_no_form_element() {
    let body = p(vec![], vec![text("本文です。")]);
    let html = render(&docs_page("タイトル", "", sample_sidebar(), body));
    assert!(!html.contains("<form"));
}

/// イシュー #3604: 全本体ページの `<head>` が `base_path` 付きの SVG favicon
/// link を持つ（無いとブラウザが `/favicon.ico` を取りに行き 404 になる）。
#[test]
fn head_carries_base_path_aware_svg_favicon_link() {
    let body = p(vec![], vec![text("本文")]);
    let cases = [
        ("/fandhe-frontend", "/fandhe-frontend/assets/favicon.svg"),
        ("/fandhe-frontend/", "/fandhe-frontend/assets/favicon.svg"),
        ("", "/assets/favicon.svg"),
    ];
    for (base, href) in cases {
        let html = render(&docs_page("T", base, sample_sidebar(), body.clone()));
        let link = format!(r#"<link rel="icon" type="image/svg+xml" href="{href}">"#);
        let at = html
            .find(&link)
            .unwrap_or_else(|| panic!("{link} missing in {html}"));
        assert!(at < html.find("</head>").unwrap());
    }
}
/// ヘッダー部分（`<header` から `</header>` まで）だけを切り出す。
fn header_html(html: &str) -> &str {
    let start = html.find("<header").expect("header should exist");
    let end = html[start..]
        .find("</header>")
        .expect("header should close")
        + start;
    &html[start..end]
}

/// イシュー #3606: ブランド部の DOM 順（brand < version badge < actions）、
/// ロゴの `aria-hidden`、badge 文言が `site_version` と一致することを固定する。
#[test]
fn header_brand_has_aria_hidden_logo_and_core_version_badge() {
    let body = p(vec![], vec![text("本文です。")]);
    let html = render(&docs_page("タイトル", "", sample_sidebar(), body));
    let header = header_html(&html);

    let brand = header.find(r#"class="docs-brand""#).expect("brand");
    let mark = header
        .find(r#"class="docs-brand-mark" aria-hidden="true""#)
        .expect("brand mark should be aria-hidden");
    let version = header
        .find(r#"class="docs-brand-version""#)
        .expect("version");
    let actions = header
        .find(r#"class="docs-header-actions""#)
        .expect("actions");
    assert!(brand < mark && mark < version && version < actions);
    // badge は brand リンクの外（リンク名に版数を混ぜない）。
    let brand_end = header[brand..].find("</a>").expect("brand a closes") + brand;
    assert!(version > brand_end);

    let v = fandhe_frontend_docs_site::site_version::core_version().expect("core version");
    assert!(header.contains(&format!("core v{v}")), "{header}");
    assert!(header[version..].contains(r#"data-scope="badge""#));
}

/// イシュー #3606: GitHub リンクは `target`/`rel` をそれぞれちょうど 1 回だけ出す
/// （pre-styled `link` の `external` が付与。attrs で重ねると重複する）。
#[test]
fn header_github_link_has_single_target_and_rel() {
    let body = p(vec![], vec![text("本文です。")]);
    let html = render(&docs_page("タイトル", "", sample_sidebar(), body));
    let header = header_html(&html);
    assert_eq!(header.matches(r#"target="_blank""#).count(), 1, "{header}");
    assert_eq!(
        header.matches(r#"rel="noopener noreferrer""#).count(),
        1,
        "{header}"
    );
}

/// イシュー #3606/#3672: 検索の素の input は dialog 内の `input_group` の内側、
/// label と結果一覧は group の外側にある。ショートカット `/` はボタン側
/// （`aria-keyshortcuts` と kbd）が担い、dialog はボタンの後ろに置かれる。
#[test]
fn header_search_input_is_inside_dialog_input_group_with_slash_hint_on_trigger() {
    let body = p(vec![], vec![text("本文です。")]);
    let html = render(&docs_page("タイトル", "", sample_sidebar(), body));
    let header = header_html(&html);

    let trigger = header
        .find(r#"class="docs-search-trigger""#)
        .expect("trigger");
    let kbd = header.find(r#"data-scope="kbd""#).expect("kbd");
    let dialog = header.find("<dialog").expect("dialog");
    let label = header.find(r#"class="docs-search-label""#).expect("label");
    let root = header
        .find(r#"data-scope="input-group" data-part="root""#)
        .expect("input_group root");
    let input = header.find(r#"class="docs-search-input""#).expect("input");
    let results = header.find(r#"id="docs-search-results""#).expect("results");
    assert!(trigger < kbd && kbd < dialog && dialog < label);
    assert!(label < root && root < input && input < results);
    assert!(header.contains(r#"aria-keyshortcuts="/""#));
    assert!(header.contains(r#"data-scope="icon""#));
}

/// イシュー #3672: ダイアログは `open` なしで SSR され、ボタンは
/// `aria-haspopup`/`aria-expanded` を静的に持たない（JS が付与する）。
#[test]
fn header_search_dialog_is_closed_and_trigger_has_no_static_popup_state() {
    let body = p(vec![], vec![text("本文です。")]);
    let html = render(&docs_page("タイトル", "", sample_sidebar(), body));
    let header = header_html(&html);
    let start = header.find("<dialog").expect("dialog");
    let tag_end = header[start..].find('>').expect("dialog tag end") + start;
    let dialog_tag = &header[start..tag_end];
    assert!(!dialog_tag.contains("open"), "{dialog_tag}");
    assert!(
        dialog_tag.contains(r#"id="docs-search-dialog""#),
        "{dialog_tag}"
    );
    assert!(!header.contains("aria-haspopup"), "{header}");
    let trigger_start = header
        .find(r#"class="docs-search-trigger""#)
        .expect("trigger");
    let before_dialog = &header[trigger_start..start];
    assert!(!before_dialog.contains("aria-expanded"), "{before_dialog}");
    assert!(before_dialog.contains(r#"aria-controls="docs-search-dialog""#));
}

/// イシュー #3609: `footer` が `Some` のとき `<body>` の最後の子（`div.docs-container`
/// の直後）へ出力され、`None` では出力されない。`main` の外に置くことで暗黙の
/// `contentinfo` が成立する。
#[test]
fn docs_page_with_assets_places_footer_after_container_as_last_body_child() {
    let nav = parse_nav(sample_nav_toml()).expect("fixture nav.toml should parse");
    let footer = fandhe_frontend_core::footer(
        vec![("class", "docs-footer")],
        vec![p(vec![], vec![text("フッター")])],
    );
    let html = render(&docs_page_with_assets(
        "タイトル",
        "",
        sample_sidebar(),
        p(vec![], vec![text("本文です。")]),
        &[],
        Some(header_nav(&nav, "/")),
        Some(footer),
    ));
    let skip = html.find("Skip to content").expect("skip nav");
    let header = html.find("<header class=\"docs-header\"").expect("header");
    let container = html.find("class=\"docs-container").expect("container");
    let main_end = html.rfind("</main>").expect("main end");
    let footer_at = html.find("<footer class=\"docs-footer\"").expect("footer");
    assert!(skip < header && header < container && container < main_end);
    assert!(main_end < footer_at, "footer は main の外（後ろ）に置く");
    assert!(html.ends_with("</footer></body></html>"), "{html}");

    let without = render(&docs_page(
        "タイトル",
        "",
        sample_sidebar(),
        p(vec![], vec![text("本文です。")]),
    ));
    assert!(!without.contains("docs-footer"));
}

/// イシュー #3612: ランディング骨格は右目次・折りたたみ目次を出さず、サイドバー・
/// SkipNav・本文 article の DOM 順序は標準骨格と同じに保つ。
#[test]
fn landing_layout_drops_toc_but_keeps_sidebar_and_skip_nav_order() {
    let body = fandhe_frontend_core::div(
        vec![],
        vec![
            h2(vec![], vec![text("見出し")]),
            p(vec![], vec![text("本文")]),
        ],
    );
    let landing = render(&docs_page_with_layout(
        "T",
        "",
        sample_sidebar(),
        body.clone(),
        &[],
        None,
        None,
        None,
        PageLayout::Landing,
    ));
    assert!(landing.contains(r#"class="docs-container docs-landing""#));
    assert!(!landing.contains("docs-toc-aside"));
    assert!(!landing.contains("docs-toc-inline"));
    assert!(landing.contains(r#"class="docs-sidebar""#));
    let skip_link = landing.find(r#"data-part="link""#).expect("skip link");
    let header = landing.find("<header").expect("header");
    let target = landing.find(r#"data-part="content""#).expect("skip target");
    let article = landing.find(r#"class="docs-content""#).expect("article");
    assert!(skip_link < header && target < article);

    // 標準骨格は従来どおり右目次を持ち、docs_page_with_assets と出力が一致する。
    let docs = render(&docs_page_with_layout(
        "T",
        "",
        sample_sidebar(),
        body.clone(),
        &[],
        None,
        None,
        None,
        PageLayout::Docs,
    ));
    assert!(docs.contains("docs-toc-aside") && !docs.contains("docs-landing"));
    assert_eq!(
        docs,
        render(&docs_page_with_assets(
            "T",
            "",
            sample_sidebar(),
            body,
            &[],
            None,
            None
        ))
    );
}

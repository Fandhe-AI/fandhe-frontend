//! `crate::nav::sidebar()` のカテゴリ階層描画（`<details>`/`<summary>`、
//! イシュー #940）そのものの回帰テスト。
//!
//! `tests/nav_group_schema.rs` は `nav.toml` 3 階層スキーマのパース・
//! 走査順・（ページタイトルの）XSS 回帰を検証する統合テストであり、
//! `sidebar()` の**描画結果**（DOM 構造・`open` 状態・後方互換・
//! グループタイトルの XSS・無 JS 到達性）は本ファイルのスコープとして
//! 分離する（責務の重複を避ける）。
//!
//! 設計の正本は `crates/docs-site/src/nav.rs::sidebar` の rustdoc
//! （「カテゴリ階層描画（イシュー #940）」節）と #940 実装計画 §3.1。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::nav::{parse_nav, sidebar, Nav};

/// 直下ページ 1 件 + グループ 2 件（うち 1 グループは配下ページ複数）を
/// 持つ最小フィクスチャ。`current_path` は呼び出し側で切り替えて
/// `open` 状態の分岐を検証する。
fn fixture_nav_with_groups() -> Nav {
    let toml = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"

[[section.page]]
title = "Overview"
source = "overview.md"
path = "/components/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/components/button/"

[[section.group.page]]
title = "Checkbox"
source = "checkbox.md"
path = "/components/checkbox/"

[[section.group]]
title = "Layout"

[[section.group.page]]
title = "Grid"
source = "grid.md"
path = "/components/grid/"
"#;
    parse_nav(toml).expect("group fixture nav.toml should parse")
}

/// グループを持たない既存 2 階層フィクスチャ（後方互換検証専用）。
fn fixture_nav_without_groups() -> Nav {
    let toml = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Getting Started"

[[section.page]]
title = "Intro"
source = "intro.md"
path = "/"

[[section.page]]
title = "Quickstart"
source = "quickstart.md"
path = "/quickstart/"
"#;
    parse_nav(toml).expect("no-group fixture nav.toml should parse")
}

/// グループのみで直下ページが 0 件のセクションを持つフィクスチャ
/// （イシュー #939 で正当化された構成。空 `<ul>` を出力しないことの検証専用）。
fn fixture_nav_group_only_section() -> Nav {
    let toml = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/components/button/"
"#;
    parse_nav(toml).expect("group-only fixture nav.toml should parse")
}

// ---- 正常系: DOM 構造 ----

#[test]
fn renders_details_summary_for_each_group_in_declaration_order() {
    let nav = fixture_nav_with_groups();
    let html = render(&sidebar(&nav, "/"));

    assert!(html.contains(r#"<details class="docs-nav-group">"#));
    assert!(html.contains(r#"<summary class="docs-nav-group-summary">Forms</summary>"#));
    assert!(html.contains(r#"<summary class="docs-nav-group-summary">Layout</summary>"#));
    assert!(html.contains(r#"class="docs-nav-group-list""#));

    // 宣言順（Forms → Layout）を保つ。
    let forms_pos = html
        .find("Forms")
        .expect("Forms group summary should be present");
    let layout_pos = html
        .find("Layout")
        .expect("Layout group summary should be present");
    assert!(
        forms_pos < layout_pos,
        "グループは宣言順（Forms → Layout）で描画される必要がある"
    );

    // グループ配下ページのリンクが出力される。
    assert!(html.contains(r#"href="/components/button/""#));
    assert!(html.contains(r#"href="/components/checkbox/""#));
    assert!(html.contains(r#"href="/components/grid/""#));
    assert!(html.contains(">Button<"));
    assert!(html.contains(">Checkbox<"));
    assert!(html.contains(">Grid<"));
}

/// 直下ページ `ul` が全 `<details>` より前に描画される（§6-2 描画順契約
/// 「直下ページ → グループ（宣言順）」の機械固定）。
#[test]
fn direct_pages_ul_precedes_all_group_details() {
    let nav = fixture_nav_with_groups();
    let html = render(&sidebar(&nav, "/"));

    let overview_ul_pos = html
        .find(r#"href="/components/""#)
        .expect("Overview direct page link should be present");
    let first_details_pos = html
        .find("<details")
        .expect("at least one <details> should be present");
    assert!(
        overview_ul_pos < first_details_pos,
        "直下ページ（Overview）は最初の <details> より前に描画される必要がある"
    );
}

// ---- 正常系: open 状態 ----

#[test]
fn only_the_group_containing_current_path_is_open() {
    let nav = fixture_nav_with_groups();
    let html = render(&sidebar(&nav, "/components/button/"));

    // Forms（current_path を含む）は open、Layout は open ではない。
    let forms_start = html
        .find(r#"<details class="docs-nav-group" open="">"#)
        .expect("Forms group should carry the open attribute");
    let forms_summary = html[forms_start..]
        .find("Forms")
        .map(|rel| forms_start + rel)
        .expect("Forms summary should follow the open details tag");
    let _ = forms_summary;

    // open 属性を持つ <details> は 1 件のみ。
    let open_count = html
        .matches(r#"<details class="docs-nav-group" open="">"#)
        .count();
    assert_eq!(open_count, 1, "open 状態の <details> は 1 件のみのはず");

    // Layout グループの <details> は open 属性を持たない。
    let layout_start = html
        .find(r#"<summary class="docs-nav-group-summary">Layout</summary>"#)
        .expect("Layout group summary should be present");
    let preceding = &html[..layout_start];
    let layout_details_start = preceding
        .rfind("<details")
        .expect("Layout group should be preceded by its own <details> open tag");
    let layout_open_tag_end = html[layout_details_start..]
        .find('>')
        .map(|rel| layout_details_start + rel + 1)
        .expect("Layout <details> open tag should terminate with >");
    let layout_open_tag = &html[layout_details_start..layout_open_tag_end];
    assert!(
        !layout_open_tag.contains("open"),
        "Layout グループの <details> は open 属性を持たないはず: {layout_open_tag}"
    );
}

/// どのグループにも属さない `current_path`（直下ページ・nav 外パス）では
/// `open` 属性が 0 件になる。
#[test]
fn no_group_is_open_when_current_path_matches_no_group_page() {
    let nav = fixture_nav_with_groups();

    // 直下ページが現在ページのケース。
    let html_direct = render(&sidebar(&nav, "/components/"));
    assert!(!html_direct.contains("open=\"\""));

    // nav に存在しないパス（サイトトップ等）のケース。
    let html_outside = render(&sidebar(&nav, "/does-not-exist/"));
    assert!(!html_outside.contains("open=\"\""));
}

// ---- 後方互換 ----

/// グループを持たない nav の `sidebar()` 出力に `<details`/`docs-nav-group`
/// が一切現れない（2 階層パスの出力が本イシューで変化しないことの機械固定。
/// `site/nav.toml` は本 PR で未変更のため実サイト出力もこの契約に含まれる）。
#[test]
fn sidebar_without_groups_emits_no_details_markup() {
    let nav = fixture_nav_without_groups();
    let html = render(&sidebar(&nav, "/quickstart/"));

    assert!(!html.contains("<details"));
    assert!(!html.contains("docs-nav-group"));
}

/// 直下ページ 0 件・グループのみのセクションでは空 `<ul>` を出力しない
/// （イシュー #939 で正当化された構成に対する `sidebar()` 側の対応、
/// [`sidebar`] rustdoc 参照）。
#[test]
fn group_only_section_emits_no_empty_direct_pages_ul() {
    let nav = fixture_nav_group_only_section();
    let html = render(&sidebar(&nav, "/components/button/"));

    assert!(
        !html.contains("<ul></ul>"),
        "空の直下ページ <ul> を出力してはならない: {html}"
    );
    // グループ自体は描画される。
    assert!(html.contains("docs-nav-group"));
}

// ---- XSS 回帰 ----

/// グループタイトルは `<summary>` へそのまま置かれる新規の注入面。
/// `tests/nav_group_schema.rs::sidebar_and_header_nav_escape_group_page_titles`
/// はページタイトルのみを覆っており、`<summary>` 内のグループタイトルは
/// 未検証だったため専用の回帰テストとして追加する。
#[test]
fn summary_escapes_group_title() {
    let toml = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"

[[section.group]]
title = "<script>alert(1)</script>&\"Forms\""

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/components/button/"
"#;
    let nav = parse_nav(toml).expect("valid nav.toml should parse");
    let html = render(&sidebar(&nav, "/components/button/"));

    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;&amp;&quot;Forms&quot;"));
}

// ---- 無 JS 到達性 ----

/// 出力 HTML に `<script` タグ・`on[a-z]+=` 系のイベントハンドラ属性が
/// 含まれないことを assert する（受け入れ条件「JS 無効環境でもナビゲーション
/// が成立する」の機械固定。`<details>` はネイティブ挙動のため、配下リンクは
/// JS の有無に関わらず常に DOM 内に存在し到達可能）。
#[test]
fn rendered_sidebar_contains_no_script_or_inline_event_handlers() {
    let nav = fixture_nav_with_groups();
    let html = render(&sidebar(&nav, "/components/button/"));

    assert!(!html.contains("<script"));
    assert!(!html.to_ascii_lowercase().contains(" onclick="));
    assert!(!html.to_ascii_lowercase().contains(" onload="));
    assert!(!html.to_ascii_lowercase().contains(" onerror="));

    // 全リンクが href を持ち到達可能であることも併せて確認する。
    assert!(html.contains(r#"href="/components/button/""#));
    assert!(html.contains(r#"href="/components/checkbox/""#));
    assert!(html.contains(r#"href="/components/grid/""#));
}

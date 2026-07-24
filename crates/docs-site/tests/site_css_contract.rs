//! `crates/docs-site/src/layout.rs` / `nav.rs` が生成する HTML の `class`
//! 属性値と、サイト骨格 CSS（`crate::site_theme::stylesheet()` がビルド時
//! 生成する `assets/site.css`）のクラス名契約が乖離していないことを検証する
//! 回帰テスト（イシュー #488、CSS 供給方式のビルド時生成への切替はイシュー
//! #905）。
//!
//! # 背景
//!
//! `layout.rs` は独自に `docs-*` プレフィックスの class を組み立て、
//! `nav.rs` は `sidebar` / `current` / `prev-next` / `prev` / `next` を
//! 独自に組み立てる並列実装であり、両者が実際に一致しているかは
//! コンパイラでは検証されない（CSS 文字列は Rust の型システム外）。過去に
//! `site.css` 側だけが `site-*` プレフィックスの想定クラス名で書かれ、
//! `layout.rs` の実出力（`docs-*` プレフィックス）と食い違ったまま放置され、
//! 本番 docs サイトで CSS がほぼ効かない不具合が発生した。本テストは
//! `layout.rs` / `nav.rs` の実出力に含まれる全 `class` 属性値（空白区切りの
//! 各トークン）が生成 `assets/site.css` 内にセレクタとして出現することを
//! 機械的に検証し、再発を fail-closed で検知する（取得元をイシュー #905 で
//! 静的ファイル読込から [`site_theme::stylesheet`] の呼び出しへ差し替えた
//! のみで、class 抽出・検証ロジック自体は不変）。
//!
//! Markdown レンダラ（`markdown.rs`）が動的に生成する `language-<lang>`
//! クラス（コードブロックの言語トークン依存で無数の値を取りうる）は本テスト
//! のスコープ外とする（`.docs-content pre code` の要素セレクタでスタイルが
//! 適用されるため、契約ドリフトの対象にならない）。
//!
//! 同様に `markdown.rs` が admonition（`> [!NOTE]` 等）から生成する
//! `fd-alert--status-*` class はサイト骨格 CSS の契約対象外（サイト骨格 CSS
//! は変更しない不変条件、イシュー #715）。代わりに `crate::admonition::stylesheet()`
//! が生成する `assets/admonition.css` 側が契約を持つため、
//! `admonition_markdown_output_classes_are_covered_by_generated_admonition_css`
//! が両者の乖離を検知する。

use std::collections::HashSet;

use fandhe_frontend_core::{li, p, render, text, ul, Node};
use fandhe_frontend_docs_site::layout::docs_page;
use fandhe_frontend_docs_site::nav::{parse_nav, prev_next_nav, sidebar, Nav};
use fandhe_frontend_docs_site::site_theme;

/// サイト骨格 CSS 全量を取得する（イシュー #905: 静的ファイル読込から
/// [`site_theme::stylesheet`] 呼び出しへ差し替え。class 抽出・検証ロジック
/// 自体は不変）。
fn site_css() -> String {
    site_theme::stylesheet()
        .expect("site theme stylesheet should assemble")
        .as_css()
        .to_string()
}

/// TOC（`h2`/`h3` 見出し）・サイドバー 2 セクション・前後ページ双方が揃う
/// ように仕立てたフィクスチャ `Nav`。`docs-toc-level-2` /
/// `docs-toc-level-3` と `prev-next` の `prev`/`next` 両方を同時に
/// 発生させるための最小構成。
fn fixture_nav() -> Nav {
    let toml = r#"
[site]
title = "Fixture"
base_path = ""

[[section]]
title = "Getting Started"

[[section.page]]
title = "Intro"
source = "site/index.md"
path = "/"

[[section.page]]
title = "Quickstart"
source = "site/index.md"
path = "/quickstart/"

[[section]]
title = "Guides"

[[section.page]]
title = "Advanced"
source = "site/index.md"
path = "/advanced/"
"#;
    parse_nav(toml).expect("fixture nav.toml should parse")
}

fn fixture_body() -> Node {
    fandhe_frontend_core::div(
        vec![],
        vec![
            fandhe_frontend_core::h2(vec![], vec![text("導入")]),
            p(vec![], vec![text("本文です。")]),
            fandhe_frontend_core::h3(vec![], vec![text("詳細")]),
            p(vec![], vec![text("詳細本文です。")]),
        ],
    )
}

fn fixture_sidebar() -> Node {
    // `docs_page` 単独呼び出しテストでは `nav::sidebar()` の実出力を使わず
    // 最小の `ul`/`li` を渡す既存 `layout_render.rs` の流儀に合わせつつ、
    // 本テストでは `nav::sidebar()` 自体の class も別途検証する
    // （`sidebar_html_class_tokens_are_covered_by_site_css` 参照）。
    ul(vec![], vec![li(vec![], vec![text("はじめに")])])
}

/// html 文字列中の全 `class="..."` 属性値を、空白区切りトークンへ展開して
/// 収集する。
fn extract_class_tokens(html: &str) -> HashSet<String> {
    let mut tokens = HashSet::new();
    let mut rest = html;
    while let Some(start) = rest.find(r#"class=""#) {
        let after = &rest[start + r#"class=""#.len()..];
        let Some(end) = after.find('"') else { break };
        let value = &after[..end];
        for token in value.split_whitespace() {
            tokens.insert(token.to_string());
        }
        rest = &after[end + 1..];
    }
    tokens
}

/// `/* ... */` コメントを取り除く。`site.css` 冒頭のクラス名契約コメントは
/// 実セレクタと同じ `.docs-header` のような記法で説明文を書いているため、
/// コメントを除去せずに [`extract_css_class_selectors`] を適用すると
/// 「コメントで名前に言及されているだけ」で実セレクタが存在するかのように
/// 誤判定してしまう（本テストが検知すべき乖離をすり抜けてしまう）。
/// ネストしないブロックコメントのみを前提とする単純な走査で十分
/// （`site.css` は CSS の仕様どおりネストしないブロックコメントしか使わない）。
fn strip_css_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find("*/") {
            Some(end) => rest = &after[end + 2..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

/// CSS テキストから `.identifier` 形式のクラスセレクタトークンをすべて
/// 収集する。コメントは事前に [`strip_css_comments`] で除去してから走査する
/// （契約コメント中の記法をセレクタと誤認しないため）。数値（`0.5rem` 等）の
/// 小数点は次の文字が識別子開始文字（英字 / `_`）でないため誤検出しない。
fn extract_css_class_selectors(css: &str) -> HashSet<String> {
    let css = strip_css_comments(css);
    let mut tokens = HashSet::new();
    let chars: Vec<char> = css.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '.' {
            let next = chars.get(i + 1).copied();
            if matches!(next, Some(c) if c.is_ascii_alphabetic() || c == '_') {
                let mut j = i + 1;
                let mut token = String::new();
                while j < chars.len()
                    && (chars[j].is_ascii_alphanumeric() || chars[j] == '-' || chars[j] == '_')
                {
                    token.push(chars[j]);
                    j += 1;
                }
                tokens.insert(token);
                i = j;
                continue;
            }
        }
        i += 1;
    }
    tokens
}

/// 生成 HTML 中の全 class トークンが `site.css` 側にセレクタとして
/// 存在することを検証する（fail-closed。1 つでも欠けていれば即失敗）。
fn assert_all_classes_covered(html: &str, css_tokens: &HashSet<String>, context: &str) {
    let html_tokens = extract_class_tokens(html);
    assert!(
        !html_tokens.is_empty(),
        "{context}: フィクスチャ HTML から class トークンが 1 件も抽出できなかった（テスト自体の不備の可能性）"
    );
    let missing: Vec<&String> = html_tokens
        .iter()
        .filter(|t| !css_tokens.contains(t.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "{context}: 以下の class が生成 assets/site.css にセレクタとして存在しない: {missing:?}\n\
         layout.rs / nav.rs の実出力と site_theme のクラス名契約が乖離している。"
    );
}

#[test]
fn docs_page_html_class_tokens_are_covered_by_site_css() {
    let css_tokens = extract_css_class_selectors(&site_css());
    let node = docs_page("タイトル", "", fixture_sidebar(), fixture_body());
    let html = render(&node);
    assert_all_classes_covered(&html, &css_tokens, "docs_page");
}

#[test]
fn sidebar_html_class_tokens_are_covered_by_site_css() {
    let css_tokens = extract_css_class_selectors(&site_css());
    let nav = fixture_nav();
    // 現在ページを 2 番目のページに一致させ、`aria-current`/`class="current"`
    // 双方の分岐を実際に発生させる。
    let node = sidebar(&nav, "/quickstart/");
    let html = render(&node);
    assert_all_classes_covered(&html, &css_tokens, "nav::sidebar");
}

#[test]
fn prev_next_nav_html_class_tokens_are_covered_by_site_css() {
    let css_tokens = extract_css_class_selectors(&site_css());
    let nav = fixture_nav();
    // 中間ページを指定し、prev/next 両方の `<a>` を同時に発生させる。
    let node = prev_next_nav(&nav, "/quickstart/");
    let html = render(&node);
    assert_all_classes_covered(&html, &css_tokens, "nav::prev_next_nav");
}

#[test]
fn extract_css_class_selectors_ignores_decimal_numbers() {
    let css = "margin: 0.5rem; .docs-toc { color: red; }";
    let tokens = extract_css_class_selectors(css);
    assert!(tokens.contains("docs-toc"));
    assert!(!tokens.contains("5rem"));
}

/// イシュー #715 の乖離検知テスト（モジュール doc 冒頭の追記参照）:
/// `markdown.rs` の admonition レンダリングが生成する全 `fd-alert--status-*`
/// class が `crate::admonition::stylesheet()`（`assets/admonition.css` の
/// 実体）にセレクタとして存在することを固定する。`site/assets/site.css` 側は
/// 対象外（分離 CSS 方式のため、`assert_all_classes_covered` は使わない）。
#[test]
fn admonition_markdown_output_classes_are_covered_by_generated_admonition_css() {
    use fandhe_frontend_docs_site::admonition;
    use fandhe_frontend_docs_site::markdown::render_markdown;

    let admonition_css = admonition::stylesheet()
        .expect("admonition stylesheet should assemble")
        .as_css()
        .to_string();
    let css_tokens = extract_css_class_selectors(&admonition_css);

    let markdown = "\
> [!NOTE]\n> note body\n\n\
> [!TIP]\n> tip body\n\n\
> [!IMPORTANT]\n> important body\n\n\
> [!WARNING]\n> warning body\n\n\
> [!CAUTION]\n> caution body\n";
    let html = render_markdown(markdown)
        .iter()
        .map(render)
        .collect::<Vec<_>>()
        .join("");
    assert_all_classes_covered(&html, &css_tokens, "markdown::render_markdown (admonition)");
}

/// イシュー #910 の乖離検知テスト: `nav::sidebar()` の実出力（headless
/// `nav_list` markup、`data-scope="nav-list" data-part="heading|list|item|
/// link"`）に対応するセレクタが生成 `assets/site.css`（`site_theme::stylesheet()`
/// が `fandhe_frontend_pre_styled_ui::nav_list::stylesheet()` を取り込んだ
/// もの）に実在することを検証する（`admonition_markdown_output_classes_are_covered_by_generated_admonition_css`
/// / `docs_page_skip_nav_parts_are_covered_by_generated_skip_nav_css` と
/// 同型の「実出力 ⇔ 生成 CSS」乖離検知）。#906（`site_css_contract.rs` の
/// 作り替え）が先にマージされた場合は本テストを新契約へ統合する
/// （検証意図は不変）。
#[test]
fn sidebar_nav_list_parts_are_covered_by_generated_site_css() {
    let css = site_css();
    let nav = fixture_nav();
    let html = render(&sidebar(&nav, "/quickstart/"));

    for part in ["root", "heading", "list", "item", "link"] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "sidebar html should contain data-part=\"{part}\""
        );
    }
    assert!(html.contains(r#"aria-current="page""#));

    for selector in [
        r#"[data-scope="nav-list"][data-part="heading"]"#,
        r#"[data-scope="nav-list"][data-part="list"]"#,
        r#"[data-scope="nav-list"][data-part="link"]"#,
        r#"[data-scope="nav-list"][data-part="link"][aria-current="page"]"#,
    ] {
        assert!(
            css.contains(selector),
            "generated assets/site.css should contain selector {selector}"
        );
    }
}

/// イシュー #776 の乖離検知テスト: `layout::docs_page` が全ページ骨格へ
/// 常時挿入する SkipNav の `link`/`content` パーツセレクタ
/// （`data-scope="skip-nav"`）が、`crate::skip_nav::stylesheet()` が生成する
/// CSS 側に実在することを検証する（admonition の
/// `admonition_markdown_output_classes_are_covered_by_generated_admonition_css`
/// と同型の「実出力 ⇔ 生成 CSS」乖離検知。`site/assets/site.css` はこの
/// 契約に関与しない — #715 の分離 CSS 不変条件どおり、SkipNav も専用
/// `assets/skip-nav.css` のみで完結する）。
#[test]
fn docs_page_skip_nav_parts_are_covered_by_generated_skip_nav_css() {
    use fandhe_frontend_docs_site::skip_nav;

    let skip_nav_css = skip_nav::stylesheet()
        .expect("skip_nav stylesheet should assemble")
        .as_css()
        .to_string();

    let html = render(&docs_page(
        "Fixture",
        "",
        p(vec![], vec![]),
        p(vec![], vec![]),
    ));

    assert!(html.contains(r#"data-scope="skip-nav""#));
    assert!(html.contains(r#"data-part="link""#));
    assert!(html.contains(r#"data-part="content""#));
    assert!(html.contains(r#"href="/assets/skip-nav.css""#));

    for selector in [
        r#"[data-scope="skip-nav"][data-part="link"]"#,
        r#"[data-scope="skip-nav"][data-part="content"]"#,
        r#"[data-scope="skip-nav"][data-part="link"]:focus-visible"#,
    ] {
        assert!(
            skip_nav_css.contains(selector),
            "generated assets/skip-nav.css should contain selector {selector}"
        );
    }
}

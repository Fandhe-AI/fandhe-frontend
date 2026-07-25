//! `crates/docs-site/src/layout.rs` / `nav.rs` が生成する HTML の `class`
//! 属性値と、サイト骨格 CSS（`crate::site_theme::stylesheet()` がビルド時
//! 生成する `assets/site.css`）のクラス名契約が乖離していないことを検証する
//! 回帰テスト（イシュー #488、CSS 供給方式のビルド時生成への切替はイシュー
//! #905、双方向 fail-closed 契約への作り替えはイシュー #906）。
//!
//! # 背景
//!
//! `layout.rs` は独自に `docs-*` プレフィックスの class を組み立て、
//! `nav.rs` は `sidebar` / `current` / `prev-next` / `prev` / `next` を
//! 独自に組み立てる並列実装であり、両者が実際に一致しているかは
//! コンパイラでは検証されない（CSS 文字列は Rust の型システム外）。過去に
//! `site.css` 側だけが `site-*` プレフィックスの想定クラス名で書かれ、
//! `layout.rs` の実出力（`docs-*` プレフィックス）と食い違ったまま放置され、
//! 本番 docs サイトで CSS がほぼ効かない不具合が発生した。
//!
//! # 3 層構成（イシュー #906 の作り替え）
//!
//! 本ファイルは 3 層に分かれる。層ごとの検証軸を混同しないこと。
//!
//! - **層 1（[`STRUCTURE_CLASS_CONTRACT`] 以下）**: `layout.rs`/`nav.rs` が
//!   出す新骨格 class（3 カラム・ヘッダー・目次カラム、イシュー #907〜#920）
//!   の明示的な期待表を single source of truth とし、(a) HTML に出ること・
//!   (b) 生成 `site.css` にセレクタとして出ること・(c) 表に無い `docs-*`
//!   class が HTML に現れたら失敗すること、の 3 方向を検証する。旧実装
//!   （`assert_all_classes_covered`）は (a)(b) 方向のみで、`layout.rs` が
//!   class の出力自体をやめても検知できない片方向契約だった。(c) を追加
//!   することで「class が消えても PASS する」という抜け穴を塞ぐ。
//! - **層 2（既存の部分集合網羅テスト群）**: 旧実装の `assert_all_classes_covered`
//!   系テストをそのまま維持する。層 1 とは独立に、個々の関数単位（`sidebar`
//!   単体・`prev_next_nav` 単体等）で「HTML の class ⊆ CSS のセレクタ」を
//!   確認する網羅ガードとして併存させる（弱体化させない。
//!   `.claude/rules/coding-rust.md` 「XSS 回帰テストを削除・弱体化しない」に
//!   準ずる方針をテスト全般に適用）。
//! - **層 3（[`extract_block`] 以下）**: 生成 `site.css` のダークモード
//!   custom property 契約（設計文書 `docs/design/docs-site-three-column-redesign.md`
//!   §5 第 3 項、`crates/pre-styled-ui/tests/theme_css.rs` の #732 型契約の
//!   docs-site 側ミラー）。`@media (prefers-color-scheme: dark)` ブロックと
//!   `:root[data-theme="dark"]` ブロックが同一のトークン名集合を宣言する
//!   ことを検証する。
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
//! が両者の乖離を検知する。層 1 (c) 方向の判定（[`classes_outside_contract`]）
//! は `docs-` 接頭辞トークンのみを対象にすることで、`language-*`/`fd-alert--*`
//! のような別契約管轄のクラスを誤って層 1 の違反として扱わない。

use std::collections::HashSet;

use fandhe_frontend_core::{div, h2, h3, li, p, render, text, ul, Node};
use fandhe_frontend_docs_site::layout::{docs_page, docs_page_with_assets};
use fandhe_frontend_docs_site::nav::{header_nav, parse_nav, prev_next_nav, sidebar, Nav};
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
/// 発生させるための最小構成。`/quickstart/`（2 番目のページ）を現在ページに
/// 指定すれば前後双方が存在する（層 1 のフィクスチャは全テストで
/// `/quickstart/` を現在ページとして使う）。
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

/// イシュー #908 の乖離検知テスト: `nav::header_nav()` が生成する
/// `docs-header-*` class（トリガー・ドロップダウン含む）がすべて
/// 生成 `assets/site.css` にセレクタとして存在することを固定する。
/// `docs_page_with_assets` 経由でヘッダーへ埋め込んだ実配線状態
/// （`crate::build::build_site` の呼び出し形と同型）で検証する。
#[test]
fn header_nav_html_class_tokens_are_covered_by_site_css() {
    let css_tokens = extract_css_class_selectors(&site_css());
    let nav = fixture_nav();
    let node = docs_page_with_assets(
        "タイトル",
        "",
        fixture_sidebar(),
        fixture_body(),
        &[],
        Some(header_nav(&nav, "/quickstart/")),
    );
    let html = render(&node);
    assert_all_classes_covered(
        &html,
        &css_tokens,
        "nav::header_nav (via docs_page_with_assets)",
    );
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
/// 同型の「実出力 ⇔ 生成 CSS」乖離検知）。
///
/// 本テストは検証軸が層 1（`STRUCTURE_CLASS_CONTRACT`、class トークン契約）
/// とは異なるため #906 の作り替えでも統合しない: 本テストが固定するのは
/// `data-scope`/`data-part` **属性**セレクタ（headless `nav_list` の
/// anatomy 契約）であり、class トークンの集合ではない。属性セレクタは
/// [`extract_css_class_selectors`]（`.` 始まりの class セレクタのみを拾う）
/// の対象外であるため、層 1 の表に混ぜると検証漏れになる。
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

// ============================================================================
// 層 1: 明示的 class 契約表（イシュー #906 の主眼）
// ============================================================================

/// 新骨格 class 契約の single source of truth。`(class 名, 出現箇所の説明)`
/// のペアで、`layout.rs`/`nav.rs` の実出力から採取した確定値（設計文書の
/// 記載ではなく実装コードの実出力が正）。
///
/// 全ページで無条件に出現する class のみを列挙する。TOC の有無で出現が
/// 変わる class は [`TOC_ONLY_CLASSES`] / [`NO_TOC_ONLY_CLASSES`] へ分離する
/// （本表に混ぜると (a) 方向の「常に出現するはず」という検証が
/// 成立しなくなるため）。`docs-` 接頭辞を持たない既存 class
/// （`sidebar`/`prev-next`/`prev`/`next`）は [`NON_DOCS_PREFIXED_CLASSES`]
/// で別枠管理する。
const STRUCTURE_CLASS_CONTRACT: &[(&str, &str)] = &[
    (
        "docs-header",
        "header.docs-header（<body> 直下、SkipNav リンクの次）",
    ),
    ("docs-brand", "header 第 1 子のブランドリンク a"),
    ("docs-container", "3 カラム grid コンテナ div"),
    ("docs-sidebar", "左カラム aside"),
    (
        "docs-sidebar-toggle",
        "input[type=checkbox]（aside.docs-sidebar 先頭）",
    ),
    ("docs-sidebar-toggle-label", "上記の可視ラベル label"),
    ("docs-main", "中央カラム main"),
    ("docs-content", "本文 article"),
    ("docs-header-nav", "ヘッダードロップダウン群のコンテナ nav"),
    ("docs-header-menu", "同 ul"),
    ("docs-header-group", "セクションごとの li"),
    ("docs-header-trigger", "button[type=button]"),
    ("docs-header-dropdown", "ドロップダウン ul"),
    (
        "docs-header-actions",
        "ヘッダー右側のアクション群 div（イシュー #951）",
    ),
    (
        "docs-github-link",
        "GitHub リポジトリへの外部リンク a（イシュー #951）",
    ),
    (
        "docs-theme-toggle",
        "テーマトグル button[type=button]（既定 hidden、イシュー #951）",
    ),
];

/// 見出し（h2/h3）が 1 つ以上あるページのみ出現する class。
const TOC_ONLY_CLASSES: &[&str] = &[
    "docs-toc-aside",
    "docs-toc",
    "docs-toc-level-2",
    "docs-toc-level-3",
];

/// 見出しが 1 つも無いページのみ出現する修飾 class
/// （`docs-container` と併記される）。
const NO_TOC_ONLY_CLASSES: &[&str] = &["docs-container--no-toc"];

/// `docs-` 接頭辞を持たない既存 class（`nav.rs` 由来）。イシュー #906 の
/// スコープでは `docs-` への統一は行わず、現状を明示的に固定するに留める
/// （out-of-scope-tracking の対象。計画本文 §9 参照）。
const NON_DOCS_PREFIXED_CLASSES: &[&str] = &["sidebar", "prev-next", "prev", "next"];

/// `crate::build::build_site` の実組み立て（`docs_page_with_assets` +
/// `header_nav` + `prev_next_nav` の同時配線、`build.rs` の
/// `docs_page_with_assets` 呼び出しと同型）と同じ形の 1 枚のフィクスチャを
/// 生成する。本文へ `prev_next_nav` を含め、ヘッダーへ `header_nav` を渡す
/// ことで layout.rs / nav.rs の全 class を 1 回のレンダリングで発生させる。
///
/// `with_headings = true`: 本文に h2/h3 を含む（[`TOC_ONLY_CLASSES`] が
/// 出現し、[`NO_TOC_ONLY_CLASSES`] は出現しない）。
/// `with_headings = false`: 見出し無し（逆の関係）。
///
/// フィクスチャ現在ページは `/quickstart/`（[`fixture_nav`] の中間ページ）
/// 固定とし、`prev`/`next` 双方・`docs-header-group` 複数（2 セクション）を
/// 同時に発生させる。
fn full_page_html(with_headings: bool) -> String {
    let nav = fixture_nav();
    let sidebar_node = sidebar(&nav, "/quickstart/");
    let header_nav_node = header_nav(&nav, "/quickstart/");
    let prev_next_node = prev_next_nav(&nav, "/quickstart/");

    let mut body_children: Vec<Node> = if with_headings {
        vec![
            h2(vec![], vec![text("導入")]),
            p(vec![], vec![text("本文です。")]),
            h3(vec![], vec![text("詳細")]),
            p(vec![], vec![text("詳細本文です。")]),
        ]
    } else {
        vec![p(vec![], vec![text("見出しの無い本文です。")])]
    };
    body_children.push(prev_next_node);
    let body = div(vec![], body_children);

    let node = docs_page_with_assets(
        "タイトル",
        "",
        sidebar_node,
        body,
        &[],
        Some(header_nav_node),
    );
    render(&node)
}

/// `html` から抽出した class トークンのうち、`docs-` で始まり、かつ
/// [`STRUCTURE_CLASS_CONTRACT`] のキー集合に含まれないものを返す（空なら
/// 層 1 (c) 方向の契約違反なし）。純関数として実装し、
/// [`contract_violation_is_detected_for_unknown_docs_class`]（層 1 の
/// ヘルパ自己テスト）でプロダクションコードを改変せずに判定能力を
/// 独立検証できるようにする（`.claude/rules/coding-rust.md` のテスト
/// 規約に沿い、`#[ignore]`・条件緩和に頼らない fail-closed 判定を関数として
/// 切り出す）。
fn classes_outside_contract(html: &str) -> Vec<String> {
    let contract: HashSet<&str> = STRUCTURE_CLASS_CONTRACT
        .iter()
        .map(|(name, _)| *name)
        .collect();
    let mut violations: Vec<String> = extract_class_tokens(html)
        .into_iter()
        .filter(|token| token.starts_with("docs-") && !contract.contains(token.as_str()))
        .collect();
    violations.sort();
    violations
}

#[test]
fn structure_class_contract_appears_in_rendered_html() {
    let html_with_toc = full_page_html(true);
    let html_no_toc = full_page_html(false);
    let tokens_with_toc = extract_class_tokens(&html_with_toc);
    let tokens_no_toc = extract_class_tokens(&html_no_toc);

    for (class, description) in STRUCTURE_CLASS_CONTRACT {
        assert!(
            tokens_with_toc.contains(*class),
            "{class}（{description}）がフルページフィクスチャ（見出しあり）の HTML に出現しない"
        );
        assert!(
            tokens_no_toc.contains(*class),
            "{class}（{description}）がフルページフィクスチャ（見出しなし）の HTML に出現しない"
        );
    }

    for class in TOC_ONLY_CLASSES {
        assert!(
            tokens_with_toc.contains(*class),
            "{class} は見出しありページで出現するはずだが出現しない"
        );
        assert!(
            !tokens_no_toc.contains(*class),
            "{class} は見出しなしページで出現しないはずだが出現した"
        );
    }

    for class in NO_TOC_ONLY_CLASSES {
        assert!(
            !tokens_with_toc.contains(*class),
            "{class} は見出しありページで出現しないはずだが出現した"
        );
        assert!(
            tokens_no_toc.contains(*class),
            "{class} は見出しなしページで出現するはずだが出現しない"
        );
    }
}

#[test]
fn structure_class_contract_has_selector_in_generated_site_css() {
    let css_tokens = extract_css_class_selectors(&site_css());

    for (class, description) in STRUCTURE_CLASS_CONTRACT {
        assert!(
            css_tokens.contains(*class),
            "{class}（{description}）が生成 assets/site.css にセレクタとして存在しない"
        );
    }
    for class in TOC_ONLY_CLASSES.iter().chain(NO_TOC_ONLY_CLASSES) {
        assert!(
            css_tokens.contains(*class),
            "{class} が生成 assets/site.css にセレクタとして存在しない"
        );
    }
    for class in NON_DOCS_PREFIXED_CLASSES {
        assert!(
            css_tokens.contains(*class),
            "{class} が生成 assets/site.css にセレクタとして存在しない"
        );
    }
}

/// イシュー #951 受入条件（JS 無効時にトグル非表示 + `prefers-color-scheme`
/// 追従）の機械固定: 生成 `assets/site.css` に `.docs-theme-toggle[hidden]`
/// セレクタが存在し、`display: none` を宣言することを確認する。
/// `crate::layout` が既定で `hidden` 属性を付与し、`crate::script::SITE_JS`
/// のイベント配線完了後にのみこれを除去する契約（`crate::script` モジュール
/// doc 手順 5）の CSS 側の裏付け。
#[test]
fn generated_site_css_hides_theme_toggle_while_hidden_attribute_is_present() {
    let css = site_css();
    let start = css
        .find(".docs-theme-toggle[hidden]")
        .expect(".docs-theme-toggle[hidden] セレクタが生成 assets/site.css に存在しない");
    let block_start = css[start..]
        .find('{')
        .expect(".docs-theme-toggle[hidden] のルールブロック開始 { が見つからない");
    let block_end = css[start + block_start..]
        .find('}')
        .expect(".docs-theme-toggle[hidden] のルールブロック終了 } が見つからない");
    let block = &css[start + block_start..start + block_start + block_end];
    assert!(
        block.contains("display: none"),
        ".docs-theme-toggle[hidden] は display: none を宣言している必要がある: {block}"
    );
}

/// Bugbot 指摘（PR #967, イシュー #951）の回帰ガード。`min-width: 768px`
/// 帯域で `.docs-header-actions` の `margin-left: auto` を打ち消す override
/// が、`.docs-header-nav` 直後に限定した隣接セレクタ
/// （`.docs-header-nav + .docs-header-actions`）であることを固定する。
///
/// `crate::layout::docs_page` は `header_nav` が `None`（`docs_page` 単体
/// 呼び出し等）でも `.docs-header-actions` を無条件出力するが
/// `.docs-header-nav` 自体は出力しない。override が無条件セレクタ
/// （`.docs-header-actions { margin-left: 0.75rem; }`）のままだと、
/// `header_nav: None` の構成で `min-width: 768px` 以上において基底帯域の
/// `margin-left: auto` が打ち消され、GitHub リンク・テーマトグルがヘッダー
/// 右端（トレイリングエッジ）ではなくブランド直後に居座ってしまう
/// （`docs/design/docs-site-three-column-redesign.md` の想定レイアウトから
/// の逸脱）。隣接セレクタなら `.docs-header-nav` が存在しない構成では
/// override 自体が不成立のままとなり、基底帯域の `margin-left: auto` が
/// 有効であり続けるため右端配置が保たれる。
#[test]
fn header_actions_margin_override_is_scoped_to_header_nav_sibling() {
    let css = site_css();
    assert!(
        css.contains(".docs-header-nav + .docs-header-actions"),
        "min-width: 768px 帯域の .docs-header-actions margin override は \
         .docs-header-nav + .docs-header-actions（隣接セレクタ）に限定する必要がある: \
         header_nav が None の構成でトレイリングエッジ配置が崩れる（イシュー #951 Bugbot 指摘）"
    );
    // 上の contains チェックだけでは「隣接セレクタが *どこかに* 存在する」
    // ことしか確認できず、無条件セレクタ `.docs-header-actions { ... }` が
    // 別途 margin-left: 0.75rem を宣言していても検知できない（インデント差
    // による厳密な部分文字列一致は整形変更で偽陰性になるため使わない）。
    // 生成 CSS 中の全 `.docs-header-actions {` 開始位置を洗い出し、その
    // ルールブロックが `margin-left: 0.75rem` を宣言する場合は必ず直前に
    // `.docs-header-nav + ` が付いている（無条件セレクタ単体では
    // override が成立しない）ことを構造的に確認する。
    let mut search_from = 0usize;
    let mut found_override_block = false;
    while let Some(rel_selector_end) = css[search_from..].find(".docs-header-actions {") {
        let selector_end = search_from + rel_selector_end + ".docs-header-actions {".len();
        let block_close_rel = css[selector_end..]
            .find('}')
            .expect(".docs-header-actions ルールブロックの閉じ } が見つからない");
        let block = &css[selector_end..selector_end + block_close_rel];
        if block.contains("margin-left: 0.75rem") {
            found_override_block = true;
            let selector_start = search_from + rel_selector_end;
            let preceding = &css[..selector_start];
            assert!(
                preceding.ends_with(".docs-header-nav + "),
                "margin-left: 0.75rem を宣言する .docs-header-actions ルールは \
                 直前が `.docs-header-nav + `（隣接セレクタ）でなければならない \
                 （無条件セレクタでの override 復活はイシュー #951 の配置崩れを再発させる）: \
                 selector 開始位置直前の文字列 = {:?}",
                &preceding[preceding.len().saturating_sub(40)..]
            );
        }
        search_from = selector_end;
    }
    assert!(
        found_override_block,
        "margin-left: 0.75rem を宣言する .docs-header-actions ルールが \
         生成 CSS に見つからない（override 自体が削除されていないか確認）"
    );
}

/// 層 1 (c) 方向の主眼テスト: フルページフィクスチャの HTML に、
/// [`STRUCTURE_CLASS_CONTRACT`]（+ TOC 条件付き class）に無い `docs-*`
/// class が 1 件でも現れたら失敗する。`layout.rs`/`nav.rs` が無断で新しい
/// class を追加し、対応する表の更新を忘れた場合にこのテストが検知する
/// （旧実装が持たなかった検証方向）。
///
/// 併せて `docs-` 以外の残余トークンが [`NON_DOCS_PREFIXED_CLASSES`] と
/// 完全一致することも確認する（未知の非 `docs-` プレフィックス class の
/// 混入も検知する）。
#[test]
fn rendered_html_has_no_class_outside_the_contract() {
    let toc_classes: HashSet<String> = TOC_ONLY_CLASSES
        .iter()
        .chain(NO_TOC_ONLY_CLASSES)
        .map(|s| s.to_string())
        .collect();
    let expected_non_docs: HashSet<String> = NON_DOCS_PREFIXED_CLASSES
        .iter()
        .map(|s| s.to_string())
        .collect();

    for with_headings in [true, false] {
        let html = full_page_html(with_headings);

        // TOC 条件付き class は classes_outside_contract の判定対象外
        // （STRUCTURE_CLASS_CONTRACT 本体には含めない設計のため、ここで
        // 誤検知しないことを別途確認する）。
        let violations: Vec<String> = classes_outside_contract(&html)
            .into_iter()
            .filter(|c| !toc_classes.contains(c))
            .collect();
        assert!(
            violations.is_empty(),
            "with_headings={with_headings}: 契約表に無い docs-* class が HTML に出現した: {violations:?}"
        );

        let non_docs_tokens: HashSet<String> = extract_class_tokens(&html)
            .into_iter()
            .filter(|t| !t.starts_with("docs-"))
            .collect();
        assert_eq!(
            non_docs_tokens, expected_non_docs,
            "with_headings={with_headings}: docs- 接頭辞を持たない class トークン集合が NON_DOCS_PREFIXED_CLASSES と一致しない"
        );
    }
}

#[test]
fn contract_violation_is_detected_for_unknown_docs_class() {
    // ヘルパの自己テスト: プロダクションコードを一切改変せずに、
    // classes_outside_contract が「表に無い docs-* class」を検知できることを
    // 合成 HTML で証明する（CI では本テストのみが常時走り、実装改変を伴う
    // ドリフト注入確認は実装者の手元検証のみで行う。計画 §6 手順 5 参照）。
    let html = r#"<div class="docs-header docs-unknown-thing"></div>"#;
    let violations = classes_outside_contract(html);
    assert_eq!(violations, vec!["docs-unknown-thing".to_string()]);
}

// ============================================================================
// 層 3: 生成 CSS のテーマトークン整合（受入条件 2、#732 型契約の docs-site 側ミラー）
// ============================================================================

/// `marker` で始まるブロックを、最初の行頭 `}`（`"\n}\n"`）までで切り出す。
/// `Theme::to_css` の出力書式（`crates/pre-styled-ui/src/theme.rs`、
/// `crates/pre-styled-ui/tests/theme_css.rs::custom_theme_output_matches_full_snapshot`
/// が凍結）では `@media (prefers-color-scheme: dark)` ブロックの内側
/// `:root:not(...)` の閉じ括弧はインデント付き（`  }`）であるため、行頭 `}`
/// は必ず外側の閉じ括弧になる（内側の `  }\n` は直前に空白があるため
/// `"\n}\n"` にマッチしない）。
fn extract_block<'a>(css: &'a str, marker: &str) -> &'a str {
    let start = css
        .find(marker)
        .unwrap_or_else(|| panic!("marker not found in css: {marker}"));
    let after = &css[start..];
    let close_at = after.find("\n}\n").unwrap_or_else(|| {
        panic!("closing brace (line-start `}}`) not found for marker: {marker}")
    });
    &after[..close_at + "\n}\n".len()]
}

/// ブロック内の custom property **宣言**名を集める。名前の直後に `: ` が
/// 続くものだけを宣言とみなす（`var(--fandhe-space-4)` のような参照を
/// 宣言と誤認しないための必須条件。参照の直後は `)` 等になり `: ` が続かない）。
fn collect_declared_token_names(block: &str) -> HashSet<String> {
    let mut names = HashSet::new();
    let chars: Vec<char> = block.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '-' && chars.get(i + 1) == Some(&'-') {
            let mut j = i + 2;
            let mut token = String::from("--");
            while j < chars.len()
                && (chars[j].is_ascii_alphanumeric() || chars[j] == '-' || chars[j] == '_')
            {
                token.push(chars[j]);
                j += 1;
            }
            if chars.get(j) == Some(&':') && chars.get(j + 1) == Some(&' ') {
                names.insert(token);
            }
            i = j;
            continue;
        }
        i += 1;
    }
    names
}

#[test]
fn generated_site_css_dark_blocks_declare_the_same_token_names() {
    let css = site_css();
    let media_block = extract_block(&css, "@media (prefers-color-scheme: dark) {");
    let data_theme_block = extract_block(&css, ":root[data-theme=\"dark\"] {");

    let media_names = collect_declared_token_names(media_block);
    let data_theme_names = collect_declared_token_names(data_theme_block);

    assert!(
        !media_names.is_empty(),
        "@media (prefers-color-scheme: dark) ブロックから custom property 宣言が 1 件も抽出できなかった"
    );
    assert_eq!(
        media_names, data_theme_names,
        "@media (prefers-color-scheme: dark) と :root[data-theme=\"dark\"] の宣言トークン名集合が一致しない"
    );
}

#[test]
fn generated_site_css_declares_docs_specific_tokens_in_both_dark_blocks() {
    let css = site_css();
    let media_block = extract_block(&css, "@media (prefers-color-scheme: dark) {");
    let data_theme_block = extract_block(&css, ":root[data-theme=\"dark\"] {");

    assert!(
        collect_declared_token_names(media_block).contains("--fandhe-color-docs-accent-bg"),
        "docs 固有トークン --fandhe-color-docs-accent-bg が @media dark ブロックに無い"
    );
    assert!(
        collect_declared_token_names(data_theme_block).contains("--fandhe-color-docs-accent-bg"),
        "docs 固有トークン --fandhe-color-docs-accent-bg が :root[data-theme=\"dark\"] ブロックに無い"
    );
}

#[test]
fn generated_site_css_orders_data_theme_block_after_media_query_block() {
    // `@media` と `:root[data-theme="dark"]` は同特異度のため、後勝ちである
    // CSS のカスケード規則上、`data-theme` ブロックが出力順で後に来ることが
    // 「明示指定が OS 設定より常に勝つ」という仕様の必須条件になる
    // （`crates/pre-styled-ui/tests/theme_css.rs::data_theme_dark_block_is_ordered_after_media_query_block`
    // の docs-site 生成物版）。
    let css = site_css();

    let media_pos = css
        .find("@media (prefers-color-scheme: dark)")
        .expect("media query block must exist");
    let data_theme_pos = css
        .find(":root[data-theme=\"dark\"]")
        .expect("data-theme dark block must exist");

    assert!(
        media_pos < data_theme_pos,
        "data-theme dark block must be ordered after the media query block"
    );
}

#[test]
fn extract_block_stops_at_top_level_close_brace() {
    // ヘルパの自己テスト: ネストしたインデント付き `}` を跨いで外側 `}`
    // まで切り出すことを、Theme::to_css と同型のインデント構造を持つ合成
    // CSS で検証する。
    let css = "@media (x) {\n  :root:not(y) {\n    color: red;\n  }\n}\nafter { color: blue; }\n";
    let block = extract_block(css, "@media (x) {");
    assert!(block.contains("color: red"));
    assert!(!block.contains("color: blue"));
    assert!(block.ends_with("}\n"));
}

#[test]
fn collect_declared_token_names_ignores_var_references() {
    // ヘルパの自己テスト: `var(--fandhe-space-4)` のような参照を宣言として
    // 拾わず、`--fandhe-color-bg: #fff;` のような宣言のみを拾うことを検証する。
    let block = "  --fandhe-color-bg: #fff;\n  margin: var(--fandhe-space-4);\n";
    let names = collect_declared_token_names(block);
    assert!(names.contains("--fandhe-color-bg"));
    assert!(!names.contains("--fandhe-space-4"));
}

#[test]
fn dark_block_token_sets_mismatch_is_detected() {
    // ヘルパの自己テスト: 片方のブロックにトークンが 1 件多い合成 CSS に
    // 対して、集合が不一致になることを固定する（本物の site.css で両者が
    // 一致した場合に「たまたま一致している」のではなく「不一致なら検知
    // できる」ことを独立に証明する）。
    let media_block = "@media (x) {\n  --fandhe-color-bg: #000;\n  --fandhe-color-fg: #fff;\n}\n";
    let data_theme_block = "[data-theme] {\n  --fandhe-color-bg: #000;\n}\n";

    let media_names = collect_declared_token_names(media_block);
    let data_theme_names = collect_declared_token_names(data_theme_block);

    assert_ne!(media_names, data_theme_names);
}

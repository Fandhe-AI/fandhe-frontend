//! Blocks（`/blocks/`）ページの契約テスト（イシュー #2088）。
//!
//! `build_site` で実サイトをビルドし、生成物に対して以下を固定する。
//! いずれも設計文書 `docs/design/docs-site-blocks-section.md` と実装計画
//! §2.3/§2.6/§2.7/§6 が定める不変条件であり、後続イシュー #2089〜#2095 が
//! block を追加する際もそのまま継承される。
//!
//! - `/blocks/login-01/` に `class="blocks-demo"` と `pre-styled-ui.css` /
//!   `blocks.css` の `<link>` があり、`/blocks/` 索引ページには無い
//! - block ページ全体に `<form` が無い（無 JS 制約、`crate::layout` 参照）
//! - `src="data:` が無い（A05、`showcase::image_demo_svg` と同じ判断軸）
//! - 節順序が H1 → Demo → 使用部品 → Rust コード
//! - 使用部品リンクが各 `Part.path` を指す
//! - 合成関数の出力を直接 `render()` した結果に未エスケープ `<script` が
//!   無い（XSS 回帰）
//! - `blocks::stylesheet()` が `.blocks-demo` の `overflow-x` 宣言を含む
//! - `login-01` の `data-blocks-login-01-card`/`-field`/`-submit` 属性が
//!   生成 HTML に実際に出力され、`blocks::stylesheet()` にも対応する
//!   `[data-blocks-login-01-*]` セレクタが存在する（イシュー #2088 PR #2277
//!   codex-review P1 / Cursor Bugbot 指摘の是正: `card::root`/`field::root`/
//!   `button::button` は `drop_class_attr` で呼び出し側 `class` を除去する
//!   ため、これら 3 パーツの CSS フックは `class` ではなく `data-*` 属性で
//!   渡す契約に変更した）

use std::path::{Path, PathBuf};

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::blocks;

#[path = "support/shared_site.rs"]
mod shared_site;

fn repo_root() -> PathBuf {
    shared_site::repo_root()
}

/// 実サイトビルドの生成物ディレクトリを返す（読み取り専用）。
///
/// 従来は本ファイル内でテストごとに `build_site` を再実行していた
/// （13 テストで 13 回の実サイトフルビルド、イシュー #2299）が、いずれの
/// テストも生成物を読み取るだけで書き込み・削除は行わないため、
/// `tests/support/shared_site.rs` の共有ビルド（テストバイナリ内で 1 回だけ
/// 実行）へ切り替えた。返す `&Path` は共有ビルドの出力を指すため、呼び出し
/// 元で書き込み・削除しないこと。
fn build_real_site() -> &'static Path {
    shared_site::real_site().out_dir.as_path()
}

#[test]
fn login_01_page_wires_demo_class_and_both_stylesheets_index_page_does_not() {
    let out = build_real_site();

    let login_html = std::fs::read_to_string(out.join("blocks/login-01/index.html"))
        .expect("blocks/login-01/index.html should be generated");
    assert!(
        login_html.contains("class=\"blocks-demo blocks-login-01\""),
        "login-01 page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        login_html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "login-01 page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        login_html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "login-01 page should link the Blocks-specific stylesheet"
    );
    // codex-review P1 是正（イシュー #2088 PR #2277 指摘）: card::root/
    // field::root/button::button は drop_class_attr で呼び出し側 class を
    // 除去するため、blocks.css のレイアウト規則は class ではなく data-*
    // 属性へ張り替えた（login_01.rs の実装コメント参照）。生成 HTML に
    // その属性が実際に出力され、blocks.css 側にも対応するセレクタが
    // 存在することの両方を固定し、「CSS フックが黙って効かない」再発を防ぐ。
    for hook in [
        "data-blocks-login-01-card=\"\"",
        "data-blocks-login-01-field=\"\"",
        "data-blocks-login-01-submit=\"\"",
    ] {
        assert!(
            login_html.contains(hook),
            "login-01 page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-login-01-card]",
        "[data-blocks-login-01-field]",
        "[data-blocks-login-01-submit]",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }

    let index_html = std::fs::read_to_string(out.join("blocks/index.html"))
        .expect("blocks/index.html should be generated");
    assert!(
        !index_html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "Blocks index page should not link pre-styled-ui.css (no generated content)"
    );
    assert!(
        !index_html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "Blocks index page should not link blocks.css (no Demo section)"
    );
}

/// login-01 の合成部品（card/field::group/input/button の 3 variant）が
/// shadcn `login-01` 相当の構成で実際に出力されていること、`<form>`・死
/// リンク（`href="#"`）・`field::error_text` の `role="alert"`（#2092 で
/// DOM から除去）・`card::footer`（同、`card::body` 側へ移設）を持ち込んで
/// いないことを固定する（イシュー #2092、`sidebar_03_composes_expected_parts`
/// と同型）。
#[test]
fn login_01_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/login-01/index.html"))
        .expect("blocks/login-01/index.html should be generated");
    for needle in [
        "data-scope=\"card\"",
        "data-part=\"group\"",
        "type=\"email\"",
        "placeholder=\"m@example.com\"",
        "type=\"password\"",
        "fd-button--variant-outline",
        "fd-button--variant-link",
        "Login with SSO",
    ] {
        assert!(
            html.contains(needle),
            "login-01 page should contain {needle}"
        );
    }
    for absent in [
        "<form",
        "href=\"#\"",
        "role=\"alert\"",
        "data-part=\"footer\"",
    ] {
        assert!(
            !html.contains(absent),
            "login-01 should never contain {absent}"
        );
    }
}

#[test]
fn block_pages_never_contain_a_form_element_or_data_uri() {
    let out = build_real_site();
    let mut relatives: Vec<String> = vec!["blocks/index.html".to_string()];
    relatives.extend(blocks::BLOCKS.iter().map(|block| {
        let kebab = block
            .path
            .trim_start_matches("/blocks/")
            .trim_end_matches('/');
        format!("blocks/{kebab}/index.html")
    }));
    for relative in relatives {
        let html = std::fs::read_to_string(out.join(&relative))
            .unwrap_or_else(|e| panic!("{relative}: {e}"));
        assert!(
            !html.contains("<form"),
            "{relative} should never contain a <form> element (no-JS implicit submit avoidance)"
        );
        assert!(
            !html.contains("src=\"data:"),
            "{relative} should never contain a data: URI src"
        );
    }
}

/// dashboard-01 の Demo 固有 CSS フック（sidebar/inset/stats 等）が実際に
/// 生成 HTML へ出力され、`blocks::stylesheet()` にも対応するセレクタが
/// 存在することを固定する（login-01 のイシュー #2088 codex-review 是正と
/// 同型: `sidebar` の全パーツ・`card::root`・`stat::root` 等は
/// `drop_class_attr` で呼び出し側 `class` を除去するため、CSS フックは
/// `class` ではなく `data-*` 属性で渡す契約になっている、
/// `crates/docs-site/src/blocks/dashboard_01.rs` モジュール doc参照）。
#[test]
fn dashboard_01_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/dashboard-01/index.html"))
        .expect("blocks/dashboard-01/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-dashboard-01\""),
        "dashboard-01 page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "dashboard-01 page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "dashboard-01 page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-dashboard-01-card=\"\"",
        "data-blocks-dashboard-01-header=\"\"",
        "data-blocks-dashboard-01-chart=\"\"",
        "data-blocks-dashboard-01-table=\"\"",
        "data-blocks-dashboard-01-header-link=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "dashboard-01 page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-dashboard-01-card]",
        "[data-blocks-dashboard-01-header]",
        "[data-blocks-dashboard-01-chart]",
        "[data-blocks-dashboard-01-table]",
        "[data-blocks-dashboard-01-header-link]",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// sidebar-07 の Demo 固有 CSS フック（stack/header/nav-trigger/label/
/// chevron 等）が実際に生成 HTML へ出力され、`blocks::stylesheet()` にも
/// 対応するセレクタが存在することを固定する（login-01/dashboard-01 の
/// イシュー #2088 codex-review 是正と同型: `sidebar` の全パーツ・
/// `menu::root`・`avatar::root`・`breadcrumb::root`・`separator::separator`
/// は `drop_class_attr` で呼び出し側 `class` を除去するため、CSS フックは
/// `class` ではなく `data-*` 属性で渡す契約になっている、
/// `crates/docs-site/src/blocks/sidebar_07.rs` モジュール doc参照）。
#[test]
fn sidebar_07_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/sidebar-07/index.html"))
        .expect("blocks/sidebar-07/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-sidebar-07\""),
        "sidebar-07 page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "sidebar-07 page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "sidebar-07 page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-sidebar-07-instance=\"\"",
        "data-blocks-sidebar-07-stack=\"\"",
        "data-blocks-sidebar-07-header=\"\"",
        "data-blocks-sidebar-07-nav-trigger=\"\"",
        "data-blocks-sidebar-07-label=\"\"",
        "data-blocks-sidebar-07-chevron=\"\"",
        "data-blocks-sidebar-07-grid=\"\"",
        "data-blocks-sidebar-07-placeholder=\"\"",
        "data-blocks-sidebar-07-placeholder-lg=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "sidebar-07 page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-sidebar-07-stack]",
        "[data-blocks-sidebar-07-header]",
        "[data-blocks-sidebar-07-nav-trigger]",
        "[data-blocks-sidebar-07-label]",
        "[data-blocks-sidebar-07-chevron]",
        "[data-blocks-sidebar-07-grid]",
        "[data-blocks-sidebar-07-placeholder]",
        "[data-blocks-sidebar-07-placeholder-lg]",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
    // icon 折りたたみ時にラベルを clip する規則（`display: none` ではなく
    // アクセシブルネームを保つ手法、モジュール doc「icon 折りたたみ時に
    // 自動で隠れないものへの補完 CSS」参照）が実在すること。
    assert!(
        sheet_css.contains(
            "[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-blocks-sidebar-07-label]"
        ),
        "blocks.css should clip [data-blocks-sidebar-07-label] when the sidebar is icon-collapsed"
    );
    // `data-blocks-sidebar-07-instance` は provider 要素自身に付与される
    // ため、対応する min-height/min-width 上書き規則は子孫コンビネータ
    // （属性セレクタ間の空白）ではなく同一要素への複合セレクタでなければ
    // マッチしない（codex-review/Bugbot 指摘の回帰防止、イシュー #2090）。
    assert!(
        sheet_css.contains(
            "[data-blocks-sidebar-07-instance][data-scope=\"sidebar\"][data-part=\"provider\"]"
        ),
        "blocks.css should target [data-blocks-sidebar-07-instance] as a compound selector on the provider element, not a descendant combinator"
    );
}

/// sidebar-07 の合成部品（sidebar/collapsible/menu/avatar/breadcrumb/
/// separator）が anatomy の `data-*` として実際に出力されていること、
/// expanded/collapsed 双方のインスタンスが存在すること、死リンク
/// （`href="#"`）が無いことを固定する。
#[test]
fn sidebar_07_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/sidebar-07/index.html"))
        .expect("blocks/sidebar-07/index.html should be generated");
    for needle in [
        "data-scope=\"sidebar\"",
        "data-collapsible=\"icon\"",
        "data-state=\"collapsed\"",
        "data-state=\"expanded\"",
        "data-scope=\"collapsible\"",
        "data-scope=\"breadcrumb\"",
        "data-scope=\"menu\"",
        "data-scope=\"avatar\"",
        "data-active",
    ] {
        assert!(
            html.contains(needle),
            "sidebar-07 page should contain {needle}"
        );
    }
    assert!(
        !html.contains("<form"),
        "sidebar-07 should never contain a <form>"
    );
    assert!(
        !html.contains("href=\"#\""),
        "sidebar-07 should never contain a dead href=\"#\" link"
    );
}

/// sidebar-03 の Demo が `blocks-demo`/block 固有 class・pre-styled-ui.css/
/// blocks.css の `<link>`・`data-blocks-sidebar-03-*` CSS フックを実際に
/// 出力し、`blocks::stylesheet()` に対応するセレクタが存在することを固定
/// する（`sidebar_07_page_wires_demo_class_and_css_hooks` と同型）。
#[test]
fn sidebar_03_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/sidebar-03/index.html"))
        .expect("blocks/sidebar-03/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-sidebar-03\""),
        "sidebar-03 page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "sidebar-03 page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "sidebar-03 page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-sidebar-03-instance=\"\"",
        "data-blocks-sidebar-03-brand=\"\"",
        "data-blocks-sidebar-03-brand-icon=\"\"",
        "data-blocks-sidebar-03-parent=\"\"",
        "data-blocks-sidebar-03-header=\"\"",
        "data-blocks-sidebar-03-grid=\"\"",
        "data-blocks-sidebar-03-placeholder=\"\"",
        "data-blocks-sidebar-03-placeholder-lg=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "sidebar-03 page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-sidebar-03-brand-icon]",
        "[data-blocks-sidebar-03-parent]",
        "[data-blocks-sidebar-03-header]",
        "[data-blocks-sidebar-03-grid]",
        "[data-blocks-sidebar-03-placeholder]",
        "[data-blocks-sidebar-03-placeholder-lg]",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
    // `data-blocks-sidebar-03-instance` は provider 要素自身に付与される
    // ため、対応する min-height/min-width 上書き規則は子孫コンビネータ
    // ではなく同一要素への複合セレクタでなければマッチしない（sidebar-07
    // の codex-review/Bugbot 指摘の回帰防止と同型、イシュー #2091）。
    assert!(
        sheet_css.contains(
            "[data-blocks-sidebar-03-instance][data-scope=\"sidebar\"][data-part=\"provider\"]"
        ),
        "blocks.css should target [data-blocks-sidebar-03-instance] as a compound selector on the provider element, not a descendant combinator"
    );
}

/// sidebar-03 の合成部品（sidebar/breadcrumb/separator/icon）が anatomy の
/// `data-*` として実際に出力されていること、現在項目に `data-active` が
/// 付与されること、`collapsible`/`menu`/`avatar` scope を持ち込んでいない
/// こと（sidebar-07 との構成上の区別）、死リンク・`<form>` が無いことを
/// 固定する。
#[test]
fn sidebar_03_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/sidebar-03/index.html"))
        .expect("blocks/sidebar-03/index.html should be generated");
    for needle in [
        "data-scope=\"sidebar\"",
        "data-part=\"menu-sub\"",
        "data-part=\"menu-sub-button\"",
        "data-active",
        "data-state=\"expanded\"",
        "data-collapsible=\"offcanvas\"",
        "data-scope=\"breadcrumb\"",
        "aria-current=\"page\"",
        "data-size=\"lg\"",
    ] {
        assert!(
            html.contains(needle),
            "sidebar-03 page should contain {needle}"
        );
    }
    for absent in [
        "<form",
        "href=\"#\"",
        "data-scope=\"collapsible\"",
        "data-scope=\"menu\"",
        "data-scope=\"avatar\"",
    ] {
        assert!(
            !html.contains(absent),
            "sidebar-03 should never contain {absent}"
        );
    }
}

/// Demo が使う `aria-controls`/`aria-labelledby`/`aria-describedby` の
/// 参照先 `id` が同一 Demo 出力内に実在し、`id` が重複しないことを固定する
/// （対象はページ全体ではなく `(block.demo)()` の部分木のみ。レイアウト側
/// 〔インライン TOC・aside TOC・検索入力〕の id を巻き込んで偽陽性にしない
/// ため）。sidebar-07 は expanded/collapsed 2 インスタンス分の id を
/// suffix で分けており、suffix 漏れによる id 衝突・宙ぶらりん参照を
/// 検知する（dashboard-01 の codex-review P1/Bugbot 指摘と同型の回帰
/// ガード）。
#[test]
fn demo_output_has_no_dangling_aria_references_or_duplicate_ids() {
    fn extract_attr_values<'a>(html: &'a str, attr: &str) -> Vec<&'a str> {
        let needle = format!("{attr}=\"");
        let mut values = Vec::new();
        let mut offset = 0usize;
        while let Some(rel_start) = html[offset..].find(&needle) {
            let start = offset + rel_start;
            // 直前が空白（属性境界）であることを要求する。`id="..."` を
            // 素の部分文字列検索で探すと `data-blocks-sidebar-07-grid=""`
            // のような無関係な属性（`grid` の末尾 `id`）を誤って `id`
            // 属性として拾ってしまうため（実測: sidebar-07 の `-grid`
            // フックが偽陽性を出した）、attr 名の直前が識別子文字（英数字・
            // `-`）でないことを境界条件として課す。
            let boundary_ok = match html[..start].chars().next_back() {
                Some(c) => !(c.is_ascii_alphanumeric() || c == '-'),
                None => true,
            };
            let after = &html[start + needle.len()..];
            let Some(end) = after.find('"') else {
                break;
            };
            if boundary_ok {
                values.push(&after[..end]);
            }
            offset = start + needle.len() + end + 1;
        }
        values
    }

    for block in blocks::BLOCKS {
        let html = render(&(block.demo)());
        let ids: Vec<&str> = extract_attr_values(&html, "id");
        let id_set: std::collections::BTreeSet<&str> = ids.iter().copied().collect();
        assert_eq!(
            ids.len(),
            id_set.len(),
            "block {} demo output should not contain duplicate id attributes: {ids:?}",
            block.path
        );

        for attr in ["aria-controls", "aria-labelledby", "aria-describedby"] {
            for value in extract_attr_values(&html, attr) {
                for referenced in value.split_whitespace() {
                    assert!(
                        id_set.contains(referenced),
                        "block {} demo output has {attr}=\"{value}\" referencing missing id=\"{referenced}\"",
                        block.path
                    );
                }
            }
        }
    }
}

/// dashboard-01 の合成部品（sidebar/stat/gradient area-chart/toggle-group/
/// tabs/table）が anatomy の `data-*` として実際に出力されていることを
/// 固定する。
#[test]
fn dashboard_01_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/dashboard-01/index.html"))
        .expect("blocks/dashboard-01/index.html should be generated");
    for needle in [
        "data-scope=\"sidebar\"",
        "data-variant=\"inset\"",
        "data-scope=\"stat\"",
        "<linearGradient",
        "data-range=\"90d\"",
        "aria-label=\"Select date range\"",
        "aria-sort=\"ascending\"",
        "data-selected",
        "data-scope=\"tabs\"",
    ] {
        assert!(
            html.contains(needle),
            "dashboard-01 page should contain {needle}"
        );
    }
    assert!(
        !html.contains("<form"),
        "dashboard-01 should never contain a <form>"
    );
    assert!(
        !html.contains("href=\"#\""),
        "dashboard-01 should never contain a dead href=\"#\" link"
    );
    assert!(
        !html.contains("aria-labelledby=\"blocks-dashboard-01-range-label\""),
        "dashboard-01 should not reference a range-toggle label id that has no matching element \
         (regression: codex-review P1 / Cursor Bugbot Low on PR #2280)"
    );
}

#[test]
fn login_01_page_orders_h1_then_demo_then_used_parts_then_rust_code() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/login-01/index.html"))
        .expect("blocks/login-01/index.html should be generated");

    // イシュー #1080: `main.docs-main` の折りたたみ目次
    // （`nav.docs-toc-inline`）は本文（`article.docs-content`）より前に
    // 置かれ、見出しテキスト（"Demo"/"使用部品"/"Rust コード"）を先出し
    // する（`crate::layout::docs_page_with_assets` 参照）。節順序の検証は
    // この重複箇所を含めず、`article.docs-content` の開始位置より後ろだけを
    // 対象にする。
    let content_start = html
        .find(r#"class="docs-content""#)
        .expect("page should have an article.docs-content");
    let content = &html[content_start..];

    // 見出し要素そのもの（`<h2>テキスト</h2>`）でしか一致しない部分文字列で
    // 探す。素の Markdown 本文（例: 導入段落中の「Rust コードで書いて
    // ください」）に見出しと同じ語が偶然出現しても誤検知しないため
    // （実測で発生した false positive、`site/blocks/login-01.md` 参照）。
    let h1_pos = content.find("<h1").expect("page should have an h1");
    let demo_pos = content
        .find(">Demo</h2>")
        .expect("page should have a Demo heading");
    let parts_pos = content
        .find(">使用部品</h2>")
        .expect("page should have a 使用部品 heading");
    let rust_pos = content
        .find(">Rust コード</h2>")
        .expect("page should have a Rust コード heading");

    assert!(
        h1_pos < demo_pos && demo_pos < parts_pos && parts_pos < rust_pos,
        "section order should be H1 -> Demo -> 使用部品 -> Rust コード, got positions {h1_pos}/{demo_pos}/{parts_pos}/{rust_pos}"
    );
}

#[test]
fn login_01_used_parts_links_point_at_each_declared_part_path() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/login-01/index.html"))
        .expect("blocks/login-01/index.html should be generated");

    let block = blocks::BLOCKS
        .iter()
        .find(|b| b.path == "/blocks/login-01/")
        .expect("login-01 should be registered");
    assert!(!block.parts.is_empty(), "login-01 should declare parts");
    for part in block.parts {
        let expected_href = format!("href=\"/fandhe-frontend{}\"", part.path);
        assert!(
            html.contains(&expected_href),
            "login-01 page should link to used part {} via {expected_href:?}",
            part.path
        );
    }
}

#[test]
fn demo_output_never_leaks_an_unescaped_script_tag() {
    for block in blocks::BLOCKS {
        let html = render(&(block.demo)());
        assert!(
            !html.contains("<script"),
            "block {} demo output should never contain an unescaped <script tag",
            block.path
        );
    }
}

#[test]
fn blocks_stylesheet_declares_demo_frame_overflow() {
    let sheet = blocks::stylesheet().expect("blocks::stylesheet should build");
    assert!(sheet.as_css().contains(".blocks-demo"));
    assert!(sheet.as_css().contains("overflow-x: auto"));
}

/// 受け入れ条件 f の実効化: 現行 Demo（`login-01`）は `<script` を含む
/// 入力を一切持たないため `demo_output_never_leaks_an_unescaped_script_tag`
/// は空虚な検証にとどまる。`crates/docs-site/src/blocks/` 配下のソース
/// そのものを走査し、`raw_html()`（REQ-1 迂回）・HTML 文字列直接組み立て
/// （`format!("<...`）を使っていないことを固定する
/// （`tests/primitive_specs_1026.rs::primitive_specs_source_does_not_use_raw_html`
/// と同型のガード。後続イシュー #2089〜#2095 が block を追加してもこの
/// 不変条件が自動的に効く）。
#[test]
fn blocks_source_does_not_use_raw_html_or_build_html_strings() {
    fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    fn code_lines_without_comments(path: &Path) -> String {
        let src = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{} should be readable: {e}", path.display()));
        src.lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    let dir = repo_root().join("crates/docs-site/src/blocks");
    let mut files = Vec::new();
    collect_rs_files(&dir, &mut files);
    assert!(
        !files.is_empty(),
        "crates/docs-site/src/blocks/ should contain at least one .rs file to guard"
    );
    for path in &files {
        let code_only = code_lines_without_comments(path);
        assert!(
            !code_only.contains("raw_html"),
            "{} must not use raw_html() (REQ-1 escape bypass) in code (non-comment) lines",
            path.display()
        );
        assert!(
            !code_only.contains("format!(\"<"),
            "{} must not build HTML strings directly (use the node-tree API)",
            path.display()
        );
    }
}

/// signup-01 ページが `blocks-demo`/block 固有 class・両スタイルシート・
/// `data-blocks-signup-01-*` CSS フックを実際に出力し、`blocks.css` 側にも
/// 対応するセレクタが存在することを固定する（login_01 と同型の検証、
/// イシュー #2094）。
#[test]
fn signup_01_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/signup-01/index.html"))
        .expect("blocks/signup-01/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-signup-01\""),
        "signup-01 page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "signup-01 page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "signup-01 page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-signup-01-card=\"\"",
        "data-blocks-signup-01-field=\"\"",
        "data-blocks-signup-01-submit=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "signup-01 page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-signup-01-card]",
        "[data-blocks-signup-01-field]",
        "[data-blocks-signup-01-submit]",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// signup-01 の合成部品（card/field::group/input/button の 3 variant +
/// helper_text 3 件）が shadcn `signup-01` 相当の構成で実際に出力されて
/// いること、`<form>`・死リンク（`href="#"`）・`field::error_text` の
/// `role="alert"`・`card::footer`・実企業名（Google）を持ち込んでいない
/// ことを固定する（login_01_composes_expected_parts と同型、イシュー
/// #2094）。
#[test]
fn signup_01_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/signup-01/index.html"))
        .expect("blocks/signup-01/index.html should be generated");
    for needle in [
        "data-scope=\"card\"",
        "data-part=\"group\"",
        "data-part=\"helper-text\"",
        "placeholder=\"John Doe\"",
        "type=\"email\"",
        "placeholder=\"m@example.com\"",
        "type=\"password\"",
        "fd-button--variant-outline",
        "fd-button--variant-link",
        "Sign up with SSO",
        "Create Account",
    ] {
        assert!(
            html.contains(needle),
            "signup-01 page should contain {needle}"
        );
    }
    for absent in [
        "<form",
        "href=\"#\"",
        "role=\"alert\"",
        "data-part=\"footer\"",
    ] {
        assert!(
            !html.contains(absent),
            "signup-01 should never contain {absent}"
        );
    }
}

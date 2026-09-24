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

/// login-04 の Demo 固有 CSS フック（card/body/form/field/submit/separator/
/// providers/provider/image/img/stack）が実際に生成 HTML へ出力され、
/// `blocks::stylesheet()` にも対応するセレクタが存在することを固定する
/// （login-01 のイシュー #2088 codex-review 是正と同型: `card::root`/
/// `field::root`/`button::button`/`image::image` は `drop_class_attr` で
/// 呼び出し側 `class` を除去するため、CSS フックは `class` ではなく
/// `data-*` 属性で渡す契約になっている、`crates/docs-site/src/blocks/
/// login_04.rs` 参照）。
#[test]
fn login_04_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/login-04/index.html"))
        .expect("blocks/login-04/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-login-04\""),
        "login-04 page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "login-04 page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "login-04 page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-login-04-stack=\"\"",
        "data-blocks-login-04-card=\"\"",
        "data-blocks-login-04-body=\"\"",
        "data-blocks-login-04-form=\"\"",
        "data-blocks-login-04-field=\"\"",
        "data-blocks-login-04-submit=\"\"",
        "data-blocks-login-04-separator=\"\"",
        "data-blocks-login-04-providers=\"\"",
        "data-blocks-login-04-provider=\"\"",
        "data-blocks-login-04-image=\"\"",
        "data-blocks-login-04-img=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "login-04 page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-login-04-stack]",
        "[data-blocks-login-04-card]",
        "[data-blocks-login-04-body]",
        "[data-blocks-login-04-form]",
        "[data-blocks-login-04-field]",
        "[data-blocks-login-04-submit]",
        "[data-blocks-login-04-providers]",
        "[data-blocks-login-04-image]",
        "[data-blocks-login-04-img]",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// login-04 の合成部品（card/field::group/field::separator/input/button/
/// icon_button/image の合成）が shadcn `login-04` 相当の構成で実際に
/// 出力されていること、実企業名・商標ロゴ（Apple/Google/Meta）・`<form>`・
/// 死リンク（`href="#"`）・`data:` URI を持ち込んでいないことを固定する
/// （イシュー #2093、`login_01_composes_expected_parts` と同型）。
#[test]
fn login_04_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/login-04/index.html"))
        .expect("blocks/login-04/index.html should be generated");
    let login_04_block = blocks::all_blocks()
        .into_iter()
        .find(|block| block.path == "/blocks/login-04/")
        .expect("login-04 should be registered in blocks::all_blocks()");
    let demo_html = render(&(login_04_block.demo)());
    for needle in [
        "data-scope=\"card\"",
        "data-part=\"group\"",
        "type=\"email\"",
        "placeholder=\"m@example.com\"",
        "type=\"password\"",
        "data-part=\"separator\"",
        "data-part=\"separator-content\"",
        "Or continue with",
        "fd-button--variant-outline",
        "fd-button--icon-only",
        "fd-button--variant-link",
        "data-scope=\"image\"",
        "src=\"../../assets/image-demo.svg\"",
        "aria-label=\"Login with",
    ] {
        assert!(
            html.contains(needle),
            "login-04 page should contain {needle}"
        );
    }
    for absent in [
        "<form",
        "href=\"#\"",
        "role=\"alert\"",
        "data-part=\"footer\"",
        "src=\"data:",
    ] {
        assert!(
            !html.contains(absent),
            "login-04 should never contain {absent}"
        );
    }
    // 実ブランド名（Apple/Google/Meta）の不在は Demo 部分木（`(block.demo)()`
    // を直接 render した出力）に対してのみ検証する。ページ全体には
    // 「shadcn 側との差分メモ」節（原稿の説明文としてブランド名へ言及）が
    // 存在するため、`html` 全体を対象にすると常に FAIL する（モジュール doc
    // 「プロバイダボタン 3 個」節が守る不変条件は Demo 部分木限定）。
    for absent in ["Apple", "Google", "Meta"] {
        assert!(
            !demo_html.contains(absent),
            "login-04 Demo subtree should never contain {absent}"
        );
    }
}

#[test]
fn block_pages_never_contain_a_form_element_or_data_uri() {
    let out = build_real_site();
    let mut relatives: Vec<String> = vec!["blocks/index.html".to_string()];
    relatives.extend(blocks::all_blocks().iter().map(|block| {
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

/// signup-05 の Demo 固有 CSS フック（stack/field/submit/providers/
/// provider）が実際に生成 HTML へ出力され、`blocks::stylesheet()`
/// にも対応するセレクタが存在することを固定する（login-01/sidebar-03 の
/// codex-review 是正と同型: `field::root`/`button::button`/`heading::heading`/
/// `icon::icon` は `drop_class_attr` で呼び出し側 `class` を除去するため、
/// CSS フックは `class` ではなく `data-*` 属性で渡す契約になっている、
/// `crates/docs-site/src/blocks/signup_05.rs` モジュール doc 参照）。
#[test]
fn signup_05_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/signup-05/index.html"))
        .expect("blocks/signup-05/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-signup-05\""),
        "signup-05 page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "signup-05 page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "signup-05 page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-signup-05-stack=\"\"",
        "data-blocks-signup-05-field=\"\"",
        "data-blocks-signup-05-submit=\"\"",
        "data-blocks-signup-05-providers=\"\"",
        "data-blocks-signup-05-provider=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "signup-05 page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-signup-05-stack]",
        "[data-blocks-signup-05-field]",
        "[data-blocks-signup-05-submit]",
        "[data-blocks-signup-05-providers]",
        "[data-blocks-signup-05-provider]",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// signup-05 の合成部品（field::group/field::root/input/button/heading/icon/
/// field::separator）が期待どおりの構成で実際に出力されていること、
/// `card`/死リンク（`href="#"`）・`<form>`・実企業名（Apple/Google/Meta）を
/// 持ち込んでいないこと、ページ内に `<h1` が 1 個のみ（Demo 内の見出しは
/// H3 として TOC を汚染しない）ことを固定する（login-01/sidebar-03 と同型）。
#[test]
fn signup_05_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/signup-05/index.html"))
        .expect("blocks/signup-05/index.html should be generated");
    for needle in [
        "data-part=\"group\"",
        "type=\"email\"",
        "placeholder=\"m@example.com\"",
        "data-part=\"separator-content\"",
        ">Or<",
        "data-scope=\"heading\"",
        "data-scope=\"icon\"",
        "aria-label=\"Acme Inc.\"",
        "fd-button--variant-outline",
        "fd-button--variant-link",
        "Create Account",
        "Continue with provider",
    ] {
        assert!(
            html.contains(needle),
            "signup-05 page should contain {needle}"
        );
    }
    for absent in [
        "<form",
        "href=\"#\"",
        "role=\"alert\"",
        "data-scope=\"card\"",
        "src=\"data:",
    ] {
        assert!(
            !html.contains(absent),
            "signup-05 should never contain {absent}"
        );
    }
    assert_eq!(
        html.matches("<h1").count(),
        1,
        "signup-05 page should contain exactly one <h1> (the page heading, not the Demo brand heading)"
    );

    // Demo 部分木のみを対象に、実ブランド名（Apple/Google/Meta）を持ち込んで
    // いないことも固定する（ページ全体に対する上の否定チェックと二重化する
    // ことで、レイアウト側の文言に依存しない検証にする）。
    let block = blocks::all_blocks()
        .into_iter()
        .find(|b| b.path == "/blocks/signup-05/")
        .expect("signup-05 block should be registered");
    let demo_html = render(&(block.demo)());
    for absent in ["Apple", "Google", "Meta"] {
        assert!(
            !demo_html.contains(absent),
            "signup-05 demo output should never contain the real brand name {absent}"
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

    for block in blocks::all_blocks() {
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

    let block = blocks::all_blocks()
        .into_iter()
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
    for block in blocks::all_blocks() {
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

/// testimonials-stack ページが `blocks-demo`/block 固有 class・両
/// スタイルシート・`data-blocks-testimonials-stack-*` CSS フックを実際に
/// 出力し、`blocks.css` 側にも対応するセレクタが存在することを固定する
/// （login-01/signup-05 と同型の検証、イシュー #2548）。
#[test]
fn testimonials_stack_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/testimonials-stack/index.html"))
        .expect("blocks/testimonials-stack/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-testimonials-stack\""),
        "testimonials-stack page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "testimonials-stack page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "testimonials-stack page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-testimonials-stack-card=\"\"",
        "data-blocks-testimonials-stack-avatar=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "testimonials-stack page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-testimonials-stack-card]",
        "[data-blocks-testimonials-stack-avatar]",
        ".blocks-testimonials-stack-stage",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// pricing-tiers-morph ページの Demo クラス・両スタイルシート・
/// `data-blocks-pricing-tiers-morph-*` CSS フックが実際に出力され、
/// `blocks::stylesheet()` にも対応するセレクタが存在することを固定する
/// （signup-05 と同型の検証、イシュー #2547）。
#[test]
fn pricing_tiers_morph_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/pricing-tiers-morph/index.html"))
        .expect("blocks/pricing-tiers-morph/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-pricing-tiers-morph\""),
        "pricing-tiers-morph page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "pricing-tiers-morph page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "pricing-tiers-morph page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-pricing-tiers-morph-grid=\"\"",
        "data-blocks-pricing-tiers-morph-tier=\"\"",
        "data-blocks-pricing-tiers-morph-footer=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "pricing-tiers-morph page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-pricing-tiers-morph-grid]",
        "[data-blocks-pricing-tiers-morph-tier]",
        "[data-blocks-pricing-tiers-morph-footer]",
        ".blocks-pricing-tiers-morph-featured",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// testimonials-stack の合成部品（card/blockquote/avatar）が期待どおりの
/// 構成で出力され、3 枚のカードのうち先頭 1 枚のみ `data-state="active"`・
/// 残り 2 枚は `data-state="inactive"` であること、`<form>`・実企業名を
/// 持ち込んでいないことを固定する（signup_05_composes_expected_parts と
/// 同型、イシュー #2548）。
#[test]
fn testimonials_stack_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/testimonials-stack/index.html"))
        .expect("blocks/testimonials-stack/index.html should be generated");
    for needle in [
        "data-scope=\"card\"",
        "data-scope=\"blockquote\"",
        "data-scope=\"avatar\"",
        "data-part=\"fallback\"",
    ] {
        assert!(
            html.contains(needle),
            "testimonials-stack page should contain {needle}"
        );
    }
    assert_eq!(
        html.matches("data-blocks-testimonials-stack-card=\"\"")
            .count(),
        3,
        "testimonials-stack should render exactly 3 testimonial cards"
    );
    assert_eq!(
        html.matches("data-state=\"active\"").count(),
        1,
        "testimonials-stack should mark exactly one card as active (the front card)"
    );
    assert_eq!(
        html.matches("data-state=\"inactive\"").count(),
        2,
        "testimonials-stack should mark exactly two cards as inactive (the back cards)"
    );
    for absent in ["<form", "href=\"#\"", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "testimonials-stack should never contain {absent}"
        );
    }
}

/// `id="<id>"` を持つタグ全体（`<` から対応する `>` まで）を抜き出す。
/// タグの属性文字列（`hidden`/`data-state` 等の有無）を検査するための
/// 補助関数（PR #2568 codex-review 指摘の是正で追加）。
fn extract_tag_by_id<'a>(html: &'a str, id: &str) -> &'a str {
    let needle = format!("id=\"{id}\"");
    let id_idx = html
        .find(&needle)
        .unwrap_or_else(|| panic!("id={id} should be present in the page"));
    let tag_start = html[..id_idx]
        .rfind('<')
        .expect("an opening '<' should precede the id attribute");
    let tag_end = html[id_idx..]
        .find('>')
        .map(|i| id_idx + i)
        .expect("a closing '>' should follow the id attribute");
    &html[tag_start..=tag_end]
}

/// pricing-tiers-morph の合成部品（tabs/card/badge/button/border-beam）が
/// 期待どおりの構成で実際に出力されていること、月額/年額の両 billing 状態
/// が実際に**可視**な状態で SSR 出力へ存在すること（無 JS 併記の回帰、
/// イシュー #2547 PR #2568 codex-review P1 指摘の是正: 単一 `tabs`
/// インスタンスへの `hidden` 併記だけでは非選択側パネルを無 JS で閲覧
/// できないため、`selected` が異なる 2 インスタンスを静的に併記する構成へ
/// 変更した）、`<form>`/死リンクを持ち込んでいないことを固定する。
#[test]
fn pricing_tiers_morph_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/pricing-tiers-morph/index.html"))
        .expect("blocks/pricing-tiers-morph/index.html should be generated");
    for needle in [
        "data-scope=\"tabs\" data-part=\"content\"",
        "data-scope=\"card\" data-part=\"root\"",
        "fd-badge--variant-solid",
        "fd-border-beam",
        "Starter",
        "Growth",
        "Enterprise",
        "$9",
        "$29",
        "$99",
        "$86",
        "$278",
        "$950",
        "Get started",
        "Contact sales",
    ] {
        assert!(
            html.contains(needle),
            "pricing-tiers-morph page should contain {needle}"
        );
    }
    for absent in ["<form", "href=\"#\"", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "pricing-tiers-morph should never contain {absent}"
        );
    }

    // 月額選択インスタンス（id プレフィックス `-monthly`）: 月額パネルは
    // 選択済み・可視（`hidden` なし・`data-state="active"`）、年額パネルは
    // 非選択のため `hidden`。
    let monthly_instance_monthly_panel =
        extract_tag_by_id(&html, "blocks-pricing-tiers-morph-monthly-content-monthly");
    assert!(
        monthly_instance_monthly_panel.contains("data-state=\"active\""),
        "monthly instance's monthly panel should be the active (visible) one"
    );
    assert!(
        !monthly_instance_monthly_panel.contains("hidden"),
        "monthly instance's monthly panel should not carry the hidden attribute"
    );
    let monthly_instance_yearly_panel =
        extract_tag_by_id(&html, "blocks-pricing-tiers-morph-monthly-content-yearly");
    assert!(
        monthly_instance_yearly_panel.contains("hidden"),
        "monthly instance's yearly panel should remain hidden"
    );

    // 年額選択インスタンス（id プレフィックス `-yearly`）: 年額パネルは
    // 選択済み・可視、月額パネルは非選択のため `hidden`。この可視な年額
    // パネルの存在が、無 JS の docs サイトでも年額プランを実際に閲覧
    // できることの回帰固定である。
    let yearly_instance_yearly_panel =
        extract_tag_by_id(&html, "blocks-pricing-tiers-morph-yearly-content-yearly");
    assert!(
        yearly_instance_yearly_panel.contains("data-state=\"active\""),
        "yearly instance's yearly panel should be the active (visible) one"
    );
    assert!(
        !yearly_instance_yearly_panel.contains("hidden"),
        "yearly instance's yearly panel should not carry the hidden attribute"
    );
    let yearly_instance_monthly_panel =
        extract_tag_by_id(&html, "blocks-pricing-tiers-morph-yearly-content-monthly");
    assert!(
        yearly_instance_monthly_panel.contains("hidden"),
        "yearly instance's monthly panel should remain hidden"
    );
}

/// pricing-usage-slider ページの Demo クラス・両スタイルシート・
/// `data-blocks-pricing-usage-slider-*` CSS フックが実際に出力され、
/// `blocks::stylesheet()` にも対応するセレクタが存在することを固定する
/// （イシュー #2547）。
#[test]
fn pricing_usage_slider_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/pricing-usage-slider/index.html"))
        .expect("blocks/pricing-usage-slider/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-pricing-usage-slider\""),
        "pricing-usage-slider page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "pricing-usage-slider page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "pricing-usage-slider page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-pricing-usage-slider-layout=\"\"",
        "data-blocks-pricing-usage-slider-slider=\"\"",
        "data-blocks-pricing-usage-slider-stat=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "pricing-usage-slider page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-pricing-usage-slider-layout]",
        "[data-blocks-pricing-usage-slider-slider]",
        "[data-blocks-pricing-usage-slider-stat]",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// `testimonials-stack` のレイアウト CSS が [`fandhe_frontend_pre_styled_ui::
/// recipe::STAGGER_INDEX_VAR`] を実際に参照していることを固定する
/// （イシュー #2548。当初はリテラル複製 + ソーステキスト突合の契約
/// だったが、`pre-styled-ui` の `motion` feature が本クレートの
/// `Cargo.toml` で既に有効なため、`crates/docs-site/src/blocks/
/// testimonials_stack.rs` 側で定数を直接 import する設計へ是正した
/// codex-review 指摘。値のドリフトはコンパイラが型レベルで防ぐため、
/// 本テストはその参照が実際に生成 CSS へ反映されることのみを検証する）。
#[test]
fn testimonials_stack_stagger_var_matches_pre_styled_ui_recipe_source() {
    use fandhe_frontend_pre_styled_ui::recipe::STAGGER_INDEX_VAR;

    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    assert!(
        sheet_css.contains(STAGGER_INDEX_VAR),
        "blocks.css should reference {STAGGER_INDEX_VAR}"
    );
}

/// caption（出典欄）の `display: flex` がレシピの base slot セレクタ
/// （`[data-scope="blockquote"][data-part="caption"]`、詳細度 (0,2,0)）に
/// 詳細度で負けていないことを固定する（codex-review P2 / Cursor Bugbot
/// 指摘の是正、イシュー #2548）。`.blocks-testimonials-stack-meta` 単独
/// クラス（詳細度 (0,1,0)）では常に負けるため、生成 CSS 側が属性セレクタ
/// を含む詳細度 (0,3,0) 以上のセレクタで宣言していることを検証する。
#[test]
fn testimonials_stack_caption_meta_selector_outweighs_recipe_base() {
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    assert!(
        sheet_css.contains(
            r#"[data-scope="blockquote"][data-part="caption"].blocks-testimonials-stack-meta {"#
        ),
        "blocks.css should declare display:flex for the caption meta row with a \
         selector at least as specific as [data-scope][data-part] (0,2,0), otherwise \
         the recipe base rule wins and the avatar/byline row does not lay out inline"
    );
}

/// testimonials-stack のトランジションがトークン参照（`var(--fandhe-motion-
/// duration-*)`）のみで構成され、固定 ms 値の `@keyframes`/`animation:` を
/// 使わないことを固定する（`prefers-reduced-motion: reduce` 下の縮退が
/// `Theme::to_css` のトークン 0ms 化に自動追従する根拠、イシュー #2548）。
#[test]
fn testimonials_stack_uses_motion_tokens_not_raw_durations() {
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    assert!(
        sheet_css.contains("var(--fandhe-motion-duration-"),
        "blocks.css should reference a --fandhe-motion-duration-* token for testimonials-stack transitions"
    );
    assert!(
        sheet_css.contains("var(--fandhe-motion-easing-standard)"),
        "blocks.css should reference the --fandhe-motion-easing-standard token"
    );
    // イシュー #2549 で bento-staggered/feature-expand が
    // `fandhe_frontend_pre_styled_ui::motion::KEYFRAMES_CSS` を合成
    // stylesheet へ導入したため、結合済み `blocks.css` 全体に `@keyframes`
    // が存在しないことは主張できなくなった。testimonials-stack 自身が
    // 追加の `@keyframes` を持ち込んでいないことのみを、結合済み CSS 中の
    // `@keyframes` 出現数が共有 `KEYFRAMES_CSS` 由来の出現数と一致するかで
    // 固定する。イシュー #2546 で `text-split-reveal`（`text_reveal::
    // TEXT_REVEAL_CSS` の `fd-text-reveal-in`）と `hero-parallax-layers`
    // （`SlotRecipe::parallax` の `fandhe-motion-parallax`）が、
    // `KEYFRAMES_CSS` 由来ではない自前の `@keyframes` を新たに 2 件
    // 導入したため、期待値へその 2 件分を明示的に加算する
    // （testimonials-stack 自身が新規 `@keyframes` を持ち込んでいないこと
    // を固定する不変条件は変えない）。
    const NEW_KEYFRAMES_OUTSIDE_SHARED_MOTION_CSS: usize = 2;
    let keyframes_in_sheet = sheet_css.matches("@keyframes").count();
    let keyframes_in_shared_motion_css = fandhe_frontend_pre_styled_ui::motion::KEYFRAMES_CSS
        .matches("@keyframes")
        .count();
    assert_eq!(
        keyframes_in_sheet,
        keyframes_in_shared_motion_css + NEW_KEYFRAMES_OUTSIDE_SHARED_MOTION_CSS,
        "testimonials-stack should not introduce @keyframes beyond the shared \
         motion::KEYFRAMES_CSS import and the known text-split-reveal/\
         hero-parallax-layers additions (no reduced-motion @media override needed)"
    );
}

/// pricing-usage-slider の合成部品（slider/stat）が期待どおりの構成で
/// 実際に出力されていること、価格表示がスライダーの固定初期値
/// （50 千件 → $29）と一致していること、`<form>`/`data:` URI を持ち込んで
/// いないことを固定する（イシュー #2547）。
#[test]
fn pricing_usage_slider_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/pricing-usage-slider/index.html"))
        .expect("blocks/pricing-usage-slider/index.html should be generated");
    for needle in [
        "data-scope=\"slider\" data-part=\"root\"",
        "data-scope=\"slider\" data-part=\"marker-group\"",
        "data-scope=\"stat\" data-part=\"root\"",
        "$29",
        "想定コスト",
    ] {
        assert!(
            html.contains(needle),
            "pricing-usage-slider page should contain {needle}"
        );
    }
    for absent in ["<form", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "pricing-usage-slider should never contain {absent}"
        );
    }
}

/// pricing-usage-slider の thumb（`role="slider"`）に、表示ラベル
/// （`slider::label` の `span`）への `aria-labelledby` 関連付けが実際に
/// 出力されていること、参照先の `id` が同ページ内に存在することを固定
/// する（codex-review P1 指摘の回帰防止、イシュー #2547）。
/// `aria-valuetext`（「50 千件」）は値の説明であり操作部の名前の代わりに
/// ならないため、名前付けは `aria-labelledby` 側で担保する。
#[test]
fn pricing_usage_slider_thumb_is_labelled_by_visible_label() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/pricing-usage-slider/index.html"))
        .expect("blocks/pricing-usage-slider/index.html should be generated");
    assert!(
        html.contains("aria-labelledby=\"blocks-pricing-usage-slider-label\""),
        "pricing-usage-slider thumb should reference the visible label via aria-labelledby"
    );
    assert!(
        html.contains("id=\"blocks-pricing-usage-slider-label\""),
        "pricing-usage-slider label span should carry the id referenced by aria-labelledby \
         (no dangling IDREF)"
    );
}

/// bento-staggered ページが `blocks-demo blocks-bento-staggered` class・
/// 両 stylesheet の `<link>`・各セルの `data-blocks-bento-staggered-item`
/// CSS フックを実際に出力し、`blocks::stylesheet()` にも対応するセレクタが
/// 存在すること（イシュー #2549）。
#[test]
fn bento_staggered_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/bento-staggered/index.html"))
        .expect("blocks/bento-staggered/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-bento-staggered\""),
        "bento-staggered page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "bento-staggered page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "bento-staggered page should link the Blocks-specific stylesheet"
    );
    assert!(
        html.contains("data-blocks-bento-staggered-item=\"\""),
        "bento-staggered page should output the scroll-driven CSS hook attribute"
    );
    assert!(
        html.contains("data-blocks-bento-staggered-hero=\"\""),
        "bento-staggered page should mark exactly the hero cell"
    );
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for needle in [
        "[data-blocks-bento-staggered-item]",
        "[data-blocks-bento-staggered-hero]",
        "animation-timeline: view()",
        "@media (prefers-reduced-motion: reduce)",
    ] {
        assert!(
            sheet_css.contains(needle),
            "blocks.css should declare {needle} for bento-staggered"
        );
    }
}

/// bento-staggered の合成部品（card/icon）が期待どおりの構成で 6 枚出力
/// されていること、`--fandhe-motion-stagger-index` が 0〜5 の連番で各セル
/// へ書き出されていること、`<form>`/`data:` URI を持ち込んでいないことを
/// 固定する（イシュー #2549）。
#[test]
fn bento_staggered_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/bento-staggered/index.html"))
        .expect("blocks/bento-staggered/index.html should be generated");
    assert_eq!(
        html.matches("data-scope=\"card\" data-part=\"root\"")
            .count(),
        6,
        "bento-staggered should render exactly 6 cards"
    );
    for i in 0..6 {
        let needle = format!("--fandhe-motion-stagger-index: {i}");
        assert!(
            html.contains(&needle),
            "bento-staggered should output {needle}"
        );
    }
    for needle in [
        "Realtime Sync",
        "Global CDN",
        "data-scope=\"icon\" data-part=\"root\"",
    ] {
        assert!(
            html.contains(needle),
            "bento-staggered page should contain {needle}"
        );
    }
    for absent in ["<form", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "bento-staggered should never contain {absent}"
        );
    }
}

/// feature-expand ページが `blocks-demo blocks-feature-expand` class・
/// 両 stylesheet の `<link>`・各カードの `data-blocks-feature-expand-item`
/// CSS フックを実際に出力し、`blocks::stylesheet()` にも hover/
/// focus-within の展開規則が存在すること（イシュー #2549）。
#[test]
fn feature_expand_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/feature-expand/index.html"))
        .expect("blocks/feature-expand/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-feature-expand\""),
        "feature-expand page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "feature-expand page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "feature-expand page should link the Blocks-specific stylesheet"
    );
    assert!(
        html.contains("data-blocks-feature-expand-item=\"\""),
        "feature-expand page should output the hover-expand CSS hook attribute"
    );
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for needle in [
        "[data-blocks-feature-expand-item]:hover [data-blocks-feature-expand-wrap]",
        "[data-blocks-feature-expand-item]:focus-within [data-blocks-feature-expand-wrap]",
        "grid-template-rows: 0fr",
    ] {
        assert!(
            sheet_css.contains(needle),
            "blocks.css should declare {needle} for feature-expand"
        );
    }
}

/// feature-expand の合成部品（card/icon/button）が期待どおりの構成で
/// 6 枚出力されていること、各カードの詳細説明が hover 前提の非表示処理
/// （`hidden`/`aria-hidden`）を持たず常時 DOM 上に存在すること、
/// `<form>`/`data:` URI を持ち込んでいないことを固定する（イシュー #2549）。
#[test]
fn feature_expand_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/feature-expand/index.html"))
        .expect("blocks/feature-expand/index.html should be generated");
    assert_eq!(
        html.matches("data-scope=\"card\" data-part=\"root\"")
            .count(),
        6,
        "feature-expand should render exactly 6 cards"
    );
    for needle in [
        "Instant Search",
        "Audit Trail",
        "Learn more",
        "data-scope=\"icon\" data-part=\"root\"",
        "data-scope=\"button\" data-part=\"root\"",
    ] {
        assert!(
            html.contains(needle),
            "feature-expand page should contain {needle}"
        );
    }
    for absent in [
        "<form",
        "src=\"data:",
        "aria-hidden=\"true\" data-blocks-feature-expand",
    ] {
        assert!(
            !html.contains(absent),
            "feature-expand should never contain {absent}"
        );
    }
}

/// cta-banner-magnetic ページの Demo クラス・両スタイルシート・
/// `data-blocks-cta-banner-magnetic-*` CSS フックが実際に出力され、
/// `blocks::stylesheet()` にも対応するセレクタが存在することを固定する
/// （イシュー #2550）。
#[test]
fn cta_banner_magnetic_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/cta-banner-magnetic/index.html"))
        .expect("blocks/cta-banner-magnetic/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-cta-banner-magnetic\""),
        "cta-banner-magnetic page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "cta-banner-magnetic page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "cta-banner-magnetic page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-cta-banner-magnetic-banner=\"\"",
        "data-blocks-cta-banner-magnetic-title=\"\"",
        "data-blocks-cta-banner-magnetic-cta=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "cta-banner-magnetic page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-cta-banner-magnetic-banner]",
        "[data-blocks-cta-banner-magnetic-cta]",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// cta-banner-magnetic の合成部品（`button`）が期待どおりの構成で実際に
/// 出力されていること、opt-in マーカー `data-fandhe-magnetic` が CTA
/// ボタンへ実際に付与されていること、CSS 側が
/// `--fandhe-motion-magnetic-x`/`-y` を消費する `transform` 規則を持つこと、
/// `<form>` を持ち込んでいないことを固定する（イシュー #2550）。
#[test]
fn cta_banner_magnetic_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/cta-banner-magnetic/index.html"))
        .expect("blocks/cta-banner-magnetic/index.html should be generated");
    for needle in [
        "data-fandhe-magnetic=\"\"",
        "Ready to get started?",
        "Get started",
    ] {
        assert!(
            html.contains(needle),
            "cta-banner-magnetic page should contain {needle}"
        );
    }
    for absent in ["<form", "href=\"#\"", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "cta-banner-magnetic should never contain {absent}"
        );
    }

    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    assert!(
        sheet_css.contains("--fandhe-motion-magnetic-x")
            && sheet_css.contains("--fandhe-motion-magnetic-y"),
        "blocks.css should consume the magnetic offset custom properties in a transform rule"
    );
}

/// cta-signup-celebrate ページの Demo クラス・両スタイルシート・
/// `data-blocks-cta-signup-celebrate-*` CSS フックが実際に出力され、
/// `blocks::stylesheet()` にも対応するセレクタが存在することを固定する
/// （イシュー #2550）。
#[test]
fn cta_signup_celebrate_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/cta-signup-celebrate/index.html"))
        .expect("blocks/cta-signup-celebrate/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-cta-signup-celebrate\""),
        "cta-signup-celebrate page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "cta-signup-celebrate page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "cta-signup-celebrate page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-cta-signup-celebrate-stack=\"\"",
        "data-blocks-cta-signup-celebrate-card=\"\"",
        "data-blocks-cta-signup-celebrate-field=\"\"",
        "data-blocks-cta-signup-celebrate-submit=\"\"",
        "data-blocks-cta-signup-celebrate-canvas=\"\"",
        "data-blocks-cta-signup-celebrate-celebrate=\"\"",
        "data-blocks-cta-signup-celebrate-caption=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "cta-signup-celebrate page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-cta-signup-celebrate-stack]",
        "[data-blocks-cta-signup-celebrate-caption]",
        "[data-blocks-cta-signup-celebrate-field]",
        "[data-blocks-cta-signup-celebrate-submit]",
        "[data-blocks-cta-signup-celebrate-canvas]",
        "[data-blocks-cta-signup-celebrate-celebrate]",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// cta-signup-celebrate の合成部品（`card`/`field`/`input`/`button`）が
/// 期待どおりの構成で実際に出力されていること、confetti opt-in 属性
/// （`data-fandhe-confetti-trigger`/`data-fandhe-confetti-canvas`）の値と
/// `id` の対応が実際に一致していること、送信前/送信完了の両カードが
/// 可視状態（`hidden` なし）で出力されていること、`<form>`/実データを
/// 持ち込んでいないことを固定する（イシュー #2550）。
#[test]
fn cta_signup_celebrate_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/cta-signup-celebrate/index.html"))
        .expect("blocks/cta-signup-celebrate/index.html should be generated");
    for needle in [
        "type=\"email\"",
        "placeholder=\"m@example.com\"",
        "Notify me",
        "You&#x27;re all set!",
        "Before submission",
        "After submission (celebrate)",
        "data-fandhe-confetti-trigger=\"cta-signup-celebrate-canvas\"",
        "id=\"cta-signup-celebrate-canvas\"",
        "data-fandhe-confetti-canvas=\"\"",
    ] {
        assert!(
            html.contains(needle),
            "cta-signup-celebrate page should contain {needle}"
        );
    }
    for absent in ["<form", "href=\"#\"", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "cta-signup-celebrate should never contain {absent}"
        );
    }

    // 「送信前」「送信完了」の両カードが `hidden` なしで実際に可視状態
    // であること（`sidebar_07`/`pricing_tiers_morph` と同型の 2 状態併記
    // 回帰固定。無 JS のため両方を静的に見せる設計の検証）。
    let before_card = extract_tag_by_id(&html, "blocks-cta-signup-celebrate-email-control");
    assert!(
        !before_card.contains("hidden"),
        "the before-submit card's email input should be visible (not hidden)"
    );

    // confetti canvas 側マーカーは既存の `wasm-full/src/confetti.rs`
    // ロケータ契約（`root` 包含 + `data-fandhe-confetti-canvas` 必須）を
    // そのまま満たす想定であることの回帰固定として、トリガーの値と
    // canvas の `id` が一致することを再確認する。
    assert!(
        html.contains(r#"data-fandhe-confetti-trigger="cta-signup-celebrate-canvas""#)
            && html.contains(r#"id="cta-signup-celebrate-canvas""#),
        "confetti trigger value must match the canvas id for the locator contract to resolve"
    );
}

/// footer-sticky-reveal ページの Demo クラス・両スタイルシート・
/// `data-blocks-footer-sticky-reveal-*` CSS フックが実際に出力され、
/// `blocks::stylesheet()` にも sticky/scroll-driven の宣言が存在することを
/// 固定する（イシュー #2551）。
#[test]
fn footer_sticky_reveal_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/footer-sticky-reveal/index.html"))
        .expect("blocks/footer-sticky-reveal/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-footer-sticky-reveal\""),
        "footer-sticky-reveal page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "footer-sticky-reveal page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "footer-sticky-reveal page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-footer-sticky-reveal-content=\"\"",
        "data-blocks-footer-sticky-reveal-footer=\"\"",
        "data-blocks-footer-sticky-reveal-footer-inner=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "footer-sticky-reveal page should output the {hook} CSS hook attribute"
        );
    }

    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for needle in [
        "[data-blocks-footer-sticky-reveal-content]",
        "position: sticky",
        "bottom: 0",
        "animation-timeline: scroll(nearest)",
        "@media (prefers-reduced-motion: reduce)",
    ] {
        assert!(
            sheet_css.contains(needle),
            "blocks.css should contain {needle} for footer-sticky-reveal"
        );
    }

    // `@supports` ブロックは末尾の `@media (prefers-reduced-motion: reduce)`
    // より前に出現すること（記述順後勝ちの固定、モジュール doc「追加の
    // scroll-driven 強調」節参照）。
    let supports_pos = sheet_css
        .find("@supports (animation-timeline: scroll())")
        .expect("blocks.css should contain the @supports block for footer-sticky-reveal");
    let reduce_pos = sheet_css
        .rfind("@media (prefers-reduced-motion: reduce)")
        .expect("blocks.css should contain a prefers-reduced-motion block");
    assert!(
        supports_pos < reduce_pos,
        "@supports block must appear before the trailing @media reduce block (last-wins order)"
    );
}

/// footer-sticky-reveal の合成部品（`card`/`heading`/`link`/`nav_list`）が
/// 期待どおり出力され、`<form>`/`href="#"`/`src="data:` を持ち込んでいない
/// ことを固定する（イシュー #2551）。
#[test]
fn footer_sticky_reveal_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/footer-sticky-reveal/index.html"))
        .expect("blocks/footer-sticky-reveal/index.html should be generated");
    assert!(
        html.contains("<footer"),
        "Demo should render a <footer> element"
    );
    assert!(
        html.contains(r#"data-scope="link" data-part="root""#),
        "Demo should compose the styled link part"
    );
    assert!(
        html.contains(r#"data-scope="nav-list" data-part="root""#),
        "Demo should compose the styled nav_list root part"
    );
    let card_count = html.matches(r#"data-scope="card""#).count();
    assert!(
        card_count >= 3,
        "Demo should compose at least 3 dummy content cards, got {card_count}"
    );
    for absent in ["<form", "href=\"#\"", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "footer-sticky-reveal should never contain {absent}"
        );
    }
}

/// footer-newsletter ページの Demo クラス・両スタイルシート・
/// `data-blocks-footer-newsletter-*` CSS フックが実際に出力され、
/// `blocks::stylesheet()` にも presence 同型宣言が存在することを固定する
/// （イシュー #2551）。
#[test]
fn footer_newsletter_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/footer-newsletter/index.html"))
        .expect("blocks/footer-newsletter/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-footer-newsletter\""),
        "footer-newsletter page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "footer-newsletter page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "footer-newsletter page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-footer-newsletter-root=\"\"",
        "data-blocks-footer-newsletter-panel=\"\"",
        "data-blocks-footer-newsletter-field=\"\"",
        "data-blocks-footer-newsletter-submit=\"\"",
        "data-blocks-footer-newsletter-caption=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "footer-newsletter page should output the {hook} CSS hook attribute"
        );
    }

    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector_or_decl in [
        "[data-blocks-footer-newsletter-panel]",
        "transition-behavior: allow-discrete",
        "@starting-style",
        "var(--fandhe-motion-duration-normal)",
    ] {
        assert!(
            sheet_css.contains(selector_or_decl),
            "blocks.css should contain {selector_or_decl} for footer-newsletter"
        );
    }
}

/// footer-newsletter の合成部品（`field`/`input`/`button`/`link`）が期待
/// どおり出力され、「入力」「完了」の 2 インスタンス併記で各 4 個の panel
/// のうち 2 個だけが `hidden` であること、`<form>`/`href="#"` を持ち込んで
/// いないことを固定する（イシュー #2551）。
#[test]
fn footer_newsletter_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/footer-newsletter/index.html"))
        .expect("blocks/footer-newsletter/index.html should be generated");
    for needle in [
        "type=\"email\"",
        "placeholder=\"m@example.com\"",
        "Subscribe",
        "Subscribed! Thanks for joining.",
        "Before subscribe",
        "After subscribe",
    ] {
        assert!(
            html.contains(needle),
            "footer-newsletter page should contain {needle}"
        );
    }
    for absent in ["<form", "href=\"#\"", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "footer-newsletter should never contain {absent}"
        );
    }

    // 2 インスタンス（Before/After）× 2 panel（入力/完了）= 4 個の panel の
    // うち、各インスタンスで片方だけが `hidden` になること（無 JS での
    // 2 状態併記の可視性回帰固定、`pricing_tiers_morph`/
    // `cta_signup_celebrate` と同型）。
    let panel_open_count = html
        .matches("data-blocks-footer-newsletter-panel=\"\"")
        .count();
    let hidden_panel_count = html
        .matches("data-blocks-footer-newsletter-panel=\"\" hidden=\"\"")
        .count();
    assert_eq!(
        panel_open_count, 4,
        "expected 4 newsletter panels (2 instances x 2 panels), got {panel_open_count}"
    );
    assert_eq!(
        hidden_panel_count, 2,
        "expected exactly 2 hidden panels (1 per instance), got {hidden_panel_count}"
    );
}

/// hero-editorial-stagger ページが `blocks-demo blocks-hero-editorial-
/// stagger` class・両 stylesheet の `<link>`・stagger 用 CSS フックを
/// 実際に出力し、`blocks::stylesheet()` にも対応するセレクタが存在する
/// こと（イシュー #2546）。
#[test]
fn hero_editorial_stagger_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/hero-editorial-stagger/index.html"))
        .expect("blocks/hero-editorial-stagger/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-hero-editorial-stagger\""),
        "hero-editorial-stagger page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "hero-editorial-stagger page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "hero-editorial-stagger page should link the Blocks-specific stylesheet"
    );
    assert_eq!(
        html.matches("data-blocks-hero-editorial-stagger-item=\"\"")
            .count(),
        4,
        "hero-editorial-stagger should mark exactly 4 stagger items"
    );
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for needle in [
        "[data-blocks-hero-editorial-stagger-item]",
        "animation-delay: calc(var(--fandhe-motion-stagger-index",
        "@media (prefers-reduced-motion: reduce)",
    ] {
        assert!(
            sheet_css.contains(needle),
            "blocks.css should declare {needle} for hero-editorial-stagger"
        );
    }
}

/// hero-editorial-stagger の合成部品（badge/heading/text/button）が期待
/// どおりの構成で出力され、stagger index が 0〜3 の連番で各要素へ書き出
/// されていること、`<form>`/`data:` URI を持ち込んでいないことを固定
/// する（イシュー #2546）。
#[test]
fn hero_editorial_stagger_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/hero-editorial-stagger/index.html"))
        .expect("blocks/hero-editorial-stagger/index.html should be generated");
    for i in 0..4 {
        let needle = format!("--fandhe-motion-stagger-index: {i}");
        assert!(
            html.contains(&needle),
            "hero-editorial-stagger should output {needle}"
        );
    }
    for needle in [
        "data-scope=\"badge\"",
        "data-scope=\"heading\"",
        "data-scope=\"text\"",
        "Get started",
        "View docs",
    ] {
        assert!(
            html.contains(needle),
            "hero-editorial-stagger page should contain {needle}"
        );
    }
    for absent in ["<form", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "hero-editorial-stagger should never contain {absent}"
        );
    }
}

/// hero-parallax-layers ページが `blocks-demo blocks-hero-parallax-
/// layers` class・両 stylesheet の `<link>`・3 レイヤーの `data-scope`/
/// `data-part` を実際に出力し、`blocks::stylesheet()` にも
/// `SlotRecipe::parallax` のプログレッシブエンハンスメント契約（
/// ネイティブ/reduced-motion の 2 ブロック）が存在すること（イシュー
/// #2546）。
#[test]
fn hero_parallax_layers_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/hero-parallax-layers/index.html"))
        .expect("blocks/hero-parallax-layers/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-hero-parallax-layers\""),
        "hero-parallax-layers page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "hero-parallax-layers page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "hero-parallax-layers page should link the Blocks-specific stylesheet"
    );
    for part in ["layer-back", "layer-mid", "layer-front", "content"] {
        let needle = format!("data-scope=\"blocks-hero-parallax\" data-part=\"{part}\"");
        assert!(
            html.contains(&needle),
            "hero-parallax-layers page should output {needle}"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for needle in [
        "@supports (animation-timeline: view())",
        "@media (prefers-reduced-motion: reduce)",
        "--fandhe-motion-parallax-distance",
    ] {
        assert!(
            sheet_css.contains(needle),
            "blocks.css should declare {needle} for hero-parallax-layers"
        );
    }
}

/// hero-parallax-layers の合成部品（heading/text/button）が期待どおりの
/// 構成で出力されていること、`<form>`/`data:` URI・`data-fandhe-scroll-
/// progress` 属性を持ち込んでいないこと（モジュール doc「`data-fandhe-
/// scroll-progress` を付与しない理由」節）を固定する（イシュー #2546）。
#[test]
fn hero_parallax_layers_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/hero-parallax-layers/index.html"))
        .expect("blocks/hero-parallax-layers/index.html should be generated");
    for needle in [
        "data-scope=\"heading\"",
        "data-scope=\"text\"",
        "A workspace that moves with you",
        "Explore",
    ] {
        assert!(
            html.contains(needle),
            "hero-parallax-layers page should contain {needle}"
        );
    }
    // `data-fandhe-scroll-progress=` (属性としての出現、`=` 込み) のみを
    // 禁止する。導入文の説明文（`<code>data-fandhe-scroll-progress</code>`)
    // は属性ではなくプレーンテキストとして正当に許容される（`=` を持たない）。
    for absent in ["<form", "src=\"data:", "data-fandhe-scroll-progress="] {
        assert!(
            !html.contains(absent),
            "hero-parallax-layers should never contain {absent}"
        );
    }
}

/// hero-terminal ページが `blocks-demo blocks-hero-terminal` class・両
/// stylesheet の `<link>`・行ごとの stagger CSS フックを実際に出力し、
/// `blocks::stylesheet()` にも対応するセレクタが存在すること（イシュー
/// #2546）。
#[test]
fn hero_terminal_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/hero-terminal/index.html"))
        .expect("blocks/hero-terminal/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-hero-terminal\""),
        "hero-terminal page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "hero-terminal page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "hero-terminal page should link the Blocks-specific stylesheet"
    );
    assert_eq!(
        html.matches("data-blocks-hero-terminal-line=\"\"").count(),
        4,
        "hero-terminal should mark exactly 4 lines (3 commands + typewriter)"
    );
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for needle in [
        "[data-blocks-hero-terminal-line]",
        "animation-delay: calc(var(--fandhe-motion-stagger-index",
        "@media (prefers-reduced-motion: reduce)",
    ] {
        assert!(
            sheet_css.contains(needle),
            "blocks.css should declare {needle} for hero-terminal"
        );
    }
}

/// hero-terminal の合成部品（code/kbd）・typewriter opt-in マーカーが
/// 期待どおりに 1 回だけ出力されていること、`<form>`/`data:` URI を持ち
/// 込んでいないことを固定する（イシュー #2546）。
#[test]
fn hero_terminal_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/hero-terminal/index.html"))
        .expect("blocks/hero-terminal/index.html should be generated");
    assert_eq!(
        html.matches("data-scope=\"code\"").count(),
        3,
        "hero-terminal should render exactly 3 code lines"
    );
    assert_eq!(
        html.matches("data-fandhe-typewriter").count(),
        1,
        "hero-terminal should carry exactly one typewriter opt-in marker"
    );
    for needle in [
        "fw new my-app",
        "cd my-app",
        "cargo run",
        "data-scope=\"kbd\"",
    ] {
        assert!(
            html.contains(needle),
            "hero-terminal page should contain {needle}"
        );
    }
    for absent in ["<form", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "hero-terminal should never contain {absent}"
        );
    }
}

/// text-split-reveal ページが `blocks-demo blocks-text-split-reveal`
/// class・両 stylesheet の `<link>`・`text_reveal::TEXT_REVEAL_CSS` の
/// `@keyframes`/reduced-motion 縮退を実際に出力していることを固定する
/// （イシュー #2546）。
#[test]
fn text_split_reveal_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/text-split-reveal/index.html"))
        .expect("blocks/text-split-reveal/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-text-split-reveal\""),
        "text-split-reveal page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "text-split-reveal page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "text-split-reveal page should link the Blocks-specific stylesheet"
    );
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for needle in [
        ".fd-text-reveal__unit",
        "@media (prefers-reduced-motion: reduce)",
    ] {
        assert!(
            sheet_css.contains(needle),
            "blocks.css should declare {needle} for text-split-reveal"
        );
    }
}

/// text-split-reveal の合成部品（heading/text/button）が
/// `text_reveal::chars`/`words` をそれぞれ 1 回ずつ使い、SR 用の分割前
/// テキストレイヤーと `aria-hidden` の分割済み表示レイヤーの両方を持つ
/// こと、`<form>`/`data:` URI を持ち込んでいないことを固定する（イシュー
/// #2546）。
#[test]
fn text_split_reveal_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/text-split-reveal/index.html"))
        .expect("blocks/text-split-reveal/index.html should be generated");
    assert_eq!(
        html.matches("data-fandhe-text-reveal=\"chars\"").count(),
        1,
        "text-split-reveal should use chars() exactly once"
    );
    assert_eq!(
        html.matches("data-fandhe-text-reveal=\"words\"").count(),
        1,
        "text-split-reveal should use words() exactly once"
    );
    assert_eq!(
        html.matches("fd-text-reveal__sr").count(),
        2,
        "text-split-reveal should carry the SR layer for both reveals"
    );
    for needle in ["Built for clarity", "Try it out", "data-scope=\"heading\""] {
        assert!(
            html.contains(needle),
            "text-split-reveal page should contain {needle}"
        );
    }
    for absent in ["<form", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "text-split-reveal should never contain {absent}"
        );
    }
}

/// game-ui-modal ページの Demo クラス・両スタイルシート・
/// `data-blocks-game-ui-modal-*` CSS フックが実際に出力され、
/// `blocks::stylesheet()` にも対応するセレクタ・keyframes 名・spring
/// イージング・stagger 変数が存在することを固定する（イシュー #2552）。
#[test]
fn game_ui_modal_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/game-ui-modal/index.html"))
        .expect("blocks/game-ui-modal/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-game-ui-modal\""),
        "game-ui-modal page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "game-ui-modal page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "game-ui-modal page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-game-ui-modal-root=\"\"",
        "data-blocks-game-ui-modal-content=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "game-ui-modal page should output the {hook} CSS hook attribute"
        );
    }
    assert_eq!(
        html.matches("data-blocks-game-ui-modal-reward=\"\"")
            .count(),
        3,
        "game-ui-modal should render exactly 3 reward rows"
    );
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for needle in [
        ".blocks-game-ui-modal [data-scope=\"dialog\"][data-part=\"backdrop\"]",
        ".blocks-game-ui-modal [data-scope=\"dialog\"][data-part=\"positioner\"]",
        "[data-blocks-game-ui-modal-reward]",
        fandhe_frontend_pre_styled_ui::motion::ZOOM_IN_KEYFRAMES_NAME,
        fandhe_frontend_pre_styled_ui::motion::SLIDE_FROM_BOTTOM_KEYFRAMES_NAME,
        fandhe_frontend_pre_styled_ui::theme::SPRING_EASING_LINEAR,
        fandhe_frontend_pre_styled_ui::recipe::STAGGER_INDEX_VAR,
    ] {
        assert!(
            sheet_css.contains(needle),
            "blocks.css should declare/reference {needle} for game-ui-modal"
        );
    }
}

/// game-ui-modal の合成部品（dialog/badge/button）が期待どおりの構成で
/// 実際に出力されていること（backdrop/positioner/content/title/footer の
/// `data-scope="dialog"`・報酬 badge・ボタン 2 個・`aria-labelledby` の id
/// 対応）、`<form>`・`data-part="trigger"`・`<script` を持ち込んでいないこと
/// を固定する（イシュー #2552）。
#[test]
fn game_ui_modal_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/game-ui-modal/index.html"))
        .expect("blocks/game-ui-modal/index.html should be generated");
    for needle in [
        "data-scope=\"dialog\" data-part=\"backdrop\"",
        "data-scope=\"dialog\" data-part=\"positioner\"",
        "data-scope=\"dialog\" data-part=\"content\"",
        "data-scope=\"dialog\" data-part=\"title\"",
        "data-scope=\"dialog\" data-part=\"description\"",
        "data-scope=\"dialog\" data-part=\"body\"",
        "data-scope=\"dialog\" data-part=\"footer\"",
        "data-scope=\"badge\"",
        "data-state=\"open\"",
    ] {
        assert!(
            html.contains(needle),
            "game-ui-modal page should contain {needle}"
        );
    }
    assert_eq!(
        html.matches("data-scope=\"button\"").count(),
        2,
        "game-ui-modal should render exactly 2 buttons (Later / Claim rewards)"
    );
    assert!(
        html.contains(r#"id="blocks-game-ui-modal-title""#)
            && html.contains(r#"aria-labelledby="blocks-game-ui-modal-title""#),
        "game-ui-modal content should be labelled by the title id"
    );
    // "<script" はページ全体に site.js 読み込みタグが常に存在するため
    // ここでは検査しない（demo() 出力自体の検査は
    // `demo_output_never_leaks_an_unescaped_script_tag` が全 block 横断で
    // 既に担う）。
    for absent in ["<form", "data-part=\"trigger\"", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "game-ui-modal should never contain {absent}"
        );
    }
}

/// game-ui-modal の CSS が生の `ms` 数値・無限反復（`infinite`）・
/// scroll-driven（`animation-timeline`）を持ち込まず、`animation-duration`/
/// `-delay` が `--fandhe-motion-duration-*` トークン参照のみで組み立てられ
/// ていることを固定する（`testimonials_stack_uses_motion_tokens_not_raw_durations`
/// と同型、イシュー #2552）。
///
/// 検証範囲は `game_ui_modal::layout_css()` に相当する連結済み CSS 中の
/// 範囲（`.blocks-game-ui-modal.blocks-demo` セレクタから次 block
/// （`game-ui-modal` は Blocks レジストリ末尾に登録済みのため CSS 末尾）
/// までの部分文字列）に限定する。`bento-staggered` が
/// `animation-timeline: view()` を、`motion::KEYFRAMES_CSS` 側が
/// `infinite`（bounce/shake keyframes 向け rustdoc 例示、CSS 本体には
/// 出現しない）をそれぞれ別ブロックで正当に使うため、結合済み
/// `blocks.css` 全体に対する `!contains` はそれらを誤検知する。
#[test]
fn game_ui_modal_css_uses_motion_tokens_and_no_infinite_keyframes() {
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    let start = sheet_css
        .find(".blocks-game-ui-modal.blocks-demo")
        .expect("blocks.css should declare the game-ui-modal layout block");
    let game_ui_modal_css = &sheet_css[start..];
    assert!(
        !game_ui_modal_css.contains("infinite"),
        "game-ui-modal should not declare an infinitely repeating animation"
    );
    assert!(
        !game_ui_modal_css.contains("animation-timeline"),
        "game-ui-modal should not use scroll-driven animation-timeline"
    );
    assert!(
        game_ui_modal_css.contains("var(--fandhe-motion-duration-"),
        "game-ui-modal CSS should reference a --fandhe-motion-duration-* token"
    );
}

/// `banner-floating-card` の実ビルド出力が Demo class・CSS 配線
/// （`pre-styled-ui.css`/`blocks.css`）・CSS フック属性・生成 CSS 中の
/// 対応セレクタを持つことを固定する（`login_04_page_wires_demo_class_and_css_hooks`
/// と同型、イシュー #2742）。
#[test]
fn banner_floating_card_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/banner-floating-card/index.html"))
        .expect("blocks/banner-floating-card/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-banner-floating-card\""),
        "banner-floating-card page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "banner-floating-card page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "banner-floating-card page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-banner-floating-card-card=\"\"",
        "data-blocks-banner-floating-card-close=\"\"",
        "data-blocks-banner-floating-card-link=\"\"",
        "data-blocks-banner-floating-card-placement=\"top\"",
        "data-blocks-banner-floating-card-placement=\"bottom\"",
        "data-blocks-banner-floating-card-align=\"center\"",
    ] {
        assert!(
            html.contains(hook),
            "banner-floating-card page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-scope=\"callout\"][data-part=\"root\"][data-blocks-banner-floating-card-card]",
        "[data-blocks-banner-floating-card-close]",
        "[data-blocks-banner-floating-card-align=\"center\"]",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// blog-featured-article ページの Demo クラス・両スタイルシート・
/// `data-blocks-blog-featured-article-*` CSS フックが実際に出力され、
/// `blocks::stylesheet()` にも対応するセレクタ・`@media` クエリ・
/// アクセントトークン参照が存在することを固定する（イシュー #2808）。
#[test]
fn blog_featured_article_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/blog-featured-article/index.html"))
        .expect("blocks/blog-featured-article/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-blog-featured-article\""),
        "blog-featured-article page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "blog-featured-article page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "blog-featured-article page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-blog-featured-article-feature=\"\"",
        "data-blocks-blog-featured-article-category=\"\"",
        "data-blocks-blog-featured-article-avatar=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "blog-featured-article page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for needle in [
        ".blocks-blog-featured-article-feature-layout",
        ".blocks-blog-featured-article-band",
        ".blocks-blog-featured-article-grid",
        "@media (max-width: 47.99rem)",
        "var(--fandhe-color-accent)",
    ] {
        assert!(
            sheet_css.contains(needle),
            "blocks.css should declare/reference {needle} for blog-featured-article"
        );
    }
}

/// `blog-featured-with-list`（イシュー #2809）の CSS フック配線検証。
/// `testimonials_stack_page_wires_demo_class_and_css_hooks` と同型
/// （`crate::blocks` モジュール doc「CSS フックが `class` と `[data-*]` で
/// 混在する理由」節参照）。
#[test]
fn blog_featured_with_list_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/blog-featured-with-list/index.html"))
        .expect("blocks/blog-featured-with-list/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-blog-featured-with-list\""),
        "blog-featured-with-list page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "blog-featured-with-list page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "blog-featured-with-list page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-blog-featured-with-list-article=\"\"",
        "data-blocks-blog-featured-with-list-author=\"\"",
        "data-blocks-blog-featured-with-list-separator=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "blog-featured-with-list page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-blog-featured-with-list-article]",
        "[data-blocks-blog-featured-with-list-author]",
        "[data-blocks-blog-featured-with-list-separator]",
        ".blocks-blog-featured-with-list-list",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// `banner-floating-card` の合成部品（callout/link/icon/button）が
/// 実ビルド HTML へ現れ、`<form>`・`href="#"`・`src="data:` を持ち込まない
/// ことを固定する（`game_ui_modal_composes_expected_parts` と同型、
/// イシュー #2742）。
#[test]
fn banner_floating_card_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/banner-floating-card/index.html"))
        .expect("blocks/banner-floating-card/index.html should be generated");
    for needle in [
        "data-scope=\"callout\" data-part=\"root\"",
        "data-scope=\"callout\" data-part=\"icon\"",
        "data-scope=\"callout\" data-part=\"text\"",
        "data-scope=\"link\" data-part=\"root\"",
        "data-scope=\"button\"",
    ] {
        assert!(
            html.contains(needle),
            "banner-floating-card page should contain {needle}"
        );
    }
    for absent in ["<form", "href=\"#\"", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "banner-floating-card should never contain {absent}"
        );
    }
}

/// blog-featured-article の合成部品（heading/text/badge/card/image/avatar/
/// link）が期待どおりの構成で実際に出力されていること（基本形 +
/// バリエーションの 2 インスタンス併記で特集記事カードが 2 件・下段
/// グリッドカードが 3 件出力される）、`<form>`・`href="#"`・`src="data:`
/// を持ち込んでいないことを固定する（イシュー #2808）。
#[test]
fn blog_featured_article_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/blog-featured-article/index.html"))
        .expect("blocks/blog-featured-article/index.html should be generated");
    for scope in [
        "data-scope=\"heading\"",
        "data-scope=\"text\"",
        "data-scope=\"badge\"",
        "data-scope=\"card\"",
        "data-scope=\"image\"",
        "data-scope=\"avatar\"",
        "data-scope=\"link\"",
    ] {
        assert!(
            html.contains(scope),
            "blog-featured-article page should contain {scope}"
        );
    }
    assert_eq!(
        html.matches("data-blocks-blog-featured-article-feature=\"\"")
            .count(),
        2,
        "blog-featured-article should render exactly 2 featured-article instances \
         (basic form + banded variant)"
    );
    assert!(
        html.contains("blocks-blog-featured-article-grid"),
        "blog-featured-article should render the banded variant's grid wrapper"
    );
    for absent in ["<form", "href=\"#\"", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "blog-featured-article should never contain {absent}"
        );
    }
}

/// `blog-featured-with-list` が使用部品どおりに合成され、非対話制約
/// （`<form>`/死リンク/`data:` URI 不在）を満たすことの回帰。
#[test]
fn blog_featured_with_list_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/blog-featured-with-list/index.html"))
        .expect("blocks/blog-featured-with-list/index.html should be generated");
    for needle in [
        "data-scope=\"heading\"",
        "data-scope=\"text\"",
        "data-scope=\"avatar\"",
        "data-scope=\"separator\"",
        "data-scope=\"link\"",
        "data-scope=\"link-overlay\"",
        "<time class=\"blocks-blog-featured-with-list-date\" datetime=",
    ] {
        assert!(
            html.contains(needle),
            "blog-featured-with-list page should contain {needle}"
        );
    }
    assert!(
        html.contains(r#"data-scope="link-overlay" data-part="overlay""#),
        "blog-featured-with-list should render the link-overlay overlay part"
    );
    // overlay の `<a>` が `aria-label` を持つこと（宙に浮いた記事タイトル
    // クリック領域に読み上げ用のラベルを与える）。
    assert!(
        html.contains(
            r#"data-part="overlay" href="https://github.com/Fandhe-AI/fandhe-frontend" aria-label=""#
        ),
        "blog-featured-with-list overlay links should carry an aria-label"
    );
    for absent in ["<form", "href=\"#\"", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "blog-featured-with-list should never contain {absent}"
        );
    }
}

/// `bento-asymmetric-rows`（イシュー #2745/#2746）の CSS フック配線検証。
/// `bento_staggered_page_wires_demo_class_and_css_hooks` と同型
/// （`crate::blocks` モジュール doc「CSS フックが `class` と `[data-*]` で
/// 混在する理由」節参照）。#2746 で 6 列/3 列の 2 列モード・`half` 幅区分
/// が追加されたため、対応する CSS フックの存在確認を追加した。
#[test]
fn bento_asymmetric_rows_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/bento-asymmetric-rows/index.html"))
        .expect("blocks/bento-asymmetric-rows/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-bento-asymmetric-rows\""),
        "bento-asymmetric-rows page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "bento-asymmetric-rows page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "bento-asymmetric-rows page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-bento-asymmetric-rows-cell=\"wide\"",
        "data-blocks-bento-asymmetric-rows-cell=\"narrow\"",
        "data-blocks-bento-asymmetric-rows-cell=\"half\"",
        "data-blocks-bento-asymmetric-rows-columns=\"three\"",
        "data-blocks-bento-asymmetric-rows-eyebrow=\"\"",
        "data-blocks-bento-asymmetric-rows-caption=\"\"",
        "data-blocks-bento-asymmetric-rows-feature=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "bento-asymmetric-rows page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for needle in [
        ".blocks-bento-asymmetric-rows-grid",
        "[data-blocks-bento-asymmetric-rows-cell=\"wide\"]",
        "@media (min-width: 48rem)",
        "@media (min-width: 64rem)",
        "grid-column: span 4",
        "grid-column: span 3",
        "repeat(3, minmax(0, 1fr))",
        "repeat(4, minmax(0, 1fr))",
    ] {
        assert!(
            sheet_css.contains(needle),
            "blocks.css should declare {needle} for bento-asymmetric-rows"
        );
    }
}

/// bento-asymmetric-rows の合成部品（badge/heading/text/card/image/icon）が
/// 期待どおりの構成で実際に出力されていること、4 バリエーション合計
/// 17 セル（イシュー #2746 で残りのバリエーションを追加）が出力される
/// こと、`<form>`/死リンク/`data:` URI を持ち込んでいないことを固定する
/// （イシュー #2745/#2746）。
#[test]
fn bento_asymmetric_rows_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/bento-asymmetric-rows/index.html"))
        .expect("blocks/bento-asymmetric-rows/index.html should be generated");
    for scope in [
        "data-scope=\"badge\"",
        "data-scope=\"heading\"",
        "data-scope=\"text\"",
        "data-scope=\"card\"",
        "data-scope=\"image\"",
        "data-scope=\"icon\"",
    ] {
        assert!(
            html.contains(scope),
            "bento-asymmetric-rows page should contain {scope}"
        );
    }
    // 4 バリエーション合計セル数（基準形 4 + 分割形 5 + ジグザグ形 4 +
    // 混在形 4 = 17）。バリエーションごとの内訳・対応表 ID はモジュール
    // doc「4 バリエーションと対応表 ID の対応」節参照。
    assert_eq!(
        html.matches("data-blocks-bento-asymmetric-rows-cell=\"")
            .count(),
        17,
        "bento-asymmetric-rows should render exactly 17 cells across its 4 variants"
    );
    for absent in ["<form", "href=\"#\"", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "bento-asymmetric-rows should never contain {absent}"
        );
    }
}

/// `blog-list-image`（イシュー #2812）の CSS フック配線検証。
/// `blog_featured_with_list_page_wires_demo_class_and_css_hooks` と同型。
#[test]
fn blog_list_image_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/blog-list-image/index.html"))
        .expect("blocks/blog-list-image/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-blog-list-image\""),
        "blog-list-image page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "blog-list-image page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "blog-list-image page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-blog-list-image-image=\"\"",
        "data-blocks-blog-list-image-main=\"\"",
        "data-blocks-blog-list-image-category=\"\"",
        "data-blocks-blog-list-image-separator=\"\"",
        "data-blocks-blog-list-image-author=\"\"",
        "data-blocks-blog-list-image-author-link=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "blog-list-image page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-blog-list-image-image]",
        "[data-blocks-blog-list-image-main]",
        "[data-blocks-blog-list-image-separator]",
        "[data-blocks-blog-list-image-author]",
        "[data-blocks-blog-list-image-author-link]",
        ".blocks-blog-list-image-list",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// blog-list-image の合成部品（heading/text/badge/image/avatar/separator/
/// link/link-overlay）が期待どおりの構成で実際に出力されていること
/// （記事 3 件・外部リンクのみ）、`<form>`・`href="#"`・`src="data:` を
/// 持ち込んでいないことを固定する（イシュー #2812）。
#[test]
fn blog_list_image_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/blog-list-image/index.html"))
        .expect("blocks/blog-list-image/index.html should be generated");
    for scope in [
        "data-scope=\"heading\"",
        "data-scope=\"text\"",
        "data-scope=\"badge\"",
        "data-scope=\"image\"",
        "data-scope=\"avatar\"",
        "data-scope=\"separator\"",
        "data-scope=\"link\"",
        "data-scope=\"link-overlay\"",
    ] {
        assert!(
            html.contains(scope),
            "blog-list-image page should contain {scope}"
        );
    }
    // ページ本文全体を包む `article.docs-content`（`crate::layout`）が別に
    // 1 件存在するため、`<article` の総数ではなく block 固有 class を数える。
    assert_eq!(
        html.matches("class=\"blocks-blog-list-image-article\"")
            .count(),
        3,
        "blog-list-image should render exactly 3 article instances"
    );
    assert!(
        html.contains("href=\"https://github.com/Fandhe-AI/fandhe-frontend\""),
        "blog-list-image should link to the fixed repository URL"
    );
    for absent in ["<form", "href=\"#\"", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "blog-list-image should never contain {absent}"
        );
    }
}

/// blog-overlay-cards ページの Demo クラス・両スタイルシート・
/// `data-blocks-blog-overlay-cards-*` CSS フックが実際に出力され、
/// `blocks::stylesheet()` にも対応するセレクタ・`@media` クエリ・
/// 詳細度引き上げセレクタが存在することを固定する（イシュー #2813）。
#[test]
fn blog_overlay_cards_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/blog-overlay-cards/index.html"))
        .expect("blocks/blog-overlay-cards/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-blog-overlay-cards\""),
        "blog-overlay-cards page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "blog-overlay-cards page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "blog-overlay-cards page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-blog-overlay-cards-card=\"\"",
        "data-blocks-blog-overlay-cards-link=\"\"",
        "data-blocks-blog-overlay-cards-bg=\"\"",
        "data-blocks-blog-overlay-cards-title=\"\"",
        "data-blocks-blog-overlay-cards-avatar=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "blog-overlay-cards page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for needle in [
        ".blocks-blog-overlay-cards-grid",
        "grid-auto-rows: 1fr",
        "@media (min-width: 64rem)",
        "[data-scope=\"card\"][data-part=\"root\"][data-blocks-blog-overlay-cards-card]",
        "[data-scope=\"image\"][data-part=\"root\"][data-blocks-blog-overlay-cards-bg]",
        ".blocks-blog-overlay-cards-scrim",
        "var(--fandhe-color-fg)",
    ] {
        assert!(
            sheet_css.contains(needle),
            "blocks.css should declare/reference {needle} for blog-overlay-cards"
        );
    }
}

/// blog-overlay-cards の合成部品（heading/text/card/image/avatar/
/// link-overlay）が期待どおりの構成で実際に出力されていること（記事カード
/// 3 件が出力される）、`<form>`・`href="#"`・`src="data:` を持ち込んで
/// いないことを固定する（イシュー #2813）。
#[test]
fn blog_overlay_cards_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/blog-overlay-cards/index.html"))
        .expect("blocks/blog-overlay-cards/index.html should be generated");
    for scope in [
        "data-scope=\"heading\"",
        "data-scope=\"text\"",
        "data-scope=\"card\"",
        "data-scope=\"image\"",
        "data-scope=\"avatar\"",
        "data-scope=\"link-overlay\"",
    ] {
        assert!(
            html.contains(scope),
            "blog-overlay-cards page should contain {scope}"
        );
    }
    assert_eq!(
        html.matches("data-blocks-blog-overlay-cards-card=\"\"")
            .count(),
        3,
        "blog-overlay-cards should render exactly 3 overlay cards"
    );
    assert!(
        html.contains(r#"data-scope="link-overlay" data-part="overlay""#),
        "blog-overlay-cards should render the link-overlay overlay part"
    );
    for absent in ["<form", "href=\"#\"", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "blog-overlay-cards should never contain {absent}"
        );
    }
}

#[test]
fn blog_split_header_grid_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/blog-split-header-grid/index.html"))
        .expect("blocks/blog-split-header-grid/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-blog-split-header-grid\""),
        "blog-split-header-grid page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "blog-split-header-grid page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "blog-split-header-grid page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-blog-split-header-grid-tagline=\"\"",
        "data-blocks-blog-split-header-grid-heading=\"\"",
        "data-blocks-blog-split-header-grid-view-all=\"\"",
        "data-blocks-blog-split-header-grid-card=\"\"",
        "data-blocks-blog-split-header-grid-image=\"\"",
        "data-blocks-blog-split-header-grid-category=\"\"",
        "data-blocks-blog-split-header-grid-title-link=\"\"",
        "data-blocks-blog-split-header-grid-author=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "blog-split-header-grid page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        ".blocks-blog-split-header-grid-layout",
        ".blocks-blog-split-header-grid-lead",
        ".blocks-blog-split-header-grid-grid",
        "[data-blocks-blog-split-header-grid-card]",
        "[data-blocks-blog-split-header-grid-image]",
        "[data-blocks-blog-split-header-grid-title-link]",
        "@media (min-width: 64rem)",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks::stylesheet should declare {selector}"
        );
    }
}

#[test]
fn blog_split_header_grid_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/blog-split-header-grid/index.html"))
        .expect("blocks/blog-split-header-grid/index.html should be generated");
    for scope in [
        "data-scope=\"heading\"",
        "data-scope=\"text\"",
        "data-scope=\"badge\"",
        "data-scope=\"card\"",
        "data-scope=\"image\"",
        "data-scope=\"link\"",
    ] {
        assert!(
            html.contains(scope),
            "blog-split-header-grid page should contain {scope}"
        );
    }
    for absent in ["<form", "href=\"#\"", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "blog-split-header-grid should never contain {absent}"
        );
    }
    assert_eq!(
        html.matches("data-blocks-blog-split-header-grid-card=\"\"")
            .count(),
        4,
        "blog-split-header-grid should render exactly 4 article cards"
    );
    // 「すべての記事を見る」は codex レビュー是正（イシュー #2814
    // PR #3165）で dead button から実際に遷移する link へ置き換えた。
    // ページ全体にはヘッダーの GitHub リンク等 block 外にも REPO への
    // href が存在する（`docs-github-link`）ため、全体の href 出現数では
    // 断定せず、view-all の CSS フック属性を持つ要素そのものの開始タグに
    // REPO への href が含まれることを確認する。
    let view_all_tag_start = html
        .find("data-blocks-blog-split-header-grid-view-all=\"\"")
        .and_then(|hook_pos| html[..hook_pos].rfind('<'))
        .expect("blog-split-header-grid should render the view-all element");
    let view_all_tag_end = html[view_all_tag_start..]
        .find('>')
        .map(|offset| view_all_tag_start + offset)
        .expect("view-all element's opening tag should be well-formed");
    let view_all_tag = &html[view_all_tag_start..view_all_tag_end];
    assert!(
        view_all_tag.starts_with("<a "),
        "blog-split-header-grid's view-all control should be a real <a> link, not a dead button: {view_all_tag}"
    );
    assert!(
        view_all_tag.contains("href=\"https://github.com/Fandhe-AI/fandhe-frontend\""),
        "blog-split-header-grid's view-all link should point at the fixed repository URL: {view_all_tag}"
    );
    assert_eq!(
        html.matches("data-blocks-blog-split-header-grid-title-link=\"\"")
            .count(),
        4,
        "blog-split-header-grid should render exactly 4 article title links"
    );
}

/// bento-three-column-tall ページが `blocks-demo` + block 固有 class・両
/// スタイルシート・各セルの配置フック（`data-blocks-bento-three-column-
/// tall-cell`）を実際に出力し、`blocks::stylesheet()` にも対応するグリッド
/// 配置規則・ブレークポイント条件が存在することを固定する（イシュー
/// #2748/#2749）。
#[test]
fn bento_three_column_tall_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/bento-three-column-tall/index.html"))
        .expect("blocks/bento-three-column-tall/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-bento-three-column-tall\""),
        "bento-three-column-tall page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "bento-three-column-tall page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "bento-three-column-tall page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-bento-three-column-tall-cell=\"start\"",
        "data-blocks-bento-three-column-tall-cell=\"center-top\"",
        "data-blocks-bento-three-column-tall-cell=\"center-bottom\"",
        "data-blocks-bento-three-column-tall-cell=\"end\"",
        "data-blocks-bento-three-column-tall-cell=\"end-top\"",
        "data-blocks-bento-three-column-tall-cell=\"end-bottom\"",
        "data-blocks-bento-three-column-tall-media=\"terminal\"",
        "data-blocks-bento-three-column-tall-media=\"code\"",
        "data-blocks-bento-three-column-tall-variant=\"both-tall\"",
        "data-blocks-bento-three-column-tall-variant=\"start-tall\"",
    ] {
        assert!(
            html.contains(hook),
            "bento-three-column-tall page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-bento-three-column-tall-cell",
        "[data-blocks-bento-three-column-tall-media",
        ".blocks-bento-three-column-tall-grid",
        ".blocks-bento-three-column-tall-header",
        "@media (min-width: 64rem)",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// bento-three-column-tall の合成部品（badge/heading/text/button/card/
/// image/code）が期待どおりの構成（カード 9 枚・CTA ボタン 1 個）で実際に
/// 出力されていること、`<form>`・`data:` URI・`href="#"` を持ち込んで
/// いないことを固定する（イシュー #2748/#2749）。
#[test]
fn bento_three_column_tall_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/bento-three-column-tall/index.html"))
        .expect("blocks/bento-three-column-tall/index.html should be generated");
    for scope in [
        "data-scope=\"badge\"",
        "data-scope=\"heading\"",
        "data-scope=\"text\"",
        "data-scope=\"button\"",
        "data-scope=\"card\" data-part=\"root\"",
        "data-scope=\"image\"",
        "data-scope=\"code\"",
    ] {
        assert!(
            html.contains(scope),
            "bento-three-column-tall page should contain {scope}"
        );
    }
    assert_eq!(
        html.matches("data-scope=\"card\" data-part=\"root\"")
            .count(),
        9,
        "bento-three-column-tall should render exactly 9 cards (4 + 5)"
    );
    assert!(
        html.contains("type=\"button\""),
        "bento-three-column-tall should render a CTA button"
    );
    for absent in ["<form", "src=\"data:", "href=\"#\""] {
        assert!(
            !html.contains(absent),
            "bento-three-column-tall should never contain {absent}"
        );
    }
}

/// `content-article-toc` ページが `class="blocks-demo blocks-content-article-toc"`
/// を持ち、`pre-styled-ui.css`/`blocks.css` の両方が配線され、主要な
/// `data-blocks-content-article-toc-*` フックが `blocks::stylesheet()` の
/// 対応するセレクタ（`@media (min-width: 64rem)` を含む）と対になっている
/// ことを固定する（イシュー #2752）。
#[test]
fn content_article_toc_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/content-article-toc/index.html"))
        .expect("blocks/content-article-toc/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-content-article-toc\""),
        "content-article-toc demo wrapper should carry both the shared and block-specific class"
    );
    for link in ["assets/pre-styled-ui.css", "assets/blocks.css"] {
        assert!(
            html.contains(link),
            "content-article-toc page should link {link}"
        );
    }
    for hook in [
        "data-blocks-content-article-toc-nav",
        "data-blocks-content-article-toc-cover",
    ] {
        assert!(
            html.contains(hook),
            "content-article-toc page should output the {hook} CSS hook attribute"
        );
    }

    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for needle in [
        ".blocks-content-article-toc-layout",
        "[data-blocks-content-article-toc-nav]",
        "@media (min-width: 64rem)",
    ] {
        assert!(
            sheet_css.contains(needle),
            "blocks.css should contain {needle}"
        );
    }
}

/// careers-split-photo-list ページが `blocks-demo`/固有 demo_class・
/// 専用スタイルシート 2 種・Demo 固有 CSS フックを配線していること
/// （イシュー #2817）。
#[test]
fn careers_split_photo_list_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/careers-split-photo-list/index.html"))
        .expect("blocks/careers-split-photo-list/index.html should be generated");
    assert!(
        html.contains(r#"class="blocks-demo blocks-careers-split-photo-list""#),
        "careers-split-photo-list page should wire the shared and block-specific demo class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "careers-split-photo-list page should link the pre-styled-ui stylesheet"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "careers-split-photo-list page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-careers-split-photo-list-job=\"\"",
        "data-blocks-careers-split-photo-list-photo=\"\"",
        "data-blocks-careers-split-photo-list-separator=\"\"",
        "data-blocks-careers-split-photo-list-salary=\"\"",
        "data-blocks-careers-split-photo-list-location=\"\"",
        "data-blocks-careers-split-photo-list-all-link=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "careers-split-photo-list page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        ".blocks-careers-split-photo-list-layout",
        "[data-blocks-careers-split-photo-list-job]",
        "@media (min-width: 64rem)",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

#[test]
fn changelog_accordion_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/changelog-accordion/index.html"))
        .expect("blocks/changelog-accordion/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-changelog-accordion\""),
        "changelog-accordion page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "changelog-accordion page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "changelog-accordion page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-changelog-accordion-root=\"\"",
        "data-blocks-changelog-accordion-tag=\"\"",
        "data-blocks-changelog-accordion-image=\"\"",
        "data-blocks-changelog-accordion-changes=\"\"",
    ] {
        assert!(
            html.contains(hook),
            "changelog-accordion page should contain {hook}"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    assert!(
        sheet_css.contains(
            r#".blocks-changelog-accordion-list [data-scope="accordion"][data-part="item"] {"#
        ),
        "blocks.css should declare the per-item frame override for changelog-accordion"
    );
}

#[test]
fn changelog_accordion_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/changelog-accordion/index.html"))
        .expect("blocks/changelog-accordion/index.html should be generated");
    for scope in [
        "data-scope=\"heading\"",
        "data-scope=\"text\"",
        "data-scope=\"badge\"",
        "data-scope=\"accordion\"",
        "data-scope=\"list\"",
        "data-scope=\"image\"",
    ] {
        assert!(
            html.contains(scope),
            "changelog-accordion page should contain {scope}"
        );
    }
    // イシュー #2818 レビュー指摘の是正: 無 JS の docs サイトでは
    // `item_trigger` の click/Enter/Space が no-op のため、閉じた項目
    // （`hidden` 属性で本文が到達不能）を残さない。全件を open + disabled
    // 固定にする（`changelog_accordion.rs` モジュール doc「静的表示」節）。
    let open_count = html
        .matches(r#"data-part="item" data-state="open""#)
        .count();
    assert!(
        open_count >= 1,
        "changelog-accordion should render at least 1 open item"
    );
    assert!(
        !html.contains(r#"data-part="item-content" data-state="closed""#),
        "changelog-accordion should not render closed (unreachable) item-content"
    );
    for absent in ["<form", "href=\"#\"", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "changelog-accordion should never contain {absent}"
        );
    }
}

#[test]
fn changelog_accordion_item_frame_selector_outweighs_recipe_last_child() {
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    assert!(
        sheet_css.contains(
            r#".blocks-changelog-accordion-list [data-scope="accordion"][data-part="item"]:last-child {"#
        ),
        "blocks.css should declare a last-child border override at least as specific as \
         the recipe's own :last-child rule for changelog-accordion"
    );
}

/// bento-two-column ページが `blocks-demo` + block 固有 class・両スタイル
/// シート・カードの配置フック（`data-blocks-bento-two-column-cell`）を
/// 実際に出力し、`blocks::stylesheet()` にも対応するグリッド配置規則・
/// ブレークポイント条件が存在することを固定する（イシュー #2750）。
#[test]
fn bento_two_column_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/bento-two-column/index.html"))
        .expect("blocks/bento-two-column/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-bento-two-column\""),
        "bento-two-column page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "bento-two-column page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "bento-two-column page should link the Blocks-specific stylesheet"
    );
    for hook in [
        "data-blocks-bento-two-column-cell=\"featured\"",
        "data-blocks-bento-two-column-cell=\"base\"",
    ] {
        assert!(
            html.contains(hook),
            "bento-two-column page should output the {hook} CSS hook attribute"
        );
    }
    let sheet_css = blocks::stylesheet()
        .expect("blocks::stylesheet should build")
        .as_css()
        .to_string();
    for selector in [
        "[data-blocks-bento-two-column-cell",
        ".blocks-bento-two-column-grid",
        "@media (min-width: 48rem)",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// `content-article-toc` が使用部品 7 件（badge/heading/text/image/avatar/
/// nav-list/separator）を実際に合成しており、目次リンクの `href="#…"` が
/// 同一 Demo 内の `id` と一対一で往復対応し、`<form>`・死リンク・`data:`
/// URI・`aria-current` を持たないことを固定する（イシュー #2752）。
///
/// Demo 単体（`render(&(block.demo)())`）に対して判定することで、ページ側
/// サイドバーが現在ページへ出力する `aria-current="page"` を巻き込まない
/// （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` と
/// 同じ判断軸）。
#[test]
fn content_article_toc_composes_expected_parts() {
    let block = blocks::all_blocks()
        .into_iter()
        .find(|b| b.path == "/blocks/content-article-toc/")
        .expect("content-article-toc block should be registered");
    let demo_html = render(&(block.demo)());

    for scope in [
        "data-scope=\"badge\"",
        "data-scope=\"heading\"",
        "data-scope=\"text\"",
        "data-scope=\"image\"",
        "data-scope=\"avatar\"",
        "data-scope=\"nav-list\"",
        "data-scope=\"separator\"",
    ] {
        assert!(
            demo_html.contains(scope),
            "content-article-toc demo should contain {scope}"
        );
    }
    assert_eq!(
        demo_html
            .matches("data-scope=\"nav-list\" data-part=\"root\"")
            .count(),
        2,
        "content-article-toc should render exactly 2 nav_list roots (instance A/B)"
    );
    assert!(
        demo_html.contains("基準形: 見出し・カバー画像を縦に積み"),
        "content-article-toc should render the instance A caption"
    );
    assert!(
        demo_html.contains("横並び形: 64rem 以上で見出し群と画像を横に並べ"),
        "content-article-toc should render the instance B caption"
    );

    let mut hrefs: Vec<&str> = Vec::new();
    let mut offset = 0usize;
    let needle = "href=\"#";
    while let Some(rel) = demo_html[offset..].find(needle) {
        let start = offset + rel + needle.len();
        let end = demo_html[start..]
            .find('"')
            .map(|i| start + i)
            .expect("href attribute should be closed");
        hrefs.push(&demo_html[start..end]);
        offset = end;
    }
    assert_eq!(
        hrefs.len(),
        6,
        "content-article-toc should have 6 toc links (3 sections x 2 instances)"
    );
    for frag in &hrefs {
        let id_needle = format!("id=\"{frag}\"");
        assert_eq!(
            demo_html.matches(&id_needle).count(),
            1,
            "content-article-toc toc link #{frag} should have exactly one matching id"
        );
    }

    for absent in ["<form", "href=\"#\"", "src=\"data:", "aria-current"] {
        assert!(
            !demo_html.contains(absent),
            "content-article-toc demo should never contain {absent}"
        );
    }
}

/// careers-split-photo-list の合成部品（heading/text/image/separator/link/
/// link-overlay/visually-hidden）が期待どおりの構成（求人 3 件・区切り線
/// 2 本）で実際に出力されていること、`<form>`・`data:` URI・`href="#"` を
/// 持ち込んでいないことを固定する（イシュー #2817）。
#[test]
fn careers_split_photo_list_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/careers-split-photo-list/index.html"))
        .expect("blocks/careers-split-photo-list/index.html should be generated");
    for scope in [
        "data-scope=\"heading\"",
        "data-scope=\"text\"",
        "data-scope=\"image\"",
        "data-scope=\"separator\"",
        "data-scope=\"link\"",
        "data-scope=\"link-overlay\"",
        "data-scope=\"visually-hidden\"",
    ] {
        assert!(
            html.contains(scope),
            "careers-split-photo-list page should contain {scope}"
        );
    }
    // 「## Rust コード」節の手書きソース表示にも同じ
    // `data-blocks-careers-split-photo-list-*` 属性名がリテラル文字列として
    // 現れるため（`Block::demo` の実装そのものを表示する節、
    // `crate::blocks` モジュール doc「マーカー規約」参照）、件数の厳密な
    // 検証は「Demo」節（`>Rust コード</h2>` より前）に限定する
    // （`login_01_page_orders_h1_then_demo_then_used_parts_then_rust_code`
    // と同じ境界検出手法）。
    let rust_code_pos = html
        .find(">Rust コード</h2>")
        .expect("page should have a Rust コード heading");
    let demo_html = &html[..rust_code_pos];
    assert_eq!(
        demo_html
            .matches("data-blocks-careers-split-photo-list-job")
            .count(),
        3,
        "careers-split-photo-list should render exactly 3 job rows"
    );
    assert_eq!(
        demo_html
            .matches("data-blocks-careers-split-photo-list-separator")
            .count(),
        2,
        "careers-split-photo-list should render exactly 2 separators"
    );
    assert!(
        demo_html.contains(r#"aria-label="バックエンドエンジニア""#),
        "careers-split-photo-list should label the overlay with the visible job title"
    );
    assert!(
        demo_html.contains("一緒にプロダクトを育てる仲間を募集しています"),
        "careers-split-photo-list should render the section heading text"
    );
    for absent in ["<form", "src=\"data:", "href=\"#\""] {
        assert!(
            !html.contains(absent),
            "careers-split-photo-list should never contain {absent}"
        );
    }
}

/// bento-two-column の合成部品（badge/heading/text/card/image）が期待どおり
/// の構成（カード 7 枚: 基準形 4 枚 + 全幅形 3 枚）で実際に出力されている
/// こと、`<form>`・`data:` URI・`href="#"` を持ち込んでいないことを固定
/// する（イシュー #2750）。
#[test]
fn bento_two_column_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/bento-two-column/index.html"))
        .expect("blocks/bento-two-column/index.html should be generated");
    for scope in [
        "data-scope=\"badge\"",
        "data-scope=\"heading\"",
        "data-scope=\"text\"",
        "data-scope=\"card\" data-part=\"root\"",
        "data-scope=\"image\"",
    ] {
        assert!(
            html.contains(scope),
            "bento-two-column page should contain {scope}"
        );
    }
    assert_eq!(
        html.matches("data-scope=\"card\" data-part=\"root\"")
            .count(),
        7,
        "bento-two-column should render exactly 7 cards"
    );
    for absent in ["<form", "src=\"data:", "href=\"#\""] {
        assert!(
            !html.contains(absent),
            "bento-two-column should never contain {absent}"
        );
    }
}

/// content-columns-screenshot の Demo ラッパ・CSS 配線・block 固有 CSS
/// （2 列 grid・md ブレークポイント・フェード用 linear-gradient・image
/// フック）が実際に出力されていることを固定する（イシュー #2753）。
#[test]
fn content_columns_screenshot_page_wires_demo_class_and_css_hooks() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/content-columns-screenshot/index.html"))
        .expect("blocks/content-columns-screenshot/index.html should be generated");
    assert!(
        html.contains("class=\"blocks-demo blocks-content-columns-screenshot\""),
        "content-columns-screenshot page should wrap the Demo in blocks-demo + block-specific class"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#),
        "content-columns-screenshot page should link pre-styled-ui.css (parts' own look)"
    );
    assert!(
        html.contains(r#"href="/fandhe-frontend/assets/blocks.css""#),
        "content-columns-screenshot page should link the Blocks-specific stylesheet"
    );

    let sheet = blocks::stylesheet().expect("blocks::stylesheet() should build");
    let sheet_css = sheet.as_css();
    for selector in [
        ".blocks-content-columns-screenshot-columns",
        "grid-template-columns: repeat(2, minmax(0, 1fr))",
        "@media (max-width: 47.99rem)",
        "linear-gradient(to top, var(--fandhe-color-bg-subtle), transparent)",
        "[data-scope=\"image\"][data-part=\"root\"][data-blocks-content-columns-screenshot-image]",
    ] {
        assert!(
            sheet_css.contains(selector),
            "blocks.css should declare a rule for {selector}"
        );
    }
}

/// content-columns-screenshot の合成部品（badge/heading/text/button/
/// image）が期待どおりの構成で実際に出力されていること、`<form>`・
/// `data:` URI を持ち込んでいないことを固定する（イシュー #2753）。
#[test]
fn content_columns_screenshot_composes_expected_parts() {
    let out = build_real_site();
    let html = std::fs::read_to_string(out.join("blocks/content-columns-screenshot/index.html"))
        .expect("blocks/content-columns-screenshot/index.html should be generated");
    for scope in [
        "data-scope=\"badge\"",
        "data-scope=\"heading\"",
        "data-scope=\"text\"",
        "data-scope=\"button\"",
        "data-scope=\"image\"",
    ] {
        assert!(
            html.contains(scope),
            "content-columns-screenshot page should contain {scope}"
        );
    }
    assert_eq!(
        html.matches("<img").count(),
        1,
        "content-columns-screenshot should render exactly 1 screenshot image"
    );
    assert!(html.contains("src=\"../../assets/blocks-demo-screenshot.svg\""));
    // 4 件は Demo 出力（2 列 × 2 段落）、残り 1 件は「Rust コード」節が表示
    // する原稿フェンス内のソースコード自体に含まれる同じ属性名のリテラル
    // （`blocks_code_drift.rs` が実装との一致を固定するマーカー内容）。
    assert_eq!(
        html.matches("data-blocks-content-columns-screenshot-paragraph")
            .count(),
        5,
        "content-columns-screenshot should render 4 paragraphs across the 2 columns \
         plus 1 occurrence in the displayed Rust source"
    );
    assert!(html.contains("type=\"button\""));
    for absent in ["<form", "src=\"data:"] {
        assert!(
            !html.contains(absent),
            "content-columns-screenshot should never contain {absent}"
        );
    }
}

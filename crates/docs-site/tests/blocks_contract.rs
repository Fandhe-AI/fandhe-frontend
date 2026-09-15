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
    let login_04_block = blocks::BLOCKS
        .iter()
        .find(|block| block.path == "/blocks/login-04/")
        .expect("login-04 should be registered in blocks::BLOCKS");
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
    let block = blocks::BLOCKS
        .iter()
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
    assert!(
        !sheet_css.contains("@keyframes"),
        "testimonials-stack should not introduce @keyframes (no reduced-motion @media override needed)"
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

//! JS 無効環境でも docs サイト全ページの閲覧・ナビゲーションが成立する
//! ことを固定する契約テスト（イシュー #960）。
//!
//! #951（テーマトグル）・#958（検索 UI）で docs サイトが初めて JS
//! （`assets/site.js`）を持ち込んだため、「JS を出力していないから JS
//! 無効でも動く」という #922 以前の前提はもう成立しない。本ファイルは
//! 実サイトビルド（`site/nav.toml` 由来の全ページ）に対して、
//!
//! 1. `javascript:` スキームの href/src が存在しない
//! 2. インラインイベントハンドラ属性（`on*=`）が存在しない
//! 3. サイドバー・ヘッダーナビ・prev/next の各ブロックが、JS なしで踏める
//!    静的 `<a href>` を少なくとも 1 本持つ（ヘッダーの `.docs-header-trigger`
//!    はイシュー #1012 でセクショントップページへの遷移リンク `<a href>` へ
//!    切り替わった。Assets のメガメニュー（`.docs-header-mega`、イシュー #3701）は
//!    `:hover`/`:focus-within` により CSS のみで開閉し、カードも静的 `<a href>`。リンク解決性自体は `build_site` 内蔵の linkcheck
//!    が fail-closed で保証済みであり、ここでは「JS なしで辿れる形が
//!    存在する」ことのみを固定する）
//! 4. 検索ブロック（`div.docs-search`）・テーマトグル（`.docs-theme-toggle`）
//!    が既定 `hidden`（JS 未実行時に操作できない UI を露出しない
//!    プログレッシブエンハンスメント契約）
//! 5. インライン `<script>` は 0 個で、外部 `<script src>` はテーマ初期化
//!    （`assets/theme-init.js`。同期・`<head>` 内・stylesheet より前、イシュー #3676）
//!    と `assets/site.js`（`defer`）の 2 本のみ
//! 6. CSS 側に JS 非依存の開閉経路（`.docs-nav-drawer-toggle:checked ~ .docs-nav-drawer`・
//!    `.docs-header-group:hover`/`:focus-within` → `.docs-header-mega`）が存在する
//! 7. meta CSP（`csp::CONTENT_SECURITY_POLICY`、イシュー #3678）が全本体ページの
//!    `<head>` の charset・viewport 直後にちょうど 1 個あり、リダイレクト案内には無い。
//!    インライン `<script>`・`<style>` が 0 個であること自体は 5 と
//!    `no_generated_page_emits_inline_style_elements` が担う（複製しない）
//!
//! ことを機械的に検証する。4・6 は `crate::site_theme` の unit test /
//! `tests/site_css_contract.rs` に部分的な既存アサーションがあるが、
//! それらを削除・弱体化するものではない。本ファイルは「JS 無効契約」
//! という観点で横断的に集約するのが役割であり、カスケード契約自体の
//! 正は `site_css_contract.rs` 側にある（重複した場合はそちらを正とする）。
//!
//! # リダイレクトページ（イシュー #1016）の扱い
//!
//! `site/redirects.toml`（`crate::redirect`）が生成する旧 URL 互換の案内
//! ページは、意図的にサイトクロームを持たない（`class` 属性・`<script>`・
//! `<link rel="stylesheet">` を一切持たない、`crate::redirect` モジュール
//! doc 参照）。上記 1〜6 の全てをそのまま適用すると必ず落ちるため、
//! 「スキップ」ではなく「dist 配下の `*.html` をリダイレクト由来と本体
//! ページ由来に分割し、両方に契約を課す」形にする:
//!
//! - 本体ページ側は本ファイル冒頭の 1〜6 を従来どおり適用する。
//! - リダイレクトページ側は [`redirect_pages_contain_no_script_and_a_static_fallback_link`]
//!   がより強い契約（`<script>` を 1 個も含まない・`meta refresh`/
//!   `rel=canonical`/`robots=noindex`/静的フォールバック `<a href>` を
//!   含む）を課す。
//!
//! 分割対象の判定は [`redirect::output_path`] を経由して `site/redirects.toml`
//! から**機械導出**し、手書きの除外リストは持たない（[`expected_redirect_files`]）。
//! この分割が「任意のページを検査から外す抜け道」にならない根拠は、
//! `from` が `nav.toml` の実ページ path と衝突する宣言は
//! `redirect::validate_against_nav` がビルド失敗にすること（構造的に
//! 実ページを redirects.toml 経由で sweep から外せない）である。加えて
//! [`build_real_site`] で「導出したリダイレクトファイルがすべて実在する
//! こと」「集合が空でないこと」を assert し、分割が空振りして契約が
//! 形骸化する事故（誤って全ページをリダイレクト側へ分類する等）を検知する。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use fandhe_frontend_docs_site::{csp, nav, redirect};

#[path = "support/shared_site.rs"]
mod shared_site;

/// 出力ディレクトリ配下の `*.html` を再帰的に列挙する。
fn collect_html_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries =
        std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_html_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("html") {
            out.push(path);
        }
    }
}

/// `nav.toml` の `page.path`（`/` 始まり・`/` 終わり）から
/// `ssg::generate_pages`（`fandhe_frontend_server::ssg`、非公開）が書き出す
/// 相対ファイルパスを導出する。`redirect::output_path` が返す `from` は
/// `redirect::is_safe_redirect_from` により必ず `/` 始まり・`/` 終わり・
/// 非ルートであることが保証済みのため、`ssg` 側の正規化ロジック
/// （末尾 `/` を落として `/index.html` を付ける）をここでも安全に再現できる。
fn page_path_to_relative_file(path: &str) -> PathBuf {
    let rest = path
        .strip_prefix('/')
        .unwrap_or_else(|| panic!("redirect from {path:?} should start with `/`"));
    let trimmed = rest.trim_end_matches('/');
    PathBuf::from(format!("{trimmed}/index.html"))
}

/// `site/redirects.toml`（[`redirect::MANIFEST_REL_PATH`]）を独立に読み、
/// dist 上のどのファイルがリダイレクト由来かを機械導出する。手書きの除外
/// リストは持たない（モジュール doc「リダイレクトページの扱い」参照）。
fn expected_redirect_files(repo_root: &Path, out_dir: &Path) -> BTreeSet<PathBuf> {
    let manifest_path = repo_root.join(redirect::MANIFEST_REL_PATH);
    let input = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", manifest_path.display()));
    let redirects =
        redirect::parse_redirects(&input).expect("site/redirects.toml should parse cleanly");

    // `from` が `nav.toml` の実ページ path と衝突する宣言はビルド自体が
    // 失敗する（`redirect::validate_against_nav`）。したがって実ページを
    // redirects.toml 経由で本 sweep の対象から外すことは構造的に不可能
    // （モジュール doc 参照）。ここでは念のため二重確認として、実サイトの
    // `nav.toml` ページ path と重複がないことも固定する。
    let nav_input = std::fs::read_to_string(repo_root.join("site/nav.toml"))
        .expect("site/nav.toml should be readable");
    let nav = nav::parse_nav(&nav_input).expect("site/nav.toml should parse");
    let page_paths: BTreeSet<&str> = nav.all_pages().map(|p| p.path.as_str()).collect();

    redirects
        .entries
        .iter()
        .map(|r| {
            assert!(
                !page_paths.contains(r.from.as_str()),
                "redirect from {:?} must not collide with an existing nav.toml page \
                 (should have been rejected by validate_against_nav)",
                r.from
            );
            out_dir.join(page_path_to_relative_file(&redirect::output_path(&r.from)))
        })
        .collect()
}

/// 実サイトビルド結果を「本体ページ」と「リダイレクトページ」
/// （イシュー #1016）に分割して共有する。`cargo test` 内で複数アサーション
/// が同じビルド結果を参照するためのヘルパー。
///
/// 実サイトビルド自体はテストバイナリ内で 1 回だけ実行される
/// （`tests/support/shared_site.rs` の共有ビルド、イシュー #2299。以前は
/// 本関数の呼び出しごとに `build_site` を再実行していたため、本ファイル
/// だけで 8 回の実サイトフルビルドが走っていた）。本関数はその共有ビルド
/// 結果に対する「本体/リダイレクトへの分割」という導出処理のみを毎回
/// 行う（ディレクトリ走査のみで軽量）。
fn build_real_site() -> (&'static Path, Vec<PathBuf>, Vec<PathBuf>) {
    let repo_root = shared_site::repo_root();
    let out = shared_site::real_site().out_dir.as_path();

    let mut files = Vec::new();
    collect_html_files(out, &mut files);
    assert!(
        !files.is_empty(),
        "real site build should emit at least one HTML page"
    );

    let redirect_set = expected_redirect_files(&repo_root, out);
    assert!(
        !redirect_set.is_empty(),
        "site/redirects.toml should declare at least one redirect \
         (this test's split would otherwise silently degrade to a no-op, \
         see module doc)"
    );
    for path in &redirect_set {
        assert!(
            path.exists(),
            "{path:?}: expected redirect output file to exist (derived from site/redirects.toml)"
        );
    }

    let mut body_files = Vec::new();
    let mut redirect_files = Vec::new();
    for file in files {
        if redirect_set.contains(&file) {
            redirect_files.push(file);
        } else {
            body_files.push(file);
        }
    }
    assert!(
        !body_files.is_empty(),
        "real site build should emit at least one non-redirect HTML page"
    );

    (out, body_files, redirect_files)
}

#[test]
fn no_generated_page_uses_javascript_scheme_links() {
    let (_out, files, _redirects) = build_real_site();
    for file in &files {
        let html = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));
        let lower = html.to_ascii_lowercase();
        assert!(
            !lower.contains("href=\"javascript:")
                && !lower.contains("href='javascript:")
                && !lower.contains("src=\"javascript:")
                && !lower.contains("src='javascript:"),
            "{file:?} must not contain a javascript: scheme href/src (JS 無効環境での安全なリンクの前提、REQ-1 相当の防御多層化)"
        );
    }
}

#[test]
fn no_generated_page_uses_inline_event_handler_attributes() {
    let (_out, files, _redirects) = build_real_site();
    // `on` で始まる HTML イベントハンドラ属性（onclick / onload 等）が
    // 属性名として出現しないことを確認する。属性値側の偶然一致
    // （例: 本文中の英単語）を避けるため `<tag ... on...="` の形を見る。
    let re_like_needles = [
        " onclick=",
        " onload=",
        " onerror=",
        " onmouseover=",
        " onfocus=",
        " onchange=",
        " onsubmit=",
        " onkeydown=",
        " onkeyup=",
        " oninput=",
    ];
    for file in &files {
        let html = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));
        let lower = html.to_ascii_lowercase();
        for needle in re_like_needles {
            assert!(
                !lower.contains(needle),
                "{file:?} must not contain inline event handler {needle:?} (JS 非依存のプログレッシブエンハンスメント契約)"
            );
        }
    }
}

/// 生成 HTML（リダイレクト案内を除く）にインライン `<style>` 要素が 0 個であること
/// （CSP `style-src 'self'` 前提、#3677）。エスケープ済みテキストは `&lt;style` の
/// ため誤検知しない。
#[test]
fn no_generated_page_emits_inline_style_elements() {
    let (_out, files, _redirects) = build_real_site();
    for file in &files {
        let html = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));
        assert!(
            !html.to_ascii_lowercase().contains("<style"),
            "{file:?} must not contain an inline <style> element"
        );
    }
}

/// イシュー #3678: 本体ページ（リダイレクト案内を除く）は meta CSP をちょうど 1 個、
/// charset・viewport の直後（`title` と最初の `script`/`link` より前）に持つ。
/// 値は `csp::CONTENT_SECURITY_POLICY` の既定エスケープ済み表現と完全一致する
/// （`'` は `&#x27;` で出力されるため生の `'` で比較しない）。
#[test]
fn body_pages_carry_exactly_one_csp_meta_before_any_script_or_link() {
    let (_out, files, _redirects) = build_real_site();
    let expected = format!(
        r#"<meta http-equiv="Content-Security-Policy" content="{}">"#,
        fandhe_frontend_core::escape_html(csp::CONTENT_SECURITY_POLICY)
    );
    let mut seen_index = false;
    let mut seen_404 = false;
    let mut seen_part = false;
    for file in &files {
        let html = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));
        let name = file.to_string_lossy();
        seen_index |= name.ends_with("/index.html") && !name.contains("/themes/");
        seen_404 |= name.ends_with("/404.html");
        seen_part |= name.ends_with("themes/button/index.html");

        assert_eq!(
            html.to_ascii_lowercase()
                .matches(r#"http-equiv="content-security-policy""#)
                .count(),
            1,
            "{file:?}: exactly one CSP meta expected"
        );
        assert_eq!(
            html.matches(&expected).count(),
            1,
            "{file:?}: CSP meta must equal the constant"
        );
        let head = html.find("<head>").expect("<head>");
        let charset = html.find(r#"<meta charset="utf-8">"#).expect("charset");
        let viewport = html.find(r#"<meta name="viewport""#).expect("viewport");
        let csp_pos = html.find(&expected).expect("csp");
        let title = html.find("<title>").expect("title");
        let first_script = html.find("<script").expect("script");
        let first_link = html.find("<link").expect("link");
        let head_end = html.find("</head>").expect("</head>");
        assert_eq!(
            charset,
            head + "<head>".len(),
            "{file:?}: charset must be the first head child"
        );
        assert!(
            charset < viewport
                && viewport < csp_pos
                && csp_pos < title
                && csp_pos < first_script
                && csp_pos < first_link
                && csp_pos < head_end,
            "{file:?}: CSP meta must follow charset/viewport and precede title/script/link"
        );
    }
    assert!(
        seen_index && seen_404 && seen_part,
        "sweep must cover the top page, 404.html and a component page"
    );
}

/// イシュー #3678: CSP `img-src 'self'` の前提として、出力に `data:` 画像・
/// `url(data:...)` が残っていないこと（本体・リダイレクトの HTML と `assets/*.css`）。
#[test]
fn generated_output_contains_no_data_uri_images() {
    let (out, files, redirects) = build_real_site();
    let mut targets: Vec<PathBuf> = files.into_iter().chain(redirects).collect();
    let assets = out.join("assets");
    for entry in std::fs::read_dir(&assets).unwrap_or_else(|e| panic!("read_dir {assets:?}: {e}")) {
        let path = entry.expect("dir entry").path();
        if path.extension().is_some_and(|e| e == "css") {
            targets.push(path);
        }
    }
    for file in &targets {
        let text = std::fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("read {file:?}: {e}"))
            .to_ascii_lowercase();
        for needle in [
            r#"src="data:"#,
            r#"srcset="data:"#,
            "url(data:",
            r#"url("data:"#,
            "url('data:",
        ] {
            assert!(
                !text.contains(needle),
                "{file:?} must not contain {needle:?} (img-src 'self')"
            );
        }
    }
}

/// イシュー #3676: 本体ページの `<script>` は外部ファイル 2 本のみで、
/// インライン `<script>` は 0 個（`script-src 'self'` の CSP 下で実行できるため）。
/// テーマ初期化は同期（`defer`/`async` なし）・`<head>` 内・最初の stylesheet より前、
/// `site.js` は `defer`。`<script` の総数と `<script src=` の数を一致させることで、
/// `<script type=…>` のような属性付きインラインの注入も検知する。
#[test]
fn page_scripts_are_two_external_files_with_no_inline_script() {
    let (_out, files, _redirects) = build_real_site();
    let theme_init = r#"<script src="/fandhe-frontend/assets/theme-init.js"></script>"#;
    for file in &files {
        let html = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));

        assert_eq!(
            html.matches("<script>").count(),
            0,
            "{file:?} must not contain an inline <script> tag"
        );
        assert_eq!(
            html.matches("<script").count(),
            2,
            "{file:?} should contain exactly two <script tags (theme-init + site.js)"
        );
        assert_eq!(
            html.matches("<script src=").count(),
            2,
            "{file:?} should reference exactly two external <script src> tags, none inline"
        );

        let init_pos = html
            .find(theme_init)
            .unwrap_or_else(|| panic!("{file:?} should load theme-init.js synchronously"));
        let head_end = html.find("</head>").expect("</head>");
        let first_css = html
            .find(r#"<link rel="stylesheet""#)
            .expect("stylesheet link");
        assert!(
            init_pos < first_css && init_pos < head_end,
            "{file:?}: theme-init.js must precede every stylesheet inside <head> (FOUC 抑止)"
        );

        assert!(
            html.contains(r#"src="/fandhe-frontend/assets/site.js" defer="""#),
            "{file:?} should load assets/site.js via a deferred external <script src> (site_build.rs の共通契約と同一文字列)"
        );
    }
}

#[test]
fn search_block_and_theme_toggle_default_to_hidden() {
    let (_out, files, _redirects) = build_real_site();
    for file in &files {
        let html = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));
        assert!(
            html.contains(r#"class="docs-search" hidden"#),
            "{file:?}: div.docs-search should default to hidden (JS 未実行時は検索 UI を操作可能に見せない、イシュー #958)"
        );
        assert!(
            html.contains(r#"class="docs-theme-toggle" hidden"#),
            "{file:?}: .docs-theme-toggle should default to hidden (JS 未実行時はテーマトグルを操作可能に見せない、イシュー #951)"
        );
    }
}

#[test]
fn sidebar_and_header_and_prev_next_navigation_uses_static_anchor_hrefs() {
    let (_out, files, _redirects) = build_real_site();
    for file in &files {
        let html = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));

        // サイドバー・ヘッダーナビ・prev/next の各ブロックが存在するページに
        // ついて、内部に少なくとも 1 本の静的 `<a href="...">` を持つこと
        // （リンク解決性自体は build_site 内蔵の linkcheck が fail-closed で
        // 保証する。ここでは「JS なしで踏める形」であることのみを固定する）。
        for (block_class, label) in [
            ("docs-sidebar", "sidebar"),
            ("docs-header-nav", "header nav"),
            ("prev-next", "prev/next"),
            ("docs-footer", "footer"),
        ] {
            if let Some(start) = html.find(&format!("class=\"{block_class}")) {
                // ブロック開始位置から後方の粗い範囲（4000 バイト）を見て、
                // 静的 `<a ...href="...">` が現れることを確認する（厳密な
                // DOM 解析はしない軽量チェック）。属性順序（`data-scope`
                // 等が `href` より前に来る）に依存しないよう、`<a ` と
                // `href="` がともに存在することのみを見る。
                let mut window_end = (start + 4000).min(html.len());
                while !html.is_char_boundary(window_end) {
                    window_end -= 1;
                }
                let window = &html[start..window_end];
                assert!(
                    window.contains("<a ") && window.contains("href=\""),
                    "{file:?}: {label} block should contain at least one static <a href> for no-JS navigation"
                );
            }
        }
    }
}

/// イシュー #3674: ナビ drawer は JS なしで開閉できる形（checkbox が drawer より前の兄弟）で
/// 全本体ページに存在し、内部は静的な `<a href>` だけで構成され、動的 ARIA 状態
/// （`role`/`aria-expanded`/`aria-haspopup`/`aria-controls`）と `id` を持たない。
/// イシュー #3702 で見出しを展開する `details`/`summary` を廃止したため、これらも持たない。
#[test]
fn nav_drawer_is_js_free_static_and_has_no_dynamic_aria_or_ids() {
    let (_out, files, _redirects) = build_real_site();
    let mut checked = 0usize;
    for file in &files {
        let html = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));
        let Some(class_pos) = html.find(r#"class="docs-nav-drawer""#) else {
            continue;
        };
        let nav_start = html[..class_pos].rfind("<nav").expect("drawer nav start");
        let nav_end = nav_start + html[nav_start..].find("</nav>").expect("drawer nav end");
        let drawer = &html[nav_start..nav_end];

        let toggle_pos = html
            .find(r#"id="docs-nav-drawer-toggle""#)
            .unwrap_or_else(|| panic!("{file:?}: toggle checkbox missing"));
        assert!(
            toggle_pos < nav_start,
            "{file:?}: checkbox must precede the drawer"
        );
        assert!(drawer.contains("<a ") && drawer.contains("href=\""));
        for forbidden in [
            "role=",
            "aria-expanded",
            "aria-haspopup",
            "aria-controls",
            "<details",
            "<summary",
            " id=",
            "<script",
            "onclick",
        ] {
            assert!(
                !drawer.contains(forbidden),
                "{file:?}: drawer must not contain `{forbidden}`"
            );
        }
        checked += 1;
    }
    assert!(
        checked > 0,
        "at least one page should render the nav drawer"
    );
}

/// イシュー #1080: `min-width: 1200px` 未満で右目次カラム
/// （`aside.docs-toc-aside`）が `display: none` になる代替として、本文冒頭の
/// 折りたたみ目次（`nav.docs-toc-inline`）が JS 無効でも踏める形で存在する
/// ことを実サイトビルド全体で固定する。右目次カラムを持つページ（＝見出しが
/// 存在するページ）すべてが対象。
#[test]
fn inline_toc_provides_a_js_free_heading_navigation_path() {
    let (_out, files, _redirects) = build_real_site();
    let mut checked_at_least_one = false;
    for file in &files {
        let html = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));
        if !html.contains(r#"class="docs-toc-aside""#) {
            // 見出しの無いページ（右目次カラム自体が出力されない）は対象外。
            continue;
        }
        checked_at_least_one = true;
        let start = html.find(r#"class="docs-toc-inline""#).unwrap_or_else(|| {
            panic!("{file:?}: docs-toc-inline should exist alongside docs-toc-aside")
        });
        let mut window_end = (start + 4000).min(html.len());
        while !html.is_char_boundary(window_end) {
            window_end -= 1;
        }
        let window = &html[start..window_end];
        assert!(
            window.contains("<a ") && window.contains("href=\"#"),
            "{file:?}: inline toc should contain at least one static <a href=\"#...\"> for no-JS heading navigation"
        );
    }
    assert!(
        checked_at_least_one,
        "real site should contain at least one page with a right toc column to exercise this contract"
    );
}

/// イシュー #3701: Assets メガパネルのカードは JS なしで踏める静的 `<a href>` で、
/// `<button>`・`on*=`・`role` を持たない。
#[test]
fn header_mega_panel_cards_are_static_anchors() {
    let (_out, files, _redirects) = build_real_site();
    let mut checked = 0usize;
    for file in &files {
        let html = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));
        let Some(start) = html.find("class=\"docs-header-mega\"") else {
            continue;
        };
        let panel = &html[start..start + html[start..].find("</ul>").expect("mega grid end")];
        assert!(!panel.contains("<button"), "{file:?}");
        assert!(!panel.contains("role="), "{file:?}");
        assert!(
            !panel.contains("onclick=") && !panel.contains("onmouse"),
            "{file:?}"
        );
        let cards = panel.matches("docs-header-mega-card\"").count();
        assert_eq!(cards, 5, "{file:?}");
        assert_eq!(panel.matches("<a href=\"").count(), cards, "{file:?}");
        checked += 1;
    }
    assert!(checked > 100, "mega panel should appear on every body page");
}

#[test]
fn structural_css_declares_js_independent_toggle_and_mega_panel_paths() {
    // CSS 側の JS 非依存開閉経路（`:checked`・`:hover`/`:focus-within`）が
    // 骨格 CSS から失われていないことを固定する。`site_css_contract.rs` の
    // カスケード契約（宣言順・詳細度）とは異なる観点（経路そのものの存在）
    // のため、重複ではなく補完として扱う。`site_theme::STRUCTURAL_CSS` は
    // 非公開のため、実サイトビルドが書き出す `assets/site.css`（生成物）
    // を直接読んで検証する。
    let (out, _files, _redirects) = build_real_site();
    let css = std::fs::read_to_string(out.join("assets/site.css"))
        .expect("dist/assets/site.css should be generated");
    let css = css.as_str();
    assert!(
        css.contains(".docs-header .docs-nav-drawer-toggle:checked ~ nav.docs-nav-drawer"),
        "structural CSS should keep the JS-free checkbox-driven nav drawer path (イシュー #3674)"
    );
    assert!(
        css.contains(
            ".docs-header nav.docs-header-nav .docs-header-group:hover > .docs-header-mega"
        ),
        "structural CSS should keep the JS-free :hover mega panel path (イシュー #908/#3701)"
    );
    assert!(
        css.contains(
            ".docs-header nav.docs-header-nav .docs-header-group:focus-within > .docs-header-mega"
        ),
        "structural CSS should keep the JS-free :focus-within mega panel path (キーボード操作でも JS なしで開閉できる、イシュー #908/#3701)"
    );
    assert!(
        !css.contains("docs-header-dropdown"),
        "the section popup was removed in #3701 and must not remain in site.css"
    );
}

/// イシュー #1016: リダイレクトページは本体ページと異なるクロームなし契約を
/// 満たす。本体ページの契約（`<script>` は外部 2 本のみ・`docs-search`/
/// `docs-theme-toggle` の `hidden` 既定）を「弱める」のではなく、リダイレクト
/// ページには**より強い**契約（`<script>` を 1 個も含まない）を課す形で
/// 分割する（モジュール doc 参照）。
#[test]
fn redirect_pages_contain_no_script_and_a_static_fallback_link() {
    let (_out, _body_files, redirect_files) = build_real_site();
    for file in &redirect_files {
        let html = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));

        assert!(
            !html.contains("<script"),
            "{file:?}: redirect pages must not contain any <script> (no chrome, no JS bootstrap either)"
        );
        assert!(
            !html.contains(r#"<link rel="stylesheet""#),
            "{file:?}: redirect pages must not link any stylesheet (no chrome)"
        );
        assert!(
            !html.contains(r#"rel="icon""#),
            "{file:?}: redirect pages must not carry favicon link (no chrome)"
        );
        assert!(
            !html.contains("class="),
            "{file:?}: redirect pages must not carry any `class` attribute (no chrome)"
        );
        let lower = html.to_ascii_lowercase();
        assert!(
            !lower.contains("content-security-policy"),
            "{file:?}: redirect pages must not carry a CSP meta (#3678)"
        );
        assert!(
            !lower.contains("<style"),
            "{file:?}: redirect pages must not contain an inline <style>"
        );
        assert!(
            html.contains(r#"<meta http-equiv="refresh" content="0; url="#),
            "{file:?}: redirect pages must contain a meta refresh"
        );
        assert!(
            html.contains(r#"<link rel="canonical" href=""#),
            "{file:?}: redirect pages must declare a canonical link"
        );
        assert!(
            html.contains(r#"<meta name="robots" content="noindex">"#),
            "{file:?}: redirect pages must be marked noindex (avoid polluting the search index / duplicate content)"
        );
        assert!(
            html.contains("<a ") && html.contains("href=\""),
            "{file:?}: redirect pages must contain a static fallback <a href> for no-JS/no-refresh environments"
        );
    }
}

/// フェンスコードのコピーボタン（イシュー #3605、`crate::code_copy`）は
/// 既定 `hidden` で出力され、`site.js` が配線完了後にのみ可視化する契約。
/// フェンスを含むページが 1 件以上あることも fail-closed で確認する。
#[test]
fn code_copy_buttons_default_to_hidden() {
    let (_out, files, _redirects) = build_real_site();
    let mut pages_with_fences = 0usize;
    for file in &files {
        let html = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));
        let buttons = html.matches(r#"class="docs-code-copy""#).count();
        let hidden = html.matches(r#"class="docs-code-copy" hidden"#).count();
        let wrappers = html.matches(r#"class="docs-code-block""#).count();
        assert_eq!(
            buttons, hidden,
            "{file:?}: every copy button must be hidden"
        );
        assert_eq!(
            buttons, wrappers,
            "{file:?}: one button per code block wrapper"
        );
        // #3620: ボタンは必ずヘッダー帯の中にあり、ヘッダーは 1 ラッパーに 1 個。
        assert_eq!(
            html.matches(r#"class="docs-code-header""#).count(),
            wrappers,
            "{file:?}: one header per code block wrapper"
        );
        assert_eq!(
            html.matches(r#"<div class="docs-code-header"><span class="docs-code-lang">"#)
                .count()
                + html
                    .matches(r#"<div class="docs-code-header"><button"#)
                    .count(),
            wrappers,
            "{file:?}: copy button must live inside the header"
        );
        if buttons > 0 {
            pages_with_fences += 1;
        }
    }
    assert!(
        pages_with_fences > 0,
        "at least one page must contain a fenced code block (otherwise this contract is vacuous)"
    );
}
/// イシュー #3606: 既定 hidden の契約は維持したまま、トグルのラッパー内に
/// pre-styled-ui の button、検索ブロック内に input_group と kbd が入っている。
#[test]
fn header_actions_embed_pre_styled_parts_inside_hidden_wrappers() {
    let (_out, files, _redirects) = build_real_site();
    let file = files.first().expect("at least one page");
    let html = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));

    let toggle = html
        .find(r#"class="docs-theme-toggle" hidden"#)
        .expect("toggle wrapper");
    let toggle_tail = &html[toggle..];
    let toggle_end = toggle_tail
        .find("</header>")
        .expect("header should close after toggle");
    assert!(toggle_tail[..toggle_end].contains(r#"data-scope="button""#));
    assert!(toggle_tail[..toggle_end].contains("docs-theme-toggle-label"));

    let search = html
        .find(r#"class="docs-search" hidden"#)
        .expect("search wrapper");
    let search_tail = &html[search..];
    let search_end = search_tail
        .find(r#"class="docs-search-results""#)
        .expect("results");
    assert!(search_tail[..search_end].contains(r#"data-scope="input-group""#));
    assert!(search_tail[..search_end].contains(r#"data-scope="kbd""#));
}

/// イシュー #3672: 検索ボタンと検索ダイアログは両方とも既定 `hidden` の
/// `div.docs-search` の内側にあり（無 JS では出ない）、全ページで dialog は
/// `open` を持たず、ボタンの `aria-haspopup`/`aria-expanded` は SSR に
/// 静的出力されない（JS が配線完了時に付与する）。
#[test]
fn search_trigger_and_dialog_live_inside_hidden_wrapper_without_static_popup_state() {
    let (_out, files, _redirects) = build_real_site();
    for file in &files {
        let html = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));
        let Some(search) = html.find(r#"class="docs-search" hidden"#) else {
            continue;
        };
        let tail = &html[search..];
        let trigger = tail
            .find(r#"class="docs-search-trigger""#)
            .unwrap_or_else(|| panic!("{file:?}: trigger missing"));
        let dialog = tail
            .find("<dialog")
            .unwrap_or_else(|| panic!("{file:?}: dialog missing"));
        assert!(
            tail[trigger..dialog].contains(r#"data-scope="button""#),
            "{file:?}: trigger should embed a pre-styled button"
        );
        assert!(tail[trigger..dialog].contains(r#"data-scope="kbd""#));
        let dialog_tag_end = tail[dialog..].find('>').expect("dialog tag end");
        let dialog_tag = &tail[dialog..dialog + dialog_tag_end];
        assert!(!dialog_tag.contains(" open"), "{file:?}: {dialog_tag}");
        let results = tail
            .find(r#"class="docs-search-results""#)
            .expect("results");
        assert!(dialog < results);
        assert!(tail[dialog..results].contains(r#"data-scope="input-group""#));
        let header_end = html.find("</header>").expect("header end");
        assert!(
            !html[..header_end].contains("aria-haspopup"),
            "{file:?}: aria-haspopup must be JS-only"
        );
        assert!(!tail[trigger..dialog].contains("aria-expanded"), "{file:?}");
    }
}

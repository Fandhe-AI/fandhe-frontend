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
//!    切り替わった。ドロップダウン自体は `:hover`/`:focus-within` により
//!    CSS のみで開閉する。リンク解決性自体は `build_site` 内蔵の linkcheck
//!    が fail-closed で保証済みであり、ここでは「JS なしで辿れる形が
//!    存在する」ことのみを固定する）
//! 4. 検索ブロック（`div.docs-search`）・テーマトグル（`.docs-theme-toggle`）
//!    が既定 `hidden`（JS 未実行時に操作できない UI を露出しない
//!    プログレッシブエンハンスメント契約）
//! 5. `assets/site.js` は `defer` 付き外部 `<script src>` 1 本のみで読み込まれる
//!    （`<head>` 先頭の FOUC 抑止インラインブートストラップ `INLINE_THEME_BOOTSTRAP`
//!    のみ唯一の例外として許容する。`try/catch` で `localStorage` 例外を
//!    握りつぶす自己完結スニペットで、JS 無効環境では単に実行されず既定
//!    テーマのまま表示されるだけで閲覧・ナビゲーションには影響しない）
//! 6. CSS 側に JS 非依存の開閉経路（`.docs-sidebar-toggle:checked`・
//!    `.docs-header-group:hover`/`:focus-within`）が存在する
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

use fandhe_frontend_docs_site::build::build_site;
use fandhe_frontend_docs_site::script::INLINE_THEME_BOOTSTRAP;
use fandhe_frontend_docs_site::{nav, redirect};

/// 統合テストのスクラッチ基点。`CARGO_TARGET_TMPDIR` は cargo が統合テスト
/// バイナリの**コンパイル時のみ**設定する（Cargo Book）ため `env!` で確定し、
/// 実行時 env による明示上書きのみ許容する。`/tmp` へは一切フォールバック
/// しない（イシュー #637 の事実誤認の再発防止、`tests/site_build.rs` と
/// 同一パターン）。
fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = std::fs::create_dir_all(&root);
    root
}

/// テスト専用の一時出力ディレクトリ。`tests/site_build.rs::TempDir` と
/// 同方針（外部クレート `tempfile` を追加しない、REQ-3）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = scratch_root().join(format!(
            "fandhe-frontend-docs-site-no-js-{tag}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp dir for no_js_contract.rs test");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

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

/// 実サイトビルドを 1 回だけ実行し、生成ページを「本体ページ」と
/// 「リダイレクトページ」（イシュー #1016）に分割して共有する。
/// `cargo test` 内で複数アサーションが同じビルド結果を参照するための
/// ヘルパー（毎テストで再ビルドすると `linkcheck` 込みで冗長）。
fn build_real_site() -> (TempDir, Vec<PathBuf>, Vec<PathBuf>) {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve repository root");
    let out = TempDir::new("no-js");
    build_site(&repo_root, &out.0).expect("real site/nav.toml should build cleanly");

    let mut files = Vec::new();
    collect_html_files(&out.0, &mut files);
    assert!(
        !files.is_empty(),
        "real site build should emit at least one HTML page"
    );

    let redirect_set = expected_redirect_files(&repo_root, &out.0);
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

#[test]
fn site_js_is_loaded_as_single_deferred_external_script() {
    let (_out, files, _redirects) = build_real_site();
    // 唯一許容するインラインスクリプトは `<head>` 先頭の FOUC 抑止
    // ブートストラップ（`crate::script::INLINE_THEME_BOOTSTRAP`）のみ。
    // `try/catch` で `localStorage` 例外を握りつぶす自己完結スニペットで
    // あり、JS 無効環境では単に実行されず既定テーマのまま表示される
    // （閲覧・ナビゲーションに影響しない）ため no-JS 契約の例外として許す。
    let allowed_inline = format!("<script>{INLINE_THEME_BOOTSTRAP}</script>");
    for file in &files {
        let html = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));

        // `<script>` 開始タグの出現は許容されたブートストラップの 1 個のみ。
        let script_tag_count = html.matches("<script>").count();
        assert_eq!(
            script_tag_count, 1,
            "{file:?} should contain exactly one inline <script> tag (the allowed FOUC bootstrap, no-JS 契約は他のインラインスクリプトを許容しない)"
        );
        assert!(
            html.contains(&allowed_inline),
            "{file:?}: the sole inline <script> must be the theme bootstrap snippet verbatim"
        );

        assert!(
            html.contains(r#"src="/fandhe-frontend/assets/site.js" defer="""#),
            "{file:?} should load assets/site.js via a single deferred external <script src> (site_build.rs の共通契約と同一文字列)"
        );
        // <script src> の出現回数はちょうど 1 本であること。
        let script_src_count = html.matches("<script src=").count();
        assert_eq!(
            script_src_count, 1,
            "{file:?} should reference exactly one external <script src> tag"
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

#[test]
fn structural_css_declares_js_independent_toggle_and_dropdown_paths() {
    // CSS 側の JS 非依存開閉経路（`:checked`・`:hover`/`:focus-within`）が
    // 骨格 CSS から失われていないことを固定する。`site_css_contract.rs` の
    // カスケード契約（宣言順・詳細度）とは異なる観点（経路そのものの存在）
    // のため、重複ではなく補完として扱う。`site_theme::STRUCTURAL_CSS` は
    // 非公開のため、実サイトビルドが書き出す `assets/site.css`（生成物）
    // を直接読んで検証する。
    let (out, _files, _redirects) = build_real_site();
    let css = std::fs::read_to_string(out.0.join("assets/site.css"))
        .expect("dist/assets/site.css should be generated");
    let css = css.as_str();
    assert!(
        css.contains(".docs-sidebar-toggle:checked ~ nav.sidebar"),
        "structural CSS should keep the JS-free checkbox-driven sidebar toggle path (イシュー #916)"
    );
    assert!(
        css.contains(".docs-header-group:hover > .docs-header-dropdown")
            || css.contains(
                ".docs-header nav.docs-header-nav .docs-header-group:hover > .docs-header-dropdown"
            ),
        "structural CSS should keep the JS-free :hover dropdown path (イシュー #908)"
    );
    assert!(
        css.contains(".docs-header-group:focus-within > .docs-header-dropdown")
            || css.contains(".docs-header nav.docs-header-nav .docs-header-group:focus-within > .docs-header-dropdown"),
        "structural CSS should keep the JS-free :focus-within dropdown path (キーボード操作でも JS なしで開閉できる、イシュー #908)"
    );
}

/// イシュー #1016: リダイレクトページは本体ページと異なるクロームなし契約を
/// 満たす。本体ページの契約（`<script>` 正確に 1 個・`docs-search`/
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
            !html.contains("class="),
            "{file:?}: redirect pages must not carry any `class` attribute (no chrome)"
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

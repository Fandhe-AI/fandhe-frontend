//! イシュー #1155（clipboard / skip-nav の Themes 部品ページ充填）が供給する
//! `crate::component_specs::interactive_utilities::SPECS` レジストリの契約
//! テスト。
//!
//! `crates/docs-site/tests/component_pages.rs` / `tests/site_showcase.rs` は
//! 並列実行される他イシュー（#1154 等）も触り得る共有ファイルのため変更
//! しない方針（`crates/docs-site/tests/component_page_specs_948.rs` と同じ
//! per-issue テストファイル方式）。本ファイルは本イシュー担当の 2 ページ
//! （clipboard / skip-nav）のみを検証する。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use fandhe_frontend_docs_site::build::build_site;
use fandhe_frontend_docs_site::component_page::generated_content;
use fandhe_frontend_docs_site::component_specs::interactive_utilities::SPECS;
use fandhe_frontend_docs_site::showcase;

/// `SPECS` 内でパスが重複していないこと（`component_page_specs_948.rs` の
/// 同名テストと同じ回帰防止意図）。
#[test]
fn specs_has_no_duplicate_paths() {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut duplicates: Vec<&str> = Vec::new();
    for (path, _) in SPECS {
        if !seen.insert(path) {
            duplicates.push(path);
        }
    }
    assert!(
        duplicates.is_empty(),
        "component_specs::interactive_utilities::SPECS has duplicate path(s): {duplicates:?}"
    );
}

/// ちょうど 2 パス（`/themes/clipboard/` `/themes/skip-nav/`）が登録されて
/// いること（過不足の検知）。
#[test]
fn specs_registers_exactly_clipboard_and_skip_nav() {
    let expected: BTreeSet<&str> = ["/themes/clipboard/", "/themes/skip-nav/"]
        .into_iter()
        .collect();
    let actual: BTreeSet<&str> = SPECS.iter().map(|(path, _)| *path).collect();
    assert_eq!(actual, expected);
}

/// clipboard / skip-nav は `showcase::COMPONENT_PAGES` に未登録（Demo
/// フォールバック経由でのみ到達する、モジュール doc 参照）。誤って
/// `showcase.rs` 側へも登録すると `ComponentPageSpec::demo` が到達不能な
/// デッドコードになる事故を防ぐ（`component_page_specs_948.rs` の
/// `specs_paths_are_registered_component_pages` の逆方向契約）。
#[test]
fn specs_paths_are_not_registered_in_showcase_component_pages() {
    let registered: BTreeSet<&str> = showcase::component_page_paths().collect();
    for (path, _) in SPECS {
        assert!(
            !registered.contains(path),
            "component_specs::interactive_utilities::SPECS registers {path}, which IS in \
             showcase::component_page_paths(); Demo フォールバック（ComponentPageSpec::demo）\
             は showcase 側未登録ページ専用であり、二重登録は showcase 側が優先されて \
             demo フィールドがデッドコード化する"
        );
    }
}

/// HTML から `h2` 見出しテキストを出現順に抽出する（`component_pages.rs`
/// の `h2_texts` と同型のローカルヘルパ、共有ファイルへは触れない方針）。
fn h2_texts(html: &str) -> Vec<String> {
    let open = "<h2>";
    let close = "</h2>";
    let mut out = Vec::new();
    let mut idx = 0;
    while let Some(rel) = html[idx..].find(open) {
        let start = idx + rel + open.len();
        let Some(end_rel) = html[start..].find(close) else {
            break;
        };
        out.push(html[start..start + end_rel].to_string());
        idx = start + end_rel + close.len();
    }
    out
}

const CANONICAL_SECTIONS: &[&str] = &[
    "Demo",
    "Features",
    "Anatomy",
    "API Reference",
    "Examples",
    "Accessibility",
];

/// 2 ページとも `generated_content` が `Some` を返し、canonical 6 節の h2
/// が順序どおりすべて揃うこと（受け入れ条件 1 の機械固定）。
#[test]
fn clipboard_and_skip_nav_render_all_six_canonical_sections() {
    use fandhe_frontend_core::render;

    for (path, _) in SPECS {
        let node = generated_content(path)
            .unwrap_or_else(|| panic!("generated_content({path}) should be Some"));
        let html = render(&node);
        let headings = h2_texts(&html);
        assert_eq!(
            headings, CANONICAL_SECTIONS,
            "{path} should render all six canonical sections in order, got: {headings:?}"
        );
    }
}

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する
/// （`tests/site_showcase.rs` と同じ規約）。
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo_root should resolve from CARGO_MANIFEST_DIR")
}

/// 統合テストのスクラッチ基点（`tests/site_showcase.rs` と同一パターン、
/// イシュー #637/#658。`/tmp` へはフォールバックしない）。
fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = std::fs::create_dir_all(&root);
    root
}

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = scratch_root().join(format!(
            "fandhe-frontend-docs-site-component-specs-1155-{tag}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp dir for component_specs_1155.rs test");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn read_component_page(out: &Path, page_rel: &str) -> String {
    let page_path = out.join(page_rel).join("index.html");
    std::fs::read_to_string(&page_path)
        .unwrap_or_else(|e| panic!("component page should be written at {page_path:?}: {e}"))
}

/// 実サイトビルドでの CSS 網羅検証（`tests/site_showcase.rs::forms_demo_fallback_pages_ship_scoped_css`
/// と同型、#979 Bugbot 指摘の再発防止パターン）: `themes/clipboard/index.html`
/// に出現する `data-scope="clipboard"` へ対応する CSS セレクタが
/// `assets/pre-styled-ui.css` に実在すること（HTML → CSS の片方向網羅）。
#[test]
fn clipboard_page_ships_scoped_css() {
    let out = TempDir::new("clipboard-css");
    build_site(&repo_root(), &out.0).expect("real site should build");

    let html = read_component_page(&out.0, "themes/clipboard");
    assert!(html.contains(r#"data-scope="clipboard""#));

    let css_path = out.0.join(showcase::STYLESHEET_REL_PATH);
    let css = std::fs::read_to_string(&css_path).unwrap();
    assert!(
        css.contains(r#"[data-scope="clipboard"]"#),
        "themes/clipboard renders data-scope=\"clipboard\" but showcase::stylesheet() does not \
         declare a matching [data-scope=\"clipboard\"] selector in assets/pre-styled-ui.css"
    );
}

/// `themes/skip-nav/index.html` の Demo に `data-scope="skip-nav"` が出現し、
/// 全ページ共通 `assets/skip-nav.css`（`crate::skip_nav::stylesheet`、
/// `showcase::stylesheet()` とは別出荷経路）に対応するセレクタが実在する
/// こと。加えて、ページ内で `id="fandhe-skip-nav"`（`DEFAULT_ID`、レイアウト
/// 実適用分）の出現がちょうど 1 回であることを固定する — Demo がカスタム
/// id を使う契約の回帰ガード（id 重複は HTML 仕様違反かつ SkipNav の
/// `href="#…"` 関連付け破壊を招く）。
#[test]
fn skip_nav_page_ships_scoped_css_and_demo_uses_a_custom_id() {
    let out = TempDir::new("skip-nav-css");
    build_site(&repo_root(), &out.0).expect("real site should build");

    let html = read_component_page(&out.0, "themes/skip-nav");
    assert!(html.contains(r#"data-scope="skip-nav""#));

    let skip_nav_css_path = out.0.join("assets/skip-nav.css");
    let skip_nav_css = std::fs::read_to_string(&skip_nav_css_path).unwrap();
    assert!(
        skip_nav_css.contains(r#"[data-scope="skip-nav"][data-part="link"]"#),
        "assets/skip-nav.css should declare a [data-scope=\"skip-nav\"][data-part=\"link\"] \
         selector (shipped site-wide by crate::skip_nav::stylesheet, unrelated to this page's \
         own build)"
    );

    let default_id_occurrences = html.matches(r#"id="fandhe-skip-nav""#).count();
    assert_eq!(
        default_id_occurrences, 1,
        "themes/skip-nav should contain exactly one id=\"fandhe-skip-nav\" (the site-wide \
         layout's SkipNav, from crate::layout::docs_page_with_assets); the page's own Demo \
         must use a distinct custom id to avoid id duplication, got {default_id_occurrences} \
         occurrence(s)"
    );
}

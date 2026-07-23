//! UI コンポーネントショーケースページ（`crate::showcase`、pre-styled-ui
//! 統合）の実サイトビルド検証。
//!
//! `site/nav.toml` の実宣言（`/components/pre-styled-ui/`）と
//! `showcase::PAGE_PATH` の一致、生成 HTML への styled 部品マークアップの
//! 埋め込み、専用 CSS（`assets/pre-styled-ui.css`）の書き出し・`<link>` 参照
//! を end-to-end で固定する。`tests/site_build.rs` の実サイトビルド検証と
//! 同じくリポジトリルートを `--root` 相当として `build_site` を直接呼ぶ。

use std::path::{Path, PathBuf};

use fandhe_frontend_docs_site::build::build_site;
use fandhe_frontend_docs_site::showcase;

/// 統合テストのスクラッチ基点（`tests/site_build.rs` と同一パターン、
/// イシュー #637/#658。`/tmp` へはフォールバックしない）。
fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = std::fs::create_dir_all(&root);
    root
}

/// テスト専用の一時出力ディレクトリ（外部クレート `tempfile` を追加しない、
/// REQ-3。`tests/site_build.rs` の `TempDir` と同方針）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = scratch_root().join(format!(
            "fandhe-frontend-docs-site-showcase-{tag}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp dir for site_showcase.rs test");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する
/// （`tests/site_css_contract.rs` と同じ規約）。
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo_root should resolve from CARGO_MANIFEST_DIR")
}

#[test]
fn real_site_build_emits_showcase_page_and_dedicated_css() {
    let out = TempDir::new("real-site");
    let report = build_site(&repo_root(), &out.0).expect("real site should build");

    // showcase ページの HTML（page.path = showcase::PAGE_PATH の出力先）。
    let page_rel = showcase::PAGE_PATH.trim_start_matches('/');
    let page_path = out.0.join(page_rel).join("index.html");
    assert!(
        page_path.exists(),
        "showcase page should be written at {page_path:?} (site/nav.toml と showcase::PAGE_PATH の乖離を疑う)"
    );

    let html = std::fs::read_to_string(&page_path).unwrap();
    // Markdown 導入文（site/components-pre-styled-ui.md）と Rust 生成
    // コンテンツ（styled 部品）が同一ページに合成されている。
    assert!(html.contains("pre-styled-ui コンポーネントショーケース"));
    for scope in [
        "button",
        "badge",
        "spinner",
        "alert",
        "card",
        "tabs",
        "accordion",
    ] {
        assert!(
            html.contains(&format!(r#"data-scope="{scope}""#)),
            "missing data-scope={scope} in showcase page"
        );
    }
    // サイト骨格 CSS と showcase 専用 CSS の両方を <link> 参照する
    // （base_path = /fandhe-frontend を考慮した href）。
    assert!(html.contains(r#"href="/fandhe-frontend/assets/site.css""#));
    assert!(html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#));

    // 専用 CSS が書き出され、テーマトークン + recipe セレクタを含む。
    let css_path = out.0.join(showcase::STYLESHEET_REL_PATH);
    assert!(css_path.exists());
    assert!(report.assets.iter().any(|a| a == &css_path));
    let css = std::fs::read_to_string(&css_path).unwrap();
    assert!(css.contains("--fandhe-color-"));
    assert!(css.contains(".fd-button--variant-solid"));
    assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"]"#));
    // site.css の `.docs-content h3` が Accordion anatomy の h3 へ漏れるのを
    // 遮断する見出しリセットが専用 CSS 側に含まれる（site.css は変更しない
    // 分離契約のまま showcase 側で上書きする。Bugbot 指摘の回帰防止）。
    assert!(css.contains(r#".pre-styled-showcase [data-scope="accordion"] h3"#));
    assert!(!css.contains('<'));

    // ページ内目次（docs-toc）にはセクション見出しのみが載り、コンポーネント
    // anatomy 内の見出し（Accordion trigger の h3・Card title の h3）は
    // 混入しない（`layout::with_heading_anchors` の data-scope 部分木除外）。
    let toc = html
        .split(r#"<nav class="docs-toc">"#)
        .nth(1)
        .and_then(|rest| rest.split("</nav>").next())
        .expect("showcase page should have a docs-toc nav");
    assert!(toc.contains(">Accordion<"));
    assert!(toc.contains(">Card<"));
    assert!(
        !toc.contains("pre-styled-ui とは何ですか"),
        "accordion trigger heading must not leak into TOC: {toc}"
    );
    assert!(
        !toc.contains(">Elevated<"),
        "card title heading must not leak into TOC: {toc}"
    );
}

#[test]
fn non_showcase_pages_do_not_reference_showcase_css() {
    let out = TempDir::new("no-extra-link");
    build_site(&repo_root(), &out.0).expect("real site should build");

    // Markdown のみのページには追加 <link> を差し込まない（サイト骨格の
    // カスケードへ影響させない分離契約）。
    let index_html = std::fs::read_to_string(out.0.join("index.html")).unwrap();
    assert!(!index_html.contains("pre-styled-ui.css"));
}

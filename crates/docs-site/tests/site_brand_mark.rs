//! イシュー #3722: `[site].brand_mark` / `brand_color` が `assets/favicon.svg` と
//! ヘッダーのインライン SVG へ同じ値で反映されることの結合テスト。
//!
//! 実サイトではなく最小フィクスチャを `CARGO_TARGET_TMPDIR` 配下へ書き出してビルドする
//! （`/tmp` へはフォールバックしない。`site_build.rs` の `TempDir` と同方針）。

use std::path::{Path, PathBuf};

use fandhe_frontend_docs_site::build::build_site_with;
use fandhe_frontend_docs_site::favicon;
use fandhe_frontend_docs_site::page_sections::EMPTY_REGISTRY;

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "fandhe-frontend-docs-site-brand-mark-{tag}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp dir");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `extra` を `[site]` へ足した最小サイトを作り、出力ディレクトリを返す。
fn build(tag: &str, extra: &str) -> Result<(TempDir, PathBuf), String> {
    let root = TempDir::new(tag);
    std::fs::create_dir_all(root.0.join("site")).unwrap();
    std::fs::write(
        root.0.join("site/nav.toml"),
        format!(
            "[site]\ntitle = \"Fixture\"\nbase_path = \"\"\n{extra}\n[[section]]\ntitle = \"Guide\"\nindex_path = \"/\"\n\n[[section.page]]\ntitle = \"Home\"\nsource = \"site/index.md\"\npath = \"/\"\n"
        ),
    )
    .unwrap();
    std::fs::write(root.0.join("site/index.md"), "# Home\n\nbody\n").unwrap();
    let out = root.0.join("out");
    build_site_with(&root.0, &out, &EMPTY_REGISTRY).map_err(|e| e.to_string())?;
    Ok((root, out))
}

fn read(out: &Path, rel: &str) -> String {
    std::fs::read_to_string(out.join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// ヘッダーの `a.docs-brand` 内インライン SVG（`<svg` から最初の `</svg>` まで）を抜き出す。
fn header_svg(html: &str) -> &str {
    let brand = html.find("class=\"docs-brand\"").expect("docs-brand");
    let start = brand + html[brand..].find("<svg").expect("inline svg");
    let end = start + html[start..].find("</svg>").expect("svg end") + "</svg>".len();
    &html[start..end]
}

#[test]
fn custom_mark_is_identical_in_favicon_and_header() {
    let (_t, out) = build(
        "custom",
        "brand = \"Acme\"\nbrand_mark = \"A\"\nbrand_color = \"#AB12cd\"\n",
    )
    .unwrap();
    let favicon_svg = read(&out, "assets/favicon.svg");
    assert!(favicon_svg.contains("fill=\"#AB12cd\""));
    assert!(favicon_svg.contains(">A</text>"));
    assert!(favicon_svg.contains("aria-label=\"Acme\""));
    for page in ["index.html", "404.html"] {
        assert_eq!(header_svg(&read(&out, page)), favicon_svg, "{page}");
    }
}

#[test]
fn defaults_match_legacy_favicon() {
    let (_t, out) = build("default", "").unwrap();
    let favicon_svg = read(&out, "assets/favicon.svg");
    assert_eq!(favicon_svg, favicon::svg());
    assert_eq!(header_svg(&read(&out, "index.html")), favicon_svg);
}

#[test]
fn color_only_keeps_f_glyph() {
    let (_t, out) = build("color-only", "brand_color = \"#112233\"\n").unwrap();
    let favicon_svg = read(&out, "assets/favicon.svg");
    assert!(!favicon_svg.contains("<text"));
    assert!(favicon_svg.contains("rx=\"7\" fill=\"#112233\""));
    assert_eq!(header_svg(&read(&out, "index.html")), favicon_svg);
}

#[test]
fn mark_only_keeps_default_label_and_color() {
    let (_t, out) = build("mark-only", "brand_mark = \"Q\"\n").unwrap();
    let favicon_svg = read(&out, "assets/favicon.svg");
    assert!(favicon_svg.contains("aria-label=\"fandhe-frontend\""));
    assert!(favicon_svg.contains("rx=\"7\" fill=\"#3182ce\""));
    assert!(favicon_svg.contains(">Q</text>"));
}

#[test]
fn custom_mark_adds_no_style_script_or_external_reference() {
    let (_t, out) = build(
        "no-style",
        "brand_mark = \"A\"\nbrand_color = \"#AB12cd\"\n",
    )
    .unwrap();
    let favicon_svg = read(&out, "assets/favicon.svg");
    for forbidden in [
        "<script",
        "<style",
        "style=",
        "href",
        "url(",
        "foreignObject",
    ] {
        assert!(!favicon_svg.contains(forbidden), "{forbidden}");
    }
    let index = read(&out, "index.html");
    assert!(!index.contains("<style"));
    assert!(index.contains("Content-Security-Policy"));
}

#[test]
fn invalid_values_fail_the_build() {
    assert!(build("bad-color", "brand_color = \"#fff\"\n").is_err());
    assert!(build("bad-mark", "brand_mark = \"ab\"\n").is_err());
}

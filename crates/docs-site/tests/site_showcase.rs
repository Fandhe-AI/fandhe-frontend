//! 部品ページ（`crate::showcase` / `crate::component_page`、pre-styled-ui
//! 統合）の実サイトビルド検証。
//!
//! イシュー #941〜#943 で「1 ページ = pre-styled-ui の公開部品 1 件」へ
//! 分解されたため、本テストは `/components/pre-styled-ui/`（索引ページ、
//! `showcase::PAGE_PATH`）ではなく個別部品ページ（`/components/<kebab>/`）
//! を対象に、生成 HTML への styled 部品マークアップの埋め込み・専用 CSS
//! （`assets/pre-styled-ui.css`）の書き出し・`<link>` 参照を end-to-end で
//! 固定する。`tests/site_build.rs` の実サイトビルド検証と同じくリポジトリ
//! ルートを `--root` 相当として `build_site` を直接呼ぶ。

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

/// 部品ページの HTML を読み出す（`page_rel` は `showcase::COMPONENT_PAGES`
/// / `nav.toml` の `page.path` から先頭 `/` を除いたもの）。
fn read_component_page(out: &Path, page_rel: &str) -> String {
    let page_path = out.join(page_rel).join("index.html");
    std::fs::read_to_string(&page_path)
        .unwrap_or_else(|e| panic!("component page should be written at {page_path:?}: {e}"))
}

#[test]
fn real_site_build_emits_component_pages_and_dedicated_css() {
    let out = TempDir::new("real-site");
    let report = build_site(&repo_root(), &out.0).expect("real site should build");

    // 基本部品（Button）: Demo 節の styled 部品マークアップ・CSS 配線を
    // 固定する。
    let button_html = read_component_page(&out.0, "components/button");
    assert!(button_html.contains(r#"data-scope="button""#));
    assert!(button_html.contains(">Demo<"));
    // サイト骨格 CSS と部品ページ専用 CSS の両方を <link> 参照する
    // （base_path = /fandhe-frontend を考慮した href）。
    assert!(button_html.contains(r#"href="/fandhe-frontend/assets/site.css""#));
    assert!(button_html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#));

    // 専用 CSS が書き出され、テーマトークン + recipe セレクタを含む
    // （全部品ページが共有する単一の CSS 束、showcase::stylesheet 参照）。
    let css_path = out.0.join(showcase::STYLESHEET_REL_PATH);
    assert!(css_path.exists());
    assert!(report.assets.iter().any(|a| a == &css_path));
    let css = std::fs::read_to_string(&css_path).unwrap();
    assert!(css.contains("--fandhe-color-"));
    assert!(css.contains(".fd-button--variant-solid"));
    assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"]"#));
    assert!(css.contains(r#"[data-scope="dialog"][data-part="content"]"#));
    assert!(css.contains(r#"[data-scope="switch"][data-part="control"]"#));
    assert!(css.contains(".fd-avatar--size-md"));
    // site.css の `.docs-content h3` が Accordion anatomy の h3 へ漏れるのを
    // 遮断する見出しリセットが専用 CSS 側に含まれる（site.css は変更しない
    // 分離契約のまま部品ページ側で上書きする。Bugbot 指摘の回帰防止）。
    assert!(css.contains(r#".pre-styled-showcase [data-scope="accordion"] h3"#));
    assert!(!css.contains('<'));

    // イシュー #691 の目次漏れ回帰: `layout::with_heading_anchors` の
    // data-scope 部分木除外により、コンポーネント anatomy 内の見出し
    // （Accordion trigger の h3・Card title の h3・Dialog/Popover の
    // title h2）はページ内目次（docs-toc）へ混入しない。各部品ページで
    // 個別に固定する（イシュー #943 でページ単位分解済み、旧集約ページ
    // での一括検証から移設）。イシュー #950 で `nav.docs-toc` へ
    // `aria-labelledby` が付いたため完全一致で切り出す。
    let toc_of = |html: &str| -> String {
        html.split(r#"<nav class="docs-toc" aria-labelledby="docs-toc-heading">"#)
            .nth(1)
            .and_then(|rest| rest.split("</nav>").next())
            .expect("component page should have a docs-toc nav")
            .to_string()
    };

    let accordion_html = read_component_page(&out.0, "components/accordion");
    let accordion_toc = toc_of(&accordion_html);
    assert!(accordion_toc.contains(">Demo<"));
    assert!(
        !accordion_toc.contains("pre-styled-ui とは何ですか"),
        "accordion trigger heading must not leak into TOC: {accordion_toc}"
    );

    let card_html = read_component_page(&out.0, "components/card");
    let card_toc = toc_of(&card_html);
    assert!(
        !card_toc.contains(">Elevated<"),
        "card title heading must not leak into TOC: {card_toc}"
    );

    let dialog_html = read_component_page(&out.0, "components/dialog");
    let dialog_toc = toc_of(&dialog_html);
    assert!(
        !dialog_toc.contains("Confirm action"),
        "dialog title heading must not leak into TOC: {dialog_toc}"
    );

    let popover_html = read_component_page(&out.0, "components/popover");
    let popover_toc = toc_of(&popover_html);
    assert!(
        !popover_toc.contains("About this feature"),
        "popover title heading must not leak into TOC: {popover_toc}"
    );
}

/// イシュー #945 の Bugbot 指摘（PR #979）の回帰テスト。
///
/// `component_specs::forms` の Demo フォールバック 6 部品（Angle Slider /
/// Image Cropper / Pin Input / Signature Pad / Toggle / Toggle Group、
/// `showcase.rs` に節を持たない）は、実 styled 部品マークアップ（
/// `data-scope="..."`）をページへ埋め込む一方、`showcase::stylesheet()` に
/// 対応する `push_css` 呼び出しが欠けていたため無 CSS のまま出荷されて
/// いた。本テストは各ページの HTML に出現する `data-scope="..."` 値が、
/// 全部品ページ共有の専用 CSS（`assets/pre-styled-ui.css`）に対応する
/// `[data-scope="..."]` セレクタとして存在することを固定する
/// （HTML → CSS の片方向網羅、`tests/site_css_contract.rs` の層 2 と同型）。
#[test]
fn forms_demo_fallback_pages_ship_scoped_css() {
    let out = TempDir::new("forms-demo-fallback-css");
    build_site(&repo_root(), &out.0).expect("real site should build");

    let css_path = out.0.join(showcase::STYLESHEET_REL_PATH);
    let css = std::fs::read_to_string(&css_path).unwrap();

    for (page_rel, expected_scope) in [
        ("components/angle-slider", "angle-slider"),
        ("components/image-cropper", "image-cropper"),
        ("components/pin-input", "pin-input"),
        ("components/signature-pad", "signature-pad"),
        ("components/toggle", "toggle"),
        ("components/toggle-group", "toggle-group"),
    ] {
        let html = read_component_page(&out.0, page_rel);
        let marker = format!(r#"data-scope="{expected_scope}""#);
        assert!(
            html.contains(&marker),
            "{page_rel} should render `{marker}` (Demo fallback markup, forms.rs::demo_*)"
        );

        let selector = format!(r#"[data-scope="{expected_scope}"]"#);
        assert!(
            css.contains(&selector),
            "{page_rel} renders `{marker}` but showcase::stylesheet() does not declare \
             `{selector}` in assets/pre-styled-ui.css \
             (showcase.rs must push_css the matching pre-styled-ui module)"
        );
    }
}

#[test]
fn non_showcase_pages_do_not_reference_showcase_css() {
    let out = TempDir::new("no-extra-link");
    build_site(&repo_root(), &out.0).expect("real site should build");

    // Markdown のみのページには追加 <link> を差し込まない（サイト骨格の
    // カスケードへ影響させない分離契約）。
    let index_html = std::fs::read_to_string(out.0.join("index.html")).unwrap();
    assert!(!index_html.contains("pre-styled-ui.css"));

    // イシュー #943: `/components/pre-styled-ui/`（索引ページ）は Rust
    // 生成コンテンツを持たない（showcase::generated_content が None を
    // 返す）ため、集約レンダリングが撤去されたことをここで固定する。
    // 索引ページに専用 CSS の <link> も部品の data-scope マークアップも
    // 出現しないことが「集約を撤去した」ことの直接証拠になる。
    // 索引ページの本文（`site/components-pre-styled-ui.md`）は説明文の中で
    // `assets/pre-styled-ui.css` という語句自体には言及するため、
    // `<link ... href="...">` の実配線有無で判定する
    // （素の部分文字列一致だと本文中の言及と誤検知が区別できない）。
    let index_page_rel = showcase::PAGE_PATH.trim_start_matches('/');
    let component_index_html = read_component_page(&out.0, index_page_rel);
    assert!(!component_index_html.contains(r#"href="/fandhe-frontend/assets/pre-styled-ui.css""#));
    assert!(!component_index_html.contains(r#"data-scope="button""#));
    assert!(!component_index_html.contains(r#"class="pre-styled-showcase""#));
}

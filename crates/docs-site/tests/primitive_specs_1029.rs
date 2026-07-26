//! Primitives 部品ページ原稿の充填（イシュー #1029、Data Display /
//! Utilities 10 部品）を検証する統合テスト。
//!
//! 対象は本カテゴリの 10 パスのみ（`tests/component_pages.rs`/
//! `tests/primitive_showcase.rs` は共有ファイルのため触らず、本イシュー
//! 専用の検証は本ファイルへ切り出す。`tests/primitive_specs_overlay_disclosure.rs`
//! （#1027）と同型の構成）。

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::{
    generated_content, render_component_page, ArgRow, AriaRow, ComponentPageSpec, Layer,
};
use fandhe_frontend_docs_site::primitive_specs;
use fandhe_frontend_docs_site::primitives_catalog::{self, PrimitiveCategory};

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する
/// （`tests/component_pages.rs`/`tests/primitive_showcase.rs` と同じ規約）。
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo_root should resolve from CARGO_MANIFEST_DIR")
}

/// 本イシューの対象 10 パス（`site/primitives/*.md` のファイル名 = path 末尾）。
/// `primitives_catalog::PrimitiveCategory::DataDisplayUtilities` のカタログ順
/// と一致させる（[`data_display_utilities_paths_match_catalog`] で突合する）。
const PATHS: &[&str] = &[
    "/primitives/avatar/",
    "/primitives/carousel/",
    "/primitives/json-tree-view/",
    "/primitives/scroll-area/",
    "/primitives/skip-nav/",
    "/primitives/splitter/",
    "/primitives/steps/",
    "/primitives/tour/",
    "/primitives/tree-view/",
    "/primitives/visually-hidden/",
];

/// HTML 文字列から `h2` 見出しテキストを出現順に抽出する
/// （`tests/component_pages.rs::h2_texts` と同型の簡易抽出。本テストの
/// 入力は本モジュール自身が組み立てた既知の構造のみであり、汎用 HTML
/// パーサは不要）。
fn h2_texts(html: &str) -> Vec<String> {
    extract_heading_texts(html, "h2")
}

fn extract_heading_texts(html: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find(&open) {
        let after_open = &rest[start + open.len()..];
        let Some(end) = after_open.find(&close) else {
            break;
        };
        out.push(after_open[..end].to_string());
        rest = &after_open[end + close.len()..];
    }
    out
}

/// 受け入れ条件 2 の前提: 本ファイルの [`PATHS`] が
/// `primitives_catalog::entries_in(PrimitiveCategory::DataDisplayUtilities)`
/// から導出した path 集合と完全一致すること（第 2 のハードコード一覧が
/// 台帳とドリフトしないことを固定する）。
#[test]
fn data_display_utilities_paths_match_catalog() {
    let from_catalog: Vec<&str> =
        primitives_catalog::entries_in(PrimitiveCategory::DataDisplayUtilities)
            .map(|entry| entry.path)
            .collect();
    assert_eq!(
        from_catalog, PATHS,
        "PATHS must match primitives_catalog order/content for DataDisplayUtilities exactly"
    );
}

/// イシュー #1029 の受け入れ条件 1: 10 ページすべてが Demo → Features →
/// Anatomy → API Reference → Examples → Accessibility の 6 節を**順序込み**
/// で持つこと（`tests/component_pages.rs::tab_nav_page_renders_all_six_canonical_sections`
/// と同型）。
#[test]
fn data_display_utilities_primitive_pages_render_all_six_sections() {
    const EXPECTED: &[&str] = &[
        "Demo",
        "Features",
        "Anatomy",
        "API Reference",
        "Examples",
        "Accessibility",
    ];
    for path in PATHS {
        let page = generated_content(path)
            .unwrap_or_else(|| panic!("{path} should have generated content"));
        let html = render(&page);
        let headings = h2_texts(&html);
        assert_eq!(
            headings, EXPECTED,
            "{path} should render all six canonical sections in order, got {headings:?}"
        );
    }
}

/// 受け入れ条件 1 の実効固定: `API Reference` の H2 は機械導出の
/// `Data Attributes` 表だけでも出現しうるため、H2 の存在確認だけでは
/// `arguments: &[]` を見逃す。`<h3>Arguments</h3>` の実在を直接検査する。
/// 併せて Primitives 層の恒常省略契約（`CSS Variables` が出現しないこと）
/// も固定する。
///
/// `Data Attributes` 表は Demo（`primitive_showcase::data_display_utilities`）
/// が実際に出力する `data-scope`/`data-part` 以外の `data-*` 属性から
/// 機械導出されるため（`component_page.rs::collect_data_attrs_from_tree`）、
/// 本カテゴリの原稿データの充填有無とは独立した Demo 側の特性である。
/// skip-nav（`href`/`id`/`tabindex` のみで `data-*` を追加出力しない、
/// `skip_nav.rs`）と visually-hidden（`data-scope`/`data-part` のみ、
/// `visually_hidden.rs`）は Demo に固有の `data-*` が無いため
/// `Data Attributes` 表を持たない（実測で確認済み、`SKIP_ATTR_TABLE` 参照）。
/// 本テストが実装外の対応を「有る」と誤って固定しないよう、Data Attributes
/// の非空要求はこの 2 件を除外する。
const SKIP_ATTR_TABLE: &[&str] = &["/primitives/skip-nav/", "/primitives/visually-hidden/"];

#[test]
fn data_display_utilities_primitive_pages_have_arguments_table() {
    for path in PATHS {
        let page = generated_content(path)
            .unwrap_or_else(|| panic!("{path} should have generated content"));
        let html = render(&page);
        assert!(
            html.contains("<h3>Arguments</h3>"),
            "{path} should render an Arguments table (ComponentPageSpec::arguments must be non-empty)"
        );
        if !SKIP_ATTR_TABLE.contains(path) {
            assert!(
                html.contains("<h3>Data Attributes</h3>"),
                "{path} should render a machine-derived Data Attributes table"
            );
        }
        assert!(
            !html.contains("<h3>CSS Variables</h3>"),
            "{path}: Primitives layer must not render a CSS Variables section"
        );
    }
}

/// 受け入れ条件 1（強化）: `features`/`examples`/`aria` も同時に非空である
/// ことを個別に固定する（948 の `||` より強い AND）。`Examples` H2 は登場
/// するが `examples: &[]` というすり抜けを防ぐため、H3 見出し数で確認する。
#[test]
fn data_display_utilities_primitive_pages_have_features_examples_and_aria() {
    for path in PATHS {
        let page = generated_content(path)
            .unwrap_or_else(|| panic!("{path} should have generated content"));
        let html = render(&page);
        assert!(
            html.contains("<h2>Features</h2>"),
            "{path} should render a Features section"
        );
        assert!(
            html.contains("<h2>Examples</h2>"),
            "{path} should render an Examples section"
        );
        assert!(
            html.contains("<h2>Accessibility</h2>"),
            "{path} should render an Accessibility section"
        );
    }
}

/// 受け入れ条件 2 の機械検知: `primitive_specs::SPEC_TABLES` に本カテゴリ
/// 10 パスがすべて登録済みで、重複が無いこと（レジストリ追記漏れの
/// fail-closed 検知）。
#[test]
fn data_display_utilities_specs_are_registered_in_primitive_spec_tables() {
    let registered: BTreeSet<&str> = primitive_specs::SPEC_TABLES
        .iter()
        .flat_map(|table| table.iter())
        .map(|(path, _)| *path)
        .collect();
    for path in PATHS {
        assert!(
            registered.contains(path),
            "{path} should be registered in primitive_specs::SPEC_TABLES"
        );
    }
    let mut seen = BTreeSet::new();
    for table in primitive_specs::SPEC_TABLES {
        for (path, _) in table.iter() {
            assert!(
                seen.insert(*path),
                "duplicate path across primitive_specs::SPEC_TABLES: {path}"
            );
        }
    }
}

/// §2.5: 原稿 `.md` は H1 + 導入文のみで H2 を作らない（右カラム目次への
/// 意図しない混入を防ぐ、`tests/component_pages.rs::component_markdown_sources_have_no_h2_headings`
/// と同型。63 件横断ガードは親 #1030 の統合時に入れる方針のため、本テストは
/// 本カテゴリ 10 件に限定する）。
#[test]
fn data_display_utilities_markdown_sources_have_no_h2_headings() {
    const KEBABS: &[&str] = &[
        "avatar",
        "carousel",
        "json-tree-view",
        "scroll-area",
        "skip-nav",
        "splitter",
        "steps",
        "tour",
        "tree-view",
        "visually-hidden",
    ];
    for kebab in KEBABS {
        let path = repo_root().join(format!("site/primitives/{kebab}.md"));
        let src = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} should be readable: {e}", path.display()));
        for line in src.lines() {
            assert!(
                !line.trim_start().starts_with("## "),
                "{} should not contain H2 headings, found: {line}",
                path.display()
            );
        }
    }
}

/// 受け入れ条件 3: 本モジュールが `fandhe-frontend-pre-styled-ui` の部品
/// 関数（styled 層）を呼ばないこと。許可されるのは
/// `fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui`（headless-ui
/// への再エクスポート import、イシュー #693 方針）1 行のみ。`raw_html`
/// も使わないこと（`tests/component_pages.rs::assert_file_has_no_raw_html_in_code`
/// と同型）。
#[test]
fn data_display_utilities_specs_do_not_use_pre_styled_ui_components() {
    let path = repo_root().join("crates/docs-site/src/primitive_specs/data_display_utilities.rs");
    let src = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} should be readable: {e}", path.display()));
    let code_only: String = src
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        !code_only.contains("raw_html"),
        "{} must not use raw_html() (REQ-1 escape bypass) in code (non-comment) lines",
        path.display()
    );

    let occurrences = code_only.matches("fandhe_frontend_pre_styled_ui::").count();
    let reexport_occurrences = code_only
        .matches("fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui")
        .count();
    assert_eq!(
        occurrences,
        reexport_occurrences,
        "{} must reference fandhe_frontend_pre_styled_ui:: only via the \
         fandhe_frontend_headless_ui re-export import, not styled-layer component functions",
        path.display()
    );
    assert_eq!(
        reexport_occurrences,
        1,
        "{} should import the headless-ui re-export exactly once",
        path.display()
    );
}

/// XSS 回帰（維持）: 合成フィクスチャの `ComponentPageSpec` へ script
/// ペイロードを注入し、`render_component_page`（`Layer::Primitives`）が
/// 既定エスケープを経由することを固定する。本番の原稿データにはペイロード
/// を入れない（`tests/component_pages.rs::features_and_table_cells_escape_xss_payloads`
/// と同型の方針）。
#[test]
fn spec_strings_are_escaped_in_rendered_page() {
    use fandhe_frontend_core::{div, text, Node};

    let payload = "<script>alert(1)</script>";
    let spec = ComponentPageSpec {
        features: &["<script>alert(1)</script>"],
        arguments: &[ArgRow {
            name: "<script>alert(1)</script>",
            kind: "<script>alert(1)</script>",
            default: "<script>alert(1)</script>",
            description: "<script>alert(1)</script>",
        }],
        examples: &[],
        keyboard: &[],
        aria: &[AriaRow {
            attribute: "\"><img src=x onerror=alert(1)>",
            description: "\"><img src=x onerror=alert(1)>",
        }],
        demo: None,
    };
    let demo: Node = div(
        vec![("data-scope", "widget")],
        vec![div(vec![("data-part", "root")], vec![text("demo")])],
    );
    let page = render_component_page("/primitives/widget/", demo, &spec, Layer::Primitives);
    let html = render(&page);
    assert!(!html.contains(payload), "raw script payload leaked: {html}");
    assert!(
        !html.contains("<img src=x"),
        "raw img payload leaked: {html}"
    );
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
}

//! Primitives Forms A（入力系 11 部品、イシュー #1024）原稿レジストリ
//! （[`fandhe_frontend_docs_site::primitive_specs::forms_a`]）専用の回帰
//! テスト。
//!
//! `tests/component_pages.rs`・`tests/primitive_showcase.rs` 等の共有テスト
//! ファイルは編集せず（6 並列実装の衝突源を避けるため、実装計画 §4 参照）、
//! 本ファイル単独で以下を固定する:
//!
//! - T1: Forms A 11 path すべてで 6 節がこの順で出る（受け入れ条件「4 節が
//!   空でない」の直接固定）。
//! - T2: `forms_a::SPECS` の path 集合が `primitives_catalog::PRIMITIVES` の
//!   `PrimitiveCategory::FormsA` と過不足なく一致する。
//! - T3: `primitive_specs::SPEC_TABLES` 全体で path 重複が無い。
//! - T4: `crates/docs-site/src/primitive_specs/` に `raw_html` が出現しない
//!   （`component_pages.rs::component_specs_source_does_not_use_raw_html` は
//!   `component_specs/` のみを対象とするため、Primitives 側の穴を塞ぐ）。
//! - T5: Examples レンダラが `fandhe_frontend_pre_styled_ui::` を
//!   `fandhe_frontend_headless_ui` 再エクスポート経由でのみ使う。
//! - T6: 11 件すべて `demo.is_none()`（D2、#982 の二重登録事故の再発防止）。
//! - T7: `site/primitives/*.md` に `## ` 見出しが無い（既存ガードは
//!   `site/themes` のみが対象）。
//! - T8: `Layer::Primitives` での XSS 回帰（既存 `features_and_table_cells_escape_xss_payloads`
//!   は `Layer::Themes` 固定のため Primitives 側を追加で固定する）。
//! - T9: 各 spec の `features`/`arguments`/`examples` が非空、かつ
//!   `keyboard`/`aria` の少なくとも一方が非空（T1 と二重化し、節省略ロジック
//!   のすり抜けを防ぐ）。

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use fandhe_frontend_core::{div, el, p, render, text};
use fandhe_frontend_docs_site::component_page::{
    render_component_page, ArgRow, AriaRow, ComponentPageSpec, KeyRow, Layer,
};
use fandhe_frontend_docs_site::primitive_specs::{self, forms_a};
use fandhe_frontend_docs_site::primitives_catalog::{PrimitiveCategory, PRIMITIVES};

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する
/// （`tests/component_pages.rs`/`tests/site_css_contract.rs` と同じ規約）。
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo_root should resolve from CARGO_MANIFEST_DIR")
}

fn h2_texts(html: &str) -> Vec<String> {
    extract_heading_texts(html, "h2")
}

fn extract_heading_texts(html: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut idx = 0;
    while let Some(rel) = html[idx..].find(&open) {
        let start = idx + rel + open.len();
        let Some(end_rel) = html[start..].find(&close) else {
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

/// T1: Forms A 11 path すべてで 6 節がこの順で出る。
#[test]
fn forms_a_pages_render_all_six_canonical_sections() {
    for (path, _) in forms_a::SPECS {
        let content = fandhe_frontend_docs_site::component_page::generated_content(path)
            .unwrap_or_else(|| panic!("{path} should have generated content"));
        let html = render(&content);
        let headings = h2_texts(&html);
        assert_eq!(
            headings, CANONICAL_SECTIONS,
            "{path} should render all six canonical sections in order, got {headings:?}"
        );
    }
}

/// T2: `forms_a::SPECS` の path 集合が台帳の Forms A カテゴリと過不足なく
/// 一致する。
#[test]
fn forms_a_paths_match_the_catalog_forms_a_category_exactly() {
    let spec_paths: BTreeSet<&str> = forms_a::SPECS.iter().map(|(path, _)| *path).collect();
    let catalog_paths: BTreeSet<&str> = PRIMITIVES
        .iter()
        .filter(|entry| entry.category == PrimitiveCategory::FormsA)
        .map(|entry| entry.path)
        .collect();
    assert_eq!(
        spec_paths, catalog_paths,
        "forms_a::SPECS must match PrimitiveCategory::FormsA exactly (no missing/extra path)"
    );
}

/// T3: `primitive_specs::SPEC_TABLES` 全体で path 重複が無い（#982 の
/// 二重登録事故と同型の再発防止）。
#[test]
fn primitive_spec_tables_have_no_duplicate_paths() {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut duplicates: Vec<&str> = Vec::new();
    for table in primitive_specs::SPEC_TABLES {
        for (path, _) in *table {
            if !seen.insert(*path) {
                duplicates.push(path);
            }
        }
    }
    assert!(
        duplicates.is_empty(),
        "SPEC_TABLES registers the following path(s) in more than one table: {duplicates:?}"
    );
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("{} should be readable: {e}", dir.display()));
    for entry in entries {
        let entry = entry.expect("dir entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn assert_file_has_no_raw_html_in_code(path: &Path) {
    let src = fs::read_to_string(path)
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
}

/// T4: `crates/docs-site/src/primitive_specs/` 配下に `raw_html` が
/// 出現しない（REQ-1、`component_pages.rs::component_specs_source_does_not_use_raw_html`
/// は `component_specs/` のみを対象とするため、Primitives 側の穴を塞ぐ）。
#[test]
fn primitive_specs_source_does_not_use_raw_html() {
    let dir = repo_root().join("crates/docs-site/src/primitive_specs");
    let mut files = Vec::new();
    collect_rs_files(&dir, &mut files);
    assert!(
        !files.is_empty(),
        "primitive_specs/ should contain at least one .rs file to guard"
    );
    for path in &files {
        assert_file_has_no_raw_html_in_code(path);
    }
}

/// T5: Examples レンダラが `fandhe_frontend_pre_styled_ui::` を直接
/// import せず、`fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui`
/// 再エクスポート経由でのみ使う（受け入れ条件「Examples が headless-ui の
/// API のみ」の機械化）。
#[test]
fn forms_a_examples_use_only_headless_ui_reexport() {
    let path = repo_root().join("crates/docs-site/src/primitive_specs/forms_a.rs");
    let src = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    // コメント行（doc コメントが規約自体を説明するために当該語を含む）は
    // 対象外とし、コードとして実際に評価される行のみを走査する
    // （`assert_file_has_no_raw_html_in_code` と同じ判断）。
    let code_only: String = src
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    const NEEDLE: &str = "fandhe_frontend_pre_styled_ui::";
    const ALLOWED_SUFFIX: &str = "fandhe_frontend_headless_ui";
    let mut idx = 0;
    let mut occurrences = 0usize;
    while let Some(rel) = code_only[idx..].find(NEEDLE) {
        let start = idx + rel;
        let after = &code_only[start + NEEDLE.len()..];
        assert!(
            after.starts_with(ALLOWED_SUFFIX),
            "forms_a.rs must only reference fandhe_frontend_pre_styled_ui:: via the \
             fandhe_frontend_headless_ui re-export; found unexpected use at byte offset {start}"
        );
        occurrences += 1;
        idx = start + NEEDLE.len();
    }
    assert!(
        occurrences >= 1,
        "forms_a.rs should reference the headless-ui re-export at least once"
    );
}

/// T6: 11 件すべて `demo.is_none()`（D2）。`generated_content` は
/// `Layer::Primitives` のとき `primitive_showcase` を先に照会するため、
/// ここで `Some` を書くと到達不能なデッドコードになる（#982 の二重登録
/// 事故と同じ形）。
#[test]
fn forms_a_specs_declare_no_demo_override() {
    for (path, spec) in forms_a::SPECS {
        assert!(
            spec.demo.is_none(),
            "{path} spec.demo must be None (primitive_showcase::forms_a already supplies Demo; \
             a Some(..) here would be unreachable dead code, see PR #982 precedent)"
        );
    }
}

/// T7: `site/primitives/*.md` に `## ` 見出しが無い（既存ガード
/// `component_markdown_sources_have_no_h2_headings` は `site/themes` のみが
/// 対象のため、Primitives 側の穴を塞ぐ。`build.rs` は Markdown 本文の後ろへ
/// Rust 生成本文を連結するため、`.md` 側に H2 があると Demo より前に出力
/// され節順が壊れる）。
#[test]
fn primitives_markdown_sources_have_no_h2_headings() {
    let dir = repo_root().join("site/primitives");
    let mut files = Vec::new();
    collect_rs_files_by_ext(&dir, "md", &mut files);
    assert!(
        !files.is_empty(),
        "site/primitives/ should contain at least one .md file to guard"
    );
    for path in &files {
        let src = fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for line in src.lines() {
            assert!(
                !line.trim_start().starts_with("## "),
                "{} must not contain an H2 heading (Rust-generated content is appended after \
                 the Markdown body; an H2 here would break the canonical section order)",
                path.display()
            );
        }
    }
}

fn collect_rs_files_by_ext(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("{} should be readable: {e}", dir.display()));
    for entry in entries {
        let entry = entry.expect("dir entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files_by_ext(&path, ext, out);
        } else if path.extension().is_some_and(|e| e == ext) {
            out.push(path);
        }
    }
}

/// `render_component_page` の走査対象になる、最小限の合成デモ（`scope`
/// 解決には影響しない `Layer::Primitives` の class を持つ）。
fn synthetic_primitives_demo() -> fandhe_frontend_core::Node {
    div(
        vec![("class", "primitives-showcase")],
        vec![el(
            "section",
            vec![],
            vec![
                el("h2", vec![], vec![text("Widget")]),
                p(vec![], vec![text("説明文")]),
            ],
        )],
    )
}

/// T8: `Layer::Primitives` での XSS 回帰固定。既存
/// `component_pages.rs::features_and_table_cells_escape_xss_payloads` は
/// `Layer::Themes` 固定のため、本テストが Primitives 側を追加で固定する。
#[test]
fn primitives_layer_spec_cells_escape_xss_payloads() {
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
        keyboard: &[KeyRow {
            key: "<script>alert(1)</script>",
            description: "<script>alert(1)</script>",
        }],
        aria: &[AriaRow {
            attribute: "<script>alert(1)</script>",
            description: "<script>alert(1)</script>",
        }],
        demo: None,
    };
    let demo = synthetic_primitives_demo();
    let page = render_component_page("/primitives/widget/", demo, &spec, Layer::Primitives);
    let html = render(&page);
    assert!(!html.contains(payload), "raw payload leaked: {html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
}

/// T9: 各 spec の `features`/`arguments`/`examples` が非空、かつ
/// `keyboard`/`aria` の少なくとも一方が非空。T1（描画側の節順固定）と
/// 二重化し、節省略ロジックのすり抜けを防ぐ。
#[test]
fn forms_a_pages_have_nonempty_features_arguments_examples_and_a11y() {
    for (path, spec) in forms_a::SPECS {
        assert!(
            !spec.features.is_empty(),
            "{path}: features must not be empty"
        );
        assert!(
            !spec.arguments.is_empty(),
            "{path}: arguments must not be empty"
        );
        assert!(
            !spec.examples.is_empty(),
            "{path}: examples must not be empty"
        );
        assert!(
            !spec.keyboard.is_empty() || !spec.aria.is_empty(),
            "{path}: at least one of keyboard/aria must not be empty"
        );
    }
}

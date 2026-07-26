//! イシュー #1026 専用の回帰テスト（Primitives Forms C・日付・状態表示
//! 10 部品の原稿充填、`crate::primitive_specs::forms_c_date_status`）。
//!
//! 共有テストファイル（`tests/component_pages.rs` / `tests/primitive_showcase.rs` /
//! `tests/primitive_showcase_xss.rs`）は #1024〜#1029 の 6 並列実装が同時に
//! 編集するため、コンフリクトを避けるべく本イシュー専用のガードをここへ
//! 独立して置く（計画書 §対象ファイル・変更箇所を参照）。
//!
//! REQ-1（既定エスケープ）・責務境界（`docs/policy/intentional-non-adoption.md`
//! §3.25、headless-ui への到達経路限定）は本ファイルが機械的に固定する。

use std::fs;
use std::path::{Path, PathBuf};

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page;
use fandhe_frontend_docs_site::primitive_specs::forms_c_date_status;
use fandhe_frontend_docs_site::primitives_catalog::{PrimitiveCategory, PRIMITIVES};

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する
/// （`tests/component_pages.rs` と同じ規約）。
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo_root should resolve from CARGO_MANIFEST_DIR")
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
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

/// `path` のコード行（`//`/`//!`/`///` コメント行を除く）から `needle` を
/// 除いた行のみを対象に走査する（`tests/component_pages.rs::assert_file_has_no_raw_html_in_code`
/// と同型のロジック。共有ファイルを編集しないため本ファイル内へ複製する）。
fn code_lines_without_comments(path: &Path) -> String {
    let src = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{} should be readable: {e}", path.display()));
    src.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 受け入れ条件 1: `raw_html()` 迂回の不在（REQ-1）。
///
/// `primitive_specs/` 配下は複数イシュー（#1024〜#1029）が原稿ノード木を
/// 大量に組み立てるディレクトリであり、`component_pages.rs` の既存ガードが
/// 対象にしていないため、本イシューの追加分だけでも独立してガードする。
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
        let code_only = code_lines_without_comments(path);
        assert!(
            !code_only.contains("raw_html"),
            "{} must not use raw_html() (REQ-1 escape bypass) in code (non-comment) lines",
            path.display()
        );
    }
}

/// 受け入れ条件 2: Forms C・日付・状態表示 10 ページすべてで Features /
/// API Reference / Examples / Accessibility の 4 節が省略されないこと。
///
/// 4 節すべてが実装根拠付きで埋まることを事前に確認済み（計画書 Step 2）
/// のため、除外リストは持たない（ワイルドカード・フラグによる緩和はしない、
/// `.claude/rules/security.md` A05）。
#[test]
fn forms_c_pages_have_all_four_manuscript_sections() {
    let required_headings = ["Features", "API Reference", "Examples", "Accessibility"];
    for (path, _spec) in forms_c_date_status::SPECS {
        let content = component_page::generated_content(path)
            .unwrap_or_else(|| panic!("registered primitive page {path} must render"));
        let html = render(&content);
        for heading in required_headings {
            let needle = format!("<h2>{heading}</h2>");
            assert!(
                html.contains(&needle),
                "{path}: missing required section <h2>{heading}</h2>"
            );
        }
    }
}

/// 受け入れ条件 3: `SPECS` の path 集合が `primitives_catalog` の
/// `FormsCDateStatus` カテゴリ 10 件と過不足なく一致すること。
#[test]
fn forms_c_specs_cover_the_catalog_category_exactly() {
    let catalog_paths: std::collections::BTreeSet<&str> = PRIMITIVES
        .iter()
        .filter(|entry| entry.category == PrimitiveCategory::FormsCDateStatus)
        .map(|entry| entry.path)
        .collect();
    let spec_paths: std::collections::BTreeSet<&str> = forms_c_date_status::SPECS
        .iter()
        .map(|(path, _)| *path)
        .collect();
    assert_eq!(
        catalog_paths, spec_paths,
        "forms_c_date_status::SPECS の path 集合は primitives_catalog の \
         FormsCDateStatus 10 件と過不足なく一致する必要がある"
    );
    assert_eq!(
        catalog_paths.len(),
        10,
        "FormsCDateStatus カテゴリは 10 件であるはず"
    );
}

/// 受け入れ条件 4: Examples が `fandhe_frontend_pre_styled_ui::` を
/// headless-ui 再エクスポート経路（`::fandhe_frontend_headless_ui`）以外で
/// 参照していないこと（pre-styled-ui のスタイル済み部品関数を呼ばない、
/// イシュー #1022 の受け入れ条件を踏襲）。
#[test]
fn forms_c_examples_do_not_call_pre_styled_ui_component_fns() {
    let path = repo_root().join("crates/docs-site/src/primitive_specs/forms_c_date_status.rs");
    let code_only = code_lines_without_comments(&path);
    const NEEDLE: &str = "fandhe_frontend_pre_styled_ui::";
    let mut idx = 0;
    while let Some(found) = code_only[idx..].find(NEEDLE) {
        let start = idx + found;
        let rest = &code_only[start + NEEDLE.len()..];
        assert!(
            rest.starts_with("fandhe_frontend_headless_ui"),
            "{}: `{NEEDLE}` 出現はすべて `::fandhe_frontend_headless_ui` \
             再エクスポート経路である必要がある（周辺: {:?}）",
            path.display(),
            &code_only[start..(start + NEEDLE.len() + 40).min(code_only.len())]
        );
        idx = start + NEEDLE.len();
    }
}

/// XSS 回帰: 原稿由来文字列（Features 節）に `<script>` を含むダミー spec を
/// 通しても `render()` の既定エスケープでエスケープされることを確認する
/// （`tests/primitive_showcase_xss.rs` は Demo 経路のみを覆うため、原稿
/// 経路の直接的な回帰として本テストを追加する）。
#[test]
fn manuscript_feature_strings_are_escaped_on_render() {
    use fandhe_frontend_docs_site::component_page::ComponentPageSpec;

    const XSS_PAYLOAD: &str = "<script>alert(1)</script>";
    let spec = ComponentPageSpec {
        features: &[XSS_PAYLOAD],
        arguments: &[],
        examples: &[],
        keyboard: &[],
        aria: &[],
        demo: None,
    };
    let demo = fandhe_frontend_core::text("dummy demo");
    let node = component_page::render_component_page(
        "/primitives/dummy/",
        demo,
        &spec,
        component_page::Layer::Primitives,
    );
    let html = render(&node);
    assert!(
        !html.contains("<script>alert(1)</script>"),
        "Features 節の原稿文字列は既定エスケープを経由する必要がある"
    );
    assert!(html.contains("&lt;script&gt;"));
}

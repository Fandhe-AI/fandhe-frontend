//! `crate::primitive_specs::navigation`（イシュー #1028、親トラッキング
//! #1035 Phase 5）の原稿充填を検証する統合テスト。
//!
//! フィールド単位で空欄を検知する（受け入れ条件 1 の本体）・6 節の描画順序を
//! 固定する・台帳との集合完全一致・`raw_html` 不使用・XSS エスケープ回帰・
//! 層専用 class 契約・決定性、の 7 観点を担う。共有テストファイル
//! （`component_pages.rs`/`primitive_showcase.rs` 等）は編集せず、本ファイル
//! に複製することで並列実装される他カテゴリ（#1024〜#1027/#1029）との
//! 衝突を避ける。

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::{
    render_component_page, ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow, Layer,
};
use fandhe_frontend_docs_site::{component_page, primitive_specs, primitives_catalog};

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する
/// （`tests/primitives_catalog.rs` 等と同じ規約）。
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo_root should resolve from CARGO_MANIFEST_DIR")
}

/// Navigation カテゴリのテーブルを取り出す。
///
/// Phase 5 は 6 カテゴリが別々の issue で並列に充填されるため
/// [`primitive_specs::SPEC_TABLES`] の要素数・並び順はカテゴリ横断で変動する。
/// 添字ではなくカテゴリ専用の定数を直接参照し、そのうえで
/// [`navigation_table_is_registered`] がレジストリへの登録を別途固定する。
fn navigation_table() -> &'static [(&'static str, ComponentPageSpec)] {
    primitive_specs::navigation::SPECS
}

/// Navigation テーブルが [`primitive_specs::SPEC_TABLES`] へ登録されていること
/// を固定する（[`navigation_table`] が定数を直接参照するため、レジストリへの
/// 追記漏れはこのテストでしか検知できない）。
///
/// 判定はテーブルのアドレス比較ではなく path 集合の包含で行う（`SPECS` は
/// `const` であり、使用箇所ごとに独立して埋め込まれ得るためアドレスの一致は
/// 保証されない）。兄弟カテゴリの同種テスト（`primitive_specs_1029.rs` 等）と
/// 同じ方式に揃えてある。
#[test]
fn navigation_table_is_registered() {
    let registered: BTreeSet<&str> = primitive_specs::SPEC_TABLES
        .iter()
        .flat_map(|table| table.iter().map(|(path, _)| *path))
        .collect();
    for (path, _) in primitive_specs::navigation::SPECS {
        assert!(
            registered.contains(path),
            "{path} should be registered in primitive_specs::SPEC_TABLES"
        );
    }
}

const EXPECTED_PATHS: &[&str] = &[
    "/primitives/action-bar/",
    "/primitives/breadcrumb/",
    "/primitives/link/",
    "/primitives/link-overlay/",
    "/primitives/menu/",
    "/primitives/menubar/",
    "/primitives/nav-list/",
    "/primitives/navigation-menu/",
    "/primitives/pagination/",
    "/primitives/tabs/",
    "/primitives/toolbar/",
];

/// 受け入れ条件 1 の本体: Navigation 11 部品それぞれについて
/// `features`/`arguments`/`examples` が非空、かつ `keyboard`/`aria` の和が
/// 非空であることを固定する（「6 節の H2 が揃う」だけでは `data-*` 属性表
/// のみで API Reference 見出しが出てしまい、空欄のまま緑になる穴がある）。
#[test]
fn navigation_specs_have_no_empty_manuscript_fields() {
    let table = navigation_table();
    assert_eq!(table.len(), EXPECTED_PATHS.len());
    for (path, spec) in table {
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
            "{path}: keyboard と aria の少なくとも一方は非空でなければならない"
        );
    }
}

/// Demo → Features → Anatomy → API Reference → Examples → Accessibility の
/// 6 節すべてが正順で描画されることを固定する（フィールド単位の検証とは
/// 別の失敗様態を捕捉するため両方置く）。
#[test]
fn navigation_pages_render_all_six_canonical_sections() {
    const REQUIRED_SECTIONS: &[&str] = &[
        "Demo",
        "Features",
        "Anatomy",
        "API Reference",
        "Examples",
        "Accessibility",
    ];
    for path in EXPECTED_PATHS {
        let node = component_page::generated_content(path)
            .unwrap_or_else(|| panic!("{path} should render generated content"));
        let html = render(&node);
        let mut last_index = 0usize;
        for heading in REQUIRED_SECTIONS {
            let marker = format!(">{heading}<");
            let index = html[last_index..].find(&marker).unwrap_or_else(|| {
                panic!("{path}: missing heading {heading:?} after byte {last_index}")
            });
            last_index += index + marker.len();
        }
    }
}

/// Navigation テーブルの path 集合が `primitives_catalog` の Navigation
/// カテゴリの path 集合と完全一致することを双方向 fail-closed に固定する
/// （過不足のいずれも見逃さない）。
#[test]
fn navigation_specs_match_the_catalog_exactly() {
    let table = navigation_table();
    let spec_paths: BTreeSet<&str> = table.iter().map(|(path, _)| *path).collect();
    let catalog_paths: BTreeSet<&str> = primitives_catalog::PRIMITIVES
        .iter()
        .filter(|entry| entry.category == primitives_catalog::PrimitiveCategory::Navigation)
        .map(|entry| entry.path)
        .collect();
    assert_eq!(
        spec_paths, catalog_paths,
        "primitive_specs::navigation::SPECS の path 集合は primitives_catalog の Navigation カテゴリと完全一致すること"
    );
    assert_eq!(spec_paths.len(), EXPECTED_PATHS.len());
}

/// `crates/docs-site/src/primitive_specs/` 配下のコード（コメント行を除く）
/// に `raw_html` が出現しないことを固定する（REQ-1 の機械的ガード。
/// `component_pages.rs::component_specs_source_does_not_use_raw_html` と
/// 同型のガードを、共有ファイルを編集せず本ファイルへ複製する）。
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

/// XSS 回帰（受け入れ条件 5）: `features`/`ArgRow`/`AriaRow`/`KeyRow` に
/// `<script>` ペイロードを仕込んだ合成フィクスチャを Primitives 層で描画し、
/// 生ペイロードが出力へ混入せず、エスケープ済み文字列へ変換されることを
/// 固定する。
#[test]
fn primitives_layer_manuscript_fields_escape_xss_payloads() {
    const PAYLOAD: &str = "<script>alert(1)</script>";
    fn render_example() -> fandhe_frontend_core::Node {
        fandhe_frontend_core::text(PAYLOAD)
    }
    let spec = ComponentPageSpec {
        features: &[PAYLOAD],
        arguments: &[ArgRow {
            name: PAYLOAD,
            kind: PAYLOAD,
            default: PAYLOAD,
            description: PAYLOAD,
        }],
        examples: &[ExampleEntry {
            title: PAYLOAD,
            description: PAYLOAD,
            render: render_example,
        }],
        keyboard: &[KeyRow {
            key: PAYLOAD,
            description: PAYLOAD,
        }],
        aria: &[AriaRow {
            attribute: PAYLOAD,
            description: PAYLOAD,
        }],
        demo: None,
    };
    let demo = fandhe_frontend_core::div(vec![], vec![]);
    let node = render_component_page("/primitives/xss-fixture/", demo, &spec, Layer::Primitives);
    let html = render(&node);
    assert!(
        !html.contains("<script>"),
        "raw <script> payload must not appear in rendered output: {html}"
    );
    assert!(
        html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"),
        "payload must be escaped: {html}"
    );
}

/// Examples 節が pre-styled-ui 専用 class（`showcase-row` 等）を持ち込まず、
/// Primitives 層の許容 class（`primitives-showcase`/`primitives-demo-*`）
/// 以外を出力しないことを固定する（`site_css_contract.rs` に依存せず本 PR
/// 単体でも検知できるようにする）。
#[test]
fn navigation_examples_introduce_no_foreign_classes() {
    for path in EXPECTED_PATHS {
        let node = component_page::generated_content(path)
            .unwrap_or_else(|| panic!("{path} should render generated content"));
        let html = render(&node);
        for class_value in extract_class_values(&html) {
            for token in class_value.split_whitespace() {
                assert!(
                    token == "primitives-showcase" || token.starts_with("primitives-demo-"),
                    "{path}: unexpected class token {token:?} (html={html})"
                );
            }
        }
    }
}

/// `class="..."` の値を単純走査で抽出する（HTML パーサ依存を避けた最小限の
/// 文字列走査。属性値内に `"` を含まない前提は本クレートの `render()` が
/// 属性値をエスケープすることで保証される）。
fn extract_class_values(html: &str) -> Vec<String> {
    let marker = "class=\"";
    let mut values = Vec::new();
    let mut idx = 0;
    while let Some(rel) = html[idx..].find(marker) {
        let start = idx + rel + marker.len();
        if let Some(end_rel) = html[start..].find('"') {
            values.push(html[start..start + end_rel].to_string());
            idx = start + end_rel;
        } else {
            break;
        }
    }
    values
}

/// 11 path で `generated_content` を 2 回描画して一致することを固定する
/// （既存の決定性契約に合わせる。ビルド差分の予測不能な揺れを防ぐ）。
#[test]
fn navigation_pages_are_deterministic() {
    for path in EXPECTED_PATHS {
        let first = render(
            &component_page::generated_content(path)
                .unwrap_or_else(|| panic!("{path} should render generated content")),
        );
        let second = render(
            &component_page::generated_content(path)
                .unwrap_or_else(|| panic!("{path} should render generated content")),
        );
        assert_eq!(
            first, second,
            "{path}: generated_content must be deterministic"
        );
    }
}

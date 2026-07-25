//! 部品ページ雛形レンダラ（`crate::component_page`、イシュー #942）の
//! 節順・Anatomy 機械導出・XSS 回帰を検証する統合テスト。
//!
//! 実ページ（[`showcase::component_page_paths`] の全登録パス）に加え、
//! 合成 [`ComponentPageSpec`] フィクスチャで 6 節すべてを埋めた場合の
//! 節順を固定する（Phase 3 時点では `COMPONENT_SPECS` が空のため、実ページ
//! だけでは Features/API 引数表/Examples/Accessibility が揃わない。合成
//! フィクスチャが「6 節すべて揃った場合の順序契約」を discharge する）。

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::{
    render_component_page, ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow,
};
use fandhe_frontend_docs_site::showcase;

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する
/// （`tests/site_css_contract.rs`/`tests/site_showcase.rs` と同じ規約）。
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo_root should resolve from CARGO_MANIFEST_DIR")
}

/// HTML 文字列から `h2` 見出しテキストを出現順に抽出する（雑駁だが
/// 本テストの入力は本モジュール自身が組み立てた既知の構造のみであり、
/// 汎用 HTML パーサは不要）。
fn h2_texts(html: &str) -> Vec<String> {
    extract_heading_texts(html, "h2")
}

fn h3_texts(html: &str) -> Vec<String> {
    extract_heading_texts(html, "h3")
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

/// 合成フィクスチャ用の Examples レンダラ。
fn example_render() -> fandhe_frontend_core::Node {
    fandhe_frontend_core::p(vec![], vec![fandhe_frontend_core::text("example body")])
}

/// 全 6 節を埋める合成 spec（受け入れ条件 1 の discharge に使う）。
fn full_spec() -> ComponentPageSpec {
    ComponentPageSpec {
        features: &["Feature one", "Feature two"],
        arguments: &[ArgRow {
            name: "variant",
            kind: "Variant",
            default: "Solid",
            description: "見た目のバリアント。",
        }],
        examples: &[ExampleEntry {
            title: "Basic",
            description: "最小構成の例。",
            render: example_render,
        }],
        keyboard: &[KeyRow {
            key: "ArrowDown",
            description: "次の項目へ移動する。",
        }],
        aria: &[AriaRow {
            attribute: "aria-expanded",
            description: "開閉状態を表す。",
        }],
    }
}

/// [`showcase::generated_content`] 相当の最小デモ木（`div.pre-styled-showcase`
/// 直下に `section`（`h2`/`p`/`data-scope` 要素）を持つ形）。
/// `data-scope="widget"` 要素を含めることで Anatomy 節（機械導出）が
/// 省略されずに埋まる（`/components/widget/` のパス kebab と一致させ、
/// §3.4 のバケット 1 解決に乗せる）。
fn synthetic_demo() -> fandhe_frontend_core::Node {
    use fandhe_frontend_core::{div, el, p, text};
    div(
        vec![("class", "pre-styled-showcase")],
        vec![el(
            "section",
            vec![],
            vec![
                el("h2", vec![], vec![text("Widget")]),
                p(vec![], vec![text("説明文")]),
                el(
                    "div",
                    vec![
                        ("data-scope", "widget"),
                        ("data-part", "root"),
                        ("data-state", "open"),
                    ],
                    vec![],
                ),
            ],
        )],
    )
}

#[test]
fn full_spec_fixture_fixes_the_canonical_six_section_order() {
    let demo = synthetic_demo();
    let page = render_component_page("/components/widget/", demo, &full_spec());
    let html = render(&page);
    assert_eq!(h2_texts(&html), CANONICAL_SECTIONS.to_vec());
    // API Reference/Accessibility 節内の小見出しは H3 に留まる契約
    // （右カラム目次が `docs-toc-level-2`/`-3` の 2 段しか出さない前提）。
    // 「widget」は実在しない合成スコープのため CSS 変数表（実 CSS から機械
    // 導出）は 0 件で省略される。Arguments/Data Attributes は spec/demo 由来
    // で必ず埋まる。
    assert_eq!(
        h3_texts(&html),
        vec![
            "Arguments".to_string(),
            "Data Attributes".to_string(),
            "Basic".to_string(),
            "Keyboard Interactions".to_string(),
            "WAI-ARIA".to_string(),
        ]
    );
}

/// `data-scope` を一切持たない最小デモ木（Anatomy 節も含め全省略節が
/// 揃った状態を作るための専用フィクスチャ）。
fn synthetic_demo_without_scope() -> fandhe_frontend_core::Node {
    use fandhe_frontend_core::{div, el, p, text};
    div(
        vec![("class", "pre-styled-showcase")],
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

#[test]
fn empty_spec_still_emits_demo_section_only() {
    let demo = synthetic_demo_without_scope();
    let page = render_component_page("/components/widget/", demo, &ComponentPageSpec::EMPTY);
    let html = render(&page);
    // Anatomy 節は demo に data-scope が無いため省略される。Demo のみが残る。
    assert_eq!(h2_texts(&html), vec!["Demo".to_string()]);
}

#[test]
fn real_pages_h2_sequence_is_a_prefix_respecting_subsequence_of_canonical_order_and_starts_with_demo(
) {
    for path in showcase::component_page_paths() {
        let content = fandhe_frontend_docs_site::component_page::generated_content(path)
            .unwrap_or_else(|| panic!("registered path {path} must have generated content"));
        let html = render(&content);
        let headings = h2_texts(&html);
        assert_eq!(
            headings.first().map(String::as_str),
            Some("Demo"),
            "page {path} must start with a Demo section, got {headings:?}"
        );
        // 部分列であることの検証: canonical 順のインデックス列が単調増加。
        let mut last_idx: i32 = -1;
        for heading in &headings {
            let idx = CANONICAL_SECTIONS
                .iter()
                .position(|s| s == heading)
                .unwrap_or_else(|| panic!("page {path} has unexpected h2 {heading:?}"));
            assert!(
                (idx as i32) > last_idx,
                "page {path} h2 sequence {headings:?} is not a subsequence of canonical order"
            );
            last_idx = idx as i32;
        }
    }
}

#[test]
fn real_pages_demo_section_has_exactly_one_h2() {
    for path in showcase::component_page_paths() {
        let content = fandhe_frontend_docs_site::component_page::generated_content(path).unwrap();
        let html = render(&content);
        let demo_marker = "<h2>Demo</h2>";
        let start = html
            .find(demo_marker)
            .unwrap_or_else(|| panic!("page {path} must contain a Demo heading"));
        // Demo 節の範囲は次の h2（存在すれば）または文書末尾まで。
        let after = start + demo_marker.len();
        let rest = &html[after..];
        let next_h2_offset = rest.find("<h2>").unwrap_or(rest.len());
        let demo_body = &rest[..next_h2_offset];
        let h2_count_in_body = demo_body.matches("<h2>").count();
        assert_eq!(
            h2_count_in_body, 0,
            "page {path} Demo section must not contain a residual component-name h2, body={demo_body:?}"
        );
    }
}

/// headless-ui/pre-styled-ui ソースから `data-part` として出力され得る
/// リテラル文字列の全量を集める。3 パターンを併用する:
///
/// 1. `Anatomy::part("<name>", …)` 呼び出し（空白・改行を挟む整形にも対応）
/// 2. `el(tag, vec![("data-part", "<name>"), …], …)` のような直接構築
/// 3. 上記 2 パターンで拾いきれない、`match` 経由で組み立てるパーツ名
///    （例: `color_picker::Channel::parts()` が返す `"hue-slider"` 等）を
///    拾うための保険として、ハイフンを含む kebab-case 文字列リテラル全量
///
/// 3 番目は粗い over-approximation だが、対象を「headless-ui/pre-styled-ui
/// の実ソース」に限定しているため、docs 側の手書きパーツ一覧を許容する
/// ものではない（一次情報が headless-ui/pre-styled-ui のソースコードで
/// あることの検証という受け入れ条件 2 の趣旨は保たれる）。
fn declared_part_name_pool() -> BTreeSet<String> {
    let repo_root = repo_root();
    let mut files = Vec::new();
    collect_rs_files(&repo_root.join("crates/headless-ui/src"), &mut files);
    collect_rs_files(&repo_root.join("crates/pre-styled-ui/src"), &mut files);
    let mut set = BTreeSet::new();
    for file in files {
        let Ok(src) = fs::read_to_string(&file) else {
            continue;
        };
        collect_literals_after(&src, ".part(", &mut set);
        collect_literals_after(&src, "(\"data-part\",", &mut set);
        collect_hyphenated_literals(&src, &mut set);
    }
    set
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

fn collect_literals_after(src: &str, marker: &str, set: &mut BTreeSet<String>) {
    let mut idx = 0;
    while let Some(rel) = src[idx..].find(marker) {
        let after = idx + rel + marker.len();
        let rest = src[after..].trim_start();
        if let Some(stripped) = rest.strip_prefix('"') {
            if let Some(end_rel) = stripped.find('"') {
                set.insert(stripped[..end_rel].to_string());
            }
        }
        idx = after;
    }
}

fn collect_hyphenated_literals(src: &str, set: &mut BTreeSet<String>) {
    let mut idx = 0;
    while let Some(rel) = src[idx..].find('"') {
        let start = idx + rel + 1;
        let Some(end_rel) = src[start..].find('"') else {
            break;
        };
        let candidate = &src[start..start + end_rel];
        if is_kebab_with_hyphen(candidate) {
            set.insert(candidate.to_string());
        }
        idx = start + end_rel + 1;
    }
}

fn is_kebab_with_hyphen(s: &str) -> bool {
    if !s.contains('-')
        || s.len() > 40
        || s.starts_with('-')
        || s.ends_with('-')
        || s.contains("--")
    {
        return false;
    }
    let Some(first) = s.chars().next() else {
        return false;
    };
    first.is_ascii_lowercase()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Anatomy コードブロック（`<h2>Anatomy</h2><pre><code>…</code></pre>`）の
/// 本文からパーツ名（各行、インデント除去済み）を取り出す。
fn anatomy_part_names(html: &str) -> BTreeSet<String> {
    let marker = "<h2>Anatomy</h2><pre><code>";
    let Some(start_rel) = html.find(marker) else {
        return BTreeSet::new();
    };
    let after = start_rel + marker.len();
    let rest = &html[after..];
    let Some(end_rel) = rest.find("</code></pre>") else {
        return BTreeSet::new();
    };
    rest[..end_rel]
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[test]
fn anatomy_parts_are_a_subset_of_headless_ui_declared_parts_for_every_page() {
    let pool = declared_part_name_pool();
    for path in showcase::component_page_paths() {
        let content = fandhe_frontend_docs_site::component_page::generated_content(path).unwrap();
        let html = render(&content);
        let derived = anatomy_part_names(&html);
        let extra: Vec<_> = derived.difference(&pool).collect();
        assert!(
            extra.is_empty(),
            "page {path} derived anatomy parts {extra:?} are not traceable to any \
             headless-ui/pre-styled-ui `.part(...)`/`(\"data-part\", ...)` literal"
        );
    }
}

/// `card`（pre-styled-ui 単体、6 パーツ）・`drawer`（headless-ui 定義、8
/// パーツ）は、ショーケースデモが全パーツを描画する部品として実測で選定
/// した（`crates/pre-styled-ui/src/card.rs`/`crates/headless-ui/src/drawer.rs`
/// の `.part("<name>", …)` 呼び出し数と Anatomy 導出結果が一致することを
/// 事前調査で確認済み）。accordion は `item-indicator` を描画しないため
/// 完全一致の固定には使えない（設計 §3.4/§6 の除外理由どおり）。
#[test]
fn anatomy_parts_exactly_match_declared_parts_for_fully_demonstrated_components() {
    let cases: &[(&str, &[&str])] = &[
        (
            "/components/card/",
            &["root", "header", "body", "footer", "title", "description"],
        ),
        (
            "/components/drawer/",
            &[
                "root",
                "trigger",
                "backdrop",
                "positioner",
                "content",
                "title",
                "description",
                "close-trigger",
            ],
        ),
    ];
    for (path, expected_parts) in cases {
        let content = fandhe_frontend_docs_site::component_page::generated_content(path).unwrap();
        let html = render(&content);
        let derived = anatomy_part_names(&html);
        let expected: BTreeSet<String> = expected_parts.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            derived, expected,
            "page {path} anatomy parts should exactly match the demo's fully-demonstrated part set"
        );
    }
}

/// §3.4 のスコープ解決バケット件数（実測、`origin/main` d0ca7c5 時点の
/// `COMPONENT_PAGES` 登録 88 件に対して固定）。将来の部品追加でバケット 3
/// （Anatomy 省略）へ無言に落ちることを検知するための固定値テスト。
/// バケット 2（フォールバック解決）は `input`/`textarea`/`native-select`
/// （いずれも headless `field::input` の共有スコープ `"field"` を使い、
/// パスの kebab（`input`/`textarea`/`native-select`）と一致しない）と
/// `charts`（複数チャート scope の集約ページで単一 scope に一致しない）の
/// 4 件。
#[test]
fn scope_resolution_buckets_match_expected_counts() {
    let mut bucket1_path_match = 0usize;
    let mut bucket2_fallback = 0usize;
    let mut bucket3_none = 0usize;
    for path in showcase::component_page_paths() {
        let content = fandhe_frontend_docs_site::component_page::generated_content(path).unwrap();
        let html = render(&content);
        let has_anatomy = html.contains("<h2>Anatomy</h2>");
        if !has_anatomy {
            bucket3_none += 1;
            continue;
        }
        let candidate = path.trim_matches('/').rsplit('/').next().unwrap();
        let candidate_scope_marker = format!("data-scope=\"{candidate}\"");
        if html.contains(&candidate_scope_marker) {
            bucket1_path_match += 1;
        } else {
            bucket2_fallback += 1;
        }
    }
    assert_eq!(bucket1_path_match, 84);
    assert_eq!(bucket2_fallback, 4);
    assert_eq!(bucket3_none, 0);
}

#[test]
fn features_and_table_cells_escape_xss_payloads() {
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
    };
    let demo = synthetic_demo();
    let page = render_component_page("/components/widget/", demo, &spec);
    let html = render(&page);
    assert!(!html.contains(payload), "raw payload leaked: {html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
}

#[test]
fn component_page_source_does_not_use_raw_html() {
    let src = fs::read_to_string(repo_root().join("crates/docs-site/src/component_page.rs"))
        .expect("component_page.rs should be readable");
    // ドキュメンテーションコメント（`//!`/`///`）は `raw_html()` を
    // 「使わない」と説明するために当該語を含むため、コード行（コメント
    // 以外）のみを対象に実呼び出し・import が無いことを検証する。
    let code_only: String = src
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.starts_with("//")
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !code_only.contains("raw_html"),
        "component_page.rs must not use raw_html() (REQ-1 escape bypass) in code (non-comment) lines"
    );
}

#[test]
fn data_attrs_and_css_var_tables_are_deterministic_across_repeated_renders() {
    for path in ["/components/accordion/", "/components/dialog/"] {
        let first = render(
            &fandhe_frontend_docs_site::component_page::generated_content(path)
                .expect("registered path must have generated content"),
        );
        let second = render(
            &fandhe_frontend_docs_site::component_page::generated_content(path)
                .expect("registered path must have generated content"),
        );
        assert_eq!(
            first, second,
            "generated_content({path}) must be deterministic"
        );
    }
}

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
    render_component_page, ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow, Layer,
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
        demo: None,
    }
}

/// [`showcase::generated_content`] 相当の最小デモ木（`div.pre-styled-showcase`
/// 直下に `section`（`h2`/`p`/`data-scope` 要素）を持つ形）。
/// `data-scope="widget"` 要素を含めることで Anatomy 節（機械導出）が
/// 省略されずに埋まる（`/themes/widget/` のパス kebab と一致させ、
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
    let page = render_component_page("/themes/widget/", demo, &full_spec(), Layer::Themes);
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

/// `data-scope="accordion"` を持つ最小デモ木。`accordion` は実 CSS
/// （`showcase::stylesheet()`）に `--fandhe-accordion-*` 変数が実在する
/// スコープであることを実測確認済み（`cargo run -p fandhe-frontend-docs-site`
/// で生成した `dist/themes/accordion/index.html` に `CSS Variables` 節が
/// 出現することを事前検証した、計画 §6-3 の非空虚性要件）。層差テスト
/// （[`layer_governs_css_variables_section_and_wrapper_class`]）の対照群・
/// 実験群を同一フィクスチャで作るための専用ヘルパ。
fn synthetic_demo_with_accordion_scope() -> fandhe_frontend_core::Node {
    use fandhe_frontend_core::{div, el, p, text};
    div(
        vec![("class", "pre-styled-showcase")],
        vec![el(
            "section",
            vec![],
            vec![
                el("h2", vec![], vec![text("Accordion")]),
                p(vec![], vec![text("説明文")]),
                el(
                    "div",
                    vec![
                        ("data-scope", "accordion"),
                        ("data-part", "root"),
                        ("data-state", "open"),
                    ],
                    vec![],
                ),
            ],
        )],
    )
}

/// イシュー #1021: `Layer` が (1) CSS 変数表の有無、(2) Demo ラッパ class を
/// 制御し、(3) Anatomy・`data-*` 属性表の走査は層非依存で共通に効くことを
/// 同一フィクスチャの対照群（Themes）・実験群（Primitives）で固定する
/// （非空虚性を担保するため必ずペアで書く、計画 §6-3）。
#[test]
fn layer_governs_css_variables_section_and_wrapper_class() {
    let themes_page = render_component_page(
        "/themes/accordion/",
        synthetic_demo_with_accordion_scope(),
        &full_spec(),
        Layer::Themes,
    );
    let themes_html = render(&themes_page);
    assert!(
        themes_html.contains("<h3>CSS Variables</h3>"),
        "Themes 層は CSS 変数表を出す契約: {themes_html}"
    );
    assert!(themes_html.contains(r#"class="pre-styled-showcase""#));

    let primitives_page = render_component_page(
        "/primitives/accordion/",
        synthetic_demo_with_accordion_scope(),
        &full_spec(),
        Layer::Primitives,
    );
    let primitives_html = render(&primitives_page);
    assert!(
        !primitives_html.contains("CSS Variables"),
        "Primitives 層は CSS 変数表を恒常的に省略する契約: {primitives_html}"
    );
    assert!(primitives_html.contains(r#"class="primitives-showcase""#));

    // Anatomy・data-* 属性表の走査は層非依存で共通に効くことの肯定形確認。
    assert!(primitives_html.contains("<h2>Anatomy</h2>"));
    assert!(primitives_html.contains("<h3>Data Attributes</h3>"));
}

/// [`Layer::from_page_path`] の全域性を固定する。
#[test]
fn layer_from_page_path_is_total_and_defaults_to_themes() {
    assert_eq!(
        Layer::from_page_path("/primitives/accordion/"),
        Layer::Primitives
    );
    assert_eq!(Layer::from_page_path("/themes/accordion/"), Layer::Themes);
    assert_eq!(Layer::from_page_path("/guides/"), Layer::Themes);
    assert_eq!(Layer::from_page_path("/"), Layer::Themes);
    assert_eq!(Layer::from_page_path(""), Layer::Themes);
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
    let page = render_component_page(
        "/themes/widget/",
        demo,
        &ComponentPageSpec::EMPTY,
        Layer::Themes,
    );
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
            "/themes/card/",
            &["root", "header", "body", "footer", "title", "description"],
        ),
        (
            "/themes/drawer/",
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
        ("/themes/toggle/", &["root", "indicator"]),
        ("/themes/toggle-group/", &["root", "item"]),
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

/// §3.4 のスコープ解決バケット件数（実測、`COMPONENT_PAGES` 登録件数
/// 〔イシュー #980 で toggle/toggle-group・#991 で Toolbar・#992 で
/// Menubar・#993 で Navigation Menu・#994 で Callout・#995 で Quote /
/// Strong・#996 で Tab Nav・#997 で Checkbox Group を追加登録した後の
/// 件数〕に対して固定）。将来の
/// 部品追加でバケット 3（Anatomy 省略）へ無言に落ちることを検知するための
/// 固定値テスト。バケット 2（フォールバック解決）は
/// `input`/`textarea`/`native-select`（いずれも headless `field::input` の
/// 共有スコープ `"field"` を使い、パスの kebab
/// （`input`/`textarea`/`native-select`）と一致しない）と `charts`
/// （複数チャート scope の集約ページで単一 scope に一致しない）の 4 件
/// （不変）。Toolbar/Menubar/Navigation Menu/Callout/Tab Nav は
/// `data-scope="<kebab>"` がパスの kebab と一致するためバケット 1 に加わる。
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
    // イシュー #994 で Callout（path 由来の kebab callout が data-scope と
    // 一致）が加わり 89 -> 90、イシュー #995 で Quote / Strong の 2 部品
    // ページが加わり（いずれも path 由来の kebab quote/strong が
    // data-scope とそのまま一致する）90 -> 92、イシュー #996 で Tab Nav が
    // 加わり 92 -> 93、イシュー #997 で Checkbox Group（path 由来の kebab
    // checkbox-group が data-scope="checkbox-group" と一致）が加わり
    // 93 -> 94、イシュー #1154 で Link / Link Overlay / Nav List の 3 部品
    // （いずれも path 由来の kebab link/link-overlay/nav-list が
    // data-scope とそのまま一致する）が加わり 94 -> 97、イシュー #1683 で
    // Collapsible（path 由来の kebab collapsible が data-scope="collapsible"
    // と一致）が加わり 97 -> 98、イシュー #1685 で Field（path 由来の
    // kebab field が data-scope="field" と一致）が加わり 98 -> 99、
    // イシュー #1687 で Fieldset（path 由来の kebab fieldset が
    // data-scope="fieldset" と一致）が加わり 99 -> 100 へ増える。
    assert_eq!(bucket1_path_match, 100);
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
        demo: None,
    };
    let demo = synthetic_demo();
    let page = render_component_page("/themes/widget/", demo, &spec, Layer::Themes);
    let html = render(&page);
    assert!(!html.contains(payload), "raw payload leaked: {html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
}

#[test]
fn component_page_source_does_not_use_raw_html() {
    // イシュー #946: `crate::component_specs_overlay`（Overlay/Disclosure
    // 系 13 部品の原稿データ）・`crate::component_specs_nav_data`
    // （Navigation/Data Display 系 27 部品の原稿データ、イシュー #947）も
    // 同じ REQ-1 不変条件の走査対象に含める。
    for rel_path in [
        "crates/docs-site/src/component_page.rs",
        "crates/docs-site/src/component_specs_overlay.rs",
        "crates/docs-site/src/component_specs_nav_data.rs",
    ] {
        assert_file_has_no_raw_html_in_code(&repo_root().join(rel_path));
    }
}

/// [`component_page_source_does_not_use_raw_html`] の REQ-1 ガードを
/// `crates/docs-site/src/component_specs/` 配下へ拡張する（イシュー #945）。
/// Phase 4（#945〜#948）の各 issue がノード木を大量に組み立てる原稿データを
/// 本ディレクトリへ追加するため、`component_page.rs` 1 ファイルのみを検査
/// する従来のガードでは空洞化する。
#[test]
fn component_specs_source_does_not_use_raw_html() {
    let dir = repo_root().join("crates/docs-site/src/component_specs");
    let mut files = Vec::new();
    collect_rs_files(&dir, &mut files);
    assert!(
        !files.is_empty(),
        "component_specs/ should contain at least one .rs file to guard"
    );
    for path in &files {
        assert_file_has_no_raw_html_in_code(path);
    }
}

/// `crates/docs-site/src/primitive_specs/` 配下の REQ-1 ガード
/// （`raw_html()` 迂回検出）は `tests/primitive_specs_forms_a.rs::T4` /
/// `tests/primitive_specs_1026.rs::primitive_specs_source_does_not_use_raw_html`
/// が `primitive_specs/` 配下の全 `.rs`（Forms B の `forms_b.rs` を含む）を
/// 再帰的に検査済みであり、本ファイルへ重複するテストを追加しない
/// （Cursor Bugbot 指摘、イシュー #1025 PR #1050 レビュー、二重の正本化を防ぐ）。
///
/// `path` のコード行（`//`/`//!`/`///` コメント行を除く）に `raw_html` が
/// 出現しないことを検証する（REQ-1 の機械的ガード。ドキュメンテーション
/// コメントが「`raw_html()` を使わない」と説明するために当該語を含む
/// ことがあるため、コメント行は対象外とする）。
fn assert_file_has_no_raw_html_in_code(path: &Path) {
    let src = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{} should be readable: {e}", path.display()));
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
        "{} must not use raw_html() (REQ-1 escape bypass) in code (non-comment) lines",
        path.display()
    );
}

/// イシュー #946: Overlay / Disclosure 系 13 部品ページ（`COMPONENT_SPECS`
/// へ登録済み）が Demo / Features / Anatomy / API Reference / Accessibility
/// の 5 節（Examples は任意のため必須にしない）をすべて含むことを固定する。
/// イシュー #991 で Toolbar が加わり 14 部品ページになった。
#[test]
fn overlay_disclosure_pages_include_all_required_sections() {
    const REQUIRED_SECTIONS: &[&str] = &[
        "Demo",
        "Features",
        "Anatomy",
        "API Reference",
        "Accessibility",
    ];
    const PATHS: &[&str] = &[
        "/themes/accordion/",
        "/themes/action-bar/",
        "/themes/collapsible/",
        "/themes/dialog/",
        "/themes/drawer/",
        "/themes/floating-panel/",
        "/themes/hover-card/",
        "/themes/menu/",
        "/themes/menubar/",
        "/themes/popover/",
        "/themes/tabs/",
        "/themes/toast/",
        "/themes/toggle-tip/",
        "/themes/toolbar/",
        "/themes/tooltip/",
        "/themes/tour/",
    ];
    for path in PATHS {
        let content = fandhe_frontend_docs_site::component_page::generated_content(path)
            .unwrap_or_else(|| panic!("registered path {path} must have generated content"));
        let html = render(&content);
        let headings = h2_texts(&html);
        for section in REQUIRED_SECTIONS {
            assert!(
                headings.iter().any(|h| h == section),
                "page {path} is missing required section {section:?}, got {headings:?}"
            );
        }
    }
}

/// イシュー #946: 掲示制約 note（オーバーレイ配置の掲示専用 CSS 説明）を
/// 持つべき 11 ページの `.md` に文言が残り、種別が `[!IMPORTANT]`（未充填
/// マーカー `[!NOTE]` の残置ゼロ）であることを固定する。11 件のリストは
/// 増減を fail-closed に検知するための定数であり、対象ページを増減する
/// 変更は本テストの更新も伴う必要がある。
#[test]
fn overlay_pages_retain_overlay_placement_admonition_as_important() {
    const PAGES_WITH_OVERLAY_NOTE: &[&str] = &[
        "action-bar",
        "dialog",
        "drawer",
        "floating-panel",
        "hover-card",
        "menu",
        "popover",
        "toast",
        "toggle-tip",
        "tooltip",
        "tour",
    ];
    for name in PAGES_WITH_OVERLAY_NOTE {
        let path = repo_root().join(format!("site/themes/{name}.md"));
        let src = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} should be readable: {e}", path.display()));
        assert!(
            src.contains("オーバーレイ部品"),
            "site/themes/{name}.md must retain the overlay-placement admonition text"
        );
        assert!(
            src.contains("recipe CSS"),
            "site/themes/{name}.md must retain the overlay-placement admonition text"
        );
        assert!(
            src.contains("[!IMPORTANT]"),
            "site/themes/{name}.md must use [!IMPORTANT] for the overlay-placement admonition"
        );
        assert!(
            !src.contains("[!NOTE]"),
            "site/themes/{name}.md must not retain the Phase 4 stub [!NOTE] marker"
        );
    }
}

/// イシュー #946: 本 PR が充填する 15 ページすべてから「Phase 4（#945〜#948）
/// で充填予定」のスタブ文言が除去されていることを固定する。
#[test]
fn filled_pages_no_longer_reference_phase_4_stub_note() {
    const FILLED_PAGES: &[&str] = &[
        "accordion",
        "action-bar",
        "dialog",
        "drawer",
        "floating-panel",
        "hover-card",
        "menu",
        "popover",
        "tabs",
        "toast",
        "toggle-tip",
        "tooltip",
        "tour",
        "toggle",
        "toggle-group",
    ];
    for name in FILLED_PAGES {
        let path = repo_root().join(format!("site/themes/{name}.md"));
        let src = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} should be readable: {e}", path.display()));
        assert!(
            !src.contains("Phase 4"),
            "site/themes/{name}.md must no longer reference the Phase 4 stub note"
        );
    }
}

/// Forms 33 ページ（イシュー #945、#1685 で Field・#1687 で Fieldset を追加）の充填を機械的に固定する。各ページが
/// `Demo`/`Features`/`Anatomy`/`API Reference` の 4 節を（この順の部分列と
/// して）持つこと、および `Examples`/`Accessibility` を含む場合は
/// [`CANONICAL_SECTIONS`] 順であることを検証する（設計 §7 は Examples/
/// Accessibility を任意としているため必須節には含めない）。
#[test]
fn forms_pages_have_the_canonical_sections_filled() {
    const REQUIRED_PREFIX: &[&str] = &["Demo", "Features", "Anatomy", "API Reference"];
    for path in FORMS_PATHS {
        let content = fandhe_frontend_docs_site::component_page::generated_content(path)
            .unwrap_or_else(|| panic!("{path} should have generated content"));
        let html = render(&content);
        let headings = h2_texts(&html);
        assert!(
            is_subsequence(REQUIRED_PREFIX, &headings),
            "{path}: expected {REQUIRED_PREFIX:?} as a subsequence of {headings:?}"
        );
        let headings_str: Vec<&str> = headings.iter().map(String::as_str).collect();
        assert!(
            is_str_subsequence(&headings_str, CANONICAL_SECTIONS),
            "{path}: headings {headings:?} must be a subsequence of canonical order {CANONICAL_SECTIONS:?}"
        );
    }
}

/// `site/themes/<kebab>.md` の Forms 33 件が Phase 4 未充填を示す
/// `[!NOTE]` admonition（「Phase 4」文言を含む）を残していないことを検証
/// する（充填したページから admonition を削除する前提、イシュー #945）。
#[test]
fn forms_markdown_sources_do_not_retain_the_phase4_unfilled_marker() {
    for path in FORMS_PATHS {
        let kebab = path.trim_matches('/').rsplit('/').next().unwrap();
        let md_path = repo_root().join(format!("site/themes/{kebab}.md"));
        let src = fs::read_to_string(&md_path)
            .unwrap_or_else(|e| panic!("{} should be readable: {e}", md_path.display()));
        assert!(
            !(src.contains("[!NOTE]") && src.contains("Phase 4")),
            "{}: still contains the Phase 4 unfilled-section admonition",
            md_path.display()
        );
    }
}

/// `needle` が `haystack` の（連続とは限らない）部分列であるかを判定する。
fn is_subsequence(needle: &[&str], haystack: &[String]) -> bool {
    let mut it = haystack.iter();
    needle
        .iter()
        .all(|item| it.any(|candidate| candidate == item))
}

/// [`is_subsequence`] の `&str` 版（両辺 `&[&str]` の比較用）。
fn is_str_subsequence(needle: &[&str], haystack: &[&str]) -> bool {
    let mut it = haystack.iter();
    needle
        .iter()
        .all(|item| it.any(|candidate| candidate == item))
}

const FORMS_PATHS: &[&str] = &[
    "/themes/angle-slider/",
    "/themes/button/",
    "/themes/calendar/",
    "/themes/checkbox/",
    "/themes/checkbox-card/",
    "/themes/checkbox-group/",
    "/themes/color-picker/",
    "/themes/combobox/",
    "/themes/date-input/",
    "/themes/date-picker/",
    "/themes/download-trigger/",
    "/themes/editable/",
    "/themes/field/",
    "/themes/fieldset/",
    "/themes/file-upload/",
    "/themes/image-cropper/",
    "/themes/input/",
    "/themes/listbox/",
    "/themes/native-select/",
    "/themes/number-input/",
    "/themes/password-input/",
    "/themes/pin-input/",
    "/themes/radio-card/",
    "/themes/radio-group/",
    "/themes/rating-group/",
    "/themes/segment-group/",
    "/themes/select/",
    "/themes/signature-pad/",
    "/themes/slider/",
    "/themes/switch/",
    "/themes/tags-input/",
    "/themes/textarea/",
    "/themes/toggle/",
    "/themes/toggle-group/",
];

#[test]
fn data_attrs_and_css_var_tables_are_deterministic_across_repeated_renders() {
    for path in ["/themes/accordion/", "/themes/dialog/"] {
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

/// 部品ページ原稿 `.md` の設計契約（`docs/design/docs-site-component-pages.md`
/// §7a.1）を fail-closed で強制する: 原稿は H1（`# `）+ 導入文のみに保ち、
/// Features/Anatomy/API Reference/Accessibility 等の H2（`## `）節は増やさ
/// ない（それらは `ComponentPageSpec`（Rust）から機械生成される、
/// `crate::build::build_site` が「Markdown 本文 → Rust 生成コンテンツ」の
/// 順で連結する設計）。イシュー #980 は `site/themes/toggle.md`/
/// `toggle-group.md` が #979 の CSS 配線後もこの契約に違反したまま出荷され
/// ていた（Demo/Features/Anatomy/API Reference/Accessibility の重複・
/// 虚偽の「Demo を持たない」注記が残存）ことの是正であり、同種の乖離を
/// 二度と見逃さないための機械ガード。
#[test]
fn component_markdown_sources_have_no_h2_headings() {
    let dir = repo_root().join("site/themes");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("{} should be readable: {e}", dir.display()));
    let mut violations: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.expect("dir entry should be readable");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let src = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} should be readable: {e}", path.display()));
        if src.lines().any(|line| line.starts_with("## ")) {
            violations.push(path.display().to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "the following component markdown sources contain H2 (`## `) headings, \
         violating docs/design/docs-site-component-pages.md §7a.1 (原稿 `.md` は \
         H1 + 導入文のみに保ち、Features 等は `ComponentPageSpec`（Rust）から供給 \
         する): {violations:?}"
    );
}

/// イシュー #996 受け入れ条件: `/themes/tab-nav/` が Demo → Features →
/// Anatomy → API Reference → Examples → Accessibility の 6 節すべてを
/// この順で描画すること。[`overlay_disclosure_pages_include_all_required_sections`]
/// はオーバーレイ部品の明示リストのみを対象とするため、`tab-nav` の
/// `keyboard`/`aria`（Accessibility 節を構成する 2 フィールド）が空だと
/// サイレントに Accessibility 節が省略されてもテストは緑のままという穴が
/// あった。本テストはその穴を塞ぎ、6 節の完全性と順序を直接固定する。
#[test]
fn tab_nav_page_renders_all_six_canonical_sections() {
    const PATH: &str = "/themes/tab-nav/";
    let content = fandhe_frontend_docs_site::component_page::generated_content(PATH)
        .unwrap_or_else(|| panic!("registered path {PATH} must have generated content"));
    let html = render(&content);
    let headings = h2_texts(&html);
    assert_eq!(
        headings,
        vec![
            "Demo".to_string(),
            "Features".to_string(),
            "Anatomy".to_string(),
            "API Reference".to_string(),
            "Examples".to_string(),
            "Accessibility".to_string(),
        ],
        "page {PATH} must render all six canonical sections in order, got {headings:?}"
    );
}

/// イシュー #1022: `component_page::generated_content` は `/primitives/`
/// パスに対して `crate::primitive_showcase`（headless-ui 専用の Demo
/// レジストリ）を照会する。`/themes/accordion/` の Demo（`showcase.rs` 経由）
/// が `/primitives/accordion/` へ漏れないこと（層を跨いだ混入がないこと）を
/// 同一 kebab の 2 パスで対照的に固定する。
#[test]
fn primitives_pages_render_via_primitive_showcase_not_showcase() {
    let themes_html = render(
        &fandhe_frontend_docs_site::component_page::generated_content("/themes/accordion/")
            .expect("/themes/accordion/ must have generated content"),
    );
    let primitives_html = render(
        &fandhe_frontend_docs_site::component_page::generated_content("/primitives/accordion/")
            .expect("/primitives/accordion/ must have generated content"),
    );

    // Themes 側は pre-styled-ui 由来のラッパ class を、Primitives 側は
    // primitives-showcase 由来のラッパ class を持つ（層混同がないことの
    // 直接証拠）。
    assert!(themes_html.contains(r#"class="pre-styled-showcase""#));
    assert!(!themes_html.contains(r#"class="primitives-showcase""#));
    assert!(primitives_html.contains(r#"class="primitives-showcase""#));
    assert!(!primitives_html.contains(r#"class="pre-styled-showcase""#));

    // 両ページとも headless-ui/pre-styled-ui のいずれかの実体マークアップ
    // （`data-scope="accordion"`）を持つ、すなわちどちらも空の Demo では
    // ないことを確認する（非空虚性）。
    assert!(themes_html.contains(r#"data-scope="accordion""#));
    assert!(primitives_html.contains(r#"data-scope="accordion""#));
}

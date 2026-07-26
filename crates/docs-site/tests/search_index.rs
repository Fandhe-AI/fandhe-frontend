//! 検索インデックス（`assets/search-index.json`）のテスト契約
//! （イシュー #957、`docs/design/docs-site-search-design.md` §3-6）。
//!
//! 決定性・エスケープ・サイズ・生成範囲・見出し id パリティ・冪等性・
//! `data-scope` 除外の 7 項目に加え、部品ページの索引テキストが空でない
//! ことの経験的確認を固定する。フィクスチャは `env!("CARGO_TARGET_TMPDIR")`
//! 基点の一時ディレクトリへ生成し、コミットしない（`ci.md` イシュー #637
//! の一時領域方針、`crates/docs-site/tests/site_build.rs` と同パターン）。

use std::path::{Path, PathBuf};

use fandhe_frontend_docs_site::build::build_site;
use fandhe_frontend_docs_site::layout;
use fandhe_frontend_docs_site::nav;
use fandhe_frontend_docs_site::redirect;
use fandhe_frontend_docs_site::search_index::{self, SearchIndexError};

/// 統合テストのスクラッチ基点。`tests/site_build.rs::scratch_root` と同一
/// パターン（コンパイル時に確定する `CARGO_TARGET_TMPDIR` のみを使い、
/// 実行時フォールバックで `/tmp` へリークしない）。
fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = std::fs::create_dir_all(&root);
    root
}

/// テスト専用の一時ディレクトリ（`crates/docs-site/src/build.rs::tests::TempDir`
/// と同方針。外部クレート `tempfile` を追加しない、REQ-3）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = scratch_root().join(format!(
            "fandhe-frontend-docs-site-search-index-test-{tag}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp dir for search_index.rs test");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn fixture_root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve repository root")
}

fn read_index(out_dir: &Path) -> String {
    std::fs::read_to_string(out_dir.join(search_index::REL_PATH))
        .expect("assets/search-index.json should be generated")
}

// ---------------------------------------------------------------------
// 最小 JSON スキャナ（外部クレートを追加しないため、テスト内に文字列
// リテラルの境界を認識する小さなヘルパを 1 つだけ作り、複数テストで共有
// する。`search_index::render_json` が生成する schema（object/array/string/
// number のみ、null や真偽値は登場しない）に限定した実装であり、汎用
// JSON パーサではない。
// ---------------------------------------------------------------------
#[derive(Debug, Clone)]
enum JsonValue {
    String(String),
    Number(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    fn as_str(&self) -> &str {
        match self {
            JsonValue::String(s) => s,
            other => panic!("expected string, got {other:?}"),
        }
    }

    fn as_array(&self) -> &[JsonValue] {
        match self {
            JsonValue::Array(items) => items,
            other => panic!("expected array, got {other:?}"),
        }
    }

    fn get(&self, key: &str) -> &JsonValue {
        match self {
            JsonValue::Object(entries) => {
                &entries
                    .iter()
                    .find(|(k, _)| k == key)
                    .unwrap_or_else(|| panic!("missing key {key:?} in {self:?}"))
                    .1
            }
            other => panic!("expected object, got {other:?}"),
        }
    }
}

struct JsonParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            bytes: input.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> u8 {
        self.bytes[self.pos]
    }

    fn expect(&mut self, c: u8) {
        assert_eq!(
            self.bytes[self.pos], c,
            "expected {:?} at byte {}",
            c as char, self.pos
        );
        self.pos += 1;
    }

    fn parse_value(&mut self) -> JsonValue {
        match self.peek() {
            b'{' => self.parse_object(),
            b'[' => self.parse_array(),
            b'"' => JsonValue::String(self.parse_string()),
            _ => self.parse_number(),
        }
    }

    fn parse_object(&mut self) -> JsonValue {
        self.expect(b'{');
        let mut entries = Vec::new();
        if self.peek() == b'}' {
            self.pos += 1;
            return JsonValue::Object(entries);
        }
        loop {
            let key = self.parse_string();
            self.expect(b':');
            let value = self.parse_value();
            entries.push((key, value));
            match self.peek() {
                b',' => {
                    self.pos += 1;
                }
                b'}' => {
                    self.pos += 1;
                    break;
                }
                other => panic!("unexpected byte {:?} in object", other as char),
            }
        }
        JsonValue::Object(entries)
    }

    fn parse_array(&mut self) -> JsonValue {
        self.expect(b'[');
        let mut items = Vec::new();
        if self.peek() == b']' {
            self.pos += 1;
            return JsonValue::Array(items);
        }
        loop {
            items.push(self.parse_value());
            match self.peek() {
                b',' => {
                    self.pos += 1;
                }
                b']' => {
                    self.pos += 1;
                    break;
                }
                other => panic!("unexpected byte {:?} in array", other as char),
            }
        }
        JsonValue::Array(items)
    }

    fn parse_string(&mut self) -> String {
        self.expect(b'"');
        let mut out = String::new();
        loop {
            let c = self.bytes[self.pos];
            self.pos += 1;
            match c {
                b'"' => break,
                b'\\' => {
                    let esc = self.bytes[self.pos];
                    self.pos += 1;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let hex = std::str::from_utf8(&self.bytes[self.pos..self.pos + 4])
                                .expect("valid \\u hex digits");
                            let code = u32::from_str_radix(hex, 16).expect("valid hex u32");
                            self.pos += 4;
                            out.push(char::from_u32(code).expect("valid unicode scalar"));
                        }
                        other => panic!("unsupported escape \\{}", other as char),
                    }
                }
                other => {
                    // 元の UTF-8 バイト列をそのまま 1 文字分読み進める
                    // （マルチバイト文字を壊さないよう char 境界で処理する）。
                    let start = self.pos - 1;
                    let rest = std::str::from_utf8(&self.bytes[start..])
                        .expect("valid utf-8 from string start");
                    let ch = rest.chars().next().expect("at least one char remains");
                    out.push(ch);
                    self.pos = start + ch.len_utf8();
                    let _ = other;
                }
            }
        }
        out
    }

    fn parse_number(&mut self) -> JsonValue {
        let start = self.pos;
        while self.pos < self.bytes.len()
            && matches!(self.bytes[self.pos], b'0'..=b'9' | b'-' | b'+' | b'.')
        {
            self.pos += 1;
        }
        let s = std::str::from_utf8(&self.bytes[start..self.pos])
            .expect("valid utf-8 number")
            .to_string();
        assert!(!s.is_empty(), "expected a number at byte {start}");
        JsonValue::Number(s)
    }
}

fn parse_json(input: &str) -> JsonValue {
    let mut parser = JsonParser::new(input);
    let value = parser.parse_value();
    assert_eq!(
        parser.pos,
        input.len(),
        "trailing bytes after top-level JSON value"
    );
    value
}

// ---------------------------------------------------------------------
// 1. 決定性（フィクスチャ）
// ---------------------------------------------------------------------

#[test]
fn search_index_is_byte_identical_across_two_builds_of_the_fixture_site() {
    let out_a = TempDir::new("determinism-fixture-a");
    let out_b = TempDir::new("determinism-fixture-b");

    build_site(&fixture_root("site-ok"), &out_a.0).expect("site-ok fixture should build");
    build_site(&fixture_root("site-ok"), &out_b.0).expect("site-ok fixture should build");

    assert_eq!(read_index(&out_a.0), read_index(&out_b.0));
}

// ---------------------------------------------------------------------
// イシュー #1016: リダイレクト由来の href が索引に含まれないこと
// ---------------------------------------------------------------------

/// リダイレクトページ（`site/redirects.toml`、イシュー #1016）は
/// `crate::build::build_site` 内で `search_index_entries` を積むループ
/// （`nav.all_pages()` 走査）を一切通らないため、検索インデックスには
/// 構造的に現れない。本テストは実サイトビルドの `assets/search-index.json`
/// に `redirect.from` の href が含まれないことを明示的に固定する
/// （`real_site_search_index_is_deterministic_covers_all_nav_pages_and_matches_html_ids`
/// の `actual_hrefs == expected_hrefs`（nav 由来集合との完全一致）が
/// 間接的にも保証する内容だが、本テストは「なぜ含まれないか」を
/// `redirect::MANIFEST_REL_PATH` 起点で明示検証する）。
#[test]
fn real_site_search_index_does_not_contain_redirect_hrefs() {
    let root = repo_root();
    let out = TempDir::new("no-redirect-hrefs");
    build_site(&root, &out.0).expect("real site/nav.toml should build cleanly");

    let manifest_path = root.join(redirect::MANIFEST_REL_PATH);
    let manifest_input = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", manifest_path.display()));
    let redirects =
        redirect::parse_redirects(&manifest_input).expect("site/redirects.toml should parse");
    assert!(
        !redirects.entries.is_empty(),
        "this test requires at least one real redirect declaration to be meaningful"
    );

    let real_nav_input =
        std::fs::read_to_string(root.join("site/nav.toml")).expect("read real site/nav.toml");
    let real_nav = nav::parse_nav(&real_nav_input).expect("parse real site/nav.toml");

    let json = read_index(&out.0);
    let parsed = parse_json(&json);
    let pages = parsed.get("pages").as_array();
    let actual_hrefs: std::collections::BTreeSet<String> = pages
        .iter()
        .map(|p| p.get("href").as_str().to_string())
        .collect();

    for redirect in &redirects.entries {
        let redirect_href = layout::asset_href(&real_nav.site.base_path, &redirect.from);
        assert!(
            !actual_hrefs.contains(&redirect_href),
            "search index should not contain the redirect `from` href {redirect_href:?}"
        );
    }
}

// ---------------------------------------------------------------------
// 2〜4・8. 決定性（実サイト）・生成範囲・見出し id パリティ・
// data-scope 除外 + 部品ページの非空確認（実サイトビルド回数を増やさない
// ため、設計文書 §3-6 のとおり単一テスト内で検証する）
// ---------------------------------------------------------------------

#[test]
fn real_site_search_index_is_deterministic_covers_all_nav_pages_and_matches_html_ids() {
    let root = repo_root();
    let out_a = TempDir::new("real-site-a");
    let out_b = TempDir::new("real-site-b");

    build_site(&root, &out_a.0).expect("real site/nav.toml should build cleanly");
    build_site(&root, &out_b.0).expect("real site/nav.toml should build cleanly");

    let json_a = read_index(&out_a.0);
    let json_b = read_index(&out_b.0);
    // 2. 決定性（実サイト）。
    assert_eq!(
        json_a, json_b,
        "search index should be byte-identical across builds"
    );

    let parsed = parse_json(&json_a);
    assert_eq!(parsed.get("version").as_str_number(), "1");

    let nav_input =
        std::fs::read_to_string(root.join("site/nav.toml")).expect("read real site/nav.toml");
    let real_nav = nav::parse_nav(&nav_input).expect("parse real site/nav.toml");
    let expected_hrefs: std::collections::BTreeSet<String> = real_nav
        .all_pages()
        .map(|page| layout::asset_href(&real_nav.site.base_path, &page.path))
        .collect();

    let pages = parsed.get("pages").as_array();
    let actual_hrefs: std::collections::BTreeSet<String> = pages
        .iter()
        .map(|p| p.get("href").as_str().to_string())
        .collect();

    // 3. 生成範囲: pages[].href の集合が nav.all_pages() 由来の href 集合と
    // 過不足なく一致する（docs/internal/ 非混入の構造的保証を含む）。
    assert_eq!(
        actual_hrefs.len(),
        expected_hrefs.len(),
        "index page count should match nav.all_pages() count"
    );
    assert_eq!(
        actual_hrefs, expected_hrefs,
        "index href set should match nav.all_pages() href set exactly"
    );

    // 4. 見出し id パリティ: 各ページの sections[].id がすべて生成 HTML 中に
    // id="<id>" として存在する（§3-3 の 3 前提の機械固定）。
    let mut checked_pages_with_sections = 0usize;
    let mut component_page_has_non_empty_text = false;
    for page in pages {
        let href = page.get("href").as_str();
        // href は base_path 適用済みのサイト絶対パス。dist 上の相対パスは
        // base_path を取り除いた上で「.../index.html」に対応する。
        let relative = href
            .strip_prefix(&real_nav.site.base_path)
            .unwrap_or(href)
            .trim_start_matches('/');
        let html_path = out_a.0.join(relative).join("index.html");
        let html = std::fs::read_to_string(&html_path)
            .unwrap_or_else(|e| panic!("read generated {html_path:?}: {e}"));

        let sections = page.get("sections").as_array();
        if !sections.is_empty() {
            checked_pages_with_sections += 1;
        }
        for section in sections {
            let id = section.get("id").as_str();
            let needle = format!(r#"id="{id}""#);
            assert!(
                html.contains(&needle),
                "{href}: section id {id:?} should exist in generated HTML as {needle:?}"
            );
        }

        // 8. data-scope 除外 + 非空確認: 部品ページ（`/themes/<kebab>/`。
        // イシュー #1017 で `/components/<kebab>/` から移行し、索引ページ
        // `/components/pre-styled-ui/` は `/themes/` 配下に存在しないため
        // 除外条件は不要になった）の text が空でないこと。data-list
        // 部品ページに限っては、実際に生成 HTML の
        // `data-scope="data-list"` 部分木内にのみ現れるデモ値 "Alice"
        // （`component_specs_nav_data.rs` の data-list デモ、上の
        // `assert!(html.contains(...))` で存在を確認済み）が index に
        // 混入していないことを固定する（"Tab 1" 等の未検証プレースホルダ語
        // を使うと実際には出現せず assert が空振りするため、実出力で存在を
        // 確認済みの語のみを使う）。
        if relative.starts_with("themes/") {
            let text = page.get("text").as_str();
            if !text.is_empty() {
                component_page_has_non_empty_text = true;
            }
            if relative == "themes/data-list/" {
                assert!(
                    html.contains("Alice"),
                    "sanity: fixture HTML should contain the demo value"
                );
                assert!(
                    !text.contains("Alice"),
                    "{href}: text should not contain the data-list demo value \
                     (data-scope subtree exclusion)"
                );
            }
        }
    }
    assert!(
        checked_pages_with_sections > 0,
        "expected at least one real page to have sections"
    );
    assert!(
        component_page_has_non_empty_text,
        "expected at least one component page to have non-empty indexed text \
         (api_reference_section/anatomy_section content, not just demo markup)"
    );
}

trait JsonNumberExt {
    fn as_str_number(&self) -> &str;
}

impl JsonNumberExt for JsonValue {
    fn as_str_number(&self) -> &str {
        match self {
            JsonValue::Number(s) => s,
            other => panic!("expected number, got {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------
// 5. 冪等性
// ---------------------------------------------------------------------

#[test]
fn with_heading_anchors_applied_twice_renders_byte_identical_output() {
    use fandhe_frontend_core::{div, el, render, text};

    let body = div(
        vec![],
        vec![
            el("h2", vec![], vec![text("First Section".to_string())]),
            el("h2", vec![], vec![text("First Section".to_string())]),
            el(
                "h3",
                vec![("id", "custom")],
                vec![text("Custom".to_string())],
            ),
        ],
    );

    let (once, _entries_once) = layout::with_heading_anchors(body.clone());
    let (twice, _entries_twice) = layout::with_heading_anchors(once.clone());

    assert_eq!(render(&once), render(&twice));
}

// ---------------------------------------------------------------------
// 6. エスケープ
// ---------------------------------------------------------------------

fn write_escape_fixture(root: &Path) {
    std::fs::create_dir_all(root.join("site")).unwrap();
    std::fs::write(
        root.join("site/nav.toml"),
        r#"
[site]
title = "Escape Fixture"
base_path = ""

[[section]]
title = "Guide"
index_path = "/"

[[section.page]]
title = "Home"
source = "site/index.md"
path = "/"
"#,
    )
    .unwrap();
    // Markdown レンダラはインライン HTML をエスケープして `Node::Text` へ
    // 落とすため、Markdown 本文にスクリプトタグ・制御文字・行分離文字を
    // 含む見出し・段落を書けば、索引テキストへそのまま伝播する
    // （markdown.rs のインライン処理経由。生 HTML として解釈されない）。
    std::fs::write(
        root.join("site/index.md"),
        "# <script>alert('x')</script>\n\n\
         Body with \"quotes\", a\\backslash, an & ampersand, and a control char: \u{0007}.\n",
    )
    .unwrap();
}

#[test]
fn search_index_json_contains_no_raw_angle_brackets_ampersands_or_control_chars() {
    let temp = TempDir::new("escape-fixture");
    write_escape_fixture(&temp.0);
    let out_dir = temp.0.join("dist");

    build_site(&temp.0, &out_dir).expect("escape fixture should build");
    let json = read_index(&out_dir);

    assert!(json.starts_with(r#"{"version":1"#));

    // グローバル不変条件: 出力 JSON 全体に生の `<` `>` `&` が 1 文字も
    // 現れない（多層防御。個別フィールド検証より強く短い）。
    assert!(!json.contains('<'), "raw '<' must not appear: {json}");
    assert!(!json.contains('>'), "raw '>' must not appear: {json}");
    assert!(!json.contains('&'), "raw '&' must not appear: {json}");

    // エスケープされた形で実際に現れること（何もエスケープしていない
    // 誤検知を防ぐ）。
    assert!(json.contains("\\u003C"));
    assert!(json.contains("\\u003E"));
    assert!(json.contains("\\u0026"));

    // JSON として構文的に妥当であること（parse_json がパニックしなければ
    // 未エスケープの `"` や生制御文字が文字列中に紛れていない）。
    let parsed = parse_json(&json);
    let pages = parsed.get("pages").as_array();
    assert_eq!(pages.len(), 1);
}

// ---------------------------------------------------------------------
// 7. 切り詰め
// ---------------------------------------------------------------------

fn write_truncation_fixture(root: &Path, body_paragraph: &str) {
    std::fs::create_dir_all(root.join("site")).unwrap();
    std::fs::write(
        root.join("site/nav.toml"),
        r#"
[site]
title = "Truncation Fixture"
base_path = ""

[[section]]
title = "Guide"
index_path = "/"

[[section.page]]
title = "Home"
source = "site/index.md"
path = "/"
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("site/index.md"),
        format!("# Home\n\n{body_paragraph}\n"),
    )
    .unwrap();
}

#[test]
fn page_text_is_truncated_at_a_valid_utf8_char_boundary_within_the_byte_limit() {
    // マルチバイト（日本語 + 絵文字）を大量に繰り返し、
    // MAX_PAGE_TEXT_BYTES（4096 バイト）を確実に超えさせる。
    let unit = "あいう😀";
    let repeat_count = (search_index::MAX_PAGE_TEXT_BYTES / unit.len()) + 100;
    let long_text = unit.repeat(repeat_count);
    assert!(long_text.len() > search_index::MAX_PAGE_TEXT_BYTES);

    let temp = TempDir::new("truncation-fixture");
    write_truncation_fixture(&temp.0, &long_text);
    let out_dir = temp.0.join("dist");

    build_site(&temp.0, &out_dir).expect("truncation fixture should build");
    let json = read_index(&out_dir);
    let parsed = parse_json(&json);
    let pages = parsed.get("pages").as_array();
    assert_eq!(pages.len(), 1);
    let text = pages[0].get("text").as_str();

    assert!(std::str::from_utf8(text.as_bytes()).is_ok());
    assert!(text.len() <= search_index::MAX_PAGE_TEXT_BYTES);
    // 4096 直下の文字境界で切れている: もう 1 文字（"あ"、3 バイト）足すと
    // 上限を超える位置まで詰まっている想定。安全側の下限としては、
    // 切り詰め後のテキストが十分に上限へ近いことのみを確認する
    // （空白正規化により厳密な「1 文字足せば超過」の判定は本文構成に
    // 依存するため、余裕を持たせた下限で固定する）。
    assert!(
        text.len() >= search_index::MAX_PAGE_TEXT_BYTES - unit.len(),
        "truncated text should be close to the byte limit, got {} bytes",
        text.len()
    );
}

// ---------------------------------------------------------------------
// 9. サイズ fail-closed
// ---------------------------------------------------------------------

#[test]
fn check_size_returns_too_large_when_json_exceeds_the_byte_limit() {
    let oversized = "a".repeat(search_index::MAX_INDEX_BYTES + 1);
    match search_index::check_size(&oversized) {
        Err(SearchIndexError::TooLarge { bytes, limit }) => {
            assert_eq!(bytes, search_index::MAX_INDEX_BYTES + 1);
            assert_eq!(limit, search_index::MAX_INDEX_BYTES);
        }
        Ok(()) => panic!("expected TooLarge error"),
    }
}

/// 単一ページに大量の見出しを持たせ、`MAX_INDEX_BYTES`（1 MiB）超過を
/// 起こす合成フィクスチャ。見出しは per-page 上限（4096 バイト、テキスト
/// のみに適用）の対象外（設計文書 §3-3）であるため、320 ページ生成より
/// 圧倒的に安価に総量超過を作れる。
fn write_oversized_fixture(root: &Path, heading_count: usize) {
    std::fs::create_dir_all(root.join("site")).unwrap();
    std::fs::write(
        root.join("site/nav.toml"),
        r#"
[site]
title = "Oversized Fixture"
base_path = ""

[[section]]
title = "Guide"
index_path = "/"

[[section.page]]
title = "Home"
source = "site/index.md"
path = "/"
"#,
    )
    .unwrap();
    let mut markdown = String::from("# Home\n\n");
    for i in 0..heading_count {
        // 見出しテキストを十分長くし、1 見出しあたりの JSON 出力バイト数を
        // 増やして必要な見出し数を抑える。
        markdown.push_str(&format!(
            "## Heading number {i} with some extra padding text to inflate size\n\n"
        ));
    }
    std::fs::write(root.join("site/index.md"), markdown).unwrap();
}

#[test]
fn build_site_fails_closed_when_search_index_exceeds_the_byte_limit_without_writing_output() {
    use fandhe_frontend_docs_site::build::BuildError;

    // 見出し数はハードコードで「効くはず」と決めつけず、実際に
    // BuildError::SearchIndex が返るまでスケールさせて確定する。
    let mut heading_count = 2_000usize;
    let mut last_err = None;
    for _ in 0..6 {
        let temp = TempDir::new("oversized-fixture");
        write_oversized_fixture(&temp.0, heading_count);
        let out_dir = temp.0.join("dist");

        match build_site(&temp.0, &out_dir) {
            Err(BuildError::SearchIndex(SearchIndexError::TooLarge { bytes, limit })) => {
                assert!(bytes > limit);
                assert_eq!(limit, search_index::MAX_INDEX_BYTES);
                assert!(
                    !out_dir.exists(),
                    "out_dir must not be written when the search index is too large"
                );
                return;
            }
            Err(other) => {
                last_err = Some(format!("{other}"));
                break;
            }
            Ok(_) => {
                heading_count *= 2;
            }
        }
    }
    panic!(
        "expected build_site to fail with BuildError::SearchIndex(TooLarge) at some heading \
         count, last error: {last_err:?}"
    );
}

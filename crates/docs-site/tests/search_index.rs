//! 検索インデックス（マニフェスト `assets/search-index.json` + セクション別
//! `assets/search-index/<slug>.json`、イシュー #957 / #3173）のテスト契約
//! （`docs/design/docs-site-search-design.md` §3-6・§10-15）。
//!
//! 決定性・エスケープ・サイズ・生成範囲・見出し id パリティ・冪等性・
//! `data-scope` 除外の 7 項目に加え、部品ページの索引テキストが空でない
//! ことの経験的確認、およびセクション分割後の契約（マニフェストと
//! `site/nav.toml` の `[[section]]` の過不足なき一致・per-file 上限の
//! fail-closed・旧単一ファイル形式を生成しないこと）を固定する。フィクスチャは `env!("CARGO_TARGET_TMPDIR")`
//! 基点の一時ディレクトリへ生成し、コミットしない（`ci.md` イシュー #637
//! の一時領域方針、`crates/docs-site/tests/site_build.rs` と同パターン）。

use std::path::{Path, PathBuf};

use fandhe_frontend_docs_site::build::{build_site, BuildError};
use fandhe_frontend_docs_site::layout;
use fandhe_frontend_docs_site::nav;
use fandhe_frontend_docs_site::redirect;
use fandhe_frontend_docs_site::search_index::{self, SearchIndexError};

/// fixture 上のビルド用。本番登録表は実 `site/nav.toml` のページを前提にするため、
/// 合成 nav では空の登録表を使う（イシュー #3616）。
fn build_fixture(
    repo_root: &Path,
    out_dir: &Path,
) -> Result<fandhe_frontend_docs_site::build::BuildReport, BuildError> {
    fandhe_frontend_docs_site::build::build_site_with(
        repo_root,
        out_dir,
        &fandhe_frontend_docs_site::page_sections::EMPTY_REGISTRY,
    )
}

#[path = "support/shared_site.rs"]
mod shared_site;

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

/// マニフェスト（`assets/search-index.json`）の生 JSON。
fn read_manifest(out_dir: &Path) -> String {
    std::fs::read_to_string(out_dir.join(search_index::REL_PATH))
        .expect("assets/search-index.json should be generated")
}

/// マニフェストが列挙する順に `(dist 相対パス, 生 JSON)` を返す
/// （先頭はマニフェスト自身）。決定性の比較・エスケープの全域検査に使う。
fn read_index_files(out_dir: &Path) -> Vec<(String, String)> {
    let manifest = read_manifest(out_dir);
    let parsed = parse_json(&manifest);
    let base_path = parsed.get("base_path").as_str().to_string();
    let mut files = vec![(search_index::REL_PATH.to_string(), manifest.clone())];
    for section in parsed.get("sections").as_array() {
        let href = section.get("href").as_str();
        let relative = href
            .strip_prefix(&base_path)
            .unwrap_or(href)
            .trim_start_matches('/')
            .to_string();
        let json = std::fs::read_to_string(out_dir.join(&relative))
            .unwrap_or_else(|e| panic!("section index {relative} should be generated: {e}"));
        files.push((relative, json));
    }
    files
}

/// 全セクションファイルの `pages` をマニフェスト順に連結して返す
/// （検索 UI が `Promise.all` 後に行う結合と同じ順序）。
fn read_all_pages(out_dir: &Path) -> Vec<JsonValue> {
    let mut pages = Vec::new();
    for (relative, json) in read_index_files(out_dir).into_iter().skip(1) {
        let parsed = parse_json(&json);
        assert_eq!(
            parsed.get("version").as_str_number(),
            search_index::SCHEMA_VERSION.to_string(),
            "{relative}: section file schema version"
        );
        pages.extend(parsed.get("pages").as_array().iter().cloned());
    }
    pages
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

    fn has_key(&self, key: &str) -> bool {
        match self {
            JsonValue::Object(entries) => entries.iter().any(|(k, _)| k == key),
            other => panic!("expected object, got {other:?}"),
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
                    // 先頭バイトから当該 1 文字のバイト長を求め、その範囲だけを
                    // `from_utf8` で検証する。残りバッファ全体を毎回検証する
                    // 旧実装は 1 文字ごとに O(n) となり、実サイトの検索
                    // インデックス（約 1MB）のパースに数分を要していた
                    // （イシュー #2299 で判明した二乗時間の原因）。
                    let start = self.pos - 1;
                    let len = match other {
                        0x00..=0x7F => 1,
                        0xC0..=0xDF => 2,
                        0xE0..=0xEF => 3,
                        _ => 4,
                    };
                    let ch = std::str::from_utf8(&self.bytes[start..start + len])
                        .expect("valid utf-8 char in string")
                        .chars()
                        .next()
                        .expect("at least one char remains");
                    out.push(ch);
                    self.pos = start + len;
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

    build_fixture(&fixture_root("site-ok"), &out_a.0).expect("site-ok fixture should build");
    build_fixture(&fixture_root("site-ok"), &out_b.0).expect("site-ok fixture should build");

    assert_eq!(read_index_files(&out_a.0), read_index_files(&out_b.0));
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
    // 共有ビルド（イシュー #2299）: 読み取り専用のため実サイトビルドを
    // 使い回す。
    let shared = shared_site::real_site();
    let root = shared_site::repo_root();
    let out_dir = shared.out_dir.as_path();

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

    let pages = read_all_pages(out_dir);
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
    // 決定性検証のため、共有ビルド（イシュー #2299）を一方の入力に使い、
    // 比較対象のもう一方は独自にビルドする（shared_site モジュール doc の
    // 契約どおり）。
    let shared = shared_site::real_site();
    let root = shared_site::repo_root();
    let out_a_dir = shared.out_dir.as_path();
    let out_b = TempDir::new("real-site-b");

    build_site(&root, &out_b.0).expect("real site/nav.toml should build cleanly");

    // 2. 決定性（実サイト）: マニフェスト + 全セクションファイルがバイト一致。
    assert_eq!(
        read_index_files(out_a_dir),
        read_index_files(&out_b.0),
        "search index files should be byte-identical across builds"
    );

    let manifest = parse_json(&read_manifest(out_a_dir));
    assert_eq!(
        manifest.get("version").as_str_number(),
        search_index::SCHEMA_VERSION.to_string()
    );

    let nav_input =
        std::fs::read_to_string(root.join("site/nav.toml")).expect("read real site/nav.toml");
    let real_nav = nav::parse_nav(&nav_input).expect("parse real site/nav.toml");
    let expected_hrefs: std::collections::BTreeSet<String> = real_nav
        .all_pages()
        .map(|page| layout::asset_href(&real_nav.site.base_path, &page.path))
        .collect();

    let pages = read_all_pages(out_a_dir);
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
    for page in &pages {
        let href = page.get("href").as_str();
        // href は base_path 適用済みのサイト絶対パス。dist 上の相対パスは
        // base_path を取り除いた上で「.../index.html」に対応する。
        let relative = href
            .strip_prefix(&real_nav.site.base_path)
            .unwrap_or(href)
            .trim_start_matches('/');
        let html_path = out_a_dir.join(relative).join("index.html");
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
        // イシュー #1017 で `/components/<kebab>/` から移行した）の text が
        // 空でないこと。イシュー #1018 で索引ページも `/themes/` 配下
        // （`/themes/` 自身）へ移設されたため `relative.starts_with("themes/")`
        // には索引ページも含まれるが、索引ページも本文が非空であり
        // このアサーションは変わらず成立する（除外条件の追加は不要）。data-list
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

// ---------------------------------------------------------------------
// イシュー #2862（§10-13）: Blocks ページのフェンスコードブロック本文を
// 索引テキストから除外する恒久対処の回帰（`search_index::page_entry` の
// `exclude_code_blocks` 配線が `crate::build::build_site` から実サイトの
// Blocks ページへ正しく届いていることを固定する）。
// ---------------------------------------------------------------------

#[test]
fn real_site_search_index_excludes_blocks_page_rust_code_fence_but_keeps_prose() {
    // `pricing-comparison-table` block（`crates/docs-site/src/blocks/
    // marketing/pricing/pricing_comparison_table.rs`）の実装のみに現れる
    // 識別子 `COL_COUNT_STR` は、除外が正しく効いていれば索引から消える。
    // 一方でページ見出し（Markdown 原稿由来のプレーンテキスト）は
    // フェンス除外の対象外であり、引き続き索引に残る。
    let shared = shared_site::real_site();

    let pages = read_all_pages(&shared.out_dir);

    let page = pages
        .iter()
        .find(|p| {
            p.get("href")
                .as_str()
                .ends_with("/blocks/pricing-comparison-table/")
        })
        .expect("pricing-comparison-table block page should be indexed");
    let text = page.get("text").as_str();

    assert!(
        !text.contains("COL_COUNT_STR"),
        "Blocks page indexed text should not contain Rust code fence identifiers \
         after code-block exclusion, got: {text}"
    );
    assert!(
        text.contains("Rust コード") || text.contains("使用部品"),
        "Blocks page indexed text should still contain its non-code prose sections: {text}"
    );
}

// ---------------------------------------------------------------------
// イシュー #1078: コードブロックのシンタックスハイライト導入後も検索索引の
// 到達性が失われないことの回帰（実装計画 §2.7）。
// ---------------------------------------------------------------------

#[test]
fn real_site_search_index_still_contains_code_block_keywords_after_highlighting() {
    // `crate::highlight::highlight_children` は Rust フェンス本文へ
    // `<span class="token-*">` を挿入する。`search_index::collect_text_into`
    // は `is_token_span` によりこの span を要素境界の空白挿入から除外する
    // （span 化前と同じ「単一の Text ノード」相当の連結結果になる）ため、
    // 単語自体が欠落・分断されることはない。この回帰テストは「色分け導入
    // により検索から消える」退行を機械固定する（`fn`/`use`/`user_badge`
    // という単独の語のみを見るため、キーワード/リテラルに隣接する非空白
    // 文字を含む語句の分断は
    // [`real_site_search_index_keeps_words_adjacent_to_highlight_token_spans_contiguous`]
    // が別途検証する）。
    // 共有ビルド（イシュー #2299）: 読み取り専用のため実サイトビルドを
    // 使い回す。
    let shared = shared_site::real_site();

    let pages = read_all_pages(&shared.out_dir);

    // docs/guides/component-authoring.md（`site/nav.toml` の
    // `/guides/component-authoring/`）は ```rust フェンスを複数含み、識別子
    // `user_badge`・キーワード `fn`/`use` を含む（本文実測）。
    let page = pages
        .iter()
        .find(|p| {
            p.get("href")
                .as_str()
                .ends_with("/guides/component-authoring/")
        })
        .expect("component-authoring page should be indexed");
    let text = page.get("text").as_str();

    for needle in ["fn", "use", "user_badge"] {
        assert!(
            text.contains(needle),
            "indexed text for component-authoring should still contain {needle:?} \
             after fence highlighting: {text}"
        );
    }
}

/// [`real_site_search_index_still_contains_code_block_keywords_after_highlighting`]
/// が検証する `fn`/`use`/`user_badge` は、キーワード・数値トークンが
/// span 化されても前後に空白しか隣接しないため「単独で分断されない語」
/// であり、意図した退行クラス（キーワード/リテラルに**隣接する非空白
/// 文字を含む**語句、例: `crate::highlight` の `crate`、`foo(1)` の `1`）
/// を検出できない（レビュー指摘、イシュー #1078）。
///
/// 本テストは `docs/**` の実文書（将来の編集で内容が変わり得る）ではなく、
/// 専用フィクスチャ（`tests/fixtures/site-highlighted-code/`）の Rust
/// フェンスに対して、`crate::highlight::highlight_children` →
/// `crate::markdown::parse_fence` → [`search_index::page_entry`] のパイプ
/// ライン全体を通した索引テキストで隣接語句が連続していることを固定する。
#[test]
fn real_site_search_index_keeps_words_adjacent_to_highlight_token_spans_contiguous() {
    let out = TempDir::new("code-block-adjacent-words");
    build_fixture(&fixture_root("site-highlighted-code"), &out.0)
        .expect("site-highlighted-code fixture should build cleanly");

    let pages = read_all_pages(&out.0);
    let page = pages
        .iter()
        .find(|p| p.get("href").as_str().ends_with("/fixture-base/"))
        .expect("fixture home page should be indexed");
    let text = page.get("text").as_str();

    // `crate` は token-keyword span で包まれるが、直後の `::highlight` は
    // span を持たない兄弟トークンとして続く。要素境界の空白挿入が
    // span だけを透過しない実装だと `"crate ::highlight"` に分断される。
    assert!(
        text.contains("crate::highlight"),
        "\"crate::highlight\" should remain contiguous in the index text \
         even though `crate` is wrapped in a token-keyword span: {text}"
    );

    // `1` は token-number span で包まれるが、前後の `foo(`/`)` は span を
    // 持たない。span 前後どちらの境界も透過しない実装だと
    // `"foo( 1 )"` のように両側へ空白が入る。
    assert!(
        text.contains("foo(1)"),
        "\"foo(1)\" should remain contiguous in the index text even though \
         the literal `1` is wrapped in a token-number span: {text}"
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

    build_fixture(&temp.0, &out_dir).expect("escape fixture should build");
    let files = read_index_files(&out_dir);
    assert_eq!(files.len(), 2, "manifest + 1 section file");

    // グローバル不変条件: マニフェスト・セクションファイルのいずれにも
    // 生の `<` `>` `&` が 1 文字も現れない（多層防御。個別フィールド検証
    // より強く短い）。
    for (relative, json) in &files {
        assert!(json.starts_with(r#"{"version":2"#), "{relative}: {json}");
        assert!(
            !json.contains('<'),
            "{relative}: raw '<' must not appear: {json}"
        );
        assert!(
            !json.contains('>'),
            "{relative}: raw '>' must not appear: {json}"
        );
        assert!(
            !json.contains('&'),
            "{relative}: raw '&' must not appear: {json}"
        );
        // JSON として構文的に妥当であること（parse_json がパニックしなければ
        // 未エスケープの `"` や生制御文字が文字列中に紛れていない）。
        parse_json(json);
    }

    // エスケープされた形で実際に現れること（何もエスケープしていない
    // 誤検知を防ぐ）。フィクスチャの本文はセクションファイル側にある。
    let section_json = &files[1].1;
    assert!(section_json.contains("\\u003C"));
    assert!(section_json.contains("\\u003E"));
    assert!(section_json.contains("\\u0026"));

    let pages = read_all_pages(&out_dir);
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
    // MAX_PAGE_TEXT_BYTES を確実に超えさせる。
    let unit = "あいう😀";
    let repeat_count = (search_index::MAX_PAGE_TEXT_BYTES / unit.len()) + 100;
    let long_text = unit.repeat(repeat_count);
    assert!(long_text.len() > search_index::MAX_PAGE_TEXT_BYTES);

    let temp = TempDir::new("truncation-fixture");
    write_truncation_fixture(&temp.0, &long_text);
    let out_dir = temp.0.join("dist");

    build_fixture(&temp.0, &out_dir).expect("truncation fixture should build");
    let pages = read_all_pages(&out_dir);
    assert_eq!(pages.len(), 1);
    let text = pages[0].get("text").as_str();

    assert!(std::str::from_utf8(text.as_bytes()).is_ok());
    assert!(text.len() <= search_index::MAX_PAGE_TEXT_BYTES);
    // 上限直下の文字境界で切れている: もう 1 文字（"あ"、3 バイト）足すと
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
    let oversized = "a".repeat(search_index::MAX_SECTION_INDEX_BYTES + 1);
    match search_index::check_size("guide", &oversized) {
        Err(SearchIndexError::TooLarge {
            section,
            bytes,
            limit,
        }) => {
            assert_eq!(section, "guide");
            assert_eq!(bytes, search_index::MAX_SECTION_INDEX_BYTES + 1);
            assert_eq!(limit, search_index::MAX_SECTION_INDEX_BYTES);
        }
        Err(other) => panic!("expected TooLarge error, got {other}"),
        Ok(()) => panic!("expected TooLarge error"),
    }
}

/// 単一ページに大量の見出しを持たせ、セクションファイルの
/// `MAX_SECTION_INDEX_BYTES` 超過を
/// 起こす合成フィクスチャ。見出しは per-page 上限（`MAX_PAGE_TEXT_BYTES`、
/// テキストのみに適用）の対象外（設計文書 §3-3）であるため、320 ページ生成より
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
    // 見出し数はハードコードで「効くはず」と決めつけず、実際に
    // BuildError::SearchIndex が返るまでスケールさせて確定する。
    let mut heading_count = 2_000usize;
    let mut last_err = None;
    for _ in 0..6 {
        let temp = TempDir::new("oversized-fixture");
        write_oversized_fixture(&temp.0, heading_count);
        let out_dir = temp.0.join("dist");

        match build_fixture(&temp.0, &out_dir) {
            Err(BuildError::SearchIndex(SearchIndexError::TooLarge {
                section,
                bytes,
                limit,
            })) => {
                assert_eq!(section, "guide");
                assert!(bytes > limit);
                assert_eq!(limit, search_index::MAX_SECTION_INDEX_BYTES);
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

/// `MAX_SECTION_INDEX_BYTES` を機械的に再引き上げすることを牽制するための
/// 固定ピン（イシュー #3173、`docs/design/docs-site-search-design.md` §10-15）。
///
/// 旧 `MAX_INDEX_BYTES`（1 ファイル全体上限）は引き上げを繰り返した末に
/// 「これ以上引き上げない」ハードルールへ至った（§10-6）。セクション分割後の
/// per-file 上限も同じ轍を踏まないよう、本テストが FAIL した場合は値を書き
/// 換える前に §10-15 の再評価トリガー（超過セクションのさらなる分割、Blocks
/// なら `crate::blocks::BlockSection` 単位）を先に検討すること
/// （`crates/xtask/tests/` `SKIPPED_ALLOWLIST` と同種の意図的な摩擦点）。
#[test]
fn max_section_index_bytes_is_pinned_to_the_issue_3173_rationale() {
    assert_eq!(
        search_index::MAX_SECTION_INDEX_BYTES,
        2_621_440,
        "MAX_SECTION_INDEX_BYTES の機械的な再引き上げは牽制されている。超過した \
         セクションをさらに分割することを先に検討すること \
         （docs/design/docs-site-search-design.md §10-15 参照）。"
    );
}

// ---------------------------------------------------------------------
// イシュー #3173: セクション分割後の構造契約
// ---------------------------------------------------------------------

/// マニフェストの `sections[]` が `site/nav.toml` の `[[section]]` と宣言順
/// まで含めて過不足なく一致し、各セクションファイルが当該セクション配下の
/// ページ（`Section::all_pages` の順）だけを持つことを固定する
/// （レジストリ駆動: セクション追加時に build.rs / script.rs へ分岐が増えて
/// いないことの機械的な裏付け）。あわせて、旧単一ファイル形式（マニフェスト
/// パスに `pages` を直接持つ形）を生成していないことを固定する。
#[test]
fn real_site_search_index_manifest_matches_nav_sections_and_each_file_holds_its_section_pages() {
    let shared = shared_site::real_site();
    let root = shared_site::repo_root();
    let out_dir = shared.out_dir.as_path();

    let nav_input =
        std::fs::read_to_string(root.join("site/nav.toml")).expect("read real site/nav.toml");
    let real_nav = nav::parse_nav(&nav_input).expect("parse real site/nav.toml");

    let manifest = parse_json(&read_manifest(out_dir));
    assert!(
        !manifest.has_key("pages"),
        "manifest must not carry page entries (legacy single-file form)"
    );
    let manifest_sections = manifest.get("sections").as_array();
    let manifest_titles: Vec<&str> = manifest_sections
        .iter()
        .map(|s| s.get("title").as_str())
        .collect();
    let nav_titles: Vec<&str> = real_nav.sections.iter().map(|s| s.title.as_str()).collect();
    assert_eq!(
        manifest_titles, nav_titles,
        "manifest sections should equal nav.toml [[section]] titles in declaration order"
    );

    let files = read_index_files(out_dir);
    assert_eq!(files.len(), 1 + real_nav.sections.len());
    for (section, (relative, json)) in real_nav.sections.iter().zip(files.iter().skip(1)) {
        assert_eq!(
            *relative,
            search_index::section_rel_path(&search_index::section_slug(&section.title)),
            "section file path should be derived from the section title slug"
        );
        assert!(
            json.len() <= search_index::MAX_SECTION_INDEX_BYTES,
            "{relative}: {} bytes exceeds MAX_SECTION_INDEX_BYTES",
            json.len()
        );
        let expected: Vec<String> = section
            .all_pages()
            .map(|page| layout::asset_href(&real_nav.site.base_path, &page.path))
            .collect();
        let actual: Vec<String> = parse_json(json)
            .get("pages")
            .as_array()
            .iter()
            .map(|p| p.get("href").as_str().to_string())
            .collect();
        assert_eq!(
            actual, expected,
            "{relative}: pages should be exactly this section's pages in Section::all_pages order"
        );
    }
}

/// イシュー #3600: 和文のソフト改行は索引テキストにも空白を入れない。
#[test]
fn page_entry_text_has_no_space_at_japanese_soft_break() {
    use fandhe_frontend_core::{el, Node};
    let nodes =
        fandhe_frontend_docs_site::markdown::render_markdown("AI 時代の\nセキュリティリスク");
    let body: Node = el("div", vec![], nodes);
    let entry = search_index::page_entry("/x/", "t", &body, false);
    assert!(
        entry.text.contains("AI 時代のセキュリティリスク"),
        "{}",
        entry.text
    );
    assert!(!entry.text.contains("時代の セキュリティ"));
}

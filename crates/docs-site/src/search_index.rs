//! ビルド時検索インデックス（`assets/search-index.json`）の生成（イシュー #957）。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::build::build_site`] がページループ内で [`page_entry`] を各ページから
//! 収集し、`ssg::generate_pages` による書き出しより前に [`render_json`] +
//! [`check_size`] を完了させてから `assets/search-index.json` として書き出す
//! （`crate::build` モジュール doc の処理順を参照）。生成した JSON は
//! `#958`（検索 UI、`crate::script` へ第 3 の IIFE として追加予定）が
//! `fetch()` で遅延読み込みする契約であり、本モジュールは HTML へのインライン化
//! を一切行わない（不変条件は下記参照）。
//!
//! 設計の正は `docs/design/docs-site-search-design.md` §3 であり、本モジュールは
//! 同文書から逸脱しない（`MAX_PAGE_TEXT_BYTES` 等の定数値を含む）。
//!
//! # セキュリティ不変条件（REQ-1、`.claude/rules/coding-rust.md`）
//!
//! - インデックスは常に**独立ファイルとして fetch される**。HTML への埋め込み
//!   （インライン `<script>`・`data-*` 属性への本文格納）は禁止する。
//! - JSON シリアライズは手書き（[`escape_json_string`]）で行い、外部クレートを
//!   追加しない（`crates/docs-site` は内部 path 依存のみ、REQ-3）。
//! - `"` `\` に加え制御文字（`U+0000`〜`U+001F`）・`<` `>` `&` `U+2028`/`U+2029`
//!   をエスケープする多層防御（この JSON が将来 `<script>` へインライン化される
//!   変更が入っても `</script>` 断片が生成されない構造的防御。
//!   `crate::script::is_escape_safe` の思想と揃える）。
//! - [`page_entry`] のテキスト抽出は [`Node::RawHtml`] を連結しない
//!   （`crate::layout::extract_text` と同方針。docs-site は `raw_html()` を
//!   使わない方針だが防御的に実装する）。

use std::fmt;

use fandhe_frontend_core::Node;

use crate::layout;

/// `assets/search-index.json` の `out_dir` 起点相対パス（`crate::build` が
/// 書き出し先の単一実装点として使う）。
pub const REL_PATH: &str = "assets/search-index.json";

/// インデックス JSON のスキーマバージョン。破壊的変更時にインクリメントする。
/// JS 側（#958）は `version !== 1` を fail-closed で不使用（検索を無効表示の
/// まま）とする契約（設計文書 §3-1）。
pub const SCHEMA_VERSION: u32 = 1;

/// 1 ページあたりの `text` フィールドの最大バイト数。超過分は UTF-8 文字境界で
/// 決定的に切り詰める（エラーにしない、設計文書 §3-4）。
pub const MAX_PAGE_TEXT_BYTES: usize = 4096;

/// インデックス JSON 全体の最大バイト数。超過時は fail-closed（設計文書 §3-4）。
pub const MAX_INDEX_BYTES: usize = 1_048_576;

/// ページ内目次の 1 見出しに対応するインデックスエントリ。
///
/// [`page_entry`] が [`layout::with_heading_anchors`] の戻り値
/// （[`layout::TocEntry`]）から 1:1 で写す。`id` は実 HTML の見出し `id`
/// 属性と一致することが `tests/search_index.rs` の見出し id パリティテストで
/// 機械固定されている（設計文書 §3-3 末尾）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionEntry {
    /// アンカー先 `id` 属性値。
    pub id: String,
    /// 見出しレベル（`h2` → 2 / `h3` → 3）。
    pub level: u8,
    /// 見出しの表示テキスト。
    pub title: String,
}

/// 1 ページ分のインデックスエントリ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageEntry {
    /// `base_path` 適用済みのサイト絶対パス（[`layout::asset_href`] と同一の
    /// 単一実装点で生成される）。
    pub href: String,
    /// ページタイトル。
    pub title: String,
    /// ページ内目次に対応する見出し列（`h2`/`h3` 全件。目次と異なり
    /// `TOC_MAX_LEVEL` による間引きは行わない）。
    pub sections: Vec<SectionEntry>,
    /// 正規化・[`MAX_PAGE_TEXT_BYTES`] 以下への切り詰めを終えた本文プレーン
    /// テキスト。
    pub text: String,
}

/// [`check_size`] が返す失敗理由。
#[derive(Debug)]
pub enum SearchIndexError {
    /// 生成した JSON が [`MAX_INDEX_BYTES`] を超過した。
    TooLarge {
        /// 実際のバイト数。
        bytes: usize,
        /// 上限バイト数（[`MAX_INDEX_BYTES`]）。
        limit: usize,
    },
}

impl fmt::Display for SearchIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchIndexError::TooLarge { bytes, limit } => {
                write!(
                    f,
                    "search index is {bytes} bytes, exceeding the {limit} byte limit"
                )
            }
        }
    }
}

impl std::error::Error for SearchIndexError {}

/// 1 ページの本文 `Node` からインデックスエントリを組み立てる。
///
/// `body` は [`crate::build::build_site`] のページループが `prev_next_nav` を
/// 追記する**前**の `[rewritten_body, generated_content]` を想定する
/// （設計文書 §3-2）。見出し id の取得は `body.clone()` に対して
/// [`layout::with_heading_anchors`] を再実行して行う（同関数は「既存 id を
/// 尊重し、衝突時のみ採番する」契約のため冪等であり、`docs_page_with_assets`
/// 内部の実行と同一の id 列を返す。`tests/search_index.rs` の冪等性テストが
/// この前提を機械固定する）。
pub fn page_entry(href: &str, title: &str, body: &Node) -> PageEntry {
    let (_annotated, toc_entries) = layout::with_heading_anchors(body.clone());
    let sections = toc_entries
        .into_iter()
        .map(|entry| SectionEntry {
            id: entry.id,
            level: entry.level,
            title: entry.title,
        })
        .collect();

    let raw_text = collect_text(body);
    let normalized = normalize_whitespace(&raw_text);
    let text = truncate_to_byte_limit(&normalized, MAX_PAGE_TEXT_BYTES);

    PageEntry {
        href: href.to_string(),
        title: title.to_string(),
        sections,
        text,
    }
}

/// [`Node`] 木から本文プレーンテキストを抽出する（[`page_entry`] の内部実装）。
///
/// `crate::layout::extract_text` を流用しない（同関数は TOC タイトル用の単純
/// 連結であり `data-scope` 部分木を除外しないため、部品ページの anatomy デモ
/// （「Tab 1」等のプレースホルダ語）由来のノイズを索引に含めてしまう）。
/// `data-scope` 部分木の除外は [`layout::with_heading_anchors`] の TOC 除外
/// ルール（`crate::layout::inject_heading_anchors` の同名分岐）と同一基準を
/// 独立実装で踏襲し、二重基準を作らない。
fn collect_text(node: &Node) -> String {
    let mut out = String::new();
    collect_text_into(node, &mut out);
    out
}

/// [`collect_text`] の内部再帰実装。要素の切れ目に単一の半角スペースを挿入する
/// （正規化前の粗いブロック境界。連続空白の畳み込みは [`normalize_whitespace`]
/// が担う）。
fn collect_text_into(node: &Node, out: &mut String) {
    match node {
        Node::Text(s) => out.push_str(s),
        Node::Element {
            attrs, children, ..
        } => {
            // headless-ui anatomy ルート（`data-scope` 属性）の部分木は
            // `layout::inject_heading_anchors` と同一基準で丸ごと除外する。
            if attrs.iter().any(|(name, _)| name == "data-scope") {
                return;
            }
            out.push(' ');
            for child in children {
                collect_text_into(child, out);
            }
            out.push(' ');
        }
        // 索引テキストへ生 HTML 断片を取り込まない（docs-site は raw_html() を
        // 使わない方針だが防御的に実装する。モジュール doc のセキュリティ
        // 不変条件参照）。
        Node::RawHtml(_) => {}
    }
}

/// 空白（`char::is_whitespace`、`U+00A0`/`U+3000` を含む）の連続を単一
/// `U+0020` へ畳み、前後を trim する。
///
/// 「空白」の定義を `char::is_whitespace` に固定することが決定性の根拠であり
/// （設計文書 §3-3）、実装を変更する場合はこの doc コメントを更新する。
fn normalize_whitespace(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_was_space = true; // 先頭の空白を出力しないための初期値
    for c in input.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
        } else {
            out.push(c);
            last_was_space = false;
        }
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

/// `s` を `max_bytes` バイト以下の最大の UTF-8 文字境界で切り詰める。
///
/// 切り詰め痕跡の付加文字（`…` 等）は付けない（決定性と単純さを優先する、
/// 設計文書 §3-4）。呼び出し元は（正規化を先に行った後で）本関数を呼ぶ契約
/// （順序を逆にするとバイト数がずれる）。
fn truncate_to_byte_limit(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut boundary = 0;
    for (idx, _) in s.char_indices() {
        if idx > max_bytes {
            break;
        }
        boundary = idx;
    }
    s[..boundary].to_string()
}

/// `s` 中の JSON 文字列リテラル向けエスケープ対象文字を `out` へ書き出す。
///
/// 必須（JSON 仕様）: `"` → `\"`、`\` → `\\`、制御文字（`U+0000`〜`U+001F`）
/// → `\u00XX`（`\n`/`\t` 等の短縮形は使わない。表記ゆれがバイト一致決定性を
/// 壊すため長形式で統一する）。
/// 追加（多層防御）: `<` → `<`、`>` → `>`、`&` → `&`、
/// `U+2028`/`U+2029` → ` `/` `。将来この JSON が `<script>` へ
/// インライン化される変更が入っても `</script>` 断片が生成されない構造的防御
/// （モジュール doc 参照）。それ以外の UTF-8 はそのまま出力する（日本語を
/// `\uXXXX` 化しない）。
fn escape_json_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '<' => out.push_str("\\u003C"),
            '>' => out.push_str("\\u003E"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) <= 0x1F => {
                out.push_str(&format!("\\u{:04X}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// [`PageEntry`] 列から決定的な JSON を組み立てる（外部クレート非依存の手書き
/// シリアライザ）。
///
/// キー順を固定する: `version` → `base_path` → `pages`、`pages` 内は
/// `href` → `title` → `sections` → `text`、`sections` 内は
/// `id` → `level` → `title`。`HashMap` を一切使わない（`Vec` のみ）ため
/// キー順は常に決定的である。`base_path` も [`escape_json_string`] を通す
/// （`nav.toml` 由来の著者入力であり、素の補間で埋め込まない）。
pub fn render_json(base_path: &str, entries: &[PageEntry]) -> String {
    let mut out = String::new();
    out.push('{');

    out.push_str("\"version\":");
    out.push_str(&SCHEMA_VERSION.to_string());
    out.push(',');

    out.push_str("\"base_path\":");
    escape_json_string(base_path, &mut out);
    out.push(',');

    out.push_str("\"pages\":[");
    for (i, entry) in entries.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('{');

        out.push_str("\"href\":");
        escape_json_string(&entry.href, &mut out);
        out.push(',');

        out.push_str("\"title\":");
        escape_json_string(&entry.title, &mut out);
        out.push(',');

        out.push_str("\"sections\":[");
        for (j, section) in entry.sections.iter().enumerate() {
            if j > 0 {
                out.push(',');
            }
            out.push('{');
            out.push_str("\"id\":");
            escape_json_string(&section.id, &mut out);
            out.push(',');
            out.push_str("\"level\":");
            out.push_str(&section.level.to_string());
            out.push(',');
            out.push_str("\"title\":");
            escape_json_string(&section.title, &mut out);
            out.push('}');
        }
        out.push_str("],");

        out.push_str("\"text\":");
        escape_json_string(&entry.text, &mut out);

        out.push('}');
    }
    out.push_str("]}");

    out
}

/// `json` のバイト数が [`MAX_INDEX_BYTES`] 以下であることを検証する。
///
/// # Errors
///
/// 超過時は [`SearchIndexError::TooLarge`] を返す。呼び出し元
/// （[`crate::build::build_site`]）はこれを `ssg::generate_pages` より前に
/// 呼び、失敗時は `out_dir` に一切書き出さない（fail-closed、設計文書 §3-4）。
pub fn check_size(json: &str) -> Result<(), SearchIndexError> {
    let bytes = json.len();
    if bytes > MAX_INDEX_BYTES {
        return Err(SearchIndexError::TooLarge {
            bytes,
            limit: MAX_INDEX_BYTES,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{div, el, text};

    #[test]
    fn escape_json_string_escapes_required_and_defense_in_depth_chars() {
        let mut out = String::new();
        escape_json_string("a\"b\\c<d>e&f\u{0007}g", &mut out);
        assert_eq!(out, "\"a\\\"b\\\\c\\u003Cd\\u003Ee\\u0026f\\u0007g\"");
    }

    #[test]
    fn escape_json_string_uses_long_form_for_control_chars_not_short_escapes() {
        let mut out = String::new();
        escape_json_string("a\nb\tc", &mut out);
        // 短縮形ではなく \u00XX の長形式で統一する
        // （表記ゆれがバイト一致決定性を壊すため）。
        assert_eq!(out, "\"a\\u000Ab\\u0009c\"");
    }

    #[test]
    fn escape_json_string_preserves_non_ascii_utf8_as_is() {
        let mut out = String::new();
        escape_json_string("日本語", &mut out);
        assert_eq!(out, "\"日本語\"");
    }

    #[test]
    fn normalize_whitespace_collapses_runs_and_trims() {
        assert_eq!(normalize_whitespace("  a   b\n\tc  "), "a b c");
        assert_eq!(normalize_whitespace("a\u{00A0}\u{3000}b"), "a b");
        assert_eq!(normalize_whitespace(""), "");
    }

    #[test]
    fn truncate_to_byte_limit_cuts_at_utf8_char_boundary() {
        // 「あ」は UTF-8 で 3 バイト。上限 4 バイトなら 1 文字（3 バイト）まで。
        let s = "ああ";
        let truncated = truncate_to_byte_limit(s, 4);
        assert_eq!(truncated, "あ");
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
    }

    /// 「上限直下の最大の文字境界で切る」ことを、markdown パイプライン越しの
    /// 統合テスト（`tests/search_index.rs`）に頼らず本関数だけで固定する。
    /// 切り詰め後の残りバイト数に、切り捨てた側の次の 1 文字を足すと必ず
    /// `max_bytes` を超えることを確認する（「もう 1 文字足せば超過する」＝
    /// 最大境界であることの直接証明）。
    #[test]
    fn truncate_to_byte_limit_cuts_at_the_maximal_boundary_not_earlier() {
        let cases: &[(&str, usize)] = &[
            ("hello world", 5), // ASCII、境界ちょうど
            ("ああ", 4),        // 先頭 1 文字だけ収まる
            ("aああ", 4),       // ASCII 1 文字 + マルチバイトの混在
            ("あa", 3),         // マルチバイト文字の直後で切れる境界
            ("ab😀cd", 6),      // 絵文字（4 バイト）を跨ぐ境界
            ("hello", 4096),    // 上限内（no-op）
        ];
        for &(s, limit) in cases {
            let result = truncate_to_byte_limit(s, limit);
            assert!(
                result.len() <= limit,
                "result must not exceed the byte limit: {result:?} ({} bytes) > {limit}",
                result.len()
            );
            assert!(std::str::from_utf8(result.as_bytes()).is_ok());
            if let Some(next_char) = s[result.len()..].chars().next() {
                assert!(
                    result.len() + next_char.len_utf8() > limit,
                    "boundary is not maximal: {result:?} ({} bytes) + {next_char:?} \
                     would still fit within {limit}",
                    result.len()
                );
            }
        }
    }

    #[test]
    fn truncate_to_byte_limit_is_noop_when_within_limit() {
        assert_eq!(truncate_to_byte_limit("hello", 4096), "hello");
    }

    #[test]
    fn check_size_passes_at_exact_limit_and_fails_one_byte_over() {
        let ok = "a".repeat(MAX_INDEX_BYTES);
        assert!(check_size(&ok).is_ok());
        let too_big = "a".repeat(MAX_INDEX_BYTES + 1);
        match check_size(&too_big) {
            Err(SearchIndexError::TooLarge { bytes, limit }) => {
                assert_eq!(bytes, MAX_INDEX_BYTES + 1);
                assert_eq!(limit, MAX_INDEX_BYTES);
            }
            Ok(()) => panic!("expected TooLarge error"),
        }
    }

    #[test]
    fn render_json_key_order_is_fixed() {
        let entries = vec![PageEntry {
            href: "/a/".to_string(),
            title: "A".to_string(),
            sections: vec![SectionEntry {
                id: "s1".to_string(),
                level: 2,
                title: "S1".to_string(),
            }],
            text: "body text".to_string(),
        }];
        let json = render_json("/base", &entries);
        assert_eq!(
            json,
            "{\"version\":1,\"base_path\":\"/base\",\"pages\":[{\"href\":\"/a/\",\"title\":\"A\",\"sections\":[{\"id\":\"s1\",\"level\":2,\"title\":\"S1\"}],\"text\":\"body text\"}]}"
        );
    }

    #[test]
    fn render_json_is_deterministic() {
        let entries = vec![PageEntry {
            href: "/a/".to_string(),
            title: "A".to_string(),
            sections: vec![],
            text: "t".to_string(),
        }];
        let first = render_json("/base", &entries);
        let second = render_json("/base", &entries);
        assert_eq!(first, second);
    }

    #[test]
    fn collect_text_excludes_data_scope_subtree() {
        let body = div(
            vec![],
            vec![
                text("visible"),
                el(
                    "div",
                    vec![("data-scope", "tabs")],
                    vec![text("hidden anatomy demo text")],
                ),
                text("also visible"),
            ],
        );
        let extracted = collect_text(&body);
        assert!(extracted.contains("visible"));
        assert!(extracted.contains("also visible"));
        assert!(!extracted.contains("hidden anatomy demo text"));
    }

    #[test]
    fn collect_text_does_not_concatenate_raw_html() {
        let body = div(
            vec![],
            vec![text("before"), Node::RawHtml("<b>raw</b>".to_string())],
        );
        let extracted = collect_text(&body);
        assert!(extracted.contains("before"));
        assert!(!extracted.contains("raw"));
        assert!(!extracted.contains('<'));
    }

    #[test]
    fn page_entry_produces_normalized_and_truncated_text_with_sections() {
        let body = div(
            vec![],
            vec![
                el("h2", vec![], vec![text("Heading One".to_string())]),
                text("  some   body   text  ".to_string()),
            ],
        );
        let entry = page_entry("/page/", "Page", &body);
        assert_eq!(entry.href, "/page/");
        assert_eq!(entry.title, "Page");
        assert_eq!(entry.sections.len(), 1);
        assert_eq!(entry.sections[0].level, 2);
        assert_eq!(entry.sections[0].title, "Heading One");
        assert!(entry.text.contains("some body text"));
    }
}

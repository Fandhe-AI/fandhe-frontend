//! ビルド時検索インデックスの生成（イシュー #957、セクション粒度分割は
//! イシュー #3173）。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::build::build_site`] がページループ内で [`page_entry`] を各ページから
//! 収集し、`site/nav.toml` の `[[section]]`（[`crate::nav::Section`]）ごとに
//! まとめて [`build_files`] へ渡す。[`build_files`] は `ssg::generate_pages`
//! による書き出しより前に完了し、次の 2 種類のファイルを返す:
//!
//! - マニフェスト [`REL_PATH`]（`assets/search-index.json`）: セクション一覧
//!   （`title` と各セクションファイルの `href`）のみを持つ小さな JSON。
//!   `layout` の `data-search-index` 属性が指す唯一の入口。
//! - セクションファイル `assets/search-index/<slug>.json`（[`section_rel_path`]）:
//!   当該セクション配下のページエントリ（`pages`）。ファイル数は nav.toml の
//!   `[[section]]` 数と常に一致し、セクション追加時に本モジュール・
//!   `crate::build`・`crate::script` のいずれにも分岐を足す必要がない
//!   （レジストリ駆動、設計文書 §10-15）。
//!
//! 生成した JSON は検索 UI（`crate::script` の第 3 IIFE）が初回 focus 時に
//! マニフェスト → 全セクションファイルの順で `fetch()` する契約であり、本
//! モジュールは HTML へのインライン化を一切行わない（不変条件は下記参照）。
//!
//! 設計の正は `docs/design/docs-site-search-design.md` §3・§10-15 であり、本
//! モジュールは同文書から逸脱しない（`MAX_PAGE_TEXT_BYTES` 等の定数値を含む）。
//!
//! # セキュリティ不変条件（REQ-1、`.claude/rules/coding-rust.md`）
//!
//! - インデックスは常に**独立ファイルとして fetch される**。HTML への埋め込み
//!   （インライン `<script>`・`data-*` 属性への本文格納）は禁止する。
//! - JSON シリアライズは手書き（`escape_json_string`）で行い、外部クレートを
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

/// マニフェスト JSON（`assets/search-index.json`）の `out_dir` 起点相対パス。
/// `crate::layout` の `data-search-index` 属性値（[`crate::layout::asset_href`]
/// 経由）と `crate::build` の書き出し先の単一実装点。イシュー #3173 以前は
/// 全ページを 1 ファイルに集約した索引本体だったが、現在はセクション一覧のみ
/// を持つ（ページエントリは [`section_rel_path`] のファイルへ分割される）。
pub const REL_PATH: &str = "assets/search-index.json";

/// セクションファイルを置くディレクトリの `out_dir` 起点相対パス
/// （[`section_rel_path`] が `"<dir>/<slug>.json"` を組み立てる）。
/// `site/assets/` はディレクトリを許容しない（`crate::build` の
/// `list_regular_files` が `UnsupportedAssetEntry` で拒否する）ため、
/// `RESERVED_ASSET_NAMES` による basename 衝突判定の対象外である。
pub const SECTION_DIR_REL_PATH: &str = "assets/search-index";

/// インデックス JSON のスキーマバージョン。破壊的変更時にインクリメントする。
/// マニフェスト・セクションファイルの双方が同じ値を持ち、JS 側は
/// `version !== 2` を fail-closed で不使用（検索を無効表示のまま）とする契約
/// （設計文書 §3-1）。イシュー #3173 で 1 → 2（単一ファイル → マニフェスト
/// + セクションファイル）。
pub const SCHEMA_VERSION: u32 = 2;

/// 1 ページあたりの `text` フィールドの最大バイト数。超過分は UTF-8 文字境界で
/// 決定的に切り詰める（エラーにしない、設計文書 §3-4）。
///
/// #957 設計時点の 4096 から、旧 `MAX_INDEX_BYTES`（1 ファイル全体上限）超過
/// への緊急避難として 4032（§10-10）→ 4000（§10-12）へ引き下げられた経緯を
/// 持つ。イシュー #3173 のセクション分割で全体上限の概念自体が消えたため、
/// 本定数を索引総量の調整弁として再び上下させることはしない（索引精度を
/// 一律に落とす対症療法であり、§10-14 で「機械的に繰り返さない」と決定
/// 済み）。値は 4000 のまま据え置く。
pub const MAX_PAGE_TEXT_BYTES: usize = 4000;

/// セクションファイル 1 件あたりの最大バイト数（2.5 MiB）。超過時は
/// fail-closed（[`SearchIndexError::TooLarge`]、設計文書 §10-15）。
///
/// イシュー #3173 以前の `MAX_INDEX_BYTES`（全ページを集約した 1 ファイルの
/// 上限、最終値 1,703,936 バイト）は「これ以上引き上げない」ハードルール
/// （設計文書 §10-6）のもとで Blocks ページ 1 件の追加ごとに超過していた。
/// セクション分割後は「1 ファイル全体」という概念自体が存在せず、本定数が
/// 唯一のサイズ防波堤である。
///
/// 値の根拠（イシュー #3173 時点の実測、設計文書 §10-15）: 最大セクション
/// Blocks は 174 ページ・527,062 バイト（1 ページ平均 3,029 バイト、`text`
/// 以外のオーバーヘッド平均 337 バイト・最大 1,826 バイト）。拡充ツリー
/// #2730 完了後の約 400 ページへ線形外挿すると、平均で約 1.19 MB（本上限の
/// 約 46%）、全ページが [`MAX_PAGE_TEXT_BYTES`] に張り付いた現実的な最悪値
/// （4,000 + 337）× 400 = 1.73 MB（約 66%）、理論上の最悪値（4,000 + 1,826）
/// × 400 = 2.33 MB（約 89%）のいずれも本上限に収まる。
///
/// 再評価トリガー: いずれかのセクションファイルが本上限の 80% を超えた場合、
/// 本定数を引き上げるのではなく、当該セクションをさらに分割する
/// （Blocks なら `crate::blocks::BlockSection` 単位）ことを先に検討する。
pub const MAX_SECTION_INDEX_BYTES: usize = 2_621_440;

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

/// [`build_files`] / [`check_size`] が返す失敗理由。
#[derive(Debug)]
pub enum SearchIndexError {
    /// 生成したセクションファイル JSON が [`MAX_SECTION_INDEX_BYTES`] を
    /// 超過した。
    TooLarge {
        /// 超過したセクションのスラッグ（[`section_slug`]）。
        section: String,
        /// 実際のバイト数。
        bytes: usize,
        /// 上限バイト数（[`MAX_SECTION_INDEX_BYTES`]）。
        limit: usize,
    },
    /// セクションタイトルから ASCII 英数字を 1 文字も取り出せず、ファイル名
    /// を決定できない（[`section_slug`] が空文字を返した）。
    EmptySectionSlug {
        /// 元のセクションタイトル。
        title: String,
    },
    /// 2 つのセクションタイトルが同じスラッグに写り、ファイルが上書きされる
    /// （`"API Reference"` と `"api-reference"` 等）。
    DuplicateSectionSlug {
        /// 衝突したスラッグ。
        slug: String,
        /// 先に登録されたセクションタイトル。
        first: String,
        /// 後から衝突したセクションタイトル。
        second: String,
    },
}

impl fmt::Display for SearchIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchIndexError::TooLarge {
                section,
                bytes,
                limit,
            } => {
                write!(
                    f,
                    "search index section `{section}` is {bytes} bytes, exceeding the {limit} byte per-file limit"
                )
            }
            SearchIndexError::EmptySectionSlug { title } => {
                write!(
                    f,
                    "search index section title {title:?} yields an empty file name slug"
                )
            }
            SearchIndexError::DuplicateSectionSlug {
                slug,
                first,
                second,
            } => {
                write!(
                    f,
                    "search index sections {first:?} and {second:?} both map to file name slug `{slug}`"
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
///
/// `exclude_code_blocks` が `true` のページ（Blocks ページ、呼び出し元の
/// `crate::build::build_site` が `blocks::block_for_path` で判定する）は
/// フェンスコードブロック（`pre`/`code`）本文を索引テキストから除外する
/// （設計文書 §3-3・§10-13。Blocks ページの `## Rust コード` 節はページ本文の
/// 大半〔実測で 9 割超〕を占める部品実装コードの全文複製であり、`fn`/`use`
/// 等の API 名検索を想定した §3-3 の「コードブロックは含める」方針が前提と
/// する「総量が支配的でない」という条件がもはや成立しないため。API 参照
/// ページ（`docs/guides/component-authoring.md` 等）はこのフラグを立てず
/// 従来どおりコードブロックを索引に含める、`tests/search_index.rs` の
/// `real_site_search_index_still_contains_code_block_keywords_after_highlighting`
/// が回帰を固定する）。
pub fn page_entry(href: &str, title: &str, body: &Node, exclude_code_blocks: bool) -> PageEntry {
    let (_annotated, toc_entries) = layout::with_heading_anchors(body.clone());
    let sections = toc_entries
        .into_iter()
        .map(|entry| SectionEntry {
            id: entry.id,
            level: entry.level,
            title: entry.title,
        })
        .collect();

    let raw_text = collect_text(body, exclude_code_blocks);
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
fn collect_text(node: &Node, exclude_code_blocks: bool) -> String {
    let mut out = String::new();
    collect_text_into(node, &mut out, exclude_code_blocks);
    out
}

/// [`collect_text`] の内部再帰実装。要素の切れ目に単一の半角スペースを挿入する
/// （正規化前の粗いブロック境界。連続空白の畳み込みは [`normalize_whitespace`]
/// が担う）。
///
/// # ハイライトトークン `span` の透過扱い（イシュー #1078 レビュー指摘）
///
/// `crate::highlight::highlight_children` はフェンスコードブロック内の
/// トークンを `span(vec![("class", "token-*")], vec![text(t.text)])` で包む。
/// 本関数が要素境界に一律スペースを挿入すると、たとえば Rust フェンス中の
/// `crate::highlight`（`crate` はキーワードで span に包まれる）が索引上で
/// `"crate ::highlight"` のように分断され、利用者が `crate::highlight` で
/// 全文検索してもヒットしなくなる（span 化前は単一の `Text` ノードだった
/// ため発生しなかった退行）。`class` 属性値が `token-` 接頭辞を持つ `span`
/// は「表示上の色分けのみを目的とした透過的な装飾」であり本文の語境界では
/// ないため、スペースを挿入せず子ノードへそのまま連結する
/// （[`is_token_span`] 参照）。
fn collect_text_into(node: &Node, out: &mut String, exclude_code_blocks: bool) {
    match node {
        Node::Text(s) => out.push_str(s),
        Node::Element {
            tag,
            attrs,
            children,
        } => {
            // セクション索引カード（`section_index` の `li.docs-index-card`）は
            // 部品 root（`data-scope="card"`）の内側にタイトル・説明文を持つが、
            // 手書き索引の置換先として検索対象に含める契約（#3616）のため、
            // この部分木に限り `data-scope` 除外を適用せず全文を連結する。
            if is_index_card(tag, attrs) {
                out.push(' ');
                collect_all_text(node, out);
                out.push(' ');
                return;
            }
            // headless-ui anatomy ルート（`data-scope` 属性）の部分木は
            // `layout::inject_heading_anchors` と同一基準で丸ごと除外する。
            if attrs.iter().any(|(name, _)| name == "data-scope") {
                return;
            }
            // Blocks ページ限定でフェンスコードブロック（`crate::markdown::
            // parse_fence` が生成する `pre` 要素、doc コメント参照）本文を
            // 除外する。`pre` の部分木を丸ごと落とすため `code` 単体の判定は
            // 不要（`pre` は常に `code` 子 1 個のみを持つ、`markdown::
            // parse_fence` 参照）。
            if exclude_code_blocks && *tag == "pre" {
                return;
            }
            if is_token_span(tag, attrs) {
                for child in children {
                    collect_text_into(child, out, exclude_code_blocks);
                }
                return;
            }
            out.push(' ');
            for child in children {
                collect_text_into(child, out, exclude_code_blocks);
            }
            out.push(' ');
        }
        // 索引テキストへ生 HTML 断片を取り込まない（docs-site は raw_html() を
        // 使わない方針だが防御的に実装する。モジュール doc のセキュリティ
        // 不変条件参照）。
        Node::RawHtml(_) => {}
    }
}

/// 索引カードのラッパー（`li.docs-index-card`）またはトップページの特徴カード
/// （`li.docs-feature`、#3613。旧 `site/index.md` の「特徴」節の置換先）かどうか。
fn is_index_card(tag: &str, attrs: &[(String, String)]) -> bool {
    tag == "li"
        && attrs.iter().any(|(name, value)| {
            name == "class"
                && value
                    .split_whitespace()
                    .any(|c| c == "docs-index-card" || c == "docs-feature")
        })
}

/// `data-scope` 除外を行わず部分木のテキストを連結する（索引カード専用）。
/// `RawHtml` は [`collect_text_into`] と同じく取り込まない。
fn collect_all_text(node: &Node, out: &mut String) {
    match node {
        Node::Text(s) => out.push_str(s),
        Node::Element { children, .. } => {
            out.push(' ');
            for child in children {
                collect_all_text(child, out);
            }
            out.push(' ');
        }
        Node::RawHtml(_) => {}
    }
}

/// `crate::highlight::highlight_children` が生成する色分け `span` かどうかを
/// 判定する（[`collect_text_into`] が語結合破壊を避けるために参照する）。
///
/// `class` 属性値が `token-` で始まる `span` 要素のみを対象とする。
/// `crate::highlight::TokenKind::class` が返すクラス名は常に `token-` 接頭辞
/// （`token-keyword` 等）であり、他の意図（`pre-styled-ui` のバッジ等）で
/// `span` に `token-` 接頭辞のクラスが使われることは想定していない
/// （万一の衝突があっても「語を分断しない」方向への誤判定でしかなく、索引の
/// 検索可能性を損なう副作用は生じない）。
fn is_token_span(tag: &str, attrs: &[(String, String)]) -> bool {
    tag == "span"
        && attrs
            .iter()
            .any(|(name, value)| name == "class" && value.starts_with("token-"))
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

/// セクションタイトルをファイル名スラッグへ写す（`"API Reference"` →
/// `"api-reference"`）。
///
/// ASCII 英数字は小文字化してそのまま、それ以外の文字の連続は 1 個の `-`
/// に畳み、先頭・末尾の `-` は落とす。出力は `fandhe_frontend_server::ssg`
/// のアセットファイル名検証（`is_safe_asset_file_name`）を常に通る文字集合
/// （`[a-z0-9-]`）に閉じる。非 ASCII のみのタイトルは空文字になり、
/// [`build_files`] が [`SearchIndexError::EmptySectionSlug`] で拒否する。
pub fn section_slug(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

/// セクションファイルの `out_dir` 起点相対パス（`assets/search-index/<slug>.json`）。
pub fn section_rel_path(slug: &str) -> String {
    format!("{SECTION_DIR_REL_PATH}/{slug}.json")
}

/// 1 セクション分の入力（[`build_files`] の引数）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionInput {
    /// `[[section]]` の `title`（マニフェストへそのまま写す。ファイル名は
    /// [`section_slug`] で導出する）。
    pub title: String,
    /// このセクション配下のページエントリ（[`crate::nav::Section::all_pages`]
    /// の順）。
    pub entries: Vec<PageEntry>,
}

/// セクション別インデックスファイル群とマニフェストを組み立てる
/// （イシュー #3173）。
///
/// 戻り値は `("/<out_dir 起点相対パス>", JSON)` の列で、先頭がマニフェスト
/// （[`REL_PATH`]）、以降が `sections` の順のセクションファイル
/// （[`section_rel_path`]）。呼び出し元（[`crate::build::build_site`]）は
/// そのまま `ssg::generate_assets` へ渡す。
///
/// # Errors
///
/// セクションファイルが 1 件でも [`MAX_SECTION_INDEX_BYTES`] を超えた場合
/// （[`SearchIndexError::TooLarge`]）、スラッグが空・重複した場合
/// （[`SearchIndexError::EmptySectionSlug`] /
/// [`SearchIndexError::DuplicateSectionSlug`]）。いずれも呼び出し元が
/// `ssg::generate_pages` より前に検知し `out_dir` を汚さない fail-closed。
pub fn build_files(
    base_path: &str,
    sections: &[SectionInput],
) -> Result<Vec<(String, String)>, SearchIndexError> {
    let mut files = Vec::with_capacity(sections.len() + 1);
    let mut manifest_entries: Vec<(String, String)> = Vec::with_capacity(sections.len());
    let mut seen: Vec<(String, &str)> = Vec::with_capacity(sections.len());
    for section in sections {
        let slug = section_slug(&section.title);
        if slug.is_empty() {
            return Err(SearchIndexError::EmptySectionSlug {
                title: section.title.clone(),
            });
        }
        if let Some((_, first)) = seen.iter().find(|(s, _)| *s == slug) {
            return Err(SearchIndexError::DuplicateSectionSlug {
                slug,
                first: (*first).to_string(),
                second: section.title.clone(),
            });
        }
        let json = render_section_json(base_path, &section.entries);
        check_size(&slug, &json)?;
        let rel_path = section_rel_path(&slug);
        manifest_entries.push((
            section.title.clone(),
            layout::asset_href(base_path, &rel_path),
        ));
        files.push((format!("/{rel_path}"), json));
        seen.push((slug, &section.title));
    }
    files.insert(
        0,
        (
            format!("/{REL_PATH}"),
            render_manifest_json(base_path, &manifest_entries),
        ),
    );
    Ok(files)
}

/// マニフェスト JSON（`{"version":2,"base_path":"…","sections":[{"title":"…","href":"…"}]}`）
/// を決定的に組み立てる。`sections` は `(title, href)` の列で、`href` は
/// `base_path` 適用済みのサイト絶対パス（ページエントリの `href` と同じ
/// [`layout::asset_href`] 単一実装点）。
pub fn render_manifest_json(base_path: &str, sections: &[(String, String)]) -> String {
    let mut out = String::new();
    out.push('{');
    out.push_str("\"version\":");
    out.push_str(&SCHEMA_VERSION.to_string());
    out.push(',');
    out.push_str("\"base_path\":");
    escape_json_string(base_path, &mut out);
    out.push(',');
    out.push_str("\"sections\":[");
    for (i, (title, href)) in sections.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('{');
        out.push_str("\"title\":");
        escape_json_string(title, &mut out);
        out.push(',');
        out.push_str("\"href\":");
        escape_json_string(href, &mut out);
        out.push('}');
    }
    out.push_str("]}");
    out
}

/// [`PageEntry`] 列から 1 セクションファイル分の決定的な JSON を組み立てる
/// （外部クレート非依存の手書きシリアライザ）。
///
/// キー順を固定する: `version` → `base_path` → `pages`、`pages` 内は
/// `href` → `title` → `sections` → `text`、`sections` 内は
/// `id` → `level` → `title`。`HashMap` を一切使わない（`Vec` のみ）ため
/// キー順は常に決定的である。`base_path` も `escape_json_string` を通す
/// （`nav.toml` 由来の著者入力であり、素の補間で埋め込まない）。
pub fn render_section_json(base_path: &str, entries: &[PageEntry]) -> String {
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

/// セクションファイル `json` のバイト数が [`MAX_SECTION_INDEX_BYTES`] 以下で
/// あることを検証する（`section` は診断メッセージ用のスラッグ）。
///
/// # Errors
///
/// 超過時は [`SearchIndexError::TooLarge`] を返す。[`build_files`] 経由で
/// `ssg::generate_pages` より前に呼ばれ、失敗時は `out_dir` に一切書き出さ
/// ない（fail-closed、設計文書 §3-4）。
pub fn check_size(section: &str, json: &str) -> Result<(), SearchIndexError> {
    let bytes = json.len();
    if bytes > MAX_SECTION_INDEX_BYTES {
        return Err(SearchIndexError::TooLarge {
            section: section.to_string(),
            bytes,
            limit: MAX_SECTION_INDEX_BYTES,
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
        let ok = "a".repeat(MAX_SECTION_INDEX_BYTES);
        assert!(check_size("blocks", &ok).is_ok());
        let too_big = "a".repeat(MAX_SECTION_INDEX_BYTES + 1);
        match check_size("blocks", &too_big) {
            Err(SearchIndexError::TooLarge {
                section,
                bytes,
                limit,
            }) => {
                assert_eq!(section, "blocks");
                assert_eq!(bytes, MAX_SECTION_INDEX_BYTES + 1);
                assert_eq!(limit, MAX_SECTION_INDEX_BYTES);
            }
            Err(other) => panic!("expected TooLarge error, got {other}"),
            Ok(()) => panic!("expected TooLarge error"),
        }
    }

    #[test]
    fn section_slug_lowercases_and_collapses_non_alphanumerics() {
        assert_eq!(section_slug("API Reference"), "api-reference");
        assert_eq!(section_slug("Getting Started"), "getting-started");
        assert_eq!(section_slug("  Blocks / Extra!! "), "blocks-extra");
        assert_eq!(section_slug("日本語"), "");
    }

    fn section(title: &str) -> SectionInput {
        SectionInput {
            title: title.to_string(),
            entries: vec![PageEntry {
                href: format!("/{}/", section_slug(title)),
                title: title.to_string(),
                sections: vec![],
                text: "t".to_string(),
            }],
        }
    }

    #[test]
    fn build_files_emits_manifest_first_then_one_file_per_section_in_order() {
        let files = build_files("/base", &[section("Guides"), section("API Reference")])
            .expect("build_files should succeed");
        assert_eq!(files.len(), 3);
        assert_eq!(files[0].0, "/assets/search-index.json");
        assert_eq!(
            files[0].1,
            "{\"version\":2,\"base_path\":\"/base\",\"sections\":[{\"title\":\"Guides\",\"href\":\"/base/assets/search-index/guides.json\"},{\"title\":\"API Reference\",\"href\":\"/base/assets/search-index/api-reference.json\"}]}"
        );
        assert_eq!(files[1].0, "/assets/search-index/guides.json");
        assert_eq!(files[2].0, "/assets/search-index/api-reference.json");
        assert!(files[1].1.contains("\"href\":\"/guides/\""));
    }

    #[test]
    fn build_files_rejects_empty_and_duplicate_slugs() {
        match build_files("", &[section("日本語")]) {
            Err(SearchIndexError::EmptySectionSlug { title }) => assert_eq!(title, "日本語"),
            other => panic!("expected EmptySectionSlug, got {other:?}"),
        }
        match build_files("", &[section("API Reference"), section("api-reference")]) {
            Err(SearchIndexError::DuplicateSectionSlug {
                slug,
                first,
                second,
            }) => {
                assert_eq!(slug, "api-reference");
                assert_eq!(first, "API Reference");
                assert_eq!(second, "api-reference");
            }
            other => panic!("expected DuplicateSectionSlug, got {other:?}"),
        }
    }

    #[test]
    fn render_section_json_key_order_is_fixed() {
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
        let json = render_section_json("/base", &entries);
        assert_eq!(
            json,
            "{\"version\":2,\"base_path\":\"/base\",\"pages\":[{\"href\":\"/a/\",\"title\":\"A\",\"sections\":[{\"id\":\"s1\",\"level\":2,\"title\":\"S1\"}],\"text\":\"body text\"}]}"
        );
    }

    #[test]
    fn render_section_json_is_deterministic() {
        let entries = vec![PageEntry {
            href: "/a/".to_string(),
            title: "A".to_string(),
            sections: vec![],
            text: "t".to_string(),
        }];
        let first = render_section_json("/base", &entries);
        let second = render_section_json("/base", &entries);
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
        let extracted = collect_text(&body, false);
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
        let extracted = collect_text(&body, false);
        assert!(extracted.contains("before"));
        assert!(!extracted.contains("raw"));
        assert!(!extracted.contains('<'));
    }

    /// キーワード/リテラルに隣接する非空白文字を含む語句
    /// （`crate::highlight` の `crate`、`foo(1)` の `1` 等）が、ハイライト
    /// トークン `span` を経由しても索引テキスト上で分断されないことを検証
    /// する（レビュー指摘の回帰テスト、イシュー #1078）。`fn`/`use`/
    /// `user_badge` のような「単独で分断されない単語」だけを見る従来の
    /// `tests/search_index.rs` の統合テストでは検出できない退行クラス
    /// （語の前後に非空白の記号・識別子が直接続くケース）を、本モジュール
    /// 内で `crate::highlight::highlight_children` の実出力に対して直接
    /// 検証する。
    #[test]
    fn collect_text_does_not_split_words_adjacent_to_highlight_token_spans() {
        // `crate::highlight` 相当: キーワード `crate` の直後に `::highlight`
        // が続く（`crate` だけが token-keyword span で包まれ、`::highlight`
        // は同じ children 列内の別トークンとして並ぶ）。
        let rust_src = "crate::highlight";
        let children = crate::highlight::highlight_children(rust_src, "rust")
            .expect("rust highlighting should succeed for this fixture");
        let body = el("pre", vec![], vec![el("code", vec![], children)]);
        let extracted = collect_text(&body, false);
        assert!(
            extracted.contains("crate::highlight"),
            "expected \"crate::highlight\" to remain contiguous in the index text, got: {extracted:?}"
        );

        // `foo(1)` 相当: 数値リテラル `1` の前後に `foo(` `)` が隣接する。
        let call_src = "foo(1)";
        let children = crate::highlight::highlight_children(call_src, "rust")
            .expect("rust highlighting should succeed for this fixture");
        let body = el("pre", vec![], vec![el("code", vec![], children)]);
        let extracted = collect_text(&body, false);
        assert!(
            extracted.contains("foo(1)"),
            "expected \"foo(1)\" to remain contiguous in the index text, got: {extracted:?}"
        );
    }

    /// イシュー #2862（§10-13）の恒久対処回帰: `exclude_code_blocks: true` の
    /// ページ（Blocks ページ相当）はフェンスコードブロック本文
    /// （`pre` の部分木）を索引テキストから除外し、それ以外の本文プレーン
    /// テキストは変わらず含む。
    #[test]
    fn collect_text_excludes_code_blocks_when_requested() {
        let body = div(
            vec![],
            vec![
                text("prose before"),
                el(
                    "pre",
                    vec![],
                    vec![el(
                        "code",
                        vec![],
                        vec![text("fn user_badge() {}".to_string())],
                    )],
                ),
                text("prose after"),
            ],
        );
        let excluded = collect_text(&body, true);
        assert!(excluded.contains("prose before"));
        assert!(excluded.contains("prose after"));
        assert!(!excluded.contains("user_badge"));

        // `exclude_code_blocks: false`（既定の非 Blocks ページ経路）は従来
        // どおりコードブロック本文も索引に含む。
        let included = collect_text(&body, false);
        assert!(included.contains("user_badge"));
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
        let entry = page_entry("/page/", "Page", &body, false);
        assert_eq!(entry.href, "/page/");
        assert_eq!(entry.title, "Page");
        assert_eq!(entry.sections.len(), 1);
        assert_eq!(entry.sections[0].level, 2);
        assert_eq!(entry.sections[0].title, "Heading One");
        assert!(entry.text.contains("some body text"));
    }

    /// [`page_entry`] の `exclude_code_blocks` 引数が実際に `collect_text`
    /// へ配線されていることの結合確認（配線漏れの回帰、イシュー #2862）。
    #[test]
    fn page_entry_excludes_code_blocks_when_requested() {
        let body = div(
            vec![],
            vec![el(
                "pre",
                vec![],
                vec![el(
                    "code",
                    vec![],
                    vec![text("fn pricing_comparison_table() {}".to_string())],
                )],
            )],
        );
        let entry = page_entry("/blocks/pricing-comparison-table/", "Page", &body, true);
        assert!(!entry.text.contains("pricing_comparison_table"));
    }

    /// トップページの特徴カード（`li.docs-feature`、#3613）は `data-scope` を
    /// 持つ部品 root を内包しても検索対象に残る。
    #[test]
    fn page_entry_keeps_landing_feature_card_text() {
        let body = el(
            "li",
            vec![("class", "docs-feature")],
            vec![el(
                "div",
                vec![("data-scope", "card")],
                vec![text("raw_html() 明示オプトイン".to_string())],
            )],
        );
        let entry = page_entry("/", "Home", &body, false);
        assert!(entry.text.contains("raw_html()"));
    }
}

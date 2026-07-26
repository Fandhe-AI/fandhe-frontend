//! 旧 URL 互換のリダイレクトページ生成機構（イシュー #1016）。
//!
//! # 背景・呼び出し文脈
//!
//! GitHub Pages は静的配信のみでサーバーサイドリダイレクトを持たないため、
//! `/components/<kebab>/` → `/themes/<kebab>/` のような URL 移行（#1017 で
//! 実施予定）を行うと旧 URL は 404 になる。本モジュールは `site/redirects.toml`
//! （[`MANIFEST_REL_PATH`]）の宣言を読み、`meta refresh` + `rel=canonical` の
//! 案内ページを生成することで旧 URL を 404 にしない。
//!
//! [`crate::build::build_site`] が [`parse_redirects`] → [`validate_against_nav`]
//! → [`redirect_page`] の順で呼び、生成した `(from, Node)` を
//! `fandhe_frontend_server::ssg::generate_pages` へ本体ページより先に渡す
//! （`build.rs` モジュール doc §5 参照）。
//!
//! # `nav.toml` 内 `[[redirect]]` ではなく別ファイルにした理由
//!
//! `docs/design/docs-site-primitives-themes-split.md` §4 は「`site/redirects.toml`
//! の新設、または `nav.toml` 内の専用トップレベルテーブル」のどちらでもよいと
//! している。本実装は別ファイル方式を採る。理由は以下の 3 点:
//!
//! 1. リダイレクトを [`crate::nav::Nav`] の構造から完全に分離できる。
//!    `nav.toml` に足すと [`crate::nav::parse_nav`] が redirect を知ることに
//!    なり、`search_index` の収集経路・`linkcheck` の解決先集合・
//!    `tests/site_nav.rs` のページ数期待値のいずれにも「redirect を除く」
//!    という除外述語を新たに持ち込む必要が生じる。別ファイルならこれらの
//!    契約は**除外述語ゼロのまま**成立する（[`Nav`](crate::nav::Nav) は
//!    1 行も変わらない）。
//! 2. `nav.toml` は #943 以降ページ部分が機械生成対象であり、#1017 が
//!    109 件を追記する予定。別ファイルなら nav 生成ロジックと衝突しない。
//! 3. パーサのエラー型（[`RedirectError`]）を [`crate::nav::NavError`] と
//!    独立させられる。
//!
//! # 4 契約への例外が「除外述語ゼロ」で成立する構造
//!
//! リダイレクトページは [`crate::build::build_site`] 内で `pages` にも
//! `search_index_entries` にも積まれない（生成場所が構造的に分かれている
//! ため、`nav.all_pages()` を走査する既存ループが redirect を意識する必要が
//! ない）。したがって:
//!
//! - `search-index.json` … redirect の `from` は収集ループを一切通らないため
//!   最初から載らない
//! - `linkcheck::check_links` の既知ターゲット表 … 同様に最初から載らない
//!   （「サイト内リンクの解決先として `from` を扱わない」という設計文書 §4
//!   の規定を、除外フラグではなく生成経路の分離で満たす）
//! - `tests/site_nav.rs` のページ数期待値 … [`crate::nav::Nav::all_pages`]
//!   にリダイレクトは現れないため無変更
//! - `tests/no_js_contract.rs` の全ページ sweep … これだけは dist 配下の
//!   `*.html` を無差別に再帰列挙するため唯一の分岐点になるが、[`output_path`]
//!   を経由した機械導出（手書き除外リストではない）で対処する。加えて、
//!   `from` が実ページ path と衝突する宣言は [`validate_against_nav`] が
//!   ビルド失敗にするため、「実ページを redirects.toml 経由で sweep から
//!   除外する」ことは構造的に不可能。詳細は `tests/no_js_contract.rs` の
//!   doc コメントを参照。

use std::collections::BTreeSet;
use std::fmt;

use fandhe_frontend_core::{el, text, Node};

use crate::layout;
use crate::nav::{is_safe_page_path, Nav};

/// `site/redirects.toml`（`repo_root` からの相対パス）。
pub const MANIFEST_REL_PATH: &str = "site/redirects.toml";

/// 入力サイズの上限（`crate::nav::MAX_INPUT_BYTES` と同値。DoS 抑止）。
const MAX_INPUT_BYTES: usize = 1024 * 1024;

/// `[[redirect]]` 1 件分。`from`/`to` はいずれも [`parse_redirects`] の時点で
/// パス形状検証（`/` 始まり・`/` 終わり・空セグメント禁止・許可セグメント）
/// 済みだが、`to` が実在ページであることの検証は [`validate_against_nav`]
/// が別途行う（`nav::Page`/`nav::validate_sources` と同型の 2 段階検証）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect {
    /// 旧 URL（サイト内絶対パス、`/` 始まり・`/` 終わり）。
    pub from: String,
    /// 移転先 URL（サイト内絶対パス）。[`validate_against_nav`] が
    /// `nav.all_pages()` に実在することを検証する。
    pub to: String,
}

/// [`parse_redirects`] の結果。宣言順を保持する。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Redirects {
    pub entries: Vec<Redirect>,
}

/// [`parse_redirects`] / [`validate_against_nav`] の失敗理由。
///
/// `Display` は行番号・理由・サイト内パスのみを含み、入力全文・絶対パス・
/// 環境変数は含めない（`security.md` の機微情報露出防止方針、
/// [`crate::nav::NavError`] と同方針）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedirectError {
    /// 入力サイズが [`MAX_INPUT_BYTES`] を超えた。
    TooLarge,
    /// 構文エラー（未知のテーブル・未知のキー・非対応の値型・重複キー等）。
    Parse {
        /// 1 始まりの行番号。ファイル全体に関するエラーは `0`。
        line: usize,
        /// エラー理由。
        message: String,
    },
    /// 必須キーが欠落している。
    MissingKey {
        /// 欠落箇所（常に `"redirect"`）。
        context: &'static str,
        /// 欠落したキー名。
        key: &'static str,
    },
    /// `from`/`to` のいずれかがパス形状の安全条件（`/` 始まり・`/` 終わり・
    /// 空セグメント禁止・セグメントのホワイトリスト）を満たさない。
    UnsafePath {
        /// 1 始まりの行番号。
        line: usize,
        /// 拒否されたパス文字列。
        path: String,
    },
    /// `from` が複数回宣言されている。
    DuplicateFrom(String),
    /// `from` が `nav.toml` の実ページ `path` と衝突している（実ページを
    /// リダイレクトが覆い隠す事故の防止）。
    CollidesWithPage(String),
    /// `to` が `nav.toml` に実在するページを指していない（「宣言のみで
    /// 実体が無い移転先」の禁止）。
    UnknownTarget {
        /// 対応する `from`。
        from: String,
        /// 実在しない `to`。
        to: String,
    },
}

impl fmt::Display for RedirectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RedirectError::TooLarge => {
                write!(
                    f,
                    "{MANIFEST_REL_PATH} exceeds the {MAX_INPUT_BYTES} byte size limit"
                )
            }
            RedirectError::Parse { line, message } => {
                write!(f, "{MANIFEST_REL_PATH}:{line}: {message}")
            }
            RedirectError::MissingKey { context, key } => {
                write!(f, "missing required key `{key}` in [{context}]")
            }
            RedirectError::UnsafePath { line, path } => write!(
                f,
                "{MANIFEST_REL_PATH}:{line}: path `{path}` must start and end with `/` with no empty segments, segments limited to alphanumerics, `-`, `_`"
            ),
            RedirectError::DuplicateFrom(path) => write!(f, "duplicate redirect `from` `{path}`"),
            RedirectError::CollidesWithPage(path) => write!(
                f,
                "redirect `from` `{path}` collides with an existing nav.toml page path"
            ),
            RedirectError::UnknownTarget { from, to } => write!(
                f,
                "redirect `from` `{from}` targets `to` `{to}` which is not a page registered in nav.toml"
            ),
        }
    }
}

impl std::error::Error for RedirectError {}

/// パース中に組み立て途上の 1 件。
struct RedirectBuilder {
    from: Option<String>,
    to: Option<String>,
}

/// パース中の現在テーブル（`[[redirect]]` の外側にいるかどうか）。
enum Ctx {
    None,
    Redirect(usize),
}

fn parse_err(line: usize, message: impl Into<String>) -> RedirectError {
    RedirectError::Parse {
        line,
        message: message.into(),
    }
}

/// テーブルヘッダ・値の後続部分が「空、または `#` 始まりのコメント」で
/// あることを検証する（`crate::nav::check_trailing` と同方針・別実装。
/// エラー型が異なるためコード共有はしない）。
fn check_trailing(rest: &str, line: usize) -> Result<(), RedirectError> {
    let rest = rest.trim_start();
    if rest.is_empty() || rest.starts_with('#') {
        Ok(())
    } else {
        Err(parse_err(
            line,
            format!("unexpected trailing content `{rest}`"),
        ))
    }
}

/// `value_part`（`=` の右側、先頭空白は trim 済み）からダブルクォート
/// 文字列 1 個を読み取る。エスケープは `\"` `\\` `\n` `\t` のみ対応
/// （`crate::nav::parse_quoted_string` と同方針・別実装）。
fn parse_quoted_string(value_part: &str, line: usize) -> Result<(String, &str), RedirectError> {
    let mut chars = value_part.char_indices();
    match chars.next() {
        Some((_, '"')) => {}
        _ => return Err(parse_err(
            line,
            "expected a double-quoted string value (this parser accepts no other TOML value type)",
        )),
    }

    let mut out = String::new();
    loop {
        match chars.next() {
            None => return Err(parse_err(line, "unterminated string literal")),
            Some((idx, '"')) => {
                let remainder = &value_part[idx + '"'.len_utf8()..];
                return Ok((out, remainder));
            }
            Some((_, '\\')) => match chars.next() {
                Some((_, '"')) => out.push('"'),
                Some((_, '\\')) => out.push('\\'),
                Some((_, 'n')) => out.push('\n'),
                Some((_, 't')) => out.push('\t'),
                Some((_, other)) => {
                    return Err(parse_err(
                        line,
                        format!("unsupported escape sequence `\\{other}`"),
                    ))
                }
                None => return Err(parse_err(line, "unterminated escape sequence")),
            },
            Some((_, c)) => out.push(c),
        }
    }
}

fn set_once(
    slot: &mut Option<String>,
    value: String,
    line: usize,
    name: &str,
) -> Result<(), RedirectError> {
    if slot.is_some() {
        return Err(parse_err(line, format!("duplicate key `{name}`")));
    }
    *slot = Some(value);
    Ok(())
}

/// `path` が redirect の `from`/`to` として安全かを判定する。
///
/// [`is_safe_page_path`]（`nav.toml` の `page.path` allowlist）を土台とし、
/// さらに 2 点厳しくする:
///
/// 1. 空セグメント（`//` や `/a//b/`）を拒否する。`nav::is_safe_page_path`
///    は対称性のため `"//"` を許容するが、`ssg::normalize_page_path`
///    （書き出し時の正規化）はこれを拒否するため、redirect 側を緩いままに
///    すると行番号付き `RedirectError` ではなく不透明な `SsgError` で
///    ビルドが落ちる。`//evil.example` のような protocol-relative URL の
///    構文形も併せて構造的に遮断する。
/// 2. `from == "/"`（サイトトップ単体）を拒否する。サイトトップを
///    リダイレクトで覆い隠すことを常に禁止する（`to == "/"` は許可する。
///    「何かをサイトトップへ移転する」ことまでは禁止しない）。
fn is_safe_redirect_from(path: &str) -> bool {
    is_safe_page_path(path) && path != "/" && !path.contains("//")
}

/// `to` 用のパス検証。`from` と異なり `"/"`（サイトトップへの移転）は許可する。
fn is_safe_redirect_to(path: &str) -> bool {
    is_safe_page_path(path) && !path.contains("//")
}

/// `site/redirects.toml`（[`MANIFEST_REL_PATH`]）の内容（文字列）をパースし、
/// `from`/`to` のパス形状検証・`from` の重複検証までを行う純関数。
/// `nav.toml` との突合（`to` の実在確認・`from` の衝突検証）は
/// [`validate_against_nav`] が別途行う（`nav::parse_nav` / `nav::validate_sources`
/// と同じ 2 段階の役割分担）。
///
/// # Errors
///
/// 対応外の TOML 構文・必須キー欠落・`from`/`to` の形式違反・`from` 重複の
/// いずれかがあれば [`RedirectError`] を返す。
pub fn parse_redirects(input: &str) -> Result<Redirects, RedirectError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(RedirectError::TooLarge);
    }

    let mut ctx = Ctx::None;
    let mut redirects: Vec<RedirectBuilder> = Vec::new();

    for (line_no0, raw_line) in input.lines().enumerate() {
        let line = line_no0 + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("[[") {
            let end = rest
                .find("]]")
                .ok_or_else(|| parse_err(line, "expected closing `]]`"))?;
            let header = rest[..end].trim();
            check_trailing(&rest[end + 2..], line)?;
            match header {
                "redirect" => {
                    redirects.push(RedirectBuilder {
                        from: None,
                        to: None,
                    });
                    ctx = Ctx::Redirect(redirects.len() - 1);
                }
                other => return Err(parse_err(line, format!("unknown table `[[{other}]]`"))),
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix('[') {
            let end = rest
                .find(']')
                .ok_or_else(|| parse_err(line, "expected closing `]`"))?;
            let header = rest[..end].trim();
            return Err(parse_err(line, format!("unknown table `[{header}]`")));
        }

        let eq = trimmed
            .find('=')
            .ok_or_else(|| parse_err(line, "expected `key = \"value\"`"))?;
        let key = trimmed[..eq].trim();
        if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(parse_err(line, format!("invalid key `{key}`")));
        }
        let value_part = trimmed[eq + 1..].trim_start();
        let (value, remainder) = parse_quoted_string(value_part, line)?;
        check_trailing(remainder, line)?;

        match ctx {
            Ctx::None => return Err(parse_err(line, "key-value pair outside of any table")),
            Ctx::Redirect(idx) => {
                let entry = &mut redirects[idx];
                match key {
                    "from" => set_once(&mut entry.from, value, line, "redirect.from")?,
                    "to" => set_once(&mut entry.to, value, line, "redirect.to")?,
                    other => {
                        return Err(parse_err(
                            line,
                            format!("unknown key `{other}` in [[redirect]]"),
                        ))
                    }
                }
            }
        }
    }

    let mut seen_from: BTreeSet<String> = BTreeSet::new();
    let mut entries = Vec::with_capacity(redirects.len());
    for entry in redirects {
        let from = entry.from.ok_or(RedirectError::MissingKey {
            context: "redirect",
            key: "from",
        })?;
        let to = entry.to.ok_or(RedirectError::MissingKey {
            context: "redirect",
            key: "to",
        })?;
        if !is_safe_redirect_from(&from) {
            return Err(RedirectError::UnsafePath {
                line: 0,
                path: from,
            });
        }
        if !is_safe_redirect_to(&to) {
            return Err(RedirectError::UnsafePath { line: 0, path: to });
        }
        if !seen_from.insert(from.clone()) {
            return Err(RedirectError::DuplicateFrom(from));
        }
        entries.push(Redirect { from, to });
    }

    Ok(Redirects { entries })
}

/// [`parse_redirects`] が返した宣言を `nav.toml` の全ページと突合する。
///
/// 比較は正規化形（末尾 `/` の有無を同一視）で行う。
/// `fandhe_frontend_server::ssg::normalize_page_path` が `/a/` と `/a` を
/// 同一ファイルへ写すため、素の文字列比較では衝突をすり抜ける
/// （本モジュールの `from`/`to` は allowlist 検証済みで必ず末尾 `/` を
/// 持つため、実際には無害化のための保険的な正規化である）。
///
/// # Errors
///
/// - `to` が `nav.all_pages()` に実在しない → [`RedirectError::UnknownTarget`]
/// - `from` が `nav.all_pages()` の既存ページ path と衝突する →
///   [`RedirectError::CollidesWithPage`]
pub fn validate_against_nav(redirects: &Redirects, nav: &Nav) -> Result<(), RedirectError> {
    let page_paths: BTreeSet<&str> = nav.all_pages().map(|p| p.path.as_str()).collect();

    for redirect in &redirects.entries {
        if page_paths.contains(redirect.from.as_str()) {
            return Err(RedirectError::CollidesWithPage(redirect.from.clone()));
        }
        if !page_paths.contains(redirect.to.as_str()) {
            return Err(RedirectError::UnknownTarget {
                from: redirect.from.clone(),
                to: redirect.to.clone(),
            });
        }
    }
    Ok(())
}

/// リダイレクトページの出力パスを決める。`from` をそのまま
/// `fandhe_frontend_server::ssg::generate_pages` へ渡す `page.path`
/// として使う薄い契約点（`nav::Page::path` と同じ意味づけ）。
///
/// [`crate::build::build_site`] が `(output_path(from), redirect_page(...))`
/// のタプルを組み立てる際に呼ぶ。分離した関数として存在する理由は、
/// `tests/no_js_contract.rs` が「dist 配下の `*.html` のうちどれが
/// リダイレクト由来か」を `site/redirects.toml` から機械導出する際に
/// 同じ関数を経由し、手書きの除外リストを持たないため（モジュール doc
/// 「4 契約への例外が『除外述語ゼロ』で成立する構造」参照）。
pub fn output_path(from: &str) -> String {
    from.to_string()
}

/// 旧 URL 互換の案内ページ（`meta refresh` + `rel=canonical` + `noindex` +
/// 静的フォールバックリンク）を組み立てる。
///
/// `to_href` は呼び出し元（[`crate::build::build_site`]）が必ず
/// [`layout::asset_href`] で組み立てた値を渡すこと（`base_path` を反映した
/// 単一実装点。素の `to` を渡すと GitHub Pages プロジェクトサイト
/// （`base_path` 非空）で壊れる）。
///
/// # セキュリティ上の不変条件（多層防御）
///
/// - 組み立ては `fandhe_frontend_core` のノード木 API（[`el`]/[`text`]）
///   のみで行い、HTML 文字列の直接組み立て（`format!("<meta ...>")`）は
///   しない。`content` 属性値の `format!("0; url={to_href}")` は
///   **マークアップの組み立てではなく属性値の組み立て**であり、
///   [`fandhe_frontend_core::render`] の既定エスケープ（属性値は
///   `"` 等を実体参照へ変換する）を通るため REQ-1 の対象範囲内で許容される。
/// - `<meta http-equiv="refresh" content="...">` の `content` 属性は
///   `fandhe_frontend_core::url::is_url_attr` の対象外であり、`is_safe_url`
///   の検証を通らない。さらに `is_safe_url` はスキーム無しの値
///   （`//evil.example` を含む）を一律 `true` にする実装のため、**この
///   `redirect_page` を呼ぶ側（[`parse_redirects`]/[`validate_against_nav`]
///   の allowlist・実在検証）が refresh 先の唯一の防壁である**。
///   `fandhe_frontend_core` の URL 検証を当てにしない設計判断を doc として
///   明示する。
fn redirect_document(site_title: &str, to_href: &str) -> Node {
    let head = el(
        "head",
        vec![],
        vec![
            el("meta", vec![("charset", "utf-8")], vec![]),
            el(
                "meta",
                vec![
                    ("http-equiv", "refresh"),
                    ("content", &format!("0; url={to_href}")),
                ],
                vec![],
            ),
            el(
                "link",
                vec![("rel", "canonical"), ("href", to_href)],
                vec![],
            ),
            el(
                "meta",
                vec![("name", "robots"), ("content", "noindex")],
                vec![],
            ),
            el(
                "title",
                vec![],
                vec![text(format!("移転しました | {site_title}"))],
            ),
        ],
    );

    let body = el(
        "body",
        vec![],
        vec![el(
            "p",
            vec![],
            vec![
                text(
                    "このページは移動しました。自動的に移動しない場合は次のリンクを開いてください: "
                        .to_string(),
                ),
                el(
                    "a",
                    vec![("href", to_href)],
                    vec![text(to_href.to_string())],
                ),
            ],
        )],
    );

    el("html", vec![("lang", "ja")], vec![head, body])
}

/// [`crate::build::build_site`] から呼ぶ公開エントリ。`to`（サイト内絶対
/// パス）を `base_path` 込みの href へ変換した上で [`redirect_document`]
/// を組み立てる。href 変換を呼び出し元へ委ねない（`base_path` 反映漏れの
/// 事故を単一実装点で防ぐ）。
pub fn redirect_page(site_title: &str, base_path: &str, to: &str) -> Node {
    let to_href = layout::asset_href(base_path, to);
    redirect_document(site_title, &to_href)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn minimal_nav_toml() -> &'static str {
        r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Guide"
index_path = "/"

[[section.page]]
title = "Intro"
source = "site/intro.md"
path = "/"

[[section.page]]
title = "Components"
source = "site/components.md"
path = "/components/pre-styled-ui/"
"#
    }

    // ---- parse_redirects: 正常系 ----

    #[test]
    fn parses_single_redirect() {
        let redirects = parse_redirects(
            r#"
[[redirect]]
from = "/components/"
to = "/components/pre-styled-ui/"
"#,
        )
        .expect("valid manifest should parse");
        assert_eq!(redirects.entries.len(), 1);
        assert_eq!(redirects.entries[0].from, "/components/");
        assert_eq!(redirects.entries[0].to, "/components/pre-styled-ui/");
    }

    #[test]
    fn empty_manifest_parses_to_zero_entries() {
        let redirects = parse_redirects("# no redirects yet\n").expect("empty manifest is valid");
        assert!(redirects.entries.is_empty());
    }

    #[test]
    fn ignores_comments_and_blank_lines() {
        let redirects = parse_redirects(
            r#"
# leading comment

[[redirect]]
# comment inside table
from = "/a/"   # trailing comment
to = "/b/"

"#,
        )
        .expect("comments should be ignored");
        assert_eq!(redirects.entries.len(), 1);
    }

    // ---- parse_redirects: fail-closed ----

    #[test]
    fn rejects_unknown_table() {
        let err = parse_redirects("[[redirects]]\nfrom = \"/a/\"\nto = \"/b/\"\n").unwrap_err();
        match err {
            RedirectError::Parse { line, message } => {
                assert_eq!(line, 1);
                assert!(message.contains("unknown table"));
            }
            other => panic!("expected Parse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unknown_top_level_single_bracket_table() {
        let err = parse_redirects("[redirect]\nfrom = \"/a/\"\n").unwrap_err();
        assert!(matches!(err, RedirectError::Parse { line: 1, .. }));
    }

    #[test]
    fn rejects_unknown_key() {
        let err = parse_redirects("[[redirect]]\nfroom = \"/a/\"\nto = \"/b/\"\n").unwrap_err();
        match err {
            RedirectError::Parse { line, message } => {
                assert_eq!(line, 2);
                assert!(message.contains("unknown key"));
            }
            other => panic!("expected Parse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_missing_from() {
        let err = parse_redirects("[[redirect]]\nto = \"/b/\"\n").unwrap_err();
        assert!(matches!(
            err,
            RedirectError::MissingKey {
                context: "redirect",
                key: "from"
            }
        ));
    }

    #[test]
    fn rejects_missing_to() {
        let err = parse_redirects("[[redirect]]\nfrom = \"/a/\"\n").unwrap_err();
        assert!(matches!(
            err,
            RedirectError::MissingKey {
                context: "redirect",
                key: "to"
            }
        ));
    }

    #[test]
    fn rejects_non_string_value() {
        let err = parse_redirects("[[redirect]]\nfrom = 1\nto = \"/b/\"\n").unwrap_err();
        assert!(matches!(err, RedirectError::Parse { line: 2, .. }));
    }

    #[test]
    fn rejects_trailing_content_after_closing_brackets() {
        let err =
            parse_redirects("[[redirect]] extra\nfrom = \"/a/\"\nto = \"/b/\"\n").unwrap_err();
        assert!(matches!(err, RedirectError::Parse { line: 1, .. }));
    }

    #[test]
    fn rejects_unterminated_table_header() {
        let err = parse_redirects("[[redirect\nfrom = \"/a/\"\nto = \"/b/\"\n").unwrap_err();
        assert!(matches!(err, RedirectError::Parse { line: 1, .. }));
    }

    #[test]
    fn rejects_input_exceeding_size_limit() {
        let mut input = String::from("[[redirect]]\nfrom = \"/a/\"\nto = \"/b/\"\n");
        input.push_str(&"#".repeat(MAX_INPUT_BYTES + 1));
        let err = parse_redirects(&input).unwrap_err();
        assert_eq!(err, RedirectError::TooLarge);
    }

    #[test]
    fn rejects_key_value_pair_outside_any_table() {
        let err = parse_redirects("from = \"/a/\"\n").unwrap_err();
        assert!(matches!(err, RedirectError::Parse { line: 1, .. }));
    }

    #[test]
    fn rejects_duplicate_key_within_entry() {
        let err = parse_redirects("[[redirect]]\nfrom = \"/a/\"\nfrom = \"/a2/\"\nto = \"/b/\"\n")
            .unwrap_err();
        assert!(matches!(err, RedirectError::Parse { line: 3, .. }));
    }

    #[test]
    fn rejects_duplicate_from_across_entries() {
        let err = parse_redirects(
            r#"
[[redirect]]
from = "/a/"
to = "/b/"

[[redirect]]
from = "/a/"
to = "/c/"
"#,
        )
        .unwrap_err();
        assert_eq!(err, RedirectError::DuplicateFrom("/a/".to_string()));
    }

    // ---- パス検証拒否 ----

    #[test]
    fn rejects_scheme_url_as_to() {
        let err = parse_redirects("[[redirect]]\nfrom = \"/a/\"\nto = \"http://evil.example/\"\n")
            .unwrap_err();
        assert!(matches!(err, RedirectError::UnsafePath { .. }));
    }

    #[test]
    fn rejects_protocol_relative_url_as_to() {
        let err = parse_redirects("[[redirect]]\nfrom = \"/a/\"\nto = \"//evil.example/\"\n")
            .unwrap_err();
        assert!(matches!(err, RedirectError::UnsafePath { .. }));
    }

    #[test]
    fn rejects_javascript_scheme_as_to() {
        let err = parse_redirects("[[redirect]]\nfrom = \"/a/\"\nto = \"javascript:alert(1)\"\n")
            .unwrap_err();
        assert!(matches!(err, RedirectError::UnsafePath { .. }));
    }

    #[test]
    fn rejects_dot_dot_segment() {
        let err = parse_redirects("[[redirect]]\nfrom = \"/a/../b/\"\nto = \"/c/\"\n").unwrap_err();
        assert!(matches!(err, RedirectError::UnsafePath { .. }));
    }

    #[test]
    fn rejects_double_slash_in_path() {
        let err = parse_redirects("[[redirect]]\nfrom = \"/a//b/\"\nto = \"/c/\"\n").unwrap_err();
        assert!(matches!(err, RedirectError::UnsafePath { .. }));
    }

    #[test]
    fn rejects_bare_root_as_from() {
        let err = parse_redirects("[[redirect]]\nfrom = \"/\"\nto = \"/c/\"\n").unwrap_err();
        assert!(matches!(err, RedirectError::UnsafePath { .. }));
    }

    #[test]
    fn allows_root_as_to() {
        let redirects = parse_redirects("[[redirect]]\nfrom = \"/old/\"\nto = \"/\"\n")
            .expect("to = \"/\" should be allowed");
        assert_eq!(redirects.entries[0].to, "/");
    }

    #[test]
    fn rejects_missing_leading_slash() {
        let err =
            parse_redirects("[[redirect]]\nfrom = \"components/\"\nto = \"/c/\"\n").unwrap_err();
        assert!(matches!(err, RedirectError::UnsafePath { .. }));
    }

    #[test]
    fn rejects_missing_trailing_slash() {
        let err =
            parse_redirects("[[redirect]]\nfrom = \"/components\"\nto = \"/c/\"\n").unwrap_err();
        assert!(matches!(err, RedirectError::UnsafePath { .. }));
    }

    #[test]
    fn rejects_unsafe_characters_in_segment() {
        let err =
            parse_redirects("[[redirect]]\nfrom = \"/comp onents/\"\nto = \"/c/\"\n").unwrap_err();
        assert!(matches!(err, RedirectError::UnsafePath { .. }));

        let err2 = parse_redirects("[[redirect]]\nfrom = \"/comp\\\\onents/\"\nto = \"/c/\"\n")
            .unwrap_err();
        assert!(matches!(err2, RedirectError::UnsafePath { .. }));
    }

    #[test]
    fn allows_underscore_and_hyphen_segments() {
        let redirects = parse_redirects("[[redirect]]\nfrom = \"/a_b-c/\"\nto = \"/d/\"\n")
            .expect("underscore/hyphen segments should be allowed");
        assert_eq!(redirects.entries[0].from, "/a_b-c/");
    }

    // ---- Display: 機微情報の非露出 ----

    #[test]
    fn display_does_not_leak_full_input_or_absolute_paths() {
        let err = parse_redirects("[[redirects]]\n").unwrap_err();
        let rendered = err.to_string();
        assert!(rendered.contains(MANIFEST_REL_PATH));
        assert!(!rendered.contains('\n'));
    }

    // ---- validate_against_nav ----

    #[test]
    fn validate_against_nav_accepts_declaration_targeting_existing_page() {
        let nav = crate::nav::parse_nav(minimal_nav_toml()).expect("fixture nav should parse");
        let redirects = parse_redirects(
            "[[redirect]]\nfrom = \"/components/\"\nto = \"/components/pre-styled-ui/\"\n",
        )
        .expect("valid manifest");
        assert!(validate_against_nav(&redirects, &nav).is_ok());
    }

    #[test]
    fn validate_against_nav_rejects_unknown_target() {
        let nav = crate::nav::parse_nav(minimal_nav_toml()).expect("fixture nav should parse");
        let redirects =
            parse_redirects("[[redirect]]\nfrom = \"/old/\"\nto = \"/does-not-exist/\"\n")
                .expect("valid manifest");
        let err = validate_against_nav(&redirects, &nav).unwrap_err();
        match err {
            RedirectError::UnknownTarget { from, to } => {
                assert_eq!(from, "/old/");
                assert_eq!(to, "/does-not-exist/");
            }
            other => panic!("expected UnknownTarget, got {other:?}"),
        }
    }

    #[test]
    fn validate_against_nav_rejects_from_colliding_with_existing_page() {
        let nav = crate::nav::parse_nav(minimal_nav_toml()).expect("fixture nav should parse");
        let redirects =
            parse_redirects("[[redirect]]\nfrom = \"/\"\nto = \"/components/pre-styled-ui/\"\n");
        // `from = "/"` はパス検証自体で拒否される（`is_safe_redirect_from`）。
        // 衝突検知は "/" 以外のページと衝突するケースで確認する。
        assert!(matches!(redirects, Err(RedirectError::UnsafePath { .. })));

        let redirects =
            parse_redirects("[[redirect]]\nfrom = \"/components/pre-styled-ui/\"\nto = \"/\"\n")
                .expect("valid manifest shape");
        let err = validate_against_nav(&redirects, &nav).unwrap_err();
        assert_eq!(
            err,
            RedirectError::CollidesWithPage("/components/pre-styled-ui/".to_string())
        );
    }

    // ---- redirect_page / redirect_document ----

    #[test]
    fn redirect_page_contains_the_four_required_elements() {
        let node = redirect_page("Docs", "/fandhe-frontend", "/components/pre-styled-ui/");
        let html = render(&node);
        assert!(html.contains(
            r#"<meta http-equiv="refresh" content="0; url=/fandhe-frontend/components/pre-styled-ui/">"#
        ));
        assert!(html.contains(
            r#"<link rel="canonical" href="/fandhe-frontend/components/pre-styled-ui/">"#
        ));
        assert!(html.contains(r#"<meta name="robots" content="noindex">"#));
        assert!(html.contains(
            r#"<a href="/fandhe-frontend/components/pre-styled-ui/">/fandhe-frontend/components/pre-styled-ui/</a>"#
        ));
    }

    #[test]
    fn redirect_page_reflects_base_path_in_href() {
        let node = redirect_page("Docs", "/fandhe-frontend", "/components/pre-styled-ui/");
        let html = render(&node);
        assert!(html.contains("/fandhe-frontend/components/pre-styled-ui/"));
        // base_path 込みでない素の to がどこにも現れないこと（asset_href
        // を経由せず素の to を使う実装への回帰防止）。
        assert!(!html.contains("url=/components/pre-styled-ui/\""));
    }

    #[test]
    fn redirect_page_has_no_chrome_elements() {
        let node = redirect_page("Docs", "", "/x/");
        let html = render(&node);
        assert!(!html.contains("class="));
        assert!(!html.contains("<script"));
        assert!(!html.contains(r#"<link rel="stylesheet""#));
    }

    /// エスケープ回帰（多層防御）: allowlist を迂回して `redirect_page` を
    /// 直接呼んでも、属性値・テキストの既定エスケープにより生の `<script>`
    /// タグや生のダブルクォートが出力に現れないことを固定する。allowlist が
    /// 唯一の防壁ではないことの直接証明（モジュール doc 参照）。
    #[test]
    fn redirect_page_escapes_malicious_input_even_if_validation_is_bypassed() {
        let node = redirect_page("t", "", "/x\"><script>alert(1)</script>/");
        let html = render(&node);
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(!html.contains("\"><script>"));
    }

    // ---- output_path ----

    #[test]
    fn output_path_is_identity() {
        assert_eq!(output_path("/components/"), "/components/");
    }
}

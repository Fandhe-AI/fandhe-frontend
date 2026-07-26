//! `site/nav.toml`（docs サイトのナビゲーション構成マニフェスト）のパース、
//! およびサイドバー・前後ページナビの [`Node`] 生成を担うモジュール。
//!
//! # 呼び出し文脈
//!
//! 後続イシュー #470 の `main.rs` から [`parse_nav`] → [`validate_sources`]
//! の順で呼ばれ、得られた [`Nav`] を #469（`layout.rs`）が [`sidebar`] /
//! [`prev_next_nav`] 経由でページレイアウトへ埋め込む。最終的な HTML は
//! `fandhe_frontend_server::ssg::generate_pages`（PR #477）へ渡される
//! `(path, Node)` の一部として書き出される。
//!
//! # 対応する TOML サブセット
//!
//! `nav.toml` は以下の構文のみを許可するサブセットとして扱う（それ以外は
//! すべて `NavError::Parse` で明示的に失敗する。fail-closed。未対応構文を
//! 黙って無視することはしない）。
//!
//! - `#` から始まる行コメント、および文字列値の終端後に続く `# ...`
//! - `[site]` テーブル（`title` / `base_path` の 2 キー）
//! - `[[section]]` array-of-tables（`title` / `index_path` の 2 キー。
//!   `index_path` はセクショントップページの出力 URL パスを指す必須項目
//!   （イシュー #1010）。ヘッダー href（#1012）・サイドバースコープ判定
//!   （#1013）が参照する唯一の情報源になる）
//! - `[[section.page]]` array-of-tables（直前の `[[section]]` に属する。
//!   `title` / `source` / `path` の 3 キー）
//! - `[[section.group]]` array-of-tables（直前の `[[section]]` に属する
//!   カテゴリ。`title` の 1 キーのみ。イシュー #939）
//! - `[[section.group.page]]` array-of-tables（直前の `[[section.group]]`
//!   に属する。`title` / `source` / `path` の 3 キー。イシュー #939）
//! - `key = "value"`（ダブルクォート文字列のみ。エスケープは `\"` `\\`
//!   `\n` `\t` の 4 種類のみ対応）
//!
//! グループの入れ子は 1 段のみ（`[[section.group.group]]` は未知テーブル
//! として明示的にエラーになる）。1 つの `[[section]]` は直下ページ
//! （`[[section.page]]`）とグループ（`[[section.group]]`）を同時に持って
//! よく、その場合の描画順・走査順は「直下ページ → グループ（宣言順）→
//! グループ内ページ（宣言順）」に固定する（[`Section::all_pages`] /
//! [`Nav::all_pages`] 参照）。`[[section.page]]` が `[[section.group]]`
//! より後方に現れても直下ページとして扱い、エラーにはしない（#943 が
//! 機械生成する `nav.toml` へ宣言順の追加制約を課さないための意図的な
//! 仕様）。
//!
//! 整数・真偽値・inline table・複数行文字列・配列などは非対応であり、
//! 出現した場合はエラーにする。
//!
//! # `crates/cli/src/toml.rs` を流用しない理由
//!
//! `fandhe-frontend-cli` の `structure.toml` 用パーサ（`crates/cli/src/toml.rs`）
//! は (a) `[[a]]` 形式の array-of-tables を明示的に拒否しており本モジュールが
//! 必要とする `[[section]]` / `[[section.page]]` を扱えない、(b) `cli` は
//! bin クレートで `lib` ターゲットを持たずクレート間で参照できない、(c) 仮に
//! ライブラリ化しても `docs-site` から `cli` への依存は `structure.toml` の
//! クレート責務境界（`docs-site` は `core`/`app`/`server` のみを
//! `depends_on` として宣言）に反する — の 3 点から、コード共有はせず
//! 同じ設計方針（fail-closed・行番号付きエラー・入力サイズ上限・
//! `unwrap()`/`expect()`/`panic!` 不使用）を踏襲した専用の最小パーサを
//! 本モジュールに自前実装する（イシュー #468 実装計画より）。

use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;

use fandhe_frontend_core::{el, text, Node};
// サイドバー（イシュー #756）: pre-styled-ui が薄く再エクスポートする
// headless nav_list の自由関数を直接使う。styled `nav_list::root`（本クレート
// 未使用）は呼び出し側の `class` を drop_class_attr で除去するため、
// `class="sidebar"` を温存したい本モジュールは headless の `root`
// （`fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui` 経由の
// 再エクスポート crate）を直接呼ぶ。これにより `crates/docs-site/Cargo.toml`
// へ `fandhe-frontend-headless-ui` への新規直接依存を追加せずに済む
// （イシュー #693 の既存整理を維持する）。`heading`/`list`/`item`/`link` は
// class を持たない純粋な anatomy パーツのため styled 層の再エクスポート
// （`fandhe_frontend_pre_styled_ui::nav_list::{heading, item, link, list}`）
// をそのまま使う。
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::nav_list::root as nav_list_root;
use fandhe_frontend_pre_styled_ui::nav_list::{heading, item, link as nav_link, list};
// 前後ページャ（イシュー #756）: 同じ理由で LinkOverlay も headless
// `root`（class 温存のため）+ styled 層再エクスポートの `overlay` を使う。
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::link_overlay::root as link_overlay_root;
use fandhe_frontend_pre_styled_ui::link_overlay::overlay as link_overlay_overlay;

/// `nav.toml` 入力の上限サイズ（`crates/cli/src/toml.rs` の DoS 抑止方針と
/// 同値。再帰を使わない行単位パースのためネスト深度問題は生じないが、
/// 巨大入力そのものによる処理時間膨張は別途抑止する）。
const MAX_INPUT_BYTES: usize = 1024 * 1024;

/// `nav.toml` 全体をパースした結果のモデル。フィールドはすべて検証済み
/// （必須キー充足・`page.path` / `site.base_path` 形式・`page.path` 重複なし）。
/// `page.source` の実ファイル存在は [`validate_sources`] が別途担う
/// （パーサ本体を FS 非依存に保ち単体テストしやすくするため）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nav {
    /// サイト全体設定。
    pub site: Site,
    /// 宣言順を保持したセクション列。
    pub sections: Vec<Section>,
}

impl Nav {
    /// 全セクションを宣言順に、各セクション内は [`Section::all_pages`]
    /// の順序で連結した「文書順」の全ページ列を返す**唯一の正規走査経路**。
    ///
    /// [`validate_sources`] / [`prev_next`] / [`crate::linkcheck::source_to_path_map`] /
    /// `crate::build::build_site` のページ生成ループはすべて本イテレータを
    /// 経由する。グループ配下ページが検証・ビルド・リンク検査から漏れる
    /// サイレントな取りこぼしを防ぐため、`nav.sections` を直接二重ループで
    /// 手繰る新しい走査経路を作らないこと（イシュー #939）。
    pub fn all_pages(&self) -> impl Iterator<Item = &Page> {
        self.sections.iter().flat_map(|s| s.all_pages())
    }

    /// `path` を含むセクションを返す（イシュー #1013 のサイドバースコープ
    /// 判定 — 現在ページが属するセクションのみへ絞り込む — が利用する
    /// 解決 API）。
    ///
    /// 走査は必ず [`Section::all_pages`]（本モジュールが定める唯一の
    /// 正規走査経路）を経由する。`pages` / `groups` を個別に手繰る
    /// 二重ループをここで新設しない（グループ配下ページの取りこぼしを
    /// 防ぐための規約、同メソッド rustdoc 参照）。
    pub fn section_for_path(&self, path: &str) -> Option<&Section> {
        self.sections
            .iter()
            .find(|s| s.all_pages().any(|p| p.path == path))
    }
}

/// `[site]` テーブル。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Site {
    /// サイトタイトル。
    pub title: String,
    /// GitHub Pages プロジェクトサイト等でルート以外にホストする場合の
    /// ベースパス。`""` または `/` 始まり・`/` 終わりでない文字列。
    pub base_path: String,
}

/// `[[section]]` 1 件分。
///
/// `pages`（直下ページ）と `groups`（カテゴリ）は同時に存在してよい
/// （イシュー #939）。両方が空の場合のみ [`NavError::EmptySection`] になる。
/// 走査は必ず [`Section::all_pages`] を経由し、直下ページ・グループ配下
/// ページを個別に手繰る二重ループを新設しないこと（唯一の正規走査経路。
/// 順序契約が意味を持たなくなる）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// サイドバーの見出しとして表示するセクションタイトル。
    pub title: String,
    /// セクショントップページの出力 URL パス（必須、イシュー #1010）。
    /// このセクション配下（直下ページ or グループ内ページ）のいずれかの
    /// `page.path` と完全一致することが [`parse_nav`] のパース時点で
    /// 保証される（`validate_page_path` を通過済みの `page.path` 集合との
    /// 完全一致でのみ受理し、独立した形式検証は持たない。これにより
    /// `index_path ⊆ 生成ページの path 集合` が構造的な不変条件になる）。
    /// #1012（ヘッダー href）・#1013（サイドバースコープ判定、
    /// [`Nav::section_for_path`] 経由）が参照する唯一の情報源。
    pub index_path: String,
    /// 宣言順を保持した直下ページ列（グループに属さないページ）。
    pub pages: Vec<Page>,
    /// 宣言順を保持したグループ列（各グループのページは 1 件以上、
    /// 空グループはパース時点で [`NavError::EmptyGroup`]）。
    pub groups: Vec<Group>,
}

/// `[[section.group]]` 1 件分（`Components > カテゴリ` のような 1 段の
/// カテゴリ分類）。入れ子は許可しない（`[[section.group.group]]` は
/// パース時点で未知テーブルとしてエラーになる）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// カテゴリ見出しとして表示するグループタイトル。
    pub title: String,
    /// 宣言順を保持したページ列（1 件以上、空グループはパース時点でエラー）。
    pub pages: Vec<Page>,
}

impl Section {
    /// このセクション配下の全ページを「直下ページ → グループ（宣言順）→
    /// グループ内ページ（宣言順）」の順で列挙する、本モジュールが定める
    /// **唯一の正規走査経路**。
    ///
    /// この順序は §6-2 の描画順契約そのものであり、サイドバー階層描画
    /// （イシュー #940）を含む後続実装はここから逸脱しないこと
    /// （ドリフト防止のため、直下ページ・グループ配下ページを個別に
    /// 手繰る新たな二重ループを作らず、必ず本イテレータを使う）。
    pub fn all_pages(&self) -> impl Iterator<Item = &Page> {
        self.pages
            .iter()
            .chain(self.groups.iter().flat_map(|g| g.pages.iter()))
    }
}

/// `[[section.page]]` 1 件分。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    /// サイドバー・前後ナビのリンクテキスト。
    pub title: String,
    /// Markdown ソースファイルの `repo_root` からの相対パス
    /// （[`validate_sources`] が実在確認する）。
    pub source: String,
    /// 出力 URL パス。`/` 始まり・`/` 終わり必須。
    pub path: String,
}

/// [`parse_nav`] / [`validate_sources`] の失敗理由。
///
/// `Display` 実装は行番号と理由のみを含み、入力全文・絶対パス・環境変数は
/// 含めない（`security.md` の機微情報露出防止方針。`crates/cli/src/toml.rs`
/// の `TomlError` と同方針）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavError {
    /// 入力サイズが [`MAX_INPUT_BYTES`] を超えた。
    TooLarge,
    /// 構文エラー（未知のテーブル・未知のキー・非対応の値型・重複キー等）。
    Parse {
        /// 1 始まりの行番号。ファイル全体に関するエラーは `0`。
        line: usize,
        /// エラー理由（入力値の断片は含めても入力全文は含めない）。
        message: String,
    },
    /// 複数セクションにまたがり `page.path` が重複している。
    DuplicatePath(String),
    /// `page.source` が `repo_root` 配下のファイルとして実在しない。
    MissingSource(String),
    /// `page.source` が相対パスの安全条件（絶対パス禁止・`..` 禁止・
    /// `\` 禁止）を満たさない。
    UnsafeSource(String),
    /// `page.path` が `/` 始まり・`/` 終わり、またはセグメントの
    /// ホワイトリスト（英数字・`-`・`_`）を満たさない。
    UnsafePagePath(String),
    /// 必須キーが欠落している。
    MissingKey {
        /// 欠落箇所（`"site"` / `"section"` / `"section.page"` /
        /// `"section.group"` / `"section.group.page"`）。
        context: String,
        /// 欠落したキー名。
        key: String,
    },
    /// セクションに直下ページ・グループのいずれも 1 件も宣言されていない
    /// （イシュー #939: グループのみで直下ページが 0 件のセクションは
    /// 正当な構成であり、ここには含まれない）。
    EmptySection(String),
    /// グループにページが 1 件も宣言されていない（イシュー #939）。
    EmptyGroup(String),
    /// `[[section]]` に必須キー `index_path` が宣言されていない
    /// （イシュー #1010）。
    MissingSectionIndex {
        /// `[[section]]` ヘッダ行（1 始まり）。
        line: usize,
        /// セクションタイトル。
        section: String,
    },
    /// `index_path` が当該セクション配下のどの `page.path` とも一致しない
    /// （他セクションのページを指す場合・存在しない path を指す場合の
    /// 双方を含む、イシュー #1010）。
    SectionIndexNotFound {
        /// `index_path = "..."` の行（1 始まり）。
        line: usize,
        /// セクションタイトル。
        section: String,
        /// 一致しなかった `index_path` の値。
        index_path: String,
    },
}

impl fmt::Display for NavError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NavError::TooLarge => {
                write!(f, "nav.toml exceeds the {MAX_INPUT_BYTES} byte size limit")
            }
            NavError::Parse { line, message } => write!(f, "nav.toml:{line}: {message}"),
            NavError::DuplicatePath(path) => write!(f, "duplicate page.path `{path}`"),
            NavError::MissingSource(source) => {
                write!(f, "page.source `{source}` does not exist under repo_root")
            }
            NavError::UnsafeSource(source) => {
                write!(f, "page.source `{source}` is not a safe relative path")
            }
            NavError::UnsafePagePath(path) => write!(
                f,
                "page.path `{path}` must start and end with `/` with segments limited to alphanumerics, `-`, `_`"
            ),
            NavError::MissingKey { context, key } => {
                write!(f, "missing required key `{key}` in [{context}]")
            }
            NavError::EmptySection(title) => write!(f, "section `{title}` has no pages"),
            NavError::EmptyGroup(title) => write!(f, "group `{title}` has no pages"),
            NavError::MissingSectionIndex { line, section } => write!(
                f,
                "nav.toml:{line}: section `{section}` is missing required key `index_path`"
            ),
            NavError::SectionIndexNotFound {
                line,
                section,
                index_path,
            } => write!(
                f,
                "nav.toml:{line}: section `{section}` index_path `{index_path}` does not match any page.path in this section"
            ),
        }
    }
}

impl std::error::Error for NavError {}

/// パース中に組み立て途上のセクション。必須キーの充足は全行走査後に
/// まとめて検証する（欠落順序に依存しない一貫したエラーにするため）。
struct SectionBuilder {
    title: Option<String>,
    /// `[[section]]` ヘッダ行（1 始まり）。`NavError::MissingSectionIndex`
    /// の行番号として使う（イシュー #1010）。
    header_line: usize,
    index_path: Option<String>,
    /// `index_path = "..."` の行（1 始まり）。`NavError::SectionIndexNotFound`
    /// の行番号として使う（イシュー #1010）。
    index_path_line: Option<usize>,
    pages: Vec<PageBuilder>,
    groups: Vec<GroupBuilder>,
}

/// パース中に組み立て途上のグループ（イシュー #939）。
struct GroupBuilder {
    title: Option<String>,
    pages: Vec<PageBuilder>,
}

struct PageBuilder {
    title: Option<String>,
    source: Option<String>,
    path: Option<String>,
}

/// 現在どのテーブルの直下を走査しているかを表す。`[[section.page]]` は
/// 直前に開始された `[[section]]`（`sections` の末尾）に属する。
/// `[[section.group]]` も同様に `sections` 末尾へ属し、
/// `[[section.group.page]]` は `[[section.group]]` 開始時点の
/// `(sections.len() - 1, groups.len() - 1)` へ属する。
enum Ctx {
    None,
    Site,
    Section(usize),
    Page(usize, usize),
    /// `(section index, group index)`。
    Group(usize, usize),
    /// `(section index, group index, page index)`。group index は
    /// `[[section.group.page]]` 出現時点で `sections[sidx].groups.len()
    /// .checked_sub(1)` から都度導出する（`Ctx::Group` にキャッシュした
    /// index を使い回さない。新しい `[[section]]` が開いた直後に
    /// `[[section.group.page]]` が現れた場合、前セクション末尾の group へ
    /// 誤って吸着することを構造的に防ぐため）。
    GroupPage(usize, usize, usize),
}

fn parse_err(line: usize, message: impl Into<String>) -> NavError {
    NavError::Parse {
        line,
        message: message.into(),
    }
}

/// テーブルヘッダ・値の後続部分が「空、または `#` 始まりのコメント」で
/// あることを検証する。それ以外の残存文字列はサブセット外構文として拒否する。
fn check_trailing(rest: &str, line: usize) -> Result<(), NavError> {
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
/// 文字列 1 個を読み取る。エスケープは `\"` `\\` `\n` `\t` のみ対応。
/// 戻り値は `(パース済み文字列, 閉じクォート以降の残り文字列)`。
fn parse_quoted_string(value_part: &str, line: usize) -> Result<(String, &str), NavError> {
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
) -> Result<(), NavError> {
    if slot.is_some() {
        return Err(parse_err(line, format!("duplicate key `{name}`")));
    }
    *slot = Some(value);
    Ok(())
}

/// `id` が出力パス片として安全（英数字・`-`・`_` のみ、非空）かを検証する。
/// `fandhe_frontend_server::ssg` の `is_safe_path_segment` と同一の
/// ホワイトリストを、`generate_pages()` へ渡す前段で早期適用する
/// （多層防御。二重検証の意図はここに明記する）。
fn is_safe_path_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn validate_base_path(base_path: &str) -> Result<(), NavError> {
    if base_path.is_empty() {
        return Ok(());
    }
    if base_path.starts_with('/') && !base_path.ends_with('/') {
        Ok(())
    } else {
        Err(parse_err(
            0,
            format!(
                "site.base_path `{base_path}` must be \"\" or start with `/` and not end with `/`"
            ),
        ))
    }
}

fn validate_page_path(path: &str) -> Result<(), NavError> {
    if !path.starts_with('/') || !path.ends_with('/') {
        return Err(NavError::UnsafePagePath(path.to_string()));
    }
    if path.len() == 1 {
        // "/"（サイトトップ）はセグメントなしで許可する。
        //
        // 単一文字 "/" は開始・終了の '/' が同一バイトを指すため、下の
        // `path[1..path.len() - 1]` スライス（1..0）は範囲が逆転してパニック
        // する（イシュー #473 実装時に検出）。長さ 1 の場合はスライス計算に
        // 入る前に早期リターンする。
        return Ok(());
    }
    let inner = &path[1..path.len() - 1];
    if inner.is_empty() {
        // "//" のような縮退ケース。セグメントなしとして許可する
        // （現状 nav.toml では使用しないが、ホワイトリスト方式の
        // 対称性のため拒否しない）。
        return Ok(());
    }
    if inner.split('/').all(is_safe_path_segment) {
        Ok(())
    } else {
        Err(NavError::UnsafePagePath(path.to_string()))
    }
}

/// `source` が相対パスの安全条件（絶対パス禁止・`..` セグメント禁止・
/// `\` 禁止）を満たすかを構文レベルで検証する（パストラバーサル対策の
/// 早期検出。実ファイル存在確認は [`validate_sources`] が別途行う）。
fn validate_source_shape(source: &str) -> Result<(), NavError> {
    let looks_safe = !source.is_empty()
        && !source.starts_with('/')
        && !source.contains('\\')
        && source.split('/').all(|segment| segment != "..");
    if looks_safe {
        Ok(())
    } else {
        Err(NavError::UnsafeSource(source.to_string()))
    }
}

/// `nav.toml` の内容（文字列）をパースし、スキーマ・`page.path` /
/// `site.base_path` の形式・`page.path` の重複検証までを行う純関数。
/// ファイルシステムには一切アクセスしない（`page.source` の実在確認は
/// [`validate_sources`] を別途呼ぶこと）。
///
/// # Errors
///
/// 対応外の TOML 構文・必須キー欠落・空セクション・`page.path` 重複・
/// `page.path` / `site.base_path` の形式違反・`page.source` の構文上の
/// 危険性（絶対パス・`..`・`\`）のいずれかがあれば [`NavError`] を返す。
pub fn parse_nav(input: &str) -> Result<Nav, NavError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(NavError::TooLarge);
    }

    let mut ctx = Ctx::None;
    let mut site_title: Option<String> = None;
    let mut site_base_path: Option<String> = None;
    let mut sections: Vec<SectionBuilder> = Vec::new();

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
                "section" => {
                    sections.push(SectionBuilder {
                        title: None,
                        header_line: line,
                        index_path: None,
                        index_path_line: None,
                        pages: Vec::new(),
                        groups: Vec::new(),
                    });
                    ctx = Ctx::Section(sections.len() - 1);
                }
                "section.page" => {
                    let sidx = sections.len().checked_sub(1).ok_or_else(|| {
                        parse_err(line, "[[section.page]] appeared before any [[section]]")
                    })?;
                    sections[sidx].pages.push(PageBuilder {
                        title: None,
                        source: None,
                        path: None,
                    });
                    let pidx = sections[sidx].pages.len() - 1;
                    ctx = Ctx::Page(sidx, pidx);
                }
                "section.group" => {
                    let sidx = sections.len().checked_sub(1).ok_or_else(|| {
                        parse_err(line, "[[section.group]] appeared before any [[section]]")
                    })?;
                    sections[sidx].groups.push(GroupBuilder {
                        title: None,
                        pages: Vec::new(),
                    });
                    let gidx = sections[sidx].groups.len() - 1;
                    ctx = Ctx::Group(sidx, gidx);
                }
                "section.group.page" => {
                    let sidx = sections.len().checked_sub(1).ok_or_else(|| {
                        parse_err(
                            line,
                            "[[section.group.page]] appeared before any [[section]]",
                        )
                    })?;
                    // gidx をその場で導出する（`Ctx::Group` の index を
                    // 使い回さない理由は `Ctx::GroupPage` の doc 参照）。
                    let gidx = sections[sidx].groups.len().checked_sub(1).ok_or_else(|| {
                        parse_err(
                            line,
                            "[[section.group.page]] appeared before any [[section.group]]",
                        )
                    })?;
                    sections[sidx].groups[gidx].pages.push(PageBuilder {
                        title: None,
                        source: None,
                        path: None,
                    });
                    let pidx = sections[sidx].groups[gidx].pages.len() - 1;
                    ctx = Ctx::GroupPage(sidx, gidx, pidx);
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
            check_trailing(&rest[end + 1..], line)?;
            match header {
                "site" => ctx = Ctx::Site,
                other => return Err(parse_err(line, format!("unknown table `[{other}]`"))),
            }
            continue;
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
            Ctx::Site => match key {
                "title" => set_once(&mut site_title, value, line, "site.title")?,
                "base_path" => set_once(&mut site_base_path, value, line, "site.base_path")?,
                other => return Err(parse_err(line, format!("unknown key `{other}` in [site]"))),
            },
            Ctx::Section(sidx) => match key {
                "title" => set_once(&mut sections[sidx].title, value, line, "section.title")?,
                "index_path" => {
                    set_once(
                        &mut sections[sidx].index_path,
                        value,
                        line,
                        "section.index_path",
                    )?;
                    sections[sidx].index_path_line = Some(line);
                }
                other => {
                    return Err(parse_err(
                        line,
                        format!("unknown key `{other}` in [[section]]"),
                    ))
                }
            },
            Ctx::Page(sidx, pidx) => {
                let page = &mut sections[sidx].pages[pidx];
                match key {
                    "title" => set_once(&mut page.title, value, line, "page.title")?,
                    "source" => set_once(&mut page.source, value, line, "page.source")?,
                    "path" => set_once(&mut page.path, value, line, "page.path")?,
                    other => {
                        return Err(parse_err(
                            line,
                            format!("unknown key `{other}` in [[section.page]]"),
                        ))
                    }
                }
            }
            Ctx::Group(sidx, gidx) => {
                let group = &mut sections[sidx].groups[gidx];
                match key {
                    "title" => set_once(&mut group.title, value, line, "group.title")?,
                    other => {
                        return Err(parse_err(
                            line,
                            format!("unknown key `{other}` in [[section.group]]"),
                        ))
                    }
                }
            }
            Ctx::GroupPage(sidx, gidx, pidx) => {
                let page = &mut sections[sidx].groups[gidx].pages[pidx];
                match key {
                    "title" => set_once(&mut page.title, value, line, "group.page.title")?,
                    "source" => set_once(&mut page.source, value, line, "group.page.source")?,
                    "path" => set_once(&mut page.path, value, line, "group.page.path")?,
                    other => {
                        return Err(parse_err(
                            line,
                            format!("unknown key `{other}` in [[section.group.page]]"),
                        ))
                    }
                }
            }
        }
    }

    let site = Site {
        title: site_title.ok_or_else(|| NavError::MissingKey {
            context: "site".to_string(),
            key: "title".to_string(),
        })?,
        base_path: site_base_path.ok_or_else(|| NavError::MissingKey {
            context: "site".to_string(),
            key: "base_path".to_string(),
        })?,
    };
    validate_base_path(&site.base_path)?;

    if sections.is_empty() {
        return Err(parse_err(
            0,
            "nav.toml must declare at least one [[section]]",
        ));
    }

    let mut seen_paths: BTreeSet<String> = BTreeSet::new();
    let mut out_sections = Vec::with_capacity(sections.len());
    for section in sections {
        let title = section.title.ok_or_else(|| NavError::MissingKey {
            context: "section".to_string(),
            key: "title".to_string(),
        })?;
        // イシュー #939: 直下ページ 0 件でもグループが 1 件以上あれば正当な
        // 構成（`Components > カテゴリ` のように直下ページを持たない
        // セクションを許可する）。両方が空の場合のみ EmptySection。
        if section.pages.is_empty() && section.groups.is_empty() {
            return Err(NavError::EmptySection(title));
        }
        let mut out_pages = Vec::with_capacity(section.pages.len());
        for page in section.pages {
            let page = finalize_page(page, "section.page", &mut seen_paths)?;
            out_pages.push(page);
        }
        let mut out_groups = Vec::with_capacity(section.groups.len());
        for group in section.groups {
            let group_title = group.title.ok_or_else(|| NavError::MissingKey {
                context: "section.group".to_string(),
                key: "title".to_string(),
            })?;
            if group.pages.is_empty() {
                return Err(NavError::EmptyGroup(group_title));
            }
            let mut out_group_pages = Vec::with_capacity(group.pages.len());
            for page in group.pages {
                let page = finalize_page(page, "section.group.page", &mut seen_paths)?;
                out_group_pages.push(page);
            }
            out_groups.push(Group {
                title: group_title,
                pages: out_group_pages,
            });
        }

        // イシュー #1010: `index_path` 必須化の検証は「pages/groups 確定
        // 後・EmptySection より後」で行う。ページが 1 件も無いセクション
        // には index の指しようがなく、EmptySection の方が情報量の多い
        // 診断であるため（既存回帰テスト `rejects_empty_section` が
        // fixture 無変更のまま通ることの保証でもある）。
        let index_path = section
            .index_path
            .ok_or_else(|| NavError::MissingSectionIndex {
                line: section.header_line,
                section: title.clone(),
            })?;
        // 検証は「finalize_page で validate_page_path を通過した
        // page.path 集合との完全一致」の 1 ルールのみとする。独立した
        // 形式検証を追加しない（`index_path ⊆ 生成ページの path 集合`
        // という構造的不変条件をドリフトさせないため。§3.6 参照）。
        let index_path_matches_a_page = out_pages
            .iter()
            .chain(out_groups.iter().flat_map(|g| g.pages.iter()))
            .any(|p| p.path == index_path);
        if !index_path_matches_a_page {
            return Err(NavError::SectionIndexNotFound {
                line: section.index_path_line.unwrap_or(section.header_line),
                section: title,
                index_path,
            });
        }

        out_sections.push(Section {
            title,
            index_path,
            pages: out_pages,
            groups: out_groups,
        });
    }

    Ok(Nav {
        site,
        sections: out_sections,
    })
}

/// [`PageBuilder`] を検証済み [`Page`] へ確定する。必須キー欠落・
/// `page.path` / `page.source` の形式検証・`seen_paths` を横断した
/// `path` 重複検査を一箇所へ集約し、section 直下・グループ配下の双方が
/// 同一の検証（パストラバーサル対策含む）を必ず通ることを構造的に保証する
/// （イシュー #939: グループ配下だけ検証を迂回する分岐を作らない）。
fn finalize_page(
    page: PageBuilder,
    context: &str,
    seen_paths: &mut BTreeSet<String>,
) -> Result<Page, NavError> {
    let title = page.title.ok_or_else(|| NavError::MissingKey {
        context: context.to_string(),
        key: "title".to_string(),
    })?;
    let source = page.source.ok_or_else(|| NavError::MissingKey {
        context: context.to_string(),
        key: "source".to_string(),
    })?;
    let path = page.path.ok_or_else(|| NavError::MissingKey {
        context: context.to_string(),
        key: "path".to_string(),
    })?;
    validate_page_path(&path)?;
    validate_source_shape(&source)?;
    if !seen_paths.insert(path.clone()) {
        return Err(NavError::DuplicatePath(path));
    }
    Ok(Page {
        title,
        source,
        path,
    })
}

/// 各 `page.source` が `repo_root` 配下の実ファイルとして存在することを
/// 検証する。[`parse_nav`] から FS アクセスを分離し、単体テストを
/// ファイルシステムに依存させないための独立関数（イシュー #468 実装計画）。
///
/// # Errors
///
/// いずれかの `page.source` が `repo_root` 配下のファイルとして存在しない
/// 場合、最初に見つかった不在ファイルについて `NavError::MissingSource` を返す。
pub fn validate_sources(nav: &Nav, repo_root: &Path) -> Result<(), NavError> {
    // `nav.all_pages()`（唯一の正規走査経路）を使い、グループ配下ページの
    // `source` 実在確認も直下ページと同様に行う（イシュー #939）。
    for page in nav.all_pages() {
        let full_path = repo_root.join(&page.source);
        if !full_path.is_file() {
            return Err(NavError::MissingSource(page.source.clone()));
        }
    }
    Ok(())
}

/// `nav.site.base_path` + `page.path` を単純連結した href を返す。
/// 両者とも [`parse_nav`] で形式検証済み（`base_path` は `/` 終わりでない、
/// `path` は `/` 始まり）のため、二重 `/` は発生しない。
fn href(nav: &Nav, path: &str) -> String {
    format!("{}{}", nav.site.base_path, path)
}

/// サイドバー [`Node`] を生成する。セクション・ページとも宣言順で列挙し、
/// `current_path` に一致するページの `<a>` にのみ `aria-current="page"`
/// （+ `data-current`）を付与する。`current_path` が `nav` 中のどの
/// `page.path` にも一致しない場合はハイライトなしで全ページを列挙する
/// （サイトトップ等、nav セクション外のページが正当に存在しうるため
/// エラーにはしない契約）。
///
/// headless `nav_list`（`fandhe-frontend-headless-ui`、イシュー #756）の
/// anatomy パーツ（`root`/`heading`/`list`/`item`/`link`）で組み立てる。
/// `nav_list` は `role` を一切付与しない素の `nav`/`h2`/`ul`/`li`/`a` 構造の
/// ため、`crate::site_theme::stylesheet()` が生成する CSS のタグ・class
/// セレクタ（`nav.sidebar h2`/`nav.sidebar ul` 等）は変更なしで適用され
/// 続ける（`docs/design/docs-site-styled-ui-adoption.md` §3.1 の意味論
/// 不整合解消の記録参照）。実出力は同時に `data-scope="nav-list"
/// data-part="heading|list|item|link"` を持ち、
/// `fandhe_frontend_pre_styled_ui::nav_list::stylesheet()` の
/// コンポーネント基底 CSS（list-style 除去・`aria-current="page"` の
/// accent 色等）にも適用される。`crate::site_theme` が両者を連結する
/// 順序・カスケード上の関係はイシュー #910・`site_theme` モジュール doc
/// 参照。
///
/// # カテゴリ階層描画（イシュー #940）
///
/// `section.groups`（`[[section.group]]`、イシュー #939）はセクション見出し
/// 直下ページ一覧の**後ろ**に、プレーン HTML の `<details>`/`<summary>` で
/// 折りたたみ可能なカテゴリとして描画する（JS を一切使わない。受け入れ
/// 条件「JS 無効環境でもナビゲーションが成立する」を `<details>` の
/// ネイティブ挙動のみで満たす）。DOM 構造:
///
/// ```text
/// nav.sidebar[aria-label="Documentation"]     … nav_list root（既存）
///   h2                                        … セクション見出し（既存）
///   ul                                        … 直下ページ（section.pages が非空のときのみ）
///     li > a[href]（現在ページのみ aria-current="page" + data-current）
///   details.docs-nav-group[open?]             … グループ 1 件 = details 1 件（宣言順）
///     summary.docs-nav-group-summary          … グループ見出し（プレーンテキスト）
///     ul.docs-nav-group-list                  … nav_list list を再利用
///       li > a[href]
/// ```
///
/// 確定した設計判断（再検討しない。詳細は #940 実装計画 §3.1 参照）:
///
/// - `<details>` は `<ul>` の子にできない（HTML 仕様）ため、`nav.sidebar`
///   の直接の子として直下ページ `ul` の後ろに置く。これは
///   [`Section::all_pages`] が定める「直下ページ → グループ（宣言順）→
///   グループ内ページ（宣言順）」の描画順契約そのもの。
/// - 直下ページが 0 件のセクションでは `ul` を出力しない（空 `ul` を
///   出さない。#939 で `EmptySection` の判定が `pages.is_empty() &&
///   groups.is_empty()` へ是正され「グループのみのセクション」が正当な
///   構成になったため必須）。
/// - グループ配下の `ul`/`li`/`a` は [`list`]/[`item`]/[`nav_link`]
///   （nav_list anatomy）を再利用する。基底 CSS（list-style 除去・
///   `aria-current` の accent 色）をそのまま継承させ、`aria-current`
///   付与ロジックを二重実装しないため。
/// - `<summary>` の中身はプレーンテキストのみ（`h3` を入れない）。
///   `<summary>` は既にディスクロージャウィジェットとしてラベルを読み
///   上げるため、見出し要素を追加するとスクリーンリーダー実装間で挙動が
///   割れる。これは [`header_nav`] が `docs-header-trigger` へ
///   `role`/`aria-expanded`/`aria-haspopup` を付けないとした判断と同じ
///   立場（同関数 rustdoc 参照）。
/// - `open` 属性は `group.pages` に `current_path` を含むグループにのみ
///   `("open", "")`（boolean 属性として正当）で付与する。どのグループにも
///   一致しない場合は全グループを閉じたまま（直下ページが現在ページの
///   ケース等）。`<details name=...>`（排他アコーディオン）は使わない
///   （ブラウザ対応が新しく、複数グループを同時に開く自由を奪うため）。
/// - `header_nav` は本イシューのスコープ外でフラット列挙のまま（同関数の
///   rustdoc・#939 に既存記載のとおり）。
///
/// タイトル・href はすべて headless 層 → [`fandhe_frontend_core::render`]
/// の既定エスケープ（REQ-1）を必ず経由する。`<details>`/`<summary>` も
/// [`fandhe_frontend_core::el`] のプレーン HTML 組み立てで、HTML 文字列の
/// 直接組み立て・`raw_html()` は使用しない。
pub fn sidebar(nav: &Nav, current_path: &str) -> Node {
    let mut section_nodes: Vec<Node> = Vec::new();
    for section in &nav.sections {
        section_nodes.push(heading(vec![], vec![text(section.title.clone())]));

        // 直下ページ（`section.pages`）が非空のときのみ `ul` を出力する。
        // フラット列挙用の `section.all_pages()` はここでは使わない（グループ
        // 見出しを挟んだ階層描画には直下ページとグループを個別に扱う必要が
        // あるため。順序契約自体は `all_pages()` と同一に保つ）。
        if !section.pages.is_empty() {
            let mut items: Vec<Node> = Vec::new();
            for page in &section.pages {
                let link_href = href(nav, &page.path);
                let is_current = page.path == current_path;
                let a = nav_link(
                    &link_href,
                    is_current,
                    vec![],
                    vec![text(page.title.clone())],
                );
                items.push(item(vec![], vec![a]));
            }
            section_nodes.push(list(vec![], items));
        }

        for group in &section.groups {
            section_nodes.push(group_node(nav, group, current_path));
        }
    }
    nav_list_root("Documentation", vec![("class", "sidebar")], section_nodes)
}

/// [`sidebar`] からグループ 1 件（`[[section.group]]`）を `<details>` へ
/// 変換する private ヘルパ。`open` は `group.pages` が `current_path` を
/// 含む場合にのみ付与する（[`sidebar`] rustdoc「カテゴリ階層描画」参照）。
fn group_node(nav: &Nav, group: &Group, current_path: &str) -> Node {
    let is_open = group.pages.iter().any(|p| p.path == current_path);
    let mut items: Vec<Node> = Vec::new();
    for page in &group.pages {
        let link_href = href(nav, &page.path);
        let is_current = page.path == current_path;
        let a = nav_link(
            &link_href,
            is_current,
            vec![],
            vec![text(page.title.clone())],
        );
        items.push(item(vec![], vec![a]));
    }
    let summary = el(
        "summary",
        vec![("class", "docs-nav-group-summary")],
        vec![text(group.title.clone())],
    );
    let group_list = list(vec![("class", "docs-nav-group-list")], items);
    let mut attrs = vec![("class", "docs-nav-group")];
    if is_open {
        // boolean 属性 `open`。`el` は空文字列値として `open=""` を出力する
        // （HTML5 boolean attribute として正当）。
        attrs.push(("open", ""));
    }
    el("details", attrs, vec![summary, group_list])
}

/// ヘッダーのセクション別ドロップダウンメニュー [`Node`] を生成する
/// （イシュー #908。トリガーの遷移リンク化・ドロップダウン抑制は
/// イシュー #1012）。`nav.toml` の `[[section]]` ごとにトリガー
/// `<a href>`（セクショントップページ `section.index_path` への遷移
/// リンク）+ ドロップダウン `<ul>`（直下ページ列）をグループ化した
/// `<nav class="docs-header-nav">` を返す。
///
/// # イシュータイトルとの差分（`pre-styled-ui menu` を使わない理由）
///
/// イシュータイトルは「pre-styled-ui menu によるドロップダウン」だが、
/// `docs/design/docs-site-three-column-redesign.md` §3.5 の 3 案比較の結果、
/// 本関数は素の `nav`/`ul`/`li`/`a` + CSS のみの開閉（`:hover` /
/// `:focus-within`）を採用する（案 (b)）。理由は 2 点:
///
/// 1. **意味論不整合**: WAI-ARIA `menu` ロールは操作コマンドリスト向けで
///    あり、文書リンク集ナビ（本関数の用途）へ転用するとスクリーン
///    リーダー利用者へ「操作可能なメニュー」と誤って伝わる
///    （`crate::nav` の [`sidebar`] が headless `nav_list`
///    （`fandhe-frontend-headless-ui`）を採用した理由と同型。
///    `docs-site-styled-ui-adoption.md` §3.1 参照）。
/// 2. **無 JS 制約**: pre-styled-ui `menu` の `data-state` 開閉は
///    wasm-full 配線（hydration）前提であり、JS を持たない docs-site
///    では動作しない。
///
/// # `role`/`aria-expanded`/`aria-haspopup` を付与しない理由
///
/// トリガーはドロップダウンの開閉状態を JS で更新する経路を持たない
/// （CSS の `:hover`/`:focus-within` のみで開閉する）。ARIA の動的状態
/// 属性を静的な固定値のまま出力すると支援技術に虚偽の状態を伝えること
/// になるため、`role`/`aria-expanded`/`aria-haspopup` のいずれも付与
/// しない（[`fandhe_frontend_headless_ui::nav_list`] が「素の要素の暗黙
/// ARIA ロールのみを使う」とした判断をそのまま踏襲する）。トリガーが
/// `<button>` から `<a href>` に変わった後も、`<a>` はリンクとしての
/// 暗黙ロールを持つのみでありこの判断は変わらない。
///
/// # DOM 構造
///
/// ```text
/// nav.docs-header-nav[aria-label="Site sections"]  … headless nav_list root
///   ul.docs-header-menu                            … nav_list list
///     li.docs-header-group（セクションごと）        … nav_list item
///       a.docs-header-trigger[href=base_path+index_path]（el 直接。
///         現在セクションのみ aria-current="true" + data-current）
///       ul.docs-header-dropdown                    … nav_list list（再利用）
///         li > a[href]（直下ページのみ。現在ページのみ
///           aria-current="page" + data-current）
/// ```
///
/// セクションが単一ページのみでも一律ドロップダウン構造にする
/// （決定性・実装単純化を優先。§3.5 が実装時裁量とした点の確定）。
///
/// `aria-current` は 2 つの意味軸を衝突させない: `"page"` はドロップ
/// ダウン内の現在ページ 1 件との完全一致、`"true"` はトリガー側の現在
/// セクション所属を表す（同一マークアップ内で `page`/`true` が同時に
/// 出ても意味が異なるため矛盾しない）。
///
/// タイトル・href はすべて headless 層 → [`fandhe_frontend_core::render`]
/// の既定エスケープ（REQ-1）を必ず経由する。HTML 文字列の直接組み立て・
/// `raw_html()` は使用しない。トリガー href は `section.index_path`
/// （[`parse_nav`] のパース時点で当該セクション内の実在 `page.path` との
/// 完全一致が保証される。[`Section::index_path`] の doc コメント参照）を
/// 経由するため、`validate_page_path` を通過済みの検証済み文字列のみが
/// href に現れる（新たなパストラバーサル面を作らない）。
pub fn header_nav(nav: &Nav, current_path: &str) -> Node {
    let mut groups: Vec<Node> = Vec::new();
    for section in &nav.sections {
        let trigger_href = href(nav, &section.index_path);
        // 現在セクション判定はローカルにループ内で行う（`Nav::section_for_path`
        // はポインタ同一性比較が必要になり脆いため、#1013 のサイドバー
        // スコープ判定側の用途に譲り、ここでは使わない）。
        let is_current_section = section.all_pages().any(|p| p.path == current_path);

        let mut dropdown_items: Vec<Node> = Vec::new();
        // ドロップダウン項目はセクション直下ページのみを列挙する（Rule A、
        // イシュー #1012）。`section.all_pages()`（グループ配下まで平坦化
        // する走査、イシュー #939）を使うと、Components セクションのように
        // グループ配下ページが 100 件超あるセクションでドロップダウンが
        // ビューポート外へはみ出し実質操作不能になる（実測: 108 項目 /
        // 16KB。`.docs-header-dropdown` に `max-height`/`overflow` を持た
        // ない）。トリガー自体が本イシューでセクショントップページへの
        // 遷移リンクになったため、グループ配下ページの一覧はサイドバー
        // （#1013 でセクションスコープに限定される）に委ねる。
        for page in &section.pages {
            let link_href = href(nav, &page.path);
            let is_current = page.path == current_path;
            let a = nav_link(
                &link_href,
                is_current,
                vec![],
                vec![text(page.title.clone())],
            );
            dropdown_items.push(item(vec![], vec![a]));
        }
        // 直下ページの中にセクション索引ページ（`index_path` と同一 path）
        // が無く、かつグループが存在する場合のみ「すべて見る」項目を追加
        // する。索引ページが直下ページに既に含まれる場合は同一リンクの
        // 重複を避ける（Rule A の重複回避条件）。
        let index_already_listed = section.pages.iter().any(|p| p.path == section.index_path);
        if !section.groups.is_empty() && !index_already_listed {
            let all_link = nav_link(&trigger_href, false, vec![], vec![text("すべて見る")]);
            dropdown_items.push(item(vec![], vec![all_link]));
        }

        // トリガーを `<a href>` 化し、セクショントップページ
        // （`section.index_path`）への遷移リンクにする（イシュー #1012）。
        // `<a>` はフォーム送信を行わないため `type="button"` は不要
        // （旧 `<button type="button">` 時代の A05 対策の削除）。
        let mut trigger_attrs: Vec<(&str, &str)> =
            vec![("href", &trigger_href), ("class", "docs-header-trigger")];
        if is_current_section {
            // ページ完全一致用の `aria-current="page"`（ドロップダウン内
            // リンクが使う）とは軸が異なるため `"true"` を使い衝突させない
            // （関数 rustdoc「`aria-current` は 2 つの意味軸」参照）。
            trigger_attrs.push(("aria-current", "true"));
            trigger_attrs.push(("data-current", ""));
        }
        let trigger = el("a", trigger_attrs, vec![text(section.title.clone())]);
        let dropdown = list(vec![("class", "docs-header-dropdown")], dropdown_items);
        groups.push(item(
            vec![("class", "docs-header-group")],
            vec![trigger, dropdown],
        ));
    }
    let menu = list(vec![("class", "docs-header-menu")], groups);
    // サイドバー（`sidebar()`）の `aria-label="Documentation"` と区別できる
    // ラベルにする（複数 nav ランドマークが存在する文書でスクリーン
    // リーダー利用者が識別できるようにするため）。
    nav_list_root(
        "Site sections",
        vec![("class", "docs-header-nav")],
        vec![menu],
    )
}

/// 全セクションを文書順（宣言順）に平坦化したページ列における、
/// `current_path` の前後ページを返す。`current_path` が見つからない場合は
/// `(None, None)`。先頭ページは `(None, Some(next))`、末尾ページは
/// `(Some(prev), None)` になる。
pub fn prev_next<'a>(nav: &'a Nav, current_path: &str) -> (Option<&'a Page>, Option<&'a Page>) {
    // `nav.all_pages()`（唯一の正規走査経路）で全ページを貫通する
    // （イシュー #939: グループ配下ページも前後ナビの対象になる）。
    let flat: Vec<&Page> = nav.all_pages().collect();
    let Some(idx) = flat.iter().position(|p| p.path == current_path) else {
        return (None, None);
    };
    let prev = if idx > 0 { Some(flat[idx - 1]) } else { None };
    let next = flat.get(idx + 1).copied();
    (prev, next)
}

/// 前後ページリンクの [`Node`]（`<nav class="prev-next">` 配下に
/// 存在する側のみの LinkOverlay カード。`<div class="prev">`/`<div
/// class="next">`（headless `link_overlay::root`）が外枠、内側の
/// `[data-part="overlay"]`（headless `link_overlay::overlay`）が実際の
/// アンカーであり、カード全面がクリック可能な状態を保つ）を生成する。
///
/// `fandhe-frontend-headless-ui` の `link_overlay`（イシュー #756）へ移行
/// した理由は `docs/design/docs-site-styled-ui-adoption.md` §3.2（「pre-styled-ui
/// の `card` はアンカー全面クリック化に非対応」という見送り判断）を解消
/// するため。本モジュールの用途では `overlay` がカードの唯一の子であり、
/// 通常のフローで全面を占めるため、`link_overlay` の一般的な
/// `position: absolute` 拡張パターン（`crates/pre-styled-ui/src/link_overlay.rs`
/// 参照）は使わず、`crate::site_theme::stylesheet()` 側で `overlay` 自体に
/// 従来のカード CSS（枠線・padding・角丸）をそのまま当てる。この判断は
/// イシュー #910 でも再確認済み（`link_overlay::stylesheet()` は `overlay`
/// に `position: absolute; inset: 0;` を登録するため、唯一の子要素である
/// 本用途に適用するとカードの高さが 0 に潰れる。`fandhe_frontend_pre_styled_ui::nav_list::stylesheet()`
/// は取り込むが `link_overlay::stylesheet()` は取り込まない非対称な採用に
/// なる理由）。
pub fn prev_next_nav(nav: &Nav, current_path: &str) -> Node {
    let (prev, next) = prev_next(nav, current_path);
    let mut children: Vec<Node> = Vec::new();
    if let Some(page) = prev {
        let link_href = href(nav, &page.path);
        children.push(link_overlay_root(
            vec![("class", "prev")],
            vec![link_overlay_overlay(
                &link_href,
                vec![],
                vec![text(page.title.clone())],
            )],
        ));
    }
    if let Some(page) = next {
        let link_href = href(nav, &page.path);
        children.push(link_overlay_root(
            vec![("class", "next")],
            vec![link_overlay_overlay(
                &link_href,
                vec![],
                vec![text(page.title.clone())],
            )],
        ));
    }
    el("nav", vec![("class", "prev-next")], children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// テスト専用の一時ディレクトリ。`Drop` でベストエフォート削除する。
    /// 外部クレート（`tempfile` 等）を追加せず `crate::test_scratch::scratch_root()` +
    /// プロセス固有サフィックスで代用する（REQ-3: 外部依存ゼロを維持する。
    /// `crates/server/tests/support/temp_dir.rs` と同方針）。
    struct TempDir(std::path::PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            let unique = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let path = crate::test_scratch::scratch_root().join(format!(
                "fandhe-frontend-docs-site-nav-test-{tag}-{}-{unique}",
                std::process::id()
            ));
            std::fs::create_dir_all(&path).expect("create temp dir for nav.rs test");
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const SAMPLE: &str = r#"
[site]
title = "fandhe-frontend docs"
base_path = "/fandhe-frontend"

[[section]]
title = "Guide"
index_path = "/guide/intro/"

[[section.page]]
title = "Introduction"
source = "docs/guide/intro.md"
path = "/guide/intro/"

[[section.page]]
title = "Getting Started"
source = "docs/guide/getting-started.md"
path = "/guide/getting-started/"

[[section]]
title = "Reference"
index_path = "/reference/api/"

[[section.page]]
title = "API"
source = "docs/reference/api.md"
path = "/reference/api/"
"#;

    // ---- 正常系（受け入れ条件 1） ----

    #[test]
    fn parses_site_sections_and_pages_in_declaration_order() {
        let nav = parse_nav(SAMPLE).expect("valid nav.toml should parse");
        assert_eq!(nav.site.title, "fandhe-frontend docs");
        assert_eq!(nav.site.base_path, "/fandhe-frontend");
        assert_eq!(nav.sections.len(), 2);
        assert_eq!(nav.sections[0].title, "Guide");
        assert_eq!(nav.sections[0].pages.len(), 2);
        assert_eq!(nav.sections[0].pages[0].title, "Introduction");
        assert_eq!(nav.sections[0].pages[0].source, "docs/guide/intro.md");
        assert_eq!(nav.sections[0].pages[0].path, "/guide/intro/");
        assert_eq!(nav.sections[0].pages[1].title, "Getting Started");
        assert_eq!(nav.sections[1].title, "Reference");
        assert_eq!(nav.sections[1].pages.len(), 1);
        assert_eq!(nav.sections[1].pages[0].path, "/reference/api/");
    }

    #[test]
    fn supports_full_line_and_trailing_comments() {
        let input = r#"
# full line comment
[site]
title = "Docs" # trailing comment
base_path = ""

[[section]] # comment after header
title = "Guide"
index_path = "/intro/"

[[section.page]]
title = "Intro"
source = "intro.md"
path = "/intro/"
"#;
        let nav = parse_nav(input).expect("comments should be tolerated");
        assert_eq!(nav.site.title, "Docs");
        assert_eq!(nav.site.base_path, "");
    }

    #[test]
    fn supports_basic_string_escapes() {
        let input = r#"
[site]
title = "Line1\nLine2 \"quoted\" \\backslash\\"
base_path = ""

[[section]]
title = "S"
index_path = "/p/"

[[section.page]]
title = "P"
source = "p.md"
path = "/p/"
"#;
        let nav = parse_nav(input).expect("escapes should be supported");
        assert_eq!(nav.site.title, "Line1\nLine2 \"quoted\" \\backslash\\");
    }

    // ---- 異常系（受け入れ条件 3） ----

    #[test]
    fn rejects_duplicate_path_across_sections() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/dup/"

[[section.page]]
title = "P1"
source = "p1.md"
path = "/dup/"

[[section]]
title = "B"
index_path = "/dup/"

[[section.page]]
title = "P2"
source = "p2.md"
path = "/dup/"
"#;
        match parse_nav(input) {
            Err(NavError::DuplicatePath(path)) => assert_eq!(path, "/dup/"),
            other => panic!("expected DuplicatePath, got {other:?}"),
        }
    }

    #[test]
    fn validate_sources_reports_missing_source_file() {
        let temp = TempDir::new("missing-source");
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/p1/"

[[section.page]]
title = "P1"
source = "does-not-exist.md"
path = "/p1/"
"#;
        let nav = parse_nav(input).expect("structurally valid nav.toml should parse");
        match validate_sources(&nav, &temp.0) {
            Err(NavError::MissingSource(source)) => assert_eq!(source, "does-not-exist.md"),
            other => panic!("expected MissingSource, got {other:?}"),
        }
    }

    #[test]
    fn validate_sources_accepts_existing_files() {
        let temp = TempDir::new("existing-source");
        std::fs::write(temp.0.join("p1.md"), b"# hello").expect("write fixture source file");
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/p1/"

[[section.page]]
title = "P1"
source = "p1.md"
path = "/p1/"
"#;
        let nav = parse_nav(input).expect("valid nav.toml should parse");
        assert!(validate_sources(&nav, &temp.0).is_ok());
    }

    #[test]
    fn rejects_parent_traversal_in_source() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/p1/"

[[section.page]]
title = "P1"
source = "../secret.md"
path = "/p1/"
"#;
        match parse_nav(input) {
            Err(NavError::UnsafeSource(source)) => assert_eq!(source, "../secret.md"),
            other => panic!("expected UnsafeSource, got {other:?}"),
        }
    }

    #[test]
    fn rejects_absolute_path_source() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/p1/"

[[section.page]]
title = "P1"
source = "/etc/passwd"
path = "/p1/"
"#;
        assert!(matches!(parse_nav(input), Err(NavError::UnsafeSource(_))));
    }

    /// イシュー #473 実装時に検出した回帰テスト。`path = "/"`
    /// （サイトトップ）は `validate_page_path` 内のスライス計算
    /// （`path[1..path.len() - 1]`）が `1..0` の逆転範囲になりパニックして
    /// いた。長さ 1 の早期リターンで解消したことを確認する。
    #[test]
    fn accepts_site_root_page_path() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/"

[[section.page]]
title = "Top"
source = "index.md"
path = "/"
"#;
        let nav = parse_nav(input).expect("path = \"/\" should be accepted as the site root");
        assert_eq!(nav.sections[0].pages[0].path, "/");
    }

    #[test]
    fn rejects_page_path_without_leading_slash() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "p1/"

[[section.page]]
title = "P1"
source = "p1.md"
path = "p1/"
"#;
        assert!(matches!(parse_nav(input), Err(NavError::UnsafePagePath(_))));
    }

    #[test]
    fn rejects_page_path_without_trailing_slash() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/p1"

[[section.page]]
title = "P1"
source = "p1.md"
path = "/p1"
"#;
        assert!(matches!(parse_nav(input), Err(NavError::UnsafePagePath(_))));
    }

    #[test]
    fn rejects_page_path_with_unsafe_segment_characters() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/../p1/"

[[section.page]]
title = "P1"
source = "p1.md"
path = "/../p1/"
"#;
        assert!(matches!(parse_nav(input), Err(NavError::UnsafePagePath(_))));
    }

    #[test]
    fn rejects_missing_required_site_key() {
        let input = r#"
[site]
title = "Docs"

[[section]]
title = "A"
index_path = "/p1/"

[[section.page]]
title = "P1"
source = "p1.md"
path = "/p1/"
"#;
        match parse_nav(input) {
            Err(NavError::MissingKey { context, key }) => {
                assert_eq!(context, "site");
                assert_eq!(key, "base_path");
            }
            other => panic!("expected MissingKey, got {other:?}"),
        }
    }

    // ---- グループ 3 階層スキーマ（イシュー #939、後方互換・回帰） ----

    /// グループを一切含まない既存 `nav.toml`（`SAMPLE`）が従来どおり通り、
    /// `groups` が空であることを固定する（後方互換の回帰テスト）。
    #[test]
    fn sections_without_groups_have_empty_groups_vec() {
        let nav = parse_nav(SAMPLE).expect("valid nav.toml should parse");
        for section in &nav.sections {
            assert!(section.groups.is_empty());
        }
    }

    /// イシュー #939 での `EmptySection` 条件是正: 直下ページ 0 件・
    /// グループのみのセクションはエラーにならない。
    #[test]
    fn section_with_only_groups_and_no_direct_pages_is_not_empty() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/button/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/components/button/"
"#;
        let nav = parse_nav(input).expect("group-only section should not be EmptySection");
        assert!(nav.sections[0].pages.is_empty());
        assert_eq!(nav.sections[0].groups.len(), 1);
        assert_eq!(nav.sections[0].groups[0].pages[0].title, "Button");
    }

    #[test]
    fn rejects_empty_section() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Empty"
"#;
        match parse_nav(input) {
            Err(NavError::EmptySection(title)) => assert_eq!(title, "Empty"),
            other => panic!("expected EmptySection, got {other:?}"),
        }
    }

    #[test]
    fn rejects_section_page_before_any_section() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section.page]]
title = "Orphan"
source = "orphan.md"
path = "/orphan/"
"#;
        assert!(matches!(parse_nav(input), Err(NavError::Parse { .. })));
    }

    #[test]
    fn rejects_unsupported_value_types() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/p1/"

[[section.page]]
title = "P1"
source = "p1.md"
path = "/p1/"
weight = 1
"#;
        assert!(matches!(parse_nav(input), Err(NavError::Parse { .. })));
    }

    #[test]
    fn rejects_unterminated_string() {
        let input = "[site]\ntitle = \"unterminated\nbase_path = \"\"\n";
        assert!(matches!(parse_nav(input), Err(NavError::Parse { .. })));
    }

    #[test]
    fn rejects_input_larger_than_size_limit() {
        let mut input = String::from("[site]\ntitle = \"");
        input.push_str(&"a".repeat(MAX_INPUT_BYTES + 1));
        input.push_str("\"\nbase_path = \"\"\n");
        assert!(matches!(parse_nav(&input), Err(NavError::TooLarge)));
    }

    #[test]
    fn rejects_invalid_base_path() {
        let input = r#"
[site]
title = "Docs"
base_path = "no-leading-slash"

[[section]]
title = "A"
index_path = "/p1/"

[[section.page]]
title = "P1"
source = "p1.md"
path = "/p1/"
"#;
        assert!(matches!(parse_nav(input), Err(NavError::Parse { .. })));
    }

    // ---- サイドバー（受け入れ条件 2） ----

    #[test]
    fn sidebar_lists_all_pages_in_document_order_with_current_highlighted() {
        let nav = parse_nav(SAMPLE).unwrap();
        let html = render(&sidebar(&nav, "/guide/getting-started/"));
        // 文書順: Introduction, Getting Started, API
        let intro_idx = html.find("Introduction").unwrap();
        let getting_started_idx = html.find("Getting Started").unwrap();
        let api_idx = html.find("API").unwrap();
        assert!(intro_idx < getting_started_idx);
        assert!(getting_started_idx < api_idx);

        assert!(html.contains(r#"href="/fandhe-frontend/guide/getting-started/""#));
        // 現在ページのみ aria-current="page"（+ data-current）を持つ
        // （イシュー #756 で headless nav_list へ移行、`class="current"` は
        // 廃止し属性のみに一本化した）。
        assert_eq!(html.matches(r#"aria-current="page""#).count(), 1);
        assert!(html.contains("data-current"));
        assert!(!html.contains(r#"class="current""#));
    }

    #[test]
    fn sidebar_has_no_highlight_when_current_path_absent() {
        let nav = parse_nav(SAMPLE).unwrap();
        let html = render(&sidebar(&nav, "/not-in-nav/"));
        assert!(!html.contains("aria-current"));
        assert!(!html.contains(r#"class="current""#));
    }

    #[test]
    fn sidebar_escapes_title_and_attribute_content() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "<script>alert(1)</script>"
index_path = "/p1/"

[[section.page]]
title = "Quote\"Title"
source = "p1.md"
path = "/p1/"
"#;
        let nav = parse_nav(input).unwrap();
        let html = render(&sidebar(&nav, "/p1/"));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("Quote&quot;Title"));
    }

    // ---- 前後ナビ（受け入れ条件 2） ----

    #[test]
    fn prev_next_at_first_page_has_no_prev() {
        let nav = parse_nav(SAMPLE).unwrap();
        let (prev, next) = prev_next(&nav, "/guide/intro/");
        assert!(prev.is_none());
        assert_eq!(next.unwrap().path, "/guide/getting-started/");
    }

    #[test]
    fn prev_next_at_last_page_has_no_next() {
        let nav = parse_nav(SAMPLE).unwrap();
        let (prev, next) = prev_next(&nav, "/reference/api/");
        assert_eq!(prev.unwrap().path, "/guide/getting-started/");
        assert!(next.is_none());
    }

    #[test]
    fn prev_next_crosses_section_boundary() {
        let nav = parse_nav(SAMPLE).unwrap();
        let (prev, next) = prev_next(&nav, "/guide/getting-started/");
        assert_eq!(prev.unwrap().path, "/guide/intro/");
        assert_eq!(next.unwrap().path, "/reference/api/");
    }

    #[test]
    fn prev_next_absent_current_path_returns_none_none() {
        let nav = parse_nav(SAMPLE).unwrap();
        let (prev, next) = prev_next(&nav, "/not-in-nav/");
        assert!(prev.is_none());
        assert!(next.is_none());
    }

    #[test]
    fn prev_next_nav_renders_only_present_sides() {
        let nav = parse_nav(SAMPLE).unwrap();
        let html_first = render(&prev_next_nav(&nav, "/guide/intro/"));
        assert!(!html_first.contains(r#"class="prev""#));
        assert!(html_first.contains(r#"class="next""#));

        let html_last = render(&prev_next_nav(&nav, "/reference/api/"));
        assert!(html_last.contains(r#"class="prev""#));
        assert!(!html_last.contains(r#"class="next""#));
    }

    // ---- ヘッダードロップダウンメニュー（イシュー #908） ----

    #[test]
    fn header_nav_groups_sections_in_declaration_order_with_correct_hrefs() {
        let nav = parse_nav(SAMPLE).unwrap();
        let html = render(&header_nav(&nav, "/guide/getting-started/"));
        assert!(html.starts_with("<nav"));
        assert!(html.contains(r#"class="docs-header-nav""#));
        assert!(html.contains(r#"aria-label="Site sections""#));
        assert!(html.contains(r#"class="docs-header-menu""#));
        assert!(html.contains(r#"class="docs-header-group""#));
        assert!(html.contains(r#"class="docs-header-trigger""#));
        assert!(html.contains(r#"class="docs-header-dropdown""#));

        // セクションタイトルがトリガーとして宣言順に出力される。
        let guide_idx = html.find("Guide").unwrap();
        let reference_idx = html.find("Reference").unwrap();
        assert!(guide_idx < reference_idx);

        assert!(html.contains(r#"href="/fandhe-frontend/guide/getting-started/""#));
        assert!(html.contains(r#"href="/fandhe-frontend/reference/api/""#));

        // トリガー自体のリンク先（`section.index_path`、イシュー #1012）。
        assert!(html.contains(r#"href="/fandhe-frontend/guide/intro/""#));
    }

    #[test]
    fn header_nav_highlights_only_current_page() {
        let nav = parse_nav(SAMPLE).unwrap();
        let html = render(&header_nav(&nav, "/guide/getting-started/"));
        // ページ完全一致用 `aria-current="page"`（ドロップダウン内リンク）と
        // セクション所属用 `aria-current="true"`（トリガー）は意味の軸が
        // 異なるため衝突しない。個別に件数を固定する（イシュー #1012）。
        assert_eq!(html.matches(r#"aria-current="page""#).count(), 1);
        assert_eq!(html.matches(r#"aria-current="true""#).count(), 1);
        assert!(html.contains("data-current"));
    }

    /// トリガーは `<a href>`（セクショントップページへの遷移リンク、
    /// イシュー #1012）のみで、`role`/`aria-expanded`/`aria-haspopup` の
    /// いずれも含まない（無 JS では状態更新できない ARIA 属性を静的に
    /// 約束しない、rustdoc 「`role`/`aria-expanded`/`aria-haspopup` を
    /// 付与しない理由」参照）ことを固定する。
    #[test]
    fn header_nav_trigger_has_no_menu_role_or_dynamic_aria_state() {
        let nav = parse_nav(SAMPLE).unwrap();
        let html = render(&header_nav(&nav, "/guide/getting-started/"));
        assert!(!html.contains("<button"));
        assert!(html.contains(r#"class="docs-header-trigger""#));
        assert!(html.contains(r#"href="/fandhe-frontend/guide/intro/""#));
        assert!(!html.contains("role="));
        assert!(!html.contains("aria-expanded"));
        assert!(!html.contains("aria-haspopup"));
    }

    /// トリガー href が常に `base_path + section.index_path` を指すことを
    /// 各セクションについて固定する（イシュー #1012）。
    #[test]
    fn header_nav_trigger_links_to_section_index_path() {
        let nav = parse_nav(SAMPLE).unwrap();
        let html = render(&header_nav(&nav, "/guide/getting-started/"));
        assert!(html.contains(r#"href="/fandhe-frontend/guide/intro/""#));
        assert!(html.contains(r#"href="/fandhe-frontend/reference/api/""#));
    }

    /// 現在セクションのトリガーにのみ `aria-current="true"` + `data-current`
    /// が付き、非現在セクションのトリガーには付かないことを固定する
    /// （イシュー #1012）。
    #[test]
    fn header_nav_marks_current_section_trigger_without_page_scope() {
        let nav = parse_nav(SAMPLE).unwrap();
        // Guide セクション配下の "/guide/getting-started/" が現在ページ。
        let html = render(&header_nav(&nav, "/guide/getting-started/"));
        let guide_trigger = r#"href="/fandhe-frontend/guide/intro/" class="docs-header-trigger" aria-current="true""#;
        assert!(html.contains(guide_trigger));
        // Reference セクションのトリガーには aria-current="true" が付かない。
        let reference_trigger_idx = html
            .find(r#"href="/fandhe-frontend/reference/api/""#)
            .unwrap();
        // 固定バイト幅の範囲演算子（`idx..idx+120`）はマルチバイト文字が
        // 境界にかかると char 境界不一致でパニックし得るため、
        // `char_indices` で 120 バイト以内に収まる直近の char 境界を
        // 探して切り出す安全な実装にする（レビュー指摘）。
        let rest = &html[reference_trigger_idx..];
        let safe_end = rest
            .char_indices()
            .map(|(byte_idx, _)| byte_idx)
            .chain(std::iter::once(rest.len()))
            .take_while(|&byte_idx| byte_idx <= 120)
            .last()
            .unwrap_or(0);
        let reference_trigger_slice = &rest[..safe_end];
        assert!(!reference_trigger_slice.contains(r#"aria-current="true""#));
    }

    /// ドロップダウン項目はグループ配下ページを一切含まず、直下ページの
    /// みを列挙する（Rule A、イシュー #1012）。直下ページに `index_path`
    /// と同一 path が無い場合のみ「すべて見る」項目が追加され、その href
    /// はトリガーと同一（`index_path`）になる。直下ページに `index_path`
    /// と同一 path がある場合は重複を避けるため追加されない。
    #[test]
    fn header_nav_dropdown_lists_only_direct_pages_and_adds_index_link_for_grouped_section() {
        let grouped = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/pre-styled-ui/"

[[section.page]]
title = "コンポーネント索引"
source = "components-pre-styled-ui.md"
path = "/components/pre-styled-ui/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "components/button.md"
path = "/components/button/"

[[section]]
title = "NoIndexInPages"
index_path = "/no-index-in-pages/index/"

[[section.page]]
title = "Direct"
source = "no-index/direct.md"
path = "/no-index-in-pages/direct/"

[[section.group]]
title = "Group"

[[section.group.page]]
title = "GroupPage"
source = "no-index/group-page.md"
path = "/no-index-in-pages/index/"
"#;
        let nav = parse_nav(grouped).unwrap();
        let html = render(&header_nav(&nav, "/components/pre-styled-ui/"));

        // グループ配下ページ（Button）はドロップダウンに出ない（否定的断定）。
        assert!(!html.contains("Button"));
        assert!(!html.contains(r#"href="/components/button/""#));

        // Components: 直下ページに index_path と同一 path があるため
        // 「すべて見る」は追加されず、ドロップダウンは 1 件のみ。
        // NoIndexInPages: 直下ページに index_path と同一 path が無いため
        // 「すべて見る」が追加され、href はトリガーと同一（index_path）。
        // 全体で「すべて見る」がちょうど 1 件（NoIndexInPages 分のみ）である
        // ことで両セクションの挙動差を機械固定する。
        assert!(html.contains("コンポーネント索引"));
        assert!(html.contains("Direct"));
        assert_eq!(html.matches("すべて見る").count(), 1);
        assert!(html.contains(r#"href="/no-index-in-pages/index/""#));

        // いずれの `ul.docs-header-dropdown` も空にならない。
        assert!(!html.contains(r#"class="docs-header-dropdown"></ul>"#));
    }

    #[test]
    fn header_nav_escapes_section_and_page_titles() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "<script>alert(1)</script>"
index_path = "/p1/"

[[section.page]]
title = "Quote\"Title"
source = "p1.md"
path = "/p1/"
"#;
        let nav = parse_nav(input).unwrap();
        let html = render(&header_nav(&nav, "/p1/"));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("Quote&quot;Title"));
    }

    // ---- `[[section]].index_path` 必須項目（イシュー #1010） ----

    #[test]
    fn rejects_section_without_index_path() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"

[[section.page]]
title = "P1"
source = "p1.md"
path = "/p1/"
"#;
        match parse_nav(input) {
            Err(NavError::MissingSectionIndex { line, section }) => {
                // `[[section]]` ヘッダ行（入力の 6 行目、先頭改行 1 行分を
                // 含めた行番号であることに注意）と一致することを固定する。
                assert_eq!(line, 6);
                assert_eq!(section, "A");
            }
            other => panic!("expected MissingSectionIndex, got {other:?}"),
        }
    }

    /// `[[section]]` が受け付けるキーは `title` / `index_path` の 2 つに
    /// 限定される（イシュー #1010 で 1 → 2 キーへ拡張）。未知キーを黙って
    /// 無視しない fail-closed 原則の回帰（拡張後もキーのホワイトリストが
    /// 崩れていないことを固定する）。
    #[test]
    fn rejects_unknown_key_in_section_still() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/p1/"
weight = "1"

[[section.page]]
title = "P1"
source = "p1.md"
path = "/p1/"
"#;
        match parse_nav(input) {
            Err(NavError::Parse { message, .. }) => {
                assert_eq!(message, "unknown key `weight` in [[section]]");
            }
            other => panic!("expected Parse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_index_path_pointing_to_other_section_page() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/b/p1/"

[[section.page]]
title = "P1"
source = "a-p1.md"
path = "/a/p1/"

[[section]]
title = "B"
index_path = "/b/p1/"

[[section.page]]
title = "P1"
source = "b-p1.md"
path = "/b/p1/"
"#;
        match parse_nav(input) {
            Err(NavError::SectionIndexNotFound {
                line,
                section,
                index_path,
            }) => {
                // `index_path = "/b/p1/"`（セクション A 側、入力の 8 行目）
                // と一致することを固定する（`index_path_line` の配線を
                // 実際に検証する。`line` を `..` で無視すると
                // `SectionIndexNotFound { line: section.header_line, .. }`
                // のような誤配線でもテストが通ってしまう）。
                assert_eq!(line, 8);
                assert_eq!(section, "A");
                assert_eq!(index_path, "/b/p1/");
            }
            other => panic!("expected SectionIndexNotFound, got {other:?}"),
        }
    }

    #[test]
    fn rejects_index_path_not_matching_any_page() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/nowhere/"

[[section.page]]
title = "P1"
source = "p1.md"
path = "/p1/"
"#;
        match parse_nav(input) {
            Err(NavError::SectionIndexNotFound {
                section,
                index_path,
                ..
            }) => {
                assert_eq!(section, "A");
                assert_eq!(index_path, "/nowhere/");
            }
            other => panic!("expected SectionIndexNotFound, got {other:?}"),
        }
    }

    /// トラバーサル形状の `index_path`（`page.path` として未登録）が
    /// `SectionIndexNotFound` として拒否されることを固定する（A01 対策の
    /// 中核テスト。`index_path` は `validate_page_path` を通過済みの
    /// `page.path` 集合との完全一致でのみ受理され、独立した形式検証を
    /// 持たないため、トラバーサル形状の値は単に「一致しない」ものとして
    /// 一様に拒否される。§3.6 参照）。
    #[test]
    fn rejects_traversal_shaped_index_path() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/../etc/"

[[section.page]]
title = "P1"
source = "p1.md"
path = "/p1/"
"#;
        match parse_nav(input) {
            Err(NavError::SectionIndexNotFound { index_path, .. }) => {
                assert_eq!(index_path, "/../etc/");
            }
            other => panic!("expected SectionIndexNotFound, got {other:?}"),
        }
    }

    #[test]
    fn rejects_duplicate_index_path_key() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/p1/"
index_path = "/p1/"

[[section.page]]
title = "P1"
source = "p1.md"
path = "/p1/"
"#;
        match parse_nav(input) {
            Err(NavError::Parse { message, .. }) => {
                assert!(message.contains("duplicate key `section.index_path`"));
            }
            other => panic!("expected Parse (duplicate key), got {other:?}"),
        }
    }

    #[test]
    fn parses_section_index_path() {
        let nav = parse_nav(SAMPLE).expect("valid nav.toml should parse");
        assert_eq!(nav.sections[0].index_path, "/guide/intro/");
        assert_eq!(nav.sections[1].index_path, "/reference/api/");
    }

    #[test]
    fn accepts_index_path_pointing_to_group_page() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/button/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/components/button/"
"#;
        let nav = parse_nav(input).expect("index_path pointing to a group page should be accepted");
        assert_eq!(nav.sections[0].index_path, "/components/button/");
    }

    #[test]
    fn accepts_site_root_as_index_path() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/"

[[section.page]]
title = "Top"
source = "index.md"
path = "/"
"#;
        let nav = parse_nav(input).expect("index_path = \"/\" should be accepted");
        assert_eq!(nav.sections[0].index_path, "/");
    }

    /// `EmptySection`（直下ページ・グループがともに 0 件）は `index_path`
    /// 欠落より優先して検出されることを固定する（§3.5 の検証順序、
    /// ドリフト防止テスト）。既存 fixture（`rejects_empty_section`）は
    /// `index_path` を意図的に持たないまま維持する。
    #[test]
    fn empty_section_takes_precedence_over_missing_index_path() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Empty"
"#;
        match parse_nav(input) {
            Err(NavError::EmptySection(title)) => assert_eq!(title, "Empty"),
            other => panic!("expected EmptySection (not MissingSectionIndex), got {other:?}"),
        }
    }

    /// `UnsafePagePath`（`page.path` の形式違反）は index 系の検証より
    /// 先に落ちることを固定する（§3.5 の検証順序、ドリフト防止テスト）。
    #[test]
    fn unsafe_page_path_takes_precedence_over_index_checks() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "A"
index_path = "/../p1/"

[[section.page]]
title = "P1"
source = "p1.md"
path = "/../p1/"
"#;
        assert!(matches!(parse_nav(input), Err(NavError::UnsafePagePath(_))));
    }

    // ---- `Nav::section_for_path`（イシュー #1010・#1013 用の解決 API） ----

    #[test]
    fn section_for_path_finds_section_by_direct_page() {
        let nav = parse_nav(SAMPLE).unwrap();
        let section = nav
            .section_for_path("/guide/getting-started/")
            .expect("direct page should resolve to its section");
        assert_eq!(section.title, "Guide");
    }

    #[test]
    fn section_for_path_finds_section_by_group_page() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/button/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/components/button/"
"#;
        let nav = parse_nav(input).unwrap();
        let section = nav
            .section_for_path("/components/button/")
            .expect("group page should resolve to its section");
        assert_eq!(section.title, "Components");
    }

    #[test]
    fn section_for_path_returns_none_for_unknown_path() {
        let nav = parse_nav(SAMPLE).unwrap();
        assert!(nav.section_for_path("/not-in-nav/").is_none());
    }
}

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
//! - `[[menu]]` array-of-tables（複数セクションを 1 つのヘッダー項目へ束ねる
//!   メニュー。`title` / `index_path` / `source` の 3 キー。イシュー #3699。
//!   メニューはページではなく [`Nav::all_pages`] には含めない。ページ化は
//!   後続 #3700 の責務）
//! - `[[menu.item]]` array-of-tables（直前の `[[menu]]` に属するメンバー。
//!   `section`〔メンバーセクションの `index_path`〕/ `description` の 2 キー。
//!   宣言順がパネル内の並び順になる）。`[[menu]]` の後に `[[section]]` を
//!   挟まず `[[section.page]]` 等が現れた場合は直前セクションへ誤吸着
//!   させず `NavError::Parse` にする
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
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::link_overlay::root as link_overlay_root;
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link_overlay::overlay as link_overlay_overlay;
use fandhe_frontend_pre_styled_ui::recipe::Size;

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
    /// 宣言順を保持したメニュー列（`[[menu]]`、イシュー #3699）。
    /// 宣言がなければ空で、その場合の挙動は従来と完全に同一。
    pub menus: Vec<Menu>,
}

/// `[[menu]]` 1 件分。複数セクションを 1 つのヘッダー項目（Assets 等）へ
/// 束ねる。後続 #3700（集約ページ）・#3701（ヘッダーのメガメニュー）・
/// #3703（フッター）が共通に参照するモデルで、#3700 で集約ページ
/// （[`crate::menu_index`]）の生成が参照を始めた。
///
/// `title` / `description` は後続イシューで必ず `text()` / `el()` 経由で
/// 出力する（既定エスケープを迂回しない）前提。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    /// ヘッダー・集約ページに表示するメニュー名。空でない。
    pub title: String,
    /// 集約ページの出力 URL パス。どの `page.path` とも衝突しない。
    pub index_path: String,
    /// 集約ページ原稿の repo 相対パス（実在は [`validate_sources`] が検証）。
    pub source: String,
    /// 宣言順（＝パネル内の並び順）のメンバー。1 件以上。
    pub items: Vec<MenuItem>,
}

impl Menu {
    /// 集約ページを通常ページと同じ組み立て API（`page_header` / `search_index`
    /// 等）へ渡すための写像。どのセクションにも属さず、`sidebar` /
    /// `prev_next` / `all_pages` には現れない（イシュー #3700）。
    #[must_use]
    pub fn as_page(&self) -> Page {
        Page {
            title: self.title.clone(),
            source: self.source.clone(),
            path: self.index_path.clone(),
        }
    }
}

/// `[[menu.item]]` 1 件分。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    /// メンバーセクションの `index_path`（既存セクションと一致検証済み）。
    pub section: String,
    /// カード用の 1 行説明（空でなく改行を含まない）。
    pub description: String,
}

/// [`Nav::header_entries`] の要素。ヘッダー等が走査する 1 項目。
#[derive(Debug, Clone, Copy)]
pub enum HeaderEntry<'a> {
    /// メニューに属さない単独セクション。
    Section(&'a Section),
    /// 複数セクションを束ねたメニュー。
    Menu(&'a Menu),
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

    /// `section` を束ねるメニュー（無ければ `None`）。`index_path` は全体で
    /// 一意なので文字列比較で同一性を判定する。
    fn menu_of_section(&self, section: &Section) -> Option<&Menu> {
        self.menus
            .iter()
            .find(|m| m.items.iter().any(|i| i.section == section.index_path))
    }

    /// ヘッダー等が走査する項目列を返す（イシュー #3699）。
    ///
    /// 並び規則: メニューに属さないセクションは宣言順。メニューは、
    /// メンバーのうち宣言順で最も早いセクションの位置を占める（以降の
    /// メンバーはスキップ）。現行 `site/nav.toml` では Getting Started /
    /// Guides / Assets / API Reference の順になる。メニュー宣言が無ければ
    /// `sections` と同順の [`HeaderEntry::Section`] 列（後方互換）。
    pub fn header_entries(&self) -> Vec<HeaderEntry<'_>> {
        let mut out = Vec::with_capacity(self.sections.len());
        let mut emitted: Vec<&str> = Vec::new();
        for section in &self.sections {
            match self.menu_of_section(section) {
                None => out.push(HeaderEntry::Section(section)),
                Some(menu) => {
                    if !emitted.contains(&menu.index_path.as_str()) {
                        emitted.push(menu.index_path.as_str());
                        out.push(HeaderEntry::Menu(menu));
                    }
                }
            }
        }
        out
    }

    /// `path` がメンバーセクション配下のページ、またはメニュー自身の
    /// `index_path` ならそのメニューを返す。`menu.index_path` は
    /// `page.path` と衝突しない（パース時検証）ため両分岐は排他。
    pub fn menu_for_path(&self, path: &str) -> Option<&Menu> {
        if let Some(menu) = self.menus.iter().find(|m| m.index_path == path) {
            return Some(menu);
        }
        let section = self.section_for_path(path)?;
        self.menu_of_section(section)
    }

    /// `menu` のメンバーを `items` の宣言順で `(セクション, 項目)` として返す。
    /// 検証済みのため通常欠落しないが、解決できない項目は黙って飛ばす
    /// （`unwrap` / `expect` を使わない）。
    pub fn menu_members<'a>(
        &'a self,
        menu: &'a Menu,
    ) -> impl Iterator<Item = (&'a Section, &'a MenuItem)> {
        menu.items.iter().filter_map(move |item| {
            self.sections
                .iter()
                .find(|s| s.index_path == item.section)
                .map(|s| (s, item))
        })
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
    /// `path` のページが属するグループを返す（直下ページ・未登録パスは `None`）。
    ///
    /// ページ見出しのパンくず（`crate::page_header`、イシュー #3607）が
    /// 「セクション / グループ / ページ」の中間項目を決めるために使う。
    /// `groups` を呼び出し側で手繰る二重ループを作らないための唯一の解決経路。
    pub fn group_for_path(&self, path: &str) -> Option<&Group> {
        self.groups
            .iter()
            .find(|g| g.pages.iter().any(|p| p.path == path))
    }
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

/// セクションの「見出し一覧」1 項目（#3670）。サイドバーとヘッダー popup が
/// 共有する唯一の情報源で、[`Section::headings`] が返す。
#[derive(Debug, Clone, Copy)]
pub enum SectionHeading<'a> {
    /// セクション直下ページ（リンク 1 件）。
    Page(&'a Page),
    /// グループ見出し（サイドバーでは `<details>`、popup では索引内アンカーへのリンク）。
    Group(&'a Group),
}

impl Section {
    /// 「直下ページ（宣言順）→ グループ（宣言順）」の見出し一覧を返す。
    /// 順序は [`Section::all_pages`] と同じ契約。[`sidebar`] と [`header_nav`] は
    /// 見出しをこの関数以外で数えない（両者の項目集合・順序・表記の一致を構造で保証する）。
    pub fn headings(&self) -> impl Iterator<Item = SectionHeading<'_>> {
        self.pages
            .iter()
            .map(SectionHeading::Page)
            .chain(self.groups.iter().map(SectionHeading::Group))
    }
}

/// グループ名からアンカー id を決定的に作る。見出し自動採番（`layout` の
/// `with_heading_anchors`）と同じ `slugify` に委譲するため、Themes / Primitives の
/// 既存 id と byte 単位で一致する。サイドバー・popup・索引ページが共有する。
/// 索引ページに同 slug の先行見出しがあると `-2` が付いてずれるが、その場合は
/// リンク検証が失敗する（fail-closed）。
pub fn group_anchor_id(title: &str) -> String {
    crate::layout::slugify(title)
}

/// ヘッダー popup のグループ見出しリンク先。
///
/// 通常は索引ページ内の該当カテゴリ位置（`#<slug>`）を返す。グループ見出しの
/// アンカーは索引ページ生成処理が付与するため、`index_path` がグループ配下の
/// ページを指す構成（アンカーを持つ索引ページがない構成）では存在しない
/// アンカーへのリンクになる。その場合はアンカーを付けず、グループ先頭ページへ
/// リンクして有効な遷移先を保つ（グループが空ならセクショントップへ戻す）。
pub fn group_href(nav: &Nav, section: &Section, group: &Group) -> String {
    let index_in_group = section
        .groups
        .iter()
        .any(|g| g.pages.iter().any(|p| p.path == section.index_path));
    if index_in_group {
        return match group.pages.first() {
            Some(p) => href(nav, &p.path),
            None => href(nav, &section.index_path),
        };
    }
    format!(
        "{}#{}",
        href(nav, &section.index_path),
        group_anchor_id(&group.title)
    )
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
    /// 入力サイズが `MAX_INPUT_BYTES` を超えた。
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

/// パース中に組み立て途上のメニュー（イシュー #3699）。行番号は後段検証の
/// エラー報告用に保持する。
struct MenuBuilder {
    header_line: usize,
    title: Option<(String, usize)>,
    index_path: Option<(String, usize)>,
    source: Option<(String, usize)>,
    items: Vec<MenuItemBuilder>,
}

struct MenuItemBuilder {
    header_line: usize,
    section: Option<(String, usize)>,
    description: Option<(String, usize)>,
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
    /// `[[menu]]`（`menus` の末尾）。
    Menu(usize),
    /// `(menu index, item index)`。
    MenuItem(usize, usize),
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

/// `path` が `nav.toml` の `page.path` として安全か（`/` 始まり・`/` 終わり・
/// セグメントが英数字/`-`/`_` のホワイトリストのみ）を判定する述語。
///
/// [`validate_page_path`] から抽出した純判定ロジック（イシュー #1016）。
/// `crate::redirect` が旧 URL 互換のリダイレクト宣言（`from`/`to`）の
/// パス形状検証にも同じホワイトリストを要求するため `pub(crate)` として
/// 切り出し、ページパス・リダイレクトパスの許可規則を単一実装点に保つ
/// （2 箇所で微妙に異なる正規表現を持たせて片方だけ緩む事故を防ぐ）。
/// `"/"`（サイトトップ）・`"//"` のような縮退ケースをセグメントなしとして
/// 許可する対称性は本モジュール（ページパス）由来の既存仕様であり、
/// `crate::redirect` 側はこの述語を通した上でさらに厳しい制約
/// （空セグメント拒否等）を追加で課す。
pub(crate) fn is_safe_page_path(path: &str) -> bool {
    if !path.starts_with('/') || !path.ends_with('/') {
        return false;
    }
    if path.len() == 1 {
        // "/"（サイトトップ）はセグメントなしで許可する。
        //
        // 単一文字 "/" は開始・終了の '/' が同一バイトを指すため、下の
        // `path[1..path.len() - 1]` スライス（1..0）は範囲が逆転してパニック
        // する（イシュー #473 実装時に検出）。長さ 1 の場合はスライス計算に
        // 入る前に早期リターンする。
        return true;
    }
    let inner = &path[1..path.len() - 1];
    if inner.is_empty() {
        // "//" のような縮退ケース。セグメントなしとして許可する
        // （現状 nav.toml では使用しないが、ホワイトリスト方式の
        // 対称性のため拒否しない）。
        return true;
    }
    inner.split('/').all(is_safe_path_segment)
}

fn validate_page_path(path: &str) -> Result<(), NavError> {
    if is_safe_page_path(path) {
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
    let mut menus: Vec<MenuBuilder> = Vec::new();
    // `[[menu]]` の後に `[[section]]` を挟まず `[[section.*]]` が現れた場合に
    // 直前セクションへ黙って吸着する事故を防ぐ（イシュー #3699）。
    let mut section_open = false;

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
                    section_open = true;
                }
                "menu" => {
                    menus.push(MenuBuilder {
                        header_line: line,
                        title: None,
                        index_path: None,
                        source: None,
                        items: Vec::new(),
                    });
                    ctx = Ctx::Menu(menus.len() - 1);
                    section_open = false;
                }
                "menu.item" => {
                    // 直前のコンテキストが `[[menu]]` / `[[menu.item]]` のときだけ
                    // 受理する。間に `[[section]]` 等を挟んだ離れた項目が、
                    // 以前のメニューへ黙って吸着するのを防ぐ（イシュー #3699）。
                    let midx = match ctx {
                        Ctx::Menu(i) | Ctx::MenuItem(i, _) => i,
                        _ if menus.is_empty() => {
                            return Err(parse_err(
                                line,
                                "[[menu.item]] appeared before any [[menu]]",
                            ))
                        }
                        _ => {
                            return Err(parse_err(
                                line,
                                "[[menu.item]] must directly follow a [[menu]] or another [[menu.item]]",
                            ))
                        }
                    };
                    menus[midx].items.push(MenuItemBuilder {
                        header_line: line,
                        section: None,
                        description: None,
                    });
                    ctx = Ctx::MenuItem(midx, menus[midx].items.len() - 1);
                }
                "section.page" => {
                    let sidx = sections.len().checked_sub(1).ok_or_else(|| {
                        parse_err(line, "[[section.page]] appeared before any [[section]]")
                    })?;
                    if !section_open {
                        return Err(parse_err(
                            line,
                            "[[section.page]] must follow a [[section]] (not directly after [[menu]])",
                        ));
                    }
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
                    if !section_open {
                        return Err(parse_err(
                            line,
                            "[[section.group]] must follow a [[section]] (not directly after [[menu]])",
                        ));
                    }
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
                    if !section_open {
                        return Err(parse_err(
                            line,
                            "[[section.group.page]] must follow a [[section]] (not directly after [[menu]])",
                        ));
                    }
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
            Ctx::Menu(midx) => match key {
                "title" => set_once_at(&mut menus[midx].title, value, line, "menu.title")?,
                "index_path" => {
                    set_once_at(&mut menus[midx].index_path, value, line, "menu.index_path")?
                }
                "source" => set_once_at(&mut menus[midx].source, value, line, "menu.source")?,
                other => {
                    return Err(parse_err(
                        line,
                        format!("unknown key `{other}` in [[menu]]"),
                    ))
                }
            },
            Ctx::MenuItem(midx, iidx) => {
                let item = &mut menus[midx].items[iidx];
                match key {
                    "section" => set_once_at(&mut item.section, value, line, "menu.item.section")?,
                    "description" => {
                        set_once_at(&mut item.description, value, line, "menu.item.description")?
                    }
                    other => {
                        return Err(parse_err(
                            line,
                            format!("unknown key `{other}` in [[menu.item]]"),
                        ))
                    }
                }
            }
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

    let out_menus = finalize_menus(menus, &out_sections, &seen_paths)?;

    Ok(Nav {
        site,
        sections: out_sections,
        menus: out_menus,
    })
}

/// 値と行番号を一度だけ保存する（重複キーは `NavError::Parse`）。
fn set_once_at(
    slot: &mut Option<(String, usize)>,
    value: String,
    line: usize,
    name: &str,
) -> Result<(), NavError> {
    if slot.is_some() {
        return Err(parse_err(line, format!("duplicate key `{name}`")));
    }
    *slot = Some((value, line));
    Ok(())
}

/// 必須キーを取り出す。欠落は `header_line` を報告行にする。
fn require_menu_key(
    slot: Option<(String, usize)>,
    header_line: usize,
    what: &str,
    key: &str,
) -> Result<(String, usize), NavError> {
    slot.ok_or_else(|| {
        parse_err(
            header_line,
            format!("{what} is missing required key `{key}`"),
        )
    })
}

/// [`MenuBuilder`] 群を検証済み [`Menu`] へ確定する（イシュー #3699）。
/// セクション・`page.path` の確定後に呼ぶことで、メニューを含まない入力の
/// エラー優先順位を従来から変えない。検証はすべて fail-closed。
fn finalize_menus(
    menus: Vec<MenuBuilder>,
    sections: &[Section],
    page_paths: &BTreeSet<String>,
) -> Result<Vec<Menu>, NavError> {
    let mut out: Vec<Menu> = Vec::with_capacity(menus.len());
    // (section index_path, 所属メニュー名)。複数メニュー所属の検出用。
    let mut owner: Vec<(String, String)> = Vec::new();
    for mb in menus {
        let (title, title_line) = require_menu_key(mb.title, mb.header_line, "menu", "title")?;
        let (index_path, index_line) =
            require_menu_key(mb.index_path, mb.header_line, "menu", "index_path")?;
        let (source, source_line) = require_menu_key(mb.source, mb.header_line, "menu", "source")?;
        if title.trim().is_empty() {
            return Err(parse_err(title_line, "menu title must not be empty"));
        }
        if !is_safe_page_path(&index_path) {
            return Err(parse_err(
                index_line,
                format!("menu index_path `{index_path}` is not a safe page path"),
            ));
        }
        if page_paths.contains(&index_path) {
            return Err(parse_err(
                index_line,
                format!("menu index_path `{index_path}` collides with an existing page.path"),
            ));
        }
        if out.iter().any(|m| m.index_path == index_path) {
            return Err(parse_err(
                index_line,
                format!("duplicate menu index_path `{index_path}`"),
            ));
        }
        if validate_source_shape(&source).is_err() {
            return Err(parse_err(
                source_line,
                format!("menu source `{source}` is not a safe relative path"),
            ));
        }
        // 原稿の共有を許すと linkcheck の source → path 対応表が後勝ちになり、
        // 通常ページ宛の相対 `.md` リンクがメニュー集約ページへ向いてしまう。
        if sections
            .iter()
            .flat_map(Section::all_pages)
            .any(|p| p.source == source)
            || out.iter().any(|m| m.source == source)
        {
            return Err(parse_err(
                source_line,
                format!("menu source `{source}` is already used by another page or menu"),
            ));
        }
        if mb.items.is_empty() {
            return Err(parse_err(
                mb.header_line,
                format!("menu `{title}` has no items"),
            ));
        }
        let mut items: Vec<MenuItem> = Vec::with_capacity(mb.items.len());
        for ib in mb.items {
            let (section, section_line) =
                require_menu_key(ib.section, ib.header_line, "menu item", "section")?;
            let (description, desc_line) =
                require_menu_key(ib.description, ib.header_line, "menu item", "description")?;
            if description.trim().is_empty() {
                return Err(parse_err(
                    desc_line,
                    "menu item description must not be empty",
                ));
            }
            if description.contains('\n') {
                return Err(parse_err(
                    desc_line,
                    "menu item description must be a single line",
                ));
            }
            if !sections.iter().any(|s| s.index_path == section) {
                return Err(parse_err(
                    section_line,
                    format!(
                        "menu item section `{section}` does not match any [[section]] index_path"
                    ),
                ));
            }
            if items.iter().any(|i| i.section == section) {
                return Err(parse_err(
                    section_line,
                    format!("section `{section}` is listed more than once in menu `{title}`"),
                ));
            }
            if let Some((_, other)) = owner.iter().find(|(s, _)| *s == section) {
                return Err(parse_err(
                    section_line,
                    format!("section `{section}` already belongs to menu `{other}`"),
                ));
            }
            owner.push((section.clone(), title.clone()));
            items.push(MenuItem {
                section,
                description,
            });
        }
        out.push(Menu {
            title,
            index_path,
            source,
            items,
        });
    }
    Ok(out)
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
    // メニューはページではない（`all_pages()` に混ぜない）ため別ループで
    // 集約ページ原稿の実在を確認する（イシュー #3699）。
    for menu in &nav.menus {
        if !repo_root.join(&menu.source).is_file() {
            return Err(NavError::MissingSource(menu.source.clone()));
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
/// # セクションスコープ（イシュー #1013）
///
/// 描画対象は「現在ページが属するセクション 1 件のみ」に絞り込む
/// （[`Nav::section_for_path`] が唯一の解決経路。`pages`/`groups` を
/// 個別に手繰る新しい判定をここに作らない）。Components セクションが
/// グループ配下に 100 件超のページを持つため、無関係なセクションを
/// 開いているときまでその見出し・部品一覧がサイドバーへ付いてくる
/// 状態を解消する。
///
/// - `current_path` が nav 中のどの `page.path` にも一致しない場合
///   （サイトトップ・将来の 404 等）は**全セクションを描画するフォール
///   バック**を維持する。これはナビゲーションの見た目のみに関わる
///   意図的な fail-open であり、`docs-site` は公開静的サイトのため
///   サイドバーの可視性はアクセス境界ではない（空サイドバーという
///   実害のある UX 退行を避けるための安全側の既定）。加えて、
///   `current_path` は `crate::build::build_site` のページ生成ループが
///   [`Nav::all_pages`] から渡す値のみであり、`parse_nav` の形式検証
///   （§ `href`）を通過済みの nav 由来データに限られる。攻撃者制御の
///   入力でこの分岐へ到達する経路は存在しない。
/// - 他セクションへの到達性は本関数のスコープ外で担保される:
///   [`header_nav`]（全セクションのトリガー + 直下ページのドロップ
///   ダウン）・各セクションの `index_path` トップページ・`prev_next`
///   （セクション境界を跨ぐ挙動は本イシューで変更しない）・全文検索
///   インデックス（`assets/search-index.json`）。[`header_nav`] は本
///   イシューの対象外で全セクション列挙のまま（同関数 rustdoc 参照）。
/// - `aria-current`/`open` の付与ロジック・描画本体（見出し → 直下
///   ページ `ul` → グループ `<details>`）は不変。スコープ限定は走査
///   対象のスライスを絞るのみで、単一セクションを描画する処理自体は
///   1 経路のまま複製しない。
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
///     summary.docs-nav-group-summary          … グループ見出し（span.docs-nav-group-title + span.docs-nav-group-count > badge、#3611）
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
/// - `<summary>` の中身は phrasing の span（タイトルと件数 badge）のみで、見出し要素・リンク・ボタンを入れない。
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
    // 現在ページが属するセクションのみへ絞り込む（イシュー #1013）。
    // 未解決（nav 未登録 path）時は全セクション描画へフォールバックする
    // 契約 — 理由は本関数 rustdoc「セクションスコープ」節参照。
    let scoped: &[Section] = match nav.section_for_path(current_path) {
        Some(section) => std::slice::from_ref(section),
        None => &nav.sections,
    };
    let mut section_nodes: Vec<Node> = Vec::new();
    for section in scoped {
        section_nodes.push(heading(vec![], vec![text(section.title.clone())]));
        section_nodes.extend(section_sidebar_nodes(nav, section, current_path));
    }
    nav_list_root("Documentation", vec![("class", "sidebar")], section_nodes)
}

/// セクション 1 件分のサイドバー本体（見出し `h2` を除く、直下ページ `ul` と
/// グループ `<details>` の列）を作る。[`sidebar`] と [`nav_drawer`]（現在セクション側）
/// が共有する唯一の経路で、`Section::headings` を情報源にする（イシュー #3674）。
fn section_sidebar_nodes(nav: &Nav, section: &Section, current_path: &str) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::new();
    // 直下ページが 0 件のときは `ul` を出さない。
    let mut items: Vec<Node> = Vec::new();
    for heading_item in section.headings() {
        match heading_item {
            SectionHeading::Page(page) => {
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
            SectionHeading::Group(group) => {
                if !items.is_empty() {
                    nodes.push(list(vec![], std::mem::take(&mut items)));
                }
                nodes.push(group_node(nav, group, current_path));
            }
        }
    }
    if !items.is_empty() {
        nodes.push(list(vec![], items));
    }
    nodes
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
    // summary は見出し要素・リンク・ボタンを含めず phrasing の span のみで構成する
    // （開閉操作を summary に一本化する無 JS 契約）。件数 badge の見た目は
    // Primitives ページ（recipe 抜き CSS）でも崩れないよう docs 側 CSS で完結させる
    // （イシュー #3611）。件数は数値文字列を `text()` 経由で渡し既定エスケープを保つ。
    let count = badge(
        &BadgeProps {
            size: Size::Sm,
            ..BadgeProps::default()
        },
        vec![],
        vec![text(group.pages.len().to_string())],
    );
    let summary = el(
        "summary",
        vec![("class", "docs-nav-group-summary")],
        vec![
            el(
                "span",
                vec![("class", "docs-nav-group-title")],
                vec![text(group.title.clone())],
            ),
            el("span", vec![("class", "docs-nav-group-count")], vec![count]),
        ],
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

/// セクション 1 件の見出し一覧（直下ページのリンクと、グループ見出しの索引アンカー
/// リンク）を `li` の列として作る。[`header_nav`] のドロップダウンと [`nav_drawer`]
/// （現在でないセクション側）が共有する唯一の経路で、グループ配下の個別ページは出さない
/// （イシュー #3670 / #3674）。
fn section_heading_items(nav: &Nav, section: &Section, current_path: &str) -> Vec<Node> {
    let mut dropdown_items: Vec<Node> = Vec::new();
    for heading_item in section.headings() {
        match heading_item {
            SectionHeading::Page(page) => {
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
            SectionHeading::Group(group) => {
                // headless の `nav_link` は呼び出し側の `aria-current` を捨てる
                // ため、所属表示（"true"）を付けるグループ見出しは `el` で組む。
                let group_link = group_href(nav, section, group);
                let mut attrs: Vec<(&str, &str)> = vec![("href", &group_link)];
                if group_link == href(nav, current_path) {
                    // `index_path` がグループ配下を指す構成では、リンク先が
                    // 現在ページそのものになる。ページ完全一致の意味軸
                    // （`"page"`）を保ち、所属のみの `"true"` と区別する。
                    attrs.push(("aria-current", "page"));
                    attrs.push(("data-current", ""));
                } else if group.pages.iter().any(|p| p.path == current_path) {
                    attrs.push(("aria-current", "true"));
                    attrs.push(("data-current", ""));
                }
                dropdown_items.push(item(
                    vec![],
                    vec![el("a", attrs, vec![text(group.title.clone())])],
                ));
            }
        }
    }
    dropdown_items
}

/// ヘッダーのナビ [`Node`] を生成する（イシュー #908 / #1012 / #3701）。
///
/// [`Nav::header_entries`] を走査し、メニューに属さない単独セクションは
/// トリガー `<a href>`（セクショントップページへの遷移リンク）だけを出す。
/// `[[menu]]`（現行は Assets）はトリガー（メニュー索引ページ `/assets/` への
/// リンク）と、直後に全幅のメガメニューパネル（メンバーセクションのカード列）を
/// 出す。セクション別の見出し一覧 popup は #3701（ユーザー判断 2026-10-04、#3670 の
/// 見直し）で廃止した。見出し一覧は [`nav_drawer`] とサイドバーが担う。
///
/// # `pre-styled-ui menu` / ARIA 動的状態を使わない理由
///
/// 開閉は JS を使わず CSS の `:hover` / `:focus-within` のみで行う
/// （`docs/design/docs-site-three-column-redesign.md` §3.5 案 (b)）。WAI-ARIA
/// `menu` ロールは操作コマンド向けで文書リンク集には不適切、かつ pre-styled-ui
/// `menu` は wasm 配線前提で無 JS の docs-site では動かない。開閉状態を JS で
/// 更新できないため、`role` / `aria-expanded` / `aria-haspopup` / `aria-controls`
/// は付与せず（静的な固定値は支援技術へ虚偽の状態を伝える）、要素に `id` も付けない。
/// トリガーは通常のリンクで、パネルの代わりに遷移できる `/assets/` 集約ページを持つ。
///
/// # DOM 構造
///
/// ```text
/// nav.docs-header-nav[aria-label="Site sections"]  … headless nav_list root
///   ul.docs-header-menu                            … nav_list list
///     li.docs-header-group（単独セクション）
///       a.docs-header-trigger[href=base_path+index_path]
///         （現在セクションのみ aria-current="true" + data-current）
///     li.docs-header-group（メニュー）
///       a.docs-header-trigger[href=base_path+menu.index_path]
///         （現在ページがメンバー配下かメニュー索引なら aria-current="true" + data-current）
///       div.docs-header-mega                       … 全幅パネル（包含ブロックは div.docs-header-inner）
///         ul.docs-header-mega-grid
///           li.docs-header-mega-cell（`[[menu.item]]` の宣言順）
///             a.docs-header-mega-card[href=base_path+section.index_path]
///               （現在ページを含むメンバーのみ aria-current="true" + data-current）
///               span.docs-header-mega-title / span.docs-header-mega-desc
/// ```
///
/// `aria-current` はヘッダーでは所属を表す `"true"` だけを使う（ページ完全一致の
/// `"page"` は [`nav_drawer`] 側の見出し一覧にのみ残る）。
///
/// タイトル・説明・href はすべて [`fandhe_frontend_core::el`] / `text` の既定
/// エスケープ（REQ-1）を経由し、`raw_html()` と HTML 文字列の直接組み立ては
/// 使わない。href は [`parse_nav`] で検証済みの `index_path` と `base_path` だけから作る。
pub fn header_nav(nav: &Nav, current_path: &str) -> Node {
    let mut groups: Vec<Node> = Vec::new();
    for entry in nav.header_entries() {
        match entry {
            HeaderEntry::Section(section) => {
                let trigger_href = href(nav, &section.index_path);
                let is_current = section.all_pages().any(|p| p.path == current_path);
                let trigger = header_trigger(&trigger_href, is_current, &section.title);
                groups.push(item(vec![("class", "docs-header-group")], vec![trigger]));
            }
            HeaderEntry::Menu(menu) => {
                let trigger_href = href(nav, &menu.index_path);
                let is_current = nav
                    .menu_for_path(current_path)
                    .is_some_and(|m| m.index_path == menu.index_path);
                let trigger = header_trigger(&trigger_href, is_current, &menu.title);
                let panel = mega_panel(nav, menu, current_path);
                groups.push(item(
                    vec![("class", "docs-header-group")],
                    vec![trigger, panel],
                ));
            }
        }
    }
    let menu = list(vec![("class", "docs-header-menu")], groups);
    // サイドバー（`sidebar()`）の `aria-label="Documentation"` と区別できる
    // ラベルにする（複数 nav ランドマークの識別用）。
    nav_list_root(
        "Site sections",
        vec![("class", "docs-header-nav")],
        vec![menu],
    )
}

/// ヘッダーのトリガーリンク（`a.docs-header-trigger`）を作る。
/// `is_current` のとき所属表示 `aria-current="true"` + `data-current` を付ける。
fn header_trigger(trigger_href: &str, is_current: bool, title: &str) -> Node {
    let mut attrs: Vec<(&str, &str)> =
        vec![("href", trigger_href), ("class", "docs-header-trigger")];
    if is_current {
        attrs.push(("aria-current", "true"));
        attrs.push(("data-current", ""));
    }
    el("a", attrs, vec![text(title.to_string())])
}

/// Assets 等のメニューのメガメニューパネルを作る（[`header_nav`] から呼ばれる）。
///
/// カードは `[[menu.item]]` の宣言順で、メンバーセクションのタイトルと
/// `description` を表示する。見出し要素は使わず（見出しアウトラインを汚さない）、
/// headless の `list()` も使わない（`nav_list` anatomy の縦積み規則と grid の衝突回避）。
fn mega_panel(nav: &Nav, menu: &Menu, current_path: &str) -> Node {
    let cells: Vec<Node> = nav
        .menu_members(menu)
        .map(|(section, menu_item)| {
            let card_href = href(nav, &section.index_path);
            let mut attrs: Vec<(&str, &str)> =
                vec![("href", &card_href), ("class", "docs-header-mega-card")];
            if section.all_pages().any(|p| p.path == current_path) {
                attrs.push(("aria-current", "true"));
                attrs.push(("data-current", ""));
            }
            let card = el(
                "a",
                attrs,
                vec![
                    el(
                        "span",
                        vec![("class", "docs-header-mega-title")],
                        vec![text(section.title.clone())],
                    ),
                    el(
                        "span",
                        vec![("class", "docs-header-mega-desc")],
                        vec![text(menu_item.description.clone())],
                    ),
                ],
            );
            el("li", vec![("class", "docs-header-mega-cell")], vec![card])
        })
        .collect();
    el(
        "div",
        vec![("class", "docs-header-mega")],
        vec![el("ul", vec![("class", "docs-header-mega-grid")], cells)],
    )
}

/// 全セクションへ移れるナビ drawer [`Node`] を生成する（イシュー #3674）。
///
/// 768px 未満（ヘッダーナビが非表示）と、768px 以上の `(hover: none)` 端末
/// （`:hover` の popup が安定しない）で、ヘッダーのハンバーガー
/// （`crate::layout` の checkbox hack）から開く。`crate::build::build_site` と
/// `crate::not_found` が [`header_nav`] とは別 Node として
/// `crate::layout::docs_page_with_layout` へ渡す（`header_nav` の項目件数を数える
/// 検証へ drawer のリンクが混ざらないよう分離している）。
///
/// # DOM 構造
///
/// ```text
/// nav.docs-nav-drawer[aria-label="Site navigation"]   … headless nav_list root
///   ul.docs-nav-drawer-sections
///     li.docs-nav-drawer-section（セクションごと、宣言順）
///       a.docs-nav-drawer-section-link[href=base_path+index_path]（現在セクションのみ aria-current="true"）
///       details.docs-nav-drawer-details[open は現在セクションのみ]
///         summary.docs-nav-drawer-summary > span.docs-nav-drawer-summary-text（視覚上隠す）
///         div.docs-nav-drawer-body
///           現在セクション … [`sidebar`] と同じ本体（直下ページ + グループ `details`）
///           他セクション   … ul.docs-nav-drawer-list（[`header_nav`] と同じ見出し一覧）
/// ```
///
/// - セクション索引へのリンクを `summary` の外に置き、「ハンバーガー → セクション
///   リンク」の 2 タップで全セクションへ移れるようにする。`summary` は開閉専用で
///   リンクを含めない（[`sidebar`] の #940 判断と同じ）。
/// - 現在セクションにはサイドバー形式（個別ページまで）だけを出し、popup 形式との
///   二重掲載はしない。どちらも [`Section::headings`] を唯一の情報源にする。
/// - `role`/`aria-expanded`/`aria-haspopup` は付けない（[`header_nav`] と同じ理由）。
///   開閉状態は checkbox のネイティブなチェック状態が支援技術へ伝わる。
/// - 要素に `id` を一切付けない（固定 `id` は `crate::layout` の toggle のみ）。
///
/// タイトル・href はすべて headless 層と [`fandhe_frontend_core::el`]/`text` を経由し、
/// 既定エスケープ（REQ-1）を通る。`raw_html()` と HTML 文字列の直接組み立ては使わない。
/// href は [`parse_nav`] で検証済みの `index_path` / `page.path` と [`group_href`] だけから作る。
pub fn nav_drawer(nav: &Nav, current_path: &str) -> Node {
    let mut rows: Vec<Node> = Vec::new();
    for section in &nav.sections {
        let is_current_section = section.all_pages().any(|p| p.path == current_path);
        let section_href = href(nav, &section.index_path);
        let mut link_attrs: Vec<(&str, &str)> = vec![
            ("href", &section_href),
            ("class", "docs-nav-drawer-section-link"),
        ];
        if is_current_section {
            link_attrs.push(("aria-current", "true"));
            link_attrs.push(("data-current", ""));
        }
        let section_link = el("a", link_attrs, vec![text(section.title.clone())]);

        let body_children = if is_current_section {
            section_sidebar_nodes(nav, section, current_path)
        } else {
            vec![list(
                vec![("class", "docs-nav-drawer-list")],
                section_heading_items(nav, section, current_path),
            )]
        };
        let body = el(
            "div",
            vec![("class", "docs-nav-drawer-body")],
            body_children,
        );
        let summary = el(
            "summary",
            vec![("class", "docs-nav-drawer-summary")],
            vec![el(
                "span",
                vec![("class", "docs-nav-drawer-summary-text")],
                vec![text(format!("Pages in {}", section.title))],
            )],
        );
        let mut details_attrs = vec![("class", "docs-nav-drawer-details")];
        if is_current_section {
            details_attrs.push(("open", ""));
        }
        let details = el("details", details_attrs, vec![summary, body]);
        rows.push(item(
            vec![("class", "docs-nav-drawer-section")],
            vec![section_link, details],
        ));
    }
    nav_list_root(
        "Site navigation",
        vec![("class", "docs-nav-drawer")],
        vec![list(vec![("class", "docs-nav-drawer-sections")], rows)],
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

/// 前後ページャのカードが指す方向。DOM 順（矢印アイコンの位置）と
/// 方向ラベルの文言を決める。
#[derive(Clone, Copy)]
enum PagerSide {
    Prev,
    Next,
}

/// 前後ページャ用のシェブロン矢印（線画、装飾のため `aria-hidden`）。
/// `docs_layout_prev_next` block を手本にするが、block の私有関数は
/// 呼ばずに docs-site 側へ置く（本番から `blocks::*` を呼ばない契約）。
fn chevron_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 1 側分のカード（方向ラベル・ページ名・セクション名・矢印）を組む。
///
/// 文言はすべて `text()` ノード経由（既定エスケープ）。ラベル群は
/// `<a>` 内に収めるため `p` ではなく `span` で組む（phrasing 維持。
/// Primitives ページは recipe CSS を読まず docs 側 CSS のみで成立させる）。
fn pager_card(nav: &Nav, page: &Page, side: PagerSide) -> Node {
    let (label, side_class) = match side {
        PagerSide::Prev => ("前へ", "prev"),
        PagerSide::Next => ("次へ", "next"),
    };
    let mut meta_children = vec![
        el(
            "span",
            vec![("class", "docs-pager-label")],
            vec![text(label)],
        ),
        el(
            "span",
            vec![("class", "docs-pager-title")],
            vec![text(page.title.clone())],
        ),
    ];
    if let Some(section) = nav.section_for_path(&page.path) {
        meta_children.push(el(
            "span",
            vec![("class", "docs-pager-section")],
            vec![text(section.title.clone())],
        ));
    }
    let meta = el("span", vec![("class", "docs-pager-meta")], meta_children);
    let body_children = match side {
        PagerSide::Prev => vec![chevron_icon("M15 4l-8 8 8 8"), meta],
        PagerSide::Next => vec![meta, chevron_icon("M9 4l8 8-8 8")],
    };
    let card_node = card::root(
        CardProps {
            variant: CardVariant::Outline,
            ..CardProps::default()
        },
        vec![],
        vec![card::body(vec![], body_children)],
    );
    let link_href = href(nav, &page.path);
    link_overlay_root(
        vec![("class", side_class)],
        vec![link_overlay_overlay(
            &link_href,
            vec![("class", "docs-pager-link")],
            vec![card_node],
        )],
    )
}

/// 前後ページリンクの [`Node`]（`<nav class="prev-next">` 配下に存在する
/// 側のみのカード型ページャ、イシュー #3608）を生成する。
///
/// 構造は外枠 `div.prev|next`（headless `link_overlay::root`）の内側に、
/// 唯一のアンカー `a.docs-pager-link`（`link_overlay::overlay`、全面クリック・
/// フォーカス対象）があり、その中に pre-styled `card`（Outline）と矢印 `icon`
/// + ラベル群が入る。アンカーを入れ子にしない不変条件を保つ。
///
/// `link_overlay::stylesheet()` は `overlay` を `position: absolute` にして
/// 高さ 0 に潰すため site.css へ積まない（イシュー #910）。Primitives ページ
/// は recipe CSS を含まない `site-primitives.css` を読むため、カード装飾と
/// アイコン寸法は `crate::site_theme` の docs 側 CSS だけで成立させる。
pub fn prev_next_nav(nav: &Nav, current_path: &str) -> Node {
    let (prev, next) = prev_next(nav, current_path);
    let mut children: Vec<Node> = Vec::new();
    if let Some(page) = prev {
        children.push(pager_card(nav, page, PagerSide::Prev));
    }
    if let Some(page) = next {
        children.push(pager_card(nav, page, PagerSide::Next));
    }
    el(
        "nav",
        vec![("class", "prev-next"), ("aria-label", "前後のページ")],
        children,
    )
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
    fn sidebar_lists_only_current_section_pages_in_document_order_with_current_highlighted() {
        // イシュー #1013: サイドバーは現在ページが属するセクション
        // （ここでは Guide）のみを描画し、他セクション（Reference）は
        // 出さない。
        let nav = parse_nav(SAMPLE).unwrap();
        let html = render(&sidebar(&nav, "/guide/getting-started/"));
        // 文書順: Introduction, Getting Started
        let intro_idx = html.find("Introduction").unwrap();
        let getting_started_idx = html.find("Getting Started").unwrap();
        assert!(intro_idx < getting_started_idx);

        assert!(html.contains(r#"href="/fandhe-frontend/guide/getting-started/""#));
        // 現在ページのみ aria-current="page"（+ data-current）を持つ
        // （イシュー #756 で headless nav_list へ移行、`class="current"` は
        // 廃止し属性のみに一本化した）。
        assert_eq!(html.matches(r#"aria-current="page""#).count(), 1);
        assert!(html.contains("data-current"));
        assert!(!html.contains(r#"class="current""#));
        // セクション見出しはスコープ対象の 1 件のみ描画される。
        assert_eq!(html.matches("<h2").count(), 1);
    }

    #[test]
    fn sidebar_falls_back_to_all_sections_when_current_path_absent() {
        // イシュー #1013: current_path が nav 未登録のときは全セクション
        // 描画へフォールバックする（空サイドバー事故の防止。
        // `sidebar` rustdoc「セクションスコープ」節参照）。
        let nav = parse_nav(SAMPLE).unwrap();
        let html = render(&sidebar(&nav, "/not-in-nav/"));
        assert!(!html.contains("aria-current"));
        assert!(!html.contains(r#"class="current""#));
        assert_eq!(html.matches("<h2").count(), 2);
        assert!(html.contains(r#"href="/fandhe-frontend/guide/intro/""#));
        assert!(html.contains(r#"href="/fandhe-frontend/guide/getting-started/""#));
        assert!(html.contains(r#"href="/fandhe-frontend/reference/api/""#));
    }

    #[test]
    fn sidebar_omits_pages_from_other_sections() {
        // イシュー #1013 の回帰固定: スコープ対象外セクションの見出し・
        // href が 1 件も漏れないことを否定的断定で fail-closed に守る。
        let nav = parse_nav(SAMPLE).unwrap();

        let html_guide = render(&sidebar(&nav, "/guide/getting-started/"));
        assert!(!html_guide.contains(r#"href="/fandhe-frontend/reference/api/""#));
        assert!(!html_guide.contains(">Reference<"));

        let html_reference = render(&sidebar(&nav, "/reference/api/"));
        assert!(!html_reference.contains(r#"href="/fandhe-frontend/guide/intro/""#));
        assert!(!html_reference.contains(r#"href="/fandhe-frontend/guide/getting-started/""#));
        assert!(!html_reference.contains(">Guide<"));
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

    #[test]
    fn prev_next_nav_cards_show_direction_title_and_section() {
        let nav = parse_nav(SAMPLE).unwrap();
        let html = render(&prev_next_nav(&nav, "/guide/getting-started/"));
        assert!(html.contains(r#"aria-label="前後のページ""#));
        assert!(html.contains("前へ") && html.contains("次へ"));
        assert!(html.contains("Introduction") && html.contains("API"));
        assert!(html.contains(">Guide<") && html.contains(">Reference<"));
        assert!(html.find("前へ").unwrap() < html.find("次へ").unwrap());
    }

    #[test]
    fn prev_next_nav_has_one_anchor_per_side_and_no_nested_anchor() {
        let nav = parse_nav(SAMPLE).unwrap();
        let count = |p: &str| render(&prev_next_nav(&nav, p)).matches("<a ").count();
        assert_eq!(count("/guide/intro/"), 1);
        assert_eq!(count("/guide/getting-started/"), 2);
        assert_eq!(count("/reference/api/"), 1);
    }

    #[test]
    fn prev_next_nav_icons_are_decorative_and_positioned_by_side() {
        let nav = parse_nav(SAMPLE).unwrap();
        let html = render(&prev_next_nav(&nav, "/guide/getting-started/"));
        assert_eq!(html.matches(r#"data-scope="icon""#).count(), 2);
        assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 2);
        let prev_svg = html.find("<svg").unwrap();
        let prev_meta = html.find("docs-pager-meta").unwrap();
        assert!(prev_svg < prev_meta, "prev は矢印が先頭");
        let next_meta = html.rfind("docs-pager-meta").unwrap();
        let next_svg = html.rfind("<svg").unwrap();
        assert!(next_meta < next_svg, "next は矢印が末尾");
    }

    #[test]
    fn prev_next_nav_escapes_page_and_section_titles() {
        let toml = SAMPLE
            .replace("Introduction", "<img src=x onerror=alert(1)>")
            .replace(r#"title = "Guide""#, r#"title = "<b>G</b>""#);
        let nav = parse_nav(&toml).unwrap();
        let html = render(&prev_next_nav(&nav, "/guide/getting-started/"));
        assert!(!html.contains("<img"));
        assert!(!html.contains("<b>"));
        assert!(html.contains("&lt;img"));
    }

    // ---- ヘッダーナビと Assets メガメニュー（イシュー #908 / #3701） ----

    /// 単独 → メニュー → 単独 の並びになるフィクスチャ。メニュー項目の宣言順は
    /// セクション宣言順と逆（B → A）にして、カード順が `[[menu.item]]` 順であることを固定する。
    const SAMPLE_WITH_MENU: &str = r#"
[site]
title = "Docs"
base_path = "/base"

[[section]]
title = "Guides"
index_path = "/guides/"

[[section.page]]
title = "Guides Index"
source = "g.md"
path = "/guides/"

[[section.page]]
title = "Intro"
source = "gi.md"
path = "/guides/intro/"

[[section]]
title = "Alpha"
index_path = "/alpha/"

[[section.page]]
title = "Alpha Index"
source = "a.md"
path = "/alpha/"

[[section.group]]
title = "Hidden Group"

[[section.group.page]]
title = "Alpha Button"
source = "ab.md"
path = "/alpha/button/"

[[section]]
title = "Beta"
index_path = "/beta/"

[[section.page]]
title = "Beta Index"
source = "b.md"
path = "/beta/"

[[section]]
title = "Api"
index_path = "/api/"

[[section.page]]
title = "Api Index"
source = "api.md"
path = "/api/"

[[menu]]
title = "Assets"
index_path = "/assets/"
source = "assets.md"

[[menu.item]]
section = "/beta/"
description = "Beta description"

[[menu.item]]
section = "/alpha/"
description = "Alpha description"
"#;

    fn trigger_count_with_current(html: &str) -> usize {
        html.matches(r#"class="docs-header-trigger" aria-current="true""#)
            .count()
    }

    #[test]
    fn header_nav_orders_entries_and_cards_by_declaration() {
        let nav = parse_nav(SAMPLE_WITH_MENU).unwrap();
        let html = render(&header_nav(&nav, "/guides/"));
        assert!(html.starts_with("<nav"));
        assert!(html.contains(r#"aria-label="Site sections""#));
        assert_eq!(html.matches("docs-header-trigger").count(), 3);
        let guides = html.find(r#"href="/base/guides/""#).unwrap();
        let assets = html.find(r#"href="/base/assets/""#).unwrap();
        let api = html.rfind(r#"href="/base/api/""#).unwrap();
        assert!(guides < assets && assets < api);
        // カードは [[menu.item]] の順（Beta → Alpha）で、href はメンバーの index_path。
        let beta = html.find(r#"class="docs-header-mega-card""#).unwrap();
        let beta_href = html.find(r#"href="/base/beta/""#).unwrap();
        let alpha_href = html.find(r#"href="/base/alpha/""#).unwrap();
        assert!(beta_href < alpha_href && beta > assets - 200);
        assert_eq!(html.matches("docs-header-mega-card").count(), 2);
        assert!(html.contains("Beta description"));
        assert!(html.contains("Alpha description"));
    }

    #[test]
    fn header_nav_has_no_popup_or_page_headings() {
        let nav = parse_nav(SAMPLE_WITH_MENU).unwrap();
        let html = render(&header_nav(&nav, "/guides/"));
        assert!(!html.contains("docs-header-dropdown"));
        for absent in ["Intro", "Hidden Group", "Alpha Button", "Guides Index"] {
            assert!(!html.contains(absent), "{absent} must not appear");
        }
        // 単独セクションの li にはパネルを含まない。
        assert_eq!(html.matches("docs-header-mega\"").count(), 1);
    }

    #[test]
    fn header_nav_marks_current_for_member_page() {
        let nav = parse_nav(SAMPLE_WITH_MENU).unwrap();
        let html = render(&header_nav(&nav, "/alpha/button/"));
        assert_eq!(html.matches(r#"aria-current="true""#).count(), 2);
        assert_eq!(html.matches("data-current").count(), 2);
        assert!(html
            .contains(r#"href="/base/assets/" class="docs-header-trigger" aria-current="true""#));
        assert!(html
            .contains(r#"href="/base/alpha/" class="docs-header-mega-card" aria-current="true""#));
        assert!(!html.contains(r#"aria-current="page""#));
    }

    #[test]
    fn header_nav_marks_only_trigger_on_menu_index_page() {
        let nav = parse_nav(SAMPLE_WITH_MENU).unwrap();
        let html = render(&header_nav(&nav, "/assets/"));
        assert_eq!(html.matches(r#"aria-current="true""#).count(), 1);
        assert_eq!(trigger_count_with_current(&html), 1);
        assert!(!html.contains(r#"aria-current="page""#));
    }

    #[test]
    fn header_nav_marks_standalone_section_and_unknown_path() {
        let nav = parse_nav(SAMPLE_WITH_MENU).unwrap();
        let html = render(&header_nav(&nav, "/guides/intro/"));
        assert_eq!(html.matches(r#"aria-current="true""#).count(), 1);
        assert!(html
            .contains(r#"href="/base/guides/" class="docs-header-trigger" aria-current="true""#));
        let html = render(&header_nav(&nav, "/unknown/"));
        assert_eq!(html.matches("aria-current").count(), 0);
    }

    #[test]
    fn header_nav_has_no_role_dynamic_aria_or_id() {
        let nav = parse_nav(SAMPLE_WITH_MENU).unwrap();
        let html = render(&header_nav(&nav, "/alpha/button/"));
        assert!(!html.contains("<button"));
        for forbidden in [
            "role=",
            "aria-expanded",
            "aria-haspopup",
            "aria-controls",
            " id=",
        ] {
            assert!(!html.contains(forbidden), "{forbidden} must not appear");
        }
    }

    #[test]
    fn header_nav_without_menus_lists_every_section_trigger_only() {
        let nav = parse_nav(SAMPLE).unwrap();
        let html = render(&header_nav(&nav, "/guide/getting-started/"));
        assert!(!html.contains("docs-header-mega\""));
        assert!(!html.contains("docs-header-dropdown"));
        let guide = html.find("Guide").unwrap();
        let reference = html.find("Reference").unwrap();
        assert!(guide < reference);
        assert!(html.contains(r#"href="/fandhe-frontend/guide/intro/""#));
        assert!(html.contains(r#"href="/fandhe-frontend/reference/api/""#));
        assert_eq!(html.matches(r#"aria-current="true""#).count(), 1);
    }

    #[test]
    fn header_nav_escapes_titles_and_descriptions() {
        let toml = SAMPLE_WITH_MENU
            .replace(r#"title = "Guides""#, r#"title = "<script>g</script>""#)
            .replace(r#"title = "Assets""#, r#"title = "A&B""#)
            .replace(r#"title = "Beta""#, r#"title = "Be\"ta""#)
            .replace("Beta description", "<img src=x onerror=1> & more");
        let nav = parse_nav(&toml).unwrap();
        let html = render(&header_nav(&nav, "/guides/"));
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;script&gt;g&lt;/script&gt;"));
        assert!(html.contains("A&amp;B"));
        assert!(html.contains("Be&quot;ta"));
        assert!(html.contains("&lt;img src=x onerror=1&gt; &amp; more"));
    }

    // ---- ナビ drawer（イシュー #3674） ----

    const DRAWER_NAV: &str = r#"
[site]
title = "Docs"
base_path = "/base"

[[section]]
title = "Guides"
index_path = "/guides/"

[[section.page]]
title = "Guides Index"
source = "g.md"
path = "/guides/"

[[section]]
title = "Themes"
index_path = "/themes/"

[[section.page]]
title = "Themes Index"
source = "t.md"
path = "/themes/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "b.md"
path = "/themes/button/"
"#;

    #[test]
    fn nav_drawer_lists_every_section_in_declaration_order_with_index_hrefs() {
        let nav = parse_nav(DRAWER_NAV).unwrap();
        let html = render(&nav_drawer(&nav, "/guides/"));
        let guides = html.find(r#"href="/base/guides/""#).expect("guides link");
        let themes = html.find(r#"href="/base/themes/""#).expect("themes link");
        assert!(guides < themes);
        assert_eq!(html.matches("docs-nav-drawer-section-link").count(), 2);
        assert!(html.contains(r#"aria-label="Site navigation""#));
    }

    #[test]
    fn nav_drawer_opens_and_marks_only_the_current_section() {
        let nav = parse_nav(DRAWER_NAV).unwrap();
        let html = render(&nav_drawer(&nav, "/themes/button/"));
        // 現在セクションの details 1 件 + 現在ページを含むグループ 1 件のみ open。
        assert_eq!(html.matches(r#" open="""#).count(), 2);
        // 現在セクションのリンクだけが所属を示す（"true"）。
        assert_eq!(html.matches(r#"aria-current="true""#).count(), 1);
        let themes_link = html.find(r#"href="/base/themes/""#).unwrap();
        let current = html.find(r#"aria-current="true""#).unwrap();
        assert!(current.abs_diff(themes_link) < 120);
    }

    #[test]
    fn nav_drawer_current_section_is_sidebar_form_and_others_are_heading_lists() {
        let nav = parse_nav(DRAWER_NAV).unwrap();
        let html = render(&nav_drawer(&nav, "/themes/button/"));
        // 現在セクション: グループ details と配下ページ、現在ページの aria-current。
        assert!(html.contains(r#"href="/base/themes/button/""#));
        assert!(html.contains(r#"aria-current="page""#));
        assert!(html.contains("docs-nav-group-summary"));
        // 現在でないセクション: 見出し一覧（popup 形式）のみ。
        assert_eq!(html.matches("docs-nav-drawer-list").count(), 1);
        assert_eq!(html.matches(r#"aria-current="true""#).count(), 1);
    }

    #[test]
    fn nav_drawer_other_section_groups_are_anchor_links_without_group_pages() {
        let nav = parse_nav(DRAWER_NAV).unwrap();
        let html = render(&nav_drawer(&nav, "/guides/"));
        assert!(html.contains(r#"href="/base/themes/#forms""#));
        assert!(!html.contains(r#"href="/base/themes/button/""#));
    }

    #[test]
    fn nav_drawer_unregistered_path_closes_every_section() {
        let nav = parse_nav(DRAWER_NAV).unwrap();
        let html = render(&nav_drawer(&nav, "/404.html"));
        assert!(!html.contains(r#" open="""#));
        assert!(!html.contains("aria-current"));
        assert_eq!(html.matches("docs-nav-drawer-list").count(), 2);
    }

    #[test]
    fn nav_drawer_has_no_role_dynamic_aria_or_ids() {
        let nav = parse_nav(DRAWER_NAV).unwrap();
        for path in ["/guides/", "/themes/button/", "/404.html"] {
            let html = render(&nav_drawer(&nav, path));
            for forbidden in ["role=", "aria-expanded", "aria-haspopup", " id="] {
                assert!(!html.contains(forbidden), "{path}: found {forbidden}");
            }
        }
    }

    #[test]
    fn nav_drawer_escapes_section_and_page_titles() {
        let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "<script>alert(1)</script>\"S"
index_path = "/p1/"

[[section.page]]
title = "Quote\"Title"
source = "p1.md"
path = "/p1/"
"#;
        let nav = parse_nav(input).unwrap();
        for path in ["/p1/", "/other/"] {
            let html = render(&nav_drawer(&nav, path));
            assert!(!html.contains("<script>"));
            assert!(html.contains("&lt;script&gt;"));
            assert!(html.contains("Quote&quot;Title"));
            assert!(!html.contains("javascript:"));
        }
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

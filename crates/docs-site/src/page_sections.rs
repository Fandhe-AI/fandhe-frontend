//! 任意ページへ Rust 生成節を差し込む汎用フック（イシュー #3598）。
//!
//! # 役割・呼び出し文脈
//!
//! docs サイトの Markdown レンダラ（[`crate::markdown`]）は生 HTML を構文として
//! 解釈しないため、pre-styled-ui 部品で組んだ節（ヒーロー・カードグリッド等）を
//! トップページやセクション索引へ置くには Rust 側から `Node` を差し込むしかない。
//! 既存の Rust 生成コンテンツ経路は特定のページ種別に固定されている
//! （[`crate::component_page`]: 部品ページ / [`crate::blocks`]: block ページと
//! `/blocks/` 索引 / [`crate::wireframes`]: wireframe ページ）。本モジュールは
//! それらと同型の第 5 の経路として、**ページパスを鍵とする登録表**
//! （[`PAGE_SECTIONS`]）から生成関数を引く汎用フックを提供する。
//!
//! [`crate::build::build_site_with`] が `render_markdown` → blocks → wireframes
//! の挿入の**直後**に [`insert_generated_sections_with`] を呼ぶ。本番登録表は
//! トップのヒーロー（#3612、[`crate::landing`]）のみで、Phase 4（索引の
//! カードグリッド化）が各ページの登録を追加する。空の登録表
//! （[`EMPTY_REGISTRY`]）では全ページの出力が変更前と 1 バイトも変わらない
//! （`tests/page_sections.rs` が固定する）。
//!
//! # 挿入位置の規約（[`Placement`]）
//!
//! - [`Placement::Prepend`]: Markdown 本文の先頭（H1 より前）。トップページが
//!   ヒーローを最上段に置くための位置。フックは H1 の除去・統合を行わない
//! - [`Placement::BeforeFirstH2`]: 最初の `h2` の直前。`h2` が無いページでは
//!   末尾へ追加する（[`crate::blocks`] / [`crate::wireframes`] と同じ規約）
//! - [`Placement::Append`]: Markdown 本文の末尾。前後ナビより前
//!
//! 「特定見出しの節を置換する」位置は採用しない。Markdown の見出し文言と
//! Rust 側が文字列で結合し、見出しを改稿しただけで黙って壊れるためである。
//! 置換が必要な場合は原稿側から該当箇所を削除したうえで上記のいずれかで
//! 差し込む。
//!
//! # アセット（CSS）
//!
//! 生成節が必要とする追加 CSS は [`PAGE_STYLESHEETS`] へ登録し、
//! [`PageSection::stylesheets`] から `rel_path` で参照する。使われている
//! ページにだけ `<link>` を配線し、linkcheck の既知 href 登録・書き出しは
//! [`crate::build`] が行う（書き出し前に組み立てる fail-closed の処理順）。
//!
//! # 検索インデックス・id・リンクの規約
//!
//! - 生成節は `rewrite_md_links` より前に差し込まれるため、検索インデックスの
//!   本文へ自動的に含まれる。ただし `data-scope` を持つ部品 root 配下の
//!   テキスト・見出しは TOC と検索テキストの両方から除外される
//!   （[`crate::layout`] の既存規則）。検索に載せたい見出し・文は `docs-*`
//!   ラッパー側（`data-scope` の外）に置く
//! - 生成節の `h2`/`h3` へ固定 id を付けない（`with_heading_anchors` が
//!   [`crate::layout::RESERVED_LAYOUT_IDS`] を予約したうえで採番する）。
//!   見出し以外の要素の id が予約 id と衝突する登録は [`validate`] が拒否する
//! - 生成節の href は [`crate::layout::asset_href`] による絶対パスに限る
//!   （`.md` 相対リンクは書かない）。最終ページは linkcheck が走査する
//! - 生成節が `alert` 部品を含むと admonition CSS も配線される（無害）
//!
//! # セキュリティ上の不変条件
//!
//! 生成節は `fandhe_frontend_core` のノード木 API だけで組む。本モジュールは
//! `Node` 列を差し込むだけで、文字列を HTML として扱う経路・`raw_html()` を
//! 新設しない（REQ-1）。登録表の不整合は [`validate`] がビルド時に fail-closed
//! で拒否する。[`PageSectionError`] の `Display` は登録パスと `rel_path` のみを
//! 含み、絶対パス等の内部情報を出さない。

use std::fmt;

use fandhe_frontend_core::Node;
use fandhe_frontend_pre_styled_ui::{StyleSheet, StylesheetError};

use crate::blocks;
use crate::build::RESERVED_ASSET_NAMES;
use crate::component_page;
use crate::landing;
use crate::layout::{PageLayout, RESERVED_LAYOUT_IDS};
use crate::nav::Nav;
use crate::wireframes;

/// 生成節を Markdown 本文のどこへ差し込むか（モジュール doc「挿入位置の規約」）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Markdown 本文の先頭（H1 より前）。
    Prepend,
    /// 最初の `h2` の直前（`h2` が無ければ末尾）。
    BeforeFirstH2,
    /// Markdown 本文の末尾（前後ナビより前）。
    Append,
}

/// 生成節が必要とする追加 CSS 1 本分。`rel_path` は `assets/<basename>.css`。
#[derive(Clone, Copy)]
pub struct PageStylesheet {
    /// `out_dir` 起点の出力先。`assets/<basename>.css` 形式のみ許可する。
    pub rel_path: &'static str,
    /// CSS 本体の組み立て（書き出し前に呼ばれ、失敗はビルド失敗になる）。
    pub build: fn() -> Result<StyleSheet, StylesheetError>,
}

/// 1 ページ分の生成節登録。
#[derive(Clone, Copy)]
pub struct PageSection {
    /// `site/nav.toml` の `page.path` と完全一致するページパス。
    pub path: &'static str,
    /// 差し込み位置。
    pub placement: Placement,
    /// 生成関数。引数は `nav.site.base_path`（href 生成用）。
    pub render: fn(base_path: &str) -> Vec<Node>,
    /// 配線する追加 CSS（[`Registry::stylesheets`] の `rel_path` を参照）。
    pub stylesheets: &'static [&'static str],
    /// ページ骨格の種別（イシュー #3612）。トップのランディングだけが
    /// [`PageLayout::Landing`]、他は [`PageLayout::Docs`]。
    pub layout: PageLayout,
}

/// 登録表一式。本番は [`REGISTRY`]、テストは合成エントリで構築する。
#[derive(Clone, Copy)]
pub struct Registry {
    /// ページ別の生成節。
    pub sections: &'static [PageSection],
    /// 追加 CSS の実体。
    pub stylesheets: &'static [PageStylesheet],
}

/// 本番の生成節登録表。トップのヒーロー（イシュー #3612）が 1 件。後続の
/// #3613〜#3615 は `/` の登録を増やさず [`landing::render`] の返す節列へ追記する
/// （[`validate`] の `DuplicatePath` 制約）。Phase 4 の索引カードは各索引パスを追加する。
pub const PAGE_SECTIONS: &[PageSection] = &[PageSection {
    path: landing::PATH,
    placement: Placement::Prepend,
    render: landing::render,
    stylesheets: &[],
    layout: PageLayout::Landing,
}];

/// 本番の追加 CSS 登録表（基盤導入時点では空）。
pub const PAGE_STYLESHEETS: &[PageStylesheet] = &[];

/// 本番の登録表。[`crate::build::build_site`] が使う。
pub const REGISTRY: Registry = Registry {
    sections: PAGE_SECTIONS,
    stylesheets: PAGE_STYLESHEETS,
};

/// 生成節を一切持たない登録表。フィクスチャ（一時ディレクトリで組む合成サイト）
/// のビルドが、本サイト専用のヒーロー・ランディング骨格・CTA リンクを
/// 引き込まないために使う（`build_site_with(.., &EMPTY_REGISTRY)`）。
pub const EMPTY_REGISTRY: Registry = Registry {
    sections: &[],
    stylesheets: &[],
};

/// [`validate`] の失敗理由。`Display` は登録パス・`rel_path`・id のみを含む。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageSectionError {
    /// 同じ `path` が複数回登録されている。
    DuplicatePath(String),
    /// `path` が nav の実在ページに含まれない。
    UnknownPage(String),
    /// `path` が block / wireframe / component_page の既存生成経路と重なる。
    ConflictsWithGeneratedPage(String),
    /// `stylesheets` が登録表に無い `rel_path` を参照している。
    UnknownStylesheet {
        /// 参照元ページパス。
        path: String,
        /// 未登録の `rel_path`。
        rel_path: String,
    },
    /// `rel_path` が `assets/<basename>.css` 形式でない。
    InvalidStylesheetPath(String),
    /// `rel_path` が重複している。
    DuplicateStylesheet(String),
    /// `rel_path` の basename がビルド時生成アセットの予約名と一致する。
    ReservedStylesheetName(String),
    /// 生成節の非見出し要素の `id` が予約 id と衝突する。
    ReservedId {
        /// 登録ページパス。
        path: String,
        /// 衝突した id。
        id: String,
    },
}

impl fmt::Display for PageSectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicatePath(p) => write!(f, "page section registered twice for {p}"),
            Self::UnknownPage(p) => write!(f, "page section path is not a nav page: {p}"),
            Self::ConflictsWithGeneratedPage(p) => write!(
                f,
                "page section path overlaps an existing generated-content page: {p}"
            ),
            Self::UnknownStylesheet { path, rel_path } => write!(
                f,
                "page section {path} references unregistered stylesheet {rel_path}"
            ),
            Self::InvalidStylesheetPath(r) => {
                write!(f, "page stylesheet path must be assets/<name>.css: {r}")
            }
            Self::DuplicateStylesheet(r) => write!(f, "page stylesheet registered twice: {r}"),
            Self::ReservedStylesheetName(r) => {
                write!(
                    f,
                    "page stylesheet name is reserved for a built-in asset: {r}"
                )
            }
            Self::ReservedId { path, id } => {
                write!(f, "page section {path} emits reserved layout id \"{id}\"")
            }
        }
    }
}

impl std::error::Error for PageSectionError {}

/// `path` に登録された生成節を返す（無ければ `None`）。
#[must_use]
pub fn section_for_path_in<'a>(registry: &'a Registry, path: &str) -> Option<&'a PageSection> {
    registry.sections.iter().find(|s| s.path == path)
}

/// `path` のページ骨格種別を返す。未登録ページは [`PageLayout::Docs`]。
#[must_use]
pub fn layout_for_path_in(registry: &Registry, path: &str) -> PageLayout {
    section_for_path_in(registry, path).map_or(PageLayout::Docs, |s| s.layout)
}

/// `path` の生成節が配線する追加 CSS を、`rel_path` の重複なしで返す。
/// 未登録の参照は [`validate`] が事前に拒否するため、ここでは読み飛ばす。
#[must_use]
pub fn stylesheets_for_path_in<'a>(registry: &'a Registry, path: &str) -> Vec<&'a PageStylesheet> {
    let mut out: Vec<&PageStylesheet> = Vec::new();
    if let Some(section) = section_for_path_in(registry, path) {
        for rel in section.stylesheets {
            if let Some(sheet) = registry.stylesheets.iter().find(|s| s.rel_path == *rel) {
                if !out.iter().any(|s| s.rel_path == sheet.rel_path) {
                    out.push(sheet);
                }
            }
        }
    }
    out
}

/// 本番登録表 [`REGISTRY`] 用の [`insert_generated_sections_with`]。
#[must_use]
pub fn insert_generated_sections(path: &str, base_path: &str, blocks: Vec<Node>) -> Vec<Node> {
    insert_generated_sections_with(&REGISTRY, path, base_path, blocks)
}

/// `path` が `registry` に登録されていれば、生成節を [`Placement`] どおりに
/// Markdown ブロック列へ差し込む。未登録ページでは `blocks` をそのまま返す
/// （no-op。AC: 空登録表で出力がバイト一致）。
#[must_use]
pub fn insert_generated_sections_with(
    registry: &Registry,
    path: &str,
    base_path: &str,
    mut blocks: Vec<Node>,
) -> Vec<Node> {
    let Some(section) = section_for_path_in(registry, path) else {
        return blocks;
    };
    let generated = (section.render)(base_path);
    match section.placement {
        Placement::Prepend => {
            let mut out = generated;
            out.append(&mut blocks);
            out
        }
        Placement::Append => {
            blocks.extend(generated);
            blocks
        }
        Placement::BeforeFirstH2 => {
            let at = blocks
                .iter()
                .position(|n| matches!(n, Node::Element { tag, .. } if *tag == "h2"))
                .unwrap_or(blocks.len());
            let tail = blocks.split_off(at);
            blocks.extend(generated);
            blocks.extend(tail);
            blocks
        }
    }
}

/// 登録表の整合性を検証する（[`crate::build::build_site_with`] が書き出し前に
/// 呼ぶ fail-closed 検証）。検査内容はモジュール doc と [`PageSectionError`]
/// の各バリアントに対応する。`/blocks/` 索引だけは既存の索引節との合成を
/// 許可する（既存節の後に本フックが適用される）。
///
/// # Errors
///
/// 最初に見つかった不整合を [`PageSectionError`] で返す。
pub fn validate(registry: &Registry, nav: &Nav) -> Result<(), PageSectionError> {
    for (i, sheet) in registry.stylesheets.iter().enumerate() {
        let name = sheet
            .rel_path
            .strip_prefix("assets/")
            .filter(|n| {
                n.ends_with(".css")
                    && n.len() > ".css".len()
                    && !n.starts_with('.')
                    && n.chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
            })
            .ok_or_else(|| PageSectionError::InvalidStylesheetPath(sheet.rel_path.to_string()))?;
        if RESERVED_ASSET_NAMES.contains(&name) {
            return Err(PageSectionError::ReservedStylesheetName(
                sheet.rel_path.to_string(),
            ));
        }
        if registry.stylesheets[..i]
            .iter()
            .any(|s| s.rel_path == sheet.rel_path)
        {
            return Err(PageSectionError::DuplicateStylesheet(
                sheet.rel_path.to_string(),
            ));
        }
    }
    for (i, section) in registry.sections.iter().enumerate() {
        let path = section.path;
        if registry.sections[..i].iter().any(|s| s.path == path) {
            return Err(PageSectionError::DuplicatePath(path.to_string()));
        }
        if !nav.all_pages().any(|p| p.path == path) {
            return Err(PageSectionError::UnknownPage(path.to_string()));
        }
        if blocks::block_for_path(path).is_some()
            || wireframes::wireframe_for_path(path).is_some()
            || component_page::generated_content(path).is_some()
        {
            return Err(PageSectionError::ConflictsWithGeneratedPage(
                path.to_string(),
            ));
        }
        for rel in section.stylesheets {
            if !registry.stylesheets.iter().any(|s| s.rel_path == *rel) {
                return Err(PageSectionError::UnknownStylesheet {
                    path: path.to_string(),
                    rel_path: (*rel).to_string(),
                });
            }
        }
        let nodes = (section.render)(&nav.site.base_path);
        if let Some(id) = find_reserved_id(&nodes) {
            return Err(PageSectionError::ReservedId {
                path: path.to_string(),
                id,
            });
        }
    }
    Ok(())
}

/// ノード木を走査し、予約レイアウト id と一致する `id` 属性値を返す。
fn find_reserved_id(nodes: &[Node]) -> Option<String> {
    for node in nodes {
        if let Node::Element {
            attrs, children, ..
        } = node
        {
            for (name, value) in attrs {
                if name == "id" && RESERVED_LAYOUT_IDS.contains(&value.as_str()) {
                    return Some(value.clone());
                }
            }
            if let Some(id) = find_reserved_id(children) {
                return Some(id);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{div, el, h1, h2, p, text};

    fn base() -> Vec<Node> {
        vec![
            h1(vec![], vec![text("T")]),
            p(vec![], vec![text("lead")]),
            h2(vec![], vec![text("A")]),
            p(vec![], vec![text("body")]),
        ]
    }

    fn marker(_: &str) -> Vec<Node> {
        vec![div(vec![("class", "docs-gen")], vec![])]
    }

    fn reserved(_: &str) -> Vec<Node> {
        vec![div(
            vec![],
            vec![el("span", vec![("id", "docs-search-input")], vec![])],
        )]
    }

    fn css() -> Result<StyleSheet, StylesheetError> {
        Ok(StyleSheet::new())
    }

    static OK_SHEETS: [PageStylesheet; 1] = [PageStylesheet {
        rel_path: "assets/x.css",
        build: css,
    }];

    fn reg(sections: &'static [PageSection], sheets: &'static [PageStylesheet]) -> Registry {
        Registry {
            sections,
            stylesheets: sheets,
        }
    }

    fn one(placement: Placement) -> Registry {
        let s: &'static [PageSection] = Box::leak(Box::new([PageSection {
            path: "/",
            placement,
            render: marker,
            stylesheets: &[],
            layout: PageLayout::Docs,
        }]));
        reg(s, &[])
    }

    fn is_gen(n: &Node) -> bool {
        matches!(n, Node::Element { attrs, .. } if attrs.iter().any(|(k, v)| k == "class" && v == "docs-gen"))
    }

    #[test]
    fn production_registry_has_only_landing_hero() {
        assert!(PAGE_STYLESHEETS.is_empty());
        let paths: Vec<_> = PAGE_SECTIONS.iter().map(|s| s.path).collect();
        assert_eq!(paths, ["/"]);
        assert_eq!(layout_for_path_in(&REGISTRY, "/"), PageLayout::Landing);
        assert_eq!(layout_for_path_in(&REGISTRY, "/guides/"), PageLayout::Docs);
        assert_eq!(layout_for_path_in(&EMPTY_REGISTRY, "/"), PageLayout::Docs);
    }

    #[test]
    fn unregistered_path_is_noop() {
        let out = insert_generated_sections_with(&one(Placement::Prepend), "/other/", "", base());
        assert_eq!(out, base());
    }

    #[test]
    fn placement_prepend_before_h1() {
        let out = insert_generated_sections_with(&one(Placement::Prepend), "/", "", base());
        assert!(is_gen(&out[0]));
        assert_eq!(out.len(), 5);
    }

    #[test]
    fn placement_before_first_h2() {
        let out = insert_generated_sections_with(&one(Placement::BeforeFirstH2), "/", "", base());
        assert!(is_gen(&out[2]));
        assert!(matches!(&out[3], Node::Element { tag, .. } if *tag == "h2"));
    }

    #[test]
    fn placement_before_first_h2_falls_back_to_end() {
        let blocks = vec![h1(vec![], vec![text("T")])];
        let out = insert_generated_sections_with(&one(Placement::BeforeFirstH2), "/", "", blocks);
        assert!(is_gen(out.last().unwrap()));
    }

    #[test]
    fn placement_append_at_end() {
        let out = insert_generated_sections_with(&one(Placement::Append), "/", "", base());
        assert!(is_gen(out.last().unwrap()));
        assert_eq!(out.len(), 5);
    }

    fn nav() -> Nav {
        crate::nav::parse_nav(
            "[site]\ntitle = \"T\"\nbase_path = \"/b\"\n\n[[section]]\ntitle = \"G\"\nindex_path = \"/\"\n\n[[section.page]]\ntitle = \"H\"\nsource = \"site/index.md\"\npath = \"/\"\n",
        )
        .expect("valid nav")
    }

    fn sec(
        path: &'static str,
        render: fn(&str) -> Vec<Node>,
        stylesheets: &'static [&'static str],
    ) -> PageSection {
        PageSection {
            path,
            placement: Placement::Append,
            render,
            stylesheets,
            layout: PageLayout::Docs,
        }
    }

    fn leak<T>(v: Vec<T>) -> &'static [T] {
        Box::leak(v.into_boxed_slice())
    }

    #[test]
    fn validate_accepts_good_registry() {
        let r = reg(leak(vec![sec("/", marker, &["assets/x.css"])]), &OK_SHEETS);
        assert_eq!(validate(&r, &nav()), Ok(()));
        assert_eq!(stylesheets_for_path_in(&r, "/").len(), 1);
        assert!(stylesheets_for_path_in(&r, "/zzz/").is_empty());
    }

    #[test]
    fn validate_rejects_duplicate_and_unknown_path() {
        let dup = reg(
            leak(vec![sec("/", marker, &[]), sec("/", marker, &[])]),
            &[],
        );
        assert_eq!(
            validate(&dup, &nav()),
            Err(PageSectionError::DuplicatePath("/".into()))
        );
        let unk = reg(leak(vec![sec("/nope/", marker, &[])]), &[]);
        assert_eq!(
            validate(&unk, &nav()),
            Err(PageSectionError::UnknownPage("/nope/".into()))
        );
    }

    #[test]
    fn validate_rejects_unknown_stylesheet_reference() {
        let r = reg(leak(vec![sec("/", marker, &["assets/none.css"])]), &[]);
        assert!(matches!(
            validate(&r, &nav()),
            Err(PageSectionError::UnknownStylesheet { .. })
        ));
    }

    #[test]
    fn validate_rejects_bad_stylesheet_paths() {
        for bad in [
            "../x.css",
            "assets/a/b.css",
            "assets/x.txt",
            "x.css",
            "assets/.css",
        ] {
            let sheets = leak(vec![PageStylesheet {
                rel_path: bad,
                build: css,
            }]);
            assert_eq!(
                validate(&reg(&[], sheets), &nav()),
                Err(PageSectionError::InvalidStylesheetPath(bad.into())),
                "{bad}"
            );
        }
        let reserved = leak(vec![PageStylesheet {
            rel_path: "assets/site.css",
            build: css,
        }]);
        assert!(matches!(
            validate(&reg(&[], reserved), &nav()),
            Err(PageSectionError::ReservedStylesheetName(_))
        ));
        let dup = leak(vec![OK_SHEETS[0], OK_SHEETS[0]]);
        assert!(matches!(
            validate(&reg(&[], dup), &nav()),
            Err(PageSectionError::DuplicateStylesheet(_))
        ));
    }

    #[test]
    fn validate_rejects_reserved_id() {
        let r = reg(leak(vec![sec("/", reserved, &[])]), &[]);
        assert!(matches!(
            validate(&r, &nav()),
            Err(PageSectionError::ReservedId { .. })
        ));
    }

    #[test]
    fn validate_rejects_existing_generated_pages() {
        let real = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../site/nav.toml"),
        )
        .expect("read site nav");
        let real = crate::nav::parse_nav(&real).expect("parse site nav");
        let block = blocks::all_blocks()
            .into_iter()
            .next()
            .expect("a block")
            .path;
        let themes = real
            .all_pages()
            .map(|p| p.path.clone())
            .find(|p| p.starts_with("/themes/") && component_page::generated_content(p).is_some())
            .expect("a themes component page");
        let themes: &'static str = Box::leak(themes.into_boxed_str());
        for path in [block, themes] {
            let r = reg(leak(vec![sec(path, marker, &[])]), &[]);
            assert_eq!(
                validate(&r, &real),
                Err(PageSectionError::ConflictsWithGeneratedPage(path.into()))
            );
        }
    }
}

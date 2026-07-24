//! docs サイトのビルドパイプライン本体（イシュー #470）。
//!
//! # 呼び出し文脈
//!
//! [`crate::main`]（バイナリ本体、引数パース・終了コード変換のみを担う薄い
//! ラッパー）と `tests/site_build.rs`（E2E テスト）の双方から [`build_site`]
//! を直接呼ぶ。bin/lib 両方から同一のビルドロジックを共有するために本モジュール
//! を `lib.rs` 側に置く（`crates/docs-site/CLAUDE.md` の crate 構成注記どおり）。
//!
//! # 処理順（fail-closed）
//!
//! 1. `<repo_root>/site/nav.toml` を [`nav::parse_nav`] → [`nav::validate_sources`]
//! 2. 各ページの Markdown を [`markdown::render_markdown`] → [`linkcheck::rewrite_md_links`]
//!    （`.md` リンクをサイト内パスへ書き換え）→ [`layout::docs_page`] で文書化
//! 3. [`linkcheck::check_links`] で全ページの内部リンクを突合検証し、1 件でも
//!    壊れていれば **書き出しより前に** [`BuildError::LinkCheck`] で失敗させる
//!    （「一部だけ更新された dist/」を残さない。`ssg::generate_pages` 自体も
//!    同じ fail-closed 方針だが、linkcheck はそれより手前の層で同じ方針を守る）
//! 4. [`fandhe_frontend_server::ssg::generate_pages`] で `out_dir` へ書き出す
//! 5. `<repo_root>/site/assets/` 配下を `<out_dir>/assets/` へコピーする
//!    （通常ファイルのみ許可。シンボリックリンク・ディレクトリ以外の特殊
//!    エントリはエラーにする fail-closed。リポジトリ外ファイルの持ち出し
//!    防止のため走査対象を固定ディレクトリに限定する）。`site/assets/` が
//!    存在しない場合はアセット 0 件として許容する（イシュー #905 でサイト
//!    骨格 CSS がビルド時生成へ切り替わり、`site/assets/` に静的ファイルを
//!    置く必然性が無くなったため。他の I/O エラーは従来どおり `BuildError::Io`）
//!
//! # サイト骨格 CSS（[`crate::site_theme`]、イシュー #905）
//!
//! `assets/site.css`（サイト骨格スタイル）は `site/assets/` の静的コピーでは
//! なく、[`crate::site_theme::stylesheet`] がビルド時生成する（[`crate::skip_nav`]
//! と同じ「全ビルド無条件」区分）。生成することが確定しているため、
//! `site/assets/` 配下に同名ファイルが実在する場合は静的ファイルの黙った
//! 上書き・生成物のすり替わりを防ぐため [`BuildError::ReservedAssetName`] で
//! 書き出し前にエラーにする（[`RESERVED_ASSET_NAMES`] 参照）。
//!
//! # Rust 生成コンテンツページ（[`crate::showcase`]）
//!
//! Markdown では表現できない「pre-styled-ui コンポーネントの実レンダリング」
//! を掲載するページ（UI ショーケース）のため、ステップ 2 で
//! [`showcase::generated_content`] を `page.path` で照会し、`Some` の場合のみ
//! Markdown 本文の直後（前後ナビの手前）へ生成 `Node` を追記する。該当
//! ページには専用 CSS（[`showcase::STYLESHEET_REL_PATH`]）への追加 `<link>` を
//! [`layout::docs_page_with_assets`] で差し込み、CSS 本体はステップ 5 の後に
//! [`showcase::stylesheet`] から書き出す。生成 CSS の組み立て（fallible）は
//! linkcheck と同じく **書き出しより前** に行い、失敗時は `out_dir` を汚さない
//! （fail-closed の処理順を維持する）。
//!
//! # admonition 構文（[`crate::markdown`]）が使う CSS（イシュー #715）
//!
//! `> [!NOTE]` 等の admonition マーカーは [`markdown::render_markdown`] が
//! `alert` 部品へ描画するが、その専用 CSS（[`admonition::STYLESHEET_REL_PATH`]）
//! は showcase と同型に「使われているページだけ」へ配線する。ステップ 2 の
//! `rewritten_body` を [`admonition::contains_admonition`] で走査し、1 つでも
//! 含むページには追加 `<link>` を差し込み・linkcheck の既知 href へ登録する。
//! 1 ページでも使われていれば CSS 本体（[`admonition::stylesheet`]）を組み立て
//! （showcase と同じく linkcheck より前・書き出しより前に完了させる
//! fail-closed 処理順）、ステップ 5 の後に書き出す。admonition を含まない
//! ページ・フィクスチャサイトのビルド結果は 1 バイトも変わらない。

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use fandhe_frontend_core::{div, Node};
use fandhe_frontend_server::ssg::{self, SsgError};

use fandhe_frontend_pre_styled_ui::StylesheetError;

use crate::admonition;
use crate::layout;
use crate::linkcheck::{self, BrokenLink};
use crate::markdown::render_markdown;
use crate::nav::{self, NavError};
use crate::showcase;
use crate::site_theme::{self, SiteThemeError};
use crate::skip_nav;

/// `site/assets/` 配下に存在すると [`BuildError::ReservedAssetName`] で
/// 拒否するファイル名（ビルド時生成 CSS と同名のファイル名）。
/// [`site_theme::STYLESHEET_REL_PATH`]/[`skip_nav::STYLESHEET_REL_PATH`]/
/// [`showcase::STYLESHEET_REL_PATH`]/[`admonition::STYLESHEET_REL_PATH`] は
/// いずれも `assets/<basename>` の形をしており、`site/assets/` 直下との
/// 名前衝突は basename の一致だけで判定できる。
const RESERVED_ASSET_NAMES: &[&str] = &[
    "site.css",
    "skip-nav.css",
    "pre-styled-ui.css",
    "admonition.css",
];

/// [`build_site`] が成功時に返すビルド結果のサマリ。
#[derive(Debug, Clone)]
pub struct BuildReport {
    /// 書き出したページファイルの絶対パス一覧（`generate_pages` の戻り値）。
    pub written: Vec<PathBuf>,
    /// コピーしたアセットファイルの絶対パス一覧。
    pub assets: Vec<PathBuf>,
}

/// [`build_site`] の失敗理由。
///
/// `Display` はリポジトリ相対パス・行番号・href のみを含み、絶対パス・
/// 環境変数・スタックトレース等の機微情報は含めない
/// （`security.md` の機微情報露出防止方針。[`NavError`] と同方針）。
#[derive(Debug)]
pub enum BuildError {
    /// `site/nav.toml` の読込・パース・ソース存在検証のいずれかが失敗した。
    Nav(NavError),
    /// ページ Markdown の読込・アセットコピーで I/O エラーが発生した。
    Io {
        /// 対象パス（表示用。`repo_root` からの相対パスを優先して構成する）。
        path: PathBuf,
        /// 発生した I/O エラー。
        source: std::io::Error,
    },
    /// `fandhe_frontend_server::ssg::generate_pages` が失敗した。
    Ssg(SsgError),
    /// 内部リンクの突合検証（`.md` リンク解決を含む）で 1 件以上のリンク
    /// 切れが見つかった。書き出しは一切行われていない。
    LinkCheck(Vec<BrokenLink>),
    /// `site/assets/` 配下にシンボリックリンク・ディレクトリ以外の特殊
    /// エントリが存在し、通常ファイルのみ許可する方針に反した
    /// （リポジトリ外ファイルの持ち出し防止のための fail-closed 検証）。
    UnsupportedAssetEntry(PathBuf),
    /// ショーケース専用 CSS（[`showcase::stylesheet`]）・admonition 専用 CSS
    /// （[`admonition::stylesheet`]）のいずれかの組み立てが
    /// [`StyleSheet`](fandhe_frontend_pre_styled_ui::StyleSheet) の検証に
    /// 落ちた（通常は到達しない。黙って CSS の欠けたページを公開しない
    /// fail-closed）。
    Stylesheet(StylesheetError),
    /// サイト骨格 CSS（[`site_theme::stylesheet`]）の組み立てが失敗した
    /// （docs 固有トークンの allowlist 検証、または [`StyleSheet::push_css`]
    /// の検証に落ちた。イシュー #905。通常は到達しない fail-closed）。
    SiteTheme(SiteThemeError),
    /// `site/assets/` 配下にビルド時生成 CSS と同名のファイル（[`RESERVED_ASSET_NAMES`]）
    /// が存在する（静的ファイルの黙った上書き・生成物のすり替わりを防ぐ
    /// fail-closed 検証、イシュー #905）。
    ReservedAssetName(PathBuf),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::Nav(e) => write!(f, "{e}"),
            BuildError::Io { path, source } => {
                write!(f, "I/O error at {path:?}: {source}")
            }
            BuildError::Ssg(e) => write!(f, "{e}"),
            BuildError::LinkCheck(broken) => {
                writeln!(f, "link check failed with {} broken link(s):", broken.len())?;
                for (i, b) in broken.iter().enumerate() {
                    if i > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "  - {b}")?;
                }
                Ok(())
            }
            BuildError::UnsupportedAssetEntry(path) => {
                write!(
                    f,
                    "unsupported entry under site/assets/ (only regular files are allowed): {path:?}"
                )
            }
            BuildError::Stylesheet(e) => {
                write!(f, "failed to assemble a generated stylesheet: {e}")
            }
            BuildError::SiteTheme(e) => {
                write!(f, "failed to assemble the site theme stylesheet: {e}")
            }
            BuildError::ReservedAssetName(path) => {
                write!(
                    f,
                    "site/assets/ contains a file name reserved for build-time generated CSS: {path:?}"
                )
            }
        }
    }
}

impl std::error::Error for BuildError {}

impl From<NavError> for BuildError {
    fn from(e: NavError) -> Self {
        BuildError::Nav(e)
    }
}

impl From<SsgError> for BuildError {
    fn from(e: SsgError) -> Self {
        BuildError::Ssg(e)
    }
}

impl From<StylesheetError> for BuildError {
    fn from(e: StylesheetError) -> Self {
        BuildError::Stylesheet(e)
    }
}

impl From<SiteThemeError> for BuildError {
    fn from(e: SiteThemeError) -> Self {
        BuildError::SiteTheme(e)
    }
}

/// `repo_root/site/nav.toml` を読み込み、全ページを組み立て、内部リンクを
/// 検証した上で `out_dir` へ書き出す。
///
/// # Errors
///
/// [`BuildError`] の各種別を参照。リンク切れが 1 件でもあれば
/// [`BuildError::LinkCheck`] を返し、`out_dir` には一切書き出さない。
pub fn build_site(repo_root: &Path, out_dir: &Path) -> Result<BuildReport, BuildError> {
    let nav_path = repo_root.join("site/nav.toml");
    let nav_input = fs::read_to_string(&nav_path).map_err(|source| BuildError::Io {
        path: PathBuf::from("site/nav.toml"),
        source,
    })?;
    let nav = nav::parse_nav(&nav_input)?;
    nav::validate_sources(&nav, repo_root)?;

    let source_to_path = linkcheck::source_to_path_map(&nav);

    let mut pages: Vec<(String, Node)> = Vec::new();
    let mut broken: Vec<BrokenLink> = Vec::new();
    // Rust 生成コンテンツページ（showcase）を 1 件以上組み込んだか。
    // 専用 CSS（showcase::STYLESHEET_REL_PATH）の書き出し・linkcheck 用
    // href 登録は該当ページが nav に存在するときだけ行う（フィクスチャ
    // サイト等、showcase を持たないサイトのビルド結果を変えないため）。
    let mut has_generated_page = false;
    // admonition 構文（`> [!NOTE]` 等）を 1 ページでも含んだか。showcase と
    // 同型の判定で、専用 CSS（admonition::STYLESHEET_REL_PATH）の書き出し・
    // linkcheck 用 href 登録を「実際に使われているときだけ」行う（モジュール
    // doc の admonition 節参照）。
    let mut has_admonition = false;

    for section in &nav.sections {
        for page in &section.pages {
            let source_path = repo_root.join(&page.source);
            let markdown_input =
                fs::read_to_string(&source_path).map_err(|source_err| BuildError::Io {
                    path: PathBuf::from(&page.source),
                    source: source_err,
                })?;

            let blocks = render_markdown(&markdown_input);
            let raw_body = div(vec![], blocks);
            let rewritten_body = linkcheck::rewrite_md_links(
                raw_body,
                &page.source,
                &nav,
                &page.path,
                &source_to_path,
                &mut broken,
            );

            // このページの admonition 使用有無（Markdown 由来の rewritten_body
            // のみを走査する。showcase の生成コンテンツは Markdown 外であり、
            // admonition マーカーを含み得ないため対象外）。
            let page_has_admonition = admonition::contains_admonition(&rewritten_body);
            if page_has_admonition {
                has_admonition = true;
            }

            // Rust 生成コンテンツ（showcase）は Markdown 本文の直後・前後
            // ナビの手前へ追記する（モジュール doc の処理順注記参照）。
            let generated = showcase::generated_content(&page.path);
            let mut extra_stylesheets: Vec<&str> = Vec::new();
            if generated.is_some() {
                has_generated_page = true;
                extra_stylesheets.push(showcase::STYLESHEET_REL_PATH);
            }
            if page_has_admonition {
                extra_stylesheets.push(admonition::STYLESHEET_REL_PATH);
            }

            let mut body_children = vec![rewritten_body];
            if let Some(generated_body) = generated {
                body_children.push(generated_body);
            }
            body_children.push(nav::prev_next_nav(&nav, &page.path));
            let body = div(vec![], body_children);

            let document = layout::docs_page_with_assets(
                &page.title,
                &nav.site.base_path,
                nav::sidebar(&nav, &page.path),
                body,
                &extra_stylesheets,
                Some(nav::header_nav(&nav, &page.path)),
            );

            pages.push((page.path.clone(), document));
        }
    }

    let mut asset_hrefs = collect_asset_hrefs(repo_root, &nav.site.base_path)?;

    // showcase / admonition の専用 CSS はいずれもビルド時生成のため
    // site/assets/ には存在しない。linkcheck が追加 <link> の href を
    // 「未知のターゲット」と誤検知しないよう、生成することが確定した時点で
    // 既知 href へ登録する。CSS 本体の組み立ても linkcheck より前に済ませ、
    // 失敗時は書き出し前に打ち切る（fail-closed の処理順、モジュール doc 参照）。
    let showcase_sheet = if has_generated_page {
        asset_hrefs.push(layout::asset_href(
            &nav.site.base_path,
            showcase::STYLESHEET_REL_PATH,
        ));
        Some(showcase::stylesheet()?)
    } else {
        None
    };
    let admonition_sheet = if has_admonition {
        asset_hrefs.push(layout::asset_href(
            &nav.site.base_path,
            admonition::STYLESHEET_REL_PATH,
        ));
        Some(admonition::stylesheet()?)
    } else {
        None
    };
    // SkipNav（イシュー #776）・サイト骨格 CSS（`site_theme`、イシュー #905）は
    // showcase/admonition と異なり全ページへ無条件に適用するため、条件判定
    // なしで常に href 登録・CSS 組み立てを行う（`crate::skip_nav`/
    // `crate::site_theme` モジュール doc 参照）。
    asset_hrefs.push(layout::asset_href(
        &nav.site.base_path,
        skip_nav::STYLESHEET_REL_PATH,
    ));
    let skip_nav_sheet = skip_nav::stylesheet()?;
    asset_hrefs.push(layout::asset_href(
        &nav.site.base_path,
        site_theme::STYLESHEET_REL_PATH,
    ));
    let site_theme_sheet = site_theme::stylesheet()?;

    let mut link_check_broken = linkcheck::check_links(&pages, &nav.site.base_path, &asset_hrefs);
    broken.append(&mut link_check_broken);

    if !broken.is_empty() {
        // fail-closed: 書き出しより前に打ち切る。`generate_pages` にも
        // 到達させない（「一部だけ更新された dist/」を残さないため）。
        return Err(BuildError::LinkCheck(broken));
    }

    let written = ssg::generate_pages(&pages, out_dir)?;
    let mut assets = copy_assets(repo_root, out_dir)?;

    if let Some(sheet) = showcase_sheet {
        let css_path = out_dir.join(showcase::STYLESHEET_REL_PATH);
        sheet
            .write_css_file(&css_path)
            .map_err(|source| BuildError::Io {
                path: PathBuf::from(showcase::STYLESHEET_REL_PATH),
                source,
            })?;
        assets.push(css_path);
    }
    if let Some(sheet) = admonition_sheet {
        let css_path = out_dir.join(admonition::STYLESHEET_REL_PATH);
        sheet
            .write_css_file(&css_path)
            .map_err(|source| BuildError::Io {
                path: PathBuf::from(admonition::STYLESHEET_REL_PATH),
                source,
            })?;
        assets.push(css_path);
    }
    {
        let css_path = out_dir.join(skip_nav::STYLESHEET_REL_PATH);
        skip_nav_sheet
            .write_css_file(&css_path)
            .map_err(|source| BuildError::Io {
                path: PathBuf::from(skip_nav::STYLESHEET_REL_PATH),
                source,
            })?;
        assets.push(css_path);
    }
    {
        let css_path = out_dir.join(site_theme::STYLESHEET_REL_PATH);
        site_theme_sheet
            .write_css_file(&css_path)
            .map_err(|source| BuildError::Io {
                path: PathBuf::from(site_theme::STYLESHEET_REL_PATH),
                source,
            })?;
        assets.push(css_path);
    }

    Ok(BuildReport { written, assets })
}

/// `site/assets/` 配下の通常ファイル一覧から、突合検証用の href
/// （`base_path + "/assets/" + ファイル名`）列を構築する。
///
/// ディレクトリ走査自体は行わず [`copy_assets`] と同じ列挙ロジックを
/// 再利用しないのは、linkcheck を書き出しより前に完了させる本モジュールの
/// 処理順（モジュール冒頭の設計）上、コピーの副作用（`out_dir` への書き込み）
/// より前に href 集合だけを先に必要とするため。
fn collect_asset_hrefs(repo_root: &Path, base_path: &str) -> Result<Vec<String>, BuildError> {
    let assets_dir = repo_root.join("site/assets");
    let mut hrefs = Vec::new();
    for entry in list_regular_files(&assets_dir)? {
        let file_name = entry
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        hrefs.push(format!("{base_path}/assets/{file_name}"));
    }
    Ok(hrefs)
}

/// `dir` 直下の通常ファイルのみを列挙する。シンボリックリンク・
/// サブディレクトリ・その他特殊エントリが見つかった場合は
/// [`BuildError::UnsupportedAssetEntry`] を返す（fail-closed。
/// `site/assets/` はリポジトリ管理下の固定ディレクトリであり、想定外の
/// エントリ種別を許容しないことでリポジトリ外ファイルの持ち出し・
/// 予期しないシンボリックリンク追従を防ぐ）。
///
/// `dir` 自体が存在しない場合は空列を返す（イシュー #905: サイト骨格 CSS が
/// [`crate::site_theme`] のビルド時生成へ切り替わり、`site/assets/` に静的
/// ファイルを置く必然性が無くなったため、ディレクトリ不存在はエラーではなく
/// 「アセットなし」として許容する。それ以外の I/O エラーは従来どおり
/// [`BuildError::Io`] を返す）。
///
/// 列挙した各ファイルの basename が [`RESERVED_ASSET_NAMES`]（ビルド時生成
/// CSS と同名）に一致する場合は [`BuildError::ReservedAssetName`] を返す
/// （静的ファイルの黙った上書き・生成物のすり替わりを防ぐ fail-closed
/// 検証。呼び出し元は [`collect_asset_hrefs`]/[`copy_assets`] の双方で
/// 本関数を経由するため、書き出しより前（`collect_asset_hrefs` の呼び出し
/// 時点）に検知が完了する）。
fn list_regular_files(dir: &Path) -> Result<Vec<PathBuf>, BuildError> {
    let read_dir = match fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(source) => {
            return Err(BuildError::Io {
                path: PathBuf::from("site/assets"),
                source,
            })
        }
    };
    let mut files = Vec::new();
    for entry in read_dir {
        let entry = entry.map_err(|source| BuildError::Io {
            path: PathBuf::from("site/assets"),
            source,
        })?;
        // `DirEntry::metadata` はシンボリックリンク自体の種別を返す
        // （`std::fs::metadata` のようにリンク先を追跡することはない。
        // リンク先を追跡してしまうと、リンク先がリポジトリ外の通常ファイルの
        // 場合に判定をすり抜ける）。
        let file_type = entry
            .metadata()
            .map_err(|source| BuildError::Io {
                path: entry.path(),
                source,
            })?
            .file_type();
        if file_type.is_file() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if RESERVED_ASSET_NAMES.contains(&name) {
                    return Err(BuildError::ReservedAssetName(path));
                }
            }
            files.push(path);
        } else {
            return Err(BuildError::UnsupportedAssetEntry(entry.path()));
        }
    }
    files.sort();
    Ok(files)
}

/// `repo_root/site/assets/` 配下の通常ファイルを `out_dir/assets/` へコピーする。
fn copy_assets(repo_root: &Path, out_dir: &Path) -> Result<Vec<PathBuf>, BuildError> {
    let assets_dir = repo_root.join("site/assets");
    let out_assets_dir = out_dir.join("assets");
    fs::create_dir_all(&out_assets_dir).map_err(|source| BuildError::Io {
        path: out_assets_dir.clone(),
        source,
    })?;

    let mut copied = Vec::new();
    for src in list_regular_files(&assets_dir)? {
        let file_name = src.file_name().unwrap_or_default();
        let dest = out_assets_dir.join(file_name);
        fs::copy(&src, &dest).map_err(|source| BuildError::Io {
            path: dest.clone(),
            source,
        })?;
        copied.push(dest);
    }
    Ok(copied)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト専用の一時ディレクトリ。`nav.rs`/`ssg.rs` のテストヘルパーと
    /// 同方針（外部クレート `tempfile` を追加しない、REQ-3）。
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            let unique = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let path = crate::test_scratch::scratch_root().join(format!(
                "fandhe-frontend-docs-site-build-test-{tag}-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("create temp dir for build.rs test");
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_fixture_site(root: &Path) {
        fs::create_dir_all(root.join("site/assets")).unwrap();
        fs::write(
            root.join("site/nav.toml"),
            r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Guide"

[[section.page]]
title = "Intro"
source = "site/intro.md"
path = "/"

[[section.page]]
title = "Next"
source = "site/next.md"
path = "/next/"
"#,
        )
        .unwrap();
        fs::write(root.join("site/intro.md"), "# Intro\n\n[Next](./next.md)\n").unwrap();
        fs::write(root.join("site/next.md"), "# Next\n\nBack to intro.\n").unwrap();
        // `site/assets/` は空のまま残す（`build_site_rejects_directory_entry_under_assets`
        // がこの下にサブディレクトリを追加する用途で使う）。サイト骨格 CSS
        // （`assets/site.css`）はイシュー #905 以降ビルド時生成のため、
        // ここで静的ファイルを書かない（書けば `RESERVED_ASSET_NAMES` に
        // 抵触し `BuildError::ReservedAssetName` になる）。
    }

    #[test]
    fn build_site_writes_pages_and_assets_for_valid_fixture() {
        let temp = TempDir::new("ok");
        write_fixture_site(&temp.0);
        let out_dir = temp.0.join("dist");

        let report = build_site(&temp.0, &out_dir).expect("valid fixture should build");
        assert_eq!(report.written.len(), 2);
        // サイト骨格 CSS（`site_theme`、ビルド時生成）+ SkipNav 専用 CSS
        // （イシュー #776、全ビルドで無条件に書き出す。`crate::skip_nav`
        // モジュール doc 参照）の 2 件。showcase/admonition 専用 CSS は本
        // フィクスチャが使わないため含まれない。
        assert_eq!(report.assets.len(), 2);
        assert!(out_dir.join("index.html").exists());
        assert!(out_dir.join("next/index.html").exists());
        assert!(out_dir.join("assets/site.css").exists());
        assert!(out_dir.join(skip_nav::STYLESHEET_REL_PATH).exists());

        let index_html = fs::read_to_string(out_dir.join("index.html")).unwrap();
        assert!(index_html.contains(r#"href="/next/""#));
        assert!(!index_html.contains(".md"));
    }

    #[test]
    fn build_site_fails_closed_on_broken_md_link_without_writing_output() {
        let temp = TempDir::new("broken-md-link");
        write_fixture_site(&temp.0);
        fs::write(
            temp.0.join("site/intro.md"),
            "# Intro\n\n[Missing](./missing.md)\n",
        )
        .unwrap();
        let out_dir = temp.0.join("dist");

        let err = build_site(&temp.0, &out_dir).expect_err("broken .md link should fail the build");
        match err {
            BuildError::LinkCheck(broken) => {
                assert_eq!(broken.len(), 1);
                assert!(broken[0].href.contains("missing.md"));
            }
            other => panic!("expected LinkCheck, got {other:?}"),
        }
        assert!(!out_dir.exists());
    }

    #[test]
    fn build_site_fails_closed_on_broken_absolute_link_without_writing_output() {
        let temp = TempDir::new("broken-abs-link");
        write_fixture_site(&temp.0);
        fs::write(
            temp.0.join("site/intro.md"),
            "# Intro\n\n[Ghost](/does-not-exist/)\n",
        )
        .unwrap();
        let out_dir = temp.0.join("dist");

        let err = build_site(&temp.0, &out_dir).expect_err("broken absolute link should fail");
        assert!(matches!(err, BuildError::LinkCheck(_)));
        assert!(!out_dir.exists());
    }

    #[test]
    fn build_site_reports_nav_error_for_missing_nav_toml() {
        let temp = TempDir::new("missing-nav");
        let out_dir = temp.0.join("dist");
        let err = build_site(&temp.0, &out_dir).expect_err("missing nav.toml should fail");
        assert!(matches!(err, BuildError::Io { .. }));
    }

    #[test]
    fn build_site_rejects_directory_entry_under_assets() {
        let temp = TempDir::new("bad-asset-entry");
        write_fixture_site(&temp.0);
        fs::create_dir_all(temp.0.join("site/assets/nested")).unwrap();
        let out_dir = temp.0.join("dist");
        let err = build_site(&temp.0, &out_dir).expect_err("directory under assets should fail");
        assert!(matches!(err, BuildError::UnsupportedAssetEntry(_)));
        assert!(!out_dir.exists());
    }

    /// イシュー #905: `site/assets/` にビルド時生成 CSS と同名のファイルを
    /// 置くと、静的ファイルの黙った上書き・生成物のすり替わりを防ぐため
    /// `BuildError::ReservedAssetName` で拒否される（書き出しより前に検知
    /// する fail-closed 検証、モジュール doc 参照）。
    #[test]
    fn build_site_rejects_reserved_asset_name_under_assets() {
        let temp = TempDir::new("reserved-asset-name");
        write_fixture_site(&temp.0);
        fs::write(temp.0.join("site/assets/site.css"), "body{}\n").unwrap();
        let out_dir = temp.0.join("dist");
        let err =
            build_site(&temp.0, &out_dir).expect_err("reserved asset name should fail the build");
        assert!(matches!(err, BuildError::ReservedAssetName(_)));
        assert!(!out_dir.exists());
    }

    /// イシュー #905: `site/assets/` ディレクトリ自体が存在しなくても
    /// ビルドは「アセットなし」として成功する（サイト骨格 CSS がビルド時
    /// 生成へ切り替わり、静的ファイルを置く必然性が無くなったため）。
    #[test]
    fn build_site_succeeds_when_site_assets_directory_is_absent() {
        let temp = TempDir::new("no-assets-dir");
        write_fixture_site(&temp.0);
        fs::remove_dir_all(temp.0.join("site/assets")).unwrap();
        let out_dir = temp.0.join("dist");

        let report =
            build_site(&temp.0, &out_dir).expect("missing site/assets/ directory should build");
        // サイト骨格 CSS + SkipNav 専用 CSS のみ（`site/assets/` 由来のコピー
        // アセットは 0 件）。
        assert_eq!(report.assets.len(), 2);
        assert!(out_dir.join("assets/site.css").exists());
    }
}

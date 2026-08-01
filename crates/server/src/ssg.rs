//! SSG エントリ（TASK-6.1c・#348）: [`crate::ssr::respond_with`] の 200 応答
//! ボディをそのまま静的ファイルへ書き出す。
//!
//! # 呼び出し文脈・契約
//!
//! - `server/src/bin/ssg.rs`（CLI 版 SSG バイナリ）から呼ばれる。
//! - SSR 出力（[`crate::ssr::SsrResponse::body`]）を**そのまま**ファイルへ
//!   書き出すのみで、独自の HTML 組み立て・独自のエスケープ処理を行わない
//!   （REQ-6: SSR/SSG 出力の文字列完全一致が構成上自明になる。
//!   `docs/api/app-api.md` 第 4 節・判断 5・`docs/design/loader-trait-design.md`
//!   §4「SSG が独自に loader を呼ぶ描画経路を新設しない」）。
//! - ルート列挙（一覧に何件のアイテムがあるか）は
//!   [`fandhe_frontend_app::Loader::load`]（一覧 loader）でビルド時に解決する。各ルート
//!   の HTML 生成自体は従来どおり [`crate::ssr::respond_with`] を呼ぶため、
//!   loader は 1 回の [`generate_with`] 実行で複数回（列挙 1 回 + 各ルート
//!   描画 1 回）呼ばれる。決定的な loader（同一入力に同一出力を返す）で
//!   あることは型システムの外側の責務であり、テスト
//!   （`server/tests/ssr_ssg_parity.rs`）で固定する（`fandhe-frontend-app` の `Loader`
//!   rustdoc の「型で保証する範囲」注記と同じ位置づけ）。
//! - `std::fs` のみを使用し、外部クレート（`tempfile` 等）を追加しない
//!   （REQ-3、`coding-rust.md`）。
//! - [`generate_pages`]（イシュー #463）は上記 2 API とは別系統の汎用 SSG
//!   API で、固定ルート表・`Loader`・`respond_with` を経由せず、呼び出し側
//!   が渡した任意の (リクエストパス, [`fandhe_frontend_core::Node`]) 列を直接
//!   `fandhe_frontend_core::render` してファイル化する。後続の
//!   `fandhe-frontend-docs-site`（イシュー #457 系）が任意階層のドキュメント
//!   ページを `dist/` へ書き出す土台として呼ぶ想定（親イシュー #457
//!   Phase 1-1）。
//! - [`generate_assets`]（イシュー #1119）は [`generate_pages`] と同じ
//!   fail-closed のパス検証系を使いつつ、出力を `<path>/index.html` 固定
//!   ではなく任意のファイル名（`sitemap.xml` / `robots.txt` 等、拡張子
//!   付き・拡張子なしいずれも可）へ拡張した汎用アセット書き出し API。
//!   利用者が `std::fs::write` で直書きし `generate_pages` のパス検証の
//!   恩恵を受けられなかった非 HTML 生成物向け（イシュー本文の動機）。
//!   `Node` 木を経由せず文字列コンテンツをそのまま書き出すため、HTML の
//!   組み立てには使わないこと（詳細は [`generate_assets`] rustdoc）。
//!   イシュー #1137 で中間ディレクトリセグメントにもファイル名と同じ
//!   ドット許可述語（[`is_safe_asset_file_name`]）を適用し、
//!   `/.well-known/security.txt` のような RFC 8615 well-known URI 配下への
//!   出力を許可した（詳細は [`normalize_asset_path`] rustdoc）。
//!
//! # セキュリティ不変条件（OWASP A01 パストラバーサル対策・fail-closed）
//!
//! - 出力ファイルパスは固定ルート表（`/` → `index.html`、
//!   `/items/{id}` → `items/{id}/index.html`）から `out_dir` 配下に限定して
//!   構成する。`Item::id` は `fandhe-frontend-app` の公開フィールドであり loader 由来の
//!   任意の値を持ちうるため、`..`・`/`・`\` を含む id はエラーとして拒否し、
//!   英数字・`-`・`_` のみを許可するホワイトリスト検証を loader 出力の各
//!   `item.id` に対して従来どおり適用する（この段落は [`generate`] /
//!   [`generate_with`] の固定ルート表に限定される）。
//! - [`generate_pages`] の任意ページパス（例: `/guide/foo/`）も同じ
//!   `is_safe_path_segment` ホワイトリストを全セグメントに適用して検証する
//!   （先頭 `/` 必須・`..`/`.`/空セグメント拒否）。1 件でも検証・重複判定に
//!   失敗した場合は **どのページも書き出さずに** エラーを返す（fail-closed。
//!   `generate_with` の「ルート単位で逐次書き出し」より強い、全件事前検証の
//!   保証）。
//! - loader が解決に失敗した場合（一覧列挙・各ルート描画のいずれでも）は
//!   [`SsgError::LoaderError`] としてビルドを即座に失敗させ、それまでに
//!   書き出したファイルの有無に関わらずエラーを返す（部分成功で握り
//!   つぶさない = fail-closed、設計書 §5）。`Loader::Error` の値自体は
//!   [`SsgError::Display`] にも一切含めない（[`crate::ssr::loader_error_response`]
//!   と同様の非露出契約。`security.md`「機微情報の露出」）。
//! - `unwrap`/`panic!` は使わず、書き込み・検証の失敗はすべて
//!   [`SsgError`]（`Result`）として呼び出し元へ伝える
//!   （`coding-rust.md` のエラー処理規約）。
//! - [`generate_assets`] の中間ディレクトリセグメントは
//!   [`is_safe_asset_dir_segment`]（イシュー #1137）で検証する。ドット始まり
//!   の名前（`.well-known` 等）を許可しつつ、`.`/`..`/`...` のようなドット
//!   のみの名前は位置を問わず構造的に拒否し（トラバーサル不可の不変条件を
//!   維持）、加えて `.git`（ASCII 大文字小文字非区別）を defense-in-depth
//!   として明示拒否する（`out_dir` が git worktree の場合の `config`/
//!   `hooks` 汚染防止、OWASP A01）。

use crate::ssr::respond_with;
use fandhe_frontend_app::{DemoItemDetailLoader, DemoItemsLoader, Item, Loader};
use fandhe_frontend_core::{render, Node};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// [`generate`] の失敗理由。
#[derive(Debug)]
pub enum SsgError {
    /// `Item::id` に `..`・`/`・`\` 等の非許可文字を含み、出力パスの構成を
    /// 拒否した（パストラバーサル対策）。
    UnsafeItemId(String),
    /// 出力先ディレクトリの作成に失敗した。
    CreateDir {
        /// 作成しようとしたディレクトリパス。
        path: PathBuf,
        /// `std::fs::create_dir_all` が返した I/O エラー。
        source: std::io::Error,
    },
    /// ファイル書き込みに失敗した。
    WriteFile {
        /// 書き込み先ファイルパス。
        path: PathBuf,
        /// `std::fs::write` が返した I/O エラー。
        source: std::io::Error,
    },
    /// [`crate::ssr::respond`] が `None` を返した（ルート定義との不整合。
    /// 通常到達しないが、固定ルート表の変更漏れを検知するために保持する）。
    RouteNotFound(String),
    /// [`crate::ssr::respond_with`] が 200 以外のステータス（例: 404）を返した。
    /// `generate_with()` は一覧 loader 自身から導出したパスしか
    /// `write_route` に渡さないため通常到達しないが、「200 応答ボディを
    /// そのまま書き出す」という契約をコード上でも明示的に強制するために
    /// 検証する（ルート表と loader 出力が将来ズレた場合の防御）。500
    /// （loader 失敗）はこのバリアントではなく [`SsgError::LoaderError`] へ
    /// 区別して伝播する。
    UnexpectedStatus {
        /// 対象のリクエストパス。
        path: String,
        /// `respond_with()` が実際に返したステータスコード。
        status: u16,
    },
    /// loader がデータ解決に失敗した（一覧列挙時の直接失敗、または各ルート
    /// 描画時に [`crate::ssr::respond_with`] が 500 応答を返した場合の両方を
    /// 含む）。`Display` はルートパスのみを含み、`Loader::Error` の内部詳細
    /// （内部パス・接続情報等）は一切含めない（fail-closed、設計書 §5・§9-5）。
    LoaderError {
        /// 解決に失敗したルートパス（一覧列挙自体の失敗時は `"/"`）。
        path: String,
    },
    /// [`generate_pages`]/[`generate_assets`] に渡されたページ/アセット
    /// パスが検証を通らなかった（先頭 `/` が無い・`..`/`.` を含む・空
    /// セグメントを含む・非許可文字を含む、のいずれか）。`Display` は
    /// 呼び出し元が渡したパス文字列のみを含み、内部パス等の機微情報は
    /// 含めない。
    UnsafePagePath(String),
    /// [`generate_pages`]/[`generate_assets`] で複数のページ/アセットパスが
    /// 正規化後に同じ出力先（例: `/a` と `/a/` はいずれも `a/index.html`）を
    /// 指した。サイレント上書きを避けるため fail-closed でエラー化する。
    DuplicatePagePath(String),
}

impl fmt::Display for SsgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SsgError::UnsafeItemId(id) => {
                write!(f, "item id contains disallowed characters: {id:?}")
            }
            SsgError::CreateDir { path, source } => {
                write!(f, "failed to create directory {path:?}: {source}")
            }
            SsgError::WriteFile { path, source } => {
                write!(f, "failed to write file {path:?}: {source}")
            }
            SsgError::RouteNotFound(path) => {
                write!(f, "no SSR route matched fixed path {path:?}")
            }
            SsgError::UnexpectedStatus { path, status } => {
                write!(
                    f,
                    "SSR route {path:?} returned unexpected status {status} (expected 200)"
                )
            }
            SsgError::LoaderError { path } => {
                write!(f, "loader failed to resolve data for route {path:?}")
            }
            SsgError::UnsafePagePath(path) => {
                write!(f, "page/asset path failed validation: {path:?}")
            }
            SsgError::DuplicatePagePath(path) => {
                write!(
                    f,
                    "page/asset path resolves to a duplicate output: {path:?}"
                )
            }
        }
    }
}

impl std::error::Error for SsgError {}

/// `id` が出力パス片として安全（英数字・`-`・`_` のみ）かを検証する。
///
/// デモデータ（[`fandhe_frontend_app::demo_items`]）はすべて数値 id だが、`Item` は
/// 公開構造体であり将来任意の由来（DB・API 等）のデータを持ちうるため、
/// `..`・`/`・`\` を含む id を機械的に拒否する（OWASP A01）。
fn is_safe_path_segment(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// `out_dir` 配下へ `/` と既定 loader（[`DemoItemsLoader`] /
/// [`DemoItemDetailLoader`]）が列挙する各詳細ページを静的ファイルとして
/// 書き出す。[`generate_with`] を既定 loader で呼ぶ薄い互換ラッパーであり、
/// 公開シグネチャは #347 以前から非破壊（`server/src/bin/ssg.rs` は無修正の
/// まま利用継続できる）。
pub fn generate(out_dir: &Path) -> Result<Vec<PathBuf>, SsgError> {
    generate_with(&DemoItemsLoader, &DemoItemDetailLoader, out_dir)
}

/// loader を差し替え可能なジェネリック版。書き出したファイルの絶対パス
/// 一覧を返す。
///
/// - ルート列挙: `list_loader.load(&())` をビルド時に呼ぶ（SSG のビルド時
///   解決）。`Err(_)` は即座に `SsgError::LoaderError { path: "/".into() }`
///   へ変換してビルドを失敗させる（部分成功で握りつぶさない = fail-closed）。
/// - 各ルートの HTML 生成: 従来どおり [`crate::ssr::respond_with`] の 200
///   応答ボディをそのまま書き出す（SSR/SSG バイト完全一致の構造的保証を
///   維持。同一 loader を列挙と描画で 2 回呼ぶ点はモジュール冒頭の注記を
///   参照）。
/// - `is_safe_path_segment` による id ホワイトリスト検証（OWASP A01）は
///   loader 出力の各 `item.id` に対して従来どおり適用する。
pub fn generate_with<L, D>(
    list_loader: &L,
    detail_loader: &D,
    out_dir: &Path,
) -> Result<Vec<PathBuf>, SsgError>
where
    L: Loader<Input = (), Output = Vec<Item>>,
    D: Loader<Input = String, Output = Option<Item>>,
{
    let items = list_loader.load(&()).map_err(|_| SsgError::LoaderError {
        path: "/".to_string(),
    })?;

    let mut written = Vec::new();

    written.push(write_route(
        list_loader,
        detail_loader,
        out_dir,
        "/",
        "index.html",
    )?);

    for item in items {
        if !is_safe_path_segment(&item.id) {
            return Err(SsgError::UnsafeItemId(item.id));
        }
        let request_path = format!("/items/{}", item.id);
        let relative = format!("items/{}/index.html", item.id);
        written.push(write_route(
            list_loader,
            detail_loader,
            out_dir,
            &request_path,
            &relative,
        )?);
    }

    Ok(written)
}

/// 1 ルート分を解決して `out_dir/relative_path` へ書き出す共通処理。
///
/// `respond_with()` が返した 200 応答ボディのみを書き出す契約。500
/// （loader 失敗）は [`SsgError::LoaderError`] へ、それ以外の非 200
/// （`RouteNotFound`/`UnexpectedStatus`）は既存どおり区別してエラーとして
/// 呼び出し元（[`generate_with`]）へ伝播し、いずれもファイルを書き出さない。
fn write_route<L, D>(
    list_loader: &L,
    detail_loader: &D,
    out_dir: &Path,
    request_path: &str,
    relative_path: &str,
) -> Result<PathBuf, SsgError>
where
    L: Loader<Input = (), Output = Vec<Item>>,
    D: Loader<Input = String, Output = Option<Item>>,
{
    let response = respond_with(list_loader, detail_loader, request_path)
        .ok_or_else(|| SsgError::RouteNotFound(request_path.to_string()))?;
    if response.status == 500 {
        return Err(SsgError::LoaderError {
            path: request_path.to_string(),
        });
    }
    if response.status != 200 {
        return Err(SsgError::UnexpectedStatus {
            path: request_path.to_string(),
            status: response.status,
        });
    }

    write_file(out_dir, relative_path, &response.body)
}

/// `out_dir/relative_path` へ `body` を書き出す共通 I/O ヘルパー。
///
/// [`write_route`]（固定ルート表・`respond_with` 経由）と [`generate_pages`]
/// （任意ページパス・`render` 経由）の両方から呼ばれる、ディレクトリ作成 +
/// ファイル書き込みのみを担う末端処理。呼び出し元がそれぞれの契約
/// （ステータス検証・パス検証）を満たした後にのみ呼ぶこと。
fn write_file(out_dir: &Path, relative_path: &str, body: &str) -> Result<PathBuf, SsgError> {
    let file_path = out_dir.join(relative_path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).map_err(|source| SsgError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(&file_path, body.as_bytes()).map_err(|source| SsgError::WriteFile {
        path: file_path.clone(),
        source,
    })?;

    Ok(file_path)
}

/// [`generate_pages`] 用のページパス正規化・検証。
///
/// - `/` → `"index.html"`
/// - `/guide/foo/`・`/guide/foo`（末尾スラッシュの有無を問わない）→
///   `"guide/foo/index.html"`
/// - 先頭 `/` が無い・`..`/`.` を含む・空セグメント（`//`）を含む・
///   非許可文字（英数字・`-`・`_` 以外）を含む場合は
///   [`SsgError::UnsafePagePath`] を返す。
///
/// セグメント単位の検証は [`generate`]/[`generate_with`] の `item.id`
/// 検証と同じ [`is_safe_path_segment`] を再利用し、ホワイトリストの
/// 二重管理を避ける。先頭 `/` 必須 + 全セグメントホワイトリスト通過に
/// より、戻り値を `out_dir.join(..)` した結果が `out_dir` 外を指す経路は
/// 構造的に存在しない（OWASP A01）。
fn normalize_page_path(path: &str) -> Result<String, SsgError> {
    let Some(rest) = path.strip_prefix('/') else {
        return Err(SsgError::UnsafePagePath(path.to_string()));
    };

    if rest.is_empty() {
        return Ok("index.html".to_string());
    }

    let trimmed = rest.strip_suffix('/').unwrap_or(rest);
    if trimmed.is_empty() {
        // 入力が "/" のみだった場合はここに到達しない（rest.is_empty() で
        // 先に処理済み）。"//" のように先頭スラッシュ直後が空セグメントの
        // ケースはここで拒否する。
        return Err(SsgError::UnsafePagePath(path.to_string()));
    }

    for segment in trimmed.split('/') {
        if !is_safe_path_segment(segment) {
            return Err(SsgError::UnsafePagePath(path.to_string()));
        }
    }

    Ok(format!("{trimmed}/index.html"))
}

/// 任意の (リクエストパス, [`Node`]) 列を `out_dir` 配下へ静的書き出しする
/// 汎用 SSG API（イシュー #463）。
///
/// [`generate`]/[`generate_with`] が固定ルート表（`/` と `/items/{id}`）に
/// 限定されるのに対し、本関数は任意階層のページ（例: `/guide/foo/`）を
/// 書き出せる。後続の `fandhe-frontend-docs-site`（イシュー #457 系）から、
/// Markdown 等をレンダリングして得た `Node` 列を渡して `dist/` を生成する
/// 想定で呼ばれる。
///
/// # 契約
///
/// - 各ページの HTML 化は `format!("<!DOCTYPE html>\n{}", fandhe_frontend_core::render(node))`
///   で行う。`render()` を経由するため `Node::Text`・属性値は必ず既定
///   エスケープを通る（REQ-1）。`<!DOCTYPE html>` はユーザー入力を含まない
///   固定リテラルの前置のみであり、[`fandhe_frontend_app::page_shell`] と
///   同一の許容済みパターン（新たなエスケープ迂回経路ではない）。
/// - `pages` 全件のパスを先に [`normalize_page_path`] で検証し、正規化後の
///   出力先の重複も検出する。1 件でも不正・重複があれば
///   **ファイルを 1 つも書き出さずに** エラーを返す（fail-closed。
///   `generate_with` のルート単位の逐次書き出しより強い保証）。
/// - `pages` が空なら `Ok(vec![])` を返し、何も書き出さない。
///
/// # Errors
///
/// - [`SsgError::UnsafePagePath`][]: いずれかのページパスが検証に失敗した。
/// - [`SsgError::DuplicatePagePath`][]: 正規化後の出力先が重複した。
/// - [`SsgError::CreateDir`][]/[`SsgError::WriteFile`][]: I/O エラー。
pub fn generate_pages(pages: &[(String, Node)], out_dir: &Path) -> Result<Vec<PathBuf>, SsgError> {
    // fail-closed: 書き出し前に全ページのパスを検証・重複判定する。
    // 途中まで書き出してからエラーで打ち切ると、失敗時に「一部だけ更新
    // された dist/」が残り部分成功を招く（設計書 §5 と同じ方針）。
    let mut relative_paths = Vec::with_capacity(pages.len());
    for (path, _) in pages {
        let relative = normalize_page_path(path)?;
        if relative_paths.contains(&relative) {
            return Err(SsgError::DuplicatePagePath(path.clone()));
        }
        relative_paths.push(relative);
    }

    let mut written = Vec::with_capacity(pages.len());
    for (relative, (_, node)) in relative_paths.iter().zip(pages.iter()) {
        let body = format!("<!DOCTYPE html>\n{}", render(node));
        written.push(write_file(out_dir, relative, &body)?);
    }

    Ok(written)
}

/// アセットのファイル名（[`normalize_asset_path`] の最終セグメント）が
/// 出力パス片として安全かを検証する。
///
/// [`is_safe_path_segment`] の許可文字集合（英数字・`-`・`_`）に加え、
/// ファイル拡張子（`sitemap.xml`・`robots.txt` 等）を表現するための `.`
/// を許可する。ただし `.`・`..`・`...` のようなドットのみの名前は
/// パストラバーサル対策として構造的に拒否するため、非 `.` 文字を最低 1
/// つ含むことを必須条件とする（`/`・`\` はそもそも許可文字集合に無く
/// 拒否される）。
fn is_safe_asset_file_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        && name.chars().any(|c| c != '.')
}

/// アセットパスの中間セグメント（ディレクトリ名）が出力パス片として
/// 安全かを検証する（イシュー #1137）。
///
/// [`is_safe_asset_file_name`] と同じ許可文字集合・「ドットのみの名前を
/// 拒否」の不変条件をそのまま再利用し、`.well-known` のようなドット始まり
/// ディレクトリを許可する（RFC 8615 well-known URI 配下へのアセット出力を
/// 可能にする、イシュー本文の動機）。加えて `.git`（ASCII 大文字小文字
/// 非区別）を defense-in-depth として明示拒否する。アセットパスは通常
/// 開発者記述の定数だが、loader 由来データからパスを合成する利用も
/// 否定できず、`out_dir` が git worktree（gh-pages デプロイ等）である
/// 場合に `.git/config`・`.git/hooks/...` への書き出しは任意コード実行に
/// つながり得るため、fail-closed 方針に沿って安全側に倒す（OWASP A01）。
fn is_safe_asset_dir_segment(name: &str) -> bool {
    is_safe_asset_file_name(name) && !name.eq_ignore_ascii_case(".git")
}

/// [`generate_assets`] 用のアセットパス正規化・検証。
///
/// - `/sitemap.xml` → `"sitemap.xml"`、`/assets/site.css` →
///   `"assets/site.css"`、`/healthz` → `"healthz"`（拡張子の有無を問わない）、
///   `/.well-known/security.txt` → `".well-known/security.txt"`
///   （イシュー #1137、ドット始まり中間ディレクトリの許可）。
/// - 先頭 `/` が無い・末尾が `/`（[`normalize_page_path`] と異なりアセットは
///   常にファイルを指すため `/healthz/` のような表記は非対応）・空セグメント
///   （`//`）を含む場合は [`SsgError::UnsafePagePath`] を返す。
/// - 中間セグメント（ディレクトリ名相当）・最終セグメント（ファイル名）の
///   いずれにも [`is_safe_asset_dir_segment`]/[`is_safe_asset_file_name`]
///   （英数字・`-`・`_`・`.`、ドットのみの名前は拒否）を適用する。中間
///   セグメントはさらに `.git`（大文字小文字非区別）を拒否する
///   （[`is_safe_asset_dir_segment`] 参照）。`.`・`..`・`...` はいずれの
///   セグメント位置でも構造的に拒否されるため、`out_dir.join(..)` した
///   結果が `out_dir` 外を指す経路は本変更後も存在しない。
fn normalize_asset_path(path: &str) -> Result<String, SsgError> {
    let Some(rest) = path.strip_prefix('/') else {
        return Err(SsgError::UnsafePagePath(path.to_string()));
    };

    if rest.is_empty() || rest.ends_with('/') {
        return Err(SsgError::UnsafePagePath(path.to_string()));
    }

    let segments: Vec<&str> = rest.split('/').collect();
    let (file_name, dir_segments) = segments
        .split_last()
        .expect("split('/') always yields at least one segment");

    for segment in dir_segments {
        if !is_safe_asset_dir_segment(segment) {
            return Err(SsgError::UnsafePagePath(path.to_string()));
        }
    }
    if !is_safe_asset_file_name(file_name) {
        return Err(SsgError::UnsafePagePath(path.to_string()));
    }

    Ok(rest.to_string())
}

/// 任意の (リクエストパス, コンテンツ文字列) 列を `out_dir` 配下へ
/// パス検証付きで静的書き出しする汎用アセット API（イシュー #1119）。
///
/// [`generate_pages`] が `<path>/index.html` 固定の HTML ページ専用なのに
/// 対し、本関数は `sitemap.xml` / `robots.txt` / `404.html` / `healthz` の
/// ような**任意のファイル名**を持つ非 HTML 生成物（あるいは呼び出し側が
/// 既に文字列化済みの HTML）を、`generate_pages` と同じ fail-closed の
/// パス検証系（[`normalize_asset_path`]）を通してから書き出す。
///
/// # 契約
///
/// - コンテンツは無加工で書き出す（`fs::write` 相当 + パス検証のみ）。
///   `Node` 木・`fandhe_frontend_core::render` を経由しないため既定
///   エスケープ（REQ-1）は適用されない。**HTML ページの生成には本 API を
///   使わず [`generate_pages`] を使うこと**。`404.html` のような HTML
///   アセットを書く場合は、呼び出し側が
///   `format!("<!DOCTYPE html>\n{}", fandhe_frontend_core::render(&node))`
///   のようにノード木 API 経由で文字列化してから渡すことを推奨する
///   （`coding-rust.md`「HTML 文字列の直接組み立て禁止」に抵触しないよう、
///   本 API 自身は HTML を組み立てない）。`sitemap.xml` 内の URL 等、
///   コンテンツ内部のエスケープ（XML エスケープ等）は呼び出し側の責務。
/// - `assets` 全件のパスを先に [`normalize_asset_path`] で検証し、正規化後
///   の出力先の重複も検出する。1 件でも不正・重複があれば**ファイルを 1
///   つも書き出さずに**エラーを返す（fail-closed。[`generate_pages`] と
///   同型の全件事前検証）。
/// - `assets` が空なら `Ok(vec![])` を返し、何も書き出さない。
/// - `generate_pages`/`generate`/`generate_with` と本関数を同一 `out_dir`
///   へ併用した場合の呼び出し間の出力衝突（例: アセット `/index.html` と
///   ページ `/`）は検出対象外（重複検出は 1 回の呼び出し内でしか効かない。
///   `crates/docs-site/src/build.rs` の既存注記と同型の caveat）。
///
/// # Errors
///
/// - [`SsgError::UnsafePagePath`][]: いずれかのアセットパスが検証に失敗した。
/// - [`SsgError::DuplicatePagePath`][]: 正規化後の出力先が重複した。
/// - [`SsgError::CreateDir`][]/[`SsgError::WriteFile`][]: I/O エラー。
pub fn generate_assets(
    assets: &[(String, String)],
    out_dir: &Path,
) -> Result<Vec<PathBuf>, SsgError> {
    // fail-closed: generate_pages と同じく、書き出し前に全アセットの
    // パスを検証・重複判定する（部分成功で dist/ を汚さない）。
    let mut relative_paths = Vec::with_capacity(assets.len());
    for (path, _) in assets {
        let relative = normalize_asset_path(path)?;
        if relative_paths.contains(&relative) {
            return Err(SsgError::DuplicatePagePath(path.clone()));
        }
        relative_paths.push(relative);
    }

    let mut written = Vec::with_capacity(assets.len());
    for (relative, (_, content)) in relative_paths.iter().zip(assets.iter()) {
        written.push(write_file(out_dir, relative, content)?);
    }

    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssr::respond;
    use fandhe_frontend_app::demo_items;
    use std::fs;

    // `TempDir` は integration test（`server/tests/three_mode_integration.rs`）
    // と重複実装しない共有ヘルパー。unit test（本モジュール）と integration
    // test は別クレートとしてリンクされ `#[cfg(test)]` アイテムを跨いで
    // 共有できないため、`include!` でソースを直接展開する
    // （`server/tests/support/temp_dir.rs` 参照）。
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/temp_dir.rs"
    ));

    /// 受け入れ条件 2 用フィクスチャ: 一覧 loader が必ず失敗する
    /// （`server/src/ssr.rs` の `FailingListLoader` と同様、ダミー機微文字列
    /// を `Error` に含める）。
    struct FailingListLoader;

    impl Loader for FailingListLoader {
        type Input = ();
        type Output = Vec<Item>;
        type Error = String;

        fn load(&self, _input: &()) -> Result<Vec<Item>, String> {
            Err("db_password=dummy-secret /internal/path".to_string())
        }
    }

    #[test]
    fn generate_writes_index_and_each_item_matching_ssr_bytes() {
        let dir = TempDir::new("basic");
        let written = generate(&dir.0).expect("generate should succeed");

        // ルート表: index.html + demo_items() 件数分の items/{id}/index.html。
        assert_eq!(written.len(), 1 + demo_items().len());

        let index_body = fs::read_to_string(dir.0.join("index.html")).unwrap();
        assert_eq!(index_body, respond("/").unwrap().body);

        for item in demo_items() {
            let path = dir.0.join("items").join(&item.id).join("index.html");
            let body = fs::read_to_string(&path).unwrap();
            assert_eq!(body, respond(&format!("/items/{}", item.id)).unwrap().body);
        }
    }

    #[test]
    fn is_safe_path_segment_rejects_traversal_like_ids() {
        assert!(!is_safe_path_segment(".."));
        assert!(!is_safe_path_segment("../etc/passwd"));
        assert!(!is_safe_path_segment("a/b"));
        assert!(!is_safe_path_segment("a\\b"));
        assert!(!is_safe_path_segment(""));
        assert!(is_safe_path_segment("1"));
        assert!(is_safe_path_segment("item-2_final"));
    }

    #[test]
    fn write_route_rejects_non_200_ssr_response() {
        // 存在しないアイテム id は `respond_with()` が 404 を返す固定ルートで、
        // `write_route` がステータス検証で書き出しを拒否することを固定する
        // （「200 応答ボディをそのまま書き出す」契約のコード上の担保）。
        let dir = TempDir::new("unexpected-status");
        let err = write_route(
            &DemoItemsLoader,
            &DemoItemDetailLoader,
            &dir.0,
            "/items/does-not-exist",
            "items/does-not-exist/index.html",
        )
        .expect_err("404 route should be rejected before writing");

        match err {
            SsgError::UnexpectedStatus { path, status } => {
                assert_eq!(path, "/items/does-not-exist");
                assert_eq!(status, 404);
            }
            other => panic!("expected UnexpectedStatus, got {other:?}"),
        }
        assert!(!dir.0.join("items/does-not-exist/index.html").exists());
    }

    /// 受け入れ条件 2: 一覧 loader が失敗した場合、`generate_with` が
    /// `SsgError::LoaderError` を返し、ファイルを 1 つも書き出さないこと
    /// （fail-closed・部分成功で握りつぶさないことの直接証明）。
    #[test]
    fn generate_with_returns_loader_error_and_writes_nothing_when_list_loader_fails() {
        let dir = TempDir::new("loader-error");
        let err = generate_with(&FailingListLoader, &DemoItemDetailLoader, &dir.0)
            .expect_err("failing list loader should abort the build");

        match err {
            SsgError::LoaderError { path } => assert_eq!(path, "/"),
            other => panic!("expected LoaderError, got {other:?}"),
        }
        assert!(!dir.0.join("index.html").exists());
        assert!(fs::read_dir(&dir.0)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(true));
    }

    /// 受け入れ条件 1: `normalize_page_path` の正常系（多階層・末尾スラッシュ
    /// の有無・ルート）を固定する。
    #[test]
    fn normalize_page_path_accepts_valid_paths() {
        assert_eq!(normalize_page_path("/").unwrap(), "index.html");
        assert_eq!(
            normalize_page_path("/guide/foo/").unwrap(),
            "guide/foo/index.html"
        );
        assert_eq!(
            normalize_page_path("/guide/foo").unwrap(),
            "guide/foo/index.html"
        );
        assert_eq!(normalize_page_path("/about").unwrap(), "about/index.html");
    }

    /// 受け入れ条件 2: `normalize_page_path` の拒否系（先頭 `/` 無し・
    /// `..`・空セグメント・非許可文字）を固定する。
    #[test]
    fn normalize_page_path_rejects_unsafe_paths() {
        for input in [
            "guide/foo",       // 先頭 / なし
            "/../etc",         // .. トラバーサル
            "/guide/..",       // .. トラバーサル
            "//",              // 空セグメント
            "/a//b",           // 中間の空セグメント
            "/guide/foo\\bar", // バックスラッシュ
            "/guide/./foo",    // ドットセグメント
        ] {
            assert!(
                matches!(normalize_page_path(input), Err(SsgError::UnsafePagePath(_))),
                "expected UnsafePagePath for {input:?}"
            );
        }
    }

    /// `SsgError::Display` の文言にダミー機微文字列が含まれないこと
    /// （`Loader::Error` の値を一切参照しない構造の直接証明）。
    #[test]
    fn loader_error_display_does_not_leak_loader_error_details() {
        let err = SsgError::LoaderError {
            path: "/".to_string(),
        };
        let message = err.to_string();
        assert!(!message.contains("db_password"));
        assert!(!message.contains("dummy-secret"));
        assert!(!message.contains("/internal/path"));
        assert!(message.contains('/'));
    }

    /// `is_safe_asset_file_name` の境界値: `.`/`..`/`...` のようなドットのみ
    /// の名前を拒否し、拡張子付き通常名は許可することを固定する
    /// （パストラバーサル対策、イシュー #1119）。
    #[test]
    fn is_safe_asset_file_name_rejects_dot_only_names() {
        assert!(!is_safe_asset_file_name("."));
        assert!(!is_safe_asset_file_name(".."));
        assert!(!is_safe_asset_file_name("..."));
        assert!(!is_safe_asset_file_name(""));
        assert!(!is_safe_asset_file_name("a/b"));
        assert!(!is_safe_asset_file_name("a\\b"));
        assert!(is_safe_asset_file_name("sitemap.xml"));
        assert!(is_safe_asset_file_name("robots.txt"));
        assert!(is_safe_asset_file_name("healthz"));
        assert!(is_safe_asset_file_name("404.html"));
        assert!(is_safe_asset_file_name(".htaccess"));
    }

    /// `is_safe_asset_dir_segment` の境界値: ドット始まりディレクトリ名
    /// （`.well-known` 等）を許可しつつ、ドットのみの名前・`.git`（大文字
    /// 小文字問わず）を拒否することを固定する（イシュー #1137）。
    #[test]
    fn is_safe_asset_dir_segment_allows_dot_leading_dirs_but_rejects_git_and_dot_only() {
        assert!(is_safe_asset_dir_segment(".well-known"));
        assert!(is_safe_asset_dir_segment("assets"));
        assert!(is_safe_asset_dir_segment(".hidden"));

        assert!(!is_safe_asset_dir_segment("."));
        assert!(!is_safe_asset_dir_segment(".."));
        assert!(!is_safe_asset_dir_segment("..."));
        assert!(!is_safe_asset_dir_segment(""));
        assert!(!is_safe_asset_dir_segment(".git"));
        assert!(!is_safe_asset_dir_segment(".GIT"));
        assert!(!is_safe_asset_dir_segment(".Git"));
        assert!(!is_safe_asset_dir_segment("a/b"));
        assert!(!is_safe_asset_dir_segment("a\\b"));
    }

    /// `normalize_asset_path` の正常系（拡張子付き・拡張子なし・ネスト
    /// パス・ドット始まり中間ディレクトリ）を固定する。
    #[test]
    fn normalize_asset_path_accepts_valid_paths() {
        assert_eq!(normalize_asset_path("/sitemap.xml").unwrap(), "sitemap.xml");
        assert_eq!(normalize_asset_path("/robots.txt").unwrap(), "robots.txt");
        assert_eq!(normalize_asset_path("/healthz").unwrap(), "healthz");
        assert_eq!(
            normalize_asset_path("/assets/site.css").unwrap(),
            "assets/site.css"
        );
        // イシュー #1137: RFC 8615 well-known URI 配下への出力を許可する。
        assert_eq!(
            normalize_asset_path("/.well-known/security.txt").unwrap(),
            ".well-known/security.txt"
        );
        assert_eq!(
            normalize_asset_path("/.well-known/acme-challenge/token").unwrap(),
            ".well-known/acme-challenge/token"
        );
    }

    /// `normalize_asset_path` の拒否系（先頭 `/` 無し・`..`・末尾 `/`・
    /// 空セグメント・非許可文字・ドットのみのセグメント・`.git`・ドット
    /// 始まりディレクトリ経由のトラバーサル）を固定する。
    #[test]
    fn normalize_asset_path_rejects_unsafe_paths() {
        for input in [
            "sitemap.xml",           // 先頭 / なし
            "/../etc/passwd",        // .. トラバーサル
            "/a/../b.txt",           // .. トラバーサル
            "/healthz/",             // 末尾スラッシュ（ファイル前提のため非対応）
            "//",                    // 空セグメント
            "/a//b.txt",             // 中間の空セグメント
            "/.",                    // ファイル名がドットのみ
            "/..",                   // ファイル名がドットのみ（トラバーサル）
            "/...",                  // ファイル名がドットのみ
            "/guide/foo\\bar.txt",   // バックスラッシュ
            "/./x.txt",              // 中間セグメントがドットのみ
            "/.../x.txt",            // 中間セグメントがドットのみ
            "/.git/config",          // .git ディレクトリ（defense-in-depth）
            "/.GIT/config",          // .git 大文字小文字非区別
            "/a/.git/hooks",         // ネストした .git ディレクトリ
            "/.well-known/../x.txt", // ドット始まりディレクトリ経由のトラバーサル回帰
        ] {
            assert!(
                matches!(
                    normalize_asset_path(input),
                    Err(SsgError::UnsafePagePath(_))
                ),
                "expected UnsafePagePath for {input:?}"
            );
        }
    }
}

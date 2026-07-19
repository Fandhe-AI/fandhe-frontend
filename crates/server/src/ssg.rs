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
//!
//! # セキュリティ不変条件（OWASP A01 パストラバーサル対策・fail-closed）
//!
//! - 出力ファイルパスは固定ルート表（`/` → `index.html`、
//!   `/items/{id}` → `items/{id}/index.html`）から `out_dir` 配下に限定して
//!   構成する。`Item::id` は `fandhe-frontend-app` の公開フィールドであり loader 由来の
//!   任意の値を持ちうるため、`..`・`/`・`\` を含む id はエラーとして拒否し、
//!   英数字・`-`・`_` のみを許可するホワイトリスト検証を loader 出力の各
//!   `item.id` に対して従来どおり適用する。
//! - loader が解決に失敗した場合（一覧列挙・各ルート描画のいずれでも）は
//!   [`SsgError::LoaderError`] としてビルドを即座に失敗させ、それまでに
//!   書き出したファイルの有無に関わらずエラーを返す（部分成功で握り
//!   つぶさない = fail-closed、設計書 §5）。`Loader::Error` の値自体は
//!   [`SsgError::Display`] にも一切含めない（[`crate::ssr::loader_error_response`]
//!   と同様の非露出契約。`security.md`「機微情報の露出」）。
//! - `unwrap`/`panic!` は使わず、書き込み・検証の失敗はすべて
//!   [`SsgError`]（`Result`）として呼び出し元へ伝える
//!   （`coding-rust.md` のエラー処理規約）。

use crate::ssr::respond_with;
use fandhe_frontend_app::{DemoItemDetailLoader, DemoItemsLoader, Item, Loader};
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

    let file_path = out_dir.join(relative_path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).map_err(|source| SsgError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(&file_path, response.body.as_bytes()).map_err(|source| SsgError::WriteFile {
        path: file_path.clone(),
        source,
    })?;

    Ok(file_path)
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
}

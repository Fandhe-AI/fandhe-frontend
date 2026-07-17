//! SSG エントリ（TASK-6.1c）: [`crate::ssr::respond`] の 200 応答ボディを
//! そのまま静的ファイルへ書き出す。
//!
//! # 呼び出し文脈・契約
//!
//! - `server/src/bin/ssg.rs`（CLI 版 SSG バイナリ）から呼ばれる。
//! - SSR 出力（[`crate::ssr::SsrResponse::body`]）を**そのまま**ファイルへ
//!   書き出すのみで、独自の HTML 組み立て・独自のエスケープ処理を行わない
//!   （REQ-6: SSR/SSG 出力の文字列完全一致が構成上自明になる。
//!   `docs/app-api.md` 第 4 節・判断 5）。
//! - `std::fs` のみを使用し、外部クレート（`tempfile` 等）を追加しない
//!   （REQ-3、`coding-rust.md`）。
//!
//! # セキュリティ不変条件（OWASP A01 パストラバーサル対策）
//!
//! - 出力ファイルパスは固定ルート表（`/` → `index.html`、
//!   `/items/{id}` → `items/{id}/index.html`）から `out_dir` 配下に限定して
//!   構成する。`Item::id` は `rws-app` の公開フィールドであり将来任意の値を
//!   持ちうるため、`..`・`/`・`\` を含む id はエラーとして拒否し、
//!   英数字・`-`・`_` のみを許可するホワイトリスト検証を行う
//!   （デモデータは数値 id のみだが防御的に実装する）。
//! - `unwrap`/`panic!` は使わず、書き込み・検証の失敗はすべて
//!   [`SsgError`]（`Result`）として呼び出し元へ伝える
//!   （`coding-rust.md` のエラー処理規約）。

use crate::ssr::respond;
use rws_app::demo_items;
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
        }
    }
}

impl std::error::Error for SsgError {}

/// `id` が出力パス片として安全（英数字・`-`・`_` のみ）かを検証する。
///
/// デモデータ（[`rws_app::demo_items`]）はすべて数値 id だが、`Item` は
/// 公開構造体であり将来任意の由来（DB・API 等）のデータを持ちうるため、
/// `..`・`/`・`\` を含む id を機械的に拒否する（OWASP A01）。
fn is_safe_path_segment(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// `out_dir` 配下へ `/` と各 `demo_items()` の詳細ページを静的ファイルとして
/// 書き出す。書き出したファイルの絶対パス一覧を返す。
///
/// `respond()` の 200/404 ボディをそのまま書き出すため、SSR と SSG の出力は
/// 構成上完全一致する（テストは `server/tests/three_mode_integration.rs` で
/// バイト一致を固定）。
pub fn generate(out_dir: &Path) -> Result<Vec<PathBuf>, SsgError> {
    let mut written = Vec::new();

    written.push(write_route(out_dir, "/", "index.html")?);

    for item in demo_items() {
        if !is_safe_path_segment(&item.id) {
            return Err(SsgError::UnsafeItemId(item.id));
        }
        let request_path = format!("/items/{}", item.id);
        let relative = format!("items/{}/index.html", item.id);
        written.push(write_route(out_dir, &request_path, &relative)?);
    }

    Ok(written)
}

/// 1 ルート分を解決して `out_dir/relative_path` へ書き出す共通処理。
fn write_route(
    out_dir: &Path,
    request_path: &str,
    relative_path: &str,
) -> Result<PathBuf, SsgError> {
    let response =
        respond(request_path).ok_or_else(|| SsgError::RouteNotFound(request_path.to_string()))?;

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
    use std::fs;

    /// テスト専用の一時ディレクトリ。`tempfile` は追加せず
    /// `std::env::temp_dir()` + プロセス固有サフィックスで代替する
    /// （`coding-rust.md`: `core` は依存ゼロ、本クレートも外部依存を増やさない）。
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "rws-server-ssg-test-{tag}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            ));
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            // 後片付け失敗はテスト失敗にしない（`fs::remove_dir_all` の
            // エラーを握りつぶす。一時ディレクトリの残留はテスト結果の
            // 正当性に影響しない）。
            let _ = fs::remove_dir_all(&self.0);
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
}

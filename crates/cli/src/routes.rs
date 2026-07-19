//! `fandhe-frontend-router-v1` 組み込み抽出器（TASK-13.1c, #130）と、コンポーネント境界抽出。
//!
//! [`crate::structure`] の `[routing] extractor = "fandhe-frontend-router-v1"` 宣言に対応する
//! 唯一の実装。`structure.toml` の `[routing].definition_dir`（検証済み・
//! `^[a-z0-9_-]+$` を満たすディレクトリ名のみ）配下の `.rs` ファイルを文字列走査し、
//! `server/src/router.rs`（`fandhe-frontend-server`）が実装する `Router::route(path, handler)`
//! 相当の呼び出しからルート文字列を抽出する。
//!
//! PoC-7 が採用していた「マニフェスト由来の任意正規表現をツールが評価する」設計は
//! `docs/design/structure-manifest.md` 2.2.2 節で廃止済み（ReDoS・パターンインジェクション面の
//! 排除）。本抽出器は正規表現クレートを使わず、`.route(` 呼び出しの引数を文字列として
//! 走査するのみで、抽出したパス文字列を実行・評価はしない。
//!
//! # パストラバーサル対策
//!
//! 走査対象ディレクトリは [`crate::structure::is_valid_directory_name`] 相当
//! （呼び出し側で検証済みの `structure.toml` の `directories` キー）からのみ
//! 構成され、ワークスペースルート配下に限定する（[`resolve_within_root`]）。
//! 走査の起点はこの 1 段のみをルート配下に確認するが、再帰走査
//! （[`list_rs_files`]）中に遭遇したシンボリックリンク（ディレクトリ・ファイル
//! いずれも）は辿らず一律スキップする。リンク先を都度 canonicalize してルート
//! 内外を判定するのではなく、リンクそのものを対象外とすることで、
//! シンボリックリンク経由でワークスペースルート外の `.rs` を読み出す経路を
//! 構造的に排除する（OWASP A01/A05 対策）。

use std::path::{Path, PathBuf};

/// 抽出処理の失敗。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractError {
    /// 走査対象ディレクトリがワークスペースルート配下に収まらない
    /// （シンボリックリンク経由の脱出を含む）。
    EscapesWorkspaceRoot,
    /// 走査対象ディレクトリが存在しない、またはディレクトリでない。
    NotADirectory,
    /// ファイル読み込みに失敗した（内部パスは含めず種別のみ記録する）。
    Io(String),
}

impl std::fmt::Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractError::EscapesWorkspaceRoot => {
                write!(f, "scan target escapes workspace root")
            }
            ExtractError::NotADirectory => write!(f, "scan target is not a directory"),
            ExtractError::Io(kind) => write!(f, "I/O error while scanning: {kind}"),
        }
    }
}

impl std::error::Error for ExtractError {}

/// 走査済みの 1 ルート定義。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedRoute {
    pub path: String,
    pub handler: String,
}

/// `workspace_root` 配下の `dir_name`（`structure.toml` で検証済みの
/// `^[a-z0-9_-]+$` ディレクトリ名）を再帰走査し、`.rs` ファイル中の
/// `Router::route(...)` 呼び出しからルートを抽出する。
///
/// `dir_name` はワークスペース相対の単純なディレクトリ名 1 段のみを受け付け、
/// 呼び出し元（`main.rs`）は `structure.toml` の `directories` キーそのものを渡す契約
/// （`../` 等パス区切りを含む値を渡さない。含んでいた場合は
/// [`ExtractError::EscapesWorkspaceRoot`] として拒否する防御を内部に持つ）。
///
/// # Errors
///
/// 走査対象がワークスペースルート外を指す場合・ディレクトリが存在しない場合・
/// I/O に失敗した場合に [`ExtractError`] を返す。
pub fn extract_routes(
    workspace_root: &Path,
    dir_name: &str,
) -> Result<Vec<ExtractedRoute>, ExtractError> {
    let target = scan_root(workspace_root, dir_name)?;
    let mut routes = Vec::new();
    for file in list_rs_files(&target)? {
        let content = std::fs::read_to_string(&file)
            .map_err(|e| ExtractError::Io(format!("{:?}", e.kind())))?;
        routes.extend(extract_routes_from_source(&content));
    }
    Ok(routes)
}

/// 走査ルートを決定する: `workspace_root / dir_name / src` が存在すればそこを、
/// なければ `workspace_root / dir_name` 自体を返す。
///
/// `dir_name` が予約名 [`crate::structure::ROOT_DIR_KEY`]（`root`。クレートが
/// ワークスペースルート直下に直接配置される慣習、`fw new`・イシュー #353）の
/// 場合は [`resolve_within_root`] が `workspace_root` 自身を返すため、
/// 本関数はその配下の `src/`（`<workspace_root>/src`）を走査ルートとして
/// 解決する（通常のディレクトリ名と同じ「`<dir>/src` があればそこを使う」
/// ロジックがそのまま適用される）。
///
/// Cargo の慣例上 `tests/`（integration test）・`benches/`・`examples/` は
/// `src/` の外に置かれる。これらは `#[cfg(test)]` 属性を持たない（cargo が
/// ビルドグラフ自体でテスト専用と扱うため）ので、[`truncate_before_test_cfg`]
/// では除外できない。`src/` に限定して走査することで、製品コードではない
/// フィクスチャ呼び出しを構造的に除外する（AST 精密化の代替としての
/// ディレクトリ規約ベースの簡易対策。`docs/design/structure-manifest.md` §5）。
pub(crate) fn scan_root(workspace_root: &Path, dir_name: &str) -> Result<PathBuf, ExtractError> {
    let target = resolve_within_root(workspace_root, dir_name)?;
    let src = target.join("src");
    if src.is_dir() {
        Ok(src)
    } else {
        Ok(target)
    }
}

/// [`resolve_within_root`] が受け付ける `dir_name`（`/` 区切りマルチセグメント
/// 実配置パスを含む）の書式検証（イシュー #436）。空セグメント（先頭・末尾・
/// 連続 `/`）・段数超過・セグメント文字集合外を fail-closed で拒否する。
fn is_valid_scan_dir_path(dir_name: &str) -> bool {
    if dir_name.is_empty() || dir_name.starts_with('/') || dir_name.ends_with('/') {
        return false;
    }
    let segments: Vec<&str> = dir_name.split('/').collect();
    !segments.is_empty()
        && segments.len() <= crate::structure::MAX_PATH_SEGMENTS
        && segments
            .iter()
            .all(|seg| crate::structure::is_valid_directory_name(seg))
}

/// `workspace_root / dir_name` を解決し、結果が `workspace_root` 配下（シンボリック
/// リンク解決後も含む）に収まることを確認する。収まらない場合・
/// 存在しない場合はエラーを返す（`unsafe` を使わず `std::fs::canonicalize` のみで
/// パストラバーサル面を塞ぐ）。
///
/// `dir_name` が予約名 [`crate::structure::ROOT_DIR_KEY`] の場合は
/// `workspace_root` 自身を候補パスとする（`workspace_root / root` という
/// 実在しないパスへ解決してしまうことを防ぐ、イシュー #353）。この場合も
/// 「`workspace_root` 配下に収まる」チェックは自明に成立するが、`is_dir` の
/// 確認は引き続き通す。
pub(crate) fn resolve_within_root(
    workspace_root: &Path,
    dir_name: &str,
) -> Result<PathBuf, ExtractError> {
    // `dir_name` は単純な 1 段ディレクトリ名（従来仕様）か、`crates/core` のような
    // `/` 区切りの実配置パス（イシュー #436、`structure.toml` の `path` キー由来）の
    // いずれかを受け付ける。各セグメントは [`crate::structure::is_valid_directory_name`]
    // （`^[a-z0-9_-]+$`）を満たすこと・段数は
    // [`crate::structure::MAX_PATH_SEGMENTS`] 以内であることを二重に検証する
    // （`structure.toml` 側の `validate()` を経由しない誤用を想定した防御。
    // `..` は文字集合外として、絶対パス（先頭 `/`）は空セグメント発生により拒否される）。
    if dir_name != crate::structure::ROOT_DIR_KEY && !is_valid_scan_dir_path(dir_name) {
        return Err(ExtractError::EscapesWorkspaceRoot);
    }

    let candidate = if dir_name == crate::structure::ROOT_DIR_KEY {
        workspace_root.to_path_buf()
    } else {
        let mut candidate = workspace_root.to_path_buf();
        for segment in dir_name.split('/') {
            candidate.push(segment);
        }
        candidate
    };
    let canonical_root = std::fs::canonicalize(workspace_root)
        .map_err(|e| ExtractError::Io(format!("{:?}", e.kind())))?;
    let canonical_candidate = std::fs::canonicalize(&candidate).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ExtractError::NotADirectory
        } else {
            ExtractError::Io(format!("{:?}", e.kind()))
        }
    })?;
    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(ExtractError::EscapesWorkspaceRoot);
    }
    if !canonical_candidate.is_dir() {
        return Err(ExtractError::NotADirectory);
    }
    Ok(canonical_candidate)
}

/// `dir` 配下の `.rs` ファイルを再帰列挙する。深さは実ワークスペース構成
/// （`src/` 1 段程度）を十分超える上限で打ち切り、シンボリックリンクによる
/// 循環走査を防ぐ。
pub(crate) fn list_rs_files(dir: &Path) -> Result<Vec<PathBuf>, ExtractError> {
    const MAX_DEPTH: usize = 32;
    let mut out = Vec::new();
    list_rs_files_inner(dir, 0, MAX_DEPTH, &mut out)?;
    Ok(out)
}

fn list_rs_files_inner(
    dir: &Path,
    depth: usize,
    max_depth: usize,
    out: &mut Vec<PathBuf>,
) -> Result<(), ExtractError> {
    if depth > max_depth {
        return Ok(());
    }
    let entries =
        std::fs::read_dir(dir).map_err(|e| ExtractError::Io(format!("{:?}", e.kind())))?;
    for entry in entries {
        let entry = entry.map_err(|e| ExtractError::Io(format!("{:?}", e.kind())))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| ExtractError::Io(format!("{:?}", e.kind())))?;
        // シンボリックリンク（ディレクトリ・ファイルいずれも）は辿らず無条件に
        // スキップする。[`resolve_within_root`] はスキャンの起点 1 段のみを
        // ルート配下に確認しており、再帰の各段でリンク先を都度 canonicalize すると
        // コストが増す上、`DirEntry::file_type` がリンクを辿らない挙動は
        // プラットフォーム・ファイルシステム依存の詳細に委ねられている
        // （レビュー指摘 #127: symlink 経由でワークスペースルート外の `.rs` を
        // 読み出せてしまう懸念）。`is_symlink()` を明示チェックすることで、
        // 実装詳細に依存せずリンクを一律拒否する（OWASP A01/A05 対策の fail-closed）。
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            list_rs_files_inner(&path, depth + 1, max_depth, out)?;
        } else if file_type.is_file() && path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    Ok(())
}

/// 最初の `#[cfg(test)]` 行より前の部分だけを返す（テストモジュール除外。
/// このリポジトリのコーディング規約上、テストモジュールはファイル末尾に
/// `#[cfg(test)] mod tests { ... }` として置かれる想定に依拠する軽量な前処理）。
///
/// `pub(crate)`: `loaders.rs`（`extract_loader_impls_from_source`、イシュー #353）も
/// 同じ「テストモジュール以降を製品コードの走査対象から除外する」前処理を
/// 必要とするため共有する（重複実装しない）。
pub(crate) fn truncate_before_test_cfg(content: &str) -> &str {
    match content.find("#[cfg(test)]") {
        Some(idx) => &content[..idx],
        None => content,
    }
}

/// 行コメント（`//`・`///`・`//!` いずれも含む）を丸ごと除去した文字列を返す。
/// 文字列リテラル内の `//`（現状のルート定義に現れない想定）は区別しないが、
/// rustdoc の使用例（`.route(...)` を含む説明文）を実ルートと誤認しないための
/// 簡易フィルタとして十分な精度を持つ。
///
/// `pub(crate)`: [`truncate_before_test_cfg`] と同じ理由で `loaders.rs` と共有する。
pub(crate) fn strip_comment_lines(content: &str) -> String {
    content
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// ソース文字列から `.route("<path>", "<handler>")` / `.route("<path>", handler_ident)`
/// 呼び出しを抽出する（正規表現不使用、単純な部分文字列走査）。
///
/// `server/src/router.rs` の `Router::route` は第 2 引数にハンドラ値（文字列
/// リテラルまたは識別子）を取る。本抽出器はフル AST 解析は行わない軽量
/// ヒューリスティックであり、REQ-13 の「機械可読なプロジェクト構造」を大まかに
/// 列挙する用途に限定する（AST 精密化はスコープ外、`docs/design/structure-manifest.md` §5）。
/// ノイズを減らすため以下の 2 点のみ前処理で除外する:
/// - 行コメント（`//` 始まり。rustdoc 例（`///`）も含む）: 使用例・説明文中の
///   `.route(...)` を実ルート定義と誤認しないため（[`strip_comment_lines`]）。
/// - `#[cfg(test)]` 以降: テストモジュール内のフィクスチャ呼び出しは製品の
///   ルート定義ではないため対象外とする（[`truncate_before_test_cfg`]）。
///
/// `impact.rs`（TASK-13.2b, #134）が `fw impact` の `affected_files` と
/// ルート定義の突き合わせ（`affected_routes` の構築）に再利用するため
/// `pub(crate)` として公開する。
pub(crate) fn extract_routes_from_source(content: &str) -> Vec<ExtractedRoute> {
    let filtered = strip_comment_lines(truncate_before_test_cfg(content));
    let content = filtered.as_str();
    let mut routes = Vec::new();
    let bytes = content.as_bytes();
    let needle = b".route(";
    let mut i = 0usize;
    while i + needle.len() <= bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            let after = i + needle.len();
            if let Some((path, handler, consumed)) = parse_route_args(&content[after..]) {
                routes.push(ExtractedRoute { path, handler });
                i = after + consumed;
                continue;
            }
        }
        i += 1;
    }
    routes
}

/// `.route(` の直後（`"<path>", <handler>)` 側）を解析し、パス文字列・ハンドラ表現・
/// 消費バイト数を返す。第 1 引数が文字列リテラルでない、またはカンマ・閉じ括弧が
/// 続かない場合は `None`（`.route(` を含む無関係な呼び出しは黙ってスキップする）。
fn parse_route_args(rest: &str) -> Option<(String, String, usize)> {
    let mut chars = rest.char_indices().peekable();
    // 先頭の空白を読み飛ばす。
    while let Some((_, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
    let (start, c) = chars.next()?;
    if c != '"' {
        return None;
    }
    let mut path = String::new();
    let mut end = start + 1;
    while let Some((idx, ch)) = chars.next() {
        end = idx + ch.len_utf8();
        if ch == '"' {
            break;
        }
        if ch == '\\' {
            // エスケープシーケンスは実行しないが、直後の 1 文字を消費して
            // 誤って閉じ引用符と誤認しないようにする。
            if let Some((idx2, ch2)) = chars.next() {
                end = idx2 + ch2.len_utf8();
                path.push(ch2);
            }
            continue;
        }
        path.push(ch);
    }
    // カンマまでスキップし、第 2 引数を「次の `,` または `)` まで」の生テキストとして
    // 粗く取り出す（識別子・文字列リテラルいずれも対応する軽量実装）。
    while let Some((idx, c)) = chars.peek().copied() {
        if c == ',' {
            end = idx + c.len_utf8();
            chars.next();
            break;
        }
        if c == ')' {
            // 第 2 引数がない呼び出し（このスキーマでは不正だが、抽出はスキップする）。
            return None;
        }
        end = idx + c.len_utf8();
        chars.next();
    }
    let mut handler = String::new();
    let mut depth = 0i32;
    for (idx, ch) in chars.by_ref() {
        end = idx + ch.len_utf8();
        match ch {
            '(' => depth += 1,
            ')' if depth == 0 => break,
            ')' => depth -= 1,
            _ => {}
        }
        if !(ch == ')' && depth < 0) {
            handler.push(ch);
        }
    }
    let handler = handler.trim().trim_matches('"').to_string();
    Some((path, handler, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_simple_route_calls() {
        let src = r#"
            Router::new()
                .route("/", "home")?
                .route("/items/:id", "item_detail")?;
        "#;
        let routes = extract_routes_from_source(src);
        assert_eq!(
            routes,
            vec![
                ExtractedRoute {
                    path: "/".to_string(),
                    handler: "home".to_string(),
                },
                ExtractedRoute {
                    path: "/items/:id".to_string(),
                    handler: "item_detail".to_string(),
                },
            ]
        );
    }

    #[test]
    fn ignores_route_calls_without_string_first_argument() {
        let src = r#".route(path_var, "home")"#;
        assert!(extract_routes_from_source(src).is_empty());
    }

    /// イシュー #353: 予約名 `root`（`crate::structure::ROOT_DIR_KEY`）を
    /// `scan_root` へ渡した場合、`workspace_root` 自身の `src/` を走査すること
    /// （`workspace_root/root/src` という実在しないパスへ解決されないこと）。
    #[test]
    fn scan_root_resolves_root_convention_to_workspace_root_src() {
        let tmp = std::env::temp_dir().join(format!(
            "fw-routes-root-convention-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        let src = tmp.join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("main.rs"), "pub fn main() {}").unwrap();

        let target = scan_root(&tmp, crate::structure::ROOT_DIR_KEY).expect("scan should succeed");
        let canonical_src = std::fs::canonicalize(&src).unwrap();
        assert_eq!(target, canonical_src);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// イシュー #353: `root` 慣習下でもワークスペースルート「全体」を走査対象に
    /// せず `src/` に限定すること（`target/` 等の混入・過検知防止、実装計画
    /// §3.1 の非目標）。
    #[test]
    fn scan_root_root_convention_does_not_scan_entire_workspace_root() {
        let tmp = std::env::temp_dir().join(format!(
            "fw-routes-root-convention-scope-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        let src = tmp.join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("main.rs"), "pub fn main() {}").unwrap();
        // src/ の外（ワークスペースルート直下）に .rs ファイルを置く。
        std::fs::write(tmp.join("build.rs"), "fn main() {}").unwrap();

        let target = scan_root(&tmp, crate::structure::ROOT_DIR_KEY).expect("scan should succeed");
        let files = list_rs_files(&target).expect("scan should succeed");
        assert_eq!(
            files.len(),
            1,
            "must scan only src/, not build.rs at workspace root: {files:?}"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// イシュー #353: `resolve_within_root` に `root` を渡した場合、
    /// パストラバーサル境界検証（ワークスペースルート配下チェック）が
    /// 弱体化していないこと（`workspace_root` 自身は自明にルート配下）。
    #[test]
    fn resolve_within_root_accepts_root_convention_key() {
        let tmp = std::env::temp_dir().join(format!(
            "fw-routes-resolve-root-key-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let resolved =
            resolve_within_root(&tmp, crate::structure::ROOT_DIR_KEY).expect("should resolve");
        assert_eq!(resolved, std::fs::canonicalize(&tmp).unwrap());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn resolve_within_root_rejects_traversal_in_dir_name() {
        let root = std::env::temp_dir();
        let err = resolve_within_root(&root, "../etc").unwrap_err();
        assert_eq!(err, ExtractError::EscapesWorkspaceRoot);
    }

    #[test]
    fn resolve_within_root_rejects_missing_directory() {
        let root = std::env::temp_dir();
        let err = resolve_within_root(&root, "definitely-not-a-real-dir-name").unwrap_err();
        assert_eq!(err, ExtractError::NotADirectory);
    }

    /// レビュー指摘 #127: Medium severity。`resolve_within_root` は走査起点の
    /// 1 段のみをルート配下に確認しており、再帰走査中に遭遇したシンボリック
    /// リンク（ディレクトリ・ファイルいずれも）はチェックなく辿られ得た。
    /// リンク先にワークスペースルート外の `.rs` ファイルを配置しても、
    /// `list_rs_files` の結果に含まれないこと（辿らずスキップされること）を
    /// 確認する。
    #[cfg(unix)]
    #[test]
    fn list_rs_files_does_not_follow_symlinked_directory() {
        let tmp =
            std::env::temp_dir().join(format!("fw-routes-symlink-dir-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let scan_root = tmp.join("scan_root");
        let outside = tmp.join("outside");
        std::fs::create_dir_all(&scan_root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("secret.rs"), "pub fn secret() {}").unwrap();
        std::os::unix::fs::symlink(&outside, scan_root.join("link_to_outside")).unwrap();

        let files = list_rs_files(&scan_root).expect("scan should succeed");
        assert!(
            files.is_empty(),
            "symlinked directory must not be followed: found {files:?}"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// 上記と同様に、シンボリックリンクされた単一ファイル（`.rs` へのリンク）も
    /// 実体ファイルとして読み込まれないこと（`is_file()` を辿らずスキップ）を
    /// 確認する。
    #[cfg(unix)]
    #[test]
    fn list_rs_files_does_not_follow_symlinked_file() {
        let tmp = std::env::temp_dir().join(format!(
            "fw-routes-symlink-file-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        let scan_root = tmp.join("scan_root");
        let outside = tmp.join("outside");
        std::fs::create_dir_all(&scan_root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        let target = outside.join("secret.rs");
        std::fs::write(&target, "pub fn secret() {}").unwrap();
        std::os::unix::fs::symlink(&target, scan_root.join("link.rs")).unwrap();

        let files = list_rs_files(&scan_root).expect("scan should succeed");
        assert!(
            files.is_empty(),
            "symlinked file must not be followed: found {files:?}"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn extract_routes_reads_real_router_source() {
        // 統合的な回帰テスト: このリポジトリの実 `app/src/`（`routes.rs`）から
        // 実際にルートを抽出できること（`fandhe-frontend-router-v1` 抽出器の実体確認）。
        // イシュー #407 でルート定義の正本を server から app へ移設し、
        // `structure.toml` の `[routing] definition_dir` も `"app"` へ追随した
        // （抽出器本体は無改修、文字列走査のまま追随できることの回帰）。
        // `router.rs` 自体の rustdoc 例・`#[cfg(test)]` 内呼び出し・
        // `app/tests/`（integration test、`src/` の外）は対象外
        // （[`scan_root`] が `src/` 配下に限定して走査するため）。
        // このテストバイナリは `crates/cli/` 配下でビルドされるため、2 段の
        // 親ディレクトリでワークスペースルートを得る（イシュー #436）。
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("crates/cli/ has a workspace root two levels up");
        let routes = extract_routes(workspace_root, "crates/app").expect("scan should succeed");
        assert!(routes
            .iter()
            .any(|r| r.path == "/" && r.handler == "AppRoute::List"));
        assert!(routes
            .iter()
            .any(|r| r.path == "/items/:id" && r.handler == "AppRoute::Detail"));
        assert_eq!(routes.len(), 2, "src/ scan should exclude test fixtures");
    }
}

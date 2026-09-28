//! `crates/cli/embedded-examples/<name>/README.md`（`fw new --example`
//! が展開する同梱コピー）に、example ディレクトリの外を指す相対リンクが
//! 紛れ込んでいないことを検証する再発防止テスト（イシュー #3342）。
//!
//! # 背景
//!
//! 正本 `examples/<name>/README.md` は `docs/guides/quickstart.md` や
//! 別 example の README など、リポジトリ内の他ファイルを相対リンクで
//! 参照することがある。しかし `crates/cli/embedded-examples/<name>/` と
//! `fw new --example` の生成先には `docs/` も他の example も存在しないため、
//! こうしたリンクは同梱コピー・生成プロジェクトの双方で必ず切れる。
//!
//! # 規約
//!
//! example ディレクトリの外を指すリンクは、`https://github.com/Fandhe-AI/
//! fandhe-frontend/blob/main/<repo 相対パス>` 形式の絶対 URL で書く
//! （docs サイト上ではサイト内遷移が GitHub への外部リンクに変わるが、
//! イシュー #3342 の方針でこの副作用は許容している）。example ディレクトリ
//! 内に閉じた相対リンク（`./src/main.rs` 等）はそのまま維持してよい。
//!
//! # 正本を直接検査しない理由
//!
//! 正本 `examples/<name>/README.md` と同梱コピー
//! `crates/cli/embedded-examples/<name>/README.md` のバイト一致は
//! `example_publish_copy_drift.rs` が別途保証している。本テストが同梱
//! コピー側だけを検査すれば、その一致性により正本側も間接的に固定される
//! （二重検査を避ける）。
//!
//! # 走査対象の決め方
//!
//! `crates/cli/embedded-examples/` 配下のディレクトリ（`README.md` を除く）
//! を `read_dir` で列挙し、固定リストは持たない。イシュー #3289 の
//! `vercel-ssr` のように後から追加される example も自動で検査対象になる
//! ようにするため（fail-closed: 対象が 0 件ならテスト自体を失敗させる）。

use std::path::{Path, PathBuf};

/// workspace ルート（`cli/` の親の親ディレクトリ）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/cli/ has a workspace root two levels up")
        .to_path_buf()
}

/// Markdown 本文からリンク先（インラインリンク・画像・参照形式のリンク
/// 定義）をすべて抽出する。パーサ相当の複雑な実装は避け、本リポジトリの
/// README で実際に使われている書式（インライン `[text](dest)`・画像
/// `![alt](dest)`・参照形式 `[ref]: dest`）に絞った文字列走査で十分な
/// 検出精度を確保する（外部依存ゼロ方針、REQ-3）。
fn extract_link_destinations(text: &str) -> Vec<String> {
    let mut out = Vec::new();

    // インラインリンク・画像: `](` の直後から対応する `)` までを取り出す。
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b']' && bytes[i + 1] == b'(' {
            let start = i + 2;
            if let Some(rel_end) = text[start..].find(')') {
                let dest = &text[start..start + rel_end];
                out.push(normalize_link_destination(dest));
                i = start + rel_end;
                continue;
            }
        }
        i += 1;
    }

    // 参照形式のリンク定義: 行頭（先頭空白を除く）が `[...]: ` の行。
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix('[') {
            if let Some(close) = rest.find("]: ") {
                let dest_part = rest[close + "]: ".len()..].trim();
                if !dest_part.is_empty() {
                    out.push(normalize_link_destination(dest_part));
                }
            }
        }
    }

    out
}

/// リンク先文字列を正規化する。`<...>` で囲まれた形式のかっこを外し、
/// title が続く形式（`dest "title"`）は最初の空白までを採用する。
fn normalize_link_destination(raw: &str) -> String {
    let raw = raw.trim();
    let raw = raw
        .strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(raw);
    let dest = raw.split_whitespace().next().unwrap_or(raw);
    dest.to_string()
}

/// 1 件のリンク先が「example ディレクトリの外を指す相対リンク」として
/// 違反かどうかを判定する。違反なら `true`。
///
/// 判定ロジック:
/// 1. `#` 以降・`?` 以降を除去する。空になったらページ内アンカーとして
///    スキップする。
/// 2. `http://` / `https://` / `mailto:` はスキップする（絶対 URL・
///    メールリンクは同梱コピーでも壊れない）。
/// 3. その他の `scheme:` 形式（例: `ftp:`）は fail-closed で違反扱いにする。
/// 4. `/` 始まりの絶対パスは、生成プロジェクトのルートを基準にしても
///    解決できないため違反にする。
/// 5. 残りは `/` 区切りの相対パスとして深さを追跡する（`.` と空要素は
///    無視、`..` は深さを 1 減らす、それ以外は 1 増やす）。深さが負に
///    なった時点で「example ディレクトリの外へ出た」とみなし違反にする。
fn is_violation(dest: &str) -> bool {
    let without_fragment = dest.split('#').next().unwrap_or("");
    let without_query = without_fragment.split('?').next().unwrap_or("");
    if without_query.is_empty() {
        // ページ内アンカーのみ、または空リンク。
        return false;
    }

    if without_query.starts_with("http://")
        || without_query.starts_with("https://")
        || without_query.starts_with("mailto:")
    {
        return false;
    }

    // `scheme:` 形式（`http(s)`/`mailto` 以外）は未知の迂回経路として
    // fail-closed に違反扱いにする。Windows ドライブレター（`C:\`）等は
    // 本リポジトリの README では使わない前提。
    if let Some(colon_pos) = without_query.find(':') {
        let scheme = &without_query[..colon_pos];
        if !scheme.is_empty()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
        {
            return true;
        }
    }

    if without_query.starts_with('/') {
        return true;
    }

    let mut depth: i32 = 0;
    for segment in without_query.split('/') {
        match segment {
            "" | "." => continue,
            ".." => {
                depth -= 1;
                if depth < 0 {
                    return true;
                }
            }
            _ => depth += 1,
        }
    }

    false
}

/// テキスト全体を検査し、違反リンク先の一覧を返す（重複除去はしない。
/// 呼び出し元で file 名と結合してエラーメッセージへ含めるため）。
fn violations(text: &str) -> Vec<String> {
    extract_link_destinations(text)
        .into_iter()
        .filter(|dest| is_violation(dest))
        .collect()
}

#[test]
fn embedded_example_readmes_have_no_links_escaping_example_dir() {
    let root = workspace_root();
    let embedded_root = root.join("crates/cli/embedded-examples");

    let mut example_dirs: Vec<PathBuf> = std::fs::read_dir(&embedded_root)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", embedded_root.display()))
        .map(|e| {
            e.unwrap_or_else(|e| panic!("failed to read entry: {e}"))
                .path()
        })
        .filter(|p| p.is_dir())
        .collect();
    example_dirs.sort();

    assert!(
        !example_dirs.is_empty(),
        "crates/cli/embedded-examples/ 配下に example ディレクトリが 1 件も見つからない \
         (fail-closed: 走査対象が空のまま PASS にしない)"
    );

    let mut all_violations: Vec<String> = Vec::new();

    for dir in &example_dirs {
        let example_name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .expect("example directory name must be valid UTF-8")
            .to_string();
        let readme_path = dir.join("README.md");
        assert!(
            readme_path.is_file(),
            "crates/cli/embedded-examples/{example_name}/README.md が存在しない \
             (fail-closed: README を持たない example ディレクトリを許容しない)"
        );

        let text = std::fs::read_to_string(&readme_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", readme_path.display()));

        for dest in violations(&text) {
            all_violations.push(format!(
                "crates/cli/embedded-examples/{example_name}/README.md: {dest}"
            ));
        }
    }

    assert!(
        all_violations.is_empty(),
        "同梱コピーの README に example ディレクトリの外を指す相対リンクがある: \
         {all_violations:?}\n\
         正本 examples/<name>/README.md のリンクを \
         https://github.com/Fandhe-AI/fandhe-frontend/blob/main/<path> 形式に置き換え、\
         同梱コピーへ再同期すること（イシュー #3342）。"
    );
}

#[cfg(test)]
mod violations_unit_tests {
    use super::violations;

    #[test]
    fn detects_relative_links_escaping_example_dir() {
        let text = "\
[a](../../docs/x.md)
[b](../other/README.md)
[c](./a/../../b.md)
[d](/docs/x.md)
[r]: ../x.md
";
        let found = violations(text);
        assert_eq!(found.len(), 5, "expected 5 violations, got {found:?}");
    }

    #[test]
    fn allows_absolute_urls_anchors_and_inner_relative_links() {
        let text = "\
[a](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/x.md)
[b](#anchor)
[c](./src/main.rs)
[d](src/a/../b.rs)
[e](mailto:someone@example.com)
";
        let found = violations(text);
        assert!(found.is_empty(), "expected no violations, got {found:?}");
    }
}

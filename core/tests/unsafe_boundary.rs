//! `unsafe` 境界（REQ-2）の回帰テスト（TASK-2.2a）。
//!
//! `docs/spec/04-requirements.md` の REQ-2 は「コアロジックを safe Rust に収め、
//! `unsafe` は WASM バインディング層・FFI 境界に限定する」ことを受け入れ基準とする。
//! 本テストは workspace ルート `Cargo.toml` の `members` を読み取り、
//! **unsafe 許可リスト**（`wasm-client` / `wasm-full` / `wasm-thin` —
//! 仕様上 unsafe が許容される WASM/FFI 境界クレート）**以外**の全メンバーについて、
//! (1) クレートルート（`lib.rs` / `main.rs`）に `#![forbid(unsafe_code)]` が
//! 存在すること、(2) ソース中にコメントを除いた `unsafe` トークンが
//! 出現しないこと、の 2 点を機械的に担保する。
//!
//! `#![forbid(unsafe_code)]` はクレート内で override 不可能な属性であるため、
//! (1) の存在確認だけでも unsafe 不在の強い保証になる。(2) は forbid 属性の
//! 削除と unsafe 追加が同一 PR で行われた場合の二重チェックとして機能する
//! （ドキュメントコメント中の「unsafe」という語への言及は誤検知しないよう、
//! コメント除去後にトークン走査する）。
//!
//! 将来 `wasm-client` 等の WASM/FFI 境界クレートが追加された場合は、本ファイルの
//! `UNSAFE_ALLOWED_MEMBERS` を更新した上で `docs/unsafe-boundary.md`
//! （TASK-2.2b、#14）に unsafe 使用箇所と安全性根拠（`// SAFETY:`）を追記する。
//!
//! ファイル走査は `CARGO_MANIFEST_DIR`（`core/`）の親（workspace ルート）配下に
//! 限定し、シンボリックリンクは辿らない。

use std::fs;
use std::path::{Path, PathBuf};

/// 仕様上 unsafe の使用が許容される WASM/FFI 境界クレート名の許可リスト。
///
/// このリストに **含まれないメンバー**（`core` 等の safe 域クレート）は
/// `#![forbid(unsafe_code)]` を必須とする。クレート追加時は本リストと
/// `docs/unsafe-boundary.md`（#14）を同時に更新する運用とする。
const UNSAFE_ALLOWED_MEMBERS: &[&str] = &["wasm-client", "wasm-full", "wasm-thin"];

/// workspace ルート（`core/` の親ディレクトリ）の絶対パスを返す。
///
/// `cargo test` は `CARGO_MANIFEST_DIR` に各クレートのマニフェストディレクトリ
/// （本クレートでは `core/`）を設定するため、その親を workspace ルートとみなす。
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("core/ は workspace ルート直下に存在する前提")
        .to_path_buf()
}

/// workspace ルート `Cargo.toml` の `[workspace] members` を素朴にパースする。
///
/// TOML パーサを追加すると REQ-3（依存グラフ上限）・core 外部依存ゼロの
/// 監査対象が増えるため、本テストでは正規表現・TOML クレートに頼らず
/// `members = [...]` 行を文字列処理のみで読み取る。
fn workspace_members() -> Vec<String> {
    let root = workspace_root();
    let manifest = fs::read_to_string(root.join("Cargo.toml"))
        .expect("workspace ルート Cargo.toml の読み取りに失敗した");

    let start = manifest
        .find("members")
        .expect("[workspace] members が Cargo.toml に見つからない");
    let after = &manifest[start..];
    let open = after
        .find('[')
        .expect("members の配列開始 `[` が見つからない");
    let close = after[open..]
        .find(']')
        .expect("members の配列終了 `]` が見つからない");
    let list = &after[open + 1..open + close];

    list.split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// 指定ディレクトリ配下の `*.rs` ファイルを再帰列挙する（シンボリックリンクは辿らない）。
fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        // SAFETY 相当の安全境界: symlink_metadata でシンボリックリンクを検出し辿らない。
        // パストラバーサル・意図しない外部ファイル読み取りを防ぐ。
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// 行コメント（`//`）・ブロックコメント（`/* */`）を除去する簡易フィルタ。
///
/// 文字列リテラル中の `//` は本クレート規模のソースには実質出現しないため、
/// 簡易実装でも偽陰性リスクは小さい。forbid 属性の存在確認を一次判定とし、
/// 本関数によるトークン走査は補助チェックと位置付ける（計画書の方針どおり）。
fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'/') {
            for c2 in chars.by_ref() {
                if c2 == '\n' {
                    out.push('\n');
                    break;
                }
            }
        } else if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut prev = ' ';
            for c2 in chars.by_ref() {
                if prev == '*' && c2 == '/' {
                    break;
                }
                prev = c2;
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// ソース中に `unsafe` トークンが（コメント除去後に）出現するか判定する。
fn contains_unsafe_token(src: &str) -> bool {
    let stripped = strip_comments(src);
    stripped
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|tok| tok == "unsafe")
}

/// メンバー名からクレートディレクトリを解決する。
///
/// `members` の要素はディレクトリ名と一致する前提（本 workspace の命名規約）。
fn member_dir(root: &Path, member: &str) -> PathBuf {
    root.join(member)
}

/// safe 域クレート（許可リスト外のメンバー）のクレートルートに
/// `#![forbid(unsafe_code)]` が存在することを検証する。
///
/// REQ-2 の境界不変条件を回帰的に担保するテスト。`#![forbid(unsafe_code)]` は
/// クレート内で override 不能なため、本チェックのみで unsafe 不在の強い保証になる。
/// wasm 系クレート追加時は `UNSAFE_ALLOWED_MEMBERS` を更新すること。
#[test]
fn safe_domain_crates_forbid_unsafe_code() {
    let root = workspace_root();
    let members = workspace_members();
    assert!(
        !members.is_empty(),
        "workspace members が空。Cargo.toml のパースに失敗している可能性がある"
    );

    for member in &members {
        if UNSAFE_ALLOWED_MEMBERS.contains(&member.as_str()) {
            continue;
        }
        let dir = member_dir(&root, member);
        let candidates = [dir.join("src/lib.rs"), dir.join("src/main.rs")];
        let root_file = candidates.iter().find(|p| p.exists()).unwrap_or_else(|| {
            panic!("クレート `{member}` のクレートルート（lib.rs/main.rs）が見つからない: {dir:?}")
        });
        let src = fs::read_to_string(root_file)
            .unwrap_or_else(|e| panic!("{root_file:?} の読み取りに失敗した: {e}"));
        assert!(
            src.contains("#![forbid(unsafe_code)]"),
            "safe 域クレート `{member}` のクレートルート {root_file:?} に \
             `#![forbid(unsafe_code)]` が見つからない。REQ-2 違反の可能性がある"
        );
    }
}

/// 補助チェック: safe 域クレートのソース全体（コメント除去後）に
/// `unsafe` トークンが一切出現しないことを検証する。
///
/// forbid 属性の削除と unsafe コードの追加が同一 PR で行われた場合の
/// 二重の防御線として機能する（一次判定は上記の forbid 属性存在確認）。
#[test]
fn safe_domain_crates_contain_no_unsafe_token() {
    let root = workspace_root();
    let members = workspace_members();

    for member in &members {
        if UNSAFE_ALLOWED_MEMBERS.contains(&member.as_str()) {
            continue;
        }
        let dir = member_dir(&root, member);
        let src_dir = dir.join("src");
        let mut files = Vec::new();
        collect_rs_files(&src_dir, &mut files);
        assert!(
            !files.is_empty(),
            "クレート `{member}` の src/ 配下に .rs ファイルが見つからない: {src_dir:?}"
        );

        for file in files {
            let content = fs::read_to_string(&file)
                .unwrap_or_else(|e| panic!("{file:?} の読み取りに失敗した: {e}"));
            assert!(
                !contains_unsafe_token(&content),
                "safe 域クレート `{member}` のファイル {file:?} に `unsafe` トークンが \
                 検出された。REQ-2（unsafe は WASM/FFI 境界に限定）違反の可能性がある"
            );
        }
    }
}

/// `core` の外部依存が 0 件のままであることを回帰的に確認する。
///
/// REQ-3（依存グラフ上限）・coding-rust.md「core は外部依存ゼロ」の不変条件。
/// FFI 依存クレート経由での unsafe 持ち込み（PoC-2 脅威モデルの残存リスク）は
/// 依存が増えた時点で本テスト失敗として検知され、#14 のドキュメント監査対象となる。
#[test]
fn core_has_zero_external_dependencies() {
    let root = workspace_root();
    let manifest = fs::read_to_string(root.join("core/Cargo.toml"))
        .expect("core/Cargo.toml の読み取りに失敗した");

    let deps_start = manifest
        .find("[dependencies]")
        .expect("core/Cargo.toml に [dependencies] セクションが見つからない");
    let after = &manifest[deps_start + "[dependencies]".len()..];
    let section_end = after.find("\n[").unwrap_or(after.len());
    let section = after[..section_end].trim();

    assert!(
        section.is_empty(),
        "core/Cargo.toml の [dependencies] が空でない: {section:?}。\
         core は外部依存ゼロが不変条件（依存追加には事前のユーザー承認が必要）"
    );
}

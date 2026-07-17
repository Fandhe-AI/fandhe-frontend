//! `unsafe` 境界（REQ-2）の回帰テスト（TASK-2.2a・#155 で deny 域チェックを追加）。
//!
//! `docs/spec/04-requirements.md` の REQ-2 は「コアロジックを safe Rust に収め、
//! `unsafe` は WASM バインディング層・FFI 境界に限定する」ことを受け入れ基準とする。
//! 本ファイルは 2 段階のポリシーで unsafe 境界を機械的に担保する。
//!
//! 1. **safe 域**（`UNSAFE_ALLOWED_MEMBERS` に **含まれない**全メンバー。`core` 等）:
//!    (1) クレートルート（`lib.rs` / `main.rs`）に `#![forbid(unsafe_code)]` が
//!    存在すること、(2) ソース中にコメントを除いた `unsafe` トークンが
//!    出現しないこと、の 2 点を検証する（`safe_domain_crates_*` テスト）。
//! 2. **deny 域**（`DENY_UNSAFE_FFI_MEMBERS`。`wasm-full` — REQ-11 の
//!    wasm-bindgen/web-sys FFI 境界のため `forbid` ではなく `deny` を採用するが、
//!    自作コード側の `unsafe` は 0 件を CI で強制する、#155）:
//!    (a) クレートルートに `#![deny(unsafe_code)]` が存在すること、
//!    (b) `src/` 配下の全 `.rs` にコメント除去後の `unsafe` トークンが出現しない
//!    こと、(c) `#[allow(unsafe_code)]`／`#![allow(unsafe_code)]`（`cfg_attr` 経由も
//!    含む）による deny の上書きが存在しないこと、の 3 点を検証する
//!    （`ffi_deny_crates_*` テスト）。`UNSAFE_ALLOWED_MEMBERS`
//!    （`wasm-client` / `wasm-thin`）はスコープ外（#155 参照）として引き続き
//!    完全免除のままとし、両リストとも本ファイルの `safe_domain_crates_*` テスト
//!    からは skip する。
//!
//! `#![forbid(unsafe_code)]` はクレート内で override 不可能な属性であるため、
//! safe 域は存在確認だけでも unsafe 不在の強い保証になる。`#![deny(unsafe_code)]`
//! はソース側の `#[allow(unsafe_code)]` で上書き可能なため、deny 域は
//! 属性存在確認・unsafe トークン走査・allow 上書き検出の 3 点を組み合わせて
//! forbid 相当の強制を実現する（ドキュメントコメント中の語句への言及は
//! 誤検知しないよう、いずれもコメント除去後に判定する）。
//!
//! 将来クレートを追加・移行する場合は、本ファイルの `UNSAFE_ALLOWED_MEMBERS` /
//! `DENY_UNSAFE_FFI_MEMBERS` を更新した上で `docs/unsafe-boundary.md`
//! （TASK-2.2b、#14／#155）に unsafe 使用箇所と安全性根拠（`// SAFETY:`）を追記する。
//!
//! ファイル走査は `CARGO_MANIFEST_DIR`（`core/`）の親（workspace ルート）配下に
//! 限定し、シンボリックリンクは辿らない。

use std::fs;
use std::path::{Path, PathBuf};

/// 仕様上 unsafe の使用が許容され、CI 検証を完全免除する WASM/FFI 境界クレート
/// 名の許可リスト（スコープ外・#155 参照）。
///
/// このリストに **含まれないメンバー**（`core` 等の safe 域クレート）は
/// `#![forbid(unsafe_code)]` を必須とする。クレート追加時は本リストと
/// `docs/unsafe-boundary.md`（#14）を同時に更新する運用とする。
const UNSAFE_ALLOWED_MEMBERS: &[&str] = &["wasm-client", "wasm-thin"];

/// `#![deny(unsafe_code)]` を採用しつつ、自作コード側の `unsafe` を CI で
/// forbid 相当に強制する WASM/FFI 境界クレート名のリスト（#155）。
///
/// `wasm-bindgen` 展開コードの内部 `unsafe` と衝突するため `forbid` は
/// 採用しないが、`src/` 配下の自作コードには `unsafe` トークン・
/// `allow(unsafe_code)` による deny の上書きのいずれも許可しない。
/// クレート追加時は本リストと `docs/unsafe-boundary.md` を同時に更新する。
const DENY_UNSAFE_FFI_MEMBERS: &[&str] = &["wasm-full"];

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
/// キーの探索は `find_members_key` により行頭一致で行い、
/// `default-members` 等の複合キーへの部分文字列誤マッチを避ける。
fn workspace_members() -> Vec<String> {
    let root = workspace_root();
    let manifest = fs::read_to_string(root.join("Cargo.toml"))
        .expect("workspace ルート Cargo.toml の読み取りに失敗した");

    let start =
        find_members_key(&manifest).expect("[workspace] members キーが Cargo.toml に見つからない");
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

/// `members` キー（`default-members` 等、前方に別の文字が連結したキーは除外）が
/// 行頭（前後空白許容）から始まり、直後（空白を挟んでよい）に `=` が続く箇所の
/// バイトオフセットを返す。
///
/// 単純な部分文字列探索（`str::find("members")`）では `default-members = [...]`
/// のような行にも誤ってマッチしてしまうため、行単位で先頭一致を確認する。
fn find_members_key(manifest: &str) -> Option<usize> {
    let mut offset = 0;
    for line in manifest.split_inclusive('\n') {
        let trimmed_start = line.trim_start();
        let leading_ws = line.len() - trimmed_start.len();
        if let Some(rest) = trimmed_start.strip_prefix("members") {
            if rest.trim_start().starts_with('=') {
                return Some(offset + leading_ws);
            }
        }
        offset += line.len();
    }
    None
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
/// `//!` ドキュメンテーションコメント中に同一文字列が書かれているだけの
/// 偽陽性を避けるため、コメント除去後のソース（`strip_comments`）に対して
/// 実際の属性構文（行頭で `#![forbid(unsafe_code)]` が閉じている）を確認する。
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
        if UNSAFE_ALLOWED_MEMBERS.contains(&member.as_str())
            || DENY_UNSAFE_FFI_MEMBERS.contains(&member.as_str())
        {
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
            contains_forbid_unsafe_code_attribute(&src),
            "safe 域クレート `{member}` のクレートルート {root_file:?} に \
             実際の属性としての `#![forbid(unsafe_code)]` が見つからない \
             （コメント内の言及のみは無効）。REQ-2 違反の可能性がある"
        );
    }
}

/// コメントを除去したソースの各行が `#![forbid(unsafe_code)]` 属性
/// （空白の入り方に多少の揺れがあっても許容）そのものであるかを検証する。
///
/// `strip_comments` で `//` `/* */` コメントを除去した上で判定するため、
/// `//! ... #![forbid(unsafe_code)] ...` のようなドキュメンテーションコメント
/// 内の文字列の言及だけでは true にならない（クレートルート属性の実効性を担保する）。
fn contains_forbid_unsafe_code_attribute(src: &str) -> bool {
    let stripped = strip_comments(src);
    stripped.lines().any(|line| {
        let normalized: String = line.chars().filter(|c| !c.is_whitespace()).collect();
        normalized == "#![forbid(unsafe_code)]"
    })
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
        if UNSAFE_ALLOWED_MEMBERS.contains(&member.as_str())
            || DENY_UNSAFE_FFI_MEMBERS.contains(&member.as_str())
        {
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

/// ソース中（コメント除去後）に `allow(unsafe_code)` による deny/forbid の
/// 上書きが存在するかを判定する。
///
/// `#[allow(unsafe_code)]`・`#![allow(unsafe_code)]` に加え、`cfg_attr(...,
/// allow(unsafe_code))` のように `cfg_attr` 経由で条件付き付与されるケースも
/// まとめて検出する（空白の入り方に依存しないよう、判定前に空白を除去する）。
/// `#[allow(dead_code, unsafe_code)]` のように他の lint 名とカンマ区切りで
/// 併記された場合も見逃さないよう、`allow(...)` の括弧内をカンマ分割して
/// 各要素が `unsafe_code` と完全一致するかを判定する（部分文字列の完全一致
/// だけを見る単純な `contains` 判定では、この併記パターンを検出できない）。
fn contains_unsafe_code_allow_override(src: &str) -> bool {
    let stripped = strip_comments(src);
    let normalized: String = stripped.chars().filter(|c| !c.is_whitespace()).collect();

    // `allow(` の出現ごとに対応する閉じ括弧までを取り出し、カンマ区切りの
    // lint 名リストの中に `unsafe_code` が単体で含まれるかを確認する。
    let bytes = normalized.as_bytes();
    let mut search_from = 0usize;
    while let Some(rel_start) = normalized[search_from..].find("allow(") {
        let paren_open = search_from + rel_start + "allow(".len() - 1;
        // 対応する閉じ括弧をネスト深度を追いながら探す（`cfg_attr(target_os =
        // "wasm32", allow(unsafe_code))` のように外側にも括弧があるケースを
        // 誤って途中で打ち切らないため）。
        let mut depth = 0i32;
        let mut close_idx = None;
        for (i, &b) in bytes.iter().enumerate().skip(paren_open) {
            match b {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        close_idx = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(close_idx) = close_idx else {
            break;
        };
        let inner = &normalized[paren_open + 1..close_idx];
        if inner.split(',').any(|lint| lint == "unsafe_code") {
            return true;
        }
        search_from = paren_open + 1;
    }
    false
}

/// deny 域クレート（`DENY_UNSAFE_FFI_MEMBERS`）のクレートルートに
/// `#![deny(unsafe_code)]` が実在することを検証する（#155）。
///
/// `wasm-full` は `#[wasm_bindgen]` 展開コードの内部 `unsafe` と衝突するため
/// `forbid(unsafe_code)` ではなく `deny(unsafe_code)` を採用する方針
/// （`wasm-full/src/lib.rs` 冒頭 doc コメント・`docs/unsafe-boundary.md` 第 2 節）。
/// `deny` はソース側の `allow` で上書き可能なため、本テストは
/// `ffi_deny_crates_contain_no_unsafe_token_nor_allow_override` と組み合わせて
/// forbid 相当の強制を構成する一次防御を担う。
#[test]
fn ffi_deny_crates_have_deny_unsafe_code_attribute() {
    let root = workspace_root();
    let members = workspace_members();
    assert!(
        !members.is_empty(),
        "workspace members が空。Cargo.toml のパースに失敗している可能性がある"
    );

    for member in DENY_UNSAFE_FFI_MEMBERS {
        assert!(
            members.iter().any(|m| m == member),
            "DENY_UNSAFE_FFI_MEMBERS のクレート `{member}` が workspace members に \
             見つからない。Cargo.toml との同期を確認すること"
        );
        let dir = member_dir(&root, member);
        let candidates = [dir.join("src/lib.rs"), dir.join("src/main.rs")];
        let root_file = candidates.iter().find(|p| p.exists()).unwrap_or_else(|| {
            panic!("クレート `{member}` のクレートルート（lib.rs/main.rs）が見つからない: {dir:?}")
        });
        let src = fs::read_to_string(root_file)
            .unwrap_or_else(|e| panic!("{root_file:?} の読み取りに失敗した: {e}"));
        let stripped = strip_comments(&src);
        let has_deny_attribute = stripped.lines().any(|line| {
            let normalized: String = line.chars().filter(|c| !c.is_whitespace()).collect();
            normalized == "#![deny(unsafe_code)]"
        });
        assert!(
            has_deny_attribute,
            "deny 域クレート `{member}` のクレートルート {root_file:?} に \
             実際の属性としての `#![deny(unsafe_code)]` が見つからない \
             （コメント内の言及のみは無効）。#155 の CI 強制が退行している可能性がある"
        );
    }
}

/// deny 域クレート（`DENY_UNSAFE_FFI_MEMBERS`）の `src/` 配下全 `.rs` に、
/// (a) コメント除去後の `unsafe` トークンが 0 件、(b) `allow(unsafe_code)`
/// による deny の上書きが 0 件、であることを検証する（#155）。
///
/// (a)+(b) を `ffi_deny_crates_have_deny_unsafe_code_attribute` の attribute
/// 存在確認と組み合わせることで、「属性削除」「allow 上書き」「unsafe 直接追加」
/// のいずれについても CI 失敗となり、forbid(unsafe_code) 相当の強制が成立する。
/// 許容される unsafe は wasm-bindgen/web-sys の依存クレート内部・
/// `#[wasm_bindgen]` マクロ展開の自動生成コードのみであり、いずれもここで
/// 走査する自作ソース（`wasm-full/src/`）には現れない
/// （`docs/unsafe-boundary.md` 第 2 節の許容 FFI 境界の記述を参照）。
#[test]
fn ffi_deny_crates_contain_no_unsafe_token_nor_allow_override() {
    let root = workspace_root();
    let members = workspace_members();

    for member in DENY_UNSAFE_FFI_MEMBERS {
        assert!(
            members.iter().any(|m| m == member),
            "DENY_UNSAFE_FFI_MEMBERS のクレート `{member}` が workspace members に \
             見つからない。Cargo.toml との同期を確認すること"
        );
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
                "deny 域クレート `{member}` のファイル {file:?} に自作コード側の \
                 `unsafe` トークンが検出された。REQ-11 受け入れ基準 2（safe Rust に \
                 収まること）違反の可能性がある（#155）"
            );
            assert!(
                !contains_unsafe_code_allow_override(&content),
                "deny 域クレート `{member}` のファイル {file:?} に \
                 `allow(unsafe_code)` による deny の上書きが検出された。\
                 #![deny(unsafe_code)] の実効性が失われている（#155）"
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

/// `contains_unsafe_code_allow_override` が `allow(unsafe_code)` の単独指定
/// だけでなく、他の lint 名とカンマ区切りで併記された場合（例:
/// `#[allow(dead_code, unsafe_code)]`）も検出できることを確認する回帰テスト。
///
/// 部分文字列 `"allow(unsafe_code)"` の完全一致のみを見る単純な実装だと、
/// この併記パターンで検出漏れが発生する（レビュー指摘、#155）。
#[test]
fn contains_unsafe_code_allow_override_detects_comma_separated_lint_list() {
    assert!(
        contains_unsafe_code_allow_override("#[allow(unsafe_code)]"),
        "単独指定のケースを検出できていない"
    );
    assert!(
        contains_unsafe_code_allow_override("#[allow(dead_code, unsafe_code)]"),
        "unsafe_code が末尾に併記されたケースを検出できていない"
    );
    assert!(
        contains_unsafe_code_allow_override("#[allow(unsafe_code, dead_code)]"),
        "unsafe_code が先頭に併記されたケースを検出できていない"
    );
    assert!(
        contains_unsafe_code_allow_override(
            "#[cfg_attr(target_arch = \"wasm32\", allow(dead_code, unsafe_code))]"
        ),
        "cfg_attr 経由かつ併記のケースを検出できていない"
    );
    assert!(
        !contains_unsafe_code_allow_override("#[allow(dead_code, unsafe_code_typo)]"),
        "unsafe_code に類似する別 lint 名を誤検出している（部分一致の偽陽性）"
    );
    assert!(
        !contains_unsafe_code_allow_override("#[allow(dead_code)]"),
        "unsafe_code を含まない属性を誤検出している"
    );
}

//! 最小埋め込み・フルスタック構成間の「分岐なし」の回帰テスト（TASK-7.3、#58）。
//!
//! `docs/spec/05-tasks.md` TASK-7.3 / `docs/spec/04-requirements.md` REQ-7 は、
//! 「最小埋め込み構成（既存 HTML ページの `<div>` へのマウント）とフルスタック
//! 構成（SSR＋ルーティング）が、コンポーネントロジックに一切分岐を持たない
//! 同一関数を呼び出すこと」を受け入れ基準とする。
//!
//! `core/Cargo.toml` は外部依存ゼロが不変条件（`.claude/rules/coding-rust.md`）
//! であり、`fandhe_frontend_app` / `fandhe_frontend_server` / `fandhe_frontend_wasm_client` を dev-dependency として
//! リンクし挙動を検証することはできない。そのため本ファイルは
//! `core/tests/unsafe_boundary.rs`（TASK-2.2a）と同じ方式 — `std::fs` のみで
//! workspace 内ソースを走査する**静的解析テスト**として実装する。
//!
//! # 責務境界（重複実装しない領域）
//!
//! 挙動面のモード同一性（SSR/SSG/CSR の出力完全一致）は
//! `server/tests/ssr_ssg_parity.rs`（TASK-6.4）・`wasm-client` の doctest
//! （[`render_list_page_html`] 等）で既に固定済み。本テストの責務は
//! 「構成による分岐が構造的に存在しないこと」の固定に限定する。
//!
//! `dist-server` の `force-embed` feature は配布層（HTTP 配信方式）の
//! 埋め込み方式切り替えであり、コンポーネントロジック（`fandhe-frontend-app`）自体の
//! 分岐ではないため、本テストの検証対象には含めない（スコープ外、#58 計画）。
//!
//! # 検証内容
//!
//! 1. `app/src/`（fandhe-frontend-app: コンポーネントロジックの実体）に `cfg(test)` 以外の
//!    構成分岐属性（`#[cfg(...)]` / `cfg!(...)` / `#[cfg_attr(...)]`）が存在
//!    しないこと、および `app/Cargo.toml` に `[features]` セクションが存在
//!    しないこと。
//! 2. フルスタック側（`server/src/ssr.rs`）・最小埋め込み側
//!    （`wasm-client/src/lib.rs`）の双方が、共通契約関数
//!    （[`SHARED_PAGE_FUNCTIONS`]）を `fandhe_frontend_app::` 経由で参照し、かつ
//!    同名関数を自前定義（コンポーネントロジックの重複再実装）していないこと。
//! 3. `templates/embed/embed.html`（最小埋め込みの入口）が、検証 2 で固定した
//!    `wasm-client` の共通経路に接続される `mount_csr` を参照していること。
//! 4. 両呼び出し層に `raw_html(` 呼び出しが出現しないこと（構成別の生 HTML
//!    出し分けという「隠れ分岐」の予防、REQ-1 連動）。

use std::fs;
use std::path::{Path, PathBuf};

/// 三モード（SSR/SSG/CSR）から分岐なく呼ばれることを契約とするページ関数群。
///
/// `app/src/lib.rs` の rustdoc（三モード契約・REQ-6）が定める
/// [`fandhe_frontend_app::list_page`] / [`fandhe_frontend_app::detail_page`] に対応する。
/// フルスタック側・最小埋め込み側の双方がこのリストの**全関数**を
/// `fandhe_frontend_app::` 経由で参照していることを検証 2 で固定する。
const SHARED_PAGE_FUNCTIONS: &[&str] = &["list_page", "detail_page"];

/// workspace ルート（`core/` の親ディレクトリ）の絶対パスを返す。
///
/// `unsafe_boundary.rs` と同じ前提（`cargo test` が `CARGO_MANIFEST_DIR` に
/// 各クレートのマニフェストディレクトリを設定する）に基づく。
fn workspace_root() -> PathBuf {
    // `crates/core/` から 2 段上でワークスペースルートに到達する
    // （イシュー #436、`crates/` 配下移設）。
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/core/ has a workspace root two levels up")
        .to_path_buf()
}

/// 指定ディレクトリ配下の `*.rs` ファイルを再帰列挙する（シンボリックリンクは
/// 辿らない）。`unsafe_boundary.rs::collect_rs_files` の複製。
///
/// integration test は各ファイルが独立バイナリとしてコンパイルされるため
/// テストファイル間でのコード共有ができない（`core/tests/support/` 等への
/// 切り出しは `unsafe_boundary.rs` 側の変更を伴いスコープ外、#58 計画書参照）。
/// パストラバーサル対策としてシンボリックリンクを辿らない方針を含めて複製する。
fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
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
/// `unsafe_boundary.rs::strip_comments` の複製（複製理由は上記
/// [`collect_rs_files`] のコメント参照）。
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

/// 指定ファイルの内容をコメント除去済みで読み取る。
fn read_stripped(path: &Path) -> String {
    let src =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("{path:?} の読み取りに失敗した: {e}"));
    strip_comments(&src)
}

/// HTML コメント（`<!-- ... -->`）を除去する簡易フィルタ。
///
/// Cursor Bugbot 指摘（PR #242, Medium）: `embed_template_entry_point_uses_shared_mount_csr`
/// が HTML 全体（コメントを含む）に対する部分文字列一致で判定していたため、
/// 長い解説コメント内の "mount_csr" という語だけで偽陽性になり得た
/// （実際の `<script type="module">` から import/呼び出しが失われても検知できない）。
/// このフィルタでコメントを除去してから検証することで、実際のスクリプト本文に
/// `mount_csr` が残っていることのみを固定する。
fn strip_html_comments(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    loop {
        match rest.find("<!--") {
            Some(start) => {
                out.push_str(&rest[..start]);
                match rest[start..].find("-->") {
                    Some(end_rel) => {
                        rest = &rest[start + end_rel + "-->".len()..];
                    }
                    None => break,
                }
            }
            None => {
                out.push_str(rest);
                break;
            }
        }
    }
    out
}

/// 識別子境界を考慮して `haystack` が `ident` を含むかを判定する。
///
/// `str::contains` による単純な部分文字列一致では、`list_page` に対して
/// `list_page_extra` のような別識別子も誤って一致してしまう。前後が
/// 識別子構成文字（英数字・`_`）でないことを確認し、意図した識別子のみを
/// 検出する。
fn contains_identifier(haystack: &str, ident: &str) -> bool {
    fn is_ident_char(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'_'
    }

    let bytes = haystack.as_bytes();
    let mut search_from = 0usize;
    while let Some(rel) = haystack[search_from..].find(ident) {
        let start = search_from + rel;
        let end = start + ident.len();
        let before_ok = start == 0 || !is_ident_char(bytes[start - 1]);
        let after_ok = end == bytes.len() || !is_ident_char(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        search_from = start + 1;
    }
    false
}

/// `use fandhe_frontend_app::{ ... };` 形式の named import ブロック（複数行にまたがる場合を
/// 含む）を走査し、`func` が識別子としてブロック内に出現するかを判定する。
///
/// Cursor Bugbot 指摘（PR #242, Medium）:
/// `both_call_sites_reference_shared_app_functions_without_redefining` の
/// 旧実装は `line.contains("use fandhe_frontend_app::") && line.contains(func)` という
/// 1 行完結の判定であったため、`server/src/ssr.rs` のような通常の複数行
/// `use fandhe_frontend_app::{\n    list_page,\n    detail_page,\n};` 形式では
/// `use fandhe_frontend_app::` と `func` が同一行に現れず偽陰性（誤って未参照と判定）に
/// なり得た。`use fandhe_frontend_app::` の出現位置から対応する `;` までを 1 つの
/// import ブロックとして切り出し、改行をまたいで検索する。
fn contains_use_import(stripped: &str, func: &str) -> bool {
    let mut search_from = 0usize;
    while let Some(rel) = stripped[search_from..].find("use fandhe_frontend_app::") {
        let start = search_from + rel;
        let after = &stripped[start..];
        let end = after
            .find(';')
            .map(|i| start + i + 1)
            .unwrap_or(stripped.len());
        let block = &stripped[start..end];
        if contains_identifier(block, func) {
            return true;
        }
        search_from = start + "use fandhe_frontend_app::".len();
    }
    false
}

/// 検証 1: `app/src/`（fandhe-frontend-app）に `cfg(test)` 以外の構成分岐属性が存在せず、
/// かつ `app/Cargo.toml` に `[features]` セクションが存在しないことを確認する。
///
/// `cfg(test)` はテストビルド判定であり最小埋め込み/フルスタックというモード
/// 分岐ではないため唯一の許可対象とする（`#[cfg_attr(test, ...)]` も同様に
/// 許可する）。`feature =` / `target_arch` / `target_os` 等の他の cfg 条件は
/// `#[cfg(...)]` / `cfg!(...)` に加え `#[cfg_attr(...)]` 経由の混入も含めて
/// コンポーネントロジックへのモード分岐混入とみなし失敗させる。
/// `#[cfg_attr(target_arch = "wasm32", path = "...")]` のようなモジュール差し
/// 替えは REQ-7 が防ぎたい分岐そのものであり、`#[cfg(` の部分文字列一致だけ
/// では見逃すため個別に検出する。
#[test]
fn app_component_logic_has_no_mode_branching_cfg() {
    let root = workspace_root();
    let app_src_dir = root.join("crates/app/src");
    let mut files = Vec::new();
    collect_rs_files(&app_src_dir, &mut files);
    assert!(
        !files.is_empty(),
        "app/src/ 配下に .rs ファイルが見つからない: {app_src_dir:?}"
    );

    for file in &files {
        let stripped = read_stripped(file);
        for (idx, line) in stripped.lines().enumerate() {
            let normalized: String = line.chars().filter(|c| !c.is_whitespace()).collect();
            let has_cfg_attr = normalized.contains("#[cfg(")
                || normalized.contains("cfg!(")
                || normalized.contains("#[cfg_attr(");
            if !has_cfg_attr {
                continue;
            }
            let line_no = idx + 1;
            assert!(
                normalized.contains("#[cfg(test)]")
                    || normalized.contains("cfg!(test)")
                    || normalized.contains("#[cfg_attr(test,"),
                "fandhe-frontend-app（コンポーネントロジック）{file:?}:{line_no} に \
                 `cfg(test)` / `cfg_attr(test, ...)` 以外の構成分岐属性が \
                 見つかった: {line:?}。REQ-7 はコンポーネントロジックが \
                 構成間で分岐を持たないことを要求する"
            );
        }
    }

    let app_manifest = fs::read_to_string(root.join("crates/app/Cargo.toml"))
        .expect("app/Cargo.toml の読み取りに失敗した");
    assert!(
        !app_manifest.contains("[features]"),
        "app/Cargo.toml に [features] セクションが存在する。feature フラグによる \
         構成分岐の入口となるため fandhe-frontend-app では持たない契約（REQ-7）"
    );
}

/// 検証 2: フルスタック側（`server/src/ssr.rs`）・最小埋め込み側
/// （`wasm-client/src/lib.rs`）が [`SHARED_PAGE_FUNCTIONS`] の全関数を
/// `fandhe_frontend_app::` 経由で参照し、かつ同名関数を自前定義していないことを確認する。
///
/// 自前定義（`fn list_page` 等）はコンポーネントロジックの重複再実装であり、
/// 将来的に一方だけ実装が変わる「事実上の分岐」の温床になるため検知する。
///
/// イシュー #375（`fandhe-frontend-wasm-client` の Loader 移行）で許容参照形を拡張した:
/// `fandhe_frontend_app::assemble_{func}`（`assemble_list_page`/`assemble_detail_page`、
/// `docs/design/loader-trait-design.md` §3.3 の共通契約ラッパー）経由の参照も
/// `fandhe_frontend_app::{func}` 直参照と同格に許容する。`assemble_*` は fandhe-frontend-app 内部で
/// `{func}` を呼ぶのみで独自ロジックを持たないため、REQ-7 の意図（共通関数
/// 経由・コンポーネントロジックの重複再実装禁止）を弱めない。同時に
/// `assemble_{func}` 自体の自前定義（`fn assemble_list_page(` 等）も
/// 重複再実装として検知対象へ加える。
#[test]
fn both_call_sites_reference_shared_app_functions_without_redefining() {
    let root = workspace_root();
    let ssr_path = root.join("crates/server/src/ssr.rs");
    let wasm_client_path = root.join("crates/wasm-client/src/lib.rs");

    for path in [&ssr_path, &wasm_client_path] {
        let stripped = read_stripped(path);

        for func in SHARED_PAGE_FUNCTIONS {
            let assemble_func = format!("assemble_{func}");

            let referenced_via_fandhe_frontend_app = stripped
                .contains(&format!("fandhe_frontend_app::{func}"))
                || stripped.contains(&format!("fandhe_frontend_app :: {func}"));
            // `use fandhe_frontend_app::{ ... , list_page, ... };` 形式の named import も
            // 呼び出し面として許容する（server/src/ssr.rs の実装形）。複数行に
            // またがる import ブロックも検出するため [`contains_use_import`] を
            // 使う（1 行完結判定による偽陰性の回避、Bugbot 指摘 #2 対応）。
            let imported_via_use = contains_use_import(&stripped, func);

            // `fandhe_frontend_app::assemble_{func}` 直参照・`use fandhe_frontend_app::{ assemble_{func}, ... }`
            // 形式の named import も許容参照形とする（イシュー #375、上記
            // ドキュメンテーションコメント参照）。
            let referenced_via_fandhe_frontend_app_assemble = stripped
                .contains(&format!("fandhe_frontend_app::{assemble_func}"))
                || stripped.contains(&format!("fandhe_frontend_app :: {assemble_func}"));
            let imported_via_use_assemble = contains_use_import(&stripped, &assemble_func);

            assert!(
                referenced_via_fandhe_frontend_app
                    || imported_via_use
                    || referenced_via_fandhe_frontend_app_assemble
                    || imported_via_use_assemble,
                "{path:?} が fandhe_frontend_app::{func}（共通契約関数）も fandhe_frontend_app::{assemble_func}\
                 （共通契約ラッパー）も経由して参照していない。最小埋め込み・フルスタック\
                 双方が同一関数を呼ぶことが REQ-7 の受け入れ基準"
            );

            let self_defined = stripped.contains(&format!("fn {func}("));
            assert!(
                !self_defined,
                "{path:?} が `fn {func}` を自前定義している。コンポーネントロジックの \
                 重複再実装は構成間の事実上の分岐を招くため禁止（REQ-7）"
            );

            let assemble_self_defined = stripped.contains(&format!("fn {assemble_func}("));
            assert!(
                !assemble_self_defined,
                "{path:?} が `fn {assemble_func}` を自前定義している。共通契約ラッパー \
                 （fandhe_frontend_app::{assemble_func}）の重複再実装は構成間の事実上の分岐を招くため \
                 禁止（REQ-7、イシュー #375）"
            );
        }
    }
}

/// 検証 3: `templates/embed/embed.html`（最小埋め込みの入口）が `mount_csr`
/// （検証 2 で固定した `wasm-client` の共通呼び出し面への配線）を参照している
/// ことを確認する。
///
/// 埋め込みテンプレートが独自のロジックを持たず、共通経路（wasm-client →
/// fandhe_frontend_app → fandhe_frontend_core::render）へそのまま接続されていることの固定。
#[test]
fn embed_template_entry_point_uses_shared_mount_csr() {
    let root = workspace_root();
    let embed_path = root.join("templates/embed/embed.html");
    let html = fs::read_to_string(&embed_path)
        .unwrap_or_else(|e| panic!("{embed_path:?} の読み取りに失敗した: {e}"));
    // HTML コメント内の解説文に "mount_csr" という語が出現するため、コメントを
    // 除去してから実際のスクリプト本文のみを検証する（Bugbot 指摘 #1 対応、
    // 偽陽性の回避）。
    let html_without_comments = strip_html_comments(&html);

    assert!(
        html_without_comments.contains("mount_csr"),
        "{embed_path:?} が mount_csr を参照していない。最小埋め込みの入口は \
         wasm-client の共通経路（fandhe_frontend_app 経由）に接続される契約（REQ-7）"
    );
}

/// 検証 4: フルスタック側・最小埋め込み側の呼び出し層に `raw_html(` 呼び出しが
/// 出現しないことを確認する。
///
/// 構成別に生 HTML を出し分ける実装は「隠れた分岐」であり、既定エスケープ
/// 迂回（REQ-1）にもつながるため両方の呼び出し層で禁止する。
#[test]
fn call_sites_do_not_bypass_default_escaping_via_raw_html() {
    let root = workspace_root();
    let ssr_path = root.join("crates/server/src/ssr.rs");
    let wasm_client_path = root.join("crates/wasm-client/src/lib.rs");

    for path in [&ssr_path, &wasm_client_path] {
        let stripped = read_stripped(path);
        assert!(
            !stripped.contains("raw_html("),
            "{path:?} に raw_html( 呼び出しが検出された。構成別の生 HTML \
             出し分けという隠れ分岐・既定エスケープ迂回（REQ-1）を防ぐため禁止"
        );
    }
}

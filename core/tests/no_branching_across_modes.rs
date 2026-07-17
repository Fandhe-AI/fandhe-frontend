//! 最小埋め込み・フルスタック構成間の「分岐なし」の回帰テスト（TASK-7.3、#58）。
//!
//! `docs/spec/05-tasks.md` TASK-7.3 / `docs/spec/04-requirements.md` REQ-7 は、
//! 「最小埋め込み構成（既存 HTML ページの `<div>` へのマウント）とフルスタック
//! 構成（SSR＋ルーティング）が、コンポーネントロジックに一切分岐を持たない
//! 同一関数を呼び出すこと」を受け入れ基準とする。
//!
//! `core/Cargo.toml` は外部依存ゼロが不変条件（`.claude/rules/coding-rust.md`）
//! であり、`rws_app` / `rws_server` / `rws_wasm_client` を dev-dependency として
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
//! 埋め込み方式切り替えであり、コンポーネントロジック（`rws-app`）自体の
//! 分岐ではないため、本テストの検証対象には含めない（スコープ外、#58 計画）。
//!
//! # 検証内容
//!
//! 1. `app/src/`（rws-app: コンポーネントロジックの実体）に `cfg(test)` 以外の
//!    構成分岐属性（`#[cfg(...)]` / `cfg!(...)` / `#[cfg_attr(...)]`）が存在
//!    しないこと、および `app/Cargo.toml` に `[features]` セクションが存在
//!    しないこと。
//! 2. フルスタック側（`server/src/ssr.rs`）・最小埋め込み側
//!    （`wasm-client/src/lib.rs`）の双方が、共通契約関数
//!    （[`SHARED_PAGE_FUNCTIONS`]）を `rws_app::` 経由で参照し、かつ
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
/// [`rws_app::list_page`] / [`rws_app::detail_page`] に対応する。
/// フルスタック側・最小埋め込み側の双方がこのリストの**全関数**を
/// `rws_app::` 経由で参照していることを検証 2 で固定する。
const SHARED_PAGE_FUNCTIONS: &[&str] = &["list_page", "detail_page"];

/// workspace ルート（`core/` の親ディレクトリ）の絶対パスを返す。
///
/// `unsafe_boundary.rs` と同じ前提（`cargo test` が `CARGO_MANIFEST_DIR` に
/// 各クレートのマニフェストディレクトリを設定する）に基づく。
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("core/ は workspace ルート直下に存在する前提")
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

/// 検証 1: `app/src/`（rws-app）に `cfg(test)` 以外の構成分岐属性が存在せず、
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
    let app_src_dir = root.join("app/src");
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
                "rws-app（コンポーネントロジック）{file:?}:{line_no} に \
                 `cfg(test)` / `cfg_attr(test, ...)` 以外の構成分岐属性が \
                 見つかった: {line:?}。REQ-7 はコンポーネントロジックが \
                 構成間で分岐を持たないことを要求する"
            );
        }
    }

    let app_manifest = fs::read_to_string(root.join("app/Cargo.toml"))
        .expect("app/Cargo.toml の読み取りに失敗した");
    assert!(
        !app_manifest.contains("[features]"),
        "app/Cargo.toml に [features] セクションが存在する。feature フラグによる \
         構成分岐の入口となるため rws-app では持たない契約（REQ-7）"
    );
}

/// 検証 2: フルスタック側（`server/src/ssr.rs`）・最小埋め込み側
/// （`wasm-client/src/lib.rs`）が [`SHARED_PAGE_FUNCTIONS`] の全関数を
/// `rws_app::` 経由で参照し、かつ同名関数を自前定義していないことを確認する。
///
/// 自前定義（`fn list_page` 等）はコンポーネントロジックの重複再実装であり、
/// 将来的に一方だけ実装が変わる「事実上の分岐」の温床になるため検知する。
#[test]
fn both_call_sites_reference_shared_app_functions_without_redefining() {
    let root = workspace_root();
    let ssr_path = root.join("server/src/ssr.rs");
    let wasm_client_path = root.join("wasm-client/src/lib.rs");

    for path in [&ssr_path, &wasm_client_path] {
        let stripped = read_stripped(path);

        for func in SHARED_PAGE_FUNCTIONS {
            let referenced_via_rws_app = stripped.contains(&format!("rws_app::{func}"))
                || stripped.contains(&format!("rws_app :: {func}"));
            let imported_via_use = {
                // `use rws_app::{ ... , list_page, ... };` 形式の named import も
                // 呼び出し面として許容する（server/src/ssr.rs の実装形）。
                stripped
                    .lines()
                    .any(|line| line.contains("use rws_app::") && line.contains(func))
            };
            assert!(
                referenced_via_rws_app || imported_via_use,
                "{path:?} が rws_app::{func}（共通契約関数）を経由して参照していない。\
                 最小埋め込み・フルスタック双方が同一関数を呼ぶことが REQ-7 の受け入れ基準"
            );

            let self_defined = stripped.contains(&format!("fn {func}("));
            assert!(
                !self_defined,
                "{path:?} が `fn {func}` を自前定義している。コンポーネントロジックの \
                 重複再実装は構成間の事実上の分岐を招くため禁止（REQ-7）"
            );
        }
    }
}

/// 検証 3: `templates/embed/embed.html`（最小埋め込みの入口）が `mount_csr`
/// （検証 2 で固定した `wasm-client` の共通呼び出し面への配線）を参照している
/// ことを確認する。
///
/// 埋め込みテンプレートが独自のロジックを持たず、共通経路（wasm-client →
/// rws_app → rws_core::render）へそのまま接続されていることの固定。
#[test]
fn embed_template_entry_point_uses_shared_mount_csr() {
    let root = workspace_root();
    let embed_path = root.join("templates/embed/embed.html");
    let html = fs::read_to_string(&embed_path)
        .unwrap_or_else(|e| panic!("{embed_path:?} の読み取りに失敗した: {e}"));

    assert!(
        html.contains("mount_csr"),
        "{embed_path:?} が mount_csr を参照していない。最小埋め込みの入口は \
         wasm-client の共通経路（rws_app 経由）に接続される契約（REQ-7）"
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
    let ssr_path = root.join("server/src/ssr.rs");
    let wasm_client_path = root.join("wasm-client/src/lib.rs");

    for path in [&ssr_path, &wasm_client_path] {
        let stripped = read_stripped(path);
        assert!(
            !stripped.contains("raw_html("),
            "{path:?} に raw_html( 呼び出しが検出された。構成別の生 HTML \
             出し分けという隠れ分岐・既定エスケープ迂回（REQ-1）を防ぐため禁止"
        );
    }
}

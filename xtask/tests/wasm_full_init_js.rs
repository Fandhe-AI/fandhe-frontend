//! `static/wasm-full-init.js`（REQ-11 受け入れ基準 3・イシュー #156）の構造回帰テスト。
//!
//! `xtask/tests/view_transitions_js.rs`（イシュー #61）と同じパターンを踏襲する:
//! CI に Node ランタイムが存在しないため、JS 成果物の検証は本 workspace の
//! `cargo test --workspace` 経路に乗せる構造チェックとして実装する。JS の
//! 実行結果ではなくソーステキストの契約（import 元・呼び出し API・
//! セキュリティ不変条件）を固定する。
//!
//! 実効 LOC（10 行以内、REQ-11 受け入れ基準 3）の機械カウントと CI ゲート化は
//! `xtask/src/check_loc.rs`（本ファイルと同じイシュー #156 のスコープだが
//! 別モジュール）が担う。本テストはあくまで参照グルーのソーステキスト契約のみ
//! を固定し、LOC 計測には関与しない。

use std::path::PathBuf;

/// workspace ルート（`xtask/` の親ディレクトリ）から見た対象ファイルの絶対パスを返す。
fn wasm_full_init_js_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask/ には親ディレクトリ（workspace ルート）が存在する")
        .join("static")
        .join("wasm-full-init.js")
}

fn read_source() -> String {
    std::fs::read_to_string(wasm_full_init_js_path())
        .expect("static/wasm-full-init.js の読み取りに失敗した")
}

#[test]
fn wasm_full_init_js_exists_and_is_readable() {
    let path = wasm_full_init_js_path();
    assert!(
        path.is_file(),
        "static/wasm-full-init.js が存在しない: {}",
        path.display()
    );
    read_source();
}

#[test]
fn wasm_full_init_js_imports_from_same_origin_dist_server_path() {
    let source = read_source();
    // import 元は dist-server が実配信する同一オリジンパスに固定する契約
    // （`dist-server/tests/wasm_assets.rs` が検証する配信 URL と一致させる）。
    // 外部 CDN からの読み込みはサプライチェーン対策（security.md）として禁止する。
    assert!(
        source.contains("from \"/static/wasm/fandhe_frontend_wasm_full.js\""),
        "同一オリジンの /static/wasm/fandhe_frontend_wasm_full.js からの import が見つからない"
    );
    assert!(
        !source.contains("http://") && !source.contains("https://"),
        "外部 URL からの import は禁止（同一オリジンのみ許可）"
    );
}

#[test]
fn wasm_full_init_js_calls_hydrate_as_the_default_entrypoint() {
    let source = read_source();
    // 既定方式（SSR + ハイドレーション、`wasm-full/src/entry.rs` の `hydrate`
    // 契約）を単一呼び出しで用いる参照実装であることを固定する。
    assert!(
        source.contains("{ hydrate }"),
        "`hydrate` の named import が見つからない"
    );
    assert!(
        source.contains("hydrate(\"app\")"),
        "hydrate(\"app\") の呼び出しが見つからない"
    );
    assert!(
        source.contains("await init()"),
        "wasm-bindgen の初期化呼び出し（await init()）が見つからない"
    );
}

#[test]
fn wasm_full_init_js_does_not_build_html_strings_or_bypass_escaping() {
    let source = read_source();
    // XSS 保証を Rust 側（fandhe-frontend-core の既定エスケープ、REQ-1）に閉じたままにする
    // 不変条件。グルー JS 側で DOM 文字列を直接組み立てる経路を持ち込まないことを
    // 固定する（`.claude/rules/code-comment-style.md` セキュリティ不変条件の
    // 明文化と対になる回帰テスト）。
    //
    // 冒頭の `/** ... */` 文脈コメント自体が説明のため対象語を含む
    // （本ファイル参照）ので、判定は実コード（最後の `*/` より後）のみを
    // 対象にする。コメント本文の字面一致による誤検知を避けるため。
    let code_only = source
        .rsplit_once("*/")
        .map(|(_, code)| code)
        .unwrap_or(source.as_str());

    assert!(
        !code_only.contains("innerHTML"),
        "innerHTML への直接代入は禁止（既定エスケープ迂回経路になり得る）"
    );
    assert!(
        !code_only.contains("document.write"),
        "document.write の使用は禁止"
    );
    assert!(
        !code_only.contains("outerHTML"),
        "outerHTML への直接代入は禁止"
    );
}

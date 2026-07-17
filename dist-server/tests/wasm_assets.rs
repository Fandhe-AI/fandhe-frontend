//! WASM ビルドステージ（TASK-10.2b、イシュー #110、`dist-server/build.rs`）の
//! 配信検証。
//!
//! `build.rs` は WASM ステージが実際に埋め込みテーブルへ合流したときのみ
//! `wasm_assets_embedded` cfg を有効にする（`RWS_WASM_BUILD=0` でオプトアウト
//! した場合や、wasm ツールチェーン不在でビルド自体が失敗する経路では
//! 立たない）。本ファイル全体をこの cfg でゲートすることで、
//! - オプトアウトしたジョブ（例: forbid-unsafe。self-hosted で `RUSTFLAGS`
//!   を設定するため WASM ステージ自体を無効化する運用、`.github/workflows/ci.yml`
//!   参照）では本テストは存在しないもの（空バイナリ）として扱われ
//!   `cargo test --workspace` を壊さない
//! - WASM ステージを実行するジョブ（`.github/workflows/ci.yml` の `test`）では
//!   本テストが必ずコンパイル・実行され、REQ-10 条件 3（単一 `cargo build` で
//!   ネイティブ + WASM 双方の成果物が生成される）を静かにスキップせず固定する
//!
//! （`.claude/rules/coding-rust.md`「テストの `#[ignore]` 追加でごまかさない」
//! に対応する、実行時スキップではなくコンパイル時ゲートの選択）。
#![cfg(wasm_assets_embedded)]

use rws_dist_server::routes::route_request;

#[test]
fn wasm_bindgen_js_glue_is_served_with_javascript_content_type() {
    let response = route_request("/static/wasm/rws_wasm_full.js");
    assert_eq!(response.status, 200);
    assert_eq!(response.content_type, "text/javascript; charset=utf-8");
    assert!(!response.body.is_empty());
}

#[test]
fn wasm_binary_is_served_with_wasm_content_type() {
    let response = route_request("/static/wasm/rws_wasm_full_bg.wasm");
    assert_eq!(response.status, 200);
    assert_eq!(response.content_type, "application/wasm");
    assert!(!response.body.is_empty());
    // WASM バイナリのマジックナンバー（`\0asm`）を確認し、空ファイル・破損
    // ファイルの埋め込みを検知する。
    assert_eq!(&response.body[..4], b"\0asm");
}

#[test]
fn unknown_wasm_path_still_returns_404() {
    // 既存のパストラバーサル・未知パス防御を WASM 資産追加後も維持することを
    // 固定する回帰テスト。
    assert_eq!(
        route_request("/static/wasm/does-not-exist.wasm").status,
        404
    );
}

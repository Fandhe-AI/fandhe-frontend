//! CSR 経路の loader 解決層（TASK-CSR-loader・#349、親 #337）。
//!
//! `fandhe_frontend_app::Loader`（#347・イシュー #346 設計確定書）を `fandhe-frontend-wasm-full` から
//! 参照する唯一の入口。**クライアント側で新規データ解決が必要になった場合
//! （将来のクライアント遷移を含む）にのみ**使う経路であり、初期表示
//! （ハイドレーション、`crate::entry` / `crate::hydration`）では呼ばない
//! （`docs/design/loader-trait-design.md` §4・§7.3 の凍結事項: 初期表示は
//! サーバー解決済み状態の注入を再利用し、loader を再実行しない）。
//!
//! `server/src/ssr.rs`（#348）の `loader_error_response`（500 固定文言応答）
//! と同型のパターンを CSR 側にも敷く。`fandhe_frontend_app::Loader::Error` の
//! 値をシグネチャ上一切受け取らないことで、loader 実装が `Error` に内部
//! パス・接続情報等を含めていても本モジュールの出力へ混入する経路が構造的に
//! 存在しない（`security.md`「機微情報の露出」・設計書 §9-5）。
//!
//! DOM（`web_sys`）に一切依存しない純粋層に限定する（`events.rs`/
//! `hydration.rs`/`dom.rs` と同じ 2 層構成方針、native の
//! `cargo test --workspace` からも直接呼べる）。実 DOM への反映は呼び出し元
//! （テスト・将来のクライアント遷移機構）が `fandhe_frontend_core::render` +
//! `set_inner_html` で行う。
//!
//! # `fandhe-frontend-wasm-client` への一本化（イシュー #375、Bugbot 指摘対応）
//!
//! `loader_error_view` / `resolve_list_node` / `resolve_detail_node` は
//! `fandhe-frontend-wasm-client`（`wasm-client/src/lib.rs`）の実装と完全に同一のロジック
//! （fail-closed・`Loader::Error` 値を一切受け取らない構造的保証）であり、
//! `fandhe-frontend-wasm-full` は既に `fandhe-frontend-wasm-client` に workspace path 依存している
//! （`wasm-full/Cargo.toml`）。将来のセキュリティ・挙動修正が本モジュールと
//! `wasm-client` 側の 2 箇所に分岐しないよう、本モジュールは実装を持たず
//! `fandhe-frontend-wasm-client` の公開関数をそのまま再エクスポートする一本化窓口とする。
pub use fandhe_frontend_wasm_client::{loader_error_view, resolve_detail_node, resolve_list_node};

// native テストは integration test（`wasm-full/tests/loader_csr.rs`）に
// 配置する（決定性・型接続・fail-closed・XSS 回帰。実装計画 Step 3）。
// `resolve_list_node`/`resolve_detail_node`/`loader_error_view` は本モジュール
// の公開面のみで検証可能なため、内部実装への特権的アクセスを必要としない。

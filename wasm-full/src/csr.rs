//! CSR 経路の loader 解決層（TASK-CSR-loader・#349、親 #337）。
//!
//! `rws_app::Loader`（#347・イシュー #346 設計確定書）を `rws-wasm-full` から
//! 参照する唯一の入口。**クライアント側で新規データ解決が必要になった場合
//! （将来のクライアント遷移を含む）にのみ**使う経路であり、初期表示
//! （ハイドレーション、`crate::entry` / `crate::hydration`）では呼ばない
//! （`docs/design/loader-trait-design.md` §4・§7.3 の凍結事項: 初期表示は
//! サーバー解決済み状態の注入を再利用し、loader を再実行しない）。
//!
//! `server/src/ssr.rs`（#348）の `loader_error_response`（500 固定文言応答）
//! と同型のパターンを CSR 側にも敷く。`rws_app::Loader::Error` の
//! 値をシグネチャ上一切受け取らないことで、loader 実装が `Error` に内部
//! パス・接続情報等を含めていても本モジュールの出力へ混入する経路が構造的に
//! 存在しない（`security.md`「機微情報の露出」・設計書 §9-5）。
//!
//! DOM（`web_sys`）に一切依存しない純粋層に限定する（`events.rs`/
//! `hydration.rs`/`dom.rs` と同じ 2 層構成方針、native の
//! `cargo test --workspace` からも直接呼べる）。実 DOM への反映は呼び出し元
//! （テスト・将来のクライアント遷移機構）が `rws_core::render` +
//! `set_inner_html` で行う。

use rws_app::{assemble_detail_page, assemble_list_page, Item, Loader};
use rws_core::Node;

/// loader 解決失敗時の fail-closed ビュー（`server/src/ssr.rs::loader_error_response`
/// と同型の構造的保証）。
///
/// **呼び出し元はこの関数へ `Loader::Error` の値を渡さない**（意図的に
/// シグネチャへ含めない）。`Display`/`Debug` を一切経由しないため、loader
/// 実装が `Error` に機微情報を含めていても出力へ混入する経路が型レベルで
/// 存在しない。本文はノード木 API（[`rws_core`]）のみで組み立て、`format!`
/// によるタグ文字列の直接組み立ては行わない（REQ-1）。英語固定文言とする
/// （`.claude/rules/japanese-style.md`「エラーメッセージ・ログ等のユーザー
/// 向け文字列は英語」）。
pub fn loader_error_view() -> Node {
    rws_core::div(
        vec![("data-rws", "csr-error")],
        vec![rws_core::p(
            vec![],
            vec![rws_core::text("Something went wrong. Please try again.")],
        )],
    )
}

/// 一覧画面向け CSR loader 解決。
///
/// `assemble_list_page(loader, &())` の `Ok` はそのまま返し、`Err(_)` は
/// 値に一切触れず [`loader_error_view`] へ変換する（fail-closed、未解決
/// データで描画を続行しない、設計書 §5）。`L::Output` が `Vec<Item>` でない
/// loader を渡すとコンパイルエラーになる（`where` 束縛による型接続、
/// 設計書 §3.4）。
pub fn resolve_list_node<L>(loader: &L) -> Node
where
    L: Loader<Input = (), Output = Vec<Item>>,
{
    match assemble_list_page(loader, &()) {
        Ok(node) => node,
        Err(_) => loader_error_view(),
    }
}

/// 詳細画面向け CSR loader 解決。
///
/// `assemble_detail_page(loader, id)` の `Ok` はそのまま返す。`Output`
/// （`Option<Item>`）が `None`（未知の id、404 相当）の場合は
/// `detail_page(None)` の既存契約どおり描画する（見つからない、を
/// `Error` として扱わない — 設計書 §3.3）。`Err(_)` のみ値に触れず
/// [`loader_error_view`] へ変換する。
pub fn resolve_detail_node<D>(loader: &D, id: &str) -> Node
where
    D: Loader<Input = String, Output = Option<Item>>,
{
    match assemble_detail_page(loader, &id.to_string()) {
        Ok(node) => node,
        Err(_) => loader_error_view(),
    }
}

// native テストは integration test（`wasm-full/tests/loader_csr.rs`）に
// 配置する（決定性・型接続・fail-closed・XSS 回帰。実装計画 Step 3）。
// `resolve_list_node`/`resolve_detail_node`/`loader_error_view` は本モジュール
// の公開面のみで検証可能なため、内部実装への特権的アクセスを必要としない。

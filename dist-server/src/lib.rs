//! `rws-dist-server`: TASK-9.1b。`rws-core` / `rws-app` / `rws-server` を
//! 単一実行ファイルへ統合し、コンパイル時埋め込みの静的アセット（`static/`）と
//! `rws-app` の SSR ページ（`/`・`/items/:id`）を配信する配布用サーバー。
//!
//! # クレート構成
//!
//! - [`mime`]: 拡張子 → `Content-Type` の固定表（`mime_guess` の代替）。
//! - [`assets`]: `build.rs` が生成した埋め込みテーブルの検索層。
//! - [`routes`]: HTTP に依存しないルート解決層（[`routes::route_request`]）。
//!   `rws_server::router::Router`（REQ-7 共通コア）でページを、
//!   `assets::lookup` で静的アセットを解決する。
//! - `main.rs`: hyper 接続処理（本ファイルには含めない。テスト容易性のため
//!   HTTP 層と純粋なルーティング層を分離する）。
//!
//! # 依存構成の理由（REQ-3）
//!
//! `Cargo.toml` のコメント参照。`rust-embed`・`axum` はいずれも依存グラフの
//! 深さ上限（6）を構造的に超過するため採用せず、`hyper` + `hyper-util` +
//! `http-body-util` + `tokio` の直接構成としている。
//!
//! # 既定エスケープ・forbid(unsafe_code) の維持
//!
//! 本クレートは HTML 文字列を独自に組み立てない（`routes` モジュール参照）。
//! `unsafe` も使用しない（`hyper`/`tokio` 自体の内部実装は対象外。
//! `docs/unsafe-boundary.md` の対象は WASM/FFI 境界クレートのみで
//! 本クレートは非該当）。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod assets;
pub mod mime;
pub mod routes;

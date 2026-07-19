//! `fandhe-frontend-server` のパスマッチングルーター公開面（イシュー #407）。
//!
//! # 位置付け
//!
//! ルーターの実体は `fandhe-frontend-app`（[`fandhe_frontend_app::router`]）へ移設した。`fandhe-frontend-app` は
//! `server`・`dist-server`・`wasm-full`・`wasm-client` のいずれからも依存可能な
//! 唯一の層（`structure.toml` の `allowed_dependents` 参照）であり、
//! server（SSR/SSG）・wasm-full（CSR）双方のルート解決エンジンを 1 つに
//! 保つことで、パスマッチング**意味論**のドリフト（末尾スラッシュ・空セグメント
//! 等の扱いの食い違い）を構造的に排除する（`docs/design/route-definition-sharing.md`
//! 案 B-1）。
//!
//! 本モジュールは `fandhe_frontend_app::router` をそのまま再エクスポートするのみであり、
//! `server/tests/router_resolution.rs`・`dist-server` 等の既存呼び出し元が
//! `fandhe_frontend_server::router::{Router, RouterError, Params, RouteMatch}` を
//! 無修正のまま利用継続できるようにする（公開 API パス非破壊）。

pub use fandhe_frontend_app::router::{Params, RouteMatch, Router, RouterError};

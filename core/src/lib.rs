//! `rws-core`: 描画コア（外部依存ゼロ）。
//!
//! フロントエンドフレームワークの中核クレート。ノード木 API・既定エスケープを
//! 提供し、`rws-server`（SSR/SSG）・`rws-wasm-client` / `rws-wasm-full`（CSR）
//! など上位クレートから描画基盤として利用される。
//!
//! # 本クレートの不変条件（REQ-1・REQ-2）
//!
//! - **既定エスケープ**: テキスト補間は必ず [`escape_html`] / [`escape_html_into`]
//!   を経由する。エスケープを迂回できるのは `raw_html()`
//!   （明示的オプトイン API、TASK-1.1b で追加予定）のみとし、新たな迂回経路を作らない。
//! - **`unsafe` コード禁止**: `#![forbid(unsafe_code)]` によりクレート全体で
//!   機械的に禁止する。`unsafe` は WASM バインディング層・FFI 境界に限定され、
//!   本クレートには含まれない。
//! - **外部依存ゼロ**: `Cargo.toml` の `[dependencies]` は常に空を維持する。
#![forbid(unsafe_code)]

mod escape;

pub use escape::{escape_html, escape_html_into};

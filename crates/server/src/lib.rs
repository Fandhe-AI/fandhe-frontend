//! `fandhe-frontend-server`: SSR / SSG / 単一バイナリ配布のいずれのエントリからも
//! 共有される共通コア（TASK-6.1c・TASK-7.2 系）。
//!
//! 本クレートは以下の 3 モジュールを提供する。外部依存ゼロを維持し
//! （`Cargo.toml` 参照）、HTTP ソケット層（TCP リッスン・hyper 処理等）は
//! 一切持たない。実際の HTTP 配信は `fandhe-frontend-dist-server`
//! （`dist-server/src/routes.rs`）が [`ssr::respond`] を呼んで担う
//! （`docs/api/app-api.md` 第 4 節: axum 不採用の実測根拠に基づく設計判断）。
//!
//! - [`router`]: HTTP・HTML を一切知らないパスマッチングルーター
//!   （TASK-7.2 系）。
//! - [`ssr`]: fandhe-frontend-app のページ関数を分岐なく呼び、パスをステータス・
//!   Content-Type・既定エスケープ済み HTML 文字列へ変換する SSR コア
//!   （TASK-6.1c）。`server/src/main.rs`（CLI エントリ）・`fandhe-frontend-dist-server`
//!   （HTTP 配信）の両方から共有される。
//! - [`ssg`]: [`ssr::respond`] の出力をそのままファイルへ書き出す SSG コア
//!   （TASK-6.1c）。`server/src/bin/ssg.rs` から呼ばれる。SSR/SSG が同一
//!   ボディを共有するため、両モードの出力完全一致（REQ-6）が構造的に保証
//!   される。
//!
//! # 既定エスケープの引き継ぎ（REQ-1）
//!
//! 本クレート自身は HTML 文字列を一切組み立てない（`ssr`/`ssg` は
//! `fandhe-frontend-app` の既定エスケープ済み出力をそのまま扱うのみ）。`router` の解決で
//! 得た [`router::Params`] を画面へ出力する際は、呼び出し元が必ず
//! `fandhe_frontend_core::text` / `fandhe_frontend_core::el` の attrs 経由で既定エスケープを通すこと
//! （router の契約はあくまで「生文字列を返す」ことまでであり、エスケープは
//! 行わない）。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod router;
pub mod ssg;
pub mod ssr;

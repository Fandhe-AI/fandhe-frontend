//! `rws-server` の SSR エントリ（TASK-6.1c）。
//!
//! 指定パス（既定 `/`）に対する [`rws_server::ssr::respond`] の結果
//! （ステータス・Content-Type・HTML）を stdout へ出力する std のみの CLI。
//!
//! # スコープ（`docs/api/app-api.md` との乖離の記録）
//!
//! 設計確定書（`docs/api/app-api.md`）第 2 節は「axum / tokio 等のサーバー依存は
//! `server/`（TASK-6.1c）側に隔離する」としていたが、`dist-server/Cargo.toml`
//! の実測（axum は tokio-macros → syn 連鎖で依存グラフ深さ 7〜9 に達し REQ-3
//! に構造的に違反）により、本クレートは HTTP ソケット層を持たない
//! （`docs/api/app-api.md` 第 4 節へ追記済み）。HTTP 配信（実際の TCP リッスン・
//! hyper 処理）は `rws-dist-server`（`dist-server/src/main.rs`）が担い、
//! 本バイナリは「パス文字列 → レスポンス文字列化」のみを提供する。
//!
//! `#![forbid(unsafe_code)]` はクレートルートを跨いで継承されないため、
//! バイナリクレートルートである本ファイルにも明示的に付与する
//! （`dist-server/src/main.rs` と同じ方針）。

#![forbid(unsafe_code)]

use rws_server::ssr::respond;
use std::process::ExitCode;

fn main() -> ExitCode {
    // 最初の引数のみをパスとして扱う（既定 `/`）。CLI フラグ解析ライブラリは
    // 追加しない（REQ-3、外部依存ゼロを維持）。
    let path = std::env::args().nth(1).unwrap_or_else(|| "/".to_string());

    match respond(&path) {
        Some(response) => {
            println!("Status: {}", response.status);
            println!("Content-Type: {}", response.content_type);
            println!();
            println!("{}", response.body);
            if response.status >= 400 {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        None => {
            // 固定ルート表に一致しないパス。内部パス等の機微情報は含めない
            // 固定文言のみを標準エラーへ出力する（`security.md`）。
            eprintln!("rws-server: no route matched path {path:?}");
            ExitCode::FAILURE
        }
    }
}

//! `fandhe-frontend-dist-server`: TASK-9.1b。`fandhe-frontend-core` / `fandhe-frontend-app` / `fandhe-frontend-server` を
//! 単一実行ファイルへ統合し、コンパイル時埋め込みの静的アセット（`static/`）と
//! `fandhe-frontend-app` の SSR ページ（`/`・`/items/:id`）を配信する配布用サーバー。
//!
//! # クレート構成
//!
//! - [`mime`]: 拡張子 → `Content-Type` の固定表（`mime_guess` の代替）。
//! - [`assets`]: 静的アセット配信層。開発 / 本番モード（[`assets::AssetMode`]）
//!   に応じて `static/` ディレクトリからの実行時読み込みと `build.rs` 生成
//!   埋め込みテーブルの完全一致検索を切り替える（TASK-10.1a、イシュー #106）。
//! - [`routes`]: HTTP に依存しないルート解決層（[`routes::route_request`]）。
//!   ページ解決は `fandhe_frontend_server::ssr::respond`（TASK-6.1c の SSR コア）へ委譲し、
//!   静的アセットは `assets::lookup` で解決する。
//! - `main.rs`: hyper 接続処理（本ファイルには含めない。テスト容易性のため
//!   HTTP 層と純粋なルーティング層を分離する）。起動時にアセット配信モードを
//!   1 行ログ出力する（[`assets::active_mode`] 参照）。
//!
//! # 開発 / 本番モード切り替え（TASK-10.1a、イシュー #106）
//!
//! | ビルド条件 | モード | 実行時ファイルシステムアクセス |
//! |-----------|--------|-------------------------------|
//! | `debug_assertions` かつ `not(feature = "force-embed")` | [`assets::AssetMode::DevFilesystem`] | あり（`static/` から読む。リビルドなし反映） |
//! | release、または `force-embed` フィーチャー有効 | [`assets::AssetMode::Embedded`] | なし（コンパイル時埋め込みテーブルの完全一致検索のみ） |
//!
//! 開発モードのファイルシステム読み込みは「`..`/絶対パス成分の事前拒否 +
//! `fs::canonicalize` 後の `static/` ルート `starts_with` 検査」の二重防御で
//! パストラバーサル不能性（OWASP A01）を維持する（`assets.rs` の `dev_fs`
//! モジュール参照）。リビルドなし反映（REQ-10）は、`dev_fs::lookup` が
//! 毎リクエストごとにディスクの最新内容を読みキャッシュ・メモ化しないこと
//! （`assets.rs` の回帰テスト参照）と、開発モードの静的アセット応答へ
//! `Cache-Control: no-store` を付与してブラウザキャッシュの影響を排除する
//! こと（[`routes::RouteResponse::cache_control`]）の 2 点で実証済み
//! （TASK-10.1b、イシュー #107）。
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
//! `docs/policy/unsafe-boundary.md` の対象は WASM/FFI 境界クレートのみで
//! 本クレートは非該当）。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod assets;
/// TASK-10.4a（イシュー #119、REQ-10）: `benches/rebuild_latency.rs` の判定・
/// サマリ整形ロジック。ベンチ本体からのみ利用する内部ユーティリティのため
/// `#[doc(hidden)]` とし、クレートの公開 API 面には含めない。
#[doc(hidden)]
pub mod bench_support;
pub mod mime;
pub mod routes;
#[cfg(test)]
mod test_scratch;
/// WASM ビルドステージの有効・無効判定（`FANDHE_FRONTEND_WASM_BUILD`）。
/// `wasm_stage_cache` と同様に `build.rs` から `#[path]` でソースレベル共有
/// する（`src/wasm_build_gate.rs` 冒頭コメント参照）。
#[doc(hidden)]
pub mod wasm_build_gate;
/// TASK-10.2c（イシュー #111）: `build.rs` の WASM ビルドステージ キャッシュ
/// 判定ロジック（fingerprint 計算・成果物完全性チェック）。`build.rs` 自身は
/// パッケージ自身の lib を `build-dependencies` にできないため、
/// `src/wasm_stage_cache.rs` を `#[path]` でソースレベル共有し、こちら側
/// （通常のクレートモジュール）では `cargo test` によるユニットテスト対象と
/// する。ベンチ本体からのみ利用する `bench_support` と同様、クレートの公開
/// API 面を汚さないよう `#[doc(hidden)]` とする。
#[doc(hidden)]
pub mod wasm_stage_cache;
/// `build.rs` がワークスペース内ビルドかパッケージ単体ビルド（`cargo publish`
/// の tarball 検証・crates.io 利用者ビルド等）かを判定する純粋関数。
/// `wasm_build_gate`・`wasm_stage_cache` と同様に `build.rs` から `#[path]` で
/// ソースレベル共有する（`src/workspace_detect.rs` 冒頭コメント参照）。
#[doc(hidden)]
pub mod workspace_detect;

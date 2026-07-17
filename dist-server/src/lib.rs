//! `rws-dist-server`: TASK-9.1b。`rws-core` / `rws-app` / `rws-server` を
//! 単一実行ファイルへ統合し、コンパイル時埋め込みの静的アセット（`static/`）と
//! `rws-app` の SSR ページ（`/`・`/items/:id`）を配信する配布用サーバー。
//!
//! # クレート構成
//!
//! - [`mime`]: 拡張子 → `Content-Type` の固定表（`mime_guess` の代替）。
//! - [`assets`]: 静的アセット配信層。開発 / 本番モード（[`assets::AssetMode`]）
//!   に応じて `static/` ディレクトリからの実行時読み込みと `build.rs` 生成
//!   埋め込みテーブルの完全一致検索を切り替える（TASK-10.1a、イシュー #106）。
//! - [`routes`]: HTTP に依存しないルート解決層（[`routes::route_request`]）。
//!   `rws_server::router::Router`（REQ-7 共通コア）でページを、
//!   `assets::lookup` で静的アセットを解決する。
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
//! `docs/unsafe-boundary.md` の対象は WASM/FFI 境界クレートのみで
//! 本クレートは非該当）。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod assets;
pub mod mime;
pub mod routes;

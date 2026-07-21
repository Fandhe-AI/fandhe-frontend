//! `interactive-vt-wasm`: `examples/interactive-view-transitions`
//! （イシュー #503）が同梱する CSR wasm ビルドの薄い glue クレート。
//!
//! # 役割・責務境界
//!
//! `fandhe-frontend-wasm-full`（crates.io バージョン依存。正本は
//! `crates/wasm-full/`）が `#[wasm_bindgen]` エクスポートとして既に定義
//! している `hydrate` / `mount` / `start_router`（`wasm-full/src/entry.rs`）を
//! 再エクスポートするのみで、このクレート自身は HTML 組み立て・DOM 直接
//! 操作・`raw_html()` の呼び出しを一切行わない
//! （`.claude/rules/coding-rust.md`「HTML 文字列の直接組み立て禁止」）。
//!
//! `hydrate`（`AppState` のカウンター・フォーム・動的リストデモ、
//! `id="interactive-root"`）と `start_router`（`layout()` が組む
//! `<div id="app-root">` の一覧・詳細ページ系）は**別系統・別 DOM**である
//! （`wasm-full::entry` の doc 参照）。`static/embed.html` は両方を異なる
//! `root_id` で呼び出す。
//!
//! # 呼び出し元
//!
//! `tools/wasm/build.sh` が `wasm-bindgen --target web` でこのクレートの
//! `.wasm` を後処理し、`static/wasm/fandhe_frontend_wasm_full.js` /
//! `fandhe_frontend_wasm_full_bg.wasm` を生成する（`--out-name
//! fandhe_frontend_wasm_full` で glue クレート名に依存させず、
//! `static/embed.html` の import パスと整合させる）。`static/embed.html` は
//! この glue クレートの存在を意識しない（`hydrate`/`mount`/`start_router`
//! という関数名契約のみに依存する）。
#![deny(unsafe_code)]

// `fandhe-frontend-wasm-full` の `hydrate`/`mount`/`start_router` は
// `#[cfg(target_arch = "wasm32")]` の `entry` モジュール（`wasm-full/src/lib.rs`）
// にのみ存在する。本クレートを誤って native ターゲットで `cargo build`
// された場合に「unresolved import」で失敗するのを避け、意図が伝わる
// 空クレートとして振る舞わせるため、再エクスポート自体を wasm32 に限定する
// （`tools/wasm/build.sh` は常に `--target wasm32-unknown-unknown` を指定する
// ため、実運用の経路には影響しない）。
#[cfg(target_arch = "wasm32")]
pub use fandhe_frontend_wasm_full::entry::{hydrate, mount, start_router};

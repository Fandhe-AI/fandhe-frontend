//! `app-csr-wasm`: `fw new --template app`（イシュー #411）が同梱する CSR
//! wasm ビルドの薄い glue クレート。
//!
//! # 役割・責務境界
//!
//! `vendor/rws-wasm-client`（イシュー #378/#411 で vendor 同梱、正本は
//! `wasm-client/`）が `#[wasm_bindgen]` エクスポートとして既に定義している
//! `hydrate` / `mount_csr`（`wasm-client/src/lib.rs` の `wiring` モジュール）を
//! 再エクスポートするのみで、このクレート自身は HTML 組み立て・DOM 直接
//! 操作・`raw_html()` の呼び出しを一切行わない
//! （`.claude/rules/coding-rust.md`「HTML 文字列の直接組み立て禁止」）。
//!
//! `#[wasm_bindgen]` proc マクロが生成する記述子（wasm-bindgen CLI が読む
//! カスタムセクション）は `rws-wasm-client` 側のコンパイル単位で生成され、
//! 本クレートを cdylib としてリンクする際にそのまま最終 `.wasm` へ含まれる
//! （`wasm-full`/`wasm-client` 間の "describe" シンボル重複を避ける設計
//! （`wasm-client/src/lib.rs` 冒頭コメント参照）が、シンボルはクレート単位で
//! 重複しない限り問題なくリンクされることの根拠）。
//!
//! # 呼び出し元
//!
//! `tools/wasm/build.sh` が `wasm-bindgen --target web` でこのクレートの
//! `.wasm` を後処理し、`static/wasm/rws_wasm_client.js` /
//! `rws_wasm_client_bg.wasm` を生成する（`--out-name rws_wasm_client` で
//! glue クレート名に依存させず、`static/embed.html` の import パスと整合
//! させる）。`static/embed.html` はこの glue クレートの存在を意識しない
//! （`mount_csr`/`hydrate` という関数名契約のみに依存する）。
#![deny(unsafe_code)]

// `rws-wasm-client` の `hydrate`/`mount_csr` は wasm32 配線層
// （`#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen-exports"))]`）
// にのみ存在する。本クレートを誤って native ターゲットで `cargo build`
// された場合に「unresolved import」で失敗するのを避け、意図が伝わる
// 空クレートとして振る舞わせるため、再エクスポート自体を wasm32 に限定する
// （`tools/wasm/build.sh` は常に `--target wasm32-unknown-unknown` を指定する
// ため、実運用の経路には影響しない）。
#[cfg(target_arch = "wasm32")]
pub use rws_wasm_client::{hydrate, mount_csr};

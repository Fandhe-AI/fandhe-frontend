#![deny(unsafe_code)]

//! `fandhe-frontend-animation`: Web アニメーションアダプタ。
//!
//! # 役割
//!
//! `fandhe-animation`（プラットフォーム非依存のアニメーション演算基幹）が定義する
//! `Driver`/`Target`/`Interpolate` 等の trait を、wasm-bindgen / web-sys / js-sys を
//! 用いて requestAnimationFrame・DOM・Web Animations API 上に実装する層である。
//!
//! # 責務境界
//!
//! - **演算**（イージング・補間・spring・keyframes・timeline の計算）は
//!   `fandhe-animation` の責務であり、本クレートは実装しない
//! - **`data-*` 属性のハイドレーション配線**（利用者が書いた `data-*` 属性から
//!   本クレートの API を呼び出す統合）は `fandhe-frontend-wasm-full` の責務であり、
//!   本クレートはその配線を持たない
//!
//! 3 層構成の詳細は `docs/design/motion-reference-adoption-policy.md` §6・
//! `docs/design/animation-core-architecture.md` §2/§6 を参照。
//!
//! # 依存方向
//!
//! `fandhe-animation ← fandhe-frontend-animation ← wasm-full(optional)`。
//! 本クレートは `fandhe-frontend-wasm-full`/`-wasm-client`/`-wasm-thin` の
//! いずれにも依存しない独立クレートであり、`crates/wasm-full/` が optional
//! 依存として取り込む配線層を担う（`Cargo.toml` の optional 依存は既定 feature
//! では有効化されないため、`fw structure`/`cargo metadata` の既定解決には
//! 現れない。`crates/xtask/tests/wasm_full_animation_optional_dep.rs` が
//! この不変条件を機械固定する）。
//!
//! # 不変条件
//!
//! 1. **自作コード側の `unsafe` は 0 件**（`#![deny(unsafe_code)]`。wasm-bindgen
//!    展開コード内部の `unsafe` と衝突するため `forbid` ではなく `deny` を採用する
//!    ——`wasm-client`/`wasm-full` と同方針、`docs/policy/unsafe-boundary.md` 参照）
//! 2. `fandhe-frontend-wasm-full`/`-wasm-client`/`-wasm-thin` へ依存しない
//!
//! # 現状
//!
//! rAF Driver（[`raf_driver::RafDriver`]）・DOM Target
//! （[`dom_target::DomTarget`]）を実装済み（イシュー #2403/#2517）。
//! WAAPI / FLIP / SVG path / scroll 等の実装は Phase 4 の各後続 issue で
//! 追加する（`docs/design/animation-core-architecture.md` §6.2 参照）。

pub mod dom_target;
pub mod raf_driver;

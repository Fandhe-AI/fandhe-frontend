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
//! 依存として取り込む配線層を担う（`Cargo.toml` の optional 依存は
//! `wasm-full` 側の `"animate"`/`"animation-driver"` feature（いずれも
//! 既定 on）が有効化するため、`fw structure`/`cargo metadata` の既定解決
//! にも現れる。`crates/xtask/tests/wasm_full_animation_optional_dep.rs` が
//! 既定 feature・`--no-default-features`・`--all-features` の 3 通りで
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
//! `animate`（`element.animate()` WAAPI 薄いラッパ、イシュー #2398）も
//! 実装済み。scroll ドライバ（[`scroll_driver`]、`animation-timeline`
//! 委譲の機能検出 + rAF フォールバックの計算・計測プリミティブ、イシュー
//! #2521）も実装済み。layout FLIP アニメーション（[`flip`]、イシュー
//! #2518）も実装済み。pointer capture ベースの汎用ドラッグ演算
//! （[`drag::DragController`]、軸制約・範囲クランプ・離脱速度推定 +
//! spring 復帰、イシュー #2535）・`prefers-reduced-motion` 判定ヘルパ
//! （[`reduced_motion::prefers_reduced_motion`]）も実装済み。confetti
//! （[`canvas_target::CanvasTarget`]・[`confetti::fire`]、canvas 2D
//! 描画によるパーティクル発火、イシュー #2533）も実装済み。SVG path
//! drawing（[`svg_path::draw_path`]、`getTotalLength()` +
//! `stroke-dashoffset` の WAAPI アニメーション、イシュー #2519）も
//! 実装済み。magnetic（[`magnetic::compute_pull`]・
//! [`magnetic::write_offset`]、ポインタ追従オフセットの計算・CSS
//! カスタムプロパティ書き込み、イシュー #2550）も実装済み。残りの実装は
//! Phase 4 の各後続 issue で追加する
//! （`docs/design/animation-core-architecture.md` §6.2 参照）。

pub mod animate;
pub mod canvas_target;
pub mod confetti;
pub mod dom_target;
pub mod drag;
pub mod flip;
pub mod magnetic;
pub mod raf_driver;
pub mod reduced_motion;
pub mod scroll_driver;
pub mod svg_path;

// `fandhe-animation`（演算基幹）の型（`Keyframes`/`Keyframe` 等）は、本クレートの
// `[dependencies]` にのみ存在し推移依存としてアプリの名前解決に公開されない。
// `crates/wasm-full/src/lib.rs` の `pub use fandhe_frontend_animation;`（`animate`
// feature 有効時）と同じ理由（構造グラフ〔`structure.toml`〕へ新規の直接依存
// エッジを追加せず、既存の縦の依存方向 `fandhe-animation ← fandhe-frontend-
// animation` を経由して型へアクセスできるようにする）で crate 自体を
// 再エクスポートする。
pub use fandhe_animation;

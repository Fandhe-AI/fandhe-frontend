//! `fandhe-frontend-animation`（Web アニメーションアダプタ）を利用可能に
//! する配線点（イシュー #2403/#2517、`feature = "animation-driver"`）。
//!
//! # 責務境界
//!
//! 本モジュールは `fandhe-frontend-animation` の公開型を薄く再公開する
//! のみで、アニメーション演算（`fandhe-animation` の責務）・DOM/Web
//! Animations API への適用ロジック（`fandhe-frontend-animation` の責務）は
//! 一切実装しない（`docs/design/animation-core-architecture.md` の層責務、
//! `docs/guides/wasm-full-features.md` §11.2）。`data-*` 属性から本クレート
//! の API を呼び出すハイドレーション配線（stagger 配線・animate 等）は
//! 後続 issue（親トラッキング #2508）が本モジュールの上に追加する。

pub use fandhe_frontend_animation::dom_target::DomTarget;
pub use fandhe_frontend_animation::raf_driver::{AnimationLoop, RafDriver};

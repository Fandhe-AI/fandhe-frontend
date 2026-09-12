#![forbid(unsafe_code)]

//! `fandhe-animation`: プラットフォーム非依存のアニメーション演算基幹。
//!
//! # 役割
//!
//! 値・イージング・補間・spring・keyframes・timeline の**計算のみ**を担う。
//! DOM・requestAnimationFrame・Web Animations API には一切触れない。
//!
//! # 責務境界
//!
//! - フレームループの駆動・DOM への書き込みは `fandhe-frontend-animation`
//!   （Web アダプタ、イシュー #2417）が担う
//! - `data-*` 属性のハイドレーション配線は `fandhe-frontend-wasm-full` の責務
//!
//! 3 層構成の詳細は `docs/design/motion-reference-adoption-policy.md` §6・
//! `docs/design/animation-core-architecture.md` §2/§6 を参照。
//!
//! # 不変条件
//!
//! 1. `#![forbid(unsafe_code)]`（REQ-2）
//! 2. `[dependencies]` は常に空（REQ-3、外部依存ゼロ）
//! 3. `fandhe-frontend-*` へも依存しない（将来の別リポジトリ切り出し前提、
//!    `docs/design/animation-core-architecture.md` §5）
//!
//! # 依存方向
//!
//! `fandhe-animation ← fandhe-frontend-animation ← wasm-full(optional)`。
//! 本クレートの唯一の依存元は `fandhe-frontend-animation` である。
//!
//! # 現状
//!
//! 雛形（イシュー #2371）。公開 API はまだ無い。モジュール骨格
//! （`easing` / `interpolate` / `spring` / `keyframes` / `timeline` /
//! `driver` / `target`）はイシュー #2373〜#2378 で追加予定。

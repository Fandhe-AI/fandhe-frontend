#![forbid(unsafe_code)]

//! `fandhe-frontend-wireframe-ui`: blocks.pm 参照のローファイ・モノクロ
//! ワイヤーフレーム UI コンポーネント層。
//!
//! # 役割
//!
//! 画面設計初期段階での配置イメージ提示に特化した、非インタラクティブな
//! 表示専用プレースホルダー部品（計 49 部品、blocks.pm 由来 35 + 追加 14）
//! を提供する。詳細は `docs/design/wireframe-ui-architecture.md` を参照。
//!
//! # 責務境界
//!
//! - `fandhe-frontend-headless-ui`（Primitives）・`fandhe-frontend-pre-styled-ui`
//!   （Themes）とは独立した第 3 の UI コンポーネント層であり、両者への
//!   依存・両者からの依存のいずれも持たない
//! - 依存は `fandhe-frontend-core` のみ（外部依存ゼロ）
//! - **SSR 専用**。`wasm-full` 配線（ハイドレーション・インタラクション）
//!   は行わない。REQ-11 の gzip 計測（wasm-full/dist-server 経路）には
//!   非影響
//! - 全部品は非インタラクティブな表示専用プレースホルダーとする。対話的な
//!   WAI-ARIA セマンティクス（`role`/`aria-expanded`/`aria-haspopup` 等）・
//!   `tabindex`・キーボードイベントハンドラ・フォーカス管理・状態遷移は
//!   一切実装しない。ネイティブに対話セマンティクスを持つ HTML 要素
//!   （`button`/`input`/`select`/`a[href]` 等）も出力しない
//!
//! # 不変条件
//!
//! 1. `#![forbid(unsafe_code)]`（REQ-2）
//! 2. `[dependencies]` は `fandhe-frontend-core` のみ（REQ-3）
//! 3. テキスト引数は `fandhe-frontend-core` の既定エスケープ経由（REQ-1）
//!
//! # 現状
//!
//! 雛形段階（イシュー #2603）。公開 API は未実装。`Size` 列挙・共通型・
//! `WIREFRAME_CSS` はイシュー #2605、SVG アイコン基盤は #2606、個別部品
//! 実装は Phase 1 以降（#2608〜）で追加する。

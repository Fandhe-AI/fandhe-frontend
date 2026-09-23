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
//! 共通基盤 API 実装済み（イシュー #2605）: [`Size`]・[`Bold`]/[`Primary`]/
//! [`Active`]/[`Disabled`]/[`Orientation`]（共通型）・モノクロトークン
//! （[`tokens`]）・[`wireframe_css`]（CSS 集約出力）・[`class_list`]。
//! SVG アイコン基盤（[`icon`]、イシュー #2606）実装済み: 12 種以上の
//! ラインアートアイコンと `Node` スロット規約（`docs/design/wireframe-ui-architecture.md`
//! §11）。個別部品は Phase 2「テキスト・注釈」の [`annotation`]
//! （イシュー #2617）から実装を開始し、Phase 1「レイアウト骨格」の
//! [`grid`]（イシュー #2611）・[`divider`]（イシュー #2612、
//! `props::Orientation` の最初の消費者）・[`stack`]（イシュー #2610）が
//! 続いた。Phase 2 は [`tag`]（イシュー #2619）が続いた。残りは Phase 1
//! の他部品（frame）・Phase 2 の他部品（text/paragraph/rich-text/link 等）
//! および Phase 3 以降（#2608〜）で順次追加する。
//!
//! # class 命名規約
//!
//! 全 class は [`CLASS_PREFIX`]（`fw-wire-`）で始まる。部品ルートは
//! `fw-wire-<kebab>`、部品内パートは `fw-wire-<kebab>-<part>`、共通修飾
//! （部品名を含まない横断 class）は `fw-wire-size-<xs|sm|md|lg|xl>` /
//! `fw-wire-bold` / `fw-wire-primary` / `fw-wire-horizontal|vertical`。
//! 表示状態は class ではなく `data-active`/`data-disabled` で表す。CSS
//! カスタムプロパティは `--fw-wire-*`（pre-styled-ui の `--fandhe-*` とは
//! 意図的に別プレフィックス）。部品ルートなしで単独使用する唯一の例外的
//! パート class として `fw-wire-icon-glyph`（[`icon`] のグリフ）を持つ。
//! 詳細・追記契約は `docs/design/wireframe-ui-architecture.md` §10 を参照。

pub mod annotation;
pub mod class;
pub mod css;
pub mod divider;
pub mod grid;
pub mod icon;
pub mod props;
pub mod size;
pub mod stack;
pub mod tag;
pub mod tokens;

pub use annotation::annotation;
pub use class::{class_list, CLASS_PREFIX};
pub use css::{wireframe_css, PARTS};
pub use divider::divider;
pub use grid::{grid, MAX_COLUMNS};
pub use props::{Active, Bold, Disabled, Orientation, Primary};
pub use size::Size;
pub use stack::stack;
pub use tag::tag;

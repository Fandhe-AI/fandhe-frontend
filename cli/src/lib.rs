//! `rws-cli`（バイナリ名 `fw`）: AI 自己保守・改修のためのフック・ゲート機構（REQ-13）。
//!
//! `docs/spec/04-requirements.md` REQ-13 が要求する `fw structure` / `fw impact` /
//! `fw gate` の情報源は、対象プロジェクト直下に置かれる機械可読なマニフェスト
//! `structure.toml`（ディレクトリごとの役割・依存関係・ルーティング規約を宣言する
//! 唯一の情報源。設計は TASK-13.1a #128、PoC-7 `docs/spec/03-poc/ai-self-maintenance/`
//! を参照）である。
//!
//! 本クレートはこのマニフェストを読み込むための最小限の基盤を提供する:
//!
//! - [`toml`]: `structure.toml` に必要な範囲のみを扱う TOML サブセットパーサ
//!   （外部依存ゼロ。`xtask/src/json.rs` の設計を踏襲し、`unwrap()` / `panic!` を
//!   使わずすべてのパース失敗を `Result` で返す）
//! - [`structure`]: TOML から得た値を型付きモデル（[`structure::StructureManifest`]）
//!   へ変換し、必須キー・役割の妥当性・依存関係の参照整合性を検証する
//!
//! `cargo metadata` 連携によるマニフェスト生成・ルート抽出・JSON 出力（TASK-13.1c
//! #130）、および `fw structure` / `fw impact` / `fw gate` サブコマンドの実体
//! （TASK-13.1c 以降）は本クレートの後続タスクのスコープであり、本モジュール群は
//! それらから呼び出される契約を前提に「パース・検証」までを担う。
#![forbid(unsafe_code)]

pub mod structure;
pub mod toml;

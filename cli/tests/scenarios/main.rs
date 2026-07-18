//! TASK-13.4（#143、親トラッキング）: 代表的改修シナリオ 3 件の `impact`→
//! 変更適用→`gate` 一連フローを製品 CLI（`fw`）に対する統合テストとして
//! 再現し、PoC-7（`docs/spec/03-poc/ai-self-maintenance/`）で実証したライフ
//! サイクル全体（事前判定 → 欠陥混入 → BLOCKED → 修正 → PASS）を CI で継続
//! 検証する（REQ-13）。
//!
//! 本ファイル（`tests/scenarios/main.rs`）はサブモジュール群を束ねる統合テスト
//! ターゲットのエントリポイント。cargo は `tests/<dir>/main.rs` 形式を
//! `tests/*.rs` と同様に自動検出してテストターゲット `scenarios` として
//! ビルドするため、`cli/Cargo.toml` の編集は不要（複数ファイルで 1 ターゲットを
//! 共有し、`mod common;` のヘルパーを各シナリオが再利用する）。
//!
//! - `bugfix_escape`: TASK-13.4b（#145、本イシュー）シナリオ 1（バグ修正）。
//!   `escape_html` のエスケープ回帰を注入し、`fw gate` が BLOCKED → 修正後に
//!   PASS することを検証する。
//! - シナリオ 2（#146）・シナリオ 3（#147）は、本ファイルへの `mod` 追加と
//!   `common.rs` のヘルパー再利用で合流する想定（`_/local-plans/145-*.md` の
//!   実装計画 §3 参照）。
//!
//! 本ファイル・配下のテストの削除・弱体化（アサーション削除・`#[ignore]`
//! 付与等）は REQ-13 の受け入れ基準（impact による事前判定・BLOCKED・修正後
//! PASS のライフサイクル全体が担保されていること）を失わせるため行わない
//! （coding-rust.md「テストの `#[ignore]` 追加でごまかさない」）。

mod bugfix_escape;
mod common;

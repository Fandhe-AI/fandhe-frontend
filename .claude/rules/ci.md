# CI 規約

## Runner 方針

- GitHub Actions の CI ジョブは `runs-on: self-hosted` を既定とする（ユーザー指示 2026-07-18）
- 新規ジョブ追加時も self-hosted を使用し、`ubuntu-latest` 等の GitHub ホステッドランナーを使わない
- 理由: 自社 runner 管理下での安全性・コスト最適化・大規模テストへの対応

## Self-hosted 環境の前提

- 共有 `CARGO_TARGET_DIR=/cargo-target` が使われるため、テストフィクスチャはクレート名衝突・キャッシュ誤命中を避ける必要がある
- 対策: フィクスチャ専用 `CARGO_TARGET_DIR` を明示指定する（例: `cli/tests/negative_cases.rs` / `cli/tests/raw_html_lint_e2e.rs`、PR #264）

## ツール前提の明示

- runner に常設が保証されないツール（wasm-bindgen-cli / wasm-pack / cargo-deny / clippy component / Chrome 等）に依存するステップは以下のいずれかを実行する
  - 存在チェック付きインストール（`command -v` / `where` 等で確認してから `cargo install` 等を実行）
  - ワークフロー YAML に明示的な前提コメント（例: `# 要: wasm-pack がインストール済み`）
- **`fw gate`（`cli/src/gate.rs`）系のツール（clippy component / cargo-deny）**: `tools/ci/ensure-gate-tools.sh` を標準ブートストラップとする（イシュー #292）。CI（`.github/workflows/ci.yml` の test ジョブ）・ローカル開発・AI 自己保守フックのいずれも `fw gate` 実行前にこのスクリプトを前置する運用を推奨する。バージョン固定・SHA256 チェックサム検証はスクリプト側に一元化し、CI ワークフロー側との二重管理でドリフトさせない。前置されなかった場合でも `fw gate` 側のプリフライト検出（`docs/gate-design.md` §2.3a）が「環境エラーであること」を決定的なメッセージ（是正コマンド付き）で示し、コード起因の FAIL との区別を保つ

## ワークフロー YAML の規約

- ステップ名（`name:` フィールド）に「: 」を含める場合はクォートで囲む（例: `name: "test: verify escaping"` ）
- 理由: 過去に構文エラーで CI 全滅の実績（PR #264 で修正）
- YAML の仕様上、構造化された値（コロン含む）はクォート必須

## runner イメージの常設要件・保守ワークフロー（イシュー #295）

- self-hosted runner イメージへ常設を依頼したい項目（libnss3/libnspr4 等）は `docs/ci-runner-requirements.md` に一覧化する
- プール状態の検査・旧バイナリ／stale tmp のクリーンアップは `.github/workflows/runner-maintenance.yml`（`workflow_dispatch` 起点、report-only）で行う
- イメージ側の常設が進んでも、各ワークフローの存在チェック付きインストール（安全網）は削除・弱体化しない

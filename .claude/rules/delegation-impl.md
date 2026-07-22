# 委譲マッピング（作成・編集フェーズ）

## 目的

ファイルの作成・編集は対象パスに応じた builder Agent へ委譲し、main の直接編集を最小化する。

## パスベース委譲マッピング

| 対象パス | 委譲先 Agent | model |
|---------|-------------|-------|
| `crates/core/`（fandhe-frontend-core: 描画コア） | core-builder | sonnet |
| `crates/interactive/`（fandhe-frontend-interactive: 状態管理コア） | core-builder | sonnet |
| `crates/headless-ui/`（fandhe-frontend-headless-ui: headless UI コンポーネント層） | core-builder | sonnet |
| `crates/pre-styled-ui/`（fandhe-frontend-pre-styled-ui: pre-styled UI コンポーネント層） | core-builder | sonnet |
| `crates/app/`（fandhe-frontend-app: アプリ構築層） | server-builder | sonnet |
| `crates/server/`（fandhe-frontend-server: SSR/SSG/ルーティング） | server-builder | sonnet |
| `crates/wasm-client/` `crates/wasm-full/` `crates/wasm-thin/` | wasm-builder | sonnet |
| `static/`（埋め込み HTML） | wasm-builder | sonnet |
| `crates/xtask/` `crates/cli/` `.github/` `Dockerfile` `deny.toml` `Cargo.toml`（workspace） | tooling-builder | sonnet |
| `docs/`（`docs/spec/` を除く）・README.md・CLAUDE.md | docs-writer | haiku |
| テスト実行・失敗分析 | test-runner | sonnet |

## 検証・レビューの委譲

| 工程 | 委譲先 | model |
|------|--------|-------|
| コミット前レビュー | reviewer | sonnet |
| セキュリティ監査（PR 前必須） | security-auditor | sonnet |
| fmt / clippy / frontmatter の機械チェック | linter | haiku |

## 運用ルール

- **`docs/spec/` は編集禁止**（サブモジュール。変更は fandhe-frontend-spec リポジトリで行う）
- 複数クレートにまたがる変更は、クレート単位に分割して各 builder へ並列委譲する。ただし依存関係（core → server 等）がある場合は依存順に直列化する
- builder への委譲プロンプトには「対象タスク（TASK-x.x）・変更対象ファイル・厳守事項（既定エスケープ・forbid(unsafe_code) 等）・完了条件（テスト通過）」を含める
- 委譲後の統合確認（workspace 全体の `cargo test`）は test-runner に委譲する
- main が直接編集してよいのは、1 ファイル数行の軽微な修正・委譲結果の微調整のみ

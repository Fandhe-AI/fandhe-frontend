# CLAUDE.md

## Overview

Rust 製フロントエンドフレームワーク。AI 時代のセキュリティリスク低減を目的に、プレーンな HTML / JavaScript / CSS を尊重しつつ SSR / SPA / SSG / トランジションなどモダン機能を網羅する。部分埋め込みの最小構成からフル機能構成までのグラデーションを持ち、単一実行ファイルでのデプロイ（Docker 想定）を目標とする。

- 正式名称は `fandhe-frontend`（確定、2026-07-19）。決定記録・新旧マッピング表は `docs/design/framework-naming.md` を参照。crate 名は #441 で `rws-*` から `fandhe-frontend-*` へ改名済み。リポジトリ名は #439 で `Fandhe-AI/fandhe-frontend` へ改名済み。全 9 クレート（fandhe-frontend-core / -interactive / -app / -server / -wasm-client / -wasm-full / -wasm-thin / -dist-server / -cli）は v0.1.0 で 2026-07-20 に crates.io へ公開済み。加えて `fandhe-frontend-headless-ui` / `fandhe-frontend-pre-styled-ui`（ark-ui / chakra-ui 参考の 2 層 UI コンポーネント構成、親トラッキング #520 で新設）も v0.1.0 で crates.io へ公開済み（イシュー #608）
- 仕様書は [Fandhe-AI/fandhe-frontend-spec](https://github.com/Fandhe-AI/fandhe-frontend-spec) を `docs/spec/` サブモジュールとして取り込み管理
- 開発は `docs/spec/06-roadmap.md` のマイルストーン MS-1〜MS-5 に従う（最初のタスクは TASK-1.1: `fandhe-frontend-core` 既定エスケープの製品化）
- 計画クレート: `fandhe-frontend-core`（描画コア・外部依存ゼロ）/ `fandhe-frontend-app` / `fandhe-frontend-server`（SSR/SSG）/ `fandhe-frontend-wasm-client`・`fandhe-frontend-wasm-full`（WASM/CSR）/ `fandhe-frontend-interactive`（状態管理）/ `fandhe-frontend-headless-ui`（headless UI コンポーネント層、#520）/ `fandhe-frontend-pre-styled-ui`（pre-styled UI コンポーネント層、#520）/ `xtask`（CI 計測）/ `fandhe-frontend-cli`（`fw` コマンド・AI 自己保守フック、REQ-13）

## Repository Structure

```
fandhe-frontend/
├── CLAUDE.md
├── README.md
├── skills-lock.json          # npx skills add の導入記録
├── docs/
│   ├── design/               # 設計文書（gate-design / wasm-full-architecture / structure-manifest 等）
│   ├── api/                  # API 仕様（component-api / hydration-api / hydration-state-format 等）
│   ├── guides/               # 利用者向けガイド（embedding-guide / npm-asset-build / browser-testing 等）
│   ├── policy/               # 規約・セキュリティポリシー（unsafe-boundary / dependency-graph-policy / cargo-deny-advisories / intentional-non-adoption 等）
│   ├── ci/                   # CI・runner 運用（ci-runner-requirements / perf-browser-harness / cargo-semver-checks-evaluation）
│   ├── reports/              # 実測・受け入れレポート（perf-browser-report / *-acceptance-report 等）
│   └── spec/                 # 仕様サブモジュール (fandhe-frontend-spec)
│       ├── 01-brainstorm.md
│       ├── 02-poc-plan.md
│       ├── 03-poc/           # PoC-1〜7 成果物（rendering-web-standards が中核）
│       ├── 04-requirements.md  # MoSCoW 要件・受け入れ基準
│       ├── 05-tasks.md         # タスク分解（依存・工数）
│       └── 06-roadmap.md       # MS-1〜MS-5・着手判定
├── examples/
│   ├── ssr-routing/          # SSR + ルーティング正本サンプル・examples 規約の初例（crates.io バージョン依存、イシュー #499）
│   ├── ssg-blog/             # SSG（generate_pages）による静的ブログ正本サンプル（crates.io バージョン依存、イシュー #501）
│   ├── dist-server-docker/  # 単一バイナリ配布 + Docker 正本サンプル（crates.io バージョン依存、イシュー #502）
│   ├── interactive-view-transitions/  # 状態管理（fandhe-frontend-interactive）+ View Transitions 正本サンプル（イシュー #503）
│   └── headless-pre-styled-ui/  # headless-ui / pre-styled-ui コンポーネントショーケース（crates.io バージョン依存、`fw new --example` 対応、イシュー #609）
├── templates/
│   ├── default/
│   │   ├── deny.toml         # 標準プロジェクトテンプレート同梱の cargo-deny 設定（TASK-4.1 / REQ-4）
│   │   ├── structure.toml    # fw gate が唯一の情報源として読む構造マニフェスト（生成直後 fw gate PASS 保証、イシュー #351）
│   │   ├── tools/
│   │   │   └── npm-asset-build/  # NPM 静的アセットゲートの同梱コピー（正本は tools/npm-asset-build/、イシュー #316）
│   │   └── .github/
│   │       └── workflows/
│   │           ├── deny.yml            # 禁止クレート追加を CI でブロックするテンプレートワークフロー（TASK-4.2 / REQ-4）
│   │           └── npm-asset-gate.yml  # NPM 静的アセットゲート（install.sh 経由）のテンプレートワークフロー（REQ-12, イシュー #316）
│   ├── app/                   # `fw new --template app`（イシュー #378/#411）: fandhe-frontend-core/fandhe-frontend-app 依存の拡充テンプレート（wasm ビルド込み CSR 完全実体を同梱）
│   │   ├── src/main.rs       # Loader・束縛点 API（bind_text/keyed_list）・render の実体サンプル
│   │   ├── wasm/              # CSR wasm ビルド用の独立ワークスペース（glue クレート app-csr-wasm、root の依存グラフから隔離）
│   │   ├── tools/wasm/build.sh # wasm ビルド手順（wasm-bindgen-cli バージョン整合の fail-closed 検証込み）
│   │   └── static/embed.html # CSR マウント骨格（templates/embed/embed.html の同梱コピー、build.sh 実行後に動作）
│   └── embed/                 # `fw new --template embed`（イシュー #410）: 静的単一ファイルの部分埋め込み構成（cargo パッケージなし）
│       ├── embed.html        # TASK-7.1a（#52）正本（templates/app/static/embed.html と同一）
│       └── structure.toml    # fw gate 静的専用（asset-only）モードの明示宣言（role = "asset" のみ、crate キーなし）
└── .claude/
    ├── agents/               # カテゴリ別 sub-agent 定義
    ├── rules/                # 委譲・コーディング・セキュリティ規約
    ├── skills/               # npx skills add 導入スキル
    └── settings.json         # SessionStart / PostToolUse hooks
```

全メンバークレートは `crates/` 配下に配置する（イシュー #436）:

```
crates/
├── core/          # fandhe-frontend-core: 描画コア・外部依存ゼロ
├── interactive/   # fandhe-frontend-interactive: 状態管理コア
├── app/           # fandhe-frontend-app: モード非依存の共通コンポーネント
├── server/        # fandhe-frontend-server: SSR/SSG エントリ
├── wasm-client/   # fandhe-frontend-wasm-client: クライアントランタイム基盤
├── wasm-full/     # fandhe-frontend-wasm-full: CSR/ハイドレーション フルセット
├── wasm-thin/     # fandhe-frontend-wasm-thin: CSR/ハイドレーション 最小構成
├── dist-server/   # fandhe-frontend-dist-server: 単一実行ファイル配布サーバー
├── headless-ui/   # fandhe-frontend-headless-ui: headless UI コンポーネント層（anatomy・data-*・WAI-ARIA、イシュー #520/#522）
├── pre-styled-ui/ # fandhe-frontend-pre-styled-ui: pre-styled UI コンポーネント層（headless-ui 上層のスタイル済み部品、イシュー #520/#546）
├── docs-site/     # fandhe-frontend-docs-site: docs サイトジェネレータ（外部依存ゼロ・配布物に含めない開発者/CI 用ツール）
├── cli/           # fandhe-frontend-cli: `fw` コマンド（structure.toml のスキーマ・パース・生成、REQ-13）
│   ├── templates/          # `fw new --template` 埋め込み用の同梱コピー（正本はルート `templates/`。`new_template.rs` が `include_str!` で吸収、乖離は `tests/template_publish_copy_drift.rs` が検知）
│   └── embedded-examples/  # `fw new --example` 埋め込み用の同梱コピー（正本はルート `examples/`。パッケージ名は置換せず正本と全ファイルバイト一致、乖離は `tests/example_publish_copy_drift.rs` が検知、イシュー #500）
└── xtask/         # CI 計測用の開発者ツール
```

ルート `Cargo.toml` は `members = ["crates/*"]`（glob）。リポジトリ自身の
`structure.toml` は各 `[directories.<name>]` に `path = "crates/<name>"` を
宣言し、依存宣言の論理名（`<name>`）とは独立して実配置を表す
（`docs/design/structure-manifest.md` §2.2.0a 参照）。`fw new` が生成する
ユーザープロジェクト（`templates/`）は `path` を使わないフラット配置のまま
不変。

## 委譲方針（必読）

main セッションは**指揮・統合・ユーザー対話に専念**し、調査・実装・テスト・レビューは sub-agent へ委譲して main のコンテキスト消費を抑える。詳細は `.claude/rules/delegation.md`（調査・設計）と `.claude/rules/delegation-impl.md`（作成・編集）を参照。

### パスベース切り替え表

| 対象パス | 委譲先 Agent |
|---------|-------------|
| `crates/core/` `crates/interactive/` | core-builder |
| `crates/headless-ui/` `crates/pre-styled-ui/` | core-builder |
| `crates/app/` `crates/server/` | server-builder |
| `crates/wasm-client/` `crates/wasm-full/` `crates/wasm-thin/` `static/` | wasm-builder |
| `crates/xtask/` `crates/cli/` `.github/` `Dockerfile` `deny.toml` `templates/` | tooling-builder |
| `docs/`（spec 以外）・CLAUDE.md | docs-writer |
| `docs/spec/`（読み取り調査） | explorer |
| テスト実行・失敗分析 | test-runner |
| レビュー | reviewer / security-auditor |

### model 配分表

| 用途 | model |
|------|-------|
| 複雑な横断判断・アーキテクチャ設計 | opus または fable（fable は特に大規模設計・横断判断の最上位 tier） |
| 調査・生成・実装・レビュー | sonnet |
| 機械的集計・lint・ドキュメント更新 | haiku |

## Sub-agents

`.claude/agents/<category>/<name>.md` に定義。

| カテゴリ | subagent_type | model | 役割 |
|---------|---------------|-------|------|
| research | explorer | sonnet | コードベース・`docs/spec/` 横断調査（読み取り専用） |
| research | reference-researcher | sonnet | 外部仕様（Rust / WASM / Web 標準 / 依存クレート）調査 |
| implement | core-builder | sonnet | `crates/core/` `crates/interactive/` — 描画・状態管理コア（`forbid(unsafe_code)` 域） |
| implement | server-builder | sonnet | `crates/app/` `crates/server/` — SSR / SSG / ルーティング |
| implement | wasm-builder | sonnet | `crates/wasm-client/` `crates/wasm-full/` `crates/wasm-thin/` `static/` — CSR / ハイドレーション / WASM |
| implement | tooling-builder | sonnet | `crates/xtask/` / CI / Dockerfile / cargo-deny / 単一バイナリ配布 / AI 自己保守フック |
| testing | test-runner | sonnet | `cargo test` / XSS 回帰 / wasm テストの実行と失敗分析 |
| quality | reviewer | sonnet | 仕様準拠・アーキテクチャ整合・Rust イディオムのレビュー |
| quality | security-auditor | sonnet | OWASP・XSS エスケープ保証・`unsafe` 境界・依存監査 |
| quality | linter | haiku | rustfmt / clippy / frontmatter の機械的チェック |
| docs | docs-writer | haiku | README / CLAUDE.md / docs/（spec 除く）の更新 |

## Rules

`.claude/rules/` に定義。

| ファイル | 内容 |
|---------|------|
| `delegation.md` | 調査・設計フェーズの委譲原則・パスベース切り替え |
| `delegation-impl.md` | 作成・編集フェーズの委譲マッピング |
| `coding-rust.md` | Rust 規約（既定エスケープ厳守・`forbid(unsafe_code)`・依存上限 60 件/深さ 6・core 外部依存ゼロ） |
| `security.md` | OWASP Top 10・秘密情報混入防止・サプライチェーン対策 |
| `japanese-style.md` | 日本語出力スタイル |
| `conventional-commits.md` | Conventional Commits 詳細規約（scope 一覧含む） |
| `code-comment-style.md` | コメント規約（役割・責務・呼び出し文脈・`// SAFETY:` を埋め込む） |
| `out-of-scope-tracking.md` | 実装対象外の追跡規約（スコープ外事項を Issue 化して放置しない） |
| `ci.md` | CI 規約（self-hosted runner 既定・共有 CARGO_TARGET_DIR 対策・ツール前提の明示） |

## Current Skills

`npx skills add Fandhe-AI/agent-cli-skills` で導入（`skills-lock.json` で追跡）。

- **コミット・PR**: create-commit / create-pr / implement-review / implement-review-pr
- **Issue**: create-issue / create-issue-tree / update-issue-tree / implement-issue / implement-issue-tree
- **計画・ドキュメント**: create-plan / update-docs / comment-code
- **Project v2**: project-init / project-add-items / project-create-issues / project-update-items / project-view-status / project-sync-issues / project-archive-done
- **.claude 体系**: init-claude / update-claude / sync-skills-lock / contribute-skill / update-reference
- **リファレンス**: rust / github-docs / commitlint / lefthook / editorconfig

## Conventions

- **日本語**: やりとり・ドキュメント・コミット/PR 本文は日本語（`japanese-style.md`）
- **Conventional Commits**: create-commit スキルを使用。`--no-verify` 禁止（`conventional-commits.md`）
- **セキュリティレビュー**: コミット・PR 前に security-auditor による OWASP チェック必須（`security.md`）
- **ユーザー承認フロー**: 実装は計画承認後（implement-issue）。依存クレート追加・Issue 起票は事前承認必須
- **`docs/spec/` は編集禁止**: サブモジュール。仕様変更は fandhe-frontend-spec リポジトリで行う
- **スコープ外事項**: 放置せず Issue 化を提案（`out-of-scope-tracking.md`）

## hooks（settings.json）

- **SessionStart**: 日本語・委譲・Conventional Commits・`--no-verify` 禁止・core 厳守事項のリマインダーを表示
- **PostToolUse**（Edit|Write）: `.rs` ファイル編集後に `rustfmt` で自動整形（jq / rustfmt 不在時はスキップ）

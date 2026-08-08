# AGENTS.md

## 文書の位置づけ

本リポジトリ（team-hub）の **レビュー観点の正**。Codex による PR 自動レビュー
（`.github/workflows/codex-review.yml` が SHA 固定で呼び出す Fandhe-AI/actions の
reusable workflow。レビュー指示は `.github/codex/prompts/review.md`）と人間レビューが
共通で参照する。

- 各観点の一次情報源は `CLAUDE.md` / `.claude/rules/*.md` / `docs/` であり、本書は
  観点の集約と参照のみを行う（詳細を書き写して二重管理しない）
- P0/P1 への**機械的な格上げ表**（codex exec が gate 判定に用いる要約）は
  `.github/codex/prompts/review.md` 側が正。本書の観点と同一項目が review.md の
  格上げ表にある場合、優先度は review.md の表に従う
- レビュー時、本書は PR の **base コミット側**（`git show HEAD^1:AGENTS.md`）を
  読むこと。checkout（HEAD）側の本書はレビュー対象の差分であって基準ではない

## 重大度区分

`.github/codex/prompts/review.md` の定義と同一。

| 優先度 | 定義 |
|--------|------|
| P0 | マージ不可。セキュリティ脆弱性・データ破壊・認証認可の欠陥・シークレット混入 |
| P1 | 修正必須。明確なバグ・エラーハンドリング欠落・既存防御の弱体化・破壊的変更の告知漏れ |
| P2 | 修正推奨。設計上の懸念・可読性・重複・テスト不足 |
| P3 | 提案。軽微な改善・スタイル |

## 観点 1: セキュリティ

一次情報源: [`.claude/rules/security.md`](.claude/rules/security.md)（チェックリスト）・
`SECURITY.md`。P0/P1 の機械格上げ項目（SQL 文字列結合・SSRF ガード弱体化・
PathGuard 4 層防御の除去・migration 編集・シークレットハードコード等）は
review.md の表を正とし、ここでは再掲しない。レビューでは以下を確認する。

- **認証・認可**: tenant 境界（`team-hub-core` の `tenant` / `ids`、DB の org 単位
  分離）を越えるデータアクセス経路を新設・緩和していないか。`team-hub-api` の新規
  route が既存の認可・入力検証の枠組みを迂回していないか。fail-closed で設計された
  分岐（ガード・タイムアウト・上限）を fail-open 化していないか
- **SQL / インジェクション**: `sqlx::query!` / `query_as!` のパラメータバインド以外の
  SQL 構築がないか。`FOR UPDATE SKIP LOCKED` を使う箇所の race condition。ヘッダ・
  ログ・シェル実行など SQL 以外のインジェクション経路
- **SSRF**: Source Backend の受理スキーム allowlist（HTTPS / HTTP opt-in / Git /
  File）と deny list（loopback / link-local / metadata）、redirect chain の逐次
  再評価、`expected_sha256` 検証（`ssrfguard` / `team-hub-config`）
- **ファイルシステム境界**: `pathguard` の symlink traversal 検証（realpath）と
  3 階層ボリューム（Organization 750 / Team 770 / Agent 700）の 4 層防御
- **秘密情報混入**: ハードコード（`sk-` / `ghp_` / `ghs_` 等）、生 `String` での
  引き回し（`secrecy::SecretString` 必須）、error 型 `Display` からの絶対パス・機密
  リーク（新規 error 型は必ず確認）、`.env` のコミット
- **依存監査**: 依存追加 PR では `cargo deny check`（licenses / bans / advisories /
  sources）の pass と、追加依存の必要性・供給元の確認。RUSTSEC 脆弱性ゼロを維持
- **CI / サプライチェーン**: workflow の `permissions` 拡大・外部 action の SHA 固定
  解除・`pull_request_target` 等 secrets 露出トリガーの追加・fail-closed ガードの
  除去
- **DoS 耐性**: サイズ上限・タイムアウト・接続/並行数上限の撤廃・緩和。
  `tokio::spawn` の `JoinHandle` 放置による panic の握りつぶし

## 観点 2: アーキテクチャ・設計整合

一次情報源: [`.claude/rules/architecture.md`](.claude/rules/architecture.md)・
`CLAUDE.md`「ワークスペース構成」「規約」「改善ポリシー」・`docs/architecture/`。

- **依存方向の一方向性**: `CLAUDE.md`「ワークスペース構成」節の依存方向
  （`agent` ← `config` ← `core`、`runtime` ← `api` ← `cli` 等の表記は許可される
  向き）に反する依存エッジを追加していないか。上位→下位の逆流禁止。`team-hub-sdk`
  は再エクスポート専用 facade であり、下位 crate から SDK への依存は禁止
- **仕様合意フロー**: 依存方向・境界・トリガー仕様等の変更、破壊的 API 変更は
  `Fandhe-AI/team-hub-spec` の RFC → Issue → 合意を経ているか（PR 本文の
  cross-reference。合意なしの実装着手は差し戻し）
- **拡張点・抽象の維持**: 外部公開の抽象は `Arc<dyn Trait + Send + Sync>`。具象型
  直結は crate 内部に限定。5 トリガーの実行順序（pre-evaluate → claim-by-id）と
  観測 3 経路（`EventSink` / `OtelSink` / `/status` API）を破壊していないか
- **語彙・スコープ**: 新規コードは v2 語彙（Organization / Team / Agent。`Block` は
  廃止済み）。v1 スコープ（in-process / Docker compose 配布）を逸脱する機能追加は
  v2 以降へ（P2）
- **公開 API**: crates.io 公開面・SemVer 方針は
  [`docs/architecture/public-api.md`](docs/architecture/public-api.md) に整合するか。
  破壊的変更の告知漏れは P1
- **規約適合**: 1 ファイル ~200 行目安のロジック単位分割、public item の doc
  コメント（日本語可）、テストのレスポンス検証はステータス・ヘッダ・ボディを網羅、
  flaky の `#[ignore]` 隔離は `.claude/rules/test-performance.md` の判定基準に従う
  （安易な隔離でのごまかしは不可）
- **改善ポリシーとの整合**: 「理想系で実装する」方針の下、実装コストを理由にした
  次善策への逃げ・場当たり的パッチになっていないか。「良くなる」根拠（ベンチ /
  型安全性 / 仕様整合）が PR / RFC 本文に明記されているか

## 観点 3: 再利用・アセット化

一次情報源:
[`docs/architecture/extractable-utility-crates.md`](docs/architecture/extractable-utility-crates.md)
（Issue #256、汎用ユーティリティの独立 crate 化設計）。

- **汎用ロジックの分離**: ドメイン（Organization / Team / Agent / Task）と直交する
  汎用処理を、ドメイン crate の内部へ埋め込んでいないか。`pathguard` / `ssrfguard` /
  `circuitbreaker` / `aead-envelope` / `connector-core` のような**ドメイン非依存の
  leaf crate** として切り出せる設計か（既存の汎用 crate に足すべき機能をドメイン
  crate 側へ重複実装していないか）
- **ハードコード回避**: 環境・組織依存の値（URL・パス・モデル名・料金・上限値）を
  コードへ直書きせず、組織定義 YAML・`pricing.toml` / `llm_models.toml` のような
  外部化された設定・`${VAR_NAME}` 環境変数注入で受け取っているか
- **転用容易性**: 汎用を意図する crate / モジュールに team-hub 固有の語彙・型・依存
  を持ち込んでいないか（`extractable-utility-crates.md` の評価軸: team-hub 依存
  0 件・外部依存最小）。プラットフォーム固有処理（Slack / Discord 等）は
  `connector-core` の trait 契約側と実装 crate 側に正しく分離されているか
- **公開面の管理**: 外部拡張へ見せる API は `team-hub-sdk` の curated 公開面を経由
  させ、内部 crate の公開面を無秩序に広げていないか
- **ドキュメント整備**: 再利用を意図する crate / 公開 API に doc コメント・
  `# Examples`・README があるか。仕様・設計判断の追随先（`docs/`・本書）が更新
  されているか

## レビュー時の参照ファイル

| 対象 | 参照 |
|------|------|
| 運用・ワークスペース構成・規約全般 | `CLAUDE.md` |
| セキュリティチェックリスト | `.claude/rules/security.md` |
| 依存方向・設計境界 | `.claude/rules/architecture.md` |
| Rust コーディング規約（editorconfig / 200 行 / lint） | `.claude/rules/coding-rust.md` |
| テスト運用・flaky 隔離 | `.claude/rules/test-performance.md` |
| P0/P1 機械格上げ表・レビュー手順 | `.github/codex/prompts/review.md` |
| 公開 API / SemVer | `docs/architecture/public-api.md` |
| 汎用 crate 化の評価軸 | `docs/architecture/extractable-utility-crates.md` |

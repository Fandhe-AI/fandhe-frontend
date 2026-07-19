# フレームワーク正式名称の決定記録

> **ステータス: 正式名称 `fandhe-frontend` に確定。**
> ユーザー決定（2026-07-19、親イシュー #433）により、fandhe ブランドで複数
> フレームワーク（backend / frontend / AI）を展開する方針（fandhe-backend
> #200、`docs/design/framework-naming.md`）に基づき、org 名 `fandhe` を
> プレフィックスとする統一命名 **`fandhe-frontend`** を正式名称として確定
> した。姉妹リポジトリ [Fandhe-AI/fandhe-backend](https://github.com/Fandhe-AI/fandhe-backend)
> の先例（改名ツリー #200 配下 #201〜#205）に倣い、名称・crate 名・
> ディレクトリ構成・表記を段階的に移行する。決定の根拠・可用性証跡・確定版
> 新旧マッピング表は「決定（確定版）」節を参照。

対応: #433（`fandhe-frontend` への改名・crates/ 構成移行ツリー・親イシュー）・
#434（本イシュー、決定記録の新規作成）。

## 決定（確定版）

**決定名称: `fandhe-frontend`**（ユーザー決定 2026-07-19、親イシュー #433）。
crate プレフィックスは `fandhe-frontend-*`。

### 根拠

- fandhe ブランドで複数フレームワーク（backend / frontend / AI）を展開する
  方針（fandhe-backend #200 で確立）に基づき、各フレームワークを org 名
  `fandhe` をプレフィックスとする統一命名体系（`fandhe-backend` /
  `fandhe-frontend` / `fandhe-ai`）に揃えることで、ブランドとしての
  一貫性・検索性・今後の関連プロジェクトとの整合を優先した
- **`fandhe-web` ではなく `fandhe-frontend` を採用した理由**: fandhe-backend
  の決定記録（`docs/design/framework-naming.md`、fandhe-backend リポジトリ）
  では将来名の例示として `fandhe-web` が挙がっていたが、本リポジトリでは
  `fandhe-backend` との**完全対称**（backend/frontend という役割対で命名を
  揃える）を優先した。`web` は本来フロントエンド・バックエンドの両方を
  包含し得る語であり、役割の非対称な曖昧さを持ち込むより、対になる
  `frontend` を採用するほうが「fandhe 傘下のフロントエンドフレームワーク」
  であることを名称から直接読み取れ、`fandhe-ai` 等の将来追加フレームワーク
  とも命名規則上の対称性を保てると判断した
- 全 crate が `publish = false`（crates.io 未公開）のため、crate 改名に
  伴う外部互換負担が現時点で存在しない

### 可用性確認の証跡（2026-07-19、親イシュー #433 の実測を出典として転記）

| 対象 | 確認方法 | 結果 |
|------|---------|------|
| crates.io `fandhe-frontend` | `https://crates.io/api/v1/crates/fandhe-frontend`（ブラウザ相当 User-Agent） | 未使用 (404) |
| GitHub `Fandhe-AI/fandhe-frontend` | `https://api.github.com/repos/Fandhe-AI/fandhe-frontend` | 未使用 (404) |

crates.io の API 応答はブラウザ相当の `User-Agent` ヘッダ付与が必要（既定
User-Agent では Cloudflare 由来の 403 を返す環境がある、fandhe-backend の
先例と同様）。本確認は crates.io registry・GitHub API による一次
スクリーニングであり、商標登録データベースの調査や法的なクリアランスでは
ない（「責務分界」節参照）。フェイルクローズ方針: 到達不能・想定外応答の
場合は「未使用」と断定せず、親イシュー #433 の記載を出典として明記する
（本記録は #433 記載の実測結果をそのまま転記している）。

### 確定版 新旧マッピング表（#433 本文を正とし転記。#435〜#439 の実装が参照する）

| 種別 | 旧 | 新 |
|------|-----|-----|
| Cargo package | `rws-core` / `rws-interactive` / `rws-app` / `rws-server` / `rws-wasm-client` / `rws-wasm-full` / `rws-wasm-thin` / `rws-cli` / `rws-dist-server` | `fandhe-frontend-core` / `fandhe-frontend-interactive` / `fandhe-frontend-app` / `fandhe-frontend-server` / `fandhe-frontend-wasm-client` / `fandhe-frontend-wasm-full` / `fandhe-frontend-wasm-thin` / `fandhe-frontend-cli` / `fandhe-frontend-dist-server` |
| Rust import | `rws_core::` 等 | `fandhe_frontend_core::` 等 |
| 環境変数 | `RWS_BIND_ADDR` / `RWS_WASM_BUILD` | `FANDHE_FRONTEND_BIND_ADDR` / `FANDHE_FRONTEND_WASM_BUILD` |
| ディレクトリ | ルート直下 `core/` `app/` 等の平置き | `crates/core` `crates/app` 等（fandhe-backend と同形） |
| GitHub リポジトリ | `Fandhe-AI/frontend-framework` | `Fandhe-AI/fandhe-frontend` |
| 文書表記 | `frontend-framework` / `rws-*` | `fandhe-frontend` / `fandhe-frontend-*` |

対象外（親イシュー #433・fandhe-backend #200 と同一方針）:

- `fw` バイナリ名（中立な CLI コマンド名として維持。package 名
  `rws-cli` → `fandhe-frontend-cli` の改名とは独立）
- `xtask`（中立な補助クレート名）
- `docs/spec/`（別リポジトリ `fandhe-frontend-spec` のサブモジュール。
  本ツリーでは参照表記のみを更新し、spec リポジトリ自体の改名・
  `RWS_BIND_ADDR` 記載（`04-requirements.md`）の更新は
  fandhe-frontend-spec 側へ別途申し入れる）

## 責務分界

以下は org 管理者権限・法務判断が必要な**人間実施**の作業であり、AI
エージェントの自律実装スコープ外とする。

- 商標・法的なクリアランス確認（可用性証跡節の一次スクリーニングを超える
  正式調査）
- crates.io 上での名称確保（予約公開）の要否判断・実施
- GitHub リポジトリ名の変更（第 6 段階 #439）の実施可否判断・実施（旧 URL
  からのリダイレクト・外部リンク・CI シークレット等への影響評価を含む）
- `docs/spec/`（別リポジトリ `Fandhe-AI/fandhe-frontend-spec`）側の
  名称関連記述の更新（submodule のため本リポジトリ側からは書き換えない）

## 段階的移行計画

親イシュー #433 の段階別実装計画（実行順 = 直列依存）をそのまま転記する。
各段階は個別 Issue・個別 PR とする。

| 順 | Issue | 内容 |
|----|-------|------|
| 1 | #434 | 決定記録 `docs/design/framework-naming.md` 作成（本書） |
| 2 | #435 | crate 名 `rws-*` → `fandhe-frontend-*` 一括改名 |
| 3 | #436 | クレートディレクトリの `crates/` 配下移設（完了。`structure.toml` の任意 `path` キーで論理名と実配置を分離し、`Cargo.toml` は `members = ["crates/*"]` へ集約） |
| 4 | #437 | 環境変数 `RWS_*` → `FANDHE_FRONTEND_*` 改名 |
| 5 | #438 | docs・CI・スクリプトの表記統一 |
| 6 | #439 | リポジトリ名変更（GitHub リネーム・remote 追随、人間管理者実施） |

## 参照

- 決定の親イシュー・改名ツリー: #433（本記録は配下 #434 対応）
- fandhe-backend の先例: [Fandhe-AI/fandhe-backend](https://github.com/Fandhe-AI/fandhe-backend) #200〜#205、`docs/design/framework-naming.md`（fandhe-backend リポジトリ）
- セキュリティ規約（サプライチェーン・なりすまし観点）: [`.claude/rules/security.md`](../../.claude/rules/security.md)
- スコープ外課題の追跡: [`.claude/rules/out-of-scope-tracking.md`](../../.claude/rules/out-of-scope-tracking.md)
- 文体: [`.claude/rules/japanese-style.md`](../../.claude/rules/japanese-style.md)

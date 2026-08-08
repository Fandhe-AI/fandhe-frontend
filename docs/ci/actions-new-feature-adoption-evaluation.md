# Fandhe-AI/actions 新規 15 コミット分の機能の採用可否評価（イシュー #1288）

## 1. 背景

PR #1285 で `.github/workflows/update-external.yml` が参照する
Fandhe-AI/actions の SHA を `c80ff94f...` から `5982e2f502a19effc36a7b161660b5a4ea17b886`
へ追随した。この差分（15 コミット）に含まれる新機能のうち本リポジトリ未採用
分の採用可否評価が out-of-scope として残され、本イシューで実施する。

## 2. 対象 15 コミットの分類

`gh api repos/Fandhe-AI/actions/compare/c80ff94f...5982e2f5`（total_commits:
15）で確認した。

### 2.1 採用済み（記録のみ）

| 機能 | 本リポジトリでの採用 |
|---|---|
| reusable workflow `codex-review.yml` | イシュー #1275 / PR #1278 で `.github/workflows/codex-review.yml` として導入済み |
| composite action `rust-toolchain-setup` | イシュー #1273 / PR #1280 で全ワークフロー置換済み |
| composite action `wasm-tool-install` | イシュー #1274 / PR #1279 で置換済み |
| runner 方針の反転（public は GitHub ホステッド既定） | イシュー #1281 で `.claude/rules/ci.md` へ反映済み |
| skills-lock 同期 | actions リポジトリ内部の同期であり本リポジトリへの適用対象なし |

### 2.2 今回評価した 5 件

| 機能 | 判断 | 根拠の要点 |
|---|---|---|
| reusable workflow `pages-deploy.yml` | **採用**（本イシューで実装） | `docs-site.yml` の deploy ジョブ（`upload-pages-artifact` → `deploy-pages` の直接実行）の置換対象として設計されたもの。Pages 固有 action の SHA 追随義務を共通側へ集約でき、dist-dir のパストラバーサル拒否・空サイト非公開の fail-closed 検証が加わる。rust-toolchain-setup / wasm-tool-install / codex-review 採用と同一方向の org 共通化 |
| reusable workflow `rust-base-ci.yml` | **見送り** | (1) 本リポジトリの `ci.yml` は fmt/clippy/test/deny 相当が大幅カスタム済み（キャッシュ+ガードステップ契約 `workflow_shared_target_contract.rs`・fw gate e2e・required checks）で固定ジョブ構成に収まらない。(2) 同 workflow の cargo-deny 導入は「バージョン固定 cargo install（ソースビルド）」であり、本リポジトリの #314 統一方針「prebuilt バイナリ + SHA256 チェックサム検証」（`.claude/rules/ci.md`）と非整合。(3) ジョブ単位の有効/無効入力を持たない設計のため部分採用も不可 |
| reusable workflow `lint-docs.yml` | **見送り** | markdownlint/commitlint 等を npm（`setup-node` + npx）・pypi 経由で導入する構成であり、npm 経路 CI ツール導入を構造的に受け入れないとした先例（a11y 評価 #1076、REQ-12 サプライチェーン方針）と非整合。commitlint 相当は `tools/hooks/commit-msg-check.sh`（npm 非依存）が意図的にローカル担保済み |
| composite action `cargo-tool-install` | **見送り** | 適用先が存在しない（本リポジトリの CI に `cargo install` を行うステップがない。cargo-deny は #314 の prebuilt+SHA256 パターン、wasm ツールは wasm-tool-install 採用済み）。同 action はソースビルドでバイナリ SHA256 検証を持たず、既存パターンからの置換動機もない |
| composite action `idempotent-issue` | **見送り** | CI からの自動 Issue 起票ジョブが現存しない。将来（性能退行・監査検知の自動起票導入時）の第一候補として記録する |

## 3. 採用（pages-deploy）の実装

`.github/workflows/docs-site.yml` の deploy ジョブを
`Fandhe-AI/actions/.github/workflows/pages-deploy.yml@5982e2f502a19effc36a7b161660b5a4ea17b886`
の呼び出しへ置換した（build ジョブは変更なし、dist sanity check の `test -f`
群も削除・弱体化していない）。

- build ジョブの `actions/upload-pages-artifact` 直接実行を、汎用
  `actions/upload-artifact`（artifact 名 `pages-dist`、`if-no-files-found: error`、
  `include-hidden-files: true`）へ置換。Pages 専用形式への変換以降は
  reusable workflow 側が担う（`pages-deploy/README.md` §「設計判断」の案 B）
- deploy ジョブは `uses: Fandhe-AI/actions/.github/workflows/pages-deploy.yml@<SHA>`
  へ置換し、`dist-dir: "."`・`runner-label: ubuntu-latest` を指定
- `runner-label` の明示指定は必須: reusable 側の既定値は `self-hosted`
  （fandhe 系リポジトリの旧前提）であり、省略すると本リポジトリ（public・
  ホステッドランナー既定方針）で self-hosted runner 非登録により deploy が
  永久 pending になる
- workflow レベル `permissions: contents: read`・`concurrency`
  （`cancel-in-progress: false`）は変更なし。`pages: write` / `id-token: write`
  は deploy 呼び出しジョブに限定付与（現行と同一の最小権限、reusable
  workflow は呼び出し側が付与した以上の権限を持てない）

### 3.1 検証

- `cargo test -p xtask --test workflow_runner_policy --test workflow_shared_target_contract`:
  PASS（`self-hosted` リテラル非混入・ガード契約非破壊を機械確認）
- YAML 構文: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/docs-site.yml'))"`
  で確認
- `docs-site.yml` は `pull_request` トリガーを持たないため、deploy の実挙動
  （build → deploy の成功・`page_url` 出力）は main push 後に確認する

## 4. 見送り 4 件の詳細根拠

### 4.1 `rust-base-ci.yml`

本リポジトリの `ci.yml` は以下の点で reusable workflow の固定ジョブ構成に
収まらない:

- 共有 `CARGO_TARGET_DIR` 汚染対策（`workflow_shared_target_contract.rs` が
  fail-closed に強制するガードステップ契約）
- `fw gate` の自己適用（`gate-self-apply` ジョブ）・examples e2e・
  `version-bump-guard`・`dep-version-check` 等、本リポジトリ固有のカスタム
  ジョブ群
- cargo-deny 導入パターンが `rust-base-ci.yml` は cargo install（ソース
  ビルド）である一方、本リポジトリは #314 で「バージョン固定 + SHA256
  チェックサム検証済みプリビルトバイナリ」に統一済み

ジョブ単位の有効/無効切り替え入力を `rust-base-ci.yml` は持たないため、
部分採用（例: fmt ジョブのみ委譲）もできない。

### 4.2 `lint-docs.yml`

markdownlint・commitlint 等の導入を npm（`actions/setup-node` + `npx`）経由
で行う構成であり、`tools/npm-asset-build/check_static_only.py` の
`HARD_DENY_EXTS`（実行コード拡張子の allowlist 方式・既定拒否、REQ-12）が
体現するサプライチェーン方針と非整合（a11y 自動検証評価 #1076 で下した
判断と同型の理由）。commitlint 相当の検証は `tools/hooks/commit-msg-check.sh`
（npm 非依存の自前実装）が既に担っている。

### 4.3 `cargo-tool-install`

本リポジトリの CI ワークフローに `cargo install` を実行するステップが
現存しない（`grep -rn 'cargo install' .github/workflows/` で確認）。
cargo-deny は既に prebuilt+SHA256 パターン（`tools/ci/ensure-gate-tools.sh`）
で導入済み、wasm 関連ツールは `wasm-tool-install` へ置換済みのため、置換
対象・新規導入対象のいずれも存在しない。同 action はソースビルドで
バイナリの SHA256 検証を持たないため、既存パターンからの置換動機もない。

### 4.4 `idempotent-issue`

CI から自動で GitHub Issue を起票するジョブが本リポジトリに現存しない。
性能退行検知・監査結果の自動起票等を将来導入する場合の第一候補として
本節に記録し、再評価トリガー（§5）に含める。

## 5. 再評価トリガー

- `ci.yml` のベースライン部分（fmt/clippy/test/deny）を大規模再編する
  タイミングで `rust-base-ci.yml` を再評価する。ただしその時点でも
  cargo-deny 導入パターンの非整合（§4.1）が解消していない限り採用不可
- org が npm 非依存の lint 手段（markdownlint/commitlint 相当の Rust 製・
  prebuilt バイナリ配布等）を提供したとき、`lint-docs.yml` を再評価する
- CI へ `cargo install` を要するステップの追加が必要になったとき、
  `cargo-tool-install` を再評価する
- CI からの自動 Issue 起票（性能退行・監査検知等）を導入する判断が下され
  たとき、`idempotent-issue` を第一候補として採用可否を検討する

## 6. セキュリティ考慮（OWASP Top 10 観点）

- **A01/A05（アクセス制御・設定ミス）**: workflow レベルは `contents: read`
  を維持し、`pages: write` / `id-token: write` は deploy 呼び出しジョブのみに
  限定付与（現行と同一の最小権限）。reusable workflow は呼び出し側が付与
  した以上の権限を持てない
- **A06/A08（サプライチェーン・整合性）**: 参照はすべて commit SHA 固定
  （pages-deploy 呼び出しは既存採用と同一の `5982e2f5`、新規導入した
  `actions/upload-artifact` も SHA 固定 + バージョンコメント）。`latest`・
  タグ参照は使わない。見送り判断自体もサプライチェーン方針（#314
  prebuilt+SHA256、REQ-12 npm 経路排除）の維持が主根拠であり、方針の
  弱体化を伴わない
- **A01（パストラバーサル）**: pages-deploy 側（reusable workflow 実装）が
  `dist-dir` の絶対パス・`..`/`.` セグメントを fail-closed で拒否し、
  空サイトの黙示公開も防ぐ（現行の直接 deploy 実装には無かった追加防御）
- **A03（インジェクション）**: ワークフローへユーザー制御値のシェル展開を
  追加しない（`with:` の静的値のみ）
- **秘密情報**: Secrets 追加なし。OIDC id-token は deploy 呼び出しジョブに
  限定。コミット前に staged 差分のシークレット混入確認を実施した

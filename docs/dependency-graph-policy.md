# 依存グラフ上限値 運用ポリシー（草案）

## 1. 目的とトレーサビリティ

本ドキュメントは REQ-3（依存グラフの浅さ・監査可能性、`docs/spec/04-requirements.md` の REQ-3 節）が求める
「なぜその上限値か」「超過したらどうするか」「上限を守れても残るリスクは何か」を明文化するための成果物です。
Conditional Go 条件 2（依存グラフ上限の要件化）に対応し、`docs/spec/05-tasks.md` の TASK-3.1〜TASK-3.3 系列のうち
TASK-3.3（依存グラフ上限値運用ドキュメントの整備）を担います。

TASK-3.3 は 2 段階に分割されています。

- **TASK-3.3a（本ドキュメント）**: 上限値の算出根拠・超過時の対応フロー・サプライチェーンリスクの限界を
  明文化した**草案**の作成
- **TASK-3.3b（Issue #24）**: 本草案のレビュー反映・確定。TASK-3.2（build.rs 保有クレートの機械的列挙）の
  完了状況を踏まえた第 6 節の確定、および Conditional Go 条件 2 の解消判定は TASK-3.3b のスコープです

**本文書のステータス**: 草案（TASK-3.3a）。第 6 節に記載のとおり前提タスク TASK-3.2 が未完了のため、
一部の記述は「整備中」として TASK-3.3b に引き継ぎます。

## 2. 上限値と算出根拠

フレームワーク標準構成（コアクレート・SSR サーバー構成）の依存グラフに対し、次の上限値を設定します。

- **依存パッケージ数**: 標準サーバー構成で解決済み依存パッケージ 60 件以内
- **依存グラフ最大深さ**: 6 以内

根拠は PoC-2（マクロ DSL・Leptos 構成）と PoC-3（純 Rust 方式・`rws-server` 相当構成）の実測差です。

| 構成 | パッケージ件数 | 最大深さ | 出典 |
|------|--------------|---------|------|
| PoC-2（マクロ DSL・Leptos 構成） | 202 | 14 | `docs/spec/04-requirements.md` REQ-3 詳細・概要（25 行目） |
| PoC-3（純 Rust 方式・rws-server 相当構成） | 52 | 5 | `docs/spec/04-requirements.md` REQ-3 詳細・受け入れ基準、`docs/spec/03-poc/` |
| 削減率（PoC-2 → PoC-3） | 約 74% 減 | 約 64% 減 | `docs/spec/04-requirements.md` REQ-3 詳細（PoC-2/PoC-3 実測差の記述） |
| **採用上限**（`MAX_PACKAGES` / `MAX_DEPTH`） | **60** | **6** | PoC-3 実測（52 件/深さ 5）に実装拡張分の余裕を加算。`xtask/src/check_deps.rs` |

コアクレート（`rws-core` / `rws-interactive`）は外部依存パッケージ数 0 件であることを別途受け入れ基準としています
（REQ-3 受け入れ基準 1 点目）。`core/Cargo.toml` への外部クレート追加は `.claude/rules/coding-rust.md` により禁止されています。

### 執筆時点の実測値（参考）

本草案の執筆時点（2026-07-16、origin/main 相当）で `cargo run --locked -p xtask -- check-deps --package rws-core
--package xtask` を実行した結果は次のとおりです（両パッケージとも外部依存を持たないため 0/0）。

```
deps-check: packages=0/60 depth=0/6 result=PASS
deps-check: packages=0/60 depth=0/6 result=PASS
```

`rws-server`（標準サーバー構成の本体）は本草案作成時点で未実装のため、REQ-3 が本来対象とする「標準サーバー構成」の
実測値はまだ得られていません。`server` クレート実装後に計測対象へ追加し、実測値を本節に反映することが必要です
（第 4 節参照）。

## 3. 計測の定義と「正」の所在

計測の実体は `xtask` の `check-deps` サブコマンドです。

```bash
cargo run --locked -p xtask -- check-deps --package <NAME> [--package <NAME> ...]
```

定義（`xtask/src/check_deps.rs` の rustdoc と整合）:

- **件数**: `cargo metadata --format-version 1 --filter-platform <host-triple>` の `resolve.nodes` を正とし、
  ルートパッケージから `DepKind::Normal` 辺のみを辿って到達可能な一意パッケージ数（ルート自身を除く）。
  dev 依存は除外する（PoC-3 の `cargo tree -e normal` と整合）
- **深さ**: ルートを深さ 0 とした最長経路長。dev 依存を除いた解決グラフは DAG であるため、メモ化 DFS により
  厳密に算出する（`cargo tree` の `(*)` 重複省略による過小評価を避ける）
- **プラットフォーム**: `--filter-platform` にホストの target triple を渡し、ホストで有効にならない
  cfg 条件付き依存（target-specific な normal edge）を計測から除外する

しきい値の唯一の正は `xtask/src/check_deps.rs` の `MAX_PACKAGES`（60）・`MAX_DEPTH`（6）定数です。
`--locked` 実行を必須とし、CLI 引数・環境変数・`continue-on-error` 等による緩和経路は意図的に設けません
（迂回経路を作らない設計）。

CI 組み込みは `.github/workflows/deps-check.yml` が担い、fail-closed（PASS/FAIL をそのまま CI の成否に伝播）で
運用します。同ワークフローも `--locked` を必須とし、外側の `cargo run` が `Cargo.lock` を書き換えないことを
保証しています。

## 4. 計測対象パッケージ

現時点の計測対象は次の 2 パッケージです（`.github/workflows/deps-check.yml` と一致）。

- `rws-core`（ディレクトリは `core/`。外部依存ゼロ契約）
- `xtask`（外部依存ゼロ契約）

`rws-server`（標準サーバー構成の本体）が実装された後は、REQ-3 が本来意図する「標準サーバー構成」の計測対象として
`--package server` を追加することが必須です。この追加は server クレート導入イシュー側の対応事項とし、
本ドキュメントおよび `.github/workflows/deps-check.yml` のコメントに記載済みの引き継ぎ事項とします。

## 5. 上限超過時の対応フロー

1. **検出**: `deps-check` CI ジョブが FAIL する（`deps-check: packages=<n>/<limit> depth=<n>/<limit>
   result=FAIL` の 1 行サマリが Step Summary に転記される）
2. **原因分析**: `xtask check-deps` の出力・`cargo tree` を用いて、件数・深さ増加の原因となった依存を特定する
3. **原則対応（依存削減）**: 次の優先順で依存削減を検討する
   1. 不要な feature フラグの削減
   2. より依存の浅い代替クレートへの置き換え
   3. 該当機能の自前実装の検討
4. **依存追加が不可避な場合**: `.claude/rules/coding-rust.md` / `.claude/rules/security.md` に従い、
   `cargo metadata` で影響を事前確認し、`build.rs` の有無を確認したうえで、**ユーザー承認**を得てから追加する
5. **上限値自体の見直しが必要な場合**: 本リポジトリ内では変更しません。上限値は REQ-3（`frontend-framework-spec`
   リポジトリ管理）に由来するため、まず同リポジトリへ仕様変更（REQ-3 改訂）を提案し、承認を経たうえで
   `xtask/src/check_deps.rs` の定数変更 PR（レビュー必須）を行います。CI ワークフロー側に一時的な緩和手順・
   スキップ手順は設けません

## 6. build.rs 保有クレートの監査（監査可能性）

REQ-3 の受け入れ基準は「`build.rs` を持つ依存クレートの一覧が、ビルド成果物または CI ログとして機械的に
列挙できること」を求めています。

**本節のステータス: 整備中**。この機能は TASK-3.2（Issue #19 系列: #20 TASK-3.2a 列挙ロジック実装・
#21 TASK-3.2b CI 出力統合）で `xtask` のサブコマンドとして実装される計画ですが、本草案の執筆時点
（2026-07-16 時点の origin/main）では未着手であり、`xtask/src/main.rs` に `check-deps` 以外のサブコマンドは
存在しません。`check_deps.rs` には `DepKind` の分類が定義されていますが、`build.rs` 保有クレートの列挙に
特化したサブコマンド・出力形式は未実装です。

TASK-3.2 完了後、本節を実コマンド名・出力形式・CI 統合方法で確定記述に更新することを TASK-3.3b（Issue #24）
へ引き継ぎます。

## 7. サプライチェーンリスクの限界（安全性主張のスコープ）

本ポリシーが担保するのは「依存数・深さの相対的な浅さ＝監査コストの低減」であり、次の点について
過大な安全性主張は行いません。

- `build.rs`・手続きマクロによる任意コード実行は、Cargo エコシステム全体に共通する構造的リスクであり、
  本フレームワーク単体の実装方針では解消できません（PoC-1 の空白 B 判定、PoC-2 の逆転発見:
  `docs/spec/04-requirements.md` 25 行目・28 行目）
- 「上限（60 件/深さ 6）を満たしていれば安全」という読み替えは誤りです。上限は監査対象の絶対量を
  抑制するものであり、個々の依存クレートに内在する暗黙実行リスクそのものを排除するものではありません
- メモリ安全性の保証範囲は `core` / `interactive`（`#![forbid(unsafe_code)]` を設定したクレート）に
  限定されます。WASM バインディング層・FFI 依存クレートの残存リスクは `docs/unsafe-boundary.md` の
  スコープです

## 8. 草案ステータスと TASK-3.3b への引き継ぎ

TASK-3.3b（Issue #24）で以下のレビュー観点を消化し、本ドキュメントを確定させ、Conditional Go 条件 2 の
解消判定を行うことを想定しています。

- [ ] 算出根拠（第 2 節の数値表）が `docs/spec/04-requirements.md` の記述と一致しているか
- [ ] 超過時の対応フロー（第 5 節）が実際の運用として実行可能か（依存追加承認フローとの整合を含む）
- [ ] TASK-3.2（build.rs 列挙）完了後、第 6 節を実コマンド名・出力形式で確定記述に更新できているか
- [ ] `rws-server` 実装後、第 2 節・第 4 節に標準サーバー構成の実測値を反映できているか
- [ ] Conditional Go 条件 2（依存グラフ上限の要件化）の解消判定

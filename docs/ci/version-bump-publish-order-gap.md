# version バンプ PR と crates.io 公開の順序ギャップ是正方針（イシュー #884）

## 1. 背景

`templates/app`（`fw new --template app`）は fandhe-frontend-core / -app /
-interactive / -wasm-client への crates.io バージョン依存で完結する
（`docs/design/template-vendor-to-version-switch.md`）。これらのクレートの
`src/` を変更する PR では、次の 3 つの機械検証が同時に働く。

| 機構 | 場所 | 強制内容 |
|---|---|---|
| version-bump-guard（#638） | `.github/workflows/ci.yml` の `version-bump-guard` ジョブ / `crates/xtask/src/check_version_bump.rs` | 公開済みクレートの `src/`・`Cargo.toml`・`build.rs` 変更に version バンプを強制 |
| template_vendor_drift テスト | `crates/cli/tests/template_vendor_drift.rs` | `templates/app/Cargo.toml`・`templates/app/wasm/Cargo.toml` の依存要求が正本 `crates/*/Cargo.toml` の `version` と完全一致することを強制 |
| template-app-wasm-smoke ジョブ（#411） | `.github/workflows/ci.yml` の `template-app-wasm-smoke` ジョブ | `fw new --template app` 生成物を crates.io 実 index で依存解決してビルド検証 |

バンプ先バージョンは PR マージ・crates.io 公開前は index に存在しないため、
guard とドリフトテストを満たすと smoke が必ず fail する（三すくみ構造）。

### PR #872 での実例

PR #872（`feat(headless-ui): SignaturePad を実装する`、ブランチ
`feat/843-signature-pad`）で、`fandhe-frontend-wasm-client` を対象に
release ワークフロー（`mode: publish`）が同一 PR ブランチに対して 2 回実行
された。

| run ID | 実行日時 (UTC) | 対象クレート | 結果 |
|---|---|---|---|
| 30095512591 | 2026-07-24T13:05:20Z | fandhe-frontend-wasm-client | success（1 回目の公開） |
| 30100463714 | 2026-07-24T14:18:33Z | fandhe-frontend-wasm-client | success（2 回目の公開） |

両 run とも `headBranch: feat/843-signature-pad`（PR マージ前のブランチ ref）
から `cargo publish` まで到達している。1 回目の公開後に当該 PR ブランチへ
追加 push が発生し、`version-bump-guard` が「公開済み + `src/` 変更あり」を
再検知して再バンプを要求し、2 回目の公開に至ったと推定される（**再バンプ
ループ**）。この運用は release.yml が想定する「マージ後・人間の明示判断による
公開」という設計意図（後述）に反しており、レビュー未完のブランチコードが
crates.io へ複数回公開される事態を招く。

## 2. 関係コンポーネントの設計前提

- **version-bump-guard**（`.github/workflows/ci.yml`）: `pull_request` イベントで
  常時実行。crates.io sparse index への到達性を前提とし、到達不可は
  `environment error: ` プレフィックスで fail-closed に停止する。
- **template_vendor_drift テスト**（`crates/cli/tests/template_vendor_drift.rs`）:
  ネットワーク照会なし。`templates/app` の依存要求とワークスペース内正本の
  `version` の文字列一致のみを見る。
- **template-app-wasm-smoke ジョブ**（`.github/workflows/ci.yml`）: `fw new
  --template app` で生成したプロジェクトを実際に `cargo build` させ、
  crates.io 実 index に対する依存解決が成立することを検証する。path 依存や
  vendor 同梱を持たないため、バンプ先バージョンが未公開の間は必ず失敗する。
- **release.yml**（`workflow_dispatch` 起点）: `crate`（choice 型、11 クレート
  固定選択肢）・`version`（文字列、Cargo.toml の version と完全一致必須）・
  `mode`（`dry-run-only`（既定・安全側）/ `publish`）の 3 入力を取る。
  既定値が `dry-run-only` である設計意図は「実公開は明示的な人間判断を必須
  とする」ことであり、これが不可逆操作（crates.io 公開、yank のみ可能）の
  唯一の承認境界になっている。

## 3. 3 案の比較

比較軸: (a) fail-closed 性の維持 (b) 不可逆操作（crates.io 公開）の承認境界
(c) CI 運用の複雑さ (d) 再発時の手作業量。

### 案 1: マージ前ブランチ公開の正式化

PR レビュー完了後、マージ前に PR ブランチ ref から release.yml を手動実行して
crates.io へ先行公開し、guard・drift テスト・smoke をすべて green にしてから
マージする運用を正式手順化する。

- (a) 維持: smoke は実 index 検証のまま変わらない。
- (b) **最悪**: レビュー未完・マージ前のブランチコードを公開する運用が常態化
  する。公開は取消不能（yank のみ）であり、release.yml の「明示的
  `mode: publish`」設計意図（人間承認の重み）に反する運用が定着する。
  さらに PR #872 の実例が示すとおり、公開後に追加 push が入ると
  version-bump-guard が再度「公開済み + `src/` 変更あり」を検知し、再バンプ
  → 再公開の**再バンプループ**を誘発する。
- (c) 低い（手順文書のみで実装コードの変更は不要）。ただし再バンプループ
  回避手順（「公開後は push 禁止」）は人手の規律に依存し脆弱。
- (d) 高い（PR ごとに release.yml 手動実行が 1〜2 回発生する。PR #872 では
  実際に 2 回発生した）。

### 案 2: smoke ジョブの依存解決フォールバック

`template-app-wasm-smoke` ジョブに、生成プロジェクトが要求する
fandhe-frontend-* の各バージョンを sparse index へ照会し、未公開であることを
確認できた場合のみ `[patch.crates-io]`（ワークスペース内 `crates/*` への
path 指定）を注入して依存解決するフォールバックを追加する。

- (a) 条件付き維持: sparse index 照会で「未公開」を fail-closed に確認できた
  場合のみフォールバックが発動し、`::warning::` アノテーション + 1 行サマリで
  発動を明示する（サイレントなテスト弱体化にしない）。index 到達不可は
  version-bump-guard と同型の `environment error: ` で fail-closed 停止する。
- (b) **最良**: 公開はマージ後の `workflow_dispatch` による人間の明示判断の
  ままで一切動かさない。フォールバックはあくまで CI 内部の検証経路の調整で
  あり、crates.io への実公開とは無関係。
- (c) 中程度: `template-app-wasm-smoke` ジョブ 1 箇所への未公開判定ステップ
  + `[patch.crates-io]` 注入ロジックの追加が必要。sparse index 照会は
  `check_version_bump` の既存パターンを踏襲できる。
- (d) **ゼロ**: PR 作成者はマージ後の通常 release 運用（1 回の
  `workflow_dispatch` 実行）のみで完結し、ブランチ公開・再バンプ対応は不要
  になる。

### 案 3: マージ後公開の自動化

main への push をトリガーに、バンプされたクレートを自動検知して release
ワークフローを自動実行し公開する。

- (a) 弱体化: 公開が完了するまでの間、main の smoke ジョブが fail する状態を
  「既知事象」として一定期間容認することになる。
- (b) 悪い: main push を起点に不可逆操作（crates.io 公開）を自動実行する。
  release.yml の既定 `dry-run-only`（安全側・人間の明示選択を必須とする）
  という設計と正面から矛盾する。
- (c) 高い: main push トリガーの自動公開ワークフロー新設 + smoke fail を
  「公開待ち」として区別表示する機構 + `CARGO_REGISTRY_TOKEN` の自動供給
  経路拡大（現状は `workflow_dispatch` の `mode: publish` 選択時のみに限定
  注入されており、常時稼働ワークフローへの供給はサプライチェーン面の露出を
  増やす）が必要。
- (d) 低い（自動化により定常運用の手作業はほぼゼロ）。ただし公開失敗時の
  復旧は手動対応が必要。

### 比較表

| 軸 | 案 1 | 案 2 | 案 3 |
|---|---|---|---|
| (a) fail-closed | 維持 | 条件付き維持（明示アノテーション） | 弱体化 |
| (b) 承認境界 | 最悪（常態的ブランチ公開・再バンプループ） | 最良（不変） | 悪い（自動公開） |
| (c) 複雑さ | 低（ただし運用脆弱） | 中 | 高 |
| (d) 手作業量 | 高（PR #872 で実際に 2 回発生） | ゼロ | 低（復旧時のみ手動） |

## 4. 採用案と理由

**採用案: 案 2（smoke ジョブの依存解決フォールバック）**。

理由:

- 不可逆操作（crates.io 公開）の承認境界を一切動かさない唯一の案である。
  security.md のサプライチェーン観点（依存追加・公開操作は脅威面の拡大）に
  照らして最重要の判断軸であり、他の 3 軸より優先する。
- フォールバックは「バンプ先バージョンが sparse index に未公開であることを
  確認できた場合のみ」発動し、発動時は `::warning::` アノテーション + 1 行
  サマリで明示するため「テストの弱体化」に当たらない。index 到達不可は
  fail-closed に停止し、緩和用の workflow_dispatch 入力・環境変数は設けない
  （既存の迂回禁止原則を踏襲）。
- むしろ `[patch.crates-io]` 経由の path 依存は「これから公開されるコード
  そのもの」を検証する経路であり、検証対象の実質は crates.io 実解決版より
  むしろ向上する（先行公開されていない未レビューコードを誤って公開する
  リスクがそもそも生じない）。
- 案 1 は緊急時の暫定手順として §6 に最小化して記録するが、正式運用には
  しない（PR #872 の再バンプループが示す運用脆弱性のため）。
- 案 3 は再評価トリガー（§7）付きで不採用として記録する。

## 5. 実装スコープ（後続 issue へ引き渡す粒度）

本イシューでは設計・文書化のみを行い、以下は後続 issue のスコープとする。

1. **未公開判定ステップの追加**: `template-app-wasm-smoke` ジョブに、生成
   プロジェクトの `templates/app/Cargo.toml`・`templates/app/wasm/Cargo.toml`
   が要求する fandhe-frontend-* 各クレートのバージョンを sparse index
   （`https://index.crates.io`）へ照会するステップを追加する。`curl` には
   `--connect-timeout`・`--max-time` を付与し、到達不可・異常応答（HTTP
   非 200 系・body 空/パース不能）は `environment error: ` プレフィックスで
   fail-closed 停止する（`check_version_bump` の `query_index` と同型）。
2. **`[patch.crates-io]` 注入によるフォールバック**: 未公開バージョンが 1 つ
   でもあれば、生成プロジェクトのルート `Cargo.toml`・`wasm/Cargo.toml` へ
   `[patch.crates-io]`（ワークスペース内 `crates/*` への path 指定）を注入
   して依存解決する。未公開バージョンは `Cargo.lock` の再生成自体が crates.io
   側で不能なため、フォールバック発動時は `--locked` を外す判断が必要になる
   （この場合の再現性低下は許容し、その旨をログに明記する）。
3. **アノテーション・サマリ契約**: フォールバック発動時は `::warning::` +
   Step Summary への 1 行サマリ（契約例:
   `template-app-wasm-smoke: dep=<crate> version=<v> resolution=<crates-io|path-override>`）
   を出力する。全依存が公開済みであればフォールバック経路には一切入らず、
   従来どおり crates.io 実解決のみで完結する。
4. **緩和経路を設けない**: workflow_dispatch 入力・環境変数によるフォール
   バックの有効/無効切り替えは設けない（既存の迂回禁止原則と同型）。
5. **回帰テスト方針の検討事項**: サマリ行契約の固定方法（grep ベースの
   CI アノテーション抽出契約とするか）、`[patch.crates-io]` 注入ロジックを
   xtask 側のサブコマンドとして切り出し単体テスト可能にするか、あるいは
   ワークフロー YAML 内のシェルスクリプトとして完結させるかは後続 issue で
   検討する。

### 実装結果（イシュー #885）

上記 5 項目はすべて xtask サブコマンド `patch-template-smoke`
（`crates/xtask/src/patch_template_smoke.rs`、`main.rs::run_patch_template_smoke`）
として実装済み。

```
cargo run --locked -p xtask -- patch-template-smoke \
  --project-dir <fw new が生成したプロジェクト> \
  --repo-root <リポジトリ checkout の絶対パス> \
  [--index-base-url <URL>]   # テスト専用の差し替え口（既定 https://index.crates.io）
```

- `check_version_bump::query_index`（sparse index 照会の fail-closed 契約）を
  そのまま再利用し、pin・契約の二重管理を避けた。
- 対象は生成プロジェクトのルート `Cargo.toml`・`wasm/Cargo.toml` の直接依存
  （`fandhe-frontend-* = "X.Y.Z"` 形式）。テーブル形式・path 依存の検出、
  既存 `[patch.crates-io]` セクションの検出はいずれも fail-closed エラーとする
  （想定外状態を無条件に上書きしない）。
- 依存 1 件ごとに 1 行サマリ
  `template-app-wasm-smoke: dep=<crate> version=<v> resolution=<crates-io|path-override>`
  を出力する契約（`patch_template_smoke::format_dep_report`）。
- `.github/workflows/ci.yml` の `template-app-wasm-smoke` ジョブは「fw new」
  ステップの直後に本サブコマンドを実行する。`resolution=path-override` の
  発動時は `::warning::` アノテーション + Step Summary への転記でサイレントな
  弱体化にしない。index 到達不可・異常応答は `environment error: `
  プレフィックス付きで fail-closed に停止し、version-bump-guard と同型の
  `if PIPELINE; then ... fi` 判定パイプラインで「runner/ネットワーク起因」と
  「コード起因」を CI アノテーションとして区別する。
- 発動有無を切り替える workflow_dispatch input・環境変数は設けていない
  （項目 4 の方針どおり）。
- CLI 契約の回帰テストは `crates/xtask/tests/cli_patch_template_smoke.rs`
  （終了コード・1 行サマリ書式・`[patch.crates-io]` 注入・`Cargo.lock` 削除・
  各エラー分類を固定）。

## 6. 暫定運用（案 2 実装済み（#885）につき原則不要）

案 2（`patch-template-smoke` フォールバック）は実装済みであり、
templates/app が依存する crates.io バージョン依存クレートのバンプ PR は
本フォールバックにより smoke ジョブが green のまま進行できる。以下は
案 2 実装前に運用していた緊急手順の記録であり、正式運用として推奨するもの
ではない（フォールバックが何らかの理由で機能しない場合の最終手段としてのみ
参照する）。

1. PR を最終形まで完成させ、レビュー完了後に release.yml を PR ブランチ ref
   から `mode: publish` で実行する。
2. **公開後は当該 PR へ一切 push しない**（再バンプループ防止。PR #872 の
   実例が示すとおり、公開後の追加 push は version-bump-guard の再検知 →
   再バンプ → 再公開を招く）。
3. 失敗している smoke ジョブのみ re-run する。
4. green 化を確認したら即マージする。
5. やむを得ず追加 push が必要になった場合は、再バンプおよび再公開を受け
   入れる（PR #872 と同じ経路をたどることを許容する）。

## 7. 不採用案の再評価トリガー

案 3（マージ後公開の自動化）は、案 2 運用後もテンプレート依存クレートの
バンプ頻度が高く、マージ後の release.yml 手動実行が月あたり一定回数
（目安: 月 4 回超）の負担となった場合に再評価する。再評価時は
`CARGO_REGISTRY_TOKEN` の自動供給経路拡大に伴うサプライチェーン面の露出増
（security.md A08）を主要な検討事項とする。

## 8. `.claude/rules/ci.md`・CLAUDE.md への反映

本イシュー（#884）に対応する PR で、`.claude/rules/ci.md` の
`version-bump-guard` ジョブの記述末尾へ本文書への参照を追記済みである
（受け入れ条件「`.claude/rules/ci.md` から設計文書への参照が追記されている」
を充足）。CLAUDE.md の `docs/ci/` 列挙にも本文書名を追加済み。

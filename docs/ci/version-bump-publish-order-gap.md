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
| `test` ジョブの app テンプレート gate e2e（#351、実質は三すくみの第 3 の当事者） | `.github/workflows/ci.yml` の `test` ジョブ・`crates/cli/tests/new_gate_e2e.rs::fw_new_app_template_output_passes_fw_gate`/`fw_new_app_template_default_escape_check_detects_injected_violation` | `fw new --template app` 生成物へ `fw gate`（内部で crates.io 実 index での依存解決を伴う）を実行して PASS/期待 FAIL を検証 |

バンプ先バージョンは PR マージ・crates.io 公開前は index に存在しないため、
guard とドリフトテストを満たすと smoke・app テンプレート gate e2e の双方が
必ず fail する（三すくみ構造。実際には smoke ジョブと test ジョブの 2 経路が
同型の症状を示す）。

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

本イシューでは設計・文書化のみを行い、以下は後続 issue のスコープとする
（当初の対象は smoke ジョブのみだったが、#895 で `test` ジョブの app
テンプレート gate e2e にも同型のデッドロックが残存していたことが判明し、
スコープを両ジョブへ拡張した。詳細は「実装結果（イシュー #895）」参照）。

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

### 実装結果（イシュー #895）

#885 の対応範囲は `template-app-wasm-smoke` ジョブのみだったため、
`.github/workflows/ci.yml` の `test` ジョブが実行する
`crates/cli/tests/new_gate_e2e.rs::fw_new_app_template_output_passes_fw_gate`
（および `fw_new_app_template_default_escape_check_detects_injected_violation`）
には同型のデッドロックが残存していた。イシュー #895 で `xtask
patch-template-smoke` をテスト側から再利用する形で是正した。

- `new_gate_e2e.rs::apply_patch_template_smoke`（テストローカルヘルパー）
  が `cargo run --locked -p xtask -- patch-template-smoke --project-dir
  <scratch> --repo-root <このリポジトリの checkout 絶対パス>` をサブプロセス
  として起動する。`Command` への引数個別渡し（シェル非経由）であり、
  `project_dir`/`repo_root` はテストが自ら生成した scratch パス・
  `canonicalize()` 済みリポジトリルートで外部入力ではない。
- `xtask` のライブラリ化・`fandhe-frontend-cli` への dev-dependency 追加は
  行わなかった（`fandhe-frontend-cli` は crates.io 公開済みかつ外部依存ゼロ
  方針を維持しており、依存グラフへの影響を避けるため）。サブプロセス方式は
  依存グラフを一切変えず、smoke ジョブと完全に同一の CLI 契約（1 行サマリ・
  `environment error: ` プレフィックス・終了コード）を再利用できる。
- `fw gate` は cargo 起動へ常に `--locked` を付与するため、フォールバック
  発動時（`[patch.crates-io]` 注入に伴いルート `Cargo.lock` が削除される）は
  テスト側で `cargo generate-lockfile` を実行して再生成してから `fw gate` を
  呼ぶ。再現性低下は smoke ジョブ側と同じく既知・許容の判断（本文書 §5
  項 2 の判断をテスト経路にも踏襲）。`wasm/Cargo.lock` は `fw gate` の対象
  外（`fw gate` はプロジェクトルートのみをビルド対象にする）のため再生成
  不要。
- `.github/workflows/ci.yml` の `test` ジョブは、従来の 2 ステップ
  （`--skip example_` / `example_` フィルタ）を 3 ステップへ再編した:
  1. `--skip example_ --skip fw_new_app_template`（examples・app テンプレート
     を除く残り 7 件）
  2. `fw_new_app_template` フィルタ（app テンプレート gate e2e 2 件）。
     `resolution=path-override` を含む出力があれば `::warning::` + Step
     Summary へ転記し、失敗時は `environment error: ` プレフィックス有無で
     「runner/ネットワーク起因」と「コード起因」を CI アノテーションとして
     区別する（version-bump-guard・smoke ジョブと同型の判定パイプライン）。
  3. `example_` フィルタ（examples e2e 5 件、従来どおり）
  3 フィルタが相互排他かつ `new_gate_e2e.rs` 全 14 件を重複・漏れなく
  網羅することは `cargo test ... -- --list` で確認済み（7 + 2 + 5 = 14）。
- サマリ行プレフィックス `template-app-wasm-smoke: ` はテスト経路でも
  そのまま共用する（呼び出し元別のプレフィックス化は契約変更を伴うため
  見送り。`patch_template_smoke.rs`/`main.rs` のドキュメンテーションコメント
  へ呼び出し元が 2 経路になった旨を追記済み）。
- 緩和用の環境変数・CLI フラグは追加していない（項目 4 の迂回禁止原則を
  テスト経路にも継続適用）。
- 全依存が公開済みの通常時（本 PR 時点の `templates/app` 依存はすべて
  公開済み）は `apply_patch_template_smoke` がファイルを一切変更せず
  `resolution=crates-io` のみを報告してそのまま `fw gate` へ進むことを
  ローカル実行で確認済み（crates.io 実解決の検証能力は不変）。

## 6. 暫定運用（案 2 実装済み（#885）につき原則不要）

案 2（`patch-template-smoke` フォールバック）は実装済みであり、
templates/app が依存する crates.io バージョン依存クレートのバンプ PR は
本フォールバックにより smoke ジョブが green のまま進行できる。以下は
案 2 実装前に運用していた緊急手順の記録であり、正式運用として推奨するもの
ではない（フォールバックが何らかの理由で機能しない場合の最終手段としてのみ
参照する）。**codex-review 導入後（イシュー #1306）は §10「同時公開フロー」が
正式な選択肢であり、下記手順 2 の「公開後は当該 PR へ一切 push しない」は
§10.2 項目 3・4 の「lock 再生成コミットは許容し、それ以外の追加変更・
force-push を禁止する」条件付き許容へ読み替える。**

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

上記トリガーに基づく実際の再評価を実施した結果は §9（イシュー #896）に
記録済みである。

## 8. `.claude/rules/ci.md`・CLAUDE.md への反映

本イシュー（#884）に対応する PR で、`.claude/rules/ci.md` の
`version-bump-guard` ジョブの記述末尾へ本文書への参照を追記済みである
（受け入れ条件「`.claude/rules/ci.md` から設計文書への参照が追記されている」
を充足）。CLAUDE.md の `docs/ci/` 列挙にも本文書名を追加済み。

## 9. 案 3 の再評価（イシュー #896）

§7 の再評価トリガーに基づき、案 2（`patch-template-smoke` フォールバック）
実装後の運用実績を踏まえて案 3（マージ後 crates.io 公開の自動化）を
再評価する。構成は `docs/ci/cargo-semver-checks-evaluation.md`（イシュー
#656）と同型（運用実績の集計 → 評価 → 結論 → 再評価トリガー → 将来参考メモ）
とする。

### 9.1 運用実績の集計

- **観察対象・起点**: 案 2 はイシュー #885 で `xtask patch-template-smoke`
  として実装され、PR #893（マージコミット `2aefada`、
  2026-07-24T15:55:48Z（UTC。ローカル表記では 2026-07-25T00:55:48+09:00）
  マージ）で main へ入った。案 3 が対象とする「案 2 運用後のマージ後
  release.yml 手動実行」の集計はこの時刻以降を起点とする。
- **本評価作成時点**: 2026-07-24（マージから数時間程度）。**観察期間は
  実質ゼロ**であり、cargo-semver-checks 評価（イシュー #656、観察期間
  実質 1 日・PR 1 件で見送り結論）よりもさらに短い。
- **案 2 マージ以降のマージ後 release.yml 実行回数**: 0 件。
  `gh run list --workflow release.yml` の全履歴（直近 50 件、
  2026-07-21T15:35Z〜2026-07-24T14:18Z）はいずれも案 2 マージ時刻
  （2026-07-24T15:55:48Z）より前であり、マージ後の release run 自体が
  まだ 1 件も発生していない。
- **案 2 マージ以前の実績（参考。案 3 の判定対象には含めない）**:
  `headBranch` 別に分類すると、`main`（通常のマージ後手動公開、初回一括
  公開 #608 等を含む）が大半を占める一方、PR ブランチからの緊急公開が
  2 系統確認できる: `feat/843-signature-pad`（PR #872 の再バンプループ、
  同一クレートへの publish run が 2 回連続）・
  `fix/632-embedded-example-drift`（同一 PR ブランチへの release run が
  4 回連続）。これらはいずれも案 2 導入前の「三すくみデッドロック」回避を
  目的とした緊急手順（§6）の実例であり、案 2 導入後は smoke ジョブの
  フォールバックにより発生しなくなる想定の事象である。案 3 の再評価トリガー
  （§7、「月 4 回超」）が対象とするのは案 2 運用下でのマージ後手動実行の
  負担であり、これらの導入前の緊急公開実績はその判定に含めない。
- **案 2 マージ直後の main CI（run 30107372939、2026-07-24T15:55:52Z 開始）**:
  `fw new --template app` の wasm ビルド込み e2e ジョブを含め全ジョブ
  green。ただし本 run は「フォールバック実装コードがマージ後の CI でも
  green であること」を確認したのみで、実際にバンプ済み未公開バージョンに
  対してフォールバックが発動した実績（`resolution=path-override`）では
  ない（案 2 マージ時点でテンプレート依存クレートに未公開バンプは存在
  しないため）。
- **評価可能になる条件**: 以下のいずれかを満たした時点で改めて定量評価を
  行う。
  1. 案 2 運用下でテンプレート依存クレート（core / app / interactive /
     wasm-client）の version バンプを伴う PR が 3 件以上マージされ、
     マージ後 release.yml 実行の所要時間・頻度が観測できること。
  2. 案 2 マージから観察期間が 1 か月以上経過していること。
  3. `resolution=path-override`（フォールバック発動）が 1 回以上記録され、
     発動から実際の crates.io 公開までのリードタイムが実測できること。

集計に用いたコマンド（再現用。PR 本文・ログはユーザー制御の信頼しない
入力のため、シェルへ直接展開せず `--json`/`--jq` の構造化出力のみを
参照した）:

```bash
git log --format='%H %cI %s' -1 2aefada

gh run list --workflow release.yml \
  --json databaseId,createdAt,headBranch,conclusion -L 50

gh run list --workflow ci.yml --branch main \
  --json databaseId,createdAt,conclusion -L 20

gh run view <run-id> --json jobs \
  --jq '.jobs[] | select(.name | contains("template-app-wasm-smoke")) | {name,conclusion}'
```

### 9.2 論点評価

§3 の案 3 評価時点から状況が変化したかを 3 論点で確認する。

1. **不可逆操作の承認境界**: release.yml は現在も `workflow_dispatch` +
   `mode: publish` の明示選択のみを公開の承認境界としている（§2）。
   main push トリガーへ置き換える場合、fail-closed に定義するために
   最低限次の要素が必要になる: (a) バンプ検知の決定的判定
   （`check_version_bump` が既に持つロジックを流用可能）、(b) 対象クレート
   の許可リスト固定（release.yml の `crate` choice 型入力と同じ 11 件
   固定）、(c) 公開失敗時に後続の自動公開を止め手動復旧に倒す挙動、
   (d) 二重公開防止（sparse index の既公開チェックを `query_index` から
   流用可能）。これらを実装しても、「レビュー済み main のコードのみが
   対象になる」利点と「公開の実行自体に人間の明示判断が介在しなくなる」
   というトレードオフは変わらず残る。案 2 採用時の判断
   （§4「不可逆操作の承認境界を一切動かさない唯一の案」を他の 3 軸より
   優先する）から状況の変化はない。
2. **`CARGO_REGISTRY_TOKEN` の供給経路**: 現状は `workflow_dispatch` の
   `mode: publish` 選択時のみステップ限定注入（§2・`.claude/rules/ci.md`）
   であり、これを push トリガーの常時稼働ワークフローへ拡大することは
   トークンの到達可能面を増やす（security.md A08 相当。悪意ある/侵害された
   依存や Action がワークフロー実行のたびにトークンへ到達し得る経路が
   常設される）。`pull_request` 系イベントとの分離維持（フォーク PR から
   トークンへ到達できない設計）は自動化後も維持できるが、
   `workflow_dispatch` 限定という現在の「発火条件そのものが人間操作」な
   構造は失われる。この評価軸も §3 の案 3 評価から変化していない。
3. **依存順公開の自動化可否**: release.yml は現状クレート横断の順序保証を
   持たない（§2）。自動化する場合は `cargo metadata` ベースの依存グラフを
   トポロジカルソートし、公開後の sparse index 反映待ち（headless-ui →
   pre-styled-ui のバンプ時に実際に必要だった反映確認、`.claude/rules/ci.md`
   の `dep-version-check` 項参照）を組み込む実装が要る。
   `xtask check-dep-versions`（イシュー #657）が既に workspace 内の
   `path + version` 併記依存グラフを `cargo metadata --no-deps` から
   構築しているため、依存順序の抽出自体は実装可能性として既存資産を
   流用できる。ただし本評価時点でこの部分の実装は行っていない
   （実装可否の評価に留める）。

### 9.3 結論

**現時点では見送りを継続する。** 根拠:

1. **観察期間が実質ゼロ**: 案 2 マージから本評価作成までの間にマージ後
   release.yml 実行が 1 件も発生しておらず、§7 の再評価トリガー
   （「月 4 回超」）を判定する材料が存在しない。cargo-semver-checks 評価
   （観察期間 1 日で見送り）よりもさらに短い観察期間で「採用」へ倒す
   根拠はない。
2. **承認境界・トークン露出の論点に状況変化がない**: §9.2 の論点 1・2 は
   案 2 採用時（§4）の評価から変わっておらず、案 3 の本質的なリスク
   （不可逆操作の自動化・トークン供給経路拡大）は解消されていない。
3. **案 2 により当面のブロッカーは解消済み**: 案 2（`patch-template-smoke`）
   により PR 進行を妨げていた三すくみデッドロック（§1）は解消されており、
   案 3 が解決しようとしていた残課題は「マージ後の release.yml 手動実行が
   1 回発生する」という定常運用上の軽微な手作業のみである。この手作業量
   （§7 の目安: 月 4 回超）が負担化している実績がまだない以上、
   不可逆操作の承認境界を弱める変更に踏み切る理由がない。

### 9.4 再評価トリガー（更新）

§7 のトリガーを踏襲しつつ、以下を追加する。次のいずれかに該当した場合、
再評価のためのイシューを起票する:

- 案 2 運用下でのマージ後 release.yml 手動実行が月あたり 4 回超に達した
  場合（§7 の既存トリガー）。
- `patch-template-smoke` のフォールバック発動（`resolution=path-override`）
  が恒常化し、フォールバック発動から実際の crates.io 公開までのリードタイム
  （バンプ先バージョンが未公開のまま生成プロジェクトが妥当性検証を受け続ける
  期間）が定常的な問題として顕在化した場合。
- 公開クレート数・公開頻度が大きく増え、マージ後公開の自動化による手作業
  削減効果が、承認境界を弱めるリスクを明確に上回ると判断できる材料が
  揃った場合。

### 9.5 採用する場合の実装方式メモ（将来の再評価用参考）

- トリガー判定: `check_version_bump` の crates.io sparse index 照会
  （`query_index`）と `cargo metadata` によるバンプ検知ロジックをそのまま
  再利用する。
- 対象クレートの許可リスト: release.yml の `crate` choice 型入力にある
  固定選択肢（11 件）をそのまま流用し、リスト外のクレートは対象にしない
  （新規クレート追加時は明示的なリスト更新を要求する）。
- 依存順制御: `xtask check-dep-versions`（イシュー #657）が構築する
  workspace 内 `path + version` 併記依存グラフを流用し、トポロジカル
  ソート順に公開する。公開後は sparse index への反映を
  `check_version_bump::query_index` 相当のポーリングで確認してから次の
  クレートへ進む。
- トークン供給: 現行の「`mode: publish` 選択ステップにのみ限定注入」の
  原則を維持し、自動実行ワークフローでも「公開を実行するジョブ内の
  該当ステップの `env:` にのみ注入・ログ非出力」を踏襲する。
  `pull_request` 系イベントとの分離（フォーク PR からトークンへ到達
  できない構成）は必須要件とする。
- 失敗時の挙動: 公開失敗時は後続クレートの自動公開を停止し、手動復旧
  （release.yml の `workflow_dispatch` 実行）へフォールバックする
  「fail-closed」設計とする。既存の `environment error: ` プレフィックス
  による環境エラーとコード起因失敗の区別規約（`docs/design/gate-design.md`
  §2.3a）を踏襲する。
- 二重公開防止: `check_version_bump::query_index` による既公開チェックを
  公開直前にも再照会し、トリガーから公開実行までの間に他経路（手動
  `workflow_dispatch` 等）で先に公開済みになっていた場合はスキップする。

## 10. 同時公開フロー（ユーザー決定 2026-08-10、イシュー #1306）

### 10.1 背景（codex-review 導入後の新たなデッドロック）

案 2（`patch-template-smoke`）は `template-app-wasm-smoke` ジョブ・app テンプレート
gate e2e の三すくみ（§1・§5）を解消したが、`templates/app/wasm/Cargo.lock`
自体の再生成は依然として crates.io 側にバンプ先バージョンが公開されるまで
不能である（`Cargo.lock` はチェックサム込みでロックするため、未公開バージョン
に対しては生成できない）。従来はこの lock 再生成を公開後の後続 PR へ先送りする
運用だった（PR #1148 → #1150 が先例）。

しかし codex-review（イシュー #1275/PR #1278 で導入、P0/P1 指摘は必須マージ
条件）は stale な `Cargo.lock`（依存先バージョンと不整合な lock）を P1 として
検知するため、「バンプ PR 内では lock を更新できない」「lock を更新しない
PR は codex-review が P1 でブロックする」という新たな構造的デッドロックが
PR #1304 で顕在化した。後続 PR への先送りは codex-review 導入前提では
機能しない。

### 10.2 許容するフロー

上記デッドロックの解消として、ユーザーは「バンプ PR が open のまま、当該 PR
ブランチを ref として release.yml（`mode: publish`）で先行公開し、同一 PR 内で
`templates/app/wasm/Cargo.lock`（および `crates/cli/templates/` の同梱コピー）
を公開済みバージョンで再生成してから merge する」フロー（**同時公開フロー**）
を許容する 2026-08-10 に決定した。適用条件は以下のすべてを満たすこととする。

1. **verify（dry-run）を先行実行して green を確認してから `mode: publish` を
   実行する**: release.yml の `mode: dry-run-only`（既定・安全側、§2）で
   `cargo package`/`cargo publish --dry-run` が通ることを確認する。ただし
   dry-run の green は `mode: publish` 実行の**必要条件の 1 つ**にすぎず、
   単独では十分条件にならない（次項参照）。
2. **`mode: publish` 実行前に、PR の CI・codex-review の状態を確認する**:
   公開は不可逆操作（yank のみ可能、取消不能）であるため、dry-run 成功のみを
   根拠に実行しない。実行前に次の両方を満たすことを確認する。
   - **CI**: 当該 PR の CI チェックのうち、「未公開バージョン起因で構造的に
     fail するもの」（`template_vendor_drift`・`template-app-wasm-smoke`・
     `version-bump-guard`・codex-review の stale lock 指摘。項目 4・5 で
     個別に扱う）を除く全チェックが green であること。これらを除いた時点で
     red のチェックが 1 件でもあれば、その原因を解消してから公開する
     （未公開バージョン起因以外の red を「公開すれば直る」と誤認して素通り
     させない）。
   - **codex-review**: 直近の codex-review 実行結果を確認し、findings が
     「template lock の未公開バージョン起因の stale 指摘」（§10.1 で説明した
     既知パターン。lock のチェックサムが未公開バージョンと整合しないことを
     指摘するもので、公開により解消される）**のみ**であることを確認する。
     それ以外の P0/P1 指摘が 1 件でも残っている場合は、修正 push → codex
     再実行で green 化するか、指摘が上記の既知パターンのみに収束するまで
     `mode: publish` を実行しない（未確認の P0/P1 を残したまま公開しない）。
3. **公開順序は依存グラフに従う**: workspace 内 `path + version` 併記依存
   （`xtask check-dep-versions`〔イシュー #657〕が構築するグラフと同じ）の
   トポロジカル順で公開する（例: `fandhe-frontend-wasm-client` を公開してから
   これに依存する `fandhe-frontend-wasm-full` を公開する）。依存先が sparse
   index へ反映される前に依存元を公開すると `cargo publish` 自体が失敗する
   ため、この順序は正しさの前提でもある。
4. **公開後は同一 PR 内で template lock を速やかに再生成する**:
   公開完了（sparse index への反映確認、`check_version_bump::query_index` 相当の
   照会で確認できる）後、`templates/app/wasm/Cargo.lock` と
   `crates/cli/templates/`（`fw new --template app` 埋め込み用の同梱コピー、
   `template_publish_copy_drift.rs` がバイト一致を検証）を公開済みバージョンで
   再生成し、`template_vendor_drift`・`template-app-wasm-smoke` を green 化する。
5. **`version-bump-guard` は `version-bump-exempt` 宣言で免除する**:
   本 PR から当該クレートを実際に公開した後は、`version-bump-guard`
   （`.github/workflows/ci.yml`・`crates/xtask/src/check_version_bump.rs`）
   の判定条件（「公開済みクレートの `src/`・`Cargo.toml`・`build.rs` に base
   比の差分がある」かつ「`version` が crates.io 既公開バージョン」）を PR
   ブランチ自身が満たしてしまい、再実行のたびに FAIL する（§1 の PR #872
   「再バンプループ」と同型の検知。項目 7 の force-push 禁止だけでは防げない、
   push なしの再実行でも同じ判定になるため）。この経路は
   `.claude/rules/coding-rust.md`・`.claude/rules/ci.md` が定める既存の免除
   手段（PR 本文へ `version-bump-exempt: <crate-name>`（クレート名の完全一致・
   理由を同一行に記載）を宣言する）を使う。「公開は本 PR から実施済み
   （イシュー #1306 の同時公開フロー）」等、公開済みである旨を理由として明記
   する。包括免除（クレート名を伴わないマーカーのみ）は認めない
   （security.md A05）。
6. **merge 前に green 化を確認する**: `template_vendor_drift`・
   `template-app-wasm-smoke`・`version-bump-guard`（免除適用込み）・
   codex-review のいずれも green であることを確認してから merge する
   （項目 2 は `mode: publish` 実行前の確認であり、本項目は lock 再生成
   （項目 4）・免除宣言（項目 5）の反映後、最終的に merge 可能な状態に
   なっていることを再確認するもの）。
7. **公開実行後の当該ブランチへの force-push・公開済みバージョンに影響する
   追加変更は禁止する**: crates.io は yank 以外で取り消せないため、`mode:
   publish` 実行時点のコード内容が当該バージョンとして確定する。公開後に
   同一バージョンのソース内容を変える追加コミット（force-push によるものを
   含む）は、公開物と PR 上のコードが乖離する事態を招くため行わない
   （lock 再生成・ドキュメント調整等、公開済みバージョンの実体に影響しない
   変更は対象外）。この制約は §1 の PR #872「再バンプループ」が示した問題
   （公開後の追加 push が version-bump-guard を再度発火させ再公開を招く）
   を回避する目的も兼ねる。
8. **`mode: publish` の明示選択という承認境界は不変とする**: 自動化しない。
   トークン供給経路（`CARGO_REGISTRY_TOKEN` の `mode: publish` ステップ限定
   注入、§2・`.claude/rules/ci.md`）もそのまま維持する。

### 10.3 残存リスク

マージされずに reject された PR であっても、`mode: publish` を実行済みで
あれば当該バージョンは crates.io 上に残る（crates.io の設計上、公開の取消は
yank のみで実体の削除ではない）。同時公開フローを適用する場合は、実行前に
「このバージョンは merge されなくても crates.io に残り続ける」ことを承知の
うえで判断する。この残存リスクは案 1（§3「マージ前ブランチ公開の正式化」）が
「(b) 最悪」と評価された理由と同根であり、同時公開フローはこのリスクを
解消するものではなく、codex-review 導入後のデッドロック回避のために限定的に
許容するものである。§6 の暫定運用（緊急手順）とは異なり、本フローは
codex-review 前提下での正式な選択肢の一つとして位置づける（§5 の
`patch-template-smoke` フォールバックで解消できない「lock 自体の再生成」
という残課題に対する対処であり、両者は排他ではなく併用され得る）。

### 10.4 従来フロー（後続 PR での lock 再生成）との関係

codex-review 導入前は「バンプ PR 内では lock を更新せず、公開後の後続 PR で
まとめて再生成する」運用（PR #1148 → #1150）が既定だった。codex-review が
stale lock を P1 として検知するようになったため、**同時公開フローを既定の
選択肢とする**。後続 PR への先送りは、codex-review の P1 判定が何らかの理由で
効かない場合（例: 一時的な codex-review 停止時）に限る例外的経路として残す。

# 実ブラウザハイドレーション検証レポート（TASK-6.3d・Conditional Go 条件 1 解消判定）

## 1. 目的とトレーサビリティ

- TASK-6.3【Conditional Go 条件 1】（親イシュー #64・クローズ済み、REQ-6）は、
  ハイドレーション（`hydrate()`）の実ブラウザでの動作（クリックイベント発火・
  状態復元）を `wasm-pack test --headless` 等の実ブラウザ検証基盤で正式に実証する
  タスク（REQ-6 受け入れ基準 4「【Conditional Go 条件 1・宿題】」、
  `docs/spec/04-requirements.md`）。
- 4 分割サブタスクの内訳:
  - TASK-6.3a（#65・クローズ済み）: CI `browser-test` ジョブ（`wasm-pack test
    --headless --chrome`、wasm-pack v0.13.1 チェックサム固定・ランナー内蔵
    chromedriver 明示指定）+ `docs/guides/browser-testing.md` を整備。
  - TASK-6.3b（#66・issue は open だが成果物は PR #241 で main へマージ済み）:
    `wasm-client/tests/hydration_browser.rs`（6 テスト関数・5 観点）を実装。
  - TASK-6.3c（#67・issue は open）: 実証実行と不具合修正。
  - TASK-6.3d（本ドキュメント・#68）: 検証レポート作成・Conditional Go 条件 1
    （TASK-6.3 側）の解消判定。
- 本ドキュメントは TASK-6.3d の成果物（検証レポート）であり、main の実体（テスト
  ファイル）と CI 実行証跡（`browser-test` ジョブ）を根拠に、Conditional Go 条件 1
  のうち TASK-6.3（ハイドレーション実証）側の解消可否を判定する。条件 1 は
  TASK-6.3 と TASK-11.5（性能実証）の両輪で解消される構成であり、性能側は
  `docs/reports/perf-browser-report.md`（TASK-11.5c・#88）で既に **Go 判定済み**。本
  ドキュメントの判定と合わせることで、条件 1 全体の解消の証跡が両側揃う。
- 執筆時点（2026-07-17）のスナップショットであることを明記する。#66・#67 の
  issue クローズ操作は本ドキュメントのスコープ外（各担当・親イシュー側の
  責務）であり、以降の状態変化はこのレポートの判定を無効化しない
  （成果物は既に main にマージ済みで、CI で継続的に検証されているため）。

## 2. 判定ステータス: Go — TASK-6.3（ハイドレーション実証）側の Conditional Go 条件 1 は解消

`wasm-client/tests/hydration_browser.rs`（PR #241、main へマージ済み・2026-07-17T10:43:06Z）
が実ブラウザ検証基盤（`wasm-pack test --headless --chrome`）上で 5 観点・6 テスト
関数として実装済みであり、main 上の直近 CI run（run ID 29597370869、2026-07-17
16:44:53Z 開始、結論 success）の `browser-test` ジョブログで、当該 6 テストが
実際に headless Chrome 上で実行され全て pass したことを確認した（第 4 節）。
したがって TASK-6.3【Conditional Go 条件 1】は **解消（Go）** と判定する。

TASK-11.5 側（`docs/reports/perf-browser-report.md`）も Go 判定済みのため、
Conditional Go 条件 1（実ブラウザでの正式実証、`docs/spec/06-roadmap.md` 第
11 行）は **両輪とも解消**しており、`docs/spec/06-roadmap.md` MS-3 完了時の
Go/No-Go 確認（同ファイル第 155 行）における条件 1 の判断材料が揃った。

## 3. 判定基準（実証すべき 5 観点）

`docs/api/hydration-api.md` 第 3・6 節の不変条件、および REQ-6 受け入れ基準に対応する
5 観点（`wasm-client/tests/hydration_browser.rs` 冒頭 doc comment に明記済み）。

| # | 観点 | 対応テスト関数 |
|---|------|---------------|
| 1 | SSR/SSG 出力との整合（CSR の DOM 反映） | `mount_csr_reflects_same_render_output_as_ssr` |
| 2 | サーバー出力済み DOM の非再構築 | `hydrate_does_not_rebuild_server_rendered_dom` |
| 3 | クリックイベント発火 | `hydrate_toggles_liked_class_on_click_and_untoggles_on_second_click` |
| 4 | 状態復元（既存状態保持・再ハイドレーション） | `hydrate_preserves_pre_existing_liked_state` / `re_hydrate_preserves_click_state_and_fires_exactly_once` |
| 5 | 実ブラウザでの既定エスケープ証跡（REQ-1 連動） | `xss_payload_item_does_not_produce_script_element_in_real_dom` |

観点 4 は 2 テスト関数に分かれる（4a: 事前付与状態の保持、4b: 再ハイドレーション後
のクリック状態保持・リスナー二重発火なし。PR #236 の Bugbot 指摘の回帰確認を兼ねる）。

## 4. 検証基盤と実行結果

### 4.1 検証基盤の構成（TASK-6.3a・#65）

| 項目 | 値 |
|------|-----|
| テストランナー | `wasm-pack test --headless --chrome`（`wasm-bindgen-test` + `run_in_browser`） |
| ランナー | `ubuntu-latest`（GitHub ホストランナー、Chrome/chromedriver プリインストール済み） |
| wasm-pack | v0.13.1（`.github/workflows/ci.yml` にバージョン固定 + SHA256 チェックサム検証付きで導入） |
| chromedriver | ランナー内蔵バイナリを `CHROMEDRIVER="${CHROMEWEBDRIVER}/chromedriver"` で明示指定（実行時の自動ダウンロードを回避） |
| 対象コマンド | `CHROMEDRIVER="${CHROMEWEBDRIVER}/chromedriver" wasm-pack test --headless --chrome wasm-client` |
| 存在ガード | `wasm-client/Cargo.toml` の有無で判定し、未作成の間はスキップ（現在は存在するため実行される） |

### 4.2 main での実行証跡（CI run）

- run ID: `29597370869`（`gh run view 29597370869 --json jobs` で取得）
- ワークフロー実行日時: 2026-07-17T16:44:53Z 開始・結論 `success`
- `browser-test` ジョブ（`Browser tests (wasm-pack --headless --chrome)`）:
  開始 2026-07-17T16:44:56Z・完了 2026-07-17T16:46:19Z・結論 `success`
- ジョブログ（`gh run view 29597370869 --log`）で、`wasm-client` 存在ガードが
  スキップではなく実行された（`Run browser tests (headless Chrome)` ステップが
  実際に走った）ことを確認。同ステップのログに以下の実行行が記録されている:

  ```text
  Running tests/hydration_browser.rs (.../deps/hydration_browser-3ad98e0b635ee12d.wasm)
  Running headless tests in Chrome on `http://127.0.0.1:34951/`
  running 6 tests
  test result: ok. 6 passed; 0 failed; 0 ignored; 0 filtered out; finished in 0.04s
  ```

  6 テスト全件（第 3 節の 5 観点に対応）が headless Chrome 上で pass している。
- ローカル再現手順は `docs/guides/browser-testing.md` 第 5 節を参照。

### 4.3 テスト実行の分担（重複回避）

`wasm-client/tests/hydration_browser.rs` の doc comment が明記するとおり、以下は
本レポートの対象外であり別テストが担保する:

| 対象 | 担当ファイル | 実行環境 |
|------|-------------|---------|
| SSR/SSG 間の文字列完全一致 | `server/tests/ssr_ssg_parity.rs` | native（実ブラウザ不要） |
| 環境実証スモークテスト（CI green 維持の最小限） | `wasm-client/tests/hydrate_smoke.rs`（TASK-6.2c・#49） | headless Chrome（同一 `browser-test` ジョブ内、`hydration_browser.rs` 実行前に 3 テストが pass 済み） |

## 5. テスト対応表

`wasm-client/tests/hydration_browser.rs` の 6 テスト関数と 5 観点の対応（第 3 節と
同一。再掲）。全関数が `#[wasm_bindgen_test]`（`wasm_bindgen_test_configure!(run_in_browser)`）
として headless Chrome 上で実行される。

| テスト関数 | 観点 |
|-----------|------|
| `mount_csr_reflects_same_render_output_as_ssr` | 観点 1 |
| `hydrate_does_not_rebuild_server_rendered_dom` | 観点 2 |
| `hydrate_toggles_liked_class_on_click_and_untoggles_on_second_click` | 観点 3 |
| `hydrate_preserves_pre_existing_liked_state` | 観点 4a |
| `re_hydrate_preserves_click_state_and_fires_exactly_once` | 観点 4b |
| `xss_payload_item_does_not_produce_script_element_in_real_dom` | 観点 5 |

## 6. スコープ外と残課題

- **状態注入フォーマット（`data-hydrate-*` エンコード）**: `hydration_browser.rs`
  の観点 4 は DOM 属性（`class`）の素朴な保持確認にとどまり、状態注入フォーマット
  自体の製品化は TASK-11.4 系（#81〜#84）のスコープ（`docs/api/hydration-api.md`
  第 5 節スコープ外表）。
- **性能実証**: 初期ロード・DOM 操作性能の判定は TASK-11.5 側（`docs/reports/perf-browser-report.md`）
  が担当し、本ドキュメントの対象外。
- **#66・#67 が open のままである点**: 成果物（`hydration_browser.rs`）は
  PR #241 で main に既にマージ済みであり（2026-07-17T10:43:06Z）、実証も本
  ドキュメント第 4 節の CI run（2026-07-17T16:44:53Z 開始、`hydration_browser.rs`
  の 6 テスト pass）で確認済み。issue のクローズ操作自体は各担当・親イシュー
  （#64、クローズ済み）側の運用判断であり、本レポートの判定はこれに依存しない。
- **計測環境の限定**: 本レポートは CI（GitHub ホストランナー・`ubuntu-latest`）
  上の実行証跡に基づく。ローカル環境（実機ブラウザ）での再実行は
  `docs/guides/browser-testing.md` 第 5 節の手順で可能だが、本レポート作成時点では
  ローカル実行のログは収集していない。CI 実行環境（Chrome/chromedriver
  プリインストール済みの実ブラウザ）は「実ブラウザでの正式実証」という
  REQ-6 受け入れ基準の要求を満たすと判断する。

## 7. セキュリティ考慮事項（OWASP Top 10 観点）

- **A03 インジェクション / XSS**: 観点 5（`xss_payload_item_does_not_produce_script_element_in_real_dom`）
  が、XSS ペイロードを含むタイトルを実 DOM 上に描画した際に `script` 要素が
  生成されず、ペイロードがテキストとして表示されることを実ブラウザで確認済み。
  DOM への HTML 挿入は `render_detail_page_html`（`rws_core::render` 出力、既定
  エスケープ済み）のみを経由しており、`format!` による HTML 文字列組み立てや
  `raw_html()` は使用していない（`docs/api/hydration-api.md` 第 6 節不変条件）。
  本レポートはこの既存対策を正として記載し、エスケープ迂回を推奨・容認する
  記述は行わない。
- **A05 セキュリティ設定ミス / A08 サプライチェーン**: `browser-test` ジョブは
  wasm-pack バージョン固定 + SHA256 チェックサム検証・ランナー内蔵 chromedriver
  の明示指定（実行時自動ダウンロード禁止）・第三者製 action 不使用という既存の
  対策を維持しており、本ドキュメントはこれを緩める手順を一切記載しない
  （`docs/guides/browser-testing.md` 第 8 節と整合）。
- **機微情報の露出**: 本レポートに記載する実行環境情報は run ID・実行日時・
  ジョブ名・テスト件数等の公開情報のみであり、トークン・内部 URL・秘匿情報は
  含まない。

## 8. 参照

- `wasm-client/tests/hydration_browser.rs`（TASK-6.3b・#66、実証テスト本体）
- `docs/guides/browser-testing.md`（TASK-6.3a・#65、検証環境・第 7 節引き継ぎ表）
- `docs/api/hydration-api.md`（第 3・5・6 節、ハイドレーション API 契約・不変条件）
- `docs/reports/perf-browser-report.md`（TASK-11.5c・#88、性能側の Conditional Go 条件 1 判定）
- `server/tests/ssr_ssg_parity.rs`（TASK-6.4、SSR/SSG 間の文字列完全一致・native）
- `wasm-client/tests/hydrate_smoke.rs`（TASK-6.2c・#49、環境実証スモークテスト）
- `docs/spec/04-requirements.md` REQ-6（受け入れ基準 4「Conditional Go 条件 1・宿題」）
- `docs/spec/05-tasks.md` TASK-6.3（親タスク受け入れ基準・4 分割サブタスク）
- `docs/spec/06-roadmap.md`（Conditional Go 条件 1・MS-3 完了ゲート、第 11・155 行）
- PR #241（`hydration_browser.rs` 導入・main へマージ、2026-07-17T10:43:06Z）
- Issue #64（親・クローズ済み）・#65（クローズ済み）・#66・#67（open、
  成果物は PR #241 でマージ済み）・#68（本イシュー）
- CI run 29597370869（`browser-test` ジョブ・結論 success・2026-07-17T16:44:53Z 開始）

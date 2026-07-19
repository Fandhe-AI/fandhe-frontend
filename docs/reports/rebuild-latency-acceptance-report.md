# 本番差分ビルド反映時間 受け入れ基準検証レポート（TASK-10.4b）

> **注記（#437）**: 本レポート中の `RWS_BIND_ADDR` / `RWS_WASM_BUILD` はその後 #437 で `FANDHE_FRONTEND_BIND_ADDR` / `FANDHE_FRONTEND_WASM_BUILD` へ改名されました。以下の記録は当時の実測値・環境変数名のまま残しています。

## 1. 目的とトレーサビリティ

- TASK-10.4「本番差分ビルド反映時間の受け入れ基準検証」（親イシュー #118、
  `docs/spec/05-tasks.md` 265〜270 行）は、PoC-4 実測（0.571〜0.597 秒、
  `docs/spec/03-poc/single-binary-distribution/README.md` 106〜109 行）を
  基準に、本番ビルドのアセット変更反映（差分ビルド）が **5 秒以内**である
  ことを CI ベンチマークとして継続計測するタスク。
- 2 分割サブタスクの内訳:
  - TASK-10.4a（#119・クローズ済み、PR #224 / commit 2389c9b）:
    `dist-server/benches/rebuild_latency.rs` のベンチ実装
    （`harness = false` / `test = false` の std-only ベンチ）と、判定ロジック
    `dist-server/src/bench_support.rs`（`LIMIT_SECONDS = 5.0`、ユニット
    テスト付き）・CI ジョブ `rebuild-latency`
    （`.github/workflows/ci.yml` の「REQ-10 rebuild latency (5s limit)」）
    による CI 統合
  - TASK-10.4b（本ドキュメント・#120・クローズ済み、PR #222 /
    commit 7b18df3）: 受け入れ基準の検証とレポート作成
- 親イシュー #118 は上記 2 サブタスクの成果物を踏まえ、本レポート第 2 節の
  判定を PENDING から確定（PASS）へ更新する最終検証を担う。
- 判定基準の中核は REQ-10 受け入れ基準（`docs/spec/04-requirements.md`
  138 行）の第 2 項「本番ビルドのアセット変更反映（差分ビルド）が 5 秒以内
  であること（PoC-4 実績: 0.571〜0.597 秒）」。同旨の文言は
  `docs/spec/06-roadmap.md` 96 行（TASK-10.4 のチェックボックス）・101 行
  「本番ビルドの差分反映が 5 秒以内であること」にも再掲されている。
  REQ-10 の他の受け入れ基準（開発時アセット変更の即時反映＝TASK-10.1・
  137 行、単一 `cargo build` での双方成果物生成＝TASK-10.2・139 行、Docker
  マルチステージビルド内 WASM 再ビルド＝TASK-10.3・140 行）は本レポートの
  スコープ外である。
- PoC-4（`docs/spec/03-poc/single-binary-distribution/README.md`）は本番
  ビルド（`cargo build -p rws-dist-server --release`、依存クレートキャッシュ
  利用の差分ビルド）を 2 回計測し、0.571 秒・0.597 秒を得て目標の 5 秒以内
  を達成したと報告している（同 106〜109 行、133 行、142 行）。同 109 行は
  「依存クレートを含む完全クリーンビルドは wasm ビルド込みで約 12 秒＋
  ネイティブビルド約 6.5 秒程度（初回のみ）であり、日常的なアセット変更の
  反復（差分ビルド）には影響しない」と明記しており、クリーンビルドは受け
  入れ基準（5 秒以内）の対象外である。

## 2. 判定ステータス: 検証済み（PASS） — #118 実装時点で確定

TASK-10.4a（#119）が `origin/main` にマージ済みで
`dist-server/benches/rebuild_latency.rs`（std-only ベンチ）・
`dist-server/src/bench_support.rs`（判定ロジック、
`LIMIT_SECONDS = 5.0`）・CI ジョブ `rebuild-latency`
（`.github/workflows/ci.yml`「REQ-10 rebuild latency (5s limit)」）が
存在することを確認した。第 3 節の判定基準 4 項目を突合し、いずれも Pass
であることを第 5 節の実測結果とともに確認したため、REQ-10 受け入れ基準
第 2 項の解消判定を **検証済み（PASS）** に確定する。

- 「差分ビルド反映時間」: CI 実測（run 29579583291）で
  `max_s=2.451`（5 秒閾値に対し約 2 倍のマージン）、ローカル再実測でも
  `max_s=0.653` と、いずれも 5 秒以内を満たす。
- 「反映の実効性」: `dist-server/benches/rebuild_latency.rs` のソースを
  読み、各サンプル反復で `static/rebuild-latency-probe.txt` へ一意な
  マーカーを書き込み→差分ビルド→`binary_contains_marker` で再ビルド後の
  バイナリ（`include_bytes!` 経由で埋め込みテーブルへ焼き込まれる）に
  当該マーカーが含まれるかを検査し、含まれない場合は fail-closed
  （終了コード 1）で終了する実装であることを確認済み。しきい値判定側
  （`bench_support::judge`）のユニットテストとは独立した確認である。
- 「CI 統合」: `rebuild-latency` ジョブが `dist-server/` を含む push ごとに
  自動実行されることを `.github/workflows/ci.yml` の定義で確認済み
  （単発の手動計測ではない）。
- 「計測環境差の扱い」: CI（ubuntu-latest）・ローカル環境の双方で 5 秒閾値
  に対し十分なマージン（それぞれ約 2 倍・約 7〜8 倍）を確認済み。

## 3. 判定基準（REQ-10 受け入れ基準第 2 項の機械検証項目化）

`docs/spec/04-requirements.md`（REQ-10・138 行）・`docs/spec/05-tasks.md`
TASK-10.4（265〜270 行）・PoC-4 実証手順（`docs/spec/03-poc/
single-binary-distribution/README.md` 103〜109 行）に基づく。

| 項目 | 判定基準 | 対応する検証手段 | 実測担当 |
|------|---------|-----------------|---------|
| 差分ビルド反映時間 | `static/` 配下のアセット変更後、`cargo build -p rws-dist-server --release`（依存クレートはビルド済みキャッシュを再利用する差分ビルド）が 5 秒以内に完了すること | `dist-server/benches/rebuild_latency.rs`（ビルド呼び出し＋所要時間計測） | TASK-10.4a（#119） |
| 反映の実効性 | 再ビルド後のバイナリが変更後のアセットを実際に配信すること（`rust-embed` 相当の自前埋め込みテーブルが再生成されていることの確認、`dist-server/build.rs` 参照） | TASK-10.4a（#119）のベンチ／付随テスト | TASK-10.4a（#119） |
| CI 統合 | 上記計測が単発の手動実行ではなく、CI 上で自動的に継続実行されること | CI ワークフロー（`.github/workflows/`）へのベンチジョブ組み込み | TASK-10.4a（#119） |
| 計測環境差の扱い | PoC-4 実測環境（実測当時のローカル開発環境）と CI ランナー環境の差異を踏まえ、5 秒という閾値に対して十分なマージンを確保すること（PoC-4 実測 0.571〜0.597 秒は 5 秒に対し約 8〜9 倍の余裕があり、CI ランナーが多少低速でも閾値割れのリスクは小さいと見込まれる） | ベンチ実行結果と閾値の比較、複数回実行時のばらつき確認 | TASK-10.4a（#119） |

- 「差分ビルド反映時間」「反映の実効性」の 2 項目が Pass した場合に、
  REQ-10 受け入れ基準第 2 項（`docs/spec/04-requirements.md` 138 行「本番
  ビルドのアセット変更反映（差分ビルド）が 5 秒以内であること」）を満たす
  と判定する。
- 「CI 統合」は継続的な回帰検知を保証するための項目であり、単発の手動計測
  のみでは本レポートの判定を「検証済み」に確定しない（先例
  `docs/reports/isolated-run-acceptance-report.md` 59〜60 行と同一の運用）。
- 「計測環境差の扱い」は判定確定の必須条件ではないが、CI 実行結果を記録
  する際に環境情報（OS・ランナー種別）を併記し、閾値に対するマージンを
  明示するための項目とする。

## 4. 検証手順の様式（TASK-10.4a 実装参照用）

PoC-4 実証手順（`docs/spec/03-poc/single-binary-distribution/README.md`
103〜109 行）を製品版向けに再定義する。`rebuild_latency.rs` はこの手順を
自動ベンチとして実装する。

1. **初回フルビルド**: `cargo build -p rws-dist-server --release` を実行し、
   依存クレートのビルドキャッシュを確立する（この 1 回目の所要時間は計測
   対象外）。
2. **アセット変更**: `static/` 配下の 1 ファイル（CSS または HTML 等）に
   計測用の軽微な変更を加える（内容の意味は問わないが、コンパイル時埋め
   込みテーブルの再生成が誘発される変更であること）。
3. **差分ビルドの所要時間計測**: `time cargo build -p rws-dist-server
   --release` 相当の処理を実行し、依存クレートキャッシュを再利用した状態
   での所要時間を計測する。5 秒以内であることを確認する。
4. **配信内容の反映確認**: 再ビルド後のバイナリを起動し、変更後のアセット
   が配信されること（埋め込みテーブルが再生成され、変更前の内容が残留し
   ていないこと）を確認する。
5. **後始末**: 手順 2 で加えた変更を元に戻す、またはテスト専用の一時的な
   変更に留め、`static/` の実体を汚染しない。
6. **CI 組み込み**: 上記 1〜5 を CI（`.github/workflows/` のジョブ）に組み
   込み、`dist-server/` への変更ごとに自動実行する。

## 5. 実測結果記録欄

### CI 実測（判定の正）

CI run 29579583291（`.github/workflows/ci.yml` ジョブ「REQ-10 rebuild
latency (5s limit)」、ubuntu-latest、commit `23255322`、
2026-07-17T12:15Z 開始）のサマリ行:

```
rebuild-latency: samples=3 s1=2.451 s2=2.430 s3=2.427 max_s=2.451 limit_s=5.0 result=PASS
```

| 項目 | 結果（複数回計測） | 実行コミット SHA | 実行環境 | 5 秒閾値との比較 | 判定 |
|------|-------------------|-----------------|---------|-----------------|------|
| 差分ビルド反映時間 | s1=2.451 s2=2.430 s3=2.427（max=2.451） | `23255322` | GitHub Actions ubuntu-latest（Ubuntu 24.04 イメージ） | 5.0 秒に対し約 2 倍のマージン | Pass |
| 反映の実効性 | ベンチ内バイナリマーカー検査が fail-closed で実施され、3 サンプルとも成功 | `23255322` | 同上 | — | Pass |
| CI 統合 | `rebuild-latency` ジョブが push ごとに自動実行（単発の手動実行ではない） | `23255322` | 同上 | — | Pass |

### ローカル参考値（追加確認）

本イシュー（#118）の最終検証時に、ワークツリー環境で
`cargo bench --locked -p rws-dist-server --bench rebuild_latency` を
CI と同一条件（`RWS_WASM_BUILD` 未設定、WASM ステージ込み）で実行し、
以下のサマリ行を得た（実行環境情報は OS の粒度に留める）。

```
rebuild-latency: samples=3 s1=0.617 s2=0.599 s3=0.653 max_s=0.653 limit_s=5.0 result=PASS
```

- 実行環境: Linux（ローカル開発ワークツリー、CPU アーキテクチャ・ホスト名は
  記載しない）
- 5 秒閾値に対し約 7〜8 倍のマージン。CI 実測（ubuntu-latest、約 2 倍の
  マージン）と方向性が一致し、判定を補強する参考値と位置付ける。
- **判定の正は CI 実測（上表）とする**。ローカル値は環境依存のばらつきを
  含むため参考情報に留める。

## 6. 確定運用（実施済み）

1. TASK-10.4a（#119、PR #224 / commit 2389c9b）がマージされ
   `dist-server/benches/rebuild_latency.rs` が `origin/main` に存在する
   ことを確認済み。
2. CI 実測（run 29579583291）およびローカル参考実測を第 5 節の結果記録欄
   に転記済み（コミット SHA・実行環境・所要時間・Pass/Fail）。
3. CI 上でベンチが自動実行されるジョブ（`rebuild-latency`、
   `.github/workflows/ci.yml`）が push ごとに自動実行されることを確認済み
   （単発の手動計測のみではない）。
4. 第 3 節の判定基準のうち「差分ビルド反映時間」「反映の実効性」「CI
   統合」の 3 項目すべてが Pass のため、本レポート第 2 節を「検証済み
   （PASS）」に確定した（REQ-10 受け入れ基準第 2 項を満たすと判定）。
5. 親イシュー #118 の受け入れ条件チェックボックスは本 PR のマージ後に
   更新する。`docs/spec/06-roadmap.md` 96 行のチェックボックスは
   `docs/spec/` サブモジュールであり本リポジトリでは編集禁止のため、
   更新提案は frontend-framework-spec リポジトリへの起票として別途扱う
   （本レポート自体のスコープ外、`out-of-scope-tracking.md`）。

## 7. セキュリティ・不変条件の確認

- 本レポートはドキュメントのみの変更であり、コード（`.rs`）・CI
  （`.github/`）・依存構成（`Cargo.toml`）を一切変更しない。
- 既定エスケープ（REQ-1）・`#![forbid(unsafe_code)]`（REQ-2、`core` /
  `interactive`）のいずれにも影響しない。
- 依存グラフ上限（REQ-3、60 件/深さ 6）に対し、`cargo run -p xtask --
  check-deps --package rws-dist-server` による再計測でも
  `packages=21/60 depth=5/6 result=PASS` を確認し、本レポート確定時点でも
  不変であることを確認した。`dist-server/Cargo.toml` の `[dev-dependencies]`
  は空のままであり、TASK-10.4a のベンチ実装は依存追加を伴わない。

## 8. 参照

- `docs/spec/04-requirements.md` REQ-10（138 行、受け入れ基準第 2 項）
- `docs/spec/05-tasks.md` TASK-10.4（265〜270 行）
- `docs/spec/06-roadmap.md` 96 行（TASK-10.4 チェックボックス）・101 行
  （REQ-10 受け入れ基準の再掲）
- `docs/spec/03-poc/single-binary-distribution/README.md`（PoC-4 実測、
  103〜109 行・133 行・142 行）
- `docs/reports/isolated-run-acceptance-report.md`（判定基盤整備＋保留方式の先例、
  TASK-9.2b・PR #211）
- `docs/reports/wasm-build-integration-report.md`（同様の保留方式の先例、
  TASK-10.2e・PR #219）
- `dist-server/Cargo.toml`（REQ-3 実測コメント、21 件/深さ 5）
- Issue #118（親・TASK-10.4）・#119（TASK-10.4a・
  `rebuild_latency.rs` 実装、本レポートの前提タスク）・#120（本レポート・
  TASK-10.4b）

## 9. 追記（イシュー #294）: 判定プロトコルを max-of-N → median-of-N へ変更

- 上記第 1〜8 節は TASK-10.4a/b 確定時点（max-of-N 判定）の記録として
  そのまま保持する。以下は #294 対応（共有 self-hosted runner の CPU 競合に
  よる間欠フレーク対策）による判定プロトコルの変更点の追記であり、既存の
  実測ログ・判定根拠を書き換えるものではない。
- 背景: PR #291 にて、N=3 サンプルの **最大値** を `LIMIT_SECONDS = 5.0`
  と比較する従来判定が、共有 runner（6 並列）上の他ジョブとの CPU 競合で
  1 サンプルのみ跳ねたこと（5.494 秒、rerun では 2.x〜4.x 秒台に収束）に
  より間欠的に FAIL した。
- 検討した対応案とその却下理由: 当初 `judge()` を最小値（min-of-N）基準に
  変更する案を実装したが、レビュー指摘により却下した。min-of-N は
  「N=3 中 1 サンプルでもしきい値以内なら PASS」となるため、残り 2
  サンプルが恒常的に超過していても検出できない（例: `[6.0, 6.0, 4.9]` は
  過半数が超過しているが min=4.9 で PASS してしまう）。これは受け入れ基準
  「リグレッション検出力の維持」を満たさない、過度に寛容な統計量だった。
- 対応: `dist-server/src/bench_support.rs` の `judge()` を **中央値
  （median-of-N）** 基準に変更した（本レポート第 2〜3 節が根拠とする PoC-4
  実測 0.571〜0.597 秒・`LIMIT_SECONDS = 5.0` 自体は不変）。median-of-N は
  少数派（N=3 なら 1 サンプル）のみの環境ノイズは吸収しつつ、過半数の
  サンプルが超過する恒常的・間欠的リグレッションを引き続き検出できる。
  正直なトレードオフとして、**少数派**（N=3 なら 1 サンプルのみ）が間欠的に
  遅くなる製品リグレッションは median-of-N でも検出できない（min-of-N ほど
  寛容ではないが、ゼロではない）。サマリ書式には `median_s` を追加し、
  `min_s`・`max_s` は観測性のためそのまま残す。
- 変更しないもの: `LIMIT_SECONDS = 5.0`・`SAMPLE_COUNT = 3`・
  `runs-on: self-hosted`・緩和用 input / 環境変数 / continue-on-error を
  設けない運用原則（第 7 節のセキュリティ・不変条件は本追記後も維持）。
- 詳細な設計根拠は `dist-server/src/bench_support.rs` のモジュール
  ドキュメント、CI 契約の変更点は `.github/workflows/ci.yml` の
  `rebuild-latency` ジョブのコメントを参照（イシュー #294）。

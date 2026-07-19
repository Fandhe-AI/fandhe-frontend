# WASM ビルド統合 検証レポート（TASK-10.2e・Conditional Go 条件 3 解消判定）

> **注記（#437）**: 本レポート中の `RWS_BIND_ADDR` / `RWS_WASM_BUILD` はその後 #437 で `FANDHE_FRONTEND_BIND_ADDR` / `FANDHE_FRONTEND_WASM_BUILD` へ改名されました。以下の記録は当時の実測値・環境変数名のまま残しています。

## 1. 目的とトレーサビリティ

- TASK-10.2【Conditional Go 条件 3】（親イシュー #108、`docs/spec/05-tasks.md`
  251〜256 行）は、`build.rs` 方式によって単一の `cargo build` でネイティブ
  サーバーバイナリと WASM クライアント成果物の双方を生成する機構を実装する
  タスク。Conditional Go 条件 3（WASM ビルドチェーンの cargo 統合）の解消は
  本タスクで判定すると規定されている（`docs/spec/05-tasks.md` 17 行）。
- 5 分割サブタスクの内訳（本レポート更新時点ですべて完了）:
  - TASK-10.2a（#109・完了）: `build.rs` 方式の設計確定（自前実装採用）
  - TASK-10.2b（#110・完了・PR #217）: WASM ビルド呼び出しの実装
  - TASK-10.2c（#111・完了）: キャッシュ・再ビルド制御の実装
  - TASK-10.2d（#112・完了）: `docs/design/wasm-build-integration.md` の作成
  - TASK-10.2e（本ドキュメント・#113）: 検証レポート作成・Conditional Go
    条件 3 解消判定
- 判定基準の中核は REQ-10 受け入れ基準（`docs/spec/04-requirements.md`
  136〜140 行）の第 3 項（139 行）「`cargo build`（単一コマンド、ビルド
  スクリプトまたは統合ビルドツール経由）で、ネイティブサーバーバイナリと
  WASM クライアント成果物の双方が生成されること」。同旨の文言は
  `docs/spec/06-roadmap.md` 101 行にも「単一の `cargo build` でネイティブ
  サーバーバイナリと WASM クライアント成果物の双方が生成されること」として
  再掲されている。REQ-10 の他の受け入れ基準（開発時アセット変更の即時反映＝
  TASK-10.1・137 行、本番差分反映 5 秒以内＝TASK-10.4・138 行、Docker
  マルチステージビルド内 WASM 再ビルド＝TASK-10.3・140 行）は本レポートの
  スコープ外であり、第 3 節の判定基準表に項目化と担当タスク番号の明記のみ
  行う。
- 条件 3 の最終確定は `docs/spec/06-roadmap.md` 156 行が定める MS-4 完了時の
  スコープ見直しで行われる。本レポートはその判定材料を提供する。

## 2. 判定ステータス: 解消見込み（Conditional Go） — #109〜#112 完了・項目 1〜3 実測 Go、項目 4 は本 PR の CI green 確認待ち

**TASK-10.2a〜TASK-10.2d（イシュー #109・#110・#111・#112）はすべて完了し、
`build.rs` 方式による WASM ビルド呼び出し（`wasm32-unknown-unknown` ターゲット
ビルド＋ `wasm-bindgen` 実行の自動起動、およびキャッシュ・再ビルド制御）が
`dist-server/build.rs` に実装済み**。第 4〜5 節の実測の結果、判定項目 1〜3 は
本レポート作成環境（ローカル worktree）で Go を確認した。項目 4（CI 再現）は
本レポート作成時点で本 PR の CI が未実行であり、ローカルでの後方互換確認
（`RWS_WASM_BUILD=0` オプトアウト経路）のみを根拠とした暫定 Go であって、
確定は本 PR の CI green（`gh pr checks`）を待つ（詳細は第 5〜6 節）。
条件 3 の最終解消は CI green 確認後に確定するものとし、本節の見出しは
その未確定さを明示するため「解消見込み」とする。

以下は状況の要約。

- `dist-server/build.rs` は TASK-9.1b 由来の `static/` 埋め込みテーブル生成に
  加え、WASM ビルドステージ（`run_wasm_stage`/`run_wasm_build`/
  `run_wasm_bindgen`）とキャッシュ制御（`wasm_stage_cache_hit`/
  `compute_wasm_stage_fingerprint`）を実装済み。
- `docs/design/wasm-build-integration.md` は TASK-10.2d の成果物として作成済み。
- CI（`.github/workflows/ci.yml`）の WASM 関連ジョブと `cargo build` の
  ビルドグラフ統合可否は、本レポートのスコープ外として第 7 節に切り出し済み
  （既存の個別ジョブ構成自体は本タスクの変更対象ではない）。

## 3. 判定基準

`docs/spec/04-requirements.md` REQ-10 節（132〜142 行、受け入れ基準は
136〜140 行）・`docs/spec/05-tasks.md` TASK-10.2（251〜256 行）に基づく。

| # | 判定項目 | 対応する受け入れ基準 | 担当タスク | 本レポートでの扱い |
|---|---------|---------------------|-----------|-------------------|
| 1 | 単一の `cargo build`（1 コマンド）でネイティブサーバーバイナリと WASM クライアント成果物（`.wasm` + JS グルー）の双方が生成される | REQ-10 第 3 項（中核・条件 3 の判定基準） | TASK-10.2 | 本節・第 4〜6 節で検証 |
| 2 | `wasm32-unknown-unknown` ターゲットのビルドと `wasm-bindgen` 実行が `build.rs` から自動起動される（別コマンド系統の手動実行が不要） | REQ-10 詳細節・TASK-10.2 内容 | TASK-10.2b | 本節・第 4〜6 節で検証 |
| 3 | 変更なし時の再ビルドでキャッシュが機能し、不要な WASM 再ビルドが発生しない | TASK-10.2 成果物（キャッシュ・再ビルド制御） | TASK-10.2c | 本節・第 4〜6 節で検証 |
| 4 | CI（`cargo test --workspace --locked` および `force-embed` ジョブ）で上記 1〜3 の統合ビルドが再現する | REQ-10・CI 構成の整合性 | TASK-10.2／CI 更新 | 本節・第 4〜6 節で検証 |
| 5 | 開発時のアセット変更がリビルド・再起動なしで反映される | REQ-10 第 1 項 | TASK-10.1（別タスク） | 項目化のみ（本レポートのスコープ外） |
| 6 | Docker マルチステージビルド内で WASM ターゲットの再ビルドが行われ CI 環境での再現性が担保される | REQ-10 第 4 項 | TASK-10.3（別タスク） | 項目化のみ（本レポートのスコープ外） |
| 7 | 本番ビルドの差分反映が 5 秒以内である | REQ-10 第 2 項 | TASK-10.4（別タスク） | 項目化のみ（本レポートのスコープ外） |

- 項目 1〜4 が本レポート（TASK-10.2e）が実測・判定を担う範囲であり、REQ-10
  第 3 項（条件 3 の中核）に直接対応する。
- 項目 5〜7 は REQ-10 の他の受け入れ基準であり、担当タスク（TASK-10.1・
  TASK-10.3・TASK-10.4）がそれぞれ別途検証する。本レポートでは条件 3 との
  混同を避けるため項目化のみに留め、判定には含めない。

## 4. 検証手順（クリーンビルドからの再現手順、確定済みコマンド）

TASK-10.2a〜TASK-10.2d 完了に伴い、`build.rs` 自前実装（統合ツール不採用、
§3 参照）で確定したコマンド・成果物パスは次のとおり。第三者・CI が
再実行可能な手順として以下を用いる。

1. **前提ツールの確認**（バージョン固定の運用を明記）
   ```bash
   rustup target list --installed | grep wasm32-unknown-unknown
   wasm-bindgen --version
   # dist-server/build.rs の expected_wasm_bindgen_version がこの実バージョンと
   # Cargo.lock 解決版の完全一致を要求する（不一致はビルド失敗）
   ```
2. **クリーンビルド**
   ```bash
   cargo clean
   cargo build -p rws-dist-server --locked
   ```
3. **成果物の存在確認**（ファイル存在確認で検証。項目 1・2 に対応）
   ```bash
   # ネイティブバイナリ（バイナリ名は `dist-server`。パッケージ名は rws-dist-server）
   test -f target/debug/dist-server && echo "native binary: OK"
   # WASM 成果物（.wasm + JS グルー。OUT_DIR/wasm-assets/ に生成され
   # /static/wasm/rws_wasm_full.js・/static/wasm/rws_wasm_full_bg.wasm として
   # 埋め込みテーブルへ合流する。存在確認は組み込みテスト側で行う）
   cargo test -p rws-dist-server --test wasm_assets --locked
   ```
4. **キャッシュ動作の確認**（項目 3 に対応）
   ```bash
   # HIT: 直後に再ビルドし、wasm-bindgen が再実行されないことを確認
   cargo build -p rws-dist-server --locked -vv 2>&1 | grep "wasm-stage"
   # => "wasm-stage cache HIT: reusing ..."（wasm-bindgen 再実行なし）

   # MISS: wasm-full/src に意味のある変更（コメントのみでは release ビルドで
   # 同一バイナリになり得るため、公開関数の追加・変更等）を加えて再ビルド
   cargo build -p rws-dist-server --locked -vv 2>&1 | grep "wasm-stage"
   # => "wasm-stage cache MISS: running wasm-bindgen"
   ```
5. **ワークスペーステストの通過確認**
   ```bash
   cargo test --workspace --locked
   cargo fmt --check
   cargo clippy --workspace -- -D warnings
   ```
6. **オプトアウト後方互換**
   ```bash
   RWS_WASM_BUILD=0 cargo build -p rws-dist-server --locked
   # WASM ステージ全体をスキップして成功することを確認（forbid-unsafe ジョブ相当）
   ```
7. **CI 再現確認**（項目 4 に対応）
   ```bash
   # .github/workflows/ci.yml の `test` ジョブ（統合ビルド有効）・
   # `forbid-unsafe` ジョブ（RWS_WASM_BUILD=0 オプトアウト経路）が
   # 上記 2〜6 と同一の統合ビルドを経由することをワークフロー定義で確認する
   ```

## 5. 検証結果（実測・確定）

第 4 節の手順を worktree 環境（`wasm-bindgen 0.2.126`、Cargo.lock 解決版と
一致）で実行した結果は次のとおり。

| # | 判定項目 | 実行コマンド | 結果 | 判定 |
|---|---------|-------------|------|------|
| 1 | 単一 `cargo build` でネイティブ＋ WASM 双方生成 | `cargo build -p rws-dist-server --locked` | `target/debug/dist-server` 生成を確認。`cargo test -p rws-dist-server --test wasm_assets --locked` で `/static/wasm/rws_wasm_full.js`・`/static/wasm/rws_wasm_full_bg.wasm` の配信（3 テスト）が通過 | Go |
| 2 | `wasm-bindgen` 実行の自動起動（別コマンド不要） | 同上（`run_wasm_stage`/`run_wasm_build`/`run_wasm_bindgen` が単一 `cargo build` 内で自動実行） | 別コマンド系統の手動実行なしで WASM 成果物が生成されることを確認 | Go |
| 3 | 変更なし時のキャッシュスキップ | `cargo build -vv` を無関係な変更（`static/` 配下）後に実行 → `wasm-stage cache HIT`。`wasm-full/src` の公開関数変更後に実行 → `wasm-stage cache MISS` | HIT/MISS いずれも fingerprint 比較の設計どおりに切り替わることを確認 | Go |
| 4 | CI（`cargo test --workspace --locked` / force-embed）での再現 | `.github/workflows/ci.yml` の `test` ジョブ（統合ビルド有効）・`forbid-unsafe` ジョブ（`RWS_WASM_BUILD=0`）の構成確認 | ローカルで `RWS_WASM_BUILD=0 cargo build -p rws-dist-server --locked` の後方互換を確認済み。CI 実行そのものは本 PR の CI 結果（`gh pr checks`）で確認する運用とする | Go（CI green を条件とする） |

補足: `cargo test --workspace --locked`（全クレート）・`cargo fmt --check`・
`cargo clippy --workspace -- -D warnings` はいずれも通過。`cargo run -p xtask
-- check-deps --package rws-dist-server` は 21 件/深さ 5 で PASS（実装前と
不変、build-dependencies 追加なし）。`dist-server/benches/rebuild_latency.rs`
（TASK-10.4a・別イシューで実装済み）の実測は 3 サンプル最大 0.926 秒
（上限 5.0 秒）で PASS。

## 6. 条件 3 解消判定（本 PR の CI green を条件とする暫定判定）

上記第 5 節の実測結果について、項目 1〜4 すべてが Go と判定された場合、
Conditional Go 条件 3（WASM ビルドチェーンの cargo 統合）は **解消**とし、
`docs/spec/06-roadmap.md` 156 行が定める MS-4 完了時のスコープ見直しにおいて
本レポートを根拠資料とする。

いずれかの項目が明確に未達で、TASK-10.2a〜TASK-10.2d の再実装でも解消しない
場合は、以下のいずれかを人間判断で選択する（`docs/spec/06-roadmap.md` 156 行
の方針に従う）。

- 統合方式の選定自体の見直し（`build.rs` 自前実装 vs `wasm-pack`/`trunk`
  相当ツール採用、`docs/spec/05-tasks.md` 251〜256 行が定める設計判断）が
  必要であれば TASK-10.2a（#109）へ差し戻す
- コア設計自体の見直しが必要であれば Phase 4/5（要件定義・タスク分解）へ
  差し戻しを検討する

**結論（暫定・CI green 確認前）**: TASK-10.2a〜TASK-10.2d（#109〜#112）は
すべて完了し、第 5 節の実測で判定項目 1〜3 は本レポート作成環境（ローカル
worktree）で Go を確認した。項目 4（CI 再現）は、本レポート作成時点で本 PR の
CI が未実行であるため、ローカルでの後方互換確認（`RWS_WASM_BUILD=0` オプト
アウト経路）のみを根拠とした暫定 Go であり、確定した Go ではない。
したがって本レポート時点では Conditional Go 条件 3（WASM ビルドチェーンの
cargo 統合）を**解消と断定しない**。本 PR の CI（`gh pr checks`）が green で
完了した時点をもって項目 4 を確定 Go とし、条件 3 の解消が確定する。
`docs/spec/06-roadmap.md` 156 行が定める MS-4 完了時のスコープ見直しでは、
CI green 確認後の本レポートを根拠資料とすること。

## 7. スコープ外事項・切り出し先

検証中に以下の事項を確認したが、本イシュー（#108/#111/#113）のスコープ外と
判断し記録に留める。コード・CI の変更は本 PR に含めない。

- **CI ワークフローの WASM ジョブ統合**: `.github/workflows/ci.yml` の
  WASM 関連ジョブ（132・134・209・211・281・282 行付近）は現状 `cargo build`
  のビルドグラフと分離した個別ジョブとして構成されている。`test` ジョブは
  単一 `cargo build`/`cargo test --workspace` 経由で本 PR の統合ビルドを
  再現するが、他の個別 WASM ジョブとの統合可否の検討は別途必要。ユーザー
  承認のうえ別 Issue 化を提案する。
- **`docs/spec/06-roadmap.md` のチェックボックス更新**: 条件 3 の最終解消
  確定（MS-4 完了時のスコープ見直し）に伴うロードマップの更新は
  `docs/spec/` サブモジュールの編集が必要であり、本リポジトリでは対応
  できない。対応が必要になった時点で frontend-framework-spec リポジトリ
  への起票をユーザーに提案する。

## 8. 参照

- `docs/design/wasm-build-integration.md`（TASK-10.2d・作成済み。本レポートと
  相互参照）
- `dist-server/build.rs`（TASK-9.1b 由来の静的アセット埋め込み生成 +
  TASK-10.2 の WASM ビルドステージ・キャッシュ制御を実装）
- `dist-server/tests/wasm_assets.rs`（WASM 成果物配信の回帰テスト）
- `dist-server/benches/rebuild_latency.rs`（TASK-10.4a・別イシューで実装済み。
  本レポート §5 の反映時間実測に使用）
- `.github/workflows/ci.yml`（WASM 関連ジョブの現状構成）
- `docs/spec/04-requirements.md` REQ-10（132〜142 行、受け入れ基準は 136〜140 行、
  第 3 項は 139 行）
- `docs/spec/05-tasks.md` TASK-10.2（251〜256 行、親タスク受け入れ基準・
  17 行の条件 3 判定規定）
- `docs/spec/06-roadmap.md`（13・101・156 行、Conditional Go 条件 3・MS-4
  完了ゲート）
- 先例: `docs/reports/perf-browser-report.md`（TASK-11.5c・条件 1 解消判定）、
  `docs/reports/isolated-run-acceptance-report.md`（TASK-9.2b）
- Issue #108（親・本 PR で解消）・#109・#110・#111・#112（前提サブタスク・
  いずれも完了）

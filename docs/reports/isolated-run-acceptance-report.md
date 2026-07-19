# 外部ファイル非依存起動の受け入れ基準検証レポート（TASK-9.2b）

> **注記（#433 改名）**: 本レポートは旧名称時代の実測記録です。crate 名 `rws-*`（`rws-dist-server` 等）は #441 で `fandhe-frontend-*` へ、クレート配置はルート直下から #442 で `crates/` 配下へ、環境変数 `RWS_BIND_ADDR` / `RWS_WASM_BUILD` は #437 で `FANDHE_FRONTEND_BIND_ADDR` / `FANDHE_FRONTEND_WASM_BUILD` へ改名され、リポジトリ名 `Fandhe-AI/frontend-framework` は #439 で `Fandhe-AI/fandhe-frontend` へ改名済みです（新旧対応は `docs/design/framework-naming.md` 参照）。以下の記録中のコマンド・パス・URL・値は当時のまま残しています。

## 1. 目的とトレーサビリティ

- TASK-9.2「外部ファイル非依存起動の受け入れ基準検証」（親イシュー #98、REQ-9）は、
  PoC-4（`docs/spec/03-poc/single-binary-distribution/README.md`）で実施した
  「ビルド成果物をソースツリーと無関係なディレクトリへコピーして起動する」実証手順を
  CI 統合テストとして自動化するタスク（`docs/spec/05-tasks.md` TASK-9.2）。
- 2 分割サブタスクの内訳:
  - TASK-9.2a（#99）: `dist-server/tests/isolated_run.rs` のテスト実装
  - TASK-9.2b（本ドキュメント・#100）: 受け入れ基準の検証とレポート作成
- 本ドキュメントは TASK-9.2b の成果物（検証レポート）であり、REQ-9 の受け入れ基準
  （`docs/spec/04-requirements.md` 188〜200 行）を判定基準として様式化し、
  TASK-9.2a（#99）が実装するテストの実行結果を転記・分析して判定を確定する運用を
  定義する。

## 2. 判定ステータス: 検証済み（PASS）

TASK-9.1（`dist-server/` 製品版整備）・TASK-9.2a（#99、`dist-server/tests/
isolated_run.rs` 実装）はいずれもマージ済みで、`dist-server/` クレートが
`origin/main` に存在する。第 5 節の実測結果のとおり、第 3 節の判定基準
（「バインドアドレス切り替え」を除く 5 項目）はすべて Pass しており、
REQ-9 の受け入れ基準（`docs/spec/04-requirements.md` 193 行「単一バイナリが
外部ファイル・Node ランタイム・依存インストールなしで自己完結して動作する
こと」）を満たすと判定する。

- `RWS_BIND_ADDR` によるバインドアドレス切り替えの実測検証は別イシュー #162
  「RWS_BIND_ADDR によるバインドアドレス切り替えの検証」の担当範囲であり、
  本レポートでは判定基準表への項目化のみを行い、実測は #162 の成果に委ねる
  （本レポートの判定確定条件には含めない）。
- 静的アセット配信の実測は、リポジトリの `static/` に実在するアセット
  （`.js`・埋め込み WASM 成果物）に対して行った。第 3 節「静的アセット配信」
  の判定基準文言にある CSS・HTML は、本リポジトリの `static/` に実体を持たない
  （実在するのは `view-transitions.js` のみ、WASM 成果物は
  `dist-server/build.rs` が `OUT_DIR` へ別途埋め込む）ため対象外とした。

## 3. 判定基準（REQ-9 受け入れ基準の機械検証項目化）

`docs/spec/04-requirements.md`（REQ-9・188〜200 行）・`docs/spec/05-tasks.md`
TASK-9.2・PoC-4 実証手順（`docs/spec/03-poc/single-binary-distribution/README.md`
「実施内容 2」）に基づく。

| 項目 | 判定基準 | 対応する検証手段 | 実測担当 |
|------|---------|-----------------|---------|
| 隔離起動 | `static/`・`Cargo.toml`・ソースコードが一切存在しない隔離ディレクトリへバイナリ単体をコピーし、外部ファイル・Node ランタイム・依存インストールなしで起動できること | `isolated_run.rs`（プロセス起動＋終了コード確認） | TASK-9.2a（#99） |
| SSR 応答 | `GET /`（一覧ページ）が `200` で SSR HTML を返すこと | `isolated_run.rs`（HTTP レスポンス検証） | TASK-9.2a（#99） |
| XSS エスケープ維持 | `GET /items/{id}`（XSS ペイロード入りアイテム）が `200` を返し、`<script>` 等が既定エスケープ済みで出力されること（REQ-1 不変条件） | `isolated_run.rs`（レスポンスボディの文字列検証） | TASK-9.2a（#99）、TASK-9.4（#104・単一バイナリでの XSS 維持検証） |
| 静的アセット配信 | `GET /static/*`（実在するアセットの拡張子。本リポジトリでは JS・WASM。CSS・HTML は `static/` に実体なし — 第 2 節注記参照）が `200` で埋め込みアセットを返すこと | `isolated_run.rs`（複数エンドポイントの HTTP レスポンス検証） | TASK-9.2a（#99） |
| バインドアドレス切り替え | `RWS_BIND_ADDR` 環境変数でバインドアドレスを切り替えられること（既定 `127.0.0.1:3100` 相当） | 専用の起動テスト | #162（本レポートのスコープ外、項目化のみ） |
| CI 統合 | 上記検証が `cargo test -p rws-dist-server --test isolated_run --features force-embed --locked` として CI 上で自動実行されること | CI ワークフロー（`.github/workflows/ci.yml` の `dist-server-embedded-mode` ジョブ） | TASK-9.2a（#99） |

- 「隔離起動」「SSR 応答」「XSS エスケープ維持」「静的アセット配信」「CI 統合」の
  5 項目すべてが Pass した場合に、REQ-9 の受け入れ基準
  （`docs/spec/04-requirements.md` 193 行「単一バイナリが外部ファイル・Node
  ランタイム・依存インストールなしで自己完結して動作すること」）を満たすと判定する。
- 「バインドアドレス切り替え」は REQ-9 の受け入れ基準（200 行）に含まれるが、
  実測は #162 の担当範囲とし、本レポートの判定確定条件には含めない（該当項目は
  #162 側のレポートまたはイシューコメントで別途判定する）。
- 「CI 統合」は自動回帰を継続的に保証するための項目であり、単発の手動検証のみでは
  本レポートの判定を「検証済み」に確定しない（本レポートは `.github/workflows/
  ci.yml` へのステップ追加をもって Pass と判定する。第 5 節参照）。

## 4. 検証手順の様式（TASK-9.2a 実装参照用）

PoC-4 実証手順（`docs/spec/03-poc/single-binary-distribution/README.md`
「実施内容 2」）を製品版向けに再定義する。`isolated_run.rs` はこの手順を
自動テストとして実装する。

1. **ビルド**: `cargo build -p rws-dist-server --release` でネイティブの単一実行
   ファイルをビルドする。
2. **隔離ディレクトリの作成**: OS 標準の一時領域（`std::env::temp_dir()` 相当）配下に
   ソースツリーと無関係な一時ディレクトリを作成する。このディレクトリには
   `static/`・`Cargo.toml`・ソースコードを一切配置しない。
3. **バイナリのコピー**: ビルド済みバイナリ 1 つのみを隔離ディレクトリへコピーする。
4. **起動**: 隔離ディレクトリ内でバイナリを起動する。バインドアドレスは
   `RWS_BIND_ADDR` 未設定時の既定値（loopback、例 `127.0.0.1:<ポート>`）を用い、
   テスト実行中に外部公開しない。ポート番号はテスト間の衝突を避けるため、
   固定値ではなく OS 割当（ポート `0` 指定）または重複回避済みの値を用いる。
5. **エンドポイント検証**: 起動したプロセスに対し、以下を HTTP クライアントで検証する。
   - `GET /` → `200`、SSR HTML であること
   - `GET /items/{id}`（XSS ペイロード入りアイテム）→ `200`、ペイロードが
     エスケープ済みで出力されること（`raw_html()` 経由でない出力であることの確認）
   - `GET /static/*`（実在するアセットの拡張子。本リポジトリでは JS・WASM）→ `200`
6. **後始末**: プロセスを終了させ、隔離ディレクトリを削除する（テスト終了後に
   一時領域を残置しない）。
7. **CI 組み込み**: 上記 1〜6 を `cargo test -p rws-dist-server --test isolated_run
   --features force-embed --locked` として CI（`.github/workflows/ci.yml` の
   `dist-server-embedded-mode` ジョブ）に組み込み、`dist-server/` への変更ごとに
   自動実行する（`--features force-embed` が必要な理由: debug ビルドの既定
   `AssetMode::DevFilesystem` は `CARGO_MANIFEST_DIR` 由来の絶対パスから
   `static/` を読むため、隔離ディレクトリへコピーしたバイナリでも外部ファイル
   依存が生じてしまい検証にならない。`isolated_run.rs` 自体もこの理由で
   `#![cfg(any(not(debug_assertions), feature = "force-embed"))]` によって
   ファイル全体をコンパイル時ゲートしている）。

## 5. 結果記録欄

TASK-9.2a（#99、`dist-server/tests/isolated_run.rs`）の実装完了後、以下の実測結果を
記録する。

| 項目 | 結果 | 実行コミット SHA | 実行環境 | 判定 |
|------|------|-----------------|---------|------|
| 隔離起動 | `isolated_binary_boots_without_source_tree` ほか 6 テスト、`cargo test -p rws-dist-server --test isolated_run --features force-embed --locked` で PASS | 8309094895 | Linux x86_64、rustc 1.96.0 | Pass |
| SSR 応答 | `isolated_get_root_returns_ssr_html_with_escaped_payload` PASS（`GET /` → `200`、SSR HTML） | 8309094895 | Linux x86_64、rustc 1.96.0 | Pass |
| XSS エスケープ維持 | 同テストで `&lt;script&gt;` を含み生 `<script>` を含まないことを確認、`isolated_get_item_detail_keeps_default_escaping`（`GET /items/2` → `200`）も PASS | 8309094895 | Linux x86_64、rustc 1.96.0 | Pass |
| 静的アセット配信 | `isolated_static_assets_served_from_embedded_table`（`GET /static/view-transitions.js` → `200`）PASS。`--release`（WASM ビルドステージ有効時）では `isolated_wasm_assets_served` も PASS（`\0asm` マジックナンバー確認） | 8309094895 | Linux x86_64、rustc 1.96.0（`--release`、WASM ビルド有効） | Pass |
| CI 統合 | `.github/workflows/ci.yml` の `dist-server-embedded-mode` ジョブへ独立ステップ「Isolated run acceptance test (REQ-9)」として追加済み | 65c04a3 | GitHub Actions（`ubuntu-latest`） | Pass |

- `isolated_path_traversal_still_returns_404`（`GET /static/../Cargo.toml` → `404`）・
  `isolated_dir_stays_clean`（隔離ディレクトリにコピー済みバイナリ以外のファイルが
  生成されないこと）も PASS しており、上記 5 項目の裏付けとして併せて確認した。
- 「実行コミット SHA」は本ブランチ（`test/98-isolated-run`）上での該当実装コミットの
  短縮ハッシュを記録する。マージ後のコミットハッシュに置き換わる可能性がある点に
  留意する。
- 検証は debug（`--features force-embed`）・release（`--release`、WASM ビルド有効）
  の双方で実施し、いずれも安定して Pass することを複数回の再実行で確認した。

## 6. 確定運用（実施済み）

1. TASK-9.1（#94、子 #95〜#97）がマージされ `dist-server/` クレートが
   `origin/main` に存在することを確認済み。
2. TASK-9.2a（#99）で `dist-server/tests/isolated_run.rs` を実装済み。
3. `cargo test -p rws-dist-server --test isolated_run --features force-embed
   --locked`（debug）・`cargo test -p rws-dist-server --test isolated_run
   --release --locked`（release）を実行し、第 5 節の結果記録欄に実測結果を
   転記済み。
4. 第 3 節の判定基準（「バインドアドレス切り替え」を除く 5 項目）がすべて Pass
   したため、本レポートの第 2 節を「検証済み（PASS）」に更新し、REQ-9 の受け入れ
   基準（運用・保守性の項）を満たしたと判定した。
5. 親イシュー #98 の受け入れ条件チェックボックスは本レポートの確定結果に基づいて
   更新し、クローズ可否を判断する（本レポート自体のスコープ外、#98 側で対応）。
6. `RWS_BIND_ADDR` の実測検証（#162）が別途完了した場合、その結果を本レポート
   第 3 節の該当行またはイシューコメントで参照できるようにする（未完了のため
   本レポートの判定確定条件には含めない）。

## 7. 参照

- `docs/spec/04-requirements.md` REQ-9（121 行）・受け入れ基準（188〜200 行）
- `docs/spec/05-tasks.md` TASK-9.2（親タスク受け入れ基準）
- `docs/spec/03-poc/single-binary-distribution/README.md`（PoC-4 実証手順・実施内容 2）
- `docs/reports/perf-browser-report.md`（判定基盤整備＋保留方式の先例、TASK-11.5c・PR #208）
- Issue #98（親）・#99（TASK-9.2a・isolated_run.rs 実装）・#94〜#97（TASK-9.1・
  前提タスク）・#104（TASK-9.4・単一バイナリでの XSS エスケープ維持検証）・
  #162（`RWS_BIND_ADDR` 切り替え検証、本レポートのスコープ外）

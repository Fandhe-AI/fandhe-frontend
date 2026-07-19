# 開発時アセット変更の即時反映（REQ-10・TASK-10.1）

## 位置づけ

本書は `fandhe-frontend-dist-server`（`crates/dist-server/`）における静的アセット配信の開発体験
（DX）ガイドです。対象読者は `crates/dist-server/` を触る開発者、および `docs/spec/`
の REQ-10・TASK-10.1（`docs/spec/05-tasks.md`）の受け入れ状況を確認したい人。

- **実装元**: TASK-10.1a（イシュー #106、PR #215）でモード切り替え本体を、
  TASK-10.1b（イシュー #107、PR #216）で即時反映の回帰テストとキャッシュ
  ヘッダを製品化しました。親イシュー #105（TASK-10.1）はこの 2 つの統合と
  受け入れ検証・ドキュメント整備を扱います。
- **実装コード**: `crates/dist-server/src/assets.rs`（モード判定・アセット検索の本体）、
  `crates/dist-server/src/routes.rs`（`Cache-Control` ヘッダの付与判定）、
  `crates/dist-server/src/main.rs`（HTTP レスポンスへのヘッダ反映）。
- **設計上の位置づけ**: 統合設計書 `docs/design/dist-server-design.md` の §4.5 は
  TASK-9.1a 時点（実装前）の草案（`include_dir` クレート採用を想定）であり、
  実装はそれとは異なる自前実装（下記）を採用しています。§4.5 には実装済みで
  ある旨の注記を追加済みです。本書が現行の正とする DX ドキュメントです。

## 1. モード切り替え表

`crates/dist-server/src/assets.rs::AssetMode` はコンパイル時（`cfg!`）に確定し、
実行時には変化しません。

| ビルド条件 | モード（`AssetMode`） | 配信経路 |
|---|---|---|
| debug ビルド（`cfg(debug_assertions)`）かつ `force-embed` フィーチャー無効 | `DevFilesystem` | `static/` を毎リクエスト `fs::read` で読み込み（`dev_fs::lookup`） |
| debug ビルドで `force-embed` フィーチャー有効（`cargo test/build --features force-embed`） | `Embedded` | `build.rs` 生成の埋め込みテーブル完全一致検索のみ（`embedded_lookup`） |
| release ビルド（`cargo build --release` 等） | `Embedded` | 同上 |

**例外（WASM ビルド成果物）**: URL パスが `/static/wasm/` で始まるアセットは、
`DevFilesystem` モードでも常に埋め込みテーブルから配信されます。WASM 成果物
（TASK-10.2b、イシュー #110）は `crates/dist-server/build.rs` が `OUT_DIR` に生成する
ものであり、ソースツリー `static/` に実体を持たないためです。したがって
**「毎リクエストでディスクの最新内容を反映する」保証は `/static/wasm/*` 以外
の静的アセットにのみ適用されます**。

`force-embed` はコード・依存を一切持たない空フィーチャー（`crates/dist-server/Cargo.toml`
参照）で、`cfg(all(debug_assertions, not(feature = "force-embed")))` の判定
にのみ関与します。REQ-3（依存グラフ上限 60 件/深さ 6）の実測値（21 件/深さ 5）
に影響しません。

## 2. 即時反映の保証

`dev_fs::lookup`（`DevFilesystem` モードの実装）はリクエストのたびに
`fs::read` でディスクから読み直し、内容をキャッシュ・メモ化しません。この
契約は `dev_fs::tests` の 3 つの回帰テストで固定されています（TASK-10.1b、
イシュー #107）。

- `updated_file_content_is_reflected_on_next_lookup`: 既存ファイルの内容を
  書き換えた直後の次リクエストで新内容が返ることを確認
- `file_created_after_startup_is_served_immediately`: 起動後に新規作成した
  ファイルが即座に配信されることを確認
- `deleted_file_returns_none_immediately`: 削除直後のファイルが即座に
  404 相当（`None`）になることを確認

将来キャッシュ等の最適化を加える場合は、この 3 テストを弱めずに REQ-10 を
維持する必要があります。

**ブラウザ側キャッシュ対策**: `routes.rs::RouteResponse::cache_control` は
`DevFilesystem` モードの静的アセット 200 応答にのみ `Some("no-store")` を
設定し、`main.rs` がこれを `Cache-Control: no-store` ヘッダとして反映します
（ページ応答・404 応答・`Embedded` モードでは常に `None` で、ヘッダは付与
されません）。値は固定 `&'static str` のみで、リクエスト由来の文字列を
ヘッダへ流し込むことはありません（ヘッダインジェクション対策、
`.claude/rules/security.md`）。これにより、ブラウザキャッシュによって
ディスクの変更が体感上反映されない事態を防ぎます。

## 3. 開発フロー手順

1. `cargo run -p fandhe-frontend-dist-server`（または `FANDHE_FRONTEND_WASM_BUILD=0 cargo run -p fandhe-frontend-dist-server`
   — WASM ビルドステージをスキップする場合。`crates/dist-server/build.rs` 冒頭
   ドキュメント参照）で debug ビルドを起動する。既定で `DevFilesystem`
   モードになる。
2. `static/` 配下のファイル（`/static/wasm/*` を除く）を編集・追加・削除する。
3. サーバーをリビルド・再起動せずに、次回リクエストから変更が反映される。
   `curl -I` で `Cache-Control: no-store` が付与されていることも確認できる。
4. `/static/wasm/*` を配信対象に含めて検証したい場合は、WASM ビルド成果物の
   再生成（`build.rs` のビルドスクリプト実行、通常は `cargo build`/`cargo run`
   の再実行で行われる）が必要（1 の即時反映例外を参照）。

## 4. `force-embed` の用途

`cargo test -p fandhe-frontend-dist-server --features force-embed --locked` /
`cargo build -p fandhe-frontend-dist-server --features force-embed` のように指定すると、
debug ビルドのままファイルシステム読み込み経路を無効化し、release と同じ
`Embedded` モードの配信経路を検証できます。CI ジョブ
`dist-server-embedded-mode`（`.github/workflows/ci.yml`）がこれを実行し、
`Embedded` モードが常に有効に保たれていることを継続的に検証します
（PoC-4 の `rust-embed` `debug-embed` フィーチャー相当を自前実装で踏襲）。

## 5. セキュリティ不変条件（OWASP A01 パストラバーサル）

- `embedded_lookup`（`Embedded` モード）はコンパイル時に確定した固定テーブル
  への完全一致検索のみを行い、実行時にファイルシステムへアクセスしません。
  `../` を含むパスや URL エンコードされたパストラバーサル試行はテーブル中の
  いずれのキーとも完全一致しないため常に `None`（404 相当）になります。
- `dev_fs::lookup`（`DevFilesystem` モード）は二重防御を行います。
  1. **事前拒否**: `/static/` プレフィックス以外・`..` 成分・絶対パス成分・
     NUL 混入を文字列検査で拒否（パーセントデコードは行わないため、
     `..%2f` 等はデコードされず単なる不明ファイル名として 404 になります）。
  2. **事後検証**: 結合後パスを `fs::canonicalize` し、`static/` ルート
     （同じく canonicalize 済み）配下であることを `starts_with` で確認。
     シンボリックリンク経由でルート外へ脱出する試みはここで遮断されます
     （事前拒否だけでは防げないケースの最終防衛線）。
- **release バイナリへの攻撃面の持ち込み防止**: `dev_fs` モジュール全体が
  `#[cfg(all(debug_assertions, not(feature = "force-embed")))]` でゲートされて
  おり、release ビルド（および `force-embed` 有効時）にはファイルシステム
  読み込みコード自体がコンパイルされません。`Embedded` モードの `lookup()`
  実装は `dev_fs` を一切参照しない別関数であり、release バイナリに開発時
  専用の攻撃面が構造的に持ち込まれないことをコンパイラが保証します。

## 関連

- REQ-10（`docs/spec/04-requirements.md`）
- TASK-10.1 / TASK-10.1a / TASK-10.1b（`docs/spec/05-tasks.md`）
- イシュー #105（親）・#106（PR #215）・#107（PR #216）
- `docs/design/dist-server-design.md` §4.5（設計当時の草案・実装済み注記）

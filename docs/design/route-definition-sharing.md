# server / client 単一定義からのルート生成（共有機構）の設計比較（イシュー #407）

## 1. 目的とトレーサビリティ

イシュー #374（クライアント側ルーティング）は、`wasm-full` が `fandhe-frontend-server`
へ依存できない（`structure.toml` の `server.allowed_dependents =
["dist-server"]`）という構造上の制約から、ルート定義を `crates/server/src/ssr.rs`
（`PageRoute` enum + `Router` 登録 + タイトルリテラル）と
`crates/wasm-full/src/nav.rs`（`ClientRoute` enum + 独自セグメント一致 + タイトル
リテラル）に二重定義し、`crates/wasm-full/tests/route_sync_static.rs`（静的ソース
走査）によるドリフト**検知**でのみ同期を担保していた（PR #383 の申し送り）。

本書はイシュー #407 の受け入れ条件に従い、次を行う。

1. 単一定義から双方のルート解決を導出する構成を設計し、AI 開発評価軸
   （`docs/policy/intentional-non-adoption.md` §2 の 4 軸: 明示性・決定性・
   機械検証可能性・コンテキスト消費）で現行ドリフト検知方式と比較する
2. 採用する場合、ドリフト検知テストを生成方式の検証テストへ置き換える
   （検証の弱体化はしない）
3. 非採用の場合、`docs/policy/intentional-non-adoption.md` へ再評価トリガー
   付きで記録する

**結論（先出し）**: 案 B-1（`fandhe-frontend-app` へルート表 + マッチングエンジンを
集約する構成）を採用した。実装は本書の判断に従い完了済みであり、
`crates/app/src/router.rs`・`crates/app/src/routes.rs`・`crates/server/src/ssr.rs`・
`crates/wasm-full/src/nav.rs`・`structure.toml`・`crates/wasm-full/tests/route_shared_static.rs`
に反映されている。

## 2. 前提事実（実装着手前の調査結果）

| 事実 | 根拠 |
|------|------|
| `wasm-full` は `fandhe-frontend-server` へ依存不可 | `structure.toml`: `server.allowed_dependents = ["dist-server"]` |
| `fandhe-frontend-app` は `server` / `dist-server` / `wasm-full` / `wasm-client` すべてから依存可能 | `structure.toml`: `app.allowed_dependents` — **共有定義の適地は app 層のみ** |
| `fandhe-frontend-app` は外部依存ゼロ（`fandhe-frontend-core` のみに依存） | `crates/app/Cargo.toml` |
| `fw structure` の `fandhe-frontend-router-v1` 抽出器（`crates/cli/src/routes.rs`）は `[routing] definition_dir` 配下の `.rs` ファイルを**文字列走査**（正規表現・AST 不使用）し、`.route("<literal>", handler)` の第 1 引数が文字列リテラルであるものだけを拾う | `crates/cli/src/routes.rs::extract_routes_from_source`／`parse_route_args` |
| 抽出器は行コメント（`//`・`///`・`//!`）と `#[cfg(test)]` 以降を除去してから走査する（`strip_comment_lines`／`truncate_before_test_cfg`） | 同上。**`Router` の rustdoc 使用例（`.route("…")`）を実ルートと誤認しない** |
| 抽出器は `<dir>/src/` 配下のみを走査する（`tests/` 等の integration test は対象外） | `crates/cli/src/routes.rs::scan_root` |
| `Router`（`crates/server/src/router.rs`）は `crates/server/src/ssr.rs` からのみ実利用されており、`dist-server` は `fandhe_frontend_server::ssr::respond` 経由でのみ触れる（`Router` を直接 import しない） | `grep -rn "router::Router\|use crate::router"` の結果。移設の波及は server クレート内に閉じる |
| `fandhe-frontend-app` は既に `wasm-full`（`nav.rs` の `use fandhe_frontend_app::{Item, Loader}`）・`server` の双方に通常依存として存在する | `crates/wasm-full/Cargo.toml`・`crates/server/Cargo.toml` |

## 3. 比較対象の選択肢

### 案 A（現状維持）: 二重定義 + 静的ドリフト検知

`crates/server/src/ssr.rs` と `crates/wasm-full/src/nav.rs` にルート表を独立実装し、
`crates/wasm-full/tests/route_sync_static.rs` がパターン・タイトルの**リテラル
存在**を突き合わせる。

### 案 B（採用）: `fandhe-frontend-app` へのルート表 + マッチングエンジン集約

`crates/app/src/router.rs`（マッチングエンジン、`crates/server/src/router.rs` から移設）
と `crates/app/src/routes.rs`（ルート表: パターン + ハンドラ + タイトルの単一定義、
`resolve()`/`title()`）を新設し、`server`・`wasm-full` の双方がこれを呼ぶ。

サブ案を 2 つ検討した。

- **B-1（採用）**: `Router` を `fandhe-frontend-app` へ実体移設し、`fandhe-frontend-server` は
  `pub use fandhe_frontend_app::router::{...}` の再エクスポートシムに置き換える。
  マッチングエンジンが物理的に 1 つになるため、意味論ドリフト
  （末尾スラッシュ・空セグメント等の扱いの食い違い）が構造的に消滅する。
  §2 の調査で `Router` の実利用が `server` クレート内に閉じることを確認済み
  であり、波及は限定的。
- **B-2（不採用）**: `Router` は `server` に残し、`fandhe-frontend-app` に v1 仕様
  サブセットの小型 resolver を別実装する。エンジンが 2 つ存在するため
  等価性テストで意味論一致を強制する必要があり、B-1 より保守コストが高い。
  B-1 の実現性（§2 で確認済み）が高いため採用しない。

### 案 C（不採用）: core 層へのルート定義配置

`fandhe-frontend-core` は「外部依存ゼロの描画コア」であり、ルーティングは責務外
（`structure.toml` の role 宣言・`no_branching_across_modes` の設計思想に
反する）。`core` は `interactive`/`app`/`server`/`wasm-*`/`dist-server` の
すべてから依存される最下層であり、ルーティングのようなアプリケーション層の
関心事を持ち込むと責務境界が曖昧になる。比較のみ行い実装は検討しない。

## 4. AI 開発評価軸での比較

`docs/policy/intentional-non-adoption.md` §2 の 4 軸で案 A・案 B-1 を比較する。

| 評価軸 | 案 A（現状維持: 二重定義 + ドリフト検知） | 案 B-1（採用: fandhe-frontend-app へ単一定義） |
|--------|------------------------------------------|--------------------------------------|
| **明示性** | ルート追加のたびに `ssr.rs`・`nav.rs`・`route_sync_static.rs` の 3 箇所を手動同期する必要があり、「どこが正か」が実装から読み取れない（3 箇所とも「実装」に見える） | `crates/app/src/routes.rs` が唯一の正本であることが `pub mod routes` のドキュメントコメント・rustdoc から自明。`server`/`wasm-full` は `fandhe_frontend_app::routes::resolve`/`title` を呼ぶだけで独自ロジックを持たない |
| **決定性** | 2 つの独立実装（`Router::resolve` と `nav.rs` の独自セグメント一致）が同じ入力に対し同じ結果を返す保証がテストでしか担保されず、実装変更時に一方だけ更新されるとテストが通るまで気づけない（テストのカバレッジ漏れが即ドリフトに直結） | エンジンが物理的に 1 つ（`fandhe_frontend_app::router::Router`）のため、同一入力に対する結果不一致がそもそも発生し得ない（テストの網羅性に依存しない構造的決定性） |
| **機械検証可能性** | `route_sync_static.rs` は「両ファイルに同じ**リテラル**が存在する」ことしか検証できず、意味論（末尾スラッシュの扱い等）の一致は検証対象外 | 検証すべきは「両ファイルがリテラルを再定義していないこと」＋「共有 API を参照していること」のみであり、静的走査（`route_shared_static.rs`）で十分かつ従来より狭い検証面で足りる。意味論は共有実装のユニットテスト（`crates/app/src/routes.rs`）1 箇所に集約される |
| **コンテキスト消費** | ルート変更時、実装者（人間・AI いずれも）が 3 ファイルすべてを把握し同期させる必要がある。レビュー時も 3 箇所の diff を突き合わせる必要がある | ルート変更は `crates/app/src/routes.rs` 1 箇所の編集で完結する。`server`/`wasm-full` 側は呼び出しコードの変更が不要なため、レビュー・実装いずれもコンテキスト消費が小さい |

**判定**: 4 軸すべてで案 B-1 が案 A を上回る。

## 5. 採用判断基準（決定ゲート）の評価

実装計画に定めた 4 基準をすべて満たすことを実装後に確認した。

1. **外部クレート追加ゼロ**: `crates/app/Cargo.toml` に変更なし（`fandhe-frontend-core` のみに
   依存のまま）。`cargo metadata` の `resolved_package_count` は移設前後で
   72 のまま不変（`fw structure` 出力で確認）。✅
2. **`fandhe-frontend-router-v1` 抽出器の追随**: `structure.toml` の
   `[routing] definition_dir` を `"server"` → `"app"` に変更するのみで、
   抽出器本体（`crates/cli/src/routes.rs`）は無改修のまま
   `crates/app/src/routes.rs::build_router()` の `.route("/", AppRoute::List)` /
   `.route("/items/:id", AppRoute::Detail)` を抽出できることを
   `crates/cli/src/routes.rs::extract_routes_reads_real_router_source`（走査先を
   `"app"` へ追随）で固定した。`crates/app/src/router.rs` の rustdoc 使用例
   （`.route("/items/:id", "item_detail")?` 等）は `strip_comment_lines` の
   `///` フィルタで除外されることを実行結果で確認済み。✅
3. **既存テストの無弱体化での全 pass**: `cargo test --workspace` で
   `nav_native.rs`（13 件）・`server` の ssr/ssg/router 統合テスト・
   `no_branching_across_modes.rs` を含む全テストが**無修正**のまま pass
   した。✅
4. **`wasm-full` の bundle_size 予算内**: `crates/wasm-full/tests/bundle_size.rs`
   の `wasm_full_bundle_gzip_size_within_req11_limit` が pass（`fandhe-frontend-app` は
   移設前から `wasm-full` の依存グラフに含まれていたため、`Router`/`routes`
   追加分のコード量のみが増分であり、予算超過なし）。✅

4 基準すべてを満たすため、案 B-1 を採用する。

## 6. 実装概要（案 B-1）

| パス | 内容 |
|------|------|
| `crates/app/src/router.rs`（新規、`crates/server/src/router.rs` から移設） | v1 マッチングエンジン本体（`Router<H>`/`Params`/`RouteMatch`/`RouterError`）。rustdoc を `fandhe_frontend_app::router::Router` 参照へ更新 |
| `crates/app/src/routes.rs`（新規） | `AppRoute`（`List`/`Detail`）・`ResolvedRoute`・`resolve(path) -> Option<ResolvedRoute>`・`title(route) -> &'static str` のルート表単一定義。ビルダー DSL は `.route("<literal>", AppRoute::Variant)` の形を維持し抽出器互換を保つ |
| `crates/server/src/router.rs`（置換） | `pub use fandhe_frontend_app::router::{Params, RouteMatch, Router, RouterError};` の再エクスポートシムのみ。`crates/server/tests/router_resolution.rs` 等の既存呼び出し元は無修正で利用継続 |
| `crates/server/src/ssr.rs`（変更） | `PageRoute` enum・`build_page_router()`・`page_router()` を削除し、`fandhe_frontend_app::routes::{resolve, title, AppRoute}` を直接呼ぶ形へ置換。タイトルリテラルの再定義を排除 |
| `crates/wasm-full/src/nav.rs`（変更） | `resolve_path` の内部実装を `fandhe_frontend_app::routes::resolve` への委譲へ置換。`resolve_route_view_with` のタイトルを `fandhe_frontend_app::routes::title` 経由へ置換。`ClientRoute` 型・公開シグネチャは非破壊のため呼び出し元（`nav_native.rs`・`nav_browser.rs`・`wiring` モジュール）は無修正 |
| `crates/wasm-full/tests/route_sync_static.rs` → `route_shared_static.rs`（置換） | 検証内容は §7 参照（受け入れ条件 2） |
| `structure.toml` | `[routing] definition_dir = "app"`。`app`/`server` の `description` を移設内容に合わせて更新 |
| `crates/cli/src/routes.rs` | 統合回帰テスト `extract_routes_reads_real_router_source` の走査先・期待ハンドラ名（`AppRoute::List`/`AppRoute::Detail`）を追随（抽出器本体は無改修） |
| `templates/app/Cargo.toml` | `fandhe-frontend-app = "0.1.0"` への crates.io バージョン依存宣言により、生成プロジェクトが公開クレートから `router.rs`/`routes.rs` を取得する形に変更（イシュー #493）。`crates/cli/tests/template_vendor_drift.rs` が依存バージョンの整合性を検証 |

## 7. ドリフト検知テストの置き換え（受け入れ条件 2: 弱体化しない）

`crates/wasm-full/tests/route_shared_static.rs` は旧 `route_sync_static.rs` の検証
範囲（パターンリテラル存在・タイトルリテラル存在の確認）を次の点で**上回る**。

1. **単一定義の強制**（旧: リテラルの**存在**確認 → 新: **非再定義**の確認）:
   `crates/server/src/ssr.rs`・`crates/wasm-full/src/nav.rs` のいずれにもルートパターン
   リテラル（`"/"`・`"/items/:id"`）・タイトルリテラル（`"記事一覧"`・
   `"記事詳細"`）が**存在しない**こと、かつ両ファイルが `fandhe_frontend_app::routes`
   を参照していることを固定する（`server_ssr_does_not_redefine_route_literals_and_references_shared_routes`・
   `wasm_full_nav_does_not_redefine_route_literals_and_references_shared_routes`）。
   旧テストは「両ファイルに同じリテラルがある」ことしか見ておらず、
   片方だけ更新されるドリフトを**事後**にしか検知できなかったが、新テストは
   リテラルの再定義自体を禁止するため、ドリフトが構造的に起こり得ない。
2. **エンジン一本化の固定**: `crates/server/src/router.rs` が独自の `struct Router`
   定義を持たず `fandhe_frontend_app::router` の再エクスポートのみであることを固定する
   （`server_router_module_is_a_reexport_shim_not_a_duplicate_engine`）。
   B-1（エンジン統合）が今後の変更で B-2 相当（エンジン分岐）へ後退しない
   ことを継続検証する。
3. **意味論の直接固定**: `crates/app/src/routes.rs` の `#[cfg(test)]` に v1 仕様
   ベクトル（クエリ除去・末尾スラッシュ厳格一致・空セグメント拒否・`:id`
   捕捉・非 `/` 始まり拒否・XSS ペイロード風パス）を追加した。共有 resolver
   は単一実装のため、旧設計が想定していた「2 実装の等価性テスト」は
   B-1 採用により不要になった（エンジンが 1 つしかないため原理的に等価）。
4. 既存の `nav_native.rs`（13 件）・`server` の ssr/ssg テスト・XSS 回帰は
   **無修正のまま維持**した（`#[ignore]` 追加・削除・弱体化なし。
   `cargo test --workspace` で全 pass を確認済み）。

## 8. セキュリティ考慮事項（OWASP Top 10）

- **A03 インジェクション / XSS**: ルート定義の共有化でエスケープ経路は
  変更していない。タイトル・ページ本文は従来どおり `page_shell` / ノード木
  API（既定エスケープ）経由のみ。`:id` 捕捉値（`ResolvedRoute::id`）は
  loader 入力にのみ使い HTML へ直接出力しない契約を `crates/app/src/routes.rs` の
  rustdoc に明文化した。XSS 回帰テスト（`crates/app/src/router.rs`・
  `crates/app/src/routes.rs`・`server`・`wasm-full` の各テスト）は削除・弱体化
  していない
- **A01 / オープンリダイレクト**: `nav.rs` の `is_safe_relative_path`・
  ルート表非一致時のブラウザ既定遷移委譲は無変更
- **A04 安全でない設計（fail-closed）**: `respond_with`／
  `resolve_route_view_with` の loader `Err` → 固定文言応答の構造は無変更。
  `ResolvedRoute::id` が `None` になる防御的フォールバック（`/items/:id` は
  常に `id` を捕捉するため通常到達しないが、内部実装変更に対する防御として
  維持）も踏襲した
- **A05 セキュリティ設定ミス**: `structure.toml` の `allowed_dependents`
  制約（`wasm-full` → `server` 依存禁止）を破らない構成（`app` 層配置）とし、
  `fw structure`（`cargo run -p fandhe-frontend-cli -- structure --project .`）で
  validate が通ることを確認した
- **A06 脆弱な依存 / サプライチェーン**: 外部クレート追加ゼロ（`cargo
  metadata` の `resolved_package_count` 不変を確認）

## 9. 対象外（out-of-scope）

- `wasm-thin`（最小 CSR 構成）へのルーティング機能横展開は本イシューの
  スコープ外（`wasm-thin` は現状ルーティングを持たない）
- `templates/default`（`fw new`（テンプレートなし）の標準テンプレート）への
  ルート共有パターンの反映は、標準テンプレートがルーティング機能を含む
  アプリを生成しないため対象外
- `/search` ルートの `fandhe-frontend-app` 側実装追加は既存の凍結事項（`app-api.md`）を
  踏襲しスコープ外のまま

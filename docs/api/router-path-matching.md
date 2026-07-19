# fandhe-frontend-server パスマッチング仕様（v1）（TASK-7.2a）

## 1. 目的とトレーサビリティ

本ドキュメントは REQ-7（`docs/spec/04-requirements.md` REQ-7 節）が求める
「最小埋め込み〜フルスタックの共通コアとして、基本的なパスマッチングによる
ルート解決を提供する」の受け入れ基準（PoC-3 の 3 ルート相当:
`/`・`/items/:id`・`/search`）を満たす、v1 スコープのパスマッチング仕様を
**確定**するための成果物です。

`docs/spec/05-tasks.md` の TASK-7.2 は a〜c に分割されています。

- **TASK-7.2a（本ドキュメント・#55）**: v1 パスマッチング仕様の設計確定
- **TASK-7.2b（#56・実装済み）**: パスマッチングエンジンの実装
- **TASK-7.2c（#57）**: 公開 API 経由の統合ルーティングテスト整備

**実装位置の更新（イシュー #407）**: エンジン本体は `app/src/router.rs`
（`fandhe_frontend_app::router::Router`）へ移設した。`server`（SSR/SSG）・`wasm-full`
（CSR）の双方が同一エンジンを共有し、パスマッチング意味論のドリフトを
構造的に排除するための移設であり、本書が定める v1 仕様自体に変更はない。
`server/src/router.rs`（`fandhe_frontend_server::router`）は非破壊のための再エクスポート
シムとして存置する。設計判断の詳細は `docs/design/route-definition-sharing.md`
を参照。

**本文書のステータス**: TASK-7.2a 確定版。TASK-7.2b は本書に先行して
実装されているが、実装時点の挙動は本書が定める仕様と一致することを
本書作成時に確認済みであり、差分はない。今後 `router.rs` に変更を加える
場合は本書を正とし、実装と本書の記述に乖離が生じたときは本書の更新
または実装側の修正のいずれかで整合を取り戻すこと（PR レビューで指摘する）。

## 2. 責務境界・呼び出し文脈

`fandhe_frontend_server::router::Router<H>` は HTTP・HTML を一切知らない、パス文字列と
ハンドラ型 `H` のみを扱う汎用パスマッチング機構です。

- SSR（`server/src/ssr.rs` の `respond()`）・SSG（`server/src/ssg.rs`）・
  単一バイナリ配布（`fandhe-frontend-dist-server`）のいずれの上位層からも同一の
  `Router` / `Router::resolve()` を呼び出せることを想定する。
- `Router` はエスケープ責務を持たない。抽出したパスパラメータ（`Params`）は
  URL デコードしない生文字列のまま返し、HTML へ出力する際は呼び出し元が
  必ず `fandhe_frontend_core::text` / `fandhe_frontend_core::el` の attrs 経由で既定エスケープ
  （REQ-1）を通すこと。

## 3. v1 マッチング仕様

| 項目 | 仕様 |
|------|------|
| パターンの開始 | `/` から始まる必要がある（`route()` は違反時 `RouterError::MissingLeadingSlash` を返す） |
| セグメント一致 | セグメント単位の完全一致 |
| パスパラメータ | `:name` セグメントは空でない 1 セグメントを捕捉し `Params` へ格納する |
| 優先度規則 | なし。登録順の先勝ち（同一パターンを複数回登録した場合は最初の登録が勝つ） |
| クエリ文字列 | `?` 以降は照合前に切り落とす（`resolve()` 内部で `split_once('?')`） |
| 末尾スラッシュ | 正規化しない厳格一致（`/items/1/` と `/items/1` は別物として扱い、一致しない） |
| 連続スラッシュ | パターン・リクエストパスとも空セグメントとして扱われ、一致しない（パターン登録時は `RouterError::EmptySegment` で拒否） |
| 空パラメータ名 | `"/items/:"` のようにコロン直後が空の場合は `RouterError::EmptyParamName` を返す |
| パラメータ名重複 | 同一パターン内で `:id` を 2 回宣言する等は `RouterError::DuplicateParamName` を返す |
| パラメータ値 | URL デコードしない生文字列のまま `Params` に保持する |

## 4. v1 スコープ外（明記）

以下は v1 のパスマッチング仕様には含めない。将来必要になった場合は
別タスクとして起票する（`.claude/rules/out-of-scope-tracking.md` 準拠）。

- ワイルドカードセグメント（`*path` 等の可変長キャッチオール）
- パーセントデコード（`%XX` エスケープの解決）
- HTTP メソッド別ディスパッチ（`GET`/`POST` 等でのルート分岐）
- ネストレイアウト・データローディング（フレームワーク上位層のルーティング機能）
- パターン間の優先度規則（静的セグメント優先等の曖昧性解消）

## 5. セキュリティ不変条件

- **パストラバーサル耐性**: 照合は文字列比較のみで行い、ファイルシステムへ
  一切アクセスしない。ルーターの実装はファイルパス解決 API を持たない
  構造上の理由により、パストラバーサルの影響面を持ち得ない。
- **DoS 耐性**: 登録ルート数 × リクエストパスのセグメント数に比例する
  線形走査のみを行う。正規表現・再帰・バックトラックを一切使わない。
- **panic しない**: 不正なパターン登録（`route()`）は `RouterError` を
  `Result::Err` として返す。リクエストパスの解決（`resolve()`）は
  一致しないパスに対して `None` を返す。いずれもエンドユーザー入力
  （リクエストパス）に起因して panic することはない。
- **エスケープ非経由**: `Params` の値は生文字列であり、HTML 化は本モジュールの
  責務外。既定エスケープ（REQ-1）は呼び出し元（`fandhe-frontend-app` の描画関数経由）が
  担う契約を `router.rs` のモジュール doc に明記する。

## 6. REQ-7 受け入れ基準との対応

| 受け入れ基準 | 対応 |
|---|---|
| PoC-3 相当の 3 ルート（`/`・`/items/:id`・`/search`）が解決できる | `server/src/router.rs` の unit テスト `resolves_req7_baseline_routes` で固定。`server/tests/router_resolution.rs`（TASK-7.2c・#57）の `resolves_req7_baseline_routes_via_public_api` で公開 API 経由でも固定済み |
| 高度なルーティング機能（ワイルドカード・データローディング等）は対象外 | 本書 §4 に明記し、実装しない |

## 7. 備考: `/search` の配線状況

`server/src/ssr.rs` の `respond()` は `/` と `/items/:id` のみを
`fandhe_frontend_app` のページ関数へ配線している。`/search` はルーター自体では
マッチング可能（unit テストで検証済み）だが、`fandhe-frontend-app` の凍結 API
（`docs/api/app-api.md`）に検索ページの実装がないため、SSR エントリでの
ページ配線はスコープ外として見送っている。将来 `fandhe-frontend-app` に検索ページが
追加された際に配線する（別タスク）。

# Vercel 対応方式の比較と採用方式の決定

## 1. 背景・目的

親ツリー #3282（Vercel デプロイ対応）の Phase 1（#3283）の最終タスクとして、
Vercel 対応方式の比較と採用方式の決定を行う決定記録です。先行 2 件の調査は
完了・マージ済みで、成果物は次の 2 本のレポートです。

- `docs/reports/vercel-runtime-panic-repro-3284.md`（#3284 / PR #3311）:
  `vercel_runtime 1.1.6` は依存クレート `lambda_runtime 0.14.4` の
  `Config::from_env()` が `AWS_LAMBDA_FUNCTION_NAME`・`_MEMORY_SIZE`・
  `_VERSION` の 3 環境変数を `expect` で必須にしており、これらを持たない
  Vercel の Rust Function 実行環境では起動直後に panic して全リクエストが
  HTTP 500 になることをローカル・Vercel 本番の両方で実測しています。
  fandhe-frontend 側の実装は panic 経路に一切関与しません。
- `docs/reports/vercel-runtime-version-survey-3285.md`（#3285 / PR #3312）:
  `vercel_runtime 2.x`（crates.io 最新 `2.4.1`）は `lambda_runtime`・
  `lambda_http` に一切依存しない独立実装であり、Vercel 本番で HTTP 200 を
  実測済みです。`lambda_runtime` は git 最新（`1.4.0`）でも同じ 3 変数に加え
  `AWS_LAMBDA_RUNTIME_API` が必須なため、Vercel では構造的に動作しません
  （版を固定しても解決しない性質の不一致です）。`vercel-community/rust`
  （1.x を配布していたリポジトリ）は archived で、README に正式な
  Deprecation Notice があり、upstream への新規報告は不要と判定済みです。
- 親 #3282 の記述: SSG（`generate_pages`）の出力を Build Output API 形式で
  `vercel deploy --prebuilt` 配置する形態は既に成立しています。

本文書の目的は、上記の事実をもとに Vercel 対応の 4 案を比較し、採用方式と
不採用理由を決定記録として残すことです。Phase 2（#3287 配下の
#3288〜#3292）が前提として参照できる状態にします。

### 比較対象の 4 案

| 案 | 内容 |
|---|---|
| a | 動作する Vercel ランタイムクレートの版を固定して使う（`vercel_runtime = "=2.4.1"`） |
| b | `lambda_runtime` を使わない Vercel 向けアダプタを fandhe 側で自作する |
| c | SSG の出力を Build Output API 形式で `--prebuilt` 配置し、動的処理だけを別の関数にする |
| d | Docker（`examples/dist-server-docker` の単一バイナリ）で動かす |

## 2. 前提となる実測事実

| 事実 | 出典 |
|---|---|
| `vercel_runtime 1.1.6` は `lambda_runtime 0.14.4::Config::from_env()` の `expect` により、Vercel Rust Function 実行環境（`AWS_LAMBDA_*` 系環境変数が皆無）で起動時 panic し、全リクエストが HTTP 500 になる | #3284 §6・§8（ローカル・Vercel 本番実測） |
| Vercel の Rust Function 実行環境には `AWS_LAMBDA_*` 系の環境変数が一切存在しない（実測 43 個の環境変数名に該当なし） | #3284 §7 |
| `vercel_runtime 2.4.1`（crates.io 最新）は `lambda_runtime`/`lambda_http` に依存しない独立実装であり、`cargo tree -i lambda_runtime` で当該パッケージが存在しないことを確認済み。Vercel 本番で HTTP 200 を実測 | #3285 §6（セル B） |
| `lambda_runtime` は git 最新（`1.4.0`）でも 3 変数の `expect` に加え、`lambda_runtime_api_client` が `AWS_LAMBDA_RUNTIME_API`（AWS Lambda Runtime API のポーリングエンドポイント）を必須にする。Vercel はこの API 自体を提供しないため、版を固定しても解決しない | #3285 §7 |
| `vercel-community/rust`（1.x の配布元）は archived、README に「legacy な 1.x は非推奨、新規 issue は `vercel/vercel` へ」という明示的な Deprecation Notice がある。upstream への新規報告は不要と判定済み | #3285 §8・§9 |
| SSG（`generate_pages`）の出力を Build Output API 形式で `vercel deploy --prebuilt` 配置する形態は成立している | 親 #3282 issue 本文 |
| 新規 Vercel プロジェクトは既定で Deployment Protection（Vercel Authentication / SSO）が有効で、無効化しないと 302 でリダイレクトされる | 親 #3282 issue 本文 |
| ただし #3284 の検証では、この迂回操作は不要だった（本番エイリアス URL への直接アクセスで 401/302 を挟まず 500 が返り、Function ログにも到達できた） | #3284 §5 補足 |
| Vercel Rust ランタイム（`vercel_runtime`/`@vercel/rust`）は 2026-09-27 時点で公式に **Beta** と明記されている（"🔒 Permissions Required: The Rust runtime (Beta)"、変更履歴「Rust runtime now in public beta for Vercel Functions」） | 公式ドキュメント <https://vercel.com/docs/functions/runtimes/rust>（`last_updated: 2025-12-08`、2026-09-27 取得） |
| Vercel は Container Images（OCI 互換の任意コンテナイメージを Vercel Functions として実行する仕組み）を提供しており、2026-09-27 時点で公式に **Beta** と明記されている。デプロイ経路はローカル/CI で `Dockerfile.vercel` を**利用者側がビルド**し、そのコンテナイメージを `vercel deploy` 等で Vercel Container Registry へ格納・Fluid Compute 上で自動スケールする形態であり、Vercel 側が Dockerfile からイメージをビルドするわけではない（Vercel 側に Rust ツールチェーンは不要）。要件は「`$PORT`（既定 80）で HTTP サーバーを待ち受ける」「スケールイン時に 30 秒の猶予付き `SIGTERM` を受け取り自身で終了処理する」の 2 点 | 公式ドキュメント <https://vercel.com/docs/functions/container-images>（`last_updated: 2026-07-07`、2026-09-27 取得） |
| `crates/dist-server/src/main.rs` の既定 bind は `FANDHE_FRONTEND_BIND_ADDR`（既定 `127.0.0.1` 系のループバック）であり、Vercel Container Images が要求する `$PORT` 環境変数の読み取りには対応していない（現状） | `crates/dist-server/src/main.rs` 実装確認（2026-09-27） |

## 3. 比較表

観点は受入基準の 4 つ（成立性・保守負担・REQ-1/REQ-2 との整合・beta 依存
リスク）に、補助観点として REQ-3（依存上限）・REQ-4（cargo-deny）・Vercel
側で Rust ツールチェーンが必要かを加えます。

| 観点 | a. ランタイム版固定（2.x） | b. 自作アダプタ | c. SSG + Build Output API | d. Docker（単一バイナリ） |
|---|---|---|---|---|
| **成立性** | 成立（実測: Vercel 本番 HTTP 200、#3285 §6 セル B）。ただし SSR/動的処理限定 | 未検証・未実装。Vercel Functions の内部プロトコル（`VERCEL_IPC_PATH` 等）は非公開のため設計から不確実 | 成立（実測、親 #3282 issue 本文）。静的出力限定 | 部分的に成立し得る（推定）。Vercel Container Images（Beta、2026-07-07 確認）は任意 OCI イメージを Functions として実行できるが、`$PORT` 待受・`SIGTERM` 対応の追加実装が現状の `dist-server` に必要（未実装、上表参照） |
| **保守負担** | 中。crates.io 版を `=2.4.1` で固定し API 変更（`run(handler)` → `run(service_fn(handler))`）に追随する程度 | 高。文書化されていない Vercel 内部プロトコルの再実装をフレームワーク側で抱え込む | 低。既存の `ssg::generate_pages`（外部依存ゼロ）のみで完結 | 中〜高。REQ-9 の配布形態を Vercel Container Images の要件（`$PORT`/`SIGTERM`）へ適合させる追加アダプタ作業が必要 |
| **REQ-1/REQ-2 との整合** | 整合可能。ハンドラで `fandhe_frontend_server::ssr::respond`/`respond_with`（既定エスケープ経由、外部依存ゼロ）を呼ぶ限り既定エスケープは保たれる | 設計次第。自作アダプタ自体は `fandhe-frontend-core`/`-server` の外側（example）に置く前提のため既定エスケープ自体には影響しないが、独自プロトコル実装が `unsafe` 境界を増やすリスクは残る | 整合。`generate_pages` は既定エスケープ経由の出力をそのままファイル化するのみ | 整合。`fandhe-frontend-dist-server` は既存の REQ-1/REQ-2 準拠実装をそのまま使う |
| **beta 依存リスク** | あり。Rust ランタイム自体が公式に Beta（上表参照）。加えて [vercel/vercel#14532](https://github.com/vercel/vercel/issues/14532)（Rust runtime v2 の「address already in use」、2026-09-27 時点で open）が既知の留意点 | **該当なしではない**。案 b も `@vercel/rust` ビルダー（下表「Vercel 側で Rust ツールチェーンが必要か」参照）を前提とするため、Rust ランタイム自体が公式に Beta である点は案 a と共有するリスクである。これに加えて、文書化されていない Vercel Functions 内部プロトコルを自前実装する非公開プロトコル再実装リスクを独自に負う（案 a より広いリスク面） | 低。Build Output API・`--prebuilt` は Vercel の基盤機能であり Rust 固有の beta 機能に依存しない | あり。Container Images 自体が公式に Beta（2026-07-07 確認） |
| REQ-3（依存 60 件/深さ 6） | 未確認。`vercel_runtime 2.4.1` 単体の依存木の件数・深さは #3285 で未計測（`lambda_runtime` が消えたことのみ確認済み）。Phase 2（#3288）の前提条件とする | 該当なし（依存追加なし） | 影響なし（既存クレートのみ） | 影響なし（既存 `dist-server` の構成のまま） |
| REQ-4（cargo-deny） | 未確認。`=2.4.1` の advisories は example ローカルの `deny.toml` で確認する（Phase 2 前提条件） | 該当なし | 影響なし | 影響なし |
| Vercel 側で Rust ツールチェーンが必要か | 必要（`@vercel/rust` ビルダーが Vercel 側でコンパイル） | 必要（同上、自作でも Rust ハンドラをビルドする以上不可避） | 不要（ローカル/CI でビルド済みの静的出力を配置するのみ） | 不要（コンテナイメージはローカル/CI でビルド済み。Vercel は VCR へ格納するのみ） |

## 4. 採否判定

### 採用: 案 c（SSG + Build Output API + `--prebuilt`）を既定方式とする

実測で成立しており（§2）、Vercel 側に Rust ツールチェーンが不要で、beta の
ランタイムにも依存しません。レンダリングは
`fandhe_frontend_server::ssg::generate_pages`（外部依存ゼロ・既定エスケープ
経由）だけで行います。

### 併用（条件付き）: 案 a（`vercel_runtime = "=2.4.1"`）を動的処理・SSR 用に採用する

「案 a」は `vercel_runtime = "=2.4.1"`（2.x 系の完全一致固定）という意味に
**限定**します。「`lambda_runtime` の版を固定する」という読み方は #3285 §7
のとおり構造的に成立しないため、この意味では不採用です。

配置の決定は次のとおりです。

- **`vercel_runtime` は `crates/*`（公開クレート）の依存に入れません。**
  アダプタ（ハンドラ）は `examples/vercel-ssr`（`examples/ssr-routing` と
  同様に独立 workspace として root から切り離す）の中に置きます。理由:
  1. 新しい公開クレートを作ると `docs/ci/version-bump-publish-order-gap.md`
     §11 の初回公開手順が必要になる
  2. root の `deny.toml` と REQ-3（60 件・深さ 6）の計測対象に、依存木が
     未確認の 2.x が入ってしまう
  3. beta の上流が変化したときの影響範囲を example に閉じ込められる
  4. コアクレートの「外部依存ゼロ」を崩さない
- **ラップする対象**: #3288 のタイトルにある `route_request` は
  `fandhe-frontend-dist-server`（hyper / hyper-util / tokio と WASM ビルド用
  `build.rs` を持つ）にあり、Vercel 関数からの呼び出しには向きません。HTTP
  に依存しない面として `fandhe_frontend_server::ssr::respond` /
  `respond_with`（`SsrResponse { status, content_type, body }` を返し、外部
  依存ゼロ、`crates/server/src/ssr.rs`）を推奨対象とします。ただし
  `respond_with` の loader 型は `Item` の一覧・詳細というデモ用の形に固定
  されているため、利用者のアプリで「自前ルータ → `Node` → `render()` →
  (status, content_type, body)」の形にどう適用するかは #3288 で確かめる
  課題として引き継ぎます。静的アセットは Build Output API の `static/` が
  担うため、`route_request` の `/static/` 分岐は不要です。
- **Phase 2 の前提条件**: 2.x を使う前に、`vercel_runtime 2.4.1` の依存木に
  ついて `cargo tree` / `cargo metadata` による件数・深さの計測と、依存各
  クレートの `build.rs` の有無を確認します。あわせて example ローカルの
  `deny.toml` で advisories を確認します（#3285 ではこの確認をしていない
  ため、#3288 の完了条件に含めます）。

### 不採用: 案 b（自作アダプタ）

2.x は `VERCEL_IPC_PATH` 等を使う Vercel 内部の IPC 規約で動いていると
見られます（実装ソースの公開状況からの推定であり、Vercel 自身が仕様を
公開しているわけではありません）。また、案 b も `@vercel/rust` ビルダーを
前提とする以上、Rust ランタイム自体が公式に Beta である点は案 a と共有する
リスクです。自作アダプタは、この文書化されていない規約を fandhe 側で
再実装することになり、案 a と共有する beta リスクに加えて非公開プロトコル
再実装という独自リスクを重ねて抱え込む形になります。保守負担が最も大きい
ことを理由に不採用とします。

### 保留（既定不採用、再評価余地あり）: 案 d（Docker）

**#2 節の実測事実を踏まえ、当初想定より成立性の見通しは改善しています。**
Vercel は Container Images（Beta、2026-07-07 確認）により、ローカル/CI で
`Dockerfile.vercel` からビルドした任意の OCI イメージを Vercel Functions
として実行できます（Vercel 側は当該イメージを Container Registry へ格納
するのみで、Dockerfile からのビルド自体は行いません）。これは
「Vercel は任意のコンテナイメージの実行を一切サポートしない」という単純な
不成立ではなく、「Vercel Functions の一形態として、`$PORT` 待受・
`SIGTERM` 対応というアプリケーション側の追加要件を満たせばコンテナイメージ
を実行できる」という条件付きの成立性です。

ただし、次の理由により Phase 2 の対象には含めず、Vercel 上の既定方式とし
ては採用しません。

1. `fandhe-frontend-dist-server`（`crates/dist-server/src/main.rs`）は現状
   `FANDHE_FRONTEND_BIND_ADDR`（既定ループバック）で bind しており、
   Vercel Container Images が要求する `$PORT` 環境変数の読み取りに対応して
   いません。適合には追加のアダプタ実装が必要です（未実装）。
2. Container Images 自体が Beta であり、案 a と同種の beta 依存リスクを
   負います。案 c（既定方式）・案 a（動的処理の併用）で受入基準を満たせる
   ため、beta 機能への依存を重ねて増やす理由がありません。
3. REQ-9 の単一バイナリ配布（Docker 想定）はフレームワーク本来の配布経路
   として Vercel 以外のホスティング（コンテナ実行基盤全般）向けに変わらず
   有効であり、Vercel 固有の適合作業を今 Phase 2 のスコープへ含める必要性
   は薄いと判断します。

再評価トリガーは §5 に記載します。この判断は「Vercel はコンテナ実行を
サポートしない」という誤った前提に基づくものではなく、「サポートはある
が beta かつ追加適合作業が要る」という事実に基づく優先順位判断です。

## 5. 実装上の決定事項

- アダプタ（案 a）は `examples/vercel-ssr` に閉じ込め、`crates/*` の依存
  グラフには一切入れません。
- ラップ対象は `fandhe_frontend_server::ssr::respond` / `respond_with`
  系であり、`fandhe-frontend-dist-server::routes::route_request` ではあり
  ません（HTTP ソケット層に依存しない純粋関数を Vercel ハンドラから直接
  呼ぶ設計とします）。
- 2.x の依存木計測（`cargo tree`/`cargo metadata`）・`build.rs` の有無確認・
  example ローカル `deny.toml` による advisories 確認は #3288 の完了条件に
  含めます（本イシューでは未実施）。
- `vercel.json` の `functions.*.runtime` 明示指定は 2.x（`@vercel/rust`
  ゼロコンフィグ）では不要です（#3285 §3.3・§10、公式ドキュメントの
  `Cargo.toml` 例でも `vercel.json` を必要としていません）。

## 6. Phase 2 への引き継ぎ表

| Issue | 引き継ぐ前提 |
|---|---|
| #3288（feat: `route_request` を Vercel Functions で動かすアダプタを実装する） | **要見直し**: issue タイトルは `route_request`（`fandhe-frontend-dist-server`）を指しているが、本決定記録の推奨ラップ対象は `fandhe_frontend_server::ssr::respond`/`respond_with`（外部依存ゼロ、HTTP ソケット非依存）である。案 a（`vercel_runtime = "=2.4.1"`）でハンドラを `examples/vercel-ssr` 内に実装し、依存木計測（`cargo tree`/`cargo metadata`）・`build.rs` の有無確認・`deny.toml` advisories 確認を完了条件に含める |
| #3289（feat: `examples/vercel-ssr` を追加し `fw new --example` で取得可能にする） | `examples/vercel-ssr` は `examples/ssr-routing` と同様に独立 workspace とし、`fw new --example` への同梱（`crates/cli/embedded-examples/` の複製ドリフト検知）を行う |
| #3290（feat: `examples/vercel-ssg`〔`generate_pages` → Build Output API → `--prebuilt`〕を追加する） | 案 c が既定の推奨方式。Vercel 側に Rust ツールチェーンは不要 |
| #3291（docs: デプロイガイドに Vercel の節を追加する） | SSG（既定、案 c）と SSR（2.x・beta 前提、案 a）の選び方を書く。Deployment Protection（既定 SSO 有効、302 リダイレクト）・fail-closed の Basic 認証は本 issue の範囲として詳述する（本決定記録では前提事実の引用のみ） |
| #3292（ci: Build Output API 出力構造のスモークテストを追加する） | 案 c の出力（`.vercel/output` ディレクトリ構造、`config.json` の `version` フィールド等）を対象とする |

## 7. 再評価トリガー

- Vercel Rust ランタイム（`vercel_runtime`/`@vercel/rust`）の GA 化、または
  破壊的変更（`vercel_runtime` 3.x 以降の非 alpha リリース等）
- [vercel/vercel#14532](https://github.com/vercel/vercel/issues/14532) の
  クローズ、または fandhe-frontend 構成での再現の有無が判明したとき
- `vercel_runtime 2.x` の依存木が REQ-3 上限を超える、`build.rs` を持つ
  依存が見つかる、cargo-deny の advisories に違反する、のいずれかが判明
  したとき（→ 案 a の併用を取り下げて案 c 単独にする）
- Vercel Container Images（Beta）が GA 化したとき、または
  `fandhe-frontend-dist-server` が `$PORT`/`SIGTERM` 対応を実装したとき
  （→ 案 d を Vercel 上の方式として再評価する）
- Vercel の Functions 内部プロトコル（`VERCEL_IPC_PATH` 等）が公式に文書化
  されたとき（→ 案 b を再評価する）

## 8. セキュリティ考慮事項（OWASP Top 10 観点）

- **A03 インジェクション / XSS**: 採用する 2 方式（案 c・併用の案 a）は
  いずれも、レンダリングを `ssg::generate_pages` または `ssr::respond` /
  `respond_with`（いずれも既定エスケープ経由）に限定します。Vercel
  ハンドラで `format!` による HTML 文字列の直接組み立てや `raw_html()` の
  不当な使用をしないことを、本決定記録の不変条件として #3288 の完了条件へ
  引き継ぎます。
- **REQ-2（`forbid(unsafe_code)`）**: 本決定は `crates/core`/`crates/interactive`
  に影響しません。`vercel_runtime` を example に閉じるため、公開クレートの
  `unsafe` 境界（`docs/policy/unsafe-boundary.md`）も変わりません。
- **A06 脆弱・古いコンポーネント / サプライチェーン**: 2.x は `=2.4.1` の
  完全一致で固定し、依存木の件数・深さの計測、`build.rs` の有無の確認、
  example ローカルの `deny.toml` による advisories 確認を Phase 2（#3288）
  の前提条件とします。beta の上流の変化を再評価トリガー（§7）として監視
  します。案 b の不採用理由として、文書化されていない内部プロトコルを
  自前実装する脅威面の拡大も挙げます。
- **A05 セキュリティ設定ミス / A01 アクセス制御**: 新規 Vercel プロジェクト
  では Deployment Protection が既定で有効であること（§2）、SSG に Basic
  認証をかける場合は環境変数で管理し未設定なら拒否する（fail-closed）こと
  は #3291 の範囲として参照し、本文書では重複記述しません。
- **A02 / 機微情報の露出**: 本決定記録には環境変数の名前だけを載せ、値・
  プロジェクト ID・デプロイ URL のトークン類は一切載せていません（#3284・
  #3285 の方針を踏襲）。404 やエラーの本文に内部情報を含めない既存方針
  （`ssr::respond`/`respond_with` の固定文言、`crates/server/src/ssr.rs` の
  セキュリティ不変条件節）を、#3288 で引き継ぐべき不変条件として明記し
  ます。
- **A10 SSRF / パストラバーサル**: 案 c の静的配信は Build Output API の
  `static/` に任せ、アダプタではリクエストパスからファイルシステムの
  パスを組み立てない方針とします。

## 9. スコープ外

- upstream（`vercel-community/rust`・`vercel/vercel`・
  `aws/aws-lambda-rust-runtime`）への報告は #3285 で不要と判定済みであり、
  本文書ではこの判断を変更しません。
- Vercel 以外のサーバーレス基盤（Cloudflare Workers・AWS Lambda 直接利用
  等）は本決定記録の対象外です。
- Phase 2（#3288〜#3292）の実装自体は本文書に含みません。
- `vercel_runtime 2.4.1` の依存木計測・`build.rs` 確認・advisories 確認は
  #3288 の完了条件として引き継ぎ、本文書では実施していません。
- #3288 の issue タイトル（`route_request`）自体の修正は本イシューの
  作業範囲外です（§6 の引き継ぎ表に「要見直し」として記録するに留めます）。

## 10. 参照

- 親ツリー: #3282（feat(global): Vercel デプロイ対応）
- Phase 1: #3283（feat(phase-1): Vercel SSR 不成立の原因特定と方式決定）
- 本 Phase 内: #3284（chore: panic 再現・環境変数採取）・#3285（chore: 新版・
  git 版調査）・#3286（本イシュー）
- Phase 2: #3287（feat(phase-2): Vercel 向けアダプタ・example・ドキュメント
  整備）配下の #3288〜#3292
- レポート: `docs/reports/vercel-runtime-panic-repro-3284.md`（PR #3311）・
  `docs/reports/vercel-runtime-version-survey-3285.md`（PR #3312）
- 外部ドキュメント（いずれも 2026-09-27 取得）:
  - Rust ランタイム（Beta）: <https://vercel.com/docs/functions/runtimes/rust>
    （`last_updated: 2025-12-08`）
  - Container Images（Beta）: <https://vercel.com/docs/functions/container-images>
    （`last_updated: 2026-07-07`）
  - Build Output API 概要: <https://vercel.com/docs/build-output-api>
    （`last_updated: 2026-08-11`。`config.json` の `version` フィールドの
    詳細仕様は本文書では未確認、#3292 で確認する）
  - Docker で Rust を Vercel へデプロイするガイド（Container Images 経由、
    任意コンテナの汎用実行ではなく Vercel Functions の一形態であることを
    確認）: <https://vercel.com/kb/guide/deploy-rust-on-vercel-with-docker>
  - upstream 既知課題: [vercel/vercel#14532](https://github.com/vercel/vercel/issues/14532)

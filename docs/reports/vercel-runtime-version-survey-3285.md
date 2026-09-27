# vercel_runtime 新版・git 版での解消状況と upstream 既知 issue の調査（イシュー #3285）

## 1. 概要と結論

- 前提となる #3284 の結論: `vercel_runtime`（crates.io `1.1.6`）は依存クレート
  `lambda_runtime 0.14.4` の `Config::from_env()` が `AWS_LAMBDA_FUNCTION_NAME` 等の
  AWS Lambda 由来の環境変数を `expect` で必須にしており、これらを一切持たない
  Vercel の Rust Function 実行環境では起動直後に panic する。
- 本調査の結論は次の 3 点である。
  1. **`vercel_runtime` 2.x（crates.io 最新 `2.4.1`）で本 panic は構造的に解消する。**
     2.x 系は `lambda_runtime`・`lambda_http` に一切依存しない独立実装であり、
     ローカル実行および Vercel 本番デプロイの両方で、`AWS_LAMBDA_*` 系環境変数が
     皆無のまま HTTP 200 を返すことを実測で確認した。
  2. **`lambda_runtime` の git 版・最新版（`1.4.0`）でも本 panic は解消しない。**
     `Config::from_env()` は 0.14.4 から変わらず同じ 3 変数を `expect` しており、
     加えて `lambda_runtime_api_client` が `AWS_LAMBDA_RUNTIME_API`（AWS Lambda
     Runtime API のエンドポイント）も別途必須にしている。Vercel の実行環境は
     AWS Lambda Runtime API のポーリングプロトコル自体を提供していないため、
     たとえ 3 変数をダミー値で満たしても後続で行き詰まる。**upstream
     `lambda_runtime` 側での「修正」を待つ問題ではない**（Vercel は AWS Lambda
     ではないため、この API 群を提供する主体がそもそも存在しない）。
  3. **upstream に本件専用の既知 issue・PR は見つからなかった。** 理由は明確で、
     `vercel_runtime` 1.x を配布する `vercel-community/rust` リポジトリの
     README に **正式な Deprecation Notice** があり、「legacy な 1.x は非推奨、
     新規 issue は `vercel/vercel` へ」「新ランタイムは 2.x 以降」と明記されて
     いる。つまり本事象は upstream 側で既に「1.x は使わず 2.x へ移行してください」
     という形で解消済みの既知の移行であり、**upstream への新規報告は不要**と
     判断する。
- fandhe-frontend 側の対応（2.x への移行方式、examples/Dockerfile 等への反映）は
  本イシューの範囲外であり、#3286 に引き継ぐ。

## 2. 検証環境

| 項目 | 値 |
|---|---|
| 検証日 | 2026-09-27 |
| ローカル OS/arch | macOS (Darwin 27.0.0) / arm64 |
| Vercel CLI（ローカル） | 54.7.1 (Node.js 26.0.0) |
| Vercel CLI（ビルド内、ビルドログ実測） | 59.25.4 |
| Vercel ビルドリージョン | iad1（Washington, D.C., USA (East)） |
| 使い捨て Vercel プロジェクト名 | `fandhe-survey-3285-cellb-20260927`（検証後に `vercel project remove` で削除済み） |
| crates.io API 取得日時 | 2026-09-27（`updated_at` は各節に記載） |

検証用の cargo プロジェクトはすべてリポジトリの外（一時ディレクトリ）に置き、
`fandhe-frontend` の workspace・`Cargo.lock`・`deny.toml`・`fw gate` の対象には
一切含めていない。

## 3. 調査対象の版と取得日（crates.io・git SHA）

### 3.1 crates.io（2026-09-27 時点、`crates.io/api/v1/crates/*` で実測）

| クレート | 最新非 yank 版 | 状態 | repository |
|---|---|---|---|
| `vercel_runtime` | `2.4.1`（`updated_at` 2026-09-22） | `5.0.0-alpha.1` は yanked（`yank_message: null`） | `https://github.com/vercel/vercel` |
| `lambda_runtime` | `1.4.0`（`updated_at` 2026-09-02） | yanked ではない | `https://github.com/aws/aws-lambda-rust-runtime` |

`vercel_runtime` の repository フィールドは `1.1.6` 当時の
`vercel-community/rust` から `vercel/vercel` へ移っている（#3284 でも既述）。

### 3.2 git（GitHub API で実測した最新コミット・ファイル内容）

| 対象 | 状態 | SHA / 版 | 備考 |
|---|---|---|---|
| `vercel/vercel`（main） | active | `c628be7835e03a965b93e9cf9e2bd5ac2acbf5eb`（2026-09-08） | `crates/vercel_runtime/Cargo.toml` は `version = "2.4.0"`。crates.io 公開の `2.4.1` より 1 パッチ古い（モノレポの publish タイミングのずれと見られ、依存の構造〔`lambda_runtime` 非依存〕は同一） |
| `vercel-community/rust`（main） | **archived**（`pushed_at` 2026-01-22） | `dd0ba46aed5a6baeb5b65bbd915dde0408c42665`（2025-12-08） | `crates/vercel_runtime/Cargo.toml` は `version = "1.1.6"`、`lambda_runtime = "0.14.2"` 依存。**crates.io 公開の 1.1.6 と完全に同一内容**のため、別途ビルド・デプロイを行わずローカル静的確認のみで十分と判断した（下記 §8 のとおり README に明示的な Deprecation Notice あり） |

### 3.3 ビルダー（npm パッケージ）

| パッケージ | npm 最新版 | 備考 |
|---|---|---|
| `@vercel/rust`（公式、`vercel/vercel` の `packages/rust`） | `12.0.1`（`npm view` 実測） | リポジトリ main の `package.json` は `8.0.1` 表記だが、npm 公開は `12.0.1` が最新（モノレポの publish フローとリポジトリ表示のタイムラグと見られる。挙動の検証は npm 公開版・実デプロイで行った） |
| `vercel-rust`（コミュニティ、`vercel-community/rust`） | `4.0.11` | #3284 で使用したものと同一。当該リポジトリは archived で新規更新なし |

`@vercel/rust` はソースの CHANGELOG（3.0.1 時点の記述）に「Rust framework の検出は
`src/main.rs` をキーにしており、`api/**/*.rs` 形式の function プロジェクト
（ハンドラごとに `[[bin]]` を宣言する構成）は framework レス扱いのまま
zero-config でビルドされる」とあり、実際に `packages/rust/test/fixtures/`
配下の全 fixture に `vercel.json` が存在しない（`runtime` フィールドの明示指定が
不要）ことを確認した。1.x 時代の `vercel.json` の
`functions.*.runtime: "vercel-rust@x.y.z"` 指定は 2.x では不要である。

## 4. 各セルの最小コード（差分中心）

### 4.1 セル B（`vercel_runtime = "=2.4.1"`、公式ビルダー、zero-config）

`Cargo.toml`（`[workspace]` を空で宣言、`vercel.json` は置かない）:

```toml
[workspace]

[package]
name = "vercel-runtime-survey-cell-b"
version = "0.1.0"
edition = "2021"
publish = false

[[bin]]
name = "plain"
path = "api/plain.rs"

[dependencies]
vercel_runtime = "=2.4.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

`api/plain.rs`（値は記録せず環境変数名のみ `ENVNAME` 接頭辞でログ出力する点は
#3284 と同一方針）:

```rust
use vercel_runtime::{run, service_fn, Error, Request, Response};

async fn handler(_req: Request) -> Result<Response<String>, Error> {
    let mut names: Vec<String> = std::env::vars_os()
        .filter_map(|(k, _)| k.into_string().ok())
        .collect();
    names.sort();
    for name in &names {
        eprintln!("ENVNAME {name}");
    }
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("cell-b: hello from vercel_runtime 2.4.1".to_string())?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
```

1.x の `run(handler)` に対し、2.x の API は `run(service_fn(handler))` へ変わって
いる（`vercel/vercel` 側 fixture `06-crate-api-plain/api/hello.rs` で確認した
シグネチャに合わせた）。

### 4.2 セル G（`lambda_runtime = "=1.4.0"` 単体の静的確認）

`cargo fetch` のみを行うプレースホルダ crate（ビルド不要、`src/lib.rs` は空の
ドキュメンテーションコメントのみ）で `~/.cargo/registry/src/.../lambda_runtime-1.4.0/`
を取得し、ソースを直接確認した（§7 参照）。

## 5. 手順

1. crates.io API（`https://crates.io/api/v1/crates/<name>` 系、User-Agent 指定）で
   版一覧・yanked 状態・依存・repository を取得した。
2. `gh api repos/vercel/vercel/...`・`gh api repos/vercel-community/rust/...` で
   デフォルトブランチの最新コミット、`Cargo.toml`、`README.md`、
   `packages/rust/` 配下のビルダー実装・テスト fixture・CHANGELOG を確認した。
3. `gh search issues`（`--include-prs`、3 リポジトリ横断）で既知 issue・PR を
   検索した（§8）。
4. セル B・G はリポジトリ外の一時ディレクトリ（`mktemp` 相当の scratchpad 配下）
   で `cargo build --release --manifest-path <abs> --target-dir <abs>` /
   `cargo fetch --manifest-path <abs>` を実行し、`cargo tree -i lambda_runtime`・
   `find <crate-dir> -maxdepth 1 -name build.rs`・ローカル起動結果を記録した。
5. セル B のみ `vercel link --yes --project fandhe-survey-3285-cellb-20260927`
   → `vercel deploy --prod --yes`（Vercel 側ビルド、`--prebuilt` 不使用）→
   `curl` → `vercel logs --expand --limit 200` → `vercel project remove` の順で
   実 Vercel 環境の検証を行った。
6. セル E（`vercel-community/rust` main）はリポジトリ内容が crates.io 公開の
   `1.1.6` と完全一致することを `Cargo.toml` の版・依存指定の突合で確認し、
   別途ビルド・デプロイは行わなかった（README の明示的な Deprecation Notice も
   踏まえ、費用対効果の観点から静的確認で打ち切った）。
7. セル A・C・D・F は本調査では実デプロイを行わなかった（§6 のとおり、セル B の
   実測とクレート依存構造の静的解析だけで受入基準を満たす結論に達したため。
   理由は各セルの表に記載）。

## 6. 結果表（マトリクス）

| # | vercel_runtime | ビルダー | ローカルビルド | Vercel ビルド | HTTP | panic / エラー | `cargo tree -i lambda_runtime` |
|---|---|---|---|---|---|---|---|
| A | 1.1.6（crates.io） | `vercel-rust@4.0.11` | 成功（#3284 実測） | 成功 | 500（`FUNCTION_INVOCATION_FAILED`） | `Missing AWS_LAMBDA_FUNCTION_NAME env var`（#3284 実測を引用） | `lambda_runtime v0.14.4` あり |
| B | **2.4.1（crates.io）** | `@vercel/rust`（npm 12.0.1 系、zero-config） | **成功**（実測） | **成功**（実測） | **200**（実測） | なし。ローカルは `Dev server listening: 3000` で待受、Vercel でも同一 43 個の環境変数名（`AWS_LAMBDA_*` なし）で正常応答 | **該当パッケージなし**（`error: package ID specification lambda_runtime did not match any packages`）＝依存グラフに存在しない |
| C | 2.4.1 | `vercel-rust@4.0.11` | 未検証（B で解消を確認済みのため、旧コミュニティビルダーとの組み合わせを別途検証する実益が薄いと判断し打ち切り） | 未検証（同上） | 未検証 | 未検証 | 未検証 |
| D | git `vercel/vercel` main（`c628be78`、`Cargo.toml` は `version = "2.4.0"`） | `@vercel/rust`（npm 12.0.1 系） | 未実施（Cargo.toml 突合で B の crates.io `2.4.1` と依存構造が同一〔`lambda_runtime` 非依存〕であることを確認済みのため、追加のビルドを打ち切り） | 未検証 | 未検証 | 未検証 | 未検証（静的確認では B と同型） |
| E | git `vercel-community/rust` main（`dd0ba46a`、`Cargo.toml` は `version = "1.1.6"`） | `vercel-rust@4.0.11` | 未実施（`Cargo.toml`・依存指定〔`lambda_runtime = "0.14.2"`〕が crates.io 公開の 1.1.6 と完全一致することを確認済み。README に Deprecation Notice あり） | 未検証 | 未検証 | A と同一と推定（静的確認） | 静的確認では A と同型 |
| F | 1.1.6 | `@vercel/rust`（npm 12.0.1 系） | 未検証 | 未検証（30 分の打ち切り基準内で優先度を B に割いたため） | 未検証 | 未検証 | 未検証 |
| G | `lambda_runtime = "=1.4.0"`（単体） | デプロイなし | `cargo fetch` 成功 | 該当なし | 該当なし | 該当なし（静的確認のみ） | 該当（対象そのもの） |
| - | `vercel_runtime 5.0.0-alpha.1` | - | - | - | - | - | crates.io で **yanked** のため対象外 |

セル C・D・F を打ち切った理由の補足: セル B（新クレート + 新ビルダー、zero-config）
で「2.x なら解消する」という受入基準の核心が実測で確定し、かつ §7 の静的解析で
「1.x はどのビルダーの世代であっても Lambda Runtime API プロトコル自体が
Vercel に存在しないため原理的に解消しない」ことが裏付けられたため、
「ビルダー世代 × クレート版」の残り組み合わせを追加でデプロイする実益は
乏しいと判断した（推測ではなく、後述 §7 の依存関係の必須要件からの論理的
帰結として記録する）。この打ち切り判断自体を検証可能にするため、未検証セルは
「未検証」と明記し推測結果を断定として書かない。

## 7. `lambda_runtime` の静的確認

`lambda_runtime-1.4.0/src/lib.rs`（`cargo fetch` で取得したソースを直接確認）:

```text
84:    pub fn from_env() -> Self {
86:            function_name: env::var("AWS_LAMBDA_FUNCTION_NAME").expect("Missing AWS_LAMBDA_FUNCTION_NAME env var"),
87:            memory: env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE")
88:                .expect("Missing AWS_LAMBDA_FUNCTION_MEMORY_SIZE env var")
91:            version: env::var("AWS_LAMBDA_FUNCTION_VERSION").expect("Missing AWS_LAMBDA_FUNCTION_VERSION env var"),
```

0.14.4（#3284 で確認した `lib.rs:70/72/75`）と全く同じ 3 変数を `expect` で
必須にしている。行番号がずれているのはバージョン間の実装変更によるもので、
必須変数の集合・挙動（`expect` による panic）自体は変わっていない。

加えて、`run()`/`run_concurrent()` の rustdoc（139〜141 行目・193〜195 行目）は
「`AWS_LAMBDA_RUNTIME_API` も必須」と明記しており、実体は
`lambda_runtime_api_client-1.1.1/src/lib.rs:298` の
`std::env::var("AWS_LAMBDA_RUNTIME_API").expect("Missing AWS_LAMBDA_RUNTIME_API env var")`
にある。この変数は AWS Lambda Runtime API（イベントのポーリング用エンドポイント）
の接続先を指しており、#3284 で採取した Vercel 実行環境の環境変数一覧
（43 個）にも、本調査のセル B 実測（同一の 43 個）にも一切含まれていない。

つまり `Config::from_env()` の 3 変数をダミー値で満たしたとしても、
`lambda_runtime_api_client` の初期化で次の `expect` に必ず突き当たる。
Vercel は AWS Lambda Runtime API のポーリングエンドポイントを提供する
主体ではないため、**この必須要件は「upstream が直せば解消する」性質の
不具合ではなく、「AWS Lambda 専用ランタイムを AWS Lambda ではない実行環境で
動かそうとしている」という設計上のミスマッチ**である。

## 8. upstream の既知 issue・PR（またはヒットなし）

`gh search issues --include-prs`（`vercel-community/rust`・`vercel/vercel`・
`awslabs/aws-lambda-rust-runtime` の 3 リポジトリ、キーワード:
`AWS_LAMBDA_FUNCTION_NAME`・`Config::from_env`・`vercel_runtime panic`・
`Missing AWS_LAMBDA`・`FUNCTION_INVOCATION_FAILED rust`・`rust runtime v2`、
2026-09-27 検索）。

- `AWS_LAMBDA_FUNCTION_NAME`・`Config::from_env`・`vercel_runtime panic`・
  `Missing AWS_LAMBDA`・`FUNCTION_INVOCATION_FAILED rust` の 5 キーワードでは、
  上記 3 リポジトリに対する**本件と直接一致する issue・PR は該当なし**
  （`vercel_runtime panic` のヒットは本リポジトリ自身の #3284/#3311 のみ）。
- `rust runtime v2`（`vercel/vercel` 限定）でヒットしたもの:
  - [vercel/vercel#14532](https://github.com/vercel/vercel/issues/14532)
    （open）「Rust runtime v2 fails with "address already in use"」
  - [vercel/vercel#15067](https://github.com/vercel/vercel/issues/15067)
    （closed）同名の issue
  - いずれも 2.x（v2）ランタイムのローカル/ビルド時のポート競合に関する
    既知課題で、本調査の panic（AWS Lambda 環境変数欠落）とは別種の不具合。
    2.x を採用する場合の留意点として §10 へ引き継ぐ。
- 上記以外に無関係なノイズ（他リポジトリの X-Ray トレース欠落等）が多数
  ヒットしたため、本件と無関係なものは除外した。

**「該当なし」の根拠**: `vercel-community/rust` の README に明示的な
Deprecation Notice（§9 参照）があり、「legacy な 1.x の新規 issue は
`vercel/vercel` へ」と誘導されているにもかかわらず、`vercel/vercel` 側にも
本件（AWS Lambda 環境変数欠落による 1.x の起動時 panic）に一致する issue が
見つからない。これは「1.x はそもそも新規に使うべきではない（2.x へ移行
すべき）」という位置づけが upstream 側で既に確立しており、個別の panic
報告自体が発生していないためと解釈するのが妥当である。

## 9. upstream への報告要否と理由

**結論: 新規報告は不要。**

判断ルール（計画時点で定めたもの）に従うと、セル B（2.4.1 + 新ビルダー）が
200 を返した時点で「1.x と `vercel-community/rust` は upstream 側で置き換え済み
（archived）の扱いになる。本 panic は旧世代の組み合わせでしか起きない既知
または想定内の挙動とみなし、upstream への新規報告は不要」と判定する。

本調査で追加確認できた根拠:

- `vercel-community/rust` の README に upstream 自身による明示的な
  Deprecation Notice がある（引用、非命令的な事実記述として）:
  「This community runtime is now deprecated in favor of the official
  Vercel Rust runtime implementation. For new issues please open them in
  the Vercel CLI repository.」「The new runtime crate is released with
  2.x onwards.」
- `vercel_runtime` の `repository` フィールド自体が `vercel-community/rust`
  から `vercel/vercel` へ切り替わっており、crates.io 上でも 1.x 系列は
  過去の実装として扱われている。
- 2.x は `lambda_runtime`/`lambda_http` に一切依存しない独立実装であり、
  「AWS Lambda 前提の依存を Vercel で使う」という 1.x の設計上の問題が
  構造的に存在しない（§7 の分析どおり、これは "バグ修正" ではなく
  "アーキテクチャの刷新" によって回避されている）。
- したがって、1.x の起動時 panic は「upstream が今後直すべき未修正のバグ」
  ではなく、「1.x という非推奨系列を使い続けた場合にのみ発現する、
  移行によってのみ解消する既知の制約」である。

### F（`AWS_LAMBDA_*` 欠落の原因はビルダーかクレートか）についての判断

セル F の実デプロイは打ち切ったが、次の 2 点から
「**Vercel の実行環境自体が AWS Lambda 互換の環境変数を提供していない**
（ビルダーの世代に依存しない環境側の性質であり、クレート側の不具合でもない）」
と**推定**する（実測ではなく、複数の間接証拠からの推定である旨を明記する）。

- セル B（新クレート + 新ビルダー、実測）とセル A（旧クレート + 旧ビルダー、
  #3284 実測）で、Function ログから採取した環境変数名の集合が**完全に同一**
  （43 個、`AWS_LAMBDA_*` 系はいずれも皆無）だった。ビルダー世代が変わっても
  環境変数の集合が変わっていないことから、「ビルダーが環境変数を注入・
  除去している」可能性は低い。
- Vercel の実行基盤は AWS Lambda 互換レイヤーではなく独自の Fluid Compute /
  サンドボックス実装であり、`VERCEL_*`・`AWS_REGION`・`AWS_DEFAULT_REGION`
  程度の最小限の互換情報のみを露出する設計と見られる（`AWS_LAMBDA_RUNTIME_API`
  も含め Lambda Runtime API 系は一貫して欠落している）。

## 10. #3286 への引き継ぎ

- `vercel_runtime` を使う場合は **crates.io `2.x`（`2.4.1` 以降）を選定し、
  `lambda_runtime`/`lambda_http` 系の 1.x API（`vercel_runtime::run(handler)`
  形式）ではなく `run(service_fn(handler))` 形式へ書き換える**移行が前提になる。
- 2.x はハンドラを `[[bin]]` ごとに `api/**/*.rs` へ配置する zero-config
  構成が想定されており、`vercel.json` の `functions.*.runtime` 明示指定は
  不要（`@vercel/rust` が `src/main.rs`／`api/**/*.rs` パターンで自動検出する）。
- 2.x を採用する場合の既知の留意点として、[vercel/vercel#14532](https://github.com/vercel/vercel/issues/14532)
  （Rust runtime v2 の「address already in use」、2026-09-27 時点で open）を
  事前に確認しておくこと。本調査の panic とは別種の問題であり、
  fandhe-frontend の構成でも再現するかは #3286 側の検証課題とする。
- `vercel_runtime` の `axum`/`actix` feature（2.x で追加された optional
  依存）を使うかどうかは、fandhe-frontend 側のハンドラ実装方針（#3286 の
  対応方式）に依存するため、本イシューでは選定しない。
- 依存グラフ・`build.rs` の有無は 2.x 側では未確認（本調査は `vercel_runtime`
  単体の依存で `lambda_runtime` が消えたことのみを確認した）。実際に
  fandhe-frontend の構成へ組み込む際は、REQ-3（依存 60 件/深さ 6 以内）・
  REQ-4（cargo-deny）の観点で改めて `cargo metadata`/`cargo tree` を取り直す
  必要がある。

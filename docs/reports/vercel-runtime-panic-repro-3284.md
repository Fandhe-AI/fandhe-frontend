# vercel_runtime 1.1.6 起動時 panic の最小再現レポート（イシュー #3284）

## 1. 概要と結論

`vercel_runtime`（crates.io `1.1.6`）を Vercel の Rust Function 実行環境で使うと、
リクエストの中身に関わらず全呼び出しが HTTP 500（`FUNCTION_INVOCATION_FAILED`）に
なる。原因は fandhe-frontend 側ではなく、依存クレート `lambda_runtime 0.14.4` の
`Config::from_env()` が `AWS_LAMBDA_FUNCTION_NAME` 環境変数を `expect` で必須とし、
Vercel の Rust Function 実行環境にはこの変数（および `AWS_LAMBDA_FUNCTION_MEMORY_SIZE`・
`AWS_LAMBDA_FUNCTION_VERSION`）が存在しないために panic することにある。

`fandhe-frontend-core` に依存しない素のハンドラ（`plain`）と、
`fandhe-frontend-core::render()` を使うハンドラ（`fandhe`）の両方で同一の panic を
確認しており、fandhe-frontend 側の実装は関与しない。

## 2. 検証環境

| 項目 | 値 |
|---|---|
| 検証日 | 2026-09-27 |
| ローカル OS/arch | macOS (Darwin 27.0.0) / arm64 |
| ローカル rustc | 1.98.1 (48a229cea 2026-09-01) |
| Vercel CLI | 54.7.1 (Node.js 26.0.0) |
| Vercel ビルドリージョン | iad1（Washington, D.C., USA (East)） |
| Vercel ビルド内で使われた Vercel CLI | 59.25.4 |
| Vercel Function ランタイム（builder） | `vercel-rust@4.0.11`（npm パッケージ `vercel-rust`。`vercel.json` の `functions.*.runtime` に指定した識別子で、ビルドログの `Installing Builder: vercel-rust@4.0.11` 行で実測確認） |
| 使い捨て Vercel プロジェクト名 | `fandhe-repro-3284-20260927`（検証後に `vercel project remove` で削除済み） |

`vercel-rust@4.0.11` はコミュニティ製ビルダー（[vercel-community/rust](https://github.com/vercel-community/rust)）で、
Vercel ネイティブの一次サポート Rust ランタイムではない。`vercel_runtime` クレート自体も
同じ組織が公開している。

## 3. 依存の解決版

一時ディレクトリの独立 cargo プロジェクト（`[workspace]` を空で宣言し、
`fandhe-frontend` の workspace には属さない）で `cargo build --release` した際の
`Cargo.lock` の解決版。

| クレート | 解決版 |
|---|---|
| `vercel_runtime` | 1.1.6（`=1.1.6` 固定指定） |
| `vercel_runtime_router` | 1.1.7 |
| `vercel_runtime_macro` | 1.1.7 |
| `lambda_runtime` | 0.14.4（`vercel_runtime` の依存指定は `lambda_runtime = "^0.14.2"`） |
| `lambda_runtime_api_client` | 0.12.4 |
| `lambda_http` | 0.15.1 |
| `aws_lambda_events` | 0.16.1 |
| `fandhe-frontend-core` | 0.4.3（`=0.4.3` 固定指定） |

`cargo tree -i lambda_runtime` の結果:

```text
lambda_runtime v0.14.4
├── lambda_http v0.15.1
│   └── vercel_runtime v1.1.6
│       └── vercel-runtime-panic-repro v0.1.0 (...)
└── vercel_runtime v1.1.6 (*)
```

`vercel_runtime`・`lambda_runtime`・`lambda_runtime_api_client`・`lambda_http` の
いずれのクレートディレクトリ直下にも `build.rs` は存在しない（ローカル cargo
registry キャッシュを確認、`find <crate-dir> -maxdepth 1 -name build.rs` が
全クレートで空）。

## 4. 最小再現コード全文

一時ディレクトリに作成した、どの workspace にも属さない独立 cargo プロジェクト。
`fandhe-frontend` リポジトリ本体には追加していない（root workspace・`fw gate`・
cargo-deny・`version-bump-guard` の対象へ持ち込まないため）。

### `Cargo.toml`

```toml
[workspace]

[package]
name = "vercel-runtime-panic-repro"
version = "0.1.0"
edition = "2021"
publish = false

[[bin]]
name = "plain"
path = "api/plain.rs"

[[bin]]
name = "fandhe"
path = "api/fandhe.rs"

[dependencies]
vercel_runtime = "=1.1.6"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde_json = "1"
fandhe-frontend-core = "=0.4.3"
```

### `vercel.json`

```json
{
  "functions": {
    "api/**/*.rs": {
      "runtime": "vercel-rust@4.0.11"
    }
  }
}
```

### `api/plain.rs`（fandhe-frontend 非依存版）

```rust
//! イシュー #3284 の最小再現バイナリ（fandhe-frontend 非依存版）。
//!
//! 目的: vercel_runtime 1.1.6 が Vercel の Rust Function 実行環境で
//! 起動時 panic するかどうかを、fandhe-frontend-core への依存を含まない
//! 素の vercel_runtime hello world で切り分ける。panic が本バイナリでも
//! 再現すれば、原因が fandhe-frontend 側ではなく依存（vercel_runtime /
//! lambda_runtime）側にあることの裏付けになる。
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // `lambda_runtime::run` (vercel_runtime::run 内部で呼ばれる) は
    // `Config::from_env()` で AWS_LAMBDA_* 環境変数を読む。その expect
    // による panic より前に、値を一切含めず環境変数名だけを記録する。
    // `ENVNAME ` 接頭辞は Vercel Function ログからの機械的な抽出用。
    let mut names: Vec<String> = std::env::vars_os()
        .filter_map(|(k, _)| k.into_string().ok())
        .collect();
    names.sort();
    for name in &names {
        eprintln!("ENVNAME {name}");
    }

    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body("plain: hello from vercel_runtime 1.1.6".into())?)
}
```

### `api/fandhe.rs`（fandhe-frontend-core 依存版）

```rust
//! イシュー #3284 の最小再現バイナリ（fandhe-frontend-core 依存版）。
//!
//! `plain.rs` と対にして、ハンドラ内で fandhe-frontend-core のノード木 API
//! + `render()` を使った場合でも panic 箇所（lambda_runtime 起動処理）が
//! 変わらないことを確認する。既定エスケープ（REQ-1）を守るため、
//! `raw_html()` や `format!` による HTML 文字列の直接組み立ては使わない。
use fandhe_frontend_core::{el, render, text};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // plain.rs と同一の環境変数名採取処理（値は記録しない）。
    let mut names: Vec<String> = std::env::vars_os()
        .filter_map(|(k, _)| k.into_string().ok())
        .collect();
    names.sort();
    for name in &names {
        eprintln!("ENVNAME {name}");
    }

    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    // ノード木 API で組み立て、render() の既定エスケープ経由で HTML 化する。
    let node = el(
        "p",
        vec![],
        vec![text("fandhe: hello from fandhe-frontend-core")],
    );
    let html = render(&node);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(html.into())?)
}
```

## 5. 再現手順

### ローカル

1. 上記ファイルを一時ディレクトリに配置し `cargo build --release` する（成功する）
2. `./target/release/plain` をそのまま実行すると、起動直後に
   `AWS_LAMBDA_FUNCTION_NAME` 欠落で panic する
3. `AWS_LAMBDA_FUNCTION_NAME=dummy` を与えて再実行すると、次は
   `AWS_LAMBDA_FUNCTION_MEMORY_SIZE` 欠落で panic する
4. さらに `AWS_LAMBDA_FUNCTION_MEMORY_SIZE=128` も与えて再実行すると、
   最後に `AWS_LAMBDA_FUNCTION_VERSION` 欠落で panic する
5. `./target/release/fandhe` でも 2. と全く同じ panic（ファイル・行・メッセージ）
   になることを確認する

### Vercel

1. `vercel whoami` でログイン状態を確認する
2. `vercel link --yes --project <使い捨てプロジェクト名>` で新規プロジェクトへ
   紐付ける（既存プロジェクトへは link しない）
3. `vercel deploy --prod --yes` でビルド・デプロイする（Vercel 側ビルド、
   `--prebuilt` は使わない）
4. `curl <production-url>/api/plain` と `/api/fandhe` を呼び出す
5. `vercel logs <production-url> --expand --limit 200`（`--follow` を付けない
   ため即座に終了する）で Function ログを取得し、`ENVNAME ` 接頭辞の行と
   panic 行を確認する
6. 検証後、`vercel project remove <プロジェクト名>` で使い捨てプロジェクトを
   削除する

補足: 今回の検証では Deployment Protection（Vercel Authentication）の
迂回操作は不要だった。本番エイリアス URL（`https://<project>.vercel.app/`）への
直接アクセスで 401/302 を挟まずに 500 が返り、Function ログにも到達できた。

## 6. 結果

| ハンドラ | 実行環境 | HTTP ステータス | panic 位置 | panic メッセージ |
|---|---|---|---|---|
| `plain` | ローカル | （HTTP なし。プロセス起動直後に panic） | `lambda_runtime-0.14.4/src/lib.rs:70:65` | `Missing AWS_LAMBDA_FUNCTION_NAME env var: NotPresent` |
| `plain`（`AWS_LAMBDA_FUNCTION_NAME=dummy` のみ付与） | ローカル | 同上 | `lambda_runtime-0.14.4/src/lib.rs:72:18` | `Missing AWS_LAMBDA_FUNCTION_MEMORY_SIZE env var: NotPresent` |
| `plain`（上記 2 変数をダミー付与） | ローカル | 同上 | `lambda_runtime-0.14.4/src/lib.rs:75:62` | `Missing AWS_LAMBDA_FUNCTION_VERSION env var: NotPresent` |
| `fandhe` | ローカル | 同上 | `lambda_runtime-0.14.4/src/lib.rs:70:65` | `Missing AWS_LAMBDA_FUNCTION_NAME env var: NotPresent`（`plain` と同一） |
| `plain` | Vercel（`GET /api/plain`） | 500（`FUNCTION_INVOCATION_FAILED`） | `/rust/registry/src/index.crates.io-1949cf8c6b5b557f/lambda_runtime-0.14.4/src/lib.rs:70:65` | `Missing AWS_LAMBDA_FUNCTION_NAME env var: NotPresent`（exit status 101） |
| `fandhe` | Vercel（`GET /api/fandhe`） | 500（`FUNCTION_INVOCATION_FAILED`） | 同上 | 同上 |

ローカルと Vercel の両方で、`plain`/`fandhe` の 2 系統とも同一の panic 箇所・
同一のメッセージになった。

## 7. Vercel 実行環境の環境変数名一覧

`vercel logs --expand` で取得した Function ログの `ENVNAME ` 行から抽出した、
Vercel の Rust Function 実行環境に設定されている環境変数の**名前のみ**
（値は一切記録していない）。`plain`/`fandhe` の 2 リクエストで同一の 43 個。

```text
AWS_DEFAULT_REGION
AWS_REGION
LANG
LD_LIBRARY_PATH
LD_PRELOAD
NOW_REGION
NX_DAEMON
PATH
PWD
SHLVL
TURBO_CACHE
TURBO_DOWNLOAD_LOCAL_ENABLED
TURBO_PLATFORM_ENV
TURBO_REMOTE_ONLY
TURBO_RUN_SUMMARY
TZ
VERCEL
VERCEL_CACHE_HANDLER_MEMORY_CACHE
VERCEL_DEPLOYMENT_ID
VERCEL_DEPLOYMENT_KEY
VERCEL_ENV
VERCEL_FLUID
VERCEL_GIT_COMMIT_AUTHOR_LOGIN
VERCEL_GIT_COMMIT_AUTHOR_NAME
VERCEL_GIT_COMMIT_MESSAGE
VERCEL_GIT_COMMIT_REF
VERCEL_GIT_COMMIT_SHA
VERCEL_GIT_PREVIOUS_SHA
VERCEL_GIT_PROVIDER
VERCEL_GIT_PULL_REQUEST_ID
VERCEL_GIT_REPO_ID
VERCEL_GIT_REPO_OWNER
VERCEL_GIT_REPO_SLUG
VERCEL_HANDLER
VERCEL_IPC_PATH
VERCEL_PARENT_SPAN_ID
VERCEL_PROJECT_ID
VERCEL_PROJECT_NAME
VERCEL_PROJECT_PRODUCTION_URL
VERCEL_REGION
VERCEL_TARGET_ENV
VERCEL_URL
VERCEL_VDC_REMOTE_CACHE_ENABLED
```

### `Config::from_env` の必須 3 変数との照合表

| `lambda_runtime::Config::from_env` が要求する変数 | Vercel 実行環境に存在するか |
|---|---|
| `AWS_LAMBDA_FUNCTION_NAME` | なし |
| `AWS_LAMBDA_FUNCTION_MEMORY_SIZE` | なし |
| `AWS_LAMBDA_FUNCTION_VERSION` | なし |

`AWS_*` で始まる変数は `AWS_DEFAULT_REGION`・`AWS_REGION` の 2 つのみで、
`AWS_LAMBDA_*` 名の変数は（`expect` で必須の 3 変数だけでなく、
`AWS_LAMBDA_LOG_STREAM_NAME`・`AWS_LAMBDA_LOG_GROUP_NAME`〔`unwrap_or_default()`
で必須ではない〕も含め）一切存在しない。

## 8. 原因の切り分け

- fandhe-frontend-core に依存しない `plain` バイナリでも、`fandhe` バイナリと
  完全に同一の panic（ファイル・行・メッセージ）になる
- panic は `lambda_runtime::Config::from_env()`（`lambda_runtime-0.14.4/src/lib.rs`
  70 行目付近）の `expect` で起きており、呼び出し経路は
  `vercel_runtime::run()`（`vercel_runtime-1.1.6/src/lib.rs` 28 行目、
  `lambda_runtime::run(handler).await`）→ `lambda_runtime::run` →
  `Config::from_env()` という静的な呼び出し関係で確認できる
- `fandhe-frontend-core::render()` はハンドラ本体（`handler` 関数）の中でのみ
  呼ばれ、`main()` の `run(handler).await` が panic するより後にしか到達しない。
  つまり fandhe-frontend-core は起動時 panic の経路に一切関与しない
- 以上から、原因は依存側（`vercel_runtime 1.1.6` が前提とする AWS Lambda 互換の
  環境変数を、現行の Vercel Rust Function 実行環境〔`vercel-rust@4.0.11`
  ビルダー〕が与えていないこと）にあると結論する
- なお、この不一致がなぜ生じているか（Vercel 側のランタイム世代交代・
  `vercel-rust` ビルダーの実行環境の変更など）は本 issue の検証範囲では
  特定できておらず、**推定に留まる**。原因の特定自体は #3285（新版・git 版
  での解消確認、upstream の既知 issue 調査）に引き継ぐ

## 9. 後続への引き継ぎ

- **#3285**: `vercel_runtime` 2.x や `lambda_runtime` の git 版で本 panic が
  解消するかの確認、upstream（vercel-community/rust、awslabs/aws-lambda-rust-runtime）
  の既知 issue の調査
- **#3286**: 本レポートの事実（`vercel_runtime` 1.1.6 系は現行の Vercel Rust
  Function 実行環境では起動時 panic で使用不能・環境変数の充足による回避は
  Vercel 側の対応が必要で fandhe-frontend 側では制御できない）を踏まえた、
  Vercel 対応方式の比較と ADR

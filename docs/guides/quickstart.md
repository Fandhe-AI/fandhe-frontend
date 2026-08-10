# クイックスタート

本ガイドは、`fandhe-frontend` を初めて使う方が「導入 → `fw new` でプロジェクト
作成 → ビルド → ブラウザで確認」までを最短経路でたどるための入門ガイドです。
手順はすべて実機で検証済みです（コマンドと出力例は実行結果をそのまま転記して
います）。

## 1. 本ガイドで得られるもの

このガイドを上から実行すると、次の 2 種類の「動くプロジェクト」が手元に
できます。

- `fw new --template app` で生成した SSR/SSG プロジェクト（`cargo run` で
  静的 HTML を書き出し、ブラウザで一覧・詳細ページの遷移を確認できます）
- 同じプロジェクトの CSR（WASM）ビルド（同一オリジンで配信した
  `embed.html` を開くと、WASM モジュールがマウントされて動作します）

必要なツールは Rust ツールチェーン（`rustup` / `cargo`）と `git` のみです。
CSR（WASM）ビルドを試す場合は追加で `wasm32-unknown-unknown` ターゲットと
`wasm-bindgen-cli` を使いますが、導入手順は本ガイド内で案内します。

## 2. 前提条件

以下がインストール済みであることを確認してください。

- Rust ツールチェーン（`rustup` 経由が推奨、https://rustup.rs/ ）
- `git`

バージョンを確認します。

```
$ cargo --version
cargo 1.96.0 (30a34c682 2026-05-25)
$ rustup --version
rustup 1.29.0 (28d1352db 2026-03-05)
```

表示されるバージョンはお使いの環境によって異なります。コマンドが見つからない
場合は https://rustup.rs/ の手順に従って導入してください。

## 3. `fw` CLI の導入

`fw` コマンド（`fandhe-frontend-cli`）は crates.io で公開されており、次の
コマンドで導入できます。

```
$ cargo install fandhe-frontend-cli
```

導入できたかどうかは、引数なしで `fw` を実行して確認します。サブコマンド
一覧が表示されれば導入成功です。

```
$ fw
fw: a subcommand is required
Usage: fw <subcommand> [--project <dir>]
Subcommands:
  structure    generate/validate the machine-readable project structure manifest
  gate         run the AI self-maintenance verification gate (type/escape/lint/test/policy)
  impact       analyze the change impact of a symbol (breaking risk, affected crates/routes)
  new          deterministically scaffold a new project from templates/default
```

### 開発版を使う場合

開発版の `fw` を使いたい場合は、リポジトリを clone してソースから導入します。
CLI の導入だけであれば `docs/spec/` サブモジュールの取得は不要です。

```
$ git clone git@github.com:Fandhe-AI/fandhe-frontend.git
$ cd fandhe-frontend
$ cargo install --path crates/cli
```

`cargo install` を使わずに、リポジトリ内から都度実行することもできます
（グローバル環境を汚したくない場合に便利です）。

```
$ cargo run -p fandhe-frontend-cli --bin fw -- <サブコマンド>
```

## 4. プロジェクト作成（`fw new --template app`）

`fw new` は決定的にプロジェクトを生成するコマンドです。テンプレートには
次の 3 種類があります。

| テンプレート | 用途 |
|---|---|
| `default` | `fw new` の既定テンプレート。標準的な cargo プロジェクト構成 |
| `app` | SSR/SSG 出力と CSR（WASM）ビルドの両方を含む拡充テンプレート |
| `embed` | 静的単一ファイル（`embed.html`）のみの最小埋め込み構成（cargo パッケージなし） |

本ガイドでは `app` テンプレートを使い、動くプロジェクトを最短でたどります。

```
$ fw new my-app --template app
{"created":"/path/to/my-app","template":"app","files":["Cargo.toml","src/main.rs", ...(全ファイル一覧)]}
```

コマンドは生成したファイルパスの一覧を JSON で 1 行に出力します（上記は
紙面の都合で省略しています）。生成された `my-app/` の主なファイルは次の
とおりです。

- `src/main.rs`: ページ構築（Loader・束縛点 API・render）の実体サンプル
- `wasm/`: CSR（WASM）ビルド用の独立ワークスペース
- `static/embed.html`: CSR のマウント骨格（後述のビルド後に動作します）
- `structure.toml`: `fw gate` が読む構造マニフェスト（生成直後から PASS する構成です）

## 5. ビルドとブラウザ確認（SSR/SSG 出力）

生成したプロジェクトのディレクトリへ移動します。初回ビルド時は `Cargo.toml` で宣言された fandhe-frontend-core / fandhe-frontend-app を crates.io から取得するため、インターネット接続が必要です。

```
$ cd my-app
$ cargo test
```

テストとビルドを実行します。

```
$ cd my-app
$ cargo test
running 2 tests
test list_page_escapes_xss_payload_in_demo_items ... ok
test detail_page_escapes_xss_payload_in_demo_items ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

このテストは、既定エスケープ（本フレームワークの中核価値）がテンプレートの
サンプルデータでも効いていることを確認する回帰テストです。

続けて `cargo run` を実行すると、`dist/` 配下に SSG 出力（静的 HTML）が
書き出されます。

```
$ cargo run
wrote 5 pages to dist/
```

`dist/index.html` をブラウザで開くと、記事一覧ページが表示されます。一覧の
リンクをクリックすると詳細ページへ遷移します。

## 6. CSR（WASM）ビルドとブラウザ確認

`app` テンプレートは SSR/SSG と同じコンポーネント実装を CSR（WASM）からも
呼び出せます。まず `wasm32-unknown-unknown` ターゲットを追加します。

```
$ rustup target add wasm32-unknown-unknown
```

次に `wasm-bindgen-cli` を導入します。バージョンは `wasm/Cargo.lock` が
解決した `wasm-bindgen` クレートのバージョンと完全一致させる必要があります。
`wasm/Cargo.lock` を確認してから `--locked` 付きで導入してください。

```
$ grep -A1 'name = "wasm-bindgen"' wasm/Cargo.lock | head -2
name = "wasm-bindgen"
version = "0.2.127"
$ cargo install wasm-bindgen-cli --version 0.2.127 --locked
```

バージョンが一致しない状態でビルドスクリプトを実行すると、`tools/wasm/build.sh`
が fail-closed で停止し、是正コマンド（上記と同様の `cargo install` 例）を
標準エラー出力へ表示します。黙って古いバージョンのまま実行されることは
ありません。

準備ができたらビルドスクリプトを実行します。

```
$ ./tools/wasm/build.sh
wasm build complete: /path/to/my-app/static/wasm/fandhe_frontend_wasm_client.js, /path/to/my-app/static/wasm/fandhe_frontend_wasm_client_bg.wasm
```

`static/wasm/` に生成物ができたら、`static/` ディレクトリを HTTP サーバーで
配信します。ES モジュールと WASM は `file://` では動作しないため、必ず
HTTP 経由でアクセスしてください。ローカル確認用サーバーは意図しない公開を
避けるため `127.0.0.1` にバインドします。

```
$ python3 -m http.server 8000 --bind 127.0.0.1 --directory static
```

ブラウザで `http://127.0.0.1:8000/embed.html` を開くと、WASM モジュールが
読み込まれてマウントされます。

## 7. 最小埋め込みテンプレート（`fw new --template embed`）

既存の静的 HTML ページの一部だけをフレームワークの管理下に置きたい場合は、
`embed` テンプレートを使います。cargo パッケージを含まない、静的単一ファイル
構成です。

```
$ fw new my-embed --template embed
{"created":"/path/to/my-embed","template":"embed","files":["embed.html","structure.toml"]}
```

生成される `embed.html` は `app` テンプレートの `static/embed.html` と同一の
マウント骨格です。WASM アセットの用意方法・責務境界の詳細は
[最小埋め込みガイド](./embedding-guide.md) を参照してください。

## 8. 次のステップ

ここまでで、SSR/SSG・CSR の両方を最短経路で体験できました。次は目的に
応じて以下のガイドへ進んでください。

- [コンポーネント作成ガイド](./component-authoring.md): ページ・コンポーネントの実装方法
- [最小埋め込みガイド](./embedding-guide.md): 既存ページへの部分埋め込み
- [ビュー遷移ガイド](./view-transitions.md): ページ遷移アニメーション
- [NPM アセットビルドガイド](./npm-asset-build.md): NPM 互換の静的アセットパイプライン

また、生成したプロジェクトは `fw gate --project .` で型チェック・既定
エスケープ検査・lint・テスト・依存ポリシーを一括検証できます。生成直後は
無編集で PASS する構成になっています。

```
$ fw gate --project .
{"gate_result":"PASS","action":"all checks passed; changes may proceed", ...}
```

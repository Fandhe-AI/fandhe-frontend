# examples/dist-server-docker

## 概要

`fandhe-frontend` フレームワークの単一バイナリ配布 + Docker（`FROM scratch`）
正本サンプルです（イシュー #502）。`examples/ssr-routing`（イシュー #499）に
続く examples 規約の 2 例目で、crates.io へ公開済みの
`fandhe-frontend-dist-server`（v0.2.0、イシュー #1159 で v0.1.0 から追随）を通常の外部依存として使います。

## 学べること

- `fandhe-frontend-dist-server` の `routes::route_request`（HTTP に依存しない
  純粋なルーティング関数）・`assets::active_mode`（開発 / 本番モード判定）を
  外部依存として呼び出す薄い hyper トランスポート層の書き方
- **crates.io からの外部依存利用時、`static/` アセットが `lib` 経由では
  配信されない制約**（実測で確認済み、下記「実測結果」参照）。
  `fandhe-frontend-dist-server` の `assets::lookup`／`build.rs` はいずれも、
  ライブラリ自身の `crates/dist-server/` を基準にした固定パス
  （`CARGO_MANIFEST_DIR` の 2 段上 + `static`）から `static/` を解決するため、
  外部依存として使う本サンプルの `static/`（本ディレクトリ直下）は対象外に
  なります。本サンプルではこの制約への対処として、`src/main.rs` が
  `/static/style.css` への完全一致のみを `include_bytes!` で自前配信します
  （ユーザー入力からパスを組み立てないためパストラバーサル面はゼロ）。
- crates.io からの外部依存ビルドでは `fandhe-frontend-dist-server` の
  `build.rs`（ネスト wasm32 ビルド + wasm-bindgen によるフル機能構成用の
  wasm ステージ）が警告なく自動スキップされること（`workspace_detect` が
  「ワークスペースルートでのビルドではない」と判定するため。実測で確認済み）
- 単一バイナリ + `FROM scratch` の Docker イメージ最小化（ルート
  `Dockerfile`、REQ-9 の縮小版）

## 実測結果（イシュー #502 実装時、scratchpad の使い捨てプロジェクトで確認）

`fandhe-frontend-dist-server = "0.1.0"` へバージョン依存する最小プロジェクトで
`assets::active_mode()` / `assets::lookup("/static/style.css")` /
`routes::route_request("/")` を直接呼び出した結果:

| ビルド | `active_mode()` | `lookup("/static/style.css")` | `route_request("/").status` |
|--------|------------------|-------------------------------|------------------------------|
| debug（`cargo run`） | `DevFilesystem` | `None`（プロジェクト直下 `static/` は解決されない） | `200` |
| release（`cargo run --release`） | `Embedded` | `None`（埋め込みテーブルが空） | `200` |

いずれのビルドでも wasm ビルドステージの警告・エラーは出力されませんでした
（`build.rs` の自動スキップを確認）。この実測結果に基づき、本サンプルの
`src/main.rs` は `/static/style.css` のみ自前で `include_bytes!` 配信します
（上記「学べること」参照）。`fandhe-frontend-dist-server` ライブラリ側で
外部依存時にもプロジェクト直下 `static/` を配信できるようにする改善は
本サンプルのスコープ外です（PR 本文で後続 Issue 起票を提案）。

## 前提

- Rust ツールチェーン（`cargo`）
- crates.io（`https://index.crates.io` / `https://static.crates.io`）への到達性
  （依存解決に使用します）
- `fw gate --project examples/dist-server-docker` を実行する場合は clippy
  component / cargo-deny が必要です（`tools/ci/ensure-gate-tools.sh` で
  導入できます）
- Docker イメージのビルド・起動を試す場合は Docker（`docker build`/`docker run`）

## 動かし方

```bash
# ネイティブ起動（debug、DevFilesystem モード）
cargo run
# 別シェルで:
curl -sS http://127.0.0.1:3100/
curl -sS http://127.0.0.1:3100/static/style.css

# release ビルド（Embedded モード）
cargo run --release

# テスト（実プロセス起動 + GET / ・GET /static/style.css ・404 の検証）
cargo test

# fw gate（リポジトリルートから実行）
tools/ci/ensure-gate-tools.sh
cargo run -p fandhe-frontend-cli -- gate --project examples/dist-server-docker

# Docker イメージのビルド・起動（受け入れ条件 3。CI での自動検証は本サンプルの
# スコープ外、PR 本文の後続 Issue 提案を参照）
docker build -t dist-server-docker-example .
docker run --rm -p 3100:3100 dist-server-docker-example
# 別シェルで:
curl -sS http://127.0.0.1:3100/
```

## 主要ファイル

| ファイル | 説明 |
|---------|------|
| `Cargo.toml` | `fandhe-frontend-dist-server` + hyper 系クレートへの crates.io バージョン依存。root workspace から独立した `[workspace] members = ["."]` |
| `structure.toml` | `fw gate` が唯一の情報源として読む構造マニフェスト |
| `clippy.toml` | `raw_html()` 迂回検出ポリシー（`templates/default/` と内容同一） |
| `deny.toml` | 依存ポリシー（`templates/default/` と内容同一） |
| `src/main.rs` | 薄い hyper トランスポート層 + `/static/style.css` の自前配信 |
| `static/style.css` | 実演用の最小 CSS（`include_bytes!` で埋め込み） |
| `tests/boot.rs` | 実プロセス起動検証（GET / ・GET /static/style.css ・404） |
| `Dockerfile` | musl 静的リンク → `FROM scratch` のマルチステージビルド |
| `.dockerignore` | ビルドコンテキストからの除外（ルート版を流用） |

## 関連ガイド

- [`docs/guides/quickstart.md`](../../docs/guides/quickstart.md)
- [`docs/design/dist-server-design.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/design/dist-server-design.md)
- [`examples/ssr-routing/README.md`](../ssr-routing/README.md)（examples 規約の初例）

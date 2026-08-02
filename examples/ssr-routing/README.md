# examples/ssr-routing

## 概要

`fandhe-frontend` フレームワークの SSR（サーバーサイドレンダリング）+
ルーティングの正本サンプルです（イシュー #499）。`templates/`（`fw new` が
展開する「生成の雛形」）とは異なり、本サンプルは crates.io へ公開済みの
`fandhe-frontend-core` / `fandhe-frontend-app` / `fandhe-frontend-server`
（いずれも v0.2.0、イシュー #1159 で追随）をバージョン依存として実際に使う「正本」であり、
`examples/` 配下に追加される以降の全サンプルが従う標準構成（examples 規約）
の初例です。

## 学べること

- `fandhe_frontend_app::Loader` trait の自作実装（`fandhe_frontend_app::DemoItemsLoader` /
  `DemoItemDetailLoader` への決め打ちを避けた最小サンプル）
- `fandhe_frontend_server::ssr::respond_with` による一覧・詳細画面の SSR 応答組み立て
- `fandhe_frontend_app::router::Router` を独自ルート（`/hello/:name`）に直接使う実演と、
  `Params`（URL デコードされていない生文字列）を必ず `text()` 経由で出力する既定
  エスケープの実践
- 既定エスケープ（REQ-1）: HTML はすべてノード木 API（`el` / `p` / `text` /
  `page_shell`）で組み立て、`format!` によるタグ文字列の直接組み立て・
  `raw_html()` は使いません

## 前提

- Rust ツールチェーン（`cargo`）
- crates.io（`https://index.crates.io` / `https://static.crates.io`）への到達性
  （依存解決に使用します）
- `fw gate --project examples/ssr-routing` を実行する場合は clippy component /
  cargo-deny が必要です（`tools/ci/ensure-gate-tools.sh` で導入できます）

## 動かし方

```bash
# 詳細画面（200）
cargo run -- /items/1

# 未知パス（404）
cargo run -- /unknown

# 一覧画面（既定 "/"）
cargo run

# Router 実演（/hello/:name、text() 経由で name を出力）
cargo run -- /hello/world

# テスト（既定エスケープ回帰を含む）
cargo test

# fw gate（リポジトリルートから実行）
tools/ci/ensure-gate-tools.sh
cargo run -p fandhe-frontend-cli -- gate --project examples/ssr-routing
```

## 主要ファイル

| ファイル | 説明 |
|---------|------|
| `Cargo.toml` | crates.io バージョン依存 3 件のみ（`fandhe-frontend-core` / `-app` / `-server`）。root workspace から独立した `[workspace] members = ["."]` |
| `structure.toml` | `fw gate` が唯一の情報源として読む構造マニフェスト |
| `clippy.toml` | `raw_html()` 迂回検出ポリシー（`templates/default/` と内容同一） |
| `deny.toml` | 依存ポリシー（`templates/default/` と内容同一） |
| `src/main.rs` | SSR CLI エントリ（Loader 2 種 + `respond_with` + `Router` 実演） |
| `tests/routing.rs` | `respond_with` の 200/404・既定エスケープ回帰テスト |

## 関連ガイド

- [`docs/guides/quickstart.md`](../../docs/guides/quickstart.md)
- [`docs/api/app-api.md`](../../docs/api/app-api.md)
- [`docs/design/loader-trait-design.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/design/loader-trait-design.md)
- [`docs/design/route-definition-sharing.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/design/route-definition-sharing.md)

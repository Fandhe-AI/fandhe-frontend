# examples/interactive-view-transitions

## 概要

`fandhe-frontend` フレームワークの状態管理（REQ-8）+ View Transitions の
正本サンプルです（イシュー #503）。`examples/ssr-routing`（イシュー #499、
examples 規約の初例）と同じ構成規約に従い、crates.io へ公開済みの
`fandhe-frontend-core` / `fandhe-frontend-app` / `fandhe-frontend-interactive`
（いずれも v0.1.0）をバージョン依存として実際に使う「正本」です。

`fandhe-frontend-interactive` の状態機械 API（`Component` / `dispatch` /
`decode_action` / `render_for_hydration`）と、`page_shell` 同梱の
`@view-transition` at-rule + `fandhe-frontend-wasm-full` の `start_router`
（SPA 内遷移の View Transitions が JS 0 行で自動有効）を実演します。

## 学べること

- `fandhe_frontend_interactive::Component` trait（`update` / `view` /
  `decode_action`）を実装した参照コンポーネント `AppState`（カウンター・
  フォーム入力・動的リスト）に対する `dispatch(component, name, payload)`
  境界関数の使い方
- 未知アクション名の `dispatch` が no-op（`false` を返し状態不変）になる
  安全側フォールバック契約
- `render_for_hydration` によるハイドレーション属性付き `Node` の組み立てと、
  `fandhe_frontend_core::render()` の既定エスケープ（REQ-1）
- `page_shell` 同梱の `@view-transition { navigation: auto; }` と、
  `fandhe-frontend-wasm-full::entry::start_router` によるクロスドキュメント /
  SPA 内ページ遷移時の View Transitions 自動有効化（JS 0 行）
- `hydrate`（`AppState` 系、`id="interactive-root"`）と `start_router`
  （`layout()` が組む `<div id="app-root">` 系）は**別系統・別 DOM**である
  契約（`fandhe-frontend-wasm-full` entry.rs の doc 参照）

## 前提

- Rust ツールチェーン（`cargo`）
- crates.io（`https://index.crates.io` / `https://static.crates.io`）への到達性
  （依存解決に使用します）
- `fw gate --project examples/interactive-view-transitions` を実行する場合は
  clippy component / cargo-deny が必要です（`tools/ci/ensure-gate-tools.sh`
  で導入できます）
- ブラウザでの実動作確認（wasm ビルド）には `rustup target add
  wasm32-unknown-unknown` と `wasm/Cargo.lock` が解決したバージョンと一致する
  wasm-bindgen-cli が必要です（`tools/wasm/build.sh` 参照）

## 動かし方

```bash
# native デモ: 状態機械の dispatch 実演 + dist/index.html への SSR HTML 書き出し
cargo run

# テスト（既定エスケープ回帰・状態機械の不変条件を含む）
cargo test

# fw gate（リポジトリルートから実行）
tools/ci/ensure-gate-tools.sh
cargo run -p fandhe-frontend-cli -- gate --project examples/interactive-view-transitions

# ブラウザでの実動作確認（wasm ビルド。事前に rustup target add
# wasm32-unknown-unknown と wasm-bindgen-cli の導入が必要）
tools/wasm/build.sh
python3 -m http.server --directory static 8000
# ブラウザで http://localhost:8000/embed.html を開く
# （history API を使う data-nav 遷移の確認には file:// ではなく HTTP 配信が必須）
```

## 主要ファイル

| ファイル | 説明 |
|---------|------|
| `Cargo.toml` | crates.io バージョン依存 3 件のみ（`fandhe-frontend-core` / `-app` / `-interactive`）。root workspace から独立した `[workspace] members = ["."]` |
| `structure.toml` | `fw gate` が唯一の情報源として読む構造マニフェスト |
| `clippy.toml` | `raw_html()` 迂回検出ポリシー（`templates/default/` と内容同一） |
| `deny.toml` | 依存ポリシー（`templates/default/` と内容同一） |
| `src/main.rs` | native デモ（`AppState` の `dispatch` 実演）+ `dist/index.html` への SSR HTML 書き出し |
| `tests/state_machine.rs` | `dispatch` の状態遷移・未知アクション no-op・`render_for_hydration`・既定エスケープ回帰テスト |
| `static/embed.html` | ブラウザマウント骨格。`tools/wasm/build.sh` 実行後に動作（`hydrate("interactive-root")` + `start_router("app-root")`）。`interactive-root` は `cargo run` が書き出す `dist/index.html` 同要素（`data-hydrate-*` 属性付き）を事前に埋め込み済みで、`hydrate()` の状態復元が成功する（空のまま呼ぶと CSR フォールバックが `AppState::view()` を二重に差し込み id 衝突するため） |
| `tools/wasm/build.sh` | `wasm/`（独立ワークスペースの glue クレート）を wasm32 へビルドする手順 |
| `wasm/` | `fandhe-frontend-wasm-full` の `hydrate` / `mount` / `start_router` を再エクスポートする薄い glue クレート（root の依存グラフから隔離） |

## 関連ガイド

- [`docs/guides/quickstart.md`](../../docs/guides/quickstart.md)
- [`docs/api/interactive-api.md`](../../docs/api/interactive-api.md)
- [`docs/api/hydration-api.md`](../../docs/api/hydration-api.md)
- [`docs/design/wasm-full-architecture.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/design/wasm-full-architecture.md)

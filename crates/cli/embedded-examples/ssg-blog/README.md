# examples/ssg-blog

## 概要

`fandhe-frontend` フレームワークの SSG（静的サイト生成）正本サンプルです
（イシュー #501）。`examples/ssr-routing`（イシュー #499、SSR + ルーティング）
に続く examples 規約の 2 件目のサンプルであり、crates.io へ公開済みの
`fandhe-frontend-core` / `fandhe-frontend-server`（いずれも v0.1.0）を
バージョン依存として実際に使う「正本」です。記事一覧 + 各記事詳細ページを
静的 HTML として `dist/` へ書き出す最小ブログサイトを実演します。

## 学べること

- `fandhe_frontend_server::ssg::generate_pages`（イシュー #463）による任意の
  (リクエストパス, `Node`) 列の静的書き出し
- パス検証の fail-closed 契約: 不正なページパス（`..` を含む等）が 1 件でも
  あれば、正当な他のページも含めて何も書き出さないこと
- 正規化後の出力先重複（`/a` と `/a/` はいずれも `a/index.html`）の拒否
- 既定エスケープ（REQ-1）: 記事タイトル・本文はすべて `text()` 経由で
  ノード木へ載せ、`raw_html()` や HTML 文字列の直接組み立ては使いません
- `@view-transition { navigation: auto; }` による Cross-Document View
  Transitions の有効化（`fandhe_frontend_app::page_shell` と同一の固定リテラル）

## 前提

- Rust ツールチェーン（`cargo`）
- crates.io（`https://index.crates.io` / `https://static.crates.io`）への到達性
  （依存解決に使用します）
- `fw gate --project examples/ssg-blog` を実行する場合は clippy component /
  cargo-deny が必要です（`tools/ci/ensure-gate-tools.sh` で導入できます）

## 動かし方

```bash
# dist/ へ静的サイトを生成
cargo run

# 生成結果をブラウザで確認（任意）
python3 -m http.server -d dist 8000

# テスト（既定エスケープ回帰・fail-closed 回帰を含む）
cargo test

# fw gate（リポジトリルートから実行）
tools/ci/ensure-gate-tools.sh
cargo run -p fandhe-frontend-cli -- gate --project examples/ssg-blog
```

`cargo run` の実行後、`dist/index.html` と `dist/posts/<slug>/index.html`
（`hello-ssg` / `default-escaping` / `view-transitions` の 3 記事分）が
生成されます。

## 主要ファイル

| ファイル | 説明 |
|---------|------|
| `Cargo.toml` | crates.io バージョン依存 2 件のみ（`fandhe-frontend-core` / `-server`）。root workspace から独立した `[workspace] members = ["."]` |
| `structure.toml` | `fw gate` が唯一の情報源として読む構造マニフェスト |
| `clippy.toml` | `raw_html()` 迂回検出ポリシー（`templates/default/` と内容同一） |
| `deny.toml` | 依存ポリシー（`templates/default/` と内容同一） |
| `src/main.rs` | SSG エントリ（`layout` / `index_page` / `post_page` / `build_pages` + `generate_pages` 呼び出し） |
| `src/posts.rs` | 固定記事データ（XSS ペイロード入り記事 1 件を含む） |
| `tests/ssg_output.rs` | `generate_pages` の既定エスケープ・fail-closed・重複拒否回帰と CLI ブラックボックステスト |

## 関連ガイド

- [`docs/guides/quickstart.md`](../../docs/guides/quickstart.md)
- [`docs/api/app-api.md`](../../docs/api/app-api.md)
- [`examples/ssr-routing/README.md`](../ssr-routing/README.md)

# examples/wireframe-ui

## 概要

`fandhe-frontend-wireframe-ui`（blocks.pm 参照のローファイ・モノクロ
ワイヤーフレーム UI コンポーネント層、イシュー #2599/#2600/#2603）の
ショーケース正本サンプルです（イシュー #2667）。`examples/headless-pre-styled-ui`
（イシュー #609）に続く examples 規約の 6 件目のサンプルであり、crates.io へ
公開済みの `fandhe-frontend-core` / `fandhe-frontend-wireframe-ui`
（v0.4.3/v0.52.0）をバージョン依存として実際に使う「正本」です。全 49 部品
（blocks.pm 由来 35 + 追加 14）を Phase 1〜8 の区分どおりに 1 ページへ並べて
実演します。

`fandhe-frontend-wireframe-ui` は SSR 専用・非インタラクティブな表示専用
部品層であり、状態遷移・対話操作を一切持ちません。そのため本サンプルは
`fandhe-frontend-server`（`generate_pages` 等の SSG API）にも依存せず、
ページを 1 枚組み立てて `dist/` へ書き出すだけの最小構成です。

## 学べること

- `fandhe_frontend_wireframe_ui` の Phase 1〜8・全 49 部品の呼び出し方
  （`src/sections.rs`）
- 既定エスケープ（REQ-1）: 部品へ渡すテキスト引数は内部で
  `fandhe_frontend_core::text()` 経由へ渡され既定エスケープされます。
  `<script>` を含む注釈タイトルの実演（`src/sections.rs` の XSS 実演節）で
  確認できます
- `wireframe_css()` の CSS 出力を静的アセットとして書き出す最小構成、および
  `<style>` インライン埋め込みではなく `<link rel="stylesheet">` 経由の別
  ファイル参照を採る設計判断（`wireframe_css()` が `>` を含む子結合子
  セレクタを持つため、`<style>` へインライン埋め込みすると既定エスケープに
  より `>` が `&gt;` に実体参照化されてセレクタが壊れる。詳細は
  `src/main.rs` rustdoc「CSS の出力方式」節を参照）

## 前提

- Rust ツールチェーン（`cargo`）
- crates.io（`https://index.crates.io` / `https://static.crates.io`）への到達性
  （依存解決に使用します）
- `fw gate --project examples/wireframe-ui` を実行する場合は clippy component /
  cargo-deny が必要です（`tools/ci/ensure-gate-tools.sh` で導入できます）

## 動かし方

```bash
# dist/ へ静的ページを生成
cargo run

# 生成結果をブラウザで確認（任意）
python3 -m http.server -d dist 8000

# テスト（既定エスケープ回帰を含む）
cargo test

# fw gate（リポジトリルートから実行）
tools/ci/ensure-gate-tools.sh
cargo run -p fandhe-frontend-cli -- gate --project examples/wireframe-ui
```

`cargo run` の実行後、`dist/index.html`（全 49 部品のショーケース）と
`dist/assets/wireframe.css`（`wireframe_css()` の出力）が生成されます。

## 主要ファイル

| ファイル | 説明 |
|---------|------|
| `Cargo.toml` | crates.io バージョン依存 2 件のみ（`fandhe-frontend-core` / `-wireframe-ui`）。root workspace から独立した `[workspace] members = ["."]` |
| `structure.toml` | `fw gate` が唯一の情報源として読む構造マニフェスト |
| `clippy.toml` | `raw_html()` 迂回検出ポリシー（`templates/default/` と内容同一） |
| `deny.toml` | 依存ポリシー（`templates/default/` と内容同一） |
| `src/main.rs` | ページ骨格（`layout`）+ `dist/` への書き出しエントリ |
| `src/sections.rs` | Phase 1〜8・全 49 部品の呼び出しとショーケース本文の組み立て |
| `tests/cli_output.rs` | 既定エスケープ回帰と CLI ブラックボックステスト |

## 関連ガイド

- `docs/design/wireframe-ui-architecture.md`
- [`docs/guides/quickstart.md`](../../docs/guides/quickstart.md)
- [`examples/headless-pre-styled-ui/README.md`](../headless-pre-styled-ui/README.md)

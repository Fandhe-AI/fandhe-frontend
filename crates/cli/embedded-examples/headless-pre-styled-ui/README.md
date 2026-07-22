# examples/headless-pre-styled-ui

## 概要

`fandhe-frontend-headless-ui`（ark-ui 相当の headless UI コンポーネント層）
のショーケース正本サンプルです（イシュー #552、親トラッキング #520 の
Phase 4）。Tabs / Accordion / Dialog / Switch / RadioGroup / Avatar の代表的な
コンポーネントを静的 SSR マークアップとして 1 ページに組み立て、`dist/` へ
書き出します。

## crates.io バージョン依存について

他の examples/（`ssr-routing` / `ssg-blog` / `dist-server-docker` /
`interactive-view-transitions`）と同じく、`fandhe-frontend-core` /
`fandhe-frontend-headless-ui` への crates.io バージョン依存のみで完結する
正本サンプルです（examples 規約、イシュー #499）。作成当初（イシュー #552）
は `fandhe-frontend-headless-ui` が crates.io 未公開だったため path 依存の
意図的な例外でしたが、前提クレート公開（イシュー #608）を受けてイシュー
#609 でバージョン依存へ切り替え、`fw new --example headless-pre-styled-ui`
にも対応しました。

## pre-styled-ui 統合について

`fandhe-frontend-pre-styled-ui`（chakra-ui 相当の上層、#520/#546）は本サンプ
ル作成時点でクレート骨格のみ（テーマトークン #547・variant API・静的 CSS
生成 #548・styled 部品 #550/#551 が並列進行中で未マージ）であり、公開 API を
持ちません。そのため本サンプルは `fandhe-frontend-pre-styled-ui` を依存に
持たず、代わりに headless-ui が出力する `data-scope`/`data-part`/`data-state`
セレクタへ手書きで当てる最小 CSS（`static/ui.css`）を同梱しています。
pre-styled-ui の公開 API が揃い次第、本サンプルへの統合をフォローアップします。

## 学べること

- `fandhe-frontend-headless-ui` の anatomy（`data-scope`/`data-part`）・
  `data-*` 状態属性・WAI-ARIA 属性付与（Tabs / Accordion / Dialog / Switch /
  RadioGroup / Avatar の 6 コンポーネント）
- 既定エスケープ（REQ-1）: コンポーネントへ渡す文字列はすべて `text()` 経由で
  ノード木へ載せ、`raw_html()` や `format!` によるタグ文字列の直接組み立ては
  使いません
- `@view-transition { navigation: auto; }` による Cross-Document View
  Transitions の有効化（`fandhe_frontend_app::page_shell` と同一の固定リテラル）

## 前提

- Rust ツールチェーン（`cargo`）
- 本サンプルは crates.io バージョン依存で完結するため、`https://index.crates.io`・
  `https://static.crates.io` への到達性が必要です（到達不可の場合は環境エラー
  として扱います。`.claude/rules/ci.md` 参照）
- `fw gate --project examples/headless-pre-styled-ui` を実行する場合は clippy
  component / cargo-deny が必要です（`tools/ci/ensure-gate-tools.sh` で
  導入できます）

## 動かし方

```bash
# fw new --example でリポジトリ外へ展開する場合
fw new my-headless-pre-styled-ui --example headless-pre-styled-ui

# dist/ へショーケースページを生成
cargo run

# 生成結果をブラウザで確認（任意）
python3 -m http.server -d dist 8000

# テスト（anatomy・data-state・ARIA・既定エスケープ回帰を含む）
cargo test

# fw gate（リポジトリルートから実行）
tools/ci/ensure-gate-tools.sh
cargo run -p fandhe-frontend-cli -- gate --project examples/headless-pre-styled-ui
```

`cargo run` の実行後、`dist/index.html` と `dist/assets/ui.css` が生成されます。

## 主要ファイル

| ファイル | 説明 |
|---------|------|
| `Cargo.toml` | `fandhe-frontend-core` / `-headless-ui` への crates.io バージョン依存 2 件。root workspace から独立した `[workspace] members = ["."]` |
| `structure.toml` | `fw gate` が唯一の情報源として読む構造マニフェスト |
| `clippy.toml` | `raw_html()` 迂回検出ポリシー（`templates/default/` と内容同一） |
| `deny.toml` | 依存ポリシー（`templates/default/` と内容同一） |
| `src/main.rs` | ショーケースページ組み立て（`layout` + コンポーネントごとの `*_section` 関数）+ `dist/` 書き出し |
| `static/ui.css` | headless-ui の `data-scope`/`data-part`/`data-state` セレクタへ当てる手書き CSS（pre-styled-ui 未実装のための暫定代替） |
| `tests/cli_output.rs` | anatomy・ARIA・既定エスケープ回帰の CLI ブラックボックステスト（`src/main.rs` 内の `#[cfg(test)]` ユニットテストと二本立て） |

## 関連ガイド

- [`docs/api/headless-ui-api.md`](../../docs/api/headless-ui-api.md)
- [`docs/api/pre-styled-ui-api.md`](../../docs/api/pre-styled-ui-api.md)
- [`docs/api/component-api.md`](../../docs/api/component-api.md)
- [`examples/ssg-blog/README.md`](../ssg-blog/README.md)

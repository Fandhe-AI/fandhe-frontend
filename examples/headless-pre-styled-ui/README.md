# examples/headless-pre-styled-ui

## 概要

`fandhe-frontend-headless-ui`（ark-ui 相当の headless UI コンポーネント層）と
`fandhe-frontend-pre-styled-ui`（chakra-ui 相当の pre-styled 上層）の
2 層 UI コンポーネント構成のショーケース正本サンプルです（イシュー #552、
親トラッキング #520 の Phase 4）。両層の代表的なコンポーネントを静的 SSR
マークアップとして 1 ページに組み立て、`dist/` へ書き出します。

## crates.io バージョン依存について

他の examples/（`ssr-routing` / `ssg-blog` / `dist-server-docker` /
`interactive-view-transitions`）と同じく、`fandhe-frontend-core` /
`fandhe-frontend-headless-ui` / `fandhe-frontend-pre-styled-ui` への crates.io
バージョン依存のみで完結する正本サンプルです（examples 規約、イシュー #499）。
作成当初（イシュー #552）は `fandhe-frontend-headless-ui` が crates.io 未公開
だったため path 依存の意図的な例外でしたが、前提クレート公開（イシュー #608）
を受けてイシュー #609 でバージョン依存へ切り替え、
`fw new --example headless-pre-styled-ui` にも対応しました。

## pre-styled-ui 統合について

サンプル作成時点（イシュー #552、2026-07-22）では pre-styled-ui がクレート
骨格のみだったため、headless-ui + 手書き CSS（`static/ui.css`）で代替して
いました。pre-styled-ui v0.3.1 で公開 API（styled 部品・headless ラッパー・
`StyleSheet`/`Theme`）が揃ったため、本サンプルは 2 層構成のデモとして統合
済みです。各コンポーネントの層別内訳:

| コンポーネント | 使用する層 | 備考 |
|---------------|-----------|------|
| Tabs / Accordion / Dialog / Menu / Select / Popover / Tooltip | pre-styled-ui（headless ラッパー） | マークアップは headless 層の再エクスポート、既定 CSS は各モジュールの `stylesheet()`。Menu / Select はラッパー第 1 弾（#551）、Popover / Tooltip は第 2 弾（#664、PR #672）で追加 |
| Button / Badge / Card / Alert / Spinner | pre-styled-ui（単純 styled 部品） | variant / size / colorPalette を Rust enum で型安全に指定 |
| Switch / RadioGroup / Avatar | headless-ui + 手書き CSS | pre-styled-ui に styled ラッパー未提供のため `static/ui.css` で直接スタイル |

Menu / Select / Popover / Tooltip はいずれも `positioner` が `position:
absolute` のオーバーレイ型のため、Dialog 節と同じ「SSR 初期状態は closed、
全 anatomy を DOM に掲載（`hidden` 付き）」方針で掲示します。Select のみ、
listbox を closed のまま「選択済み値」（`value_text`/`aria-selected`/
`hidden_select` の `selected` option）を実演し、Menu は virtual focus による
`data-highlighted` 項目の実演を含みます。

CSS はテーマトークン（`Theme::default()`）・使用コンポーネントの recipe
CSS・手書き残存分（`static/ui.css`）を `StyleSheet` へ集約し、
`StyleSheet::write_css_file`（SSG 向け経路）で `dist/assets/ui.css` 1 ファイル
へ書き出します（`src/main.rs` の `build_stylesheet()`）。

## 学べること

- `fandhe-frontend-headless-ui` の anatomy（`data-scope`/`data-part`）・
  `data-*` 状態属性・WAI-ARIA 属性付与（Tabs / Accordion / Dialog / Menu /
  Select / Popover / Tooltip / Switch / RadioGroup / Avatar）
- `fandhe-frontend-pre-styled-ui` の variant API（`ButtonVariant`/`Size`/
  `ColorPalette` 等の Rust enum によるクラス切り替え）・headless ラッパー・
  `StyleSheet`/`Theme` による静的 CSS 集約
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
| `Cargo.toml` | `fandhe-frontend-core` / `-headless-ui` / `-pre-styled-ui` への crates.io バージョン依存 3 件。root workspace から独立した `[workspace] members = ["."]` |
| `structure.toml` | `fw gate` が唯一の情報源として読む構造マニフェスト |
| `clippy.toml` | `raw_html()` 迂回検出ポリシー（`templates/default/` と内容同一） |
| `deny.toml` | 依存ポリシー（`templates/default/` と内容同一） |
| `src/main.rs` | ショーケースページ組み立て（`layout` + コンポーネントごとの `*_section` 関数）+ `build_stylesheet()` による CSS 集約 + `dist/` 書き出し |
| `static/ui.css` | ページ骨格 + pre-styled-ui 未提供コンポーネント（Switch / RadioGroup / Avatar）向けの手書き残存 CSS（`build_stylesheet()` が `StyleSheet` へ取り込む） |
| `tests/cli_output.rs` | anatomy・ARIA・既定エスケープ回帰の CLI ブラックボックステスト（`src/main.rs` 内の `#[cfg(test)]` ユニットテストと二本立て） |

## 関連ガイド

- [`docs/api/headless-ui-api.md`](../../docs/api/headless-ui-api.md)
- [`docs/api/pre-styled-ui-api.md`](../../docs/api/pre-styled-ui-api.md)
- [`docs/api/component-api.md`](../../docs/api/component-api.md)
- [`examples/ssg-blog/README.md`](../ssg-blog/README.md)

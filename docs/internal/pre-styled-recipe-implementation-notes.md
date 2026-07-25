# fandhe-frontend-pre-styled-ui slot recipe 実装ノート（docs サイト非掲載）

> 本書は docs サイトへ出力しない内部設計記録である（`site/nav.toml` へ登録
> しない）。リポジトリは public であり「非掲載＝非公開」ではない。
> 分離方針は `docs/design/docs-site-api-reference-split.md`（イシュー #952）。
> 凍結された公開 API 契約自体の正は
> [`docs/api/pre-styled-recipe-api.md`](../api/pre-styled-recipe-api.md)。
> Phase 7 の全文検索インデックス（#956〜#958）にも本書は含めない
> （既定、`docs-site-api-reference-split.md` §6 再評価トリガー 4）。

## 0. 旧 → 新 マッピング表

| 旧（`docs/api/pre-styled-recipe-api.md`） | 新（本書） |
|---|---|
| §1 目的とトレーサビリティ（親イシュー・実装イシュー・スコープ節） | 本書 §1 |
| §7 `#547`（テーマトークン）との関係（実配線の経緯） | 本書 §2 |
| §8 スコープ外（Issue 化候補） | 本書 §3 |

## 1. トレーサビリティ・スコープ（イシュー #548）

- 親: Phase 3 親イシュー #545、トラッキング #520
- 実装: `crates/pre-styled-ui/src/css.rs`（低レベル宣言・検証・シリアライズ）・
  `crates/pre-styled-ui/src/recipe.rs`（`SlotRecipe`/`VariantValue`/`Size`、
  compoundVariants 相当は `VariantCondition`/`when`/`SlotRecipe::compound_variant`、
  イシュー #604）
- テスト: `crates/pre-styled-ui/tests/recipe_css.rs`（golden・headless 接続・
  fail-closed・compound variant）・`crates/pre-styled-ui/tests/recipe_determinism.rs`
  （決定性、compound variant を含む）

**本タスク（#548）のスコープ**: variant 定義 API（base / variants /
defaultVariants 相当）・静的 CSS 生成・headless 層セレクタとの接続・決定性の
担保のみ。テーマトークン・ダークモード基盤はイシュー #547、styled 部品実装
（Button 等 #550・Dialog 等ラッパー #551）は別イシューのスコープ。

## 2. `#547`（テーマトークン）実配線の経緯

`#606` で colorPalette 軸を実配線した際、`palette_declarations` は
`crate::theme` が生成する `--fandhe-color-*`（テーマ層の名前空間）とは別の
`--fandhe-palette-*` 名前空間へ、選択された palette に対応する
`accent`/`info`/`success`/`warning`/`danger` の 3 役割（base/emphasized/fg）を
`var()` 参照として束ねる（chakra-ui の virtual token 方式の静的 CSS 版）とする
設計判断を行った。名前空間分離・radii/shadow トークン参照の契約自体は
[`docs/api/pre-styled-recipe-api.md`](../api/pre-styled-recipe-api.md) §7 を
正とする（本節は「#606 でこの設計を採用した」という実装経緯のみを記録する）。

## 3. スコープ外（Issue 化候補、#548 時点）

- recipe 出力の CSS ファイル書き出し・`<style>` 埋め込みヘルパ:
  **イシュー #605 で実装済み**。`crate::stylesheet::StyleSheet`
  （[`docs/api/pre-styled-ui-api.md`](../api/pre-styled-ui-api.md) 参照）が
  `SlotRecipe::css()`/`Theme::to_css()` の出力を集約し、
  `write_css_file`（静的 `.css` 書き出し）・`style_element`（SSR 用
  `<style>` 要素、`raw_html()` を内部に閉じ込めた検証済み CSS 型経由）の
  2 経路を提供する。
- `#547` テーマトークンとの palette 実配線（colorPalette 軸の意味付け）:
  **イシュー #606 で実装済み**（`palette_declarations`・`ColorPalette`）。
- 既存 styled 部品（button/alert 等）への compound variant の実適用（必要に
  なった時点で該当部品のイシューで対応）

`compoundVariants` 相当（複数軸の組み合わせ条件スタイル）は
[`docs/api/pre-styled-recipe-api.md`](../api/pre-styled-recipe-api.md)
`§2`〜`§6` に記載のとおりイシュー #604 で実装済み。

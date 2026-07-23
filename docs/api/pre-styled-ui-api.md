# fandhe-frontend-pre-styled-ui API

## 1. 目的とトレーサビリティ

本ドキュメントは `fandhe-frontend-pre-styled-ui`（chakra-ui 参考の
pre-styled UI コンポーネント層、親トラッキング #520・骨格新設 #546）の
公開 API 表面をまとめる。`fandhe-frontend-headless-ui`（ark-ui 相当の下層、
[`docs/api/headless-ui-api.md`](./headless-ui-api.md)）の上に、テーマ
トークン・variant API・静的 CSS 生成を重ね、styled 部品を実装する 2 層
構造の上層を担う。

**spec 未反映の注記**: `fandhe-frontend-headless-ui` と同様、本クレートに
対応する REQ / TASK は `docs/spec/` に存在しない（要件提案は
fandhe-frontend-spec リポジトリの Issue #20 として起票済み、#520 参照）。

## 2. 実装状況（本書作成時点、2026-07-22）

本クレートは **crate 骨格のみ**（イシュー #546）であり、公開 API を持たない
（`src/lib.rs` はクレート doc コメントのみ）。以下は並列進行中のイシューで
あり、本書はそれらのマージ後に更新する。

| イシュー | 内容 | 状態（本書作成時点） |
|---|---|---|
| #547 | テーマトークン・ダークモード基盤 | 実装中（未マージ） |
| #548 | slot recipe 相当の variant API・静的 CSS 生成 | 実装中（未マージ） |
| #550 | Button 等の単純な styled 部品 | 未着手 |
| #551 | headless-ui ラッパー（Accordion/Dialog 等の styled 版） | 未着手 |
| #606 | colorPalette 軸の実配線・radii/shadow トークン追加 | 実装中（未マージ） |

**本節の既知の陳腐化**: 上表は #550/#551/#606 のマージ後も未更新のまま残って
いる（本項目は #606 実装時点の out-of-scope 候補として記録。全面改訂は別
イシューで扱う）。実装済み API の正本は `crates/pre-styled-ui/src/lib.rs`
冒頭の rustdoc を参照。

`examples/headless-pre-styled-ui`（#552）は本クレートが未実装のため、
headless-ui の `data-scope`/`data-part`/`data-state` セレクタへ手書きで
当てる CSS（`examples/headless-pre-styled-ui/static/ui.css`）を暫定的な
代替として同梱している。本クレートの公開 API が揃い次第、同サンプルへの
統合をフォローアップする。

## 3. 不変条件（実装済み・骨格に記載済み、`src/lib.rs` 参照）

1. コンポーネントは `fandhe_frontend_headless_ui` 経由で
   `fandhe_frontend_core::Node` を返す通常の Rust 関数として実装する
   （REQ-5、マクロ DSL は採用しない）。
2. 出力は `fandhe_frontend_core::render` の既定エスケープを必ず経由する。
   本クレート内で `raw_html()` を使用しない（新たなエスケープ迂回経路を
   作らない）。
3. `#![forbid(unsafe_code)]`（REQ-2）によりクレート全体で `unsafe` を機械的
   に禁止する。
4. 外部依存は `fandhe-frontend-headless-ui`（path）のみ。
   `fandhe-frontend-core` への直接依存は宣言しない（headless-ui 経由で
   間接的に利用する。`fandhe-frontend-core` はスモークテスト用の
   dev-dependency としてのみ許容する）。

これらの不変条件は #547/#548/#550/#551 の実装レビューでもそのまま適用される
（`.claude/rules/coding-rust.md`・`docs/api/headless-ui-api.md` §6 と同一の
制約を上層でも維持する）。

## 4. 設計方針（予定、#547/#548 の実装完了後に本節を更新）

- **テーマトークン**（#547）: 色・スペーシング等のデザイントークンと
  ダークモード切り替えの基盤。chakra-ui の `system`/`recipe` 相当の設計を
  参考にしつつ、静的 SSR 出力（ビルド時に確定する CSS）を前提とする。
- **variant API・静的 CSS 生成**（#548）: chakra-ui の slot recipe 相当。
  コンポーネントの見た目バリエーション（size/variant/colorPalette 等）を
  型安全に選択し、対応する静的 CSS を生成する。
- **styled 部品**（#550/#551）: #550 は Button 等の単純な部品、#551 は
  headless-ui の Accordion/Dialog 等をラップした styled 版を提供する予定。

## 4a. `stylesheet::StyleSheet`（recipe / theme CSS の書き出し・埋め込みヘルパ、イシュー #605）

`SlotRecipe::css()`・`Theme::to_css()`・各 styled 部品の `css()`/`stylesheet()`
は決定的な CSS 文字列を返すのみで、その先の配布は呼び出し側任せだった
（`examples/headless-pre-styled-ui` の手書き `static/ui.css` コピーが実例）。
`stylesheet::StyleSheet` はこれを集約し、2 つの配布経路を提供する。

- `StyleSheet::new()` / `push_css(&mut self, css: &str) -> Result<(), StylesheetError>`:
  唯一の fallible な取り込み口。`<` を含む、または改行・タブ・復帰以外の
  制御文字を含む入力は `Err(StylesheetError::CssRejected { .. })` になる
  （fail-closed）。
- `push_recipe(&mut self, recipe: &SlotRecipe)` / `push_theme(&mut self, theme: &Theme)`:
  生成側 allowlist 検証（`<` を構成不能にする）に依拠した infallible な
  薄いラッパ。
- `as_css(&self) -> &str`: 取り込んだ CSS 全量。
- `write_css_file(&self, path: &Path) -> std::io::Result<()>`: 静的 `.css`
  ファイルへの書き出し（SSG・ビルドスクリプト向け。親ディレクトリを自動作成）。
- `style_element(&self) -> Node`: SSR 用 `<style>` 要素ノード。本クレートで
  `raw_html()` を使用する唯一の箇所（`src/lib.rs` 冒頭の不変条件 2 の例外）
  であり、呼び出し文に
  `#[expect(clippy::disallowed_methods, reason = "ESCAPE-REVIEWED: ...")]`
  を付与済み。`StyleSheet` は private フィールドのみで構成され、検証済み
  CSS 以外から構築する経路を公開しないため、呼び出し側へエスケープ迂回
  経路を公開しない。

```rust
use fandhe_frontend_pre_styled_ui::stylesheet::StyleSheet;
use fandhe_frontend_pre_styled_ui::theme::Theme;

let mut sheet = StyleSheet::new();
sheet.push_theme(&Theme::default());
sheet.push_css(&fandhe_frontend_pre_styled_ui::button::css()).unwrap();

// SSG: 静的ファイルとして配信する
sheet.write_css_file(std::path::Path::new("static/ui.css")).unwrap();

// SSR: <style> 要素として埋め込む（render() が既定エスケープを適用する
// 他のノードと同様に合成できる）
let _style_node = sheet.style_element();
```

## 4b. styled RadioGroup ラッパー（イシュー #683）

`radio_group` モジュールは `fandhe_frontend_headless_ui::radio_group`
（イシュー #558/#536）の Root/Label/Item/ItemControl/ItemText/
ItemHiddenInput 6 anatomy パーツと `RadioGroup` 状態機械をそのまま
再エクスポート（`pub use fandhe_frontend_headless_ui::radio_group::*`）し、
`stylesheet()` で既定 CSS を追加提供する（設計方針は #551/#664 の他
headless ラッパーと同じ、`src/lib.rs` 冒頭の rustdoc 参照）。

- **`item-hidden-input` の視覚的非表示化**: headless 層はネイティブ
  `<input type="radio">` に `aria`/`data-*` のみを設定し視覚的な非表示化を
  行わない契約のため、styled 層が visually-hidden パターン（`position:
  absolute` + 1px クリップ、`select` モジュールの `hidden-select` 規則と
  同一の 9 宣言）で覆い隠し、`item-control` をカスタムラジオ円として描画
  する。フォーム送信・キーボード操作・グループ内排他選択はネイティブ
  semantics のまま維持される。
- **`StateCondition::FocusWithin` の追加**: `item-hidden-input` を視覚的に
  隠すと、ネイティブのフォーカスリングも見えなくなる。実フォーカスは
  隠された `<input>` にあり、`item`（`<label>`、input の祖先）へ
  `:focus-within` を当てるのが CSS 的に成立する唯一の経路のため、
  `recipe::StateCondition` へ `FocusWithin`（`:focus-within` 擬似クラス）を
  追加した（既存の `Attr`/`AttrEq`/`FocusVisible` に次ぐ 4 つ目の状態条件）。
- 他の headless ラッパーと同様、variant（size 等）ごとのクラス切り替えは
  スコープ外（単一既定スタイルのみ）。

## 5. 関連ドキュメント

- [`docs/api/headless-ui-api.md`](./headless-ui-api.md): 本クレートの下層
- [`docs/api/component-api.md`](./component-api.md): `Node`/`el`/`text`/
  `raw_html`/`render` の凍結 API 表面
- [`examples/headless-pre-styled-ui/README.md`](../../examples/headless-pre-styled-ui/README.md):
  本クレート未実装時点での暫定サンプル（pre-styled-ui 統合について節参照）
- `.claude/skills/chakra-ui/`: 設計時の参考にした chakra-ui リファレンス
  スキル

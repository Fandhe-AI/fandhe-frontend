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

## 2. 実装状況（v0.4.0 時点、2026-07-23 更新）

**記載方針**: 実装済み API の正は `crates/pre-styled-ui/src/lib.rs` 冒頭の
rustdoc および各モジュール冒頭の rustdoc とする。本節はモジュール一覧の
概要のみを保持し、イシューごとの進行状態（未着手・実装中・マージ待ち等）は
記載しない。マージ済みイシューを本節から都度更新する運用は陳腐化しやすく、
実際に骨格新設（#546）時点の記述が長期間放置されていた（イシュー #714）。

本クレートは第 5 弾ツリー（#680）完了・crates.io v0.4.0 公開（#686）を経て
19 の公開モジュールを持つ。内訳は次の通り。

| 分類 | モジュール | 由来イシュー |
|---|---|---|
| 基盤 | `theme` | #547/#606 |
| 基盤 | `css` | #548 |
| 基盤 | `recipe` | #548/#606/#604（詳細は [`pre-styled-recipe-api.md`](./pre-styled-recipe-api.md)） |
| 基盤 | `stylesheet` | #605（CSS 集約・配布ヘルパ、§4a 参照） |
| 単純 styled 部品 | `button` / `badge` / `spinner` / `alert` / `card` | #550/#606 |
| headless ラッパー第 1 弾 | `dialog` / `tabs` / `accordion` / `menu` / `select` | #551 |
| headless ラッパー第 2 弾 | `popover` / `tooltip` | #664 |
| headless ラッパー第 3 弾 | `switch` | #682 |
| headless ラッパー第 4 弾 | `radio_group` | #683（§4c 参照） |
| headless ラッパー | `avatar` | #684（§4b 参照） |

各 headless ラッパーモジュールは対応する `fandhe_frontend_headless_ui`
モジュールの anatomy パーツ・状態機械を薄く再エクスポートし、
`stylesheet()`（モジュールにより `css()`）で既定 CSS を追加提供する共通
設計方針を採る。詳細・スコープ外事項は各モジュール冒頭の rustdoc を参照
（例: `switch` は `src/switch.rs`、`avatar`/`radio_group` は §4b/§4c）。

クレートルート再エクスポート（`fandhe_frontend_headless_ui` /
`fandhe_frontend_core` / `OpenState` / `Orientation` ほか、イシュー #685）は
§3a を参照。

`examples/headless-pre-styled-ui`（#552/#678/#698/#704）は本クレート
v0.4.0（`fandhe-frontend-pre-styled-ui = "0.4.0"`、crates.io バージョン
依存）へ統合済みである。旧来 headless-ui の `data-scope`/`data-part`/
`data-state` セレクタへ手書きで当てていたコンポーネント CSS は撤去され
（イシュー #689）、`src/main.rs` の `build_stylesheet()` が `Theme`/
`SlotRecipe` から生成した CSS を `stylesheet::StyleSheet` で集約し
`dist/assets/ui.css` へ書き出す方式へ切り替え済み。
`static/ui.css` はショーケースページ固有の骨格レイアウトのみを保持する
形で残存する。

## 3. 不変条件（実装済み・骨格に記載済み、`src/lib.rs` 参照）

1. コンポーネントは `fandhe_frontend_headless_ui` 経由で
   `fandhe_frontend_core::Node` を返す通常の Rust 関数として実装する
   （REQ-5、マクロ DSL は採用しない）。
2. 出力は `fandhe_frontend_core::render` の既定エスケープを必ず経由する。
   `raw_html()` の使用は `stylesheet::StyleSheet::style_element` 内の
   レビュー済み 1 箇所（`#[expect(clippy::disallowed_methods, ...)]` 付き）
   に限定する（イシュー #605、§4a 参照）。新たなエスケープ迂回経路を
   作らない。
3. `#![forbid(unsafe_code)]`（REQ-2）によりクレート全体で `unsafe` を機械的
   に禁止する。
4. 外部依存は `fandhe-frontend-headless-ui`（path）のみ。
   `fandhe-frontend-core` への直接依存は宣言しない（headless-ui 経由で
   間接的に利用する。`fandhe-frontend-core` はスモークテスト用の
   dev-dependency としてのみ許容する）。

これらの不変条件は実装済み各モジュール（§2 参照）でも維持されている
（`.claude/rules/coding-rust.md`・`docs/api/headless-ui-api.md` §6 と同一の
制約を上層でも維持する）。

## 3a. headless 型の再エクスポート契約（イシュー #685）

`fandhe-frontend-headless-ui` の 7 モジュール（`tabs`/`accordion`/`dialog`/
`menu`/`select`/`popover`/`tooltip`）を薄くラップする各 pre-styled-ui
モジュールは `pub use fandhe_frontend_headless_ui::<mod>::*;` で同名モジュールを
再エクスポートするが、この glob 再エクスポートは**ラッパー呼び出しに必要な
「モジュール外」の headless 型**（`state`/`data_attrs` モジュール由来）まで
は届かない。PR #679 で `fandhe-frontend-docs-site` が `fandhe-frontend-headless-ui`
へ直接依存せざるを得なかったのはこのためである（`Orientation`/`OpenState`
を pre-styled-ui のパスから import できなかった）。

本イシューはこれを解消し、**pre-styled-ui のみへの依存でラッパーを呼び出せる
ことを保証する契約**として、以下を明示 `pub use` で再エクスポートする
（棚卸し表、`crates/pre-styled-ui/src/{tabs,accordion,dialog,menu,select,
popover,tooltip}.rs` の各ファイル冒頭の `pub use` 直後のコメント参照）。

| pre-styled-ui モジュール | 再エクスポートする headless 型 | 由来 |
|---|---|---|
| `tabs` | `Orientation` | `data_attrs` |
| `accordion` | `OpenState` / `SingleSelectAction` / `MultiSelectAction` | `state` |
| `dialog` | `OpenState` / `DisclosureAction` | `state` |
| `menu` | `OpenState` / `DisclosureAction` / `CheckableAction` / `SingleSelectAction` | `state` |
| `select` | `OpenState` | `state` |
| `popover` | `OpenState` / `DisclosureAction` | `state` |
| `tooltip` | `OpenState` / `DisclosureAction` | `state` |

`ActivationMode`/`TabItem`/`TabsProps`（tabs）・`DialogRole`/`ContentIds`
（dialog）・`SelectAction`（select）は各 headless モジュール内定義のため
既存の glob 再エクスポートで到達可能であり、追加の再エクスポートは不要
（モジュール自身の `impl Component` の `Action` として使う場合を含む）。

加えて、クレートルート（`crates/pre-styled-ui/src/lib.rs`）から次を
再エクスポートする。

- `pub use fandhe_frontend_headless_ui;`: headless 層クレートそのもの。
  headless-ui が core に対して行う再エクスポート（イシュー #550）と同型の
  エスケープハッチであり、各ラッパーモジュールの glob では届かない
  headless API 全域（`positioning`/`aria` 等）への到達路を確保する。
- `pub use fandhe_frontend_headless_ui::fandhe_frontend_core;`: `Node` を
  組み立てる core API（`el`/`text`/`render` 等）への推移的再エクスポート。
  `fandhe_frontend_pre_styled_ui::fandhe_frontend_core::{el, text, render,
  Node}` という単独依存パスを完結させる（`Cargo.toml` へ
  `fandhe-frontend-core` への直接依存を追加しない、不変条件 4 を維持）。
- `pub use fandhe_frontend_headless_ui::{OpenState, Orientation};`:
  ラッパー呼び出しに頻出する状態値。`fandhe-frontend-docs-site` の実利用
  パス（`fandhe_frontend_headless_ui::{OpenState, Orientation}`）と同型の
  import を pre-styled-ui 単独依存で可能にする。この契約はイシュー #693 で
  実際に消化され、`fandhe-frontend-docs-site` は headless-ui への直接依存
  （`Cargo.toml`・`structure.toml` 双方のエッジ）を撤去して pre-styled-ui
  単独依存へ移行済みである（`crates/docs-site/src/showcase.rs` の import は
  本再エクスポート経由に切り替え済み）。

**セキュリティ上の注意（REQ-1、`.claude/rules/security.md` A03）**:
`fandhe_frontend_pre_styled_ui::fandhe_frontend_core` 経由で `raw_html()` へ
到達できる経路が増えるが、`raw_html()` 自体は既存の明示的オプトイン API
であり、本変更は新たな迂回経路を作らない（headless-ui が #550 で確立した
既存パターンの推移）。pre-styled-ui 内部の不変条件（`raw_html()` の使用は
[`stylesheet::StyleSheet::style_element`] 内の 1 箇所限定）は「使用」に関する
規約であり、`pub use` によるクレート到達性の追加はこれに抵触しない。

固定テストは `crates/pre-styled-ui/tests/headless_reexports.rs`
（import を `fandhe_frontend_pre_styled_ui::` パスのみに限定し、コンパイル
と実行時アサーションの両方で契約を固定する）。

## 4. 設計方針

- **テーマトークン**（#547/#606）: 色・スペーシング等のデザイントークンと
  ダークモード切り替えの基盤。chakra-ui の `system`/`recipe` 相当の設計を
  参考にしつつ、静的 SSR 出力（ビルド時に確定する CSS）を前提とする。
  詳細は `theme` モジュール rustdoc を参照。
- **variant API・静的 CSS 生成**（#548/#606/#604）: chakra-ui の slot
  recipe 相当。コンポーネントの見た目バリエーション（size/variant/
  colorPalette 等）を型安全に選択し、対応する静的 CSS を生成する。詳細は
  [`pre-styled-recipe-api.md`](./pre-styled-recipe-api.md) を参照。
- **styled 部品**（#550/#551/#664/#682/#683/#684）: #550 は Button 等の
  単純な部品、#551 以降は headless-ui の Accordion/Dialog/Popover/
  Tooltip/Switch/RadioGroup/Avatar 等をラップした styled 版を提供する
  （一覧は §2 の表を参照）。

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

## 4b. `avatar`（Avatar の styled ラッパー、イシュー #684）

`fandhe_frontend_headless_ui::avatar`（Root/Image/Fallback の 3 anatomy
パーツと `Avatar` 状態機械）を薄く再利用し、`stylesheet()` で既定 CSS を
追加提供する（設計方針は `crate::dialog`/`crate::tooltip` と同じ、
`src/avatar.rs` 冒頭の rustdoc 参照）。

- **選択的 re-export（`Avatar` 型は再エクスポートしない）**: `fallback`/
  `image`/`AvatarAction`/`ImageStatus` を headless 層からそのまま再
  エクスポートする。styled `root` は本モジュールで variant クラス付与の
  ために再定義するため、`pub use ...::*` ではなく選択的 re-export とする
  （headless の自由関数 `root` との名前衝突を避けるため）。状態機械
  `Avatar` はあえて再エクスポートしない（PR #695 Bugbot 指摘、イシュー
  #684 是正）: `Avatar::root()` は headless 自由関数 `root` へそのまま
  委譲するのみで `size`/`shape` variant クラスを一切付与しないため、
  再エクスポートすると呼び出し側が styled 層のつもりで `Avatar::root()`
  を呼びレイアウトが静かに崩れる事故を誘発する。`Avatar` による状態
  管理・hydration が必要な呼び出し側は
  `fandhe_frontend_headless_ui::avatar::Avatar` を直接 import すること。
- **`root(size, shape, attrs, children) -> Node`**: styled root パーツ。
  `size`（`Size::Sm`/`Md`/`Lg`、既定 `Md`）・`shape`（`AvatarShape::Circle`/
  `Rounded`/`Square`、既定 `Circle`）の 2 軸 variant に応じたクラス
  （`fd-avatar--size-<value>` / `fd-avatar--shape-<value>`）を付与する。
  呼び出し側 `attrs` の `class` は除去してから合成するため `class` 属性は
  常に単一。実体は `fandhe_frontend_headless_ui::avatar::root` へ委譲する
  （呼び出し側 `data-scope`/`data-part` 偽装は headless 側で除去される）。
- **`AvatarShape`**: `recipe::VariantValue` 実装 enum（`Size` と並ぶ本
  クレート 2 例目の variant 軸）。
- **`stylesheet() -> String`**: この styled Avatar の静的 CSS 全量を返す
  （決定的）。`image`/`fallback` の base 規則は `display` を宣言せず、
  headless 層が付与する `hidden` 存在属性（UA 既定 `[hidden] { display:
  none }`）による JS なし SSR の表示制御を壊さない。`data-state="hidden"`
  一致時の `display: none` は `SlotRecipe::state` 経由で多層防御として
  追加登録する（`src/avatar.rs` 冒頭の rustdoc 参照）。

## 4c. styled RadioGroup ラッパー（イシュー #683）

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
  本クレート v0.4.0 へ統合済みのショーケースサンプル（§2 参照）
- `.claude/skills/chakra-ui/`: 設計時の参考にした chakra-ui リファレンス
  スキル

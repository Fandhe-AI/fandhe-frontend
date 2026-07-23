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
冒頭の rustdoc を参照。同じ理由で headless ラッパー第 2 弾（#664:
Popover/Tooltip）・第 3 弾（#682: Switch）も上表へは追加していない。
`switch` モジュール（headless ラッパー第 3 弾、イシュー #682。`size`/
`color-palette` variant 拡張はイシュー #708）は
`fandhe_frontend_headless_ui::switch` の Control/Thumb/Label/HiddenInput
4 パーツを再エクスポートし、`switch::stylesheet()` で `data-state`
（`"checked"`/`"unchecked"`）連動の既定 CSS を追加提供する。styled `root`
は `size`（`Size::Sm`/`Md`/`Lg`、既定 `Md`）・`palette`（`ColorPalette`
5 値、既定 `Accent`）の 2 軸 variant クラスを付与するため本モジュールで
再定義し（`fd-switch--size-<value>` / `fd-switch--color-palette-<value>`）、
`Switch` 状態機械は再エクスポートしない（`avatar` の `Avatar` 非
再エクスポートと同型、詳細は `src/switch.rs` 冒頭の rustdoc を参照）。

`radio_group` モジュール（イシュー #683。`size`/`color-palette` variant
拡張はイシュー #708）も同様に、styled `root` が `size`（既定 `Md`）・
`palette`（既定 `Accent`）に応じたクラス（`fd-radio-group--size-<value>` /
`fd-radio-group--color-palette-<value>`）を付与する。`RadioGroup` 状態機械
は inherent `root()` を持たないため引き続き再エクスポートする（4c 節参照）。

`switch`/`radio_group` いずれも、`size` variant は root スコープの CSS
custom property（`--fandhe-switch-*`/`--fandhe-radio-group-*`）を root へ
登録し、通常の CSS 継承で `control`/`item-control` 等の子孫パーツへ伝える
（`SlotRecipe` へ子孫セレクタ機構は追加していない）。`palette` variant は
既存の `recipe::palette_declarations`（`--fandhe-palette-*`、#606）を
再利用する。

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

## 4c. styled RadioGroup ラッパー（イシュー #683、`size`/`palette` 拡張は #708）

`radio_group` モジュールは `fandhe_frontend_headless_ui::radio_group`
（イシュー #558/#536）の Label/Item/ItemControl/ItemText/ItemHiddenInput
5 anatomy パーツと `RadioGroup` 状態機械を選択的に再エクスポートし、
`stylesheet()` で既定 CSS を追加提供する（設計方針は #551/#664 の他
headless ラッパーと同じ、`src/radio_group.rs` 冒頭の rustdoc 参照）。

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
- **`root(size, palette, disabled, orientation, labelled_by, attrs,
  children) -> Node`**（イシュー #708）: styled root パーツ。`size`
  （`Size::Sm`/`Md`/`Lg`、既定 `Md`）・`palette`（`ColorPalette` 5 値、既定
  `Accent`）の 2 軸 variant クラス（`fd-radio-group--size-<value>` /
  `fd-radio-group--color-palette-<value>`）を付与する。headless 自由関数
  `root` との名前衝突を避けるため本モジュールで再定義し、`pub use
  ...::*` ではなく選択的 re-export とする。`RadioGroup` 状態機械は
  inherent `root()` を持たないため（item 系メソッドのみ）、`avatar` の
  `Avatar` と異なりそのまま再エクスポートを維持する。

## 4d. 複合部品の variant 統一方針・variant 表（イシュー #708）

単純部品（button/badge/spinner）・avatar に続き、headless 状態機械を持つ
複合部品ラッパーへ `size`/`color-palette` variant を拡張する際の統一方針は
`crates/pre-styled-ui/src/lib.rs` 冒頭の rustdoc「複合部品の variant 統一
方針」節が正本。要旨:

1. クラスは root slot のみに付与し、子孫パーツへの伝搬は root が登録する
   CSS custom property の通常の継承で行う（`SlotRecipe` へ子孫セレクタ
   機構は追加しない）。
2. `var()` には Md/Accent 相当のフォールバック値を書き、headless 直接利用
   でも現行外観を維持する。
3. `size` はフォーム操作部品・トリガー系へ、`color-palette` は選択・
   チェック状態を示す部品へ提供する。popover/tooltip は配置・寸法が
   positioning 起因のため提供しない。

| 部品 | size | color-palette | 状態 |
|---|---|---|---|
| button/badge/spinner | ✓ | ✓ | 実装済み（#550/#606） |
| avatar | ✓ | – (shape) | 実装済み（#684） |
| switch | ✓ | ✓ | 実装済み（#708） |
| radio-group | ✓ | ✓ | 実装済み（#708） |
| tabs | 候補 | 候補（selected trigger） | フォローアップ |
| accordion / dialog / menu / select | 候補（size のみ） | – | フォローアップ |
| popover / tooltip | 提供しない | 提供しない | 方針確定 |

## 5. 関連ドキュメント

- [`docs/api/headless-ui-api.md`](./headless-ui-api.md): 本クレートの下層
- [`docs/api/component-api.md`](./component-api.md): `Node`/`el`/`text`/
  `raw_html`/`render` の凍結 API 表面
- [`examples/headless-pre-styled-ui/README.md`](../../examples/headless-pre-styled-ui/README.md):
  本クレート未実装時点での暫定サンプル（pre-styled-ui 統合について節参照）
- `.claude/skills/chakra-ui/`: 設計時の参考にした chakra-ui リファレンス
  スキル

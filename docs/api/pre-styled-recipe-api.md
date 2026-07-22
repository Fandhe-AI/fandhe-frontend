# pre-styled-ui slot recipe API（イシュー #548）

## 1. 目的とトレーサビリティ

本ドキュメントは `fandhe-frontend-pre-styled-ui`（イシュー #520/#546）に実装した
slot recipe 相当の variant API（`SlotRecipe`/`VariantValue`）と静的 CSS 生成の
仕様を記録する。chakra-ui の recipe / slot recipe を参考に、複数 anatomy パーツ
（slot）を横断する variant（size / variant / colorPalette 相当）を型安全な Rust
API（enum ベース）で定義し、クラス名と静的 CSS を決定的に生成する基盤である。

- 親: Phase 3 親イシュー #545、トラッキング #520
- 実装: `crates/pre-styled-ui/src/css.rs`（低レベル宣言・検証・シリアライズ）・
  `crates/pre-styled-ui/src/recipe.rs`（`SlotRecipe`/`VariantValue`/`Size`）
- テスト: `crates/pre-styled-ui/tests/recipe_css.rs`（golden・headless 接続・
  fail-closed）・`crates/pre-styled-ui/tests/recipe_determinism.rs`（決定性）

**本タスクのスコープ**: variant 定義 API（base / variants / defaultVariants 相当）・
静的 CSS 生成・headless 層セレクタとの接続・決定性の担保のみ。テーマトークン・
ダークモード基盤はイシュー #547、styled 部品実装（Button 等 #550・Dialog 等
ラッパー #551）は別イシューのスコープ。

## 2. 公開 API

```rust
// fandhe_frontend_pre_styled_ui::css
pub struct Declaration { /* property, value: &'static str */ }
pub const fn decl(property: &'static str, value: &'static str) -> Declaration;

// fandhe_frontend_pre_styled_ui::recipe
pub trait VariantValue: Copy {
    fn axis(self) -> &'static str;   // 例: "size"
    fn value(self) -> &'static str;  // 例: "sm"
}

pub enum Size { Sm, Md, Lg } // axis = "size"

pub struct SlotRecipe { /* ... */ }
impl SlotRecipe {
    pub const fn new(scope: &'static str, slots: &'static [&'static str]) -> Self;
    pub fn base(self, slot: &'static str, declarations: Vec<Declaration>) -> Self;
    pub fn variant<V: VariantValue>(self, v: V, slot: &'static str, declarations: Vec<Declaration>) -> Self;
    pub fn default_variant<V: VariantValue>(self, v: V) -> Self;
    pub fn css(&self) -> String;
    pub fn variant_class<V: VariantValue>(&self, v: V) -> String;
    pub fn variant_classes(&self, selection: &[(&str, &str)]) -> String;
}
```

`SlotRecipe::new`/`base`/`variant`/`default_variant` は自己消費の builder
（chakra-ui の `defineSlotRecipe({ base, variants, defaultVariants })` 相当の
宣言を、通常の Rust メソッドチェーンで表現する。マクロ DSL は採用しない、REQ-5）。

`colorPalette` 相当は独立の仕組みではなく通常の variant 軸として表現できる
（`docs/api` 掲載の例・`tests/recipe_css.rs` の `ColorPalette` enum 参照）。

`compoundVariants` 相当（複数軸の組み合わせ条件スタイル）は本イシューのスコープ外
（`§5` 参照）。

## 3. `scope` と headless 層との契約

`SlotRecipe::new(scope, slots)` の `scope` は、対応する
`fandhe-frontend-headless-ui` の `Anatomy::new(scope)`（例:
`crates/headless-ui/src/tabs.rs` の `const ANATOMY: Anatomy = anatomy("tabs");`）
と同じ値を渡す契約とする。`slots` は同コンポーネントの anatomy part 名一覧
（Tabs であれば `root`/`list`/`trigger`/`content`）と一致させる。

この契約により、`SlotRecipe::css()` が生成するセレクタ
`[data-scope="<scope>"][data-part="<slot>"]` が、headless 層が
`Anatomy::part()` を通じて実際にレンダリングする属性と一致する
（`crates/pre-styled-ui/tests/recipe_css.rs::base_selectors_match_actual_headless_markup`
が `fandhe_frontend_headless_ui::tabs::tabs()` の実マークアップと照合して固定する）。

## 4. セレクタ・クラス命名規則・出力書式（凍結）

- base セレクタ: `[data-scope="<scope>"][data-part="<slot>"]`（詳細度 (0,2,0)）
- variant セレクタ: `[data-scope="<scope>"][data-part="<slot>"].fd-<scope>--<axis>-<value>`
  （詳細度 (0,3,0)。base に必ず勝つため、CSS 記述順に依存しない上書きを保証する）
- クラス名形式: `fd-{scope}--{axis}-{value}`（prefix `fd` はライブラリ固定。変更用
  API は設けない）
- 出力書式（golden テストの前提、変更しない）:
  - 規則単位: `<selector> {\n  <property>: <value>;\n  ...\n}\n`（インデント 2
    スペース、1 宣言 1 行）
  - 規則間は空行 1 つ
  - `SlotRecipe::css()` 全体の出力順: base（`slots` 宣言順）→ variants（登録順）

## 5. 順序規約・決定性

- 内部ストレージは `Vec` のみ。`HashMap`/`HashSet` は使わない（反復順序がプロセスごとに
  変わりうる型を持ち込まない）
- 同一 slot・同一 axis/value への複数回登録は「後に登録された規則が CSS 中で後に
  出力される」（CSS のカスケードにおいて後勝ちになる）という規約に従う。これより
  複雑な優先順位判定は行わない
- `variant_classes(selection)` は `selection` で指定されなかった axis を
  `default_variant` で補完する。戻り値は axis の登録順（`variant`/`default_variant`
  で最初に現れた順）で連結したクラス文字列
- 決定性は `crates/pre-styled-ui/tests/recipe_determinism.rs` が固定する: 同一入力
  から独立に構築した 2 インスタンスの `css()`/`variant_classes()` が byte 一致する
  こと、同一インスタンスへの繰り返し呼び出しが安定していること

## 6. fail-closed 検証ポリシー

`crates/core/src/lib.rs` が不正なタグ名・属性名を「panic させず出力からスキップ」
する規約（`.claude/rules/coding-rust.md` の panic 回避方針）を踏襲する。

- 識別子（scope / slot / axis / value）: `[a-z][a-z0-9-]*` に一致しない場合、その
  規則・クラスを出力からスキップする
- プロパティ名: 通常のプロパティ名に加えカスタムプロパティ（`--fd-*` プレフィックス）
  を許容する（イシュー #547 のテーマトークン参照 `var(--fd-color-primary)` を
  見越した設計）
- 宣言値: `{` `}` `;` `<` および制御文字を含む場合、その宣言をスキップする。
  `<` の拒否は、下流（styled 部品・examples 等）が生成 CSS を `<style>` へ
  インライン埋め込みした場合の `</style>` 突破（HTML コンテキスト脱出）を防ぐ
  セキュリティ上の不変条件である
- `slots` に宣言していない slot への `base`/`variant` 登録は出力から除外する
- いずれも panic なし・スキップ動作。`crates/pre-styled-ui/tests/recipe_css.rs::invalid_identifiers_and_structural_chars_are_skipped_not_panicking`
  が固定する

## 7. `#547`（テーマトークン）との関係

宣言値は不透明な `&'static str` として扱うため、トークン参照は
`decl("color", "var(--fd-color-primary)")` のような値として自然に載る。
本イシューの実装は `#547` の API へコード依存を持たない。

## 8. スコープ外（Issue 化候補）

- `compoundVariants` 相当（複数軸の組み合わせ条件スタイル）
- recipe 出力の CSS ファイル書き出し・`<style>` 埋め込みヘルパ（#550/#552 で
  必要性を判断）
- `#547` テーマトークンとの palette 実配線（colorPalette 軸の意味付け）

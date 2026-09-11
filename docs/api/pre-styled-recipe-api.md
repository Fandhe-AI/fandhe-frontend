# pre-styled-ui slot recipe API

## 1. 目的とトレーサビリティ

本ドキュメントは `fandhe-frontend-pre-styled-ui` に実装した slot recipe
相当の variant API（`SlotRecipe`/`VariantValue`）と静的 CSS 生成の仕様を
記録する。chakra-ui の recipe / slot recipe を参考に、複数 anatomy パーツ
（slot）を横断する variant（size / variant / colorPalette 相当）を型安全な Rust
API（enum ベース）で定義し、クラス名と静的 CSS を決定的に生成する基盤である。

- 実装: `crates/pre-styled-ui/src/css.rs`（低レベル宣言・検証・シリアライズ）・
  `crates/pre-styled-ui/src/recipe.rs`（`SlotRecipe`/`VariantValue`/`Size`、
  compoundVariants 相当は `VariantCondition`/`when`/`SlotRecipe::compound_variant`）
- テスト: `crates/pre-styled-ui/tests/recipe_css.rs`（golden・headless 接続・
  fail-closed・compound variant）・`crates/pre-styled-ui/tests/recipe_determinism.rs`
  （決定性、compound variant を含む）

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

// 標準 colorPalette 軸。crate::theme のセマンティック色
// （accent/info/success/warning/danger）と 1:1 対応する。
pub enum ColorPalette { Accent, Info, Success, Warning, Danger } // axis = "color-palette"
pub fn palette_declarations(p: ColorPalette) -> Vec<Declaration>;

pub struct SlotRecipe { /* ... */ }
impl SlotRecipe {
    pub const fn new(scope: &'static str, slots: &'static [&'static str]) -> Self;
    pub fn base(self, slot: &'static str, declarations: Vec<Declaration>) -> Self;
    pub fn variant<V: VariantValue>(self, v: V, slot: &'static str, declarations: Vec<Declaration>) -> Self;
    pub fn default_variant<V: VariantValue>(self, v: V) -> Self;
    pub fn compound_variant(self, conditions: Vec<VariantCondition>, slot: &'static str, declarations: Vec<Declaration>) -> Self;
    pub fn pseudo_element(self, slot: &'static str, pseudo: PseudoElement, declarations: Vec<Declaration>) -> Self;
    // `@starting-style { ... }` 規則の登録（イシュー #2192）。condition なし/
    // StateCondition 条件付きの 2 形態。Hover 系条件は css() が fail-closed に
    // 除外する（§6 参照）。
    pub fn starting_style(self, slot: &'static str, declarations: Vec<Declaration>) -> Self;
    pub fn starting_style_state(self, slot: &'static str, condition: StateCondition, declarations: Vec<Declaration>) -> Self;
    // headless の disclosure 系 content パートへ --fandhe-content-height 連動の
    // 高さトランジションを 1 呼び出しで適用する preset（イシュー #2192）。
    // base（content_height_open_declarations）・[hidden] state
    // （content_height_closed_declarations）・starting_style（同左）を一括登録する。
    pub fn content_height_transition(self, slot: &'static str, duration: MotionDuration) -> Self;
    pub fn css(&self) -> String;
    pub fn variant_class<V: VariantValue>(&self, v: V) -> String;
    pub fn variant_classes(&self, selection: &[(&str, &str)]) -> String;
}

// compoundVariants 相当
pub struct VariantCondition { /* axis, value: &'static str（型消去済み） */ }
pub fn when<V: VariantValue>(v: V) -> VariantCondition;

// 疑似要素（イシュー #2201）
pub enum PseudoElement { Before, After } // ::before / ::after

// transition/starting-style 関連ヘルパ（イシュー #1425/#2192）
pub fn transition_declarations(properties: &'static str, duration: MotionDuration) -> Vec<Declaration>;
// transition_declarations の 3 宣言 + transition-behavior: allow-discrete。
// properties に display を含めるのが典型用途（hidden 属性による display: none
// の実適用を遷移完了まで遅延させる、MDN transition-behavior）。
pub fn transition_declarations_allow_discrete(properties: &'static str, duration: MotionDuration) -> Vec<Declaration>;

// fandhe-frontend-wasm-full が実測高さを書き込む CSS custom property 名の写し
// （crates/wasm-full/src/content_height.rs::CONTENT_HEIGHT_VAR とのドリフトは
// tests/content_height_var_drift.rs が fail-closed に検知する）。
pub const CONTENT_HEIGHT_VAR: &str = "--fandhe-content-height";
pub fn content_height_open_declarations(duration: MotionDuration) -> Vec<Declaration>;
pub fn content_height_closed_declarations() -> Vec<Declaration>;
```

`SlotRecipe::new`/`base`/`variant`/`default_variant` は自己消費の builder
（chakra-ui の `defineSlotRecipe({ base, variants, defaultVariants })` 相当の
宣言を、通常の Rust メソッドチェーンで表現する。マクロ DSL は採用しない、REQ-5）。

`colorPalette` 相当は独立の仕組みではなく通常の variant 軸として表現できる
（`docs/api` 掲載の例・`tests/recipe_css.rs` の `ColorPalette` enum 参照）。

`compoundVariants` 相当（複数軸の組み合わせ条件スタイル）は
[`SlotRecipe::compound_variant`] で表現する。条件部は [`when()`] で
[`VariantValue`] 実装 enum から作った [`VariantCondition`] の `Vec`（AND
条件）として渡す:

```rust
recipe.compound_variant(
    vec![when(Size::Sm), when(ColorPalette::Red)],
    "trigger",
    vec![decl("font-weight", "bold")],
)
```

生の文字列ではなく `when()` を介した enum ベースの構築のみを許すことで、
`variant()` と同じ型安全性を条件部でも保つ（`tests/recipe_css.rs` の
`tabs_recipe()` 参照）。

疑似要素（`::before`/`::after`）は [`SlotRecipe::pseudo_element`] で表現する
（イシュー #2201）:

```rust
recipe.pseudo_element(
    "handle",
    PseudoElement::After,
    vec![decl("position", "absolute")],
)
```

`declarations` に `content` プロパティが含まれない場合は `content: "";` が
自動的に先頭へ前置される（疑似要素はブラウザの既定で `content` が無いと
ボックスを生成しないため）。呼び出し側が `content` を渡した場合は二重化せず
そのままの順序を使う。`content` の不正な値（`<`/`;`/`{`/`}`/制御文字を含む）
はその宣言のみが除外され、既定値の再注入は行わない（§6 参照）。

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

`scope` はセレクタ・クラス名へ `slot`/`axis`/`value` と同様にそのまま埋め込まれるため、
`SlotRecipe::css()`/`variant_class()`/`variant_classes()` はいずれも呼び出し時に
`scope` を `is_valid_identifier`（`css.rs`）で検証し、不正な場合は空文字列を
fail-closed で返す（`slot`/`axis`/`value` 側の検証だけでは `scope` 経由の
セレクタ脱出・`</style>` 混入を防げないため、`scope` にも同じ検証を適用する）。

## 4. セレクタ・クラス命名規則・出力書式（凍結）

- base セレクタ: `[data-scope="<scope>"][data-part="<slot>"]`（詳細度 (0,2,0)）
- variant セレクタ: `[data-scope="<scope>"][data-part="<slot>"].fd-<scope>--<axis>-<value>`
  （詳細度 (0,3,0)。base に必ず勝つため、CSS 記述順に依存しない上書きを保証する）
- compound variant セレクタ:
  `[data-scope="<scope>"][data-part="<slot>"].fd-<scope>--<a1>-<v1>.fd-<scope>--<a2>-<v2>...`
  （`conditions` の登録順に条件クラスを連結する。新しいクラス名は生成せず、
  `variant_classes()` が emit する既存の軸別クラスの共起にセレクタとして
  反応するだけなので、HTML 側への影響はない）
- 疑似要素セレクタ（イシュー #2201）:
  `[data-scope="<scope>"][data-part="<slot>"]::before` /
  `[data-scope="<scope>"][data-part="<slot>"]::after`（結合子を含まない、
  同一要素上の複合セレクタの末尾への付加のみ。`SlotRecipe` は子孫/子/隣接
  結合子を生成する経路を持たないという既存方針〔イシュー #708〕を変更しない）
- クラス名形式: `fd-{scope}--{axis}-{value}`（prefix `fd` はライブラリ固定。変更用
  API は設けない）
- 出力書式（golden テストの前提、変更しない）:
  - 規則単位: `<selector> {\n  <property>: <value>;\n  ...\n}\n`（インデント 2
    スペース、1 宣言 1 行）
  - 規則間は空行 1 つ
  - `SlotRecipe::css()` 全体の出力順: base（`slots` 宣言順）→ variants
    （登録順）→ compound variants（登録順）→ states（登録順。`Hover` 系は
    `@media (hover: hover)` へ集約され末尾に回る）→ pseudo-elements
    （登録順、イシュー #2201）→ `@starting-style`（登録順、1 個の
    ブロックへ集約、イシュー #2192）→ `@supports not (height: calc-size(auto,
    size))`（登録順、1 個のブロックへ集約、イシュー #2192）→ breakpoints
    （[`Breakpoint`] の昇順、イシュー #2197）→ `@media (hover: hover) { ... }`
    （`Hover` 系 state が存在する場合のみ、常に出力全体の末尾）

### 4.1 compound variant の上書き保証（2 段）

- 条件 2 個以上: セレクタの詳細度が (0,4,0) 以上となり、単一 variant
  セレクタ (0,3,0) に記述順へ依存せず必ず勝つ
- 条件 1 個: 詳細度は単一 variant と同じ (0,3,0) だが、compound ブロックを
  variants ブロックより後に出力するため CSS カスケードの後勝ちで上書きされる

chakra-ui の「compoundVariants は variants を上書きする」という意味論に
この 2 段の保証で対応する。

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
  こと、同一インスタンスへの繰り返し呼び出しが安定していること（compound variant
  を含む場合も同様）

## 6. fail-closed 検証ポリシー

`crates/core/src/lib.rs` が不正なタグ名・属性名を「panic させず出力からスキップ」
する規約（`.claude/rules/coding-rust.md` の panic 回避方針）を踏襲する。

- 識別子（scope / slot / axis / value）: `[a-z][a-z0-9-]*` に一致しない場合、その
  規則・クラスを出力からスキップする
- プロパティ名: 通常のプロパティ名に加えカスタムプロパティ（`--fd-*` プレフィックス）
  を許容する（テーマトークン参照 `var(--fd-color-primary)` を見越した設計）
- 宣言値: `{` `}` `;` `<` および制御文字を含む場合、その宣言をスキップする。
  `<` の拒否は、下流（styled 部品・examples 等）が生成 CSS を `<style>` へ
  インライン埋め込みした場合の `</style>` 突破（HTML コンテキスト脱出）を防ぐ
  セキュリティ上の不変条件である
- `slots` に宣言していない slot への `base`/`variant`/`compound_variant` 登録は
  出力から除外する
- compound variant 固有の検証（`crates/pre-styled-ui/tests/recipe_css.rs::compound_variant_fail_closed_cases_are_skipped_not_panicking`
  が固定する）:
  - `conditions` が空の規則は base と同義になる無意味な規則として除外する
  - `conditions` 内に同一 axis が重複する規則は、`variant_classes()` が 1 軸
    につき高々 1 クラスしか emit しないため決して同時に一致しない矛盾条件で
    あるとみなし、dead CSS の混入防止として除外する
  - 条件の `(axis, value)` の組が `variant()`/`default_variant()` のいずれにも
    未登録の規則は、axis/value のタイポによる dead CSS の混入防止として除外する
    （検証は `css()` 呼び出し時に行うため builder の呼び出し順には依存しない）
- いずれも panic なし・スキップ動作。`crates/pre-styled-ui/tests/recipe_css.rs::invalid_identifiers_and_structural_chars_are_skipped_not_panicking`
  が固定する
- pseudo-element 固有の検証（イシュー #2201、
  `crates/pre-styled-ui/tests/recipe_css.rs::pseudo_element_fail_closed_cases_are_skipped_not_panicking`
  が固定する）:
  - `slots` に宣言していない slot、または識別子として不正な slot への登録は除外する
  - `declarations` が空の規則は無意味な規則として除外する（`::after { content: "" }`
    だけの dead CSS を混入させない）
  - `content` の値が不正（`<`/`;`/`{`/`}`/制御文字を含む）な場合は `content` 宣言
    のみを除外する。他の宣言が有効であれば規則自体は出力される。**既定値の
    再注入は行わない**（不正な値を親切に空文字列へ差し替えると、fail-closed の
    意味が薄れるため）
- `StateCondition::AttrEqAll`（値付き属性の AND）・`StateCondition::AttrAll`
  （値なし存在属性の AND、イシュー #2203）固有の検証:
  - スライスが空の規則は base と同義になる無意味な規則として除外する
  - スライス要素（`AttrEqAll` は属性名・属性値、`AttrAll` は属性名）が識別子
    として不正な規則は除外する
  - `crates/pre-styled-ui/tests/recipe_css.rs::state_attr_all_fail_closed_cases_are_skipped_not_panicking`
    が固定する
- `starting_style_state` の Hover 系 `StateCondition`（`Hover`/`HoverExcept`/
  `HoverExceptAttr`/`HoverExceptAttrEq`）は `@starting-style` 内で意味を持たない
  （starting style は遷移開始前の静的スナップショットであり、`:hover` の
  ような動的擬似クラスの「開始状態」という概念が成立しないため）ため、規則
  ごと除外する（fail-closed、イシュー #2192。
  `crates/pre-styled-ui/tests/recipe_css.rs::starting_style_fail_closed_cases_are_skipped_not_panicking`
  が固定する）

## 7. テーマトークンとの関係

宣言値は不透明な `&'static str` として扱うため、トークン参照は
`decl("color", "var(--fd-color-primary)")` のような値として自然に載る。

colorPalette 軸実配線時、`palette_declarations` は `crate::theme` が生成する
`--fandhe-color-*`（テーマ層の名前空間）とは別の `--fandhe-palette-*` 名前空間へ、
選択された palette に対応する `accent`/`info`/`success`/`warning`/`danger` の
3 役割（base/emphasized/fg）を `var()` 参照として束ねる。styled 部品
（Button/Badge/Spinner、`crates/pre-styled-ui/src/button.rs` 等）は
`var(--fandhe-palette)` 等を参照するだけで、`palette` variant の選択に応じて
色が切り替わる。名前空間を分離しているため、ユーザーがカスタムテーマへ
`Theme::push_color("palette", ...)` のような独自トークンを追加しても
`--fandhe-palette-*` の生成とは衝突しない。既定トークンの上書きは
`push_*` ではなく `upsert_*` を使う（詳細は
[`pre-styled-ui-api.md`](./pre-styled-ui-api.md) §4l）。

`crate::theme` は加えて radii（`--fandhe-radius-<name>`）・shadow
（`--fandhe-shadow-<name>`、light/dark 2 値）トークングループを持つ。
styled 部品は `border-radius`/`box-shadow` の値としてこれらを参照する
（例: `decl("border-radius", "var(--fandhe-radius-md)")`）。

## 関連ドキュメント

- [`docs/api/pre-styled-ui-api.md`](./pre-styled-ui-api.md): 本 API の上層
  （styled 部品・`stylesheet::StyleSheet`）
- `docs/internal/pre-styled-recipe-implementation-notes.md`: 実装経緯・
  スコープ外事項・トレーサビリティの記録（docs サイト非掲載のためリンク化
  しない）

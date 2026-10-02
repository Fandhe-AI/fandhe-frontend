//! styled Field（イシュー #1684、親 #1671、祖父トラッキング #520）。
//!
//! `fandhe_frontend_headless_ui::field`（#538/#602）が出力する
//! `data-scope="field"` の anatomy へ、ラベル・補助テキスト・エラーテキスト・
//! 必須マークの型階層と `root` の余白レイアウトを重ねる薄い委譲層である。
//!
//! # スコープ（本イシューで実装するもの／しないもの）
//!
//! 本モジュールが宣言する slot は `root`/`label`/`helper-text`/`error-text`/
//! `required-indicator` の 5 つのみで、**`input`/`textarea`/`select` は
//! 宣言しない**。これらのコントロールパーツは既に [`crate::input`](mod@crate::input)/
//! [`crate::textarea`](mod@crate::textarea)/[`crate::native_select`](mod@crate::native_select) が recipe scope `"field"` を
//! 共有しつつ独占的に所有している（[`crate::input`](mod@crate::input) モジュール doc
//! 「`field` scope を共有する理由」節参照）。本モジュールが `input`/
//! `textarea`/`select` slot へ base 宣言を追加登録すると、集約 stylesheet
//! （`crate::stylesheet::all_styled_component_css`）中に同一セレクタの
//! base ブロックが二重出現しカスケードを汚すため、意図的に宣言しない。
//! （例外: イシュー #3134 の `Inset` ラベル配置は、`SlotRecipe` の base/
//! variant ではなく `.fd-field--label-placement-inset > [data-part="input"]`
//! という `root` 側クラスを条件にした**子孫セレクタでの上書き**を 1 件だけ
//! 持つ。これは `input` slot の base を再登録するものではなく、既存の
//! `crate::input::css` が出力する base/variant 宣言を opt-in 時のみ
//! 特異度で上書きする追記であり、二重出現にはならない。）
//!
//! docs サイトへの `/themes/field/` ページ登録（showcase Demo・
//! `SPEC_TABLES` 原稿・`site/nav.toml`）は #1685 で実施済み。本モジュールは
//! recipe のみを提供し、ページ側は `crates/docs-site` が担う。
//!
//! # 責務境界（`docs/policy/intentional-non-adoption.md` §3.25 規則 1）
//!
//! バリデーション処理（値の妥当性判定・送信処理）は実装しない。headless
//! [`fandhe_frontend_headless_ui::field`] が出力する `data-invalid`/
//! `data-disabled`/`data-required` を CSS セレクタとして**参照するだけ**で
//! 見た目を切り替える（`docs/design/pre-styled-ui-data-attr-vocabulary.md`
//! §3.1 規約 A・役割 B）。本モジュール自身は独自の `data-*` を一切出力しない。
//!
//! # 状態機械を持たない理由
//!
//! headless [`fandhe_frontend_headless_ui::field`] 自身が「props から決定的
//! にマークアップを組み立てる純粋関数群」（状態機械なし）として実装されて
//! いるため、本モジュールもその設計をそのまま継承する（[`crate::input`](mod@crate::input)
//! モジュール doc と同型の判断）。
//!
//! # variant 軸: `orientation` のみ
//!
//! [`FieldOrientation`]（既定 `Vertical`）のみを提供する。`size` 軸は持たない
//! （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` の保有
//! 判定基準: 子の寸法に従属するレイアウト部品の root は size 軸を持たない。
//! ラベル・補助テキスト・エラーテキストの文字サイズは固定の型階層で表現する）。
//! `color-palette` 軸も持たない（フォーム入力系は非提供、[`crate::input`](mod@crate::input)
//! と同じ判断）。
//!
//! # 意図的非採用（参考サイト比較、chakra-ui v3 Field / ark-ui Field）
//!
//! - **hover**: `root`/`label` はインタラクティブ slot（`cursor: pointer`）
//!   ではないため付与しない。
//! - **focus ring**: 実フォーカスはコントロール（input 等）側にあり、
//!   [`crate::input`](mod@crate::input) 等が既に focus ring を所有する。
//! - **transition**: 状態遷移に伴う視覚変化がないため付与しない。
//! - **`data-readonly` によるラベル色変更**: chakra-ui v3 も持たない。
//!   readonly はコントロール側の見た目のみで伝える。
//! - **`data-required` への CSS**: 表示切替は headless `required_indicator`
//!   の `hidden` 属性フリップが担う。本モジュールは `[hidden]` を
//!   `display: none` にする規則のみを持つ。
//! - **ErrorIcon・`Field.Item` パーツ**: headless [`fandhe_frontend_headless_ui::field`]
//!   の anatomy に存在しないため実装しない（headless anatomy 変更はスコープ
//!   外、#1671 側で扱う）。
//!
//! # shadcn/ui 突合（イシュー #2014）
//!
//! 親トラッキング #2008（Phase 1: Forms）の一環として、shadcn/ui の
//! `field.tsx`（`FieldSet`/`FieldLegend`/`FieldGroup`/`Field`/
//! `FieldContent`/`FieldLabel`/`FieldTitle`/`FieldDescription`/
//! `FieldSeparator`/`FieldError` の 10 サブコンポーネント）を**補完参照**
//! として突合した（#2001 Phase 0 の適用原則: shadcn/ui の視覚言語へ置き換
//! えることは目的としない。2026-09-07 のユーザー判断〔イシュー #2153、
//! `docs/design/shadcn-reference-adoption-policy.md` §8〕で shadcn/ui は
//! 主基準の 1 つへ改訂されたが、この判断は改訂後も不変）。
//!
//! ## 採用したもの
//!
//! - **`label` の `data-invalid` 反応**: 上記「意図的非採用」節が以前
//!   記していた「`data-invalid` によるラベル色変更: chakra-ui v3 も
//!   持たない」という判断を、shadcn の `Field` が
//!   `data-[invalid=true]:text-destructive` を持つことを踏まえて是正した。
//!   既存の `error-text` 表示切替と矛盾せずエラー視認性を追加的に高める
//!   ため、`label[data-invalid]` の文字色を `--fandhe-color-danger` へ
//!   切り替える規則を追加した。WCAG 1.4.1 は「色のみに依存しない伝達
//!   手段」を要求するが、`error-text` のテキストによる代替伝達が既にある
//!   ため抵触しない。
//! - **`error-text` 直下 `<ul>` の複数エラーメッセージ表示**: shadcn の
//!   `FieldError` は複数エラーを重複排除して `<ul class="list-disc">` で
//!   表示する。headless [`fandhe_frontend_headless_ui::field::error_text`]
//!   は children をそのまま透過する薄いラッパーのため、呼び出し側が
//!   `<ul>`/`<li>` を children として渡す運用に対応する CSS
//!   （`error-text > ul` のリスト整形）のみを追加した（重複排除・`<ul>`/
//!   `<li>` の組み立て自体は本モジュールの責務外のまま、呼び出し側が担う）。
//!   **追補（PR #2147 codex-review/Cursor Bugbot 指摘の是正）**: 当初の
//!   実装は headless 側が `error-text` を `<span>` で出力しており、`<ul>`
//!   を children に渡すと `<span><ul>...</ul></span>` という HTML コンテン
//!   ツモデル違反（`span` の phrasing content は `ul` を含められない）に
//!   なっていた。headless 側の anatomy 変更はスコープ外という上記整理を
//!   本件では見送り、[`fandhe_frontend_headless_ui::field::error_text`] の
//!   出力タグを `span` から `div` へ変更（0.65.0 → 0.66.0、破壊的変更）
//!   することで是正した（`fieldset::error_text` も同型のため同時変更）。
//!   加えて `error-text > ul` の CSS が `display: flex` と
//!   `list-style: disc` を併用しており、flex item となった `<li>` が
//!   `display: list-item` を失い `::marker` が生成されない不具合（disc
//!   マーカー非表示）があったため、`<ul>` を block（UA 既定）のまま
//!   `<li>` 間の縦間隔を隣接兄弟セレクタの `margin-top` で表現する形へ
//!   変更した（[`css`] 参照）。
//! - **`FieldError` の `role="alert"`**（イシュー #2184）: headless
//!   `error_text` が `role="alert"` と明示 `aria-live="polite"` を併せて
//!   出力するようになった（`aria-live="polite"` は据え置き。判断根拠は
//!   [`fandhe_frontend_headless_ui::field`] モジュール doc「`role="alert"`
//!   の採用（イシュー #2184）」節を参照）。本モジュールは headless の
//!   `error_text` を再エクスポートするのみで CSS 側の変更は不要。
//!
//! ## 見送ったもの（記録のみ、Issue 化はユーザー承認前提のため未実施）
//!
//! - **`FieldGroup`/`FieldContent`/`FieldTitle`/テキスト付き
//!   `FieldSeparator`**: 対応する headless anatomy が存在せず、新設するには
//!   headless-ui 拡張または独立 anatomy 新設のいずれかが必要で、本イシュー
//!   の粒度（既存 5 slot の recipe 補修）を超えるため不採用。
//! - **`FieldLegend` の `legend`/`label` 2 段見出しサイズ**: `fieldset.rs`
//!   は本イシューの対象ファイル外（#2014 の対象は `field.rs` のみ）だった。
//!   イシュー #2214 で `fieldset.rs` 側（`legend_with_variant`/
//!   `LegendVariant`）に実装済み。
//! - **choice card（label が checkbox/radio を包む形）**: [`crate::checkbox_card`]/
//!   [`crate::radio_card`] が既に独立 anatomy として提供済みであり
//!   `field.rs` 側の対応は不要（gap なし）。
//!
//! ## 採用したもの（イシュー #2185、上記「見送ったもの」からの是正）
//!
//! headless-ui 側で `group`/`content`/`title`/テキスト付き `separator`
//! （内部パーツ `separator-line`/`separator-content`）の anatomy が新設
//! （イシュー #2185）されたことを受け、本モジュールも 6 slot を追加登録
//! して着装する（`SLOTS` 末尾に純追加、既存 5 slot の base/state 出力
//! バイトは変更しない）。
//!
//! - **`group` の間隔**: shadcn/ui の合成（複数 `Field` を縦積みし線区切り
//!   する構成、`docs/design/reference-screenshots/shadcn-field-1.png`）を
//!   採るが、値は本リポジトリの space トークン
//!   （`--fandhe-space-6` = 1.5rem。shadcn `gap-7` = 1.75rem に最も近い
//!   段）を採る。理由: トークン外の生 `rem` 値は #1423 のスケール決定に
//!   反する。
//! - **`separator` の線描画**: `border-*`（chakra-ui 方式）を採る。理由:
//!   [`crate::separator`](mod@crate::separator)（#2053）が既に持つ `--fandhe-separator-thickness`
//!   の上書き契約を共有し、区切り線の太さ制御を一貫させるため。線は実要素
//!   `separator-line`（`hr`）の `border-top` で描き、擬似要素は使わない
//!   （[`StateCondition`] が擬似要素セレクタを表現できないため）。
//! - **`title`**: `label` と同じ型階層（サイズ・太さ・行間・配色）を採る。
//!   `<label for>` を結べない場面の見出しとして視覚的に同格に見せるため。
//! - **`content`**: disabled 時の減光を付与しない（子の `label`/`title`/
//!   `helper-text` が既に減光するため、コンテナ自身が二重に薄くなるのを
//!   避ける）。
//!
//! 参照スクショについての注記: イシュー本文が指す番号と実ファイルが食い
//! 違うため、Examples（docs-site 側）では `shadcn-field-1.png`
//! （Payment Method フォーム: 複数 field の縦積み・区切り線）と
//! `shadcn-field-2.png`（Username/Password の content レイアウト）の両方を
//! 参照する。
//!
//! ## `orientation="responsive"`（`@container` クエリ、イシュー #2199、
//! 上記「見送ったもの」からの是正）
//!
//! [`crate::recipe`] に `@container` 条件の宣言手段（[`crate::recipe::
//! SlotRecipe::container_slot`]/[`crate::recipe::SlotRecipe::container`]/
//! [`crate::recipe::SlotRecipe::container_variant`]）が新設されたことを
//! 受け、`group`（イシュー #2185 で追加した slot）を
//! [`crate::recipe::SlotRecipe::container_slot`] として宣言し、`root` へ
//! [`FieldOrientation::Responsive`] を追加した。`group` の inline サイズが
//! [`crate::recipe::ContainerBreakpoint::Md`]（448px、shadcn/ui
//! `@md/field-group` 相当）以上のときのみ `Horizontal` と同じ宣言
//! （`horizontal_root_declarations`）を `root` へ適用する。`group` の
//! **外**に `Responsive` な `root` を置いた場合、container が存在しないため
//! 常に縦積みのまま（`@container` は無条件で不一致になる。mobile-first の
//! 安全な劣化）。
//!
//! 受け入れ条件が述べる `data-orientation="responsive"` は実装していない:
//! headless `field::root`（`fandhe_frontend_headless_ui::field`）は
//! `data-orientation` を意図的に持たず、本モジュールも独自 `data-*` を出力
//! しない契約（`crates/pre-styled-ui/tests/data_attr_vocabulary.rs`）のため、
//! 既存の `Vertical`/`Horizontal` と同じくクラス
//! `fd-field--orientation-responsive` として語彙化した（PR 本文にこの読み
//! 替えを明記する）。
//!
//! ## `helper-text` の `text-wrap: balance`（イシュー #2160、上記
//! 「見送ったもの」からの是正）
//!
//! 上記「見送ったもの」節はかつて「実現には `list.rs` 型の手書き
//! descendant セレクタ追記が必要」を理由に不採用としていたが、その追記
//! 手段自体は shadcn/ui 突合（イシュー #2014）で `error-text > ul` の
//! リスト整形として既に実装済みであり、技術的障壁は解消していた。加えて
//! 2026-09-07 のユーザー判断で shadcn/ui が pre-styled-ui の主基準の 1 つへ
//! 格上げされ（`docs/design/shadcn-reference-adoption-policy.md` §8）、
//! 「新規追加分は shadcn の値を採ってよい」が確定したため、本イシューで
//! 採用へ転じた。
//!
//! 参照競合の判定: field の helper-text（horizontal 時の折り返し）は
//! shadcn-ui の値（`text-wrap: balance`）を採る。理由: chakra-ui v3
//! `field` recipe の `helperText` slot（`color: fg.muted`/`textStyle: sm`
//! 程度）には `text-wrap` 指定がなく、Radix Themes は Field 相当の部品を
//! 持たないため、いずれとも競合しない。既存 variant の CSS 出力は
//! バイト同一のまま末尾への純追加で実現でき、未対応ブラウザでは宣言が
//! 無効値として破棄されるだけで劣化しない（`text-wrap: balance` は
//! Baseline 2024 相当、Chromium 114+/Firefox 121+/Safari 17.5+）。
//!
//! 実装は shadcn の `group-has-[[data-orientation=horizontal]]/field`
//! （祖先条件）を、本モジュールの語彙へ 2 点読み替えて表す:
//!
//! - **`data-orientation` は使わない**: headless `field::root` は
//!   `data-orientation` を意図的に持たず、本モジュールも独自 `data-*`
//!   を出力しない契約のため、上記「`orientation="responsive"`」節と同じ
//!   クラス `fd-field--orientation-horizontal`/
//!   `fd-field--orientation-responsive` を条件に使う。
//! - **結合子は子孫（空白）**: `helper-text` は `content` の内側に置かれ
//!   得る（上記「採用したもの（イシュー #2185）」節）ため `root` の直下
//!   とは限らない。shadcn の祖先条件も子結合子ではないため、この点は
//!   読み替えではなく shadcn 自身の構造と一致する。
//!
//! [`SlotRecipe`] の `state()`/`container_variant()` は同一 slot 内条件
//! のみ表現できるため、`error-text > ul` と同型の直接追記（[`css`] 末尾へ
//! 静的 CSS 文字列を連結）で表す。`responsive`（[`FieldOrientation::
//! Responsive`]）も対象に含める: rustdoc・docs-site 原稿はいずれも
//! `responsive` を「`group` container が 448px 以上で `horizontal` と同じ」
//! と定義しており、horizontal のみへ追加すると記述が事実と食い違うため
//! （`@container fd-field-group (min-width: 448px)` の内側に同一宣言の
//! `.fd-field--orientation-responsive` 版を静的追記する）。
//!
//! # ラベル配置（イシュー #3134）
//!
//! Blocks 取り込みの対応表で「既存部品では表現できない」と判定された
//! 2 レイアウトを、`orientation` とは独立な opt-in variant
//! [`FieldLabelPlacement`] として追加する。既定（[`FieldLabelPlacement::
//! Outside`]）は従来どおり `root()` の出力と完全に一致し（golden は末尾への
//! 純追加のみ）、[`root_with_label_placement`] を明示的に呼んだ場合のみ
//! クラスが 1 つ付与される。
//!
//! - **`Inset`**: ラベルを枠の内側・上部に置く。`root` 自身が枠線・背景を
//!   持つ box になり、`label`/`input` を内包する。[`inset_stack`]
//!   （`data-part="inset-stack"` の wrapper）の直下に縦に並べたときだけ、
//!   隣接する `root` 同士が枠線を共有して連結する。連結規則は wrapper を
//!   条件にするため、`group` のような `gap` を持つコンテナへ直接並べても
//!   発動しない（隣接兄弟セレクタ `+` は親の `gap` を区別できないため、
//!   暗黙の隣接ではなく明示の wrapper を契約とする。codex-review 指摘、
//!   PR #3570）。
//! - **`Overlap`**: `root` 自身に枠線を持たせ、ラベルをその枠線の上へ
//!   絶対配置で重ねる。`--fandhe-field-label-bg`（既定
//!   `var(--fandhe-color-bg)`）でラベル背景を地の色に合わせて上書きできる。
//!
//! ## 対象外（範囲外として明示）
//!
//! - inset がリセットする対象は `input` のみ。`textarea`/`select` は対象外
//!   （必要になれば同型の宣言を同じ条件で追加できる）。
//! - `orientation = Horizontal`/`Responsive` との併用、`forms_motion` の
//!   floating label との併用はいずれも対象外とし、`Vertical` での使用の
//!   みを前提とする。
//! - inset の内側に `helper-text`/`error-text` を置くと枠の中に表示される
//!   （ARIA は id 結び付けのため、枠の外に出したい場合は root の外に置く）。
//!
//! ## 状態表示の対応表
//!
//! `Inset`/`Overlap` はどちらも枠線を `root` が描くため、`crate::input` が
//! `input` 自身へ与える状態表示のうち枠線に結び付くもの（invalid の
//! `border-color`・focus のリング）をそのままにすると、枠のない `input` の
//! 矩形に沿って表示されて `root` の枠と二重になる。そこで両
//! variant で同一の規則集合（`boxed_root_state_css`）を `root` 側へ持ち、
//! `input` 側の同種宣言を子孫セレクタで打ち消す。
//!
//! | 状態 | `root`（Inset/Overlap 共通） | `input` 側の打ち消し |
//! |------|------------------------------|----------------------|
//! | `data-invalid` | `border-color: danger`（枠線全体） | なし（`input` は枠線を持たないため不要） |
//! | `data-disabled` | 宣言なし（子の `label`/`helper-text`/`input` が各自 0.5 へ減衰済み。root にも当てると 0.25 の二重減衰になる） | なし |
//! | `data-readonly` | 視覚宣言なし（`crate::input` の意図的非採用に揃える） | なし |
//! | `:focus-within` | 枠線の内側にリング（`FocusRingOffset::Inset`） | `:focus-visible { outline: none }` |
//!
//! 状態の重なりは別プロパティなので干渉しない: invalid + focus は
//! エラー色の枠線 + リング（`crate::input` の挙動と同じ）、disabled は
//! フォーカス不可のため focus と重ならず、invalid + disabled はエラー色の
//! 枠線のまま子パーツだけが減衰する。[`inset_stack`] 内の縦連結では共有する
//! 辺が先行要素の `border-bottom` になるため、後続要素だけが invalid の
//! ときは `:has(+ ...[data-invalid])` で先行側の下辺をエラー色にする
//! （focus/disabled は枠線色を変えないため、写すべき状態は invalid のみ）。
//!
//! ## 意図的非採用（focus ring の例外）
//!
//! モジュール doc 冒頭「意図的非採用」節は「実フォーカスはコントロール
//! 側にあるため `root` は focus ring を持たない」としているが、`Inset`/
//! `Overlap` はこの例外である: いずれも `input` 自身の `outline` を消す
//! ため、代わりに `root` の `:focus-within` へフォーカスリングを付け、
//! キーボードでのフォーカス表示が失われないようにする。
//!
//! # セキュリティ不変条件
//!
//! - 全出力は `fandhe_frontend_core::el`/`fandhe_frontend_core::text`
//!   （headless 層経由）を通り、`fandhe_frontend_core::render` の既定
//!   エスケープ（REQ-1）を必ず経由する。`raw_html()` は使用しない。
//! - 呼び出し側 `class` は `drop_class_attr` で除去してから recipe が
//!   生成したクラスへ完全に置き換える（生文字列をクラス名合成へ混入させない）。
//! - CSS 宣言はすべてコンパイル時静的リテラルであり、[`crate::css::decl`] の
//!   `is_valid_value` 検証を通過する値のみを使う。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::css::Declaration;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, ContainerBreakpoint, FocusRingColor,
    FocusRingOffset, SlotRecipe, StateCondition, VariantValue,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, Node};

// headless `field` の型のうち、見た目を重ねる必要がなくそのまま透過できる
// もの（`FieldIds`/`FieldProps`）と、コード実体を持たないため styled 側の
// 再定義が不要なパーツ（`label`/`helper_text`/`error_text`/
// `required_indicator`）を選択的に再エクスポートする（規約 A、
// `crate::lib` 「headless 再エクスポートの形式規約（イシュー #1062）」節）。
// `root` は本モジュールが variant クラスを重ねるため同名再定義し、
// `input`/`textarea`/`select` は `crate::input`/`crate::textarea`/
// `crate::native_select` が担当するためここでは再エクスポートしない
// （呼び出し側はそれぞれのモジュールから `input`/`textarea`/`native_select`
// を使う）。
pub use fandhe_frontend_headless_ui::field::{
    content, error_text, group, helper_text, label, required_indicator, separator, title, FieldIds,
    FieldProps,
};

/// slot 一覧（headless [`fandhe_frontend_headless_ui::field`] の anatomy の
/// うち、本モジュールが CSS を持つ 11 パーツ）。末尾 6 件はイシュー #2185
/// で純追加した拡張パーツ（既存 5 slot の宣言順・出力は不変）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "helper-text",
    "error-text",
    "required-indicator",
    "group",
    "content",
    "title",
    "separator",
    "separator-line",
    "separator-content",
];

/// `root` の配置軸（chakra-ui v3 `Field.Root` の `orientation` prop 相当）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FieldOrientation {
    /// ラベル→コントロールを縦積みする配置（既定）。
    #[default]
    Vertical,
    /// ラベルとコントロールを横並びにする配置。
    Horizontal,
    /// `group`（祖先の [`group`] slot）の inline サイズが 448px 以上のとき
    /// のみ `Horizontal` と同じ配置へ切り替わる配置（イシュー #2199、
    /// モジュール doc「`orientation="responsive"`」節参照）。`group` の外に
    /// 置いた場合は常に縦積みのまま。
    Responsive,
}

impl VariantValue for FieldOrientation {
    fn axis(self) -> &'static str {
        "orientation"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
            Self::Responsive => "responsive",
        }
    }
}

/// `Horizontal`/`Responsive` の双方が共有する `root` 宣言（イシュー #2199）。
///
/// `Horizontal` は無条件で、`Responsive` は `group` container の
/// [`ContainerBreakpoint::Md`] 以上でのみ、この同一の宣言列を `root` へ
/// 適用する。両呼び出し元で手書きを重複させるとドリフトし得るため、この
/// 関数を唯一の定義元とする（`recipe()` 内の 2 箇所が呼ぶ）。
fn horizontal_root_declarations() -> Vec<Declaration> {
    vec![
        decl("flex-direction", "row"),
        decl("align-items", "center"),
        decl("justify-content", "space-between"),
        decl("gap", "var(--fandhe-space-2)"),
    ]
}

/// [`root`] の見た目設定。
#[derive(Debug, Clone, Copy, Default)]
pub struct FieldRootProps {
    /// 配置軸（既定 `Vertical`）。
    pub orientation: FieldOrientation,
}

/// `root` のラベル配置 variant（イシュー #3134、モジュール doc「ラベル配置」
/// 節参照）。`orientation` とは独立な軸であり、[`root_with_label_placement`]
/// でのみ指定できる（[`FieldRootProps`] へフィールドを増やすと、呼び出し側
/// の構造体リテラル多数がコンパイルエラーになるため新関数として追加した）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FieldLabelPlacement {
    /// 従来どおりラベルを `root` の外形に影響させない配置（既定）。
    /// クラスを出力せず既定の HTML 出力をバイト単位で不変に保つ。
    #[default]
    Outside,
    /// ラベルを枠の内側・上部に置く（Tailwind UI "Inset label" 相当）。
    Inset,
    /// ラベルを `root` の枠線の上へ重ねる（Tailwind UI "Overlapping label"
    /// 相当）。
    Overlap,
}

impl VariantValue for FieldLabelPlacement {
    fn axis(self) -> &'static str {
        "label-placement"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Outside => "outside",
            Self::Inset => "inset",
            Self::Overlap => "overlap",
        }
    }
}

/// この styled Field の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("field", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                // codex-review #1791（`breadcrumb.rs`）と同じ理由:
                // `Theme::empty()` 系カスタムテーマでは `--fandhe-space-1-5`
                // が定義されない可能性があるため、フォールバックを明示する。
                decl("gap", "var(--fandhe-space-1-5, 0.375rem)"),
                decl("width", "100%"),
                decl("position", "relative"),
                decl("box-sizing", "border-box"),
            ],
        )
        .base(
            "label",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("user-select", "none"),
            ],
        )
        .base(
            "helper-text",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "error-text",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("color", "var(--fandhe-color-danger)"),
            ],
        )
        .base(
            "required-indicator",
            vec![
                decl("color", "var(--fandhe-color-danger)"),
                decl("line-height", "var(--fandhe-font-line-height-tight)"),
            ],
        )
        // 以下 6 slot はイシュー #2185 の純追加（shadcn/ui FieldGroup/
        // FieldContent/FieldTitle/テキスト付き FieldSeparator 相当、モジュール
        // doc「採用したもの（イシュー #2185）」節の判断根拠を参照）。
        .base(
            "group",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-6)"),
                decl("width", "100%"),
            ],
        )
        .base(
            "content",
            vec![
                decl("display", "flex"),
                decl("flex", "1 1 0%"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1-5, 0.375rem)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
            ],
        )
        .base(
            "title",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("width", "fit-content"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("user-select", "none"),
            ],
        )
        .base(
            "separator",
            vec![
                decl("position", "relative"),
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("height", "var(--fandhe-space-5)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
            ],
        )
        .base(
            "separator-line",
            vec![
                decl("position", "absolute"),
                decl("inset-inline", "0"),
                decl("top", "50%"),
                decl("margin", "0"),
                decl("border-width", "0"),
                decl("border-top-width", "var(--fandhe-separator-thickness, 1px)"),
                decl("border-top-style", "solid"),
                decl("border-top-color", "var(--fandhe-color-border)"),
            ],
        )
        .base(
            "separator-content",
            vec![
                decl("position", "relative"),
                decl("padding-inline", "var(--fandhe-space-2)"),
                decl("background-color", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("white-space", "nowrap"),
            ],
        )
        .variant(
            FieldOrientation::Horizontal,
            "root",
            horizontal_root_declarations(),
        )
        .default_variant(FieldOrientation::Vertical)
        // container query（イシュー #2199、モジュール doc
        // 「`orientation="responsive"`」節参照）: `group` を container・
        // `root` を container_variant の対象 slot として、`Horizontal` と
        // 同一の宣言を 448px 以上でのみ適用する。
        .container_slot("group")
        .container_variant(
            FieldOrientation::Responsive,
            "root",
            ContainerBreakpoint::Md,
            horizontal_root_declarations(),
        )
        // headless `error_text`/`required_indicator` は非該当状態で
        // `hidden` 存在属性を出す fail-closed 描画（`field.rs` rustdoc
        // 参照）。base の `display: inline-flex` が UA の
        // `[hidden] { display: none; }` を上書きしてしまわないよう、
        // 明示的に `[hidden] { display: none; }` を登録する（先例:
        // `dialog.rs`/`drawer.rs`/`action_bar.rs`/`editable.rs`）。
        .state(
            "error-text",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "required-indicator",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "label",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "helper-text",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // shadcn/ui 突合（イシュー #2014、field.rs モジュール doc「shadcn/ui
        // 突合」節参照）: `Field` の `data-[invalid=true]:text-destructive`
        // 相当。既存の `error-text` 表示切替と矛盾せずエラー視認性を追加的に
        // 高める。
        .state(
            "label",
            StateCondition::Attr("data-invalid"),
            vec![decl("color", "var(--fandhe-color-danger)")],
        )
        // `title` は `label` と同じ型階層（モジュール doc「採用したもの
        // （イシュー #2185）」節参照）のため、disabled 減光・invalid 配色も
        // `label` と同じ規則に揃える。`content` には付与しない（子の
        // `label`/`title`/`helper-text` が既に減光するため二重減光を避ける、
        // モジュール doc 参照）。
        .state(
            "title",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "title",
            StateCondition::Attr("data-invalid"),
            vec![decl("color", "var(--fandhe-color-danger)")],
        )
}

/// この styled Field が生成する静的 CSS 全量を返す（決定的。
/// [`crate::input::css`] と同じ契約）。
///
/// [`SlotRecipe`] の `state()`/`container_variant()` は同一 slot 内条件
/// しか表現できず、`helper-text`（子孫）や `error-text` の子 `<ul>` への
/// ネストセレクタは表現できないため、`list.rs` と同型の直接追記（recipe
/// 出力の末尾に静的 CSS 文字列を連結）で表す静的追記が 2 種ある
/// （shadcn/ui 突合、イシュー #2014/#2160。モジュール doc「shadcn/ui
/// 突合」節参照）: (1) `error-text > ul` のリスト整形、(2) `helper-text`
/// の horizontal/responsive 時 `text-wrap: balance`。
#[must_use]
pub fn css() -> String {
    let mut out = recipe().css();
    out.push('\n');
    out.push_str(
        // `display: flex` は flex item となる `<li>` から UA 既定の
        // `display: list-item` を奪い `::marker` を生成させない（Chromium/
        // WebKit で disc マーカーが表示されない、PR #2147 Cursor Bugbot
        // 指摘の是正）。`list-style: disc` を実際に効かせるため `<ul>` は
        // block（UA 既定）のまま、outside マーカー用の box を `padding: 0`
        // で潰さないよう字下げは `padding-left`（UA 既定の慣習）で表現し
        // `margin` は 0 とする（同 Bugbot 指摘）。`<li>` の縦間隔は隣接
        // 兄弟セレクタの `margin-top` で表現する。
        "[data-scope=\"field\"][data-part=\"error-text\"] > ul {\n  \
         margin: 0;\n  padding: 0 0 0 var(--fandhe-space-4);\n  \
         list-style: disc;\n}\n\
         [data-scope=\"field\"][data-part=\"error-text\"] > ul > li + li {\n  \
         margin-top: var(--fandhe-space-1);\n}\n",
    );
    out.push_str(
        // shadcn `FieldDescription` の `group-has-[[data-orientation=horizontal]]/field`
        // 相当（イシュー #2160）。`data-orientation` は本モジュールの語彙に
        // 存在しないためクラス条件（`fd-field--orientation-horizontal`/
        // `-responsive`）へ読み替える。`helper-text` は `content` の内側
        // （イシュー #2185）に置かれ得るため子結合子ではなく子孫結合子を
        // 使う。未対応ブラウザでは `text-wrap: balance` が無効値として
        // 破棄されるだけで無害（Baseline 2024 相当）。`responsive` も
        // 対象に含める（モジュール doc「`helper-text` の `text-wrap:
        // balance`」節参照）。
        "[data-scope=\"field\"][data-part=\"root\"].fd-field--orientation-horizontal \
         [data-scope=\"field\"][data-part=\"helper-text\"] {\n  \
         text-wrap: balance;\n}\n\
         @container fd-field-group (min-width: 448px) {\n  \
         [data-scope=\"field\"][data-part=\"root\"].fd-field--orientation-responsive \
         [data-scope=\"field\"][data-part=\"helper-text\"] {\n    \
         text-wrap: balance;\n  }\n}\n",
    );
    // ラベル配置（イシュー #3134、モジュール doc「ラベル配置」節参照）。
    // `SlotRecipe` は同一 slot 内条件しか表現できず、ここで必要なのは
    // 「root の variant を条件とした子（label/input）への宣言」と「root の
    // :focus-within」のため、上記 2 ブロックと同型の直接追記で表す。
    // セレクタの class 部分は `recipe().variant_class(...)` から作るため、
    // HTML 側のクラス名と CSS 側のセレクタが構造的にずれない。
    //
    // 枠線を `root` が描く点は Inset/Overlap で共通のため、状態表示
    // （invalid/disabled/focus）は `boxed_root_state_css` に一本化し、両
    // variant が同じ規則集合を持つ（モジュール doc「状態表示の対応表」節）。
    let recipe = recipe();
    let inset_class = recipe.variant_class(FieldLabelPlacement::Inset);
    let overlap_class = recipe.variant_class(FieldLabelPlacement::Overlap);
    let root_inset = format!("[data-scope=\"field\"][data-part=\"root\"].{inset_class}");
    let root_overlap = format!("[data-scope=\"field\"][data-part=\"root\"].{overlap_class}");
    out.push_str(&format!(
        "{root_inset} {{\n  \
         gap: 0;\n  \
         padding: var(--fandhe-space-2-5, 0.625rem) var(--fandhe-space-3) var(--fandhe-space-1-5, 0.375rem);\n  \
         border: 1px solid var(--fandhe-color-border);\n  \
         border-radius: var(--fandhe-radius-md);\n  \
         background: var(--fandhe-color-bg);\n}}\n",
    ));
    out.push_str(&format!(
        "{root_inset} > [data-scope=\"field\"][data-part=\"label\"] {{\n  \
         font-size: var(--fandhe-font-font-size-xs);\n}}\n",
    ));
    out.push_str(&format!(
        "{root_inset} > [data-scope=\"field\"][data-part=\"input\"] {{\n  \
         height: auto;\n  padding: 0;\n  border: 0;\n  border-radius: 0;\n  \
         background: transparent;\n}}\n",
    ));
    out.push_str(&boxed_root_state_css(&root_inset));
    // inset の縦連結は `inset_stack` の直下に限る（モジュール doc「ラベル
    // 配置」節）。隣接兄弟セレクタ `+` は親の `gap` を区別できず、`group`
    // 内の離れた root にも誤って発動する（codex-review 指摘、PR #3570）ため、
    // `data-part="inset-stack"` の子結合子を全規則の前置条件にする。先行・
    // 後続の双方が `border: 1px solid` を持つため負マージンで重ねると共有
    // する辺に 2 本の border が同時に描画される（同指摘）ので、後続要素の
    // `border-top` を消し、共有する辺の描画を先行要素の `border-bottom` の
    // みへ一本化する。
    let stack = format!("{INSET_STACK_SELECTOR} > ");
    out.push_str(&format!(
        "{INSET_STACK_SELECTOR} {{\n  display: flex;\n  flex-direction: column;\n}}\n",
    ));
    out.push_str(&format!(
        "{stack}{root_inset} + {root_inset} {{\n  \
         border-top: 0;\n  border-start-start-radius: 0;\n  border-start-end-radius: 0;\n}}\n",
    ));
    out.push_str(&format!(
        "{stack}{root_inset}:has(+ {root_inset}) {{\n  \
         border-end-start-radius: 0;\n  border-end-end-radius: 0;\n}}\n",
    ));
    // 共有する辺は先行要素の `border-bottom` なので、後続要素だけが
    // `data-invalid` のときはそのままでは通常色のまま残る（codex-review
    // 指摘、PR #3570）。CSS は前方参照できないため `:has(+ ...)` で後続の
    // 状態を先行側へ写し、エラー枠の上辺を成立させる（`:has()` は上の
    // 角丸規則で既に前提にしている）。`:has()` 引数側の属性セレクタ 1 件分
    // だけ `{root_inset}[data-invalid]` より詳細度が高く、先行要素自身が
    // invalid でも同じエラー色なので宣言順に依存しない。focus/disabled は
    // 枠線色を変えない（focus はリング、disabled は opacity）ため、
    // 共有する辺について写すべき状態は invalid のみ。
    out.push_str(&format!(
        "{stack}{root_inset}:has(+ {root_inset}[data-invalid]) {{\n  \
         border-bottom-color: var(--fandhe-color-danger);\n}}\n",
    ));
    // overlap: root の base は position: relative を既に持つ（本モジュール
    // `recipe()` の `root` base 宣言参照）が、モジュール doc が謳う「root の
    // 枠線の上へ重ねる」を成立させるには root 自身が枠線を持つ必要がある
    // （codex-review 指摘、PR #3570）。input 自身も既定（Outline variant）で
    // `border: 1px solid` を持つ（`input.rs` 参照）ため、root 側を単一の
    // 枠として統一し input 側の枠線は消す（codex-review/Bugbot 指摘）。
    out.push_str(&format!(
        "{root_overlap} {{\n  \
         border: 1px solid var(--fandhe-color-border);\n  \
         border-radius: var(--fandhe-radius-md);\n}}\n",
    ));
    out.push_str(&format!(
        "{root_overlap} > [data-scope=\"field\"][data-part=\"input\"] {{\n  \
         border: 0;\n}}\n",
    ));
    out.push_str(&boxed_root_state_css(&root_overlap));
    out.push_str(&format!(
        "{root_overlap} > [data-scope=\"field\"][data-part=\"label\"] {{\n  \
         position: absolute;\n  top: 0;\n  inset-inline-start: var(--fandhe-space-2);\n  \
         z-index: 1;\n  transform: translateY(-50%);\n  padding-inline: var(--fandhe-space-1);\n  \
         font-size: var(--fandhe-font-font-size-xs);\n  \
         background-color: var(--fandhe-field-label-bg, var(--fandhe-color-bg));\n}}\n",
    ));
    out
}

/// [`inset_stack`] wrapper の CSS セレクタ（イシュー #3134）。`Inset` の
/// 枠線連結規則はすべてこのセレクタの子結合子を前置条件に持つ。
const INSET_STACK_SELECTOR: &str = "[data-scope=\"field\"][data-part=\"inset-stack\"]";

/// `Inset` ラベル配置の `root` を縦に連結する wrapper（`div`、イシュー
/// #3134、モジュール doc「ラベル配置」節参照）。直下に並べた
/// [`FieldLabelPlacement::Inset`] の `root` 同士だけが枠線を共有して
/// 連結し、この wrapper の外（`group` 等）では連結規則は発動しない。
/// headless 側に対応パーツはなく（レイアウト専用で意味論を持たない）、
/// `data-scope="field"`/`data-part="inset-stack"` を本モジュールが直接
/// 出力する。`attrs` は後置で合成し、`class` 属性も呼び出し側の指定を
/// そのまま通す（本 wrapper は recipe のクラスを持たないため）。
#[must_use]
pub fn inset_stack<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("data-scope", "field"), ("data-part", "inset-stack")];
    merged.extend(attrs);
    el("div", merged, children)
}

/// `root` 自身が枠線を描く配置（`Inset`/`Overlap`）に共通する状態表示の
/// 規則を組み立てる（内部ヘルパ、[`css`] のみが呼ぶ。イシュー #3134、
/// モジュール doc「状態表示の対応表」節参照）。`root_selector` は
/// `[data-scope="field"][data-part="root"].<variant class>` 形式の完全な
/// セレクタで、戻り値はそのセレクタを前置した規則群。
///
/// `crate::input::css` が `input` 自身へ与える invalid（`border-color`）・
/// focus（[`focus_ring_declarations`]）の 2 状態を、枠線を描く側である
/// `root` へそのまま写す。`input` 側の focus outline は枠線がない状態では
/// 枠より内側のリングとして二重表示になるため、子孫セレクタで打ち消す。
/// disabled は各子パーツの既存 `[data-disabled]` 減衰に委ね root へは置かず、
/// `data-readonly` は `input.rs` と同じく視覚宣言を持たない（意図的非採用、
/// `crate::input` モジュール doc 参照）。
fn boxed_root_state_css(root_selector: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{root_selector}[data-invalid] {{\n  border-color: var(--fandhe-color-danger);\n}}\n",
    ));
    // disabled: root 側には宣言を置かない。`label`/`helper-text`/`title`
    // （本モジュール `recipe()`）と `input`（`crate::input`）が各自
    // `[data-disabled]` で 0.5 へ減衰済みのため、root にも opacity を当てると
    // 0.25 まで二重に薄くなる（Bugbot 指摘、PR #3570）。減衰は子側の 1 回のみ。
    // リングは枠線の内側に描く（`FocusRingOffset::Inset`）。`Inset` 連結時の
    // 後続要素は `border-top: 0` で先行要素と重ならないため、重ね順の調整
    // （z-index）は不要。
    if let Some(block) = crate::css::serialize_rule(
        &format!("{root_selector}:focus-within"),
        &focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Inset),
    ) {
        out.push_str(&block);
    }
    out.push_str(&format!(
        "{root_selector} > [data-scope=\"field\"][data-part=\"input\"]:focus-visible {{\n  \
         outline: none;\n}}\n",
    ));
    out
}

/// styled `root` パーツを組み立てる。`orientation` に応じたクラスを付与し
/// （`drop_class_attr` により呼び出し側の `class` は除去してから合成する）、
/// `disabled`/`invalid`/`required`/`readonly` の data-* フラグ・
/// アクセシビリティ配線は [`fandhe_frontend_headless_ui::field::root`] へ
/// そのまま委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps, FieldRootProps};
///
/// let f = FieldProps {
///     id: "email",
///     ids: FieldIds::default(),
///     disabled: false,
///     invalid: false,
///     required: false,
///     readonly: false,
///     has_helper_text: false,
/// };
/// let node = field::root(&FieldRootProps::default(), &f, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="field" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    props: &FieldRootProps,
    field: &FieldProps<'_>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    root_with_label_placement(props, FieldLabelPlacement::Outside, field, attrs, children)
}

/// [`root`] にラベル配置 variant（イシュー #3134、モジュール doc「ラベル
/// 配置」節参照）を重ねて組み立てる。`placement` が [`FieldLabelPlacement::
/// Outside`]（既定）のときは [`root`] と完全に同じ出力になる（`orientation`
/// クラスのみ、`label-placement` クラスは付与しない）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::field::{
///     self, FieldIds, FieldLabelPlacement, FieldProps, FieldRootProps,
/// };
///
/// let f = FieldProps {
///     id: "email",
///     ids: FieldIds::default(),
///     disabled: false,
///     invalid: false,
///     required: false,
///     readonly: false,
///     has_helper_text: false,
/// };
/// let node = field::root_with_label_placement(
///     &FieldRootProps::default(),
///     FieldLabelPlacement::Inset,
///     &f,
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains("fd-field--label-placement-inset"));
/// ```
#[must_use]
pub fn root_with_label_placement<'a>(
    props: &FieldRootProps,
    placement: FieldLabelPlacement,
    field: &FieldProps<'_>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let mut class = recipe.variant_classes(&[("orientation", props.orientation.value())]);
    if placement != FieldLabelPlacement::Outside {
        let placement_class = recipe.variant_class(placement);
        if !placement_class.is_empty() {
            class.push(' ');
            class.push_str(&placement_class);
        }
    }
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::field::root(field, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    fn default_field(id: &str) -> FieldProps<'_> {
        FieldProps {
            id,
            ids: FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        }
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="field"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = css();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn default_class_is_orientation_vertical() {
        let f = default_field("f");
        let node = root(&FieldRootProps::default(), &f, vec![], vec![]);
        let html = render(&node);
        assert!(html.contains("fd-field--orientation-vertical"));
        assert!(!html.contains("fd-field--orientation-horizontal"));
    }

    #[test]
    fn horizontal_orientation_switches_class() {
        let f = default_field("f");
        let props = FieldRootProps {
            orientation: FieldOrientation::Horizontal,
        };
        let node = root(&props, &f, vec![], vec![]);
        let html = render(&node);
        assert!(html.contains("fd-field--orientation-horizontal"));
        assert!(!html.contains("fd-field--orientation-vertical"));
    }

    #[test]
    fn responsive_orientation_switches_class() {
        let f = default_field("f");
        let props = FieldRootProps {
            orientation: FieldOrientation::Responsive,
        };
        let node = root(&props, &f, vec![], vec![]);
        let html = render(&node);
        assert!(html.contains("fd-field--orientation-responsive"));
        assert!(!html.contains("fd-field--orientation-vertical"));
        assert!(!html.contains("fd-field--orientation-horizontal"));
    }

    #[test]
    fn css_contains_container_block() {
        let out = css();
        assert!(out.contains(r#"[data-scope="field"][data-part="group"] {"#));
        assert!(out.contains("container-type: inline-size;"));
        assert!(out.contains("container-name: fd-field-group;"));
        assert!(out.contains("@container fd-field-group (min-width: 448px) {"));
        assert!(out.contains(
            r#"[data-scope="field"][data-part="root"].fd-field--orientation-responsive {"#
        ));
    }

    #[test]
    fn caller_class_is_dropped_and_replaced_by_recipe_class() {
        let f = default_field("f");
        let node = root(
            &FieldRootProps::default(),
            &f,
            vec![("class", "evil")],
            vec![],
        );
        let html = render(&node);
        assert!(!html.contains("evil"));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-field--orientation-vertical"));
    }

    #[test]
    fn root_propagates_field_state_flags() {
        let f = FieldProps {
            id: "f",
            ids: FieldIds::default(),
            disabled: true,
            invalid: true,
            required: true,
            readonly: true,
            has_helper_text: false,
        };
        let html = render(&root(&FieldRootProps::default(), &f, vec![], vec![]));
        assert!(html.contains("data-disabled"));
        assert!(html.contains("data-invalid"));
        assert!(html.contains("data-required"));
        assert!(html.contains("data-readonly"));
    }

    #[test]
    fn css_contains_hidden_and_disabled_state_rules() {
        let out = css();
        assert!(out.contains(r#"[data-scope="field"][data-part="error-text"][hidden]"#));
        assert!(out.contains(r#"[data-scope="field"][data-part="required-indicator"][hidden]"#));
        assert!(out.contains(r#"[data-scope="field"][data-part="label"][data-disabled]"#));
        assert!(out.contains(r#"[data-scope="field"][data-part="helper-text"][data-disabled]"#));
    }

    #[test]
    fn css_does_not_declare_control_slots() {
        // イシュー #3134: `Inset` ラベル配置が `.fd-field--label-placement-
        // inset > [data-part="input"]` の子孫セレクタ上書きを 1 件だけ持つ
        // （module doc「スコープ」節の例外参照）。これは `input`/`textarea`/
        // `select` の **base 宣言の再登録ではない**ことを、素の
        // `[data-scope="field"][data-part="input"] {`（base セレクタそのもの）
        // が現れないことで固定する。`textarea`/`select` は例外を持たない
        // ため従来どおり完全不在を固定する。行頭（セレクタの開始位置）に
        // 素の base セレクタが来る行が無いことを確認する（`>` などの結合子
        // を前置した子孫セレクタは行頭に一致しないため判別できる）。
        let out = css();
        assert!(!out
            .lines()
            .any(|line| line.starts_with(r#"[data-scope="field"][data-part="input"] {"#)));
        assert!(!out.contains(r#"[data-part="textarea"]"#));
        assert!(!out.contains(r#"[data-part="select"]"#));
    }

    #[test]
    fn reexported_parts_smoke_render_without_panicking() {
        let f = default_field("f");
        let _ = render(&label(&f, vec![], vec![text("Email")]));
        let _ = render(&helper_text(&f, vec![], vec![text("hint")]));
        let _ = render(&error_text(&f, vec![], vec![text("error")]));
        let _ = render(&required_indicator(&f, vec![], vec![text("*")]));
    }

    // イシュー #3134: ラベル配置 variant（inset/overlap）のユニットテスト。

    #[test]
    fn root_and_root_with_label_placement_outside_are_byte_identical() {
        let f = default_field("f");
        let a = render(&root(&FieldRootProps::default(), &f, vec![], vec![]));
        let b = render(&root_with_label_placement(
            &FieldRootProps::default(),
            FieldLabelPlacement::Outside,
            &f,
            vec![],
            vec![],
        ));
        assert_eq!(a, b);
        assert!(!a.contains("label-placement"));
    }

    #[test]
    fn inset_placement_adds_single_class_and_keeps_orientation_class() {
        let f = default_field("f");
        let html = render(&root_with_label_placement(
            &FieldRootProps::default(),
            FieldLabelPlacement::Inset,
            &f,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-field--label-placement-inset"));
        assert!(!html.contains("fd-field--label-placement-overlap"));
        assert!(html.contains("fd-field--orientation-vertical"));
        assert_eq!(html.matches("class=\"").count(), 1);
    }

    #[test]
    fn overlap_placement_adds_single_class() {
        let f = default_field("f");
        let html = render(&root_with_label_placement(
            &FieldRootProps::default(),
            FieldLabelPlacement::Overlap,
            &f,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-field--label-placement-overlap"));
        assert!(!html.contains("fd-field--label-placement-inset"));
    }

    #[test]
    fn label_placement_caller_class_is_dropped() {
        let f = default_field("f");
        let html = render(&root_with_label_placement(
            &FieldRootProps::default(),
            FieldLabelPlacement::Inset,
            &f,
            vec![("class", "evil")],
            vec![],
        ));
        assert!(!html.contains("evil"));
    }

    #[test]
    fn css_contains_inset_rules() {
        let out = css();
        assert!(out.contains("fd-field--label-placement-inset"));
        assert!(out.contains(
            "[data-scope=\"field\"][data-part=\"root\"].fd-field--label-placement-inset:focus-within"
        ));
        assert!(out.contains(
            ".fd-field--label-placement-inset > [data-scope=\"field\"][data-part=\"input\"]"
        ));
        assert!(out.contains(":focus-visible"));
    }

    #[test]
    fn inset_stack_renders_scope_part_and_passes_attrs() {
        let html = render(&inset_stack(vec![("id", "s")], vec![text("x")]));
        assert_eq!(
            html,
            r#"<div data-scope="field" data-part="inset-stack" id="s">x</div>"#
        );
    }

    #[test]
    fn css_inset_connection_rules_are_scoped_to_inset_stack() {
        // codex-review 指摘（PR #3570）: `+` は親の gap を区別できないため、
        // 連結規則はすべて `inset-stack` の子結合子を前置条件に持ち、
        // wrapper なしの隣接だけで発動する規則を 1 件も持たない。
        let out = css();
        let root = "[data-scope=\"field\"][data-part=\"root\"].fd-field--label-placement-inset";
        for line in out
            .lines()
            .filter(|l| l.contains(&format!("{root} + ")) || l.contains(&format!("{root}:has(")))
        {
            assert!(
                line.starts_with(&format!("{INSET_STACK_SELECTOR} > ")),
                "unscoped connection rule: {line}"
            );
        }
        assert!(out.contains(&format!(
            "{INSET_STACK_SELECTOR} {{\n  display: flex;\n  flex-direction: column;\n}}\n"
        )));
    }

    #[test]
    fn css_contains_inset_sibling_connection_rules() {
        let out = css();
        assert!(out.contains(
            "fd-field--label-placement-inset + [data-scope=\"field\"][data-part=\"root\"].fd-field--label-placement-inset"
        ));
        assert!(out.contains(
            ":has(+ [data-scope=\"field\"][data-part=\"root\"].fd-field--label-placement-inset)"
        ));
        // イシュー #3134 codex-review 指摘（PR #3570）是正: 共有する辺の
        // border 描画を先行要素側へ一本化するため、後続要素は
        // `border-top: 0` を持ち二重線にならないことを固定する。
        assert!(out.contains("border-top: 0;"));
    }

    #[test]
    fn css_contains_inset_invalid_sibling_shared_border_rule() {
        // PR #3570 レビュー指摘是正: 後続フィールドのみ `data-invalid` の
        // ときも、共有する辺（先行要素の `border-bottom`）がエラー色へ
        // 切り替わることを固定する。
        let out = css();
        assert!(out.contains(
            "fd-field--label-placement-inset:has(+ [data-scope=\"field\"][data-part=\"root\"].fd-field--label-placement-inset[data-invalid]) {\n  \
             border-bottom-color: var(--fandhe-color-danger);\n}\n"
        ));
    }

    #[test]
    fn css_contains_overlap_rules_and_label_bg_var() {
        let out = css();
        assert!(out.contains("fd-field--label-placement-overlap"));
        assert!(out.contains("--fandhe-field-label-bg"));
        // イシュー #3134 codex-review 指摘（PR #3570）是正: ラベルが
        // 重なる対象の枠線を root 自身に持たせる。
        assert!(out.contains(
            "[data-scope=\"field\"][data-part=\"root\"].fd-field--label-placement-overlap {\n  \
             border: 1px solid var(--fandhe-color-border);\n"
        ));
    }

    #[test]
    fn css_inset_and_overlap_share_boxed_root_state_rules() {
        // モジュール doc「状態表示の対応表」節: 両 variant の状態規則は
        // `boxed_root_state_css` 由来で同一であることを固定する。
        let out = css();
        let inset = "[data-scope=\"field\"][data-part=\"root\"].fd-field--label-placement-inset";
        let overlap =
            "[data-scope=\"field\"][data-part=\"root\"].fd-field--label-placement-overlap";
        let normalize = |s: &str| s.replace(inset, "ROOT").replace(overlap, "ROOT");
        assert_eq!(
            normalize(&boxed_root_state_css(inset)),
            normalize(&boxed_root_state_css(overlap))
        );
        for root in [inset, overlap] {
            assert!(out.contains(&boxed_root_state_css(root)));
            assert!(!out.contains(&format!("{root}[data-readonly]")));
            // disabled は子パーツの既存減衰のみ（root 側に opacity を重ねない）。
            assert!(!out.contains(&format!("{root}[data-disabled]")));
        }
        // 連結時に重なりがないため z-index は不要（dead 宣言を持たない）。
        assert!(!out.contains(":focus-within {\n  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));\n  z-index"));
    }

    #[test]
    fn css_contains_overlap_focus_within_ring_and_input_outline_reset() {
        // PR #3570 レビュー指摘是正: input 自身の `:focus-visible` outline を
        // 消し、root 側の `:focus-within` が枠線全体を縁取るリングへ一体化
        // することを固定する（Inset と同型）。
        let out = css();
        assert!(out.contains(
            "[data-scope=\"field\"][data-part=\"root\"].fd-field--label-placement-overlap:focus-within"
        ));
        assert!(out.contains(
            ".fd-field--label-placement-overlap > [data-scope=\"field\"][data-part=\"input\"]:focus-visible {\n  \
             outline: none;\n}\n"
        ));
    }

    #[test]
    fn css_label_placement_block_comes_after_text_wrap_balance_block() {
        let out = css();
        let balance_pos = out
            .find("text-wrap: balance")
            .expect("helper-text の text-wrap: balance 規則が存在すること");
        let inset_pos = out
            .find("fd-field--label-placement-inset")
            .expect("inset 規則が存在すること");
        assert!(
            inset_pos > balance_pos,
            "label-placement の追記は css() 末尾への純追加であること"
        );
    }

    #[test]
    fn css_base_root_label_input_blocks_are_unchanged() {
        // 既存 base ブロック（root/label）が label-placement 追加によって
        // 変化していないことを固定する（受け入れ条件 3: 既定出力は不変）。
        let out = css();
        assert!(out.contains(
            "[data-scope=\"field\"][data-part=\"root\"] {\n  display: flex;\n  flex-direction: column;\n"
        ));
        assert!(out.contains(
            "[data-scope=\"field\"][data-part=\"label\"] {\n  display: flex;\n  align-items: center;\n"
        ));
    }

    #[test]
    fn default_root_html_does_not_contain_label_placement_class() {
        let f = default_field("f");
        let html = render(&root(&FieldRootProps::default(), &f, vec![], vec![]));
        assert!(!html.contains("label-placement"));
    }
}

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
//! - **`helper-text` の `text-wrap: balance`（horizontal 時のみ）**: shadcn
//!   は祖先の `orientation` を Tailwind `group-has` で参照するが、
//!   [`SlotRecipe`] は同一 slot 内条件のみ表現可能なため、実現には
//!   `list.rs` 型の手書き descendant セレクタ追記が必要。効果が視覚微調整
//!   に留まる割に新規複雑化を伴うため不採用。
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
    disabled_declarations, ContainerBreakpoint, SlotRecipe, StateCondition, VariantValue,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

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
/// [`SlotRecipe`] の `state()` は同一 slot 内の属性条件のみを表現でき、
/// `error-text` の子 `<ul>` へのネストセレクタは表現できないため、
/// `list.rs` と同型の直接追記（recipe 出力の末尾に静的 CSS 文字列を
/// 連結）で表す（shadcn/ui 突合、イシュー #2014。モジュール doc
/// 「shadcn/ui 突合」節参照）。
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
    let recipe = recipe();
    let class = recipe.variant_classes(&[("orientation", props.orientation.value())]);
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
        let out = css();
        assert!(!out.contains(r#"[data-part="input"]"#));
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
}

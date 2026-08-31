//! styled CheckboxGroup（headless ラッパー、イシュー #997、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::checkbox_group`（イシュー #997）の
//! Label / Item / ItemControl / ItemIndicator / ItemText 5 anatomy パーツと
//! [`fandhe_frontend_headless_ui::checkbox_group::CheckboxGroup`] 状態機械を
//! そのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::radio_group`] の rustdoc と同じ
//! 方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::radio_group::root`] と同型）を本モジュールで再定義する。
//! headless 自由関数 `root` と名前衝突するため、`pub use ...::*` ではなく
//! 必要な識別子（[`label`]/[`item`]/[`item_control`]/[`item_indicator`]/
//! [`item_text`]/[`CheckboxGroup`]/[`DATA_STATE_CHECKED`]/
//! [`DATA_STATE_UNCHECKED`]）のみを選択的に再エクスポートする。
//!
//! [`CheckboxGroup`] 状態機械は inherent `root()` を持たない（item 系
//! メソッドのみ、`crates/headless-ui/src/checkbox_group.rs` 参照）ため、
//! そのまま再エクスポートしても未スタイル `root` の静かな適用漏れは
//! 発生しない（[`crate::radio_group`] の `RadioGroup` 非対称処理と同じ判断）。
//!
//! # `item-hidden-input` を本モジュールが持たない理由（`checkbox::stylesheet()` 併用が必須）
//!
//! headless 層（`crates/headless-ui/src/checkbox_group.rs`）は
//! `item-hidden-input` パーツを新設せず、ネイティブ `<input type="checkbox">`
//! を [`fandhe_frontend_headless_ui::checkbox::hidden_input`] の入れ子
//! 再利用で賄う（headless 側モジュール doc「anatomy」節参照）。この設計を
//! 継承し、**本モジュールの [`recipe`] は `hidden-input` slot の
//! visually-hidden 規則を一切再宣言しない**
//! （`[data-scope="checkbox"][data-part="hidden-input"]` として
//! `crate::checkbox` の recipe に既存であり、本モジュールで重複実装すると
//! `checkbox` recipe とのドリフト・二重管理を招く）。styled CheckboxGroup を
//! 使う呼び出し側は、本モジュールの [`stylesheet`] に加えて
//! `fandhe_frontend_pre_styled_ui::checkbox::stylesheet()`
//! も併せて読み込む必要がある（`crates/docs-site/src/showcase.rs` が
//! 両方を `push_css` する実例を参照）。
//!
//! # data-state とスタイルの連動
//!
//! `item`/`item-control`/`item-indicator`（選択状態、
//! `data-state="checked"`/`"unchecked"`）の見た目の切り替えを [`recipe`] へ
//! 登録する（[`crate::recipe::SlotRecipe::state`]）。`root` の
//! `data-orientation="horizontal"` でも配置切り替えを行う（[`crate::radio_group`]
//! と同型）。
//!
//! # `:focus-within` によるフォーカスリング
//!
//! 実フォーカスは（呼び出し側が入れ子にする）
//! `fandhe_frontend_pre_styled_ui::checkbox::hidden_input` が受ける。
//! [`crate::radio_group`] と同じ理由（`item`〔`<label>`〕が hidden input の
//! 祖先であること）により、`item` へ `:focus-within` のフォーカスリングを
//! 登録する。
//!
//! # `size`/`palette` variant
//!
//! [`crate::radio_group`] rustdoc「`size`/`palette` variant」節と同じ設計
//! （`root` スコープの custom property 経由で `item-control`/`item-text` の
//! 寸法・見た目を切り替える）に従う。`size` は
//! `--fandhe-checkbox-group-control-size`/`-font-size`/`-item-gap`/`-gap`
//! （イシュー #1461 で `-item-gap`/`-gap` を新設）を、`palette` は
//! [`crate::recipe::palette_scale_declarations`] を登録する。`var()` にはいずれも
//! Md サイズ・Accent パレット相当のフォールバック値を書き、styled `root` を
//! 経由しない headless 直接利用マークアップでも現行外観を維持する
//! （fail-safe）。size variant の一括登録は [`crate::recipe::SlotRecipe::size_variants`]
//! （イシュー #1424 の共通生成手段）を使い、既定 `md` の設定漏れを構造的に
//! 防ぐ。
//!
//! # スタイル調整（イシュー #1461、size バリアント・ラベル/item-text の型階層）
//!
//! 親 #1459（checkbox-group を chakra-ui / Radix Themes 基準へ調整）の
//! 分割 2/2。1/2（イシュー #1460）が root の `gap` custom property 化
//! （受け口）・`orientation` 折り返し・`data-invalid`/`data-disabled`
//! 伝播・`item` の `width`/opacity/cursor・`item-control` の
//! hover/transition/focus ring canonical 化を担当するのに対し、本イシューは
//! **size バリアントの寸法段階設計とラベル・item-text の型階層**を担当する
//! （担当領域を分けているため互いの変更範囲には触れない）。
//!
//! - **size variant の一括登録**: 5 段の `.variant(Size::*, "root", ...)` を
//!   個別に手書きする代わりに [`crate::recipe::SlotRecipe::size_variants`]
//!   を使う（`crate::checkbox`/`crate::checkbox_card` と同型）。
//! - **control 寸法を 4px 格子へ**: `xs`/`sm` のみ `0.75rem`（12px）/
//!   `0.875rem`（14px）へ変更する（`crate::checkbox` #1735 と同値、Radix
//!   Themes `size1`/`size2` 相当）。`md`/`lg`/`xl` は既存の外観を変えない。
//!   チェックマーク寸法（`check-width`/`check-height`）は control に対する
//!   光学的な比率値であり、[`item_indicator`] の `margin-bottom: 0.1rem` と
//!   同じ「spacing スケール外の意図的な例外」として現状値のまま維持する。
//! - **`item` の `gap` を size 連動に**: `--fandhe-checkbox-group-item-gap`
//!   （control ↔ text 余白）を新設し、`item` base の `gap` 宣言を
//!   `var(--fandhe-checkbox-group-item-gap, var(--fandhe-space-2))` へ変更
//!   する（フォールバックは既存の Md 相当値）。xs〜xl で `--fandhe-space-1`/
//!   `-1-5`/`-2`/`-2-5`/`-3` の単調増加（`crate::checkbox` の
//!   `--fandhe-checkbox-gap` と同じ段階）。
//! - **`root` の `gap` を size 連動に**: `--fandhe-checkbox-group-gap`
//!   （項目間余白、1/2 が用意した受け口）を xs〜xl で `--fandhe-space-0-5`/
//!   `-1`/`-1`/`-1-5`/`-2` の**非減少**（sm と md が同値）で定義する。厳密
//!   単調にすると md の既定外観が変わるため、現行外観維持を優先し非減少に
//!   留める。
//! - **`label`（グループ見出し）の型階層**: `font-size` を固定 `sm` から
//!   size 連動 custom property へ変更し、`font-weight: medium` と
//!   `line-height: normal` を追加する（chakra `Fieldset.Legend` の
//!   `textStyle: sm` + medium と同型、`crate::checkbox`/`crate::checkbox_card`
//!   の `label` 語彙に揃える）。
//! - **`item-text`（項目テキスト）の型階層**: `line-height: normal`
//!   （複数行項目の行送り）と `user-select: none`（クリックでトグルする
//!   `item`〔`<label>`〕内テキストの誤選択防止、chakra label と同じ）を
//!   追加する。`font-weight` は宣言せず通常ウェイトを継承する（Radix
//!   Themes 参照スクリーンショットの項目テキストが通常ウェイトであること、
//!   `label` の medium との 2 段階を作ることが根拠）。
//!
//! ## 意図的に合わせない点
//!
//! - **ヘルパーテキスト（description）パートは追加しない**: headless
//!   anatomy（`crates/headless-ui/src/checkbox_group.rs`）に存在せず、
//!   anatomy 構造は headless 層の責務（`crate::checkbox` #1455 と同じ判断）。
//!   `data-part="description"` を pre-styled-ui 側だけで新設すると
//!   Primitives/Themes 間の anatomy ドリフト検知
//!   （`crates/docs-site/tests/wrap_state.rs` 等）と公開 API 追加を伴うため
//!   見送る。説明文が必要な呼び出し側は `--fandhe-color-fg-muted` + 1 段
//!   小さい font-size を自前合成する。headless 側の description パート
//!   検討はイシュー #1602 の射程とする。
//! - **`label` の `margin-bottom` は size 連動させない**: `root` の `gap`
//!   と加算されるため、現行外観維持のため据え置く。
//! - **`label`/`item-text` へ hover/transition/`data-*` は追加しない**:
//!   非インタラクティブなテキストで、disabled の見た目は 1/2 の `item`
//!   opacity 伝播が波及済みで足りる。
//! - **チェックマーク線幅（2px 固定）は size 連動させない**: `crate::checkbox`
//!   と同じ判断。
//! - **variant 軸（solid/subtle/outline 等）は追加しない**: 1/2・
//!   `crate::checkbox`/`crate::checkbox_card` と同じ判断（`root()` シグネ
//!   チャ変更は破壊的、Forms 家族横断の判断が必要）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層の再エクスポートと静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（`value`/属性/children）へ CSS 値として流し込む経路を持たない
//! （動的値は headless 層経由で `fandhe_frontend_core::render` の既定
//! エスケープを必ず通る、REQ-1）。styled `root` は [`drop_class_attr`] に
//! より呼び出し側の `class` を除去してから合成するため、`class` 属性は常に
//! 単一（[`crate::radio_group::root`] と同型）。
//!
//! # 本イシューのスコープ外
//!
//! headless 層モジュール doc「out-of-scope」節（キーボードナビゲーション・
//! 実 DOM 配線・全選択/一部選択集約・Field 連携・`checkbox_card` を item
//! として使う構成）をそのまま継承する。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_scale_declarations, ColorPalette, Size, SlotRecipe, StateCondition, VariantValue,
};

// headless 自由関数 `root` はあえて再エクスポートしない（本モジュール冒頭
// の rustdoc「選択的 re-export」節参照、`root` は本モジュールで styled 版
// として再定義する）。
pub use fandhe_frontend_headless_ui::checkbox_group::{
    item, item_control, item_indicator, item_text, label, CheckboxGroup, DATA_STATE_CHECKED,
    DATA_STATE_UNCHECKED,
};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// headless `checkbox_group` anatomy の `data-part` 一覧（`crates/headless-ui/src/checkbox_group.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。`item-hidden-input` を
/// 含まない理由はモジュール doc「`item-hidden-input` を本モジュールが
/// 持たない理由」節参照）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "item",
    "item-control",
    "item-indicator",
    "item-text",
];

/// この styled CheckboxGroup の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("checkbox-group", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl(
                    "gap",
                    "var(--fandhe-checkbox-group-gap, var(--fandhe-space-1))",
                ),
            ],
        )
        .base(
            "label",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "font-size",
                    "var(--fandhe-checkbox-group-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl(
                    "gap",
                    "var(--fandhe-checkbox-group-item-gap, var(--fandhe-space-2))",
                ),
                decl("cursor", "pointer"),
            ],
        )
        .base(
            "item-control",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("width", "var(--fandhe-checkbox-group-control-size, 1rem)"),
                decl("height", "var(--fandhe-checkbox-group-control-size, 1rem)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("flex-shrink", "0"),
                decl("transition", "background 0.15s, border-color 0.15s"),
            ],
        )
        .base(
            "item-text",
            vec![
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "font-size",
                    "var(--fandhe-checkbox-group-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("user-select", "none"),
            ],
        )
        // イシュー #997: indicator の base に `display` 宣言を置かない
        // （headless 層が `data-state="unchecked"` 時に `hidden` 存在属性を
        // 付与する規約と衝突させないため。`crates/pre-styled-ui/src/checkbox.rs`
        // 「`indicator` の `hidden` 属性意味論を CSS が壊さない設計」節と
        // 同型の判断）。
        .base(
            "item-indicator",
            vec![
                // イシュー #997 Bugbot 指摘（Medium）回帰固定: 固定寸法ではなく
                // `--fandhe-checkbox-group-check-width`/`-check-height`
                // custom property（`root` の size variant が切り替える）を
                // 参照する。`crates/pre-styled-ui/src/checkbox.rs` の
                // `indicator`（`--fandhe-checkbox-check-width`/`-check-height`）
                // と同型。
                decl("width", "var(--fandhe-checkbox-group-check-width, 0.25rem)"),
                decl(
                    "height",
                    "var(--fandhe-checkbox-group-check-height, 0.5rem)",
                ),
                decl(
                    "border-right",
                    "2px solid var(--fandhe-palette-fg, var(--fandhe-color-accent-fg))",
                ),
                decl(
                    "border-bottom",
                    "2px solid var(--fandhe-palette-fg, var(--fandhe-color-accent-fg))",
                ),
                decl("transform", "rotate(45deg)"),
                decl("margin-bottom", "0.1rem"),
            ],
        )
        // `root` の `data-orientation="horizontal"`（headless 層が
        // `data_orientation` 経由で出力）では縦積みではなく横並びへ切り替える。
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "horizontal"),
            vec![decl("flex-direction", "row")],
        )
        // 選択済み item-control の見た目（角丸の四角、palette 色の塗り。
        // ラジオの円形〔`border-radius: 50%`〕ではない）。
        .state(
            "item-control",
            StateCondition::AttrEq("data-state", "checked"),
            vec![
                decl(
                    "border-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
            ],
        )
        // `data-disabled`（headless 層が `data_disabled` 経由で `item`/
        // `item-control`/`item-indicator`/`item-text` へ出力）時の操作不能な
        // 見た目。
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        // 実フォーカスは呼び出し側が入れ子にする checkbox::hidden_input が
        // 受ける（visually-hidden 化されている）ため、祖先 `item`
        // （`<label>`）へ `:focus-within` で反映する（[`crate::radio_group`]
        // と同型のフォールバック、モジュール doc 参照）。
        .state(
            "item",
            StateCondition::FocusWithin,
            vec![
                decl(
                    "outline",
                    "2px solid var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("outline-offset", "2px"),
            ],
        )
        // イシュー #1461: 5 段の `.variant(Size::*, "root", ...)` を個別に
        // 手書きする代わりに `size_variants`（イシュー #1424 の共通生成
        // 手段、`checkbox.rs`/`checkbox_card.rs` と同型）を使い、既定 `md`
        // の設定漏れを構造的に防ぐ（規約は
        // `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
        // §4）。control 寸法は xs/sm のみ 4px 格子（12px/14px）へ是正し、
        // md/lg/xl は既存外観を維持する（`checkbox.rs` #1735 と同値）。
        // チェックマーク寸法（比率値）は現状維持。`--fandhe-checkbox-group-
        // item-gap`（item 内の control ↔ text 余白）と
        // `--fandhe-checkbox-group-gap`（root の項目間余白）は本イシューで
        // 新設した size 連動 custom property。item-gap は xs〜xl まで
        // spacing トークン経由で単調増加させる。root の gap は現行外観
        // （md）を変えないため非減少（sm と md が同値）に留める。
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![
                        decl("--fandhe-checkbox-group-control-size", "0.75rem"),
                        decl("--fandhe-checkbox-group-check-width", "0.15rem"),
                        decl("--fandhe-checkbox-group-check-height", "0.3rem"),
                        decl(
                            "--fandhe-checkbox-group-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                        decl("--fandhe-checkbox-group-item-gap", "var(--fandhe-space-1)"),
                        decl("--fandhe-checkbox-group-gap", "var(--fandhe-space-0-5)"),
                    ],
                ),
                (
                    Size::Sm,
                    vec![
                        decl("--fandhe-checkbox-group-control-size", "0.875rem"),
                        decl("--fandhe-checkbox-group-check-width", "0.2rem"),
                        decl("--fandhe-checkbox-group-check-height", "0.4rem"),
                        decl(
                            "--fandhe-checkbox-group-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl(
                            "--fandhe-checkbox-group-item-gap",
                            "var(--fandhe-space-1-5)",
                        ),
                        decl("--fandhe-checkbox-group-gap", "var(--fandhe-space-1)"),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl("--fandhe-checkbox-group-control-size", "1rem"),
                        decl("--fandhe-checkbox-group-check-width", "0.25rem"),
                        decl("--fandhe-checkbox-group-check-height", "0.5rem"),
                        decl(
                            "--fandhe-checkbox-group-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl("--fandhe-checkbox-group-item-gap", "var(--fandhe-space-2)"),
                        decl("--fandhe-checkbox-group-gap", "var(--fandhe-space-1)"),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        decl("--fandhe-checkbox-group-control-size", "1.25rem"),
                        decl("--fandhe-checkbox-group-check-width", "0.3rem"),
                        decl("--fandhe-checkbox-group-check-height", "0.6rem"),
                        decl(
                            "--fandhe-checkbox-group-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                        decl(
                            "--fandhe-checkbox-group-item-gap",
                            "var(--fandhe-space-2-5)",
                        ),
                        decl("--fandhe-checkbox-group-gap", "var(--fandhe-space-1-5)"),
                    ],
                ),
                (
                    Size::Xl,
                    vec![
                        decl("--fandhe-checkbox-group-control-size", "1.5rem"),
                        decl("--fandhe-checkbox-group-check-width", "0.35rem"),
                        decl("--fandhe-checkbox-group-check-height", "0.7rem"),
                        decl(
                            "--fandhe-checkbox-group-font-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                        decl("--fandhe-checkbox-group-item-gap", "var(--fandhe-space-3)"),
                        decl("--fandhe-checkbox-group-gap", "var(--fandhe-space-2)"),
                    ],
                ),
            ],
        )
        .default_variant(ColorPalette::Accent);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        recipe = recipe.variant(palette, "root", palette_scale_declarations(palette));
    }
    recipe
}

/// この styled CheckboxGroup が生成する静的 CSS 全量を返す（決定的。
/// [`crate::radio_group::stylesheet`] と同じ契約）。CheckboxGroup を実際に
/// 利用する際は `crate::checkbox::stylesheet()` も併せて読み込む必要がある
/// （モジュール doc「`item-hidden-input` を本モジュールが持たない理由」
/// 節参照）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去
/// してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::checkbox_group::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::checkbox_group;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = checkbox_group::root(
///     Size::Md,
///     ColorPalette::Accent,
///     false,
///     None,
///     None,
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"data-scope="checkbox-group" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    disabled: bool,
    orientation: Option<Orientation>,
    labelled_by: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::checkbox_group::root(
        disabled,
        orientation,
        labelled_by,
        merged,
        children,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="checkbox-group"][data-part="item-control"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_never_duplicates_checkbox_hidden_input_rules() {
        // §設計判断（モジュール doc 参照）: visually-hidden の 9 宣言は
        // `crate::checkbox` recipe の `hidden-input` slot にのみ存在し、
        // 本 stylesheet では再宣言しない（重複実装の回帰固定）。
        let css = stylesheet();
        assert!(!css.contains("hidden-input"));
        assert!(!css.contains("clip: rect(0, 0, 0, 0);"));
    }

    #[test]
    fn stylesheet_links_data_state_checked_to_item_control_style() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="checkbox-group"][data-part="item-control"][data-state="checked"]"#
        ));
        assert!(css.contains("border-color: var(--fandhe-palette, var(--fandhe-color-accent));"));
        // ラジオの円形ではなく角丸の四角（border-radius: 50% を含まない）。
        assert!(!css.contains("border-radius: 50%;"));
    }

    #[test]
    fn indicator_base_has_no_display_declaration() {
        // headless 層の `hidden` 存在属性の意味論を壊さないため
        // （`crate::checkbox` の同名テストと同型の回帰固定）。
        let css = stylesheet();
        let scope = r#"[data-scope="checkbox-group"][data-part="item-indicator"] {"#;
        let start = css.find(scope).expect("item-indicator base block missing");
        let end = css[start..]
            .find('}')
            .map(|i| start + i)
            .expect("closing brace missing");
        let block = &css[start..end];
        assert!(!block.contains("display:"));
    }

    #[test]
    fn root_switches_to_row_layout_on_horizontal_orientation() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="checkbox-group"][data-part="root"][data-orientation="horizontal"]"#
        ));
        assert!(css.contains("flex-direction: row;"));
    }

    #[test]
    fn disabled_item_gets_not_allowed_cursor() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="checkbox-group"][data-part="item"][data-disabled]"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn item_focus_within_gets_accent_outline_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="checkbox-group"][data-part="item"]:focus-within {"#));
        assert!(
            css.contains("outline: 2px solid var(--fandhe-palette, var(--fandhe-color-accent));")
        );
    }

    /// イシュー #1461: control 寸法（`--fandhe-checkbox-group-control-size`）
    /// が xs〜xl で単調増加することを rem 値の parse で固定する
    /// （`crate::checkbox` の同名テストと同型）。
    #[test]
    fn size_variants_control_size_is_monotonic() {
        let css = stylesheet();
        let mut sizes_rem = Vec::new();
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl] {
            let selector = format!(
                r#"[data-scope="checkbox-group"][data-part="root"].fd-checkbox-group--size-{}"#,
                size.value()
            );
            let start = css
                .find(&selector)
                .unwrap_or_else(|| panic!("size variant selector not found: {selector} in {css}"));
            let block_end = css[start..]
                .find('}')
                .map(|i| start + i)
                .unwrap_or(css.len());
            let block = &css[start..block_end];
            let decl_start = block
                .find("--fandhe-checkbox-group-control-size: ")
                .unwrap_or_else(|| panic!("control-size declaration not found in {block}"));
            let after = &block[decl_start + "--fandhe-checkbox-group-control-size: ".len()..];
            let value_end = after
                .find(';')
                .unwrap_or_else(|| panic!("control-size declaration not terminated in {block}"));
            let raw = &after[..value_end];
            let rem = raw
                .strip_suffix("rem")
                .unwrap_or_else(|| panic!("control-size value not in rem: {raw}"))
                .parse::<f64>()
                .unwrap_or_else(|_| panic!("control-size value not numeric: {raw}"));
            sizes_rem.push((size, rem));
        }
        for pair in sizes_rem.windows(2) {
            let (prev_size, prev) = pair[0];
            let (next_size, next) = pair[1];
            assert!(
                prev < next,
                "control-size not monotonic: {prev_size:?}={prev}rem >= {next_size:?}={next}rem"
            );
        }
    }

    /// イシュー #1461: `--fandhe-checkbox-group-item-gap`（item 内の control
    /// ↔ text 余白）が xs〜xl で spacing トークン経由の単調増加になることを
    /// 固定する。
    #[test]
    fn size_variants_set_item_gap_custom_property_monotonically() {
        let css = stylesheet();
        let expected = [
            (Size::Xs, "var(--fandhe-space-1)"),
            (Size::Sm, "var(--fandhe-space-1-5)"),
            (Size::Md, "var(--fandhe-space-2)"),
            (Size::Lg, "var(--fandhe-space-2-5)"),
            (Size::Xl, "var(--fandhe-space-3)"),
        ];
        for (size, gap) in expected {
            let selector = format!(
                r#"[data-scope="checkbox-group"][data-part="root"].fd-checkbox-group--size-{}"#,
                size.value()
            );
            let start = css
                .find(&selector)
                .unwrap_or_else(|| panic!("size variant selector not found: {selector} in {css}"));
            let block_end = css[start..]
                .find('}')
                .map(|i| start + i)
                .unwrap_or(css.len());
            let block = &css[start..block_end];
            let expected_decl = format!("--fandhe-checkbox-group-item-gap: {gap};");
            assert!(
                block.contains(&expected_decl),
                "size={size:?} variant block missing {expected_decl}: {block}"
            );
        }
    }

    /// イシュー #1461: `--fandhe-checkbox-group-gap`（root の項目間余白）が
    /// xs〜xl で非減少（sm と md が同値、md の現行外観維持のため厳密単調
    /// にしない旨のコメントをモジュール doc に残す）になることを固定する。
    #[test]
    fn size_variants_set_root_gap_custom_property_non_decreasing() {
        let css = stylesheet();
        let expected = [
            (Size::Xs, "var(--fandhe-space-0-5)"),
            (Size::Sm, "var(--fandhe-space-1)"),
            (Size::Md, "var(--fandhe-space-1)"),
            (Size::Lg, "var(--fandhe-space-1-5)"),
            (Size::Xl, "var(--fandhe-space-2)"),
        ];
        for (size, gap) in expected {
            let selector = format!(
                r#"[data-scope="checkbox-group"][data-part="root"].fd-checkbox-group--size-{}"#,
                size.value()
            );
            let start = css
                .find(&selector)
                .unwrap_or_else(|| panic!("size variant selector not found: {selector} in {css}"));
            let block_end = css[start..]
                .find('}')
                .map(|i| start + i)
                .unwrap_or(css.len());
            let block = &css[start..block_end];
            let expected_decl = format!("--fandhe-checkbox-group-gap: {gap};");
            assert!(
                block.contains(&expected_decl),
                "size={size:?} variant block missing {expected_decl}: {block}"
            );
        }
    }

    /// イシュー #1461: 5 段すべての size variant ブロックに
    /// `--fandhe-checkbox-group-font-size` が登録されていることを固定する。
    #[test]
    fn size_variants_set_font_size_custom_property() {
        let css = stylesheet();
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl] {
            let selector = format!(
                r#"[data-scope="checkbox-group"][data-part="root"].fd-checkbox-group--size-{}"#,
                size.value()
            );
            let start = css
                .find(&selector)
                .unwrap_or_else(|| panic!("size variant selector not found: {selector} in {css}"));
            let block_end = css[start..]
                .find('}')
                .map(|i| start + i)
                .unwrap_or(css.len());
            assert!(
                css[start..block_end].contains("--fandhe-checkbox-group-font-size"),
                "size={size:?} variant block missing --fandhe-checkbox-group-font-size: {}",
                &css[start..block_end]
            );
        }
    }

    /// イシュー #1461: `label`（グループ見出し）が `item-text`（項目テキスト）
    /// より 1 段強い型階層（medium ウェイト・行送り・size 連動 font-size）を
    /// 持つことを固定する。
    #[test]
    fn label_has_typography_hierarchy_declarations() {
        let css = stylesheet();
        let selector = r#"[data-scope="checkbox-group"][data-part="label"] {"#;
        let start = css
            .find(selector)
            .unwrap_or_else(|| panic!("label base selector not found in {css}"));
        let block_end = css[start..]
            .find('}')
            .map(|i| start + i)
            .unwrap_or(css.len());
        let block = &css[start..block_end];
        assert!(
            block.contains(
                "font-size: var(--fandhe-checkbox-group-font-size, var(--fandhe-font-font-size-sm));"
            ),
            "label block missing size-linked font-size: {block}"
        );
        assert!(
            block.contains("font-weight: var(--fandhe-font-font-weight-medium);"),
            "label block missing font-weight: {block}"
        );
        assert!(
            block.contains("line-height: var(--fandhe-font-line-height-normal);"),
            "label block missing line-height: {block}"
        );
        assert!(
            block.contains("color: var(--fandhe-color-fg);"),
            "label block missing color: {block}"
        );
    }

    /// イシュー #1461: `item-text`（項目テキスト）が `line-height`/
    /// `user-select` を持つ一方、`font-weight` は持たない（label との
    /// 2 段階の型階層を作る）ことを固定する。
    #[test]
    fn item_text_has_line_height_and_user_select_without_font_weight() {
        let css = stylesheet();
        let selector = r#"[data-scope="checkbox-group"][data-part="item-text"] {"#;
        let start = css
            .find(selector)
            .unwrap_or_else(|| panic!("item-text base selector not found in {css}"));
        let block_end = css[start..]
            .find('}')
            .map(|i| start + i)
            .unwrap_or(css.len());
        let block = &css[start..block_end];
        assert!(
            block.contains("line-height: var(--fandhe-font-line-height-normal);"),
            "item-text block missing line-height: {block}"
        );
        assert!(
            block.contains("user-select: none;"),
            "item-text block missing user-select: {block}"
        );
        assert!(
            !block.contains("font-weight:"),
            "item-text block should not declare font-weight: {block}"
        );
    }

    /// イシュー #1461: `item` base の `gap` が `--fandhe-checkbox-group-
    /// item-gap` custom property をフォールバック `--fandhe-space-2`（既存
    /// Md 相当値）付きで参照することを固定する。
    #[test]
    fn item_base_gap_uses_item_gap_custom_property_with_space_2_fallback() {
        let css = stylesheet();
        let selector = r#"[data-scope="checkbox-group"][data-part="item"] {"#;
        let start = css
            .find(selector)
            .unwrap_or_else(|| panic!("item base selector not found in {css}"));
        let block_end = css[start..]
            .find('}')
            .map(|i| start + i)
            .unwrap_or(css.len());
        let block = &css[start..block_end];
        assert!(
            block.contains("gap: var(--fandhe-checkbox-group-item-gap, var(--fandhe-space-2));"),
            "item block missing item-gap linked gap: {block}"
        );
    }

    // --- variant クラス ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="checkbox-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="group""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-checkbox-group--size-md"));
        assert!(html.contains("fd-checkbox-group--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-checkbox-group--size-xs"),
            (Size::Sm, "fd-checkbox-group--size-sm"),
            (Size::Md, "fd-checkbox-group--size-md"),
            (Size::Lg, "fd-checkbox-group--size-lg"),
            (Size::Xl, "fd-checkbox-group--size-xl"),
        ] {
            let html = render(&root(
                size,
                ColorPalette::Accent,
                false,
                None,
                None,
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (
                ColorPalette::Accent,
                "fd-checkbox-group--color-palette-accent",
            ),
            (ColorPalette::Info, "fd-checkbox-group--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-checkbox-group--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-checkbox-group--color-palette-warning",
            ),
            (
                ColorPalette::Danger,
                "fd-checkbox-group--color-palette-danger",
            ),
            (
                ColorPalette::Neutral,
                "fd-checkbox-group--color-palette-neutral",
            ),
        ] {
            let html = render(&root(Size::Md, palette, false, None, None, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn reexported_root_with_horizontal_orientation_emits_data_orientation() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            Some(Orientation::Horizontal),
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-orientation="horizontal""#));
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_contains_size_and_palette_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--color-palette-"));
        assert!(css.contains("--fandhe-checkbox-group-control-size"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="checkbox-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn xss_payload_in_item_value_is_escaped_by_render() {
        let payload = "\"><script>alert(1)</script>";
        let html = render(&item(false, false, payload, vec![], vec![text(payload)]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn xss_payload_in_item_text_children_is_escaped_by_render() {
        let payload = "\"><img src=x onerror=alert(1)>";
        let html = render(&item_text(false, false, vec![], vec![text(payload)]));
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_checkbox_group_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut g = CheckboxGroup::default();
        assert_eq!(g.selected(), &[] as &[String]);

        assert!(dispatch(&mut g, "select", "red"));
        assert!(g.is_checked("red"));

        let ssr_html = render(&g.item_control("red", false, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="checked""#));

        let hydrate_html = render(&render_for_hydration(&g));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = CheckboxGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert!(restored.is_checked("red"));
    }

    #[test]
    fn item_indicator_hidden_state_semantics_preserved_through_reexport() {
        let unchecked = render(&item_indicator(false, false, vec![], vec![]));
        assert!(unchecked.contains(r#"hidden="""#));

        let checked = render(&item_indicator(true, false, vec![], vec![]));
        assert!(!checked.contains(r#"hidden="""#));
    }

    #[test]
    fn data_state_constants_reexported() {
        assert_eq!(DATA_STATE_CHECKED, "checked");
        assert_eq!(DATA_STATE_UNCHECKED, "unchecked");
    }
}

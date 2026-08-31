//! styled CheckboxCard（イシュー #747、親 #520/#545、Phase 3 親 #736）。
//!
//! chakra-ui `forms/checkbox-card` 相当のカード型選択 UI。ark-ui には
//! checkbox-card 相当の anatomy は存在せず、chakra-ui が headless の
//! Checkbox 状態機械の上に独自 slot recipe として実装している構図を、本
//! クレートでもそのまま踏襲する。**headless-ui には手を入れない**（受け入れ
//! 条件）。
//!
//! # anatomy は pre-styled 層で新規定義する（[`crate::card`] 先例準拠）
//!
//! `fandhe_frontend_headless_ui::checkbox`（イシュー #535/#595）の 5 anatomy
//! パーツ（`root`/`control`/`indicator`/`label`/`hidden-input`）は
//! `data-scope="checkbox"` に固定されており、カード型の 9 パーツ構成へ
//! そのまま拡張できない。[`crate::card`] が pre-styled 層で独自 anatomy
//! （`data-scope="card"`）を持つ先例（同モジュール rustdoc 参照）に倣い、
//! 本モジュールは新規 anatomy `data-scope="checkbox-card"` を
//! [`fandhe_frontend_headless_ui::anatomy`] で定義する。既存 `checkbox` scope
//! とは完全に独立するため、[`crate::checkbox`] の CSS/属性契約と衝突しない。
//!
//! # 状態機械の再利用（受け入れ条件 1: 新規状態機械を作らない）
//!
//! SSR 静的 props は
//! [`fandhe_frontend_headless_ui::checkbox::CheckboxProps`]/[`CheckedState`]
//! をそのまま再利用する。動的状態遷移（dispatch/hydration）は
//! [`fandhe_frontend_headless_ui::checkbox::Checkbox`] をそのまま利用し、本
//! モジュールから再エクスポートしない（[`crate::checkbox`] の「`Checkbox`
//! 型を再エクスポートしない理由」節と同じ判断: `Checkbox` の inherent
//! `.root(...)` は `size`/`palette` クラスを付与しない未スタイル実体であり、
//! 誤って呼ぶと見た目が静かに崩れるため）。呼び出し側は
//! `fandhe_frontend_headless_ui::checkbox::Checkbox` を直接 import し、描画は
//! 本モジュールのパーツ関数で組み立てる。
//!
//! # anatomy パーツ構成（chakra-ui slot 準拠、9 パーツ）
//!
//! - [`root`][]: `<label>`。カード全体の起点。`size`/`palette` クラスを付与する
//!   唯一のパーツ。
//! - [`control`][]: `<div>`。indicator と content を横に並べる領域。
//! - [`content`][]: `<div>`。label/description/addon を縦に積むコンテナ。
//! - [`label`][]: `<div>`。見出しテキスト。
//! - [`description`][]: `<div>`。補足テキスト。
//! - [`addon`][]: `<div>`。任意の付加コンテンツ（アイコン等）。
//! - [`indicator`][]: `<div>`。チェックボックス外枠（[`crate::checkbox::control`]
//!   と同型の border/background 描画）。
//! - [`indicator_check`][]: `<div>`。チェックマーク本体（[`crate::checkbox`]
//!   headless 側 `indicator` 相当。[`CheckedState::Unchecked`] のとき
//!   `hidden`）。chakra-ui の単一 Indicator を 2 要素（外枠 + マーク）へ
//!   分けるのは、[`crate::recipe::SlotRecipe`] が疑似要素を持たず、既存
//!   checkbox の実証済み border/transform 描画をそのまま再利用するため。
//! - [`hidden_input`][]: `<input type="checkbox">`。フォーム送信の実体。
//!   `type`/`name`/`value`/`checked`/`aria-checked="mixed"`/`aria-invalid`/
//!   `disabled`/`required` の属性契約は
//!   `crates/headless-ui/src/checkbox.rs` の `hidden_input` と同一ロジックで
//!   出力する（両ファイルを合わせて確認する契約。ずれると headless 版と
//!   挙動が乖離する）。
//!
//! # `indicator_check` の `hidden` 属性意味論（[`crate::checkbox`] と同じ設計）
//!
//! [`indicator_check`] の `base` に `display` 宣言を置かない（unchecked 時に
//! ブラウザ UA stylesheet の `[hidden] { display: none }` を上書きしてしまう
//! 回帰を防ぐ。`indicator_check_base_has_no_display_declaration` テストで
//! 固定。詳細な根拠は [`crate::checkbox`] rustdoc 参照）。
//!
//! # フォーカスリング（本イシューのスコープ、§ out-of-scope 参照）
//!
//! 実フォーカスは [`hidden_input`] が受けるため、[`crate::radio_group`] の
//! `item` と同型の [`StateCondition::FocusWithin`]（wasm なしでも成立する
//! no-JS フォールバック）のみを [`root`] へ登録する。`data-focus-visible`
//! （wasm 配線によるキーボード操作専用リング）は
//! `crates/wasm-full/src/focus_visible.rs` の `(scope, part)` マッピングに
//! `"checkbox-card"` が未登録のため本イシューでは実装しない（フォローアップ、
//! PR 本文参照）。
//!
//! # `size`/`palette` variant
//!
//! [`crate::checkbox`] rustdoc「複合部品の variant 統一方針」節（#708）と
//! 同型。`size`（[`Size`]）は [`root`] へのみクラスを付与し、[`recipe`] が
//! 登録する `--fandhe-checkbox-card-*` の root スコープ custom property 経由で
//! `control`/`indicator`/`indicator-check`/`label` の寸法を切り替える。
//! `palette`（[`ColorPalette`]）は [`crate::recipe::palette_scale_declarations`] を
//! `root` へ登録し、checked/indeterminate 時の枠線・背景色を
//! `var(--fandhe-palette, ...)` 経由で切り替える。
//!
//! # スタイル調整（イシュー #1458、内部レイアウト slot と size 軸）
//!
//! 親 #1456（chakra-ui `checkbox-card` / Radix Themes `checkbox-cards` 基準
//! への視覚調整）の 2/2 分割。1/2（イシュー #1457・PR #1736）が `root` slot の
//! base/state（hover / disabled / transition / focus-ring / `data-invalid`）を
//! 担当するのに対し、本イシューは内部レイアウト slot（`control`/`content`/
//! `label`/`description`/`addon`/`indicator`/`indicator-check`）と size
//! バリアント（`root` へ登録する `--fandhe-checkbox-card-*` custom property
//! 群）を担当する。**`root` の base/state 宣言は一切変更しない**（slot 境界の
//! 競合を避けるため）。
//!
//! - **size 軸を `size_variants` へ移行**: 5 段の `.variant(Size::*, "root",
//!   ...)` を個別に手書きする代わりに [`crate::recipe::SlotRecipe::size_variants`]
//!   （イシュー #1424 の共通生成手段、[`crate::checkbox`] #1455 と同型）を使い、
//!   既定 `md` の設定漏れを構造的に防ぐ。
//! - **padding を spacing トークンへ**: 生 rem リテラルだった `xs`〜`xl` の
//!   `--fandhe-checkbox-card-padding` を chakra md（16px）を基準に 4px 格子・
//!   spacing トークン（`--fandhe-space-2`/`-3`/`-4`/`-5`/`-6`）で単調増加させる。
//! - **control-size を [`crate::checkbox`] と統一**: indicator 寸法（`0.75rem`〜
//!   `1.5rem`）を checkbox 家族で共通化する（chakra のカード indicator が
//!   通常 checkbox より 1 段大きい点は意図的に合わせない）。
//! - **`control` の `gap` を size 連動に**: `--fandhe-checkbox-card-gap`
//!   （新設）を `control` の `gap: var(--fandhe-checkbox-card-gap,
//!   var(--fandhe-space-2))` へ登録し、xs〜xl で spacing トークンの単調増加
//!   （`--fandhe-space-1`/`-1-5`/`-2-5`/`-3`/`-4`）を割り当てる。
//! - **`label` に型階層を追加**: `font-weight: medium`・`color: fg` に加え、
//!   `line-height: normal`（複数行ラベルの行送り）と `user-select: none`
//!   （[`crate::checkbox`] の `label` と同語彙）を追加する。
//! - **`description` を size 連動フォントサイズに**: `--fandhe-checkbox-card-
//!   description-font-size`（新設、label より常に 1 段下）+ `line-height:
//!   normal` を追加する。色は opacity ではなくトークン `fg-muted` を維持する
//!   （ダーク追従のため、chakra の `opacity: 0.64` に対する意図的差分）。
//! - **`addon` を description と同じ型階層に**: `align-items: center` +
//!   `gap` + `font-size`/`color`（description と同じ custom property・
//!   トークン）を追加する。chakra の下部フッター帯（`border-top` 付き）は
//!   本クレートの anatomy（content 内の縦積み子）には転用しない意図的差分
//!   （帯にする場合は独立 slot の新設を要し、本イシューのスコープ外）。
//! - **`indicator` の transition を motion トークンへ**: 直書きの
//!   `transition: background 0.15s, border-color 0.15s` を
//!   [`crate::recipe::transition_declarations`]（[`crate::recipe::MotionDuration::Fast`]）へ
//!   置換する（イシュー #1425 写像表）。
//! - **`indicator` へ invalid 表現を追加**: [`crate::checkbox::control`] と
//!   同型の `[data-invalid] { border-color: danger }` を追加する（`root` 側の
//!   invalid 表現は 1/2 の担当）。
//! - **意図的に合わせなかった点**: (1) indicator の左右位置は chakra/Radix
//!   ともに右端だがローカルは左のまま維持する（CSS `order` は使わず DOM 順で
//!   決める設計方針のため、`showcase` の子順変更のみで視覚確認する）。
//!   (2) disabled 時の二重減光（chakra は label/description/addon にも
//!   `opacity: 0.5`）は追加しない（`root` の `opacity: 0.5` が子孫へ CSS
//!   継承で波及済みのため）。(3) indicator は非インタラクティブ slot のため
//!   hover を追加しない（#1425 判定基準）。
//!
//! # セキュリティ不変条件
//!
//! `raw_html()` は使用しない。CSS 宣言値はすべて静的リテラルで、動的値
//! （`name`/`value`/attrs/children）は
//! [`fandhe_frontend_headless_ui::fandhe_frontend_core::render`] の既定
//! エスケープを必ず経由する（REQ-1）。呼び出し側 `attrs` の `class` は
//! [`drop_class_attr`] で除去してから合成し、`class` 属性は常に単一
//! （[`crate::checkbox::root`] と同型）。`data-state`/`data-disabled`/
//! `data-invalid`/`data-required`/`data-readonly` の状態キーと、
//! `hidden_input` が固定する `type`/`checked`/`aria-checked`/`aria-invalid`/
//! `name`/`value`/`disabled`/`required` は呼び出し側の偽装値を fail-closed で
//! 除去する（`crates/headless-ui/src/checkbox.rs` の `STATE_RESERVED`/
//! `HIDDEN_INPUT_RESERVED`/`drop_reserved` と同型の判断を本モジュールで
//! 独立に実装する — [`fandhe_frontend_headless_ui::anatomy::Anatomy::part`]
//! は `data-scope`/`data-part` のみを守るため、それ以外の予約キー保護は
//! 各 styled 部品自身の責務であることは headless 側モジュール doc の
//! 「セキュリティ不変条件」節と同じ）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `fandhe-frontend-wasm-full` の focus/クリック配線（`(scope, part)` を
//!   `("checkbox-card", "hidden-input") -> "root"` へ写像し
//!   `data-focus-visible` を CSS で伝える対応）。
//! - `examples/headless-pre-styled-ui` への追随（pre-styled-ui 公開後に
//!   別 PR で対応）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_scale_declarations, transition_declarations, ColorPalette, MotionDuration, Size,
    SlotRecipe, StateCondition, VariantValue,
};
use fandhe_frontend_headless_ui::aria::{aria_checked, AriaChecked};
pub use fandhe_frontend_headless_ui::checkbox::{CheckboxProps, CheckedState};
use fandhe_frontend_headless_ui::data_attrs::{
    data_disabled, data_invalid, data_readonly, data_required, data_state,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="checkbox-card"` を固定した本コンポーネントの anatomy
/// （既存 `data-scope="checkbox"` とは独立、モジュール冒頭 rustdoc参照）。
const ANATOMY: Anatomy = anatomy("checkbox-card");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &[
    "root",
    "control",
    "content",
    "label",
    "description",
    "addon",
    "indicator",
    "indicator-check",
    "hidden-input",
];

/// 全パーツ共通の `data-state`/`data-disabled`/`data-invalid`/`data-required`/
/// `data-readonly` 属性列を組み立てる非公開ヘルパ（`crates/headless-ui/src/checkbox.rs`
/// の `state_attrs` と同型）。
fn state_attrs(props: &CheckboxProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs: Vec<(&'static str, &'static str)> =
        vec![data_state(props.checked.as_data_state())];
    attrs.extend(data_disabled(props.disabled));
    attrs.extend(data_invalid(props.invalid));
    attrs.extend(data_required(props.required));
    attrs.extend(data_readonly(props.readonly));
    attrs
}

/// [`state_attrs`] が全パーツへ一律付与する属性キー一覧（呼び出し側 `attrs`
/// からの偽装を fail-closed で除去する対象、`crates/headless-ui/src/checkbox.rs`
/// の `STATE_RESERVED` と同型）。
const STATE_RESERVED: &[&str] = &[
    "data-state",
    "data-disabled",
    "data-invalid",
    "data-required",
    "data-readonly",
];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（`crates/headless-ui/src/checkbox.rs` の `drop_reserved` と同型）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// この styled CheckboxCard の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("checkbox-card", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("align-items", "flex-start"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("cursor", "pointer"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
                decl("padding", "var(--fandhe-checkbox-card-padding, 0.75rem)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("transition", "border-color 0.15s, box-shadow 0.15s"),
            ],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-state", "checked"),
            vec![
                decl(
                    "border-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "box-shadow",
                    "0 0 0 1px var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
            ],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-state", "indeterminate"),
            vec![
                decl(
                    "border-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "box-shadow",
                    "0 0 0 1px var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
            ],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        // イシュー #747: 実フォーカスは hidden-input が受けるため、祖先
        // root（`<label>`）へ `:focus-within` で no-JS フォールバックのリング
        // を反映する（`crate::radio_group` の `item` と同型、モジュール
        // rustdoc 参照）。
        .state(
            "root",
            StateCondition::FocusWithin,
            vec![
                decl(
                    "outline",
                    "2px solid var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("outline-offset", "2px"),
            ],
        )
        .base(
            "control",
            vec![
                decl("display", "flex"),
                decl("align-items", "flex-start"),
                decl("gap", "var(--fandhe-checkbox-card-gap, var(--fandhe-space-2))"),
                decl("flex", "1"),
            ],
        )
        .base(
            "content",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("flex", "1"),
            ],
        )
        .base(
            "label",
            vec![
                decl(
                    "font-size",
                    "var(--fandhe-checkbox-card-label-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("user-select", "none"),
            ],
        )
        .base(
            "description",
            vec![
                decl(
                    "font-size",
                    "var(--fandhe-checkbox-card-description-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "addon",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "font-size",
                    "var(--fandhe-checkbox-card-description-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "indicator",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("width", "var(--fandhe-checkbox-card-control-size, 1rem)"),
                decl("height", "var(--fandhe-checkbox-card-control-size, 1rem)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("flex-shrink", "0"),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結されるため、
        // 上記 base ブロックを書き換えずに純追加する（イシュー #1425 写像表、
        // `crate::checkbox::control` と同型のパターン）。直書きの
        // `transition: background 0.15s, border-color 0.15s` を motion
        // トークン経由（`MotionDuration::Fast`）へ置換する（イシュー #1458）。
        .base(
            "indicator",
            transition_declarations("background, border-color", MotionDuration::Fast),
        )
        .state(
            "indicator",
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
        .state(
            "indicator",
            StateCondition::AttrEq("data-state", "indeterminate"),
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
        // headless 層が invalid な選択肢へ出す `data-invalid` を indicator
        // slot へ反映する（[`crate::checkbox::control`] の `data-invalid`
        // 規則と同型の視覚言語。イシュー #1458。`root` 側の invalid 表現は
        // 1/2（PR #1736）の担当のため本規則は追加しない）。
        .state(
            "indicator",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        // `indicator-check` の base に `display` 宣言を置かない（モジュール
        // rustdoc「`indicator_check` の `hidden` 属性意味論」節参照。
        // `indicator_check_base_has_no_display_declaration` テストで固定）。
        .base(
            "indicator-check",
            vec![
                decl("width", "var(--fandhe-checkbox-card-check-width, 0.25rem)"),
                decl("height", "var(--fandhe-checkbox-card-check-height, 0.5rem)"),
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
        .state(
            "indicator-check",
            StateCondition::AttrEq("data-state", "indeterminate"),
            vec![
                decl("transform", "none"),
                decl("border-right", "0"),
                decl(
                    "border-bottom",
                    "2px solid var(--fandhe-palette-fg, var(--fandhe-color-accent-fg))",
                ),
                decl("width", "var(--fandhe-checkbox-card-dash-width, 0.5rem)"),
                decl("height", "0"),
                decl("margin-bottom", "0"),
            ],
        )
        // hidden-input の視覚的非表示化（[`crate::checkbox`]/[`crate::switch`]
        // と同じ visually-hidden パターン）。
        .base(
            "hidden-input",
            vec![
                decl("position", "absolute"),
                decl("width", "1px"),
                decl("height", "1px"),
                decl("padding", "0"),
                decl("margin", "-1px"),
                decl("overflow", "hidden"),
                decl("clip", "rect(0, 0, 0, 0)"),
                decl("white-space", "nowrap"),
                decl("border", "0"),
            ],
        )
        // イシュー #1458: 5 段の `.variant(Size::*, "root", ...)` を個別に
        // 手書きする代わりに `size_variants`（イシュー #1424 の共通生成
        // 手段、`crate::checkbox` #1455 と同型）を使い、既定 `md` の設定漏れを
        // 構造的に防ぐ。padding は生 rem リテラルではなく spacing トークン
        // （chakra md=16px を基準に 4px 格子で単調増加）へ載せ替え、
        // description の size 連動フォントサイズ（label より 1 段下）と
        // root 全体の gap（`--fandhe-checkbox-card-gap`、[`control`] base が
        // 参照）を新設する。control-size は [`crate::checkbox`] と同一値へ
        // そろえる（fandhe の checkbox 家族で indicator 寸法を統一する意図的
        // 判断。chakra のカード indicator が通常 checkbox より 1 段大きい点は
        // 意図的に合わせない）。チェックマーク寸法（比率値）は現状維持。
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![
                        decl("--fandhe-checkbox-card-padding", "var(--fandhe-space-2)"),
                        decl("--fandhe-checkbox-card-control-size", "0.75rem"),
                        decl("--fandhe-checkbox-card-check-width", "0.15rem"),
                        decl("--fandhe-checkbox-card-check-height", "0.3rem"),
                        decl("--fandhe-checkbox-card-dash-width", "0.3rem"),
                        decl(
                            "--fandhe-checkbox-card-label-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                        decl(
                            "--fandhe-checkbox-card-description-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                        decl("--fandhe-checkbox-card-gap", "var(--fandhe-space-1)"),
                    ],
                ),
                (
                    Size::Sm,
                    vec![
                        decl("--fandhe-checkbox-card-padding", "var(--fandhe-space-3)"),
                        decl("--fandhe-checkbox-card-control-size", "0.875rem"),
                        decl("--fandhe-checkbox-card-check-width", "0.2rem"),
                        decl("--fandhe-checkbox-card-check-height", "0.4rem"),
                        decl("--fandhe-checkbox-card-dash-width", "0.4rem"),
                        decl(
                            "--fandhe-checkbox-card-label-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl(
                            "--fandhe-checkbox-card-description-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                        decl("--fandhe-checkbox-card-gap", "var(--fandhe-space-1-5)"),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl("--fandhe-checkbox-card-padding", "var(--fandhe-space-4)"),
                        decl("--fandhe-checkbox-card-control-size", "1rem"),
                        decl("--fandhe-checkbox-card-check-width", "0.25rem"),
                        decl("--fandhe-checkbox-card-check-height", "0.5rem"),
                        decl("--fandhe-checkbox-card-dash-width", "0.5rem"),
                        decl(
                            "--fandhe-checkbox-card-label-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl(
                            "--fandhe-checkbox-card-description-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl("--fandhe-checkbox-card-gap", "var(--fandhe-space-2-5)"),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        decl("--fandhe-checkbox-card-padding", "var(--fandhe-space-5)"),
                        decl("--fandhe-checkbox-card-control-size", "1.25rem"),
                        decl("--fandhe-checkbox-card-check-width", "0.3rem"),
                        decl("--fandhe-checkbox-card-check-height", "0.6rem"),
                        decl("--fandhe-checkbox-card-dash-width", "0.6rem"),
                        decl(
                            "--fandhe-checkbox-card-label-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                        decl(
                            "--fandhe-checkbox-card-description-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl("--fandhe-checkbox-card-gap", "var(--fandhe-space-3)"),
                    ],
                ),
                (
                    Size::Xl,
                    vec![
                        decl("--fandhe-checkbox-card-padding", "var(--fandhe-space-6)"),
                        decl("--fandhe-checkbox-card-control-size", "1.5rem"),
                        decl("--fandhe-checkbox-card-check-width", "0.35rem"),
                        decl("--fandhe-checkbox-card-check-height", "0.7rem"),
                        decl("--fandhe-checkbox-card-dash-width", "0.7rem"),
                        decl(
                            "--fandhe-checkbox-card-label-font-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                        decl(
                            "--fandhe-checkbox-card-description-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                        decl("--fandhe-checkbox-card-gap", "var(--fandhe-space-4)"),
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

/// この styled CheckboxCard が生成する静的 CSS 全量を返す（決定的。
/// [`crate::checkbox::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与する
/// 唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去して
/// から合成する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::checkbox_card::{self, CheckboxProps};
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = checkbox_card::root(Size::Md, ColorPalette::Accent, &CheckboxProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="checkbox-card" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    props: &CheckboxProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let attrs = drop_reserved(drop_class_attr(attrs), STATE_RESERVED);
    let mut merged = state_attrs(props);
    merged.push(("class", class.as_str()));
    merged.extend(attrs);
    ANATOMY.part("root", "label", merged, children)
}

/// control パーツ（`<div>`）。indicator と content を横に並べる領域。
#[must_use]
pub fn control<'a>(
    props: &CheckboxProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// content パーツ（`<div>`）。label/description/addon を縦に積む。
#[must_use]
pub fn content<'a>(
    props: &CheckboxProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(attrs);
    ANATOMY.part("content", "div", merged, children)
}

/// label パーツ（`<div>`）。見出しテキスト。
#[must_use]
pub fn label<'a>(
    props: &CheckboxProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(attrs);
    ANATOMY.part("label", "div", merged, children)
}

/// description パーツ（`<div>`）。補足テキスト。
#[must_use]
pub fn description<'a>(
    props: &CheckboxProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(attrs);
    ANATOMY.part("description", "div", merged, children)
}

/// addon パーツ（`<div>`）。任意の付加コンテンツ（アイコン等）。
#[must_use]
pub fn addon<'a>(
    props: &CheckboxProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(attrs);
    ANATOMY.part("addon", "div", merged, children)
}

/// indicator パーツ（`<div>`）。チェックボックス外枠
/// （[`crate::checkbox::control`] 相当の見た目）。
#[must_use]
pub fn indicator<'a>(
    props: &CheckboxProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(attrs);
    ANATOMY.part("indicator", "div", merged, children)
}

/// indicator-check パーツ（`<div>`）。チェックマーク本体
/// （[`crate::checkbox`] headless 側 `indicator` 相当）。
/// [`CheckedState::Unchecked`] のときは `hidden` 存在属性を付与する
/// （モジュール rustdoc「`indicator_check` の `hidden` 属性意味論」節参照）。
#[must_use]
pub fn indicator_check<'a>(
    props: &CheckboxProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(drop_reserved(attrs, STATE_RESERVED), &["hidden"]);
    let mut merged = state_attrs(props);
    if props.checked == CheckedState::Unchecked {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("indicator-check", "div", merged, children)
}

/// フレームワークが `hidden_input` に固定する属性キー一覧
/// （`crates/headless-ui/src/checkbox.rs` の `HIDDEN_INPUT_RESERVED` と同型）。
const HIDDEN_INPUT_RESERVED: &[&str] = &[
    "type",
    "checked",
    "aria-checked",
    "aria-invalid",
    "name",
    "value",
    "disabled",
    "required",
];

/// hidden-input パーツ（`<input type="checkbox">`）。フォーム送信の実体。
///
/// 属性契約は `crates/headless-ui/src/checkbox.rs::hidden_input` と同一
/// ロジック（両ファイルを合わせて確認する契約、モジュール rustdoc 参照）。
#[must_use]
pub fn hidden_input<'a>(
    props: &CheckboxProps,
    name: &'a str,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let attrs = drop_reserved(drop_reserved(attrs, STATE_RESERVED), HIDDEN_INPUT_RESERVED);
    let mut merged = state_attrs(props);
    merged.push(("type", "checkbox"));
    merged.push(("name", name));
    merged.push(("value", value));
    if props.checked == CheckedState::Checked {
        merged.push(("checked", ""));
    }
    if props.checked == CheckedState::Indeterminate {
        merged.push(aria_checked(AriaChecked::Mixed));
    }
    if props.invalid {
        merged.push(("aria-invalid", "true"));
    }
    if props.disabled {
        merged.push(("disabled", ""));
    }
    if props.required {
        merged.push(("required", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("hidden-input", "input", merged, vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_headless_ui::checkbox::Checkbox;
    use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

    fn unchecked() -> CheckboxProps {
        CheckboxProps::default()
    }

    fn checked() -> CheckboxProps {
        CheckboxProps {
            checked: CheckedState::Checked,
            ..CheckboxProps::default()
        }
    }

    fn indeterminate() -> CheckboxProps {
        CheckboxProps {
            checked: CheckedState::Indeterminate,
            ..CheckboxProps::default()
        }
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="checkbox-card"][data-part="indicator"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_root_to_checked_and_indeterminate_state() {
        let css = stylesheet();
        assert!(css
            .contains(r#"[data-scope="checkbox-card"][data-part="root"][data-state="checked"] {"#));
        assert!(css.contains(
            r#"[data-scope="checkbox-card"][data-part="root"][data-state="indeterminate"] {"#
        ));
    }

    #[test]
    fn stylesheet_links_root_to_focus_within_outline() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="checkbox-card"][data-part="root"]:focus-within {"#));
    }

    #[test]
    fn stylesheet_links_root_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="checkbox-card"][data-part="root"][data-disabled] {"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn hidden_input_is_visually_hidden_not_display_none() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="checkbox-card"][data-part="hidden-input"] {"#));
        assert!(css.contains("clip: rect(0, 0, 0, 0);"));
        assert!(!css.contains("display: none"));
    }

    #[test]
    fn indicator_check_base_has_no_display_declaration() {
        let css = stylesheet();
        let start = css
            .find(r#"[data-scope="checkbox-card"][data-part="indicator-check"] {"#)
            .expect("indicator-check base block must exist");
        let end = css[start..]
            .find('}')
            .map(|i| start + i)
            .unwrap_or(css.len());
        assert!(
            !css[start..end].contains("display"),
            "indicator-check base block must not declare display: {}",
            &css[start..end]
        );
    }

    // --- イシュー #1458: 内部レイアウト slot・size 軸 ---

    /// `--fandhe-checkbox-card-gap`（`control` の `gap` が参照）が xs〜xl で
    /// spacing トークン経由の単調増加になることを固定する。
    #[test]
    fn size_variants_set_gap_custom_property_monotonically() {
        let css = stylesheet();
        let expected = [
            (Size::Xs, "var(--fandhe-space-1)"),
            (Size::Sm, "var(--fandhe-space-1-5)"),
            (Size::Md, "var(--fandhe-space-2-5)"),
            (Size::Lg, "var(--fandhe-space-3)"),
            (Size::Xl, "var(--fandhe-space-4)"),
        ];
        for (size, gap) in expected {
            let selector = format!(
                r#"[data-scope="checkbox-card"][data-part="root"].fd-checkbox-card--size-{}"#,
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
            let expected_decl = format!("--fandhe-checkbox-card-gap: {gap};");
            assert!(
                block.contains(&expected_decl),
                "size={size:?} variant block missing {expected_decl}: {block}"
            );
        }
    }

    /// `--fandhe-checkbox-card-padding` が生 rem リテラルではなく spacing
    /// トークンで xs〜xl 定義されることを固定する。
    #[test]
    fn size_variants_padding_uses_spacing_tokens() {
        let css = stylesheet();
        let expected = [
            (Size::Xs, "var(--fandhe-space-2)"),
            (Size::Sm, "var(--fandhe-space-3)"),
            (Size::Md, "var(--fandhe-space-4)"),
            (Size::Lg, "var(--fandhe-space-5)"),
            (Size::Xl, "var(--fandhe-space-6)"),
        ];
        for (size, padding) in expected {
            let selector = format!(
                r#"[data-scope="checkbox-card"][data-part="root"].fd-checkbox-card--size-{}"#,
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
            let expected_decl = format!("--fandhe-checkbox-card-padding: {padding};");
            assert!(
                block.contains(&expected_decl),
                "size={size:?} variant block missing {expected_decl}: {block}"
            );
            assert!(
                !block.contains("--fandhe-checkbox-card-padding: 0.")
                    && !block.contains("--fandhe-checkbox-card-padding: 1."),
                "size={size:?} variant block still uses a raw rem literal for padding: {block}"
            );
        }
    }

    /// control 寸法（`--fandhe-checkbox-card-control-size`）が xs〜xl で
    /// 単調増加することを rem 値の parse で固定する（[`crate::checkbox`] と
    /// 同一値へそろえる意図的判断、モジュール rustdoc 参照）。
    #[test]
    fn size_variants_control_size_is_monotonic() {
        let css = stylesheet();
        let mut sizes_rem = Vec::new();
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl] {
            let selector = format!(
                r#"[data-scope="checkbox-card"][data-part="root"].fd-checkbox-card--size-{}"#,
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
                .find("--fandhe-checkbox-card-control-size: ")
                .unwrap_or_else(|| panic!("control-size declaration not found in {block}"));
            let after = &block[decl_start + "--fandhe-checkbox-card-control-size: ".len()..];
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
                "control-size not monotonic: {prev_size:?}={prev} >= {next_size:?}={next}"
            );
        }
    }

    /// `--fandhe-checkbox-card-description-font-size` が xs〜xl すべてで
    /// 定義され、`description` base がそれを参照することを固定する。
    #[test]
    fn size_variants_set_description_font_size_custom_property() {
        let css = stylesheet();
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl] {
            let selector = format!(
                r#"[data-scope="checkbox-card"][data-part="root"].fd-checkbox-card--size-{}"#,
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
                css[start..block_end].contains("--fandhe-checkbox-card-description-font-size"),
                "size={size:?} variant block missing --fandhe-checkbox-card-description-font-size: {}",
                &css[start..block_end]
            );
        }
        let description_selector = r#"[data-scope="checkbox-card"][data-part="description"] {"#;
        let start = css
            .find(description_selector)
            .expect("description base block must exist");
        let block_end = css[start..]
            .find('}')
            .map(|i| start + i)
            .unwrap_or(css.len());
        assert!(
            css[start..block_end].contains(
                "font-size: var(--fandhe-checkbox-card-description-font-size, var(--fandhe-font-font-size-sm));"
            ),
            "description base block missing size-linked font-size: {}",
            &css[start..block_end]
        );
    }

    /// label が [`crate::checkbox`] と同型の型階層（medium font-weight・
    /// 前景色・行送り・誤選択防止）を持つことを固定する。
    #[test]
    fn label_has_typography_hierarchy_declarations() {
        let css = stylesheet();
        let selector = r#"[data-scope="checkbox-card"][data-part="label"] {"#;
        let start = css
            .find(selector)
            .unwrap_or_else(|| panic!("label base selector not found in {css}"));
        let block_end = css[start..]
            .find('}')
            .map(|i| start + i)
            .unwrap_or(css.len());
        let block = &css[start..block_end];
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
        assert!(
            block.contains("user-select: none;"),
            "label block missing user-select: {block}"
        );
    }

    /// `indicator` の transition が motion トークン経由になり、直書きの
    /// `transition:` shorthand が残らないことを固定する（イシュー #1425）。
    #[test]
    fn indicator_transition_uses_motion_tokens() {
        // `SlotRecipe::base` は同一 slot への複数回登録を出力順で連結する
        // （`checkbox.rs` の `control` と同型）ため、`indicator` の base
        // 宣言はセレクタが同じ 2 つの `{...}` ブロックに分かれて出力される。
        // 単一ブロックの範囲切り出しではなく全文検索で確認する。
        let css = stylesheet();
        assert!(
            css.contains("transition-duration: var(--fandhe-motion-duration-fast);"),
            "stylesheet missing motion-token transition-duration for indicator: {css}"
        );
        assert!(
            !css.contains("transition: background 0.15s"),
            "stylesheet still contains raw transition shorthand for indicator: {css}"
        );
    }

    /// `indicator` が `data-invalid` へ [`crate::checkbox::control`] と同型の
    /// 枠線色変化を反映することを固定する（`root` 側は 1/2 の担当）。
    #[test]
    fn stylesheet_links_indicator_to_data_invalid_state() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="checkbox-card"][data-part="indicator"][data-invalid] {"#)
        );
    }

    // --- variant クラス ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &unchecked(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="checkbox-card""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &unchecked(),
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-checkbox-card--size-md"));
        assert!(html.contains("fd-checkbox-card--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-checkbox-card--size-xs"),
            (Size::Sm, "fd-checkbox-card--size-sm"),
            (Size::Md, "fd-checkbox-card--size-md"),
            (Size::Lg, "fd-checkbox-card--size-lg"),
            (Size::Xl, "fd-checkbox-card--size-xl"),
        ] {
            let html = render(&root(
                size,
                ColorPalette::Accent,
                &unchecked(),
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
                "fd-checkbox-card--color-palette-accent",
            ),
            (ColorPalette::Info, "fd-checkbox-card--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-checkbox-card--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-checkbox-card--color-palette-warning",
            ),
            (
                ColorPalette::Danger,
                "fd-checkbox-card--color-palette-danger",
            ),
            (
                ColorPalette::Neutral,
                "fd-checkbox-card--color-palette-neutral",
            ),
        ] {
            let html = render(&root(Size::Md, palette, &unchecked(), vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &unchecked(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &unchecked(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="checkbox-card""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn root_reflects_checked_and_indeterminate_props() {
        let checked_html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &checked(),
            vec![],
            vec![],
        ));
        assert!(checked_html.contains(r#"data-state="checked""#));

        let indeterminate_html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &indeterminate(),
            vec![],
            vec![],
        ));
        assert!(indeterminate_html.contains(r#"data-state="indeterminate""#));
    }

    #[test]
    fn indicator_check_hidden_when_unchecked_visible_otherwise() {
        let unchecked_html = render(&indicator_check(&unchecked(), vec![], vec![]));
        assert!(unchecked_html.contains(r#"hidden="""#));

        let checked_html = render(&indicator_check(&checked(), vec![], vec![]));
        assert!(!checked_html.contains("hidden="));
    }

    #[test]
    fn hidden_input_reflects_checked_and_indeterminate() {
        let checked_html = render(&hidden_input(&checked(), "terms", "on", vec![]));
        assert!(checked_html.contains(r#"checked="""#));

        let indeterminate_html = render(&hidden_input(&indeterminate(), "terms", "on", vec![]));
        assert!(indeterminate_html.contains(r#"aria-checked="mixed""#));
    }

    #[test]
    fn hidden_input_drops_caller_supplied_reserved_attrs() {
        let node = hidden_input(
            &checked(),
            "terms",
            "on",
            vec![
                ("type", "text"),
                ("name", "attacker"),
                ("value", "attacker"),
            ],
        );
        let html = render(&node);
        assert!(html.contains(r#"type="checkbox""#));
        assert!(html.contains(r#"name="terms""#));
        assert!(html.contains(r#"value="on""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &unchecked(),
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_label_children_are_escaped_on_render() {
        let html = render(&label(
            &unchecked(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn hidden_input_name_value_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&hidden_input(&unchecked(), PAYLOAD, PAYLOAD, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    // --- SSR/hydration 往復（受け入れ条件 1: headless Checkbox 状態機械を再利用） ---

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_checkbox_state_machine() {
        let mut cb = Checkbox::default();
        assert!(!cb.is_checked());

        let props = CheckboxProps {
            checked: if cb.is_checked() {
                CheckedState::Checked
            } else {
                CheckedState::Unchecked
            },
            ..CheckboxProps::default()
        };
        let ssr_html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &props,
            vec![],
            vec![],
        ));
        assert!(ssr_html.contains(r#"data-state="unchecked""#));

        assert!(dispatch(&mut cb, "toggle", ""));
        let hydrate_html = render(&render_for_hydration(&cb));
        assert!(hydrate_html.contains(r#"data-hydrate-checked="checked""#));

        let restored = Checkbox::from_hydration_attrs(&cb.hydration_attrs()).unwrap();
        assert_eq!(restored, cb);
    }
}

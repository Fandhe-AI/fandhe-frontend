//! styled RadioCard（イシュー #747、親 #520/#545、Phase 3 親 #736）。
//!
//! chakra-ui `forms/radio-card` 相当のカード型選択 UI。ark-ui には
//! radio-card 相当の anatomy は存在せず、chakra-ui が headless の
//! RadioGroup 状態機械の上に独自 slot recipe として実装している構図を、本
//! クレートでもそのまま踏襲する。**headless-ui には手を入れない**（受け入れ
//! 条件）。設計判断は [`crate::checkbox_card`] rustdoc と同型（本モジュールは
//! RadioCard 版）。セキュリティ不変条件も [`crate::checkbox_card`] と同型に
//! 統一した: 本モジュールは `STATE_RESERVED`/`HIDDEN_INPUT_RESERVED`/
//! `drop_reserved` により、呼び出し側 `attrs` による `data-state`/
//! `data-value`/`data-orientation`/`role`/`type`/`checked`/`name`/`value`/
//! `disabled` 等の予約キーなりすましを fail-closed で除去する（下記
//! 「セキュリティ不変条件」節参照）。イシュー #747 レビュー指摘を受けて
//! 是正済み。既存の [`crate::radio_group`] 側には同型の欠落が残っており、
//! 別イシューとして追跡する（下記「本イシューのスコープ外」節参照）。
//!
//! # anatomy は pre-styled 層で新規定義する（[`crate::card`]/[`crate::checkbox_card`] 先例準拠）
//!
//! `fandhe_frontend_headless_ui::radio_group`（イシュー #536/#595）の 6
//! anatomy パーツは `data-scope="radio-group"` に固定されており、カード型の
//! 10 パーツ構成へそのまま拡張できない。本モジュールは新規 anatomy
//! `data-scope="radio-card"` を [`fandhe_frontend_headless_ui::anatomy`] で
//! 定義する。既存 `radio-group` scope とは完全に独立するため、
//! [`crate::radio_group`] の CSS/属性契約と衝突しない。
//!
//! # 状態機械の再利用（受け入れ条件 1: 新規状態機械を作らない）
//!
//! [`fandhe_frontend_headless_ui::radio_group::RadioGroup`]（single-select
//! 状態機械、イシュー #524/#536）をそのまま利用する。本モジュールから
//! 再エクスポートしない（[`crate::radio_group`] は inherent `root()` を
//! 持たないため再エクスポートしても実害がないが、本モジュールは独自
//! anatomy を持ち `RadioGroup` に `item`/`item_control` 等の card 版
//! inherent メソッドは存在しないため、呼び出し側は
//! `fandhe_frontend_headless_ui::radio_group::RadioGroup` を直接 import し、
//! `.is_checked(value)` 等の問い合わせ結果を本モジュールのパーツ関数へ渡す
//! 契約とする）。
//!
//! # anatomy パーツ構成（chakra-ui slot 準拠、10 パーツ）
//!
//! - [`root`][]: `<div role="radiogroup">`。グループ全体、`size`/`palette`
//!   クラスを付与する唯一のパーツ。`orientation`/`labelled_by` は
//!   [`fandhe_frontend_headless_ui::radio_group::root`] と同じ任意入力。
//! - [`label`][]: `<span>`。グループ全体の見出し（[`crate::radio_group::label`]
//!   と同型、`<label>` ではなく `<span>` を採用する理由は headless 側
//!   モジュール doc 参照）。
//! - [`item`][]: `<label>`。選択肢 1 個のカード本体。
//! - [`item_control`][]: `<div>`。indicator と content を横に並べる領域。
//! - [`item_content`][]: `<div>`。text/description/addon を縦に積むコンテナ。
//! - [`item_text`][]: `<div>`。見出しテキスト。
//! - [`item_description`][]: `<div>`。補足テキスト。
//! - [`item_addon`][]: `<div>`。任意の付加コンテンツ（アイコン等）。
//! - [`item_indicator`][]: `<span>`。ラジオ円（[`crate::radio_group::item_control`]
//!   相当の box-shadow inset ドット描画）。headless radio_group の
//!   `item-control`（＝ラジオ円）と名前が同じだが、本モジュールの
//!   `item_indicator` は別 anatomy（`data-scope="radio-card"`）のパーツで
//!   あり衝突しない。
//! - [`item_hidden_input`][]: `<input type="radio">`。フォーム送信・
//!   キーボード操作・グループ内排他選択の実体（`crates/headless-ui/src/radio_group.rs`
//!   の `item_hidden_input` と同一属性契約、両ファイルを合わせて確認する契約）。
//!
//! # `item_hidden_input` の視覚的非表示化（[`crate::radio_group`] と同じ責務分担）
//!
//! headless 層は視覚的な非表示化を行わない契約（`crates/headless-ui/src/radio_group.rs`
//! 参照）のため、styled 層である本モジュールが visually-hidden パターン
//! （[`crate::radio_group::item_hidden_input`] の CSS と同一の 9 宣言）で
//! 覆い隠し、`item_indicator` をカスタムラジオ円として描画する。
//!
//! # フォーカスリング（イシュー #1424 規約への canonical 化、イシュー #1491）
//!
//! 実フォーカスは [`item_hidden_input`] が受けるため、[`crate::radio_group`]
//! の `item` と同型の [`StateCondition::FocusWithin`]（no-JS フォールバック）
//! のみを [`item`] へ登録する。宣言列は `crate::recipe::focus_ring_declarations`
//! （[`crate::recipe::FocusRingColor::Palette`] /
//! [`crate::recipe::FocusRingOffset::Outside`]）が組み立てる canonical 形
//! （`outline`/`outline-offset` の 2 宣言、トークン参照）へ統一し、以前の
//! `:focus-within` 直書きは廃止した
//! （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` 参照）。
//! `data-focus-visible`（wasm 配線によるキーボード操作専用リング）は
//! `crates/wasm-full/src/focus_visible.rs` の `(scope, part)` マッピングに
//! `"radio-card"` が未登録のため本イシューでは実装しない（フォローアップ、
//! PR 本文参照）。
//!
//! # スタイル調整（イシュー #1491、`item` slot のみ）
//!
//! 親 #1490（chakra-ui `radio-card` / Radix Themes `radio-cards` 準拠の
//! 見た目調整）の 1/2 分割で、担当範囲は `item`（カード枠）の状態表現に限る。
//! `item-indicator`/`item-text`/`item-description`/`item-addon`/
//! `item-control`/`item-content`/`item-hidden-input` および size バリアント
//! は兄弟イシュー #1492（2/2）の担当であり、本イシューでは変更しない。
//!
//! 本イシューで `item` へ適用した変更:
//! - hover: `crate::recipe::hover_bg_muted` を base へ、
//!   `crate::recipe::hover_surface_declarations` を
//!   [`StateCondition::Hover`] へ登録し、参照サイトと同様のカード面変化を
//!   反映する。
//! - disabled: `crate::recipe::disabled_declarations` へ置換（値は不変、
//!   宣言順のみ変わる）。
//! - transition: `crate::recipe::transition_declarations`
//!   （`background, border-color, box-shadow` /
//!   [`crate::recipe::MotionDuration::Fast`]）へ置換し、reduced-motion は
//!   `crate::theme::Theme::to_css` の一括上書きに委ねる。
//! - `data-invalid`: `border-color` を `var(--fandhe-color-danger)` へ切り替える
//!   状態を新規登録する（`box-shadow` は palette 色のまま残し、枠線のみで
//!   invalid を表現する方針は [`crate::checkbox_card`] の `root` と同型）。
//!   headless `radio_group` は現状 `data-invalid` を出力しないため
//!   （Field #538 連携は headless 側の将来イシュー）、呼び出し側 `attrs`
//!   パススルー（[`ITEM_RESERVED`] は非予約）で付与する契約とする。
//!   `data-invalid` は境界線の視覚表現に留まる CSS フックであり、
//!   支援技術への状態通知は別途 [`item_hidden_input`] の `attrs` へ
//!   `aria-invalid="true"` を渡して併用することを呼び出し側の責務とする
//!   （codex-review 指摘、イシュー #1491）。
//!
//! 意図的に参照サイトへ合わせない点（親 #1490 の比較チェックリスト・イシュー
//! 本文参照）:
//! - **variant 軸**（chakra `surface/subtle/outline/solid`、Radix
//!   `surface/classic`）は追加しない。`root()`/`item()` のシグネチャ変更
//!   （破壊的変更）を伴い、`checkbox`/`checkbox-card`/`radio-group` 等 Forms
//!   家族の軸語彙と横断で判断すべき事項のため（[`crate::checkbox_card`] の
//!   同型判断と同じ）。
//! - **`data-readonly`** は参照サイトいずれも視覚差がないため非視覚化を維持する。
//! - **`:focus-within` 継続**（`data-focus-visible` の wasm 配線は別クレート・
//!   別イシューの担当、上記フォーカスリング節参照）。
//! - **内部レイアウト**（label / description / addon / indicator / size
//!   バリアント）はイシュー #1492 の担当（下記「スタイル調整（イシュー
//!   #1492、内部レイアウト slot と size 軸）」節参照）。
//!
//! # スタイル調整（イシュー #1492、内部レイアウト slot と size 軸）
//!
//! 親 #1490 の 2/2 分割で、担当範囲は上記 #1491（`item` slot の状態表現）
//! を除く内部レイアウト slot（[`label`]/[`item_control`]/[`item_content`]/
//! [`item_text`]/[`item_description`]/[`item_addon`]/[`item_indicator`]）と
//! size バリアント（[`recipe`] が `root` へ登録する `--fandhe-radio-card-*`
//! custom property 群）。同型の先例 checkbox-card 2/2（イシュー #1458）の
//! 変更パターンをそのまま写像する。
//!
//! 本イシューで適用した変更:
//! - [`item_control`]: `gap` を `var(--fandhe-space-2)` 固定から
//!   `var(--fandhe-radio-card-gap, var(--fandhe-space-2))`（size 連動、
//!   新設）へ変更した。
//! - [`item_text`]: `line-height: var(--fandhe-font-line-height-normal)` と
//!   `user-select: none` を追加した（checkbox-card `label`・radio_group
//!   `item-text` #1495 と同語彙）。
//! - [`item_description`]: `font-size` を固定 `sm` から
//!   `var(--fandhe-radio-card-description-font-size, var(--fandhe-font-font-size-sm))`
//!   （label より常に 1 段下、新設）へ変更し、`line-height` を追加した。
//!   色は `fg-muted` トークンのまま維持する（chakra の `opacity: 0.64` に
//!   対する意図的差分。ダーク追従のため、checkbox-card と同判断）。
//! - [`item_addon`]: `align-items: center`/`gap`/`font-size`
//!   （description と同じ custom property）/`color` を追加した。chakra の
//!   下部フッター帯（border-top）は anatomy 転用しない意図的差分。
//! - [`item_indicator`]: `box-sizing: border-box` を追加（checkbox
//!   `control` #1454 / radio_group `item-control` #1494 と寸法解釈を
//!   統一）。`transition_declarations` による motion トークン経由の
//!   transition を純追加し（`base` の同一 slot 複数回登録は出力順で
//!   連結される仕様、checkbox-card `indicator` と同型）、
//!   `[data-invalid]` 状態（`border-color: var(--fandhe-color-danger)`）を
//!   新規登録した（checkbox-card `indicator` #1458 と同型の視覚言語。
//!   `item` 側の invalid 表現は 1/2 実装済みのため触らない）。
//! - [`label`]（グループ見出し）: radio_group `label` #1495 と同型に
//!   `font-size` を `var(--fandhe-radio-card-label-font-size, ...)`
//!   （size 連動、[`item_text`] と custom property を共有）へ変更し、
//!   `font-weight: medium`・`line-height: normal` を追加した。
//!   `margin-bottom` は維持する。
//! - size バリアントを 5 段の `.variant(Size::*, "root", ...)` 手書きから
//!   [`crate::recipe::SlotRecipe::size_variants`]（既定 `md` を構造的に
//!   保証、イシュー #1424）へ移行し、padding を生 rem リテラルから
//!   spacing トークン（chakra md=16px 基準・4px 格子で単調増加、
//!   checkbox-card と同値系列）へ、control-size を checkbox 家族と同一値へ
//!   載せ替えた（chakra のカード indicator が通常より 1 段大きい点は意図
//!   的に合わせない）。dot-inset・label-font-size は既存値を維持し、
//!   description-font-size・gap の 2 custom property を新設した。
//!
//! 意図的に参照サイトへ合わせない点:
//! - **indicator の左右位置**: CSS `order` は使わず DOM 順に委ねる
//!   （showcase・呼び出し側が [`item_content`] → [`item_indicator`] の順で
//!   子を並べれば右配置になる。宣言的な CSS レイアウト操作より呼び出し側
//!   の DOM 構成に委ねる方針）。
//! - **description の色**: 上記のとおり `fg-muted` トークンを維持する。
//! - **addon のフッター帯**: chakra の border-top 区切りは転用しない。
//! - **root グループ間隔**（`--fandhe-space-2` 固定）: [`crate::radio_group`]
//!   と異なりカード間隔は size 非連動のまま維持する（chakra-ui の
//!   `radio-card` グループ間隔も size 非連動であることに基づく意図的判断）。
//! - **disabled の二重減光**: 追加しない。[`item`] の
//!   `disabled_declarations()`（`opacity: 0.5`）が子孫へ継承されるため
//!   [`item_indicator`] 側で重ねて減光しない。
//! - **indicator の hover**: 追加しない。[`item_indicator`] は
//!   `<span>` の非インタラクティブ slot であり、hover は祖先 [`item`]
//!   （`<label>`）側で表現する契約のまま。
//! - **variant 軸**: 上記 #1491 節と同じ理由で追加しない。
//!
//! # `size`/`palette` variant
//!
//! [`crate::radio_group`] rustdoc「複合部品の variant 統一方針」節（#708）と
//! 同型。`size`（[`Size`]）は [`root`] へのみクラスを付与し、[`recipe`] が
//! 登録する `--fandhe-radio-card-*` の root スコープ custom property 経由で
//! `item`/`item_indicator`/`item_text` の寸法を切り替える。`palette`
//! （[`ColorPalette`]）は [`crate::recipe::palette_scale_declarations`] を `root`
//! へ登録し、選択済みカードの枠線・背景・ドット色を
//! `var(--fandhe-palette, ...)` 経由で切り替える。
//!
//! # セキュリティ不変条件
//!
//! `raw_html()` は使用しない。CSS 宣言値はすべて静的リテラルで、動的値
//! （`value`/`name`/attrs/children）は
//! [`fandhe_frontend_headless_ui::fandhe_frontend_core::render`] の既定
//! エスケープを必ず経由する（REQ-1）。呼び出し側 `attrs` の `class` は
//! [`drop_class_attr`] で除去してから合成し、`class` 属性は常に単一
//! （[`crate::radio_group::root`] と同型）。[`ROOT_RESERVED`]（`role`/
//! `data-orientation`/`aria-orientation`/`aria-labelledby`/`data-disabled`）・
//! [`STATE_RESERVED`]（`data-state`/`data-value`/`data-disabled`）・
//! [`HIDDEN_INPUT_RESERVED`]（`type`/`value`/`data-state`/`name`/`checked`/
//! `disabled`）の各予約キーは、パーツごとに [`drop_reserved`] で呼び出し側
//! `attrs` から fail-closed に除去してから合成する
//! （[`crate::checkbox_card`] の `STATE_RESERVED`/`HIDDEN_INPUT_RESERVED`/
//! `drop_reserved` と同型の判断を本モジュールで独立に実装する —
//! [`fandhe_frontend_headless_ui::anatomy::Anatomy::part`] は
//! `data-scope`/`data-part` のみを守るため、それ以外の予約キー保護は各
//! styled 部品自身の責務であることは headless 側モジュール doc の
//! 「セキュリティ不変条件」節と同じ）。呼び出し側が誤ってこれらのキーを
//! `attrs` へ混入させても、フレームワーク側の固定値が優先され
//! `type="radio" type="text"` のような重複属性は出力されない
//! （[`crate::radio_group`] 側には同型の欠落が残っており、別イシューとして
//! 追跡する）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `fandhe-frontend-wasm-full` の focus/クリック配線（`(scope, part)` を
//!   `("radio-card", "item-hidden-input") -> "item"` へ写像し
//!   `data-focus-visible` を CSS で伝える対応、headless 配線の select
//!   アクション写像の card scope 対応）。
//! - `examples/headless-pre-styled-ui` への追随（pre-styled-ui 公開後に
//!   別 PR で対応）。
//! - [`crate::radio_group`] 側の同型の予約キー保護欠落（本モジュールの
//!   是正で非対称になった。out-of-scope-tracking に従い別イシューとして
//!   起票・追跡する）。
//!
//! # `data-value` 語彙（イシュー #1063）
//!
//! `data-value`（[`item`] が出力、値は選択肢の値）は
//! `fandhe_frontend_headless_ui::radio_group`/`checkbox_group`/
//! `toggle_group`/`tree_view`/`rating_group` の各パーツが出力する
//! `data-value` と同一意味論（「項目の値」）の共有語彙である
//! （`docs/design/pre-styled-ui-data-attr-vocabulary.md` 規約 B-2）。
//! いずれの層にも `data_attrs::data_value` ヘルパは未整備だが、本イシューの
//! 範囲では新設しない（headless-ui 側の公開 API 拡張が伴うため。設計文書
//! 「スコープ外」節参照）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    palette_scale_declarations, transition_declarations, ColorPalette, FocusRingColor,
    FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};
use fandhe_frontend_headless_ui::aria::{aria_labelledby, aria_orientation, role};
pub use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::data_attrs::{data_disabled, data_orientation, data_state};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="radio-card"` を固定した本コンポーネントの anatomy
/// （既存 `data-scope="radio-group"` とは独立、モジュール冒頭 rustdoc 参照）。
const ANATOMY: Anatomy = anatomy("radio-card");

/// [`root`] が固定付与する属性キー一覧（呼び出し側 `attrs` からの偽装を
/// fail-closed で除去する対象。`class` は [`drop_class_attr`] が別途処理する
/// ため含めない。モジュール冒頭 rustdoc「セキュリティ不変条件」節参照）。
const ROOT_RESERVED: &[&str] = &[
    "role",
    "data-orientation",
    "aria-orientation",
    "aria-labelledby",
    "data-disabled",
];

/// [`item`]/[`item_control`]/[`item_indicator`] が共通で固定付与する属性キー
/// 一覧（`crates/pre-styled-ui/src/checkbox_card.rs` の `STATE_RESERVED` と
/// 同型の判断、モジュール冒頭 rustdoc 参照）。[`item`] のみさらに `data-value`
/// を追加で保護する。
const STATE_RESERVED: &[&str] = &["data-state", "data-disabled"];

/// [`item`] がさらに固定付与する属性キー（[`STATE_RESERVED`] に加えて保護）。
const ITEM_RESERVED: &[&str] = &["data-state", "data-value", "data-disabled"];

/// [`item_hidden_input`] が固定付与する属性キー一覧
/// （`crates/pre-styled-ui/src/checkbox_card.rs` の `HIDDEN_INPUT_RESERVED` と
/// 同型）。
const HIDDEN_INPUT_RESERVED: &[&str] =
    &["type", "value", "data-state", "name", "checked", "disabled"];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（`crates/headless-ui/src/checkbox.rs`・
/// `crates/pre-styled-ui/src/checkbox_card.rs` の `drop_reserved` と同型）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "item",
    "item-control",
    "item-content",
    "item-text",
    "item-description",
    "item-addon",
    "item-indicator",
    "item-hidden-input",
];

/// この styled RadioCard の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("radio-card", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "horizontal"),
            vec![decl("flex-direction", "row")],
        )
        .base(
            "label",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "font-size",
                    "var(--fandhe-radio-card-label-font-size, var(--fandhe-font-font-size-sm))",
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
                decl("align-items", "flex-start"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("cursor", "pointer"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
                decl("padding", "var(--fandhe-radio-card-padding, 0.75rem)"),
                decl("background", "var(--fandhe-color-bg)"),
                hover_bg_muted(),
            ]
            .into_iter()
            .chain(transition_declarations(
                "background, border-color, box-shadow",
                MotionDuration::Fast,
            ))
            .collect(),
        )
        .state(
            "item",
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
            "item",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // イシュー #1424: フォーカスリングは `crate::recipe::focus_ring_declarations`
        // の canonical 形（outline + outline-offset の 2 宣言、トークン参照）
        // へ統一する。実フォーカスは item-hidden-input が受けるため、
        // 祖先 item（`<label>`）へ `:focus-within` で no-JS フォールバックの
        // リングを反映する（[`crate::radio_group`] の `item` と同型）。
        .state(
            "item",
            StateCondition::FocusWithin,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        // イシュー #1491: hover 時のカード面変化（`hover_bg_muted()` で
        // `--fandhe-hover-bg` を base に定義し、`hover_surface_declarations()`
        // が参照する。`item` は `<label>` + `cursor: pointer` のインタラ
        // クティブ slot のため対象とする。
        .state("item", StateCondition::Hover, hover_surface_declarations())
        .base(
            "item-control",
            vec![
                decl("display", "flex"),
                decl("align-items", "flex-start"),
                decl("gap", "var(--fandhe-radio-card-gap, var(--fandhe-space-2))"),
                decl("flex", "1"),
            ],
        )
        .base(
            "item-content",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("flex", "1"),
            ],
        )
        .base(
            "item-text",
            vec![
                decl(
                    "font-size",
                    "var(--fandhe-radio-card-label-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("user-select", "none"),
            ],
        )
        .base(
            "item-description",
            vec![
                decl(
                    "font-size",
                    "var(--fandhe-radio-card-description-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "item-addon",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "font-size",
                    "var(--fandhe-radio-card-description-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "item-indicator",
            vec![
                decl("display", "inline-flex"),
                decl("box-sizing", "border-box"),
                decl("width", "var(--fandhe-radio-card-control-size, 1rem)"),
                decl("height", "var(--fandhe-radio-card-control-size, 1rem)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "50%"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("flex-shrink", "0"),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結されるため、
        // 上記 base ブロックを書き換えずに純追加する（checkbox-card
        // `indicator` #1458 と同型のパターン）。
        .base(
            "item-indicator",
            transition_declarations("background, border-color", MotionDuration::Fast),
        )
        .state(
            "item-indicator",
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
                decl(
                    "box-shadow",
                    "inset 0 0 0 var(--fandhe-radio-card-dot-inset, 3px) var(--fandhe-color-bg)",
                ),
            ],
        )
        // headless 層が invalid な選択肢へ出す `data-invalid` を item-indicator
        // slot へ反映する（checkbox-card `indicator` #1458 と同型の視覚言語。
        // `item` 側の invalid 表現は 1/2〔PR #1768〕の担当のため本規則は
        // 追加しない）。
        .state(
            "item-indicator",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        // item-hidden-input の視覚的非表示化（[`crate::radio_group::item_hidden_input`]
        // と同一の visually-hidden パターン）。
        .base(
            "item-hidden-input",
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
        // イシュー #1492: 5 段の `.variant(Size::*, "root", ...)` を個別に
        // 手書きする代わりに `size_variants`（イシュー #1424 の共通生成
        // 手段、checkbox-card #1458 と同型）を使い、既定 `md` の設定漏れを
        // 構造的に防ぐ（従来の `.default_variant(Size::Md)` 明示呼び出しは
        // `size_variants` が構造的に保証するため削除）。padding は生 rem
        // リテラルではなく spacing トークン（chakra md=16px を基準に 4px
        // 格子で単調増加、checkbox-card と同値系列）へ載せ替える。
        // control-size は checkbox 家族と同一値へそろえる（fandhe の
        // checkbox 系で indicator 寸法を統一する意図的判断。chakra のカード
        // indicator が通常より 1 段大きい点は意図的に合わせない）。
        // description の size 連動フォントサイズ（label より常に 1 段下）と
        // item-control の gap（`--fandhe-radio-card-gap`）を新設する。
        // dot-inset・label-font-size は既存値を維持する。
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![
                        decl("--fandhe-radio-card-padding", "var(--fandhe-space-2)"),
                        decl("--fandhe-radio-card-control-size", "0.75rem"),
                        decl("--fandhe-radio-card-dot-inset", "1px"),
                        decl(
                            "--fandhe-radio-card-label-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                        decl(
                            "--fandhe-radio-card-description-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                        decl("--fandhe-radio-card-gap", "var(--fandhe-space-1)"),
                    ],
                ),
                (
                    Size::Sm,
                    vec![
                        decl("--fandhe-radio-card-padding", "var(--fandhe-space-3)"),
                        decl("--fandhe-radio-card-control-size", "0.875rem"),
                        decl("--fandhe-radio-card-dot-inset", "2px"),
                        decl(
                            "--fandhe-radio-card-label-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl(
                            "--fandhe-radio-card-description-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                        decl("--fandhe-radio-card-gap", "var(--fandhe-space-1-5)"),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl("--fandhe-radio-card-padding", "var(--fandhe-space-4)"),
                        decl("--fandhe-radio-card-control-size", "1rem"),
                        decl("--fandhe-radio-card-dot-inset", "3px"),
                        decl(
                            "--fandhe-radio-card-label-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl(
                            "--fandhe-radio-card-description-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl("--fandhe-radio-card-gap", "var(--fandhe-space-2-5)"),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        decl("--fandhe-radio-card-padding", "var(--fandhe-space-5)"),
                        decl("--fandhe-radio-card-control-size", "1.25rem"),
                        decl("--fandhe-radio-card-dot-inset", "4px"),
                        decl(
                            "--fandhe-radio-card-label-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                        decl(
                            "--fandhe-radio-card-description-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl("--fandhe-radio-card-gap", "var(--fandhe-space-3)"),
                    ],
                ),
                (
                    Size::Xl,
                    vec![
                        decl("--fandhe-radio-card-padding", "var(--fandhe-space-6)"),
                        decl("--fandhe-radio-card-control-size", "1.5rem"),
                        decl("--fandhe-radio-card-dot-inset", "5px"),
                        decl(
                            "--fandhe-radio-card-label-font-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                        decl(
                            "--fandhe-radio-card-description-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                        decl("--fandhe-radio-card-gap", "var(--fandhe-space-4)"),
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

/// この styled RadioCard が生成する静的 CSS 全量を返す（決定的。
/// [`crate::radio_group::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去
/// してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::radio_group::root`] と同じ属性契約
/// （`role="radiogroup"`/`orientation`/`labelled_by`）を独自 anatomy 上で
/// 組み立てる。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::radio_card;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = radio_card::root(
///     Size::Md,
///     ColorPalette::Accent,
///     false,
///     None,
///     None,
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"data-scope="radio-card" data-part="root""#));
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
    let mut merged: Vec<(&str, &str)> = vec![role("radiogroup"), ("class", class.as_str())];
    if let Some(orientation) = orientation {
        merged.push(aria_orientation(orientation));
        merged.push(data_orientation(orientation));
    }
    if let Some(id) = labelled_by {
        merged.push(aria_labelledby(id));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(drop_reserved(drop_class_attr(attrs), ROOT_RESERVED));
    ANATOMY.part("root", "div", merged, children)
}

/// label パーツ（`<span>`）。グループ全体の見出し
/// （[`crate::radio_group::label`] と同型）。
#[must_use]
pub fn label<'a>(id: Option<&'a str>, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("label", "span", merged, children)
}

/// item パーツ（`<label>`）。選択肢 1 個のカード本体。
#[must_use]
pub fn item<'a>(
    checked: bool,
    disabled: bool,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        data_state(if checked { "checked" } else { "unchecked" }),
        ("data-value", value),
    ];
    merged.extend(data_disabled(disabled));
    merged.extend(drop_reserved(attrs, ITEM_RESERVED));
    ANATOMY.part("item", "label", merged, children)
}

/// item-control パーツ（`<div>`）。indicator と content を横に並べる領域。
#[must_use]
pub fn item_control<'a>(
    checked: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![data_state(if checked { "checked" } else { "unchecked" })];
    merged.extend(data_disabled(disabled));
    merged.extend(drop_reserved(attrs, STATE_RESERVED));
    ANATOMY.part("item-control", "div", merged, children)
}

/// item-content パーツ（`<div>`）。item-text/item-description/item-addon を
/// 縦に積むコンテナ。
#[must_use]
pub fn item_content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-content", "div", attrs, children)
}

/// item-text パーツ（`<div>`）。選択肢の見出しテキスト。
#[must_use]
pub fn item_text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-text", "div", attrs, children)
}

/// item-description パーツ（`<div>`）。選択肢の補足テキスト。
#[must_use]
pub fn item_description<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-description", "div", attrs, children)
}

/// item-addon パーツ（`<div>`）。任意の付加コンテンツ（アイコン等）。
#[must_use]
pub fn item_addon<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-addon", "div", attrs, children)
}

/// item-indicator パーツ（`<span>`）。ラジオ円
/// （[`crate::radio_group::item_control`] 相当の box-shadow inset ドット
/// 描画。名前は headless `radio_group::item_control` と異なるが、混同を
/// 避けるため本モジュールでは `item_indicator` と命名する）。
#[must_use]
pub fn item_indicator<'a>(checked: bool, disabled: bool, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![data_state(if checked { "checked" } else { "unchecked" })];
    merged.extend(data_disabled(disabled));
    merged.extend(drop_reserved(attrs, STATE_RESERVED));
    ANATOMY.part("item-indicator", "span", merged, vec![])
}

/// item-hidden-input パーツ（`<input type="radio">`）。フォーム送信・
/// キーボード操作・グループ内排他選択の実体
/// （`crates/headless-ui/src/radio_group.rs::item_hidden_input` と同一属性
/// 契約、両ファイルを合わせて確認する契約、モジュール rustdoc 参照）。
#[must_use]
pub fn item_hidden_input<'a>(
    checked: bool,
    disabled: bool,
    name: Option<&'a str>,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "radio"),
        ("value", value),
        data_state(if checked { "checked" } else { "unchecked" }),
    ];
    if let Some(name) = name {
        merged.push(("name", name));
    }
    if checked {
        merged.push(("checked", ""));
    }
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(drop_reserved(attrs, HIDDEN_INPUT_RESERVED));
    ANATOMY.part("item-hidden-input", "input", merged, vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_headless_ui::radio_group::RadioGroup;
    use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="radio-card"][data-part="item-indicator"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn item_hidden_input_is_visually_hidden() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="radio-card"][data-part="item-hidden-input"]"#));
        assert!(css.contains("clip: rect(0, 0, 0, 0);"));
        assert!(css.contains("position: absolute;"));
    }

    #[test]
    fn stylesheet_links_data_state_checked_to_item_style() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="radio-card"][data-part="item"][data-state="checked"]"#)
        );
        assert!(css.contains("border-color: var(--fandhe-palette, var(--fandhe-color-accent));"));
    }

    #[test]
    fn stylesheet_links_data_state_checked_to_item_indicator_style() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="radio-card"][data-part="item-indicator"][data-state="checked"]"#
        ));
    }

    #[test]
    fn root_switches_to_row_layout_on_horizontal_orientation() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="radio-card"][data-part="root"][data-orientation="horizontal"]"#
        ));
        assert!(css.contains("flex-direction: row;"));
    }

    #[test]
    fn disabled_item_gets_not_allowed_cursor() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="radio-card"][data-part="item"][data-disabled]"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn item_focus_within_gets_accent_outline_ring() {
        let css = stylesheet();
        let start = css
            .find(r#"[data-scope="radio-card"][data-part="item"]:focus-within {"#)
            .expect("item focus-within block must exist");
        let block = &css[start..];
        assert!(block.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));"
        ));
        assert!(block.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
    }

    #[test]
    fn stylesheet_links_item_to_data_invalid_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="radio-card"][data-part="item"][data-invalid] {"#));
        assert!(css.contains("border-color: var(--fandhe-color-danger);"));
    }

    #[test]
    fn stylesheet_registers_item_hover_inside_hover_media_query() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover) {"));
        assert!(css.contains(
            r#"[data-scope="radio-card"][data-part="item"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn item_transition_uses_motion_tokens() {
        let css = stylesheet();
        let start = css
            .find(r#"[data-scope="radio-card"][data-part="item"] {"#)
            .expect("item base block must exist");
        let end = css[start..].find('}').expect("item base block must close");
        let block = &css[start..start + end];
        assert!(block.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
        assert!(!block.contains("transition: "));
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
        assert!(html.contains(r#"data-scope="radio-card""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="radiogroup""#));
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
        assert!(html.contains("fd-radio-card--size-md"));
        assert!(html.contains("fd-radio-card--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-radio-card--size-xs"),
            (Size::Sm, "fd-radio-card--size-sm"),
            (Size::Md, "fd-radio-card--size-md"),
            (Size::Lg, "fd-radio-card--size-lg"),
            (Size::Xl, "fd-radio-card--size-xl"),
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
            (ColorPalette::Accent, "fd-radio-card--color-palette-accent"),
            (ColorPalette::Info, "fd-radio-card--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-radio-card--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-radio-card--color-palette-warning",
            ),
            (ColorPalette::Danger, "fd-radio-card--color-palette-danger"),
            (
                ColorPalette::Neutral,
                "fd-radio-card--color-palette-neutral",
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
        assert!(html.contains(r#"data-scope="radio-card""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn root_drops_caller_supplied_reserved_attrs() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![
                ("role", "attacker"),
                ("data-disabled", "attacker"),
                ("aria-labelledby", "attacker"),
            ],
            vec![],
        ));
        assert!(html.contains(r#"role="radiogroup""#));
        assert!(!html.contains("attacker"));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("aria-labelledby"));
    }

    #[test]
    fn item_drops_caller_supplied_reserved_attrs() {
        let html = render(&item(
            false,
            false,
            "red",
            vec![("data-state", "checked"), ("data-value", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert!(html.contains(r#"data-value="red""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn item_hidden_input_drops_caller_supplied_reserved_attrs() {
        // レビュー指摘（イシュー #747）の具体例: unchecked にもかかわらず
        // 呼び出し側 attrs が `checked` を混入させても、フレームワーク側の
        // 固定値（未 checked のため checked 属性なし）が優先され漏れ出ない。
        let node = item_hidden_input(
            false,
            false,
            Some("color"),
            "red",
            vec![
                ("type", "text"),
                ("name", "attacker"),
                ("value", "attacker"),
                ("checked", ""),
            ],
        );
        let html = render(&node);
        assert!(html.contains(r#"type="radio""#));
        assert!(html.contains(r#"name="color""#));
        assert!(html.contains(r#"value="red""#));
        assert!(!html.contains("checked="));
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
        let html = render(&item_text(vec![], vec![text(payload)]));
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn item_hidden_input_name_and_value_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&item_hidden_input(
            false,
            false,
            Some(PAYLOAD),
            PAYLOAD,
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    // --- SSR/hydration 往復（受け入れ条件 1: headless RadioGroup 状態機械を再利用） ---

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_radio_group_state_machine() {
        let mut g = RadioGroup::default();
        assert_eq!(g.value(), None);

        assert!(dispatch(&mut g, "select", "red"));
        assert_eq!(g.value(), Some("red"));

        let ssr_html = render(&item_control(g.is_checked("red"), false, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="checked""#));

        let hydrate_html = render(&render_for_hydration(&g));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = RadioGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored.value(), Some("red"));
    }

    // --- イシュー #1492: 内部レイアウト slot・size 軸 ---

    /// `--fandhe-radio-card-gap`（`item-control` の `gap` が参照）が xs〜xl
    /// で spacing トークン経由の単調増加になることを固定する
    /// （checkbox-card `size_variants_set_gap_custom_property_monotonically`
    /// #1458 と同型）。
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
                r#"[data-scope="radio-card"][data-part="root"].fd-radio-card--size-{}"#,
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
            let expected_decl = format!("--fandhe-radio-card-gap: {gap};");
            assert!(
                block.contains(&expected_decl),
                "size={size:?} variant block missing {expected_decl}: {block}"
            );
        }
    }

    /// `--fandhe-radio-card-padding` が生 rem リテラルではなく spacing
    /// トークンで xs〜xl 定義されることを固定する（checkbox-card
    /// `size_variants_padding_uses_spacing_tokens` #1458 と同型）。
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
                r#"[data-scope="radio-card"][data-part="root"].fd-radio-card--size-{}"#,
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
            let expected_decl = format!("--fandhe-radio-card-padding: {padding};");
            assert!(
                block.contains(&expected_decl),
                "size={size:?} variant block missing {expected_decl}: {block}"
            );
            assert!(
                !block.contains("--fandhe-radio-card-padding: 0.")
                    && !block.contains("--fandhe-radio-card-padding: 1."),
                "size={size:?} variant block still uses a raw rem literal for padding: {block}"
            );
        }
    }

    /// control 寸法（`--fandhe-radio-card-control-size`）が xs〜xl で単調
    /// 増加することを rem 値の parse で固定する（checkbox 家族と同一値へ
    /// そろえる意図的判断、モジュール rustdoc 参照。checkbox-card
    /// `size_variants_control_size_is_monotonic` #1458 と同型）。
    #[test]
    fn size_variants_control_size_is_monotonic() {
        let css = stylesheet();
        let mut sizes_rem = Vec::new();
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl] {
            let selector = format!(
                r#"[data-scope="radio-card"][data-part="root"].fd-radio-card--size-{}"#,
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
                .find("--fandhe-radio-card-control-size: ")
                .unwrap_or_else(|| panic!("control-size declaration not found in {block}"));
            let after = &block[decl_start + "--fandhe-radio-card-control-size: ".len()..];
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

    /// `--fandhe-radio-card-description-font-size` が xs〜xl すべてで定義
    /// され、`item-description` base がそれを参照することを固定する
    /// （checkbox-card `size_variants_set_description_font_size_custom_property`
    /// #1458 と同型）。
    #[test]
    fn size_variants_set_description_font_size_custom_property() {
        let css = stylesheet();
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl] {
            let selector = format!(
                r#"[data-scope="radio-card"][data-part="root"].fd-radio-card--size-{}"#,
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
                css[start..block_end].contains("--fandhe-radio-card-description-font-size"),
                "size={size:?} variant block missing --fandhe-radio-card-description-font-size: {}",
                &css[start..block_end]
            );
        }
        let description_selector = r#"[data-scope="radio-card"][data-part="item-description"] {"#;
        let start = css
            .find(description_selector)
            .expect("item-description base block must exist");
        let block_end = css[start..]
            .find('}')
            .map(|i| start + i)
            .unwrap_or(css.len());
        assert!(
            css[start..block_end].contains(
                "font-size: var(--fandhe-radio-card-description-font-size, var(--fandhe-font-font-size-sm));"
            ),
            "item-description base block missing size-linked font-size: {}",
            &css[start..block_end]
        );
    }

    /// `item-text` が checkbox-card `label` #1458 と同型の型階層（medium
    /// font-weight・行送り・誤選択防止）を持つことを固定する。
    #[test]
    fn item_text_has_typography_hierarchy_declarations() {
        let css = stylesheet();
        let selector = r#"[data-scope="radio-card"][data-part="item-text"] {"#;
        let start = css
            .find(selector)
            .unwrap_or_else(|| panic!("item-text base selector not found in {css}"));
        let block_end = css[start..]
            .find('}')
            .map(|i| start + i)
            .unwrap_or(css.len());
        let block = &css[start..block_end];
        assert!(
            block.contains("font-weight: var(--fandhe-font-font-weight-medium);"),
            "item-text block missing font-weight: {block}"
        );
        assert!(
            block.contains("line-height: var(--fandhe-font-line-height-normal);"),
            "item-text block missing line-height: {block}"
        );
        assert!(
            block.contains("color: var(--fandhe-color-fg);"),
            "item-text block missing color: {block}"
        );
        assert!(
            block.contains("user-select: none;"),
            "item-text block missing user-select: {block}"
        );
    }

    /// `label`（グループ見出し）が radio_group `label` #1495 と同型に
    /// size 連動 font-size・medium font-weight・行送りを持つことを固定
    /// する。
    #[test]
    fn label_has_typography_hierarchy_declarations() {
        let css = stylesheet();
        let selector = r#"[data-scope="radio-card"][data-part="label"] {"#;
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
                "font-size: var(--fandhe-radio-card-label-font-size, var(--fandhe-font-font-size-sm));"
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
    }

    /// `item-indicator` の transition が motion トークン経由になることを
    /// 固定する（checkbox-card `indicator_transition_uses_motion_tokens`
    /// #1425/#1458 と同型）。
    #[test]
    fn item_indicator_transition_uses_motion_tokens() {
        let css = stylesheet();
        assert!(
            css.contains("transition-duration: var(--fandhe-motion-duration-fast);"),
            "stylesheet missing motion-token transition-duration for item-indicator: {css}"
        );
    }

    /// `item-indicator` が `box-sizing: border-box` を持つことを固定する
    /// （checkbox `control` #1454 / radio_group `item-control` #1494 と
    /// 寸法解釈を統一する意図的判断）。
    #[test]
    fn item_indicator_has_box_sizing_border_box() {
        let css = stylesheet();
        let selector = r#"[data-scope="radio-card"][data-part="item-indicator"] {"#;
        let start = css
            .find(selector)
            .unwrap_or_else(|| panic!("item-indicator base selector not found in {css}"));
        let block_end = css[start..]
            .find('}')
            .map(|i| start + i)
            .unwrap_or(css.len());
        assert!(
            css[start..block_end].contains("box-sizing: border-box;"),
            "item-indicator base block missing box-sizing: {}",
            &css[start..block_end]
        );
    }

    /// `item-indicator` が `data-invalid` へ checkbox-card `indicator`
    /// #1458 と同型の枠線色変化を反映することを固定する（`item` 側は
    /// 1/2〔PR #1768〕の担当）。
    #[test]
    fn stylesheet_links_item_indicator_to_data_invalid_state() {
        let css = stylesheet();
        assert!(css
            .contains(r#"[data-scope="radio-card"][data-part="item-indicator"][data-invalid] {"#));
    }
}

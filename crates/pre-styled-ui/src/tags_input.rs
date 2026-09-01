//! styled TagsInput（headless ラッパー、イシュー #744、親 #736/#520/#546）。
//!
//! `fandhe_frontend_headless_ui::tags_input`（イシュー #744）の
//! Label / Control / Input / Item / ItemPreview / ItemText / ItemInput /
//! ItemDeleteTrigger / ClearTrigger / HiddenInput 10 anatomy パーツをそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠は [`crate::pin_input`]/[`crate::number_input`] の rustdoc と同じ方針
//! に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`TagsInput` 型・
//! headless `root` を再エクスポートしない理由）
//!
//! 本モジュールは `size` variant クラス付与のため styled `root`
//! （[`crate::pin_input::root`] と同型）を本モジュールで再定義する。headless
//! 自由関数 `root` と名前衝突するため、`pub use ...::*` ではなく必要な識別子
//! （[`label`]/[`control`]/[`input`]/[`item`]/[`item_preview`]/[`item_text`]/
//! [`item_input`]/[`item_delete_trigger`]/[`clear_trigger`]/[`hidden_input`]/
//! [`TagsInputAction`]）のみを選択的に再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::tags_input::TagsInput`] は**あえて**
//! 再エクスポートしない（[`crate::pin_input`]/[`crate::number_input`] の非再
//! エクスポートと同じ理由、PR #695 Bugbot 指摘の前例）。状態管理・hydration
//! が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::tags_input::TagsInput` を直接 import し、
//! 実際の描画は本モジュールの styled [`root`]（および再エクスポート済みの
//! パーツ関数）を組み合わせて構築すること。
//!
//! # `hidden-input` に CSS を付与しない理由
//!
//! [`crate::pin_input`] の `hidden-input` と同じ理由（`<input type="hidden">`
//! は UA 既定挙動として常にレンダリングされない）で、[`recipe`] は
//! `hidden-input` slot へ一切の CSS を登録しない
//! （`hidden_input_slot_has_no_css_rules` テストで固定）。
//!
//! # `size` variant
//!
//! [`crate::switch`] rustdoc「複合部品の variant 統一方針」節（#708）に従い、
//! `size`（[`Size`]）は styled `root` へのみクラスを付与し、[`recipe`] が
//! 登録する `--fandhe-tags-input-*` の root スコープ CSS custom property
//! （通常の CSS 継承）経由で `item-preview`/`input` の寸法・書体を切り替える。
//! `base`/`variant` 規則の `var()` にはいずれも Md サイズ相当のフォールバック
//! 値を書き、styled `root` を経由しない headless 直接利用マークアップでも
//! 現行外観を維持する（fail-safe）。フォーム入力部品のため palette
//! （color-palette 軸）は本イシューでは提供しない（[`crate::number_input`]
//! の先例に倣う）。
//!
//! # フォーカスリング（外枠 `control` に `:focus-within` として出す構成、
//! イシュー #1698）
//!
//! 実フォーカスは `input`（`<input type="text">` 自身）が受けるネイティブ
//! 要素だが、リングは `input` 自体ではなく外枠 `control` へ
//! `StateCondition::FocusWithin`（`:focus-within` 疑似クラス）+
//! [`crate::recipe::focus_ring_declarations`]（`FocusRingColor::Token`,
//! `FocusRingOffset::Outside`）として出す（[`crate::combobox`]
//! （イシュー #1467）と同型の chakra 的表現: `control` 側にタグチップ群と
//! `input` を並べた「1 つの入力欄」として枠取りしているため、`input` へ
//! 直接輪郭を描くより外枠 `control` へリングを出すほうが視覚的に自然）。
//! `input` の base `outline: none` は維持し、`input` 自身は `:focus-visible`
//! 用の state を持たない。**fail-safe の注記**: styled `control` を経由せず
//! headless `input` を直接利用するマークアップ（`control` を介さない構成）
//! ではリングが表示されなくなる副作用があるが、combobox #1467 が既に同じ
//! トレードオフを確定させている。
//!
//! # 外枠パート（root/control/input）のスタイル調整（イシュー #1698、親
//! #1510）
//!
//! 兄弟イシュー #1699（内部パート・状態遷移）とスコープを分割した前半分。
//! 本節に列挙する変更のみを適用し、item 系・clear-trigger・label は一切
//! 変更していない。
//!
//! - **`root` の disabled を canonical 化**: `[data-disabled]` を生の
//!   `vec![cursor, opacity]` から [`crate::recipe::disabled_declarations`]
//!   （宣言順 `opacity` → `cursor`）へ置換した（Phase 0 統一形、イシュー
//!   #1425）。視覚は不変（宣言順のみ変わる golden 更新を伴う）。
//! - **`control` の角丸を Forms 家族標準へ**: `var(--fandhe-radius-sm)` →
//!   `var(--fandhe-radius-md)`（イシュー #1482、[`crate::input`]/
//!   [`crate::date_input`] と同じ角丸）。
//! - **`control` に transition を追加**: 上記フォーカスリング節の
//!   `:focus-within` 遷移・`data-invalid` の枠色変化を滑らかにするため
//!   `transition_declarations("border-color, background",
//!   MotionDuration::Fast)` を base へ純追加した（combobox #1467 と同型）。
//! - **`control` の `[data-disabled]`**: 上記フォーカスリング節の直後の
//!   コード参照。
//! - **`control` `[data-invalid]`**: 既存の
//!   `border-color: var(--fandhe-color-danger)` はトークン準拠済みのため
//!   変更なし（点検結果として記録）。
//! - **hover は意図的に非採用のまま維持**: [`crate::input`] rustdoc が
//!   明文化する方針（テキストフィールドは hover 背景変化を持たないのが
//!   chakra / Radix Themes 標準。hover はインタラクティブ slot =
//!   `cursor: pointer` を持つ slot のみ、イシュー #1425）に従い、`control`
//!   へ hover state を追加しない。
//! - **variant 軸（面バリアント）は対象外**: `root` シグネチャ変更（破壊的
//!   変更）を伴うため見送り（checkbox_card / file-upload #1696 と同じ判断）。
//! - **`label` はスコープ外**: #1698 の対象列挙（root/control/input）に
//!   含まれないため変更しない。
//!
//! # 内部パート（item 等）のスタイル調整と状態遷移（イシュー #1699、親
//! #1510）
//!
//! #1698（外枠パート）の後半分。以下の内部パート・状態のみを変更し、
//! `clear-trigger`・`label`・外枠パート（root/control/input）は一切
//! 変更していない。
//!
//! - **`item`（新規）**: `display: inline-flex` を base として追加した
//!   （`item-preview`/`item-input` を包む薄いコンテナとしての役割を明示。
//!   従来は `<div>` の UA 既定 `display: block` に依存していた）。
//!   `[data-disabled]`（headless `item` が自身の `disabled` 引数から出す）
//!   には `cursor: not-allowed` のみを付与し、`disabled_declarations()`
//!   （opacity 0.5 併載）は使わない。`item` は常に `root` 配下で使われ
//!   `root` 側の `data-disabled` state が既に `opacity: 0.5` を適用済みの
//!   ため、二重に重ねると 0.5 × 0.5 = 0.25 の過剰な減衰になる（`control`・
//!   `input`・`item-delete-trigger`・`clear-trigger` の既存判断と同じ、
//!   #1698 rustdoc 参照）。`[data-editing]` には CSS を付けない（編集中の
//!   視覚は `item-input` 自身が担うため、`item` コンテナ側で追加の見た目
//!   変化を持たせる必要がない）。
//! - **`item-preview` に transition を追加**: `data-highlighted` の
//!   accent 反転（既存）が瞬時切り替えだったのを
//!   `transition_declarations("background, color", MotionDuration::Fast)`
//!   の base 純追加で滑らかにした（#1425 統一形。duration は
//!   `var(--fandhe-motion-duration-fast)` 参照のため
//!   `prefers-reduced-motion: reduce` 環境では `Theme::to_css` 側の一括
//!   上書きにより自動的に 0ms へ倒れる）。
//! - **`item-text`**: 点検の結果、既存の `white-space: nowrap` のみで
//!   参考サイトと乖離なし。変更なし。
//! - **`item-input`（新規）**: 編集中タグのネイティブ `<input>` へ UA
//!   既定の枠・背景をリセットし、兄弟 `input`（外枠側の新規タグ入力欄）と
//!   書体を統一する（`box-sizing: border-box`・`border: none`・
//!   `outline: none`・`background: transparent`・`padding: 0`・
//!   `font-size`/`color` は `input` slot と同じトークン参照）。
//!   `outline: none` は #1698 で確立したフォーカスリング方針（リングは
//!   外枠 `control` の `:focus-within` に出す）と整合する。`item-input` も
//!   `control` 配下の要素であるため、`item-input` がフォーカスを受けても
//!   `control` の `:focus-within` によりリングは変わらず表示される。
//! - **`item-delete-trigger` に hover/transition を追加**:
//!   `cursor: pointer` を持つインタラクティブ slot にもかかわらず
//!   hover/transition を欠いていた（#1425 規約違反状態）ため是正した。
//!   base へ [`hover_bg_muted`]（`--fandhe-hover-bg` 定義）+
//!   `transition_declarations("background, color", MotionDuration::Fast)`
//!   を純追加し、`.state("item-delete-trigger", StateCondition::Hover,
//!   hover_surface_declarations())` を新設した（recipe 層が
//!   `@media (hover: hover)` + `:hover:not([data-disabled])` として出力、
//!   #1425 §5 手順どおり）。既存の `[data-disabled]`（cursor のみ）は
//!   維持する。
//! - **`data-selected` は意図的 N/A**: headless `item_preview` は
//!   「既に確定したタグ」を表し `aria-selected="true"` を常時固定で出力し、
//!   `data-selected` 属性は出力しない（選択が可変でない意味論、
//!   `crates/headless-ui/src/tags_input.rs::item_preview` rustdoc 参照）。
//!   死に CSS（存在しない属性へのセレクタ）を追加せず本節に理由のみ記録
//!   する。将来 headless 側の意味論が変わる（`data-selected` の出力が
//!   追加される）場合はこの判断を再評価する。
//! - **`clear-trigger` はスコープ外**: #1699 の対象列挙（item 系）に
//!   含まれないため変更しない。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

// `TagsInput` 状態機械・headless 自由関数 `root` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::tags_input::TagsInput` を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::tags_input::{
    clear_trigger, control, hidden_input, input, item, item_delete_trigger, item_input,
    item_preview, item_text, label, TagsInputAction,
};

/// headless `tags_input` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/tags_input.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を出力しない
/// fail-closed 側の不具合として現れるため、変更時は両ファイルを合わせて
/// 確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "input",
    "item",
    "item-preview",
    "item-text",
    "item-input",
    "item-delete-trigger",
    "clear-trigger",
    "hidden-input",
];

/// この styled TagsInput の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("tags-input", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .base(
            "label",
            vec![decl("font-size", "var(--fandhe-font-font-size-sm)")],
        )
        .base(
            "control",
            vec![
                decl("display", "flex"),
                decl("flex-wrap", "wrap"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("box-sizing", "border-box"),
                decl("padding", "var(--fandhe-space-2)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結されるため、
        // 上記 base ブロックを書き換えずに純追加する（combobox #1467 と
        // 同型のパターン、モジュール rustdoc「外枠パートのスタイル調整」
        // 節参照）。
        .base(
            "control",
            transition_declarations("border-color, background", MotionDuration::Fast),
        )
        .state(
            "control",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        // 実フォーカスは `input`（`<input type="text">`）自身が受けるが、
        // リングは外枠 `control` へ `:focus-within` として出す（combobox
        // #1467 と同型、モジュール rustdoc「フォーカスリング」節参照）。
        .state(
            "control",
            StateCondition::FocusWithin,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // headless `control` は自身の `disabled` 引数から `data-disabled` を
        // 出す（`crates/headless-ui/src/tags_input.rs::control`）。
        // `disabled_declarations()`（opacity 0.5 + cursor）ではなく
        // `cursor: not-allowed` のみに留めるのは、`control` は常に `root`
        // 配下で使われ `root` 側の `data-disabled` state が既に
        // `opacity: 0.5` を適用済みのため（重ねると 0.5 × 0.5 = 0.25 の
        // 二重 opacity になる。file-upload trigger（イシュー #1696）・
        // 本ファイル `input`/`item-delete-trigger`/`clear-trigger` の既存
        // `data-disabled` state と同じ判断）。
        .state(
            "control",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        // `item` slot（新規、モジュール rustdoc「内部パートのスタイル
        // 調整」節参照）: `item-preview`/`item-input` を包む薄い
        // コンテナとして `display: inline-flex` を明示する。
        .base("item", vec![decl("display", "inline-flex")])
        // headless `item` は自身の `disabled` 引数から `data-disabled` を
        // 出す。`item` は常に `root` 配下で使われ `root` 側の
        // `data-disabled` state が既に `opacity: 0.5` を適用済みのため、
        // `disabled_declarations()` は使わず `cursor: not-allowed` のみに
        // 留める（`control`/`input`/`item-delete-trigger`/`clear-trigger`
        // の既存判断と同じ、モジュール rustdoc「内部パートのスタイル
        // 調整」節参照）。
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        // `data-editing` には CSS を付けない（編集中の視覚は `item-input`
        // 自身が担う、モジュール rustdoc 参照）。
        .base(
            "item-preview",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl(
                    "font-size",
                    "var(--fandhe-tags-input-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl(
                    "padding",
                    "var(--fandhe-tags-input-chip-padding-y, 0.125rem) var(--fandhe-tags-input-chip-padding-x, 0.5rem)",
                ),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結されるため、
        // 上記 base ブロックを書き換えずに純追加する（#1698 と同型の
        // パターン）。`data-highlighted` の accent 反転（下記 state）を
        // 滑らかにする。
        .base(
            "item-preview",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .state(
            "item-preview",
            StateCondition::Attr("data-highlighted"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        .base("item-text", vec![decl("white-space", "nowrap")])
        .base(
            "item-delete-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("width", "1rem"),
                decl("height", "1rem"),
                decl("padding", "0"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "transparent"),
                decl("color", "inherit"),
                decl("cursor", "pointer"),
                decl("line-height", "1"),
            ],
        )
        // `cursor: pointer` を持つインタラクティブ slot のため hover/
        // transition を付与する（#1425 規約、モジュール rustdoc「内部
        // パートのスタイル調整」節参照）。
        .base(
            "item-delete-trigger",
            vec![hover_bg_muted()],
        )
        .base(
            "item-delete-trigger",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .state(
            "item-delete-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "item-delete-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .base(
            "item-input",
            vec![
                decl("box-sizing", "border-box"),
                decl("border", "none"),
                decl("outline", "none"),
                decl("background", "transparent"),
                decl("padding", "0"),
                decl(
                    "font-size",
                    "var(--fandhe-tags-input-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .base(
            "input",
            vec![
                decl("flex", "1 1 auto"),
                decl("min-width", "var(--fandhe-tags-input-input-min-width, 6rem)"),
                decl("box-sizing", "border-box"),
                decl("border", "none"),
                decl("outline", "none"),
                decl("background", "transparent"),
                decl(
                    "font-size",
                    "var(--fandhe-tags-input-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .state(
            "input",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .base(
            "clear-trigger",
            vec![
                decl("align-self", "flex-start"),
                decl("border", "none"),
                decl("background", "transparent"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("cursor", "pointer"),
                decl(
                    "font-size",
                    "var(--fandhe-tags-input-font-size, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        .state(
            "clear-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-tags-input-font-size", "var(--fandhe-font-font-size-xs)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl(
                "--fandhe-tags-input-font-size",
                "var(--fandhe-font-font-size-xs)",
            )],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl(
                "--fandhe-tags-input-font-size",
                "var(--fandhe-font-font-size-sm)",
            )],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl(
                "--fandhe-tags-input-font-size",
                "var(--fandhe-font-font-size-md)",
            )],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-tags-input-font-size", "var(--fandhe-font-font-size-lg)"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled TagsInput が生成する静的 CSS 全量を返す（決定的。
/// [`crate::pin_input::stylesheet`]/[`crate::number_input::stylesheet`] と
/// 同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::tags_input::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::tags_input;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = tags_input::root(Size::Md, false, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="tags-input" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::tags_input::root(disabled, merged, children)
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
        assert!(a.contains(r#"[data-scope="tags-input"][data-part="control"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_control_to_invalid_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tags-input"][data-part="control"][data-invalid] {"#));
    }

    #[test]
    fn stylesheet_links_item_preview_to_highlighted_state() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="tags-input"][data-part="item-preview"][data-highlighted] {"#
        ));
    }

    #[test]
    fn stylesheet_links_item_to_disabled_state() {
        // イシュー #1699: `item` の `[data-disabled]` は `cursor:
        // not-allowed` のみ（`root` 側の opacity 二重減衰回避、モジュール
        // rustdoc「内部パートのスタイル調整」節参照）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tags-input"][data-part="item"][data-disabled] {"#));
        assert!(css.contains(
            r#"[data-scope="tags-input"][data-part="item"][data-disabled] {
  cursor: not-allowed;
}"#
        ));
    }

    #[test]
    fn stylesheet_styles_item_input_base() {
        // イシュー #1699: 編集中タグの `item-input` は UA 既定をリセットし
        // `input`（外枠の新規タグ入力欄）と書体を統一する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tags-input"][data-part="item-input"] {"#));
        assert!(css.contains(
            r#"[data-scope="tags-input"][data-part="item-input"] {
  box-sizing: border-box;
  border: none;
  outline: none;
  background: transparent;
  padding: 0;"#
        ));
    }

    #[test]
    fn stylesheet_links_item_preview_to_highlighted_transition() {
        // イシュー #1699: `data-highlighted` の accent 反転を滑らかにする
        // transition（`item-preview` base への純追加）。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="tags-input"][data-part="item-preview"] {
  display: inline-flex"#
        ));
        assert!(css.contains("transition-property: background, color;"));
        assert!(css.contains("var(--fandhe-motion-duration-fast)"));
    }

    #[test]
    fn stylesheet_links_item_delete_trigger_to_hover_under_hover_media() {
        // イシュー #1699: `cursor: pointer` を持つインタラクティブ slot
        // への hover 付与（#1425 規約是正）。recipe 層は hover state を
        // `@media (hover: hover)` 配下の `:hover:not([data-disabled])` へ
        // 集約出力する。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover) {"));
        assert!(css.contains(
            r#"  [data-scope="tags-input"][data-part="item-delete-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }"#
        ));
        assert!(css.contains("--fandhe-hover-bg: var(--fandhe-color-bg-muted);"));
    }

    #[test]
    fn stylesheet_links_control_to_focus_within_ring() {
        // フォーカスリングは `input` ではなく外枠 `control` の
        // `:focus-within` へ出す（モジュール rustdoc「フォーカスリング」節、
        // combobox #1467 と同型）。canonical なトークン参照形
        // （`--fandhe-focus-ring-width`/`--fandhe-color-focus-ring`）を持つ。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="tags-input"][data-part="control"]:focus-within {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}"#
        ));
        // `input` 自身は `:focus-visible` state を持たない（リング移設に伴う
        // 削除の確認）。
        assert!(!css.contains(r#"[data-part="input"]:focus-visible"#));
    }

    #[test]
    fn stylesheet_links_control_to_disabled_state_and_transition() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tags-input"][data-part="control"][data-disabled] {"#));
        assert!(css.contains(
            r#"[data-scope="tags-input"][data-part="control"] {
  display: flex"#
        ));
        assert!(css.contains("transition-property: border-color, background;"));
    }

    #[test]
    fn stylesheet_links_root_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="tags-input"][data-part="root"][data-disabled] {"#));
        assert!(css.contains("cursor: not-allowed;"));
        assert!(css.contains("opacity: 0.5;"));
    }

    #[test]
    fn hidden_input_slot_has_no_css_rules() {
        // モジュール rustdoc「`hidden-input` に CSS を付与しない理由」参照。
        let css = stylesheet();
        assert!(!css.contains(r#"[data-part="hidden-input"]"#));
    }

    // --- variant クラス ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(Size::Md, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="tags-input""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md() {
        let html = render(&root(Size::Md, false, vec![], vec![]));
        assert!(html.contains("fd-tags-input--size-md"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-tags-input--size-xs"),
            (Size::Sm, "fd-tags-input--size-sm"),
            (Size::Md, "fd-tags-input--size-md"),
            (Size::Lg, "fd-tags-input--size-lg"),
            (Size::Xl, "fd-tags-input--size-xl"),
        ] {
            let html = render(&root(size, false, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            false,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_contains_size_variant_selectors_and_custom_properties() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--fandhe-tags-input-font-size"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="tags-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn root_reflects_disabled_prop() {
        let html = render(&root(Size::Md, true, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            Size::Md,
            false,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_label_children_are_escaped_on_render() {
        let html = render(&label(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_item_text_tag_payload_is_escaped_on_render() {
        let html = render(&item_text(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_hidden_input_name_value_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&hidden_input(PAYLOAD, PAYLOAD, false, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_item_delete_trigger_aria_label_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&item_delete_trigger(PAYLOAD, false, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_tags_input_state_machine() {
        // `TagsInput` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「選択的 re-export」節参照）ため、headless-ui から
        // 直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_headless_ui::tags_input::TagsInput;
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut t = TagsInput::new(Vec::new(), Some(5));
        assert!(t.is_empty());

        let ssr_html = render(&t.root(false, vec![], vec![]));
        assert!(!ssr_html.contains("data-hydrate-"));

        assert!(dispatch(&mut t, "add", "rust"));
        let hydrate_html = render(&render_for_hydration(&t));
        assert!(hydrate_html.contains(r#"data-hydrate-max="5""#));

        let restored = TagsInput::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored.value(), t.value());
    }
}

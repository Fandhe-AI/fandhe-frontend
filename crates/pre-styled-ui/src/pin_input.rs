//! styled PinInput（headless ラッパー、イシュー #739、親 #736/#520/#546）。
//!
//! `fandhe_frontend_headless_ui::pin_input`（イシュー #739）の
//! Label / Control / Input / HiddenInput 4 anatomy パーツをそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠は [`crate::checkbox`]/[`crate::switch`] の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`PinInput` 型・
//! headless `root` を再エクスポートしない理由）
//!
//! 本モジュールは `size` variant クラス付与のため styled `root`
//! （[`crate::checkbox::root`]/[`crate::switch::root`] と同型）を本モジュール
//! で再定義する。headless 自由関数 `root` と名前衝突するため、
//! `pub use ...::*` ではなく必要な識別子（[`label`]/[`control`]/[`input`]/
//! [`hidden_input`]/[`PinInputAction`]/[`PinInputKind`]）のみを選択的に
//! 再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::pin_input::PinInput`] は**あえて**
//! 再エクスポートしない（[`crate::switch`]/[`crate::checkbox`] の非再
//! エクスポートと同じ理由、PR #695 Bugbot 指摘の前例）。`PinInput` は
//! `.root(disabled, attrs, children)` 等の inherent メソッドを持つが、これは
//! headless 自由関数へそのまま委譲するのみで `size` variant クラスを一切
//! 付与しない未スタイルの実体である。本モジュールが `PinInput` を丸ごと
//! 再エクスポートすると、呼び出し側が（styled 層のつもりで）
//! `pin_input_instance.root(...)` を呼んでしまい、`size` が付与されず
//! 見た目が静かに崩れる事故を誘発する。`PinInput` による状態管理・
//! hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::pin_input::PinInput` を直接 import し、
//! 実際の描画は本モジュールの styled [`root`]（および再エクスポート済みの
//! パーツ関数）を組み合わせて構築すること。
//!
//! # `hidden-input` に CSS を付与しない理由
//!
//! [`crate::checkbox`]/[`crate::switch`] の `hidden-input` は
//! `<input type="checkbox">` という「見た目上は目に見えるはずの」要素を
//! visually-hidden パターン（`position: absolute` + 1px クリップ）で隠す
//! 必要があった。本モジュールの [`hidden_input`] は
//! `<input type="hidden">` であり、ブラウザの UA 既定挙動として
//! 常にレンダリングされない（`display`/`visibility` を問わず描画対象に
//! ならない）。したがって visually-hidden パターンを適用する必要がなく、
//! [`recipe`] は `hidden-input` slot へ一切の CSS を登録しない
//! （`hidden_input_slot_has_no_css_rules` テストで固定）。
//!
//! # `size` variant
//!
//! [`crate::switch`] rustdoc「複合部品の variant 統一方針」節（#708）に従い、
//! `size`（[`Size`]）は styled `root` へのみクラスを付与し、[`recipe`] が
//! 登録する `--fandhe-pin-input-size`/`-font-size` の root スコープ CSS
//! custom property（通常の CSS 継承）経由で `input` の寸法・書体を
//! 切り替える。`base`/`variant` 規則の `var()` にはいずれも Md サイズ
//! 相当のフォールバック値を書き、styled `root` を経由しない headless
//! 直接利用マークアップでも現行外観を維持する（fail-safe）。palette
//! variant は本イシューのスコープ外（第 2 弾で他部品と合わせて展開する
//! 既存方針、`docs/api/pre-styled-ui-api.md` 参照）。
//!
//! # フォーカスリング（実フォーカスを `input` 自身が受ける構成）
//!
//! [`crate::checkbox`]/[`crate::switch`] の `control` はネイティブ入力
//! （hidden-input）から分離した装飾パーツのため `data-focus-visible`
//! 存在属性方式を使うが、本モジュールの `input` は `<input type="text">`/
//! `<input type="password">` 自身が実フォーカスを受けるネイティブ要素で
//! あるため、[`crate::accordion`]/[`crate::tabs`] と同様に
//! `StateCondition::FocusVisible`（`:focus-visible` 疑似クラス）を直接
//! `input` slot へ登録する。実装は `outline`/`outline-offset` の canonical 形
//! （[`crate::recipe::focus_ring_declarations`]）を使う（イシュー #1489 で
//! リテラル直書きから移行、下記「スタイル調整」節参照）。`palette` 軸を
//! 持たない部品のため [`crate::recipe::FocusRingColor::Token`] を使う。
//! オフセットは密に並ぶセル間の視覚干渉がない独立セルのため `Outside`
//! （既定）を選ぶ（[`crate::input`] と同じ判断）。
//!
//! # スタイル調整（イシュー #1489、親 UI 部品スタイル調整ツリー #1420）
//!
//! chakra-ui（PinInput は Input のスタイルを継承する設計）/ Radix
//! Primitives（one-time-password-field）/ ark-ui と 7 軸で比較し是正した点・
//! 意図的に合わせなかった点を記録する。特に input #1482（PR #1761）の
//! 是正内容を最も直接の参照実装とした。
//!
//! - **是正**: `input` の角丸を `--fandhe-radius-sm` から `--fandhe-radius-md`
//!   （button #1447・date-input #1469・input #1482 が確立した Forms 家族の
//!   標準角丸）へ変更。`transition` の shorthand リテラルを
//!   [`crate::recipe::transition_declarations`]（longhand 3 宣言 + motion
//!   トークン）へ移行。`:focus-visible` を `focus_ring_declarations` の
//!   canonical 形へ移行。`root` の `[data-disabled]` を
//!   [`crate::recipe::disabled_declarations`] へ統一（宣言順が
//!   `opacity` → `cursor` に変わるが値は不変）。`input` へ
//!   [`crate::recipe::hover_bg_muted`] + `StateCondition::Hover` による
//!   hover 背景を追加（ark-ui 参照スクショで小型セル群に hover
//!   フィードバックがあり、構造的に最も近い date-input `segment`
//!   （#1469）の先例と同型と判断。input #1761 は hover 非採用だが、
//!   pin-input は独立した複数セルという date-input segment 側の構造に
//!   近いため踏襲する）。size 5 段の値を #1678 の
//!   `--fandhe-size-control-height-<段>`/`--fandhe-size-control-font-size-<段>`
//!   参照（フォールバック付き）へ揃える。
//! - **意図的に合わせなかった点**:
//!   - **variant 軸（chakra `outline`/`subtle`/`flushed` 相当）は追加
//!     しない**。date-input #1469・combobox #1467・checkbox #1454 と
//!     同一の判断軸（Forms 家族横断の軸語彙判断のため本イシュー単独では
//!     先行しない）。
//!   - **色・ダーク（トークン参照のみ）は元々参照サイト水準に達していた
//!     ため変更しない**。focus-ring 色は canonical 形移行によりダーク
//!     追従トークンへ自動的に載る。
//!
//! # headless 層の `data-invalid`/`data-readonly`/`data-required` 対応
//! （イシュー #1615 追記）
//!
//! headless 層（`crates/headless-ui/src/pin_input.rs`）が ark-ui/Radix 参照
//! 突合（イシュー #1615）により [`fandhe_frontend_headless_ui::pin_input::PinInputProps`]
//! （disabled/readonly/invalid/required）を新設し、`data-invalid`/
//! `data-readonly`（root/label/input）・`data-required`（label）・
//! `aria-invalid`/ネイティブ `readonly`（input）を出力するようになった。
//! 本モジュールの styled [`root`] 公開シグネチャは非破壊のまま
//! （`size`/`complete`/`disabled` のみ）維持し、内部で
//! `PinInputProps { disabled, ..Default::default() }` を組み立てて headless
//! `root` へ委譲する（#1876 checkbox-group の非破壊化パターンと同型）。
//! styled 層で invalid/readonly を受け取る API 拡張・対応する CSS
//! （`[data-invalid]`/`[data-readonly]` 選択子）の追加は本イシューの
//! スコープ外とする（下記「スコープ外」節）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - variant 軸（chakra `outline`/`subtle`/`flushed` 相当）の追加は
//!   上記「スタイル調整」節のとおり本イシューのスコープ外とする。
//! - styled [`root`] へ invalid/readonly を受け取る引数を追加し、対応する
//!   CSS（`[data-invalid]`/`[data-readonly]` 選択子）を実装することは
//!   イシュー #1615 のスコープ外とする（headless 層の対応のみが本イシュー
//!   の対象）。必要なら別 Issue として起票を提案する。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

// `PinInput` 状態機械・headless 自由関数 `root` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::pin_input::PinInput` を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::pin_input::{
    control, hidden_input, input, label, PinInputAction, PinInputKind, PinInputProps,
};

/// headless `pin_input` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/pin_input.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を出力しない
/// fail-closed 側の不具合として現れるため、変更時は両ファイルを合わせて
/// 確認する）。
const SLOTS: &[&str] = &["root", "label", "control", "input", "hidden-input"];

/// この styled PinInput の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("pin-input", SLOTS)
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
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "input",
            vec![
                decl("box-sizing", "border-box"),
                decl(
                    "width",
                    "var(--fandhe-pin-input-size, var(--fandhe-size-control-height-md, 2.5rem))",
                ),
                decl(
                    "height",
                    "var(--fandhe-pin-input-size, var(--fandhe-size-control-height-md, 2.5rem))",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-pin-input-font-size, var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-md)))",
                ),
                decl("text-align", "center"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                hover_bg_muted(),
            ],
        )
        // 別 `.base` 呼び出しでの純追加（date-input #1469 の「既存 base
        // ブロックを書き換えない」パターンを踏襲する）。
        .base(
            "input",
            transition_declarations("border-color, background", MotionDuration::Fast),
        )
        .state(
            "input",
            StateCondition::Attr("data-complete"),
            vec![decl("border-color", "var(--fandhe-color-accent)")],
        )
        // `opacity` は `root` のみに適用する（[`crate::switch`]/
        // [`crate::checkbox`] と同じ方針）。headless 側は `data-disabled` を
        // `root`/`input` の両方に付与するため、両パーツへ `opacity: 0.5` を
        // 重ねるとネストした opacity の掛け算で `input` が実質約 25% まで
        // 減光し `root`（50%）と不整合になる。`cursor: not-allowed` のみ
        // `input` にも適用し、減光は `root` の 1 箇所に一元化する
        // （PR #784 Cursor Bugbot 指摘、イシュー #739）。
        .state(
            "input",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .state("input", StateCondition::Hover, hover_surface_declarations())
        // 実フォーカスを `input` 自身が受けるため `:focus-visible` を直接
        // 登録する（モジュール rustdoc「フォーカスリング」節参照）。
        .state(
            "input",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl(
                    "--fandhe-pin-input-size",
                    "var(--fandhe-size-control-height-xs, 2rem)",
                ),
                decl(
                    "--fandhe-pin-input-font-size",
                    "var(--fandhe-size-control-font-size-xs, var(--fandhe-font-font-size-xs))",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-pin-input-size",
                    "var(--fandhe-size-control-height-sm, 2.25rem)",
                ),
                decl(
                    "--fandhe-pin-input-font-size",
                    "var(--fandhe-size-control-font-size-sm, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-pin-input-size",
                    "var(--fandhe-size-control-height-md, 2.5rem)",
                ),
                decl(
                    "--fandhe-pin-input-font-size",
                    "var(--fandhe-size-control-font-size-md, var(--fandhe-font-font-size-md))",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-pin-input-size",
                    "var(--fandhe-size-control-height-lg, 2.75rem)",
                ),
                decl(
                    "--fandhe-pin-input-font-size",
                    "var(--fandhe-size-control-font-size-lg, var(--fandhe-font-font-size-lg))",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl(
                    "--fandhe-pin-input-size",
                    "var(--fandhe-size-control-height-xl, 3rem)",
                ),
                decl(
                    "--fandhe-pin-input-font-size",
                    "var(--fandhe-size-control-font-size-xl, var(--fandhe-font-font-size-xl))",
                ),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled PinInput が生成する静的 CSS 全量を返す（決定的。
/// [`crate::switch::stylesheet`]/[`crate::checkbox::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::pin_input::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::pin_input;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = pin_input::root(Size::Md, false, false, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="pin-input" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    complete: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    // 公開シグネチャは `disabled: bool` のまま非破壊で維持し、内部で
    // headless の `PinInputProps` を組み立てて委譲する（イシュー #1615
    // 追記、モジュール rustdoc 参照）。
    let props = PinInputProps {
        disabled,
        ..Default::default()
    };
    fandhe_frontend_headless_ui::pin_input::root(complete, &props, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="pin-input"][data-part="input"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_input_to_complete_and_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="pin-input"][data-part="input"][data-complete] {"#));
        assert!(css.contains(r#"[data-scope="pin-input"][data-part="input"][data-disabled] {"#));
    }

    #[test]
    fn stylesheet_links_input_to_focus_visible_outline() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="pin-input"][data-part="input"]:focus-visible {"#));
        // イシュー #1489: canonical `outline` 形（`FocusRingColor::Token`・
        // `FocusRingOffset::Outside`）へ移行したことを固定する。
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
        assert!(css.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
    }

    #[test]
    fn stylesheet_links_root_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="pin-input"][data-part="root"][data-disabled] {"#));
        assert!(css.contains("cursor: not-allowed;"));
        assert!(css.contains("opacity: 0.5;"));
    }

    #[test]
    fn stylesheet_links_input_to_hover_background() {
        // イシュー #1489: date-input `segment`（#1469）と同型の hover
        // フィードバックを固定する。`hover_surface_declarations` +
        // `hover_bg_muted` は `@media (hover: hover)` +
        // `:hover:not([data-disabled])` で自動的にラップされる契約
        // （`crate::recipe::SlotRecipe::css` 参照）。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(
            r#"[data-scope="pin-input"][data-part="input"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains("--fandhe-hover-bg: var(--fandhe-color-bg-muted);"));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn stylesheet_input_transition_uses_longhand_declarations() {
        // イシュー #1489: shorthand `transition` リテラルを
        // `transition_declarations` 由来の longhand 3 宣言へ移行したことを
        // 固定する。
        let css = stylesheet();
        assert!(!css.contains("transition: border-color 0.15s, background 0.15s;"));
        assert!(css.contains("transition-property: border-color, background;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast"));
    }

    #[test]
    fn stylesheet_input_uses_radius_md() {
        // イシュー #1489: button #1447・date-input #1469・input #1482 が
        // 確立した Forms 家族の標準角丸へ揃えたことを固定する。
        let css = stylesheet();
        assert!(css.contains("border-radius: var(--fandhe-radius-md);"));
        assert!(!css.contains("border-radius: var(--fandhe-radius-sm);"));
    }

    #[test]
    fn stylesheet_size_variants_reference_control_tokens() {
        // イシュー #1489: size 5 段の値を #1678 の
        // `--fandhe-size-control-height-*`/`--fandhe-size-control-font-size-*`
        // 参照へ揃えたことを固定する。
        let css = stylesheet();
        for suffix in ["xs", "sm", "md", "lg", "xl"] {
            assert!(
                css.contains(&format!("--fandhe-size-control-height-{suffix}")),
                "size={suffix} の height トークン参照が見つからない: {css}"
            );
            assert!(
                css.contains(&format!("--fandhe-size-control-font-size-{suffix}")),
                "size={suffix} の font-size トークン参照が見つからない: {css}"
            );
        }
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
        let html = render(&root(Size::Md, false, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="pin-input""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md() {
        let html = render(&root(Size::Md, false, false, vec![], vec![]));
        assert!(html.contains("fd-pin-input--size-md"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-pin-input--size-xs"),
            (Size::Sm, "fd-pin-input--size-sm"),
            (Size::Md, "fd-pin-input--size-md"),
            (Size::Lg, "fd-pin-input--size-lg"),
            (Size::Xl, "fd-pin-input--size-xl"),
        ] {
            let html = render(&root(size, false, false, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            false,
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
        assert!(css.contains("--fandhe-pin-input-size"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            false,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="pin-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn root_reflects_complete_and_disabled_props() {
        let html = render(&root(Size::Md, true, false, vec![], vec![]));
        assert!(html.contains(r#"data-complete="""#));

        let html = render(&root(Size::Md, false, true, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            Size::Md,
            false,
            false,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_label_children_are_escaped_on_render() {
        let html = render(&label(
            false,
            &PinInputProps::default(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
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
    fn ssr_and_hydration_round_trip_via_headless_pin_input_state_machine() {
        // `PinInput` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「選択的 re-export」節参照）ため、headless-ui から
        // 直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_headless_ui::pin_input::PinInput;

        let mut p = PinInput::new(4, PinInputKind::Numeric);
        assert!(!p.is_complete());

        let ssr_html = render(&p.root(&PinInputProps::default(), vec![], vec![]));
        assert!(!ssr_html.contains("data-complete"));

        assert!(dispatch(&mut p, "paste", "1234"));
        let hydrate_html = render(&render_for_hydration(&p));
        assert!(hydrate_html.contains(r#"data-hydrate-count="4""#));

        let restored = PinInput::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored.value(), p.value());
    }
}

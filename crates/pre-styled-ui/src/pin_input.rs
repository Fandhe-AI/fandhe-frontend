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
//! `input` slot へ登録する。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};

// `PinInput` 状態機械・headless 自由関数 `root` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::pin_input::PinInput` を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::pin_input::{
    control, hidden_input, input, label, PinInputAction, PinInputKind,
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
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
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
                decl("width", "var(--fandhe-pin-input-size, 2.5rem)"),
                decl("height", "var(--fandhe-pin-input-size, 2.5rem)"),
                decl(
                    "font-size",
                    "var(--fandhe-pin-input-font-size, var(--fandhe-font-font-size-md))",
                ),
                decl("text-align", "center"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("transition", "border-color 0.15s, background 0.15s"),
            ],
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
        // 実フォーカスを `input` 自身が受けるため `:focus-visible` を直接
        // 登録する（モジュール rustdoc「フォーカスリング」節参照）。
        .state(
            "input",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-pin-input-size", "1.5rem"),
                decl(
                    "--fandhe-pin-input-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-pin-input-size", "2rem"),
                decl(
                    "--fandhe-pin-input-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-pin-input-size", "2.5rem"),
                decl(
                    "--fandhe-pin-input-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-pin-input-size", "3rem"),
                decl(
                    "--fandhe-pin-input-font-size",
                    "var(--fandhe-font-font-size-lg)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-pin-input-size", "3.5rem"),
                decl(
                    "--fandhe-pin-input-font-size",
                    "var(--fandhe-font-font-size-xl)",
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
    fandhe_frontend_headless_ui::pin_input::root(complete, disabled, merged, children)
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
        assert!(css.contains(
            r#"[data-scope="pin-input"][data-part="input"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}"#
        ));
    }

    #[test]
    fn stylesheet_links_root_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="pin-input"][data-part="root"][data-disabled] {"#));
        assert!(css.contains("cursor: not-allowed;"));
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

        let ssr_html = render(&p.root(false, vec![], vec![]));
        assert!(!ssr_html.contains("data-complete"));

        assert!(dispatch(&mut p, "paste", "1234"));
        let hydrate_html = render(&render_for_hydration(&p));
        assert!(hydrate_html.contains(r#"data-hydrate-count="4""#));

        let restored = PinInput::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored.value(), p.value());
    }
}

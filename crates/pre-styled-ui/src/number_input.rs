//! styled NumberInput（headless ラッパー、イシュー #738、親 #520/#545/#736）。
//!
//! `fandhe_frontend_headless_ui::number_input`（イシュー #738）の Label /
//! Control / Input / IncrementTrigger / DecrementTrigger の 5 anatomy
//! パーツをそのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供
//! する。薄い委譲の根拠は [`crate::switch`]/[`crate::radio_group`] の
//! rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`NumberInput` 型・
//! headless `root` を再エクスポートしない理由）
//!
//! 本モジュールは `size` variant クラス付与のため styled `root`
//! （[`crate::switch::root`]・[`crate::avatar::root`] と同型）を本モジュール
//! で再定義する。headless 自由関数 `root` と名前衝突するため、`pub use
//! ...::*` ではなく必要な識別子（[`label`]/[`control`]/[`input`]/
//! [`increment_trigger`]/[`decrement_trigger`]/[`NumberInputAction`]/
//! [`NumberInputFlags`]）のみを選択的に再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::number_input::NumberInput`] は
//! **あえて**再エクスポートしない（[`crate::switch`] の `Switch` 非再
//! エクスポートと同じ理由）。`NumberInput` は `.root(disabled, invalid,
//! attrs, children)` という inherent メソッドを持つが、これは headless
//! 自由関数 `root` へそのまま委譲するのみで `size` variant クラスを一切
//! 付与しない未スタイルの実体である。本モジュールが `NumberInput` を丸ごと
//! 再エクスポートすると、呼び出し側が（styled 層のつもりで）
//! `number_input_instance.root(...)` を呼んでしまい、`size` が付与されず
//! 見た目が静かに崩れる事故を誘発する。`NumberInput` による状態管理・
//! hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::number_input::NumberInput` を直接 import
//! し、実際の描画は本モジュールの styled [`root`]（および再エクスポート済み
//! のパーツ関数）を組み合わせて構築すること。
//!
//! # `data-state` を持たない理由
//!
//! headless 層（`crates/headless-ui/src/number_input.rs`）は連続量の値を
//! 扱うため `data-state` を持たない（モジュール doc 参照）。[`recipe`] の
//! 境界到達時のスタイルは `increment-trigger`/`decrement-trigger` の
//! `data-disabled` 存在属性のみを条件にする。
//!
//! # `size` variant（イシュー #708 方針の踏襲）
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-number-input-control-height`/`-font-size`/`-trigger-size`
//! （root スコープの CSS custom property。通常の CSS 継承により
//! `control`/`input`/`increment-trigger`/`decrement-trigger` へ伝わる）
//! 経由で寸法を切り替える。`color-palette` 軸は本コンポーネントでは提供
//! しない（`crate` rustdoc「複合部品の variant 統一方針」の軸提供基準 3
//! に従い、フォーム操作部品として `size` のみを対象とする。選択・チェック
//! 状態を示す色ではなく汎用フォーム入力のため）。base 規則の `var()` には
//! Md 相当のフォールバック値を書き、styled `root` を経由しない headless
//! 直接利用マークアップでも現行外観を維持する（fail-safe）。
//!
//! # トリガーの視覚配置（縦積み、chakra-ui 風）
//!
//! `control` を `position: relative` のコンテナとし、
//! `increment-trigger`/`decrement-trigger` を右端に縦に積むレイアウトを
//! 既定 CSS として提供する（chakra-ui NumberInput の既定見た目に近づける
//! 判断）。フォーカスリングは通常のフォーカス可能要素（`input`/`button`）が
//! ネイティブに受けるため、[`crate::switch`] のような hidden-input 特有の
//! `data-focus-visible` 対応は不要（`input`/トリガー自体がフォーカスを
//! 受ける契約、モジュール doc「`data-state` を持たない理由」参照）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく Scrubber パーツ・キーボード操作の DOM 配線は
//!   スコープ外（`fandhe_frontend_headless_ui::number_input` モジュール
//!   doc 参照）。
//! - `examples/headless-pre-styled-ui`（crates.io バージョン依存）への
//!   NumberInput 追加は、未公開の新バージョンを参照できないため本イシュー
//!   のスコープ外とする（9c0e4f6 の先例どおり crates.io 公開後に追随）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};

// `NumberInput` 状態機械・headless 自由関数 `root` はあえて再エクスポート
// しない（本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::number_input::NumberInput` を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::number_input::{
    control, decrement_trigger, increment_trigger, input, label, NumberInputAction,
    NumberInputFlags,
};

/// headless `number_input` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/number_input.rs` の `ANATOMY.part(...)`
/// 呼び出しと同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を
/// 出力しない fail-closed 側の不具合として現れるため、変更時は両ファイルを
/// 合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "input",
    "increment-trigger",
    "decrement-trigger",
];

/// この styled NumberInput の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("number-input", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.5")],
        )
        .base(
            "label",
            vec![decl(
                "font-size",
                "var(--fandhe-number-input-font-size, var(--fandhe-font-font-size-sm))",
            )],
        )
        .base(
            "control",
            vec![
                decl("position", "relative"),
                decl("display", "inline-flex"),
                decl("align-items", "center"),
            ],
        )
        .base(
            "input",
            vec![
                decl("box-sizing", "border-box"),
                decl("width", "100%"),
                decl(
                    "height",
                    "var(--fandhe-number-input-control-height, 2.5rem)",
                ),
                decl(
                    "padding",
                    "0 var(--fandhe-number-input-trigger-size, 1.5rem) 0 var(--fandhe-space-2)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-number-input-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md, 0.375rem)"),
                decl("background", "var(--fandhe-color-bg)"),
            ],
        )
        .state(
            "input",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        .state(
            "input",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .base(
            "increment-trigger",
            vec![
                decl("position", "absolute"),
                decl("right", "1px"),
                decl("top", "1px"),
                decl("width", "var(--fandhe-number-input-trigger-size, 1.5rem)"),
                decl("height", "50%"),
                decl("border", "none"),
                decl("background", "transparent"),
                decl("cursor", "pointer"),
                decl("line-height", "1"),
            ],
        )
        .state(
            "increment-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.4")],
        )
        .base(
            "decrement-trigger",
            vec![
                decl("position", "absolute"),
                decl("right", "1px"),
                decl("bottom", "1px"),
                decl("width", "var(--fandhe-number-input-trigger-size, 1.5rem)"),
                decl("height", "50%"),
                decl("border", "none"),
                decl("background", "transparent"),
                decl("cursor", "pointer"),
                decl("line-height", "1"),
            ],
        )
        .state(
            "decrement-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.4")],
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-number-input-control-height", "1.5rem"),
                decl(
                    "--fandhe-number-input-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
                decl("--fandhe-number-input-trigger-size", "1rem"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-number-input-control-height", "2rem"),
                decl(
                    "--fandhe-number-input-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
                decl("--fandhe-number-input-trigger-size", "1.25rem"),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-number-input-control-height", "2.5rem"),
                decl(
                    "--fandhe-number-input-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
                decl("--fandhe-number-input-trigger-size", "1.5rem"),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-number-input-control-height", "3rem"),
                decl(
                    "--fandhe-number-input-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
                decl("--fandhe-number-input-trigger-size", "1.75rem"),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-number-input-control-height", "3.5rem"),
                decl(
                    "--fandhe-number-input-font-size",
                    "var(--fandhe-font-font-size-lg)",
                ),
                decl("--fandhe-number-input-trigger-size", "2rem"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled NumberInput が生成する静的 CSS 全量を返す（決定的。
/// [`crate::switch::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::number_input::root`]
/// へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::number_input;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = number_input::root(Size::Md, false, false, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="number-input" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    disabled: bool,
    invalid: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::number_input::root(disabled, invalid, merged, children)
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
        assert!(a.contains(r#"[data-scope="number-input"][data-part="control"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_triggers_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="number-input"][data-part="increment-trigger"][data-disabled] {"#
        ));
        assert!(css.contains(
            r#"[data-scope="number-input"][data-part="decrement-trigger"][data-disabled] {"#
        ));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn stylesheet_links_input_to_invalid_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="number-input"][data-part="input"][data-invalid] {"#));
        assert!(css.contains("border-color: var(--fandhe-color-danger);"));
    }

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(Size::Md, false, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="number-input""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md() {
        let html = render(&root(Size::Md, false, false, vec![], vec![]));
        assert!(html.contains("fd-number-input--size-md"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-number-input--size-xs"),
            (Size::Sm, "fd-number-input--size-sm"),
            (Size::Md, "fd-number-input--size-md"),
            (Size::Lg, "fd-number-input--size-lg"),
            (Size::Xl, "fd-number-input--size-xl"),
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
        assert!(css.contains("--fandhe-number-input-control-height"));
        assert!(css.contains("--fandhe-number-input-trigger-size"));
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
        assert!(html.contains(r#"data-scope="number-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
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
            false,
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_input_name_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&input(
            PAYLOAD,
            None,
            None,
            "0",
            "100",
            NumberInputFlags::default(),
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_number_input_state_machine() {
        // `NumberInput` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「`NumberInput` 型を再エクスポートしない理由」参照）
        // ため、headless-ui から直接 import して state machine 契約のみ
        // 検証する。
        use fandhe_frontend_headless_ui::number_input::NumberInput;

        let mut n = NumberInput::new(Some(0.0), 0.0, 10.0, 1.0);
        assert_eq!(n.value(), Some(0.0));

        let ssr_html = render(&n.control(false, false, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-part="control""#));

        assert!(dispatch(&mut n, "increment", ""));
        let hydrate_html = render(&render_for_hydration(&n));
        assert!(hydrate_html.contains(r#"data-hydrate-value="1""#));

        let restored = NumberInput::from_hydration_attrs(&n.hydration_attrs()).unwrap();
        assert_eq!(restored, n);
    }
}

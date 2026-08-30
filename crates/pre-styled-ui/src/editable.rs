//! styled Editable（headless ラッパー、イシュー #745、親 #520/#736）。
//!
//! `fandhe_frontend_headless_ui::editable`（イシュー #745）の Label / Area /
//! Input / Preview / Control / EditTrigger / SubmitTrigger / CancelTrigger の
//! 8 anatomy パーツをそのまま再エクスポートし、[`stylesheet`] で既定 CSS を
//! 追加提供する。薄い委譲の根拠は [`crate::number_input`]/[`crate::slider`]
//! の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Editable` 型・
//! headless `root` を再エクスポートしない理由）
//!
//! 本モジュールは `size` variant クラス付与のため styled `root`
//! （[`crate::number_input::root`]/[`crate::slider::root`] と同型）を本
//! モジュールで再定義する。headless 自由関数 `root` と名前衝突するため、
//! `pub use ...::*` ではなく必要な識別子のみを選択的に再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::editable::Editable`] は
//! **あえて**再エクスポートしない（[`crate::number_input`] の `NumberInput`
//! 非再エクスポートと同じ理由）。`Editable` は `.root(...)` という inherent
//! メソッドを持つが、これは headless 自由関数 `root` へそのまま委譲するのみ
//! で `size` variant クラスを一切付与しない未スタイルの実体である。本
//! モジュールが `Editable` を丸ごと再エクスポートすると、呼び出し側が
//! （styled 層のつもりで）`editable_instance.root(...)` を呼んでしまい、
//! `size` が付与されず見た目が静かに崩れる事故を誘発する。状態管理・
//! hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::editable::Editable` を直接 import し、実際の
//! 描画は本モジュールの styled [`root`]（および再エクスポート済みのパーツ
//! 関数）を組み合わせて構築すること。
//!
//! # `size` variant（イシュー #708 方針の踏襲）
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-editable-font-size`（root スコープの CSS custom property。
//! 通常の CSS 継承により `input`/`preview` へ伝わる）経由で寸法を切り替える。
//! `color-palette` 軸は本コンポーネントでは提供しない（`crate` rustdoc
//! 「複合部品の variant 統一方針」の軸提供基準 3 に従い、フォーム操作部品
//! として `size` のみを対象とする。[`crate::number_input`] と同じ判断）。
//! base 規則の `var()` には Md 相当のフォールバック値を書き、styled `root`
//! を経由しない headless 直接利用マークアップでも現行外観を維持する
//! （fail-safe）。
//!
//! # `input`/`preview` の重ね合わせレイアウト（PR #792 Bugbot 指摘対応、Medium）
//!
//! `area` を CSS Grid の単一セル（`display: grid`）とし、`input`/`preview`
//! の双方に `grid-area: 1 / 1` を与えて同一グリッドセルへ重ねる
//! （chakra-ui Editable の既定見た目に近づける判断）。両者は headless 層の
//! `hidden` 属性で排他表示され、非表示側は `display: none`（`preview` は
//! 直上の `[hidden]` 規則、`input` は要素の UA 既定 `display` に対し本
//! モジュールが `display` を宣言しないため UA 既定の `[hidden]{display:none}`
//! がそのまま効く）になるため、グリッドの track サイズは表示中の 1 パーツ
//! のみで決まる。`position: relative` だけを宣言し `input`/`preview` を
//! 通常フローに残した旧実装は、両者が `area` の `inline-flex` 内で並んで
//! 描画され「重ね合わせ」にならず、chakra-ui 由来の見た目契約に反していた
//! （Bugbot 指摘）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - headless 層と同じく activationMode/submitMode の実挙動・autoResize は
//!   スコープ外（`fandhe_frontend_headless_ui::editable` モジュール doc
//!   参照）。
//! - `examples/headless-pre-styled-ui`（crates.io バージョン依存）への
//!   Editable 追加は、未公開の新バージョンを参照できないため本イシューの
//!   スコープ外とする（[`crate::slider`] の先例どおり crates.io 公開後に
//!   追随）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};

// `Editable` 状態機械・headless 自由関数 `root` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::editable::Editable` を直接 import する。
pub use fandhe_frontend_headless_ui::editable::{
    area, cancel_trigger, control, edit_trigger, input, label, preview, submit_trigger, EditMode,
    EditableAction, EditableActivationMode, EditableInputFlags, EditableInputProps,
    EditableSubmitMode,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// headless `editable` anatomy の `data-part` 一覧（`crates/headless-ui/src/editable.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "area",
    "input",
    "preview",
    "control",
    "edit-trigger",
    "submit-trigger",
    "cancel-trigger",
];

/// この styled Editable の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("editable", SLOTS)
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
                "var(--fandhe-editable-font-size, var(--fandhe-font-font-size-sm))",
            )],
        )
        .base(
            "area",
            vec![decl("position", "relative"), decl("display", "inline-grid")],
        )
        .base(
            "input",
            vec![
                decl("grid-area", "1 / 1"),
                decl("box-sizing", "border-box"),
                decl("width", "100%"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
                decl(
                    "font-size",
                    "var(--fandhe-editable-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md, 0.375rem)"),
                decl("background", "var(--fandhe-color-bg)"),
            ],
        )
        .state(
            "input",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .base(
            "preview",
            vec![
                decl("grid-area", "1 / 1"),
                decl("display", "inline-block"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
                decl(
                    "font-size",
                    "var(--fandhe-editable-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("border", "1px solid transparent"),
                decl("border-radius", "var(--fandhe-radius-md, 0.375rem)"),
                decl("cursor", "text"),
            ],
        )
        // PR #792 Bugbot 指摘対応（High）: preview の base 規則が
        // `display: inline-block` を宣言しており、UA 既定の
        // `[hidden] { display: none }`（詳細度 (0,1,0)）を
        // `[data-scope][data-part]`（詳細度 (0,2,0)）が上書きしてしまう。
        // edit モードで headless 層が付与する `hidden` 存在属性を確実に
        // 非表示化として機能させるため、より詳細度の高い `[hidden]`
        // 属性セレクタで `display: none` を明示的に上書きする
        // （`crate::dialog` の positioner[hidden] と同型の対処）。
        .state(
            "preview",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "preview",
            StateCondition::Attr("data-placeholder-shown"),
            vec![decl("color", "var(--fandhe-color-fg-muted, currentColor)")],
        )
        .base(
            "control",
            vec![
                decl("display", "inline-flex"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "edit-trigger",
            vec![
                decl("border", "none"),
                decl("background", "transparent"),
                decl("cursor", "pointer"),
            ],
        )
        .state(
            "edit-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.4")],
        )
        .base(
            "submit-trigger",
            vec![
                decl("border", "none"),
                decl("background", "transparent"),
                decl("cursor", "pointer"),
            ],
        )
        .state(
            "submit-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.4")],
        )
        .base(
            "cancel-trigger",
            vec![
                decl("border", "none"),
                decl("background", "transparent"),
                decl("cursor", "pointer"),
            ],
        )
        .state(
            "cancel-trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.4")],
        )
        .variant(
            Size::Xs,
            "root",
            vec![decl(
                "--fandhe-editable-font-size",
                "var(--fandhe-font-font-size-xs, 0.75rem)",
            )],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl(
                "--fandhe-editable-font-size",
                "var(--fandhe-font-font-size-xs, 0.75rem)",
            )],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl(
                "--fandhe-editable-font-size",
                "var(--fandhe-font-font-size-sm)",
            )],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl(
                "--fandhe-editable-font-size",
                "var(--fandhe-font-font-size-md)",
            )],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl(
                "--fandhe-editable-font-size",
                "var(--fandhe-font-font-size-lg)",
            )],
        )
        .default_variant(Size::Md)
}

/// この styled Editable が生成する静的 CSS 全量を返す（決定的。
/// [`crate::number_input::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::editable::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::editable;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = editable::root(
///     Size::Md,
///     editable::EditMode::Preview,
///     false,
///     false,
///     editable::EditableActivationMode::default(),
///     editable::EditableSubmitMode::default(),
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"data-scope="editable" data-part="root""#));
/// ```
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn root<'a>(
    size: Size,
    mode: EditMode,
    disabled: bool,
    readonly: bool,
    activation_mode: EditableActivationMode,
    submit_mode: EditableSubmitMode,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::editable::root(
        mode,
        disabled,
        readonly,
        activation_mode,
        submit_mode,
        merged,
        children,
    )
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
        assert!(a.contains(r#"[data-scope="editable"][data-part="area"]"#));
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
        assert!(
            css.contains(r#"[data-scope="editable"][data-part="edit-trigger"][data-disabled] {"#)
        );
        assert!(
            css.contains(r#"[data-scope="editable"][data-part="submit-trigger"][data-disabled] {"#)
        );
        assert!(
            css.contains(r#"[data-scope="editable"][data-part="cancel-trigger"][data-disabled] {"#)
        );
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn stylesheet_links_preview_to_placeholder_shown_state() {
        let css = stylesheet();
        assert!(css
            .contains(r#"[data-scope="editable"][data-part="preview"][data-placeholder-shown] {"#));
    }

    #[test]
    fn edit_mode_preview_hidden_attr_overrides_display_inline_block() {
        // PR #792 Bugbot 指摘対応（High）: preview の base 規則
        // `display: inline-block` が UA 既定の `[hidden] { display: none }`
        // を詳細度で上書きし、edit モードで headless 層が付与する `hidden`
        // 存在属性があっても preview が表示され続け、preview/edit の排他
        // 表示が壊れる不具合の回帰（`crate::avatar`/`crate::dialog`/
        // `crate::tooltip` で既に対処済みの同種の落とし穴）。`[hidden]`
        // 属性セレクタでの明示的な `display: none` 上書きが出力され、
        // base 規則より後段（= 詳細度同点時に優先される）で登録されることを
        // 固定する。
        let css = stylesheet();
        let preview_hidden_selector = r#"[data-scope="editable"][data-part="preview"][hidden] {"#;
        assert!(css.contains(preview_hidden_selector));
        let rule_start = css
            .find(preview_hidden_selector)
            .expect("preview[hidden] rule must be present");
        let rule_body = &css[rule_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        assert!(rule_body[..rule_end].contains("display: none;"));

        // base 規則（`display: inline-block` を含む）より後に出現すること。
        // 同一詳細度の CSS 規則はソース順で後者が勝つため、順序が逆転すると
        // 上書きが機能しない。
        let base_preview_selector = r#"[data-scope="editable"][data-part="preview"] {"#;
        let base_start = css
            .find(base_preview_selector)
            .expect("base preview rule must be present");
        assert!(base_start < rule_start);
    }

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            Size::Md,
            EditMode::Preview,
            false,
            false,
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="editable""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md() {
        let html = render(&root(
            Size::Md,
            EditMode::Preview,
            false,
            false,
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-editable--size-md"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-editable--size-xs"),
            (Size::Sm, "fd-editable--size-sm"),
            (Size::Md, "fd-editable--size-md"),
            (Size::Lg, "fd-editable--size-lg"),
            (Size::Xl, "fd-editable--size-xl"),
        ] {
            let html = render(&root(
                size,
                EditMode::Preview,
                false,
                false,
                EditableActivationMode::default(),
                EditableSubmitMode::default(),
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            EditMode::Preview,
            false,
            false,
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
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
        assert!(css.contains("--fandhe-editable-font-size"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            EditMode::Preview,
            false,
            false,
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="editable""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            Size::Md,
            EditMode::Preview,
            false,
            false,
            EditableActivationMode::default(),
            EditableSubmitMode::default(),
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_label_children_are_escaped_on_render() {
        let html = render(&label(
            EditMode::Preview,
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
            EditMode::Edit,
            PAYLOAD,
            PAYLOAD,
            EditableInputProps::default(),
            EditableInputFlags::default(),
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_editable_state_machine() {
        // `Editable` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「`Editable` 型を再エクスポートしない理由」参照）
        // ため、headless-ui から直接 import して state machine 契約のみ
        // 検証する。
        use fandhe_frontend_headless_ui::editable::Editable;

        let mut e = Editable::new("Ada", None);
        assert_eq!(e.value(), "Ada");

        let ssr_html = render(&e.control(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-part="control""#));

        assert!(dispatch(&mut e, "edit", ""));
        assert!(dispatch(&mut e, "set", "Grace Hopper"));
        let hydrate_html = render(&render_for_hydration(&e));
        assert!(hydrate_html.contains(r#"data-hydrate-draft="Grace Hopper""#));

        let restored = Editable::from_hydration_attrs(&e.hydration_attrs()).unwrap();
        assert_eq!(restored, e);
    }
}

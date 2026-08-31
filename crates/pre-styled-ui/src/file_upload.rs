//! styled FileUpload（headless ラッパー、イシュー #840、親 #520/#546）。
//!
//! `fandhe_frontend_headless_ui::file_upload`（イシュー #840）の
//! Label / Dropzone / Trigger / ItemGroup / Item / ItemName / ItemSizeText /
//! ItemDeleteTrigger / ClearTrigger / HiddenInput 10 anatomy パーツをそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠は [`crate::tags_input`] の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`FileUpload` 型・
//! headless `root` を再エクスポートしない理由）
//!
//! 本モジュールは `size` variant クラス付与のため styled `root`
//! （[`crate::tags_input::root`] と同型）を本モジュールで再定義する。headless
//! 自由関数 `root` と名前衝突するため、`pub use ...::*` ではなく必要な識別子
//! （[`label`]/[`dropzone`]/[`trigger`]/[`item_group`]/[`item`]/[`item_name`]/
//! [`item_size_text_node`]/[`item_delete_trigger`]/[`clear_trigger`]/
//! [`hidden_input`]/[`FileUploadAction`]/[`FileUploadItem`]/
//! [`FileRejectionReason`]）のみを選択的に再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::file_upload::FileUpload`] は
//! **あえて**再エクスポートしない（[`crate::tags_input`] の非再エクスポートと
//! 同じ理由）。状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::file_upload::FileUpload` を直接 import し、
//! 実際の描画は本モジュールの styled [`root`]（および再エクスポート済みの
//! パーツ関数）を組み合わせて構築すること。
//!
//! # `hidden-input` に CSS を付与しない理由
//!
//! [`crate::tags_input`] の `hidden-input` と同じ理由（`<input type="file">`
//! は視覚的に非表示にする構成を前提とする、モジュール doc の headless
//! 側 rustdoc 参照）で、[`recipe`] は `hidden-input` slot へ一切の CSS を
//! 登録しない（`hidden_input_slot_has_no_css_rules` テストで固定）。
//!
//! # `size` variant
//!
//! [`crate::switch`] rustdoc「複合部品の variant 統一方針」節（#708）に従い、
//! `size`（[`Size`]）は styled `root` へのみクラスを付与し、[`recipe`] が
//! 登録する `--fandhe-file-upload-*` の root スコープ CSS custom property
//! （通常の CSS 継承）経由で `item`/`trigger` の寸法・書体を切り替える。
//! `base`/`variant` 規則の `var()` にはいずれも Md サイズ相当のフォールバック
//! 値を書き、styled `root` を経由しない headless 直接利用マークアップでも
//! 現行外観を維持する（fail-safe）。フォーム入力部品のため palette
//! （color-palette 軸）は本イシューでは提供しない（[`crate::tags_input`]
//! の先例に倣う）。
//!
//! # フォーカスリング（`dropzone` がフォーカスを受ける構成）
//!
//! [`crate::tags_input`] の `input` と異なり、本コンポーネントの実操作対象は
//! `dropzone`（`role="button"` + `tabindex="0"`）であるため、
//! `StateCondition::FocusVisible`（`:focus-visible` 疑似クラス）を
//! `dropzone` slot へ登録する。
//!
//! # 内部パート（`item-group` 以下）のスタイル調整（イシュー #1697、親 #1478）
//!
//! 親イシュー #1478 の比較観点チェックリスト（hover / disabled /
//! transition / 状態の視覚反映）を内部パート（`item-group`/`item`/
//! `item-name`/`item-size-text`/`item-delete-trigger`）へ適用する。外枠パート
//! （`root`/`label`/`dropzone`/`trigger`）は兄弟イシュー #1696 の担当であり
//! 本節の対象外。
//!
//! - **`item` の `data-invalid`（headless 未出力、`attrs` 経由でのみ付与
//!   可能）**: [`crate::checkbox_group`] の同型記述に倣い、CSS 側は
//!   `[data-invalid]` セレクタを常時出力するが、headless
//!   `file_upload::item` はこの属性を出力しない。利用者が `item` の `attrs`
//!   へ `("data-invalid", "")` を直接渡すことで有効化する（`border-color`
//!   のみ danger 色化する Forms 家族共通の視覚言語、[`crate::checkbox`]/
//!   [`crate::checkbox_card`]/[`crate::checkbox_group`] と統一）。
//! - **`item` の `data-disabled` と `root` の disabled の opacity 重複を
//!   許容する理由**: headless の API 上 `root` の disabled と `item` の
//!   disabled は独立したフラグであり、両方 true にする構成は利用者判断に
//!   委ねられている（`crate::checkbox` 家族と同型の挙動）。`item` 単体を
//!   disabled にしても `root` は変化しないため、両方 disabled にした場合の
//!   opacity 二重適用（0.5 × 0.5）は意図的な許容であり、item 側の
//!   disabled 宣言を独自に弱めない。ただし三重適用（`root`×`item`×
//!   `item-delete-trigger`）までは許容しない: `item-delete-trigger` の
//!   `data-disabled` state は兄弟イシュー #1696 の `trigger`
//!   （`root` 配下で常用されるため `disabled_declarations` を使わず
//!   `cursor: not-allowed` のみに留めた判断）と同じ理由で
//!   `disabled_declarations()` を使わず `cursor: not-allowed` のみに
//!   留める（`item-delete-trigger` は常に `item` 配下で使われ、`item` の
//!   `data-disabled` state が既に `opacity: 0.5` を適用済みのため）。
//! - **`item`（`<li>` コンテナ）・`item-group`・`item-name`・
//!   `item-size-text` へ hover を付けない理由**: 表示専用の slot（クリック
//!   可能な操作面を持たない）であり、[`crate::recipe`] の「hover 共通
//!   ビジュアル言語」節が定める「インタラクティブ slot にのみ付ける」規則に
//!   従う。参照 3 サイト（ark-ui/chakra-ui/Radix）も file item 自体に hover
//!   背景変化を持たない。
//! - **`item-preview` パートは本コンポーネントの anatomy に存在しない**:
//!   headless `file_upload` の `SLOTS`（11 slot）は `item-preview` を持たず、
//!   `tags_input` の同名パートからの類推による誤認と判断し実装しない
//!   （親イシュー #1478 へ N/A として記録）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, hover_bg_muted, hover_surface_declarations, transition_declarations,
    MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};

// `FileUpload` 状態機械・headless 自由関数 `root` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は
// `fandhe_frontend_headless_ui::file_upload::FileUpload` を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::file_upload::{
    clear_trigger, dropzone, hidden_input, item, item_delete_trigger, item_group, item_name,
    item_size_text, item_size_text_node, label, trigger, FileRejectionReason, FileUploadAction,
    FileUploadItem,
};

/// headless `file_upload` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/file_upload.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を出力しない
/// fail-closed 側の不具合として現れるため、変更時は両ファイルを合わせて
/// 確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "dropzone",
    "trigger",
    "item-group",
    "item",
    "item-name",
    "item-size-text",
    "item-delete-trigger",
    "clear-trigger",
    "hidden-input",
];

/// この styled FileUpload の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("file-upload", SLOTS)
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
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        .base(
            "label",
            vec![decl("font-size", "var(--fandhe-font-font-size-sm)")],
        )
        .base(
            "dropzone",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl(
                    "padding",
                    "var(--fandhe-file-upload-dropzone-padding, var(--fandhe-space-6))",
                ),
                decl("gap", "var(--fandhe-space-2)"),
                decl("border", "2px dashed var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("cursor", "pointer"),
            ],
        )
        .state(
            "dropzone",
            StateCondition::Attr("data-dragging"),
            vec![
                decl("border-color", "var(--fandhe-color-accent)"),
                decl("background", "var(--fandhe-color-bg-subtle)"),
            ],
        )
        .state(
            "dropzone",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .state(
            "dropzone",
            StateCondition::FocusVisible,
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .base(
            "trigger",
            vec![
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("cursor", "pointer"),
                decl(
                    "font-size",
                    "var(--fandhe-file-upload-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-3)"),
            ],
        )
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .base(
            "item-group",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("padding", "0"),
                decl("margin", "0"),
                decl("list-style", "none"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("box-sizing", "border-box"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl(
                    "font-size",
                    "var(--fandhe-file-upload-font-size, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結されるため、
        // 上記 base ブロックを書き換えずに純追加する（`checkbox_group.rs` と
        // 同型）。`transition-property` は `data-invalid` が変える
        // `border-color` のみを列挙する（`opacity`（`data-disabled` 用）は
        // 本クレートの他モジュールに前例がない組み合わせのため見送り、
        // `background`（`item` に動的変化の規則が無い）も含めない。
        // `checkbox_group` の `item-control` と異なり本 slot は hover を
        // 持たない、モジュール rustdoc 参照）。
        .base(
            "item",
            transition_declarations("border-color", MotionDuration::Fast),
        )
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // `data-invalid`（headless 未出力、`attrs` 経由でのみ付与可能。
        // モジュール rustdoc「内部パートのスタイル調整」節参照）。
        .state(
            "item",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        .base(
            "item-name",
            vec![
                decl("flex", "1 1 auto"),
                decl("overflow", "hidden"),
                decl("text-overflow", "ellipsis"),
                decl("white-space", "nowrap"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .base(
            "item-size-text",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
            ],
        )
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
                // ghost 系（背景 transparent）のため `hover_bg_muted()` で
                // `--fandhe-hover-bg` を muted 背景に定義する（モジュール
                // rustdoc「内部パートのスタイル調整」節参照）。
                hover_bg_muted(),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結されるため、
        // 上記 base ブロックを書き換えずに純追加する（`checkbox_group.rs` と
        // 同型）。`transition-property` は実際に変化する `background`
        // （hover）のみを列挙する（`color` は本 recipe のどの規則も変更
        // しないため含めない）。
        .base(
            "item-delete-trigger",
            transition_declarations("background", MotionDuration::Fast),
        )
        .state(
            "item-delete-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // `disabled_declarations()`（opacity 0.5 + cursor）ではなく
        // `cursor: not-allowed` のみに留める（PR #1696 の `trigger` と同じ
        // 判断: `item-delete-trigger` は常に `item` 配下で使われ、`item` の
        // `data-disabled` state が既に `opacity: 0.5` を適用済みのため、
        // ここでも `disabled_declarations` を使うと `root`（0.5）×
        // `item`（0.5）× `item-delete-trigger`（0.5）で opacity が 0.125
        // まで三重に減衰してしまう。モジュール rustdoc「内部パートの
        // スタイル調整」節の「opacity 二重適用の許容」は `root`×`item` の
        // 2 段に限った判断であり、3 段目はここで避ける）。
        .state(
            "item-delete-trigger",
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
                    "var(--fandhe-file-upload-font-size, var(--fandhe-font-font-size-sm))",
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
            vec![decl(
                "--fandhe-file-upload-font-size",
                "var(--fandhe-font-font-size-xs)",
            )],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl(
                "--fandhe-file-upload-font-size",
                "var(--fandhe-font-font-size-xs)",
            )],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl(
                "--fandhe-file-upload-font-size",
                "var(--fandhe-font-font-size-sm)",
            )],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl(
                "--fandhe-file-upload-font-size",
                "var(--fandhe-font-font-size-md)",
            )],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl(
                "--fandhe-file-upload-font-size",
                "var(--fandhe-font-font-size-lg)",
            )],
        )
        .default_variant(Size::Md)
}

/// この styled FileUpload が生成する静的 CSS 全量を返す（決定的。
/// [`crate::tags_input::stylesheet`]と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::file_upload::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::file_upload;
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = file_upload::root(Size::Md, false, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="file-upload" data-part="root""#));
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
    fandhe_frontend_headless_ui::file_upload::root(disabled, merged, children)
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
        assert!(a.contains(r#"[data-scope="file-upload"][data-part="dropzone"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_dropzone_to_dragging_state() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="file-upload"][data-part="dropzone"][data-dragging] {"#)
        );
    }

    #[test]
    fn stylesheet_links_dropzone_to_focus_visible_outline() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="file-upload"][data-part="dropzone"]:focus-visible {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}"#
        ));
    }

    #[test]
    fn stylesheet_links_root_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="file-upload"][data-part="root"][data-disabled] {"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn hidden_input_slot_has_no_css_rules() {
        // モジュール rustdoc「`hidden-input` に CSS を付与しない理由」参照。
        let css = stylesheet();
        assert!(!css.contains(r#"[data-part="hidden-input"]"#));
    }

    // --- 内部パート（イシュー #1697） ---

    #[test]
    fn stylesheet_links_item_to_invalid_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="file-upload"][data-part="item"][data-invalid] {"#));
        assert!(css.contains("border-color: var(--fandhe-color-danger);"));
    }

    #[test]
    fn stylesheet_links_item_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="file-upload"][data-part="item"][data-disabled] {"#));
    }

    #[test]
    fn stylesheet_wraps_delete_trigger_hover_in_hover_media() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(
            r#"[data-scope="file-upload"][data-part="item-delete-trigger"]:hover:not([data-disabled]) {"#
        ));
    }

    #[test]
    fn stylesheet_uses_motion_tokens_for_item_and_delete_trigger_transitions() {
        let css = stylesheet();
        assert!(css.contains("var(--fandhe-motion-duration-fast)"));
        assert!(css.contains("transition-property: border-color;"));
        assert!(css.contains("transition-property: background;"));
    }

    // --- variant クラス ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(Size::Md, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="file-upload""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md() {
        let html = render(&root(Size::Md, false, vec![], vec![]));
        assert!(html.contains("fd-file-upload--size-md"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-file-upload--size-xs"),
            (Size::Sm, "fd-file-upload--size-sm"),
            (Size::Md, "fd-file-upload--size-md"),
            (Size::Lg, "fd-file-upload--size-lg"),
            (Size::Xl, "fd-file-upload--size-xl"),
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
        assert!(css.contains("--fandhe-file-upload-font-size"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="file-upload""#));
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
    fn reexported_item_name_tag_payload_is_escaped_on_render() {
        let html = render(&item_name(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn reexported_item_delete_trigger_aria_label_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&item_delete_trigger(PAYLOAD, false, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_hidden_input_accept_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&hidden_input(PAYLOAD, false, false, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_file_upload_state_machine() {
        // `FileUpload` は本モジュールから再エクスポートしない（本モジュール
        // 冒頭の rustdoc「選択的 re-export」節参照）ため、headless-ui から
        // 直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_headless_ui::file_upload::FileUpload;
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Component, Hydrate};

        let mut f = FileUpload::new("image/*", Some(5), None, None);
        assert!(f.is_empty());

        let ssr_html = render(&f.root(false, vec![], vec![]));
        assert!(!ssr_html.contains("data-hydrate-"));

        f.update(FileUploadAction::AddFiles(vec![FileUploadItem::new(
            "a.png",
            100,
            "image/png",
        )]));
        let hydrate_html = render(&render_for_hydration(&f));
        assert!(hydrate_html.contains(r#"data-hydrate-max-files="5""#));

        let restored = FileUpload::from_hydration_attrs(&f.hydration_attrs()).unwrap();
        assert_eq!(restored.accepted(), f.accepted());

        assert!(dispatch(&mut f, "clear", ""));
        assert!(f.is_empty());
    }
}

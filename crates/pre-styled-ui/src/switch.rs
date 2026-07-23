//! styled Switch（headless ラッパー第 3 弾、イシュー #682、`size`/`palette`
//! variant 拡張はイシュー #708、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::switch`（イシュー #537/#595）の Control /
//! Thumb / Label / HiddenInput 4 anatomy パーツをそのまま再エクスポートし、
//! [`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠は
//! [`crate::dialog`]/[`crate::popover`]/[`crate::tooltip`] の rustdoc と同じ
//! 方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Switch` 型・headless
//! `root` を再エクスポートしない理由、イシュー #708）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::avatar::root`]・[`crate::card::root`] と同型）を本モジュールで
//! 再定義する。headless 自由関数 `root` と名前衝突するため、
//! `pub use ...::*` ではなく必要な識別子（[`control`]/[`thumb`]/[`label`]/
//! [`hidden_input`]/[`SwitchAction`]）のみを選択的に再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::switch::Switch`] は**あえて**
//! 再エクスポートしない（[`crate::avatar`] の `Avatar` 非再エクスポートと
//! 同じ理由、イシュー #684/PR #695 Bugbot 指摘）。`Switch` は
//! `.root(disabled, attrs, children)` という inherent メソッドを持つが、
//! これは headless 自由関数 `root` へそのまま委譲するのみで `size`/
//! `palette` variant クラスを一切付与しない未スタイルの実体である。本
//! モジュールが `Switch` を丸ごと再エクスポートすると、呼び出し側が
//! （styled 層のつもりで）`switch_instance.root(...)` を呼んでしまい、
//! `size`/`palette` が付与されず見た目が静かに崩れる事故を誘発する。
//! `Switch` による状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::switch::Switch` を直接 import し、実際の
//! 描画は本モジュールの styled [`root`]（および再エクスポート済みの
//! パーツ関数）を組み合わせて構築すること。
//!
//! # `data-state` 語彙について
//!
//! headless 層は Switch を `"checked"`/`"unchecked"` 語彙（open/closed では
//! ない）で表現する（`crates/headless-ui/src/switch.rs` の
//! [`crate::state::Checkable`] 埋め込み参照）。[`recipe`] の `control`/`thumb`
//! への状態連動規則もこの語彙に合わせて `data-state="checked"` を条件とする。
//!
//! # `hidden-input` は `display: none` にしない（視覚的非表示化の判断）
//!
//! headless 層の `hidden_input` は `<input type="checkbox" role="switch">`
//! で意味論・フォーム送信・キーボード操作を担う実体であり、視覚的な見た目
//! （トラック/つまみ）は `control`/`thumb` が装飾として担う。この 2 層構造を
//! 保ちつつ `hidden_input` 自体のフォーカス・タブ順・支援技術からの到達性を
//! 失わないため、`display: none`/`visibility: hidden` ではなく
//! [`crate::select`] の `hidden-select` と同じ visually-hidden パターン
//! （`position: absolute` + 1px クリップ、PR #575 Bugbot 指摘対応の前例）を
//! 採用する。
//!
//! # `control` の `box-sizing: border-box`（PR #697 Bugbot 指摘対応）
//!
//! `control` の `width`/水平 `padding` と、checked 時の `thumb` の
//! `translateX` はいずれも border-box（`padding` を `width` に含める箱
//! モデル）を前提に値を計算している。既定の content-box のままだと
//! `width` に `padding` が加算されトラック内の実効幅がずれ、checked 時に
//! `thumb` がトラック右端まで届かない／両端の余白が不均等になる。この
//! クレート・利用側 embed にグローバルな border-box リセットは無いため、
//! `control` へ明示的に `box-sizing: border-box` を設定して自己完結させる。
//!
//! # `hidden-input` フォーカス時の `control` へのフォーカスリング反映（イシュー #709）
//!
//! `hidden-input` フォーカス時に `control` へフォーカスリングを反映する
//! 課題は、[`crate::recipe::StateCondition`] へ親子・兄弟関係の関係セレクタ
//! （`:has()`・兄弟結合子）を追加するのではなく、headless 層
//! （`fandhe_frontend_headless_ui::data_attrs::data_focus_visible`）が
//! 出力する `data-focus-visible` 存在属性 + クライアントランタイム
//! （`fandhe-frontend-wasm-full` の focus 配線）による `root`/`control`
//! 双方への付け外しで解決する（`crates/headless-ui/src/switch.rs` の
//! フォーカスリング契約 doc 参照）。本モジュールは `control` slot へ
//! `StateCondition::Attr("data-focus-visible")` の状態規則を登録するのみで、
//! 属性の付け外し自体は headless/wasm 層の責務のまま変えない（旧版で
//! 本節に記載していた out-of-scope はこの解決により解消済み）。
//!
//! # `size`/`palette` variant（イシュー #708）
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-switch-track-width`/`-track-height`/`-thumb-size`/
//! `-thumb-travel`/`-label-font-size` の root スコープ custom property
//! （CSS の通常のプロパティ継承により `control`/`thumb`/`label` へ伝わる。
//! `root` は `<label>` でこれらのパーツを内包する祖先要素であるため、
//! [`crate::recipe::SlotRecipe`] へ子孫セレクタ機構を追加せずに実現できる）
//! 経由で `control`/`thumb`/`label` の寸法を切り替える。`palette`
//! （[`ColorPalette`]）は既存の [`crate::recipe::palette_declarations`]
//! （chakra-ui virtual token 方式、#606）を `root` へ登録し、checked 時の
//! `control` 背景・`thumb` の色を `var(--fandhe-palette, ...)` 経由で
//! 切り替える。`base`/`state` 規則の `var()` にはいずれも Md サイズ・
//! Accent パレット相当のフォールバック値を書き、styled `root` を経由しない
//! headless 直接利用マークアップでも現行外観を維持する（fail-safe、
//! `crate::lib` rustdoc「複合部品の variant 統一方針」節参照）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - [`crate::stylesheet::StyleSheet`] の
//!   `push_recipe_is_infallible_for_all_styled_components` テストへの
//!   popover/tooltip（#664）の登録漏れは #707 で解消済み。
//! - tabs/accordion/dialog/menu/select への size（および tabs への
//!   palette）展開は本イシューの方針を第 2 弾として別途適用する
//!   （`docs/api/pre-styled-ui-api.md` の variant 表参照）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    palette_declarations, ColorPalette, Size, SlotRecipe, StateCondition, VariantValue,
};

// `Switch` 状態機械・headless 自由関数 `root` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。状態管理・
// hydration が必要な呼び出し側は `fandhe_frontend_headless_ui::switch::Switch`
// を直接 import する。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::switch::{control, hidden_input, label, thumb, SwitchAction};

/// headless `switch` anatomy の `data-part` 一覧（`crates/headless-ui/src/switch.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &["root", "control", "thumb", "label", "hidden-input"];

/// この styled Switch の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("switch", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("cursor", "pointer"),
            ],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed"), decl("opacity", "0.5")],
        )
        .base(
            "control",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("box-sizing", "border-box"),
                decl("width", "var(--fandhe-switch-track-width, 2.5rem)"),
                decl("height", "var(--fandhe-switch-track-height, 1.4rem)"),
                decl("border-radius", "999px"),
                decl("background", "var(--fandhe-color-border)"),
                decl("padding", "0 0.15rem"),
                decl("transition", "background 0.15s"),
            ],
        )
        .state(
            "control",
            StateCondition::AttrEq("data-state", "checked"),
            vec![decl(
                "background",
                "var(--fandhe-palette, var(--fandhe-color-accent))",
            )],
        )
        // イシュー #709: 実フォーカスは hidden-input が受けるため、wasm 層
        // （`fandhe-frontend-wasm-full` の focus 配線）が `control` へも
        // 付け外しする `data-focus-visible` をキーボード操作専用のフォーカス
        // リング条件として使う（`select` の `trigger`
        // `StateCondition::FocusVisible` と同じ視覚言語、モジュール rustdoc 参照）。
        .state(
            "control",
            StateCondition::Attr("data-focus-visible"),
            vec![
                decl("outline", "2px solid var(--fandhe-color-accent)"),
                decl("outline-offset", "2px"),
            ],
        )
        .base(
            "thumb",
            vec![
                decl("width", "var(--fandhe-switch-thumb-size, 1.1rem)"),
                decl("height", "var(--fandhe-switch-thumb-size, 1.1rem)"),
                decl("border-radius", "999px"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("transition", "transform 0.15s"),
            ],
        )
        .state(
            "thumb",
            StateCondition::AttrEq("data-state", "checked"),
            vec![decl(
                "transform",
                "translateX(var(--fandhe-switch-thumb-travel, 1.1rem))",
            )],
        )
        .base(
            "label",
            vec![decl(
                "font-size",
                "var(--fandhe-switch-label-font-size, var(--fandhe-font-font-size-sm))",
            )],
        )
        // hidden-input の視覚的非表示化（[`crate::select`] の `hidden-select` と
        // 同じ visually-hidden パターン。モジュール doc 参照）。
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
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-switch-track-width", "2rem"),
                decl("--fandhe-switch-track-height", "1.15rem"),
                decl("--fandhe-switch-thumb-size", "0.85rem"),
                decl("--fandhe-switch-thumb-travel", "0.85rem"),
                decl(
                    "--fandhe-switch-label-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-switch-track-width", "2.5rem"),
                decl("--fandhe-switch-track-height", "1.4rem"),
                decl("--fandhe-switch-thumb-size", "1.1rem"),
                decl("--fandhe-switch-thumb-travel", "1.1rem"),
                decl(
                    "--fandhe-switch-label-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-switch-track-width", "3rem"),
                decl("--fandhe-switch-track-height", "1.65rem"),
                decl("--fandhe-switch-thumb-size", "1.35rem"),
                decl("--fandhe-switch-thumb-travel", "1.35rem"),
                decl(
                    "--fandhe-switch-label-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(ColorPalette::Accent);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
    ] {
        recipe = recipe.variant(palette, "root", palette_declarations(palette));
    }
    recipe
}

/// この styled Switch が生成する静的 CSS 全量を返す（決定的。
/// [`crate::dialog::stylesheet`]/[`crate::tooltip::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与する
/// 唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去して
/// から合成する）。実体は [`fandhe_frontend_headless_ui::switch::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::switch::{self, SwitchAction as _};
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = switch::root(Size::Md, ColorPalette::Accent, false, false, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="switch" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    checked: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::switch::root(checked, disabled, merged, children)
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
        assert!(a.contains(r#"[data-scope="switch"][data-part="control"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_control_and_thumb_to_checked_state() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="switch"][data-part="control"][data-state="checked"] {
  background: var(--fandhe-palette, var(--fandhe-color-accent));
}"#
        ));
        assert!(css.contains(
            r#"[data-scope="switch"][data-part="thumb"][data-state="checked"] {
  transform: translateX(var(--fandhe-switch-thumb-travel, 1.1rem));
}"#
        ));
    }

    #[test]
    fn control_uses_border_box_so_thumb_travel_matches_track_bounds() {
        // Cursor Bugbot 指摘（PR #697, review 3636964684）対応の回帰:
        // `control` の `width`/`padding` と `thumb` の `translateX` は
        // border-box を前提に計算されている。`box-sizing: border-box` が
        // 欠けると content-box 既定によりつまみの移動量とトラック内幅が
        // ずれる（checked 時につまみが手前で止まる／両端の余白が不均等）。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="switch"][data-part="control"] {
  display: inline-flex;
  align-items: center;
  box-sizing: border-box;
  width: var(--fandhe-switch-track-width, 2.5rem);"#
        ));
    }

    #[test]
    fn stylesheet_links_root_to_disabled_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="switch"][data-part="root"][data-disabled] {"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn hidden_input_is_visually_hidden_not_display_none() {
        // フォーカス・フォーム送信・支援技術の到達性を保つため
        // `display: none` を使わないことをモジュール doc 通りに固定する
        // （フォーカス到達性の回帰防止）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="switch"][data-part="hidden-input"] {"#));
        assert!(css.contains("clip: rect(0, 0, 0, 0);"));
        assert!(!css.contains("display: none"));
    }

    // --- variant クラス（イシュー #708） ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-switch--size-md"));
        assert!(html.contains("fd-switch--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-switch--size-sm"),
            (Size::Md, "fd-switch--size-md"),
            (Size::Lg, "fd-switch--size-lg"),
        ] {
            let html = render(&root(
                size,
                ColorPalette::Accent,
                false,
                false,
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-switch--color-palette-accent"),
            (ColorPalette::Info, "fd-switch--color-palette-info"),
            (ColorPalette::Success, "fd-switch--color-palette-success"),
            (ColorPalette::Warning, "fd-switch--color-palette-warning"),
            (ColorPalette::Danger, "fd-switch--color-palette-danger"),
        ] {
            let html = render(&root(Size::Md, palette, false, false, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            false,
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
        assert!(css.contains("--fandhe-switch-track-width"));
    }

    #[test]
    fn size_variants_set_label_font_size_custom_property() {
        // Cursor Bugbot 指摘（PR #719 レビュー）対応の回帰: `label` の base
        // 規則が参照する `--fandhe-switch-label-font-size` を各 size
        // variant が設定していないと、control 自体はスケールしてもラベル
        // 文字サイズがフォールバック（sm 相当）のまま変わらない
        // （`radio_group.rs` の `--fandhe-radio-group-font-size` と対称の
        // 契約）。
        let css = stylesheet();
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let selector = format!(
                r#"[data-scope="switch"][data-part="root"].fd-switch--size-{}"#,
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
                css[start..block_end].contains("--fandhe-switch-label-font-size"),
                "size={size:?} variant block missing --fandhe-switch-label-font-size: {}",
                &css[start..block_end]
            );
        }
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        // headless anatomy の fail-closed 偽装除去を styled root 経由でも
        // 継承していることの回帰（[`crate::avatar`] の同型テストに準拠）。
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
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
        // イシュー #682: styled Switch 経由でも既定エスケープ（REQ-1）が
        // 効くことを固定する（headless ラッパー第 1・2 弾と同じ回帰）。
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
        let html = render(&hidden_input(PAYLOAD, PAYLOAD, false, false, false, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_switch_state_machine() {
        // `Switch` は本モジュールから再エクスポートしない（本モジュール冒頭の
        // rustdoc「`Switch` 型を再エクスポートしない理由」参照）ため、
        // headless-ui から直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_headless_ui::switch::Switch;

        let mut s = Switch::default();
        assert!(!s.is_checked());

        let ssr_html = render(&s.root(false, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="unchecked""#));

        assert!(dispatch(&mut s, "toggle", ""));
        let hydrate_html = render(&render_for_hydration(&s));
        assert!(hydrate_html.contains(r#"data-hydrate-checked="checked""#));

        let restored = Switch::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }
}

//! styled DatePicker（headless ラッパー、イシュー #835、親 #520/#546）。
//!
//! `fandhe_frontend_headless_ui::date_picker` の Root / Label / Control /
//! Input / Trigger / ClearTrigger / Positioner / Content 8 anatomy パーツを
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠は [`crate::calendar`]（本クレート内の同型先行例）の rustdoc と同じ
//! 方針に従う。`content` 内部に [`crate::calendar`] の styled パーツを合成
//! する想定である。
//!
//! # 選択的 re-export
//!
//! `size` variant クラス付与のため styled [`root`] を本モジュールで新設する。
//! 状態機械 [`fandhe_frontend_headless_ui::date_picker::DatePicker`] は
//! **あえて**再エクスポートしない（[`crate::calendar`]/[`crate::select`] と
//! 同じ理由）。
//!
//! # スタイル調整（イシュー #1471、control/input/trigger/clear-trigger
//! パートのみ）
//!
//! 親 #1470（chakra-ui / ark-ui 基準への調整、Phase 2 / ルート #1420）のうち
//! `control`/`input`/`trigger`/`clear-trigger` の 4 パートを担当する分割
//! 1/3。分割 2/3（カレンダーグリッド、#1472）・3/3（ビュー切り替え・
//! ポジショナ、#1473）とはファイルを共有するため、以下は本イシューが
//! 確定した意図的差分である（combobox 1/2、PR #1744・イシュー #1467 と
//! 同型の記録方針）:
//!
//! - **radius トークン化**: `input`/`trigger` の `border-radius`（生
//!   `0.375rem`）を `var(--fandhe-radius-md)` へ置換した。`root`/`label`/
//!   `positioner`/`content` は 2/3・3/3 のスコープのため変更しない
//! - **canonical フォーカスリング**: `input`/`trigger` の
//!   `:focus-visible` を [`crate::recipe::focus_ring_declarations`]
//!   （`FocusRingColor::Token`、date-picker は palette 軸を持たないため）
//!   へ置換した。`trigger` はハードコードの `outline: 2px solid
//!   var(--fandhe-color-accent)` を、`input` は `border-color` のみの
//!   弱い表現をそれぞれ置き換えている
//! - **hover は `trigger`/`clear-trigger` にのみ付ける**: `control` は
//!   レイアウトのみのコンテナ（headless が `data-state` のみを出し、
//!   クリック操作を担わない）で hover 対象としない。`input` はテキスト
//!   入力面であり参照サイト（chakra-ui/ark-ui）もこの面自体への hover
//!   表現を持たないため対象外とする（combobox 1/2 の `control`/`input` と
//!   同じ判断軸）。`trigger`/`clear-trigger`（クリック操作を担う slot）
//!   にのみ [`crate::recipe::hover_bg_muted`]（`--fandhe-hover-bg` 定義）+
//!   `.state(slot, StateCondition::Hover,
//!   crate::recipe::hover_surface_declarations())`（実適用）を付ける
//! - **disabled 視覚は `input`/`trigger` のみに付ける**: headless
//!   （`crates/headless-ui/src/date_picker.rs`）が `data-disabled` を出すのは
//!   `input`/`trigger` のみで、`control`/`clear-trigger` へは出さないため
//!   本 CSS 側でも対象外とする（消費できない属性へ規則を書かない、
//!   combobox 1/2 と同じ判断）
//! - **`control` は変更しない**: border を持たない純レイアウトコンテナで
//!   あり、headless も `data-state` 以外の状態属性を出さないため、7 軸
//!   チェックリスト上で是正対象がない
//! - `clear-trigger`（`<button>`）にブラウザ既定のボタン装飾（border・
//!   背景）が露出していた実不具合を是正し、`trigger` と同じリセット
//!   （`display: inline-flex`/`align-items: center`/`justify-content:
//!   center`/`background: transparent`/`border: none`）+
//!   `border-radius: var(--fandhe-radius-sm)` + `hover_bg_muted()` を
//!   追加した（combobox 1/2 の `clear-trigger` 是正と同型）。headless が
//!   `clear-trigger` へ `data-disabled` を出さないため disabled 視覚は
//!   付けない
//! - **transition は純追加**: `input`/`trigger`/`clear-trigger` へ
//!   [`crate::recipe::transition_declarations`] を `base` の 2 個目登録
//!   （同一 slot への複数回 `.base()` 登録は出力順で連結される）として
//!   追加し、既存 `base` ブロックは書き換えない
//! - **variant 軸（chakra の `outline`/`subtle` 相当）の追加は見送る**:
//!   `root()` シグネチャ変更（破壊的）を伴い Forms 家族横断の軸判断で
//!   あるため、部品単独で先行しない（checkbox 1/2・combobox 1/2 と同じ
//!   判断軸）
//! - **size 連動の `font-size` 追加は見送る**: 同上の横断判断のため
//!   見送る

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

pub use fandhe_frontend_headless_ui::date_picker::{
    clear_trigger, content, control, input, label, positioner, trigger,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::state::OpenState;

/// headless `date_picker` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/date_picker.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "input",
    "trigger",
    "clear-trigger",
    "positioner",
    "content",
];

/// この styled DatePicker の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("date-picker", SLOTS)
        .base("root", vec![decl("position", "relative")])
        .base(
            "label",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "control",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "input",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl(
                    "padding",
                    "var(--fandhe-date-picker-input-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結されるため、
        // 上記 base ブロックを書き換えずに純追加する（combobox 1/2、
        // イシュー #1467 の transition 追加と同型のパターン）。
        .base(
            "input",
            transition_declarations("border-color, background", MotionDuration::Fast),
        )
        .base(
            "trigger",
            vec![
                decl("cursor", "pointer"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("padding", "var(--fandhe-space-2)"),
                hover_bg_muted(),
            ],
        )
        .base(
            "trigger",
            transition_declarations("border-color, background, color", MotionDuration::Fast),
        )
        .base(
            "clear-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                hover_bg_muted(),
            ],
        )
        .base(
            "clear-trigger",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .base(
            "positioner",
            vec![
                decl("position", "absolute"),
                decl("top", "100%"),
                decl("left", "0"),
                decl("z-index", "10"),
                decl("margin-top", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "0.375rem"),
                decl("box-shadow", "0 4px 6px rgba(0, 0, 0, 0.15)"),
                decl(
                    "padding",
                    "var(--fandhe-date-picker-content-padding, var(--fandhe-space-2))",
                ),
            ],
        )
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("border-color", "var(--fandhe-color-accent)")],
        )
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "input",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // headless（`crates/headless-ui/src/date_picker.rs`）が `input`/
        // `trigger` へ出す `data-disabled` を消費する（`control`/
        // `clear-trigger` へは出さないため対象外、モジュール rustdoc
        // 「スタイル調整」節参照）。
        .state(
            "input",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // trigger/clear-trigger の hover 実適用（`--fandhe-hover-bg` の間接
        // 参照経由、モジュール rustdoc「スタイル調整」節参照）。`control`/
        // `input` 自体には付けない。
        .state(
            "trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "clear-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-date-picker-input-padding", "var(--fandhe-space-0-5) var(--fandhe-space-1)"),
                decl("--fandhe-date-picker-content-padding", "var(--fandhe-space-0-5)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl(
                    "--fandhe-date-picker-input-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl(
                    "--fandhe-date-picker-content-padding",
                    "var(--fandhe-space-1)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl(
                    "--fandhe-date-picker-input-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl(
                    "--fandhe-date-picker-content-padding",
                    "var(--fandhe-space-2)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl(
                    "--fandhe-date-picker-input-padding",
                    "var(--fandhe-space-3) var(--fandhe-space-4)",
                ),
                decl(
                    "--fandhe-date-picker-content-padding",
                    "var(--fandhe-space-3)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-date-picker-input-padding", "var(--fandhe-space-4) var(--fandhe-space-5)"),
                decl("--fandhe-date-picker-content-padding", "var(--fandhe-space-4)"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled DatePicker が生成する静的 CSS 全量を返す（決定的。
/// [`crate::calendar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ。実体は [`fandhe_frontend_headless_ui::date_picker::root`] へ委譲する。
#[must_use]
pub fn root<'a>(
    size: Size,
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::date_picker::root(state, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="date-picker"][data-part="trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(Size::Md, OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="date-picker""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl] {
            let html = render(&root(
                size,
                OpenState::Closed,
                vec![("class", "attacker")],
                vec![],
            ));
            let expected_class = format!("fd-date-picker--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="date-picker"][data-part="trigger"][data-state="open"]"#)
        );
    }

    #[test]
    fn input_and_trigger_use_tokenized_radius_not_raw_literal() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="date-picker"][data-part="input"] {
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);"#
        ));
        assert!(css.contains(
            r#"[data-scope="date-picker"][data-part="trigger"] {
  cursor: pointer;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);"#
        ));
    }

    #[test]
    fn content_border_radius_is_untouched_raw_literal() {
        // root/label/positioner/content は分割 2/3・3/3（#1472/#1473）の
        // スコープであり、本イシューでは変更しない（意図的な非対称、
        // モジュール rustdoc「スタイル調整」節参照）。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="date-picker"][data-part="content"] {
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: 0.375rem;"#
        ));
    }

    #[test]
    fn input_and_trigger_expose_canonical_focus_ring() {
        let css = stylesheet();
        let ring = "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));";
        assert!(css.contains(ring), "css={css}");
        // ハードコードのアウトラインが残っていないこと。
        assert!(!css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }

    #[test]
    fn disabled_declarations_apply_to_input_and_trigger_only() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="date-picker"][data-part="input"][data-disabled]"#));
        assert!(css.contains(r#"[data-scope="date-picker"][data-part="trigger"][data-disabled]"#));
        assert!(!css.contains(r#"[data-scope="date-picker"][data-part="control"][data-disabled]"#));
        assert!(!css
            .contains(r#"[data-scope="date-picker"][data-part="clear-trigger"][data-disabled]"#));
        assert!(css.contains("opacity: 0.5"));
        assert!(css.contains("cursor: not-allowed"));
    }

    #[test]
    fn hover_rules_are_scoped_to_media_hover_query() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        let media_start = css
            .find("@media (hover: hover)")
            .expect("media block present");
        let media_block = &css[media_start..];
        assert!(media_block.contains(
            r#"[data-scope="date-picker"][data-part="trigger"]:hover:not([data-disabled])"#
        ));
        assert!(media_block.contains(
            r#"[data-scope="date-picker"][data-part="clear-trigger"]:hover:not([data-disabled])"#
        ));
    }

    #[test]
    fn clear_trigger_resets_native_button_chrome() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="date-picker"][data-part="clear-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: var(--fandhe-radius-sm);
  cursor: pointer;
  color: var(--fandhe-color-fg-muted);"#
        ));
    }
}

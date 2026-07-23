//! styled Switch（headless ラッパー第 3 弾、イシュー #682、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::switch`（イシュー #537/#595）の Root /
//! Control / Thumb / Label / HiddenInput 5 anatomy パーツと
//! [`fandhe_frontend_headless_ui::switch::Switch`] 状態機械をそのまま
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠は [`crate::dialog`]/[`crate::popover`]/[`crate::tooltip`] の rustdoc
//! と同じ方針に従う。
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
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - variant（size/palette）ごとのクラス切り替えは headless ラッパー第 1 弾
//!   （#551）と同じくスコープ外とする。
//! - `hidden-input` フォーカス時の `control` へのフォーカスリング反映は、
//!   [`crate::recipe::StateCondition`] が親子・兄弟関係を表す関係セレクタ
//!   （`:has()`・兄弟結合子）を持たず、headless 層も `data-focus-visible` を
//!   出力しないため本イシューでは対応しない（headless 層への
//!   `data-focus-visible` 追加とあわせた Issue 化を別途提案する）。
//! - [`crate::stylesheet::StyleSheet`] の
//!   `push_recipe_is_infallible_for_all_styled_components` テストへの
//!   popover/tooltip（#664）の登録漏れは #707 で解消済み。

use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};

pub use fandhe_frontend_headless_ui::switch::*;

/// headless `switch` anatomy の `data-part` 一覧（`crates/headless-ui/src/switch.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &["root", "control", "thumb", "label", "hidden-input"];

/// この styled Switch の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("switch", SLOTS)
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
                decl("width", "2.5rem"),
                decl("height", "1.4rem"),
                decl("border-radius", "999px"),
                decl("background", "var(--fandhe-color-border)"),
                decl("padding", "0 0.15rem"),
                decl("transition", "background 0.15s"),
            ],
        )
        .state(
            "control",
            StateCondition::AttrEq("data-state", "checked"),
            vec![decl("background", "var(--fandhe-color-accent)")],
        )
        .base(
            "thumb",
            vec![
                decl("width", "1.1rem"),
                decl("height", "1.1rem"),
                decl("border-radius", "999px"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("transition", "transform 0.15s"),
            ],
        )
        .state(
            "thumb",
            StateCondition::AttrEq("data-state", "checked"),
            vec![decl("transform", "translateX(1.1rem)")],
        )
        .base(
            "label",
            vec![decl("font-size", "var(--fandhe-font-font-size-sm)")],
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
}

/// この styled Switch が生成する静的 CSS 全量を返す（決定的。
/// [`crate::dialog::stylesheet`]/[`crate::tooltip::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
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
  background: var(--fandhe-color-accent);
}"#
        ));
        assert!(css.contains(
            r#"[data-scope="switch"][data-part="thumb"][data-state="checked"] {
  transform: translateX(1.1rem);
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
  width: 2.5rem;"#
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

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(false, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="root""#));
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
    fn ssr_and_hydration_round_trip_via_reexported_switch_state_machine() {
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

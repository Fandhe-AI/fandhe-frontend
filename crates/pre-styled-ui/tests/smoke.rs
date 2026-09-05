//! `fandhe-frontend-pre-styled-ui` のスモークテスト（イシュー #546、拡充 #550/#551）。
//!
//! 骨格段階（#546）では公開 API を持たなかったため `fandhe-frontend-headless-ui`
//! への path 依存の存在確認のみだったが、#550 で単純 styled 部品（Button/Badge/
//! Card/Alert/Spinner）、#551 で headless 5 コンポーネント（Dialog / Tabs /
//! Accordion / Menu / Select）のラッパーを実装したため、ここでは両方の公開 API
//! 経由の XSS 回帰（REQ-1）を集約する。#553（XSS 回帰テスト本格整備）の先行
//! アンカーであり、本ファイルの個別テストを削除・弱体化しない
//! （`.claude/rules/coding-rust.md` のテスト方針）。

use fandhe_frontend_core::{el, render, text};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_pre_styled_ui::alert::{self, AlertProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::{
    accordion, dialog, menu, popover, select, switch, tabs, toggle_tip, tooltip,
};
use fandhe_frontend_pre_styled_ui::{badge, button, spinner};
use fandhe_frontend_pre_styled_ui::{BadgeProps, ButtonProps, ColorPalette, Size, SpinnerProps};

/// XSS ペイロード（`<script>` タグ）。テキスト子ノード・属性値の両方へ
/// 使い回し、既定エスケープ（REQ-1）が両経路で効くことを固定する。
const XSS_PAYLOAD: &str = "<script>alert('xss')</script>";

#[test]
fn default_escape_holds_via_core_dev_dependency() {
    let node = el("div", vec![], vec![text(XSS_PAYLOAD)]);
    let html = render(&node);

    assert!(
        !html.contains("<script>"),
        "既定エスケープが効いていない: {html}"
    );
    assert!(html.contains("&lt;script&gt;"));
}

/// 各 styled 部品ラッパーについて (a) `data-scope`/`data-part` が保持される
/// こと、(b) children テキストのエスケープが効くこと、(c) 生成 CSS が
/// `</style>`/`<` を含まない安全な静的文字列であることを固定する。
mod wrapper_escape_and_stylesheet_safety {
    use super::*;

    #[test]
    fn dialog_close_trigger_children_are_escaped() {
        let html = render(&dialog::close_trigger(vec![], vec![text(XSS_PAYLOAD)]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains(r#"data-scope="dialog""#));
    }

    #[test]
    fn accordion_item_trigger_children_are_escaped() {
        let html = render(&accordion::item_trigger(
            OpenState::Closed,
            false,
            "panel-1",
            None,
            None,
            vec![],
            vec![text(XSS_PAYLOAD)],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains(r#"data-scope="accordion""#));
    }

    #[test]
    fn menu_item_children_are_escaped() {
        let html = render(&menu::item(
            "item-1",
            false,
            false,
            vec![],
            vec![text(XSS_PAYLOAD)],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains(r#"data-scope="menu""#));
    }

    #[test]
    fn select_item_text_children_are_escaped() {
        let html = render(&select::item_text(
            select::OpenState::Closed,
            &select::SelectProps::default(),
            false,
            false,
            None,
            vec![],
            vec![text(XSS_PAYLOAD)],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains(r#"data-scope="select""#));
    }

    #[test]
    fn tabs_trigger_label_is_escaped() {
        let props = tabs::TabsProps {
            id: "t1",
            selected: "one",
            orientation: Orientation::Horizontal,
            activation_mode: tabs::ActivationMode::Automatic,
            loop_focus: true,
            indicator: false,
        };
        let items = vec![tabs::TabItem {
            value: "one",
            trigger: vec![text(XSS_PAYLOAD)],
            content: vec![],
            disabled: false,
        }];
        let html = render(&tabs::tabs(Size::Md, ColorPalette::Accent, &props, items));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains(r#"data-scope="tabs""#));
    }

    #[test]
    fn popover_close_trigger_children_are_escaped() {
        // イシュー #664: styled Popover 経由でも既定エスケープ（REQ-1）が
        // 効くことを固定する（headless ラッパー第 2 弾）。
        let html = render(&popover::close_trigger(vec![], vec![text(XSS_PAYLOAD)]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains(r#"data-scope="popover""#));
    }

    #[test]
    fn tooltip_content_children_are_escaped() {
        // イシュー #664: styled Tooltip 経由でも既定エスケープ（REQ-1）が
        // 効くことを固定する（headless ラッパー第 2 弾）。
        let html = render(&tooltip::content(
            OpenState::Open,
            None,
            vec![],
            vec![text(XSS_PAYLOAD)],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains(r#"data-scope="tooltip""#));
    }

    #[test]
    fn all_five_stylesheets_are_free_of_style_breakout_sequences() {
        for css in [
            dialog::stylesheet(),
            tabs::stylesheet(),
            accordion::stylesheet(),
            menu::stylesheet(),
            select::stylesheet(),
        ] {
            assert!(!css.contains("</style"), "CSS breakout 発生: {css}");
            assert!(!css.contains('<'), "CSS に '<' が混入: {css}");
            assert!(!css.is_empty());
        }
    }

    #[test]
    fn all_five_stylesheets_are_deterministic_across_calls() {
        assert_eq!(dialog::stylesheet(), dialog::stylesheet());
        assert_eq!(tabs::stylesheet(), tabs::stylesheet());
        assert_eq!(accordion::stylesheet(), accordion::stylesheet());
        assert_eq!(menu::stylesheet(), menu::stylesheet());
        assert_eq!(select::stylesheet(), select::stylesheet());
    }

    #[test]
    fn popover_and_tooltip_stylesheets_are_free_of_style_breakout_sequences() {
        // イシュー #664: headless ラッパー第 2 弾（Popover/Tooltip）分を
        // 既存の 5 種とは独立に固定する（第 1 弾のテストは変更しない）。
        for css in [popover::stylesheet(), tooltip::stylesheet()] {
            assert!(!css.contains("</style"), "CSS breakout 発生: {css}");
            assert!(!css.contains('<'), "CSS に '<' が混入: {css}");
            assert!(!css.is_empty());
        }
    }

    #[test]
    fn popover_and_tooltip_stylesheets_are_deterministic_across_calls() {
        assert_eq!(popover::stylesheet(), popover::stylesheet());
        assert_eq!(tooltip::stylesheet(), tooltip::stylesheet());
    }

    #[test]
    fn toggle_tip_content_children_are_escaped() {
        // イシュー #761: styled ToggleTip 経由でも既定エスケープ（REQ-1）が
        // 効くことを固定する（[`tooltip_content_children_are_escaped`] と同型）。
        let html = render(&toggle_tip::content(
            OpenState::Open,
            None,
            vec![],
            vec![text(XSS_PAYLOAD)],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains(r#"data-scope="toggle-tip""#));
    }

    #[test]
    fn toggle_tip_stylesheet_is_free_of_style_breakout_sequences() {
        let css = toggle_tip::stylesheet();
        assert!(!css.contains("</style"), "CSS breakout 発生: {css}");
        assert!(!css.contains('<'), "CSS に '<' が混入: {css}");
        assert!(!css.is_empty());
    }

    #[test]
    fn toggle_tip_stylesheet_is_deterministic_across_calls() {
        assert_eq!(toggle_tip::stylesheet(), toggle_tip::stylesheet());
    }

    #[test]
    fn switch_label_children_are_escaped() {
        // イシュー #682: styled Switch 経由でも既定エスケープ（REQ-1）が
        // 効くことを固定する（headless ラッパー第 3 弾）。
        let html = render(&switch::label(
            false,
            &switch::SwitchProps::default(),
            vec![],
            vec![text(XSS_PAYLOAD)],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains(r#"data-scope="switch""#));
    }

    #[test]
    fn switch_stylesheet_is_free_of_style_breakout_sequences() {
        let css = switch::stylesheet();
        assert!(!css.contains("</style"), "CSS breakout 発生: {css}");
        assert!(!css.contains('<'), "CSS に '<' が混入: {css}");
        assert!(!css.is_empty());
    }

    #[test]
    fn switch_stylesheet_is_deterministic_across_calls() {
        assert_eq!(switch::stylesheet(), switch::stylesheet());
    }
}

/// XSS 回帰: Button の子ノード経由。
#[test]
fn button_children_xss_payload_is_escaped() {
    let node = button(
        &ButtonProps::default(),
        vec![],
        vec![text("<script>alert('xss')</script>")],
    );
    let html = render(&node);
    assert!(!html.contains("<script>"), "{html}");
    assert!(html.contains("&lt;script&gt;"));
}

/// XSS 回帰: Badge の子ノード経由。
#[test]
fn badge_children_xss_payload_is_escaped() {
    let node = badge(
        &BadgeProps::default(),
        vec![],
        vec![text("<script>alert('xss')</script>")],
    );
    let html = render(&node);
    assert!(!html.contains("<script>"), "{html}");
    assert!(html.contains("&lt;script&gt;"));
}

/// XSS 回帰: Spinner の `label` 属性値経由。
#[test]
fn spinner_label_attribute_xss_payload_is_escaped() {
    let node = spinner(&SpinnerProps {
        label: "\" onmouseover=\"alert(1)",
        ..SpinnerProps::default()
    });
    let html = render(&node);
    assert!(!html.contains("onmouseover=\"alert"), "{html}");
    assert!(html.contains("&quot;"));
}

/// XSS 回帰: Card の title/description 子ノード経由。
#[test]
fn card_title_and_description_xss_payload_is_escaped() {
    let node = card::root(
        CardProps::default(),
        vec![],
        vec![
            card::title(vec![], vec![text("<script>alert(1)</script>")]),
            card::description(vec![], vec![text("<img src=x onerror=alert(1)>")]),
        ],
    );
    let html = render(&node);
    assert!(!html.contains("<script>"), "{html}");
    assert!(!html.contains("<img src=x onerror"), "{html}");
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&lt;img"));
}

/// XSS 回帰: Alert の title/description 子ノード経由。
#[test]
fn alert_title_and_description_xss_payload_is_escaped() {
    let node = alert::root(
        &AlertProps::default(),
        vec![],
        vec![alert::content(
            vec![],
            vec![
                alert::title(vec![], vec![text("<script>alert(1)</script>")]),
                alert::description(vec![], vec![text("<script>alert(2)</script>")]),
            ],
        )],
    );
    let html = render(&node);
    assert!(!html.contains("<script>alert"), "{html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"));
}

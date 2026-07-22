//! `fandhe-frontend-pre-styled-ui` のスモークテスト（イシュー #546、拡充 #551）。
//!
//! 骨格段階（#546）では公開 API を持たなかったため `fandhe-frontend-headless-ui`
//! への path 依存の存在確認のみだったが、#551 で headless 5 コンポーネント
//! （Dialog / Tabs / Accordion / Menu / Select）のラッパーを実装したため、
//! ここではラッパー経由の XSS 回帰（REQ-1）を集約する。#553（XSS 回帰テスト
//! 本格整備）の先行アンカーであり、本ファイルの個別テストを削除・弱体化しない
//! （`.claude/rules/coding-rust.md` のテスト方針）。

use fandhe_frontend_core::{el, render, text};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_pre_styled_ui::{accordion, dialog, menu, select, tabs};

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
        let html = render(&select::item_text(None, vec![], vec![text(XSS_PAYLOAD)]));
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
        };
        let items = vec![tabs::TabItem {
            value: "one",
            trigger: vec![text(XSS_PAYLOAD)],
            content: vec![],
            disabled: false,
        }];
        let html = render(&tabs::tabs(&props, items));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains(r#"data-scope="tabs""#));
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
}

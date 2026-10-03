//! イシュー #3677（インライン `<style>` を外部 CSS へ移す）専用の契約テスト。
//!
//! `/themes/button-group/` の split-menu demo がインライン `<style>` を出さず、
//! トリガー用の ID スコープ規則が `showcase::stylesheet()`
//! （`assets/pre-styled-ui.css`）側にあることを固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::generated_content;
use fandhe_frontend_docs_site::showcase;

#[test]
fn button_group_page_has_no_inline_style_but_keeps_trigger_id() {
    let node = generated_content("/themes/button-group/").expect("button-group page");
    let html = render(&node);
    assert!(!html.contains("<style"));
    assert!(html.contains(r#"id="button-group-split-menu-trigger""#));
}

#[test]
fn showcase_stylesheet_carries_split_menu_trigger_rules() {
    let css = showcase::stylesheet()
        .expect("showcase stylesheet")
        .as_css()
        .to_string();
    for needle in [
        "#button-group-split-menu-trigger { background: var(--fandhe-color-accent); color: var(--fandhe-color-accent-fg); border: none; }",
        "#button-group-split-menu-trigger:hover { background: var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized)); }",
    ] {
        assert!(css.contains(needle), "missing {needle}");
    }
}

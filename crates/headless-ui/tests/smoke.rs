//! `fandhe-frontend-headless-ui` の骨格スモークテスト（イシュー #522 起源）。
//!
//! `fandhe-frontend-core` への path 依存が実体を持つこと・本クレート経由でも
//! 既定エスケープ（REQ-1）が効くことに加え、イシュー #523 で実装した
//! anatomy / data-* API がエンドツーエンドで期待通りの HTML を出力することを
//! 固定する。属性エスケープの XSS 回帰は `tests/helpers_escape.rs` を参照。

use fandhe_frontend_core::{el, render, text};
use fandhe_frontend_headless_ui::{anatomy, data_state};

#[test]
fn default_escape_holds_via_core_dependency() {
    let node = el("div", vec![], vec![text("<script>alert('xss')</script>")]);
    let html = render(&node);

    assert!(
        !html.contains("<script>"),
        "既定エスケープが効いていない: {html}"
    );
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn anatomy_part_with_data_state_renders_expected_html() {
    let accordion = anatomy("accordion");
    let node = accordion.part("item", "div", vec![data_state("open")], vec![text("panel")]);

    assert_eq!(
        render(&node),
        r#"<div data-scope="accordion" data-part="item" data-state="open">panel</div>"#
    );
}

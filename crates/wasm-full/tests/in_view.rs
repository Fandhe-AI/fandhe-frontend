//! `fandhe_frontend_wasm_full::in_view`（イシュー #2396、親 #2394）の
//! 純粋層（DOM 非依存）単体テスト。
//!
//! 実 DOM 経由の `IntersectionObserver` 配線検証は
//! `wasm-full/tests/in_view_browser.rs` が担当する。

use fandhe_frontend_wasm_full::in_view::{
    in_view_once_from_attr, IN_VIEW_ATTR, IN_VIEW_ONCE_ATTR, IN_VIEW_SELECTOR,
};

#[test]
fn in_view_once_from_attr_requires_strict_true() {
    assert!(!in_view_once_from_attr(None));
    assert!(!in_view_once_from_attr(Some("false")));
    assert!(!in_view_once_from_attr(Some("")));
    assert!(!in_view_once_from_attr(Some("TRUE")));
    assert!(!in_view_once_from_attr(Some("yes")));
    assert!(in_view_once_from_attr(Some("true")));
}

#[test]
fn attribute_and_selector_constants_are_stable() {
    // 他クレート（pre-styled-ui 側の将来的な CSS フック等）が本属性名で
    // ドリフトしないよう文字列値を固定する。
    assert_eq!(IN_VIEW_ATTR, "data-in-view");
    assert_eq!(IN_VIEW_ONCE_ATTR, "data-in-view-once");
    assert_eq!(IN_VIEW_SELECTOR, "[data-in-view]");
}

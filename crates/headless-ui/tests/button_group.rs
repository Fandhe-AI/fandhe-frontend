//! `fandhe-frontend-headless-ui` の Button Group（[`button_group`] モジュール、
//! イシュー #2059）の公開 API 経由の統合テスト。
//!
//! `crates/headless-ui/src/button_group.rs` 内の `#[cfg(test)]` ユニット
//! テストが内部実装を含めた網羅を担うのに対し、本ファイルは
//! `fandhe-frontend-headless-ui` の公開 API（`lib.rs` の再エクスポート
//! `pub mod button_group;`）のみを経由し、`fandhe-frontend-pre-styled-ui`
//! （styled ラッパー、#2060）が実際に使う想定の外部からの利用形態
//! （root + text + button 複数 + separator の組み合わせ、ネスト）を固定
//! する回帰テスト（`tests/toolbar.rs` と同型の位置付け）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::button_group;
use fandhe_frontend_headless_ui::Orientation;

/// root（horizontal）+ text + 素の `button` 要素 3 個 + separator を
/// 1 つのノード木へ組み合わせても anatomy が破綻せず、`role="group"`・
/// `aria-label`・`data-orientation="horizontal"`・separator の
/// `role="separator"`・`aria-orientation="vertical"` が出力され、
/// `role="group"` 側に `aria-orientation` が一切出現しないことを固定する。
#[test]
fn root_with_text_buttons_and_separator_has_expected_anatomy() {
    let node = button_group::root(
        Orientation::Horizontal,
        "File actions",
        vec![],
        vec![
            button_group::text(vec![], vec![text("Save as")]),
            fandhe_frontend_core::button(vec![("type", "button")], vec![text("Save")]),
            fandhe_frontend_core::button(vec![("type", "button")], vec![text("Save As")]),
            fandhe_frontend_core::button(vec![("type", "button")], vec![text("Export")]),
            button_group::separator(Orientation::Horizontal, vec![], vec![]),
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="button-group""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"role="group""#));
    assert!(html.contains(r#"aria-label="File actions""#));
    assert!(html.contains(r#"data-orientation="horizontal""#));
    assert!(html.contains(r#"data-part="text""#));
    assert!(html.contains(r#"data-part="separator""#));
    assert!(html.contains(r#"role="separator""#));
    assert!(html.contains(r#"aria-orientation="vertical""#));

    // `role="group"` に `aria-orientation` を付与しない不変条件
    // （モジュール doc「`role="group"` に `aria-orientation` を付与
    // しない」節、WAI-ARIA 上 group ロールへの aria-orientation は
    // 非許可）。root 部分のみを抽出して確認する。
    let root_start = html.find(r#"data-part="root""#).unwrap();
    let root_tag_end = html[root_start..].find('>').unwrap() + root_start;
    assert!(!html[root_start..root_tag_end].contains("aria-orientation"));

    // 状態機械を持たないため hydration 属性は一切出力されない。
    assert!(!html.contains("data-hydrate-"));
}

/// vertical グループの separator は水平（`horizontal`）になる直交規則を
/// 公開 API 経由でも固定する。
#[test]
fn vertical_group_separator_is_horizontal() {
    let node = button_group::root(
        Orientation::Vertical,
        "",
        vec![],
        vec![button_group::separator(
            Orientation::Vertical,
            vec![],
            vec![],
        )],
    );
    let html = render(&node);
    assert!(html.contains(r#"data-orientation="vertical""#));
    assert!(html.contains(r#"aria-orientation="horizontal""#));
}

/// ネスト（グループの中にグループ）で外側・内側それぞれの
/// `data-orientation` が独立して保持されることを公開 API 経由で固定する。
#[test]
fn nested_group_orientations_are_independent_via_public_api() {
    let inner = button_group::root(
        Orientation::Vertical,
        "Nested",
        vec![],
        vec![fandhe_frontend_core::button(
            vec![("type", "button")],
            vec![text("Inner")],
        )],
    );
    let outer = button_group::root(
        Orientation::Horizontal,
        "Outer",
        vec![],
        vec![
            fandhe_frontend_core::button(vec![("type", "button")], vec![text("Outer")]),
            inner,
        ],
    );
    let html = render(&outer);

    let outer_idx = html.find(r#"data-orientation="horizontal""#).unwrap();
    let inner_idx = html.find(r#"data-orientation="vertical""#).unwrap();
    assert!(outer_idx < inner_idx);
    // 2 つの root（`data-part="root"`）が両方出現する。
    assert_eq!(html.matches(r#"data-part="root""#).count(), 2);
}

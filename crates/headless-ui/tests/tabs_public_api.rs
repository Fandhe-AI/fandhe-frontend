//! `tabs`/`TabItem`/`TabsProps`（イシュー #528）の公開 API 経由統合テスト。
//!
//! クレートルートからの re-export が実際に使えることを確認したうえで、
//! 代表スナップショットと XSS 回帰を固定する。ユニットテスト（値ごとの
//! 詳細な属性検証）は `crates/headless-ui/src/tabs.rs` 側に置き、本ファイルは
//! 「公開 API 経由で壊れていないか」の統合確認に絞る。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::{tabs, TabItem, TabsProps};

#[test]
fn tabs_public_api_is_usable_from_crate_root_and_renders_expected_html() {
    let node = tabs(
        &TabsProps {
            id: "settings",
            selected: "profile",
            orientation: Orientation::Horizontal,
        },
        vec![
            TabItem {
                value: "profile",
                trigger: vec![text("Profile")],
                content: vec![text("Profile panel")],
                disabled: false,
            },
            TabItem {
                value: "billing",
                trigger: vec![text("Billing")],
                content: vec![text("Billing panel")],
                disabled: false,
            },
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="tabs" data-part="root""#));
    assert!(html.contains(r#"role="tablist""#));
    assert!(html.contains(r#"id="settings-trigger-profile""#));
    assert!(html.contains(r#"aria-controls="settings-content-profile""#));
    assert!(html.contains(r#"id="settings-content-profile""#));
    assert!(html.contains(r#"aria-labelledby="settings-trigger-profile""#));
    assert!(html.contains(r#"aria-selected="true""#));
    assert!(html.contains("Profile panel"));
    assert!(html.contains("Billing panel"));
}

// --- XSS 回帰: value・trigger/content 子ノードに攻撃者制御文字列が入っても
// 既定エスケープ（REQ-1）が効くことを、公開 API 経由で確認する。SSR 経路の
// XSS 回帰テスト網への追加であり削除・弱体化の対象にしない。
#[test]
fn tabs_public_api_escapes_xss_payload_in_value_and_children() {
    let payload_value = "x\" onmouseover=\"alert(1)";
    let node = tabs(
        &TabsProps {
            id: "t",
            selected: payload_value,
            orientation: Orientation::Horizontal,
        },
        vec![TabItem {
            value: payload_value,
            trigger: vec![text("<script>alert(1)</script>")],
            content: vec![text("<script>alert(2)</script>")],
            disabled: false,
        }],
    );
    let html = render(&node);

    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(!html.contains("<script>alert(2)</script>"));
    assert!(!html.contains("onmouseover=\"alert"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"));
    assert!(html.contains("&quot;"));
}

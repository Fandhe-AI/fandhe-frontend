//! `fandhe-frontend-pre-styled-ui` の骨格スモークテスト（イシュー #546）。
//!
//! `fandhe-frontend-headless-ui` への path 依存が実体を持つこと・
//! `fandhe-frontend-core`（dev-dependency）経由でも既定エスケープ（REQ-1）が
//! 効くことを固定する。イシュー #550 で単純 styled 部品（Button/Badge/Card/
//! Alert/Spinner）を実装したため、各部品の公開 API 経由でも既定エスケープが
//! 効くことを本ファイルで横断的に固定する（#553 の XSS 回帰テスト CI 整備の
//! 先行アンカー）。テーマ・variant API（#547/#548）に続く拡充。

use fandhe_frontend_core::{el, render, text};
use fandhe_frontend_pre_styled_ui::alert::{self, AlertStatus};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::{badge, button, spinner};
use fandhe_frontend_pre_styled_ui::{BadgeProps, ButtonProps, SpinnerProps};

#[test]
fn default_escape_holds_via_core_dev_dependency() {
    let node = el("div", vec![], vec![text("<script>alert('xss')</script>")]);
    let html = render(&node);

    assert!(
        !html.contains("<script>"),
        "既定エスケープが効いていない: {html}"
    );
    assert!(html.contains("&lt;script&gt;"));
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
        size: fandhe_frontend_pre_styled_ui::Size::Md,
        label: "\" onmouseover=\"alert(1)",
    });
    let html = render(&node);
    assert!(!html.contains("onmouseover=\"alert"), "{html}");
    assert!(html.contains("&quot;"));
}

/// XSS 回帰: Card の title/description 子ノード経由。
#[test]
fn card_title_and_description_xss_payload_is_escaped() {
    let node = card::root(
        CardVariant::default(),
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
        AlertStatus::default(),
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

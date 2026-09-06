//! `steps`（イシュー #752）を ark-ui/Zag.js（`steps` machine）・chakra-ui と
//! 突合した契約を fail-closed に固定する統合テスト（イシュー #1665、
//! `toolbar_reference_parity.rs`/`splitter_reference_parity.rs` と同型の
//! 立て付け）。
//!
//! # 突合結果（詳細は `crate::steps` モジュール doc「参照突合（イシュー
//! #1665）」節参照）
//!
//! 是正した差分（trigger/content/completed-content への `data-orientation`
//! 加算、prev/next-trigger への `data-disabled` 加算、`progress` パーツ新設、
//! 全パーツへの `drop_reserved` 導入）を固定しつつ、意図的に合わせなかった
//! 差分（trigger の `data-state` 語彙・tabs 意味論非採用・indicator の
//! `aria-hidden` 非採用・`dir`/`--percent` 非採用・`data-skippable` 非採用）
//! を回帰ガードする。
//!
//! 公開 API（`fandhe_frontend_headless_ui::{Steps, Orientation}`）のみを
//! 使い、`crate` 内部実装には依存しない。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::{Orientation, Steps};

/// `tag>` までを抜き出し開始タグのみを対象に属性を検査するヘルパ
/// （`toolbar_reference_parity.rs` と同型のパターン）。
fn tag_slice<'a>(html: &'a str, needle: &str) -> &'a str {
    let start = html.find(needle).unwrap_or_else(|| {
        panic!("needle {needle:?} not found in html: {html}");
    });
    let end = html[start..].find('>').unwrap() + start;
    &html[start..end]
}

// --- 是正した差分の固定 ---

#[test]
fn trigger_has_data_orientation_and_native_state_vocabulary() {
    let s = Steps::new(3, 1, Orientation::Vertical);
    let html = render(&s.trigger(1, vec![], vec![]));
    let tag = tag_slice(&html, r#"data-part="trigger""#);
    assert!(tag.contains(r#"data-orientation="vertical""#));
    // Zag.js は trigger に `data-state="open"|"closed"` を出すが、本実装は
    // `complete`/`current`/`incomplete` を維持する（golden CSS 破壊回避）。
    assert!(tag.contains(r#"data-state="current""#));
    assert!(!tag.contains(r#"data-state="open""#));
    assert!(!tag.contains(r#"data-state="closed""#));
}

#[test]
fn content_and_completed_content_have_data_orientation() {
    let s = Steps::new(3, 1, Orientation::Vertical);
    let content_html = render(&s.content(1, vec![], vec![]));
    assert!(tag_slice(&content_html, r#"data-part="content""#)
        .contains(r#"data-orientation="vertical""#));

    let completed_html = render(&s.completed_content(vec![], vec![]));
    assert!(
        tag_slice(&completed_html, r#"data-part="completed-content""#)
            .contains(r#"data-orientation="vertical""#)
    );
}

#[test]
fn prev_next_trigger_emit_native_and_data_disabled_at_bounds() {
    let at_start = Steps::new(3, 0, Orientation::Horizontal);
    let prev_html = render(&at_start.prev_trigger(vec![], vec![]));
    let prev_tag = tag_slice(&prev_html, r#"data-part="prev-trigger""#);
    assert!(prev_tag.contains("disabled"));
    assert!(prev_tag.contains("data-disabled"));

    let mid = Steps::new(3, 1, Orientation::Horizontal);
    let prev_mid_html = render(&mid.prev_trigger(vec![], vec![]));
    let prev_mid_tag = tag_slice(&prev_mid_html, r#"data-part="prev-trigger""#);
    assert!(!prev_mid_tag.contains("disabled"));

    let at_end = Steps::new(3, 3, Orientation::Horizontal);
    let next_html = render(&at_end.next_trigger(vec![], vec![]));
    let next_tag = tag_slice(&next_html, r#"data-part="next-trigger""#);
    assert!(next_tag.contains("disabled"));
    assert!(next_tag.contains("data-disabled"));
}

#[test]
fn progress_exposes_progressbar_semantics() {
    let s = Steps::new(4, 1, Orientation::Horizontal);
    let html = render(&s.progress(vec![], vec![]));
    assert!(html.contains(r#"data-scope="steps""#));
    let tag = tag_slice(&html, r#"data-part="progress""#);
    assert!(tag.contains(r#"role="progressbar""#));
    assert!(tag.contains(r#"aria-valuemin="0""#));
    assert!(tag.contains(r#"aria-valuemax="100""#));
    assert!(tag.contains(r#"aria-valuenow="25""#));
    assert!(tag.contains(r#"aria-valuetext="25% complete""#));

    let done = Steps::new(4, 4, Orientation::Horizontal);
    let done_html = render(&done.progress(vec![], vec![]));
    assert!(tag_slice(&done_html, r#"data-part="progress""#).contains("data-complete"));
}

#[test]
fn caller_cannot_spoof_reserved_attrs_via_drop_reserved() {
    let s = Steps::new(3, 1, Orientation::Horizontal);

    let root_html = render(&s.root(vec![("data-orientation", "attacker")], vec![]));
    let root_tag = tag_slice(&root_html, r#"data-part="root""#);
    assert_eq!(root_tag.matches("data-orientation").count(), 1);
    assert!(root_tag.contains(r#"data-orientation="horizontal""#));

    let trigger_html = render(&s.trigger(1, vec![("aria-current", "attacker")], vec![]));
    let trigger_tag = tag_slice(&trigger_html, r#"data-part="trigger""#);
    assert_eq!(trigger_tag.matches("aria-current").count(), 1);
    assert!(trigger_tag.contains(r#"aria-current="step""#));
}

// --- 非採用ガード（回帰防止） ---

#[test]
fn list_has_no_tablist_role_or_aria_orientation() {
    let s = Steps::new(3, 1, Orientation::Horizontal);
    let html = render(&s.list(vec![], vec![]));
    let tag = tag_slice(&html, r#"data-part="list""#);
    assert!(!tag.contains("role="));
    assert!(!tag.contains("aria-orientation"));
    assert!(!tag.contains("aria-owns"));
}

#[test]
fn trigger_has_no_tab_role_or_aria_selected_or_controls() {
    let s = Steps::new(3, 1, Orientation::Horizontal);
    let html = render(&s.trigger(1, vec![], vec![]));
    let tag = tag_slice(&html, r#"data-part="trigger""#);
    assert!(!tag.contains(r#"role="tab""#));
    assert!(!tag.contains("aria-selected"));
    assert!(!tag.contains("aria-controls"));
}

#[test]
fn content_has_no_tabpanel_role_or_aria_labelledby_or_tabindex() {
    let s = Steps::new(3, 1, Orientation::Horizontal);
    let html = render(&s.content(1, vec![], vec![]));
    let tag = tag_slice(&html, r#"data-part="content""#);
    assert!(!tag.contains(r#"role="tabpanel""#));
    assert!(!tag.contains("aria-labelledby"));
    assert!(!tag.contains("tabindex"));
}

#[test]
fn indicator_has_no_aria_hidden() {
    let s = Steps::new(3, 1, Orientation::Horizontal);
    let html = render(&s.indicator(1, vec![], vec![]));
    let tag = tag_slice(&html, r#"data-part="indicator""#);
    assert!(!tag.contains("aria-hidden"));
}

#[test]
fn root_has_no_percent_style_or_dir() {
    let s = Steps::new(3, 1, Orientation::Horizontal);
    let html = render(&s.root(vec![], vec![]));
    let tag = tag_slice(&html, r#"data-part="root""#);
    assert!(!tag.contains("style="));
    assert!(!tag.contains(" dir="));
}

#[test]
fn item_has_no_data_skippable() {
    let s = Steps::new(3, 1, Orientation::Horizontal);
    let html = render(&s.item(0, vec![], vec![]));
    let tag = tag_slice(&html, r#"data-part="item""#);
    assert!(!tag.contains("data-skippable"));
}

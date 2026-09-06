//! `splitter`（イシュー #826）を ark-ui docs / zag.js `splitter.connect.ts` /
//! WAI-ARIA APG Window Splitter パターンと突合した契約を fail-closed に
//! 固定する統合テスト（イシュー #1664、`toolbar_reference_parity.rs` と
//! 同型の立て付け）。Radix Primitives に Splitter 相当は存在しないため
//! 突合対象に含めない（`docs/design/component-coverage-map.md` 参照）。
//!
//! # 突合結果（詳細は `crate::splitter` モジュール doc「参照突合
//! （イシュー #1664）」節参照）
//!
//! - **是正**: `panel` へ `data-index`/`data-id` を追加、`resize_trigger` の
//!   `aria-controls`/`data-id` を隣接 2 パネル（先行/後続）へ拡張、
//!   `SplitterAction::IncrementLarge`/`DecrementLarge`（zag.js
//!   `keyboardResizeBy` 既定値 ×10 相当）を追加、`drop_reserved` による
//!   予約キーなりすまし除去を追加。
//! - **非追随**: `data-focus`/`data-dragging`（focus・pointer 由来の DOM
//!   ローカル状態）、resize-trigger-indicator への `data-orientation`
//!   追加、Enter（collapse/expand）・F6（フォーカス循環）、`dir`。
//! - **APG 準拠を維持**: `aria-orientation` はセパレータ自体の向き
//!   （パネルレイアウトと逆）を出力し続ける（zag.js の非反転出力とは非同値
//!   のまま）。disabled 時の `tabindex="-1"` + `aria-disabled` も
//!   リポジトリ横断規約の superset として維持する。
//!
//! 公開 API（`fandhe_frontend_headless_ui::{splitter, Orientation}`）のみを
//! 使い、`crate` 内部実装には依存しない。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::splitter;
use fandhe_frontend_headless_ui::Orientation;

/// `tag>` までを抜き出し開始タグのみを対象に属性を検査するヘルパ
/// （`toolbar_reference_parity.rs` と同型のパターン）。
fn tag_slice<'a>(html: &'a str, needle: &str) -> &'a str {
    let start = html.find(needle).unwrap_or_else(|| {
        panic!("needle {needle:?} not found in html: {html}");
    });
    let end = html[start..].find('>').unwrap() + start;
    &html[start..end]
}

/// ark-ui docs の `data-*` 表: root は `data-orientation` のみ持つ。
#[test]
fn root_has_data_orientation_only() {
    let html = render(&splitter::root(
        Orientation::Horizontal,
        false,
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="root""#);
    assert!(tag.contains(r#"data-orientation="horizontal""#));
    assert!(!tag.contains("data-dragging"));
    assert!(!tag.contains(" dir="));
}

/// ark-ui docs の `data-*` 表: panel は `data-orientation`/`data-id`/
/// `data-index` を持つ（`data-index`/`data-id` はイシュー #1664 で追加）。
#[test]
fn panel_has_data_orientation_data_index_and_data_id() {
    let html = render(&splitter::panel(
        "panel-a",
        0,
        Orientation::Vertical,
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="panel""#);
    assert!(tag.contains(r#"data-orientation="vertical""#));
    assert!(tag.contains(r#"data-index="0""#));
    assert!(tag.contains(r#"data-id="panel-a""#));
    assert!(!tag.contains("data-focus"));
    assert!(!tag.contains("data-dragging"));
}

/// ark-ui docs の `data-*` 表: resize-trigger は `data-id`/`data-orientation`/
/// `data-disabled` を持つ。`data-id` は `"<leading>:<trailing>"` 形式
/// （イシュー #1664 で追加）。
#[test]
fn resize_trigger_has_data_id_orientation_and_disabled() {
    let html = render(&splitter::resize_trigger(
        Orientation::Horizontal,
        "0",
        "100",
        "50",
        "panel-a",
        "panel-b",
        true,
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="resize-trigger""#);
    assert!(tag.contains(r#"data-id="panel-a:panel-b""#));
    assert!(tag.contains(r#"data-orientation="horizontal""#));
    assert!(tag.contains(r#"data-disabled="""#));
    assert!(!tag.contains("data-focus"));
    assert!(!tag.contains("data-dragging"));
}

/// zag.js に合わせ `aria-controls` を隣接 2 パネルの id（空白区切り）へ
/// 拡張した（イシュー #1664、`controls: &str` 単一引数からの破壊的変更）。
#[test]
fn resize_trigger_aria_controls_lists_both_adjacent_panels() {
    let html = render(&splitter::resize_trigger(
        Orientation::Horizontal,
        "0",
        "100",
        "50",
        "panel-a",
        "panel-b",
        false,
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="resize-trigger""#);
    assert!(tag.contains(r#"aria-controls="panel-a panel-b""#));
}

/// WAI-ARIA APG Window Splitter パターンに従い、`aria-orientation` は
/// セパレータ自体の向き（パネルレイアウトと逆）を維持する（zag.js の実出力
/// とは非同値のまま、モジュール doc「`aria-orientation` の向き」節参照）。
#[test]
fn resize_trigger_aria_orientation_stays_reversed_from_layout() {
    let horizontal_layout = render(&splitter::resize_trigger(
        Orientation::Horizontal,
        "0",
        "100",
        "50",
        "panel-a",
        "panel-b",
        false,
        vec![],
        vec![],
    ));
    let tag = tag_slice(&horizontal_layout, r#"data-part="resize-trigger""#);
    assert!(tag.contains(r#"aria-orientation="vertical""#));
    assert!(tag.contains(r#"data-orientation="horizontal""#));
}

/// disabled 時は `tabindex="-1"` + `aria-disabled="true"` を維持する
/// （zag.js は disabled 時にこれらを出力しないが、本実装はリポジトリ横断
/// 規約の superset として意図的に維持する、非採用ガード）。
#[test]
fn resize_trigger_disabled_keeps_tabindex_and_aria_disabled_superset() {
    let html = render(&splitter::resize_trigger(
        Orientation::Horizontal,
        "0",
        "100",
        "50",
        "panel-a",
        "panel-b",
        true,
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="resize-trigger""#);
    assert!(tag.contains(r#"tabindex="-1""#));
    assert!(tag.contains(r#"aria-disabled="true""#));
}

/// ark-ui docs の Anatomy には resize-trigger-indicator の `data-*` 行が
/// 無いため、`data-orientation` を追加しない（`fandhe-frontend-pre-styled-ui`
/// の recipe が「indicator は `data-orientation` を受け取らない」前提で
/// 組まれている既存設計を維持する、非採用ガード）。
#[test]
fn resize_trigger_indicator_has_no_data_orientation() {
    let html = render(&splitter::resize_trigger_indicator(vec![], vec![]));
    let tag = tag_slice(&html, r#"data-part="resize-trigger-indicator""#);
    assert!(!tag.contains("data-orientation"));
}

/// Enter（collapse/expand トグル）・F6（トリガー間フォーカス循環）は
/// `Splitter` の dispatch 語彙に存在しない（意図的非採用、モジュール doc
/// 「スコープ外」節参照）。
#[test]
fn enter_and_focus_cycle_dispatch_names_are_unknown() {
    use fandhe_frontend_headless_ui::splitter::Splitter;
    use fandhe_frontend_interactive::Component;

    assert!(Splitter::decode_action("enter", "0").is_none());
    assert!(Splitter::decode_action("focus_cycle", "0").is_none());
    assert!(Splitter::decode_action("f6", "0").is_none());
}

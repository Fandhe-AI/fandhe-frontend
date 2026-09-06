//! `toolbar`（イシュー #991）を Radix Primitives `Toolbar` と突合した契約を
//! fail-closed に固定する統合テスト（イシュー #1657、`tabs_reference_parity.rs`
//! と同型の立て付け）。
//!
//! # 突合結果（詳細は #1657 コメント・`crate::toolbar` モジュール doc
//! 「参照突合（イシュー #1657）」節参照）
//!
//! Radix の `data-*` 表（root: `data-orientation` / button: `data-orientation`
//! / separator: `data-orientation`〔toolbar 本体と直交〕/ toggle-group:
//! `data-orientation` / toggle-item: `data-state="on"|"off"`, `data-disabled`,
//! `data-orientation`）と一致するよう `data-orientation` の欠落 5 箇所を
//! `crates/headless-ui/src/toolbar.rs` へ追加した（自由関数の先頭引数へ
//! `orientation: Orientation` を追加する破壊的変更）。`link` は Radix の
//! `data-*` 表には載らないが、実 DOM（RovingFocusGroup.Item 経由）では
//! 出力される値であるため `button`/`toggle_item` との対称性を優先した
//! superset として追加する。
//!
//! 一方で Radix Button が native `disabled` を透過してフォーカス順序から
//! 除外する挙動、`dir`、`asChild`、`loop` 既定値 `true` は意図的に非採用の
//! ままとする（本ファイルの非採用ガードで回帰を防ぐ）。
//!
//! 公開 API（`fandhe_frontend_headless_ui::{toolbar, Orientation}`）のみを
//! 使い、`crate` 内部実装には依存しない（`toolbar.rs`（統合テスト）と同じ
//! 立て付け）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::toolbar;
use fandhe_frontend_headless_ui::Orientation;

/// `tag>` までを抜き出し開始タグのみを対象に属性を検査するヘルパ
/// （`tabs_reference_parity.rs` と同型のパターン）。
fn tag_slice<'a>(html: &'a str, needle: &str) -> &'a str {
    let start = html.find(needle).unwrap_or_else(|| {
        panic!("needle {needle:?} not found in html: {html}");
    });
    let end = html[start..].find('>').unwrap() + start;
    &html[start..end]
}

/// Radix Primitives の root data-* 表: `data-orientation` を持つ。
#[test]
fn root_has_data_orientation() {
    let html = render(&toolbar::root(
        Orientation::Horizontal,
        "Text formatting",
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="root""#);
    assert!(tag.contains(r#"data-orientation="horizontal""#));
    assert!(tag.contains(r#"role="toolbar""#));
    assert!(!tag.contains(" dir=")); // Radix の dir は非採用（本リポジトリ横断方針）
}

/// Radix Primitives の button data-* 表: `data-orientation` を持つ。native
/// `disabled` は付与しない（disabled もフォーカス可能にする本実装の意図的
/// 差分、モジュール doc「参照突合」節参照）。
#[test]
fn button_has_data_orientation_and_no_native_disabled() {
    let html = render(&toolbar::button(
        Orientation::Vertical,
        false,
        true,
        vec![],
        vec![text("Bold")],
    ));
    let tag = tag_slice(&html, r#"data-part="button""#);
    assert!(tag.contains(r#"data-orientation="vertical""#));
    assert!(tag.contains(r#"aria-disabled="true""#));
    // 空値のネイティブ boolean 属性そのものの不在を確認する
    // （`aria-disabled="true"` の部分文字列一致を避けるための厳密比較）。
    assert!(!tag.contains(r#" disabled="""#));
    // disabled でもフォーカス順序から除外しない（tabindex を維持する）。
    assert!(tag.contains(r#"tabindex="-1""#));
}

/// link は Radix の `data-*` 表には載らないが、実 DOM 準拠の superset として
/// `data-orientation` を持つ（モジュール doc「参照突合」節参照）。
#[test]
fn link_has_data_orientation_as_superset() {
    let html = render(&toolbar::link(
        Orientation::Horizontal,
        true,
        "/docs",
        false,
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="link""#);
    assert!(tag.contains(r#"data-orientation="horizontal""#));
    assert!(!tag.contains("data-state"));
}

/// Radix Primitives の separator data-* 表: `data-orientation` は toolbar
/// 本体と直交した値を持つ（横向き toolbar のセパレータは縦線になるため
/// `vertical`）。
#[test]
fn separator_has_orthogonal_data_orientation_and_role() {
    let html = render(&toolbar::separator(Orientation::Horizontal, vec![], vec![]));
    let tag = tag_slice(&html, r#"data-part="separator""#);
    assert!(tag.contains(r#"role="separator""#));
    assert!(tag.contains(r#"aria-orientation="vertical""#));
    assert!(tag.contains(r#"data-orientation="vertical""#));
}

/// Radix Primitives の toggle-group data-* 表: `data-orientation` を持つ。
/// `role="group"` には `aria-orientation` が許可されないため付与しない
/// （既存判断、`crate::toggle_group::root` PR #791 と同じ）。
#[test]
fn toggle_group_has_data_orientation_without_aria_orientation() {
    let html = render(&toolbar::toggle_group(
        Orientation::Vertical,
        vec![],
        vec![],
    ));
    let tag = tag_slice(&html, r#"data-part="toggle-group""#);
    assert!(tag.contains(r#"role="group""#));
    assert!(tag.contains(r#"data-orientation="vertical""#));
    assert!(!tag.contains("aria-orientation"));
}

/// Radix Primitives の toggle-item data-* 表: `data-state="on"|"off"` +
/// `data-disabled` + `data-orientation` の 3 点を持つ。
#[test]
fn toggle_item_has_full_radix_data_vocabulary() {
    let pressed_html = render(&toolbar::toggle_item(
        Orientation::Horizontal,
        true,
        true,
        false,
        "bold",
        vec![],
        vec![],
    ));
    let pressed_tag = tag_slice(&pressed_html, r#"data-part="toggle-item""#);
    assert!(pressed_tag.contains(r#"data-state="on""#));
    assert!(pressed_tag.contains(r#"data-orientation="horizontal""#));
    assert!(!pressed_tag.contains("data-disabled"));

    let disabled_html = render(&toolbar::toggle_item(
        Orientation::Horizontal,
        false,
        false,
        true,
        "italic",
        vec![],
        vec![],
    ));
    let disabled_tag = tag_slice(&disabled_html, r#"data-part="toggle-item""#);
    assert!(disabled_tag.contains(r#"data-state="off""#));
    assert!(disabled_tag.contains("data-disabled"));
    assert!(disabled_tag.contains(r#"data-orientation="horizontal""#));
}

/// 非採用ガード: `dir` 属性は本リポジトリ横断方針により全パーツで
/// 出力されない（黙って再導入されないための回帰ガード）。
#[test]
fn dir_attribute_is_not_adopted_anywhere() {
    let root_html = render(&toolbar::root(
        Orientation::Horizontal,
        "Toolbar",
        vec![],
        vec![],
    ));
    assert!(!root_html.contains(" dir="));

    let button_html = render(&toolbar::button(
        Orientation::Horizontal,
        true,
        false,
        vec![],
        vec![],
    ));
    assert!(!button_html.contains(" dir="));
}

/// XSS 回帰（REQ-1）: `value`/`href` への攻撃者制御文字列が属性値として
/// エスケープされることを本テストでも確認する
/// （`toolbar.rs`（ユニットテスト）と重複しない範囲の 1 件）。
#[test]
fn toggle_item_value_containing_html_and_quotes_is_escaped() {
    let payload = "x\"><script>alert(1)</script>";
    let html = render(&toolbar::toggle_item(
        Orientation::Horizontal,
        false,
        false,
        false,
        payload,
        vec![],
        vec![],
    ));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(!html.contains("x\">"));
    assert!(html.contains("&quot;"));
    assert!(html.contains("&lt;script&gt;"));
}

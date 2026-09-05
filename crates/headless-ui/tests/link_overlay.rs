//! `link_overlay::{root, overlay}`（イシュー #756）の公開 API 経由統合
//! テスト。
//!
//! `crates/headless-ui/src/link_overlay.rs` 側のユニットテストが値ごとの
//! 詳細な属性検証を行っているのに対し、本ファイルは参考サイトとの突合
//! 契約（イシュー #1650）に絞る。参照実体は chakra-ui の `LinkBox`/
//! `LinkOverlay` のみである（ark-ui の `link-overlay` ページは 404 で
//! 実在せず、Radix Primitives/Radix Themes にも対応部品がない。
//! `docs/design/component-coverage-map.md` 参照）。chakra-ui は
//! Anatomy 節・Keyboard Interactions 節・`data-*` 語彙・独自 ARIA 付与の
//! いずれも持たない styled 部品であり、本実装の `root`/`overlay` 2 パーツ・
//! `data-scope`/`data-part` は参照側に概念自体が存在しない superset で
//! ある。以下はこの一致点・意図的差分の双方を fail-closed に固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::link_overlay::{overlay, root};

/// anatomy は `root`（`div`）/ `overlay`（`a`）の 2 パーツ構成であり、
/// `data-part=` の出現がそれぞれ 1 回に限られることを固定する（参照側に
/// Anatomy 節が無くパート分割の概念自体が存在しないことに対応する
/// 最小構成であり、過不足がないことの確認）。
#[test]
fn reference_anatomy_is_root_div_and_overlay_anchor() {
    let root_html = render(&root(vec![], vec![]));
    assert!(root_html.starts_with("<div"), "root は div: {root_html}");
    assert!(root_html.contains(r#"data-scope="link-overlay""#));
    assert!(root_html.contains(r#"data-part="root""#));
    assert_eq!(root_html.matches("data-part=").count(), 1);

    let overlay_html = render(&overlay(
        "https://example.com/articles/getting-started",
        vec![],
        vec![text("Getting started")],
    ));
    assert!(
        overlay_html.starts_with("<a"),
        "overlay は a: {overlay_html}"
    );
    assert!(overlay_html.contains(r#"data-scope="link-overlay""#));
    assert!(overlay_html.contains(r#"data-part="overlay""#));
    assert_eq!(overlay_html.matches("data-part=").count(), 1);
}

/// 参照側（chakra-ui LinkBox/LinkOverlay）は状態を表す `data-*` を一切
/// 持たない。本実装も既定状態では `data-scope`/`data-part` 以外の
/// `data-*`（状態系）を一切出力しないことを固定する（`docs/policy/
/// intentional-non-adoption.md` §3.25 規則 2: 装飾・アニメーション関心を
/// headless へ持ち込まない不変条件）。
#[test]
fn no_state_data_attributes_by_default() {
    for html in [
        render(&root(vec![], vec![])),
        render(&overlay("https://example.com/docs", vec![], vec![])),
    ] {
        for forbidden in [
            "data-state",
            "data-disabled",
            "data-invalid",
            "data-orientation",
            "data-motion",
            "data-current",
        ] {
            assert!(
                !html.contains(forbidden),
                "{forbidden} を含むべきでない: {html}"
            );
        }
        assert_eq!(
            html.matches("data-").count(),
            2,
            "data-scope/data-part 以外の data-* を出力しない: {html}"
        );
    }
}

/// 参照側は `role`/`aria-*` を独自付与せず、ネイティブ `a` の暗黙
/// `link` ロールに委ねる。本実装も既定状態では `role=`/`aria-` を一切
/// 出力しないことを固定する。
#[test]
fn no_role_or_aria_by_default() {
    let html = render(&overlay("https://example.com/docs", vec![], vec![]));
    assert!(!html.contains("role="), "{html}");
    assert!(!html.contains("aria-"), "{html}");
}

/// 呼び出し側 `attrs`（`id`/`aria-label`/`class` 等の自前 CSS フック用
/// 属性を含む）がそのまま透過することを固定する（headless 契約により
/// スタイルレスのまま受け口のみを提供する）。
#[test]
fn caller_attrs_pass_through() {
    let root_html = render(&root(vec![("class", "x")], vec![]));
    assert!(root_html.contains(r#"class="x""#));

    let overlay_html = render(&overlay(
        "https://example.com/docs",
        vec![("id", "y"), ("aria-label", "Read more")],
        vec![],
    ));
    assert!(overlay_html.contains(r#"id="y""#));
    assert!(overlay_html.contains(r#"aria-label="Read more""#));
}

/// 呼び出し側 `attrs` に `data-scope`/`data-part` の偽装値を混入させても
/// anatomy 側の正規値で上書きされ fail-closed に除去されることを固定する
/// （`tests/link.rs` の同型テストと同じ不変条件）。
#[test]
fn caller_cannot_spoof_scope_or_part() {
    let html = render(&root(
        vec![("data-scope", "attacker"), ("data-part", "attacker")],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="link-overlay""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(!html.contains("attacker"));
}

/// [`overlay`] が固定付与する `href` は呼び出し側 `attrs` からのなりすまし
/// （大文字小文字無視）を除去し、正規値のみが 1 回出力されることを固定
/// する（`fandhe_frontend_core::el` は同名属性を重複除去しないため
/// `drop_reserved` 側の保証、イシュー #1650）。
#[test]
fn caller_supplied_href_reserved_key_is_dropped() {
    let html = render(&overlay(
        "https://example.com/docs",
        vec![("HREF", "https://example.com/evil")],
        vec![],
    ));
    assert_eq!(html.matches("href=").count(), 1, "{html}");
    assert!(html.contains(r#"href="https://example.com/docs""#));
    assert!(!html.contains("evil"));
}

/// 危険な URL スキームでは `href` 属性ごと出力されない（既定エスケープ
/// 経路の fail-closed 拒否）。参照側はネイティブ `a` そのままで同種の
/// 保証を持たないため、本実装の superset な安全性として固定する。
#[test]
fn dangerous_href_scheme_drops_href_attribute() {
    let html = render(&overlay("javascript:alert(1)", vec![], vec![]));
    assert!(
        !html.contains("href="),
        "危険な URL スキームなのに href 属性が出力されている: {html}"
    );
}

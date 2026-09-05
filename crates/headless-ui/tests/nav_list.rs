//! `nav_list`（イシュー #756）の公開 API 経由統合テスト。
//!
//! `crates/headless-ui/src/nav_list.rs` 側のユニットテストが値ごとの詳細な
//! 属性検証を行っているのに対し、本ファイルは参考サイトとの突合契約
//! （イシュー #1653）に絞る。`nav_list` は ark-ui / Radix Primitives /
//! Radix Themes に 1:1 対応物を持たない fandhe 独自部品（#756、
//! `docs/design/component-coverage-map.md:837`）であり、イシューが指す
//! chakra-ui `List`（`.agents/skills/chakra-ui/references/components/
//! typography/list.md`）は `variant`/`align`/`colorPalette`/`asChild`/
//! `List.Indicator` を持つ汎用の marker 付きリストで、Anatomy 図・
//! Keyboard Interactions 表・独自 ARIA を持たない（真の対応物は Themes 層
//! `fandhe-frontend-pre-styled-ui::list`、#771）。以下はこの参照側との
//! 一致点（状態 `data-*` 非付与・独自 ARIA 非付与）と、本実装が文書ナビ
//! 固有の superset として持つ 5 パーツ anatomy・`data-current` 語彙・
//! 予約キーなりすまし防止を fail-closed に固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::nav_list::{heading, item, link, list, root};

/// anatomy は `root`（`nav`）/ `heading`（`h2`）/ `list`（`ul`）/
/// `item`（`li`）/ `link`（`a`）の 5 パーツ構成であることを固定する
/// （参照側〔chakra `List`〕に 1:1 の Anatomy 図はないが、`List.Root`/
/// `List.Item` に相当する `list`/`item` へ文書ナビ固有の `root`/
/// `heading`/`link` を加えた superset であることの確認）。
#[test]
fn reference_anatomy_has_five_parts_on_expected_tags() {
    let html = render(&root(
        "Documentation",
        vec![],
        vec![
            heading(vec![], vec![text("Guides")]),
            list(
                vec![],
                vec![item(
                    vec![],
                    vec![link(
                        "/docs/intro",
                        false,
                        vec![],
                        vec![text("Introduction")],
                    )],
                )],
            ),
        ],
    ));
    assert!(html.starts_with("<nav"), "root は nav 要素で始まる: {html}");
    assert!(html.contains("<h2"));
    assert!(html.contains("<ul"));
    assert!(html.contains("<li"));
    assert!(html.contains("<a"));
    assert_eq!(
        html.matches("data-part=").count(),
        5,
        "nav-list の anatomy は 5 パーツ構成: {html}"
    );
    for part in ["root", "heading", "list", "item", "link"] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "{part} パーツが欠落: {html}"
        );
    }
}

/// 全パーツで `role` を一切付与しないことを固定する（素の `nav`/`h2`/
/// `ul`/`li`/`a` の暗黙 ARIA ロールに委ねる本部品の存在理由そのもの、
/// `crate::menu` の `role="menu"` との誤読回避）。
#[test]
fn no_role_is_ever_output() {
    let html = render(&root(
        "Documentation",
        vec![],
        vec![
            heading(vec![], vec![]),
            list(
                vec![],
                vec![item(
                    vec![],
                    vec![link("/docs/intro", true, vec![], vec![])],
                )],
            ),
        ],
    ));
    assert!(!html.contains("role="), "{html}");
}

/// 参照側（chakra `List`）は状態を表す `data-*` を一切持たない。本実装も
/// 既定状態（`current=false`）では `data-current` を含め状態系 `data-*`
/// を一切出力せず、`data-scope`/`data-part` のみであることを固定する
/// （`docs/policy/intentional-non-adoption.md` §3.25 規則 2 の不変条件）。
#[test]
fn no_state_data_attributes_by_default() {
    let html = render(&link("/docs/intro", false, vec![], vec![]));
    for forbidden in [
        "data-state",
        "data-disabled",
        "data-orientation",
        "data-motion",
        "data-active",
        "data-selected",
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

/// `current=true` のとき `aria-current="page"` と `data-current` が
/// 同時に出力されることを固定する（`crate::link`/`crate::breadcrumb` と
/// 共有する語彙、片方のみが出る経路がないことの保証）。
#[test]
fn current_adds_aria_current_page_and_data_current_together() {
    let html = render(&link("/docs/intro", true, vec![], vec![]));
    assert!(html.contains(r#"aria-current="page""#));
    assert!(html.contains("data-current"));
}

/// 危険な URL スキームでは `href` 属性ごと出力されない（既定エスケープ
/// 経路の fail-closed 拒否）。属性ごと欠落するため暗黙 `link` ロールも
/// 同時に失われる。
#[test]
fn dangerous_href_scheme_drops_href_attribute() {
    let html = render(&link("javascript:alert(1)", false, vec![], vec![]));
    assert!(
        !html.contains("href="),
        "危険な URL スキームなのに href 属性が出力されている: {html}"
    );
}

/// 呼び出し側 `attrs` に予約キー（`aria-label`/`href`/`aria-current`/
/// `data-current`）を混入させてもなりすましが成立しないことを固定する
/// （イシュー #1653 で追加した `drop_reserved`、`crate::breadcrumb` と
/// 同型の不変条件）。
#[test]
fn caller_cannot_spoof_reserved_attributes() {
    let root_html = render(&root(
        "Documentation",
        vec![("aria-label", "attacker")],
        vec![],
    ));
    assert_eq!(root_html.matches("aria-label").count(), 1);
    assert!(!root_html.contains("attacker"));

    let link_html = render(&link(
        "/docs/intro",
        false,
        vec![
            ("href", "javascript:alert(1)"),
            ("aria-current", "page"),
            ("data-current", ""),
        ],
        vec![],
    ));
    assert_eq!(link_html.matches("href=").count(), 1);
    assert!(link_html.contains(r#"href="/docs/intro""#));
    assert!(!link_html.contains("javascript:"));
    assert!(!link_html.contains("aria-current"));
    assert!(!link_html.contains("data-current"));
}

/// 呼び出し側 `attrs` で任意の属性（自前 CSS のフック用の `class` 等、
/// 予約キー以外）がそのまま透過することを固定する。
#[test]
fn non_reserved_caller_attrs_pass_through() {
    let html = render(&link(
        "/docs/intro",
        false,
        vec![("class", "x"), ("id", "y"), ("target", "_self")],
        vec![],
    ));
    assert!(html.contains(r#"class="x""#));
    assert!(html.contains(r#"id="y""#));
    assert!(html.contains(r#"target="_self""#));
}

/// 呼び出し側 `attrs` に `data-scope`/`data-part` の偽装値を混入させても
/// anatomy 側の正規値で上書きされ fail-closed に除去されることを固定する
/// （`tests/link.rs` の同型テストと同じ不変条件）。
#[test]
fn caller_cannot_spoof_scope_or_part() {
    let html = render(&root(
        "Documentation",
        vec![("data-scope", "attacker"), ("data-part", "attacker")],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="nav-list""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(!html.contains("attacker"));
}

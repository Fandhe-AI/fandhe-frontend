//! SkipNav（イシュー #776、参考サイト突合はイシュー #1663）の統合テスト。
//!
//! `crates/headless-ui/src/skip_nav.rs` の inline unit tests がパーツ単体の
//! 属性出力・予約キー除去・XSS 回帰を固定するのに対し、本ファイルは
//! 「link + content」の 2 パーツ組み立て全体を公開 API のみを使って外部から
//! 固定し、参考サイト（chakra-ui v3。Ark UI は該当ページ 404、Radix
//! Primitives / Radix Themes には該当部品なし）と突合した契約（anatomy の
//! 完全一致・`data-*`/`role`/`aria-*` の非付与範囲・契約属性のなりすまし
//! 防止・`href` のフラグメント限定）を回帰として保護する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::skip_nav::{content, link, DEFAULT_ID};

/// link と content を並べて組み立てたときの `data-scope`/`data-part` 出力と、
/// `href`/`id` の対応関係を固定する（chakra-ui の `SkipNavLink`/
/// `SkipNavContent` と anatomy が完全一致、イシュー #1663 突合結果）。
#[test]
fn full_assembly_pairs_link_and_content() {
    let link_html = render(&link(DEFAULT_ID, vec![], vec![text("Skip to content")]));
    let content_html = render(&content(DEFAULT_ID, vec![], vec![text("main content")]));

    assert!(link_html.contains(r#"data-scope="skip-nav""#));
    assert!(link_html.contains(r#"data-part="link""#));
    assert!(link_html.contains(r##"href="#fandhe-skip-nav""##));

    assert!(content_html.contains(r#"data-scope="skip-nav""#));
    assert!(content_html.contains(r#"data-part="content""#));
    assert!(content_html.contains(r#"id="fandhe-skip-nav""#));

    // link の href フラグメントと content の id が一致し、実際に対になる
    // ことを外部視点で確認する（フォーカス移動先の解決可能性）。
    assert!(link_html.contains(&format!(r##"href="#{DEFAULT_ID}""##)));
    assert!(content_html.contains(&format!(r#"id="{DEFAULT_ID}""#)));
}

/// 呼び出し側 `attrs` に混入させた契約属性（link の `href`、content の
/// `id`/`tabindex`）は大文字小文字を無視して除去され、1 回だけ正規値が
/// 出力される（chakra-ui は `{...rest}` を後展開するため上書き可能だが、
/// 本実装は意図的により厳格な fail-closed を維持する、イシュー #1663）。
#[test]
fn reserved_keys_are_dropped_on_both_parts() {
    for key in ["href", "HREF", "Href"] {
        let html = render(&link(DEFAULT_ID, vec![(key, "attacker")], vec![]));
        assert_eq!(html.matches("href=").count(), 1, "key={key} html={html}");
        assert!(html.contains(r##"href="#fandhe-skip-nav""##));
        assert!(!html.contains("attacker"));
    }

    for (key, spoofed) in [
        ("id", "attacker"),
        ("ID", "attacker"),
        ("tabindex", "0"),
        ("TabIndex", "0"),
    ] {
        let html = render(&content(DEFAULT_ID, vec![(key, spoofed)], vec![]));
        assert_eq!(html.matches("id=").count(), 1, "key={key} html={html}");
        assert_eq!(
            html.matches("tabindex=").count(),
            1,
            "key={key} html={html}"
        );
        assert!(html.contains(r#"id="fandhe-skip-nav""#));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(!html.contains("attacker"));
    }
}

/// `docs/policy/intentional-non-adoption.md` §3.25 規則 2 のガード:
/// chakra-ui の `SkipNavContent` が出力する inline `style={{ outline: 0 }}`
/// のような装飾を headless-ui へ持ち込まないこと、`role`/`aria-*` を
/// 追加しないこと、`data-*` の出現が `data-scope`/`data-part` のみである
/// ことを両パーツで固定する（イシュー #1663 突合結果）。
#[test]
fn rule2_guard_no_style_role_aria_or_extra_data_attrs() {
    for html in [
        render(&link(DEFAULT_ID, vec![], vec![])),
        render(&content(DEFAULT_ID, vec![], vec![])),
    ] {
        assert!(!html.contains("style="), "unexpected style= in: {html}");
        assert!(!html.contains("role="), "unexpected role= in: {html}");
        assert!(!html.contains("aria-"), "unexpected aria- in: {html}");

        let data_attr_count = html.matches(" data-").count();
        let known_data_attr_count =
            html.matches(r#"data-scope="skip-nav""#).count() + html.matches("data-part=").count();
        assert_eq!(
            data_attr_count, known_data_attr_count,
            "unexpected extra data-* attribute in: {html}"
        );
    }
}

/// `id` にスキーム付き文字列や `javascript:` を渡しても、[`link`] の
/// `href` は常に `#` で始まるフラグメントのみを組み立てる（スキーム注入
/// 経路を構造的に持たない不変条件の外部視点固定。XSS エスケープ自体は
/// `tests/xss_escape.rs::skip_nav_id_and_children_are_escaped_for_all_payloads`
/// が別途カバーするため、ここでは `#` 接頭の構造契約に絞る）。
#[test]
fn href_is_fragment_only_for_any_id() {
    for id in [
        "https://example.com",
        "javascript:alert(1)",
        "//evil.example",
        DEFAULT_ID,
    ] {
        let html = render(&link(id, vec![], vec![]));
        assert_eq!(html.matches("href=").count(), 1, "id={id:?} html={html}");
        assert!(
            html.contains(&format!("href=\"#{id}\"")),
            "href does not equal the expected '#' fragment for id={id:?}: {html}"
        );
    }
}

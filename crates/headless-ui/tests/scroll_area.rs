//! ScrollArea（イシュー #825、参考サイト突合はイシュー #1662）の統合テスト。
//!
//! `crates/headless-ui/src/scroll_area.rs` の inline unit tests がパーツ単体
//! の属性出力・予約キー除去・XSS 回帰を固定するのに対し、本ファイルは
//! 「root > viewport > content + scrollbar(vertical/horizontal) + thumb +
//! corner」の 6 パーツ組み立て全体を公開 API のみを使って外部から固定し、
//! 参考サイト（ark-ui/Zag.js・Radix Primitives）と突合した契約（anatomy の
//! 完全一致・`role`/`aria-*` の非付与範囲・`data-orientation` 値語彙・
//! 予約キーのなりすまし防止）を回帰として保護する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::scroll_area::{content, corner, root, scrollbar, thumb, viewport};

/// 6 パーツをすべて組み立てたときの `data-scope`/`data-part` 出力を固定する
/// （ark-ui/Zag.js の anatomy と完全一致、イシュー #1662 突合結果）。
#[test]
fn full_assembly_outputs_all_six_parts() {
    let tree = root(
        vec![],
        vec![viewport(
            vec![],
            vec![
                content(vec![], vec![text("scrollable body")]),
                scrollbar(
                    Orientation::Vertical,
                    vec![],
                    vec![thumb(Orientation::Vertical, vec![], vec![])],
                ),
                scrollbar(
                    Orientation::Horizontal,
                    vec![],
                    vec![thumb(Orientation::Horizontal, vec![], vec![])],
                ),
                corner(vec![], vec![]),
            ],
        )],
    );
    let html = render(&tree);

    assert!(html.contains(r#"data-scope="scroll-area""#));
    for part in [
        "root",
        "viewport",
        "content",
        "scrollbar",
        "thumb",
        "corner",
    ] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "missing data-part=\"{part}\" in: {html}"
        );
    }
}

/// root/viewport/content は `role`/`aria-*` を一切出力しない（Zag.js の
/// `role="presentation"` は WAI-ARIA 1.2 §5.4 によりフォーカス可能な
/// viewport では UA に無視されるため追加しない、イシュー #1662 突合結果）。
#[test]
fn root_viewport_content_have_no_role_or_aria_attrs() {
    let viewport_html = render(&viewport(vec![], vec![content(vec![], vec![text("x")])]));
    assert!(!viewport_html.contains("role="));
    assert!(!viewport_html.contains("aria-"));

    let root_html = render(&root(vec![], vec![]));
    assert!(!root_html.contains("role="));
    assert!(!root_html.contains("aria-"));
}

/// `aria-hidden="true"` は scrollbar/corner のみに付与される（ネイティブ
/// スクロールバーとの意味重複を明示する本実装独自の付与、イシュー #1662）。
#[test]
fn aria_hidden_is_scoped_to_scrollbar_and_corner_only() {
    let scrollbar_html = render(&scrollbar(Orientation::Vertical, vec![], vec![]));
    assert!(scrollbar_html.contains(r#"aria-hidden="true""#));

    let corner_html = render(&corner(vec![], vec![]));
    assert!(corner_html.contains(r#"aria-hidden="true""#));

    let thumb_html = render(&thumb(Orientation::Vertical, vec![], vec![]));
    assert!(!thumb_html.contains("aria-hidden"));

    let content_html = render(&content(vec![], vec![]));
    assert!(!content_html.contains("aria-hidden"));
}

/// viewport はキーボードスクロール到達性のため `tabindex="0"` を固定で持つ
/// （SSR では overflow の有無を判定できず WCAG 2.1.1 に対して安全側、
/// イシュー #1662 突合結果）。
#[test]
fn viewport_always_has_tabindex_zero() {
    let html = render(&viewport(vec![], vec![]));
    assert!(html.contains(r#"tabindex="0""#));
}

/// `data-orientation` の値語彙は `vertical`/`horizontal` の 2 値のみ
/// （`crate::data_attrs::Orientation` に一元化、参照サイトと一致）。
#[test]
fn data_orientation_vocabulary_is_vertical_or_horizontal_only() {
    let vertical = render(&scrollbar(Orientation::Vertical, vec![], vec![]));
    assert!(vertical.contains(r#"data-orientation="vertical""#));

    let horizontal = render(&thumb(Orientation::Horizontal, vec![], vec![]));
    assert!(horizontal.contains(r#"data-orientation="horizontal""#));
}

/// 規則 2 ガード（`docs/policy/intentional-non-adoption.md` §3.25）:
/// Zag.js/Radix が持つ DOM 計測・ポインタ操作由来の `data-*`（overflow 有無・
/// 端到達・hover/scrolling/dragging・可視性状態）はいずれも SSR の静的
/// マークアップでは真の値を決定できないため、本実装のどのパーツも出力
/// しない（イシュー #1662 突合結果、`crate::navigation_menu` の
/// `no_part_outputs_data_motion` と同型）。
#[test]
fn no_part_outputs_measurement_or_pointer_derived_state() {
    let tree = root(
        vec![],
        vec![viewport(
            vec![],
            vec![
                content(vec![], vec![text("body")]),
                scrollbar(
                    Orientation::Vertical,
                    vec![],
                    vec![thumb(Orientation::Vertical, vec![], vec![])],
                ),
                corner(vec![], vec![]),
            ],
        )],
    );
    let html = render(&tree);

    for forbidden in [
        "data-state",
        "data-overflow-x",
        "data-overflow-y",
        "data-at-top",
        "data-at-bottom",
        "data-at-left",
        "data-at-right",
        "data-hover",
        "data-scrolling",
        "data-dragging",
        "data-ownedby",
        " dir=",
        " id=",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected attribute `{forbidden}` in: {html}"
        );
    }
}

/// 呼び出し側 `attrs` による予約キーのなりすまし・重複出力を
/// `drop_reserved`（イシュー #1662）が fail-closed に除去することを、
/// クレート外部の公開 API のみを使って固定する（大文字小文字無視）。
#[test]
fn caller_attrs_cannot_override_reserved_keys() {
    let viewport_html = render(&viewport(vec![("tabindex", "-1")], vec![]));
    assert!(viewport_html.contains(r#"tabindex="0""#));
    assert!(!viewport_html.contains(r#"tabindex="-1""#));

    let scrollbar_html = render(&scrollbar(
        Orientation::Vertical,
        vec![("aria-hidden", "false")],
        vec![],
    ));
    assert!(scrollbar_html.contains(r#"aria-hidden="true""#));
    assert!(!scrollbar_html.contains(r#"aria-hidden="false""#));

    let corner_html = render(&corner(vec![("aria-hidden", "false")], vec![]));
    assert!(corner_html.contains(r#"aria-hidden="true""#));
    assert!(!corner_html.contains(r#"aria-hidden="false""#));
}

// --- XSS 回帰: attrs/children にペイロードを渡してもエスケープされる ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";
const SCRIPT_PAYLOAD: &str = "<script>alert(1)</script>";

#[test]
fn attrs_payload_is_escaped_across_all_parts() {
    let tree = root(
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![viewport(
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![
                content(vec![("data-testid", ATTR_BREAK_PAYLOAD)], vec![]),
                scrollbar(
                    Orientation::Vertical,
                    vec![("data-testid", ATTR_BREAK_PAYLOAD)],
                    vec![thumb(
                        Orientation::Vertical,
                        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
                        vec![],
                    )],
                ),
                corner(vec![("data-testid", ATTR_BREAK_PAYLOAD)], vec![]),
            ],
        )],
    );
    let html = render(&tree);
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn children_text_payload_is_escaped() {
    let html = render(&content(vec![], vec![text(SCRIPT_PAYLOAD)]));
    assert!(!html.contains(SCRIPT_PAYLOAD));
    assert!(html.contains("&lt;script&gt;"));
}

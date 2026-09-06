//! VisuallyHidden（イシュー #776、参考サイト突合はイシュー #1668）の統合テスト。
//!
//! `crates/headless-ui/src/visually_hidden.rs` の inline unit tests がパーツ
//! 単体の属性出力・`aria-hidden` 非付与・なりすまし除去・XSS 回帰を固定する
//! のに対し、本ファイルは公開 API のみを使って外部視点から契約
//! （anatomy 1 パーツ構成・`data-*`/`role`/`aria-*`/`tabindex` の非付与範囲・
//! 参照サイト同型の利用パターン）を回帰として保護する。参照突合の詳細
//! （Radix Primitives / Radix Themes / chakra-ui との突合結果、Ark UI は該当
//! ページ 404）は `src/visually_hidden.rs` のモジュール doc「参考サイトとの
//! 突合」節を正とする。

use fandhe_frontend_core::{button, render, text};
use fandhe_frontend_headless_ui::visually_hidden::root;

/// [`root`] は `span` 1 パーツのみを出力し、`data-scope`/`data-part` を
/// 伴う（Radix Primitives `VisuallyHidden.Root` / chakra-ui `VisuallyHidden`
/// と同型の 1 パーツ anatomy、イシュー #1668 突合結果）。
#[test]
fn root_is_a_span_with_scope_and_part_only() {
    let html = render(&root(vec![], vec![text("補足テキスト")]));
    assert!(html.starts_with("<span"));
    assert!(html.trim_end().ends_with("</span>"));
    assert!(html.contains(r#"data-scope="visually-hidden""#));
    assert!(html.contains(r#"data-part="root""#));
}

/// 参照サイト（Radix/chakra）は `data-*` の状態語彙を一切持たないため、
/// 本モジュールも `data-scope`/`data-part` の 2 個以外の `data-*` を出力
/// しない（イシュー #1668 突合結果: 追加の `data-*` は不要と結論）。
#[test]
fn root_emits_no_state_vocabulary_beyond_scope_and_part() {
    let html = render(&root(vec![], vec![]));
    assert_eq!(html.matches("data-").count(), 2);
    for forbidden in [
        "data-state",
        "data-disabled",
        "data-invalid",
        "data-readonly",
        "data-orientation",
        "data-placement",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden} in {html}"
        );
    }
}

/// 参照 3 軸（Radix Primitives / Radix Themes / chakra-ui）とも
/// `role`/`aria-*`/`tabindex` を自ら付与しない。本モジュールも同様であり、
/// 呼び出し側が明示的に渡さない限り一切現れない（キーボード操作なし・
/// 非対話要素であることの外部視点固定、イシュー #1668）。
#[test]
fn root_emits_no_role_aria_or_tabindex_by_default() {
    let html = render(&root(vec![], vec![text("補足テキスト")]));
    assert!(!html.contains("role="));
    assert!(!html.contains("aria-"));
    assert!(!html.contains("tabindex"));
}

/// `data-scope`/`data-part` のなりすましは大文字小文字を無視して除去される
/// （[`fandhe_frontend_headless_ui`] 側 `Anatomy::part` の既存契約を
/// 外部視点から再固定、`id`/`class` 等それ以外の呼び出し側属性は透過）。
#[test]
fn caller_attrs_pass_through_except_contract_keys() {
    let html = render(&root(
        vec![
            ("id", "sr-label"),
            ("class", "x"),
            ("data-scope", "attacker"),
            ("data-part", "attacker"),
            ("DATA-SCOPE", "attacker2"),
        ],
        vec![],
    ));
    assert!(html.contains(r#"id="sr-label""#));
    assert!(html.contains(r#"class="x""#));
    assert!(html.contains(r#"data-scope="visually-hidden""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(!html.contains("attacker"));
}

/// Radix/chakra のデモ（アイコンのみボタン + 隠しラベル）と同型の利用
/// パターンを固定する: [`root`] のテキストはボタンの子ノードとして DOM に
/// 残り続け、`aria-hidden` は現れない（支援技術がラベルを読み上げられる
/// ことの回帰、イシュー #1668）。
#[test]
fn icon_only_button_pattern_keeps_label_in_dom_without_aria_hidden() {
    let html = render(&button(
        vec![("type", "button")],
        vec![text("🔔"), root(vec![], vec![text("Notifications")])],
    ));
    assert!(html.contains(">Notifications<"));
    assert!(html.contains(">🔔"));
    assert!(!html.contains("aria-hidden"));
}

// --- エスケープ回帰（外部視点からの再固定） ---

#[test]
fn children_and_attrs_payloads_are_escaped() {
    let html = render(&root(
        vec![("data-testid", "\"><script>alert(1)</script>")],
        vec![text("<script>alert(2)</script>")],
    ));
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"));
    assert!(html.contains("&quot;"));
}

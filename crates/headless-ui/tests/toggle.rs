//! Toggle（`crates/headless-ui/src/toggle.rs`、イシュー #746）の SSR パーツ
//! 関数群（`toggle::{root, indicator}`、純粋関数で完結）を参考サイト
//! （ark-ui / Radix Primitives）と突合し続けることを fail-closed に固定する
//! （`tests/checkbox.rs` の「参考サイト突合契約（イシュー #1602）」節と同型の
//! 趣旨。差分調査の詳細はイシュー #1629 コメント参照）。
//!
//! `src/toggle.rs` 内の `#[cfg(test)]` が `Toggle`（dispatch/hydration 統合）
//! と各パーツの基本出力（scope/part/state/aria）を固定するのに対し、本
//! ファイルは公開 API のみを使い、ark-ui `toggle.connect.ts` が Indicator
//! パートへ `data-state`/`data-pressed`/`data-disabled` の 3 属性を持たせる
//! ことへの突合（イシュー #1629 で是正した差分）を中心に固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::toggle::{indicator, root};

/// root/indicator の 2 パートが `data-scope="toggle"` と期待する `data-part`
/// を持つことを固定する（ark-ui の Toggle anatomy は Root/Indicator の 2
/// パーツ、Radix Primitives の Toggle は Root 単独構成であり後者は前者の
/// 部分集合のため、fandhe の 2 パーツ構成はいずれとも矛盾しない）。
#[test]
fn reference_anatomy_part_names_match_ark_ui() {
    let parts: [(&str, String); 2] = [
        ("root", render(&root(false, false, vec![], vec![]))),
        (
            "indicator",
            render(&indicator(false, false, vec![], vec![])),
        ),
    ];
    for (part, html) in &parts {
        assert!(
            html.contains(r#"data-scope="toggle""#),
            "{part} が data-scope=\"toggle\" を持たない: {html}"
        );
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "{part} が期待する data-part を持たない: {html}"
        );
    }
}

/// `pressed` true/false が root/indicator 双方の `data-state` へ一貫して
/// `"on"`/`"off"` を反映することを固定する（ark-ui の「全パーツが
/// data-state を持つ」規約）。
#[test]
fn data_state_vocabulary_is_on_off_on_every_part() {
    for (pressed, expected) in [(false, "off"), (true, "on")] {
        let expected_attr = format!(r#"data-state="{expected}""#);
        assert!(render(&root(pressed, false, vec![], vec![])).contains(&expected_attr));
        assert!(render(&indicator(pressed, false, vec![], vec![])).contains(&expected_attr));
    }
}

/// ark-ui `toggle.connect.ts` は Root/Indicator 双方へ `data-pressed`
/// （存在属性）・`data-disabled`（存在属性）を持たせる。イシュー #1629 是正
/// 前は `indicator` がこの 2 属性を欠いていた差分を固定する。
#[test]
fn data_pressed_and_data_disabled_are_present_on_every_part() {
    // pressed のみ true: data-pressed が両パートに現れ、data-disabled はどちらにも現れない。
    let root_html = render(&root(true, false, vec![], vec![]));
    let indicator_html = render(&indicator(true, false, vec![], vec![]));
    assert!(root_html.contains(r#"data-pressed="""#), "{root_html}");
    assert!(!root_html.contains("data-disabled"), "{root_html}");
    assert!(
        indicator_html.contains(r#"data-pressed="""#),
        "{indicator_html}"
    );
    assert!(
        !indicator_html.contains("data-disabled"),
        "{indicator_html}"
    );

    // disabled のみ true: data-disabled が両パートに現れ、data-pressed はどちらにも現れない。
    let root_html = render(&root(false, true, vec![], vec![]));
    let indicator_html = render(&indicator(false, true, vec![], vec![]));
    assert!(root_html.contains(r#"data-disabled="""#), "{root_html}");
    assert!(!root_html.contains("data-pressed"), "{root_html}");
    assert!(
        indicator_html.contains(r#"data-disabled="""#),
        "{indicator_html}"
    );
    assert!(!indicator_html.contains("data-pressed"), "{indicator_html}");

    // 両方 true: 両パートとも両属性を持つ。
    let root_html = render(&root(true, true, vec![], vec![]));
    let indicator_html = render(&indicator(true, true, vec![], vec![]));
    assert!(root_html.contains(r#"data-pressed="""#) && root_html.contains(r#"data-disabled="""#));
    assert!(
        indicator_html.contains(r#"data-pressed="""#)
            && indicator_html.contains(r#"data-disabled="""#)
    );
}

/// `root` はネイティブ `<button type="button">` であり `aria-pressed` を
/// 持ち `role` を出力しない（ネイティブ button の暗黙ロール）。`disabled`
/// 時はネイティブ `disabled` 属性も付与する。`indicator` は装飾用パーツの
/// ため `aria-pressed` を持たない（Radix Primitives の Toggle も同様に
/// `button` 単体で `aria-pressed` を担う）。
#[test]
fn root_is_native_button_with_aria_pressed_and_no_role() {
    let html = render(&root(true, true, vec![], vec![]));
    assert!(html.contains(r#"type="button""#));
    assert!(html.contains(r#"aria-pressed="true""#));
    assert!(html.contains(r#"disabled="""#));
    assert!(!html.contains("role="));

    let indicator_html = render(&indicator(true, false, vec![], vec![]));
    assert!(!indicator_html.contains("aria-pressed"));
}

/// ark-ui は pointer/focus のローカル操作状態として
/// `data-hover`/`data-active`/`data-focus`/`data-motion` を付与するが、本
/// 実装は SSR 静的出力にこれらを持たせない設計判断（UI 部品の責務境界、
/// `docs/policy/intentional-non-adoption.md` §3.25 規則 2）を固定する
/// （`tests/checkbox.rs::no_part_outputs_pointer_or_focus_interaction_attrs`
/// と同型の趣旨）。
#[test]
fn no_part_outputs_pointer_or_focus_interaction_attrs() {
    for pressed in [false, true] {
        for disabled in [false, true] {
            let html = render(&root(
                pressed,
                disabled,
                vec![],
                vec![indicator(pressed, disabled, vec![], vec![text("B")])],
            ));
            assert!(!html.contains("data-hover"), "{html}");
            assert!(!html.contains("data-active"), "{html}");
            assert!(
                !html.contains("data-focus=\"") && !html.contains("data-focus \""),
                "{html}"
            );
            assert!(!html.contains("data-motion"), "{html}");
        }
    }
}

// --- XSS 回帰: 新引数（indicator の disabled）経路を含む攻撃ペイロード ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

/// `indicator` の新引数経路（`disabled`）を含めても、呼び出し側 `attrs`・
/// `children` は既定エスケープを経由する（REQ-1）。呼び出し側の
/// `data-scope`/`data-part`/`data-pressed`/`data-disabled` 偽装は
/// `Anatomy::part` の既存 fail-closed 規約（`ANATOMY.part` が固定属性を
/// 呼び出し側 `attrs` より先に置き、後続 `merged.extend(attrs)` は
/// `Vec` への追加のため属性名が重複しても最終レンダリングでは両方出力
/// される仕様。ここでは値のエスケープのみを固定し、偽装排除は
/// `caller_supplied_scope_and_part_are_dropped`（`src/toggle.rs` 内
/// テスト）が別途固定する）。
#[test]
fn indicator_new_disabled_arg_path_escapes_attrs_and_children() {
    let html = render(&indicator(
        true,
        true,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![text("<script>alert(1)</script>")],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;"));
}

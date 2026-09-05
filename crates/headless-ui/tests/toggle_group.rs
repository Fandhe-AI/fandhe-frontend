//! ToggleGroup（`crates/headless-ui/src/toggle_group.rs`、イシュー #746）の
//! SSR パーツ関数群（`toggle_group::{root, item}`、純粋関数で完結）を参考
//! サイト（ark-ui / Radix Primitives）と突合し続けることを fail-closed に
//! 固定する（`tests/toggle.rs`「参考サイト突合契約（イシュー #1629）」節・
//! `tests/checkbox.rs`「参考サイト突合契約（イシュー #1602）」節と同型の
//! 趣旨。差分調査の詳細はイシュー #1630 コメント参照）。
//!
//! `src/toggle_group.rs` 内の `#[cfg(test)]` が `ToggleGroup`/
//! `MultiToggleGroup`（dispatch/hydration 統合）と各パーツの基本出力
//! （scope/part/state/aria）を固定するのに対し、本ファイルは
//! `fandhe_frontend_headless_ui` の公開 API（`lib.rs` の再エクスポート）
//! のみを経由し、イシュー #1630 で是正した参照突合差分（item への
//! `data-orientation` 追加・root disabled の item への伝播・roving
//! tabindex opt-in・呼び出し側 `attrs` 偽装の fail-closed 除去）を中心に
//! 固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::{toggle_group, ToggleGroup, ToggleGroupProps};

/// root/item の 2 パートが `data-scope="toggle-group"` と期待する
/// `data-part` を持つことを固定する（ark-ui/Radix とも Root/Item の 2
/// anatomy パーツ構成）。`ToggleGroupProps`/`ToggleGroup` へは
/// `fandhe_frontend_headless_ui` の再エクスポート経由でのみ到達し、
/// `lib.rs` の再エクスポートが崩れていないことも併せて固定する。
#[test]
fn reference_anatomy_part_names_match_ark_ui_and_radix() {
    let props = ToggleGroupProps::default();
    let parts: [(&str, String); 2] = [
        (
            "root",
            render(&toggle_group::root(&props, None, vec![], vec![])),
        ),
        (
            "item",
            render(&toggle_group::item(
                &props,
                false,
                false,
                false,
                "bold",
                vec![],
                vec![],
            )),
        ),
    ];
    for (part, html) in &parts {
        assert!(
            html.contains(r#"data-scope="toggle-group""#),
            "{part} が data-scope=\"toggle-group\" を持たない: {html}"
        );
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "{part} が期待する data-part を持たない: {html}"
        );
    }
}

/// ark-ui（Zag `toggle-group`）/ Radix Primitives とも Item に
/// `data-orientation` を持つ。イシュー #1630 是正前は root のみへ付与して
/// いた差分を固定する: `props.orientation` が `Some` のとき item も root と
/// 同値の `data-orientation` を持ち、`None` のときは item も出力しない。
#[test]
fn item_data_orientation_matches_root_when_some_and_absent_when_none() {
    use fandhe_frontend_headless_ui::data_attrs::Orientation;

    let none_props = ToggleGroupProps::default();
    let root_html = render(&toggle_group::root(&none_props, None, vec![], vec![]));
    let item_html = render(&toggle_group::item(
        &none_props,
        false,
        false,
        false,
        "bold",
        vec![],
        vec![],
    ));
    assert!(!root_html.contains("data-orientation"), "{root_html}");
    assert!(!item_html.contains("data-orientation"), "{item_html}");

    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let props = ToggleGroupProps {
            orientation: Some(orientation),
            ..ToggleGroupProps::default()
        };
        let root_html = render(&toggle_group::root(&props, None, vec![], vec![]));
        let item_html = render(&toggle_group::item(
            &props,
            false,
            false,
            false,
            "bold",
            vec![],
            vec![],
        ));
        let expected = match orientation {
            Orientation::Horizontal => r#"data-orientation="horizontal""#,
            Orientation::Vertical => r#"data-orientation="vertical""#,
        };
        assert!(root_html.contains(expected), "{root_html}");
        assert!(item_html.contains(expected), "{item_html}");
    }
}

/// ark-ui/Radix とも Root の disabled は全 item を無効化する契約。イシュー
/// #1630 是正前は root にのみ `data-disabled` を付与し item へ伝播しな
/// かった差分を固定する: `props.disabled = true` のとき item 自身の
/// `disabled` 引数が `false` でも実効的に無効化される。
#[test]
fn root_disabled_propagates_to_item_even_when_item_disabled_arg_is_false() {
    let props = ToggleGroupProps {
        disabled: true,
        ..ToggleGroupProps::default()
    };
    let html = render(&toggle_group::item(
        &props,
        false,
        false,
        false,
        "bold",
        vec![],
        vec![],
    ));
    assert!(html.contains(r#"data-disabled="""#), "{html}");
    assert!(html.contains(r#"disabled="""#), "{html}");

    // item 個別の disabled との論理和であることも確認する（root は
    // disabled=false のままでも item 個別の disabled=true は維持される）。
    let default_props = ToggleGroupProps::default();
    let html = render(&toggle_group::item(
        &default_props,
        false,
        false,
        true,
        "bold",
        vec![],
        vec![],
    ));
    assert!(html.contains(r#"data-disabled="""#), "{html}");
}

/// `ToggleGroupProps::roving_focus`（既定 `false`）+ `item` の `focused`
/// 引数によるオプトインの roving tabindex（イシュー #1630 で新設）。
/// `false` のときは `focused` の値によらず `tabindex` を一切出力しない
/// （no-JS SSR で全 item が Tab で到達可能な後方互換を維持する）。
#[test]
fn roving_focus_opt_in_controls_tabindex_output() {
    let default_props = ToggleGroupProps::default();
    for focused in [false, true] {
        let html = render(&toggle_group::item(
            &default_props,
            false,
            focused,
            false,
            "bold",
            vec![],
            vec![],
        ));
        assert!(!html.contains("tabindex"), "{html}");
    }

    let roving_props = ToggleGroupProps {
        roving_focus: true,
        ..ToggleGroupProps::default()
    };
    let focused_html = render(&toggle_group::item(
        &roving_props,
        false,
        true,
        false,
        "bold",
        vec![],
        vec![],
    ));
    let unfocused_html = render(&toggle_group::item(
        &roving_props,
        false,
        false,
        false,
        "bold",
        vec![],
        vec![],
    ));
    assert!(focused_html.contains(r#"tabindex="0""#), "{focused_html}");
    assert!(
        unfocused_html.contains(r#"tabindex="-1""#),
        "{unfocused_html}"
    );
}

/// 呼び出し側 `attrs` からの `tabindex`/`data-value`/`aria-pressed`/
/// `data-orientation`/`data-state`/`disabled` 等の偽装を ASCII 大文字小文字
/// 無視で fail-closed に除去する（イシュー #1630 で新設した
/// `drop_reserved`。`Anatomy::part` の `data-scope`/`data-part` 除去と同型の
/// 追加防御）。
#[test]
fn caller_attrs_cannot_forge_reserved_item_and_root_keys() {
    let props = ToggleGroupProps {
        roving_focus: true,
        orientation: Some(fandhe_frontend_headless_ui::data_attrs::Orientation::Horizontal),
        ..ToggleGroupProps::default()
    };
    let item_html = render(&toggle_group::item(
        &props,
        true,
        true,
        false,
        "bold",
        vec![
            ("TYPE", "submit"),
            ("Aria-Pressed", "false"),
            ("DATA-STATE", "off"),
            ("data-value", "attacker"),
            ("Data-Orientation", "attacker"),
            ("TabIndex", "99"),
            ("Disabled", "attacker"),
        ],
        vec![],
    ));
    assert!(item_html.contains(r#"type="button""#), "{item_html}");
    assert!(item_html.contains(r#"aria-pressed="true""#), "{item_html}");
    assert!(item_html.contains(r#"data-state="on""#), "{item_html}");
    assert!(item_html.contains(r#"data-value="bold""#), "{item_html}");
    assert!(
        item_html.contains(r#"data-orientation="horizontal""#),
        "{item_html}"
    );
    assert!(item_html.contains(r#"tabindex="0""#), "{item_html}");
    assert!(!item_html.contains("attacker"), "{item_html}");
    assert!(!item_html.contains("99"), "{item_html}");
    assert!(!item_html.contains("submit"), "{item_html}");

    let root_html = render(&toggle_group::root(
        &props,
        Some("group-label"),
        vec![
            ("ROLE", "attacker"),
            ("Data-Orientation", "attacker"),
            ("ARIA-LABELLEDBY", "attacker"),
            ("Data-Disabled", "attacker"),
        ],
        vec![],
    ));
    assert!(root_html.contains(r#"role="group""#), "{root_html}");
    assert!(
        root_html.contains(r#"data-orientation="horizontal""#),
        "{root_html}"
    );
    assert!(
        root_html.contains(r#"aria-labelledby="group-label""#),
        "{root_html}"
    );
    assert!(!root_html.contains("attacker"), "{root_html}");
}

/// ark-ui は pointer/focus のローカル操作状態として `data-hover`/
/// `data-active`/`data-focus`/`data-motion` を付与するが、本実装は SSR
/// 静的出力にこれらを持たせない設計判断（UI 部品の責務境界、
/// `docs/policy/intentional-non-adoption.md` §3.25 規則 2）を固定する
/// （`tests/toggle.rs::no_part_outputs_pointer_or_focus_interaction_attrs`
/// と同型の趣旨）。
#[test]
fn no_part_outputs_pointer_or_focus_interaction_attrs() {
    let props = ToggleGroupProps {
        roving_focus: true,
        ..ToggleGroupProps::default()
    };
    for pressed in [false, true] {
        for focused in [false, true] {
            for disabled in [false, true] {
                let html = render(&toggle_group::root(
                    &props,
                    None,
                    vec![],
                    vec![toggle_group::item(
                        &props,
                        pressed,
                        focused,
                        disabled,
                        "bold",
                        vec![],
                        vec![text("B")],
                    )],
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
}

// --- XSS 回帰: value/labelled_by/呼び出し側 attrs/children ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

/// 動的値（`value`/`labelled_by`/呼び出し側 `attrs`/`children` テキスト）は
/// 新引数（`props`/`focused`）経路を含めても既定エスケープ（REQ-1）を必ず
/// 経由する。
#[test]
fn dynamic_values_are_escaped_through_new_props_and_focused_arguments() {
    let props = ToggleGroupProps {
        roving_focus: true,
        ..ToggleGroupProps::default()
    };

    let value_html = render(&toggle_group::item(
        &props,
        false,
        true,
        false,
        ATTR_BREAK_PAYLOAD,
        vec![],
        vec![],
    ));
    assert!(
        !value_html.contains("onmouseover=\"alert(1)"),
        "{value_html}"
    );
    assert!(value_html.contains("&quot;"), "{value_html}");

    let labelled_by_html = render(&toggle_group::root(
        &props,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    ));
    assert!(
        !labelled_by_html.contains("onmouseover=\"alert(1)"),
        "{labelled_by_html}"
    );
    assert!(labelled_by_html.contains("&quot;"), "{labelled_by_html}");

    let attrs_html = render(&toggle_group::root(
        &props,
        None,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(
        !attrs_html.contains("onmouseover=\"alert(1)"),
        "{attrs_html}"
    );

    let children_html = render(&toggle_group::item(
        &props,
        false,
        false,
        false,
        "bold",
        vec![],
        vec![text("<script>alert(1)</script>")],
    ));
    assert!(
        !children_html.contains("<script>alert(1)</script>"),
        "{children_html}"
    );
    assert!(children_html.contains("&lt;script&gt;"), "{children_html}");
}

/// `ToggleGroup::item` 利便メソッド（`fandhe_frontend_headless_ui` 経由の
/// 公開 API）が新シグネチャ（`props`/`focused`）でも自由関数 `item` と同じ
/// 出力になることを固定する。
#[test]
fn toggle_group_convenience_item_method_matches_free_function_via_public_api() {
    use fandhe_frontend_interactive::dispatch;

    let mut g = ToggleGroup::default();
    assert!(dispatch(&mut g, "toggle", "bold"));

    let props = ToggleGroupProps {
        roving_focus: true,
        ..ToggleGroupProps::default()
    };
    let via_method = render(&g.item(&props, "bold", true, false, vec![], vec![text("B")]));
    let via_free_fn = render(&toggle_group::item(
        &props,
        true,
        true,
        false,
        "bold",
        vec![],
        vec![text("B")],
    ));
    assert_eq!(via_method, via_free_fn);
}

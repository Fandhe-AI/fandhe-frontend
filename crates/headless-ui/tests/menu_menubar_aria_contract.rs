//! `menu` ↔ `menubar` の ARIA・状態語彙が同一であることを固定する横断契約
//! テスト（イシュー #1068、親 #1058）。
//!
//! # 本ファイルの責務境界
//!
//! `crates/headless-ui/src/menu.rs` / `src/menubar.rs` の `#[cfg(test)]`
//! ユニットテストと `tests/menu.rs` / `tests/menubar.rs` は、それぞれ
//! **menu 単体**・**menubar 単体**の属性出力そのものを固定する（片側完結）。
//! 本ファイルはそれらと重複させず、**menu と menubar の「関係」だけ**を
//! 検査する。すなわち「両モジュールが同じロール・状態語彙を再利用する」と
//! いう `src/menubar.rs` モジュール doc（「`menu` mod 再利用の内訳」節）の
//! 設計前提そのものを、両モジュールを跨いで突き合わせることでロックする。
//! 片側だけの変更（例: `menubar::content` から `role="menu"` を落とす、
//! `menubar::separator` のタグを `hr` から `div` に変える、`menu::trigger_item`
//! に新しい属性を足して `menubar::sub_trigger` へ反映し忘れる）は、片側完結の
//! 既存テストでは検知できないが本ファイルでは検知される。
//!
//! 公開 API（`fandhe_frontend_headless_ui::{menu, menubar}` と
//! `fandhe_frontend_core::Node`）のみを経由し、`raw_html()` は使わず、HTML
//! 文字列を直接組み立てない（`.claude/rules/coding-rust.md`）。
//!
//! 本ファイルの契約テストは、以後の削除・弱体化・`#[ignore]` 化を禁止する
//! （`tests/xss_escape.rs` と同じ運用、`.claude/rules/coding-rust.md`）。

use std::collections::BTreeMap;

use fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::menu;
use fandhe_frontend_headless_ui::menubar;
use fandhe_frontend_headless_ui::OpenState;

/// `Node` 木を走査し、指定した `data-scope` を持つ各パーツについて
/// `(タグ名, data-scope/data-part を除いた属性マップ)` を集める。
///
/// 属性は `BTreeMap<String, String>`（順序非依存）で比較する。HTML の属性
/// 出力順序は意味を持たず、順序まで固定すると実装の些末な並べ替え（例:
/// `merged.extend` の呼び出し順の入れ替え）で偽陽性が出るため、生 HTML
/// 文字列の完全一致比較へは後退させない。
///
/// `Anatomy::part`（`crate::anatomy::Anatomy::part`）は `data-scope`/
/// `data-part` 以外の重複除去を行わないため、属性名が重複したまま
/// `BTreeMap` へ入れると後勝ちで握り潰されてしまう。これはそれ自体が
/// 契約違反（同一パーツが同一属性を二重出力している）として扱い、
/// 検出したら `panic!` してテストを失敗させる。
fn parts_of(node: &Node, scope: &str) -> BTreeMap<String, (String, BTreeMap<String, String>)> {
    let mut out = BTreeMap::new();
    collect_parts(node, scope, &mut out);
    out
}

fn collect_parts(
    node: &Node,
    scope: &str,
    out: &mut BTreeMap<String, (String, BTreeMap<String, String>)>,
) {
    match node {
        Node::Element {
            tag,
            attrs,
            children,
        } => {
            let node_scope = attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("data-scope"))
                .map(|(_, v)| v.as_str());
            let node_part = attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("data-part"))
                .map(|(_, v)| v.as_str());

            if node_scope == Some(scope) {
                if let Some(part) = node_part {
                    let mut map = BTreeMap::new();
                    for (k, v) in attrs {
                        if k.eq_ignore_ascii_case("data-scope")
                            || k.eq_ignore_ascii_case("data-part")
                        {
                            continue;
                        }
                        if map.insert(k.clone(), v.clone()).is_some() {
                            panic!(
                                "重複した属性名 `{k}` が data-scope=\"{scope}\" data-part=\"{part}\" \
                                 のパーツから検出された（契約違反）"
                            );
                        }
                    }
                    out.insert(part.to_string(), (tag.to_string(), map));
                }
            }

            for child in children {
                collect_parts(child, scope, out);
            }
        }
        Node::Text(_) | Node::RawHtml(_) => {}
    }
}

/// 単一パーツの `(タグ名, 属性マップ)` を取得する。見つからなければ
/// `panic!`（呼び出し元テストの前提が崩れていることを明確にするため）。
fn part_of(node: &Node, scope: &str, part: &str) -> (String, BTreeMap<String, String>) {
    parts_of(node, scope)
        .remove(part)
        .unwrap_or_else(|| panic!("data-scope=\"{scope}\" data-part=\"{part}\" が見つからない"))
}

// --- 1. menubar パーツ集合の凍結 ---
//
// menubar が出すパーツ集合が「対応表」記載の 11 件と完全一致することを
// 固定する。menubar 側にパーツを足す／消す変更は必ずこのテストを落とし、
// 対応表（本ファイルの各テスト）の見直しを強制する。
//
// menu 側のパーツ全 16 件はあえて凍結しない。menu へ無関係なパーツが
// 増えたときに「menubar 語彙一致」という名前のテストが落ちるのは読み手を
// 混乱させ、#1068 の要求範囲でもない。対応表（2. 以降）が「対になる menu
// パーツが今も存在すること」を別途保証するため、リネーム・削除はそちらで
// 検知される。

#[test]
fn menubar_part_inventory_is_frozen() {
    let tree = sample_menubar_tree();
    let parts = parts_of(&tree, "menubar");
    // BTreeMap のキー順（アルファベット順）で比較する。anatomy 上の宣言順
    // ではなく集合としての一致を確認したいため、期待値もアルファベット順に
    // 揃える（本ファイル §対応表の宣言順とは意図的に異なる）。
    let names: Vec<&str> = parts.keys().map(String::as_str).collect();

    let mut expected = [
        "root",
        "menu",
        "trigger",
        "positioner",
        "content",
        "item",
        "item-group",
        "item-group-label",
        "separator",
        "sub-trigger",
        "sub-content",
    ];
    expected.sort_unstable();

    assert_eq!(
        names, expected,
        "menubar のパーツ集合が対応表（本ファイル §1）の 11 件と一致しない。\
         パーツを追加・削除・改名した場合はこのテストと対応表を見直すこと"
    );
}

/// menubar の全 11 パーツを 1 本の木に組み立てたサンプル。open/closed 双方
/// が観測できるよう、`content` は open、`sub-content` は closed で作る。
fn sample_menubar_tree() -> Node {
    let item_group = menubar::item_group(
        Some("group-label-1"),
        vec![],
        vec![
            menubar::item_group_label(Some("group-label-1"), vec![], vec![]),
            menubar::item("item-1", false, false, vec![], vec![]),
        ],
    );
    let separator = menubar::separator(vec![], vec![]);
    let sub_trigger = menubar::sub_trigger(
        OpenState::Closed,
        false,
        false,
        Some("sub-1"),
        vec![],
        vec![],
    );
    let sub_content = menubar::sub_content(OpenState::Closed, Some("sub-1"), None, vec![], vec![]);
    let content = menubar::content(
        OpenState::Open,
        Some("content-1"),
        None,
        vec![],
        vec![item_group, separator, sub_trigger, sub_content],
    );
    let positioner = menubar::positioner(OpenState::Open, vec![], vec![content]);
    let trigger = menubar::trigger(
        true,
        OpenState::Open,
        false,
        false,
        Some("content-1"),
        vec![],
        vec![],
    );
    let menu_wrap = menubar::menu(OpenState::Open, vec![], vec![trigger, positioner]);
    menubar::root(
        fandhe_frontend_headless_ui::Orientation::Horizontal,
        "App menu",
        vec![],
        vec![menu_wrap],
    )
}

// --- 2. 完全一致 8 ペアの総当たり ---

/// 完全一致すべき menubar ↔ menu パーツ対応（対応表本体）。
const IDENTICAL_PAIRS: &[(&str, &str)] = &[
    ("content", "content"),
    ("sub-content", "content"),
    ("sub-trigger", "trigger-item"),
    ("item", "item"),
    ("item-group", "item-group"),
    ("item-group-label", "item-group-label"),
    ("separator", "separator"),
    ("positioner", "positioner"),
];

#[test]
fn paired_parts_render_identical_tag_and_attribute_map() {
    for state in [OpenState::Open, OpenState::Closed] {
        for disabled in [true, false] {
            for highlighted in [true, false] {
                for id in [None, Some("id-1")] {
                    for labelledby in [None, Some("label-1")] {
                        for controls in [None, Some("controls-1")] {
                            assert_pair(
                                "content",
                                "content",
                                menubar::content(state, id, labelledby, vec![], vec![]),
                                menu::content(state, id, labelledby, vec![], vec![]),
                                state,
                            );
                            assert_pair(
                                "sub-content",
                                "content",
                                menubar::sub_content(state, id, labelledby, vec![], vec![]),
                                menu::content(state, id, labelledby, vec![], vec![]),
                                state,
                            );
                            assert_pair(
                                "sub-trigger",
                                "trigger-item",
                                menubar::sub_trigger(
                                    state,
                                    disabled,
                                    highlighted,
                                    controls,
                                    vec![],
                                    vec![],
                                ),
                                menu::trigger_item(
                                    state,
                                    disabled,
                                    highlighted,
                                    controls,
                                    vec![],
                                    vec![],
                                ),
                                state,
                            );
                            assert_pair(
                                "item",
                                "item",
                                menubar::item("v", disabled, highlighted, vec![], vec![]),
                                menu::item("v", disabled, highlighted, vec![], vec![]),
                                state,
                            );
                            assert_pair(
                                "item-group",
                                "item-group",
                                menubar::item_group(labelledby, vec![], vec![]),
                                menu::item_group(labelledby, vec![], vec![]),
                                state,
                            );
                            assert_pair(
                                "item-group-label",
                                "item-group-label",
                                menubar::item_group_label(id, vec![], vec![]),
                                menu::item_group_label(id, vec![], vec![]),
                                state,
                            );
                            assert_pair(
                                "separator",
                                "separator",
                                menubar::separator(vec![], vec![]),
                                menu::separator(vec![], vec![]),
                                state,
                            );
                            assert_pair(
                                "positioner",
                                "positioner",
                                menubar::positioner(state, vec![], vec![]),
                                menu::positioner(state, vec![], vec![]),
                                state,
                            );
                        }
                    }
                }
            }
        }
    }
    // `IDENTICAL_PAIRS` は本テストが実際に検査するペア一覧のドキュメントと
    // して保持する（実検査は上の呼び出しで行う。件数の食い違いに気付ける
    // よう、対応表の総数をここで確認する）。
    assert_eq!(
        IDENTICAL_PAIRS.len(),
        8,
        "対応表のペア数が変わった場合は本テストの呼び出しも見直すこと"
    );
}

/// `menubar_node`/`menu_node` それぞれから対象パーツ 1 件を取り出し、
/// `(タグ, 属性マップ)` が完全一致することを確認する。
fn assert_pair(
    menubar_part: &str,
    menu_part: &str,
    menubar_node: Node,
    menu_node: Node,
    state: OpenState,
) {
    let (menubar_tag, menubar_attrs) = part_of(&menubar_node, "menubar", menubar_part);
    let (menu_tag, menu_attrs) = part_of(&menu_node, "menu", menu_part);
    assert_eq!(
        menubar_tag, menu_tag,
        "menubar:{menubar_part} ↔ menu:{menu_part} のタグ名が不一致（state={state:?}）"
    );
    assert_eq!(
        menubar_attrs, menu_attrs,
        "menubar:{menubar_part} ↔ menu:{menu_part} の属性マップが不一致（state={state:?}）"
    );
}

// --- 3. trigger ペア: 共通語彙 + 意図的差分 ---

#[test]
fn trigger_pair_shares_core_vocabulary_with_documented_deltas() {
    for state in [OpenState::Open, OpenState::Closed] {
        for disabled in [true, false] {
            let menubar_trigger = menubar::trigger(
                true,
                state,
                disabled,
                false,
                Some("controls-1"),
                vec![],
                vec![],
            );
            let menu_trigger = menu::trigger(state, disabled, Some("controls-1"), vec![], vec![]);

            let (mb_tag, mb_attrs) = part_of(&menubar_trigger, "menubar", "trigger");
            let (m_tag, m_attrs) = part_of(&menu_trigger, "menu", "trigger");

            assert_eq!(mb_tag, "button");
            assert_eq!(m_tag, "button");
            assert_eq!(mb_tag, m_tag, "trigger ペアのタグは共通のはず");

            // (a) 共通属性は値まで完全一致する。
            for key in [
                "type",
                "aria-haspopup",
                "aria-expanded",
                "data-state",
                "aria-controls",
            ] {
                assert_eq!(
                    mb_attrs.get(key),
                    m_attrs.get(key),
                    "共通属性 `{key}` の値が menubar/menu 間で不一致（state={state:?}, disabled={disabled}）"
                );
            }

            // (b) menubar 固有の属性名集合（tabindex は roving のため常時、
            //     role/aria-disabled は disabled 時のみ・意図的な出力差分）。
            let mut expected_menubar_only: Vec<&str> = vec!["role", "tabindex"];
            if disabled {
                expected_menubar_only.push("aria-disabled");
            }
            let menubar_only: Vec<&str> = mb_attrs
                .keys()
                .filter(|k| !m_attrs.contains_key(*k))
                .map(String::as_str)
                .collect();
            assert_eq_unordered(
                &menubar_only,
                &expected_menubar_only,
                "menubar::trigger 固有属性集合",
                state,
                disabled,
            );

            // (c) menu 固有の属性名集合（disabled 時のみネイティブ disabled）。
            let mut expected_menu_only: Vec<&str> = vec![];
            if disabled {
                expected_menu_only.push("disabled");
            }
            let menu_only: Vec<&str> = m_attrs
                .keys()
                .filter(|k| !mb_attrs.contains_key(*k))
                .map(String::as_str)
                .collect();
            assert_eq_unordered(
                &menu_only,
                &expected_menu_only,
                "menu::trigger 固有属性集合",
                state,
                disabled,
            );
        }
    }
}

fn assert_eq_unordered(
    actual: &[&str],
    expected: &[&str],
    label: &str,
    state: OpenState,
    disabled: bool,
) {
    let mut a: Vec<&str> = actual.to_vec();
    let mut e: Vec<&str> = expected.to_vec();
    a.sort_unstable();
    e.sort_unstable();
    assert_eq!(
        a, e,
        "{label} が期待値と不一致（state={state:?}, disabled={disabled}）"
    );
}

// --- 4. aria-expanded ⇔ data-state の整合（4 関数） ---

#[test]
fn expanded_and_data_state_agree_in_both_scopes() {
    for state in [OpenState::Open, OpenState::Closed] {
        let cases: Vec<(&str, Node)> = vec![
            (
                "menu::trigger",
                menu::trigger(state, false, None, vec![], vec![]),
            ),
            (
                "menu::trigger_item",
                menu::trigger_item(state, false, false, None, vec![], vec![]),
            ),
            (
                "menubar::trigger",
                menubar::trigger(true, state, false, false, None, vec![], vec![]),
            ),
            (
                "menubar::sub_trigger",
                menubar::sub_trigger(state, false, false, None, vec![], vec![]),
            ),
        ];
        for (label, node) in cases {
            let html = fandhe_frontend_core::render(&node);
            let expanded_true = html.contains(r#"aria-expanded="true""#);
            let state_open = html.contains(r#"data-state="open""#);
            assert_eq!(
                expanded_true, state_open,
                "{label}: aria-expanded と data-state が矛盾している（state={state:?}）"
            );
            assert_eq!(expanded_true, state.is_open());
        }
    }
}

// --- 5. hidden の有無 ⇔ closed（positioner/content/sub-content） ---

#[test]
fn hidden_presence_follows_closed_state_in_both_scopes() {
    for state in [OpenState::Open, OpenState::Closed] {
        let cases: Vec<(&str, Node)> = vec![
            ("menu::positioner", menu::positioner(state, vec![], vec![])),
            (
                "menu::content",
                menu::content(state, None, None, vec![], vec![]),
            ),
            (
                "menubar::positioner",
                menubar::positioner(state, vec![], vec![]),
            ),
            (
                "menubar::content",
                menubar::content(state, None, None, vec![], vec![]),
            ),
            (
                "menubar::sub_content",
                menubar::sub_content(state, None, None, vec![], vec![]),
            ),
        ];
        for (label, node) in cases {
            let html = fandhe_frontend_core::render(&node);
            let has_hidden = html.contains("hidden");
            assert_eq!(
                has_hidden,
                !state.is_open(),
                "{label}: hidden の有無が closed 状態と矛盾している（state={state:?}）"
            );
        }
    }
}

// --- 6. aria-haspopup は両 scope 常に "menu" ---

#[test]
fn haspopup_value_is_menu_in_both_scopes() {
    let cases: Vec<(&str, Node)> = vec![
        (
            "menu::trigger",
            menu::trigger(OpenState::Open, false, None, vec![], vec![]),
        ),
        (
            "menu::trigger_item",
            menu::trigger_item(OpenState::Open, false, false, None, vec![], vec![]),
        ),
        (
            "menubar::trigger",
            menubar::trigger(true, OpenState::Open, false, false, None, vec![], vec![]),
        ),
        (
            "menubar::sub_trigger",
            menubar::sub_trigger(OpenState::Open, false, false, None, vec![], vec![]),
        ),
    ];
    for (label, node) in cases {
        let html = fandhe_frontend_core::render(&node);
        assert!(
            html.contains(r#"aria-haspopup="menu""#),
            "{label}: aria-haspopup=\"menu\" が出力されていない"
        );
        assert!(
            !html.contains(r#"aria-haspopup="true""#),
            "{label}: aria-haspopup=\"true\" へフォールバックしてはならない"
        );
    }
}

// --- 7. role 語彙の共有（menubar 固有の menubar/none を除く） ---

#[test]
fn menu_role_vocabulary_is_shared() {
    let menu_roles = collect_roles(&sample_menu_tree());
    let menubar_roles = collect_roles(&sample_menubar_tree());

    let menubar_specific: BTreeMap<&str, &str> = [("menubar", "root"), ("none", "menu")]
        .into_iter()
        .collect();

    for (role, part) in &menubar_roles {
        if menubar_specific.get(role.as_str()) == Some(&part.as_str()) {
            continue;
        }
        assert!(
            menu_roles.contains_key(role),
            "menubar の role \"{role}\"（パーツ {part}）が menu 側の role 語彙に存在しない"
        );
    }
}

fn collect_roles(node: &Node) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    collect_roles_inner(node, &mut out);
    out
}

fn collect_roles_inner(node: &Node, out: &mut BTreeMap<String, String>) {
    if let Node::Element {
        attrs, children, ..
    } = node
    {
        let role = attrs
            .iter()
            .find(|(k, _)| k == "role")
            .map(|(_, v)| v.clone());
        let part = attrs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("data-part"))
            .map(|(_, v)| v.clone());
        if let (Some(role), Some(part)) = (role, part) {
            out.entry(role).or_insert(part);
        }
        for child in children {
            collect_roles_inner(child, out);
        }
    }
}

/// menu の全パーツを role 収集用に組み立てたサンプル木（checkbox_item /
/// radio_item を含む。開閉系以外の role も網羅する）。
fn sample_menu_tree() -> Node {
    use fandhe_frontend_core::text;

    let group = menu::item_group(
        Some("g1"),
        vec![],
        vec![
            menu::item_group_label(Some("g1"), vec![], vec![]),
            menu::item("item-1", false, false, vec![], vec![]),
        ],
    );
    let radio_group = menu::radio_item_group(
        None,
        vec![],
        vec![menu::radio_item(false, "r1", false, false, vec![], vec![])],
    );
    let content = menu::content(
        OpenState::Open,
        Some("content-1"),
        None,
        vec![],
        vec![
            group,
            menu::separator(vec![], vec![]),
            menu::trigger_item(OpenState::Closed, false, false, None, vec![], vec![]),
            menu::checkbox_item(false, "c1", false, false, vec![], vec![]),
            radio_group,
        ],
    );
    let positioner = menu::positioner(OpenState::Open, vec![], vec![content]);
    let trigger = menu::trigger(
        OpenState::Open,
        false,
        Some("content-1"),
        vec![],
        vec![text("Open")],
    );
    menu::root(OpenState::Open, vec![], vec![trigger, positioner])
}

// --- 8. 開閉系パーツの data-state 値語彙は両 scope で {open, closed} のみ ---
//
// menu の checkbox_item/radio_item は data-state に "checked"/"unchecked" を
// 出す（checked 状態機械であり開閉状態機械ではない）。このため本 assert は
// 「開閉を表すパーツ」に限定する（sample_menu_tree に含まれるが対象外とする
// パーツを明示することで、限定の意図をテストコード自体に残す）。

#[test]
fn open_close_parts_share_data_state_vocabulary() {
    for state in [OpenState::Open, OpenState::Closed] {
        let open_close_nodes: Vec<(&str, Node)> = vec![
            ("menu::root", menu::root(state, vec![], vec![])),
            (
                "menu::trigger",
                menu::trigger(state, false, None, vec![], vec![]),
            ),
            ("menu::indicator", menu::indicator(state, vec![], vec![])),
            ("menu::positioner", menu::positioner(state, vec![], vec![])),
            (
                "menu::content",
                menu::content(state, None, None, vec![], vec![]),
            ),
            (
                "menu::trigger_item",
                menu::trigger_item(state, false, false, None, vec![], vec![]),
            ),
            (
                "menu::context_trigger",
                menu::context_trigger(state, vec![], vec![]),
            ),
            ("menubar::menu", menubar::menu(state, vec![], vec![])),
            (
                "menubar::trigger",
                menubar::trigger(true, state, false, false, None, vec![], vec![]),
            ),
            (
                "menubar::positioner",
                menubar::positioner(state, vec![], vec![]),
            ),
            (
                "menubar::content",
                menubar::content(state, None, None, vec![], vec![]),
            ),
            (
                "menubar::sub_trigger",
                menubar::sub_trigger(state, false, false, None, vec![], vec![]),
            ),
            (
                "menubar::sub_content",
                menubar::sub_content(state, None, None, vec![], vec![]),
            ),
        ];
        for (label, node) in open_close_nodes {
            let html = fandhe_frontend_core::render(&node);
            let expected = format!(r#"data-state="{}""#, state.as_data_state());
            assert!(
                html.contains(&expected),
                "{label}: data-state が {expected} を含んでいない（state={state:?}）"
            );
            assert!(
                !html.contains(r#"data-state="checked""#)
                    && !html.contains(r#"data-state="unchecked""#),
                "{label}: 開閉系パーツに checked/unchecked 値語彙が混入している"
            );
        }
    }
}

// --- 9. A05: type="button" 固定 + menubar::trigger の呼び出し側 tabindex 偽装除去 ---

#[test]
fn type_button_is_fixed_and_caller_tabindex_spoof_is_dropped() {
    let menu_trigger_html = fandhe_frontend_core::render(&menu::trigger(
        OpenState::Closed,
        false,
        None,
        vec![],
        vec![],
    ));
    assert!(menu_trigger_html.contains(r#"type="button""#));

    let menubar_trigger_html = fandhe_frontend_core::render(&menubar::trigger(
        true,
        OpenState::Closed,
        false,
        false,
        None,
        vec![],
        vec![],
    ));
    assert!(menubar_trigger_html.contains(r#"type="button""#));

    // 呼び出し側が `tabindex="99"`（偽装）を渡しても、roving tabindex の
    // 合成値（focused=true → "0"）のみが出力され、呼び出し側の値は残らない。
    let spoofed = fandhe_frontend_core::render(&menubar::trigger(
        true,
        OpenState::Closed,
        false,
        false,
        None,
        vec![("tabindex", "99"), ("TABINDEX", "77")],
        vec![],
    ));
    assert!(spoofed.contains(r#"tabindex="0""#));
    assert!(!spoofed.contains(r#"tabindex="99""#));
    assert!(!spoofed.contains(r#"tabindex="77""#));
}

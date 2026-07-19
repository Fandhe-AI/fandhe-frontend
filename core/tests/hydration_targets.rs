//! `find_attr_values` / `find_nav_targets`（TASK-6.2b, `docs/api/hydration-api.md`
//! 第 3 節・公開 API 凍結表）のネイティブ回帰テスト。
//!
//! これらは `rws-wasm-client` の `hydrate()` がハイドレーション対象を特定する
//! ために呼ぶ契約の DOM 非依存純粋関数であり、wasm ビルドを介さずネイティブ
//! 環境で回帰確認できることが設計上の要点（`docs/api/hydration-api.md` 判断 3）。
//! 本ファイルはその契約（ネスト木の走査・属性欠落時の空配列・重複属性の
//! 全列挙・`data-nav` ショートカットの委譲関係）を固定する。

use rws_core::{a, div, el, find_attr_values, find_nav_targets, li, text, ul};

#[test]
fn find_attr_values_collects_nested_descendants_in_order() {
    let tree = div(
        vec![],
        vec![ul(
            vec![],
            vec![
                li(
                    vec![],
                    vec![a(vec![("data-nav", "/items/1")], vec![text("記事1")])],
                ),
                li(
                    vec![],
                    vec![a(vec![("data-nav", "/items/2")], vec![text("記事2")])],
                ),
            ],
        )],
    );
    assert_eq!(
        find_attr_values(&tree, "data-nav"),
        vec!["/items/1".to_string(), "/items/2".to_string()]
    );
}

#[test]
fn find_attr_values_returns_empty_when_attribute_absent() {
    let tree = div(vec![], vec![a(vec![("href", "/")], vec![text("home")])]);
    assert!(find_attr_values(&tree, "data-nav").is_empty());
}

#[test]
fn find_attr_values_lists_duplicate_attributes_in_occurrence_order() {
    // `el()` の attrs は Vec のため、理論上同名属性の重複指定があり得る。
    // 呼び出し側で重複除去はせず、出現順にすべて列挙することを固定する。
    let node = el(
        "div",
        vec![("data-hydrate", "like"), ("data-hydrate", "dislike")],
        vec![],
    );
    assert_eq!(
        find_attr_values(&node, "data-hydrate"),
        vec!["like".to_string(), "dislike".to_string()]
    );
}

#[test]
fn find_attr_values_ignores_text_and_raw_html_nodes() {
    #[expect(
        clippy::disallowed_methods,
        reason = "ESCAPE-REVIEWED: find_attr_values が RawHtml ノードを無視することの検証。固定の信頼済み文字列のみで外部入力を含まない"
    )]
    let tree = div(
        vec![],
        vec![text("<data-nav>"), rws_core::raw_html("<span>ok</span>")],
    );
    assert!(find_attr_values(&tree, "data-nav").is_empty());
}

#[test]
fn find_nav_targets_is_a_data_nav_shortcut_of_find_attr_values() {
    let tree = div(
        vec![],
        vec![
            a(vec![("data-nav", "/")], vec![text("一覧")]),
            a(vec![("data-hydrate", "like")], vec![text("いいね")]),
        ],
    );
    assert_eq!(find_nav_targets(&tree), find_attr_values(&tree, "data-nav"));
    assert_eq!(find_nav_targets(&tree), vec!["/".to_string()]);
}

#[test]
fn find_attr_values_finds_attr_on_root_element_itself() {
    // 子孫だけでなく自身（root）が対象属性を持つ場合も列挙対象に含む。
    let node = el(
        "button",
        vec![("id", "like-btn"), ("data-hydrate", "like")],
        vec![],
    );
    assert_eq!(
        find_attr_values(&node, "data-hydrate"),
        vec!["like".to_string()]
    );
}

//! `fandhe-frontend-headless-ui` の Item（[`item`] モジュール、イシュー
//! #2065）の公開 API 経由の統合テスト。
//!
//! `crates/headless-ui/src/item.rs` 内の `#[cfg(test)]` ユニットテストが
//! 内部実装を含めた網羅を担うのに対し、本ファイルは
//! `fandhe-frontend-headless-ui` の公開 API（`lib.rs` の再エクスポート
//! `pub mod item;`）のみを経由し、`fandhe-frontend-pre-styled-ui`
//! （styled ラッパー、#2066）が実際に使う想定の外部からの利用形態
//! （group > root（div）+ separator + root（a）の組み合わせ）を固定する
//! 回帰テスト（`tests/button_group.rs` と同型の位置付け）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::item::{self, ItemMediaVariant, ItemRootProps};

/// `group` > `root`（div）+ `separator` + `root`（a）の組み合わせで
/// 全 10 パーツの `data-part` が出現し、`role="group"` 側に
/// `aria-orientation` がなく、`a` 側に `role` がないことを固定する。
#[test]
fn group_with_two_items_and_separator_has_all_ten_parts() {
    let node = item::group(
        "Recent items",
        vec![],
        vec![
            item::root(
                ItemRootProps::default(),
                vec![],
                vec![
                    item::header(vec![], vec![text("New")]),
                    item::media(ItemMediaVariant::Icon, vec![], vec![]),
                    item::content(
                        vec![],
                        vec![
                            item::title(vec![], vec![text("First item")]),
                            item::description(vec![], vec![text("Description")]),
                        ],
                    ),
                    item::actions(
                        vec![],
                        vec![fandhe_frontend_core::button(
                            vec![("type", "button")],
                            vec![text("Open")],
                        )],
                    ),
                    item::footer(vec![], vec![text("Updated just now")]),
                ],
            ),
            item::separator(vec![], vec![]),
            item::root(
                ItemRootProps {
                    href: Some("/primitives/"),
                    ..Default::default()
                },
                vec![],
                vec![
                    item::media(ItemMediaVariant::Image, vec![], vec![]),
                    item::content(vec![], vec![item::title(vec![], vec![text("Second item")])]),
                ],
            ),
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="item""#));
    for part in [
        "root",
        "media",
        "content",
        "title",
        "description",
        "actions",
        "header",
        "footer",
        "group",
        "separator",
    ] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "part={part} が出力されていない: html={html}"
        );
    }

    assert!(html.contains(r#"role="group""#));
    assert!(html.contains(r#"role="separator""#));
    assert!(html.contains(r#"href="/primitives/""#));

    // `role="group"` 側に `aria-orientation` を付与しない不変条件
    // （`item` モジュール doc「`group` は `role="group"`」参照）。
    let group_start = html.find(r#"data-part="group""#).unwrap();
    let group_tag_end = html[group_start..].find('>').unwrap() + group_start;
    assert!(!html[group_start..group_tag_end].contains("aria-orientation"));

    // `a` 側に `role` を付与しない不変条件（モジュール doc「`root` を `a`
    // として描画する経路」参照）。
    let a_start = html.find("<a").unwrap();
    let a_tag_end = html[a_start..].find('>').unwrap() + a_start;
    assert!(!html[a_start..a_tag_end].contains("role="));

    // 状態機械を持たないため hydration 属性は一切出力されない。
    assert!(!html.contains("data-hydrate-"));
}

/// `ItemRootProps::default()` が `div`・`data-variant="default"`・
/// `data-size="default"` になることを公開 API 経由で固定する。
#[test]
fn default_root_props_render_default_div() {
    let html = render(&item::root(ItemRootProps::default(), vec![], vec![]));
    assert!(html.starts_with("<div"));
    assert!(html.contains(r#"data-variant="default""#));
    assert!(html.contains(r#"data-size="default""#));
}

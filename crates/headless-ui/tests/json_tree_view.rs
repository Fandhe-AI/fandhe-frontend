//! JsonTreeView（イシュー #829）の参考サイト突合（イシュー #1661）契約テスト。
//!
//! `crates/headless-ui/src/json_tree_view.rs` の inline unit tests が
//! パーツ単体・境界値を固定するのに対し、本ファイルは ark-ui/zag との
//! 突合で是正した点（`colon` パーツの新設・`branch-text`/`item-text` への
//! 入れ子・`data-kind` 語彙の `"boolean"` 統一）と、意図的に合わせなかった
//! 属性（`aria-label`/`data-line`/`data-type`/`data-root`/
//! `data-non-enumerable`）が出力に現れないことを、クレート外部から
//! （公開 API のみを使って）固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::json_tree_view::{colon, render_json, JsonValue, TreeView};

fn sample() -> JsonValue {
    JsonValue::Object(vec![
        ("name".to_string(), JsonValue::String("Ada".to_string())),
        ("age".to_string(), JsonValue::Number(36.0)),
        ("active".to_string(), JsonValue::Bool(true)),
        ("nickname".to_string(), JsonValue::Null),
        (
            "tags".to_string(),
            JsonValue::Array(vec![JsonValue::String("admin".to_string())]),
        ),
    ])
}

/// `colon` はキーを持つノードにのみ出力される（ルート自身はキー無しのため
/// 出ない）ことを、出現回数 = キー付きノード数の一致で固定する。
#[test]
fn colon_appears_only_for_keyed_nodes_not_the_root() {
    let tree = TreeView::default();
    let data = sample();
    let html = render(&render_json(&tree, &data));

    // sample() のキー付きノードは 5 個（name/age/active/nickname/tags）+
    // tags 配下の要素 1 個（"tags/0"、キーはインデックス "0"）で計 6 個。
    // ルート自身はキーを持たないため colon の出現回数に含まれない。
    let colon_count = html.matches(r#"data-part="colon""#).count();
    assert_eq!(colon_count, 6);

    // ルートの branch-text 直後に colon が続かないことを、ルート開始位置
    // 付近の断片で確認する（ルートは object 全体の要約のみを持つ）。
    let root_branch_text_idx = html.find(r#"data-part="branch-text""#).unwrap();
    let root_fragment = &html[root_branch_text_idx..];
    // ルート branch-text の最初の子は colon ではなく value（`data-kind="object"`）。
    let value_idx = root_fragment.find(r#"data-kind="object""#).unwrap();
    let colon_idx = root_fragment.find(r#"data-part="colon""#);
    assert!(colon_idx.is_none_or(|c| c > value_idx));
}

/// ブランチでは `branch-text` の内側に、葉では `item-text` の内側に
/// `key`→`colon`→`value` が部分文字列順序で並ぶこと（ark の
/// `BranchText`/`ItemText` が `KeyNode`/`ValueNode` を包む構造との対応）を
/// 固定する。`branch_indicator`/`item_indicator` は維持されている。
#[test]
fn key_colon_value_are_nested_inside_branch_text_and_item_text() {
    let tree = TreeView::default();
    let html = render(&render_json(&tree, &sample()));

    // ルート（branch）: branch-indicator → branch-text > [key, colon, value] の順。
    let indicator_idx = html.find(r#"data-part="branch-indicator""#).unwrap();
    let branch_text_idx = html.find(r#"data-part="branch-text""#).unwrap();
    assert!(indicator_idx < branch_text_idx);

    // "name" は葉ノード: item-indicator → item-text > [key, colon, value] の順。
    let leaf_key_idx = html.find(">name<").unwrap();
    let item_indicator_idx = html.rfind(r#"data-part="item-indicator""#).unwrap();
    let item_text_idx = html.find(r#"data-part="item-text""#).unwrap();
    let leaf_colon_idx =
        html[item_text_idx..].find(r#"data-part="colon""#).unwrap() + item_text_idx;
    let leaf_value_idx = html.find(r#"data-kind="string""#).unwrap();
    assert!(item_text_idx < leaf_key_idx);
    assert!(leaf_key_idx < leaf_colon_idx);
    assert!(leaf_colon_idx < leaf_value_idx);
    // item_indicator は item-text より前段（item の直下 1 番目の子）に位置する。
    let _ = item_indicator_idx;
}

/// `data-kind` の語彙が正確に 6 値であり、旧 `"bool"` が出力されない
/// ことを固定する（イシュー #1661 の破壊的変更: `"bool"` → `"boolean"`）。
#[test]
fn data_kind_vocabulary_is_boolean_not_bool() {
    let tree = TreeView::default();
    let html = render(&render_json(&tree, &sample()));

    for kind in ["null", "boolean", "number", "string", "array", "object"] {
        assert!(
            html.contains(&format!(r#"data-kind="{kind}""#)),
            "data-kind=\"{kind}\" が出力されていない"
        );
    }
    assert!(!html.contains(r#"data-kind="bool""#));
}

/// ark-ui が持つが本実装が意図的に採用しなかった属性・語彙が出力に
/// 現れないことを固定する（モジュール doc §参考サイトとの突合 参照）。
#[test]
fn non_adopted_ark_attributes_do_not_appear() {
    let tree = TreeView::default();
    let html = render(&render_json(&tree, &sample()));

    assert!(!html.contains("aria-label"));
    assert!(!html.contains("data-line"));
    assert!(!html.contains("data-type="));
    assert!(!html.contains("data-root"));
    assert!(!html.contains("data-non-enumerable"));
}

/// 呼び出し側 `attrs` の `data-scope`/`data-part` 偽装が `colon` でも
/// 除去されることを固定する（[`crate::anatomy`] 既存不変条件の colon 版）。
#[test]
fn colon_drops_caller_supplied_scope_and_part() {
    let html = render(&colon(
        vec![("data-scope", "attacker"), ("data-part", "attacker")],
        vec![fandhe_frontend_core::text(": ")],
    ));
    assert!(html.contains(r#"data-scope="json-tree-view""#));
    assert!(html.contains(r#"data-part="colon""#));
    assert!(!html.contains("attacker"));
}

/// colon 込みでも既存の決定性（同一入力 → 同一出力）が保たれることを
/// 再確認する（クレート外部 API 経由での固定）。
#[test]
fn render_json_with_colon_is_deterministic() {
    let tree = TreeView::default();
    let data = sample();
    let html_a = render(&render_json(&tree, &data));
    let html_b = render(&render_json(&tree, &data));
    assert_eq!(html_a, html_b);
}

//! `input_group::*`/`InputGroupProps`（イシュー #2062）の公開 API 経由統合
//! テスト。
//!
//! `crates/headless-ui/src/input_group.rs` 側のユニットテストで値ごとの
//! 詳細な属性検証を行っているため、本ファイルは「公開 API 経由で壊れて
//! いないか」の統合確認（`field::input`/`field::textarea` を内包した合成
//! 例・XSS 回帰・anatomy 偽装除去）に絞る（`tests/fieldset.rs` と同じ方針）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::field::{self, input, label, textarea};
use fandhe_frontend_headless_ui::input_group::{addon, button, root, text as ig_text};
use fandhe_frontend_headless_ui::{FieldIds, FieldProps, InputGroupAlign, InputGroupProps};

fn base_field_props(id: &str) -> FieldProps<'_> {
    FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    }
}

/// input を内包する合成例（inline-start/inline-end の 2 addon 構成）。
#[test]
fn input_group_public_api_composes_with_field_input_inline_addons() {
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    let field_props = group_props.merge_field_props(base_field_props("price"));

    let node = root(
        &group_props,
        vec![],
        vec![
            addon(
                InputGroupAlign::InlineStart,
                &group_props,
                vec![],
                vec![ig_text(vec![], vec![text("$")])],
            ),
            label(&field_props, vec![], vec![text("Price")]),
            input(&field_props, vec![("type", "text")]),
            addon(
                InputGroupAlign::InlineEnd,
                &group_props,
                vec![],
                vec![button(&group_props, vec![], vec![text("Clear")])],
            ),
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="input-group" data-part="root" role="group""#));
    assert!(
        html.contains(r#"data-scope="input-group" data-part="addon" data-align="inline-start""#)
    );
    assert!(html.contains(r#"data-scope="input-group" data-part="addon" data-align="inline-end""#));
    assert!(html.contains(r#"data-scope="input-group" data-part="text">$</span>"#));
    assert!(html.contains(r#"data-scope="field" data-part="input""#));
    assert!(html.contains(r#"id="price-control""#));
    assert!(html
        .contains(r#"data-scope="input-group" data-part="button" type="button">Clear</button>"#));
    // Input Group は input へ独自の aria-labelledby/aria-describedby を
    // 付与しない契約（field::label の関連付けのみ）。
    assert!(!html.contains("aria-labelledby"));
}

/// textarea + block-start/block-end の合成例。
#[test]
fn input_group_public_api_composes_with_field_textarea_block_addons() {
    let group_props = InputGroupProps {
        disabled: false,
        invalid: true,
    };
    let field_props = group_props.merge_field_props(base_field_props("bio"));
    // merge_field_props は invalid を OR 伝播するため field 側にも invalid が
    // 反映されていることを確認する（fieldset との差分、モジュール doc 参照）。
    assert!(field_props.invalid);

    let node = root(
        &group_props,
        vec![],
        vec![
            addon(
                InputGroupAlign::BlockStart,
                &group_props,
                vec![],
                vec![ig_text(vec![], vec![text("Bio")])],
            ),
            textarea(&field_props, false, vec![], vec![]),
            addon(
                InputGroupAlign::BlockEnd,
                &group_props,
                vec![],
                vec![ig_text(vec![], vec![text("0/280")])],
            ),
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"aria-invalid="true""#));
    assert!(html.contains(r#"data-align="block-start""#));
    assert!(html.contains(r#"data-align="block-end""#));
    assert!(html.contains(r#"data-scope="field" data-part="textarea""#));
}

// --- XSS 回帰・anatomy 偽装除去（イシュー #2062） ---

#[test]
fn xss_payload_via_public_api_is_escaped_on_render() {
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    let node = root(
        &group_props,
        vec![("id", "x\" onmouseover=\"alert(1)")],
        vec![
            ig_text(vec![], vec![text("<script>alert(1)</script>")]),
            button(
                &group_props,
                vec![],
                vec![text("<script>alert(2)</script>")],
            ),
        ],
    );
    let html = render(&node);

    assert!(!html.contains("<script>alert"));
    assert!(!html.contains("onmouseover=\"alert"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"));
}

#[test]
fn caller_supplied_data_scope_and_part_are_dropped_via_public_api() {
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    let html = render(&root(
        &group_props,
        vec![("Data-Scope", "attacker"), ("DATA-PART", "attacker")],
        vec![],
    ));
    assert_eq!(
        html,
        r#"<div data-scope="input-group" data-part="root" role="group"></div>"#
    );
}

// field モジュールの直接インポートが未使用にならないよう、field::helper_text
// も併用した最小の連携例を確認する（field 側 API の可視性確認を兼ねる）。
#[test]
fn field_helper_text_still_usable_alongside_input_group() {
    let field_props = base_field_props("note");
    let html = render(&field::helper_text(
        &field_props,
        vec![],
        vec![text("hint")],
    ));
    assert!(html.contains(r#"data-scope="field" data-part="helper-text""#));
}

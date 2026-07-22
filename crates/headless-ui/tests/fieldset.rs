//! `fieldset::*`/`FieldsetProps`（イシュー #602）の公開 API 経由統合テスト。
//!
//! `crates/headless-ui/src/fieldset.rs` 側のユニットテストで値ごとの詳細な
//! 属性検証を行っているため、本ファイルは「公開 API 経由で壊れていないか」
//! の統合確認（合成例・XSS 回帰・anatomy 偽装除去）に絞る
//! （`tests/field.rs` と同じ方針）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::field::{input, label};
use fandhe_frontend_headless_ui::fieldset::{error_text, helper_text, legend, root};
use fandhe_frontend_headless_ui::{FieldIds, FieldProps, FieldsetProps};

fn base_fieldset_props(id: &str) -> FieldsetProps<'_> {
    FieldsetProps {
        id,
        disabled: false,
        invalid: false,
        has_helper_text: false,
    }
}

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

#[test]
fn fieldset_public_api_is_usable_from_crate_root_and_renders_expected_scaffolding() {
    let mut fs_props = base_fieldset_props("address");
    fs_props.has_helper_text = true;

    let field_props = fs_props.merge_field_props(base_field_props("street"));

    let node = root(
        &fs_props,
        vec![],
        vec![
            legend(&fs_props, vec![], vec![text("Address")]),
            helper_text(&fs_props, vec![], vec![text("Shipping address only.")]),
            label(&field_props, vec![], vec![text("Street")]),
            input(&field_props, vec![("type", "text")]),
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="fieldset" data-part="root""#));
    assert!(html.contains(r#"data-part="legend""#));
    assert!(html.contains(r#"id="address-legend""#));
    assert!(html.contains(r#"aria-describedby="address-helper-text""#));
    assert!(html.contains(r#"data-scope="field" data-part="input""#));
    assert!(html.contains(r#"id="street-control""#));
}

#[test]
fn merge_field_props_or_propagates_disabled_into_composed_field() {
    let mut fs_props = base_fieldset_props("group");
    fs_props.disabled = true;
    let field_props = fs_props.merge_field_props(base_field_props("email"));

    let html = render(&input(&field_props, vec![]));
    assert!(html.contains(r#"disabled="""#));
    assert!(html.contains(r#"data-disabled=""#));
}

#[test]
fn merge_field_props_both_disabled_false_stays_enabled() {
    let fs_props = base_fieldset_props("group");
    let field_props = fs_props.merge_field_props(base_field_props("email"));

    let html = render(&input(&field_props, vec![]));
    assert!(!html.contains("disabled"));
}

#[test]
fn merge_field_props_does_not_propagate_invalid_to_field() {
    let mut fs_props = base_fieldset_props("group");
    fs_props.invalid = true;
    let field_props = fs_props.merge_field_props(base_field_props("email"));

    let html = render(&input(&field_props, vec![]));
    assert!(!html.contains("aria-invalid"));
}

// --- XSS 回帰: id・children に攻撃者制御文字列が入っても既定エスケープが効くこと、
// および anatomy の data-scope/data-part 偽装が fail-closed で除去されることを固定する ---

#[test]
fn xss_payload_in_id_and_children_is_escaped() {
    let payload_id = "x\" onmouseover=\"alert(1)";
    let props = base_fieldset_props(payload_id);
    let node = root(
        &props,
        vec![],
        vec![
            legend(&props, vec![], vec![text("<script>alert(1)</script>")]),
            helper_text(&props, vec![], vec![text("<script>alert(2)</script>")]),
        ],
    );
    let html = render(&node);
    assert!(!html.contains("<script>alert"));
    assert!(!html.contains("onmouseover=\"alert"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_supplied_data_scope_and_part_forgery_is_dropped_fail_closed() {
    let props = base_fieldset_props("f");
    let node = root(
        &props,
        vec![("Data-Scope", "attacker"), ("DATA-PART", "attacker")],
        vec![],
    );
    assert_eq!(
        render(&node),
        r#"<fieldset data-scope="fieldset" data-part="root"></fieldset>"#
    );
}

#[test]
fn error_text_fail_closed_hidden_default() {
    let props = base_fieldset_props("f");
    let html = render(&error_text(&props, vec![], vec![text("bad")]));
    assert!(html.contains(r#"hidden="""#));
    assert!(html.contains(r#"aria-live="polite""#));

    let mut props_invalid = base_fieldset_props("f");
    props_invalid.invalid = true;
    let visible_html = render(&error_text(&props_invalid, vec![], vec![text("bad")]));
    assert!(!visible_html.contains("hidden"));
}

//! `field::*`/`FieldProps`（イシュー #538）の公開 API 経由統合テスト。
//!
//! クレートルート・`field` モジュールからの re-export が実際に使えることを
//! 確認したうえで、代表スナップショット・data-*/ARIA 属性の網羅・
//! `aria-describedby` 合成則・XSS 回帰を固定する。値ごとの詳細な属性検証は
//! `crates/headless-ui/src/field.rs` 側のユニットテストに置き、本ファイルは
//! 「公開 API 経由で壊れていないか」の統合確認に絞る（`tabs_public_api.rs`
//! と同じ方針）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::field::{
    error_text, helper_text, input, label, required_indicator, root, select, textarea,
};
use fandhe_frontend_headless_ui::FieldProps;

fn base_props(id: &str) -> FieldProps<'_> {
    FieldProps {
        id,
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    }
}

#[test]
fn field_public_api_is_usable_from_crate_root_and_renders_expected_scaffolding() {
    let mut props = base_props("email");
    props.required = true;
    props.has_helper_text = true;

    let node = root(
        &props,
        vec![],
        vec![
            label(&props, vec![], vec![text("Email")]),
            required_indicator(&props, vec![], vec![text("*")]),
            input(&props, vec![("type", "email")]),
            helper_text(&props, vec![], vec![text("We never share your email.")]),
            error_text(&props, vec![], vec![text("Invalid email address.")]),
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="field" data-part="root""#));
    assert!(html.contains(r#"data-required=""#));
    assert!(html.contains(r#"for="email-control""#));
    assert!(html.contains(r#"id="email-control""#));
    assert!(html.contains(r#"required=""#));
    assert!(html.contains(r#"aria-describedby="email-helper-text""#));
    assert!(html.contains(r#"aria-hidden="true""#));
    assert!(html.contains(r#"aria-live="polite""#));
    // required なので required_indicator（aria-hidden="true")は非表示にならない。
    assert!(html.contains(r#"data-part="required_indicator""#));
    // error_text は invalid でないため hidden。一方 required_indicator は
    // required のため hidden ではない（両パーツの hidden 出現数は 1 のみ）。
    assert_eq!(html.matches(r#"hidden="""#).count(), 1);
}

#[test]
fn data_attrs_reflect_all_four_flags_across_parts() {
    let mut props = base_props("f");
    props.disabled = true;
    props.invalid = true;
    props.required = true;
    props.readonly = true;

    let root_html = render(&root(&props, vec![], vec![]));
    assert!(root_html.contains(r#"data-disabled=""#));
    assert!(root_html.contains(r#"data-invalid=""#));
    assert!(root_html.contains(r#"data-required=""#));
    assert!(root_html.contains(r#"data-readonly=""#));

    let input_html = render(&input(&props, vec![]));
    assert!(input_html.contains(r#"disabled=""#));
    assert!(input_html.contains(r#"required=""#));
    assert!(input_html.contains(r#"readonly=""#));
    assert!(input_html.contains(r#"aria-invalid="true""#));
    assert!(input_html.contains(r#"data-disabled=""#));
    assert!(input_html.contains(r#"data-invalid=""#));
    assert!(input_html.contains(r#"data-required=""#));
    assert!(input_html.contains(r#"data-readonly=""#));
}

#[test]
fn data_attrs_are_absent_when_all_flags_false() {
    let props = base_props("f");
    let root_html = render(&root(&props, vec![], vec![]));
    assert!(!root_html.contains("data-disabled"));
    assert!(!root_html.contains("data-invalid"));
    assert!(!root_html.contains("data-required"));
    assert!(!root_html.contains("data-readonly"));

    let input_html = render(&input(&props, vec![]));
    assert!(!input_html.contains("disabled"));
    assert!(!input_html.contains("required"));
    assert!(!input_html.contains("readonly"));
    assert!(!input_html.contains("aria-invalid"));
}

#[test]
fn textarea_and_select_parts_render_expected_scopes() {
    let props = base_props("bio");
    let ta_html = render(&textarea(&props, vec![], vec![text("hello")]));
    assert!(ta_html.contains(r#"data-scope="field" data-part="textarea""#));
    assert!(ta_html.contains(r#"id="bio-control""#));
    assert!(ta_html.contains("hello"));

    let sel_html = render(&select(&props, vec![], vec![]));
    assert!(sel_html.contains(r#"data-scope="field" data-part="select""#));
    assert!(sel_html.contains(r#"id="bio-control""#));
}

#[test]
fn describedby_composition_three_cases() {
    // ケース 1: helper のみ。
    let mut props_helper_only = base_props("f");
    props_helper_only.has_helper_text = true;
    let html = render(&input(&props_helper_only, vec![]));
    assert!(html.contains(r#"aria-describedby="f-helper-text""#));

    // ケース 2: invalid + helper（error id が先頭）。
    let mut props_both = base_props("f");
    props_both.invalid = true;
    props_both.has_helper_text = true;
    let html = render(&input(&props_both, vec![]));
    assert!(html.contains(r#"aria-describedby="f-error-text f-helper-text""#));

    // ケース 3: どちらも無し → 属性自体を出力しない。
    let props_none = base_props("f");
    let html = render(&input(&props_none, vec![]));
    assert!(!html.contains("aria-describedby"));
}

#[test]
fn error_text_and_required_indicator_fail_closed_hidden_defaults() {
    let props = base_props("f");
    let error_html = render(&error_text(&props, vec![], vec![text("bad")]));
    assert!(error_html.contains(r#"hidden="""#));
    assert!(error_html.contains(r#"aria-live="polite""#));

    let indicator_html = render(&required_indicator(&props, vec![], vec![text("*")]));
    assert!(indicator_html.contains(r#"hidden="""#));
    assert!(indicator_html.contains(r#"aria-hidden="true""#));
}

// --- XSS 回帰: id・呼び出し側 attrs・children に攻撃者制御文字列が入っても
// 既定エスケープが効くこと、および anatomy の data-scope/data-part 偽装が
// fail-closed で除去されることを固定する ---

#[test]
fn xss_payload_in_id_and_caller_supplied_attrs_and_children_is_escaped() {
    let payload_id = "x\" onmouseover=\"alert(1)";
    let props = base_props(payload_id);
    let node = root(
        &props,
        vec![],
        vec![
            label(&props, vec![], vec![text("<script>alert(1)</script>")]),
            input(
                &props,
                vec![("value", "<script>alert(2)</script>\" onfocus=\"alert(3)")],
            ),
        ],
    );
    let html = render(&node);
    assert!(!html.contains("<script>alert"));
    assert!(!html.contains("onmouseover=\"alert"));
    assert!(!html.contains("onfocus=\"alert"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_supplied_data_scope_and_part_forgery_is_dropped_fail_closed() {
    let props = base_props("f");
    let node = root(
        &props,
        vec![("Data-Scope", "attacker"), ("DATA-PART", "attacker")],
        vec![],
    );
    assert_eq!(
        render(&node),
        r#"<div data-scope="field" data-part="root"></div>"#
    );
}

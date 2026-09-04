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
use fandhe_frontend_headless_ui::{FieldIds, FieldProps};

fn base_props(id: &str) -> FieldProps<'_> {
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
    assert!(html.contains(r#"data-part="required-indicator""#));
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
    let ta_html = render(&textarea(&props, false, vec![], vec![text("hello")]));
    assert!(ta_html.contains(r#"data-scope="field" data-part="textarea""#));
    assert!(ta_html.contains(r#"id="bio-control""#));
    assert!(ta_html.contains("hello"));

    let sel_html = render(&select(&props, vec![], vec![]));
    assert!(sel_html.contains(r#"data-scope="field" data-part="select""#));
    assert!(sel_html.contains(r#"id="bio-control""#));
}

#[test]
fn select_does_not_emit_native_readonly_while_input_and_textarea_do() {
    let mut props = base_props("bio");
    props.readonly = true;

    let sel_html = render(&select(&props, vec![], vec![]));
    // `data-readonly=""` 自体に部分文字列として `readonly=""` を含むため、
    // ネイティブ属性は先頭空白付きで区別して検証する。
    assert!(!sel_html.contains(r#" readonly="""#));
    assert!(sel_html.contains(r#"data-readonly=""#));

    let input_html = render(&input(&props, vec![]));
    assert!(input_html.contains(r#"readonly="""#));

    let ta_html = render(&textarea(&props, false, vec![], vec![]));
    assert!(ta_html.contains(r#"readonly="""#));
}

#[test]
fn textarea_autoresize_flag_toggles_data_autoresize() {
    let props = base_props("bio");
    let with_autoresize = render(&textarea(&props, true, vec![], vec![]));
    assert!(with_autoresize.contains(r#"data-autoresize=""#));

    let without_autoresize = render(&textarea(&props, false, vec![], vec![]));
    assert!(!without_autoresize.contains("data-autoresize"));
}

#[test]
fn field_ids_override_propagates_across_public_api() {
    let mut props = base_props("f");
    props.ids.root = Some("custom-root");
    props.ids.control = Some("custom-control");
    props.ids.label = Some("custom-label");
    props.ids.helper_text = Some("custom-helper");
    props.ids.error_text = Some("custom-error");
    props.invalid = true;
    props.has_helper_text = true;

    let root_html = render(&root(&props, vec![], vec![]));
    assert!(root_html.contains(r#"id="custom-root""#));

    let label_html = render(&label(&props, vec![], vec![text("Name")]));
    assert!(label_html.contains(r#"for="custom-control""#));
    assert!(label_html.contains(r#"id="custom-label""#));

    let input_html = render(&input(&props, vec![]));
    assert!(input_html.contains(r#"id="custom-control""#));
    assert!(input_html.contains(r#"aria-describedby="custom-error custom-helper""#));

    let helper_html = render(&helper_text(&props, vec![], vec![text("hint")]));
    assert!(helper_html.contains(r#"id="custom-helper""#));

    let error_html = render(&error_text(&props, vec![], vec![text("bad")]));
    assert!(error_html.contains(r#"id="custom-error""#));
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

// --- 参考サイト（ark-ui Field / chakra-ui Field / Radix Primitives Label）
// 突合契約（イシュー #1607） ---
//
// 上記のテスト群が `field::*`（純粋関数群、#538/#602）の公開 API 経由の
// 挙動を固定するのに対し、以下は anatomy（8 パーツ構成）・`data-*` 語彙・
// ARIA・「装飾/レイアウト計測の関心を持ち込まない」不変条件が参照サイトと
// 一致し続けることを fail-closed に固定する（`tests/checkbox.rs` 「参考
// サイト突合契約（イシュー #1602）」節と同型の趣旨）。差分調査の詳細は
// イシュー #1607 コメント参照。

/// 8 パーツ（root/label/input/textarea/select/helper-text/error-text/
/// required-indicator）をすべて描画し、`data-scope="field"` の存在と
/// `data-part` の集合が ark-ui の Field anatomy と一致することを固定する。
#[test]
fn reference_anatomy_part_names_match_ark_ui() {
    let props = base_props("f");
    let parts: [(&str, String); 8] = [
        ("root", render(&root(&props, vec![], vec![]))),
        ("label", render(&label(&props, vec![], vec![text("L")]))),
        ("input", render(&input(&props, vec![]))),
        ("textarea", render(&textarea(&props, false, vec![], vec![]))),
        ("select", render(&select(&props, vec![], vec![]))),
        (
            "helper-text",
            render(&helper_text(&props, vec![], vec![text("H")])),
        ),
        (
            "error-text",
            render(&error_text(&props, vec![], vec![text("E")])),
        ),
        (
            "required-indicator",
            render(&required_indicator(&props, vec![], vec![text("*")])),
        ),
    ];
    for (part, html) in &parts {
        assert!(
            html.contains(r#"data-scope="field""#),
            "{part} が data-scope=\"field\" を持たない: {html}"
        );
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "{part} が期待する data-part を持たない: {html}"
        );
    }
}

/// zag.js Field の `dataAttrs`（`disabled`/`invalid`/`required`/`readonly`
/// の存在属性）は 8 パーツすべてへ一貫して付与される規約であることを固定
/// する（`crate::field::state_data_attrs` が root/label/helper_text/
/// error_text/required_indicator/コントロール全てへ委譲する不変条件）。
#[test]
fn all_eight_parts_reflect_all_four_data_flags() {
    let mut props = base_props("f");
    props.disabled = true;
    props.invalid = true;
    props.required = true;
    props.readonly = true;

    let parts: [(&str, String); 8] = [
        ("root", render(&root(&props, vec![], vec![]))),
        ("label", render(&label(&props, vec![], vec![text("L")]))),
        ("input", render(&input(&props, vec![]))),
        ("textarea", render(&textarea(&props, false, vec![], vec![]))),
        ("select", render(&select(&props, vec![], vec![]))),
        (
            "helper-text",
            render(&helper_text(&props, vec![], vec![text("H")])),
        ),
        (
            "error-text",
            render(&error_text(&props, vec![], vec![text("E")])),
        ),
        (
            "required-indicator",
            render(&required_indicator(&props, vec![], vec![text("*")])),
        ),
    ];
    for (part, html) in &parts {
        assert!(html.contains(r#"data-disabled=""#), "{part}: {html}");
        assert!(html.contains(r#"data-invalid=""#), "{part}: {html}");
        assert!(html.contains(r#"data-required=""#), "{part}: {html}");
        assert!(html.contains(r#"data-readonly=""#), "{part}: {html}");
    }
}

/// 上記の逆側: 4 フラグすべて `false` のとき、8 パーツいずれも 4 種の
/// `data-*` を出力しないことを固定する。
#[test]
fn all_eight_parts_omit_data_flags_when_false() {
    let props = base_props("f");
    let parts: [(&str, String); 8] = [
        ("root", render(&root(&props, vec![], vec![]))),
        ("label", render(&label(&props, vec![], vec![text("L")]))),
        ("input", render(&input(&props, vec![]))),
        ("textarea", render(&textarea(&props, false, vec![], vec![]))),
        ("select", render(&select(&props, vec![], vec![]))),
        (
            "helper-text",
            render(&helper_text(&props, vec![], vec![text("H")])),
        ),
        (
            "error-text",
            render(&error_text(&props, vec![], vec![text("E")])),
        ),
        (
            "required-indicator",
            render(&required_indicator(&props, vec![], vec![text("*")])),
        ),
    ];
    for (part, html) in &parts {
        assert!(!html.contains("data-disabled"), "{part}: {html}");
        assert!(!html.contains("data-invalid"), "{part}: {html}");
        assert!(!html.contains("data-required"), "{part}: {html}");
        assert!(!html.contains("data-readonly"), "{part}: {html}");
    }
}

/// 参照 headless（ark-ui/zag.js）の Field は `data-state`/`data-orientation`/
/// `data-motion`/pointer・focus ローカル状態を持たない。chakra-ui の
/// `orientation` バリアントは styled 層の関心（`docs/policy/
/// intentional-non-adoption.md` §3.25 規則 2）であり headless へ持ち込まない。
/// Radix `Form.Field` の `data-valid` も、`Form` 自体が同 §3.25 規則 1 で
/// 不採用のため導入しない（ark-ui にも存在しない）。
#[test]
fn no_part_outputs_state_orientation_motion_or_pointer_attrs() {
    let mut props = base_props("f");
    props.disabled = true;
    props.invalid = true;
    props.required = true;
    props.readonly = true;

    let parts: [(&str, String); 8] = [
        ("root", render(&root(&props, vec![], vec![]))),
        ("label", render(&label(&props, vec![], vec![text("L")]))),
        ("input", render(&input(&props, vec![]))),
        ("textarea", render(&textarea(&props, false, vec![], vec![]))),
        ("select", render(&select(&props, vec![], vec![]))),
        (
            "helper-text",
            render(&helper_text(&props, vec![], vec![text("H")])),
        ),
        (
            "error-text",
            render(&error_text(&props, vec![], vec![text("E")])),
        ),
        (
            "required-indicator",
            render(&required_indicator(&props, vec![], vec![text("*")])),
        ),
    ];
    for (part, html) in &parts {
        assert!(!html.contains("data-state"), "{part}: {html}");
        assert!(!html.contains("data-orientation"), "{part}: {html}");
        assert!(!html.contains("data-motion"), "{part}: {html}");
        assert!(!html.contains("data-hover"), "{part}: {html}");
        assert!(!html.contains("data-active"), "{part}: {html}");
        assert!(
            !html.contains("data-focus=\"") && !html.contains("data-focus \""),
            "{part}: {html}"
        );
        assert!(!html.contains("data-valid"), "{part}: {html}");
        assert!(!html.contains("data-placement"), "{part}: {html}");
    }
}

/// 8 パーツいずれもネイティブ要素の暗黙ロールに依存し、明示 `role` 属性を
/// 持たないことを固定する（zag.js Field と一致）。
#[test]
fn no_part_outputs_explicit_role() {
    let mut props = base_props("f");
    props.disabled = true;
    props.invalid = true;
    props.required = true;
    props.readonly = true;

    let parts: [(&str, String); 8] = [
        ("root", render(&root(&props, vec![], vec![]))),
        ("label", render(&label(&props, vec![], vec![text("L")]))),
        ("input", render(&input(&props, vec![]))),
        ("textarea", render(&textarea(&props, false, vec![], vec![]))),
        ("select", render(&select(&props, vec![], vec![]))),
        (
            "helper-text",
            render(&helper_text(&props, vec![], vec![text("H")])),
        ),
        (
            "error-text",
            render(&error_text(&props, vec![], vec![text("E")])),
        ),
        (
            "required-indicator",
            render(&required_indicator(&props, vec![], vec![text("*")])),
        ),
    ];
    for (part, html) in &parts {
        assert!(!html.contains(" role=\""), "{part}: {html}");
    }
}

/// input/textarea/select の 3 コントロールが zag.js Field のネイティブ属性則
/// （`disabled`/`required`/`readonly`〔select は除く〕存在属性・
/// `aria-invalid="true"`・`aria-describedby` 合成則）に従うことをまとめて
/// 固定する（`select_does_not_emit_native_readonly_while_input_and_textarea_do`
/// が単体で固定する select readonly 除外規則も含めて再確認する）。
#[test]
fn native_control_attrs_follow_zag_field_rules() {
    let mut props = base_props("f");
    props.disabled = true;
    props.invalid = true;
    props.required = true;
    props.readonly = true;
    props.has_helper_text = true;

    let input_html = render(&input(&props, vec![]));
    assert!(input_html.contains(r#"disabled=""#));
    assert!(input_html.contains(r#"required=""#));
    assert!(input_html.contains(r#" readonly="""#));
    assert!(input_html.contains(r#"aria-invalid="true""#));
    assert!(input_html.contains(r#"aria-describedby="f-error-text f-helper-text""#));

    let textarea_html = render(&textarea(&props, false, vec![], vec![]));
    assert!(textarea_html.contains(r#"disabled=""#));
    assert!(textarea_html.contains(r#"required=""#));
    assert!(textarea_html.contains(r#" readonly="""#));
    assert!(textarea_html.contains(r#"aria-invalid="true""#));
    assert!(textarea_html.contains(r#"aria-describedby="f-error-text f-helper-text""#));

    let select_html = render(&select(&props, vec![], vec![]));
    assert!(select_html.contains(r#"disabled=""#));
    assert!(select_html.contains(r#"required=""#));
    assert!(!select_html.contains(r#" readonly="""#));
    assert!(select_html.contains(r#"aria-invalid="true""#));
    assert!(select_html.contains(r#"aria-describedby="f-error-text f-helper-text""#));
}

/// `label` の `for` と input/textarea/select の `id` が同一 props で
/// 決定的に一致すること（`label[for]` ↔ control `id` の対応）を、3 コント
/// ロールすべてについて固定する。
#[test]
fn label_for_and_control_id_pair_across_all_three_controls() {
    let props = base_props("f");
    let label_html = render(&label(&props, vec![], vec![text("L")]));
    assert!(label_html.contains(r#"for="f-control""#));

    assert!(render(&input(&props, vec![])).contains(r#"id="f-control""#));
    assert!(render(&textarea(&props, false, vec![], vec![])).contains(r#"id="f-control""#));
    assert!(render(&select(&props, vec![], vec![])).contains(r#"id="f-control""#));
}

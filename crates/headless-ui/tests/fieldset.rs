//! `fieldset::*`/`FieldsetProps`（イシュー #602）の公開 API 経由統合テスト。
//!
//! `crates/headless-ui/src/fieldset.rs` 側のユニットテストで値ごとの詳細な
//! 属性検証を行っているため、本ファイルは「公開 API 経由で壊れていないか」
//! の統合確認（合成例・XSS 回帰・anatomy 偽装除去）に絞る
//! （`tests/field.rs` と同じ方針）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::field::{self, input, label};
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

// --- 参考サイト突合契約（イシュー #1608） ---
//
// 上記のテスト群が `fieldset::*`（純粋関数群、#602）の公開 API 経由の
// 挙動を固定するのに対し、以下は anatomy（4 パーツ構成）・`data-*` 語彙・
// ARIA・「装飾/レイアウト計測の関心を持ち込まない」不変条件が参照サイト
// （ark-ui Fieldset / chakra-ui Fieldset。Radix Primitives に Fieldset は
// 存在しない）と一致し続けることを fail-closed に固定する（`tests/field.rs`
// 「突合契約（イシュー #1607）」節と同型の趣旨）。差分調査の詳細はイシュー
// #1608 コメント参照。

/// 4 パーツ（root/legend/helper-text/error-text）をすべて描画し、
/// `data-scope="fieldset"` の存在と `data-part` の集合が ark-ui の Fieldset
/// anatomy と一致することを固定する。
#[test]
fn reference_anatomy_part_names_match_ark_ui() {
    let props = base_fieldset_props("f");
    let parts: [(&str, String); 4] = [
        ("root", render(&root(&props, vec![], vec![]))),
        ("legend", render(&legend(&props, vec![], vec![text("L")]))),
        (
            "helper-text",
            render(&helper_text(&props, vec![], vec![text("H")])),
        ),
        (
            "error-text",
            render(&error_text(&props, vec![], vec![text("E")])),
        ),
    ];
    for (part, html) in &parts {
        assert!(
            html.contains(r#"data-scope="fieldset""#),
            "{part} が data-scope=\"fieldset\" を持たない: {html}"
        );
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "{part} が期待する data-part を持たない: {html}"
        );
    }
}

/// `disabled`/`invalid` の存在属性（`data-disabled`/`data-invalid`）は
/// 4 パーツすべてへ一貫して付与される規約であることを固定する
/// （`crate::fieldset::state_data_attrs` が root/legend/helper_text/
/// error_text 全てへ委譲する不変条件）。
#[test]
fn all_four_parts_reflect_both_data_flags() {
    let mut props = base_fieldset_props("f");
    props.disabled = true;
    props.invalid = true;

    let parts: [(&str, String); 4] = [
        ("root", render(&root(&props, vec![], vec![]))),
        ("legend", render(&legend(&props, vec![], vec![text("L")]))),
        (
            "helper-text",
            render(&helper_text(&props, vec![], vec![text("H")])),
        ),
        (
            "error-text",
            render(&error_text(&props, vec![], vec![text("E")])),
        ),
    ];
    for (part, html) in &parts {
        assert!(html.contains(r#"data-disabled=""#), "{part}: {html}");
        assert!(html.contains(r#"data-invalid=""#), "{part}: {html}");
    }
}

/// 上記の逆側: 両フラグとも `false` のとき、4 パーツいずれも 2 種の
/// `data-*` を出力しないことを固定する。
#[test]
fn all_four_parts_omit_data_flags_when_false() {
    let props = base_fieldset_props("f");
    let parts: [(&str, String); 4] = [
        ("root", render(&root(&props, vec![], vec![]))),
        ("legend", render(&legend(&props, vec![], vec![text("L")]))),
        (
            "helper-text",
            render(&helper_text(&props, vec![], vec![text("H")])),
        ),
        (
            "error-text",
            render(&error_text(&props, vec![], vec![text("E")])),
        ),
    ];
    for (part, html) in &parts {
        assert!(!html.contains("data-disabled"), "{part}: {html}");
        assert!(!html.contains("data-invalid"), "{part}: {html}");
    }
}

/// 参照 headless（ark-ui/zag.js）の Fieldset は `data-state`/`data-orientation`/
/// `data-motion`/pointer・focus ローカル状態を持たない。chakra-ui の
/// `Fieldset.Content`（レイアウト用ラッパー）は styled 層の関心
/// （`docs/policy/intentional-non-adoption.md` §3.25 規則 2）であり headless
/// へ持ち込まない。
#[test]
fn no_part_outputs_state_orientation_motion_or_pointer_attrs() {
    let mut props = base_fieldset_props("f");
    props.disabled = true;
    props.invalid = true;

    let parts: [(&str, String); 4] = [
        ("root", render(&root(&props, vec![], vec![]))),
        ("legend", render(&legend(&props, vec![], vec![text("L")]))),
        (
            "helper-text",
            render(&helper_text(&props, vec![], vec![text("H")])),
        ),
        (
            "error-text",
            render(&error_text(&props, vec![], vec![text("E")])),
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

/// 4 パーツいずれもネイティブ要素（`<fieldset>`/`<legend>`/`<span>`）の
/// 暗黙ロールに依存し、明示 `role` 属性を持たないことを固定する
/// （ark-ui Fieldset と一致）。
#[test]
fn no_part_outputs_explicit_role() {
    let mut props = base_fieldset_props("f");
    props.disabled = true;
    props.invalid = true;

    let parts: [(&str, String); 4] = [
        ("root", render(&root(&props, vec![], vec![]))),
        ("legend", render(&legend(&props, vec![], vec![text("L")]))),
        (
            "helper-text",
            render(&helper_text(&props, vec![], vec![text("H")])),
        ),
        (
            "error-text",
            render(&error_text(&props, vec![], vec![text("E")])),
        ),
    ];
    for (part, html) in &parts {
        assert!(!html.contains(" role=\""), "{part}: {html}");
    }
}

/// `root`（`<fieldset>`）がネイティブ `disabled` 存在属性・
/// `aria-describedby` の合成則（ark-ui Fieldset と同型: invalid のとき
/// error id 先頭、has_helper_text のとき helper id を続ける）に従うことを
/// まとめて固定する。
#[test]
fn root_native_disabled_and_describedby_follow_ark_fieldset_rules() {
    let mut props = base_fieldset_props("f");
    props.disabled = true;
    props.invalid = true;
    props.has_helper_text = true;
    let html = render(&root(&props, vec![], vec![]));
    assert!(html.contains(r#"disabled="""#));
    assert!(html.contains(r#"aria-describedby="f-error-text f-helper-text""#));

    // helper のみ。
    let mut props_helper_only = base_fieldset_props("f");
    props_helper_only.has_helper_text = true;
    let html_helper_only = render(&root(&props_helper_only, vec![], vec![]));
    assert!(html_helper_only.contains(r#"aria-describedby="f-helper-text""#));

    // どちらも無し → 属性自体を出力しない。
    let props_none = base_fieldset_props("f");
    let html_none = render(&root(&props_none, vec![], vec![]));
    assert!(!html_none.contains("aria-describedby"));
}

/// `error_text` の可視性・`aria-live` が ark-ui Fieldset の ErrorText と
/// 一致することを固定する（fail-closed: 非 invalid で `hidden`）。
#[test]
fn error_text_visibility_and_live_region_match_ark_fieldset() {
    let props = base_fieldset_props("f");
    let hidden_html = render(&error_text(&props, vec![], vec![text("bad")]));
    assert!(hidden_html.contains(r#"hidden="""#));
    assert!(hidden_html.contains(r#"aria-live="polite""#));

    let mut props_invalid = base_fieldset_props("f");
    props_invalid.invalid = true;
    let visible_html = render(&error_text(&props_invalid, vec![], vec![text("bad")]));
    assert!(!visible_html.contains("hidden"));
}

/// `merge_field_props` で合成した [`FieldProps`] を `field::root` で描画
/// すると `data-disabled` が出ることを固定する（Demo の disabled インスタ
/// ンスが依拠する挙動、イシュー #1608）。
#[test]
fn merge_field_props_keeps_nested_field_data_disabled_consistent() {
    let mut fs_props = base_fieldset_props("group");
    fs_props.disabled = true;
    let field_props = fs_props.merge_field_props(base_field_props("email"));

    let html = render(&field::root(&field_props, vec![], vec![]));
    assert!(html.contains(r#"data-disabled=""#));
}

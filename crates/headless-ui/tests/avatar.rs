//! Avatar（イシュー #543）の統合テスト。
//!
//! `crates/headless-ui/src/avatar.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは「root > image + fallback」の
//! 組み立て全体の data-* 出力・dispatch 統合・SSR/hydration 両経路・XSS 回帰を
//! クレート外部から（公開 API のみを使って）固定する。

use fandhe_frontend_core::{render, text, Node};
use fandhe_frontend_headless_ui::avatar::{self, Avatar, ImageStatus};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_root_image_and_fallback() {
    let image = avatar::image(ImageStatus::Loaded, "/avatar.png", "Naoko Miyazaki", vec![]);
    let fallback = avatar::fallback(ImageStatus::Loaded, vec![], vec![text("NM")]);
    let root = avatar::root(vec![], vec![image, fallback]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="avatar""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="image""#));
    assert!(html.contains(r#"data-part="fallback""#));
    assert!(html.contains(r#"src="/avatar.png""#));
    assert!(html.contains(r#"alt="Naoko Miyazaki""#));
    assert!(html.contains(r#"data-state="visible""#)); // image
    assert!(html.contains(r#"data-state="hidden""#)); // fallback
    assert!(html.contains("NM"));
}

#[test]
fn dispatch_transitions_flip_visibility_across_parts() {
    let mut a = Avatar::default();
    assert_eq!(a.status(), ImageStatus::Loading);
    assert!(render(&a.image("/a.png", "avatar", vec![])).contains(r#"data-state="hidden""#));
    assert!(render(&a.fallback(vec![], vec![])).contains(r#"data-state="visible""#));

    assert!(dispatch(&mut a, "loaded", ""));
    assert_eq!(a.status(), ImageStatus::Loaded);
    assert!(render(&a.image("/a.png", "avatar", vec![])).contains(r#"data-state="visible""#));
    assert!(render(&a.fallback(vec![], vec![])).contains(r#"data-state="hidden""#));

    assert!(dispatch(&mut a, "error", ""));
    assert_eq!(a.status(), ImageStatus::Error);
    assert!(render(&a.image("/a.png", "avatar", vec![])).contains(r#"data-state="hidden""#));
    assert!(render(&a.fallback(vec![], vec![])).contains(r#"data-state="visible""#));

    assert!(dispatch(&mut a, "reset", ""));
    assert_eq!(a.status(), ImageStatus::Loading);

    assert!(!dispatch(&mut a, "no_such_action", ""));
    assert_eq!(a.status(), ImageStatus::Loading);
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let a = Avatar::default();
    let html = render(&a.view());
    assert!(!html.contains("data-hydrate-"));
    assert!(html.contains(r#"data-state="hidden""#)); // fallback visible => image hidden
}

#[test]
fn view_root_is_element_node_for_hydration() {
    // render_for_hydration の前提（ルートが Node::Element であること）を
    // 公開 API 経由で固定する。
    assert!(matches!(Avatar::default().view(), Node::Element { .. }));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let a = Avatar::new(ImageStatus::Loaded);
    let html = render(&render_for_hydration(&a));
    assert!(html.contains(r#"data-hydrate-status="loaded""#));

    let restored = Avatar::from_hydration_attrs(&a.hydration_attrs()).unwrap();
    assert_eq!(restored, a);
}

#[test]
fn hydration_tampered_value_returns_error_not_panic() {
    for bogus in ["LOADED", "<script>alert(1)</script>", ""] {
        let attrs = vec![("data-hydrate-status".to_string(), bogus.to_string())];
        let err = Avatar::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

#[test]
fn hydration_missing_attr_returns_error_not_panic() {
    let err = Avatar::from_hydration_attrs(&[]).unwrap_err();
    assert_eq!(
        err,
        HydrateError::MissingAttr("data-hydrate-status".to_string())
    );
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";
const SCRIPT_PAYLOAD: &str = "<script>alert(1)</script>";

#[test]
fn src_alt_payloads_are_escaped_end_to_end() {
    let image = avatar::image(
        ImageStatus::Loaded,
        ATTR_BREAK_PAYLOAD,
        ATTR_BREAK_PAYLOAD,
        vec![],
    );
    let html = render(&avatar::root(vec![], vec![image]));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&avatar::root(
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}

#[test]
fn fallback_children_script_payload_is_escaped_end_to_end() {
    let fallback = avatar::fallback(ImageStatus::Loading, vec![], vec![text(SCRIPT_PAYLOAD)]);
    let html = render(&avatar::root(vec![], vec![fallback]));

    assert!(!html.contains(SCRIPT_PAYLOAD));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn caller_supplied_scope_and_part_are_dropped_end_to_end() {
    let html = render(&avatar::root(
        vec![("data-scope", "attacker"), ("data-part", "attacker")],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="avatar""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(!html.contains("attacker"));
}

// ---------------------------------------------------------------------
// 参考サイトとの突合（イシュー #1659）: 省略属性の固定
//
// avatar.rs モジュール doc「参考サイトとの突合」節が列挙する非採用属性
// （role/aria-*・dir・id）が実際に出力に現れないこと、および root への
// data-state 非付与・image/fallback の data-state 語彙が visible/hidden の
// 2 値のみであることを、突合結果の正しさを機械的に固定するために検証する。
// ---------------------------------------------------------------------

#[test]
fn assembly_omits_role_aria_dir_and_id_attrs() {
    let image = avatar::image(ImageStatus::Loaded, "/avatar.png", "Naledi Khumalo", vec![]);
    let fallback = avatar::fallback(ImageStatus::Error, vec![], vec![text("NK")]);
    let html = render(&avatar::root(vec![], vec![image, fallback]));

    // ark-ui/Zag.js・Radix Primitives・Radix Themes・chakra-ui のいずれも
    // role/aria-*/dir/id を付与しない（参考サイトとの突合で確認済み）。
    assert!(!html.contains("role="));
    assert!(!html.contains("aria-"));
    assert!(!html.contains(" dir="));
    assert!(!html.contains(" id="));
}

#[test]
fn root_never_carries_data_state() {
    // ark-ui Notes 節準拠: data-state は image/fallback 固有の情報であり
    // root の関心事ではない（avatar.rs 冒頭 rustdoc 参照）。
    let root_only = avatar::root(vec![], vec![]);
    let html = render(&root_only);
    assert!(html.contains(r#"data-part="root""#));
    assert!(!html.contains("data-state"));
}

#[test]
fn image_and_fallback_data_state_vocabulary_is_exactly_two_values() {
    for status in [
        ImageStatus::Loading,
        ImageStatus::Loaded,
        ImageStatus::Error,
    ] {
        let image_html = render(&avatar::image(status, "/a.png", "avatar", vec![]));
        let fallback_html = render(&avatar::fallback(status, vec![], vec![]));

        let image_state = if status.is_image_visible() {
            "visible"
        } else {
            "hidden"
        };
        let fallback_state = if status.is_image_visible() {
            "hidden"
        } else {
            "visible"
        };
        assert!(image_html.contains(&format!(r#"data-state="{image_state}""#)));
        assert!(fallback_html.contains(&format!(r#"data-state="{fallback_state}""#)));
        // "visible"/"hidden" 以外の第三の値が混入しないことを確認する。
        assert!(!image_html.contains(r#"data-state="loading""#));
        assert!(!fallback_html.contains(r#"data-state="loading""#));
    }
}

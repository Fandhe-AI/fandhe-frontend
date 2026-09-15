//! `fandhe_frontend_pre_styled_ui::toast_motion`（イシュー #2543）の
//! golden・契約テスト。`docs/internal/pre-styled-ui-golden-test-update-guide.md`
//! 方式 (a)（実出力貼り付けによるバイト一致）に従う。

#![cfg(feature = "motion")]

use std::fs;
use std::path::PathBuf;

use fandhe_frontend_core::keyed::{KeyedListError, BIND_LIST_ATTR, KEY_ATTR};
use fandhe_frontend_core::render;
use fandhe_frontend_pre_styled_ui::theme::Theme;
use fandhe_frontend_pre_styled_ui::toast::{self, root, ToastPlacement, ToastStatus};
use fandhe_frontend_pre_styled_ui::toast_motion::{
    stack_group_keyed, FLIP_AUTO_ATTR, STACK_ATTR, STAGGER_AUTO_FIRST_ATTR, TOAST_STACK_CSS,
};

/// (a) [`TOAST_STACK_CSS`] の golden バイト一致。
#[test]
fn toast_stack_css_matches_golden_fixture() {
    let expected = concat!(
        "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack] {\n",
        "  display: grid;\n",
        "  grid-template-areas: \"stack\";\n",
        "  --fandhe-toast-stack-offset: var(--fandhe-space-3);\n",
        "  --fandhe-toast-stack-scale-step: 0.05;\n",
        "}\n",
        "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack][data-placement^=\"bottom\"] {\n",
        "  align-items: end;\n",
        "}\n",
        "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack][data-placement^=\"top\"] {\n",
        "  align-items: start;\n",
        "}\n",
        "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack] > [data-scope=\"toast\"][data-part=\"root\"] {\n",
        "  grid-area: stack;\n",
        "  --fandhe-toast-stack-index: var(--fandhe-motion-stagger-index, 0);\n",
        "  z-index: calc(100 - var(--fandhe-toast-stack-index));\n",
        "  translate: 0 calc(var(--fandhe-toast-stack-index) * -1 * var(--fandhe-toast-stack-offset));\n",
        "  scale: calc(1 - var(--fandhe-toast-stack-index) * var(--fandhe-toast-stack-scale-step));\n",
        "  transform-origin: center bottom;\n",
        "  transition-property: translate, scale, opacity;\n",
        "  transition-duration: var(--fandhe-motion-duration-normal);\n",
        "  transition-timing-function: var(--fandhe-motion-easing-standard);\n",
        "}\n",
        "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack][data-placement^=\"top\"] > [data-scope=\"toast\"][data-part=\"root\"] {\n",
        "  translate: 0 calc(var(--fandhe-toast-stack-index) * var(--fandhe-toast-stack-offset));\n",
        "  transform-origin: center top;\n",
        "}\n",
        "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack] > [data-scope=\"toast\"][data-part=\"root\"]:nth-child(n+4) {\n",
        "  opacity: 0;\n",
        "  pointer-events: none;\n",
        "}\n",
        "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack]:is(:hover, :focus-within) {\n",
        "  display: flex;\n",
        "}\n",
        "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack]:is(:hover, :focus-within) > [data-scope=\"toast\"][data-part=\"root\"] {\n",
        "  translate: none;\n",
        "  scale: none;\n",
        "  opacity: 1;\n",
        "  pointer-events: auto;\n",
        "}\n",
    );
    assert_eq!(TOAST_STACK_CSS, expected);
}

/// (b) [`Theme::to_css_with_toast_motion`] が `to_css()` への pure append
/// であること。
#[test]
fn to_css_with_toast_motion_is_pure_append() {
    let theme = Theme::default();
    assert_eq!(
        theme.to_css_with_toast_motion(),
        format!("{}{TOAST_STACK_CSS}", theme.to_css())
    );
}

/// (c) [`STACK_ATTR`] が `toast::group` の render 出力へ透過すること。
#[test]
fn stack_attr_appears_in_group_render_output() {
    let html = render(&toast::group(
        ToastPlacement::BottomEnd,
        "Notifications",
        vec![(STACK_ATTR, "")],
        vec![],
    ));
    assert!(html.contains(STACK_ATTR));
}

/// (d) [`stack_group_keyed`] の出力が `data-scope="toast"`・placement
/// class・3 opt-in 属性・`data-bind-list` を持つこと。
#[test]
fn stack_group_keyed_output_has_required_markers() {
    let items = vec![("t-1".to_string(), root(ToastStatus::Info, vec![], vec![]))];
    let node = stack_group_keyed(
        ToastPlacement::BottomEnd,
        "Notifications",
        vec![],
        "toasts",
        items,
    )
    .expect("有効な items のため Ok");
    let html = render(&node);
    assert!(html.contains(r#"data-scope="toast""#));
    assert!(html.contains(r#"data-part="group""#));
    assert!(html.contains("fd-toast--placement-bottom-end"));
    assert!(html.contains(STACK_ATTR));
    assert!(html.contains(STAGGER_AUTO_FIRST_ATTR));
    assert!(html.contains(FLIP_AUTO_ATTR));
    assert!(html.contains(BIND_LIST_ATTR));
}

/// (e) `attrs` に `data-bind-list` を混ぜると `Err` を返すこと。
#[test]
fn stack_group_keyed_rejects_data_bind_list_in_attrs() {
    let err = stack_group_keyed(
        ToastPlacement::BottomEnd,
        "Notifications",
        vec![(BIND_LIST_ATTR, "evil")],
        "toasts",
        vec![],
    )
    .expect_err("data-bind-list 偽装は拒否されること");
    assert_eq!(
        err,
        KeyedListError::ReservedAttr {
            attr: BIND_LIST_ATTR
        }
    );
}

/// (e') `attrs` に `data-key` を混ぜても同様に `Err` を返すこと。
#[test]
fn stack_group_keyed_rejects_data_key_in_attrs() {
    let err = stack_group_keyed(
        ToastPlacement::BottomEnd,
        "Notifications",
        vec![(KEY_ATTR, "evil")],
        "toasts",
        vec![],
    )
    .expect_err("data-key 偽装は拒否されること");
    assert_eq!(err, KeyedListError::ReservedAttr { attr: KEY_ATTR });
}

// ---------------------------------------------------------------------
// (f) wasm-full 側リテラルとのドリフト検知（`button_motion_attr_drift.rs`
// と同型の手法。`fandhe-frontend-pre-styled-ui` は `fandhe-frontend-
// wasm-full` に依存していないため実行時にソースファイルを読む）。
// ---------------------------------------------------------------------

fn wasm_full_src_path(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("wasm-full")
        .join("src")
        .join(file)
}

fn read_source(file: &str) -> String {
    let path = wasm_full_src_path(file);
    fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "{} を読めること（fail-closed。wasm-full 側の定数定義との \
             ドリフト検知が本テストの目的であり、読み取り不能を PASS 扱い \
             にしない）: {err}",
            path.display()
        )
    })
}

fn assert_const_matches(content: &str, const_name: &str, expected_value: &str, source_file: &str) {
    let expected_decl = format!("pub const {const_name}: &str = \"{expected_value}\";");
    assert!(
        content.contains(&expected_decl),
        "{source_file} の {const_name} 定義が toast_motion 側（{expected_value:?}）と \
         一致しません。期待した宣言行: {expected_decl:?}"
    );
}

#[test]
fn stagger_auto_first_attr_matches_wasm_full() {
    let content = read_source("stagger_index.rs");
    assert_const_matches(
        &content,
        "STAGGER_AUTO_FIRST_ATTR",
        STAGGER_AUTO_FIRST_ATTR,
        "stagger_index.rs",
    );
}

#[test]
fn flip_auto_attr_matches_wasm_full() {
    let content = read_source("layout_flip.rs");
    assert_const_matches(&content, "FLIP_AUTO_ATTR", FLIP_AUTO_ATTR, "layout_flip.rs");
}

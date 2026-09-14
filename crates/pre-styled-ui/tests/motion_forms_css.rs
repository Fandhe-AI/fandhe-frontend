//! イシュー #2545「field / input の Motion+ 由来アニメーション
//! （examples/forms 4 点）」の golden・契約テスト。`motion` feature 配下
//! のみコンパイルする（feature off では空テストバイナリ、
//! `tests/motion_zero_cost.rs` の「無効時ゼロコスト」契約と両立する。
//! `tests/motion_border_beam_css.rs` と同型の構成）。
#![cfg(feature = "motion")]

use fandhe_frontend_core::render;
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps, FieldRootProps};
use fandhe_frontend_pre_styled_ui::forms_motion::{
    self, FLOATING_LABEL_CLASS, FLOATING_LABEL_CSS, SHAKE_CSS, UNDERLINE_GROW_CSS,
};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps, InputVariant};
use fandhe_frontend_pre_styled_ui::stylesheet::StyleSheet;
use fandhe_frontend_pre_styled_ui::theme::Theme;

/// [`SHAKE_CSS`] の golden（バイト一致、
/// `docs/internal/pre-styled-ui-golden-test-update-guide.md` 方式 (a)）。
/// 更新手順: `cargo test -p fandhe-frontend-pre-styled-ui --features motion
/// --test motion_forms_css` の実出力を貼り付ける。
const EXPECTED_SHAKE_CSS: &str = "[data-scope=\"field\"][data-part=\"input\"][data-invalid] {\n  animation: fd-forms-motion-shake 400ms ease-in-out 1;\n}\n@keyframes fd-forms-motion-shake {\n  10%, 90% {\n    transform: translateX(-1px);\n  }\n  20%, 80% {\n    transform: translateX(2px);\n  }\n  30%, 50%, 70% {\n    transform: translateX(-4px);\n  }\n  40%, 60% {\n    transform: translateX(4px);\n  }\n}\n@media (prefers-reduced-motion: reduce) {\n  [data-scope=\"field\"][data-part=\"input\"][data-invalid] {\n    animation: none;\n  }\n}\n";

const EXPECTED_UNDERLINE_GROW_CSS: &str = "[data-scope=\"field\"][data-part=\"input\"].fd-field--variant-flushed {\n  background-image: linear-gradient(var(--fandhe-color-focus-ring, var(--fandhe-color-accent)), var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));\n  background-repeat: no-repeat;\n  background-position: bottom center;\n  background-size: 0% 2px;\n  transition-property: background-size;\n  transition-duration: var(--fandhe-motion-duration-normal);\n  transition-timing-function: var(--fandhe-motion-easing-standard);\n}\n[data-scope=\"field\"][data-part=\"input\"].fd-field--variant-flushed:focus-visible {\n  background-size: 100% 2px;\n}\n";

const EXPECTED_FLOATING_LABEL_CSS: &str = ".fd-field-floating-label [data-scope=\"field\"][data-part=\"label\"] {\n  position: absolute;\n  left: var(--fandhe-size-control-padding-x-md, 1rem);\n  top: 50%;\n  transform: translateY(-50%);\n  transform-origin: left top;\n  transition-property: transform, top, color;\n  transition-duration: var(--fandhe-motion-duration-normal);\n  transition-timing-function: var(--fandhe-motion-easing-standard);\n  pointer-events: none;\n  background: var(--fandhe-color-bg);\n  padding: 0 var(--fandhe-space-1, 0.25rem);\n}\n[data-scope=\"field\"][data-part=\"input\"]:not(:placeholder-shown) ~ [data-scope=\"field\"][data-part=\"label\"],\n[data-scope=\"field\"][data-part=\"input\"]:focus ~ [data-scope=\"field\"][data-part=\"label\"] {\n  top: 0;\n  transform: translateY(-50%) scale(0.85);\n  color: var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n}\n";

fn default_field(id: &str) -> FieldProps<'_> {
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
fn shake_css_matches_golden() {
    assert_eq!(SHAKE_CSS, EXPECTED_SHAKE_CSS);
}

#[test]
fn underline_grow_css_matches_golden() {
    assert_eq!(UNDERLINE_GROW_CSS, EXPECTED_UNDERLINE_GROW_CSS);
}

#[test]
fn floating_label_css_matches_golden() {
    assert_eq!(FLOATING_LABEL_CSS, EXPECTED_FLOATING_LABEL_CSS);
}

/// [`UNDERLINE_GROW_CSS`] のセレクタが `input::input`（`InputVariant::Flushed`）
/// の実際のレンダリング出力クラスと一致することを固定する（drift 検知、
/// `docs/design/...` の手書きリテラルが `crate::recipe` の命名規則から
/// 逸脱していないことをレンダリング経由で確認する）。
#[test]
fn underline_grow_css_selector_matches_rendered_flushed_class() {
    let f = default_field("amount");
    let node = input::input(
        &InputProps {
            variant: InputVariant::Flushed,
            ..InputProps::default()
        },
        &f,
        vec![("placeholder", " ")],
    );
    let html = render(&node);
    let class = html
        .split("class=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("class 属性が見つからない");
    for token in class.split_whitespace() {
        if token.starts_with("fd-field--variant-") {
            assert!(
                UNDERLINE_GROW_CSS.contains(token),
                "UNDERLINE_GROW_CSS のセレクタ ({token}) が実際のレンダリング\
                 出力クラスと一致しない: {class}"
            );
            return;
        }
    }
    panic!("variant クラスが見つからない: {class}");
}

/// [`FLOATING_LABEL_CSS`] のセレクタ（`data-scope`/`data-part`）が
/// `field::root`/`field::label`/`input::input` の実際のレンダリング出力と
/// 一致することを固定する（drift 検知）。
#[test]
fn floating_label_css_selector_matches_rendered_field_and_input_parts() {
    let f = default_field("email");
    let node = field::root(
        &FieldRootProps::default(),
        &f,
        vec![],
        vec![
            input::input(&InputProps::default(), &f, vec![("placeholder", " ")]),
            field::label(&f, vec![], vec![fandhe_frontend_core::text("Email")]),
        ],
    );
    let html = render(&node);
    assert!(html.contains(r#"data-scope="field" data-part="input""#));
    assert!(html.contains(r#"data-scope="field" data-part="label""#));
    assert!(FLOATING_LABEL_CSS.contains(r#"[data-scope="field"][data-part="input"]"#));
    assert!(FLOATING_LABEL_CSS.contains(r#"[data-scope="field"][data-part="label"]"#));
}

#[test]
fn floating_label_class_wraps_field_root_without_altering_its_output() {
    let f = default_field("email");
    let root_node = field::root(&FieldRootProps::default(), &f, vec![], vec![]);
    let root_html_direct = render(&root_node);

    let root_node_2 = field::root(&FieldRootProps::default(), &f, vec![], vec![]);
    let wrapped = fandhe_frontend_core::el(
        "div",
        vec![("class", FLOATING_LABEL_CLASS)],
        vec![root_node_2],
    );
    let wrapped_html = render(&wrapped);
    assert!(wrapped_html.contains(&root_html_direct));
    assert!(wrapped_html.starts_with(&format!(r#"<div class="{FLOATING_LABEL_CLASS}">"#)));
}

#[test]
fn does_not_use_angle_bracket_literal_anywhere() {
    assert!(!SHAKE_CSS.contains('<'));
    assert!(!UNDERLINE_GROW_CSS.contains('<'));
    assert!(!FLOATING_LABEL_CSS.contains('<'));
    assert!(!forms_motion::error_text_presence_css().contains('<'));
}

#[test]
fn error_text_presence_css_matches_shared_preset_output() {
    // `crate::recipe::SlotRecipe::presence_transition` の共通 preset を
    // `error-text` slot に適用しただけであることを、宣言内容（opacity 1・
    // transition-behavior: allow-discrete・hidden state の opacity 0）で
    // 確認する（新規セレクタ組み立てを持たないことの回帰確認）。
    let out = forms_motion::error_text_presence_css();
    assert!(out.contains("opacity: 1;"));
    assert!(out.contains("transition-behavior: allow-discrete;"));
    assert!(out.contains(r#"[data-scope="field"][data-part="error-text"][hidden] {"#));
    assert!(out.contains("opacity: 0;"));
}

#[test]
fn to_css_with_forms_motion_is_to_css_plus_css() {
    let theme = Theme::default();
    let base = theme.to_css();
    let extended = theme.to_css_with_forms_motion();
    let extra = forms_motion::forms_motion_css();
    assert!(extended.starts_with(&base));
    assert_eq!(extended.len(), base.len() + extra.len());
    assert_eq!(&extended[base.len()..], extra);
}

#[test]
fn forms_motion_css_passes_stylesheet_push_css() {
    let mut sheet = StyleSheet::new();
    assert!(sheet.push_css(&forms_motion::forms_motion_css()).is_ok());
}

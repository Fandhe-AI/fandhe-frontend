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

const EXPECTED_FLOATING_LABEL_CSS: &str = ".fd-field-floating-label {\n  position: relative;\n  display: flex;\n}\n.fd-field-floating-label [data-scope=\"field\"][data-part=\"label\"] {\n  position: absolute;\n  left: var(--fandhe-size-control-padding-x-md, 1rem);\n  top: 50%;\n  transform: translateY(-50%);\n  transform-origin: left top;\n  transition-property: transform, top, color;\n  transition-duration: var(--fandhe-motion-duration-normal);\n  transition-timing-function: var(--fandhe-motion-easing-standard);\n  pointer-events: none;\n  background: var(--fandhe-color-bg);\n  padding: 0 var(--fandhe-space-1, 0.25rem);\n}\n.fd-field-floating-label [data-scope=\"field\"][data-part=\"input\"]:not(:placeholder-shown) ~ [data-scope=\"field\"][data-part=\"label\"],\n.fd-field-floating-label [data-scope=\"field\"][data-part=\"input\"]:focus ~ [data-scope=\"field\"][data-part=\"label\"] {\n  top: 0;\n  transform: translateY(-50%) scale(0.85);\n}\n.fd-field-floating-label [data-scope=\"field\"][data-part=\"input\"]:not(:placeholder-shown) ~ [data-scope=\"field\"][data-part=\"label\"]:not([data-invalid]),\n.fd-field-floating-label [data-scope=\"field\"][data-part=\"input\"]:focus ~ [data-scope=\"field\"][data-part=\"label\"]:not([data-invalid]) {\n  color: var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n}\n";

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

/// PR #2567 レビュー是正（P1-2/Bugbot「Floating label ignores extra field
/// parts」）の回帰確認: wrapper は `input`/`label` の 2 要素だけを子に持ち、
/// `field::root` **自体**をラップするのではなく、その wrapper を
/// `field::root` の children の 1 要素として渡す（helper-text は wrapper
/// の外・root 直下の兄弟）。この構造では wrapper（`position: relative`
/// 基準）の高さが helper-text の有無・行数に左右されないことを、
/// レンダリング出力の構造で固定する。
#[test]
fn floating_label_class_wraps_only_input_and_label_inside_field_root() {
    let f = default_field("email");
    let wrapper = fandhe_frontend_core::el(
        "div",
        vec![("class", FLOATING_LABEL_CLASS)],
        vec![
            input::input(&InputProps::default(), &f, vec![("placeholder", " ")]),
            field::label(&f, vec![], vec![fandhe_frontend_core::text("Email")]),
        ],
    );
    let root_node = field::root(
        &FieldRootProps::default(),
        &f,
        vec![],
        vec![
            wrapper,
            field::helper_text(&f, vec![], vec![fandhe_frontend_core::text("補助テキスト")]),
        ],
    );
    let html = render(&root_node);

    // wrapper は input → label の順で 2 要素だけを内包する（helper-text は
    // 含まない）。
    let wrapper_start = html
        .find(&format!(r#"<div class="{FLOATING_LABEL_CLASS}">"#))
        .expect("wrapper の開始タグが見つからない");
    let input_idx = html
        .find(r#"data-scope="field" data-part="input""#)
        .expect("input が見つからない");
    let label_idx = html
        .find(r#"data-scope="field" data-part="label""#)
        .expect("label が見つからない");
    let helper_idx = html
        .find(r#"data-scope="field" data-part="helper-text""#)
        .expect("helper-text が見つからない");
    assert!(wrapper_start < input_idx);
    assert!(input_idx < label_idx);
    // helper-text は wrapper の外（label より後・root 直下）にある。
    assert!(label_idx < helper_idx);
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

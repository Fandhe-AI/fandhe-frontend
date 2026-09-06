//! styled Field（イシュー #1684、親 #1671）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/alert_css.rs` と同型の golden fixture
//! テスト。`field` recipe は `root`/`label`/`helper-text`/`error-text`/
//! `required-indicator` の 5 slot のみを宣言し、`input`/`textarea`/`select`
//! slot は [`crate::input`]/[`crate::textarea`]/[`crate::native_select`]
//! （`crates/pre-styled-ui/src/input.rs` 参照）が所有するため意図的に
//! 宣言しない（`field.rs` モジュール doc「スコープ」節参照）。本ファイルは
//! それを CSS 出力側からも固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};

const FIELD_GOLDEN_CSS: &str = r#"[data-scope="field"][data-part="root"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1-5, 0.375rem);
  width: 100%;
  position: relative;
  box-sizing: border-box;
}

[data-scope="field"][data-part="label"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-1);
  font-size: var(--fandhe-font-font-size-sm);
  font-weight: var(--fandhe-font-font-weight-medium);
  line-height: var(--fandhe-font-line-height-normal);
  color: var(--fandhe-color-fg);
  user-select: none;
}

[data-scope="field"][data-part="helper-text"] {
  font-size: var(--fandhe-font-font-size-sm);
  line-height: var(--fandhe-font-line-height-normal);
  color: var(--fandhe-color-fg-muted);
}

[data-scope="field"][data-part="error-text"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-1);
  font-size: var(--fandhe-font-font-size-sm);
  line-height: var(--fandhe-font-line-height-normal);
  color: var(--fandhe-color-danger);
}

[data-scope="field"][data-part="required-indicator"] {
  color: var(--fandhe-color-danger);
  line-height: var(--fandhe-font-line-height-tight);
}

[data-scope="field"][data-part="root"].fd-field--orientation-horizontal {
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  gap: var(--fandhe-space-2);
}

[data-scope="field"][data-part="error-text"][hidden] {
  display: none;
}

[data-scope="field"][data-part="required-indicator"][hidden] {
  display: none;
}

[data-scope="field"][data-part="label"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="field"][data-part="helper-text"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}
"#;

#[test]
fn field_css_matches_golden_fixture() {
    assert_eq!(field::css(), FIELD_GOLDEN_CSS);
}

#[test]
fn css_is_byte_identical_across_calls() {
    assert_eq!(field::css(), field::css());
}

#[test]
fn css_never_contains_style_breakout_sequences() {
    let css = field::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

/// `orientation` 軸のクラスセレクタが CSS 中に存在することを固定する
/// （golden 全文一致に加え、軸の意図が読み取れる形で明示する。既定
/// `vertical` は `default_variant` のみでベース CSS 側の追加宣言を持たない
/// ため、`horizontal` のみクラスセレクタを持つ）。
#[test]
fn css_declares_orientation_horizontal_selector() {
    let css = field::css();
    assert!(css.contains(".fd-field--orientation-horizontal {"));
    assert!(!css.contains(".fd-field--orientation-vertical {"));
}

/// `[hidden]` を `display: none` に固定する規則が `error-text`/
/// `required-indicator` の両方に存在することを固定する。headless
/// `field::error_text`/`field::required_indicator` は非該当状態で `hidden`
/// 存在属性を出す fail-closed 描画（`crates/headless-ui/src/field.rs`
/// 参照）であり、base の `display: inline-flex`（`error-text`）が UA の
/// `[hidden] { display: none; }` を上書きしてしまわないよう本規則が必要。
#[test]
fn css_hides_error_text_and_required_indicator_when_hidden_attr_present() {
    let css = field::css();
    assert!(css.contains(r#"[data-scope="field"][data-part="error-text"][hidden] {"#));
    assert!(css.contains(r#"[data-scope="field"][data-part="required-indicator"][hidden] {"#));
}

/// `field` recipe が `input`/`textarea`/`select` slot への CSS を一切
/// 持たないこと（`crate::input` 等との二重定義防止）を固定する。
#[test]
fn css_does_not_declare_control_slots() {
    let css = field::css();
    assert!(!css.contains(r#"[data-part="input"]"#));
    assert!(!css.contains(r#"[data-part="textarea"]"#));
    assert!(!css.contains(r#"[data-part="select"]"#));
}

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

/// styled `root` が headless `field::root` の `data-scope="field"
/// data-part="root"` へ正しく接続していることを実レンダリングで確認する
/// （golden の静的 CSS とマークアップの整合性を突合）。
#[test]
fn styled_root_connects_to_headless_field_root_markup() {
    let f = default_field("f");
    let html = render(&field::root(&FieldRootProps::default(), &f, vec![], vec![]));
    assert!(html.contains(r#"data-scope="field" data-part="root""#));
    assert!(html.contains("fd-field--orientation-vertical"));
}

/// horizontal orientation を選択したときにクラスが切り替わることを実
/// レンダリングで確認する。
#[test]
fn styled_root_horizontal_orientation_applies_class() {
    let f = default_field("f");
    let props = FieldRootProps {
        orientation: FieldOrientation::Horizontal,
    };
    let html = render(&field::root(&props, &f, vec![], vec![]));
    assert!(html.contains("fd-field--orientation-horizontal"));
}

/// 選択的再エクスポート（`label`/`helper_text`/`error_text`/
/// `required_indicator`）が headless の `data-scope="field"
/// data-part="<slot>"` へ正しく接続していることを確認する。
#[test]
fn reexported_parts_connect_to_headless_field_markup() {
    use fandhe_frontend_core::text;
    use fandhe_frontend_pre_styled_ui::field::{
        error_text, helper_text, label, required_indicator,
    };

    let f = default_field("f");
    assert!(render(&label(&f, vec![], vec![text("Email")]))
        .contains(r#"data-scope="field" data-part="label""#));
    assert!(render(&helper_text(&f, vec![], vec![text("hint")]))
        .contains(r#"data-scope="field" data-part="helper-text""#));
    assert!(render(&error_text(&f, vec![], vec![text("error")]))
        .contains(r#"data-scope="field" data-part="error-text""#));
    assert!(render(&required_indicator(&f, vec![], vec![text("*")]))
        .contains(r#"data-scope="field" data-part="required-indicator""#));
}

//! styled Fieldset（イシュー #1686、親 #1672）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/field_css.rs` と同型の golden fixture
//! テスト。`fieldset` recipe は `root`/`legend`/`helper-text`/`error-text`
//! の 4 slot のみを宣言する（chakra-ui v3 の `Content` パーツは headless
//! anatomy に存在しないため実装しない、`fieldset.rs` モジュール doc
//! 「スコープ」節参照）。

use fandhe_frontend_core::render;
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;

const FIELDSET_GOLDEN_CSS: &str = r#"[data-scope="fieldset"][data-part="root"] {
  display: flex;
  flex-direction: column;
  width: 100%;
  min-width: 0;
  margin: 0;
  padding: 0;
  border: 0;
  box-sizing: border-box;
  position: relative;
}

[data-scope="fieldset"][data-part="legend"] {
  padding: 0;
  display: block;
  color: var(--fandhe-color-fg);
  font-weight: var(--fandhe-font-font-weight-medium);
  line-height: var(--fandhe-font-line-height-normal);
}

[data-scope="fieldset"][data-part="helper-text"] {
  font-size: var(--fandhe-font-font-size-sm);
  line-height: var(--fandhe-font-line-height-normal);
  color: var(--fandhe-color-fg-muted);
}

[data-scope="fieldset"][data-part="error-text"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  font-size: var(--fandhe-font-font-size-sm);
  font-weight: var(--fandhe-font-font-weight-medium);
  line-height: var(--fandhe-font-line-height-normal);
  color: var(--fandhe-color-danger);
}

[data-scope="fieldset"][data-part="root"].fd-fieldset--size-sm {
  gap: var(--fandhe-space-2);
}

[data-scope="fieldset"][data-part="root"].fd-fieldset--size-md {
  gap: var(--fandhe-space-4);
}

[data-scope="fieldset"][data-part="root"].fd-fieldset--size-lg {
  gap: var(--fandhe-space-6);
}

[data-scope="fieldset"][data-part="legend"].fd-fieldset--size-sm {
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="fieldset"][data-part="legend"].fd-fieldset--size-md {
  font-size: var(--fandhe-font-font-size-md);
}

[data-scope="fieldset"][data-part="legend"].fd-fieldset--size-lg {
  font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="fieldset"][data-part="error-text"][hidden] {
  display: none;
}

[data-scope="fieldset"][data-part="legend"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="fieldset"][data-part="helper-text"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}
"#;

#[test]
fn fieldset_css_matches_golden_fixture() {
    assert_eq!(fieldset::css(), FIELDSET_GOLDEN_CSS);
}

#[test]
fn css_is_byte_identical_across_calls() {
    assert_eq!(fieldset::css(), fieldset::css());
}

#[test]
fn css_never_contains_style_breakout_sequences() {
    let css = fieldset::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

/// `size` 軸の 3 段（`sm`/`md`/`lg`）のセレクタが存在し、`xs`/`xl` は
/// 未登録であることを固定する（モジュール doc「variant 軸」節参照）。
#[test]
fn css_declares_size_sm_md_lg_selectors_only() {
    let css = fieldset::css();
    assert!(css.contains(".fd-fieldset--size-sm {"));
    assert!(css.contains(".fd-fieldset--size-md {"));
    assert!(css.contains(".fd-fieldset--size-lg {"));
    assert!(!css.contains(".fd-fieldset--size-xs"));
    assert!(!css.contains(".fd-fieldset--size-xl"));
}

/// `[hidden]` を `display: none` に固定する規則が `error-text` に存在する
/// ことを固定する。headless `fieldset::error_text` は非該当状態（`!invalid`）
/// で `hidden` 存在属性を出す fail-closed 描画（`crates/headless-ui/src/
/// fieldset.rs` 参照）であり、base の `display: inline-flex` が UA の
/// `[hidden] { display: none; }` を上書きしてしまわないよう本規則が必要。
#[test]
fn css_hides_error_text_when_hidden_attr_present() {
    let css = fieldset::css();
    assert!(css.contains(r#"[data-scope="fieldset"][data-part="error-text"][hidden] {"#));
}

/// `legend`/`helper-text` の `[data-disabled]` 規則が存在することを固定
/// する。
#[test]
fn css_disables_legend_and_helper_text() {
    let css = fieldset::css();
    assert!(css.contains(r#"[data-scope="fieldset"][data-part="legend"][data-disabled] {"#));
    assert!(css.contains(r#"[data-scope="fieldset"][data-part="helper-text"][data-disabled] {"#));
}

/// `root` にはネイティブ `disabled` 伝播があるため
/// [`fandhe_frontend_pre_styled_ui::recipe::disabled_declarations`] を
/// 付与しない（モジュール doc「意図的非採用」節参照）。
#[test]
fn css_does_not_apply_disabled_declarations_to_root() {
    let css = fieldset::css();
    assert!(!css.contains(r#"[data-scope="fieldset"][data-part="root"][data-disabled]"#));
}

/// `legend` は `data-invalid` による色変更を持たない（chakra-ui v3 も
/// 非対応）。CSS が `[data-invalid]` を参照する dead セレクタを持たない
/// ことを固定する。
#[test]
fn css_does_not_declare_dead_invalid_selector() {
    let css = fieldset::css();
    assert!(!css.contains("[data-invalid]"));
}

fn default_fieldset(id: &str) -> FieldsetProps<'_> {
    FieldsetProps {
        id,
        disabled: false,
        invalid: false,
        has_helper_text: false,
    }
}

/// styled `root` が headless `fieldset::root` の `data-scope="fieldset"
/// data-part="root"` へ正しく接続していることを実レンダリングで確認する
/// （golden の静的 CSS とマークアップの整合性を突合）。
#[test]
fn styled_root_connects_to_headless_fieldset_root_markup() {
    let f = default_fieldset("f");
    let html = render(&fieldset::root(
        &FieldsetRootProps::default(),
        &f,
        vec![],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="fieldset" data-part="root""#));
    assert!(html.contains("fd-fieldset--size-md"));
}

/// `Size::Lg` を選択したときにクラスが切り替わることを実レンダリングで
/// 確認する。
#[test]
fn styled_root_lg_size_applies_class() {
    let f = default_fieldset("f");
    let props = FieldsetRootProps { size: Size::Lg };
    let html = render(&fieldset::root(&props, &f, vec![], vec![]));
    assert!(html.contains("fd-fieldset--size-lg"));
}

/// 選択的再エクスポート（`legend`/`helper_text`/`error_text`）が headless
/// の `data-scope="fieldset" data-part="<slot>"` へ正しく接続していることを
/// 確認する。
#[test]
fn reexported_parts_connect_to_headless_fieldset_markup() {
    use fandhe_frontend_core::text;
    use fandhe_frontend_pre_styled_ui::fieldset::{error_text, helper_text, legend};

    let f = default_fieldset("f");
    assert!(render(&legend(&f, vec![], vec![text("Address")]))
        .contains(r#"data-scope="fieldset" data-part="legend""#));
    assert!(render(&helper_text(&f, vec![], vec![text("hint")]))
        .contains(r#"data-scope="fieldset" data-part="helper-text""#));

    let mut invalid = default_fieldset("f");
    invalid.invalid = true;
    assert!(render(&error_text(&invalid, vec![], vec![text("error")]))
        .contains(r#"data-scope="fieldset" data-part="error-text""#));
}

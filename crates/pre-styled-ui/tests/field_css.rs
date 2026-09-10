//! styled Field（イシュー #1684、親 #1671。#2185 で group/content/title/
//! separator の 6 slot を追加。#2199 で `group` を container とする
//! `orientation="responsive"` を追加）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/alert_css.rs` と同型の golden fixture
//! テスト。`field` recipe は元々 `root`/`label`/`helper-text`/`error-text`/
//! `required-indicator` の 5 slot のみを宣言していた。`input`/`textarea`/
//! `select` slot は [`crate::input`]/[`crate::textarea`]/
//! [`crate::native_select`]（`crates/pre-styled-ui/src/input.rs` 参照）が
//! 所有するため意図的に宣言しない（`field.rs` モジュール doc「スコープ」
//! 節参照）。本ファイルはそれを CSS 出力側からも固定する。イシュー #2185
//! で `group`/`content`/`title`/`separator`/`separator-line`/
//! `separator-content` の 6 slot が純追加された（`FIELD_GOLDEN_BLOCKS_BEFORE_2185`
//! が既存ブロック群の verbatim 維持を固定する）。イシュー #2199 で `group`
//! 2 個目の base ブロック（`container-type`/`container-name`）の中間挿入と、
//! 末尾（`title[data-invalid]` の後・`error-text > ul` の前）への
//! `@container` ブロック追加が生じた（`FIELD_GOLDEN_BLOCKS_BEFORE_2185` の
//! 対象外。同定数は #2185 以前のブロックのみを扱う契約のため、#2199 の
//! 中間挿入・追記は `field_css_matches_golden_fixture` の全文一致が
//! 単独で固定する）。

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

[data-scope="field"][data-part="group"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-6);
  width: 100%;
}

[data-scope="field"][data-part="group"] {
  container-type: inline-size;
  container-name: fd-field-group;
}

[data-scope="field"][data-part="content"] {
  display: flex;
  flex: 1 1 0%;
  flex-direction: column;
  gap: var(--fandhe-space-1-5, 0.375rem);
  line-height: var(--fandhe-font-line-height-normal);
}

[data-scope="field"][data-part="title"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-1);
  width: fit-content;
  font-size: var(--fandhe-font-font-size-sm);
  font-weight: var(--fandhe-font-font-weight-medium);
  line-height: var(--fandhe-font-line-height-normal);
  color: var(--fandhe-color-fg);
  user-select: none;
}

[data-scope="field"][data-part="separator"] {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  height: var(--fandhe-space-5);
  font-size: var(--fandhe-font-font-size-sm);
  line-height: var(--fandhe-font-line-height-normal);
}

[data-scope="field"][data-part="separator-line"] {
  position: absolute;
  inset-inline: 0;
  top: 50%;
  margin: 0;
  border-width: 0;
  border-top-width: var(--fandhe-separator-thickness, 1px);
  border-top-style: solid;
  border-top-color: var(--fandhe-color-border);
}

[data-scope="field"][data-part="separator-content"] {
  position: relative;
  padding-inline: var(--fandhe-space-2);
  background-color: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg-muted);
  white-space: nowrap;
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

[data-scope="field"][data-part="label"][data-invalid] {
  color: var(--fandhe-color-danger);
}

[data-scope="field"][data-part="title"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="field"][data-part="title"][data-invalid] {
  color: var(--fandhe-color-danger);
}

@container fd-field-group (min-width: 448px) {
  [data-scope="field"][data-part="root"].fd-field--orientation-responsive {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    gap: var(--fandhe-space-2);
  }
}

[data-scope="field"][data-part="error-text"] > ul {
  margin: 0;
  padding: 0 0 0 var(--fandhe-space-4);
  list-style: disc;
}
[data-scope="field"][data-part="error-text"] > ul > li + li {
  margin-top: var(--fandhe-space-1);
}
"#;

/// イシュー #2185 以前から存在するブロック群（`root`/`label`/`helper-text`/
/// `error-text`/`required-indicator` の base・`root` の horizontal variant・
/// 各種 state・`error-text > ul` 静的追記）を宣言順のまま連結したもの。
/// 新 6 slot は `required-indicator` base の直後・`.fd-field--orientation-horizontal`
/// の前へ中間挿入されるため、golden 全文一致だけでは既存ブロックの
/// verbatim 維持（バイト単位で変更していないこと）が読み取りにくい。
/// このため `field::css()` が本定数の全ブロックを元の順序のまま含む
/// ことを別途固定する（pie_donut #2084 と同型の「既存ブロック verbatim +
/// 中間挿入」検証パターン、`docs/internal/pre-styled-ui-golden-test-update-guide.md`
/// 参照）。
const FIELD_GOLDEN_BLOCKS_BEFORE_2185: &[&str] = &[
    "[data-scope=\"field\"][data-part=\"root\"] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1-5, 0.375rem);\n  width: 100%;\n  position: relative;\n  box-sizing: border-box;\n}\n",
    "[data-scope=\"field\"][data-part=\"label\"] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  line-height: var(--fandhe-font-line-height-normal);\n  color: var(--fandhe-color-fg);\n  user-select: none;\n}\n",
    "[data-scope=\"field\"][data-part=\"helper-text\"] {\n  font-size: var(--fandhe-font-font-size-sm);\n  line-height: var(--fandhe-font-line-height-normal);\n  color: var(--fandhe-color-fg-muted);\n}\n",
    "[data-scope=\"field\"][data-part=\"error-text\"] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n  font-size: var(--fandhe-font-font-size-sm);\n  line-height: var(--fandhe-font-line-height-normal);\n  color: var(--fandhe-color-danger);\n}\n",
    "[data-scope=\"field\"][data-part=\"required-indicator\"] {\n  color: var(--fandhe-color-danger);\n  line-height: var(--fandhe-font-line-height-tight);\n}\n",
    "[data-scope=\"field\"][data-part=\"root\"].fd-field--orientation-horizontal {\n  flex-direction: row;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n}\n",
    "[data-scope=\"field\"][data-part=\"error-text\"][hidden] {\n  display: none;\n}\n",
    "[data-scope=\"field\"][data-part=\"required-indicator\"][hidden] {\n  display: none;\n}\n",
    "[data-scope=\"field\"][data-part=\"label\"][data-disabled] {\n  opacity: 0.5;\n  cursor: not-allowed;\n}\n",
    "[data-scope=\"field\"][data-part=\"helper-text\"][data-disabled] {\n  opacity: 0.5;\n  cursor: not-allowed;\n}\n",
    "[data-scope=\"field\"][data-part=\"label\"][data-invalid] {\n  color: var(--fandhe-color-danger);\n}\n",
    "[data-scope=\"field\"][data-part=\"error-text\"] > ul {\n  margin: 0;\n  padding: 0 0 0 var(--fandhe-space-4);\n  list-style: disc;\n}\n[data-scope=\"field\"][data-part=\"error-text\"] > ul > li + li {\n  margin-top: var(--fandhe-space-1);\n}\n",
];

#[test]
fn field_css_matches_golden_fixture() {
    assert_eq!(field::css(), FIELD_GOLDEN_CSS);
}

/// イシュー #2185 以前から存在する全ブロックが、内容・宣言順ともに
/// 変更されずそのまま `field::css()` に含まれることを固定する
/// （新 6 slot は中間挿入のみで、既存ブロックのバイトは 1 つも変えて
/// いないことの確認）。
#[test]
fn existing_blocks_are_preserved_verbatim_after_2185_mid_insertion() {
    let css = field::css();
    let mut search_from = 0usize;
    for block in FIELD_GOLDEN_BLOCKS_BEFORE_2185 {
        let found = css[search_from..]
            .find(block)
            .unwrap_or_else(|| panic!("既存ブロックが見つからないか順序が崩れている: {block}"));
        search_from += found + block.len();
    }
}

/// イシュー #2185 で純追加した 6 slot（`group`/`content`/`title`/
/// `separator`/`separator-line`/`separator-content`）のセレクタが存在する
/// ことを固定する。
#[test]
fn css_declares_extended_2185_slot_selectors() {
    let css = field::css();
    assert!(css.contains(r#"[data-scope="field"][data-part="group"] {"#));
    assert!(css.contains(r#"[data-scope="field"][data-part="content"] {"#));
    assert!(css.contains(r#"[data-scope="field"][data-part="title"] {"#));
    assert!(css.contains(r#"[data-scope="field"][data-part="separator"] {"#));
    assert!(css.contains(r#"[data-scope="field"][data-part="separator-line"] {"#));
    assert!(css.contains(r#"[data-scope="field"][data-part="separator-content"] {"#));
    assert!(css.contains(r#"[data-scope="field"][data-part="title"][data-disabled] {"#));
    assert!(css.contains(r#"[data-scope="field"][data-part="title"][data-invalid] {"#));
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

/// `orientation="responsive"`（イシュー #2199）のセレクタが `@container`
/// ブロックの内側にのみ現れること、`group` slot の container-type
/// ブロックが存在すること、`@container` ブロックが
/// `title[data-invalid]` より後・`error-text > ul` より前（recipe 出力の
/// 末尾）に位置することを固定する。
#[test]
fn css_declares_orientation_responsive_selector_inside_container_block() {
    let css = field::css();
    assert!(css.contains(r#"[data-scope="field"][data-part="group"] {"#));
    assert!(css.contains("container-type: inline-size;"));
    assert!(css.contains("container-name: fd-field-group;"));
    assert!(css.contains("@container fd-field-group (min-width: 448px) {"));
    // `.fd-field--orientation-responsive` はブロックの外（base セレクタ等）
    // には現れない: `@container` プレリュードの直後にのみ出現することを、
    // プレリュード文字列を含む行の直後で出現する位置関係として確認する。
    let container_pos = css
        .find("@container fd-field-group (min-width: 448px) {")
        .expect("container block must exist");
    let responsive_pos = css
        .find(".fd-field--orientation-responsive {")
        .expect("responsive selector must exist");
    assert!(responsive_pos > container_pos);

    let invalid_title_pos = css
        .find(r#"[data-scope="field"][data-part="title"][data-invalid] {"#)
        .expect("title[data-invalid] rule must exist");
    let error_list_pos = css
        .find(r#"[data-scope="field"][data-part="error-text"] > ul {"#)
        .expect("error-text > ul rule must exist");
    assert!(container_pos > invalid_title_pos);
    assert!(container_pos < error_list_pos);
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

/// shadcn/ui 突合（イシュー #2014）で追加した 2 規則
/// （`label[data-invalid]` の文字色切替・`error-text > ul` のリスト整形）を
/// 個別に固定する（golden 全文一致に加え、追加意図が読み取れる形で明示する。
/// `field.rs` モジュール doc「shadcn/ui 突合」節参照）。
#[test]
fn css_declares_invalid_label_color_and_error_list_layout() {
    let css = field::css();
    assert!(css.contains(r#"[data-scope="field"][data-part="label"][data-invalid] {"#));
    assert!(css.contains(r#"[data-scope="field"][data-part="error-text"] > ul {"#));
    assert!(css.contains("list-style: disc;"));
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

/// responsive orientation を選択したときにクラスが切り替わることを実
/// レンダリングで確認する（イシュー #2199）。
#[test]
fn styled_root_responsive_orientation_applies_class() {
    let f = default_field("f");
    let props = FieldRootProps {
        orientation: FieldOrientation::Responsive,
    };
    let html = render(&field::root(&props, &f, vec![], vec![]));
    assert!(html.contains("fd-field--orientation-responsive"));
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

/// イシュー #2185 で追加した再エクスポート（`group`/`content`/`title`/
/// `separator`）が headless の `data-scope="field" data-part="<slot>"` へ
/// 正しく接続していることを確認する。
#[test]
fn reexported_extended_2185_parts_connect_to_headless_field_markup() {
    use fandhe_frontend_core::text;
    use fandhe_frontend_pre_styled_ui::field::{content, group, separator, title};

    let f = default_field("f");
    assert!(
        render(&group(vec![], vec![text("g")])).contains(r#"data-scope="field" data-part="group""#)
    );
    assert!(render(&content(&f, vec![], vec![text("c")]))
        .contains(r#"data-scope="field" data-part="content""#));
    assert!(render(&title(&f, vec![], vec![text("t")]))
        .contains(r#"data-scope="field" data-part="title""#));
    let separator_html = render(&separator(vec![], vec![text("Or continue with")]));
    assert!(separator_html.contains(r#"data-scope="field" data-part="separator""#));
    assert!(separator_html.contains(r#"data-scope="field" data-part="separator-line""#));
    assert!(separator_html.contains(r#"data-scope="field" data-part="separator-content""#));
}

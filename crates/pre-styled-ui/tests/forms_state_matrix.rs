//! Forms 家族横断の `label[data-required]` 視覚化・`control`/`clear-trigger`
//! の `data-disabled` 消費規則（イシュー #2195）を出力ベースで固定する契約
//! テスト。
//!
//! 決定根拠・対応表（R1〜R4）の正は
//! `docs/design/pre-styled-ui-forms-disabled-required-matrix.md`。本ファイル
//! は同文書が確定した規則をソース走査ではなく `stylesheet()`/`css()`（CSS
//! 出力）と `render()`（headless 出力）で機械検証する。
//!
//! - R1（opacity 単一階層）: 各部品の `control[data-disabled]`/
//!   `clear-trigger[data-disabled]` ブロックを抽出し、`opacity` の有無を
//!   期待表どおりに固定する。
//! - R2（`label[data-required]` は Themes で CSS 消費しない）: 対象 Forms
//!   部品のスタイルシートに `[data-required]` セレクタが一切出現しないこと
//!   を固定する。
//! - R3（headless が出さない属性へ規則を書かない）: `pin_input::control`/
//!   `editable::control` は `disabled: true` 相当でも `data-disabled` を
//!   出力しないことを固定する（将来 headless が出し始めたらこのテストが
//!   落ち、Themes 側の規則追加検討を促す）。
//!
//! 新規に是正した 7 部品（date-picker/combobox/select/color-picker/
//! number-input/rating-group/date-input）の `control`/`clear-trigger` を
//! 対象とする。tags-input/file-upload/signature-pad/password-input は
//! #1696 等の既存イシューで `control`/`clear-trigger` の disabled 消費が
//! 既に実装・テスト済みのため、本ファイルでは重複対象としない。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::color_picker::{self, ColorPickerProps};
use fandhe_frontend_headless_ui::combobox::{self, ComboboxProps};
use fandhe_frontend_headless_ui::date_input::{self, DateInputProps};
use fandhe_frontend_headless_ui::date_picker::{self, DatePickerProps};
use fandhe_frontend_headless_ui::editable;
use fandhe_frontend_headless_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_headless_ui::pin_input;
use fandhe_frontend_headless_ui::rating_group::{self, RatingGroupProps};
use fandhe_frontend_headless_ui::select::{self, SelectProps};
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_pre_styled_ui::{
    color_picker as styled_color_picker, combobox as styled_combobox,
    date_input as styled_date_input, date_picker as styled_date_picker,
    editable as styled_editable, file_upload as styled_file_upload,
    number_input as styled_number_input, password_input as styled_password_input,
    pin_input as styled_pin_input, rating_group as styled_rating_group, select as styled_select,
    tags_input as styled_tags_input,
};

/// `css` 中で `selector_with_brace`（例: `"...[data-disabled] {"`）から
/// 対応する `}` までの本文を抜き出す（`date_picker.rs`/`combobox.rs`/
/// `select.rs` の同名インラインヘルパと同型、こちらは複数クレート横断の
/// external テストのため個別ファイルへ再実装せず本ファイルへ 1 箇所だけ
/// 定義する）。
fn extract_block<'a>(css: &'a str, selector_with_brace: &str) -> &'a str {
    let block_start = css
        .find(selector_with_brace)
        .unwrap_or_else(|| panic!("selector not found: {selector_with_brace}, css={css}"));
    let body_start = block_start + selector_with_brace.len();
    let body_end = css[body_start..]
        .find('}')
        .map(|offset| body_start + offset)
        .unwrap_or_else(|| panic!("unterminated block for {selector_with_brace}"));
    &css[body_start..body_end]
}

/// R1 期待値: `(scope, part, opacity を含むべきか)`。
struct Expectation {
    scope: &'static str,
    part: &'static str,
    owns_opacity: bool,
}

const EXPECTATIONS: &[Expectation] = &[
    // 葉所有型（input/trigger が opacity を所有、control は cursor のみ）
    Expectation {
        scope: "date-picker",
        part: "control",
        owns_opacity: false,
    },
    Expectation {
        scope: "date-picker",
        part: "clear-trigger",
        owns_opacity: true,
    },
    Expectation {
        scope: "combobox",
        part: "control",
        owns_opacity: false,
    },
    Expectation {
        scope: "combobox",
        part: "clear-trigger",
        owns_opacity: true,
    },
    Expectation {
        scope: "select",
        part: "control",
        owns_opacity: false,
    },
    Expectation {
        scope: "select",
        part: "clear-trigger",
        owns_opacity: true,
    },
    Expectation {
        scope: "color-picker",
        part: "control",
        owns_opacity: false,
    },
    // コンテナ所有型（root が opacity を所有、control は cursor のみ）
    Expectation {
        scope: "number-input",
        part: "control",
        owns_opacity: false,
    },
    Expectation {
        scope: "date-input",
        part: "control",
        owns_opacity: false,
    },
    // rating-group は item（葉）が opacity を所有、control は cursor のみ
    Expectation {
        scope: "rating-group",
        part: "control",
        owns_opacity: false,
    },
];

/// 部品スコープ名と対応する `stylesheet()`/`css()` 関数のペア一覧の型
/// （`clippy::type_complexity` 回避のためのエイリアス。値そのものは
/// 単純な `(&str, fn() -> String)` のスライスであり、意味は自明のため
/// 型定義を分けるだけで十分）。
type ScopedStylesheetFns = &'static [(&'static str, fn() -> String)];

#[test]
fn control_and_clear_trigger_disabled_follow_r1_opacity_single_layer() {
    let sheets: ScopedStylesheetFns = &[
        ("date-picker", styled_date_picker::stylesheet),
        ("combobox", styled_combobox::stylesheet),
        ("select", styled_select::stylesheet),
        ("color-picker", styled_color_picker::css),
        ("number-input", styled_number_input::stylesheet),
        ("date-input", styled_date_input::stylesheet),
        ("rating-group", styled_rating_group::stylesheet),
    ];

    for expectation in EXPECTATIONS {
        let (_, sheet_fn) = sheets
            .iter()
            .find(|(scope, _)| *scope == expectation.scope)
            .unwrap_or_else(|| panic!("no stylesheet registered for scope {}", expectation.scope));
        let css = sheet_fn();
        let selector = format!(
            r#"[data-scope="{}"][data-part="{}"][data-disabled] {{"#,
            expectation.scope, expectation.part
        );
        let block = extract_block(&css, &selector);
        if expectation.owns_opacity {
            assert!(
                block.contains("opacity: 0.5"),
                "{}::{} must own opacity (leaf slot): {block}",
                expectation.scope,
                expectation.part
            );
        } else {
            assert!(
                !block.contains("opacity"),
                "{}::{} must not own opacity (layout-only slot, avoids double dimming): {block}",
                expectation.scope,
                expectation.part
            );
        }
        assert!(
            block.contains("cursor: not-allowed"),
            "{}::{} must apply cursor: not-allowed: {block}",
            expectation.scope,
            expectation.part
        );
    }
}

#[test]
fn no_forms_stylesheet_consumes_data_required_r2() {
    // イシュー #2195 R2: 必須マーカーは `field::required_indicator` の
    // 合成で表現し、Themes recipe は `label[data-required]` を CSS 消費
    // しない（`*` 等の生成コンテンツを追加しない）。
    let sheets: &[(&str, String)] = &[
        ("date-picker", styled_date_picker::stylesheet()),
        ("combobox", styled_combobox::stylesheet()),
        ("select", styled_select::stylesheet()),
        ("color-picker", styled_color_picker::css()),
        ("number-input", styled_number_input::stylesheet()),
        ("date-input", styled_date_input::stylesheet()),
        ("rating-group", styled_rating_group::stylesheet()),
        ("tags-input", styled_tags_input::stylesheet()),
        ("file-upload", styled_file_upload::stylesheet()),
        ("password-input", styled_password_input::stylesheet()),
        ("pin-input", styled_pin_input::stylesheet()),
        ("editable", styled_editable::stylesheet()),
    ];
    for (scope, css) in sheets {
        assert!(
            !css.contains("data-required"),
            "{scope} stylesheet must not consume [data-required] (R2): {css}"
        );
    }
}

#[test]
fn headless_control_disabled_output_matches_r3() {
    // R3: headless が `data-disabled` を出す control/clear-trigger は
    // render() 出力に `data-disabled=""` を含む。
    let date_picker_props = DatePickerProps {
        disabled: true,
        ..DatePickerProps::default()
    };
    let html = render(&date_picker::control(
        OpenState::default(),
        &date_picker_props,
        vec![],
        vec![],
    ));
    assert!(html.contains("data-disabled"));
    let html = render(&date_picker::clear_trigger(
        &date_picker_props,
        vec![],
        vec![],
    ));
    assert!(html.contains("data-disabled"));

    let combobox_props = ComboboxProps {
        disabled: true,
        ..ComboboxProps::default()
    };
    let html = render(&combobox::control(
        OpenState::default(),
        &combobox_props,
        vec![],
        vec![],
    ));
    assert!(html.contains("data-disabled"));
    let html = render(&combobox::clear_trigger(&combobox_props, vec![], vec![]));
    assert!(html.contains("data-disabled"));

    let select_props = SelectProps {
        disabled: true,
        ..SelectProps::default()
    };
    let html = render(&select::control(
        OpenState::default(),
        &select_props,
        vec![],
        vec![],
    ));
    assert!(html.contains("data-disabled"));
    let html = render(&select::clear_trigger(&select_props, vec![], vec![]));
    assert!(html.contains("data-disabled"));

    let color_picker_props = ColorPickerProps {
        disabled: true,
        ..ColorPickerProps::default()
    };
    let html = render(&color_picker::control(
        OpenState::default(),
        &color_picker_props,
        vec![],
        vec![],
    ));
    assert!(html.contains("data-disabled"));

    let number_input_flags = NumberInputFlags {
        disabled: true,
        ..NumberInputFlags::default()
    };
    let html = render(&number_input::control(number_input_flags, vec![], vec![]));
    assert!(html.contains("data-disabled"));

    let rating_group_props = RatingGroupProps {
        disabled: true,
        ..RatingGroupProps::default()
    };
    let html = render(&rating_group::control(
        &rating_group_props,
        None,
        vec![],
        vec![],
    ));
    assert!(html.contains("data-disabled"));

    let date_input_props = DateInputProps {
        disabled: true,
        ..DateInputProps::default()
    };
    let html = render(&date_input::control(date_input_props, vec![], vec![]));
    assert!(html.contains("data-disabled"));

    // headless が出さない 2 例（R3、モジュール rustdoc「headless が出さない
    // 2 箇所」節参照）: pin-input `control`/editable `control` はそもそも
    // `data-disabled` を受け取る経路を持たない（引数に disabled 相当が
    // 存在しない）ため、render() 出力に一切含まれない。
    let html = render(&pin_input::control(vec![], vec![]));
    assert!(!html.contains("data-disabled"));
    let html = render(&editable::control(
        editable::EditMode::Preview,
        vec![],
        vec![],
    ));
    assert!(!html.contains("data-disabled"));
}

#[test]
fn forms_stylesheets_never_contain_style_breakout_sequences() {
    let sheets: &[(&str, String)] = &[
        ("date-picker", styled_date_picker::stylesheet()),
        ("combobox", styled_combobox::stylesheet()),
        ("select", styled_select::stylesheet()),
        ("color-picker", styled_color_picker::css()),
        ("number-input", styled_number_input::stylesheet()),
        ("date-input", styled_date_input::stylesheet()),
        ("rating-group", styled_rating_group::stylesheet()),
    ];
    for (scope, css) in sheets {
        assert!(!css.contains("</style"), "{scope} contains </style");
        assert!(!css.contains('<'), "{scope} contains a raw '<'");
    }
}

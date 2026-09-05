//! Primitives Demo — Forms B（11 件、原稿は #1025）。
//! 執筆規約は `crate::primitive_showcase` モジュール doc 参照。

use fandhe_frontend_core::{text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::data_attrs::Orientation;
use hui::number_input::{self, NumberInputFlags};
use hui::password_input::{self, PasswordAutocomplete, PasswordInputProps};
use hui::pin_input::{self, PinInputKind, PinInputProps};
use hui::radio_group;
use hui::rating_group::{self, RatingGroupProps, RatingItemFlags};
use hui::segment_group;
use hui::select;
use hui::signature_pad::{self, Point, SignaturePad, Stroke};
use hui::slider;
use hui::switch::{self, SwitchProps};
use hui::tags_input;
use hui::OpenState;

use super::demo_page;

/// NumberInput Demo（イシュー #1613 参照突合: 基本 / 下限到達 / disabled /
/// invalid+required / readonly の 5 インスタンス化。基本インスタンスは
/// [`number_input::value_text`]（同イシューで新設した 7 番目のパーツ）も
/// 露出する）。
pub(super) fn number_input_section() -> Node {
    let flags = NumberInputFlags::default();
    let basic = vec![number_input::root(
        flags,
        vec![],
        vec![
            number_input::label(flags, Some("ni-input"), vec![], vec![text("Quantity")]),
            number_input::control(
                flags,
                vec![],
                vec![
                    number_input::decrement_trigger(
                        Some("ni-input"),
                        false,
                        vec![],
                        vec![text("−")],
                    ),
                    number_input::input(
                        "quantity",
                        Some("ni-input"),
                        Some("3"),
                        "0",
                        "99",
                        flags,
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("ni-input"),
                        false,
                        vec![],
                        vec![text("+")],
                    ),
                ],
            ),
            number_input::value_text(flags, vec![], vec![text("3")]),
        ],
    )];

    // 下限到達（イシュー #1613: decrement トリガーが disabled になる境界例）。
    let at_min = vec![number_input::root(
        flags,
        vec![],
        vec![
            number_input::label(flags, Some("ni-input-min"), vec![], vec![text("At min")]),
            number_input::control(
                flags,
                vec![],
                vec![
                    number_input::decrement_trigger(
                        Some("ni-input-min"),
                        true,
                        vec![],
                        vec![text("−")],
                    ),
                    number_input::input(
                        "quantity-min",
                        Some("ni-input-min"),
                        Some("0"),
                        "0",
                        "99",
                        flags,
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("ni-input-min"),
                        false,
                        vec![],
                        vec![text("+")],
                    ),
                ],
            ),
        ],
    )];

    // disabled。
    let disabled_flags = NumberInputFlags {
        disabled: true,
        ..NumberInputFlags::default()
    };
    let disabled = vec![number_input::root(
        disabled_flags,
        vec![],
        vec![
            number_input::label(
                disabled_flags,
                Some("ni-input-disabled"),
                vec![],
                vec![text("Disabled")],
            ),
            number_input::control(
                disabled_flags,
                vec![],
                vec![
                    number_input::decrement_trigger(
                        Some("ni-input-disabled"),
                        true,
                        vec![],
                        vec![text("−")],
                    ),
                    number_input::input(
                        "quantity-disabled",
                        Some("ni-input-disabled"),
                        Some("3"),
                        "0",
                        "99",
                        disabled_flags,
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("ni-input-disabled"),
                        true,
                        vec![],
                        vec![text("+")],
                    ),
                ],
            ),
        ],
    )];

    // invalid + required（イシュー #1613: label の `data-required` を Demo で
    // 露出する）。
    let invalid_required_flags = NumberInputFlags {
        invalid: true,
        required: true,
        ..NumberInputFlags::default()
    };
    let invalid_required = vec![number_input::root(
        invalid_required_flags,
        vec![],
        vec![
            number_input::label(
                invalid_required_flags,
                Some("ni-input-invalid"),
                vec![],
                vec![text("Invalid, required")],
            ),
            number_input::control(
                invalid_required_flags,
                vec![],
                vec![
                    number_input::decrement_trigger(
                        Some("ni-input-invalid"),
                        false,
                        vec![],
                        vec![text("−")],
                    ),
                    number_input::input(
                        "quantity-invalid",
                        Some("ni-input-invalid"),
                        Some("3"),
                        "0",
                        "99",
                        invalid_required_flags,
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("ni-input-invalid"),
                        false,
                        vec![],
                        vec![text("+")],
                    ),
                ],
            ),
        ],
    )];

    // readonly（イシュー #1613: root/control の `data-readonly` を Demo で
    // 露出する）。
    let readonly_flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    let readonly = vec![number_input::root(
        readonly_flags,
        vec![],
        vec![
            number_input::label(
                readonly_flags,
                Some("ni-input-readonly"),
                vec![],
                vec![text("Readonly")],
            ),
            number_input::control(
                readonly_flags,
                vec![],
                vec![
                    number_input::decrement_trigger(
                        Some("ni-input-readonly"),
                        true,
                        vec![],
                        vec![text("−")],
                    ),
                    number_input::input(
                        "quantity-readonly",
                        Some("ni-input-readonly"),
                        Some("7"),
                        "0",
                        "99",
                        readonly_flags,
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("ni-input-readonly"),
                        true,
                        vec![],
                        vec![text("+")],
                    ),
                ],
            ),
        ],
    )];

    demo_page(
        "Number Input",
        [basic, at_min, disabled, invalid_required, readonly].concat(),
    )
}

/// `password_input_section` の 1 インスタンス分を組み立てる非公開ヘルパ。
/// `visible`/`props` の組み合わせで `data-state`/`data-readonly` 等の
/// `Observed Values` を描き分ける（`forms_a::checkbox_instance` と同型の
/// パターン、イシュー #1614）。
fn password_input_instance(
    visible: bool,
    props: &PasswordInputProps<'_>,
    label_text: &str,
) -> Node {
    password_input::root(
        visible,
        props,
        vec![],
        vec![
            password_input::label(props, vec![], vec![text(label_text)]),
            password_input::control(
                visible,
                props,
                vec![],
                vec![
                    password_input::input(visible, props, vec![]),
                    password_input::visibility_trigger(
                        visible,
                        props,
                        vec![(
                            "aria-label",
                            if visible {
                                "Hide password"
                            } else {
                                "Show password"
                            },
                        )],
                        vec![password_input::indicator(
                            visible,
                            props,
                            vec![],
                            vec![text("👁")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// `data-state`（visible/hidden）と `data-disabled`/`data-invalid`/
/// `data-required`/`data-readonly` を描き分けた 5 インスタンスを並べる
/// （イシュー #1614、参照突合の一環）。
pub(super) fn password_input_section() -> Node {
    let hidden = PasswordInputProps {
        id: "pw-hidden",
        disabled: false,
        readonly: false,
        invalid: false,
        required: false,
        autocomplete: PasswordAutocomplete::CurrentPassword,
    };
    let visible = PasswordInputProps {
        id: "pw-visible",
        disabled: false,
        readonly: false,
        invalid: false,
        required: false,
        autocomplete: PasswordAutocomplete::CurrentPassword,
    };
    let disabled = PasswordInputProps {
        id: "pw-disabled",
        disabled: true,
        readonly: false,
        invalid: false,
        required: false,
        autocomplete: PasswordAutocomplete::CurrentPassword,
    };
    let invalid_required = PasswordInputProps {
        id: "pw-invalid",
        disabled: false,
        readonly: false,
        invalid: true,
        required: true,
        autocomplete: PasswordAutocomplete::NewPassword,
    };
    let readonly = PasswordInputProps {
        id: "pw-readonly",
        disabled: false,
        readonly: true,
        invalid: false,
        required: false,
        autocomplete: PasswordAutocomplete::CurrentPassword,
    };
    let body = vec![
        password_input_instance(false, &hidden, "Password"),
        password_input_instance(true, &visible, "Password (visible)"),
        password_input_instance(false, &disabled, "Password (disabled)"),
        password_input_instance(false, &invalid_required, "New password"),
        password_input_instance(false, &readonly, "Password (read-only)"),
    ];
    demo_page("Password Input", body)
}

/// `values`（各桁の値、空文字列 = 未入力）・`otp`・`props` から 1 個の
/// PinInput インスタンス（root > label + control(input×N) + hidden_input）
/// を組み立てる内部ヘルパ（[`pin_input_section`] のみが呼ぶ）。
/// [`password_input_instance`] と同型のパターン。
fn pin_input_instance(values: &[&str], otp: bool, props: &PinInputProps, label_text: &str) -> Node {
    let count = values.len();
    let complete = values.iter().all(|v| !v.is_empty());
    let inputs: Vec<Node> = values
        .iter()
        .enumerate()
        .map(|(i, value)| {
            pin_input::input(
                i,
                count,
                value,
                PinInputKind::Numeric,
                false,
                otp,
                props,
                complete,
                vec![],
            )
        })
        .collect();
    pin_input::root(
        complete,
        props,
        vec![],
        vec![
            pin_input::label(complete, props, vec![], vec![text(label_text)]),
            pin_input::control(vec![], inputs),
            pin_input::hidden_input("otp", &values.concat(), props.disabled, vec![]),
        ],
    )
}

pub(super) fn pin_input_section() -> Node {
    // ark-ui 公式 Data Attributes 表の全語彙（data-complete/data-disabled/
    // data-invalid/data-readonly/data-required/data-index/data-filled）と
    // aria-invalid/native readonly が Anatomy/data-* 表へ機械導出されるよう、
    // 既定・complete・disabled・invalid+required・readonly の 5 状態を並べる
    // （イシュー #1615）。
    let default_props = PinInputProps::default();
    let disabled_props = PinInputProps {
        disabled: true,
        ..Default::default()
    };
    let invalid_required_props = PinInputProps {
        invalid: true,
        required: true,
        ..Default::default()
    };
    let readonly_props = PinInputProps {
        readonly: true,
        ..Default::default()
    };
    let body = vec![
        pin_input_instance(&["1", "2", "", ""], true, &default_props, "One-time code"),
        pin_input_instance(&["1", "2", "3", "4"], true, &default_props, "Complete"),
        pin_input_instance(&["", "", "", ""], true, &disabled_props, "Disabled"),
        pin_input_instance(
            &["", "", "", ""],
            true,
            &invalid_required_props,
            "Invalid + required",
        ),
        pin_input_instance(&["1", "2", "", ""], true, &readonly_props, "Read-only"),
    ];
    demo_page("Pin Input", body)
}

/// `label_text`/`label_id`/`name`/`props`/`orientation`/`items`（各項目は
/// `(value, text, checked, item_disabled)`）から 1 個の RadioGroup
/// インスタンス（root > label + item(item_control + item_text +
/// item_hidden_input)×N）を組み立てる内部ヘルパ（[`radio_group_section`]
/// のみが呼ぶ）。[`pin_input_instance`] と同型のパターン。項目単位で
/// disabled を上書きしたい場合は `item_disabled` を `true` にした
/// `RadioGroupProps` コピーを各パーツへ渡す（イシュー #1616）。
fn radio_group_instance(
    label_text: &str,
    label_id: &str,
    name: &str,
    props: &radio_group::RadioGroupProps,
    orientation: Option<Orientation>,
    items: &[(&str, &str, bool, bool)],
) -> Node {
    let item_nodes: Vec<Node> = items
        .iter()
        .map(|(value, text_label, checked, item_disabled)| {
            let item_props = radio_group::RadioGroupProps {
                disabled: props.disabled || *item_disabled,
                ..*props
            };
            radio_group::item(
                *checked,
                &item_props,
                value,
                vec![],
                vec![
                    radio_group::item_control(*checked, &item_props, vec![]),
                    radio_group::item_text(*checked, &item_props, vec![], vec![text(*text_label)]),
                    radio_group::item_hidden_input(
                        *checked,
                        &item_props,
                        Some(name),
                        value,
                        vec![],
                    ),
                ],
            )
        })
        .collect();
    let mut children = vec![radio_group::label(
        props,
        Some(label_id),
        vec![],
        vec![text(label_text)],
    )];
    children.extend(item_nodes);
    radio_group::root(props, orientation, Some(label_id), vec![], children)
}

pub(super) fn radio_group_section() -> Node {
    // ark-ui / Radix Primitives の Data Attributes・Keyboard 表の全語彙
    // （data-disabled/data-invalid/data-readonly/data-required/
    // aria-required/aria-readonly/aria-disabled/aria-hidden/aria-invalid/
    // data-orientation）が Anatomy/data-* 表へ機械導出されるよう、既定
    // （縦）・horizontal・disabled item・disabled group・invalid+required・
    // readonly の 6 状態を並べる（イシュー #1616）。
    let default_props = radio_group::RadioGroupProps::default();
    let disabled_props = radio_group::RadioGroupProps {
        disabled: true,
        ..Default::default()
    };
    let invalid_required_props = radio_group::RadioGroupProps {
        invalid: true,
        required: true,
        ..Default::default()
    };
    let readonly_props = radio_group::RadioGroupProps {
        readonly: true,
        ..Default::default()
    };
    let body = vec![
        radio_group_instance(
            "Plan",
            "rg-plan-label",
            "plan",
            &default_props,
            None,
            &[
                ("monthly", "Monthly", true, false),
                ("yearly", "Yearly", false, false),
            ],
        ),
        radio_group_instance(
            "Plan (horizontal)",
            "rg-plan-horizontal-label",
            "plan-horizontal",
            &default_props,
            Some(Orientation::Horizontal),
            &[
                ("monthly", "Monthly", true, false),
                ("yearly", "Yearly", false, false),
            ],
        ),
        radio_group_instance(
            "Plan (item disabled)",
            "rg-plan-item-disabled-label",
            "plan-item-disabled",
            &default_props,
            None,
            &[
                ("monthly", "Monthly", true, false),
                ("yearly", "Yearly", false, true),
            ],
        ),
        radio_group_instance(
            "Plan (group disabled)",
            "rg-plan-disabled-label",
            "plan-disabled",
            &disabled_props,
            None,
            &[
                ("monthly", "Monthly", true, false),
                ("yearly", "Yearly", false, false),
            ],
        ),
        radio_group_instance(
            "Plan (invalid + required)",
            "rg-plan-invalid-label",
            "plan-invalid",
            &invalid_required_props,
            None,
            &[
                ("monthly", "Monthly", false, false),
                ("yearly", "Yearly", false, false),
            ],
        ),
        radio_group_instance(
            "Plan (read-only)",
            "rg-plan-readonly-label",
            "plan-readonly",
            &readonly_props,
            None,
            &[
                ("monthly", "Monthly", true, false),
                ("yearly", "Yearly", false, false),
            ],
        ),
    ];
    demo_page("Radio Group", body)
}

pub(super) fn rating_group_section() -> Node {
    let props = RatingGroupProps::default();
    let mk = |index: u32, checked: bool, highlighted: bool| {
        rating_group::item(
            index,
            RatingItemFlags {
                checked,
                highlighted,
                disabled: false,
                readonly: false,
            },
            &format!("{index} star"),
            vec![],
            vec![text("★")],
        )
    };
    let body = vec![rating_group::root(
        &props,
        vec![],
        vec![
            rating_group::label(&props, None, vec![], vec![text("Rating")]),
            rating_group::control(
                &props,
                None,
                vec![],
                vec![mk(1, false, true), mk(2, true, true), mk(3, false, false)],
            ),
            rating_group::hidden_input(&props, Some("rating"), "2", vec![]),
        ],
    )];
    demo_page("Rating Group", body)
}

/// `name`/`props`/`orientation`/`selected_index`/`items`（各要素は表示
/// テキスト）から 1 個の SegmentGroup インスタンス（root > indicator +
/// item(item_hidden_input + item_control + item_text)×N）を組み立てる内部
/// ヘルパ（[`segment_group_section`] のみが呼ぶ）。[`radio_group_instance`]
/// と同型のパターン。`name` は各インスタンスで一意にする必要がある
/// （同一 DOM 上のネイティブ `<input type="radio">` は同名グループ間で
/// 排他選択が干渉するため、イシュー #1618 は #1886 の radio-group Demo で
/// 判明した先例に倣う）。
fn segment_group_instance(
    name: &str,
    props: &segment_group::SegmentGroupProps,
    orientation: Option<Orientation>,
    selected_index: usize,
    items: &[&str],
) -> Node {
    let mut children = vec![segment_group::indicator(
        Some((selected_index, items.len())),
        props,
        orientation,
        vec![],
    )];
    children.extend(items.iter().enumerate().map(|(index, label)| {
        let checked = index == selected_index;
        let value = label.to_lowercase();
        segment_group::item(
            checked,
            props,
            &value,
            vec![],
            vec![
                segment_group::item_hidden_input(checked, props, Some(name), &value, vec![]),
                segment_group::item_control(checked, props, vec![]),
                segment_group::item_text(checked, props, vec![], vec![text(*label)]),
            ],
        )
    }));
    segment_group::root(props, orientation, None, vec![], children)
}

pub(super) fn segment_group_section() -> Node {
    // ark-ui の Data Attributes 表の全語彙（data-disabled/data-invalid/
    // data-readonly/data-required/aria-required/aria-readonly/
    // aria-disabled/aria-hidden/aria-invalid/data-orientation）が
    // Anatomy/data-* 表へ機械導出されるよう、既定・disabled・
    // invalid+required・readonly・vertical の 5 状態を並べる（イシュー
    // #1618。`name` は各インスタンスで一意）。
    let default_props = segment_group::SegmentGroupProps::default();
    let disabled_props = segment_group::SegmentGroupProps {
        disabled: true,
        ..Default::default()
    };
    let invalid_required_props = segment_group::SegmentGroupProps {
        invalid: true,
        required: true,
        ..Default::default()
    };
    let readonly_props = segment_group::SegmentGroupProps {
        readonly: true,
        ..Default::default()
    };
    let items = ["List", "Grid"];
    let body = vec![
        segment_group_instance("view", &default_props, None, 0, &items),
        segment_group_instance("view-disabled", &disabled_props, None, 0, &items),
        segment_group_instance("view-invalid", &invalid_required_props, None, 0, &items),
        segment_group_instance("view-readonly", &readonly_props, None, 0, &items),
        segment_group_instance(
            "view-vertical",
            &default_props,
            Some(Orientation::Vertical),
            1,
            &items,
        ),
    ];
    demo_page("Segment Group", body)
}

/// open + 選択済み インスタンス（第 1）を組み立てる内部ヘルパ（イシュー
/// #1619 参照突合、`SelectProps` 導入に伴う 4 インスタンス化）。anatomy 15
/// パーツすべて（`item_group`/`item_group_label`/`item_indicator`/
/// `clear_trigger`/`hidden_select` を含む）を本インスタンスで網羅し、
/// `tests/primitive_showcase.rs::anatomy_coverage_matches_known_uncovered_exactly`
/// の allowlist を増やさない。
fn select_open_instance() -> Node {
    let state = OpenState::Open;
    let props = select::SelectProps::default();
    select::root(
        state,
        &props,
        vec![],
        vec![
            select::label(&props, Some("sel-label"), vec![], vec![text("Fruit")]),
            select::control(
                state,
                &props,
                vec![],
                vec![
                    select::trigger(
                        state,
                        &props,
                        false,
                        Some("sel-content"),
                        Some("sel-label"),
                        vec![],
                        vec![
                            select::value_text(false, &props, vec![], vec![text("Apple")]),
                            select::indicator(state, &props, vec![], vec![text("▾")]),
                        ],
                    ),
                    select::clear_trigger(&props, vec![], vec![text("×")]),
                ],
            ),
            select::positioner(
                state,
                vec![],
                vec![select::content(
                    state,
                    Some("sel-content"),
                    Some("sel-label"),
                    Some("sel-item-0"),
                    vec![],
                    vec![select::item_group(
                        &props,
                        Some("sel-item-group-label"),
                        vec![],
                        vec![
                            select::item_group_label(
                                Some("sel-item-group-label"),
                                vec![],
                                vec![text("Common")],
                            ),
                            select::item(
                                OpenState::Open,
                                &props,
                                false,
                                true,
                                "apple",
                                Some("sel-item-0"),
                                vec![],
                                vec![
                                    select::item_text(
                                        OpenState::Open,
                                        &props,
                                        false,
                                        true,
                                        None,
                                        vec![],
                                        vec![text("Apple")],
                                    ),
                                    select::item_indicator(
                                        OpenState::Open,
                                        vec![],
                                        vec![text("✓")],
                                    ),
                                ],
                            ),
                        ],
                    )],
                )],
            ),
            select::hidden_select(
                Some("apple"),
                Some("fruit"),
                &props,
                vec![],
                vec![("apple", "Apple"), ("banana", "Banana")],
            ),
        ],
    )
}

/// closed + `disabled` + 未選択 インスタンス（第 2）を組み立てる内部
/// ヘルパ（イシュー #1619 参照突合。trigger/value-text 双方の
/// `data-placeholder-shown` を表出させる）。
fn select_disabled_instance() -> Node {
    let state = OpenState::Closed;
    let props = select::SelectProps {
        disabled: true,
        ..select::SelectProps::default()
    };
    select::root(
        state,
        &props,
        vec![],
        vec![
            select::label(&props, None, vec![], vec![text("Fruit (disabled)")]),
            select::control(
                state,
                &props,
                vec![],
                vec![
                    select::trigger(
                        state,
                        &props,
                        true,
                        None,
                        None,
                        vec![],
                        vec![
                            select::value_text(true, &props, vec![], vec![text("Select a fruit")]),
                            select::indicator(state, &props, vec![], vec![text("▾")]),
                        ],
                    ),
                    select::clear_trigger(&props, vec![], vec![text("×")]),
                ],
            ),
        ],
    )
}

/// closed + `invalid` + `required` インスタンス（第 3）を組み立てる内部
/// ヘルパ（イシュー #1619 参照突合。label の `data-required`・hidden-select
/// の `required` を表出させる）。
fn select_invalid_required_instance() -> Node {
    let state = OpenState::Closed;
    let props = select::SelectProps {
        invalid: true,
        required: true,
        ..select::SelectProps::default()
    };
    select::root(
        state,
        &props,
        vec![],
        vec![
            select::label(
                &props,
                None,
                vec![],
                vec![text("Fruit (invalid, required)")],
            ),
            select::control(
                state,
                &props,
                vec![],
                vec![select::trigger(
                    state,
                    &props,
                    true,
                    None,
                    None,
                    vec![],
                    vec![
                        select::value_text(true, &props, vec![], vec![text("Select a fruit")]),
                        select::indicator(state, &props, vec![], vec![text("▾")]),
                    ],
                )],
            ),
            select::hidden_select(None, Some("fruit-invalid"), &props, vec![], vec![]),
        ],
    )
}

/// closed + `readonly` インスタンス（第 4）を組み立てる内部ヘルパ
/// （イシュー #1619 参照突合。`fandhe-frontend-wasm-full` の keynav が
/// trigger の `data-readonly` を確認して keydown を no-op にする契約の
/// SSR 表出）。
fn select_readonly_instance() -> Node {
    let state = OpenState::Closed;
    let props = select::SelectProps {
        readonly: true,
        ..select::SelectProps::default()
    };
    select::root(
        state,
        &props,
        vec![],
        vec![
            select::label(&props, None, vec![], vec![text("Fruit (readonly)")]),
            select::control(
                state,
                &props,
                vec![],
                vec![select::trigger(
                    state,
                    &props,
                    false,
                    None,
                    None,
                    vec![],
                    vec![
                        select::value_text(false, &props, vec![], vec![text("Apple")]),
                        select::indicator(state, &props, vec![], vec![text("▾")]),
                    ],
                )],
            ),
        ],
    )
}

/// Select の Demo（イシュー #1619 参照突合、`SelectProps` 導入に伴い 4
/// インスタンス化。combobox #1605/#1619 と同型のパターン）。open+選択済み
/// （既定、anatomy 15 パーツ全網羅）・closed+disabled・
/// closed+invalid+required・closed+readonly の 4 状態を 1 ページ上に並べ、
/// `data-disabled`/`data-readonly`/`data-invalid`/`data-required`/
/// `data-placeholder-shown`/`data-selected` の機械導出
/// （`component_page.rs::collect_data_attrs_from_tree`）を成立させる。
pub(super) fn select_section() -> Node {
    let body = vec![
        select_open_instance(),
        select_disabled_instance(),
        select_invalid_required_instance(),
        select_readonly_instance(),
    ];
    demo_page("Select", body)
}

pub(super) fn signature_pad_section() -> Node {
    // 参照突合（イシュー #1620）: 是正した data-*/ARIA（label の
    // data-disabled・guide の data-disabled・control の
    // role="application"/aria-roledescription/tabindex/aria-disabled）が
    // すべて Anatomy/data-* 表へ機械導出されるよう、既定（ストロークあり）・
    // 空・disabled・read-only の 4 状態を並べる（`name` は各インスタンスで
    // 一意）。
    let stroke = Stroke::new(vec![
        Point::new(4.0, 40.0),
        Point::new(40.0, 8.0),
        Point::new(80.0, 40.0),
    ])
    .expect("固定座標列は Stroke::new の不変条件（非空・有限値）を満たす");
    let stroke_value = signature_pad::stroke_path_d(&stroke);

    // 既定（ストローク 1 本）。
    let default_paths = vec![signature_pad::segment_path(&stroke, vec![])];
    let default_instance = signature_pad::root(
        false,
        false,
        vec![],
        vec![
            signature_pad::label(false, vec![], vec![text("Signature")]),
            signature_pad::control(
                false,
                vec![("aria-label", "Signature pad")],
                vec![
                    signature_pad::guide(false, vec![], vec![]),
                    signature_pad::segment(
                        160,
                        60,
                        Some("Signature preview"),
                        vec![],
                        default_paths,
                    ),
                ],
            ),
            signature_pad::clear_trigger(false, vec![], vec![text("Clear")]),
            signature_pad::hidden_input("signature", &stroke_value, false, vec![]),
        ],
    );

    // 空（ストロークなし）。ark-ui の空状態相当。
    let empty_instance = signature_pad::root(
        false,
        true,
        vec![],
        vec![
            signature_pad::label(false, vec![], vec![text("Signature (empty)")]),
            signature_pad::control(
                false,
                vec![("aria-label", "Signature pad (empty)")],
                vec![
                    signature_pad::guide(false, vec![], vec![]),
                    signature_pad::segment(160, 60, Some("Signature preview"), vec![], vec![]),
                ],
            ),
            signature_pad::clear_trigger(true, vec![], vec![text("Clear")]),
            signature_pad::hidden_input("signature-empty", "", false, vec![]),
        ],
    );

    // disabled（root/label/control/guide/clear-trigger/hidden-input 全部で
    // data-disabled・aria-disabled を観測させる）。
    let disabled_paths = vec![signature_pad::segment_path(&stroke, vec![])];
    let disabled_instance = signature_pad::root(
        true,
        false,
        vec![],
        vec![
            signature_pad::label(true, vec![], vec![text("Signature (disabled)")]),
            signature_pad::control(
                true,
                vec![("aria-label", "Signature pad (disabled)")],
                vec![
                    signature_pad::guide(true, vec![], vec![]),
                    signature_pad::segment(
                        160,
                        60,
                        Some("Signature preview"),
                        vec![],
                        disabled_paths,
                    ),
                ],
            ),
            signature_pad::clear_trigger(true, vec![], vec![text("Clear")]),
            signature_pad::hidden_input("signature-disabled", &stroke_value, true, vec![]),
        ],
    );

    // read-only（`data-readonly` は SignaturePad::control メソッド経由での
    // み付与されるため struct メソッドを使う）。
    let read_only_pad = SignaturePad::new(vec![stroke], false, true);
    let read_only_instance = read_only_pad.root(
        vec![],
        vec![
            read_only_pad.label(vec![], vec![text("Signature (read-only)")]),
            read_only_pad.control(
                vec![("aria-label", "Signature pad (read-only)")],
                vec![
                    read_only_pad.guide(vec![], vec![]),
                    read_only_pad.segment(
                        160,
                        60,
                        Some("Signature preview"),
                        vec![],
                        read_only_pad.segment_paths(),
                    ),
                ],
            ),
            read_only_pad.clear_trigger(vec![], vec![text("Clear")]),
            read_only_pad.hidden_input("signature-readonly", vec![]),
        ],
    );

    let body = vec![
        default_instance,
        empty_instance,
        disabled_instance,
        read_only_instance,
    ];
    demo_page("Signature Pad", body)
}

pub(super) fn slider_section() -> Node {
    let orientation = Orientation::Horizontal;
    let props = slider::SliderProps::default();
    let body = vec![slider::root(
        orientation,
        &props,
        vec![],
        vec![
            slider::label(&props, vec![], vec![text("Volume")]),
            slider::control(
                orientation,
                &props,
                vec![],
                vec![
                    slider::track(
                        orientation,
                        &props,
                        vec![],
                        vec![slider::range(orientation, &props, vec![], vec![])],
                    ),
                    slider::thumb(
                        orientation,
                        "0",
                        "100",
                        "40",
                        Some("40%"),
                        &props,
                        vec![],
                        vec![],
                    ),
                    // MarkerGroup/Marker（イシュー #1621: ark-ui/zag.js との
                    // 参照突合で追加した anatomy パーツ）。
                    slider::marker_group(
                        vec![],
                        vec![
                            slider::marker(0.0, 40.0, 0.0, 100.0, false, vec![], vec![]),
                            slider::marker(50.0, 40.0, 0.0, 100.0, false, vec![], vec![]),
                            slider::marker(100.0, 40.0, 0.0, 100.0, false, vec![], vec![]),
                        ],
                    ),
                ],
            ),
            slider::hidden_input("volume", "40", false, vec![]),
            slider::value_text(vec![], vec![text("40%")]),
        ],
    )];
    demo_page("Slider", body)
}

/// `switch_section` の 1 インスタンス分を組み立てる非公開ヘルパ
/// （`checkbox_instance`〔forms_a.rs〕と同型、イシュー #1622）。`name` は
/// フォーム送信名（インスタンス間で一意な無害なダミー値）、`label_text` は
/// 表示ラベル。
fn switch_instance(
    checked: bool,
    props: &SwitchProps,
    name: &'static str,
    label_text: &'static str,
) -> Node {
    switch::root(
        checked,
        props,
        vec![],
        vec![
            switch::hidden_input(name, "on", checked, props, vec![]),
            switch::control(
                checked,
                props,
                vec![],
                vec![switch::thumb(checked, props, vec![], vec![])],
            ),
            switch::label(checked, props, vec![], vec![text(label_text)]),
        ],
    )
}

/// `data-state`（checked/unchecked の 2 値）と
/// `data-disabled`/`data-invalid`/`data-required`/`data-readonly` を
/// 描き分けた 5 インスタンスを並べる（イシュー #1622、参照突合の一環。
/// `checkbox_section`〔forms_a.rs〕と同型のデモ構成）。
pub(super) fn switch_section() -> Node {
    let plain = SwitchProps::default();
    let disabled_checked = SwitchProps {
        disabled: true,
        ..SwitchProps::default()
    };
    let invalid_required_unchecked = SwitchProps {
        invalid: true,
        required: true,
        ..SwitchProps::default()
    };
    let readonly_checked = SwitchProps {
        readonly: true,
        ..SwitchProps::default()
    };
    let body = vec![
        switch_instance(true, &plain, "notifications", "Notifications"),
        switch_instance(false, &plain, "wifi", "Wi-Fi"),
        switch_instance(
            true,
            &disabled_checked,
            "airplane-mode",
            "Airplane mode (disabled)",
        ),
        switch_instance(false, &invalid_required_unchecked, "terms", "Accept terms"),
        switch_instance(true, &readonly_checked, "locked", "Locked (read-only)"),
    ];
    demo_page("Switch", body)
}

pub(super) fn tags_input_section() -> Node {
    let body = vec![tags_input::root(
        false,
        vec![],
        vec![
            tags_input::label(vec![], vec![text("Tags")]),
            tags_input::control(
                false,
                false,
                "Tags",
                vec![],
                vec![
                    tags_input::item(
                        false,
                        false,
                        vec![],
                        vec![tags_input::item_preview(
                            false,
                            vec![],
                            vec![
                                tags_input::item_text(vec![], vec![text("rust")]),
                                tags_input::item_delete_trigger(
                                    "rust",
                                    false,
                                    vec![],
                                    vec![text("×")],
                                ),
                            ],
                        )],
                    ),
                    tags_input::input("", false, false, vec![]),
                ],
            ),
            tags_input::clear_trigger(false, vec![], vec![text("Clear")]),
            tags_input::hidden_input("tags", "rust", false, vec![]),
            // live_region は control の兄弟として root 直下に置く（配置制約、
            // モジュール doc参照）。
            tags_input::live_region(vec![], vec![text("1 tag")]),
        ],
    )];
    demo_page("Tags Input", body)
}

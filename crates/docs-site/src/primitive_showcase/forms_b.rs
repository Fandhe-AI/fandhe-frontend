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
use hui::signature_pad::{self, Point, Stroke};
use hui::slider;
use hui::switch;
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

pub(super) fn segment_group_section() -> Node {
    let body = vec![segment_group::root(
        false,
        None,
        None,
        vec![],
        vec![
            segment_group::indicator(Some((0, 2)), None, vec![]),
            segment_group::item(
                true,
                false,
                "list",
                vec![],
                vec![
                    segment_group::item_control(true, false, vec![]),
                    segment_group::item_text(true, false, vec![], vec![text("List")]),
                    segment_group::item_hidden_input(true, false, Some("view"), "list", vec![]),
                ],
            ),
            segment_group::item(
                false,
                false,
                "grid",
                vec![],
                vec![
                    segment_group::item_control(false, false, vec![]),
                    segment_group::item_text(false, false, vec![], vec![text("Grid")]),
                    segment_group::item_hidden_input(false, false, Some("view"), "grid", vec![]),
                ],
            ),
        ],
    )];
    demo_page("Segment Group", body)
}

pub(super) fn select_section() -> Node {
    let state = OpenState::Open;
    let body = vec![select::root(
        state,
        vec![],
        vec![
            select::label(Some("sel-label"), vec![], vec![text("Fruit")]),
            select::control(
                state,
                vec![],
                vec![
                    select::trigger(
                        state,
                        false,
                        Some("sel-content"),
                        Some("sel-label"),
                        vec![],
                        vec![
                            select::value_text(false, vec![], vec![text("Apple")]),
                            select::indicator(state, vec![], vec![text("▾")]),
                        ],
                    ),
                    select::clear_trigger(vec![], vec![text("×")]),
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
                        None,
                        vec![],
                        vec![
                            select::item_group_label(None, vec![], vec![text("Common")]),
                            select::item(
                                OpenState::Open,
                                false,
                                true,
                                "apple",
                                Some("sel-item-0"),
                                vec![],
                                vec![
                                    select::item_text(None, vec![], vec![text("Apple")]),
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
                false,
                vec![],
                vec![("apple", "Apple"), ("banana", "Banana")],
            ),
        ],
    )];
    demo_page("Select", body)
}

pub(super) fn signature_pad_section() -> Node {
    let stroke = Stroke::new(vec![
        Point::new(4.0, 40.0),
        Point::new(40.0, 8.0),
        Point::new(80.0, 40.0),
    ]);
    let paths: Vec<Node> = match stroke {
        Ok(stroke) => vec![signature_pad::segment_path(&stroke, vec![])],
        Err(_) => vec![],
    };
    let body = vec![signature_pad::root(
        false,
        false,
        vec![],
        vec![
            signature_pad::label(vec![], vec![text("Signature")]),
            signature_pad::control(
                false,
                vec![],
                vec![
                    signature_pad::guide(vec![], vec![]),
                    signature_pad::segment(160, 60, Some("Signature preview"), vec![], paths),
                ],
            ),
            signature_pad::clear_trigger(false, vec![], vec![text("Clear")]),
            signature_pad::hidden_input("signature", "M4,40 L40,8 L80,40", false, vec![]),
        ],
    )];
    demo_page("Signature Pad", body)
}

pub(super) fn slider_section() -> Node {
    let orientation = Orientation::Horizontal;
    let body = vec![slider::root(
        orientation,
        false,
        vec![],
        vec![
            slider::label(vec![], vec![text("Volume")]),
            slider::control(
                orientation,
                false,
                vec![],
                vec![
                    slider::track(
                        orientation,
                        false,
                        vec![],
                        vec![slider::range(orientation, false, vec![], vec![])],
                    ),
                    slider::thumb(
                        orientation,
                        "0",
                        "100",
                        "40",
                        Some("40%"),
                        false,
                        vec![],
                        vec![],
                    ),
                ],
            ),
            slider::hidden_input("volume", "40", false, vec![]),
            slider::value_text(vec![], vec![text("40%")]),
        ],
    )];
    demo_page("Slider", body)
}

pub(super) fn switch_section() -> Node {
    let body = vec![switch::root(
        true,
        false,
        vec![],
        vec![
            switch::control(
                true,
                false,
                vec![],
                vec![switch::thumb(true, vec![], vec![])],
            ),
            switch::label(true, vec![], vec![text("Notifications")]),
            switch::hidden_input("notifications", "on", true, false, false, vec![]),
        ],
    )];
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

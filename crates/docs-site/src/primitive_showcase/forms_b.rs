//! Primitives Demo — Forms B（11 件、原稿は #1025）。
//! 執筆規約は `crate::primitive_showcase` モジュール doc 参照。

use fandhe_frontend_core::{text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::data_attrs::Orientation;
use hui::number_input::{self, NumberInputFlags};
use hui::password_input::{self, PasswordAutocomplete, PasswordInputProps};
use hui::pin_input::{self, PinInputKind};
use hui::radio_group;
use hui::rating_group::{self, RatingItemFlags};
use hui::segment_group;
use hui::select;
use hui::signature_pad::{self, Point, Stroke};
use hui::slider;
use hui::switch;
use hui::tags_input;
use hui::OpenState;

use super::demo_page;

pub(super) fn number_input_section() -> Node {
    let flags = NumberInputFlags::default();
    let body = vec![number_input::root(
        false,
        false,
        vec![],
        vec![
            number_input::label(
                false,
                false,
                Some("ni-input"),
                vec![],
                vec![text("Quantity")],
            ),
            number_input::control(
                false,
                false,
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
        ],
    )];
    demo_page("Number Input", body)
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

pub(super) fn pin_input_section() -> Node {
    let body = vec![pin_input::root(
        false,
        false,
        vec![],
        vec![
            pin_input::label(false, vec![], vec![text("One-time code")]),
            pin_input::control(
                vec![],
                vec![
                    pin_input::input(
                        0,
                        4,
                        "1",
                        PinInputKind::Numeric,
                        false,
                        true,
                        false,
                        false,
                        vec![],
                    ),
                    pin_input::input(
                        1,
                        4,
                        "2",
                        PinInputKind::Numeric,
                        false,
                        true,
                        false,
                        false,
                        vec![],
                    ),
                    pin_input::input(
                        2,
                        4,
                        "",
                        PinInputKind::Numeric,
                        false,
                        true,
                        false,
                        false,
                        vec![],
                    ),
                    pin_input::input(
                        3,
                        4,
                        "",
                        PinInputKind::Numeric,
                        false,
                        true,
                        false,
                        false,
                        vec![],
                    ),
                ],
            ),
            pin_input::hidden_input("otp", "12", false, vec![]),
        ],
    )];
    demo_page("Pin Input", body)
}

pub(super) fn radio_group_section() -> Node {
    let body = vec![radio_group::root(
        false,
        None,
        None,
        vec![],
        vec![
            radio_group::label(None, vec![], vec![text("Plan")]),
            radio_group::item(
                true,
                false,
                "monthly",
                vec![],
                vec![
                    radio_group::item_control(true, false, vec![]),
                    radio_group::item_text(true, false, vec![], vec![text("Monthly")]),
                    radio_group::item_hidden_input(true, false, Some("plan"), "monthly", vec![]),
                ],
            ),
            radio_group::item(
                false,
                false,
                "yearly",
                vec![],
                vec![
                    radio_group::item_control(false, false, vec![]),
                    radio_group::item_text(false, false, vec![], vec![text("Yearly")]),
                    radio_group::item_hidden_input(false, false, Some("plan"), "yearly", vec![]),
                ],
            ),
        ],
    )];
    demo_page("Radio Group", body)
}

pub(super) fn rating_group_section() -> Node {
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
        false,
        false,
        vec![],
        vec![
            rating_group::label(None, vec![], vec![text("Rating")]),
            rating_group::control(
                None,
                vec![],
                vec![mk(1, false, true), mk(2, true, true), mk(3, false, false)],
            ),
            rating_group::hidden_input(Some("rating"), "2", false, vec![]),
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

//! Primitives Demo — Forms A（11 件、原稿は #1024）。
//!
//! `crate::primitive_showcase` モジュール doc のデモ執筆規約（scope 最外殻・
//! パート網羅・見出しタグ回避・`primitives-demo-*` class 限定・静的初期状態・
//! ダミー文字列は無害なもの）に従う。各関数は `fandhe_frontend_pre_styled_ui`
//! の再エクスポート経由で headless-ui のパート関数のみを呼び、
//! `crate::primitive_showcase::mod::demo_page` が返す
//! `div > section > [h2, p, div.primitives-demo-frame]` へ組み込まれる。

use fandhe_frontend_core::{text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::checkbox::{CheckboxProps, CheckedState};
use hui::checkbox_group;
use hui::color_picker::{self, Channel};
use hui::combobox;
use hui::editable::{
    self, EditMode, EditableActivationMode, EditableInputFlags, EditableInputProps,
    EditableSubmitMode,
};
use hui::field::{self, FieldProps};
use hui::fieldset::{self, FieldsetProps};
use hui::file_upload;
use hui::image_cropper::{self, HandlePosition};
use hui::listbox;
use hui::{angle_slider, checkbox, OpenState};

use super::demo_page;

pub(super) fn angle_slider_section() -> Node {
    let body = vec![angle_slider::root(
        false,
        vec![],
        vec![
            angle_slider::label(vec![], vec![text("Direction")]),
            angle_slider::control(
                false,
                vec![],
                vec![angle_slider::thumb("135", "135deg", false, vec![], vec![])],
            ),
            angle_slider::hidden_input("direction", "135", false, vec![]),
            angle_slider::value_text(vec![], vec![text("135°")]),
        ],
    )];
    demo_page("Angle Slider", body)
}

/// `checkbox_section` の 1 インスタンス分を組み立てる非公開ヘルパ。
/// `name` はフォーム送信名（インスタンス間で一意な無害なダミー値）、
/// `label_text` は表示ラベル。ark-ui の Checkbox anatomy（root/control/
/// indicator/label/hidden-input の 5 パーツ）をそのまま踏襲する。
fn checkbox_instance(props: &CheckboxProps, name: &'static str, label_text: &'static str) -> Node {
    checkbox::root(
        props,
        vec![],
        vec![
            checkbox::control(
                props,
                vec![],
                vec![checkbox::indicator(props, vec![], vec![text("✓")])],
            ),
            checkbox::label(props, vec![], vec![text(label_text)]),
            checkbox::hidden_input(props, name, "on", vec![]),
        ],
    )
}

/// `data-state`（checked/unchecked/indeterminate の 3 値）と
/// `data-disabled`/`data-invalid`/`data-required`/`data-readonly` を
/// 描き分けた 6 インスタンスを並べる（イシュー #1602、参照突合の一環。
/// `Observed Values` を充実させるデモ執筆規約 5 の範囲内）。
pub(super) fn checkbox_section() -> Node {
    let checked = CheckboxProps {
        checked: CheckedState::Checked,
        ..Default::default()
    };
    let unchecked = CheckboxProps {
        checked: CheckedState::Unchecked,
        ..Default::default()
    };
    let indeterminate = CheckboxProps {
        checked: CheckedState::Indeterminate,
        ..Default::default()
    };
    let disabled_checked = CheckboxProps {
        checked: CheckedState::Checked,
        disabled: true,
        ..Default::default()
    };
    let invalid_required_unchecked = CheckboxProps {
        checked: CheckedState::Unchecked,
        invalid: true,
        required: true,
        ..Default::default()
    };
    let readonly_checked = CheckboxProps {
        checked: CheckedState::Checked,
        readonly: true,
        ..Default::default()
    };
    let body = vec![
        checkbox_instance(&checked, "newsletter", "Accept newsletter"),
        checkbox_instance(&unchecked, "updates", "Receive updates"),
        checkbox_instance(&indeterminate, "select-all", "Select all"),
        checkbox_instance(&disabled_checked, "archived", "Archived (disabled)"),
        checkbox_instance(&invalid_required_unchecked, "terms", "Accept terms"),
        checkbox_instance(&readonly_checked, "locked", "Locked (read-only)"),
    ];
    demo_page("Checkbox", body)
}

pub(super) fn checkbox_group_section() -> Node {
    let body = vec![checkbox_group::root(
        false,
        None,
        None,
        vec![],
        vec![
            checkbox_group::label(None, vec![], vec![text("Fruits")]),
            checkbox_group::item(
                true,
                false,
                "apple",
                vec![],
                vec![
                    checkbox_group::item_control(
                        true,
                        false,
                        vec![],
                        vec![checkbox_group::item_indicator(
                            true,
                            false,
                            vec![],
                            vec![text("✓")],
                        )],
                    ),
                    checkbox_group::item_text(true, false, vec![], vec![text("Apple")]),
                ],
            ),
            checkbox_group::item(
                false,
                false,
                "banana",
                vec![],
                vec![
                    checkbox_group::item_control(
                        false,
                        false,
                        vec![],
                        vec![checkbox_group::item_indicator(
                            false,
                            false,
                            vec![],
                            vec![text("✓")],
                        )],
                    ),
                    checkbox_group::item_text(false, false, vec![], vec![text("Banana")]),
                ],
            ),
        ],
    )];
    demo_page("Checkbox Group", body)
}

pub(super) fn color_picker_section() -> Node {
    let state = OpenState::Open;
    let body = vec![color_picker::root(
        state,
        vec![],
        vec![
            color_picker::label(vec![], vec![text("Color")]),
            color_picker::control(
                vec![],
                vec![color_picker::trigger(
                    state,
                    false,
                    Some("cp-content"),
                    vec![],
                    vec![text("#3366ff")],
                )],
            ),
            color_picker::positioner(
                state,
                vec![],
                vec![color_picker::content(
                    state,
                    Some("cp-content"),
                    vec![],
                    vec![
                        color_picker::area(
                            vec![],
                            vec![
                                color_picker::area_background(vec![], vec![]),
                                color_picker::area_thumb("#3366ff", false, vec![], vec![]),
                            ],
                        ),
                        color_picker::channel_slider(
                            Channel::Hue,
                            vec![],
                            vec![color_picker::channel_slider_track(
                                Channel::Hue,
                                vec![],
                                vec![color_picker::channel_slider_thumb(
                                    Channel::Hue,
                                    "0",
                                    "359",
                                    "220",
                                    false,
                                    vec![],
                                    vec![],
                                )],
                            )],
                        ),
                        color_picker::channel_input("#3366ff", false, vec![]),
                        color_picker::value_text(vec![], vec![text("#3366ff")]),
                        color_picker::hidden_input("color", "#3366ff", false, vec![]),
                    ],
                )],
            ),
        ],
    )];
    demo_page("Color Picker", body)
}

pub(super) fn combobox_section() -> Node {
    let state = OpenState::Open;
    let body = vec![combobox::root(
        state,
        vec![],
        vec![
            combobox::label(
                Some("cb-label"),
                Some("cb-input"),
                vec![],
                vec![text("Fruit")],
            ),
            combobox::control(
                state,
                vec![],
                vec![
                    combobox::input(
                        state,
                        "Ap",
                        false,
                        Some("cb-content"),
                        Some("cb-item-0"),
                        Some("fruit"),
                        vec![("id", "cb-input")],
                    ),
                    combobox::trigger(state, false, Some("cb-content"), vec![], vec![text("▾")]),
                    combobox::clear_trigger(vec![], vec![text("×")]),
                ],
            ),
            combobox::positioner(
                state,
                vec![],
                vec![combobox::content(
                    state,
                    Some("cb-content"),
                    Some("cb-label"),
                    vec![],
                    vec![combobox::item_group(
                        None,
                        vec![],
                        vec![combobox::item(
                            OpenState::Open,
                            false,
                            true,
                            "apple",
                            Some("cb-item-0"),
                            vec![],
                            vec![
                                combobox::item_text(
                                    Some("cb-item-0-text"),
                                    vec![],
                                    vec![text("Apple")],
                                ),
                                combobox::item_indicator(OpenState::Open, vec![], vec![text("✓")]),
                            ],
                        )],
                    )],
                )],
            ),
            // live_region は control の兄弟として root 直下に置く（listbox
            // の許容子ロールに反しないための配置制約、モジュール doc参照）。
            combobox::live_region(vec![], vec![text("1 result available")]),
        ],
    )];
    demo_page("Combobox", body)
}

pub(super) fn editable_section() -> Node {
    let mode = EditMode::Preview;
    let body = vec![editable::root(
        mode,
        false,
        false,
        EditableActivationMode::Focus,
        EditableSubmitMode::Both,
        vec![],
        vec![
            editable::label(
                mode,
                false,
                Some("editable-input"),
                vec![],
                vec![text("Display name")],
            ),
            editable::area(
                mode,
                false,
                vec![],
                vec![
                    editable::preview(mode, false, vec![], vec![text("Ada Lovelace")]),
                    editable::input(
                        mode,
                        "display-name",
                        "Ada Lovelace",
                        EditableInputProps {
                            id: Some("editable-input"),
                            placeholder: None,
                            max_length: None,
                        },
                        EditableInputFlags::default(),
                        vec![],
                    ),
                ],
            ),
            editable::control(
                mode,
                vec![],
                vec![
                    editable::edit_trigger(mode, false, vec![], vec![text("Edit")]),
                    editable::submit_trigger(mode, false, vec![], vec![text("Save")]),
                    editable::cancel_trigger(mode, false, vec![], vec![text("Cancel")]),
                ],
            ),
        ],
    )];
    demo_page("Editable", body)
}

pub(super) fn field_section() -> Node {
    let props = FieldProps {
        id: "field-email",
        ids: Default::default(),
        disabled: false,
        invalid: true,
        required: true,
        readonly: false,
        has_helper_text: true,
    };
    let body = vec![field::root(
        &props,
        vec![],
        vec![
            field::label(
                &props,
                vec![],
                vec![
                    text("Email"),
                    field::required_indicator(&props, vec![], vec![text("*")]),
                ],
            ),
            field::input(
                &props,
                vec![("type", "email"), ("name", "email"), ("value", "")],
            ),
            field::helper_text(&props, vec![], vec![text("Work email preferred.")]),
            field::error_text(&props, vec![], vec![text("Enter a valid email address.")]),
        ],
    )];
    demo_page("Field", body)
}

pub(super) fn fieldset_section() -> Node {
    let props = FieldsetProps {
        id: "fieldset-shipping",
        disabled: false,
        invalid: false,
        has_helper_text: true,
    };
    let body = vec![fieldset::root(
        &props,
        vec![],
        vec![
            fieldset::legend(&props, vec![], vec![text("Shipping address")]),
            fieldset::helper_text(&props, vec![], vec![text("Used for delivery only.")]),
            fieldset::error_text(&props, vec![], vec![text("Address is required.")]),
        ],
    )];
    demo_page("Fieldset", body)
}

pub(super) fn file_upload_section() -> Node {
    let body = vec![file_upload::root(
        false,
        vec![],
        vec![
            file_upload::label(vec![], vec![text("Attachments")]),
            file_upload::dropzone(
                false,
                false,
                vec![("aria-label", "Drop files here")],
                vec![
                    text("Drag files here or"),
                    file_upload::trigger(false, vec![], vec![text("Browse")]),
                    file_upload::hidden_input("image/*", true, false, vec![]),
                ],
            ),
            file_upload::item_group(
                vec![],
                vec![file_upload::item(
                    false,
                    vec![],
                    vec![
                        file_upload::item_name(vec![], vec![text("photo.png")]),
                        file_upload::item_size_text_node(
                            vec![],
                            vec![text(file_upload::item_size_text(204_800))],
                        ),
                        file_upload::item_delete_trigger(
                            "photo.png",
                            false,
                            vec![],
                            vec![text("Remove")],
                        ),
                    ],
                )],
            ),
            file_upload::clear_trigger(false, vec![], vec![text("Clear all")]),
        ],
    )];
    demo_page("File Upload", body)
}

pub(super) fn image_cropper_section() -> Node {
    let body = vec![image_cropper::root(
        vec![],
        vec![
            image_cropper::viewport(
                vec![],
                vec![image_cropper::image(
                    "https://example.com/sample.jpg",
                    "Sample photo to crop",
                    vec![],
                )],
            ),
            image_cropper::selection(
                vec![],
                vec![
                    image_cropper::handle(HandlePosition::N, vec![]),
                    image_cropper::handle(HandlePosition::S, vec![]),
                    image_cropper::handle(HandlePosition::E, vec![]),
                    image_cropper::handle(HandlePosition::W, vec![]),
                ],
            ),
            image_cropper::grid(vec![]),
        ],
    )];
    demo_page("Image Cropper", body)
}

pub(super) fn listbox_section() -> Node {
    let selection_state = OpenState::Open;
    let body = vec![listbox::root(
        selection_state,
        false,
        vec![],
        vec![
            listbox::label(Some("lb-label"), vec![], vec![text("Fruit")]),
            listbox::content(
                false,
                Some("lb-content"),
                Some("lb-label"),
                Some("lb-item-0"),
                vec![],
                vec![listbox::item_group(
                    None,
                    vec![],
                    vec![
                        listbox::item_group_label(None, vec![], vec![text("Common")]),
                        listbox::item(
                            OpenState::Open,
                            false,
                            true,
                            "apple",
                            Some("lb-item-0"),
                            vec![],
                            vec![
                                listbox::item_text(None, vec![], vec![text("Apple")]),
                                listbox::item_indicator(OpenState::Open, vec![], vec![text("✓")]),
                            ],
                        ),
                        listbox::item(
                            OpenState::Closed,
                            false,
                            false,
                            "banana",
                            None,
                            vec![],
                            vec![
                                listbox::item_text(None, vec![], vec![text("Banana")]),
                                listbox::item_indicator(OpenState::Closed, vec![], vec![text("✓")]),
                            ],
                        ),
                    ],
                )],
            ),
            listbox::value_text(false, vec![], vec![text("Apple")]),
        ],
    )];
    demo_page("Listbox", body)
}

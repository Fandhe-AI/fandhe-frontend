//! Primitives Demo — Forms A（11 件、原稿は #1024）。
//!
//! `crate::primitive_showcase` モジュール doc のデモ執筆規約（scope 最外殻・
//! パート網羅・見出しタグ回避・`primitives-demo-*` class 限定・静的初期状態・
//! ダミー文字列は無害なもの）に従う。各関数は `fandhe_frontend_pre_styled_ui`
//! の再エクスポート経由で headless-ui のパート関数のみを呼び、
//! `crate::primitive_showcase::mod::demo_page` が返す
//! `div > section > [h2, p, div.primitives-demo-frame]` へ組み込まれる。

use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::angle_slider::AngleSliderProps;
use hui::checkbox::{CheckboxProps, CheckedState};
use hui::checkbox_group;
use hui::color_picker::{self, Channel, ColorPickerProps};
use hui::combobox;
use hui::editable::{
    self, EditMode, EditableActivationMode, EditableInputFlags, EditableInputProps,
    EditableSubmitMode,
};
use hui::field::{self, FieldProps};
use hui::fieldset::{self, FieldsetProps};
use hui::file_upload;
use hui::image_cropper::{self, GridAxis, HandlePosition, ImageCropper, ImageCropperProps};
use hui::listbox;
use hui::{angle_slider, checkbox, OpenState};

use super::demo_page;

/// Angle Slider の Demo（イシュー #1601 参照突合）。
///
/// 主インスタンス（135°）は MarkerGroup/Marker を含み、90/135/180 の 3 目盛り
/// で `data-state` の 3 値（`under-value`/`at-value`/`over-value`）すべてを
/// 露出する。第 2 インスタンス（45°）は `readonly`/`invalid` を root/label/
/// control/thumb へ反映し、それらの `data-*` 表出を Demo 上で確認できる
/// ようにする（`crate::primitive_showcase` モジュール doc の「パート網羅」
/// 規約対応）。
pub(super) fn angle_slider_section() -> Node {
    let primary_props = AngleSliderProps::default();
    let readonly_invalid_props = AngleSliderProps {
        readonly: true,
        invalid: true,
        ..Default::default()
    };
    let body = vec![angle_slider::root(
        &primary_props,
        vec![],
        vec![
            angle_slider::label(&primary_props, vec![], vec![text("Direction")]),
            angle_slider::control(
                &primary_props,
                vec![],
                vec![
                    angle_slider::thumb("135", "135deg", &primary_props, vec![], vec![]),
                    angle_slider::marker_group(
                        vec![],
                        vec![
                            angle_slider::marker(90, 135, false, vec![], vec![]),
                            angle_slider::marker(135, 135, false, vec![], vec![]),
                            angle_slider::marker(180, 135, false, vec![], vec![]),
                        ],
                    ),
                ],
            ),
            angle_slider::hidden_input("direction", "135", false, vec![]),
            angle_slider::value_text(vec![], vec![text("135°")]),
        ],
    )];
    let readonly_body = vec![angle_slider::root(
        &readonly_invalid_props,
        vec![],
        vec![
            angle_slider::label(
                &readonly_invalid_props,
                vec![],
                vec![text("Direction (readonly, invalid)")],
            ),
            angle_slider::control(
                &readonly_invalid_props,
                vec![],
                vec![angle_slider::thumb(
                    "45",
                    "45deg",
                    &readonly_invalid_props,
                    vec![],
                    vec![],
                )],
            ),
            angle_slider::hidden_input("direction-readonly", "45", false, vec![]),
            angle_slider::value_text(vec![], vec![text("45°")]),
        ],
    )];
    demo_page("Angle Slider", [body, readonly_body].concat())
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

/// [`checkbox_group::item`]/[`checkbox_group::item_control`]/
/// [`checkbox_group::item_indicator`]/[`checkbox_group::item_text`] を
/// まとめて組み立てる非公開ヘルパ（イシュー #1603、`checkbox_group_section`
/// のみが呼ぶ）。デモ執筆規約 1（`checkbox::hidden_input` は Demo に含めない）
/// を維持したまま、2 グループ構成への拡張で生じる重複を避ける。
fn checkbox_group_item(
    checked: bool,
    props: &checkbox_group::CheckboxGroupProps,
    value: &str,
    label: &str,
) -> Node {
    checkbox_group::item(
        checked,
        props,
        value,
        vec![],
        vec![
            checkbox_group::item_control(
                checked,
                props,
                vec![],
                vec![checkbox_group::item_indicator(
                    checked,
                    props,
                    vec![],
                    vec![text("✓")],
                )],
            ),
            checkbox_group::item_text(checked, props, vec![], vec![text(label)]),
        ],
    )
}

pub(super) fn checkbox_group_section() -> Node {
    // グループ 1: 縦積み・aria-labelledby 付き・checked/unchecked/disabled
    // の 3 項目（Radix Themes デモの 3 項目構成に合わせる、イシュー #1603）。
    let default_props = checkbox_group::CheckboxGroupProps::default();
    let disabled_item_props = checkbox_group::CheckboxGroupProps {
        disabled: true,
        ..Default::default()
    };
    let group1 = checkbox_group::root(
        &default_props,
        Some(hui::Orientation::Vertical),
        Some("primitives-checkbox-group-fruits-label"),
        vec![],
        vec![
            checkbox_group::label(
                Some("primitives-checkbox-group-fruits-label"),
                vec![],
                vec![text("Fruits")],
            ),
            checkbox_group_item(true, &default_props, "apple", "Apple"),
            checkbox_group_item(false, &default_props, "banana", "Banana"),
            checkbox_group_item(false, &disabled_item_props, "cherry", "Cherry (disabled)"),
        ],
    );

    // グループ 2: invalid + readonly + 横並びで data-invalid/data-readonly/
    // data-orientation="horizontal" を Observed Values に出す（イシュー
    // #1603 D4）。
    let invalid_readonly_props = checkbox_group::CheckboxGroupProps {
        disabled: false,
        readonly: true,
        invalid: true,
    };
    let group2 = checkbox_group::root(
        &invalid_readonly_props,
        Some(hui::Orientation::Horizontal),
        Some("primitives-checkbox-group-veggies-label"),
        vec![],
        vec![
            checkbox_group::label(
                Some("primitives-checkbox-group-veggies-label"),
                vec![],
                vec![text("Vegetables")],
            ),
            checkbox_group_item(true, &invalid_readonly_props, "carrot", "Carrot"),
            checkbox_group_item(false, &invalid_readonly_props, "potato", "Potato"),
        ],
    );

    demo_page("Checkbox Group", vec![group1, group2])
}

pub(super) fn color_picker_section() -> Node {
    let state = OpenState::Open;
    let none = ColorPickerProps::default();
    let body = vec![color_picker::root(
        state,
        &none,
        vec![],
        vec![
            color_picker::label(&none, vec![], vec![text("Color")]),
            color_picker::control(
                state,
                &none,
                vec![],
                vec![color_picker::trigger(
                    state,
                    &none,
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
                            &none,
                            vec![],
                            vec![
                                color_picker::area_background(&none, vec![], vec![]),
                                color_picker::area_thumb("#3366ff", &none, vec![], vec![]),
                            ],
                        ),
                        color_picker::channel_slider(
                            Channel::Hue,
                            hui::Orientation::Horizontal,
                            vec![],
                            vec![color_picker::channel_slider_track(
                                Channel::Hue,
                                hui::Orientation::Horizontal,
                                vec![],
                                vec![color_picker::channel_slider_thumb(
                                    Channel::Hue,
                                    hui::Orientation::Horizontal,
                                    "0",
                                    "359",
                                    "220",
                                    &none,
                                    vec![],
                                    vec![],
                                )],
                            )],
                        ),
                        color_picker::channel_input("#3366ff", &none, vec![]),
                        color_picker::value_text(&none, vec![], vec![text("#3366ff")]),
                        color_picker::hidden_input("color", "#3366ff", &none, vec![]),
                    ],
                )],
            ),
        ],
    )];

    // イシュー #1604: readonly/invalid/required を一律付与した閉状態を
    // 第 2 インスタンスとして追加し、`data-readonly`/`data-invalid`/
    // `data-required`（label のみ）と control の `data-state="closed"` を
    // Demo 上に露出する（`id` は `cp-content-ro` として上の
    // `cp-content`・`ex_color_picker` の `cp-content-2` と衝突させない）。
    let closed = OpenState::Closed;
    let ro = ColorPickerProps {
        readonly: true,
        invalid: true,
        required: true,
        ..ColorPickerProps::default()
    };
    let readonly_body = vec![color_picker::root(
        closed,
        &ro,
        vec![],
        vec![
            color_picker::label(&ro, vec![], vec![text("Color (readonly/invalid)")]),
            color_picker::control(
                closed,
                &ro,
                vec![],
                vec![color_picker::trigger(
                    closed,
                    &ro,
                    Some("cp-content-ro"),
                    vec![],
                    vec![text("#3366ff")],
                )],
            ),
            color_picker::positioner(
                closed,
                vec![],
                vec![color_picker::content(
                    closed,
                    Some("cp-content-ro"),
                    vec![],
                    vec![
                        color_picker::channel_input("#3366ff", &ro, vec![]),
                        color_picker::hidden_input("color-ro", "#3366ff", &ro, vec![]),
                    ],
                )],
            ),
        ],
    )];

    demo_page("Color Picker", [body, readonly_body].concat())
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

/// `editable_section` の 1 インスタンス分を組み立てる非公開ヘルパ
/// （イシュー #1606、参照突合の一環）。`id_prefix` は `label`/`input` の
/// `id`/`for` 関連付けと `name` をインスタンス間で一意化するために使う。
/// `flags` はそのまま root/label/area/preview/input の全パーツへ共通で渡し
/// （headless-ui 側の共有契約、`crates/headless-ui/src/editable.rs` module
/// doc「参照突合」節参照）、`placeholder_shown` が `true` のときは
/// `value`/`preview` テキストを空にして `data-placeholder-shown` の表出を
/// Demo 上で確認できるようにする。
fn editable_instance(
    id_prefix: &'static str,
    mode: EditMode,
    flags: EditableInputFlags,
    label_text: &'static str,
    value: &'static str,
    placeholder_shown: bool,
) -> Node {
    let input_id = format!("editable-{id_prefix}-input");
    let name = format!("editable-{id_prefix}-name");
    editable::root(
        mode,
        flags,
        EditableActivationMode::Focus,
        EditableSubmitMode::Both,
        vec![],
        vec![
            editable::label(
                mode,
                flags,
                Some(input_id.as_str()),
                vec![],
                vec![text(label_text)],
            ),
            editable::area(
                mode,
                flags,
                placeholder_shown,
                vec![],
                vec![
                    editable::preview(mode, flags, placeholder_shown, vec![], vec![text(value)]),
                    editable::input(
                        mode,
                        name.as_str(),
                        value,
                        EditableInputProps {
                            id: Some(input_id.as_str()),
                            placeholder: Some("Enter a value"),
                            max_length: None,
                        },
                        flags,
                        vec![],
                    ),
                ],
            ),
            editable::control(
                mode,
                vec![],
                vec![
                    editable::edit_trigger(mode, flags.disabled, vec![], vec![text("Edit")]),
                    editable::submit_trigger(mode, flags.disabled, vec![], vec![text("Save")]),
                    editable::cancel_trigger(mode, flags.disabled, vec![], vec![text("Cancel")]),
                ],
            ),
        ],
    )
}

/// preview/edit/disabled/readonly/invalid+required/空値（placeholder）の
/// 6 インスタンスを並べ、9 anatomy パーツすべてと `data-invalid`/
/// `data-required`/`data-disabled`/`data-readonly`/`data-placeholder-shown`
/// の表出を Demo 上で確認できるようにする（イシュー #1606、参照突合の
/// 一環。`crate::primitive_showcase` モジュール doc の「パート網羅」規約
/// 対応）。
pub(super) fn editable_section() -> Node {
    let body = vec![
        editable_instance(
            "preview",
            EditMode::Preview,
            EditableInputFlags::default(),
            "Display name",
            "Ada Lovelace",
            false,
        ),
        editable_instance(
            "edit",
            EditMode::Edit,
            EditableInputFlags::default(),
            "Display name (editing)",
            "Grace Hopper",
            false,
        ),
        editable_instance(
            "disabled",
            EditMode::Preview,
            EditableInputFlags {
                disabled: true,
                ..EditableInputFlags::default()
            },
            "Display name (disabled)",
            "Locked value",
            false,
        ),
        editable_instance(
            "readonly",
            EditMode::Preview,
            EditableInputFlags {
                readonly: true,
                ..EditableInputFlags::default()
            },
            "Display name (readonly)",
            "Read-only value",
            false,
        ),
        editable_instance(
            "invalid",
            EditMode::Edit,
            EditableInputFlags {
                invalid: true,
                required: true,
                ..EditableInputFlags::default()
            },
            "Display name (invalid, required)",
            "",
            false,
        ),
        editable_instance(
            "empty",
            EditMode::Preview,
            EditableInputFlags::default(),
            "Display name (empty)",
            "",
            true,
        ),
    ];
    demo_page("Editable", body)
}

/// `field_section` の 1 インスタンス分を組み立てる非公開ヘルパ（イシュー
/// #1607、`checkbox_instance` と同型）。`control` は呼び出し側が
/// `field::input`/`field::textarea`/`field::select` のいずれかを組み立てて
/// 渡す（「1 Field = 1 コントロール」契約、`field.rs` モジュール doc 参照）。
/// `helper`/`error` は `Some` のときのみ対応パーツを描画し、`props.
/// has_helper_text` は実際に `helper_text` パーツを描画するインスタンスの
/// みで `true` にする呼び出し側契約を守る。
fn field_instance(
    props: &FieldProps<'_>,
    control: Node,
    label_text: &'static str,
    helper: Option<&'static str>,
    error: Option<&'static str>,
) -> Node {
    let mut children = vec![field::label(
        props,
        vec![],
        vec![
            text(label_text),
            field::required_indicator(props, vec![], vec![text("*")]),
        ],
    )];
    children.push(control);
    if let Some(helper) = helper {
        children.push(field::helper_text(props, vec![], vec![text(helper)]));
    }
    if let Some(error) = error {
        children.push(field::error_text(props, vec![], vec![text(error)]));
    }
    field::root(props, vec![], children)
}

/// 8 パーツ（root/label/input/textarea/select/helper-text/error-text/
/// required-indicator）を 7 インスタンスへ描き分ける（イシュー #1607、
/// 参照突合の一環）。既定（helper 併用）/invalid（error_text 併用で
/// `aria-describedby` の 2 id 合成を観測）/disabled/readonly/required（helper
/// なし）/textarea（`autoresize` で `data-autoresize` を露出）/select の
/// 順に並べ、textarea・select を Demo に含めることで
/// `crates/docs-site/tests/primitive_showcase.rs::KNOWN_UNCOVERED` の
/// `("field","select",…)`/`("field","textarea",…)` 免除を不要にする。
pub(super) fn field_section() -> Node {
    let default_props = FieldProps {
        id: "field-email",
        ids: Default::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: true,
    };
    let invalid_props = FieldProps {
        id: "field-username",
        ids: Default::default(),
        disabled: false,
        invalid: true,
        required: false,
        readonly: false,
        has_helper_text: true,
    };
    let disabled_props = FieldProps {
        id: "field-plan",
        ids: Default::default(),
        disabled: true,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let readonly_props = FieldProps {
        id: "field-account-id",
        ids: Default::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: true,
        has_helper_text: false,
    };
    let required_props = FieldProps {
        id: "field-full-name",
        ids: Default::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let textarea_props = FieldProps {
        id: "field-bio",
        ids: Default::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: true,
    };
    let select_props = FieldProps {
        id: "field-country",
        ids: Default::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };

    let body = vec![
        field_instance(
            &default_props,
            field::input(
                &default_props,
                vec![("type", "email"), ("name", "email"), ("value", "")],
            ),
            "Email",
            Some("Work email preferred."),
            None,
        ),
        field_instance(
            &invalid_props,
            field::input(
                &invalid_props,
                vec![("type", "text"), ("name", "username"), ("value", "")],
            ),
            "Username",
            Some("Letters, numbers and underscores only."),
            Some("This username is already taken."),
        ),
        field_instance(
            &disabled_props,
            field::input(
                &disabled_props,
                vec![("type", "text"), ("name", "plan"), ("value", "Free")],
            ),
            "Plan",
            None,
            None,
        ),
        field_instance(
            &readonly_props,
            field::input(
                &readonly_props,
                vec![
                    ("type", "text"),
                    ("name", "account-id"),
                    ("value", "acct_example123"),
                ],
            ),
            "Account ID",
            None,
            None,
        ),
        field_instance(
            &required_props,
            field::input(
                &required_props,
                vec![("type", "text"), ("name", "full-name"), ("value", "")],
            ),
            "Full name",
            None,
            None,
        ),
        field_instance(
            &textarea_props,
            field::textarea(&textarea_props, true, vec![("name", "bio")], vec![text("")]),
            "Bio",
            Some("Autoresizes as you type."),
            None,
        ),
        field_instance(
            &select_props,
            field::select(
                &select_props,
                vec![("name", "country")],
                vec![
                    el("option", vec![("value", "jp")], vec![text("Japan")]),
                    el("option", vec![("value", "us")], vec![text("United States")]),
                    el("option", vec![("value", "other")], vec![text("Other")]),
                ],
            ),
            "Country",
            None,
            None,
        ),
    ];
    demo_page("Field", body)
}

/// 4 パーツ（root/legend/helper-text/error-text）を basic/disabled/invalid
/// の 3 インスタンスへ描き分ける（イシュー #1608、参照突合の一環）。参照
/// スクリーンショット（ark-fieldset-1〜3・chakra-fieldset-1〜3）に倣い、
/// 各インスタンスは内包 [`field::root`]（`fs_props.merge_field_props` で
/// 合成した [`FieldProps`]）を 1〜2 件含めることで `data-disabled`/
/// `data-invalid`（`data-*` 表の機械導出元）を Demo 木へ露出させる。内包
/// Field により Demo 木に `data-scope="field"` が混在するが、
/// `component_page.rs` の scope 解決は最外側・初出の `data-scope`
/// （fieldset root）を採用し、Anatomy・`data-*` 走査は scope 一致で
/// フィルタするため Field 側のパートは漏れない（確認済み）。
pub(super) fn fieldset_section() -> Node {
    let basic_props = FieldsetProps {
        id: "fieldset-contact",
        disabled: false,
        invalid: false,
        has_helper_text: true,
    };
    let disabled_props = FieldsetProps {
        id: "fieldset-shipping-disabled",
        disabled: true,
        invalid: false,
        has_helper_text: false,
    };
    let invalid_props = FieldsetProps {
        id: "fieldset-shipping-invalid",
        disabled: false,
        invalid: true,
        has_helper_text: false,
    };

    let basic_name_field = basic_props.merge_field_props(FieldProps {
        id: "field-contact-name",
        ids: Default::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    });
    let basic_email_field = basic_props.merge_field_props(FieldProps {
        id: "field-contact-email",
        ids: Default::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    });
    let disabled_name_field = disabled_props.merge_field_props(FieldProps {
        id: "field-shipping-name",
        ids: Default::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    });
    let disabled_address_field = disabled_props.merge_field_props(FieldProps {
        id: "field-shipping-address",
        ids: Default::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    });
    let invalid_name_field = invalid_props.merge_field_props(FieldProps {
        id: "field-shipping-invalid-name",
        ids: Default::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    });

    let body = vec![
        // basic: legend + helper_text + 内包 Field 2 件（error_text は
        // 非 invalid のため hidden）。
        fieldset::root(
            &basic_props,
            vec![],
            vec![
                fieldset::legend(&basic_props, vec![], vec![text("Contact details")]),
                fieldset::helper_text(
                    &basic_props,
                    vec![],
                    vec![text("Please provide your contact details below.")],
                ),
                field_instance(
                    &basic_name_field,
                    field::input(
                        &basic_name_field,
                        vec![("type", "text"), ("name", "contact-name"), ("value", "")],
                    ),
                    "Name",
                    None,
                    None,
                ),
                field_instance(
                    &basic_email_field,
                    field::input(
                        &basic_email_field,
                        vec![("type", "email"), ("name", "contact-email"), ("value", "")],
                    ),
                    "Email",
                    None,
                    None,
                ),
                fieldset::error_text(&basic_props, vec![], vec![text("Some fields are invalid.")]),
            ],
        ),
        // disabled: legend + 内包 Field 2 件（merge_field_props の OR 伝播で
        // Field 側にも data-disabled/disabled が出る）。
        fieldset::root(
            &disabled_props,
            vec![],
            vec![
                fieldset::legend(&disabled_props, vec![], vec![text("Shipping details")]),
                field_instance(
                    &disabled_name_field,
                    field::input(
                        &disabled_name_field,
                        vec![("type", "text"), ("name", "shipping-name"), ("value", "")],
                    ),
                    "Name",
                    None,
                    None,
                ),
                field_instance(
                    &disabled_address_field,
                    field::input(
                        &disabled_address_field,
                        vec![
                            ("type", "text"),
                            ("name", "shipping-address"),
                            ("value", ""),
                        ],
                    ),
                    "Address",
                    None,
                    None,
                ),
            ],
        ),
        // invalid: legend + 内包 Field 1 件 + error_text（表示状態）。
        fieldset::root(
            &invalid_props,
            vec![],
            vec![
                fieldset::legend(&invalid_props, vec![], vec![text("Shipping details")]),
                field_instance(
                    &invalid_name_field,
                    field::input(
                        &invalid_name_field,
                        vec![
                            ("type", "text"),
                            ("name", "shipping-invalid-name"),
                            ("value", ""),
                        ],
                    ),
                    "Name",
                    None,
                    None,
                ),
                fieldset::error_text(
                    &invalid_props,
                    vec![],
                    vec![text("Some fields are invalid. Please check them.")],
                ),
            ],
        ),
    ];
    demo_page("Fieldset", body)
}

/// [`file_upload::FileUploadProps`] を状態別に組み立てる非公開ヘルパ
/// （既定/disabled/readonly+invalid+required の 3 状態を Demo が並べる、
/// イシュー #1609 参照突合）。
fn file_upload_props(
    disabled: bool,
    readonly: bool,
    invalid: bool,
    required: bool,
) -> file_upload::FileUploadProps {
    file_upload::FileUploadProps {
        disabled,
        readonly,
        invalid,
        required,
    }
}

/// 1 個の受理済み/拒否済みファイル `item` を組み立てる非公開ヘルパ。
fn file_upload_item(
    name: &'static str,
    size_bytes: u64,
    item_type: file_upload::ItemType,
    props: &file_upload::FileUploadProps,
) -> Node {
    file_upload::item(
        item_type,
        props,
        vec![],
        vec![
            file_upload::item_name(item_type, props, vec![], vec![text(name)]),
            file_upload::item_size_text_node(
                item_type,
                props,
                vec![],
                vec![text(file_upload::item_size_text(size_bytes))],
            ),
            file_upload::item_delete_trigger(name, item_type, props, vec![], vec![text("Remove")]),
        ],
    )
}

/// 1 セット分（root > label + dropzone(trigger + hidden-input) +
/// item-group(accepted) + item-group(rejected) + clear-trigger）を組み立てる
/// 非公開ヘルパ。11 anatomy パーツ全てを毎回含める。
fn file_upload_instance(
    caption: &'static str,
    props: file_upload::FileUploadProps,
    dragging: bool,
    has_items: bool,
) -> Node {
    let accepted_items = if has_items {
        vec![file_upload_item(
            "photo.png",
            204_800,
            file_upload::ItemType::Accepted,
            &props,
        )]
    } else {
        vec![]
    };
    let rejected_items = if has_items {
        vec![file_upload_item(
            "malware.exe",
            10_240,
            file_upload::ItemType::Rejected,
            &props,
        )]
    } else {
        vec![]
    };
    fandhe_frontend_core::el(
        "div",
        vec![],
        vec![
            fandhe_frontend_core::el("p", vec![], vec![text(caption)]),
            file_upload::root(
                &props,
                dragging,
                vec![],
                vec![
                    file_upload::label(&props, vec![], vec![text("Attachments")]),
                    file_upload::dropzone(
                        &props,
                        dragging,
                        vec![],
                        vec![
                            text("Drag files here or"),
                            file_upload::trigger(&props, vec![], vec![text("Browse")]),
                            file_upload::hidden_input("image/*", true, &props, vec![]),
                        ],
                    ),
                    file_upload::item_group(
                        file_upload::ItemType::Accepted,
                        &props,
                        vec![],
                        accepted_items,
                    ),
                    file_upload::item_group(
                        file_upload::ItemType::Rejected,
                        &props,
                        vec![],
                        rejected_items,
                    ),
                    file_upload::clear_trigger(&props, !has_items, vec![], vec![text("Clear all")]),
                ],
            ),
        ],
    )
}

pub(super) fn file_upload_section() -> Node {
    let body = vec![
        file_upload_instance(
            "Default（受理済み + 拒否済みファイル各 1 件）",
            file_upload_props(false, false, false, false),
            false,
            true,
        ),
        file_upload_instance(
            "Disabled",
            file_upload_props(true, false, false, false),
            false,
            false,
        ),
        file_upload_instance(
            "Readonly + Invalid + Required",
            file_upload_props(false, true, true, true),
            false,
            false,
        ),
        file_upload_instance(
            "Dragging（data-dragging）",
            file_upload_props(false, false, false, false),
            true,
            false,
        ),
    ];
    demo_page("File Upload", body)
}

/// イシュー #1610（参照実装突合）: `ImageCropperProps`（`data-disabled`/
/// `data-dragging`）・`GridAxis`（`data-axis`）を含む `data-*` 表を機械
/// 導出するため、既定 props（8 方位 handle + 横軸/縦軸 grid）と
/// disabled + dragging props（`grid(None, ..)`）の 2 インスタンスを並べる
/// （`angle_slider_section` の既定/readonly・invalid 2 インスタンス構成と
/// 同型）。
pub(super) fn image_cropper_section() -> Node {
    let state = ImageCropper::default();
    let default_props = ImageCropperProps::default();
    let body = vec![
        image_cropper::root(
            &default_props,
            vec![],
            vec![
                image_cropper::viewport(
                    &default_props,
                    vec![],
                    vec![image_cropper::image(
                        "https://example.com/sample.jpg",
                        "Sample photo to crop",
                        vec![],
                    )],
                ),
                image_cropper::selection(
                    &state,
                    &default_props,
                    vec![],
                    vec![
                        image_cropper::handle(HandlePosition::N, &default_props, vec![]),
                        image_cropper::handle(HandlePosition::S, &default_props, vec![]),
                        image_cropper::handle(HandlePosition::E, &default_props, vec![]),
                        image_cropper::handle(HandlePosition::W, &default_props, vec![]),
                        image_cropper::handle(HandlePosition::Ne, &default_props, vec![]),
                        image_cropper::handle(HandlePosition::Nw, &default_props, vec![]),
                        image_cropper::handle(HandlePosition::Se, &default_props, vec![]),
                        image_cropper::handle(HandlePosition::Sw, &default_props, vec![]),
                    ],
                ),
                image_cropper::grid(Some(GridAxis::Horizontal), &default_props, vec![]),
                image_cropper::grid(Some(GridAxis::Vertical), &default_props, vec![]),
            ],
        ),
        image_cropper::root(
            &ImageCropperProps {
                disabled: true,
                dragging: true,
            },
            vec![],
            vec![
                image_cropper::viewport(
                    &ImageCropperProps {
                        disabled: true,
                        dragging: true,
                    },
                    vec![],
                    vec![image_cropper::image(
                        "https://example.com/sample.jpg",
                        "Sample photo to crop",
                        vec![],
                    )],
                ),
                image_cropper::selection(
                    &state,
                    &ImageCropperProps {
                        disabled: true,
                        dragging: true,
                    },
                    vec![],
                    vec![image_cropper::handle(
                        HandlePosition::Se,
                        &ImageCropperProps {
                            disabled: true,
                            dragging: true,
                        },
                        vec![],
                    )],
                ),
                image_cropper::grid(
                    None,
                    &ImageCropperProps {
                        disabled: true,
                        dragging: true,
                    },
                    vec![],
                ),
            ],
        ),
    ];
    demo_page("Image Cropper", body)
}

pub(super) fn listbox_section() -> Node {
    // single モード（イシュー #1611 参照突合前と同一の見た目）。
    let single_props = hui::listbox::ListboxProps::default();
    let selection_state = OpenState::Open;
    let single = listbox::root(
        selection_state,
        &single_props,
        vec![],
        vec![
            listbox::label(&single_props, Some("lb-label"), vec![], vec![text("Fruit")]),
            listbox::content(
                false,
                &single_props,
                Some("lb-content"),
                Some("lb-label"),
                Some("lb-item-0"),
                vec![],
                vec![listbox::item_group(
                    &single_props,
                    None,
                    vec![],
                    vec![
                        listbox::item_group_label(None, vec![], vec![text("Common")]),
                        listbox::item(
                            OpenState::Open,
                            &single_props,
                            false,
                            true,
                            "apple",
                            Some("lb-item-0"),
                            vec![],
                            vec![
                                listbox::item_text(
                                    OpenState::Open,
                                    &single_props,
                                    false,
                                    true,
                                    None,
                                    vec![],
                                    vec![text("Apple")],
                                ),
                                listbox::item_indicator(OpenState::Open, vec![], vec![text("✓")]),
                            ],
                        ),
                        listbox::item(
                            OpenState::Closed,
                            &single_props,
                            false,
                            false,
                            "banana",
                            None,
                            vec![],
                            vec![
                                listbox::item_text(
                                    OpenState::Closed,
                                    &single_props,
                                    false,
                                    false,
                                    None,
                                    vec![],
                                    vec![text("Banana")],
                                ),
                                listbox::item_indicator(OpenState::Closed, vec![], vec![text("✓")]),
                            ],
                        ),
                        // item-level disabled（イシュー #1611 参照突合: root disabled=false
                        // + item disabled=true が data-orientation/data-selected と並んで
                        // Anatomy/data-* 表へ機械的に露出することを示す）。
                        listbox::item(
                            OpenState::Closed,
                            &single_props,
                            true,
                            false,
                            "cherry",
                            None,
                            vec![],
                            vec![
                                listbox::item_text(
                                    OpenState::Closed,
                                    &single_props,
                                    true,
                                    false,
                                    None,
                                    vec![],
                                    vec![text("Cherry (disabled)")],
                                ),
                                listbox::item_indicator(OpenState::Closed, vec![], vec![text("✓")]),
                            ],
                        ),
                    ],
                )],
            ),
            listbox::value_text(false, &single_props, vec![], vec![text("Apple")]),
        ],
    );

    // multiple + horizontal + root disabled モード（イシュー #1611 参照突合:
    // root disabled が item へ伝播すること・data-orientation="horizontal" が
    // root/content/item-group/item へ出力されることを Demo 経由で
    // Anatomy/data-* 表へ機械的に露出させる）。
    let multi_props = hui::listbox::ListboxProps {
        disabled: true,
        orientation: hui::Orientation::Horizontal,
    };
    let multiple = listbox::root(
        OpenState::Open,
        &multi_props,
        vec![],
        vec![
            listbox::label(
                &multi_props,
                Some("lb-multi-label"),
                vec![],
                vec![text("Toppings")],
            ),
            listbox::content(
                true,
                &multi_props,
                Some("lb-multi-content"),
                Some("lb-multi-label"),
                None,
                vec![],
                vec![listbox::item_group(
                    &multi_props,
                    Some("lb-multi-group-label"),
                    vec![],
                    vec![
                        listbox::item_group_label(
                            Some("lb-multi-group-label"),
                            vec![],
                            vec![text("Cheese")],
                        ),
                        listbox::item(
                            OpenState::Open,
                            &multi_props,
                            false,
                            false,
                            "cheddar",
                            None,
                            vec![],
                            vec![
                                listbox::item_text(
                                    OpenState::Open,
                                    &multi_props,
                                    false,
                                    false,
                                    None,
                                    vec![],
                                    vec![text("Cheddar")],
                                ),
                                listbox::item_indicator(OpenState::Open, vec![], vec![text("✓")]),
                            ],
                        ),
                        listbox::item(
                            OpenState::Open,
                            &multi_props,
                            false,
                            false,
                            "mozzarella",
                            None,
                            vec![],
                            vec![
                                listbox::item_text(
                                    OpenState::Open,
                                    &multi_props,
                                    false,
                                    false,
                                    None,
                                    vec![],
                                    vec![text("Mozzarella")],
                                ),
                                listbox::item_indicator(OpenState::Open, vec![], vec![text("✓")]),
                            ],
                        ),
                    ],
                )],
            ),
            listbox::value_text(false, &multi_props, vec![], vec![text("2 selected")]),
        ],
    );

    demo_page("Listbox", vec![single, multiple])
}

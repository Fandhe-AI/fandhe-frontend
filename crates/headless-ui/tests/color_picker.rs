//! ColorPicker の動的状態機械（[`ColorPicker`]、イシュー #839、#1604 参照
//! 突合）の統合テスト。
//!
//! `crates/headless-ui/src/color_picker.rs` の `#[cfg(test)] mod tests` が
//! モジュール内部（`super::*`）から自由関数の attrs マージ・XSS 回帰を
//! 固定するのに対し、本ファイルは公開 API（[`ColorPicker`] の利便メソッド +
//! [`ColorPickerProps`]）のみを使って anatomy 全パート名・状態 `data-*` の
//! 一律付与・dispatch 契約・hydration 往復を固定する
//! （`tests/checkbox.rs`/`tests/toggle_tip.rs` と同じ位置付け）。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::color_picker::{Channel, ColorPicker, ColorPickerProps};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

fn all_states() -> ColorPickerProps {
    ColorPickerProps {
        disabled: true,
        readonly: true,
        invalid: true,
        required: true,
    }
}

/// anatomy 全パート名（`data-part`）が期待どおり出力されること
/// （`crate::color_picker` モジュール doc 冒頭の anatomy 一覧との対応固定）。
#[test]
fn convenience_methods_output_expected_anatomy_part_names() {
    let cp = ColorPicker::default();
    let props = ColorPickerProps::default();

    let parts: &[(&str, fandhe_frontend_core::Node)] = &[
        ("root", cp.root(&props, vec![], vec![])),
        ("label", cp.label(&props, vec![], vec![])),
        ("control", cp.control(&props, vec![], vec![])),
        ("trigger", cp.trigger(&props, None, vec![], vec![])),
        ("positioner", cp.positioner(vec![], vec![])),
        ("content", cp.content(None, vec![], vec![])),
        ("area", cp.area(&props, vec![], vec![])),
        (
            "area-background",
            cp.area_background(&props, vec![], vec![]),
        ),
        ("area-thumb", cp.area_thumb(&props, vec![], vec![])),
        (
            "hue-slider",
            cp.channel_slider(Channel::Hue, Orientation::Horizontal, vec![], vec![]),
        ),
        (
            "hue-slider-track",
            cp.channel_slider_track(Channel::Hue, Orientation::Horizontal, vec![], vec![]),
        ),
        (
            "hue-slider-thumb",
            cp.channel_slider_thumb(
                Channel::Hue,
                Orientation::Horizontal,
                &props,
                vec![],
                vec![],
            ),
        ),
        ("channel-input", cp.channel_input(&props, vec![])),
        ("value-text", cp.value_text(&props, vec![], vec![])),
        ("hidden-input", cp.hidden_input("color", &props, vec![])),
    ];

    for (expected_part, node) in parts {
        let html = render(node);
        assert!(
            html.contains(r#"data-scope="color-picker""#),
            "{expected_part}: {html}"
        );
        assert!(
            html.contains(&format!(r#"data-part="{expected_part}""#)),
            "{expected_part}: {html}"
        );
    }
}

/// `ColorPickerProps` の disabled/readonly/invalid が root/label/control/
/// trigger/area/area-background/area-thumb/channel-input へ一律反映される
/// こと（利便メソッド経由）。
#[test]
fn props_state_flows_through_convenience_methods() {
    let cp = ColorPicker::default();
    let props = all_states();

    for html in [
        render(&cp.root(&props, vec![], vec![])),
        render(&cp.label(&props, vec![], vec![])),
        render(&cp.control(&props, vec![], vec![])),
        render(&cp.trigger(&props, None, vec![], vec![])),
        render(&cp.area(&props, vec![], vec![])),
        render(&cp.area_background(&props, vec![], vec![])),
        render(&cp.area_thumb(&props, vec![], vec![])),
        render(&cp.channel_input(&props, vec![])),
    ] {
        assert!(html.contains(r#"data-disabled="""#), "{html}");
        assert!(html.contains(r#"data-invalid="""#), "{html}");
        assert!(html.contains(r#"data-readonly="""#), "{html}");
    }

    // label のみ data-required を持つ。
    assert!(render(&cp.label(&props, vec![], vec![])).contains(r#"data-required="""#));
    assert!(!render(&cp.root(&props, vec![], vec![])).contains("data-required"));
}

/// `channel_slider_thumb` に `data-channel`/`data-orientation`/
/// `aria-orientation` が反映されること（利便メソッド経由、4 チャンネル網羅）。
#[test]
fn channel_slider_thumb_convenience_method_outputs_channel_and_orientation() {
    let cp = ColorPicker::default();
    let props = ColorPickerProps::default();
    for channel in [
        Channel::Hue,
        Channel::Saturation,
        Channel::Value,
        Channel::Alpha,
    ] {
        let html = render(&cp.channel_slider_thumb(
            channel,
            Orientation::Vertical,
            &props,
            vec![],
            vec![],
        ));
        assert!(html.contains(&format!(r#"data-channel="{}""#, channel.as_str())));
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(html.contains(r#"aria-orientation="vertical""#));
    }
}

/// dispatch 契約 7 語彙（open/close/toggle/set_hex/set_channel/increment/
/// decrement）が公開 API 経由で機能すること。
#[test]
fn dispatch_supports_all_seven_actions() {
    let mut cp = ColorPicker::default();

    assert!(dispatch(&mut cp, "open", ""));
    assert!(cp.is_open());
    assert!(dispatch(&mut cp, "toggle", ""));
    assert!(!cp.is_open());
    assert!(dispatch(&mut cp, "close", ""));
    assert!(!cp.is_open());

    // 赤（primary 色）は HSV round trip でも量子化ドリフトが生じないことが
    // `crate::color` の既知値網羅テストで固定済みのため、ここでは
    // ドリフトの影響を受けない値を選ぶ（`color_picker.rs` 内 in-module
    // テスト `dispatch_set_hex_updates_color` と同じ判断）。
    assert!(dispatch(&mut cp, "set_hex", "#ff0000"));
    assert_eq!(cp.hex(), "#ff0000");

    assert!(dispatch(&mut cp, "set_channel", "alpha:100"));
    assert_eq!(cp.alpha_value(), 100);

    assert!(dispatch(&mut cp, "increment", "alpha"));
    assert_eq!(cp.alpha_value(), 101);
    assert!(dispatch(&mut cp, "decrement", "alpha"));
    assert_eq!(cp.alpha_value(), 100);
}

/// hydration 往復（`data-hydrate-h/s/v/a` + `Disclosure` の
/// `data-hydrate-state`）が公開 API のみで固定されること。
#[test]
fn hydration_round_trip_via_public_api() {
    let mut cp = ColorPicker::from_color(fandhe_frontend_headless_ui::color::Color::from_rgba(
        fandhe_frontend_headless_ui::color::Rgb::new(0x3b, 0x82, 0xf6),
        0x80,
    ));
    assert!(dispatch(&mut cp, "open", ""));

    let rendered = render(&render_for_hydration(&cp));
    assert!(rendered.contains("data-hydrate-h="));
    assert!(rendered.contains(r#"data-hydrate-state="open""#));

    let restored = ColorPicker::from_hydration_attrs(&cp.hydration_attrs()).unwrap();
    assert_eq!(restored, cp);
}

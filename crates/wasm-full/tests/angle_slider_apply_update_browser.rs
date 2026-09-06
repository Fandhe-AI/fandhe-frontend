//! `Runtime::wire_angle_slider` の dispatch 後再描画接続（イシュー #1956）
//! の実ブラウザ回帰テスト（`wasm-pack test --headless --chrome`）。
//!
//! `crates/wasm-full/src/angle_slider.rs` の native `#[cfg(test)] mod
//! tests` は `angle_from_offset`/`action_for_key` 等の純粋ロジック層のみを
//! 検証する。本ファイルはその先、`crates/wasm-full/tests/
//! headless_timer_browser.rs` の `runtime_dirty_rerender` モジュールと
//! 同型のパターンで、`Runtime::hydrate` 経由で配線した AngleSlider Thumb
//! への合成 keydown（`ArrowRight`）が、
//!
//! 1. AngleSlider 自身の `aria-valuenow`/`aria-valuetext`（[`view()`]
//!    の再描画対象）
//! 2. アプリ側（`C`）の派生フィールド（`data-bind-text` 束縛点）
//!
//! の両方へ実際に反映されることを検証する（修正前は
//! `angle_slider::wire_angle_slider_events` へ渡す `on_action` が
//! `dispatch` のみで [`fandhe_frontend_wasm_full::Runtime::apply_update_for_dirty`]
//! を一切呼ばず、pointer/keydown 操作で状態は更新されても DOM 再描画が
//! 一切走らないバグだった）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::{bind_text, render, Node};
use fandhe_frontend_headless_ui::angle_slider::{AngleSlider, AngleSliderProps};
use fandhe_frontend_interactive::{Component, DirtyTracked, Hydrate, HydrateError};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, KeyboardEvent, KeyboardEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// `Runtime::hydrate` の `root_id` として使う固定 id
/// （本ファイル内で 1 テストのみが本ホストを使うため、
/// `headless_timer_browser.rs::runtime_dirty_rerender::ROOT_ID` と同様に
/// 単一の固定文字列で足りる）。
const ROOT_ID: &str = "angle-slider-host-runtime-dirty-rerender-root";

/// テスト終了時に要素を DOM から除去する RAII ガード
/// （`headless_timer_browser.rs::RemoveOnDrop` と同型）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// 合成 `keydown`（`bubbles: true, cancelable: true`）を組み立てる
/// （`number_input_browser.rs::keydown_event` と同型）。
fn keydown_event(key: &str) -> Event {
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail")
        .dyn_into::<Event>()
        .expect("KeyboardEvent must cast to Event")
}

/// `AngleSlider` をラップし、アプリ側の派生フィールド（角度ラベル）のみを
/// dirty へ積む最小ホスト（`headless_timer_browser.rs::runtime_dirty_rerender::TimerHost`
/// と同じ設計）。
struct AngleSliderHost {
    slider: AngleSlider,
    angle_label: String,
    dirty: Vec<&'static str>,
}

impl AngleSliderHost {
    fn new(slider: AngleSlider) -> Self {
        let angle_label = format!("{}deg", slider.angle_deg());
        Self {
            slider,
            angle_label,
            dirty: Vec::new(),
        }
    }
}

impl Component for AngleSliderHost {
    type Action = <AngleSlider as Component>::Action;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        self.slider.update(action);
        let new_label = format!("{}deg", self.slider.angle_deg());
        if new_label != self.angle_label {
            self.angle_label = new_label;
            self.dirty.push("angle_label");
        }
    }

    fn view(&self) -> Node {
        let props = AngleSliderProps::default();
        self.slider.root(
            &props,
            vec![("id", ROOT_ID)],
            vec![
                self.slider.control(
                    &props,
                    Vec::new(),
                    vec![self.slider.thumb(&props, Vec::new(), Vec::new())],
                ),
                bind_text(
                    "span",
                    vec![("data-testid", "angle-label")],
                    "angle_label",
                    self.angle_label.clone(),
                ),
            ],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
        AngleSlider::decode_action(name, payload)
    }
}

impl DirtyTracked for AngleSliderHost {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

impl BindingSource for AngleSliderHost {
    fn bound_value(&self, field: &str) -> Option<BoundValue> {
        match field {
            "angle_label" => Some(BoundValue::Text(self.angle_label.clone())),
            _ => None,
        }
    }
}

impl Hydrate for AngleSliderHost {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.slider.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        AngleSlider::from_hydration_attrs(attrs).map(Self::new)
    }
}

fn document() -> Document {
    web_sys::window()
        .expect("window must exist")
        .document()
        .expect("document must exist")
}

/// `Runtime::hydrate` 経由で配線した Thumb keydown（`ArrowRight`）が、
/// AngleSlider 自身の `aria-valuenow`/`aria-valuetext`（束縛点解決される
/// [`Self::wire`] の再描画経路）と `C`（`AngleSliderHost`）側の
/// `data-bind-text="angle_label"` の双方へ反映されることを検証する
/// （受け入れ条件、イシュー #1956）。修正前は
/// `angle_slider::wire_angle_slider_events` の `on_action` が dispatch の
/// みで `apply_update_for_dirty` を呼ばず、いずれの属性も更新されなかった。
#[wasm_bindgen_test]
fn thumb_arrow_right_keydown_dispatches_and_rerenders_dom() {
    let document = document();

    let host = AngleSliderHost::new(AngleSlider::new(10, 5));
    let html = render(&host.view());
    document
        .body()
        .expect("document body must exist in browser test environment")
        .insert_adjacent_html("beforeend", &html)
        .expect("insert_adjacent_html must not fail");
    let root_el = document
        .get_element_by_id(ROOT_ID)
        .expect("rendered AngleSlider root must have the expected id");
    let _cleanup = RemoveOnDrop(root_el.clone());

    // `render_for_hydration` が行う「view() の root へ hydration_attrs を
    // 後付けする」処理を、実 DOM 属性として直接再現する
    // （`runtime_browser.rs::hydrate_restores_state_from_existing_dom_and_wires_events`
    // と同じ手順）。
    for (name, value) in host.hydration_attrs() {
        root_el
            .set_attribute(&name, &value)
            .expect("set_attribute must not fail");
    }

    let runtime = Runtime::hydrate(ROOT_ID, AngleSliderHost::new(AngleSlider::new(10, 5)))
        .expect("hydrate must succeed for well-formed attrs");
    assert_eq!(
        runtime.root().id(),
        ROOT_ID,
        "hydrate は root_id 要素自身を AngleSlider root として復元すること"
    );

    let thumb = root_el
        .query_selector("[data-scope='angle-slider'][data-part='thumb']")
        .expect("query_selector must not fail")
        .expect("thumb part must exist");
    let angle_label = || {
        root_el
            .query_selector("[data-bind-text='angle_label']")
            .expect("query_selector must not fail")
            .expect("angle_label binding point must exist")
            .text_content()
            .unwrap_or_default()
    };

    assert_eq!(thumb.get_attribute("aria-valuenow").as_deref(), Some("10"));
    assert_eq!(angle_label(), "10deg");

    let default_not_prevented = thumb.dispatch_event(&keydown_event("ArrowRight")).unwrap();
    assert!(
        !default_not_prevented,
        "ArrowRight は claim され prevent_default() が呼ばれること"
    );

    assert_eq!(
        runtime.component().slider.angle_deg(),
        15,
        "keydown dispatch が AngleSliderAction::Increment（step=5）で \
         状態を更新すること"
    );
    assert_eq!(
        thumb.get_attribute("aria-valuenow").as_deref(),
        Some("15"),
        "Runtime::wire_angle_slider が dispatch 後の apply_update_for_dirty \
         を呼び、AngleSlider 自身の aria-valuenow が再描画されること \
         （イシュー #1956 の受け入れ条件）"
    );
    assert_eq!(
        thumb.get_attribute("aria-valuetext").as_deref(),
        Some("15deg"),
    );
    assert_eq!(
        angle_label(),
        "15deg",
        "Runtime::wire_angle_slider が dispatch 後の dirty_fields() を \
         apply_update_for_dirty へ渡し、C 側の束縛点（angle_label）が \
         再描画されること（イシュー #1956 の受け入れ条件）"
    );
}

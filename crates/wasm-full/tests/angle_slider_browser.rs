//! `Runtime::wire_angle_slider` の dispatch 後再描画接続（イシュー #1956/
//! #1957）の実ブラウザ回帰テスト（`wasm-pack test --headless --chrome`）。
//!
//! `crates/wasm-full/src/angle_slider.rs` の native `#[cfg(test)] mod
//! tests` は `angle_from_offset`/`action_for_key` 等の純粋ロジック層のみを
//! 検証する。本ファイルはその先、`crates/wasm-full/tests/
//! headless_timer_browser.rs` の `runtime_dirty_rerender` モジュールと
//! 同型のパターンで、`Runtime::hydrate` 経由で配線した AngleSlider への
//! 実 DOM 操作が、
//!
//! 1. pointerdown（[`pointerdown_dispatches_and_rerenders_dom`]）
//! 2. keydown（[`thumb_arrow_right_keydown_dispatches_and_rerenders_dom`]）
//! 3. disabled 時の no-op（[`disabled_pointerdown_and_keydown_are_no_op`]）
//!
//! の 3 ケースで、AngleSlider 自身の `aria-valuenow`/`aria-valuetext`
//! （[`AngleSliderHost::view`] が `data-bind-attr` で束縛する束縛点）と
//! アプリ側（`C`）の派生フィールド（`data-bind-text` 束縛点）の双方へ
//! 正しく反映される（またはケース 3 では一切変化しない）ことを検証する。
//!
//! # 束縛点設計（イシュー #1956 の根本原因と是正）
//!
//! 修正前は `angle_slider::wire_angle_slider_events` へ渡す `on_action` が
//! `dispatch` のみで [`fandhe_frontend_wasm_full::Runtime::apply_update_for_dirty`]
//! を一切呼ばず、pointer/keydown 操作で状態は更新されても DOM 再描画が
//! 一切走らないバグだった（`Runtime::wire_angle_slider` へ修正済み）。
//!
//! 加えて、修正直後の初版テスト（イシュー #1956 実装 PR 初稿）は
//! `Runtime::apply_update_for_dirty` が呼ばれるようにはなったものの、
//! [`fandhe_frontend_headless_ui::angle_slider::thumb`] が出力する
//! `aria-valuenow`/`aria-valuetext` は**静的属性**（束縛点マーカーを持たない）
//! であるにもかかわらず、テストホストが `dirty_fields()` へ
//! アプリ側派生フィールド（`angle_label`）しか積んでいなかったため、
//! 束縛点対応表にも keyed list にも該当しない・かつ「未解決 dirty field」
//! としても検知されない（`angle_label` 自体は束縛点を持つため
//! `unresolved_field` が立たない）という盲点があり、`aria-valuenow` が
//! 実際には更新されないまま green を主張していた（レビュー指摘）。
//!
//! 是正: `fandhe_frontend_core::bind_attr_tokens` で `thumb()` の呼び出し側
//! `attrs` へ `data-bind-attr="aria-valuenow:angle_now aria-valuetext:angle_text"`
//! を付与し、`AngleSliderHost` が [`fandhe_frontend_wasm_client::BindingSource`]
//! で `angle_now`/`angle_text` を解決し、`update()` で角度が変化した際に
//! 両フィールドを `dirty_fields()` へ積む。これは `Runtime::wire_angle_slider`
//! rustdoc「AngleSlider の `aria-valuenow` 等は束縛点で解決される想定」が
//! 実際に成立するための、アプリ側が担うべき配線（`Self::wire` の既存
//! 束縛点更新経路のみを使い、`fandhe-frontend-wasm-full`/
//! `fandhe-frontend-headless-ui` 側の変更は不要）である。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::{bind_attr_tokens, bind_text, render, Node};
use fandhe_frontend_headless_ui::angle_slider::{AngleSlider, AngleSliderProps};
use fandhe_frontend_interactive::{Component, DirtyTracked, Hydrate, HydrateError};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{
    Document, Element, Event, KeyboardEvent, KeyboardEventInit, PointerEvent, PointerEventInit,
};

wasm_bindgen_test_configure!(run_in_browser);

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

/// 合成 `pointerdown`（`bubbles: true, cancelable: true`）を、指定した
/// クライアント座標・`pointerId` で組み立てる
/// （`headless_signature_pad_browser.rs::new_pointer_event` を拡張し
/// `client_x`/`client_y` を追加。`fandhe_frontend_wasm_full::angle_slider::
/// handle_pointerdown` は `PointerEvent::client_x()`/`client_y()` から
/// `Control` の `getBoundingClientRect()` 中心相対の角度を計算するため
/// 座標指定が必須）。
fn pointerdown_event(pointer_id: i32, client_x: f64, client_y: f64) -> Event {
    let init = PointerEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_pointer_id(pointer_id);
    init.set_client_x(client_x.round() as i32);
    init.set_client_y(client_y.round() as i32);
    PointerEvent::new_with_event_init_dict("pointerdown", &init)
        .expect("PointerEvent::new must not fail")
        .dyn_into::<Event>()
        .expect("PointerEvent must cast to Event")
}

/// `AngleSlider` をラップし、
///
/// - AngleSlider 自身の `aria-valuenow`/`aria-valuetext`（`angle_now`/
///   `angle_text` フィールドへ `data-bind-attr` で束縛、モジュール冒頭
///   doc「束縛点設計」節参照）
/// - アプリ側の派生フィールド（`angle_label`、`data-bind-text` で束縛）
///
/// の両方を `dirty_fields()` へ積む最小ホスト
/// （`headless_timer_browser.rs::runtime_dirty_rerender::TimerHost` と
/// 同じ設計）。`root_id`/`thumb_bind_attr` はテストごとに一意な `id` と
/// `data-bind-attr` 文字列を保持するための固定フィールド（`view()` が
/// `&self` のみを取るため、呼び出しごとに動的構築せず構築済みの値を
/// 使い回す）。
struct AngleSliderHost {
    slider: AngleSlider,
    props: AngleSliderProps,
    root_id: &'static str,
    thumb_bind_attr: String,
    angle_now: String,
    angle_text: String,
    angle_label: String,
    dirty: Vec<&'static str>,
}

impl AngleSliderHost {
    fn new(root_id: &'static str, slider: AngleSlider) -> Self {
        Self::with_props(root_id, slider, AngleSliderProps::default())
    }

    fn disabled(root_id: &'static str, slider: AngleSlider) -> Self {
        Self::with_props(
            root_id,
            slider,
            AngleSliderProps {
                disabled: true,
                readonly: false,
                invalid: false,
            },
        )
    }

    fn with_props(root_id: &'static str, slider: AngleSlider, props: AngleSliderProps) -> Self {
        let angle_now = slider.angle_deg().to_string();
        let angle_text = format!("{}deg", slider.angle_deg());
        let angle_label = angle_text.clone();
        let thumb_bind_attr = bind_attr_tokens(&[
            ("aria-valuenow", "angle_now"),
            ("aria-valuetext", "angle_text"),
        ]);
        Self {
            slider,
            props,
            root_id,
            thumb_bind_attr,
            angle_now,
            angle_text,
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
        let now = self.slider.angle_deg().to_string();
        let text = format!("{}deg", self.slider.angle_deg());
        if now != self.angle_now {
            self.angle_now = now;
            self.dirty.push("angle_now");
        }
        if text != self.angle_text {
            self.angle_text = text.clone();
            self.dirty.push("angle_text");
        }
        if text != self.angle_label {
            self.angle_label = text;
            self.dirty.push("angle_label");
        }
    }

    fn view(&self) -> Node {
        self.slider.root(
            &self.props,
            vec![("id", self.root_id)],
            vec![
                self.slider.control(
                    &self.props,
                    Vec::new(),
                    vec![self.slider.thumb(
                        &self.props,
                        vec![("data-bind-attr", self.thumb_bind_attr.as_str())],
                        Vec::new(),
                    )],
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
            "angle_now" => Some(BoundValue::Text(self.angle_now.clone())),
            "angle_text" => Some(BoundValue::Text(self.angle_text.clone())),
            "angle_label" => Some(BoundValue::Text(self.angle_label.clone())),
            _ => None,
        }
    }
}

impl Hydrate for AngleSliderHost {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.slider.hydration_attrs()
    }

    fn from_hydration_attrs(_attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        // `Runtime::hydrate` は復元失敗時のみこの経路を使う。本ファイルの
        // テストはいずれも `mount` 経由で well-formed な `data-hydrate-*`
        // 属性を明示付与するため実際には呼ばれないが、`Hydrate` 実装として
        // `root_id`/`props` を持たない構築不能な形（`unimplemented!` 等）は
        // 避け、`AngleSliderProps::default` へ fail-closed でフォールバック
        // する（`root_id` はプレースホルダとし、通常経路では到達しない
        // ことを doc で明示する）。
        Err(HydrateError::MissingAttr("angle-slider-host".to_string()))
    }
}

fn document() -> Document {
    web_sys::window()
        .expect("window must exist")
        .document()
        .expect("document must exist")
}

/// `host` を document body へ挿入し、`host.hydration_attrs()` を DOM
/// 属性として付与したうえで `Runtime::hydrate` を呼ぶ（`render_for_hydration`
/// が行う「view() の root へ hydration_attrs を後付けする」処理を実 DOM
/// 属性として直接再現する、`runtime_browser.rs::
/// hydrate_restores_state_from_existing_dom_and_wires_events` と同じ手順）。
fn mount(host: AngleSliderHost) -> (Element, Runtime<AngleSliderHost>) {
    let document = document();
    let root_id = host.root_id;
    let html = render(&host.view());
    document
        .body()
        .expect("document body must exist in browser test environment")
        .insert_adjacent_html("beforeend", &html)
        .expect("insert_adjacent_html must not fail");
    let root_el = document
        .get_element_by_id(root_id)
        .expect("rendered AngleSlider root must have the expected id");

    for (name, value) in host.hydration_attrs() {
        root_el
            .set_attribute(&name, &value)
            .expect("set_attribute must not fail");
    }

    let runtime =
        Runtime::hydrate(root_id, host).expect("hydrate must succeed for well-formed attrs");
    assert_eq!(
        runtime.root().id(),
        root_id,
        "hydrate は root_id 要素自身を AngleSlider root として復元すること"
    );
    (root_el, runtime)
}

/// `Runtime::hydrate` 経由で配線した Thumb keydown（`ArrowRight`）が、
/// AngleSlider 自身の `aria-valuenow`/`aria-valuetext`（`data-bind-attr`
/// 束縛点、モジュール冒頭 doc「束縛点設計」節参照）と `C`
/// （`AngleSliderHost`）側の `data-bind-text="angle_label"` の双方へ
/// 反映されることを検証する（受け入れ条件、イシュー #1956/#1957 ケース 2）。
#[wasm_bindgen_test]
fn thumb_arrow_right_keydown_dispatches_and_rerenders_dom() {
    let (root_el, runtime) = mount(AngleSliderHost::new(
        "angle-slider-host-keydown-root",
        AngleSlider::new(10, 5),
    ));
    let _cleanup = RemoveOnDrop(root_el.clone());

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
         （イシュー #1956/#1957 の受け入れ条件）"
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
         再描画されること（イシュー #1956/#1957 の受け入れ条件）"
    );
}

/// `Runtime::hydrate` 経由で配線した Control への pointerdown が、
/// [`fandhe_frontend_wasm_full::angle_slider`] の `handle_pointerdown`
/// （`getBoundingClientRect` 中心相対の角度計算）を経て `"set"` を
/// dispatch し、keydown と同じ束縛点反映経路（`aria-valuenow`/
/// `aria-valuetext`）が動くことを検証する（受け入れ条件、イシュー
/// #1956/#1957 ケース 1）。`control.set_pointer_capture` は合成イベント
/// 環境では `NotFoundError` になり得るが `handle_pointerdown` はその失敗を
/// `let _ =` で無視して座標反映を続行するため（`angle_slider.rs`
/// 実装参照）、本テストは pointer capture の成否に依存しない。
#[wasm_bindgen_test]
fn pointerdown_dispatches_and_rerenders_dom() {
    let (root_el, runtime) = mount(AngleSliderHost::new(
        "angle-slider-host-pointerdown-root",
        AngleSlider::new(10, 5),
    ));
    let _cleanup = RemoveOnDrop(root_el.clone());

    let control = root_el
        .query_selector("[data-scope='angle-slider'][data-part='control']")
        .expect("query_selector must not fail")
        .expect("control part must exist");
    let thumb = root_el
        .query_selector("[data-scope='angle-slider'][data-part='thumb']")
        .expect("query_selector must not fail")
        .expect("thumb part must exist");

    assert_eq!(thumb.get_attribute("aria-valuenow").as_deref(), Some("10"));

    // Control 中心の真右（dx > 0, dy = 0）は `angle_from_offset` の規約
    // （0 度=真上、時計回り増加）で 90 度になる（`angle_slider.rs` native
    // `mod tests` の網羅表と同じ規約）。実レイアウトに依存しないよう、
    // 実測 `getBoundingClientRect` からの相対座標で組み立てる。
    let rect = control.get_bounding_client_rect();
    let client_x = rect.left() + rect.width() / 2.0 + 50.0;
    let client_y = rect.top() + rect.height() / 2.0;
    control
        .dispatch_event(&pointerdown_event(1, client_x, client_y))
        .unwrap();

    assert_eq!(
        runtime.component().slider.angle_deg(),
        90,
        "Control 中心の真右への pointerdown が \"set\" action=90 を \
         dispatch すること"
    );
    assert_eq!(
        thumb.get_attribute("aria-valuenow").as_deref(),
        Some("90"),
        "Runtime::wire_angle_slider が pointerdown dispatch 後の \
         apply_update_for_dirty を呼び、AngleSlider 自身の aria-valuenow \
         が再描画されること（イシュー #1956/#1957 の受け入れ条件）"
    );
    assert_eq!(
        thumb.get_attribute("aria-valuetext").as_deref(),
        Some("90deg"),
    );
}

/// `data-disabled` を持つ AngleSlider（[`AngleSliderProps::disabled`]）は
/// pointerdown/keydown いずれも
/// [`fandhe_frontend_wasm_full::angle_slider`] の
/// `has_noninteractive_ancestor` で no-op になり、dispatch も DOM 再描画も
/// 一切発生しないことを検証する（受け入れ条件、イシュー #1956/#1957
/// ケース 3）。
#[wasm_bindgen_test]
fn disabled_pointerdown_and_keydown_are_no_op() {
    let (root_el, runtime) = mount(AngleSliderHost::disabled(
        "angle-slider-host-disabled-root",
        AngleSlider::new(10, 5),
    ));
    let _cleanup = RemoveOnDrop(root_el.clone());

    let control = root_el
        .query_selector("[data-scope='angle-slider'][data-part='control']")
        .expect("query_selector must not fail")
        .expect("control part must exist");
    let thumb = root_el
        .query_selector("[data-scope='angle-slider'][data-part='thumb']")
        .expect("query_selector must not fail")
        .expect("thumb part must exist");

    assert_eq!(thumb.get_attribute("aria-valuenow").as_deref(), Some("10"));

    let rect = control.get_bounding_client_rect();
    let client_x = rect.left() + rect.width() / 2.0 + 50.0;
    let client_y = rect.top() + rect.height() / 2.0;
    control
        .dispatch_event(&pointerdown_event(1, client_x, client_y))
        .unwrap();

    let keydown_not_prevented = thumb.dispatch_event(&keydown_event("ArrowRight")).unwrap();
    assert!(
        keydown_not_prevented,
        "disabled 時は keydown が claim されず prevent_default() が \
         呼ばれないこと"
    );

    assert_eq!(
        runtime.component().slider.angle_deg(),
        10,
        "disabled 時は pointerdown/keydown いずれも dispatch されず状態が \
         変化しないこと"
    );
    assert_eq!(
        thumb.get_attribute("aria-valuenow").as_deref(),
        Some("10"),
        "disabled 時は DOM 再描画も発生しないこと（イシュー #1956/#1957 \
         の受け入れ条件）"
    );
}

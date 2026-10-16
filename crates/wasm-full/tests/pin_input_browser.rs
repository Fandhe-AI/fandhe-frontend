//! `fandhe_frontend_wasm_full::Runtime::mount`/`hydrate` が PinInput の `input` / `focusin` /
//! `keydown` / `paste` / `compositionend` を配線することを確認する実ブラウザ統合テスト
//! （`wasm-pack test --headless --chrome`、イシュー #3834・#3835、PR #3855 レビュー指摘 1〜11 の回帰固定。
//! 本ファイル後半（「#3836」節）が貼り付け〔区切り除去・大文字化・拒否〕・矢印キー・Backspace/Delete・
//! 修飾キー/変換中の keydown・IME 確定・RFC 8628 想定の一連操作を実ブラウザで網羅検証する）。
//!
//! 検証する不変条件: 入力の受理・拒否と `focused` の位置は状態機械（`PinInput`）だけが決め、
//! DOM 層は dispatch 後の状態（`BindingSource` の予約 field 経由）を読み戻して値とフォーカスを
//! 反映するだけ。拒否された文字・正規化・同じ文字の打ち直し・要素の差し替え・`data-action-input`
//! 経路・非連続桁は、いずれも「状態機械の結果が DOM に現れる」ことで検証する。
//!
//! ホスト（`TestPinHost`）は `Runtime` が要求する trait（`Component`/`DirtyTracked`/
//! `BindingSource`/`Hydrate`）を `PinInput` へ委譲する最小実装。シナリオごとのフラグで
//! 読み戻しの有無・再描画・アプリ所有の `data-action-input`・Root 自身への配線を切り替える。
//! 実イベントの代わりに合成イベント（`input`/`focusin`、`bubbles: true`）を発火する。

#![cfg(all(target_arch = "wasm32", feature = "pin-input"))]

use fandhe_frontend_core::{el, Node};
use fandhe_frontend_headless_ui::pin_input::{
    PinInput, PinInputAction, PinInputKind, PinInputProps,
};
use fandhe_frontend_interactive::{Component, DirtyTracked, Hydrate, HydrateError};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::pin_input::{bound_value_for, parse_field};
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{
    ClipboardEvent, DataTransfer, Document, Element, Event, EventInit, HtmlInputElement,
    KeyboardEvent, KeyboardEventInit,
};

wasm_bindgen_test_configure!(run_in_browser);

/// ホストのアクション。`TypeAt` はアプリ所有の `data-action-input="digit-N"` 用
/// （桁を action 名で識別し `Focus(i)` → `Input(c)` を適用する、モジュール doc の契約）。
enum HostAction {
    Pin(PinInputAction),
    TypeAt(usize, char),
}

/// シナリオ切り替えフラグ付きのテストホスト。
struct TestPinHost {
    pin: PinInput,
    /// 各桁へ `data-action-input="digit-N"` を付ける（アプリ所有の経路）。
    custom_action: bool,
    /// Control を `data-action-input` を持つラッパー `div` で包む（祖先の所有者）。
    wrapper_action: bool,
    /// dispatch 後に `dirty_fields` を返し構造フォールバックで要素を差し替えさせる。
    rerender: bool,
    /// 予約 field を答える（偽なら `None` を返す = 読み戻せない）。
    readback: bool,
    /// 配線 root 自身が PinInput Root（view は Control だけを返す）。
    self_root: bool,
    /// アプリ所有の `TypeAt` を受けた回数（二重 dispatch の検知用）。
    type_calls: usize,
}

impl TestPinHost {
    fn new(count: usize, kind: PinInputKind) -> Self {
        Self {
            pin: PinInput::new(count, kind),
            custom_action: false,
            wrapper_action: false,
            rerender: false,
            readback: true,
            self_root: false,
            type_calls: 0,
        }
    }
}

impl Component for TestPinHost {
    type Action = HostAction;

    fn update(&mut self, action: HostAction) {
        match action {
            HostAction::Pin(a) => self.pin.update(a),
            HostAction::TypeAt(i, c) => {
                self.type_calls += 1;
                self.pin.update(PinInputAction::Focus(i));
                self.pin.update(PinInputAction::Input(c));
            }
        }
    }

    fn view(&self) -> Node {
        let props = PinInputProps::default();
        let inputs: Vec<Node> = (0..self.pin.count())
            .map(|i| {
                let action = format!("digit-{i}");
                let attrs = if self.custom_action {
                    vec![("data-action-input", action.as_str())]
                } else {
                    Vec::new()
                };
                self.pin.input(i, false, false, &props, attrs)
            })
            .collect();
        let control = self.pin.control(Vec::new(), inputs);
        let control = if self.wrapper_action {
            el("div", vec![("data-action-input", "digit-0")], vec![control])
        } else {
            control
        };
        if self.self_root {
            control
        } else {
            self.pin.root(&props, Vec::new(), vec![control])
        }
    }

    fn decode_action(name: &str, payload: &str) -> Option<HostAction> {
        if let Some(index) = name.strip_prefix("digit-") {
            let index = index.parse::<usize>().ok()?;
            let c = payload.chars().next()?;
            return Some(HostAction::TypeAt(index, c));
        }
        PinInput::decode_action(name, payload).map(HostAction::Pin)
    }
}

impl DirtyTracked for TestPinHost {
    fn dirty_fields(&self) -> &[&'static str] {
        if self.rerender {
            &["pin"]
        } else {
            &[]
        }
    }
}

impl BindingSource for TestPinHost {
    fn bound_value(&self, field: &str) -> Option<BoundValue> {
        if !self.readback {
            return None;
        }
        let (ordinal, field) = parse_field(field)?;
        (ordinal == 0).then(|| bound_value_for(&self.pin, field))
    }
}

impl Hydrate for TestPinHost {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.pin.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        PinInput::from_hydration_attrs(attrs).map(|pin| Self {
            pin,
            custom_action: false,
            wrapper_action: false,
            rerender: false,
            readback: true,
            self_root: false,
            type_calls: 0,
        })
    }
}

fn document() -> Document {
    web_sys::window()
        .expect("window must exist")
        .document()
        .expect("document must exist")
}

/// 一意な id のプレースホルダを body へ追加する（テスト間の要素奪い合い防止）。
fn create_placeholder(id: &str) -> Element {
    let container = document()
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(id);
    document()
        .body()
        .expect("document body must exist")
        .append_child(&container)
        .expect("append_child must not fail");
    container
}

/// テスト末尾でプレースホルダを確実に除去する RAII ガード。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

fn bubbling_event(kind: &str) -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict(kind, &init).expect("Event::new must not fail")
}

/// `data-index` が `index` の桁 input（再描画後の最新の要素を引き直す）。
fn digit(root: &Element, index: usize) -> HtmlInputElement {
    root.query_selector(&format!(
        r#"[data-scope="pin-input"][data-part="input"][data-index="{index}"]"#
    ))
    .expect("query_selector must not fail")
    .expect("digit input must exist")
    .dyn_into::<HtmlInputElement>()
    .expect("digit must be an input")
}

/// 現在フォーカスされている桁の `data-index`（桁 input 以外・無しは `None`）。
fn active_digit(root: &Element) -> Option<String> {
    let active = document().active_element()?;
    root.contains(Some(&active))
        .then(|| active.get_attribute("data-index"))
        .flatten()
}

/// 桁へフォーカスし、`value` を入力して合成 `input` イベントを発火する（実ブラウザの 1 文字入力の模倣）。
fn type_into(root: &Element, index: usize, value: &str) {
    digit(root, index).focus().expect("focus must not fail");
    // focusin の dispatch が再描画する構成では要素が差し替わるため、フォーカス後に引き直す
    // （実ユーザーの打鍵はフォーカスされた最新の要素へ入る）。
    let input = digit(root, index);
    input.set_value(value);
    input
        .dispatch_event(&bubbling_event("input"))
        .expect("dispatch_event must not fail");
}

fn mount(id: &str, host: TestPinHost) -> (Runtime<TestPinHost>, Element, RemoveOnDrop) {
    let placeholder = create_placeholder(id);
    let guard = RemoveOnDrop(placeholder.clone());
    if host.self_root {
        placeholder
            .set_attribute("data-scope", "pin-input")
            .unwrap();
        placeholder.set_attribute("data-part", "root").unwrap();
    }
    let runtime = Runtime::mount(id, host).expect("Runtime::mount must not fail");
    (runtime, placeholder, guard)
}

/// 指摘 1・11: 受理された入力の後、フォーカスは状態機械の focused（次桁）へ移り、値は状態の値のまま。
#[wasm_bindgen_test]
fn accepted_input_advances_focus_to_state_focused() {
    let (runtime, root, _g) = mount("pin-it-advance", TestPinHost::new(4, PinInputKind::Numeric));
    type_into(&root, 0, "5");
    assert_eq!(runtime.component().pin.digit(0), "5");
    assert_eq!(runtime.component().pin.focused_index(), Some(1));
    assert_eq!(active_digit(&root).as_deref(), Some("1"));
    assert_eq!(digit(&root, 0).value(), "5");
}

/// ネイティブ削除（Empty、`inputType` 不明のため `delete`）した桁は状態でも空になり、別の桁の入力で
/// 復活しない（値の書き込みは入力のあった桁だけ。#3835 で削除が状態へ届くようになった）。
#[wasm_bindgen_test]
fn deleted_digit_is_not_resurrected_by_another_digits_input() {
    let (runtime, root, _g) = mount(
        "pin-it-noresurrect",
        TestPinHost::new(4, PinInputKind::Numeric),
    );
    type_into(&root, 0, "1");
    type_into(&root, 1, "2");
    type_into(&root, 2, "3");
    type_into(&root, 2, "");
    type_into(&root, 0, "5");
    assert_eq!(runtime.component().pin.digit(2), "");
    assert_eq!(digit(&root, 0).value(), "5");
    assert_eq!(digit(&root, 2).value(), "");
}

/// 指摘 2・6・7: kind を DOM は知らない。Numeric へ 'a' を入れても状態機械が拒否し、
/// DOM の値は状態（空）へ戻りフォーカスは動かない。再 dispatch もされない。
#[wasm_bindgen_test]
fn rejected_input_is_restored_from_state_and_focus_stays() {
    let (runtime, root, _g) = mount("pin-it-reject", TestPinHost::new(4, PinInputKind::Numeric));
    type_into(&root, 0, "a");
    assert_eq!(runtime.component().pin.value(), "");
    assert_eq!(digit(&root, 0).value(), "");
    assert_eq!(active_digit(&root).as_deref(), Some("0"));

    // 旧値 '3' の桁へ拒否される文字を入れると、状態の '3' へ戻る（ライブ値・属性は証拠にならない）。
    type_into(&root, 0, "3");
    assert_eq!(runtime.component().pin.value(), "3");
    type_into(&root, 0, "x");
    assert_eq!(runtime.component().pin.value(), "3");
    assert_eq!(digit(&root, 0).value(), "3");
    assert_eq!(active_digit(&root).as_deref(), Some("0"));
}

/// 指摘 8: 同じ文字の打ち直しは状態機械が受理し focused が次桁へ進む（ビューは変わらない）。
#[wasm_bindgen_test]
fn same_char_retype_advances_focus() {
    let mut host = TestPinHost::new(4, PinInputKind::Numeric);
    host.pin.update(PinInputAction::Input('1'));
    let (runtime, root, _g) = mount("pin-it-retype", host);
    type_into(&root, 0, "1");
    assert_eq!(runtime.component().pin.digit(0), "1");
    assert_eq!(runtime.component().pin.focused_index(), Some(1));
    assert_eq!(active_digit(&root).as_deref(), Some("1"));
}

/// 指摘 10: 大文字そろえの kind で既存桁 'A' へ 'a' を再入力。状態機械が正規化して受理し、
/// DOM は kind を知らずにフォーカスが次桁へ移り、値は状態の 'A' へそろう。
#[wasm_bindgen_test]
fn uppercase_retype_normalizes_in_state_and_advances_focus() {
    let mut host = TestPinHost::new(4, PinInputKind::AlphanumericUppercase);
    host.pin.update(PinInputAction::Input('A'));
    let (runtime, root, _g) = mount("pin-it-upper", host);
    type_into(&root, 0, "a");
    assert_eq!(runtime.component().pin.digit(0), "A");
    assert_eq!(digit(&root, 0).value(), "A");
    assert_eq!(active_digit(&root).as_deref(), Some("1"));
}

/// 指摘 11（汎用経路）: 非連続桁（桁 3）の入力はその桁へ書かれ、先頭の空き桁へは書かれない。
#[wasm_bindgen_test]
fn non_contiguous_digit_receives_the_input() {
    let (runtime, root, _g) = mount(
        "pin-it-noncontig",
        TestPinHost::new(5, PinInputKind::Numeric),
    );
    type_into(&root, 3, "9");
    assert_eq!(runtime.component().pin.digit(3), "9");
    assert_eq!(runtime.component().pin.digit(0), "");
    assert_eq!(active_digit(&root).as_deref(), Some("4"));
}

/// 読み戻せないホスト（予約 field 未実装）は fail-closed: dispatch はするが DOM のフォーカスと値は触らない。
#[wasm_bindgen_test]
fn unreadable_host_leaves_dom_focus_and_value_untouched() {
    let mut host = TestPinHost::new(4, PinInputKind::Numeric);
    host.readback = false;
    let (runtime, root, _g) = mount("pin-it-noreadback", host);
    type_into(&root, 0, "5");
    assert_eq!(runtime.component().pin.digit(0), "5");
    assert_eq!(active_digit(&root).as_deref(), Some("0"));
    assert_eq!(digit(&root, 0).value(), "5");
}

/// 2 文字以上は `paste`（autofill・`insertFromPaste`）として全桁へ分配され、0 文字（`inputType` 不明の
/// 合成 `Event`）は `delete` として状態の桁も空になる（#3835 で接続。#3834 では何も dispatch しなかった）。
#[wasm_bindgen_test]
fn empty_and_multi_char_values_reach_state_machine() {
    let (runtime, root, _g) = mount("pin-it-multi", TestPinHost::new(4, PinInputKind::Numeric));
    type_into(&root, 0, "12");
    assert_eq!(runtime.component().pin.value(), "12");
    assert_eq!(digit(&root, 0).value(), "1");
    assert_eq!(digit(&root, 1).value(), "2");
    type_into(&root, 1, "");
    assert_eq!(runtime.component().pin.digit(1), "");
    assert_eq!(digit(&root, 1).value(), "");
}

/// keydown の Backspace は `preventDefault` し、現在桁を消して前の桁へフォーカスを戻す（スモーク）。
#[wasm_bindgen_test]
fn backspace_keydown_clears_digit_moves_focus_and_prevents_default() {
    let (runtime, root, _g) = mount("pin-it-bs", TestPinHost::new(4, PinInputKind::Numeric));
    type_into(&root, 0, "1");
    type_into(&root, 1, "2");
    let target = digit(&root, 1);
    target.focus().expect("focus must not fail");
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key("Backspace");
    let event = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail");
    target
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");
    assert!(event.default_prevented());
    assert_eq!(runtime.component().pin.digit(1), "");
    assert_eq!(digit(&root, 1).value(), "");
    assert_eq!(active_digit(&root).as_deref(), Some("0"));
}

/// 指摘 3: 配線 root 自身が PinInput Root でも入力とフォーカス移動が成立する。
#[wasm_bindgen_test]
fn wiring_root_that_is_the_pin_input_root_works() {
    let mut host = TestPinHost::new(4, PinInputKind::Numeric);
    host.self_root = true;
    let (runtime, root, _g) = mount("pin-it-selfroot", host);
    type_into(&root, 0, "7");
    assert_eq!(runtime.component().pin.digit(0), "7");
    assert_eq!(active_digit(&root).as_deref(), Some("1"));
}

/// 指摘 4: `focusin` からの dispatch で再描画（構造フォールバック）され桁 input が差し替わっても、
/// 状態機械の focused の桁（新しい要素）へフォーカスが戻る。
#[wasm_bindgen_test]
fn focusin_rerender_keeps_focus_on_the_state_focused_digit() {
    let mut host = TestPinHost::new(4, PinInputKind::Numeric);
    host.rerender = true;
    let (runtime, root, _g) = mount("pin-it-rerender", host);
    let old = digit(&root, 2);
    old.focus().expect("focus must not fail");
    old.dispatch_event(&bubbling_event("focusin"))
        .expect("dispatch_event must not fail");
    assert_eq!(runtime.component().pin.focused_index(), Some(2));
    assert!(
        !old.is_connected(),
        "再描画で桁 input が差し替わっていること"
    );
    assert_eq!(active_digit(&root).as_deref(), Some("2"));
}

/// 指摘 4（`input` 経路）: 1 文字入力の dispatch で再描画され桁 input が差し替わっても、
/// フォーカスは状態機械の focused（次桁の新しい要素）へ置かれ、値は状態の値のまま。
#[wasm_bindgen_test]
fn input_rerender_keeps_focus_on_the_state_focused_digit() {
    let mut host = TestPinHost::new(4, PinInputKind::Numeric);
    host.rerender = true;
    let (runtime, root, _g) = mount("pin-it-input-rerender", host);
    let old = digit(&root, 0);
    type_into(&root, 0, "5");
    assert_eq!(runtime.component().pin.digit(0), "5");
    assert!(
        !old.is_connected(),
        "再描画で桁 input が差し替わっていること"
    );
    assert_eq!(digit(&root, 0).value(), "5");
    assert_eq!(active_digit(&root).as_deref(), Some("1"));
}

/// 指摘 5: 祖先（ラッパー `div`）の `data-action-input` も所有者として扱い、
/// 本モジュールは dispatch しない（1 回の入力で 1 桁だけ埋まる）。
#[wasm_bindgen_test]
fn ancestor_action_owner_is_dispatched_exactly_once() {
    let mut host = TestPinHost::new(4, PinInputKind::Numeric);
    host.wrapper_action = true;
    let (runtime, root, _g) = mount("pin-it-wrapper", host);
    type_into(&root, 0, "7");
    assert_eq!(runtime.component().pin.value(), "7");
    assert_eq!(runtime.component().pin.focused_index(), Some(1));
    assert_eq!(active_digit(&root).as_deref(), Some("1"));
}

/// 指摘 9・11（アプリ所有の経路）: `data-action-input` 桁では汎用 "focus" を送らない
/// （focusin で状態の focused が動かない）。アプリが action 名で桁を識別するため、
/// 非連続桁の入力もその桁へ書かれ、アプリの dispatch 後に読み戻しでフォーカスが進む。
#[wasm_bindgen_test]
fn app_owned_digits_get_no_generic_focus_and_still_reflect_state() {
    let mut host = TestPinHost::new(5, PinInputKind::Numeric);
    host.custom_action = true;
    let (runtime, root, _g) = mount("pin-it-custom", host);
    digit(&root, 3)
        .dispatch_event(&bubbling_event("focusin"))
        .expect("dispatch_event must not fail");
    assert_eq!(
        runtime.component().pin.focused_index(),
        None,
        "汎用 focus を dispatch しないこと"
    );
    type_into(&root, 3, "9");
    assert_eq!(runtime.component().pin.digit(3), "9");
    assert_eq!(runtime.component().pin.digit(0), "");
    assert_eq!(active_digit(&root).as_deref(), Some("4"));
    // アプリ所有の経路でも、拒否された文字は状態の値へ戻る（状態機械の判断を DOM が推測しない）。
    type_into(&root, 0, "x");
    assert_eq!(runtime.component().pin.digit(0), "");
    assert_eq!(digit(&root, 0).value(), "");
}

/// `Runtime::hydrate`（CSR フォールバック経路）でも同じ配線が組み込まれる
/// （`mount` と別コードパスの配線漏れ検知）。
#[wasm_bindgen_test]
fn runtime_hydrate_csr_fallback_wires_pin_input() {
    let placeholder = create_placeholder("pin-it-hydrate");
    let _g = RemoveOnDrop(placeholder.clone());
    let runtime = Runtime::hydrate("pin-it-hydrate", TestPinHost::new(4, PinInputKind::Numeric))
        .expect("Runtime::hydrate must not fail");
    type_into(&placeholder, 0, "5");
    assert_eq!(runtime.component().pin.digit(0), "5");
    assert_eq!(active_digit(&placeholder).as_deref(), Some("1"));
}
/// 入れ子の配線 root を手組みの DOM に 2 重で配線し、各配線の dispatch 回数を数える。
/// 戻り値は (外側 root, 内側の桁 input, 外側の回数, 内側の回数, 除去ガード)。
type Counter = std::rc::Rc<std::cell::Cell<usize>>;

fn nested_wired(id: &str) -> (Element, HtmlInputElement, Counter, Counter, RemoveOnDrop) {
    let outer = create_placeholder(id);
    let guard = RemoveOnDrop(outer.clone());
    outer.set_inner_html(
        r#"<div data-scope="pin-input" data-part="root" id="inner-root"><input data-scope="pin-input" data-part="input" data-index="0"><input data-scope="pin-input" data-part="input" data-index="1"></div>"#,
    );
    let inner = outer
        .query_selector("#inner-root")
        .unwrap()
        .expect("inner root must exist");
    let outer_count: Counter = std::rc::Rc::new(std::cell::Cell::new(0));
    let inner_count: Counter = std::rc::Rc::new(std::cell::Cell::new(0));
    let oc = outer_count.clone();
    let ic = inner_count.clone();
    fandhe_frontend_wasm_full::pin_input::wire_pin_input_events(
        inner,
        move |_| {
            ic.set(ic.get() + 1);
            true
        },
        |_| None,
    )
    .expect("inner wiring must not fail");
    fandhe_frontend_wasm_full::pin_input::wire_pin_input_events(
        outer.clone(),
        move |_| {
            oc.set(oc.get() + 1);
            true
        },
        |_| None,
    )
    .expect("outer wiring must not fail");
    let target = digit(&outer, 0);
    (outer, target, outer_count, inner_count, guard)
}

/// PR #3856 指摘: 入れ子の配線 root で keydown が最内の配線だけに処理される。
#[wasm_bindgen_test]
fn nested_roots_handle_keydown_only_in_the_innermost_wiring() {
    let (_outer, target, outer_count, inner_count, _g) = nested_wired("pin-it-nested-key");
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key("Backspace");
    let event = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail");
    target
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");
    assert!(inner_count.get() > 0);
    assert_eq!(outer_count.get(), 0);
}

/// 指摘: 変換の取り消し（`data` が空）で既存の数字が復元されても dispatch しない。
/// 確定（`data` あり）は dispatch し、入れ子でも最内だけが処理する。
#[wasm_bindgen_test]
fn compositionend_cancel_with_restored_digit_dispatches_nothing() {
    let (_outer, target, outer_count, inner_count, _g) = nested_wired("pin-it-comp-cancel");
    target.set_value("5");
    let cancel = bubbling_event("compositionend");
    js_sys::Reflect::set(cancel.as_ref(), &"data".into(), &"".into()).unwrap();
    target.dispatch_event(&cancel).unwrap();
    assert_eq!(inner_count.get(), 0);
    assert_eq!(outer_count.get(), 0);

    let confirm = bubbling_event("compositionend");
    js_sys::Reflect::set(confirm.as_ref(), &"data".into(), &"5".into()).unwrap();
    target.dispatch_event(&confirm).unwrap();
    assert!(inner_count.get() > 0);
    assert_eq!(outer_count.get(), 0);
}

/// PR #3856 指摘（P1）: 確定後の追加 `input`（Safari 系）の抑止判定は、入れ子の配線 root でも
/// 外側・内側の双方の capture で維持され、内側の bubble で二重 dispatch されない。
#[wasm_bindgen_test]
fn nested_roots_suppress_post_composition_input_in_all_listeners() {
    let (_outer, target, outer_count, inner_count, _g) = nested_wired("pin-it-nested-ime");
    target.set_value("5");
    let confirm = bubbling_event("compositionend");
    js_sys::Reflect::set(confirm.as_ref(), &"data".into(), &"5".into()).unwrap();
    target.dispatch_event(&confirm).unwrap();
    let after_confirm = inner_count.get();
    assert!(after_confirm > 0);

    let init = web_sys::InputEventInit::new();
    init.set_bubbles(true);
    let extra = web_sys::InputEvent::new_with_event_init_dict("input", &init).unwrap();
    // 合成イベントの `inputType` は初期化辞書では反映されない環境があるため、インスタンスへ直接定義する。
    let desc = js_sys::Object::new();
    js_sys::Reflect::set(&desc, &"value".into(), &"insertFromComposition".into()).unwrap();
    js_sys::Object::define_property(extra.as_ref(), &"inputType".into(), &desc);
    target.dispatch_event(&extra).unwrap();
    assert_eq!(
        inner_count.get(),
        after_confirm,
        "内側でも二重 dispatch しない"
    );
    assert_eq!(outer_count.get(), 0);
}

/// キャンセル可能なイベントを `target` へ送り、先行リスナー（target 上）が `preventDefault` する。
fn dispatch_pre_cancelled(target: &HtmlInputElement, event: &Event) {
    let cb = wasm_bindgen::closure::Closure::<dyn FnMut(Event)>::new(|e: Event| {
        e.prevent_default();
    });
    target
        .add_event_listener_with_callback(&event.type_(), cb.as_ref().unchecked_ref())
        .unwrap();
    target.dispatch_event(event).unwrap();
    target
        .remove_event_listener_with_callback(&event.type_(), cb.as_ref().unchecked_ref())
        .unwrap();
}

/// PR #3856 指摘（P1）: 先行リスナーが `preventDefault` 済みの keydown / paste は処理しない。
#[wasm_bindgen_test]
fn pre_cancelled_keydown_and_paste_are_ignored() {
    let (runtime, root, _g) = mount(
        "pin-it-cancelled",
        TestPinHost::new(4, PinInputKind::Numeric),
    );
    type_into(&root, 0, "1");
    type_into(&root, 1, "2");
    let target = digit(&root, 1);
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key("Backspace");
    let key = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap();
    dispatch_pre_cancelled(&target, &key);
    assert_eq!(runtime.component().pin.digit(1), "2");

    let init = EventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    let paste = Event::new_with_event_init_dict("paste", &init).unwrap();
    dispatch_pre_cancelled(&digit(&root, 1), &paste);
    assert_eq!(runtime.component().pin.value(), "12");
}

/// PR #3856 指摘（P1）: `data-action-input` 桁で `compositionend` の小文字が大文字へ正規化されても、
/// 直後の追加 `input` で同じ確定操作を再 dispatch しない（アプリへ 1 回だけ届く）。
#[wasm_bindgen_test]
fn app_owned_ime_confirm_normalized_to_uppercase_dispatches_once() {
    let mut host = TestPinHost::new(4, PinInputKind::AlphanumericUppercase);
    host.custom_action = true;
    let (runtime, root, _g) = mount("pin-it-ime-upper", host);
    let target = digit(&root, 0);
    target.focus().unwrap();
    let target = digit(&root, 0);
    target.set_value("a");
    let confirm = bubbling_event("compositionend");
    js_sys::Reflect::set(confirm.as_ref(), &"data".into(), &"a".into()).unwrap();
    target.dispatch_event(&confirm).unwrap();
    let target = digit(&root, 0);
    assert_eq!(runtime.component().type_calls, 1);
    assert_eq!(target.value(), "A");

    let init = web_sys::InputEventInit::new();
    init.set_bubbles(true);
    let extra = web_sys::InputEvent::new_with_event_init_dict("input", &init).unwrap();
    let desc = js_sys::Object::new();
    js_sys::Reflect::set(&desc, &"value".into(), &"insertFromComposition".into()).unwrap();
    js_sys::Object::define_property(extra.as_ref(), &"inputType".into(), &desc);
    target.dispatch_event(&extra).unwrap();
    assert_eq!(runtime.component().type_calls, 1, "二重 dispatch しない");
}
// ---------------------------------------------------------------------------
// #3836: paste・keydown・IME・大文字化の網羅的な実ブラウザ検証
// 対応表・目的写像そのものは `pin_input_native.rs` が native で検証済みのため、ここでは
// DOM で観測できる結果（状態・桁の値・`document.activeElement`・`default_prevented()`）だけを固定する。
// 値の適否は常に状態機械が決め、DOM 層は推測しない（§3.25: 検証はアプリの責務、区切り除去・大文字化は入力補助）。
// ---------------------------------------------------------------------------

/// 指定の修飾状態の `keydown` を `index` 桁へ送って返す（桁へフォーカスしてから要素を引き直す）。
fn press(
    root: &Element,
    index: usize,
    key: &str,
    ctrl: bool,
    shift: bool,
    composing: bool,
) -> KeyboardEvent {
    digit(root, index).focus().expect("focus must not fail");
    let init = KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_key(key);
    init.set_ctrl_key(ctrl);
    init.set_shift_key(shift);
    init.set_is_composing(composing);
    let event = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        .expect("KeyboardEvent::new must not fail");
    digit(root, index)
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");
    event
}

fn press_plain(root: &Element, index: usize, key: &str) -> KeyboardEvent {
    press(root, index, key, false, false, false)
}

/// `text/plain` を持つ実 `ClipboardEvent("paste")` を作る。`clipboardData` を読めない環境では
/// インスタンスへ直接定義し、それでも読めなければ前提崩れとして失敗させる（黙って素通りさせない）。
fn paste_event(text: &str) -> ClipboardEvent {
    let dt = DataTransfer::new().expect("DataTransfer::new must not fail");
    dt.set_data("text/plain", text)
        .expect("set_data must not fail");
    let init = js_sys::Object::new();
    for k in ["bubbles", "cancelable"] {
        js_sys::Reflect::set(&init, &k.into(), &true.into()).unwrap();
    }
    js_sys::Reflect::set(&init, &"clipboardData".into(), dt.as_ref()).unwrap();
    let ctor: js_sys::Function = js_sys::Reflect::get(&js_sys::global(), &"ClipboardEvent".into())
        .unwrap()
        .dyn_into()
        .expect("ClipboardEvent must be a constructor");
    let args = js_sys::Array::of2(&"paste".into(), init.as_ref());
    let event: ClipboardEvent = js_sys::Reflect::construct(&ctor, &args)
        .expect("ClipboardEvent construct must not fail")
        .dyn_into()
        .expect("must be a ClipboardEvent");
    if event.clipboard_data().is_none() {
        let desc = js_sys::Object::new();
        js_sys::Reflect::set(&desc, &"value".into(), dt.as_ref()).unwrap();
        js_sys::Object::define_property(event.as_ref(), &"clipboardData".into(), &desc);
    }
    assert!(
        event.clipboard_data().is_some(),
        "clipboardData must be readable"
    );
    event
}

fn paste_into(root: &Element, index: usize, text: &str) -> ClipboardEvent {
    digit(root, index).focus().expect("focus must not fail");
    let event = paste_event(text);
    digit(root, index)
        .dispatch_event(&event)
        .expect("dispatch_event must not fail");
    event
}

/// 状態の各桁と DOM の各桁の値が一致していること。
fn assert_dom_matches_state(runtime: &Runtime<TestPinHost>, root: &Element) {
    let pin = &runtime.component().pin;
    for i in 0..pin.count() {
        assert_eq!(digit(root, i).value(), pin.digit(i), "digit {i}");
    }
}

/// マウント前に `chars` を 1 文字ずつ状態へ入力しておいたホスト（事前状態の構築用）。
fn filled(count: usize, kind: PinInputKind, chars: &str) -> TestPinHost {
    let mut host = TestPinHost::new(count, kind);
    for c in chars.chars() {
        host.pin.update(PinInputAction::Input(c));
    }
    host
}

/// 区切り入りコード（RFC 8628 の `WDJB-MJHT` 形）の貼り付けは全桁へ分配され、最終桁へフォーカスが移る。
#[wasm_bindgen_test]
fn paste_hyphenated_code_fills_all_digits() {
    let (runtime, root, _g) = mount(
        "pin-it-paste-a",
        TestPinHost::new(8, PinInputKind::AlphanumericUppercase),
    );
    let ev = paste_into(&root, 0, "WDJB-MJHT");
    assert!(ev.default_prevented());
    assert_eq!(runtime.component().pin.value(), "WDJBMJHT");
    assert_dom_matches_state(&runtime, &root);
    assert_eq!(active_digit(&root).as_deref(), Some("7"));
}

/// 小文字の貼り付けは大文字化と区切り除去が同時に効く。
#[wasm_bindgen_test]
fn paste_lowercase_is_uppercased_and_separators_removed() {
    let (runtime, root, _g) = mount(
        "pin-it-paste-b",
        TestPinHost::new(8, PinInputKind::AlphanumericUppercase),
    );
    paste_into(&root, 0, "wdjb-mjht");
    assert_eq!(runtime.component().pin.value(), "WDJBMJHT");
    assert_dom_matches_state(&runtime, &root);
}

/// kind 不一致の貼り付けは全体拒否され、生の文字列は DOM の桁へ入らない（`preventDefault` 済み）。
#[wasm_bindgen_test]
fn paste_of_wrong_kind_is_rejected_without_raw_text_in_dom() {
    let (runtime, root, _g) = mount("pin-it-paste-c", TestPinHost::new(8, PinInputKind::Numeric));
    let ev = paste_into(&root, 0, "WDJB-MJHT");
    assert!(ev.default_prevented());
    assert_eq!(runtime.component().pin.value(), "");
    assert_dom_matches_state(&runtime, &root);
}

/// 桁数超過の貼り付けは拒否され、貼り付け前の入力は保持される。
#[wasm_bindgen_test]
fn paste_longer_than_digit_count_is_rejected_and_keeps_input() {
    let host = filled(8, PinInputKind::AlphanumericUppercase, "AB");
    let (runtime, root, _g) = mount("pin-it-paste-d", host);
    paste_into(&root, 2, "WDJB-MJHTX");
    assert_eq!(runtime.component().pin.value(), "AB");
    assert_dom_matches_state(&runtime, &root);
}

/// 短い貼り付けは先頭から充填し、残りの桁をクリアする。
#[wasm_bindgen_test]
fn short_paste_replaces_and_clears_remaining_digits() {
    let host = filled(6, PinInputKind::Numeric, "123456");
    let (runtime, root, _g) = mount("pin-it-paste-e", host);
    paste_into(&root, 0, "98-7");
    assert_eq!(runtime.component().pin.value(), "987");
    assert_dom_matches_state(&runtime, &root);
    assert_eq!(active_digit(&root).as_deref(), Some("3"));
}

/// 非先頭桁での貼り付けでも先頭桁から充填される（状態機械の仕様）。
#[wasm_bindgen_test]
fn paste_from_non_first_digit_fills_from_the_first() {
    let (runtime, root, _g) = mount("pin-it-paste-f", TestPinHost::new(6, PinInputKind::Numeric));
    paste_into(&root, 3, "1234");
    assert_eq!(runtime.component().pin.digit(0), "1");
    assert_eq!(runtime.component().pin.value(), "1234");
    assert_dom_matches_state(&runtime, &root);
}

/// 区切り文字だけの貼り付けは拒否され状態は不変。
#[wasm_bindgen_test]
fn paste_of_separators_only_is_rejected() {
    let host = filled(6, PinInputKind::Numeric, "12");
    let (runtime, root, _g) = mount("pin-it-paste-g", host);
    paste_into(&root, 0, "- -");
    assert_eq!(runtime.component().pin.value(), "12");
    assert_dom_matches_state(&runtime, &root);
}

/// 矢印キーは値を書かずフォーカスだけを動かし、`preventDefault` する。
#[wasm_bindgen_test]
fn arrow_left_and_right_move_focus_without_touching_values() {
    let host = filled(5, PinInputKind::Numeric, "1234");
    let (runtime, root, _g) = mount("pin-it-arrows", host);
    let ev = press_plain(&root, 2, "ArrowLeft");
    assert!(ev.default_prevented());
    assert_eq!(active_digit(&root).as_deref(), Some("1"));
    let ev = press_plain(&root, 1, "ArrowRight");
    assert!(ev.default_prevented());
    assert_eq!(active_digit(&root).as_deref(), Some("2"));
    assert_eq!(runtime.component().pin.value(), "1234");
    assert_dom_matches_state(&runtime, &root);
}

/// 先頭桁の ArrowLeft・最終桁の ArrowRight はフォーカスも値も動かさない。
#[wasm_bindgen_test]
fn arrows_at_boundaries_keep_focus_and_values() {
    let host = filled(4, PinInputKind::Numeric, "1234");
    let (runtime, root, _g) = mount("pin-it-arrow-edge", host);
    press_plain(&root, 0, "ArrowLeft");
    assert_eq!(active_digit(&root).as_deref(), Some("0"));
    press_plain(&root, 3, "ArrowRight");
    assert_eq!(active_digit(&root).as_deref(), Some("3"));
    assert_eq!(runtime.component().pin.value(), "1234");
    assert_dom_matches_state(&runtime, &root);
}

/// 先頭桁の Backspace は桁を消去してその桁に留まる。
#[wasm_bindgen_test]
fn backspace_on_first_digit_clears_and_stays() {
    let host = filled(4, PinInputKind::Numeric, "12");
    let (runtime, root, _g) = mount("pin-it-bs-first", host);
    press_plain(&root, 0, "Backspace");
    assert_eq!(runtime.component().pin.digit(0), "");
    assert_eq!(runtime.component().pin.digit(1), "2");
    assert_eq!(active_digit(&root).as_deref(), Some("0"));
    assert_dom_matches_state(&runtime, &root);
}

/// 空の桁の Backspace は前の桁の値を消さず、フォーカスだけ戻す。
#[wasm_bindgen_test]
fn backspace_on_empty_digit_keeps_previous_value_and_moves_back() {
    let host = filled(4, PinInputKind::Numeric, "12");
    let (runtime, root, _g) = mount("pin-it-bs-empty", host);
    press_plain(&root, 2, "Backspace");
    assert_eq!(runtime.component().pin.value(), "12");
    assert_eq!(active_digit(&root).as_deref(), Some("1"));
    assert_dom_matches_state(&runtime, &root);
}

/// 末尾から連続 Backspace すると全桁が空になり先頭桁へ到達する。
#[wasm_bindgen_test]
fn repeated_backspace_clears_everything_and_reaches_first_digit() {
    let host = filled(4, PinInputKind::Numeric, "1234");
    let (runtime, root, _g) = mount("pin-it-bs-all", host);
    for index in [3, 2, 1, 0] {
        press_plain(&root, index, "Backspace");
    }
    assert_eq!(runtime.component().pin.value(), "");
    assert_eq!(active_digit(&root).as_deref(), Some("0"));
    assert_dom_matches_state(&runtime, &root);
}

/// Delete は現在桁だけ消し、フォーカスは動かさない。
#[wasm_bindgen_test]
fn delete_clears_only_the_current_digit_and_keeps_focus() {
    let host = filled(4, PinInputKind::Numeric, "123");
    let (runtime, root, _g) = mount("pin-it-delete", host);
    let ev = press_plain(&root, 1, "Delete");
    assert!(ev.default_prevented());
    assert_eq!(runtime.component().pin.digit(1), "");
    assert_eq!(runtime.component().pin.digit(0), "1");
    assert_eq!(runtime.component().pin.digit(2), "3");
    assert_eq!(active_digit(&root).as_deref(), Some("1"));
    assert_dom_matches_state(&runtime, &root);
}

/// 修飾キー付きの keydown はネイティブ動作に任せる（`preventDefault` も dispatch もしない）。
#[wasm_bindgen_test]
fn modified_keydown_is_left_to_the_browser() {
    let host = filled(4, PinInputKind::Numeric, "123");
    let (runtime, root, _g) = mount("pin-it-mods", host);
    let ev = press(&root, 2, "Backspace", true, false, false);
    assert!(!ev.default_prevented());
    let ev = press(&root, 2, "ArrowLeft", false, true, false);
    assert!(!ev.default_prevented());
    assert_eq!(runtime.component().pin.value(), "123");
}

/// 変換中（`isComposing`）の keydown は無視する。
#[wasm_bindgen_test]
fn composing_keydown_is_ignored() {
    let host = filled(4, PinInputKind::Numeric, "123");
    let (runtime, root, _g) = mount("pin-it-composing-key", host);
    let ev = press(&root, 2, "Backspace", false, false, true);
    assert!(!ev.default_prevented());
    assert_eq!(runtime.component().pin.value(), "123");
}

/// 汎用経路の `compositionend` 確定は大文字化して 1 回だけ適用され、直後の追加 `input` で二重適用されない。
#[wasm_bindgen_test]
fn compositionend_confirms_uppercased_value_once() {
    let (runtime, root, _g) = mount(
        "pin-it-comp-generic",
        TestPinHost::new(4, PinInputKind::AlphanumericUppercase),
    );
    digit(&root, 0).focus().unwrap();
    let target = digit(&root, 0);
    target.set_value("a");
    let confirm = bubbling_event("compositionend");
    js_sys::Reflect::set(confirm.as_ref(), &"data".into(), &"a".into()).unwrap();
    target.dispatch_event(&confirm).unwrap();
    assert_eq!(runtime.component().pin.digit(0), "A");
    assert_eq!(digit(&root, 0).value(), "A");
    assert_eq!(active_digit(&root).as_deref(), Some("1"));

    let init = web_sys::InputEventInit::new();
    init.set_bubbles(true);
    let extra = web_sys::InputEvent::new_with_event_init_dict("input", &init).unwrap();
    let desc = js_sys::Object::new();
    js_sys::Reflect::set(&desc, &"value".into(), &"insertFromComposition".into()).unwrap();
    js_sys::Object::define_property(extra.as_ref(), &"inputType".into(), &desc);
    digit(&root, 0).dispatch_event(&extra).unwrap();
    assert_eq!(runtime.component().pin.value(), "A");
    assert_eq!(active_digit(&root).as_deref(), Some("1"));
}

/// 変換中の `input` は状態へ届かない。
#[wasm_bindgen_test]
fn input_during_composition_is_ignored() {
    let (runtime, root, _g) = mount(
        "pin-it-comp-input",
        TestPinHost::new(4, PinInputKind::Numeric),
    );
    digit(&root, 0).focus().unwrap();
    let target = digit(&root, 0);
    target.set_value("5");
    let init = web_sys::InputEventInit::new();
    init.set_bubbles(true);
    init.set_is_composing(true);
    let ev = web_sys::InputEvent::new_with_event_init_dict("input", &init).unwrap();
    let desc = js_sys::Object::new();
    js_sys::Reflect::set(&desc, &"value".into(), &true.into()).unwrap();
    js_sys::Object::define_property(ev.as_ref(), &"isComposing".into(), &desc);
    target.dispatch_event(&ev).unwrap();
    assert_eq!(runtime.component().pin.value(), "");
}

/// 空の桁へ小文字を入力すると、状態・DOM とも大文字になり次の桁へ進む。
#[wasm_bindgen_test]
fn lowercase_typed_into_empty_digit_is_uppercased() {
    let (runtime, root, _g) = mount(
        "pin-it-upper-new",
        TestPinHost::new(4, PinInputKind::AlphanumericUppercase),
    );
    type_into(&root, 0, "w");
    assert_eq!(runtime.component().pin.digit(0), "W");
    assert_eq!(digit(&root, 0).value(), "W");
    assert_eq!(active_digit(&root).as_deref(), Some("1"));
}

/// RFC 8628 のユーザーコード入力を想定した一連操作（入力・Backspace・打ち直し・矢印で戻る）。
#[wasm_bindgen_test]
fn rfc8628_user_code_entry_scenario() {
    let (runtime, root, _g) = mount(
        "pin-it-scenario",
        TestPinHost::new(8, PinInputKind::AlphanumericUppercase),
    );
    for (i, c) in ["w", "d", "j"].into_iter().enumerate() {
        type_into(&root, i, c);
    }
    assert_eq!(runtime.component().pin.value(), "WDJ");
    assert_eq!(active_digit(&root).as_deref(), Some("3"));
    // 空の桁 3 の Backspace は前の桁の値を消さずフォーカスだけ戻す。続けて桁 2 の Backspace で 'J' を消す。
    press_plain(&root, 3, "Backspace");
    assert_eq!(runtime.component().pin.value(), "WDJ");
    assert_eq!(active_digit(&root).as_deref(), Some("2"));
    press_plain(&root, 2, "Backspace");
    assert_eq!(runtime.component().pin.value(), "WD");
    assert_eq!(active_digit(&root).as_deref(), Some("1"));
    type_into(&root, 2, "j");
    assert_eq!(active_digit(&root).as_deref(), Some("3"));
    press_plain(&root, 3, "ArrowLeft");
    assert_eq!(active_digit(&root).as_deref(), Some("2"));
    for (i, c) in ["j", "b", "m", "j", "h", "t"].into_iter().enumerate() {
        type_into(&root, 2 + i, c);
    }
    assert_eq!(runtime.component().pin.value(), "WDJBMJHT");
    assert_dom_matches_state(&runtime, &root);
}

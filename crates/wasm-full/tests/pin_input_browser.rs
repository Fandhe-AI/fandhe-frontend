//! `fandhe_frontend_wasm_full::Runtime::mount`/`hydrate` が PinInput の `input` / `focusin`
//! を配線することを確認する実ブラウザ統合テスト（`wasm-pack test --headless --chrome`、
//! イシュー #3834、PR #3855 レビュー指摘 1〜11 の回帰固定）。
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
use web_sys::{Document, Element, Event, EventInit, HtmlInputElement};

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
        }
    }
}

impl Component for TestPinHost {
    type Action = HostAction;

    fn update(&mut self, action: HostAction) {
        match action {
            HostAction::Pin(a) => self.pin.update(a),
            HostAction::TypeAt(i, c) => {
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

/// ネイティブ削除（Empty）した桁は、別の桁の入力で状態から復活しない（値の書き込みは入力のあった桁だけ）。
#[wasm_bindgen_test]
fn natively_cleared_digit_is_not_resurrected_by_another_digits_input() {
    let (runtime, root, _g) = mount(
        "pin-it-noresurrect",
        TestPinHost::new(4, PinInputKind::Numeric),
    );
    type_into(&root, 0, "1");
    type_into(&root, 1, "2");
    type_into(&root, 2, "3");
    type_into(&root, 2, "");
    type_into(&root, 0, "5");
    assert_eq!(runtime.component().pin.digit(2), "3");
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

/// 0 文字・2 文字以上は #3835 の担当のため何も dispatch せず DOM の値も変えない。
#[wasm_bindgen_test]
fn empty_and_multi_char_values_are_left_alone() {
    let (runtime, root, _g) = mount("pin-it-multi", TestPinHost::new(4, PinInputKind::Numeric));
    type_into(&root, 0, "12");
    assert_eq!(runtime.component().pin.value(), "");
    assert_eq!(digit(&root, 0).value(), "12");
    type_into(&root, 1, "");
    assert_eq!(runtime.component().pin.value(), "");
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

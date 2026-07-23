//! `fandhe_frontend_wasm_full::focus_visible::wire_focus_visible`
//!（hidden-input パターンのフォーカスリング配線・イシュー #709、親 #520）の
//! 実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/src/focus_visible.rs` の native テストは純粋層
//! （[`fandhe_frontend_wasm_full::focus_visible::boundary_part_for`]）までを
//! 検証済みである。本ファイルはその先、`wire_focus_visible` が実 DOM
//! （headless Chromium）上で hidden-input の focusin/focusout に応じて
//! `data-focus-visible` を境界パーツ・同一 scope の descendant パーツへ
//! 正しく付け外しすることを検証する。
//!
//! DOM 構造は `crates/headless-ui/src/switch.rs`/`radio_group.rs` の SSR
//! 出力契約（`data-scope`/`data-part`）を手組みで再現する（本クレートは
//! `fandhe-frontend-headless-ui` に依存しないため、`keynav_browser.rs` と
//! 同方針で属性契約のみを手組みする）。
//!
//! # `:focus-visible` 判定について（モジュール rustdoc 参照）
//!
//! `HtmlElement::focus()` によるスクリプト起因のフォーカスは、直前に
//! マウス/ポインタ操作による干渉がない限り Chromium の既定挙動で
//! `:focus-visible` に一致する（キーボード操作等価として扱われる）。本
//! テストは事前のポインタ操作を行わないため、`focus()` 呼び出し後に
//! `data-focus-visible` が付与されることを期待する。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_wasm_full::focus_visible::wire_focus_visible;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, HtmlElement, MouseEvent, MouseEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾でプレースホルダを document から確実に除去する RAII ガード
/// （`keynav_browser.rs::RemoveOnDrop` と同じ意図。テスト間 DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

fn html_element(element: &Element) -> HtmlElement {
    element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("element must be an HtmlElement")
}

/// `target` へ `bubbles: true` の `MouseEvent` を dispatch する（pointerdown
/// 名を渡しても `PointerEvent` ではなく `MouseEvent` として発火するが、
/// `wire_focus_visible` 側のリスナーは `Event`/`event.target()` のみを見る
/// ため検証上は問題ない。実ブラウザの `:focus-visible` 内部判定が
/// スクリプト発火イベントに追随するかは Chromium の実装依存であり本テストは
/// その挙動如何によらず「イベント発火直後、`data-focus-visible` の有無が
/// 常に `:focus-visible` の実判定と一致する」という不変条件を検証する）。
fn dispatch_mouse_event(target: &Element, event_type: &str) {
    let init = MouseEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    let event = MouseEvent::new_with_mouse_event_init_dict(event_type, &init)
        .expect("MouseEvent construction must not fail");
    target
        .dispatch_event(&Event::from(event))
        .expect("dispatch_event must not fail");
}

/// `crates/headless-ui/src/switch.rs` の SSR 出力契約（root > control +
/// hidden-input の兄弟配置）を手組みで再現した Switch DOM を生成する。
/// 返り値: `(root, control, hidden_input)`。
fn build_switch_dom(document: &Document, root_id: &str) -> (Element, Element, Element) {
    let root = document.create_element("label").unwrap();
    root.set_id(root_id);
    root.set_attribute("data-scope", "switch").unwrap();
    root.set_attribute("data-part", "root").unwrap();

    let control = document.create_element("span").unwrap();
    control.set_attribute("data-scope", "switch").unwrap();
    control.set_attribute("data-part", "control").unwrap();
    root.append_child(&control).unwrap();

    let hidden_input = document.create_element("input").unwrap();
    hidden_input.set_attribute("type", "checkbox").unwrap();
    hidden_input.set_attribute("data-scope", "switch").unwrap();
    hidden_input
        .set_attribute("data-part", "hidden-input")
        .unwrap();
    root.append_child(&hidden_input).unwrap();

    document
        .body()
        .unwrap()
        .append_child(&root)
        .expect("append_child must not fail for a detached label");
    (root, control, hidden_input)
}

/// `crates/headless-ui/src/radio_group.rs` の SSR 出力契約（root > item
/// （label） > item-control + item-hidden-input）を手組みで再現した
/// RadioGroup DOM を生成する。返り値: `(root, item, item_control,
/// item_hidden_input)`。
fn build_radio_group_dom(
    document: &Document,
    root_id: &str,
) -> (Element, Element, Element, Element) {
    let root = document.create_element("div").unwrap();
    root.set_id(root_id);
    root.set_attribute("data-scope", "radio-group").unwrap();
    root.set_attribute("data-part", "root").unwrap();

    let item = document.create_element("label").unwrap();
    item.set_attribute("data-scope", "radio-group").unwrap();
    item.set_attribute("data-part", "item").unwrap();

    let item_control = document.create_element("span").unwrap();
    item_control
        .set_attribute("data-scope", "radio-group")
        .unwrap();
    item_control
        .set_attribute("data-part", "item-control")
        .unwrap();
    item.append_child(&item_control).unwrap();

    let item_hidden_input = document.create_element("input").unwrap();
    item_hidden_input.set_attribute("type", "radio").unwrap();
    item_hidden_input
        .set_attribute("data-scope", "radio-group")
        .unwrap();
    item_hidden_input
        .set_attribute("data-part", "item-hidden-input")
        .unwrap();
    item.append_child(&item_hidden_input).unwrap();

    root.append_child(&item).unwrap();
    document
        .body()
        .unwrap()
        .append_child(&root)
        .expect("append_child must not fail for a detached div");
    (root, item, item_control, item_hidden_input)
}

/// 検証 1（Switch）: hidden-input への focus で `root`/`control` 双方へ
/// `data-focus-visible` が付与され、blur で双方から除去される（付与・除去
/// 両方向を検証する。片方向のみで PASS 扱いにしない）。
#[wasm_bindgen_test]
fn switch_hidden_input_focus_and_blur_toggle_data_focus_visible_on_root_and_control() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, control, hidden_input) = build_switch_dom(&document, "fv-switch-1");
    let _cleanup = RemoveOnDrop(root.clone());
    wire_focus_visible(root.clone()).expect("wire_focus_visible must succeed");

    assert!(!root.has_attribute("data-focus-visible"));
    assert!(!control.has_attribute("data-focus-visible"));

    html_element(&hidden_input)
        .focus()
        .expect("focus must not fail");

    assert!(
        root.has_attribute("data-focus-visible"),
        "root must gain data-focus-visible after hidden-input focus"
    );
    assert!(
        control.has_attribute("data-focus-visible"),
        "control must gain data-focus-visible after hidden-input focus (recipe selector targets control directly)"
    );

    html_element(&hidden_input)
        .blur()
        .expect("blur must not fail");

    assert!(
        !root.has_attribute("data-focus-visible"),
        "root must lose data-focus-visible after hidden-input blur"
    );
    assert!(
        !control.has_attribute("data-focus-visible"),
        "control must lose data-focus-visible after hidden-input blur"
    );
}

/// 検証 2（RadioGroup）: `item-hidden-input` への focus で `item`/
/// `item-control` 双方へ `data-focus-visible` が付与され、blur で除去される。
#[wasm_bindgen_test]
fn radio_group_item_hidden_input_focus_and_blur_toggle_data_focus_visible_on_item_and_item_control()
{
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, item, item_control, item_hidden_input) =
        build_radio_group_dom(&document, "fv-radio-1");
    let _cleanup = RemoveOnDrop(root.clone());
    wire_focus_visible(root.clone()).expect("wire_focus_visible must succeed");

    html_element(&item_hidden_input)
        .focus()
        .expect("focus must not fail");

    assert!(
        item.has_attribute("data-focus-visible"),
        "item must gain data-focus-visible after item-hidden-input focus"
    );
    assert!(
        item_control.has_attribute("data-focus-visible"),
        "item-control must gain data-focus-visible after item-hidden-input focus"
    );

    html_element(&item_hidden_input)
        .blur()
        .expect("blur must not fail");

    assert!(!item.has_attribute("data-focus-visible"));
    assert!(!item_control.has_attribute("data-focus-visible"));
}

/// 検証 3: `root` の外側にある hidden-input パターンには反応しない
/// （`events::wire_events`/`keynav::wire_keynav` と同じ `contains` 封じ込め）。
#[wasm_bindgen_test]
fn hidden_input_outside_wired_root_is_ignored() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, _control, _hidden_input) = build_switch_dom(&document, "fv-switch-outside-wired");
    let _cleanup_wired = RemoveOnDrop(root.clone());
    wire_focus_visible(root.clone()).expect("wire_focus_visible must succeed");

    // wire_focus_visible の対象外の別 root 配下に同型の Switch DOM を生成する。
    let (other_root, other_control, other_hidden_input) =
        build_switch_dom(&document, "fv-switch-unwired");
    let _cleanup_unwired = RemoveOnDrop(other_root.clone());

    html_element(&other_hidden_input)
        .focus()
        .expect("focus must not fail");

    assert!(!other_root.has_attribute("data-focus-visible"));
    assert!(!other_control.has_attribute("data-focus-visible"));
}

/// 検証 4（イシュー #709 PR #720 Cursor Bugbot 指摘の回帰テスト）:
/// hidden-input がフォーカスを保持したまま pointerdown/mousedown/click を
/// 受けても、`data-focus-visible` の有無は常にその時点の
/// `:focus-visible` 実判定と一致し続ける（focusin/focusout のみに依存する
/// 旧実装では、これらのイベントで判定が変化しても blur まで
/// `data-focus-visible` が残留し不変条件が崩れ得た）。
///
/// スクリプト発火イベントが Chromium の `:focus-visible` 内部判定を実際に
/// 変化させるかは実装依存のため、本テストは「変化の有無によらず追随する」
/// ことを検証する（`dispatch_mouse_event` doc 参照）。
#[wasm_bindgen_test]
fn pointer_events_while_focus_retained_keep_data_focus_visible_in_sync_with_focus_visible_match() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (root, control, hidden_input) = build_switch_dom(&document, "fv-switch-pointer-sync");
    let _cleanup = RemoveOnDrop(root.clone());
    wire_focus_visible(root.clone()).expect("wire_focus_visible must succeed");

    html_element(&hidden_input)
        .focus()
        .expect("focus must not fail");
    assert!(
        root.has_attribute("data-focus-visible"),
        "root must gain data-focus-visible after hidden-input focus"
    );

    for event_type in ["pointerdown", "mousedown", "click"] {
        dispatch_mouse_event(&hidden_input, event_type);

        let expected = hidden_input.matches(":focus-visible").unwrap_or(false);
        assert_eq!(
            root.has_attribute("data-focus-visible"),
            expected,
            "root data-focus-visible must track :focus-visible after {event_type} \
             while hidden-input retains focus"
        );
        assert_eq!(
            control.has_attribute("data-focus-visible"),
            expected,
            "control data-focus-visible must track :focus-visible after {event_type} \
             while hidden-input retains focus"
        );
    }
}

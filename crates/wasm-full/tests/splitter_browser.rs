//! `Runtime::wire_splitter` の dispatch 後再描画接続（イシュー #1996）の
//! 実ブラウザ回帰テスト（`wasm-pack test --headless --chrome`）。
//!
//! `crates/wasm-full/src/splitter.rs`（`wiring` モジュール）の native
//! `#[cfg(test)] mod tests` は `wire_splitter_events` が発火する
//! `(action, payload)` の組み立て（キー→アクション変換・disabled 除外・
//! root 封じ込め）のみを、`crates/wasm-full/tests/keynav_browser.rs` の
//! Splitter ケース群が手組みの記録用クロージャ（`RecordedActions`）越しに
//! 検証する。いずれも `Runtime::wire_splitter`（`crates/wasm-full/src/
//! lib.rs`）を経由しないため、修正前の `wire_splitter` 実装（
//! `fandhe_frontend_interactive::dispatch` を直接呼ぶのみで
//! `Self::wire` の束縛点更新・keyed list 差し替え・構造フォールバックを
//! 一切呼ばない）でも同様に green だった（＝今回の修正を検証しない）。
//!
//! 本ファイルは `angle_slider_browser.rs`（イシュー #1956/#1957）と同型の
//! パターンで、`Runtime::mount`/`Runtime::hydrate` 経由で配線した実
//! Splitter への ArrowRight keydown が
//!
//! 1. hydrate 経路（[`hydrate_arrow_right_keydown_dispatches_and_rerenders_dom`]）
//! 2. mount（CSR）経路（[`mount_arrow_right_keydown_dispatches_and_rerenders_dom`]）
//!
//! の双方で、resize-trigger 自身の `aria-valuenow`（`data-bind-attr`
//! 束縛点、モジュール冒頭 doc「束縛点設計」節参照）と `C`（[`SplitterHost`]）
//! 側の派生フィールド（`data-bind-text` 束縛点）へ正しく反映されることを
//! 検証する。加えて disabled resize-trigger への keydown が no-op のままで
//! あること（[`disabled_resize_trigger_keydown_is_noop`]）も固定する。
//!
//! # 束縛点設計
//!
//! [`fandhe_frontend_headless_ui::splitter::resize_trigger`] が出力する
//! `aria-valuenow` は SSR 時点の値を焼き込んだ**静的属性**であり、
//! 束縛点マーカー（`data-bind-attr`）を自動では持たない
//! （`RESIZE_TRIGGER_RESERVED` に `data-bind-attr` は含まれない、
//! `Runtime::wire_splitter` rustdoc 参照）。このため [`SplitterHost::view`]
//! は `resize_trigger` 呼び出し側 `attrs` へ
//! `fandhe_frontend_core::bind_attr_tokens(&[("aria-valuenow", "size_now")])`
//! を明示的に付与し、[`fandhe_frontend_wasm_client::BindingSource`] で
//! `size_now` を解決し、`update()` でトリガー 0 の先行パネルサイズが変化
//! した際に `dirty_fields()` へ積む。これは `Runtime::wire_splitter`
//! rustdoc「Splitter 自身の `aria-valuenow` 等を更新するにはアプリ側の
//! 束縛点配線が必要」が実際に成立するための、アプリ側が担うべき配線
//! （`Self::wire` の既存束縛点更新経路のみを使い、
//! `fandhe-frontend-wasm-full`/`fandhe-frontend-headless-ui` 側の変更は
//! 不要）である。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::{bind_attr_tokens, bind_text, render, Node};
use fandhe_frontend_headless_ui::splitter::{PanelSpec, Splitter, SplitterAction};
use fandhe_frontend_headless_ui::Orientation;
use fandhe_frontend_interactive::{Component, DirtyTracked, Hydrate, HydrateError};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, KeyboardEvent, KeyboardEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト終了時に要素を DOM から除去する RAII ガード
/// （`angle_slider_browser.rs::RemoveOnDrop` と同型）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        if let Some(parent) = self.0.parent_node() {
            let _ = parent.remove_child(&self.0);
        }
    }
}

/// `key` の keydown `Event` を組み立てる（bubbles: true、cancelable: true。
/// `wire_splitter_events` は root への delegation でリスンするため
/// bubbles が必須）。
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

/// 2 パネル・1 リサイズトリガーの最小 Splitter を包むテストホスト。
/// トリガー 0 の先行パネル（`panel-0`）サイズを `size_now` として
/// `aria-valuenow` へ束縛する（モジュール冒頭 doc「束縛点設計」節参照）。
struct SplitterHost {
    splitter: Splitter,
    root_id: String,
    disabled: bool,
    trigger_bind_attr: String,
    size_now: String,
    size_label: String,
    dirty: Vec<&'static str>,
}

/// ホスト自身（`Splitter` の外側）の状態を往復させる `data-hydrate-*`
/// 属性名（`angle_slider_browser.rs` の `HOST_ATTR_*` と同型）。
const HOST_ATTR_ROOT_ID: &str = "data-hydrate-host-root-id";
/// ホストの `disabled` を往復させる属性名。
const HOST_ATTR_DISABLED: &str = "data-hydrate-host-disabled";

impl SplitterHost {
    fn new(root_id: &str) -> Self {
        Self::with_disabled(root_id, false)
    }

    fn disabled(root_id: &str) -> Self {
        Self::with_disabled(root_id, true)
    }

    fn with_disabled(root_id: &str, disabled: bool) -> Self {
        let splitter = Splitter::new(
            &[
                PanelSpec::new(50.0, 0.0, 100.0),
                PanelSpec::new(50.0, 0.0, 100.0),
            ],
            Orientation::Horizontal,
        );
        let size_now = format!("{}", splitter.size(0).unwrap_or(50.0));
        let size_label = size_now.clone();
        let trigger_bind_attr = bind_attr_tokens(&[("aria-valuenow", "size_now")]);
        Self {
            splitter,
            root_id: root_id.to_string(),
            disabled,
            trigger_bind_attr,
            size_now,
            size_label,
            dirty: Vec::new(),
        }
    }
}

impl Component for SplitterHost {
    type Action = SplitterAction;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        self.splitter.update(action);
        let now = format!("{}", self.splitter.size(0).unwrap_or(50.0));
        if now != self.size_now {
            self.size_now = now.clone();
            self.dirty.push("size_now");
        }
        if now != self.size_label {
            self.size_label = now;
            self.dirty.push("size_label");
        }
    }

    fn view(&self) -> Node {
        self.splitter.root(
            self.disabled,
            vec![("id", self.root_id.as_str())],
            vec![
                self.splitter.panel(0, "panel-0", Vec::new(), Vec::new()),
                self.splitter.resize_trigger(
                    0,
                    "panel-0",
                    "panel-1",
                    self.disabled,
                    vec![("data-bind-attr", self.trigger_bind_attr.as_str())],
                    Vec::new(),
                ),
                self.splitter.panel(1, "panel-1", Vec::new(), Vec::new()),
                bind_text(
                    "span",
                    vec![("data-testid", "size-label")],
                    "size_label",
                    self.size_label.clone(),
                ),
            ],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
        Splitter::decode_action(name, payload)
    }
}

impl DirtyTracked for SplitterHost {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

impl BindingSource for SplitterHost {
    fn bound_value(&self, field: &str) -> Option<BoundValue> {
        match field {
            "size_now" => Some(BoundValue::Text(self.size_now.clone())),
            "size_label" => Some(BoundValue::Text(self.size_label.clone())),
            _ => None,
        }
    }
}

impl Hydrate for SplitterHost {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let mut attrs = self.splitter.hydration_attrs();
        attrs.push((HOST_ATTR_ROOT_ID.to_string(), self.root_id.clone()));
        attrs.push((HOST_ATTR_DISABLED.to_string(), self.disabled.to_string()));
        attrs
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let splitter = Splitter::from_hydration_attrs(attrs)?;

        let find = |name: &str| -> Result<&str, HydrateError> {
            attrs
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.as_str())
                .ok_or_else(|| HydrateError::MissingAttr(name.to_string()))
        };
        let root_id = find(HOST_ATTR_ROOT_ID)?.to_string();
        let disabled = match find(HOST_ATTR_DISABLED)? {
            "true" => true,
            "false" => false,
            _ => {
                return Err(HydrateError::InvalidValue {
                    attr: HOST_ATTR_DISABLED.to_string(),
                    reason: "expected \"true\" or \"false\"".to_string(),
                })
            }
        };

        let size_now = format!("{}", splitter.size(0).unwrap_or(50.0));
        let size_label = size_now.clone();
        let trigger_bind_attr = bind_attr_tokens(&[("aria-valuenow", "size_now")]);
        Ok(Self {
            splitter,
            root_id,
            disabled,
            trigger_bind_attr,
            size_now,
            size_label,
            dirty: Vec::new(),
        })
    }
}

fn document() -> Document {
    web_sys::window()
        .expect("window must exist")
        .document()
        .expect("document must exist")
}

/// `host` を document body へ挿入し、`host.hydration_attrs()` を DOM
/// 属性として付与したうえで `Runtime::hydrate` を呼ぶ
/// （`angle_slider_browser.rs::mount` と同型）。
fn mount_via_hydrate(host: SplitterHost) -> (Element, Runtime<SplitterHost>) {
    let document = document();
    let root_id = host.root_id.clone();
    let html = render(&host.view());
    document
        .body()
        .expect("document body must exist in browser test environment")
        .insert_adjacent_html("beforeend", &html)
        .expect("insert_adjacent_html must not fail");
    let root_el = document
        .get_element_by_id(&root_id)
        .expect("rendered Splitter root must have the expected id");

    for (name, value) in host.hydration_attrs() {
        root_el
            .set_attribute(&name, &value)
            .expect("set_attribute must not fail");
    }

    let runtime =
        Runtime::hydrate(&root_id, host).expect("hydrate must succeed for well-formed attrs");
    assert_eq!(
        runtime.root().id(),
        root_id,
        "hydrate は root_id 要素自身を Splitter root として復元すること"
    );
    (root_el, runtime)
}

/// 空の `#root_id` 要素へ `Runtime::mount`（CSR 経路）で `host` を配線する。
fn mount_via_csr(host: SplitterHost) -> (Element, Runtime<SplitterHost>) {
    let document = document();
    let root_id = host.root_id.clone();
    let container = document.create_element("div").expect("create_element");
    container
        .set_attribute("id", &root_id)
        .expect("set_attribute must not fail");
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&container)
        .expect("append_child must not fail");

    let runtime = Runtime::mount(&root_id, host).expect("mount must succeed");
    let root_el = runtime.root().clone();
    (root_el, runtime)
}

fn resize_trigger(root_el: &Element) -> Element {
    root_el
        .query_selector("[data-scope='splitter'][data-part='resize-trigger']")
        .expect("query_selector must not fail")
        .expect("resize-trigger part must exist")
}

fn size_label(root_el: &Element) -> String {
    root_el
        .query_selector("[data-bind-text='size_label']")
        .expect("query_selector must not fail")
        .expect("size_label binding point must exist")
        .text_content()
        .unwrap_or_default()
}

/// `Runtime::hydrate` 経由で配線した resize-trigger keydown（`ArrowRight`）
/// が、Splitter 自身の `aria-valuenow`（`data-bind-attr` 束縛点）と `C`
/// （`SplitterHost`）側の `data-bind-text="size_label"` の双方へ反映される
/// ことを検証する（受け入れ条件、イシュー #1996）。修正前の
/// `Runtime::wire_splitter`（`fandhe_frontend_interactive::dispatch` を
/// 直接呼ぶのみ）では `aria-valuenow`/`size_label` のいずれも dispatch 後に
/// 更新されず、次の click/input まで DOM に反映されないバグだった。
#[wasm_bindgen_test]
fn hydrate_arrow_right_keydown_dispatches_and_rerenders_dom() {
    let (root_el, runtime) = mount_via_hydrate(SplitterHost::new("splitter-host-hydrate-root"));
    let _cleanup = RemoveOnDrop(root_el.clone());

    let trigger = resize_trigger(&root_el);
    assert_eq!(
        trigger.get_attribute("aria-valuenow").as_deref(),
        Some("50")
    );
    assert_eq!(size_label(&root_el), "50");

    let default_not_prevented = trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(
        !default_not_prevented,
        "ArrowRight は claim され prevent_default() が呼ばれること"
    );

    assert_eq!(
        runtime.component().splitter.size(0),
        Some(51.0),
        "keydown dispatch が SplitterAction::Increment（trigger=0）で \
         状態を更新すること"
    );
    assert_eq!(
        trigger.get_attribute("aria-valuenow").as_deref(),
        Some("51"),
        "Runtime::wire_splitter が Self::wire の閉包を配線し、dispatch 後の \
         apply_update_for_dirty が Splitter 自身の aria-valuenow を \
         再描画すること（イシュー #1996 の受け入れ条件）"
    );
    assert_eq!(
        size_label(&root_el),
        "51",
        "Runtime::wire_splitter が dispatch 後の dirty_fields() を \
         apply_update_for_dirty へ渡し、C 側の束縛点（size_label）が \
         再描画されること（イシュー #1996 の受け入れ条件）"
    );

    // ArrowLeft（Decrement）も同じ経路で対称に反映されることを確認する。
    trigger.dispatch_event(&keydown_event("ArrowLeft")).unwrap();
    assert_eq!(runtime.component().splitter.size(0), Some(50.0));
    assert_eq!(
        trigger.get_attribute("aria-valuenow").as_deref(),
        Some("50")
    );
    assert_eq!(size_label(&root_el), "50");
}

/// `Runtime::mount`（CSR 経路）で配線した場合も
/// [`hydrate_arrow_right_keydown_dispatches_and_rerenders_dom`] と同じ反映が
/// 起きることを検証する（イシュー #1996 の指摘「`mount`/`hydrate` 両経路を
/// カバーすべき」対応。`Runtime::mount`/`Runtime::hydrate` の双方が今回
/// `Self::wire_splitter` の呼び出し引数を変更しているため、片方のみの
/// カバレッジでは他方の配線ミス（例: `binding_table`/`keyed_list_cache` の
/// 取り違え）を検知できない）。
#[wasm_bindgen_test]
fn mount_arrow_right_keydown_dispatches_and_rerenders_dom() {
    let (root_el, runtime) = mount_via_csr(SplitterHost::new("splitter-host-mount-root"));
    let _cleanup = RemoveOnDrop(root_el.clone());

    let trigger = resize_trigger(&root_el);
    assert_eq!(
        trigger.get_attribute("aria-valuenow").as_deref(),
        Some("50")
    );

    trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();

    assert_eq!(runtime.component().splitter.size(0), Some(51.0));
    assert_eq!(
        trigger.get_attribute("aria-valuenow").as_deref(),
        Some("51"),
        "Runtime::mount 経由の Runtime::wire_splitter も dispatch 後に \
         aria-valuenow を再描画すること（イシュー #1996、CSR 経路の \
         カバレッジ）"
    );
    assert_eq!(size_label(&root_el), "51");
}

/// disabled な resize-trigger への keydown が引き続き no-op であること
/// （`Self::wire` の閉包配線後も `wiring::handle_keydown` の disabled 除外が
/// 保たれていること）を固定する。
#[wasm_bindgen_test]
fn disabled_resize_trigger_keydown_is_noop() {
    let (root_el, runtime) =
        mount_via_hydrate(SplitterHost::disabled("splitter-host-disabled-root"));
    let _cleanup = RemoveOnDrop(root_el.clone());

    let trigger = resize_trigger(&root_el);
    assert_eq!(
        trigger.get_attribute("aria-valuenow").as_deref(),
        Some("50")
    );

    trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();

    assert_eq!(
        runtime.component().splitter.size(0),
        Some(50.0),
        "disabled な resize-trigger への keydown は状態を変えないこと"
    );
    assert_eq!(
        trigger.get_attribute("aria-valuenow").as_deref(),
        Some("50")
    );
    assert_eq!(size_label(&root_el), "50");
}

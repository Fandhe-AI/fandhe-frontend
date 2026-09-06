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

use fandhe_frontend_core::{bind_attr_tokens, bind_text, el, render, Node};
use fandhe_frontend_headless_ui::splitter::{PanelSpec, Splitter, SplitterAction};
use fandhe_frontend_headless_ui::Orientation;
use fandhe_frontend_interactive::{Component, DirtyTracked, Hydrate, HydrateError};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, HtmlElement, KeyboardEvent, KeyboardEventInit};

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

/// [`SplitterHost::with_structural_fallback`] が `dirty_fields()` へ積む
/// 「束縛点にも keyed list にも対応しない」フィールド名
/// （`angle_slider_browser.rs::STRUCTURAL_ONLY_FIELD` と同型）。
///
/// `Runtime::apply_update_for_dirty` はこの field を `BindingTable::has_field`
/// でも `find_list_element` でも解決できないため `unresolved_field` を立て、
/// `Runtime::rerender_subtree`（`root` 配下の丸ごと差し替え）へフォール
/// バックする。[`resize_trigger_focus_is_restored_after_structural_fallback_on_keydown`]
/// が resize-trigger detach を誘発するのに使う（イシュー #1996
/// codex-review P1 是正の回帰テスト）。
const STRUCTURAL_ONLY_FIELD: &str = "structural_only";

/// 2 パネル・1 リサイズトリガーの最小 Splitter を包むテストホスト。
/// トリガー 0 の先行パネル（`panel-0`）サイズを `size_now` として
/// `aria-valuenow` へ束縛する（モジュール冒頭 doc「束縛点設計」節参照）。
/// `structural_fallback` を立てたホストは加えて [`STRUCTURAL_ONLY_FIELD`]
/// を積み、`Runtime::rerender_subtree` による構造フォールバックを毎
/// dispatch で誘発する（`angle_slider_browser.rs::AngleSliderHost` と
/// 同型の設計）。
struct SplitterHost {
    splitter: Splitter,
    root_id: String,
    disabled: bool,
    structural_fallback: bool,
    /// `true` の場合 Splitter Root/resize-trigger のいずれにも `id` を
    /// 付けない「標準構成」で描画する（headless-ui `splitter::root`/
    /// `resize_trigger` はいずれも `id` を必須付与しない、モジュール冒頭
    /// doc 参照）。`view()` はこの場合 [`SplitterHost::root_id`] を
    /// Runtime のマウント先（本テストの `id` を持つ最外殻ラッパー div）
    /// にのみ使い、Splitter Root 自身には `id` を渡さない
    /// （`splitter::wiring::TriggerKey` の `data-id` フォールバック経路
    /// の回帰テスト、イシュー #1996 codex-review P1 是正）。
    omit_splitter_ids: bool,
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
/// ホストの `structural_fallback` を往復させる属性名。
const HOST_ATTR_STRUCTURAL: &str = "data-hydrate-host-structural";
/// ホストの `omit_splitter_ids` を往復させる属性名（イシュー #1996
/// codex-review P1 是正の回帰テスト用。[`SplitterHost::omit_splitter_ids`]
/// doc 参照）。`Runtime::hydrate` は復元成功時に渡された `component` では
/// なく `from_hydration_attrs` の再構築結果を使うため、この往復がないと
/// 構造フォールバック後の再描画で `id` が復活してしまい「標準構成」の
/// 検証にならない。
const HOST_ATTR_OMIT_IDS: &str = "data-hydrate-host-omit-splitter-ids";

impl SplitterHost {
    fn new(root_id: &str) -> Self {
        Self::with_options(root_id, false, false, false)
    }

    fn disabled(root_id: &str) -> Self {
        Self::with_options(root_id, true, false, false)
    }

    /// 毎 dispatch で [`STRUCTURAL_ONLY_FIELD`] を積み、
    /// `Runtime::apply_update_for_dirty` の構造フォールバック
    /// （`Runtime::rerender_subtree`）を必ず通すホスト。
    fn with_structural_fallback(root_id: &str) -> Self {
        Self::with_options(root_id, false, true, false)
    }

    /// [`Self::with_structural_fallback`] に加え、Splitter Root/
    /// resize-trigger のいずれにも `id` を付けない「標準構成」で描画する
    /// ホスト（[`SplitterHost::omit_splitter_ids`] doc 参照）。
    fn with_structural_fallback_without_ids(root_id: &str) -> Self {
        Self::with_options(root_id, false, true, true)
    }

    fn with_options(
        root_id: &str,
        disabled: bool,
        structural_fallback: bool,
        omit_splitter_ids: bool,
    ) -> Self {
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
            structural_fallback,
            omit_splitter_ids,
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
        // 値が実際に変化した dispatch に限って構造フォールバック用の
        // 未解決 field を積む（no-op dispatch で再描画を誘発しない）。
        if self.structural_fallback && !self.dirty.is_empty() {
            self.dirty.push(STRUCTURAL_ONLY_FIELD);
        }
    }

    fn view(&self) -> Node {
        // `omit_splitter_ids` のとき Splitter Root へ `id` を渡さない
        // （headless-ui `splitter::root` は `id` を要求しない、
        // `SplitterHost::omit_splitter_ids` doc 参照）。この場合
        // `Runtime::mount`/`Runtime::hydrate` のマウント先には別途 `id` を
        // 持つ最外殻ラッパー div を用意する必要があるため、Splitter 全体を
        // その子として包む。
        let splitter_root_attrs: Vec<(&str, &str)> = if self.omit_splitter_ids {
            Vec::new()
        } else {
            vec![("id", self.root_id.as_str())]
        };
        let splitter_node = self.splitter.root(
            self.disabled,
            splitter_root_attrs,
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
        );
        if self.omit_splitter_ids {
            el(
                "div",
                vec![("id", self.root_id.as_str())],
                vec![splitter_node],
            )
        } else {
            splitter_node
        }
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
        attrs.push((
            HOST_ATTR_STRUCTURAL.to_string(),
            self.structural_fallback.to_string(),
        ));
        attrs.push((
            HOST_ATTR_OMIT_IDS.to_string(),
            self.omit_splitter_ids.to_string(),
        ));
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
        let parse_bool = |name: &str| -> Result<bool, HydrateError> {
            match find(name)? {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(HydrateError::InvalidValue {
                    attr: name.to_string(),
                    reason: "expected \"true\" or \"false\"".to_string(),
                }),
            }
        };
        let disabled = parse_bool(HOST_ATTR_DISABLED)?;
        let structural_fallback = parse_bool(HOST_ATTR_STRUCTURAL)?;
        let omit_splitter_ids = parse_bool(HOST_ATTR_OMIT_IDS)?;

        let size_now = format!("{}", splitter.size(0).unwrap_or(50.0));
        let size_label = size_now.clone();
        let trigger_bind_attr = bind_attr_tokens(&[("aria-valuenow", "size_now")]);
        Ok(Self {
            splitter,
            root_id,
            disabled,
            structural_fallback,
            omit_splitter_ids,
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

/// 構造フォールバック（`Runtime::rerender_subtree`）を挟んだ矢印キー
/// keydown が resize-trigger のフォーカスを再描画後の同じ resize-trigger
/// へ復元することを検証する（イシュー #1996 codex-review P1 是正の受け入れ
/// 条件。`angle_slider_browser.rs::thumb_focus_is_restored_after_structural_fallback_on_keydown`
/// と同型）。復元前は本テストが再現する構成（束縛点にも keyed list にも
/// 対応しない dirty field を積むアプリ）で `remove_child` により
/// フォーカス中の resize-trigger が削除され、以後の矢印キーが届かず
/// サイズ調整が 1 回で途切れる回帰があった。
#[wasm_bindgen_test]
fn resize_trigger_focus_is_restored_after_structural_fallback_on_keydown() {
    let (root_el, runtime) = mount_via_hydrate(SplitterHost::with_structural_fallback(
        "splitter-host-focus-root",
    ));
    let _cleanup = RemoveOnDrop(root_el.clone());

    let active_element = || document().active_element();

    let initial_trigger = resize_trigger(&root_el);
    initial_trigger
        .dyn_ref::<HtmlElement>()
        .expect("resize-trigger must be an HtmlElement")
        .focus()
        .expect("focus must not fail");
    assert_eq!(
        active_element().as_ref(),
        Some(&initial_trigger),
        "keydown 前は resize-trigger がフォーカスされていること（前提成立確認）"
    );

    initial_trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert_eq!(runtime.component().splitter.size(0), Some(51.0));

    let trigger_after = resize_trigger(&root_el);
    assert!(
        trigger_after != initial_trigger,
        "STRUCTURAL_ONLY_FIELD により Runtime::rerender_subtree が走り、\
         フォーカスしていた resize-trigger が差し替わっていること（本 \
         テストが前提とする状況の成立確認）"
    );
    assert_eq!(
        active_element().as_ref(),
        Some(&trigger_after),
        "splitter::wiring::restore_trigger_focus が再描画後の同じ \
         resize-trigger へフォーカスを復元すること（イシュー #1996 \
         codex-review P1 是正の受け入れ条件）"
    );

    // フォーカスが復元されていれば、2 回目の矢印キーも同じ resize-trigger
    // へ届き続けサイズ調整が途切れないこと。
    trigger_after
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert_eq!(
        runtime.component().splitter.size(0),
        Some(52.0),
        "フォーカス復元により 2 回目の ArrowRight も resize-trigger へ届き、\
         サイズ調整が継続すること"
    );
}

/// [`resize_trigger_focus_is_restored_after_structural_fallback_on_keydown`]
/// の「id なし標準構成」版（イシュー #1996 codex-review P1 是正の回帰
/// テスト）。headless-ui `splitter::root`/`resize_trigger` はいずれも
/// `id` を自動付与も必須付与もしないため、アプリが `id` を付けない標準的
/// な描画構成でも構造フォールバックをまたいだフォーカス復元・矢印キーの
/// 継続操作が成立する必要がある。是正前は `TriggerKey::RootIdAndIndex`
/// が Splitter Root の `id` を前提としており、`id` が一切無い本構成では
/// フォールバックが `None` を返して復元を断念し、矢印キー操作が構造
/// フォールバック 1 回で途切れていた。
#[wasm_bindgen_test]
fn resize_trigger_focus_is_restored_after_structural_fallback_without_ids() {
    let (root_el, runtime) = mount_via_hydrate(SplitterHost::with_structural_fallback_without_ids(
        "splitter-host-no-id-focus-root",
    ));
    let _cleanup = RemoveOnDrop(root_el.clone());

    let active_element = || document().active_element();

    let initial_trigger = resize_trigger(&root_el);
    assert!(
        initial_trigger.id().is_empty(),
        "resize-trigger 自身に id が付与されていないこと（本テストの前提）"
    );
    let splitter_root = initial_trigger
        .parent_element()
        .expect("resize-trigger must have a parent element (Splitter root)");
    assert!(
        splitter_root.id().is_empty(),
        "Splitter Root にも id が付与されていないこと（本テストの前提。\
         `root_el`〔ラッパー div〕にのみ id を持つ「標準構成」を再現する）"
    );

    initial_trigger
        .dyn_ref::<HtmlElement>()
        .expect("resize-trigger must be an HtmlElement")
        .focus()
        .expect("focus must not fail");
    assert_eq!(
        active_element().as_ref(),
        Some(&initial_trigger),
        "keydown 前は resize-trigger がフォーカスされていること（前提成立確認）"
    );

    initial_trigger
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert_eq!(runtime.component().splitter.size(0), Some(51.0));

    let trigger_after = resize_trigger(&root_el);
    assert!(
        trigger_after != initial_trigger,
        "STRUCTURAL_ONLY_FIELD により Runtime::rerender_subtree が走り、\
         フォーカスしていた resize-trigger が差し替わっていること（本 \
         テストが前提とする状況の成立確認）"
    );
    assert_eq!(
        active_element().as_ref(),
        Some(&trigger_after),
        "id を一切持たない標準構成でも、resize-trigger の data-id \
         （隣接パネル id の組）により splitter::wiring::restore_trigger_focus \
         が再描画後の同じ resize-trigger へフォーカスを復元すること \
         （イシュー #1996 codex-review P1 是正の受け入れ条件）"
    );

    // フォーカスが復元されていれば、2 回目の矢印キーも同じ resize-trigger
    // へ届き続けサイズ調整が途切れないこと（id 必須化が無くとも継続操作が
    // 成立することの直接証明）。
    trigger_after
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert_eq!(
        runtime.component().splitter.size(0),
        Some(52.0),
        "id なし標準構成でもフォーカス復元により 2 回目の ArrowRight が \
         resize-trigger へ届き、サイズ調整が継続すること"
    );
}

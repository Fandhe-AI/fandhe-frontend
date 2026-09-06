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
//! Splitter への keydown が
//!
//! 1. ArrowRight（Increment）、hydrate 経路
//!    （[`hydrate_arrow_right_keydown_dispatches_and_rerenders_dom`]）
//! 2. ArrowRight（Increment）、mount（CSR）経路
//!    （[`mount_arrow_right_keydown_dispatches_and_rerenders_dom`]）
//! 3. disabled resize-trigger への keydown が no-op のままであること
//!    （[`disabled_resize_trigger_keydown_is_noop`]）
//! 4. 構造フォールバックを挟んだフォーカス復元（3 変種、イシュー #1996
//!    codex-review P1 是正の回帰。
//!    [`resize_trigger_focus_is_restored_after_structural_fallback_on_keydown`]/
//!    [`resize_trigger_focus_is_restored_after_structural_fallback_without_ids`]/
//!    [`resize_trigger_focus_is_restored_after_structural_fallback_with_colon_panel_ids`]）
//! 5. Home/End（SetToMin/SetToMax）が先行パネルを min/max へ設定すること
//!    （イシュー #1997、
//!    [`home_and_end_keydown_set_min_and_max_and_rerender_dom`]）
//! 6. `Runtime::hydrate` の通常復元経路（`from_hydration_attrs` が `Ok` を
//!    返すケース）で min/max が属性由来に復元され、DOM 同一性が保たれる
//!    こと（イシュー #1997、
//!    [`hydrate_restores_host_from_attrs_and_preserves_dom_identity`]）
//!
//! の各ケースで、resize-trigger 自身の `aria-valuenow`（`data-bind-attr`
//! 束縛点、モジュール冒頭 doc「束縛点設計」節参照）・`C`
//! （[`SplitterHost`]）側の派生フィールド（`data-bind-text` 束縛点）・
//! panel-0 の `style`（パネルサイズ、イシュー #1997）が正しく反映される
//! ことを検証する。
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
//!
//! 同様に [`fandhe_frontend_headless_ui::splitter::panel`] は `id`/
//! `data-orientation`/`data-index`/`data-id` のみを出力し、パネルサイズの
//! DOM 表現（`style`）を一切持たない（`crates/wasm-full/src/splitter.rs`
//! モジュール doc「`aria-valuenow` を直接書き換えない設計判断」節が
//! 明記するとおり、`Runtime::wire_splitter` はこの反映を保証しない
//! **アプリ側配線**である）。本ファイルは
//! `fandhe-frontend-pre-styled-ui` クレートへ依存せず、同クレートの
//! `splitter::panel`（`percent_style`）と同じ書式
//! `--fandhe-splitter-size: {percent}%` の文字列を
//! [`SplitterHost::panel0_style`] として保持し、
//! `bind_attr_tokens(&[("style", "panel0_style")])` を panel-0 呼び出し
//! 側 `attrs` へ付与することで、アプリが束縛した場合にこの契約が実際に
//! 成立することを示す（`fandhe_frontend_wasm_client::binding_dom::apply_one`
//! は `style` を通常の `set_attribute` で反映し、URL 検証・イベント
//! ハンドラ属性拒否の対象外であることは調査済み）。

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
    /// （`splitter::wiring::TriggerKey` の `aria-controls` フォールバック
    /// 経路の回帰テスト、イシュー #1996 codex-review P1 是正）。
    omit_splitter_ids: bool,
    /// トリガー 0 が隣接する 2 パネルの `id`（既定 `("panel-0",
    /// "panel-1")`）。codex-review P1 是正（イシュー #1996）の回帰テスト
    /// （[`Self::with_structural_fallback_and_colon_panel_ids`]）でのみ
    /// コロンを含む値へ差し替える。`splitter::wiring::TriggerKey::Controls`
    /// は `aria-controls`（空白区切り）で照合するため、パネル `id` 自体に
    /// コロンを含めても構造フォールバック後の再解決が曖昧にならない
    /// ことを検証する。
    panel_ids: (String, String),
    trigger_bind_attr: String,
    /// panel-0（`panel_ids.0`）の `style` 属性を束縛する
    /// `data-bind-attr` トークン（イシュー #1997。モジュール冒頭 doc
    /// 「束縛点設計」節参照）。`aria-valuenow` とは独立に、パネルサイズの
    /// DOM 反映（`fandhe-frontend-pre-styled-ui` の CSS カスタム
    /// プロパティと同じ書式 `--fandhe-splitter-size: {percent}%`）を
    /// アプリ側配線として検証するために [`SplitterHost::view`] が
    /// panel-0 の `attrs` へ渡す。
    panel_bind_attr: String,
    size_now: String,
    size_label: String,
    /// panel-0 の `style` 属性値（[`Self::panel_bind_attr`] が束縛する
    /// `panel0_style` フィールドの実体）。`size_now`/`size_label` と同じく
    /// [`SplitterHost::update`] がトリガー 0 の先行パネルサイズ変化を
    /// 検知するたびに再計算し `dirty` へ積む。
    panel0_style: String,
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
/// ホストの `panel_ids.0`（leading パネル `id`）を往復させる属性名
/// （イシュー #1996 codex-review P1 是正の回帰テスト用。
/// [`SplitterHost::panel_ids`] doc 参照）。
const HOST_ATTR_PANEL_ID_LEADING: &str = "data-hydrate-host-panel-id-leading";
/// ホストの `panel_ids.1`（trailing パネル `id`）を往復させる属性名。
const HOST_ATTR_PANEL_ID_TRAILING: &str = "data-hydrate-host-panel-id-trailing";

/// panel-0 の `style` 属性値の書式（`fandhe-frontend-pre-styled-ui` の
/// `splitter::panel`（`percent_style`）と同一書式、モジュール冒頭 doc
/// 「束縛点設計」節参照）。
fn panel0_style_value(size_now: &str) -> String {
    format!("--fandhe-splitter-size: {size_now}%")
}

impl SplitterHost {
    fn new(root_id: &str) -> Self {
        Self::with_options(root_id, false, false, false)
    }

    fn disabled(root_id: &str) -> Self {
        Self::with_options(root_id, true, false, false)
    }

    /// 指定した `Splitter`（min/max を既定 `(0.0, 100.0)` から変えたい
    /// ケース向け）で標準構成（disabled なし・構造フォールバックなし・
    /// id あり）のホストを作る（イシュー #1997。Home/End が min/max へ
    /// 実際に到達することを検証するテスト（[`home_and_end_keydown_set_min_and_max_and_rerender_dom`]）や、
    /// ハイドレーション属性由来の min/max 復元を検証するテスト
    /// （[`hydrate_restores_host_from_attrs_and_preserves_dom_identity`]）で使う）。
    fn with_splitter(root_id: &str, splitter: Splitter) -> Self {
        Self::new_with(root_id, splitter, false, false, false)
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

    /// [`Self::with_structural_fallback`] に加え、隣接 2 パネルの `id` に
    /// コロンを含む値（`"panel:a"`/`"panel:b"`）を使う（イシュー #1996
    /// codex-review P1 是正の回帰テスト。[`SplitterHost::panel_ids`] doc
    /// 参照）。是正前の `data-id="<leading_id>:<trailing_id>"`
    /// （コロン結合）方式では、パネル `id` 自体にコロンを含められるため
    /// 一意性が保証されなかった問題を検証する。
    fn with_structural_fallback_and_colon_panel_ids(root_id: &str) -> Self {
        let mut host = Self::with_options(root_id, false, true, false);
        host.panel_ids = ("panel:a".to_string(), "panel:b".to_string());
        host
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
        Self::new_with(
            root_id,
            splitter,
            disabled,
            structural_fallback,
            omit_splitter_ids,
        )
    }

    /// 全コンストラクタが収束する共通実装（[`Self::with_options`]/
    /// [`Self::with_splitter`] から呼ばれる）。`panel_ids` は既定
    /// `("panel-0", "panel-1")` のまま返し、コロン付き id への差し替えは
    /// 呼び出し側（[`Self::with_structural_fallback_and_colon_panel_ids`]）
    /// が行う。
    fn new_with(
        root_id: &str,
        splitter: Splitter,
        disabled: bool,
        structural_fallback: bool,
        omit_splitter_ids: bool,
    ) -> Self {
        let size_now = format!("{}", splitter.size(0).unwrap_or(50.0));
        let size_label = size_now.clone();
        let trigger_bind_attr = bind_attr_tokens(&[("aria-valuenow", "size_now")]);
        let panel_bind_attr = bind_attr_tokens(&[("style", "panel0_style")]);
        let panel0_style = panel0_style_value(&size_now);
        Self {
            splitter,
            root_id: root_id.to_string(),
            disabled,
            structural_fallback,
            omit_splitter_ids,
            panel_ids: ("panel-0".to_string(), "panel-1".to_string()),
            trigger_bind_attr,
            panel_bind_attr,
            size_now,
            size_label,
            panel0_style,
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
            self.size_label = now.clone();
            self.dirty.push("size_label");
        }
        let new_panel0_style = panel0_style_value(&now);
        if new_panel0_style != self.panel0_style {
            self.panel0_style = new_panel0_style;
            self.dirty.push("panel0_style");
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
                self.splitter.panel(
                    0,
                    self.panel_ids.0.as_str(),
                    vec![
                        // headless-ui `splitter::panel` はパネルサイズの DOM
                        // 表現（`style`）を一切持たない（モジュール冒頭 doc
                        // 「束縛点設計」節参照）。`aria-valuenow`（headless-ui
                        // 側が SSR 値を静的に出力する）とは異なり、`style`
                        // 自体の初期値もアプリ側（本テストホスト）が明示的に
                        // 出力する必要がある。`data-bind-attr` は以後の
                        // dispatch 後更新のための束縛点マーカーに過ぎない。
                        ("style", self.panel0_style.as_str()),
                        ("data-bind-attr", self.panel_bind_attr.as_str()),
                    ],
                    Vec::new(),
                ),
                self.splitter.resize_trigger(
                    0,
                    self.panel_ids.0.as_str(),
                    self.panel_ids.1.as_str(),
                    self.disabled,
                    vec![("data-bind-attr", self.trigger_bind_attr.as_str())],
                    Vec::new(),
                ),
                self.splitter
                    .panel(1, self.panel_ids.1.as_str(), Vec::new(), Vec::new()),
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
            "panel0_style" => Some(BoundValue::Text(self.panel0_style.clone())),
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
        attrs.push((
            HOST_ATTR_PANEL_ID_LEADING.to_string(),
            self.panel_ids.0.clone(),
        ));
        attrs.push((
            HOST_ATTR_PANEL_ID_TRAILING.to_string(),
            self.panel_ids.1.clone(),
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
        let panel_id_leading = find(HOST_ATTR_PANEL_ID_LEADING)?.to_string();
        let panel_id_trailing = find(HOST_ATTR_PANEL_ID_TRAILING)?.to_string();

        let size_now = format!("{}", splitter.size(0).unwrap_or(50.0));
        let size_label = size_now.clone();
        let trigger_bind_attr = bind_attr_tokens(&[("aria-valuenow", "size_now")]);
        let panel_bind_attr = bind_attr_tokens(&[("style", "panel0_style")]);
        let panel0_style = panel0_style_value(&size_now);
        Ok(Self {
            splitter,
            root_id,
            disabled,
            structural_fallback,
            omit_splitter_ids,
            panel_ids: (panel_id_leading, panel_id_trailing),
            trigger_bind_attr,
            panel_bind_attr,
            size_now,
            size_label,
            panel0_style,
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

/// panel-0（`data-index='0'` の `[data-scope='splitter'][data-part='panel']`）
/// の `style` 属性値（イシュー #1997。モジュール冒頭 doc「束縛点設計」節
/// 参照）。束縛点が未反映（`style` 属性自体が無い）場合は `None` を返す。
fn panel0_style_attr(root_el: &Element) -> Option<String> {
    root_el
        .query_selector("[data-scope='splitter'][data-part='panel'][data-index='0']")
        .expect("query_selector must not fail")
        .expect("panel-0 part must exist")
        .get_attribute("style")
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
    assert_eq!(
        panel0_style_attr(&root_el).as_deref(),
        Some("--fandhe-splitter-size: 50%"),
        "panel-0 の style 束縛点が初期サイズを反映していること"
    );

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
    assert_eq!(
        panel0_style_attr(&root_el).as_deref(),
        Some("--fandhe-splitter-size: 51%"),
        "panel-0 の style 束縛点（アプリ側配線）が dispatch 後の \
         パネルサイズ反映として再描画されること（イシュー #1997 の \
         受け入れ条件）"
    );

    // ArrowLeft（Decrement）も同じ経路で対称に反映されることを確認する。
    trigger.dispatch_event(&keydown_event("ArrowLeft")).unwrap();
    assert_eq!(runtime.component().splitter.size(0), Some(50.0));
    assert_eq!(
        trigger.get_attribute("aria-valuenow").as_deref(),
        Some("50")
    );
    assert_eq!(size_label(&root_el), "50");
    assert_eq!(
        panel0_style_attr(&root_el).as_deref(),
        Some("--fandhe-splitter-size: 50%")
    );
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
    assert_eq!(
        panel0_style_attr(&root_el).as_deref(),
        Some("--fandhe-splitter-size: 51%"),
        "Runtime::mount 経由でも panel-0 の style 束縛点（パネルサイズ \
         反映）が dispatch 後に再描画されること（イシュー #1997）"
    );
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
    assert_eq!(
        panel0_style_attr(&root_el).as_deref(),
        Some("--fandhe-splitter-size: 50%"),
        "disabled な resize-trigger への keydown はパネルサイズ（style \
         束縛点）も変えないこと（イシュー #1997）"
    );
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
        "id を一切持たない標準構成でも、resize-trigger の aria-controls \
         （隣接パネル id の空白区切りペア）により \
         splitter::wiring::restore_trigger_focus \
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

/// [`resize_trigger_focus_is_restored_after_structural_fallback_on_keydown`]
/// の「パネル `id` にコロンを含む」版（イシュー #1996 codex-review P1
/// 是正の回帰テスト）。是正前は resize-trigger の再解決キーを
/// `data-id="<leading_id>:<trailing_id>"`（コロン結合）で組み立てていた
/// ため、パネル `id` 自体にコロンを含められる構成では「異なるパネル
/// ペアが同じ結合文字列に一致する」曖昧性を構造的に排除できなかった
/// （`splitter::wiring::TriggerKey` doc 参照）。是正後は `aria-controls`
/// （空白区切り、HTML `id` 属性値は空白を含められない仕様のため区切りが
/// 曖昧にならない）で照合するため、パネル `id` にコロンを含めても
/// フォーカス復元・矢印キー操作の継続が成立する必要がある。
#[wasm_bindgen_test]
fn resize_trigger_focus_is_restored_after_structural_fallback_with_colon_panel_ids() {
    let (root_el, runtime) =
        mount_via_hydrate(SplitterHost::with_structural_fallback_and_colon_panel_ids(
            "splitter-host-colon-panel-id-focus-root",
        ));
    let _cleanup = RemoveOnDrop(root_el.clone());

    let active_element = || document().active_element();

    let initial_trigger = resize_trigger(&root_el);
    assert_eq!(
        initial_trigger.get_attribute("aria-controls").as_deref(),
        Some("panel:a panel:b"),
        "resize-trigger の aria-controls がコロンを含むパネル id を \
         空白区切りでそのまま保持していること（本テストの前提成立確認）"
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
        "パネル id にコロンを含む構成でも、aria-controls による照合で \
         splitter::wiring::restore_trigger_focus が再描画後の同じ \
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
        "パネル id にコロンを含む構成でもフォーカス復元により 2 回目の \
         ArrowRight が resize-trigger へ届き、サイズ調整が継続すること"
    );
}

/// Home/End keydown が `SplitterKeyAction::SetToMin`/`SetToMax`
/// （`fandhe_frontend_wasm_full::keynav::splitter_key_action`、dispatch
/// アクション名 `"home"`/`"end"`）へ写像され、resize-trigger 0 の先行
/// パネルがその `min`/`max` へ設定されたうえで `aria-valuenow`・
/// `size_label`・panel-0 の `style`（パネルサイズ、イシュー #1997）の
/// 全てへ反映されることを検証する（イシュー #1997 の受け入れ条件（2））。
/// ArrowRight/ArrowLeft のみを検証する
/// [`hydrate_arrow_right_keydown_dispatches_and_rerenders_dom`] は
/// `SplitterKeyAction::Increment`/`Decrement` のみを通り、`SetToMin`/
/// `SetToMax` 経路の dispatch 配線（`ACTION_HOME`/`ACTION_END`）は
/// カバーしない。
///
/// `min`/`max` を既定 `(0.0, 100.0)` から絞った `PanelSpec::new(50.0,
/// 20.0, 80.0)` を両パネルへ使い、「min へ clamp された」ことと「0 へ
/// 落ちた」ことを区別できるようにする。
#[wasm_bindgen_test]
fn home_and_end_keydown_set_min_and_max_and_rerender_dom() {
    let splitter = Splitter::new(
        &[
            PanelSpec::new(50.0, 20.0, 80.0),
            PanelSpec::new(50.0, 20.0, 80.0),
        ],
        Orientation::Horizontal,
    );
    let (root_el, runtime) = mount_via_hydrate(SplitterHost::with_splitter(
        "splitter-host-home-end-root",
        splitter,
    ));
    let _cleanup = RemoveOnDrop(root_el.clone());

    let trigger = resize_trigger(&root_el);
    assert_eq!(
        trigger.get_attribute("aria-valuenow").as_deref(),
        Some("50")
    );

    let default_not_prevented = trigger.dispatch_event(&keydown_event("Home")).unwrap();
    assert!(
        !default_not_prevented,
        "Home は claim され prevent_default() が呼ばれること"
    );
    assert_eq!(
        runtime.component().splitter.size(0),
        Some(20.0),
        "Home keydown が SplitterAction::SetToMin（trigger=0）で \
         先行パネルをその min（20.0）へ設定すること"
    );
    assert_eq!(
        trigger.get_attribute("aria-valuenow").as_deref(),
        Some("20"),
        "Home 後に aria-valuenow が min へ反映されること"
    );
    assert_eq!(size_label(&root_el), "20");
    assert_eq!(
        panel0_style_attr(&root_el).as_deref(),
        Some("--fandhe-splitter-size: 20%"),
        "Home 後にパネルサイズ（style 束縛点）が min へ反映されること \
         （イシュー #1997 の受け入れ条件）"
    );

    let default_not_prevented = trigger.dispatch_event(&keydown_event("End")).unwrap();
    assert!(
        !default_not_prevented,
        "End は claim され prevent_default() が呼ばれること"
    );
    assert_eq!(
        runtime.component().splitter.size(0),
        Some(80.0),
        "End keydown が SplitterAction::SetToMax（trigger=0）で \
         先行パネルをその max（80.0）へ設定すること"
    );
    assert_eq!(
        trigger.get_attribute("aria-valuenow").as_deref(),
        Some("80"),
        "End 後に aria-valuenow が max へ反映されること"
    );
    assert_eq!(size_label(&root_el), "80");
    assert_eq!(
        panel0_style_attr(&root_el).as_deref(),
        Some("--fandhe-splitter-size: 80%"),
        "End 後にパネルサイズ（style 束縛点）が max へ反映されること \
         （イシュー #1997 の受け入れ条件）"
    );
}

/// `Runtime::hydrate` の**通常の復元経路**（`Hydrate::from_hydration_attrs`
/// が `Ok` を返すケース）を検証する（イシュー #1997 の受け入れ条件（4）、
/// `angle_slider_browser.rs::hydrate_restores_host_from_attrs_and_preserves_dom_identity`
/// と同型）。
///
/// SSR 相当の出力（sizes 60/40、mins 10、maxs 90）を DOM へ入れ、
/// `hydration_attrs()`（`Splitter::hydration_attrs` が含む
/// `data-hydrate-sizes`/`-mins`/`-maxs` を含む）を付与したうえで、
/// あえて異なる初期状態（既定 50/50、min/max 0..100）を持つホストで
/// `Runtime::hydrate` を呼ぶ。
///
/// 1. `splitter.size(0)`/`min(0)`/`max(0)` が属性由来（60/10/90）で
///    復元され、引数のホストの値（50/0/100）で上書きされないこと
/// 2. 復元成功時は SSR 出力の DOM が維持され、hydrate 前後で
///    resize-trigger・panel-0 の DOM 同一性（`Node::is_same_node`）が
///    保たれること
/// 3. 復元後のホストへの Home/End keydown が属性由来の min/max（10/90）
///    へ反映されること（引数のホストの 0/100 ではないこと＝min/max が
///    属性由来であることの直接証明）
///
/// の 3 点を確認する。
#[wasm_bindgen_test]
fn hydrate_restores_host_from_attrs_and_preserves_dom_identity() {
    const ROOT_ID: &str = "splitter-host-restore-identity-root";

    let document = document();
    let ssr_splitter = Splitter::new(
        &[
            PanelSpec::new(60.0, 10.0, 90.0),
            PanelSpec::new(40.0, 10.0, 90.0),
        ],
        Orientation::Horizontal,
    );
    let ssr_host = SplitterHost::with_splitter(ROOT_ID, ssr_splitter);
    let html = render(&ssr_host.view());
    document
        .body()
        .expect("document body must exist in browser test environment")
        .insert_adjacent_html("beforeend", &html)
        .expect("insert_adjacent_html must not fail");
    let root_el = document
        .get_element_by_id(ROOT_ID)
        .expect("rendered Splitter root must have the expected id");
    let _cleanup = RemoveOnDrop(root_el.clone());
    for (name, value) in ssr_host.hydration_attrs() {
        root_el
            .set_attribute(&name, &value)
            .expect("set_attribute must not fail");
    }

    let trigger_before = resize_trigger(&root_el);
    assert_eq!(
        trigger_before.get_attribute("aria-valuenow").as_deref(),
        Some("60")
    );
    let panel0_before = root_el
        .query_selector("[data-scope='splitter'][data-part='panel'][data-index='0']")
        .expect("query_selector must not fail")
        .expect("panel-0 part must exist");
    assert_eq!(
        panel0_before.get_attribute("style").as_deref(),
        Some("--fandhe-splitter-size: 60%")
    );

    // 復元が属性由来であることを確かめるため、hydrate へ渡すホストには
    // あえて別の初期状態（既定 50/50、min/max 0..100）を持たせる。
    let default_splitter = Splitter::new(
        &[
            PanelSpec::new(50.0, 0.0, 100.0),
            PanelSpec::new(50.0, 0.0, 100.0),
        ],
        Orientation::Horizontal,
    );
    let runtime = Runtime::hydrate(
        ROOT_ID,
        SplitterHost::with_splitter(ROOT_ID, default_splitter),
    )
    .expect("hydrate must succeed for well-formed attrs");

    assert_eq!(
        runtime.component().splitter.size(0),
        Some(60.0),
        "from_hydration_attrs が data-hydrate-sizes 由来の 60.0 を \
         復元すること（引数のホストの初期値 50.0 で上書きされないこと）"
    );
    assert_eq!(
        runtime.component().splitter.min(0),
        Some(10.0),
        "from_hydration_attrs が data-hydrate-mins 由来の min=10.0 を \
         復元すること"
    );
    assert_eq!(
        runtime.component().splitter.max(0),
        Some(90.0),
        "from_hydration_attrs が data-hydrate-maxs 由来の max=90.0 を \
         復元すること"
    );

    let trigger_after = resize_trigger(&root_el);
    // `fandhe_frontend_core::Node` と名前が衝突するため完全修飾で書く。
    let trigger_before_node: &web_sys::Node = trigger_before.as_ref();
    assert!(
        trigger_after.is_same_node(Some(trigger_before_node)),
        "復元成功時は CSR 再描画へフォールバックせず、hydrate 前後で \
         resize-trigger 要素の DOM 同一性が維持されること"
    );
    let panel0_after = root_el
        .query_selector("[data-scope='splitter'][data-part='panel'][data-index='0']")
        .expect("query_selector must not fail")
        .expect("panel-0 part must exist");
    let panel0_before_node: &web_sys::Node = panel0_before.as_ref();
    assert!(
        panel0_after.is_same_node(Some(panel0_before_node)),
        "復元成功時は panel-0 要素の DOM 同一性も維持されること"
    );

    // 復元後の配線確認: Home/End が属性由来の min/max（10/90）へ反映される
    // こと（引数のホストの 0/100 ではないこと）。
    trigger_after
        .dispatch_event(&keydown_event("Home"))
        .unwrap();
    assert_eq!(runtime.component().splitter.size(0), Some(10.0));
    assert_eq!(
        trigger_after.get_attribute("aria-valuenow").as_deref(),
        Some("10"),
        "属性から復元した min=10 へ Home が設定すること（引数のホストの \
         min=0 ではないこと）"
    );
    assert_eq!(size_label(&root_el), "10");
    assert_eq!(
        panel0_style_attr(&root_el).as_deref(),
        Some("--fandhe-splitter-size: 10%")
    );

    trigger_after.dispatch_event(&keydown_event("End")).unwrap();
    assert_eq!(runtime.component().splitter.size(0), Some(90.0));
    assert_eq!(
        trigger_after.get_attribute("aria-valuenow").as_deref(),
        Some("90"),
        "属性から復元した max=90 へ End が設定すること（引数のホストの \
         max=100 ではないこと）"
    );
    assert_eq!(size_label(&root_el), "90");
    assert_eq!(
        panel0_style_attr(&root_el).as_deref(),
        Some("--fandhe-splitter-size: 90%")
    );
}

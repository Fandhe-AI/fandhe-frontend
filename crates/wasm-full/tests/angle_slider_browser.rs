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
//! 4. 構造フォールバックを挟んだポインタドラッグの継続
//!    （[`pointer_drag_continues_across_structural_fallback`]、
//!    イシュー #1956 codex-review P1 是正）
//! 5. ハイドレーションの通常復元経路と DOM 同一性の維持
//!    （[`hydrate_restores_host_from_attrs_and_preserves_dom_identity`]、
//!    イシュー #1956 codex-review P2 是正）
//!
//! の 5 ケースで、AngleSlider 自身の `aria-valuenow`/`aria-valuetext`
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
//!
//! # ハイドレーション復元（イシュー #1956 codex-review P2 是正）
//!
//! `fandhe_frontend_wasm_full::Runtime::hydrate` は
//! `hydration::restore_state` → `C::from_hydration_attrs` を**通常の復元時
//! にも必ず**呼び、`Err` のときだけ引数のホストで CSR 再描画
//! （`dom::mount_initial`）へフォールバックする。旧版の
//! [`AngleSliderHost`] は `from_hydration_attrs` が常に `Err` を返して
//! いたため、本ファイルの全ケースが「SSR 出力の DOM を捨てて丸ごと
//! 作り直す」フォールバック経路を通っており、`Hydrate` 実装としても
//! 「復元失敗時のみこの経路を使う」という説明コメントが実装と逆だった
//! （レビュー指摘）。
//!
//! 是正: [`AngleSliderHost`] は `AngleSlider` の
//! `data-hydrate-value`/`data-hydrate-step` に加えて、ホスト自身の状態
//! （`root_id` / [`AngleSliderProps::disabled`] / `structural_fallback`）を
//! [`HOST_ATTR_ROOT_ID`]/[`HOST_ATTR_DISABLED`]/[`HOST_ATTR_STRUCTURAL`]
//! として `hydration_attrs()` へ出力し、`from_hydration_attrs` でそれらから
//! ホストを復元する（`data-hydrate-` プレフィックスを持つため
//! `hydration::filter_hydration_attrs` を通過する）。これにより本ファイルの
//! 全ケースが実運用と同じ「復元成功 → SSR DOM 維持」経路を通り、
//! [`hydrate_restores_host_from_attrs_and_preserves_dom_identity`] が
//! hydrate 前後の DOM 同一性を明示的に固定する。
//!
//! # 構造フォールバック中のドラッグ継続（イシュー #1956 codex-review P1
//! 是正）
//!
//! `Runtime::apply_update_for_dirty` は「束縛点にも keyed list にも
//! 対応しない dirty field」を検知すると `Runtime::rerender_subtree` で
//! `root` 配下を丸ごと差し替える。pointerdown の dispatch がこれを誘発
//! すると、直前に `setPointerCapture` を設定した Control 要素が detach され
//! ブラウザ側の capture も失われるため、`has_pointer_capture` のみで
//! 継続判定していた旧実装ではドラッグが最初の座標更新で止まっていた。
//! [`STRUCTURAL_ONLY_FIELD`] を積む
//! [`AngleSliderHost::with_structural_fallback`] がこの状況を最小構成で
//! 再現し、[`pointer_drag_continues_across_structural_fallback`] が是正
//! （`angle_slider.rs` の `wiring::DragState`）を回帰テストとして固定する。

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

/// 合成 pointer 系イベント（`bubbles: true, cancelable: true`）を、指定した
/// 種別・クライアント座標・`pointerId` で組み立てる
/// （`headless_signature_pad_browser.rs::new_pointer_event` を拡張し
/// `client_x`/`client_y` を追加。`fandhe_frontend_wasm_full::angle_slider::
/// handle_pointerdown`/`handle_pointermove` は `PointerEvent::client_x()`/
/// `client_y()` から `Control` の `getBoundingClientRect()` 中心相対の角度を
/// 計算するため座標指定が必須）。
///
/// 合成イベントには「アクティブな pointer」が存在しないため
/// `Element::set_pointer_capture` は `NotFoundError` で失敗する。配線側は
/// その失敗を無視してドラッグ追跡（`angle_slider.rs` の
/// `wiring::DragState`）で継続を判定するため、本ヘルパで組み立てた
/// pointerdown → pointermove の連続は実ブラウザのキャプチャ喪失時と同じ
/// 経路を通る（[`pointer_drag_continues_across_structural_fallback`] が
/// それを利用する）。
fn pointer_event(kind: &str, pointer_id: i32, client_x: f64, client_y: f64) -> Event {
    let init = PointerEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    init.set_pointer_id(pointer_id);
    init.set_client_x(client_x.round() as i32);
    init.set_client_y(client_y.round() as i32);
    PointerEvent::new_with_event_init_dict(kind, &init)
        .expect("PointerEvent::new must not fail")
        .dyn_into::<Event>()
        .expect("PointerEvent must cast to Event")
}

/// [`pointer_event`] の `pointerdown` 版（既存ケースの呼び出しを短く保つ）。
fn pointerdown_event(pointer_id: i32, client_x: f64, client_y: f64) -> Event {
    pointer_event("pointerdown", pointer_id, client_x, client_y)
}

/// AngleSlider Control パーツの CSS セレクタ。
const CONTROL_SELECTOR: &str = "[data-scope='angle-slider'][data-part='control']";
/// AngleSlider Thumb パーツの CSS セレクタ。
const THUMB_SELECTOR: &str = "[data-scope='angle-slider'][data-part='thumb']";

/// [`AngleSliderHost::with_structural_fallback`] が `dirty_fields()` へ積む
/// 「束縛点にも keyed list にも対応しない」フィールド名。
///
/// `Runtime::apply_update_for_dirty` はこの field を `BindingTable::has_field`
/// でも `find_list_element` でも解決できないため `unresolved_field` を立て、
/// `Runtime::rerender_subtree`（`root` 配下の丸ごと差し替え）へフォール
/// バックする。実アプリの「画面遷移のような構造変化」を最小構成で再現する
/// ための仕掛けであり、[`pointer_drag_continues_across_structural_fallback`]
/// がドラッグ中の Control detach を誘発するのに使う。
const STRUCTURAL_ONLY_FIELD: &str = "structural_only";

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
/// 使い回す）。`structural_fallback` を立てたホストは加えて
/// [`STRUCTURAL_ONLY_FIELD`] を積み、`Runtime::rerender_subtree` による
/// 構造フォールバックを毎 dispatch で誘発する。
///
/// `root_id`/`props`/`structural_fallback` は [`Hydrate::hydration_attrs`]
/// 側で `data-hydrate-host-*` 属性として往復させる（モジュール冒頭 doc
/// 「ハイドレーション復元」節参照）ため、`&'static str` ではなく所有
/// `String` で保持する。
struct AngleSliderHost {
    slider: AngleSlider,
    props: AngleSliderProps,
    root_id: String,
    structural_fallback: bool,
    thumb_bind_attr: String,
    angle_now: String,
    angle_text: String,
    angle_label: String,
    dirty: Vec<&'static str>,
}

/// ホスト自身（`AngleSlider` の外側）の状態を往復させる `data-hydrate-*`
/// 属性名（モジュール冒頭 doc「ハイドレーション復元」節参照）。
const HOST_ATTR_ROOT_ID: &str = "data-hydrate-host-root-id";
/// ホストの [`AngleSliderProps::disabled`] を往復させる属性名。
const HOST_ATTR_DISABLED: &str = "data-hydrate-host-disabled";
/// ホストの `structural_fallback` を往復させる属性名。
const HOST_ATTR_STRUCTURAL: &str = "data-hydrate-host-structural";

impl AngleSliderHost {
    fn new(root_id: &str, slider: AngleSlider) -> Self {
        Self::with_props(root_id, slider, AngleSliderProps::default(), false)
    }

    fn disabled(root_id: &str, slider: AngleSlider) -> Self {
        Self::with_props(
            root_id,
            slider,
            AngleSliderProps {
                disabled: true,
                readonly: false,
                invalid: false,
            },
            false,
        )
    }

    /// 毎 dispatch で [`STRUCTURAL_ONLY_FIELD`] を積み、
    /// `Runtime::apply_update_for_dirty` の構造フォールバック
    /// （`Runtime::rerender_subtree`）を必ず通すホスト。
    fn with_structural_fallback(root_id: &str, slider: AngleSlider) -> Self {
        Self::with_props(root_id, slider, AngleSliderProps::default(), true)
    }

    fn with_props(
        root_id: &str,
        slider: AngleSlider,
        props: AngleSliderProps,
        structural_fallback: bool,
    ) -> Self {
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
            root_id: root_id.to_string(),
            structural_fallback,
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
        // 値が実際に変化した dispatch に限って構造フォールバック用の
        // 未解決 field を積む（no-op dispatch で再描画を誘発しない）。
        if self.structural_fallback && !self.dirty.is_empty() {
            self.dirty.push(STRUCTURAL_ONLY_FIELD);
        }
    }

    fn view(&self) -> Node {
        self.slider.root(
            &self.props,
            vec![("id", self.root_id.as_str())],
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
    /// `AngleSlider` 自身の `data-hydrate-value`/`data-hydrate-step` に加え、
    /// ホスト側の `root_id`/`props.disabled`/`structural_fallback` を
    /// `data-hydrate-host-*` として出力する（モジュール冒頭 doc
    /// 「ハイドレーション復元」節参照）。
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let mut attrs = self.slider.hydration_attrs();
        attrs.push((HOST_ATTR_ROOT_ID.to_string(), self.root_id.clone()));
        attrs.push((
            HOST_ATTR_DISABLED.to_string(),
            self.props.disabled.to_string(),
        ));
        attrs.push((
            HOST_ATTR_STRUCTURAL.to_string(),
            self.structural_fallback.to_string(),
        ));
        attrs
    }

    /// `Runtime::hydrate` は**通常の復元時に必ず**この経路を通る
    /// （`fandhe_frontend_wasm_full::hydration::restore_state` →
    /// `C::from_hydration_attrs`）。`Err` を返すと `Runtime::hydrate` は
    /// 引数のホストで CSR 再描画（`dom::mount_initial`）へフォールバックし、
    /// SSR 出力の DOM が丸ごと捨てられるため、テストホストとして
    /// 「属性からホストを復元する」実装を持つ（イシュー #1956
    /// codex-review P2 是正。旧実装は常に `Err` を返しており、本ファイルの
    /// 全ケースが CSR 再描画経路を通っていた）。
    ///
    /// 改ざん・欠落した属性はクライアント入力として扱い、`panic!` せず
    /// [`HydrateError`] を返す（`AngleSlider::from_hydration_attrs` と同じ
    /// fail-closed 契約）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let slider = AngleSlider::from_hydration_attrs(attrs)?;

        let find = |name: &str| -> Result<&str, HydrateError> {
            attrs
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.as_str())
                .ok_or_else(|| HydrateError::MissingAttr(name.to_string()))
        };
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

        let root_id = find(HOST_ATTR_ROOT_ID)?.to_string();
        let disabled = parse_bool(HOST_ATTR_DISABLED)?;
        let structural_fallback = parse_bool(HOST_ATTR_STRUCTURAL)?;

        Ok(Self::with_props(
            &root_id,
            slider,
            AngleSliderProps {
                disabled,
                readonly: false,
                invalid: false,
            },
            structural_fallback,
        ))
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
    let root_id = host.root_id.clone();
    let html = render(&host.view());
    document
        .body()
        .expect("document body must exist in browser test environment")
        .insert_adjacent_html("beforeend", &html)
        .expect("insert_adjacent_html must not fail");
    let root_el = document
        .get_element_by_id(&root_id)
        .expect("rendered AngleSlider root must have the expected id");

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

/// pointerdown 中の dispatch が `Runtime::apply_update_for_dirty` の構造
/// フォールバック（`Runtime::rerender_subtree`）を誘発して Control 要素が
/// detach されても、続く pointermove が値を更新し続けることを検証する
/// （イシュー #1956 codex-review P1 是正の受け入れ条件）。
///
/// 修正前は `angle_slider.rs` の `handle_pointermove` が
/// `Element::has_pointer_capture` のみでドラッグ継続を判定していたため、
/// 差し替えられた新しい Control は capture を持たず全 pointermove が
/// 拒否され、ドラッグが最初の座標更新で止まっていた。是正後は配線側の
/// `wiring::DragState`（`pointerId` + `root` 配下 Control 群の添字）が
/// 継続を判定し、dispatch 後に新しい Control へ capture を掛け直す。
///
/// 座標は「真右（90 度）」「真左（270 度）」のみを使う。いずれも
/// `client_y` が Control 中心の `y` と同じ計算式（既存の
/// [`pointerdown_dispatches_and_rerenders_dom`] と同一）になるため、
/// `client_x`/`client_y` の `i32` 丸めに依存せず期待角度が決定的に定まる。
#[wasm_bindgen_test]
fn pointer_drag_continues_across_structural_fallback() {
    let (root_el, runtime) = mount(AngleSliderHost::with_structural_fallback(
        "angle-slider-host-drag-root",
        AngleSlider::new(10, 5),
    ));
    let _cleanup = RemoveOnDrop(root_el.clone());

    let control = || {
        root_el
            .query_selector(CONTROL_SELECTOR)
            .expect("query_selector must not fail")
            .expect("control part must exist")
    };
    // 「現在の Control 中心から右/左へ 50px」のクライアント座標を返す
    // （構造フォールバックで Control が差し替わるため毎回再取得する）。
    let offset_from_center = |control: &Element, dx: f64| -> (f64, f64) {
        let rect = control.get_bounding_client_rect();
        (
            rect.left() + rect.width() / 2.0 + dx,
            rect.top() + rect.height() / 2.0,
        )
    };

    let initial_control = control();
    let (down_x, down_y) = offset_from_center(&initial_control, 50.0);
    initial_control
        .dispatch_event(&pointer_event("pointerdown", 1, down_x, down_y))
        .unwrap();

    assert_eq!(
        runtime.component().slider.angle_deg(),
        90,
        "pointerdown（Control 中心の真右）が \"set\" action=90 を dispatch \
         すること"
    );
    let control_after_down = control();
    assert!(
        control_after_down != initial_control,
        "STRUCTURAL_ONLY_FIELD により Runtime::rerender_subtree が走り、\
         pointerdown で pointer capture を設定した Control 要素が detach \
         されていること（本テストが前提とする状況の成立確認）"
    );

    let (move_x, move_y) = offset_from_center(&control_after_down, -50.0);
    root_el
        .dispatch_event(&pointer_event("pointermove", 1, move_x, move_y))
        .unwrap();
    assert_eq!(
        runtime.component().slider.angle_deg(),
        270,
        "構造フォールバックで Control が差し替わり pointer capture を \
         失った後でも、pointermove（真左）がドラッグとして継続し 270 度へ \
         更新されること（イシュー #1956 codex-review P1 の受け入れ条件）"
    );

    let control_after_move = control();
    let (move_back_x, move_back_y) = offset_from_center(&control_after_move, 50.0);
    root_el
        .dispatch_event(&pointer_event("pointermove", 1, move_back_x, move_back_y))
        .unwrap();
    assert_eq!(
        runtime.component().slider.angle_deg(),
        90,
        "2 回目以降の pointermove も継続して値を更新すること（1 回限りの \
         復帰ではなくドラッグ全体が継続する）"
    );
    assert_eq!(
        control()
            .query_selector(THUMB_SELECTOR)
            .expect("query_selector must not fail")
            .expect("thumb part must exist")
            .get_attribute("aria-valuenow")
            .as_deref(),
        Some("90"),
        "再描画後の DOM でも束縛点（aria-valuenow）が最新値を反映すること"
    );

    // pointerup で追跡が解除され、以後の pointermove は no-op になること
    // （ドラッグ終了後に値が動き続ける暴走を防ぐ）。
    root_el
        .dispatch_event(&pointer_event("pointerup", 1, move_back_x, move_back_y))
        .unwrap();
    let control_after_up = control();
    let (stray_x, stray_y) = offset_from_center(&control_after_up, -50.0);
    root_el
        .dispatch_event(&pointer_event("pointermove", 1, stray_x, stray_y))
        .unwrap();
    assert_eq!(
        runtime.component().slider.angle_deg(),
        90,
        "pointerup 後は DragState が解除され、capture も持たない \
         pointermove が no-op になること"
    );
}

/// `Runtime::hydrate` の**通常の復元経路**（`Hydrate::from_hydration_attrs`
/// が `Ok` を返すケース）を検証する（イシュー #1956 codex-review P2 是正の
/// 受け入れ条件）。
///
/// `Runtime::hydrate` は復元失敗時だけでなく通常時も
/// `from_hydration_attrs` を呼ぶため、テストホストが常に `Err` を返して
/// いた旧実装では本ファイルの全ケースが CSR 再描画
/// （`dom::mount_initial`）へのフォールバックを通っていた
/// （モジュール冒頭 doc「ハイドレーション復元」節参照）。
///
/// 本ケースは
///
/// 1. `data-hydrate-*` 属性から `value`/`step`/ホスト状態が復元される
/// 2. 復元成功時は SSR 出力の DOM が維持され、hydrate 前後で Thumb 要素の
///    DOM 同一性（`Node::is_same_node`）が保たれる
/// 3. 復元後のホストへの keydown が属性由来の `step` で増分する
///
/// の 3 点を確認する。
#[wasm_bindgen_test]
fn hydrate_restores_host_from_attrs_and_preserves_dom_identity() {
    const ROOT_ID: &str = "angle-slider-host-hydrate-root";

    let document = document();
    // SSR 相当の出力（value=200, step=5）を DOM へ入れ、hydration 属性を付ける。
    let ssr_host = AngleSliderHost::new(ROOT_ID, AngleSlider::new(200, 5));
    let html = render(&ssr_host.view());
    document
        .body()
        .expect("document body must exist in browser test environment")
        .insert_adjacent_html("beforeend", &html)
        .expect("insert_adjacent_html must not fail");
    let root_el = document
        .get_element_by_id(ROOT_ID)
        .expect("rendered AngleSlider root must have the expected id");
    let _cleanup = RemoveOnDrop(root_el.clone());
    for (name, value) in ssr_host.hydration_attrs() {
        root_el
            .set_attribute(&name, &value)
            .expect("set_attribute must not fail");
    }

    let thumb_before = root_el
        .query_selector(THUMB_SELECTOR)
        .expect("query_selector must not fail")
        .expect("thumb part must exist");
    assert_eq!(
        thumb_before.get_attribute("aria-valuenow").as_deref(),
        Some("200"),
    );

    // 復元が属性由来であることを確かめるため、hydrate へ渡すホストには
    // あえて別の初期状態（value=0, step=1）を持たせる。
    let runtime = Runtime::hydrate(
        ROOT_ID,
        AngleSliderHost::new(ROOT_ID, AngleSlider::new(0, 1)),
    )
    .expect("hydrate must succeed for well-formed attrs");

    assert_eq!(
        runtime.component().slider.angle_deg(),
        200,
        "from_hydration_attrs が data-hydrate-value=200 を復元すること \
         （引数のホストの初期値 0 で上書きされないこと）"
    );
    assert_eq!(
        runtime.component().slider.step(),
        5,
        "from_hydration_attrs が data-hydrate-step=5 を復元すること"
    );

    let thumb_after = root_el
        .query_selector(THUMB_SELECTOR)
        .expect("query_selector must not fail")
        .expect("thumb part must exist");
    // `fandhe_frontend_core::Node` と名前が衝突するため完全修飾で書く。
    let thumb_before_node: &web_sys::Node = thumb_before.as_ref();
    assert!(
        thumb_after.is_same_node(Some(thumb_before_node)),
        "復元成功時は CSR 再描画へフォールバックせず、hydrate 前後で \
         Thumb 要素の DOM 同一性が維持されること（イシュー #1956 \
         codex-review P2 の受け入れ条件）"
    );

    let default_not_prevented = thumb_after
        .dispatch_event(&keydown_event("ArrowRight"))
        .unwrap();
    assert!(
        !default_not_prevented,
        "復元後のホストにも keydown 配線が効いていること"
    );
    assert_eq!(
        runtime.component().slider.angle_deg(),
        205,
        "属性から復元した step=5 で増分すること（引数のホストの step=1 で \
         はないこと）"
    );
    assert_eq!(
        thumb_after.get_attribute("aria-valuenow").as_deref(),
        Some("205"),
        "維持された DOM の束縛点が dispatch 後に更新されること"
    );
}

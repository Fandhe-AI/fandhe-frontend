//! `fandhe_frontend_wasm_full::Runtime::mount`/`hydrate` が SignaturePad
//! （イシュー #843、親 #735/#520）を実際に配線することを確認する実ブラウザ
//! 統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! # 検証動機（Bugbot 指摘の回帰固定）
//!
//! `crates/wasm-full/src/headless_signature_pad.rs`
//! （native、`StrokeCollector` 単体・`wire_signature_pad_component` の
//! ポインタ座標収集ロジック）は元々検証済みだったが、`Runtime::mount`/
//! `Runtime::hydrate`（`crates/wasm-full/src/lib.rs`）が
//! `wire_signature_pad_component`（または `wire_stroke_collector`）を
//! 一切呼び出していなかったため、標準の `Runtime` 経路を使うアプリでは
//! SignaturePad のポインタ/クリック配線が実行されない不具合があった
//! （PR #872 に対する Cursor Bugbot 指摘「Runtime omits signature pad
//! wiring」）。本ファイルは `Runtime::mount` 経由でこの配線が実際に成立する
//! ことを検証し、同種の「新規 headless コンポーネント追加時に `Self::mount`/
//! `Self::hydrate` への配線呼び出しを追加し忘れる」回帰を機械的に検知する
//! （`crates/wasm-full/tests/headless_avatar_browser.rs` 「(g)〜(i)
//! `Runtime::mount`/`hydrate` への統合」節と同じ検証層）。
//!
//! ポインタ座標収集（`wire_stroke_collector`）自体は `getBoundingClientRect`/
//! `viewBox`/`setPointerCapture` の実レイアウト依存が大きく、本ファイルでは
//! より決定的な ClearTrigger クリック経路（`crate::headless::wire_headless_component`
//! 経由、`MAPPING_TABLE` の `("signature-pad", "clear-trigger") -> "clear"`
//! 行）で「`Runtime::mount` が SignaturePad 配線を組み込んでいること」を
//! 確認する。`wire_signature_pad_component` はクリック配線・ポインタ配線の
//! 両方を 1 回のマウントで組み込む単一関数のため、クリック経路が動作して
//! いれば `Self::wire_signature_pad` が呼ばれていることの十分な証拠となる。

//!
//! # 構造フォールバック跨ぎと stale 解除（イシュー #1991/#1993/#1994）
//!
//! 上記の「クリック経路のみで配線成立を確認する」方針は、`Runtime::mount`
//! が SignaturePad の配線を組み込んでいることの検証には十分だが、
//! ポインタ経路自体（`wire_stroke_collector` の pointerdown/pointermove/
//! pointerup）が構造フォールバックや capture 喪失をまたいで正しく動作
//! することまでは固定しない。以下 2 ケースはその隙間を埋める:
//!
//! - [`structural_fallback::stroke_continues_across_structural_fallback_via_root_id_reattach`][]:
//!   ストローク中に `Runtime::rerender_subtree`（構造フォールバック）で
//!   Control 要素が detach され pointer capture が暗黙に失われても、
//!   `wiring::wire_stroke_collector` が追跡開始時に採取した SignaturePad
//!   Root の `id`（`wiring::capture_anchor_root_id`）から Control を
//!   再解決（`wiring::resolve_control_by_root_id`）して座標収集を継続し、
//!   `pointerup` で `add-stroke` が過不足なく 1 回だけ dispatch されることを
//!   検証する（イシュー #1993 の受け入れ条件）。
//! - [`stale_stroke_tracking_is_released_when_no_button_is_held`][]: capture
//!   喪失中に `root` 外で `pointerup`/`pointercancel` を取り逃し
//!   `active_pointer_id` が stale 化した追跡が、`buttons == 0` の
//!   `pointermove`（[`StrokeCollector::release_if_stale`]）で自己解除され、
//!   別 pointer id での新規ストローク開始が恒久ブロックされないことを
//!   検証する（イシュー #1992 の受け入れ条件）。
//!
//! いずれも `angle_slider_browser.rs` の
//! `pointer_drag_continues_across_structural_fallback`/
//! `stale_drag_tracking_is_released_when_no_button_is_held` と同型の
//! パターンを signature-pad 向けに最小化したものである。合成
//! `PointerEvent`（`wasm_bindgen_test`）では `has_pointer_capture` が常に
//! 偽・`set_pointer_capture` は `NotFoundError`（配線側で握りつぶし済み）
//! となるため、毎 `pointermove` で capture 再解決の分岐が通る
//! （実ブラウザのユーザー操作より再解決経路の露出頻度が高い、テストが
//! 意図的に利用する性質）。Root `id` が無い構成での fail-closed 挙動
//! （#1993 受け入れ条件の他方）は本ファイルの
//! `runtime_mount_wires_clear_trigger_click_for_signature_pad` 等、正準
//! ビュー（Root に `id` を持たない）を使う既存ケースが間接的に固定して
//! いる（専用ケースは持たない）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_headless_ui::signature_pad::{Point, SignaturePad, SignaturePadAction, Stroke};
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素を document body へ 1 個生成する。id を一意に
/// することで、同一テストバイナリ内の複数テストケースが要素を奪い合わない
/// ようにする（`runtime_browser.rs::create_placeholder` と同じ意図）。
fn create_placeholder(document: &Document, id: &str) -> Element {
    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(id);
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&container)
        .expect("append_child must not fail for a detached div");
    container
}

/// テスト末尾でプレースホルダを document から確実に除去する RAII ガード
/// （`runtime_browser.rs::RemoveOnDrop` と同じ、テスト間 DOM 汚染の再発防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// 合成 `click` イベントを生成する（`bubbles: true`）。`Runtime::mount` は
/// リスナーをルート要素へ登録するため、子要素上で発火したイベントが
/// バブリングでルートまで届く必要がある（`runtime_browser.rs::bubbling_event`
/// と同じ意図）。
fn bubbling_event(kind: &str) -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict(kind, &init).expect("Event::new must not fail")
}

/// `SignaturePad` を `Runtime<C>` へ載せるための最小ホスト
/// （`headless_avatar_browser.rs::TestAvatarHost` と同じ「dispatch/view を
/// そのまま委譲するだけの薄いラッパー」パターン）。`DirtyTracked` は
/// `SignaturePad::dirty_fields()` へそのまま委譲する（イシュー #843、Bugbot
/// 指摘「Runtime skips stroke DOM updates」の回帰固定に `add-stroke` 経由の
/// keyed list 差分適用を使うテストがあるため、空実装のままでは
/// `Runtime::wire_signature_pad` の dirty 駆動 DOM 更新経路自体が検証できない）。
struct TestSignaturePadHost(SignaturePad);

impl fandhe_frontend_interactive::Component for TestSignaturePadHost {
    type Action = SignaturePadAction;

    fn update(&mut self, action: Self::Action) {
        fandhe_frontend_interactive::Component::update(&mut self.0, action);
    }

    fn view(&self) -> fandhe_frontend_core::Node {
        fandhe_frontend_interactive::Component::view(&self.0)
    }

    fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
        SignaturePad::decode_action(name, payload)
    }
}

impl fandhe_frontend_interactive::DirtyTracked for TestSignaturePadHost {
    fn dirty_fields(&self) -> &[&'static str] {
        fandhe_frontend_interactive::DirtyTracked::dirty_fields(&self.0)
    }
}

impl fandhe_frontend_wasm_client::BindingSource for TestSignaturePadHost {
    fn bound_value(&self, _field: &str) -> Option<fandhe_frontend_wasm_client::BoundValue> {
        None
    }
}

impl fandhe_frontend_interactive::Hydrate for TestSignaturePadHost {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        fandhe_frontend_interactive::Hydrate::hydration_attrs(&self.0)
    }

    fn from_hydration_attrs(
        attrs: &[(String, String)],
    ) -> Result<Self, fandhe_frontend_interactive::HydrateError> {
        fandhe_frontend_interactive::Hydrate::from_hydration_attrs(attrs).map(Self)
    }
}

/// `Runtime::mount` 経由で配線した ClearTrigger の実クリックが SignaturePad
/// の全ストロークを削除すること（受け入れ条件、Bugbot 指摘の回帰固定）。
///
/// 修正前（`Self::mount`/`Self::hydrate` が `wire_signature_pad_component` を
/// 呼ばない状態）では、`wire_headless_component`（クリック配線）自体が
/// 登録されないため本テストは `strokes().len()` が 1 のまま失敗し、
/// 回帰を検知する。
#[wasm_bindgen_test]
fn runtime_mount_wires_clear_trigger_click_for_signature_pad() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "signature-pad-runtime-mount-clear-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let initial_stroke = Stroke::new(vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)])
        .expect("2 点のストロークは有効であること");
    let host = TestSignaturePadHost(SignaturePad::new(vec![initial_stroke], false, false));

    let runtime = Runtime::mount("signature-pad-runtime-mount-clear-root", host)
        .expect("Runtime::mount must not fail");

    assert_eq!(
        runtime.component().0.strokes().len(),
        1,
        "mount 直後は初期状態の 1 ストロークを保持していること"
    );

    let clear_trigger = placeholder
        .query_selector(r#"[data-scope="signature-pad"][data-part="clear-trigger"]"#)
        .expect("query_selector must not fail")
        .expect("mount 後に clear-trigger 要素が存在すること");
    clear_trigger
        .dispatch_event(&bubbling_event("click"))
        .expect("dispatch_event must not fail");

    assert!(
        runtime.component().0.is_empty(),
        "Runtime::mount が SignaturePad の ClearTrigger クリックを配線している場合、\
         クリック後は全ストロークが削除されていること \
         （イシュー #843 Bugbot 指摘「Runtime omits signature pad wiring」の回帰）"
    );
}

/// `Runtime::hydrate`（ハイドレーション属性が存在せず CSR フォールバックする
/// 経路）でも同じ配線が組み込まれること。`hydrate` は `mount` と別の
/// コードパス（`Self::wire_signature_pad` 呼び出し箇所が異なる）のため、
/// 片方だけ直したつもりで他方を直し忘れる回帰も検知する。
#[wasm_bindgen_test]
fn runtime_hydrate_csr_fallback_wires_clear_trigger_click_for_signature_pad() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "signature-pad-runtime-hydrate-clear-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    // `data-hydrate-*` 属性を一切持たないプレースホルダのため、
    // `Runtime::hydrate` は復元に失敗し初期状態での CSR 再描画へ
    // フォールバックする（`wasm-full/src/lib.rs::Runtime::hydrate` doc 参照）。
    let initial_stroke =
        Stroke::new(vec![Point::new(2.0, 2.0)]).expect("1 点のストロークは有効であること");
    let host = TestSignaturePadHost(SignaturePad::new(vec![initial_stroke], false, false));

    let runtime = Runtime::hydrate("signature-pad-runtime-hydrate-clear-root", host)
        .expect("Runtime::hydrate must not fail");

    assert_eq!(runtime.component().0.strokes().len(), 1);

    let clear_trigger = placeholder
        .query_selector(r#"[data-scope="signature-pad"][data-part="clear-trigger"]"#)
        .expect("query_selector must not fail")
        .expect("CSR フォールバック後に clear-trigger 要素が存在すること");
    clear_trigger
        .dispatch_event(&bubbling_event("click"))
        .expect("dispatch_event must not fail");

    assert!(
        runtime.component().0.is_empty(),
        "Runtime::hydrate（CSR フォールバック経路）でも ClearTrigger クリックで \
         全ストロークが削除されていること"
    );
}

/// `Runtime::mount` 経由で dispatch された `add-stroke` が、マウント済み DOM の
/// `data-bind-list="strokes"` keyed list へ実際に新規 `<path>` を反映し、
/// かつその `<path>` が SVG 名前空間で生成されること（イシュー #843、Bugbot
/// 指摘「Runtime skips stroke DOM updates」の回帰固定）。
///
/// 修正前は 2 段階の不具合があった: (1) `SignaturePad` が `DirtyTracked` を
/// 実装しておらず `Component::view` も静的な子ノード列を描画していたため
/// `Runtime::wire_signature_pad` の dirty 駆動 DOM 反映が一切発火しなかった、
/// (2) 仮に発火しても `fandhe-frontend-wasm-client` の keyed list 挿入
/// （`build_dom_node`）は `document.create_element` で HTML 名前空間の
/// 要素を作るため、`<svg>` 配下へ挿入された `<path>` は SVG として描画
/// されなかった（`crates/wasm-client/src/keyed_dom.rs` 側の是正）。本テストは
/// 両方の是正を通しで検証する。
#[wasm_bindgen_test]
fn runtime_mount_add_stroke_updates_keyed_list_dom_in_svg_namespace() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "signature-pad-runtime-mount-add-stroke-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let host = TestSignaturePadHost(SignaturePad::new(Vec::new(), false, false));
    let runtime = Runtime::mount("signature-pad-runtime-mount-add-stroke-root", host)
        .expect("Runtime::mount must not fail");

    let segment = placeholder
        .query_selector(r#"[data-scope="signature-pad"][data-part="segment"]"#)
        .expect("query_selector must not fail")
        .expect("mount 後に segment（svg）要素が存在すること");
    assert!(
        segment
            .query_selector_all("[data-key]")
            .expect("query_selector_all must not fail")
            .length()
            == 0,
        "mount 直後（ストロークなし）は keyed list 子要素が 0 件であること"
    );

    let control = placeholder
        .query_selector(r#"[data-scope="signature-pad"][data-part="control"]"#)
        .expect("query_selector must not fail")
        .expect("mount 後に control 要素が存在すること");
    control
        .dispatch_event(&new_pointer_event("pointerdown", 7))
        .expect("dispatch_event(pointerdown) must not fail");
    control
        .dispatch_event(&new_pointer_event("pointerup", 7))
        .expect("dispatch_event(pointerup) must not fail");

    assert_eq!(
        runtime.component().0.strokes().len(),
        1,
        "pointerdown/pointerup で 1 ストロークが確定していること（前提条件）"
    );

    let path = segment
        .query_selector("path[data-key]")
        .expect("query_selector must not fail")
        .expect(
            "add-stroke 後、Runtime::wire_signature_pad の dirty 駆動 keyed list \
             差分適用でマウント済み DOM へ新規 <path data-key> が挿入されて \
             いること（この要素が見つからない場合、Bugbot 指摘のとおり \
             SignaturePad が dirty_fields を報告していないか、view() が \
             keyed list を描画していない）",
        );

    assert_eq!(
        path.namespace_uri().as_deref(),
        Some("http://www.w3.org/2000/svg"),
        "keyed list 経由で挿入された <path> は SVG 名前空間で生成されている \
         こと（HTML 名前空間のままだとブラウザが SVG として描画しない、\
         `crates/wasm-client/src/keyed_dom.rs` の是正対象）"
    );
}

/// `control`（`segment` を内包する外側コンテナ）上、かつ `segment`（SVG）の
/// 外側にあたる余白部分での `pointerdown` でもストロークが開始されること
/// （Cursor Bugbot 指摘「Control clicks skip stroke start」の回帰固定、
/// イシュー #843 PR #872）。
///
/// 修正前は `segment_rect_transform` が `closest` のみで `segment` 祖先を
/// 探しており、`control`（`segment` の祖先ではなく親、つまり `segment` は
/// `control` の子孫）上のイベントでは解決に失敗し `pointerdown` が早期
/// リターンしていた（`is_drawable_part` が `control` を描画可能パーツとして
/// 許可していたにもかかわらず）。本テストは `control` 要素自身へ
/// `pointerdown`/`pointerup` を送り、`segment` 子要素を子孫探索
/// （`query_selector`）で解決してストロークが確定することを検証する。
#[wasm_bindgen_test]
fn pointerdown_on_control_container_starts_stroke_via_descendant_segment_lookup() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "signature-pad-control-click-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let host = TestSignaturePadHost(SignaturePad::new(Vec::new(), false, false));

    let runtime = Runtime::mount("signature-pad-control-click-root", host)
        .expect("Runtime::mount must not fail");

    assert!(
        runtime.component().0.strokes().is_empty(),
        "mount 直後はストロークなしであること"
    );

    let control = placeholder
        .query_selector(r#"[data-scope="signature-pad"][data-part="control"]"#)
        .expect("query_selector must not fail")
        .expect("mount 後に control 要素が存在すること");

    // `control` 要素自身（`segment` の外側の余白相当）へポインタイベントを
    // 送る。`event.target()` は `control` 自身になる（`segment` の子孫では
    // ない）ため、修正前の `closest` のみの実装ではストローク開始に失敗する。
    let pointer_down = new_pointer_event("pointerdown", 1);
    control
        .dispatch_event(&pointer_down)
        .expect("dispatch_event(pointerdown) must not fail");
    let pointer_up = new_pointer_event("pointerup", 1);
    control
        .dispatch_event(&pointer_up)
        .expect("dispatch_event(pointerup) must not fail");

    assert_eq!(
        runtime.component().0.strokes().len(),
        1,
        "control 要素上での pointerdown/pointerup でも 1 ストロークが確定して \
         いること（イシュー #843 Bugbot 指摘「Control clicks skip stroke \
         start」の回帰）"
    );
}

/// 合成 `PointerEvent` を生成する（`bubbles: true`、指定した `pointerId` を
/// 持つ）。`StrokeCollector` は `pointer_id` の一致で追跡対象を判定するため、
/// pointerdown/pointerup で同じ id を使う必要がある。
fn new_pointer_event(kind: &str, pointer_id: i32) -> web_sys::PointerEvent {
    let init = web_sys::PointerEventInit::new();
    init.set_bubbles(true);
    init.set_pointer_id(pointer_id);
    web_sys::PointerEvent::new_with_event_init_dict(kind, &init)
        .expect("PointerEvent::new must not fail")
}

/// [`new_pointer_event`] の拡張版: クライアント座標と `buttons`
/// （ビットマスク）を明示指定する合成 `PointerEvent`
/// （`angle_slider_browser.rs::pointer_event_with_buttons` と同型）。
///
/// `StrokeCollector::release_if_stale`（イシュー #1992）は `pointermove` の
/// `buttons` を確認するため、既存 [`new_pointer_event`]（`buttons` は既定の
/// `0`）では stale 解除の検証に不可欠な「押下中の pointermove」
/// （`buttons=1`）を組み立てられない。構造フォールバック跨ぎ・stale 解除の
/// 検証（イシュー #1991/#1993/#1994、`structural_fallback` モジュール・
/// [`stale_stroke_tracking_is_released_when_no_button_is_held`]）専用に
/// 追加する（既存 [`new_pointer_event`] の呼び出しは変更しない）。
fn pointer_event_at(
    kind: &str,
    pointer_id: i32,
    client_x: f64,
    client_y: f64,
    buttons: u16,
) -> web_sys::PointerEvent {
    let init = web_sys::PointerEventInit::new();
    init.set_bubbles(true);
    init.set_pointer_id(pointer_id);
    init.set_client_x(client_x.round() as i32);
    init.set_client_y(client_y.round() as i32);
    init.set_buttons(buttons);
    web_sys::PointerEvent::new_with_event_init_dict(kind, &init)
        .expect("PointerEvent::new must not fail")
}

/// SignaturePad の ClearTrigger（`Self::wire_signature_pad` 経路）が keyed
/// list の構造変化を起こした直後、通常の `data-action` クリック
/// （`Self::wire`/`events::wire_events` 経路）が新規挿入ノード内の
/// `data-bind-text` 束縛点を正しく更新できること（PR #872 に対する Cursor
/// Bugbot 指摘「Binding table cache desync」の回帰固定）。
///
/// 修正前は `Self::wire_signature_pad` の `on_update` がローカルに
/// `BindingTable::scan` を取り直すのみで、`Self::wire` 側がクロージャ内に
/// 閉じ込めていた対応表キャッシュを更新しなかった。そのため
/// ClearTrigger クリックで keyed list へ新規ノードが挿入された後、続けて
/// `increment` ボタン（`Self::wire` 経由）をクリックしても、新規ノード内の
/// `data-bind-text="total"` 要素は「ClearTrigger クリック時点より前」の
/// 対応表にしか含まれておらず更新が反映されなかった
/// （リストが空の初期状態から始めるため、修正前は対応表に 1 件も
/// 登録されず更新が完全にスキップされる）。
mod binding_table_cache_desync {
    use super::*;

    /// テスト専用の最小コンポーネント。
    ///
    /// - ClearTrigger（`data-scope="signature-pad" data-part="clear-trigger"`）:
    ///   `MAPPING_TABLE`（`crate::headless`）の `("signature-pad",
    ///   "clear-trigger") -> "clear"` 行を再利用し、`Self::wire_signature_pad`
    ///   経路（keyed list `log` へ現在の `total` を記録したエントリを追加）を
    ///   起動する。実際の `SignaturePad` 型は使わず、マッピング表が
    ///   scope/part 文字列のみで動作することを利用して構成を単純化する。
    /// - `increment` ボタン（`data-action="increment"`）: `Self::wire` 経路で
    ///   `total` を加算し、`data-bind-text="total"` の束縛点（`log` の各
    ///   エントリ内）を更新する。
    struct DesyncHost {
        log: Vec<String>,
        total: u32,
        dirty: Vec<&'static str>,
    }

    enum DesyncAction {
        Clear,
        Increment,
    }

    const FIELD_TOTAL: &str = "total";
    const FIELD_LOG: &str = "log";

    impl fandhe_frontend_interactive::Component for DesyncHost {
        type Action = DesyncAction;

        fn update(&mut self, action: Self::Action) {
            self.dirty.clear();
            match action {
                DesyncAction::Clear => {
                    self.log.push(self.total.to_string());
                    self.dirty.push(FIELD_LOG);
                }
                DesyncAction::Increment => {
                    self.total += 1;
                    self.dirty.push(FIELD_TOTAL);
                }
            }
        }

        fn view(&self) -> fandhe_frontend_core::Node {
            use fandhe_frontend_core::{bind_text, el, keyed::keyed_list, text};

            let items = self
                .log
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    (
                        index.to_string(),
                        el(
                            "li",
                            vec![],
                            vec![bind_text("span", vec![], FIELD_TOTAL, value.clone())],
                        ),
                    )
                })
                .collect();
            let list_node =
                keyed_list("ul", vec![], FIELD_LOG, items).expect("valid keyed list construction");

            let clear_trigger = el(
                "button",
                vec![
                    ("data-scope", "signature-pad"),
                    ("data-part", "clear-trigger"),
                ],
                vec![text("clear")],
            );
            let increment = el(
                "button",
                vec![("data-action", "increment")],
                vec![text("+")],
            );

            el("div", vec![], vec![clear_trigger, increment, list_node])
        }

        fn decode_action(name: &str, _payload: &str) -> Option<Self::Action> {
            match name {
                "clear" => Some(DesyncAction::Clear),
                "increment" => Some(DesyncAction::Increment),
                _ => None,
            }
        }
    }

    impl fandhe_frontend_interactive::DirtyTracked for DesyncHost {
        fn dirty_fields(&self) -> &[&'static str] {
            &self.dirty
        }
    }

    impl fandhe_frontend_wasm_client::BindingSource for DesyncHost {
        fn bound_value(&self, field: &str) -> Option<fandhe_frontend_wasm_client::BoundValue> {
            match field {
                f if f == FIELD_TOTAL => Some(fandhe_frontend_wasm_client::BoundValue::Text(
                    self.total.to_string(),
                )),
                _ => None,
            }
        }
    }

    #[wasm_bindgen_test]
    fn clear_trigger_structural_change_keeps_binding_table_in_sync_for_later_actions() {
        let window = web_sys::window().expect("window must exist");
        let document = window.document().expect("document must exist");
        let placeholder = create_placeholder(&document, "signature-pad-binding-cache-desync-root");
        let _cleanup = RemoveOnDrop(placeholder.clone());

        let host = DesyncHost {
            log: Vec::new(),
            total: 0,
            dirty: Vec::new(),
        };

        let runtime = Runtime::mount("signature-pad-binding-cache-desync-root", host)
            .expect("Runtime::mount must not fail");

        // (1) ClearTrigger クリック（`Self::wire_signature_pad` 経路）で
        // keyed list `log` へ 1 件追加する。挿入された `<li>` 内の
        // `data-bind-text="total"` はこの時点の `total`（0）を静的に保持する。
        let clear_trigger = placeholder
            .query_selector(r#"[data-scope="signature-pad"][data-part="clear-trigger"]"#)
            .expect("query_selector must not fail")
            .expect("clear-trigger element must exist after mount");
        clear_trigger
            .dispatch_event(&bubbling_event("click"))
            .expect("dispatch_event must not fail");

        assert_eq!(
            runtime.component().log.len(),
            1,
            "ClearTrigger クリックで log に 1 件追加されていること"
        );

        let inserted_span = placeholder
            .query_selector(r#"li [data-bind-text="total"]"#)
            .expect("query_selector must not fail")
            .expect("ClearTrigger クリック後に keyed list の新規ノードが存在すること");
        assert_eq!(inserted_span.text_content().as_deref(), Some("0"));

        // (2) `increment` ボタンクリック（`Self::wire`/`events::wire_events`
        // 経路）で `total` を更新する。修正前は `Self::wire` 側の対応表
        // キャッシュが (1) の構造変化を反映しておらず、新規ノード内の
        // `data-bind-text="total"` が更新されずに "0" のまま取り残される。
        let increment = placeholder
            .query_selector(r#"[data-action="increment"]"#)
            .expect("query_selector must not fail")
            .expect("increment element must exist after mount");
        increment
            .dispatch_event(&bubbling_event("click"))
            .expect("dispatch_event must not fail");

        assert_eq!(runtime.component().total, 1);

        let inserted_span = placeholder
            .query_selector(r#"li [data-bind-text="total"]"#)
            .expect("query_selector must not fail")
            .expect("increment クリック後も keyed list のノードが存在すること");
        assert_eq!(
            inserted_span.text_content().as_deref(),
            Some("1"),
            "ClearTrigger クリックで挿入された keyed list ノード内の \
             data-bind-text=\"total\" が increment クリック後に更新されている \
             こと（イシュー #843 Bugbot 指摘「Binding table cache desync」の回帰）"
        );
    }
}

/// capture 喪失中に `root` 外で `pointerup`/`pointercancel` を取り逃した
/// stale な追跡が、`buttons == 0` の `pointermove`
/// （[`fandhe_frontend_wasm_full::headless_signature_pad::StrokeCollector::release_if_stale`]、
/// イシュー #1992）で自己解除され、以後の新規ストローク開始が恒久的に
/// ブロックされないこと（モジュール冒頭 rustdoc「構造フォールバック跨ぎと
/// stale 解除」節参照）。
///
/// `release_if_stale` は capture 再解決（イシュー #1993）より先に評価
/// される（`wiring::wire_stroke_collector` の pointermove 配線）ため、
/// 正準ビュー（Root に `id` を持たない `TestSignaturePadHost`）のままで
/// 検証できる。
///
/// 修正前（`release_if_stale` 導入前）: `pointerdown(1)` で追跡開始した
/// まま `pointerup(1)` を送らずに `pointermove(1, buttons=0)` を送っても
/// 追跡は解除されず、後続の別 pointer id（2）での `pointerdown`/
/// `pointerup` は「既に追跡中」として無視され、ストロークが確定しない
/// （`StrokeCollector::on_pointer_down` の「既に追跡中なら無視」ガード）。
#[wasm_bindgen_test]
fn stale_stroke_tracking_is_released_when_no_button_is_held() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = create_placeholder(&document, "signature-pad-stale-release-root");
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let host = TestSignaturePadHost(SignaturePad::new(Vec::new(), false, false));
    let runtime = Runtime::mount("signature-pad-stale-release-root", host)
        .expect("Runtime::mount must not fail");

    let control = placeholder
        .query_selector(r#"[data-scope="signature-pad"][data-part="control"]"#)
        .expect("query_selector must not fail")
        .expect("mount 後に control 要素が存在すること");
    let segment = placeholder
        .query_selector(r#"[data-scope="signature-pad"][data-part="segment"]"#)
        .expect("query_selector must not fail")
        .expect("mount 後に segment 要素が存在すること");
    let rect = segment.get_bounding_client_rect();
    let (x, y) = (rect.left() + 10.0, rect.top() + 10.0);

    // 1. pointerdown（pointer_id=1、押下中）で追跡開始する。
    control
        .dispatch_event(&pointer_event_at("pointerdown", 1, x, y, 1))
        .expect("dispatch_event(pointerdown) must not fail");

    // 2. pointerup を送らず、押下していない（buttons=0）pointermove を送る
    // （capture 喪失中に root 外で pointerup/pointercancel を取り逃した
    // stale な追跡を模す）。
    control
        .dispatch_event(&pointer_event_at("pointermove", 1, x, y, 0))
        .expect("dispatch_event(pointermove) must not fail");

    assert!(
        runtime.component().0.strokes().is_empty(),
        "stale 判定（release_if_stale）で座標列を破棄しているため、この時点\
         ではストロークが確定していないこと"
    );

    // 3. 別 pointer id（2）で新規ストロークを開始・終了する。
    //    修正前（stale のまま）: on_pointer_down(2) は「追跡中」として無視
    //    され、on_pointer_up(2) は None を返しストロークが確定しない。
    //    修正後: release_if_stale が手順 2 の時点で追跡を解除しているため、
    //    新規追跡が開始され 1 ストロークが確定する。
    control
        .dispatch_event(&pointer_event_at("pointerdown", 2, x, y, 1))
        .expect("dispatch_event(pointerdown) must not fail");
    control
        .dispatch_event(&pointer_event_at("pointerup", 2, x, y, 0))
        .expect("dispatch_event(pointerup) must not fail");

    assert_eq!(
        runtime.component().0.strokes().len(),
        1,
        "stale な追跡が release_if_stale で解除され、別 pointer id での新規\
         ストロークが開始・確定できること（イシュー #1992 の受け入れ条件）"
    );
    assert_eq!(
        runtime.component().0.strokes()[0].points().len(),
        1,
        "stale 側（pointer_id=1）の座標が混入せず、新規ストローク（pointer_id=2）\
         の座標のみが記録されていること"
    );

    // 4. 任意: stale 解除後に pointerdown なしで pointermove（buttons=1）を
    // 送っても no-op であること（pointerdown を経ない新規追跡は開始しない、
    // fail-closed 性質の維持。`angle_slider_browser.rs` の同型ケース末尾と
    // 同じ確認）。
    control
        .dispatch_event(&pointer_event_at("pointermove", 3, x, y, 1))
        .expect("dispatch_event(pointermove) must not fail");
    assert_eq!(
        runtime.component().0.strokes().len(),
        1,
        "pointerdown を経ない pointermove（pointer_id=3）は新規追跡を開始せず、         ストローク件数が変化しないこと"
    );
}

/// ストローク中の構造フォールバック（`Runtime::rerender_subtree`）を挟んだ
/// pointer capture の再付与（イシュー #1993）を検証するテストホスト・
/// ヘルパを閉じ込めるモジュール（モジュール冒頭 rustdoc「構造フォールバック
/// 跨ぎと stale 解除」節参照）。
mod structural_fallback {
    use super::*;

    /// [`StructuralFallbackSignaturePadHost`] が毎 `update()` で
    /// `dirty_fields()` へ積む「束縛点にも keyed list にも対応しない」
    /// フィールド名（`angle_slider_browser.rs::STRUCTURAL_ONLY_FIELD` と
    /// 同型のパターン）。`Runtime::apply_update_for_dirty` はこの field を
    /// `BindingTable::has_field` でも `find_list_element` でも解決できない
    /// ため `unresolved_field` を立て、`Runtime::rerender_subtree`（`root`
    /// 配下の丸ごと差し替え）へフォールバックする。
    const STRUCTURAL_ONLY_FIELD: &str = "structural_only";

    /// `SignaturePad` をラップし、
    ///
    /// - `view()` で SignaturePad Root（`[data-part="root"]`）へ明示的な
    ///   `id`（`root_id`）を付与する（`SignaturePad::view()` の正準ビューは
    ///   Root に `id` を付与しないため、`wiring::capture_anchor_root_id`/
    ///   `resolve_control_by_root_id` による再解決を起動するにはアプリ側の
    ///   明示付与が必須、モジュール `headless_signature_pad.rs` doc「pointer
    ///   capture の再付与」節参照）
    /// - 毎 `update()` で無条件に [`STRUCTURAL_ONLY_FIELD`] を積み、
    ///   `Clear`-on-empty のような no-op 更新でも構造フォールバックを
    ///   誘発する（`Runtime::rerender_subtree` が `root` 配下を丸ごと作り
    ///   直すため、pointerdown で `set_pointer_capture` を設定した Control
    ///   要素がストローク中に detach される状況を最小構成で再現する）
    /// - `AddStroke` の dispatch 回数（`add_stroke_count`）を数える
    ///   （受け入れ条件「`add-stroke` 相当が過不足なく 1 回だけ dispatch
    ///   される」ことを直接検証するため）
    ///
    /// の最小ホスト。`segment` の children は非 keyed の
    /// [`SignaturePad::segment_paths`] を使う（`"strokes"` は束縛点にも
    /// keyed list にも対応せず、[`STRUCTURAL_ONLY_FIELD`] により構造
    /// フォールバックが常に発動する前提のため、keyed 差分経路自体は本
    /// ケースの検証対象外）。
    struct StructuralFallbackSignaturePadHost {
        pad: SignaturePad,
        root_id: &'static str,
        dirty: Vec<&'static str>,
        add_stroke_count: u32,
    }

    impl StructuralFallbackSignaturePadHost {
        fn new(root_id: &'static str) -> Self {
            Self {
                pad: SignaturePad::new(Vec::new(), false, false),
                root_id,
                dirty: Vec::new(),
                add_stroke_count: 0,
            }
        }
    }

    impl fandhe_frontend_interactive::Component for StructuralFallbackSignaturePadHost {
        type Action = SignaturePadAction;

        fn update(&mut self, action: Self::Action) {
            self.dirty.clear();
            if matches!(action, SignaturePadAction::AddStroke(_)) {
                self.add_stroke_count += 1;
            }
            fandhe_frontend_interactive::Component::update(&mut self.pad, action);
            self.dirty
                .extend_from_slice(fandhe_frontend_interactive::DirtyTracked::dirty_fields(
                    &self.pad,
                ));
            // 無条件に構造フォールバックを誘発する（struct doc 参照）。
            self.dirty.push(STRUCTURAL_ONLY_FIELD);
        }

        fn view(&self) -> fandhe_frontend_core::Node {
            let segment = self
                .pad
                .segment(300, 150, None, Vec::new(), self.pad.segment_paths());
            self.pad.root(
                vec![("id", self.root_id)],
                vec![
                    self.pad.control(Vec::new(), vec![segment]),
                    self.pad.clear_trigger(Vec::new(), Vec::new()),
                ],
            )
        }

        fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
            SignaturePad::decode_action(name, payload)
        }
    }

    impl fandhe_frontend_interactive::DirtyTracked for StructuralFallbackSignaturePadHost {
        fn dirty_fields(&self) -> &[&'static str] {
            &self.dirty
        }
    }

    impl fandhe_frontend_wasm_client::BindingSource for StructuralFallbackSignaturePadHost {
        fn bound_value(&self, _field: &str) -> Option<fandhe_frontend_wasm_client::BoundValue> {
            None
        }
    }

    /// ストローク中に構造フォールバック（ClearTrigger クリック経由の
    /// `Runtime::rerender_subtree`）を挟んでも、`wiring::wire_stroke_collector`
    /// が SignaturePad Root の `id`（anchor）から Control を再解決して
    /// pointer capture を掛け直し、座標収集・`add-stroke` dispatch が
    /// 継続すること（イシュー #1993 の受け入れ条件、モジュール冒頭
    /// rustdoc「構造フォールバック跨ぎと stale 解除」節参照）。
    ///
    /// 手順 3 で非描画パーツ（差し替え後の `clear-trigger`）を target に
    /// 選ぶのは回帰判別性のため: 修正前の `segment_rect_transform` は
    /// `closest` → `query_selector` の 2 段探索を持つため、`root` や新しい
    /// `control` を target にすると修正なしでも座標変換の基準が解決できて
    /// しまい回帰を検知できない。`clear-trigger` は SignaturePad の描画
    /// パーツ（`control`/`segment`/`segment-path`）の子孫でも祖先でもない
    /// ため、修正前は `segment_rect_transform` が解決に失敗して座標が
    /// 落ち、修正後は anchor の Root `id` から再解決した Control 基準で
    /// 座標変換が行われるため座標が集まる。
    #[wasm_bindgen_test]
    fn stroke_continues_across_structural_fallback_via_root_id_reattach() {
        const RUNTIME_ROOT_ID: &str = "signature-pad-fallback-runtime-root";
        const PART_ROOT_ID: &str = "signature-pad-fallback-part-root";

        let window = web_sys::window().expect("window must exist");
        let document = window.document().expect("document must exist");
        let placeholder = create_placeholder(&document, RUNTIME_ROOT_ID);
        let _cleanup = RemoveOnDrop(placeholder.clone());

        let host = StructuralFallbackSignaturePadHost::new(PART_ROOT_ID);
        let runtime = Runtime::mount(RUNTIME_ROOT_ID, host).expect("Runtime::mount must not fail");

        let control = || {
            placeholder
                .query_selector(r#"[data-scope="signature-pad"][data-part="control"]"#)
                .expect("query_selector must not fail")
                .expect("control part must exist")
        };
        let segment = || {
            placeholder
                .query_selector(r#"[data-scope="signature-pad"][data-part="segment"]"#)
                .expect("query_selector must not fail")
                .expect("segment part must exist")
        };
        let clear_trigger = || {
            placeholder
                .query_selector(r#"[data-scope="signature-pad"][data-part="clear-trigger"]"#)
                .expect("query_selector must not fail")
                .expect("clear-trigger part must exist")
        };
        // 「現在の segment 左上 + オフセット」のクライアント座標を返す
        // （構造フォールバックで要素が差し替わるため毎回再取得する、
        // `angle_slider_browser.rs::offset_from_center` と同じ意図）。
        let point_in_segment = |dx: f64, dy: f64| -> (f64, f64) {
            let rect = segment().get_bounding_client_rect();
            (rect.left() + dx, rect.top() + dy)
        };

        // 1. pointerdown（追跡開始 + anchor 採取）。
        let (down_x, down_y) = point_in_segment(10.0, 10.0);
        control()
            .dispatch_event(&pointer_event_at("pointerdown", 1, down_x, down_y, 1))
            .expect("dispatch_event(pointerdown) must not fail");

        // 2. ストローク中に ClearTrigger をクリックし、STRUCTURAL_ONLY_FIELD
        // による構造フォールバックを誘発する。
        let control_before = control();
        clear_trigger()
            .dispatch_event(&bubbling_event("click"))
            .expect("dispatch_event(click) must not fail");
        let control_after = control();
        assert!(
            control_after != control_before,
            "ClearTrigger クリックで構造フォールバックが起き、pointerdown で\
             pointer capture を持っていた Control 要素が detach されている\
             こと（本テストが前提とする状況の成立確認）"
        );

        // 3. 差し替え後の非描画パーツ（clear-trigger）を target にして
        // pointermove を 2 回送る（上記 doc「回帰判別性」節参照）。
        let (move1_x, move1_y) = point_in_segment(20.0, 20.0);
        clear_trigger()
            .dispatch_event(&pointer_event_at("pointermove", 1, move1_x, move1_y, 1))
            .expect("dispatch_event(pointermove) must not fail");
        let (move2_x, move2_y) = point_in_segment(30.0, 30.0);
        clear_trigger()
            .dispatch_event(&pointer_event_at("pointermove", 1, move2_x, move2_y, 1))
            .expect("dispatch_event(pointermove) must not fail");

        // 4. pointerup で add-stroke を確定する。
        clear_trigger()
            .dispatch_event(&pointer_event_at("pointerup", 1, move2_x, move2_y, 0))
            .expect("dispatch_event(pointerup) must not fail");

        assert_eq!(
            runtime.component().add_stroke_count,
            1,
            "add-stroke 相当が過不足なく 1 回だけ dispatch されていること"
        );
        assert_eq!(runtime.component().pad.strokes().len(), 1);
        assert_eq!(
            runtime.component().pad.strokes()[0].points().len(),
            3,
            "pointerdown 1 点 + pointermove 2 点の計 3 点が収集されている\
             こと（capture 喪失後も座標収集が継続することの直接的な証拠。\
             `len() == 1` のような件数のみの確認では down/up のみでも成立\
             してしまうため点数で検証する）"
        );
        assert!(
            segment()
                .query_selector(r#"path[data-scope="signature-pad"][data-part="segment-path"]"#)
                .expect("query_selector must not fail")
                .is_some(),
            "pointerup 後の add-stroke dispatch でも STRUCTURAL_ONLY_FIELD に\
             より全再描画が走り、差し替え後の segment に確定したストローク\
             の <path> が反映されていること"
        );

        // 5. 追跡解除の確認: pointerdown を経ない pointermove/pointerup を
        // 送っても新規ストロークが確定しない（add_stroke_count が増えない）
        // こと。
        let (stray_x, stray_y) = point_in_segment(5.0, 5.0);
        clear_trigger()
            .dispatch_event(&pointer_event_at("pointermove", 1, stray_x, stray_y, 1))
            .expect("dispatch_event(pointermove) must not fail");
        clear_trigger()
            .dispatch_event(&pointer_event_at("pointerup", 1, stray_x, stray_y, 0))
            .expect("dispatch_event(pointerup) must not fail");
        assert_eq!(
            runtime.component().add_stroke_count,
            1,
            "pointerup 後は追跡が解除されており、pointerdown を経ない後続の             pointermove/pointerup では新規ストロークが確定しないこと"
        );
    }
}

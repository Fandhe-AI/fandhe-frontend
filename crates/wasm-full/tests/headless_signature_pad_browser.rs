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
/// そのまま委譲するだけの薄いラッパー」パターン）。本テストでは束縛点
/// 更新は検証対象外（`runtime.component()` から直接状態を読むため）のため
/// `DirtyTracked`/`BindingSource` は空実装で足りる。
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
        &[]
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

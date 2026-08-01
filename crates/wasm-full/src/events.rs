//! イベント委譲によるクリック/入力処理（TASK-11.2b、イシュー #75）。
//!
//! `fandhe-frontend-wasm-full` は REQ-11（WASM 完全方式）の既定実装であり、クライアントの
//! イベント処理・DOM 更新を JS グルーへ漏らさず safe Rust の範囲に閉じ込める
//! ことが目的である。本モジュールはその「イベント処理」区画を担当し、
//! DOM 更新（TASK-11.2c、#76）とは責務を分離する。`mount()`/`hydrate()` の
//! 既定実装化（TASK-11.2d、#77）は `wasm-full/src/lib.rs` の `Runtime` が
//! [`wire_events`] を呼び出す形で統合する（本モジュール自体は `Runtime` に
//! 依存しない）。
//!
//! # 設計（PoC-5 `wasm-runtime-split/wasm-full/src/lib.rs` の一般化）
//!
//! - ルート要素へ `click` / `input` / `change` リスナーを **マウント時に 1 回だけ**
//!   委譲登録する（[`wire_events`]）。再描画で子要素が入れ替わってもルートの
//!   リスナーは保持されるため、再描画のたびにリスナーを張り直す必要がない。
//! - リスナー登録は [`wasm_bindgen::closure::Closure::forget`] を click / input /
//!   change の 3 回に限定する（イシュー #1120 で change リスナーを追加、旧 2 回
//!   から改訂）。`forget` は safe API であり `unsafe` ブロックを要しないが、
//!   登録回数を定数個に抑えることで無制限リーク（メモリ枯渇 DoS）を構造的に
//!   回避する（A04: 安全でない設計への対策）。
//! - 属性からのアクション判定ロジック（[`action_from_click`] / [`action_from_input`]）
//!   は web-sys に依存しない純粋関数として切り出し、native の `cargo test` で
//!   検証できるようにする（配線層のみ `#[cfg(target_arch = "wasm32")]` でゲート
//!   し、native ビルドへ web-sys 依存を混入させない）。
//!
//! # 他クレート・他モジュールとの契約
//!
//! - [`ActionRef`] の `action` / `payload` は `fandhe_frontend_interactive::dispatch` の
//!   `name` / `payload` 引数仕様と一致する（`data-action` / `data-payload` 属性、
//!   `interactive/src/lib.rs` の `render_with_root_attrs` が出力する DOM 契約）。
//! - [`wire_events`] は状態更新（`dispatch`）・再描画（DOM 更新、#76 のスコープ）を
//!   直接呼ばず、すべて `on_action` コールバックへ委譲する。これにより本モジュールは
//!   `fandhe-frontend-interactive` の具象状態にも DOM 更新実装にも結合しない。
//! - 再描画出力は呼び出し側（#76/#77）が `fandhe_frontend_core::render()`（既定エスケープ）を
//!   経由させる前提であり、本モジュールは HTML 文字列を一切組み立てない
//!   （REQ-1 不変条件、`.claude/rules/coding-rust.md`）。

/// クリック/入力イベントから判定した「dispatch すべきアクション」への参照。
///
/// `action` は `data-action` 属性値、`payload` は `data-payload`（クリック時）
/// または入力値そのもの（input 時）に対応する。`fandhe_frontend_interactive::dispatch`
/// （`interactive/src/lib.rs`）の `(name, payload)` 引数へそのまま渡せる形。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRef {
    /// `data-action` 属性値（`fandhe_frontend_interactive::Component::decode_action` の `name`）。
    pub action: String,
    /// dispatch へ渡す payload（`fandhe_frontend_interactive::Component::decode_action` の `payload`）。
    pub payload: String,
}

/// イベントターゲット（祖先方向の探索結果を含む）の属性読み取り抽象。
///
/// `web_sys::Element` とテストダブルの双方が実装できるようにし、
/// [`action_from_click`] を native の `cargo test` で検証可能にする。
pub trait AttrSource {
    /// 指定名の属性値を読む。属性が存在しなければ `None`。
    fn attr(&self, name: &str) -> Option<String>;
}

/// click イベントのターゲット属性から dispatch すべきアクションを判定する。
///
/// `data-action` 属性が無ければフレームワーク管轄外のクリックとして `None`
/// を返す（安全側 no-op）。`data-payload` が無い場合は空文字列を payload
/// とする（`fandhe_frontend_interactive::Component::decode_action` 側が未知/不正な
/// payload を no-op として扱う契約に委ねる、不変条件 4）。
///
/// 配線層（[`wire_events`]）は `target.closest("[data-action]")`
/// （`web_sys::Element::closest`）で得た祖先要素を `target` として渡す想定
/// であり、本関数自体は「渡された 1 要素の属性を読む」責務のみを持つ
/// （祖先探索は DOM API 依存のため配線層の責務とし、ここでは扱わない）。
pub fn action_from_click<T: AttrSource>(target: &T) -> Option<ActionRef> {
    let action = target.attr("data-action")?;
    let payload = target.attr("data-payload").unwrap_or_default();
    Some(ActionRef { action, payload })
}

/// input イベントから draft 更新アクションを判定する。
///
/// 対象は `id="draft-input"` の入力欄のみ（`interactive/src/lib.rs` の
/// `render_with_root_attrs` が出力するフォーム入力欄の id 契約に合わせる）。
/// 他 id の input イベントはフレームワーク管轄外として `None` を返す。
///
/// イシュー #345 より前は `should_repaint: false` を返し、`set_inner_html`
/// 全置換によるフォーカス・キャレット破壊を避けるため input イベント後の
/// 再描画自体をスキップしていた（PoC-5 由来の対症療法）。#345 でイベント後
/// 更新が束縛点更新（`set_text_content`/`set_attribute`、変更フィールド数に
/// 比例する冪等な最小更新）へ置き換わったため、この特別扱いは不要になり
/// `should_repaint` フィールド自体を撤去した（`docs/design/dom-binding-update-design.md`
/// #345 実装確定節 §6.1）。キャレット位置の保持は `wasm-client::binding_dom`
/// の value プロパティ等値ガード（現在値と等しければ `set_value` を呼ばない）
/// が担う。
///
/// # レガシー経路（イシュー #1120）
///
/// `id` ハードコードは PoC-5 由来のデモ専用経路であり、`<select>`/`<textarea>`
/// への一般化ができない・利用者アプリが `draft-input` という id を偶然
/// 共有しない限り再利用できないという課題があった（イシュー #1120 の
/// フィードバック 2）。新規アプリは [`ACTION_INPUT_ATTR`]（`data-action-input`
/// 属性契約）を使う [`action_from_form_control`] を使用すべきであり、本関数は
/// 既存の `interactive::AppState` デモ・ブラウザテスト・
/// `docs/api/interactive-api.md` の id 契約との後方互換のためにのみ残す
/// （[`wiring::wire_events`] が `data-action-input` 属性がない場合のみ本関数へ
/// フォールバックする）。
pub fn action_from_input(id: &str, value: &str) -> Option<ActionRef> {
    if id != "draft-input" {
        return None;
    }
    Some(ActionRef {
        action: "set_draft".to_string(),
        payload: value.to_string(),
    })
}

/// input イベント配線で `data-action-input` 属性値をアクション名として使う
/// ことを示す属性契約（イシュー #1120）。
///
/// [`action_from_input`]（`id="draft-input"` ハードコード）の一般化。値
/// フォーム要素（`<input>`/`<textarea>`/`<select>`）に `data-action-input`
/// を付けるだけで input イベントを dispatch へ配線できる。
pub const ACTION_INPUT_ATTR: &str = "data-action-input";

/// change イベント配線で `data-action-change` 属性値をアクション名として
/// 使うことを示す属性契約（イシュー #1120）。
///
/// `<select>`/`<input type="checkbox">`/`<input type="radio">`/`<input
/// type="date">` 等、input イベントではなく change イベントで確定する
/// フォーム要素を dispatch へ配線するための契約（イシュー #1120 の
/// フィードバック 2「select / date / radio / checkbox の change を dispatch
/// に載せる公式経路がない」の解消）。
pub const ACTION_CHANGE_ATTR: &str = "data-action-change";

/// フォーム要素（`input`/`change` の対象）から `attr` 属性値をアクション名、
/// `value` を payload として [`ActionRef`] を組み立てる純粋関数
/// （イシュー #1120）。
///
/// `attr` は [`ACTION_INPUT_ATTR`] または [`ACTION_CHANGE_ATTR`] を渡す想定。
/// 属性が付いていない要素（フレームワーク管轄外の input/change）は `None`
/// を返す（安全側 no-op、[`action_from_click`] と同じ方針）。`value` の
/// 抽出（`checked`/`value` のどちらを使うか）は配線層
/// （[`wiring::extract_form_value`]）の責務であり、本関数は文字列化済みの
/// `value` を受け取るだけの薄いロジックに留める。
pub fn action_from_form_control<T: AttrSource>(
    target: &T,
    attr: &str,
    value: &str,
) -> Option<ActionRef> {
    let action = target.attr(attr)?;
    Some(ActionRef {
        action,
        payload: value.to_string(),
    })
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        action_from_click, action_from_form_control, action_from_input, ActionRef, AttrSource,
        ACTION_CHANGE_ATTR, ACTION_INPUT_ATTR,
    };
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};

    /// `web_sys::Element` を [`AttrSource`] に橋渡しする薄いラッパー。
    ///
    /// 配線層（本モジュール）専用のアダプタであり、純粋ロジック層
    /// （[`action_from_click`]）を web-sys の具象型から独立させたまま保つ。
    struct ElementAttrSource<'a>(&'a Element);

    impl AttrSource for ElementAttrSource<'_> {
        fn attr(&self, name: &str) -> Option<String> {
            self.0.get_attribute(name)
        }
    }

    /// input/change イベントターゲットからフォーム値を文字列として抽出する
    /// （イシュー #1120）。
    ///
    /// `HtmlInputElement`（`type="checkbox"`/`type="radio"` は `checked` を
    /// `"true"`/`"false"` に文字列化、それ以外は `value`）→
    /// `HtmlSelectElement`（`value`）→ `HtmlTextAreaElement`（`value`）の順に
    /// キャストを試み、いずれにも該当しない要素（フレームワークが関知しない
    /// カスタム要素等）は `None` を返す（安全側 no-op）。
    fn extract_form_value(target: &web_sys::EventTarget) -> Option<String> {
        if let Some(input) = target.dyn_ref::<HtmlInputElement>() {
            let input_type = input.type_();
            return Some(if input_type == "checkbox" || input_type == "radio" {
                input.checked().to_string()
            } else {
                input.value()
            });
        }
        if let Some(select) = target.dyn_ref::<HtmlSelectElement>() {
            return Some(select.value());
        }
        if let Some(textarea) = target.dyn_ref::<HtmlTextAreaElement>() {
            return Some(textarea.value());
        }
        None
    }

    /// input イベントの `target` が属性契約 [`ACTION_INPUT_ATTR`] を持つ
    /// 祖先要素に一致する場合のみ [`ActionRef`] を組み立てる（イシュー
    /// #1120）。一致しない・`root` の子孫でない・フォーム値が抽出できない
    /// ・属性値が付いていないのいずれかであれば `None`（呼び出し側は
    /// レガシー経路へフォールバックする、`wire_events` doc 参照）。
    ///
    /// `selector`（`"[data-action-input]"`）は [`wire_events`] がマウント時に
    /// 1 回だけ組み立てて渡す（毎イベントで `format!` を呼ぶアロケーションを
    /// 避けるため）。
    fn attribute_input_action(
        root: &Element,
        target: &web_sys::EventTarget,
        selector: &str,
    ) -> Option<ActionRef> {
        let element = target.dyn_ref::<Element>()?;
        let matched = element.closest(selector).ok().flatten()?;
        if !root.contains(Some(&matched)) {
            return None;
        }
        let value = extract_form_value(target)?;
        let source = ElementAttrSource(&matched);
        action_from_form_control(&source, ACTION_INPUT_ATTR, &value)
    }

    /// ルート要素へ `click` / `input` の委譲リスナーをマウント時に 1 回だけ登録する。
    ///
    /// - `click`: `event.target()` から `closest("[data-action]")` で祖先方向に
    ///   `data-action` 属性を持つ要素を探索する（ボタン内の子要素クリックを
    ///   取りこぼさないための対策。PoC 版は `target()` 直接参照のため子要素
    ///   クリックを取りこぼしていた）。`event.target()` がテキストノード
    ///   （`fandhe_frontend_core::text` が生成するボタン文言等）の場合は `Element` への
    ///   キャストが失敗するため、`Node::parent_element()` で直近の親要素まで
    ///   遡ってから `closest` を呼ぶ（テキストノードクリックの取りこぼし対策、
    ///   PR #200 Cursor Bugbot 指摘）。`Element::closest` は呼び出し要素自身
    ///   から祖先方向へ辿るのみで文書全体は走査しないが、`root` より外側の
    ///   祖先に `data-action` 要素があれば理論上そこまで一致し得るため、本関数は
    ///   `contains` で「ヒットした要素が root の子孫（root 自身を含む）であること」
    ///   を確認してから採用する。
    /// - `input`: `event.target()` から `closest("[data-action-input]")`（click と
    ///   同型の祖先探索。値要素自身が属性を持つ通常のケースでは 1 ステップで
    ///   一致する）で属性契約 [`ACTION_INPUT_ATTR`] 一致を試み、
    ///   [`action_from_form_control`] へ渡す。一致しない場合は
    ///   `event.target()` を `HtmlInputElement` へキャストできた場合のみ
    ///   レガシー経路 [`action_from_input`]（`id="draft-input"` ハードコード）
    ///   へフォールバックする（イシュー #1120。既存デモ・回帰テストの
    ///   非退行）。
    /// - `change`: `event.target()` から `closest("[data-action-change]")` で
    ///   属性契約 [`ACTION_CHANGE_ATTR`] 一致を試みる（`<select>`/checkbox/
    ///   radio/date 等、input イベントでは確定しないフォーム要素向け。
    ///   イシュー #1120 で新規追加）。
    ///
    /// アクション判定に成功した場合のみ `on_action` を呼ぶ（状態更新・再描画は
    /// 呼び出し側の責務。本関数は関知しない）。
    ///
    /// `Closure::forget` は click / input / change の 3 回のみに限定する
    /// （イシュー #1120 で change 分を追加）。マウントはアプリ生存期間に
    /// 1 度だけの前提であり、リーク数は定数個に収まる（`forget` は safe API
    /// であり `unsafe` を要しない）。
    pub fn wire_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let click_root = root.clone();
        let on_action_click = std::rc::Rc::new(std::cell::RefCell::new(on_action));
        let on_action_input = on_action_click.clone();
        let on_action_change = on_action_click.clone();
        let input_root = root.clone();
        let change_root = root.clone();
        // `closest` へ渡すセレクタ文字列はマウント時に 1 回だけ組み立てる
        // （毎イベントで `format!` を呼ぶアロケーションを避けるため、
        // イシュー #1120）。
        let input_selector = format!("[{ACTION_INPUT_ATTR}]");
        let change_selector = format!("[{ACTION_CHANGE_ATTR}]");

        let click_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            // `event.target()` はクリックされた最も深いノードを指し、テキスト
            // ノード（`fandhe_frontend_core::text` が生成する `data-action` ボタン内の文言
            // 等）であることがある。テキストノードは `Element` ではないため
            // `dyn_ref::<Element>()` は `None` を返すが、これは「フレームワーク
            // 管轄外のクリック」ではなく「祖先探索の起点を要素まで遡る必要が
            // ある」ケースである。`Node::parent_element()` で直近の親要素へ
            // 遡ってから `closest` を呼ぶことで、テキストノードクリックでも
            // `data-action` 祖先探索を取りこぼさないようにする（Cursor Bugbot
            // 指摘、PR #200 review 4719004004）。
            let target_element: Element = match target.dyn_ref::<Element>() {
                Some(element) => element.clone(),
                None => {
                    let Some(node) = target.dyn_ref::<web_sys::Node>() else {
                        return;
                    };
                    let Some(parent) = node.parent_element() else {
                        return;
                    };
                    parent
                }
            };
            // data-action を持つ祖先要素を探索する。探索失敗（None）・
            // クエリ不正（Err）はいずれもフレームワーク管轄外のクリックとして
            // 無視する（安全側 no-op）。
            let Ok(Some(matched)) = target_element.closest("[data-action]") else {
                return;
            };
            // click_root の子孫でない要素（closest が別ツリーへ抜けた場合）は
            // 採用しない。`contains` は自分自身も含むため matched == root の
            // ケースも許容する。
            if !click_root.contains(Some(&matched)) {
                return;
            }
            let source = ElementAttrSource(&matched);
            if let Some(action_ref) = action_from_click(&source) {
                (on_action_click.borrow_mut())(action_ref);
            }
        });
        root.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())?;
        click_closure.forget();

        let input_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            // 属性契約 `data-action-input` を優先する（イシュー #1120）。値
            // 要素自身（`event.target()`）が対象のため click と異なりテキスト
            // ノード遡りは不要だが、`closest` は呼び出し要素自身も含めて
            // 祖先方向へ辿るため、値要素自身に属性が付いている通常の構成では
            // そのまま一致する。属性契約に一致しなかった場合のみレガシー
            // 経路（`id="draft-input"` ハードコード）へフォールバックする
            // （`action_from_input` doc 参照、既存アプリの非退行）。
            if let Some(action_ref) = attribute_input_action(&input_root, &target, &input_selector)
            {
                (on_action_input.borrow_mut())(action_ref);
                return;
            }
            let Some(input) = target.dyn_ref::<HtmlInputElement>() else {
                return;
            };
            if let Some(action_ref) = action_from_input(&input.id(), &input.value()) {
                (on_action_input.borrow_mut())(action_ref);
            }
        });
        root.add_event_listener_with_callback("input", input_closure.as_ref().unchecked_ref())?;
        input_closure.forget();

        let change_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            let Some(element) = target.dyn_ref::<Element>() else {
                return;
            };
            let Ok(Some(matched)) = element.closest(&change_selector) else {
                return;
            };
            if !change_root.contains(Some(&matched)) {
                return;
            }
            let Some(value) = extract_form_value(&target) else {
                return;
            };
            let source = ElementAttrSource(&matched);
            if let Some(action_ref) = action_from_form_control(&source, ACTION_CHANGE_ATTR, &value)
            {
                (on_action_change.borrow_mut())(action_ref);
            }
        });
        root.add_event_listener_with_callback("change", change_closure.as_ref().unchecked_ref())?;
        change_closure.forget();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_events;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// native `cargo test` 用のテストダブル。web-sys 非依存で
    /// [`action_from_click`] の判定ロジックを検証する。
    struct FakeElement {
        attrs: HashMap<&'static str, &'static str>,
    }

    impl AttrSource for FakeElement {
        fn attr(&self, name: &str) -> Option<String> {
            self.attrs.get(name).map(|v| v.to_string())
        }
    }

    fn element(attrs: &[(&'static str, &'static str)]) -> FakeElement {
        FakeElement {
            attrs: attrs.iter().copied().collect(),
        }
    }

    #[test]
    fn click_with_action_and_payload_dispatches() {
        let target = element(&[("data-action", "remove_item"), ("data-payload", "2")]);
        let action_ref = action_from_click(&target).expect("data-action present");
        assert_eq!(action_ref.action, "remove_item");
        assert_eq!(action_ref.payload, "2");
    }

    #[test]
    fn click_without_data_action_is_ignored() {
        let target = element(&[("data-testid", "some-div")]);
        assert_eq!(action_from_click(&target), None);
    }

    #[test]
    fn click_with_action_but_no_payload_uses_empty_payload() {
        let target = element(&[("data-action", "increment")]);
        let action_ref = action_from_click(&target).expect("data-action present");
        assert_eq!(action_ref.action, "increment");
        assert_eq!(action_ref.payload, "");
    }

    #[test]
    fn click_with_unknown_action_still_produces_action_ref() {
        // 未知アクション名の判定自体は本モジュールの責務ではない。
        // no-op 化は fandhe_frontend_interactive::dispatch/decode_action 側の契約
        // （不変条件 4）に委ねる。
        let target = element(&[("data-action", "no_such_action")]);
        let action_ref = action_from_click(&target).expect("data-action present");
        assert_eq!(action_ref.action, "no_such_action");
    }

    #[test]
    fn input_on_draft_input_dispatches_set_draft() {
        let action_ref = action_from_input("draft-input", "hello").expect("draft-input matches");
        assert_eq!(action_ref.action, "set_draft");
        assert_eq!(action_ref.payload, "hello");
    }

    #[test]
    fn input_on_other_id_is_ignored() {
        assert_eq!(action_from_input("other-input", "hello"), None);
    }

    /// REQ-1（既定エスケープ）の経路一貫性回帰テスト:
    /// イベント判定 → dispatch → `fandhe_frontend_core::render` の一連の経路を通しても
    /// 生タグが出力に現れないこと（`docs/spec/04-requirements.md` の
    /// 「イベント処理・DOM 更新経由の出力にも同一のエスケープ保証」対応）。
    #[test]
    fn event_to_dispatch_to_render_roundtrip_escapes_script_payload() {
        use fandhe_frontend_interactive::{dispatch, AppState, Component};

        let target = element(&[
            ("data-action", "set_draft"),
            ("data-payload", "<script>alert(1)</script>"),
        ]);
        let action_ref = action_from_click(&target).expect("data-action present");

        let mut state = AppState::new();
        assert!(dispatch(
            &mut state,
            &action_ref.action,
            &action_ref.payload
        ));
        // set_draft だけでは items へ反映されないため、描画確認用に add_item も
        // dispatch して draft の内容を items へ確定させる。
        assert!(dispatch(&mut state, "add_item", ""));

        let html = fandhe_frontend_core::render(&state.view());
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;alert"));
    }

    /// `data-idx`/payload の数値パース失敗が panic しないこと
    /// （`remove_item` は `fandhe_frontend_interactive::AppState::decode_action` 側で
    /// `parse::<usize>()` の失敗を no-op とする契約、境界外・非数値入力）。
    #[test]
    fn remove_item_with_non_numeric_payload_is_noop_not_panic() {
        use fandhe_frontend_interactive::{dispatch, AppState};

        let target = element(&[
            ("data-action", "remove_item"),
            ("data-payload", "not-a-number"),
        ]);
        let action_ref = action_from_click(&target).expect("data-action present");

        let mut state = AppState::new();
        let before = state.clone();
        let dispatched = dispatch(&mut state, &action_ref.action, &action_ref.payload);
        assert!(!dispatched);
        assert_eq!(state, before);
    }

    // -----------------------------------------------------------------
    // イシュー #1120: `data-action-input`/`data-action-change` 属性契約
    // （`action_from_form_control`）の native テスト。
    // -----------------------------------------------------------------

    #[test]
    fn form_control_with_matching_attr_dispatches() {
        let target = element(&[(ACTION_INPUT_ATTR, "select_status")]);
        let action_ref = action_from_form_control(&target, ACTION_INPUT_ATTR, "shipped")
            .expect("data-action-input present");
        assert_eq!(action_ref.action, "select_status");
        assert_eq!(action_ref.payload, "shipped");
    }

    #[test]
    fn form_control_without_matching_attr_is_ignored() {
        let target = element(&[("data-testid", "some-select")]);
        assert_eq!(
            action_from_form_control(&target, ACTION_INPUT_ATTR, "shipped"),
            None
        );
    }

    #[test]
    fn form_control_with_empty_value_uses_empty_payload() {
        let target = element(&[(ACTION_CHANGE_ATTR, "select_status")]);
        let action_ref = action_from_form_control(&target, ACTION_CHANGE_ATTR, "")
            .expect("data-action-change present");
        assert_eq!(action_ref.action, "select_status");
        assert_eq!(action_ref.payload, "");
    }

    /// REQ-1（既定エスケープ）の経路一貫性回帰テスト（属性契約経路版）:
    /// `data-action-input`/`data-action-change` 経由でも XSS ペイロードが
    /// エスケープされること（`event_to_dispatch_to_render_roundtrip_escapes_script_payload`
    /// と同型、イシュー #1120）。
    #[test]
    fn form_control_to_dispatch_to_render_roundtrip_escapes_script_payload() {
        use fandhe_frontend_interactive::{dispatch, AppState, Component};

        let target = element(&[(ACTION_INPUT_ATTR, "set_draft")]);
        let action_ref =
            action_from_form_control(&target, ACTION_INPUT_ATTR, "<script>alert(1)</script>")
                .expect("data-action-input present");

        let mut state = AppState::new();
        assert!(dispatch(
            &mut state,
            &action_ref.action,
            &action_ref.payload
        ));
        assert!(dispatch(&mut state, "add_item", ""));

        let html = fandhe_frontend_core::render(&state.view());
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;alert"));
    }
}

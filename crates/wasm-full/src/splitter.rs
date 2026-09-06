//! Splitter（`fandhe-frontend-headless-ui` `splitter` モジュール）の矢印
//! キーリサイズ配線（イシュー #1074、親トラッキング #1058 配下の keynav
//! 拡充シリーズ #1070/#1071/#1073 に続く）。
//!
//! `crates/headless-ui/src/splitter.rs` は Root/Panel/ResizeTrigger/
//! ResizeTriggerIndicator の anatomy と `SplitterAction::{Increment,
//! Decrement, SetToMin, SetToMax}`（dispatch 名 `"increment"`/
//! `"decrement"`/`"home"`/`"end"`）を提供する一方、実際のキーボード操作 DOM
//! 配線は同モジュール冒頭 rustdoc「スコープ外」節が明記するとおり本クレート
//! （wasm 層）の後続責務とされていた。本モジュールがその配線を実装する。
//!
//! # `aria-valuenow` を直接書き換えない設計判断
//!
//! headless 層の `panel()` は `id`/`data-orientation` しか出力せず、パネル
//! サイズの DOM 表現（`style` 等）を持たない（サイズは
//! `fandhe-frontend-pre-styled-ui` の CSS カスタムプロパティにのみ存在する）。
//! 本モジュールが `aria-valuenow` のみを直接書き換えると、セパレータの読み
//! 上げ値と実際のパネルサイズが乖離する ARIA 上の虚偽表示になる（out-of-scope
//! を残す現状より a11y として悪化する）。そのため本モジュールは `on_action`
//! で dispatch 依頼のみを行い、`aria-valuenow`・パネルサイズの双方が同時に
//! 再生成される再描画へ DOM 反映を委ねる（`crate::angle_slider` の Thumb
//! 回転・`aria-valuenow` 更新と同じ責務分離、`angle_slider.rs` モジュール doc
//! 参照）。この委譲先は `crate::lib::Runtime::wire_splitter` が
//! `crate::lib::Runtime::wire` の返す閉包をそのまま `on_action` として
//! `wire_splitter_events` へ渡すことで実装済み（イシュー #1996。
//! `wire_angle_slider` と同型）。ただし `aria-valuenow` 等の属性自体が
//! dispatch 後に更新されるにはアプリ側が `bind_attr_tokens` で
//! `data-bind-attr` を明示付与している必要がある（`wire_splitter` の
//! rustdoc 参照。`RESIZE_TRIGGER_RESERVED` に `data-bind-attr` は含まれない
//! ため付与は可能）。
//!
//! # `crate::keynav` へ統合しない理由
//!
//! `crate::headless::MAPPING_TABLE` は (scope, part) 1 組につきアクション 1
//! つの静的マッピングであり、方向（ArrowLeft/ArrowRight の増減方向）を
//! 符号化できない。開閉・状態遷移は click 合成で dispatch 経路へ委譲できる
//! 他 keynav 対象と異なり、Splitter は dispatch チャネル（`on_action`
//! コールバック）を要する点で `crate::angle_slider` と同型であり、
//! `crate::keynav` の next-index 系関数へは統合せず、本モジュール内で完結
//! させる（純粋判定は [`fandhe_frontend_wasm_full::keynav::splitter_key_action`]
//! を呼ぶだけで独自ロジックを持たない）。
//!
//! # trigger index の導出
//!
//! `SplitterAction` の `trigger` は「トリガー `i` はパネル `i` とパネル
//! `i+1` の境界」（`Splitter::leading_panel`）という契約だが、DOM は trigger
//! index を明示的に持たない。本モジュールは「アプリは resize-trigger を
//! `0..n-1` の順に描画する」という前提のもと、`root` 配下の resize-trigger
//! を document 順に収集した序数を trigger index として扱う。序数が求まらない
//! 場合（`root` 外要素・ネストした Splitter への誤爆等）は no-op
//! （fail-closed）とし、範囲外 trigger は
//! `SplitterAction::decode_action`/`Splitter::apply_set_size` 側の既存境界
//! チェックがさらに多層防御する。
//!
//! # セキュリティ不変条件
//!
//! - dispatch payload（trigger index の 10 進文字列）は本モジュールが自前で
//!   組み立てた `usize` のみで、DOM 由来の文字列を payload へ通す経路を
//!   持たない。
//! - `data-disabled` を持つ resize-trigger（祖先方向・Splitter root 自身を
//!   含む）上の keydown は no-op（`crate::angle_slider::wiring::has_disabled_ancestor`
//!   と同型の判定を本モジュール内に個別実装する）。
//! - DOM 反映は `get_attribute`/`matches`/`query_selector_all`/
//!   `prevent_default` のみで行い、`set_inner_html` を含む HTML 文字列を
//!   一切組み立てない（REQ-1）。属性名・セレクタはすべて `&'static str`
//!   リテラル固定で、動的値が属性名・セレクタスロットへ混入する経路はない。
//! - `Closure::forget` は 1 回のみ（keydown 委譲リスナー 1 個。
//!   `crate::keynav::wire_keynav` の 3 回・`crate::angle_slider` の 3 回とは
//!   独立したリスナーであり、無制限リークを構造的に回避する定数個契約は
//!   モジュールをまたいでも維持される。A04 対策）。
//! - 新規 `unsafe` コードは追加しない（`web-sys`/`wasm-bindgen` の safe API
//!   のみ使用）。

/// Splitter の `data-scope` 属性値。
const SPLITTER_SCOPE: &str = "splitter";
/// Root パーツの `data-part` 属性値。`crate::keynav::wiring` の他 `*_SELECTOR`
/// 定数と異なり配線層（`#[cfg(target_arch = "wasm32")]`）内でのみ参照する
/// ため、native ビルド（`cargo clippy --lib`、wasm32 ゲート・
/// `#[cfg(test)]` のいずれも対象外）では未使用になる。`headless_clipboard.rs`/
/// `headless_avatar.rs` 等の同型定数と同じ理由で `allow(dead_code)` を
/// native 限定で付与する。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const ROOT_PART: &str = "root";
/// ResizeTrigger パーツの `data-part` 属性値。
const RESIZE_TRIGGER_PART: &str = "resize-trigger";

/// dispatch アクション名 `"increment"`（`SplitterAction::Increment`/
/// `Splitter::decode_action` の対応する分岐と一致）。
pub const ACTION_INCREMENT: &str = "increment";
/// dispatch アクション名 `"decrement"`。
pub const ACTION_DECREMENT: &str = "decrement";
/// dispatch アクション名 `"home"`。
pub const ACTION_HOME: &str = "home";
/// dispatch アクション名 `"end"`。
pub const ACTION_END: &str = "end";

/// クリック/キーボードターゲットが Splitter の resize-trigger パーツかどうか
/// を判定する純粋関数（DOM 非依存、native `cargo test` で検証可能。
/// `crate::angle_slider::is_angle_slider_control_or_thumb` と同型の 2 層
/// 構成）。配線層（[`wiring::is_resize_trigger`]）はこの純粋判定へ
/// `Element::get_attribute` の読み取り結果を渡すだけで、独自の scope/part
/// 比較ロジックを持たない。
#[must_use]
pub fn is_resize_trigger_part(scope: Option<&str>, part: Option<&str>) -> bool {
    scope == Some(SPLITTER_SCOPE) && part == Some(RESIZE_TRIGGER_PART)
}

// ---------------------------------------------------------------------
// 配線層: web-sys/js-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`angle_slider.rs`/`headless_clipboard.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        is_resize_trigger_part, ACTION_DECREMENT, ACTION_END, ACTION_HOME, ACTION_INCREMENT,
        ROOT_PART, SPLITTER_SCOPE,
    };
    use crate::events::ActionRef;
    use crate::keynav::{splitter_key_action, Modifiers, Orientation, SplitterKeyAction};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, HtmlElement, KeyboardEvent};

    /// `target` から `root`（含む）まで祖先方向へ辿り、`data-scope`/
    /// `data-part` が指定値と一致する最初の要素を返す
    /// （`crate::angle_slider::wiring::closest_matching` と同型）。
    fn closest_matching(
        root: &Element,
        start: &Element,
        scope: &str,
        part: &str,
    ) -> Option<Element> {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if !root.contains(Some(&element)) {
                break;
            }
            if element.get_attribute("data-scope").as_deref() == Some(scope)
                && element.get_attribute("data-part").as_deref() == Some(part)
            {
                return Some(element);
            }
            if element == *root {
                break;
            }
            current = element.parent_element();
        }
        None
    }

    /// `start` から `root` まで祖先方向を辿り、`data-disabled` を持つ要素が
    /// 1 つでもあれば `true` を返す（disabled な祖先・Splitter root 自身を
    /// 含めて no-op とする fail-closed 判定。
    /// `crate::angle_slider::wiring::has_disabled_ancestor` と同型）。
    fn has_disabled_ancestor(root: &Element, start: &Element) -> bool {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if element.has_attribute("data-disabled") {
                return true;
            }
            if !root.contains(Some(&element)) || element == *root {
                break;
            }
            current = element.parent_element();
        }
        false
    }

    /// `element` が Splitter の resize-trigger パーツかどうか
    /// （[`is_resize_trigger_part`] 純粋層への薄いアダプタ）。
    fn is_resize_trigger(element: &Element) -> bool {
        is_resize_trigger_part(
            element.get_attribute("data-scope").as_deref(),
            element.get_attribute("data-part").as_deref(),
        )
    }

    /// resize-trigger パーツの CSS セレクタ（[`collect_own_resize_triggers`]/
    /// [`resolve_trigger`] の双方が使う共通定数）。
    const RESIZE_TRIGGER_SELECTOR: &str = "[data-scope=\"splitter\"][data-part=\"resize-trigger\"]";

    /// `splitter_root` 配下の resize-trigger を document 順に収集する。
    /// ネストした Splitter インスタンスの resize-trigger を誤って拾わない
    /// よう、各要素について [`closest_matching`]（scope=splitter,
    /// part=root）が `splitter_root` 自身に一致するもののみを採用する
    /// （モジュール doc §trigger index の導出参照。A01 対策）。
    fn collect_own_resize_triggers(splitter_root: &Element) -> Vec<Element> {
        let Ok(node_list) = splitter_root.query_selector_all(RESIZE_TRIGGER_SELECTOR) else {
            return Vec::new();
        };
        let len = node_list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            let Some(node) = node_list.get(i) else {
                continue;
            };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            let owns = closest_matching(splitter_root, &element, SPLITTER_SCOPE, ROOT_PART)
                .is_some_and(|owner| owner.is_same_node(Some(splitter_root)));
            if owns {
                out.push(element);
            }
        }
        out
    }

    /// `root` 配下の Splitter resize-trigger へ `keydown` 委譲リスナーを
    /// マウント時に 1 回だけ登録する（モジュール doc §セキュリティ不変条件
    /// 参照）。
    ///
    /// `on_action` は `"increment"`/`"decrement"`/`"home"`/`"end"` の
    /// dispatch 依頼を呼び出し側（`crate::lib::Runtime::wire_splitter`）へ
    /// 渡すのみで、状態更新・DOM 反映は行わない
    /// （`angle_slider::wire_angle_slider_events` と同じ責務分離）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` の失敗を伝播する。
    pub fn wire_splitter_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let on_action = std::rc::Rc::new(std::cell::RefCell::new(on_action));
        let keydown_root = root.clone();
        let keydown_on_action = on_action.clone();
        let keydown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_keydown(&keydown_root, &event, &keydown_on_action);
        });
        root.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())?;
        keydown_closure.forget();
        Ok(())
    }

    /// keydown: resize-trigger 上でのみ反応する（[`is_resize_trigger`]）。
    /// root 封じ込め検査・disabled 除外・Splitter root の解決・
    /// [`splitter_key_action`] への委譲・trigger index 導出をこの 1 関数に
    /// まとめる。
    ///
    /// dispatch が構造フォールバックを誘発して resize-trigger ごと
    /// 差し替えた場合は [`restore_trigger_focus`] で再描画後の同じ
    /// resize-trigger へフォーカスを戻し、連続した矢印キー操作が
    /// 途切れないようにする（イシュー #1996 codex-review P1 是正、
    /// `angle_slider::wiring::handle_keydown`/`restore_thumb_focus` と
    /// 同型）。
    fn handle_keydown(
        root: &Element,
        event: &Event,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        let Some(target) = event.target() else {
            return;
        };
        let Some(target_element) = target.dyn_ref::<Element>() else {
            return;
        };
        if !is_resize_trigger(target_element) {
            return;
        }
        if !root.contains(Some(target_element)) {
            return;
        }
        if has_disabled_ancestor(root, target_element) {
            return;
        }
        let Some(splitter_root) = closest_matching(root, target_element, SPLITTER_SCOPE, ROOT_PART)
        else {
            return;
        };

        let orientation =
            Orientation::from_attr(splitter_root.get_attribute("data-orientation").as_deref());
        let modifiers = Modifiers {
            ctrl: keyboard_event.ctrl_key(),
            alt: keyboard_event.alt_key(),
            meta: keyboard_event.meta_key(),
        };
        let Some(action) = splitter_key_action(&keyboard_event.key(), orientation, modifiers)
        else {
            return;
        };

        let triggers = collect_own_resize_triggers(&splitter_root);
        let Some(index) = triggers
            .iter()
            .position(|el| el.is_same_node(Some(target_element)))
        else {
            return;
        };

        // dispatch 前にフォーカス復元用のキーを採取する（dispatch 後は
        // 対象要素が detach され `closest_matching` による Root 探索が
        // できなくなるため、`angle_slider::wiring::handle_keydown` と同じ
        // 手順）。
        let key = trigger_key(target_element);

        keyboard_event.prevent_default();
        let action_name = match action {
            SplitterKeyAction::Increment => ACTION_INCREMENT,
            SplitterKeyAction::Decrement => ACTION_DECREMENT,
            SplitterKeyAction::SetToMin => ACTION_HOME,
            SplitterKeyAction::SetToMax => ACTION_END,
        };
        if let Ok(mut cb) = on_action.try_borrow_mut() {
            (cb)(ActionRef {
                action: action_name.to_string(),
                payload: index.to_string(),
            });
        }

        restore_trigger_focus(root, target_element, key.as_ref());
    }

    /// 再描画をまたいで同じ resize-trigger を再解決するための識別子
    /// （イシュー #1996 codex-review P1 是正）。
    ///
    /// 構造フォールバック（[`crate::Runtime::rerender_subtree`]）は
    /// `state.view()` から DOM を作り直すため、要素参照も要素の同一性も
    /// 再描画をまたいで保持できない。当初は Splitter Root の `id` と
    /// document 順序（trigger index）の組をフォールバック識別子にして
    /// いたが、(1) 再解決時に `collect_own_resize_triggers` と同じ
    /// 所有権フィルタを適用しておらず、ネストした Splitter が存在すると
    /// 別の Splitter の resize-trigger を index 経由で誤って再解決し得た、
    /// (2) headless-ui の [`fandhe_frontend_headless_ui::splitter::root`]/
    /// [`fandhe_frontend_headless_ui::splitter::resize_trigger`] はいずれも `id`
    /// 自動付与も必須付与もしないため、Root/resize-trigger に `id` を
    /// 付けない標準構成ではフォールバック自体が成立せずキーボード操作が
    /// 構造フォールバック 1 回で途切れていた（codex-review P1 x2 是正）。
    ///
    /// resize-trigger は [`fandhe_frontend_headless_ui::splitter::resize_trigger`]
    /// が必須引数 `leading_id`/`trailing_id`（隣接パネルの `id`。
    /// [`fandhe_frontend_headless_ui::splitter::panel`] も `id` を必須で受け取るため
    /// 常に存在する）から `data-id="<leading_id>:<trailing_id>"` を常に
    /// 出力する。パネル `id` は WAI-ARIA `aria-controls` の参照先であり
    /// 文書内で一意であることが前提のため、この `data-id` は
    /// Root/resize-trigger 自身の `id` の有無に関わらず常に得られる
    /// 安定識別子として使える（ネストした Splitter 同士でも `leading_id`/
    /// `trailing_id` の組が重複しない限り一意）。
    #[derive(Clone)]
    enum TriggerKey {
        /// resize-trigger 自身の `id` 属性（最も安定。アプリが `id` を
        /// 付けている場合に使う）。
        OwnId(String),
        /// resize-trigger の `data-id` 属性（`"<leading_id>:<trailing_id>"`。
        /// `id` を付けない標準構成でも常に得られるフォールバック）。
        DataId(String),
    }

    /// `target_element`（resize-trigger）を再描画後に再解決するための
    /// [`TriggerKey`] を決める。resize-trigger は常に `data-id` を持つため
    /// （headless-ui `resize_trigger` の契約、[`TriggerKey`] doc 参照）、
    /// `is_resize_trigger` 判定を通過した呼び出し元では実質的に必ず
    /// `Some` を返す。属性が欠落した想定外の DOM のみ `None`（fail-closed、
    /// 呼び出し側はフォーカス復元を行わず本 PR 以前と同じ挙動へ
    /// フォールバックする）。
    fn trigger_key(target_element: &Element) -> Option<TriggerKey> {
        let own_id = target_element.id();
        if !own_id.is_empty() {
            return Some(TriggerKey::OwnId(own_id));
        }
        let data_id = target_element.get_attribute("data-id")?;
        if data_id.is_empty() {
            return None;
        }
        Some(TriggerKey::DataId(data_id))
    }

    /// `root` 配下から [`TriggerKey`] に対応する resize-trigger を
    /// 再解決する。対象が消えている・一意に定まらない場合はいずれも
    /// `None`（fail-closed）。呼び出し側はフォーカス復元断念のシグナル
    /// として扱う。
    fn resolve_trigger(root: &Element, key: &TriggerKey) -> Option<Element> {
        match key {
            TriggerKey::OwnId(id) => {
                let document = root.owner_document()?;
                let candidate = document.get_element_by_id(id)?;
                if !root.contains(Some(&candidate)) {
                    return None;
                }
                if !is_resize_trigger(&candidate) {
                    return None;
                }
                Some(candidate)
            }
            TriggerKey::DataId(data_id) => {
                // CSS セレクタへ動的な属性値をそのまま埋め込まず、
                // `root` 配下の resize-trigger 全件を固定セレクタで収集
                // してから Rust 側で `data-id` を比較する（属性値に
                // クォート等の CSS 特殊文字が含まれても injection の
                // 余地がない）。`data-id` は隣接 2 パネルの `id` から
                // 一意に定まる契約（[`TriggerKey`] doc 参照）のため、
                // ここでの所有権検証はネストした Splitter を含めても
                // 一致件数の一意性確認のみで足りる（複数一致は
                // fail-closed に諦める）。
                let list = root.query_selector_all(RESIZE_TRIGGER_SELECTOR).ok()?;
                let mut found: Option<Element> = None;
                for i in 0..list.length() {
                    let element = list.get(i)?.dyn_into::<Element>().ok()?;
                    if element.get_attribute("data-id").as_deref() == Some(data_id.as_str()) {
                        if found.is_some() {
                            return None;
                        }
                        found = Some(element);
                    }
                }
                found
            }
        }
    }

    /// keydown の dispatch 後、resize-trigger が構造フォールバックで
    /// detach されていた場合に再描画後の同じ resize-trigger へフォーカスを
    /// 戻す（イシュー #1996 codex-review P1 是正、
    /// `angle_slider::wiring::restore_thumb_focus` と同型）。
    ///
    /// 復元は以下の条件をすべて満たす場合に限る（fail-closed）:
    ///
    /// - dispatch 前の resize-trigger が実際に detach された
    ///   （`is_connected()` が `false`）。再描画が起きなかった通常経路では
    ///   何もしない
    /// - [`TriggerKey`] から再描画後の resize-trigger を一意に再解決できた
    ///   （[`resolve_trigger`] doc 参照）
    ///
    /// いずれかを満たさない場合は何もしない（利用者のフォーカスを勝手に
    /// 奪わない）。
    fn restore_trigger_focus(
        root: &Element,
        previous_trigger: &Element,
        trigger_key: Option<&TriggerKey>,
    ) {
        if previous_trigger.is_connected() {
            return;
        }
        let Some(trigger_key) = trigger_key else {
            return;
        };
        let Some(trigger) = resolve_trigger(root, trigger_key) else {
            return;
        };
        if let Some(focusable) = trigger.dyn_ref::<HtmlElement>() {
            let _ = focusable.focus();
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_splitter_events;

#[cfg(test)]
mod tests {
    use super::*;

    // --- ドリフト検知: headless-ui の実出力（data-scope/data-part 値）が
    // 本モジュールのリテラルと一致すること（`angle_slider.rs` の同型テストと
    // 同じ方針）。---

    #[test]
    fn headless_ui_root_output_matches_module_literals() {
        use fandhe_frontend_core::render;
        use fandhe_frontend_headless_ui::data_attrs::Orientation as HeadlessOrientation;
        use fandhe_frontend_headless_ui::splitter::root;

        let html = render(&root(
            HeadlessOrientation::Horizontal,
            false,
            Vec::new(),
            Vec::new(),
        ));
        assert!(html.contains(&format!(r#"data-scope="{SPLITTER_SCOPE}""#)));
        assert!(html.contains(&format!(r#"data-part="{ROOT_PART}""#)));
    }

    #[test]
    fn headless_ui_resize_trigger_output_matches_module_literals() {
        use fandhe_frontend_core::render;
        use fandhe_frontend_headless_ui::data_attrs::Orientation as HeadlessOrientation;
        use fandhe_frontend_headless_ui::splitter::resize_trigger;

        let html = render(&resize_trigger(
            HeadlessOrientation::Horizontal,
            "0",
            "100",
            "50",
            "panel-0",
            "panel-1",
            false,
            Vec::new(),
            Vec::new(),
        ));
        assert!(html.contains(&format!(r#"data-scope="{SPLITTER_SCOPE}""#)));
        assert!(html.contains(&format!(r#"data-part="{RESIZE_TRIGGER_PART}""#)));
    }

    // --- dispatch アクション名の一致: 本モジュールの ACTION_* 定数が
    // `Splitter::decode_action` の受理する名前と一致すること ---

    #[test]
    fn decode_action_accepts_action_constants() {
        use fandhe_frontend_headless_ui::splitter::Splitter;
        use fandhe_frontend_interactive::Component;

        assert!(<Splitter as Component>::decode_action(ACTION_INCREMENT, "0").is_some());
        assert!(<Splitter as Component>::decode_action(ACTION_DECREMENT, "0").is_some());
        assert!(<Splitter as Component>::decode_action(ACTION_HOME, "0").is_some());
        assert!(<Splitter as Component>::decode_action(ACTION_END, "0").is_some());
        assert!(<Splitter as Component>::decode_action("no_such_action", "0").is_none());
    }

    // --- roundtrip: action 名 → dispatch → Splitter::size ---

    #[test]
    fn increment_and_decrement_roundtrip_via_dispatch() {
        use fandhe_frontend_headless_ui::splitter::Splitter;

        let mut s = Splitter::default();
        assert_eq!(s.size(0), Some(50.0));

        assert!(fandhe_frontend_interactive::dispatch(
            &mut s,
            ACTION_INCREMENT,
            "0"
        ));
        assert_eq!(s.size(0), Some(51.0));

        assert!(fandhe_frontend_interactive::dispatch(
            &mut s,
            ACTION_DECREMENT,
            "0"
        ));
        assert_eq!(s.size(0), Some(50.0));
    }

    #[test]
    fn home_and_end_roundtrip_via_dispatch() {
        use fandhe_frontend_headless_ui::data_attrs::Orientation as HeadlessOrientation;
        use fandhe_frontend_headless_ui::splitter::{PanelSpec, Splitter};

        let mut s = Splitter::new(
            &[
                PanelSpec::new(50.0, 20.0, 80.0),
                PanelSpec::new(50.0, 20.0, 80.0),
            ],
            HeadlessOrientation::Horizontal,
        );
        assert!(fandhe_frontend_interactive::dispatch(
            &mut s,
            ACTION_HOME,
            "0"
        ));
        assert_eq!(s.size(0), Some(20.0));

        assert!(fandhe_frontend_interactive::dispatch(
            &mut s, ACTION_END, "0"
        ));
        assert_eq!(s.size(0), Some(80.0));
    }
}

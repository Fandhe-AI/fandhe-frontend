//! hover / press ジェスチャー配線（イシュー #2520、親 #2508/#2491）。
//!
//! `docs/design/motion-reference-adoption-policy.md` §4 は motion.dev の
//! `hover()`/`press()` を「A 群: 既存 CSS `:hover`/`:active`/
//! `:focus-visible`（`docs/design/pre-styled-ui-interaction-visual-language.md`）
//! で足りる」に大半を分類する。本モジュールはその外側にある、CSS 疑似
//! クラスだけでは表現できない 2 点の残余ギャップのみを埋める:
//!
//! 1. タッチ端末で `:hover` がタップ後に貼り付く疑似 hover を除去した
//!    「非タッチ限定 hover」の判定を `data-*` として書き戻す。
//! 2. `role="button"` 等のカスタムインタラクティブ要素は keyboard
//!    （Enter/Space）活性化で `:active` が付かないため、press 視覚状態を
//!    `data-*` として供給する。
//!
//! `IntersectionObserver`/Pointer Events の購読と `data-*` 書き戻しのみで
//! 完結する群 B 相当であり、`fandhe_animation`/`fandhe_frontend_animation`
//! の `Driver`/`Target` 連携は不要（3 層構成、#2491）。`events.rs`/
//! `keynav.rs`/`sidebar.rs` と同じ 2 層構成（DOM 非依存の純粋ロジック層 +
//! `#[cfg(target_arch = "wasm32")]` 配線層）を踏襲する。
//!
//! # `data-*` 属性の命名（既存語彙との意図的な非採用の踏襲）
//!
//! `data-hover`/`data-active`/`data-state="hover"` は採用しない。
//! `crates/headless-ui/`（`switch.rs` 他）は ark-ui 由来の `data-hover`/
//! `data-active` を「CSS 疑似クラスで足りる」として意図的に不採用にした
//! 設計判断を明記済みであり、ここで同名を新規採用すると読み手を混乱
//! させる。`data-active`（chart 系ハイライト等）・`data-pressed`
//! （Toggle の永続選択状態）・`data-state`（各部品の状態機械）はいずれも
//! 既存の別意味論で広く使用済みのため避け、内部実装用の
//! `data-fandhe-*` プレフィックス（`stagger_index.rs`/`content_height.rs`
//! と同型）を用いる。opt-in を hover/press で分離するのは、コストが
//! 異なる（hover は 2 リスナー共有、press は 6 リスナー共有）ため、
//! 利用者が必要な方だけ opt-in できるようにするため。
//!
//! # Runtime への統合
//!
//! `crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が
//! `Self::wire_in_view` の直後で `wire_gesture` を呼ぶ（feature
//! `gesture`、既定 on）。`dispatch` チャネルを持たない属性専用配線のため
//! `Component`/`binding_table`/`keyed_list_cache` は必要としない
//! （`Self::wire_sidebar`/`Self::wire_in_view` と同型）。
//!
//! # セキュリティ不変条件（REQ-1・security.md A03）
//!
//! DOM へ書き込む値は [`HOVER_STATE_ATTR`]/[`PRESS_STATE_ATTR`] いずれも
//! 常に空文字列 `""` の存在属性のみであり、ポインタ座標・キー文字列・
//! 利用者制御の入力を属性値・セレクタへ混ぜない。

/// opt-in（著者が SSR 出力に静的に付与）: hover 検知を有効化する。
pub const GESTURE_HOVER_ATTR: &str = "data-fandhe-gesture-hover";
/// opt-in（著者が SSR 出力に静的に付与）: press 検知を有効化する。
pub const GESTURE_PRESS_ATTR: &str = "data-fandhe-gesture-press";
/// 状態（wasm-full が動的に付け外す、存在で真を表す）: 非タッチ hover 中。
pub const HOVER_STATE_ATTR: &str = "data-fandhe-hover";
/// 状態（同上）: press 中（pointer または keyboard 活性化）。
pub const PRESS_STATE_ATTR: &str = "data-fandhe-press";

/// `PointerEvent::pointer_type()` がタッチ由来か（タッチ疑似 hover 除去用）。
///
/// `crate::chart::is_sticky_pointer` と同一ロジックだが意味的文脈が異なる
/// （sticky tooltip 判定 vs pseudo-hover 除去）ため独立定義する。
#[must_use]
pub fn is_touch_pointer(pointer_type: &str) -> bool {
    pointer_type == "touch"
}

/// `KeyboardEvent::key()` が press 活性化キー（Enter/Space）か。
///
/// `keynav.rs` の活性化キー判定と同じ 2 語のみを見る（"Spacebar" 等の
/// レガシー別名は既存コードに前例がないため追加しない）。
#[must_use]
pub fn is_press_activation_key(key: &str) -> bool {
    key == "Enter" || key == " "
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        is_press_activation_key, is_touch_pointer, GESTURE_HOVER_ATTR, GESTURE_PRESS_ATTR,
        HOVER_STATE_ATTR, PRESS_STATE_ATTR,
    };
    use crate::dom::set_dom_attribute_result as set_dom_attribute;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, FocusEvent, KeyboardEvent, MouseEvent, PointerEvent};

    /// 内部専用（外部非公開）: press が pointer 由来であることを示す
    /// マーカー。`pointerout` が keyboard 由来の press まで誤って解除
    /// しないための区別に使う（codex-review 指摘の是正）。[`HOVER_STATE_ATTR`]/
    /// [`PRESS_STATE_ATTR`] と同じく常に空文字列の存在属性のみで REQ-1 の
    /// 不変条件を保つ。
    const PRESS_POINTER_ACTIVE_ATTR: &str = "data-fandhe-press-pointer-active";

    /// `pointerdown` で実際に press された要素の共有状態
    /// （`PointerEvent::pointer_id()` ごとに保持する）。`pointerup`/
    /// `pointercancel`/`pointerout` は `event.target()` から祖先を
    /// 再計算せずこの共有状態を参照する（親子とも opt-in の場合に
    /// pointerdown 時と異なる要素を解除してしまう不具合の是正、
    /// codex-review 指摘）。`pointer_id` 単位で保持するのは、フォーカス
    /// 移動を伴わずに複数ポインタ（マルチタッチ・複数マウス）が別要素を
    /// 同時に押下したとき、後発の pointerdown が先発の押下要素を上書きして
    /// 解放漏れを起こす不具合の是正（codex-review 指摘）。
    type ActivePress = Rc<RefCell<HashMap<i32, Element>>>;

    /// `active` から `pointer_id` エントリを取り除き、取り除いた要素を
    /// 指す他の `pointer_id` エントリが 1 件も残っていない場合のみ
    /// press 状態属性（[`PRESS_STATE_ATTR`]/[`PRESS_POINTER_ACTIVE_ATTR`]）
    /// を解除する。同一要素を 2 本指で押下した状態から片方だけ離すと、
    /// もう片方の `pointer_id` が生きていても即座に属性が消えていた
    /// 不具合の是正（codex-review 指摘）: 押下源を要素単位で集約し、
    /// 全ポインタが解放されたときのみ解除する。
    fn release_pointer_press(active: &ActivePress, pointer_id: i32) {
        let mut active = active.borrow_mut();
        let Some(released) = active.remove(&pointer_id) else {
            return;
        };
        let still_pressed = active.values().any(|element| element == &released);
        drop(active);
        if !still_pressed {
            let _ = released.remove_attribute(PRESS_STATE_ATTR);
            let _ = released.remove_attribute(PRESS_POINTER_ACTIVE_ATTR);
        }
    }

    /// `event.target()` を `Element` として取得する（`Text` ノード等は
    /// `None`）。
    fn event_target_element(event: &Event) -> Option<Element> {
        event.target()?.dyn_into::<Element>().ok()
    }

    /// `event` の `relatedTarget` を `Element` として取得する。
    /// `pointerover`/`pointerout`（`MouseEvent`/`PointerEvent`）と
    /// `focusout`（`FocusEvent`）の両方に対応する（Bugbot 指摘の是正:
    /// `handle_focusout` が新フォーカス先の内外を判定できるようにする）。
    fn related_target_element(event: &Event) -> Option<Element> {
        if let Some(mouse) = event.dyn_ref::<MouseEvent>() {
            return mouse.related_target()?.dyn_into::<Element>().ok();
        }
        event
            .dyn_ref::<FocusEvent>()?
            .related_target()?
            .dyn_into::<Element>()
            .ok()
    }

    /// `related`（`relatedTarget`）が `boundary`（含む）配下に留まって
    /// いるか。子要素間の内部移動を「離脱」と誤判定しないための境界判定
    /// （`sidebar::wiring` の `pointerover`/`pointerout` 判定と同型）。
    fn related_within(event: &Event, boundary: &Element) -> bool {
        related_target_element(event).is_some_and(|related| boundary.contains(Some(&related)))
    }

    /// `event` の `PointerEvent::pointer_id()`（`PointerEvent` でなければ
    /// `None`）。複数ポインタを区別して press 状態を独立管理するための
    /// キー（`ActivePress` 参照）。
    fn event_pointer_id(event: &Event) -> Option<i32> {
        Some(event.dyn_ref::<PointerEvent>()?.pointer_id())
    }

    /// `root` 配下・`opt_in_attr` を持つ最も近い祖先（自身含む）を返す。
    fn closest_opted_in(root: &Element, target: &Element, opt_in_attr: &str) -> Option<Element> {
        let selector = format!("[{opt_in_attr}]");
        let matched = target.closest(&selector).ok().flatten()?;
        root.contains(Some(&matched)).then_some(matched)
    }

    /// `root` 配下・`opt_in_attr` を持つ祖先（自身含む）を近い順に**すべて**
    /// 返す。入れ子の opt-in 要素（親子とも opt-in）を想定し、`target` から
    /// 1 段ずつ `closest` を辿って積み上げる（`closest_opted_in` は最も
    /// 近い 1 件しか返さないため、離脱時に外側の祖先が取り残される
    /// 不具合の是正、codex-review 指摘）。
    fn opted_in_ancestors(root: &Element, target: &Element, opt_in_attr: &str) -> Vec<Element> {
        let selector = format!("[{opt_in_attr}]");
        let mut ancestors = Vec::new();
        let mut current = Some(target.clone());
        while let Some(from) = current {
            let Some(matched) = from.closest(&selector).ok().flatten() else {
                break;
            };
            if !root.contains(Some(&matched)) {
                break;
            }
            current = matched.parent_element();
            ancestors.push(matched);
        }
        ancestors
    }

    /// `pointerover`: 非タッチかつ真の進入（`related_target` が
    /// 各祖先の外）の祖先ごとに [`HOVER_STATE_ATTR`] を付与する。入れ子の
    /// opt-in 祖先すべてを対象にすることで、内側要素へ直接進入した場合に
    /// 外側祖先の hover 状態が更新されない不具合を避ける。
    fn handle_pointerover(root: &Element, event: &Event) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        if is_touch_pointer(&pointer_event.pointer_type()) {
            return;
        }
        let Some(target) = event_target_element(event) else {
            return;
        };
        for hover_target in opted_in_ancestors(root, &target, GESTURE_HOVER_ATTR) {
            if related_within(event, &hover_target) {
                continue;
            }
            let _ = set_dom_attribute(&hover_target, HOVER_STATE_ATTR, "");
        }
    }

    /// `pointerout`: 真の離脱（`related_target` が対象外）かつ非タッチの
    /// 祖先ごとに [`HOVER_STATE_ATTR`] を外す（入れ子の opt-in 祖先すべてが
    /// 対象、codex-review 指摘の是正）。非タッチ判定を追加するのは、
    /// マウスで hover 中の要素へタッチが重なると `pointerType: "touch"` の
    /// `pointerout` が発火し、無関係なマウス hover まで消してしまう
    /// 不具合の是正（`handle_pointerover` と対称にする、codex-review
    /// 指摘）。press 側は `active_pointer_press`（実際に pointerdown で
    /// 押下された要素）のみを対象にする（`opted_in_ancestors(root,
    /// &target, ...)` で祖先を再計算すると、親子とも opt-in の場合に
    /// pointerdown 時と異なる要素を解除してしまうため。codex-review
    /// 指摘の是正）。同一要素への複数ポインタ同時押下を考慮し、実際の
    /// 解除は [`release_pointer_press`] に委ねる。
    fn handle_pointerout(root: &Element, event: &Event, active_pointer_press: &ActivePress) {
        let Some(target) = event_target_element(event) else {
            return;
        };
        let is_touch = event
            .dyn_ref::<PointerEvent>()
            .is_some_and(|pointer_event| is_touch_pointer(&pointer_event.pointer_type()));
        if !is_touch {
            for hover_target in opted_in_ancestors(root, &target, GESTURE_HOVER_ATTR) {
                if !related_within(event, &hover_target) {
                    let _ = hover_target.remove_attribute(HOVER_STATE_ATTR);
                }
            }
        }
        let Some(pointer_id) = event_pointer_id(event) else {
            return;
        };
        let press_target = active_pointer_press.borrow().get(&pointer_id).cloned();
        let Some(press_target) = press_target else {
            return;
        };
        if !related_within(event, &press_target) {
            release_pointer_press(active_pointer_press, pointer_id);
        }
    }

    /// `pointerdown`: opt-in 要素へ [`PRESS_STATE_ATTR`] を付与する
    /// （pointer/touch 両方対象、タッチ除外は行わない）。あわせて
    /// [`PRESS_POINTER_ACTIVE_ATTR`] を立て、この press が pointer 由来
    /// であることを記録し、`active_pointer_press` に押下要素そのものを
    /// 保持する（`pointerup`/`pointerout` が pointerdown 時と同じ要素を
    /// 確実に解除できるようにするため。codex-review 指摘の是正: 親子とも
    /// opt-in の場合、離脱・解放イベントの `target` から祖先を再計算する
    /// と pointerdown 時に決めた要素と食い違いうる）。
    fn handle_pointerdown(root: &Element, event: &Event, active_pointer_press: &ActivePress) {
        let Some(target) = event_target_element(event) else {
            return;
        };
        let Some(press_target) = closest_opted_in(root, &target, GESTURE_PRESS_ATTR) else {
            return;
        };
        let Some(pointer_id) = event_pointer_id(event) else {
            return;
        };
        let _ = set_dom_attribute(&press_target, PRESS_STATE_ATTR, "");
        let _ = set_dom_attribute(&press_target, PRESS_POINTER_ACTIVE_ATTR, "");
        active_pointer_press
            .borrow_mut()
            .insert(pointer_id, press_target);
    }

    /// `pointerup`/`pointercancel`: `active_pointer_press`（pointerdown で
    /// 実際に押下された要素）を解放する（同一 [`Closure`] を両イベント名
    /// で登録しリスナー数を節約）。`event` の `target` から祖先を再計算
    /// しないのは `handle_pointerdown` と同じ理由（親子とも opt-in で
    /// 子上へドラッグして pointerup すると親の press が残留する不具合の
    /// 是正、codex-review 指摘）。実際の属性解除・エントリ削除は
    /// [`release_pointer_press`] に委ね、同一要素への 2 本目以降の
    /// ポインタが生きている間は press 状態属性を維持する。
    fn handle_pointerup_or_cancel(
        _root: &Element,
        event: &Event,
        active_pointer_press: &ActivePress,
    ) {
        let Some(pointer_id) = event_pointer_id(event) else {
            return;
        };
        release_pointer_press(active_pointer_press, pointer_id);
    }

    /// `keydown`: 活性化キー（Enter/Space）・非リピート・opt-in 要素の
    /// ときのみ [`PRESS_STATE_ATTR`] を付与する。`prevent_default()` は
    /// 呼ばない（ネイティブ要素への意図しない誤爆を避け、純粋な視覚状態
    /// シグナルに留める）。`event.target()` をそのまま信頼する（フォーカス
    /// 実体の有無を追加検証しない）: プログラム的な `dispatch_event` は
    /// 実フォーカスを伴わないため、`document.activeElement` による検証を
    /// 課すとテスト・プログラム的発火の双方で press が付与されなくなる
    /// （cursor(Bugbot) 指摘の是正。`handle_keyup`/`handle_focusout` も
    /// 同じく `target` を信頼する対称設計）。
    fn handle_keydown(root: &Element, event: &Event) {
        let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        if keyboard_event.repeat() || !is_press_activation_key(&keyboard_event.key()) {
            return;
        }
        let Some(target) = event_target_element(event) else {
            return;
        };
        let Some(press_target) = closest_opted_in(root, &target, GESTURE_PRESS_ATTR) else {
            return;
        };
        let _ = set_dom_attribute(&press_target, PRESS_STATE_ATTR, "");
    }

    /// `keyup`: 活性化キーの解放で [`PRESS_STATE_ATTR`] を外す。
    fn handle_keyup(root: &Element, event: &Event) {
        let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        if !is_press_activation_key(&keyboard_event.key()) {
            return;
        }
        let Some(target) = event_target_element(event) else {
            return;
        };
        let Some(press_target) = closest_opted_in(root, &target, GESTURE_PRESS_ATTR) else {
            return;
        };
        let _ = press_target.remove_attribute(PRESS_STATE_ATTR);
    }

    /// `focusout`: フォーカスが外れた要素から keyboard 活性化由来の
    /// [`PRESS_STATE_ATTR`] を外す。Space 押下中（keydown 済み）に Tab で
    /// フォーカス移動すると、後続の `keyup` はそのとき実際にフォーカスを
    /// 持つ別要素へ発火し元要素へは届かないため（codex-review 指摘）、
    /// `keyup` を待たずフォーカス離脱時点で確実に解除する。`focusout` は
    /// バブルするため root 委譲で拾える。
    ///
    /// pointer 由来（[`PRESS_POINTER_ACTIVE_ATTR`] を持つ）の press は
    /// ここでは一切解除しない。真の解除は必ず `pointerup`/`pointercancel`/
    /// `pointerout` にのみ委ねる（フォーカス変化は物理的なポインタの
    /// 押下状態を表さないため。Bugbot 指摘の是正: 押下中の祖先とは無関係な
    /// 子孫のフォーカス喪失〔`relatedTarget` が null になるだけの
    /// window blur 等〕で、祖先の生きた pointer press が誤って解除されて
    /// いた「Nested focusout clears live pointer press」）。
    ///
    /// keyboard 由来の press は `target` から祖先方向へ opt-in（press）
    /// 要素を**すべて**辿り（[`opted_in_ancestors`]）、真にフォーカスが
    /// 外れた祖先ごとに解除する。`closest_opted_in`（最も近い 1 件のみ）
    /// では、入れ子 opt-in（親・子とも opt-in）で子から focusout した
    /// とき、親自身が実際の press 保持者であっても子だけを見て親を見逃す
    /// 不具合があった。
    ///
    /// `press_target` が focusout の対象（`target`）そのものである場合は
    /// `relatedTarget` の位置に関わらず必ず解除する。keydown を受けた
    /// 要素自身が直接フォーカスを失った以上、対応する `keyup` は二度と
    /// この要素へは届かないため（codex-review 指摘の是正: 親要素自身が
    /// Space 押下後に子へフォーカス移動すると、`relatedTarget` が親配下
    /// である限り親の press が残留していた）。`press_target` が `target`
    /// の祖先（複合ウィジェット自身）である場合のみ、新フォーカス先
    /// （`relatedTarget`）がまだその祖先配下にあるなら解除しない
    /// （複合ウィジェット内でのフォーカス移動を離脱と誤判定しないため）。
    fn handle_focusout(root: &Element, event: &Event) {
        if event.dyn_ref::<FocusEvent>().is_none() {
            return;
        }
        let Some(target) = event_target_element(event) else {
            return;
        };
        for press_target in opted_in_ancestors(root, &target, GESTURE_PRESS_ATTR) {
            if press_target.has_attribute(PRESS_POINTER_ACTIVE_ATTR) {
                continue;
            }
            if press_target != target && related_within(event, &press_target) {
                continue;
            }
            let _ = press_target.remove_attribute(PRESS_STATE_ATTR);
        }
    }

    /// `root` へ hover/press 検知の 8 リスナー（pointerover/pointerout/
    /// pointerdown/pointerup/pointercancel/keydown/keyup/focusout）を
    /// 委譲登録する
    /// （[`crate::lib::Runtime::mount`]/[`crate::lib::Runtime::hydrate`]
    /// から呼ばれる）。`events::wire_events` と同じく登録回数を定数個に
    /// 抑える方針（A04 対策）で、要素ごとの動的登録・解除は行わない
    /// （動的挿入要素にも root 委譲で自動対応する）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` の失敗を伝播する。
    pub fn wire_gesture(root: Element) -> Result<(), JsValue> {
        // pointerdown/pointerup/pointercancel/pointerout/focusout が共有する
        // 「実際に press された要素」（`pointer_id` ごと）。フォーカス移動を
        // 伴わない複数ポインタの同時押下でも、それぞれ独立して解放できる。
        let active_pointer_press: ActivePress = Rc::new(RefCell::new(HashMap::new()));

        let pointerover_root = root.clone();
        let pointerover_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerover(&pointerover_root, &event);
        });
        root.add_event_listener_with_callback(
            "pointerover",
            pointerover_closure.as_ref().unchecked_ref(),
        )?;
        pointerover_closure.forget();

        let pointerout_root = root.clone();
        let pointerout_active = Rc::clone(&active_pointer_press);
        let pointerout_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerout(&pointerout_root, &event, &pointerout_active);
        });
        root.add_event_listener_with_callback(
            "pointerout",
            pointerout_closure.as_ref().unchecked_ref(),
        )?;
        pointerout_closure.forget();

        let pointerdown_root = root.clone();
        let pointerdown_active = Rc::clone(&active_pointer_press);
        let pointerdown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerdown(&pointerdown_root, &event, &pointerdown_active);
        });
        root.add_event_listener_with_callback(
            "pointerdown",
            pointerdown_closure.as_ref().unchecked_ref(),
        )?;
        pointerdown_closure.forget();

        let pointerup_root = root.clone();
        let pointerup_active = Rc::clone(&active_pointer_press);
        let pointerup_or_cancel_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerup_or_cancel(&pointerup_root, &event, &pointerup_active);
        });
        root.add_event_listener_with_callback(
            "pointerup",
            pointerup_or_cancel_closure.as_ref().unchecked_ref(),
        )?;
        root.add_event_listener_with_callback(
            "pointercancel",
            pointerup_or_cancel_closure.as_ref().unchecked_ref(),
        )?;
        pointerup_or_cancel_closure.forget();

        // `keydown` は capture フェーズで登録する（bubble ではなく）。
        // opt-in 要素自身のアプリケーションコードが keydown ハンドラ内で
        // 同期的に別要素へ `focus()` する場合、bubble フェーズ登録では
        // target 自身のハンドラ（先に実行される）が先に focus() を呼び、
        // focusout がこの press 設定より前に完了してしまい、その後
        // 設定された press が二度と解除されず残留する不具合があった
        // （codex-review 指摘の是正）。capture フェーズなら root への
        // このリスナーが常に target 自身のあらゆる bubble リスナーより
        // 先に実行されるため、`handle_keydown` が press を設定した時点で
        // 対象要素はまだ実際にフォーカスを保持しており、後続で
        // `focus()` が呼ばれても通常どおり `focusout`（`handle_focusout`）
        // が確実に解除できる。
        let keydown_root = root.clone();
        let keydown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_keydown(&keydown_root, &event);
        });
        root.add_event_listener_with_callback_and_bool(
            "keydown",
            keydown_closure.as_ref().unchecked_ref(),
            true,
        )?;
        keydown_closure.forget();

        let keyup_root = root.clone();
        let keyup_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_keyup(&keyup_root, &event);
        });
        root.add_event_listener_with_callback("keyup", keyup_closure.as_ref().unchecked_ref())?;
        keyup_closure.forget();

        let focusout_root = root.clone();
        let focusout_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_focusout(&focusout_root, &event);
        });
        root.add_event_listener_with_callback(
            "focusout",
            focusout_closure.as_ref().unchecked_ref(),
        )?;
        focusout_closure.forget();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_gesture;

#[cfg(test)]
mod tests {
    use super::{is_press_activation_key, is_touch_pointer};

    #[test]
    fn is_touch_pointer_matches_only_touch() {
        assert!(is_touch_pointer("touch"));
        assert!(!is_touch_pointer("mouse"));
        assert!(!is_touch_pointer("pen"));
        assert!(!is_touch_pointer(""));
    }

    #[test]
    fn is_press_activation_key_matches_enter_and_space() {
        assert!(is_press_activation_key("Enter"));
        assert!(is_press_activation_key(" "));
        assert!(!is_press_activation_key("Escape"));
        assert!(!is_press_activation_key("Spacebar"));
        assert!(!is_press_activation_key(""));
    }
}

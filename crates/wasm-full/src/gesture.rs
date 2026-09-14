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
        is_touch_pointer, GESTURE_HOVER_ATTR, GESTURE_PRESS_ATTR, HOVER_STATE_ATTR,
        PRESS_STATE_ATTR,
    };
    use crate::dom::set_dom_attribute_result as set_dom_attribute;
    use std::cell::RefCell;
    use std::collections::{HashMap, HashSet};
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, KeyboardEvent, MouseEvent, PointerEvent};

    /// 押下源の集合を要素単位で追跡する状態モデル（PR #2555 レビュー
    /// 指摘の是正で再設計。旧実装は press の解除判定に DOM 属性
    /// （撤去済みの `data-fandhe-press-pointer-active`）を使っており、
    /// pointerdown のハンドラ実行と focusout の発火順序に解除可否が
    /// 依存する競合を持っていた〔Bugbot「Nested focusout clears live
    /// pointer press」〕。本状態モデルは 2 つの独立した押下源を
    /// `Rc<RefCell<..>>` のみで管理し、DOM 属性の有無に一切依存しない:
    ///
    /// - **pointer 源**（[`ActivePointerPress`]）: `pointerdown` で
    ///   実際に press された要素を `PointerEvent::pointer_id()` ごとに
    ///   保持する。同一要素を 2 本指で押下した状態から片方だけ離しても、
    ///   もう片方の `pointer_id` が生きていれば要素はまだ「押下中」と
    ///   みなす（複数ポインタを横断した要素単位の集約）。
    /// - **keyboard 源**（[`ActiveKeyboardPress`]）: 一度に 1 件のみ
    ///   （実フォーカスは同時に 1 要素にしか存在しないため）。`keydown`
    ///   時点の実フォーカス要素（`origin`）と、実際に属性を受け取った
    ///   closest opt-in 祖先（`press_target`）を組で保持する。
    ///
    /// [`PRESS_STATE_ATTR`] の解除は必ず [`clear_press_if_unreferenced`]
    /// を経由し、pointer 源・keyboard 源のどちらの集合にもその要素が
    /// 残っていない場合のみ実行する。これにより「ポインタを押したまま
    /// Space を押して片方だけ離す」「同一要素を 2 本指で押して片方だけ
    /// 離す」のいずれも、残っている押下源を確認してから解除する契約を
    /// 満たす。
    type ActivePointerPress = Rc<RefCell<HashMap<i32, Element>>>;

    /// `keydown` に対応する keyboard 押下源。フィールドの役割:
    ///
    /// - `origin`: `keydown` イベントの実際の `event.target()`
    ///   （＝そのとき実フォーカスを持っていた要素）。`focusout` は
    ///   常にこの要素に対して発火するため、`handle_focusout` は
    ///   `origin` との一致だけを見て解除する（`relatedTarget` の位置や
    ///   祖先の入れ子構造は一切見ない）。複合ウィジェット（opt-in 祖先
    ///   配下に非 opt-in の子・opt-in の子が混在する構成）で、
    ///   `keydown` を受けた子から他の子へフォーカスが移っても、対応する
    ///   `keyup` は二度と `origin` へ届かないため、`origin` 自身の
    ///   `focusout` で確実に解除できる。
    /// - `press_target`: `closest_opted_in(root, origin, GESTURE_PRESS_ATTR)`
    ///   で実際に [`PRESS_STATE_ATTR`] を受け取った要素（`origin` 自身の
    ///   場合と、`origin` の opt-in 祖先である場合の両方がある）。
    /// - `active_keys`: 現在押下中の活性化キー（[`normalize_activation_key`]
    ///   が返す正規化済み文字列、`"Enter"`/`" "` のいずれか）の集合。
    ///   同一要素で Space と Enter を同時に押した状態から片方だけ離しても、
    ///   もう片方が `active_keys` に残っていれば keyboard 源は解除しない
    ///   （PR #2555 レビュー指摘の是正: 旧実装は活性化キーを区別せず、
    ///   どちらか一方の `keyup` だけで keyboard 源全体を解除していた）。
    ///   `active_keys` が空になった時点で初めて [`release_keyboard_press`]
    ///   が呼ばれる。
    struct KeyboardPress {
        origin: Element,
        press_target: Element,
        active_keys: HashSet<&'static str>,
    }

    type ActiveKeyboardPress = Rc<RefCell<Option<KeyboardPress>>>;

    /// 活性化キー（`super::is_press_activation_key` と同じ 2 語のみ）を
    /// [`KeyboardPress::active_keys`] のキーとして使う正規化済み文字列へ
    /// 変換する。`KeyboardEvent::key()` はすでにこの 2 語のいずれかである
    /// ことが呼び出し元で保証されるため、実質的には恒等変換だが、
    /// `&'static str` へ正規化することで `HashSet` に所有権を持たせない。
    fn normalize_activation_key(key: &str) -> Option<&'static str> {
        match key {
            "Enter" => Some("Enter"),
            " " => Some(" "),
            _ => None,
        }
    }

    /// `press_target` が pointer 源集合に含まれているか。
    fn pointer_active_on(active_pointer: &ActivePointerPress, press_target: &Element) -> bool {
        active_pointer
            .borrow()
            .values()
            .any(|element| element == press_target)
    }

    /// `press_target` が現在の keyboard 押下源と一致するか。
    fn keyboard_active_on(active_keyboard: &ActiveKeyboardPress, press_target: &Element) -> bool {
        active_keyboard
            .borrow()
            .as_ref()
            .is_some_and(|state| &state.press_target == press_target)
    }

    /// `press_target` を指す押下源（pointer/keyboard）が 1 件も残って
    /// いない場合のみ [`PRESS_STATE_ATTR`] を解除する（根本契約:
    /// 「pointer または keyboard 活性化中」という状態を横断的に確認
    /// してから解除する）。
    fn clear_press_if_unreferenced(
        press_target: &Element,
        active_pointer: &ActivePointerPress,
        active_keyboard: &ActiveKeyboardPress,
    ) {
        if !pointer_active_on(active_pointer, press_target)
            && !keyboard_active_on(active_keyboard, press_target)
        {
            let _ = press_target.remove_attribute(PRESS_STATE_ATTR);
        }
    }

    /// `active_pointer` から `pointer_id` エントリを取り除き、
    /// [`clear_press_if_unreferenced`] で横断的に解除可否を判定する。
    /// `pointerup`/`pointercancel`/`pointerout`（要素外への真の離脱）の
    /// いずれからも呼ばれる。
    fn release_pointer_press(
        active_pointer: &ActivePointerPress,
        active_keyboard: &ActiveKeyboardPress,
        pointer_id: i32,
    ) {
        let released = active_pointer.borrow_mut().remove(&pointer_id);
        let Some(released) = released else {
            return;
        };
        clear_press_if_unreferenced(&released, active_pointer, active_keyboard);
    }

    /// 現在の keyboard 押下源を解放し、[`clear_press_if_unreferenced`] で
    /// 横断的に解除可否を判定する。`keyup`・`focusout`（`origin` 一致時）・
    /// `keydown`（旧押下源が残っていた場合の防御的な事前解放）から呼ばれる。
    fn release_keyboard_press(
        active_pointer: &ActivePointerPress,
        active_keyboard: &ActiveKeyboardPress,
    ) {
        let released = active_keyboard.borrow_mut().take();
        let Some(released) = released else {
            return;
        };
        clear_press_if_unreferenced(&released.press_target, active_pointer, active_keyboard);
    }

    /// `event.target()` を `Element` として取得する（`Text` ノード等は
    /// `None`）。
    fn event_target_element(event: &Event) -> Option<Element> {
        event.target()?.dyn_into::<Element>().ok()
    }

    /// `event` の `relatedTarget` を `Element` として取得する
    /// （`pointerover`/`pointerout` の `MouseEvent`/`PointerEvent` のみ
    /// 対応。`focusout` は `origin` 一致のみで解除を判定するため
    /// `relatedTarget` を参照しない、上記状態モデルの doc 参照）。
    fn related_target_element(event: &Event) -> Option<Element> {
        let mouse = event.dyn_ref::<MouseEvent>()?;
        mouse.related_target()?.dyn_into::<Element>().ok()
    }

    /// `related`（`relatedTarget`）が `boundary`（含む）配下に留まって
    /// いるか。子要素間の内部移動を「離脱」と誤判定しないための境界判定
    /// （`sidebar::wiring` の `pointerover`/`pointerout` 判定と同型）。
    fn related_within(event: &Event, boundary: &Element) -> bool {
        related_target_element(event).is_some_and(|related| boundary.contains(Some(&related)))
    }

    /// `event` の `PointerEvent::pointer_id()`（`PointerEvent` でなければ
    /// `None`）。複数ポインタを区別して press 状態を独立管理するための
    /// キー（[`ActivePointerPress`] 参照）。
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
    /// 不具合の是正、codex-review 指摘）。hover（[`GESTURE_HOVER_ATTR`]）
    /// 側の `pointerover`/`pointerout` のみで使う（press 側は上記の
    /// 状態モデルが要素そのものを保持するため祖先再計算を行わない）。
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
    /// 不具合の是正（`handle_pointerover` と対称にする。マウス hover 中の
    /// タッチ重なりは `pointerout_from_touch_does_not_clear_mouse_hover`
    /// で回帰固定）。press 側は `active_pointer_press`（実際に pointerdown
    /// で押下された要素）のみを対象にする（`opted_in_ancestors(root,
    /// &target, ...)` で祖先を再計算すると、親子とも opt-in の場合に
    /// pointerdown 時と異なる要素を解除してしまうため。codex-review
    /// 指摘の是正）。press の解除はタッチ限定にせず（タッチのドラッグ
    /// 離脱でも press は解除する）、実際の解除は [`release_pointer_press`]
    /// に委ねる。
    fn handle_pointerout(
        root: &Element,
        event: &Event,
        active_pointer: &ActivePointerPress,
        active_keyboard: &ActiveKeyboardPress,
    ) {
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
        let press_target = active_pointer.borrow().get(&pointer_id).cloned();
        let Some(press_target) = press_target else {
            return;
        };
        if !related_within(event, &press_target) {
            release_pointer_press(active_pointer, active_keyboard, pointer_id);
        }
    }

    /// `pointerdown`: opt-in 要素へ [`PRESS_STATE_ATTR`] を付与し、
    /// `active_pointer_press` に押下要素そのものを保持する
    /// （`pointerup`/`pointerout` が pointerdown 時と同じ要素を確実に
    /// 解除できるようにするため。codex-review 指摘の是正: 親子とも
    /// opt-in の場合、離脱・解放イベントの `target` から祖先を再計算する
    /// と pointerdown 時に決めた要素と食い違いうる）。pointer/touch 両方
    /// 対象（タッチ除外は行わない）。
    ///
    /// 同一 `pointer_id` に対する既存エントリを、対応する解除処理なしに
    /// 上書きしない（Bugbot 指摘「Pointer id reuse leaks press」の是正）:
    /// ブラウザ・OS の実装によっては `pointerup`/`pointercancel` を
    /// 取りこぼした状態のまま同じ `pointer_id` が別要素へ再利用される
    /// ことがあり、旧実装は `insert` で単に上書きするだけだったため
    /// 最初に押下した要素の [`PRESS_STATE_ATTR`] が永続的に残留していた。
    /// `insert` の戻り値（上書きされた旧エントリ）を確認し、新しい
    /// 押下対象と異なる要素であれば [`clear_press_if_unreferenced`] 経由で
    /// 解除する（他の `pointer_id`・keyboard 源がまだその要素を参照して
    /// いれば解除しない、根本契約は変わらない）。
    fn handle_pointerdown(
        root: &Element,
        event: &Event,
        active_pointer: &ActivePointerPress,
        active_keyboard: &ActiveKeyboardPress,
    ) {
        let Some(target) = event_target_element(event) else {
            return;
        };
        let Some(press_target) = closest_opted_in(root, &target, GESTURE_PRESS_ATTR) else {
            return;
        };
        let Some(pointer_id) = event_pointer_id(event) else {
            return;
        };
        let previous = active_pointer
            .borrow_mut()
            .insert(pointer_id, press_target.clone());
        if let Some(previous) = previous {
            if previous != press_target {
                clear_press_if_unreferenced(&previous, active_pointer, active_keyboard);
            }
        }
        let _ = set_dom_attribute(&press_target, PRESS_STATE_ATTR, "");
    }

    /// `pointerup`/`pointercancel`: `active_pointer_press`（pointerdown で
    /// 実際に押下された要素）を解放する（同一 [`Closure`] を両イベント名
    /// で登録しリスナー数を節約）。`event` の `target` から祖先を再計算
    /// しないのは `handle_pointerdown` と同じ理由（親子とも opt-in で
    /// 子上へドラッグして pointerup すると親の press が残留する不具合の
    /// 是正、codex-review 指摘）。実際の属性解除・エントリ削除は
    /// [`release_pointer_press`] に委ね、同一要素への 2 本目以降の
    /// ポインタや keyboard 押下源が生きている間は press 状態属性を維持する。
    fn handle_pointerup_or_cancel(
        event: &Event,
        active_pointer: &ActivePointerPress,
        active_keyboard: &ActiveKeyboardPress,
    ) {
        let Some(pointer_id) = event_pointer_id(event) else {
            return;
        };
        release_pointer_press(active_pointer, active_keyboard, pointer_id);
    }

    /// `keydown`: 活性化キー（Enter/Space）・非リピート・opt-in 要素の
    /// ときのみ [`PRESS_STATE_ATTR`] を付与する。`prevent_default()` は
    /// 呼ばない（ネイティブ要素への意図しない誤爆を避け、純粋な視覚状態
    /// シグナルに留める）。`event.target()` をそのまま信頼する（フォーカス
    /// 実体の有無を追加検証しない）: プログラム的な `dispatch_event` は
    /// 実フォーカスを伴わないため、`document.activeElement` による検証を
    /// 課すとテスト・プログラム的発火の双方で press が付与されなくなる
    /// （cursor(Bugbot) 指摘の是正。`handle_keyup`/`handle_focusout` も
    /// 同じく `target`/追跡済み状態を信頼する対称設計）。`event.target()`
    /// を [`KeyboardPress::origin`] として保持し、以後の解除判定は
    /// この値のみを見る（`focusout`/`keyup` が別要素の DOM 位置を
    /// 再計算しない、上記状態モデルの doc 参照）。
    ///
    /// 同一 `origin` に対する `keydown` であれば、既存の
    /// [`KeyboardPress::active_keys`] へ活性化キーを追加するだけで
    /// 押下源自体は作り直さない（codex-review 指摘の是正: 同じ要素で
    /// Space を押したまま Enter も押すと、上書きでは Space の押下記録が
    /// 失われ、後続の Space の `keyup` で誤って解除されてしまう）。
    /// `origin` が異なる（通常は起きないが `keyup`/`focusout` を経ずに
    /// 別要素へ新たな `keydown` を受けた場合の防御的な処理）場合のみ、
    /// 旧押下源を先に解放してから新規に作り直す。
    fn handle_keydown(
        root: &Element,
        event: &Event,
        active_pointer: &ActivePointerPress,
        active_keyboard: &ActiveKeyboardPress,
    ) {
        let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        if keyboard_event.repeat() {
            return;
        }
        let Some(key) = normalize_activation_key(&keyboard_event.key()) else {
            return;
        };
        let Some(origin) = event_target_element(event) else {
            return;
        };
        let Some(press_target) = closest_opted_in(root, &origin, GESTURE_PRESS_ATTR) else {
            return;
        };

        let same_origin_active = active_keyboard
            .borrow()
            .as_ref()
            .is_some_and(|state| state.origin == origin);
        if same_origin_active {
            if let Some(state) = active_keyboard.borrow_mut().as_mut() {
                state.active_keys.insert(key);
            }
        } else {
            release_keyboard_press(active_pointer, active_keyboard);
            let mut active_keys = HashSet::new();
            active_keys.insert(key);
            *active_keyboard.borrow_mut() = Some(KeyboardPress {
                origin,
                press_target: press_target.clone(),
                active_keys,
            });
        }
        let _ = set_dom_attribute(&press_target, PRESS_STATE_ATTR, "");
    }

    /// `keyup`: 活性化キーの解放で、対応するキーだけを
    /// [`KeyboardPress::active_keys`] から取り除く。取り除いた結果
    /// `active_keys` が空になったときのみ [`release_keyboard_press`] で
    /// keyboard 押下源全体を解放する（codex-review 指摘の是正: 同一要素で
    /// Space と Enter を同時に押した状態から片方だけ離しても、もう片方が
    /// 活性化中なら press を解除しない）。`event.target()` から祖先を
    /// 再計算しない（`handle_keydown` が保持した `press_target` をそのまま
    /// 使う）: フォーカスが元の要素から別要素へ移動した状態で `keyup` が
    /// 新フォーカス先へ発火しても、追跡済みの押下源を正しく解放できる
    /// （codex-review 指摘の是正）。
    fn handle_keyup(
        event: &Event,
        active_pointer: &ActivePointerPress,
        active_keyboard: &ActiveKeyboardPress,
    ) {
        let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        let Some(key) = normalize_activation_key(&keyboard_event.key()) else {
            return;
        };
        let should_release = {
            let mut keyboard = active_keyboard.borrow_mut();
            match keyboard.as_mut() {
                Some(state) => {
                    state.active_keys.remove(key);
                    state.active_keys.is_empty()
                }
                None => false,
            }
        };
        if should_release {
            release_keyboard_press(active_pointer, active_keyboard);
        }
    }

    /// `focusout`: `event.target()` が現在の keyboard 押下源の
    /// [`KeyboardPress::origin`] と一致するときのみ、無条件に（
    /// `relatedTarget` の位置を一切見ずに）解放する。Space 押下中
    /// （keydown 済み）にフォーカスが移動すると、後続の `keyup` は
    /// そのとき実際にフォーカスを持つ別要素へ発火し `origin` へは
    /// 二度と届かないため（codex-review 指摘）、`keyup` を待たず
    /// フォーカス離脱時点で確実に解除する。`focusout` はバブルするため
    /// root 委譲で拾える。
    ///
    /// `origin` 一致のみを条件にすることで、以下をまとめて解決する:
    ///
    /// - 親自身が press opt-in で Space を受けてから子へフォーカスが
    ///   移動しても（`relatedTarget` が親配下でも）、親自身が `origin`
    ///   である以上、必ず解除する（旧実装は `relatedTarget` が
    ///   `press_target` 配下なら解除を省略しており、親の press が
    ///   残留していた）。
    /// - 複合ウィジェット（opt-in 祖先の配下に非 opt-in の子・opt-in の
    ///   子が混在）で、`keydown` を受けた非 opt-in の子から他の子へ
    ///   フォーカスが移動しても、その子自身が `origin` である以上、
    ///   `press_target`（opt-in 祖先）を確実に解除する（旧実装は
    ///   `relatedTarget` が opt-in 祖先の配下に留まる限り解除を
    ///   省略しており、祖先の press が残留していた）。
    /// - pointer 由来の press には一切触れない（`active_pointer` を
    ///   参照しない設計であり、DOM 属性の有無や祖先の入れ子構造にも
    ///   依存しないため、pointerdown のハンドラ実行と focusout の発火
    ///   順序に依存する競合が構造的に発生しない。Bugbot 指摘
    ///   「Nested focusout clears live pointer press」の是正）。真の
    ///   pointer press 解除は必ず `pointerup`/`pointercancel`/
    ///   `pointerout` にのみ委ねる。
    ///
    /// `wire_gesture` は本ハンドラも `keydown`/`keyup` と同じく capture
    /// フェーズで登録する（codex-review 指摘の是正）: opt-in 子で Space を
    /// 押した後、その子自身の `focusout` ハンドラ（アプリケーションコード）
    /// が `stopPropagation()` を呼びつつフォーカスが `root` の外へ完全に
    /// 抜けると、bubble 登録では root まで伝播せず本ハンドラが一切
    /// 呼ばれないため keyboard 押下源が永続的に残留していた。capture
    /// フェーズなら root のリスナーは子孫のあらゆるリスナーより先に
    /// 実行されるため、子孫が事後に `stopPropagation()` を呼んでも解除
    /// 処理には影響しない。
    fn handle_focusout(
        event: &Event,
        active_pointer: &ActivePointerPress,
        active_keyboard: &ActiveKeyboardPress,
    ) {
        let Some(target) = event_target_element(event) else {
            return;
        };
        let is_origin = active_keyboard
            .borrow()
            .as_ref()
            .is_some_and(|state| state.origin == target);
        if is_origin {
            release_keyboard_press(active_pointer, active_keyboard);
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
    /// 全リスナーを capture フェーズで登録する（bubble ではなく、PR #2555
    /// レビュー指摘の是正）: 子孫（opt-in 要素自身を含む）が自前のイベント
    /// ハンドラで `stopPropagation()` を呼ぶと、bubble 登録では root まで
    /// 伝播せず対応するハンドラが一切呼ばれず、press/hover 状態が残留する
    /// （Bugbot 指摘「Pointer press sticks after stop」「Keydown capture
    /// leaves stale press」、および `focusout` の同型不具合、いずれも
    /// codex-review でも指摘）。capture フェーズなら root のリスナーは
    /// 子孫のあらゆるリスナーより先に実行されるため、子孫が事後に
    /// `stopPropagation()` を呼んでも解除処理には影響しない。`pointerover`/
    /// `pointerout`/`pointerdown`/`pointerup`/`pointercancel` は互いに
    /// 対称性を保つため足並みを揃えて全て capture 化する
    /// （`pointerover`/`pointerdown` 自体はレビュー指摘の直接対象ではない
    /// が、`stopPropagation()` に対する頑健性は他のポインタイベントと
    /// 統一しておくべき性質のため）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback_and_bool` の失敗を伝播する。
    pub fn wire_gesture(root: Element) -> Result<(), JsValue> {
        // pointerdown/pointerup/pointercancel/pointerout/keyup/focusout が
        // 共有する押下源の集約状態（上記状態モデルの doc 参照）。
        let active_pointer_press: ActivePointerPress = Rc::new(RefCell::new(HashMap::new()));
        let active_keyboard_press: ActiveKeyboardPress = Rc::new(RefCell::new(None));

        let pointerover_root = root.clone();
        let pointerover_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerover(&pointerover_root, &event);
        });
        root.add_event_listener_with_callback_and_bool(
            "pointerover",
            pointerover_closure.as_ref().unchecked_ref(),
            true,
        )?;
        pointerover_closure.forget();

        let pointerout_root = root.clone();
        let pointerout_pointer = Rc::clone(&active_pointer_press);
        let pointerout_keyboard = Rc::clone(&active_keyboard_press);
        let pointerout_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerout(
                &pointerout_root,
                &event,
                &pointerout_pointer,
                &pointerout_keyboard,
            );
        });
        root.add_event_listener_with_callback_and_bool(
            "pointerout",
            pointerout_closure.as_ref().unchecked_ref(),
            true,
        )?;
        pointerout_closure.forget();

        let pointerdown_root = root.clone();
        let pointerdown_pointer = Rc::clone(&active_pointer_press);
        let pointerdown_keyboard = Rc::clone(&active_keyboard_press);
        let pointerdown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerdown(
                &pointerdown_root,
                &event,
                &pointerdown_pointer,
                &pointerdown_keyboard,
            );
        });
        root.add_event_listener_with_callback_and_bool(
            "pointerdown",
            pointerdown_closure.as_ref().unchecked_ref(),
            true,
        )?;
        pointerdown_closure.forget();

        let pointerup_pointer = Rc::clone(&active_pointer_press);
        let pointerup_keyboard = Rc::clone(&active_keyboard_press);
        let pointerup_or_cancel_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerup_or_cancel(&event, &pointerup_pointer, &pointerup_keyboard);
        });
        root.add_event_listener_with_callback_and_bool(
            "pointerup",
            pointerup_or_cancel_closure.as_ref().unchecked_ref(),
            true,
        )?;
        root.add_event_listener_with_callback_and_bool(
            "pointercancel",
            pointerup_or_cancel_closure.as_ref().unchecked_ref(),
            true,
        )?;
        pointerup_or_cancel_closure.forget();

        let keydown_root = root.clone();
        let keydown_pointer = Rc::clone(&active_pointer_press);
        let keydown_keyboard = Rc::clone(&active_keyboard_press);
        let keydown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_keydown(&keydown_root, &event, &keydown_pointer, &keydown_keyboard);
        });
        root.add_event_listener_with_callback_and_bool(
            "keydown",
            keydown_closure.as_ref().unchecked_ref(),
            true,
        )?;
        keydown_closure.forget();

        // `keyup` も `keydown` と対称に capture フェーズで登録する。
        // bubble 登録のままだと、opt-in 要素の子孫が自前の `keyup`
        // ハンドラで `stopPropagation()` すると root まで伝播せず
        // `handle_keyup` が呼ばれずに press が残留する（Bugbot 指摘
        // 「Keydown capture leaves stale press」の是正）。capture
        // フェーズなら root のリスナーは子孫のバブルフェーズハンドラ
        // より先に実行されるため、子孫が事後に `stopPropagation()` を
        // 呼んでも解除処理には影響しない。
        let keyup_pointer = Rc::clone(&active_pointer_press);
        let keyup_keyboard = Rc::clone(&active_keyboard_press);
        let keyup_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_keyup(&event, &keyup_pointer, &keyup_keyboard);
        });
        root.add_event_listener_with_callback_and_bool(
            "keyup",
            keyup_closure.as_ref().unchecked_ref(),
            true,
        )?;
        keyup_closure.forget();

        let focusout_pointer = Rc::clone(&active_pointer_press);
        let focusout_keyboard = Rc::clone(&active_keyboard_press);
        let focusout_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_focusout(&event, &focusout_pointer, &focusout_keyboard);
        });
        root.add_event_listener_with_callback_and_bool(
            "focusout",
            focusout_closure.as_ref().unchecked_ref(),
            true,
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

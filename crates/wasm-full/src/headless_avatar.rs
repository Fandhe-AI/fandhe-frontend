//! Avatar（`fandhe-frontend-headless-ui` `avatar` モジュール）の `load`/`error`
//! イベント検知グルー（イシュー #591、親 #520/#542/#543）。
//!
//! `crates/headless-ui/src/avatar.rs` は Root/Image/Fallback の 3 anatomy
//! パーツと `ImageStatus`（loading/loaded/error）状態機械を提供する一方、
//! クライアント側で実 DOM の `img` 要素の `load`/`error` イベントを検知して
//! dispatch（`"loaded"`/`"error"`）へ橋渡しする配線は同モジュール冒頭の
//! rustdoc「スコープ外」節が明記するとおり本クレート（wasm 層）の後続
//! スコープとされていた。本モジュールがそのグルーを実装する。
//!
//! # Runtime への統合（イシュー #711）
//!
//! [`wire_avatar_events`] 単体は `crate::lib::Runtime::mount`/`Runtime::hydrate`
//! から自動配線されない独立配線 API として先行実装された（#591）。イシュー
//! #711 で `Runtime::mount`/`Runtime::hydrate` 双方（`crate::lib` 参照）が
//! `keynav::wire_keynav` の直後に本モジュールの配線を標準経路へ組み込み、
//! アプリ側の手動配線を要さずに Avatar の `ImageStatus`／`data-state` が
//! 更新されるようにした。`events`/`keynav` と同じ「マウント時 1 回」契約は
//! 維持される（`Runtime::wire_avatar` 参照）。
//!
//! # `events.rs`/`keynav.rs`/`overlay.rs` との責務分離
//!
//! [`crate::events`] のクリック/入力委譲、[`crate::keynav`] のキーボード
//! 操作配線、[`crate::overlay`] の Escape/外側クリックによる閉鎖制御と同じ
//! 2 層構成（DOM 非依存の純粋ロジック層 + `#[cfg(target_arch = "wasm32")]`
//! 配線層）を踏襲する。ただし本モジュールが扱う `load`/`error` イベントは
//! （click/keydown とは異なり）**バブリングしない**ため、`root` への委譲は
//! バブリングフェーズではなく **capture フェーズ**
//! （`add_event_listener_with_callback_and_bool(..., true)`）で行う。
//! これが本モジュールを `events.rs` の委譲リスナーへ単純に相乗りできない
//! 理由であり、独立モジュールとして切り出す設計上の根拠である。
//!
//! # `src` 差し替え検知（イシュー #731）
//!
//! `wire_avatar_events` は `load`/`error` イベント検知グルーに加えて、
//! `MutationObserver`（`attributes: true` + `attributeFilter: ["src"]` +
//! `subtree: true`）を `root` へ登録し、Avatar image 要素の `src` 属性差し
//! 替えを検知して `"reset"`（→ `ImageStatus::Loading`）を自動 dispatch する
//! （[`avatar_action_for_src_mutation`] が判定を担う純粋関数、
//! `wiring::wire_avatar_src_observer` が配線を担う）。クライアント側で
//! `img.src` を動的に差し替えた場合（署名付き URL の遅延差し込み・再描画に
//! よる束縛点更新等）に、旧画像の `Loaded`/`Error` 状態を引きずらず新画像の
//! 読み込み中を正しく `Loading` へ戻す（ark-ui/Zag.js と同じ挙動）。
//! `subtree: true` により、再描画で `img` 要素自体が入れ替わっても
//! （`root` 直下ではなくネストしていても）新要素の `src` 変異を検知できる。
//! `attributeFilter: ["src"]` により、reset 後に `apply_avatar_visibility`
//! が書き込む `data-state`/`hidden` 属性の変異は observer を再発火させない
//! （無限ループの構造的な回避）。
//!
//! # 他クレート・他モジュールとの契約
//!
//! - [`avatar_action_for_image_event`] が判定するアクション名
//!   （`"loaded"`/`"error"`）は `fandhe_frontend_headless_ui::avatar::Avatar::decode_action`
//!   の対応する分岐と一致する。[`avatar_action_for_src_mutation`] が判定する
//!   `"reset"` も同分岐と一致する（上記「`src` 差し替え検知」節参照）。
//! - [`image_visible_after_action`] は
//!   `fandhe_frontend_headless_ui::avatar::ImageStatus::is_image_visible` と同一の
//!   可視性規則を文字列語彙（`"loaded"`/`"error"`/`"reset"`）で表現する。
//!   本クレートは `fandhe-frontend-headless-ui` を製品依存に持たない
//!   （`[dependencies]` ではなく `[dev-dependencies]` のみ）ため、規則の複製を
//!   native テスト側のドリフト検知（`wasm-full/tests/headless_avatar.rs`）で
//!   固定する。
//! - [`wire_avatar_events`]/[`apply_avatar_visibility`] は状態更新・DOM 反映の
//!   いずれも HTML 文字列を組み立てない（REQ-1）。DOM 反映は
//!   `set_attribute`/`remove_attribute` のみで、属性名・属性値はすべて
//!   `&'static str` リテラル（不変条件、`.claude/rules/coding-rust.md`）。

/// Avatar の `data-scope` 属性値（`fandhe_frontend_headless_ui::avatar` の
/// `ANATOMY` と一致、`crates/headless-ui/src/avatar.rs` 参照）。
const AVATAR_SCOPE: &str = "avatar";
/// Avatar Image パーツの `data-part` 属性値。
const AVATAR_IMAGE_PART: &str = "image";
/// Avatar Fallback パーツの `data-part` 属性値。
///
/// 純粋ロジック層（[`avatar_action_for_image_event`]）は fallback 上の
/// イベントを扱わないため参照しない。wasm32 配線層
/// （`wiring::apply_avatar_visibility`）と native テストのみが参照するため、
/// native の非テストビルドでは未使用と誤検出される（[`DATA_STATE_VISIBLE`]
/// と同じ理由の dead_code 抑制）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const AVATAR_FALLBACK_PART: &str = "fallback";

/// `data-state` 属性値 "visible"（`fandhe_frontend_headless_ui::avatar` と同一語彙）。
///
/// wasm32 配線層（`wiring::apply_avatar_visibility`）専用の定数だが、
/// クレート冒頭の crate-level rustdoc から `pub use` 未経由でも参照可能な
/// 位置に置くため、native の非テストビルドでは未使用と誤検出される
/// （`hydration.rs::MAX_ATTR_VALUE_LEN` と同じ理由の dead_code 抑制）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const DATA_STATE_VISIBLE: &str = "visible";
/// `data-state` 属性値 "hidden"（同上）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const DATA_STATE_HIDDEN: &str = "hidden";

/// dispatch アクション名 "loaded"。
const ACTION_LOADED: &str = "loaded";
/// dispatch アクション名 "error"。
const ACTION_ERROR: &str = "error";
/// dispatch アクション名 "reset"。呼び出し側 UI が `Reset` アクションを
/// dispatch した後の DOM 反映に [`image_visible_after_action`]/
/// [`apply_avatar_visibility`] を再利用できるようにするための語彙であると
/// 同時に、[`avatar_action_for_src_mutation`] が `src` 属性差し替え検知時に
/// 自動発火するアクション名でもある（イシュー #731、モジュール doc
/// 「`src` 差し替え検知」参照）。
const ACTION_RESET: &str = "reset";

/// `img`/`error` イベントのターゲット属性から dispatch すべきアクションを
/// 判定する（DOM 非依存の純粋関数、native `cargo test` で検証可能）。
///
/// ターゲットが `data-scope="avatar"` かつ `data-part="image"` の場合のみ
/// 判定を行う（改ざんされた `data-*` 値を持つ無関係要素上のイベントを
/// dispatch へ流さない fail-closed 不変条件）。`event_type` が `"load"`/
/// `"error"` 以外、または `scope`/`part` が一致しない場合はすべて `None`。
///
/// `payload` は常に空文字列（`AvatarAction::Loaded`/`AvatarAction::Error` は
/// payload を使わない、`crates/headless-ui/src/avatar.rs` の `decode_action`
/// 参照）。
#[must_use]
pub fn avatar_action_for_image_event(
    event_type: &str,
    scope: Option<&str>,
    part: Option<&str>,
) -> Option<crate::events::ActionRef> {
    if scope != Some(AVATAR_SCOPE) || part != Some(AVATAR_IMAGE_PART) {
        return None;
    }
    let action = match event_type {
        "load" => ACTION_LOADED,
        "error" => ACTION_ERROR,
        _ => return None,
    };
    Some(crate::events::ActionRef {
        action: action.to_string(),
        payload: String::new(),
    })
}

/// `MutationObserver` が検知した属性変異から dispatch すべきアクションを
/// 判定する（DOM 非依存の純粋関数、native `cargo test` で検証可能。
/// イシュー #731）。
///
/// ターゲットが `data-scope="avatar"` かつ `data-part="image"` の場合のみ
/// 判定を行い、変異した属性が `"src"` の場合のみ `"reset"` を返す
/// （[`avatar_action_for_image_event`] と同型の fail-closed ガード。
/// 改ざんされた `data-*` 値を持つ無関係要素・`src` 以外の属性変異は
/// dispatch へ流さない）。`attribute_name` は `MutationRecord::attribute_name()`
/// の戻り値（`Option<String>`）をそのまま受け取る想定。
///
/// `payload` は常に空文字列（`AvatarAction::Reset` は payload を使わない、
/// `crates/headless-ui/src/avatar.rs` の `decode_action` 参照）。
#[must_use]
pub fn avatar_action_for_src_mutation(
    attribute_name: Option<&str>,
    scope: Option<&str>,
    part: Option<&str>,
) -> Option<crate::events::ActionRef> {
    if attribute_name != Some("src") {
        return None;
    }
    if scope != Some(AVATAR_SCOPE) || part != Some(AVATAR_IMAGE_PART) {
        return None;
    }
    Some(crate::events::ActionRef {
        action: ACTION_RESET.to_string(),
        payload: String::new(),
    })
}

/// 配線時点で読み込みが既に決着済みの画像に対する合成 dispatch 判定。
///
/// `wire_avatar_events` がマウント時に root 配下の Avatar image 要素を走査し、
/// 本関数が `Some` を返した画像に対して `load`/`error` イベントを待たずに
/// 即座に合成 dispatch を行う。wasm 初期化・hydration 復元より前に画像
/// 読み込みが完了して `load`/`error` イベントがもう発火しないレース
/// （hydration 後の接続で最も典型的な取りこぼし）を塞ぐための判定。
///
/// - `complete && natural_width > 0` → `Some("loaded")`
/// - `complete && natural_width == 0` → `Some("error")`（ark-ui/Zag.js と
///   同じヒューリスティック。SVG 画像は仕様上 `naturalWidth` が常に `0` を
///   返す場合がある既知のエッジケースであり、そのような画像は本判定では
///   `error` 扱いになる。より精密な判定が必要になった場合は Issue 化を検討
///   する）
/// - `!complete` → `None`（以後の `load`/`error` イベントに判定を委ねる）
#[must_use]
pub fn avatar_action_for_settled_image(complete: bool, natural_width: u32) -> Option<&'static str> {
    if !complete {
        return None;
    }
    if natural_width > 0 {
        Some(ACTION_LOADED)
    } else {
        Some(ACTION_ERROR)
    }
}

/// dispatch 後の DOM 反映（`apply_avatar_visibility`）が使う、
/// アクション名から画像の可視性への変換。
///
/// `fandhe_frontend_headless_ui::avatar::ImageStatus::is_image_visible` と同一の
/// 可視性規則を文字列語彙で表現する（本クレートは `fandhe-frontend-headless-ui`
/// を製品依存に持たないため文字列で複製し、ドリフトは
/// `wasm-full/tests/headless_avatar.rs` の native テストで検知する）。
///
/// - `"loaded"` → `Some(true)`（image 表示・fallback 非表示）
/// - `"error"`/`"reset"` → `Some(false)`（image 非表示・fallback 表示。
///   `"reset"` は loading 状態相当であり、loading では image は非表示扱い）
/// - 未知のアクション名 → `None`（呼び出し側は DOM 反映をスキップする）
#[must_use]
pub fn image_visible_after_action(action: &str) -> Option<bool> {
    match action {
        ACTION_LOADED => Some(true),
        ACTION_ERROR | ACTION_RESET => Some(false),
        _ => None,
    }
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`events.rs`/`keynav.rs`/`overlay.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        avatar_action_for_image_event, avatar_action_for_settled_image,
        avatar_action_for_src_mutation, AVATAR_FALLBACK_PART, AVATAR_IMAGE_PART, AVATAR_SCOPE,
        DATA_STATE_HIDDEN, DATA_STATE_VISIBLE,
    };
    use crate::events::ActionRef;
    use fandhe_frontend_interactive::Component;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{
        Element, Event, HtmlImageElement, MutationObserver, MutationObserverInit, MutationRecord,
    };

    /// `[data-scope="avatar"][data-part="image"]` セレクタ（settle 検査の
    /// 走査対象を Avatar image 要素のみに絞る。有界な走査であり
    /// DoS 耐性を持つ）。
    const AVATAR_IMAGE_SELECTOR: &str = "[data-scope=\"avatar\"][data-part=\"image\"]";

    /// `root` 配下の Avatar image 要素（複数可）を出現順に集める。
    /// `query_selector_all` の失敗は空 `Vec` として扱う
    /// （fail-closed、panic しない。`keynav.rs::collect_parts` と同じ方針）。
    fn collect_avatar_images(root: &Element) -> Vec<HtmlImageElement> {
        let Ok(node_list) = root.query_selector_all(AVATAR_IMAGE_SELECTOR) else {
            return Vec::new();
        };
        let len = node_list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            if let Some(node) = node_list.get(i) {
                if let Ok(img) = node.dyn_into::<HtmlImageElement>() {
                    out.push(img);
                }
            }
        }
        out
    }

    /// `root` 配下の `[data-scope="avatar"][data-part="image"/"fallback"]` へ
    /// `data-state`（`"visible"`/`"hidden"`）と `hidden` 存在属性を反映する。
    ///
    /// dispatch 成功後の DOM 反映ヘルパ（受け入れ条件 1）。
    /// `fandhe_frontend_core::render` を経由した再描画は行わず、既存 DOM の
    /// 属性のみを書き換える（`set_attribute`/`remove_attribute` のみ。
    /// HTML 文字列組み立て・`innerHTML` は一切使わない、REQ-1）。
    /// 複数の Avatar が `root` 配下にネストしている場合は全て同じ
    /// `image_visible` へ揃える（1 root : 1 状態機械契約、モジュール doc
    /// 「スコープ境界」参照。複数 Avatar の個別追跡は別スコープ）。
    ///
    /// # Errors
    ///
    /// `query_selector_all`/`set_attribute`/`remove_attribute` が失敗した
    /// 場合に `Err` を返す。呼び出し側は panic せず伝播する契約
    /// （`.claude/rules/coding-rust.md`）。
    pub fn apply_avatar_visibility(root: &Element, image_visible: bool) -> Result<(), JsValue> {
        let (image_state, fallback_state) = if image_visible {
            (DATA_STATE_VISIBLE, DATA_STATE_HIDDEN)
        } else {
            (DATA_STATE_HIDDEN, DATA_STATE_VISIBLE)
        };
        apply_part_visibility(root, AVATAR_IMAGE_PART, image_state, image_visible)?;
        apply_part_visibility(root, AVATAR_FALLBACK_PART, fallback_state, !image_visible)?;
        Ok(())
    }

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （イシュー #401 の `fw gate` `url_validation_check` 契約に準拠、
    /// `.claude/rules/security.md`）。本モジュールが書き込む属性
    /// （`data-state`/`hidden`）はいずれも `&'static str` リテラルで固定
    /// された非 URL・非イベントハンドラ属性であり実害はないが、
    /// `fandhe_frontend_core::url` のガード関数群
    /// （`is_event_handler_attr`/`is_url_attr`/`is_safe_url`/
    /// `is_safe_srcset`）を経由することで、将来 `name`/`value` が動的な
    /// 入力から組み立てられるよう変更された場合の防御としても機能する
    /// （`keynav.rs::wiring::set_dom_attribute` と同じガード方針）。
    fn set_dom_attribute(element: &Element, name: &str, value: &str) -> Result<(), JsValue> {
        if fandhe_frontend_core::is_event_handler_attr(name) {
            return Ok(());
        }
        if fandhe_frontend_core::is_url_attr(name) && !fandhe_frontend_core::is_safe_url(value) {
            return Ok(());
        }
        if name.eq_ignore_ascii_case("srcset") && !fandhe_frontend_core::is_safe_srcset(value) {
            return Ok(());
        }
        element.set_attribute(name, value)
    }

    /// [`apply_avatar_visibility`] の内部ヘルパ。指定パートの `data-state` と
    /// `hidden` 存在属性を反映する。
    fn apply_part_visibility(
        root: &Element,
        part: &str,
        data_state: &'static str,
        visible: bool,
    ) -> Result<(), JsValue> {
        let selector = format!("[data-scope=\"{AVATAR_SCOPE}\"][data-part=\"{part}\"]");
        let Ok(node_list) = root.query_selector_all(&selector) else {
            return Ok(());
        };
        let len = node_list.length();
        for i in 0..len {
            let Some(node) = node_list.get(i) else {
                continue;
            };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            set_dom_attribute(&element, "data-state", data_state)?;
            if visible {
                element.remove_attribute("hidden")?;
            } else {
                set_dom_attribute(&element, "hidden", "")?;
            }
        }
        Ok(())
    }

    /// `root` 配下の Avatar image 要素へ `load`/`error` イベント検知グルーを
    /// マウント時に 1 回だけ配線する。
    ///
    /// `load`/`error` はバブリングしないため、`root` への委譲リスナーは
    /// **capture フェーズ**（`add_event_listener_with_callback_and_bool(...,
    /// true)`）で登録する（モジュール doc「`events.rs`/`keynav.rs`/
    /// `overlay.rs` との責務分離」参照）。capture フェーズは伝播パス上の
    /// 祖先で非バブリングイベントも受信できるため、再描画で `img` が
    /// 入れ替わっても `root` のリスナーは保持されたまま新しい `img` の
    /// イベントも受信できる。
    ///
    /// `Closure::forget` は `load`/`error`/`MutationObserver` コールバックの
    /// **3 回のみ**に限定する（イシュー #731 で 2 回から 3 回へ増加。
    /// `events.rs::wire_events` と同じ「マウント時 1 回・定数個リーク」契約は
    /// 維持される。A04: 安全でない設計への対策、無制限リークによるメモリ枯渇
    /// DoS の構造的回避）。
    ///
    /// 配線と同時に、`root` 配下で **既に読み込みが決着済み**の Avatar image
    /// （wasm 初期化前に `load`/`error` が発火し終えているケース）を
    /// [`avatar_action_for_settled_image`] で検査し、決着済みのものには
    /// イベントを待たず即座に合成 dispatch を行う（受け入れ条件 2 の中核。
    /// hydration 復元後の接続で最も典型的なレースを塞ぐ）。
    ///
    /// さらに `root` へ [`wire_avatar_src_observer`] で `MutationObserver` を
    /// 登録し、Avatar image の `src` 属性差し替えを検知して `"reset"` を
    /// 自動 dispatch する（イシュー #731、モジュール doc「`src` 差し替え
    /// 検知」参照）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback_and_bool`・
    /// `MutationObserver::observe_with_options` の失敗を伝播する。
    pub fn wire_avatar_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let on_action = std::rc::Rc::new(std::cell::RefCell::new(on_action));
        let load_root = root.clone();
        let load_on_action = on_action.clone();
        let error_root = root.clone();
        let error_on_action = on_action.clone();

        let load_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_image_event(&load_root, &event, &load_on_action);
        });
        root.add_event_listener_with_callback_and_bool(
            "load",
            load_closure.as_ref().unchecked_ref(),
            true,
        )?;
        load_closure.forget();

        let error_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_image_event(&error_root, &event, &error_on_action);
        });
        root.add_event_listener_with_callback_and_bool(
            "error",
            error_closure.as_ref().unchecked_ref(),
            true,
        )?;
        error_closure.forget();

        // settle 検査: 配線時点で既に決着済みの画像には合成 dispatch を行う。
        // `try_borrow_mut` 失敗（配線処理内からの再入は通常発生しないが、
        // 呼び出し側コールバックの実装次第では起こり得るため防御的に
        // no-op とする、panic 回避）。
        for img in collect_avatar_images(&root) {
            if let Some(action) =
                avatar_action_for_settled_image(img.complete(), img.natural_width())
            {
                if let Ok(mut cb) = on_action.try_borrow_mut() {
                    (cb)(ActionRef {
                        action: action.to_string(),
                        payload: String::new(),
                    });
                }
            }
        }

        wire_avatar_src_observer(&root, on_action)?;

        Ok(())
    }

    /// `root` 配下の Avatar image 要素の `src` 属性差し替えを
    /// `MutationObserver` で検知し、`"reset"` を `on_action` へ dispatch する
    /// （イシュー #731、[`wire_avatar_events`] から呼ばれる）。
    ///
    /// `attributes: true` + `attributeFilter: ["src"]` + `subtree: true` で
    /// 登録する。`subtree: true` により `root` 配下にネストした `img`（再描画
    /// で入れ替わった新要素を含む）の `src` 変異も検知できる。
    /// `attributeFilter` で `src` 以外の属性変異（`apply_avatar_visibility`
    /// が書き込む `data-state`/`hidden` を含む）を構造的に除外することで、
    /// reset 後の DOM 反映が observer を再発火させる無限ループを起こさない。
    ///
    /// コールバック内では `MutationRecord::type_()`/`attribute_name()` を
    /// 防御的に再検証したうえで（`attributeFilter` はブラウザ実装への委任で
    /// あり、二重チェックにコストはほぼない）、`target()` が `root` の子孫
    /// （`root` 自身を含む）であることを [`Element::contains`] で確認する
    /// （`handle_image_event` の `contains` ガードと同じ意図。改ざんされた
    /// `data-*` を持つ無関係要素・`root` の外側の変異を dispatch へ流さない
    /// fail-closed 不変条件）。
    ///
    /// # Errors
    ///
    /// `MutationObserver::new`・`observe_with_options` の失敗を伝播する。
    fn wire_avatar_src_observer(
        root: &Element,
        on_action: std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) -> Result<(), JsValue> {
        let observed_root = root.clone();
        let callback = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::new(
            move |records: js_sys::Array, _observer: MutationObserver| {
                for record in records.iter() {
                    let Ok(record) = record.dyn_into::<MutationRecord>() else {
                        continue;
                    };
                    if record.type_() != "attributes" {
                        continue;
                    }
                    let attribute_name = record.attribute_name();
                    let Some(target) = record.target() else {
                        continue;
                    };
                    let Some(element) = target.dyn_ref::<Element>() else {
                        continue;
                    };
                    if !observed_root.contains(Some(element)) {
                        continue;
                    }
                    let scope = element.get_attribute("data-scope");
                    let part = element.get_attribute("data-part");
                    let Some(action_ref) = avatar_action_for_src_mutation(
                        attribute_name.as_deref(),
                        scope.as_deref(),
                        part.as_deref(),
                    ) else {
                        continue;
                    };
                    if let Ok(mut cb) = on_action.try_borrow_mut() {
                        (cb)(action_ref);
                    }
                }
            },
        );

        let observer = MutationObserver::new(callback.as_ref().unchecked_ref())?;
        let init = MutationObserverInit::new();
        init.set_attributes(true);
        init.set_subtree(true);
        init.set_attribute_filter(&js_sys::Array::of1(&JsValue::from_str("src")));
        observer.observe_with_options(root, &init)?;

        // `Closure::forget`: マウント時 1 回・定数個リーク契約（上記
        // `wire_avatar_events` doc 参照）。`observer` 自体は `observe()` 対象
        // ノードが登録済み observer として保持するため、`callback` を
        // forget した後も生存する。
        callback.forget();

        Ok(())
    }

    /// `load`/`error` イベント本体から `event.target()` の属性を読み、
    /// [`avatar_action_for_image_event`] の判定結果が `Some` の場合のみ
    /// `on_action` を呼ぶ。
    fn handle_image_event(
        root: &Element,
        event: &Event,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let Some(target) = event.target() else {
            return;
        };
        let Some(element) = target.dyn_ref::<Element>() else {
            return;
        };
        // capture フェーズは root 自身を含む祖先すべてを通過するため、
        // root より外側で発火した非バブリングイベントを誤って拾わないよう
        // 「target が root の子孫（root 自身を含む）であること」を確認する
        // （`events.rs::wire_events` の `contains` ガードと同じ意図）。
        if !root.contains(Some(element)) {
            return;
        }
        let scope = element.get_attribute("data-scope");
        let part = element.get_attribute("data-part");
        let Some(action_ref) =
            avatar_action_for_image_event(&event.type_(), scope.as_deref(), part.as_deref())
        else {
            return;
        };
        let Ok(mut cb) = on_action.try_borrow_mut() else {
            return;
        };
        (cb)(action_ref);
    }

    /// [`wire_avatar_events`] の便宜 API。`component`（Avatar 状態機械を含む
    /// 具象 `Component`）へ dispatch し、成功時のみ `on_update` を呼ぶ。
    ///
    /// `try_borrow_mut` 失敗（再入）は no-op とする（panic 回避、
    /// `crate::lib::Runtime::wire` と同じ方針）。DOM 更新は呼び出し側が
    /// `on_update` 内で（例えば [`apply_avatar_visibility`] を使って）行う
    /// 責務とし、本関数自体は `fandhe_frontend_interactive::dispatch` への橋渡し
    /// のみに専念する。
    ///
    /// # Errors
    ///
    /// [`wire_avatar_events`] のエラーをそのまま伝播する。
    pub fn wire_avatar_component<C>(
        root: Element,
        component: std::rc::Rc<std::cell::RefCell<C>>,
        mut on_update: impl FnMut(&C, &Element) + 'static,
    ) -> Result<(), JsValue>
    where
        C: Component + 'static,
    {
        let update_root = root.clone();
        wire_avatar_events(root, move |action_ref: ActionRef| {
            let Ok(mut state) = component.try_borrow_mut() else {
                return;
            };
            let dispatched = fandhe_frontend_interactive::dispatch(
                &mut *state,
                &action_ref.action,
                &action_ref.payload,
            );
            if !dispatched {
                return;
            }
            on_update(&state, &update_root);
        })
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{apply_avatar_visibility, wire_avatar_component, wire_avatar_events};

#[cfg(test)]
mod tests {
    use super::*;

    // --- avatar_action_for_image_event ---

    #[test]
    fn load_on_avatar_image_dispatches_loaded() {
        let action_ref =
            avatar_action_for_image_event("load", Some("avatar"), Some("image")).unwrap();
        assert_eq!(action_ref.action, "loaded");
        assert_eq!(action_ref.payload, "");
    }

    #[test]
    fn error_on_avatar_image_dispatches_error() {
        let action_ref =
            avatar_action_for_image_event("error", Some("avatar"), Some("image")).unwrap();
        assert_eq!(action_ref.action, "error");
        assert_eq!(action_ref.payload, "");
    }

    #[test]
    fn unknown_event_type_is_ignored() {
        assert_eq!(
            avatar_action_for_image_event("click", Some("avatar"), Some("image")),
            None
        );
    }

    #[test]
    fn mismatched_scope_is_ignored() {
        assert_eq!(
            avatar_action_for_image_event("load", Some("attacker"), Some("image")),
            None
        );
    }

    #[test]
    fn mismatched_part_is_ignored() {
        assert_eq!(
            avatar_action_for_image_event("load", Some("avatar"), Some("fallback")),
            None
        );
    }

    #[test]
    fn missing_attrs_is_ignored() {
        assert_eq!(avatar_action_for_image_event("load", None, None), None);
    }

    // --- avatar_action_for_settled_image ---

    #[test]
    fn settled_incomplete_image_yields_none() {
        assert_eq!(avatar_action_for_settled_image(false, 0), None);
        assert_eq!(avatar_action_for_settled_image(false, 100), None);
    }

    #[test]
    fn settled_complete_image_with_width_yields_loaded() {
        assert_eq!(avatar_action_for_settled_image(true, 100), Some("loaded"));
    }

    #[test]
    fn settled_complete_image_without_width_yields_error() {
        assert_eq!(avatar_action_for_settled_image(true, 0), Some("error"));
    }

    // --- avatar_action_for_src_mutation（イシュー #731） ---

    #[test]
    fn src_mutation_on_avatar_image_dispatches_reset() {
        let action_ref =
            avatar_action_for_src_mutation(Some("src"), Some("avatar"), Some("image")).unwrap();
        assert_eq!(action_ref.action, "reset");
        assert_eq!(action_ref.payload, "");
    }

    #[test]
    fn non_src_attribute_mutation_is_ignored() {
        assert_eq!(
            avatar_action_for_src_mutation(Some("data-state"), Some("avatar"), Some("image")),
            None
        );
        assert_eq!(
            avatar_action_for_src_mutation(Some("hidden"), Some("avatar"), Some("image")),
            None
        );
        assert_eq!(
            avatar_action_for_src_mutation(None, Some("avatar"), Some("image")),
            None
        );
    }

    #[test]
    fn src_mutation_with_mismatched_scope_or_part_is_ignored() {
        assert_eq!(
            avatar_action_for_src_mutation(Some("src"), Some("attacker"), Some("image")),
            None
        );
        assert_eq!(
            avatar_action_for_src_mutation(Some("src"), Some("avatar"), Some("fallback")),
            None
        );
        assert_eq!(
            avatar_action_for_src_mutation(Some("src"), None, None),
            None
        );
    }

    /// roundtrip: `avatar_action_for_src_mutation` → `dispatch` →
    /// `Avatar::status() == ImageStatus::Loading`（`Loaded` 起点から）。
    /// `wire_avatar_src_observer`（wasm32 配線層）が実際に呼ぶ経路を
    /// native 側で固定する（イシュー #731 受け入れ条件）。
    #[test]
    fn src_mutation_reset_action_roundtrip_returns_avatar_to_loading() {
        use fandhe_frontend_headless_ui::avatar::{Avatar, ImageStatus};

        let action_ref =
            avatar_action_for_src_mutation(Some("src"), Some("avatar"), Some("image")).unwrap();

        let mut avatar = Avatar::new(ImageStatus::Loaded);
        let dispatched = fandhe_frontend_interactive::dispatch(
            &mut avatar,
            &action_ref.action,
            &action_ref.payload,
        );
        assert!(dispatched);
        assert_eq!(avatar.status(), ImageStatus::Loading);
        assert_eq!(image_visible_after_action(&action_ref.action), Some(false));
    }

    // --- image_visible_after_action ---

    #[test]
    fn loaded_action_makes_image_visible() {
        assert_eq!(image_visible_after_action("loaded"), Some(true));
    }

    #[test]
    fn error_action_makes_image_hidden() {
        assert_eq!(image_visible_after_action("error"), Some(false));
    }

    #[test]
    fn reset_action_makes_image_hidden() {
        assert_eq!(image_visible_after_action("reset"), Some(false));
    }

    #[test]
    fn unknown_action_yields_none() {
        assert_eq!(image_visible_after_action("no_such_action"), None);
    }

    // --- XSS 回帰: 属性破りペイロードを持つ画像でも判定 → dispatch →
    // render の往復で生タグが出力されないこと（`events.rs` の roundtrip
    // テストと同型）。

    #[test]
    fn event_to_dispatch_to_render_roundtrip_escapes_attacker_alt() {
        use fandhe_frontend_headless_ui::avatar::{Avatar, ImageStatus};

        let action_ref =
            avatar_action_for_image_event("load", Some("avatar"), Some("image")).unwrap();

        let mut avatar = Avatar::new(ImageStatus::Loading);
        let dispatched = fandhe_frontend_interactive::dispatch(
            &mut avatar,
            &action_ref.action,
            &action_ref.payload,
        );
        assert!(dispatched);
        assert_eq!(avatar.status(), ImageStatus::Loaded);

        let alt = "\"><script>alert(1)</script>";
        let node = avatar.image("/a.png", alt, Vec::new());
        let html = fandhe_frontend_core::render(&node);
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;alert"));
    }

    // --- ドリフト検知: headless-ui の実出力（data-scope/data-part 値）が
    // 本モジュールのリテラルと一致すること。

    #[test]
    fn headless_ui_image_output_matches_module_literals() {
        use fandhe_frontend_headless_ui::avatar::{image, ImageStatus};

        let html = fandhe_frontend_core::render(&image(
            ImageStatus::Loading,
            "/a.png",
            "avatar",
            Vec::new(),
        ));
        assert!(html.contains(&format!(r#"data-scope="{AVATAR_SCOPE}""#)));
        assert!(html.contains(&format!(r#"data-part="{AVATAR_IMAGE_PART}""#)));
    }

    #[test]
    fn headless_ui_fallback_output_matches_module_literals() {
        use fandhe_frontend_headless_ui::avatar::{fallback, ImageStatus};

        let html =
            fandhe_frontend_core::render(&fallback(ImageStatus::Loaded, Vec::new(), Vec::new()));
        assert!(html.contains(&format!(r#"data-scope="{AVATAR_SCOPE}""#)));
        assert!(html.contains(&format!(r#"data-part="{AVATAR_FALLBACK_PART}""#)));
    }

    #[test]
    fn decode_action_accepts_loaded_error_reset_and_rejects_unknown() {
        use fandhe_frontend_headless_ui::avatar::Avatar;
        use fandhe_frontend_interactive::Component;

        assert!(<Avatar as Component>::decode_action("loaded", "").is_some());
        assert!(<Avatar as Component>::decode_action("error", "").is_some());
        assert!(<Avatar as Component>::decode_action("reset", "").is_some());
        assert!(<Avatar as Component>::decode_action("no_such_action", "").is_none());
    }

    #[test]
    fn image_visible_after_action_matches_image_status_is_image_visible() {
        use fandhe_frontend_headless_ui::avatar::ImageStatus;

        assert_eq!(
            image_visible_after_action("loaded"),
            Some(ImageStatus::Loaded.is_image_visible())
        );
        assert_eq!(
            image_visible_after_action("error"),
            Some(ImageStatus::Error.is_image_visible())
        );
        assert_eq!(
            image_visible_after_action("reset"),
            Some(ImageStatus::Loading.is_image_visible())
        );
    }
}

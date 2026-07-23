//! hidden-input パターンのフォーカスリング配線（イシュー #709、親 #520）。
//!
//! Switch（`crates/headless-ui/src/switch.rs`）・RadioGroup
//! （`crates/headless-ui/src/radio_group.rs`）・Checkbox
//! （`crates/headless-ui/src/checkbox.rs`）は実フォーカスを visually-hidden
//! なネイティブ `<input>`（`hidden-input`/`item-hidden-input`）に置く設計
//! であり、視覚上のパーツ（`control`/`item-control`）へフォーカスリングを
//! CSS だけで伝播できない。本モジュールはこの隙間を埋めるため、hidden-input
//! の focusin/focusout イベントに加え pointerdown/mousedown/click イベント
//! と `Element::matches(":focus-visible")` 判定に基づき、各 headless
//! モジュールが契約する `data-focus-visible`
//! 存在属性（`fandhe_frontend_headless_ui::data_attrs::data_focus_visible`）を
//! 境界パーツと同一 `data-scope` を共有する descendant パーツへ動的に
//! 付け外しする。フォーカスを保持したままポインター操作で
//! `:focus-visible` 判定が変化するケース（Tab キーでフォーカス後、同じ
//! コントロールをクリックする操作等）は focusin/focusout のいずれも発火
//! しないため、pointerdown/mousedown/click でも再評価する
//! （`wiring::wire_focus_visible` doc 参照、イシュー #709 PR #720 Cursor
//! Bugbot 指摘）。`fandhe-frontend-pre-styled-ui` はこの属性を CSS セレクタ
//! （例: `[data-scope="switch"][data-part="control"][data-focus-visible]`）
//! で参照しフォーカスリングを表現する（`crates/pre-styled-ui/src/switch.rs`/
//! `radio_group.rs` 参照）。
//!
//! [`keynav`](crate::keynav)・[`events`](crate::events) と同じ
//! 「純粋ロジック層（native `cargo test` 可）+
//! `#[cfg(target_arch = "wasm32")]` 配線層」の 2 層構成を踏襲する。
//!
//! # 設計: 状態機械へは一切波及しない、表示専用の属性付け替え
//!
//! `data-focus-visible` は SSR 静的表現のみを持つ transient 状態であり
//! （`data_focus_visible` の doc 参照）、本モジュールは DOM 属性の付け外し
//! のみを行う。`fandhe_frontend_interactive::dispatch` へは一切流さない
//! （`keynav` の「DOM 属性のみを読み書きする」性質と同型）。`hidden-input`
//! を改ざんされうる入力として扱い、[`boundary_part_for`] の静的マッピング
//! 表にない `(data-scope, data-part)` の組は no-op とする fail-closed 方針
//! （`headless.rs::action_for_part` と同じ設計）。

/// `(data-scope, data-part)` から、フォーカスリングを反映すべき境界パーツ
/// （`data-part` 名）への静的マッピング。
///
/// 境界パーツは各 headless モジュールのフォーカスリング契約 doc
/// （`crates/headless-ui/src/switch.rs` 等）で定義された祖先パーツであり、
/// 実 DOM 上は hidden-input の祖先（Switch/Checkbox: `root`）または
/// 直接の親（RadioGroup: `item`）にあたる。表にない組は `None`
/// （fail-closed、未知の hidden-input パターンへ誤って反応しない）。
#[must_use]
pub fn boundary_part_for(scope: &str, part: &str) -> Option<&'static str> {
    match (scope, part) {
        ("switch", "hidden-input") => Some("root"),
        ("radio-group", "item-hidden-input") => Some("item"),
        ("checkbox", "hidden-input") => Some("root"),
        _ => None,
    }
}

/// hidden-input 側 `data-part` を対象とする CSS セレクタ（focusin/focusout
/// のターゲット判定に使う。[`boundary_part_for`] のマッピング表と 1:1
/// 対応させる契約であり、表に組を追加する際は本セレクタにも追記する）。
pub const HIDDEN_INPUT_SELECTOR: &str = "[data-scope=\"switch\"][data-part=\"hidden-input\"], \
     [data-scope=\"radio-group\"][data-part=\"item-hidden-input\"], \
     [data-scope=\"checkbox\"][data-part=\"hidden-input\"]";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_hidden_input_parts_map_to_boundary() {
        assert_eq!(boundary_part_for("switch", "hidden-input"), Some("root"));
        assert_eq!(
            boundary_part_for("radio-group", "item-hidden-input"),
            Some("item")
        );
        assert_eq!(boundary_part_for("checkbox", "hidden-input"), Some("root"));
    }

    #[test]
    fn unknown_combinations_are_fail_closed_none() {
        // 未知の scope/part の組は反応しない（改ざんされた data-scope/data-part
        // への誤反応を避ける fail-closed 方針）。
        assert_eq!(boundary_part_for("switch", "control"), None);
        assert_eq!(boundary_part_for("select", "hidden-select"), None);
        assert_eq!(boundary_part_for("unknown", "hidden-input"), None);
        assert_eq!(boundary_part_for("switch", ""), None);
    }
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、native の
// `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （keynav.rs/events.rs/hydration.rs/dom.rs と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{boundary_part_for, HIDDEN_INPUT_SELECTOR};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event};

    /// `data-focus-visible` 属性名（[`fandhe_frontend_headless_ui::data_attrs::data_focus_visible`]
    /// が出力する属性と同一。本モジュールはこの `&'static str` リテラルのみを
    /// 属性名として使い、動的値が属性名スロットへ混入する経路を持たない）。
    const DATA_FOCUS_VISIBLE: &str = "data-focus-visible";

    /// `target` が [`HIDDEN_INPUT_SELECTOR`] に一致し `root` 配下にある場合、
    /// `(data-scope, 境界パーツを表す Element)` を返す。`data-scope` 欠落・
    /// 未知の組・`root` 外要素はいずれも `None`（fail-closed）。
    fn resolve_boundary(root: &Element, target: &Element) -> Option<(String, Element)> {
        if !target.matches(HIDDEN_INPUT_SELECTOR).unwrap_or(false) {
            return None;
        }
        if !root.contains(Some(target)) {
            return None;
        }
        let scope = target.get_attribute("data-scope")?;
        let part = target.get_attribute("data-part")?;
        let boundary_part = boundary_part_for(&scope, &part)?;
        let selector = format!("[data-scope=\"{scope}\"][data-part=\"{boundary_part}\"]");
        let boundary = target.closest(&selector).ok().flatten()?;
        if !root.contains(Some(&boundary)) {
            return None;
        }
        Some((scope, boundary))
    }

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （イシュー #401 の `fw gate` `url_validation_check` 契約に準拠、
    /// `.claude/rules/security.md`。`keynav.rs::wiring::set_dom_attribute` と
    /// 同じ方針）。本モジュールが書き込む属性（[`DATA_FOCUS_VISIBLE`]）は
    /// `&'static str` リテラルで固定された非 URL・非イベントハンドラ属性で
    /// あり値も常に空文字列だが、`fandhe_frontend_core::url` のガード関数群
    /// （`is_event_handler_attr`/`is_url_attr`/`is_safe_url`/
    /// `is_safe_srcset`）を経由することで、将来 `name`/`value` が動的な
    /// 入力から組み立てられるよう変更された場合の防御としても機能する。
    fn set_dom_attribute(element: &Element, name: &str, value: &str) {
        if fandhe_frontend_core::is_event_handler_attr(name) {
            return;
        }
        if fandhe_frontend_core::is_url_attr(name) && !fandhe_frontend_core::is_safe_url(value) {
            return;
        }
        if name.eq_ignore_ascii_case("srcset") && !fandhe_frontend_core::is_safe_srcset(value) {
            return;
        }
        let _ = element.set_attribute(name, value);
    }

    /// `boundary` 自身と、その配下で同一 `data-scope` を共有するパーツ
    /// （`control`/`item-control` 等、リングを見せたい視覚パーツを含む）へ
    /// `data-focus-visible` を付与する。単一要素にしか付与しないと
    /// `fandhe-frontend-pre-styled-ui` の recipe セレクタ（同一要素上の
    /// 属性有無で組み立てる、`crates/pre-styled-ui/src/recipe.rs` 参照）が
    /// 一致しないため、部分木全体へ伝播させる。
    fn set_focus_visible(scope: &str, boundary: &Element) {
        set_dom_attribute(boundary, DATA_FOCUS_VISIBLE, "");
        let selector = format!("[data-scope=\"{scope}\"]");
        if let Ok(node_list) = boundary.query_selector_all(&selector) {
            for i in 0..node_list.length() {
                if let Some(node) = node_list.get(i) {
                    if let Ok(element) = node.dyn_into::<Element>() {
                        set_dom_attribute(&element, DATA_FOCUS_VISIBLE, "");
                    }
                }
            }
        }
    }

    /// [`set_focus_visible`] と対の除去処理。
    fn remove_focus_visible(scope: &str, boundary: &Element) {
        let _ = boundary.remove_attribute(DATA_FOCUS_VISIBLE);
        let selector = format!("[data-scope=\"{scope}\"]");
        if let Ok(node_list) = boundary.query_selector_all(&selector) {
            for i in 0..node_list.length() {
                if let Some(node) = node_list.get(i) {
                    if let Ok(element) = node.dyn_into::<Element>() {
                        let _ = element.remove_attribute(DATA_FOCUS_VISIBLE);
                    }
                }
            }
        }
    }

    /// `event.target()` を [`HIDDEN_INPUT_SELECTOR`]/`root` 境界に解決し、
    /// 現在の `:focus-visible` 判定に応じて `data-focus-visible` を
    /// 付け外しする共通処理。
    ///
    /// `allow_removal` が偽の場合は「一致すれば付与する」片方向のみ
    /// （focusin の既存挙動: フォーカス直後は `:focus-visible` が真のときのみ
    /// 付与し、偽の場合は何もしない。まだフォーカスしていない要素へ
    /// 誤って除去処理が走らないための区別）。真の場合は「一致すれば付与・
    /// 不一致なら除去」の双方向評価（pointerdown 等、既にフォーカス保持中の
    /// 要素で `:focus-visible` 判定がその場で変化しうるイベント向け）。
    fn sync_focus_visible(root: &Element, target_element: &Element, allow_removal: bool) {
        let Some((scope, boundary)) = resolve_boundary(root, target_element) else {
            return;
        };
        if target_element.matches(":focus-visible").unwrap_or(false) {
            set_focus_visible(&scope, &boundary);
        } else if allow_removal {
            remove_focus_visible(&scope, &boundary);
        }
    }

    /// `root` 配下の hidden-input パターン（Switch/RadioGroup/Checkbox）へ
    /// focusin/focusout に加え pointerdown/mousedown/click の計 5 リスナーを
    /// 委譲登録する。
    ///
    /// - **focusin**: ターゲットが [`HIDDEN_INPUT_SELECTOR`] に一致し、かつ
    ///   `Element::matches(":focus-visible")`（キーボード操作等による
    ///   フォーカスをブラウザネイティブ実装へ判定委譲。独自のキーボード/
    ///   ポインタ判定は再実装しない、`.claude/rules/security.md` A04）が
    ///   真のときのみ [`set_focus_visible`] する（[`sync_focus_visible`]
    ///   `allow_removal = false`）。
    /// - **focusout**: `:focus-visible` 判定を行わず常に
    ///   [`remove_focus_visible`]（フォーカスが外れた時点でリングは不要な
    ///   ため判定不要、未付与でも `remove_attribute` は no-op）。
    /// - **pointerdown/mousedown/click**: hidden-input がフォーカスを
    ///   保持したままポインター操作を受けた場合（例: Tab キーでフォーカス
    ///   した後、同じコントロールをクリックする操作）、フォーカスイベントは
    ///   一切発火しないため focusin/focusout だけでは `:focus-visible` の
    ///   状態変化を検知できず `data-focus-visible` が blur まで残留する
    ///   （Cursor Bugbot 指摘、イシュー #709 PR #720）。この 3 イベントは
    ///   マウス/タッチ/ペン操作の開始から完了までを跨いで発火するため、
    ///   いずれかの時点でブラウザの `:focus-visible` 内部判定が更新され
    ///   次第 [`sync_focus_visible`]（`allow_removal = true`、双方向評価）
    ///   で追随できる。3 イベントとも同じ再評価を行うだけの冪等な処理
    ///   なので、重複発火しても副作用はない
    ///   （`set_attribute`/`remove_attribute` の重ね書きは無害）。
    ///
    /// 状態機械（`fandhe_frontend_interactive::dispatch`）へは一切流さない
    /// 純粋な表示属性の付け替えであり、[`keynav::wire_keynav`](crate::keynav::wire_keynav)
    /// とは独立した経路のため、失敗しても他の配線の成立を妨げない
    /// （`lib.rs` のマウントパス参照）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` が失敗した場合に `Err` を返す。
    pub fn wire_focus_visible(root: Element) -> Result<(), JsValue> {
        let focusin_root = root.clone();
        let focusin_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            let Some(target_element) = target.dyn_ref::<Element>().cloned() else {
                return;
            };
            sync_focus_visible(&focusin_root, &target_element, false);
        });
        root.add_event_listener_with_callback("focusin", focusin_closure.as_ref().unchecked_ref())?;
        focusin_closure.forget();

        let focusout_root = root.clone();
        let focusout_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            let Some(target_element) = target.dyn_ref::<Element>().cloned() else {
                return;
            };
            let Some((scope, boundary)) = resolve_boundary(&focusout_root, &target_element) else {
                return;
            };
            remove_focus_visible(&scope, &boundary);
        });
        root.add_event_listener_with_callback(
            "focusout",
            focusout_closure.as_ref().unchecked_ref(),
        )?;
        focusout_closure.forget();

        // pointerdown/mousedown/click: フォーカス保持中のポインター操作による
        // `:focus-visible` 状態変化を拾うための追加リスナー（doc 参照）。
        // 3 イベントとも同一ハンドラ相当のロジックを個別クロージャで登録する
        // （`Closure::<dyn FnMut(Event)>` は 1 イベント種別につき 1 個の
        // `add_event_listener_with_callback` 呼び出しを要するため）。
        for event_name in ["pointerdown", "mousedown", "click"] {
            let sync_root = root.clone();
            let sync_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
                let Some(target) = event.target() else {
                    return;
                };
                let Some(target_element) = target.dyn_ref::<Element>().cloned() else {
                    return;
                };
                sync_focus_visible(&sync_root, &target_element, true);
            });
            root.add_event_listener_with_callback(
                event_name,
                sync_closure.as_ref().unchecked_ref(),
            )?;
            sync_closure.forget();
        }

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_focus_visible;

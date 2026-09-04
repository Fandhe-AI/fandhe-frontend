//! AngleSlider（`fandhe-frontend-headless-ui` `angle_slider` モジュール）の
//! ポインタ座標 → 角度変換・DOM 配線（イシュー #842、非採用の再導入、
//! 親トラッキング #520）。
//!
//! `crates/headless-ui/src/angle_slider.rs` は Root/Label/Control/Thumb/
//! MarkerGroup/Marker/ValueText/HiddenInput の 8 anatomy パーツと整数角度
//! 状態機械（`"set"`/`"increment"`/`"decrement"`/`"home"`/`"end"` dispatch。
//! Home/End はイシュー #1601 で追加）を提供する一方、実際にポインタ座標を
//! 角度へ変換する処理・DOM イベント配線は同モジュール冒頭 rustdoc
//! 「スコープ外」節が明記するとおり本クレート（wasm 層）の後続責務と
//! されていた。本モジュールがその変換・配線を実装する（ただし Home/End
//! の keydown 配線は REQ-11 予算逼迫のため未対応、下記「キーボード操作」
//! 節参照）。
//!
//! # 「非採用の再導入」の中核: 座標 → 角度変換の隔離
//!
//! `docs/policy/intentional-non-adoption.md` §3.22 が AngleSlider を非採用と
//! した理由は「ポインタ座標 → 角度変換の暗黙性・非決定性・機械検証困難」
//! である。本モジュールは [`angle_from_offset`] という**単一の純粋関数**
//! （`web-sys` 非依存、`atan2` の使用はこの関数内のみ）へ変換ロジックを
//! 完全に閉じ込め、native `cargo test` の網羅表（8 方位・丸め境界・中心点・
//! 非有限入力）で決定性を固定する。DOM 配線層（`#[cfg(target_arch =
//! "wasm32")]`）はこの純粋関数を呼ぶだけで、独自の座標計算ロジックを
//! 一切持たない。
//!
//! # 座標系規約（ark-ui 互換）
//!
//! [`angle_from_offset`] は「中心からのオフセット座標 `(dx, dy)`」を入力に
//! 取る（画面座標系: `dy` は下方向が正）。`0` 度を真上、時計回りに増加する
//! 角度を返す（ark-ui AngleSlider と同じ規約）。「最後に観測した座標
//! 1 点から角度を再計算する」設計であり、ポインタイベントのストリーム
//! 頻度・座標精度差・履歴・速度には一切依存しない（決定性、モジュール冒頭
//! 「非採用の再導入」節参照）。
//!
//! # `events.rs`/`headless_clipboard.rs` との責務分離
//!
//! [`crate::events`] のクリック/入力委譲と同じ 2 層構成（DOM 非依存の純粋
//! ロジック層 + `#[cfg(target_arch = "wasm32")]` 配線層）を踏襲するが、
//! pointerdown/pointermove/pointerup という click/input 以外のイベント種別
//! を扱うため `crate::headless::MAPPING_TABLE`（同期的な (scope, part) →
//! action の静的マッピング）には**乗せない**（[`crate::headless_clipboard`]
//! と同型の独立配線モジュールとして切り出す）。
//!
//! # Runtime への統合
//!
//! [`wire_angle_slider_events`] は `crate::lib::Runtime::mount`/
//! `Runtime::hydrate` の双方から `Self::wire_timer` の直後に組み込まれる
//! （`crate::lib::Runtime::wire_angle_slider` 参照）。`events`/`keynav`/
//! `headless_clipboard` と同じ「マウント時 1 回」契約を維持する。
//!
//! # キーボード操作（イシュー #1601 で参照突合。Home/End は headless 側の
//! dispatch API までで DOM keydown 配線は未対応）
//!
//! Thumb（`role="slider"`、`tabindex` でフォーカス可能）上の keydown を
//! 独立に配線する。ArrowUp/ArrowRight は `"increment"`、ArrowDown/
//! ArrowLeft は `"decrement"` を dispatch する（矢印キーの意味論は
//! `crate::keynav` の他コンポーネントと同型だが、AngleSlider は単一値の
//! 増減のみでインデックスベースの項目ナビゲーションを持たないため
//! `crate::keynav` の `MAPPING_TABLE`/next-index 系関数へは統合せず、本
//! モジュール内で完結させる）。
//!
//! [`fandhe_frontend_headless_ui::angle_slider::AngleSlider`] は
//! イシュー #1601 で `"home"`/`"end"` dispatch（`AngleSliderAction::SetToMin`/
//! `SetToMax`）を状態機械レベルで受理するようになったが、本モジュールの
//! `action_for_key`/keydown 配線は **意図的に** Home/End キーを追加しない
//! （zag.js 相当の挙動だが本イシューでは見送り）。理由は REQ-11
//! （WASM バンドルサイズ 200KB gzip 上限）の予算逼迫: main 時点で既に
//! 199,962/200,000 B（余裕 38 B）しかなく、`action_for_key` へ Home/End
//! の 2 分岐を追加するだけで実測 +139 B（`docs/spec/04-requirements.md`
//! REQ-11）を要し、単独では収まらない。ヘッドレス層の dispatch 契約は
//! アプリ側が独自にキーボード配線する場合や、将来 REQ-11 予算の見直し後に
//! 本モジュールへ追加する場合の受け皿として維持する。Shift+Arrow の ×10
//! step も同様に本イシューでは未実装（いずれも PR 本文でスコープ外 Issue
//! 化を提案）。
//!
//! # セキュリティ不変条件
//!
//! - dispatch payload（`"set"` の角度整数文字列）は
//!   [`fandhe_frontend_headless_ui::angle_slider::AngleSlider::decode_action`]
//!   が改めて `u16`・`0..=360` 範囲で厳密検証する（本モジュールはあくまで
//!   payload 文字列を組み立てるのみで、検証は headless 層の既存契約に
//!   委ねる、多層防御）。`"home"`/`"end"` は payload を持たない。
//! - `data-disabled` **または** `data-readonly` を持つ Control/Thumb（祖先
//!   方向を含む）上の pointerdown/keydown は no-op
//!   （[`has_noninteractive_ancestor`]、`crate::headless.rs` の fail-closed
//!   契約と同型。イシュー #1601 で `data-readonly`
//!   〔[`fandhe_frontend_headless_ui::angle_slider::AngleSliderProps::readonly`]〕
//!   出力を追加したのに合わせ、readonly でも操作を抑止しないと
//!   「読み取り専用の見た目なのに編集できる」矛盾になるため拡張した。zag の
//!   `interactive = !(disabled || readOnly)` 判定と同型）。
//! - DOM 反映は `set_attribute`/`get_attribute`/`get_bounding_client_rect`
//!   のみで行い、HTML 文字列を一切組み立てない（REQ-1）。属性名はすべて
//!   `&'static str` リテラル。
//! - ポインタ座標・計算済み角度値はいずれも `console`・例外メッセージへ
//!   出力しない（機微情報の露出防止という程の秘匿性はないが、
//!   `.claude/rules/security.md` A09 の一般方針として他 headless_* モジュール
//!   と同じログ非出力方針を踏襲する）。
//! - 新規 `unsafe` コードは追加しない（`web-sys`/`js-sys` の safe API のみ
//!   使用）。

/// AngleSlider の `data-scope` 属性値（`fandhe_frontend_headless_ui::angle_slider`
/// の `ANATOMY` と一致、`crates/headless-ui/src/angle_slider.rs` 参照）。
const ANGLE_SLIDER_SCOPE: &str = "angle-slider";
/// AngleSlider Control パーツの `data-part` 属性値。
const CONTROL_PART: &str = "control";
/// AngleSlider Thumb パーツの `data-part` 属性値。
const THUMB_PART: &str = "thumb";

/// dispatch アクション名 `"set"`（`AngleSliderAction::Set`/
/// `AngleSlider::decode_action` の対応する分岐と一致）。
pub const ACTION_SET: &str = "set";
/// dispatch アクション名 `"increment"`。
pub const ACTION_INCREMENT: &str = "increment";
/// dispatch アクション名 `"decrement"`。
pub const ACTION_DECREMENT: &str = "decrement";
// `"home"`/`"end"`（`AngleSliderAction::SetToMin`/`SetToMax`）はヘッドレス層の
// dispatch 契約としては存在するが、本モジュールの `action_for_key`/keydown
// 配線からは意図的に呼ばれない（モジュール冒頭 doc「キーボード操作」節、
// REQ-11 予算逼迫の理由を参照）。専用の定数は持たない。

/// 「中心からのオフセット座標」`(dx, dy)` を `0..=359` の整数角度（度）へ
/// 変換する純粋関数（`web-sys` 非依存、native `cargo test` で決定的に
/// 検証可能。`atan2` の使用は本関数内のみ、モジュール冒頭 doc「座標 → 角度
/// 変換の隔離」節参照）。
///
/// 画面座標系（`dy` は下方向が正）を前提に、`0` 度を真上、時計回りに
/// 増加する角度を返す（ark-ui AngleSlider 互換、モジュール冒頭 doc
/// 「座標系規約」節参照）。
///
/// - 中心点そのもの（`dx == 0.0 && dy == 0.0`）は角度が数学的に未定義の
///   ため `None`（fail-closed）。
/// - `dx`/`dy` のいずれかが非有限（`NaN`/無限大）の場合も `None`
///   （fail-closed）。
/// - 四捨五入した結果が `360.0` に到達する場合（例: `359.6` 度）は `0` 度へ
///   ラップする。
#[must_use]
pub fn angle_from_offset(dx: f64, dy: f64) -> Option<u16> {
    if !dx.is_finite() || !dy.is_finite() {
        return None;
    }
    if dx == 0.0 && dy == 0.0 {
        return None;
    }
    let degrees = dx.atan2(-dy).to_degrees();
    // `rem_euclid` は負の角度も含めて常に `[0.0, 360.0)` へ正規化する
    // （符号付き剰余 `%` と異なり負値を返さない）。
    let normalized = degrees.rem_euclid(360.0);
    let rounded = normalized.round();
    let wrapped = if rounded >= 360.0 { 0.0 } else { rounded };
    Some(wrapped as u16)
}

/// クリックターゲットが AngleSlider の Control/Thumb 要素かどうかを判定
/// する純粋関数（DOM 非依存、native `cargo test` で検証可能）。
#[must_use]
pub fn is_angle_slider_control_or_thumb(scope: Option<&str>, part: Option<&str>) -> bool {
    scope == Some(ANGLE_SLIDER_SCOPE) && matches!(part, Some(CONTROL_PART) | Some(THUMB_PART))
}

/// キー名から dispatch すべきアクション名を判定する純粋関数（DOM 非依存）。
///
/// ArrowUp/ArrowRight は時計回り増加（[`ACTION_INCREMENT`]）、ArrowDown/
/// ArrowLeft は反時計回り減少（[`ACTION_DECREMENT`]）。Home/End は
/// ヘッドレス層の dispatch 契約（`"home"`/`"end"`）としては存在するが、
/// 本関数からは意図的に対応しない（モジュール冒頭 doc「キーボード操作」
/// 節、REQ-11 予算逼迫の理由を参照）。それ以外のキーは `None`
/// （no-op、fail-closed）。
#[must_use]
pub fn action_for_key(key: &str) -> Option<&'static str> {
    match key {
        "ArrowUp" | "ArrowRight" => Some(ACTION_INCREMENT),
        "ArrowDown" | "ArrowLeft" => Some(ACTION_DECREMENT),
        _ => None,
    }
}

// ---------------------------------------------------------------------
// 配線層: web-sys/js-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`events.rs`/`headless_clipboard.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        action_for_key, angle_from_offset, is_angle_slider_control_or_thumb, ACTION_SET,
        ANGLE_SLIDER_SCOPE, CONTROL_PART, THUMB_PART,
    };
    use crate::events::ActionRef;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, KeyboardEvent, PointerEvent};

    /// `target` から `root`（含む）まで祖先方向へ辿り、`data-scope`/
    /// `data-part` が指定値と一致する最初の要素を返す
    /// （`crate::headless_clipboard::wiring::closest_matching` と同型）。
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

    /// `start` から `root` まで祖先方向を辿り、`data-disabled` **または**
    /// `data-readonly` を持つ要素が 1 つでもあれば `true` を返す（disabled/
    /// readonly な祖先を含めて no-op とする fail-closed 判定。
    /// `crate::headless.rs` の祖先 disabled 対策、および zag の
    /// `interactive = !(disabled || readOnly)` 判定と同型。イシュー #1601
    /// で `data-readonly` も見るよう拡張した旧 `has_disabled_ancestor`）。
    fn has_noninteractive_ancestor(root: &Element, start: &Element) -> bool {
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if element.has_attribute("data-disabled") || element.has_attribute("data-readonly") {
                return true;
            }
            if !root.contains(Some(&element)) || element == *root {
                break;
            }
            current = element.parent_element();
        }
        false
    }

    /// AngleSlider の Control 要素を、クリック/ポインタ操作対象の要素から
    /// 祖先方向に探す（`start` 自身が Control/Thumb の場合はそこから、
    /// そうでなければ最も近い Control 祖先を返す）。
    fn resolve_control(root: &Element, start: &Element) -> Option<Element> {
        let scope = start.get_attribute("data-scope");
        let part = start.get_attribute("data-part");
        if is_angle_slider_control_or_thumb(scope.as_deref(), part.as_deref()) {
            if part.as_deref() == Some(CONTROL_PART) {
                return Some(start.clone());
            }
            return closest_matching(root, start, ANGLE_SLIDER_SCOPE, CONTROL_PART);
        }
        closest_matching(root, start, ANGLE_SLIDER_SCOPE, CONTROL_PART)
    }

    /// `control` の `getBoundingClientRect` 中心座標からのオフセットで
    /// クライアント座標 `(client_x, client_y)` の角度を求める。
    fn angle_at_client_point(control: &Element, client_x: f64, client_y: f64) -> Option<u16> {
        let rect = control.get_bounding_client_rect();
        let center_x = rect.left() + rect.width() / 2.0;
        let center_y = rect.top() + rect.height() / 2.0;
        angle_from_offset(client_x - center_x, client_y - center_y)
    }

    /// `root` 配下の AngleSlider Control/Thumb へ pointerdown/pointermove/
    /// pointerup と Thumb keydown の配線を 1 回だけ登録する（マウント時
    /// 1 回契約）。
    ///
    /// pointer 系は `setPointerCapture` により、pointerdown が発生した
    /// Control 要素へ以後の pointermove/pointerup を固定する（`root` 全体
    /// への delegate ではなく、pointer 個別のキャプチャで完結させる。
    /// 複数 AngleSlider が同一ページに存在しても互いに干渉しない）。
    ///
    /// `on_action` は `"set"`/`"increment"`/`"decrement"` の dispatch 依頼を
    /// 呼び出し側（`crate::lib::Runtime::wire_angle_slider`）へ渡すのみで、
    /// 状態更新・DOM 反映は行わない（`headless_clipboard::wire_clipboard_events`
    /// と同じ責務分離）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` の失敗を伝播する。
    pub fn wire_angle_slider_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let on_action = std::rc::Rc::new(std::cell::RefCell::new(on_action));

        let pointerdown_root = root.clone();
        let pointerdown_on_action = on_action.clone();
        let pointerdown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointerdown(&pointerdown_root, &event, &pointerdown_on_action);
        });
        root.add_event_listener_with_callback(
            "pointerdown",
            pointerdown_closure.as_ref().unchecked_ref(),
        )?;
        pointerdown_closure.forget();

        let pointermove_root = root.clone();
        let pointermove_on_action = on_action.clone();
        let pointermove_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointermove(&pointermove_root, &event, &pointermove_on_action);
        });
        root.add_event_listener_with_callback(
            "pointermove",
            pointermove_closure.as_ref().unchecked_ref(),
        )?;
        pointermove_closure.forget();

        let keydown_root = root.clone();
        let keydown_on_action = on_action.clone();
        let keydown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_keydown(&keydown_root, &event, &keydown_on_action);
        });
        root.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())?;
        keydown_closure.forget();

        Ok(())
    }

    /// pointerdown: Control/Thumb 上でのみ反応し、以後の pointermove を同一
    /// pointer に固定する（`setPointerCapture`）うえで最初の座標を反映する。
    fn handle_pointerdown(
        root: &Element,
        event: &Event,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        let Some(target) = event.target() else {
            return;
        };
        let Some(target_element) = target.dyn_ref::<Element>() else {
            return;
        };

        let Some(control) = resolve_control(root, target_element) else {
            return;
        };
        if has_noninteractive_ancestor(root, &control) {
            return;
        }

        let _ = control.set_pointer_capture(pointer_event.pointer_id());
        dispatch_angle_at_point(
            &control,
            pointer_event.client_x() as f64,
            pointer_event.client_y() as f64,
            on_action,
        );
    }

    /// pointermove: `setPointerCapture` 済みの pointer からのみ、Control 中心
    /// からの角度を再計算して `"set"` を dispatch する。
    fn handle_pointermove(
        root: &Element,
        event: &Event,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        let Some(target) = event.target() else {
            return;
        };
        let Some(target_element) = target.dyn_ref::<Element>() else {
            return;
        };

        let Some(control) = resolve_control(root, target_element) else {
            return;
        };
        if !control.has_pointer_capture(pointer_event.pointer_id()) {
            // このイベントは本 Control が capture 中の pointer に由来しない
            // （キャプチャ前の hover 移動等）。誤反応を避け no-op とする。
            return;
        }
        if has_noninteractive_ancestor(root, &control) {
            return;
        }

        dispatch_angle_at_point(
            &control,
            pointer_event.client_x() as f64,
            pointer_event.client_y() as f64,
            on_action,
        );
    }

    /// keydown: Thumb 上の ArrowUp/ArrowRight/ArrowDown/ArrowLeft のみ反応
    /// する（[`action_for_key`]、モジュール冒頭 doc「キーボード操作」節
    /// 参照）。Home/End は REQ-11 の gzip 予算逼迫により意図的に未配線
    /// （`home_and_end_keys_are_intentionally_not_wired_pending_req11_budget`
    /// テスト参照）。
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

        let scope = target_element.get_attribute("data-scope");
        let part = target_element.get_attribute("data-part");
        if scope.as_deref() != Some(ANGLE_SLIDER_SCOPE) || part.as_deref() != Some(THUMB_PART) {
            return;
        }
        if has_noninteractive_ancestor(root, target_element) {
            return;
        }

        let Some(action) = action_for_key(&keyboard_event.key()) else {
            return;
        };

        keyboard_event.prevent_default();
        if let Ok(mut cb) = on_action.try_borrow_mut() {
            (cb)(ActionRef {
                action: action.to_string(),
                payload: String::new(),
            });
        }
    }

    /// `control` の中心とクライアント座標から角度を計算し、有効な場合のみ
    /// `"set"` を dispatch する（[`angle_from_offset`] が `None` を返す
    /// 場合は no-op、fail-closed）。
    fn dispatch_angle_at_point(
        control: &Element,
        client_x: f64,
        client_y: f64,
        on_action: &std::rc::Rc<std::cell::RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        let Some(angle) = angle_at_client_point(control, client_x, client_y) else {
            return;
        };
        if let Ok(mut cb) = on_action.try_borrow_mut() {
            (cb)(ActionRef {
                action: ACTION_SET.to_string(),
                payload: angle.to_string(),
            });
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_angle_slider_events;

#[cfg(test)]
mod tests {
    use super::*;

    // --- angle_from_offset: 8 方位の網羅表 ---

    #[test]
    fn eight_compass_points_map_to_expected_degrees() {
        let cases: &[((f64, f64), u16)] = &[
            ((0.0, -1.0), 0),
            ((1.0, -1.0), 45),
            ((1.0, 0.0), 90),
            ((1.0, 1.0), 135),
            ((0.0, 1.0), 180),
            ((-1.0, 1.0), 225),
            ((-1.0, 0.0), 270),
            ((-1.0, -1.0), 315),
        ];
        for &((dx, dy), expected) in cases {
            assert_eq!(
                angle_from_offset(dx, dy),
                Some(expected),
                "dx={dx}, dy={dy}"
            );
        }
    }

    // --- 丸め境界 ---

    #[test]
    fn near_360_degrees_rounds_and_wraps_to_zero() {
        // 359.7 度付近（真上からわずかに反時計回り）を単位円上の座標で構成する。
        let theta = 359.7_f64.to_radians();
        let dx = theta.sin();
        let dy = -theta.cos();
        assert_eq!(angle_from_offset(dx, dy), Some(0));
    }

    #[test]
    fn near_zero_degrees_rounds_down_to_zero() {
        let theta = 0.3_f64.to_radians();
        let dx = theta.sin();
        let dy = -theta.cos();
        assert_eq!(angle_from_offset(dx, dy), Some(0));
    }

    #[test]
    fn exact_half_degree_boundary_rounds_up() {
        // 44.5 度は四捨五入で 45 度側へ丸まる。
        let theta = 44.5_f64.to_radians();
        let dx = theta.sin();
        let dy = -theta.cos();
        assert_eq!(angle_from_offset(dx, dy), Some(45));
    }

    // --- fail-closed: 中心点・非有限入力 ---

    #[test]
    fn center_point_is_none() {
        assert_eq!(angle_from_offset(0.0, 0.0), None);
    }

    #[test]
    fn non_finite_inputs_are_none() {
        assert_eq!(angle_from_offset(f64::NAN, 1.0), None);
        assert_eq!(angle_from_offset(1.0, f64::NAN), None);
        assert_eq!(angle_from_offset(f64::INFINITY, 1.0), None);
        assert_eq!(angle_from_offset(1.0, f64::NEG_INFINITY), None);
    }

    // --- スケール不変性: 半径が異なっても同じ方向なら同じ角度 ---

    #[test]
    fn scaling_the_offset_does_not_change_the_angle() {
        assert_eq!(angle_from_offset(10.0, -10.0), angle_from_offset(1.0, -1.0));
        assert_eq!(
            angle_from_offset(0.001, -0.001),
            angle_from_offset(1.0, -1.0)
        );
    }

    // --- is_angle_slider_control_or_thumb ---

    #[test]
    fn control_and_thumb_match_scope_and_part() {
        assert!(is_angle_slider_control_or_thumb(
            Some("angle-slider"),
            Some("control")
        ));
        assert!(is_angle_slider_control_or_thumb(
            Some("angle-slider"),
            Some("thumb")
        ));
        assert!(!is_angle_slider_control_or_thumb(
            Some("angle-slider"),
            Some("root")
        ));
        assert!(!is_angle_slider_control_or_thumb(
            Some("attacker"),
            Some("control")
        ));
        assert!(!is_angle_slider_control_or_thumb(None, None));
    }

    // --- action_for_key ---

    #[test]
    fn arrow_up_and_right_increment() {
        assert_eq!(action_for_key("ArrowUp"), Some(ACTION_INCREMENT));
        assert_eq!(action_for_key("ArrowRight"), Some(ACTION_INCREMENT));
    }

    #[test]
    fn arrow_down_and_left_decrement() {
        assert_eq!(action_for_key("ArrowDown"), Some(ACTION_DECREMENT));
        assert_eq!(action_for_key("ArrowLeft"), Some(ACTION_DECREMENT));
    }

    #[test]
    fn home_and_end_keys_are_intentionally_not_wired_pending_req11_budget() {
        // モジュール冒頭 doc「キーボード操作」節参照: ヘッドレス層は
        // "home"/"end" dispatch を受理するが、本関数（DOM keydown 配線の
        // 判定）は REQ-11（WASM バンドルサイズ）予算逼迫のため意図的に
        // 対応しない。
        assert_eq!(action_for_key("Home"), None);
        assert_eq!(action_for_key("End"), None);
    }

    #[test]
    fn unknown_key_is_none() {
        assert_eq!(action_for_key("Enter"), None);
        assert_eq!(action_for_key("a"), None);
        assert_eq!(action_for_key(""), None);
    }

    // --- ドリフト検知: headless-ui の実出力（data-scope/data-part 値）が
    // 本モジュールのリテラルと一致すること。---

    #[test]
    fn headless_ui_control_output_matches_module_literals() {
        use fandhe_frontend_headless_ui::angle_slider::{control, AngleSliderProps};

        let html = fandhe_frontend_core::render(&control(
            &AngleSliderProps::default(),
            Vec::new(),
            Vec::new(),
        ));
        assert!(html.contains(&format!(r#"data-scope="{ANGLE_SLIDER_SCOPE}""#)));
        assert!(html.contains(&format!(r#"data-part="{CONTROL_PART}""#)));
    }

    #[test]
    fn headless_ui_thumb_output_matches_module_literals() {
        use fandhe_frontend_headless_ui::angle_slider::{thumb, AngleSliderProps};

        let html = fandhe_frontend_core::render(&thumb(
            "0",
            "0deg",
            &AngleSliderProps::default(),
            Vec::new(),
            Vec::new(),
        ));
        assert!(html.contains(&format!(r#"data-scope="{ANGLE_SLIDER_SCOPE}""#)));
        assert!(html.contains(&format!(r#"data-part="{THUMB_PART}""#)));
    }

    #[test]
    fn decode_action_accepts_set_increment_decrement_and_rejects_unknown() {
        use fandhe_frontend_headless_ui::angle_slider::AngleSlider;
        use fandhe_frontend_interactive::Component;

        assert!(<AngleSlider as Component>::decode_action(ACTION_SET, "90").is_some());
        assert!(<AngleSlider as Component>::decode_action(ACTION_INCREMENT, "").is_some());
        assert!(<AngleSlider as Component>::decode_action(ACTION_DECREMENT, "").is_some());
        assert!(<AngleSlider as Component>::decode_action("no_such_action", "").is_none());
    }

    #[test]
    fn decode_action_accepts_home_end_at_headless_layer_though_unwired_here() {
        // ヘッドレス層の dispatch 契約自体は "home"/"end" を受理する
        // （本モジュールの keydown 配線が対応しないのとは独立、モジュール
        // 冒頭 doc「キーボード操作」節参照）。
        use fandhe_frontend_headless_ui::angle_slider::AngleSlider;
        use fandhe_frontend_interactive::Component;

        assert!(<AngleSlider as Component>::decode_action("home", "").is_some());
        assert!(<AngleSlider as Component>::decode_action("end", "").is_some());
    }

    // --- roundtrip: action 名 → dispatch → AngleSlider::angle_deg ---

    #[test]
    fn set_action_roundtrip_via_dispatch() {
        use fandhe_frontend_headless_ui::angle_slider::AngleSlider;

        let mut a = AngleSlider::default();
        assert_eq!(a.angle_deg(), 0);

        let angle = angle_from_offset(1.0, 0.0).unwrap();
        let dispatched =
            fandhe_frontend_interactive::dispatch(&mut a, ACTION_SET, &angle.to_string());
        assert!(dispatched);
        assert_eq!(a.angle_deg(), 90);
    }
}

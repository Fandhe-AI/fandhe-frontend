//! SignaturePad（`fandhe-frontend-headless-ui` `signature_pad` モジュール）の
//! ポインタ座標収集グルー（イシュー #843、親 #735/#520）。
//!
//! `crates/headless-ui/src/signature_pad.rs` は「ストローク（座標列）の列 →
//! SVG path 文字列」の決定的な純粋関数のみを持ち、ポインタイベントから座標を
//! 収集する処理は同モジュール冒頭 rustdoc が明記するとおり本クレート
//! （wasm 層）の責務としている。本モジュールがその収集処理を実装する。
//!
//! # 2 層構成（`headless_avatar.rs`/`headless_timer.rs` と同型）
//!
//! - 純粋ロジック層（[`StrokeCollector`]）は web-sys に依存せず、native の
//!   `cargo test` で検証できる。ポインタ座標（`x`/`y`）を明示的な引数として
//!   受け取るのみで、実際の `PointerEvent`/DOM からは独立している
//!   （計画書「座標アサーション（合成座標列 + 合成 PointerEvent）による
//!   決定的検証」に対応する設計）。
//! - 配線層（[`wiring::wire_signature_pad_component`]）のみ
//!   `#[cfg(target_arch = "wasm32")]` でゲートし、native ビルドへ web-sys
//!   依存を混入させない。
//!
//! # `fandhe_frontend_headless_ui::signature_pad` を直接利用する（文字列複製
//! しない）理由
//!
//! `crates/wasm-full/Cargo.toml` は `fandhe-frontend-headless-ui` を通常の
//! `[dependencies]`（製品依存）として持つ（イシュー #590 以降）。そのため
//! [`StrokeCollector::finish`] は `fandhe_frontend_headless_ui::signature_pad::Stroke`/
//! `stroke_to_payload` を直接呼び、座標列の直列化フォーマット（丸め規則を
//! 含む）を一切複製しない（ドリフトの心配が構造的に生じない設計）。
//!
//! # 単一ストローク・単一ポインタの追跡モデル
//!
//! [`StrokeCollector`] は同時に 1 本のストロークのみを追跡する
//! （`active_pointer_id` が `Some` の間は他の `pointer_id` からの
//! `pointerdown` を無視する）。ark-ui SignaturePad も同様にマルチタッチでの
//! 同時複数ストローク描画をサポートしないため、この単純化は仕様追随である。
//!
//! # clear-trigger のクリックは既存の汎用配線を再利用する
//!
//! ClearTrigger のクリックは `data-scope`/`data-part` から文字列アクションへの
//! 静的マッピング（[`crate::headless::MAPPING_TABLE`] へ
//! `("signature-pad", "clear-trigger") -> "clear"` 行を追加）と
//! [`crate::headless::wire_headless_component`]（イシュー #580）をそのまま
//! 再利用し、本モジュールは重複するクリック判定コードを持たない。
//! [`wiring::wire_signature_pad_component`] はこの汎用クリック配線と
//! ポインタ座標収集配線の両方を 1 回のマウントで組み込む。
//!
//! # セキュリティ不変条件
//!
//! - `unsafe` は新規追加しない（`web-sys`/`js-sys` の safe API のみ使用）。
//! - ポインタ座標は `PointerEvent::client_x`/`client_y`（`f64`）から素直に
//!   計算するのみで、任意の文字列・DOM 属性値として解釈しない
//!   （HTML/属性注入経路にならない）。
//! - [`StrokeCollector`] は 1 ストロークあたりの点数を
//!   [`fandhe_frontend_headless_ui::signature_pad::MAX_POINTS_PER_STROKE`] で
//!   打ち切る（改ざんされた大量の合成イベントによる無制限メモリ確保 DoS を
//!   防止、A04）。
//! - dispatch payload の生成（[`StrokeCollector::finish`]）は
//!   `fandhe_frontend_headless_ui::signature_pad::Stroke::new`/
//!   `stroke_to_payload` を経由するため、非有限値・空ストロークは
//!   headless 層の既存 fail-closed 検証がそのまま適用される。

use fandhe_frontend_headless_ui::signature_pad::{stroke_to_payload, Point, Stroke};

/// SignaturePad の `data-scope` 属性値（`fandhe_frontend_headless_ui::signature_pad`
/// の `ANATOMY` と一致）。
pub const SIGNATURE_PAD_SCOPE: &str = "signature-pad";

/// dispatch アクション名 "add-stroke"（
/// `fandhe_frontend_headless_ui::signature_pad::SignaturePad::decode_action`
/// の対応する分岐と一致する契約）。
pub const ACTION_ADD_STROKE: &str = "add-stroke";

/// ポインタ座標からストロークを収集する純粋な状態機械（DOM 非依存、native
/// `cargo test` で検証可能）。
///
/// `pointerdown` で追跡を開始し、以後同じ `pointer_id` の `pointermove` で
/// 座標点を追加する。`pointerup`/`pointercancel` で追跡を終了する
/// （`pointercancel` は座標列を破棄し dispatch しない。ブラウザがジェス
/// チャー競合等でポインタ操作を取り消した場合の標準的な扱い）。
#[derive(Debug, Default)]
pub struct StrokeCollector {
    /// 現在追跡中のポインタ ID（`None` = 追跡していない）。
    active_pointer_id: Option<i32>,
    /// 追跡中の座標点列。
    points: Vec<Point>,
}

impl StrokeCollector {
    /// 新しい空の収集器を作る。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 現在追跡中かどうか。
    #[must_use]
    pub fn is_tracking(&self) -> bool {
        self.active_pointer_id.is_some()
    }

    /// `pointerdown`: 既に他のポインタを追跡中でなければ、`pointer_id` の
    /// 追跡を開始する（モジュール doc「単一ストローク・単一ポインタの
    /// 追跡モデル」参照）。既に追跡中の場合は無視する（fail-closed、
    /// 同時複数ストローク描画をサポートしない設計）。
    pub fn on_pointer_down(&mut self, pointer_id: i32, x: f64, y: f64) {
        if self.active_pointer_id.is_some() {
            return;
        }
        self.active_pointer_id = Some(pointer_id);
        self.points = vec![Point::new(x, y)];
    }

    /// `pointermove`: `pointer_id` が現在追跡中のものと一致する場合のみ
    /// 座標点を追加する。[`fandhe_frontend_headless_ui::signature_pad::MAX_POINTS_PER_STROKE`]
    /// に達した以降の点は黙って無視する（追跡自体は継続する。改ざんされた
    /// 大量の合成イベントによる無制限メモリ確保 DoS の防止、A04）。
    pub fn on_pointer_move(&mut self, pointer_id: i32, x: f64, y: f64) {
        if self.active_pointer_id != Some(pointer_id) {
            return;
        }
        if self.points.len() >= fandhe_frontend_headless_ui::signature_pad::MAX_POINTS_PER_STROKE {
            return;
        }
        self.points.push(Point::new(x, y));
    }

    /// `pointerup`: `pointer_id` が現在追跡中のものと一致する場合に追跡を
    /// 終了し、確定したストロークの dispatch payload
    /// （[`fandhe_frontend_headless_ui::signature_pad::stroke_to_payload`]
    /// 形式）を返す。1 点も収集していない場合・`pointer_id` が不一致の場合は
    /// `None`（fail-closed、payload を生成しない）。
    #[must_use]
    pub fn on_pointer_up(&mut self, pointer_id: i32) -> Option<String> {
        if self.active_pointer_id != Some(pointer_id) {
            return None;
        }
        let points = std::mem::take(&mut self.points);
        self.active_pointer_id = None;
        let stroke = Stroke::new(points).ok()?;
        Some(stroke_to_payload(&stroke))
    }

    /// `pointercancel`: `pointer_id` が現在追跡中のものと一致する場合、
    /// 座標列を破棄して追跡を終了する（dispatch しない）。
    pub fn on_pointer_cancel(&mut self, pointer_id: i32) {
        if self.active_pointer_id == Some(pointer_id) {
            self.active_pointer_id = None;
            self.points.clear();
        }
    }
}

/// クリックされた要素が SignaturePad の描画領域（Control/Segment/
/// SegmentPath）かどうかを判定する純粋関数（DOM 非依存、native
/// `cargo test` で検証可能）。ClearTrigger 等の無関係パーツ上での
/// `pointerdown` を描画開始として誤認しないための fail-closed ガード。
#[must_use]
pub fn is_drawable_part(scope: Option<&str>, part: Option<&str>) -> bool {
    scope == Some(SIGNATURE_PAD_SCOPE)
        && matches!(
            part,
            Some("control") | Some("segment") | Some("segment-path")
        )
}

/// `segment` 要素の `viewBox="0 0 {width} {height}"` 属性値から
/// `(width, height)`（SVG ユーザー単位）を取り出す純粋関数（DOM 非依存、
/// native `cargo test` で検証可能）。
///
/// `fandhe_frontend_headless_ui::signature_pad::segment` が生成する形式
/// （`min-x`/`min-y` は常に `0`、4 トークン空白区切り）のみを想定する。
/// 想定外の形式（トークン数不一致・非数値・非有限値・0 以下）は `None`
/// （fail-closed。呼び出し側はスケーリングを諦め等倍にフォールバックする）。
// 呼び出し元は wasm32 の `wiring` モジュール（[`wiring::segment_rect_transform`]）
// と native テストのみのため、native の非テストビルドでは未使用と誤検出される
// （`headless_avatar.rs::AVATAR_FALLBACK_PART` と同じ理由の dead_code 抑制）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[must_use]
fn parse_view_box_dimensions(view_box: &str) -> Option<(f64, f64)> {
    let tokens: Vec<&str> = view_box.split_whitespace().collect();
    let [_, _, width, height] = tokens[..] else {
        return None;
    };
    let width: f64 = width.parse().ok()?;
    let height: f64 = height.parse().ok()?;
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return None;
    }
    Some((width, height))
}

/// 表示上の bounding box（CSS ピクセル、`getBoundingClientRect()` 由来）から
/// SVG `viewBox` のユーザー単位への軸ごとの倍率を計算する純粋関数（DOM 非依存、
/// native `cargo test` で検証可能）。
///
/// pre-styled 側で `width: 100%` 等により表示サイズが `viewBox` の
/// `width`/`height` と異なる場合、CSS ピクセルのポインタ座標をそのまま
/// ユーザー単位として扱うとストロークが実際のカーソル位置からずれる
/// （Bugbot 指摘、イシュー #843 PR #872）。`rect_dim` の要素が非有限・0 以下
/// （レイアウト未確定・display:none 等）の場合は等倍（`1.0`）にフォール
/// バックする（fail-closed。座標を欠落させるより歪みなく描画を継続する
/// ほうを優先する設計）。
///
/// # 前提（`preserveAspectRatio` 非指定）
///
/// `fandhe_frontend_headless_ui::signature_pad::segment` は `svg` 要素に
/// `preserveAspectRatio` を指定しない（SVG 既定 `xMidYMid meet`）。かつ
/// `pre-styled-ui::signature_pad::stylesheet` が付与する CSS は `width: 100%`
/// のみで `height` を独立指定しない（イシュー #843）ため、実際の運用では
/// 表示上の縦横比が常に `viewBox` の縦横比と一致する（ブラウザの置換要素の
/// 既定挙動により、明示 `height` 未指定時は `viewBox` の縦横比を保って高さが
/// 決まる）。本関数は軸ごとに独立した倍率を返すため、呼び出し元が
/// `viewBox` と異なる縦横比で明示的に `width`/`height` を独立指定する構成
/// （本関数の前提が崩れる構成）を許す場合、letterbox/pillarbox オフセット
/// を考慮しない歪みが生じ得る（現状の pre-styled-ui スタイルシートは
/// この構成を作らないため、Bugbot 指摘のシナリオでは影響しない）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[must_use]
fn scale_factors(view_box_dim: (f64, f64), rect_dim: (f64, f64)) -> (f64, f64) {
    let scale = |view_box: f64, rect: f64| -> f64 {
        if rect.is_finite() && rect > 0.0 {
            view_box / rect
        } else {
            1.0
        }
    };
    (
        scale(view_box_dim.0, rect_dim.0),
        scale(view_box_dim.1, rect_dim.1),
    )
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`headless_avatar.rs`/`headless_timer.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        is_drawable_part, parse_view_box_dimensions, scale_factors, StrokeCollector,
        ACTION_ADD_STROKE,
    };
    use crate::events::ActionRef;
    use fandhe_frontend_interactive::Component;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, PointerEvent};

    /// `event.target()` から祖先方向（自身を含む）へ
    /// `[data-scope="signature-pad"][data-part="segment"]` 要素を探し、
    /// 見つかった場合はその `getBoundingClientRect()` 原点（`viewBox` の
    /// `(0, 0)` に対応する画面座標）と、CSS ピクセルから `viewBox` ユーザー
    /// 単位への軸ごとの倍率（`(origin_x, origin_y, scale_x, scale_y)`）を
    /// 返す。`segment-path`（ストローク自体）上のイベントも `closest` が
    /// 祖先の `segment` を辿って解決する。見つからない場合（描画領域外、
    /// 例: ClearTrigger 上）は `None`（fail-closed）。
    ///
    /// pre-styled 側で `width: 100%` 等により表示上の bounding box が
    /// `viewBox="0 0 {width} {height}"` の寸法と異なる場合、CSS ピクセルの
    /// ポインタ座標をそのままユーザー単位として扱うとストロークが実際の
    /// カーソル位置からずれる（Bugbot 指摘、イシュー #843 PR #872）。
    /// `viewBox` 属性が想定外の形式・bounding box が非有限/0 以下の場合は
    /// [`parse_view_box_dimensions`]/[`scale_factors`] がそれぞれ `None`/
    /// 等倍にフォールバックする（fail-closed）。
    ///
    /// 呼び出し側（[`is_drawable_part`] を経由しない `pointermove`/
    /// `pointerup` を含む）は、追跡開始済みかどうかを [`StrokeCollector`]
    /// 自身の `pointer_id` 一致判定に委ねるため、本関数は「描画領域内か」
    /// の判定そのものは行わず、原点・倍率の解決失敗のみを扱う。
    fn segment_rect_transform(target: &Element) -> Option<(f64, f64, f64, f64)> {
        let segment = target
            .closest(r#"[data-scope="signature-pad"][data-part="segment"]"#)
            .ok()
            .flatten()?;
        let view_box = segment.get_attribute("viewBox")?;
        let view_box_dim = parse_view_box_dimensions(&view_box)?;
        let rect = segment.get_bounding_client_rect();
        let (scale_x, scale_y) = scale_factors(view_box_dim, (rect.width(), rect.height()));
        Some((rect.left(), rect.top(), scale_x, scale_y))
    }

    /// `PointerEvent` からクライアント座標を取り出す（内部ヘルパ）。
    fn client_xy(event: &PointerEvent) -> (f64, f64) {
        (event.client_x() as f64, event.client_y() as f64)
    }

    /// `root` 配下の SignaturePad 描画領域へ `pointerdown`/`pointermove`/
    /// `pointerup`/`pointercancel` を配線し、確定したストロークを
    /// `on_action`（`ActionRef { action: "add-stroke", payload }`）へ
    /// 橋渡しする。
    ///
    /// ポインタ座標は [`is_drawable_part`] が描画領域と判定した要素上の
    /// イベントのみを扱い、[`segment_rect_transform`] が返す `segment`
    /// 要素の左上を原点とし、CSS ピクセルから `viewBox` ユーザー単位へ
    /// スケーリングしたローカル座標へ変換する
    /// （`fandhe_frontend_headless_ui::signature_pad::segment` の `viewBox`
    /// 原点 `(0, 0)` と一致させ、表示上の bounding box が `viewBox` の
    /// 寸法と異なる場合でもカーソル位置とストロークの追随を一致させる。
    /// Bugbot 指摘、イシュー #843 PR #872）。
    ///
    /// `Closure::forget` は `pointerdown`/`pointermove`/`pointerup`/
    /// `pointercancel` の **4 回のみ**に限定する（`events.rs`/
    /// `headless_avatar.rs` と同じ「マウント時 1 回・定数個リーク」契約、
    /// A04 対策）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` の失敗を伝播する。
    pub fn wire_stroke_collector(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let collector = std::rc::Rc::new(std::cell::RefCell::new(StrokeCollector::new()));
        // `on_action` を配線ごとに共有できるよう Rc<RefCell<>> でラップする
        // （`headless_avatar.rs::wire_avatar_events` と同じ方針）。
        let on_action = std::rc::Rc::new(std::cell::RefCell::new(on_action));

        // pointerdown: 描画領域（[`is_drawable_part`]）内のみ追跡開始する
        // （ClearTrigger 等の無関係パーツ上での押下を描画開始と誤認しない
        // fail-closed ガード）。
        {
            let down_root = root.clone();
            let down_collector = collector.clone();
            let closure = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
                let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                    return;
                };
                if !down_root.contains(Some(&target)) {
                    return;
                }
                let scope = target.get_attribute("data-scope");
                let part = target.get_attribute("data-part");
                if !is_drawable_part(scope.as_deref(), part.as_deref()) {
                    return;
                }
                let Some((origin_x, origin_y, scale_x, scale_y)) = segment_rect_transform(&target)
                else {
                    return;
                };
                let (client_x, client_y) = client_xy(&event);
                if let Ok(mut c) = down_collector.try_borrow_mut() {
                    c.on_pointer_down(
                        event.pointer_id(),
                        (client_x - origin_x) * scale_x,
                        (client_y - origin_y) * scale_y,
                    );
                }
                // ポインタキャプチャを取得し、以後 pointermove/pointerup の
                // `event.target()` が描画領域外へドラッグしても本要素に
                // 固定されるようにする（キャプチャなしでは画面境界を跨いだ
                // 際に `segment_rect_transform` が解決できず、ストロークが
                // 途切れて見える不具合を防ぐ）。失敗しても致命的ではないため
                // 結果は無視する（`Result` を握りつぶす、panic 回避）。
                let _ = target.set_pointer_capture(event.pointer_id());
            });
            root.add_event_listener_with_callback("pointerdown", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let move_root = root.clone();
            let move_collector = collector.clone();
            let closure = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
                let Ok(mut c) = move_collector.try_borrow_mut() else {
                    return;
                };
                if !c.is_tracking() {
                    return;
                }
                let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                    return;
                };
                if !move_root.contains(Some(&target)) {
                    return;
                }
                let Some((origin_x, origin_y, scale_x, scale_y)) = segment_rect_transform(&target)
                else {
                    return;
                };
                let (client_x, client_y) = client_xy(&event);
                c.on_pointer_move(
                    event.pointer_id(),
                    (client_x - origin_x) * scale_x,
                    (client_y - origin_y) * scale_y,
                );
            });
            root.add_event_listener_with_callback("pointermove", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let up_collector = collector.clone();
            let up_on_action = on_action.clone();
            let closure = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
                let payload = {
                    let Ok(mut c) = up_collector.try_borrow_mut() else {
                        return;
                    };
                    c.on_pointer_up(event.pointer_id())
                };
                if let Some(payload) = payload {
                    if let Ok(mut cb) = up_on_action.try_borrow_mut() {
                        (cb)(ActionRef {
                            action: ACTION_ADD_STROKE.to_string(),
                            payload,
                        });
                    }
                }
            });
            root.add_event_listener_with_callback("pointerup", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let cancel_collector = collector.clone();
            let closure = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
                if let Ok(mut c) = cancel_collector.try_borrow_mut() {
                    c.on_pointer_cancel(event.pointer_id());
                }
            });
            root.add_event_listener_with_callback(
                "pointercancel",
                closure.as_ref().unchecked_ref(),
            )?;
            closure.forget();
        }

        Ok(())
    }

    /// [`wire_stroke_collector`]（描画のポインタ座標収集）と
    /// [`crate::headless::wire_headless_component`]（ClearTrigger クリック、
    /// モジュール doc「clear-trigger のクリックは既存の汎用配線を再利用
    /// する」参照）の両方を 1 回のマウントで組み込む便宜 API。
    ///
    /// 両配線とも成功時のみ `on_update` を呼ぶ
    /// （[`crate::lib::Runtime::wire`] と同じ「配線は状態更新・再描画に
    /// 結合しない」方針）。
    ///
    /// # Errors
    ///
    /// [`wire_stroke_collector`]・[`crate::headless::wire_headless_component`]
    /// のいずれかの失敗をそのまま伝播する。
    pub fn wire_signature_pad_component<C>(
        root: Element,
        component: std::rc::Rc<std::cell::RefCell<C>>,
        on_update: impl FnMut(&C, &Element) + 'static,
    ) -> Result<(), JsValue>
    where
        C: Component + 'static,
    {
        let on_update = std::rc::Rc::new(std::cell::RefCell::new(on_update));

        // クリック（ClearTrigger）: 既存の汎用マッピング表配線を再利用する。
        {
            let click_component = component.clone();
            let click_on_update = on_update.clone();
            let click_root = root.clone();
            crate::headless::wire_headless_component(
                click_root,
                click_component,
                move |state, r| {
                    if let Ok(mut cb) = click_on_update.try_borrow_mut() {
                        (cb)(state, r);
                    }
                },
            )?;
        }

        // ポインタ座標収集（描画）: dispatch 成功時のみ on_update を呼ぶ。
        {
            let stroke_component = component;
            let stroke_root = root.clone();
            let stroke_on_update = on_update;
            wire_stroke_collector(root, move |action_ref: ActionRef| {
                let Ok(mut state) = stroke_component.try_borrow_mut() else {
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
                if let Ok(mut cb) = stroke_on_update.try_borrow_mut() {
                    (cb)(&state, &stroke_root);
                }
            })?;
        }

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{wire_signature_pad_component, wire_stroke_collector};

#[cfg(test)]
mod tests {
    use super::*;

    // --- StrokeCollector ---

    #[test]
    fn pointer_down_then_move_then_up_produces_payload() {
        let mut collector = StrokeCollector::new();
        collector.on_pointer_down(1, 0.0, 0.0);
        collector.on_pointer_move(1, 10.0, 10.0);
        let payload = collector.on_pointer_up(1).unwrap();
        assert_eq!(payload, "0.00,0.00 10.00,10.00");
        assert!(!collector.is_tracking());
    }

    #[test]
    fn pointer_up_with_only_down_produces_single_point_payload() {
        let mut collector = StrokeCollector::new();
        collector.on_pointer_down(1, 5.0, 5.0);
        let payload = collector.on_pointer_up(1).unwrap();
        assert_eq!(payload, "5.00,5.00");
    }

    #[test]
    fn mismatched_pointer_id_move_is_ignored() {
        let mut collector = StrokeCollector::new();
        collector.on_pointer_down(1, 0.0, 0.0);
        collector.on_pointer_move(2, 99.0, 99.0);
        let payload = collector.on_pointer_up(1).unwrap();
        assert_eq!(payload, "0.00,0.00");
    }

    #[test]
    fn second_pointer_down_while_tracking_is_ignored() {
        let mut collector = StrokeCollector::new();
        collector.on_pointer_down(1, 0.0, 0.0);
        collector.on_pointer_down(2, 50.0, 50.0);
        // 2 番目の pointerdown は無視されるため、追跡中の pointer は 1 のまま。
        collector.on_pointer_move(2, 99.0, 99.0);
        let payload = collector.on_pointer_up(1).unwrap();
        assert_eq!(payload, "0.00,0.00");
    }

    #[test]
    fn pointer_up_with_mismatched_id_returns_none_and_keeps_tracking() {
        let mut collector = StrokeCollector::new();
        collector.on_pointer_down(1, 0.0, 0.0);
        assert_eq!(collector.on_pointer_up(2), None);
        assert!(collector.is_tracking());
    }

    #[test]
    fn pointer_up_without_down_returns_none() {
        let mut collector = StrokeCollector::new();
        assert_eq!(collector.on_pointer_up(1), None);
    }

    #[test]
    fn pointer_cancel_discards_stroke() {
        let mut collector = StrokeCollector::new();
        collector.on_pointer_down(1, 0.0, 0.0);
        collector.on_pointer_move(1, 10.0, 10.0);
        collector.on_pointer_cancel(1);
        assert!(!collector.is_tracking());
        // cancel 後の pointerup は無関係な ID として無視される。
        assert_eq!(collector.on_pointer_up(1), None);
    }

    #[test]
    fn pointer_cancel_with_mismatched_id_is_ignored() {
        let mut collector = StrokeCollector::new();
        collector.on_pointer_down(1, 0.0, 0.0);
        collector.on_pointer_cancel(2);
        assert!(collector.is_tracking());
    }

    #[test]
    fn move_beyond_max_points_is_capped() {
        use fandhe_frontend_headless_ui::signature_pad::MAX_POINTS_PER_STROKE;
        let mut collector = StrokeCollector::new();
        collector.on_pointer_down(1, 0.0, 0.0);
        for i in 0..MAX_POINTS_PER_STROKE + 100 {
            collector.on_pointer_move(1, i as f64, i as f64);
        }
        let payload = collector.on_pointer_up(1).unwrap();
        assert_eq!(payload.split(' ').count(), MAX_POINTS_PER_STROKE);
    }

    // --- is_drawable_part ---

    #[test]
    fn drawable_part_accepts_control_segment_and_segment_path() {
        assert!(is_drawable_part(Some("signature-pad"), Some("control")));
        assert!(is_drawable_part(Some("signature-pad"), Some("segment")));
        assert!(is_drawable_part(
            Some("signature-pad"),
            Some("segment-path")
        ));
    }

    #[test]
    fn drawable_part_rejects_clear_trigger_and_mismatched_scope() {
        assert!(!is_drawable_part(
            Some("signature-pad"),
            Some("clear-trigger")
        ));
        assert!(!is_drawable_part(Some("attacker"), Some("control")));
        assert!(!is_drawable_part(None, None));
    }

    // --- parse_view_box_dimensions / scale_factors（Bugbot 指摘、イシュー
    // #843 PR #872: viewBox スケーリング未考慮によるポインタ座標ずれ） ---

    #[test]
    fn parse_view_box_dimensions_reads_width_and_height() {
        assert_eq!(
            parse_view_box_dimensions("0 0 300 150"),
            Some((300.0, 150.0))
        );
    }

    #[test]
    fn parse_view_box_dimensions_rejects_malformed_input() {
        assert_eq!(parse_view_box_dimensions(""), None);
        assert_eq!(parse_view_box_dimensions("0 0 300"), None);
        assert_eq!(parse_view_box_dimensions("0 0 300 abc"), None);
        assert_eq!(parse_view_box_dimensions("0 0 0 150"), None);
        assert_eq!(parse_view_box_dimensions("0 0 -300 150"), None);
        assert_eq!(parse_view_box_dimensions("0 0 NaN 150"), None);
    }

    #[test]
    fn scale_factors_computes_ratio_when_displayed_size_differs_from_view_box() {
        // pre-styled で `width: 100%` により表示が 600x300（viewBox の 2 倍）
        // に拡大された場合、CSS ピクセル座標は 0.5 倍してユーザー単位へ
        // 変換する必要がある（Bugbot 指摘のシナリオそのもの）。
        let (scale_x, scale_y) = scale_factors((300.0, 150.0), (600.0, 300.0));
        assert!((scale_x - 0.5).abs() < f64::EPSILON);
        assert!((scale_y - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn scale_factors_uses_independent_per_axis_ratio_for_mismatched_aspect_ratio() {
        // `scale_factors` は軸ごとに独立した比率を返す仕様であることを固定
        // する回帰テスト（`scale_factors` の rustdoc「前提」節参照）。この
        // 独立比率は表示上の縦横比が `viewBox` の縦横比と一致する構成
        // （pre-styled-ui の現行 CSS はこの構成のみを作る）でのみ letterbox/
        // pillarbox オフセットなしに正しく動作する。縦横比が一致しない構成
        // （本テストの 300x150 → 600x600 等）を許す呼び出し元を新設する場合は
        // 本関数の前提が崩れるため、その時点で uniform-scale + centering
        // offset への切り替えを検討する必要がある。
        let (scale_x, scale_y) = scale_factors((300.0, 150.0), (600.0, 600.0));
        assert!((scale_x - 0.5).abs() < f64::EPSILON);
        assert!((scale_y - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn scale_factors_is_identity_when_sizes_match() {
        let (scale_x, scale_y) = scale_factors((300.0, 150.0), (300.0, 150.0));
        assert_eq!((scale_x, scale_y), (1.0, 1.0));
    }

    #[test]
    fn scale_factors_falls_back_to_identity_for_non_positive_rect() {
        // レイアウト未確定・display:none 等で rect が 0/非有限になっても
        // 座標を破棄せず等倍にフォールバックする（fail-closed）。
        assert_eq!(scale_factors((300.0, 150.0), (0.0, 0.0)), (1.0, 1.0));
        assert_eq!(
            scale_factors((300.0, 150.0), (f64::NAN, f64::NAN)),
            (1.0, 1.0)
        );
    }

    // --- dispatch roundtrip（ドリフト検知） ---

    #[test]
    fn add_stroke_payload_roundtrips_through_signature_pad_dispatch() {
        use fandhe_frontend_headless_ui::signature_pad::SignaturePad;

        let mut collector = StrokeCollector::new();
        collector.on_pointer_down(1, 1.0, 2.0);
        collector.on_pointer_move(1, 3.0, 4.0);
        let payload = collector.on_pointer_up(1).unwrap();

        let mut pad = SignaturePad::default();
        let dispatched =
            fandhe_frontend_interactive::dispatch(&mut pad, ACTION_ADD_STROKE, &payload);
        assert!(dispatched);
        assert_eq!(pad.strokes().len(), 1);
    }
}

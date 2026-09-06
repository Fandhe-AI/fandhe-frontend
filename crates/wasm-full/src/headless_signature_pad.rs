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
//!
//! # pointer capture の再付与（イシュー #1993、親 #1991）
//!
//! `wiring::wire_stroke_collector` は `pointerdown` 時に描画要素
//! （`event.target()`）へ `set_pointer_capture` を掛けるが、signature-pad は
//! `pointermove` では dispatch しない（`pointerup` でのみ `add-stroke` を
//! dispatch する）ため、ストローク中の再描画は本モジュール自身の配線では
//! なく外部要因（同じ `root` 配下の別配線の `on_update`・`Runtime::rerender`
//! の明示呼び出し等）で起きる。[`crate::lib::Runtime::rerender_subtree`]
//! による構造フォールバックは `root` 配下の全子ノードを作り直すため、
//! capture を持っていた要素が detach され、ブラウザ側の capture は暗黙に
//! 失われる。capture 喪失後は `pointermove` の `event.target()` が新しい
//! DOM の要素（非描画パーツを含む）になり、`segment_rect_transform` が
//! 解決できず座標が黙って落ちる、あるいはポインタが `root` 外へ出た場合
//! `pointermove`/`pointerup` 自体が届かなくなる。
//!
//! [`StrokeCollector`]（純粋層）は `pointer_id`・座標列を閉包内（DOM 外）に
//! 保持するため再描画をまたいでも失われない。失われるのは「どの DOM 要素へ
//! capture を掛け直すか」の再解決手段のみであり、`wiring` は追跡開始時に
//! 採取した SignaturePad Root（`[data-part="root"]`）の `id` を安定識別子
//! （`angle_slider.rs` の `PartKey::RootId` 分岐と同型のパターン。ただし
//! 共通化はせず本モジュール内に閉じた private 実装とする）として保持し、
//! 以後の `pointermove` で `event.target()` が capture を保持していない
//! （＝喪失した）と判定した場合にこの Root `id` から Control 要素を再解決
//! して `set_pointer_capture` を掛け直す。
//!
//! **Root `id` が無い構成では再解決を開始しない**（fail-closed。位置ベースの
//! 識別（文書順の添字・要素数）は `angle_slider.rs::PartKey` doc と同じ理由
//! で使わない）。`fandhe_frontend_headless_ui::signature_pad::SignaturePad::view`
//! が生成する正準ビューは Root に `id` を付与しないため、アプリが
//! `root(..)` の `attrs` で明示的に `id` を与えた構成でのみ本対策が有効に
//! なる。また、ポインタが `root` 外にある間は capture が無い以上イベント
//! 自体が `root` へ届かないため、この窓（再描画直後・かつポインタが
//! `root` 外へ出た場合）は設計上回復不能である。以後 `root` 内へポインタが
//! 戻った時点の `pointermove` から再解決が効く。

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
///
/// # pointer capture 喪失時の自己回復（イシュー #1992、親 #1991）
///
/// `wiring::wire_stroke_collector` は `pointerdown` 時に描画要素へ
/// `set_pointer_capture` を掛けるが、構造フォールバックによる要素差し替え等
/// で capture が暗黙に失われることがある。capture 喪失中に `root` 外で
/// ポインタボタンが離されると `pointerup`/`pointercancel` を取り逃し、
/// `active_pointer_id` が `Some` のまま stale 化する。`on_pointer_down` は
/// 追跡中なら新規 `pointerdown` を無視する設計のため、放置すると以後
/// すべての新規ストローク開始が恒久的にブロックされてしまう。
///
/// [`StrokeCollector::release_if_stale`] は `angle_slider.rs` の
/// `DragState`（`handle_pointermove` の「stale な追跡の自己解除」節）と
/// 同型のパターンを適用し、pointermove 経路で毎回 `buttons` を確認する
/// ことでこの恒久ブロックを次の pointermove 1 件で自己修復する
/// （fail-closed。座標列は破棄し dispatch しない）。
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

    /// 現在追跡中の `pointer_id`（追跡していなければ `None`）。
    ///
    /// `wiring::wire_stroke_collector` の pointermove 配線が capture 再付与
    /// （イシュー #1993、struct doc「pointer capture の再付与」節参照）の
    /// 対象を限定するために使う: `set_pointer_capture` は「このイベントの
    /// pointer が追跡中の pointer と一致する場合」にのみ掛けなければならず、
    /// [`Self::on_pointer_move`] が既に行っている `pointer_id` 不一致判定を
    /// capture 再付与の判定側でも独立に確認する必要があるため公開する。
    #[must_use]
    pub fn active_pointer_id(&self) -> Option<i32> {
        self.active_pointer_id
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

    /// pointer capture 喪失時の自己回復ガード（struct doc「pointer capture
    /// 喪失時の自己回復」節参照、イシュー #1992）。
    ///
    /// 追跡中の `pointer_id` と一致し（`active_pointer_id ==
    /// Some(pointer_id)`）、かつ `buttons == 0`（どのボタンも押されていない・
    /// ペン/指が接触していない）の場合のみ、座標列を破棄して追跡を終了する
    /// （[`Self::on_pointer_cancel`] と同じ「破棄して静かに終了する」契約。
    /// dispatch しない）。それ以外（非追跡中、`pointer_id` が追跡中のものと
    /// 不一致、またはいずれかのボタンが押されたまま追跡継続中）は何もせず
    /// `false` を返す。
    ///
    /// `pointer_id` の一致確認が必須な理由（PR #1999 レビュー指摘）:
    /// 単一ポインタ追跡モデル（struct doc「単一ストローク・単一ポインタの
    /// 追跡モデル」節参照）では、指/ペンで描画中に別デバイス（マウス等）が
    /// `root` 上で hover しても `buttons == 0` の pointermove イベントが
    /// 発火し得る。`pointer_id` を確認せず `buttons` のみで stale 判定する
    /// と、この無関係な別ポインタの hover により描画中の正当なストローク
    /// まで誤って破棄してしまう（[`Self::on_pointer_move`] が本来担う
    /// `pointer_id` 不一致イベントの無視より本ガードが先に実行されるため、
    /// その保護を迂回してしまう）。
    ///
    /// 呼び出し元は `wiring::wire_stroke_collector` の pointermove
    /// クロージャで、`PointerEvent::pointer_id()`/`PointerEvent::buttons()`
    /// を毎回渡すことで capture 喪失中に `root` 外で取り逃した
    /// `pointerup`/`pointercancel` を次の pointermove 1 件で自己修復する
    /// （`document` への追加リスナーは張らない fail-closed 設計。
    /// `angle_slider.rs::handle_pointermove` と同型のパターン。同関数も
    /// `pointer_id` 一致確認後に `buttons` を判定する）。
    ///
    /// 戻り値 `true` は stale な追跡を実際に解除したことを示す。呼び出し側
    /// はこの場合、当該 pointermove イベント自体の座標を追加せず早期
    /// リターンする（解除直後の座標は信頼できないため）。
    pub fn release_if_stale(&mut self, pointer_id: i32, buttons: u16) -> bool {
        if self.active_pointer_id != Some(pointer_id) || buttons != 0 {
            return false;
        }
        self.active_pointer_id = None;
        self.points.clear();
        true
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
        ACTION_ADD_STROKE, SIGNATURE_PAD_SCOPE,
    };
    use crate::events::ActionRef;
    use fandhe_frontend_interactive::Component;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, PointerEvent};

    /// `event.target()` から `[data-scope="signature-pad"][data-part="segment"]`
    /// 要素を探し、見つかった場合はその `getBoundingClientRect()` 原点
    /// （`viewBox` の `(0, 0)` に対応する画面座標）と、CSS ピクセルから
    /// `viewBox` ユーザー単位への軸ごとの倍率（`(origin_x, origin_y,
    /// scale_x, scale_y)`）を返す。見つからない場合（描画領域外、例:
    /// ClearTrigger 上）は `None`（fail-closed）。
    ///
    /// 探索は 2 段階（`closest` → `query_selector`）で行う。`segment`/
    /// `segment-path`（ストローク自体）上のイベントは `closest` が祖先の
    /// `segment` を辿って解決するが、`control`（`segment` を内包する外側
    /// コンテナ、`fandhe_frontend_headless_ui::signature_pad::control`
    /// 参照）の余白部分（SVG の外側だが `control` 内側）でのクリックは
    /// `segment` がイベントターゲットの祖先ではなく子孫であるため
    /// `closest` では解決できない（[`is_drawable_part`] は `control` を
    /// 描画可能パーツとして許可しているにもかかわらず、`pointerdown` が
    /// 早期リターンしてストローク開始を取りこぼす不具合、Bugbot 指摘・
    /// イシュー #843 PR #872）。そのため `closest` が失敗した場合、
    /// `target` 自身を起点に子孫方向へ `segment` を探す `query_selector`
    /// へフォールバックする（`control` 上のクリックは高々 1 つの
    /// `segment` 子を持つ想定のため、最初に見つかった要素を採用する）。
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
        const SEGMENT_SELECTOR: &str = r#"[data-scope="signature-pad"][data-part="segment"]"#;
        let segment = target
            .closest(SEGMENT_SELECTOR)
            .ok()
            .flatten()
            .or_else(|| target.query_selector(SEGMENT_SELECTOR).ok().flatten())?;
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

    /// SignaturePad Root（`[data-scope="signature-pad"][data-part="root"]`）
    /// を探すセレクタ（モジュール doc「pointer capture の再付与」節参照、
    /// イシュー #1993）。
    const ROOT_SELECTOR: &str = r#"[data-scope="signature-pad"][data-part="root"]"#;

    /// SignaturePad Control（`[data-scope="signature-pad"][data-part="control"]`）
    /// を探すセレクタ。anatomy 上 Root 1 個につき Control は 1 個であるため
    /// [`resolve_control_by_root_id`] は Root `id` から一意に定まる。
    const CONTROL_SELECTOR: &str = r#"[data-scope="signature-pad"][data-part="control"]"#;

    /// `pointerdown` で実際に追跡を開始した場合、以後の capture 再付与に
    /// 使う再解決キー（SignaturePad Root の `id`）を採取する。
    ///
    /// `target` から祖先方向へ `closest` で Root を探し、`runtime_root`
    /// （`Runtime` のマウントコンテナ。SignaturePad Root パーツ自身とは別
    /// 要素）の子孫であることを確認してから `id` を読む。Root が見つから
    /// ない・`id` が空（`SignaturePad::view()` の正準ビューは Root に `id`
    /// を付与しない）場合はいずれも `None`（fail-closed。呼び出し側は
    /// 再解決を試みず従来の `event.target()` ベース経路のみで動作する）。
    fn capture_anchor_root_id(runtime_root: &Element, target: &Element) -> Option<String> {
        let part_root = target.closest(ROOT_SELECTOR).ok().flatten()?;
        if !runtime_root.contains(Some(&part_root)) {
            return None;
        }
        let root_id = part_root.id();
        if root_id.is_empty() {
            return None;
        }
        Some(root_id)
    }

    /// [`capture_anchor_root_id`] が採取した Root `id` から、`runtime_root`
    /// 配下の SignaturePad Control 要素を再解決する（`angle_slider.rs` の
    /// `resolve_part`（`PartKey::RootId` 分岐）と同型のパターンだが、本
    /// モジュール専用の private 実装として閉じる）。
    ///
    /// - 同一 `id` の要素が `runtime_root` 配下に無い、または
    ///   `runtime_root` の子孫でない（`root` 外の同名 `id` を誤って掴まない
    ///   ための検証）場合は `None`
    /// - `data-scope`/`data-part` が SignaturePad Root と一致しない場合は
    ///   `None`
    /// - Root 配下の Control 一致要素が 1 個以外（0 個・複数個）の場合は
    ///   `None`（anatomy 上通常は 1 個のみだが、崩れた構成で誤った要素へ
    ///   capture を移さないための fail-closed 確認）
    fn resolve_control_by_root_id(runtime_root: &Element, root_id: &str) -> Option<Element> {
        let document = runtime_root.owner_document()?;
        let part_root = document.get_element_by_id(root_id)?;
        if !runtime_root.contains(Some(&part_root)) {
            return None;
        }
        if part_root.get_attribute("data-scope").as_deref() != Some(SIGNATURE_PAD_SCOPE)
            || part_root.get_attribute("data-part").as_deref() != Some("root")
        {
            return None;
        }
        let list = part_root.query_selector_all(CONTROL_SELECTOR).ok()?;
        if list.length() != 1 {
            return None;
        }
        list.get(0)?.dyn_into::<Element>().ok()
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
    /// pointermove 配線は [`StrokeCollector::release_if_stale`] による
    /// stale 自己解錠ガードに加え、capture 再付与（イシュー #1993、
    /// モジュール doc「pointer capture の再付与」節参照）を含む。
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
        // capture 再付与の再解決キー（追跡開始時に採取した SignaturePad
        // Root の `id`）。`is_tracking()` が真の間のみ `Some` を維持する
        // 不変条件（モジュール doc・[`capture_anchor_root_id`] 参照、
        // イシュー #1993）。
        let anchor: std::rc::Rc<std::cell::RefCell<Option<String>>> =
            std::rc::Rc::new(std::cell::RefCell::new(None));

        // pointerdown: 描画領域（[`is_drawable_part`]）内のみ追跡開始する
        // （ClearTrigger 等の無関係パーツ上での押下を描画開始と誤認しない
        // fail-closed ガード）。
        {
            let down_root = root.clone();
            let down_collector = collector.clone();
            let down_anchor = anchor.clone();
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
                    // 追跡を実際に開始した場合（既に他ポインタを追跡中で
                    // 無視された 2 本目の pointerdown ではない場合）のみ
                    // 再解決キーを採取・更新する（anchor 不変条件、
                    // struct doc「単一ストローク・単一ポインタの追跡
                    // モデル」参照）。
                    let was_tracking = c.is_tracking();
                    c.on_pointer_down(
                        event.pointer_id(),
                        (client_x - origin_x) * scale_x,
                        (client_y - origin_y) * scale_y,
                    );
                    if !was_tracking && c.is_tracking() {
                        if let Ok(mut a) = down_anchor.try_borrow_mut() {
                            *a = capture_anchor_root_id(&down_root, &target);
                        }
                    }
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
            let move_anchor = anchor.clone();
            let closure = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
                let Ok(mut c) = move_collector.try_borrow_mut() else {
                    return;
                };
                if c.release_if_stale(event.pointer_id(), event.buttons()) {
                    // capture 喪失中に `root` 外で pointerup/pointercancel を
                    // 取り逃した stale な追跡（`StrokeCollector::
                    // release_if_stale` doc 参照）。追跡を解除したので
                    // この move イベント自体の座標は破棄する（fail-closed）。
                    if let Ok(mut a) = move_anchor.try_borrow_mut() {
                        a.take();
                    }
                    return;
                }
                if !c.is_tracking() {
                    return;
                }
                let Some(event_target) = event.target().and_then(|t| t.dyn_into::<Element>().ok())
                else {
                    return;
                };
                // capture 再付与（イシュー #1993、モジュール doc「pointer
                // capture の再付与」節参照）: このイベントの pointer が
                // 追跡中の pointer と一致し、かつ `event_target` が capture
                // を保持していない（＝喪失した。通常経路では
                // `set_pointer_capture` によりブラウザが `event.target()`
                // を capture 要素へ retarget するため、保持中は
                // `has_pointer_capture` が真になり本分岐へ入らない）場合に
                // 限り、anchor の Root `id` から Control を再解決して
                // capture を掛け直す。座標変換の基準（`target`）も
                // 再解決した Control に切り替える（`event_target` が
                // ClearTrigger 等の非描画パーツでも座標が落ちないように
                // するため）。
                let target = if c.active_pointer_id() == Some(event.pointer_id())
                    && !event_target.has_pointer_capture(event.pointer_id())
                {
                    let root_id = move_anchor.try_borrow().ok().and_then(|a| a.clone());
                    match root_id.and_then(|id| resolve_control_by_root_id(&move_root, &id)) {
                        Some(control) => {
                            if !control.has_pointer_capture(event.pointer_id()) {
                                // 合成イベント（`wasm_bindgen_test`）では
                                // `NotFoundError` を投げるが、以後の追跡継続
                                // 判定は `StrokeCollector` 側の pointer_id
                                // 一致で行うため無視して構わない
                                // （`angle_slider.rs::reattach_pointer_capture`
                                // と同じ扱い）。
                                let _ = control.set_pointer_capture(event.pointer_id());
                            }
                            control
                        }
                        None => {
                            // Root が消えた・Control が一意に定まらない。
                            // 座標列は破棄せず、anchor のみ解除して従来の
                            // `event.target()` ベース経路へ戻す
                            // （fail-closed。以後 `root` 外へ離脱した場合の
                            // 回収は #1992 の stale ガードに委ねる）。
                            if let Ok(mut a) = move_anchor.try_borrow_mut() {
                                a.take();
                            }
                            event_target
                        }
                    }
                } else {
                    event_target
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
            let up_anchor = anchor.clone();
            let closure = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
                let payload = {
                    let Ok(mut c) = up_collector.try_borrow_mut() else {
                        return;
                    };
                    let payload = c.on_pointer_up(event.pointer_id());
                    if !c.is_tracking() {
                        if let Ok(mut a) = up_anchor.try_borrow_mut() {
                            a.take();
                        }
                    }
                    payload
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
            let cancel_anchor = anchor.clone();
            let closure = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
                if let Ok(mut c) = cancel_collector.try_borrow_mut() {
                    c.on_pointer_cancel(event.pointer_id());
                    if !c.is_tracking() {
                        if let Ok(mut a) = cancel_anchor.try_borrow_mut() {
                            a.take();
                        }
                    }
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
        // 2 番目の pointerdown は無視されるため、追跡中の pointer は 1 のまま
        // （`wiring::wire_stroke_collector` の capture 再付与用 anchor が、
        // この無視された pointerdown で上書きされてはならない不変条件の
        // 根拠、イシュー #1993）。
        assert_eq!(collector.active_pointer_id(), Some(1));
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

    #[test]
    fn active_pointer_id_reflects_tracking_lifecycle() {
        // `wiring::wire_stroke_collector` の capture 再付与（イシュー
        // #1993）は本 getter で「このイベントの pointer が追跡中の
        // pointer と一致するか」を判定するため、追跡開始/終了に正しく
        // 追随することを固定する。
        let mut collector = StrokeCollector::new();
        assert_eq!(collector.active_pointer_id(), None);
        collector.on_pointer_down(1, 0.0, 0.0);
        assert_eq!(collector.active_pointer_id(), Some(1));
        let _ = collector.on_pointer_up(1);
        assert_eq!(collector.active_pointer_id(), None);
    }

    #[test]
    fn active_pointer_id_is_none_after_cancel_and_stale_release() {
        let mut collector = StrokeCollector::new();
        collector.on_pointer_down(1, 0.0, 0.0);
        collector.on_pointer_cancel(1);
        assert_eq!(collector.active_pointer_id(), None);

        collector.on_pointer_down(2, 0.0, 0.0);
        assert_eq!(collector.active_pointer_id(), Some(2));
        assert!(collector.release_if_stale(2, 0));
        assert_eq!(collector.active_pointer_id(), None);
    }

    #[test]
    fn release_if_stale_releases_stale_tracking_when_no_button_is_held() {
        let mut collector = StrokeCollector::new();
        collector.on_pointer_down(1, 0.0, 0.0);
        collector.on_pointer_move(1, 10.0, 10.0);
        assert!(collector.release_if_stale(1, 0));
        assert!(!collector.is_tracking());
        // 座標列は破棄されているため、同じ pointer_id の pointerup は
        // 無関係な ID として無視される（`on_pointer_cancel` と同じ契約）。
        assert_eq!(collector.on_pointer_up(1), None);
    }

    #[test]
    fn release_if_stale_keeps_tracking_when_button_is_held() {
        let mut collector = StrokeCollector::new();
        collector.on_pointer_down(1, 0.0, 0.0);
        assert!(!collector.release_if_stale(1, 1));
        assert!(collector.is_tracking());
        collector.on_pointer_move(1, 10.0, 10.0);
        let payload = collector.on_pointer_up(1).unwrap();
        assert_eq!(payload, "0.00,0.00 10.00,10.00");
    }

    #[test]
    fn release_if_stale_is_noop_when_not_tracking() {
        let mut collector = StrokeCollector::new();
        assert!(!collector.release_if_stale(1, 0));
        assert!(!collector.is_tracking());
    }

    #[test]
    fn release_if_stale_ignores_mismatched_pointer_id() {
        // PR #1999 レビュー指摘の回帰: 別デバイス（例: マウス）が root 上で
        // hover するだけで buttons == 0 の pointermove が発火し得るため、
        // 追跡中の pointer_id と異なる ID では stale 判定してはならない
        // （描画中の正当なストロークを誤って破棄しない）。
        let mut collector = StrokeCollector::new();
        collector.on_pointer_down(1, 0.0, 0.0);
        collector.on_pointer_move(1, 10.0, 10.0);
        assert!(!collector.release_if_stale(2, 0));
        assert!(collector.is_tracking());
        let payload = collector.on_pointer_up(1).unwrap();
        assert_eq!(payload, "0.00,0.00 10.00,10.00");
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

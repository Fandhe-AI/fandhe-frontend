//! pointer capture ベースの汎用ドラッグ演算層（イシュー #2535、親 #2530）。
//!
//! `crates/wasm-full/src/drag_gesture.rs` が pointer/keyboard イベントから
//! 抽出した座標・タイムスタンプのみをここへ渡し、軸制約・範囲クランプ・
//! 離脱速度推定・spring 復帰の計算と DOM 書き込みは本モジュールが担う
//! （3 層構成、`docs/design/motion-reference-adoption-policy.md` §6）。
//! `getBoundingClientRect` によるドラッグ範囲コンテナの実測は `wasm-full`
//! 側の責務とし（`angle_slider.rs`/`sidebar.rs` が同種の計測を既に担う
//! 前例を踏襲）、本モジュールは数値化済みの [`DragConstraint`] を受け取る
//! のみで `web-sys` の `DomRect`/`DomRectReadOnly`/`Element` feature を
//! 追加しない（`Cargo.toml` の対応コメント参照）。
//!
//! # 位置の書き込み先
//!
//! [`dom_target::DomTarget`] と同じく `transform` を直接上書きせず、
//! CSS カスタムプロパティ 2 本（[`DRAG_X_PROPERTY`]/[`DRAG_Y_PROPERTY`]、
//! px 単位）へ書く。利用者は自分の CSS で
//! `transform: translate(var(--fandhe-drag-x, 0px), var(--fandhe-drag-y, 0px))`
//! のように消費する。
//!
//! # `touch-action: none`
//!
//! [`DragController::attach`] が CSSOM
//! （`style().set_property("touch-action", "none")`）で無条件に設定する。
//! タッチでのドラッグはブラウザのスクロール処理と競合し、無いと
//! `pointercancel` がスクロール開始と共に発火してドラッグが中断するため
//! （`content_height.rs` の「CSSOM 経由でプロパティのみ更新、既存インライン
//! 宣言を破壊しない」方針を踏襲）。

use std::cell::RefCell;
use std::rc::Rc;

use fandhe_animation::driver::Driver;
use fandhe_animation::interpolate::Vec2;
use fandhe_animation::spring::{Spring, SpringConfig};
use fandhe_animation::target::Target;
use web_sys::HtmlElement;

use crate::dom_target::DomTarget;
use crate::raf_driver::{AnimationLoop, RafDriver};
use crate::reduced_motion::prefers_reduced_motion;

/// ドラッグ位置の X 成分を書き込む CSS カスタムプロパティ名。
pub const DRAG_X_PROPERTY: &str = "--fandhe-drag-x";
/// ドラッグ位置の Y 成分を書き込む CSS カスタムプロパティ名。
pub const DRAG_Y_PROPERTY: &str = "--fandhe-drag-y";

/// 最後の `pointermove` サンプルから release までの経過時間（ms）が
/// これを超えたら、離脱速度を 0（停止）とみなす（Bugbot 指摘「Stale
/// velocity after paused drag」是正、PR #2565 第 2 ラウンド）。
///
/// [`DragController::on_release`] は直近 2 サンプルから速度を推定する
/// （[`estimate_velocity`]）が、そのサンプルは「最後に `pointermove` が
/// 発火した時刻」の位置しか記録しない。ユーザーが範囲外へ出した状態で
/// 静止してからしばらく経って指/マウスを離した場合、直近サンプルは
/// 静止前の速い動きを反映したままであり、実際には停止しているにも
/// 関わらず古い速度で spring がスナップバックしてしまう。Motion
/// （`docs/design/motion-reference-adoption-policy.md` 参照）の
/// pointer-events 実装が採用する 100ms 前後の「最終操作からの経過が
/// これを超えたら速度を打ち切る」慣例に合わせた値。
const STALE_VELOCITY_THRESHOLD_MS: f64 = 100.0;

/// 軸制約。ドラッグ中・キーボード移動（[`DragController::nudge`]）の
/// いずれにも継続的に適用する構造的な制約（範囲クランプとは別に扱う、
/// 下記 [`DragConstraint`] 参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DragAxis {
    /// 2 軸自由（省略時の既定）。
    #[default]
    Free,
    /// X 軸のみ移動可能（Y 成分は常に固定）。
    X,
    /// Y 軸のみ移動可能（X 成分は常に固定）。
    Y,
}

/// `delta` を `axis` に従って絞り込む（許可されない軸の成分を 0 にする）。
#[must_use]
pub fn apply_axis(delta: Vec2, axis: DragAxis) -> Vec2 {
    match axis {
        DragAxis::Free => delta,
        DragAxis::X => Vec2 { x: delta.x, y: 0.0 },
        DragAxis::Y => Vec2 { x: 0.0, y: delta.y },
    }
}

/// ドラッグ範囲コンテナから実測した min/max（px、要素の相対オフセット
/// 座標系）。`wasm-full` 側が対象祖先要素の `getBoundingClientRect()` から
/// 算出して渡す（本モジュールは実測しない、モジュール doc 参照）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragConstraint {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

/// `pos` を `constraint` の範囲内へクランプする。
///
/// ドラッグ中は適用せず（自由移動、実装計画 §8）、release 時の最終位置
/// 決定にのみ使う。`min > max` のような不正な constraint（測定誤差等）は
/// `f64::clamp` の panic 契約を避けるため呼び出し前に正規化しておくこと
/// （[`DragController::set_constraint`] 参照）。
#[must_use]
pub fn clamp_to_constraint(pos: Vec2, constraint: DragConstraint) -> Vec2 {
    Vec2 {
        x: pos.x.clamp(constraint.min_x, constraint.max_x),
        y: pos.y.clamp(constraint.min_y, constraint.max_y),
    }
}

/// [`clamp_to_constraint`] を `axis` で許可された成分にのみ適用する。
///
/// `DragAxis::X`/`DragAxis::Y` の禁止軸は、その成分がドラッグ中
/// 一度も動いていなくても（[`apply_axis`] により常に起点のまま）
/// `constraint` の範囲外にあり得る（実測誤差・レイアウト変更等）ため、
/// 無条件クランプでは軸固定契約を破って禁止軸の値まで動かしてしまう
/// （codex-review P1 是正）。禁止軸の成分は `pos` の値をそのまま保持する。
#[must_use]
fn clamp_to_constraint_for_axis(pos: Vec2, constraint: DragConstraint, axis: DragAxis) -> Vec2 {
    let clamped = clamp_to_constraint(pos, constraint);
    match axis {
        DragAxis::Free => clamped,
        DragAxis::X => Vec2 {
            x: clamped.x,
            y: pos.y,
        },
        DragAxis::Y => Vec2 {
            x: pos.x,
            y: clamped.y,
        },
    }
}

/// `constraint` の `min <= max` を保証する正規化（不正な実測値による
/// `f64::clamp` panic を防ぐ、security.md A05 対応）。`min > max` の場合は
/// 2 値を入れ替える。
#[must_use]
fn normalize_constraint(constraint: DragConstraint) -> DragConstraint {
    let (min_x, max_x) = if constraint.min_x <= constraint.max_x {
        (constraint.min_x, constraint.max_x)
    } else {
        (constraint.max_x, constraint.min_x)
    };
    let (min_y, max_y) = if constraint.min_y <= constraint.max_y {
        (constraint.min_y, constraint.max_y)
    } else {
        (constraint.max_y, constraint.min_y)
    };
    DragConstraint {
        min_x,
        max_x,
        min_y,
        max_y,
    }
}

/// [`estimate_velocity`] が速度計算に使う最小経過時間（秒）。これ未満の
/// `dt` は「事実上同時刻のサンプル」として速度 0 を返す（イシュー #2541
/// codex-review 指摘・`carousel_motion_browser.rs` 実測 是正の一部: 合成
/// `PointerEvent` を待機なしで連続 dispatch すると `time_stamp()` の差が
/// サブミリ秒になり得る。下限を設けないと極小 `dt` による除算で非現実的
/// に巨大な速度を返す）。1ms は 1000Hz 相当の入力デバイスでも通常発生
/// しない下限であり、実運用の 60〜144Hz ポインタ入力（約 7〜16ms 間隔）
/// には影響しない。
const MIN_VELOCITY_DT_S: f64 = 0.001;

/// ドラッグ開始位置から release 時点までの**総移動距離**（px）がこれ
/// 未満なら「操作全体が微小だった」とみなし離脱速度を 0 として扱う
/// （[`is_travel_negligible`] 参照）。
///
/// # 直近 2 サンプル間の距離では判定しない理由（codex-review 指摘 是正、
/// イシュー #2541 第 2 ラウンド）
///
/// 当初は本閾値を [`estimate_velocity`] 内部で「直近 2 サンプル間の
/// 距離」に適用していたが、この判定は入力のサンプリング間隔に依存して
/// 結果が変わってしまう欠陥があった——例えば 8ms ごとに 2px 動く操作
/// （実際には 250px/s 相当の正当なフリック）でも、隣接 2 サンプルの
/// 距離だけを見ると常に閾値未満で速度 0 に潰れる。同じ実速度でも
/// サンプリング周期が高い（1 サンプルあたりの移動量が小さい）だけで
/// 結果が変わるのは誤り。是正として、本閾値は [`estimate_velocity`]
/// 自体からは外し、[`DragController::on_release`]/`crate::carousel::
/// CarouselTrack::on_release` が「ドラッグ開始位置から release 時点
/// までの累積移動距離」（[`is_travel_negligible`]）で「操作全体が微小
/// だったか」を判定してから、[`estimate_velocity`] が返す値をそのまま
/// 使うか 0 に倒すかを選ぶ（`small_move_settles_back_to_origin_index`
/// が検証する「2px だけ動かしてすぐ離す」ケースは、累積移動距離も
/// 同じく 2px のため引き続き速度 0 になる）。値は
/// `fandhe_frontend_wasm_full::carousel_motion::CLICK_GUARD_PX`
/// （5.0px、合成 click 抑止のドラッグ判定閾値）よりわずかに小さい
/// 3.0px とし、「ドラッグと認識されるほどの移動」はこの下限に阻まれず
/// 従来どおり速度を反映する。
const MIN_VELOCITY_DISTANCE_PX: f64 = 3.0;

/// `total_distance_px`（ドラッグ開始位置から release 時点までの累積
/// 移動距離、px）が [`MIN_VELOCITY_DISTANCE_PX`] 未満で「操作全体が
/// 微小だった」かを判定する（[`MIN_VELOCITY_DISTANCE_PX`] doc 参照）。
/// `NaN` は非数のため安全側（微小＝真）に倒す。
#[must_use]
pub(crate) fn is_travel_negligible(total_distance_px: f64) -> bool {
    !total_distance_px.is_finite() || total_distance_px < MIN_VELOCITY_DISTANCE_PX
}

/// 直近 2 サンプル（位置・`performance.now()`/`event.time_stamp()` 相当の
/// ms タイムスタンプ）から離脱速度（px/秒）を推定する。
///
/// いずれかのサンプルが欠けている・時間差が[`MIN_VELOCITY_DT_S`]未満
/// （0 以下を含む。同時刻の重複サンプル・クロックの逆行・測定不能なほど
/// 近接した連続サンプル）場合は `Vec2::default()`（速度 0）を返す
/// （spring の初速 0 は「行き過ぎない」安全側の既定値）。「操作全体が
/// 微小だったか」（[`MIN_VELOCITY_DISTANCE_PX`]/[`is_travel_negligible`]）
/// は呼び出し側の責務であり、本関数自体は距離で速度をゼロにしない
/// （[`MIN_VELOCITY_DISTANCE_PX`] doc「直近 2 サンプル間の距離では判定
/// しない理由」節参照）。
#[must_use]
pub fn estimate_velocity(previous: Option<(Vec2, f64)>, latest: Option<(Vec2, f64)>) -> Vec2 {
    let (Some((p0, t0)), Some((p1, t1))) = (previous, latest) else {
        return Vec2::default();
    };
    let dt = (t1 - t0) / 1000.0;
    if !dt.is_finite() || dt < MIN_VELOCITY_DT_S {
        return Vec2::default();
    }
    Vec2 {
        x: (p1.x - p0.x) / dt,
        y: (p1.y - p0.y) / dt,
    }
}

/// `latest`（直近の `pointermove` サンプルとその時刻）から `time_ms`
/// （release 時刻）までの経過が [`STALE_VELOCITY_THRESHOLD_MS`] を超えて
/// いるかを判定する（[`DragController::on_release`] doc 参照。Bugbot
/// 指摘「Stale velocity after paused drag」是正、PR #2565 第 2 ラウンド）。
///
/// サンプルが無い（`None`）場合は「経過が測れない」ため stale とは
/// 判定しない（`estimate_velocity` 側が `None` を渡されたときと同じ
/// 「速度 0」へ自然に帰着するため、ここで重複判定しない）。`time_ms` が
/// `sampled_at` より小さい（クロックの逆行）場合も差分が非正になり
/// 閾値を超えないため stale 側へは倒れない（安全側: 実際に停止していない
/// ケースを誤って速度 0 にしない）。
// イシュー #2541: `crate::carousel::CarouselTrack::on_release` が同じ
// 「直近サンプルの陳腐化判定」を再利用するため、crate 内へ可視性を広げる
// （呼び出し元・契約は本モジュール内の `DragController::on_release` と同一）。
#[must_use]
pub(crate) fn is_velocity_stale(latest: Option<(Vec2, f64)>, time_ms: f64) -> bool {
    latest.is_some_and(|(_, sampled_at)| (time_ms - sampled_at) > STALE_VELOCITY_THRESHOLD_MS)
}

/// `element` の CSS カスタムプロパティ 2 本へ `pos` を書き込む。
///
/// モジュール doc の契約（`translate(var(--fandhe-drag-x, 0px), ...)`）
/// どおり px 単位を付与する。`DomTarget::custom_property`（単位なし）を
/// そのまま使うと非ゼロ値が `transform` として無効になり要素が動かない
/// ため、`style_property` へ `"px"` 単位を渡す（codex-review/Bugbot 是正）。
fn write_dom(element: &HtmlElement, pos: Vec2) {
    DomTarget::style_property(element.clone(), DRAG_X_PROPERTY, "px").write(pos.x);
    DomTarget::style_property(element.clone(), DRAG_Y_PROPERTY, "px").write(pos.y);
}

/// `pointerdown` 時点のドラッグ起点（クライアント座標・その時点の位置）。
#[derive(Debug, Clone, Copy, PartialEq)]
struct DragStart {
    client: Vec2,
    origin_position: Vec2,
}

/// [`DragController::set_axis`] doc「ドラッグ中の起点を再構築する」節
/// 参照（codex-review P1 是正、PR #2565 第 4 ラウンド）。ドラッグ中に軸を
/// 切り替える際、現在の保持位置（`current_position`）を新たな
/// `origin_position` とし、直近の入力座標（`last_client`。無ければ
/// `current_position` にフォールバック）を新たな起点クライアント座標
/// とする。以降の `on_pointer_move` は「軸変更時点」を起点として delta
/// を計算するため、変更前に確定していた成分が巻き戻らない。
#[must_use]
fn rebuild_start_for_axis_change(current_position: Vec2, last_client: Option<Vec2>) -> DragStart {
    DragStart {
        client: last_client.unwrap_or(current_position),
        origin_position: current_position,
    }
}

/// [`DragController::set_axis`] doc「軸が実際に変わらない呼び出しは
/// no-op」節参照（codex-review P1・Bugbot Medium 是正、PR #2565 第 5
/// ラウンド）。`old`/`new` が同じ [`DragAxis`] なら `false`（no-op すべき）
/// を返す。
#[must_use]
fn axis_changed(old: DragAxis, new: DragAxis) -> bool {
    old != new
}

/// 1 要素分のドラッグ状態を保持し、pointer/keyboard から抽出された座標を
/// 受け取って範囲制約・spring 復帰を計算し DOM へ反映する。
///
/// `crates/wasm-full/src/drag_gesture.rs` の配線層が opt-in 要素ごとに
/// 1 個保持し、`pointerdown`/`pointermove`/`pointerup`/`pointercancel`/
/// `keydown`（矢印キー）のハンドラから対応するメソッドを呼ぶ。
pub struct DragController {
    element: HtmlElement,
    axis: DragAxis,
    constraint: Option<DragConstraint>,
    /// 現在位置（release 用 spring ループからも書き込まれるため
    /// `Rc<RefCell<..>>` で共有する）。
    position: Rc<RefCell<Vec2>>,
    start: Option<DragStart>,
    /// 直近の入力座標（`pointerdown`/`pointermove` の `client`、軸適用
    /// 前の raw 値）。[`Self::set_axis`] がドラッグ中に軸を切り替える際、
    /// この座標と現在位置から [`DragStart`] を再構築する（codex-review
    /// P1 是正、PR #2565 第 4 ラウンド）。
    last_client: Option<Vec2>,
    previous_sample: Option<(Vec2, f64)>,
    latest_sample: Option<(Vec2, f64)>,
    /// release 時の spring 復帰ループ（進行中のみ `Some`）。`Drop` で
    /// 確実に `cancelAnimationFrame` される（`AnimationLoop` の契約）。
    release_anim: Option<AnimationLoop>,
}

impl DragController {
    /// `element` にドラッグ演算を紐づける。`touch-action: none` を
    /// CSSOM で設定する（モジュール doc 参照）。
    #[must_use]
    pub fn attach(element: HtmlElement, axis: DragAxis) -> Self {
        let _ = element.style().set_property("touch-action", "none");
        Self {
            element,
            axis,
            constraint: None,
            position: Rc::new(RefCell::new(Vec2::default())),
            start: None,
            last_client: None,
            previous_sample: None,
            latest_sample: None,
            release_anim: None,
        }
    }

    /// ドラッグ範囲コンテナの実測結果を更新する（`None` は無制約）。
    /// `wasm-full` 側が `pointerdown` のたびに祖先要素の
    /// `getBoundingClientRect()` を再計測して呼ぶ想定（レイアウト変更へ
    /// 追随するため、`attach` 時の 1 回きりでは固定できない）。
    pub fn set_constraint(&mut self, constraint: Option<DragConstraint>) {
        self.constraint = constraint.map(normalize_constraint);
    }

    /// 軸制約を更新する（現在位置は保持したまま軸のみ切り替える。
    /// codex-review P1 是正、PR #2565 第 3・第 4 ラウンド）。
    ///
    /// keyed list の既存行更新で [`crate::drag`] 呼び出し元
    /// （`crates/wasm-full/src/drag_gesture.rs::DRAG_AXIS_ATTR`）の値が
    /// `x` ⇔ `y` へ変わっても、`controller_for` は既存 [`DragController`]
    /// をそのまま返すだけで軸を読み直さない（軸は [`Self::attach`] 時に
    /// しか読まれない）。`wasm-full` 側の再同期経路が本メソッドで最新の
    /// 属性値を反映する。
    ///
    /// # ドラッグ中の起点を再構築する（codex-review P1 是正、PR #2565
    /// 第 4 ラウンド）
    ///
    /// 軸のみを書き換えて [`DragStart`]（`pointerdown` 時点の起点）を
    /// そのまま残すと、次の `pointermove` は「軸変更前の起点」からの
    /// 相対移動を計算する。例えば X 軸で `(0, 0)` から `(50, 0)` へ移動
    /// 後に軸を Y へ変更すると、次の `pointermove` が `(60, 10)` を渡した
    /// 場合、旧起点からの delta は `(60, 10)`（軸 Y 適用で `(0, 10)`）と
    /// なり、次の位置は `origin_position(0,0) + (0,10) = (0,10)` と
    /// 計算されてしまい、直前まで保持していた X 座標 `50` が巻き戻る
    /// （「位置保持・軸固定」契約への違反）。ドラッグ中（`self.start` が
    /// `Some`）であれば、現在の保持位置（[`Self::position`]）と直近の
    /// 入力座標（`last_client`）で [`DragStart`] を再構築し、
    /// 以降の `pointermove` が「軸変更時点」を新たな起点として計算する
    /// ようにする。
    ///
    /// # 進行中の release spring を打ち切る
    ///
    /// release 後の spring 復帰（`release_anim`）は旧軸の
    /// [`clamp_to_constraint_for_axis`] で計算した目標へ向かって進行
    /// するため、軸変更後もそのまま動かすと旧軸の目標へ向かい続けて
    /// しまう。ドラッグ中でなくても軸変更時は spring を打ち切り、現在
    /// 位置で停止させる（新軸での目標再計算は次回の `on_release` に
    /// 委ねる、`Self::nudge` と同じ「軸変更は進行中の運動を打ち切る」
    /// 方針）。
    ///
    /// # 軸が実際に変わらない呼び出しは no-op（codex-review P1・Bugbot
    /// Medium 是正、PR #2565 第 5 ラウンド）
    ///
    /// `crates/wasm-full/src/drag_gesture.rs::resync_one_drag_element`
    /// は keyed list の Update 後、`data-fandhe-drag-axis` の値が
    /// 変わっていない要素にも本メソッドを呼ぶ（属性値ではなく
    /// 「既存コントローラを再利用したか」で呼び出し対象を決めている
    /// ため）。無条件に `release_anim` を破棄すると、範囲外から
    /// spring で復帰中の要素が、同じ行の無関係な内容変更や別行の
    /// 追加・更新だけで復帰を打ち切られ、範囲外に取り残される
    /// （release 後に制約内へ復帰する契約への違反）。軸が実際に
    /// 変わった場合（`axis != self.axis`）のみ、上記の中断・起点
    /// 再構築を行う。
    pub fn set_axis(&mut self, axis: DragAxis) {
        if !axis_changed(self.axis, axis) {
            return;
        }
        self.axis = axis;
        self.release_anim = None;
        if self.start.is_some() {
            self.start = Some(rebuild_start_for_axis_change(
                self.position(),
                self.last_client,
            ));
        }
    }

    /// 現在の書き込み済み位置（テスト・呼び出し元の状態確認用）。
    #[must_use]
    pub fn position(&self) -> Vec2 {
        *self.position.borrow()
    }

    /// 現在ポインタドラッグが進行中か（`on_pointer_down` 済み・
    /// `on_release`/`nudge` 未実行）。
    ///
    /// `crates/wasm-full/src/drag_gesture.rs::resync_one_drag_element`
    /// が、ドラッグ中に keyed list の Update で
    /// `data-fandhe-dragging` 属性が失われた際、この状態に基づいて
    /// 属性を復元すべきかを判定する（codex-review P1 是正、PR #2565
    /// 第 4 ラウンド）。
    #[must_use]
    pub fn is_dragging(&self) -> bool {
        self.start.is_some()
    }

    /// `touch-action: none` と現在の保持位置（[`DRAG_X_PROPERTY`]/
    /// [`DRAG_Y_PROPERTY`]）を要素へ再適用する（codex-review P1 是正、
    /// PR #2565 第 2 ラウンド）。
    ///
    /// keyed list の既存行更新（`fandhe_frontend_wasm_client::keyed_dom`
    /// の `sync_attrs`）は新しい view に無い `style` 属性を削除・上書き
    /// するため、[`Self::attach`] が一度設定した `touch-action: none` や
    /// 既に移動済みの CSS カスタムプロパティが失われうる。
    /// `crates/wasm-full/src/drag_gesture.rs::resync_drag_gesture_attachments`
    /// は既存コントローラに対してもこのメソッドを呼び、`Self` が保持する
    /// 論理位置（[`Self::position`]）を再度 DOM へ書き戻すことで、表示
    /// 位置と保持位置の食い違い（次操作での跳ね）を防ぐ。
    ///
    /// # 新規 attach 直後には呼ばない契約（Bugbot 是正、PR #2565 第 3
    /// ラウンド）
    ///
    /// 本メソッドは保持位置（既定 `(0, 0)`）を無条件に DOM へ明示
    /// 書き込みするため、一度もドラッグされていない新規要素へ呼ぶと
    /// 「未ドラッグの要素はカスタムプロパティが未設定（`None`）」という
    /// 既存契約を破ってしまう（`attach` 自身は `touch-action` のみ設定し
    /// 位置は書かないため、この差が生まれる）。呼び出し元
    /// （`resync_drag_gesture_attachments`）は既存コントローラを再利用
    /// した場合にのみ本メソッドを呼ぶ契約とし、新規 attach 直後には
    /// 呼ばない。
    pub fn resync_dom(&self) {
        let _ = self.element.style().set_property("touch-action", "none");
        write_dom(&self.element, self.position());
    }

    /// `pointerdown` 相当の入力。進行中の release spring を打ち切り、
    /// ドラッグ起点を記録する。
    pub fn on_pointer_down(&mut self, client: Vec2, time_ms: f64) {
        self.release_anim = None;
        let origin_position = self.position();
        self.start = Some(DragStart {
            client,
            origin_position,
        });
        self.last_client = Some(client);
        self.previous_sample = None;
        self.latest_sample = Some((origin_position, time_ms));
    }

    /// `pointermove` 相当の入力。ドラッグ中は自由移動（範囲クランプは
    /// release 時のみ、モジュール doc 参照）で、軸制約のみ継続適用する。
    /// `on_pointer_down` を経ていない（`start` が `None`）呼び出しは無視する。
    pub fn on_pointer_move(&mut self, client: Vec2, time_ms: f64) {
        let Some(start) = self.start else {
            return;
        };
        let raw_delta = Vec2 {
            x: client.x - start.client.x,
            y: client.y - start.client.y,
        };
        let delta = apply_axis(raw_delta, self.axis);
        let next = Vec2 {
            x: start.origin_position.x + delta.x,
            y: start.origin_position.y + delta.y,
        };
        self.write_position(next);
        self.last_client = Some(client);
        self.previous_sample = self.latest_sample;
        self.latest_sample = Some((next, time_ms));
    }

    /// `pointerup`/`pointercancel` 相当の入力。離脱速度を推定し、範囲
    /// 制約があれば spring で範囲内へ復帰させる（`prefers-reduced-motion`
    /// が真、または制約が無い/既に範囲内の場合は即時反映）。
    /// ドラッグ中でなかった（`start` が `None`）呼び出しは無視する。
    ///
    /// `time_ms` は呼び出し元イベント（`pointerup`/`pointercancel`/
    /// `keydown` 等）の `event.time_stamp()` 相当を渡すこと。直近の
    /// `pointermove` サンプルからこの時刻までの経過が
    /// [`STALE_VELOCITY_THRESHOLD_MS`] を超えていれば、離脱速度を
    /// 0（停止）とみなす（Bugbot 指摘「Stale velocity after paused
    /// drag」是正、PR #2565 第 2 ラウンド。範囲外で静止してから離した
    /// 場合に、静止前の古い速度で spring がスナップバックする不具合の
    /// 是正）。
    pub fn on_release(&mut self, time_ms: f64) {
        let Some(start) = self.start.take() else {
            return;
        };
        let current = self.position();
        let total_distance =
            (current.x - start.origin_position.x).hypot(current.y - start.origin_position.y);
        let velocity = if is_velocity_stale(self.latest_sample, time_ms)
            || is_travel_negligible(total_distance)
        {
            Vec2::default()
        } else {
            estimate_velocity(self.previous_sample, self.latest_sample)
        };
        self.previous_sample = None;
        self.latest_sample = None;
        self.settle(current, velocity);
    }

    /// キーボード（矢印キー）による固定ステップ移動。範囲制約があれば
    /// 即時クランプする（spring は使わない、即時反映）。進行中の
    /// pointer ドラッグ・release spring は防御的に打ち切る。
    pub fn nudge(&mut self, step: Vec2) {
        self.release_anim = None;
        self.start = None;
        self.previous_sample = None;
        self.latest_sample = None;
        let filtered = apply_axis(step, self.axis);
        let current = self.position();
        let mut next = Vec2 {
            x: current.x + filtered.x,
            y: current.y + filtered.y,
        };
        if let Some(constraint) = self.constraint {
            next = clamp_to_constraint_for_axis(next, constraint, self.axis);
        }
        self.write_position(next);
    }

    /// 位置を即時反映する（DOM 書き込み + 保持している位置の更新）。
    fn write_position(&mut self, next: Vec2) {
        *self.position.borrow_mut() = next;
        write_dom(&self.element, next);
    }

    /// release 時の最終位置決定 + spring 復帰の開始。
    ///
    /// `velocity` は禁止軸の成分を [`apply_axis`] で 0 に絞ってから
    /// spring へ渡す（codex-review P1 是正、PR #2565 第 5 ラウンド）。
    /// 直近のポインタサンプルは軸変更前に記録されたものである可能性が
    /// あり（[`Self::set_axis`] は速度サンプル自体はリセットしない）、
    /// 例えば Free で両軸を範囲外へ動かしてから Y 軸へ変更し、追加の
    /// `pointermove` なしで release すると、`estimate_velocity` が返す
    /// 速度は禁止軸（X）の成分も非ゼロのままになる。禁止軸は
    /// `target.x == current.x`（クランプなし）で位置は動かないはずでも、
    /// 非ゼロ初速度を渡すと spring がその軸へも一時的に動いてしまい
    /// 「禁止軸を固定する」契約に違反する。
    fn settle(&mut self, current: Vec2, velocity: Vec2) {
        let velocity = apply_axis(velocity, self.axis);
        let target = match self.constraint {
            Some(constraint) => clamp_to_constraint_for_axis(current, constraint, self.axis),
            None => current,
        };
        if target == current || prefers_reduced_motion() {
            self.write_position(target);
            return;
        }
        let config = SpringConfig::default();
        let (Some(spring_x), Some(spring_y)) = (
            Spring::new(config, current.x, target.x, velocity.x),
            Spring::new(config, current.y, target.y, velocity.y),
        ) else {
            // 非有限値等、構築できない極端な入力は spring を使わず
            // 即時反映へフォールバックする（`Spring::new` の契約どおり
            // panic しない、ライブラリコードで unwrap しない）。
            self.write_position(target);
            return;
        };
        let Some(mut driver) = RafDriver::new() else {
            // 非ブラウザ環境（native/SSR）では rAF が存在しないため
            // 即時反映へフォールバックする（`RafDriver::new` の契約）。
            self.write_position(target);
            return;
        };
        let position = Rc::clone(&self.position);
        let element = self.element.clone();
        let elapsed = Rc::new(RefCell::new(0.0_f64));
        self.release_anim = Some(AnimationLoop::start(move || {
            let Some(dt) = driver.tick() else {
                return true;
            };
            let t = {
                let mut elapsed = elapsed.borrow_mut();
                *elapsed += dt;
                *elapsed
            };
            let state_x = spring_x.at(t);
            let state_y = spring_y.at(t);
            let next = Vec2 {
                x: state_x.value,
                y: state_y.value,
            };
            *position.borrow_mut() = next;
            write_dom(&element, next);
            !(state_x.done && state_y.done)
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_axis, axis_changed, clamp_to_constraint, clamp_to_constraint_for_axis,
        estimate_velocity, is_travel_negligible, is_velocity_stale, normalize_constraint,
        rebuild_start_for_axis_change, DragAxis, DragConstraint, DragStart,
        STALE_VELOCITY_THRESHOLD_MS,
    };
    use fandhe_animation::interpolate::Vec2;

    #[test]
    fn apply_axis_free_keeps_both_components() {
        let delta = Vec2 { x: 3.0, y: -4.0 };
        assert_eq!(apply_axis(delta, DragAxis::Free), delta);
    }

    #[test]
    fn apply_axis_x_zeroes_y() {
        let delta = Vec2 { x: 3.0, y: -4.0 };
        assert_eq!(apply_axis(delta, DragAxis::X), Vec2 { x: 3.0, y: 0.0 });
    }

    #[test]
    fn apply_axis_y_zeroes_x() {
        let delta = Vec2 { x: 3.0, y: -4.0 };
        assert_eq!(apply_axis(delta, DragAxis::Y), Vec2 { x: 0.0, y: -4.0 });
    }

    #[test]
    fn clamp_to_constraint_clamps_both_axes() {
        let constraint = DragConstraint {
            min_x: 0.0,
            max_x: 100.0,
            min_y: -50.0,
            max_y: 50.0,
        };
        assert_eq!(
            clamp_to_constraint(Vec2 { x: -10.0, y: 999.0 }, constraint),
            Vec2 { x: 0.0, y: 50.0 }
        );
        assert_eq!(
            clamp_to_constraint(Vec2 { x: 40.0, y: -999.0 }, constraint),
            Vec2 { x: 40.0, y: -50.0 }
        );
    }

    #[test]
    fn clamp_to_constraint_is_noop_within_bounds() {
        let constraint = DragConstraint {
            min_x: 0.0,
            max_x: 100.0,
            min_y: 0.0,
            max_y: 100.0,
        };
        let pos = Vec2 { x: 50.0, y: 50.0 };
        assert_eq!(clamp_to_constraint(pos, constraint), pos);
    }

    #[test]
    fn clamp_to_constraint_for_axis_x_leaves_y_untouched_out_of_bounds() {
        // DragAxis::X（Y 固定）の契約: Y が constraint の範囲外にあっても
        // クランプしてはならない（codex-review P1 是正の回帰）。
        let constraint = DragConstraint {
            min_x: 0.0,
            max_x: 100.0,
            min_y: -50.0,
            max_y: 50.0,
        };
        let pos = Vec2 { x: -10.0, y: 999.0 };
        assert_eq!(
            clamp_to_constraint_for_axis(pos, constraint, DragAxis::X),
            Vec2 { x: 0.0, y: 999.0 }
        );
    }

    #[test]
    fn clamp_to_constraint_for_axis_y_leaves_x_untouched_out_of_bounds() {
        let constraint = DragConstraint {
            min_x: 0.0,
            max_x: 100.0,
            min_y: -50.0,
            max_y: 50.0,
        };
        let pos = Vec2 {
            x: -999.0,
            y: 999.0,
        };
        assert_eq!(
            clamp_to_constraint_for_axis(pos, constraint, DragAxis::Y),
            Vec2 { x: -999.0, y: 50.0 }
        );
    }

    #[test]
    fn clamp_to_constraint_for_axis_free_clamps_both() {
        let constraint = DragConstraint {
            min_x: 0.0,
            max_x: 100.0,
            min_y: -50.0,
            max_y: 50.0,
        };
        let pos = Vec2 { x: -10.0, y: 999.0 };
        assert_eq!(
            clamp_to_constraint_for_axis(pos, constraint, DragAxis::Free),
            Vec2 { x: 0.0, y: 50.0 }
        );
    }

    #[test]
    fn normalize_constraint_swaps_inverted_bounds() {
        let constraint = DragConstraint {
            min_x: 100.0,
            max_x: 0.0,
            min_y: 50.0,
            max_y: -50.0,
        };
        let normalized = normalize_constraint(constraint);
        assert_eq!(normalized.min_x, 0.0);
        assert_eq!(normalized.max_x, 100.0);
        assert_eq!(normalized.min_y, -50.0);
        assert_eq!(normalized.max_y, 50.0);
    }

    #[test]
    fn estimate_velocity_returns_zero_without_two_samples() {
        assert_eq!(estimate_velocity(None, None), Vec2::default());
        assert_eq!(
            estimate_velocity(None, Some((Vec2::default(), 0.0))),
            Vec2::default()
        );
    }

    #[test]
    fn estimate_velocity_computes_px_per_second() {
        let previous = Some((Vec2 { x: 0.0, y: 0.0 }, 0.0));
        let latest = Some((Vec2 { x: 100.0, y: -50.0 }, 500.0));
        let velocity = estimate_velocity(previous, latest);
        assert!((velocity.x - 200.0).abs() < 1e-9);
        assert!((velocity.y - (-100.0)).abs() < 1e-9);
    }

    /// イシュー #2541 codex-review 指摘 是正の回帰（第 2 ラウンド）:
    /// `estimate_velocity` 自体はサンプル間の距離で速度をゼロにしない
    /// （高頻度サンプリングの正当なフリックを誤って潰さないため、
    /// `MIN_VELOCITY_DISTANCE_PX` doc「直近 2 サンプル間の距離では判定
    /// しない理由」節参照）。「操作全体が微小だったか」の判定は
    /// `is_travel_negligible` へ委譲する。
    #[test]
    fn estimate_velocity_does_not_floor_on_adjacent_sample_distance() {
        let previous = Some((Vec2 { x: 0.0, y: 0.0 }, 0.0));
        let latest = Some((Vec2 { x: 2.0, y: 0.0 }, 8.0));
        let velocity = estimate_velocity(previous, latest);
        assert!((velocity.x - 250.0).abs() < 1e-6);
    }

    #[test]
    fn is_travel_negligible_floors_total_distance() {
        assert!(is_travel_negligible(2.9));
        assert!(!is_travel_negligible(3.0));
        assert!(is_travel_negligible(f64::NAN));
    }

    #[test]
    fn estimate_velocity_returns_zero_for_non_positive_dt() {
        let previous = Some((Vec2 { x: 0.0, y: 0.0 }, 100.0));
        let latest = Some((Vec2 { x: 10.0, y: 10.0 }, 100.0));
        assert_eq!(estimate_velocity(previous, latest), Vec2::default());
        let latest_earlier = Some((Vec2 { x: 10.0, y: 10.0 }, 50.0));
        assert_eq!(estimate_velocity(previous, latest_earlier), Vec2::default());
    }

    /// Bugbot 指摘「Stale velocity after paused drag」是正の回帰
    /// （PR #2565 第 2 ラウンド）: サンプルなしは経過が測れないため
    /// stale と判定しない（`estimate_velocity` の `None` 分岐が既に
    /// 速度 0 を返すため、ここで重複して stale 扱いにする必要がない）。
    #[test]
    fn is_velocity_stale_returns_false_without_a_sample() {
        assert!(!is_velocity_stale(None, 1_000.0));
    }

    /// 直近サンプルからの経過が閾値以下なら stale ではない。
    #[test]
    fn is_velocity_stale_returns_false_within_threshold() {
        let latest = Some((Vec2::default(), 1_000.0));
        assert!(!is_velocity_stale(
            latest,
            1_000.0 + STALE_VELOCITY_THRESHOLD_MS
        ));
    }

    /// 直近サンプルからの経過が閾値を超えると stale（速度 0 とみなす
    /// べき停止状態）と判定する。
    #[test]
    fn is_velocity_stale_returns_true_beyond_threshold() {
        let latest = Some((Vec2::default(), 1_000.0));
        assert!(is_velocity_stale(
            latest,
            1_000.0 + STALE_VELOCITY_THRESHOLD_MS + 1.0
        ));
    }

    /// クロックの逆行（release 時刻がサンプルより前）は差分が非正になり
    /// 閾値を超えないため、stale 側へは倒れない（安全側、doc 参照）。
    #[test]
    fn is_velocity_stale_returns_false_when_release_precedes_sample() {
        let latest = Some((Vec2::default(), 1_000.0));
        assert!(!is_velocity_stale(latest, 500.0));
    }

    /// codex-review P1 是正の回帰（PR #2565 第 4 ラウンド）: X 軸で
    /// `(0, 0)` から `(50, 0)` へ移動した後に軸を Y へ変更すると、
    /// [`rebuild_start_for_axis_change`] が現在位置 `(50, 0)` を新たな
    /// `origin_position` として起点を再構築する。以降 `(60, 10)` へ
    /// `pointermove` した際の delta は新起点からの相対値になるため、
    /// 修正前は巻き戻っていた X 座標 `50` が保持されたまま Y 成分だけが
    /// 動くことを固定する（`apply_axis` は既存の純関数をそのまま使う）。
    #[test]
    fn rebuild_start_for_axis_change_preserves_position_locked_axis() {
        // X 軸ドラッグで (0, 0) から (50, 0) へ移動済みの状態
        // （origin_position=(0,0)・client=(0,0) の起点から
        // client=(50,0) まで移動した想定）。
        let current_position = Vec2 { x: 50.0, y: 0.0 };
        let last_client = Some(Vec2 { x: 50.0, y: 0.0 });

        let rebuilt = rebuild_start_for_axis_change(current_position, last_client);
        assert_eq!(
            rebuilt,
            DragStart {
                client: Vec2 { x: 50.0, y: 0.0 },
                origin_position: Vec2 { x: 50.0, y: 0.0 },
            }
        );

        // 軸を Y へ変更した後、(60, 10) へ pointermove した場合の
        // 次の位置（on_pointer_move と同じ計算式）。
        let next_client = Vec2 { x: 60.0, y: 10.0 };
        let raw_delta = Vec2 {
            x: next_client.x - rebuilt.client.x,
            y: next_client.y - rebuilt.client.y,
        };
        let delta = apply_axis(raw_delta, DragAxis::Y);
        let next = Vec2 {
            x: rebuilt.origin_position.x + delta.x,
            y: rebuilt.origin_position.y + delta.y,
        };
        assert_eq!(
            next,
            Vec2 { x: 50.0, y: 10.0 },
            "軸変更前に確定していた X=50 が巻き戻らず、Y だけが動くこと"
        );
    }

    /// `last_client` が無い（`pointerdown` を経ずに軸だけ変わるような
    /// 防御的ケース）場合は現在位置へフォールバックする。
    #[test]
    fn rebuild_start_for_axis_change_falls_back_to_current_position_without_last_client() {
        let current_position = Vec2 { x: 12.0, y: 34.0 };
        let rebuilt = rebuild_start_for_axis_change(current_position, None);
        assert_eq!(
            rebuilt,
            DragStart {
                client: current_position,
                origin_position: current_position,
            }
        );
    }

    /// codex-review P1・Bugbot Medium 是正の回帰（PR #2565 第 5
    /// ラウンド）: 同じ軸を渡された場合は「変更あり」と判定しない
    /// （`DragController::set_axis` の no-op ガードがこれで早期 return
    /// し、進行中の `release_anim`・起点を破壊しないことの基盤）。
    #[test]
    fn axis_changed_returns_false_for_same_axis() {
        assert!(!axis_changed(DragAxis::Free, DragAxis::Free));
        assert!(!axis_changed(DragAxis::X, DragAxis::X));
        assert!(!axis_changed(DragAxis::Y, DragAxis::Y));
    }

    /// 軸が実際に異なる場合は「変更あり」と判定する。
    #[test]
    fn axis_changed_returns_true_for_different_axis() {
        assert!(axis_changed(DragAxis::X, DragAxis::Y));
        assert!(axis_changed(DragAxis::Free, DragAxis::X));
        assert!(axis_changed(DragAxis::Y, DragAxis::Free));
    }

    /// codex-review P1 是正の回帰（PR #2565 第 5 ラウンド）:
    /// `DragController::settle` は spring 構築前に `apply_axis` で
    /// 速度を絞る。ご指摘のシナリオ（Free で両軸を範囲外へ動かし、
    /// 軸を Y へ変更後、追加の pointermove なしで release）を模し、
    /// 両軸に非ゼロ速度を持つサンプルに軸 Y の `apply_axis` を適用すると
    /// X 成分（禁止軸）が 0 になり、Y 成分（許可軸）は変わらないことを
    /// 固定する（`settle` 自身が `HtmlElement`/`RafDriver` を要し native
    /// では直接呼べないため、`settle` が内部で使う同じ純関数を直接
    /// 検証する）。
    #[test]
    fn settle_velocity_input_is_axis_locked_before_spring_construction() {
        // Free で範囲外へ動かした結果の推定速度（両軸非ゼロ）。
        let velocity_before_axis_change = Vec2 {
            x: 300.0,
            y: -150.0,
        };
        let locked_velocity = apply_axis(velocity_before_axis_change, DragAxis::Y);
        assert_eq!(
            locked_velocity,
            Vec2 { x: 0.0, y: -150.0 },
            "軸 Y へ変更後は禁止軸（X）の初速度が 0 に絞られ、\
             許可軸（Y）はそのまま渡されるべき"
        );
    }
}

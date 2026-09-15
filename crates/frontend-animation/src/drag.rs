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

/// 直近 2 サンプル（位置・`performance.now()`/`event.time_stamp()` 相当の
/// ms タイムスタンプ）から離脱速度（px/秒）を推定する。
///
/// いずれかのサンプルが欠けている・時間差が 0 以下（同時刻の重複サンプル・
/// クロックの逆行）の場合は `Vec2::default()`（速度 0）を返す
/// （spring の初速 0 は「行き過ぎない」安全側の既定値）。
#[must_use]
pub fn estimate_velocity(previous: Option<(Vec2, f64)>, latest: Option<(Vec2, f64)>) -> Vec2 {
    let (Some((p0, t0)), Some((p1, t1))) = (previous, latest) else {
        return Vec2::default();
    };
    let dt = (t1 - t0) / 1000.0;
    if !dt.is_finite() || dt <= 0.0 {
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
#[must_use]
fn is_velocity_stale(latest: Option<(Vec2, f64)>, time_ms: f64) -> bool {
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

    /// 軸制約を更新する（現在位置・進行中のドラッグ状態は保持したまま
    /// 軸のみ切り替える。codex-review P1 是正、PR #2565 第 3 ラウンド）。
    ///
    /// keyed list の既存行更新で [`crate::drag`] 呼び出し元
    /// （`crates/wasm-full/src/drag_gesture.rs::DRAG_AXIS_ATTR`）の値が
    /// `x` ⇔ `y` へ変わっても、`controller_for` は既存 [`DragController`]
    /// をそのまま返すだけで軸を読み直さない（軸は [`Self::attach`] 時に
    /// しか読まれない）。`wasm-full` 側の再同期経路が本メソッドで最新の
    /// 属性値を反映する。
    pub fn set_axis(&mut self, axis: DragAxis) {
        self.axis = axis;
    }

    /// 現在の書き込み済み位置（テスト・呼び出し元の状態確認用）。
    #[must_use]
    pub fn position(&self) -> Vec2 {
        *self.position.borrow()
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
        if self.start.take().is_none() {
            return;
        }
        let current = self.position();
        let velocity = if is_velocity_stale(self.latest_sample, time_ms) {
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
    fn settle(&mut self, current: Vec2, velocity: Vec2) {
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
        apply_axis, clamp_to_constraint, clamp_to_constraint_for_axis, estimate_velocity,
        is_velocity_stale, normalize_constraint, DragAxis, DragConstraint,
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
}

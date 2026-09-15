//! カスタムカーソル（cursor 部品）の spring 追従演算・DOM 書き込み
//! （イシュー #2542、親 #2530/#2476）。
//!
//! Motion+ の Cursor（premium library、ポインタに spring で追従する
//! カスタムカーソル）に相当する効果のうち、「目標座標 → 追従位置」の
//! spring 演算（[`CursorFollower`]）と、その位置を DOM の CSS カスタム
//! プロパティへ書き込む rAF ループ駆動（[`CursorAnimator`]）のみを本
//! モジュールが担う。ポインタイベントの購読・opt-in 属性判定・hover
//! 対象の解決・`prefers-reduced-motion`/`pointer: coarse` 検出は
//! `fandhe-frontend-wasm-full` の `cursor` モジュールの責務であり、本
//! モジュールは持たない（3 層構成、`docs/design/
//! motion-reference-adoption-policy.md` §6）。
//!
//! # `drag.rs::settle` と同型の spring 再構築パターン
//!
//! ポインタは連続的に動く（目標が動き続ける）ため、[`Spring`] の解析解
//! （固定の `from`/`to`）をそのまま使い続けることはできない。
//! [`CursorFollower::retarget`] は「現在値・現在速度」を新しい spring の
//! 初期条件として使い、目標が更新されるたびに spring を作り直す
//! （[`crate::drag::DragController::settle`] と同じ考え方だが、settle は
//! release 時に 1 回だけ発火するのに対し、本モジュールは `pointermove`
//! のたびに発火する点が異なる）。
//!
//! # `prefers-reduced-motion: reduce` のフェイルセーフ方向
//!
//! [`CursorAnimator::move_to`] は `reduced` が `true` のとき spring を
//! 経由せず [`write_position`] で即時反映する（呼び出し元
//! `fandhe-frontend-wasm-full::cursor` が `prefers_reduced_motion()` の
//! 結果を注入する。`crate::magnetic`/`crate::drag` と同じ「配線元が
//! 検出し本モジュールへ渡す」方針）。

#[cfg(target_arch = "wasm32")]
use fandhe_animation::driver::Driver;
use fandhe_animation::spring::{Spring, SpringConfig};
#[cfg(target_arch = "wasm32")]
use std::cell::{Cell, RefCell};
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlElement;

#[cfg(target_arch = "wasm32")]
use crate::raf_driver::{AnimationLoop, RafDriver};

/// [`write_position`] が書き込む CSS カスタムプロパティ名（X 軸）。
pub const CURSOR_X_PROPERTY: &str = "--fandhe-motion-cursor-x";
/// [`write_position`] が書き込む CSS カスタムプロパティ名（Y 軸）。
pub const CURSOR_Y_PROPERTY: &str = "--fandhe-motion-cursor-y";

/// spring 追従の演算のみを担う DOM 非依存の型（native テスト対象）。
///
/// 構築直後（[`Self::new`] のみ、[`Self::retarget`] 未呼び出し）は spring
/// を持たず、[`Self::step`] は即座に `done=true` を返す（追従先が
/// まだ決まっていないため、静止状態として扱う）。
pub struct CursorFollower {
    config: SpringConfig,
    current: (f64, f64),
    velocity: (f64, f64),
    spring_x: Option<Spring>,
    spring_y: Option<Spring>,
    elapsed: f64,
}

impl CursorFollower {
    /// `initial` を現在位置・追従先として構築する（spring 未構築、
    /// 静止状態）。
    #[must_use]
    pub fn new(config: SpringConfig, initial: (f64, f64)) -> Self {
        Self {
            config,
            current: initial,
            velocity: (0.0, 0.0),
            spring_x: None,
            spring_y: None,
            elapsed: 0.0,
        }
    }

    /// 追従目標を `(x, y)` へ更新し、現在値・現在速度を初期条件として
    /// spring を作り直す（モジュール doc「`drag.rs::settle` と同型の
    /// spring 再構築パターン」節参照）。
    ///
    /// `Spring::new` が `None` を返す入力（非有限値・極端な値）では
    /// spring を構築せず、現在位置を目標へ即座にスナップする
    /// （panic しない fail-safe、`drag.rs::settle` と同じ方針）。
    pub fn retarget(&mut self, x: f64, y: f64) {
        self.elapsed = 0.0;
        let spring_x = Spring::new(self.config, self.current.0, x, self.velocity.0);
        let spring_y = Spring::new(self.config, self.current.1, y, self.velocity.1);
        match (spring_x, spring_y) {
            (Some(spring_x), Some(spring_y)) => {
                self.spring_x = Some(spring_x);
                self.spring_y = Some(spring_y);
            }
            _ => {
                self.spring_x = None;
                self.spring_y = None;
                self.current = (x, y);
                self.velocity = (0.0, 0.0);
            }
        }
    }

    /// `dt`（秒）だけ進め、`(現在位置, 両軸 done)` を返す。
    ///
    /// spring 未構築（[`Self::retarget`] 未呼び出し、または直前の
    /// `retarget` が非有限値でスナップした場合）は `dt` を消費せず
    /// `done=true` を返す。
    pub fn step(&mut self, dt: f64) -> ((f64, f64), bool) {
        let (Some(spring_x), Some(spring_y)) = (&self.spring_x, &self.spring_y) else {
            return (self.current, true);
        };
        self.elapsed += dt;
        let state_x = spring_x.at(self.elapsed);
        let state_y = spring_y.at(self.elapsed);
        self.current = (state_x.value, state_y.value);
        self.velocity = (state_x.velocity, state_y.velocity);
        (self.current, state_x.done && state_y.done)
    }

    /// 現在位置（テスト・呼び出し元での参照用途）。
    #[must_use]
    pub fn position(&self) -> (f64, f64) {
        self.current
    }
}

/// `element` の style へ `(x, y)`（CSS ピクセル）を [`CURSOR_X_PROPERTY`]/
/// [`CURSOR_Y_PROPERTY`] として書き込む（`crate::magnetic::write_offset`
/// と同型、`CSSStyleDeclaration.setProperty` の 2 引数 API のみ使用、
/// A03 対策）。
#[cfg(target_arch = "wasm32")]
pub fn write_position(element: &HtmlElement, x: f64, y: f64) {
    let style = element.style();
    let _ = style.set_property(CURSOR_X_PROPERTY, &format!("{x}px"));
    let _ = style.set_property(CURSOR_Y_PROPERTY, &format!("{y}px"));
}

/// [`CursorFollower`] を rAF ループ（[`AnimationLoop`]/[`RafDriver`]）で
/// 駆動し、`element` へ [`write_position`] し続ける薄いラッパー
/// （`fandhe-frontend-wasm-full::cursor` から 1 個だけ構築・保持される）。
#[cfg(target_arch = "wasm32")]
pub struct CursorAnimator {
    element: HtmlElement,
    follower: Rc<RefCell<CursorFollower>>,
    /// ループが現在稼働中かどうか（`AnimationLoop` 自体は停止後も
    /// `Some` のまま残るため、再起動要否の判定には別途この flag を使う）。
    running: Rc<Cell<bool>>,
    #[allow(dead_code)]
    anim_loop: Option<AnimationLoop>,
    reduced: bool,
}

#[cfg(target_arch = "wasm32")]
impl CursorAnimator {
    /// `element` を `(0.0, 0.0)` 起点として構築する。
    #[must_use]
    pub fn new(element: HtmlElement, config: SpringConfig, reduced: bool) -> Self {
        Self {
            element,
            follower: Rc::new(RefCell::new(CursorFollower::new(config, (0.0, 0.0)))),
            running: Rc::new(Cell::new(false)),
            anim_loop: None,
            reduced,
        }
    }

    /// 追従目標を `(x, y)` へ更新する。
    ///
    /// `reduced`（構築時に注入された `prefers-reduced-motion: reduce`）
    /// の場合は spring を経由せず [`write_position`] で即時反映する
    /// （モジュール doc「フェイルセーフ方向」節）。それ以外は
    /// [`CursorFollower::retarget`] で目標を更新し、ループが未稼働
    /// （初回呼び出し、または前回の追従が収束してループが自動停止した後）
    /// なら [`AnimationLoop`] を（再）起動する。`RafDriver::new()` が
    /// `None`（非ブラウザ環境）を返す場合は即時反映へフォールバックする。
    pub fn move_to(&mut self, x: f64, y: f64) {
        if self.reduced {
            write_position(&self.element, x, y);
            return;
        }
        self.follower.borrow_mut().retarget(x, y);
        if self.running.get() {
            return;
        }
        let Some(mut driver) = RafDriver::new() else {
            let pos = self.follower.borrow().position();
            write_position(&self.element, pos.0, pos.1);
            return;
        };
        let follower = Rc::clone(&self.follower);
        let element = self.element.clone();
        let running = Rc::clone(&self.running);
        running.set(true);
        self.anim_loop = Some(AnimationLoop::start(move || {
            let Some(dt) = driver.tick() else {
                return true;
            };
            let (pos, done) = follower.borrow_mut().step(dt);
            write_position(&element, pos.0, pos.1);
            if done {
                running.set(false);
            }
            !done
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::{CursorFollower, CURSOR_X_PROPERTY, CURSOR_Y_PROPERTY};
    use fandhe_animation::spring::SpringConfig;

    #[test]
    fn property_names_are_stable_strings() {
        assert_eq!(CURSOR_X_PROPERTY, "--fandhe-motion-cursor-x");
        assert_eq!(CURSOR_Y_PROPERTY, "--fandhe-motion-cursor-y");
    }

    #[test]
    fn new_follower_is_done_before_any_retarget() {
        let mut follower = CursorFollower::new(SpringConfig::default(), (5.0, 5.0));
        let (pos, done) = follower.step(0.016);
        assert_eq!(pos, (5.0, 5.0));
        assert!(done);
    }

    #[test]
    fn retarget_preserves_continuity_of_current_value() {
        let mut follower = CursorFollower::new(SpringConfig::default(), (0.0, 0.0));
        follower.retarget(100.0, 0.0);
        // 数フレーム進めて中間位置まで到達させる。
        let mut mid = (0.0, 0.0);
        for _ in 0..5 {
            (mid, _) = follower.step(0.016);
        }
        let before_retarget = follower.position();
        assert_eq!(before_retarget, mid);
        // 目標を変更しても、直前の現在値から連続する（値が飛ばない）。
        follower.retarget(200.0, 50.0);
        assert_eq!(follower.position(), before_retarget);
    }

    #[test]
    fn step_converges_to_static_target() {
        let mut follower = CursorFollower::new(SpringConfig::default(), (0.0, 0.0));
        follower.retarget(50.0, -20.0);
        let mut done = false;
        let mut pos = (0.0, 0.0);
        for _ in 0..1000 {
            (pos, done) = follower.step(0.016);
            if done {
                break;
            }
        }
        assert!(done, "1000 フレーム以内に収束するはず");
        assert!((pos.0 - 50.0).abs() < 1e-6);
        assert!((pos.1 - (-20.0)).abs() < 1e-6);
    }

    #[test]
    fn retarget_with_nan_snaps_immediately_without_panic() {
        let mut follower = CursorFollower::new(SpringConfig::default(), (0.0, 0.0));
        follower.retarget(f64::NAN, 10.0);
        let (x, y) = follower.position();
        assert!(x.is_nan());
        assert_eq!(y, 10.0);
        // spring が構築されていないため step は即座に done を返す。
        let (_, done) = follower.step(0.016);
        assert!(done);
    }
}

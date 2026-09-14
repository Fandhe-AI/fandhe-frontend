//! `requestAnimationFrame` を計測源とする [`fandhe_animation::driver::Driver`]
//! 実装（イシュー #2403/#2517）。
//!
//! `fandhe-animation` は「フレームループを持たない pull 型」設計
//! （`docs/design/animation-core-architecture.md` §2.1）のため、実際に毎
//! フレーム [`fandhe_animation::driver::Driver::tick`] を呼び続けるループ
//! 自体は本モジュールが [`AnimationLoop`] として補助的に提供する
//! （`fandhe-animation` 側の責務ではない）。

use std::cell::RefCell;
use std::rc::Rc;

use fandhe_animation::driver::Driver;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

/// `window.performance.now()` の差分（秒）を返す [`Driver`] 実装。
///
/// 初回 `tick` は基準時刻の記録のみを行い `None` を返す（`Driver::tick`
/// の契約どおり、経過時間が確定できないフレームは `None`）。
pub struct RafDriver {
    performance: web_sys::Performance,
    last_ms: Option<f64>,
}

impl RafDriver {
    /// `window`/`window.performance` が取得できるブラウザ環境でのみ
    /// `Some` を返す（非ブラウザ実行環境で panic しない、入力検証の一環）。
    pub fn new() -> Option<Self> {
        let performance = web_sys::window()?.performance()?;
        Some(Self {
            performance,
            last_ms: None,
        })
    }
}

impl Driver for RafDriver {
    fn tick(&mut self) -> Option<f64> {
        let now_ms = self.performance.now();
        let delta = self.last_ms.map(|last| (now_ms - last) / 1000.0);
        self.last_ms = Some(now_ms);
        delta
    }
}

/// `requestAnimationFrame` で `step` を毎フレーム呼び続け、`step` が
/// `false` を返したら自動停止する（または [`AnimationLoop::stop`] で
/// 明示停止する）ループ補助。
///
/// # Closure のライフタイム管理（A08 対策）
///
/// rAF コールバックは「次フレームの自分自身」を都度 `request_animation_frame`
/// で再登録する自己参照構造を持つ。`crates/wasm-client/src/timer.rs`
/// （イシュー #1121）と同じ理由——実行中の `Closure`（呼び出し元の自分
/// 自身）を `call_mut` 実行中に drop すると use-after-free になる——を
/// 避けるため、`Rc<RefCell<Option<Closure<dyn FnMut()>>>>`
/// （[`SharedClosureSlot`]）で `Closure` を共有所有し、コールバック内から
/// 自身の `Rc` を経由して次フレームを予約する（drop はコールバックの
/// 呼び出しフレームを抜けた後、`stop`/次回 `start` 呼び出し時にのみ
/// 行われる）。
pub struct AnimationLoop {
    closure_slot: SharedClosureSlot,
    request_id: Rc<RefCell<Option<i32>>>,
}

/// rAF コールバックを再入可能に共有所有するための型
/// （`clippy::type_complexity` 回避の別名、意味は [`AnimationLoop`] doc 参照）。
type SharedClosureSlot = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

impl AnimationLoop {
    /// `step`（`FnMut() -> bool`。継続するなら `true` を返す）を
    /// `requestAnimationFrame` ごとに呼ぶループを開始する。
    ///
    /// `window` が取得できない環境では何もしない no-op として振る舞う
    /// （`RafDriver::new` と同じ fail-safe 方針。呼び出し側は戻り値の
    /// `AnimationLoop` を保持するだけでよく、環境有無の分岐を書く必要が
    /// ない）。
    pub fn start(mut step: impl FnMut() -> bool + 'static) -> Self {
        let closure_slot: SharedClosureSlot = Rc::new(RefCell::new(None));
        let request_id: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

        let closure_slot_for_body = closure_slot.clone();
        let request_id_for_body = request_id.clone();
        let tick = Closure::wrap(Box::new(move || {
            if step() {
                if let Some(id) = request_animation_frame(&closure_slot_for_body) {
                    *request_id_for_body.borrow_mut() = Some(id);
                }
            }
        }) as Box<dyn FnMut()>);

        *closure_slot.borrow_mut() = Some(tick);
        if let Some(id) = request_animation_frame(&closure_slot) {
            *request_id.borrow_mut() = Some(id);
        }

        Self {
            closure_slot,
            request_id,
        }
    }

    /// 予約済みの次フレームを取り消し、`Closure` を解放する。
    ///
    /// `step` 自身の呼び出し（`call_mut`）はコールバック本体が既に完了した
    /// 後にのみ次フレームを再登録する（このメソッドを呼ぶ側が `step` の
    /// 呼び出しフレームの内側にいることはない）ため、ここでの `borrow_mut`
    /// が二重借用で panic することはない。
    pub fn stop(&self) {
        if let Some(id) = self.request_id.borrow_mut().take() {
            cancel_animation_frame(id);
        }
        self.closure_slot.borrow_mut().take();
    }
}

impl Drop for AnimationLoop {
    fn drop(&mut self) {
        self.stop();
    }
}

fn request_animation_frame(closure_slot: &SharedClosureSlot) -> Option<i32> {
    let window = web_sys::window()?;
    let closure_ref = closure_slot.borrow();
    let closure = closure_ref.as_ref()?;
    window
        .request_animation_frame(closure.as_ref().unchecked_ref())
        .ok()
}

fn cancel_animation_frame(id: i32) {
    if let Some(window) = web_sys::window() {
        let _ = window.cancel_animation_frame(id);
    }
}

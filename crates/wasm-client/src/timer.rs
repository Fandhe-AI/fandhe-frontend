//! ワンショット副作用（一度きりのタイマー）向け Closure 寿命管理
//! （イシュー #1121）。
//!
//! `registry.rs`（クリックリスナー等の継続的なハンドラ寿命管理）と同じ
//! `thread_local!` レジストリ方式を、`setTimeout`/`clearTimeout` の
//! ワンショット用途向けに提供する。イシュー #1121 報告者が自前実装を
//! 強いられていた「ワンショット副作用の Closure 管理」を公式パターン化
//! する。
//!
//! # 寿命管理の不変条件
//!
//! [`registry::replace_handles`]（`registry.rs`）が「DOM リスナー解除 →
//! `Closure` drop」の順序を守るのと同型に、本モジュールも**同一 `key` への
//! 再登録・明示的な [`clear_timeout_once`] は必ず `clearTimeout` を呼んで
//! から旧 `Closure` を drop する**。これにより「JS 側タイマーだけ残って
//! Rust 側 `Closure` が消える」孤立（drop 済み `Closure` 発火時の実行時
//! エラー、イシュー #1121 の報告内容そのもの）を構造的に防ぐ。
//!
//! `Closure::forget()`（意図的リーク）は使わない。タイマー発火後は当該
//! `key` のエントリが `TIMERS` レジストリに残留するが（有界: `key` あたり
//! 高々 1 個）、次回の同 `key` への [`set_timeout_once`] 呼び出し、または
//! 明示的な [`clear_timeout_once`] 呼び出し時に上記の順序で回収される
//! （遅延回収方式。発火直後に自己回収する設計〔実行中の `Closure` が自身を
//! drop する自己参照パターン〕は正当性の直感的な確認が難しいため、本
//! モジュールでは採用しない）。

use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

/// 登録済みワンショットタイマー 1 件分。`window.clearTimeout` に必要な
/// handle と、発火まで生存させる必要がある `Closure` の所有権をまとめて
/// 保持する。
struct TimerHandle {
    timeout_id: i32,
    closure: Closure<dyn FnMut()>,
}

thread_local! {
    /// key -> 現在登録済みのワンショットタイマー。[`set_timeout_once`]/
    /// [`clear_timeout_once`] からのみ書き込まれる。
    static TIMERS: RefCell<HashMap<String, TimerHandle>> = RefCell::new(HashMap::new());
}

/// `delay_ms` 経過後に `callback`（`FnOnce`）を一度だけ実行するタイマーを
/// 登録する。
///
/// 同一 `key` へ既存の登録がある場合は、まず [`clear_timeout_once`] で
/// 「`clearTimeout` → 旧 `Closure` drop」の順序で回収してから新しい
/// タイマーを登録する（同一 key の多重登録を防ぎ、常に高々 1 個に保つ）。
///
/// `callback` は `FnOnce` として受け取るが、`wasm-bindgen` の `Closure` は
/// `FnMut` 境界を要求するため、内部で `RefCell<Option<F>>` に包み初回
/// 呼び出し時にのみ `take()` して実行する（2 回目以降の呼び出しがあっても
/// 何もしない防御的実装。`setTimeout` は仕様上 1 回のみ発火するため通常
/// 到達しない）。
///
/// # Errors
///
/// `window` が取得できない環境（非ブラウザ実行）、または
/// `window.setTimeout` 呼び出し自体が失敗した場合に `Err` を返す。
pub fn set_timeout_once(
    key: &str,
    delay_ms: i32,
    callback: impl FnOnce() + 'static,
) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;

    // 不変条件: 再登録時も「clearTimeout → drop」の順序を守る
    // （モジュール冒頭コメント参照）。
    clear_timeout_once(key);

    let callback_cell = RefCell::new(Some(callback));
    let closure = Closure::wrap(Box::new(move || {
        if let Some(cb) = callback_cell.borrow_mut().take() {
            cb();
        }
        // 発火後もこのエントリ自体は TIMERS に残留する（次回の同一 key
        // 登録・clear 時に回収する遅延回収方式、モジュール冒頭コメント
        // 参照）。ここで自身を能動的に取り除く自己回収は行わない。
    }) as Box<dyn FnMut()>);

    let timeout_id = window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            delay_ms,
        )
        .map_err(|_| JsValue::from_str("set_timeout_with_callback failed"))?;

    TIMERS.with(|cell| {
        cell.borrow_mut().insert(
            key.to_string(),
            TimerHandle {
                timeout_id,
                closure,
            },
        );
    });

    Ok(())
}

/// `key` に対応するワンショットタイマーが登録済みなら、`clearTimeout` で
/// JS 側のタイマーを解除してから `Closure` を drop する。
///
/// 未登録の `key`（既に発火済みで [`set_timeout_once`] の再登録・本関数の
/// 呼び出しによって回収済み、または一度も登録されていない）に対しては
/// 何もしない（no-op、panic しない）。
pub fn clear_timeout_once(key: &str) {
    let removed = TIMERS.with(|cell| cell.borrow_mut().remove(key));
    if let Some(handle) = removed {
        // JS 側タイマーの解除を Closure の drop より先に行う（不変条件）。
        // 既に発火済みの timeout_id に対する clearTimeout は仕様上 no-op
        // であり、エラーにはならない（ブラウザ標準の setTimeout/clearTimeout
        // の契約）。
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(handle.timeout_id);
        }
        drop(handle.closure);
    }
}

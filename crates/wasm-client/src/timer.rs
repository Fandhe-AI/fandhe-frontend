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
//!
//! # 再入時（コールバック内からの同一 key 再登録）の安全性
//!
//! コールバック（`FnOnce`）自身が [`set_timeout_once`] を同一 `key` で
//! 呼び直す「発火後の自己リスケジュール」は妥当な利用形態だが、素朴な
//! 実装では実行中の `Closure`（＝呼び出し元の自分自身）を
//! `clear_timeout_once` 経由で即座に drop してしまい、`Closure` の
//! `call_mut` 実行途中でその裏付けメモリを解放する use-after-free になる
//! （Cursor Bugbot 指摘、イシュー #1121 PR #1131 レビュー）。本モジュールは
//! `FIRING_KEY`（現在発火中コールバックの `key`）と `TRASH`（即時 drop が
//! 危険な `TimerHandle` の退避先）の 2 つの `thread_local!` でこれを回避
//! する: 発火中の `key` に対する [`clear_timeout_once`] は該当エントリを
//! `TRASH` へ退避するのみで `Closure` を drop しない。`TRASH` は
//! [`set_timeout_once`]/[`clear_timeout_once`] の**次回呼び出し時**
//! （＝発火中コールバックの呼び出しフレームを必ず抜けた後）にまとめて
//! drop する。JS のシングルスレッド実行モデル上、あるタイマー
//! コールバックの実行中に**別の**コールバックが割り込むことはないため、
//! `FIRING_KEY` はスタックではなく単一のスカラーで足りる。

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

    /// 現在 `call_mut` 実行中（発火中）のコールバックの `key`。発火中の
    /// コールバックが自分自身と同じ `key` で [`set_timeout_once`]/
    /// [`clear_timeout_once`] を呼び直す「自己リスケジュール」時、その
    /// `key` に対応する `TimerHandle`（＝発火中の自分自身）を即座に drop
    /// しないための再入検出フラグ（モジュール冒頭コメント参照）。
    static FIRING_KEY: RefCell<Option<String>> = const { RefCell::new(None) };

    /// 発火中で即時 drop が危険なため退避された `TimerHandle` の集積先。
    /// [`set_timeout_once`]/[`clear_timeout_once`] の呼び出しの都度、
    /// 冒頭で [`drain_trash`] によりまとめて drop する（発火中コールバック
    /// の呼び出しフレームを必ず抜けた後にのみ到達するため安全）。
    static TRASH: RefCell<Vec<TimerHandle>> = const { RefCell::new(Vec::new()) };
}

/// `TRASH` に退避済みの `TimerHandle` をすべて drop する。
///
/// [`set_timeout_once`]/[`clear_timeout_once`] の冒頭で呼ぶことで、発火中
/// コールバックの呼び出しフレームを抜けた後の最初の機会に回収する（遅延
/// 回収方式、モジュール冒頭コメント参照）。`TRASH` へ積まれた時点で JS
/// 側の `clearTimeout` は既に呼び出し済みのため、ここでは `Closure` の
/// drop のみを行う。
fn drain_trash() {
    TRASH.with(|cell| cell.borrow_mut().clear());
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

    // 発火中コールバックの呼び出しフレームを抜けた後の最初の機会に限り
    // 到達する回収処理（モジュール冒頭コメント参照）。
    drain_trash();

    // 不変条件: 再登録時も「clearTimeout → drop（または再入時は
    // TRASH への退避）」の順序を守る（モジュール冒頭コメント参照）。
    clear_timeout_once(key);

    let key_for_closure = key.to_string();
    let callback_cell = RefCell::new(Some(callback));
    let closure = Closure::wrap(Box::new(move || {
        // このコールバック自身が同じ key で set_timeout_once/
        // clear_timeout_once を呼び直す「自己リスケジュール」に備え、
        // 発火中である旨を記録する（cb() 実行中のみ有効）。
        FIRING_KEY.with(|cell| *cell.borrow_mut() = Some(key_for_closure.clone()));
        if let Some(cb) = callback_cell.borrow_mut().take() {
            cb();
        }
        // cb() を抜けた時点でこの Closure（自分自身）はまだ呼び出し
        // フレーム上にあるため、ここで TRASH の回収は行わない
        // （実行中の自分自身を drop してしまう use-after-free を避ける、
        // モジュール冒頭コメント参照）。回収は次回の set_timeout_once/
        // clear_timeout_once 呼び出し時（drain_trash 経由）に行う。
        FIRING_KEY.with(|cell| *cell.borrow_mut() = None);
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
///
/// `key` が現在発火中のコールバック自身のものである場合（コールバック
/// 内から同一 key を再登録・明示 clear する自己リスケジュールの経路）は、
/// `Closure` の drop を今すぐ行わず `TRASH` へ退避する（モジュール冒頭
/// コメント参照。実行中の自分自身を drop する use-after-free を避ける）。
pub fn clear_timeout_once(key: &str) {
    drain_trash();

    let removed = TIMERS.with(|cell| cell.borrow_mut().remove(key));
    if let Some(handle) = removed {
        // JS 側タイマーの解除を Closure の drop より先に行う（不変条件）。
        // 既に発火済みの timeout_id に対する clearTimeout は仕様上 no-op
        // であり、エラーにはならない（ブラウザ標準の setTimeout/clearTimeout
        // の契約）。
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(handle.timeout_id);
        }

        let is_firing_self = FIRING_KEY.with(|cell| cell.borrow().as_deref() == Some(key));
        if is_firing_self {
            // 発火中の自分自身の Closure を今 drop すると call_mut 実行中の
            // メモリを解放する use-after-free になるため、呼び出しフレーム
            // を抜けた後の次回 set_timeout_once/clear_timeout_once 呼び出し
            // （drain_trash 経由）まで所有権だけを退避する。
            TRASH.with(|cell| cell.borrow_mut().push(handle));
        } else {
            drop(handle.closure);
        }
    }
}

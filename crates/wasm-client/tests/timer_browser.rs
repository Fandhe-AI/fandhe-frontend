//! `set_timeout_once()`/`clear_timeout_once()` の実ブラウザテスト
//! （イシュー #1121）。
//!
//! 検証する不変条件（`crates/wasm-client/src/timer.rs` rustdoc・
//! `lib.rs` クレート冒頭不変条件 8 参照）:
//! - 登録したコールバックが発火すること。
//! - 同一 key への再登録で旧タイマーが発火しないこと（`clearTimeout` →
//!   旧 `Closure` drop の順序不変条件の間接検証）。
//! - `clear_timeout_once` で明示解除したタイマーは発火しないこと。
//! - 再登録・clear 後も後続の `set_timeout_once` 呼び出しがランタイム
//!   エラー（drop 済み `Closure` 発火）を起こさず正常に完了すること。
//!
//! `setTimeout` の実発火をブラウザテストで待つため、`wasm_bindgen_test`
//! の非同期対応（`#[wasm_bindgen_test]` を `async fn` に付与）と
//! `js_sys::Promise` ベースの `sleep` ヘルパーを使う。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_wasm_client::{clear_timeout_once, set_timeout_once};
use js_sys::Promise;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// `delay_ms` だけ待つ非同期ヘルパー。テスト対象の `setTimeout` とは別の
/// `window.setTimeout` 呼び出しであり、`set_timeout_once`/`clear_timeout_once`
/// の内部状態（`TIMERS` レジストリ）には触れない。`wasm-full/tests/
/// headless_timer_browser.rs::sleep_ms` と同型のテスト専用ヘルパー
/// （`Closure::once` + `forget()` はテストヘルパー限定のイディオムであり、
/// 本体実装（`timer.rs`）の `forget()` 不使用方針とは別の話であることに
/// 注意）。
async fn sleep(delay_ms: i32) {
    let promise = Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist");
        let closure = Closure::once(move || {
            resolve.call0(&JsValue::NULL).ok();
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                delay_ms,
            )
            .expect("setTimeout must not fail in test helper");
        closure.forget();
    });
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("sleep promise must resolve");
}

/// 観点 1: 登録したコールバックが発火すること。
#[wasm_bindgen_test]
async fn set_timeout_once_fires_the_callback() {
    let fired = Rc::new(Cell::new(false));
    let fired_for_closure = fired.clone();
    set_timeout_once("timer-browser-fires", 10, move || {
        fired_for_closure.set(true);
    })
    .expect("set_timeout_once must succeed in a browser environment");

    sleep(100).await;

    assert!(fired.get(), "登録したコールバックが発火すること");
}

/// 観点 2: 同一 key への再登録で旧タイマーが発火しないこと。
#[wasm_bindgen_test]
async fn re_registering_same_key_cancels_the_previous_timer() {
    let old_fired = Rc::new(Cell::new(false));
    let old_fired_for_closure = old_fired.clone();
    set_timeout_once("timer-browser-rekey", 10, move || {
        old_fired_for_closure.set(true);
    })
    .expect("first set_timeout_once must succeed");

    let new_fired = Rc::new(Cell::new(false));
    let new_fired_for_closure = new_fired.clone();
    set_timeout_once("timer-browser-rekey", 10, move || {
        new_fired_for_closure.set(true);
    })
    .expect("second set_timeout_once on the same key must succeed and cancel the first");

    sleep(100).await;

    assert!(
        !old_fired.get(),
        "同一 key への再登録により旧タイマーは発火しないこと"
    );
    assert!(new_fired.get(), "新しいタイマーは発火すること");
}

/// 観点 3: `clear_timeout_once` で明示解除したタイマーは発火しないこと。
#[wasm_bindgen_test]
async fn clear_timeout_once_prevents_the_callback_from_firing() {
    let fired = Rc::new(Cell::new(false));
    let fired_for_closure = fired.clone();
    set_timeout_once("timer-browser-clear", 10, move || {
        fired_for_closure.set(true);
    })
    .expect("set_timeout_once must succeed");

    clear_timeout_once("timer-browser-clear");

    sleep(100).await;

    assert!(
        !fired.get(),
        "clear_timeout_once で解除したタイマーは発火しないこと"
    );
}

/// 観点 4: 存在しない key への `clear_timeout_once` は panic せず no-op で
/// あること（未登録・発火済み回収後のどちらのケースも想定する防御的契約）。
#[wasm_bindgen_test]
fn clear_timeout_once_on_unknown_key_is_a_noop() {
    clear_timeout_once("timer-browser-unknown-key-never-registered");
}

/// 観点 5: 再登録・clear 後も後続の `set_timeout_once` 呼び出しが
/// ランタイムエラー（drop 済み `Closure` 発火）を起こさず正常に完了する
/// こと（イシュー #1121 の報告内容そのものの回帰テスト）。
#[wasm_bindgen_test]
async fn set_timeout_once_after_clear_and_rekey_does_not_panic() {
    let key = "timer-browser-reuse-after-clear";

    set_timeout_once(key, 10, || {}).expect("first registration must succeed");
    clear_timeout_once(key);
    set_timeout_once(key, 10, || {}).expect("re-registration after clear must succeed");
    clear_timeout_once(key);

    let fired = Rc::new(Cell::new(false));
    let fired_for_closure = fired.clone();
    set_timeout_once(key, 10, move || {
        fired_for_closure.set(true);
    })
    .expect("final registration must succeed");

    sleep(100).await;

    assert!(
        fired.get(),
        "複数回の再登録・clear を経ても最終登録は正常に発火すること"
    );
}

/// 観点 6: 発火中のコールバック自身が同一 key で `set_timeout_once` を
/// 呼び直す「自己リスケジュール」がパニック（発火中 `Closure` の
/// use-after-free）を起こさず、再登録した 2 回目のタイマーも正常に発火
/// すること（Cursor Bugbot 指摘、イシュー #1121 PR #1131 レビューの
/// 回帰テスト。`crates/wasm-client/src/timer.rs` の `FIRING_KEY`/`TRASH`
/// 不変条件を実ブラウザで検証する）。
#[wasm_bindgen_test]
async fn rescheduling_from_within_the_firing_callback_does_not_panic() {
    let key = "timer-browser-reschedule-from-callback";
    let fire_count = Rc::new(Cell::new(0_u32));

    let fire_count_for_first = fire_count.clone();
    let key_for_reschedule = key.to_string();
    set_timeout_once(key, 10, move || {
        fire_count_for_first.set(fire_count_for_first.get() + 1);

        // 発火中のコールバック自身が同じ key で再登録する
        // （実行中の Closure を即座に drop すると use-after-free になる
        // 経路そのもの）。
        let fire_count_for_second = fire_count_for_first.clone();
        set_timeout_once(&key_for_reschedule, 10, move || {
            fire_count_for_second.set(fire_count_for_second.get() + 1);
        })
        .expect("rescheduling from within the firing callback must succeed");
    })
    .expect("first registration must succeed");

    sleep(200).await;

    assert_eq!(
        fire_count.get(),
        2,
        "自己リスケジュールされた 1 回目・2 回目のコールバックが両方発火すること"
    );
}

/// 観点 7: 発火中のコールバック自身が同一 key で `clear_timeout_once` を
/// 直接呼んでもパニック（発火中 `Closure` の use-after-free）を起こさない
/// こと（観点 6 と対になる自己 clear 経路の回帰テスト）。
#[wasm_bindgen_test]
async fn clearing_own_key_from_within_the_firing_callback_does_not_panic() {
    let key = "timer-browser-self-clear-from-callback";
    let ran = Rc::new(Cell::new(false));

    let ran_for_closure = ran.clone();
    let key_for_self_clear = key.to_string();
    set_timeout_once(key, 10, move || {
        ran_for_closure.set(true);
        // 発火中の自分自身を同じ key で明示的に clear する。
        clear_timeout_once(&key_for_self_clear);
    })
    .expect("registration must succeed");

    sleep(100).await;

    assert!(
        ran.get(),
        "自己 clear を行うコールバック自体は正常に実行を完了すること"
    );

    // 後続の別 key 登録がパニックせず正常に完了すること（TRASH 経由の
    // 遅延回収が破綻していないことの間接検証）。
    let fired = Rc::new(Cell::new(false));
    let fired_for_closure = fired.clone();
    set_timeout_once("timer-browser-after-self-clear", 10, move || {
        fired_for_closure.set(true);
    })
    .expect("registration after self-clear must succeed");

    sleep(100).await;

    assert!(
        fired.get(),
        "自己 clear の後も別 key のタイマー登録が正常に発火すること"
    );
}

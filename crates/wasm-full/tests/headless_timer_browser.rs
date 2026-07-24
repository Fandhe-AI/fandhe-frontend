//! `fandhe_frontend_wasm_full::headless_timer`（イシュー #836、親トラッキング
//! #520）の実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/tests/headless_timer.rs`（native）は hydration ラウンド
//! トリップ・判定関数 → dispatch の統合経路までを検証済みである。本ファイル
//! はその先、`wire_timer_events` 経由で配線した ActionTrigger クリックが
//! 実 DOM 上で `data-state`/`data-elapsed`/item-value テキストへ反映され、
//! `setInterval` による実 tick 駆動で start → running → pause → resume →
//! complete まで振る舞うことを検証する（実装計画 §5.2 対応）。
//!
//! # 短い interval・小さい目標値を使う理由
//!
//! 実ブラウザテストは実時間待機を伴うため、`wasm-bindgen-test` の既定
//! タイムアウト（20 秒）内に決定的に完走させる必要がある。本ファイルは
//! `interval_ms` に 30ms 前後、カウントダウン目標に 150〜300ms 程度の小さい
//! 値を使い、`wait_for`（ポーリング）で状態遷移を待つ
//! （`headless_clipboard_browser.rs`/`headless_avatar_browser.rs` と同じ
//! 「固定 sleep を避け条件ポーリングで待つ」方針）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::timer::{
    action_trigger, area, control, item, item_value, root, TimerControl, TimerPhase, TimerUnit,
};
use js_sys::Promise;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{Document, Element, MouseEvent, MouseEventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素を document body へ 1 個生成する。
fn create_container(document: &Document, id: &str) -> Element {
    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container.set_id(id);
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&container)
        .expect("append_child must not fail for a detached div");
    container
}

/// テスト末尾でコンテナを document から確実に除去する RAII ガード。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `crates/headless-ui/src/timer.rs` の SSR 出力契約そのもの（root/area/
/// item x4/control/action-trigger x4）で Timer のマークアップを組み立てて
/// `container` へ流し込む。
#[allow(clippy::too_many_arguments)]
fn mount_timer(
    container: &Element,
    countdown: bool,
    start_ms: u64,
    target_ms: u64,
    interval_ms: u64,
    elapsed_ms: u64,
    phase: TimerPhase,
) {
    let items: Vec<_> = [
        TimerUnit::Days,
        TimerUnit::Hours,
        TimerUnit::Minutes,
        TimerUnit::Seconds,
    ]
    .into_iter()
    .map(|unit| {
        item(
            unit,
            Vec::new(),
            vec![item_value(unit, Vec::new(), Vec::new())],
        )
    })
    .collect();

    let controls: Vec<_> = [
        TimerControl::Start,
        TimerControl::Pause,
        TimerControl::Resume,
        TimerControl::Reset,
    ]
    .into_iter()
    .map(|kind| action_trigger(kind, Vec::new(), Vec::new()))
    .collect();

    let node = root(
        countdown,
        start_ms,
        target_ms,
        interval_ms,
        elapsed_ms,
        phase,
        Vec::new(),
        vec![area(Vec::new(), items), control(Vec::new(), controls)],
    );
    container.set_inner_html(&render(&node));
}

fn root_element(container: &Element) -> Element {
    container
        .query_selector("[data-scope='timer'][data-part='root']")
        .expect("query_selector must not fail")
        .expect("root part must exist")
}

fn action_trigger_element(container: &Element, action: &str) -> Element {
    container
        .query_selector(&format!(
            "[data-scope='timer'][data-part='action-trigger'][data-action='{action}']"
        ))
        .expect("query_selector must not fail")
        .expect("action-trigger part must exist")
}

/// 合成 `click`（bubbles: true、通常のユーザークリックを模す）を生成する。
fn synthetic_click() -> MouseEvent {
    let init = MouseEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    MouseEvent::new_with_mouse_event_init_dict("click", &init)
        .expect("MouseEvent construction must not fail")
}

/// `condition` が真になるまでポーリングする
/// （`headless_clipboard_browser.rs::wait_for` と同型）。
async fn wait_for(mut condition: impl FnMut() -> bool) {
    for _ in 0..500 {
        if condition() {
            return;
        }
        let promise = Promise::new(&mut |resolve, _reject| {
            let window = web_sys::window().expect("window must exist");
            let closure = Closure::once(move || {
                resolve.call0(&JsValue::NULL).ok();
            });
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    10,
                )
                .expect("setTimeout must not fail");
            closure.forget();
        });
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .expect("timeout promise must resolve");
    }
}

// --- 検証: start → running → pause → resume → complete ------------------

#[wasm_bindgen_test]
async fn start_pause_resume_and_complete_lifecycle_reflects_in_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "timer-lifecycle-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // countdown 300ms・interval 30ms: 既定タイムアウト（20 秒）内に十分
    // 決定的に完走する小さい値。
    mount_timer(&container, true, 300, 0, 30, 0, TimerPhase::Idle);

    fandhe_frontend_wasm_full::headless_timer::wire_timer_events(
        container.clone(),
        |_action_ref| {},
    )
    .expect("wire_timer_events must not fail");

    // start クリック → running へ遷移。
    action_trigger_element(&container, "start")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for(|| {
        root_element(&container)
            .get_attribute("data-state")
            .as_deref()
            == Some("running")
    })
    .await;

    // pause クリック → paused へ遷移し、以後 elapsed が増えないこと。
    action_trigger_element(&container, "pause")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for(|| {
        root_element(&container)
            .get_attribute("data-state")
            .as_deref()
            == Some("paused")
    })
    .await;
    let elapsed_at_pause: u64 = root_element(&container)
        .get_attribute("data-elapsed")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    // paused 中に interval が解除されていること（elapsed が変化しないまま
    // 数 tick 分の猶予を置く）を確認する。
    wait_for(|| false).await;
    let elapsed_still_paused: u64 = root_element(&container)
        .get_attribute("data-elapsed")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    assert_eq!(elapsed_at_pause, elapsed_still_paused);

    // resume クリック → running へ復帰し、tick が再開すること。
    action_trigger_element(&container, "resume")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for(|| {
        root_element(&container)
            .get_attribute("data-state")
            .as_deref()
            == Some("running")
    })
    .await;

    // 完了（completed）まで待つ。
    wait_for(|| {
        root_element(&container)
            .get_attribute("data-state")
            .as_deref()
            == Some("completed")
    })
    .await;

    let root_el = root_element(&container);
    assert_eq!(
        root_el.get_attribute("data-elapsed").as_deref(),
        Some("300")
    );

    // 完了後は item-value（seconds）が "00"（残り 0ms）を表示すること。
    let seconds_value = container
        .query_selector("[data-scope='timer'][data-part='item-value'][data-type='seconds']")
        .expect("query_selector must not fail")
        .expect("seconds item-value must exist");
    assert_eq!(seconds_value.text_content().as_deref(), Some("00"));

    // 完了後、interval が解除され elapsed がこれ以上増えないこと。
    wait_for(|| false).await;
    assert_eq!(
        root_element(&container)
            .get_attribute("data-elapsed")
            .as_deref(),
        Some("300")
    );
}

// --- 検証: reset クリックで idle へ戻り elapsed がゼロへ戻ること -----------

#[wasm_bindgen_test]
async fn reset_click_returns_to_idle_and_stops_ticking() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "timer-reset-root");
    let _cleanup = RemoveOnDrop(container.clone());

    mount_timer(&container, false, 0, 0, 30, 0, TimerPhase::Idle);

    fandhe_frontend_wasm_full::headless_timer::wire_timer_events(
        container.clone(),
        |_action_ref| {},
    )
    .expect("wire_timer_events must not fail");

    action_trigger_element(&container, "start")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for(|| {
        root_element(&container)
            .get_attribute("data-state")
            .as_deref()
            == Some("running")
    })
    .await;
    // 少なくとも 1 tick 経過するのを待つ。
    wait_for(|| {
        root_element(&container)
            .get_attribute("data-elapsed")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0)
            > 0
    })
    .await;

    action_trigger_element(&container, "reset")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for(|| {
        root_element(&container)
            .get_attribute("data-state")
            .as_deref()
            == Some("idle")
    })
    .await;
    assert_eq!(
        root_element(&container)
            .get_attribute("data-elapsed")
            .as_deref(),
        Some("0")
    );

    // reset 後、interval が解除され elapsed が増え続けないこと。
    wait_for(|| false).await;
    assert_eq!(
        root_element(&container)
            .get_attribute("data-elapsed")
            .as_deref(),
        Some("0")
    );
}

// --- 検証: hydration 相当（初期状態が既に running）で即座に tick 予約される ---

#[wasm_bindgen_test]
async fn already_running_root_at_wire_time_resumes_ticking_immediately() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "timer-hydrate-running-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // ハイドレーション直後に既に running な Timer が存在する状況を模す。
    mount_timer(&container, false, 0, 0, 30, 0, TimerPhase::Running);

    fandhe_frontend_wasm_full::headless_timer::wire_timer_events(
        container.clone(),
        |_action_ref| {},
    )
    .expect("wire_timer_events must not fail");

    wait_for(|| {
        root_element(&container)
            .get_attribute("data-elapsed")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0)
            > 0
    })
    .await;
}

// --- 検証: XSS 回帰（攻撃者制御の children を持つマークアップ） -----------

#[wasm_bindgen_test]
fn mounting_markup_with_attacker_controlled_label_creates_no_script_element() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "timer-xss-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let payload = "\"><script>window.__timer_xss = true;</script>";
    let node = root(
        false,
        0,
        0,
        1000,
        0,
        TimerPhase::Idle,
        Vec::new(),
        vec![area(
            Vec::new(),
            vec![item(
                TimerUnit::Seconds,
                Vec::new(),
                vec![item_value(
                    TimerUnit::Seconds,
                    Vec::new(),
                    vec![fandhe_frontend_core::text(payload)],
                )],
            )],
        )],
    );
    container.set_inner_html(&render(&node));

    assert!(container.query_selector("script").ok().flatten().is_none());
    assert!(
        js_sys::Reflect::get(&JsValue::from(window), &JsValue::from_str("__timer_xss"))
            .map(|v| v.is_undefined())
            .unwrap_or(true)
    );
}

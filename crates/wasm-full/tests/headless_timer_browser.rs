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
//!
//! # `wait_for(|| false)` を短い猶予待機へ置き換えた理由（イシュー #886）
//!
//! 「interval 解除後に elapsed が変化しない」ことの確認には、恒偽条件で
//! `wait_for` を呼び出し 500 回（名目 5 秒）のポーリングを全消費させる実装が
//! 3 箇所あった。`start_pause_resume_and_complete_lifecycle_reflects_in_dom`
//! はこれを 2 回含み名目だけで約 10.5 秒となり、`wasm-bindgen-test` の既定
//! タイムアウト（20 秒/テスト）に対する余裕が乏しかった。headless Chrome は
//! 負荷下で `setTimeout` 解決が遅延しやすく、余裕の乏しさがタイムアウト
//! 超過 → chromedriver 強制終了（SIGKILL）に直結していた（`nav_browser.rs`
//! の PR #420 事例と同型の症状経路）。加えてファイル全体の実行時間が長い
//! ほど Chrome インスタンスの生存時間が延び、共有 runner のメモリ圧による
//! OOM 起因 SIGKILL への曝露も増えていた。
//!
//! 是正として `sleep_ms` ヘルパー（`setTimeout` 1 回のみの単純な待機）を
//! 導入し、`interval_ms`（30ms）の数個分（150ms）の猶予待機へ置き換えた。
//! 検証能力は変わらない: これらの assert は「`setInterval` が解除されて
//! おり elapsed が増えないこと」の確認が目的である。仮に interval が解除
//! されていなければ、次の tick は `interval_ms`（30ms）以内にタイマー
//! キューへ積まれる。タイマーキューは登録順に処理されるため、`sleep_ms
//! (150)` の解決は pending の 30ms interval コールバックより**必ず後**に
//! 実行される。したがって 150ms の猶予は誤検出なく違反 tick を観測でき、
//! 5 秒の恒偽ポーリングは検証能力を一切追加していなかった。
//!
//! # `Runtime::wire_timer` の dispatch 後再描画接続（イシュー #1959）
//!
//! 上記の検証はいずれも `headless_timer::wire_timer_events` を直接
//! `container`（プレーンな div）へ配線しており、`fandhe_frontend_wasm_full::Runtime`
//! を経由しない。`runtime_dirty_rerender` モジュールはその先、
//! `Runtime::hydrate` 経由で配線した場合に、`C` 側の束縛点
//! （`data-bind-text`）が dispatch 後に実際に再描画されること、かつ
//! Timer 自身が書く `data-state` 属性が 1 クリックにつき 1 回のみ書かれる
//! （`Runtime::wire_timer` の束縛点更新経路と二重に書かれない）ことを検証
//! する（click 経由）。
//!
//! # tick 経由の束縛点更新（イシュー #1960）
//!
//! #1959 のケースは start/pause クリックのみを対象にしており、
//! `interval_ms` を意図的に大きく（60,000ms）取って tick を事実上封じて
//! いた。`runtime_dirty_rerender::runtime_hydrate_tick_rerenders_elapsed_binding_and_writes_data_elapsed_once`
//! はその先、`setInterval` → `handle_tick` →
//! `"timer:tick"` 通知 → `Runtime::wire_timer` → `C` 側の
//! `apply_update_for_dirty` という tick 駆動の経路を実 DOM で固定する。
//!
//! 検証内容: (1) tick ごとに `C` 側の束縛点（`elapsed_label`）が実際に
//! 再描画されること、(2) `write_timer` の `data-elapsed` 直書きと
//! `apply_update_for_dirty` の束縛点更新が整合していること（同じ値が
//! 1 回だけ書かれ、二重描画にならないこと）。件数の固定式は
//! `data-elapsed` への `MutationRecord` 件数 == `tick_count + 2`
//! （start クリック 1 回 + tick ごと 1 回 + pause クリック 1 回。
//! `write_timer` 以外に `data-elapsed` を書く経路が無いことの根拠）。
//!
//! **`runtime.component()` 借用に関する禁止事項**: `Runtime::wire_timer`
//! のクロージャは `component.try_borrow_mut()` 失敗時に早期 return する
//! （dispatch を静かに捨てる）。テスト側が `runtime.component()` の
//! `Ref`（不変借用）を interval 稼働中に保持したまま `.await` をまたぐと、
//! その間の tick は `write_timer`（`data-elapsed` 直書き）だけが実行され
//! `C` 側 `tick_count` が増えず、上記の件数式が 1 ずれて原因不明の失敗に
//! なる。`runtime.component()` は pause 完了・猶予待機後の静止状態でのみ
//! 呼び、`Ref` は同一式内で即 drop する。
//!
//! # `panic=abort` な wasm32 テストビルドで `RemoveOnDrop` が保証されない
//!
//! `wasm32-unknown-unknown` の `wasm-bindgen-test` ビルドは
//! `panic=abort` プロファイルであり、`assert!`/`assert_eq!` 失敗はスタック
//! 巻き戻し（unwind）を経ずに即座に `unreachable`（トラップ）へ収束する。
//! このため、ある `#[wasm_bindgen_test]` が assert 失敗した場合、そのテスト
//! 関数内の `RemoveOnDrop` 等の `Drop` 実装は実行されない（後始末が
//! 保証されない）。`runtime_dirty_rerender::mount_host_and_hydrate`
//! が各テストに専用の `root_id` を要求するのはこのため: 固定 id・
//! セレクタ頼りの「直近描画した要素を探す」設計だと、先行テストが
//! パニックして取り残した要素と自テストが挿入した要素の区別が
//! つかなくなりうる。
//!
//! # `wait_for` が一度も `.await` せず返る場合の microtask フラッシュ
//! （イシュー #1960）
//!
//! `wait_for` は `condition()` を最初に同期的に評価し、既に真であれば
//! `Promise`/`setTimeout` を一切経由せず即座に返る。`write_timer` は
//! クリックハンドラ内で `data-state`/`data-elapsed` を同期的に書き換える
//! ため、`dispatch_event` 直後の `wait_for` はこの「一度も `.await` しない」
//! 経路を取りやすい。一方 `MutationObserver` の通知は microtask で配送
//! されるため、`.await` が一度も発生しないと通知がまだキューに残った
//! ままテストの次のコードへ進んでしまい、`MutationRecord` の件数を
//! 数えるアサートが実際にはまだ配送されていない記録を見て失敗しうる
//! （`records.borrow().len()` が実際の変化回数より少なく観測される）。
//! `runtime_hydrate_click_rerenders_host_binding_and_writes_data_state_once`
//! はこの理由で各クリック後の `wait_for(...).await` に続けて
//! `sleep_ms(0)`（`setTimeout(0)` という macrotask）を挟む。HTML/JS の
//! イベントループ仕様上、macrotask の実行前に pending の microtask
//! （`MutationObserver` 通知を含む）は必ず全てフラッシュされるため、
//! `sleep_ms(0)` の解決後は該当クリックの `MutationRecord` 配送が
//! 完了していることが保証される。

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

    // イシュー #1632: 5 action すべてを描画する（`hidden` は phase から
    // 導出されるため、フィクスチャは phase 引数をそのまま action_trigger へ
    // 伝える）。
    let controls: Vec<_> = [
        TimerControl::Start,
        TimerControl::Pause,
        TimerControl::Resume,
        TimerControl::Reset,
        TimerControl::Restart,
    ]
    .into_iter()
    .map(|kind| action_trigger(kind, phase, Vec::new(), Vec::new()))
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
///
/// 500 回（名目 5 秒）のポーリング枠を使い切っても `condition` が真になら
/// ない場合は、待機対象を示す `desc` を添えて `panic!` する（イシュー
/// #886: 無言で return して後続 assert の不可解な失敗や次の待機への
/// 突入を招く旧実装を、診断可能な即時失敗へ変換する。
/// `nav_browser.rs` の fail-fast 化と同方針）。
async fn wait_for(desc: &str, mut condition: impl FnMut() -> bool) {
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
    panic!("wait_for timed out after 500 polls (~5s): {desc}");
}

/// `setTimeout(ms)` を 1 回だけ発行し解決を待つ、条件ポーリングを伴わない
/// 単純な猶予待機ヘルパー（イシュー #886）。「interval 解除後に状態が
/// 変化しないこと」の確認用に、タイマーキューが `ms` 分の猶予を処理し
/// 終えるまで待つ目的でのみ使う（`wait_for(|| false)` の置き換え先。
/// ファイル冒頭 `//!` の根拠コメント参照）。
async fn sleep_ms(ms: i32) {
    let promise = Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist");
        let closure = Closure::once(move || {
            resolve.call0(&JsValue::NULL).ok();
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                ms,
            )
            .expect("setTimeout must not fail");
        closure.forget();
    });
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("timeout promise must resolve");
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

    // `wire_timer_events` の `root` 引数は headless_timer.rs の
    // `read_timer`/`write_timer`/`sync_interval` が `data-state`/
    // `data-elapsed` を直接読み書きする対象そのもの（timer::root() が
    // 出力する `[data-scope='timer'][data-part='root']` 要素）である必要が
    // ある。`container`（テスト用の外側ラッパー div）を渡すと、これらの
    // 属性はラッパー自身に書き込まれ、`root_element(&container)` が読む
    // 内側の実 root 要素には反映されず、状態遷移が永久に観測できなくなる
    // （イシュー #886 で判明した pre-existing のテスト側配線ミス。旧
    // `wait_for` の無言 return + SIGKILL によるタイムアウト超過がこれまで
    // 顕在化を隠していた）。
    fandhe_frontend_wasm_full::headless_timer::wire_timer_events(
        root_element(&container),
        |_action_ref| {},
    )
    .expect("wire_timer_events must not fail");

    // start クリック → running へ遷移。
    action_trigger_element(&container, "start")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for("data-state to become running after start click", || {
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
    wait_for("data-state to become paused after pause click", || {
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
    // interval_ms（30ms）数個分の猶予を置く。ファイル冒頭 `//!` の根拠
    // コメント参照）を確認する。
    sleep_ms(150).await;
    let elapsed_still_paused: u64 = root_element(&container)
        .get_attribute("data-elapsed")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    assert_eq!(elapsed_at_pause, elapsed_still_paused);

    // resume クリック → running へ復帰し、tick が再開すること。
    action_trigger_element(&container, "resume")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for("data-state to become running after resume click", || {
        root_element(&container)
            .get_attribute("data-state")
            .as_deref()
            == Some("running")
    })
    .await;

    // 完了（completed）まで待つ。
    wait_for("data-state to become completed", || {
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
    sleep_ms(150).await;
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

    // `wire_timer_events` の `root` 引数は headless_timer.rs の
    // `read_timer`/`write_timer`/`sync_interval` が `data-state`/
    // `data-elapsed` を直接読み書きする対象そのもの（timer::root() が
    // 出力する `[data-scope='timer'][data-part='root']` 要素）である必要が
    // ある。`container`（テスト用の外側ラッパー div）を渡すと、これらの
    // 属性はラッパー自身に書き込まれ、`root_element(&container)` が読む
    // 内側の実 root 要素には反映されず、状態遷移が永久に観測できなくなる
    // （イシュー #886 で判明した pre-existing のテスト側配線ミス。旧
    // `wait_for` の無言 return + SIGKILL によるタイムアウト超過がこれまで
    // 顕在化を隠していた）。
    fandhe_frontend_wasm_full::headless_timer::wire_timer_events(
        root_element(&container),
        |_action_ref| {},
    )
    .expect("wire_timer_events must not fail");

    action_trigger_element(&container, "start")
        .dispatch_event(&synthetic_click())
        .expect("dispatch_event must not fail");
    wait_for("data-state to become running after start click", || {
        root_element(&container)
            .get_attribute("data-state")
            .as_deref()
            == Some("running")
    })
    .await;
    // 少なくとも 1 tick 経過するのを待つ。
    wait_for("data-elapsed to advance past zero", || {
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
    wait_for("data-state to become idle after reset click", || {
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
    sleep_ms(150).await;
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

    // `wire_timer_events` の `root` 引数は headless_timer.rs の
    // `read_timer`/`write_timer`/`sync_interval` が `data-state`/
    // `data-elapsed` を直接読み書きする対象そのもの（timer::root() が
    // 出力する `[data-scope='timer'][data-part='root']` 要素）である必要が
    // ある。`container`（テスト用の外側ラッパー div）を渡すと、これらの
    // 属性はラッパー自身に書き込まれ、`root_element(&container)` が読む
    // 内側の実 root 要素には反映されず、状態遷移が永久に観測できなくなる
    // （イシュー #886 で判明した pre-existing のテスト側配線ミス。旧
    // `wait_for` の無言 return + SIGKILL によるタイムアウト超過がこれまで
    // 顕在化を隠していた）。
    fandhe_frontend_wasm_full::headless_timer::wire_timer_events(
        root_element(&container),
        |_action_ref| {},
    )
    .expect("wire_timer_events must not fail");

    wait_for("data-elapsed to advance past zero", || {
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

// --- 検証: Runtime::hydrate 経由の再描画接続（イシュー #1959・#1960） -----

mod runtime_dirty_rerender {
    use super::{synthetic_click, wait_for, RemoveOnDrop};
    use fandhe_frontend_core::{bind_text, render, Node};
    use fandhe_frontend_headless_ui::timer::{
        action_trigger, control, Timer, TimerAction, TimerControl,
    };
    use fandhe_frontend_interactive::{Component, DirtyTracked, Hydrate, HydrateError};
    use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
    use fandhe_frontend_wasm_full::Runtime;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;
    use web_sys::{Document, Element, MutationObserver, MutationObserverInit, MutationRecord};

    /// `Runtime::wire_timer`（イシュー #1959）・tick 経由の束縛点更新
    /// （イシュー #1960）の dispatch 後再描画接続を実 DOM で固定するための
    /// 最小ホスト。`Timer` をラップし、アプリ側の派生フィールド
    /// （フェーズラベル・経過時間ラベル）を dirty へ積む
    /// （`crates/wasm-full/tests/headless_timer.rs::wire_timer_dirty_contract::TimerHost`
    /// と同じ設計。あちらは native で `dirty_fields()` の内容自体を固定し、
    /// 本ホストは実 DOM でのハイドレーション・束縛点反映・二重描画回避を
    /// 検証する点が異なる）。
    struct TimerHost {
        timer: Timer,
        phase_label: String,
        elapsed_label: String,
        /// `"timer:tick"` の dispatch 回数（native 側
        /// `headless_timer.rs::wire_timer_dirty_contract::TimerHost` と
        /// 同名フィールド）。tick 経由で `C` が実際に更新を受け取った回数の
        /// 直接証拠として `runtime_hydrate_tick_rerenders_elapsed_binding_and_writes_data_elapsed_once`
        /// が `data-elapsed` への書き込み回数（`tick_count + 2`）の固定に使う。
        tick_count: u32,
        dirty: Vec<&'static str>,
        /// `view()` の root 要素へ焼き込む id（`mount_host_and_hydrate`
        /// 参照）。`insert_adjacent_html` 後に `:not([id])` を頼りに直近
        /// 描画した root を探す設計は、`panic=abort` な wasm32 テスト
        /// ビルドで先行テストが assert 失敗した場合に危険になる:
        /// パニックが即座に `unreachable` へ収束し `RemoveOnDrop` の
        /// 後始末（要素の除去）が実行されないため、id 未設定の root 要素が
        /// DOM に取り残されたまま次のテストが `insert_adjacent_html` した
        /// 直後に同じセレクタで `query_selector` すると、取り残された
        /// 要素と自テストが挿入した要素のどちらを掴むか不定になる
        /// （ファイル冒頭 `//!` の「`RemoveOnDrop` の後始末も保証されない」
        /// 節参照）。id をレンダリング時点で HTML 文字列へ直接焼き込み、
        /// 各テストが専用の `root_id` を使うことで、この危険を構造的に
        /// 排除する。
        root_id: String,
    }

    impl TimerHost {
        fn new(timer: Timer) -> Self {
            let phase_label = timer.phase().as_str().to_string();
            let elapsed_label = timer.elapsed_ms().to_string();
            Self {
                timer,
                phase_label,
                elapsed_label,
                tick_count: 0,
                dirty: Vec::new(),
                root_id: String::new(),
            }
        }

        /// `view()` が出力する root 要素の id を設定する
        /// （`mount_host_and_hydrate` 専用。`Hydrate::from_hydration_attrs`
        /// 経由で復元される側の `TimerHost` は id 未設定のままで良い:
        /// 復元成功時、その `view()` は `Runtime::hydrate` 内部の
        /// keyed-list キャッシュ種付けや、以後 `apply_update_for_dirty` の
        /// 構造フォールバック（子ノード再構築）で使われるのみで、実 DOM の
        /// root 要素**自体**（その id 属性を含む）は差し替えられないため）。
        fn with_root_id(mut self, root_id: &str) -> Self {
            self.root_id = root_id.to_string();
            self
        }
    }

    impl Component for TimerHost {
        type Action = <Timer as Component>::Action;

        fn update(&mut self, action: Self::Action) {
            self.dirty.clear();
            // `tick_count` は「実際に Tick アクションが dispatch された
            // 回数」であって「elapsed_label が変化した回数」ではない
            // （`crates/wasm-full/tests/headless_timer.rs::wire_timer_dirty_contract::TimerHost`
            // と同じ判定順序: action 種別を `self.timer.update` 実行前に
            // 判定してから委譲する）。同一ミリ秒内に 2 tick 目が来て
            // delta（elapsed の変化量）が 0 になるケースでも
            // `write_timer` は `data-elapsed` を毎 tick 無条件に
            // 書き込む（`set_dom_attribute` に値の同値スキップは無い）ため、
            // elapsed_label の変化有無で数えると
            // `data-elapsed` の MutationRecord 数と `tick_count` の対応式
            // （`tick_count + 2`）が崩れる。
            let is_tick = matches!(action, TimerAction::Tick(_));
            self.timer.update(action);
            if is_tick {
                self.tick_count += 1;
            }

            let new_phase_label = self.timer.phase().as_str().to_string();
            if new_phase_label != self.phase_label {
                self.phase_label = new_phase_label;
                self.dirty.push("phase_label");
            }

            let new_elapsed_label = self.timer.elapsed_ms().to_string();
            if new_elapsed_label != self.elapsed_label {
                self.elapsed_label = new_elapsed_label;
                self.dirty.push("elapsed_label");
            }
        }

        fn view(&self) -> Node {
            self.timer.root(
                vec![("id", self.root_id.as_str())],
                vec![
                    control(
                        Vec::new(),
                        vec![
                            action_trigger(
                                TimerControl::Start,
                                self.timer.phase(),
                                Vec::new(),
                                Vec::new(),
                            ),
                            action_trigger(
                                TimerControl::Pause,
                                self.timer.phase(),
                                Vec::new(),
                                Vec::new(),
                            ),
                        ],
                    ),
                    bind_text(
                        "span",
                        vec![("data-testid", "phase-label")],
                        "phase_label",
                        self.phase_label.clone(),
                    ),
                    bind_text(
                        "span",
                        vec![("data-testid", "elapsed-label")],
                        "elapsed_label",
                        self.elapsed_label.clone(),
                    ),
                ],
            )
        }

        fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
            Timer::decode_action(name, payload)
        }
    }

    impl DirtyTracked for TimerHost {
        fn dirty_fields(&self) -> &[&'static str] {
            &self.dirty
        }
    }

    impl BindingSource for TimerHost {
        fn bound_value(&self, field: &str) -> Option<BoundValue> {
            match field {
                "phase_label" => Some(BoundValue::Text(self.phase_label.clone())),
                "elapsed_label" => Some(BoundValue::Text(self.elapsed_label.clone())),
                _ => None,
            }
        }
    }

    impl Hydrate for TimerHost {
        fn hydration_attrs(&self) -> Vec<(String, String)> {
            self.timer.hydration_attrs()
        }

        fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
            Timer::from_hydration_attrs(attrs).map(Self::new)
        }
    }

    /// `TimerHost::view()` を `document.body` 直下へ描画し、`Runtime::hydrate`
    /// する共通ヘルパー（各テストが専用の `root_id` を渡すことで、本
    /// ファイル内の複数テストが本ホストを使っても id 衝突しない）。
    /// `id` は `TimerHost::with_root_id` で描画される HTML 文字列へ直接
    /// 焼き込む（`insert_adjacent_html` 後に `:not([id])` を頼りに直近
    /// 描画した root を探す設計は、`panic=abort` 下で先行テストが後始末
    /// されないまま残す可能性があり危険。`TimerHost::root_id` の doc
    /// コメント・ファイル冒頭 `//!` の該当節参照）。
    /// `render_for_hydration` が行う「view() の root へ hydration_attrs を
    /// 後付けする」処理を、実 DOM 属性として直接再現する
    /// （`runtime_browser.rs::hydrate_restores_state_from_existing_dom_and_wires_events`
    /// と同じ手順）。戻り値の `Element` は呼び出し側が `RemoveOnDrop` で
    /// 後始末する（本ヘルパーは後始末を行わない）。
    fn mount_host_and_hydrate(
        document: &Document,
        root_id: &str,
        timer: Timer,
    ) -> (Element, Runtime<TimerHost>) {
        let host = TimerHost::new(timer).with_root_id(root_id);
        let html = render(&host.view());
        document
            .body()
            .expect("document body must exist in browser test environment")
            .insert_adjacent_html("beforeend", &html)
            .expect("insert_adjacent_html must not fail");
        let root_el: Element = document
            .get_element_by_id(root_id)
            .expect("rendered Timer root must have the expected id");

        for (name, value) in host.hydration_attrs() {
            root_el
                .set_attribute(&name, &value)
                .expect("set_attribute must not fail");
        }

        let runtime = Runtime::hydrate(root_id, TimerHost::new(timer))
            .expect("hydrate must succeed for well-formed attrs");
        assert_eq!(
            runtime.root().id(),
            root_id,
            "hydrate は root_id 要素自身を Timer root として復元すること"
        );

        (root_el, runtime)
    }

    /// `root` へ `MutationObserver`（`attributes: true`,
    /// `attribute_filter: [attr_name]`）を張り、`attr_name` の変更を記録
    /// する（`crates/wasm-full/tests/keyed_update_browser.rs` と同じ
    /// パターン。イシュー #1960 で `observe_data_state` から対象属性を
    /// 引数化して一般化した）。呼び出し側が `observer.disconnect()` を
    /// 制御できるよう `MutationObserver` 自身も返す。
    fn observe_attribute(
        root: &web_sys::Element,
        attr_name: &str,
    ) -> (
        MutationObserver,
        std::rc::Rc<std::cell::RefCell<Vec<MutationRecord>>>,
    ) {
        let records = std::rc::Rc::new(std::cell::RefCell::new(Vec::<MutationRecord>::new()));
        let records_clone = records.clone();
        let callback = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::new(
            move |entries: js_sys::Array, _observer: MutationObserver| {
                for entry in entries.iter() {
                    if let Ok(record) = entry.dyn_into::<MutationRecord>() {
                        records_clone.borrow_mut().push(record);
                    }
                }
            },
        );
        let observer = MutationObserver::new(callback.as_ref().unchecked_ref())
            .expect("MutationObserver::new must not fail");
        // `callback` は observer が生存する限り呼ばれ続けるため forget する
        // （`keyed_update_browser.rs` と同じ理由：テスト末尾で
        // `observer.disconnect()` を呼ぶため無期限リークにはならない）。
        callback.forget();
        let init = MutationObserverInit::new();
        init.set_attributes(true);
        let filter = js_sys::Array::new();
        filter.push(&wasm_bindgen::JsValue::from_str(attr_name));
        init.set_attribute_filter(&filter);
        observer
            .observe_with_options(root, &init)
            .expect("observe_with_options must not fail");
        (observer, records)
    }

    /// `root` へ `MutationObserver`（`childList: true`, `subtree: false`）
    /// を張り、`root` 直下の子ノード追加・削除を記録する（イシュー
    /// #1960）。`apply_update_for_dirty` の構造フォールバック（dirty field
    /// が束縛点・keyed list のどちらにも解決できない場合の `root` 全子
    /// ノード再構築）が tick ごとに走っていないことを 0 件で固定する
    /// 用途専用。
    fn observe_child_list(
        root: &web_sys::Element,
    ) -> (
        MutationObserver,
        std::rc::Rc<std::cell::RefCell<Vec<MutationRecord>>>,
    ) {
        let records = std::rc::Rc::new(std::cell::RefCell::new(Vec::<MutationRecord>::new()));
        let records_clone = records.clone();
        let callback = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::new(
            move |entries: js_sys::Array, _observer: MutationObserver| {
                for entry in entries.iter() {
                    if let Ok(record) = entry.dyn_into::<MutationRecord>() {
                        records_clone.borrow_mut().push(record);
                    }
                }
            },
        );
        let observer = MutationObserver::new(callback.as_ref().unchecked_ref())
            .expect("MutationObserver::new must not fail");
        callback.forget();
        let init = MutationObserverInit::new();
        init.set_child_list(true);
        init.set_subtree(false);
        observer
            .observe_with_options(root, &init)
            .expect("observe_with_options must not fail");
        (observer, records)
    }

    /// `Runtime::hydrate` 経由で配線した Start/Pause クリックが、Timer 自身
    /// の `data-state` 直書きと `C`（`TimerHost`）側の束縛点
    /// （`data-bind-text="phase_label"`）の双方へ反映され、かつ
    /// `data-state` 属性への書き込みが 1 クリックにつき 1 回のみ
    /// （`Runtime::wire_timer` の `apply_update_for_dirty` が Timer の
    /// `data-state` 自体を再書きしない）ことを検証する（受け入れ条件、
    /// イシュー #1959）。
    #[wasm_bindgen_test]
    async fn runtime_hydrate_click_rerenders_host_binding_and_writes_data_state_once() {
        let window = web_sys::window().expect("window must exist");
        let document = window.document().expect("document must exist");

        // `interval_ms` を 60,000ms（1 分）と大きく取り、既定タイムアウト
        // （20 秒/テスト）内で tick が発生しないことを保証する（本テストの
        // 検証対象は click 経由の再描画接続のみであり、tick 経由の検証は
        // `runtime_hydrate_tick_rerenders_elapsed_binding_and_writes_data_elapsed_once`
        // に委ねる）。
        let (root_el, runtime) = mount_host_and_hydrate(
            &document,
            "timer-host-runtime-dirty-rerender-click-root",
            Timer::count_up(0, 60_000),
        );
        let _cleanup = RemoveOnDrop(root_el.clone());

        let (observer, records) = observe_attribute(&root_el, "data-state");

        let phase_label = || {
            root_el
                .query_selector("[data-bind-text='phase_label']")
                .expect("query_selector must not fail")
                .expect("phase_label binding point must exist")
                .text_content()
                .unwrap_or_default()
        };
        assert_eq!(phase_label(), "idle");

        let start_button = root_el
            .query_selector("[data-scope='timer'][data-part='action-trigger'][data-action='start']")
            .expect("query_selector must not fail")
            .expect("start action-trigger must exist");
        start_button
            .dispatch_event(&synthetic_click())
            .expect("dispatch_event must not fail");

        wait_for("data-state becomes running after start click", || {
            root_el.get_attribute("data-state").as_deref() == Some("running")
        })
        .await;
        // `write_timer` はクリックハンドラ内で同期的に `data-state` を
        // 書き換えるため、`wait_for` は最初の同期チェックで真になり
        // 一度も `.await` を経ずに返ることがある（`condition()` が最初の
        // 呼び出しで真の場合、内部の `Promise`/`setTimeout` を一切
        // 経由しない）。その場合 JS の microtask キュー（`MutationObserver`
        // の通知は microtask で配送される）が一度もフラッシュされないまま
        // 次の同期コードへ進んでしまい、`records.borrow().len()` が
        // まだ 0 のまま観測される（`data-state` は実際には変化済みなのに
        // 記録だけが遅延する誤検出）。`sleep_ms(0)`（`setTimeout(0)`
        // という macrotask）を挟むことで、その macrotask が実行される
        // 前に必ず全 microtask がフラッシュされる（HTML/JS 仕様の
        // 「macrotask 実行前に microtask キューを空にする」処理順序）
        // ことを利用し、確実に `MutationObserver` の通知を先に届ける。
        super::sleep_ms(0).await;
        assert_eq!(
            phase_label(),
            "running",
            "Runtime::wire_timer が dispatch 後の dirty_fields() を \
             apply_update_for_dirty へ渡し、C 側の束縛点（phase_label）が \
             再描画されること（イシュー #1959 の受け入れ条件）"
        );
        assert_eq!(
            records.borrow().len(),
            1,
            "start クリック 1 回につき data-state 属性への書き込みは \
             write_timer による 1 回のみであり、apply_update_for_dirty \
             側から Timer の data-state 自体への二重書き込みは \
             発生しないこと（イシュー #1959「二重描画にならない根拠」）: {:?}",
            records
                .borrow()
                .iter()
                .map(MutationRecord::attribute_name)
                .collect::<Vec<_>>()
        );

        let pause_button = root_el
            .query_selector("[data-scope='timer'][data-part='action-trigger'][data-action='pause']")
            .expect("query_selector must not fail")
            .expect("pause action-trigger must exist");
        pause_button
            .dispatch_event(&synthetic_click())
            .expect("dispatch_event must not fail");

        wait_for("data-state becomes paused after pause click", || {
            root_el.get_attribute("data-state").as_deref() == Some("paused")
        })
        .await;
        // start クリック直後と同じ理由（上記コメント参照）で、pause
        // クリック後も `MutationObserver` 通知の microtask フラッシュを
        // 待ってから件数を確認する。
        super::sleep_ms(0).await;
        assert_eq!(phase_label(), "paused");
        assert_eq!(
            records.borrow().len(),
            2,
            "pause クリック後は data-state 書き込みが累計 2 回（start 1 回 + \
             pause 1 回）のみであること: {:?}",
            records
                .borrow()
                .iter()
                .map(MutationRecord::attribute_name)
                .collect::<Vec<_>>()
        );

        observer.disconnect();
        assert_eq!(runtime.component().phase_label, "paused");
    }

    /// `Runtime::hydrate` 経由で配線した Timer の `setInterval` tick
    /// （`headless_timer::handle_tick` → `"timer:tick"` 通知 →
    /// `Runtime::wire_timer`）が `C`（`TimerHost`）側の束縛点
    /// （`data-bind-text="elapsed_label"`）へ実際に反映されること、かつ
    /// `write_timer` の `data-elapsed` 直書きと `apply_update_for_dirty` の
    /// 束縛点更新が整合していること（同じ値が 1 回だけ書かれ、二重描画に
    /// ならないこと）を検証する（受け入れ条件、イシュー #1960）。
    #[wasm_bindgen_test]
    async fn runtime_hydrate_tick_rerenders_elapsed_binding_and_writes_data_elapsed_once() {
        let window = web_sys::window().expect("window must exist");
        let document = window.document().expect("document must exist");

        // 無期限カウントアップ（`target_ms = 0`）・interval 30ms:
        // 既定タイムアウト（20 秒）内に複数 tick を決定的に観測できる
        // 小さい値（ファイル冒頭 `//!` の「短い interval・小さい目標値を
        // 使う理由」節と同じ方針）。
        let (root_el, runtime) = mount_host_and_hydrate(
            &document,
            "timer-host-runtime-dirty-rerender-tick-root",
            Timer::count_up(0, 30),
        );
        let _cleanup = RemoveOnDrop(root_el.clone());

        // 観測器は start クリック前に張る（最初の tick も取りこぼさない）。
        let (elapsed_observer, elapsed_records) = observe_attribute(&root_el, "data-elapsed");
        let (child_observer, child_records) = observe_child_list(&root_el);

        let elapsed_label = || {
            root_el
                .query_selector("[data-bind-text='elapsed_label']")
                .expect("query_selector must not fail")
                .expect("elapsed_label binding point must exist")
                .text_content()
                .unwrap_or_default()
        };
        assert_eq!(elapsed_label(), "0");
        assert_eq!(root_el.get_attribute("data-elapsed").as_deref(), Some("0"));

        let start_button = root_el
            .query_selector("[data-scope='timer'][data-part='action-trigger'][data-action='start']")
            .expect("query_selector must not fail")
            .expect("start action-trigger must exist");
        start_button
            .dispatch_event(&synthetic_click())
            .expect("dispatch_event must not fail");

        wait_for("data-state becomes running after start click", || {
            root_el.get_attribute("data-state").as_deref() == Some("running")
        })
        .await;

        // 束縛点（elapsed_label）が tick 経由で実際に再描画されること
        // （受け入れ条件 1）。
        wait_for("elapsed-label text advances past zero via tick", || {
            elapsed_label().parse::<u64>().unwrap_or(0) > 0
        })
        .await;

        // JS はシングルスレッドのため、同一同期ブロック内での 2 読み取り
        // の間に tick は挟まらない。束縛点（C 側）と Timer 自身の直書き
        // （data-elapsed）が常に一致していることの直接検証（受け入れ条件
        // 2 の一部）。
        assert_eq!(
            elapsed_label(),
            root_el.get_attribute("data-elapsed").unwrap_or_default(),
            "tick 経由で束縛点（elapsed_label）と Timer 自身の \
             data-elapsed 直書きは常に一致すること（C と DOM 側 Timer が \
             同じアクション列・同じ delta を同順で受けるため）"
        );

        // 継続更新の確認（少なくとも 2 tick 分の更新を観測する）。
        let first_value: u64 = elapsed_label().parse().unwrap_or(0);
        wait_for(
            "elapsed-label text advances past the first observed tick",
            || elapsed_label().parse::<u64>().unwrap_or(0) > first_value,
        )
        .await;

        let pause_button = root_el
            .query_selector("[data-scope='timer'][data-part='action-trigger'][data-action='pause']")
            .expect("query_selector must not fail")
            .expect("pause action-trigger must exist");
        pause_button
            .dispatch_event(&synthetic_click())
            .expect("dispatch_event must not fail");
        wait_for("data-state becomes paused after pause click", || {
            root_el.get_attribute("data-state").as_deref() == Some("paused")
        })
        .await;

        // interval 解除後の静止猶予（ファイル冒頭 `//!`
        // 「`wait_for(|| false)` を短い猶予待機へ置き換えた理由」節と同じ
        // 方針: `interval_ms`（30ms）の数個分の猶予を置けば、pending tick
        // が残っていれば必ずここで顕在化する）。
        super::sleep_ms(150).await;

        // 静止状態でのみ `runtime.component()` を借用する（ファイル冒頭
        // `//!` の「`runtime.component()` 借用に関する禁止事項」参照。
        // `Ref` は同一式内で即 drop する）。
        let tick_count = runtime.component().tick_count;
        assert!(
            tick_count >= 2,
            "少なくとも 2 tick 分の C 側更新（tick_count）が観測できること: {tick_count}"
        );

        assert_eq!(
            elapsed_records.borrow().len(),
            (tick_count + 2) as usize,
            "data-elapsed への書き込みは start クリック 1 回 + tick ごと \
             1 回（tick_count 回）+ pause クリック 1 回のみであり、\
             write_timer 以外の経路から data-elapsed への書き込みが \
             発生しないこと（受け入れ条件 2: 同じ値が 1 回だけ書かれ、\
             二重描画にならないこと）: tick_count={tick_count}, \
             records={:?}",
            elapsed_records
                .borrow()
                .iter()
                .map(MutationRecord::attribute_name)
                .collect::<Vec<_>>()
        );

        assert_eq!(
            child_records.borrow().len(),
            0,
            "tick ごとに apply_update_for_dirty の構造フォールバック \
             （root 全子ノード再構築）が走っていないこと（elapsed_label が \
             束縛点で解決済みであることの証拠）: {:?}",
            child_records.borrow().len()
        );

        let final_elapsed_label = elapsed_label();
        assert_eq!(
            final_elapsed_label,
            root_el.get_attribute("data-elapsed").unwrap_or_default(),
            "静止後も束縛点と data-elapsed 直書きが一致していること"
        );
        assert_eq!(
            runtime.component().elapsed_label,
            final_elapsed_label,
            "C 側状態（elapsed_label）と DOM 側束縛点の表示が一致すること"
        );

        elapsed_observer.disconnect();
        child_observer.disconnect();
    }
}

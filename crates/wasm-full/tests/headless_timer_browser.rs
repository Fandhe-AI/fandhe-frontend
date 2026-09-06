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
//! する。tick 経由の束縛点更新は #1960（本イシューの sub-issue 2）に委ねる。

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

// --- 検証: Runtime::hydrate 経由の再描画接続（イシュー #1959） -----------

mod runtime_dirty_rerender {
    use super::{create_container, synthetic_click, wait_for, RemoveOnDrop};
    use fandhe_frontend_core::{bind_text, render, Node};
    use fandhe_frontend_headless_ui::timer::{action_trigger, control, Timer, TimerControl};
    use fandhe_frontend_interactive::{Component, DirtyTracked, Hydrate, HydrateError};
    use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
    use fandhe_frontend_wasm_full::Runtime;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;
    use web_sys::{MutationObserver, MutationObserverInit, MutationRecord};

    /// `Runtime::hydrate` が Timer root 要素そのものを `root_id` として扱う
    /// 前提（`crate::lib::Runtime::wire_timer` rustdoc「前提: Runtime root が
    /// Timer root であること」参照）に合わせ、`view()` のルートへ固定 id を
    /// 付与する。本ファイル内で 1 テストのみが本ホストを使うため、
    /// `runtime_browser.rs::AppState` と同様に単一の固定文字列で足りる。
    const ROOT_ID: &str = "timer-host-runtime-dirty-rerender-root";

    /// `Runtime::wire_timer`（イシュー #1959）の dispatch 後再描画接続を
    /// 実 DOM で固定するための最小ホスト。`Timer` をラップし、アプリ側の
    /// 派生フィールド（フェーズラベル）のみを dirty へ積む
    /// （`crates/wasm-full/tests/headless_timer.rs::wire_timer_dirty_contract::TimerHost`
    /// と同じ設計。あちらは native で `dirty_fields()` の内容自体を固定し、
    /// 本ホストは実 DOM でのハイドレーション・束縛点反映・二重描画回避を
    /// 検証する点が異なる）。tick 経由の束縛点更新検証は #1960 に委ねる
    /// ため、start/pause の 2 アクションのみを対象にする。
    struct TimerHost {
        timer: Timer,
        phase_label: String,
        dirty: Vec<&'static str>,
    }

    impl TimerHost {
        fn new(timer: Timer) -> Self {
            let phase_label = timer.phase().as_str().to_string();
            Self {
                timer,
                phase_label,
                dirty: Vec::new(),
            }
        }
    }

    impl Component for TimerHost {
        type Action = <Timer as Component>::Action;

        fn update(&mut self, action: Self::Action) {
            self.dirty.clear();
            self.timer.update(action);
            let new_label = self.timer.phase().as_str().to_string();
            if new_label != self.phase_label {
                self.phase_label = new_label;
                self.dirty.push("phase_label");
            }
        }

        fn view(&self) -> Node {
            self.timer.root(
                vec![("id", ROOT_ID)],
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

    /// `root` へ `MutationObserver`（`attributes: true`,
    /// `attribute_filter: ["data-state"]`）を張り、`data-state` の変更を
    /// 記録する（`crates/wasm-full/tests/keyed_update_browser.rs` と同じ
    /// パターン）。呼び出し側が `observer.disconnect()` を制御できるよう
    /// `MutationObserver` 自身も返す。
    fn observe_data_state(
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
        // `callback` は observer が生存する限り呼ばれ続けるため forget する
        // （`keyed_update_browser.rs` と同じ理由：テスト末尾で
        // `observer.disconnect()` を呼ぶため無期限リークにはならない）。
        callback.forget();
        let init = MutationObserverInit::new();
        init.set_attributes(true);
        let filter = js_sys::Array::new();
        filter.push(&wasm_bindgen::JsValue::from_str("data-state"));
        init.set_attribute_filter(&filter);
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
        // `create_container` は body 直下に空の div を追加するだけの薄い
        // ヘルパーであり、他テストと同じ「後始末対象を必ず 1 個持つ」規約
        // を守るためだけに使う（実際の Timer root は `insert_adjacent_html`
        // で別途 body 直下へ挿入する。二重に後始末しないよう、こちらは
        // 未使用のまま `RemoveOnDrop` で除去する）。
        let marker = create_container(&document, "timer-host-runtime-dirty-rerender-marker");
        let _marker_cleanup = RemoveOnDrop(marker);

        let host = TimerHost::new(Timer::count_up(0, 60_000));

        let html = render(&host.view());
        document
            .body()
            .expect("document body must exist in browser test environment")
            .insert_adjacent_html("beforeend", &html)
            .expect("insert_adjacent_html must not fail");
        let root_el = document
            .get_element_by_id(ROOT_ID)
            .expect("rendered Timer root must have the expected id");
        let _cleanup = RemoveOnDrop(root_el.clone());

        // `render_for_hydration` が行う「view() の root へ hydration_attrs
        // を後付けする」処理を、実 DOM 属性として直接再現する
        // （`runtime_browser.rs::hydrate_restores_state_from_existing_dom_and_wires_events`
        // と同じ手順）。
        for (name, value) in host.hydration_attrs() {
            root_el
                .set_attribute(&name, &value)
                .expect("set_attribute must not fail");
        }

        let runtime = Runtime::hydrate(ROOT_ID, TimerHost::new(Timer::count_up(0, 60_000)))
            .expect("hydrate must succeed for well-formed attrs");
        assert_eq!(
            runtime.root().id(),
            ROOT_ID,
            "hydrate は root_id 要素自身を Timer root として復元すること"
        );

        let (observer, records) = observe_data_state(&root_el);

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
}

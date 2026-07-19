//! TASK-11.5（親イシュー #85、REQ-11）: 実ブラウザ性能計測ハーネス。
//!
//! TASK-11.5【Conditional Go 条件 1】は、PoC-5 の Node.js 近似計測に代えて
//! 実ブラウザで初期ロード（描画＋ハイドレーション完了 300ms 以内）・DOM 操作
//! 性能（16ms/フレーム予算内）を正式計測することを求める
//! （`docs/spec/05-tasks.md` TASK-11.5）。3 分割サブタスクのうち、本ファイルは
//! TASK-11.5a（#86・計測ハーネス構築）と TASK-11.5b（#87・計測実行・しきい値
//! 有効化）の双方の成果物を統合したものである。
//!
//! `docs/guides/browser-testing.md`（TASK-6.3a・#65）が構築した実ブラウザテスト環境
//! （headless Chromium・`wasm-pack test --headless`）を再利用する。
//!
//! # 計測シナリオ（製品経路、TASK-11.5b・#87 で確定）
//!
//! `Runtime::mount`/`Runtime::hydrate`（TASK-11.2d・#77、TASK-11.4b・#83）が
//! マージ済みのため、本ハーネスは PoC-5 相当の近似ではなく `fandhe_frontend_wasm_full::Runtime`
//! を直接経由する製品経路を計測する（`docs/design/wasm-full-architecture.md` 第 3.2 節）。
//!
//! - `initial_load`: SSR 済み HTML（[`fandhe_frontend_interactive::render_for_hydration`] →
//!   [`fandhe_frontend_core::render`]、既定エスケープ済み）の `set_inner_html` 反映（サーバー
//!   側責務相当・計測対象外）ののち、[`fandhe_frontend_wasm_full::Runtime::hydrate`] 呼び出し
//!   完了までの経過時間を計測する。これは状態復元・イベント配線という
//!   クライアント側責務そのものであり、[`Runtime::hydrate`] 内部処理と一致する
//!   （`wasm-full/src/lib.rs::Runtime::hydrate` 参照）。有界サンプル数
//!   （[`INITIAL_LOAD_SAMPLES`]）で統計値（mean/p95/max）を算出する。
//! - `dom_update`: [`fandhe_frontend_wasm_full::Runtime::mount`] 済み DOM の
//!   `[data-testid='inc-btn']` へ合成 `click` イベント（`bubbles: true`）を
//!   発火し、その同期実行（イベント委譲 → `dispatch` → 条件付き `dom::paint`）の
//!   所要時間を 1 操作として計測する（`tests/runtime_browser.rs` の合成イベント
//!   パターンを踏襲）。固定サンプル数 [`DOM_UPDATE_SAMPLES`] を維持する。
//!
//! # 不変条件（REQ-1、`docs/design/wasm-full-architecture.md` 第 7 節・不変条件 1 継承）
//!
//! `set_inner_html` へ渡す文字列は [`fandhe_frontend_core::render`] の既定エスケープ済み
//! 出力のみとする（`Runtime::mount`/`hydrate` 内部の `dom::paint` が担保）。
//! 本ハーネス自身も `format!` 等による HTML 文字列直接組み立て・
//! `fandhe_frontend_core::raw_html()` の呼び出しは一切行わない。
//!
//! # 出力契約（機械可読 1 行サマリ）
//!
//! 各計測は次の形式で `console.log` へ 1 行出力する
//! （TASK-11.5c・#88 の計測レポートが収集する契約、変更しない）。
//!
//! ```text
//! perf-browser: metric=<name> samples=<n> mean_ms=<x> p95_ms=<x> max_ms=<x>
//! ```
//!
//! この契約は [`format_summary_line`] のテスト（`#[cfg(test)]`）で固定する。
//!
//! # しきい値アサーション（feature = "perf-assert"）
//!
//! CI 共有ランナーのノイズで正式判定できないため、既定（feature 無効）では
//! ハーネス自己検証（サンプル数 > 0・値が有限かつ非負・出力行の形式）のみを
//! 行う。統制されたローカル環境での正式計測時のみ
//! `--features perf-assert` を付けて実行し、[`INITIAL_LOAD_BUDGET_MS`]（mean
//! 基準）・[`FRAME_BUDGET_MS`]（p95 基準＋ 16ms 超過率 5% 以下）をアサートする
//! （`docs/ci/perf-browser-harness.md` 第 5 節の実行手順、`docs/reports/perf-browser-report.md`
//! 第 3 節の判定基準）。CI `perf-harness` ジョブは feature なしのまま実行し、
//! スモーク（ハーネス自己検証）のみを行う方針を維持する。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_interactive::AppState;
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit};

wasm_bindgen_test_configure!(run_in_browser);

/// REQ-11 性能予算: 初期ロード（描画＋ハイドレーション完了）が満たすべき上限（ミリ秒）。
///
/// `docs/spec/05-tasks.md` TASK-11.5 の受け入れ基準。しきい値アサーションは
/// feature `perf-assert` 有効時のみ行う（ファイル冒頭 `//!` 参照）。
#[cfg_attr(not(feature = "perf-assert"), allow(dead_code))]
const INITIAL_LOAD_BUDGET_MS: f64 = 300.0;

/// REQ-11 性能予算: DOM 操作 1 回あたりが満たすべきフレーム予算（ミリ秒）。
#[cfg_attr(not(feature = "perf-assert"), allow(dead_code))]
const FRAME_BUDGET_MS: f64 = 16.0;

/// `dom_update` の 16ms 超過率がこれを超える場合、`p95_ms` が予算内でも
/// 単純な Go とはしない目安（`docs/reports/perf-browser-report.md` 第 3 節）。
#[cfg_attr(not(feature = "perf-assert"), allow(dead_code))]
const FRAME_OVERAGE_RATIO_BUDGET: f64 = 0.05;

/// `dom_update` シナリオの固定サンプル数。
///
/// 計測ループを有界にすることで無制限のメモリ・リスナー蓄積を作らない
/// （`.claude/rules/security.md` A04 相当）。`Runtime::mount` は 1 回のみ呼び、
/// イベント委譲リスナーも 1 回のみ登録される（`events.rs` 契約）ため、
/// このループ自体はリスナーを追加蓄積しない。
const DOM_UPDATE_SAMPLES: usize = 100;

/// `initial_load` シナリオの固定サンプル数。
///
/// `Runtime::hydrate` をサンプル数分だけ呼ぶため、`wire_events` が
/// マウントのたびに新規リスナーを `Closure::forget` する
/// （`wasm-full/src/events.rs` 契約）。蓄積はこの定数で有界にする
/// （A04、ファイル冒頭 `//!` 参照）。サンプルごとに前回のコンテナを
/// `remove()` してから次サンプルを生成するため、DOM 自体は蓄積しない。
const INITIAL_LOAD_SAMPLES: usize = 30;

/// 経過時間の統計サマリ（mean / p95 / max、単位ミリ秒）。
#[derive(Debug, Clone, Copy, PartialEq)]
struct DurationStats {
    samples: usize,
    mean_ms: f64,
    p95_ms: f64,
    max_ms: f64,
}

/// 経過時間サンプル列から [`DurationStats`] を算出する。
///
/// DOM/`wasm-bindgen` に依存しない純粋関数のため native `cargo test` でも
/// 検証できる（`#[cfg(test)]` 参照）。空スライスは呼び出し側の契約違反として
/// panic ではなく全項目 0.0 のサマリを返す（安全側フォールバック）。
fn summarize(samples: &[f64]) -> DurationStats {
    let n = samples.len();
    if n == 0 {
        return DurationStats {
            samples: 0,
            mean_ms: 0.0,
            p95_ms: 0.0,
            max_ms: 0.0,
        };
    }

    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mean_ms = sorted.iter().sum::<f64>() / n as f64;
    let max_ms = sorted[n - 1];
    // p95 インデックス: 0 始まりの切り上げ位置（n=1 なら常に最大値を p95 とする）。
    let p95_index = ((n as f64) * 0.95).ceil() as usize;
    let p95_index = p95_index.saturating_sub(1).min(n - 1);
    let p95_ms = sorted[p95_index];

    DurationStats {
        samples: n,
        mean_ms,
        p95_ms,
        max_ms,
    }
}

/// `dom_update` について、16ms（[`FRAME_BUDGET_MS`]）を超過したサンプルの
/// 割合を算出する（`docs/reports/perf-browser-report.md` 第 3 節「16ms 超過率」）。
/// 空スライスは 0.0 を返す安全側フォールバック（呼び出し元は非空を保証する）。
#[cfg_attr(not(feature = "perf-assert"), allow(dead_code))]
fn frame_overage_ratio(samples: &[f64]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let over = samples.iter().filter(|&&s| s > FRAME_BUDGET_MS).count();
    over as f64 / samples.len() as f64
}

/// TASK-11.5b/c が収集する機械可読 1 行サマリを整形する（出力契約本体）。
///
/// 契約: `perf-browser: metric=<name> samples=<n> mean_ms=<x> p95_ms=<x> max_ms=<x>`
/// （ファイル冒頭 `//!` 参照）。この書式は [`tests::format_summary_line_matches_contract`]
/// で固定する。
fn format_summary_line(metric: &str, stats: DurationStats) -> String {
    format!(
        "perf-browser: metric={metric} samples={} mean_ms={:.3} p95_ms={:.3} max_ms={:.3}",
        stats.samples, stats.mean_ms, stats.p95_ms, stats.max_ms
    )
}

/// 現在時刻（ミリ秒、`Performance.now()`）を取得する。
///
/// `window()`/`performance()` の取得失敗は headless Chromium 環境では
/// 想定しない異常系のため `expect` する（テストコードのみの許容、
/// `.claude/rules/coding-rust.md` テスト規約）。
fn now_ms() -> f64 {
    web_sys::window()
        .expect("window must exist in browser test environment")
        .performance()
        .expect("Performance API must be available in browser test environment")
        .now()
}

/// 計測用のルートコンテナ要素を document body へ 1 個生成する。
///
/// 呼び出しごとに一意な id を振ることで、同一テストバイナリ内の複数テスト
/// ケース・複数サンプルが要素を奪い合わないようにする
/// （`wasm-pack test --headless` は本ファイル内の全テスト関数を同一ページ上で
/// 順に実行し、ページリロードを行わない。`tests/runtime_browser.rs` と同じ
/// 前提）。
fn create_scenario_container(document: &Document, id: &str) -> Element {
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

/// テスト内で生成したコンテナ要素を、スコープを抜ける際（`expect` 等からの
/// panic による巻き戻し経由も含む）に確実に DOM から除去するための RAII
/// ガード（`tests/runtime_browser.rs::RemoveOnDrop` と同じ意図・同じ実装）。
///
/// `run_initial_load`/`setup_dom_update_scenario` は `hydrate`/`mount` の
/// `expect` が失敗しうる。手動の `remove()` 呼び出しのみに頼ると panic 時に
/// それを迂回してしまい、`"interactive-root"` 等のコンテナが共有ページに
/// 残留し後続のサンプル・テストを汚染する（`wasm-pack test --headless` は
/// 本ファイル内の全テスト関数を同一ページ上で順に実行するため）。
/// `wasm-bindgen-test` はテスト関数単位で panic を catch し巻き戻す
/// （unwind）ため、`Drop` はその巻き戻し経路でも実行される。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// 合成 `click` イベントを生成する（`bubbles: true`）。
///
/// `Runtime::mount`/`hydrate` はリスナーをルート要素へ登録するため、子要素上で
/// 発火したイベントがバブリングでルートまで届く必要がある
/// （`tests/runtime_browser.rs::bubbling_event` と同じ意図）。
fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail")
}

/// `initial_load` シナリオ 1 回分: SSR 済み HTML の `set_inner_html` 反映
/// （サーバー側責務相当・計測対象外）ののち、[`Runtime::hydrate`] 呼び出し
/// 完了までの経過時間（ミリ秒）を計測する。
///
/// `wrapper_id` は呼び出しごとに一意な値を渡すこと。[`Runtime::hydrate`] が
/// 読み取る `root_id`（`"interactive-root"`、[`fandhe_frontend_interactive::AppState::view`]
/// 固定値）は SSR 出力のルート要素自身が持つため、サンプル間で前回の
/// `wrapper` を除去してから次サンプルを実行しないと
/// `document.get_element_by_id("interactive-root")` が過去サンプルの残留要素に
/// ヒットしうる。この後始末は [`RemoveOnDrop`] ガードで行うため、
/// `Runtime::hydrate` の `expect` が panic した場合でもスコープを抜ける際に
/// 確実に実行される（成功パスのみで `remove()` する場合に生じる panic-safety
/// 上の問題を回避する）。本関数はその後始末まで含めて 1 サンプル分の責務と
/// する。
fn run_initial_load(document: &Document, wrapper_id: &str) -> f64 {
    let state = AppState::new();

    // サーバー側責務相当（計測対象外）: ハイドレーション属性付き Node を
    // 既定エスケープ済み HTML 文字列へ描画し、SSR 済みページの初期 DOM として
    // 反映する（`fandhe_frontend_interactive::render_for_hydration` → `fandhe_frontend_core::render`。
    // REQ-1 契約は `fandhe-frontend-interactive`/`fandhe-frontend-core` 側で担保済み、`raw_html()` は
    // 使用しない）。
    let html =
        fandhe_frontend_core::render(&fandhe_frontend_interactive::render_for_hydration(&state));
    let wrapper = create_scenario_container(document, wrapper_id);
    // 以降で panic しても関数を抜ける際に `wrapper` を確実に除去する
    // （成功パスのみの `remove()` 呼び出しでは `expect` の panic をすり抜けて
    // しまい、`"interactive-root"` が共有ページに残留しうる）。
    let _cleanup = RemoveOnDrop(wrapper.clone());
    wrapper.set_inner_html(&html);

    // クライアント側責務（計測対象）: 状態復元（`data-hydrate-*` 属性からの
    // `Hydrate::from_hydration_attrs` 相当）＋イベント配線までを一括して行う
    // `Runtime::hydrate` 呼び出し完了までの経過時間。これは
    // 「描画→状態復元→イベント配線」という製品経路そのものである
    // （`wasm-full/src/lib.rs::Runtime::hydrate` 参照）。
    let start = now_ms();
    let _runtime = Runtime::hydrate("interactive-root", AppState::new())
        .expect("hydrate must succeed for well-formed SSR output");

    now_ms() - start
}

/// `initial_load` の [`INITIAL_LOAD_SAMPLES`] 回分を計測する。
fn run_initial_load_samples() -> Vec<f64> {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");

    (0..INITIAL_LOAD_SAMPLES)
        .map(|i| {
            let wrapper_id = format!("perf-initial-load-wrapper-{i}");
            run_initial_load(&document, &wrapper_id)
        })
        .collect()
}

/// `dom_update` シナリオ用に [`Runtime::mount`] 済みの DOM を用意し、
/// `[data-testid='inc-btn']` 要素を返す。
///
/// `root_id` は呼び出しごとに一意な値を渡すこと（`Runtime::mount` は
/// `root_id` 要素自身へ描画するため、`tests/runtime_browser.rs` の
/// `create_placeholder` と同じ前提）。戻り値のコンテナは呼び出し元が
/// [`RemoveOnDrop`] で包み、用途終了後（後続のイベント発火が panic した
/// 場合も含む）に確実に除去すること。
///
/// `Runtime::mount` の `expect` がこの関数内で panic した場合に備え、
/// コンテナ生成直後から一時的に [`RemoveOnDrop`] で保護する。`mount` 成功後は
/// 後始末の責務を呼び出し元へ引き継ぐため、ここでは `mem::forget` によって
/// 早期の除去を防ぐ（二重 `remove()` を避ける）。
fn setup_dom_update_scenario(document: &Document, root_id: &str) -> Element {
    let placeholder = create_scenario_container(document, root_id);
    let cleanup = RemoveOnDrop(placeholder.clone());
    let _runtime = Runtime::mount(root_id, AppState::new()).expect("mount must succeed");
    std::mem::forget(cleanup);
    placeholder
}

/// `placeholder` 配下の増加ボタンを再取得して合成 `click` を発火し、その
/// 同期実行（イベント委譲 → `dispatch` → 条件付き `dom::paint`）の所要時間
/// （ミリ秒）を計測する。
///
/// `Runtime::mount`/`hydrate` の `on_action` は dispatch 成功かつ再描画対象
/// （`increment` アクション）の場合に `dom::paint` で `set_inner_html` を
/// 呼び直す（`wasm-full/src/lib.rs::Runtime::wire` 参照）ため、直前の
/// ボタン要素参照は次回発火前に無効化されうる。毎回 `query_selector` で
/// 再取得することで、テスト実行順序・再描画有無に依存しない自己完結な
/// 計測にする（クエリ自体は計測区間の外側で行う）。
fn run_dom_update_iteration(placeholder: &Element) -> f64 {
    let button = placeholder
        .query_selector("[data-testid='inc-btn']")
        .expect("query_selector must not fail")
        .expect("increment button must exist after mount/paint");

    let start = now_ms();
    button
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
    now_ms() - start
}

/// `initial_load` シナリオの自己検証: 有界サンプル数（[`INITIAL_LOAD_SAMPLES`]）
/// が収集でき、各値が有限かつ非負であること・出力契約の 1 行サマリが期待
/// 書式であることを検証する。性能予算に対するアサーションは
/// feature `perf-assert` 有効時のみ行う（[`initial_load_meets_budget`]）。
///
/// `feature = "perf-assert"` 有効時は [`initial_load_meets_budget`] が
/// 独立したサンプル実行から同一メトリクスの正式サマリ行を出力するため、
/// 本テストの `console.log` はここでは行わない（`format_summary_line` の
/// 書式検証自体は feature 無効時と同様に行う）。両者が同時に `console.log`
/// すると「1 メトリクスにつき 1 行」という収集契約（ファイル冒頭 `//!`）が
/// 崩れ、正式計測時に budget アサーションが評価したものと異なるサンプル
/// セットの行が記録されてしまう。
#[wasm_bindgen_test]
fn initial_load_harness_produces_finite_nonnegative_samples() {
    let samples = run_initial_load_samples();

    assert_eq!(samples.len(), INITIAL_LOAD_SAMPLES);
    assert!(
        samples.iter().all(|s| s.is_finite() && *s >= 0.0),
        "initial_load の各サンプルは有限かつ非負であること: {samples:?}"
    );

    let stats = summarize(&samples);
    assert_eq!(stats.samples, INITIAL_LOAD_SAMPLES);

    let summary = format_summary_line("initial_load", stats);
    #[cfg(not(feature = "perf-assert"))]
    web_sys::console::log_1(&summary.clone().into());

    assert!(
        summary.starts_with(&format!(
            "perf-browser: metric=initial_load samples={INITIAL_LOAD_SAMPLES} "
        )),
        "出力契約（1 行サマリ）を満たすこと: {summary}"
    );
}

/// `dom_update` シナリオの自己検証: [`DOM_UPDATE_SAMPLES`] 回のループが
/// 有界であること・mean/p95/max がいずれも有限かつ非負であること・出力契約の
/// 書式を満たすことを検証する（フレーム予算に対するアサーションは
/// feature `perf-assert` 有効時のみ、[`dom_update_meets_frame_budget`]）。
///
/// `feature = "perf-assert"` 有効時は [`dom_update_meets_frame_budget`] が
/// 独立したサンプル実行から同一メトリクスの正式サマリ行を出力するため、
/// 本テストの `console.log` は行わない（[`initial_load_harness_produces_finite_nonnegative_samples`]
/// と同じ重複回避方針）。
#[wasm_bindgen_test]
fn dom_update_harness_produces_bounded_finite_samples() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = setup_dom_update_scenario(&document, "perf-dom-update-mount-root");
    // 後続の `run_dom_update_iteration`（`dispatch_event`/`expect`）が panic
    // しても `placeholder` を確実に除去する（成功パスのみの `remove()` 呼び
    // 出しでは panic をすり抜けてしまう）。
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let samples: Vec<f64> = (0..DOM_UPDATE_SAMPLES)
        .map(|_| run_dom_update_iteration(&placeholder))
        .collect();

    assert_eq!(samples.len(), DOM_UPDATE_SAMPLES);
    assert!(
        samples.iter().all(|s| s.is_finite() && *s >= 0.0),
        "dom_update の各サンプルは有限かつ非負であること: {samples:?}"
    );

    let stats = summarize(&samples);
    assert_eq!(stats.samples, DOM_UPDATE_SAMPLES);

    let summary = format_summary_line("dom_update", stats);
    #[cfg(not(feature = "perf-assert"))]
    web_sys::console::log_1(&summary.clone().into());

    assert!(
        summary.starts_with(&format!(
            "perf-browser: metric=dom_update samples={DOM_UPDATE_SAMPLES} "
        )),
        "出力契約（1 行サマリ）を満たすこと: {summary}"
    );
}

/// `initial_load` の正式しきい値判定（TASK-11.5b・#87、REQ-11）。
///
/// 統制されたローカル環境でのみ有効化する（`--features perf-assert`、
/// ファイル冒頭 `//!` 参照）。`mean_ms` を基準に [`INITIAL_LOAD_BUDGET_MS`]
/// （300ms）以内であることをアサートする（`docs/reports/perf-browser-report.md`
/// 第 3 節: `initial_load` は `mean_ms` を基準とし `p95_ms` は安定性確認に
/// 用いる）。
#[cfg(feature = "perf-assert")]
#[wasm_bindgen_test]
fn initial_load_meets_budget() {
    let samples = run_initial_load_samples();
    let stats = summarize(&samples);

    let summary = format_summary_line("initial_load", stats);
    web_sys::console::log_1(&summary.clone().into());

    assert!(
        stats.mean_ms <= INITIAL_LOAD_BUDGET_MS,
        "initial_load の mean_ms が予算 {INITIAL_LOAD_BUDGET_MS}ms を超過: {summary}"
    );
}

/// `dom_update` の正式しきい値判定（TASK-11.5b・#87、REQ-11）。
///
/// 統制されたローカル環境でのみ有効化する（`--features perf-assert`）。
/// `p95_ms` を主判定に用い（`mean_ms` 単独では判定しない、
/// `docs/reports/perf-browser-report.md` 第 3 節）、16ms 超過率も
/// [`FRAME_OVERAGE_RATIO_BUDGET`]（目安 5%）以下であることをあわせて
/// アサートする。
#[cfg(feature = "perf-assert")]
#[wasm_bindgen_test]
fn dom_update_meets_frame_budget() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let placeholder = setup_dom_update_scenario(&document, "perf-dom-update-budget-mount-root");
    // 後続の `run_dom_update_iteration`（`dispatch_event`/`expect`）が panic
    // しても `placeholder` を確実に除去する（成功パスのみの `remove()` 呼び
    // 出しでは panic をすり抜けてしまう）。
    let _cleanup = RemoveOnDrop(placeholder.clone());

    let samples: Vec<f64> = (0..DOM_UPDATE_SAMPLES)
        .map(|_| run_dom_update_iteration(&placeholder))
        .collect();

    let stats = summarize(&samples);
    let overage_ratio = frame_overage_ratio(&samples);

    let summary = format_summary_line("dom_update", stats);
    web_sys::console::log_1(&summary.clone().into());
    web_sys::console::log_1(
        &format!("perf-browser: metric=dom_update overage_ratio={overage_ratio:.3}").into(),
    );

    assert!(
        stats.p95_ms <= FRAME_BUDGET_MS,
        "dom_update の p95_ms が予算 {FRAME_BUDGET_MS}ms を超過: {summary}"
    );
    assert!(
        overage_ratio <= FRAME_OVERAGE_RATIO_BUDGET,
        "dom_update の 16ms 超過率が目安 {FRAME_OVERAGE_RATIO_BUDGET} を超過: ratio={overage_ratio:.3} {summary}"
    );
}

/// native（rlib）で検証できる純粋関数のみを対象にした回帰テスト群。
///
/// `summarize`/`format_summary_line`/`frame_overage_ratio` は DOM/`wasm-bindgen`
/// に依存しないが、ファイル全体が `#![cfg(target_arch = "wasm32")]` で
/// ゲートされているため、これらの単体テストも wasm32 ターゲット上でのみ
/// 実行される（`wasm-pack test` 経由。native `cargo test --workspace` は
/// このファイル自体をコンパイル対象から除外するため、既存 CI の
/// test/forbid-unsafe ジョブに影響しない）。
#[cfg(test)]
mod tests {
    use super::*;

    #[wasm_bindgen_test]
    fn summarize_of_single_sample_reports_that_value_for_all_stats() {
        let stats = summarize(&[12.5]);
        assert_eq!(stats.samples, 1);
        assert_eq!(stats.mean_ms, 12.5);
        assert_eq!(stats.p95_ms, 12.5);
        assert_eq!(stats.max_ms, 12.5);
    }

    #[wasm_bindgen_test]
    fn summarize_of_empty_slice_is_safe_fallback_not_panic() {
        let stats = summarize(&[]);
        assert_eq!(stats.samples, 0);
        assert_eq!(stats.mean_ms, 0.0);
        assert_eq!(stats.p95_ms, 0.0);
        assert_eq!(stats.max_ms, 0.0);
    }

    #[wasm_bindgen_test]
    fn summarize_picks_max_as_p95_when_all_samples_equal() {
        let stats = summarize(&[5.0, 5.0, 5.0, 5.0]);
        assert_eq!(stats.mean_ms, 5.0);
        assert_eq!(stats.p95_ms, 5.0);
        assert_eq!(stats.max_ms, 5.0);
    }

    #[wasm_bindgen_test]
    fn format_summary_line_matches_contract() {
        let stats = DurationStats {
            samples: 3,
            mean_ms: 1.0,
            p95_ms: 2.0,
            max_ms: 3.0,
        };
        let line = format_summary_line("initial_load", stats);
        assert_eq!(
            line,
            "perf-browser: metric=initial_load samples=3 mean_ms=1.000 p95_ms=2.000 max_ms=3.000"
        );
    }

    #[wasm_bindgen_test]
    fn frame_overage_ratio_of_empty_slice_is_safe_fallback() {
        assert_eq!(frame_overage_ratio(&[]), 0.0);
    }

    #[wasm_bindgen_test]
    fn frame_overage_ratio_counts_samples_strictly_over_budget() {
        // 16.0（境界値）は超過に含めず、16.1 のみ超過としてカウントする。
        let ratio = frame_overage_ratio(&[10.0, 16.0, 16.1, 20.0]);
        assert_eq!(ratio, 0.5);
    }
}

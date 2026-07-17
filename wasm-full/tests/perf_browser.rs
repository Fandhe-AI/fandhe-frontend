//! TASK-11.5a（#86）: 実ブラウザ性能計測ハーネス。
//!
//! TASK-11.5【Conditional Go 条件 1】（親イシュー #85、REQ-11）は、PoC-5 の
//! Node.js 近似計測に代えて実ブラウザで初期ロード（描画＋ハイドレーション完了
//! 300ms 以内）・DOM 操作性能（16ms/フレーム予算内）を正式計測することを求める
//! （`docs/spec/05-tasks.md` TASK-11.5）。本ファイルは 3 分割の 1 番目
//! （TASK-11.5a・本イシュー）が担う「計測を実行できる仕組み」であり、
//! 正式計測の実行・しきい値アサーションの有効化は TASK-11.5b（#87）、
//! 計測レポート・条件 1 解消判定は TASK-11.5c（#88）へ引き継ぐ
//! （`docs/perf-browser-harness.md` 引き継ぎ表参照）。
//!
//! `docs/browser-testing.md`（TASK-6.3a・#65）が構築した実ブラウザテスト環境
//! （headless Chromium・`wasm-pack test --headless`）を再利用する。
//!
//! # 計測シナリオの近似について
//!
//! `Runtime::mount`/`Runtime::hydrate`（`docs/wasm-full-architecture.md` 第 3.2 節）
//! は本コミット時点で未実装（TASK-11.2d・#77、TASK-11.4b・#83 が並列進行中）。
//! そのため本ハーネスは現行の公開面（[`rws_wasm_full::render_component_html`]・
//! [`rws_wasm_full::events`]・`rws_interactive::{dispatch, AppState, Hydrate}`）を
//! 組み合わせて製品経路（描画 → ハイドレーション属性からの状態復元 → イベント配線 →
//! dispatch → 再描画）を近似する。`Runtime` マージ後にシナリオ関数
//! （[`run_initial_load`]/[`run_dom_update_iteration`]）を差し替える継ぎ目として
//! 分離してあり、差し替え判断は TASK-11.5b（#87）で行う。
//!
//! # 不変条件（REQ-1、`docs/wasm-full-architecture.md` 第 7 節・不変条件 1 継承）
//!
//! `set_inner_html` へ渡す文字列は [`rws_core::render`]（`render_component_html`
//! 経由）の既定エスケープ済み出力のみとする。`format!` 等による HTML 文字列
//! 直接組み立て・`rws_core::raw_html()` の呼び出しは一切行わない
//! （`wasm-full/src/dom.rs` と同一の契約）。
//!
//! # 出力契約（機械可読 1 行サマリ）
//!
//! 各計測は次の形式で `console.log` へ 1 行出力する（既存の loc-check
//! サマリ契約に倣う。TASK-11.5b/c はこの行を収集する契約とする）。
//!
//! ```text
//! perf-browser: metric=<name> samples=<n> mean_ms=<x> p95_ms=<x> max_ms=<x>
//! ```
//!
//! この契約は [`format_summary_line`] のテスト（native、`#[cfg(test)]`）で
//! 固定する。
//!
//! # 本イシューでのアサーション方針
//!
//! 性能予算（[`INITIAL_LOAD_BUDGET_MS`]/[`FRAME_BUDGET_MS`]）は定数として定義
//! するが、しきい値アサーションの有効化は行わない（CI 共有ランナーのノイズで
//! 正式判定できないため）。本イシューのテストはハーネス自己検証
//! （サンプル数 > 0・値が有限かつ非負・出力行の形式）のみを行う。正式計測は
//! TASK-11.5b（#87）で実行環境を統制して行う。

#![cfg(target_arch = "wasm32")]

use rws_interactive::{dispatch, AppState, Hydrate};
use rws_wasm_full::events::{wire_events, ActionRef};
use rws_wasm_full::render_component_html;
use wasm_bindgen_test::*;
use web_sys::{Document, Element};

wasm_bindgen_test_configure!(run_in_browser);

/// REQ-11 性能予算: 初期ロード（描画＋ハイドレーション完了）が満たすべき上限（ミリ秒）。
///
/// `docs/spec/05-tasks.md` TASK-11.5 の受け入れ基準。本イシューではしきい値
/// アサーションを有効化しない（TASK-11.5b・#87 のスコープ）。
#[allow(dead_code)]
const INITIAL_LOAD_BUDGET_MS: f64 = 300.0;

/// REQ-11 性能予算: DOM 操作 1 回あたりが満たすべきフレーム予算（ミリ秒）。
///
/// 同上、しきい値アサーションの有効化は TASK-11.5b（#87）のスコープ。
#[allow(dead_code)]
const FRAME_BUDGET_MS: f64 = 16.0;

/// `dom_update` シナリオの固定サンプル数。
///
/// 計測ループを有界にすることで無制限のメモリ・リスナー蓄積を作らない
/// （`.claude/rules/security.md` A04 相当、`docs/wasm-full-architecture.md`
/// 第 7 節の設計方針を計測ハーネスにも適用する）。
const DOM_UPDATE_SAMPLES: usize = 100;

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
/// ケースが要素を奪い合わないようにする。
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

/// `initial_load` シナリオ本体: 状態 → ハイドレーション属性付き描画 →
/// `set_inner_html` → 属性からの状態復元（[`Hydrate::from_hydration_attrs`]）→
/// イベント配線（[`wire_events`]）までの経過時間（ミリ秒）を 1 回分計測する。
///
/// `Runtime::hydrate`（TASK-11.4b・#83）マージ後は、この関数の内部実装を
/// `Runtime::hydrate` 呼び出しへ差し替える想定（ファイル冒頭 `//!` 参照）。
/// シナリオの入出力契約（戻り値: 経過ミリ秒）はその差し替えの前後で変えない。
fn run_initial_load(container: &Element) -> f64 {
    let state = AppState::new();

    let start = now_ms();

    // サーバー側責務相当: ハイドレーション属性付き Node を既定エスケープ済み
    // HTML 文字列へ描画する（`interactive::render_for_hydration` →
    // `rws_core::render`。REQ-1 契約は `rws-interactive`/`rws-core` 側で担保済み）。
    let html = rws_core::render(&rws_interactive::render_for_hydration(&state));

    // クライアント側責務相当: SSR 済み HTML を root コンテナへ反映する。
    // `render_for_hydration` の既定エスケープ済み出力のみを渡す
    // （ファイル冒頭 `//!` の不変条件、`raw_html()`/文字列組み立て不使用）。
    container.set_inner_html(&html);

    // `document.get_element_by_id` は文書全体を検索するため、`dom_update`
    // シナリオが同一 id（"interactive-root"、`interactive::render_with_root_attrs`
    // 固定値）で body へ残置する要素と衝突しうる（wasm-bindgen-test はテスト間で
    // DOM をリセットしない）。`container` 配下に限定して探索することで、
    // テスト実行順序に依存しない自己完結な計測にする。
    let root = container
        .first_element_child()
        .expect("render_for_hydration output must contain #interactive-root as a child");

    // `data-hydrate-*` 属性からの状態復元（クライアント側責務）。
    // 属性値は改ざんされうる入力のため `from_hydration_attrs` は Result を
    // 返す契約（`rws-interactive` 不変条件 3）。ハーネス自己検証のため
    // 復元失敗は panic 扱いとする（headless 環境では想定しない異常系）。
    let attrs = collect_hydrate_attrs(&root);
    let _restored =
        AppState::from_hydration_attrs(&attrs).expect("hydration attrs must round-trip");

    // イベント配線（マウント時に 1 回だけ委譲登録する契約、`events.rs` 参照）。
    // on_action はハーネス内で捨てる（本シナリオは配線完了までの時間計測が
    // 目的であり、実際の dispatch は `dom_update` シナリオが担当する）。
    wire_events(root, |_action_ref: ActionRef| {}).expect("wire_events must not fail");

    now_ms() - start
}

/// `data-hydrate-*` 属性列を要素の属性一覧から抽出する。
///
/// `web_sys::Element::attributes()`（`NamedNodeMap`）を走査する薄いヘルパ。
/// DOM 依存のため native では検証できず、本ファイル（wasm32 ゲート域）に
/// 閉じる。
fn collect_hydrate_attrs(element: &Element) -> Vec<(String, String)> {
    let attributes = element.attributes();
    let len = attributes.length();
    let mut out = Vec::with_capacity(len as usize);
    for i in 0..len {
        if let Some(attr) = attributes.item(i) {
            out.push((attr.name(), attr.value()));
        }
    }
    out
}

/// `dom_update` シナリオの 1 回分: `dispatch("increment")` → 再描画
/// （[`render_component_html`] → `set_inner_html`）までの経過時間（ミリ秒）。
///
/// `Runtime` マージ後の差し替え対象（ファイル冒頭 `//!` 参照）。
fn run_dom_update_iteration(container: &Element, state: &mut AppState) -> f64 {
    let start = now_ms();

    // WASM 境界の文字列 dispatch 契約（`rws_interactive::dispatch`）。戻り値
    // `bool` は「状態が変化したか」を表すが、本ハーネスは既知アクション
    // （"increment"）のみを送るため無視してよい（安全側 no-op 契約は
    // `rws-interactive` 不変条件 4 が担保）。
    let _ = dispatch(state, "increment", "");

    let html = render_component_html(state);
    container.set_inner_html(&html);

    now_ms() - start
}

/// `initial_load` シナリオのハーネス自己検証（TASK-11.5a のスコープ）。
///
/// 性能予算（[`INITIAL_LOAD_BUDGET_MS`]）に対するアサーションは行わない
/// （TASK-11.5b・#87 のスコープ、ファイル冒頭 `//!` 参照）。ここでは
/// サンプルが 1 件以上収集でき、値が有限かつ非負であること、出力契約の
/// 1 行サマリが期待書式であることのみを検証する。
#[wasm_bindgen_test]
fn initial_load_harness_produces_finite_nonnegative_sample() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_scenario_container(&document, "perf-initial-load-root");

    let elapsed = run_initial_load(&container);

    assert!(
        elapsed.is_finite() && elapsed >= 0.0,
        "initial_load の計測値は有限かつ非負であること: {elapsed}"
    );

    let stats = summarize(&[elapsed]);
    assert_eq!(stats.samples, 1);

    let summary = format_summary_line("initial_load", stats);
    web_sys::console::log_1(&summary.clone().into());

    assert!(
        summary.starts_with("perf-browser: metric=initial_load samples=1 "),
        "出力契約（1 行サマリ）を満たすこと: {summary}"
    );
}

/// `dom_update` シナリオのハーネス自己検証（TASK-11.5a のスコープ）。
///
/// [`DOM_UPDATE_SAMPLES`] 回のループが有界であること・mean/p95/max がいずれも
/// 有限かつ非負であること・出力契約の書式を満たすことのみを検証する
/// （フレーム予算 [`FRAME_BUDGET_MS`] に対するアサーションは TASK-11.5b・#87
/// のスコープ）。
#[wasm_bindgen_test]
fn dom_update_harness_produces_bounded_finite_samples() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_scenario_container(&document, "perf-dom-update-root");

    // 初回描画（ループ計測対象に含めない。ループは dispatch→再描画のみを計測する）。
    let mut state = AppState::new();
    container.set_inner_html(&render_component_html(&state));

    let mut samples = Vec::with_capacity(DOM_UPDATE_SAMPLES);
    for _ in 0..DOM_UPDATE_SAMPLES {
        samples.push(run_dom_update_iteration(&container, &mut state));
    }

    assert_eq!(samples.len(), DOM_UPDATE_SAMPLES);
    assert!(
        samples.iter().all(|s| s.is_finite() && *s >= 0.0),
        "dom_update の各サンプルは有限かつ非負であること: {samples:?}"
    );

    let stats = summarize(&samples);
    assert_eq!(stats.samples, DOM_UPDATE_SAMPLES);

    let summary = format_summary_line("dom_update", stats);
    web_sys::console::log_1(&summary.clone().into());

    assert!(
        summary.starts_with(&format!(
            "perf-browser: metric=dom_update samples={DOM_UPDATE_SAMPLES} "
        )),
        "出力契約（1 行サマリ）を満たすこと: {summary}"
    );
}

/// native（rlib）で検証できる純粋関数のみを対象にした回帰テスト群。
///
/// `summarize`/`format_summary_line` は DOM/`wasm-bindgen` に依存しないが、
/// ファイル全体が `#![cfg(target_arch = "wasm32")]` でゲートされているため、
/// これらの単体テストも wasm32 ターゲット上でのみ実行される
/// （`wasm-pack test` 経由。native `cargo test --workspace` はこのファイル自体を
/// コンパイル対象から除外するため、既存 CI の test/forbid-unsafe ジョブに
/// 影響しない）。
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
}

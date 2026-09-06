//! `fandhe_frontend_wasm_full::headless_timer`（イシュー #836、親トラッキング
//! #520）の統合レベル native テスト。
//!
//! `wasm-full/src/headless_timer.rs` 内のインラインテストは本モジュール単体の
//! 判定関数（[`is_timer_action_trigger`]/[`action_from_trigger`]/
//! [`clamp_interval_ms`]/[`timer_from_display_attrs`]）を対象にしている。
//! 本ファイルはその先、`fandhe_frontend_wasm_full::hydration::restore_state`
//! を経由した Timer の hydration 復元 → dispatch → 状態遷移という統合経路を
//! 検証し、`fandhe-frontend-headless-ui` の `Timer`/`TimerAction` 実装との
//! ドリフトを固定する。
//!
//! 実 DOM 経由の検証（ActionTrigger クリック → dispatch → `data-state`/
//! item-value 反映・`setInterval` による実 tick 駆動）は
//! `wasm-full/tests/headless_timer_browser.rs` が担当する。

use fandhe_frontend_headless_ui::timer::Timer;
use fandhe_frontend_interactive::{dispatch, Hydrate, HydrateError};
use fandhe_frontend_wasm_full::headless_timer::{
    action_from_trigger, clamp_interval_ms, formatted_segments, is_timer_action_trigger,
    timer_from_display_attrs, MIN_INTERVAL_MS,
};
use fandhe_frontend_wasm_full::hydration::restore_state;

// --- hydration ラウンドトリップ ------------------------------------------

#[test]
fn restore_state_roundtrips_timer_after_ticks() {
    let mut timer = Timer::countdown(5000, 250);
    assert!(dispatch(&mut timer, "timer:start", ""));
    assert!(dispatch(&mut timer, "timer:tick", "1200"));

    let attrs = timer.hydration_attrs();
    let restored: Timer = restore_state(&attrs).expect("roundtrip should succeed");
    assert_eq!(restored, timer);
}

#[test]
fn restore_state_rejects_tampered_phase_without_panicking() {
    let attrs = vec![
        ("data-hydrate-phase".to_string(), "flying".to_string()),
        ("data-hydrate-elapsed".to_string(), "0".to_string()),
        ("data-hydrate-countdown".to_string(), "false".to_string()),
        ("data-hydrate-start-ms".to_string(), "0".to_string()),
        ("data-hydrate-target-ms".to_string(), "0".to_string()),
        ("data-hydrate-interval-ms".to_string(), "1000".to_string()),
    ];
    let result = restore_state::<Timer>(&attrs);
    assert!(matches!(result, Err(HydrateError::InvalidValue { .. })));
}

#[test]
fn restore_state_rejects_missing_attrs_without_panicking() {
    let result = restore_state::<Timer>(&[]);
    assert!(matches!(result, Err(HydrateError::MissingAttr(_))));
}

// --- 判定関数 → dispatch → 状態遷移の統合経路 ---------------------------

#[test]
fn start_action_dispatch_transitions_timer_to_running() {
    let mut timer = Timer::count_up(0, 1000);
    let action = action_from_trigger(Some("start")).expect("start should be allowlisted");
    assert!(dispatch(&mut timer, action, ""));
    assert_eq!(timer.phase().as_str(), "running");
}

#[test]
fn pause_then_resume_round_trip_via_allowlisted_actions() {
    let mut timer = Timer::count_up(0, 1000);
    dispatch(&mut timer, action_from_trigger(Some("start")).unwrap(), "");
    assert!(dispatch(
        &mut timer,
        action_from_trigger(Some("pause")).unwrap(),
        ""
    ));
    assert_eq!(timer.phase().as_str(), "paused");
    assert!(dispatch(
        &mut timer,
        action_from_trigger(Some("resume")).unwrap(),
        ""
    ));
    assert_eq!(timer.phase().as_str(), "running");
}

#[test]
fn reset_action_returns_timer_to_idle() {
    let mut timer = Timer::countdown(1000, 100);
    dispatch(&mut timer, "timer:start", "");
    dispatch(&mut timer, "timer:tick", "500");
    assert!(dispatch(
        &mut timer,
        action_from_trigger(Some("reset")).unwrap(),
        ""
    ));
    assert_eq!(timer.phase().as_str(), "idle");
    assert_eq!(timer.elapsed_ms(), 0);
}

// --- timer_from_display_attrs ↔ 実際の headless-ui root 出力の整合 -------

#[test]
fn timer_from_display_attrs_matches_headless_ui_root_output() {
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::timer::{root, TimerPhase};

    let html = render(&root(
        true,
        5000,
        0,
        250,
        1200,
        TimerPhase::Running,
        Vec::new(),
        Vec::new(),
    ));
    assert!(html.contains(r#"data-scope="timer""#));
    assert!(html.contains(r#"data-part="root""#));

    let reconstructed = timer_from_display_attrs(
        Some("running"),
        Some("1200"),
        true,
        Some("5000"),
        Some("0"),
        Some("250"),
    )
    .expect("attrs matching headless-ui root output should reconstruct");
    assert_eq!(reconstructed.phase().as_str(), "running");
    assert_eq!(reconstructed.elapsed_ms(), 1200);
    assert!(reconstructed.is_countdown());
    assert_eq!(reconstructed.interval_ms(), 250);
}

// --- formatted_segments と headless-ui Timer::items の整合 ---------------

#[test]
fn formatted_segments_matches_timer_items_rendering() {
    use fandhe_frontend_core::render;

    let mut timer = Timer::countdown(93_784_000, 1000);
    dispatch(&mut timer, "timer:start", "");

    let items_html: String = timer.items().iter().map(render).collect();
    for (unit, formatted) in formatted_segments(&timer) {
        let expected = format!(r#"data-type="{}""#, unit.as_str());
        assert!(items_html.contains(&expected));
        assert!(
            items_html.contains(&format!(">{formatted}<")),
            "expected formatted value {formatted} for {unit:?} in {items_html}"
        );
    }
}

// --- is_timer_action_trigger のドリフト固定 ------------------------------

#[test]
fn action_trigger_predicate_agrees_with_headless_ui_anatomy() {
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::timer::{action_trigger, TimerControl, TimerPhase};

    let html = render(&action_trigger(
        TimerControl::Start,
        TimerPhase::Idle,
        Vec::new(),
        Vec::new(),
    ));
    assert!(html.contains(r#"data-scope="timer""#));
    assert!(html.contains(r#"data-part="action-trigger""#));
    assert!(is_timer_action_trigger(
        Some("timer"),
        Some("action-trigger")
    ));
}

// --- clamp_interval_ms の下限保証（dispatch ストーム対策） ---------------

#[test]
fn clamp_interval_ms_never_returns_below_minimum() {
    for raw in [0, 1, 15, 16, 17, 1000, u64::MAX] {
        assert!(clamp_interval_ms(raw) >= MIN_INTERVAL_MS);
    }
}

// --- Runtime::wire_timer の二重描画回避契約を native 側から固定
//     （イシュー #1959） ------------------------------------------------
//
// `fandhe_frontend_headless_ui::timer::Timer` 自体は `DirtyTracked` を
// 実装していない。`Runtime<C>` は `C: DirtyTracked` を要求するため、
// `Runtime::wire_timer`（`crates/wasm-full/src/lib.rs`）が
// `apply_update_for_dirty` へ委譲する際に読む `dirty_fields()` は、
// 常に「Timer をラップするアプリ側の状態機械 `C`」が明示的に定義した
// フィールド集合であり、Timer 自身の表示属性名（`headless_timer::wiring`
// が `write_timer` で直書きする `"data-state"`/`"data-elapsed"` 等）が
// 紛れ込むことは型レベルで構造的に起こらない。本テストはこの契約を
// `TimerHost`（`Timer` をラップし、アプリ側フィールドのみ dirty へ積む
// ラッパー）で固定する。実 DOM 経由の再描画反映自体
// （`Runtime::hydrate` → click → 束縛点更新・`data-state` 二重書き込み
// なし）は `headless_timer_browser.rs` が実ブラウザで検証する。
mod wire_timer_dirty_contract {
    use fandhe_frontend_headless_ui::timer::{Timer, TimerAction};
    use fandhe_frontend_interactive::{dispatch, Component, DirtyTracked};

    /// テスト専用ラッパー。`Timer` の状態機械をそのまま委譲しつつ、
    /// アプリ側の派生フィールド（tick 回数・フェーズラベル）のみを
    /// `dirty_fields()` に積む（`headless_timer.rs::write_timer` が直書き
    /// する Timer 表示属性名を一切含まない）。
    struct TimerHost {
        timer: Timer,
        tick_count: u32,
        dirty: Vec<&'static str>,
    }

    impl TimerHost {
        fn new(timer: Timer) -> Self {
            Self {
                timer,
                tick_count: 0,
                dirty: Vec::new(),
            }
        }
    }

    enum HostAction {
        Timer(<Timer as Component>::Action),
    }

    impl Component for TimerHost {
        type Action = HostAction;

        fn update(&mut self, action: Self::Action) {
            self.dirty.clear();
            let HostAction::Timer(timer_action) = action;
            let is_tick = matches!(timer_action, TimerAction::Tick(_));
            self.timer.update(timer_action);
            // 遷移が起きた種別に応じてアプリ側フィールドのみを dirty へ積む
            // （Timer 表示属性名 "data-state"/"data-elapsed" 等は含めない、
            // 上記モジュール doc の契約）。
            if is_tick {
                self.tick_count += 1;
                self.dirty.push("tick_count");
            } else {
                self.dirty.push("phase_label");
            }
        }

        fn view(&self) -> fandhe_frontend_core::Node {
            // `Timer::view()`（最小正準ビュー、`crates/headless-ui/src/timer.rs`
            // doc 参照）をそのまま委譲する。本テストは `dirty_fields()` の
            // 内容のみを検証し、DOM 反映は対象外のため view 内容自体は
            // 重要ではない。
            self.timer.view()
        }

        fn decode_action(name: &str, payload: &str) -> Option<Self::Action> {
            Timer::decode_action(name, payload).map(HostAction::Timer)
        }
    }

    impl DirtyTracked for TimerHost {
        fn dirty_fields(&self) -> &[&'static str] {
            &self.dirty
        }
    }

    #[test]
    fn dispatch_marks_only_app_fields_dirty_never_timer_display_attrs() {
        let mut host = TimerHost::new(Timer::count_up(0, 1000));

        assert!(dispatch(&mut host, "timer:start", ""));
        assert_eq!(host.dirty_fields(), &["phase_label"]);

        assert!(dispatch(&mut host, "timer:tick", "30"));
        assert_eq!(host.dirty_fields(), &["tick_count"]);
        assert_eq!(host.tick_count, 1);

        assert!(dispatch(&mut host, "timer:pause", ""));
        assert_eq!(host.dirty_fields(), &["phase_label"]);

        assert!(dispatch(&mut host, "timer:reset", ""));
        assert_eq!(host.dirty_fields(), &["phase_label"]);

        // いずれの dirty field も Timer の DOM 表示属性名を含まない
        // （`Runtime::wire_timer` が `apply_update_for_dirty` へ渡す前提の
        // 型レベル不変条件）。
        for field in host.dirty_fields() {
            assert_ne!(*field, "data-state");
            assert_ne!(*field, "data-elapsed");
            assert_ne!(*field, "state");
            assert_ne!(*field, "elapsed");
        }
    }

    #[test]
    fn dispatch_of_unrecognized_action_returns_false_and_leaves_dirty_empty() {
        let mut host = TimerHost::new(Timer::count_up(0, 1000));
        // `update()` は 1 度も呼ばれていないため dirty は初期値の空集合。
        assert!(host.dirty_fields().is_empty());

        assert!(!dispatch(&mut host, "timer:unknown", ""));
        assert!(host.dirty_fields().is_empty());
    }
}

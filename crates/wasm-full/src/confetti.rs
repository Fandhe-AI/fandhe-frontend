//! confetti（canvas ベース）の DOM 配線層（イシュー #2533）。
//!
//! # 責務境界
//!
//! パーティクル物理演算（決定的、乱数・重力・寿命減衰）は
//! `fandhe-animation::confetti` の責務、canvas 2D 描画・
//! `requestAnimationFrame` 駆動・発火オーケストレーションは
//! `fandhe-frontend-animation::confetti::fire` の責務であり、本モジュールは
//! 以下のみを担う（3 層構成、`docs/design/motion-reference-adoption-policy.md`
//! §6）:
//!
//! 1. `root` へのクリック委譲登録（`events::wire_events`/`gesture.rs` と
//!    同じ「登録回数を定数個に抑える」方針、A04 対策）
//! 2. トリガー要素（`[data-fandhe-confetti-trigger]`）のクリックから、
//!    属性値（発火対象 canvas の `id`）を解決し `fire()` を呼ぶ
//! 3. 発火中の `AnimationLoop` ハンドルの保持（canvas id をキーに
//!    `Rc<RefCell<HashMap<..>>>` で管理し、同じ canvas への再発火で
//!    差し替える。発火完了時（`fire()` の `on_finished` コールバック）に
//!    もマップから解放し、異なる id の canvas を生成・除去し続ける画面で
//!    完了済み `ConfettiSim`/`CanvasTarget`/`canvas` 参照が無期限に蓄積
//!    しないようにする（PR #2564 codex-review P1 是正）。詳細は
//!    `wiring::handle_click`/`wiring::schedule_finished_cleanup`
//!    （本ファイル内部実装）の doc 参照
//!
//! `wasm-full` 自体は canvas 系 web-sys feature（`CanvasRenderingContext2d`/
//! `HtmlCanvasElement`）を一切追加しない（`crates/wasm-full/Cargo.toml` の
//! SignaturePad 由来コメント「canvas 系 API は一切追加しない」方針を維持。
//! `fire` は `web_sys::Element` を受け取り、canvas への cast は
//! `fandhe-frontend-animation` 側で完結する）。
//!
//! # ロケータ契約（security.md A03）
//!
//! `data-fandhe-confetti-trigger` の値（発火対象 canvas の `id`）は
//! `document.get_element_by_id` で解決したうえで、(1) `root` 配下に
//! 収まっている、(2) `data-fandhe-confetti-canvas` を持つ、の 2 条件を
//! 両方満たす要素のみを対象にする（`chart_range.rs::resolve_chart_root`
//! と同型のロケータ契約。`query_selector` へ利用者由来文字列を補間
//! しない）。いずれかを満たさない場合は何もしない（fail-closed）。
//!
//! # Runtime への統合
//!
//! `crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が
//! `Self::wire_scroll_driver` の直後で `Self::wire_confetti` を呼ぶ（feature
//! `confetti`、既定 on）。`dispatch` チャネルを持たない属性専用配線のため
//! （`Self::wire_sidebar`/`Self::wire_gesture` と同型）、
//! `Component`/`binding_table`/`keyed_list_cache` は必要としない。
//!
//! # スコープ外（実装計画 §6 の YAGNI 判断）
//!
//! `data-*` 属性経由のパーティクル数・色・duration カスタマイズは扱わない
//! （固定デフォルト値のみ、`fandhe_animation::confetti::ConfettiConfig::default()`）。

/// opt-in（著者が SSR 出力に静的に付与）: クリックで発火するトリガー要素。
/// 値は発火対象 canvas の `id`。
pub const CONFETTI_TRIGGER_ATTR: &str = "data-fandhe-confetti-trigger";
/// トリガー要素の走査セレクタ（`closest()` に渡す）。
pub const CONFETTI_TRIGGER_SELECTOR: &str = "[data-fandhe-confetti-trigger]";
/// opt-in（著者が SSR 出力に静的に付与）: confetti の発火対象であることを
/// 示す canvas 側マーカー（値なし存在属性）。`data-fandhe-confetti-trigger`
/// の値で `id` 解決した要素がこの属性を持つかどうかをロケータ契約の一部
/// として検証する（モジュール doc 参照）。
pub const CONFETTI_CANVAS_ATTR: &str = "data-fandhe-confetti-canvas";

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{CONFETTI_CANVAS_ATTR, CONFETTI_TRIGGER_ATTR, CONFETTI_TRIGGER_SELECTOR};
    // `wasm-full` は `fandhe-animation` を直接依存に持たない
    // （`fandhe-frontend-animation` の推移依存）ため、
    // `fandhe_frontend_animation::fandhe_animation`（同クレートの
    // `pub use fandhe_animation;` 再エクスポート、`lib.rs` doc 参照）経由で
    // 型へアクセスする。
    use fandhe_frontend_animation::confetti::fire;
    use fandhe_frontend_animation::fandhe_animation::confetti::ConfettiConfig;
    use fandhe_frontend_animation::raf_driver::AnimationLoop;
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event};

    /// 発火中の [`AnimationLoop`] ハンドルを canvas `id` ごとに保持する。
    /// 値は `(generation, handle)`——`generation` は
    /// [`schedule_finished_cleanup`] が仕掛ける完了時の遅延解放が、
    /// 対象キーへ後から差し替わった新しい発火を誤って削除しないための
    /// 単調増加トークン（[`handle_click`] が発火のたびに新しい値を
    /// 払い出す）。
    ///
    /// `fire()` の所有権契約（`fandhe_frontend_animation::confetti` モジュール
    /// doc 参照）どおり、ハンドルを保持し続けている間だけループが進む。
    /// 同じ canvas への再クリックは [`handle_click`] が新しいハンドルで
    /// `HashMap::insert` により差し替える——この差し替えは新しいクリック
    /// イベントのコールバックフレーム内（旧ハンドルの `step` クロージャの
    /// 実行フレームの外側）で起きるため、`AnimationLoop::drop` が旧ループを
    /// 安全に停止できる（use-after-free を踏まない）。発火完了時は
    /// [`schedule_finished_cleanup`] がマップから該当エントリを解放する
    /// （異なる id の canvas を生成・除去し続ける画面での無制限な蓄積を
    /// 防ぐ、PR #2564 codex-review P1 是正）。
    type ActiveLoops = Rc<RefCell<HashMap<String, (u64, AnimationLoop)>>>;

    /// `event.target()` を `Element` として取得する（`gesture.rs`
    /// `event_target_element` と同型）。
    fn event_target_element(event: &Event) -> Option<Element> {
        event.target()?.dyn_into::<Element>().ok()
    }

    /// `id` を `document.get_element_by_id` で解決し、`root` 配下かつ
    /// `data-fandhe-confetti-canvas` を持つ場合のみ返す（モジュール doc
    /// 「ロケータ契約」節）。
    fn resolve_confetti_canvas(root: &Element, id: &str) -> Option<Element> {
        if id.is_empty() {
            return None;
        }
        let document = root.owner_document()?;
        let found = document.get_element_by_id(id)?;
        if !root.contains(Some(&found)) {
            return None;
        }
        found.has_attribute(CONFETTI_CANVAS_ATTR).then_some(found)
    }

    /// [`handle_click`] が `fire()` の `on_finished` に渡すコールバックが
    /// 実際に呼ぶ、完了済み [`AnimationLoop`] ハンドルの遅延解放本体。
    ///
    /// `on_finished` は confetti の `step` クロージャ（対象 `AnimationLoop`
    /// 自身の rAF コールバックの呼び出しフレームの内側）から**同期的に**
    /// 呼ばれる（`fandhe_frontend_animation::confetti` モジュール doc
    /// 「`on_finished`（発火完了通知）の安全な消費方法」節）。そのため
    /// ここで直接 `active_loops.borrow_mut().remove(id)` すると、削除した
    /// `AnimationLoop`（＝現在実行中の rAF コールバック自身を保持する
    /// `Closure`）がその場で drop され、実行中の `Closure` を `call_mut`
    /// 実行中に drop する use-after-free（`raf_driver.rs`
    /// `AnimationLoop` doc が警告する構造）を踏む。これを避けるため、
    /// `crates/wasm-full/src/command.rs::schedule_composed_guard_reset`
    /// と同型の「`set_timeout` の 0ms 遅延で次のマクロタスクへ延期し、
    /// 現在の呼び出しフレームの外側で解放する」パターンを使う。
    ///
    /// `generation` は、この遅延解放が予約されてから実際に実行されるまでの
    /// 間に同じ `canvas_id` へ新しい発火（再クリック）が上書き挿入された
    /// 場合に、その新しいエントリを誤って削除しないための一致確認
    /// （[`ActiveLoops`] doc 参照）。挿入時と一致する場合のみ削除する。
    fn schedule_finished_cleanup(active_loops: ActiveLoops, canvas_id: String, generation: u64) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let cleanup = Closure::once_into_js(move || {
            let mut loops = active_loops.borrow_mut();
            if matches!(loops.get(&canvas_id), Some((gen, _)) if *gen == generation) {
                loops.remove(&canvas_id);
            }
        });
        let _ = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(cleanup.unchecked_ref(), 0);
    }

    /// `root` 配下のクリックを委譲受信し、`[data-fandhe-confetti-trigger]`
    /// 一致時に対象 canvas を解決して [`fire`] を呼ぶ。
    fn handle_click(
        root: &Element,
        event: &Event,
        active_loops: &ActiveLoops,
        next_generation: &Rc<Cell<u64>>,
    ) {
        let Some(target) = event_target_element(event) else {
            return;
        };
        let Ok(Some(trigger)) = target.closest(CONFETTI_TRIGGER_SELECTOR) else {
            return;
        };
        if !root.contains(Some(&trigger)) {
            return;
        }
        let Some(canvas_id) = trigger.get_attribute(CONFETTI_TRIGGER_ATTR) else {
            return;
        };
        let Some(canvas) = resolve_confetti_canvas(root, &canvas_id) else {
            return;
        };

        // 発火のたびに新しい世代を払い出す（[`ActiveLoops`] doc 参照）。
        let generation = next_generation.get();
        next_generation.set(generation.wrapping_add(1));

        let active_loops_for_finish = active_loops.clone();
        let canvas_id_for_finish = canvas_id.clone();
        let on_finished = move || {
            schedule_finished_cleanup(
                active_loops_for_finish.clone(),
                canvas_id_for_finish.clone(),
                generation,
            );
        };

        // `fire` の `Err`（canvas 要素でない・2D コンテキスト取得失敗）は
        // 著者マークアップの誤りとして握り潰し、他の配線へ波及させない
        // （`Target::write` の黙殺方針、`chart_range.rs` の click ハンドラ群
        // と同じく `Result` を無視する）。`Ok(None)`（発火抑制、
        // reduced-motion 等）はループ差し替えを行わない（この場合
        // `on_finished` は一度も呼ばれず破棄される）。
        if let Ok(Some(loop_handle)) = fire(&canvas, ConfettiConfig::default(), on_finished) {
            active_loops
                .borrow_mut()
                .insert(canvas_id, (generation, loop_handle));
        }
    }

    /// `root` へ confetti トリガーのクリック委譲を 1 リスナー登録する
    /// （[`crate::lib::Runtime::mount`]/[`crate::lib::Runtime::hydrate`]
    /// から呼ばれる）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback_and_bool` の失敗を伝播する。
    pub fn wire_confetti(root: Element) -> Result<(), JsValue> {
        let active_loops: ActiveLoops = Rc::new(RefCell::new(HashMap::new()));
        let next_generation: Rc<Cell<u64>> = Rc::new(Cell::new(0));
        let click_root = root.clone();
        let click_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_click(&click_root, &event, &active_loops, &next_generation);
        });
        // `gesture.rs::wire_gesture` と同じく capture フェーズで登録する
        // （子孫が `stopPropagation()` を呼んでも root のリスナーは
        // 先に実行される）。
        root.add_event_listener_with_callback_and_bool(
            "click",
            click_closure.as_ref().unchecked_ref(),
            true,
        )?;
        click_closure.forget();
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_confetti;

#[cfg(test)]
mod tests {
    use super::{CONFETTI_CANVAS_ATTR, CONFETTI_TRIGGER_ATTR, CONFETTI_TRIGGER_SELECTOR};

    #[test]
    fn constants_are_stable_strings() {
        assert_eq!(CONFETTI_TRIGGER_ATTR, "data-fandhe-confetti-trigger");
        assert_eq!(CONFETTI_TRIGGER_SELECTOR, "[data-fandhe-confetti-trigger]");
        assert_eq!(CONFETTI_CANVAS_ATTR, "data-fandhe-confetti-canvas");
    }
}

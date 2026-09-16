//! stat の数値カウントアップ（Motion+ AnimateNumber 相当）の DOM 配線層
//! （イシュー #2539、親 #2528）。
//!
//! # 責務境界
//!
//! 書式保存の数値解析・補間・`textContent` 書き込み
//! （[`fandhe_frontend_animation::count_up::NumberText`]/
//! [`fandhe_frontend_animation::count_up::start`]）は `fandhe-frontend-animation`
//! の責務であり、本モジュールは以下のみを担う（3 層構成、`docs/design/
//! motion-reference-adoption-policy.md` §6）:
//!
//! 1. opt-in 要素（[`COUNT_UP_ATTR`]）の解決・初期テキストの解析
//! 2. トリガー（マウント時 / in-view）の判定と発火
//! 3. `MutationObserver` による外部更新（アプリの `set_text` 等）の検知・
//!    再補間の起動
//! 4. `prefers-reduced-motion: reduce` 時は配線自体を行わない（`magnetic`
//!    と同じ「wire 時に検出し、reduced なら配線しない」方針）
//!
//! # Runtime への統合
//!
//! `crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が
//! `Self::wire_magnetic` の直後で `Self::wire_count_up` を呼ぶ（feature
//! `count-up`、既定 on）。`dispatch` チャネルを持たない属性専用配線のため
//! （`Self::wire_magnetic`/`Self::wire_hold_to_confirm` と同型）、
//! `Component`/`binding_table`/`keyed_list_cache` は必要としない。
//!
//! # セキュリティ不変条件（REQ-1・security.md A03）
//!
//! DOM への書き込みは `fandhe_frontend_animation::count_up` 経由の
//! `CharacterData::set_data`/`set_text_content` のみ（HTML 解釈なし）。
//! 属性値はすべて防御的パース、セレクタへ動的文字列を混ぜない。
//!
//! # 読み書きの対象（stat の子要素を保持する）
//!
//! `stat::value_text` は `value_unit`/`up_indicator`/`down_indicator` を
//! 数値テキストと並べて子に持つ構成を公開契約とするため、初期テキストの
//! 解析・外部更新の読み取りは要素全体の `textContent` ではなく
//! [`fandhe_frontend_animation::count_up::read_value_text`]（数値テキスト
//! ノードのみ）で行い、書き込み側（`start`/`write_final`）と対象を揃える
//! （PR #2580 codex-review P1 指摘: 開始直後に単位 `<span>` や矢印が削除
//! されていた回帰の是正）。

/// opt-in（著者が SSR 出力に静的に付与）: カウントアップを有効化する
/// 要素マーカー（値なし存在属性）。
pub const COUNT_UP_ATTR: &str = "data-fandhe-count-up";
/// 候補走査セレクタ。
pub const COUNT_UP_SELECTOR: &str = "[data-fandhe-count-up]";
/// opt-in（任意）: 補間時間（ミリ秒）を著者が上書きする属性。
pub const COUNT_UP_DURATION_MS_ATTR: &str = "data-fandhe-count-up-duration-ms";
/// opt-in（任意）: 開始トリガーを指定する属性。
pub const COUNT_UP_TRIGGER_ATTR: &str = "data-fandhe-count-up-trigger";
/// [`COUNT_UP_TRIGGER_ATTR`] の値: ビューポート進入時に開始する。
pub const COUNT_UP_TRIGGER_IN_VIEW: &str = "in-view";

/// 開始トリガー（DOM 非依存、native `cargo test` で検証可能）。`wasm32`
/// 専用の `mod wiring` からのみ実消費されるため、native ビルド（`mod
/// wiring` 自体がコンパイル対象外）で `dead_code` 警告にならないよう
/// `pub`（`hold_to_confirm::parse_hold_duration_ms` と同じ理由）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// マウント/ハイドレート時に即座に開始する（既定）。
    Immediate,
    /// ビューポートへ初めて進入したときに開始する。
    InView,
}

/// [`COUNT_UP_TRIGGER_ATTR`] の値からトリガーを判定する。未知の値・
/// 未設定はすべて [`Trigger::Immediate`] へ fail-closed に倒す
/// （`in_view::in_view_once_from_attr` と同じ流儀）。
pub fn trigger_from_attr(value: Option<&str>) -> Trigger {
    if value == Some(COUNT_UP_TRIGGER_IN_VIEW) {
        Trigger::InView
    } else {
        Trigger::Immediate
    }
}

/// [`COUNT_UP_DURATION_MS_ATTR`] の値をパースする。パース失敗・非正値は
/// [`fandhe_frontend_animation::count_up::DEFAULT_COUNT_UP_DURATION_MS`]
/// へフォールバックする（`hold_to_confirm::parse_hold_duration_ms` と同型
/// の防御的パース）。
pub fn parse_count_up_duration_ms(attr_value: Option<&str>) -> f64 {
    attr_value
        .and_then(|value| value.parse::<f64>().ok())
        // `f64::from_str` は `"inf"`/`"infinity"` を正の無限大として受理する
        // ため、`> 0.0` だけでは無限大を通してしまう（`inf > 0.0` は
        // `true`）。`is_finite()` を併せて要求し、`count_up_progress` の
        // `elapsed_s / duration_s` が常に 0 のまま進捗が 1.0 に到達しない
        // 状態（PR #2580 codex-review P1 指摘）を防ぐ。
        .filter(|ms| ms.is_finite() && *ms > 0.0)
        .unwrap_or(fandhe_frontend_animation::count_up::DEFAULT_COUNT_UP_DURATION_MS)
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        parse_count_up_duration_ms, trigger_from_attr, Trigger, COUNT_UP_DURATION_MS_ATTR,
        COUNT_UP_SELECTOR, COUNT_UP_TRIGGER_ATTR,
    };
    use fandhe_frontend_animation::count_up::{self, NumberText};
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{
        Element, HtmlElement, IntersectionObserver, IntersectionObserverEntry, MutationObserver,
        MutationObserverInit,
    };

    /// `root` 配下の `[data-fandhe-count-up]` 要素（複数可）を出現順に集める
    /// （`hold_to_confirm.rs::collect_candidates` と同型、`query_selector_all`
    /// の失敗は空 `Vec` として扱う）。
    fn collect_candidates(root: &Element) -> Vec<HtmlElement> {
        let Ok(node_list) = root.query_selector_all(COUNT_UP_SELECTOR) else {
            return Vec::new();
        };
        let len = node_list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            if let Some(node) = node_list.get(i) {
                if let Ok(el) = node.dyn_into::<HtmlElement>() {
                    out.push(el);
                }
            }
        }
        out
    }

    /// `window.IntersectionObserver` の機能検出（`in_view.rs::
    /// supports_intersection_observer` と同型）。
    fn supports_intersection_observer() -> bool {
        let Some(window) = web_sys::window() else {
            return false;
        };
        js_sys::Reflect::get(&window, &JsValue::from_str("IntersectionObserver"))
            .map(|value| value.is_function())
            .unwrap_or(false)
    }

    /// 現在進行中の [`count_up::CountUp`] ハンドル（`Option` への上書きで
    /// 旧ループを停止する、`hold_to_confirm.rs::HoldSession::loop_handle`
    /// と同型）。
    type ActiveCountUp = Rc<RefCell<Option<count_up::CountUp>>>;

    /// in-view トリガーが未発火の間に [`wire_mutation_observer`] が検知した
    /// 最新の目標値（書式・数値）。`IntersectionObserver` が発火した時点で
    /// これを読み、配線時点で固定した古い目標値を使わないようにする
    /// （PR #2580 codex-review P1・Bugbot Medium 指摘: 画面外での外部更新が
    /// 進入時に古い値で上書きされていた回帰）。`None` は「待機中に非数値
    /// （"N/A" 等）へ更新済みで、カウントアップすべき数値目標が存在しない」
    /// ことを表す（PR #2580 codex-review P1 再指摘: 非数値更新後も pending
    /// が古い数値のまま残り、進入時にそれで上書きしていた回帰の是正）。
    type PendingTarget = Rc<RefCell<Option<(NumberText, f64)>>>;

    /// 直近に DOM へ書き込んだ数値（`count_up::start`/`write_final` が更新
    /// する）。外部更新時の再補間の開始値（「現在表示中の値」）として
    /// 使う。表示文字列の再解析で求めると、桁区切り `.` 書式の途中値
    /// （"1.234.567" の途中 "123.456"）を小数と誤解釈して急落する
    /// （PR #2580 codex-review P1 指摘）。`None` は「直近の表示が非数値
    /// （"N/A" 等）で開始値が存在しない」ことを表す。
    type LastValue = Rc<Cell<Option<f64>>>;

    /// `element` へ `[from, to]` 区間の補間を起動し、`active` を差し替える。
    #[allow(clippy::too_many_arguments)]
    fn start_count_up(
        element: &HtmlElement,
        format: NumberText,
        from: f64,
        to: f64,
        duration_ms: f64,
        last_value: LastValue,
        active: &ActiveCountUp,
        self_write_count: &Rc<Cell<u32>>,
    ) {
        let handle = count_up::start(
            element.clone(),
            format,
            from,
            to,
            duration_ms,
            last_value,
            Rc::clone(self_write_count),
        );
        *active.borrow_mut() = handle;
    }

    /// `element` へ `[data-fandhe-count-up]` 候補 1 件分の配線を行う。
    /// 初期テキストが [`NumberText::parse`] できない場合は何もしない
    /// （数値以外のテキストは変更しない、fail-safe）。
    fn wire_candidate(element: &HtmlElement) {
        let Some(initial) = count_up::read_value_text(element) else {
            return;
        };
        let Some(parsed) = NumberText::parse(&initial) else {
            return;
        };
        let to = parsed.value();
        let duration_ms =
            parse_count_up_duration_ms(element.get_attribute(COUNT_UP_DURATION_MS_ATTR).as_deref());
        let trigger = trigger_from_attr(element.get_attribute(COUNT_UP_TRIGGER_ATTR).as_deref());

        let last_value: LastValue = Rc::new(Cell::new(Some(to)));
        let active: ActiveCountUp = Rc::new(RefCell::new(None));
        // 自己書き込み検知カウンタ（`fandhe_frontend_animation::count_up::
        // TextTarget::write`/`write_final` が書き込みのたびにインクリメント
        // し、`wire_mutation_observer` が `MutationRecord` 件数と突き合わせて
        // 消費する）。直前に書いた文字列との一致だけで自己書き込みを
        // 判定すると、外部更新がたまたま同じ文字列を書いた場合（例:
        // in-view 待機中に開始値と同じ文字列へ外部更新された場合）に誤って
        // 無視してしまう（PR #2580 codex-review P1・Bugbot Medium 指摘の
        // 是正）。さらに真偽値 1 個の `self_write` フラグでは、自己書き込み
        // と外部更新が同じ同期処理内で両方発生し 1 回のバッチとして通知
        // された場合に外部更新側を取りこぼす（PR #2580 レビュー是正・
        // codex-review P1 指摘）。`count_up::has_external_mutation` doc
        // 参照。
        let self_write_count: Rc<Cell<u32>> = Rc::new(Cell::new(0));
        // 補間が実際に開始済みか。`Immediate` は配線時点で true、`InView`
        // は `IntersectionObserver` 発火時に true へ切り替わる。false の間
        // に [`wire_mutation_observer`] が外部更新を検知しても
        // `start_count_up` は呼ばず `pending` を更新するのみに留める
        // （画面外でアニメーションが始まってしまうのを防ぐ）。
        let started = Rc::new(Cell::new(matches!(trigger, Trigger::Immediate)));
        let pending: PendingTarget = Rc::new(RefCell::new(Some((parsed.clone(), to))));

        // `MutationObserver` は登録より前の書き込みを検知できないため、
        // 初期書き込み（`write_final`/`start_count_up`）より必ず先に登録
        // する。`MutationObserverInit::characterData` はコールバックが
        // マイクロタスクとして実行される仕様上、登録直後の同期的な初期
        // 書き込みも正しく捕捉・消費できる（[`wire_mutation_observer`]
        // doc「自己書き込み回数カウンタ」節と同じ前提）。逆順（初期書き込み
        // → 登録）だと、その書き込みが立てた `self_write_count` を消費する
        // 購読者が存在せず残留し、次に来る最初の実外部更新を誤って自己
        // 書き込みとして無視してしまう（PR #2580 codex-review P1 再指摘の
        // 回帰原因）。
        wire_mutation_observer(
            element,
            duration_ms,
            Rc::clone(&last_value),
            Rc::clone(&active),
            Rc::clone(&pending),
            Rc::clone(&started),
            Rc::clone(&self_write_count),
        );

        match trigger {
            Trigger::Immediate => {
                start_count_up(
                    element,
                    parsed,
                    0.0,
                    to,
                    duration_ms,
                    Rc::clone(&last_value),
                    &active,
                    &self_write_count,
                );
            }
            Trigger::InView => {
                count_up::write_final(element, &parsed, 0.0, &last_value, &self_write_count);
                if supports_intersection_observer() {
                    wire_in_view_trigger(
                        element,
                        Rc::clone(&pending),
                        duration_ms,
                        &last_value,
                        &active,
                        &started,
                        &self_write_count,
                    );
                } else {
                    // 非対応環境ではプログレッシブエンハンスメントとして
                    // 即座に開始する（`in_view.rs` の同種フォールバックと
                    // 同じ方針）。
                    started.set(true);
                    start_count_up(
                        element,
                        parsed,
                        0.0,
                        to,
                        duration_ms,
                        Rc::clone(&last_value),
                        &active,
                        &self_write_count,
                    );
                }
            }
        }
    }

    /// 要素専用の `IntersectionObserver` を張り、初回 `isIntersecting` で
    /// `disconnect()` してから補間を開始する（計画 §2.3「要素専用の
    /// IntersectionObserver」、`in_view.rs` の共有 observer とは異なり
    /// 動的追加要素の追随は行わない既知の制約）。
    fn wire_in_view_trigger(
        element: &HtmlElement,
        pending: PendingTarget,
        duration_ms: f64,
        last_value: &LastValue,
        active: &ActiveCountUp,
        started: &Rc<Cell<bool>>,
        self_write_count: &Rc<Cell<u32>>,
    ) {
        let element_for_callback = element.clone();
        let last_value_for_callback = Rc::clone(last_value);
        let active_for_callback = Rc::clone(active);
        let started_for_callback = Rc::clone(started);
        let self_write_count_for_callback = Rc::clone(self_write_count);
        let callback = Closure::<dyn FnMut(js_sys::Array, IntersectionObserver)>::new(
            move |entries: js_sys::Array, observer: IntersectionObserver| {
                let entered = entries.iter().any(|entry| {
                    entry
                        .dyn_into::<IntersectionObserverEntry>()
                        .is_ok_and(|entry| entry.is_intersecting())
                });
                if entered {
                    observer.disconnect();
                    started_for_callback.set(true);
                    // 画面外で待機している間に `wire_mutation_observer` が
                    // 更新した最新の目標値を読む（配線時点で固定した古い
                    // 値ではない）。`None`（待機中に非数値へ更新済み）の
                    // 場合は現在の表示（既に非数値のテキスト）をそのまま
                    // 保ち、カウントアップは起動しない。
                    let target = pending.borrow().clone();
                    if let Some((format, to)) = target {
                        start_count_up(
                            &element_for_callback,
                            format,
                            0.0,
                            to,
                            duration_ms,
                            Rc::clone(&last_value_for_callback),
                            &active_for_callback,
                            &self_write_count_for_callback,
                        );
                    }
                }
            },
        );
        let Ok(observer) = IntersectionObserver::new(callback.as_ref().unchecked_ref()) else {
            callback.forget();
            return;
        };
        callback.forget();
        observer.observe(element);
    }

    /// `element` 自身のテキスト変更（`characterData`/`childList`、
    /// `subtree: true`）を監視し、自己書き込み以外（アプリの `set_text`
    /// 等の外部更新）を検知したら現在表示中の値 → 新しい値へ再補間する。
    ///
    /// 自己書き込みの除外は自己書き込み回数カウンタ（[`count_up::
    /// TextTarget::write`]/[`count_up::write_final`] が書き込みのたびに
    /// インクリメントする）と、このコールバックが受け取ったバッチ内
    /// `MutationRecord` 件数を突き合わせて判定する
    /// （[`count_up::has_external_mutation`]）。`MutationObserver` は
    /// 同期処理が終わった後に変更をまとめて 1 回のコールバックで通知
    /// するため、自己書き込みと外部更新が同じ同期処理内で両方発生する
    /// ことがある。真偽値 1 個（「直前に自分が書いた文字列
    /// と現在の `textContent` が一致するか」、あるいは
    /// 単純な `self_write` フラグ）だけで通知全体を除外すると、この場合に
    /// 外部更新を取りこぼす（PR #2580 codex-review P1・Bugbot Medium
    /// 指摘、およびレビュー是正・codex-review P1 再指摘）。
    fn wire_mutation_observer(
        element: &HtmlElement,
        duration_ms: f64,
        last_value: LastValue,
        active: ActiveCountUp,
        pending: PendingTarget,
        started: Rc<Cell<bool>>,
        self_write_count: Rc<Cell<u32>>,
    ) {
        let element_for_callback = element.clone();
        let callback = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::new(
            move |records: js_sys::Array, _observer: MutationObserver| {
                let self_writes = self_write_count.replace(0);
                if !count_up::has_external_mutation(records.length(), self_writes) {
                    return;
                }
                let current = count_up::read_value_text(&element_for_callback);
                let Some(new_parsed) = current.as_deref().and_then(NumberText::parse) else {
                    // 数値として解析できない外部更新（例: "N/A"・空文字）。
                    // 進行中の補間を止め、古い数値で上書きし続けない
                    // （PR #2580 codex-review P1 指摘）。待機中の目標値
                    // （`pending`）も無効化する: 無効化しないと、この後
                    // 画面内へ進入した際に `wire_in_view_trigger` が古い
                    // 数値目標で上書きしてしまう（PR #2580 codex-review P1
                    // 再指摘）。開始値（`LastValue`）も無効化し、次の数値
                    // 更新は補間せず新しい値をそのまま表示する。
                    *active.borrow_mut() = None;
                    *pending.borrow_mut() = None;
                    last_value.set(None);
                    return;
                };
                let to = new_parsed.value();
                if !started.get() {
                    // まだ画面内へ進入しておらず（in-view 待機中）実際の
                    // アニメーションは開始しない。次に進入したときの目標値
                    // だけを更新する（PR #2580 Bugbot Medium 指摘）。
                    // ここで開始値 (0) を `write_final` で即座に表示する
                    // （PR #2580 codex-review P1 再指摘: 外部更新後の最終値
                    // をそのまま DOM に残すと、後で画面内へ進入した際に
                    // `wire_in_view_trigger` の `start_count_up` が 0 から
                    // 書き始め、最終値 → 0 のちらつきが再発する。初回配線
                    // 時の `Trigger::InView` 分岐と同じ 0 起点で揃える）。
                    // `write_final` はこの書き込み自体を `self_write_count`
                    // へ記録する（次回のバッチで正しく差し引かれる）ため、
                    // 待機中に受理した更新の値へ再び外部更新された際も
                    // 正しく外部更新として検知できる。
                    count_up::write_final(
                        &element_for_callback,
                        &new_parsed,
                        0.0,
                        &last_value,
                        &self_write_count,
                    );
                    *pending.borrow_mut() = Some((new_parsed, to));
                    return;
                }
                // 「現在表示中の値」= 直近に書き込んだ数値（`LastValue`
                // doc 参照。表示文字列は再解析しない）。
                let from = last_value.get().unwrap_or(to);
                start_count_up(
                    &element_for_callback,
                    new_parsed,
                    from,
                    to,
                    duration_ms,
                    Rc::clone(&last_value),
                    &active,
                    &self_write_count,
                );
            },
        );
        let Ok(mutation_observer) = MutationObserver::new(callback.as_ref().unchecked_ref()) else {
            callback.forget();
            return;
        };
        let init = MutationObserverInit::new();
        init.set_child_list(true);
        init.set_character_data(true);
        init.set_subtree(true);
        let _ = mutation_observer.observe_with_options(element, &init);
        callback.forget();
    }

    /// `root` 配下の `[data-fandhe-count-up]` 要素へ配線をマウント時に 1 回
    /// だけ登録する（[`crate::lib::Runtime::wire_count_up`] から呼ばれる）。
    /// `prefers-reduced-motion: reduce` の場合は何もしない（モジュール doc
    /// 「Reduced motion」節参照。SSR の最終値がそのまま残る）。
    ///
    /// 動的に追加された要素への追随は行わない（既知の制約、
    /// `hold_to_confirm.rs` 等と同型）。
    ///
    /// # Errors
    ///
    /// 本関数自体は失敗しない（内部の DOM 操作はすべて fail-closed に
    /// 無視する）が、他の `wire_*` 関数と統一したシグネチャのため
    /// `Result` を返す。
    pub fn wire_count_up(root: &Element) -> Result<(), JsValue> {
        if fandhe_frontend_animation::reduced_motion::prefers_reduced_motion() {
            return Ok(());
        }
        for element in collect_candidates(root) {
            wire_candidate(&element);
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_count_up;

#[cfg(test)]
mod tests {
    use super::{parse_count_up_duration_ms, trigger_from_attr, Trigger, COUNT_UP_TRIGGER_IN_VIEW};

    #[test]
    fn trigger_defaults_to_immediate() {
        assert_eq!(trigger_from_attr(None), Trigger::Immediate);
        assert_eq!(trigger_from_attr(Some("bogus")), Trigger::Immediate);
    }

    #[test]
    fn trigger_recognizes_in_view() {
        assert_eq!(
            trigger_from_attr(Some(COUNT_UP_TRIGGER_IN_VIEW)),
            Trigger::InView
        );
    }

    #[test]
    fn duration_parses_positive_value() {
        assert_eq!(parse_count_up_duration_ms(Some("500")), 500.0);
    }

    #[test]
    fn duration_falls_back_on_invalid_input() {
        assert_eq!(
            parse_count_up_duration_ms(Some("-1")),
            fandhe_frontend_animation::count_up::DEFAULT_COUNT_UP_DURATION_MS
        );
        assert_eq!(
            parse_count_up_duration_ms(Some("abc")),
            fandhe_frontend_animation::count_up::DEFAULT_COUNT_UP_DURATION_MS
        );
        assert_eq!(
            parse_count_up_duration_ms(None),
            fandhe_frontend_animation::count_up::DEFAULT_COUNT_UP_DURATION_MS
        );
    }

    /// PR #2580 codex-review P1 指摘: `f64::from_str` が `"inf"`/`"1e309"`
    /// を無限大として受理するため `> 0.0` だけでは通ってしまい、進捗が
    /// 常に 0 のまま終了しない回帰。
    #[test]
    fn duration_falls_back_on_non_finite_input() {
        assert_eq!(
            parse_count_up_duration_ms(Some("inf")),
            fandhe_frontend_animation::count_up::DEFAULT_COUNT_UP_DURATION_MS
        );
        assert_eq!(
            parse_count_up_duration_ms(Some("1e309")),
            fandhe_frontend_animation::count_up::DEFAULT_COUNT_UP_DURATION_MS
        );
    }

    #[test]
    fn constants_are_stable_strings() {
        assert_eq!(super::COUNT_UP_ATTR, "data-fandhe-count-up");
        assert_eq!(super::COUNT_UP_SELECTOR, "[data-fandhe-count-up]");
        assert_eq!(
            super::COUNT_UP_DURATION_MS_ATTR,
            "data-fandhe-count-up-duration-ms"
        );
        assert_eq!(super::COUNT_UP_TRIGGER_ATTR, "data-fandhe-count-up-trigger");
        assert_eq!(super::COUNT_UP_TRIGGER_IN_VIEW, "in-view");
    }
}

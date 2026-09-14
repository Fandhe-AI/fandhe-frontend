//! `fandhe_frontend_animation::scroll_driver`（イシュー #2521、親 `#2499`）の
//! 実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! native テスト（`crates/frontend-animation/src/scroll_driver.rs` の
//! `compute_progress_tests`/`native_no_panic_tests`）は純粋計算・native
//! no-panic を検証済みである。本ファイルは以下を実ブラウザ（headless
//! Chromium）上で検証する。
//!
//! 1. [`ScrollDriver::start`]/[`ScrollDriver::mark_dirty`] が同一フレーム内
//!    の複数呼び出しを 1 回の `recompute` へ coalesce すること
//! 2. [`update_element_progress`] が実 DOM 要素に対し、実スクロールに応じて
//!    単調に変化する custom property を書き込むこと

#![cfg(target_arch = "wasm32")]

use std::cell::RefCell;
use std::rc::Rc;

use fandhe_frontend_animation::dom_target::DomTarget;
use fandhe_frontend_animation::scroll_driver::{
    update_element_progress, update_element_progress_for_range, ProgressRange, ScrollDriver,
    SCROLL_PROGRESS_PROPERTY,
};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// 1 rAF フレーム分だけ待つ（`ScrollDriver` の dirty-flag 反映・
/// スクロール後の測定タイミング確認用）。
async fn wait_one_frame() {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist");
        let callback = Closure::once_into_js(move || {
            let _ = resolve.call0(&JsValue::NULL);
        });
        window
            .request_animation_frame(callback.unchecked_ref())
            .expect("requestAnimationFrame must not fail in test environment");
    });
    JsFuture::from(promise)
        .await
        .expect("requestAnimationFrame promise must not reject");
}

#[wasm_bindgen_test]
async fn scroll_driver_coalesces_multiple_mark_dirty_into_one_recompute_per_frame() {
    let call_count = Rc::new(RefCell::new(0_u32));
    let call_count_for_recompute = call_count.clone();
    let driver = ScrollDriver::start(move || {
        *call_count_for_recompute.borrow_mut() += 1;
    });

    // 開始直後の 1 フレーム目は無条件で 1 回呼ばれる契約（初期スクロール
    // 位置の反映）。まずこの 1 回を消化してから同一フレーム内の複数
    // `mark_dirty()` 呼び出しの coalesce を検証する。
    wait_one_frame().await;
    assert_eq!(
        *call_count.borrow(),
        1,
        "開始直後 1 フレーム目は無条件で 1 回呼ばれるはず"
    );

    // 同一フレーム内で複数回 mark_dirty しても、次フレームの recompute は
    // 1 回だけであることを確認する。
    driver.mark_dirty();
    driver.mark_dirty();
    driver.mark_dirty();
    wait_one_frame().await;
    assert_eq!(
        *call_count.borrow(),
        2,
        "複数回の mark_dirty は 1 フレームにつき 1 回の recompute へ coalesce されるはず"
    );

    // mark_dirty を呼ばないフレームでは recompute が増えないことも確認する。
    wait_one_frame().await;
    assert_eq!(
        *call_count.borrow(),
        2,
        "mark_dirty を呼ばないフレームでは recompute が増えないはず"
    );

    driver.stop();
}

#[wasm_bindgen_test]
async fn update_element_progress_writes_monotone_progress_as_scroll_advances() {
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");

    // 3000px の spacer の直後に高さ 200px の target を body 直下へ配置する。
    // ページ全体（`window`）をスクロールして target をビューポートへ近づける
    // 構成は、本番配線（`wire_scroll_driver_with_env` が `document`/`window`
    // へ `scroll`/`resize` リスナーを張る設計）に対応する最も素直な検証方法
    // である。`compute_progress` は `rect_top` の単調減少に対して
    // （clamp を介しても）単調非減少を保つ純関数のため、実際のビューポート
    // サイズ（headless Chrome の既定値）に依存せず本テストは決定的に成立
    // する。
    let spacer = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    spacer
        .set_attribute("style", "height:3000px")
        .expect("set_attribute must not fail");

    let target_el = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    target_el
        .set_attribute("style", "height:200px")
        .expect("set_attribute must not fail");

    let body = document
        .body()
        .expect("document body must exist in browser test environment");
    body.append_child(&spacer)
        .expect("append_child must not fail for a detached spacer");
    body.append_child(&target_el)
        .expect("append_child must not fail for a detached target");

    let element: web_sys::Element = target_el.clone().into();
    let mut target = DomTarget::custom_property(target_el.clone(), SCROLL_PROGRESS_PROPERTY);

    // scrollY == 0: target は spacer 3000px 分下にあり、どの一般的な
    // ビューポート高さでも `rect_top` はビューポート高さを大きく超える
    // ため、progress は 0.0 に clamp される（決定的）。
    window.scroll_to_with_x_and_y(0.0, 0.0);
    wait_one_frame().await;
    let progress_at_0 = update_element_progress(&element, &mut target)
        .expect("update_element_progress must succeed in a browser environment");

    window.scroll_to_with_x_and_y(0.0, 1500.0);
    wait_one_frame().await;
    let progress_at_mid = update_element_progress(&element, &mut target)
        .expect("update_element_progress must succeed in a browser environment");

    // scrollY == 3000: target はページ最上部近くまで到達し、`rect_top` は
    // 十分小さくなるため progress は 1.0 に clamp される（決定的）。
    window.scroll_to_with_x_and_y(0.0, 3000.0);
    wait_one_frame().await;
    let progress_at_full = update_element_progress(&element, &mut target)
        .expect("update_element_progress must succeed in a browser environment");

    for (label, value) in [
        ("progress_at_0", progress_at_0),
        ("progress_at_mid", progress_at_mid),
        ("progress_at_full", progress_at_full),
    ] {
        assert!(
            (0.0..=1.0).contains(&value),
            "{label} は 0.0..=1.0 の範囲であるはず: {value}"
        );
    }
    assert_eq!(progress_at_0, 0.0, "スクロール前は未進入で 0.0 のはず");
    assert!(
        progress_at_mid >= progress_at_0,
        "スクロールを進めると progress は単調非減少のはず: {progress_at_0} -> {progress_at_mid}"
    );
    assert!(
        progress_at_full >= progress_at_mid,
        "スクロールを進めると progress は単調非減少のはず: {progress_at_mid} -> {progress_at_full}"
    );
    assert!(
        progress_at_full > progress_at_0,
        "十分なスクロール後は未進入時より progress が増加しているはず"
    );

    let written = target_el
        .style()
        .get_property_value(SCROLL_PROGRESS_PROPERTY)
        .expect("get_property_value must not fail");
    assert_eq!(
        written
            .parse::<f64>()
            .expect("written value must be a valid f64 string"),
        progress_at_full,
        "custom property には最後に書き込んだ progress がそのまま反映されているはず"
    );

    // 後続テストへの汚染防止（ページスクロール位置・追加した DOM 要素を
    // 元に戻す）。
    window.scroll_to_with_x_and_y(0.0, 0.0);
    spacer.remove();
    target_el.remove();
}

#[wasm_bindgen_test]
async fn update_element_progress_uses_body_as_container_when_html_overflow_does_not_propagate() {
    // overflow-propagation（CSS Overflow Module Level 3
    // <https://www.w3.org/TR/css-overflow-3/#overflow-propagation>）:
    // `<html>` 自身が `overflow: hidden` を明示していると `<body>` の
    // overflow は viewport へ伝播せず、`<body>` 自身が独立したスクロール
    // コンテナになり得る（例: `html { overflow: hidden } body { height:
    // 300px; overflow-y: auto }`）。`find_scroll_container` はこの構成で
    // `<body>` を無条件除外せず通常のスクロールコンテナ候補として扱う
    // 必要があり、本テストはそれを実ブラウザで確認する
    // （codex-review P1 是正、PR #2557）。
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let document_element = document
        .document_element()
        .expect("document element must exist")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("document element must be an HtmlElement");
    let body = document
        .body()
        .expect("document body must exist in browser test environment");

    let previous_html_style = document_element.get_attribute("style").unwrap_or_default();
    let previous_body_style = body.get_attribute("style").unwrap_or_default();

    document_element
        .set_attribute("style", "overflow: hidden")
        .expect("set_attribute must not fail");
    body.set_attribute(
        "style",
        "margin: 0; height: 300px; overflow-y: auto; position: relative",
    )
    .expect("set_attribute must not fail");

    let spacer = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    spacer
        .set_attribute("style", "height:1000px")
        .expect("set_attribute must not fail");

    let target_el = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    target_el
        .set_attribute("style", "height:100px")
        .expect("set_attribute must not fail");

    body.append_child(&spacer)
        .expect("append_child must not fail for a detached spacer");
    body.append_child(&target_el)
        .expect("append_child must not fail for a detached target");

    let element: web_sys::Element = target_el.clone().into();
    let mut target = DomTarget::custom_property(target_el.clone(), SCROLL_PROGRESS_PROPERTY);

    body.set_scroll_top(0);
    wait_one_frame().await;
    let progress_at_0 = update_element_progress(&element, &mut target)
        .expect("update_element_progress must succeed in a browser environment");

    body.set_scroll_top(1000);
    wait_one_frame().await;
    let progress_at_full = update_element_progress(&element, &mut target)
        .expect("update_element_progress must succeed in a browser environment");

    // 後片付け（アサーション前に行い、失敗時も後続テストを汚染しない）。
    body.set_scroll_top(0);
    spacer.remove();
    target_el.remove();
    document_element
        .set_attribute("style", &previous_html_style)
        .expect("set_attribute must not fail");
    body.set_attribute("style", &previous_body_style)
        .expect("set_attribute must not fail");

    assert!(
        (0.0..=1.0).contains(&progress_at_0),
        "progress_at_0 は 0.0..=1.0 の範囲であるはず: {progress_at_0}"
    );
    assert!(
        (0.0..=1.0).contains(&progress_at_full),
        "progress_at_full は 0.0..=1.0 の範囲であるはず: {progress_at_full}"
    );
    assert!(
        progress_at_full > progress_at_0,
        "body 自身のスクロールに追従して progress が増加しているはず（body \
         を独立したスクロールコンテナとして基準に解決できている証拠）: \
         {progress_at_0} -> {progress_at_full}"
    );
}

#[wasm_bindgen_test]
async fn update_element_progress_holds_steady_inside_non_overflowing_hidden_container() {
    // codex-review P1 是正（PR #2557）の再現テスト: 高さ 300px の
    // `overflow: hidden` コンテナに、収まりきる高さ 100px の対象要素を
    // 配置する。コンテナ自体は溢れていない（`scrollHeight ==
    // clientHeight`）ため、`is_scroll_container` が `scrollHeight >
    // clientHeight` を必須条件にしていた旧実装ではこのコンテナを
    // 「スクロールコンテナではない」として読み飛ばし、より外側の
    // `window` を基準に進捗を計算してしまっていた（ページ全体スクロール
    // で進捗が 0 → 1 へ変化する誤り）。
    //
    // ネイティブ `animation-timeline: view()` は overflow 特性のみで
    // 近傍スクロールコンテナを決定し、実際のスクロール範囲の有無は
    // 問わない
    // (<https://drafts.csswg.org/scroll-animations-1/#view-notation>)。
    // 対象要素とコンテナは常に同じ相対位置を保ったままページスクロール
    // に追従するため、進捗はページスクロール位置によらず一定であるべき
    // （このコンテナが基準として選ばれていることの証拠）。
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let body = document
        .body()
        .expect("document body must exist in browser test environment");

    let spacer = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    spacer
        .set_attribute("style", "height:3000px")
        .expect("set_attribute must not fail");

    let container = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    container
        .set_attribute(
            "style",
            "height:300px; overflow: hidden; position: relative",
        )
        .expect("set_attribute must not fail");

    let target_el = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    target_el
        .set_attribute("style", "height:100px")
        .expect("set_attribute must not fail");

    container
        .append_child(&target_el)
        .expect("append_child must not fail for a detached target");
    body.append_child(&spacer)
        .expect("append_child must not fail for a detached spacer");
    body.append_child(&container)
        .expect("append_child must not fail for a detached container");

    let element: web_sys::Element = target_el.clone().into();
    let mut target = DomTarget::custom_property(target_el.clone(), SCROLL_PROGRESS_PROPERTY);

    window.scroll_to_with_x_and_y(0.0, 0.0);
    wait_one_frame().await;
    let progress_at_0 = update_element_progress(&element, &mut target)
        .expect("update_element_progress must succeed in a browser environment");

    window.scroll_to_with_x_and_y(0.0, 1500.0);
    wait_one_frame().await;
    let progress_at_mid = update_element_progress(&element, &mut target)
        .expect("update_element_progress must succeed in a browser environment");

    window.scroll_to_with_x_and_y(0.0, 3000.0);
    wait_one_frame().await;
    let progress_at_full = update_element_progress(&element, &mut target)
        .expect("update_element_progress must succeed in a browser environment");

    // 後片付け（アサーション前に行い、失敗時も後続テストを汚染しない）。
    window.scroll_to_with_x_and_y(0.0, 0.0);
    spacer.remove();
    container.remove();

    for (label, value) in [
        ("progress_at_0", progress_at_0),
        ("progress_at_mid", progress_at_mid),
        ("progress_at_full", progress_at_full),
    ] {
        assert!(
            (0.0..=1.0).contains(&value),
            "{label} は 0.0..=1.0 の範囲であるはず: {value}"
        );
    }
    assert_eq!(
        progress_at_0, progress_at_mid,
        "溢れていない overflow:hidden コンテナが基準として選ばれていれば、\
         ページスクロールによらず進捗は一定のはず（window 基準へ誤って \
         フォールバックしていない証拠）: {progress_at_0} -> {progress_at_mid}"
    );
    assert_eq!(
        progress_at_mid, progress_at_full,
        "溢れていない overflow:hidden コンテナが基準として選ばれていれば、\
         ページスクロールによらず進捗は一定のはず（window 基準へ誤って \
         フォールバックしていない証拠）: {progress_at_mid} -> {progress_at_full}"
    );
}

/// [`update_element_progress_for_range`]（イシュー #2534）が
/// [`ProgressRange::Cover`] で [`ProgressRange::Entry`] とは異なる（かつ
/// 数式どおりの）値を書き込むことを実 DOM で固定する。
///
/// `compute_progress_for_range` の native テスト（`cover_starts_at_zero_
/// when_top_edge_reaches_viewport_bottom` 等）が計算の正しさを固定済み
/// であり、本テストは実ブラウザの `getBoundingClientRect`/`innerHeight`
/// 計測経路と正しく配線されていることのみを検証する。
///
/// 同一 `rect_top`（= 分子 `viewport_height - rect_top` が同一）では
/// `entry = numerator / rect_height`、`cover = numerator / (viewport_
/// height + rect_height)` であり分母が `cover` の方が大きいため、
/// **`cover <= entry`** が常に成り立つ（`cover` は `entry` より遅れて
/// 1.0 へ到達する）。決定的な検証のため `rect_top` を実測の
/// `viewport_height` から逆算し、`entry` がちょうど `1.0` にクランプ
/// される（`rect_top <= viewport_height - rect_height`）一方で `cover`
/// は開区間 `(0.0, 1.0)` に収まる（非退化）位置までスクロールする。
#[wasm_bindgen_test]
async fn update_element_progress_for_range_cover_lags_behind_entry() {
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let viewport_height = window
        .inner_height()
        .expect("inner_height must not fail")
        .as_f64()
        .expect("inner_height must be a finite number");

    const TARGET_HEIGHT: f64 = 200.0;
    const SPACER_HEIGHT: f64 = 3000.0;
    // rect_top を viewport_height - 300 付近へ合わせる: entry の 1.0 到達
    // 条件（rect_top <= viewport_height - TARGET_HEIGHT）を満たしつつ、
    // cover は non-degenerate な中間値のまま（cover = 300 / (viewport_height
    // + TARGET_HEIGHT) は viewport_height が現実的な範囲である限り (0, 1)）。
    let target_document_top = SPACER_HEIGHT;
    let desired_rect_top = viewport_height - 300.0;
    let scroll_y = (target_document_top - desired_rect_top).max(0.0);

    let spacer = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    spacer
        .set_attribute("style", &format!("height:{SPACER_HEIGHT}px"))
        .expect("set_attribute must not fail");

    let target_el = document
        .create_element("div")
        .expect("create_element must not fail for a plain div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("created element must be an HtmlElement");
    target_el
        .set_attribute("style", &format!("height:{TARGET_HEIGHT}px"))
        .expect("set_attribute must not fail");

    let body = document
        .body()
        .expect("document body must exist in browser test environment");
    body.append_child(&spacer)
        .expect("append_child must not fail for a detached spacer");
    body.append_child(&target_el)
        .expect("append_child must not fail for a detached target");

    let element: web_sys::Element = target_el.clone().into();
    let mut entry_target = DomTarget::custom_property(target_el.clone(), SCROLL_PROGRESS_PROPERTY);
    let mut cover_target = DomTarget::custom_property(target_el.clone(), SCROLL_PROGRESS_PROPERTY);

    window.scroll_to_with_x_and_y(0.0, scroll_y);
    wait_one_frame().await;

    let entry_progress = update_element_progress(&element, &mut entry_target)
        .expect("update_element_progress must succeed in a browser environment");
    let cover_progress =
        update_element_progress_for_range(&element, &mut cover_target, ProgressRange::Cover)
            .expect("update_element_progress_for_range must succeed in a browser environment");

    assert_eq!(
        entry_progress, 1.0,
        "設計上の rect_top では entry が 1.0 にクランプされるはず: \
         entry={entry_progress} viewport_height={viewport_height} scroll_y={scroll_y}"
    );
    assert!(
        cover_progress > 0.0 && cover_progress < 1.0,
        "cover_progress は開区間 (0.0, 1.0) の中間値であるはず: {cover_progress}"
    );
    assert!(
        cover_progress < entry_progress,
        "同一 rect_top で cover は entry より遅れているはず（分母が大きいため）: \
         entry={entry_progress} cover={cover_progress}"
    );

    window.scroll_to_with_x_and_y(0.0, 0.0);
    spacer.remove();
    target_el.remove();
}

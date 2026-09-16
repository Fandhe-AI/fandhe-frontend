//! `fandhe_frontend_wasm_full::ticker`（イシュー #2540）の実ブラウザ
//! 統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `wasm-full/src/ticker.rs` の native テストは定数の安定性のみを検証
//! 済みである。本ファイルは入れ子 ticker（ticker 内に別の ticker を合成
//! した構成）でキーボードフォーカスが祖先すべてを一時停止させることを
//! 検証する（PR #2582 codex-review P1 指摘: `resolve_ticker_target` が
//! `closest` で最も内側の ticker だけを返していたため、内側の ticker へ
//! フォーカスしても外側は動き続けていた）。
//!
//! `Ticker` 自体の offset 前進・実効速度計算は責務境界どおり native
//! テスト（`crates/frontend-animation/src/ticker.rs`）で検証済みのため、
//! 本ファイルは実 DOM 上での `focusin`/`focusout` 配線（祖先すべてへの
//! 反映・`relatedTarget` による境界判定）のみを対象にする。

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "ticker")]

use fandhe_frontend_wasm_full::ticker::wire_ticker_with_reduced_motion;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード（`magnetic_browser.rs::
/// RemoveOnDrop` と同型。テスト間 DOM 汚染防止）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// 入れ子 ticker の DOM を組み立てる: `outer`（`data-fandhe-ticker`）の
/// `[data-part="content"]` 配下に、フォーカス可能な `outer-link` と、
/// もう 1 段の ticker（`inner`）を配置する。`inner` の `[data-part="content"]`
/// 配下には `inner-link` を置く。全要素に幅・高さを明示し、`getBoundingClientRect()`
/// に依存する複製数計算（[`crate::frontend_animation::ticker::required_copies`]）
/// が有限値を返すようにする。
fn build_nested_dom(document: &Document, id_prefix: &str) -> (Element, Element, Element) {
    let outer = document.create_element("div").unwrap();
    outer.set_id(&format!("{id_prefix}-outer"));
    outer.set_attribute("data-fandhe-ticker", "").unwrap();
    style_box(&outer, 200, 40);

    let outer_content = document.create_element("div").unwrap();
    outer_content.set_attribute("data-part", "content").unwrap();
    style_box(&outer_content, 400, 40);

    let outer_link = document.create_element("a").unwrap();
    outer_link.set_id(&format!("{id_prefix}-outer-link"));
    outer_link.set_attribute("href", "#").unwrap();
    outer_link.set_text_content(Some("outer link"));
    style_box(&outer_link, 80, 20);

    let inner = document.create_element("div").unwrap();
    inner.set_id(&format!("{id_prefix}-inner"));
    inner.set_attribute("data-fandhe-ticker", "").unwrap();
    style_box(&inner, 100, 20);

    let inner_content = document.create_element("div").unwrap();
    inner_content.set_attribute("data-part", "content").unwrap();
    style_box(&inner_content, 200, 20);

    let inner_link = document.create_element("a").unwrap();
    inner_link.set_id(&format!("{id_prefix}-inner-link"));
    inner_link.set_attribute("href", "#").unwrap();
    inner_link.set_text_content(Some("inner link"));
    style_box(&inner_link, 80, 20);

    inner_content.append_child(&inner_link).unwrap();
    inner.append_child(&inner_content).unwrap();
    outer_content.append_child(&outer_link).unwrap();
    outer_content.append_child(&inner).unwrap();
    outer.append_child(&outer_content).unwrap();

    document
        .body()
        .unwrap()
        .append_child(&outer)
        .expect("append_child must not fail for a detached div");

    (outer, inner, inner_link)
}

fn style_box(element: &Element, width: i32, height: i32) {
    let html_element = element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("element must cast to HtmlElement for style access");
    let style = html_element.style();
    style.set_property("display", "block").unwrap();
    style.set_property("width", &format!("{width}px")).unwrap();
    style
        .set_property("height", &format!("{height}px"))
        .unwrap();
}

/// `element` の `--fandhe-marquee-ticker-offset` の現在値を読む。
fn ticker_offset(element: &Element) -> String {
    let html_element = element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("element must cast to HtmlElement");
    html_element
        .style()
        .get_property_value("--fandhe-marquee-ticker-offset")
        .unwrap_or_default()
}

/// `set_timeout` を `await` 可能にする（`magnetic_browser.rs::sleep_ms`
/// と同型）。
async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist in browser test");
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// `"<n>px"` 形式の文字列から数値部分を読む（空文字列は `0.0` とみなす）。
fn px_value(value: &str) -> f64 {
    value
        .strip_suffix("px")
        .and_then(|n| n.parse::<f64>().ok())
        .unwrap_or(0.0)
}

/// 入れ子 ticker の内側要素（`inner-link`）へフォーカスすると、内側
/// （`inner`）だけでなく外側（`outer`）も一時停止すること（WCAG 2.2.2
/// 契約が入れ子祖先すべてに及ぶこと）を固定する。修正前は
/// `resolve_ticker_target` が `closest` で最も内側の 1 件しか返さず、
/// `inner` は停止するが `outer` は動き続けていた。
#[wasm_bindgen_test]
async fn nested_ticker_focus_pauses_all_ancestor_tickers() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (outer, inner, inner_link) = build_nested_dom(&document, "ticker-nest-1");
    let _guard = RemoveOnDrop(outer.clone());

    wire_ticker_with_reduced_motion(outer.clone(), false)
        .expect("wire_ticker_with_reduced_motion must not fail");

    // 配線直後、まだフォーカスしていない状態で 1 フレーム分だけ進行させ、
    // rAF ループが実際に動いていることを確認する（両方とも offset が
    // 前進しているはず）。
    sleep_ms(50).await;
    let outer_offset_before = px_value(&ticker_offset(&outer));
    let inner_offset_before = px_value(&ticker_offset(&inner));
    assert_ne!(
        outer_offset_before, 0.0,
        "フォーカス前は outer の offset が前進しているはず"
    );
    assert_ne!(
        inner_offset_before, 0.0,
        "フォーカス前は inner の offset が前進しているはず"
    );

    let html_inner_link = inner_link
        .dyn_into::<HtmlElement>()
        .expect("inner_link must cast to HtmlElement");
    html_inner_link
        .focus()
        .expect("focus() must not fail in a browser test");

    let outer_offset_at_focus = ticker_offset(&outer);
    let inner_offset_at_focus = ticker_offset(&inner);

    sleep_ms(100).await;

    assert_eq!(
        ticker_offset(&outer),
        outer_offset_at_focus,
        "入れ子内側へのフォーカスで外側 ticker の offset も停止するはず（祖先すべてへ反映、PR #2582 codex-review P1 指摘）"
    );
    assert_eq!(
        ticker_offset(&inner),
        inner_offset_at_focus,
        "入れ子内側へのフォーカスで内側 ticker の offset は停止するはず"
    );
}

/// 配線ルート自身が opt-in 要素（`data-fandhe-ticker`）である場合も配線
/// されること（`data-fandhe-ticker-active` が付与され rAF ループが動く
/// こと）を固定する。修正前は `querySelectorAll` の結果（子孫のみ）しか
/// 走査せず、ルート自身は起動しなかった（Cursor Bugbot 指摘）。
#[wasm_bindgen_test]
async fn wiring_root_itself_marked_as_ticker_is_started() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (outer, _inner, _inner_link) = build_nested_dom(&document, "ticker-root-self-1");
    let _guard = RemoveOnDrop(outer.clone());

    wire_ticker_with_reduced_motion(outer.clone(), false)
        .expect("wire_ticker_with_reduced_motion must not fail");

    assert!(
        outer.has_attribute("data-fandhe-ticker-active"),
        "配線ルート自身が data-fandhe-ticker を持つ場合は active 属性が付与されるはず"
    );
    // 最初の rAF フレームは（ページ読み込み直後など）遅延し得るため、
    // 固定の 1 回待ちではなく上限付きで offset の前進を待つ。
    let mut advanced = false;
    for _ in 0..40 {
        sleep_ms(50).await;
        if px_value(&ticker_offset(&outer)) != 0.0 {
            advanced = true;
            break;
        }
    }
    assert!(
        advanced,
        "配線ルート自身の ticker の offset が前進しているはず"
    );
}

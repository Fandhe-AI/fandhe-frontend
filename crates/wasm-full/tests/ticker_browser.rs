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

use fandhe_frontend_wasm_full::ticker::{
    active_ticker_count, stop_all_for_reduced_motion, subscription_count,
    wire_ticker_with_reduced_motion,
};
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

/// 外側が 3 個以上の複製を必要とする入れ子構成で、追加複製内の内側 ticker
/// が駆動されず静的（`data-fandhe-ticker-active` + offset `0px` 固定）で
/// あること、元の内側 ticker だけが JS 駆動されることを固定する
/// （PR #2582 codex-review P1・Cursor Bugbot 指摘: 追加複製内の内側
/// ticker は wire 時の走査後に生成されるため `active` に登録されず、
/// 元は JS 駆動・複製は CSS 駆動と位相がばらばらだった）。
#[wasm_bindgen_test]
async fn nested_ticker_clones_are_static_and_not_driven() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (outer, inner, _inner_link) = build_nested_dom(&document, "ticker-nest-clone-1");
    let _guard = RemoveOnDrop(outer.clone());
    // outer 200px / content 60px → 必要複製数 ceil(200/60)+1 = 5（3 個以上）。
    let outer_content = outer.first_element_child().unwrap();
    style_box(&outer_content, 60, 40);

    wire_ticker_with_reduced_motion(outer.clone(), false)
        .expect("wire_ticker_with_reduced_motion must not fail");

    let mut advanced = false;
    for _ in 0..40 {
        sleep_ms(50).await;
        if px_value(&ticker_offset(&inner)) != 0.0 {
            advanced = true;
            break;
        }
    }
    assert!(advanced, "元の内側 ticker は JS 駆動で前進しているはず");
    assert!(
        outer.child_element_count() >= 3,
        "前提: 外側は 3 個以上の複製を持つはず（実際 {}）",
        outer.child_element_count()
    );

    let nested = outer.query_selector_all("[data-fandhe-ticker]").unwrap();
    let mut clone_count = 0;
    for i in 0..nested.length() {
        let el = nested.item(i).unwrap().dyn_into::<Element>().unwrap();
        if el.is_same_node(Some(&inner)) {
            continue;
        }
        clone_count += 1;
        assert!(
            el.has_attribute("data-fandhe-ticker-active"),
            "複製内の内側 ticker は CSS 駆動へ落ちないよう active 属性を持つはず"
        );
        assert_eq!(
            ticker_offset(&el),
            "0px",
            "複製内の内側 ticker は offset 0px 固定（駆動されない）はず"
        );
    }
    assert!(
        clone_count >= 2,
        "前提: 内側 ticker の複製が 2 個以上あるはず"
    );
}

/// SSR 相当の 2 コピー構造（`pre-styled-ui::marquee_motion::ticker` は
/// 同じ children の content を `aria-hidden`/`inert` 付きで 2 回出力する）
/// で外側 + 内側 ticker を配線したとき、JS 駆動される内側は 1 コピー目の
/// 1 個だけで、2 コピー目の内側は静的（`data-fandhe-ticker-active` あり・
/// offset `0px` 固定・rAF 駆動なし）であること（Cursor Bugbot 指摘:
/// `ensure_copies` の静的化は追加複製にしか効かず、SSR コピー内の内側が
/// 独立駆動されていた）。
#[wasm_bindgen_test]
async fn nested_ticker_in_ssr_copy_is_static_and_not_driven() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (outer, inner, _inner_link) = build_nested_dom(&document, "ticker-nest-ssr-1");
    let _guard = RemoveOnDrop(outer.clone());
    // outer 200px / content 400px × 2 コピー → 追加複製は不要（SSR コピー
    // だけが 2 個目の content になる）。
    let outer_content = outer.first_element_child().unwrap();
    let ssr_copy = outer_content
        .clone_node_with_deep(true)
        .unwrap()
        .dyn_into::<Element>()
        .unwrap();
    ssr_copy.set_attribute("aria-hidden", "true").unwrap();
    ssr_copy.set_attribute("inert", "").unwrap();
    outer.append_child(&ssr_copy).unwrap();
    let ssr_inner = ssr_copy
        .query_selector("[data-fandhe-ticker]")
        .unwrap()
        .expect("SSR コピーにも内側 ticker が含まれるはず");

    wire_ticker_with_reduced_motion(outer.clone(), false)
        .expect("wire_ticker_with_reduced_motion must not fail");

    let mut advanced = false;
    for _ in 0..40 {
        sleep_ms(50).await;
        if px_value(&ticker_offset(&inner)) != 0.0 {
            advanced = true;
            break;
        }
    }
    assert!(
        advanced,
        "1 コピー目の内側 ticker は JS 駆動で前進しているはず"
    );
    assert!(
        ssr_inner.has_attribute("data-fandhe-ticker-active"),
        "SSR コピー内の内側 ticker は CSS 駆動へ落ちないよう active 属性を持つはず"
    );
    assert_eq!(
        ticker_offset(&ssr_inner),
        "0px",
        "SSR コピー内の内側 ticker は offset 0px 固定のはず"
    );
    sleep_ms(200).await;
    assert_eq!(
        ticker_offset(&ssr_inner),
        "0px",
        "SSR コピー内の内側 ticker は rAF で駆動されないはず"
    );
    assert_eq!(outer.child_element_count(), 2, "追加複製は生成されないはず");
}

/// ticker をマウント → 起動 → DOM から取り外すと、数フレーム後に保持
/// 一覧（`active`）から除去され件数が元に戻ること、再マウントで再び
/// 1 件増えることを固定する（PR #2582 codex-review P1 指摘: 切断済み
/// `Element`/`Ticker` が window の scroll/resize リスナー経由で蓄積して
/// いた）。他テストが残した ticker の遅延解放と干渉しないよう、待機後の
/// 件数を基準とした差分で検証する。
#[wasm_bindgen_test]
async fn disconnected_ticker_is_released_from_active_and_remount_registers_again() {
    let document = web_sys::window().unwrap().document().unwrap();
    sleep_ms(150).await;
    let baseline = active_ticker_count();
    let subscriptions_baseline = subscription_count();

    let (outer, inner, _inner_link) = build_nested_dom(&document, "ticker-release-1");
    wire_ticker_with_reduced_motion(outer.clone(), false)
        .expect("wire_ticker_with_reduced_motion must not fail");
    assert_eq!(
        active_ticker_count(),
        baseline + 2,
        "outer + inner の 2 件が登録されるはず"
    );
    assert_eq!(
        subscription_count(),
        subscriptions_baseline + 1,
        "root 側 + window 側の購読一式が 1 組保持されるはず"
    );
    let mut advanced = false;
    for _ in 0..40 {
        sleep_ms(50).await;
        if px_value(&ticker_offset(&inner)) != 0.0 {
            advanced = true;
            break;
        }
    }
    assert!(advanced, "前提: ticker は駆動中のはず");

    outer.remove();
    let mut released = false;
    for _ in 0..40 {
        sleep_ms(50).await;
        if active_ticker_count() == baseline {
            released = true;
            break;
        }
    }
    assert!(
        released,
        "切断後は数フレーム内に active から除去されるはず（実際 {}）",
        active_ticker_count()
    );
    assert_eq!(
        subscription_count(),
        subscriptions_baseline,
        "active が空になった時点で root 側リスナーを含む購読一式が drop されるはず"
    );

    let (outer2, _inner2, _inner_link2) = build_nested_dom(&document, "ticker-release-2");
    let _guard = RemoveOnDrop(outer2.clone());
    wire_ticker_with_reduced_motion(outer2.clone(), false)
        .expect("wire_ticker_with_reduced_motion must not fail");
    assert_eq!(
        active_ticker_count(),
        baseline + 2,
        "再マウントで再び登録されるはず"
    );
}

/// `prefers-reduced-motion: reduce` 確定で `stop()` した ticker は rAF が
/// 回らず切断フックが発火しないため、stop の時点で `active`・購読一式を
/// 解放し、root 取り外し後に件数が基準へ戻ることを固定する（Cursor
/// Bugbot 指摘: stop 済み ticker と切断済み DOM が scroll/resize が起き
/// ない SPA 遷移で永久に保持されていた）。実 OS 設定は切り替えられない
/// ため `stop_all_for_reduced_motion`（`change` リスナーと同じ処理）で
/// 代替する。
#[wasm_bindgen_test]
async fn stopped_ticker_by_reduced_motion_is_released_after_unmount() {
    let document = web_sys::window().unwrap().document().unwrap();
    sleep_ms(150).await;
    let baseline = active_ticker_count();
    let subscriptions_baseline = subscription_count();

    let (outer, inner, _inner_link) = build_nested_dom(&document, "ticker-release-rm-1");
    wire_ticker_with_reduced_motion(outer.clone(), false)
        .expect("wire_ticker_with_reduced_motion must not fail");
    assert_eq!(active_ticker_count(), baseline + 2);

    stop_all_for_reduced_motion();
    assert!(
        !inner.has_attribute("data-fandhe-ticker-active"),
        "reduce 確定で active 属性が外れ CSS 側の縮退へ委ねられるはず"
    );
    outer.remove();

    let mut released = false;
    for _ in 0..40 {
        sleep_ms(50).await;
        if active_ticker_count() == baseline && subscription_count() == subscriptions_baseline {
            released = true;
            break;
        }
    }
    assert!(
        released,
        "stop 済み ticker も取り外し後に解放されるはず（active {} / subscriptions {}）",
        active_ticker_count(),
        subscription_count()
    );
}

/// 入れ子構成の内側 ticker は動的複製（`ensure_copies`）を行わず SSR 時点の
/// 複製数で固定され、外側の全コピー（元 content・追加複製）の content 幅が
/// 一致することを固定する（Cursor Bugbot 指摘: 生きた内側だけが後から
/// 複製を増やすと外側のコピー間で幅が食い違い、外側の周期計測と実際の
/// コピー間隔がずれて継ぎ目が崩れていた）。
#[wasm_bindgen_test]
async fn nested_inner_ticker_keeps_copy_count_and_outer_copies_match_width() {
    let document = web_sys::window().unwrap().document().unwrap();
    let (outer, inner, _inner_link) = build_nested_dom(&document, "ticker-nest-width-1");
    let _guard = RemoveOnDrop(outer.clone());
    // outer 200px / content 60px → 外側は追加複製を持つ。inner 100px /
    // content 200px は単独なら MIN_COPIES=2 へ複製を増やす条件。
    let outer_content = outer.first_element_child().unwrap();
    style_box(&outer_content, 60, 40);
    let inner_copies_before = inner.child_element_count();

    wire_ticker_with_reduced_motion(outer.clone(), false)
        .expect("wire_ticker_with_reduced_motion must not fail");

    let mut advanced = false;
    for _ in 0..40 {
        sleep_ms(50).await;
        if px_value(&ticker_offset(&inner)) != 0.0 {
            advanced = true;
            break;
        }
    }
    assert!(advanced, "前提: 内側 ticker は JS 駆動で前進しているはず");
    assert_eq!(
        inner.child_element_count(),
        inner_copies_before,
        "入れ子の内側 ticker は動的複製を行わないはず"
    );
    assert!(
        outer.child_element_count() >= 3,
        "前提: 外側は追加複製を持つはず"
    );

    let first_width = outer_content
        .clone()
        .dyn_into::<HtmlElement>()
        .unwrap()
        .offset_width();
    let copies = outer.children();
    for i in 0..copies.length() {
        let copy = copies.item(i).unwrap().dyn_into::<HtmlElement>().unwrap();
        assert_eq!(
            copy.offset_width(),
            first_width,
            "外側のコピー {i} の幅は先頭 content と一致するはず"
        );
    }
}

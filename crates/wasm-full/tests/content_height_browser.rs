//! `fandhe_frontend_wasm_full::content_height`（イシュー #2191、親
//! トラッキング #2189。bubble の `collapse-content` 対象追加は #2282）の
//! 実ブラウザ回帰テスト。
//!
//! `crates/wasm-full/tests/content_height.rs`（native）は純粋層
//! （[`format_content_height`]/[`target_selector`]）の書式契約と
//! `TARGETS` 静的表のドリフト検知を担う。本ファイルはその先、**実ブラウザ
//! （headless Chromium、`wasm-pack test --headless --chrome`）上での
//! `wire_headless_component` 経由の実測・CSS 変数書き込み**という製品
//! 経路を検証する（`headless_wiring_browser.rs` と同型の実 DOM 検証
//! パターンを踏襲する）。
//!
//! # bubble 分のテストが click 駆動形を写せない理由
//!
//! collapsible/accordion 分（上記）はトリガークリック →
//! `wire_headless_component` の `on_update` → `sync_content_height` という
//! 製品経路をそのまま検証できるが、bubble は
//! `crates/wasm-full/src/headless.rs` の `MAPPING_TABLE` に
//! `(bubble, collapse-trigger)` の行を持たず、`wire_headless_component`
//! 経由のクリックでは dispatch されない（`crates/headless-ui/src/bubble.rs`
//! rustdoc「wasm-full 未配線」節、`src/content_height.rs` モジュール doc
//! 「スコープ外」節参照）。このため bubble 分は 2 形で検証する: (a)
//! 配線時初期同期（`wire_headless_component` 自体が呼ぶ先行同期、クリック
//! 不要）、(b) 開閉再描画 + `sync_content_height` 直接呼び出し
//! （`wire_headless_component` が `on_update` 直後に呼ぶのと同一経路を
//! 呼び出し側が直接再現する）。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_headless_ui::accordion::{self, Accordion, AccordionProps};
use fandhe_frontend_headless_ui::bubble;
use fandhe_frontend_headless_ui::collapsible::{self, Collapsible};
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_wasm_full::content_height::{sync_content_height, CONTENT_HEIGHT_VAR};
use fandhe_frontend_wasm_full::headless::wire_headless_component;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のルートコンテナ要素を document body へ 1 個生成する
/// （`headless_wiring_browser.rs::create_container` と同型）。
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

/// テスト末尾でコンテナを document から確実に除去する RAII ガード
/// （`headless_wiring_browser.rs::RemoveOnDrop` と同型）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// 合成 `click` イベントを生成する（`bubbles: true`）。
fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail for click")
}

fn dispatch_click(target: &Element) {
    target
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
}

/// `element` の CSSOM 経由の `--fandhe-content-height` 値を読む
/// （`get_attribute("style")` の文字列比較ではなく `CssStyleDeclaration`
/// 経由で読み、書き込み手段〔CSSOM〕と対称にする）。
fn content_height_var(element: &Element) -> String {
    element
        .dyn_ref::<HtmlElement>()
        .expect("element must be an HtmlElement")
        .style()
        .get_property_value(CONTENT_HEIGHT_VAR)
        .expect("get_property_value must not fail")
}

use wasm_bindgen::JsCast;

/// 固定高さの子要素（`style="height:{px}px"`）を持つ content の内側
/// ノードを組み立てる。`sync_content_height` が読む `scroll_height()` は
/// レイアウトが確定した実 DOM 上でのみ意味を持つため、子要素へ明示の
/// `height` を与えて実測値を決定的にする。`raw_html` は使わず
/// `fandhe_frontend_core::el_owned`（通常のノード木 API）で組み立てる
/// （`.claude/rules/coding-rust.md`「HTML 文字列の直接組み立て禁止」）。
fn fixed_height_child(px: u32) -> fandhe_frontend_core::Node {
    fandhe_frontend_core::el_owned(
        "div",
        vec![("style".to_string(), format!("height:{px}px"))],
        vec![],
    )
}

// --- collapsible: trigger クリックで書き込み ---

#[wasm_bindgen_test]
fn collapsible_open_click_writes_content_height_var() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "content-height-collapsible-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let html = fandhe_frontend_core::render(&collapsible::root(
        OpenState::Closed,
        false,
        vec![],
        vec![
            collapsible::trigger(
                OpenState::Closed,
                false,
                None,
                vec![],
                vec![fandhe_frontend_core::text("Toggle")],
            ),
            collapsible::content(
                OpenState::Closed,
                false,
                None,
                vec![],
                vec![fixed_height_child(240)],
            ),
        ],
    ));
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("collapsible root must exist");
    let trigger = root
        .query_selector(r#"[data-part="trigger"]"#)
        .expect("query_selector must not fail")
        .expect("trigger element must exist");

    let component = Rc::new(RefCell::new(Collapsible::default()));
    // `on_update` は closed → open 遷移後の再描画を模して content を
    // 差し替える（`set_inner_html` 全体再生成モデル、モジュール doc
    // 「遷移成立条件についての注記」の再描画モデルに対応する経路）。
    wire_headless_component(root.clone(), component.clone(), move |state, root| {
        let is_open = state.is_open();
        let html = fandhe_frontend_core::render(&collapsible::root(
            if is_open {
                OpenState::Open
            } else {
                OpenState::Closed
            },
            false,
            vec![],
            vec![
                collapsible::trigger(
                    if is_open {
                        OpenState::Open
                    } else {
                        OpenState::Closed
                    },
                    false,
                    None,
                    vec![],
                    vec![fandhe_frontend_core::text("Toggle")],
                ),
                collapsible::content(
                    if is_open {
                        OpenState::Open
                    } else {
                        OpenState::Closed
                    },
                    false,
                    None,
                    vec![],
                    vec![fixed_height_child(240)],
                ),
            ],
        ));
        root.set_inner_html(&html);
    })
    .expect("wire_headless_component must not fail");

    dispatch_click(&trigger);
    assert!(component.borrow().is_open());

    let root_after = container
        .first_element_child()
        .expect("collapsible root must exist after re-render");
    let content = root_after
        .query_selector(r#"[data-part="content"]"#)
        .expect("query_selector must not fail")
        .expect("content element must exist after re-render");
    assert_eq!(
        content_height_var(&content),
        "240px",
        "open 後の content には実測高さが書き込まれること"
    );
}

// --- accordion: item クリックで対象項目のみ書き込み ---

#[wasm_bindgen_test]
fn accordion_item_open_click_writes_content_height_var_for_target_item_only() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "content-height-accordion-root");
    let _cleanup = RemoveOnDrop(container.clone());

    fn render_accordion(a: &Accordion) -> String {
        let props = AccordionProps::default();
        fandhe_frontend_core::render(&accordion::root(
            &props,
            vec![],
            vec![
                accordion::item(
                    a.item_state("panel-1"),
                    false,
                    &props,
                    vec![],
                    vec![
                        accordion::item_trigger(
                            a.item_state("panel-1"),
                            false,
                            &props,
                            "panel-1",
                            None,
                            None,
                            vec![],
                            vec![fandhe_frontend_core::text("Panel 1")],
                        ),
                        accordion::item_content(
                            a.item_state("panel-1"),
                            false,
                            &props,
                            None,
                            None,
                            vec![],
                            vec![fixed_height_child(120)],
                        ),
                    ],
                ),
                accordion::item(
                    a.item_state("panel-2"),
                    false,
                    &props,
                    vec![],
                    vec![
                        accordion::item_trigger(
                            a.item_state("panel-2"),
                            false,
                            &props,
                            "panel-2",
                            None,
                            None,
                            vec![],
                            vec![fandhe_frontend_core::text("Panel 2")],
                        ),
                        accordion::item_content(
                            a.item_state("panel-2"),
                            false,
                            &props,
                            None,
                            None,
                            vec![],
                            vec![fixed_height_child(80)],
                        ),
                    ],
                ),
            ],
        ))
    }

    let initial = Accordion::default();
    container.set_inner_html(&render_accordion(&initial));
    let root = container
        .first_element_child()
        .expect("accordion root must exist");
    let trigger1 = root
        .query_selector(r#"[data-value="panel-1"]"#)
        .expect("query_selector must not fail")
        .expect("panel-1 trigger must exist");

    let component = Rc::new(RefCell::new(Accordion::default()));
    wire_headless_component(root.clone(), component.clone(), move |state, root| {
        root.set_inner_html(&render_accordion(state));
    })
    .expect("wire_headless_component must not fail");

    dispatch_click(&trigger1);
    assert!(component.borrow().is_open("panel-1"));

    let root_after = container
        .first_element_child()
        .expect("accordion root must exist after re-render");
    let contents = root_after
        .query_selector_all(r#"[data-part="item-content"]"#)
        .expect("query_selector_all must not fail");
    assert_eq!(contents.length(), 2);

    let content1 = contents
        .get(0)
        .expect("panel-1 content must exist")
        .dyn_into::<Element>()
        .expect("must be an Element");
    let content2 = contents
        .get(1)
        .expect("panel-2 content must exist")
        .dyn_into::<Element>()
        .expect("must be an Element");

    assert_eq!(
        content_height_var(&content1),
        "120px",
        "open している panel-1 には実測高さが書き込まれること"
    );
    assert_eq!(
        content_height_var(&content2),
        "",
        "closed のままの panel-2 には変数が書き込まれないこと"
    );
}

// --- 配線時初回同期 ---

#[wasm_bindgen_test]
fn wiring_time_initial_sync_writes_content_height_var_for_already_open_content() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "content-height-initial-sync-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // SSR 時点で既に open な collapsible（配線前から content が可視）。
    let html = fandhe_frontend_core::render(&collapsible::root(
        OpenState::Open,
        false,
        vec![],
        vec![
            collapsible::trigger(
                OpenState::Open,
                false,
                None,
                vec![],
                vec![fandhe_frontend_core::text("Toggle")],
            ),
            collapsible::content(
                OpenState::Open,
                false,
                None,
                vec![],
                vec![fixed_height_child(240)],
            ),
        ],
    ));
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("collapsible root must exist");
    let content = root
        .query_selector(r#"[data-part="content"]"#)
        .expect("query_selector must not fail")
        .expect("content element must exist");

    // 配線前は当然未設定であることの確認（前提の明示）。
    assert_eq!(content_height_var(&content), "");

    let component = Rc::new(RefCell::new(Collapsible::new(OpenState::Open)));
    wire_headless_component(root.clone(), component, |_state, _root| {})
        .expect("wire_headless_component must not fail");

    assert_eq!(
        content_height_var(&content),
        "240px",
        "配線直後（on_update を経由しない）にも初期表示の高さが同期されること"
    );
}

// --- 閉状態は上書きしない / 0px を書かない ---

#[wasm_bindgen_test]
fn closed_content_after_rerender_has_no_content_height_var() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "content-height-close-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let html = fandhe_frontend_core::render(&collapsible::root(
        OpenState::Open,
        false,
        vec![],
        vec![
            collapsible::trigger(
                OpenState::Open,
                false,
                None,
                vec![],
                vec![fandhe_frontend_core::text("Toggle")],
            ),
            collapsible::content(
                OpenState::Open,
                false,
                None,
                vec![],
                vec![fixed_height_child(240)],
            ),
        ],
    ));
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("collapsible root must exist");
    let trigger = root
        .query_selector(r#"[data-part="trigger"]"#)
        .expect("query_selector must not fail")
        .expect("trigger element must exist");

    let component = Rc::new(RefCell::new(Collapsible::new(OpenState::Open)));
    wire_headless_component(root.clone(), component.clone(), move |state, root| {
        let is_open = state.is_open();
        let s = if is_open {
            OpenState::Open
        } else {
            OpenState::Closed
        };
        let html = fandhe_frontend_core::render(&collapsible::root(
            s,
            false,
            vec![],
            vec![
                collapsible::trigger(
                    s,
                    false,
                    None,
                    vec![],
                    vec![fandhe_frontend_core::text("Toggle")],
                ),
                collapsible::content(s, false, None, vec![], vec![fixed_height_child(240)]),
            ],
        ));
        root.set_inner_html(&html);
    })
    .expect("wire_headless_component must not fail");

    // open → close: 新規生成された hidden 付き content には変数が
    // 書き込まれないこと。
    dispatch_click(&trigger);
    assert!(!component.borrow().is_open());

    let root_after = container
        .first_element_child()
        .expect("collapsible root must exist after re-render");
    let content_after = root_after
        .query_selector(r#"[data-part="content"]"#)
        .expect("query_selector must not fail")
        .expect("content element must exist after re-render");
    assert!(content_after.has_attribute("hidden"));
    assert_eq!(
        content_height_var(&content_after),
        "",
        "hidden な content には変数が書き込まれないこと"
    );
}

#[wasm_bindgen_test]
fn sync_content_height_on_hidden_root_subtree_is_noop() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "content-height-hidden-direct-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let html = fandhe_frontend_core::render(&collapsible::content(
        OpenState::Closed,
        false,
        None,
        vec![],
        vec![fixed_height_child(240)],
    ));
    container.set_inner_html(&html);
    let content = container
        .first_element_child()
        .expect("content element must exist");
    assert!(content.has_attribute("hidden"));

    // hidden を持つ要素を直接 sync_content_height に渡しても変数が
    // 書き込まれない（root 自身が対象パーツの場合の分岐、モジュール doc
    // 「`hidden`・0px の扱い」節）。
    sync_content_height(&content).expect("sync_content_height must not fail");
    assert_eq!(content_height_var(&content), "");
}

// --- ネスト 0 実測の除去 ---

#[wasm_bindgen_test]
fn nested_zero_measurement_removes_existing_var() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "content-height-nested-zero-root");
    let _cleanup = RemoveOnDrop(container.clone());

    // 祖先が display:none（closed accordion item content 相当）の下に
    // open な collapsible content を置く。scroll_height は祖先の
    // display:none により 0 になる。
    let inner = collapsible::content(
        OpenState::Open,
        false,
        None,
        vec![],
        vec![fixed_height_child(240)],
    );
    // 外側の div も HTML 文字列直接組み立てではなくノード木 API
    // （`el_owned`）で構築し、collapsible::content の Node を子として
    // 渡してから全体を render する（coding-rust.md「HTML 文字列の直接
    // 組み立て禁止」規約準拠）。
    let outer = fandhe_frontend_core::el_owned(
        "div",
        vec![("style".to_string(), "display:none".to_string())],
        vec![inner],
    );
    let outer_html = fandhe_frontend_core::render(&outer);
    container.set_inner_html(&outer_html);

    let inner = container
        .query_selector(r#"[data-part="content"]"#)
        .expect("query_selector must not fail")
        .expect("inner content element must exist");

    // 事前に非ゼロ値を書き込んでおく（前回の in-place 測定値が残って
    // いる状況の再現）。
    inner
        .dyn_ref::<HtmlElement>()
        .expect("must be HtmlElement")
        .style()
        .set_property(CONTENT_HEIGHT_VAR, "999px")
        .expect("set_property must not fail");
    assert_eq!(content_height_var(&inner), "999px");

    let root = container
        .first_element_child()
        .expect("outer wrapper must exist");
    sync_content_height(&root).expect("sync_content_height must not fail");

    assert_eq!(
        content_height_var(&inner),
        "",
        "祖先 display:none 下での実測 0 は既存値を除去すること（0px を焼き込まない）"
    );
}

// --- XSS 回帰: data-value / テキストに payload を含めても style 値は数値+px のみ ---

#[wasm_bindgen_test]
fn xss_payload_in_data_value_and_text_does_not_affect_style_value() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "content-height-xss-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let payload = "\"><script>window.__xss=1</script>";
    let props = AccordionProps::default();
    let html = fandhe_frontend_core::render(&accordion::root(
        &props,
        vec![],
        vec![accordion::item(
            OpenState::Open,
            false,
            &props,
            vec![],
            vec![
                accordion::item_trigger(
                    OpenState::Open,
                    false,
                    &props,
                    payload,
                    None,
                    None,
                    vec![],
                    vec![fandhe_frontend_core::text(payload)],
                ),
                accordion::item_content(
                    OpenState::Open,
                    false,
                    &props,
                    None,
                    None,
                    vec![],
                    vec![fixed_height_child(240)],
                ),
            ],
        )],
    ));
    container.set_inner_html(&html);
    // core の既定エスケープにより <script> はテキストとして描画され、
    // 要素としては生成されないことを確認する（REQ-1 の回帰確認）。
    assert!(
        container.query_selector("script").unwrap().is_none(),
        "payload の <script> がテキストではなく要素として解釈されていないこと"
    );

    let root = container
        .first_element_child()
        .expect("accordion root must exist");
    sync_content_height(&root).expect("sync_content_height must not fail");

    let content = root
        .query_selector(r#"[data-part="item-content"]"#)
        .expect("query_selector must not fail")
        .expect("item-content element must exist");
    let value = content_height_var(&content);
    assert_eq!(value, "240px");
    assert!(
        value
            .strip_suffix("px")
            .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())),
        "style 値は 10 進整数 + \"px\" のみであること（payload の影響を受けない）: {value}"
    );
}

// --- bubble: MAPPING_TABLE 未配線のため 2 形で検証（モジュール doc参照） ---

/// (a) 配線時初期同期: SSR 時点で既に open な bubble
/// （`wire_headless_component` の先行同期、クリック不要）。
#[wasm_bindgen_test]
fn bubble_wiring_time_initial_sync_writes_content_height_var_for_already_open_collapse_content() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "content-height-bubble-initial-sync-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let html = fandhe_frontend_core::render(&bubble::root(
        bubble::BubbleRootProps::default(),
        vec![],
        vec![
            bubble::collapse_trigger(
                OpenState::Open,
                Some("bubble-collapse-initial"),
                vec![],
                vec![fandhe_frontend_core::text("Toggle")],
            ),
            bubble::collapse_content(
                OpenState::Open,
                Some("bubble-collapse-initial"),
                vec![],
                vec![fixed_height_child(240)],
            ),
        ],
    ));
    container.set_inner_html(&html);
    let root = container
        .first_element_child()
        .expect("bubble root must exist");
    let content = root
        .query_selector(r#"[data-part="collapse-content"]"#)
        .expect("query_selector must not fail")
        .expect("collapse-content element must exist");

    // 配線前は当然未設定であることの確認（前提の明示）。
    assert_eq!(content_height_var(&content), "");

    // `MAPPING_TABLE` に bubble の行が無いため dispatch 対象にならない
    // `Component` 実装だが、`wire_headless_component` 自体の先行同期は
    // dispatch とは無関係に呼ばれる（`crate::headless::wire_headless_component`
    // 本体・モジュール doc参照）。
    let component = Rc::new(RefCell::new(Collapsible::default()));
    wire_headless_component(root.clone(), component, |_state, _root| {})
        .expect("wire_headless_component must not fail");

    assert_eq!(
        content_height_var(&content),
        "240px",
        "配線直後（クリック不要）にも初期表示の高さが同期されること"
    );
}

/// (b) 開閉再描画 + `sync_content_height` 直接呼び出し: closed → open →
/// closed と `set_inner_html` で丸ごと再描画し、各回
/// `wire_headless_component` が `on_update` 直後に呼ぶのと同一経路
/// （`sync_content_height` 直接呼び出し）で同期する。
#[wasm_bindgen_test]
fn bubble_direct_sync_after_rerender_open_writes_and_closed_removes_content_height_var() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "content-height-bubble-rerender-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let render_at = |state: OpenState| {
        fandhe_frontend_core::render(&bubble::root(
            bubble::BubbleRootProps::default(),
            vec![],
            vec![
                bubble::collapse_trigger(
                    state,
                    Some("bubble-collapse-rerender"),
                    vec![],
                    vec![fandhe_frontend_core::text("Toggle")],
                ),
                bubble::collapse_content(
                    state,
                    Some("bubble-collapse-rerender"),
                    vec![],
                    vec![fixed_height_child(240)],
                ),
            ],
        ))
    };

    // closed → open → closed の順に丸ごと再描画し、都度直接同期する。
    for (state, expect_written) in [
        (OpenState::Closed, false),
        (OpenState::Open, true),
        (OpenState::Closed, false),
    ] {
        container.set_inner_html(&render_at(state));
        let root = container
            .first_element_child()
            .expect("bubble root must exist after re-render");
        sync_content_height(&root).expect("sync_content_height must not fail");

        let content = root
            .query_selector(r#"[data-part="collapse-content"]"#)
            .expect("query_selector must not fail")
            .expect("collapse-content element must exist");
        let value = content_height_var(&content);
        if expect_written {
            assert_eq!(
                value, "240px",
                "open + 固定高さ子要素は実測値が書き込まれること"
            );
        } else {
            assert_eq!(
                value, "",
                "closed（hidden 属性あり）は測定対象外で変数が書き込まれないこと"
            );
        }
    }
}

/// (c) XSS 回帰: `id`/`controls`/テキストへ payload を注入しても style
/// 値は 10 進整数 + `px` のみであること（`xss_payload_in_data_value_and_text_does_not_affect_style_value`
/// と同型）。
#[wasm_bindgen_test]
fn bubble_xss_payload_in_id_controls_and_text_does_not_affect_style_value() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let container = create_container(&document, "content-height-bubble-xss-root");
    let _cleanup = RemoveOnDrop(container.clone());

    let payload = "\"><script>window.__xss=1</script>";
    let html = fandhe_frontend_core::render(&bubble::root(
        bubble::BubbleRootProps::default(),
        vec![],
        vec![
            bubble::collapse_trigger(
                OpenState::Open,
                Some(payload),
                vec![],
                vec![fandhe_frontend_core::text(payload)],
            ),
            bubble::collapse_content(
                OpenState::Open,
                Some(payload),
                vec![],
                vec![fixed_height_child(240)],
            ),
        ],
    ));
    container.set_inner_html(&html);
    // core の既定エスケープにより <script> はテキストとして描画され、
    // 要素としては生成されないことを確認する（REQ-1 の回帰確認）。
    assert!(
        container.query_selector("script").unwrap().is_none(),
        "payload の <script> がテキストではなく要素として解釈されていないこと"
    );

    let root = container
        .first_element_child()
        .expect("bubble root must exist");
    sync_content_height(&root).expect("sync_content_height must not fail");

    let content = root
        .query_selector(r#"[data-part="collapse-content"]"#)
        .expect("query_selector must not fail")
        .expect("collapse-content element must exist");
    let value = content_height_var(&content);
    assert_eq!(value, "240px");
    assert!(
        value
            .strip_suffix("px")
            .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())),
        "style 値は 10 進整数 + \"px\" のみであること（payload の影響を受けない）: {value}"
    );
}

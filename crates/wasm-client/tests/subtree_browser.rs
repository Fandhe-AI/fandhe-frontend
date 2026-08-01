//! `replace_subtree()` の実ブラウザテスト（イシュー #1121）。
//!
//! 検証する不変条件（`crates/wasm-client/src/subtree.rs` rustdoc・
//! `lib.rs` クレート冒頭不変条件 7 参照）:
//! - 置換成功時、返り値が実際に DOM へ挿入された新ノードであること。
//! - `Node::RawHtml` 混入時は DOM を一切変更せず `Err` を返すこと
//!   （fail-closed、既存の `build_dom_node` 契約の継承）。
//! - `<script>` 相当の文字列は `Node::Text` として渡された場合、実 DOM 上でも
//!   `<script>` 要素化せずテキストノードのまま現れること（XSS 回帰、
//!   `render_into`/`build_dom_node` と同じ既定エスケープ相当の保証を
//!   `create_text_node` 経由の DOM 直接構築でも確認する）。
//! - `href="javascript:..."` のような危険スキームは属性として書き込まれないこと。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::{el, el_owned, raw_html, text};
use fandhe_frontend_wasm_client::replace_subtree;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のルートコンテナと、その配下の差し替え対象スロット要素を
/// document body へ生成する。`wasm-bindgen-test` はテスト間で DOM をリセット
/// しないため、一意な id を振って干渉を避ける（`hydrate_smoke.rs` と同じ意図）。
fn create_slot(document: &Document, root_id: &str, slot_id: &str) -> Element {
    let root = document
        .create_element("div")
        .expect("create_element must not fail for a plain div");
    root.set_id(root_id);
    document
        .body()
        .expect("document body must exist in browser test environment")
        .append_child(&root)
        .expect("append_child must not fail for a detached div");

    let slot = document
        .create_element("span")
        .expect("create_element must not fail for a plain span");
    slot.set_id(slot_id);
    slot.set_text_content(Some("original"));
    root.append_child(&slot)
        .expect("append_child must not fail for a detached span");
    slot
}

/// 観点 1: 置換成功時、返り値が実際に DOM へ挿入された新ノードであること。
#[wasm_bindgen_test]
fn replace_subtree_returns_the_inserted_node() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let slot = create_slot(&document, "subtree-root-1", "subtree-slot-1");

    let replacement = el("strong", vec![("data-replaced", "yes")], vec![text("new")]);
    let inserted = replace_subtree(&slot, &replacement)
        .expect("replace_subtree must succeed for a plain element");

    let inserted_element: Element = inserted
        .dyn_into()
        .expect("inserted node must be an Element");
    assert_eq!(inserted_element.tag_name().to_lowercase(), "strong");
    assert_eq!(inserted_element.text_content(), Some("new".to_string()));
    assert_eq!(
        inserted_element.get_attribute("data-replaced"),
        Some("yes".to_string())
    );

    // 旧スロットは DOM から取り除かれている（置換であり追加ではない）。
    let root = document
        .get_element_by_id("subtree-root-1")
        .expect("root must still exist");
    assert!(
        document.get_element_by_id("subtree-slot-1").is_none(),
        "旧スロット要素は置換後に DOM から取り除かれること"
    );
    assert_eq!(
        root.children().length(),
        1,
        "置換後は新ノード 1 個のみが子であること"
    );
}

/// 観点 2: `Node::RawHtml` 混入時は DOM を一切変更せず `Err` を返す
/// （fail-closed、`build_dom_node` の既存契約の継承）。
#[wasm_bindgen_test]
fn replace_subtree_fails_closed_on_raw_html_without_mutating_dom() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let slot = create_slot(&document, "subtree-root-2", "subtree-slot-2");

    #[expect(
        clippy::disallowed_methods,
        reason = "ESCAPE-REVIEWED: RawHtml 混入時の fail-closed 拒否を検証するテスト。固定の信頼済み文字列のみ"
    )]
    let dangerous = el("div", vec![], vec![raw_html("<b>bold</b>")]);

    let result = replace_subtree(&slot, &dangerous);
    assert!(
        result.is_err(),
        "RawHtml を含むノードは replace_subtree が Err を返すこと"
    );

    // DOM は変更されていない（旧スロットがそのまま残る）。
    let still_there = document
        .get_element_by_id("subtree-slot-2")
        .expect("slot must remain in the DOM after a failed replace");
    assert_eq!(still_there.text_content(), Some("original".to_string()));
}

/// 観点 3: `<script>` 相当の文字列は `Node::Text` として渡す限り、実 DOM 上でも
/// `<script>` 要素化せずテキストノードのまま現れる（XSS 回帰）。
#[wasm_bindgen_test]
fn replace_subtree_keeps_script_like_text_as_plain_text() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let slot = create_slot(&document, "subtree-root-3", "subtree-slot-3");

    let payload = "<script>alert(1)</script>";
    let node = el("div", vec![], vec![text(payload)]);
    let inserted = replace_subtree(&slot, &node).expect("replace_subtree must succeed");
    let inserted_element: Element = inserted
        .dyn_into()
        .expect("inserted node must be an Element");

    assert_eq!(
        inserted_element
            .query_selector("script")
            .expect("query_selector must not fail"),
        None,
        "text() で渡した <script> 文字列が実 DOM 上で script 要素化してはならない"
    );
    assert_eq!(inserted_element.text_content(), Some(payload.to_string()));
}

/// 観点 4: `href="javascript:..."` のような危険スキームは属性として書き込まれない
/// （`build_dom_node` の URL スキーム検証を replace_subtree 経由でも継承する）。
#[wasm_bindgen_test]
fn replace_subtree_blocks_dangerous_url_scheme_attributes() {
    let window = web_sys::window().expect("window must exist");
    let document = window.document().expect("document must exist");
    let slot = create_slot(&document, "subtree-root-4", "subtree-slot-4");

    let node = el_owned(
        "a",
        vec![("href".to_string(), "javascript:alert(1)".to_string())],
        vec![text("click me")],
    );
    let inserted = replace_subtree(&slot, &node).expect("replace_subtree must succeed");
    let inserted_element: Element = inserted
        .dyn_into()
        .expect("inserted node must be an Element");

    assert_eq!(
        inserted_element.get_attribute("href"),
        None,
        "javascript: スキームの href は書き込まれないこと"
    );
}

//! `fandhe_frontend_animation::ticker` の DOM 計測・複製生成の実ブラウザ
//! 統合テスト（イシュー #2540、PR #2582 codex-review P1・Cursor Bugbot
//! 是正）。`presence_browser.rs` と同型の `wasm-pack test --headless
//! --chrome` ハーネス。
//!
//! 純粋層（`effective_speed`/`advance_offset`/`required_copies` 等）は
//! `src/ticker.rs` の native `cargo test` が検証済み。本ファイルは
//! (1) 祖先 `scale` 下でも周期をレイアウト座標で計測すること、
//! (2) `gap` shorthand 2 値でも軸に応じた longhand を読むこと、
//! (3) 追加複製内の radio がグループから隔離され元の選択が保たれること、
//! (4) 追加複製内の入れ子 ticker が静的化されることを対象とする。

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_animation::ticker::{ensure_copies, measure_len, read_gap_px, Axis};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, FormData, HtmlElement, HtmlFormElement, HtmlInputElement};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト末尾で DOM を確実に除去する RAII ガード。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

fn document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}

fn html(element: &Element) -> HtmlElement {
    element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("element must cast to HtmlElement")
}

fn div_with_style(document: &Document, style: &str) -> Element {
    let el = document.create_element("div").unwrap();
    html(&el).style().set_css_text(style);
    el
}

/// `root`（表示領域）+ `content`（複製テンプレート）+ 既存 SSR 複製 1 個
/// の marquee 相当 DOM を組み立てて body へ接続する。
fn build_marquee(document: &Document, root_style: &str, content_style: &str) -> (Element, Element) {
    let root = div_with_style(document, root_style);
    let content = div_with_style(document, content_style);
    content.set_attribute("data-part", "content").unwrap();
    let ssr_copy = content.clone_node_with_deep(true).unwrap();
    root.append_child(&content).unwrap();
    root.append_child(&ssr_copy).unwrap();
    document.body().unwrap().append_child(&root).unwrap();
    (root, content)
}

/// 祖先が `transform: scale(2)` でも、周期の計測（`measure_len`）は
/// transform 適用前のレイアウト座標（content 幅 100px）を返す。
/// `getBoundingClientRect()` ベースの旧実装は 200px を返し、CSS translate
/// のローカル px と座標系がずれていた（codex-review P1 指摘）。
#[wasm_bindgen_test]
fn measure_len_ignores_ancestor_scale() {
    let document = document();
    let scaled = div_with_style(&document, "transform: scale(2); width: 300px;");
    let _guard = RemoveOnDrop(scaled.clone());
    document.body().unwrap().append_child(&scaled).unwrap();
    let content = div_with_style(&document, "display: block; width: 100px; height: 20px;");
    scaled.append_child(&content).unwrap();

    let screen_width = content.get_bounding_client_rect().width();
    assert!(
        (screen_width - 200.0).abs() < 1.0,
        "前提: 画面上の幅は scale(2) で 200px になっているはず（実測 {screen_width}）"
    );
    assert_eq!(measure_len(&content, Axis::Horizontal), 100.0);
    assert_eq!(measure_len(&content, Axis::Vertical), 20.0);
}

/// `gap: 10px 30px`（`row-gap column-gap` の 2 値）でも、横方向は
/// `column-gap`（30px）・縦方向は `row-gap`（10px）を読む。shorthand の
/// 単一 px トークンだけを受理していた旧実装は 0 を返していた（Cursor
/// Bugbot 指摘）。
#[wasm_bindgen_test]
fn read_gap_px_reads_axis_longhand_from_two_value_shorthand() {
    let document = document();
    let el = div_with_style(&document, "display: flex; gap: 10px 30px;");
    let _guard = RemoveOnDrop(el.clone());
    document.body().unwrap().append_child(&el).unwrap();

    assert_eq!(read_gap_px(&html(&el), Axis::Horizontal), 30.0);
    assert_eq!(read_gap_px(&html(&el), Axis::Vertical), 10.0);
}

/// `gap` 未指定（computed value `normal`）は 0 へ fail-safe する。
#[wasm_bindgen_test]
fn read_gap_px_is_zero_for_normal() {
    let document = document();
    let el = div_with_style(&document, "display: flex;");
    let _guard = RemoveOnDrop(el.clone());
    document.body().unwrap().append_child(&el).unwrap();

    assert_eq!(read_gap_px(&html(&el), Axis::Horizontal), 0.0);
}

/// 選択済み radio を含む content を追加複製しても、複製内の radio は
/// `name` を持たず（グループから隔離）、元の radio の選択が保たれる
/// （codex-review P1 指摘: `cloneNode(true)` が checked を複製し、挿入時の
/// グループ同期で元の選択が解除されていた）。
#[wasm_bindgen_test]
fn ensure_copies_isolates_radio_groups_in_clones() {
    let document = document();
    let (root, content) = build_marquee(
        &document,
        "display: flex; width: 100px; overflow: hidden;",
        "display: flex; width: 40px; height: 20px;",
    );
    let _guard = RemoveOnDrop(root.clone());
    let radio = document
        .create_element("input")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    radio.set_type("radio");
    radio.set_name("choice");
    radio.set_checked(true);
    content.append_child(&radio).unwrap();
    // 既存 SSR 複製側にも同名 radio があると本テストの前提が崩れるため、
    // 複製テンプレート（content）だけに radio を持たせる。

    ensure_copies(&root, &content, 4);

    assert_eq!(root.child_element_count(), 4);
    assert!(
        radio.checked(),
        "元の radio の選択が追加複製の挿入で解除されてはならない"
    );
    let clones = root.query_selector_all("input[type=radio]").unwrap();
    let mut named = 0;
    for i in 0..clones.length() {
        let el = clones.item(i).unwrap().dyn_into::<Element>().unwrap();
        if el.has_attribute("name") {
            named += 1;
        }
    }
    assert_eq!(named, 1, "name を持つ radio は元の 1 個だけであるはず");
}

/// 追加複製内の入れ子 ticker（`[data-fandhe-ticker]`）は
/// `data-fandhe-ticker-active` を持ち、offset var が `0px` に固定される
/// （静的化。CSS `@keyframes` でも独立 rAF でも駆動されない）。
#[wasm_bindgen_test]
fn ensure_copies_neutralizes_nested_tickers_in_clones() {
    let document = document();
    let (root, content) = build_marquee(
        &document,
        "display: flex; width: 100px; overflow: hidden;",
        "display: flex; width: 40px; height: 20px;",
    );
    let _guard = RemoveOnDrop(root.clone());
    let inner = div_with_style(&document, "display: block; width: 20px; height: 20px;");
    inner.set_attribute("data-fandhe-ticker", "").unwrap();
    // 元の内側 ticker が JS 駆動中（active + 途中の offset）の状態を模す。
    inner
        .set_attribute("data-fandhe-ticker-active", "")
        .unwrap();
    html(&inner)
        .style()
        .set_property("--fandhe-marquee-ticker-offset", "-7px")
        .unwrap();
    content.append_child(&inner).unwrap();

    ensure_copies(&root, &content, 4);

    let nested = root.query_selector_all("[data-fandhe-ticker]").unwrap();
    // 元 1 個 + 追加複製 2 個（既存 SSR 複製は inner 追加前に作った）。
    assert_eq!(nested.length(), 3);
    for i in 0..nested.length() {
        let el = nested.item(i).unwrap().dyn_into::<Element>().unwrap();
        if el.is_same_node(Some(&inner)) {
            continue;
        }
        assert!(el.has_attribute("data-fandhe-ticker-active"));
        assert_eq!(
            html(&el)
                .style()
                .get_property_value("--fandhe-marquee-ticker-offset")
                .unwrap(),
            "0px",
            "複製内の入れ子 ticker は offset 0px に固定されるはず"
        );
    }
    assert_eq!(
        html(&inner)
            .style()
            .get_property_value("--fandhe-marquee-ticker-offset")
            .unwrap(),
        "-7px",
        "元の内側 ticker の offset は変更しない"
    );
}

/// フォーム内 ticker の追加複製に含まれるフォーム部品は `disabled` になり、
/// `required` 未入力の複製が `form.checkValidity()` を偽にせず、
/// `new FormData(form)` の同名エントリも元の 1 件のままであること
/// （codex-review P1 指摘: `inert`/`aria-hidden` は送信・制約検証の
/// 除外条件ではない）。
#[wasm_bindgen_test]
fn ensure_copies_disables_form_controls_in_clones() {
    let document = document();
    let form = document
        .create_element("form")
        .unwrap()
        .dyn_into::<HtmlFormElement>()
        .unwrap();
    let _guard = RemoveOnDrop(form.clone().into());
    document.body().unwrap().append_child(&form).unwrap();
    let root = div_with_style(&document, "display: flex; width: 100px; overflow: hidden;");
    let content = div_with_style(&document, "display: flex; width: 40px; height: 20px;");
    content.set_attribute("data-part", "content").unwrap();
    // 既存 SSR 複製は入力欄を持たない状態で先に作る（SSR 側の複製は別
    // 契約）。複製テンプレート（content）だけに required 入力欄を持たせ、
    // 元の値を入れておく。追加複製は `cloneNode` 時点の属性値（value
    // 属性なし）で生成されるため、複製の required 欄は未入力になる。
    let ssr_copy = content.clone_node_with_deep(true).unwrap();
    let input = document
        .create_element("input")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    input.set_type("text");
    input.set_name("note");
    input.set_required(true);
    input.set_value("filled");
    content.append_child(&input).unwrap();
    root.append_child(&content).unwrap();
    root.append_child(&ssr_copy).unwrap();
    form.append_child(&root).unwrap();

    ensure_copies(&root, &content, 4);

    assert_eq!(root.child_element_count(), 4);
    assert!(!input.disabled(), "元の入力欄は disabled にしない");
    assert!(
        form.check_validity(),
        "追加複製の required 未入力欄が制約検証を阻んではならない"
    );
    let data = FormData::new_with_form(&form).unwrap();
    assert_eq!(
        data.get_all("note").length(),
        1,
        "同名の送信エントリは元の 1 件だけであるはず"
    );
    let disabled = root.query_selector_all("input[disabled]").unwrap();
    assert_eq!(
        disabled.length(),
        2,
        "追加複製 2 個分の入力欄が disabled になるはず"
    );
}

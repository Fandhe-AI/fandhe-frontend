//! anchor positioning（`fandhe_frontend_wasm_full::position`、イシュー #590、
//! 親 #588）の実ブラウザ統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `crates/wasm-full/src/position.rs` の native 単体テスト（`#[cfg(test)]`）は
//! DOM 非依存の純粋ロジック層（`PositionedKind`/`data-side`/`data-align` の
//! fail-closed パース/`resolve_position`）までを検証済みである。本ファイルは
//! その先、`position::PositionController`（`#[cfg(target_arch = "wasm32")]`
//! 配線層）が実 DOM 上で `getBoundingClientRect`/`window` の実測値を読み、
//! positioner 要素へ `style`（`--fandhe-*` CSS 変数）・`data-side`/
//! `data-align` を実際に反映することを検証する。
//!
//! フィクスチャの HTML はすべて `fandhe-frontend-headless-ui` の Popover
//! 自由関数 + `fandhe_frontend_core::render`（既定エスケープ）で組み立て、
//! `format!` 等による HTML 文字列直接組み立て・`raw_html()` は使用しない
//! （`.claude/rules/coding-rust.md`）。
//!
//! # 検証観点
//!
//! (a) `reposition_now()` 呼び出し後、開いている positioner（
//!     `data-state="open"`）へ `style` 属性が反映され `--fandhe-x`/
//!     `--fandhe-y`/`--fandhe-reference-width` を含む
//! (b) `data-side`/`data-align` 属性が [`fandhe_frontend_headless_ui::Placement`]
//!     の語彙のいずれかへ書き換わる（欠落時の既定 `bottom`/`center` を含む）
//! (c) 閉じている positioner（`data-state` が `"open"` でない）は対象外
//!     （スキップされ `style` 属性が付与されない）
//! (d) `PositionController::new` → `Drop` の対称性（scroll/resize リスナー
//!     登録・解除、`overlay_close_browser.rs` と同じ回帰観点）が panic せず
//!     完走する

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::popover;
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_wasm_full::position::PositionController;
use wasm_bindgen_test::*;
use web_sys::{Document, Element};

wasm_bindgen_test_configure!(run_in_browser);

/// テスト用のプレースホルダ要素を document body へ 1 個生成する
/// （`overlay_close_browser.rs::create_placeholder` と同じ意図）。
fn create_placeholder(document: &Document, id: &str) -> Element {
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

/// テスト末尾でプレースホルダを document から確実に除去する RAII ガード
/// （`overlay_close_browser.rs::RemoveOnDrop` と同じ意図。テスト間 DOM 汚染
/// 対策）。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// 単一の Popover（trigger + anchor + positioner + content、open 状態）を
/// `container` 配下へ展開し、positioner 要素を返す。
fn mount_open_popover(document: &Document, container: &Element, id_prefix: &str) -> Element {
    let positioner_id = format!("{id_prefix}-positioner");
    let html = render(&popover::root(
        OpenState::Open,
        vec![],
        vec![
            popover::trigger(OpenState::Open, false, None, vec![], vec![]),
            popover::anchor(vec![], vec![]),
            popover::positioner(
                OpenState::Open,
                vec![("id", positioner_id.as_str())],
                vec![popover::content(
                    OpenState::Open,
                    None,
                    None,
                    None,
                    vec![],
                    vec![],
                )],
            ),
        ],
    ));
    container.set_inner_html(&html);
    document
        .get_element_by_id(&positioner_id)
        .expect("positioner element must exist")
}

#[wasm_bindgen_test]
fn reposition_now_sets_style_and_placement_attrs_on_open_positioner() {
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let container = create_placeholder(&document, "position-browser-open");
    let _guard = RemoveOnDrop(container.clone());

    let positioner = mount_open_popover(&document, &container, "position-browser-open");
    assert!(positioner.get_attribute("style").is_none());

    let controller =
        PositionController::new(&window).expect("PositionController::new must succeed");
    controller.reposition_now();

    let style = positioner
        .get_attribute("style")
        .expect("open positioner must receive a style attribute after reposition_now");
    assert!(style.contains("--fandhe-x:"));
    assert!(style.contains("--fandhe-y:"));
    assert!(style.contains("--fandhe-reference-width:"));

    // positioner に data-side/data-align が付与されていなかったため
    // 既定（bottom/center）へフォールバックする（fail-closed、
    // `position::parse_side_attr`/`parse_align_attr` の native テストと
    // 同じ契約をブラウザ経路でも確認する）。
    assert_eq!(
        positioner.get_attribute("data-side").as_deref(),
        Some("bottom")
    );
    assert_eq!(
        positioner.get_attribute("data-align").as_deref(),
        Some("center")
    );

    drop(controller);
}

#[wasm_bindgen_test]
fn reposition_now_skips_closed_positioner() {
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let container = create_placeholder(&document, "position-browser-closed");
    let _guard = RemoveOnDrop(container.clone());

    let positioner_id = "position-browser-closed-positioner";
    let html = render(&popover::root(
        OpenState::Closed,
        vec![],
        vec![
            popover::trigger(OpenState::Closed, false, None, vec![], vec![]),
            popover::anchor(vec![], vec![]),
            popover::positioner(
                OpenState::Closed,
                vec![("id", positioner_id)],
                vec![popover::content(
                    OpenState::Closed,
                    None,
                    None,
                    None,
                    vec![],
                    vec![],
                )],
            ),
        ],
    ));
    container.set_inner_html(&html);
    let positioner = document
        .get_element_by_id(positioner_id)
        .expect("positioner element must exist");

    let controller =
        PositionController::new(&window).expect("PositionController::new must succeed");
    controller.reposition_now();

    assert!(
        positioner.get_attribute("style").is_none(),
        "closed positioner (data-state != \"open\") must not be repositioned"
    );

    drop(controller);
}

#[wasm_bindgen_test]
fn controller_new_and_drop_are_symmetric_and_do_not_panic() {
    // `overlay_close_browser.rs` の登録・解除対称性回帰と同じ観点:
    // PositionController を繰り返し生成・破棄しても panic しない
    // （scroll/resize リスナーの Closure::forget を使わず対称的に解除する
    // 設計、position.rs::wiring::PositionController::drop 参照）。
    let window = web_sys::window().expect("window must exist in browser test environment");
    for _ in 0..3 {
        let controller =
            PositionController::new(&window).expect("PositionController::new must succeed");
        controller.reposition_now();
        drop(controller);
    }
}

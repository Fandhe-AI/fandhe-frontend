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
//!     `--fandhe-y` を含む（Popover フィクスチャは
//!     `same_width_default() == false` のため `--fandhe-reference-width`
//!     は含まないことも併せて検証する）
//! (b) `data-side`/`data-align` 属性が [`fandhe_frontend_headless_ui::Placement`]
//!     の語彙のいずれかへ書き換わる（欠落時の既定 `bottom`/`center` を含む）
//! (c) 閉じている positioner（`data-state` が `"open"` でない）は対象外
//!     （スキップされ `style` 属性が付与されない）
//! (d) `PositionController::new` → `Drop` の対称性（scroll/resize リスナー
//!     登録・解除、`overlay_close_browser.rs` と同じ回帰観点）が panic せず
//!     完走する
//!
//! 以下 (e)〜(j) はイシュー #645（親 #588 クローズコメントで追跡が要請された
//! 残課題）で追加した検証観点。(a)〜(d) までは Popover 中心だったのに対し、
//! Menu/Select/Tooltip の配線（`PositionedKind::has_arrow`/
//! `same_width_default` の分岐・希望 placement の永続化・scroll/resize 契機の
//! 再配置）が実 DOM 経路（`getBoundingClientRect` 実測 → `--fandhe-*` CSS 変数・
//! `data-side`/`data-align` 反映）でも成立することを固定する。
//!
//! (e) Menu の positioner は `same_width_default() == true` のため
//!     `--fandhe-reference-width` を含み、`has_arrow() == true` のため
//!     arrow 要素にも `style`（`--fandhe-arrow-x`/`--fandhe-arrow-y`）が
//!     複製される
//! (f) Select の positioner も `--fandhe-reference-width` を含み、その値は
//!     anchor（trigger）の実測幅と一致する。Select は
//!     `has_arrow() == false` のため、scope 内にデコイの arrow 要素を
//!     置いても `style` が複製されない（fail-closed 契約の実 DOM 確認）
//! (g) Tooltip の positioner は `same_width_default() == false` のため
//!     `--fandhe-reference-width` を含まないが、`has_arrow() == true` の
//!     ため arrow 要素へ `style` が複製される（イシュー #622 レビュー指摘:
//!     tooltip positioner の `data-state` 出力漏れで再計算対象から漏れて
//!     いた回帰の実ブラウザ固定）
//! (h) `attrs` 経由で渡した希望 placement（`data-side`/`data-align`）が
//!     `reposition_now()` 後も維持され、`data-requested-side`/
//!     `data-requested-align` へ永続化される
//! (i) trigger の位置を書き換えたうえで合成 `resize`/`scroll` イベントを
//!     `window.dispatch_event` すると、`PositionController` のリスナー配線
//!     経由で `--fandhe-x` が新しい anchor 位置へ再計算される
//! (j) Menu の item ラベル・Select の item_text・Tooltip の content・属性値へ
//!     XSS ペイロードを既定エスケープ経由（`fandhe_frontend_core::text`）で
//!     渡しても、位置決め配線（`set_dom_attribute` の `style`/`data-*` 書き
//!     込み）が既定エスケープ保証を弱めないこと（REQ-1 の位置決め経路への
//!     拡張回帰）
//!
//! 以下 (k) はイシュー #663（`--fandhe-x`/`--fandhe-y`/`--fandhe-arrow-*`
//! 位置ジオメトリの消費）で追加した検証観点。
//!
//! (k) `reposition_now()` 呼び出し前は positioner に `data-positioned`
//!     マーカーが存在せず、呼び出し後に付与される。閉じている positioner
//!     （対象外・no-op）にはマーカーが付与されない
//!     （`crates/pre-styled-ui/src/menu.rs`・`select.rs` の CSS 切り替え
//!     契約の前提となる、`docs/design/anchor-positioning-design.md` §4.4b
//!     参照）
//!
//! 以下 (l)〜(o) はイシュー #1182（`PositionedKind` scope enum への
//! navigation-menu / menubar 追加、出典 PR #1177 の out-of-scope 節）で
//! 追加した検証観点。
//!
//! (l) Menubar は `same_width_default() == true`（`--fandhe-reference-width`
//!     を含む）かつ `has_arrow() == false`（arrow 要素へ `style` を複製
//!     しない。headless-ui `menubar` モジュールの anatomy が Arrow/
//!     ArrowTip を意図的スコープ外とする契約の実 DOM 確認）
//! (m) menubar の trigger 2 個のうち 2 個目のメニューのみを開いた状態で
//!     `reposition_now()` を呼ぶと、`--fandhe-x` が 2 個目の trigger の
//!     実測位置に対応し、1 個目（先頭）の trigger へ誤ってアンカーされ
//!     ない（`find_menubar_anchor` の回帰、イシュー #622 の誤 anchor
//!     指摘と同型の問題への対処）
//! (n) navigation-menu のマークアップ（headless-ui が `positioner` パーツ
//!     を出力しない、open な content を含む）に対し `reposition_now()` が
//!     panic せず、いかなる要素にも `style`/`data-positioned` を付与しない
//!     （headless 層が positioner を出力しない現状では配線が発火しない
//!     前方互換の挙動を固定する）
//! (o) menubar の item 値・trigger ラベルへ XSS ペイロードを既定エスケープ
//!     経由で渡しても、位置決め配線が既定エスケープ保証を弱めないこと
//!     （REQ-1 拡張回帰、検証観点 (j) の menubar 版）

#![cfg(target_arch = "wasm32")]

use fandhe_frontend_core::{el, render, text};
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_headless_ui::Orientation;
use fandhe_frontend_headless_ui::{menu, menubar, navigation_menu, popover, select, tooltip};
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

/// フィクスチャの floating（positioner）要素へ与える固定サイズ用の CSS
/// クラス名。
///
/// `headless-ui` の positioner はスタイル不可知（headless）のため幅を
/// 持たず、実運用では pre-styled-ui/利用者 CSS が `width`/`height` を
/// 与える前提である。一方
/// `crates/wasm-full/src/position.rs::wiring::reposition_one` は
/// `style` 属性を再計算のたびに `--fandhe-*` CSS 変数のみへ**完全上書き**
/// する契約（既存の author スタイルとマージしない）ため、floating 要素の
/// `style` 属性へ直接 `width`/`height` を書いても最初の `reposition_now()`
/// 呼び出しで消えてしまい、2 回目以降の `getBoundingClientRect()` が
/// 無指定幅（親コンテナ全幅相当）を返してしまう（イシュー #645 の実
/// ブラウザ検証で判明: resize/scroll 契機の再計算テストで anchor 移動後の
/// 座標が shift クランプにより極小値へ潰れる偽陽性の原因だった）。
/// `reposition_one` は `class` 属性には触れないため、`<style>` 経由の
/// クラス指定で幅/高さを与えることで、何度再計算されても floating の
/// 実測値が安定する。
const FIXED_FLOATING_SIZE_CLASS: &str = "position-browser-fixed-floating-size";

/// [`FIXED_FLOATING_SIZE_CLASS`] の定義を `document.head` へ 1 度だけ挿入
/// する（`get_element_by_id` で冪等性を確保し、複数フィクスチャから呼ばれ
/// ても重複挿入しない）。
fn ensure_fixed_floating_size_stylesheet(document: &Document) {
    const STYLE_ELEMENT_ID: &str = "position-browser-fixed-floating-size-style";
    if document.get_element_by_id(STYLE_ELEMENT_ID).is_some() {
        return;
    }
    let style = document
        .create_element("style")
        .expect("create_element must not fail for a style element");
    style.set_id(STYLE_ELEMENT_ID);
    style.set_text_content(Some(&format!(
        ".{FIXED_FLOATING_SIZE_CLASS} {{ width: 100px; height: 50px; }}"
    )));
    document
        .head()
        .expect("document head must exist in browser test environment")
        .append_child(&style)
        .expect("append_child must not fail for a style element");
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
    // イシュー #663: reposition 前は data-positioned マーカーが存在しない
    // （SSR 静的フォールバックのまま、wasm 未稼働と区別が付かない状態）。
    assert!(positioner.get_attribute("data-positioned").is_none());

    let controller =
        PositionController::new(&window).expect("PositionController::new must succeed");
    controller.reposition_now();

    // イシュー #663: reposition_now() 後は data-positioned マーカーが
    // 付与され、pre-styled-ui 側の CSS が fixed 座標系へ切り替わる。
    assert_eq!(
        positioner.get_attribute("data-positioned").as_deref(),
        Some("")
    );

    let style = positioner
        .get_attribute("style")
        .expect("open positioner must receive a style attribute after reposition_now");
    assert!(style.contains("--fandhe-x:"));
    assert!(style.contains("--fandhe-y:"));
    // Popover は `PositionedKind::same_width_default()` が `false` のため
    // `css_vars_style` は `--fandhe-reference-width` を出力しない契約
    // （イシュー #622 レビュー指摘の回帰、native 側の
    // `same_width_default_true_for_menu_and_select_only`/
    // `resolve_position_includes_reference_width_for_menu_and_select_only`
    // と同じ契約をブラウザ経路でも確認する）。
    assert!(!style.contains("--fandhe-reference-width:"));

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

/// サブメニュー（`trigger-item` が anchor）1 個を `container` 配下へ展開し、
/// 子 Menu の positioner 要素を返す（イシュー #622 レビュー指摘の回帰:
/// `find_anchor` が `trigger-item` を anchor フォールバックへ含めることを
/// 確認する。親 Menu の `content` 内に子 Menu インスタンス由来の
/// `trigger_item`/`positioner`/`content` を入れ子で配置する構成は
/// `menu.rs` doc の「サブメニュー」契約どおり）。
fn mount_open_submenu(document: &Document, container: &Element, id_prefix: &str) -> Element {
    let positioner_id = format!("{id_prefix}-positioner");
    let html = render(&menu::root(
        OpenState::Open,
        vec![],
        vec![
            menu::trigger_item(OpenState::Open, false, false, None, vec![], vec![]),
            menu::positioner(
                OpenState::Open,
                vec![("id", positioner_id.as_str())],
                vec![menu::content(OpenState::Open, None, None, vec![], vec![])],
            ),
        ],
    ));
    container.set_inner_html(&html);
    document
        .get_element_by_id(&positioner_id)
        .expect("positioner element must exist")
}

#[wasm_bindgen_test]
fn reposition_now_resolves_trigger_item_as_anchor_for_submenu() {
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let container = create_placeholder(&document, "position-browser-submenu");
    let _guard = RemoveOnDrop(container.clone());

    let positioner = mount_open_submenu(&document, &container, "position-browser-submenu");
    assert!(positioner.get_attribute("style").is_none());

    let controller =
        PositionController::new(&window).expect("PositionController::new must succeed");
    controller.reposition_now();

    let style = positioner.get_attribute("style").expect(
        "submenu positioner must receive a style attribute after reposition_now \
         (find_anchor must resolve [data-part=\"trigger-item\"] as the anchor)",
    );
    assert!(style.contains("--fandhe-x:"));
    assert!(style.contains("--fandhe-y:"));

    drop(controller);
}

/// `style` 属性の値から `--fandhe-x` の数値（px 前の部分）を取り出す。
/// 座標の大小関係で「どの anchor 要素が使われたか」を判定するための
/// テスト専用ヘルパー（実装側の書式契約には依存しない緩い抽出）。
fn extract_fandhe_x(style: &str) -> f64 {
    let after = style
        .split("--fandhe-x:")
        .nth(1)
        .expect("style must contain --fandhe-x:");
    let number_part = after.split("px").next().expect("value must end with px");
    number_part
        .trim()
        .parse::<f64>()
        .expect("--fandhe-x value must be a valid number")
}

/// コンテキストメニュー（`context-trigger` が自身の anchor）配下に、
/// サブメニュー（入れ子の `Menu` インスタンス。`trigger-item` を自身の
/// anchor とする別の scope root）を 1 個ネストして `container` 配下へ
/// 展開する（イシュー #622 Bugbot 指摘の回帰: `find_anchor` が
/// `query_selector` の descendant マッチにより、外側 scope root
/// （コンテキストメニュー自身）の探索でネストしたサブメニューの
/// `trigger-item` を自身の `context-trigger` より先に拾ってしまい、
/// 外側 positioner が誤った座標で位置決めされていた）。
///
/// `context-trigger`（`context_trigger_left_px`）とネストした
/// `trigger-item`（`nested_trigger_item_left_px`）を大きく離れた
/// `left` へ `position: fixed` で固定し、外側 positioner の
/// `--fandhe-x` がどちらの矩形に由来するかを座標値の大小で判別できる
/// ようにする。返り値は `(外側 positioner, 内側/サブメニューの positioner)`。
fn mount_open_context_menu_with_nested_submenu(
    document: &Document,
    container: &Element,
    id_prefix: &str,
) -> (Element, Element) {
    ensure_fixed_floating_size_stylesheet(document);
    let outer_positioner_id = format!("{id_prefix}-outer-positioner");
    let inner_positioner_id = format!("{id_prefix}-inner-positioner");
    let context_trigger_style = "position: fixed; left: 5px; top: 5px; width: 10px; height: 10px;";
    let nested_trigger_item_style =
        "position: fixed; left: 500px; top: 500px; width: 10px; height: 10px;";
    let html = render(&menu::root(
        OpenState::Open,
        vec![],
        vec![
            menu::context_trigger(
                OpenState::Open,
                vec![("style", context_trigger_style)],
                vec![],
            ),
            menu::positioner(
                OpenState::Open,
                // floating 要素に固定サイズを与える理由は
                // [`FIXED_FLOATING_SIZE_CLASS`] のドキュメントを参照
                // （インライン `style` ではなく `class` を使うのは
                // `reposition_one` が `style` 属性を再計算のたびに
                // 完全上書きするため）。
                vec![
                    ("id", outer_positioner_id.as_str()),
                    ("class", FIXED_FLOATING_SIZE_CLASS),
                ],
                vec![menu::content(
                    OpenState::Open,
                    None,
                    None,
                    vec![],
                    vec![
                        // ネストしたサブメニュー本体（子 Menu インスタンス）。
                        // 自身の `data-part="root"` を持つため、
                        // `find_scope_root`/`find_anchor` はこの `root` を
                        // 越えて外側の `context-trigger` を誤って拾わない
                        // ことが期待される（逆方向の回帰も同時に確認する）。
                        menu::root(
                            OpenState::Open,
                            vec![],
                            vec![
                                menu::trigger_item(
                                    OpenState::Open,
                                    false,
                                    false,
                                    None,
                                    vec![("style", nested_trigger_item_style)],
                                    vec![],
                                ),
                                menu::positioner(
                                    OpenState::Open,
                                    // 上の外側 positioner と同じ理由
                                    // （[`FIXED_FLOATING_SIZE_CLASS`]
                                    // 参照）で `class` により固定サイズ
                                    // を与える。
                                    vec![
                                        ("id", inner_positioner_id.as_str()),
                                        ("class", FIXED_FLOATING_SIZE_CLASS),
                                    ],
                                    vec![menu::content(
                                        OpenState::Open,
                                        None,
                                        None,
                                        vec![],
                                        vec![],
                                    )],
                                ),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    ));
    container.set_inner_html(&html);
    let outer_positioner = document
        .get_element_by_id(&outer_positioner_id)
        .expect("outer positioner element must exist");
    let inner_positioner = document
        .get_element_by_id(&inner_positioner_id)
        .expect("inner (submenu) positioner element must exist");
    (outer_positioner, inner_positioner)
}

#[wasm_bindgen_test]
fn reposition_now_anchors_context_menu_to_its_own_context_trigger_not_nested_submenu_trigger_item()
{
    // イシュー #622 Bugbot 指摘（High Severity）の回帰: サブメニューを
    // 含むコンテキストメニューで `find_anchor` が入れ子の `trigger-item`
    // を自身の `context-trigger` より先に拾い、外側 positioner が誤った
    // `--fandhe-*` 座標で位置決めされていた。
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let container = create_placeholder(&document, "position-browser-context-menu-submenu");
    let _guard = RemoveOnDrop(container.clone());

    let (outer_positioner, inner_positioner) =
        mount_open_context_menu_with_nested_submenu(&document, &container, "ctx-submenu");

    let controller =
        PositionController::new(&window).expect("PositionController::new must succeed");
    controller.reposition_now();

    let outer_style = outer_positioner.get_attribute("style").expect(
        "outer (context menu) positioner must receive a style attribute after reposition_now",
    );
    let inner_style = inner_positioner
        .get_attribute("style")
        .expect("inner (submenu) positioner must receive a style attribute after reposition_now");

    let outer_x = extract_fandhe_x(&outer_style);
    let inner_x = extract_fandhe_x(&inner_style);

    // 外側 positioner は `context-trigger`（left: 5px）由来の小さい座標を
    // 使うべきで、ネストした `trigger-item`（left: 500px）由来の大きい
    // 座標を使ってはならない。
    assert!(
        outer_x < 100.0,
        "outer positioner must anchor to context-trigger (left: 5px), not the nested \
         trigger-item (left: 500px); got --fandhe-x: {outer_x}"
    );
    // 内側（サブメニュー）positioner は自身の `trigger-item`
    // （left: 500px）由来の大きい座標を使うべき（外側と逆方向の回帰、
    // `find_scope_root`/`find_anchor` のスコープ越えが起きていないことの
    // 確認）。
    assert!(
        inner_x > 400.0,
        "inner (submenu) positioner must anchor to its own trigger-item (left: 500px); \
         got --fandhe-x: {inner_x}"
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
    // イシュー #663: 閉じた positioner は data-positioned マーカーも
    // 付与されない（SSR 静的フォールバックのまま、fixed 座標系へ切り替わらない）。
    assert!(
        positioner.get_attribute("data-positioned").is_none(),
        "closed positioner must not receive the data-positioned marker"
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

// ---------------------------------------------------------------------
// イシュー #645（親 #588 残課題）: Menu/Select/Tooltip の配線検証。
// 冒頭 doc の検証観点 (e)〜(j) 参照。
// ---------------------------------------------------------------------

/// `style` 属性の値から任意の `--fandhe-*` CSS 変数の数値（px 前の部分）を
/// 取り出す（[`extract_fandhe_x`] の汎用版。実装側の書式契約
/// （`"name: 123.4px;"`）に依存しない緩い抽出とし、変数名の前後の空白の
/// 有無に関わらず値を取得できるようにする）。
fn extract_css_var_px(style: &str, var_name: &str) -> f64 {
    let marker = format!("{var_name}:");
    let after = style
        .split(marker.as_str())
        .nth(1)
        .unwrap_or_else(|| panic!("style must contain {marker}: {style}"));
    let number_part = after.split("px").next().expect("value must end with px");
    number_part
        .trim()
        .parse::<f64>()
        .expect("css var value must be a valid number")
}

/// 単一の Menu（trigger + positioner + content + item + arrow、open 状態）を
/// `container` 配下へ展開し、`(trigger, positioner, arrow)` を返す。
/// `trigger_style` は anchor 矩形を決定的にするための inline style
/// （`position: fixed` で viewport 上の位置・サイズを固定する）。
fn mount_open_menu_with_arrow(
    document: &Document,
    container: &Element,
    id_prefix: &str,
    trigger_style: &str,
) -> (Element, Element, Element) {
    let trigger_id = format!("{id_prefix}-trigger");
    let positioner_id = format!("{id_prefix}-positioner");
    let arrow_id = format!("{id_prefix}-arrow");
    let html = render(&menu::root(
        OpenState::Open,
        vec![],
        vec![
            menu::trigger(
                OpenState::Open,
                false,
                None,
                vec![("id", trigger_id.as_str()), ("style", trigger_style)],
                vec![],
            ),
            menu::positioner(
                OpenState::Open,
                vec![("id", positioner_id.as_str())],
                vec![
                    menu::content(
                        OpenState::Open,
                        None,
                        None,
                        vec![],
                        vec![menu::item(
                            "item-1",
                            false,
                            false,
                            vec![],
                            vec![text("Item 1")],
                        )],
                    ),
                    menu::arrow(vec![("id", arrow_id.as_str())], vec![]),
                ],
            ),
        ],
    ));
    container.set_inner_html(&html);
    let trigger = document
        .get_element_by_id(&trigger_id)
        .expect("trigger element must exist");
    let positioner = document
        .get_element_by_id(&positioner_id)
        .expect("positioner element must exist");
    let arrow = document
        .get_element_by_id(&arrow_id)
        .expect("arrow element must exist");
    (trigger, positioner, arrow)
}

#[wasm_bindgen_test]
fn reposition_now_sets_reference_width_and_arrow_style_for_menu() {
    // Menu は `PositionedKind::same_width_default() == true` かつ
    // `has_arrow() == true`（native 側の
    // `same_width_default_true_for_menu_and_select_only`/
    // `only_select_lacks_arrow` と同じ契約をブラウザ経路でも確認する）。
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let container = create_placeholder(&document, "position-browser-menu-arrow");
    let _guard = RemoveOnDrop(container.clone());

    let (_trigger, positioner, arrow) = mount_open_menu_with_arrow(
        &document,
        &container,
        "position-browser-menu-arrow",
        "position: fixed; left: 10px; top: 10px; width: 40px; height: 20px;",
    );

    let controller =
        PositionController::new(&window).expect("PositionController::new must succeed");
    controller.reposition_now();

    let style = positioner
        .get_attribute("style")
        .expect("Menu positioner must receive a style attribute after reposition_now");
    assert!(style.contains("--fandhe-x:"));
    assert!(style.contains("--fandhe-y:"));
    assert!(
        style.contains("--fandhe-reference-width:"),
        "Menu (same_width_default() == true) must output --fandhe-reference-width: {style}"
    );

    let arrow_style = arrow.get_attribute("style").expect(
        "Menu arrow element must receive a style attribute after reposition_now \
         (has_arrow() == true, find_arrow must resolve [data-part=\"arrow\"])",
    );
    assert!(arrow_style.contains("--fandhe-arrow-x:"));
    assert!(arrow_style.contains("--fandhe-arrow-y:"));

    drop(controller);
}

/// 単一の Select（control + trigger + positioner + content + item、open
/// 状態）を `container` 配下へ展開する。`decoy_arrow` として
/// `data-part="arrow"` のデコイ要素を positioner 配下へ追加し、
/// `PositionedKind::Select::has_arrow() == false` の fail-closed 契約
/// （`style` が付与されないこと）をブラウザ経路で確認できるようにする。
/// `(trigger, positioner, decoy_arrow)` を返す。
fn mount_open_select_with_decoy_arrow(
    document: &Document,
    container: &Element,
    id_prefix: &str,
    trigger_style: &str,
) -> (Element, Element, Element) {
    ensure_fixed_floating_size_stylesheet(document);
    let trigger_id = format!("{id_prefix}-trigger");
    let positioner_id = format!("{id_prefix}-positioner");
    let decoy_arrow_id = format!("{id_prefix}-decoy-arrow");
    let select_props = select::SelectProps::default();
    let html = render(&select::root(
        OpenState::Open,
        &select_props,
        vec![],
        vec![
            select::control(
                OpenState::Open,
                &select_props,
                vec![],
                vec![select::trigger(
                    OpenState::Open,
                    &select_props,
                    false,
                    None,
                    None,
                    vec![("id", trigger_id.as_str()), ("style", trigger_style)],
                    vec![],
                )],
            ),
            select::positioner(
                OpenState::Open,
                // floating 要素に固定サイズを与える理由は
                // [`FIXED_FLOATING_SIZE_CLASS`] のドキュメントを参照
                // （インライン `style` ではなく `class` を使うのは
                // `reposition_one` が `style` 属性を再計算のたびに
                // 完全上書きするため）。
                vec![
                    ("id", positioner_id.as_str()),
                    ("class", FIXED_FLOATING_SIZE_CLASS),
                ],
                vec![
                    select::content(
                        OpenState::Open,
                        None,
                        None,
                        None,
                        vec![],
                        vec![select::item(
                            OpenState::Closed,
                            &select_props,
                            false,
                            false,
                            "option-1",
                            None,
                            vec![],
                            vec![select::item_text(
                                OpenState::Closed,
                                &select_props,
                                false,
                                false,
                                None,
                                vec![],
                                vec![text("Option 1")],
                            )],
                        )],
                    ),
                    // headless-ui の select モジュールは arrow パーツを
                    // 提供しない（`has_arrow() == false`）。誤ってマークアップに
                    // 混入した `data-part="arrow"` 要素が repositioning から
                    // 除外されることを確認するためのデコイ。
                    el(
                        "div",
                        vec![("data-part", "arrow"), ("id", decoy_arrow_id.as_str())],
                        vec![],
                    ),
                ],
            ),
        ],
    ));
    container.set_inner_html(&html);
    let trigger = document
        .get_element_by_id(&trigger_id)
        .expect("trigger element must exist");
    let positioner = document
        .get_element_by_id(&positioner_id)
        .expect("positioner element must exist");
    let decoy_arrow = document
        .get_element_by_id(&decoy_arrow_id)
        .expect("decoy arrow element must exist");
    (trigger, positioner, decoy_arrow)
}

#[wasm_bindgen_test]
fn reposition_now_sets_reference_width_matching_anchor_width_for_select_and_skips_decoy_arrow() {
    // Select は `PositionedKind::same_width_default() == true` かつ
    // `has_arrow() == false`（`only_select_lacks_arrow` の native 契約を
    // ブラウザ経路で確認する）。`--fandhe-reference-width` の値が実測した
    // anchor（trigger）幅と一致することも end-to-end で確認する。
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let container = create_placeholder(&document, "position-browser-select-width");
    let _guard = RemoveOnDrop(container.clone());

    let (_trigger, positioner, decoy_arrow) = mount_open_select_with_decoy_arrow(
        &document,
        &container,
        "position-browser-select-width",
        "position: fixed; left: 20px; top: 20px; width: 150px; height: 24px;",
    );

    let controller =
        PositionController::new(&window).expect("PositionController::new must succeed");
    controller.reposition_now();

    let style = positioner
        .get_attribute("style")
        .expect("Select positioner must receive a style attribute after reposition_now");
    assert!(style.contains("--fandhe-x:"));
    assert!(style.contains("--fandhe-y:"));
    assert!(
        style.contains("--fandhe-reference-width:"),
        "Select (same_width_default() == true) must output --fandhe-reference-width: {style}"
    );

    let reference_width = extract_css_var_px(&style, "--fandhe-reference-width");
    assert!(
        (reference_width - 150.0).abs() < 1.0,
        "--fandhe-reference-width must match the anchor (trigger) measured width (150px); \
         got {reference_width}"
    );

    assert!(
        decoy_arrow.get_attribute("style").is_none(),
        "Select (has_arrow() == false) must not receive a style attribute on a decoy \
         [data-part=\"arrow\"] element"
    );

    drop(controller);
}

/// 単一の Tooltip（trigger + positioner + arrow + content、open 状態）を
/// `container` 配下へ展開し、`(positioner, arrow)` を返す。
fn mount_open_tooltip_with_arrow(
    document: &Document,
    container: &Element,
    id_prefix: &str,
) -> (Element, Element) {
    let positioner_id = format!("{id_prefix}-positioner");
    let arrow_id = format!("{id_prefix}-arrow");
    let html = render(&tooltip::root(
        OpenState::Open,
        vec![],
        vec![
            tooltip::trigger(OpenState::Open, false, None, vec![], vec![]),
            tooltip::positioner(
                OpenState::Open,
                vec![("id", positioner_id.as_str())],
                vec![
                    tooltip::arrow(vec![("id", arrow_id.as_str())], vec![]),
                    tooltip::content(OpenState::Open, None, vec![], vec![text("Tip")]),
                ],
            ),
        ],
    ));
    container.set_inner_html(&html);
    let positioner = document
        .get_element_by_id(&positioner_id)
        .expect("positioner element must exist");
    let arrow = document
        .get_element_by_id(&arrow_id)
        .expect("arrow element must exist");
    (positioner, arrow)
}

#[wasm_bindgen_test]
fn reposition_now_sets_style_and_arrow_but_omits_reference_width_for_tooltip() {
    // Tooltip は `PositionedKind::same_width_default() == false`（任意サイズの
    // コンテンツを想定）かつ `has_arrow() == true`。加えて
    // `tooltip::positioner` が `data-state` を出力すること（イシュー #622
    // レビュー指摘: 従来出力されておらず `reposition_all` の
    // `[data-part="positioner"][data-state="open"]` セレクタにマッチせず
    // 再計算対象から漏れていた）の回帰をブラウザ経路で固定する。
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let container = create_placeholder(&document, "position-browser-tooltip-arrow");
    let _guard = RemoveOnDrop(container.clone());

    let (positioner, arrow) =
        mount_open_tooltip_with_arrow(&document, &container, "position-browser-tooltip-arrow");
    assert_eq!(
        positioner.get_attribute("data-state").as_deref(),
        Some("open"),
        "tooltip::positioner must output data-state (issue #622 review regression guard)"
    );

    let controller =
        PositionController::new(&window).expect("PositionController::new must succeed");
    controller.reposition_now();

    let style = positioner
        .get_attribute("style")
        .expect("Tooltip positioner must receive a style attribute after reposition_now");
    assert!(style.contains("--fandhe-x:"));
    assert!(style.contains("--fandhe-y:"));
    assert!(
        !style.contains("--fandhe-reference-width:"),
        "Tooltip (same_width_default() == false) must not output --fandhe-reference-width: {style}"
    );

    let arrow_style = arrow
        .get_attribute("style")
        .expect("Tooltip arrow element must receive a style attribute after reposition_now");
    assert!(arrow_style.contains("--fandhe-arrow-x:"));
    assert!(arrow_style.contains("--fandhe-arrow-y:"));

    drop(controller);
}

#[wasm_bindgen_test]
fn reposition_now_preserves_requested_placement_and_persists_it_across_recalculation() {
    // SSR マークアップの positioner へ最初から data-side="right"/
    // data-align="start" を渡した Menu を展開する。trigger は viewport 左上の
    // 小さい矩形に fixed 固定し、flip が決定的に発生しない配置にする
    // （右側に十分な余白を確保）。
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let container = create_placeholder(&document, "position-browser-menu-placement");
    let _guard = RemoveOnDrop(container.clone());
    ensure_fixed_floating_size_stylesheet(&document);

    let trigger_id = "position-browser-menu-placement-trigger";
    let positioner_id = "position-browser-menu-placement-positioner";
    let html = render(&menu::root(
        OpenState::Open,
        vec![],
        vec![
            menu::trigger(
                OpenState::Open,
                false,
                None,
                vec![
                    ("id", trigger_id),
                    (
                        "style",
                        "position: fixed; left: 5px; top: 100px; width: 10px; height: 10px;",
                    ),
                ],
                vec![],
            ),
            menu::positioner(
                OpenState::Open,
                // floating 要素に固定サイズを与える理由は
                // [`FIXED_FLOATING_SIZE_CLASS`] のドキュメントを参照
                // （インライン `style` ではなく `class` を使うのは
                // `reposition_one` が `style` 属性を再計算のたびに
                // 完全上書きするため）。
                vec![
                    ("id", positioner_id),
                    ("data-side", "right"),
                    ("data-align", "start"),
                    ("class", FIXED_FLOATING_SIZE_CLASS),
                ],
                vec![menu::content(
                    OpenState::Open,
                    None,
                    None,
                    vec![],
                    vec![menu::item(
                        "item-1",
                        false,
                        false,
                        vec![],
                        vec![text("Item 1")],
                    )],
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

    assert_eq!(
        positioner.get_attribute("data-side").as_deref(),
        Some("right"),
        "requested placement (data-side=\"right\") must be honored when there is no need to flip"
    );
    assert_eq!(
        positioner.get_attribute("data-align").as_deref(),
        Some("start"),
        "requested placement (data-align=\"start\") must be honored"
    );
    assert_eq!(
        positioner.get_attribute("data-requested-side").as_deref(),
        Some("right"),
        "the initial data-side must be persisted into data-requested-side \
         (resolve_requested_placement's fallback_side input, issue #622 review fix)"
    );
    assert_eq!(
        positioner.get_attribute("data-requested-align").as_deref(),
        Some("start"),
        "the initial data-align must be persisted into data-requested-align"
    );

    drop(controller);
}

#[wasm_bindgen_test]
fn reposition_all_recalculates_positioner_after_resize_event_moves_the_anchor() {
    // trigger を `position: fixed` で固定した Select を展開し、
    // `reposition_now()` で初回の `--fandhe-x` を記録したうえで、trigger の
    // `style` 属性を書き換えたあと合成 `resize` イベントを
    // `window.dispatch_event` する。`PositionController::new` が登録した
    // resize リスナー（`wiring::PositionController::new` 参照）経由で
    // `reposition_all` が呼ばれ、開いている positioner が新しい anchor 位置へ
    // 再計算されることを確認する（`nav_pagehide_browser.rs` と同じ合成
    // イベント dispatch パターン）。
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let container = create_placeholder(&document, "position-browser-resize-reflow");
    let _guard = RemoveOnDrop(container.clone());

    let (trigger, positioner, _decoy_arrow) = mount_open_select_with_decoy_arrow(
        &document,
        &container,
        "position-browser-resize-reflow",
        "position: fixed; left: 5px; top: 50px; width: 10px; height: 10px;",
    );

    let controller =
        PositionController::new(&window).expect("PositionController::new must succeed");
    controller.reposition_now();

    let initial_style = positioner
        .get_attribute("style")
        .expect("Select positioner must receive a style attribute after the initial reposition");
    let initial_x = extract_fandhe_x(&initial_style);
    assert!(
        initial_x < 100.0,
        "initial --fandhe-x must reflect the anchor fixed at left: 5px; got {initial_x}"
    );

    trigger
        .set_attribute(
            "style",
            "position: fixed; left: 400px; top: 50px; width: 10px; height: 10px;",
        )
        .expect("set_attribute must not fail for a plain style rewrite");

    let resize_event = web_sys::Event::new("resize").expect("Event construction must not fail");
    window
        .dispatch_event(&resize_event)
        .expect("dispatch_event must not fail");

    let updated_style = positioner.get_attribute("style").expect(
        "Select positioner must still have a style attribute after the resize-triggered recalculation",
    );
    let updated_x = extract_fandhe_x(&updated_style);
    assert!(
        updated_x > 300.0,
        "after moving the anchor to left: 400px and dispatching resize, --fandhe-x must be \
         recalculated to reflect the new anchor position; got {updated_x}"
    );

    drop(controller);
}

#[wasm_bindgen_test]
fn reposition_all_recalculates_positioner_after_scroll_event_moves_the_anchor() {
    // resize 版（上記テスト）と対の scroll 契機の回帰確認。scroll リスナーは
    // キャプチャフェーズで登録される（`wiring::PositionController::new` 参照）
    // が、`window.dispatch_event` された合成イベントもキャプチャ登録済み
    // リスナーへ届く（バブリング段階を待たず、ターゲットフェーズで発火する
    // ため）。
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let container = create_placeholder(&document, "position-browser-scroll-reflow");
    let _guard = RemoveOnDrop(container.clone());

    let (trigger, positioner, _decoy_arrow) = mount_open_select_with_decoy_arrow(
        &document,
        &container,
        "position-browser-scroll-reflow",
        "position: fixed; left: 5px; top: 50px; width: 10px; height: 10px;",
    );

    let controller =
        PositionController::new(&window).expect("PositionController::new must succeed");
    controller.reposition_now();

    trigger
        .set_attribute(
            "style",
            "position: fixed; left: 400px; top: 50px; width: 10px; height: 10px;",
        )
        .expect("set_attribute must not fail for a plain style rewrite");

    let scroll_event = web_sys::Event::new("scroll").expect("Event construction must not fail");
    window
        .dispatch_event(&scroll_event)
        .expect("dispatch_event must not fail");

    let updated_style = positioner.get_attribute("style").expect(
        "Select positioner must still have a style attribute after the scroll-triggered recalculation",
    );
    let updated_x = extract_fandhe_x(&updated_style);
    assert!(
        updated_x > 300.0,
        "after moving the anchor to left: 400px and dispatching scroll, --fandhe-x must be \
         recalculated to reflect the new anchor position; got {updated_x}"
    );

    drop(controller);
}

#[wasm_bindgen_test]
fn reposition_now_does_not_weaken_default_escaping_for_menu_select_and_tooltip_content() {
    // REQ-1（既定エスケープ）の位置決め経路への拡張回帰（イシュー #645
    // 検証観点 (j)）: 位置決め配線（`set_dom_attribute` の `style`/`data-*`
    // 書き込み）は positioner/arrow 要素のみを対象とし、他のノード木 API
    // 経由コンテンツの既定エスケープ保証には触れない契約であることを、
    // 実際に XSS ペイロードを含むフィクスチャで確認する
    // （`xss_escape_wasm.rs` のペイロード集合と対応させる）。
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let container = create_placeholder(&document, "position-browser-xss");
    let _guard = RemoveOnDrop(container.clone());

    let script_payload = "<script>alert('fandhe-xss')</script>";
    let img_payload = "<img src=x onerror=alert(1)>";
    let attr_payload = "\" onmouseover=\"alert(1)";

    let menu_trigger_id = "position-browser-xss-menu-trigger";
    let menu_positioner_id = "position-browser-xss-menu-positioner";
    let select_positioner_id = "position-browser-xss-select-positioner";
    let tooltip_positioner_id = "position-browser-xss-tooltip-positioner";

    let html = render(&el(
        "div",
        vec![],
        vec![
            menu::root(
                OpenState::Open,
                vec![],
                vec![
                    menu::trigger(
                        OpenState::Open,
                        false,
                        None,
                        vec![("id", menu_trigger_id), ("data-testid", attr_payload)],
                        vec![],
                    ),
                    menu::positioner(
                        OpenState::Open,
                        vec![("id", menu_positioner_id)],
                        vec![menu::content(
                            OpenState::Open,
                            None,
                            None,
                            vec![],
                            vec![menu::item(
                                "item-1",
                                false,
                                false,
                                vec![],
                                vec![text(script_payload)],
                            )],
                        )],
                    ),
                ],
            ),
            select::root(
                OpenState::Open,
                &select::SelectProps::default(),
                vec![],
                vec![
                    select::control(
                        OpenState::Open,
                        &select::SelectProps::default(),
                        vec![],
                        vec![select::trigger(
                            OpenState::Open,
                            &select::SelectProps::default(),
                            false,
                            None,
                            None,
                            vec![],
                            vec![],
                        )],
                    ),
                    select::positioner(
                        OpenState::Open,
                        vec![("id", select_positioner_id)],
                        vec![select::content(
                            OpenState::Open,
                            None,
                            None,
                            None,
                            vec![],
                            vec![select::item(
                                OpenState::Closed,
                                &select::SelectProps::default(),
                                false,
                                false,
                                "option-1",
                                None,
                                vec![],
                                vec![select::item_text(
                                    OpenState::Closed,
                                    &select::SelectProps::default(),
                                    false,
                                    false,
                                    None,
                                    vec![],
                                    vec![text(img_payload)],
                                )],
                            )],
                        )],
                    ),
                ],
            ),
            tooltip::root(
                OpenState::Open,
                vec![],
                vec![
                    tooltip::trigger(OpenState::Open, false, None, vec![], vec![]),
                    tooltip::positioner(
                        OpenState::Open,
                        vec![("id", tooltip_positioner_id)],
                        vec![tooltip::content(
                            OpenState::Open,
                            None,
                            vec![],
                            vec![text(script_payload)],
                        )],
                    ),
                ],
            ),
        ],
    ));
    container.set_inner_html(&html);

    assert!(
        container
            .query_selector("script")
            .expect("query_selector must not fail")
            .is_none(),
        "no real <script> element must be created from the escaped payload"
    );
    assert!(
        container
            .query_selector("img")
            .expect("query_selector must not fail")
            .is_none(),
        "no real <img> element must be created from the escaped payload"
    );

    let text_content = container.text_content().unwrap_or_default();
    assert!(
        text_content.contains(script_payload),
        "the script payload must survive as literal text content (escaped, not executed): {text_content}"
    );
    assert!(
        text_content.contains(img_payload),
        "the img payload must survive as literal text content (escaped, not executed): {text_content}"
    );

    let menu_trigger = document
        .get_element_by_id(menu_trigger_id)
        .expect("menu trigger element must exist");
    assert_eq!(
        menu_trigger.get_attribute("data-testid").as_deref(),
        Some(attr_payload),
        "the attribute-injection payload must be preserved literally as the data-testid value"
    );
    assert!(
        menu_trigger.get_attribute("onmouseover").is_none(),
        "the attribute-injection payload must not break out into a real onmouseover attribute"
    );

    let menu_positioner = document
        .get_element_by_id(menu_positioner_id)
        .expect("menu positioner element must exist");
    let select_positioner = document
        .get_element_by_id(select_positioner_id)
        .expect("select positioner element must exist");
    let tooltip_positioner = document
        .get_element_by_id(tooltip_positioner_id)
        .expect("tooltip positioner element must exist");

    let controller =
        PositionController::new(&window).expect("PositionController::new must succeed");
    controller.reposition_now();

    // 位置決め配線（style/data-side/data-align 書き込み）が正常に完走し、
    // XSS ペイロードを含むコンテンツの存在が再計算そのものを妨げないこと
    // （position wiring は positioner/arrow の attrs のみを書き込み、既定
    // エスケープ保証を弱める経路にはならない契約の確認）。
    assert!(menu_positioner.get_attribute("style").is_some());
    assert!(select_positioner.get_attribute("style").is_some());
    assert!(tooltip_positioner.get_attribute("style").is_some());

    // reposition_now() 実行後も escape 済みコンテンツ・属性境界は不変のまま
    // であること（位置決め配線が既定エスケープ保証を弱めていないことの
    // 回帰）。
    assert!(container
        .query_selector("script")
        .expect("query_selector must not fail")
        .is_none());
    assert!(container
        .query_selector("img")
        .expect("query_selector must not fail")
        .is_none());
    assert_eq!(
        menu_trigger.get_attribute("data-testid").as_deref(),
        Some(attr_payload)
    );
    assert!(menu_trigger.get_attribute("onmouseover").is_none());

    drop(controller);
}

// ---------------------------------------------------------------------
// イシュー #1182: `PositionedKind` scope enum への navigation-menu /
// menubar 追加（出典 PR #1177 の out-of-scope 節）。冒頭 doc の検証観点
// (l)〜(o) 参照。
// ---------------------------------------------------------------------

/// 単一の menubar menu（trigger + positioner + content + item、open 状態）を
/// `container` 配下へ展開し、`(trigger, positioner, decoy_arrow)` を返す。
/// menubar の headless-ui anatomy は Arrow/ArrowTip を意図的スコープ外と
/// するため（モジュール doc 参照）、`[data-part="arrow"]` のデコイ要素を
/// positioner 配下へ追加し `PositionedKind::Menubar::has_arrow() == false`
/// の fail-closed 契約（`style` が付与されないこと）をブラウザ経路で確認
/// できるようにする（[`mount_open_select_with_decoy_arrow`] と同型）。
fn mount_open_menubar_with_decoy_arrow(
    document: &Document,
    container: &Element,
    id_prefix: &str,
    trigger_style: &str,
) -> (Element, Element, Element) {
    ensure_fixed_floating_size_stylesheet(document);
    let trigger_id = format!("{id_prefix}-trigger");
    let positioner_id = format!("{id_prefix}-positioner");
    let decoy_arrow_id = format!("{id_prefix}-decoy-arrow");
    let html = render(&menubar::root(
        Orientation::Horizontal,
        "",
        vec![],
        vec![menubar::menu(
            OpenState::Open,
            vec![],
            vec![
                menubar::trigger(
                    false,
                    OpenState::Open,
                    false,
                    false,
                    0,
                    None,
                    vec![("id", trigger_id.as_str()), ("style", trigger_style)],
                    vec![],
                ),
                menubar::positioner(
                    OpenState::Open,
                    // floating 要素に固定サイズを与える理由は
                    // [`FIXED_FLOATING_SIZE_CLASS`] のドキュメントを参照。
                    vec![
                        ("id", positioner_id.as_str()),
                        ("class", FIXED_FLOATING_SIZE_CLASS),
                    ],
                    vec![
                        menubar::content(
                            OpenState::Open,
                            None,
                            None,
                            vec![],
                            vec![menubar::item(
                                "item-1",
                                false,
                                false,
                                vec![],
                                vec![text("Item 1")],
                            )],
                        ),
                        // headless-ui の menubar モジュールは arrow パーツを
                        // 提供しない（`has_arrow() == false`）。誤って
                        // マークアップに混入した `data-part="arrow"` 要素が
                        // repositioning から除外されることを確認するための
                        // デコイ。
                        el(
                            "div",
                            vec![("data-part", "arrow"), ("id", decoy_arrow_id.as_str())],
                            vec![],
                        ),
                    ],
                ),
            ],
        )],
    ));
    container.set_inner_html(&html);
    let trigger = document
        .get_element_by_id(&trigger_id)
        .expect("trigger element must exist");
    let positioner = document
        .get_element_by_id(&positioner_id)
        .expect("positioner element must exist");
    let decoy_arrow = document
        .get_element_by_id(&decoy_arrow_id)
        .expect("decoy arrow element must exist");
    (trigger, positioner, decoy_arrow)
}

#[wasm_bindgen_test]
fn reposition_now_sets_reference_width_matching_anchor_width_for_menubar_and_skips_decoy_arrow() {
    // Menubar は `PositionedKind::same_width_default() == true` かつ
    // `has_arrow() == false`（native 側の
    // `same_width_default_true_for_menu_select_and_menubar_only`/
    // `arrow_target_kinds_are_popover_tooltip_menu_only` と同じ契約を
    // ブラウザ経路でも確認する。検証観点 (l)）。
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let container = create_placeholder(&document, "position-browser-menubar-width");
    let _guard = RemoveOnDrop(container.clone());

    let (_trigger, positioner, decoy_arrow) = mount_open_menubar_with_decoy_arrow(
        &document,
        &container,
        "position-browser-menubar-width",
        "position: fixed; left: 30px; top: 30px; width: 120px; height: 24px;",
    );

    let controller =
        PositionController::new(&window).expect("PositionController::new must succeed");
    controller.reposition_now();

    let style = positioner
        .get_attribute("style")
        .expect("Menubar positioner must receive a style attribute after reposition_now");
    assert!(style.contains("--fandhe-x:"));
    assert!(style.contains("--fandhe-y:"));
    assert!(
        style.contains("--fandhe-reference-width:"),
        "Menubar (same_width_default() == true) must output --fandhe-reference-width: {style}"
    );

    let reference_width = extract_css_var_px(&style, "--fandhe-reference-width");
    assert!(
        (reference_width - 120.0).abs() < 1.0,
        "--fandhe-reference-width must match the anchor (trigger) measured width (120px); \
         got {reference_width}"
    );

    assert!(
        decoy_arrow.get_attribute("style").is_none(),
        "Menubar (has_arrow() == false) must not receive a style attribute on a decoy \
         [data-part=\"arrow\"] element"
    );

    drop(controller);
}

/// 2 個の menu を持つ menubar を `container` 配下へ展開する。`index=0` の
/// menu は closed のまま、`index=1` の menu のみ open にする
/// （`find_menubar_anchor` の回帰、検証観点 (m)。イシュー #622 の
/// context-trigger/trigger-item 誤 anchor 指摘と同型の問題）。
/// `trigger0_style`/`trigger1_style` はそれぞれの anchor 矩形を決定的に
/// するための inline style（`position: fixed`）。返り値は
/// `(trigger0, trigger1, positioner0, positioner1)`。
fn mount_open_menubar_with_two_menus(
    document: &Document,
    container: &Element,
    id_prefix: &str,
    trigger0_style: &str,
    trigger1_style: &str,
) -> (Element, Element, Element, Element) {
    ensure_fixed_floating_size_stylesheet(document);
    let trigger0_id = format!("{id_prefix}-trigger-0");
    let trigger1_id = format!("{id_prefix}-trigger-1");
    let positioner0_id = format!("{id_prefix}-positioner-0");
    let positioner1_id = format!("{id_prefix}-positioner-1");
    let html = render(&menubar::root(
        Orientation::Horizontal,
        "",
        vec![],
        vec![
            menubar::menu(
                OpenState::Closed,
                vec![],
                vec![
                    menubar::trigger(
                        false,
                        OpenState::Closed,
                        false,
                        false,
                        0,
                        None,
                        vec![("id", trigger0_id.as_str()), ("style", trigger0_style)],
                        vec![],
                    ),
                    menubar::positioner(
                        OpenState::Closed,
                        vec![("id", positioner0_id.as_str())],
                        vec![menubar::content(
                            OpenState::Closed,
                            None,
                            None,
                            vec![],
                            vec![menubar::item(
                                "item-0",
                                false,
                                false,
                                vec![],
                                vec![text("Item 0")],
                            )],
                        )],
                    ),
                ],
            ),
            menubar::menu(
                OpenState::Open,
                vec![],
                vec![
                    menubar::trigger(
                        false,
                        OpenState::Open,
                        false,
                        false,
                        1,
                        None,
                        vec![("id", trigger1_id.as_str()), ("style", trigger1_style)],
                        vec![],
                    ),
                    menubar::positioner(
                        OpenState::Open,
                        // floating 要素に固定サイズを与える理由は
                        // [`FIXED_FLOATING_SIZE_CLASS`] のドキュメントを参照。
                        vec![
                            ("id", positioner1_id.as_str()),
                            ("class", FIXED_FLOATING_SIZE_CLASS),
                        ],
                        vec![menubar::content(
                            OpenState::Open,
                            None,
                            None,
                            vec![],
                            vec![menubar::item(
                                "item-1",
                                false,
                                false,
                                vec![],
                                vec![text("Item 1")],
                            )],
                        )],
                    ),
                ],
            ),
        ],
    ));
    container.set_inner_html(&html);
    let trigger0 = document
        .get_element_by_id(&trigger0_id)
        .expect("trigger0 element must exist");
    let trigger1 = document
        .get_element_by_id(&trigger1_id)
        .expect("trigger1 element must exist");
    let positioner0 = document
        .get_element_by_id(&positioner0_id)
        .expect("positioner0 element must exist");
    let positioner1 = document
        .get_element_by_id(&positioner1_id)
        .expect("positioner1 element must exist");
    (trigger0, trigger1, positioner0, positioner1)
}

#[wasm_bindgen_test]
fn reposition_now_anchors_menubar_to_its_own_menu_trigger_not_sibling_menu_trigger() {
    // イシュー #1182 検証観点 (m): menubar は単一 scope root 配下に複数の
    // menu ラッパー（trigger + positioner の組）が並ぶため、
    // `find_menubar_anchor` が「開いている positioner と同じ menu ラッパー
    // 内の trigger」を anchor として解決することを固定する（素朴な
    // `find_anchor` を適用すると root 内の先頭 trigger へ誤ってアンカー
    // される回帰、イシュー #622 と同型の問題）。
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let container = create_placeholder(&document, "position-browser-menubar-multi");
    let _guard = RemoveOnDrop(container.clone());

    let (_trigger0, _trigger1, positioner0, positioner1) = mount_open_menubar_with_two_menus(
        &document,
        &container,
        "position-browser-menubar-multi",
        "position: fixed; left: 5px; top: 5px; width: 10px; height: 10px;",
        "position: fixed; left: 500px; top: 5px; width: 10px; height: 10px;",
    );

    let controller =
        PositionController::new(&window).expect("PositionController::new must succeed");
    controller.reposition_now();

    assert!(
        positioner0.get_attribute("style").is_none(),
        "the closed (1st) menu's positioner must not be repositioned"
    );

    let style = positioner1.get_attribute("style").expect(
        "the open (2nd) menu's positioner must receive a style attribute after reposition_now \
         (find_menubar_anchor must resolve the trigger within its own [data-part=\"menu\"] \
         wrapper, not the 1st menu's trigger)",
    );
    let x = extract_fandhe_x(&style);
    assert!(
        x > 400.0,
        "positioner for the 2nd menu must anchor to its own trigger (left: 500px), not the \
         1st menu's trigger (left: 5px); got --fandhe-x: {x}"
    );

    drop(controller);
}

#[wasm_bindgen_test]
fn reposition_now_is_noop_for_navigation_menu_markup_without_positioner_part() {
    // イシュー #1182 検証観点 (n): headless-ui の navigation-menu モジュール
    // は `positioner` パーツを一切出力しない（イシュー #993、
    // `docs/policy/intentional-non-adoption.md` §3.25 規則 2）ため、
    // `PositionedKind::NavigationMenu` の scope 登録自体は成立するものの、
    // 現状のマークアップでは `reposition_now()` の発火契機
    // （`[data-part="positioner"][data-state="open"]`）が存在しない。
    // panic せず、いかなる要素にも `style`/`data-positioned` を付与しない
    // 前方互換の挙動を固定する。
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let container = create_placeholder(&document, "position-browser-navigation-menu");
    let _guard = RemoveOnDrop(container.clone());

    let trigger_id = "position-browser-navigation-menu-trigger";
    let content_id = "position-browser-navigation-menu-content";
    let nav_props = navigation_menu::NavigationMenuProps::default();
    let html = render(&navigation_menu::root(
        &nav_props,
        "Main",
        vec![],
        vec![navigation_menu::list(
            &nav_props,
            vec![],
            vec![navigation_menu::item(
                OpenState::Open,
                false,
                &nav_props,
                "products",
                vec![],
                vec![
                    navigation_menu::trigger(
                        OpenState::Open,
                        false,
                        "products",
                        Some(trigger_id),
                        Some(content_id),
                        vec![],
                        vec![text("Products")],
                    ),
                    navigation_menu::content(
                        OpenState::Open,
                        &nav_props,
                        "products",
                        Some(content_id),
                        Some(trigger_id),
                        vec![],
                        vec![navigation_menu::link(
                            "/products",
                            false,
                            vec![],
                            vec![text("All products")],
                        )],
                    ),
                ],
            )],
        )],
    ));
    container.set_inner_html(&html);

    let controller =
        PositionController::new(&window).expect("PositionController::new must succeed");
    controller.reposition_now();

    assert!(
        container
            .query_selector("[data-positioned]")
            .expect("query_selector must not fail")
            .is_none(),
        "navigation-menu markup has no positioner part, so reposition_now() must not mark any \
         element with data-positioned"
    );
    assert!(
        container
            .query_selector("[style]")
            .expect("query_selector must not fail")
            .is_none(),
        "navigation-menu markup has no positioner part, so reposition_now() must not write a \
         style attribute to any element"
    );

    drop(controller);
}

#[wasm_bindgen_test]
fn reposition_now_does_not_weaken_default_escaping_for_menubar_content() {
    // REQ-1（既定エスケープ）の位置決め経路への拡張回帰（イシュー #1182
    // 検証観点 (o)、既存の menu/select/tooltip 版と同型）。
    let window = web_sys::window().expect("window must exist in browser test environment");
    let document = window.document().expect("document must exist");
    let container = create_placeholder(&document, "position-browser-xss-menubar");
    let _guard = RemoveOnDrop(container.clone());

    let script_payload = "<script>alert('fandhe-xss-menubar')</script>";
    let attr_payload = "\" onmouseover=\"alert(1)";

    let trigger_id = "position-browser-xss-menubar-trigger";
    let positioner_id = "position-browser-xss-menubar-positioner";

    let html = render(&menubar::root(
        Orientation::Horizontal,
        "",
        vec![],
        vec![menubar::menu(
            OpenState::Open,
            vec![],
            vec![
                menubar::trigger(
                    false,
                    OpenState::Open,
                    false,
                    false,
                    0,
                    None,
                    vec![("id", trigger_id), ("data-testid", attr_payload)],
                    vec![text(script_payload)],
                ),
                menubar::positioner(
                    OpenState::Open,
                    vec![("id", positioner_id)],
                    vec![menubar::content(
                        OpenState::Open,
                        None,
                        None,
                        vec![],
                        vec![menubar::item(
                            script_payload,
                            false,
                            false,
                            vec![],
                            vec![text(script_payload)],
                        )],
                    )],
                ),
            ],
        )],
    ));
    container.set_inner_html(&html);

    assert!(
        container
            .query_selector("script")
            .expect("query_selector must not fail")
            .is_none(),
        "no real <script> element must be created from the escaped payload"
    );

    let text_content = container.text_content().unwrap_or_default();
    assert!(
        text_content.contains(script_payload),
        "the script payload must survive as literal text content (escaped, not executed): {text_content}"
    );

    let trigger = document
        .get_element_by_id(trigger_id)
        .expect("trigger element must exist");
    assert_eq!(
        trigger.get_attribute("data-testid").as_deref(),
        Some(attr_payload),
        "the attribute-injection payload must be preserved literally as the data-testid value"
    );
    assert!(
        trigger.get_attribute("onmouseover").is_none(),
        "the attribute-injection payload must not break out into a real onmouseover attribute"
    );

    let positioner = document
        .get_element_by_id(positioner_id)
        .expect("positioner element must exist");

    let controller =
        PositionController::new(&window).expect("PositionController::new must succeed");
    controller.reposition_now();

    assert!(
        positioner.get_attribute("style").is_some(),
        "position wiring must complete for the menubar positioner despite XSS-payload content"
    );

    assert!(container
        .query_selector("script")
        .expect("query_selector must not fail")
        .is_none());
    assert_eq!(
        trigger.get_attribute("data-testid").as_deref(),
        Some(attr_payload)
    );
    assert!(trigger.get_attribute("onmouseover").is_none());

    drop(controller);
}

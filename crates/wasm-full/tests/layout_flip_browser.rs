//! `layout_flip::capture_before`/`play_after`（イシュー #2518）の実ブラウザ
//! 統合テスト（`wasm-pack test --headless --chrome`）。
//!
//! `stagger_index_browser.rs` と同型の構成（本ファイル専用の最小
//! component `ListState` で keyed list 操作を誘発）で、
//! [`FLIP_AUTO_ATTR`] 付き keyed list の行を並べ替えた直後に対象行の
//! `style.transform` が一時的に補正値を持ち、収束後に解除されることを
//! 確認する。

#![cfg(target_arch = "wasm32")]
// `fandhe-frontend-animation` は `layout-animation` feature（既定 on）が有効な
// 場合のみ依存として解決される optional 依存（`Cargo.toml` `layout-animation =
// ["dep:fandhe-frontend-animation"]`）のため、feature matrix の
// `--no-default-features` 構成（イシュー #2328 baseline ジョブ）でもコンパイル
// できるよう本ファイル全体を feature ゲートする（`animate_browser.rs` と同型）。
#![cfg(feature = "layout-animation")]

use fandhe_frontend_core::keyed::keyed_list;
use fandhe_frontend_core::{bind_attr_tokens, el, text, Node};
use fandhe_frontend_interactive::{Component, DirtyTracked};
use fandhe_frontend_wasm_client::{BindingSource, BoundValue};
use fandhe_frontend_wasm_full::layout_flip::FLIP_AUTO_ATTR;
use fandhe_frontend_wasm_full::Runtime;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, Event, EventInit, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

/// `stagger_index_browser.rs::create_placeholder` と同じ意図。
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

/// `stagger_index_browser.rs::RemoveOnDrop` と同じ意図。
struct RemoveOnDrop(Element);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        self.0.remove();
    }
}

/// `stagger_index_browser.rs::bubbling_click_event` と同じ意図。
fn bubbling_click_event() -> Event {
    let init = EventInit::new();
    init.set_bubbles(true);
    Event::new_with_event_init_dict("click", &init).expect("Event::new must not fail")
}

/// `flip_browser.rs::sleep_ms` と同じ意図: 実タイマーを `Promise` 化して
/// `await` する決定的な待機。
async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window must exist");
        let callback = Closure::once_into_js(move || {
            let _ = resolve.call0(&JsValue::NULL);
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), ms)
            .expect("setTimeout must not fail in test environment");
    });
    JsFuture::from(promise)
        .await
        .expect("setTimeout promise must not reject");
}

/// `stagger_index_browser.rs::ListState` と同型の最小 component。
/// 行は絶対配置 + 明示 `top` px でレイアウト差を作る（実測ベースの
/// FLIP 計算がフレックス依存にならないようにするため）。
#[derive(Debug, Clone)]
struct ListState {
    items: Vec<(u64, String)>,
    dirty: Vec<&'static str>,
    /// [`FLIP_AUTO_ATTR`] を親要素に付けるか（オプトアウト検証用）。
    auto_flip: bool,
    /// 行に `style` 属性を一切付与しない自然フロー配置モード
    /// （codex-review 第 7 ラウンド是正、イシュー #2518。テストが行
    /// 要素へ直接書き込んだ inline `transform`/`transition`〔`style`
    /// プロパティ〕が、並べ替え後の再レンダーで `style` 属性文字列ごと
    /// 上書きされて消えてしまわないようにするための専用モード。既定の
    /// 絶対配置 + `top` px モードは行ごとに index 依存の `style` 属性
    /// 文字列を持つため、並べ替えで index が変わる行は `style` 属性が
    /// 丸ごと再設定され、テストが直接書き込んだプロパティも失われる）。
    natural_flow: bool,
}

impl ListState {
    const FIELD_ITEMS: &'static str = "items";

    fn new(initial: &[(u64, &str)]) -> Self {
        Self {
            items: initial
                .iter()
                .map(|(id, text)| (*id, text.to_string()))
                .collect(),
            dirty: Vec::new(),
            auto_flip: true,
            natural_flow: false,
        }
    }

    fn new_without_auto_flip(initial: &[(u64, &str)]) -> Self {
        Self {
            auto_flip: false,
            ..Self::new(initial)
        }
    }

    fn new_natural_flow(initial: &[(u64, &str)]) -> Self {
        Self {
            natural_flow: true,
            ..Self::new(initial)
        }
    }
}

enum ListAction {
    SwapFirstLast,
    /// 並び順・キー集合は変えず、先頭行の内容だけを変更する（レビュー
    /// 指摘対応の再現条件: `play_after` が `Before == After`〔`delta ==
    /// flip::IDENTITY`〕で Play を起動しない経路を、実際の並べ替えとは
    /// 独立に誘発するための操作、イシュー #2518 フォローアップ）。
    TouchContent,
}

impl Component for ListState {
    type Action = ListAction;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        match action {
            ListAction::SwapFirstLast => {
                let len = self.items.len();
                if len >= 2 {
                    self.items.swap(0, len - 1);
                    self.dirty.push(Self::FIELD_ITEMS);
                }
            }
            ListAction::TouchContent => {
                if let Some(first) = self.items.first_mut() {
                    first.1.push('!');
                }
                self.dirty.push(Self::FIELD_ITEMS);
            }
        }
    }

    fn view(&self) -> Node {
        let items: Vec<(String, Node)> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, (id, content))| {
                if self.natural_flow {
                    // `style` 属性を一切持たせない（[`Self::natural_flow`]
                    // doc 参照）。ブロック要素の自然な積み重ね順序だけで
                    // 並べ替え前後の実座標に意味のある差を作るため、
                    // 各行の `style` 属性文字列は index に関わらず常に
                    // 存在せず、並べ替えによる属性の再設定が発生しない。
                    return (
                        id.to_string(),
                        el(
                            "li",
                            vec![("data-testid", "flip-item")],
                            vec![text(content)],
                        ),
                    );
                }
                // 絶対配置 + `top` の行間隔で、並べ替え前後の実座標に
                // 意味のある差を作る（`get_bounding_client_rect()` が
                // 恒常的に同一矩形を返さないようにする）。
                let style = format!("position:absolute;top:{}px;left:0px;", index * 50);
                (
                    id.to_string(),
                    el(
                        "li",
                        vec![("data-testid", "flip-item"), ("style", style.as_str())],
                        vec![text(content)],
                    ),
                )
            })
            .collect();
        let mut parent_attrs = vec![("id", "flip-list")];
        if self.auto_flip {
            parent_attrs.push((FLIP_AUTO_ATTR, ""));
        }
        let list = keyed_list("ul", parent_attrs, "items", items)
            .expect("test fixture keyed items must be valid");
        el("div", vec![("id", "flip-root")], vec![list])
    }

    fn decode_action(name: &str, _payload: &str) -> Option<Self::Action> {
        match name {
            "swap_first_last" => Some(ListAction::SwapFirstLast),
            "touch_content" => Some(ListAction::TouchContent),
            _ => None,
        }
    }
}

impl DirtyTracked for ListState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

impl BindingSource for ListState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

/// `stagger_index_browser.rs::dispatch_action` と同じ手法。
fn dispatch_action(document: &Document, root: &Element, action: &str) {
    let trigger = document
        .create_element("button")
        .expect("create_element must not fail for a plain button");
    trigger
        .set_attribute("data-action", action)
        .expect("set_attribute must not fail");
    root.append_child(&trigger)
        .expect("append_child must not fail for a detached button");
    trigger
        .dispatch_event(&bubbling_click_event())
        .expect("dispatch_event must not fail");
    trigger.remove();
}

/// `root` 配下の `[data-testid='flip-item']` の `style.transform` 一覧を
/// DOM 順に読み取る。
fn read_transforms(root: &Element) -> Vec<String> {
    let nodes = root
        .query_selector_all("[data-testid='flip-item']")
        .expect("query_selector_all must not fail");
    let mut out = Vec::with_capacity(nodes.length() as usize);
    for i in 0..nodes.length() {
        let node = nodes.get(i).expect("index within length must exist");
        let html = node
            .dyn_ref::<HtmlElement>()
            .expect("keyed list item must be an HtmlElement");
        out.push(
            html.style()
                .get_property_value("transform")
                .expect("get_property_value must not fail"),
        );
    }
    out
}

/// `root` 配下の先頭の `[data-testid='flip-item']` を返す（codex-review
/// 第 7 ラウンド是正、イシュー #2518。要素自身の transform transition を
/// 直接操作するテスト用）。
fn first_flip_item(root: &Element) -> HtmlElement {
    root.query_selector("[data-testid='flip-item']")
        .expect("query_selector must not fail")
        .expect("at least one flip item must exist")
        .dyn_into::<HtmlElement>()
        .expect("keyed list item must be an HtmlElement")
}

/// 受け入れ条件: [`FLIP_AUTO_ATTR`] 付きリストの並べ替え直後は対象行の
/// `transform` が補正値を持ち、収束後（既定 spring の settle_duration
/// 経過後）には解除される。
#[wasm_bindgen_test]
async fn swap_applies_transient_transform_then_settles() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "flip-root-container-1");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "a"), (2, "b"), (3, "c")]);
    let runtime = Runtime::mount("flip-root-container-1", state).expect("mount must succeed");
    let root = runtime.root();

    dispatch_action(&document, root, "swap_first_last");

    let transforms_after_swap = read_transforms(root);
    assert!(
        transforms_after_swap.iter().any(|t| !t.is_empty()),
        "並べ替え直後は少なくとも 1 行に補正 transform が設定されているはず: {transforms_after_swap:?}"
    );

    // 既定 SpringConfig の settle_duration は数百 ms 程度（実装計画 §5 の
    // browser テスト方針どおり実時間で待機、`flip_browser.rs` と同じ
    // 余裕マージン）。
    sleep_ms(3_000).await;

    let transforms_after_settle = read_transforms(root);
    assert!(
        transforms_after_settle.iter().all(|t| t.is_empty()),
        "収束後は全行の transform が解除されているはず: {transforms_after_settle:?}"
    );
}

/// 受け入れ条件（オプトアウト）: [`FLIP_AUTO_ATTR`] を持たないリストは
/// 並べ替え後も `transform` を一切書き込まれない。
#[wasm_bindgen_test]
fn swap_without_auto_flip_attr_leaves_transform_untouched() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "flip-root-container-2");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new_without_auto_flip(&[(1, "a"), (2, "b"), (3, "c")]);
    let runtime = Runtime::mount("flip-root-container-2", state).expect("mount must succeed");
    let root = runtime.root();

    dispatch_action(&document, root, "swap_first_last");

    assert_eq!(
        read_transforms(root),
        vec!["", "", ""],
        "オプトイン属性が無いリストは transform を一切書き込まれないこと"
    );
}

/// 受け入れ条件（codex-review 第 4 ラウンド是正後も維持、イシュー
/// #2518）: 並べ替えで開始した FLIP アニメーションが進行中のまま、
/// Before == After（行の layout 位置は変化しない）となる内容のみの
/// 更新が来た場合、進行中の補正を瞬時に解除して終点へ跳ばす（旧実装の
/// 再生省略パス）のではなく、中断時点の表示位置を引き継いで滑らかに
/// 収束させる。新設計では [`fandhe_frontend_wasm_full::layout_flip::
/// capture_before`] が進行中の視覚矩形（[`fandhe_frontend_animation::
/// flip::measure`]）を旧アニメーション停止前に直接測ることでこの引き
/// 継ぎを実現する（旧実装の `current_viewport_delta`/`apply_delta_to_rect`
/// による計算は不要になった、`layout_flip.rs` モジュール doc「計測・
/// 合成の再設計」参照）。同時に、旧ループが本当に停止されていること
/// （新ループへ確実に引き継がれ、両者が競合して収束後も transform が
/// 残り続けないこと）も確認する。
#[wasm_bindgen_test]
async fn touch_content_during_flip_continues_smoothly_instead_of_snapping() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "flip-root-container-3");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = ListState::new(&[(1, "a"), (2, "b"), (3, "c")]);
    let runtime = Runtime::mount("flip-root-container-3", state).expect("mount must succeed");
    let root = runtime.root();

    // 1 回目: 実際に並べ替えて FLIP アニメーションを開始する（進行中の
    // AnimationLoop が FLIP_LOOPS に残る）。
    dispatch_action(&document, root, "swap_first_last");
    assert!(
        read_transforms(root).iter().any(|t| !t.is_empty()),
        "1 回目の並べ替え直後は少なくとも 1 行に補正 transform が設定されているはず"
    );

    // settle 前に、並び順を変えない内容のみの更新を即座にコミットする
    // （行の layout 位置は変化しない）。
    dispatch_action(&document, root, "touch_content");
    let transforms_immediately_after_touch = read_transforms(root);
    assert!(
        transforms_immediately_after_touch
            .iter()
            .any(|t| !t.is_empty()),
        "中断時点の表示位置を引き継ぐため、内容のみの更新直後も補正 \
         transform が残っている（瞬時に解除されない）はず: \
         {transforms_immediately_after_touch:?}"
    );

    // 十分に待てば、引き継いだアニメーションも最終的に収束する（旧
    // ループが確実に停止され、新ループとの競合で transform が残り続け
    // ないことの確認、レビュー指摘対応: イシュー #2518 フォローアップ
    // 「旧ループの停止を新アニメーション起動の有無に依存させない」）。
    sleep_ms(3_000).await;
    let transforms_after_settle = read_transforms(root);
    assert!(
        transforms_after_settle.iter().all(|t| t.is_empty()),
        "十分待てば引き継いだアニメーションも収束し、transform は解除される \
         はず: {transforms_after_settle:?}"
    );
}

/// 受け入れ条件（codex-review 第 7 ラウンド是正、イシュー #2518。P1
/// 「計測で確定した変形と FLIP の収束先を一致させる」）: 対象行が自身の
/// `transform`（`scale`）に CSS transition を持ち、その遷移が進行中の
/// まま並べ替えを行っても、(1) 並べ替え直後の表示位置は並べ替え直前の
/// 中間表示（First 視覚矩形）と連続し、(2) 十分待って収束した後の
/// computed transform は遷移の目標値（`scale(2)`）と一致する（中間値の
/// まま止まったり、無関係な値へ跳んだりしない）。`OriginalStyle::
/// capture` と Last 視覚矩形の捕捉を Last layout 計測（settle）の後へ
/// 移動した是正の回帰テスト（`layout_flip.rs` モジュール doc「捕捉順の
/// 契約」節参照）。
#[wasm_bindgen_test]
async fn swap_during_own_transform_transition_converges_to_settled_target() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "flip-root-container-4");
    let _guard = RemoveOnDrop(placeholder.clone());

    // `natural_flow`（`ListState::natural_flow` doc 参照）: 行に `style`
    // 属性を持たせないことで、並べ替えによる再レンダーが本テストが直接
    // 書き込む `transform`/`transition`（inline style）を上書きしない
    // ようにする。
    let state = ListState::new_natural_flow(&[(1, "a"), (2, "b"), (3, "c")]);
    let runtime = Runtime::mount("flip-root-container-4", state).expect("mount must succeed");
    let root = runtime.root();

    let first_item = first_flip_item(root);
    first_item
        .style()
        .set_property("transition", "transform 2000ms linear")
        .expect("set_property must not fail");
    // 現在の transform（未設定 = 恒等）を確定させてから遷移を開始する
    // （`flush_style` と同じ手法）。
    let _ = first_item.offset_width();
    first_item
        .style()
        .set_property("transform", "scale(2)")
        .expect("set_property must not fail");

    // 遷移が実際に開始してから、中間状態のまま並べ替えを行う。
    sleep_ms(200).await;
    let visual_before_swap = first_item.get_bounding_client_rect();

    dispatch_action(&document, root, "swap_first_last");

    // `play_after` は同期的に実行されるため、dispatch 直後の表示位置が
    // First（並べ替え直前の中間表示）と連続しているはず（跳びが無い）。
    let visual_immediately_after_swap = first_item.get_bounding_client_rect();
    assert!(
        (visual_immediately_after_swap.x() - visual_before_swap.x()).abs() < 2.0
            && (visual_immediately_after_swap.y() - visual_before_swap.y()).abs() < 2.0
            && (visual_immediately_after_swap.width() - visual_before_swap.width()).abs() < 2.0,
        "並べ替え直後の表示位置は並べ替え直前の中間表示と連続している \
         はず: before={visual_before_swap:?} after={visual_immediately_after_swap:?}"
    );

    // 十分に待って FLIP・要素自身の遷移の両方が収束するのを待つ。
    sleep_ms(3_000).await;

    let computed_transform = web_sys::window()
        .unwrap()
        .get_computed_style(&first_item)
        .expect("get_computed_style must not fail")
        .expect("computed style must exist for an attached element")
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        computed_transform, "matrix(2, 0, 0, 2, 0, 0)",
        "収束後の computed transform は遷移の目標値（scale(2)）と \
         一致しているはずが: {computed_transform}"
    );
}

/// 受け入れ条件（codex-review 第 9 ラウンド是正、イシュー #2518。P1
/// 「束縛属性の更新前に旧 FLIP を停止する」）の再現専用 component。
///
/// `id == 1` の行だけ `style` 属性を [`fandhe_frontend_core::bind_attr_
/// tokens`] で `row_style` フィールドへ束縛し、`view()` 自体はその行に
/// `style` 属性を一切埋め込まない（「行の view が変わらず」の再現条件、
/// コーディネータ指摘の引用参照）。これにより `apply_keyed_list_with_
/// previous` の内容比較（`view()` の Node 木を比較する経路）はこの行の
/// `style` を一切検知・上書きせず、`row_style` の反映は
/// `fandhe_frontend_wasm_client::BindingTable::apply_dirty`（束縛点
/// 書き込み）だけが担う——`Runtime::apply_update_for_dirty` の捕捉順
/// バグを他の経路に紛れさせず単独で再現するための設計。
#[derive(Debug, Clone)]
struct RaceState {
    items: Vec<u64>,
    row_style: String,
    dirty: Vec<&'static str>,
}

enum RaceAction {
    /// 並べ替えのみ（束縛更新なし）。
    Swap,
    /// 並べ替えと同じ更新で `id == 1` の行の `style` 束縛
    /// （`row_style` フィールド）も同時に更新する（再現条件）。
    SwapAndBindOwnTransform,
    /// `items` は一切変更せず、`row_style` フィールドのみを dirty にする
    /// （codex-review 追加ラウンド再現条件、イシュー #2518。P1「行の
    /// 束縛だけを更新する場合も進行中の FLIP を停止する」）。`items` が
    /// 同時に dirty な `SwapAndBindOwnTransform` とは異なり、`find_list_
    /// element(root, "row_style")` が `None` を返すため、リスト自体の
    /// dirty 判定だけに頼る捕捉経路ではこの更新の旧 FLIP 停止が起きない
    /// ことを検証する）。
    BindOwnTransformOnly,
}

impl RaceState {
    const FIELD_ITEMS: &'static str = "items";
    const FIELD_ROW_STYLE: &'static str = "row_style";

    fn new() -> Self {
        Self {
            items: vec![1, 2],
            row_style: String::new(),
            dirty: Vec::new(),
        }
    }
}

impl Component for RaceState {
    type Action = RaceAction;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        match action {
            RaceAction::Swap => {
                self.items.swap(0, 1);
                self.dirty.push(Self::FIELD_ITEMS);
            }
            RaceAction::SwapAndBindOwnTransform => {
                self.items.swap(0, 1);
                self.row_style = "transform:scale(1.5);".to_string();
                self.dirty.push(Self::FIELD_ITEMS);
                self.dirty.push(Self::FIELD_ROW_STYLE);
            }
            RaceAction::BindOwnTransformOnly => {
                self.row_style = "transform:scale(1.5);".to_string();
                self.dirty.push(Self::FIELD_ROW_STYLE);
            }
        }
    }

    fn view(&self) -> Node {
        let items: Vec<(String, Node)> = self
            .items
            .iter()
            .map(|id| {
                let node = if *id == 1 {
                    // `style` 属性そのものは埋め込まない（本 struct doc
                    // 「行の view が変わらず」参照）。`data-bind-attr` の
                    // 束縛値のみが `style` を決める。
                    let bind_attr = bind_attr_tokens(&[("style", RaceState::FIELD_ROW_STYLE)]);
                    el(
                        "li",
                        vec![
                            ("data-testid", "flip-item"),
                            ("data-bind-attr", bind_attr.as_str()),
                        ],
                        vec![text("a")],
                    )
                } else {
                    el("li", vec![("data-testid", "flip-item")], vec![text("b")])
                };
                (id.to_string(), node)
            })
            .collect();
        let list = keyed_list(
            "ul",
            vec![("id", "flip-list"), (FLIP_AUTO_ATTR, "")],
            "items",
            items,
        )
        .expect("test fixture keyed items must be valid");
        el("div", vec![("id", "flip-root")], vec![list])
    }

    fn decode_action(name: &str, _payload: &str) -> Option<Self::Action> {
        match name {
            "swap" => Some(RaceAction::Swap),
            "swap_and_bind_own_transform" => Some(RaceAction::SwapAndBindOwnTransform),
            "bind_own_transform_only" => Some(RaceAction::BindOwnTransformOnly),
            _ => None,
        }
    }
}

impl DirtyTracked for RaceState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

impl BindingSource for RaceState {
    fn bound_value(&self, field: &str) -> Option<BoundValue> {
        match field {
            "row_style" => Some(BoundValue::Text(self.row_style.clone())),
            _ => None,
        }
    }
}

/// 受け入れ条件（codex-review 第 9 ラウンド是正、イシュー #2518。P1
/// 「束縛属性の更新前に旧 FLIP を停止する」）: FLIP 進行中（並べ替え
/// 直後、収束前）に、同じ更新で行の `style` 束縛（`data-bind-attr`）と
/// リストの並べ替えを同時に行うと、束縛が書き込んだ新しい transform が
/// 旧 FLIP ハンドルの停止（`capture_before` 内の `OriginalStyle::
/// restore`）で失われず、収束後も残る。
#[wasm_bindgen_test]
async fn binding_write_survives_old_flip_stop_during_same_update() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "flip-root-container-5");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = RaceState::new();
    let runtime = Runtime::mount("flip-root-container-5", state).expect("mount must succeed");
    let root = runtime.root();

    // 1 回目: 並べ替えのみ（束縛更新なし）。両行の FLIP アニメーションが
    // 進行中のまま次の更新へ入る（settle 前に間を置かず 2 回目を発行）。
    dispatch_action(&document, root, "swap");
    assert!(
        read_transforms(root).iter().any(|t| !t.is_empty()),
        "1 回目の並べ替え直後は少なくとも 1 行に補正 transform が設定されているはず"
    );

    // 2 回目: 並べ替えと同じ更新で id=1 の行の style 束縛（row_style）も
    // 更新する（再現条件）。1 回目の FLIP はまだ進行中のはず。
    dispatch_action(&document, root, "swap_and_bind_own_transform");

    // 十分に待って両方の FLIP が収束するのを待つ。
    sleep_ms(3_000).await;

    let row1_element = root
        .query_selector("li[data-testid='flip-item'][data-key='1']")
        .expect("query_selector must not fail")
        .expect("id=1 の行が見つかるはず")
        .dyn_into::<HtmlElement>()
        .expect("keyed list item must be an HtmlElement");

    let computed_transform = web_sys::window()
        .unwrap()
        .get_computed_style(&row1_element)
        .expect("get_computed_style must not fail")
        .expect("computed style must exist for an attached element")
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        computed_transform, "matrix(1.5, 0, 0, 1.5, 0, 0)",
        "束縛が書き込んだ transform:scale(1.5) は旧 FLIP の停止で失われず \
         収束後も残っているはずが: {computed_transform}"
    );
}

/// 受け入れ条件（codex-review 追加ラウンド是正、イシュー #2518。P1
/// 「行の束縛だけを更新する場合も進行中の FLIP を停止する」）: FLIP
/// 進行中（並べ替え直後、収束前）に、**リスト自体（`items`）を dirty に
/// しない別更新**で行の `style` 束縛（`row_style`）だけを dirty にしても、
/// 束縛が書き込んだ新しい transform が旧 FLIP ハンドルの停止
/// （`layout_flip::capture_before` 内の `flip::FlipAnimation::drop` →
/// `OriginalStyle::restore`）で失われず、数フレーム後・収束後ともに
/// 残る。`binding_write_survives_old_flip_stop_during_same_update`
/// （`items` と `row_style` が同時に dirty なケース）とは異なり、`find_
/// list_element(root, "row_style")` が `None` を返すため、リスト自体の
/// dirty 判定だけに頼る捕捉経路（`Runtime::apply_update_for_dirty` の
/// 第 1 走査）ではこの更新の旧 FLIP 停止が起きない。行の束縛先要素の
/// 祖先探索（第 2 走査、`layout_flip::nearest_flip_list`）が必須である
/// ことを単独で再現する。
#[wasm_bindgen_test]
async fn binding_only_write_survives_old_flip_stop_across_separate_update() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "flip-root-container-6");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = RaceState::new();
    let runtime = Runtime::mount("flip-root-container-6", state).expect("mount must succeed");
    let root = runtime.root();

    // 1 回目: 並べ替えのみ（束縛更新なし）。両行の FLIP アニメーションが
    // 進行中のまま次の更新へ入る（settle 前に間を置かず 2 回目を発行）。
    dispatch_action(&document, root, "swap");
    assert!(
        read_transforms(root).iter().any(|t| !t.is_empty()),
        "1 回目の並べ替え直後は少なくとも 1 行に補正 transform が設定されているはず"
    );

    // 2 回目: `items` は一切変更せず、id=1 の行の style 束縛
    // （`row_style`）のみを dirty にする（再現条件。1 回目の FLIP は
    // まだ進行中のはず）。
    dispatch_action(&document, root, "bind_own_transform_only");

    let row1_element = root
        .query_selector("li[data-testid='flip-item'][data-key='1']")
        .expect("query_selector must not fail")
        .expect("id=1 の行が見つかるはず")
        .dyn_into::<HtmlElement>()
        .expect("keyed list item must be an HtmlElement");

    // 数フレーム後（旧 FLIP が正しく停止していれば、次フレームの
    // 補正上書きは起きないはず）: 短い待機で早期の上書きを検知する。
    sleep_ms(50).await;
    let computed_transform_early = web_sys::window()
        .unwrap()
        .get_computed_style(&row1_element)
        .expect("get_computed_style must not fail")
        .expect("computed style must exist for an attached element")
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        computed_transform_early, "matrix(1.5, 0, 0, 1.5, 0, 0)",
        "束縛が書き込んだ transform:scale(1.5) は数フレーム後も旧 FLIP に \
         上書きされていないはずが: {computed_transform_early}"
    );

    // 収束後（十分に待って残っている旧 FLIP が settle するのを待つ）:
    // settle 時の restore で束縛値が失われていないことを確認する。
    sleep_ms(3_000).await;
    let computed_transform_settled = web_sys::window()
        .unwrap()
        .get_computed_style(&row1_element)
        .expect("get_computed_style must not fail")
        .expect("computed style must exist for an attached element")
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        computed_transform_settled, "matrix(1.5, 0, 0, 1.5, 0, 0)",
        "束縛が書き込んだ transform:scale(1.5) は旧 FLIP の収束後も \
         restore で失われず残っているはずが: {computed_transform_settled}"
    );
}

/// 受け入れ条件（codex-review 追加ラウンド 2 巡目 P1「入れ子リストの
/// 行自身を動かす外側の FLIP も停止する」、イシュー #2518）の再現専用
/// component。[`RaceState`] と同型だが、`id == 1` の行要素自身が
/// [`FLIP_AUTO_ATTR`] を持つ点が異なる（外側 keyed list の直接の行が、
/// 同時に別の `FLIP_AUTO_ATTR` 付きリストの境界でもある構成の最小
/// 再現。実際に入れ子の keyed list を構築する必要はなく、`Element::
/// closest`（旧実装）が「要素自身も検索対象に含む」ため、束縛先要素
/// 自身に `FLIP_AUTO_ATTR` を持たせるだけで「最寄り 1 件」が自分自身に
/// 止まり外側リストへ到達しない状況を再現できる）。
#[derive(Debug, Clone)]
struct NestedRaceState {
    items: Vec<u64>,
    row_style: String,
    dirty: Vec<&'static str>,
}

enum NestedRaceAction {
    /// 並べ替えのみ（束縛更新なし）。
    Swap,
    /// `items` は一切変更せず、`row_style` フィールドのみを dirty にする。
    BindOwnTransformOnly,
}

impl NestedRaceState {
    const FIELD_ITEMS: &'static str = "items";
    const FIELD_ROW_STYLE: &'static str = "row_style";

    fn new() -> Self {
        Self {
            items: vec![1, 2],
            row_style: String::new(),
            dirty: Vec::new(),
        }
    }
}

impl Component for NestedRaceState {
    type Action = NestedRaceAction;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        match action {
            NestedRaceAction::Swap => {
                self.items.swap(0, 1);
                self.dirty.push(Self::FIELD_ITEMS);
            }
            NestedRaceAction::BindOwnTransformOnly => {
                self.row_style = "transform:scale(1.5);".to_string();
                self.dirty.push(Self::FIELD_ROW_STYLE);
            }
        }
    }

    fn view(&self) -> Node {
        let items: Vec<(String, Node)> = self
            .items
            .iter()
            .map(|id| {
                let node = if *id == 1 {
                    // 本 struct doc の再現条件: `style` 属性は
                    // `data-bind-attr` の束縛値のみが決める（属性その
                    // ものは埋め込まない）うえ、行要素自身が
                    // `FLIP_AUTO_ATTR` を持つ（外側リストの行かつ、
                    // それ自身も FLIP リスト境界）。
                    let bind_attr =
                        bind_attr_tokens(&[("style", NestedRaceState::FIELD_ROW_STYLE)]);
                    el(
                        "li",
                        vec![
                            ("data-testid", "flip-item"),
                            ("data-bind-attr", bind_attr.as_str()),
                            (FLIP_AUTO_ATTR, ""),
                        ],
                        vec![text("a")],
                    )
                } else {
                    el("li", vec![("data-testid", "flip-item")], vec![text("b")])
                };
                (id.to_string(), node)
            })
            .collect();
        let list = keyed_list(
            "ul",
            vec![("id", "flip-list"), (FLIP_AUTO_ATTR, "")],
            "items",
            items,
        )
        .expect("test fixture keyed items must be valid");
        el("div", vec![("id", "flip-root")], vec![list])
    }

    fn decode_action(name: &str, _payload: &str) -> Option<Self::Action> {
        match name {
            "swap" => Some(NestedRaceAction::Swap),
            "bind_own_transform_only" => Some(NestedRaceAction::BindOwnTransformOnly),
            _ => None,
        }
    }
}

impl DirtyTracked for NestedRaceState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

impl BindingSource for NestedRaceState {
    fn bound_value(&self, field: &str) -> Option<BoundValue> {
        match field {
            "row_style" => Some(BoundValue::Text(self.row_style.clone())),
            _ => None,
        }
    }
}

/// 受け入れ条件（codex-review 追加ラウンド 2 巡目 P1「入れ子リストの
/// 行自身を動かす外側の FLIP も停止する」、イシュー #2518。契約絞り込み
/// （さらに次ラウンドの P1 2 件 + Bugbot Medium 1 件、`layout_flip.rs`
/// モジュール doc「入れ子 FLIP リストの所有権契約」節参照）後の版）:
/// 束縛要素自身が `FLIP_AUTO_ATTR` を持つ（外側 keyed list の行かつ、
/// それ自身も別リストの境界）構成でも、行の `style` 束縛のみを dirty に
/// した更新は束縛値を失わない。
///
/// **新契約での assert 方針**: 新契約では最外側リスト（本テストでは
/// `id=1` の行の外側にある `<ul id="flip-list">`）が「サブツリー全体の
/// アニメーション」を所有するため、行 1 自身の `FLIP_AUTO_ATTR`（入れ子
/// マーカー）が dirty 時に停止・捕捉こそされるが、実際に play されるのは
/// 常に最外側リストのみである。1 回目の並べ替えで開始した外側 FLIP が
/// 2 回目の更新（`row_style` のみ dirty）でも停止され、束縛が書き込んだ
/// `transform:scale(1.5)` を収束後に確認する（旧実装は「行 1 自身」を
/// 最寄りの停止対象とみなしていたため、行自身の位置を制御する外側の
/// FLIP を取りこぼしていた）。旧契約（第 1 ラウンド是正時点）で行って
/// いた「数フレーム後に即座 scale(1.5) と完全一致する」early assert は
/// 削除した: 新契約では外側リストが**行 1 自身の位置変化も含めて**
/// Invert を計算し直すため、収束前の中間フレームは（外側 FLIP が正しく
/// 継続中の補正を滑らかに合成している限り）scale(1.5) 単独ではなく
/// 合成された遷移値を一時的に示し得る（`swap_during_own_transform_
/// transition_converges_to_settled_target` と同型の正当な継続性）。
/// 束縛値が失われない不変条件は「収束後」の一致でのみ検証する。
#[wasm_bindgen_test]
async fn binding_only_write_survives_old_flip_stop_when_row_is_itself_a_flip_list() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "flip-root-container-8");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = NestedRaceState::new();
    let runtime = Runtime::mount("flip-root-container-8", state).expect("mount must succeed");
    let root = runtime.root();

    // 1 回目: 並べ替えのみ。両行の FLIP アニメーション（外側リスト
    // スコープ）が進行中のまま次の更新へ入る。
    dispatch_action(&document, root, "swap");
    assert!(
        read_transforms(root).iter().any(|t| !t.is_empty()),
        "1 回目の並べ替え直後は少なくとも 1 行に補正 transform が設定されているはず"
    );

    // 2 回目: `items` は一切変更せず、id=1 の行の style 束縛
    // （`row_style`）のみを dirty にする。id=1 の行要素自身が
    // `FLIP_AUTO_ATTR` を持つため、`elements_bound_to_field` が返す
    // 束縛先要素＝行要素自身から祖先探索すると、行自身と外側リストの
    // 両方が見つかる（`flip_lists_containing` は全祖先を返す）が、新契約
    // では最外側（外側リスト）のみが play 対象になる。
    dispatch_action(&document, root, "bind_own_transform_only");

    let row1_element = root
        .query_selector("li[data-testid='flip-item'][data-key='1']")
        .expect("query_selector must not fail")
        .expect("id=1 の行が見つかるはず")
        .dyn_into::<HtmlElement>()
        .expect("keyed list item must be an HtmlElement");

    sleep_ms(3_000).await;
    let computed_transform_settled = web_sys::window()
        .unwrap()
        .get_computed_style(&row1_element)
        .expect("get_computed_style must not fail")
        .expect("computed style must exist for an attached element")
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        computed_transform_settled, "matrix(1.5, 0, 0, 1.5, 0, 0)",
        "束縛が書き込んだ transform:scale(1.5) は外側リストの旧 FLIP の \
         収束後も restore で失われず残っているはずが: {computed_transform_settled}"
    );
}

/// 受け入れ条件（codex-review 追加ラウンド 2 巡目 P1「Last 計測は全
/// dirty field の構造変化コミット完了後にまとめて行う」、イシュー
/// #2518）の再現専用 component。上下に積み重なる 2 つの独立した keyed
/// list（`upper`/`lower`）を持つ。`lower` のみ [`FLIP_AUTO_ATTR`] を
/// 持ち、`upper` は素の keyed list（FLIP 非対象）としてレイアウトへ
/// 影響する行数だけを変える。
#[derive(Debug, Clone)]
struct TwoListState {
    upper: Vec<u64>,
    lower: Vec<u64>,
    dirty: Vec<&'static str>,
}

enum TwoListAction {
    /// `lower` の並べ替えと `upper` への行追加を同じ更新で行う。dirty
    /// field は「下側 (`lower`)、上側 (`upper`)」の順で発行する
    /// （再現条件そのもの、codex-review 指摘の記述順）。
    SwapLowerGrowUpper,
}

impl TwoListState {
    const FIELD_UPPER: &'static str = "upper";
    const FIELD_LOWER: &'static str = "lower";

    fn new() -> Self {
        Self {
            upper: vec![101, 102],
            lower: vec![201, 202],
            dirty: Vec::new(),
        }
    }
}

impl Component for TwoListState {
    type Action = TwoListAction;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        match action {
            TwoListAction::SwapLowerGrowUpper => {
                let len = self.lower.len();
                if len >= 2 {
                    self.lower.swap(0, len - 1);
                }
                self.upper.push(103);
                self.dirty.push(Self::FIELD_LOWER);
                self.dirty.push(Self::FIELD_UPPER);
            }
        }
    }

    fn view(&self) -> Node {
        // 両リストとも `style` 属性を一切持たせない自然フロー配置
        // （`ListState::natural_flow` doc と同じ意図）。`upper` の行数
        // 変化が純粋な文書フローの高さ変化として `lower` の絶対位置へ
        // 伝播する状況を作る。
        let upper_items: Vec<(String, Node)> = self
            .upper
            .iter()
            .map(|id| {
                (
                    id.to_string(),
                    el(
                        "li",
                        vec![("data-testid", "upper-item")],
                        vec![text("upper row")],
                    ),
                )
            })
            .collect();
        let upper_list = keyed_list(
            "ul",
            vec![("id", "upper-list")],
            Self::FIELD_UPPER,
            upper_items,
        )
        .expect("test fixture keyed items must be valid");

        let lower_items: Vec<(String, Node)> = self
            .lower
            .iter()
            .map(|id| {
                (
                    id.to_string(),
                    el(
                        "li",
                        vec![("data-testid", "lower-item")],
                        vec![text("lower row")],
                    ),
                )
            })
            .collect();
        let lower_list = keyed_list(
            "ul",
            vec![("id", "lower-list"), (FLIP_AUTO_ATTR, "")],
            Self::FIELD_LOWER,
            lower_items,
        )
        .expect("test fixture keyed items must be valid");

        el(
            "div",
            vec![("id", "flip-root")],
            vec![upper_list, lower_list],
        )
    }

    fn decode_action(name: &str, _payload: &str) -> Option<Self::Action> {
        match name {
            "swap_lower_grow_upper" => Some(TwoListAction::SwapLowerGrowUpper),
            _ => None,
        }
    }
}

impl DirtyTracked for TwoListState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

impl BindingSource for TwoListState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

/// 受け入れ条件（codex-review 追加ラウンド 2 巡目 P1「Last 計測は全
/// dirty field の構造変化コミット完了後にまとめて行う」、イシュー
/// #2518）: `lower`（FLIP 対象）を並べ替えると同時に `upper`（FLIP 非
/// 対象、dirty 順で後発）へ行を追加すると、`upper` の行追加による
/// 純粋な文書フロー上の高さ増が `lower` の行の絶対位置へ伝播する。
/// `play_after` の Last 計測が全 dirty field の構造変化コミット完了後に
/// 行われていれば、FLIP の Invert がこの伝播分も含めて相殺し、
/// 構造変化コミット直後（frame 0、rAF 未進行）の視覚位置は更新直前と
/// ほぼ連続する（FLIP の基本契約）。Last 計測が `upper` のコミットより
/// 前に行われていた旧実装では、この差が `upper` に追加された 1 行の
/// 高さ分だけ生じる。
#[wasm_bindgen_test]
async fn play_after_uses_last_measured_after_all_dirty_fields_structural_commit() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "flip-root-container-9");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = TwoListState::new();
    let runtime = Runtime::mount("flip-root-container-9", state).expect("mount must succeed");
    let root = runtime.root();

    let lower_first = root
        .query_selector("li[data-testid='lower-item'][data-key='201']")
        .expect("query_selector must not fail")
        .expect("key=201 の行が見つかるはず")
        .dyn_into::<HtmlElement>()
        .expect("keyed list item must be an HtmlElement");

    let before_top = lower_first.get_bounding_client_rect().top();

    dispatch_action(&document, root, "swap_lower_grow_upper");

    // `play_after` は同期的に実行される（`swap_during_own_transform_
    // transition_converges_to_settled_target` と同じ手法）。frame 0
    // （rAF 未進行）の視覚位置が更新直前とほぼ連続していれば、`upper`
    // の高さ増分も含めて正しく Invert されたことになる。
    let frame0_top = lower_first.get_bounding_client_rect().top();
    let jump = (frame0_top - before_top).abs();
    assert!(
        jump < 2.0,
        "FLIP の frame 0 は視覚的に不動のはずが、before={before_top} \
         frame0={frame0_top}（差 {jump}px）。play_after の Last 計測が \
         upper の構造変化コミット前に行われていると、この差が upper に \
         追加された行の高さ分だけ生じる"
    );

    // 収束後: inline transform が解除されている（収束したことの確認。
    // 収束先の絶対座標は環境依存のため検証しない）。
    sleep_ms(3_000).await;
    let settled_transform = lower_first
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        settled_transform, "",
        "収束後は inline transform が解除されているはずが: {settled_transform}"
    );
}

/// 入れ子 FLIP リストの所有権契約（`layout_flip.rs` モジュール doc「入れ子
/// FLIP リストの所有権契約」節、イシュー #2518）の再現専用 component。
/// `outer`（外側 keyed list、[`FLIP_AUTO_ATTR`]）の `id == 1` の行だけが
/// 内側 keyed list `inner`（同じく [`FLIP_AUTO_ATTR`]）を子孫に持つ、
/// 真の 2 段入れ子構成（両方とも実際の `data-bind-list` field）。
#[derive(Debug, Clone)]
struct DeepNestedState {
    outer: Vec<u64>,
    inner: Vec<u64>,
    dirty: Vec<&'static str>,
}

// 全バリアントが「並べ替え操作」であるため `Swap` プレフィックスが
// 揃うのは意図的（`decode_action` の action 名と対応させた命名）。
#[allow(clippy::enum_variant_names)]
enum DeepNestedAction {
    /// `outer` を並べ替える（`id == 1` の行〔内側リストを含む〕が移動
    /// する）。`inner` の内容は変えないが、dirty field としては
    /// `["inner", "outer"]`（内側→外側の順）で発行する（再現条件その
    /// もの。二重補正バグは「入れ子リストを内側から処理する」ことが
    /// 引き金のため）。
    SwapOuterDirtyInnerFirst,
    /// `SwapOuterDirtyInnerFirst` と同じ並べ替えだが、dirty field の
    /// 発行順を `["outer", "inner"]`（外側→内側の順）にする。二重補正が
    /// dirty の列挙順に依存しないこと（`layout_flip.rs` モジュール doc
    /// 「この 4 点により…dirty の列挙順…に関わらず」）を、内側→外側順
    /// （`SwapOuterDirtyInnerFirst`）と外側→内側順の両方で固定するための
    /// 対照実験用アクション。
    SwapOuterDirtyOuterFirst,
    /// `outer` の並び順は一切変えず、`inner` のみを並べ替える。dirty
    /// field は `["inner"]` のみ（`outer` を含まない）。受け入れ条件
    /// 「内側のみ dirty で外側 FLIP が停止・再捕捉される」の再現用:
    /// `find_list_element(root, "inner")` の結果から
    /// `flip_lists_containing` を辿ると `[inner-list, outer-list]` の
    /// 両方が見つかる（`outer` field 自体は dirty でなくても、内側リスト
    /// の祖先である外側リストは走査 (i) で必ず捕捉される契約、
    /// `Runtime::apply_update_for_dirty` doc 参照）。
    SwapInnerOnly,
    /// `outer` のみを並べ替える（`inner` は変えない）。dirty field は
    /// `["outer"]` のみ。`SwapInnerOnly` による再捕捉を検証する前段
    /// として、外側 FLIP を進行中の状態にするために使う。
    SwapOuterOnly,
}

impl DeepNestedState {
    const FIELD_OUTER: &'static str = "outer";
    const FIELD_INNER: &'static str = "inner";

    fn new() -> Self {
        Self {
            outer: vec![1, 2],
            inner: vec![10, 20],
            dirty: Vec::new(),
        }
    }
}

impl Component for DeepNestedState {
    type Action = DeepNestedAction;

    fn update(&mut self, action: Self::Action) {
        self.dirty.clear();
        match action {
            DeepNestedAction::SwapOuterDirtyInnerFirst => {
                let len = self.outer.len();
                if len >= 2 {
                    self.outer.swap(0, len - 1);
                }
                // dirty 順を「内側、外側」に固定する（再現条件そのもの）。
                self.dirty.push(Self::FIELD_INNER);
                self.dirty.push(Self::FIELD_OUTER);
            }
            DeepNestedAction::SwapOuterDirtyOuterFirst => {
                let len = self.outer.len();
                if len >= 2 {
                    self.outer.swap(0, len - 1);
                }
                // dirty 順を「外側、内側」に固定する（対照実験）。
                self.dirty.push(Self::FIELD_OUTER);
                self.dirty.push(Self::FIELD_INNER);
            }
            DeepNestedAction::SwapInnerOnly => {
                let len = self.inner.len();
                if len >= 2 {
                    self.inner.swap(0, len - 1);
                }
                self.dirty.push(Self::FIELD_INNER);
            }
            DeepNestedAction::SwapOuterOnly => {
                let len = self.outer.len();
                if len >= 2 {
                    self.outer.swap(0, len - 1);
                }
                self.dirty.push(Self::FIELD_OUTER);
            }
        }
    }

    fn view(&self) -> Node {
        // 両リストとも `style` 属性を持たせない自然フロー配置
        // （`ListState::natural_flow` doc と同じ意図）。
        let outer_items: Vec<(String, Node)> = self
            .outer
            .iter()
            .map(|id| {
                let node = if *id == 1 {
                    let inner_items: Vec<(String, Node)> = self
                        .inner
                        .iter()
                        .map(|inner_id| {
                            (
                                inner_id.to_string(),
                                el(
                                    "li",
                                    vec![("data-testid", "inner-item")],
                                    vec![text("inner row")],
                                ),
                            )
                        })
                        .collect();
                    let inner_list = keyed_list(
                        "ul",
                        vec![("data-testid", "inner-list"), (FLIP_AUTO_ATTR, "")],
                        Self::FIELD_INNER,
                        inner_items,
                    )
                    .expect("test fixture keyed items must be valid");
                    el("li", vec![("data-testid", "outer-item")], vec![inner_list])
                } else {
                    el(
                        "li",
                        vec![("data-testid", "outer-item")],
                        vec![text("plain outer row")],
                    )
                };
                (id.to_string(), node)
            })
            .collect();
        let outer_list = keyed_list(
            "ul",
            vec![("id", "outer-list"), (FLIP_AUTO_ATTR, "")],
            Self::FIELD_OUTER,
            outer_items,
        )
        .expect("test fixture keyed items must be valid");
        el("div", vec![("id", "flip-root")], vec![outer_list])
    }

    fn decode_action(name: &str, _payload: &str) -> Option<Self::Action> {
        match name {
            "swap_outer_dirty_inner_first" => Some(DeepNestedAction::SwapOuterDirtyInnerFirst),
            "swap_outer_dirty_outer_first" => Some(DeepNestedAction::SwapOuterDirtyOuterFirst),
            "swap_inner_only" => Some(DeepNestedAction::SwapInnerOnly),
            "swap_outer_only" => Some(DeepNestedAction::SwapOuterOnly),
            _ => None,
        }
    }
}

impl DirtyTracked for DeepNestedState {
    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }
}

impl BindingSource for DeepNestedState {
    fn bound_value(&self, _field: &str) -> Option<BoundValue> {
        None
    }
}

/// 受け入れ条件（codex-review 追加ラウンド 2 巡目 P1「入れ子リストを
/// 内側から処理すると補正が二重適用され dirty 順に表示が依存する」、
/// イシュー #2518）: dirty 順が `[inner, outer]`（内側→外側）でも、
/// 内側リストは自分では play せず最外側（`outer-list`）だけが play する
/// ため、内側リストの子（`inner-item`）の frame 0（構造変化コミット
/// 直後、rAF 未進行）の視覚位置が更新直前とほぼ連続する（二重補正なら
/// 内側の子が余分に跳ぶ）。
#[wasm_bindgen_test]
async fn nested_flip_lists_do_not_double_correct_when_dirty_order_is_inner_first() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "flip-root-container-10");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = DeepNestedState::new();
    let runtime = Runtime::mount("flip-root-container-10", state).expect("mount must succeed");
    let root = runtime.root();

    let inner_first = root
        .query_selector("li[data-testid='inner-item'][data-key='10']")
        .expect("query_selector must not fail")
        .expect("inner key=10 の行が見つかるはず")
        .dyn_into::<HtmlElement>()
        .expect("keyed list item must be an HtmlElement");

    let before_top = inner_first.get_bounding_client_rect().top();

    dispatch_action(&document, root, "swap_outer_dirty_inner_first");

    // `play_after` は同期的に実行される。frame 0（rAF 未進行）の視覚
    // 位置が更新直前とほぼ連続していれば、内側リストは自分の位置補正を
    // 行わず（二重補正が起きず）、外側リストの Invert だけが正しく
    // 効いている。
    let frame0_top = inner_first.get_bounding_client_rect().top();
    let jump = (frame0_top - before_top).abs();
    assert!(
        jump < 2.0,
        "入れ子リストの子行の frame 0 は視覚的に不動のはずが、\
         before={before_top} frame0={frame0_top}（差 {jump}px）。内側
         リストが自分でも play すると、外側の補正と二重に適用され跳ぶ"
    );

    // 収束後: inline transform が解除されている（収束したことの確認）。
    sleep_ms(3_000).await;
    let settled_transform = inner_first
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        settled_transform, "",
        "収束後は inline transform が解除されているはずが: {settled_transform}"
    );
}

/// 受け入れ条件（PR #2558 追加ラウンド。`layout_flip.rs` モジュール doc
/// 「入れ子 FLIP リストの所有権契約」節の対称性確認）: dirty 順が
/// `[outer, inner]`（外側→内側）でも
/// `nested_flip_lists_do_not_double_correct_when_dirty_order_is_inner_first`
/// と同じ結果（内側リストの子の frame 0 が視覚的にほぼ不動）になる。
/// `flip_target_lists`/`flip_captured` は `is_same_node` 重複排除のみの
/// 順序非依存な集合であるため、この対称性は構造的に保証される
/// （`Runtime::apply_update_for_dirty` doc「dirty field の列挙順に
/// 依存しない」参照）。
#[wasm_bindgen_test]
async fn nested_flip_lists_do_not_double_correct_when_dirty_order_is_outer_first() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "flip-root-container-11");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = DeepNestedState::new();
    let runtime = Runtime::mount("flip-root-container-11", state).expect("mount must succeed");
    let root = runtime.root();

    let inner_first = root
        .query_selector("li[data-testid='inner-item'][data-key='10']")
        .expect("query_selector must not fail")
        .expect("inner key=10 の行が見つかるはず")
        .dyn_into::<HtmlElement>()
        .expect("keyed list item must be an HtmlElement");

    let before_top = inner_first.get_bounding_client_rect().top();

    dispatch_action(&document, root, "swap_outer_dirty_outer_first");

    let frame0_top = inner_first.get_bounding_client_rect().top();
    let jump = (frame0_top - before_top).abs();
    assert!(
        jump < 2.0,
        "dirty 順が外側→内側でも、入れ子リストの子行の frame 0 は \
         視覚的に不動のはずが、before={before_top} frame0={frame0_top}\
         （差 {jump}px）。dirty 順に結果が依存していないか二重補正が \
         起きている"
    );

    sleep_ms(3_000).await;
    let settled_transform = inner_first
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        settled_transform, "",
        "収束後は inline transform が解除されているはずが: {settled_transform}"
    );
}

/// 受け入れ条件（PR #2558 追加ラウンド。Bugbot Medium
/// `PRRT_kwDOTarxgc6iaXdt`・codex P1 `PRRT_kwDOTarxgc6iaec4` 是正確認）:
/// 外側 FLIP が進行中のときに、内側リストのみ（`outer` を含まない
/// `dirty = ["inner"]`）を構造変化させても、外側 FLIP は放置されず
/// 停止・再捕捉される。
///
/// 検証方法: (1) `swap_outer_only` で外側 FLIP を起動する（進行中に
/// する）。(2) 収束を待たずに `swap_inner_only` を発行する——`inner`
/// field 自体が `FLIP_AUTO_ATTR` 付き keyed list であるため、
/// `find_list_element(root, "inner")` → `flip_lists_containing` の走査
/// （`Runtime::apply_update_for_dirty` 走査 (i)）で外側リストも祖先と
/// して見つかり、`outer` が dirty に含まれていなくても停止・再捕捉の
/// 対象になる契約を確認する。(3) 再捕捉が正しく行われていれば、進行中
/// だった外側 FLIP の中間視覚位置がそのまま新しい First として引き継が
/// れるため、`outer-item` 行（`id=1`、内側リストの親）の位置は
/// 再捕捉の瞬間に不連続な跳びを起こさない。(4) 最終的に収束し、
/// `outer-item` 行の inline transform が解除される（孤立した
/// `AnimationLoop` が残っていれば、収束後も transform が残留し続ける
/// ため、これが「放置されず正しく完了した」ことの確認になる）。
#[wasm_bindgen_test]
async fn inner_only_dirty_stops_and_recaptures_in_progress_outer_flip() {
    let document = web_sys::window().unwrap().document().unwrap();
    let placeholder = create_placeholder(&document, "flip-root-container-12");
    let _guard = RemoveOnDrop(placeholder.clone());

    let state = DeepNestedState::new();
    let runtime = Runtime::mount("flip-root-container-12", state).expect("mount must succeed");
    let root = runtime.root();

    // 外側リストの id=1 行（内側リストの親）を追跡対象にする。
    let outer_row1 = root
        .query_selector("#outer-list > li[data-key='1']")
        .expect("query_selector must not fail")
        .expect("outer key=1 の行が見つかるはず")
        .dyn_into::<HtmlElement>()
        .expect("keyed list item must be an HtmlElement");

    // (1) 外側 FLIP を起動する（`outer` のみ dirty、`id=1` の行が移動）。
    dispatch_action(&document, root, "swap_outer_only");

    // アニメーション進行中（spring 収束前）に割り込む。
    sleep_ms(50).await;
    let mid_flight_top = outer_row1.get_bounding_client_rect().top();

    // (2) `inner` のみ dirty にする（`outer` は dirty に含めない）。
    dispatch_action(&document, root, "swap_inner_only");

    // (3) 再捕捉直後（rAF 未進行の frame 0）の視覚位置が、割り込み直前の
    // 進行中の視覚位置とほぼ連続していることを確認する（放置されていれば
    // 新旧の FlipAnimation が競合し不連続な跳びが起きる）。
    let recapture_top = outer_row1.get_bounding_client_rect().top();
    let jump = (recapture_top - mid_flight_top).abs();
    assert!(
        jump < 2.0,
        "内側のみ dirty な更新で外側 FLIP が正しく再捕捉されていれば \
         視覚位置は連続するはずが、mid_flight={mid_flight_top} \
         recapture={recapture_top}（差 {jump}px）"
    );

    // (4) 収束後、外側 FLIP が孤立せず完了し、inline transform が解除
    // されている。
    sleep_ms(3_000).await;
    let settled_transform = outer_row1
        .style()
        .get_property_value("transform")
        .expect("get_property_value must not fail");
    assert_eq!(
        settled_transform, "",
        "収束後は inline transform が解除されているはずが（孤立した \
         AnimationLoop が残っている可能性）: {settled_transform}"
    );

    // 副次確認: `inner` の並べ替え自体は正しく反映されている
    // （key=20 が key=10 より DOM 順で先になる）。
    let inner_keys: Vec<String> = {
        let nodes = root
            .query_selector_all("li[data-testid='inner-item']")
            .expect("query_selector_all must not fail");
        (0..nodes.length())
            .map(|i| {
                nodes
                    .get(i)
                    .expect("index within length must exist")
                    .dyn_into::<Element>()
                    .expect("must be an Element")
                    .get_attribute(fandhe_frontend_core::keyed::KEY_ATTR)
                    .expect("data-key must be present")
            })
            .collect()
    };
    assert_eq!(
        inner_keys,
        vec!["20", "10"],
        "swap_inner_only は inner=[10,20] を [20,10] へ並べ替えるはずが: {inner_keys:?}"
    );
}

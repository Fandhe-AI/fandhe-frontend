//! `interactive-vt-wasm`: `examples/interactive-view-transitions`
//! （イシュー #503）が同梱する CSR wasm ビルドの薄い glue クレート。
//!
//! # 役割・責務境界
//!
//! `fandhe-frontend-wasm-full`（crates.io バージョン依存。正本は
//! `crates/wasm-full/`）が `#[wasm_bindgen]` エクスポートとして既に定義
//! している `hydrate` / `mount` / `start_router`（`wasm-full/src/entry.rs`）を
//! 再エクスポートする。
//!
//! `hydrate`（`AppState` のカウンター・フォーム・動的リストデモ、
//! `id="interactive-root"`）と `start_router`（`layout()` が組む
//! `<div id="app-root">` の一覧・詳細ページ系）は**別系統・別 DOM**である
//! （`wasm-full::entry` の doc 参照）。`static/embed.html` は両方を異なる
//! `root_id` で呼び出す。
//!
//! イシュー #1199 で [`nav_overlays::hydrate_navigation_menu`]/
//! [`nav_overlays::hydrate_menubar`] を追加した。`fandhe_frontend_wasm_full::Runtime<C>`
//! は `DirtyTracked + BindingSource` を要求する（`docs/design/wasm-full-architecture.md`
//! 第 3.3 節）ため、headless-ui の `NavigationMenu`/`Menubar`
//! （`Component`/`Hydrate` のみ実装）はそのまま載らない。これらの関数は
//! `wasm-full::entry` と同型の**アプリ側の薄いラッパー**（同モジュール doc
//! 「自コンポーネントを持つアプリケーションは、本モジュールと同型のラッパーを
//! 自身のクレートに実装する前提」）の参照実装であり、`headless::wire_headless_component`/
//! `keynav::wire_keynav`/`overlay::OverlayCloseController`/
//! `position::PositionController`（`fandhe-frontend-wasm-full` 0.6.0/0.7.0 で
//! 追加されたオーバーレイ配線 API）を組み合わせて配線する。
//!
//! # HTML 組み立て・DOM 操作の不変条件（REQ-1）
//!
//! [`nav_overlays`] 内の再描画は必ず headless-ui のパーツ関数
//! （`navigation_menu::*`/`menubar::*`）で組み立てた `Node` を
//! `fandhe_frontend_core::render`（既定エスケープ）へ通してから
//! `Element::set_inner_html` へ渡す（`fandhe-frontend-wasm-full::dom::mount_initial`
//! の内部実装と同型のパターン）。`format!` によるタグ文字列の直接組み立て・
//! `raw_html()` の呼び出しは一切行わない。
//!
//! # 呼び出し元
//!
//! `tools/wasm/build.sh` が `wasm-bindgen --target web` でこのクレートの
//! `.wasm` を後処理し、`static/wasm/fandhe_frontend_wasm_full.js` /
//! `fandhe_frontend_wasm_full_bg.wasm` を生成する（`--out-name
//! fandhe_frontend_wasm_full` で glue クレート名に依存させず、
//! `static/embed.html` の import パスと整合させる）。`static/embed.html` は
//! この glue クレートの存在を意識しない（`hydrate`/`mount`/`start_router`/
//! `hydrate_navigation_menu`/`hydrate_menubar` という関数名契約のみに依存する）。
#![deny(unsafe_code)]

// `fandhe-frontend-wasm-full` の `hydrate`/`mount`/`start_router` は
// `#[cfg(target_arch = "wasm32")]` の `entry` モジュール（`wasm-full/src/lib.rs`）
// にのみ存在する。本クレートを誤って native ターゲットで `cargo build`
// された場合に「unresolved import」で失敗するのを避け、意図が伝わる
// 空クレートとして振る舞わせるため、再エクスポート自体を wasm32 に限定する
// （`tools/wasm/build.sh` は常に `--target wasm32-unknown-unknown` を指定する
// ため、実運用の経路には影響しない）。
#[cfg(target_arch = "wasm32")]
pub use fandhe_frontend_wasm_full::entry::{hydrate, mount, start_router};

#[cfg(target_arch = "wasm32")]
pub use nav_overlays::{hydrate_menubar, hydrate_navigation_menu};

/// navigation-menu / menubar のハイドレーション・オーバーレイ配線
/// （イシュー #1199、モジュール冒頭 doc 参照）。
///
/// `fandhe-frontend-wasm-full::entry` と同じ理由で wasm32 限定モジュールと
/// する（native ビルドで `unresolved import` にしない）。
#[cfg(target_arch = "wasm32")]
mod nav_overlays {
    use fandhe_frontend_core::{render, text, Node};
    use fandhe_frontend_headless_ui::menubar::{self, Menubar};
    use fandhe_frontend_headless_ui::navigation_menu::{self, NavigationMenu};
    use fandhe_frontend_interactive::{dispatch, Hydrate};
    use fandhe_frontend_wasm_full::headless::wire_headless_component;
    use fandhe_frontend_wasm_full::hydration::{read_hydration_attrs, restore_state};
    use fandhe_frontend_wasm_full::keynav::wire_keynav;
    use fandhe_frontend_wasm_full::overlay::{
        OverlayCloseController, OverlayCloseRequest, OverlayKind,
    };
    use fandhe_frontend_wasm_full::position::PositionController;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;
    use web_sys::{Document, Element, Window};

    // `Runtime<C>`（`entry.rs`）と同じ「マウント後も状態・配線の生存期間を
    // 維持する保持先が必要」という事情（同 doc 「`Runtime` の生存期間」節
    // 参照）から `thread_local!` へ保持する。`*_OVERLAY`/`*_POSITION` も
    // 同じ理由（`OverlayCloseController`/`PositionController` の `Drop` が
    // document/window のリスナーを解除するため、ローカル変数のまま関数を
    // 抜けると即座に解除されてしまう）。`*_PUSHED` はオーバーレイスタック
    // 上の現在の push index（[`sync_nav_menu_overlay`]/[`sync_menubar_overlay`]
    // 参照。本デモは各コンポーネントにつき「高々 1 項目が開く」制約
    // （`NavigationMenu`/`Menubar` いずれも single-open）のため、push index の
    // シフト（`overlay::OverlayCloseRequest::index` doc の不変条件）を気にせず
    // `Option<usize>` 1 個で足りる）。
    thread_local! {
        static NAV_MENU_STATE: RefCell<Option<Rc<RefCell<NavigationMenu>>>> =
            const { RefCell::new(None) };
        static NAV_MENU_OVERLAY: RefCell<Option<OverlayCloseController>> =
            const { RefCell::new(None) };
        static NAV_MENU_PUSHED: RefCell<Option<usize>> = const { RefCell::new(None) };

        static MENUBAR_STATE: RefCell<Option<Rc<RefCell<Menubar>>>> = const { RefCell::new(None) };
        static MENUBAR_OVERLAY: RefCell<Option<OverlayCloseController>> =
            const { RefCell::new(None) };
        static MENUBAR_PUSHED: RefCell<Option<usize>> = const { RefCell::new(None) };
        static MENUBAR_POSITION: RefCell<Option<PositionController>> =
            const { RefCell::new(None) };
    }

    /// navigation-menu デモの項目定義（`value`, 表示ラベル）。
    ///
    /// `examples/interactive-view-transitions/src/main.rs::NAV_MENU_ITEMS`
    /// （SSR 側、`dist/index.html` 検分用）と**同一のマークアップ**を出力
    /// する対の実装であり、両者は独立クレート（別ワークスペース）のため
    /// コード共有できない。片方だけ変更するとブラウザ実演（本クレート）と
    /// SSR 検分結果がドリフトする点に注意（ドリフト検知の機械テストは
    /// スコープ外、README.md の対象外事項参照）。
    const NAV_MENU_ITEMS: [(&str, &str); 2] = [("products", "製品"), ("docs", "ドキュメント")];

    /// [`NavigationMenu`] 状態から navigation-menu デモの完全なマークアップ
    /// を組み立てる（`src/main.rs::nav_menu_view` と同一の構造）。
    fn nav_menu_view(state: &NavigationMenu) -> Node {
        let hydrate_attrs = state.hydration_attrs();
        let hydrate_attrs_ref: Vec<(&str, &str)> = hydrate_attrs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let mut root_attrs: Vec<(&str, &str)> =
            vec![("id", "nav-menu-root"), ("data-testid", "nav-menu-root")];
        root_attrs.extend(hydrate_attrs_ref);

        let items: Vec<Node> = NAV_MENU_ITEMS
            .iter()
            .map(|(value, label)| {
                let trigger_id = format!("nav-menu-{value}-trigger");
                let content_id = format!("nav-menu-{value}-content");
                let link_href = format!("/nav-menu/{value}");
                let link_label = format!("{label}を見る");
                state.item(
                    value,
                    false,
                    vec![],
                    vec![
                        state.trigger(
                            value,
                            false,
                            Some(&trigger_id),
                            Some(&content_id),
                            vec![],
                            vec![text(*label)],
                        ),
                        state.content(
                            value,
                            Some(&content_id),
                            Some(&trigger_id),
                            vec![],
                            vec![navigation_menu::link(
                                &link_href,
                                false,
                                vec![],
                                vec![text(&link_label)],
                            )],
                        ),
                    ],
                )
            })
            .collect();

        navigation_menu::root(
            "製品・ドキュメントナビゲーション",
            root_attrs,
            vec![navigation_menu::list(vec![], items)],
        )
    }

    /// menubar デモの項目定義（表示ラベル, 配下メニュー項目ラベル一覧）。
    /// [`NAV_MENU_ITEMS`] と同じ「SSR 側との対の実装・ドリフト禁止」注記が
    /// 適用される（`src/main.rs::MENUBAR_MENUS` 参照）。
    const MENUBAR_MENUS: [(&str, [&str; 2]); 2] = [
        ("ファイル", ["新規", "開く"]),
        ("編集", ["コピー", "貼り付け"]),
    ];

    /// [`Menubar`] 状態から menubar デモの完全なマークアップを組み立てる
    /// （`src/main.rs::menubar_view` と同一の構造）。
    fn menubar_view(state: &Menubar) -> Node {
        let hydrate_attrs = state.hydration_attrs();
        let hydrate_attrs_ref: Vec<(&str, &str)> = hydrate_attrs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let mut root_attrs: Vec<(&str, &str)> =
            vec![("id", "menubar-root"), ("data-testid", "menubar-root")];
        root_attrs.extend(hydrate_attrs_ref);

        let menus: Vec<Node> = MENUBAR_MENUS
            .iter()
            .enumerate()
            .map(|(index, (label, items))| {
                let trigger_id = format!("menubar-trigger-{index}");
                let content_id = format!("menubar-content-{index}");
                state.menu(
                    index,
                    vec![],
                    vec![
                        state.trigger(
                            index,
                            false,
                            false,
                            Some(&content_id),
                            vec![("id", &trigger_id)],
                            vec![text(*label)],
                        ),
                        state.positioner(
                            index,
                            vec![],
                            vec![state.content(
                                index,
                                Some(&content_id),
                                Some(&trigger_id),
                                vec![],
                                items
                                    .iter()
                                    .map(|item_label| {
                                        menubar::item(
                                            item_label,
                                            false,
                                            false,
                                            vec![],
                                            vec![text(*item_label)],
                                        )
                                    })
                                    .collect(),
                            )],
                        ),
                    ],
                )
            })
            .collect();

        state.root("アプリケーションメニュー", root_attrs, menus)
    }

    /// `window`/`document` を取得する（`wasm-full::Runtime::get_root`/
    /// `document` と同じ固定文言方針。内部状態を含めない、A03 情報漏えい
    /// 対策）。
    fn window_and_document() -> Result<(Window, Document), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("document is unavailable"))?;
        Ok((window, document))
    }

    /// `value`（`state.open_value()`。dispatch("toggle", payload) の
    /// payload 由来 = trigger 要素の `data-value` 由来であり、devtools 等で
    /// 改ざんされうるクライアント入力）を [`NAV_MENU_ITEMS`] の既知値との
    /// 完全一致でのみ受理し、一致した場合にのみコンパイル時定数の
    /// `(trigger_id, content_id)` を返す。
    ///
    /// セレクタインジェクション対策: `value` を直接 `format!` で CSS
    /// セレクタ文字列へ埋め込まない（`crate::keynav` モジュール doc の
    /// 既存方針「`data-value` 等の属性値からセレクタを組み立てない
    /// （`restore_tree_focus_by_value` と同じ）」をこのデモにも適用する）。
    /// `query_selector` へ渡る文字列は常にこの関数が返す 2 種類の
    /// `&'static str` リテラルのいずれかに限定される。
    fn nav_menu_ids_for_value(value: &str) -> Option<(&'static str, &'static str)> {
        match value {
            "products" => Some(("nav-menu-products-trigger", "nav-menu-products-content")),
            "docs" => Some(("nav-menu-docs-trigger", "nav-menu-docs-content")),
            _ => None,
        }
    }

    fn nav_menu_content_element(root: &Element, value: &str) -> Option<Element> {
        let (_, content_id) = nav_menu_ids_for_value(value)?;
        root.query_selector(&format!("#{content_id}"))
            .ok()
            .flatten()
    }

    fn nav_menu_trigger_element(root: &Element, value: &str) -> Option<Element> {
        let (trigger_id, _) = nav_menu_ids_for_value(value)?;
        root.query_selector(&format!("#{trigger_id}"))
            .ok()
            .flatten()
    }

    /// オーバーレイスタックを現在の `state.open_value()` へ同期する。
    ///
    /// 毎回いったん push 済みエントリを `remove_overlay` してから、開いて
    /// いる項目があれば `root` から**再描画後の**要素を検索し直して
    /// `push_overlay` する（remove→push の順を常に踏む設計）。`root` は
    /// 再描画のたびに [`render_and_sync_nav_menu`] が `set_inner_html` で
    /// DOM を丸ごと差し替えるため、以前 push した `Element` 参照は
    /// デタッチ済みで無効になりうる。毎回 remove してから最新の DOM を
    /// 検索し直すことで、常に有効な `Element` のみをコントローラへ渡す。
    fn sync_nav_menu_overlay(root: &Element, state: &NavigationMenu) {
        NAV_MENU_OVERLAY.with(|controller_cell| {
            let controller_ref = controller_cell.borrow();
            let Some(controller) = controller_ref.as_ref() else {
                return;
            };
            NAV_MENU_PUSHED.with(|pushed_cell| {
                let mut pushed = pushed_cell.borrow_mut();
                if let Some(index) = pushed.take() {
                    controller.remove_overlay(index);
                }
                if let Some(value) = state.open_value() {
                    if let Some(content) = nav_menu_content_element(root, value) {
                        let trigger = nav_menu_trigger_element(root, value);
                        *pushed = controller.push_overlay(&content, trigger.as_ref());
                    }
                }
            });
        });
    }

    /// [`nav_menu_view`] で再描画し、[`sync_nav_menu_overlay`] でオーバー
    /// レイスタックを同期する。`headless::wire_headless_component` の
    /// `on_update`・`OverlayCloseController` の閉鎖コールバックの双方から
    /// 呼ばれる共通経路。
    fn render_and_sync_nav_menu(root: &Element, state: &NavigationMenu) {
        root.set_inner_html(&render(&nav_menu_view(state)));
        sync_nav_menu_overlay(root, state);
    }

    /// navigation-menu のハイドレーション・オーバーレイ配線エントリポイント
    /// （イシュー #1199）。
    ///
    /// `root_id` 要素の `data-hydrate-*` 属性から [`NavigationMenu`] を
    /// 復元する。復元失敗（改ざん・欠落）は `NavigationMenu::default()`
    /// （全項目 closed）での CSR 再描画へ安全側フォールバックする
    /// （`fandhe_frontend_wasm_full::Runtime::hydrate` と同じ契約）。
    ///
    /// クリック配線（`headless::wire_headless_component`、`"toggle"`
    /// dispatch → 再描画）・キーボード配線（`keynav::wire_keynav`、
    /// Arrow/Home/End/Escape）・Escape/外側クリックでの閉鎖
    /// （`overlay::OverlayCloseController`、閉鎖要求を受けて `"deselect"`
    /// dispatch → 再描画）を行う。navigation-menu の `content` は
    /// `positioner` を持たない anatomy（`headless-ui::navigation_menu` に
    /// `positioner` パーツが存在しない）ため `position::PositionController`
    /// は使わない。
    ///
    /// # Errors
    ///
    /// `root_id` に対応する要素が存在しない場合、またはイベント配線
    /// （`add_event_listener_with_callback`）が失敗した場合に `Err` を返す。
    #[wasm_bindgen]
    pub fn hydrate_navigation_menu(root_id: &str) -> Result<(), JsValue> {
        let (_window, document) = window_and_document()?;
        let root = document
            .get_element_by_id(root_id)
            .ok_or_else(|| JsValue::from_str("root element not found"))?;

        let attrs = read_hydration_attrs(&root);
        let state = match restore_state::<NavigationMenu>(&attrs) {
            Ok(restored) => restored,
            Err(_) => {
                let fallback = NavigationMenu::default();
                root.set_inner_html(&render(&nav_menu_view(&fallback)));
                fallback
            }
        };

        let state = Rc::new(RefCell::new(state));
        NAV_MENU_STATE.with(|cell| *cell.borrow_mut() = Some(state.clone()));

        wire_headless_component(root.clone(), state.clone(), |state, root| {
            render_and_sync_nav_menu(root, state);
        })?;

        wire_keynav(root.clone())?;

        let overlay_state = state.clone();
        let overlay_root = root.clone();
        let controller =
            OverlayCloseController::new(&document, move |request: OverlayCloseRequest| {
                if request.kind != OverlayKind::NavigationMenu {
                    return;
                }
                let Ok(mut current) = overlay_state.try_borrow_mut() else {
                    return;
                };
                // "deselect"（payload なし）は既に閉じている状態へ送っても
                // 冪等 no-op のまま安全に収束する（`overlay.rs` モジュール doc
                // §keynav との二重処理の収束 参照）。
                if !dispatch(&mut *current, "deselect", "") {
                    return;
                }
                let snapshot = current.clone();
                drop(current);
                render_and_sync_nav_menu(&overlay_root, &snapshot);
            })?;
        NAV_MENU_OVERLAY.with(|cell| *cell.borrow_mut() = Some(controller));

        // 初回マウント時点で既に開いている項目（SSR 初期状態）があれば
        // オーバーレイスタックへ登録する（以後の Escape/外側クリックで
        // 正しく閉鎖できるようにする）。
        let current = state.borrow();
        sync_nav_menu_overlay(&root, &current);

        Ok(())
    }

    // `nav_menu_ids_for_value` と異なり、こちらは `state.open()` の
    // `Option<usize>` を直接 `format!` へ埋め込んでよい: `usize` は
    // `Display` 実装上 10 進数字のみを出力する型であり、`"`/`#`/`[`/`]`
    // 等のセレクタ構文を注入できない（`nav_menu_*` の `&str` payload と違い
    // 文字列そのものが攻撃者制御になり得ない）。加えて `Menubar::new` の
    // `normalize_open` が常に `index < trigger_count` を保証する
    // （`Menubar::decode_action`/`normalize_open` の fail-closed 契約）。

    fn menubar_content_element(root: &Element, index: usize) -> Option<Element> {
        root.query_selector(&format!("#menubar-content-{index}"))
            .ok()
            .flatten()
    }

    fn menubar_trigger_element(root: &Element, index: usize) -> Option<Element> {
        root.query_selector(&format!("#menubar-trigger-{index}"))
            .ok()
            .flatten()
    }

    /// [`sync_nav_menu_overlay`] と同じ remove→push 方針で menubar の
    /// オーバーレイスタックを同期し、続けて `PositionController::reposition_now`
    /// で開いている positioner の座標を即座に反映する（scroll/resize 契機の
    /// 再計算とは別に、開閉直後の初期配置を確定させるため）。
    fn sync_menubar_overlay(root: &Element, state: &Menubar) {
        MENUBAR_OVERLAY.with(|controller_cell| {
            let controller_ref = controller_cell.borrow();
            let Some(controller) = controller_ref.as_ref() else {
                return;
            };
            MENUBAR_PUSHED.with(|pushed_cell| {
                let mut pushed = pushed_cell.borrow_mut();
                if let Some(index) = pushed.take() {
                    controller.remove_overlay(index);
                }
                if let Some(open_index) = state.open() {
                    if let Some(content) = menubar_content_element(root, open_index) {
                        let trigger = menubar_trigger_element(root, open_index);
                        *pushed = controller.push_overlay(&content, trigger.as_ref());
                    }
                }
            });
        });
        MENUBAR_POSITION.with(|position_cell| {
            if let Some(position) = position_cell.borrow().as_ref() {
                position.reposition_now();
            }
        });
    }

    /// [`menubar_view`] で再描画し、[`sync_menubar_overlay`] でオーバーレイ
    /// スタック・座標を同期する共通経路。
    fn render_and_sync_menubar(root: &Element, state: &Menubar) {
        root.set_inner_html(&render(&menubar_view(state)));
        sync_menubar_overlay(root, state);
    }

    /// menubar のハイドレーション・オーバーレイ配線エントリポイント
    /// （イシュー #1199）。
    ///
    /// `root_id` 要素の `data-hydrate-*` 属性から [`Menubar`] を復元する。
    /// 復元失敗は「トリガー・フォーカス 0・閉状態」（[`Menubar::default`]）
    /// での CSR 再描画へ安全側フォールバックする。
    ///
    /// [`hydrate_navigation_menu`] と同じ 3 配線（クリック・キーボード・
    /// Escape/外側クリック）に加え、`position::PositionController`
    /// （scroll/resize 契機の再計算）を `thread_local!` へ 1 個保持する
    /// （menubar の `content` は `positioner` パーツを持つ anatomy のため）。
    ///
    /// # Errors
    ///
    /// [`hydrate_navigation_menu`] と同じ条件（`root_id` 未検出・イベント
    /// 配線失敗）に加え、`PositionController::new`（scroll/resize リスナー
    /// 登録）の失敗時に `Err` を返す。
    #[wasm_bindgen]
    pub fn hydrate_menubar(root_id: &str) -> Result<(), JsValue> {
        let (window, document) = window_and_document()?;
        let root = document
            .get_element_by_id(root_id)
            .ok_or_else(|| JsValue::from_str("root element not found"))?;

        let attrs = read_hydration_attrs(&root);
        let state = match restore_state::<Menubar>(&attrs) {
            Ok(restored) => restored,
            Err(_) => {
                let fallback = Menubar::default();
                root.set_inner_html(&render(&menubar_view(&fallback)));
                fallback
            }
        };

        let state = Rc::new(RefCell::new(state));
        MENUBAR_STATE.with(|cell| *cell.borrow_mut() = Some(state.clone()));

        wire_headless_component(root.clone(), state.clone(), |state, root| {
            render_and_sync_menubar(root, state);
        })?;

        wire_keynav(root.clone())?;

        MENUBAR_POSITION.with(|cell| -> Result<(), JsValue> {
            *cell.borrow_mut() = Some(PositionController::new(&window)?);
            Ok(())
        })?;

        let overlay_state = state.clone();
        let overlay_root = root.clone();
        let controller =
            OverlayCloseController::new(&document, move |request: OverlayCloseRequest| {
                if request.kind != OverlayKind::Menubar {
                    return;
                }
                let Ok(mut current) = overlay_state.try_borrow_mut() else {
                    return;
                };
                // "close"（payload なし、MenubarAction::Close）は全 Menu を
                // 閉じる冪等操作（`overlay.rs` モジュール doc §イシュー #1173
                // 参照）。
                if !dispatch(&mut *current, "close", "") {
                    return;
                }
                let snapshot = *current;
                drop(current);
                render_and_sync_menubar(&overlay_root, &snapshot);
            })?;
        MENUBAR_OVERLAY.with(|cell| *cell.borrow_mut() = Some(controller));

        // 初回マウント時点で既に開いている Menu（SSR 初期状態）があれば
        // オーバーレイスタック・座標を初期同期する。
        let current = *state.borrow();
        sync_menubar_overlay(&root, &current);

        Ok(())
    }
}

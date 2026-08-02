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
    use fandhe_frontend_headless_ui::data_attrs::Orientation;
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
    // 参照）から `thread_local!` へ保持する。`SHARED_OVERLAY`/`MENUBAR_POSITION`
    // も同じ理由（`OverlayCloseController`/`PositionController` の `Drop` が
    // document/window のリスナーを解除するため、ローカル変数のまま関数を
    // 抜けると即座に解除されてしまう）。
    //
    // `SHARED_OVERLAY` は navigation-menu・menubar の**両方**が 1 個の
    // `OverlayCloseController`（1 つの document keydown/pointerdown リスナー・
    // 1 本のスタック）を共有する（イシュー #1200 Bugbot 指摘の修正）。
    // 当初は各コンポーネントが独立した `OverlayCloseController` を持って
    // いたが、本デモは SSR 時点で両オーバーレイを同時に開いた状態にするため、
    // 独立スタックだとそれぞれが「自分のスタックの最上位＝自分」と誤判定し、
    // Escape 1 回で両方が閉じてしまっていた（`overlay::escape_close_index` は
    // 渡されたスタックの最上位のみを対象とする設計であり、スタックを分けると
    // 「アプリ全体での最上位」を判定できない）。`NAV_MENU_ROOT`/`MENUBAR_ROOT`
    // は共有コントローラのコールバック（[`close_nav_menu_overlay`]/
    // [`close_menubar_overlay`]、クロージャ生成時点では対象の `root` 要素が
    // まだ確定していないため）が事後に参照する保持先。
    thread_local! {
        static NAV_MENU_STATE: RefCell<Option<Rc<RefCell<NavigationMenu>>>> =
            const { RefCell::new(None) };
        static NAV_MENU_ROOT: RefCell<Option<Element>> = const { RefCell::new(None) };

        static MENUBAR_STATE: RefCell<Option<Rc<RefCell<Menubar>>>> = const { RefCell::new(None) };
        static MENUBAR_ROOT: RefCell<Option<Element>> = const { RefCell::new(None) };
        static MENUBAR_POSITION: RefCell<Option<PositionController>> =
            const { RefCell::new(None) };

        static SHARED_OVERLAY: RefCell<Option<OverlayCloseController>> =
            const { RefCell::new(None) };
    }

    /// navigation-menu デモの項目定義（`value`, 表示ラベル）。
    ///
    /// `examples/interactive-view-transitions/src/main.rs::NAV_MENU_ITEMS`
    /// （SSR 側、`dist/index.html` 検分用）と**同一のマークアップ**を出力
    /// する対の実装であり、両者は独立クレート（別ワークスペース）のため
    /// コード共有できない。片方だけ変更するとブラウザ実演（本クレート）と
    /// SSR 検分結果がドリフトする点に注意（この定数を含む
    /// `// fw-drift-guard:begin`/`:end` 区間は
    /// `crates/cli/tests/example_view_drift.rs` がインデント正規化後の完全
    /// 一致を機械検証する。イシュー #1202、PR #1200 out-of-scope の解消）。
    // fw-drift-guard:begin nav-menu-items
    const NAV_MENU_ITEMS: [(&str, &str); 2] = [("products", "製品"), ("docs", "ドキュメント")];
    // fw-drift-guard:end nav-menu-items

    /// 複数の兄弟 [`Node`] を連結してレンダリングする（[`fandhe_frontend_core::render`]
    /// は単一 `Node` しか受け取らないため）。既定エスケープは各 `Node` ごとの
    /// `render` 呼び出しに閉じているため、連結しても不変条件は保たれる。
    fn render_nodes(nodes: &[Node]) -> String {
        nodes.iter().map(render).collect()
    }

    /// `attrs`（[`Hydrate::hydration_attrs`] の戻り値等）を `root` へ
    /// `set_attribute` で反映する（`wasm-full::focus_visible`/`position` 等の
    /// 「薄いガード付きラッパー」と同じ best-effort 方針。属性名は
    /// フレームワーク側が生成する `data-hydrate-*` の固定語彙のみであり、
    /// 失敗（不正属性名）は実運用上発生しない想定のため戻り値は無視する）。
    fn apply_root_hydrate_attrs(root: &Element, attrs: &[(String, String)]) {
        for (name, value) in attrs {
            let _ = root.set_attribute(name, value);
        }
    }

    /// [`NavigationMenu`] 状態から navigation-menu デモの**内容**（root 要素
    /// の子ノード列）を組み立てる（`src/main.rs::nav_menu_view` の
    /// `navigation_menu::list` 部分と同一の構造）。
    ///
    /// root 要素自体（`id="nav-menu-root"` 等）は含めない。root は
    /// [`hydrate_navigation_menu`] が DOM 上に既に確保している要素であり、
    /// [`render_and_sync_nav_menu`] がこの戻り値を `root.set_inner_html` へ
    /// 渡す（イシュー #1200 Bugbot 指摘の修正: 従来はここで root 要素込みの
    /// `Node` を組み立てて `set_inner_html` していたため、既存の root 要素の
    /// 内側にもう一つ `id="nav-menu-root"` の要素がネストされ、再描画のたびに
    /// ID が重複する無効なマークアップになっていた）。
    fn nav_menu_content(state: &NavigationMenu) -> Vec<Node> {
        // fw-drift-guard:begin nav-menu-item-nodes
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
        // fw-drift-guard:end nav-menu-item-nodes

        vec![navigation_menu::list(vec![], items)]
    }

    /// menubar デモの項目定義（表示ラベル, 配下メニュー項目ラベル一覧）。
    /// [`NAV_MENU_ITEMS`] と同じ「SSR 側との対の実装・ドリフト禁止」注記が
    /// 適用される（`src/main.rs::MENUBAR_MENUS` 参照）。
    // fw-drift-guard:begin menubar-menus
    const MENUBAR_MENUS: [(&str, [&str; 2]); 2] = [
        ("ファイル", ["新規", "開く"]),
        ("編集", ["コピー", "貼り付け"]),
    ];
    // fw-drift-guard:end menubar-menus

    /// [`Menubar`] 状態から menubar デモの**内容**（root 要素の子ノード列）
    /// を組み立てる（`src/main.rs::menubar_view` の `menus` 部分と同一の構造）。
    ///
    /// [`nav_menu_content`] と同じ理由（イシュー #1200 Bugbot 指摘）で root
    /// 要素自体は含めない。
    fn menubar_content(state: &Menubar) -> Vec<Node> {
        MENUBAR_MENUS
            .iter()
            .enumerate()
            // fw-drift-guard:begin menubar-menu-map
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
            // fw-drift-guard:end menubar-menu-map
            .collect()
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

    /// [`SHARED_OVERLAY`] が未初期化なら 1 度だけ構築する
    /// （navigation-menu/menubar のどちらの `hydrate_*` が先に呼ばれても、
    /// 2 個目の呼び出しでは既存のコントローラを再利用する）。callback は
    /// 種別（`request.kind`）で分岐し、対応する [`close_nav_menu_overlay`]/
    /// [`close_menubar_overlay`] を呼ぶ（イシュー #1200 Bugbot 指摘の修正、
    /// `SHARED_OVERLAY` の thread_local doc 参照）。
    fn ensure_shared_overlay(document: &Document) -> Result<(), JsValue> {
        let already_initialized = SHARED_OVERLAY.with(|cell| cell.borrow().is_some());
        if already_initialized {
            return Ok(());
        }
        let controller = OverlayCloseController::new(
            document,
            move |request: OverlayCloseRequest| match request.kind {
                OverlayKind::NavigationMenu => close_nav_menu_overlay(),
                OverlayKind::Menubar => close_menubar_overlay(),
                _ => {}
            },
        )?;
        SHARED_OVERLAY.with(|cell| *cell.borrow_mut() = Some(controller));
        Ok(())
    }

    /// `SHARED_OVERLAY` からの Escape/外側クリック閉鎖要求
    /// （`OverlayKind::NavigationMenu`）を処理する。`NAV_MENU_ROOT`/
    /// `NAV_MENU_STATE` から対象を復元し、`"deselect"` dispatch → 再描画する
    /// （[`hydrate_navigation_menu`] が従来クロージャ内で直接行っていた処理を、
    /// コントローラ共有化〔[`ensure_shared_overlay`]〕に伴い独立関数へ
    /// 切り出したもの）。
    fn close_nav_menu_overlay() {
        let Some(root) = NAV_MENU_ROOT.with(|cell| cell.borrow().clone()) else {
            return;
        };
        let Some(state) = NAV_MENU_STATE.with(|cell| cell.borrow().clone()) else {
            return;
        };
        let Ok(mut current) = state.try_borrow_mut() else {
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
        render_and_sync_nav_menu(&root, &snapshot);
    }

    /// [`nav_menu_content`] で再描画し、[`sync_shared_overlays`] でオーバー
    /// レイスタックを同期する。`headless::wire_headless_component` の
    /// `on_update`・`SHARED_OVERLAY` の閉鎖コールバック
    /// （[`close_nav_menu_overlay`]）の双方から呼ばれる共通経路。
    ///
    /// `root` は既に DOM 上に存在する要素（[`hydrate_navigation_menu`] が
    /// `get_element_by_id` で解決済み）であり、[`nav_menu_content`] の
    /// 戻り値（root 要素**を含まない**子ノード列）のみを `set_inner_html` へ
    /// 渡す（イシュー #1200 Bugbot 指摘の修正: root 要素込みの `Node` を渡す
    /// と、既存の root 要素の内側にもう一つ同じ `id` の要素がネストされて
    /// いた）。`data-hydrate-*` 属性は [`apply_root_hydrate_attrs`] で
    /// `root` 自身に直接反映する。
    fn render_and_sync_nav_menu(root: &Element, state: &NavigationMenu) {
        apply_root_hydrate_attrs(root, &state.hydration_attrs());
        root.set_inner_html(&render_nodes(&nav_menu_content(state)));
        // `wire_headless_component` の `on_update`（この関数の呼び出し元の一つ）
        // は `state`（`NAV_MENU_STATE` の中身）へのミュータブル借用を保持した
        // まま呼ばれる契約（`headless.rs` rustdoc）。`sync_shared_overlays()` は
        // 内部で同じ `Rc<RefCell<NavigationMenu>>` へ `try_borrow()` するため、
        // on_update 経路では常に再入 Err となりオーバーレイスタックへの push が
        // 無条件でスキップされていた（イシュー #1209）。ここでは既に手元にある
        // `state` 参照をスナップショットとして渡し、再入を回避する。
        sync_shared_overlays_with(Some(state), None);
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
    /// （[`ensure_shared_overlay`] が構築する `SHARED_OVERLAY`、閉鎖要求を
    /// 受けて `"deselect"` dispatch → 再描画）を行う。navigation-menu の
    /// `content` は `positioner` を持たない anatomy
    /// （`headless-ui::navigation_menu` に `positioner` パーツが存在しない）
    /// ため `position::PositionController` は使わない。
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
                apply_root_hydrate_attrs(&root, &fallback.hydration_attrs());
                root.set_inner_html(&render_nodes(&nav_menu_content(&fallback)));
                fallback
            }
        };

        let state = Rc::new(RefCell::new(state));
        NAV_MENU_STATE.with(|cell| *cell.borrow_mut() = Some(state.clone()));
        NAV_MENU_ROOT.with(|cell| *cell.borrow_mut() = Some(root.clone()));

        wire_headless_component(root.clone(), state.clone(), |state, root| {
            render_and_sync_nav_menu(root, state);
        })?;

        wire_keynav(root.clone())?;

        ensure_shared_overlay(&document)?;

        // 初回マウント時点で既に開いている項目（SSR 初期状態）があれば
        // オーバーレイスタックへ登録する（以後の Escape/外側クリックで
        // 正しく閉鎖できるようにする）。
        sync_shared_overlays();

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

    /// `SHARED_OVERLAY` からの Escape/外側クリック閉鎖要求
    /// （`OverlayKind::Menubar`）を処理する。[`close_nav_menu_overlay`] と
    /// 対になる関数。
    fn close_menubar_overlay() {
        let Some(root) = MENUBAR_ROOT.with(|cell| cell.borrow().clone()) else {
            return;
        };
        let Some(state) = MENUBAR_STATE.with(|cell| cell.borrow().clone()) else {
            return;
        };
        let Ok(mut current) = state.try_borrow_mut() else {
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
        render_and_sync_menubar(&root, &snapshot);
    }

    /// `controller` のスタックを空にする。常に**末尾（最上位）**から
    /// `remove_overlay` するため、`overlay::OverlayCloseRequest::index` doc が
    /// 警告する「非最上位 remove による上位 index のシフト」は発生しない
    /// （[`sync_shared_overlays`] が毎回スタック全体を作り直す設計の要）。
    fn clear_shared_overlay_stack(controller: &OverlayCloseController) {
        while controller.stack_len() > 0 {
            controller.remove_overlay(controller.stack_len() - 1);
        }
    }

    /// navigation-menu・menubar 双方の現在の開閉状態から `SHARED_OVERLAY` の
    /// スタックを作り直す（イシュー #1200 Bugbot 指摘の修正）。
    ///
    /// 当初はコンポーネントごとに「自分の push index 1 個」を
    /// `Option<usize>` で追跡していたが、これは各コンポーネントが**独立した**
    /// `OverlayCloseController`（独立したスタック）を持つ設計を前提にした
    /// ものだった。両オーバーレイが 1 個の `SHARED_OVERLAY` を共有する
    /// ようになった結果、片方の `remove_overlay` がもう片方の push index を
    /// シフトさせうる問題（`overlay::OverlayCloseRequest::index` doc の
    /// 不変条件）が生じる。個別の index を追跡する代わりに、状態変化の
    /// たびに [`clear_shared_overlay_stack`] でスタック全体を空にしてから
    /// 現在開いている項目だけを push し直すことで、index の追跡・同期を
    /// 一切不要にする（本デモの規模〈高々 2 エントリ〉では毎回の作り直しは
    /// 無視できるコスト）。
    ///
    /// `NAV_MENU_STATE`/`MENUBAR_STATE`（`hydrate_navigation_menu`/
    /// `hydrate_menubar` 末尾の初期同期、`close_nav_menu_overlay`/
    /// `close_menubar_overlay` の借用 drop 後）から両状態を読む既定経路。
    /// `wire_headless_component` の `on_update` 経路（対象状態への
    /// ミュータブル借用が生存したまま呼ばれる）からは呼ばない
    /// （[`sync_shared_overlays_with`] を使う。イシュー #1209）。
    fn sync_shared_overlays() {
        sync_shared_overlays_with(None, None);
    }

    /// [`sync_shared_overlays`] の本体。navigation-menu/menubar それぞれの
    /// 開閉状態を、呼び出し元から渡されたスナップショット（`Some`）が
    /// あればそれを使い、なければ `NAV_MENU_STATE`/`MENUBAR_STATE` から
    /// `try_borrow()` で読む（`None`）。
    ///
    /// `wire_headless_component`（`crates/wasm-full/src/headless.rs`）の
    /// `on_update` コールバックは対象コンポーネントの `Rc<RefCell<C>>` を
    /// `try_borrow_mut()` で借用したまま呼ばれる仕様（wasm-full 側の明示
    /// 契約であり本 example 側では変更しない）。[`render_and_sync_nav_menu`]/
    /// [`render_and_sync_menubar`] はこの `on_update` からも呼ばれるため、
    /// 内部で同じ `RefCell` へ再度 `try_borrow()` すると必ず `Err` になり、
    /// [`clear_shared_overlay_stack`] でスタックを空にした後の push だけが
    /// 無条件でスキップされていた（click で開いた項目が `SHARED_OVERLAY` へ
    /// 登録されず、Escape・外側クリックで閉じられない不具合、イシュー
    /// #1209）。呼び出し元が既に保持している `&C` をスナップショットとして
    /// 渡すことで、この再入を構造的に回避する。
    fn sync_shared_overlays_with(
        nav_menu_snapshot: Option<&NavigationMenu>,
        menubar_snapshot: Option<&Menubar>,
    ) {
        SHARED_OVERLAY.with(|controller_cell| {
            let controller_ref = controller_cell.borrow();
            let Some(controller) = controller_ref.as_ref() else {
                return;
            };
            clear_shared_overlay_stack(controller);

            if let Some(root) = NAV_MENU_ROOT.with(|cell| cell.borrow().clone()) {
                // スナップショット優先。渡されなかった場合は既定経路
                // （借用外からの呼び出し）として `try_borrow()` を試みる
                // （防御的 fail-closed。再入時は元のまま no-op）。
                let open_value: Option<String> = match nav_menu_snapshot {
                    Some(state) => state.open_value().map(str::to_string),
                    None => NAV_MENU_STATE.with(|cell| {
                        cell.borrow().as_ref().and_then(|state| {
                            state
                                .try_borrow()
                                .ok()
                                .and_then(|current| current.open_value().map(str::to_string))
                        })
                    }),
                };
                if let Some(value) = open_value {
                    if let Some(content) = nav_menu_content_element(&root, &value) {
                        let trigger = nav_menu_trigger_element(&root, &value);
                        let _ = controller.push_overlay(&content, trigger.as_ref());
                    }
                }
            }

            if let Some(root) = MENUBAR_ROOT.with(|cell| cell.borrow().clone()) {
                let open_index: Option<usize> = match menubar_snapshot {
                    Some(state) => state.open(),
                    None => MENUBAR_STATE.with(|cell| {
                        cell.borrow().as_ref().and_then(|state| {
                            state.try_borrow().ok().and_then(|current| current.open())
                        })
                    }),
                };
                if let Some(open_index) = open_index {
                    if let Some(content) = menubar_content_element(&root, open_index) {
                        let trigger = menubar_trigger_element(&root, open_index);
                        let _ = controller.push_overlay(&content, trigger.as_ref());
                    }
                }
            }
        });

        MENUBAR_POSITION.with(|position_cell| {
            if let Some(position) = position_cell.borrow().as_ref() {
                position.reposition_now();
            }
        });
    }

    /// [`menubar_content`] で再描画し、[`sync_shared_overlays`] でオーバーレイ
    /// スタック・座標を同期する共通経路（[`render_and_sync_nav_menu`] と
    /// 同じ「root 要素自体は書き換えず子ノードのみ差し替える」契約、イシュー
    /// #1200 Bugbot 指摘の修正）。
    fn render_and_sync_menubar(root: &Element, state: &Menubar) {
        apply_root_hydrate_attrs(root, &state.hydration_attrs());
        root.set_inner_html(&render_nodes(&menubar_content(state)));
        // [`render_and_sync_nav_menu`] と同じ理由（イシュー #1209）で、
        // `wire_headless_component` の `on_update` 経路では `state`
        // （`MENUBAR_STATE` の中身）への再入借用を避けるためスナップショットを
        // 直接渡す。
        sync_shared_overlays_with(None, Some(state));
    }

    /// menubar のハイドレーション・オーバーレイ配線エントリポイント
    /// （イシュー #1199）。
    ///
    /// `root_id` 要素の `data-hydrate-*` 属性から [`Menubar`] を復元する。
    /// 復元失敗は「[`MENUBAR_MENUS`] と同じ 2 トリガー・フォーカス 0・
    /// 閉状態」での CSR 再描画へ安全側フォールバックする。`Menubar::default()`
    /// （`trigger_count = 0`）は使わない —— [`menubar_content`] は常に
    /// `MENUBAR_MENUS` の 2 トリガーを描画するため、`trigger_count = 0` の
    /// ままだと `Menubar::normalize_open`/`decode_action` が全 index を
    /// 範囲外として no-op 扱いし、フォールバック後の menubar がクリック
    /// しても一切開閉しなくなる（イシュー #1200 Bugbot 指摘の修正）。
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
                let fallback =
                    Menubar::new(0, MENUBAR_MENUS.len(), None, false, Orientation::Horizontal);
                apply_root_hydrate_attrs(&root, &fallback.hydration_attrs());
                root.set_inner_html(&render_nodes(&menubar_content(&fallback)));
                fallback
            }
        };

        let state = Rc::new(RefCell::new(state));
        MENUBAR_STATE.with(|cell| *cell.borrow_mut() = Some(state.clone()));
        MENUBAR_ROOT.with(|cell| *cell.borrow_mut() = Some(root.clone()));

        wire_headless_component(root.clone(), state.clone(), |state, root| {
            render_and_sync_menubar(root, state);
        })?;

        wire_keynav(root.clone())?;

        MENUBAR_POSITION.with(|cell| -> Result<(), JsValue> {
            *cell.borrow_mut() = Some(PositionController::new(&window)?);
            Ok(())
        })?;

        ensure_shared_overlay(&document)?;

        // 初回マウント時点で既に開いている Menu（SSR 初期状態）があれば
        // オーバーレイスタック・座標を初期同期する。
        sync_shared_overlays();

        Ok(())
    }
}

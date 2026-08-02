//! `fandhe-frontend-example-interactive-view-transitions`: 状態管理 (REQ-8) +
//! View Transitions の正本サンプル（イシュー #503）。
//!
//! # 役割・呼び出し文脈
//!
//! `examples/ssr-routing`（イシュー #499、examples 規約の初例）と同じ
//! 構成規約に従い、crates.io へ公開済みの `fandhe-frontend-core` /
//! `fandhe-frontend-app` / `fandhe-frontend-interactive`（いずれも v0.1.0）を
//! バージョン依存として実際に使う「正本」であり、利用者・AI エージェントが
//! 状態機械（`Component`/`dispatch`/`decode_action`/`render_for_hydration`）を
//! 自作して契約からドリフトするのを防ぐための参照実装として存在する。
//!
//! イシュー #1199 で `fandhe-frontend-headless-ui`（navigation-menu /
//! menubar、0.28.0）への依存を追加し、`fandhe-frontend-wasm-full` 0.6.0/0.7.0
//! で追加された headless-ui オーバーレイ配線
//! （`headless::MAPPING_TABLE`・`overlay::OverlayKind`・`keynav`・
//! `position::PositionedKind` の scope enum 追加）の実演を担う（詳細は
//! `wasm/src/lib.rs` の `hydrate_navigation_menu`/`hydrate_menubar`）。
//!
//! ブラウザでの実動作確認（wasm ビルド・`hydrate`/`start_router`/
//! `hydrate_navigation_menu`/`hydrate_menubar` の実演）は `wasm/`
//! （独立ワークスペースの glue クレート）+ `tools/wasm/build.sh` +
//! `static/embed.html` が担う（本バイナリの責務外。詳細は README.md）。
//!
//! # 実行内容（2 段構成）
//!
//! 1. **native 状態機械実演**: [`fandhe_frontend_interactive::AppState`] /
//!    [`fandhe_frontend_headless_ui::navigation_menu::NavigationMenu`] /
//!    [`fandhe_frontend_headless_ui::menubar::Menubar`] に対する
//!    [`fandhe_frontend_interactive::dispatch`] の呼び出しと、その都度の
//!    戻り値・状態を標準出力へ書き出す。
//! 2. **SSR HTML 書き出し**: `layout` + `list_page`（`start_router` 系統、
//!    `<div id="app-root">`）・`render_for_hydration`（`hydrate` 系統、
//!    `<div id="interactive-root">`）・navigation-menu デモ
//!    （`<nav id="nav-menu-root">`）・menubar デモ（`<div id="menubar-root">`）
//!    を 1 ページに同居させた `page_shell` 出力を `dist/index.html` へ書き出す。
//!    **このファイルは検分用の SSR HTML（`<script>` を含まない）であり、
//!    ブラウザ実行は `static/embed.html` が担う**（責務分離、README.md 参照）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! HTML はすべて `fandhe_frontend_core` のノード木 API（`fandhe_frontend_app::layout` /
//! `list_page` / `page_shell` / `fandhe_frontend_interactive::render_for_hydration` /
//! `fandhe_frontend_headless_ui::{navigation_menu, menubar}` のパーツ関数）で
//! 組み立て、`format!` によるタグ文字列の直接組み立て・`raw_html()` は一切
//! 使わない。`dist/index.html` の書き出し先は固定相対パスであり、ユーザー
//! 入力を経路へ使わない（パストラバーサルの余地なし）。

#![forbid(unsafe_code)]

use fandhe_frontend_app::{demo_items, layout, list_page, page_shell};
use fandhe_frontend_core::{el, render, text, Node};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::menubar::{self, Menubar};
use fandhe_frontend_headless_ui::navigation_menu::{self, NavigationMenu};
use fandhe_frontend_interactive::{dispatch, render_for_hydration, AppState, Component, Hydrate};
use std::error::Error;
use std::fs;

/// navigation-menu デモの項目定義（`value`, 表示ラベル）。
///
/// `wasm/src/lib.rs::nav_menu_view`（glue 側）と**同一のマークアップ**を
/// 出力する対の実装であり、両者は独立クレート（別ワークスペース）のため
/// コード共有できない。片方だけ変更するとブラウザ実演（glue 側）と
/// `dist/index.html` の検分結果（本関数側）がドリフトする点に注意
/// （ドリフト検知の機械テストはスコープ外、README.md の対象外事項参照）。
const NAV_MENU_ITEMS: [(&str, &str); 2] = [("products", "製品"), ("docs", "ドキュメント")];

/// [`NavigationMenu`] 状態から navigation-menu デモの完全なマークアップ
/// （root/list/item/trigger/content/link）を組み立てる。
///
/// [`fandhe_frontend_interactive::render_for_hydration`] は
/// `Component::view()`（[`NavigationMenu`] では「共通契約のみを表す最小
/// 正準ビュー」= 空の `<nav>`）をルートに使うため、本デモのように children
/// を持つ完全なマークアップにはそのまま使えない（同関数 doc 参照）。その
/// ため本関数は [`Hydrate::hydration_attrs`] を root の `attrs` へ直接
/// マージして同等の効果（`data-hydrate-*` 付きルート）を得る。
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
///
/// [`NAV_MENU_ITEMS`] と同じ「glue 側との対の実装・ドリフト禁止」注記が
/// 適用される（`wasm/src/lib.rs::menubar_view` 参照）。
const MENUBAR_MENUS: [(&str, [&str; 2]); 2] = [
    ("ファイル", ["新規", "開く"]),
    ("編集", ["コピー", "貼り付け"]),
];

/// [`Menubar`] 状態から menubar デモの完全なマークアップ
/// （root/menu/trigger/positioner/content/item）を組み立てる。
///
/// [`nav_menu_view`] と同じ理由で `render_for_hydration` を使わず、
/// [`Hydrate::hydration_attrs`] を root の `attrs` へ直接マージする。
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

/// native 状態機械実演。[`AppState`] へ既知アクションを順に `dispatch` し、
/// その都度の戻り値・状態を標準出力へ書き出す。最後に未知アクション名の
/// `dispatch` が `false`（no-op、状態不変）を返す不変条件 4
/// （`fandhe-frontend-interactive` クレートドキュメント参照）を実演する。
fn run_native_demo() {
    let mut state = AppState::new();
    println!("=== native state machine demo (fandhe-frontend-interactive::dispatch) ===");
    println!("initial state: {state:?}");

    for (name, payload) in [
        ("increment", ""),
        ("increment", ""),
        ("set_draft", "wasm glue crate"),
        ("add_item", ""),
    ] {
        let applied = dispatch(&mut state, name, payload);
        println!("dispatch({name:?}, {payload:?}) -> applied={applied}, state={state:?}");
    }

    // 未知アクション名は `decode_action` の復号失敗として no-op になる
    // （`fandhe-frontend-interactive` の不変条件 4、安全側フォールバック）。
    let unknown_applied = dispatch(&mut state, "no-such-action", "");
    println!(
        "dispatch(\"no-such-action\", \"\") -> applied={unknown_applied} (unknown action names are a safe no-op)"
    );

    println!("=== render(&state.view()) output ===");
    println!("{}", render(&state.view()));
}

/// navigation-menu 状態機械実演（イシュー #1199）。
///
/// `("navigation-menu", "trigger")` → `"toggle"`（`crates/wasm-full/src/headless.rs`
/// の `MAPPING_TABLE`）と `"deselect"`（`overlay::OverlayCloseController` の
/// 閉鎖要求を受けた呼び出し側が dispatch するアクション、`overlay.rs`
/// モジュール doc §イシュー #1173 参照）を native 側で実演する。
fn run_navigation_menu_demo() {
    let mut state = NavigationMenu::default();
    println!("=== navigation-menu state machine demo ===");
    println!("initial state: {state:?}");

    for (name, payload) in [
        ("toggle", "products"),
        // 既に開いている項目の再クリックは disclosure nav として閉じる
        // （headless.rs コメント「開いている項目の再クリックで閉じる」）。
        ("toggle", "products"),
        ("toggle", "docs"),
        // OverlayCloseController の閉鎖要求（Escape・外側クリック）を
        // 受けた呼び出し側が dispatch する冪等操作。
        ("deselect", ""),
    ] {
        let applied = dispatch(&mut state, name, payload);
        println!("dispatch({name:?}, {payload:?}) -> applied={applied}, state={state:?}");
    }
}

/// menubar 状態機械実演（イシュー #1199）。
///
/// `("menubar", "trigger")` → `"toggle"`（payload は Menu の index）と
/// `"close"`（`OverlayCloseRequest` を受けた呼び出し側が dispatch する
/// `MenubarAction::Close`、`overlay.rs` モジュール doc §イシュー #1173
/// 参照）を native 側で実演する。
fn run_menubar_demo() {
    let mut state = Menubar::new(0, 2, None, false, Orientation::Horizontal);
    println!("=== menubar state machine demo ===");
    println!("initial state: {state:?}");

    for (name, payload) in [
        ("toggle", "0"),
        // 開いている Menu を跨いだ左右移動（menubar.rs モジュール doc
        // 「本イシューの主題」参照）: focus 移動と同時に開く Menu も移る。
        ("next", ""),
        ("close", ""),
    ] {
        let applied = dispatch(&mut state, name, payload);
        println!("dispatch({name:?}, {payload:?}) -> applied={applied}, state={state:?}");
    }
}

/// `layout` + `list_page`（`start_router` 系統）・`render_for_hydration`
/// （`hydrate` 系統）・navigation-menu デモ・menubar デモを 1 ページに
/// 同居させた `page_shell` 出力を `dist/index.html` へ書き出す。
///
/// `hydrate`（`AppState` 系）と `start_router`（`layout()` が組む
/// `<div id="app-root">` 系）は**別系統・別 DOM**であり
/// （`fandhe-frontend-wasm-full::entry` の doc 参照）、この 4 つのマウント
/// ポイントを 1 ページに同居させる場合は互いに異なる `root_id` を使う契約に
/// 従う（`static/embed.html` は `hydrate("interactive-root")` /
/// `start_router("app-root")` / `hydrate_navigation_menu("nav-menu-root")` /
/// `hydrate_menubar("menubar-root")` を呼ぶ）。
fn write_ssr_html(
    state: &AppState,
    nav_menu_state: &NavigationMenu,
    menubar_state: &Menubar,
) -> Result<(), Box<dyn Error>> {
    let router_demo = layout("記事一覧 (start_router 系統)", list_page(&demo_items()));
    let hydrate_demo = render_for_hydration(state);
    let nav_menu_demo = nav_menu_view(nav_menu_state);
    let menubar_demo = menubar_view(menubar_state);
    let combined = el(
        "div",
        vec![],
        vec![router_demo, hydrate_demo, nav_menu_demo, menubar_demo],
    );
    let html = page_shell("状態管理 + View Transitions サンプル", combined);

    fs::create_dir_all("dist")?;
    fs::write("dist/index.html", html)?;
    println!("wrote dist/index.html");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    run_native_demo();
    run_navigation_menu_demo();
    run_menubar_demo();

    let state = AppState::new();
    // navigation-menu/menubar は「products が開いている」「File メニューが
    // 開いている」初期状態にする（SSR で最初から開閉双方の見た目を検分
    // でき、ブラウザ実演でも disclosure の閉じる操作を最初から試せる）。
    let nav_menu_state = {
        let mut s = NavigationMenu::default();
        dispatch(&mut s, "select", "products");
        s
    };
    let menubar_state = Menubar::new(0, 2, Some(0), false, Orientation::Horizontal);
    write_ssr_html(&state, &nav_menu_state, &menubar_state)?;

    Ok(())
}

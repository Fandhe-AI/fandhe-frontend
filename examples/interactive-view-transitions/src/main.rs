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
//! ブラウザでの実動作確認（wasm ビルド・`hydrate`/`start_router` の実演）は
//! `wasm/`（独立ワークスペースの glue クレート）+ `tools/wasm/build.sh` +
//! `static/embed.html` が担う（本バイナリの責務外。詳細は README.md）。
//!
//! # 実行内容（2 段構成）
//!
//! 1. **native 状態機械実演**: [`fandhe_frontend_interactive::AppState`] に対する
//!    [`fandhe_frontend_interactive::dispatch`] の呼び出し（`increment` ×2 /
//!    `set_draft` / `add_item` / 未知アクション）と、その都度の戻り値・状態を
//!    標準出力へ書き出す。
//! 2. **SSR HTML 書き出し**: `layout` + `list_page`（`start_router` 系統、
//!    `<div id="app-root">`）と `render_for_hydration`（`hydrate` 系統、
//!    `<div id="interactive-root">`）を 1 つの `div` に同居させた
//!    `page_shell` 出力を `dist/index.html` へ書き出す。**このファイルは
//!    検分用の SSR HTML（`<script>` を含まない）であり、ブラウザ実行は
//!    `static/embed.html` が担う**（責務分離、README.md 参照）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! HTML はすべて `fandhe_frontend_core` のノード木 API（`fandhe_frontend_app::layout` /
//! `list_page` / `page_shell` / `fandhe_frontend_interactive::render_for_hydration`）で
//! 組み立て、`format!` によるタグ文字列の直接組み立て・`raw_html()` は一切
//! 使わない。`dist/index.html` の書き出し先は固定相対パスであり、ユーザー
//! 入力を経路へ使わない（パストラバーサルの余地なし）。

#![forbid(unsafe_code)]

use fandhe_frontend_app::{demo_items, layout, list_page, page_shell};
use fandhe_frontend_core::{el, render};
use fandhe_frontend_interactive::{dispatch, render_for_hydration, AppState, Component};
use std::error::Error;
use std::fs;

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

/// `layout` + `list_page`（`start_router` 系統）と `render_for_hydration`
/// （`hydrate` 系統）を 1 つの `div` に同居させた `page_shell` 出力を
/// `dist/index.html` へ書き出す。
///
/// `hydrate`（`AppState` 系）と `start_router`（`layout()` が組む
/// `<div id="app-root">` 系）は**別系統・別 DOM**であり
/// （`fandhe-frontend-wasm-full::entry` の doc 参照）、この 2 つの div を
/// 1 ページに同居させる場合は互いに異なる `root_id` を使う契約に従う
/// （`static/embed.html` は `hydrate("interactive-root")` +
/// `start_router("app-root")` を呼ぶ）。
fn write_ssr_html(state: &AppState) -> Result<(), Box<dyn Error>> {
    let router_demo = layout("記事一覧 (start_router 系統)", list_page(&demo_items()));
    let hydrate_demo = render_for_hydration(state);
    let combined = el("div", vec![], vec![router_demo, hydrate_demo]);
    let html = page_shell("状態管理 + View Transitions サンプル", combined);

    fs::create_dir_all("dist")?;
    fs::write("dist/index.html", html)?;
    println!("wrote dist/index.html");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    run_native_demo();

    let state = AppState::new();
    write_ssr_html(&state)?;

    Ok(())
}

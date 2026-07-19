//! fandhe-frontend フレームワークの拡張プロジェクトテンプレート（`fw new --template app`、
//! イシュー #378）。
//!
//! # 役割・契約
//!
//! `templates/default`（fandhe-frontend-core 非依存の最小骨格、TASK-4.4 の負例検出
//! テスト土台）に対し、本テンプレートは fandhe-frontend-core / fandhe-frontend-app（vendor 同梱、
//! `vendor/fandhe-frontend-core` / `vendor/fandhe-frontend-app`）へ依存し、フレームワークの実 API
//! （`Loader` trait 実装・束縛点 API・`fandhe_frontend_core::render`）を使う出発点を
//! 提供する。AI エージェントが SSR/SSG 実体を自作して構成ドリフトするのを
//! 防ぐことが目的（イシュー #378 背景）。
//!
//! `main()` は `fandhe_frontend_app::Loader` の参照実装（`DemoItemsLoader` /
//! `DemoItemDetailLoader`）でデータを解決し、`fandhe_frontend_app::{list_page,
//! detail_page, page_shell}` で描画したノード木を `fandhe_frontend_core::render`
//! （既定エスケープ済み HTML 文字列）へ変換して `dist/` へ書き出す
//! （SSG 的最小 SSR）。`raw_html()` は使用しない
//! （`clippy.toml` の `disallowed-methods` が検出する）。
//! `demo_counter_fragment` は `fandhe_frontend_core::bind_text` / `fandhe_frontend_core::keyed_list`
//! （束縛点 API）の使用サンプルであり、`dist/demo.html` へ書き出す。
//! いずれも HTML 文字列の直接組み立てはしない（ノード木 API のみ）。

#![forbid(unsafe_code)]

use fandhe_frontend_app::{DemoItemDetailLoader, DemoItemsLoader, Item, Loader};
use fandhe_frontend_core::keyed::keyed_list;
use fandhe_frontend_core::{bind_text, div, el, li, render, text, Node};
use std::fs;
use std::path::Path;

fn main() {
    let dist_dir = Path::new("dist");
    if let Err(e) = fs::create_dir_all(dist_dir) {
        eprintln!("fw-template-app: failed to create `dist/`: {e}");
        std::process::exit(1);
    }

    // 一覧画面: DemoItemsLoader（Loader trait 実装）→ list_page → render。
    let items_loader = DemoItemsLoader;
    let items = items_loader
        .load(&())
        .expect("DemoItemsLoader::load never fails (Error = Infallible)");
    let list_html = fandhe_frontend_app::page_shell("記事一覧", fandhe_frontend_app::list_page(&items));
    write_page(dist_dir, "index.html", &list_html);

    // 詳細画面: 項目ごとに DemoItemDetailLoader → detail_page → render。
    let detail_loader = DemoItemDetailLoader;
    for item in &items {
        let detail = detail_loader
            .load(&item.id)
            .expect("DemoItemDetailLoader::load never fails (Error = Infallible)");
        let detail_html = fandhe_frontend_app::page_shell("記事詳細", fandhe_frontend_app::detail_page(detail.as_ref()));
        write_page(dist_dir, &format!("items-{}.html", item.id), &detail_html);
    }

    // 束縛点 API（bind_text / keyed_list）の使用サンプル。
    let demo_html = render(&demo_counter_fragment(&items));
    write_page(dist_dir, "demo.html", &demo_html);

    println!("wrote {} pages to dist/", items.len() + 2);
}

/// `fandhe_frontend_core::bind_text` / `fandhe_frontend_core::keyed_list`（束縛点 API）の使用サンプル。
///
/// `bind_text` は `data-bind-text` マーカーを、`keyed_list` は
/// `data-bind-list`/`data-key` マーカーを付与した [`Node`] を返す
/// （`fandhe-frontend-wasm-client`/`fandhe-frontend-wasm-full` がハイドレーション時に走査する契約、
/// `core/src/lib.rs` 冒頭 rustdoc 参照）。本テンプレートはハイドレーション
/// 自体（wasm ビルド）を行わないため、SSR 出力にマーカー属性を含めるだけの
/// 最小サンプルとする（`static/embed.html` が CSR マウント骨格を示す）。
fn demo_counter_fragment(items: &[Item]) -> Node {
    let counter = bind_text(
        "span",
        vec![("data-testid", "item-count")],
        "item_count",
        items.len().to_string(),
    );

    let keyed_items: Vec<(String, Node)> = items
        .iter()
        .map(|it| (it.id.clone(), li(vec![], vec![text(it.title.clone())])))
        .collect();
    let list = keyed_list(
        "ul",
        vec![("data-testid", "keyed-items")],
        "items",
        keyed_items,
    )
    .expect("demo データの key は id 由来で非空・一意のため常に成功する");

    div(
        vec![("data-testid", "demo-fragment")],
        vec![el("p", vec![], vec![text("件数: "), counter]), list],
    )
}

/// レンダリング済み HTML 文字列を `dist_dir/filename` へ書き出す。
fn write_page(dist_dir: &Path, filename: &str, html: &str) {
    let path = dist_dir.join(filename);
    if let Err(e) = fs::write(&path, html) {
        eprintln!("fw-template-app: failed to write `{}`: {e}", path.display());
        std::process::exit(1);
    }
}

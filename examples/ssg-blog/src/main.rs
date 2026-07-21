//! `fandhe-frontend-example-ssg-blog`: SSG（`generate_pages`）の正本サンプル
//! （イシュー #501、examples 規約に従う 2 件目のサンプル）。
//!
//! # 役割・呼び出し文脈
//!
//! `examples/ssr-routing`（イシュー #499、SSR + ルーティング）と対を成し、
//! `fandhe_frontend_server::ssg::generate_pages`（イシュー #463 で追加された
//! 汎用 SSG API）を実際に使う静的ブログサイトの参照実装として存在する。
//! `generate_pages` は固定ルート表（`/` と `/items/{id}`）に限定される
//! `generate`/`generate_with` とは異なり、任意の (リクエストパス, `Node`) 列を
//! 受け取って `out_dir` 配下へ書き出す。本サンプルは記事一覧 + 各記事詳細の
//! (パス, `Node`) 列を組み立てて渡す実演を行う。
//!
//! `generate_pages` のシグネチャは `Node` を要求するため、
//! `fandhe_frontend_app::page_shell`（`String` を返す）は使えず、本サンプルは
//! 自作の [`layout`]（`Node` を返す）でページ骨格を組み立てる。
//!
//! # 学べること
//!
//! - `fandhe_frontend_server::ssg::generate_pages` によるページ列の静的書き出し
//! - `generate_pages` の出力パス検証が fail-closed であること（不正なパスが
//!   1 件でもあれば何も書き出さない）と、正規化後の重複パスを拒否すること
//!   （`tests/ssg_output.rs` で回帰を固定）
//! - 既定エスケープ（REQ-1）: 記事タイトル・本文はすべて `text()` 経由で
//!   ノード木へ載せ、`raw_html()` や `format!` によるタグ文字列の直接組み立て
//!   は使わない
//! - `@view-transition { navigation: auto; }`（`fandhe_frontend_app::page_shell`
//!   と同一の固定リテラル）による Cross-Document View Transitions の有効化
//!
//! # セキュリティ不変条件（REQ-1・OWASP A01）
//!
//! - HTML はすべて `fandhe_frontend_core` のノード木 API（`el` / `text` /
//!   `crates/core/src/tags.rs` のタグヘルパー）で組み立てる。`format!` は
//!   属性値・リンク先パスのプレーン文字列整形にのみ使い、タグ文字列の
//!   直接組み立てには使わない（`coding-rust.md`「HTML 文字列の直接組み立て
//!   禁止」の対象外）。
//! - 出力パスの安全性（`..`・`/`・`\` の拒否、正規化後の重複拒否）は
//!   `generate_pages` 内の検証に委ね、本サンプル側で独自のパス組み立て・
//!   検証迂回を行わない。記事の `slug` は静的リテラルのみを使う
//!   （`src/posts.rs` 参照）。

#![forbid(unsafe_code)]

mod posts;

use fandhe_frontend_core::{a, article, el, h1, header, main_tag, p, text, Node};
use fandhe_frontend_server::ssg::generate_pages;
use posts::{all_posts, Post};
use std::path::Path;

/// 各ページ共通の骨格（`<html>` 全体）を組み立てる。
///
/// `fandhe_frontend_app::page_shell` と同じ `@view-transition` 固定リテラルを
/// `text()` 経由で出力し、既定エスケープ経路を迂回しない
/// （`crates/app/src/lib.rs::page_shell` のコメント参照）。`page_shell` は
/// `String` を返すため `generate_pages`（`Node` 列を要求）には使えず、本関数
/// は `Node` を返す自作版として存在する。
fn layout(title: &str, main: Node) -> Node {
    let head = el(
        "head",
        vec![],
        vec![
            el("meta", vec![("charset", "utf-8")], vec![]),
            el(
                "meta",
                vec![
                    ("name", "viewport"),
                    ("content", "width=device-width, initial-scale=1"),
                ],
                vec![],
            ),
            el(
                "style",
                vec![],
                vec![text("@view-transition { navigation: auto; }")],
            ),
            el("title", vec![], vec![text(title)]),
        ],
    );
    let document_body = el(
        "body",
        vec![],
        vec![
            header(vec![], vec![a(vec![("href", "/")], vec![text("SSG Blog")])]),
            main,
        ],
    );
    el("html", vec![("lang", "ja")], vec![head, document_body])
}

/// 記事一覧（`/`）を組み立てる。各記事へのリンク先 `/posts/<slug>/` は
/// `format!` で組み立てるが、これはタグ文字列ではなく属性値のプレーン
/// 文字列整形であり、`slug` は静的リテラル（`posts.rs` 参照）に限られる。
fn index_page(all: &[Post]) -> Node {
    let items: Vec<Node> = all
        .iter()
        .map(|post| {
            p(
                vec![],
                vec![a(
                    vec![("href", format!("/posts/{}/", post.slug).as_str())],
                    vec![text(post.title)],
                )],
            )
        })
        .collect();
    main_tag(
        vec![],
        vec![h1(vec![], vec![text("Posts")])]
            .into_iter()
            .chain(items)
            .collect(),
    )
}

/// 記事詳細ページを組み立てる。タイトル（`h1`）・各段落（`p`）はいずれも
/// `text()` 経由で出力し、`<script>` 等を含むタイトルが既定エスケープされる
/// ことを `tests/ssg_output.rs` で回帰固定する。
fn post_page(post: &Post) -> Node {
    let mut children = vec![h1(vec![], vec![text(post.title)])];
    children.extend(
        post.paragraphs
            .iter()
            .map(|paragraph| p(vec![], vec![text(*paragraph)])),
    );
    main_tag(vec![], vec![article(vec![], children)])
}

/// `generate_pages` に渡す (リクエストパス, `Node`) 列を組み立てる。
///
/// `/` は記事一覧、`/posts/<slug>/` は各記事詳細。パスの安全性検証・重複
/// 検出は `generate_pages` 側の責務であり、本関数は候補列を組み立てるのみ。
fn build_pages() -> Vec<(String, Node)> {
    let all = all_posts();
    let mut pages = vec![("/".to_string(), layout("Posts", index_page(&all)))];
    for post in &all {
        pages.push((
            format!("/posts/{}/", post.slug),
            layout(post.title, post_page(post)),
        ));
    }
    pages
}

/// CLI エントリポイント。`build_pages()` の結果を `dist/` へ `generate_pages`
/// で書き出す。成功時は書き出したファイルパスを 1 行ずつ標準出力へ、失敗時は
/// エラーを標準エラーへ出力して非ゼロ終了する（`unwrap`/`panic!` は使わない、
/// `coding-rust.md` のエラー処理規約）。
fn main() {
    match generate_pages(&build_pages(), Path::new("dist")) {
        Ok(written) => {
            for path in written {
                println!("{}", path.display());
            }
        }
        Err(err) => {
            eprintln!("failed to generate static site: {err}");
            std::process::exit(1);
        }
    }
}

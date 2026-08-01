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
//! `generate_pages` に加え、`fandhe_frontend_server::ssg::generate_assets`
//! （イシュー #1119 で追加された汎用アセット書き出し API）による
//! `sitemap.xml` / `robots.txt` の書き出しも実演する（イシュー #1135）。
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
//! - `fandhe_frontend_server::ssg::generate_assets` による非 HTML アセット
//!   （`sitemap.xml` / `robots.txt`）の書き出し。`generate_pages` と同じ
//!   fail-closed のパス検証を経由するが、コンテンツは無加工書き出しのため
//!   既定エスケープは適用されない（[`build_assets`] 参照）
//!
//! # セキュリティ不変条件（REQ-1・OWASP A01）
//!
//! - HTML はすべて `fandhe_frontend_core` のノード木 API（`el` / `text` /
//!   `crates/core/src/tags.rs` のタグヘルパー）で組み立てる。`format!` は
//!   属性値・リンク先パスのプレーン文字列整形にのみ使い、タグ文字列の
//!   直接組み立てには使わない（`coding-rust.md`「HTML 文字列の直接組み立て
//!   禁止」の対象外）。
//! - 出力パスの安全性（`..`・`/`・`\` の拒否、正規化後の重複拒否）は
//!   `generate_pages`/`generate_assets` 内の検証に委ね、本サンプル側で独自の
//!   パス組み立て・検証迂回を行わない。記事の `slug` は静的リテラルのみを使う
//!   （`src/posts.rs` 参照）。
//! - `generate_assets` は無加工書き出し（既定エスケープ非適用）の API であり
//!   HTML ページの生成には使わない。本サンプルでは `sitemap.xml`（XML）と
//!   `robots.txt`（プレーンテキスト）にのみ使い、埋め込む値は静的リテラルの
//!   `slug`（英数字・`-`・`_` のみ、`src/posts.rs` 参照）と RFC 2606 予約
//!   ドメインの [`BASE_URL`] に限られるため XML 特殊文字を含まない
//!   （`build_assets` 参照）。

#![forbid(unsafe_code)]

mod posts;

use fandhe_frontend_core::{a, article, el, h1, header, main_tag, p, text, Node};
use fandhe_frontend_server::ssg::{generate_assets, generate_pages};
use posts::{all_posts, Post};
use std::path::Path;

/// サイトのベース URL。RFC 2606 で予約された `example.com` を使う
/// （実サイトへ組み込む際は利用者が実ドメインへ差し替える）。
/// `sitemap.xml` の `<loc>` と `robots.txt` の `Sitemap:` 行の両方から
/// 参照する単一の情報源とする。
const BASE_URL: &str = "https://example.com";

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
/// `all`（`all_posts()` の結果）は [`build_assets`] と共有する呼び出し元
/// （`main()`）から受け取る（`sitemap.xml` の `<loc>` 列と記事一覧の `slug`
/// を単一の情報源から揃えるため）。
fn build_pages_for(all: &[Post]) -> Vec<(String, Node)> {
    let mut pages = vec![("/".to_string(), layout("Posts", index_page(all)))];
    for post in all {
        pages.push((
            format!("/posts/{}/", post.slug),
            layout(post.title, post_page(post)),
        ));
    }
    pages
}

/// `generate_assets` に渡す (リクエストパス, コンテンツ文字列) 列を組み立てる
/// （イシュー #1135）。
///
/// `sitemap.xml`（XML）は `/` と各記事 `/posts/<slug>/` の `<loc>` を列挙し、
/// `robots.txt`（プレーンテキスト）はクロール許可と sitemap の所在を示す。
///
/// `generate_assets` は無加工書き出しのため既定エスケープ（REQ-1）は
/// 適用されない（`fandhe_frontend_core::render` を経由しない）。本関数が
/// 埋め込む値は静的リテラルの `slug`（`generate_pages` の出力パス検証と同じ
/// ホワイトリスト＝英数字・`-`・`_` のみ、`src/posts.rs` 参照）と
/// [`BASE_URL`] に限られ、いずれも XML 特殊文字（`<` `>` `&` 等）を含まない
/// ため、本サンプルではエスケープ不要で無加工書き出しが安全に成立する。
/// ユーザー入力由来の値を渡す場合は呼び出し側で XML エスケープを行うこと
/// （`generate_assets` の rustdoc「契約」節参照）。
///
/// `format!` によるこの XML/テキスト組み立ては `coding-rust.md`「HTML
/// 文字列の直接組み立て禁止」の対象外（HTML ではない）。
fn build_assets(all: &[Post]) -> Vec<(String, String)> {
    let mut sitemap = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    sitemap.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
    sitemap.push_str(&format!("  <url><loc>{BASE_URL}/</loc></url>\n"));
    for post in all {
        sitemap.push_str(&format!(
            "  <url><loc>{BASE_URL}/posts/{}/</loc></url>\n",
            post.slug
        ));
    }
    sitemap.push_str("</urlset>\n");

    let robots = format!("User-agent: *\nAllow: /\nSitemap: {BASE_URL}/sitemap.xml\n");

    vec![
        ("/sitemap.xml".to_string(), sitemap),
        ("/robots.txt".to_string(), robots),
    ]
}

/// CLI エントリポイント。`build_pages_for()` の結果を `dist/` へ
/// `generate_pages` で書き出した後、`build_assets()` の結果を
/// `generate_assets` で書き出す。
/// 成功時は書き出したファイルパスを 1 行ずつ標準出力へ、失敗時はエラーを
/// 標準エラーへ出力して非ゼロ終了する（`unwrap`/`panic!` は使わない、
/// `coding-rust.md` のエラー処理規約）。
///
/// `generate_pages`（HTML ページ）と `generate_assets`（`sitemap.xml` /
/// `robots.txt`）を同一 `out_dir`（`dist/`）へ併用しているが、出力パスは
/// `index.html` 群と衝突しないため呼び出し間の重複検出対象外という
/// caveat（`generate_assets` rustdoc 参照）の影響を受けない。
fn main() {
    let all = all_posts();

    if let Err(err) = generate_pages(&build_pages_for(&all), Path::new("dist")).and_then(|pages| {
        generate_assets(&build_assets(&all), Path::new("dist")).map(|assets| {
            for path in pages.into_iter().chain(assets) {
                println!("{}", path.display());
            }
        })
    }) {
        eprintln!("failed to generate static site: {err}");
        std::process::exit(1);
    }
}

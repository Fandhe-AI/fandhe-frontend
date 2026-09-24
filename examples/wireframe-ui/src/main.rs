//! `fandhe-frontend-example-wireframe-ui`: `fandhe-frontend-wireframe-ui`
//! （blocks.pm 参照のローファイ・モノクロワイヤーフレーム UI コンポーネント
//! 層、イシュー #2599/#2600/#2603）のショーケース正本サンプル（イシュー
//! #2667。examples 規約〔イシュー #499〕に従う 6 件目のサンプル）。
//!
//! # 役割・呼び出し文脈
//!
//! `fandhe-frontend-wireframe-ui` v0.52.0（イシュー #2668 で crates.io へ
//! 初回公開済み）が提供する全 49 部品（blocks.pm 由来 35 + 追加 14）を、
//! `docs/design/wireframe-ui-architecture.md` の Phase 1〜8 区分どおりに
//! 1 ページへ並べて実演する。`crates/docs-site` の `/wireframes/<kebab>/`
//! 部品ページ群（部品 1 件 = 1 ページ）とは異なり、本サンプルは全部品を
//! 1 ページに集約した SSR 正本サンプルという位置づけである。
//!
//! `fandhe-frontend-wireframe-ui` は SSR 専用・非インタラクティブな表示
//!専用部品層（`crates/wireframe-ui/src/lib.rs` rustdoc 参照）であり、
//! wasm 配線・状態遷移・対話操作を一切持たない。そのため本サンプルは
//! `examples/ssg-blog`（`fandhe_frontend_server::ssg` 依存なし・
//! `generate_pages` も使わない）よりさらに単純な「1 ページを組み立てて
//! `dist/index.html` へ書き出すだけ」の構成であり、
//! `fandhe_frontend_server` への依存も持たない（`Cargo.toml` 参照）。
//!
//! 各部品の実演は [`sections`] モジュール（Phase 別に分割）が担い、本
//! ファイルはページ骨格（`layout`）と `dist/` への書き出しのみを担当する。
//!
//! # CSS の出力方式（`<style>` インライン埋め込みからの意図的な逸脱）
//!
//! `fandhe_frontend_wireframe_ui::wireframe_css()` が返す CSS 文字列は
//! 子孫結合子でない子結合子（`>`）を含むセレクタを持つ（例: `stepper` /
//! `breadcrumbs` の区切り疑似要素セレクタ）。`fandhe_frontend_core::render`
//! は `<style>` を他のタグと同様に扱い、`Node::Text` の子は常に
//! `escape_html_into` を経由する（`crates/core/src/lib.rs::render_into`
//! 参照、`<style>`/`<script>` を raw-text element として特別扱いする
//! 分岐は存在しない）。そのため `el("style", vec![], vec![text(css)])` で
//! CSS をインライン埋め込みすると `>` が `&gt;` に実体参照化され、`>` を
//! 含むセレクタが壊れる。既定エスケープ（REQ-1）を弱める新たな迂回経路
//! （`raw_html()` 以外の経路）は作らない方針（`coding-rust.md`）のため、
//! CSS は `examples/headless-pre-styled-ui` と同じ「別ファイルへ書き出し、
//! `<link rel="stylesheet">` で参照する」方式を採る（`build_page`/`main`
//! 参照）。
//!
//! # 学べること
//!
//! - `fandhe_frontend_wireframe_ui` の Phase 1〜8・全 49 部品の呼び出し方
//!   （[`sections`] モジュール）
//! - 既定エスケープ（REQ-1）: 部品へ渡すテキスト引数は
//!   `fandhe_frontend_wireframe_ui` 内部で `fandhe_frontend_core::text()`
//!   経由へ渡され既定エスケープされる。本サンプルは `<script>` を含む
//!   注釈タイトルを渡し、生の `<script>` タグとしては出力されないことを
//!   [`sections::build_sections`] の XSS 実演節と `tests/cli_output.rs` で
//!   確認できる
//! - `wireframe_css()` の CSS 出力を静的アセットとして書き出す SSR の
//!   最小構成（上記「CSS の出力方式」節参照）
//!
//! # セキュリティ不変条件（REQ-1・OWASP A01）
//!
//! - HTML はすべて `fandhe_frontend_core` のノード木 API と
//!   `fandhe_frontend_wireframe_ui` の部品関数で組み立てる。`format!` は
//!   タグ文字列の直接組み立てには使わない
//! - `raw_html()` は一切使用しない
//! - 出力先パスは `dist/index.html`・`dist/assets/wireframe.css` の固定
//!   リテラルのみで、外部入力由来のパスは扱わない

#![forbid(unsafe_code)]

mod sections;

use fandhe_frontend_core::{el, header, main_tag, render, text, Node};
use std::path::Path;

/// ページ骨格（`<html>` 全体）を組み立てる。
///
/// `fandhe_frontend_app::page_shell`（`String` を返す）は使わず、
/// `examples/ssg-blog`/`examples/headless-pre-styled-ui` と同様に `Node` を
/// 返す自作の骨格を使う（本サンプルは `fandhe-frontend-app` に依存しない
/// ため、そもそも `page_shell` を呼べない）。
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
                "link",
                vec![("rel", "stylesheet"), ("href", "assets/wireframe.css")],
                vec![],
            ),
            el("title", vec![], vec![text(title)]),
        ],
    );
    let document_body = el(
        "body",
        vec![],
        vec![
            header(
                vec![],
                vec![fandhe_frontend_core::h1(
                    vec![],
                    vec![text("fandhe-frontend-wireframe-ui ショーケース")],
                )],
            ),
            main,
        ],
    );
    el("html", vec![("lang", "ja")], vec![head, document_body])
}

/// ページ全体を組み立てる。
fn build_page() -> Node {
    layout(
        "wireframe-ui ショーケース",
        main_tag(vec![], sections::build_sections()),
    )
}

/// `dist/` へ書き出す。出力先は固定リテラルのみ（外部入力由来のパスは
/// 扱わない）。
fn main() {
    let page = build_page();
    // `<!DOCTYPE html>` はユーザー入力を一切含まない固定リテラルとして
    // `render()` 済みの既定エスケープ済み HTML の前に結合するのみであり、
    // 新たなエスケープ迂回経路ではない（`examples/ssg-blog` の
    // `main` と同じ方針）。
    let html = format!("<!DOCTYPE html>\n{}", render(&page));

    let dist = Path::new("dist");
    let assets = dist.join("assets");
    if let Err(err) = std::fs::create_dir_all(&assets) {
        eprintln!("failed to create dist/assets: {err}");
        std::process::exit(1);
    }
    if let Err(err) = std::fs::write(dist.join("index.html"), html) {
        eprintln!("failed to write dist/index.html: {err}");
        std::process::exit(1);
    }
    if let Err(err) = std::fs::write(
        assets.join("wireframe.css"),
        fandhe_frontend_wireframe_ui::wireframe_css(),
    ) {
        eprintln!("failed to write dist/assets/wireframe.css: {err}");
        std::process::exit(1);
    }

    println!("dist/index.html");
    println!("dist/assets/wireframe.css");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 既定エスケープ回帰（REQ-1）: `<script>` を含む注釈タイトルが実体
    /// 参照化されて出力され、生の `<script>` タグとしては現れないこと。
    #[test]
    fn build_page_output_escapes_xss_probe_payload() {
        let html = render(&build_page());
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"));
    }

    /// ページ全体が `wireframe_css()` を直接埋め込まず、`<link>` 経由で
    /// 参照することを固定する（上記「CSS の出力方式」節の設計判断の
    /// 回帰）。CSS ルール本体（部品の class 属性値ではなく、CSS の
    /// プロパティ宣言そのもの）がページ本文へ現れないことを確認する
    /// （class 属性値の `fw-wire-*` 自体は各部品が通常どおり出力するため、
    /// class 名の不在ではなく CSS 宣言文の不在を確認する）。
    #[test]
    fn build_page_links_external_stylesheet_instead_of_inlining() {
        let html = render(&build_page());
        assert!(html.contains(r#"<link rel="stylesheet" href="assets/wireframe.css">"#));
        assert!(!html.contains("display: inline-block"));
        assert!(!html.contains('{'));
    }

    /// Phase 1〜8 の全見出しと XSS 実演節がページに含まれることを固定する
    /// （section 追加漏れの回帰）。
    #[test]
    fn build_page_contains_all_phase_headings() {
        let html = render(&build_page());
        for heading in [
            "Phase 1: レイアウト骨格",
            "Phase 2: テキスト・注釈",
            "Phase 3: Forms A",
            "Phase 4: Forms B",
            "Phase 5: Navigation",
            "Phase 6: Overlay・Feedback",
            "Phase 7: Data display",
            "Phase 8: Media・データ表示",
            "既定エスケープの実演（REQ-1）",
        ] {
            assert!(html.contains(heading), "missing heading: {heading}");
        }
    }
}

//! 標準タグショートカット群（TASK-5.1b + Issue #164 拡張分）。
//!
//! `rws-app`・`rws-server` 等の上位クレートやフレームワーク利用者コードが
//! ノード木を組み立てる際に、素の [`crate::el`] 呼び出しより読みやすい記述を
//! 提供するための薄いヘルパー関数群。**すべて [`crate::el`] への委譲のみ**で
//! あり、独自の出力経路・独自のエスケープ処理を一切持たない
//! （`core/src/lib.rs` 冒頭の不変条件 1・2 がそのまま適用される。
//! `docs/api/component-api.md` 第 4 節・定義規則 1〜3 を参照）。
//!
//! ## 定義規則（`docs/api/component-api.md` 第 4 節を踏襲）
//!
//! 1. シグネチャは `fn <name>(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node`
//!    に統一し、本体は `el("<tag>", attrs, children)` の一行のみとする。
//! 2. タグ名は関数内の `&'static str` リテラルとして固定する
//!    （`is_valid_tag_name` によるホワイトリスト検証は `el`/`render` 側の
//!    既存責務のまま変更しない）。
//! 3. Rust の予約語・標準型と衝突・混同する名前のみ `_tag` サフィックスを付ける
//!    （`main_tag` が先例。本モジュールの拡張セットに新規の衝突はない）。
//!
//! ## 選定基準・スコープ外の判断
//!
//! - 追加するタグは HTML Living Standard の一般的な構造・テキスト・フォーム・
//!   テーブル要素から選定する。
//! - **`script`/`style`/`iframe` のヘルパーは意図的に提供しない**。これらは
//!   埋め込み文字列・外部リソースを扱う攻撃面が大きいタグであり、標準ヘルパー
//!   として書きやすくすることは XSS・サプライチェーン面のリスクを増やす
//!   （`.claude/rules/security.md`）。必要な場合は利用者コードが明示的に
//!   `el("script", ...)` 等を呼ぶ（意図が書いた本人に見える形を保つ）。
//! - `select`/`option` はそれぞれ Rust の `Option` 型との混同を避けるため
//!   今回のセットから除外した（Issue #164 実装時に「不採用」として記録。
//!   将来必要になった場合は `option_tag`/`select_tag` 等の命名で再検討する）。
//! - 属性なしヘルパー・attrs ビルダ API（例: `div_()`）は導入しない。属性の
//!   タプル `(&str, &str)` は既に素の Rust であり、追加の抽象化は API 表面を
//!   広げるだけで可読性への寄与が薄いと判断した（同じく不採用として記録）。
//!
//! ## void 要素の既知の制約
//!
//! `img`/`br`/`hr`/`input` は HTML では void 要素（終了タグを持たない）だが、
//! `render`/`render_into`（`core/src/lib.rs`）は v1 では常に終了タグを出力する
//! 現行仕様を凍結している（`docs/api/component-api.md` 第 3 節・判断 4）。本モジュール
//! のヘルパーもこの挙動をそのまま継承し、`<br></br>` のような出力になる。
//! 自己終端出力への最適化は本モジュールのスコープ外とする。

use crate::{el, Node};

/// `<div>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲であり、
/// エスケープ・タグ名検証は [`el`]/[`crate::render`] の既存経路をそのまま利用する
/// （`docs/api/component-api.md` 第 4 節・定義規則）。
///
/// # Examples
///
/// ```
/// use rws_core::{div, text, render};
///
/// let node = div(vec![("class", "card")], vec![text("hello")]);
/// assert_eq!(render(&node), r#"<div class="card">hello</div>"#);
/// ```
pub fn div(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("div", attrs, children)
}

/// `<p>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{p, text, render};
///
/// let node = p(vec![], vec![text("hello")]);
/// assert_eq!(render(&node), "<p>hello</p>");
/// ```
pub fn p(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("p", attrs, children)
}

/// `<ul>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{ul, li, text, render};
///
/// let node = ul(vec![], vec![li(vec![], vec![text("item")])]);
/// assert_eq!(render(&node), "<ul><li>item</li></ul>");
/// ```
pub fn ul(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("ul", attrs, children)
}

/// `<li>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{li, text, render};
///
/// let node = li(vec![], vec![text("item")]);
/// assert_eq!(render(&node), "<li>item</li>");
/// ```
pub fn li(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("li", attrs, children)
}

/// `<a>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{a, text, render};
///
/// let node = a(vec![("href", "/about")], vec![text("about")]);
/// assert_eq!(render(&node), r#"<a href="/about">about</a>"#);
/// ```
pub fn a(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("a", attrs, children)
}

/// `<h1>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{h1, text, render};
///
/// let node = h1(vec![], vec![text("title")]);
/// assert_eq!(render(&node), "<h1>title</h1>");
/// ```
pub fn h1(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("h1", attrs, children)
}

/// `<main>` 要素を組み立てる標準タグショートカット。`main` は Rust の予約語では
/// ないが、可読性のため PoC-3 の命名（`main_tag`）をそのまま踏襲する
/// （`docs/api/component-api.md` 第 4 節・定義規則 4）。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{main_tag, text, render};
///
/// let node = main_tag(vec![], vec![text("content")]);
/// assert_eq!(render(&node), "<main>content</main>");
/// ```
pub fn main_tag(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("main", attrs, children)
}

/// `<span>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{span, text, render};
///
/// let node = span(vec![], vec![text("inline")]);
/// assert_eq!(render(&node), "<span>inline</span>");
/// ```
pub fn span(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("span", attrs, children)
}

/// `<section>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn section(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("section", attrs, children)
}

/// `<header>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn header(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("header", attrs, children)
}

/// `<footer>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn footer(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("footer", attrs, children)
}

/// `<nav>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn nav(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("nav", attrs, children)
}

/// `<article>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn article(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("article", attrs, children)
}

/// `<aside>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn aside(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("aside", attrs, children)
}

/// `<h2>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{h2, text, render};
///
/// let node = h2(vec![], vec![text("section title")]);
/// assert_eq!(render(&node), "<h2>section title</h2>");
/// ```
pub fn h2(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("h2", attrs, children)
}

/// `<h3>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn h3(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("h3", attrs, children)
}

/// `<h4>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn h4(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("h4", attrs, children)
}

/// `<h5>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn h5(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("h5", attrs, children)
}

/// `<h6>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn h6(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("h6", attrs, children)
}

/// `<ol>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn ol(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("ol", attrs, children)
}

/// `<strong>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{strong, text, render};
///
/// let node = strong(vec![], vec![text("important")]);
/// assert_eq!(render(&node), "<strong>important</strong>");
/// ```
pub fn strong(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("strong", attrs, children)
}

/// `<em>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn em(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("em", attrs, children)
}

/// `<small>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn small(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("small", attrs, children)
}

/// `<blockquote>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn blockquote(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("blockquote", attrs, children)
}

/// `<pre>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn pre(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("pre", attrs, children)
}

/// `<code>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn code(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("code", attrs, children)
}

/// `<form>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{form, button, text, render};
///
/// let node = form(vec![("method", "post")], vec![button(vec![], vec![text("送信")])]);
/// assert_eq!(render(&node), r#"<form method="post"><button>送信</button></form>"#);
/// ```
pub fn form(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("form", attrs, children)
}

/// `<label>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn label(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("label", attrs, children)
}

/// `<input>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// `input` は HTML では void 要素だが、本クレートの `render` は v1 では常に
/// 終了タグを出力する現行仕様を凍結している（本モジュール冒頭の rustdoc・
/// `docs/api/component-api.md` 第 3 節・判断 4 を参照）。
pub fn input(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("input", attrs, children)
}

/// `<button>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn button(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("button", attrs, children)
}

/// `<textarea>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn textarea(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("textarea", attrs, children)
}

/// `<table>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// # Examples
///
/// ```
/// use rws_core::{table, tr, td, text, render};
///
/// let node = table(vec![], vec![tr(vec![], vec![td(vec![], vec![text("1")])])]);
/// assert_eq!(render(&node), "<table><tr><td>1</td></tr></table>");
/// ```
pub fn table(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("table", attrs, children)
}

/// `<thead>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn thead(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("thead", attrs, children)
}

/// `<tbody>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn tbody(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("tbody", attrs, children)
}

/// `<tr>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn tr(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("tr", attrs, children)
}

/// `<th>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn th(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("th", attrs, children)
}

/// `<td>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn td(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("td", attrs, children)
}

/// `<caption>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
pub fn caption(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("caption", attrs, children)
}

/// `<img>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// `img` は HTML では void 要素だが、本クレートの `render` は v1 では常に
/// 終了タグを出力する現行仕様を凍結している（本モジュール冒頭の rustdoc・
/// `docs/api/component-api.md` 第 3 節・判断 4 を参照）。
///
/// # Examples
///
/// ```
/// use rws_core::{img, render};
///
/// let node = img(vec![("src", "/logo.png"), ("alt", "logo")], vec![]);
/// assert_eq!(render(&node), r#"<img src="/logo.png" alt="logo"></img>"#);
/// ```
pub fn img(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("img", attrs, children)
}

/// `<br>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// `br` は HTML では void 要素だが、本クレートの `render` は v1 では常に
/// 終了タグを出力する現行仕様を凍結している（本モジュール冒頭の rustdoc・
/// `docs/api/component-api.md` 第 3 節・判断 4 を参照）。
pub fn br(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("br", attrs, children)
}

/// `<hr>` 要素を組み立てる標準タグショートカット。[`el`] への薄い委譲。
///
/// `hr` は HTML では void 要素だが、本クレートの `render` は v1 では常に
/// 終了タグを出力する現行仕様を凍結している（本モジュール冒頭の rustdoc・
/// `docs/api/component-api.md` 第 3 節・判断 4 を参照）。
pub fn hr(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    el("hr", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{raw_html, render, text};

    /// TASK-5.1b で追加した最小セット（`docs/api/component-api.md` 第 4 節）が、
    /// それぞれ期待どおりのタグ名で出力されることを一括で固定する。
    /// `main_tag` のみ委譲先タグ名 `"main"` と関数名が異なる点に注意。
    #[test]
    fn tag_shortcuts_render_expected_tag_names() {
        assert_eq!(render(&div(vec![], vec![text("x")])), "<div>x</div>");
        assert_eq!(render(&p(vec![], vec![text("x")])), "<p>x</p>");
        assert_eq!(render(&ul(vec![], vec![text("x")])), "<ul>x</ul>");
        assert_eq!(render(&li(vec![], vec![text("x")])), "<li>x</li>");
        assert_eq!(render(&a(vec![], vec![text("x")])), "<a>x</a>");
        assert_eq!(render(&h1(vec![], vec![text("x")])), "<h1>x</h1>");
        assert_eq!(render(&main_tag(vec![], vec![text("x")])), "<main>x</main>");
    }

    /// Issue #164 で拡張した全ヘルパーが、それぞれ期待どおりのタグ名で
    /// 出力されることを一括で固定する（新規追加分の網羅的な出力固定テスト）。
    #[test]
    fn extended_tag_shortcuts_render_expected_tag_names() {
        let cases: Vec<(&str, Node)> = vec![
            ("span", span(vec![], vec![text("x")])),
            ("section", section(vec![], vec![text("x")])),
            ("header", header(vec![], vec![text("x")])),
            ("footer", footer(vec![], vec![text("x")])),
            ("nav", nav(vec![], vec![text("x")])),
            ("article", article(vec![], vec![text("x")])),
            ("aside", aside(vec![], vec![text("x")])),
            ("h2", h2(vec![], vec![text("x")])),
            ("h3", h3(vec![], vec![text("x")])),
            ("h4", h4(vec![], vec![text("x")])),
            ("h5", h5(vec![], vec![text("x")])),
            ("h6", h6(vec![], vec![text("x")])),
            ("ol", ol(vec![], vec![text("x")])),
            ("strong", strong(vec![], vec![text("x")])),
            ("em", em(vec![], vec![text("x")])),
            ("small", small(vec![], vec![text("x")])),
            ("blockquote", blockquote(vec![], vec![text("x")])),
            ("pre", pre(vec![], vec![text("x")])),
            ("code", code(vec![], vec![text("x")])),
            ("form", form(vec![], vec![text("x")])),
            ("label", label(vec![], vec![text("x")])),
            ("input", input(vec![], vec![text("x")])),
            ("button", button(vec![], vec![text("x")])),
            ("textarea", textarea(vec![], vec![text("x")])),
            ("table", table(vec![], vec![text("x")])),
            ("thead", thead(vec![], vec![text("x")])),
            ("tbody", tbody(vec![], vec![text("x")])),
            ("tr", tr(vec![], vec![text("x")])),
            ("th", th(vec![], vec![text("x")])),
            ("td", td(vec![], vec![text("x")])),
            ("caption", caption(vec![], vec![text("x")])),
            ("img", img(vec![], vec![text("x")])),
            ("br", br(vec![], vec![text("x")])),
            ("hr", hr(vec![], vec![text("x")])),
        ];
        for (tag, node) in cases {
            let expected = format!("<{tag}>x</{tag}>");
            assert_eq!(render(&node), expected, "tag={tag}");
        }
    }

    /// タグショートカットは `el()` への薄い委譲であるため、`el()` を直接
    /// 使った場合と出力が完全に一致することを確認する（`docs/api/component-api.md`
    /// 第 4 節・定義規則 1〜3 が求める「独自の出力経路を持たない」ことの回帰）。
    #[test]
    fn tag_shortcut_output_matches_direct_el_call() {
        let via_shortcut = div(vec![("class", "card")], vec![p(vec![], vec![])]);
        let via_el = el(
            "div",
            vec![("class", "card")],
            vec![el("p", vec![], vec![])],
        );
        assert_eq!(render(&via_shortcut), render(&via_el));

        // 拡張分の代表として table/tr/td でも同様の一致を確認する。
        let via_shortcut2 = table(vec![], vec![tr(vec![], vec![td(vec![], vec![])])]);
        let via_el2 = el(
            "table",
            vec![],
            vec![el("tr", vec![], vec![el("td", vec![], vec![])])],
        );
        assert_eq!(render(&via_shortcut2), render(&via_el2));
    }

    /// ショートカット経由でもテキスト・属性値の既定エスケープ（REQ-1）が
    /// 迂回されないことを XSS ペイロードで確認する。
    #[test]
    fn tag_shortcuts_escape_text_and_attrs_by_default() {
        let payload = "<script>alert(1)</script>";
        let node = div(
            vec![("title", "\"><script>alert(1)</script>")],
            vec![p(vec![], vec![text(payload)])],
        );
        let html = render(&node);
        assert!(
            !html.contains("<script>"),
            "ショートカット経由でエスケープが迂回された: {html}"
        );
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&quot;&gt;&lt;script&gt;"));
    }

    /// Issue #164 拡張分のショートカットでも同様にエスケープが迂回されない
    /// ことを、フォーム系・テーブル系の代表ケースで確認する。
    #[test]
    fn extended_tag_shortcuts_escape_text_and_attrs_by_default() {
        let payload = "<script>alert(1)</script>";
        let node = form(
            vec![("action", "\"><script>alert(1)</script>")],
            vec![
                label(vec![], vec![text(payload)]),
                table(
                    vec![],
                    vec![tr(vec![], vec![td(vec![], vec![text(payload)])])],
                ),
            ],
        );
        let html = render(&node);
        assert!(
            !html.contains("<script>"),
            "拡張ショートカット経由でエスケープが迂回された: {html}"
        );
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&quot;&gt;&lt;script&gt;"));
    }

    /// ショートカットと `raw_html` を組み合わせても、非エスケープ出力点が
    /// `raw_html` のみであるという境界（不変条件 2）が変わらないことを確認する。
    #[test]
    fn tag_shortcut_combined_with_raw_html_keeps_opt_in_boundary() {
        #[expect(
            clippy::disallowed_methods,
            reason = "ESCAPE-REVIEWED: タグショートカット併用時もオプトイン境界が変わらないことの検証。固定の信頼済み文字列のみ"
        )]
        let node = div(
            vec![],
            vec![text("<script>"), raw_html("<b>ok</b>"), text("<script>")],
        );
        let html = render(&node);
        assert_eq!(html, "<div>&lt;script&gt;<b>ok</b>&lt;script&gt;</div>");
    }

    /// void 要素ショートカット（`img`/`br`/`hr`/`input`）が、現行仕様どおり
    /// 常に終了タグを出力することを固定する（`docs/api/component-api.md` 第 3 節・
    /// 判断 4 の凍結仕様。将来の自己終端出力最適化はこのテストの更新を伴う）。
    #[test]
    fn void_element_shortcuts_render_closing_tag() {
        assert_eq!(render(&img(vec![], vec![])), "<img></img>");
        assert_eq!(render(&br(vec![], vec![])), "<br></br>");
        assert_eq!(render(&hr(vec![], vec![])), "<hr></hr>");
        assert_eq!(render(&input(vec![], vec![])), "<input></input>");
    }
}

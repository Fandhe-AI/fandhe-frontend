//! ページの外枠（`<!DOCTYPE html>` + `html`/`head`/`body`）を組み立てる [`Document`] builder。
//!
//! `fandhe-frontend-app` はモード非依存の純関数層であり、本モジュールも HTTP ヘッダ・
//! キャッシュ方針を持たない（それらは SSR ハンドラ側の責務）。`page_shell` が固定している
//! lang・CSS・script を、fandhe-backend の認証画面のように hydrate.js が不要・CSS の置き場所が
//! 異なる・厳格な CSP を掛ける呼び出し側が指定できるようにするために存在する。
//!
//! # セキュリティ不変条件
//!
//! - title / lang / href / src / head ノードのテキストはすべて `fandhe_frontend_core` の
//!   `el` / `text` 経由で既定エスケープされる（REQ-1）。独自エスケープ・独自 raw 経路は持たない。
//! - stylesheet の href と script の src は core の属性出力と同じ `is_safe_url`
//!   （許可リスト方式）で判定し、不合格なら **その要素ごと出力しない**（fail-closed）。
//! - `<script>` 本文（inline script）を渡す経路はなく、`src` 参照のみ。既定で inline `<style>` も
//!   出さないため厳格 CSP と両立する。
//! - 文字列でヘッドを差し込む API はなく、`Node` のみ受け付ける。

use fandhe_frontend_core::{el, is_safe_url, render, text, Node};

/// script の読み込み種別。bool 引数による取り違えを避けるため非公開 enum で保持する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScriptKind {
    /// `type="module"`
    Module,
    /// 通常の classic script
    Classic,
}

/// ページ外枠の builder。`render()` で DOCTYPE 付きの完全な HTML 文字列を返す。
///
/// head の出力順は `meta charset` → `meta viewport` → `title` → stylesheet（指定順）→
/// [`Document::head_node`]（指定順）。script は body 末尾（利用者 body の後）に指定順で出す。
/// `lang` の既定は `"ja"`（`page_shell` と同値）。値は既定エスケープのみで BCP 47 検証はしない。
///
/// HTTP ヘッダ（CSP・Cache-Control 等）は本型の責務外で、ハンドラ側で設定する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_app::Document;
/// use fandhe_frontend_core::{el, text};
///
/// let form = el("form", vec![("method", "post")], vec![text("Sign in")]);
/// let html = Document::new("Sign in", form)
///     .lang("en")
///     .stylesheet("/assets/auth.css")
///     .script_module("/assets/auth.js")
///     .render();
/// assert!(html.starts_with("<!DOCTYPE html>\n"));
/// assert!(html.contains("<title>Sign in</title>"));
/// assert!(html.contains("/assets/auth.css"));
/// assert!(html.contains("type=\"module\""));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    title: String,
    lang: String,
    body: Node,
    stylesheets: Vec<String>,
    scripts: Vec<(ScriptKind, String)>,
    head_nodes: Vec<Node>,
}

impl Document {
    /// タイトルと body 要素の内容から builder を作る（lang は `"ja"`）。
    pub fn new(title: impl Into<String>, body: Node) -> Self {
        Self {
            title: title.into(),
            lang: "ja".to_string(),
            body,
            stylesheets: Vec::new(),
            scripts: Vec::new(),
            head_nodes: Vec::new(),
        }
    }

    /// `html` 要素の `lang` を上書きする。
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = lang.into();
        self
    }

    /// `link rel="stylesheet"` を追加する（複数可・指定順）。
    /// 安全でない URL（`javascript:` 等）は要素ごと省略される。
    pub fn stylesheet(mut self, href: impl Into<String>) -> Self {
        self.stylesheets.push(href.into());
        self
    }

    /// `script type="module"` を body 末尾へ追加する（複数可・指定順）。
    /// 安全でない URL は要素ごと省略される。
    pub fn script_module(mut self, src: impl Into<String>) -> Self {
        self.scripts.push((ScriptKind::Module, src.into()));
        self
    }

    /// classic な `script` を body 末尾へ追加する（複数可・指定順）。
    /// 安全でない URL は要素ごと省略される。
    pub fn script(mut self, src: impl Into<String>) -> Self {
        self.scripts.push((ScriptKind::Classic, src.into()));
        self
    }

    /// head へ任意のノード（`meta` / `style` 等）を追加する（複数可・指定順）。
    ///
    /// 信頼できない入力を `raw_html` で包んで渡してはならない（利用者責任）。
    pub fn head_node(mut self, node: Node) -> Self {
        self.head_nodes.push(node);
        self
    }

    /// DOCTYPE 付きの完全な HTML 文字列を返す。同一入力に対し常に同一出力。
    ///
    /// 前置する `<!DOCTYPE html>\n` は利用者入力を含まない固定リテラルで、
    /// 新たなエスケープ迂回経路ではない。
    pub fn render(&self) -> String {
        let mut head = vec![
            el("meta", vec![("charset", "utf-8")], vec![]),
            el(
                "meta",
                vec![
                    ("name", "viewport"),
                    ("content", "width=device-width, initial-scale=1"),
                ],
                vec![],
            ),
            el("title", vec![], vec![text(&self.title)]),
        ];
        for href in self.stylesheets.iter().filter(|h| is_safe_url(h)) {
            head.push(el(
                "link",
                vec![("rel", "stylesheet"), ("href", href)],
                vec![],
            ));
        }
        head.extend(self.head_nodes.iter().cloned());

        let mut body_children = vec![self.body.clone()];
        for (kind, src) in self.scripts.iter().filter(|(_, s)| is_safe_url(s)) {
            let attrs = match kind {
                ScriptKind::Module => vec![("type", "module"), ("src", src.as_str())],
                ScriptKind::Classic => vec![("src", src.as_str())],
            };
            body_children.push(el("script", attrs, vec![]));
        }

        let html = el(
            "html",
            vec![("lang", self.lang.as_str())],
            vec![el("head", vec![], head), el("body", vec![], body_children)],
        );
        let mut out = String::from("<!DOCTYPE html>\n");
        out.push_str(&render(&html));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body() -> Node {
        el("p", vec![], vec![text("hi")])
    }

    #[test]
    fn minimal() {
        let s = Document::new("T", body()).render();
        assert_eq!(
            s,
            "<!DOCTYPE html>\n<html lang=\"ja\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>T</title></head><body><p>hi</p></body></html>"
        );
    }

    #[test]
    fn full_order_and_lang() {
        let s = Document::new("T", body())
            .lang("en")
            .stylesheet("/a.css")
            .stylesheet("/b.css")
            .head_node(el("style", vec![], vec![text("p{}")]))
            .script_module("/m.js")
            .script("/c.js")
            .render();
        assert!(s.contains("<html lang=\"en\">"));
        let a = s.find("/a.css").unwrap();
        let b = s.find("/b.css").unwrap();
        let st = s.find("<style>").unwrap();
        let t = s.find("<title>").unwrap();
        assert!(t < a && a < b && b < st);
        assert!(s.contains("<p>hi</p><script type=\"module\" src=\"/m.js\"></script><script src=\"/c.js\"></script></body>"));
    }

    #[test]
    fn deterministic() {
        let d = Document::new("T", body()).stylesheet("/a.css");
        assert_eq!(d.render(), d.render());
    }

    #[test]
    fn unsafe_urls_dropped_whole_element() {
        let s = Document::new("T", body())
            .stylesheet("javascript:alert(1)")
            .stylesheet("/ok.css")
            .script_module("data:text/javascript,alert(1)")
            .script("/ok.js")
            .render();
        assert!(!s.contains("javascript:"));
        assert!(!s.contains("data:"));
        assert_eq!(s.matches("<link").count(), 1);
        assert_eq!(s.matches("<script").count(), 1);
        assert!(s.contains("/ok.css") && s.contains("/ok.js"));
    }

    #[test]
    fn page_shell_output_unchanged() {
        let s = crate::page_shell("T", body());
        assert_eq!(
            s,
            "<!DOCTYPE html>\n<html lang=\"ja\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><style>@view-transition { navigation: auto; }</style><title>T</title><link rel=\"stylesheet\" href=\"/static/style.css\"></head><body><p>hi</p><script type=\"module\" src=\"/static/hydrate.js\"></script></body></html>"
        );
    }
}
